use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use protocol::{Command, DesignationKind, Dims, EntityKind, Rect, Snapshot, Speed, Tile};

use crate::palette::{
    BLANK, Cell, PEEK_DEPTH, STATUS_TEXT, carrier_cell, crowd_cell, cursor_cell, designation_cell,
    dim, entity_cell, item_cell, pending_rect_cell, tile_cell, zone_cell,
};

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
    pub mode: Mode,
    pub cursor: (i64, i64),
    pub anchor: Option<(i64, i64)>,
    pub speed: Speed,
    pub view: View,
    /// 0..8, 45 degrees apart, 0 = `+x`. An integer, not an angle: `ViewState` must
    /// stay `Copy + Eq`, and a scripted capture must render identically every run.
    pub heading: u8,
}

/// Which way the same camera is looked through. Both views share `camera` and `z`,
/// so a move made in either is reflected in the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Flat,
    Depth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Dig,
    Channel,
    Stockpile,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Redraw,
    Quit,
    Command(Command),
    // NOTE: two is the only multi-command arity needed: `x` clears both mark kinds.
    Commands([Command; 2]),
    Ignore,
}

/// The opening view. Deterministic: the same world opens on the same frame on
/// every run, so a scripted capture aims where its author thought it did.
///
/// `z_override` is `--z`. It is clamped because it is operator input.
pub fn initial(snapshot: &Snapshot, z_override: Option<i32>) -> ViewState {
    // NOTE: clamped because the dims are wire data.
    let max_x = i64::from(snapshot.dims.x.saturating_sub(1));
    let max_y = i64::from(snapshot.dims.y.saturating_sub(1));
    let max_z = i32::try_from(snapshot.dims.z.saturating_sub(1)).unwrap_or(i32::MAX);
    let camera = (
        i64::from(snapshot.dims.x / 2).clamp(0, max_x),
        i64::from(snapshot.dims.y / 2).clamp(0, max_y),
    );
    ViewState {
        camera,
        z: z_override
            .unwrap_or_else(|| opening_z(snapshot))
            .clamp(0, max_z),
        confirming_quit: false,
        mode: Mode::Normal,
        cursor: camera,
        anchor: None,
        speed: snapshot.speed,
        view: View::Flat,
        heading: 0,
    }
}

/// The z level with the most standable ground — an `Empty` tile with `Solid` or
/// `Ramp` directly beneath it. Ties resolve to the lowest such z; a world with no
/// standable ground at all opens on the middle level.
///
/// NOTE: this reads the same wire tiles the renderer already reads and asks
/// `sim-core` nothing — it is a presentation heuristic for "which level is worth
/// looking at", not a sim rule, and it is not required to agree with
/// `World::is_standable` if that rule ever changes.
///
/// It replaced opening on `entities.first()`, which took the whole opening view
/// from dwarf 0 and therefore moved as he wandered: two clients connecting
/// minutes apart opened on different levels, and every scripted `--key` capture
/// aimed at a different z depending on when it ran. That cost a false "the
/// feature does not work" verdict at story 3.3's review, with exit 0. The
/// trade-off accepted here is that the opening frame no longer centres on a
/// dwarf; if that proves annoying in play the answer is a centre-on-dwarf key,
/// never a nondeterministic opening.
fn opening_z(snapshot: &Snapshot) -> i32 {
    let Dims { x, y, z } = snapshot.dims;
    let middle = i32::try_from(z / 2).unwrap_or(i32::MAX);
    let mut best: Option<(u64, u32)> = None;
    // NOTE: from 1 — level 0 is the world floor and has nothing beneath it to
    // stand on, so it can never be standable.
    for level in 1..z {
        let mut standable = 0u64;
        for ty in 0..y {
            for tx in 0..x {
                let here = snapshot.tiles.get(tile_index(snapshot.dims, tx, ty, level));
                let below = snapshot
                    .tiles
                    .get(tile_index(snapshot.dims, tx, ty, level - 1));
                if matches!(here, Some(Tile::Empty))
                    && matches!(below, Some(Tile::Solid(_) | Tile::Ramp(_)))
                {
                    standable += 1;
                }
            }
        }
        // NOTE: strictly greater, so a tie keeps the LOWEST z. That is what makes
        // the choice reproducible rather than dependent on iteration order.
        if standable > 0 && standable > best.map_or(0, |(count, _)| count) {
            best = Some((standable, level));
        }
    }
    best.map_or(middle, |(_, level)| i32::try_from(level).unwrap_or(middle))
}

pub fn render(snapshot: &Snapshot, state: &ViewState, w: u16, h: u16) -> Framebuffer {
    let mut framebuffer = Framebuffer {
        w,
        h,
        cells: vec![BLANK; usize::from(w) * usize::from(h)],
    };
    // NOTE: deliberate — below two rows there is no map to draw, so a 1-row terminal renders
    // blank rather than a lone status line (which is what it did before the status/hint split).
    // A terminal this small cannot show the game; pinned by `one_row_terminal_renders_blank` so
    // the behaviour is a decision rather than an accident of `map_h = h - 2`.
    if w == 0 || h < 2 {
        return framebuffer;
    }

    let map_h = h - 2;
    // The dispatch lives HERE and not in `main.rs` on purpose: three call sites render,
    // and dispatching at them would let the instrument render a different path from the
    // player's — which is exactly how story 2.2 shipped an instrument that showed motion
    // as stillness. The status and hint rows stay below, shared by both views.
    match state.view {
        View::Flat => draw_flat(snapshot, state, w, map_h, &mut framebuffer.cells),
        View::Depth => crate::raycast::draw(snapshot, state, w, map_h, &mut framebuffer.cells),
    }

    draw_status_and_hint(snapshot, state, w, h, &mut framebuffer.cells);
    framebuffer
}

/// The flat view's map region: terrain with its peek-through, then zones,
/// designations, items, dwarves, the pending rectangle and the cursor.
fn draw_flat(snapshot: &Snapshot, state: &ViewState, w: u16, map_h: u16, cells: &mut [Cell]) {
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
            cells[usize::from(sx) + usize::from(sy) * usize::from(w)] = cell;
        }
    }

    let screen_index = |pos: [i32; 3]| {
        if pos[2] != state.z
            || pos[0] < 0
            || pos[1] < 0
            || i64::from(pos[0]) >= i64::from(snapshot.dims.x)
            || i64::from(pos[1]) >= i64::from(snapshot.dims.y)
        {
            return None;
        }
        let sx = i64::from(pos[0]) - state.camera.0 + i64::from(w) / 2;
        let sy = i64::from(pos[1]) - state.camera.1 + i64::from(map_h) / 2;
        (sx >= 0 && sx < i64::from(w) && sy >= 0 && sy < i64::from(map_h))
            .then(|| sx as usize + sy as usize * usize::from(w))
    };

    for zone in &snapshot.zones {
        if let Some(index) = screen_index(zone.pos) {
            cells[index] = zone_cell();
        }
    }

    for designation in &snapshot.designations {
        if let Some(index) = screen_index(designation.pos) {
            cells[index] = designation_cell(designation.kind);
        }
    }

    // Counted in the same pass that draws them, so the count and the draw can never disagree
    // about which stones are on screen, on this level, and where.
    // NOTE: two or more stones on one tile render as a single `*` with no count in the glyph —
    // there is no item stacking model. Deliberate: the sim enforces one stone per STOCKPILE tile,
    // so a pile always reads truthfully; a heap on open ground does not.
    let mut item_counts = BTreeMap::new();
    for item in &snapshot.items {
        if let Some(index) = screen_index(item.pos) {
            cells[index] = item_cell();
            *item_counts.entry(index).or_insert(0_usize) += 1;
        }
    }

    // Same filter for counting and drawing dwarves, for the same reason. A second `EntityKind`
    // would need its own contention rule; today there is only one.
    let mut dwarf_counts = BTreeMap::new();
    for entity in &snapshot.entities {
        if entity.kind == EntityKind::Dwarf
            && let Some(index) = screen_index(entity.pos)
        {
            *dwarf_counts.entry(index).or_insert(0_usize) += 1;
        }
    }
    for entity in &snapshot.entities {
        if entity.kind == EntityKind::Dwarf
            && let Some(index) = screen_index(entity.pos)
        {
            cells[index] = if dwarf_counts.get(&index).copied().unwrap_or(0) > 1 {
                crowd_cell()
            } else if item_counts.get(&index).copied().unwrap_or(0) > 0 {
                carrier_cell()
            } else {
                entity_cell(entity.kind, entity.state)
            };
        }
    }

    if let Some(anchor) = state.anchor {
        let min_x = anchor.0.min(state.cursor.0);
        let max_x = anchor.0.max(state.cursor.0);
        let min_y = anchor.1.min(state.cursor.1);
        let max_y = anchor.1.max(state.cursor.1);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let (Ok(x), Ok(y)) = (i32::try_from(x), i32::try_from(y)) else {
                    continue;
                };
                if let Some(index) = screen_index([x, y, state.z]) {
                    cells[index] = pending_rect_cell(state.mode);
                }
            }
        }
    }

    if state.mode != Mode::Normal
        && let (Ok(x), Ok(y)) = (i32::try_from(state.cursor.0), i32::try_from(state.cursor.1))
        && let Some(index) = screen_index([x, y, state.z])
    {
        cells[index] = cursor_cell();
    }
}

/// The bottom two rows, drawn once for whichever view filled the map above them.
fn draw_status_and_hint(
    snapshot: &Snapshot,
    state: &ViewState,
    w: u16,
    h: u16,
    cells: &mut [Cell],
) {
    let status = if state.confirming_quit {
        "quit? (y/n)".to_string()
    } else {
        let dwarves = snapshot
            .entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Dwarf)
            .count();
        let speed = match state.speed {
            Speed::Paused => "paused",
            Speed::Normal => "normal",
            Speed::Fast => "fast",
        };
        // The `  3d <heading>  ` token is what a scripted capture greps for, so it is
        // pinned by an exact-string test rather than left to formatting drift.
        match state.view {
            View::Flat => format!(
                "tick {}  {}  z {}/{}  dwarves {}",
                snapshot.tick,
                speed,
                state.z,
                snapshot.dims.z.saturating_sub(1),
                dwarves
            ),
            View::Depth => format!(
                "tick {}  {}  3d {}  z {}/{}  dwarves {}",
                snapshot.tick,
                speed,
                crate::raycast::heading_name(state.heading),
                state.z,
                snapshot.dims.z.saturating_sub(1),
                dwarves
            ),
        }
    };
    let status_y = h - 2;
    for (x, glyph) in (0..w).zip(status.chars()) {
        cells[usize::from(x) + usize::from(status_y) * usize::from(w)] = Cell {
            glyph,
            fg: STATUS_TEXT,
        };
    }

    let hint = hint(state);
    let hint_y = h - 1;
    for (x, glyph) in (0..w).zip(hint.chars()) {
        cells[usize::from(x) + usize::from(hint_y) * usize::from(w)] = Cell {
            glyph,
            fg: STATUS_TEXT,
        };
    }
}

fn hint(state: &ViewState) -> &'static str {
    // Prefixed `depth:` in the style of the mode hints, and deliberately NOT `3d:` —
    // that would collide with a capture's grep for the status line's `3d` token.
    if state.view == View::Depth {
        return "depth: hl turn  kj walk  <> z  v flat view  q quit client";
    }
    match (state.mode, state.anchor.is_some()) {
        (Mode::Normal, _) => {
            "d dig  c channel  p stockpile  x clear  <> z  hjkl move  q quit client"
        }
        (Mode::Dig, false) => "dig: hjkl move  <> z  Enter anchor  Esc normal  q quit",
        (Mode::Dig, true) => "dig: hjkl move  Enter commit  Esc unanchor  q quit",
        (Mode::Channel, false) => "channel: hjkl move  <> z  Enter anchor  Esc normal  q quit",
        (Mode::Channel, true) => "channel: hjkl move  Enter commit  Esc unanchor  q quit",
        (Mode::Stockpile, false) => "stockpile: hjkl move  <> z  Enter anchor  Esc normal  q quit",
        (Mode::Stockpile, true) => "stockpile: hjkl move  Enter commit  Esc unanchor  q quit",
        (Mode::Remove, false) => {
            "clear marks + stockpiles: hjkl  <>z  Enter anchor  Esc normal  q quit"
        }
        (Mode::Remove, true) => {
            "clear marks + stockpiles: hjkl  Enter commit  Esc unanchor  q quit"
        }
    }
}

pub fn apply_key(state: &mut ViewState, key: KeyEvent, dims: Dims, viewport: (u16, u16)) -> Action {
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

    let speed = state.speed;
    let command = |state: &mut ViewState, speed| {
        state.speed = speed;
        Action::Command(Command::SetSpeed { speed })
    };
    match key.code {
        KeyCode::Char('S') => Action::Command(Command::Save),
        KeyCode::Char('L') => Action::Command(Command::Load),
        KeyCode::Char(' ') => command(
            state,
            match speed {
                Speed::Paused => Speed::Normal,
                Speed::Normal | Speed::Fast => Speed::Paused,
            },
        ),
        KeyCode::Char('+') => match speed {
            Speed::Paused => command(state, Speed::Normal),
            Speed::Normal => command(state, Speed::Fast),
            Speed::Fast => Action::Ignore,
        },
        KeyCode::Char('-') => match speed {
            Speed::Fast => command(state, Speed::Normal),
            Speed::Normal => command(state, Speed::Paused),
            Speed::Paused => Action::Ignore,
        },
        // Client state only: no command goes upstream, and the guard pair mirrors the
        // designation keys below for the same reason — a half-placed rectangle must
        // never be silently lost to a view toggle.
        KeyCode::Char('v') if state.mode == Mode::Normal => {
            state.view = match state.view {
                View::Flat => View::Depth,
                View::Depth => View::Flat,
            };
            Action::Redraw
        }
        KeyCode::Char('v') => Action::Ignore,
        KeyCode::Char(key @ ('d' | 'c' | 'p' | 'x'))
            if state.mode == Mode::Normal && state.view == View::Flat =>
        {
            state.mode = match key {
                'd' => Mode::Dig,
                'c' => Mode::Channel,
                'p' => Mode::Stockpile,
                'x' => Mode::Remove,
                _ => unreachable!("guard limits mode keys"),
            };
            state.cursor = state.camera;
            state.anchor = None;
            Action::Redraw
        }
        KeyCode::Char('d' | 'c' | 'p' | 'x') => Action::Ignore,
        KeyCode::Enter if state.mode != Mode::Normal => match state.anchor.take() {
            None => {
                state.anchor = Some(state.cursor);
                Action::Redraw
            }
            Some(anchor) => {
                let rect = Rect {
                    min: [
                        anchor.0.min(state.cursor.0) as i32,
                        anchor.1.min(state.cursor.1) as i32,
                        state.z,
                    ],
                    max: [
                        anchor.0.max(state.cursor.0) as i32,
                        anchor.1.max(state.cursor.1) as i32,
                        state.z,
                    ],
                };
                match state.mode {
                    Mode::Dig => Action::Command(Command::Designate {
                        kind: DesignationKind::Dig,
                        rect,
                    }),
                    Mode::Channel => Action::Command(Command::Designate {
                        kind: DesignationKind::Channel,
                        rect,
                    }),
                    Mode::Stockpile => Action::Command(Command::PlaceStockpile { rect }),
                    Mode::Remove => Action::Commands([
                        Command::CancelDesignation { rect },
                        Command::RemoveStockpile { rect },
                    ]),
                    Mode::Normal => Action::Ignore,
                }
            }
        },
        KeyCode::Esc if state.anchor.is_some() => {
            state.anchor = None;
            Action::Redraw
        }
        KeyCode::Esc if state.mode != Mode::Normal => {
            state.mode = Mode::Normal;
            Action::Redraw
        }
        KeyCode::Char('<' | '>') if state.anchor.is_some() => Action::Ignore,
        KeyCode::Char('<') => {
            state.z = (state.z - 1).max(0);
            Action::Redraw
        }
        KeyCode::Char('>') => {
            state.z = (state.z + 1).min(dims.z.saturating_sub(1) as i32);
            Action::Redraw
        }
        // The depth view routes FIRST: in it these four keys turn and walk the camera
        // rather than panning it, and there is no cursor to move because designation
        // input stays a flat-view capability.
        KeyCode::Left | KeyCode::Char('h') => {
            if state.view == View::Depth {
                state.heading = (state.heading + 7) % 8;
            } else if state.mode == Mode::Normal {
                state.camera.0 = (state.camera.0 - 1).max(0);
            } else {
                move_cursor(state, -1, 0, dims, viewport);
            }
            Action::Redraw
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if state.view == View::Depth {
                state.heading = (state.heading + 1) % 8;
            } else if state.mode == Mode::Normal {
                state.camera.0 = (state.camera.0 + 1).min(i64::from(dims.x.saturating_sub(1)));
            } else {
                move_cursor(state, 1, 0, dims, viewport);
            }
            Action::Redraw
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if state.view == View::Depth {
                step_camera(state, 1, dims);
            } else if state.mode == Mode::Normal {
                state.camera.1 = (state.camera.1 - 1).max(0);
            } else {
                move_cursor(state, 0, -1, dims, viewport);
            }
            Action::Redraw
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.view == View::Depth {
                step_camera(state, -1, dims);
            } else if state.mode == Mode::Normal {
                state.camera.1 = (state.camera.1 + 1).min(i64::from(dims.y.saturating_sub(1)));
            } else {
                move_cursor(state, 0, 1, dims, viewport);
            }
            Action::Redraw
        }
        KeyCode::Char('q') => {
            state.confirming_quit = true;
            Action::Redraw
        }
        _ => Action::Ignore,
    }
}

/// One tile along the current heading, `sign` -1 for backwards. Clamped to the world
/// exactly as the flat view's pan is, so the camera can never leave the world.
fn step_camera(state: &mut ViewState, sign: i64, dims: Dims) {
    let (dx, dy) = crate::raycast::heading_step(state.heading);
    state.camera.0 = (state.camera.0 + dx * sign).clamp(0, i64::from(dims.x.saturating_sub(1)));
    state.camera.1 = (state.camera.1 + dy * sign).clamp(0, i64::from(dims.y.saturating_sub(1)));
}

fn move_cursor(state: &mut ViewState, dx: i64, dy: i64, dims: Dims, viewport: (u16, u16)) {
    let max_x = i64::from(dims.x.saturating_sub(1));
    let max_y = i64::from(dims.y.saturating_sub(1));
    state.cursor.0 = (state.cursor.0 + dx).clamp(0, max_x);
    state.cursor.1 = (state.cursor.1 + dy).clamp(0, max_y);

    let (w, h) = viewport;
    let map_h = h.saturating_sub(2);
    if w == 0 || map_h == 0 {
        state.camera = state.cursor;
        return;
    }

    let sx = state.cursor.0 - state.camera.0 + i64::from(w) / 2;
    if sx < 0 {
        state.camera.0 += sx;
    } else if sx >= i64::from(w) {
        state.camera.0 += sx - (i64::from(w) - 1);
    }
    let sy = state.cursor.1 - state.camera.1 + i64::from(map_h) / 2;
    if sy < 0 {
        state.camera.1 += sy;
    } else if sy >= i64::from(map_h) {
        state.camera.1 += sy - (i64::from(map_h) - 1);
    }
    state.camera.0 = state.camera.0.clamp(0, max_x);
    state.camera.1 = state.camera.1.clamp(0, max_y);
}

// NOTE: widened before multiplying — the strides come from the wire, and a u32
// product would overflow before the caller's bounds check ever sees it.
pub fn tile_index(dims: Dims, x: u32, y: u32, z: u32) -> usize {
    x as usize + y as usize * dims.x as usize + z as usize * dims.x as usize * dims.y as usize
}

#[cfg(test)]
mod tests {
    use protocol::{
        Command, Designation, DesignationKind, Entity, EntityKind, Item, JobState, Material,
        MessageType, Speed, Tile, Zone,
    };

    use super::*;

    fn empty_snapshot(dims: Dims) -> Snapshot {
        Snapshot {
            msg_type: MessageType::Snapshot,
            dims,
            tiles: vec![Tile::Empty; (dims.x * dims.y * dims.z) as usize],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
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

    fn normal_state(camera: (i64, i64), z: i32) -> ViewState {
        ViewState {
            camera,
            z,
            confirming_quit: false,
            mode: Mode::Normal,
            cursor: camera,
            anchor: None,
            speed: Speed::Normal,
            view: View::Flat,
            heading: 0,
        }
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
            let mut state = normal_state((0, 0), 0);
            state.speed = speed;

            assert_eq!(
                apply_key(&mut state, press(key), dims, (80, 24)),
                expected,
                "wrong action for {key:?} at {speed:?}"
            );
        }
    }

    #[test]
    fn optimistic_speed_keys_compose_before_a_wire_update() {
        let dims = Dims { x: 1, y: 1, z: 1 };
        let mut state = normal_state((0, 0), 0);

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('+')), dims, (80, 24)),
            Action::Command(Command::SetSpeed { speed: Speed::Fast })
        );
        assert_eq!(state.speed, Speed::Fast);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('-')), dims, (80, 24)),
            Action::Command(Command::SetSpeed {
                speed: Speed::Normal,
            })
        );
        assert_eq!(state.speed, Speed::Normal);
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
                let mut state = normal_state((0, 0), 0);
                state.speed = speed;

                assert_eq!(
                    apply_key(
                        &mut state,
                        KeyEvent::new(key, KeyModifiers::SHIFT),
                        dims,
                        (80, 24)
                    ),
                    expected,
                    "wrong action for SHIFT+{key:?} at {speed:?}"
                );
            }
        }
    }

    #[test]
    fn one_row_terminal_renders_blank() {
        // Pins the `h < 2` early return as a decision, not an accident of `map_h = h - 2`.
        // Before the status/hint split a 1-row terminal drew a status line; it now draws
        // nothing, because two rows are reserved and no map row is left. Asserted at h = 1
        // and h = 0, with h = 2 as the control proving the guard is not simply always-blank.
        let dims = Dims { x: 3, y: 3, z: 1 };
        let snapshot = empty_snapshot(dims);
        let state = normal_state((1, 1), 0);
        for h in [0, 1] {
            let framebuffer = render(&snapshot, &state, 10, h);
            assert!(
                framebuffer.cells.iter().all(|cell| *cell == BLANK),
                "h={h} should render blank"
            );
        }
        let framebuffer = render(&snapshot, &state, 10, 2);
        assert!(
            framebuffer.cells.iter().any(|cell| *cell != BLANK),
            "h=2 must still draw the status and hint rows"
        );
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
        let state = normal_state((2, 1), 2);

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
        let state = normal_state((2, 1), 1);

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

    /// Repointed at story 3.3: a dwarf on a stone used to hide it behind a plain `☺`, which is
    /// exactly the state the haul loop has to be able to show.
    #[test]
    fn items_draw_only_on_the_viewed_level_and_a_shared_cell_draws_the_carrier() {
        let dims = Dims { x: 5, y: 3, z: 3 };
        let mut snapshot = empty_snapshot(dims);
        snapshot.items = vec![
            Item {
                id: 5,
                pos: [1, 1, 1],
            },
            Item {
                id: 6,
                pos: [3, 1, 2],
            },
        ];
        snapshot.entities = vec![Entity {
            id: 1,
            kind: EntityKind::Dwarf,
            pos: [1, 1, 1],
            state: JobState::Idle,
        }];

        let framebuffer = render(&snapshot, &normal_state((2, 1), 1), 5, 4);

        assert_eq!(framebuffer.cell(1, 1), carrier_cell());
        assert_eq!(framebuffer.cell(3, 1), BLANK);
        assert_eq!(
            framebuffer
                .cells
                .iter()
                .filter(|cell| cell.glyph == '☺' || cell.glyph == '*')
                .count(),
            0,
            "a shared cell must draw neither the plain dwarf nor the stone"
        );

        // The stone one level up must not count towards the dwarf's cell: the count has to use
        // the same z filter the draw does.
        snapshot.items[0].pos = [1, 1, 2];
        let framebuffer = render(&snapshot, &normal_state((2, 1), 1), 5, 4);
        assert_eq!(framebuffer.cell(1, 1).glyph, '☺');

        snapshot.items[0].pos = [1, 1, 1];
        snapshot.entities.clear();
        let framebuffer = render(&snapshot, &normal_state((2, 1), 1), 5, 4);
        assert_eq!(framebuffer.cell(1, 1).glyph, '*');
    }

    #[test]
    fn offscreen_items_are_discarded_before_indexing_the_framebuffer() {
        let dims = Dims {
            x: 128,
            y: 128,
            z: 1,
        };
        let mut snapshot = empty_snapshot(dims);
        snapshot.items = vec![Item {
            id: 5,
            pos: [0, 0, 0],
        }];
        snapshot.entities = vec![Entity {
            id: 1,
            kind: EntityKind::Dwarf,
            pos: [127, 127, 0],
            state: JobState::Idle,
        }];

        let framebuffer = render(&snapshot, &normal_state((127, 127), 0), 5, 4);

        assert!(framebuffer.cells.iter().all(|cell| *cell != item_cell()));
        assert!(
            framebuffer.cells.iter().all(|cell| *cell != carrier_cell()),
            "an off-screen stone was counted against an on-screen dwarf"
        );
        assert!(framebuffer.cells.iter().any(|cell| cell.glyph == '☺'));
    }

    #[test]
    fn two_dwarves_on_one_cell_draw_the_crowd_glyph() {
        let dims = Dims { x: 3, y: 3, z: 1 };
        let mut snapshot = empty_snapshot(dims);
        // A stone under them too: the crowd glyph wins over the carrier glyph, or two dwarves
        // sharing a stockpile tile would read as one carrier.
        snapshot.items = vec![Item {
            id: 5,
            pos: [1, 1, 0],
        }];
        snapshot.entities = vec![
            Entity {
                id: 1,
                kind: EntityKind::Dwarf,
                pos: [1, 1, 0],
                state: JobState::Idle,
            },
            Entity {
                id: 2,
                kind: EntityKind::Dwarf,
                pos: [1, 1, 0],
                state: JobState::Walk,
            },
        ];

        let framebuffer = render(&snapshot, &normal_state((1, 1), 0), 3, 4);

        assert_eq!(framebuffer.cell(1, 1).glyph, '⚇');
        assert_eq!(
            framebuffer
                .cells
                .iter()
                .filter(|cell| cell.glyph == '☺' || cell.glyph == '☻' || cell.glyph == '*')
                .count(),
            0
        );
    }

    #[test]
    fn marks_draw_only_on_the_viewed_level() {
        let dims = Dims { x: 5, y: 5, z: 2 };
        let mut snapshot = empty_snapshot(dims);
        snapshot.designations = vec![
            Designation {
                pos: [1, 2, 1],
                kind: DesignationKind::Dig,
            },
            Designation {
                pos: [2, 2, 0],
                kind: DesignationKind::Channel,
            },
        ];
        snapshot.zones = vec![Zone { pos: [3, 2, 1] }, Zone { pos: [2, 2, 0] }];

        let framebuffer = render(&snapshot, &normal_state((2, 2), 1), 5, 5);

        assert_eq!(framebuffer.cell(1, 1).glyph, '×');
        assert_eq!(framebuffer.cell(3, 1).glyph, '≡');
        assert_eq!(framebuffer.cell(2, 1), BLANK);
    }

    #[test]
    fn marker_layers_follow_terrain_zone_designation_item_entity_pending_cursor_order() {
        let dims = Dims { x: 7, y: 3, z: 1 };
        let mut snapshot = empty_snapshot(dims);
        for x in 0..=5 {
            snapshot.tiles[index(dims, x, 1, 0)] = Tile::Solid(Material::Stone);
        }
        snapshot.zones = vec![Zone { pos: [0, 1, 0] }, Zone { pos: [1, 1, 0] }];
        snapshot.designations = vec![
            Designation {
                pos: [1, 1, 0],
                kind: DesignationKind::Dig,
            },
            Designation {
                pos: [2, 1, 0],
                kind: DesignationKind::Channel,
            },
        ];
        snapshot.items = vec![
            Item {
                id: 5,
                pos: [2, 1, 0],
            },
            Item {
                id: 6,
                pos: [3, 1, 0],
            },
        ];
        snapshot.entities = vec![
            Entity {
                id: 1,
                kind: EntityKind::Dwarf,
                pos: [3, 1, 0],
                state: JobState::Idle,
            },
            Entity {
                id: 2,
                kind: EntityKind::Dwarf,
                pos: [4, 1, 0],
                state: JobState::Idle,
            },
        ];
        let state = ViewState {
            mode: Mode::Dig,
            cursor: (5, 1),
            anchor: Some((4, 1)),
            ..normal_state((3, 1), 0)
        };

        let framebuffer = render(&snapshot, &state, 7, 5);

        assert_eq!(framebuffer.cell(0, 1).glyph, '≡');
        assert_eq!(framebuffer.cell(1, 1).glyph, '×');
        assert_eq!(framebuffer.cell(2, 1).glyph, '*');
        // The entity layer still wins over the item layer; with a stone under it, the dwarf's
        // own look is the carrier glyph.
        assert_eq!(framebuffer.cell(3, 1).glyph, '☻');
        assert_eq!(framebuffer.cell(4, 1).glyph, 'd');
        assert_eq!(framebuffer.cell(5, 1).glyph, '+');
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
        let state = normal_state((1, 0), 0);

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
        let state = normal_state((1, 0), 7);

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
        let state = normal_state((12, 34), 19);

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
        let state = normal_state((1, 1), 0);

        let framebuffer = render(&snapshot, &state, 80, 4);
        let status: String = (0..80).map(|x| framebuffer.cell(x, 2).glyph).collect();
        let hint: String = (0..80).map(|x| framebuffer.cell(x, 3).glyph).collect();

        assert!(status.starts_with("tick 0  normal  z 0/0  dwarves 0"));
        assert!(!status.contains("hjkl"));
        assert!(hint.contains("hjkl"));
        assert!(hint.contains("q quit client"));
    }

    #[test]
    fn hint_bar_names_every_modes_keys_and_fits_eighty_columns() {
        let snapshot = empty_snapshot(Dims { x: 3, y: 3, z: 1 });
        for mode in [
            Mode::Normal,
            Mode::Dig,
            Mode::Channel,
            Mode::Stockpile,
            Mode::Remove,
        ] {
            for anchored in [false, true] {
                let mut state = normal_state((1, 1), 0);
                state.mode = mode;
                state.anchor = anchored.then_some((1, 1));
                // Rendered WIDER than the budget on purpose. Reading 80 columns out of an
                // 80-wide framebuffer and asserting `<= 80` cannot fail — an over-long hint
                // would be silently truncated into a pass. At 120 the full hint survives, so
                // the width assertion below is the real guard AC10 asks for.
                let framebuffer = render(&snapshot, &state, 120, 3);
                let hint: String = (0..120)
                    .map(|x| framebuffer.cell(x, 2).glyph)
                    .collect::<String>()
                    .trim_end()
                    .to_string();

                assert!(hint.chars().count() <= 80, "{mode:?}: {hint:?}");
                assert!(hint.contains("hjkl"), "{mode:?}: {hint:?}");
                match mode {
                    Mode::Normal => {
                        for key in ["d dig", "c channel", "p stockpile", "x clear"] {
                            assert!(hint.contains(key), "normal hint missed {key:?}: {hint:?}");
                        }
                        assert!(hint.contains("q quit client"));
                    }
                    Mode::Dig => assert!(hint.starts_with("dig:")),
                    Mode::Channel => assert!(hint.starts_with("channel:")),
                    Mode::Stockpile => assert!(hint.starts_with("stockpile:")),
                    Mode::Remove => assert!(hint.contains("clear marks + stockpiles")),
                }
                if mode != Mode::Normal {
                    assert!(hint.contains(if anchored {
                        "Enter commit"
                    } else {
                        "Enter anchor"
                    }));
                    assert!(hint.contains(if anchored {
                        "Esc unanchor"
                    } else {
                        "Esc normal"
                    }));
                }
            }
        }

        // The depth view's own hint, measured the same way and at the same width.
        let state = ViewState {
            view: View::Depth,
            ..normal_state((1, 1), 0)
        };
        let framebuffer = render(&snapshot, &state, 120, 3);
        let hint: String = (0..120)
            .map(|x| framebuffer.cell(x, 2).glyph)
            .collect::<String>()
            .trim_end()
            .to_string();

        assert!(hint.chars().count() <= 80, "depth hint: {hint:?}");
        assert!(hint.starts_with("depth:"), "{hint:?}");
        for named in ["turn", "walk", "<> z", "v flat", "q quit client"] {
            assert!(
                hint.contains(named),
                "depth hint missed {named:?}: {hint:?}"
            );
        }
        // Designation input is not a depth-view capability, so the bar must not offer it.
        for absent in ["dig", "channel", "stockpile", "clear"] {
            assert!(
                !hint.contains(absent),
                "depth hint advertises {absent:?}: {hint:?}"
            );
        }
    }

    #[test]
    fn the_two_views_draw_different_pictures_of_the_same_world() {
        // The guard on the dispatch itself: without it `render` could quietly draw the
        // flat map in both views while the status line below happily reported `3d`.
        let dims = Dims { x: 32, y: 32, z: 4 };
        let mut snapshot = empty_snapshot(dims);
        for x in 0..dims.x {
            for y in 0..dims.y {
                snapshot.tiles[index(dims, x, y, 0)] = Tile::Solid(Material::Stone);
            }
        }
        snapshot.tiles[index(dims, 24, 16, 1)] = Tile::Solid(Material::Ice);
        let flat = normal_state((16, 16), 1);
        let depth = ViewState {
            view: View::Depth,
            ..flat
        };

        let map_of = |state: &ViewState| -> Vec<Cell> {
            let framebuffer = render(&snapshot, state, 21, 11);
            (0..9)
                .flat_map(|y| (0..21).map(move |x| (x, y)))
                .map(|(x, y)| framebuffer.cell(x, y))
                .collect()
        };

        assert_ne!(
            map_of(&flat),
            map_of(&depth),
            "both views drew the same picture: the dispatch is not reaching the raycaster"
        );
    }

    #[test]
    fn the_depth_view_fills_the_map_region_and_shares_the_bottom_two_rows() {
        let dims = Dims { x: 32, y: 32, z: 4 };
        let mut snapshot = empty_snapshot(dims);
        for x in 0..dims.x {
            for y in 0..dims.y {
                snapshot.tiles[index(dims, x, y, 0)] = Tile::Solid(Material::Stone);
            }
        }
        let state = ViewState {
            view: View::Depth,
            ..normal_state((16, 16), 1)
        };

        let framebuffer = render(&snapshot, &state, 11, 5);

        // One framebuffer, one size, one flush: the depth view writes the map region of
        // the same buffer the flat view does and touches nothing below it.
        let map: Vec<Cell> = (0..3)
            .flat_map(|y| (0..11).map(move |x| (x, y)))
            .map(|(x, y)| framebuffer.cell(x, y))
            .collect();
        assert!(
            map.iter().any(|cell| *cell != BLANK),
            "the depth view drew nothing at all"
        );
        for x in 0..11 {
            assert_eq!(framebuffer.cell(x, 3).fg, STATUS_TEXT, "status row");
        }
        assert_eq!(framebuffer.cell(0, 4).glyph, 'd', "hint row still below it");
    }

    #[test]
    fn the_depth_status_line_reports_the_view_and_the_heading() {
        let dims = Dims {
            x: 40,
            y: 40,
            z: 32,
        };
        let mut snapshot = empty_snapshot(dims);
        snapshot.tick = 20;
        snapshot.entities = (0..5)
            .map(|id| Entity {
                id,
                kind: EntityKind::Dwarf,
                pos: [1, 1, 30],
                state: JobState::Idle,
            })
            .collect();

        for (heading, name) in [(0, "e"), (1, "se"), (4, "w"), (7, "ne")] {
            let state = ViewState {
                view: View::Depth,
                heading,
                ..normal_state((12, 34), 17)
            };
            // The exact shape a capture greps for, written out rather than composed
            // from the code under test.
            let expected = format!("tick 20  normal  3d {name}  z 17/31  dwarves 5");

            let framebuffer = render(&snapshot, &state, 80, 3);
            let rendered: String = (0..expected.chars().count() as u16)
                .map(|x| framebuffer.cell(x, 1).glyph)
                .collect();

            assert_eq!(rendered, expected);
        }

        // The flat view keeps the line it already had — no `3d` token anywhere in it.
        let framebuffer = render(&snapshot, &normal_state((12, 34), 17), 80, 3);
        let flat: String = (0..80).map(|x| framebuffer.cell(x, 1).glyph).collect();
        assert!(flat.trim_end().ends_with("dwarves 5"));
        assert!(!flat.contains("3d"), "{flat:?}");
    }

    #[test]
    fn status_line_fits_eighty_columns_without_truncation_at_large_ticks() {
        let dims = Dims {
            x: 40,
            y: 40,
            z: 32,
        };
        // The depth view carries the widest line — the extra `3d <heading>` token and
        // the longest heading name — so it is the one that decides the budget.
        for (speed, wire_name, view) in [
            (Speed::Paused, "paused", View::Flat),
            (Speed::Normal, "normal", View::Flat),
            (Speed::Fast, "fast", View::Flat),
            (Speed::Paused, "paused", View::Depth),
            (Speed::Normal, "normal", View::Depth),
            (Speed::Fast, "fast", View::Depth),
        ] {
            let mut state = normal_state((12, 34), 19);
            state.speed = speed;
            state.view = view;
            // 3 = `sw`, one of the two-letter headings.
            state.heading = 3;
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
            let expected = match view {
                View::Flat => format!("tick 9999999  {wire_name}  z 19/31  dwarves 5"),
                View::Depth => format!("tick 9999999  {wire_name}  3d sw  z 19/31  dwarves 5"),
            };

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
        let mut state = normal_state((1, 1), 0);

        for code in [KeyCode::Char('l'), KeyCode::Char('j'), KeyCode::Char('q')] {
            assert_eq!(
                apply_key(
                    &mut state,
                    KeyEvent::new(code, KeyModifiers::CONTROL),
                    dims,
                    (80, 24),
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
                (80, 24),
            ),
            Action::Redraw
        );
        assert_eq!(state.z, 1);

        assert_eq!(
            apply_key(
                &mut state,
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                dims,
                (80, 24),
            ),
            Action::Quit
        );
    }

    #[test]
    fn mode_keys_enter_only_from_normal_and_escape_backs_out_one_level() {
        let dims = Dims { x: 20, y: 20, z: 3 };
        for (key, mode) in [
            ('d', Mode::Dig),
            ('c', Mode::Channel),
            ('p', Mode::Stockpile),
            ('x', Mode::Remove),
        ] {
            let mut state = normal_state((7, 9), 1);
            state.cursor = (0, 0);

            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char(key)), dims, (9, 7)),
                Action::Redraw
            );
            assert_eq!(state.mode, mode);
            assert_eq!(state.cursor, (7, 9));
            assert_eq!(state.anchor, None);

            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char('d')), dims, (9, 7)),
                Action::Ignore
            );
            assert_eq!(state.mode, mode);

            state.anchor = Some(state.cursor);
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Esc), dims, (9, 7)),
                Action::Redraw
            );
            assert_eq!(state.mode, mode);
            assert_eq!(state.anchor, None);
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Esc), dims, (9, 7)),
                Action::Redraw
            );
            assert_eq!(state.mode, Mode::Normal);
        }
    }

    #[test]
    fn speed_save_load_and_quit_keys_remain_global_in_every_mode() {
        let dims = Dims { x: 20, y: 20, z: 3 };
        for mode in [Mode::Dig, Mode::Channel, Mode::Stockpile, Mode::Remove] {
            let mut state = normal_state((7, 9), 1);
            state.mode = mode;
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char('S')), dims, (9, 7)),
                Action::Command(Command::Save)
            );
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char('L')), dims, (9, 7)),
                Action::Command(Command::Load)
            );
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char('+')), dims, (9, 7)),
                Action::Command(Command::SetSpeed { speed: Speed::Fast })
            );
            assert_eq!(state.speed, Speed::Fast);
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char('q')), dims, (9, 7)),
                Action::Redraw
            );
            assert!(state.confirming_quit);
        }
    }

    #[test]
    fn z_keys_work_unanchored_and_are_ignored_while_anchored() {
        let dims = Dims { x: 20, y: 20, z: 3 };
        let mut state = normal_state((7, 9), 1);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('d')), dims, (9, 7)),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('>')), dims, (9, 7)),
            Action::Redraw
        );
        assert_eq!(state.z, 2);

        state.anchor = Some(state.cursor);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('<')), dims, (9, 7)),
            Action::Ignore
        );
        assert_eq!(state.z, 2);
    }

    #[test]
    fn cursor_moves_clamps_and_pans_camera_only_after_crossing_the_window_edge() {
        let dims = Dims { x: 20, y: 20, z: 3 };
        let viewport = (5, 5);
        let mut state = normal_state((5, 5), 1);
        let _ = apply_key(&mut state, press(KeyCode::Char('d')), dims, viewport);

        let _ = apply_key(&mut state, press(KeyCode::Char('l')), dims, viewport);
        let _ = apply_key(&mut state, press(KeyCode::Right), dims, viewport);
        assert_eq!(state.cursor, (7, 5));
        assert_eq!(state.camera, (5, 5));

        let _ = apply_key(&mut state, press(KeyCode::Char('l')), dims, viewport);
        assert_eq!(state.cursor, (8, 5));
        assert_eq!(state.camera, (6, 5));

        let _ = apply_key(&mut state, press(KeyCode::Char('j')), dims, viewport);
        assert_eq!(state.cursor, (8, 6));
        assert_eq!(state.camera, (6, 5));
        let _ = apply_key(&mut state, press(KeyCode::Down), dims, viewport);
        assert_eq!(state.cursor, (8, 7));
        assert_eq!(state.camera, (6, 6));

        for _ in 0..30 {
            let _ = apply_key(&mut state, press(KeyCode::Char('l')), dims, viewport);
            let _ = apply_key(&mut state, press(KeyCode::Char('j')), dims, viewport);
        }
        assert_eq!(state.cursor, (19, 19));
        assert_eq!(state.camera, (17, 18));
    }

    #[test]
    fn second_enter_commits_each_single_command_mode_and_stays_in_mode() {
        let dims = Dims { x: 20, y: 20, z: 3 };
        let rect = Rect {
            min: [2, 3, 1],
            max: [3, 4, 1],
        };
        for (key, mode, expected) in [
            (
                'd',
                Mode::Dig,
                Command::Designate {
                    kind: DesignationKind::Dig,
                    rect,
                },
            ),
            (
                'c',
                Mode::Channel,
                Command::Designate {
                    kind: DesignationKind::Channel,
                    rect,
                },
            ),
            ('p', Mode::Stockpile, Command::PlaceStockpile { rect }),
        ] {
            let mut state = normal_state((2, 3), 1);
            let _ = apply_key(&mut state, press(KeyCode::Char(key)), dims, (9, 7));
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Enter), dims, (9, 7)),
                Action::Redraw
            );
            assert_eq!(state.anchor, Some((2, 3)));
            let _ = apply_key(&mut state, press(KeyCode::Char('l')), dims, (9, 7));
            let _ = apply_key(&mut state, press(KeyCode::Char('j')), dims, (9, 7));

            assert_eq!(
                apply_key(&mut state, press(KeyCode::Enter), dims, (9, 7)),
                Action::Command(expected)
            );
            assert_eq!(state.mode, mode);
            assert_eq!(state.anchor, None);
        }
    }

    #[test]
    fn remove_mode_commits_cancel_then_remove_stockpile_for_the_same_rect() {
        let dims = Dims { x: 20, y: 20, z: 3 };
        let mut state = normal_state((2, 3), 1);
        let _ = apply_key(&mut state, press(KeyCode::Char('x')), dims, (9, 7));
        let _ = apply_key(&mut state, press(KeyCode::Enter), dims, (9, 7));
        let _ = apply_key(&mut state, press(KeyCode::Char('l')), dims, (9, 7));
        let rect = Rect {
            min: [2, 3, 1],
            max: [3, 3, 1],
        };

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Enter), dims, (9, 7)),
            Action::Commands([
                Command::CancelDesignation { rect },
                Command::RemoveStockpile { rect },
            ])
        );
        assert_eq!(state.mode, Mode::Remove);
        assert_eq!(state.anchor, None);
    }

    #[test]
    fn out_of_world_cells_are_blank() {
        let dims = Dims { x: 3, y: 2, z: 1 };
        let mut snapshot = empty_snapshot(dims);
        snapshot.tiles[index(dims, 0, 0, 0)] = Tile::Solid(Material::Stone);
        let state = normal_state((0, 0), 0);

        let framebuffer = render(&snapshot, &state, 5, 4);

        assert_eq!(framebuffer.cell(2, 1).glyph, '█');
        for (x, y) in [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1)] {
            assert_eq!(framebuffer.cell(x, y), BLANK);
        }
    }

    #[test]
    fn keys_move_and_clamp() {
        let dims = Dims { x: 3, y: 4, z: 2 };
        let mut state = normal_state((2, 3), 1);

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('>')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(state.z, 1);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Right), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('l')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(state.camera.0, 2);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Down), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('j')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(state.camera.1, 3);

        state.camera = (0, 0);
        state.z = 0;
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('<')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(state.z, 0);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Left), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('h')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(state.camera.0, 0);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Up), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('k')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(state.camera.1, 0);

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('>')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('l')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('j')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!((state.camera, state.z), ((1, 1), 1));

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('q')), dims, (80, 24),),
            Action::Redraw
        );
        assert!(state.confirming_quit);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('y')), dims, (80, 24),),
            Action::Quit
        );

        state.confirming_quit = false;
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('q')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Esc), dims, (80, 24),),
            Action::Redraw
        );
        assert!(!state.confirming_quit);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('x')), dims, (80, 24),),
            Action::Redraw
        );
        assert_eq!(state.mode, Mode::Remove);
    }

    /// Fills level `floor` solid and leaves `floor + 1` empty, so `floor + 1`
    /// has `count` standable cells.
    fn make_standable(snapshot: &mut Snapshot, floor: u32, count: u32) {
        let dims = snapshot.dims;
        for n in 0..count {
            let (x, y) = (n % dims.x, n / dims.x);
            snapshot.tiles[index(dims, x, y, floor)] = Tile::Solid(Material::Stone);
            snapshot.tiles[index(dims, x, y, floor + 1)] = Tile::Empty;
        }
    }

    #[test]
    fn initial_view_opens_on_the_level_with_the_most_standable_ground() {
        let dims = Dims { x: 9, y: 7, z: 6 };
        let mut snapshot = empty_snapshot(dims);
        make_standable(&mut snapshot, 1, 3); // z 2 -> 3 standable
        make_standable(&mut snapshot, 3, 9); // z 4 -> 9 standable

        assert_eq!(
            initial(&snapshot, None),
            ViewState {
                camera: (4, 3),
                z: 4,
                confirming_quit: false,
                mode: Mode::Normal,
                cursor: (4, 3),
                anchor: None,
                speed: Speed::Normal,
                view: View::Flat,
                heading: 0,
            }
        );
    }

    #[test]
    fn initial_view_does_not_move_when_the_first_entity_does() {
        // The defect this replaced: the whole opening view came from dwarf 0, so
        // it moved as he wandered and no scripted capture was reproducible.
        let dims = Dims { x: 9, y: 7, z: 6 };
        let mut snapshot = empty_snapshot(dims);
        make_standable(&mut snapshot, 3, 9);
        let before = initial(&snapshot, None);

        snapshot.entities.push(Entity {
            id: 7,
            kind: EntityKind::Dwarf,
            pos: [8, 1, 5],
            state: JobState::Idle,
        });
        assert_eq!(initial(&snapshot, None), before);

        snapshot.entities[0].pos = [0, 6, 2];
        assert_eq!(initial(&snapshot, None), before);
    }

    #[test]
    fn initial_view_breaks_a_standable_tie_towards_the_lowest_level() {
        let dims = Dims { x: 9, y: 7, z: 6 };
        let mut snapshot = empty_snapshot(dims);
        make_standable(&mut snapshot, 1, 4);
        make_standable(&mut snapshot, 3, 4);

        assert_eq!(initial(&snapshot, None).z, 2);
    }

    #[test]
    fn initial_view_falls_back_to_the_middle_level_without_standable_ground() {
        let snapshot = empty_snapshot(Dims { x: 9, y: 7, z: 5 });
        assert_eq!(initial(&snapshot, None).z, 2);
    }

    #[test]
    fn initial_view_honours_a_clamped_z_override() {
        let dims = Dims { x: 9, y: 7, z: 6 };
        let mut snapshot = empty_snapshot(dims);
        make_standable(&mut snapshot, 3, 9);

        assert_eq!(initial(&snapshot, Some(1)).z, 1);
        // Operator input, so it is clamped rather than trusted.
        assert_eq!(initial(&snapshot, Some(99)).z, 5);
        assert_eq!(initial(&snapshot, Some(-4)).z, 0);
    }

    #[test]
    fn confirming_quit_replaces_the_status_line() {
        let snapshot = empty_snapshot(Dims { x: 1, y: 1, z: 1 });
        let state = ViewState {
            confirming_quit: true,
            ..normal_state((0, 0), 0)
        };

        let framebuffer = render(&snapshot, &state, 11, 3);
        let status: String = (0..11).map(|x| framebuffer.cell(x, 1).glyph).collect();

        assert_eq!(status, "quit? (y/n)");
    }

    #[test]
    fn v_toggles_the_depth_view_from_normal_and_back() {
        let dims = Dims { x: 8, y: 8, z: 4 };
        let mut state = normal_state((4, 4), 1);
        assert_eq!(state.view, View::Flat);

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('v')), dims, (80, 24)),
            Action::Redraw
        );
        assert_eq!(state.view, View::Depth);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('v')), dims, (80, 24)),
            Action::Redraw
        );
        assert_eq!(state.view, View::Flat);
    }

    #[test]
    fn v_is_ignored_in_every_designation_mode_and_leaves_the_anchor_alone() {
        let dims = Dims { x: 8, y: 8, z: 4 };
        for mode in [Mode::Dig, Mode::Channel, Mode::Stockpile, Mode::Remove] {
            for anchor in [None, Some((2, 3))] {
                let mut state = ViewState {
                    mode,
                    anchor,
                    ..normal_state((4, 4), 1)
                };

                assert_eq!(
                    apply_key(&mut state, press(KeyCode::Char('v')), dims, (80, 24)),
                    Action::Ignore,
                    "{mode:?} with anchor {anchor:?}"
                );
                assert_eq!(state.view, View::Flat, "{mode:?}");
                assert_eq!(state.mode, mode);
                assert_eq!(state.anchor, anchor, "a half-placed rect must survive `v`");
            }
        }
    }

    #[test]
    fn turning_walks_the_eight_headings_and_wraps_both_ways() {
        let dims = Dims { x: 8, y: 8, z: 4 };
        let mut state = ViewState {
            view: View::Depth,
            ..normal_state((4, 4), 1)
        };

        for expected in [1, 2, 3, 4, 5, 6, 7, 0] {
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char('l')), dims, (80, 24)),
                Action::Redraw
            );
            assert_eq!(state.heading, expected, "right turn");
        }
        for expected in [7, 6, 5, 4, 3, 2, 1, 0] {
            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char('h')), dims, (80, 24)),
                Action::Redraw
            );
            assert_eq!(state.heading, expected, "left turn");
        }

        // The arrow keys are the same two turns, not a second mechanism.
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Right), dims, (80, 24)),
            Action::Redraw
        );
        assert_eq!(state.heading, 1);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Left), dims, (80, 24)),
            Action::Redraw
        );
        assert_eq!(state.heading, 0);

        // Turning never moves the camera and never leaves the depth view.
        assert_eq!(state.camera, (4, 4));
        assert_eq!(state.view, View::Depth);
    }

    #[test]
    fn forward_then_back_returns_to_the_starting_tile_on_every_heading() {
        let dims = Dims { x: 16, y: 16, z: 4 };
        for heading in 0..8 {
            let start = (8, 8);
            let mut state = ViewState {
                view: View::Depth,
                heading,
                ..normal_state(start, 1)
            };

            apply_key(&mut state, press(KeyCode::Char('k')), dims, (80, 24));
            let moved = state.camera;
            let (dx, dy) = crate::raycast::heading_step(heading);
            assert_eq!(
                moved,
                (start.0 + dx, start.1 + dy),
                "heading {heading} did not step by its own table entry"
            );

            apply_key(&mut state, press(KeyCode::Char('j')), dims, (80, 24));
            assert_eq!(state.camera, start, "heading {heading} did not come back");
        }
    }

    #[test]
    fn forward_clamps_at_the_world_edge_rather_than_leaving_the_world() {
        let dims = Dims { x: 4, y: 4, z: 2 };
        // heading 1 = south-east, so both axes press against the far corner.
        let mut state = ViewState {
            view: View::Depth,
            heading: 1,
            ..normal_state((3, 3), 0)
        };

        for _ in 0..3 {
            apply_key(&mut state, press(KeyCode::Char('k')), dims, (80, 24));
        }
        assert_eq!(state.camera, (3, 3));

        // heading 5 = north-west, the mirror against the origin corner.
        let mut state = ViewState {
            view: View::Depth,
            heading: 5,
            ..normal_state((0, 0), 0)
        };
        for _ in 0..3 {
            apply_key(&mut state, press(KeyCode::Char('k')), dims, (80, 24));
        }
        assert_eq!(state.camera, (0, 0));
    }

    #[test]
    fn designation_keys_do_nothing_in_the_depth_view() {
        let dims = Dims { x: 8, y: 8, z: 4 };
        for key in ['d', 'c', 'p', 'x'] {
            let mut state = ViewState {
                view: View::Depth,
                ..normal_state((4, 4), 1)
            };

            assert_eq!(
                apply_key(&mut state, press(KeyCode::Char(key)), dims, (80, 24)),
                Action::Ignore,
                "{key} reached the depth view"
            );
            assert_eq!(state.mode, Mode::Normal, "{key} changed mode in depth");
            assert_eq!(state.anchor, None);
        }
    }

    #[test]
    fn global_keys_keep_working_in_the_depth_view() {
        let dims = Dims { x: 8, y: 8, z: 4 };
        let mut state = ViewState {
            view: View::Depth,
            ..normal_state((4, 4), 1)
        };

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char(' ')), dims, (80, 24)),
            Action::Command(Command::SetSpeed {
                speed: Speed::Paused
            })
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('S')), dims, (80, 24)),
            Action::Command(Command::Save)
        );
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('L')), dims, (80, 24)),
            Action::Command(Command::Load)
        );

        // z still moves, and still clamps, exactly as it does in the flat view.
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('>')), dims, (80, 24)),
            Action::Redraw
        );
        assert_eq!(state.z, 2);
        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('<')), dims, (80, 24)),
            Action::Redraw
        );
        assert_eq!(state.z, 1);

        assert_eq!(
            apply_key(&mut state, press(KeyCode::Char('q')), dims, (80, 24)),
            Action::Redraw
        );
        assert!(state.confirming_quit);
        assert_eq!(state.view, View::Depth, "quitting must not leave the view");
    }

    #[test]
    fn the_opening_view_is_flat_and_faces_east() {
        let snapshot = empty_snapshot(Dims { x: 4, y: 4, z: 2 });

        let state = initial(&snapshot, None);

        assert_eq!(state.view, View::Flat);
        assert_eq!(state.heading, 0);
    }
}
