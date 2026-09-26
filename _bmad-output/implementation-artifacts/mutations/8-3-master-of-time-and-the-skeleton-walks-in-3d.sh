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
old = '.filter(|item| zones.contains(item))'
assert s.count(old) == 1
p.write_text(s.replace(old, '.filter(|_| true)'))
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
