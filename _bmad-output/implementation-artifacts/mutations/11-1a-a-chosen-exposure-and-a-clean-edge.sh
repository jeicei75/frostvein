# Story 11.1a sabotage table. Run with scripts/mutate.sh <this file>.

mutation "the live camera keeps multisample antialiasing enabled" gui configured_camera_disables_msaa_on_the_live_rig <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Msaa::Off,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the live camera discards its chosen exposure" gui configured_camera_carries_the_chosen_ev100_on_the_live_rig <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Exposure { ev100: 10.5 },\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the --fx-off value is parsed but discarded before camera setup" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    app.insert_resource(EffectsOff::with_off(&args.fx_off));\n'
assert s.count(old) == 1
new = '    let _ = args.fx_off;\n    app.insert_resource(EffectsOff::default());\n'
p.write_text(s.replace(old, new))
PY

mutation "the camera never receives FXAA" gui configured_camera_starts_with_fxaa <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Fxaa::default(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "F10 is no longer the FXAA key" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::Fxaa => KeyCode::F10,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::Fxaa => KeyCode::F13,\n'))
PY

mutation "the FXAA readout no longer records its state" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::Fxaa => "fxaa",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::Fxaa => "post",\n'))
PY

# The rect MOVES ONTO THE CAMP, it does not trade places with its sibling.  The first version of
# this row swapped the two open-snow rects BETWEEN THEIR LABELS, which left the SET of measured
# rectangles identical -- every statistic creases.py computes was unchanged and no figure in any
# record would have moved, so the row could only ever be killed by a test restating the same
# literals.  (500,400,700,500) is the camp window AC6 forbids measuring in, because its mean moves
# 1.19 between same-build runs; a window there makes the instrument report flicker as signal.
mutation "a crease window moves onto the camp AC6 forbids measuring in" py scripts.tests.test_creases.CreasesTests.test_windows_remain_pinned_to_non_camp_rectangles <<'PY'
import pathlib
p = pathlib.Path('_bmad-output/implementation-artifacts/11-1-signoff/creases.py'); s = p.read_text()
old = '    "terrace-creases": (860, 190, 1060, 290),\n'
assert s.count(old) == 1
new = '    "terrace-creases": (500, 400, 700, 500),\n'
p.write_text(s.replace(old, new))
PY
