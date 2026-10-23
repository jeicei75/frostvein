#!/usr/bin/env bash
# Compare the sim's tick cost between two commits on the committed scenarios (story 13.1).
#
#   scripts/tick-bench.sh BASE HEAD [SCENARIO.json ...]     # RUNS=5 by default
#
# Builds each commit in its own scratch worktree, then runs every scenario (default: scenarios/*.json
# from THIS checkout, so both builds replay the same files) with `simd --bench FILE --runs 1`,
# alternating base, head and a control RUNS times so load drift lands on every build alike. Prints one
# row per scenario: base mean, head mean, delta, the control's delta, each build's own floor (max-min
# of its means over their average), and BEYOND FLOOR when |delta| exceeds every floor AND |control|.
#
# THE CONTROL is base built again with its functions in another order (lld --shuffle-sections): the
# same source, another layout. Two builds differ by layout alone, and a same-binary floor cannot see
# that: at 13.1, two builds with identical plan code read plan -6.1%, -5.4% and -5.3% BEYOND FLOOR
# against floors under 2%. NOTE: the control is a LOWER bound on that noise. On that same pair it read
# about -3% (the -C metadata and codegen-unit variants about -2%), so a BEYOND FLOOR on a change near
# |control| x2 is still weak evidence between two builds.
#
# THE VERDICT IS INFORMATIONAL. It exits 0 on a regression -- the table is the result. It exits
# non-zero only when something could not be measured: a build failed, a scenario failed to load or
# replay, a build's state diverged between its own runs, or a binary has no --bench (pre-13.1).
# A scenario that cannot be measured prints a FAILED row with its cause and the rest still run, so
# one rotted save or a refused mark does not hide every other row; the exit 1 comes at the end.
#
# RUNS must be at least 3: a floor is the spread of a build's own means, and two samples are not a
# floor (one gives 0.0%, and then a binary against itself reads BEYOND FLOOR).
#
# Judge by the MEAN. At 13.1's creation same-build p99 and max swung 3-15% between identical runs,
# while a real +4% change cleared the mean's <1% floor and did not move the max at all. That <1% is a
# same-binary floor; between two builds the control above is the figure to beat.
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
if ! [[ "$runs" =~ ^[0-9]+$ ]] || [ "$runs" -lt 3 ]; then
  echo "tick-bench: RUNS=$runs: need a whole number of at least 3 (two samples are not a floor)" >&2
  exit 2
fi

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
  local sha=$1 label=$2 flags=${3:-} tree="$scratch/tree-$1"
  if [ ! -x "$scratch/simd-$label" ]; then
    [ -d "$tree" ] || git -C "$repo" worktree add --quiet --detach "$tree" "$sha"
    echo "building $label${flags:+ ($sha)} ..." >&2
    # Only the control sets RUSTFLAGS. NOTE: set, it replaces a config's build.rustflags (none here).
    (cd "$tree" && { [ -z "$flags" ] || export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }$flags"; } &&
      CARGO_TARGET_DIR="$scratch/target-$label" cargo build --quiet --release -p simd -j 8)
    cp "$scratch/target-$label/release/simd" "$scratch/simd-$label"
  fi
  # A pre-13.1 simd reads --bench as a port and says so; anything else is a real answer. Captured
  # first: under pipefail the binary's own non-zero exit would hide grep's match.
  local probe
  probe=$("$scratch/simd-$label" --bench /nonexistent --runs 1 2>&1 || true)
  if grep -q 'invalid port argument "--bench"' <<<"$probe"; then
    echo "tick-bench: $sha has no --bench (pre-13.1); pick a commit at or after 13.1's merge" >&2
    exit 1
  fi
}
build "$base" "$base"
build "$head" "$head"
build "$base" control "-C link-arg=-Wl,--shuffle-sections=*=1"
if cmp -s "$scratch/simd-$base" "$scratch/simd-control"; then
  echo "tick-bench: the control build is byte-identical to base, so it measures nothing" >&2
  exit 1
fi
declare -A bin=([base]="$scratch/simd-$base" [head]="$scratch/simd-$head" [control]="$scratch/simd-control")

echo "base $base"
echo "head $head"
echo "control: base again, functions shuffled (--shuffle-sections=*=1)"
echo "runs $runs per build, interleaved; load before: $(uptime | sed 's/.*load average: //')"
printf '%-22s %10s %10s %8s %8s %10s %10s %10s  %-6s %s\n' \
  scenario base_ms head_ms delta control base_floor head_floor ctl_floor state verdict

unmeasured=0
for file in "${scenarios[@]}"; do
  name=$(basename "$file" .json)
  declare -A means=() states=() failed=()
  for _ in $(seq "$runs"); do
    # Every side gets its try in the round where one fails, so a FAILED row names each build that
    # failed rather than only the first one tried.
    for side in base head control; do
      if ! out=$("${bin[$side]}" --bench "$file" --runs 1 2>&1) ||
        ! line=$(grep ' run 1/1: ' <<<"$out"); then
        why=$(grep -m1 '^Error: ' <<<"$out" | sed 's/^Error: //' || true)
        # anyhow's top line names the file; the deepest cause says why (a decode error, a refusal).
        deepest=$(sed -n '/^Caused by:/,$p' <<<"$out" | tail -n1 | sed -E 's/^ *([0-9]+: )?//' || true)
        # A panic's last line is the backtrace hint; its cause is the line after `panicked at`.
        panic=$(grep -m1 -A1 ' panicked at ' <<<"$out" | sed -E 's/.* (panicked at )/\1/' | paste -sd ' ' || true)
        failed[$side]="${why:-${panic:-$(tail -n1 <<<"$out")}}${deepest:+: $deepest}"
        continue
      fi
      means[$side]+="$(sed -E 's/.* mean=([0-9.]+) .*/\1/' <<<"$line") "
      states[$side]+="$(sed -E 's/.* state=([0-9a-f]+).*/\1/' <<<"$line") "
    done
    [ "${#failed[@]}" -gt 0 ] && break
  done
  for side in base head control; do
    [ -n "${failed[$side]:-}" ] && continue
    if [ "$(tr ' ' '\n' <<<"${states[$side]}" | sed '/^$/d' | sort -u | wc -l)" -ne 1 ]; then
      failed[$side]="STATE DIVERGED between its own runs: ${states[$side]% }"
    fi
  done
  if [ "${#failed[@]}" -gt 0 ]; then
    # One cause for every failed build when they agree, else each build's own. (`if`, not `&&`: a
    # false test last in a $(...) loop fails the substitution, and set -e then exits silently.)
    sides="" each=""
    for side in base head control; do
      if [ -n "${failed[$side]:-}" ]; then
        sides+="$side, " each+=" on $side: ${failed[$side]};"
      fi
    done
    causes=$(printf '%s\n' "${failed[@]}" | sort -u)
    if [ "$(wc -l <<<"$causes")" -eq 1 ]; then
      if [ "${#failed[@]}" -eq 3 ]; then sides="every build, "; fi
      printf '%-22s FAILED on %s: %s\n' "$name" "${sides%, }" "$causes"
    else
      printf '%-22s FAILED%s\n' "$name" "${each%;}"
    fi
    unmeasured=$((unmeasured + 1))
    continue
  fi
  same=$([ "${states[base]%% *}" = "${states[head]%% *}" ] && echo same || echo differ)
  awk -v name="$name" -v same="$same" -v b="${means[base]}" -v h="${means[head]}" -v c="${means[control]}" '
    function stats(list, out,   n, v, i, sum, lo, hi) {
      n = split(list, v, " "); sum = 0; lo = v[1] + 0; hi = v[1] + 0
      for (i = 1; i <= n; i++) { sum += v[i]; if (v[i] + 0 < lo) lo = v[i] + 0; if (v[i] + 0 > hi) hi = v[i] + 0 }
      out["mean"] = sum / n; out["floor"] = (hi - lo) / out["mean"] * 100
    }
    function abs(x) { return x < 0 ? -x : x }
    BEGIN {
      stats(b, bs); stats(h, hs); stats(c, cs)
      delta = (hs["mean"] - bs["mean"]) / bs["mean"] * 100
      control = (cs["mean"] - bs["mean"]) / bs["mean"] * 100
      floor = bs["floor"] > hs["floor"] ? bs["floor"] : hs["floor"]
      floor = cs["floor"] > floor ? cs["floor"] : floor
      floor = abs(control) > floor ? abs(control) : floor
      verdict = abs(delta) > floor ? "BEYOND FLOOR" : "within floor"
      printf "%-22s %10.4f %10.4f %+7.1f%% %+7.1f%% %9.1f%% %9.1f%% %9.1f%%  %-6s %s\n", name, bs["mean"], hs["mean"], delta, control, bs["floor"], hs["floor"], cs["floor"], same, verdict
    }'
done
echo "load after: $(uptime | sed 's/.*load average: //')"
if [ "$unmeasured" -gt 0 ]; then
  echo "tick-bench: $unmeasured of ${#scenarios[@]} scenarios could not be measured (FAILED rows)" >&2
  exit 1
fi
