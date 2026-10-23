#!/usr/bin/env bash
# 13.2 Task 4: the ten scale points (scale/*.json) on 4f57027 plus prototype patches.
# Usage: scale.sh [PATCH...]   (relative to this dir, or absolute). census.patch, counters.patch and
# scale.patch are applied first, then the args (an option prototype goes here).
# Env: SCALE_TARGET = cargo target dir (default: inside the scratch worktree); SCALE_ONLY = file glob (default *);
# SCALE_RUNS = bench runs per file (default 3; counts are deterministic, so 1 suffices for counts).
# Per file: the end line, the bench summary line, each run's p99, and run 1's counter totals.
set -euo pipefail

SIGNOFF=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO=$(git -C "$SIGNOFF" rev-parse --show-toplevel)
BASE=4f57027
WT=$(mktemp -d)
trap 'git -C "$REPO" worktree remove --force "$WT/src" >/dev/null 2>&1 || true; rm -rf "$WT"' EXIT

patches=("$SIGNOFF/prototypes/census.patch" "$SIGNOFF/prototypes/counters.patch" "$SIGNOFF/prototypes/scale.patch")
for p in "$@"; do
  [[ $p = /* ]] && patches+=("$p") || patches+=("$SIGNOFF/$p")
done

echo "scale stack: $BASE"
for p in "${patches[@]}"; do
  echo "scale   + $(basename "$p") $(sha256sum "$p" | cut -c1-8)"
done

git -C "$REPO" worktree add -q --detach "$WT/src" "$BASE"
cd "$WT/src"
for p in "${patches[@]}"; do git apply "$p"; done

export CARGO_BUILD_JOBS=8 CARGO_TARGET_DIR="${SCALE_TARGET:-$WT/target}"
cargo build -q --release -p simd

runs=${SCALE_RUNS:-3}
echo "== scale points (simd --bench FILE --runs $runs; scale/*.json of $SIGNOFF)"
for f in "$SIGNOFF"/scale/${SCALE_ONLY:-*}.json; do
  out=$("$CARGO_TARGET_DIR/release/simd" --bench "$f" --runs "$runs")
  name=$(grep -o 'bench [^ ]* run' <<<"$out" | head -1 | cut -d' ' -f2)
  echo "-- $name"
  grep ' end ' <<<"$out"
  grep "^bench $name: " <<<"$out"
  grep "^bench $name run " <<<"$out" | sed 's/.*run \([0-9]\)\/[0-9]:.* commands=\([0-9/]*\) mean=\([0-9.]*\) p99=\([0-9.]*\) max=\([0-9.]*\).*/run \1 commands=\2 mean=\3 p99=\4 max=\5/'
  # Run 1's counters. A* nodes expanded and route_to_nearest cells popped are kept apart; flood
  # cells are every flood's visited cells; exhausted = A* calls that hit the node budget.
  grep "^counters $name run 1/$runs " <<<"$out" | awk '
    { kind = $5; sub(/:.*/, "", kind)
      for (i = 6; i <= NF; i++) { split($i, kv, "="); v[kind "." kv[1]] += kv[2] } }
    END { printf "totals astar_calls=%d astar_nodes=%d route_calls=%d route_cells=%d route_none=%d flood_cells=%d exhausted_astar=%d share_ticks=%d\n",
            v["astar.calls"], v["astar.nodes"], v["route.calls"], v["route.cells"], v["route.none"], v["flood.cells"], v["astar.budget"], v["emitter.shares"] }'
done
