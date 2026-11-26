# Story 13.9: forage. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-9.sh
# One row per decision (Task 10), each killed by the test named beside it. Every anchor is a named
# literal with a count guard, so `audit-mutations.py` can see it rot.
# NOTE: the gui `--drag forage` capture cannot run headless (12.7's deferral), so row 16 is killed
# by the commands_for unit test, not by the capture.

mutation "2 trade(Forage) is the miner's (AC 5)" sim-core only_haulers_hold_forage_jobs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        JobKind::Forage => Profession::Hauler,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        JobKind::Forage => Profession::Miner,\n'))
PY

mutation "5 a forage regrows next tick, not after REGROW_TICKS (AC 6)" sim-core a_regrown_bush_is_unmarked_and_not_foraged_again_until_it_is_marked_again <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    .ripe_at = tick + REGROW_TICKS;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    .ripe_at = tick + 1;\n'))
PY

mutation "6 tend_plants never removes a plant whose ground went (AC 7)" sim-core losing_the_next_bush_mid_trip_drops_the_load <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            .filter(|pos| !terrain.is_standable(**pos))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .filter(|_| false)\n'))
PY

mutation "7 a forage does not mark its plant changed (AC 4, 9)" sim-core a_forage_turns_the_plant_growing_and_the_hauler_carries_one_berries <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                plants.changed.insert(job.target);\n            }\n            ecs.resource_mut::<Jobs>().remove(job.id);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            }\n            ecs.resource_mut::<Jobs>().remove(job.id);\n'))
PY

mutation "8 a regrown plant is never announced (AC 6, 9)" sim-core a_regrown_plant_is_announced_on_the_tick_it_ripens <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if plant.ripe_at == tick {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && plant.ripe_at == tick {\n'))
PY

mutation "9 apply_delta drops the plant changes (Found F)" client-core a_plant_goes_from_ripe_to_growing_to_ripe_to_gone <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '        for change in plants {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        for change in plants.into_iter().take(0) {\n'))
PY

mutation "10 a save without plants still loads (AC 8)" sim-core a_save_missing_a_plants_key_or_an_item_count_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/save.rs'); s = p.read_text()
old = '    pub plants: Vec<(Pos, u64)>,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    #[serde(default)]\n    pub plants: Vec<(Pos, u64)>,\n'))
PY

mutation "12 the validator takes a forage job over no plant (Found H)" simd a_forage_job_over_no_plant_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    if !plant_cells.contains(&job.target) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    if false && !plant_cells.contains(&job.target) {\n'))
PY

mutation "13 the bench never counts a forage (AC 11, 13)" simd the_end_line_counts_forages_and_stored_berries <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '                .filter(|(pos, now_ripe)| !now_ripe && ripe.contains(pos))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .filter(|(pos, now_ripe)| false && !now_ripe && ripe.contains(pos))\n'))
PY

mutation "14 food counts stored wood, not berries (AC 11, 13)" simd the_end_line_counts_forages_and_stored_berries <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            .filter(|(id, kind)| *kind == ItemKind::Berries && stored.contains(&id.0))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .filter(|(id, kind)| *kind == ItemKind::Wood && stored.contains(&id.0))\n'))
PY

mutation "15 the tui draws a growing bush as ripe (AC 9, 13)" tui the_forage_capture_shows_a_ripe_bush_turn_growing_and_its_berries_appear <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '                protocol::PlantState::Growing => growing_bush_cell(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                protocol::PlantState::Growing => ripe_bush_cell(),\n'))
PY

mutation "16 the gui forage drag sends a cut mark (AC 10)" gui a_forage_drag_is_one_single_level_designate_per_level_holding_a_ripe_bush <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '            .map(|&z| Command::Designate {\n                kind: DesignationKind::Forage,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .map(|&z| Command::Designate {\n                kind: DesignationKind::Cut,\n'))
PY

mutation "18 the bridge sends a forage mark as a cut mark (R2)" simd every_designation_kind_maps_to_its_named_wire_variant <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '        sim_core::DesignationKind::Forage => protocol::DesignationKind::Forage,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        sim_core::DesignationKind::Forage => protocol::DesignationKind::Cut,\n'))
PY

mutation "19 the bridge sends count 1 for every item (R2)" simd an_item_without_a_position_does_not_shift_later_kinds <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '            count: counts[&id],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            count: 1,\n'))
PY

mutation "20 food counts the berries items, not their counts (R2)" simd the_end_line_counts_forages_and_stored_berries <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            .map(|(id, _)| counts[&id] as usize)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .map(|(id, _)| (counts[&id] as usize).min(1))\n'))
PY

mutation "17 the gui keeps a growing bush on the ripe mesh (AC 10)" gui every_mirror_plant_has_a_bush_in_its_state_and_a_forage_flips_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            PlantState::Growing => self.bush_growing_mesh.clone(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            PlantState::Growing => self.bush_ripe_mesh.clone(),\n'))
PY

mutation "21 the tui draws a marked bush as an unmarked one (R3, AC 9)" tui a_marked_bush_is_drawn_marked_and_keeps_the_tally_complete <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '        if forage_marks.contains(&pos) {\n            continue;\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && forage_marks.contains(&pos) {\n            continue;\n        }\n'))
PY

mutation "22 the gui never shows a forage mark on its bush (R3, AC 10)" gui a_forage_mark_shows_on_its_bush_and_clears_with_the_designation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '.map(|(pos, state)| (pos, (state, forage_marks.contains(&pos))))'
assert s.count(old) == 1
p.write_text(s.replace(old, '.map(|(pos, state)| (pos, (state, forage_marks.contains(&pos) && false)))'))
PY

mutation "23 the gui forage preview lights growing bushes (R3, AC 10)" gui the_forage_preview_lights_ripe_bushes_and_not_growing_ones <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'if mirror.plant(tile) == Some(PlantState::Ripe) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if mirror.plant(tile).is_some() {'))
PY

# Amendment 1 (R1) decisions, added by the orchestrator at R4.
mutation "24 a forage drag marks growing bushes too (AC 3, Q12)" sim-core a_forage_drag_with_no_ripe_bush_is_refused_and_a_growing_bush_takes_no_mark <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                                .filter(ripe)\n                                // A channel already marked on the cell keeps it.\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                                .filter(|cell| plants.contains_key(cell))\n                                // A channel already marked on the cell keeps it.\n'))
PY

mutation "25 a forage drag on the ground misses the bush above it (AC 3)" sim-core a_forage_drag_marks_the_ripe_bush_on_its_cell_or_directly_above_a_ground_cell <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                                    [\n                                        pos,\n                                        Pos {\n                                            z: pos.z + 1,\n                                            ..pos\n                                        },\n                                    ]\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                                    [pos, pos]\n'))
PY

mutation "26 the first bush is the oldest, not the nearest (AC 5, Q9)" sim-core the_first_bush_of_a_trip_is_the_nearest_not_the_oldest <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    .find(|(_, cells)| cells.contains(&end))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    .find(|(_, cells)| !cells.is_empty())\n'))
PY

mutation "27 a trip ignores its radius (AC 4, Q10)" sim-core a_bush_beyond_ten_cells_ends_the_trip <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        .filter(|(away, _)| *away <= FORAGE_TRIP_RADIUS)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .filter(|(away, _)| *away <= FORAGE_TRIP_RADIUS * 100)\n'))
PY

mutation "28 a trip takes a sixth bush (AC 4, Q10)" sim-core a_sixth_bush_in_reach_is_not_picked_on_a_trip_of_five <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            let next = (item_count(ecs, item) < FORAGE_TRIP_BUSHES)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let next = (item_count(ecs, item) < FORAGE_TRIP_BUSHES + 1)\n'))
PY

mutation "29 a later bush adds nothing to the load (AC 4, Q11)" sim-core a_trip_of_three_bushes_lands_one_berries_item_of_count_three <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        .expect("a carried berries load has a count")\n                        .0 += 1;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .expect("a carried berries load has a count")\n                        .0 += 0;\n'))
PY

mutation "30 a picked bush keeps its mark (AC 6, Q8)" sim-core a_regrown_bush_is_unmarked_and_not_foraged_again_until_it_is_marked_again <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            remove_forage_mark(ecs, job.target);\n            if ripe {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if ripe {\n'))
PY

mutation "31 a lost bush keeps its mark (AC 7)" sim-core a_marked_bush_whose_ground_is_dug_away_takes_its_mark_with_it <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        remove_forage_mark(ecs, pos);\n        let job_ids: Vec<JobId> = ecs\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let job_ids: Vec<JobId> = ecs\n'))
PY

mutation "32 the validator takes an item count of 0 (AC 8)" simd an_item_count_of_zero_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if *count == 0 {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && *count == 0 {\n'))
PY

mutation "33 the validator takes a counted stone (AC 8)" simd a_stone_with_a_count_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if *kind != sim_core::ItemKind::Berries && *count != 1 {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && *kind != sim_core::ItemKind::Berries && *count != 1 {\n'))
PY

mutation "34 the validator takes a berries load over a trip (AC 8)" simd a_berries_count_over_a_trip_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if *count > sim_core::FORAGE_TRIP_BUSHES {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && *count > sim_core::FORAGE_TRIP_BUSHES {\n'))
PY

# 13.9 seat run 2 (Wolf: "dragging works only in the surface level it was initiated").
mutation "35 the gui forage drag marks only one level (seat run 2)" gui a_forage_drag_across_a_step_sends_single_level_marks_on_both_levels <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '        DesignateMode::Forage => ripe_levels\n            .iter()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Forage => ripe_levels\n            .iter()\n            .take(1)\n'))
PY

# 13.9 seat run 3 (Wolf: "it still refuses to mark bushes in other than initial surface"): simd drops a
# rect spanning levels, so every forage rect must be single-level.
mutation "36 the gui forage drag sends a rect spanning two levels (seat run 3)" gui a_forage_drag_across_a_step_sends_single_level_marks_on_both_levels <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '                    min: [cut_rect.min[0], cut_rect.min[1], z],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    min: [cut_rect.min[0], cut_rect.min[1], z - 1],\n'))
PY

# 13.9 review round 1, #244: simd refused a save the sim wrote mid-trip (a berries load in hand, the next
# bush's Forage job held). Positive control: a save the sim wrote; negatives: a stone, a hauled load.
mutation "37 the validator refuses a save made mid forage trip (#244)" simd the_live_load_takes_a_save_the_sim_wrote_mid_forage_trip <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                if !holds_haul && !miner_clearing && !forage_trip {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                if !holds_haul && !miner_clearing {\n'))
PY

mutation "38 a forage trip excuses a carried stone (#244)" simd a_forage_trip_carrying_stone_or_a_hauled_load_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                let forage_trip = berries_ids.contains(&item)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let forage_trip = (true || berries_ids.contains(&item))\n'))
PY

mutation "39 a forage trip excuses a load a haul job names (#244)" simd a_forage_trip_carrying_stone_or_a_hauled_load_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    && !haul_job_for_item.contains_key(&item)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    && (true || !haul_job_for_item.contains_key(&item))\n'))
PY

# 13.9 review round 2 (MED, rework of #244): the trip's job clause was unpinned, so a berries carrier
# holding no job, or a non-Forage job, loaded under a mutant. Both now refused by name.
mutation "46 a forage trip excuses a carrier holding no forage job (review r2)" simd a_forage_trip_carrying_stone_or_a_hauled_load_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    && save.jobs.iter().any(|job| {\n                        dwarf.current_job == Some(job.id.0) && job.kind == sim_core::JobKind::Forage\n'
assert s.count(old) == 1
end = '                    });\n                if !holds_haul && !miner_clearing && !forage_trip {\n'
assert s.count(end) == 1
s = s.replace(old, '                    && (true || save.jobs.iter().any(|job| {\n                        dwarf.current_job == Some(job.id.0) && job.kind == sim_core::JobKind::Forage\n')
p.write_text(s.replace(end, '                    }));\n                if !holds_haul && !miner_clearing && !forage_trip {\n'))
PY

mutation "47 a forage trip excuses a carrier holding any job (review r2)" simd a_forage_trip_carrying_stone_or_a_hauled_load_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                        dwarf.current_job == Some(job.id.0) && job.kind == sim_core::JobKind::Forage\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        dwarf.current_job == Some(job.id.0)\n'))
PY

# 13.9 review round 1 (MED): a channel marked over a bush whose Forage job stands takes the cell's mark and
# the job stays; simd must take that save, and still refuse any other mark (or none) under a Forage job.
mutation "40 the validator refuses a forage job under a channel mark (review MED)" simd the_live_load_takes_a_save_with_a_channel_marked_over_a_standing_forage_job <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = 'sim_core::DesignationKind::Forage | sim_core::DesignationKind::Channel'
assert s.count(old) == 1
p.write_text(s.replace(old, 'sim_core::DesignationKind::Forage'))
PY

mutation "41 the validator takes a forage job under any mark or none (review MED)" simd a_forage_job_over_no_forage_mark_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    if !matches!(\n                        designation_kinds.get(&job.target),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    if false && !matches!(\n                        designation_kinds.get(&job.target),\n'))
PY

# 13.9 review round 1 (LOW, same function): two hand-edit states the validator took silently.
mutation "42 the validator collapses a reused plant cell (review LOW)" simd a_reused_plant_cell_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if !plant_cells.insert(*pos) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if !plant_cells.insert(*pos) && false {\n'))
PY

mutation "43 the validator takes a forage mark over no plant (review LOW)" simd a_forage_mark_over_no_plant_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if *kind == sim_core::DesignationKind::Forage && !plant_cells.contains(pos) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && *kind == sim_core::DesignationKind::Forage && !plant_cells.contains(pos) {\n'))
PY

# 13.9 review round 1 (Wolf: "let's add it to goals in this story"): berries 20, the 4th goal, by summed counts.
mutation "44 the berries goal's target is not 20 (review)" sim-core the_berries_goal_counts_each_stored_loads_count <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    (ItemKind::Berries, 20),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    (ItemKind::Berries, 2),\n'))
PY

mutation "45 the goals count stored items, not their counts (review)" sim-core the_berries_goal_counts_each_stored_loads_count <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                entity.get::<ItemCount>().map_or(1, |count| count.0);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                1;\n'))
PY
