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
