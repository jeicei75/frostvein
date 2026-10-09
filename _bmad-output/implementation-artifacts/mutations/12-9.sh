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

# Seat pass 1 (Tasks 11-12).
mutation "15 a channel is worked from the target itself" sim-core a_channel_is_worked_from_the_next_tile_and_the_stone_lands_on_the_target <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        JobKind::Channel => {
            if terrain.is_standable(job.target) {
                side_neighbours(job.target)
                    .into_iter()
                    .filter(|candidate| is_walkable(terrain, blocked, *candidate))
                    .collect()
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        JobKind::Channel => {
            if terrain.is_standable(job.target) {
                BTreeSet::from([job.target])
'''))
PY

mutation "16 dig_yaw ignores a channel" gui a_channelling_miner_faces_the_target_cell <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        | protocol::DwarfJob::Channel { target },\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "17 the blend moves a carried, unparented item" gui a_stone_the_wire_puts_in_his_hands_holds_its_cell_while_the_hauler_walks_in <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '.filter(|_| parent.is_none() && motion.is_none() && !carried.contains(&marker.0))'
assert s.count(old) == 1
p.write_text(s.replace(old, '.filter(|_| parent.is_none() && motion.is_none())'))
PY

mutation "18 the lift is instant" gui a_stone_picked_up_from_the_next_tile_rises_to_his_hands_over_a_lift <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        let progress = (motion.elapsed / LIFT_SECONDS).min(1.0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let progress = 1.0_f32;\n'))
PY

mutation "19 release snaps to the cell" gui a_stone_released_onto_the_next_tile_is_set_down_over_a_lift <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                        ItemMotion {\n                            from,\n                            to,\n                            elapsed: 0.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        ItemMotion {\n                            from: to,\n                            to,\n                            elapsed: 0.0,\n'))
PY

# ---- 12.9 review patch pass (run 1 findings) ----

# #184: depth layered through taken cells again (the shape on 34b6783), so a walled-in free centre
# holds the deepest layer and every delivery waits on it.
mutation "20 pile depth runs through taken cells" sim-core a_walled_in_free_cell_does_not_stop_the_pile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if zones.contains(&n) && is_walkable(terrain, blocked, n) && !depth.contains_key(&n) {\n'
assert s.count(old) == 1
s = s.replace(old, '            if zones.contains(&n) && terrain.is_standable(n) && !depth.contains_key(&n) {\n')
old = '        }\n    }\n    depth\n}\n'
assert s.count(old) == 1
s = s.replace(old, '        }\n    }\n    depth.retain(|cell, _| is_walkable(terrain, blocked, *cell));\n    depth\n}\n')
p.write_text(s)
PY

mutation "21 a saved path tile is not bounds-checked" simd loading_refuses_a_dwarf_path_tile_outside_the_map <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if let Some(tile) = dwarf.path.iter().find(|tile| !in_bounds(**tile)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if let Some(tile) = dwarf.path.iter().find(|_| false) {\n'))
PY

# #183: the idle blocker keeps its stale exit path when neither side of a head-on can escape.
mutation "22 a stuck head-on keeps the stale exit path" sim-core a_head_on_with_no_escape_drops_the_idle_blockers_stale_path <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if idle {\n                ecs.entity_mut(blocker).remove::<Path>();\n            }\n            return false;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            return false;\n'))
PY

# #182: a delivery lands on the deepest pile cell even when a dwarf stands on it, or filling it
# walls one in.
mutation "23 a delivery ignores who stands on the pile" sim-core no_dwarf_is_caged_by_stones_over_the_probe_seeds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-10-09 (12.9 review run 2, tick cost): `drop_cell` filters to pile cells first.
old = '        .filter(|(_, Reverse(cell))| !refused(*cell))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .filter(|(_, Reverse(cell))| { let _ = &refused; let _ = cell; true })\n'))
PY

# #182: no fill is ever refused for walling a dwarf in.
mutation "24 filling a cell never walls a dwarf in" sim-core no_dwarf_is_caged_by_stones_over_the_probe_seeds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-10-09 (12.9 review run 2, tick cost): `walls_in_a_dwarf` is a lockstep flood now.
old = '        .any(|piece| piece.len() < largest && dwarves.iter().any(|dwarf| piece.contains(dwarf)))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .any(|piece| piece.len() < largest && dwarves.iter().any(|dwarf| piece.contains(dwarf)) && false)\n'))
PY

# #182: a completing channel spawns its stone on the dwarf standing on its target.
mutation "25 a channel stone spawns under its occupant" sim-core no_dwarf_is_caged_by_stones_over_the_probe_seeds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if occupied.contains(&job.target) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if occupied.contains(&job.target) && false {\n'))
PY

# #182: a refused delivery holds at Work (the ruled first form), which deadlocks a hauler standing
# on the occupant's only way out.
mutation "26 a refused delivery holds instead of letting go" sim-core no_dwarf_is_caged_by_stones_over_the_probe_seeds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        retry_claim(ecs, entity, job.id);\n                        continue;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        *ecs.get_mut::<JobState>(entity).expect("every dwarf has a job state") = JobState::Work;\n                        continue;\n'))
PY

# #182: the abnormal drop lands on other dwarves and walls dwarves in again.
mutation "27 the abnormal drop ignores dwarves" sim-core release_claim_never_drops_where_the_stone_walls_a_dwarf_in <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    || (cell != pos && dwarves.contains(&cell))\n                    || walls_in_a_dwarf(terrain, &blocked, &dwarves, cell)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    || (cell != pos && dwarves.contains(&cell) && false)\n                    || (walls_in_a_dwarf(terrain, &blocked, &dwarves, cell) && false)\n'))
PY

# #182: a channel's stone may wall a dwarf in (the miner sealing itself, seed 10's first shape).
mutation "28 a channel stone may wall a dwarf in" sim-core no_dwarf_is_caged_by_stones_over_the_probe_seeds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if walls_in_a_dwarf(ecs.resource::<Terrain>(), &blocked, &occupied, job.target) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if walls_in_a_dwarf(ecs.resource::<Terrain>(), &blocked, &occupied, job.target) && false {\n'))
PY
