#!/usr/bin/env python3
"""SessionEnd hook: record the declared BMad phase metric however the session ends.

WHY (ep-16-us-03 review, 2026-08-31). The phase row was written only by a skill's
``on_complete``, i.e. only if the session survived to the end of its workflow. A session
that died first — context exhausted, interrupted, crashed — silently dropped its whole
spend from the ledger. This hook closes that: ``phase_state.py`` declares the phase at
skill activation, and this runs at SessionEnd.

WHY ``SessionEnd`` AND NOT ``Stop``: ``Stop`` fires whenever Claude stops responding —
every turn, plus clear/resume/compact. Recording there would append a ledger row per turn,
turning one phase into dozens of fragments. The delta cursor would keep each row honest
individually, but the ledger would be unreadable. SessionEnd fires once.

ATTRIBUTION COMES FROM ``--transcript``, NOT FROM TIMING. The state file names the
transcript the phase actually ran in, and that is what gets measured — so even a late or
unexpected firing bills the right session. The session_id check below decides only WHETHER
THIS IS THE RIGHT MOMENT: a file another live session owns is left alone for that session.

FAILURE POLICY: this must never break a session exit. It always exits 0. If recording
fails, the state file is deliberately LEFT IN PLACE — a retry is possible and
``phase_state.py show`` will reveal it, which beats losing the row silently. That is the
same principle as the mark-vs-row rule: prefer a visible discrepancy to an invisible loss.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from phase_state import read_state, state_path  # noqa: E402
from session_tokens import _claude_project_dir, _forge_root  # noqa: E402


def emit(message: str) -> None:
    """Hook JSON output. `systemMessage` surfaces in the UI; silence otherwise."""
    print(json.dumps({"systemMessage": message, "suppressOutput": True}))


def ending_session_id() -> str | None:
    try:
        payload = json.load(sys.stdin)
    except (ValueError, OSError):
        return None
    sid = payload.get("session_id") if isinstance(payload, dict) else None
    return str(sid) if sid else None


def main() -> int:
    state = read_state()
    if not state:
        return 0  # nothing declared — the overwhelmingly common case, stay silent

    phase, story, owner = state.get("phase"), state.get("story"), state.get("session")
    if not (phase and story and owner):
        emit(f"phase metric NOT recorded: {state_path()} is malformed — record it by hand.")
        return 0

    sid = ending_session_id()
    if sid and not owner.startswith(sid):
        # Another live session declared this. Leave it for that session's own SessionEnd.
        return 0

    transcript = os.path.join(_claude_project_dir(), owner)
    if not os.path.exists(transcript):
        emit(f"phase metric NOT recorded: transcript {owner} is missing. State kept.")
        return 0

    cmd = [
        sys.executable,
        os.path.join(_forge_root(), "_bmad", "scripts", "session_tokens.py"),
        "--phase", phase,
        "--story", story,
        "--transcript", transcript,
    ]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    except (OSError, subprocess.SubprocessError) as exc:
        emit(f"phase metric NOT recorded ({type(exc).__name__}). State kept for retry.")
        return 0

    if proc.returncode != 0:
        tail = (proc.stderr or proc.stdout or "").strip().splitlines()
        detail = tail[-1] if tail else f"exit {proc.returncode}"
        emit(f"phase metric NOT recorded: {detail}. State kept for retry.")
        return 0

    try:
        os.remove(state_path())
    except OSError:
        pass  # recorded is what matters; a stale file is caught by the owner check above

    cost = next((ln.strip() for ln in proc.stdout.splitlines() if "est. cost" in ln), "")
    emit(f"Recorded phase={phase} for {story} on session exit. {cost}".strip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
