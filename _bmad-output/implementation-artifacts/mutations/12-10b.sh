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

# Row 2 SURVIVED against the staircase sweep (first run, 2026-10-13): since the AC 4 ruling the
# blocks land up top, and the sweep stays 24/24 with same-level lifts. AC 6 now has its own test.
mutation "2 a loose item is lifted from the same level only" sim-core a_stone_at_a_pit_rim_is_lifted_from_the_ramp_below <<'PY'
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
# Re-pointed 2026-10-14 (review run 2 patch): the emitter test is one `on_emitter` closure for both modes.
old = '            matches!(entity.kind, EntityKind::Torch | EntityKind::Campfire) && entity.pos == tile\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            matches!(entity.kind, EntityKind::Torch | EntityKind::Campfire) && entity.pos == [tile[0], tile[1], -1]\n'))
PY

mutation "6 the Task 4 fix is reverted (no idle chain shift)" sim-core a_dead_end_miner_behind_idle_woodcutters_finishes_its_dig <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-10-14 (12.10b review patch, #194): the idle chain is now tried inside the exit's
# `or_else`; reverting it means it never runs and its shift is never tried.
old = '            if make_room_down_the_idle_chain(ecs, blocked, occupied, (pos, next), &holder_tiles) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && make_room_down_the_idle_chain(ecs, blocked, occupied, (pos, next), &holder_tiles) {\n'))
PY

mutation "7 a channel's block may land below its worker" sim-core a_channel_never_lands_its_stone_below_the_worker <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        || cell.z < pos.z\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# Row 8 SURVIVED against the staircase sweep (first run, 2026-10-13; the Task 4 fix cleared the two
# delays Agent A saw stall). The 12.9 seed sweep's UNFINISHED pin kills it: marks left undone.
mutation "8 the channel's wall-in check also reads the terrain before the cut" sim-core no_dwarf_is_caged_by_stones_over_the_probe_seeds <<'PY'
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

# 12.10b review patch (#193): the post-cut check seeds its floods from the CUT terrain again, where
# the target has no step up and the rim is never a side. Re-pointed 2026-10-14 (run 2 patch, #195): `dwarves`.
mutation "10 the channel's wall-in floods start from the sides after the cut" sim-core a_channel_that_would_cut_a_pit_off_from_its_rim_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                if walls_in_a_dwarf_from(terrain, &blocked, &dwarves, job.target, target_sides) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                if walls_in_a_dwarf(terrain, &blocked, &dwarves, job.target) {\n'))
PY

# 12.10b review patch (#194), first half: the idle blocker's way out back up the line is never
# found, so it has no exit and nothing backs out.
mutation "11 no exit back up the line for an idle blocker" sim-core a_one_wide_staircase_reaches_four_levels_down_and_a_dwarf_climbs_back_for_every_delay <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            route_to_nearest(ecs.resource::<Terrain>(), blocked, next, |tile| {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            route_to_nearest(ecs.resource::<Terrain>(), blocked, next, |tile| false && {\n'))
PY

# 12.10b review patch (#194), second half: the holder never backs out, so both wait as before.
mutation "12 the holder never backs out of the line" sim-core a_one_wide_staircase_reaches_four_levels_down_and_a_dwarf_climbs_back_for_every_delay <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-10-14 (run 2 patch, #196): the back-out left the `if idle` block, one indent out.
old = '            if let Some(back_out) = back_out {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if let Some(back_out) = back_out.filter(|_| false) {\n'))
PY

# 12.10b review patch (AC 9's preview half): the surface branch skips `sim_will_keep` again. The
# arm's own test (5c, 9) cannot see a dropped call; this one drags through `preview_cells`.
mutation "13 the channel preview skips the sim filter" gui a_channel_drag_over_the_campfire_does_not_light_its_tile <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        .into_iter()\n        .filter(|tile| sim_will_keep(mirror, *tile, mode))\n        .collect(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .into_iter()\n        .collect(),\n'))
PY

# 12.10b review run 2 patch (#195): a dwarf on the target is counted on the target again, which the
# floods close, so it is in no piece and the cut that drops it into a severed pit is worked.
mutation "14 a dwarf on the channel target is not counted where it falls" sim-core a_channel_that_would_drop_the_dwarf_on_its_target_into_a_cut_off_pit_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if dwarves.remove(&job.target) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if dwarves.remove(&job.target) && false {\n'))
PY

# 12.10b review run 2 patch (#196): the holder backs out only for an IDLE blocker again, so two job
# holders head-on in a one-wide line both wait.
mutation "15 the holder backs out only for an idle blocker" sim-core job_holders_head_on_in_a_one_wide_staircase_do_not_wait_forever <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if let Some(back_out) = back_out {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if let Some(back_out) = back_out.filter(|_| idle) {\n'))
PY

# 12.10b review run 2 patch: the Stockpile arm drops its emitter rule again (the preview lights the
# fire, which the sim's `is_walkable` drops).
mutation "16 the stockpile preview lights an emitter's tile" gui a_channel_drag_over_the_campfire_does_not_light_its_tile <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        DesignateMode::Stockpile => client_core::is_standable(mirror, tile) && !on_emitter(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Stockpile => client_core::is_standable(mirror, tile),\n'))
PY

# 12.10b review run 2 patch: the Channel arm drops the sim's "no tree at z-1" rule again.
mutation "17 the channel preview lights a cell over a tree" gui the_channel_preview_skips_a_cell_standing_on_a_tree <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                && !crate::designate::is_tree_tile(mirror, [tile[0], tile[1], tile[2] - 1])\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# 12.10b review run 2 patch: `step_and_watch` is the only per-tick enforcement of No Swap (12.9 Task 0
# Q1b) and had never been shown firing. Each row must die on ITS assert, read from the KILLED lines:
# 18 on "swapped tiles" (a yielding blocker steps onto the holder's tile as the holder steps onto
# its), 19 on "share a tile" (a dwarf told to step aside stays put and the holder steps onto it).
# Row 18 SURVIVED against the #196 fixture (first run, 2026-10-14: no head-on yield there); the
# one-wide sweep's back-out cascade yields, and dies with "dwarves 3 and 4 swapped tiles at tick 278".
# Row 19 died in debug on `World::step`'s own debug_assert ("two dwarves share a tile after tick"),
# which SHADOWS step_and_watch's shared-tile assert in every debug run. In release (no debug_assert)
# the same mutant dies on step_and_watch's: "two dwarves share a tile at tick 402" (run by hand).
# 12.10b review run 3 patch: row 19 now also compiles that debug_assert out, so the debug run must
# die on step_and_watch's "share a tile at tick" line.
mutation "18 a head-on yield swaps the two dwarves" sim-core a_one_wide_staircase_reaches_four_levels_down_and_a_dwarf_climbs_back_for_every_delay <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        .expect("every dwarf has a position") = *step;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .expect("every dwarf has a position") = pos;\n'))
PY

mutation "19 a dwarf stepped aside stays on its tile" sim-core job_holders_head_on_in_a_one_wide_staircase_do_not_wait_forever <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        .expect("every dwarf has a position") = aside;\n'
assert s.count(old) == 1
s = s.replace(old, '        .expect("every dwarf has a position") = from;\n')
old = '        self.schedule.run(&mut self.ecs);\n        debug_assert!(\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.schedule.run(&mut self.ecs);\n        #[cfg(any())]\n        debug_assert!(\n'))
PY

# 12.10b review run 3 patch (#197): the channel's landing check counts a dwarf on the target on the
# target again, where it is in no piece, so the block lands on the second step's rim and seals the
# pit it falls into. 20 must die on the landing assert; 20b stubs that assert so the route-to-the-rim
# assert has to kill on its own (an earlier assert absorbs a mutation, so name the killing line).
mutation "20 a channel's landing check counts the dwarf on the target where it stands" sim-core a_channel_under_a_dwarf_does_not_land_its_block_on_the_pits_other_way_out <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = ('                            || occupied.contains(&cell)\n'
       '                            || walls_in_a_dwarf(terrain, &blocked, &dwarves, cell)\n')
assert s.count(old) == 1
p.write_text(s.replace(old, '                            || occupied.contains(&cell)\n'
                            '                            || walls_in_a_dwarf(terrain, &blocked, &occupied, cell)\n'))
PY

mutation "20b row 20 with the landing assert stubbed (the route assert kills)" sim-core a_channel_under_a_dwarf_does_not_land_its_block_on_the_pits_other_way_out <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = ('                            || occupied.contains(&cell)\n'
       '                            || walls_in_a_dwarf(terrain, &blocked, &dwarves, cell)\n')
assert s.count(old) == 1
s = s.replace(old, '                            || occupied.contains(&cell)\n'
                   '                            || walls_in_a_dwarf(terrain, &blocked, &occupied, cell)\n')
old = '        assert_ne!(\n            items[0].1,\n            at(10, FIX_Y + 1, FIX_Z),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let _ = (\n            items[0].1,\n            at(10, FIX_Y + 1, FIX_Z),\n'))
PY

# 12.10b review run 3 patch: the Dig preview drops the sim's "never a tree" rule again.
mutation "21 the dig preview lights a tree cell" gui the_dig_preview_skips_a_tree_cell <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                && !crate::designate::is_tree_tile(mirror, tile)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
