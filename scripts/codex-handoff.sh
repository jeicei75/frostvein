#!/usr/bin/env bash
# Launch Codex (GPT-5.x) non-interactively on a BMAD story handoff.
#
# WHY THIS EXISTS: Codex's auth/config live in the workspace-local CODEX_HOME
# (/workspace/.codex), NOT the default ~/.codex. Without CODEX_HOME set, `codex
# exec` fails with 401 Unauthorized ("Missing bearer or basic authentication").
# This wrapper pins the right CODEX_HOME and the standard non-interactive flags.
#
# USAGE:
#   scripts/codex-handoff.sh <prompt-file> [run-log] [last-msg-file]
#
# The prompt file is the dev-story/spike handoff instructions (point Codex at the
# story file, restate scope guardrails, branch/commit-as-Völundr/review-gated).
# Run it in the BACKGROUND from the agent harness; then relay last-msg-file and
# independently verify the result (don't trust exit 0 — a 401 still exits 0).
#
# Sandbox: workspace-write (edits/commits within /workspace, no network, no
# writes outside the workspace). approval_policy=never so it doesn't block on a
# prompt. Working root /workspace so Codex can read forge-root .runtime/ AND
# write/commit inside projects/<sub-repo>.

set -uo pipefail

PROMPT="${1:?prompt file required (handoff instructions, read from stdin by codex)}"
RUNLOG="${2:-/tmp/codex-run.log}"
LASTMSG="${3:-/tmp/codex-last.txt}"

CODEX_HOME=/workspace/.codex codex exec \
  -s workspace-write \
  -c approval_policy="never" \
  -C /workspace/projects/frostvein \
  -o "$LASTMSG" \
  - < "$PROMPT" > "$RUNLOG" 2>&1
rc=$?

echo "codex exit: $rc  (last message -> $LASTMSG, full log -> $RUNLOG)"
if grep -q '401\|Missing bearer' "$RUNLOG" 2>/dev/null; then
  echo "WARNING: 401 in log — check CODEX_HOME/auth (codex login status with CODEX_HOME=/workspace/.codex)" >&2
fi
exit "$rc"
