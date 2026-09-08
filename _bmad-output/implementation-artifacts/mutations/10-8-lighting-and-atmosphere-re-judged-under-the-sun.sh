# Story 10.8 Task 0 and Task 6c sabotage table. Run alone: scripts/mutate.sh <this file>
# Later tasks append their own rows here; do not replace the evidence below.

mutation "restore the shipped terrain subdivision default to one" gui absent_subdiv_flag_installs_the_shipped_default_four <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'const DEFAULT_TERRAIN_SUBDIV: u32 = 4;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const DEFAULT_TERRAIN_SUBDIV: u32 = 1;'))
PY

mutation "make motion assertions ignore the captured slice again" gui motion_assertions_apply_only_when_a_dwarf_is_drawn_in_the_captured_slice <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '.any(|entity| entity.kind == EntityKind::Dwarf && entity.pos[2] <= slice_level)'
assert s.count(old) == 1
p.write_text(s.replace(old, '.any(|entity| entity.kind == EntityKind::Dwarf)'))
PY

mutation "validate a captured frame before its PNG is written" gui a_failed_range_check_leaves_the_capture_png_on_disk <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '''fn save_before_validate(save: impl FnOnce(), validate: impl FnOnce()) {
    save();
    validate();
}'''
assert s.count(old) == 1
new = '''fn save_before_validate(save: impl FnOnce(), validate: impl FnOnce()) {
    validate();
    save();
}'''
p.write_text(s.replace(old, new))
PY

mutation "demand motion below a dwarf-free captured slice again" gui a_capture_below_the_dwarves_skips_motion_but_still_writes_a_png ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '''            None if motion_assertions_apply(&mirror.0, slice.level()) => {
                capture.motion.assert_valid(capture.expect_work);
            }
            None => println!('''
assert s.count(old) == 1
new = '''            None => {
                capture.motion.assert_valid(capture.expect_work);
            }
            None => println!('''
p.write_text(s.replace(old, new))
PY

mutation "restore the shipped directional illuminance" gui appearance_tables_pin_the_cold_boot_palette <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = 'directional_illuminance: 7_000.0,'
assert s.count(old) == 1
p.write_text(s.replace(old, 'directional_illuminance: 22_000.0,'))
PY

mutation "diverge the bench ambient colour from the client" gui bench_literals_match_the_client_palette_lights_and_boot_camera <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/valley_bench.py'); s = p.read_text()
old = 'AMBIENT_RGB = (108, 128, 170)'
assert s.count(old) == 1
p.write_text(s.replace(old, 'AMBIENT_RGB = (120, 140, 165)'))
PY

mutation "restore the shipped two-cell torch spacing" sim-core generated_world_has_sorted_camp_emitters <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old_minus_x = 'x: camp.x - TORCH_RING_OFFSET,'
old_plus_x = 'x: camp.x + TORCH_RING_OFFSET,'
old_minus_y = 'y: camp.y - TORCH_RING_OFFSET,'
old_plus_y = 'y: camp.y + TORCH_RING_OFFSET,'
assert s.count(old_minus_x) == 2
assert s.count(old_plus_x) == 2
assert s.count(old_minus_y) == 2
assert s.count(old_plus_y) == 2
s = s.replace(old_minus_x, 'x: camp.x - 2,')
s = s.replace(old_plus_x, 'x: camp.x + 2,')
s = s.replace(old_minus_y, 'y: camp.y - 2,')
s = s.replace(old_plus_y, 'y: camp.y + 2,')
p.write_text(s)
PY

mutation "restore the boot7 near-white ceiling" gui committed_bevy_vistas_show_the_blown_pool_that_ground_median_cannot_see <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'pub const NEAR_WHITE_AREA_CEILING: f32 = 0.009_460_72;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const NEAR_WHITE_AREA_CEILING: f32 = 0.015_630_426;'))
PY
