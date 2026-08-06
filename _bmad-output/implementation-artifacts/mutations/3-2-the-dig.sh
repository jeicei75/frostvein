# Mutation set for story 3.2. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh

mutation "Designate ignores the diggability filter" sim-core designations_keep_only_tiles_workable_by_their_kind <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    positions()
                        .filter(|pos| match kind {
                            DesignationKind::Dig => {
                                matches!(terrain.tile(*pos), Some(Tile::Solid(_)))
                            }
                            DesignationKind::Channel => terrain.is_standable(*pos),
                        })
                        .collect()
'''
assert old in s
p.write_text(s.replace(old, '                    positions().collect()\n'))
PY

mutation "Designate ignores MAX_DESIGNATIONS" sim-core designation_budget_refuses_new_tiles_but_updates_existing_tiles_after_them <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    if designations.0.len() >= MAX_DESIGNATIONS
                        && !designations.0.contains_key(&pos)
                    {
                        continue;
                    }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "to_save drops jobs" sim-core save_round_trip_preserves_items_and_current_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            jobs,\n'
assert old in s
p.write_text(s.replace(old, '            jobs: Vec::new(),\n', 1))
PY

mutation "to_save drops items" sim-core save_round_trip_preserves_items_and_current_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            items,\n'
assert old in s
p.write_text(s.replace(old, '            items: Vec::new(),\n', 1))
PY

mutation "to_save drops current_job" sim-core save_round_trip_preserves_items_and_current_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    current_job: current_job.0.map(|job| job.0),\n'
assert old in s
p.write_text(s.replace(old, '                    current_job: None,\n'))
PY

mutation "from_save discards jobs" sim-core save_round_trip_preserves_items_and_current_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        for job in jobs {\n'
assert old in s
p.write_text(s.replace(old, '        for job in Vec::<Job>::new() {\n'))
PY

mutation "from_save discards items" sim-core save_round_trip_preserves_items_and_current_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        for (id, pos) in items {\n'
assert old in s
p.write_text(s.replace(old, '        for (id, pos) in Vec::<(u32, Pos)>::new() {\n'))
PY

mutation "from_save discards current_job" sim-core save_round_trip_preserves_items_and_current_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            let current_job = dwarf.current_job.map(JobId);\n'
assert old in s
p.write_text(s.replace(old, '            let current_job = None::<JobId>;\n'))
PY

mutation "bridge drops items from delta" simd completed_dig_streams_dirty_tile_and_item_in_the_same_delta <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '''        items: world
            .items()
            .into_iter()
            .map(|(id, pos)| protocol::Item {
                id: id.0,
                pos: pos_out(pos),
            })
            .collect(),
'''
assert s.count(old) == 2
cut = s.rfind(old)
p.write_text(s[:cut] + '        items: Vec::new(),\n' + s[cut + len(old):])
PY

mutation "items field is renamed on the wire" protocol decodes_the_documented_delta_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = '    pub items: Vec<Item>,\n'
assert s.count(old) == 2
cut = s.rfind(old)
p.write_text(s[:cut] + '    #[serde(rename = "objects")]\n    pub items: Vec<Item>,\n' + s[cut + len(old):])
PY

mutation "Item id field is renamed on the wire" protocol decodes_the_documented_wire_format <<'PY'
import pathlib
p = pathlib.Path('crates/protocol/src/lib.rs'); s = p.read_text()
old = 'pub struct Item {\n    pub id: u32,\n'
assert old in s
p.write_text(s.replace(old, 'pub struct Item {\n    #[serde(rename = "item_id")]\n    pub id: u32,\n'))
PY

mutation "crowd glyph is not drawn" tui two_dwarves_on_one_cell_draw_the_crowd_glyph <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '            framebuffer.cells[index] = if dwarf_counts.get(&index).copied().unwrap_or(0) > 1 {\n'
assert old in s
p.write_text(s.replace(old, '            framebuffer.cells[index] = if false {\n'))
PY

mutation "items draw above entities" tui items_draw_only_on_the_viewed_level_and_under_dwarves <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
item = '''    for item in &snapshot.items {
        if let Some(index) = screen_index(item.pos) {
            framebuffer.cells[index] = item_cell();
        }
    }

'''
anchor = '''    if let Some(anchor) = state.anchor {
'''
assert s.count(item) == 1 and anchor in s
s = s.replace(item, '')
s = s.replace(anchor, item + anchor)
p.write_text(s)
PY

mutation "load accepts an out-of-bounds job" simd out_of_bounds_job_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''            if !in_bounds(job.target) {
                bail!(
                    "save job target {},{},{} is outside dims {}x{}x{}",
                    job.target.x,
                    job.target.y,
                    job.target.z,
                    save.dims.x,
                    save.dims.y,
                    save.dims.z
                );
            }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load accepts an out-of-bounds item" simd out_of_bounds_item_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''        for (id, pos) in &save.items {
            if !in_bounds(*pos) {
                bail!(
                    "save item {id} position {},{},{} is outside dims {}x{}x{}",
                    pos.x,
                    pos.y,
                    pos.z,
                    save.dims.x,
                    save.dims.y,
                    save.dims.z
                );
            }
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY


mutation "Designate breaks instead of continuing when full" sim-core designation_budget_refuses_new_tiles_but_updates_existing_tiles_after_them <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    {
                        continue;
                    }
                    designations.0.insert(pos, kind);
'''
new = '''                    {
                        break;
                    }
                    designations.0.insert(pos, kind);
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "MAX_DESIGNATIONS is widened" sim-core designation_budget_refuses_new_tiles_but_updates_existing_tiles_after_them <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'const MAX_DESIGNATIONS: usize = 4096;\n'
assert old in s
p.write_text(s.replace(old, 'const MAX_DESIGNATIONS: usize = 4097;\n'))
PY

mutation "stone uses a second per-kind id counter" sim-core execute_jobs_walks_then_digs_for_exactly_five_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        let item_id = ecs.resource_mut::<IdAllocator>().allocate();\n'
assert old in s
p.write_text(s.replace(old, '        let item_id = Id(0);\n'))
PY

mutation "create_jobs creates a duplicate job every tick" sim-core designated_tiles_become_one_job_each_only_when_the_schedule_runs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old_guard = '''        if jobs.targets.contains(&target) {
            continue;
        }
'''
old_insert = '''        if self.by_id.contains_key(&job.id) || self.targets.contains(&job.target) {
            return false;
        }
'''
new_insert = '''        if self.by_id.contains_key(&job.id) {
            return false;
        }
'''
assert old_guard in s and old_insert in s
s = s.replace(old_guard, '').replace(old_insert, new_insert)
p.write_text(s)
PY

mutation "create_jobs runs during paused command intake" sim-core designated_tiles_become_one_job_each_only_when_the_schedule_runs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    designations.0.insert(pos, kind);
                }
            }
            SimCommand::CancelDesignation { .. } => {
'''
new = '''                    designations.0.insert(pos, kind);
                }
                drop(designations);
                let mut paused_schedule = Schedule::default();
                paused_schedule.add_systems(create_jobs);
                paused_schedule.run(&mut self.ecs);
            }
            SimCommand::CancelDesignation { .. } => {
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "claim_jobs walks jobs descending" sim-core claim_jobs_takes_fifo_and_skips_busy_dwarves_and_claimed_jobs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    for job in jobs.iter() {\n'
assert old in s
p.write_text(s.replace(old, '    for job in jobs.by_id.values().rev() {\n', 1))
PY

mutation "claim_jobs walks dwarves descending" sim-core claim_jobs_prefers_the_lowest_free_dwarf_id <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    dwarves.sort_by_key(|(_, id, _)| **id);\n'
assert old in s
p.write_text(s.replace(old, '    dwarves.sort_by_key(|(_, id, _)| std::cmp::Reverse(**id));\n'))
PY

mutation "claim_jobs ignores reaction delay" sim-core claim_jobs_waits_for_the_reaction_delay <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            if current.0.is_none()
                && tick.0
                    >= job
                        .created_tick
                        .saturating_add(reaction_delay(seed.0, **id, job.id))
'''
assert old in s
p.write_text(s.replace(old, '            if current.0.is_none()\n'))
PY

mutation "claim_jobs ignores retry_after" sim-core unreachable_job_stays_queued_and_retries_after_twenty_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if claimed.contains(&job.id) || tick.0 < job.retry_after {\n'
assert old in s
p.write_text(s.replace(old, '        if claimed.contains(&job.id) {\n'))
PY

mutation "claim_jobs assigns an already claimed job" sim-core claim_jobs_takes_fifo_and_skips_busy_dwarves_and_claimed_jobs <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if claimed.contains(&job.id) || tick.0 < job.retry_after {\n'
assert old in s
p.write_text(s.replace(old, '        if tick.0 < job.retry_after {\n'))
PY

mutation "reaction_delay returns a constant" sim-core reaction_delay_table_is_pinned <<'PY'
import pathlib, re
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
pattern = r'fn reaction_delay\(seed: u64, dwarf: Id, job: JobId\) -> u64 \{.*?\n\}'
new, count = re.subn(pattern, 'fn reaction_delay(_seed: u64, _dwarf: Id, _job: JobId) -> u64 {\n    5\n}', s, count=1, flags=re.S)
assert count == 1
p.write_text(new)
PY

mutation "reaction_delay drops the job id" sim-core reaction_delay_table_is_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''    for byte in job.0.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "reaction_delay drops the dwarf id" sim-core reaction_delay_table_is_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''    for byte in dwarf.0.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "A-star neighbour order is reversed" sim-core astar_horizontal_neighbour_order_is_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    const DIRECTIONS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];\n'
assert old in s
p.write_text(s.replace(old, '    const DIRECTIONS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];\n'))
PY

mutation "A-star allows z changes without a ramp" sim-core astar_crosses_only_a_ramp_backed_level_change <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            if terrain.is_standable(candidate)
                && matches!(
                    terrain.tile(Pos {
                        z: lower.z - 1,
                        ..lower
                    }),
                    Some(Tile::Ramp(_))
                )
'''
assert old in s
p.write_text(s.replace(old, '            if terrain.is_standable(candidate)\n'))
PY

mutation "A-star ignores the node cap" sim-core astar_stops_at_the_node_cap <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        if expanded >= MAX_ASTAR_NODES {
            return None;
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "MAX_ASTAR_NODES is widened" sim-core astar_stops_at_the_node_cap <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'const MAX_ASTAR_NODES: usize = 50_000;\n'
assert old in s
p.write_text(s.replace(old, 'const MAX_ASTAR_NODES: usize = 60_000;\n'))
PY

mutation "A-star ties break on insertion order" sim-core astar_ties_break_on_position_not_insertion_order <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old_open = '''    let mut open = BinaryHeap::from([Reverse((astar_heuristic(from, goals), from))]);
'''
new_open = '''    let mut open = BinaryHeap::from([Reverse((astar_heuristic(from, goals), 0_u64, from))]);
    let mut insertion_order = 0_u64;
'''
old_pop = '''    while let Some(Reverse((queued_f, current))) = open.pop() {
'''
new_pop = '''    while let Some(Reverse((queued_f, _, current))) = open.pop() {
'''
old_push = '''                open.push(Reverse((
                    next_cost + astar_heuristic(neighbour, goals),
                    neighbour,
                )));
'''
new_push = '''                insertion_order += 1;
                open.push(Reverse((
                    next_cost + astar_heuristic(neighbour, goals),
                    insertion_order,
                    neighbour,
                )));
'''
assert old_open in s and old_pop in s and old_push in s
s = s.replace(old_open, new_open).replace(old_pop, new_pop).replace(old_push, new_push)
p.write_text(s)
PY

mutation "dig sets the wrong tile" sim-core execute_jobs_walks_then_digs_for_exactly_five_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    Some(Tile::Solid(_)) => Some((job.target, Tile::Empty)),\n'
new = '''                    Some(Tile::Solid(_)) => Some((Pos {
                        x: job.target.x + 1,
                        ..job.target
                    }, Tile::Empty)),
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "dig writes terrain without set_tile" sim-core execute_jobs_walks_then_digs_for_exactly_five_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        let changed = ecs.resource_mut::<Terrain>().set_tile(changed_pos, tile);\n'
new = '''        let changed = {
            let mut terrain = ecs.resource_mut::<Terrain>();
            let index = worldgen::index(
                terrain.dims,
                changed_pos.x as u32,
                changed_pos.y as u32,
                changed_pos.z as u32,
            );
            terrain.tiles[index] = tile;
            true
        };
'''
assert old in s
p.write_text(s.replace(old, new))
PY

mutation "channel writes Empty instead of Ramp" sim-core execute_jobs_channels_a_material_preserving_ramp_and_spawns_stone <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        Some(Tile::Solid(material)) => Some((below, Tile::Ramp(material))),\n'
assert old in s
p.write_text(s.replace(old, '                        Some(Tile::Solid(_)) => Some((below, Tile::Empty)),\n'))
PY

mutation "channel loses the material" sim-core execute_jobs_channels_a_material_preserving_ramp_and_spawns_stone <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                        Some(Tile::Solid(material)) => Some((below, Tile::Ramp(material))),\n'
assert old in s
p.write_text(s.replace(old, '                        Some(Tile::Solid(_)) => Some((below, Tile::Ramp(Material::Stone))),\n'))
PY

mutation "stone is not spawned" sim-core execute_jobs_walks_then_digs_for_exactly_five_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        let item_id = ecs.resource_mut::<IdAllocator>().allocate();
        ecs.spawn((Item, item_id, job.target));
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "stone reuses an existing id" sim-core execute_jobs_walks_then_digs_for_exactly_five_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        let item_id = ecs.resource_mut::<IdAllocator>().allocate();\n'
assert old in s
p.write_text(s.replace(old, '        let item_id = Id(0);\n'))
PY

mutation "completed job is not removed" sim-core execute_jobs_walks_then_digs_for_exactly_five_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        ecs.resource_mut::<Jobs>().remove(job.id);\n'
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "completed designation is not removed" sim-core execute_jobs_walks_then_digs_for_exactly_five_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        ecs.resource_mut::<Designations>().0.remove(&job.target);\n'
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "WORK_TICKS is six" sim-core execute_jobs_walks_then_digs_for_exactly_five_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'const WORK_TICKS: u32 = 5;\n'
assert old in s
p.write_text(s.replace(old, 'const WORK_TICKS: u32 = 6;\n'))
PY

mutation "settle moves up instead of down" sim-core settle_moves_one_level_down_and_discards_the_path <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        let below = Pos {
            z: pos.z - 1,
            ..pos
        };
'''
assert old in s
p.write_text(s.replace(old, '''        let below = Pos {
            z: pos.z + 1,
            ..pos
        };
'''))
PY

mutation "settle does not clear Path" sim-core settle_moves_one_level_down_and_discards_the_path <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            ecs.entity_mut(entity).remove::<Path>();\n'
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "release skips retry cooldown" sim-core unreachable_job_stays_queued_and_retries_after_twenty_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    let retry_after = ecs.resource::<Tick>().0.saturating_add(RETRY_COOLDOWN);\n'
assert old in s
p.write_text(s.replace(old, '    let retry_after = ecs.resource::<Tick>().0;\n'))
PY

mutation "RETRY_COOLDOWN is twenty-one" sim-core unreachable_job_stays_queued_and_retries_after_twenty_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'const RETRY_COOLDOWN: u64 = 20;\n'
assert old in s
p.write_text(s.replace(old, 'const RETRY_COOLDOWN: u64 = 21;\n'))
PY

mutation "unreachable job is dropped" sim-core unreachable_job_stays_queued_and_retries_after_twenty_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''    if let Some(job) = ecs.resource_mut::<Jobs>().get_mut(job_id) {
        job.retry_after = retry_after;
    }
'''
assert old in s
p.write_text(s.replace(old, '    ecs.resource_mut::<Jobs>().remove(job_id);\n'))
PY

mutation "cancel leaves the job in place" sim-core cancelling_a_claimed_dig_releases_the_dwarf_without_touching_the_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                {
                    let mut jobs = self.ecs.resource_mut::<Jobs>();
                    for job_id in &job_ids {
                        jobs.remove(*job_id);
                    }
                }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "cancel leaves CurrentJob set" sim-core cancelling_a_claimed_dig_releases_the_dwarf_without_touching_the_tile <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                for entity in holders {
                    release_claim(&mut self.ecs, entity);
                }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "wander moves a dwarf holding a job" sim-core claim_jobs_waits_for_the_reaction_delay <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        if current_job.0.is_some() {
            continue;
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "to_save drops work progress" sim-core save_load_preserves_in_progress_work <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    work_progress: entity
                        .get::<WorkProgress>()
                        .map(|progress| progress.0)
                        .unwrap_or(0),
'''
assert old in s
p.write_text(s.replace(old, '                    work_progress: 0,\n'))
PY

mutation "from_save drops work progress" sim-core save_load_preserves_in_progress_work <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    .insert(WorkProgress(dwarf.work_progress));\n'
assert old in s
p.write_text(s.replace(old, '                    .insert(WorkProgress(0));\n'))
PY

mutation "A-star ramp heuristic overestimates" sim-core astar_prefers_the_shorter_ramp_route_over_a_flat_detour <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            horizontal.max(from.z.abs_diff(goal.z))\n'
assert old in s
p.write_text(s.replace(old, '            horizontal + from.z.abs_diff(goal.z)\n'))
PY

mutation "claimed dwarf moves before settling" sim-core claimed_dwarf_settles_before_moving_from_newly_unsupported_ground <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        if !ecs.resource::<Terrain>().is_standable(pos) {
            continue;
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load accepts duplicate item entity ids" simd duplicate_item_entity_id_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''        for (id, _) in &save.items {
            if !seen_ids.insert(*id) {
                bail!("save reuses entity id {id}");
            }
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load accepts item id at next_id" simd item_id_at_next_id_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''        if let Some(id) = seen_ids.last()
            && *id >= save.next_id
        {
            bail!(
                "save next_id {} does not exceed entity id {id}",
                save.next_id
            );
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "load accepts duplicate job ids" simd duplicate_job_id_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''            if !seen_job_ids.insert(job.id) {
                bail!("save reuses job id {}", job.id.0);
            }
'''
assert old in s
p.write_text(s.replace(old, '            seen_job_ids.insert(job.id);\n'))
PY

mutation "load accepts duplicate job targets" simd duplicate_job_target_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''            if !seen_job_targets.insert(job.target) {
                bail!(
                    "save reuses job target {},{},{}",
                    job.target.x,
                    job.target.y,
                    job.target.z
                );
            }
'''
assert old in s
p.write_text(s.replace(old, '            seen_job_targets.insert(job.target);\n'))
PY

mutation "load accepts job id at next_job_id" simd job_id_at_next_job_id_save_is_logged_and_the_daemon_keeps_ticking <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''        if let Some(id) = seen_job_ids.last()
            && id.0 >= save.next_job_id
        {
            bail!(
                "save next_job_id {} does not exceed job id {}",
                save.next_job_id,
                id.0
            );
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY
