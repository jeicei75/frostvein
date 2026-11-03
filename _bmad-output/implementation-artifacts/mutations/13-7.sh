# Story 13.7: the long way. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-7.sh
# Rows 1 and 2 restore `main`'s claim budget whole (per dwarf, per tick, with 12.4's sat-out rule),
# not a 50,000 cap per search: a per-search cap would hit the new `unreachable!` arm and KILL for
# the wrong reason. Row 1 is heavy (5 x 50,000 nodes a tick on 256^2, about 2-4 min in debug).
# Every anchor is a named literal with a count guard, so `audit-mutations.py` can see it rot
# (review run 1: an `edits = [(old, new, n)]` list hid both rows from it).

mutation "1 the claim budget is main's again: AC 1 on the 256^2 valley" sim-core a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old_components = '    let mut components: Vec<BTreeSet<Pos>> = Vec::new();\n'
assert s.count(old_components) == 1
s = s.replace(old_components, '    let mut budgets = vec![MAX_ASTAR_NODES; dwarves.len()];\n' + old_components)
old_assigned = '        let mut assigned = false;\n'
assert s.count(old_assigned) == 1
s = s.replace(old_assigned, old_assigned + '        let mut sat_out = false;\n')
old_loop = 'for (entity, id, pos, current, carrying, profession) in dwarves.iter_mut() {'
assert s.count(old_loop) == 1
s = s.replace(old_loop, 'for (slot, (entity, id, pos, current, carrying, profession)) in dwarves.iter_mut().enumerate() {')
old_attempted = '                attempted = true;\n'
assert s.count(old_attempted) == 1
s = s.replace(old_attempted, '                if budgets[slot] == 0 {\n                    sat_out = true;\n                    continue;\n                }\n' + old_attempted)
old_budget = 'let mut budget = terrain.tiles.len();'
assert s.count(old_budget) == 2
s = s.replace(old_budget, 'let budget = &mut budgets[slot];')
old_pass = '&mut budget)'
assert s.count(old_pass) == 2
s = s.replace(old_pass, 'budget)')
arm_d = '                        (_, true, _) => {\n                            unreachable!("a search sized to the world cannot run out")\n                        }\n'
assert s.count(arm_d) == 1
s = s.replace(arm_d, '                        (None, true, _) => continue,\n                        (Some(_), true, _) => unreachable!(),\n')
arm_g = '                    (_, true, _) => unreachable!("a search sized to the world cannot run out"),\n'
assert s.count(arm_g) == 1
s = s.replace(arm_g, '                    (None, true, _) => continue,\n                    (Some(_), true, _) => unreachable!(),\n')
old_stamp = '        if attempted && !assigned {\n'
assert s.count(old_stamp) == 1
s = s.replace(old_stamp, '        if attempted && !assigned && !sat_out {\n')
p.write_text(s)
PY

mutation "2 the claim budget is main's again: AC 3 stamps" sim-core unreachable_digs_are_stamped_in_one_flood_and_retried_after_the_cooldown <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old_components = '    let mut components: Vec<BTreeSet<Pos>> = Vec::new();\n'
assert s.count(old_components) == 1
s = s.replace(old_components, '    let mut budgets = vec![MAX_ASTAR_NODES; dwarves.len()];\n' + old_components)
old_assigned = '        let mut assigned = false;\n'
assert s.count(old_assigned) == 1
s = s.replace(old_assigned, old_assigned + '        let mut sat_out = false;\n')
old_loop = 'for (entity, id, pos, current, carrying, profession) in dwarves.iter_mut() {'
assert s.count(old_loop) == 1
s = s.replace(old_loop, 'for (slot, (entity, id, pos, current, carrying, profession)) in dwarves.iter_mut().enumerate() {')
old_attempted = '                attempted = true;\n'
assert s.count(old_attempted) == 1
s = s.replace(old_attempted, '                if budgets[slot] == 0 {\n                    sat_out = true;\n                    continue;\n                }\n' + old_attempted)
old_budget = 'let mut budget = terrain.tiles.len();'
assert s.count(old_budget) == 2
s = s.replace(old_budget, 'let budget = &mut budgets[slot];')
old_pass = '&mut budget)'
assert s.count(old_pass) == 2
s = s.replace(old_pass, 'budget)')
arm_d = '                        (_, true, _) => {\n                            unreachable!("a search sized to the world cannot run out")\n                        }\n'
assert s.count(arm_d) == 1
s = s.replace(arm_d, '                        (None, true, _) => continue,\n                        (Some(_), true, _) => unreachable!(),\n')
arm_g = '                    (_, true, _) => unreachable!("a search sized to the world cannot run out"),\n'
assert s.count(arm_g) == 1
s = s.replace(arm_g, '                    (None, true, _) => continue,\n                    (Some(_), true, _) => unreachable!(),\n')
old_stamp = '        if attempted && !assigned {\n'
assert s.count(old_stamp) == 1
s = s.replace(old_stamp, '        if attempted && !assigned && !sat_out {\n')
p.write_text(s)
PY

# Row 3 as the story wrote it: the stamp is u64::MAX. It dies at the FIRST stamps assert (120
# expected), so it cannot show the retry; row 3b keeps the first stamp and drops the retry.
mutation "3 the cooldown stamp is u64::MAX (dropped)" sim-core unreachable_digs_are_stamped_in_one_flood_and_retried_after_the_cooldown <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '.retry_after = tick.0.saturating_add(RETRY_COOLDOWN);'
assert s.count(old) == 1
p.write_text(s.replace(old, '.retry_after = u64::MAX;'))
PY

mutation "3b a stamped job is never retried" sim-core unreachable_digs_are_stamped_in_one_flood_and_retried_after_the_cooldown <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'claimed.contains(&job.id) || tick.0 < job.retry_after'
assert s.count(old) == 1
p.write_text(s.replace(old, 'claimed.contains(&job.id) || job.retry_after > 0'))
PY

# Review run 1 patches (2026-11-02). Rows 4-9 pin the patch pass's fix and its new asserts.
mutation "4 the execute_jobs re-plan is capped at MAX_ASTAR_NODES again (decision 1)" sim-core a_far_claim_is_kept_through_terrain_churn_past_the_node_cap <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                let (computed, _, _) = astar_with_budget(
                    terrain,
                    &blocked,
                    pos,
                    &work_positions,
                    &mut terrain.tiles.len(),
                );
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '                let computed = astar(terrain, &blocked, pos, &work_positions);\n'))
PY

# A designate filter that drops the pocket digs: without the 25-queued assert the test passes
# on main's budget too, because nothing sits ahead of the late dig.
mutation "5 the pocket digs are never queued" sim-core a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''    for (&target, &designation) in &designations.0 {
        if jobs.targets.contains(&target) {
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''    for (&target, &designation) in &designations.0 {
        if jobs.targets.contains(&target) || target.z == 1 {
'''))
PY

# Row 6 needs both sites: with only the claim sabotaged, the same tick's re-plan fails and
# `retry_claim` releases the wrong claim before `claims()` can see it (SURVIVED, 2026-11-02).
# Rewritten 2026-11-03 (13.7 review run 2, REWORK): when EVERY miner kept a wrong claim the late
# dig was never dug, so the older `must be dug within 300 ticks` assert killed the row and the id
# assert stayed unpinned. Now only miner Id(0) keeps one. Miners 1-4 still claim and dig the late
# dig, and `claims()` lists Id(0) first, so only `assert_eq!(job, late)` can see it.
mutation "6 miner 0 keeps an unreachable dig with an empty path while the others dig on" sim-core a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    (None, false, explored) => {
                        components.push(explored);
                        continue;
                    }
                    (_, true, _) => unreachable!('''
assert s.count(old) == 1
s = s.replace(old, '''                    (None, false, _) if id.0 == 0 => Vec::new(),
                    (None, false, explored) => {
                        components.push(explored);
                        continue;
                    }
                    (_, true, _) => unreachable!(''')
old_release = '''                let Some(computed) = computed else {
                    retry_claim(ecs, entity, job.id);
'''
assert s.count(old_release) == 1
s = s.replace(old_release, '''                let Some(computed) = computed else {
''')
p.write_text(s)
PY

mutation "7 the joined plates shrink under the node budget" sim-core unreachable_digs_are_stamped_in_one_flood_and_retried_after_the_cooldown <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                for x in 0..110 {
                    tiles[idx(x, y, 1 + 2 * plate)] = Tile::Empty;
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                for x in 0..90 {
                    tiles[idx(x, y, 1 + 2 * plate)] = Tile::Empty;
'''))
PY

# Rows 8-9: the "stamp, and go on to the next job" seam, unpinned since 12-4 rows 11 and 13-15
# were retired (review run 1, Edge Case Hunter's two mutants).
mutation "8 a failed goals search stops the dwarf loop" sim-core a_job_miner_zero_cannot_reach_goes_to_the_next_of_his_trade_and_others_still_stamp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    (None, false, explored) => {
                        components.push(explored);
                        continue;
                    }
                    (_, true, _) => unreachable!('''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                    (None, false, explored) => {
                        components.push(explored);
                        break;
                    }
                    (_, true, _) => unreachable!('''))
PY

mutation "9 the cached flood skips dwarves outside it" sim-core a_job_miner_zero_cannot_reach_goes_to_the_next_of_his_trade_and_others_still_stamp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''                    component.contains(*pos)
                        && (goals.is_disjoint(component)'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                    (goals.is_disjoint(component)'''))
PY
