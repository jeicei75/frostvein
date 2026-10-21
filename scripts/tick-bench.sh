#!/usr/bin/env bash
# Compare the sim's tick cost between two commits on the committed scenarios (story 13.1).
#
#   scripts/tick-bench.sh BASE HEAD [SCENARIO.json ...]     # RUNS=5 by default
#
# Builds each commit in its own scratch worktree, then runs every scenario (default: scenarios/*.json
# from THIS checkout, so both builds replay the same files) with `simd --bench FILE --runs 1`,
# alternating base and head RUNS times so load drift lands on both builds alike. Prints one row per
# scenario: base mean, head mean, delta, each build's own floor (max-min of its means over their
# average), and BEYOND FLOOR when |delta| exceeds the larger floor.
#
# THE VERDICT IS INFORMATIONAL. It exits 0 on a regression -- the table is the result. It exits
# non-zero only when something could not be measured: a build failed, a scenario failed to load or
# replay, a build's state diverged between its own runs, or a binary has no --bench (pre-13.1).
#
# Judge by the MEAN. At 13.1's creation same-build p99 and max swung 3-15% between identical runs,
# while a real +4% change cleared the mean's <1% floor and did not move the max at all.
# Never run it beside scripts/mutate.sh or the full gate: both move the floor.
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "usage: scripts/tick-bench.sh BASE HEAD [SCENARIO.json ...]" >&2
  exit 2
fi
repo=$(git rev-parse --show-toplevel)
base=$(git -C "$repo" rev-parse --verify "$1^{commit}")
head=$(git -C "$repo" rev-parse --verify "$2^{commit}")
shift 2
if [ $# -gt 0 ]; then
  scenarios=("$@")
else
  scenarios=("$repo"/scenarios/*.json)
fi
runs=${RUNS:-5}

scratch=$(mktemp -d)
cleanup() {
  for tree in "$scratch"/tree-*; do
    [ -d "$tree" ] && git -C "$repo" worktree remove --force "$tree" >/dev/null 2>&1
  done
  rm -rf "$scratch"
}
trap cleanup EXIT

# CARGO_TARGET_DIR does not choose the sources: cargo builds the CWD's workspace. So each build runs
# FROM its own worktree, into its own target dir (memory: CARGO_TARGET_DIR builds cwd sources).
build() {
  local sha=$1 tree="$scratch/tree-$1"
  if [ ! -x "$scratch/simd-$sha" ]; then
    git -C "$repo" worktree add --quiet --detach "$tree" "$sha"
    echo "building $sha ..." >&2
    (cd "$tree" && CARGO_TARGET_DIR="$scratch/target-$sha" cargo build --quiet --release -p simd -j 8)
    cp "$scratch/target-$sha/release/simd" "$scratch/simd-$sha"
  fi
  # A pre-13.1 simd reads --bench as a port and says so; anything else is a real answer. Captured
  # first: under pipefail the binary's own non-zero exit would hide grep's match.
  local probe
  probe=$("$scratch/simd-$sha" --bench /nonexistent --runs 1 2>&1 || true)
  if grep -q 'invalid port argument "--bench"' <<<"$probe"; then
    echo "tick-bench: $sha has no --bench (pre-13.1); pick a commit at or after 13.1's merge" >&2
    exit 1
  fi
}
build "$base"
build "$head"

echo "base $base"
echo "head $head"
echo "runs $runs per build, interleaved; load before: $(uptime | sed 's/.*load average: //')"
printf '%-22s %10s %10s %8s %10s %10s  %-6s %s\n' scenario base_ms head_ms delta base_floor head_floor state verdict

for file in "${scenarios[@]}"; do
  name=$(basename "$file" .json)
  base_means=() head_means=() base_states=() head_states=()
  for _ in $(seq "$runs"); do
    for side in base head; do
      sha=$([ "$side" = base ] && echo "$base" || echo "$head")
      if ! line=$("$scratch/simd-$sha" --bench "$file" --runs 1 | grep ' run 1/1: '); then
        echo "tick-bench: $name failed on $side ($sha)" >&2
        exit 1
      fi
      mean=$(sed -E 's/.* mean=([0-9.]+) .*/\1/' <<<"$line")
      state=$(sed -E 's/.* state=([0-9a-f]+).*/\1/' <<<"$line")
      if [ "$side" = base ]; then
        base_means+=("$mean") base_states+=("$state")
      else
        head_means+=("$mean") head_states+=("$state")
      fi
    done
  done
  for side in base head; do
    if [ "$side" = base ]; then states=("${base_states[@]}"); else states=("${head_states[@]}"); fi
    if [ "$(printf '%s\n' "${states[@]}" | sort -u | wc -l)" -ne 1 ]; then
      echo "tick-bench: $name STATE DIVERGED between $side's own runs: ${states[*]}" >&2
      exit 1
    fi
  done
  same=$([ "${base_states[0]}" = "${head_states[0]}" ] && echo same || echo differ)
  awk -v name="$name" -v same="$same" -v b="${base_means[*]}" -v h="${head_means[*]}" '
    function stats(list, out,   n, v, i, sum, lo, hi) {
      n = split(list, v, " "); sum = 0; lo = v[1] + 0; hi = v[1] + 0
      for (i = 1; i <= n; i++) { sum += v[i]; if (v[i] + 0 < lo) lo = v[i] + 0; if (v[i] + 0 > hi) hi = v[i] + 0 }
      out["mean"] = sum / n; out["floor"] = (hi - lo) / out["mean"] * 100
    }
    BEGIN {
      stats(b, bs); stats(h, hs)
      delta = (hs["mean"] - bs["mean"]) / bs["mean"] * 100
      floor = bs["floor"] > hs["floor"] ? bs["floor"] : hs["floor"]
      verdict = (delta > floor || -delta > floor) ? "BEYOND FLOOR" : "within floor"
      printf "%-22s %10.4f %10.4f %+7.1f%% %9.1f%% %9.1f%%  %-6s %s\n", name, bs["mean"], hs["mean"], delta, bs["floor"], hs["floor"], same, verdict
    }'
done
echo "load after: $(uptime | sed 's/.*load average: //')"
