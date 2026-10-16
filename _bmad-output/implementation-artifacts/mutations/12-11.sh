# Story 12.11: Nothing floats. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-11.sh
# Row names follow the story's Task 7 list; b-rows are second killers or the dev-time clauses
# (the marked-cell set-down refusal, the gui dig preview, the hauler's let-go, simd's save check).

mutation "1 the fall is removed (AC 2)" sim-core a_loose_stone_falls_to_standable_ground_in_one_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if !falling {\n            continue;\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if true {\n            continue;\n        }\n'))
PY

mutation "1b the fall is removed, dig case (AC 2)" sim-core a_dig_under_a_stone_does_not_leave_it_floating <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if !falling {\n            continue;\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if true {\n            continue;\n        }\n'))
PY

mutation "2 a fall may land on an emitter's tile (AC 2, FR50)" sim-core a_loose_stone_falls_to_standable_ground_in_one_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        let mut blocked = world_blocked(ecs);\n        blocked.remove(&from);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let mut blocked = blocked_cells(uncarried_stones(ecs).values());\n        blocked.remove(&from);\n'))
PY

mutation "3 a fall may land on a dwarf (AC 2)" sim-core a_fallen_stone_never_lands_on_a_dwarf <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            cell.z > from.z\n                || blocked.contains(&cell)\n                || dwarves.contains(&cell)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            cell.z > from.z\n                || blocked.contains(&cell)\n'))
PY

mutation "4 a fall may land higher than it started (AC 2)" sim-core a_fallen_stone_never_lands_above_where_it_started <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            cell.z > from.z\n                || blocked.contains(&cell)\n                || dwarves.contains(&cell)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            blocked.contains(&cell)\n                || dwarves.contains(&cell)\n'))
PY

mutation "5 the miner waits instead of clearing: tunnel (AC 14)" sim-core a_miner_clears_a_four_mark_tunnel_without_a_pile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'let clearing_item = if matches!(job.kind, JobKind::Dig | JobKind::Channel) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'let clearing_item = if false && matches!(job.kind, JobKind::Dig | JobKind::Channel) {'))
PY

mutation "5b the miner waits instead of clearing: channel target (AC 4)" sim-core a_channel_miner_carries_its_target_stone_aside_before_cutting <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'let clearing_item = if matches!(job.kind, JobKind::Dig | JobKind::Channel) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'let clearing_item = if false && matches!(job.kind, JobKind::Dig | JobKind::Channel) {'))
PY

mutation "6 the bridge reports Dig/Channel while clearing (AC 15)" simd a_clearing_miner_reads_haul_on_the_wire_and_his_own_job_before_and_after <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = 'job: if clearing.iter().any(|(miner, _)| *miner == id) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'job: if false && clearing.iter().any(|(miner, _)| *miner == id) {'))
PY

mutation "7 the set-down is allowed in the target's column (AC 4; column + mark clauses, the column alone is equivalent)" sim-core a_clearing_miner_never_sets_the_stone_down_in_the_target_column <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    (cell.x, cell.y) == (job.target.x, job.target.y)\n                        || cell.z < pos.z\n                        || blocked.contains(&cell)\n                        || occupied.contains(&cell)\n                        || job_positions.contains(&cell)\n                        || ecs.resource::<Designations>().0.contains_key(&cell)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    cell.z < pos.z\n                        || blocked.contains(&cell)\n                        || occupied.contains(&cell)\n                        || job_positions.contains(&cell)\n'))
PY

mutation "7b the set-down is allowed on a marked cell (cage sweep: 8 frozen, not 5)" sim-core no_dwarf_is_caged_by_stones_over_the_probe_seeds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        || ecs.resource::<Designations>().0.contains_key(&cell)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "8 the Q2 rule is reverted (AC 5)" sim-core digging_hillside_tree_ground_fells_the_tree <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                fell_tree(ecs, above);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "9 the D4 dig filter is dropped (AC 8)" sim-core a_dig_mark_is_refused_beneath_an_emitter_only <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                                        && !emitter_tiles.contains(&Pos {\n                                            z: pos.z + 1,\n                                            ..*pos\n                                        })\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "9b the gui dig preview lights the cell under an emitter (AC 8)" gui the_channel_preview_skips_an_emitters_tile_and_cells_below_z_two <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                && !on_emitter([tile[0], tile[1], tile[2] + 1])\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "10 the chip despawn over an emptied floor is dropped (AC 10)" gui channel_under_debris_removes_chips_above_the_opened_floor <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'if chip.0 == *position || (emptied && chip.0 == above) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if chip.0 == *position {'))
PY

mutation "11 the hauler keeps the lifted stone's job (D6, AC 3)" sim-core a_hauler_holding_the_lifted_stones_job_lets_go_and_the_stone_is_delivered_later <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                for hauler in haulers {\n                    release_claim(ecs, hauler);\n                }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "12 simd's save validation refuses a clearing miner (Task 3)" simd a_save_taken_mid_clear_loads_and_the_clear_finishes <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = 'if !holds_haul && !miner_clearing {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if !holds_haul {'))
PY
