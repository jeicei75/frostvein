# Issue #74 sabotage table. Run alone: scripts/mutate.sh <this file>
#
# The rule blocked five cells and changed NO existing test outcome, so the new scenario test is the
# only thing standing between this fix and a silently inert one. Every row below removes the rule
# from ONE writer, because a fix applied to one movement path and not the other is the exact defect
# these rows exist to catch.

# The rule itself. If `is_walkable` stops consulting the blocked set, both writers lose it at once.
mutation "the walkability rule ignores fire entirely" sim-core a_dwarf_never_stands_in_a_fire <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    terrain.is_standable(candidate) && !blocked.contains(&candidate)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    terrain.is_standable(candidate)\n'))
PY

# WANDER only. Job routing still refuses the fire, so a test that only drove A* would stay green
# while idle dwarves strolled through the campfire -- which is what the seat actually sees, because
# dwarves are idle far more often than they are hauling.
mutation "wander forgets the rule, job routing keeps it" sim-core a_dwarf_never_stands_in_a_fire <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    && is_walkable(&terrain, &blocked, *p)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    && terrain.is_standable(*p)\n'))
PY

# The blocked set is empty, so both writers ask a question whose answer is always yes. This is the
# inert-mechanism shape: the predicate is called, threaded and tested, and decides nothing.
mutation "no cell is ever blocked" sim-core a_dwarf_never_stands_in_a_fire <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '    positions.copied().collect()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = positions;\n    BTreeSet::new()\n'))
PY
