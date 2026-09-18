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
old = '            Exposure { ev100: 9.7 },\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the --fx-off value is parsed but discarded before camera setup" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    app.insert_resource(FxaaOff(args.fx_off));\n'
assert s.count(old) == 1
new = '    let _ = args.fx_off;\n    app.insert_resource(FxaaOff(false));\n'
p.write_text(s.replace(old, new))
PY

mutation "the camera never receives FXAA" gui configured_camera_starts_with_fxaa <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Fxaa::default(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "F10 is no longer the FXAA key" gui f10_toggles_fxaa_and_the_live_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if !keys.just_pressed(KeyCode::F10) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if !keys.just_pressed(KeyCode::F11) {\n'))
PY

mutation "the FXAA readout no longer records its state" gui f10_toggles_fxaa_and_the_live_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        if fxaa_enabled { "on" } else { "off" }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        "on"\n'))
PY

mutation "the crease windows swap the two open-snow rectangles" py scripts.test_creases.CreasesTests.test_windows_remain_pinned_to_non_camp_rectangles <<'PY'
import pathlib
p = pathlib.Path('_bmad-output/implementation-artifacts/11-1-signoff/creases.py'); s = p.read_text()
old = '    "open-snow-LL": (180, 620, 380, 700),\n    "open-snow-LR": (950, 590, 1150, 670),\n'
assert s.count(old) == 1
new = '    "open-snow-LL": (950, 590, 1150, 670),\n    "open-snow-LR": (180, 620, 380, 700),\n'
p.write_text(s.replace(old, new))
PY
