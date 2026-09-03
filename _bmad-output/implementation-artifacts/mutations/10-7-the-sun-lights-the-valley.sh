# Story 10.7 Task 1 sabotage table. Run alone: scripts/mutate.sh <this file>

mutation "sun direction returns to the aurora-to-camp aim" gui the_aurora_curtain_hugs_the_horizon_beyond_the_world <<'PY'
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

mutation "bench sun travels downward at the shipped elevation" py scripts.tests.test_valley_bench.ValleyFramingTests.test_sun_is_aimed_the_way_the_client_aims_it <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = '        -math.sin(elevation),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        math.sin(elevation),\n'))
PY
