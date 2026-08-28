#!/usr/bin/env bash
# Reclaim build caches -- the review layers' under /tmp, and this repo's own target/.
#
# WHY THIS EXISTS: the code-review config MANDATES that every layer build into its own
# CARGO_TARGET_DIR under /tmp (bmad-code-review.toml, "BUILD ISOLATION -- MANDATORY", Epic 3
# retro action item P2). That fix is correct and stays -- four concurrent layers each told to
# run cargo would otherwise serialize on one target/ lock, which is what killed three of four
# layers at 3.1 and again at 3.3. Its stated cost was "disk", and nothing ever paid that back.
#
# MEASURED 2026-08-28, FIRST PASS: /tmp held 277 GB of stale review and verify caches, the
# oldest from 2026-08-09, none in use. Reaped.
#
# MEASURED 2026-08-28, SECOND PASS -- and this is why the script grew a second zone: /tmp was
# the SMALLER half. The repo's own target/ stood at 62 GB, of which debug/deps was 49 GB, and
# 29 GB of THAT was 60 stale hash-copies of our own `gui` binary plus its `headless` and
# `capture` test binaries at 1.2-1.4 GB each. Bevy links with full debuginfo and cargo never
# GCs a stale hash, so every gate round that touches `gui` mints another ~4 GB and keeps it
# forever. A story like 9.1 -- cold gate, five mutation rows, a Windows cross-compile -- adds
# tens of GB on its own. The /tmp reaper could not see any of it.
#
# THE CUT IS HASH-KEYED, NOT AGE-KEYED, and that distinction is the whole safety argument.
# Artifacts are named <target>-<hash>; we keep the newest KEEP_SETS hash-sets of each
# WORKSPACE target and delete the older ones. Consequences worth stating:
#   - A build in progress writes the NEWEST hash, so it is never a deletion candidate. This is
#     what makes the script safe to run while cargo is working, which an age rule is not.
#   - Third-party rlibs (libbevy_*, ~15 GB) are NOT touched. They are the expensive half to
#     rebuild and the half that does not grow per story. Deleting them to save disk would
#     trade a cheap resource for an expensive one.
#   - Everything here is a CACHE. The worst case of a wrong deletion is a rebuild, never a
#     lost artifact -- target/ is git-ignored build output, never the record.
#
# USAGE:  scripts/reap-build-caches.sh              both zones; /tmp refuses anything touched
#                                                   in the last hour
#         scripts/reap-build-caches.sh --dry-run    list what would go, delete nothing
#         scripts/reap-build-caches.sh --force      /tmp zone ignores the age guard (use when
#                                                   a review has just ended)
#         scripts/reap-build-caches.sh --tmp-only   the review-cache zone alone. This is what
#                                                   the code-review orchestrator runs, so that
#                                                   a review never writes inside the repo.
#         scripts/reap-build-caches.sh --auto       target zone, quiet unless it reclaimed
#                                                   something. Wired into scripts/gate.sh --
#                                                   the gate is what CREATES this garbage, so
#                                                   it is where the reaping belongs. A
#                                                   procedure is exactly what an accumulating
#                                                   cache defeats.
#
# WHAT IT WILL NEVER TOUCH:
#   - any file in the repository that is not build output under the cargo target dir
#   - anything that is not a DIRECTORY matching the /tmp patterns (review notes such as
#     /tmp/review-findings.md are files and are deliberately left alone)
#   - the agent scratchpads under /tmp/claude-*
#   - third-party dependency artifacts, in either zone

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# An artifact set survives if it is newer than KEEP_DAYS, OR if it is one of the newest
# KEEP_FLOOR sets of its target. Both halves are load-bearing and both were MEASURED:
#
#   KEEP_FLOOR exists because "keep the newest N" alone is WRONG, and the number said so. One
#   gate round keeps SEVERAL hashes of the same crate alive at once -- clippy, the test build
#   and the bin build all differ -- so with N=2 the next gate cost 28-43s against 9s warm: the
#   reap was deleting artifacts the current build still wanted and buying disk with build time
#   on every commit. Measured live set right after a gate: 11 `gui` hashes.
#
#   KEEP_DAYS is what actually does the reclaiming, and it is the half that makes the steady
#   state FREE: a week of builds is not stale, so on most runs there is nothing to delete and
#   nothing to relink. Only genuinely old generations go, which is where the 29 GB was.
#
# A build in progress writes the newest set, so it is never a candidate under either half --
# that is what makes this safe to run while cargo is working, which an age rule alone is not.
KEEP_DAYS=7
KEEP_FLOOR=4

DRY_RUN=0
FORCE=0
DO_TMP=1
DO_TARGET=1
QUIET=0
for arg in "$@"; do
  case "$arg" in
    --dry-run)    DRY_RUN=1 ;;
    --force)      FORCE=1 ;;
    --tmp-only)   DO_TARGET=0 ;;
    --target-only) DO_TMP=0 ;;
    --auto)       DO_TMP=0; QUIET=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

say() { [ "$QUIET" -eq 1 ] || printf '%s\n' "$*"; }

# ---------------------------------------------------------------- zone 1: /tmp review caches

# A candidate is only ours to delete if it REALLY lives under /tmp. Testing the leaf for -L is
# not enough and the fixture proved it: /tmp/<symlink-to-repo>/x86_64-pc-windows-gnu is not itself
# a symlink, so a leaf test waved through 8.6 GB INSIDE THE REPOSITORY -- the exact outcome the
# header promises can never happen. Resolve the whole path instead, and refuse anything that
# lands outside /tmp or inside the repo.
contained_in_tmp() {
  local real
  real=$(realpath -e "$1" 2>/dev/null) || return 1
  case "$real" in
    "$REPO_ROOT"/*) return 1 ;;
    /tmp/*)         return 0 ;;
    *)              return 1 ;;
  esac
}

reap_tmp() {
  shopt -s nullglob
  local -A seen=()
  local candidates=()
  local d

  # Named shapes, from the layer prompt template (CARGO_TARGET_DIR=/tmp/review-<layer>/target).
  for d in /tmp/review-* /tmp/verify-* /tmp/mut; do
    [ -d "$d" ] || continue      # skips /tmp/review-findings.md and the stray .diff files
    contained_in_tmp "$d" || continue
    [ -n "${seen[$d]:-}" ] && continue
    seen[$d]=1; candidates+=("$d")
  done

  # Shape is not enough. MEASURED 2026-08-28: 7.6 GB across 434 /tmp directories matched none
  # of those globs -- frostvein-review-probe, fvprobe1..3, settle-target, workprogress-target2,
  # frostvein-save-size.* -- ad-hoc target dirs from past sessions, each named whatever the
  # agent picked that day. A name list can only ever catch the names we already thought of, so
  # match on the PROPERTY instead: cargo writes CACHEDIR.TAG into every target directory it
  # creates. That catches the ones nobody has invented yet, which is the point.
  for d in /tmp/*/ /tmp/*/*/; do
    d="${d%/}"
    [ -d "$d" ] || continue
    case "$d" in /tmp/claude-*) continue ;; esac          # agent scratchpads, not ours to reap
    [ -f "$d/CACHEDIR.TAG" ] || continue
    contained_in_tmp "$d" || continue
    [ -n "${seen[$d]:-}" ] && continue
    # Never reap the target dir the caller is building into right now.
    [ -n "${CARGO_TARGET_DIR:-}" ] && [ "$d" = "${CARGO_TARGET_DIR%/}" ] && continue
    seen[$d]=1; candidates+=("$d")
  done
  shopt -u nullglob

  if [ ${#candidates[@]} -eq 0 ]; then
    say "no review caches present -- nothing to reap"
    return 0
  fi

  # A layer that is still building has a recently-touched directory. Age is a proxy, so --force
  # exists for the orchestrator to use when it KNOWS the review is over; without it we refuse
  # rather than delete under a live layer.
  local live=() reap=()
  for d in "${candidates[@]}"; do
    if [ "$FORCE" -eq 0 ] && [ -n "$(find "$d" -maxdepth 0 -mmin -60)" ]; then
      live+=("$d")
    else
      reap+=("$d")
    fi
  done

  for d in "${live[@]}"; do
    say "SKIP (touched in the last hour, may be live): $d  -- use --force if the review has ended"
  done

  if [ ${#reap[@]} -eq 0 ]; then
    say "nothing eligible to reap under /tmp"
    return 0
  fi

  local total=0 mb
  for d in "${reap[@]}"; do
    mb=$(du -sm "$d" 2>/dev/null | cut -f1)
    total=$((total + mb))
    # Big ones only in a real run (the summary carries the rest); a DRY RUN lists everything,
    # since showing exactly what would go is the only thing it is for.
    { [ "$mb" -ge 100 ] || [ "$DRY_RUN" -eq 1 ]; } && say "$(printf '%-44s %6s MB   last modified %s' "$d" "$mb" "$(date -r "$d" '+%Y-%m-%d %H:%M')")"
  done

  if [ "$DRY_RUN" -eq 1 ]; then
    say "$(printf 'DRY RUN -- would reclaim %d MB (%.1f GB) from %d directories under /tmp' \
      "$total" "$(echo "$total" | awk '{print $1/1024}')" "${#reap[@]}")"
    return 0
  fi

  rm -rf -- "${reap[@]}"
  say "$(printf 'reaped %d directories under /tmp (%.1f GB)' "${#reap[@]}" "$(echo "$total" | awk '{print $1/1024}')")"
}

# ------------------------------------------------------- zone 2: this repo's own target tree

# The set of artifact prefixes we are willing to delete: the workspace crates and their
# integration-test targets, plus the lib forms. DERIVED, not hardcoded -- a new tests/*.rs file
# must not silently fall outside the reaper, which is how the stale-sabotage-literal class of
# defect keeps happening here.
workspace_prefixes() {
  local p name
  for p in "$REPO_ROOT"/crates/*/; do
    [ -d "$p" ] || continue
    name="$(basename "$p")"; name="${name//-/_}"
    printf '%s\n%s\n' "$name" "lib$name"
  done
  for p in "$REPO_ROOT"/crates/*/tests/*.rs; do
    [ -f "$p" ] || continue
    name="$(basename "$p" .rs)"; name="${name//-/_}"
    printf '%s\n%s\n' "$name" "lib$name"
  done
}

# Prune one artifact directory: group entries into <prefix>-<hash> sets, keep the newest
# KEEP_SETS of each known prefix, delete the rest whole. Deleting a set WHOLE matters -- a
# half-deleted set (binary gone, .d file left) is the kind of state that makes cargo behave
# confusingly rather than simply rebuilding.
prune_artifact_dir() {
  local dir="$1"
  [ -d "$dir" ] || return 0

  local -A newest=() paths=()
  local e base prefix hash mt key

  for e in "$dir"/*; do
    base="${e##*/}"
    # deps/.fingerprint/build use a 16-hex hash; incremental/ uses a 13-14 char base36 one.
    [[ "$base" =~ ^(.+)-([0-9a-z]{13,16})(\..*)?$ ]] || continue
    prefix="${BASH_REMATCH[1]}"
    hash="${BASH_REMATCH[2]}"
    [ -n "${WS_PREFIX[$prefix]:-}" ] || continue
    mt=$(stat -c %Y "$e" 2>/dev/null) || continue
    key="$prefix|$hash"
    if [ -z "${newest[$key]:-}" ] || [ "$mt" -gt "${newest[$key]}" ]; then newest[$key]=$mt; fi
    paths[$key]="${paths[$key]:-}$e"$'\n'
  done

  [ ${#newest[@]} -eq 0 ] && return 0

  # Rank each prefix's hash-sets newest-first; a set dies only if it is BOTH past the floor
  # and older than the window.
  local -A kept=()
  local line k pfx cutoff
  cutoff=$(( $(date +%s) - KEEP_DAYS * 86400 ))
  while read -r mt key; do
    pfx="${key%%|*}"
    kept[$pfx]=$(( ${kept[$pfx]:-0} + 1 ))
    [ "${kept[$pfx]}" -le "$KEEP_FLOOR" ] && continue
    [ "$mt" -ge "$cutoff" ] && continue
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      DOOMED+=("$line")
    done <<< "${paths[$key]}"
  done < <(for k in "${!newest[@]}"; do printf '%s %s\n' "${newest[$k]}" "$k"; done | sort -rn)
}

reap_target() {
  local target="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
  target="${target%/}"

  # Proof that this is a cargo target directory and not something else that shares the name.
  # Without this the script would happily prune whatever path a mistyped env var pointed at.
  #
  # TWO proofs, and the second is here because the first was a SILENT NO-OP on the one tree
  # this zone exists for. CACHEDIR.TAG is written by cargo only when CARGO creates the target
  # directory -- and scripts/gate.sh does `mkdir -p target` first, so this repo's target/ has
  # never had the tag. The marker test alone skipped the whole 62 GB while reporting success,
  # which is this project's signature failure shape. A .fingerprint directory under a profile
  # is structural: nothing but cargo builds that shape.
  if [ ! -f "$target/CACHEDIR.TAG" ] && \
     ! compgen -G "$target/*/.fingerprint" > /dev/null && \
     ! compgen -G "$target/*/*/.fingerprint" > /dev/null; then
    say "not a cargo target dir (no CACHEDIR.TAG, no */.fingerprint): $target -- skipping the target zone"
    return 0
  fi

  local -a prefixes
  mapfile -t prefixes < <(workspace_prefixes)
  declare -gA WS_PREFIX=()
  local p; for p in "${prefixes[@]}"; do WS_PREFIX[$p]=1; done

  DOOMED=()
  local profile sub
  # debug/ and release/ for the host, plus every cross-compile triple (the Windows gui.exe
  # tree was 8.6 GB of the 62 and is the same shape).
  for profile in "$target"/debug "$target"/release "$target"/*/debug "$target"/*/release; do
    [ -d "$profile" ] || continue
    # deps/ ONLY, and the exclusion is MEASURED, not cautious. Pruning .fingerprint/ and
    # incremental/ as well made the very next gate run cost 43s against 9s warm -- cargo loses
    # the state that lets it skip work, so the reap buys disk with build time on every commit.
    # That is the exact trade Wolf's two-tier gate ruling forbids: a clumsy gate tempts
    # --no-verify, which makes the gate worthless. And it buys almost nothing: incremental/ is
    # 4.2 GB and .fingerprint/ is 29 MB, against 49 GB in deps/ where the stale gui, headless
    # and capture binaries actually live.
    prune_artifact_dir "$profile/deps"
  done

  if [ ${#DOOMED[@]} -eq 0 ]; then
    say "target/: nothing stale -- no workspace artifact is both past the newest $KEEP_FLOOR and older than ${KEEP_DAYS}d"
    return 0
  fi

  local kb
  kb=$(du -sck "${DOOMED[@]}" 2>/dev/null | tail -1 | cut -f1)

  if [ "$DRY_RUN" -eq 1 ]; then
    printf 'DRY RUN -- would reclaim %.1f GB from %d stale artifacts in %s\n' \
      "$(echo "$kb" | awk '{print $1/1048576}')" "${#DOOMED[@]}" "$target"
    return 0
  fi

  rm -rf -- "${DOOMED[@]}"
  printf 'reaped %d stale build artifacts from %s (%.1f GB)\n' \
    "${#DOOMED[@]}" "${target#"$REPO_ROOT"/}" "$(echo "$kb" | awk '{print $1/1048576}')"
}

before=$(df -m /tmp | awk 'NR==2{print $4}')
[ "$DO_TMP" -eq 1 ] && reap_tmp
[ "$DO_TARGET" -eq 1 ] && reap_target
after=$(df -m /tmp | awk 'NR==2{print $4}')

if [ "$DRY_RUN" -eq 0 ] && [ "$after" -gt "$before" ]; then
  say "$(printf 'free space %d MB -> %d MB (reclaimed %.1f GB)' \
    "$before" "$after" "$(echo "$after $before" | awk '{print ($1-$2)/1024}')")"
fi
