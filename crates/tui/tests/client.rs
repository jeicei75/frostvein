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
        items: Vec::new(),
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
        items: Vec::new(),
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
        items: Vec::new(),
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
        // 21 = the 17 queued prelude deltas below plus the 4 that carry the consequence. The
        // capture must simply outlast the backlog; story 3.1 briefly solved this with a
        // client-side drain sized to simd's queue, removed at review as a layering breach.
        .arg("--frames")
        .arg("21")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(key) = key {
        command.arg("--key").arg(key);
    }
    let mut child = command.spawn().expect("spawn tui mark capture");

    let expects_command = key.is_some();
    let server = thread::spawn(move || {
        let mut stream = accept_with_timeout(&listener);
        // AC14 demands the negative control be the *identical* run. Both captures therefore get
        // the same large snapshot, the same 17-delta backlog and the same tick range; the only
        // difference is the key sequence, and hence whether the final deltas carry designations.
        let mut prelude = mark_snapshot_line();
        for tick in 8..=24 {
            let delta = protocol::Delta {
                msg_type: protocol::MessageType::Delta,
                tick,
                tiles: if tick == 8 {
                    vec![
                        protocol::TileChange {
                            pos: [0, 0, 0],
                            tile: protocol::Tile::Solid(protocol::Material::Ice),
                        };
                        512
                    ]
                } else {
                    Vec::new()
                },
                entities: Vec::new(),
                designations: Vec::new(),
                zones: Vec::new(),
                items: Vec::new(),
                speed: protocol::Speed::Normal,
            };
            prelude.push_str(&format!("{}\n", serde_json::to_string(&delta).unwrap()));
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

        for tick in 25..=28 {
            let delta = protocol::Delta {
                msg_type: protocol::MessageType::Delta,
                tick,
                tiles: Vec::new(),
                entities: Vec::new(),
                designations: designations.clone(),
                zones: Vec::new(),
                items: Vec::new(),
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

fn capture_dig_replay(changes: bool) -> String {
    const MAX_CAPTURE_BYTES: u64 = 2 * 1024 * 1024;
    const TARGET: [i32; 3] = [4, 2, 0];

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind dig replay stub");
    let port = listener.local_addr().expect("read stub address").port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_tui"))
        .arg(port.to_string())
        .arg("--frames")
        .arg("8")
        .arg("--key")
        .arg("d,enter,enter,l")
        .env_remove("NO_COLOR")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn tui dig replay capture");

    let server = thread::spawn(move || {
        let mut stream = accept_with_timeout(&listener);
        stream
            .write_all(mark_snapshot_line().as_bytes())
            .expect("send dig replay snapshot");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("bound designation command read");
        let mut reader = BufReader::new(stream.try_clone().expect("clone command read half"));
        let mut line = String::new();
        let bytes = (&mut reader)
            .take(4 * 1024)
            .read_line(&mut line)
            .expect("read dig replay command");
        assert!(
            bytes > 0 && line.ends_with('\n'),
            "missing command: {line:?}"
        );
        assert_eq!(
            serde_json::from_str::<protocol::Command>(&line).expect("decode dig replay command"),
            protocol::Command::Designate {
                kind: protocol::DesignationKind::Dig,
                rect: protocol::Rect {
                    min: TARGET,
                    max: TARGET,
                },
            }
        );

        for tick in 8..16 {
            let early = changes && tick <= 10;
            let late = changes && tick >= 11;
            let delta = protocol::Delta {
                msg_type: protocol::MessageType::Delta,
                tick,
                tiles: if tick == 11 && changes {
                    vec![protocol::TileChange {
                        pos: TARGET,
                        tile: protocol::Tile::Empty,
                    }]
                } else {
                    Vec::new()
                },
                entities: Vec::new(),
                designations: early
                    .then_some(protocol::Designation {
                        pos: TARGET,
                        kind: protocol::DesignationKind::Dig,
                    })
                    .into_iter()
                    .collect(),
                zones: Vec::new(),
                items: late
                    .then_some(protocol::Item {
                        id: 12,
                        pos: TARGET,
                    })
                    .into_iter()
                    .collect(),
                speed: protocol::Speed::Normal,
            };
            stream
                .write_all(format!("{}\n", serde_json::to_string(&delta).unwrap()).as_bytes())
                .expect("send dig replay delta");
            thread::sleep(Duration::from_millis(20));
        }
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stdout)
        .expect("read bounded dig replay stdout");
    assert!(stdout.len() as u64 <= MAX_CAPTURE_BYTES);
    let status = child.wait().expect("wait for dig replay capture");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stderr)
        .expect("read bounded dig replay stderr");
    assert!(stderr.len() as u64 <= MAX_CAPTURE_BYTES);
    server.join().expect("dig replay stub panicked");
    assert!(status.success(), "tui exited with {status}: {stderr}");
    stdout
}

#[test]
fn dig_replay_capture_transitions_from_designation_to_stone_at_one_cell() {
    let stdout = capture_dig_replay(true);
    let marks: Vec<_> = stdout
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            strip_ansi(text)
                .chars()
                .position(|glyph| glyph == '×')
                .map(|column| (line, column))
        })
        .collect();
    let stones: Vec<_> = stdout
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            strip_ansi(text)
                .chars()
                .position(|glyph| glyph == '*')
                .map(|column| (line, column))
        })
        .collect();

    assert!(
        !marks.is_empty(),
        "early frames contain no designation glyph"
    );
    assert!(!stones.is_empty(), "late frames contain no stone glyph");
    assert!(
        marks.iter().all(|(_, column)| *column == marks[0].1)
            && stones.iter().all(|(_, column)| *column == marks[0].1),
        "designation and stone did not occupy one target cell: {marks:?} {stones:?}"
    );
    assert!(
        marks.last().unwrap().0 < stones.first().unwrap().0,
        "stone did not arrive after the designation disappeared"
    );
}

#[test]
fn unchanged_dig_replay_capture_contains_neither_transition_glyph() {
    let stdout = capture_dig_replay(false);

    assert!(
        !stdout.contains('×'),
        "unchanged stub rendered a designation"
    );
    assert!(!stdout.contains('*'), "unchanged stub rendered a stone");
}

/// Every (line, column) a glyph reached in the capture. Line index is the clock: frames are
/// written in order, so "earlier" and "later" are line comparisons.
fn glyph_positions(stdout: &str, glyph: char) -> Vec<(usize, usize)> {
    stdout
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            strip_ansi(text)
                .chars()
                .position(|value| value == glyph)
                .map(|column| (line, column))
        })
        .collect()
}

/// Replays a whole haul past the real client: a stone at cell A, a dwarf that reaches it, the
/// stone following the dwarf across the map, and the stone left on the stockpile cell B with the
/// dwarf moved off it. With `changes = false` the identical run replays one unchanging world,
/// which is the guard on the assertions: a capture that merely CONTAINS a carrier glyph would
/// pass even if the client drew it unconditionally.
fn capture_haul_replay(changes: bool) -> String {
    const MAX_CAPTURE_BYTES: u64 = 2 * 1024 * 1024;
    const SOURCE: [i32; 3] = [2, 2, 0];
    const PILE: [i32; 3] = [5, 2, 0];

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind haul replay stub");
    let port = listener.local_addr().expect("read stub address").port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_tui"))
        .arg(port.to_string())
        .arg("--frames")
        .arg("10")
        // The stockpile is placed through the real modal machine, so the capture evidences the
        // command path as well as the render: `p`, one step right, anchor, commit, then `esc`.
        // The `esc` is load-bearing: committing does NOT leave the mode, and a mode other than
        // Normal keeps drawing the cursor over the pile cell — which is the cell the assertions
        // read, so without it this capture measures the cursor rather than the stone.
        .arg("--key")
        .arg("p,l,enter,enter,esc")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn tui haul replay capture");

    let server = thread::spawn(move || {
        let mut stream = accept_with_timeout(&listener);
        stream
            .write_all(mark_snapshot_line().as_bytes())
            .expect("send haul replay snapshot");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("bound stockpile command read");
        let mut reader = BufReader::new(stream.try_clone().expect("clone command read half"));
        let mut line = String::new();
        let bytes = (&mut reader)
            .take(4 * 1024)
            .read_line(&mut line)
            .expect("read haul replay command");
        assert!(
            bytes > 0 && line.ends_with('\n'),
            "missing command: {line:?}"
        );
        assert_eq!(
            serde_json::from_str::<protocol::Command>(&line).expect("decode stockpile command"),
            protocol::Command::PlaceStockpile {
                rect: protocol::Rect {
                    min: PILE,
                    max: PILE,
                },
            }
        );

        // dwarf position, stone position — the stone is under the dwarf while it is carried.
        let replay = [
            ([0, 2, 0], SOURCE),
            ([1, 2, 0], SOURCE),
            (SOURCE, SOURCE),
            ([3, 2, 0], [3, 2, 0]),
            ([4, 2, 0], [4, 2, 0]),
            (PILE, PILE),
            ([4, 2, 0], PILE),
            ([3, 2, 0], PILE),
            ([3, 2, 0], PILE),
            ([3, 2, 0], PILE),
        ];
        for (frame, (dwarf, stone)) in replay.into_iter().enumerate() {
            let (dwarf, stone) = if changes {
                (dwarf, stone)
            } else {
                ([0, 2, 0], SOURCE)
            };
            let delta = protocol::Delta {
                msg_type: protocol::MessageType::Delta,
                tick: 8 + frame as u64,
                tiles: Vec::new(),
                entities: vec![protocol::Entity {
                    id: 0,
                    kind: protocol::EntityKind::Dwarf,
                    pos: dwarf,
                    state: protocol::JobState::Walk,
                }],
                designations: Vec::new(),
                zones: vec![protocol::Zone { pos: PILE }],
                items: vec![protocol::Item { id: 12, pos: stone }],
                speed: protocol::Speed::Normal,
            };
            stream
                .write_all(format!("{}\n", serde_json::to_string(&delta).unwrap()).as_bytes())
                .expect("send haul replay delta");
            thread::sleep(Duration::from_millis(20));
        }
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stdout)
        .expect("read bounded haul replay stdout");
    assert!(stdout.len() as u64 <= MAX_CAPTURE_BYTES);
    let status = child.wait().expect("wait for haul replay capture");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr pipe")
        .take(MAX_CAPTURE_BYTES + 1)
        .read_to_string(&mut stderr)
        .expect("read bounded haul replay stderr");
    assert!(stderr.len() as u64 <= MAX_CAPTURE_BYTES);
    server.join().expect("haul replay stub panicked");
    assert!(status.success(), "tui exited with {status}: {stderr}");
    stdout
}

#[test]
fn haul_replay_capture_shows_the_stone_leave_the_source_ride_a_dwarf_and_reach_the_pile() {
    let stdout = capture_haul_replay(true);
    let stones = glyph_positions(&stdout, '*');
    let carriers = glyph_positions(&stdout, '☻');

    assert!(!stones.is_empty(), "no stone glyph reached the capture");
    assert!(!carriers.is_empty(), "no carrier glyph reached the capture");
    let (first_stone_line, source_column) = stones[0];
    let pile_column = stones.last().expect("a stone was captured").1;
    let first_carrier = carriers[0].0;
    let last_carrier = carriers.last().expect("a carrier was captured").0;
    assert_eq!(
        pile_column - source_column,
        3,
        "the stone did not end three cells east of where it started: {stones:?}"
    );

    // Transition one and three: the stone is at the source early and at the pile late — so it is
    // absent from the pile early and from the source late.
    let early: Vec<_> = stones
        .iter()
        .filter(|(line, _)| *line < first_carrier)
        .collect();
    let late: Vec<_> = stones
        .iter()
        .filter(|(line, _)| *line > last_carrier)
        .collect();
    assert!(
        !early.is_empty() && early.iter().all(|(_, column)| *column == source_column),
        "the stone was not sitting at the source before the carry: {early:?}"
    );
    assert!(
        !late.is_empty() && late.iter().all(|(_, column)| *column == pile_column),
        "the stone did not settle on the pile after the carry: {late:?}"
    );

    // Transition two: the carrier glyph appears in the middle of the run, not from the first
    // frame — which is the difference between showing a carry and drawing a glyph.
    assert!(
        first_carrier > first_stone_line,
        "the carrier glyph was already on screen in the first frame"
    );
}

#[test]
fn unchanged_haul_replay_capture_shows_none_of_the_three_transitions() {
    let stdout = capture_haul_replay(false);

    assert!(
        !stdout.contains('☻'),
        "unchanged stub rendered a dwarf carrying a stone"
    );
    let columns: std::collections::BTreeSet<_> = glyph_positions(&stdout, '*')
        .into_iter()
        .map(|(_, column)| column)
        .collect();
    assert_eq!(
        columns.len(),
        1,
        "the stone changed cells in a world that never changed: {columns:?}"
    );
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
        items: Vec::new(),
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
        items: Vec::new(),
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
            items: Vec::new(),
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
