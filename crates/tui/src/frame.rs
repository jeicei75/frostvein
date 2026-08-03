use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use crate::{palette::BACKGROUND, view::Framebuffer};

/// How one row is separated from the next.
///
/// The interactive view repaints a fixed grid in place, so it anchors every row
/// with `MoveTo`. `--frame` writes a stream meant to be piped or captured
/// (`--frame | head -45`), so it separates rows with newlines and never moves
/// the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowEnd {
    MoveTo,
    Newline,
}

pub fn write_frame(
    mut out: &mut dyn Write,
    framebuffer: &Framebuffer,
    row_end: RowEnd,
) -> io::Result<()> {
    if framebuffer.w == 0 || framebuffer.h == 0 {
        return Ok(());
    }

    queue!(
        &mut out,
        SetBackgroundColor(Color::Rgb {
            r: BACKGROUND.0,
            g: BACKGROUND.1,
            b: BACKGROUND.2,
        })
    )?;
    let mut previous_fg = None;
    for y in 0..framebuffer.h {
        // NOTE: anchoring each row keeps a double-width glyph or a stale
        // terminal width from shearing every row below it. Auto-wrap alone
        // never re-anchors.
        if row_end == RowEnd::MoveTo {
            queue!(&mut out, MoveTo(0, y))?;
        }
        for x in 0..framebuffer.w {
            let cell = framebuffer.cell(x, y);
            if previous_fg != Some(cell.fg) {
                queue!(
                    &mut out,
                    SetForegroundColor(Color::Rgb {
                        r: cell.fg.0,
                        g: cell.fg.1,
                        b: cell.fg.2,
                    })
                )?;
                previous_fg = Some(cell.fg);
            }
            queue!(&mut out, Print(cell.glyph))?;
        }
        if row_end == RowEnd::Newline {
            queue!(&mut out, Print('\n'))?;
        }
    }
    // Without this the caller's shell inherits the frame's near-black fg and bg
    // and the next prompt is invisible.
    queue!(&mut out, ResetColor)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::style::Colored;

    use crate::palette::Cell;

    use super::*;

    fn two_by_two() -> Framebuffer {
        Framebuffer {
            w: 2,
            h: 2,
            cells: vec![
                Cell {
                    glyph: 'A',
                    fg: (1, 2, 3),
                },
                Cell {
                    glyph: 'B',
                    fg: (4, 5, 6),
                },
                Cell {
                    glyph: 'C',
                    fg: (4, 5, 6),
                },
                Cell {
                    glyph: 'D',
                    fg: (1, 2, 3),
                },
            ],
        }
    }

    #[test]
    fn frame_bytes_are_pinned() {
        Colored::set_ansi_color_disabled(false);
        let framebuffer = Framebuffer {
            w: 2,
            h: 1,
            cells: vec![
                Cell {
                    glyph: 'A',
                    fg: (1, 2, 3),
                },
                Cell {
                    glyph: 'B',
                    fg: (4, 5, 6),
                },
            ],
        };
        let mut bytes = Vec::new();

        write_frame(&mut bytes, &framebuffer, RowEnd::MoveTo).unwrap();

        assert_eq!(
            bytes,
            b"\x1b[48;2;8;10;14m\x1b[1;1H\x1b[38;2;1;2;3mA\x1b[38;2;4;5;6mB\x1b[0m"
        );
    }

    /// Every row is anchored, and a colour survives across the row boundary
    /// without being re-emitted.
    #[test]
    fn every_row_is_anchored() {
        Colored::set_ansi_color_disabled(false);
        let mut bytes = Vec::new();

        write_frame(&mut bytes, &two_by_two(), RowEnd::MoveTo).unwrap();

        assert_eq!(
            bytes,
            b"\x1b[48;2;8;10;14m\
              \x1b[1;1H\x1b[38;2;1;2;3mA\x1b[38;2;4;5;6mB\
              \x1b[2;1HC\x1b[38;2;1;2;3mD\
              \x1b[0m"
        );
    }

    /// The `--frame` stream is capturable: rows end with `\n` and the cursor is
    /// never moved, so piping it to `head` shows the picture.
    #[test]
    fn newline_rows_are_pinned() {
        Colored::set_ansi_color_disabled(false);
        let mut bytes = Vec::new();

        write_frame(&mut bytes, &two_by_two(), RowEnd::Newline).unwrap();

        assert_eq!(
            bytes,
            b"\x1b[48;2;8;10;14m\
              \x1b[38;2;1;2;3mA\x1b[38;2;4;5;6mB\n\
              C\x1b[38;2;1;2;3mD\n\
              \x1b[0m"
        );
    }
}
