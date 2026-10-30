# Story 13.6: Two hundred on 256². Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-6.sh
# One row per decision (Task 9), each killed by the test named beside it. Row 10 is release-only
# (about 3 min): its target panics in a debug build on purpose, so it runs `release-ignored`.

mutation "1 the spill never runs: a big crew draws from the ring only (AC 4)" sim-core a_crew_of_200_spawns_near_the_camp_one_per_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if candidates.len() < crew {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false {\n'))
PY

# Tasks 1-2 found that a box taken in iteration order SURVIVES (its first cells are all
# reachable), so this row draws at random from the whole box, as the dwarf loop does.
mutation "2 the spill takes any standable cell in a box, not the camp's component (AC 4)" sim-core a_crew_of_200_spawns_near_the_camp_one_per_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                assert!(\n                    candidates.len() >= crew,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '''                candidates.clear();
                for z in camp.z - 6..=camp.z + 6 {
                    for y in camp.y - 20..=camp.y + 20 {
                        for x in camp.x - 20..=camp.x + 20 {
                            let pos = Pos { x, y, z };
                            if terrain.is_standable(pos) && !emitter_positions.contains(&pos) {
                                candidates.push(pos);
                            }
                        }
                    }
                }
''' + old))
PY

mutation "3 the spill always runs, even for crew 5 (AC 4)" sim-core spawn_positions_for_seed_42_are_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if candidates.len() < crew {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if true {\n'))
PY

# 80/80/40 holds under a plain cycle too: only the test's order assert can kill this.
mutation "4 professions cycle i % 5, never shuffled (AC 9)" sim-core crew_200_holds_the_two_two_one_mix <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        professions.shuffle(&mut profession_rng);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "5 the scenario loader ignores dims (AC 1)" simd a_seed_start_with_dims_and_crew_builds_that_world <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = 'crate::generate_world(*seed, *dims, *crew)'
assert s.count(old) == 1
p.write_text(s.replace(old, 'crate::generate_world(*seed, sim_core::Dims::DEFAULT, *crew)'))
PY

mutation "6 an absent dims means 256 (AC 1, AC 5, Q2 a)" simd bench_replays_the_scenario_identically_and_reaches_the_oracle_state <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = 'dims.map_or(sim_core::Dims::DEFAULT,'
assert s.count(old) == 1
p.write_text(s.replace(old, 'dims.map_or(sim_core::Dims::FRESH,'))
PY

mutation "7 the fresh game is 128 (AC 3)" simd a_fresh_game_serves_a_256_valley_with_five_dwarves <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = 'dims: dims.unwrap_or(sim_core::Dims::FRESH),'
assert s.count(old) == 1
p.write_text(s.replace(old, 'dims: dims.unwrap_or(sim_core::Dims::DEFAULT),'))
PY

mutation "8 done never counts a finished dig (AC 7)" simd a_finished_dig_is_counted_as_done_and_as_a_changed_cell <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = 'sim_core::DesignationKind::Dig => dug += 1,'
assert s.count(old) == 1
p.write_text(s.replace(old, 'sim_core::DesignationKind::Dig => dug += 0,'))
PY

mutation "9 cells compares the end world with itself (AC 7)" simd a_finished_dig_is_counted_as_done_and_as_a_changed_cell <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '            .zip(&start_tiles)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .zip(world.tiles())\n'))
PY

mutation "10 nothing is ever hauled (AC 6)" simd the_two_hundred_on_256_scenario_does_real_work release-ignored <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'fn create_haul_jobs(ecs: &mut EcsWorld) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, old + '    if true {\n        return;\n    }\n'))
PY

mutation "11 the focus ceiling stays 127 on a 256 world (Q5 a)" gui a_256_world_lifts_the_focus_ceiling_past_the_old_128_clamp <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '            dims.x as f32 - 1.0,\n            dims.y as f32 - 1.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            127.0,\n            127.0,\n'))
PY
