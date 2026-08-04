# Mutation set for story 2.3. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh

mutation "pause no longer skips world.step" simd paused_daemon_freezes_tick_and_entities_then_resumes <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = """        if speed != protocol::Speed::Paused {
            world.step();
        }
"""
assert old in s
p.write_text(s.replace(old, "        world.step();\n"))
PY

mutation "parsed speed command is dropped before the loop" simd speed_change_from_either_client_reaches_both_on_the_same_delta <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = """                    Ok(command) => {
                        if command_tx.send(command).is_err() {
                            break;
                        }
                    }
"""
assert old in s
p.write_text(s.replace(old, "                    Ok(_command) => {}\n"))
PY

mutation "fast period equals normal period" simd fast_deltas_arrive_in_under_half_the_normal_span <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = "const FAST_TICK_PERIOD: Duration = Duration::from_millis(20);\n"
assert old in s
p.write_text(s.replace(old, "const FAST_TICK_PERIOD: Duration = Duration::from_millis(100);\n"))
PY

mutation "paused loop uses the fast period" simd paused_daemon_freezes_tick_and_entities_then_resumes <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = "        protocol::Speed::Paused | protocol::Speed::Normal => TICK_PERIOD,\n"
assert old in s
p.write_text(s.replace(old, "        protocol::Speed::Paused => FAST_TICK_PERIOD,\n        protocol::Speed::Normal => TICK_PERIOD,\n"))
PY

mutation "snapshot bridge hardcodes normal speed" simd client_connecting_while_paused_receives_paused_snapshot <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = "        zones: Vec::new(),\n        speed,\n        tick: world.tick(),\n"
assert old in s
p.write_text(s.replace(old, "        zones: Vec::new(),\n        speed: protocol::Speed::Normal,\n        tick: world.tick(),\n"))
PY

mutation "delta bridge hardcodes normal speed" simd delta_carries_dirty_tiles_and_full_authoritative_state <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = "        zones: Vec::new(),\n        speed,\n    }\n"
assert old in s
p.write_text(s.replace(old, "        zones: Vec::new(),\n        speed: protocol::Speed::Normal,\n    }\n"))
PY

mutation "plus at fast wraps to paused" tui speed_keys_follow_the_pinned_step_table_and_clamp <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "            Speed::Fast => Action::Ignore,\n"
assert old in s
p.write_text(s.replace(old, "            Speed::Fast => command(Speed::Paused),\n", 1))
PY

mutation "space key is ignored" tui speed_keys_follow_the_pinned_step_table_and_clamp <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = """        KeyCode::Char(' ') => command(match speed {
            Speed::Paused => Speed::Normal,
            Speed::Normal | Speed::Fast => Speed::Paused,
        }),
"""
assert old in s
p.write_text(s.replace(old, "        KeyCode::Char(' ') => Action::Ignore,\n"))
PY

mutation "status line omits the speed" tui status_line_fits_eighty_columns_without_truncation_at_large_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = """        let speed = match snapshot.speed {
            Speed::Paused => "paused",
            Speed::Normal => "normal",
            Speed::Fast => "fast",
        };
"""
assert old in s
p.write_text(s.replace(old, '        let speed = "";\n'))
PY

mutation "frames instrument ignores incoming ticks" tui streamed_frame_ticks_climb_when_no_key_is_sent <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
old = "    snapshot.tick = delta.tick;\n"
assert old in s
p.write_text(s.replace(old, "    let _ = delta.tick;\n"))
PY

mutation "frames key path never writes its command" tui key_space_freezes_the_streamed_frame_tick_and_reports_paused <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
old = "            send_command(&mut writer, command)?;\n"
assert old in s
p.write_text(s.replace(old, "", 1))
PY

mutation "client command writer has no timeout" tui command_writer_has_a_thirty_second_timeout <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
old = "        .set_write_timeout(Some(COMMAND_WRITE_TIMEOUT))\n"
assert old in s
p.write_text(s.replace(old, "        .set_write_timeout(None)\n"))
PY

mutation "set_speed discriminator is renamed" protocol decodes_and_reencodes_the_documented_command_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = "pub enum Command {\n    SetSpeed { speed: Speed },\n}\n"
assert old in s
p.write_text(s.replace(old, "pub enum Command {\n    #[serde(rename = \"set_rate\")]\n    SetSpeed { speed: Speed },\n}\n"))
PY
