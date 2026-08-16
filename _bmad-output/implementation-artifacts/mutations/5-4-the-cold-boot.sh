# Story 5.4 sabotage table. Run alone with scripts/mutate.sh.

mutation "snow cap leaves bare top" gui snow_caps_follow_material_and_exposure_in_a_seed_shaped_toy_world <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    matches!(\n        mirror.tile(position),\n        Some(Tile::Solid(material) | Tile::Ramp(material))\n            if material != Material::Ice && material != Material::TreeFoliage\n    ) && !matches!(\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    false && !matches!(\n'))
PY

mutation "ice tops receive snow" gui snow_caps_follow_material_and_exposure_in_a_seed_shaped_toy_world <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'if material != Material::Ice && material != Material::TreeFoliage'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if true'))
PY

mutation "foliage receives ground snow slabs" gui snow_caps_follow_material_and_exposure_in_a_seed_shaped_toy_world <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'material != Material::Ice && material != Material::TreeFoliage'
assert s.count(old) == 1
p.write_text(s.replace(old, 'material != Material::Ice'))
PY

mutation "spruce skirt loses its taper" gui foliage_tapers_from_wide_mid_crown_to_narrow_tip_and_skirt <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        0 => 0.72,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        0 => 1.0,\n'))
PY

mutation "ramps lose their caps" gui snow_caps_follow_material_and_exposure_in_a_seed_shaped_toy_world <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'pub fn has_snow_cap(mirror: &Mirror, position: [i32; 3]) -> bool {\n    matches!(\n        mirror.tile(position),\n        Some(Tile::Solid(material) | Tile::Ramp(material))'
assert s.count(old) == 1
new = 'pub fn has_snow_cap(mirror: &Mirror, position: [i32; 3]) -> bool {\n    matches!(\n        mirror.tile(position),\n        Some(Tile::Solid(material))'
p.write_text(s.replace(old, new))
PY

mutation "snow cap paints stone flanks" gui capped_stone_keeps_its_bare_cube_beneath_a_snow_cap <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        let level = rim_level(position, mirror.dims());\n'
new = '        let level = rim_level(position, mirror.dims());\n        if has_snow_cap(mirror, position) {\n            return self.slot(TerrainSlot::Snow, level);\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, new))
PY

mutation "torch table goes cold" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '            color: Color::srgb_u8(255, 140, 62),\n'
assert s.count(old) == 2
p.write_text(s.replace(old, '            color: Color::srgb_u8(62, 140, 255),\n', 1))
PY

mutation "light budget collapses" gui campfire_keeps_local_contrast_over_the_midtone_cold_fill <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '            intensity: 32_000_000.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            intensity: 5_000.0,\n'))
PY

mutation "night lighting goes unpinned" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '        sky: Color::srgb_u8(5, 12, 28),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        sky: Color::srgb_u8(20, 20, 20),\n'))
PY

mutation "snow cap matches snow terrain" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '    Color::srgb_u8(158, 170, 196)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    Color::srgb_u8(136, 150, 178)\n'))
PY

mutation "aurora climbs overhead" gui the_aurora_curtain_hugs_the_horizon_beyond_the_world <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = 'pub const AURORA_TOP: f32 = 45.0;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const AURORA_TOP: f32 = 140.0;'))
PY

mutation "aurora curtain regains a hard edge" gui the_aurora_gradient_fades_to_nothing_at_both_edges <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '            let edges = (v * std::f32::consts::PI).sin();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let edges = 1.0;\n'))
PY

mutation "aurora ring collapses off the world" gui the_curtain_mesh_is_a_closed_ring_at_the_aurora_radius <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '        let x = SKY_CENTRE.x + AURORA_RADIUS * angle.cos();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let x = SKY_CENTRE.x + AURORA_RADIUS * 0.5 * angle.cos();\n'))
PY

mutation "stars collapse to a single point" gui the_star_shell_fills_the_visible_sky_wedge <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '        let azimuth = index as f32 * STAR_AZIMUTH_STEP * std::f32::consts::TAU;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let azimuth = 3.9_f32;\n'))
PY

mutation "stars become one uniform size" gui star_sizes_vary_so_the_shell_never_reads_as_a_lattice <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = 'STAR_SCALE_MIN + (STAR_SCALE_MAX - STAR_SCALE_MIN) * (index as f32 * 0.381_966 + 0.21).fract()'
assert s.count(old) == 1
p.write_text(s.replace(old, 'STAR_SCALE_MIN'))
PY

mutation "boot pitch loses the approved framing" gui boot_composition_places_the_camp_low_and_the_skyline_at_the_top_third <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = 'const BOOT_PITCH: f32 = 0.45;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const BOOT_PITCH: f32 = 0.8;\n'))
PY

mutation "close zoom loses the camp" gui zoom_limits_keep_the_camp_in_front_of_the_camera <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '        let composition_scale = (self.distance / BOOT_DISTANCE).min(1.0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let composition_scale = 1.0;\n'))
PY

mutation "snowfall collapses into one row" gui snowfall_scatters_through_the_camp_read_without_marching_in_rows <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '        let height = 11.0 + SNOWFLAKE_FALL_SPAN * (index as f32 * FLAKE_HEIGHT_STEP).fract();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let height = 20.0;\n'))
PY

mutation "snowfall marches at one speed" gui snowfall_scatters_through_the_camp_read_without_marching_in_rows <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '    0.7 + 0.9 * (index as f32 * 0.618_034).fract()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    1.2\n'))
PY

mutation "stars fall back onto the helix" gui stars_scatter_instead_of_lying_on_a_helix <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = 'const STAR_HEIGHT_STEP: f32 = 0.569_840_3;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const STAR_HEIGHT_STEP: f32 = 0.754_877_7;'))
PY

mutation "atmosphere loses client local marker" gui atmosphere_entities_are_client_local_and_never_world_projected <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '            ClientLocal,\n'
assert s.count(old) == 2
p.write_text(s.replace(old, '', 1))
PY

mutation "snow caps lose client local marker" gui capped_stone_keeps_its_bare_cube_beneath_a_snow_cap <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        SnowCap(position),\n        ClientLocal,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        SnowCap(position),\n'))
PY

mutation "fog stops following zoom" gui fog_range_tracks_the_camera_without_erasing_the_far_edge <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        155.0_f32.max(camera_distance * 1.7),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        155.0,\n'))
PY

mutation "capture keeps its frame graph" gui capture_forces_the_frame_time_overlay_off <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'fn force_capture_overlay_off(app: &mut App) {\n    let mut config = app.world_mut().resource_mut::<FpsOverlayConfig>();\n    config.enabled = false;\n    config.frame_time_graph_config.enabled = false;\n}\n'
new = 'fn force_capture_overlay_off(app: &mut App) {\n    let mut config = app.world_mut().resource_mut::<FpsOverlayConfig>();\n    config.enabled = false;\n}\n'
assert s.count(old) == 1
p.write_text(s.replace(old, new))
PY

mutation "emitters ignore wire light field" gui recorded_camp_snapshot_projects_exactly_five_warm_point_lights <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                    if let Some(light) = mirror_entity.light {\n                        entity.insert(point_light(light));\n                    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    let _ = mirror_entity.light;\n'))
PY

mutation "boot composition drifts sideways again" gui boot_composition_never_pushes_along_the_camera_right_vector <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '    boot_horizontal_forward() * BOOT_COMPOSITION_FORWARD + Vec3::Y * BOOT_COMPOSITION_LIFT\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    Vec3::new(0.0, BOOT_COMPOSITION_LIFT, -37.42)\n'))
PY

mutation "world edge stops dissolving" gui the_world_edge_dissolves_inward_and_leaves_the_interior_alone <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    if to_edge >= RIM_WIDTH {\n        return 0;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if true {\n        return 0;\n    }\n'))
PY

mutation "rim dissolve never reaches the sky" gui the_rim_dissolve_runs_from_the_untouched_material_to_the_bare_sky <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '    let blend = (level.min(RIM_LEVELS - 1) as f32 / steps).clamp(0.0, 1.0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let blend = (level.min(RIM_LEVELS - 1) as f32 / steps).clamp(0.0, 0.5);\n'))
PY

mutation "spruce crowns stop catching snow" gui only_the_exposed_crown_of_a_spruce_catches_snow_light <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    terrain_material_at(mirror, position) == Some(Material::TreeFoliage)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    false && terrain_material_at(mirror, position) == Some(Material::TreeFoliage)\n'))
PY

mutation "crown colour matches bare foliage" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = 'pub fn foliage_snow_color() -> Color {\n    Color::srgb_u8(172, 186, 210)\n}'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub fn foliage_snow_color() -> Color {\n    Color::srgb_u8(55, 73, 84)\n}'))
PY

mutation "light budget slides back to the dark table" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '        ambient_brightness: 6_000.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        ambient_brightness: 30_000.0,\n'))
PY

mutation "campfire blows the camp to white" gui campfire_keeps_local_contrast_over_the_midtone_cold_fill <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '            intensity: 32_000_000.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            intensity: 7_200_000_000.0,\n'))
PY

mutation "cold fill turns warm" gui the_cold_fill_is_chromatically_cold_and_the_camp_is_chromatically_warm <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '        ambient: Color::srgb_u8(120, 140, 165),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        ambient: Color::srgb_u8(165, 140, 120),\n'))
PY

mutation "ground value check reads the mean" gui the_ground_median_reads_the_valley_floor_and_ignores_the_sky <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '    samples.sort_unstable();\n    samples[samples.len() / 2]\n'
assert s.count(old) == 1
new = '    (samples.iter().map(|v| *v as u32).sum::<u32>() / samples.len() as u32) as u8\n'
p.write_text(s.replace(old, new))
PY

mutation "ground value floor drops to nothing" gui a_black_field_fails_the_value_floor_that_a_lit_one_passes <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'pub const GROUND_LUMINANCE_FLOOR: u8 = 70;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const GROUND_LUMINANCE_FLOOR: u8 = 1;'))
PY

mutation "aurora curtain loses client local marker" gui atmosphere_entities_are_client_local_and_never_world_projected <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '        Atmosphere,\n        ClientLocal,\n    ));'
assert s.count(old) == 1
p.write_text(s.replace(old, '        Atmosphere,\n    ));'))
PY

mutation "directional tint goes unpinned" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '        directional: Color::srgb_u8(150, 190, 180),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        directional: Color::srgb_u8(73, 157, 144),\n'))
PY

mutation "value ceiling stops binding" gui a_blown_out_field_fails_the_value_ceiling_that_a_midtone_one_passes <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'pub const GROUND_LUMINANCE_CEILING: u8 = 180;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const GROUND_LUMINANCE_CEILING: u8 = 255;'))
PY
