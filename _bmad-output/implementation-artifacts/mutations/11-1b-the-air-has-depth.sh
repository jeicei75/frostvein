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

# NOTE: these two rows SWAP the keys rather than moving one to F13. The F13 version was a WEAK
# KILL found by the 2026-09-19 code review: F13 is not an arm of `lighting_readout`'s key->string
# match, so the mutant died on its `unreachable!()` inside the readout formatter BEFORE any
# key-press assertion could discriminate. Both rows reported KILLED and would have reported KILLED
# even if `effect_controls` ignored the keyboard entirely. A swap stays inside the match, panics
# nothing, and makes the test notice that F11 now toggles bloom.
mutation "F11 toggles bloom instead of ambient occlusion" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::AmbientOcclusion => KeyCode::F11,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::AmbientOcclusion => KeyCode::F12,\n'))
PY

mutation "F12 toggles ambient occlusion instead of bloom" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::Bloom => KeyCode::F12,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::Bloom => KeyCode::F11,\n'))
PY

mutation "the effect readout stops recording changed state" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let text = lighting_readout(&toggles, &effects_off);\n    for mut readout in &mut readout {\n        *readout = Text::new(text.clone());\n    }\n'
assert s.count(old) == 1
new = '    let _ = (toggles, effects_off, readout);\n'
p.write_text(s.replace(old, new))
PY

mutation "--static-world never reaches the daemon" gui static_world_waits_for_its_tick_then_pauses_the_daemon_over_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            crate::command::pause_static_world,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "--static-world asks the daemon to run instead of pause" gui static_world_waits_for_its_tick_then_pauses_the_daemon_over_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '        speed: Speed::Paused,\n    });\n    eprintln!(\n        "sim PAUSE REQUESTED'
assert s.count(old) == 1
new = '        speed: Speed::Normal,\n    });\n    eprintln!(\n        "sim PAUSE REQUESTED'
p.write_text(s.replace(old, new))
PY

# --- Rows added by the 2026-09-19 code review, covering what its own patches introduced.

mutation "the --static-world pause is sent before its chosen tick" gui static_world_waits_for_its_tick_then_pauses_the_daemon_over_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = ' || mirror.0.tick() < STATIC_WORLD_PAUSE_TICK'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the pause is believed from our own request, not the daemon report" gui static_world_confirms_the_pause_from_the_daemons_own_report <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '    if mirror.0.speed() != Speed::Paused {'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false {'))
PY

mutation "Space resumes a --static-world run" gui space_cannot_resume_a_static_world_run <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '    if static_world.0 {\n        eprintln!("sim stays PAUSED: --static-world holds the world frozen for the whole run");\n        return;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "a --static-world run never hands the daemon back" gui a_static_world_run_hands_the_daemon_back_at_normal <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '    pending.push(Command::SetSpeed {\n        speed: Speed::Normal,\n    });\n    eprintln!("sim RESUMED'
assert s.count(old) == 1
new = '    eprintln!("sim RESUMED'
p.write_text(s.replace(old, new))
PY

mutation "--fx-off ao leaves AO required prepasses running" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            camera.remove::<DepthPrepass>();\n            camera.remove::<NormalPrepass>();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "--fx-off bloom takes Hdr with it, destroying AC4 control" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            camera.remove::<Bloom>();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            camera.remove_with_requires::<Bloom>();\n'))
PY
