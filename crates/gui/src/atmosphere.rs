use bevy::{
    asset::RenderAssetUsages,
    color::ColorToPacked,
    image::{Image, ImageSampler},
    mesh::{Indices, PrimitiveTopology},
    prelude::{
        AlphaMode, Assets, Commands, Component, Cuboid, Mesh, Mesh3d, MeshMaterial3d, Query, Res,
        ResMut, StandardMaterial, Time, Transform, Vec3, With,
    },
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{
    appearance::{material_color, night_lighting},
    camera::{BOOT_ASPECT_RATIO, BOOT_VERTICAL_FOV, CameraRig, boot_horizontal_forward},
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

/// The horizontal centre of the world footprint; all sky geometry is hung around it.
pub const SKY_CENTRE: Vec3 = Vec3::new(63.5, 0.0, -63.5);

/// The aurora is a curtain on a ring around the world, not a billboard: the camera orbits,
/// so a single flat quad would turn edge-on. The radius must exceed the camera's furthest
/// horizontal excursion (426 units at the 500 zoom clamp) or the vista swings the camera
/// OUTSIDE the ring and the curtain crosses in front of the valley. Bottom and top are chosen
/// so the curtain spans the sky wedge this camera can see while its top stays at or below the
/// boot eye line — that is "hugs the horizon rather than hanging overhead" in geometry.
pub const AURORA_RADIUS: f32 = 600.0;
pub const AURORA_BOTTOM: f32 = -162.0;
pub const AURORA_TOP: f32 = 45.0;
const AURORA_SEGMENTS: usize = 48;
pub const AURORA_TEXTURE_WIDTH: usize = 256;
pub const AURORA_TEXTURE_HEIGHT: usize = 64;
/// Peak opacity of the curtain. Well below 1.0 so stars read through it (AC5's translucency).
const AURORA_PEAK_ALPHA: f32 = 0.55;

pub const STAR_RADIUS: f32 = 650.0;
pub const STAR_COUNT: usize = 300;
const STAR_BAND_LOW: f32 = -130.0;
const STAR_BAND_HIGH: f32 = 120.0;
// Sized for the shell's depth: at 650 units a frame pixel is ~0.75 world units.
const STAR_SCALE_MIN: f32 = 1.1;
const STAR_SCALE_MAX: f32 = 3.0;
/// Golden-angle azimuth stepping scatters the shell without a random source or a stored table.
const GOLDEN_ANGLE: f32 = 2.399_963_2;
const GOLDEN_RATIO_FRACT: f32 = 0.618_034;

/// The compass point the curtain is brightest at, and where its light comes from.
pub fn aurora_core() -> Vec3 {
    SKY_CENTRE
        + boot_horizontal_forward() * AURORA_RADIUS
        + Vec3::Y * (AURORA_BOTTOM + AURORA_TOP) * 0.5
}

/// A ring strip around the world. Winding is irrelevant — the material disables culling so the
/// curtain reads from inside the ring, which is where the camera always is.
pub fn aurora_curtain_mesh() -> Mesh {
    let mut positions = Vec::with_capacity((AURORA_SEGMENTS + 1) * 2);
    let mut uvs = Vec::with_capacity((AURORA_SEGMENTS + 1) * 2);
    let mut normals = Vec::with_capacity((AURORA_SEGMENTS + 1) * 2);
    for segment in 0..=AURORA_SEGMENTS {
        let fraction = segment as f32 / AURORA_SEGMENTS as f32;
        let angle = fraction * std::f32::consts::TAU;
        let x = SKY_CENTRE.x + AURORA_RADIUS * angle.cos();
        let z = SKY_CENTRE.z + AURORA_RADIUS * angle.sin();
        positions.push([x, AURORA_TOP, z]);
        positions.push([x, AURORA_BOTTOM, z]);
        uvs.push([fraction, 0.0]);
        uvs.push([fraction, 1.0]);
        let inward = [-angle.cos(), 0.0, -angle.sin()];
        normals.push(inward);
        normals.push(inward);
    }
    let mut indices = Vec::with_capacity(AURORA_SEGMENTS * 6);
    for segment in 0..AURORA_SEGMENTS as u32 {
        let base = segment * 2;
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 1, base + 3]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// RGBA pixels for the curtain: the table's aurora colour throughout, with the shape carried
/// entirely by the alpha ramp. `sin(pi * v)` forces alpha to exactly zero at both edges of the
/// strip, so the curtain has no silhouette — the defect three opaque cuboids could not avoid.
///
/// NOTE: one hue for the whole curtain. Real aurorae shift hue with altitude; that needs a
/// second table entry and is not what this story is judged on.
pub fn aurora_gradient_pixels() -> Vec<u8> {
    let rgb = night_lighting().aurora.to_srgba().to_u8_array_no_alpha();
    let mut data = Vec::with_capacity(AURORA_TEXTURE_WIDTH * AURORA_TEXTURE_HEIGHT * 4);
    for row in 0..AURORA_TEXTURE_HEIGHT {
        let v = row as f32 / (AURORA_TEXTURE_HEIGHT - 1) as f32;
        for column in 0..AURORA_TEXTURE_WIDTH {
            let u = column as f32 / AURORA_TEXTURE_WIDTH as f32;
            // Integer cycle counts keep the ring seamless where u wraps.
            let folds = 0.55
                + 0.45
                    * (0.6 * (u * std::f32::consts::TAU * 3.0).sin()
                        + 0.4 * (u * std::f32::consts::TAU * 7.0 + 1.7).sin());
            let peak = 0.62 + 0.10 * (u * std::f32::consts::TAU * 2.0 + 0.9).sin();
            let band = (-(((v - peak) / 0.26).powi(2))).exp();
            let edges = (v * std::f32::consts::PI).sin();
            let alpha = (AURORA_PEAK_ALPHA * folds.max(0.0) * band * edges).clamp(0.0, 1.0);
            data.extend_from_slice(&[rgb[0], rgb[1], rgb[2], (alpha * 255.0).round() as u8]);
        }
    }
    data
}

pub fn aurora_gradient_image() -> Image {
    let mut image = Image::new(
        Extent3d {
            width: AURORA_TEXTURE_WIDTH as u32,
            height: AURORA_TEXTURE_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        aurora_gradient_pixels(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    // Nearest sampling would turn the ramp back into visible bands.
    image.sampler = ImageSampler::linear();
    image
}

/// Stars sit on a shell around the world for the same reason the aurora does: the camera orbits.
/// The band is deliberately narrow — this camera looks down, so the visible sky is a thin wedge
/// above the ridge line and a full dome would put most of the stars out of every frame.
pub fn star_positions() -> [Vec3; STAR_COUNT] {
    std::array::from_fn(|index| {
        let azimuth = index as f32 * GOLDEN_ANGLE;
        let height = STAR_BAND_LOW
            + (STAR_BAND_HIGH - STAR_BAND_LOW) * (index as f32 * GOLDEN_RATIO_FRACT).fract();
        Vec3::new(
            SKY_CENTRE.x + STAR_RADIUS * azimuth.cos(),
            height,
            SKY_CENTRE.z + STAR_RADIUS * azimuth.sin(),
        )
    })
}

/// Uniform stars read as a lattice; this varies the size deterministically instead.
pub fn star_scale(index: usize) -> f32 {
    STAR_SCALE_MIN + (STAR_SCALE_MAX - STAR_SCALE_MIN) * (index as f32 * 0.381_966 + 0.21).fract()
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
    Transform::from_translation(aurora_core()).looking_at(CAMP_FOCUS, Vec3::Y)
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
    mut images: ResMut<Assets<Image>>,
) {
    let cube = meshes.add(Mesh::from(Cuboid::default()));
    let curtain = meshes.add(aurora_curtain_mesh());
    let gradient = images.add(aurora_gradient_image());
    // The sky sits far outside the fog volume that dissolves the world edge; without this it
    // would be fogged out of existence along with the terrain behind it.
    let star = materials.add(StandardMaterial {
        base_color: night_lighting().star,
        unlit: true,
        fog_enabled: false,
        ..Default::default()
    });
    // No base_color here on purpose: the curtain's colour lives in the gradient it samples,
    // which reads it from the table. An RGB literal at this draw site would be the AC13 leak.
    let aurora = materials.add(StandardMaterial {
        base_color_texture: Some(gradient),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        fog_enabled: false,
        cull_mode: None,
        ..Default::default()
    });
    let snow = materials.add(StandardMaterial {
        base_color: material_color(protocol::Material::Snow),
        unlit: true,
        ..Default::default()
    });

    for (index, position) in star_positions().into_iter().enumerate() {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(star.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(star_scale(index))),
            Atmosphere,
            ClientLocal,
        ));
    }
    commands.spawn((
        Mesh3d(curtain),
        MeshMaterial3d(aurora),
        Transform::IDENTITY,
        Atmosphere,
        ClientLocal,
    ));
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
        AURORA_BOTTOM, AURORA_RADIUS, AURORA_TEXTURE_HEIGHT, AURORA_TEXTURE_WIDTH, AURORA_TOP,
        CAMP_FOCUS, SKY_CENTRE, SKYLINE_MAX, SNOWFLAKE_SCALE, STAR_COUNT, STAR_RADIUS, aurora_core,
        aurora_curtain_mesh, aurora_gradient_pixels, aurora_light_transform, inside_boot_frustum,
        snowflake_positions, star_positions, star_scale,
    };
    use crate::appearance::night_lighting;
    use crate::camera::{BOOT_VERTICAL_FOV, CameraRig};
    use bevy::color::ColorToPacked;

    fn boot_eye_height() -> f32 {
        CameraRig::new([64, 64, 9]).transform().translation.y
    }

    #[test]
    fn the_aurora_curtain_hugs_the_horizon_beyond_the_world() {
        // AC5 in geometry, as an ANGLE rather than a raw height: a height threshold stops
        // meaning anything once the ring radius changes. The bar is a quarter of the camera's
        // vertical half-FOV (22.5 deg), so the curtain can never claim the upper sky. A first
        // attempt used 10 deg, which at a 600-unit radius let a 140-unit top through at 8 deg
        // — the sabotage caught it. Production reads -0.95 deg.
        let ceiling = (BOOT_VERTICAL_FOV * 0.5).to_degrees() / 4.0;
        let elevation = ((AURORA_TOP - boot_eye_height()) / AURORA_RADIUS)
            .atan()
            .to_degrees();
        assert!(
            elevation <= ceiling,
            "the curtain must sit on the horizon, not overhead; top is {elevation} deg up, \
             ceiling {ceiling}"
        );
        // Compile-time: the curtain must clear the skyline it backlights, and the ring must
        // enclose the whole 128-wide footprint or it would cut through terrain.
        const { assert!(AURORA_TOP > SKYLINE_MAX) };
        const { assert!(AURORA_RADIUS > 128.0) };
        assert!(
            inside_boot_frustum(aurora_core()),
            "the bright core of the curtain must be visible at the boot framing"
        );

        // The defect this pins: at the 500 zoom clamp the camera orbits 426 units out. A ring
        // smaller than that puts the camera OUTSIDE it, and the curtain crosses the valley.
        let mut vista = CameraRig::new([64, 64, 9]);
        vista.zoom(10_000.0);
        let eye = vista.transform().translation;
        let excursion = ((eye.x - SKY_CENTRE.x).powi(2) + (eye.z - SKY_CENTRE.z).powi(2)).sqrt();
        assert!(
            excursion < AURORA_RADIUS,
            "the camera must stay inside the curtain at every zoom; {excursion} vs {AURORA_RADIUS}"
        );
        assert!(
            excursion < STAR_RADIUS,
            "the camera must stay inside the star shell at every zoom; {excursion} vs {STAR_RADIUS}"
        );

        let toward_camp = (CAMP_FOCUS - aurora_core()).normalize();
        assert!(
            aurora_light_transform()
                .forward()
                .as_vec3()
                .dot(toward_camp)
                > 0.99,
            "aurora light must arrive from the curtain side"
        );
    }

    #[test]
    fn the_curtain_mesh_is_a_closed_ring_at_the_aurora_radius() {
        let mesh = aurora_curtain_mesh();
        let positions = mesh
            .attribute(bevy::mesh::Mesh::ATTRIBUTE_POSITION)
            .expect("the curtain needs positions")
            .as_float3()
            .expect("positions are 3-component floats");
        assert!(!positions.is_empty());
        for vertex in positions {
            let radius =
                ((vertex[0] - SKY_CENTRE.x).powi(2) + (vertex[2] - SKY_CENTRE.z).powi(2)).sqrt();
            assert!(
                (radius - AURORA_RADIUS).abs() < 0.01,
                "every curtain vertex sits on the ring; found radius {radius}"
            );
            assert!(
                vertex[1] == AURORA_TOP || vertex[1] == AURORA_BOTTOM,
                "curtain vertices belong to the top or bottom edge"
            );
        }
    }

    #[test]
    fn the_aurora_gradient_fades_to_nothing_at_both_edges() {
        // The defect this pins: an opaque band has a hard silhouette. Alpha must reach exactly
        // zero on the first and last row, or the curtain gets an edge again.
        let pixels = aurora_gradient_pixels();
        assert_eq!(
            pixels.len(),
            AURORA_TEXTURE_WIDTH * AURORA_TEXTURE_HEIGHT * 4
        );

        let alpha_at =
            |row: usize, column: usize| pixels[(row * AURORA_TEXTURE_WIDTH + column) * 4 + 3];
        for column in 0..AURORA_TEXTURE_WIDTH {
            assert_eq!(alpha_at(0, column), 0, "the top edge must be invisible");
            assert_eq!(
                alpha_at(AURORA_TEXTURE_HEIGHT - 1, column),
                0,
                "the bottom edge must be invisible"
            );
        }

        let peak = (0..AURORA_TEXTURE_HEIGHT)
            .flat_map(|row| (0..AURORA_TEXTURE_WIDTH).map(move |column| (row, column)))
            .map(|(row, column)| alpha_at(row, column))
            .max()
            .expect("the gradient has pixels");
        assert!(
            (60..=170).contains(&peak),
            "the curtain must be clearly visible but still translucent; peak alpha {peak}"
        );

        // Hand-written oracle: the colour is the table's, never a literal at the draw site.
        let expected = night_lighting().aurora.to_srgba().to_u8_array_no_alpha();
        assert_eq!(&pixels[0..3], &expected[..]);
    }

    #[test]
    fn the_star_shell_fills_the_visible_sky_wedge() {
        let rig = CameraRig::new([64, 64, 9]);
        let visible: Vec<_> = star_positions()
            .into_iter()
            .filter_map(|star| rig.project_render_point(star))
            .filter(|screen| (0.0..=1.0).contains(&screen.x) && (0.0..=1.0).contains(&screen.y))
            .collect();

        assert!(
            visible.len() >= 30,
            "the boot sky needs a real star field; only {} of {STAR_COUNT} land in frame",
            visible.len()
        );
        let min_x = visible.iter().map(|s| s.x).fold(f32::INFINITY, f32::min);
        let max_x = visible
            .iter()
            .map(|s| s.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = visible.iter().map(|s| s.y).fold(f32::INFINITY, f32::min);
        let max_y = visible
            .iter()
            .map(|s| s.y)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max_x - min_x >= 0.65,
            "stars must cross most of the sky width; spread {}",
            max_x - min_x
        );
        assert!(
            max_y - min_y >= 0.10,
            "stars must fill more than one horizon row; spread {}",
            max_y - min_y
        );
        // Every visible star must be above the ridge line, not sprinkled over the valley.
        assert!(
            visible.iter().all(|screen| screen.y < 0.30),
            "stars belong in the sky above the skyline"
        );
    }

    #[test]
    fn star_sizes_vary_so_the_shell_never_reads_as_a_lattice() {
        let scales: Vec<f32> = (0..12).map(star_scale).collect();
        let min = scales.iter().copied().fold(f32::INFINITY, f32::min);
        let max = scales.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max - min > 0.6,
            "star scales must actually vary; spread {}",
            max - min
        );
        // Literal bounds, sized for the 650-unit shell where a frame pixel is ~0.75 units.
        assert!(scales.iter().all(|scale| *scale >= 1.1 && *scale <= 3.0));
    }

    #[test]
    fn snowfall_stays_in_the_camp_read_and_in_frame() {
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
}
