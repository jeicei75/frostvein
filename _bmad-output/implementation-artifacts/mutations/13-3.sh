# Story 13.3: The pile remembers (pile-memo). Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-3.sh
# One row per decision (Task 3), each killed by the test named beside it. Rows 1-2 re-express 13.2's
# pile-memo REDs, whose hunks carry a `counters::add` context line and do not apply here.

mutation "1 the key ignores blocked (AC 2; 13.2 red-key-ignores-blocked)" sim-core a_blocked_cell_beside_the_pile_gives_the_fresh_answer <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        .filter(|p| kept.near.contains(p))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .filter(|_| false)\n'))
PY

mutation "2 near reads the zone cells only (AC 2; 13.2 red-near-zone-only)" sim-core a_blocked_cell_beside_the_pile_gives_the_fresh_answer <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                .flat_map(|&cell| side_neighbours(cell).into_iter().chain([cell]))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .copied()\n'))
PY

mutation "3 set_tile keeps the memo (AC 3)" sim-core the_pile_answer_is_kept_until_a_tile_or_the_zones_change <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        kept.memo = None;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "4 a zones change keeps the memo (AC 3)" sim-core the_pile_answer_is_kept_until_a_tile_or_the_zones_change <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            memo: None,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            memo: kept.memo.take(),\n'))
PY

mutation "5 the memo is never stored (AC 3)" sim-core the_pile_answer_is_kept_until_a_tile_or_the_zones_change <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    kept.memo = Some((key, depth.clone()));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
