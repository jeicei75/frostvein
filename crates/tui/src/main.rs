#![forbid(unsafe_code)]

mod frame;
mod palette;
mod view;

use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

use anyhow::{Context, bail};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use protocol::Snapshot;

use crate::{
    frame::write_frame,
    view::{Action, apply_key, initial, render},
};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut out = io::stdout();
        let _ = execute!(out, LeaveAlternateScreen, Show);
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
            format!("invalid port argument {text:?}: expected 0-65535 (0 = OS-assigned)")
        })?;
        port_was_set = true;
    }

    let address = format!("127.0.0.1:{port}");
    let stream = TcpStream::connect(("127.0.0.1", port))
        .with_context(|| format!("could not connect to {address}"))?;
    let mut reader = BufReader::new(stream);
    let snapshot = read_snapshot(&mut reader)?;
    let mut state = initial(&snapshot);

    if frame_only {
        let (w, h) = terminal::size().unwrap_or((100, 40));
        let framebuffer = render(&snapshot, &state, w, h);
        let mut out = BufWriter::new(io::stdout());
        write_frame(&mut out, &framebuffer).context("could not write terminal frame")?;
        out.flush().context("could not flush terminal frame")?;
        return Ok(());
    }

    terminal::enable_raw_mode().context("could not enable terminal raw mode")?;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen, Hide) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
        let _ = terminal::disable_raw_mode();
        return Err(error).context("could not enter terminal view");
    }
    let _guard = TerminalGuard;
    let mut out = BufWriter::new(stdout);
    let mut size = terminal::size().context("could not read terminal size")?;
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            if size.0 != 0 && size.1 != 0 {
                let framebuffer = render(&snapshot, &state, size.0, size.1);
                write_frame(&mut out, &framebuffer).context("could not write terminal frame")?;
                out.flush().context("could not flush terminal frame")?;
            }
            needs_redraw = false;
        }

        match event::read().context("could not read terminal event")? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match apply_key(&mut state, key.code, snapshot.dims) {
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
        .read_line(&mut line)
        .context("could not read snapshot line")?;
    if bytes == 0 {
        bail!("server closed before sending a snapshot");
    }
    if !line.ends_with('\n') {
        bail!("server closed before terminating the snapshot line");
    }
    serde_json::from_str(&line).context("could not decode snapshot")
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

    #[test]
    fn rejects_a_snapshot_without_a_line_terminator() {
        let mut reader = Cursor::new(SNAPSHOT_LINE.trim_end().as_bytes());

        assert!(read_snapshot(&mut reader).is_err());
    }
}
