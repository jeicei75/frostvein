# Story 11.1b sabotage table. Run with scripts/mutate.sh <this file>.

mutation "the strengthened --lights-steady consequence is discarded before the live flicker system" gui lights_steady_reaches_the_live_flicker_system <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    app.insert_resource(LightsSteady(args.lights_steady));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = args.lights_steady;\n    app.insert_resource(LightsSteady(false));\n'))
PY

mutation "ambient occlusion is omitted from the live camera" gui ambient_occlusion_darkens_terrace_creases_and_msaa_cannot_silently_disable_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            ScreenSpaceAmbientOcclusion::default(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "MSAA is re-enabled while ambient occlusion is present" gui ambient_occlusion_darkens_terrace_creases_and_msaa_cannot_silently_disable_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Msaa::Off,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Msaa::Sample4,\n'))
PY
