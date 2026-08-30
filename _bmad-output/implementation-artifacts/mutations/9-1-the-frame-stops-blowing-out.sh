# Story 9.1 sabotage table. Run alone: scripts/mutate.sh <this file>

mutation "campfire shadows return to Bevy's default" gui campfire_light_casts_shadows_and_is_not_rewritten_by_a_later_reconciliation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        shadow_maps_enabled: matches!(kind, protocol::LightKind::Campfire),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        shadow_maps_enabled: false,\n'))
PY

# RETARGETED 2026-08-29. This row named the blown-POOL assertion, which no longer ships: the
# assertion moved to near-white AREA when the pool was measured to have a threshold cliff that
# software-rendered frames land on. Left pointing at the old text the row would APPLY-FAIL and pin
# nothing, which is the stale-literal trap this project has been bitten by before.
mutation "near-white area ceiling assertion is deleted" gui blown_pool_range_failure_is_a_real_panic_not_a_successful_capture <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '''    assert!(
        near_white <= NEAR_WHITE_AREA_CEILING,
        "near-white area is {:.4}%, above the {:.4}% ceiling calibrated on boot7.png",
        near_white * 100.0,
        NEAR_WHITE_AREA_CEILING * 100.0
    );
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = near_white;\n'))
PY

mutation "blown-pool ceiling rises past today's frame" gui committed_bevy_vistas_show_the_blown_pool_that_ground_median_cannot_see <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'pub const BLOWN_POOL_FRACTION_CEILING: f32 = 0.006_651_476;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const BLOWN_POOL_FRACTION_CEILING: f32 = 0.010_000_000;\n'))
PY

mutation "blown-pool threshold no longer separates the Bevy frames" gui committed_bevy_vistas_show_the_blown_pool_that_ground_median_cannot_see <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'pub const BLOWN_POOL_LUMINANCE_THRESHOLD: u8 = 200;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const BLOWN_POOL_LUMINANCE_THRESHOLD: u8 = 255;\n'))
PY

mutation "capture reports after the blown-pool assertion" gui capture_range_report_is_emitted_before_a_blown_pool_panic <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '''    report(&format!(
        "capture range check: warm-lit pixels={warm} ground-median-luminance={ground} \\
         near-white-area={:.4}% blown-pool={:.4}% p99-luminance={p99:.1}",
        near_white * 100.0,
        blown_pool * 100.0
    ));
'''
assert s.count(old) == 1
new = '''    let report_line = format!(
        "capture range check: warm-lit pixels={warm} ground-median-luminance={ground} \\
         near-white-area={:.4}% blown-pool={:.4}% p99-luminance={p99:.1}",
        near_white * 100.0,
        blown_pool * 100.0
    );
'''
s = s.replace(old, new)
old_assert = '''    assert!(
        near_white <= NEAR_WHITE_AREA_CEILING,
        "near-white area is {:.4}%, above the {:.4}% ceiling calibrated on boot7.png",
        near_white * 100.0,
        NEAR_WHITE_AREA_CEILING * 100.0
    );
'''
assert s.count(old_assert) == 1
p.write_text(s.replace(old_assert, old_assert + '    report(&report_line);\n'))
PY

# The stable metric that replaced the pool as the assertion, added 2026-08-29 while closing AC13.
mutation "the near-white area ceiling rises past the rejected frame" gui committed_bevy_vistas_show_the_blown_pool_that_ground_median_cannot_see <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'pub const NEAR_WHITE_AREA_CEILING: f32 = 0.015_630_426;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const NEAR_WHITE_AREA_CEILING: f32 = 0.020_000_000;\n'))
PY

mutation "near-white area counts nothing" gui committed_bevy_vistas_show_the_blown_pool_that_ground_median_cannot_see <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        .filter(|pixel| luminance(**pixel) >= threshold as f32)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .filter(|pixel| luminance(**pixel) > 255.0)\n', 1))
PY
