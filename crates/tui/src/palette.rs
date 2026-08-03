use protocol::{EntityKind, JobState, Material, Tile};

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
