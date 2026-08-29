mutation "palette drift fails the bench contract" gui bench_literals_match_the_client_palette_lights_and_boot_camera <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = 'Material::Stone => Color::srgb_u8(60, 70, 92)'
assert s.count(old) == 1
p.write_text(s.replace(old, 'Material::Stone => Color::srgb_u8(61, 70, 92)'))
PY

mutation "boot camera drift fails the bench contract" gui bench_literals_match_the_client_palette_lights_and_boot_camera <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = 'const BOOT_YAW: f32 = 0.7;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const BOOT_YAW: f32 = 0.8;'))
PY

mutation "missing range assertion fails empty-export test" py scripts.tests.test_valley_bench.ValleyBlenderTests.test_empty_export_exits_nonzero <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '        assert_range(check)'
assert s.count(old) == 1
p.write_text(s.replace(old, '        pass'))
PY

mutation "inverted neighbour predicate fails geometry test" py scripts.tests.test_valley_bench.ValleyGeometryTests.test_exposed_faces_use_six_orthogonal_neighbours_and_world_edges <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '                    if not is_solid(tile_at(snapshot, x + nx, y + ny, z + nz)):'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    if True:'))
PY

mutation "zero exit fails empty-export test" py scripts.tests.test_valley_bench.ValleyBlenderTests.test_empty_export_exits_nonzero <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '        raise SystemExit(f"range check failed: {error}") from error'
assert s.count(old) == 1
p.write_text(s.replace(old, '        return'))
PY

mutation "z-up camera basis fails the framing test" py scripts.tests.test_valley_bench.ValleyFramingTests.test_boot_projection_matches_the_client_composition <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '    right = vector_normalize(vector_cross((0.0, 1.0, 0.0), back))'
assert s.count(old) == 1
p.write_text(s.replace(old, '    right = vector_normalize(vector_cross((0.0, 0.0, 1.0), back))'))
PY

mutation "linear sky reference fails the all-sky render test" py scripts.tests.test_valley_bench.ValleyBlenderTests.test_all_sky_frame_reads_as_sky_in_a_real_render <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '    sky = tuple(component / 255.0 for component in SKY_RGB)'
assert s.count(old) == 1
p.write_text(s.replace(old, '    sky = srgb_to_linear(SKY_RGB)'))
PY

# --- Rows added by the 2026-08-29 code review -------------------------------------------------
# Each closes a guard the review proved could not fire.

mutation "unwired ambient fails the lit-render test" py scripts.tests.test_valley_bench.ValleyBlenderTests.test_a_populated_render_is_lit_not_merely_non_black <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = 'AMBIENT_STRENGTH = 3.3'
assert s.count(old) == 1
p.write_text(s.replace(old, 'AMBIENT_STRENGTH = 0.0'))
PY

mutation "dwarf colour drift fails the bench contract" gui bench_literals_match_the_client_palette_lights_and_boot_camera <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = 'Color::srgb_u8(151, 116, 96)'
assert s.count(old) == 1
p.write_text(s.replace(old, 'Color::srgb_u8(152, 116, 96)'))
PY

mutation "torch and campfire light colours swapped fails the bench contract" gui bench_literals_match_the_client_palette_lights_and_boot_camera <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = 'LightKind::Torch => LightProperties {\n            color: Color::srgb_u8(255, 140, 62),'
assert s.count(old) == 1
p.write_text(s.replace(old, 'LightKind::Torch => LightProperties {\n            color: Color::srgb_u8(255, 173, 92),'))
PY

mutation "full-size foliage fails the crown-scale test" py scripts.tests.test_valley_bench.ValleyGeometryTests.test_foliage_is_drawn_smaller_than_its_cell <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '        scale = foliage_scale(snapshot, x, y, z)'
assert s.count(old) == 1
p.write_text(s.replace(old, '        scale = 1.0'))
PY

mutation "re-copied FOV literal fails the single-source projection test" py scripts.tests.test_valley_bench.ValleyFramingTests.test_projection_reads_the_shared_fov_and_aspect_constants <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '    half_vertical = math.tan(BOOT_VERTICAL_FOV * 0.5)'
assert s.count(old) == 1
p.write_text(s.replace(old, '    half_vertical = math.tan((math.pi / 4) * 0.5)'))
PY

mutation "hand-picked sun aim fails the client-aim test" py scripts.tests.test_valley_bench.ValleyFramingTests.test_sun_is_aimed_the_way_the_client_aims_it <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '    return vector_normalize(vector_subtract(CAMP_FOCUS, aurora_core()))'
assert s.count(old) == 1
p.write_text(s.replace(old, '    return (0.044, -0.637, -0.770)'))
PY

mutation "swallowed exception fails the broken-export test" py scripts.tests.test_valley_bench.ValleyBlenderTests.test_a_broken_export_exits_nonzero_instead_of_reporting_success <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '        raise SystemExit(f"bench failed: {type(error).__name__}: {error}") from error'
assert s.count(old) == 1
p.write_text(s.replace(old, '        pass'))
PY
