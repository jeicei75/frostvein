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
    io::{Read, Write},
    net::TcpListener,
    process::{Command, Stdio},
    thread,
    time::Duration,
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
    let delta = protocol::Delta {
        msg_type: protocol::MessageType::Delta,
        tick,
        tiles: Vec::new(),
        entities: Vec::new(),
        designations: Vec::new(),
        zones: Vec::new(),
        speed: protocol::Speed::Normal,
    };
    format!(
        "{}\n",
        serde_json::to_string(&delta).expect("encode stub delta")
    )
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
