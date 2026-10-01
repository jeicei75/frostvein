# Story 12.3: a failed search's component is reused within one claim_jobs call (#132), and a
# haul's delivery leg is checked at claim time (Wolf question 2). Run alone after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-3.sh

mutation "component skip removed: every dwarf searches" sim-core unreachable_digs_never_starve_a_reachable_one <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                if components.iter().any(|component| {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                if false && components.iter().any(|component| {\n'))
PY

mutation "component skip not counted as attempted" sim-core unreachable_digs_never_starve_a_reachable_one <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                attempted = true;\n                // A component belongs to its start: it proves nothing for a dwarf outside it.\n'
assert s.count(old) == 1
s = s.replace(old, '                // A component belongs to its start: it proves nothing for a dwarf outside it.\n')
old = '                if let Some(delivery) = &delivery {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                attempted = true;\n' + old))
PY

mutation "first failed component covers every dwarf" sim-core an_unreachable_lower_id_does_not_starve_a_reachable_dwarf <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    component.contains(*pos)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    !component.is_empty()\n'))
PY

mutation "haul claimed without the delivery pre-search" sim-core a_sealed_off_pile_cell_does_not_cycle_a_stone_forever <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                if let Some(delivery) = &delivery {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                if let Some(delivery) = Option::<&BTreeSet<Pos>>::None {\n'))
PY

mutation "component skip ignores the delivery leg" sim-core a_sealed_off_pile_cell_does_not_starve_a_reachable_dig <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                            || delivery.as_ref().is_some_and(|d| d.is_disjoint(component)))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                            || false)\n'))
PY
