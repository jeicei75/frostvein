# Story 11.1b sabotage table. Run with scripts/mutate.sh <this file>.

mutation "the strengthened --lights-steady consequence is discarded before the live flicker system" gui lights_steady_reaches_the_live_flicker_system <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    app.insert_resource(LightsSteady(args.lights_steady));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = args.lights_steady;\n    app.insert_resource(LightsSteady(false));\n'))
PY

mutation "ambient occlusion is omitted from the live camera" gui ambient_occlusion_darkens_terrace_creases_and_msaa_cannot_silently_disable_it ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            ScreenSpaceAmbientOcclusion::default(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "MSAA is re-enabled while ambient occlusion is present" gui ambient_occlusion_darkens_terrace_creases_and_msaa_cannot_silently_disable_it ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Msaa::Off,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Msaa::Sample4,\n'))
PY

mutation "bloom is omitted from the live camera" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Bloom::default(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "--fx-off fxaa discards the named FXAA effect" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            "fxaa" => Ok(Self::Fxaa),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            "fxaa" => Ok(Self::AmbientOcclusion),\n'))
PY

mutation "--fx-off ao discards the named ambient-occlusion effect" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            "ao" => Ok(Self::AmbientOcclusion),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            "ao" => Ok(Self::Fxaa),\n'))
PY

mutation "--fx-off bloom discards the named bloom effect" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            "bloom" => Ok(Self::Bloom),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            "bloom" => Ok(Self::Fxaa),\n'))
PY

mutation "F11 is no longer the ambient-occlusion key" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::AmbientOcclusion => KeyCode::F11,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::AmbientOcclusion => KeyCode::F13,\n'))
PY

mutation "F12 is no longer the bloom key" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::Bloom => KeyCode::F12,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::Bloom => KeyCode::F13,\n'))
PY

mutation "the effect readout stops recording changed state" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let text = lighting_readout(&toggles, &effects_off);\n    for mut readout in &mut readout {\n        *readout = Text::new(text.clone());\n    }\n'
assert s.count(old) == 1
new = '    let _ = (toggles, effects_off, readout);\n'
p.write_text(s.replace(old, new))
PY
