# Story 12.12: The first goals. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-12.sh
# Row names follow the story's Task 7 list; b-rows are second killers (the chain test, the wire's serve test).

mutation "1 progress counts loose items too (AC 2)" sim-core goal_progress_counts_stored_items_only <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    let (stored, _) = split_stored(ecs);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let (mut stored, loose) = split_stored(ecs);\n    stored.extend(loose.iter().map(|(id, _)| *id));\n'))
PY

mutation "2 progress counts a second item on a cell (AC 2)" sim-core goal_progress_counts_stored_items_only <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if zones.contains(&pos) && occupied.insert(pos) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if zones.contains(&pos) {\n'))
PY

mutation "3 progress counts carried items (AC 2)" sim-core goal_progress_counts_stored_items_only <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    for (id, pos) in uncarried_stones(ecs) {\n        if zones.contains(&pos) && occupied.insert(pos) {\n'
assert s.count(old) == 1
new = ('    let all: BTreeMap<u32, Pos> = ecs\n'
       '        .iter_entities()\n'
       '        .filter(|entity| entity.contains::<Item>())\n'
       '        .filter_map(|entity| Some((entity.get::<Id>()?.0, *entity.get::<Pos>()?)))\n'
       '        .collect();\n'
       '    for (id, pos) in all {\n        if zones.contains(&pos) && occupied.insert(pos) {\n')
p.write_text(s.replace(old, new))
PY

mutation "4 a goal un-completes when its stock drops (AC 3)" sim-core a_completed_goal_stays_completed_when_its_stock_leaves <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    let mut done = ecs.resource::<GoalsCompleted>().0;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let mut done = 0;\n'))
PY

mutation "5 order ignored: a goal completes on its own stock (AC 3)" sim-core a_goal_completes_only_after_the_ones_ahead_of_it <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                completed: i < done,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                completed: i < done || stored.get(&kind).copied().unwrap_or(0) >= target,\n'))
PY

mutation "5b order ignored, the chain on DEFAULT_SEED (AC 6)" sim-core the_default_world_completes_the_goal_chain_with_scripted_commands <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                completed: i < done,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                completed: i < done || stored.get(&kind).copied().unwrap_or(0) >= target,\n'))
PY

mutation "6 no cascade: one goal per tick (AC 3, D2)" sim-core the_chain_cascades_in_one_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        done += 1;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        done += 1;\n        break;\n    }\n'))
PY

mutation "7 to_save drops goals_completed (AC 4)" sim-core save_load_with_a_goal_done_and_the_next_in_progress_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            goals_completed: self.ecs.resource::<GoalsCompleted>().0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            goals_completed: 0,\n'))
PY

mutation "8 the load bound dropped (AC 4, D3)" simd loading_refuses_goals_completed_beyond_the_chain <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '        if save.goals_completed as usize > sim_core::GOALS.len() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && save.goals_completed as usize > sim_core::GOALS.len() {\n'))
PY

mutation "9 the bridge reports progress 0 (AC 5)" simd snapshot_and_delta_carry_the_goal_chain_as_hand_written_wire_values <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '            progress: goal.progress,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            progress: 0,\n'))
PY

mutation "9b the bridge reports progress 0, real daemon (AC 5)" simd a_loaded_pile_reports_its_stored_count_as_goal_progress_in_the_snapshot_and_the_next_delta <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '            progress: goal.progress,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            progress: 0,\n'))
PY

mutation "10 the Mirror drops goals (Task 5)" client-core a_deltas_goals_reach_the_mirror_and_a_snapshot_resets_them <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '        self.goals = goals;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let _ = goals;\n'))
PY

mutation "11 the tui row ignores completed (AC 7)" tui the_goal_row_changes_on_the_delta_that_changes_goals <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '    match goals.iter().position(|goal| !goal.completed) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    match goals.iter().position(|_| true) {\n'))
PY

mutation "12 the gui readout ignores completed (AC 8)" gui the_goal_list_follows_the_mirrors_goals_in_the_approved_colours <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            client_core::GoalState::Done => crate::appearance::GOAL_DONE_COLOR,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            client_core::GoalState::Done => Color::WHITE,\n'))
PY

mutation "13 the Task 2b entrance rule reverted (AC 11)" sim-core a_pile_round_enclosed_ground_still_fills_every_cell <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '&& is_walkable(terrain, blocked, *n) && open_ground(*n))'
assert s.count(old) == 1
p.write_text(s.replace(old, '&& is_walkable(terrain, blocked, *n) && (open_ground(*n) || true))'))
PY

mutation "14 a hole beside rock reads as open again (AC 11, review run 1 Decision 1)" sim-core a_pile_round_ground_beside_rock_still_fills_every_cell <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if seen.len() >= cap || !in_box(cell) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if seen.len() >= cap || !in_box(cell) || side_neighbours(cell).iter().any(|n| !zones.contains(n) && !terrain.is_standable(*n)) {\n'))
PY
