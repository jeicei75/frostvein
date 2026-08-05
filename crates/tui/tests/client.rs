//! Proves the *composed* client loop — reader thread, `apply`, `render`, frame write —
//! actually consumes a live delta stream.
//!
//! Every other `tui` test exercises those pieces in isolation: `view`'s pinned assertion
//! proves `render` formats a tick it is handed, and `apply` is tested against a struct.
//! Nothing connected them to a socket, so a client whose loop had stopped consuming
//! deltas would have left `scripts/gate.sh` fully green (AC10 was live-run-only).
//!
//! The stub server here is deliberately not `simd`: `CARGO_BIN_EXE_simd` is not exported
//! to this package, and hand-rolling the wire keeps the test deterministic and fast.

use std::{
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const DIMS: protocol::Dims = protocol::Dims { x: 2, y: 2, z: 2 };
const SNAPSHOT_TICK: u64 = 7;

fn snapshot_line() -> String {
    let snapshot = protocol::Snapshot {
        msg_type: protocol::MessageType::Snapshot,
        dims: DIMS,
        tiles: vec![protocol::Tile::Solid(protocol::Material::Ice); 8],
        entities: Vec::new(),
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
        tick: SNAPSHOT_TICK,
    };
    format!(
        "{}\n",
        serde_json::to_string(&snapshot).expect("encode stub snapshot")
    )
}

fn delta_line(tick: u64) -> String {
    delta_line_with_speed(tick, protocol::Speed::Normal)
}

fn delta_line_with_speed(tick: u64, speed: protocol::Speed) -> String {
    let delta = protocol::Delta {
        msg_type: protocol::MessageType::Delta,
        tick,
        tiles: Vec::new(),
        entities: Vec::new(),
        designations: Vec::new(),
        zones: Vec::new(),
        speed,
    };
    format!(
        "{}\n",
        serde_json::to_string(&delta).expect("encode stub delta")
    )
}

fn accept_with_timeout(listener: &TcpListener) -> TcpStream {
    listener
        .set_nonblocking(true)
        .expect("stub listener must become nonblocking");
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match listener.accept() {
            Ok((stream, _)) => return stream,
            Err(error) if error.kind() == ErrorKind::WouldBlock && Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                panic!("tui did not connect to stub daemon within 3s")
            }
            Err(error) => panic!("stub daemon accept failed: {error}"),
        }
    }
}

fn strip_ansi(line: &str) -> String {
    let mut plain = String::new();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for escape in chars.by_ref() {
                if escape.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            plain.push(c);
        }
    }
    plain
}

fn capture_speed_frames(key: Option<&str>) -> Vec<String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind stub daemon");
    let port = listener
        .local_addr()
        .expect("read stub daemon address")
        .port();

    let mut command = Command::new(env!("CARGO_BIN_EXE_tui"));
    command
        .arg(port.to_string())
        .arg("--frames")
        .arg("3")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(key) = key {
        command.arg("--key").arg(key);
    }
    let mut child = command.spawn().expect("spawn tui");

    let expects_command = key.is_some();
    let server = thread::spawn(move || {
        let mut stream = accept_with_timeout(&listener);
        stream
            .write_all(snapshot_line().as_bytes())
            .expect("send stub snapshot");

        let speed = if expects_command {
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("bound stub command read");
            let read_stream = stream.try_clone().expect("clone stub read half");
            let mut reader = BufReader::new(read_stream);
            let mut line = String::new();
            let bytes = (&mut reader)
                .take(4 * 1024)
                .read_line(&mut line)
                .expect("read speed command");
            assert!(
                bytes > 0 && line.ends_with('\n'),
                "missing NDJSON command: {line:?}"
            );
            assert_eq!(
                serde_json::from_str::<protocol::Command>(&line).expect("decode speed command"),
                protocol::Command::SetSpeed {
                    speed: protocol::Speed::Paused
                }
            );
            protocol::Speed::Paused
        } else {
            protocol::Speed::Normal
        };

        for frame in 0..3 {
            let tick = if speed == protocol::Speed::Paused {
                SNAPSHOT_TICK + 1
            } else {
                SNAPSHOT_TICK + 1 + frame
            };
            stream
                .write_all(delta_line_with_speed(tick, speed).as_bytes())
                .expect("send stub delta");
            thread::sleep(Duration::from_millis(20));
        }
        thread::sleep(Duration::from_millis(500));
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .read_to_string(&mut stdout)
        .expect("read tui stdout");
    let status = child.wait().expect("wait for tui");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr pipe")
        .read_to_string(&mut stderr)
        .expect("read tui stderr");
    server.join().expect("stub daemon thread panicked");
    assert!(status.success(), "tui exited with {status}: {stderr}");

    stdout
        .lines()
        .map(strip_ansi)
        .filter(|line| line.starts_with("tick "))
        .collect()
}

fn captured_ticks(status_lines: &[String]) -> Vec<u64> {
    status_lines
        .iter()
        .map(|line| {
            line.split_whitespace()
                .nth(1)
                .expect("status line must contain a tick")
                .parse()
                .expect("status tick must be numeric")
        })
        .collect()
}

fn capture_load_frames(send_load: bool) -> Vec<String> {
    const SAVED_TICK: u64 = 3;
    const MAX_CAPTURE_BYTES: u64 = 1024 * 1024;

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind load stub daemon");
    let port = listener
        .local_addr()
        .expect("read load stub daemon address")
        .port();

    let mut command = Command::new(env!("CARGO_BIN_EXE_tui"));
    command
        .arg(port.to_string())
        .arg("--frames")
        .arg("4")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if send_load {
        command.arg("--key").arg("L");
    }
    let mut child = command.spawn().expect("spawn tui load capture");

    let server = thread::spawn(move || {
        let mut stream = accept_with_timeout(&listener);
        stream
            .write_all(snapshot_line().as_bytes())
            .expect("send initial stub snapshot");
        stream
            .write_all(delta_line(SNAPSHOT_TICK + 1).as_bytes())
            .expect("send pre-load climbing delta");

        if send_load {
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("bound load command read");
            let read_stream = stream.try_clone().expect("clone load stub read half");
            let mut reader = BufReader::new(read_stream);
            let mut line = String::new();
            let bytes = (&mut reader)
                .take(4 * 1024)
                .read_line(&mut line)
                .expect("read load command");
            assert!(
                bytes > 0 && line.ends_with('\n'),
                "missing NDJSON load command: {line:?}"
            );
            assert_eq!(
                serde_json::from_str::<protocol::Command>(&line).expect("decode load command"),
                protocol::Command::Load
            );

            let mut saved = serde_json::from_str::<protocol::Snapshot>(&snapshot_line())
                .expect("decode stub snapshot fixture");
            saved.tick = SAVED_TICK;
            stream
                .write_all(
                    format!(
                        "{}\n",
                        serde_json::to_string(&saved).expect("encode saved stub snapshot")
                    )
                    .as_bytes(),
                )
                .expect("send load snapshot");
            for tick in SAVED_TICK + 1..=SAVED_TICK + 2 {
                stream
                    .write_all(delta_line(tick).as_bytes())
                    .expect("send post-load climbing delta");
            }
        } else {
            for tick in SNAPSHOT_TICK + 2..=SNAPSHOT_TICK + 4 {
                stream
                    .write_all(delta_line(tick).as_bytes())
                    .expect("send uninterrupted climbing delta");
            }
        }
        thread::sleep(Duration::from_millis(500));
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stdout)
        .expect("read bounded tui stdout");
    assert!(
        stdout.len() as u64 <= MAX_CAPTURE_BYTES,
        "tui stdout exceeded the capture bound"
    );
    let status = child.wait().expect("wait for tui load capture");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stderr)
        .expect("read bounded tui stderr");
    assert!(
        stderr.len() as u64 <= MAX_CAPTURE_BYTES,
        "tui stderr exceeded the capture bound"
    );
    server.join().expect("load stub daemon thread panicked");
    assert!(status.success(), "tui exited with {status}: {stderr}");

    stdout
        .lines()
        .map(strip_ansi)
        .filter(|line| line.starts_with("tick "))
        .collect()
}

#[test]
fn key_space_freezes_the_streamed_frame_tick_and_reports_paused() {
    let status_lines = capture_speed_frames(Some("space"));

    assert_eq!(captured_ticks(&status_lines), vec![8, 8, 8]);
    assert!(status_lines.iter().all(|line| line.contains("paused")));
}

#[test]
fn streamed_frame_ticks_climb_when_no_key_is_sent() {
    let status_lines = capture_speed_frames(None);

    assert_eq!(captured_ticks(&status_lines), vec![8, 9, 10]);
}

// AC11 names `--key <S|L>`, so `S` needs its own proof that the real `apply_key` and the real
// write half carry it to the daemon. `L` is proven by the rewind it causes; `S` changes nothing
// the client can see, so the stub asserting the exact wire command is the only observable.
fn capture_save_command() -> (protocol::Command, Vec<u64>) {
    const MAX_CAPTURE_BYTES: u64 = 1024 * 1024;

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind save stub daemon");
    let port = listener
        .local_addr()
        .expect("read save stub daemon address")
        .port();

    let mut child = Command::new(env!("CARGO_BIN_EXE_tui"))
        .arg(port.to_string())
        .arg("--frames")
        .arg("3")
        .arg("--key")
        .arg("S")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn tui save capture");

    let server = thread::spawn(move || {
        let mut stream = accept_with_timeout(&listener);
        stream
            .write_all(snapshot_line().as_bytes())
            .expect("send stub snapshot");

        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("bound save command read");
        let read_stream = stream.try_clone().expect("clone save stub read half");
        let mut reader = BufReader::new(read_stream);
        let mut line = String::new();
        let bytes = (&mut reader)
            .take(4 * 1024)
            .read_line(&mut line)
            .expect("read save command");
        assert!(
            bytes > 0 && line.ends_with('\n'),
            "missing NDJSON save command: {line:?}"
        );
        let command =
            serde_json::from_str::<protocol::Command>(&line).expect("decode save command");

        for tick in SNAPSHOT_TICK + 1..=SNAPSHOT_TICK + 3 {
            stream
                .write_all(delta_line(tick).as_bytes())
                .expect("send post-save climbing delta");
        }
        thread::sleep(Duration::from_millis(500));
        command
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stdout)
        .expect("read bounded tui stdout");
    assert!(
        stdout.len() as u64 <= MAX_CAPTURE_BYTES,
        "tui stdout exceeded the capture bound"
    );
    let status = child.wait().expect("wait for tui save capture");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stderr)
        .expect("read bounded tui stderr");
    assert!(
        stderr.len() as u64 <= MAX_CAPTURE_BYTES,
        "tui stderr exceeded the capture bound"
    );
    let command = server.join().expect("save stub daemon thread panicked");
    assert!(status.success(), "tui exited with {status}: {stderr}");

    let status_lines: Vec<String> = stdout
        .lines()
        .map(strip_ansi)
        .filter(|line| line.starts_with("tick "))
        .collect();
    (command, captured_ticks(&status_lines))
}

#[test]
fn key_s_sends_save_and_leaves_the_streamed_ticks_climbing() {
    let (command, ticks) = capture_save_command();

    assert_eq!(command, protocol::Command::Save);
    // The connect snapshot is the first captured frame; `save` must not disturb the stream.
    assert_eq!(ticks, vec![8, 9, 10]);
}

#[test]
fn key_l_rewinds_captured_ticks_then_they_climb_from_the_saved_tick() {
    let status_lines = capture_load_frames(true);

    assert_eq!(captured_ticks(&status_lines), vec![8, 3, 4, 5]);
}

#[test]
fn load_capable_stub_climbs_monotonically_when_no_key_is_sent() {
    let status_lines = capture_load_frames(false);

    assert_eq!(captured_ticks(&status_lines), vec![8, 9, 10, 11]);
}

const MARK_DIMS: protocol::Dims = protocol::Dims { x: 8, y: 5, z: 1 };

fn mark_snapshot_line() -> String {
    let snapshot = protocol::Snapshot {
        msg_type: protocol::MessageType::Snapshot,
        dims: MARK_DIMS,
        tiles: vec![protocol::Tile::Solid(protocol::Material::Ice); 40],
        entities: Vec::new(),
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
        tick: SNAPSHOT_TICK,
    };
    let mut line = serde_json::to_string(&snapshot).unwrap();
    line.extend(std::iter::repeat_n(' ', 16 * 1024 - 1 - line.len()));
    line.push('\n');
    line
}

fn capture_designation_frames(key: Option<&str>) -> (String, String) {
    const MAX_CAPTURE_BYTES: u64 = 2 * 1024 * 1024;

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind mark stub daemon");
    let port = listener.local_addr().expect("read stub address").port();
    let mut command = Command::new(env!("CARGO_BIN_EXE_tui"));
    command
        .arg(port.to_string())
        .arg("--frames")
        .arg("4")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(key) = key {
        command.arg("--key").arg(key);
    }
    let mut child = command.spawn().expect("spawn tui mark capture");

    let expects_command = key.is_some();
    let server = thread::spawn(move || {
        let mut stream = accept_with_timeout(&listener);
        let mut prelude = mark_snapshot_line();
        if expects_command {
            for tick in 8..=24 {
                let delta = protocol::Delta {
                    msg_type: protocol::MessageType::Delta,
                    tick,
                    tiles: Vec::new(),
                    entities: Vec::new(),
                    designations: Vec::new(),
                    zones: Vec::new(),
                    speed: protocol::Speed::Normal,
                };
                prelude.push_str(&format!("{}\n", serde_json::to_string(&delta).unwrap()));
            }
        }
        stream
            .write_all(prelude.as_bytes())
            .expect("send mark snapshot and queued deltas");

        let designations = if expects_command {
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("bound designation command read");
            let mut reader = BufReader::new(stream.try_clone().expect("clone command read half"));
            let mut line = String::new();
            let bytes = (&mut reader)
                .take(4 * 1024)
                .read_line(&mut line)
                .expect("read designation command");
            assert!(
                bytes > 0 && line.ends_with('\n'),
                "missing command: {line:?}"
            );
            let command: protocol::Command = serde_json::from_str(&line).expect("decode command");
            let protocol::Command::Designate { kind, rect } = command else {
                panic!("expected designate command, got {command:?}");
            };
            assert_eq!(kind, protocol::DesignationKind::Dig);
            assert_eq!(rect.min, [4, 2, 0]);
            assert_eq!(rect.max, [6, 2, 0]);
            (rect.min[0]..=rect.max[0])
                .map(|x| protocol::Designation {
                    pos: [x, 2, 0],
                    kind,
                })
                .collect()
        } else {
            Vec::new()
        };

        let ticks = if expects_command { 25..=28 } else { 8..=11 };
        for tick in ticks {
            let delta = protocol::Delta {
                msg_type: protocol::MessageType::Delta,
                tick,
                tiles: Vec::new(),
                entities: Vec::new(),
                designations: designations.clone(),
                zones: Vec::new(),
                speed: protocol::Speed::Normal,
            };
            stream
                .write_all(format!("{}\n", serde_json::to_string(&delta).unwrap()).as_bytes())
                .expect("send mark delta");
            thread::sleep(Duration::from_millis(20));
        }
        thread::sleep(Duration::from_millis(500));
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stdout)
        .expect("read bounded mark stdout");
    assert!(stdout.len() as u64 <= MAX_CAPTURE_BYTES);
    let status = child.wait().expect("wait for mark capture");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stderr)
        .expect("read bounded mark stderr");
    assert!(stderr.len() as u64 <= MAX_CAPTURE_BYTES);
    server.join().expect("mark stub daemon thread panicked");
    assert!(status.success(), "tui exited with {status}: {stderr}");
    (stdout, stderr)
}

fn glyph_columns_for(stdout: &str, glyph: char) -> Vec<usize> {
    stdout
        .lines()
        .filter_map(|line| strip_ansi(line).chars().position(|value| value == glyph))
        .collect()
}

#[test]
fn key_sequence_designates_and_the_echoed_marker_reaches_expected_columns() {
    let (stdout, _) = capture_designation_frames(Some("d,enter,l,l,enter"));
    let columns = glyph_columns_for(&stdout, '×');
    let marker_line = stdout
        .lines()
        .map(strip_ansi)
        .find(|line| line.contains('×'))
        .expect("a streamed frame must contain the echoed marker");
    let expected = marker_line.chars().count() / 2;

    assert!(!columns.is_empty(), "no dig glyph reached the capture");
    assert!(
        columns.iter().all(|column| *column == expected),
        "{columns:?}"
    );
    assert!(
        stdout.lines().map(strip_ansi).any(|line| {
            let glyphs: Vec<_> = line.chars().collect();
            glyphs.get(expected) == Some(&'×') && glyphs.get(expected + 1) == Some(&'×')
        }),
        "echoed rect did not occupy the two expected centre columns"
    );
}

#[test]
fn identical_capture_without_a_key_sequence_contains_no_marker() {
    let (stdout, _) = capture_designation_frames(None);

    assert!(!stdout.contains('×'), "marker rendered without a command");
}

const WIDE_DIMS: protocol::Dims = protocol::Dims { x: 16, y: 16, z: 1 };

fn dwarf_at(x: i32) -> protocol::Entity {
    protocol::Entity {
        id: 0,
        kind: protocol::EntityKind::Dwarf,
        pos: [x, 8, 0],
        state: protocol::JobState::Idle,
    }
}

fn moving_snapshot_line(x: i32) -> String {
    let snapshot = protocol::Snapshot {
        msg_type: protocol::MessageType::Snapshot,
        dims: WIDE_DIMS,
        tiles: vec![protocol::Tile::Solid(protocol::Material::Ice); 256],
        entities: vec![dwarf_at(x)],
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
        tick: SNAPSHOT_TICK,
    };
    format!(
        "{}\n",
        serde_json::to_string(&snapshot).expect("encode stub snapshot")
    )
}

fn moving_delta_line(tick: u64, x: i32) -> String {
    let delta = protocol::Delta {
        msg_type: protocol::MessageType::Delta,
        tick,
        tiles: Vec::new(),
        entities: vec![dwarf_at(x)],
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
    };
    format!(
        "{}\n",
        serde_json::to_string(&delta).expect("encode stub delta")
    )
}

/// The column each rendered row places the dwarf glyph at, with the colour escapes
/// stripped so the measurement survives `NO_COLOR` being set or unset.
fn glyph_columns(stdout: &str) -> Vec<usize> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut column = 0;
            let mut chars = line.chars();
            while let Some(c) = chars.next() {
                if c == '\u{1b}' {
                    for escape in chars.by_ref() {
                        if escape.is_ascii_alphabetic() {
                            break;
                        }
                    }
                    continue;
                }
                if c == '☺' {
                    return Some(column);
                }
                column += 1;
            }
            None
        })
        .collect()
}

/// The `--frames` instrument is how this project evidences "the world visibly lives",
/// so it must not render motion as stillness. Recomputing the view state per frame
/// re-centres the camera on entity 0 every time, pinning that dwarf to the middle of
/// the screen while the terrain scrolls underneath — the glyph never moves, and an
/// AC read off that capture would be read off an artefact of the instrument.
#[test]
fn streamed_frames_hold_the_camera_still_so_a_moving_dwarf_moves_on_screen() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind stub daemon");
    let port = listener
        .local_addr()
        .expect("read stub daemon address")
        .port();

    let mut child = Command::new(env!("CARGO_BIN_EXE_tui"))
        .arg(port.to_string())
        .arg("--frames")
        .arg("3")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn tui");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("tui must connect");
        stream
            .write_all(moving_snapshot_line(4).as_bytes())
            .expect("send stub snapshot");
        for (tick, x) in (8..=10).zip(5..=7) {
            stream
                .write_all(moving_delta_line(tick, x).as_bytes())
                .expect("send stub delta");
            thread::sleep(Duration::from_millis(20));
        }
        thread::sleep(Duration::from_millis(500));
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .read_to_string(&mut stdout)
        .expect("read tui stdout");
    let status = child.wait().expect("wait for tui");
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    server.join().expect("stub daemon thread panicked");

    assert!(status.success(), "tui exited with {status}: {stderr}");

    let columns = glyph_columns(&stdout);
    assert_eq!(
        columns.len(),
        3,
        "expected one dwarf glyph per streamed frame, saw {columns:?}"
    );
    assert!(
        columns.iter().any(|column| *column != columns[0]),
        "the dwarf walked three tiles but its glyph never left column {}: the camera is \
         being recomputed per frame, so the instrument renders motion as stillness",
        columns[0]
    );
}

/// The walk colour from `palette::entity_cell`, as it appears on the wire to the terminal.
const WALK_SGR: &str = "38;2;214;154;78";

/// Runs the instrument against a stub daemon that streams one walking dwarf, and returns
/// its `(stdout, stderr)`. `no_color` chooses whether the child sees `NO_COLOR=1` — the
/// devpod sets it, so a test that did not control it would prove whatever the environment
/// happened to be that day.
fn capture_walking_dwarf(no_color: bool) -> (String, String) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind stub daemon");
    let port = listener
        .local_addr()
        .expect("read stub daemon address")
        .port();

    let mut command = Command::new(env!("CARGO_BIN_EXE_tui"));
    command
        .arg(port.to_string())
        .arg("--frames")
        .arg("1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if no_color {
        command.env("NO_COLOR", "1");
    } else {
        command.env_remove("NO_COLOR");
    }
    let mut child = command.spawn().expect("spawn tui");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("tui must connect");
        stream
            .write_all(moving_snapshot_line(4).as_bytes())
            .expect("send stub snapshot");
        let walking = protocol::Delta {
            msg_type: protocol::MessageType::Delta,
            tick: 8,
            tiles: Vec::new(),
            entities: vec![protocol::Entity {
                id: 0,
                kind: protocol::EntityKind::Dwarf,
                pos: [5, 8, 0],
                state: protocol::JobState::Walk,
            }],
            designations: Vec::new(),
            zones: Vec::new(),
            speed: protocol::Speed::Normal,
        };
        stream
            .write_all(
                format!(
                    "{}\n",
                    serde_json::to_string(&walking).expect("encode walking delta")
                )
                .as_bytes(),
            )
            .expect("send walking delta");
        thread::sleep(Duration::from_millis(500));
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .read_to_string(&mut stdout)
        .expect("read tui stdout");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr pipe")
        .read_to_string(&mut stderr)
        .expect("read tui stderr");
    let status = child.wait().expect("wait for tui");
    server.join().expect("stub daemon thread panicked");
    assert!(status.success(), "tui exited with {status}: {stderr}");

    (stdout, stderr)
}

/// The colour half of the instrument, proven through the real binary rather than through
/// `render` in isolation: a walking dwarf must actually reach the capture wearing the walk
/// colour. Nothing tested this end to end before, which is how the story's own colour
/// evidence came to be taken from a capture that had no colour in it at all.
#[test]
fn a_walking_dwarf_reaches_the_capture_wearing_the_walk_colour() {
    let (stdout, _) = capture_walking_dwarf(false);

    assert!(
        stdout.contains(WALK_SGR),
        "no walk colour ({WALK_SGR}) in a capture of a walking dwarf; \
         the job-state colour does not survive the real client path"
    );
}

/// A capture with colour suppressed still looks perfectly well-formed, so the instrument
/// has to say so itself. Without this warning a colourless capture reads as evidence that
/// the colours work — the exact way this project's evidence has already been fooled once.
#[test]
fn the_instrument_refuses_to_be_silently_colourless() {
    let (stdout, stderr) = capture_walking_dwarf(true);

    assert!(
        !stdout.contains(WALK_SGR),
        "NO_COLOR was set but the capture still carried colour; \
         the premise of this test no longer holds"
    );
    assert!(
        stderr.contains("NO_COLOR"),
        "a colourless capture was produced with no warning on stderr; \
         it would be read as evidence that the state colours work. stderr was: {stderr:?}"
    );
}

#[test]
fn the_client_loop_renders_a_frame_per_streamed_delta() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind stub daemon");
    let port = listener
        .local_addr()
        .expect("read stub daemon address")
        .port();

    let mut child = Command::new(env!("CARGO_BIN_EXE_tui"))
        .arg(port.to_string())
        .arg("--frames")
        .arg("3")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn tui");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("tui must connect");
        stream
            .write_all(snapshot_line().as_bytes())
            .expect("send stub snapshot");
        for tick in 8..=10 {
            stream
                .write_all(delta_line(tick).as_bytes())
                .expect("send stub delta");
            thread::sleep(Duration::from_millis(20));
        }
        // Hold the socket open until the client has rendered its frames and exited;
        // dropping it early would race an EOF against the third frame.
        thread::sleep(Duration::from_millis(500));
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .read_to_string(&mut stdout)
        .expect("read tui stdout");
    let status = child.wait().expect("wait for tui");
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    server.join().expect("stub daemon thread panicked");

    assert!(status.success(), "tui exited with {status}: {stderr}");

    for tick in 8..=10 {
        assert!(
            stdout.contains(&format!("tick {tick}")),
            "no frame rendered for streamed tick {tick}; the client loop is not applying deltas"
        );
    }

    // The connect snapshot is consumed before the streaming loop starts, so a frame for
    // its tick would mean frames are being driven by something other than the deltas.
    assert!(
        !stdout.contains(&format!("tick {SNAPSHOT_TICK}")),
        "a frame was rendered for the connect snapshot rather than for a streamed delta"
    );
}
