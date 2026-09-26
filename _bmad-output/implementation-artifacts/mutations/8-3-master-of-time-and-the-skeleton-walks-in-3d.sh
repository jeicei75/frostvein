# Story 8.3: gui save/load chords, the Ctrl camera gate, the hint, and the --expect-haul instrument.

mutation "Ctrl+S never saves" gui ctrl_s_saves_and_ctrl_l_loads_on_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '        pending.push(Command::Save);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "Ctrl+L never loads" gui ctrl_s_saves_and_ctrl_l_loads_on_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '            pending.push(Command::Load);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "bare letters save and load without Ctrl" gui ctrl_s_saves_and_ctrl_l_loads_on_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '    if !keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {\n        return;\n    }\n    if keys.just_pressed(KeyCode::KeyS) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if keys.just_pressed(KeyCode::KeyS) {\n'))
PY

mutation "the save/load system is never registered" gui ctrl_s_saves_and_ctrl_l_loads_on_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            crate::command::save_load_keys.before(send_commands),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "a load goes through under --static-world" gui ctrl_l_cannot_load_over_a_static_world_run <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '        if static_world.0 {\n            eprintln!("sim NOT LOADED'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false {\n            eprintln!("sim NOT LOADED'))
PY

mutation "a Ctrl chord still moves the camera" gui ctrl_s_saves_and_ctrl_l_loads_on_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let key_scale = if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {\n        0.0\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let key_scale = if false {\n        0.0\n'))
PY

mutation "the hint drops the time and save keys" gui the_production_wiring_runs_every_call_run_makes_after_its_plugins <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '"1 dig  2 channel  3 stockpile  4 clear   Space pause  +/- speed  Ctrl+S save  Ctrl+L load"'
assert s.count(old) == 1
p.write_text(s.replace(old, '"1 dig  2 channel  3 stockpile  4 clear"'))
PY

mutation "the stockpile count ignores the stockpile" gui the_haul_instrument_counts_stones_on_stockpile_tiles <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '.filter(|item| zones.contains(item) && !dwarves.contains(item))'
assert s.count(old) == 1
p.write_text(s.replace(old, '.filter(|item| !dwarves.contains(item))'))
PY

mutation "the haul assertion never fires in a capture" gui expect_haul_fails_a_world_with_no_stone_on_a_stockpile ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        if capture.expect_haul {\n            capture.motion.assert_haul();\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "--expect-haul parses but never reaches the capture" gui expect_haul_fails_a_world_with_no_stone_on_a_stockpile ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        .with_expect_haul(args.expect_haul);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ';\n'))
PY

mutation "--expect-haul needs no capture" gui expect_haul_parses_only_with_a_capture <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if expect_haul && capture.is_none() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false && expect_haul && capture.is_none() {\n'))
PY

mutation "the clock readout never follows the wire" gui the_live_clock_readout_follows_the_daemons_tick_and_speed <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        .add_systems(Update, update_clock_readout.after(ProjectionSet));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ';\n'))
PY

mutation "elapsed forgets whole days" gui the_clock_readout_names_the_hour_the_elapsed_sim_time_and_the_speed <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let elapsed = tick * 60 / crate::clock::TICKS_PER_HOUR as u64;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let elapsed = (tick % crate::clock::TICKS_PER_DAY) * 60 / crate::clock::TICKS_PER_HOUR as u64;\n'))
PY

mutation "the readout names the wrong speed" gui the_clock_readout_names_the_hour_the_elapsed_sim_time_and_the_speed <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        protocol::Speed::Fast4x => "fast4x",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        protocol::Speed::Fast4x => "fast",\n'))
PY

mutation "H hides nothing" gui h_hides_and_shows_every_hud_text <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        *visibility = if *hidden {\n            bevy::prelude::Visibility::Hidden\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        *visibility = if *hidden {\n            bevy::prelude::Visibility::Inherited\n'))
PY

mutation "a second H cannot bring the HUD back" gui h_hides_and_shows_every_hud_text <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    *hidden = !*hidden;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    *hidden = true;\n'))
PY

mutation "H leaves the fps overlay on" gui h_hides_and_shows_every_hud_text <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        overlay.enabled = !*hidden;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the hint escapes the HUD toggle" gui h_hides_and_shows_every_hud_text <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '        crate::ingest::Hud,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the HUD toggle is never registered" gui h_hides_and_shows_every_hud_text <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            toggle_hud,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "a stone in a hauler's hands counts as delivered" gui the_haul_instrument_counts_stones_on_stockpile_tiles <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '.filter(|item| zones.contains(item) && !dwarves.contains(item))'
assert s.count(old) == 1
p.write_text(s.replace(old, '.filter(|item| zones.contains(item))'))
PY

mutation "a capture keeps the HUD" gui a_capture_hides_the_hud_and_h_cannot_restore_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        app.add_systems(bevy::app::PostStartup, hide_hud_for_capture);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "H still toggles during a capture" gui a_capture_hides_the_hud_and_h_cannot_restore_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if capture.is_some() || !keys.just_pressed(KeyCode::KeyH) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if !keys.just_pressed(KeyCode::KeyH) {\n'))
PY

mutation "the hour truncates a minute short" gui the_clock_readout_names_the_hour_the_elapsed_sim_time_and_the_speed <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let minute_of_day = (hour * 60.0 + 0.001) as u64;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let minute_of_day = (hour * 60.0) as u64;\n'))
PY
