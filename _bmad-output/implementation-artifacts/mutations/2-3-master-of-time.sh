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
# Re-pointed 2026-08-22: the surrounding code grew (rect validation, haul jobs, a `path`
# argument, a larger save ceiling) and this anchor rotted with it. The seam is unchanged.
old = '''                        if command_tx.send(command).is_err() {
                            break;
                        }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '                        let _ = command;\n'))
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
# Narrowed 2026-08-22: 3.1 replaced the empty `zones` placeholder this anchored on with a
# real projection. The speed field is what the row is about, so anchor that alone.
old = "        speed,\n        tick: world.tick(),\n"
assert s.count(old) == 1
p.write_text(s.replace(old, "        speed: protocol::Speed::Normal,\n        tick: world.tick(),\n"))
PY

mutation "delta bridge hardcodes normal speed" simd delta_carries_dirty_tiles_and_full_authoritative_state <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
# Narrowed 2026-08-22: 3.1 replaced the empty `zones` placeholder this anchored on with a
# real projection. The speed field is what the row is about, so anchor that alone.
old = "        speed,\n    }\n}\n"
assert s.count(old) == 1
p.write_text(s.replace(old, "        speed: protocol::Speed::Normal,\n    }\n}\n"))
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
# Re-pointed 2026-08-22: `command(..)` gained a `state` argument and rustfmt broke the arm
# across lines.
old = '''        KeyCode::Char(' ') => command(
            state,
            match speed {
                Speed::Paused => Speed::Normal,
                Speed::Normal | Speed::Fast => Speed::Paused,
            },
        ),
'''
assert s.count(old) == 1
p.write_text(s.replace(old, "        KeyCode::Char(' ') => Action::Ignore,\n"))
PY

mutation "status line omits the speed" tui status_line_fits_eighty_columns_without_truncation_at_large_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
# Re-pointed 2026-08-22: the status line reads `state.speed` since the mirror adoption.
old = '''        let speed = match state.speed {
            Speed::Paused => "paused",
            Speed::Normal => "normal",
            Speed::Fast => "fast",
        };
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '        let speed = "";\n'))
PY

# REMOVED 2026-08-22 -- obsolete, not repaired. This row sabotaged `snapshot.tick = delta.tick`
# inside `tui`'s own delta application. Story 5.2 moved delta application out of `tui`
# entirely and into the shared `client-core` mirror, so the seam no longer exists in this
# crate and pinning it from a `tui` row is not possible. `client-core` owns it now.

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
# Narrowed 2026-08-22: this pinned the WHOLE enum as it stood with one variant, and rotted the
# moment 2.4 added Save/Load/Quit. Anchor the variant, not its container.
old = "    SetSpeed { speed: Speed },\n"
assert s.count(old) == 1
p.write_text(s.replace(old, "    #[serde(rename = \"set_rate\")]\n    SetSpeed { speed: Speed },\n"))
PY
