use bevy::{
    camera::Camera,
    prelude::{Camera3d, GlobalTransform, Query, Res, ResMut, Resource, Vec3, Window, With},
    window::PrimaryWindow,
};

use crate::{
    ingest::MirrorResource,
    project::{is_tree_foliage, is_visible_at_slice},
    slice::SliceLevel,
    transform::{render_to_world, world_to_render},
};

/// The client-local tile currently under the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Top,
    Bottom,
    East,
    West,
    North,
    South,
}

impl Face {
    pub fn normal(self) -> Vec3 {
        match self {
            Self::Top => Vec3::Y,
            Self::Bottom => -Vec3::Y,
            Self::East => Vec3::X,
            Self::West => -Vec3::X,
            Self::North => Vec3::Z,
            Self::South => -Vec3::Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickedCell {
    pub tile: [i32; 3],
    pub face: Face,
}

/// The client-local cell currently under the cursor, plus the face its ray entered.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickedTile(pub Option<PickedCell>);

impl PickedTile {
    pub fn tile(&self) -> Option<[i32; 3]> {
        self.0.map(|cell| cell.tile)
    }
}

/// Resolves the primary window's cursor through the rendering camera into the visible terrain.
pub fn update_pick(
    mut picked: ResMut<PickedTile>,
    mirror: Res<MirrorResource>,
    slice: Res<SliceLevel>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok((camera, global)) = cameras.single() else {
        picked.0 = None;
        return;
    };
    let Ok(window) = windows.single() else {
        picked.0 = None;
        return;
    };
    picked.0 = window.cursor_position().and_then(|cursor| {
        camera
            .viewport_to_world(global, cursor)
            .ok()
            .and_then(|ray| {
                first_visible_hit(
                    ray.origin,
                    ray.direction.as_vec3(),
                    &mirror.0,
                    slice.level(),
                )
            })
    });
}

fn first_visible_hit(
    origin: Vec3,
    direction: Vec3,
    mirror: &client_core::Mirror,
    level: i32,
) -> Option<PickedCell> {
    let dims = mirror.dims();
    // AC2: `world_to_render` is the ONLY axis conversion. The two opposite world corners are
    // projected through it and the cell half-extent added afterwards, so a change to the y/z
    // swap or the z negation cannot leave a second, hand-rolled copy of the same knowledge here.
    let near_corner = world_to_render([0, 0, 0]);
    let far_corner = world_to_render([dims.x as i32 - 1, dims.y as i32 - 1, dims.z as i32 - 1]);
    let min = near_corner.min(far_corner) - Vec3::splat(0.5);
    let max = near_corner.max(far_corner) + Vec3::splat(0.5);
    let (entry, exit) = ray_box_interval(origin, direction, min, max)?;
    let diagonal = (max - min).length();
    let mut distance = entry.max(0.0);
    let end = (distance + diagonal).min(exit);
    if distance > end {
        return None;
    }

    // NOTE: no boundary nudge. `f32::EPSILON` is one ULP at magnitude 1.0 and vanishes at the
    // entry distances the camera's 4.0..=500.0 clamp produces (`distance + EPSILON` was measured
    // bit-identical to `distance` at 2, 4, 10, 41, 90, 100, 183.8 and 500), so the nudge that
    // stood here was dead code claiming a protection it did not provide. The box-face entry
    // point already floors into the cell the ray enters.
    let point = origin + direction * distance;
    let mut cell = (point + Vec3::splat(0.5)).floor().as_ivec3();
    let step = direction.signum().as_ivec3();
    let next_boundary = Vec3::new(
        if direction.x >= 0.0 {
            cell.x as f32 + 0.5
        } else {
            cell.x as f32 - 0.5
        },
        if direction.y >= 0.0 {
            cell.y as f32 + 0.5
        } else {
            cell.y as f32 - 0.5
        },
        if direction.z >= 0.0 {
            cell.z as f32 + 0.5
        } else {
            cell.z as f32 - 0.5
        },
    );
    let mut next = Vec3::new(
        ray_axis_distance(origin.x, direction.x, next_boundary.x),
        ray_axis_distance(origin.y, direction.y, next_boundary.y),
        ray_axis_distance(origin.z, direction.z, next_boundary.z),
    );
    let delta = Vec3::new(
        ray_step_distance(direction.x),
        ray_step_distance(direction.y),
        ray_step_distance(direction.z),
    );

    let mut face = entry_face(origin, direction, min, max, entry);
    while distance <= end {
        let centre = cell.as_vec3();
        let world = render_to_world(centre);
        if mirror.tile(world).is_some()
            && is_visible_at_slice(mirror, world, level)
            && !is_tree_foliage(mirror, world)
        {
            return Some(PickedCell { tile: world, face });
        }
        if next.x <= next.y && next.x <= next.z {
            distance = next.x;
            next.x += delta.x;
            cell.x += step.x;
            face = if step.x >= 0 { Face::West } else { Face::East };
        } else if next.y <= next.z {
            distance = next.y;
            next.y += delta.y;
            cell.y += step.y;
            face = if step.y >= 0 { Face::Bottom } else { Face::Top };
        } else {
            distance = next.z;
            next.z += delta.z;
            cell.z += step.z;
            face = if step.z >= 0 {
                Face::South
            } else {
                Face::North
            };
        }
    }
    None
}

fn entry_face(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3, entry: f32) -> Face {
    if origin.x > min.x
        && origin.x < max.x
        && origin.y > min.y
        && origin.y < max.y
        && origin.z > min.z
        && origin.z < max.z
    {
        // A ray starting inside the world crosses no boundary, so there is no entry face to
        // compute. Label it the face the viewer is looking at head-on, which is what the DDA
        // would report had it marched in from outside.
        return facing_face(direction);
    }
    for (origin, direction, low, high, positive, negative) in [
        (origin.x, direction.x, min.x, max.x, Face::West, Face::East),
        (origin.y, direction.y, min.y, max.y, Face::Bottom, Face::Top),
        (
            origin.z,
            direction.z,
            min.z,
            max.z,
            Face::South,
            Face::North,
        ),
    ] {
        if direction.abs() >= f32::EPSILON {
            let boundary = if direction > 0.0 { low } else { high };
            if ((boundary - origin) / direction - entry).abs() <= 1e-5 {
                return if direction > 0.0 { positive } else { negative };
            }
        }
    }
    Face::Top
}

/// The face a ray of this direction presents to the viewer: the one whose outward normal most
/// opposes travel. Deliberately the same mapping the DDA uses per step, so a cell hit from inside
/// the world is labelled exactly as the same cell hit from outside it.
fn facing_face(direction: Vec3) -> Face {
    let magnitude = direction.abs();
    if magnitude.x >= magnitude.y && magnitude.x >= magnitude.z {
        if direction.x >= 0.0 {
            Face::West
        } else {
            Face::East
        }
    } else if magnitude.y >= magnitude.z {
        if direction.y >= 0.0 {
            Face::Bottom
        } else {
            Face::Top
        }
    } else if direction.z >= 0.0 {
        Face::South
    } else {
        Face::North
    }
}

fn ray_box_interval(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> Option<(f32, f32)> {
    let mut entry = f32::NEG_INFINITY;
    let mut exit = f32::INFINITY;
    for (origin, direction, min, max) in [
        (origin.x, direction.x, min.x, max.x),
        (origin.y, direction.y, min.y, max.y),
        (origin.z, direction.z, min.z, max.z),
    ] {
        if direction.abs() < f32::EPSILON {
            if origin < min || origin > max {
                return None;
            }
        } else {
            let first = (min - origin) / direction;
            let second = (max - origin) / direction;
            entry = entry.max(first.min(second));
            exit = exit.min(first.max(second));
        }
    }
    (entry <= exit).then_some((entry, exit))
}

fn ray_axis_distance(origin: f32, direction: f32, boundary: f32) -> f32 {
    if direction.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        (boundary - origin) / direction
    }
}

fn ray_step_distance(direction: f32) -> f32 {
    if direction.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        direction.abs().recip()
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec3;
    use client_core::Mirror;
    use protocol::{Dims, Material, MessageType, Snapshot, Speed, Tile};

    use super::{Face, first_visible_hit};
    use crate::camera::CameraRig;
    use crate::project::is_visible_at_slice;
    use crate::transform::world_to_render;

    /// The scale the story documents and the vehicle actually runs. Every coverage this module
    /// had before this test was indirect, through the ECS, at 9x9x4 — a world small enough that
    /// a march which terminated early, drifted a cell, or exhausted its diagonal bound would
    /// still land on the right answer by luck.
    const DIMS: Dims = Dims {
        x: 128,
        y: 128,
        z: 32,
    };
    const TOP: i32 = DIMS.z as i32 - 1;

    /// Twenty-four stone pillars scattered across the full footprint, one of them crowned with
    /// two tiles of foliage. Sparse on purpose: the ray still marches hundreds of cells before
    /// it reaches anything, which is the property 9x9x4 cannot test, while the oracle below
    /// stays cheap enough to run inside the gate.
    fn pillars() -> Mirror {
        let mut tiles = vec![Tile::Empty; (DIMS.x * DIMS.y * DIMS.z) as usize];
        let index = |[x, y, z]: [i32; 3]| {
            (x as u32 + y as u32 * DIMS.x + z as u32 * DIMS.x * DIMS.y) as usize
        };
        for i in 0..24i32 {
            let [x, y] = [5 + i * 5, 7 + (i * 11) % 120];
            for z in 0..=(1 + (i * 3) % 8) {
                tiles[index([x, y, z])] = Tile::Solid(Material::Stone);
            }
        }
        // The crowned pillar: stone to z 3, foliage at z 4 and z 5.
        for z in 0..=3 {
            tiles[index([FOLIAGE_PILLAR[0], FOLIAGE_PILLAR[1], z])] = Tile::Solid(Material::Stone);
        }
        for z in 4..=5 {
            tiles[index([FOLIAGE_PILLAR[0], FOLIAGE_PILLAR[1], z])] =
                Tile::Solid(Material::TreeFoliage);
        }
        Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: DIMS,
            tiles,
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap()
    }

    const FOLIAGE_PILLAR: [i32; 2] = [100, 100];

    #[test]
    fn a_world_boundary_hit_keeps_its_entry_face() {
        let mirror = Mirror::from_snapshot(Snapshot {
            msg_type: MessageType::Snapshot,
            dims: Dims { x: 1, y: 1, z: 1 },
            tiles: vec![Tile::Solid(Material::Stone)],
            entities: Vec::new(),
            designations: Vec::new(),
            zones: Vec::new(),
            items: Vec::new(),
            speed: Speed::Normal,
            tick: 0,
        })
        .unwrap();
        assert_eq!(
            first_visible_hit(Vec3::new(-2.0, 0.0, 0.0), Vec3::X, &mirror, 0).map(|hit| hit.face),
            Some(Face::West),
        );
    }

    /// The pillar tops, hand-derived from `pillars`' own construction rule rather than read back
    /// out of the mirror, so a mirror that lost a pillar cannot quietly shrink the test.
    fn pillar_tops() -> Vec<[i32; 3]> {
        (0..24i32)
            .map(|i| [5 + i * 5, 7 + (i * 11) % 120, 1 + (i * 3) % 8])
            .collect()
    }

    /// INDEPENDENT ORACLE for the FACE, and deliberately not a restatement of the mapping the
    /// production code uses. The invariant a hit face must satisfy is geometric: you can only see
    /// the side of a cube that faces you, so the face's outward normal must OPPOSE the ray. That
    /// holds however the six labels are assigned, which is exactly what makes it able to catch a
    /// swapped pair — inverting `West`/`East` (or `Top`/`Bottom`) sends the normal the other way
    /// and every one of these dot products flips sign.
    ///
    /// This is the check the change shipped without: the per-step `face` assignments inside the
    /// DDA loop produce the face for essentially every real pick, and both could be inverted with
    /// the whole suite green. An inverted face offsets the hover slab INTO the neighbouring cube,
    /// which is 8.1's buried-highlight defect this story exists to fix.
    #[test]
    fn a_marched_hit_face_always_opposes_the_ray_that_found_it() {
        let mirror = pillars();
        // The first pillar, tall enough to be struck side-on and from above.
        let target = [5, 7, 1];
        let centre = world_to_render(target);
        let approaches = [
            (
                "from -x",
                Vec3::new(centre.x - 40.0, centre.y, centre.z),
                Vec3::X,
            ),
            (
                "from +x",
                Vec3::new(centre.x + 40.0, centre.y, centre.z),
                -Vec3::X,
            ),
            (
                "from -z",
                Vec3::new(centre.x, centre.y, centre.z - 40.0),
                Vec3::Z,
            ),
            (
                "from +z",
                Vec3::new(centre.x, centre.y, centre.z + 40.0),
                -Vec3::Z,
            ),
            (
                "from above",
                Vec3::new(centre.x, centre.y + 40.0, centre.z),
                -Vec3::Y,
            ),
        ];
        for (label, origin, direction) in approaches {
            let hit = first_visible_hit(origin, direction, &mirror, TOP)
                .unwrap_or_else(|| panic!("the ray {label} must strike the pillar"));
            assert_eq!(hit.tile, target, "the ray {label} struck the wrong cell");
            assert!(
                hit.face.normal().dot(direction) < 0.0,
                "the ray {label} hit {:?}, whose normal {:?} points ALONG the ray {:?} — that \
                 face is turned away from the viewer and its slab lands inside the neighbouring \
                 cube",
                hit.face,
                hit.face.normal(),
                direction
            );
        }
    }

    /// A camera embedded in or touching solid rock is reachable in normal play: the camera has no
    /// terrain collision, the zoom clamp bottoms out at 4 units, and designating by mouse is a
    /// close-range interaction against tunnel and shaft walls. The ray then crosses no world
    /// boundary, so there is no entry face to compute, and the historic fallback answered `Top`
    /// regardless of where the ray was pointing.
    #[test]
    fn a_ray_starting_inside_solid_rock_reports_the_face_it_looks_at_not_the_top() {
        let mirror = pillars();
        let target = [5, 7, 1];
        let origin = world_to_render(target);
        for direction in [Vec3::X, -Vec3::X, Vec3::Z, -Vec3::Z] {
            let hit = first_visible_hit(origin, direction, &mirror, TOP)
                .expect("a ray inside a solid cell hits that cell immediately");
            assert_eq!(hit.tile, target);
            assert_ne!(
                hit.face,
                Face::Top,
                "a ray travelling {direction:?} from inside the rock reported the TOP face; the \
                 slab is then laid flat on the cell roof instead of on the wall being looked at"
            );
            assert!(
                hit.face.normal().dot(direction) < 0.0,
                "the face must still oppose the ray, whatever the ray is doing inside the rock"
            );
        }
    }

    /// INDEPENDENT ORACLE. It answers the same question by a different method: instead of
    /// walking cells in order, it tests EVERY cell in the world against the ray and keeps the
    /// nearest visible hit. Slow by construction and correct by construction — the two
    /// properties the DDA trades away for speed, and therefore the two this pins.
    fn nearest_visible_cell(
        mirror: &Mirror,
        origin: Vec3,
        direction: Vec3,
        level: i32,
    ) -> Option<[i32; 3]> {
        let mut best: Option<(f32, [i32; 3])> = None;
        for z in 0..DIMS.z as i32 {
            for y in 0..DIMS.y as i32 {
                for x in 0..DIMS.x as i32 {
                    let position = [x, y, z];
                    if !matches!(mirror.tile(position), Some(Tile::Solid(_) | Tile::Ramp(_)))
                        || !is_visible_at_slice(mirror, position, level)
                        || matches!(
                            mirror.tile(position),
                            Some(Tile::Solid(Material::TreeFoliage))
                        )
                    {
                        continue;
                    }
                    let Some(entry) =
                        cell_entry_distance(origin, direction, world_to_render(position))
                    else {
                        continue;
                    };
                    if best.is_none_or(|(nearest, _)| entry < nearest) {
                        best = Some((entry, position));
                    }
                }
            }
        }
        best.map(|(_, position)| position)
    }

    /// Where the ray first enters one unit cell, or `None` if it misses it entirely.
    fn cell_entry_distance(origin: Vec3, direction: Vec3, centre: Vec3) -> Option<f32> {
        let mut entry = f32::NEG_INFINITY;
        let mut exit = f32::INFINITY;
        for axis in 0..3 {
            let (start, along) = (origin[axis], direction[axis]);
            let (low, high) = (centre[axis] - 0.5, centre[axis] + 0.5);
            if along == 0.0 {
                if start < low || start > high {
                    return None;
                }
            } else {
                let (first, second) = ((low - start) / along, (high - start) / along);
                entry = entry.max(first.min(second));
                exit = exit.min(first.max(second));
            }
        }
        (entry <= exit && exit >= 0.0).then_some(entry.max(0.0))
    }

    /// A ray from a real camera pose through a chosen cell's centre.
    fn ray_at(target: [i32; 3], yaw: f32, pitch: f32, distance: f32) -> (Vec3, Vec3) {
        let rig = CameraRig {
            focus: target,
            yaw,
            pitch,
            distance,
        };
        let origin = rig.transform().translation;
        (origin, (world_to_render(target) - origin).normalize())
    }

    #[test]
    fn the_march_agrees_with_an_independent_tracer_across_a_full_scale_world() {
        let mirror = pillars();
        let poses = [
            (0.15f32, 4.0f32),
            (0.45, 30.0),
            (0.45, 90.0),
            (1.4207963, 500.0),
        ];
        let mut hits = 0;
        for (index, target) in pillar_tops().into_iter().enumerate() {
            let yaw = -2.1 + index as f32 * 0.27;
            let (pitch, distance) = poses[index % poses.len()];
            let (origin, direction) = ray_at(target, yaw, pitch, distance);
            let marched = first_visible_hit(origin, direction, &mirror, TOP).map(|hit| hit.tile);
            let traced = nearest_visible_cell(&mirror, origin, direction, TOP);
            assert_eq!(
                marched, traced,
                "the march and the independent tracer must agree at target {target:?}, \
                 yaw={yaw}, pitch={pitch}, distance={distance}"
            );
            if marched.is_some() {
                hits += 1;
            }
        }
        assert_eq!(
            hits, 24,
            "every pillar top aimed at must actually be hit — an all-None run would make the \
             agreement above vacuous"
        );
    }

    #[test]
    fn a_ray_straight_down_a_full_scale_world_stops_at_the_first_pillar_it_meets() {
        let mirror = pillars();
        // Hand-written: pillar 3 stands at x 20, y 40, solid through z 0..=2.
        let above = world_to_render([20, 40, TOP]) + Vec3::Y * 10.0;
        assert_eq!(
            first_visible_hit(above, -Vec3::Y, &mirror, TOP).map(|hit| hit.tile),
            Some([20, 40, 2]),
            "the march must stop at the pillar's top tile, not run through it to the one below"
        );
    }

    #[test]
    fn foliage_is_never_picked_and_never_hides_the_trunk_beneath_it() {
        let mirror = pillars();
        let [x, y] = FOLIAGE_PILLAR;
        let above = world_to_render([x, y, TOP]) + Vec3::Y * 10.0;
        assert_eq!(
            first_visible_hit(above, -Vec3::Y, &mirror, TOP).map(|hit| hit.tile),
            Some([x, y, 3]),
            "tree foliage is represented by a presentation mesh while picking still walks the \
             authoritative tile grid, so foliage must stay non-pickable and reveal the trunk \
             tile beneath it"
        );
    }
}
