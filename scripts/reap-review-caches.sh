#!/usr/bin/env bash
# Reclaim the per-layer review build caches under /tmp.
#
# WHY THIS EXISTS: the code-review config MANDATES that every layer build into its own
# CARGO_TARGET_DIR under /tmp (bmad-code-review.toml, "BUILD ISOLATION -- MANDATORY", Epic 3
# retro action item P2). That fix is correct and stays -- four concurrent layers each told to
# run cargo would otherwise serialize on one target/ lock, which is what killed three of four
# layers at 3.1 and again at 3.3. Its stated cost was "disk", and nothing ever paid that back.
#
# MEASURED 2026-08-28: /tmp held 277 GB of stale review and verify caches, the oldest from
# 2026-08-09, none in use -- 295 GB of a 1007 GB volume, against 62 GB for the real target/.
# Every previous guard was "remember to clean up", and a procedure is exactly what an
# accumulating cache defeats. Reaping is now a command.
#
# USAGE:  scripts/reap-review-caches.sh             reap; refuses anything touched in the last hour
#         scripts/reap-review-caches.sh --dry-run   list what would go, delete nothing
#         scripts/reap-review-caches.sh --force     reap regardless of age (use when a review just ended)
#
# WHAT IT WILL NEVER TOUCH, and this is the load-bearing half:
#   - anything inside the repository, tracked or untracked
#   - anything that is not a DIRECTORY matching the patterns below (review notes such as
#     /tmp/review-findings.md are files and are deliberately left alone)
#   - the agent scratchpads under /tmp/claude-*
# Findings must already live in the story file before a review is over; these directories are
# build output and scratch, never the record.

set -euo pipefail

DRY_RUN=0
FORCE=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --force)   FORCE=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

# Directories only, and only these shapes. Globs that match nothing expand to nothing.
shopt -s nullglob
candidates=()
for d in /tmp/review-* /tmp/verify-* /tmp/mut; do
  [ -d "$d" ] || continue      # skips /tmp/review-findings.md and the stray .diff files
  [ -L "$d" ] && continue      # never follow a symlink out of /tmp
  candidates+=("$d")
done
shopt -u nullglob

if [ ${#candidates[@]} -eq 0 ]; then
  echo "no review caches present -- nothing to reap"
  exit 0
fi

# A layer that is still building has a recently-touched directory. Age is a proxy, so --force
# exists for the orchestrator to use when it KNOWS the review is over; without it we refuse
# rather than delete under a live layer.
live=()
reap=()
for d in "${candidates[@]}"; do
  if [ "$FORCE" -eq 0 ] && [ -n "$(find "$d" -maxdepth 0 -mmin -60)" ]; then
    live+=("$d")
  else
    reap+=("$d")
  fi
done

for d in "${live[@]}"; do
  echo "SKIP (touched in the last hour, may be live): $d  -- use --force if the review has ended"
done

if [ ${#reap[@]} -eq 0 ]; then
  echo "nothing eligible to reap"
  exit 0
fi

total=0
for d in "${reap[@]}"; do
  mb=$(du -sm "$d" 2>/dev/null | cut -f1)
  total=$((total + mb))
  printf "%-40s %6s MB   last modified %s\n" "$d" "$mb" "$(date -r "$d" '+%Y-%m-%d %H:%M')"
done

if [ "$DRY_RUN" -eq 1 ]; then
  printf "\nDRY RUN -- would reclaim %d MB (%.1f GB) from %d directories\n" \
    "$total" "$(echo "$total" | awk '{print $1/1024}')" "${#reap[@]}"
  exit 0
fi

before=$(df -m /tmp | awk 'NR==2{print $4}')
rm -rf -- "${reap[@]}"
after=$(df -m /tmp | awk 'NR==2{print $4}')

printf "\nreaped %d directories; free space %d MB -> %d MB (reclaimed %.1f GB)\n" \
  "${#reap[@]}" "$before" "$after" "$(echo "$after $before" | awk '{print ($1-$2)/1024}')"
