//! The depth view: one ray per map cell, stepped through the voxel grid the client
//! already holds. No second camera, no second world, no second colour table.

use std::collections::BTreeMap;

use protocol::{EntityKind, JobState, Snapshot, Tile};

use crate::{
    palette::{BLANK, Cell, entity_cell, shade, tile_cell},
    view::{ViewState, tile_index},
};

/// 90 degrees horizontal.
const HALF_FOV_TAN: f64 = 1.0;
/// A terminal cell is about twice as tall as it is wide; without this the picture
/// is stretched vertically and a square corridor reads as a slot.
const CELL_ASPECT: f64 = 2.0;
// NOTE: the world is 128 across, so 96 steps reaches past anything worth seeing and
// makes a frame's cost bounded by the viewport rather than by the terrain.
const MAX_RAY_STEPS: u32 = 96;

/// Nearest to farthest. The GLYPH carries the distance because this devpod sets
/// `NO_COLOR`: a depth view whose only depth cue is a colour gradient produces a
/// well-formed capture that evidences nothing.
const BAND_GLYPHS: [char; 4] = ['█', '▓', '▒', '░'];
const BAND_LIMITS: [f64; 3] = [4.0, 10.0, 24.0];
const BAND_SHADE: [u16; 4] = [100, 80, 62, 46];
/// x, y and z faces. Without it every face of a cube is one flat colour and the
/// geometry disappears into a silhouette.
const FACE_SHADE: [u16; 3] = [100, 78, 60];

/// What a ray ran into, and where.
struct Hit {
    what: What,
    /// Euclidean, from the camera's eye point. Cheaper than perpendicular distance
    /// and the mild fisheye it leaves is a fair price at this resolution.
    distance: f64,
    /// 0 = x, 1 = y, 2 = z. The axis the ray crossed to enter the hit voxel.
    face: usize,
}

enum What {
    Terrain(Tile),
    Dwarf(JobState),
}

/// A ray's outcome.
struct Cast {
    hit: Option<Hit>,
    /// How many voxels the ray crossed. `draw` has no use for it; it is reported so
    /// that AC5's per-ray step bound is observable to a test, which is better than
    /// threading a counter through the draw path just to look at it.
    #[allow(dead_code)]
    steps: u32,
}

/// Fills the map region of `cells` in place. It owns no `Framebuffer`, so the two
/// views cannot drift apart on size, on the status rows, or on flushing.
pub fn draw(snapshot: &Snapshot, state: &ViewState, w: u16, map_h: u16, cells: &mut [Cell]) {
    if w == 0 || map_h == 0 {
        return;
    }
    let dwarves = dwarf_index(snapshot);

    // The eye sits at the centre of the camera's own voxel — the same `camera` and
    // `z` the flat view pans, never a second camera.
    let origin = (
        state.camera.0 as f64 + 0.5,
        state.camera.1 as f64 + 0.5,
        f64::from(state.z) + 0.5,
    );
    let (step_x, step_y) = heading_step(state.heading);
    let length = ((step_x * step_x + step_y * step_y) as f64).sqrt();
    let forward = (step_x as f64 / length, step_y as f64 / length);
    // Screen right is the heading turned 45 degrees clockwise twice: facing east, the
    // right of the picture is south, which is what makes `l` turn right on screen.
    let right = (-forward.1, forward.0);
    // The vertical half-angle follows from the horizontal one and the shape of the
    // viewport in real terms — cells are twice as tall as they are wide.
    let vertical = HALF_FOV_TAN * f64::from(map_h) * CELL_ASPECT / f64::from(w);

    for sy in 0..map_h {
        let v = (2.0 * (f64::from(sy) + 0.5) / f64::from(map_h) - 1.0) * vertical;
        for sx in 0..w {
            let u = (2.0 * (f64::from(sx) + 0.5) / f64::from(w) - 1.0) * HALF_FOV_TAN;
            let direction = (
                forward.0 + right.0 * u,
                forward.1 + right.1 * u,
                // Screen y grows downward, so a positive v looks down.
                -v,
            );
            let cast = cast(snapshot, &dwarves, origin, direction);
            cells[usize::from(sx) + usize::from(sy) * usize::from(w)] = match cast.hit {
                Some(hit) => {
                    let band = band_of(hit.distance);
                    let fg = match hit.what {
                        What::Terrain(tile) => tile_cell(tile).fg,
                        What::Dwarf(job) => entity_cell(EntityKind::Dwarf, job).fg,
                    };
                    Cell {
                        glyph: BAND_GLYPHS[band],
                        fg: shade(fg, BAND_SHADE[band] * FACE_SHADE[hit.face] / 100),
                    }
                }
                // Sky and "nothing drawn" are deliberately the same cell: the map area
                // is fully written every frame, so the guard against a silently blank
                // render is the capture's two-distinct-bands range check, not a sky
                // glyph that `NO_COLOR` would strip anyway.
                None => BLANK,
            };
        }
    }
}

/// Which tiles a ray must stop at. Derived from `snapshot.entities` alone — the wire
/// carries no occupancy flag and needs none.
// NOTE: the wire orders entities by ascending id, so `or_insert` gives a shared tile
// to the lowest id. A second `EntityKind` would have to decide its own rule, exactly
// as the flat view's contention rules do.
fn dwarf_index(snapshot: &Snapshot) -> BTreeMap<[i32; 3], JobState> {
    let mut index = BTreeMap::new();
    for entity in &snapshot.entities {
        if entity.kind == EntityKind::Dwarf {
            index.entry(entity.pos).or_insert(entity.state);
        }
    }
    index
}

fn band_of(distance: f64) -> usize {
    BAND_LIMITS
        .iter()
        .position(|limit| distance < *limit)
        .unwrap_or(BAND_GLYPHS.len() - 1)
}

/// Amanatides–Woo: step the voxel grid in x, y **and** z, stopping at the first
/// non-`Empty` tile, on leaving the world, or at the step cap.
fn cast(
    snapshot: &Snapshot,
    dwarves: &BTreeMap<[i32; 3], JobState>,
    origin: (f64, f64, f64),
    direction: (f64, f64, f64),
) -> Cast {
    let dims = snapshot.dims;
    let origin = [origin.0, origin.1, origin.2];
    let direction = [direction.0, direction.1, direction.2];
    let mut voxel = [
        origin[0].floor() as i64,
        origin[1].floor() as i64,
        origin[2].floor() as i64,
    ];

    let mut step = [0_i64; 3];
    // A zero component gives INFINITY, so that axis never advances rather than
    // dividing by zero.
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for axis in 0..3 {
        if direction[axis] > 0.0 {
            step[axis] = 1;
            t_max[axis] = ((voxel[axis] + 1) as f64 - origin[axis]) / direction[axis];
            t_delta[axis] = 1.0 / direction[axis];
        } else if direction[axis] < 0.0 {
            step[axis] = -1;
            t_max[axis] = (voxel[axis] as f64 - origin[axis]) / direction[axis];
            t_delta[axis] = -1.0 / direction[axis];
        }
    }

    let mut face = 0;
    let mut distance = 0.0;
    let mut steps = 0;
    loop {
        // Bounds-checked before `tiles` is indexed, never after.
        if voxel[0] < 0
            || voxel[1] < 0
            || voxel[2] < 0
            || voxel[0] >= i64::from(dims.x)
            || voxel[1] >= i64::from(dims.y)
            || voxel[2] >= i64::from(dims.z)
        {
            return Cast { hit: None, steps };
        }
        let position = [voxel[0] as i32, voxel[1] as i32, voxel[2] as i32];
        if let Some(job) = dwarves.get(&position) {
            return Cast {
                hit: Some(Hit {
                    what: What::Dwarf(*job),
                    distance,
                    face,
                }),
                steps,
            };
        }
        let tile =
            snapshot.tiles[tile_index(dims, voxel[0] as u32, voxel[1] as u32, voxel[2] as u32)];
        if tile != Tile::Empty {
            return Cast {
                hit: Some(Hit {
                    what: What::Terrain(tile),
                    distance,
                    face,
                }),
                steps,
            };
        }
        if steps >= MAX_RAY_STEPS {
            return Cast { hit: None, steps };
        }

        let axis = if t_max[0] < t_max[1] && t_max[0] < t_max[2] {
            0
        } else if t_max[1] < t_max[2] {
            1
        } else {
            2
        };
        distance = t_max[axis];
        voxel[axis] += step[axis];
        t_max[axis] += t_delta[axis];
        face = axis;
        steps += 1;
    }
}

/// The tile step for a heading. Index 0 is `+x` ("east"), then clockwise on screen —
/// screen `+y` is south, so the sequence is e, se, s, sw, w, nw, n, ne.
///
/// An integer table rather than a yaw angle: `ViewState` must stay `Copy + Eq`, and a
/// scripted `--key` capture must render byte-identically on every run.
pub fn heading_step(heading: u8) -> (i64, i64) {
    match heading % 8 {
        0 => (1, 0),
        1 => (1, 1),
        2 => (0, 1),
        3 => (-1, 1),
        4 => (-1, 0),
        5 => (-1, -1),
        6 => (0, -1),
        7 => (1, -1),
        _ => unreachable!("modulo 8"),
    }
}

/// The heading as the status line reports it.
pub fn heading_name(heading: u8) -> &'static str {
    match heading % 8 {
        0 => "e",
        1 => "se",
        2 => "s",
        3 => "sw",
        4 => "w",
        5 => "nw",
        6 => "n",
        7 => "ne",
        _ => unreachable!("modulo 8"),
    }
}

#[cfg(test)]
mod tests {
    use protocol::{Dims, Entity, EntityKind, JobState, Material, MessageType, Speed, Tile};

    use super::*;
    use crate::{
        palette::{BLANK, entity_cell, shade, tile_cell},
        view::{View, ViewState, tile_index},
    };

    // Odd on purpose: with an odd width and height the centre column and row are the
    // ray straight down the heading, exactly — no half-cell offset to reason around.
    const W: u16 = 11;
    const MAP_H: u16 = 7;

    /// A hand-built snapshot, never a `sim-core` fixture: a dev-dependency on
    /// `sim-core` trips the gate's `cargo tree -p tui` probe exactly like a normal one.
    fn world(dims: Dims) -> protocol::Snapshot {
        protocol::Snapshot {
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

    fn put(snapshot: &mut protocol::Snapshot, x: u32, y: u32, z: u32, tile: Tile) {
        let index = tile_index(snapshot.dims, x, y, z);
        snapshot.tiles[index] = tile;
    }

    fn depth_state(camera: (i64, i64), z: i32, heading: u8) -> ViewState {
        ViewState {
            camera,
            z,
            confirming_quit: false,
            mode: crate::view::Mode::Normal,
            cursor: camera,
            anchor: None,
            speed: Speed::Normal,
            view: View::Depth,
            heading,
        }
    }

    fn drawn(snapshot: &protocol::Snapshot, state: &ViewState) -> Vec<Cell> {
        let mut cells = vec![BLANK; usize::from(W) * usize::from(MAP_H)];
        draw(snapshot, state, W, MAP_H, &mut cells);
        cells
    }

    fn centre(cells: &[Cell]) -> Cell {
        cells[usize::from(MAP_H / 2) * usize::from(W) + usize::from(W / 2)]
    }

    fn centre_of(snapshot: &protocol::Snapshot, state: &ViewState) -> Cell {
        centre(&drawn(snapshot, state))
    }

    #[test]
    fn a_wall_lands_in_a_band_and_moves_to_a_nearer_one_as_it_approaches() {
        let dims = Dims { x: 32, y: 32, z: 4 };
        let mut far = world(dims);
        // Camera sits at (8.5, 8.5, 1.5) facing +x, so a wall whose near face is at
        // x = 17 stands 8.5 tiles off — the middle band.
        put(&mut far, 17, 8, 1, Tile::Solid(Material::Stone));
        let mut near = world(dims);
        put(&mut near, 12, 8, 1, Tile::Solid(Material::Stone));

        let state = depth_state((8, 8), 1, 0);

        // Hand-written glyphs, not BAND_GLYPHS[n]: the truth is stated independently of
        // the table under test, so retuning the table cannot quietly retune the test.
        assert_eq!(centre_of(&far, &state).glyph, '▓', "8.5 tiles is band 1");
        assert_eq!(centre_of(&near, &state).glyph, '█', "3.5 tiles is band 0");
    }

    #[test]
    fn every_heading_sees_the_wall_placed_in_its_own_direction_and_no_other() {
        let dims = Dims { x: 32, y: 32, z: 4 };
        for heading in 0..8u8 {
            let (dx, dy) = heading_step(heading);
            let mut snapshot = world(dims);
            put(
                &mut snapshot,
                (16 + dx * 5) as u32,
                (16 + dy * 5) as u32,
                1,
                Tile::Solid(Material::Stone),
            );

            for looking in 0..8u8 {
                let cell = centre_of(&snapshot, &depth_state((16, 16), 1, looking));
                if looking == heading {
                    assert_ne!(cell, BLANK, "heading {heading} cannot see its own wall");
                } else {
                    assert_eq!(
                        cell, BLANK,
                        "heading {looking} saw the wall placed for heading {heading}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_ray_that_leaves_the_world_draws_blank() {
        let dims = Dims { x: 8, y: 8, z: 2 };
        let snapshot = world(dims);

        // On the east edge facing east: the ray leaves after one step with nothing hit.
        let cell = centre_of(&snapshot, &depth_state((7, 4), 0, 0));

        assert_eq!(cell, BLANK);
    }

    #[test]
    fn a_dwarf_on_the_ray_is_drawn_instead_of_the_terrain_behind_it() {
        let dims = Dims { x: 32, y: 32, z: 4 };
        let mut snapshot = world(dims);
        put(&mut snapshot, 17, 8, 1, Tile::Solid(Material::Stone));
        let state = depth_state((8, 8), 1, 0);
        let terrain = centre_of(&snapshot, &state);

        snapshot.entities.push(Entity {
            id: 4,
            kind: EntityKind::Dwarf,
            pos: [12, 8, 1],
            state: JobState::Work,
        });
        let with_dwarf = centre_of(&snapshot, &state);

        // 3.5 tiles away through an x face: nearest band, no face darkening, so the
        // colour is the palette entry itself — asserted through `entity_cell`, never a
        // copied literal.
        assert_eq!(
            with_dwarf,
            Cell {
                glyph: '█',
                fg: entity_cell(EntityKind::Dwarf, JobState::Work).fg,
            }
        );
        assert_ne!(with_dwarf, terrain);

        // AC8's negative: the index must CHANGE what is drawn, not merely be built.
        snapshot.entities.clear();
        assert_eq!(
            centre_of(&snapshot, &state),
            terrain,
            "removing the entity must give the cell back to terrain"
        );
    }

    #[test]
    fn a_wall_turned_empty_reveals_the_wall_behind_it() {
        let dims = Dims { x: 32, y: 32, z: 4 };
        let mut snapshot = world(dims);
        put(&mut snapshot, 12, 8, 1, Tile::Solid(Material::Ice));
        put(&mut snapshot, 17, 8, 1, Tile::Solid(Material::Stone));
        let state = depth_state((8, 8), 1, 0);

        let near = centre_of(&snapshot, &state);
        assert_eq!(near.glyph, '█');
        assert_eq!(near.fg, tile_cell(Tile::Solid(Material::Ice)).fg);

        // Exactly what a delta does to this client's tiles.
        put(&mut snapshot, 12, 8, 1, Tile::Empty);
        let behind = centre_of(&snapshot, &state);

        assert_eq!(behind.glyph, '▓', "the far wall sits in a farther band");
        assert_eq!(
            behind.fg,
            shade(tile_cell(Tile::Solid(Material::Stone)).fg, 80)
        );
    }

    #[test]
    fn the_hit_colour_is_the_palette_entry_shaded_by_band_and_face() {
        let dims = Dims { x: 32, y: 32, z: 4 };
        let mut snapshot = world(dims);
        put(&mut snapshot, 17, 8, 1, Tile::Solid(Material::Snow));
        let state = depth_state((8, 8), 1, 0);

        // Band 1 (8.5 tiles), x face (no face darkening): 80% of the palette entry.
        assert_eq!(
            centre_of(&snapshot, &state).fg,
            shade(tile_cell(Tile::Solid(Material::Snow)).fg, 80)
        );
    }

    #[test]
    fn a_downward_ray_is_darkened_by_the_face_it_crosses_as_well_as_the_band() {
        let dims = Dims { x: 32, y: 32, z: 4 };
        let mut snapshot = world(dims);
        // A floor directly beneath the camera and nothing else: the bottom-centre ray
        // looks down and crosses the floor's TOP face less than a tile away.
        for x in 0..dims.x {
            for y in 0..dims.y {
                put(&mut snapshot, x, y, 0, Tile::Solid(Material::Soil));
            }
        }
        let state = depth_state((8, 8), 1, 0);

        let cells = drawn(&snapshot, &state);
        let floor = cells[usize::from(MAP_H - 1) * usize::from(W) + usize::from(W / 2)];
        let palette = tile_cell(Tile::Solid(Material::Soil)).fg;

        assert_eq!(
            floor.glyph, '█',
            "the floor underfoot is in the nearest band"
        );
        assert_eq!(
            floor.fg,
            shade(palette, 60),
            "nearest band through a z face"
        );
        assert_ne!(
            floor.fg,
            shade(palette, 100),
            "the crossed face must darken the colour, not only the distance"
        );
    }

    #[test]
    fn cast_reports_the_face_it_crossed_on_each_axis() {
        let dims = Dims { x: 8, y: 8, z: 8 };
        let mut snapshot = world(dims);
        put(&mut snapshot, 6, 4, 4, Tile::Solid(Material::Stone));
        put(&mut snapshot, 4, 6, 4, Tile::Solid(Material::Stone));
        put(&mut snapshot, 4, 4, 6, Tile::Solid(Material::Stone));
        let dwarves = BTreeMap::new();
        let origin = (4.5, 4.5, 4.5);

        for (direction, face) in [
            ((1.0, 0.0, 0.0), 0),
            ((0.0, 1.0, 0.0), 1),
            ((0.0, 0.0, 1.0), 2),
        ] {
            let cast = cast(&snapshot, &dwarves, origin, direction);
            assert_eq!(
                cast.hit.expect("the ray must reach its wall").face,
                face,
                "direction {direction:?}"
            );
        }
    }

    #[test]
    fn a_ray_into_open_air_stops_at_the_step_cap_rather_than_the_world_edge() {
        // Wider than the cap can reach, so the cap is what ends the ray. Without it the
        // cost of a frame would follow the terrain rather than the viewport.
        let dims = Dims {
            x: 256,
            y: 256,
            z: 3,
        };
        let snapshot = world(dims);

        let cast = cast(
            &snapshot,
            &BTreeMap::new(),
            (128.5, 128.5, 1.5),
            (1.0, 0.0, 0.0),
        );

        assert!(cast.hit.is_none());
        assert_eq!(cast.steps, MAX_RAY_STEPS);
        assert!(
            u64::from(MAX_RAY_STEPS) < u64::from(dims.x) / 2,
            "the cap must bind before the world edge or this proves nothing"
        );
    }

    #[test]
    fn a_ray_stopped_by_terrain_reports_fewer_steps_than_the_cap() {
        let dims = Dims {
            x: 256,
            y: 256,
            z: 3,
        };
        let mut snapshot = world(dims);
        put(&mut snapshot, 138, 128, 1, Tile::Solid(Material::Stone));

        let cast = cast(
            &snapshot,
            &BTreeMap::new(),
            (128.5, 128.5, 1.5),
            (1.0, 0.0, 0.0),
        );

        assert!(cast.hit.is_some());
        assert!(cast.steps < MAX_RAY_STEPS, "steps were {}", cast.steps);
    }

    #[test]
    fn the_dwarf_index_gives_a_shared_tile_to_the_lowest_id() {
        let dims = Dims { x: 4, y: 4, z: 2 };
        let mut snapshot = world(dims);
        snapshot.entities = vec![
            Entity {
                id: 3,
                kind: EntityKind::Dwarf,
                pos: [1, 1, 0],
                state: JobState::Work,
            },
            Entity {
                id: 7,
                kind: EntityKind::Dwarf,
                pos: [1, 1, 0],
                state: JobState::Idle,
            },
        ];

        let index = dwarf_index(&snapshot);

        assert_eq!(index.len(), 1);
        assert_eq!(index.get(&[1, 1, 0]), Some(&JobState::Work));
    }

    #[test]
    fn the_heading_table_is_pinned_clockwise_from_east() {
        // Hand-written truth, not derived from the function under test: index 0 faces
        // +x and each step turns 45 degrees clockwise on screen (+y is south).
        let expected = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];
        for (heading, step) in expected.iter().enumerate() {
            let heading = heading as u8;
            assert_eq!(heading_step(heading), *step, "step for heading {heading}");
        }
    }

    #[test]
    fn every_step_is_a_distinct_unit_move() {
        let mut steps = std::collections::BTreeSet::new();
        for heading in 0..8 {
            let (dx, dy) = heading_step(heading);
            assert!((-1..=1).contains(&dx) && (-1..=1).contains(&dy));
            assert!((dx, dy) != (0, 0), "heading {heading} does not move");
            assert!(steps.insert((dx, dy)), "heading {heading} repeats a step");
        }
        assert_eq!(steps.len(), 8);
    }

    #[test]
    fn opposite_headings_cancel() {
        for heading in 0..8u8 {
            let (dx, dy) = heading_step(heading);
            let (bx, by) = heading_step((heading + 4) % 8);
            assert_eq!((dx + bx, dy + by), (0, 0), "heading {heading}");
        }
    }
}
