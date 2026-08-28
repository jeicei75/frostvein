# Story 9.1 sabotage table. Run alone: scripts/mutate.sh <this file>

mutation "campfire shadows return to Bevy's default" gui campfire_light_casts_shadows_and_is_not_rewritten_by_a_later_reconciliation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        shadow_maps_enabled: matches!(kind, protocol::LightKind::Campfire),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        shadow_maps_enabled: false,\n'))
PY

mutation "blown-pool ceiling assertion is deleted" gui blown_pool_range_failure_is_a_real_panic_not_a_successful_capture <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '''    assert!(
        blown_pool <= BLOWN_POOL_FRACTION_CEILING,
        "the largest near-white pool is {:.4}%, above the {:.4}% ceiling calibrated on boot7.png",
        blown_pool * 100.0,
        BLOWN_POOL_FRACTION_CEILING * 100.0
    );
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = blown_pool;\n'))
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
         blown-pool={:.4}% p99-luminance={p99:.1}",
        blown_pool * 100.0
    ));
'''
assert s.count(old) == 1
new = '''    let report_line = format!(
        "capture range check: warm-lit pixels={warm} ground-median-luminance={ground} \\
         blown-pool={:.4}% p99-luminance={p99:.1}",
        blown_pool * 100.0
    );
'''
s = s.replace(old, new)
old_assert = '''    assert!(
        blown_pool <= BLOWN_POOL_FRACTION_CEILING,
        "the largest near-white pool is {:.4}%, above the {:.4}% ceiling calibrated on boot7.png",
        blown_pool * 100.0,
        BLOWN_POOL_FRACTION_CEILING * 100.0
    );
'''
assert s.count(old_assert) == 1
p.write_text(s.replace(old_assert, old_assert + '    report(&report_line);\n'))
PY
