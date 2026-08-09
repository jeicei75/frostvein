use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant},
};

/// Every blocking read in this harness is bounded. A daemon that binds but never
/// writes, or never logs, must fail the test rather than hang `cargo test` forever.
const IO_TIMEOUT: Duration = Duration::from_secs(10);
static DAEMON_TEST_LOCK: Mutex<()> = Mutex::new(());

struct Daemon {
    _serial: MutexGuard<'static, ()>,
    child: Child,
    address: SocketAddr,
    stderr: Receiver<String>,
    dir: PathBuf,
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
        // These tests assert wall-clock cadence and command visibility within a small number of
        // ticks. Running dozens of daemon processes concurrently lets queued startup deltas make
        // those assertions measure scheduler contention instead of the daemon under test.
        let serial = DAEMON_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // NOTE: let the daemon bind port 0 itself rather than reserving a port here and passing
        // the number on. Reserving meant dropping the listener before the child could bind it,
        // and with this many daemon tests running in parallel another test claimed that port in
        // the gap often enough to turn the gate red about one run in four. The listening line is
        // parsed below either way, so nothing here needs to know the port in advance.
        static NEXT_DAEMON: AtomicUsize = AtomicUsize::new(0);
        let unique = NEXT_DAEMON.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("frostvein-simd-{}-{unique}", std::process::id()));
        fs::create_dir(&dir).expect("create unique daemon working directory");

        let mut child = Command::new(env!("CARGO_BIN_EXE_simd"))
            .arg("0")
            .current_dir(&dir)
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
            _serial: serial,
            child,
            address: SocketAddr::from(([127, 0, 0, 1], port)),
            stderr,
            dir,
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

    fn save_path(&self) -> PathBuf {
        self.dir.join("frostvein.save")
    }

    fn wait_for_exit(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + IO_TIMEOUT;
        loop {
            match self.child.try_wait().expect("query simd exit status") {
                Some(status) => return status,
                None if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                None => panic!("simd did not exit within {IO_TIMEOUT:?}"),
            }
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.dir);
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

fn send_speed(stream: &mut TcpStream, speed: protocol::Speed) {
    let line = match speed {
        protocol::Speed::Paused => b"{\"type\":\"set_speed\",\"speed\":\"paused\"}\n".as_slice(),
        protocol::Speed::Normal => b"{\"type\":\"set_speed\",\"speed\":\"normal\"}\n".as_slice(),
        protocol::Speed::Fast => b"{\"type\":\"set_speed\",\"speed\":\"fast\"}\n".as_slice(),
    };
    stream.write_all(line).expect("speed command must write");
    stream.flush().expect("speed command must flush");
}

fn send_literal(stream: &mut TcpStream, line: &[u8]) {
    stream.write_all(line).expect("command must write");
    stream.flush().expect("command must flush");
}

fn parse_saved_tick(line: &str) -> u64 {
    line.strip_prefix("saved tick ")
        .and_then(|rest| rest.split_once(" to "))
        .expect("save log must name its tick and path")
        .0
        .parse()
        .expect("saved tick must be numeric")
}

fn read_snapshot_after_load(reader: &mut BufReader<TcpStream>) -> protocol::Snapshot {
    for _ in 0..4 {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("daemon must keep streaming after load");
        let value: serde_json::Value = serde_json::from_str(&line).expect("valid daemon line");
        match value["type"].as_str() {
            Some("snapshot") => {
                return serde_json::from_value(value).expect("load snapshot must match protocol");
            }
            Some("delta") => {}
            other => panic!("unexpected daemon message before load snapshot: {other:?}"),
        }
    }
    panic!("daemon did not broadcast a snapshot within four lines");
}

fn read_save_state(path: &Path) -> sim_core::SaveState {
    const MAX_TEST_SAVE_BYTES: u64 = 16 * 1024 * 1024;

    let file = fs::File::open(path).expect("saved file must exist");
    let mut bytes = Vec::new();
    file.take(MAX_TEST_SAVE_BYTES + 1)
        .read_to_end(&mut bytes)
        .expect("saved file must be readable");
    assert!(
        bytes.len() as u64 <= MAX_TEST_SAVE_BYTES,
        "saved file exceeded the test read bound"
    );
    serde_json::from_slice(&bytes).expect("saved file must decode as SaveState")
}

fn assert_save_is_rejected_without_stopping_ticks(state: sim_core::SaveState, expected: &str) {
    let daemon = Daemon::spawn();
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode invalid save fixture"),
    )
    .expect("write invalid save fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(log.contains(expected), "unexpected invalid-save log: {log}");

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

fn read_delta_with_speed(
    reader: &mut BufReader<TcpStream>,
    expected: protocol::Speed,
) -> protocol::Delta {
    let mut observed = Vec::new();
    for _ in 0..50 {
        let update = read_delta(reader);
        observed.push((update.tick, update.speed));
        if update.speed == expected {
            return update;
        }
    }
    panic!("daemon never reported {expected:?}; observed {observed:?}");
}

fn read_delta_with_marks(
    reader: &mut BufReader<TcpStream>,
    designations: &[protocol::Designation],
    zones: &[protocol::Zone],
) -> protocol::Delta {
    let mut observed = Vec::new();
    for _ in 0..10 {
        let update = read_delta(reader);
        observed.push((
            update.tick,
            update.designations.clone(),
            update.zones.clone(),
        ));
        if update.designations == designations && update.zones == zones {
            return update;
        }
    }
    panic!("daemon never emitted expected marks; observed {observed:?}");
}

#[test]
fn save_then_load_rewinds_every_client() {
    let daemon = Daemon::spawn();
    let first_stream = daemon.connect();
    let mut first_writer = first_stream
        .try_clone()
        .expect("first client write half must clone");
    let mut first = BufReader::new(first_stream);
    let mut second = BufReader::new(daemon.connect());
    let _ = read_snapshot(&mut first);
    let _ = read_snapshot(&mut second);

    send_literal(&mut first_writer, b"{\"type\":\"save\"}\n");
    let saved_tick = parse_saved_tick(&daemon.next_log());

    let target = saved_tick + 10;
    let mut first_tick = 0;
    let mut second_tick = 0;
    while first_tick <= target || second_tick <= target {
        if first_tick <= target {
            first_tick = read_delta(&mut first).tick;
        }
        if second_tick <= target {
            second_tick = read_delta(&mut second).tick;
        }
    }

    send_literal(&mut first_writer, b"{\"type\":\"load\"}\n");
    let first_loaded = read_snapshot_after_load(&mut first);
    let second_loaded = read_snapshot_after_load(&mut second);

    assert_eq!(first_loaded.tick, saved_tick);
    assert_eq!(second_loaded.tick, saved_tick);
    assert!(first_loaded.tick < first_tick);
    assert!(second_loaded.tick < second_tick);
}

#[test]
fn saved_file_decodes_as_a_save_state() {
    let daemon = Daemon::spawn();
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let _ = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"save\"}\n");
    let saved_tick = parse_saved_tick(&daemon.next_log());
    let state = read_save_state(&daemon.save_path());

    assert_eq!(state.tick, saved_tick);
}

#[test]
fn load_without_a_save_file_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.starts_with("could not load frostvein.save:"),
        "unexpected load error log: {log}"
    );

    let ticks = [
        read_delta(&mut reader).tick,
        read_delta(&mut reader).tick,
        read_delta(&mut reader).tick,
    ];
    assert!(ticks[0] > snapshot.tick);
    assert!(ticks[0] < ticks[1] && ticks[1] < ticks[2], "{ticks:?}");
}

#[test]
fn undecodable_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    fs::write(daemon.save_path(), b"not json").expect("write corrupt save fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("could not decode frostvein.save"),
        "unexpected corrupt-save log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn inconsistent_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.tiles.pop();
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode inconsistent save fixture"),
    )
    .expect("write inconsistent save fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save has 524287 tiles but dims 128x128x32 need 524288"),
        "unexpected inconsistent-save log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn duplicate_dwarf_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.dwarves[1].id = state.dwarves[0].id;
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode duplicate-id save fixture"),
    )
    .expect("write duplicate-id save fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save reuses dwarf id 0"),
        "unexpected duplicate-id log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn boundary_tick_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.tick = u64::MAX - 1;
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode boundary-tick save fixture"),
    )
    .expect("write boundary-tick save fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains(
            "save tick 18446744073709551614 exceeds supported maximum 9223372036854775807"
        ),
        "unexpected boundary-tick log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn out_of_bounds_dwarf_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.dwarves[0].pos.x = i32::MAX;
    state.dwarves[0].cooldown = 0;
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode out-of-bounds dwarf save fixture"),
    )
    .expect("write out-of-bounds dwarf save fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save dwarf 0 position 2147483647,65,25 is outside dims 128x128x32"),
        "unexpected out-of-bounds dwarf log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn out_of_bounds_dwarf_home_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.dwarves[0].home.x = i32::MIN;
    state.dwarves[0].cooldown = 0;
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode out-of-bounds dwarf-home fixture"),
    )
    .expect("write out-of-bounds dwarf-home fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save dwarf 0 home -2147483648,65,25 is outside dims 128x128x32"),
        "unexpected out-of-bounds dwarf-home log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn out_of_bounds_designation_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.designations.push((
        sim_core::Pos { x: -1, y: 0, z: 0 },
        sim_core::DesignationKind::Dig,
    ));
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode out-of-bounds designation fixture"),
    )
    .expect("write out-of-bounds designation fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save designation position -1,0,0 is outside dims 128x128x32"),
        "unexpected out-of-bounds designation log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn over_budget_designation_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.designations = (0..=4096)
        .map(|index| {
            (
                sim_core::Pos {
                    x: index % 128,
                    y: index / 128,
                    z: 0,
                },
                sim_core::DesignationKind::Dig,
            )
        })
        .collect();

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save has 4097 designations; limit is 4096",
    );
}

#[test]
fn over_budget_job_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.jobs = (0..=4096)
        .map(|index| sim_core::Job {
            id: sim_core::JobId(index),
            kind: sim_core::JobKind::Dig,
            target: sim_core::Pos {
                x: (index % 128) as i32,
                y: (index / 128) as i32,
                z: 0,
            },
            created_tick: 0,
            retry_after: 0,
        })
        .collect();
    state.next_job_id = 4097;

    assert_save_is_rejected_without_stopping_ticks(state, "save has 4097 jobs; limit is 4096");
}

#[test]
fn out_of_bounds_job_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.jobs.push(sim_core::Job {
        id: sim_core::JobId(0),
        kind: sim_core::JobKind::Dig,
        target: sim_core::Pos { x: -1, y: 0, z: 0 },
        created_tick: 0,
        retry_after: 0,
    });
    state.next_job_id = 1;
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode out-of-bounds job fixture"),
    )
    .expect("write out-of-bounds job fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save job target -1,0,0 is outside dims 128x128x32"),
        "unexpected out-of-bounds job log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn out_of_bounds_item_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.items.push((10, sim_core::Pos { x: 0, y: -1, z: 0 }));
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode out-of-bounds item fixture"),
    )
    .expect("write out-of-bounds item fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save item 10 position 0,-1,0 is outside dims 128x128x32"),
        "unexpected out-of-bounds item log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn duplicate_item_entity_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state
        .items
        .push((state.dwarves[0].id, state.dwarves[0].pos));

    assert_save_is_rejected_without_stopping_ticks(state, "save reuses entity id 0");
}

#[test]
fn duplicate_emitter_entity_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.emitters[0].0 = state.dwarves[0].id;

    assert_save_is_rejected_without_stopping_ticks(state, "save reuses entity id 0");
}

#[test]
fn emitter_id_at_next_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.emitters[0].0 = state.next_id;

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save next_id 10 does not exceed entity id 10",
    );
}

#[test]
fn item_id_at_next_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.items.push((state.next_id, state.dwarves[0].pos));

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save next_id 10 does not exceed entity id 10",
    );
}

#[test]
fn exhausted_next_entity_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.next_id = u32::MAX;

    assert_save_is_rejected_without_stopping_ticks(state, "save next_id 4294967295 is exhausted");
}

#[test]
fn duplicate_job_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.jobs = vec![
        sim_core::Job {
            id: sim_core::JobId(0),
            kind: sim_core::JobKind::Dig,
            target: sim_core::Pos { x: 20, y: 20, z: 8 },
            created_tick: 0,
            retry_after: 0,
        },
        sim_core::Job {
            id: sim_core::JobId(0),
            kind: sim_core::JobKind::Channel,
            target: sim_core::Pos { x: 21, y: 20, z: 8 },
            created_tick: 0,
            retry_after: 0,
        },
    ];
    state.next_job_id = 1;

    assert_save_is_rejected_without_stopping_ticks(state, "save reuses job id 0");
}

#[test]
fn job_without_matching_designation_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.jobs.push(sim_core::Job {
        id: sim_core::JobId(0),
        kind: sim_core::JobKind::Dig,
        target: sim_core::Pos { x: 20, y: 20, z: 8 },
        created_tick: 0,
        retry_after: 0,
    });
    state.next_job_id = 1;

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save job 0 has no matching dig designation at 20,20,8",
    );
}

#[test]
fn job_with_mismatched_designation_kind_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    let target = sim_core::Pos { x: 20, y: 20, z: 8 };
    state
        .designations
        .push((target, sim_core::DesignationKind::Channel));
    state.jobs.push(sim_core::Job {
        id: sim_core::JobId(0),
        kind: sim_core::JobKind::Dig,
        target,
        created_tick: 0,
        retry_after: 0,
    });
    state.next_job_id = 1;

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save job 0 has no matching dig designation at 20,20,8",
    );
}

#[test]
fn duplicate_job_target_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    let target = sim_core::Pos { x: 20, y: 20, z: 8 };
    state.jobs = vec![
        sim_core::Job {
            id: sim_core::JobId(0),
            kind: sim_core::JobKind::Dig,
            target,
            created_tick: 0,
            retry_after: 0,
        },
        sim_core::Job {
            id: sim_core::JobId(1),
            kind: sim_core::JobKind::Channel,
            target,
            created_tick: 0,
            retry_after: 0,
        },
    ];
    state.next_job_id = 2;

    assert_save_is_rejected_without_stopping_ticks(state, "save reuses job target 20,20,8");
}

/// A save with `count` stones, all parked on dwarf zero's tile, and `next_id` moved past them.
fn save_with_items(count: u32) -> sim_core::SaveState {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    let pos = state.dwarves[0].pos;
    state.items = (0..count).map(|index| (10 + index, pos)).collect();
    state.next_id = 10 + count;
    state
}

fn haul_job(id: u32, item: u32, target: sim_core::Pos) -> sim_core::Job {
    sim_core::Job {
        id: sim_core::JobId(id),
        kind: sim_core::JobKind::Haul { item },
        target,
        created_tick: 0,
        retry_after: 0,
    }
}

/// The positive half of the haul load rules: a haul job has no designation and never had one, so
/// the tile-job rules must not be applied to it. Without this the whole gauntlet could reject
/// every legitimate mid-haul save and only the rejection tests below would notice — they would
/// all still pass.
#[test]
fn a_mid_haul_save_loads_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = save_with_items(1);
    let pos = state.dwarves[0].pos;
    state.jobs = vec![haul_job(0, 10, pos)];
    state.next_job_id = 1;
    state.dwarves[0].current_job = Some(0);
    state.dwarves[0].carrying = Some(10);
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode mid-haul save"),
    )
    .expect("write mid-haul save");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let _ = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let loaded = read_snapshot_after_load(&mut reader);

    assert!(loaded.items.iter().any(|item| item.id == 10));
    assert_eq!(loaded.tick, state.tick);
    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(first < second, "the loaded world stopped ticking");
}

#[test]
fn haul_job_naming_an_absent_item_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = save_with_items(1);
    let target = state.dwarves[0].pos;
    state.jobs = vec![haul_job(0, 99, target)];
    state.next_job_id = 1;

    assert_save_is_rejected_without_stopping_ticks(state, "save haul job 0 names missing item 99");
}

#[test]
fn two_haul_jobs_on_one_item_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = save_with_items(2);
    let target = state.dwarves[0].pos;
    state.jobs = vec![haul_job(0, 10, target), haul_job(1, 10, target)];
    state.next_job_id = 2;

    assert_save_is_rejected_without_stopping_ticks(state, "save reuses haul item 10");
}

/// Pins the LOG LINE, not an outcome bound: unique-by-item and item-exists already reject this
/// save, so what is asserted here is that the count complaint is the one that fires first. Raised
/// at 3.3's review, where this test read as stronger than it is.
#[test]
fn more_haul_jobs_than_items_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = save_with_items(1);
    let target = state.dwarves[0].pos;
    state.jobs = vec![haul_job(0, 10, target), haul_job(1, 11, target)];
    state.next_job_id = 2;

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save has 2 haul jobs; limit is 1 item(s)",
    );
}

#[test]
fn dwarf_carrying_an_absent_item_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.dwarves[0].carrying = Some(99);

    assert_save_is_rejected_without_stopping_ticks(state, "save dwarf 0 carries missing item 99");
}

#[test]
fn two_dwarves_carrying_one_item_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = save_with_items(1);
    let target = state.dwarves[0].pos;
    state.jobs = vec![haul_job(0, 10, target)];
    state.next_job_id = 1;
    state.dwarves[0].carrying = Some(10);
    state.dwarves[0].current_job = Some(0);
    // No claim of its own: `save job 0 has multiple claimants` would otherwise fire first and
    // this test would prove that rule instead of this one.
    state.dwarves[1].carrying = Some(10);

    assert_save_is_rejected_without_stopping_ticks(state, "save item 10 has multiple carriers");
}

#[test]
fn carrying_dwarf_without_the_matching_haul_job_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = save_with_items(1);
    let target = state.dwarves[0].pos;
    state.jobs = vec![haul_job(0, 10, target)];
    state.next_job_id = 1;
    state.dwarves[0].carrying = Some(10);

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save dwarf 0 carries item 10 without holding its haul job",
    );
}

#[test]
fn job_id_at_next_job_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.jobs = vec![sim_core::Job {
        id: sim_core::JobId(1),
        kind: sim_core::JobKind::Dig,
        target: sim_core::Pos { x: 20, y: 20, z: 8 },
        created_tick: 0,
        retry_after: 0,
    }];
    state.next_job_id = 1;

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save next_job_id 1 does not exceed job id 1",
    );
}

#[test]
fn exhausted_next_job_id_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.next_job_id = u32::MAX;

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save next_job_id 4294967295 is exhausted",
    );
}

#[test]
fn missing_claimed_job_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.dwarves[0].current_job = Some(99);

    assert_save_is_rejected_without_stopping_ticks(state, "save dwarf 0 claims missing job 99");
}

#[test]
fn multiply_claimed_job_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.jobs.push(sim_core::Job {
        id: sim_core::JobId(0),
        kind: sim_core::JobKind::Dig,
        target: sim_core::Pos { x: 20, y: 20, z: 8 },
        created_tick: 0,
        retry_after: 0,
    });
    state.next_job_id = 1;
    state.dwarves[0].current_job = Some(0);
    state.dwarves[1].current_job = Some(0);

    assert_save_is_rejected_without_stopping_ticks(state, "save job 0 has multiple claimants");
}

#[test]
fn overflowing_work_progress_save_is_logged_and_the_daemon_keeps_ticking() {
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.jobs.push(sim_core::Job {
        id: sim_core::JobId(0),
        kind: sim_core::JobKind::Dig,
        target: sim_core::Pos { x: 20, y: 20, z: 8 },
        created_tick: 0,
        retry_after: 0,
    });
    state.next_job_id = 1;
    state.dwarves[0].current_job = Some(0);
    state.dwarves[0].work_progress = u32::MAX;

    assert_save_is_rejected_without_stopping_ticks(
        state,
        "save dwarf 0 work progress 4294967295 exceeds 5",
    );
}

#[test]
fn out_of_bounds_zone_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    let mut state = sim_core::World::generate(42, sim_core::Dims::DEFAULT).to_save();
    state.zones.push(sim_core::Pos {
        x: i32::MAX,
        y: 0,
        z: 0,
    });
    fs::write(
        daemon.save_path(),
        serde_json::to_vec(&state).expect("encode out-of-bounds zone fixture"),
    )
    .expect("write out-of-bounds zone fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save zone position 2147483647,0,0 is outside dims 128x128x32"),
        "unexpected out-of-bounds zone log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn oversized_save_is_logged_and_the_daemon_keeps_ticking() {
    // Tracks `MAX_SAVE_BYTES`, raised from 16 MB to 64 MB at story 3.1's review because a legal
    // whole-world designation encodes to ~23.2 MB and made the world unsaveable. If this fixture
    // drifts below the real cap the test stops exercising the refusal and silently checks the
    // JSON decoder instead — which is exactly how it failed when the cap moved.
    const OVERSIZED: usize = 64 * 1024 * 1024 + 1;

    let daemon = Daemon::spawn();
    fs::write(daemon.save_path(), vec![b' '; OVERSIZED]).expect("write oversized save fixture");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"load\"}\n");
    let log = daemon.next_log();
    assert!(
        log.contains("save exceeds 67108864-byte limit"),
        "unexpected oversized-save log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn unwritable_save_is_logged_and_the_daemon_keeps_ticking() {
    let daemon = Daemon::spawn();
    fs::create_dir(daemon.dir.join("frostvein.save.tmp"))
        .expect("make the temporary save path unwritable as a file");
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"save\"}\n");
    let log = daemon.next_log();
    assert!(
        log.starts_with("could not save frostvein.save:"),
        "unexpected save error log: {log}"
    );

    let first = read_delta(&mut reader).tick;
    let second = read_delta(&mut reader).tick;
    assert!(snapshot.tick < first && first < second);
}

#[test]
fn quit_exits_the_daemon_cleanly() {
    let mut daemon = Daemon::spawn();
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let _ = read_snapshot(&mut reader);

    send_literal(&mut writer, b"{\"type\":\"quit\"}\n");
    assert_eq!(daemon.next_log(), "shutting down on client quit");
    let status = daemon.wait_for_exit();
    assert!(status.success(), "simd exited with {status}");

    let deadline = Instant::now() + IO_TIMEOUT;
    let mut buffer = [0u8; 8192];
    loop {
        match reader.read(&mut buffer).expect("read client socket to EOF") {
            0 => break,
            _ if Instant::now() < deadline => {}
            _ => panic!("client socket did not reach EOF within {IO_TIMEOUT:?}"),
        }
    }
}

#[test]
fn streams_three_strictly_increasing_deltas() {
    let daemon = Daemon::spawn();
    let mut reader = BufReader::new(daemon.connect());

    let snapshot = read_snapshot(&mut reader);
    assert_eq!(snapshot.tiles.len(), 524_288);
    assert_eq!(snapshot.entities.len(), 10);

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
fn paused_daemon_freezes_tick_and_entities_then_resumes() {
    let daemon = Daemon::spawn();
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);

    let _ = read_snapshot(&mut reader);
    let _ = read_delta(&mut reader);
    let normal_started = Instant::now();
    for _ in 0..9 {
        let _ = read_delta(&mut reader);
    }
    let normal_span = normal_started.elapsed();
    send_speed(&mut writer, protocol::Speed::Paused);

    let first_paused = read_delta_with_speed(&mut reader, protocol::Speed::Paused);
    let paused_started = Instant::now();
    let mut updates = vec![first_paused];
    updates.extend((1..10).map(|_| read_delta(&mut reader)));
    let paused_span = paused_started.elapsed();
    let frozen_tick = updates[0].tick;
    let frozen_positions: Vec<_> = updates[0]
        .entities
        .iter()
        .map(|entity| (entity.id, entity.pos))
        .collect();
    assert!(
        updates
            .iter()
            .all(|update| update.speed == protocol::Speed::Paused),
        "every observed delta after the command must acknowledge paused: {:?}",
        updates
            .iter()
            .map(|update| (update.tick, update.speed))
            .collect::<Vec<_>>()
    );
    assert!(
        updates.iter().all(|update| update.tick == frozen_tick),
        "paused ticks changed: {:?}",
        updates.iter().map(|update| update.tick).collect::<Vec<_>>()
    );
    assert!(updates.iter().all(|update| {
        update
            .entities
            .iter()
            .map(|entity| (entity.id, entity.pos))
            .collect::<Vec<_>>()
            == frozen_positions
    }));
    assert!(
        paused_span > normal_span / 2 && paused_span < normal_span * 2,
        "paused span {paused_span:?} did not keep normal cadence relative to {normal_span:?}"
    );

    send_speed(&mut writer, protocol::Speed::Normal);
    let resumed = read_delta_with_speed(&mut reader, protocol::Speed::Normal);
    assert_eq!(resumed.speed, protocol::Speed::Normal);
    assert_eq!(resumed.tick, frozen_tick + 1);
}

#[test]
fn client_connecting_while_paused_receives_paused_snapshot() {
    let daemon = Daemon::spawn();
    let first_stream = daemon.connect();
    let mut writer = first_stream
        .try_clone()
        .expect("client write half must clone");
    let mut first = BufReader::new(first_stream);
    let _ = read_snapshot(&mut first);
    let _ = read_delta(&mut first);

    send_speed(&mut writer, protocol::Speed::Paused);
    let paused = read_delta_with_speed(&mut first, protocol::Speed::Paused);

    let mut second = BufReader::new(daemon.connect());
    let snapshot = read_snapshot(&mut second);
    assert_eq!(snapshot.speed, protocol::Speed::Paused);
    assert_eq!(snapshot.tick, paused.tick);
}

#[test]
fn speed_change_from_either_client_reaches_both_on_the_same_delta() {
    let daemon = Daemon::spawn();
    let first_stream = daemon.connect();
    let mut first = BufReader::new(first_stream);
    let _ = read_snapshot(&mut first);
    let mut first_current = read_delta(&mut first);

    let second_stream = daemon.connect();
    let mut second_writer = second_stream
        .try_clone()
        .expect("second client write half must clone");
    let mut second = BufReader::new(second_stream);
    let _ = read_snapshot(&mut second);
    let second_current = read_delta(&mut second);
    while first_current.tick < second_current.tick {
        first_current = read_delta(&mut first);
    }
    assert_eq!(first_current.tick, second_current.tick);

    send_speed(&mut second_writer, protocol::Speed::Paused);
    let second_update = read_delta_with_speed(&mut second, protocol::Speed::Paused);
    let first_update = read_delta_with_speed(&mut first, protocol::Speed::Paused);

    assert_eq!(first_update.speed, protocol::Speed::Paused);
    assert_eq!(second_update.speed, protocol::Speed::Paused);
    assert_eq!(first_update.tick, second_update.tick);
}

#[test]
fn designation_and_stockpile_changes_reach_both_clients() {
    let daemon = Daemon::spawn();
    let first_stream = daemon.connect();
    let mut writer = first_stream
        .try_clone()
        .expect("first client write half must clone");
    let mut first = BufReader::new(first_stream);
    let first_snapshot = read_snapshot(&mut first);
    let mut second = BufReader::new(daemon.connect());
    let _ = read_snapshot(&mut second);

    send_literal(
        &mut writer,
        b"{\"type\":\"designate\",\"kind\":\"dig\",\"rect\":{\"min\":[1,2,3],\"max\":[2,3,3]}}\n",
    );
    let designated = vec![
        protocol::Designation {
            pos: [1, 2, 3],
            kind: protocol::DesignationKind::Dig,
        },
        protocol::Designation {
            pos: [1, 3, 3],
            kind: protocol::DesignationKind::Dig,
        },
        protocol::Designation {
            pos: [2, 2, 3],
            kind: protocol::DesignationKind::Dig,
        },
        protocol::Designation {
            pos: [2, 3, 3],
            kind: protocol::DesignationKind::Dig,
        },
    ];
    let _ = read_delta_with_marks(&mut first, &designated, &[]);
    let _ = read_delta_with_marks(&mut second, &designated, &[]);

    let stockpile_pos = first_snapshot.entities[0].pos;
    let stockpile = format!(
        "{{\"type\":\"place_stockpile\",\"rect\":{{\"min\":{stockpile_pos:?},\"max\":{stockpile_pos:?}}}}}\n"
    );
    send_literal(&mut writer, stockpile.as_bytes());
    let zones = vec![protocol::Zone { pos: stockpile_pos }];
    let _ = read_delta_with_marks(&mut first, &designated, &zones);
    let _ = read_delta_with_marks(&mut second, &designated, &zones);

    send_literal(
        &mut writer,
        b"{\"type\":\"cancel_designation\",\"rect\":{\"min\":[1,2,3],\"max\":[1,3,3]}}\n",
    );
    let remaining = vec![
        protocol::Designation {
            pos: [2, 2, 3],
            kind: protocol::DesignationKind::Dig,
        },
        protocol::Designation {
            pos: [2, 3, 3],
            kind: protocol::DesignationKind::Dig,
        },
    ];
    let _ = read_delta_with_marks(&mut first, &remaining, &zones);
    let _ = read_delta_with_marks(&mut second, &remaining, &zones);

    let remove = format!(
        "{{\"type\":\"remove_stockpile\",\"rect\":{{\"min\":{stockpile_pos:?},\"max\":{stockpile_pos:?}}}}}\n"
    );
    send_literal(&mut writer, remove.as_bytes());
    let _ = read_delta_with_marks(&mut first, &remaining, &[]);
    let _ = read_delta_with_marks(&mut second, &remaining, &[]);
}

#[test]
fn designation_is_applied_while_tick_is_paused() {
    let daemon = Daemon::spawn();
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);
    let designation_pos = snapshot.entities[0].pos;

    send_speed(&mut writer, protocol::Speed::Paused);
    let first_paused = read_delta_with_speed(&mut reader, protocol::Speed::Paused);
    let second_paused = read_delta(&mut reader);
    assert_eq!(second_paused.tick, first_paused.tick);
    let frozen_tick = first_paused.tick;

    let command = format!(
        "{{\"type\":\"designate\",\"kind\":\"channel\",\"rect\":{{\"min\":{designation_pos:?},\"max\":{designation_pos:?}}}}}\n"
    );
    send_literal(&mut writer, command.as_bytes());
    let expected = [protocol::Designation {
        pos: designation_pos,
        kind: protocol::DesignationKind::Channel,
    }];
    let mut carrying = None;
    for _ in 0..10 {
        let update = read_delta(&mut reader);
        assert_eq!(
            update.tick, frozen_tick,
            "every delta from pause onward must carry the frozen tick"
        );
        if update.designations == expected {
            carrying = Some(update);
            break;
        }
    }
    assert!(carrying.is_some(), "paused daemon discarded designation");
}

#[test]
fn completed_dig_streams_dirty_tile_and_item_in_the_same_delta() {
    let daemon = Daemon::spawn();
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);
    let dims = snapshot.dims;
    let tile_at = |x: i32, y: i32, z: i32| {
        if x < 0 || y < 0 || z < 0 || x >= dims.x as i32 || y >= dims.y as i32 || z >= dims.z as i32
        {
            return None;
        }
        let index = (x as u32 + y as u32 * dims.x + z as u32 * dims.x * dims.y) as usize;
        snapshot.tiles.get(index).copied()
    };
    let mut target = None;
    'search: for z in 1..dims.z as i32 {
        for y in 0..dims.y as i32 {
            for x in 0..dims.x as i32 {
                if !matches!(tile_at(x, y, z), Some(protocol::Tile::Solid(_))) {
                    continue;
                }
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    if matches!(tile_at(x + dx, y + dy, z), Some(protocol::Tile::Empty))
                        && matches!(
                            tile_at(x + dx, y + dy, z - 1),
                            Some(protocol::Tile::Solid(_) | protocol::Tile::Ramp(_))
                        )
                    {
                        target = Some([x, y, z]);
                        break 'search;
                    }
                }
            }
        }
    }
    let target = target.expect("generated world has an exposed solid face");

    send_speed(&mut writer, protocol::Speed::Fast);
    let mut saw_fast = false;
    for _ in 0..50 {
        if read_delta(&mut reader).speed == protocol::Speed::Fast {
            saw_fast = true;
            break;
        }
    }
    assert!(saw_fast, "daemon never applied the fast command");
    let designate = format!(
        "{{\"type\":\"designate\",\"kind\":\"dig\",\"rect\":{{\"min\":{target:?},\"max\":{target:?}}}}}\n"
    );
    send_literal(&mut writer, designate.as_bytes());

    let mut completed = None;
    for _ in 0..400 {
        let update = read_delta(&mut reader);
        if !update.tiles.is_empty() && !update.items.is_empty() {
            completed = Some(update);
            break;
        }
    }
    let completed = completed.expect("dig never emitted dirty terrain and an item together");
    assert!(completed.tiles.contains(&protocol::TileChange {
        pos: target,
        tile: protocol::Tile::Empty,
    }));
    assert!(
        completed
            .items
            .iter()
            .any(|item| item.pos == target && item.id >= 5)
    );
    let mut later = BufReader::new(daemon.connect());
    let later = read_snapshot(&mut later);
    assert!(
        later
            .items
            .iter()
            .any(|item| item.pos == target && item.id >= 5)
    );
}

#[test]
fn a_designated_dig_and_a_stockpile_stream_a_stone_onto_a_zone_tile() {
    let daemon = Daemon::spawn();
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader);
    let dims = snapshot.dims;
    let tile_at = |x: i32, y: i32, z: i32| {
        if x < 0 || y < 0 || z < 0 || x >= dims.x as i32 || y >= dims.y as i32 || z >= dims.z as i32
        {
            return None;
        }
        let index = (x as u32 + y as u32 * dims.x + z as u32 * dims.x * dims.y) as usize;
        snapshot.tiles.get(index).copied()
    };
    // A solid tile whose OWN floor is solid, so the stone it drops lands on standable ground and
    // can be picked up at all, beside a standable tile that serves as both work position and
    // stockpile. Without the floor condition the search finds overhangs, where items — which
    // never fall — are unreachable and this test would hang on a haul that cannot happen.
    let mut face = None;
    'search: for z in 1..dims.z as i32 {
        for y in 0..dims.y as i32 {
            for x in 0..dims.x as i32 {
                if !matches!(tile_at(x, y, z), Some(protocol::Tile::Solid(_)))
                    || !matches!(
                        tile_at(x, y, z - 1),
                        Some(protocol::Tile::Solid(_) | protocol::Tile::Ramp(_))
                    )
                {
                    continue;
                }
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    if matches!(tile_at(x + dx, y + dy, z), Some(protocol::Tile::Empty))
                        && matches!(
                            tile_at(x + dx, y + dy, z - 1),
                            Some(protocol::Tile::Solid(_) | protocol::Tile::Ramp(_))
                        )
                    {
                        face = Some(([x, y, z], [x + dx, y + dy, z]));
                        break 'search;
                    }
                }
            }
        }
    }
    let (target, pile) = face.expect("generated world has a floored solid face");

    send_speed(&mut writer, protocol::Speed::Fast);
    let mut saw_fast = false;
    for _ in 0..50 {
        if read_delta(&mut reader).speed == protocol::Speed::Fast {
            saw_fast = true;
            break;
        }
    }
    assert!(saw_fast, "daemon never applied the fast command");
    let stockpile = format!(
        "{{\"type\":\"place_stockpile\",\"rect\":{{\"min\":{pile:?},\"max\":{pile:?}}}}}\n"
    );
    send_literal(&mut writer, stockpile.as_bytes());
    let designate = format!(
        "{{\"type\":\"designate\",\"kind\":\"dig\",\"rect\":{{\"min\":{target:?},\"max\":{target:?}}}}}\n"
    );
    send_literal(&mut writer, designate.as_bytes());

    let mut stored = None;
    for _ in 0..900 {
        let update = read_delta(&mut reader);
        if let Some(item) = update
            .items
            .iter()
            .find(|item| update.zones.iter().any(|zone| zone.pos == item.pos))
        {
            stored = Some(*item);
            break;
        }
    }
    let stored = stored.expect("no stone ever reached a stockpile tile");
    assert_eq!(stored.pos, pile);
    assert!(stored.id >= 5);

    // A client connecting afterwards sees the stone on the pile, not just the delta that moved it.
    let mut later = BufReader::new(daemon.connect());
    let later = read_snapshot(&mut later);
    assert!(
        later
            .items
            .iter()
            .any(|item| item.pos == pile && item.id == stored.id),
        "the stored stone is missing from a fresh snapshot"
    );
    assert!(later.zones.iter().any(|zone| zone.pos == pile));
}

#[test]
fn streamed_deltas_show_wandering_positions_and_states() {
    let daemon = Daemon::spawn();
    let mut reader = BufReader::new(daemon.connect());
    let snapshot = read_snapshot(&mut reader);
    let mut previous = snapshot.entities;
    let mut moved = false;
    let mut saw_idle = false;
    let mut saw_walk = false;

    for _ in 0..30 {
        let update = read_delta(&mut reader);
        for entity in &update.entities {
            if let Some(old) = previous.iter().find(|old| old.id == entity.id) {
                moved |= old.pos != entity.pos;
            }
            saw_idle |= entity.state == protocol::JobState::Idle;
            saw_walk |= entity.state == protocol::JobState::Walk;
        }
        previous = update.entities;
    }

    assert!(
        moved,
        "no entity position changed across 30 consecutive deltas"
    );
    assert!(saw_idle, "no entity reported idle across 30 deltas");
    assert!(saw_walk, "no entity reported walk across 30 deltas");
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
fn fast_deltas_arrive_in_under_half_the_normal_span() {
    const SAMPLES: usize = 20;

    let daemon = Daemon::spawn();
    let stream = daemon.connect();
    let mut writer = stream.try_clone().expect("client write half must clone");
    let mut reader = BufReader::new(stream);
    let _ = read_snapshot(&mut reader);
    let _ = read_delta(&mut reader);

    let normal_started = Instant::now();
    for _ in 0..SAMPLES {
        let update = read_delta(&mut reader);
        assert_eq!(update.speed, protocol::Speed::Normal);
    }
    let normal_span = normal_started.elapsed();

    send_speed(&mut writer, protocol::Speed::Fast);
    let _ = read_delta_with_speed(&mut reader, protocol::Speed::Fast);
    let fast_started = Instant::now();
    for _ in 0..SAMPLES {
        let update = read_delta(&mut reader);
        assert_eq!(update.speed, protocol::Speed::Fast);
    }
    let fast_span = fast_started.elapsed();

    assert!(
        fast_span < normal_span / 2,
        "fast span {fast_span:?} was not under half normal span {normal_span:?}"
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
fn unterminated_partial_line_is_not_reported_as_overflow() {
    let daemon = Daemon::spawn();
    let mut client = BufReader::new(daemon.connect());
    let _: protocol::Snapshot = read_snapshot(&mut client);

    client
        .get_mut()
        .write_all(b"{\"type\":\"designate\"")
        .expect("partial command must write");
    client
        .get_mut()
        .shutdown(Shutdown::Write)
        .expect("close client write half");

    match daemon.stderr.recv_timeout(Duration::from_millis(500)) {
        Err(RecvTimeoutError::Timeout) => {}
        Err(RecvTimeoutError::Disconnected) => panic!("daemon stderr closed unexpectedly"),
        Ok(line) => assert!(
            !line.contains("exceeded 65536 bytes"),
            "partial line was falsely reported as overflow: {line}"
        ),
    }

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
