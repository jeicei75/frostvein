#![forbid(unsafe_code)]

mod frame;
mod palette;
mod view;

use std::{
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::TcpStream,
    time::Duration,
};

use anyhow::{Context, bail};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyEventKind},
    execute,
    style::ResetColor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use protocol::Snapshot;

use crate::{
    frame::{RowEnd, write_frame},
    view::{Action, apply_key, initial, render},
};

/// Mirrors the daemon's 30 s write timeout. Without it a peer that accepts and
/// then goes silent leaves the client blocked forever.
const SNAPSHOT_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Caps the snapshot line so a server that never sends a newline cannot grow
/// the buffer without bound. The 128x128x32 snapshot is ~6.9 MB.
const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;

/// Sized so one frame reaches the terminal in a single write (AC10).
// NOTE: a terminal larger than roughly 25 000 cells splits across writes again.
const FRAME_BUFFER_BYTES: usize = 512 * 1024;

struct TerminalGuard;

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
    for arg in std::env::args_os().skip(1) {
        if arg == "--frame" {
            frame_only = true;
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

    let address = format!("127.0.0.1:{port}");
    let stream = TcpStream::connect(("127.0.0.1", port))
        .with_context(|| format!("could not connect to {address}"))?;
    stream
        .set_read_timeout(Some(SNAPSHOT_READ_TIMEOUT))
        .context("could not set the snapshot read timeout")?;
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader)?;
    let mut state = initial(&snapshot);

    if frame_only {
        let (w, h) = match terminal::size() {
            Ok((w, h)) if w > 0 && h > 0 => (w, h),
            _ => (100, 40),
        };
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

    loop {
        if needs_redraw {
            if size.0 != 0 && size.1 != 0 {
                let framebuffer = render(&snapshot, &state, size.0, size.1);
                write_frame(&mut out, &framebuffer, RowEnd::MoveTo)
                    .context("could not write terminal frame")?;
                out.flush().context("could not flush terminal frame")?;
            }
            needs_redraw = false;
        }

        match event::read().context("could not read terminal event")? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match apply_key(&mut state, key, snapshot.dims) {
                    Action::Redraw => needs_redraw = true,
                    Action::Quit => break,
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

    Ok(())
}

fn read_snapshot(reader: &mut dyn BufRead) -> anyhow::Result<Snapshot> {
    let mut line = String::new();
    let bytes = reader
        .take(MAX_SNAPSHOT_BYTES)
        .read_line(&mut line)
        .context("could not read snapshot line")?;
    if bytes == 0 {
        bail!("server closed before sending a snapshot");
    }
    if !line.ends_with('\n') {
        if bytes as u64 >= MAX_SNAPSHOT_BYTES {
            bail!("snapshot line exceeded {MAX_SNAPSHOT_BYTES} bytes with no newline");
        }
        bail!("server closed before terminating the snapshot line");
    }
    let snapshot: Snapshot = serde_json::from_str(&line).context("could not decode snapshot")?;

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

    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use protocol::{Dims, Material, MessageType, Tile};

    use super::*;

    const SNAPSHOT_LINE: &str = concat!(
        r#"{"type":"snapshot","dims":{"x":2,"y":1,"z":1},"#,
        r#""tiles":["empty",{"solid":"ice"}],"entities":[],"#,
        r#""designations":[],"zones":[],"speed":"normal","tick":9}"#,
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
        assert_eq!(snapshot.tick, 9);
        assert_eq!(reader.position(), SNAPSHOT_LINE.len() as u64);
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
