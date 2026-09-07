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

# BUILD PARALLELISM IS CAPPED BY MEMORY, NOT BY CORES, and that is the whole point of this block.
#
# Cargo defaults to one rustc per core. This devpod has 32 cores and 23 GB, which is roughly 700 MB
# per process before anything else is running -- and Bevy's crates are far hungrier than that. A
# mutation run makes it worse in two ways nothing else does: every row rebuilds, and every row EDITS
# A SOURCE FILE, which wakes rust-analyzer into a second full check against its own `target/
# flycheck0`. Two 32-way builds at once is what nearly took the machine down on 2026-09-07, during a
# 13-row table that had run fine on previous days -- the difference being branch switches across a
# `bevy` feature boundary, each of which invalidates the entire dependency graph.
#
# Derived from total memory rather than hardcoded so it stays right on a different machine. Override
# with CARGO_BUILD_JOBS if you know better than this arithmetic.
if [ -z "${CARGO_BUILD_JOBS:-}" ]; then
  mem_gb=$(awk '/MemTotal/ {print int($2 / 1024 / 1024)}' /proc/meminfo 2>/dev/null || echo 8)
  cores=$(nproc 2>/dev/null || echo 4)
  jobs=$((mem_gb / 2))
  [ "$jobs" -lt 2 ] && jobs=2
  [ "$jobs" -gt "$cores" ] && jobs="$cores"
  export CARGO_BUILD_JOBS="$jobs"
  echo "mutate.sh: capping cargo at ${CARGO_BUILD_JOBS} jobs (${mem_gb} GB / ${cores} cores)"
fi

BACKUP=$(mktemp -d)
trap 'restore_all; rm -rf "$BACKUP"' EXIT

# Snapshot every tracked source file once, so any mutation can be undone.
#
# `-m` ON THE RESTORE IS LOAD-BEARING, not tidiness (found 2026-08-23, M2-14's full re-run).
# Plain `tar -xf` restores each file's ORIGINAL mtime. Every artifact built during the run is
# therefore NEWER than the source restored after it, so cargo judges the artifact fresh and
# DOES NOT REBUILD -- leaving the LAST mutation's sabotaged binary sitting in target/ after the
# run has finished and the source is correctly back. The next `cargo test` then silently grades
# sabotaged code: a false RED that reads exactly like a regression, or -- if that sabotage was
# one the suite does not catch -- a false GREEN on a gate. Measured: after a full 15-table run,
# `cargo test -p simd` failed 1 of 18 with crates/ git-clean; `touch`ing the sources rebuilt and
# it passed 18/18, same flags, same binary name. `-m` (--touch) stamps extraction time as NOW,
# so the restored source always postdates the artifacts and cargo always rebuilds.
# The backup set must cover every tracked file a sabotage can reach, or the mutation survives the
# run ON DISK. `crates/*` alone missed `scripts/*`; `scripts/*` alone still missed `_bmad/scripts/*`,
# whose `session_tokens.py` the gate itself exercises and which the generic `py` tier can target.
backup_all() { tar -cf "$BACKUP/tree.tar" $(git ls-files 'crates/*' 'scripts/*' '_bmad/scripts/*'); }
restore_all() { [ -f "$BACKUP/tree.tar" ] && tar -xmf "$BACKUP/tree.tar"; }

# Initialized empty, not merely declared: under `set -u`, `${#NAMES[@]}` on a declared-but-unset
# array is an unbound-variable error, and this script does not `set -e`, so the empty-table guard
# below would print a diagnostic and then carry on to report success.
NAMES=()
RESULTS=()
survivors=0

# Valid tiers, resolved once. A tier is either "py" or a workspace package name; anything else is
# a TYPO, and a typo used to report a clean KILL: `cargo test -p <typo>` exits 101 with
# "did not match any packages", which is non-zero, is not "could not compile", and so fell
# straight through to the KILLED branch having run no test at all.
PACKAGES=$(rg -N '^name = "' crates/*/Cargo.toml | sed 's/.*"\(.*\)"/\1/')

# A fourth argument of `ignored` runs the test with `-- --ignored`. Without it a row naming an
# `#[ignore]`d test silently collects ZERO tests, exits 0, and reports SURVIVED -- "your test is not
# pinning what it claims" when the truth is "your test never ran". Story 10.5 hit exactly that with
# its AC10 row, which targets a test that drives the real binary and is therefore ignored by default.
mutation() {
  local name="$1" tier="$2" test="$3" mode="${4:-}"
  local script; script=$(cat)

  printf '\n=== %s ===\n' "$name"
  if [ "$tier" != "py" ] && ! printf '%s\n' "$PACKAGES" | rg -qxN -- "$tier"; then
    echo "  UNKNOWN TIER '$tier' — not \"py\" and not a workspace package; proves nothing"
    NAMES+=("$name"); RESULTS+=("BAD-TIER"); survivors=$((survivors + 1))
    return
  fi
  if ! printf '%s' "$script" | python3 -; then
    echo "  mutation script FAILED to apply — treating as a survivor"
    NAMES+=("$name"); RESULTS+=("APPLY-FAILED"); survivors=$((survivors + 1))
    restore_all
    return
  fi

  local out rc
  if [ "$tier" = "py" ]; then
    out=$(python3 -m unittest "$test" 2>&1); rc=$?
  else
    if [ "$mode" = "ignored" ]; then
      out=$(cargo test --offline -p "$tier" "$test" -- --ignored 2>&1); rc=$?
    else
      out=$(cargo test --offline -p "$tier" "$test" 2>&1); rc=$?
    fi
  fi
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
  elif [ "$tier" = "py" ] && printf '%s' "$out" | rg -qN 'SyntaxError|ImportError|ModuleNotFoundError|Failed to import test module|ERROR:.*_FailedTest'; then
    RESULTS+=("NO-COLLECT")
    survivors=$((survivors + 1))
    echo "  Python collection/import error — proves nothing, treating as a survivor"
    printf '%s\n' "$out" | rg -N 'SyntaxError|ImportError|ModuleNotFoundError|Failed to import test module|ERROR:' | head -3
  elif [ "$tier" = "py" ] && printf '%s' "$out" | rg -qN 'Ran 0 tests|OK \(skipped='; then
    # A SKIPPED test has judged nothing. Three rows in this repo target Blender-spawning tests
    # guarded by skipUnless(which("blender")); on a Blender-less machine those exit 0, miss every
    # guard above, and used to land in SURVIVED — reporting "your test is not pinning what it
    # claims" when the truth is "your test never ran". That is the false-KILL class inverted.
    RESULTS+=("NOT-RUN")
    survivors=$((survivors + 1))
    echo "  test SKIPPED or not collected — proves nothing, treating as a survivor"
    printf '%s\n' "$out" | rg -N 'skipped|Ran 0 tests' | head -3
  elif [ "$tier" != "py" ] && [ "$rc" -eq 0 ] && ! printf '%s' "$out" | rg -qN '[1-9][0-9]* passed'; then
    # The py tier has had this guard since three Blender-gated rows landed in SURVIVED; the cargo
    # tier did not, and an `#[ignore]`d target hits it the same way: every test binary reports
    # "0 passed; 0 failed; N filtered out", cargo exits 0, and the row reads SURVIVED having judged
    # nothing. Guarded on rc==0 so a genuine failure -- which also shows no passing test -- still
    # reaches the KILLED branch below.
    RESULTS+=("NOT-RUN")
    survivors=$((survivors + 1))
    echo "  cargo collected NO tests — proves nothing, treating as a survivor"
    echo "  (an #[ignore]d target needs a fourth argument: mutation \"...\" <tier> <test> ignored)"
    printf '%s\n' "$out" | rg -N 'test result|filtered out' | head -2
  elif [ "$rc" -ne 0 ]; then
    RESULTS+=("KILLED")
    printf '%s\n' "$out" | rg -N 'panicked at|AssertionError|assertion|test result: FAILED' | head -4
  else
    RESULTS+=("SURVIVED")
    survivors=$((survivors + 1))
    # `test result` is cargo's summary line and never appears in unittest output, so a py-tier
    # survivor used to print nothing at all under its banner.
    printf '%s\n' "$out" | rg -N 'test result|^Ran [0-9]+ test|^OK|^FAILED' | head -2
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
  printf 'NO-COMPILE / NO-COLLECT / APPLY-FAILED = the sabotage itself is broken and pins nothing; fix it.\n'
  exit 1
fi
printf '\nAll mutations killed.\n'
