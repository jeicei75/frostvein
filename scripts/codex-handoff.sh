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
# Sandbox: workspace-write (edits/commits within /workspace, no writes outside
# the workspace). approval_policy=never so it doesn't block on a prompt. Working
# root /workspace so Codex can read forge-root .runtime/ AND write/commit inside
# projects/<sub-repo>.
#
# NETWORK IS ON (Wolf's call at the Epic 1 retro, 2026-08-03). The sandbox denied
# loopback, so Codex could not run the daemon's own e2e tests — it reported the
# blocker and the orchestrator re-ran the gate. That was survivable when the
# daemon was one story of three; Epic 2 makes it the subject (2.1 deltas, 2.3 two
# clients, 2.4 load-broadcast all need a live socket), so Codex would ship code it
# never saw execute. The knob is a BOOLEAN, not a per-host allowlist, so this
# grants full internet: the closed-dependency-stack guarantee is now enforced by
# the handoff prompt, not the sandbox. Keep running `cargo fetch` before handoff
# and keep Codex building/testing `--offline`.
#
# Codex reads AGENTS.md from its working root upward. `-C` points at this repo,
# which now has its own AGENTS.md — before 2026-08-03 it did not, so Codex silently
# fell through to the forge's generic copy and never read frostvein's rules at all.
#
# .git MUST be listed in writable_roots: workspace-write shields .git by default,
# so without this `git checkout -b` dies with "cannot lock ref ... Read-only file
# system" and the story cannot be branched or committed. There is no
# allow_git_writes config key in codex-cli 0.146.0.

set -uo pipefail

PROMPT="${1:?prompt file required (handoff instructions, read from stdin by codex)}"
RUNLOG="${2:-/tmp/codex-run.log}"
LASTMSG="${3:-/tmp/codex-last.txt}"

CODEX_HOME=/workspace/.codex codex exec \
  -s workspace-write \
  -c approval_policy="never" \
  -c 'sandbox_workspace_write.writable_roots=["/workspace/projects/frostvein/.git"]' \
  -c sandbox_workspace_write.network_access=true \
  -C /workspace/projects/frostvein \
  -o "$LASTMSG" \
  - < "$PROMPT" > "$RUNLOG" 2>&1
rc=$?

echo "codex exit: $rc  (last message -> $LASTMSG, full log -> $RUNLOG)"
if grep -q '401\|Missing bearer' "$RUNLOG" 2>/dev/null; then
  echo "WARNING: 401 in log — check CODEX_HOME/auth (codex login status with CODEX_HOME=/workspace/.codex)" >&2
fi
exit "$rc"
