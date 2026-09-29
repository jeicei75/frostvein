use protocol::{DesignationKind, DwarfColour, EntityKind, JobState, Material, Tile};

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
        Tile::Solid(Material::TreeTrunk) => Cell {
            glyph: '│',
            fg: (105, 76, 48),
        },
        Tile::Solid(Material::TreeFoliage) => Cell {
            glyph: '♠',
            fg: (54, 106, 78),
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
        Tile::Ramp(Material::TreeTrunk) => Cell {
            glyph: '▲',
            fg: (105, 76, 48),
        },
        Tile::Ramp(Material::TreeFoliage) => Cell {
            glyph: '▲',
            fg: (54, 106, 78),
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
        (EntityKind::Torch, _) => Cell {
            glyph: '†',
            fg: (246, 166, 62),
        },
        (EntityKind::Campfire, _) => Cell {
            glyph: '♨',
            fg: (244, 92, 40),
        },
    }
}

/// The dwarf's tunic colour, from the approved 12.2 look draft. Used only for the roster row;
/// the `☺` glyph keeps its job-state colour.
pub fn dwarf_colour(colour: DwarfColour) -> Rgb {
    match colour {
        DwarfColour::Red => (0xB2, 0x3A, 0x34),
        DwarfColour::Gold => (0xD6, 0xA4, 0x2C),
        DwarfColour::Green => (0x3E, 0x92, 0x4C),
        DwarfColour::Blue => (0x3C, 0x62, 0xBA),
        DwarfColour::Purple => (0x80, 0x4C, 0xA8),
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

/// A stone on a stockpile cell: the stone glyph in the stockpile's colour, so a full pile still
/// reads as a pile once every `≡` is covered.
pub fn stored_item_cell() -> Cell {
    Cell {
        glyph: item_cell().glyph,
        fg: zone_cell().fg,
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

pub fn dim(fg: Rgb, depth: u8) -> Rgb {
    if depth == 0 {
        return fg;
    }
    let percent = DIM_PERCENT[usize::from(depth - 1)];
    (
        (u16::from(fg.0) * percent / 100) as u8,
        (u16::from(fg.1) * percent / 100) as u8,
        (u16::from(fg.2) * percent / 100) as u8,
    )
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
            (Tile::Solid(Material::TreeTrunk), '│', (105, 76, 48)),
            (Tile::Solid(Material::TreeFoliage), '♠', (54, 106, 78)),
            (Tile::Ramp(Material::Stone), '▲', (86, 92, 104)),
            (Tile::Ramp(Material::Soil), '▲', (72, 66, 58)),
            (Tile::Ramp(Material::Ice), '▲', (126, 174, 196)),
            (Tile::Ramp(Material::Snow), '▲', (206, 218, 228)),
            (Tile::Ramp(Material::TreeTrunk), '▲', (105, 76, 48)),
            (Tile::Ramp(Material::TreeFoliage), '▲', (54, 106, 78)),
            (Tile::Empty, ' ', (8, 10, 14)),
        ];
        for (tile, glyph, fg) in tiles {
            assert_eq!(tile_cell(tile), Cell { glyph, fg });
        }

        assert_eq!(
            entity_cell(EntityKind::Torch, JobState::Work),
            Cell {
                glyph: '†',
                fg: (246, 166, 62),
            }
        );
        assert_eq!(
            entity_cell(EntityKind::Campfire, JobState::Walk),
            Cell {
                glyph: '♨',
                fg: (244, 92, 40),
            }
        );

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

        for (colour, fg) in [
            (DwarfColour::Red, (178, 58, 52)),
            (DwarfColour::Gold, (214, 164, 44)),
            (DwarfColour::Green, (62, 146, 76)),
            (DwarfColour::Blue, (60, 98, 186)),
            (DwarfColour::Purple, (128, 76, 168)),
        ] {
            assert_eq!(dwarf_colour(colour), fg);
        }

        assert_eq!(
            item_cell(),
            Cell {
                glyph: '*',
                fg: (176, 172, 160),
            }
        );
        assert_eq!(
            stored_item_cell(),
            Cell {
                glyph: '*',
                fg: (88, 190, 118),
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

        let existing_glyphs = ['█', '▓', '▒', '░', '▲', ' ', '☺', '│', '♠', '†', '♨'];
        let new_glyphs = ['│', '♠', '†', '♨'];
        assert_eq!(
            new_glyphs
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            new_glyphs.len(),
            "tree and emitter glyphs must all be distinct"
        );
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
