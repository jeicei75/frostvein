use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{SocketAddr, TcpStream},
    process::{Child, Command, Stdio},
    time::Duration,
};

struct Daemon {
    child: Child,
    address: SocketAddr,
}

impl Daemon {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_simd"))
            .arg("0")
            .stdout(Stdio::piped())
            .spawn()
            .expect("simd must start");
        let stdout = child.stdout.take().expect("simd stdout must be piped");
        let mut reader = BufReader::new(stdout);
        let mut listening_line = String::new();
        reader
            .read_line(&mut listening_line)
            .expect("simd must print its listening address");
        let port = listening_line
            .trim_end()
            .strip_prefix("listening on 127.0.0.1:")
            .expect("simd must print the expected listening line")
            .parse()
            .expect("simd must print a numeric port");

        Self {
            child,
            address: SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }

    fn connect(&self) -> TcpStream {
        TcpStream::connect(self.address).expect("client must connect to simd")
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

#[test]
fn snapshot_on_connect_and_nothing_more() {
    let daemon = Daemon::spawn();
    let mut reader = BufReader::new(daemon.connect());

    let snapshot = read_snapshot(&mut reader);
    assert_eq!(snapshot.tiles.len(), 524_288);
    assert_eq!(snapshot.entities.len(), 5);

    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let mut next_line = String::new();
    match reader.read_line(&mut next_line) {
        Err(error) => assert!(
            matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut),
            "expected a read timeout, got {error:?}"
        ),
        Ok(0) => panic!("daemon closed the connection instead of leaving it idle"),
        Ok(_) => panic!("daemon sent an unexpected second line: {next_line:?}"),
    }
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
    drop(first);

    let mut second = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut second);
}

#[test]
fn client_disconnect_does_not_kill_daemon() {
    let daemon = Daemon::spawn();

    drop(daemon.connect());

    let mut next = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut next);
}
