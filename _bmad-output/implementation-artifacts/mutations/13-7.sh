# Story 13.7: the long way. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-7.sh
# Rows 1 and 2 restore `main`'s claim budget whole (per dwarf, per tick, with 12.4's sat-out rule),
# not a 50,000 cap per search: a per-search cap would hit the new `unreachable!` arm and KILL for
# the wrong reason. Row 1 is heavy (5 x 50,000 nodes a tick on 256^2, about 2-4 min in debug).

mutation "1 the claim budget is main's again: AC 1 on the 256^2 valley" sim-core a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
arm_d = '                        (_, true, _) => {\n                            unreachable!("a search sized to the world cannot run out")\n                        }\n'
arm_g = '                    (_, true, _) => unreachable!("a search sized to the world cannot run out"),\n'
edits = [
    ('    let mut components: Vec<BTreeSet<Pos>> = Vec::new();\n',
     '    let mut budgets = vec![MAX_ASTAR_NODES; dwarves.len()];\n    let mut components: Vec<BTreeSet<Pos>> = Vec::new();\n', 1),
    ('        let mut assigned = false;\n',
     '        let mut assigned = false;\n        let mut sat_out = false;\n', 1),
    ('for (entity, id, pos, current, carrying, profession) in dwarves.iter_mut() {',
     'for (slot, (entity, id, pos, current, carrying, profession)) in dwarves.iter_mut().enumerate() {', 1),
    ('                attempted = true;\n',
     '                if budgets[slot] == 0 {\n                    sat_out = true;\n                    continue;\n                }\n                attempted = true;\n', 1),
    ('let mut budget = terrain.tiles.len();', 'let budget = &mut budgets[slot];', 2),
    ('&mut budget)', 'budget)', 2),
    (arm_d, '                        (None, true, _) => continue,\n                        (Some(_), true, _) => unreachable!(),\n', 1),
    (arm_g, '                    (None, true, _) => continue,\n                    (Some(_), true, _) => unreachable!(),\n', 1),
    ('        if attempted && !assigned {\n', '        if attempted && !assigned && !sat_out {\n', 1),
]
for old, new, n in edits:
    assert s.count(old) == n, old
    s = s.replace(old, new)
p.write_text(s)
PY

mutation "2 the claim budget is main's again: AC 3 stamps" sim-core unreachable_digs_are_stamped_in_one_flood_and_retried_after_the_cooldown <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
arm_d = '                        (_, true, _) => {\n                            unreachable!("a search sized to the world cannot run out")\n                        }\n'
arm_g = '                    (_, true, _) => unreachable!("a search sized to the world cannot run out"),\n'
edits = [
    ('    let mut components: Vec<BTreeSet<Pos>> = Vec::new();\n',
     '    let mut budgets = vec![MAX_ASTAR_NODES; dwarves.len()];\n    let mut components: Vec<BTreeSet<Pos>> = Vec::new();\n', 1),
    ('        let mut assigned = false;\n',
     '        let mut assigned = false;\n        let mut sat_out = false;\n', 1),
    ('for (entity, id, pos, current, carrying, profession) in dwarves.iter_mut() {',
     'for (slot, (entity, id, pos, current, carrying, profession)) in dwarves.iter_mut().enumerate() {', 1),
    ('                attempted = true;\n',
     '                if budgets[slot] == 0 {\n                    sat_out = true;\n                    continue;\n                }\n                attempted = true;\n', 1),
    ('let mut budget = terrain.tiles.len();', 'let budget = &mut budgets[slot];', 2),
    ('&mut budget)', 'budget)', 2),
    (arm_d, '                        (None, true, _) => continue,\n                        (Some(_), true, _) => unreachable!(),\n', 1),
    (arm_g, '                    (None, true, _) => continue,\n                    (Some(_), true, _) => unreachable!(),\n', 1),
    ('        if attempted && !assigned {\n', '        if attempted && !assigned && !sat_out {\n', 1),
]
for old, new, n in edits:
    assert s.count(old) == n, old
    s = s.replace(old, new)
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
