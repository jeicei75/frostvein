# Story 6.1 sabotage table. Run alone with scripts/mutate.sh.

mutation "blend extrapolates beyond delivered state" gui blend::tests::midpoint_and_snap_are_literal_wire_positions <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/blend.rs'); s = p.read_text()
old = 'world_to_render(current), factor.clamp(0.0, 1.0))'
assert s.count(old) == 1
p.write_text(s.replace(old, 'world_to_render(current), factor)'))
PY

mutation "torch flicker band widens" gui appearance::flicker_tests::flicker_is_bounded_distinct_and_deterministic <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '            flicker_amplitude: 0.07,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            flicker_amplitude: 0.70,\n'))
PY

mutation "dig chips lose client-local ownership" gui empty_tile_delta_leaves_deterministic_client_local_chips_and_snapshot_clears_them <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                        DigChip(*position),\n                        ClientLocal,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        DigChip(*position),\n'))
PY

mutation "motion capture requires too many ticks" gui capture::tests::motion_instrument_rejects_stillness_and_accepts_the_required_observation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'self.ticks.len() >= 100'
assert s.count(old) == 1
p.write_text(s.replace(old, 'self.ticks.len() >= 101'))
PY
