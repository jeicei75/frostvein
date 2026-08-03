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
# The four checks are the ones docs/technical-preferences.md and every story's
# Verification block name. The `cargo tree` probe guards AC1's architectural rule
# that `tui` must never gain a `sim-core` edge — clients hold zero game logic.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

# mise installs the toolchain outside a non-interactive shell's PATH.
[ -d "$HOME/.cargo/bin" ] && export PATH="$HOME/.cargo/bin:$PATH"

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

if [ "$fail" -ne 0 ]; then
  echo "GATE RED"
  exit 1
fi
echo "GATE GREEN"
