# Story 13.4: Squeeze past (two dwarves trade tiles). Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-4.sh
# One row per decision (Task 6), each killed by the test named beside it. Task 6's row 6 (one per
# shape fixed in Task 4) has none: Wolf ruled at Task 3's split point to pin the 16k cage runs
# instead of fixing them (AC 5 amended; the shapes are #218).
# Run 2026-10-26 on 3751d336: 8/8 KILLED, each at its own assert (read from the panic line): 1 lib.rs
# first-look path assert; 1b the post-wait swap assert; 2 `watch`'s "two dwarves share a tile at tick
# 113"; 3 "dwarves 0 and 1 never swapped"; 4b the idle swap assert; 4 the idle `home` assert; 5
# save_load "saves taken mid-stand-off diverged"; 5b save_load "no stand-off timer ever ran".

mutation "1 the wait is 0: a swap on the first look (AC 3)" sim-core a_held_dwarf_swaps_only_after_the_wait <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    if now - since < SQUEEZE_WAIT_TICKS {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if now - since < 0 * SQUEEZE_WAIT_TICKS {\n'))
PY

# The stand-off is forgotten between looks (a blocked dwarf looks once per step period), so the
# wait never runs out and nobody swaps.
mutation "1b the stand-off is forgotten between looks (AC 3)" sim-core a_held_dwarf_swaps_only_after_the_wait <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                && now - squeeze.last <= 2 * u64::from(STEP_REST_TICKS) =>\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                && now - squeeze.last <= 0 =>\n'))
PY

# 13.2's shares=64 sabotage: the blocker is written onto the holder's tile and the holder stays.
# Must die on `watch`'s shared-tile assert ("share a tile"), not on `World::step`'s debug_assert,
# which shadows it in a debug run (12-10b.sh row 19), so that is compiled out here as row 19 does.
mutation "2 the swap writes the blocker's Pos but not the holder's (AC 2)" sim-core head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_open_end <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    *ecs.get_mut::<Pos>(holder)\n        .expect("every dwarf has a position") = next;\n'
assert s.count(old) == 1
s = s.replace(old, '')
old = '        self.schedule.run(&mut self.ecs);\n        debug_assert!(\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.schedule.run(&mut self.ecs);\n        #[cfg(any())]\n        debug_assert!(\n'))
PY

mutation "3 head-on job-holders never swap (AC 1, 7)" sim-core head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_open_end <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    if !idle {\n        try_squeeze(\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false {\n        try_squeeze(\n'))
PY

mutation "4 the idle blocker's home is not moved (AC 1, Q1 a)" sim-core a_held_dwarf_swaps_only_after_the_wait <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    if idle {\n        wander.home = pos;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "4b an idle blocker never swaps (AC 1, Q1 a)" sim-core a_held_dwarf_swaps_only_after_the_wait <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    let next = path[0];\n    let now = ecs.resource::<Tick>().0;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if idle {\n        return false;\n    }\n' + old))
PY

# Review run 1, decision 1 (a): the swapped idle blocker keeps the tail of his exit `Path`, which
# now starts two tiles from him (the idle-with-an-exit site is the only one where it is longer than 1).
mutation "4c the swapped idle blocker keeps his exit Path (AC 1, decision 1 a)" sim-core an_idle_blocker_with_an_exit_ahead_swaps_after_the_wait <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    if idle || blocker_path.len() <= 1 {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if blocker_path.len() <= 1 {\n'))
PY

mutation "5 from_save drops the stand-off timer (AC 4)" sim-core a_world_saved_at_every_tick_of_a_stand_off_loads_and_steps_identically <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if let Some(squeeze) = dwarf.squeeze {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if let Some(squeeze) = dwarf.squeeze.filter(|_| false) {\n'))
PY

mutation "5b to_save drops the stand-off timer (AC 4)" sim-core a_world_saved_at_every_tick_of_a_stand_off_loads_and_steps_identically <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    squeeze: entity.get::<Squeeze>().map(|squeeze| SavedSqueeze {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    squeeze: entity.get::<Squeeze>().filter(|_| false).map(|squeeze| SavedSqueeze {\n'))
PY
