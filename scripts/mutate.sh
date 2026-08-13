#!/usr/bin/env bash
# Batched mutation testing: apply a sabotage, run the ONE test that must die, restore,
# and print a results table.
#
# WHY THIS EXISTS: AGENTS.md rule 1 says a green suite is a claim and sabotage is the
# proof. Doing that by hand cost roughly three turns per mutation across Epic 1. This
# runs the whole set in one turn, and — more importantly — makes a SURVIVING mutation
# impossible to overlook. Story 2.1 shipped a review patch whose brand-new test passed
# with the fix removed; only the table caught it.
#
# USAGE:  scripts/mutate.sh <mutations-file>
#
# The mutations file is sourced and must define one `mutation` call per sabotage:
#
#     mutation "eviction no longer shuts the socket" simd my_test_name <<'PY'
#     import pathlib
#     p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
#     p.write_text(s.replace("            let _ = client.stream.shutdown(...);\n", ""))
#     PY
#
# The heredoc is python3 run from the repo root. Every tracked file is restored from a
# backup after each mutation — NOT via `git checkout`, which would destroy uncommitted
# work in progress. See _bmad-output/implementation-artifacts/mutations/ for worked sets.

set -uo pipefail

MUTATIONS="${1:?mutations file required — see the usage comment in this script}"
cd "$(dirname "$0")/.." || exit 1
export PATH="$HOME/.cargo/bin:$PATH"

BACKUP=$(mktemp -d)
trap 'restore_all; rm -rf "$BACKUP"' EXIT

# Snapshot every tracked source file once, so any mutation can be undone.
backup_all() { tar -cf "$BACKUP/tree.tar" $(git ls-files 'crates/*'); }
restore_all() { [ -f "$BACKUP/tree.tar" ] && tar -xf "$BACKUP/tree.tar"; }

# Initialized empty, not merely declared: under `set -u`, `${#NAMES[@]}` on a declared-but-unset
# array is an unbound-variable error, and this script does not `set -e`, so the empty-table guard
# below would print a diagnostic and then carry on to report success.
NAMES=()
RESULTS=()
survivors=0

mutation() {
  local name="$1" pkg="$2" test="$3"
  local script; script=$(cat)

  printf '\n=== %s ===\n' "$name"
  if ! printf '%s' "$script" | python3 -; then
    echo "  mutation script FAILED to apply — treating as a survivor"
    NAMES+=("$name"); RESULTS+=("APPLY-FAILED"); survivors=$((survivors + 1))
    restore_all
    return
  fi

  local out rc
  out=$(cargo test --offline -p "$pkg" "$test" 2>&1); rc=$?
  restore_all

  NAMES+=("$name")
  # A non-zero cargo exit is NOT enough to call a mutation killed: a sabotage that does not
  # COMPILE also exits non-zero, and it proves nothing about whether any test detects the
  # behaviour change. Story 5.3 shipped exactly that — its exposed-tile mutation deleted a
  # guard and left a bare `matches!(...)` mid-function (`expected ';', found 'NEIGHBOURS'`),
  # so the table claimed five kills while pinning four, and the unpinned one was AC13's
  # 53,365-vs-315,068-cube predicate. It printed no assertion, and a KILLED row's body is not
  # something anyone reads. Second false-green of this class in this script; see the
  # empty-table guard below for the first. A non-compiling mutation is a SURVIVOR: rewrite it
  # so it compiles and changes behaviour.
  if printf '%s' "$out" | rg -qN 'could not compile'; then
    RESULTS+=("NO-COMPILE")
    survivors=$((survivors + 1))
    echo "  mutation does NOT COMPILE — proves nothing, treating as a survivor"
    printf '%s\n' "$out" | rg -N '^error(\[|:)' | head -3
  elif [ "$rc" -ne 0 ]; then
    RESULTS+=("KILLED")
    printf '%s\n' "$out" | rg -N 'panicked at|assertion|test result: FAILED' | head -4
  else
    RESULTS+=("SURVIVED")
    survivors=$((survivors + 1))
    printf '%s\n' "$out" | rg -N 'test result' | head -2
  fi
}

backup_all
# shellcheck source=/dev/null
source "$MUTATIONS"
restore_all

printf '\n================ MUTATION RESULTS ================\n'
# A mutations file that defines NOTHING — a shell syntax error inside it, a bad path, a partial
# copy — used to print "All mutations killed." and exit 0. That is this script's own false-green:
# the one thing it exists to make impossible to overlook. Found at 3.3's review by feeding it a
# truncated file. An empty table is a failure, never a pass.
if [ "${#NAMES[@]}" -eq 0 ]; then
  printf 'NO MUTATIONS RAN. %s defined none — check it sourced cleanly.\n' "$MUTATIONS"
  exit 1
fi
for i in "${!NAMES[@]}"; do
  printf '%-60s %s\n' "${NAMES[$i]}" "${RESULTS[$i]}"
done

if [ "$survivors" -ne 0 ]; then
  printf '\n%d mutation(s) did not KILL. SURVIVED = the test is not pinning what it claims.\n' "$survivors"
  printf 'NO-COMPILE / APPLY-FAILED = the sabotage itself is broken and pins nothing; fix it.\n'
  exit 1
fi
printf '\nAll mutations killed.\n'
