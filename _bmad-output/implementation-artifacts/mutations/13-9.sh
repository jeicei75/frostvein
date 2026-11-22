# Story 13.9: forage. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-9.sh
# One row per decision (Task 10), each killed by the test named beside it. Every anchor is a named
# literal with a count guard, so `audit-mutations.py` can see it rot.
# NOTE: the gui `--drag forage` capture cannot run headless (12.7's deferral), so row 16 is killed
# by the commands_for unit test, not by the capture.

mutation "1 the forage area is never refused (AC 3)" sim-core a_forage_area_holding_no_plant_is_refused_naming_the_bounding_rect_and_places_nothing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if !has_plant {\n            return Some(Refusal::PlaceForageArea {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && !has_plant {\n            return Some(Refusal::PlaceForageArea {\n'))
PY

mutation "2 trade(Forage) is the miner's (AC 5)" sim-core every_ripe_plant_in_the_area_has_one_forage_job_and_only_the_woodcutter_holds_it <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        JobKind::Cut | JobKind::Forage => Profession::Woodcutter,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        JobKind::Cut => Profession::Woodcutter,\n        JobKind::Forage => Profession::Miner,\n'))
PY

mutation "3 a forage job ignores berries lying on the bush (Found E)" sim-core berries_still_lying_on_a_bush_keep_it_from_a_new_forage_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            .filter(|pos| !jobs.targets.contains(pos) && !lying.contains(pos))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            .filter(|pos| !jobs.targets.contains(pos) && (lying.is_empty() || true))\n'))
PY

mutation "4 removing the area leaves its forage jobs standing (AC 3)" sim-core removing_the_area_retires_a_held_forage_job_within_one_tick <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                !area.contains(&job.target)\n                    ||'
assert s.count(old) == 1
p.write_text(s.replace(old, '                false\n                    ||'))
PY

mutation "5 a forage regrows next tick, not after REGROW_TICKS (AC 6)" sim-core a_foraged_plant_is_foraged_again_no_earlier_than_regrow_ticks_after <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        .ripe_at = tick + REGROW_TICKS;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        .ripe_at = tick + 1;\n'))
PY

mutation "6 tend_plants never removes a plant whose ground went (AC 7)" sim-core a_plant_whose_ground_goes_is_gone_with_its_job_and_the_clients_are_told <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            plants.changed.insert(*pos);\n        }\n        stands\n    });\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            plants.changed.insert(*pos);\n        }\n        true\n    });\n'))
PY

mutation "7 a forage does not mark its plant changed (AC 4, 9)" sim-core a_forage_turns_the_plant_growing_and_leaves_one_berries_on_its_cell <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        .ripe_at = tick + REGROW_TICKS;\n                    plants.changed.insert(job.target);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        .ripe_at = tick + REGROW_TICKS;\n'))
PY

mutation "8 a regrown plant is never announced (AC 6, 9)" sim-core a_regrown_plant_is_announced_on_the_tick_it_ripens <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if plant.ripe_at == tick.0 {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && plant.ripe_at == tick.0 {\n'))
PY

mutation "9 apply_delta drops the plant changes (Found F)" client-core a_plant_goes_from_ripe_to_growing_to_ripe_to_gone_and_the_forage_area_is_resent_whole <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '        for change in plants {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        for change in plants.into_iter().take(0) {\n'))
PY

mutation "10 a save without plants still loads (AC 8)" sim-core a_save_missing_a_plants_or_forage_key_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/save.rs'); s = p.read_text()
old = '    pub plants: Vec<(Pos, u64)>,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    #[serde(default)]\n    pub plants: Vec<(Pos, u64)>,\n'))
PY

mutation "11 a save without the forage area still loads (AC 8)" sim-core a_save_missing_a_plants_or_forage_key_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/save.rs'); s = p.read_text()
old = '    pub forage: Vec<Pos>,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    #[serde(default)]\n    pub forage: Vec<Pos>,\n'))
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

mutation "16 the gui forage drag sends no area (AC 10)" gui a_forage_drag_is_one_place_forage_area_over_the_surface <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '        DesignateMode::Forage => vec![Command::PlaceForageArea {\n            rects: surface.to_vec(),\n        }],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Forage => vec![],\n'))
PY

mutation "17 the gui keeps a growing bush on the ripe mesh (AC 10)" gui every_mirror_plant_has_a_bush_in_its_state_and_a_forage_flips_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            PlantState::Growing => self.bush_growing_mesh.clone(),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            PlantState::Growing => self.bush_ripe_mesh.clone(),\n'))
PY
