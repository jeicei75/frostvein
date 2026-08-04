#![forbid(unsafe_code)]

mod frame;
mod palette;
mod view;

use std::{
    env,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::TcpStream,
    sync::mpsc::{self, SyncSender, TryRecvError},
    thread,
    time::Duration,
};

use anyhow::{Context, bail};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::ResetColor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use protocol::{Delta, Snapshot};

use crate::{
    frame::{RowEnd, write_frame},
    view::{Action, apply_key, initial, render},
};

/// Mirrors the daemon's 30 s write timeout. Without it a peer that accepts and
/// then goes silent leaves the client blocked forever.
const SNAPSHOT_READ_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(50);
const MESSAGE_QUEUE: usize = 16;

/// Caps the snapshot line so a server that never sends a newline cannot grow
/// the buffer without bound. The 128x128x32 snapshot is ~6.9 MB.
const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;

/// Sized so one frame reaches the terminal in a single write (AC10).
// NOTE: a terminal larger than roughly 25 000 cells splits across writes again.
const FRAME_BUFFER_BYTES: usize = 512 * 1024;

struct TerminalGuard;

enum Msg {
    Snapshot(Box<Snapshot>),
    Delta(Box<Delta>),
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut out = io::stdout();
        // ResetColor last: SGR state is shared with the primary screen, so
        // leaving the alternate screen does not undo the frame's colours.
        let _ = execute!(out, LeaveAlternateScreen, Show, ResetColor);
    }
}

fn main() -> anyhow::Result<()> {
    // NOTE: args_os, not args — std::env::args panics on non-UTF-8 argv during
    // iteration, which would bypass anyhow and print a raw backtrace.
    let mut port = protocol::DEFAULT_PORT;
    let mut port_was_set = false;
    let mut frame_only = false;
    let mut frames: Option<u32> = None;
    let mut expect_frame_count = false;
    let mut key = None;
    let mut expect_key = false;
    for arg in std::env::args_os().skip(1) {
        if expect_frame_count {
            let text = arg
                .to_str()
                .with_context(|| format!("--frames count is not valid UTF-8: {arg:?}"))?;
            frames = Some(text.parse().with_context(|| {
                format!("invalid --frames count {text:?}: expected a positive integer")
            })?);
            expect_frame_count = false;
            continue;
        }
        if expect_key {
            let text = arg
                .to_str()
                .with_context(|| format!("--key value is not valid UTF-8: {arg:?}"))?;
            key = Some(match text {
                "space" => KeyCode::Char(' '),
                "+" => KeyCode::Char('+'),
                "-" => KeyCode::Char('-'),
                _ => bail!("invalid --key value {text:?}: expected space, +, or -"),
            });
            expect_key = false;
            continue;
        }
        if arg == "--frame" {
            frame_only = true;
            continue;
        }
        if arg == "--frames" {
            expect_frame_count = true;
            continue;
        }
        if arg == "--key" {
            expect_key = true;
            continue;
        }
        if port_was_set {
            bail!("unexpected extra argument {arg:?}: expected at most one port (0-65535)");
        }
        let text = arg
            .to_str()
            .with_context(|| format!("port argument is not valid UTF-8: {arg:?}"))?;
        port = text.parse().with_context(|| {
            format!("invalid port argument {text:?}: expected 1-65535 (the port simd listens on)")
        })?;
        port_was_set = true;
    }
    if expect_frame_count {
        bail!("--frames requires a count, e.g. --frames 3");
    }
    if expect_key {
        bail!("--key requires space, +, or -");
    }
    if key.is_some() && frames.is_none() {
        bail!("--key requires --frames");
    }

    let address = format!("127.0.0.1:{port}");
    let stream = TcpStream::connect(("127.0.0.1", port))
        .with_context(|| format!("could not connect to {address}"))?;
    stream
        .set_read_timeout(Some(SNAPSHOT_READ_TIMEOUT))
        .context("could not set the snapshot read timeout")?;
    let mut writer = stream
        .try_clone()
        .context("could not clone the server socket for commands")?;
    let mut reader = BufReader::new(stream);
    let mut snapshot = read_snapshot(&mut reader)?;
    let mut state = initial(&snapshot);

    if let Some(count) = frames {
        if let Some(code) = key
            && let Action::Command(command) = apply_key(
                &mut state,
                KeyEvent::new(code, KeyModifiers::NONE),
                snapshot.dims,
                snapshot.speed,
            )
        {
            send_command(&mut writer, command)?;
        }
        return stream_frames(reader, snapshot, state, count);
    }

    if frame_only {
        let (w, h) = frame_size();
        let framebuffer = render(&snapshot, &state, w, h);
        let mut out = BufWriter::with_capacity(FRAME_BUFFER_BYTES, io::stdout());
        write_frame(&mut out, &framebuffer, RowEnd::Newline)
            .context("could not write terminal frame")?;
        out.flush().context("could not flush terminal frame")?;
        return Ok(());
    }

    terminal::enable_raw_mode().context("could not enable terminal raw mode")?;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen, Hide) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show, ResetColor);
        let _ = terminal::disable_raw_mode();
        return Err(error).context("could not enter terminal view");
    }
    let _guard = TerminalGuard;
    let mut out = BufWriter::with_capacity(FRAME_BUFFER_BYTES, stdout);
    let mut size = terminal::size().context("could not read terminal size")?;
    let mut needs_redraw = true;
    reader
        .get_mut()
        .set_read_timeout(None)
        .context("could not clear the snapshot read timeout")?;
    let (message_tx, message_rx) = mpsc::sync_channel(MESSAGE_QUEUE);
    thread::Builder::new()
        .name("server-read".to_string())
        .spawn(move || read_messages(reader, message_tx))
        .context("could not spawn server reader thread")?;

    'running: loop {
        if event::poll(POLL_INTERVAL).context("could not poll terminal events")? {
            match event::read().context("could not read terminal event")? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match apply_key(&mut state, key, snapshot.dims, snapshot.speed) {
                        Action::Redraw => needs_redraw = true,
                        Action::Quit => break 'running,
                        // NOTE: the next speed is derived from the last wire update. Two
                        // presses inside one round-trip therefore see the same stale speed
                        // and the second can be a no-op; optimistic local speed is omitted.
                        Action::Command(command) => {
                            send_command(&mut writer, command)?;
                        }
                        Action::Ignore => {}
                    }
                }
                Event::Resize(w, h) => {
                    size = (w, h);
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        loop {
            match message_rx.try_recv() {
                // AC8: the daemon sends exactly one snapshot, at connect, and that one
                // is consumed before this loop starts — so this arm is unreachable
                // today. If a re-snapshot ever arrives, adopt the world but KEEP the
                // camera and z-level; resetting them would silently throw away where
                // the player was looking.
                // NOTE: assumes dims never change between snapshots, which holds while
                // the daemon serves one world for its lifetime.
                Ok(Ok(Msg::Snapshot(next))) => {
                    snapshot = *next;
                    needs_redraw = true;
                }
                Ok(Ok(Msg::Delta(delta))) => {
                    apply(&mut snapshot, *delta);
                    needs_redraw = true;
                }
                Ok(Err(error)) => return Err(error),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    bail!("server reader thread stopped unexpectedly")
                }
            }
        }

        if needs_redraw {
            // A terminal reporting 0x0 at startup that never emits a Resize would
            // otherwise blank the client forever with no diagnostic: deltas keep
            // setting needs_redraw and this guard keeps skipping. Re-query rather
            // than trusting the startup reading for the process's lifetime.
            if size.0 == 0 || size.1 == 0 {
                size = terminal::size().unwrap_or(size);
            }
            if size.0 != 0 && size.1 != 0 {
                let framebuffer = render(&snapshot, &state, size.0, size.1);
                write_frame(&mut out, &framebuffer, RowEnd::MoveTo)
                    .context("could not write terminal frame")?;
                out.flush().context("could not flush terminal frame")?;
                // Left set when the size is still unusable, so the redraw is retried
                // instead of silently dropped.
                needs_redraw = false;
            }
        }
    }

    Ok(())
}

fn send_command(writer: &mut TcpStream, command: protocol::Command) -> anyhow::Result<()> {
    writeln!(
        writer,
        "{}",
        serde_json::to_string(&command).context("could not encode client command")?
    )
    .context("could not write client command")?;
    writer.flush().context("could not flush client command")?;
    Ok(())
}

fn frame_size() -> (u16, u16) {
    match terminal::size() {
        Ok((w, h)) if w > 0 && h > 0 => (w, h),
        _ => (100, 40),
    }
}

/// Headless counterpart to the interactive loop, for proving AC10 in the suite.
///
/// It runs the REAL reader thread and the real `apply` -> `render` -> `write_frame`
/// path, emitting one frame per server message and then exiting. `--frame` cannot
/// stand in for this: it renders the connect snapshot and returns before the reader
/// thread is ever spawned, so nothing in the suite could catch a client whose loop
/// had stopped consuming deltas.
fn stream_frames(
    mut reader: BufReader<TcpStream>,
    mut snapshot: Snapshot,
    state: view::ViewState,
    count: u32,
) -> anyhow::Result<()> {
    reader
        .get_mut()
        .set_read_timeout(None)
        .context("could not clear the snapshot read timeout")?;
    let (message_tx, message_rx) = mpsc::sync_channel(MESSAGE_QUEUE);
    thread::Builder::new()
        .name("server-read".to_string())
        .spawn(move || read_messages(reader, message_tx))
        .context("could not spawn server reader thread")?;

    // This mode exists to produce evidence, and colour is the only signal carrying job
    // state. When crossterm drops every colour sequence the capture still looks perfectly
    // well-formed, so a colour claim read off it is a claim about nothing — which has
    // already happened once on this project. Refuse to be silently vacuous.
    if colour_is_suppressed() {
        eprintln!(
            "warning: NO_COLOR is set, so this capture contains no colour and cannot \
             evidence dwarf job-state colours. Re-run with NO_COLOR unset to check them."
        );
    }

    let (w, h) = frame_size();
    // The camera is fixed once, exactly as the interactive path does it. Recomputing
    // `initial` per frame re-centres on entity 0 every time, which pins that dwarf to
    // the middle of the screen and hides the very motion this instrument exists to show.
    let mut out = BufWriter::with_capacity(FRAME_BUFFER_BYTES, io::stdout());
    for _ in 0..count {
        // Bounded like every other read here: a server that connects and then goes
        // quiet must fail, never hang.
        match message_rx.recv_timeout(SNAPSHOT_READ_TIMEOUT) {
            Ok(Ok(Msg::Snapshot(next))) => snapshot = *next,
            Ok(Ok(Msg::Delta(delta))) => apply(&mut snapshot, *delta),
            Ok(Err(error)) => return Err(error),
            Err(_) => bail!("no server message within {SNAPSHOT_READ_TIMEOUT:?}"),
        }
        let framebuffer = render(&snapshot, &state, w, h);
        write_frame(&mut out, &framebuffer, RowEnd::Newline)
            .context("could not write terminal frame")?;
        out.flush().context("could not flush terminal frame")?;
    }

    Ok(())
}

/// Mirrors crossterm's own rule (`Colored::ansi_color_disabled`): set and non-empty
/// disables colour, an empty value does not.
fn colour_is_suppressed() -> bool {
    !env::var("NO_COLOR").unwrap_or_default().is_empty()
}

fn read_snapshot(reader: &mut dyn BufRead) -> anyhow::Result<Snapshot> {
    match read_message(reader)? {
        Some(Msg::Snapshot(snapshot)) => Ok(*snapshot),
        Some(Msg::Delta(_)) => bail!("server sent a delta before its snapshot"),
        None => bail!("server closed before sending a snapshot"),
    }
}

fn read_message(reader: &mut dyn BufRead) -> anyhow::Result<Option<Msg>> {
    let mut line = String::new();
    let bytes = reader
        .take(MAX_SNAPSHOT_BYTES)
        .read_line(&mut line)
        .context("could not read server message line")?;
    if bytes == 0 {
        return Ok(None);
    }
    if !line.ends_with('\n') {
        if bytes as u64 >= MAX_SNAPSHOT_BYTES {
            bail!("server message line exceeded {MAX_SNAPSHOT_BYTES} bytes with no newline");
        }
        bail!("server closed before terminating its message line");
    }
    let value: serde_json::Value =
        serde_json::from_str(&line).context("could not decode server message")?;
    match value.get("type").and_then(serde_json::Value::as_str) {
        Some("snapshot") => {
            let snapshot: Snapshot =
                serde_json::from_value(value).context("could not decode snapshot")?;
            validate_snapshot(&snapshot)?;
            Ok(Some(Msg::Snapshot(Box::new(snapshot))))
        }
        Some("delta") => {
            let delta = serde_json::from_value(value).context("could not decode delta")?;
            Ok(Some(Msg::Delta(Box::new(delta))))
        }
        Some(message_type) => bail!("unknown server message type {message_type:?}"),
        None => bail!("server message has no string type field"),
    }
}

fn validate_snapshot(snapshot: &Snapshot) -> anyhow::Result<()> {
    // A decodable snapshot can still be inconsistent, and `render` indexes
    // `tiles` from `dims` directly — checking here keeps that an error rather
    // than a panic.
    let dims = snapshot.dims;
    let expected = u64::from(dims.x) * u64::from(dims.y) * u64::from(dims.z);
    if snapshot.tiles.len() as u64 != expected {
        bail!(
            "snapshot has {} tiles but dims {}x{}x{} need {expected}",
            snapshot.tiles.len(),
            dims.x,
            dims.y,
            dims.z
        );
    }

    Ok(())
}

fn read_messages(mut reader: BufReader<TcpStream>, sender: SyncSender<anyhow::Result<Msg>>) {
    loop {
        let message = match read_message(&mut reader) {
            Ok(Some(message)) => Ok(message),
            Ok(None) => Err(anyhow::anyhow!("server closed the connection")),
            Err(error) => Err(error),
        };
        let done = message.is_err();
        if sender.send(message).is_err() || done {
            return;
        }
    }
}

fn apply(snapshot: &mut Snapshot, delta: Delta) {
    for change in delta.tiles {
        let [x, y, z] = change.pos;
        if x < 0
            || y < 0
            || z < 0
            || x >= snapshot.dims.x as i32
            || y >= snapshot.dims.y as i32
            || z >= snapshot.dims.z as i32
        {
            continue;
        }
        let index = x as usize
            + y as usize * snapshot.dims.x as usize
            + z as usize * snapshot.dims.x as usize * snapshot.dims.y as usize;
        if let Some(tile) = snapshot.tiles.get_mut(index) {
            *tile = change.tile;
        }
    }
    snapshot.entities = delta.entities;
    snapshot.designations = delta.designations;
    snapshot.zones = delta.zones;
    snapshot.speed = delta.speed;
    snapshot.tick = delta.tick;
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use protocol::{
        Delta, Dims, Entity, EntityKind, JobState, Material, MessageType, Speed, Tile, TileChange,
    };

    use super::*;

    const SNAPSHOT_LINE: &str = concat!(
        r#"{"type":"snapshot","dims":{"x":2,"y":1,"z":1},"#,
        r#""tiles":["empty",{"solid":"ice"}],"entities":[{"id":7,"kind":"dwarf","pos":[0,0,0],"state":"idle"}],"#,
        r#""designations":[],"zones":[],"speed":"normal","tick":9}"#,
        "\n"
    );

    const DELTA_LINE: &str = concat!(
        r#"{"type":"delta","tick":10,"tiles":[{"pos":[1,0,0],"tile":{"solid":"stone"}}],"#,
        r#""entities":[{"id":8,"kind":"dwarf","pos":[1,0,0],"state":"walk"}],"#,
        r#""designations":[],"zones":[],"speed":"fast"}"#,
        "\n"
    );

    #[test]
    fn reads_one_snapshot_line() {
        let mut reader = Cursor::new(SNAPSHOT_LINE.as_bytes());

        let snapshot = read_snapshot(&mut reader).unwrap();

        assert_eq!(snapshot.msg_type, MessageType::Snapshot);
        assert_eq!(snapshot.dims, Dims { x: 2, y: 1, z: 1 });
        assert_eq!(
            snapshot.tiles,
            vec![Tile::Empty, Tile::Solid(Material::Ice)]
        );
        assert_eq!(snapshot.entities[0].state, JobState::Idle);
        assert_eq!(snapshot.tick, 9);
        assert_eq!(reader.position(), SNAPSHOT_LINE.len() as u64);
    }

    #[test]
    fn reads_one_delta_line() {
        let mut reader = Cursor::new(DELTA_LINE.as_bytes());

        let message = read_message(&mut reader).unwrap().unwrap();

        let Msg::Delta(delta) = message else {
            panic!("delta line decoded as a snapshot");
        };
        assert_eq!(delta.tick, 10);
        assert_eq!(delta.tiles[0].pos, [1, 0, 0]);
        assert_eq!(delta.entities[0].state, JobState::Walk);
        assert_eq!(reader.position(), DELTA_LINE.len() as u64);
    }

    #[test]
    fn applies_dirty_tiles_and_replaces_authoritative_fields() {
        let mut snapshot = read_snapshot(&mut Cursor::new(SNAPSHOT_LINE.as_bytes())).unwrap();
        snapshot.designations = vec![()];
        snapshot.zones = vec![()];
        let delta = Delta {
            msg_type: MessageType::Delta,
            tick: 10,
            tiles: vec![TileChange {
                pos: [1, 0, 0],
                tile: Tile::Solid(Material::Stone),
            }],
            entities: vec![Entity {
                id: 8,
                kind: EntityKind::Dwarf,
                pos: [1, 0, 0],
                state: JobState::Walk,
            }],
            designations: Vec::new(),
            zones: Vec::new(),
            speed: Speed::Fast,
        };

        apply(&mut snapshot, delta);

        assert_eq!(
            snapshot.tiles,
            vec![Tile::Empty, Tile::Solid(Material::Stone)]
        );
        assert_eq!(snapshot.entities[0].id, 8);
        assert!(snapshot.designations.is_empty());
        assert!(snapshot.zones.is_empty());
        assert_eq!(snapshot.speed, Speed::Fast);
        assert_eq!(snapshot.tick, 10);
    }

    #[test]
    fn rejects_a_garbage_snapshot_line() {
        let mut reader = Cursor::new(b"not json\n");

        assert!(read_snapshot(&mut reader).is_err());
    }

    #[test]
    fn rejects_an_empty_reader() {
        let mut reader = Cursor::new(Vec::<u8>::new());

        assert!(read_snapshot(&mut reader).is_err());
    }

    /// Decodable but inconsistent: `render` would index past the end of `tiles`.
    #[test]
    fn rejects_a_snapshot_whose_tiles_do_not_match_dims() {
        const SHORT: &str = concat!(
            r#"{"type":"snapshot","dims":{"x":4,"y":4,"z":4},"#,
            r#""tiles":["empty",{"solid":"ice"}],"entities":[],"#,
            r#""designations":[],"zones":[],"speed":"normal","tick":0}"#,
            "\n"
        );
        let mut reader = Cursor::new(SHORT.as_bytes());

        let error = read_snapshot(&mut reader).unwrap_err().to_string();

        assert!(
            error.contains("2 tiles") && error.contains("64"),
            "error should name the mismatch, got: {error}"
        );
    }

    #[test]
    fn rejects_a_snapshot_without_a_line_terminator() {
        let mut reader = Cursor::new(SNAPSHOT_LINE.trim_end().as_bytes());

        assert!(read_snapshot(&mut reader).is_err());
    }
}
