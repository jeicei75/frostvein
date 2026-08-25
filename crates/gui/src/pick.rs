use bevy::{
    camera::Camera,
    prelude::{Camera3d, GlobalTransform, Query, Res, ResMut, Resource, Vec3, Window, With},
    window::PrimaryWindow,
};

use crate::{
    ingest::MirrorResource, project::is_visible_at_slice, slice::SliceLevel,
    transform::render_to_world,
};

/// The client-local tile currently under the cursor.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickedTile(pub Option<[i32; 3]>);

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
) -> Option<[i32; 3]> {
    let dims = mirror.dims();
    let min = Vec3::new(-0.5, -0.5, -(dims.y as f32) + 0.5);
    let max = Vec3::new(dims.x as f32 - 0.5, dims.z as f32 - 0.5, 0.5);
    let (entry, exit) = ray_box_interval(origin, direction, min, max)?;
    let diagonal = Vec3::new(dims.x as f32, dims.z as f32, dims.y as f32).length();
    let mut distance = entry.max(0.0);
    let end = (distance + diagonal).min(exit);
    if distance > end {
        return None;
    }

    // Move the entry point one representable step into the box so a ray landing exactly on a
    // voxel boundary starts in the cell it enters rather than the one it just missed.
    let point = origin + direction * (distance + f32::EPSILON);
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

    while distance <= end {
        let centre = cell.as_vec3();
        let world = render_to_world(centre);
        if mirror.tile(world).is_some() && is_visible_at_slice(mirror, world, level) {
            return Some(world);
        }
        if next.x <= next.y && next.x <= next.z {
            distance = next.x;
            next.x += delta.x;
            cell.x += step.x;
        } else if next.y <= next.z {
            distance = next.y;
            next.y += delta.y;
            cell.y += step.y;
        } else {
            distance = next.z;
            next.z += delta.z;
            cell.z += step.z;
        }
    }
    None
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
