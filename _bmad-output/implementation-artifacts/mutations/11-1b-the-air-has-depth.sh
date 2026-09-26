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
mutation "ambient occlusion takes bloom's key" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::AmbientOcclusion => Some(KeyCode::F7),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::AmbientOcclusion => Some(KeyCode::F6),\n'))
PY

mutation "bloom takes ambient occlusion's key" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::Bloom => Some(KeyCode::F6),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::Bloom => Some(KeyCode::F7),\n'))
PY

mutation "the effect readout stops recording changed state" gui effect_keys_toggle_the_live_camera_and_readout <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let text = lighting_readout(&toggles, &effects_off);\n    for mut readout in &mut readout {\n        *readout = Text::new(text.clone());\n    }\n'
assert s.count(old) == 1
new = '    let _ = (toggles, effects_off, readout);\n'
p.write_text(s.replace(old, new))
PY

mutation "--static-world never reaches the daemon" gui static_world_schedules_its_pause_at_a_chosen_tick_over_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            crate::command::pause_static_world,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "--static-world asks the daemon to run instead of pause" gui static_world_schedules_its_pause_at_a_chosen_tick_over_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '        speed: Speed::Paused,\n        at_tick: Some(STATIC_WORLD_PAUSE_TICK),\n    });'
assert s.count(old) == 1
new = '        speed: Speed::Normal,\n        at_tick: Some(STATIC_WORLD_PAUSE_TICK),\n    });'
p.write_text(s.replace(old, new))
PY

# --- Rows added by the 2026-09-19 code review, covering what its own patches introduced.

# RE-POINTED 2026-09-20 (#111 fix). The old row deleted the client-side ` || mirror.0.tick() <
# STATIC_WORLD_PAUSE_TICK` guard, i.e. it pinned "do not send the pause before its tick". That
# guard is gone ON PURPOSE: the command now names its tick and sending EARLY is correct, because
# it only has to arrive first. The defect that replaced it is sending an UNSCHEDULED pause, which
# the daemon applies on arrival -- latency-bound, and the whole of #111.
mutation "the --static-world pause does not name its tick" gui static_world_schedules_its_pause_at_a_chosen_tick_over_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '        at_tick: Some(STATIC_WORLD_PAUSE_TICK),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        at_tick: None,\n'))
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
old = '    if static_world.0 {\n        eprintln!("sim stays PAUSED: --static-world holds the world frozen for the whole run");\n        return;\n    }\n    paused.0 = !paused.0;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    paused.0 = !paused.0;\n'))
PY

mutation "a --static-world run never hands the daemon back" gui a_static_world_run_hands_the_daemon_back_at_normal <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '    pending.push(Command::SetSpeed {\n        speed: Speed::Normal,\n        at_tick: None,\n    });\n    eprintln!("sim RESUMED'
assert s.count(old) == 1
new = '    eprintln!("sim RESUMED'
p.write_text(s.replace(old, new))
PY

# RE-POINTED 2026-09-22. This row pinned two unconditional `remove`s inside `apply_effect`'s
# AO-off arm. 11.2 deleted that seam: depth of field and haze read the DEPTH prepass and declare
# nothing, so AO-off taking it left them sampling a buffer that no longer existed, and prepass
# ownership moved to `sync_prepasses`. The row's QUESTION is unchanged -- does "ao off" leave a
# pass running that nothing samples? -- so it now sabotages the pass AO still solely owns.
mutation "--fx-off ao leaves the normal prepass running" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if ambient_occlusion {\n        camera.insert(NormalPrepass);\n    } else {\n        camera.remove::<NormalPrepass>();\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    camera.insert(NormalPrepass);\n'))
PY

mutation "--fx-off bloom takes Hdr with it, destroying AC4 control" gui fx_off_reaches_the_live_camera_and_rejects_unknown_effects <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            camera.remove::<Bloom>();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            camera.remove_with_requires::<Bloom>();\n'))
PY

# The one row only PIXELS can kill. A preset swap leaves the `Bloom` component present, so every
# component-presence test still passes; only the rendered halo delta notices. This is what the
# AC4 guard buys that the old component assertion did not.
mutation "bloom is retuned to the OLD_SCHOOL preset" gui bloom_lifts_the_camp_halo_without_brightening_open_snow ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Bloom::default(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Bloom::OLD_SCHOOL,\n'))
PY
