use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};

use crate::{palette::BACKGROUND, view::Framebuffer};

pub fn write_frame(mut out: &mut dyn Write, framebuffer: &Framebuffer) -> io::Result<()> {
    if framebuffer.w == 0 || framebuffer.h == 0 {
        return Ok(());
    }

    queue!(
        &mut out,
        MoveTo(0, 0),
        SetBackgroundColor(Color::Rgb {
            r: BACKGROUND.0,
            g: BACKGROUND.1,
            b: BACKGROUND.2,
        })
    )?;
    let mut previous_fg = None;
    for y in 0..framebuffer.h {
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
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::style::Colored;

    use crate::palette::Cell;

    use super::*;

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

        write_frame(&mut bytes, &framebuffer).unwrap();

        assert_eq!(
            bytes,
            b"\x1b[1;1H\x1b[48;2;8;10;14m\x1b[38;2;1;2;3mA\x1b[38;2;4;5;6mB"
        );
    }
}
