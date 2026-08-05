use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use protocol::{Command, Dims, EntityKind, Snapshot, Speed, Tile};

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
    Command(Command),
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
    if w == 0 || h < 2 {
        return framebuffer;
    }

    let map_h = h - 2;
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
                entity_cell(entity.kind, entity.state);
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
        let speed = match snapshot.speed {
            Speed::Paused => "paused",
            Speed::Normal => "normal",
            Speed::Fast => "fast",
        };
        format!(
            "tick {}  {}  z {}/{}  dwarves {}",
            snapshot.tick,
            speed,
            state.z,
            snapshot.dims.z.saturating_sub(1),
            dwarves
        )
    };
    let status_y = h - 2;
    for (x, glyph) in (0..w).zip(status.chars()) {
        framebuffer.cells[usize::from(x) + usize::from(status_y) * usize::from(w)] = Cell {
            glyph,
            fg: STATUS_TEXT,
        };
    }

    let hint = "d dig  c channel  p stockpile  x clear  <> z  hjkl move  q quit client";
    let hint_y = h - 1;
    for (x, glyph) in (0..w).zip(hint.chars()) {
        framebuffer.cells[usize::from(x) + usize::from(hint_y) * usize::from(w)] = Cell {
            glyph,
            fg: STATUS_TEXT,
        };
    }

    framebuffer
}

pub fn apply_key(state: &mut ViewState, key: KeyEvent, dims: Dims, speed: Speed) -> Action {
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

    let command = |speed| Action::Command(Command::SetSpeed { speed });
    match key.code {
        KeyCode::Char('S') => Action::Command(Command::Save),
        KeyCode::Char('L') => Action::Command(Command::Load),
        KeyCode::Char(' ') => command(match speed {
            Speed::Paused => Speed::Normal,
            Speed::Normal | Speed::Fast => Speed::Paused,
        }),
        KeyCode::Char('+') => match speed {
            Speed::Paused => command(Speed::Normal),
            Speed::Normal => command(Speed::Fast),
            Speed::Fast => Action::Ignore,
        },
        KeyCode::Char('-') => match speed {
            Speed::Fast => command(Speed::Normal),
            Speed::Normal => command(Speed::Paused),
            Speed::Paused => Action::Ignore,
        },
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
    use protocol::{Command, Entity, EntityKind, JobState, Material, MessageType, Speed, Tile};

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
    fn speed_keys_follow_the_pinned_step_table_and_clamp() {
        let dims = Dims { x: 1, y: 1, z: 1 };
        for (key, speed, expected) in [
            (
                KeyCode::Char(' '),
                Speed::Paused,
                Action::Command(Command::SetSpeed {
                    speed: Speed::Normal,
                }),
            ),
            (
                KeyCode::Char(' '),
                Speed::Normal,
                Action::Command(Command::SetSpeed {
                    speed: Speed::Paused,
                }),
            ),
            (
                KeyCode::Char(' '),
                Speed::Fast,
                Action::Command(Command::SetSpeed {
                    speed: Speed::Paused,
                }),
            ),
            (
                KeyCode::Char('+'),
                Speed::Paused,
                Action::Command(Command::SetSpeed {
                    speed: Speed::Normal,
                }),
            ),
            (
                KeyCode::Char('+'),
                Speed::Normal,
                Action::Command(Command::SetSpeed { speed: Speed::Fast }),
            ),
            (KeyCode::Char('+'), Speed::Fast, Action::Ignore),
            (
                KeyCode::Char('-'),
                Speed::Fast,
                Action::Command(Command::SetSpeed {
                    speed: Speed::Normal,
                }),
            ),
            (
                KeyCode::Char('-'),
                Speed::Normal,
                Action::Command(Command::SetSpeed {
                    speed: Speed::Paused,
                }),
            ),
            (KeyCode::Char('-'), Speed::Paused, Action::Ignore),
            (
                KeyCode::Char('S'),
                Speed::Paused,
                Action::Command(Command::Save),
            ),
            (
                KeyCode::Char('S'),
                Speed::Normal,
                Action::Command(Command::Save),
            ),
            (
                KeyCode::Char('S'),
                Speed::Fast,
                Action::Command(Command::Save),
            ),
            (
                KeyCode::Char('L'),
                Speed::Paused,
                Action::Command(Command::Load),
            ),
            (
                KeyCode::Char('L'),
                Speed::Normal,
                Action::Command(Command::Load),
            ),
            (
                KeyCode::Char('L'),
                Speed::Fast,
                Action::Command(Command::Load),
            ),
        ] {
            let mut state = ViewState {
                camera: (0, 0),
                z: 0,
                confirming_quit: false,
            };

            assert_eq!(
                apply_key(&mut state, press(key), dims, speed),
                expected,
                "wrong action for {key:?} at {speed:?}"
            );
        }
    }

    // A real terminal can only deliver an uppercase `S`/`L` with SHIFT held, so the table above
    // (which presses with `KeyModifiers::NONE`) does not exercise the path a user actually takes.
    // Without this, tightening the modifier gate in `apply_key` would leave every test green while
    // save and load stopped working in front of a human.
    #[test]
    fn save_and_load_keys_still_map_when_shift_is_held() {
        let dims = Dims { x: 1, y: 1, z: 1 };
        for (key, expected) in [
            (KeyCode::Char('S'), Action::Command(Command::Save)),
            (KeyCode::Char('L'), Action::Command(Command::Load)),
        ] {
            for speed in [Speed::Paused, Speed::Normal, Speed::Fast] {
                let mut state = ViewState {
                    camera: (0, 0),
                    z: 0,
                    confirming_quit: false,
                };

                assert_eq!(
                    apply_key(
                        &mut state,
                        KeyEvent::new(key, KeyModifiers::SHIFT),
                        dims,
                        speed
                    ),
                    expected,
                    "wrong action for SHIFT+{key:?} at {speed:?}"
                );
            }
        }
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
            ('t', (150, 160, 170)),
            ('i', (150, 160, 170)),
            ('c', (150, 160, 170)),
            ('k', (150, 160, 170)),
            (' ', (150, 160, 170)),
            ('0', (150, 160, 170)),
            (' ', (150, 160, 170)),
            ('d', (150, 160, 170)),
            (' ', (150, 160, 170)),
            ('d', (150, 160, 170)),
            ('i', (150, 160, 170)),
            ('g', (150, 160, 170)),
            (' ', (150, 160, 170)),
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
                fg: (150, 112, 62),
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

    #[test]
    fn walking_and_idle_dwarves_render_different_colors() {
        let dims = Dims { x: 3, y: 1, z: 1 };
        let mut snapshot = empty_snapshot(dims);
        snapshot.entities = vec![
            Entity {
                id: 1,
                kind: EntityKind::Dwarf,
                pos: [0, 0, 0],
                state: JobState::Idle,
            },
            Entity {
                id: 2,
                kind: EntityKind::Dwarf,
                pos: [2, 0, 0],
                state: JobState::Walk,
            },
        ];
        let state = ViewState {
            camera: (1, 0),
            z: 0,
            confirming_quit: false,
        };

        let framebuffer = render(&snapshot, &state, 3, 3);

        // The glyph is deliberately the same for both: what must differ is the colour, so
        // assert on `fg` alone. Comparing whole cells would also pass on a glyph change.
        let (idle, walking) = (framebuffer.cell(0, 0), framebuffer.cell(2, 0));
        assert_eq!(idle.glyph, walking.glyph);
        assert_ne!(idle.fg, walking.fg);
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

        let framebuffer = render(&snapshot, &state, 4, 3);

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
    fn status_line_reports_speed_z_and_dwarf_count() {
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

        let framebuffer = render(&snapshot, &state, 78, 3);
        let status: String = (0..78).map(|x| framebuffer.cell(x, 1).glyph).collect();

        assert_eq!(
            status,
            "tick 87  normal  z 19/31  dwarves 3                                           "
        );
    }

    #[test]
    fn status_and_hint_occupy_the_bottom_two_rows() {
        let snapshot = empty_snapshot(Dims { x: 3, y: 3, z: 1 });
        let state = ViewState {
            camera: (1, 1),
            z: 0,
            confirming_quit: false,
        };

        let framebuffer = render(&snapshot, &state, 80, 4);
        let status: String = (0..80).map(|x| framebuffer.cell(x, 2).glyph).collect();
        let hint: String = (0..80).map(|x| framebuffer.cell(x, 3).glyph).collect();

        assert!(status.starts_with("tick 0  normal  z 0/0  dwarves 0"));
        assert!(!status.contains("hjkl"));
        assert!(hint.contains("hjkl"));
        assert!(hint.contains("q quit client"));
    }

    #[test]
    fn status_line_fits_eighty_columns_without_truncation_at_large_ticks() {
        let dims = Dims {
            x: 40,
            y: 40,
            z: 32,
        };
        let state = ViewState {
            camera: (12, 34),
            z: 19,
            confirming_quit: false,
        };

        for (speed, wire_name) in [
            (Speed::Paused, "paused"),
            (Speed::Normal, "normal"),
            (Speed::Fast, "fast"),
        ] {
            let mut snapshot = empty_snapshot(dims);
            snapshot.tick = 9_999_999;
            snapshot.speed = speed;
            snapshot.entities = (0..5)
                .map(|id| Entity {
                    id,
                    kind: EntityKind::Dwarf,
                    pos: [1, 1, 30],
                    state: JobState::Idle,
                })
                .collect();
            let expected = format!("tick 9999999  {wire_name}  z 19/31  dwarves 5");

            let framebuffer = render(&snapshot, &state, 80, 3);
            let rendered_width = (0..80)
                .take_while(|x| framebuffer.cell(*x, 1).fg == STATUS_TEXT)
                .count();
            let expected_width = expected.chars().count();
            let rendered: String = (0..expected_width as u16)
                .map(|x| framebuffer.cell(x, 1).glyph)
                .collect();

            assert!(
                rendered_width <= 80,
                "{speed:?} status was {rendered_width} columns"
            );
            assert_eq!(rendered_width, expected_width);
            assert_eq!(rendered, expected);
            assert_eq!(framebuffer.cell((expected_width - 1) as u16, 1).glyph, '5');
        }
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
                apply_key(
                    &mut state,
                    KeyEvent::new(code, KeyModifiers::CONTROL),
                    dims,
                    Speed::Normal,
                ),
                Action::Ignore
            );
        }
        assert_eq!((state.camera, state.confirming_quit), ((1, 1), false));

        // `<` and `>` arrive with SHIFT held on most layouts.
        assert_eq!(
            apply_key(
                &mut state,
                KeyEvent::new(KeyCode::Char('>'), KeyModifiers::SHIFT),
                dims,
                Speed::Normal,
            ),
            Action::Redraw
        );
        assert_eq!(state.z, 1);

        assert_eq!(
            apply_key(
                &mut state,
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                dims,
                Speed::Normal,
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
            apply_key(&mut state, press(KeyCode::Char('>')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(state.z, 1);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Right), dims, Speed::Normal),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('l')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(state.camera.0, 2);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Down), dims, Speed::Normal),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('j')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(state.camera.1, 3);

        state.camera = (0, 0);
        state.z = 0;
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('<')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(state.z, 0);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Left), dims, Speed::Normal),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('h')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(state.camera.0, 0);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Up), dims, Speed::Normal),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('k')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(state.camera.1, 0);

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('>')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('l')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('j')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!((state.camera, state.z), ((1, 1), 1));

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('q')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert!(state.confirming_quit);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('y')), dims, Speed::Normal,),
            Action::Quit
        );

        state.confirming_quit = false;
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('q')), dims, Speed::Normal,),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Esc), dims, Speed::Normal),
            Action::Redraw
        );
        assert!(!state.confirming_quit);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('x')), dims, Speed::Normal,),
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

        let framebuffer = render(&snapshot, &state, 11, 3);
        let status: String = (0..11).map(|x| framebuffer.cell(x, 1).glyph).collect();

        assert_eq!(status, "quit? (y/n)");
    }
}
