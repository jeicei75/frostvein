#!/usr/bin/env python3
"""Declare which BMad phase the CURRENT session is spending tokens on.

WHY THIS EXISTS (ep-16-us-03 review, 2026-08-31)
------------------------------------------------
``session_tokens.py`` records a phase row when a skill's ``on_complete`` runs it. That
is a single point of failure at the END of a session: if the session dies first — context
exhausted, an interrupt, a crash — the row is never written and the spend leaves every
ledger at once. That is an INVISIBLE under-count, the same damage the mark-vs-row rule
exists to prevent, arrived at from the other direction.

So a skill DECLARES its phase here at activation, and a ``SessionEnd`` hook
(``record_phase_on_exit.py``) records it however the session ends. The skill's own
``on_complete`` still records normally and then clears this file — the hook is the
safety net for the abnormal exit, not the primary path. Belt and braces, deliberately:
the normal path stays visible in the skill where a human can read it.

THE STATE IS SESSION-SCOPED, and that is load-bearing. A leftover file from a session
that died must never bill its phase against a LATER session's tokens. Every write
records the session that owns it; the hook refuses to record a file another session
owns (see ``record_phase_on_exit.py``).

Usage:
    python3 phase_state.py set --phase review-patch --story ep-16-us-03-...
    python3 phase_state.py show
    python3 phase_state.py clear
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from session_tokens import _claude_project_dir, _forge_root, _newest_transcript  # noqa: E402


def state_path() -> str:
    return os.path.join(
        _forge_root(), "_bmad-output", "implementation-artifacts", "metrics", ".active-phase.json"
    )


def current_session_id() -> str | None:
    """Transcript id of the session running RIGHT NOW.

    At skill-activation time the live session is the newest top-level transcript — it is
    being appended to as this runs. ``recursive=False`` matters: a subagent file must never
    be mistaken for the session (session_tokens carries the same caveat for the same reason).
    """
    newest = _newest_transcript(_claude_project_dir(), recursive=False)
    return os.path.basename(newest) if newest else None


def read_state() -> dict | None:
    try:
        with open(state_path()) as fh:
            return json.load(fh)
    except (OSError, ValueError):
        return None


def cmd_set(args: argparse.Namespace) -> int:
    session = args.session or current_session_id()
    if not session:
        print("phase_state: cannot resolve the current session transcript", file=sys.stderr)
        return 2
    state = {
        "phase": args.phase,
        "story": args.story,
        "session": session,
        "declared_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
    }
    path = state_path()
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = path + ".tmp"
    with open(tmp, "w") as fh:
        json.dump(state, fh, indent=2, sort_keys=True)
        fh.write("\n")
    os.replace(tmp, path)  # atomic: a torn read here would mis-attribute a whole phase
    print(f"phase_state: {args.phase} / {args.story} owned by {session}")
    return 0


def cmd_clear(_: argparse.Namespace) -> int:
    try:
        os.remove(state_path())
        print("phase_state: cleared")
    except FileNotFoundError:
        print("phase_state: nothing to clear")
    return 0


def cmd_show(_: argparse.Namespace) -> int:
    state = read_state()
    if not state:
        print("phase_state: no active phase declared")
        return 0
    print(json.dumps(state, indent=2, sort_keys=True))
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("set", help="declare the phase this session is spending on")
    s.add_argument("--phase", required=True, help="create | dev | review | review-patch | live-gate | retro")
    s.add_argument("--story", required=True, help="story key (metrics ledger basename)")
    s.add_argument("--session", help="override the owning transcript id (testing)")
    s.set_defaults(func=cmd_set)

    sub.add_parser("clear", help="drop the declaration (the phase was recorded normally)").set_defaults(func=cmd_clear)
    sub.add_parser("show", help="print the current declaration").set_defaults(func=cmd_show)

    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
