use protocol::{DesignationKind, EntityKind, JobState, Material, Tile};

use crate::view::Mode;

pub type Rgb = (u8, u8, u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub glyph: char,
    pub fg: Rgb,
}

pub const PEEK_DEPTH: usize = 3;
pub const BACKGROUND: Rgb = (8, 10, 14);
pub const BLANK: Cell = Cell {
    glyph: ' ',
    fg: BACKGROUND,
};
pub const STATUS_TEXT: Rgb = (150, 160, 170);

const DIM_PERCENT: [u16; PEEK_DEPTH] = [55, 35, 22];

pub fn tile_cell(tile: Tile) -> Cell {
    match tile {
        Tile::Empty => BLANK,
        Tile::Solid(Material::Stone) => Cell {
            glyph: '█',
            fg: (86, 92, 104),
        },
        Tile::Solid(Material::Soil) => Cell {
            glyph: '▓',
            fg: (72, 66, 58),
        },
        Tile::Solid(Material::Ice) => Cell {
            glyph: '▒',
            fg: (126, 174, 196),
        },
        Tile::Solid(Material::Snow) => Cell {
            glyph: '░',
            fg: (206, 218, 228),
        },
        Tile::Ramp(Material::Stone) => Cell {
            glyph: '▲',
            fg: (86, 92, 104),
        },
        Tile::Ramp(Material::Soil) => Cell {
            glyph: '▲',
            fg: (72, 66, 58),
        },
        Tile::Ramp(Material::Ice) => Cell {
            glyph: '▲',
            fg: (126, 174, 196),
        },
        Tile::Ramp(Material::Snow) => Cell {
            glyph: '▲',
            fg: (206, 218, 228),
        },
    }
}

pub fn entity_cell(kind: EntityKind, state: JobState) -> Cell {
    match (kind, state) {
        (EntityKind::Dwarf, JobState::Idle) => Cell {
            glyph: '☺',
            fg: (150, 112, 62),
        },
        (EntityKind::Dwarf, JobState::Walk) => Cell {
            glyph: '☺',
            fg: (214, 154, 78),
        },
        (EntityKind::Dwarf, JobState::Work) => Cell {
            glyph: '☺',
            fg: (236, 186, 96),
        },
    }
}

pub fn designation_cell(kind: DesignationKind) -> Cell {
    match kind {
        DesignationKind::Dig => Cell {
            glyph: '×',
            fg: (232, 176, 72),
        },
        DesignationKind::Channel => Cell {
            glyph: '▼',
            fg: (92, 174, 224),
        },
    }
}

pub fn zone_cell() -> Cell {
    Cell {
        glyph: '≡',
        fg: (88, 190, 118),
    }
}

pub fn item_cell() -> Cell {
    Cell {
        glyph: '*',
        fg: (176, 172, 160),
    }
}

/// One dwarf sharing a cell with one or more stones — the loaded twin of `☺`.
// NOTE: the glyph states co-location, which is a carry in every case the sim produces except a
// dwarf standing on a loose stone it does not hold.
pub fn carrier_cell() -> Cell {
    Cell {
        glyph: '☻',
        fg: (226, 198, 140),
    }
}

pub fn crowd_cell() -> Cell {
    Cell {
        glyph: '⚇',
        fg: (240, 120, 130),
    }
}

pub fn cursor_cell() -> Cell {
    Cell {
        glyph: '+',
        fg: (246, 242, 226),
    }
}

pub fn pending_rect_cell(mode: Mode) -> Cell {
    match mode {
        Mode::Dig => Cell {
            glyph: 'd',
            fg: (218, 142, 54),
        },
        Mode::Channel => Cell {
            glyph: 'c',
            fg: (70, 148, 202),
        },
        Mode::Stockpile => Cell {
            glyph: 'p',
            fg: (64, 166, 96),
        },
        Mode::Remove => Cell {
            glyph: '-',
            fg: (218, 82, 82),
        },
        Mode::Normal => BLANK,
    }
}

/// Scales a colour towards black. `percent` is 0..=100; 100 returns the colour
/// unchanged.
pub fn shade(fg: Rgb, percent: u16) -> Rgb {
    (
        (u16::from(fg.0) * percent / 100) as u8,
        (u16::from(fg.1) * percent / 100) as u8,
        (u16::from(fg.2) * percent / 100) as u8,
    )
}

pub fn dim(fg: Rgb, depth: u8) -> Rgb {
    if depth == 0 {
        return fg;
    }
    shade(fg, DIM_PERCENT[usize::from(depth - 1)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_look_is_pinned() {
        let tiles = [
            (Tile::Solid(Material::Stone), '█', (86, 92, 104)),
            (Tile::Solid(Material::Soil), '▓', (72, 66, 58)),
            (Tile::Solid(Material::Ice), '▒', (126, 174, 196)),
            (Tile::Solid(Material::Snow), '░', (206, 218, 228)),
            (Tile::Ramp(Material::Stone), '▲', (86, 92, 104)),
            (Tile::Ramp(Material::Soil), '▲', (72, 66, 58)),
            (Tile::Ramp(Material::Ice), '▲', (126, 174, 196)),
            (Tile::Ramp(Material::Snow), '▲', (206, 218, 228)),
            (Tile::Empty, ' ', (8, 10, 14)),
        ];
        for (tile, glyph, fg) in tiles {
            assert_eq!(tile_cell(tile), Cell { glyph, fg });
        }

        for (state, fg) in [
            (JobState::Idle, (150, 112, 62)),
            (JobState::Walk, (214, 154, 78)),
            (JobState::Work, (236, 186, 96)),
        ] {
            assert_eq!(
                entity_cell(EntityKind::Dwarf, state),
                Cell { glyph: '☺', fg }
            );
        }

        assert_eq!(
            item_cell(),
            Cell {
                glyph: '*',
                fg: (176, 172, 160),
            }
        );
        assert_eq!(
            crowd_cell(),
            Cell {
                glyph: '⚇',
                fg: (240, 120, 130),
            }
        );
        assert_eq!(
            carrier_cell(),
            Cell {
                glyph: '☻',
                fg: (226, 198, 140),
            }
        );

        let markers = [
            designation_cell(DesignationKind::Dig),
            designation_cell(DesignationKind::Channel),
            zone_cell(),
            cursor_cell(),
            pending_rect_cell(Mode::Dig),
            pending_rect_cell(Mode::Channel),
            pending_rect_cell(Mode::Stockpile),
            pending_rect_cell(Mode::Remove),
            item_cell(),
            crowd_cell(),
            carrier_cell(),
        ];
        assert_eq!(
            markers,
            [
                Cell {
                    glyph: '×',
                    fg: (232, 176, 72),
                },
                Cell {
                    glyph: '▼',
                    fg: (92, 174, 224),
                },
                Cell {
                    glyph: '≡',
                    fg: (88, 190, 118),
                },
                Cell {
                    glyph: '+',
                    fg: (246, 242, 226),
                },
                Cell {
                    glyph: 'd',
                    fg: (218, 142, 54),
                },
                Cell {
                    glyph: 'c',
                    fg: (70, 148, 202),
                },
                Cell {
                    glyph: 'p',
                    fg: (64, 166, 96),
                },
                Cell {
                    glyph: '-',
                    fg: (218, 82, 82),
                },
                Cell {
                    glyph: '*',
                    fg: (176, 172, 160),
                },
                Cell {
                    glyph: '⚇',
                    fg: (240, 120, 130),
                },
                Cell {
                    glyph: '☻',
                    fg: (226, 198, 140),
                },
            ]
        );

        let existing_glyphs = ['█', '▓', '▒', '░', '▲', ' ', '☺'];
        let marker_glyphs: std::collections::BTreeSet<_> =
            markers.iter().map(|cell| cell.glyph).collect();
        assert_eq!(
            marker_glyphs.len(),
            markers.len(),
            "every marker must remain distinct by glyph alone"
        );
        assert!(
            marker_glyphs
                .iter()
                .all(|glyph| !existing_glyphs.contains(glyph)),
            "marker glyphs must not collide with terrain or entities"
        );
    }

    #[test]
    fn dim_darkens_monotonically() {
        let fg = (200, 160, 100);
        let expected = [(110, 88, 55), (70, 56, 35), (44, 35, 22)];

        assert_eq!(dim(fg, 0), fg);
        let mut previous = fg;
        for (depth, expected) in (1..=3).zip(expected) {
            let actual = dim(fg, depth);
            assert_eq!(actual, expected);
            assert!(actual.0 < previous.0);
            assert!(actual.1 < previous.1);
            assert!(actual.2 < previous.2);
            previous = actual;
        }
    }
}
