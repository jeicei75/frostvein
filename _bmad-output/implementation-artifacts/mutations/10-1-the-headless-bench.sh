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
