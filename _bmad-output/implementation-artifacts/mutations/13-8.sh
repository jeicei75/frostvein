# Story 13.8: they keep coming. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-8.sh
# One row per decision (Task 8), each killed by the test named beside it. Every anchor is a named
# literal with a count guard, so `audit-mutations.py` can see it rot.

mutation "1 arrive never fires (AC 2)" sim-core the_first_group_arrives_at_the_period_on_distinct_camp_cells_from_the_edge <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    if !ecs.resource::<Arrivals>().0 || !tick.is_multiple_of(ARRIVAL_PERIOD) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if true {\n'))
PY

mutation "2 the group size is not capped at CREW_CAP (AC 2)" sim-core the_crew_stops_at_two_hundred <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    let size = place_rng.random_range(ARRIVAL_GROUP).min(room);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let size = place_rng.random_range(ARRIVAL_GROUP);\n'))
PY

mutation "3 the embark epithet is always Epithet::ALL[0] (AC 5)" sim-core crew_200_holds_two_hundred_distinct_full_names <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                let epithet = *free\n                    .choose(&mut identity_rng)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let epithet = *[Epithet::ALL[0]]\n                    .choose(&mut identity_rng)\n'))
PY

# Row 4 is the RED Task 2 ran: the plan is made only on an edge cell, as if planned once at spawn.
# `every_arrival_ends_its_walk_near_its_destination` passes under it; only the churn test sees it.
mutation "4 the Arriving re-plan is gone: the walk-in is planned once (AC 3 ii)" sim-core every_arrival_ends_its_walk_through_terrain_churn <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if let Some(arriving) = arriving.as_deref() {\n            if near_arrival_target(arriving.to, here) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if let Some(arriving) = arriving.as_deref().filter(|_| {\n            here.x == 0\n                || here.y == 0\n                || here.x == terrain.dims.x as i32 - 1\n                || here.y == terrain.dims.y as i32 - 1\n        }) {\n            if near_arrival_target(arriving.to, here) {\n'))
PY

# The story's row 5: if no test can see it, the record says so rather than inventing one.
mutation "5 release_claim keeps Arriving: a job no longer ends the walk-in (AC 3 iii)" sim-core an_arriving_miner_claims_a_dig_and_digs_it <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    entity.remove::<Arriving>();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "6 the full-name check is gone from the load validator (AC 5)" simd a_save_that_repeats_a_full_name_is_refused_naming_the_cause <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if !seen_names.insert((dwarf.identity.name, dwarf.identity.epithet)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if !seen_names.insert((dwarf.identity.name, dwarf.identity.epithet)) && false {\n'))
PY

mutation "7 the Arrivals setting is ignored: arrivals always on (AC 8)" sim-core the_arrivals_setting_decides_the_crew_and_survives_a_save <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    if !ecs.resource::<Arrivals>().0 || !tick.is_multiple_of(ARRIVAL_PERIOD) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if !tick.is_multiple_of(ARRIVAL_PERIOD) {\n'))
PY

mutation "8 a crew of 3 takes the 5-cycle's [M, M, H] (AC 1)" sim-core every_crew_of_three_or_more_holds_all_three_trades <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if (3..5).contains(&crew) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if (4..5).contains(&crew) {\n'))
PY

mutation "9 the edge filter is gone: a group enters anywhere in the camp's component (AC 2)" sim-core the_first_group_arrives_at_the_period_on_distinct_camp_cells_from_the_edge <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        .filter(|cell| edge(cell) && free(cell, &occupied) && !blocked.contains(cell))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .filter(|cell| (edge(cell) || true) && free(cell, &occupied) && !blocked.contains(cell))\n'))
PY

mutation "10 the scenario arrivals key is ignored (AC 8, AC 11)" simd a_seed_start_with_arrivals_grows_the_crew_and_without_it_does_not <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '                    arrivals: arrivals.unwrap_or(false),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    arrivals: false,\n'))
PY

# Rows 11-12: Wolf's 2026-11-03 ruling (the they-keep-coming stall): a walk ends after
# ARRIVAL_PATIENCE_TICKS waiting on an occupied tile, and the count is saved.
mutation "11 the patience limit is gone: an arrival waits forever behind a blocker" sim-core an_arrival_gives_up_after_waiting_the_patience_limit_behind_a_blocker <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        if arriving.waited >= ARRIVAL_PATIENCE_TICKS {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        if false {\n'))
PY

mutation "12 a load forgets the wait count (AC 7)" sim-core a_save_round_trip_keeps_the_arrival_wait_count <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    waited: saved.waited,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    waited: 0,\n'))
PY
