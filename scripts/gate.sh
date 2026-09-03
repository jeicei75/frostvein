#!/usr/bin/env bash
# The frostvein quality gate. Every story runs this in full before "done"; the pre-commit
# hook runs the FAST tier on every commit and the pre-push hook runs the full gate
# (see .githooks/pre-commit and .githooks/pre-push).
#
# WHY THIS EXISTS: before it, "the gate is green" was whatever an agent reported.
# Story 1.1's dev record claimed a `simd` smoke assertion that did not exist. A
# script that exits non-zero makes green a fact rather than a claim.
#
# USAGE:  scripts/gate.sh          full gate. What a story runs before "done", and what
#                                 the pre-push hook runs. ~67s warm.
#         scripts/gate.sh --fast  everything except the daemon integration suite. What the
#                                 pre-commit hook runs. ~5s warm.
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
for arg in "$@"; do
  case "$arg" in
    --fast) FAST=1 ;;
    *) echo "unknown argument: $arg (expected --fast or nothing)" >&2; exit 2 ;;
  esac
done

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
run() {
  local name="$1"; shift
  printf '  %-28s' "$name"
  if out=$("$@" 2>&1); then
    # A SKIPPED check has judged nothing, and silent success is how a Blender-less machine reads
    # `bench tests  ok` while AC8's exit-code proof never ran. Say so instead.
    if printf '%s' "$out" | rg -qN 'skipped=|Ran 0 tests'; then
      echo "ok — WITH SKIPS (coverage hole)"
      printf '%s\n' "$out" | rg -N 'skipped=|Ran 0 tests' | head -3
    else
      echo "ok"
    fi
  else
    echo "FAILED"
    printf '%s\n' "$out" | tail -40
    fail=1
  fi
}

echo "frostvein gate"
run "cargo fmt --check" cargo fmt --check
run "cargo clippy -D warnings" cargo clippy --all-targets -- -D warnings
if [ "$FAST" -eq 1 ]; then
  # Everything except the daemon integration suite. simd's own unit tests still run
  # (--bins, 18 tests in 0.41s); only crates/simd/tests/serve.rs is deferred to push.
  run "cargo test (fast set)" cargo test --workspace --exclude simd
  run "cargo test -p simd --bins" cargo test -p simd --bins
else
  run "cargo test" cargo test
  # The pixel guards, and the ONLY checks here that look at a rendered frame. `cargo test` above
  # builds every binary, including the daemon these spawn, so they run after it and never alone.
  # `#[ignore]`d at the source so the fast tier skips them without needing to know their names;
  # named in the SKIPPED banner below so a fast pass can never be mistaken for having run them.
  # ~2 minutes: three real captures at 1280x720 through lavapipe. Story 10.7 is why they exist --
  # black quads every geometry count called healthy, a campfire glowing with its light switched
  # off, and an "after the fix" artifact that was the rejected fix.
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
  echo "GATE RED"
  exit 1
fi

if [ "$FAST" -eq 1 ]; then
  echo "GATE GREEN (FAST) -- NOT the full gate."
  echo "  SKIPPED: crates/simd/tests/serve.rs (61 daemon integration tests, ~59s)."
  echo "  SKIPPED: crates/gui/tests/pixel_guard.rs (2 rendered-frame guards, ~2m)."
  echo "  This is a COVERAGE HOLE, not a clean result. Run scripts/gate.sh with no"
  echo "  arguments before pushing, and before calling any story done."
  exit 0
fi
echo "GATE GREEN"
