# Story 13.10: hungry and miserable. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-10.sh
# One row per decision (Task 9), each killed by the test named beside it. Every anchor is a named
# literal with a count guard, so `audit-mutations.py` can see it rot. Rows 32, 37 and 39 are
# Task 7's instrument REDs.

mutation "1 hunger never rises (AC 1, Q2)" sim-core hunger_starts_staggered_for_the_crew_and_each_arrival_and_rises_one_a_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        hunger.0 = hunger.0.saturating_add(1);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        hunger.0 = hunger.0.saturating_add(0);\n'))
PY

mutation "2 the setting gates nothing (AC 2, Q1)" sim-core hunger_stays_zero_with_the_setting_off <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'fn tend_hunger(ecs: &mut EcsWorld) {\n    if !ecs.resource::<HungerOn>().0 {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'fn tend_hunger(ecs: &mut EcsWorld) {\n    if false && !ecs.resource::<HungerOn>().0 {\n'))
PY

mutation "3 start hunger is not staggered (Q2)" sim-core hunger_starts_staggered_for_the_crew_and_each_arrival_and_rises_one_a_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    (hash % u64::from(EAT_AT)) as u32\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    (hash % 1) as u32\n'))
PY

mutation "4 an arrival starts fed (Q2)" sim-core hunger_starts_staggered_for_the_crew_and_each_arrival_and_rises_one_a_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        start_hunger(ecs.resource::<Seed>().0, id)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        0\n'))
PY

mutation "5 a dwarf eats only once miserable (AC 1, EAT_AT)" sim-core a_hungry_miner_leaves_his_dig_eats_a_stored_berry_and_the_dig_is_done_later <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if hunger < EAT_AT {\n            continue;\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if hunger < MISERABLE_AT {\n            continue;\n        }\n'))
PY

mutation "6 a carrier is interrupted (AC 1, empty hands)" sim-core a_hungry_forager_delivers_his_load_before_he_eats_and_the_eaten_load_is_gone <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if ecs\n            .get::<Carrying>(entity)\n            .is_some_and(|carrying| carrying.0.is_some())\n        {\n            continue;\n        }\n        if (tick + u64::from(id.0)).is_multiple_of(RETRY_COOLDOWN) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if (tick + u64::from(id.0)).is_multiple_of(RETRY_COOLDOWN) {\n'))
PY

mutation "7 a miserable dwarf claims trade work (AC 3, can_try)" sim-core a_miserable_miner_lets_go_of_his_dig_and_claims_no_trade_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            && !wretched.contains(&id)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "8 a miserable dwarf keeps his trade job (AC 3)" sim-core a_miserable_miner_lets_go_of_his_dig_and_claims_no_trade_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if miserable(hunger) && held.is_some() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && miserable(hunger) && held.is_some() {\n'))
PY

mutation "9 a marked bush is eaten before its job stands (AC 3)" sim-core with_no_stored_food_a_hungry_dwarf_eats_an_unmarked_bush_and_never_a_marked_one <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    && !designations.contains_key(*pos)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "10 a bush another job names is offered (AC 5)" sim-core two_hungry_dwarves_and_one_berry_one_eats_it_and_the_other_finds_a_bush <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    && !jobs.targets.contains(*pos)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "11 an unheld meal is never retired (AC 5)" sim-core an_eat_job_nobody_holds_is_retired_on_the_next_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if food && holder.is_some() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if food || holder.is_none() {\n'))
PY

mutation "12 a meal is the miner's trade (AC 5, trade None)" sim-core a_trade_change_keeps_the_meal <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        JobKind::Eat { .. } | JobKind::EatPlant => None,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        JobKind::Eat { .. } | JobKind::EatPlant => Some(Profession::Miner),\n'))
PY

mutation "13 a meal eats no berry (AC 1, count)" sim-core a_hungry_miner_leaves_his_dig_eats_a_stored_berry_and_the_dig_is_done_later <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                count.0 = count.0.saturating_sub(1);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                count.0 = count.0.saturating_sub(0);\n'))
PY

mutation "14 an item eaten to 0 stays (AC 1, despawn)" sim-core a_hungry_forager_delivers_his_load_before_he_eats_and_the_eaten_load_is_gone <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                if count.0 == 0 {\n                    ecs.despawn(stone);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                if false {\n                    ecs.despawn(stone);\n'))
PY

mutation "15 a stored meal does not feed (AC 1, reset)" sim-core a_hungry_miner_leaves_his_dig_eats_a_stored_berry_and_the_dig_is_done_later <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    stale = true;\n                }\n                ecs.get_mut::<Hunger>(entity)\n                    .expect("every dwarf has a hunger")\n                    .0 = 0;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    stale = true;\n                }\n                ecs.get_mut::<Hunger>(entity)\n                    .expect("every dwarf has a hunger")\n                    .0 += 0;\n'))
PY

mutation "16 a bush meal does not feed (AC 1, reset)" sim-core with_no_stored_food_a_hungry_dwarf_eats_an_unmarked_bush_and_never_a_marked_one <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if ate {\n                ecs.get_mut::<Hunger>(entity)\n                    .expect("every dwarf has a hunger")\n                    .0 = 0;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if ate {\n                ecs.get_mut::<Hunger>(entity)\n                    .expect("every dwarf has a hunger")\n                    .0 += 0;\n'))
PY

mutation "17 a bush is preferred to stored berries (Q4/Q7)" sim-core two_hungry_dwarves_and_one_berry_one_eats_it_and_the_other_finds_a_bush <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    nearest(food).or_else(|| nearest(bushes()))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    nearest(bushes()).or_else(|| nearest(food))\n'))
PY

mutation "18 retire_meals is not scheduled (AC 5)" sim-core an_eat_job_nobody_holds_is_retired_on_the_next_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            advance_goals,\n            retire_meals,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            advance_goals,\n'))
PY

mutation "19 removing a pile keeps its meals (AC 6)" sim-core removing_the_pile_ends_a_meal_of_its_berries_at_once <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        retire_meals(&mut self.ecs);\n        None\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        None\n    }\n'))
PY

mutation "20 the setting is not saved (AC 6)" sim-core a_world_saved_mid_meal_matches_the_never_saved_world <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            hunger: self.ecs.resource::<HungerOn>().0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            hunger: false,\n'))
PY

mutation "21 a dwarf's hunger is not saved (AC 6)" sim-core a_world_saved_mid_meal_matches_the_never_saved_world <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    hunger: hunger.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    hunger: 0,\n'))
PY

mutation "22 a load forgets each dwarf's hunger (AC 6)" sim-core a_world_saved_mid_meal_matches_the_never_saved_world <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    Hunger(dwarf.hunger),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    Hunger(0),\n'))
PY

mutation "23 a meal nobody holds loads (AC 6)" simd a_meal_nobody_holds_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            ) && !claimed_job_ids.contains(&job.id)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            ) && false\n'))
PY

mutation "24 a meal off the pile loads (AC 6)" simd a_meal_of_no_stored_berries_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    if !(berries_ids.contains(&item) && stored_items.contains(&item)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    if !berries_ids.contains(&item) {\n'))
PY

mutation "25 a meal of a stone loads (AC 6)" simd a_meal_of_no_stored_berries_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    if !(berries_ids.contains(&item) && stored_items.contains(&item)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    if !stored_items.contains(&item) {\n'))
PY

mutation "26 two meals on one item load (AC 6)" simd two_meals_on_one_item_or_bush_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    if !seen_eat_items.insert(item) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    if false && !seen_eat_items.insert(item) {\n'))
PY

mutation "27 a bush meal does not share the job targets (AC 6)" simd two_meals_on_one_item_or_bush_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                sim_core::JobKind::EatPlant => {\n                    if !seen_job_targets.insert(job.target) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                sim_core::JobKind::EatPlant => {\n                    if false && !seen_job_targets.insert(job.target) {\n'))
PY

mutation "28 a bush meal over no ripe plant loads (AC 6)" simd a_bush_meal_over_no_ripe_plant_or_a_forage_job_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                    if !ripe_plants.contains(&job.target) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    if false {\n'))
PY

mutation "29 hunger on a world with hunger off loads (AC 6)" simd hunger_on_a_world_with_hunger_off_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if !save.hunger && dwarf.hunger > 0 {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false {\n'))
PY

mutation "30 the fresh game is not hungry (Q1)" simd a_fresh_game_is_hungry_and_a_plain_world_is_not <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '    generate_world(sim_core::DEFAULT_SEED, dims, crew, true, true)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    generate_world(sim_core::DEFAULT_SEED, dims, crew, true, false)\n'))
PY

mutation "31 a seed start ignores hunger (Q1)" simd a_seed_start_may_turn_hunger_on_and_a_save_start_may_not <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '                    hunger: hunger.unwrap_or(false),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    hunger: false,\n'))
PY

mutation "32 the end line counts no meal (AC 10, RED)" simd the_end_line_counts_meals_and_a_bush_meal_is_not_a_forage <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '            eaten += fed.len();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            eaten += 0;\n'))
PY

mutation "33 a bush meal counts as a forage (AC 10)" simd the_end_line_counts_meals_and_a_bush_meal_is_not_a_forage <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '                - fed.iter().filter(|id| at_bush.contains(id)).count();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                - 0;\n'))
PY

mutation "34 the wire says nobody is miserable (AC 7)" simd each_dwarf_carries_his_own_needs_and_an_emitter_none <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '                .map(|(_, miserable)| *miserable),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .map(|_| false),\n'))
PY

mutation "35 the wire says nobody is hungry (AC 7)" simd each_dwarf_carries_his_own_needs_and_an_emitter_none <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '                .map(|(_, hungry)| *hungry),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .map(|_| false),\n'))
PY

mutation "36 a miserable dwarf keeps his tunic (AC 7, Q6)" tui a_miserable_dwarf_is_drawn_cold_and_the_roster_and_status_name_the_needs <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '                            MISERY_COLOUR\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                            dwarf_colour(identity.colour)\n'))
PY

mutation "37 the status line ignores misery (AC 10, RED)" tui the_status_line_counts_a_dwarf_turning_miserable_under_no_color <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '        let miserable = needing(|entity| entity.miserable);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let miserable = needing(|_| None);\n'))
PY

mutation "38 the roster names no need (AC 7, Q6)" tui a_miserable_dwarf_is_drawn_cold_and_the_roster_and_status_name_the_needs <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '                .chain(\n                    need.map(|need| format!(" {need}"))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .chain(\n                    need.filter(|_| false).map(|need| format!(" {need}"))\n'))
PY

mutation "39 the slump is never registered (AC 10, RED)" gui a_miserable_dwarfs_armature_slumps_with_the_flag_and_his_entity_does_not_move <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '                crate::project::slump_miserable_dwarf,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "40 the misery line never prints a change (AC 7)" gui a_misery_line_is_reported_once_per_change_and_the_panel_names_the_need <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            .filter(|(id, miserable)| before.get(id).is_some_and(|old| old != *miserable))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .filter(|_| false)\n'))
PY

mutation "41 the slump does not sink (Q5)" gui a_miserable_dwarfs_armature_slumps_with_the_flag_and_his_entity_does_not_move <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            transform.translation.y -= SLUMP_SINK / METRES_TO_CELLS;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "42 the slump never clears (Q5)" gui a_miserable_dwarfs_armature_slumps_with_the_flag_and_his_entity_does_not_move <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        } else {\n            transform.rotation = rest;\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        }\n'))
PY
