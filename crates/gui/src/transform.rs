use bevy::prelude::Vec3;

/// Converts the simulation's right-handed Z-up coordinates to Bevy's Y-up space.
pub fn world_to_render([x, y, z]: [i32; 3]) -> Vec3 {
    Vec3::new(x as f32, z as f32, -y as f32)
}

/// Converts a voxel-aligned Bevy position back to simulation coordinates.
pub fn render_to_world(value: Vec3) -> [i32; 3] {
    [value.x as i32, -value.z as i32, value.y as i32]
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec3;

    use super::{render_to_world, world_to_render};

    #[test]
    fn coordinate_transform_round_trips_a_spread() {
        for point in [[0, 0, 0], [3, -4, 7], [-12, 9, -5], [128, 127, 31]] {
            assert_eq!(render_to_world(world_to_render(point)), point);
        }
    }

    #[test]
    fn coordinate_transform_preserves_the_pinned_handedness() {
        // NOTE: this is hand-written literally so a mirrored but round-tripping
        // transform cannot pass by deriving its expected result from production code.
        assert_eq!(world_to_render([3, -4, 7]), Vec3::new(3.0, 7.0, 4.0));
    }
}
