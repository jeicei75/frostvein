use bevy::{
    color::Alpha,
    prelude::{
        AlphaMode, Assets, Commands, Component, Cuboid, Mesh, Mesh3d, MeshMaterial3d, Query, Res,
        ResMut, StandardMaterial, Time, Transform, Vec3, With,
    },
};

use crate::{
    appearance::{material_color, night_lighting},
    project::ClientLocal,
};

#[derive(Component)]
pub struct Snowflake;

#[derive(Component)]
pub struct Atmosphere;

pub const CAMP_SURFACE_Y: f32 = 9.0;
pub const CAMP_FOCUS: Vec3 = Vec3::new(64.0, CAMP_SURFACE_Y, -64.0);
pub const SKYLINE_MAX: f32 = 26.0;
pub const FAR_TERRAIN_EDGE: f32 = -128.0;

pub fn aurora_positions() -> [Vec3; 3] {
    [
        Vec3::new(28.0, 32.0, -150.0),
        Vec3::new(58.0, 36.0, -154.0),
        Vec3::new(88.0, 40.0, -150.0),
    ]
}

pub fn star_positions() -> [Vec3; 12] {
    std::array::from_fn(|index| {
        Vec3::new(
            18.0 + (index % 6) as f32 * 20.0,
            42.0 + (index / 6) as f32 * 14.0,
            -146.0 - (index % 3) as f32 * 7.0,
        )
    })
}

pub fn snowflake_positions() -> [Vec3; 16] {
    std::array::from_fn(|index| {
        Vec3::new(
            48.0 + (index % 4) as f32 * 10.0,
            14.0 + (index / 4) as f32 * 4.0,
            -78.0 + (index % 4) as f32 * 9.0,
        )
    })
}

pub fn aurora_light_transform() -> Transform {
    let source = aurora_positions().into_iter().sum::<Vec3>() / 3.0;
    Transform::from_translation(source).looking_at(CAMP_FOCUS, Vec3::Y)
}

/// Builds decorative geometry without consulting the mirror: atmosphere has no sim meaning.
pub fn setup_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(Cuboid::default()));
    let star = materials.add(StandardMaterial {
        base_color: night_lighting().star,
        unlit: true,
        ..Default::default()
    });
    let aurora = materials.add(StandardMaterial {
        base_color: night_lighting().aurora.with_alpha(0.45),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });
    let snow = materials.add(StandardMaterial {
        base_color: material_color(protocol::Material::Snow),
        unlit: true,
        ..Default::default()
    });

    for position in star_positions() {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(star.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(0.35)),
            Atmosphere,
            ClientLocal,
        ));
    }
    for position in aurora_positions() {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(aurora.clone()),
            Transform::from_translation(position).with_scale(Vec3::new(24.0, 3.0, 0.15)),
            Atmosphere,
            ClientLocal,
        ));
    }
    for position in snowflake_positions() {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(snow.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(0.12)),
            Snowflake,
            Atmosphere,
            ClientLocal,
        ));
    }
}

pub fn fall_snow(time: Res<Time>, mut flakes: Query<&mut Transform, With<Snowflake>>) {
    for mut transform in &mut flakes {
        transform.translation.y -= time.delta_secs() * 1.2;
        if transform.translation.y < CAMP_SURFACE_Y {
            transform.translation.y = 28.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CAMP_FOCUS, FAR_TERRAIN_EDGE, SKYLINE_MAX, aurora_light_transform, aurora_positions,
        snowflake_positions, star_positions,
    };

    #[test]
    fn atmosphere_positions_stay_outside_the_terrain_and_inside_the_boot_read() {
        for band in aurora_positions() {
            assert!(
                band.z < FAR_TERRAIN_EDGE,
                "aurora belongs beyond the far edge"
            );
            assert!(band.y > SKYLINE_MAX, "aurora belongs above the skyline");
        }
        for star in star_positions() {
            assert!(star.y > SKYLINE_MAX, "stars belong above the skyline");
        }
        for flake in snowflake_positions() {
            assert!(
                flake.distance(CAMP_FOCUS) <= 32.0,
                "snowfall remains in the camp read"
            );
        }

        let source = aurora_positions().into_iter().sum::<bevy::prelude::Vec3>() / 3.0;
        let toward_camp = (CAMP_FOCUS - source).normalize();
        assert!(
            aurora_light_transform()
                .forward()
                .as_vec3()
                .dot(toward_camp)
                > 0.99,
            "aurora light must arrive from the band side"
        );
    }
}
