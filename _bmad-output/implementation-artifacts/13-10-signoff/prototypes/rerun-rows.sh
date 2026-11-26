# 13.10: rows re-run after this story moved their seam (re-pointed or reshaped around), plus 13.10 row 10 re-pointed.

mutation "23b the no-throw sweep's level assert: no dwarf ever claims a job (review LOW)" sim-core a_carried_stone_is_always_set_down_beside_its_carrier <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        trade(job.kind) == Some(profession)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        false\n'))
PY

mutation "trade check removed" sim-core hauling_starts_while_the_dig_backlog_is_still_queued <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        trade(job.kind) == Some(profession)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        true\n'))
PY

mutation "trade maps Haul to Miner" sim-core each_trade_holds_only_its_own_jobs_and_the_woodcutter_wanders <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        JobKind::Haul { .. } => Some(Profession::Hauler),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        JobKind::Haul { .. } => Some(Profession::Miner),\n'))
PY

mutation "a reassignment keeps the held job" sim-core a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if held.is_some_and(|job| trade(job.kind).is_some_and(|needed| needed != profession)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && held.is_some_and(|job| trade(job.kind).is_some_and(|needed| needed != profession)) {\n'))
PY

mutation "the released job is removed from Jobs" sim-core a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        if held.is_some_and(|job| trade(job.kind).is_some_and(|needed| needed != profession)) {
            release_claim(&mut self.ecs, entity);
        }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        if let Some(job) = held.filter(|job| trade(job.kind).is_some_and(|needed| needed != profession)) {
            release_claim(&mut self.ecs, entity);
            self.ecs.resource_mut::<Jobs>().remove(job.id);
        }
'''))
PY

mutation "retry_claim instead of release_claim" sim-core a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        if held.is_some_and(|job| trade(job.kind).is_some_and(|needed| needed != profession)) {
            release_claim(&mut self.ecs, entity);
        }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        if let Some(job) = held.filter(|job| trade(job.kind).is_some_and(|needed| needed != profession)) {
            retry_claim(&mut self.ecs, entity, job.id);
        }
'''))
PY

mutation "a reassigned hauler keeps carrying" sim-core a_reassigned_hauler_puts_the_stone_down_and_another_hauler_delivers_it <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if held.is_some_and(|job| trade(job.kind).is_some_and(|needed| needed != profession)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if held.is_some_and(|job| {\n            trade(job.kind).is_some_and(|needed| needed != profession) && !matches!(job.kind, JobKind::Haul { .. })\n        }) {\n'))
PY

mutation "4 a cut is a miner's job" sim-core cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 13.9 Amendment 1: forage is the hauler's now, so the woodcutter's arm is the cut alone.
old = '        JobKind::Cut => Some(Profession::Woodcutter),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        JobKind::Cut => Some(Profession::Miner),\n'))
PY

mutation "2 trade(Forage) is the miner's (AC 5)" sim-core only_haulers_hold_forage_jobs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        JobKind::Forage => Some(Profession::Hauler),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        JobKind::Forage => Some(Profession::Miner),\n'))
PY

mutation "loaded world schedule omits wander" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-08-22: `assemble` gained designation and zone arguments, and the save
# destructure gained items and emitters, so this anchor rotted. The seam is unchanged.
old = '''            zones.into_iter().collect(),
            goals_completed,
            arrivals,
            hunger,
            plants
                .into_iter()
                .map(|(pos, ripe_at)| (pos, Plant { ripe_at }))
                .collect(),
        );
        for dwarf in dwarves {
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''            zones.into_iter().collect(),
            goals_completed,
            arrivals,
            hunger,
            plants
                .into_iter()
                .map(|(pos, ripe_at)| (pos, Plant { ripe_at }))
                .collect(),
        );
        world.schedule = Schedule::default();
        world.schedule.add_systems(advance_tick);
        for dwarf in dwarves {
'''))
PY

mutation "from_save regenerates terrain from the seed" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-08-22: `assemble` gained designation and zone arguments, and the save
# destructure gained items and emitters, so this anchor rotted. The seam is unchanged.
old = '''            emitters,
            goals_completed,
            arrivals,
            hunger,
            plants,
        } = save;
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''            emitters,
            goals_completed,
            arrivals,
            hunger,
            plants,
        } = save;
        let tiles = World::generate(seed, dims).to_save().tiles;
'''))
PY

mutation "from_save marks every tile dirty" sim-core loading_starts_with_no_dirty_tiles <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# Re-pointed 2026-08-22: `assemble` gained designation and zone arguments, and the save
# destructure gained items and emitters, so this anchor rotted. The seam is unchanged.
old = '''            zones.into_iter().collect(),
            goals_completed,
            arrivals,
            hunger,
            plants
                .into_iter()
                .map(|(pos, ripe_at)| (pos, Plant { ripe_at }))
                .collect(),
        );
        for dwarf in dwarves {
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''            zones.into_iter().collect(),
            goals_completed,
            arrivals,
            hunger,
            plants
                .into_iter()
                .map(|(pos, ripe_at)| (pos, Plant { ripe_at }))
                .collect(),
        );
        {
            let mut terrain = world.ecs.resource_mut::<Terrain>();
            let d = terrain.dims;
            for z in 0..d.z as i32 {
                for y in 0..d.y as i32 {
                    for x in 0..d.x as i32 {
                        terrain.dirty.insert(Pos { x, y, z });
                    }
                }
            }
        }
        for dwarf in dwarves {
'''))
PY

mutation "tui never draws the roster" tui the_roster_row_names_each_dwarf_in_his_colour_and_follows_an_identity_swap <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
# Re-pointed 2026-10-02 (12.4): the roster now pairs each identity with the dwarf's profession.
# Re-pointed 2026-11-26 (13.10): the roster pairs each identity with the dwarf's need too.
old = '                .identity\n                .map(|identity| (identity, entity.profession, client_core::need_text(entity)))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .identity\n                .filter(|_| false)\n                .map(|identity| (identity, entity.profession, client_core::need_text(entity)))\n'))
PY

mutation "named tui dwarf drawn as the emoji-capable smiley" tui a_named_dwarf_is_drawn_in_his_tunic_colour_and_a_nameless_one_in_his_job_colour <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
# Re-pointed 2026-11-26 (13.10): the tunic colour gives way to the misery colour.
old = "                        glyph: '☻',\n                        fg: if entity.miserable == Some(true) {\n"
assert s.count(old) == 1
p.write_text(s.replace(old, "                        glyph: '☺',\n                        fg: if entity.miserable == Some(true) {\n"))
PY

mutation "5 the scenario loader ignores dims (AC 1)" simd a_seed_start_with_dims_and_crew_builds_that_world <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = 'crate::generate_world(*seed, *dims, *crew, *arrivals, *hunger)'
assert s.count(old) == 1
p.write_text(s.replace(old, 'crate::generate_world(*seed, sim_core::Dims::DEFAULT, *crew, *arrivals, *hunger)'))
PY

mutation "the tui status drops its compass" tui status_line_reports_speed_z_and_dwarf_count <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '            "tick {}  {}  z {}/{}  dwarves {}  N up",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            "tick {}  {}  z {}/{}  dwarves {}",\n'))
PY

mutation "1 progress counts loose items too (AC 2)" sim-core goal_progress_counts_stored_items_only <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    let (stored, _) = split_stored(ecs);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let (mut stored, loose) = split_stored(ecs);\n    stored.extend(loose.iter().map(|(id, _)| *id));\n'))
PY

mutation "RemoveStockpile also clears designations" sim-core each_eraser_leaves_the_other_mark_kind_untouched <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                for pos in positions() {
                    zones.0.remove(&pos);
                }
            }
'''
new = '''                for pos in positions() {
                    zones.0.remove(&pos);
                }
                drop(zones);
                self.ecs.resource_mut::<Designations>().0.clear();
            }
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "RemoveStockpile is a no-op" sim-core each_eraser_leaves_the_other_mark_kind_untouched <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            SimCommand::RemoveStockpile { .. } => {
                let mut zones = self.ecs.resource_mut::<Zones>();
                for pos in positions() {
                    zones.0.remove(&pos);
                }
            }
'''
assert old in s
p.write_text(s.replace(old, '            SimCommand::RemoveStockpile { .. } => {}\n'))
PY

mutation "10 a bush another job names is offered (AC 5)" sim-core one_ripe_bush_feeds_one_of_two_hungry_dwarves_and_the_other_waits <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    && !jobs.targets.contains(*pos)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
