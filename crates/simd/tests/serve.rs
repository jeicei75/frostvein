use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

/// Every blocking read in this harness is bounded. A daemon that binds but never
/// writes, or never logs, must fail the test rather than hang `cargo test` forever.
const IO_TIMEOUT: Duration = Duration::from_secs(10);

struct Daemon {
    child: Child,
    address: SocketAddr,
    stderr: Receiver<String>,
}

/// Drains a child pipe on its own thread, so the daemon can never block writing to
/// a pipe nobody is reading, and so every read here can carry a deadline.
fn line_channel<R: Read + Send + 'static>(reader: R) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            match line {
                Ok(line) => {
                    if sender.send(line).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    receiver
}

impl Daemon {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_simd"))
            .arg("0")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("simd must start");
        let stdout = line_channel(child.stdout.take().expect("simd stdout must be piped"));
        let stderr = line_channel(child.stderr.take().expect("simd stderr must be piped"));

        let listening_line = match stdout.recv_timeout(IO_TIMEOUT) {
            Ok(line) => line,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("simd never printed its listening line: {error:?}");
            }
        };
        let port = listening_line
            .trim_end()
            .strip_prefix("listening on 127.0.0.1:")
            .expect("simd must print the expected listening line")
            .parse()
            .expect("simd must print a numeric port");

        Self {
            child,
            address: SocketAddr::from(([127, 0, 0, 1], port)),
            stderr,
        }
    }

    fn connect(&self) -> TcpStream {
        let stream = TcpStream::connect(self.address).expect("client must connect to simd");
        stream
            .set_read_timeout(Some(IO_TIMEOUT))
            .expect("read timeout must be settable");
        stream
    }

    /// Blocks until the daemon logs a line, proving it actually processed the input —
    /// without this, an inbound-handling test passes whether or not the daemon read.
    fn next_log(&self) -> String {
        match self.stderr.recv_timeout(IO_TIMEOUT) {
            Ok(line) => line,
            Err(RecvTimeoutError::Timeout) => panic!("daemon logged nothing within {IO_TIMEOUT:?}"),
            Err(RecvTimeoutError::Disconnected) => panic!("daemon stderr closed unexpectedly"),
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_snapshot(reader: &mut BufReader<TcpStream>) -> protocol::Snapshot {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("daemon must send a snapshot line");
    serde_json::from_str(&line).expect("snapshot line must match the protocol")
}

fn read_delta(reader: &mut BufReader<TcpStream>) -> protocol::Delta {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("daemon must send a delta line");
    serde_json::from_str(&line).expect("delta line must match the protocol")
}

#[test]
fn streams_three_strictly_increasing_deltas() {
    let daemon = Daemon::spawn();
    let mut reader = BufReader::new(daemon.connect());

    let snapshot = read_snapshot(&mut reader);
    assert_eq!(snapshot.tiles.len(), 524_288);
    assert_eq!(snapshot.entities.len(), 5);

    let ticks = [
        read_delta(&mut reader).tick,
        read_delta(&mut reader).tick,
        read_delta(&mut reader).tick,
    ];
    assert_eq!(
        ticks,
        [snapshot.tick + 1, snapshot.tick + 2, snapshot.tick + 3]
    );
}

#[test]
fn world_advances_before_any_client_connects() {
    let daemon = Daemon::spawn();
    thread::sleep(Duration::from_millis(350));

    let mut reader = BufReader::new(daemon.connect());
    let snapshot = read_snapshot(&mut reader);

    assert!(
        snapshot.tick >= 2,
        "late snapshot tick was {}",
        snapshot.tick
    );
}

/// AC7's *rate*, which no unit test can reach. `tick_period_is_exactly_ten_hertz` pins
/// the constant against its own literal; only a running daemon shows the loop actually
/// sleeps to it. Deleting the `thread::sleep` in `tick` leaves that unit test green and
/// this one red.
#[test]
fn deltas_arrive_at_roughly_ten_per_second() {
    const SAMPLES: u64 = 20;

    let daemon = Daemon::spawn();
    let mut reader = BufReader::new(daemon.connect());
    let _ = read_snapshot(&mut reader);

    let first = read_delta(&mut reader).tick;
    let started = Instant::now();
    let mut last = first;
    for _ in 0..SAMPLES {
        last = read_delta(&mut reader).tick;
    }
    let elapsed = started.elapsed();

    assert_eq!(
        last - first,
        SAMPLES,
        "one delta per tick, no gaps or repeats"
    );
    // 20 ticks at 10Hz is 2.0s. The window is wide enough to survive a loaded CI box
    // and narrow enough that a TICK_PERIOD wrong by 5x either way fails.
    assert!(
        elapsed >= Duration::from_millis(1200) && elapsed <= Duration::from_millis(4500),
        "{SAMPLES} deltas took {elapsed:?}, expected ~2s at 10Hz"
    );
}

#[test]
fn client_connecting_later_gets_current_snapshot_then_tracks() {
    let daemon = Daemon::spawn();
    let mut first = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut first);
    let first_tick = read_delta(&mut first).tick;
    assert!(first_tick > 0);

    let mut later = BufReader::new(daemon.connect());
    let snapshot = read_snapshot(&mut later);
    assert!(snapshot.tick > 0);
    let update = read_delta(&mut later);
    assert!(update.tick > snapshot.tick);
}

#[test]
fn half_closed_client_keeps_receiving() {
    let daemon = Daemon::spawn();
    let stream = daemon.connect();
    stream
        .shutdown(Shutdown::Write)
        .expect("client write half must close");
    let mut reader = BufReader::new(stream);

    let snapshot = read_snapshot(&mut reader);
    let update = read_delta(&mut reader);

    assert_eq!(update.tick, snapshot.tick + 1);
}

#[test]
fn dropped_client_does_not_stop_another_clients_stream() {
    let daemon = Daemon::spawn();
    let mut dropped = BufReader::new(daemon.connect());
    let mut survivor = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut dropped);
    let survivor_snapshot = read_snapshot(&mut survivor);
    let _: protocol::Delta = read_delta(&mut dropped);
    drop(dropped);

    let ticks = [
        read_delta(&mut survivor).tick,
        read_delta(&mut survivor).tick,
        read_delta(&mut survivor).tick,
    ];
    assert!(ticks[0] > survivor_snapshot.tick);
    assert!(ticks[0] < ticks[1] && ticks[1] < ticks[2], "{ticks:?}");
}

#[test]
fn malformed_input_is_dropped_and_daemon_survives() {
    let daemon = Daemon::spawn();
    let mut first = BufReader::new(daemon.connect());

    let _: protocol::Snapshot = read_snapshot(&mut first);
    first.get_mut().write_all(b"not json\n").unwrap();
    first
        .get_mut()
        .write_all(b"{\"type\":\"bogus\"}\n")
        .unwrap();

    // AC6 requires the daemon to LOG the unrecognized input, not merely tolerate it.
    // Waiting for both log lines also synchronises: without it this test could pass
    // on the second connection alone, having never proven the first was read.
    assert_eq!(daemon.next_log(), "unrecognized client message: not json");
    assert_eq!(
        daemon.next_log(),
        "unrecognized client message: {\"type\":\"bogus\"}"
    );

    drop(first);

    let mut second = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut second);
}

#[test]
fn non_utf8_input_does_not_close_the_connection() {
    let daemon = Daemon::spawn();
    let mut client = BufReader::new(daemon.connect());

    let _: protocol::Snapshot = read_snapshot(&mut client);
    client.get_mut().write_all(b"\xff\xfe\n").unwrap();
    assert!(
        daemon
            .next_log()
            .starts_with("unrecognized client message:"),
        "a non-UTF-8 line must be logged and dropped like any other unrecognized input"
    );

    let update = read_delta(&mut client);
    assert!(update.tick > 0);
}

#[test]
fn oversized_line_is_refused_without_killing_the_daemon() {
    let daemon = Daemon::spawn();
    let mut client = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut client);

    // No newline: without a cap the daemon would buffer this until it ran out of memory.
    let flood = vec![b'a'; 128 * 1024];
    let _ = client.get_mut().write_all(&flood);
    assert!(
        daemon.next_log().contains("exceeded"),
        "daemon must refuse an unbounded line rather than buffer it"
    );

    let mut next = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut next);
}

#[test]
fn client_disconnect_does_not_kill_daemon() {
    let daemon = Daemon::spawn();

    drop(daemon.connect());

    let mut next = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut next);
}

#[test]
fn client_disconnect_mid_snapshot_does_not_kill_daemon() {
    let daemon = Daemon::spawn();

    // Read a fraction of the ~6.9 MB line, then vanish — this is the case where
    // write_all is mid-flight and the partial-write/BrokenPipe path actually runs.
    let mut early = daemon.connect();
    let mut buffer = vec![0u8; 64 * 1024];
    let prefix = early.read(&mut buffer).expect("client must read a prefix");
    assert!(prefix > 0, "client must receive part of the snapshot");
    drop(early);

    let mut next = BufReader::new(daemon.connect());
    let snapshot = read_snapshot(&mut next);
    assert_eq!(snapshot.tiles.len(), 524_288);
}
