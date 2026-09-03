# Story 10.7 Task 1 sabotage table. Run alone: scripts/mutate.sh <this file>

mutation "restore the shipped aurora_core aim" gui the_approved_sun_lights_downward <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '''pub fn sun_direction() -> Vec3 {
    let azimuth = SUN_AZIMUTH_DEGREES.to_radians();
    let elevation = SUN_ELEVATION_DEGREES.to_radians();
    let horizontal = elevation.cos();
    Vec3::new(
        azimuth.cos() * horizontal,
        -elevation.sin(),
        azimuth.sin() * horizontal,
    )
}
'''
assert s.count(old) == 1
new = '''pub fn sun_direction() -> Vec3 {
    (CAMP_FOCUS - aurora_core()).normalize()
}
'''
p.write_text(s.replace(old, new))
PY

mutation "bench sun elevation diverges from the client" gui bench_literals_match_the_client_palette_lights_and_boot_camera <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
lines = s.splitlines(keepends=True)
matches = [index for index, line in enumerate(lines) if line.startswith('SUN_ELEVATION_DEGREES = ')]
assert len(matches) == 1
lines[matches[0]] = 'SUN_ELEVATION_DEGREES = -SUN_AZIMUTH_DEGREES\n'
p.write_text(''.join(lines))
PY

mutation "flip the sun direction formula's elevation sign" gui the_approved_sun_lights_downward <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '        -elevation.sin(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        elevation.sin(),\n'))
PY

mutation "make every lighting toggle inert after it flips" gui lighting_keys_change_the_live_scene_and_its_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '''        light.brightness = if toggles.enabled(LightSource::Ambient) {
            night_lighting().ambient_brightness
        } else {
            0.0
        };
    }
    for mut light in &mut sun {
        light.illuminance = if toggles.enabled(LightSource::Sun) {
            night_lighting().directional_illuminance
        } else {
            0.0
        };
'''
assert s.count(old) == 1
new = '''        light.brightness = night_lighting().ambient_brightness;
    }
    for mut light in &mut sun {
        light.illuminance = night_lighting().directional_illuminance;
'''
s = s.replace(old, new)
# RE-ANCHORED 2026-09-03: the point-light branch moved to a named predicate when torches got
# their own toggle. Same seam, same sabotage — every source flips its flag and nothing dims.
old = '''        if !point_light_enabled(&toggles, kind.0) {
            light.intensity = 0.0;
        }
'''
assert s.count(old) == 1
s = s.replace(old, '        let _ = (kind, &mut light);\n')
p.write_text(s)
PY

mutation "a mesh-drawn tree hides a terrain face again" gui a_mesh_drawn_tree_hides_no_terrain_face <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'if !occludes_terrain(mirror, above, level, cover) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if !occludes(mirror, above, level) {'))
PY

mutation "the stone control stops hiding its face, so the test stops discriminating" gui a_mesh_drawn_tree_hides_no_terrain_face <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    occludes(mirror, position, level) && !is_mesh_drawn_tree(mirror, position, cover)'
assert s.count(old) == 1
p.write_text(s.replace(old, '    occludes(mirror, position, level) && false'))
PY

mutation "a light toggle flips its flag but changes nothing drawn" gui lighting_keys_change_the_live_scene_and_its_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    app.init_resource::<LightingToggles>();\n    app.add_systems(\n        Update,\n        (apply_lighting_toggles, update_lighting_readout)'
assert s.count(old) == 1
new = '    app.init_resource::<LightingToggles>();\n    app.add_systems(\n        Update,\n        (update_lighting_readout, update_lighting_readout)'
p.write_text(s.replace(old, new))
PY

# --- Added at the 2026-09-03 code review. Each pins a guard the review found missing. ---

mutation "the installed sun is aimed at nothing" gui the_installed_sun_entity_aims_downward_onto_the_valley <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        sun_light_transform(),\n        SunLight,'
assert s.count(old) == 1
p.write_text(s.replace(old, '        Transform::default(),\n        SunLight,'))
PY

mutation "a mesh-drawn tree hides the face BELOW it again" gui a_mesh_drawn_tree_hides_neither_the_face_below_it_nor_the_face_beside_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'if !occludes_terrain(mirror, under, level, cover) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if !occludes(mirror, under, level) {'))
PY

mutation "a mesh-drawn tree hides the face BESIDE it again" gui a_mesh_drawn_tree_hides_neither_the_face_below_it_nor_the_face_beside_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'let solid = occludes_terrain(mirror, neighbour, level, cover);'
assert s.count(old) == 1
p.write_text(s.replace(old, 'let solid = occludes(mirror, neighbour, level);'))
PY

mutation "the bench sun formula diverges while both constants stay identical" gui bench_literals_match_the_client_palette_lights_and_boot_camera <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '        -math.sin(elevation),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        math.sin(elevation),\n'))
PY

mutation "an unknown light source is accepted instead of refused" gui lights_off_parses_refuses_an_unknown_source_and_reaches_the_toggles_the_keys_drive <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            for name in value.to_string_lossy().split(\',\') {\n                lights_off.push(LightSource::from_name(name.trim())?);\n            }'
assert s.count(old) == 1
new = '            for name in value.to_string_lossy().split(\',\') {\n                if let Ok(source) = LightSource::from_name(name.trim()) {\n                    lights_off.push(source);\n                }\n            }'
p.write_text(s.replace(old, new))
PY
