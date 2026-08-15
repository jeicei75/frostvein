use bevy::prelude::{
    Assets, Commands, Component, Cuboid, Mesh, Mesh3d, MeshMaterial3d, Query, Res, ResMut,
    StandardMaterial, Time, Transform, Vec3, With,
};

use crate::{
    appearance::{material_color, night_lighting},
    project::ClientLocal,
};

#[derive(Component)]
pub struct Snowflake;

/// Builds decorative geometry without consulting the mirror: atmosphere has no sim meaning.
pub fn setup_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(Cuboid::default()));
    let star = materials.add(StandardMaterial {
        emissive: bevy::prelude::Color::WHITE.to_linear(),
        unlit: true,
        ..Default::default()
    });
    let aurora = materials.add(StandardMaterial {
        base_color: night_lighting().aurora,
        emissive: night_lighting().aurora.to_linear() * 0.35,
        unlit: true,
        ..Default::default()
    });
    let snow = materials.add(StandardMaterial {
        base_color: material_color(protocol::Material::Snow),
        unlit: true,
        ..Default::default()
    });

    for index in 0..12 {
        let x = -90.0 + (index % 6) as f32 * 36.0;
        let y = 30.0 + (index / 6) as f32 * 18.0;
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(star.clone()),
            Transform::from_xyz(x, y, -100.0).with_scale(Vec3::splat(0.35)),
            ClientLocal,
        ));
    }
    for index in 0..3 {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(aurora.clone()),
            Transform::from_xyz(
                -35.0 + index as f32 * 35.0,
                12.0 + index as f32 * 3.0,
                -85.0,
            )
            .with_scale(Vec3::new(22.0, 3.0, 0.15)),
            ClientLocal,
        ));
    }
    for index in 0..16 {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(snow.clone()),
            Transform::from_xyz(
                -40.0 + (index % 8) as f32 * 11.0,
                18.0 + (index / 8) as f32 * 8.0,
                -15.0 + (index % 4) as f32 * 10.0,
            )
            .with_scale(Vec3::splat(0.12)),
            Snowflake,
            ClientLocal,
        ));
    }
}

pub fn fall_snow(time: Res<Time>, mut flakes: Query<&mut Transform, With<Snowflake>>) {
    for mut transform in &mut flakes {
        transform.translation.y -= time.delta_secs() * 1.2;
        if transform.translation.y < -2.0 {
            transform.translation.y = 28.0;
        }
    }
}
