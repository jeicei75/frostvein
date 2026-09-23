#!/usr/bin/env bash
# Launch Codex non-interactively on a BMAD story handoff, on the model pinned below.
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
# CODEX_HOME (/workspace/.codex) must ALSO be writable. It sits outside the `-C`
# working root, so workspace-write leaves it read-only, and story 2.1's attempt at the
# `codex review --base main` pre-handback self-gate died with "Read-only file system
# (os error 30)" when its app-server tried to write there. Codex correctly reported the
# blocker instead of working around it, so the self-gate simply never ran that story.
# NOTE: the fix is reasoned from that error plus the path layout — it has not yet been
# confirmed by a delegated run that actually reaches the self-gate. Verify at 2.2.
# Separately: `codex review` on a ~650-line diff ran past 10 minutes unsandboxed, so
# budget real wall-clock for it rather than treating it as a quick gate.
#
# .git MUST be listed in writable_roots: workspace-write shields .git by default,
# so without this `git checkout -b` dies with "cannot lock ref ... Read-only file
# system" and the story cannot be branched or committed. There is no
# allow_git_writes config key in codex-cli 0.146.0.

set -uo pipefail

PROMPT="${1:?prompt file required (handoff instructions, read from stdin by codex)}"
RUNLOG="${2:-/tmp/codex-run.log}"
LASTMSG="${3:-/tmp/codex-last.txt}"

ROOT=/workspace/projects/frostvein

# THE MODEL IS PINNED HERE, not in $CODEX_HOME/config.toml (forge-process 1.8.0, hand-merged).
# That file is gitignored and per-pod, so the dev model used to live in no tracked file: stable
# only because nobody touched it, invisible to review, and a Codex TUI `/model` pick saved to that
# home would have silently changed every later handoff. frostvein paid for exactly that: the
# model/effort silently drifted between runs (memory codex-delegation-runbook). A `-c` flag
# outranks config.toml, so this script is now the authority and a model change is a commit.
# gpt-6-sol since 2026-09-23 (Wolf: the GPT-6 line has no Terra); gpt-5.6-terra before.
# Its price row is `gpt-6-sol` in _bmad/scripts/session_tokens.py; move both together.
CODEX_MODEL="gpt-6-sol"
CODEX_EFFORT="high"

# Codex resolves AGENTS.md upward from its working root and does NOT read CLAUDE.md. frostvein's
# AGENTS.md is its OWN Codex-facing rules file (not a copy of CLAUDE.md, so the forge's
# must-be-a-symlink check does not apply here). Without it Codex walks up and reads the forge's
# generic rules instead; frostvein ran three whole stories that way. Warn loudly rather than fail.
if [ ! -f "$ROOT/AGENTS.md" ]; then
  echo "WARNING: no AGENTS.md at $ROOT — Codex will resolve one from a PARENT directory" >&2
  echo "         and never see frostvein's rules." >&2
fi

CODEX_HOME=/workspace/.codex codex exec \
  -s workspace-write \
  -c model="$CODEX_MODEL" \
  -c model_reasoning_effort="$CODEX_EFFORT" \
  -c approval_policy="never" \
  -c "sandbox_workspace_write.writable_roots=[\"$ROOT/.git\",\"/workspace/.codex\"]" \
  -c sandbox_workspace_write.network_access=true \
  -C "$ROOT" \
  -o "$LASTMSG" \
  - < "$PROMPT" > "$RUNLOG" 2>&1
rc=$?

echo "codex exit: $rc  (model $CODEX_MODEL/$CODEX_EFFORT; last message -> $LASTMSG, full log -> $RUNLOG)"
# forge gh-100: anchor to how the error ARRIVES, never a bare `401`. The old guard matched the
# number anywhere, including inside prompt text Codex echoes back, so clean runs were told to go
# check `codex login status`. Not restricted to the log tail: auth fails on the FIRST request.
# KNOWN OPEN upstream (gh-100): `Missing bearer` is still a bare substring and the dev-story
# handoff prompt echoes that phrase, so this can still false-fire. Wolf's ruling (2026-09-17):
# capture a real 401 live, commit it as a fixture, THEN anchor to the observed shape.
if grep -qE '401 Unauthorized|Missing bearer' "$RUNLOG" 2>/dev/null; then
  echo "WARNING: 401 Unauthorized in log — check CODEX_HOME/auth (codex login status with CODEX_HOME=/workspace/.codex)" >&2
fi
# Independent `if`, not an `elif`: auth and quota are separate facts about one run, and an `elif`
# let the over-firing auth guard SHADOW the quota message on exactly the runs that need it.
# frostvein's own quota deaths left NO last-msg file (memory codex-quota-exhaustion-8-2).
if grep -qE "hit your usage limit" "$RUNLOG" 2>/dev/null; then
  echo "WARNING: Codex stopped on its USAGE LIMIT, not an auth failure — the weekly pool is shared with the forge. The log names the reset time." >&2
fi
exit "$rc"
