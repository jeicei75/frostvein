# Story 13.8b: after the big dig. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-8b.sh
# One row per decision (Task 6), each killed by the test named beside it. Every anchor is a named
# literal with a count guard, so `audit-mutations.py` can see it rot.
# Rows 4-9 and 12 die at the debug assert in `execute_jobs` ("the kept stone map is current"), not
# at an assert of the named test: the assert is the instrument (AC 5), the test is what walks it.
# NOTE: the story's row "claim_jobs reuses pile cells from the previous call" cannot be written:
# `pile` is a local of one call, so there is nothing to carry over. Row 11 pins the other side.

mutation "1 the kept failures are never used (AC 2)" sim-core an_unreachable_dig_is_flooded_once_not_once_per_cooldown <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if was_blocked.is_subset(&blocked) {\n            components = kept;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if was_blocked.is_subset(&blocked) {\n            drop(kept);\n'))
PY

mutation "2 the kept failures survive set_tile (AC 3 i)" sim-core a_tile_that_opens_the_route_drops_the_kept_failure <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            .expect("nothing panics holding the kept failures")\n            .1\n            .clear();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .expect("nothing panics holding the kept failures")\n            .1\n            .len();\n'))
PY

mutation "3 the kept failures survive a cell leaving blocked (AC 3 ii)" sim-core a_stone_hauled_off_the_only_route_drops_the_kept_failure <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if was_blocked.is_subset(&blocked) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if true || was_blocked.is_subset(&blocked) {\n'))
PY

mutation "4 the kept stone map misses a miner's lift (AC 5)" sim-core a_carried_stone_is_always_set_down_beside_its_carrier <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                // The lift, and any hauler losing custody below, change the stones.\n                stale = true;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                // The lift, and any hauler losing custody below, change the stones.\n'))
PY

mutation "5 the kept stone map misses a miner's set-down (AC 5)" sim-core a_full_stockpile_never_stacks_uncarried_stones <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                *ecs.get_mut::<Pos>(stone).expect("item position") = landing;\n                stale = true;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                *ecs.get_mut::<Pos>(stone).expect("item position") = landing;\n'))
PY

mutation "6 the kept stone map misses a dig's spawned stone (AC 5)" sim-core head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_open_end <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        stale = true;\n        clear_paths(ecs);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        clear_paths(ecs);\n'))
PY

mutation "7 the kept stone map misses a haul pick-up (AC 5)" sim-core a_3x3_pile_fills_from_the_inside_out <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                None => {\n                    stale = true;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                None => {\n'))
PY

mutation "8 the kept stone map misses a haul delivery (AC 5)" sim-core a_3x3_pile_fills_from_the_inside_out <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    // Delivered, retried or dropped below, the stone ends up somewhere else.\n                    stale = true;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    // Delivered, retried or dropped below, the stone ends up somewhere else.\n'))
PY

mutation "9 the kept stone map misses a felled tree's wood (AC 5)" sim-core digging_a_marked_trees_ground_releases_its_woodcutter <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if job.kind == JobKind::Cut {\n            stale = true;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if job.kind == JobKind::Cut {\n'))
PY

mutation "10 the stone map is built again for every dwarf (AC 5)" sim-core execute_jobs_builds_the_stone_map_once_when_no_stone_moves <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if std::mem::take(&mut stale) {\n            kept = None;\n            failed.clear();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if std::mem::take(&mut stale) || true {\n            kept = None;\n            failed.clear();\n'))
PY

mutation "11 the delivery set finds the pile cells again (AC 6)" sim-core claim_jobs_finds_the_pile_cells_once_per_call <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    Some(item),\n                    &mut pile,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    Some(item),\n                    &mut None,\n'))
PY

mutation "12 a released carrier's dropped stone is missed (AC 5)" sim-core a_carried_stone_is_always_set_down_beside_its_carrier <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    {\n        *stale = true;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    {\n        let _ = stale;\n    }\n'))
PY

# Amendment 2026-11-15 (Tasks 10 and 11; Task 13's rows). 13-4.sh row 1 pins the idle blocker's
# wait (`0 * SQUEEZE_WAIT_TICKS`); this row pins the other side, that a head-on job-holder does not
# wait. Dies at the holder's position assert, the first after the call.
mutation "13 head-on job-holders wait SQUEEZE_WAIT_TICKS again (AC 12)" sim-core two_job_holders_head_on_trade_tiles_on_the_first_look <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    if idle && now - since < SQUEEZE_WAIT_TICKS {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if now - since < SQUEEZE_WAIT_TICKS {\n'))
PY

# Dies at the first fixture's `south` assert (he steps aside onto the north pile cell).
mutation "14 the step aside lands on a free pile cell (Task 10, #189's loop)" sim-core an_idle_dwarf_moved_out_of_the_way_never_lands_on_a_free_pile_cell <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                !occupied.contains(tile) && !holder_tiles.contains(tile) && !piles.contains(tile)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                !occupied.contains(tile) && !holder_tiles.contains(tile)\n'))
PY

# Dies at the second fixture's "the exit ends on" (both sides are pile cells; the exit takes one).
mutation "15 the exit ends on a free pile cell (Task 10, #189's loop)" sim-core an_idle_dwarf_moved_out_of_the_way_never_lands_on_a_free_pile_cell <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            tile != pos && !holder_tiles.contains(&tile) && !piles.contains(&tile)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            tile != pos && !holder_tiles.contains(&tile)\n'))
PY

# The failed re-plan list is kept but never consulted: the plate is flooded once per miner.
mutation "16 a failed re-plan's component is flooded again (AC 14)" sim-core failed_re_plans_in_one_component_flood_it_once <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                let computed = if failed.iter().any(|component| {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let computed = if false && failed.iter().any(|component| {\n'))
PY

# The list outlives the stone map: a component flooded behind a stone is skipped after the lift.
mutation "17 the failed re-plans survive a stone moving (AC 14)" sim-core a_stone_lifted_mid_call_lets_a_later_re_plan_through <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            kept = None;\n            failed.clear();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            kept = None;\n'))
PY

# The skip ignores the goals: a later miner whose dig faces the flooded plate is denied his path.
mutation "18 a failed re-plan's skip ignores the goals (AC 14)" sim-core a_failed_re_plan_leaves_a_reachable_dig_in_its_component_to_the_next_miner <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    component.contains(&pos) && work_positions.is_disjoint(component)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    component.contains(&pos) && true\n'))
PY

mutation "19 a ring of job-holders is never stepped (AC 13)" sim-core a_ring_of_waiting_job_holders_steps_at_once <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            step_the_ring(ecs, blocked, pos, path, blocker);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# Dies at "the holder's path is unchanged" (the test's first assert after `step`, which the ring
# also answers `false`).
mutation "20 the ring takes a link that is not walking (AC 13)" sim-core a_chain_ending_on_a_dwarf_working_in_place_moves_nobody <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            || ecs.get::<JobState>(link) != Some(&JobState::Walk)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "21 the ring takes an idle link (AC 13)" sim-core a_chain_through_an_idle_dwarf_with_an_exit_moves_nobody <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if ecs\n            .get::<CurrentJob>(link)\n            .is_none_or(|current| current.0.is_none())\n            || ecs.get::<JobState>(link) != Some(&JobState::Walk)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if ecs.get::<JobState>(link) != Some(&JobState::Walk)\n'))
PY

mutation "22 the ring takes a link whose next step is not legal (AC 13)" sim-core a_chain_with_a_step_that_is_not_a_legal_move_moves_nobody <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            || steps.first().is_none_or(|next| !legal(from, *next))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            || steps.first().is_none()\n'))
PY

mutation "23 the ring steps its links but not the holder (AC 13)" sim-core a_ring_of_waiting_job_holders_steps_at_once <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    *ecs.get_mut::<Pos>(at[&pos])\n        .expect("every dwarf has a position") = path.remove(0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# Dies at "dwarf 1 stepped round the ring": the holder's own asserts pass.
mutation "24 the ring steps the holder but not its links (AC 13)" sim-core a_ring_of_waiting_job_holders_steps_at_once <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        *ecs.get_mut::<Pos>(member)\n            .expect("every dwarf has a position") = steps.remove(0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        steps.remove(0);\n'))
PY

mutation "25 the tick timer's step measures nothing (AC 18)" simd the_daemon_logs_its_tick_times_every_thousand_stepped_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            step = Some(at.elapsed());\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            step = Some(Duration::ZERO);\n'))
PY
