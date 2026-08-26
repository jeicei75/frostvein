#!/usr/bin/env python3
"""Static audit of every mutation row against the source it claims to sabotage.

WHY THIS EXISTS: a mutation table is evidence only as of its LAST RUN, and nothing re-runs an
old story's table. Later stories refactor the code earlier tables pin, the `old` literal stops
matching, and the row silently stops being evidence while the story record still says KILLED.

Measured 2026-08-22, the first time anyone checked: 29 of 326 rows across 9 tables could not
apply. `3-2-the-dig` alone had 12. Three rows in 5.4 died when `has_snow_cap` gained a Soil
exclusion; a fourth died when 6.1 added `ProjectedLight` to the emitter insert. The class had
already been caught by hand at 3.1, 6.1 and twice at 7.2 — always by someone happening to
re-run a table, never by a gate.

This catches the whole class in under a second and builds nothing, which is why it can live in
the gate next to the tests rather than behind a full mutation run (5.4's table alone takes ~11
minutes).

TWO FAILURE SHAPES, and they are reported differently on purpose:

  BROKEN  the row's literal no longer matches its target the number of times it asserts.
          The sabotage cannot be applied, so the seam is unpinned. Fails the gate.

  UNGUARDED  the row carries no `assert s.count(old) == N` at all. The house format requires
          one. Without it a stale literal makes `replace` a silent no-op: the source is
          unmodified, the test passes, and the runner reports SURVIVED — which reads as "your
          test is weak" when the truth is "your sabotage is broken". Warns only; there is a
          standing backlog of these and blocking every commit on them helps nobody.

Stdlib only, on the same reasoning as the metrics tests in gate.sh: the pre-commit hook must
not be breakable by a missing dev dependency.
"""

import ast
import pathlib
import re
import sys

MUTATIONS = pathlib.Path("_bmad-output/implementation-artifacts/mutations")

ROW = re.compile(r'^mutation "([^"]+)"', re.M)
TARGET = re.compile(r"pathlib\.Path\('([^']+)'\)")
# `old`, `old_expected`, `old_insert` … every literal a row edits against.
# ANY identifier bound to a string literal -- not just `old`. A row in 3-2 named its literal
# `item` and slipped past an `old\w*`-only pattern, so it was reported clean and then turned up
# APPLY-FAILED on a real run. Whether the literal matters is decided below, by whether the row
# actually uses it against the source.
LITERAL = re.compile(
    r"^([A-Za-z_]\w*) = ('''.*?'''|\"\"\".*?\"\"\"|'(?:[^'\\]|\\.)*'|\"(?:[^\"\\]|\\.)*\")",
    re.S | re.M,
)

# FOURTH ROT SHAPE, and the one that hid inside this very script. `LITERAL` only sees a literal
# BOUND TO A NAME. A row that passes its search text straight into `s.replace("...", "...")`
# binds nothing, so `LITERAL.findall` returned empty and the row was skipped ENTIRELY -- never
# checked, and not even counted among the unguarded. Measured 2026-08-26: 7 rows were invisible
# this way and one of them, `2-1`'s "client loop receives deltas but never applies them", had
# already rotted when its `tui` match arm was refactored. It writes the file back byte-identical,
# the test passes, the runner reports SURVIVED -- and this script printed a clean all-clear over
# the top of it. An inline anchor can never carry a count guard, so every one found here is
# checked as unguarded.
INLINE_ANCHOR = re.compile(
    r"s\.replace\(\s*('''.*?'''|\"\"\".*?\"\"\"|'(?:[^'\\]|\\.)*'|\"(?:[^\"\\]|\\.)*\")\s*,\s*('''.*?'''|\"\"\".*?\"\"\"|'(?:[^'\\]|\\.)*'|\"(?:[^\"\\]|\\.)*\")",
    re.S,
)

def rows(table_text):
    """Split a table into (name, block) pairs. A block runs to the next `mutation` line."""
    starts = [(m.start(), m.group(1)) for m in ROW.finditer(table_text)]
    for i, (pos, name) in enumerate(starts):
        end = starts[i + 1][0] if i + 1 < len(starts) else len(table_text)
        yield name, table_text[pos:end]


def audit():
    tables = sorted(MUTATIONS.glob("*.sh"))
    if not tables:
        print(f"  no mutation tables found under {MUTATIONS}")
        return 1

    total = 0
    broken = []
    unguarded = []
    orphaned = []
    crates = pathlib.Path("crates")
    sources = "\n".join(
        f.read_text() for f in crates.rglob("*.rs")
    ) if crates.exists() else ""

    for table in tables:
        text = table.read_text()
        for name, block in rows(text):
            total += 1
            # THIRD ROT SHAPE, and the quietest. A row names the test that must go red. When a
            # later story RENAMES or DELETES that test, `mutate.sh` finds nothing to run, the
            # empty run passes, and the row is reported SURVIVED — which reads as "your test is
            # weak" when the truth is "your test is gone". Found 2026-08-22: five rows named a
            # test that no longer existed, one of them the reason a repaired row still reported
            # SURVIVED after its literal was fixed.
            named = re.match(r'^mutation "[^"]+" \S+ (\S+)', block)
            if named and sources:
                bare = named.group(1).split("::")[-1]
                if not re.search(rf"\bfn {re.escape(bare)}\b", sources):
                    orphaned.append((table.name, name, bare))

            match = TARGET.search(block)
            if not match:
                continue
            target = pathlib.Path(match.group(1))
            if not target.exists():
                broken.append((table.name, name, f"target {target} no longer exists"))
                continue
            source = target.read_text()

            for var, literal in LITERAL.findall(block):
                # Only literals the row actually matches against the source can rot. One used
                # purely as replacement text is not an anchor and must not be flagged.
                if not re.search(
                    rf"s\.count\({var}\)|\b{var} in s\b|s\.replace\({var}\b", block
                ):
                    continue
                try:
                    value = ast.literal_eval(literal)
                except (ValueError, SyntaxError):
                    continue  # a computed literal; nothing static to check
                actual = source.count(value)
                guard = re.search(rf"assert s\.count\({var}\) == (\d+)", block)
                if guard:
                    want = int(guard.group(1))
                    if actual != want:
                        broken.append((
                            table.name, name,
                            f"{var} asserts {want} match(es) in {target}, source has {actual}",
                        ))
                        break
                elif re.search(rf"assert {var} in s\b", block):
                    # The older guard form. It fires at runtime like the count form, but says
                    # nothing about how MANY times the literal matches, so a row that starts
                    # matching twice silently sabotages both sites.
                    if actual == 0:
                        broken.append((
                            table.name, name,
                            f"{var} matches nothing in {target}",
                        ))
                        break
                else:
                    if actual == 0:
                        broken.append((
                            table.name, name,
                            f"{var} matches nothing in {target} and has no guard",
                        ))
                        break
                    unguarded.append((table.name, name, var))
            else:
                # Anchors passed straight to `s.replace(...)` without ever being named. Reached
                # only when the named pass found nothing to break on.
                # Applied CUMULATIVELY, in source order. A row may legitimately chain replaces
                # where a later anchor is TEXT AN EARLIER ONE INTRODUCED -- 3-1 swaps two match
                # arms through a temporary `SimCommand::SWAP` sentinel that appears nowhere in
                # pristine source. Checking each anchor against the untouched file called that a
                # broken row; carrying the intermediate text forward does not.
                staged = source
                for search, replacement in INLINE_ANCHOR.findall(block):
                    try:
                        search = ast.literal_eval(search)
                        replacement = ast.literal_eval(replacement)
                    except (ValueError, SyntaxError):
                        break  # a computed literal; the rest of the chain is unverifiable
                    if staged.count(search) == 0:
                        broken.append((
                            table.name, name,
                            f"inline s.replace() anchor matches nothing in {target} "
                            f"and cannot carry a guard",
                        ))
                        break
                    staged = staged.replace(search, replacement)
                    unguarded.append((table.name, name, "inline s.replace() anchor"))

    if orphaned:
        print(f"  {len(orphaned)} of {total} mutation rows NAME A TEST THAT NO LONGER EXISTS:")
        for table_name, row_name, test in orphaned:
            print(f"    {table_name}")
            print(f"      - {row_name}")
            print(f"          no `fn {test}` anywhere under crates/")
        print()
        print("  A row whose test is gone reports SURVIVED, not an error. Re-point it at the")
        print("  test that pins the seam today, or write the test that should.")
        print()

    if broken:
        print(f"  {len(broken)} of {total} mutation rows CANNOT APPLY:")
        current = None
        for table_name, row_name, why in broken:
            if table_name != current:
                print(f"    {table_name}")
                current = table_name
            print(f"      - {row_name}")
            print(f"          {why}")
        print()
        print("  A row that cannot apply pins NOTHING, however green its story record reads.")
        print("  Re-point it at the current source, or delete it if the seam is gone.")
        return 1

    if orphaned:
        return 1

    print(f"  {total} rows, every literal still matches its target", end="")
    if unguarded:
        print(f" ({len(unguarded)} rows carry no count guard)")
    else:
        print()
    return 0


if __name__ == "__main__":
    sys.exit(audit())
