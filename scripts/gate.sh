#!/usr/bin/env bash
# The frostvein quality gate. Every story runs this before "done"; the pre-commit
# hook runs it on every commit (see .githooks/pre-commit).
#
# WHY THIS EXISTS: before it, "the gate is green" was whatever an agent reported.
# Story 1.1's dev record claimed a `simd` smoke assertion that did not exist. A
# script that exits non-zero makes green a fact rather than a claim.
#
# USAGE:  scripts/gate.sh
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
    echo "ok"
  else
    echo "FAILED"
    printf '%s\n' "$out" | tail -40
    fail=1
  fi
}

echo "frostvein gate"
run "cargo fmt --check" cargo fmt --check
run "cargo clippy -D warnings" cargo clippy --all-targets -- -D warnings
run "cargo test" cargo test

# Inverted: a MATCH is the failure. `tui` depends on `protocol` only.
printf '  %-28s' "tui has no sim-core edge"
if tree=$(cargo tree -p tui 2>&1); then
  if printf '%s\n' "$tree" | rg -q sim-core; then
    echo "FAILED"
    echo "    tui must depend on protocol only; sim-core edge found:"
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

run "metrics ledger tests" python3 -m unittest discover -s _bmad/scripts/tests

if [ "$fail" -ne 0 ]; then
  echo "GATE RED"
  exit 1
fi
echo "GATE GREEN"
