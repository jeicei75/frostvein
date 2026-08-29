use std::{fs, path::Path};

fn repo_file(relative: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(relative))
        .expect("committed bench contract source must be readable")
}

#[test]
fn bench_literals_match_the_client_palette_lights_and_boot_camera() {
    let appearance = repo_file("crates/gui/src/appearance.rs");
    let camera = repo_file("crates/gui/src/camera.rs");
    let bench = repo_file("scripts/bench/valley_bench.py");

    for (client_literal, bench_literal) in [
        (
            "Material::Stone => Color::srgb_u8(60, 70, 92)",
            "\"stone\": (60, 70, 92)",
        ),
        (
            "Material::Soil => Color::srgb_u8(56, 52, 62)",
            "\"soil\": (56, 52, 62)",
        ),
        (
            "Material::Ice => Color::srgb_u8(104, 128, 170)",
            "\"ice\": (104, 128, 170)",
        ),
        (
            "Material::Snow => Color::srgb_u8(136, 150, 178)",
            "\"snow\": (136, 150, 178)",
        ),
        (
            "Material::TreeTrunk => Color::srgb_u8(43, 47, 58)",
            "\"tree_trunk\": (43, 47, 58)",
        ),
        (
            "Material::TreeFoliage => Color::srgb_u8(44, 100, 58)",
            "\"tree_foliage\": (44, 100, 58)",
        ),
        (
            "Color::srgb_u8(146, 158, 184)",
            "SNOW_CAP_RGB = (146, 158, 184)",
        ),
        (
            "Color::srgb_u8(156, 170, 196)",
            "FOLIAGE_SNOW_RGB = (156, 170, 196)",
        ),
        ("sky: Color::srgb_u8(5, 12, 28)", "SKY_RGB = (5, 12, 28)"),
        (
            "ambient: Color::srgb_u8(120, 140, 165)",
            "AMBIENT_RGB = (120, 140, 165)",
        ),
        (
            "directional: Color::srgb_u8(150, 190, 180)",
            "DIRECTIONAL_RGB = (150, 190, 180)",
        ),
        (
            "color: Color::srgb_u8(255, 140, 62)",
            "\"torch\": (255, 140, 62)",
        ),
        (
            "color: Color::srgb_u8(255, 173, 92)",
            "\"campfire\": (255, 173, 92)",
        ),
        (
            "color: Color::srgb_u8(255, 195, 110)",
            "\"lantern\": (255, 195, 110)",
        ),
        (
            "EntityKind::Dwarf => EntityAppearance {",
            "\"dwarf\": ((151, 116, 96), 0.65)",
        ),
    ] {
        assert!(
            appearance.contains(client_literal),
            "client literal drifted: {client_literal}"
        );
        assert!(
            bench.contains(bench_literal),
            "bench literal drifted: {bench_literal}"
        );
    }

    for (client_literal, bench_literal) in [
        (
            "const BOOT_YAW: f32 = 0.7;",
            "yaw, pitch, distance = 0.7, 0.45, 90.0",
        ),
        (
            "const BOOT_PITCH: f32 = 0.45;",
            "yaw, pitch, distance = 0.7, 0.45, 90.0",
        ),
        (
            "const BOOT_DISTANCE: f32 = 90.0;",
            "yaw, pitch, distance = 0.7, 0.45, 90.0",
        ),
        (
            "const BOOT_COMPOSITION_FORWARD: f32 = 33.0;",
            "vector_scale(forward, 33.0)",
        ),
        (
            "const BOOT_COMPOSITION_LIFT: f32 = -0.5;",
            "(0.0, -0.5, 0.0),",
        ),
        (
            "BOOT_VERTICAL_FOV: f32 = std::f32::consts::FRAC_PI_4",
            "camera_data.angle = math.pi / 4",
        ),
        ("BOOT_ASPECT_RATIO: f32 = 16.0 / 9.0", "resolution_x = 960"),
    ] {
        assert!(
            camera.contains(client_literal),
            "client camera literal drifted: {client_literal}"
        );
        assert!(
            bench.contains(bench_literal),
            "bench camera literal drifted: {bench_literal}"
        );
    }
    assert!(
        bench.contains("resolution_y = 540"),
        "bench must preserve the 16:9 aspect ratio"
    );
    assert!(bench.contains("camera_data.sensor_fit = \"VERTICAL\""));
}
