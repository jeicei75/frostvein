# Story 12.10b: A way down. Run alone, after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-10b.sh
# Row names follow the story's Task 7 list; 7-9 are the dev-time additions (AC 4 ruling, the
# post-cut wall-in check, the gui mirror's second half).

mutation "1 a channel is worked from the same level only" sim-core a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = ('                astar_neighbours(terrain, blocked, job.target)\n'
       '                    .into_iter()\n'
       '                    .collect()\n')
assert s.count(old) == 1
p.write_text(s.replace(old, '                side_neighbours(job.target)\n'
                            '                    .into_iter()\n'
                            '                    .filter(|candidate| is_walkable(terrain, blocked, *candidate))\n'
                            '                    .collect()\n'))
PY

mutation "2 a loose item is lifted from the same level only" sim-core a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = ('                    astar_neighbours(terrain, blocked, *pos)\n'
       '                        .into_iter()\n'
       '                        .collect()\n')
assert s.count(old) == 1
p.write_text(s.replace(old, '                    side_neighbours(*pos)\n'
                            '                        .into_iter()\n'
                            '                        .filter(|candidate| is_walkable(terrain, blocked, *candidate))\n'
                            '                        .collect()\n'))
PY

mutation "3 the base is left solid (no ramp down)" sim-core a_channel_opens_a_step_down_and_a_dwarf_walks_down_and_back <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    .set_tile(base, Tile::Ramp(material));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    .set_tile(base, Tile::Solid(material));\n'))
PY

mutation "4 the channel's item spawns on the target" sim-core execute_jobs_opens_a_step_down_and_lands_the_stone_up_top <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            landing = found;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let _ = found;\n'))
PY

mutation "5 the emitter refusal is dropped" sim-core a_channel_mark_is_refused_on_an_emitters_tile_only <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                                    || (pos.z >= 2 && !emitter_tiles.contains(pos))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                                    || (pos.z >= 2 && !emitter_tiles.is_empty())\n'))
PY

mutation "5b the z < 2 refusal is dropped" sim-core a_channel_mark_is_refused_where_z_is_below_two <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                                    || (pos.z >= 2 && !emitter_tiles.contains(pos))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                                    || !emitter_tiles.contains(pos)\n'))
PY

mutation "5c the gui preview lights an emitter's tile" gui the_channel_preview_skips_an_emitters_tile_and_cells_below_z_two <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                        && entity.pos == tile\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        && entity.pos == [tile[0], tile[1], -1]\n'))
PY

mutation "6 the Task 4 fix is reverted (no idle chain shift)" sim-core a_dead_end_miner_behind_idle_woodcutters_finishes_its_dig <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = ('            return make_room_down_the_idle_chain(\n'
       '                ecs,\n'
       '                blocked,\n'
       '                occupied,\n'
       '                (pos, next),\n'
       '                &holder_tiles,\n'
       '            );\n')
assert s.count(old) == 1
p.write_text(s.replace(old, '            let _ = make_room_down_the_idle_chain;\n            return false;\n'))
PY

mutation "7 a channel's block may land below its worker" sim-core a_channel_never_lands_its_stone_below_the_worker <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        || cell.z < pos.z\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "8 the channel's wall-in check also reads the terrain before the cut" sim-core a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = ('        if job.kind != JobKind::Channel\n'
       '            && walls_in_a_dwarf(ecs.resource::<Terrain>(), &blocked, &occupied, job.target)\n')
assert s.count(old) == 1
p.write_text(s.replace(old, '        if walls_in_a_dwarf(ecs.resource::<Terrain>(), &blocked, &occupied, job.target)\n'))
PY

mutation "9 the gui preview lights a cell below z 2" gui the_channel_preview_skips_an_emitters_tile_and_cells_below_z_two <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                && tile[2] >= 2\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
