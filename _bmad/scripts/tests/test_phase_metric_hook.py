"""Tests for the SessionEnd phase-metric safety net (phase_state.py + record_phase_on_exit.py).

The hook exists so a session that DIES mid-phase still lands its ledger row — the failure
it prevents is an invisible under-count, so these tests are mostly about the guards, not
the happy path. Two properties matter most and each has a test that reddens if it is lost:

  1. A state file owned by ANOTHER session is never recorded (mis-attribution guard).
  2. A failure to record KEEPS the state file, never silently drops it (retry guard).

Builds synthetic state in tmp dirs and mocks the recorder subprocess — never touches a
real ledger, a real cursor, or a real transcript.

Run: python3 -m unittest discover -s _bmad/scripts/tests
"""

import io
import json
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import phase_state as ps  # noqa: E402
import record_phase_on_exit as rp  # noqa: E402


class _Proc:
    def __init__(self, returncode=0, stdout="", stderr=""):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr


class PhaseHookTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.statefile = self.root / ".active-phase.json"
        self.projdir = self.root / "projects"
        self.projdir.mkdir()

        for mod in (ps, rp):
            p = unittest.mock.patch.object(mod, "state_path", lambda: str(self.statefile))
            p.start()
            self.addCleanup(p.stop)
        p = unittest.mock.patch.object(rp, "_claude_project_dir", lambda: str(self.projdir))
        p.start()
        self.addCleanup(p.stop)

    def _declare(self, session="sess-a.jsonl", phase="review-patch", story="ep-16-us-03"):
        self.statefile.write_text(json.dumps(
            {"phase": phase, "story": story, "session": session, "declared_at": "2026-08-31T00:00:00+00:00"}
        ))

    def _transcript(self, name="sess-a.jsonl"):
        (self.projdir / name).write_text("{}\n")

    def _run(self, session_id="sess-a"):
        stdin = io.StringIO(json.dumps({"session_id": session_id}))
        out = io.StringIO()
        with unittest.mock.patch.object(sys, "stdin", stdin), unittest.mock.patch.object(sys, "stdout", out):
            rc = rp.main()
        return rc, out.getvalue()

    # --- the two load-bearing guards -------------------------------------------------

    def test_state_owned_by_another_session_is_never_recorded(self):
        """MIS-ATTRIBUTION GUARD. A leftover declaration from a session that is still
        alive (or died) must not bill its phase against THIS session's tokens."""
        self._declare(session="sess-a.jsonl")
        self._transcript("sess-a.jsonl")
        with unittest.mock.patch.object(rp.subprocess, "run") as run:
            rc, out = self._run(session_id="sess-b-totally-different")
        run.assert_not_called()
        self.assertEqual(rc, 0)
        self.assertEqual(out, "")
        self.assertTrue(self.statefile.exists(), "the owning session must still find its state")

    def test_a_failed_recording_keeps_the_state_file(self):
        """RETRY GUARD. Losing the row silently is the exact damage this hook prevents,
        so a failure must leave evidence rather than tidy itself away."""
        self._declare()
        self._transcript()
        with unittest.mock.patch.object(rp.subprocess, "run", return_value=_Proc(1, stderr="boom")):
            rc, out = self._run()
        self.assertEqual(rc, 0)
        self.assertIn("NOT recorded", out)
        self.assertTrue(self.statefile.exists())

    # --- attribution comes from --transcript, not from timing ------------------------

    def test_records_against_the_transcript_named_in_the_state_not_the_newest(self):
        self._declare(session="sess-a.jsonl", phase="review-patch", story="ep-16-us-03")
        self._transcript("sess-a.jsonl")
        self._transcript("sess-newer.jsonl")  # a newer file must NOT be chosen
        with unittest.mock.patch.object(rp.subprocess, "run", return_value=_Proc(0, "est. cost $1.23")) as run:
            rc, out = self._run()
        argv = run.call_args[0][0]
        self.assertIn("--transcript", argv)
        self.assertEqual(argv[argv.index("--transcript") + 1], str(self.projdir / "sess-a.jsonl"))
        self.assertEqual(argv[argv.index("--phase") + 1], "review-patch")
        self.assertEqual(argv[argv.index("--story") + 1], "ep-16-us-03")
        self.assertEqual(rc, 0)
        self.assertIn("est. cost $1.23", out)
        self.assertFalse(self.statefile.exists(), "a recorded phase must not record twice")

    # --- everything else degrades quietly and never blocks a session exit ------------

    def test_no_declaration_is_silent(self):
        with unittest.mock.patch.object(rp.subprocess, "run") as run:
            rc, out = self._run()
        run.assert_not_called()
        self.assertEqual((rc, out), (0, ""))

    def test_garbage_state_is_treated_as_absent(self):
        self.statefile.write_text("not json at all")
        rc, out = self._run()
        self.assertEqual((rc, out), (0, ""))

    def test_incomplete_state_warns_instead_of_crashing(self):
        self.statefile.write_text(json.dumps({"phase": "review-patch"}))
        rc, out = self._run()
        self.assertEqual(rc, 0)
        self.assertIn("malformed", out)

    def test_missing_transcript_warns_and_keeps_state(self):
        self._declare()  # no transcript written
        rc, out = self._run()
        self.assertEqual(rc, 0)
        self.assertIn("missing", out)
        self.assertTrue(self.statefile.exists())

    def test_absent_session_id_still_records(self):
        """Attribution is carried by --transcript, so an unidentifiable exit is safe to
        record: the state names the transcript the phase actually ran in."""
        self._declare()
        self._transcript()
        stdin = io.StringIO("")  # no payload at all
        out = io.StringIO()
        with unittest.mock.patch.object(rp.subprocess, "run", return_value=_Proc(0)) as run, \
             unittest.mock.patch.object(sys, "stdin", stdin), unittest.mock.patch.object(sys, "stdout", out):
            rc = rp.main()
        run.assert_called_once()
        self.assertEqual(rc, 0)


class PhaseStateTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.statefile = Path(self.tmp.name) / "metrics" / ".active-phase.json"
        p = unittest.mock.patch.object(ps, "state_path", lambda: str(self.statefile))
        p.start()
        self.addCleanup(p.stop)

    def test_set_show_clear_round_trip(self):
        args = unittest.mock.Mock(phase="review-patch", story="ep-16-us-03", session="sess-a.jsonl")
        with unittest.mock.patch.object(sys, "stdout", io.StringIO()):
            self.assertEqual(ps.cmd_set(args), 0)
        state = ps.read_state()
        self.assertEqual(state["phase"], "review-patch")
        self.assertEqual(state["session"], "sess-a.jsonl")
        self.assertIn("declared_at", state)
        with unittest.mock.patch.object(sys, "stdout", io.StringIO()):
            ps.cmd_clear(args)
        self.assertIsNone(ps.read_state())

    def test_set_refuses_when_the_session_cannot_be_resolved(self):
        """Writing a declaration with no owner would let ANY later session claim it."""
        args = unittest.mock.Mock(phase="review-patch", story="s", session=None)
        with unittest.mock.patch.object(ps, "current_session_id", lambda: None), \
             unittest.mock.patch.object(sys, "stderr", io.StringIO()):
            self.assertEqual(ps.cmd_set(args), 2)
        self.assertFalse(self.statefile.exists())

    def test_clear_is_idempotent(self):
        with unittest.mock.patch.object(sys, "stdout", io.StringIO()):
            self.assertEqual(ps.cmd_clear(unittest.mock.Mock()), 0)


if __name__ == "__main__":
    unittest.main()
