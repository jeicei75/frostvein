#!/usr/bin/env bash
# Gate, push, and VERIFY the remote actually moved.
#
#   scripts/push.sh              # gate, then push the current branch
#   scripts/push.sh --no-gate    # skip the gate (say why; it prints a warning)
#
# WHY THIS EXISTS — issue #76. `.githooks/pre-push` runs the full gate, which is 350-450s. Git
# opens the connection to the remote BEFORE running the hook, so the connection sits idle for the
# whole gate and GitHub's sshd closes it. The transfer then never happens.
#
# THAT FAILURE IS SILENT IN THE DIRECTION THAT LOOKS LIKE SUCCESS, which is the actual defect. The
# last line on screen is `GATE GREEN`; the `Connection to github.com closed by remote host.` lands
# mid-gate, beside a check's own output, where it reads as unrelated noise. Observed 2026-09-07:
# a green gate, no branch on the remote, and nothing at the end saying so.
#
# So the fix is not only "make the timeout less likely" (keepalives do that, see below) — it is to
# ASK THE REMOTE afterwards. This script's last act is `git ls-remote`, and it exits non-zero if
# the remote ref is not the commit that was pushed. A push that fails now says so.
#
# The two orderings differ in a way that matters:
#   git push          -> connect, gate for 400s while idle, transfer   (the connection can die)
#   scripts/push.sh   -> gate for 400s, connect, transfer, verify      (nothing waits on a socket)
#
# Keepalives are a belt as well: `git config core.sshCommand 'ssh -o ServerAliveInterval=20'` is
# in the README's one-time setup beside `core.hooksPath`. It cannot be committed into the repo —
# git deliberately does not let a fetched repository dictate the client's ssh options.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

run_gate=1
if [ "${1:-}" = "--no-gate" ]; then
  run_gate=0
  shift
fi

branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$branch" = "HEAD" ]; then
  echo "push.sh: detached HEAD — check out a branch first" >&2
  exit 2
fi
local_sha=$(git rev-parse HEAD)

if [ "$run_gate" -eq 1 ]; then
  if ! scripts/gate.sh; then
    echo "push.sh: gate is RED — nothing pushed" >&2
    exit 1
  fi
else
  # Named rather than silent. `--no-gate` is honest only when the gate has demonstrably passed on
  # THIS commit already; it is not a way to push something unproven.
  echo "push.sh: --no-gate: the gate did NOT run here. It must already be green on $(git rev-parse --short HEAD)."
fi

# `--no-verify` skips `.githooks/pre-push`, which would otherwise re-run the whole gate a second
# time on the same commit — the gate above is that check, moved to before the connection.
echo "push.sh: pushing $branch -> origin"
GIT_SSH_COMMAND="${GIT_SSH_COMMAND:-ssh -o ServerAliveInterval=20 -o ServerAliveCountMax=60}" \
  git push --no-verify -u origin "$branch" "$@"
push_status=$?

# THE VERIFICATION, and the reason this script exists. Asked of the REMOTE, never of a local ref:
# a local ref can be updated by a push whose transfer later failed, and `git push` has been seen to
# leave the branch absent while reporting nothing at the end.
remote_sha=$(git ls-remote --heads origin "$branch" 2>/dev/null | awk '{print $1}')

if [ "$remote_sha" = "$local_sha" ]; then
  echo "push.sh: VERIFIED — origin/$branch is $(git rev-parse --short HEAD)"
  exit 0
fi

echo >&2
echo "push.sh: PUSH DID NOT LAND." >&2
echo "  local   $local_sha" >&2
echo "  remote  ${remote_sha:-<branch absent from origin>}" >&2
if [ "$push_status" -eq 0 ]; then
  echo "  git push exited 0, so this is the silent case issue #76 describes:" >&2
  echo "  the connection died and nothing at the end said so." >&2
fi
echo "  Re-run scripts/push.sh --no-gate (the gate already passed on this commit)." >&2
exit 1
