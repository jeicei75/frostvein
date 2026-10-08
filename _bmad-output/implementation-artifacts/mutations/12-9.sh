# Story 12.9: One dwarf per tile (#133), stones block (#162). Run alone, after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-9.sh
# Row names follow the story's Task 7 list (rows 10-14 added at Task 0).

mutation "1 wander ignores occupancy" sim-core a_busy_crew_never_shares_a_tile_and_still_works <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    && !occupied.contains(p)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "2 the job step ignores occupancy" sim-core head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_open_end <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if occupied.contains(&path[0])\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && occupied.contains(&path[0])\n'))
PY

mutation "3 settle ignores occupancy" sim-core a_dwarf_that_falls_onto_another_comes_to_rest_on_a_free_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            let landing = if occupied.contains(&below) || blocked.contains(&below) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let landing = if blocked.contains(&below) {\n'))
PY

# The per-step rule FR49 read literally: an "escape" is any free cell, so the lower id backs off
# whenever it has one. The story hand-traced this to a livelock in fixture (b).
mutation "4 the per-step flip rule" sim-core head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_dead_end <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
for old in ('                |tile| !blocker_tiles.contains(&tile),\n',
            '                |tile| !holder_tiles.contains(&tile),\n'):
    assert s.count(old) == 1
    s = s.replace(old, '                |_| true,\n')
p.write_text(s)
PY

mutation "5 the higher id never yields" sim-core head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_dead_end <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        (Some(_), None) => true,\n        (None, Some(_)) => false,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        (Some(_), None) if holder_id < blocker_id => true,\n        (None, Some(_)) if blocker_id < holder_id => false,\n        (Some(_), None) | (None, Some(_)) => return false,\n'))
PY

mutation "6 an idle blocker never makes way" sim-core an_idle_dwarf_on_the_dig_face_follows_the_miner_out_and_the_dig_completes <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    if idle && !head_on(&blocker_path) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false && idle && !head_on(&blocker_path) {\n'))
PY

# Also the instrument self-test (Task 6): a frozen crew shares nothing, and must not read as green.
mutation "7 no dwarf ever steps" sim-core a_busy_crew_never_shares_a_tile_and_still_works <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
for old, new in (
    ('            let next = path.remove(0);\n', '            let next = pos;\n'),
    ('                *pos = candidates[rng.0.random_range(0..n)];\n', '                let _ = candidates[rng.0.random_range(0..n)];\n'),
    ('                *pos = next;\n', '                let _ = next;\n'),
):
    assert s.count(old) == 1
    s = s.replace(old, new)
p.write_text(s)
PY

mutation "8 to_save drops path" sim-core save_load_keeps_the_exit_path_of_an_idle_dwarf_leaving_a_dead_end <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    path: entity\n                        .get::<Path>()\n                        .map(|path| path.0.clone())\n                        .unwrap_or_default(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    path: Vec::new(),\n'))
PY

mutation "9 occupied tiles join claim-time blocked" sim-core a_dwarf_in_the_only_passage_does_not_make_a_reachable_job_unreachable_at_claim_time <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    let blocked = blocked_cells(emitters.iter().chain(items.values()));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let blocked = blocked_cells(emitters.iter().chain(items.values()).chain(dwarves.iter().map(|d| d.2)));\n'))
PY

# The old Q1(a): with no way aside, the idle blocker takes the holder's tile as the holder takes its.
mutation "10 an idle blocker with no way aside swaps tiles" sim-core an_idle_dwarf_on_the_dig_face_follows_the_miner_out_and_the_dig_completes <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        let closed = closed_to_dwarves(blocked, occupied, &[pos, next]);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if true {\n            *ecs.get_mut::<Pos>(blocker).expect("every dwarf has a position") = pos;\n            return true;\n        }\n' + old))
PY

mutation "11 blocked_cells ignores items" sim-core a_dwarf_with_a_goal_past_a_stone_in_a_corridor_goes_round_it <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
for old, new, n in (
    ('blocked_cells(emitters.iter().chain(items.values()))', 'blocked_cells(emitters.iter())', 1),
    ('blocked_cells(emitters.iter().chain(stones.values()))', 'blocked_cells(emitters.iter())', 2),
    ('                .filter(|(id, _)| !carried.contains(&id.0))\n                .map(|(_, pos)| pos),\n',
     '                .filter(|_| false)\n                .map(|(_, pos)| pos),\n', 1),
):
    assert s.count(old) == n, old
    s = s.replace(old, new)
p.write_text(s)
PY

mutation "12 haul pick-up stands on the item" sim-core a_hauler_picks_up_and_drops_from_the_next_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                Some(pos) if !targets.is_empty() && terrain.is_standable(*pos) => {
                    side_neighbours(*pos)
                        .into_iter()
                        .filter(|candidate| is_walkable(terrain, blocked, *candidate))
                        .collect()
                }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                Some(pos) if !targets.is_empty() && terrain.is_standable(*pos) => {
                    BTreeSet::from([*pos])
                }
'''))
PY

mutation "13 delivery picks the shallowest free pile cell" sim-core a_3x3_pile_fills_from_the_inside_out <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
for old, new in (
    ('                let deepest = targets.values().copied().max();\n', '                let deepest = targets.values().copied().min();\n'),
    ('        .max()\n        .map(|(_, Reverse(cell))| cell)\n', '        .min()\n        .map(|(_, Reverse(cell))| cell)\n'),
):
    assert s.count(old) == 1
    s = s.replace(old, new)
p.write_text(s)
PY

mutation "14 delivery drops at the hauler's own tile" sim-core a_hauler_picks_up_and_drops_from_the_next_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                            .expect("every stone has a position") = landing;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                            .expect("every stone has a position") = pos;\n'))
PY
