use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use protocol::{Dims, EntityKind, Snapshot, Tile};

use crate::palette::{BLANK, Cell, PEEK_DEPTH, STATUS_TEXT, dim, entity_cell, tile_cell};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Framebuffer {
    pub w: u16,
    pub h: u16,
    pub cells: Vec<Cell>,
}

impl Framebuffer {
    pub fn cell(&self, x: u16, y: u16) -> Cell {
        self.cells[usize::from(x) + usize::from(y) * usize::from(self.w)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewState {
    pub camera: (i64, i64),
    pub z: i32,
    pub confirming_quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Redraw,
    Quit,
    Ignore,
}

pub fn initial(snapshot: &Snapshot) -> ViewState {
    // NOTE: clamped because the entity position is wire data. An out-of-world
    // entity would otherwise open on a blank map with no way to tell why.
    let max_x = i64::from(snapshot.dims.x.saturating_sub(1));
    let max_y = i64::from(snapshot.dims.y.saturating_sub(1));
    let max_z = i32::try_from(snapshot.dims.z.saturating_sub(1)).unwrap_or(i32::MAX);
    match snapshot.entities.first() {
        Some(entity) => ViewState {
            camera: (
                i64::from(entity.pos[0]).clamp(0, max_x),
                i64::from(entity.pos[1]).clamp(0, max_y),
            ),
            z: entity.pos[2].clamp(0, max_z),
            confirming_quit: false,
        },
        None => ViewState {
            camera: (
                i64::from(snapshot.dims.x / 2),
                i64::from(snapshot.dims.y / 2),
            ),
            z: i32::try_from(snapshot.dims.z / 2).unwrap_or(i32::MAX),
            confirming_quit: false,
        },
    }
}

pub fn render(snapshot: &Snapshot, state: &ViewState, w: u16, h: u16) -> Framebuffer {
    let mut framebuffer = Framebuffer {
        w,
        h,
        cells: vec![BLANK; usize::from(w) * usize::from(h)],
    };
    if w == 0 || h == 0 {
        return framebuffer;
    }

    let map_h = h - 1;
    for sy in 0..map_h {
        let wy = state.camera.1 + i64::from(sy) - i64::from(map_h) / 2;
        for sx in 0..w {
            let wx = state.camera.0 + i64::from(sx) - i64::from(w) / 2;
            if wx < 0
                || wy < 0
                || wx >= i64::from(snapshot.dims.x)
                || wy >= i64::from(snapshot.dims.y)
                || state.z < 0
                || i64::from(state.z) >= i64::from(snapshot.dims.z)
            {
                continue;
            }

            let x = wx as u32;
            let y = wy as u32;
            let z = state.z as u32;
            let tile = snapshot.tiles[tile_index(snapshot.dims, x, y, z)];
            let mut cell = tile_cell(tile);
            if tile == Tile::Empty {
                for depth in 1..=PEEK_DEPTH {
                    let Some(below_z) = z.checked_sub(depth as u32) else {
                        break;
                    };
                    let below = snapshot.tiles[tile_index(snapshot.dims, x, y, below_z)];
                    if below != Tile::Empty {
                        cell = tile_cell(below);
                        cell.fg = dim(cell.fg, depth as u8);
                        break;
                    }
                }
            }
            framebuffer.cells[usize::from(sx) + usize::from(sy) * usize::from(w)] = cell;
        }
    }

    for entity in &snapshot.entities {
        if entity.pos[2] != state.z
            || entity.pos[0] < 0
            || entity.pos[1] < 0
            || i64::from(entity.pos[0]) >= i64::from(snapshot.dims.x)
            || i64::from(entity.pos[1]) >= i64::from(snapshot.dims.y)
        {
            continue;
        }
        let sx = i64::from(entity.pos[0]) - state.camera.0 + i64::from(w) / 2;
        let sy = i64::from(entity.pos[1]) - state.camera.1 + i64::from(map_h) / 2;
        if sx >= 0 && sx < i64::from(w) && sy >= 0 && sy < i64::from(map_h) {
            framebuffer.cells[sx as usize + sy as usize * usize::from(w)] =
                entity_cell(entity.kind);
        }
    }

    let status = if state.confirming_quit {
        "quit? (y/n)".to_string()
    } else {
        let dwarves = snapshot
            .entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Dwarf)
            .count();
        format!(
            "tick {}  z {}/{}  camera {},{}  dwarves {}  keys: <> z  arrows/hjkl pan  q quit",
            snapshot.tick,
            state.z,
            snapshot.dims.z.saturating_sub(1),
            state.camera.0,
            state.camera.1,
            dwarves
        )
    };
    let status_y = h - 1;
    for (x, glyph) in (0..w).zip(status.chars()) {
        framebuffer.cells[usize::from(x) + usize::from(status_y) * usize::from(w)] = Cell {
            glyph,
            fg: STATUS_TEXT,
        };
    }

    framebuffer
}

pub fn apply_key(state: &mut ViewState, key: KeyEvent, dims: Dims) -> Action {
    // Wolf's call 2026-08-03: Ctrl-C quits outright. Raw mode clears ISIG, so
    // without this the conventional interrupt does nothing at all and the only
    // way out is q -> y.
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Action::Quit;
    }
    // NOTE: SHIFT is the only modifier the keymap uses (`<` and `>` arrive with
    // it). Without this, Ctrl-H/J/K/L would pan and Ctrl-Q would open the quit
    // prompt.
    if !key.modifiers.difference(KeyModifiers::SHIFT).is_empty() {
        return Action::Ignore;
    }

    if state.confirming_quit {
        return if key.code == KeyCode::Char('y') {
            Action::Quit
        } else {
            state.confirming_quit = false;
            Action::Redraw
        };
    }

    match key.code {
        KeyCode::Char('<') => {
            state.z = (state.z - 1).max(0);
            Action::Redraw
        }
        KeyCode::Char('>') => {
            state.z = (state.z + 1).min(dims.z.saturating_sub(1) as i32);
            Action::Redraw
        }
        KeyCode::Left | KeyCode::Char('h') => {
            state.camera.0 = (state.camera.0 - 1).max(0);
            Action::Redraw
        }
        KeyCode::Right | KeyCode::Char('l') => {
            state.camera.0 = (state.camera.0 + 1).min(i64::from(dims.x.saturating_sub(1)));
            Action::Redraw
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.camera.1 = (state.camera.1 - 1).max(0);
            Action::Redraw
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.camera.1 = (state.camera.1 + 1).min(i64::from(dims.y.saturating_sub(1)));
            Action::Redraw
        }
        KeyCode::Char('q') => {
            state.confirming_quit = true;
            Action::Redraw
        }
        _ => Action::Ignore,
    }
}

// NOTE: widened before multiplying — the strides come from the wire, and a u32
// product would overflow before the caller's bounds check ever sees it.
fn tile_index(dims: Dims, x: u32, y: u32, z: u32) -> usize {
    x as usize + y as usize * dims.x as usize + z as usize * dims.x as usize * dims.y as usize
}

#[cfg(test)]
mod tests {
    use protocol::{Entity, EntityKind, JobState, Material, MessageType, Speed, Tile};

    use super::*;

    fn empty_snapshot(dims: Dims) -> Snapshot {
        Snapshot {
            msg_type: MessageType::Snapshot,
            dims,
            tiles: vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        }
    }

    fn index(dims: Dims, x: u32, y: u32, z: u32) -> usize {
        (x + y * dims.x + z * dims.x * dims.y) as usize
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn renders_the_viewed_level() {
        let dims = Dims { x: 5, y: 3, z: 3 };
        let mut snapshot = empty_snapshot(dims);
        snapshot.tiles[index(dims, 0, 0, 2)] = Tile::Solid(Material::Stone);
        snapshot.tiles[index(dims, 1, 0, 2)] = Tile::Ramp(Material::Snow);
        snapshot.tiles[index(dims, 2, 0, 1)] = Tile::Solid(Material::Ice);
        snapshot.tiles[index(dims, 3, 0, 0)] = Tile::Solid(Material::Soil);
        snapshot.tiles[index(dims, 0, 1, 2)] = Tile::Solid(Material::Snow);
        snapshot.tiles[index(dims, 4, 1, 2)] = Tile::Ramp(Material::Stone);
        snapshot.tiles[index(dims, 2, 2, 2)] = Tile::Solid(Material::Ice);
        let state = ViewState {
            camera: (2, 1),
            z: 2,
            confirming_quit: false,
        };

        let framebuffer = render(&snapshot, &state, 7, 4);
        let expected = [
            (' ', (8, 10, 14)),
            ('█', (86, 92, 104)),
            ('▲', (206, 218, 228)),
            ('▒', (69, 95, 107)),
            ('▓', (25, 23, 20)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            ('░', (206, 218, 228)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            ('▲', (86, 92, 104)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            ('▒', (126, 174, 196)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            (' ', (8, 10, 14)),
            ('t', (150, 160, 170)),
            ('i', (150, 160, 170)),
            ('c', (150, 160, 170)),
            ('k', (150, 160, 170)),
            (' ', (150, 160, 170)),
            ('0', (150, 160, 170)),
            (' ', (150, 160, 170)),
        ];
        let actual: Vec<_> = framebuffer
            .cells
            .iter()
            .map(|cell| (cell.glyph, cell.fg))
            .collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn entities_draw_only_on_the_viewed_level() {
        let dims = Dims { x: 5, y: 3, z: 3 };
        let mut snapshot = empty_snapshot(dims);
        snapshot.tiles[index(dims, 1, 1, 1)] = Tile::Solid(Material::Soil);
        snapshot.tiles[index(dims, 3, 1, 1)] = Tile::Solid(Material::Ice);
        snapshot.entities = vec![
            Entity {
                id: 1,
                kind: EntityKind::Dwarf,
                pos: [1, 1, 1],
                state: JobState::Idle,
            },
            Entity {
                id: 2,
                kind: EntityKind::Dwarf,
                pos: [3, 1, 2],
                state: JobState::Idle,
            },
        ];
        let state = ViewState {
            camera: (2, 1),
            z: 1,
            confirming_quit: false,
        };

        let framebuffer = render(&snapshot, &state, 5, 4);

        assert_eq!(
            framebuffer.cell(1, 1),
            Cell {
                glyph: '☺',
                fg: (214, 154, 78),
            }
        );
        assert_eq!(
            framebuffer.cell(3, 1),
            Cell {
                glyph: '▒',
                fg: (126, 174, 196),
            }
        );
        assert_eq!(
            framebuffer
                .cells
                .iter()
                .filter(|cell| cell.glyph == '☺')
                .count(),
            1
        );
    }

    /// The peek-below cap itself, not just its dimming: ground exactly
    /// `PEEK_DEPTH` levels down is drawn, one level deeper is not. Widening
    /// `PEEK_DEPTH` must turn this red.
    #[test]
    fn peek_below_stops_at_three_levels() {
        let dims = Dims { x: 2, y: 1, z: 8 };
        let mut snapshot = empty_snapshot(dims);
        snapshot.tiles[index(dims, 0, 0, 4)] = Tile::Solid(Material::Snow);
        snapshot.tiles[index(dims, 1, 0, 3)] = Tile::Solid(Material::Snow);
        let state = ViewState {
            camera: (1, 0),
            z: 7,
            confirming_quit: false,
        };

        let framebuffer = render(&snapshot, &state, 4, 2);

        assert_eq!(
            framebuffer.cell(1, 0),
            Cell {
                glyph: '░',
                fg: (45, 47, 50),
            }
        );
        assert_eq!(framebuffer.cell(2, 0), BLANK);
    }

    #[test]
    fn status_line_reports_z_camera_and_dwarf_count() {
        let dims = Dims {
            x: 40,
            y: 40,
            z: 32,
        };
        let mut snapshot = empty_snapshot(dims);
        snapshot.tick = 87;
        snapshot.entities = (0..3)
            .map(|id| Entity {
                id,
                kind: EntityKind::Dwarf,
                pos: [1, 1, 30],
                state: JobState::Idle,
            })
            .collect();
        let state = ViewState {
            camera: (12, 34),
            z: 19,
            confirming_quit: false,
        };

        let framebuffer = render(&snapshot, &state, 78, 2);
        let status: String = (0..78).map(|x| framebuffer.cell(x, 1).glyph).collect();

        assert_eq!(
            status,
            "tick 87  z 19/31  camera 12,34  dwarves 3  keys: <> z  arrows/hjkl pan  q quit"
        );
    }

    #[test]
    fn modified_keys_are_ignored_except_ctrl_c_and_shift() {
        let dims = Dims { x: 3, y: 4, z: 2 };
        let mut state = ViewState {
            camera: (1, 1),
            z: 0,
            confirming_quit: false,
        };

        for code in [KeyCode::Char('l'), KeyCode::Char('j'), KeyCode::Char('q')] {
            assert_eq!(
                apply_key(&mut state, KeyEvent::new(code, KeyModifiers::CONTROL), dims),
                Action::Ignore
            );
        }
        assert_eq!((state.camera, state.confirming_quit), ((1, 1), false));

        // `<` and `>` arrive with SHIFT held on most layouts.
        assert_eq!(
            apply_key(
                &mut state,
                KeyEvent::new(KeyCode::Char('>'), KeyModifiers::SHIFT),
                dims
            ),
            Action::Redraw
        );
        assert_eq!(state.z, 1);

        assert_eq!(
            apply_key(
                &mut state,
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                dims
            ),
            Action::Quit
        );
    }

    #[test]
    fn out_of_world_cells_are_blank() {
        let dims = Dims { x: 3, y: 2, z: 1 };
        let mut snapshot = empty_snapshot(dims);
        snapshot.tiles[index(dims, 0, 0, 0)] = Tile::Solid(Material::Stone);
        let state = ViewState {
            camera: (0, 0),
            z: 0,
            confirming_quit: false,
        };

        let framebuffer = render(&snapshot, &state, 5, 4);

        assert_eq!(framebuffer.cell(2, 1).glyph, '█');
        for (x, y) in [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1)] {
            assert_eq!(framebuffer.cell(x, y), BLANK);
        }
    }

    #[test]
    fn keys_move_and_clamp() {
        let dims = Dims { x: 3, y: 4, z: 2 };
        let mut state = ViewState {
            camera: (2, 3),
            z: 1,
            confirming_quit: false,
        };

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('>')), dims),
            Action::Redraw
        );
        assert_eq!(state.z, 1);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Right), dims),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('l')), dims),
            Action::Redraw
        );
        assert_eq!(state.camera.0, 2);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Down), dims),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('j')), dims),
            Action::Redraw
        );
        assert_eq!(state.camera.1, 3);

        state.camera = (0, 0);
        state.z = 0;
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('<')), dims),
            Action::Redraw
        );
        assert_eq!(state.z, 0);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Left), dims),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('h')), dims),
            Action::Redraw
        );
        assert_eq!(state.camera.0, 0);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Up), dims),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('k')), dims),
            Action::Redraw
        );
        assert_eq!(state.camera.1, 0);

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('>')), dims),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('l')), dims),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('j')), dims),
            Action::Redraw
        );
        assert_eq!((state.camera, state.z), ((1, 1), 1));

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('q')), dims),
            Action::Redraw
        );
        assert!(state.confirming_quit);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('y')), dims),
            Action::Quit
        );

        state.confirming_quit = false;
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('q')), dims),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Esc), dims),
            Action::Redraw
        );
        assert!(!state.confirming_quit);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('x')), dims),
            Action::Ignore
        );
    }

    #[test]
    fn initial_view_uses_the_first_entity_or_world_middle() {
        let dims = Dims { x: 9, y: 7, z: 5 };
        let mut snapshot = empty_snapshot(dims);

        assert_eq!(
            initial(&snapshot),
            ViewState {
                camera: (4, 3),
                z: 2,
                confirming_quit: false,
            }
        );

        snapshot.entities.push(Entity {
            id: 7,
            kind: EntityKind::Dwarf,
            pos: [8, 1, 4],
            state: JobState::Idle,
        });
        assert_eq!(
            initial(&snapshot),
            ViewState {
                camera: (8, 1),
                z: 4,
                confirming_quit: false,
            }
        );
    }

    #[test]
    fn confirming_quit_replaces_the_status_line() {
        let snapshot = empty_snapshot(Dims { x: 1, y: 1, z: 1 });
        let state = ViewState {
            camera: (0, 0),
            z: 0,
            confirming_quit: true,
        };

        let framebuffer = render(&snapshot, &state, 11, 2);
        let status: String = (0..11).map(|x| framebuffer.cell(x, 1).glyph).collect();

        assert_eq!(status, "quit? (y/n)");
    }
}
