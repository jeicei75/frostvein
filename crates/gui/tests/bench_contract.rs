use std::{fs, path::Path};

fn repo_file(relative: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(relative))
        .expect("committed bench contract source must be readable")
}

/// Collapse every run of whitespace to a single space so an anchor can span lines, and survive a
/// rustfmt rewrap, without the test caring about indentation.
fn squeezed(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every anchor must match EXACTLY ONCE.
///
/// A count of zero is the obvious failure. A count above one is the subtle one, and it shipped:
/// the light rows used to match a bare `color: Color::srgb_u8(255, 140, 62)` against the whole of
/// `appearance.rs`, where that literal occurs four times. Torch and Campfire could be swapped in
/// the client and every literal was still present, so the guard stayed green while the bench's
/// lights no longer matched the build.
fn assert_anchor(haystack: &str, needle: &str, side: &str) {
    let count = haystack.matches(needle).count();
    assert_eq!(count, 1, "{side} anchor must match exactly once, matched {count}: {needle}");
}

#[test]
fn bench_literals_match_the_client_palette_lights_and_boot_camera() {
    let appearance = squeezed(&repo_file("crates/gui/src/appearance.rs"));
    let camera = squeezed(&repo_file("crates/gui/src/camera.rs"));
    let atmosphere = squeezed(&repo_file("crates/gui/src/atmosphere.rs"));
    let project = squeezed(&repo_file("crates/gui/src/project.rs"));
    let bench = squeezed(&repo_file("scripts/bench/valley_bench.py"));

    // Each row anchors the client literal to the ARM that owns it, never to a bare value: the
    // value alone cannot tell a Torch from a Campfire.
    for (client_source, client_literal, bench_literal) in [
        (&appearance, "Material::Stone => Color::srgb_u8(60, 70, 92)", "\"stone\": (60, 70, 92)"),
        (&appearance, "Material::Soil => Color::srgb_u8(56, 52, 62)", "\"soil\": (56, 52, 62)"),
        (&appearance, "Material::Ice => Color::srgb_u8(104, 128, 170)", "\"ice\": (104, 128, 170)"),
        (&appearance, "Material::Snow => Color::srgb_u8(136, 150, 178)", "\"snow\": (136, 150, 178)"),
        (
            &appearance,
            "Material::TreeTrunk => Color::srgb_u8(43, 47, 58)",
            "\"tree_trunk\": (43, 47, 58)",
        ),
        (
            &appearance,
            "Material::TreeFoliage => Color::srgb_u8(44, 100, 58)",
            "\"tree_foliage\": (44, 100, 58)",
        ),
        (&appearance, "Color::srgb_u8(146, 158, 184)", "SNOW_CAP_RGB = (146, 158, 184)"),
        (&appearance, "Color::srgb_u8(156, 170, 196)", "FOLIAGE_SNOW_RGB = (156, 170, 196)"),
        (&appearance, "sky: Color::srgb_u8(5, 12, 28)", "SKY_RGB = (5, 12, 28)"),
        (&appearance, "ambient: Color::srgb_u8(120, 140, 165)", "AMBIENT_RGB = (120, 140, 165)"),
        (
            &appearance,
            "directional: Color::srgb_u8(150, 190, 180)",
            "DIRECTIONAL_RGB = (150, 190, 180)",
        ),
        (
            &appearance,
            "LightKind::Torch => LightProperties { color: Color::srgb_u8(255, 140, 62),",
            "\"torch\": (255, 140, 62)",
        ),
        (
            &appearance,
            "LightKind::Campfire => LightProperties { color: Color::srgb_u8(255, 173, 92),",
            "\"campfire\": (255, 173, 92)",
        ),
        (
            &appearance,
            "LightKind::Lantern => LightProperties { color: Color::srgb_u8(255, 195, 110),",
            "\"lantern\": (255, 195, 110)",
        ),
        (
            &appearance,
            "EntityKind::Dwarf => EntityAppearance { color: Color::srgb_u8(151, 116, 96), scale: 0.65,",
            "\"dwarf\": ((151, 116, 96), 0.65)",
        ),
        (
            &appearance,
            "EntityKind::Torch => EntityAppearance { color: Color::srgb_u8(255, 140, 62), scale: 0.28,",
            "\"torch\": ((255, 140, 62), 0.28)",
        ),
        (
            &appearance,
            "EntityKind::Campfire => EntityAppearance { color: Color::srgb_u8(255, 173, 92), scale: 0.55,",
            "\"campfire\": ((255, 173, 92), 0.55)",
        ),
        // The boot camera. The bench holds these once each and both the projection and the
        // rendered camera read them, so a drift here cannot hide in one of two copies.
        (&camera, "const BOOT_YAW: f32 = 0.7;", "BOOT_YAW, BOOT_PITCH, BOOT_DISTANCE = 0.7, 0.45, 90.0"),
        (&camera, "const BOOT_PITCH: f32 = 0.45;", "BOOT_YAW, BOOT_PITCH, BOOT_DISTANCE = 0.7, 0.45, 90.0"),
        (&camera, "const BOOT_DISTANCE: f32 = 90.0;", "BOOT_YAW, BOOT_PITCH, BOOT_DISTANCE = 0.7, 0.45, 90.0"),
        (
            &camera,
            "const BOOT_COMPOSITION_FORWARD: f32 = 33.0;",
            "BOOT_COMPOSITION_FORWARD = 33.0",
        ),
        (&camera, "const BOOT_COMPOSITION_LIFT: f32 = -0.5;", "BOOT_COMPOSITION_LIFT = -0.5"),
        (
            &camera,
            "BOOT_VERTICAL_FOV: f32 = std::f32::consts::FRAC_PI_4",
            "BOOT_VERTICAL_FOV = math.pi / 4",
        ),
        (&camera, "BOOT_ASPECT_RATIO: f32 = 16.0 / 9.0", "BOOT_ASPECT_RATIO = 16.0 / 9.0"),
        // The key light's aim. The bench does not draw the aurora, but the client's directional
        // light comes FROM it, so these four numbers decide which faces are lit.
        (&atmosphere, "pub const AURORA_RADIUS: f32 = 600.0;", "AURORA_RADIUS = 600.0"),
        (&atmosphere, "pub const AURORA_BOTTOM: f32 = -162.0;", "AURORA_BOTTOM = -162.0"),
        (&atmosphere, "pub const AURORA_TOP: f32 = 45.0;", "AURORA_TOP = 45.0"),
        (
            &atmosphere,
            "pub const SKY_CENTRE: Vec3 = Vec3::new(63.5, 0.0, -63.5);",
            "SKY_CENTRE = (63.5, 0.0, -63.5)",
        ),
        (&atmosphere, "pub const CAMP_SURFACE_Y: f32 = 9.0;", "CAMP_FOCUS = (64.0, 9.0, -64.0)"),
        (
            &atmosphere,
            "Transform::from_translation(aurora_core()).looking_at(CAMP_FOCUS, Vec3::Y)",
            "vector_normalize(vector_subtract(CAMP_FOCUS, aurora_core()))",
        ),
        // The foliage shrink. Without it the bench drew solid canopy slabs where the client draws
        // sparse crowns, which is the difference an eye reads first.
        (
            &project,
            "match foliage_above { 0 => 0.62, 1 => 0.78, _ => 0.95, }",
            "return (0.62, 0.78, 0.95)[above]",
        ),
    ] {
        assert_anchor(client_source, client_literal, "client");
        assert_anchor(&bench, bench_literal, "bench");
    }

    // Single-sourcing, asserted directly: the renderer must READ the shared constants rather than
    // re-declare them. A parallel copy is how a 110-degree camera roll and a pi/3 FOV both passed
    // a green framing test.
    for wiring in [
        "camera_data.angle = BOOT_VERTICAL_FOV",
        "camera_data.sensor_fit = \"VERTICAL\"",
        "scene.render.resolution_x = RENDER_WIDTH",
        "scene.render.resolution_y = RENDER_HEIGHT",
        "RENDER_WIDTH = round(RENDER_HEIGHT * BOOT_ASPECT_RATIO)",
        "RENDER_HEIGHT = 540",
        // AMBIENT_RGB was a dead constant this test pinned as though it proved something.
        "ambient.inputs[\"Color\"].default_value = (*srgb_to_linear(AMBIENT_RGB), 1.0)",
        "ambient.inputs[\"Strength\"].default_value = AMBIENT_STRENGTH",
        "sun_data.color = srgb_to_linear(DIRECTIONAL_RGB)",
        "sun.rotation_euler = Vector(sun_direction()).to_track_quat(\"-Z\", \"Y\").to_euler()",
    ] {
        assert_anchor(&bench, wiring, "bench wiring");
    }
}
