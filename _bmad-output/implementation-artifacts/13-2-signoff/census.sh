#!/usr/bin/env bash
# 13.2 census: reproducer end lines + sweep readings on 4f57027 plus prototype patches.
# Usage: census.sh [PATCH...]   (relative to this dir, or absolute). census.patch is applied first.
# Env: CENSUS_TARGET = cargo target dir (default: inside the scratch worktree).
set -euo pipefail

SIGNOFF=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO=$(git -C "$SIGNOFF" rev-parse --show-toplevel)
BASE=4f57027
WT=$(mktemp -d)
trap 'git -C "$REPO" worktree remove --force "$WT/src" >/dev/null 2>&1 || true; rm -rf "$WT"' EXIT

patches=("$SIGNOFF/prototypes/census.patch")
for p in "$@"; do
  [[ $p = /* ]] && patches+=("$p") || patches+=("$SIGNOFF/$p")
done

echo "census stack: $BASE"
for p in "${patches[@]}"; do
  echo "census   + $(basename "$p") $(sha256sum "$p" | cut -c1-8)"
done

git -C "$REPO" worktree add -q --detach "$WT/src" "$BASE"
cd "$WT/src"
for p in "${patches[@]}"; do git apply "$p"; done

export CARGO_BUILD_JOBS=8 CARGO_TARGET_DIR="${CENSUS_TARGET:-$WT/target}"
cargo build -q --release -p simd
cargo test -q --release -p sim-core --test scenario --lib --no-run

echo "== reproducers (simd --bench FILE --runs 1; scenarios of $REPO)"
for f in "$REPO"/scenarios/*.json; do
  out=$("$CARGO_TARGET_DIR/release/simd" --bench "$f" --runs 1)
  grep ' end ' <<<"$out"
  echo "$(grep -o 'bench [^ ]* run' <<<"$out" | head -1 | cut -d' ' -f2) $(grep -o 'state=[0-9a-f]*' <<<"$out" | head -1)"
done

echo "== sweeps"
fail=0
run_tests() { # $1 = cargo target flags, rest = test names
  local flags=$1; shift
  local log="$WT/log"
  # shellcheck disable=SC2086
  cargo test --release -p sim-core $flags -- --nocapture --test-threads=2 "$@" >"$log" 2>&1 || true
  { grep '^census ' "$log" || true; } | sort
  for t in "$@"; do
    if grep -q "^test .*$t \.\.\. FAILED" "$log"; then
      echo "census $t PANICKED: $(grep -A1 "thread '.*$t' .*panicked" "$log" | tail -1)"
      fail=1
    elif ! grep -q "^test .*$t \.\.\. ok" "$log"; then
      echo "census $t MISSING (no ok/FAILED line)"
      fail=1
    fi
  done
}
run_tests "--test scenario" \
  a_stockpile_around_the_campfire_never_zones_or_receives_the_fire \
  no_dwarf_is_caged_by_stones_over_the_probe_seeds \
  a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay \
  a_one_wide_staircase_reaches_four_levels_down_and_a_dwarf_climbs_back_for_every_delay \
  job_holders_head_on_in_a_one_wide_staircase_do_not_wait_forever \
  a_pile_round_enclosed_ground_still_fills_every_cell \
  a_pile_round_ground_beside_rock_still_fills_every_cell \
  two_piles_that_ring_ground_together_still_fill_every_cell \
  census_207_four_bars_round_a_hole
run_tests "--lib" a_hole_outside_each_piles_own_box_is_still_a_pocket
exit $fail
