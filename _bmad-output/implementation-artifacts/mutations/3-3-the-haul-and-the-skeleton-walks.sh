# Mutation set for story 3.3. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh
#
# HISTORY, kept because it is the useful part: this file first EXCLUDED two mutations the story's
# task list named -- the pick-up leg's `is_standable` and `remove::<Path>()` at pick-up -- arguing
# they were unkillable. 3.3's Acceptance Auditor rejected that and was right. The argument was
# about SCENARIO observability: an unreachable stone is unclaimable either way, and the walk always
# exhausts the path before arrival. Both fall one level down -- `work_positions` is a private fn a
# unit test can call directly, and a component assertion can look at `Path` on the pick-up tick.
# Both are mutated at the end of this file. The lesson: "no scenario can tell them apart" is not
# the same claim as "no test can".

mutation "create_haul_jobs runs with no stockpile present" sim-core no_stockpile_means_no_haul_job_at_all <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''    if !any_zone {
        return;
    }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "create_haul_jobs re-creates a job for a stone that has one" sim-core create_haul_jobs_makes_one_job_per_loose_stone_in_ascending_item_order <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        if ecs.resource::<Jobs>().haul_items.contains(&item) {
            continue;
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "create_haul_jobs walks the stones descending" sim-core create_haul_jobs_makes_one_job_per_loose_stone_in_ascending_item_order <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    for (item, pos) in loose {\n'
assert old in s
p.write_text(s.replace(old, '    for (item, pos) in loose.into_iter().rev() {\n'))
PY

mutation "haul jobs are indexed by target instead of item" sim-core jobs_index_haul_jobs_by_item_and_never_by_target <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            JobKind::Haul { item } => {
                if self.haul_items.contains(&item) {
                    return false;
                }
                self.haul_items.insert(item);
            }'''
assert old in s
new = '''            JobKind::Haul { .. } => {
                if self.targets.contains(&job.target) {
                    return false;
                }
                self.targets.insert(job.target);
            }'''
p.write_text(s.replace(old, new))
PY

mutation "a stored stone keeps its haul job" sim-core a_stockpile_placed_over_a_loose_stone_retires_its_job_and_idles_the_claimant <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        .filter(|job| matches!(job.kind, JobKind::Haul { item } if stored.contains(&item)))\n'
assert old in s
p.write_text(s.replace(old, '        .filter(|job| matches!(job.kind, JobKind::Haul { item } if !stored.contains(&item) && false))\n'))
PY

mutation "retiring a job does not release its holder" sim-core a_stockpile_placed_over_a_loose_stone_retires_its_job_and_idles_the_claimant <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        for holder in holders {
            release_claim(ecs, holder);
        }
'''
assert old in s
p.write_text(s.replace(old, '        let _ = holders;\n'))
PY

mutation "next_job_id wraps instead of saturating" sim-core next_job_id_counts_up_and_saturates_at_the_maximum <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        self.next_id = self.next_id.saturating_add(1);\n'
assert old in s
p.write_text(s.replace(old, '        self.next_id = self.next_id.wrapping_add(1);\n'))
PY

mutation "free stockpile tiles ignore standability" sim-core a_stockpile_tile_whose_floor_is_gone_is_never_a_delivery_target <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                .filter(|pos| terrain.is_standable(*pos) && !stored.contains(pos))\n'
assert old in s
p.write_text(s.replace(old, '                .filter(|pos| !stored.contains(pos))\n'))
PY

mutation "free stockpile tiles ignore stored stones" sim-core a_full_stockpile_parks_the_haul_job_until_a_free_tile_appears <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                .filter(|pos| terrain.is_standable(*pos) && !stored.contains(pos))\n'
assert old in s
p.write_text(s.replace(old, '                .filter(|pos| terrain.is_standable(*pos))\n'))
PY

mutation "the pick-up leg drops the free-tile gate" sim-core a_full_stockpile_parks_the_haul_job_until_a_free_tile_appears <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                Some(pos) if !free.is_empty() && terrain.is_standable(*pos) => {\n'
assert old in s
p.write_text(s.replace(old, '                Some(pos) if terrain.is_standable(*pos) => {\n'))
PY

mutation "the pick-up leg uses job.target instead of the live position" sim-core haul_execution_reads_the_stones_live_position_not_the_jobs_target <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            match items.get(&item) {
                Some(pos) if !free.is_empty() && terrain.is_standable(*pos) => {
                    BTreeSet::from([*pos])
                }
                _ => BTreeSet::new(),
            }'''
assert old in s
new = '''            match Some(&job.target) {
                Some(pos) if !free.is_empty() && terrain.is_standable(*pos) => {
                    BTreeSet::from([*pos])
                }
                _ => BTreeSet::new(),
            }'''
p.write_text(s.replace(old, new))
PY

mutation "a carrying dwarf is sent to the stone's tile, not the pile" sim-core a_haul_walks_picks_up_walks_and_drops_in_two_work_runs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            if carrying.is_some() {
                return free;
            }'''
assert old in s
new = '''            if carrying.is_some() {
                return BTreeSet::from([job.target]);
            }'''
p.write_text(s.replace(old, new))
PY

mutation "a haul reaches the tile-change computation" sim-core a_haul_walks_picks_up_walks_and_drops_in_two_work_runs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if let JobKind::Haul { item } = job.kind {\n'
assert old in s
p.write_text(s.replace(old, '        if let JobKind::Haul { item } = job.kind\n            && false\n        {\n'))
PY

mutation "pickup completes the job" sim-core a_haul_walks_picks_up_walks_and_drops_in_two_work_runs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    ecs.entity_mut(entity).insert(WorkProgress(0));
                    ecs.entity_mut(entity).remove::<Path>();
'''
assert old in s
p.write_text(s.replace(old, old + '                    ecs.resource_mut::<Jobs>().remove(job.id);\n                    release_claim(ecs, entity);\n'))
PY

mutation "pickup does not reset WorkProgress" sim-core a_haul_walks_picks_up_walks_and_drops_in_two_work_runs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    ecs.entity_mut(entity).insert(WorkProgress(0));\n'
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "the drop does not move the stone" sim-core a_haul_walks_picks_up_walks_and_drops_in_two_work_runs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
# The same write appears in `carry_items`; anchor on release_claim's surrounding `if let`.
old = '''        if let (Some(pos), Some(stone)) = (dropped_at, item_entity(ecs, item)) {
            *ecs.get_mut::<Pos>(stone)
                .expect("every stone has a position") = pos;
        }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the drop removes a designation at job.target" sim-core a_haul_walks_picks_up_walks_and_drops_in_two_work_runs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                Some(_) => {
                    ecs.resource_mut::<Jobs>().remove(job.id);
'''
assert old in s
p.write_text(s.replace(old, old + '                    ecs.resource_mut::<Designations>().0.remove(&job.target);\n'))
PY

mutation "carry_items is not in the schedule" sim-core a_carried_stone_tracks_its_carrier_every_tick_including_a_settle_fall <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            wander,
            carry_items,
'''
assert old in s
p.write_text(s.replace(old, '            wander,\n'))
PY

mutation "carry_items runs before settle" sim-core a_carried_stone_tracks_its_carrier_every_tick_including_a_settle_fall <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            execute_jobs,
            settle,
            wander,
            carry_items,
'''
assert old in s
p.write_text(s.replace(old, '            execute_jobs,\n            carry_items,\n            settle,\n            wander,\n'))
PY

mutation "release_claim does not drop the carried stone" sim-core release_claim_drops_the_carried_stone_at_the_dwarfs_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''    if let Some(item) = ecs.get::<Carrying>(entity).and_then(|carrying| carrying.0) {
        let dropped_at = ecs.get::<Pos>(entity).copied();
        if let (Some(pos), Some(stone)) = (dropped_at, item_entity(ecs, item)) {
            *ecs.get_mut::<Pos>(stone)
                .expect("every stone has a position") = pos;
        }
        if let Some(mut carrying) = ecs.get_mut::<Carrying>(entity) {
            carrying.0 = None;
        }
    }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "CancelDesignation cancels haul jobs too" sim-core cancelling_marks_over_a_stone_never_drops_its_haul_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    .filter(|job| matches!(job.kind, JobKind::Dig | JobKind::Channel))\n'
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "to_save drops carrying" sim-core save_round_trip_preserves_a_mid_haul_carry <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    carrying: carrying.0,\n'
assert old in s
p.write_text(s.replace(old, '                    carrying: None,\n'))
PY

mutation "from_save discards carrying" sim-core save_round_trip_preserves_a_mid_haul_carry <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    Carrying(dwarf.carrying),\n'
assert old in s
p.write_text(s.replace(old, '                    Carrying(None),\n'))
PY

mutation "load_world accepts a haul job naming an absent item" simd haul_job_naming_an_absent_item_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''                    if !item_ids.contains(&item) {
                        bail!("save haul job {} names missing item {item}", job.id.0);
                    }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load_world accepts two dwarves carrying one item" simd two_dwarves_carrying_one_item_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''                if !carried_items.insert(item) {
                    bail!("save item {item} has multiple carriers");
                }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load_world applies the matching-designation rule to haul jobs" simd a_mid_haul_save_loads_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                sim_core::JobKind::Haul { .. } => continue,\n'
assert old in s
p.write_text(s.replace(old, '                sim_core::JobKind::Haul { .. } => (sim_core::DesignationKind::Dig, "dig"),\n'))
PY

mutation "load_world does not bound haul jobs by the item count" simd more_haul_jobs_than_items_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''        if haul_jobs > save.items.len() {
            bail!(
                "save has {haul_jobs} haul jobs; limit is {} item(s)",
                save.items.len()
            );
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load_world lets a carrying dwarf hold someone else's job" simd carrying_dwarf_without_the_matching_haul_job_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''                match haul_job_for_item.get(&item) {
                    Some(job_id) if dwarf.current_job == Some(*job_id) => {}
                    _ => bail!(
                        "save dwarf {} carries item {item} without holding its haul job",
                        dwarf.id
                    ),
                }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "the carrier glyph is never drawn" tui items_draw_only_on_the_viewed_level_and_a_shared_cell_draws_the_carrier <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''            } else if item_counts.get(&index).copied().unwrap_or(0) > 0 {
                carrier_cell()
'''
assert old in s
p.write_text(s.replace(old, '''            } else if false {
                carrier_cell()
'''))
PY

mutation "the carrier glyph wins over the crowd glyph" tui two_dwarves_on_one_cell_draw_the_crowd_glyph <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''            framebuffer.cells[index] = if dwarf_counts.get(&index).copied().unwrap_or(0) > 1 {
                crowd_cell()
            } else if item_counts.get(&index).copied().unwrap_or(0) > 0 {
                carrier_cell()
'''
assert old in s
new = '''            framebuffer.cells[index] = if item_counts.get(&index).copied().unwrap_or(0) > 0 {
                carrier_cell()
            } else if dwarf_counts.get(&index).copied().unwrap_or(0) > 1 {
                crowd_cell()
'''
p.write_text(s.replace(old, new))
PY

mutation "stone counting and stone drawing use different filters" tui items_draw_only_on_the_viewed_level_and_a_shared_cell_draws_the_carrier <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''    for item in &snapshot.items {
        if let Some(index) = screen_index(item.pos) {
            framebuffer.cells[index] = item_cell();
            *item_counts.entry(index).or_insert(0_usize) += 1;
        }
    }
'''
assert old in s
new = '''    for item in &snapshot.items {
        if let Some(index) = screen_index(item.pos) {
            framebuffer.cells[index] = item_cell();
        }
    }
    for item in &snapshot.items {
        if let Some(index) = screen_index([item.pos[0], item.pos[1], state.z]) {
            *item_counts.entry(index).or_insert(0_usize) += 1;
        }
    }
'''
p.write_text(s.replace(old, new))
PY

mutation "the pick-up leg ignores standability" sim-core haul_work_positions_gate_both_legs_on_a_free_standable_pile_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                Some(pos) if !free.is_empty() && terrain.is_standable(*pos) => {\n'
assert old in s
p.write_text(s.replace(old, '                Some(pos) if !free.is_empty() => {\n'))
PY

mutation "pickup does not spend the path" sim-core pickup_sets_carrying_resets_the_work_counter_and_spends_the_path <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    ecs.entity_mut(entity).remove::<Path>();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "every stone on a zone tile counts as stored" sim-core two_carriers_racing_for_the_last_tile_do_not_leave_a_permanent_stack <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = """            if zones.contains(&pos) && occupied.insert(pos) {"""
assert old in s
p.write_text(s.replace(old, """            if zones.contains(&pos) {"""))
PY
