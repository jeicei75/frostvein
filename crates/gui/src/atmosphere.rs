use bevy::{
    color::Alpha,
    prelude::{
        AlphaMode, Assets, Commands, Component, Cuboid, Mesh, Mesh3d, MeshMaterial3d, Query, Res,
        ResMut, StandardMaterial, Time, Transform, Vec3, With,
    },
};

use crate::{
    appearance::{material_color, night_lighting},
    camera::{BOOT_ASPECT_RATIO, BOOT_VERTICAL_FOV, CameraRig},
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
pub const SNOWFLAKE_SCALE: f32 = 0.28;
const AURORA_BAND_SCALE: Vec3 = Vec3::new(56.0, 6.0, 0.15);

pub fn aurora_positions() -> [Vec3; 3] {
    [
        Vec3::new(8.0, 28.0, -140.0),
        Vec3::new(64.0, 31.0, -146.0),
        Vec3::new(100.0, 28.0, -140.0),
    ]
}

pub fn star_positions() -> [Vec3; 12] {
    [
        Vec3::new(-180.0, 35.0, -160.0),
        Vec3::new(-130.0, 33.0, -150.0),
        Vec3::new(-80.0, 36.0, -155.0),
        Vec3::new(-35.0, 30.0, -142.0),
        Vec3::new(10.0, 41.0, -160.0),
        Vec3::new(45.0, 32.0, -146.0),
        Vec3::new(75.0, 38.0, -150.0),
        Vec3::new(95.0, 30.0, -138.0),
        Vec3::new(104.0, 33.0, -140.0),
        Vec3::new(-100.0, 28.0, -135.0),
        Vec3::new(25.0, 28.0, -135.0),
        Vec3::new(100.0, 32.0, -135.0),
    ]
}

pub fn snowflake_positions() -> [Vec3; 36] {
    std::array::from_fn(|index| {
        Vec3::new(
            52.0 + (index % 6) as f32 * 4.0,
            16.0 + (index / 6) as f32 * 2.0,
            -86.0 + (index / 6) as f32 * 2.0,
        )
    })
}

pub fn aurora_light_transform() -> Transform {
    let source = aurora_positions().into_iter().sum::<Vec3>() / 3.0;
    Transform::from_translation(source).looking_at(CAMP_FOCUS, Vec3::Y)
}

pub fn inside_boot_frustum(position: Vec3) -> bool {
    let camera = CameraRig::new([64, 64, 9]).transform();
    let offset = position - camera.translation;
    let depth = offset.dot(camera.forward().as_vec3());
    if depth <= 0.0 {
        return false;
    }
    let half_vertical = (BOOT_VERTICAL_FOV * 0.5).tan();
    offset.dot(*camera.up()).abs() <= depth * half_vertical
        && offset.dot(camera.right().as_vec3()).abs() <= depth * half_vertical * BOOT_ASPECT_RATIO
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
            Transform::from_translation(position).with_scale(AURORA_BAND_SCALE),
            Atmosphere,
            ClientLocal,
        ));
    }
    for position in snowflake_positions() {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(snow.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(SNOWFLAKE_SCALE)),
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
        CAMP_FOCUS, FAR_TERRAIN_EDGE, SKYLINE_MAX, SNOWFLAKE_SCALE, aurora_light_transform,
        aurora_positions, inside_boot_frustum, snowflake_positions, star_positions,
    };
    use crate::camera::CameraRig;

    #[test]
    fn atmosphere_positions_stay_outside_the_terrain_and_inside_the_boot_frustum() {
        for band in aurora_positions() {
            assert!(
                band.z < FAR_TERRAIN_EDGE,
                "aurora belongs beyond the far edge"
            );
            assert!(band.y > SKYLINE_MAX, "aurora belongs above the skyline");
            assert!(
                inside_boot_frustum(band),
                "aurora must be visible at the boot framing"
            );
        }
        for star in star_positions() {
            assert!(
                star.z < FAR_TERRAIN_EDGE,
                "stars belong beyond the far edge"
            );
            assert!(star.y > SKYLINE_MAX, "stars belong above the skyline");
            assert!(
                inside_boot_frustum(star),
                "stars must be visible at the boot framing"
            );
        }
        for flake in snowflake_positions() {
            assert!(
                flake.distance(CAMP_FOCUS) <= 32.0,
                "snowfall remains in the camp read"
            );
            assert!(
                inside_boot_frustum(flake),
                "snowfall must be visible at the boot framing"
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

    #[test]
    fn snowfall_fills_a_visible_grid_instead_of_a_single_diagonal_row() {
        let flakes = snowflake_positions();

        assert_eq!(flakes.len(), 36);
        assert_eq!(SNOWFLAKE_SCALE, 0.28);
        assert_eq!(flakes[0].x, flakes[6].x, "rows share x columns");
        assert_ne!(
            flakes[0].z, flakes[6].z,
            "a shared x column must span multiple z rows"
        );
    }

    #[test]
    fn stars_span_the_boot_sky_instead_of_a_thin_horizon_line() {
        let rig = CameraRig::new([64, 64, 9]);
        let screen_positions = star_positions().map(|star| {
            rig.project_render_point(star)
                .expect("all stars must stay in front of the boot camera")
        });
        let min_x = screen_positions
            .iter()
            .map(|position| position.x)
            .fold(f32::INFINITY, f32::min);
        let max_x = screen_positions
            .iter()
            .map(|position| position.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = screen_positions
            .iter()
            .map(|position| position.y)
            .fold(f32::INFINITY, f32::min);
        let max_y = screen_positions
            .iter()
            .map(|position| position.y)
            .fold(f32::NEG_INFINITY, f32::max);

        assert!(
            max_x - min_x >= 0.65,
            "stars must cross most of the sky width"
        );
        assert!(
            max_y - min_y >= 0.25,
            "stars must fill more than one horizon row"
        );
    }
}
