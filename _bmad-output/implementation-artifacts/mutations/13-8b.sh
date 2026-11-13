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
old = '        if std::mem::take(&mut stale) {\n            kept = None;\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if std::mem::take(&mut stale) || true {\n            kept = None;\n        }\n'))
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
