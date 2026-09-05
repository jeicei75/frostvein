#!/usr/bin/env bash
# The frostvein quality gate. Every story runs this in full before "done"; the pre-commit
# hook runs the FAST tier on every commit and the pre-push hook runs it scoped to the range
# being pushed (see .githooks/pre-commit and .githooks/pre-push).
#
# WHY THIS EXISTS: before it, "the gate is green" was whatever an agent reported.
# Story 1.1's dev record claimed a `simd` smoke assertion that did not exist. A
# script that exits non-zero makes green a fact rather than a claim.
#
# USAGE:  scripts/gate.sh          full gate. What a story runs before "done", and what the
#                                 pre-push hook runs when the push touches code.
#         scripts/gate.sh --fast  everything except the daemon integration suite and the pixel
#                                 guards. What the pre-commit hook runs.
#         scripts/gate.sh --range <base> <tip>
#                                 full gate, minus the heavy checks that range cannot move.
#                                 What the pre-push hook runs; see the scope block below.
#
# COSTS ARE NOT WRITTEN DOWN HERE ANY MORE, they are printed: every check reports its elapsed
# seconds and the run reports its total. The figures that used to sit in this block said ~67s
# for the full gate, measured 2026-08-23 -- correct that day and wrong from the moment the
# pixel guards landed at 10.7, since those cost ~2 minutes by themselves. Run it and read it.
#
# WHY TWO TIERS (2026-08-23, Wolf's ruling at the M2 retrospective): the gate was 67s on
# EVERY commit and that is clumsy enough to tempt --no-verify, which is the one outcome
# that makes the gate worthless. Measured that day: `simd/tests/serve.rs` is 61 tests and
# 58.9s -- 88% OF THE WHOLE GATE. Everything else together is ~5s (sim-core 102 tests in 4s;
# gui's 112 tests in 0.14s; fmt, clippy, the three dependency probes, the metrics tests and
# the mutation audit ~1s combined). serve.rs is slow BY NATURE, not by sloppiness: it spawns
# a real daemon, talks to it over a real socket, and waits on real wall-clock (a tick-rate
# test asserts elapsed within [1200, 4500] ms). Making it fast would mean making it fake.
# So it moves to the push boundary rather than being weakened.
#
# THE FAST TIER NAMES WHAT IT SKIPPED, AND SAYS "GATE GREEN (FAST)" RATHER THAN "GATE GREEN".
# That is deliberate and load-bearing: this project's standing rule is that a check which did
# not run is a COVERAGE HOLE, never a clean result -- the same reason a timed-out review layer
# is reported as a hole. A fast-tier pass pasted into a story record must be impossible to
# mistake for the full gate.
#
# The first four checks are the ones docs/technical-preferences.md and every story's
# Verification block name. The `cargo tree` probe guards AC1's architectural rule
# that `tui` must never gain a `sim-core` edge — clients hold zero game logic.
#
# The fifth is not about the Rust product at all: it runs the metrics-ledger tests.
# Added at the Epic 2 retro (2026-08-05) because that suite had NO runner — its own
# docstring records that it "went red and stayed red, unnoticed, because nothing runs
# it" after the 2026-08-01 PRICES fix shipped without it, and every cost conclusion
# this project draws comes out of that script. It is also a forge-process `FILE`, so a
# defect here propagates to every sibling project. Stdlib unittest on purpose: no
# pytest, no venv, so the pre-commit hook cannot break on a missing dev dependency.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

FAST=0
RANGE_BASE=""
RANGE_TIP=""
while [ $# -gt 0 ]; do
  case "$1" in
    --fast) FAST=1; shift ;;
    --range)
      [ $# -ge 3 ] || { echo "--range needs a base and a tip" >&2; exit 2; }
      RANGE_BASE="$2"; RANGE_TIP="$3"; shift 3 ;;
    *) echo "unknown argument: $1 (expected --fast, --range <base> <tip>, or nothing)" >&2; exit 2 ;;
  esac
done
if [ "$FAST" -eq 1 ] && [ -n "$RANGE_BASE" ]; then
  echo "--fast and --range are mutually exclusive: --fast IS a scope decision" >&2; exit 2
fi

# mise installs the toolchain outside a non-interactive shell's PATH.
[ -d "$HOME/.cargo/bin" ] && export PATH="$HOME/.cargo/bin:$PATH"

# This folder is reachable at two different absolute paths: /workspace/projects/frostvein
# in the Nidavellir devpod, and /workspace in the frostvein devpod, which mounts it as the
# root. `target/` is shared between them, and `CARGO_BIN_EXE_*` is baked into an
# integration-test binary at COMPILE time — so artifacts built under one path make every
# test that spawns simd or tui fail `NotFound` in 0.00s under the other, while unit tests
# stay green. It reads exactly like a code regression and is not one (hit 2026-08-03).
# Rebuilding only the two binary packages is enough; a full clean is not.
ROOT_STAMP=target/.frostvein-root
if [ -f "$ROOT_STAMP" ] && [ "$(cat "$ROOT_STAMP")" != "$PWD" ]; then
  echo "  build cache came from $(cat "$ROOT_STAMP"); rebuilding simd + tui for $PWD"
  cargo clean -p simd -p tui
fi
mkdir -p target && printf '%s' "$PWD" > "$ROOT_STAMP"

fail=0
# Every check reports its own elapsed seconds, and the run reports its total. The figures in
# this file's USAGE block used to be hardcoded and went stale the moment the pixel guards
# landed -- the header said ~67s while the guards it describes cost ~2 minutes on their own.
# Same lesson as the build stamp in crates/gui/build.rs: a number the artifact reports about
# itself cannot lie, a number in a comment can, and this one did.
run() {
  local name="$1"; shift
  local t0=$SECONDS
  printf '  %-28s' "$name"
  if out=$("$@" 2>&1); then
    # A SKIPPED check has judged nothing, and silent success is how a Blender-less machine reads
    # `bench tests  ok` while AC8's exit-code proof never ran. Say so instead.
    if printf '%s' "$out" | rg -qN 'skipped=|Ran 0 tests'; then
      echo "ok — WITH SKIPS (coverage hole)  $((SECONDS - t0))s"
      printf '%s\n' "$out" | rg -N 'skipped=|Ran 0 tests' | head -3
    else
      echo "ok  $((SECONDS - t0))s"
    fi
  else
    echo "FAILED  $((SECONDS - t0))s"
    printf '%s\n' "$out" | tail -40
    fail=1
  fi
}

# WHICH HEAVY CHECKS THIS RUN NEEDS (2026-09-05). Full gate unless --range narrows it.
#
# The two expensive checks are `serve.rs` (61 daemon tests) and the pixel guards (three real
# captures). Neither can observe a change that did not happen: a push carrying nothing but
# markdown cannot move a rendered frame or a socket handshake. This repo's own process mints
# those pushes constantly -- every story closes with a separate docs-only branch and PR for two
# status lines -- and each one was paying the full bill for a verdict it could not change.
#
# WHAT IS NEVER SKIPPED: fmt, clippy, the three dependency-edge probes, the metrics ledger
# tests, the bench tests and the mutation-table audit. Together they are seconds, and the last
# three grade `_bmad-output/` and `scripts/` content -- which is exactly what a "docs-only"
# push changes. Skipping those to save a second would be the trade backwards.
#
# THE RANGE COMES FROM THE HOOK, NOT FROM `main`. See .githooks/pre-push: git hands the exact
# refs being pushed on stdin. A range computed against `main` would be wrong by default here --
# every M2 story branch is stacked (epic-5-retro-2026-08-23.md:193). The diff is taken between
# the two commits rather than over a commit walk, so a rebase or force-push still answers the
# only question that matters: what does this push ADD to the remote?
# MEASURED 2026-09-05, the first thing the new timings showed: the fast and full tiers THRASH
# each other's build artifacts. `cargo test` and `cargo test --workspace --exclude simd` do not
# share a build, so alternating them rebuilds -- the fast set read 44s immediately after a full
# run and 7s on the next, identical, run. That cost is pre-existing and predates this scoping
# (a commit is fast, a push was full, so every push paid it); scoping a docs-only push to the
# fast shape now avoids it entirely, and a code push pays exactly what it always did. NOT fixed
# here -- fixing it means changing what the tiers run, which is a separate call.
RUN_SERVE=1
RUN_PIXEL=1
SCOPE_NOTE=""
SKIP_SERVE_WHY=""
SKIP_PIXEL_WHY=""
# `assets/` is deliberately in the code set as well as the render set, which keeps the render
# set a strict subset: the pixel guards can then never be selected without the full `cargo test`
# that builds the daemon they spawn.
CODE_RE='^(crates/|assets/|Cargo\.(toml|lock)$|rust-toolchain)'
RENDER_RE='^(crates/(gui|client-core|protocol)/|assets/|Cargo\.(toml|lock)$|rust-toolchain)'
# A change to the gate or its hooks always runs everything. The one artifact that must never be
# graded by its own narrowed judgement is the thing doing the narrowing.
GATE_RE='^(scripts/gate\.sh|\.githooks/)'
if [ -n "$RANGE_BASE" ]; then
  if ! changed=$(git diff --name-only "$RANGE_BASE" "$RANGE_TIP" 2>&1); then
    echo "cannot diff $RANGE_BASE $RANGE_TIP -- running the full gate" >&2
    changed=""
    SCOPE_NOTE="scope: UNREADABLE range, full gate"
  elif printf '%s\n' "$changed" | rg -qN "$GATE_RE"; then
    SCOPE_NOTE="scope: $(printf '%s\n' "$changed" | rg -cN . || echo 0) file(s); the gate itself changed, so nothing is narrowed"
  else
    n=$(printf '%s\n' "$changed" | rg -cN . || echo 0)
    if ! printf '%s\n' "$changed" | rg -qN "$CODE_RE"; then
      RUN_SERVE=0
      SKIP_SERVE_WHY="no file under crates/ or assets/, and no Cargo manifest, changed in this push"
    fi
    if ! printf '%s\n' "$changed" | rg -qN "$RENDER_RE"; then
      RUN_PIXEL=0
      SKIP_PIXEL_WHY="nothing the client renders from changed (gui, client-core, protocol, assets)"
    fi
    SCOPE_NOTE="scope: $n file(s) in $RANGE_BASE..$RANGE_TIP"
  fi
fi

START=$SECONDS
echo "frostvein gate"
[ -n "$SCOPE_NOTE" ] && echo "  $SCOPE_NOTE"
run "cargo fmt --check" cargo fmt --check
run "cargo clippy -D warnings" cargo clippy --all-targets -- -D warnings
if [ "$FAST" -eq 1 ] || [ "$RUN_SERVE" -eq 0 ]; then
  # Everything except the daemon integration suite. simd's own unit tests still run
  # (--bins, 18 tests in 0.41s); only crates/simd/tests/serve.rs is deferred to push.
  run "cargo test (fast set)" cargo test --workspace --exclude simd
  run "cargo test -p simd --bins" cargo test -p simd --bins
else
  run "cargo test" cargo test
fi
# The pixel guards, and the ONLY checks here that look at a rendered frame. The `cargo test` arm
# above builds every binary, including the daemon these spawn, so they run after it and never
# alone -- which holds under scoping too, because the render set is a strict subset of the code
# set. `#[ignore]`d at the source so the fast tier skips them without needing to know their names;
# named in the SKIPPED banner below so a partial pass can never be mistaken for the full gate.
# By a wide margin the slowest check here -- three real captures at 1280x720 through lavapipe,
# and the run prints what they cost rather than this line guessing. Story 10.7 is why they exist --
# black quads every geometry count called healthy, a campfire glowing with its light switched
# off, and an "after the fix" artifact that was the rejected fix.
if [ "$FAST" -eq 0 ] && [ "$RUN_PIXEL" -eq 1 ]; then
  run "cargo test (pixel guards)" cargo test -p gui --test pixel_guard -- --ignored
fi

# Inverted: a MATCH is the failure. Clients depend on protocol/client-core only.
for crate in tui client-core gui; do
  printf '  %-40s' "$crate has no sim-core edge"
  if tree=$(cargo tree -p "$crate" 2>&1); then
    if printf '%s\n' "$tree" | rg -q sim-core; then
      echo "FAILED"
      echo "    $crate must not depend on sim-core; edge found:"
      printf '%s\n' "$tree" | rg -n sim-core | head -5
      fail=1
    else
      echo "ok"
    fi
  else
    echo "FAILED"
    printf '%s\n' "$tree" | tail -10
    fail=1
  fi
done

run "metrics ledger tests" python3 -m unittest discover -s _bmad/scripts/tests
run "bench tests" python3 -m unittest discover -s scripts/tests

# The sixth check is about EVIDENCE, not the product. A mutation table is evidence only as of its
# last run, and nothing re-runs an old story's table — so later stories quietly refactor the code
# earlier tables pin, the literal stops matching, and the row stops being evidence while the story
# record still reads KILLED. Measured the first time anyone looked (2026-08-22): 29 of 326 rows
# across 9 tables could not apply, one of them dead since Epic 6. This is static and builds
# nothing, which is why it can sit in the gate; a real mutation run takes ~11 minutes per table.
run "mutation tables still apply" python3 scripts/audit-mutations.py

# Disk hygiene, and ONLY at the push boundary. The gate is what CREATES this garbage -- a
# gui-touching round links ~4 GB of test binaries with full debuginfo and cargo never GCs a
# stale hash -- so this is where the reaping belongs; a reminder elsewhere is what let 29 GB of
# stale `gui`, `headless` and `capture` binaries pile up by 2026-08-28. It is NOT in the fast
# tier on purpose: any reap that actually deletes something costs the next build a relink
# (measured: 28-43s against 9s warm), and Wolf's two-tier ruling is that a clumsy pre-commit
# gate tempts --no-verify, which makes the gate worthless. On a tree with nothing older than a
# week this deletes nothing and costs ~0.3s. Never allowed to fail the gate: it is hygiene, not
# a check, and a full disk is not a quality verdict.
if [ "$FAST" -eq 0 ]; then
  scripts/reap-build-caches.sh --auto || echo "  (build-cache reap skipped; not a gate failure)"
fi

if [ "$fail" -ne 0 ]; then
  echo "GATE RED  $((SECONDS - START))s"
  exit 1
fi

if [ "$FAST" -eq 1 ]; then
  echo "GATE GREEN (FAST) -- NOT the full gate.  $((SECONDS - START))s"
  echo "  SKIPPED: crates/simd/tests/serve.rs (61 daemon integration tests)."
  echo "  SKIPPED: crates/gui/tests/pixel_guard.rs (2 rendered-frame guards)."
  echo "  This is a COVERAGE HOLE, not a clean result. Run scripts/gate.sh with no"
  echo "  arguments before pushing, and before calling any story done."
  exit 0
fi

# A scoped run is still a run with holes in it, and it says so in the same words the fast tier
# uses. The difference is that each hole names the reason it could not matter, so a reader can
# check the reasoning rather than take the skip on trust -- and "GATE GREEN" unqualified stays
# reserved for the run that actually did everything.
if [ "$RUN_SERVE" -eq 0 ] || [ "$RUN_PIXEL" -eq 0 ]; then
  echo "GATE GREEN (SCOPED) -- NOT the full gate.  $((SECONDS - START))s"
  [ "$RUN_SERVE" -eq 0 ] && echo "  SKIPPED: crates/simd/tests/serve.rs -- $SKIP_SERVE_WHY."
  [ "$RUN_PIXEL" -eq 0 ] && echo "  SKIPPED: crates/gui/tests/pixel_guard.rs -- $SKIP_PIXEL_WHY."
  echo "  Scoped by the pushed range, NOT by judgement about what is worth checking."
  echo "  Before calling any story done, run scripts/gate.sh with no arguments."
  exit 0
fi
echo "GATE GREEN  $((SECONDS - START))s"
