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
old = '    fn terrain_material(&self, mirror: &Mirror, position: [i32; 3]) -> Handle<StandardMaterial> {\n        let material = terrain_material(mirror, position);\n'
new = '    fn terrain_material(&self, mirror: &Mirror, position: [i32; 3]) -> Handle<StandardMaterial> {\n        if has_snow_cap(mirror, position) {\n            return self.snow.clone();\n        }\n        let material = terrain_material(mirror, position);\n'
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
old = '            intensity: 6_000_000.0,\n'
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

mutation "aurora leaves the boot frustum" gui atmosphere_positions_stay_outside_the_terrain_and_inside_the_boot_frustum <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = 'Vec3::new(64.0, 26.1, -130.0)'
assert s.count(old) == 1
p.write_text(s.replace(old, 'Vec3::new(64.0, 35.0, -130.0)'))
PY

mutation "snowfall collapses back to a diagonal" gui snowfall_fills_a_visible_grid_instead_of_a_single_diagonal_row <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '            -82.0 + (index / 6) as f32 * 7.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            -82.0 + (index % 6) as f32 * 7.0,\n'))
PY

mutation "atmosphere loses client local marker" gui atmosphere_entities_are_client_local_and_never_world_projected <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '            ClientLocal,\n'
assert s.count(old) == 3
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
old = '        190.0_f32.max(camera_distance * 1.8),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        190.0,\n'))
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
