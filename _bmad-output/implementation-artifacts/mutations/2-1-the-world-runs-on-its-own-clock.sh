# Mutation set for story 2.1's review patches. Run: scripts/mutate.sh <this file>
# Every one of these must be KILLED. The first one SURVIVED on its first run — the
# socket-shutdown test passed with the fix removed, because dropping the evicted Client
# closed the socket unaided. Kept here as the worked example of why this script exists.

mutation "eviction no longer shuts the socket" simd full_client_queue_is_removed_from_the_registry_at_sixteen_lines <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
p.write_text(s.replace("            let _ = client.stream.shutdown(Shutdown::Both);\n", ""))
PY

mutation "tick loop never sleeps (rate broken, constant untouched)" simd deltas_arrive_at_roughly_ten_per_second <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
p.write_text(s.replace(
    "        thread::sleep(deadline.saturating_duration_since(Instant::now()));\n",
    "        let _ = deadline;\n"))
PY

mutation "client loop receives deltas but never applies them" tui the_client_loop_renders_a_frame_per_streamed_delta <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
# ROTTED and undetected until 2026-08-26: the arm this named became a multi-line block long ago,
# so `replace` matched nothing, wrote the file back byte-identical, and the row reported SURVIVED
# -- "your test is weak" when the truth was "your sabotage is broken". Re-pointed, and given the
# count guard the house format requires so it can never rot silently again.
old = '''                Ok(Ok(Msg::Delta(delta))) => {
                    mirror.apply_delta(*delta);
                    state.speed = mirror.speed();
                    needs_redraw = true;
                }'''
assert s.count(old) == 1
p.write_text(s.replace(old, old.replace('mirror.apply_delta(*delta);', 'drop(delta);')))
PY

mutation "step() clears the dirty set (per-tick instead of per-drain)" sim-core stepping_does_not_clear_the_dirty_set <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
p.write_text(s.replace(
    "        self.schedule.run(&mut self.ecs);",
    "        self.schedule.run(&mut self.ecs);\n        self.dirty.clear();"))
PY

mutation "drain_dirty returns descending order" sim-core dirty_tiles_are_sorted_and_out_of_bounds_writes_do_nothing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
p.write_text(s.replace(
    "        std::mem::take(&mut self.dirty)\n            .into_iter()",
    "        std::mem::take(&mut self.dirty)\n            .into_iter()\n            .rev()"))
PY
