"""Tests for session_tokens.py — the per-phase/per-tool/per-model cost ledger.

Covers the A1 fixes (Codex transcript parsing, delta-since-cursor isolation, per-model
pricing, Claude subagent accounting) and the axes ported from frostvein 2026-08-08:
wall-clock `minutes`, Codex weekly `quota_pp`, nested Codex rollouts, rollup
preservation, and the ledger width guard. Builds synthetic transcripts in tmp dirs —
never touches real ledgers.

Run: python3 -m unittest discover -s _bmad/scripts/tests
"""

import contextlib
import io
import json
import os
import sys
import tempfile
import unittest
import unittest.mock
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import session_tokens as st  # noqa: E402


def _claude_turn(model, inp, cw, cr, out):
    return json.dumps(
        {
            "message": {
                "model": model,
                "usage": {
                    "input_tokens": inp,
                    "cache_creation_input_tokens": cw,
                    "cache_read_input_tokens": cr,
                    "output_tokens": out,
                },
            }
        }
    )


def _codex_token_count(model, inp, cached, out, used_percent=None):
    payload = {
        "type": "token_count",
        "info": {
            "total_token_usage": {
                "input_tokens": inp,
                "cached_input_tokens": cached,
                "output_tokens": out,
                "total_tokens": inp + out,
            }
        },
    }
    if used_percent is not None:
        # SIBLING of `info`, not a member — see test_real_rollout_line_is_pinned_verbatim.
        payload["rate_limits"] = {"primary": {"used_percent": used_percent}}
    return [
        json.dumps({"type": "turn_context", "payload": {"model": model}}),
        json.dumps({"type": "event_msg", "payload": payload}),
    ]


class CodexParsingTests(unittest.TestCase):
    def test_uses_last_cumulative_event_and_splits_cached(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rollout-x.jsonl"
            lines = _codex_token_count("gpt-5.5", 100, 80, 5)  # earlier cumulative
            lines += _codex_token_count("gpt-5.5", 1000, 900, 25)[1:]  # later cumulative wins
            path.write_text("\n".join(lines) + "\n")
            s = st.sum_codex_transcript(str(path))
            self.assertEqual(s["input"], 100)  # fresh = 1000 - 900 cached
            self.assertEqual(s["cache_read"], 900)
            self.assertEqual(s["output"], 25)
            self.assertEqual(s["total"], 1025)  # == input_tokens + output_tokens
            self.assertEqual(s["models"], ["gpt-5.5"])
            self.assertIsNotNone(st.estimate_usd(s))  # gpt-5 is priced

    def test_empty_when_no_token_count(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rollout-empty.jsonl"
            path.write_text(json.dumps({"type": "session_meta", "payload": {}}) + "\n")
            s = st.sum_codex_transcript(str(path))
            self.assertEqual(s["total"], 0)


class DeltaCursorTests(unittest.TestCase):
    def test_isolates_phases_on_one_transcript(self):
        after_create = st._as_summary(
            50, ["claude-opus-4-8"], {"input": 10, "cache_creation": 100, "cache_read": 1000, "output": 20}
        )
        delta1, reset1 = st.delta_since_cursor(after_create, None)
        self.assertFalse(reset1)
        self.assertEqual(delta1["total"], after_create["total"])  # first record bills whole cumulative
        cursor = {b: int(after_create[b]) for b in st._CURSOR_BUCKETS}

        after_patch = st._as_summary(
            80, ["claude-opus-4-8"], {"input": 15, "cache_creation": 150, "cache_read": 1700, "output": 35}
        )
        delta2, reset2 = st.delta_since_cursor(after_patch, cursor)
        self.assertFalse(reset2)
        self.assertEqual(delta2["turns"], 30)
        self.assertEqual((delta2["input"], delta2["cache_read"], delta2["output"]), (5, 700, 15))
        # The fix: the patch row excludes the create cost — not billed the whole session.
        self.assertLess(delta2["total"], after_patch["total"])

    def test_resets_when_cursor_stale(self):
        cumulative = st._as_summary(5, ["gpt-5.5"], {"input": 1, "cache_creation": 0, "cache_read": 2, "output": 1})
        stale = {"turns": 999, "input": 999, "cache_creation": 999, "cache_read": 999, "output": 999}
        delta, reset = st.delta_since_cursor(cumulative, stale)
        self.assertTrue(reset)
        self.assertEqual(delta["total"], cumulative["total"])  # fall back to full cumulative


    # --- `--mark`: close a phase boundary without writing a ledger row -------------
    #
    # The delta cursor only ever advanced when a row was written, so any window NOT
    # bookended by two rows was swept into the next row that happened to be taken —
    # the leading window before a session's first record, and the gap between one
    # phase ending and the next beginning. That mixed a 14-patch verification pass
    # and a separate vehicle session into one $151.81 row in frostvein's 7-2 ledger.
    # (frostvein action item T2, handed to the forge because this script is
    # forge-owned and must stay byte-identical downstream.)

    @staticmethod
    def _turn(model, inp, cw, cr, out, ts):
        return json.dumps(
            {
                "timestamp": ts,
                "message": {
                    "model": model,
                    "usage": {
                        "input_tokens": inp,
                        "cache_creation_input_tokens": cw,
                        "cache_read_input_tokens": cr,
                        "output_tokens": out,
                    },
                },
            }
        )

    @contextlib.contextmanager
    def _forge(self):
        """A throwaway forge root + Claude transcript dir, so main() drives the real
        cursor file and the real ledger writer without touching either for real."""
        with tempfile.TemporaryDirectory() as root, tempfile.TemporaryDirectory() as proj:
            with unittest.mock.patch.object(st, "_forge_root", return_value=root), \
                 unittest.mock.patch.object(st, "_claude_project_dir", return_value=proj):
                yield Path(root), Path(proj) / "sess-abc.jsonl"

    def _main(self, *argv):
        # Both streams: the refusals print to stderr (a wrapper capturing stdout for the
        # breakdown would otherwise see an empty error next to rc 2), and `self.err` keeps
        # them separable for the test that asserts exactly that.
        out, err = io.StringIO(), io.StringIO()
        with unittest.mock.patch.object(sys, "argv", ["session_tokens.py", *argv]):
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                rc = st.main()
        self.out, self.err = out.getvalue(), err.getvalue()
        return rc, self.out + self.err

    @staticmethod
    def _fan_out(transcript, *lines):
        """A Task-tool subagent transcript beside the main one. Without this every fixture
        is a single file where `s == primary_only`, and the whole class of fan-out bugs
        --mark exists to avoid is unobservable — a cursor stamped from the primary chain
        alone passes a suite built only from single-file transcripts."""
        d = transcript.parent / transcript.name[: -len(".jsonl")] / "subagents"
        d.mkdir(parents=True, exist_ok=True)
        (d / "agent-1.jsonl").write_text("".join(line + "\n" for line in lines))

    def _cursors(self, root):
        with unittest.mock.patch.object(st, "_forge_root", return_value=str(root)):
            return st._load_cursors()

    def test_mark_then_record_bills_only_the_post_mark_window(self):
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            rc, out = self._main("--mark")
            self.assertEqual(rc, 0, out)
            # The discarded window is named out loud — a mark that vanished silently
            # would erase the tokens from all accounting with nothing to show for it.
            self.assertIn("Discarded", out)
            self.assertIn("unattributed", out)

            with transcript.open("a") as fh:
                fh.write(self._turn("claude-opus-4-8", 5, 50, 700, 15, "2026-08-24T10:00:00Z") + "\n")
            rc, out = self._main("--story", "ep-99-us-01-demo", "--phase", "dev")
            self.assertEqual(rc, 0, out)

            ledger = (root / "_bmad-output" / "implementation-artifacts" / "metrics"
                      / "ep-99-us-01-demo.md").read_text()
        rows = [r for r in ledger.splitlines() if r.startswith("| dev |")]
        self.assertEqual(len(rows), 1, ledger)
        # Only the post-mark turn is billed: 1 turn, not 2, and the pre-mark tokens
        # (1,000 cache_read) appear in no row anywhere in the ledger.
        # Explicitly, cell by cell. `assertNotIn("1,000")` looked like the guard here and
        # was not one: a build that billed both turns writes 1,700, which contains neither
        # "1,000" nor "1000", so both assertions passed on the broken code.
        cells = [c.strip() for c in rows[0].strip("|").split("|")]
        self.assertEqual(cells[:8], ["dev", "claude", "claude-opus-4-8", "1", "5", "50", "700", "15"])

    def test_mark_writes_a_cursor_identical_in_shape_to_a_recorded_one(self):
        """Trap 1: a partial cursor makes every LATER row on the transcript wrong,
        which is worse than the gap being fixed because it looks precise."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self.assertEqual(self._main("--mark")[0], 0)
            marked = self._cursors(root)["sess-abc.jsonl"]

        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self.assertEqual(self._main("--story", "s", "--phase", "dev")[0], 0)
            recorded = self._cursors(root)["sess-abc.jsonl"]

        # `marked_at` is the one field that must NOT match: only a mark sets it, and a row
        # clears it. Everything else is compared, which is the trap-1 property.
        self.assertEqual({k: v for k, v in marked.items() if k != "marked_at"},
                         {k: v for k, v in recorded.items() if k != "marked_at"},
                         "a mark must write the same cursor a row writes")
        self.assertIsNotNone(marked["marked_at"], "a mark must stamp wall-clock now")
        self.assertIsNotNone(st._parse_ts(marked["marked_at"]).tzinfo, "must be tz-aware UTC")
        self.assertIsNone(recorded["marked_at"], "a row must CLEAR any mark it supersedes")
        self.assertIs(marked["primary_only"], False)
        self.assertIs(recorded["primary_only"], False)
        for bucket in st._CURSOR_BUCKETS:
            self.assertIn(bucket, marked)
        self.assertEqual(marked["schema"], st._CURSOR_SCHEMA)
        self.assertEqual(marked["by_model"], {"claude-opus-4-8":
                                              {"input": 10, "cache_creation": 100,
                                               "cache_read": 1000, "output": 20}})
        self.assertEqual(marked["last_ts"], "2026-08-24T09:00:00Z")
        self.assertIn("quota_last", marked)

    def test_mark_refuses_no_nested(self):
        """Trap 2: the cursor stores the fan-out-inclusive cumulative. A mark that
        stamped a primary-only one would inflate every later delta by the whole
        fan-out — and with no row written, nothing would show that it had."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            rc, out = self._main("--mark", "--no-nested")
            self.assertEqual(rc, 2)
            self.assertIn("--no-nested", out)
            self.assertEqual(self._cursors(root), {}, "a refused mark must not write a cursor")

    def test_mark_refuses_to_also_record_a_row(self):
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            rc, out = self._main("--mark", "--story", "s", "--phase", "dev")
            self.assertEqual(rc, 2)
            self.assertEqual(self._cursors(root), {})

    def test_mark_refuses_to_also_build_a_rollup(self):
        """--rollup returns long before the mark branch, so an unguarded combination
        would silently DROP the mark while exiting 0 — the caller would believe a
        boundary had been set when none was."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            rc, out = self._main("--mark", "--rollup", "ep-99")
            self.assertEqual(rc, 2, out)
            self.assertEqual(self._cursors(root), {})

    def test_mark_refuses_a_v1_cursor(self):
        """Trap 3: the v1->v2 rebase path exists to bill the primary-chain delta into a
        ROW while naming what it skipped. A mark writes no row, so marking a v1 cursor
        would discard the entire fan-out backlog with nothing to name it. Refuse, and
        say that recording a row first is the way through."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            # There has to BE a backlog for the refusal to be protecting anything — see
            # test_mark_accepts_a_v1_cursor_when_there_is_no_fan_out_to_lose.
            self._fan_out(transcript, self._turn("claude-opus-4-8", 1, 2, 3, 4, "2026-08-24T09:05:00Z"))
            v1 = {"turns": 1, "input": 1, "cache_creation": 1, "cache_read": 1, "output": 1}
            with unittest.mock.patch.object(st, "_forge_root", return_value=str(root)):
                st._save_cursors({"sess-abc.jsonl": dict(v1)})
            rc, out = self._main("--mark")
            self.assertEqual(rc, 2)
            self.assertIn("v1", out)
            self.assertEqual(self._cursors(root)["sess-abc.jsonl"], v1, "cursor left untouched")

    # --- what the fixtures above could not see -----------------------------------

    @staticmethod
    def _seed(root, cursor):
        with unittest.mock.patch.object(st, "_forge_root", return_value=str(root)):
            st._save_cursors({"sess-abc.jsonl": cursor})

    def test_mark_stamps_a_fan_out_inclusive_cursor(self):
        """The cursor must hold main + subagents. Every fixture above is a single file,
        where `s == primary_only` and `cursor_from(primary_only)` passes unnoticed."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self._fan_out(
                transcript, self._turn("claude-opus-4-8", 1, 2, 3, 4, "2026-08-24T09:05:00Z")
            )
            rc, out = self._main("--mark")
            self.assertEqual(rc, 0, out)
            self.assertIn("subagent transcript", out, "the fan-out must be named in the summary")
            cursor = self._cursors(root)["sess-abc.jsonl"]
        self.assertEqual(
            {k: cursor[k] for k in st._CURSOR_BUCKETS},
            {"turns": 2, "input": 11, "cache_creation": 102, "cache_read": 1003, "output": 24},
            "a primary-only cursor here inflates every later delta by the whole fan-out",
        )

    def test_mark_against_an_existing_cursor_discards_only_the_new_window(self):
        """The discarded figures are the ONLY record of the window. A mark that reported
        the whole cumulative — or reported nothing — would be indistinguishable here."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self.assertEqual(self._main("--story", "ep-99-us-01-demo", "--phase", "create")[0], 0)
            with transcript.open("a") as fh:
                fh.write(self._turn("claude-opus-4-8", 5, 50, 700, 15, "2026-08-24T10:00:00Z") + "\n")
            rc, out = self._main("--mark")
        self.assertEqual(rc, 0, out)
        discarded = out.split("Discarded", 1)[1]
        self.assertIn("(1 turns,", discarded)
        self.assertRegex(discarded, r"input \(fresh\)\s+5\b")
        self.assertRegex(discarded, r"cache creation\s+50\b")
        self.assertRegex(discarded, r"cache read\s+700\b")
        self.assertRegex(discarded, r"output\s+15\b")

    def test_mark_accepts_a_v1_cursor_when_there_is_no_fan_out_to_lose(self):
        """The v1 refusal guards a fan-out backlog. With no `subagents/` dir there is none
        — the cursor is already complete — and refusing would force exactly the row --mark
        exists to avoid, on 72% of the live cursors."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self._seed(root, {"turns": 1, "input": 1, "cache_creation": 1,
                              "cache_read": 1, "output": 1})
            rc, out = self._main("--mark")
            self.assertEqual(rc, 0, out)
            cursor = self._cursors(root)["sess-abc.jsonl"]
        self.assertEqual(cursor["schema"], st._CURSOR_SCHEMA, "the mark rebases it to v2")
        self.assertRegex(out.split("Discarded", 1)[1], r"cache read\s+999\b")

    def test_mark_refuses_a_primary_only_cursor(self):
        """`schema: 2` does not mean "fan-out-inclusive" on its own — a row recorded with
        --no-nested stamps one too. Marking from that baseline discards a window computed
        against the wrong cumulative, silently."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self._fan_out(
                transcript, self._turn("claude-opus-4-8", 1, 2, 3, 4, "2026-08-24T09:05:00Z")
            )
            rc, _ = self._main("--no-nested", "--story", "ep-99-us-01-demo", "--phase", "create")
            self.assertEqual(rc, 0)
            recorded = self._cursors(root)["sess-abc.jsonl"]
            self.assertIs(recorded["primary_only"], True, "the flag records which cumulative")
            rc, out = self._main("--mark")
            self.assertEqual(rc, 2, out)
            self.assertIn("--no-nested", out)
            self.assertEqual(self._cursors(root)["sess-abc.jsonl"], recorded, "left untouched")

    def test_a_cursor_predating_the_flag_marks_with_the_assumption_named(self):
        """Absent is UNKNOWN, not "complete". Read as complete — refusing would block every
        pre-patch v2 cursor behind a row — but never silently."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self._seed(root, {"turns": 1, "input": 1, "cache_creation": 1, "cache_read": 1,
                              "output": 1, "schema": 2, "by_model": {}, "last_ts": None,
                              "quota_last": None})
            rc, out = self._main("--mark")
        self.assertEqual(rc, 0, out)
        self.assertIn("predates the primary_only flag", out)

    def test_marked_at_anchors_the_next_row_and_a_row_then_clears_it(self):
        """`last_ts` is the last TURN, so a span measured from it bills the idle gap between
        the mark and the next phase to that next phase. And the clearing half is the easy
        one to miss: a stale mark would keep anchoring spans long after a row moved on."""
        ledger = None
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            with unittest.mock.patch.object(st, "datetime", _ClockAt0930):
                self.assertEqual(self._main("--mark")[0], 0)
            self.assertIsNotNone(
                st._parse_ts(self._cursors(root)["sess-abc.jsonl"]["marked_at"]).tzinfo,
                "marked_at must be timezone-aware UTC, per the file's own rule",
            )
            with transcript.open("a") as fh:
                fh.write(self._turn("claude-opus-4-8", 5, 50, 700, 15, "2026-08-24T10:00:00Z") + "\n")
            self.assertEqual(self._main("--story", "ep-99-us-01-demo", "--phase", "dev")[0], 0)
            self.assertIsNone(self._cursors(root)["sess-abc.jsonl"]["marked_at"],
                              "the row supersedes the mark and must clear it")
            with transcript.open("a") as fh:
                fh.write(self._turn("claude-opus-4-8", 5, 50, 700, 15, "2026-08-24T11:00:00Z") + "\n")
            self.assertEqual(self._main("--story", "ep-99-us-01-demo", "--phase", "review")[0], 0)
            ledger = str(root / "_bmad-output" / "implementation-artifacts" / "metrics"
                         / "ep-99-us-01-demo.md")
            rows = st.parse_ledger_rows(ledger)
        self.assertEqual([r["phase"] for r in rows], ["dev", "review"])
        # 09:30 (the mark) -> 10:00, not 09:00 (the last turn) -> 10:00.
        self.assertEqual(rows[0]["minutes"], 30.0)
        # 10:00 -> 11:00. A mark left uncleared would anchor this at 09:30 and read 90.
        self.assertEqual(rows[1]["minutes"], 60.0)

    def test_unreadable_cursor_file_is_a_hard_error_not_an_empty_dict(self):
        """Swallowing it into `{}` makes one run bill every transcript's whole history and
        then rewrite the file from that run alone — a total loss that exits 0."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            path = root / "_bmad-output" / "implementation-artifacts" / "metrics" / ".session-cursors.json"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('{"sess-abc.jsonl": {"turns": 1,')
            rc, out = self._main("--mark")
            self.assertEqual(rc, 2, out)
            self.assertIn("could not be read", out)
            self.assertEqual(path.read_text(), '{"sess-abc.jsonl": {"turns": 1,',
                             "a refused run must not rewrite the file it could not read")

    def test_malformed_cursor_entry_is_named_not_a_traceback(self):
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            path = root / "_bmad-output" / "implementation-artifacts" / "metrics" / ".session-cursors.json"
            path.parent.mkdir(parents=True, exist_ok=True)
            for bad in ('{"sess-abc.jsonl": 7}', '{"sess-abc.jsonl": {"schema": "two"}}'):
                path.write_text(bad)
                rc, out = self._main("--mark")
                self.assertEqual(rc, 2, out)
                self.assertIn("sess-abc.jsonl", out)
                self.assertNotIn("Traceback", out)

    def test_a_failed_save_leaves_the_existing_cursors_intact(self):
        """Write-then-rename. A whole-file rewrite truncates before it writes, so a crash
        mid-save leaves a file that _load_cursors now refuses — turning one bad run into a
        hard stop for every transcript."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self._seed(root, {"turns": 1, "input": 1, "cache_creation": 1, "cache_read": 1,
                              "output": 1, "schema": 2, "primary_only": False, "by_model": {},
                              "last_ts": None, "quota_last": None, "marked_at": None})
            path = root / "_bmad-output" / "implementation-artifacts" / "metrics" / ".session-cursors.json"
            before = path.read_text()
            with unittest.mock.patch.object(st.json, "dump", side_effect=OSError("disk full")):
                with self.assertRaises(OSError):
                    self._main("--mark")
            self.assertEqual(path.read_text(), before)

    def test_mark_refuses_an_empty_story_or_phase(self):
        """A wrapper with an unset variable passes `--phase ""`. Truthiness let that slip
        through as a silent mark — the one outcome that leaves nothing to notice."""
        for argv in (("--mark", "--phase", ""), ("--mark", "--story", ""),
                     ("--mark", "--story", "", "--phase", "")):
            with self._forge() as (root, transcript):
                transcript.write_text(
                    self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
                )
                rc, out = self._main(*argv)
                self.assertEqual(rc, 2, f"{argv}: {out}")
                self.assertEqual(self._cursors(root), {}, argv)

    def test_mark_refuses_a_metrics_file(self):
        """The fourth row-writing flag; the other three are hard refusals. Accepting it
        says a ledger path was honoured when no ledger was touched."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            rc, out = self._main("--mark", "--metrics-file", str(root / "led.md"))
            self.assertEqual(rc, 2, out)
            self.assertIn("--metrics-file", out)
            self.assertEqual(self._cursors(root), {})

    def test_refusals_go_to_stderr(self):
        """A wrapper capturing stdout for the breakdown otherwise shows an empty error
        beside rc 2."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            self.assertEqual(self._main("--mark", "--no-nested")[0], 2)
        self.assertEqual(self.out, "", "nothing on stdout")
        self.assertIn("--no-nested", self.err)

    def test_rollup_refusal_is_reported_before_the_others(self):
        """--rollup is the flag that would be DROPPED whole; a caller passing two rejected
        flags should hear about that one, not only about the second."""
        with self._forge() as (root, transcript):
            transcript.write_text(
                self._turn("claude-opus-4-8", 10, 100, 1000, 20, "2026-08-24T09:00:00Z") + "\n"
            )
            rc, out = self._main("--mark", "--rollup", "ep-99", "--no-nested")
            self.assertEqual(rc, 2, out)
            self.assertIn("--rollup", out)
            self.assertNotIn("--no-nested", out)


class _ClockAt0930(datetime):
    """A frozen wall clock for the `marked_at` axis — the mark's own timestamp is the one
    figure in this file that does not come from a transcript."""

    @classmethod
    def now(cls, tz=None):
        return datetime(2026, 8, 24, 9, 30, tzinfo=timezone.utc)


class PricingTests(unittest.TestCase):
    def test_picks_priced_model_over_synthetic(self):
        # This test is about SELECTION — skip Claude Code's unpriced `<synthetic>`
        # pseudo-model and pick the real one. It used to assert `rates["input"] == 15.0`,
        # which coupled it to a price: when the stale Opus row was corrected $15 -> $5 on
        # 2026-08-01 the fix shipped WITHOUT its test, and this suite went red and stayed
        # red, unnoticed, because nothing runs it. Assert the row that was chosen instead.
        rates = st._rates_for(["<synthetic>", "claude-opus-4-8"])
        self.assertIs(rates, st.PRICES["opus"])
        self.assertIsNone(st._rates_for(["<synthetic>"]))

    def test_current_rates_are_pinned_deliberately(self):
        # The guard the above test should never have been doing. Prices are a DECISION:
        # changing one must break a test so it is updated on purpose, not discovered a
        # month later in a retro. (Every review row recorded before 2026-08-01 is ~3x
        # overstated precisely because a rate changed with nothing watching.)
        # Hand-written literals on purpose — never assert PRICES against itself.
        self.assertEqual(st.PRICES["opus"], {"input": 5.0, "cache_write": 6.25, "cache_read": 0.50, "output": 25.0})
        self.assertEqual(st.PRICES["fable"], {"input": 10.0, "cache_write": 12.50, "cache_read": 1.0, "output": 50.0})


class ClaudeParsingTests(unittest.TestCase):
    def test_sum_and_shared_shape(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "c.jsonl"
            path.write_text(
                _claude_turn("claude-opus-4-8", 10, 100, 1000, 20)
                + "\n"
                + _claude_turn("claude-opus-4-8", 5, 50, 500, 10)
                + "\n"
            )
            s = st.sum_claude_transcript(str(path))
            self.assertEqual((s["turns"], s["input"], s["cache_read"]), (2, 15, 1500))
            # The shared shape is asserted EXACTLY, on purpose: it is the merge point where
            # the forge's by_model and frostvein's wall-clock/quota axes met, and a missed
            # key must fail loudly rather than surface as a silent `—` in every ledger.
            self.assertEqual(
                set(s),
                {
                    "turns", "models", "input", "cache_creation", "cache_read", "output",
                    "total", "by_model", "first_ts", "last_ts", "quota_first", "quota_last",
                    "quota_pp", "counted_transcripts",
                },
            )
            self.assertEqual(s["by_model"]["claude-opus-4-8"]["cache_read"], 1500)


def _session(root: Path, session_id: str, main_turns, agents=()):
    """Lay out a Claude Code session on disk the way the CLI does: the main transcript
    at ``<dir>/<id>.jsonl`` and each subagent at ``<dir>/<id>/subagents/agent-*.jsonl``."""
    (root / f"{session_id}.jsonl").write_text("\n".join(main_turns) + "\n")
    sub = root / session_id / "subagents"
    sub.mkdir(parents=True, exist_ok=True)
    for i, turns in enumerate(agents):
        (sub / f"agent-{i}.jsonl").write_text("\n".join(turns) + "\n")
    return str(root / f"{session_id}.jsonl")


class SubagentAccountingTests(unittest.TestCase):
    """A1a / A5-bis: subagent transcripts are a sibling tree the meter never opened."""

    def test_session_sums_main_plus_subagents(self):
        with tempfile.TemporaryDirectory() as d:
            main = _session(
                Path(d),
                "sess",
                [_claude_turn("claude-opus-5", 10, 100, 1000, 20)],
                agents=[
                    [_claude_turn("claude-opus-5", 1, 10, 100, 2)],
                    [_claude_turn("claude-opus-5", 2, 20, 200, 4)],
                ],
            )
            session, main_only, n = st.sum_claude_session(main)
            self.assertEqual(n, 2)
            self.assertEqual(main_only["total"], 1130)
            self.assertEqual(session["turns"], 3)
            self.assertEqual(session["total"], 1130 + 113 + 226)
            # The regression this guards: summing the main file alone.
            self.assertGreater(session["total"], main_only["total"])

    def test_no_subagents_is_unchanged(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "solo.jsonl"
            p.write_text(_claude_turn("claude-opus-5", 10, 100, 1000, 20) + "\n")
            session, main_only, n = st.sum_claude_session(str(p))
            self.assertEqual(n, 0)
            self.assertEqual(session, main_only)

    def test_newest_transcript_never_picks_a_subagent_file(self):
        with tempfile.TemporaryDirectory() as d:
            _session(Path(d), "sess", [_claude_turn("claude-opus-5", 1, 1, 1, 1)],
                     agents=[[_claude_turn("claude-opus-5", 1, 1, 1, 1)]])
            # Make the subagent file the newest thing on disk (an interrupted run).
            agent = Path(d) / "sess" / "subagents" / "agent-0.jsonl"
            os.utime(agent, (2**31 - 1, 2**31 - 1))
            self.assertEqual(
                os.path.basename(st._newest_transcript(d, recursive=False)), "sess.jsonl"
            )

    def test_meta_json_siblings_are_not_read_as_transcripts(self):
        # A `.meta.json` sits beside every agent transcript and carries no usage. The glob
        # is `agent-*.jsonl` precisely so it falls out — assert it, don't assume it.
        with tempfile.TemporaryDirectory() as d:
            main = _session(Path(d), "sess", [_claude_turn("claude-opus-5", 1, 1, 1, 1)],
                            agents=[[_claude_turn("claude-opus-5", 2, 2, 2, 2)]])
            (Path(d) / "sess" / "subagents" / "agent-0.meta.json").write_text('{"note": "not one"}')
            found = st.subagent_transcripts(main)
        self.assertEqual([Path(f).name for f in found], ["agent-0.jsonl"])

    def test_cursor_schema_is_pinned_and_shared_with_frostvein(self):
        # Hand-written literal on purpose. Both forges reached schema 2 with the SAME rebase
        # semantics independently; drifting the number silently would make the two
        # implementations unmergeable while still looking equivalent.
        self.assertEqual(st._CURSOR_SCHEMA, 2)

    def test_cursor_rebase_does_not_dump_history_into_the_next_row(self):
        # A v1 cursor (main-only) against a v2 cumulative (main+subagents): the delta
        # must be the main-chain progress, NOT the whole subagent backlog.
        main_only = st._as_summary(
            10, ["claude-opus-5"], {"input": 10, "cache_creation": 0, "cache_read": 100, "output": 10}
        )
        v1_cursor = {"turns": 8, "input": 8, "cache_creation": 0, "cache_read": 80, "output": 8}
        delta, reset = st.delta_since_cursor(main_only, v1_cursor)
        self.assertFalse(reset)
        self.assertEqual(delta["turns"], 2)
        self.assertEqual(delta["total"], 24)  # 2 + 0 + 20 + 2 — no subagent lump


class PerModelPricingTests(unittest.TestCase):
    """Second live defect found alongside A1a: flat pricing on a mixed-model session."""

    def test_mixed_models_price_at_their_own_rates(self):
        s = st._as_summary(
            2,
            ["claude-fable-5", "claude-opus-5"],
            {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 2_000_000},
            {
                "claude-opus-5": {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 1_000_000},
                "claude-fable-5": {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 1_000_000},
            },
        )
        # opus $25/Mtok out + fable $50/Mtok out = $75.
        self.assertAlmostEqual(st.estimate_usd(s), 75.0)
        # The old flat path took the first *sorted* model — fable — for everything: $100.
        flat = dict(s)
        flat["by_model"] = {}
        self.assertAlmostEqual(st.estimate_usd(flat), 100.0)

    def test_unpriced_model_with_real_spend_refuses_to_guess(self):
        s = st._as_summary(
            1, ["some-new-model"], {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 500},
            {"some-new-model": {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 500}},
        )
        self.assertIsNone(st.estimate_usd(s))

    def test_zero_token_synthetic_model_is_ignored(self):
        s = st._as_summary(
            1, ["<synthetic>", "claude-opus-5"],
            {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 1_000_000},
            {
                "claude-opus-5": {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 1_000_000},
                "<synthetic>": {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 0},
            },
        )
        self.assertAlmostEqual(st.estimate_usd(s), 25.0)


_LEDGER = (
    "# Token metrics — ep-09-us-01-demo\n\nprose line that is not a table.\n\n"
    "| phase | tool | model | turns | input | cache_create | cache_read | output | total | est_usd | transcript | recorded |\n"
    "|---|---|---|---|---|---|---|---|---|---|---|---|\n"
    "| create | claude | claude-opus-4-8 | 55 | 22,391 | 271,857 | 3,069,875 | 91,755 | 3,455,878 | $16.92 | `a.jsonl` | t |\n"
    "| dev | codex | gpt-5.5 | 27 | 177,178 | 0 | 2,083,456 | 16,690 | 2,277,324 | $0.65 | `b.jsonl` | t |\n"
    "| review | claude | claude-opus-4-8 | 40 | 1,000 | 2,000 | 3,000 | 4,000 | 10,000 | — | `c.jsonl` | t |\n"
)

_THIN = (
    "| phase | tool | model | turns | input | cache_create | cache_read | output | total | est_usd | transcript | recorded |\n"
    "|---|---|---|---|---|---|---|---|---|---|---|---|\n"
    "| create | claude | m | 1 | 1 | 1 | 1 | 1 | 4 | $1.00 | `c.jsonl` | t |\n"
    "| dev | codex | m | 1 | 1 | 0 | 1 | 1 | 3 | $0.50 | `d.jsonl` | t |\n"
    "| review | — | unrecoverable | — | — | — | — | — | — | — | mixed `r.jsonl` | t |\n"
)


class RollupTests(unittest.TestCase):
    def test_parse_skips_prose_and_coerces_cells(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "ep-09-us-01-demo.md"
            p.write_text(_LEDGER)
            rows = st.parse_ledger_rows(str(p))
            self.assertEqual([r["phase"] for r in rows], ["create", "dev", "review"])
            self.assertEqual(rows[0]["input"], 22391)  # commas stripped
            self.assertEqual(rows[0]["est_usd"], 16.92)
            self.assertIsNone(rows[2]["est_usd"])  # "—" -> None, row still counted

    def test_rollup_separates_gap_unrecoverable_priced(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "ep-09-us-01-demo.md").write_text(_LEDGER)
            (Path(d) / "ep-09-us-02-thin.md").write_text(_THIN)
            # an annotation-only story: create row present, dev+review absent.
            (Path(d) / "ep-09-us-03-bare.md").write_text(
                "| phase | tool | model | turns | input | cache_create | cache_read | output | total | est_usd | transcript | recorded |\n"
                "|---|---|---|---|---|---|---|---|---|---|---|---|\n"
                "| create | claude | m | 1 | 1 | 1 | 1 | 1 | 4 | $1.00 | `c.jsonl` | t |\n"
            )
            (Path(d) / "ep-09-rollup.md").write_text("# stale rollup, must be excluded\n")
            roll = st.build_rollup(str(d), "ep-09")
            self.assertEqual(
                set(roll["per_story"]), {"ep-09-us-01-demo", "ep-09-us-02-thin", "ep-09-us-03-bare"}
            )
            self.assertIn(("ep-09-us-02-thin", "review"), roll["unrecoverable"])
            self.assertNotIn(("ep-09-us-02-thin", "review"), roll["gaps"])
            self.assertIn(("ep-09-us-03-bare", "dev"), roll["gaps"])  # truly absent = gap
            self.assertNotIn(("ep-09-us-01-demo", "create"), roll["gaps"])

    def test_missing_live_gate_is_reported_not_silent(self):
        # A1d: the rollup used to print "No gaps" over stories with no live-gate row.
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "ep-09-us-01-demo.md").write_text(_LEDGER)  # create/dev/review, no live-gate
            roll = st.build_rollup(str(d), "ep-09")
            self.assertIn("ep-09-us-01-demo", roll["no_live_gate"])
            md = st.render_rollup(roll)
            self.assertIn("No `live-gate` row", md)
            self.assertNotIn("**No gaps**", md)

    def test_live_gate_row_clears_the_flag(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "ep-09-us-01-demo.md").write_text(
                _LEDGER
                + "| live-gate | claude | claude-opus-5 | 9 | 1 | 1 | 1 | 1 | 4 | $2.00 | `g.jsonl` | t |\n"
            )
            roll = st.build_rollup(str(d), "ep-09")
            self.assertEqual(roll["no_live_gate"], [])

    def test_epic_ledger_is_exempt_from_story_checks(self):
        # A1e: retro/planning cost has a home (`<epic>-epic.md`) and must not be judged
        # against the per-story triad — an epic has no dev phase.
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "ep-09-epic.md").write_text(
                "| phase | tool | model | turns | input | cache_create | cache_read | output | total | est_usd | transcript | recorded |\n"
                "|---|---|---|---|---|---|---|---|---|---|---|---|\n"
                "| retro | claude | claude-opus-5 | 5 | 1 | 1 | 1 | 1 | 4 | $3.00 | `r.jsonl` | t |\n"
            )
            roll = st.build_rollup(str(d), "ep-09")
            self.assertIn("ep-09-epic", roll["per_story"])
            self.assertEqual(roll["gaps"], [])
            self.assertEqual(roll["no_live_gate"], [])
            self.assertIn("$3.00", st.render_rollup(roll))

    def test_render_marks_na_and_total(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "ep-09-us-02-thin.md").write_text(_THIN)
            md = st.render_rollup(st.build_rollup(str(d), "ep-09"))
            self.assertIn("n/a", md)  # unrecoverable cell
            self.assertIn("Unrecoverable (annotated, not silent)", md)
            self.assertIn("**n/a**", md)  # all-unrecoverable review column total


class DurationTests(unittest.TestCase):
    """Wall-clock is the third axis: cost cannot tell a phase that thought hard from one
    that STALLED. A frostvein review layer that hung for 2.5h was nearly free in dollars."""

    def test_minutes_between_handles_missing_and_backwards(self):
        self.assertEqual(st._minutes_between("2026-08-04T13:05:08Z", "2026-08-04T14:42:08Z"), 97.0)
        self.assertIsNone(st._minutes_between(None, "2026-08-04T14:42:08Z"))
        self.assertIsNone(st._minutes_between("2026-08-04T13:05:08Z", None))
        # A clock that ran backwards must report nothing rather than a negative duration.
        self.assertIsNone(st._minutes_between("2026-08-04T14:42:08Z", "2026-08-04T13:05:08Z"))

    def test_delta_window_starts_where_the_previous_record_stopped(self):
        # A review that follows a create on the same transcript must not inherit the
        # create's elapsed time — the same rule tokens already follow.
        cumulative = st._as_summary(
            100, ["claude-opus-4-8"],
            {"input": 10, "cache_creation": 20, "cache_read": 30, "output": 40},
            span=("2026-08-04T10:00:00Z", "2026-08-04T12:00:00Z"),
        )
        prev = {"turns": 40, "input": 4, "cache_creation": 8, "cache_read": 12, "output": 16,
                "last_ts": "2026-08-04T11:00:00Z"}
        delta, reset = st.delta_since_cursor(cumulative, prev)
        self.assertFalse(reset)
        self.assertEqual(delta["turns"], 60)
        self.assertEqual(st._minutes_between(delta["first_ts"], delta["last_ts"]), 60.0)

    def test_cursor_written_before_last_ts_existed_yields_no_duration(self):
        cumulative = st._as_summary(
            10, ["claude-opus-4-8"], {"input": 1, "cache_creation": 1, "cache_read": 1, "output": 1},
            span=("2026-08-04T10:00:00Z", "2026-08-04T12:00:00Z"),
        )
        legacy = {"turns": 5, "input": 0, "cache_creation": 0, "cache_read": 0, "output": 0}
        delta, _ = st.delta_since_cursor(cumulative, legacy)
        self.assertIsNone(st._minutes_between(delta["first_ts"], delta["last_ts"]))

    def test_append_inserts_after_last_table_row_not_at_eof(self):
        """A ledger legitimately grows prose BELOW its table (closure notes, caveats).
        An EOF append lands the new row under that prose, where markdown renders it as
        literal text — it corrupted a real ledger twice in one session on 2026-08-28."""
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "led.md")
            s = st._as_summary(
                3, ["claude-opus-4-8"], {"input": 1, "cache_creation": 2, "cache_read": 3, "output": 4},
            )
            st.append_ledger(path, "ep-09-us-01-demo", "create", "claude", s, "t.jsonl")
            prose = "\nStory total $1.23 across 1 row. The dev row excludes the reverted spike.\n"
            with open(path, "a", encoding="utf-8") as fh:
                fh.write(prose)
            st.append_ledger(path, "ep-09-us-01-demo", "review", "claude", s, "t.jsonl")
            text = Path(path).read_text(encoding="utf-8")
            self.assertTrue(text.endswith(prose), "prose below the table must stay last")
            lines = text.splitlines()
            review_idx = next(i for i, ln in enumerate(lines) if ln.startswith("| review |"))
            self.assertTrue(
                lines[review_idx - 1].startswith("| create |"),
                "new row must land directly under the last table row",
            )
            self.assertEqual([r["phase"] for r in st.parse_ledger_rows(path)], ["create", "review"])

    def test_ledger_row_round_trips_minutes(self):
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "led.md")
            s = st._as_summary(
                3, ["claude-opus-4-8"], {"input": 1, "cache_creation": 2, "cache_read": 3, "output": 4},
                span=("2026-08-04T10:00:00Z", "2026-08-04T10:30:00Z"),
            )
            st.append_ledger(path, "ep-09-us-01-demo", "review", "claude", s, "t.jsonl")
            rows = st.parse_ledger_rows(path)
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0]["minutes"], 30.0)
            self.assertEqual(rows[0]["phase"], "review")


class QuotaTests(unittest.TestCase):
    """Codex bills a weekly QUOTA, not tokens, so `est_usd` measures the NON-binding axis
    for every delegated dev row. It binds twice over here: nidavellir's court brain draws
    on the same weekly pool, so a dev handoff can starve a live gate."""

    def test_reads_percentage_points_consumed_across_the_rollout(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rollout-q.jsonl"
            lines = _codex_token_count("gpt-5.6-sol", 100, 80, 5, used_percent=40.0)
            lines += _codex_token_count("gpt-5.6-sol", 1000, 900, 25, used_percent=100.0)[1:]
            path.write_text("\n".join(lines) + "\n")
            s = st.sum_codex_transcript(str(path))
        self.assertEqual((s["quota_first"], s["quota_last"], s["quota_pp"]), (40.0, 100.0, 60.0))

    def test_absent_rate_limits_yield_none_not_zero(self):
        """A zero would read as 'this run was free'. It means 'not measured'."""
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rollout-noq.jsonl"
            path.write_text("\n".join(_codex_token_count("gpt-5.6-sol", 100, 80, 5)) + "\n")
            s = st.sum_codex_transcript(str(path))
        self.assertIsNone(s["quota_pp"])
        self.assertIsNone(s["quota_first"])

    def test_weekly_reset_inside_the_window_reads_blank_not_negative(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rollout-reset.jsonl"
            lines = _codex_token_count("gpt-5.6-sol", 100, 80, 5, used_percent=93.0)
            lines += _codex_token_count("gpt-5.6-sol", 1000, 900, 25, used_percent=16.0)[1:]
            path.write_text("\n".join(lines) + "\n")
            s = st.sum_codex_transcript(str(path))
        self.assertIsNone(s["quota_pp"], "a reset must not bill -77pp")

    def test_claude_transcripts_carry_no_quota(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "c.jsonl"
            path.write_text(_claude_turn("claude-opus-5", 10, 20, 30, 40) + "\n")
            s = st.sum_claude_transcript(str(path))
        self.assertIsNone(s["quota_pp"])

    def test_delta_bills_quota_from_where_the_last_record_stopped(self):
        prev = {"turns": 1, "input": 100, "cache_creation": 0, "cache_read": 80, "output": 5,
                "quota_last": 40.0}
        cum = st._as_summary(
            3, ["gpt-5.6-sol"], {"input": 1000, "cache_creation": 0, "cache_read": 900, "output": 25},
            quota=(40.0, 100.0),
        )
        delta, reset = st.delta_since_cursor(cum, prev)
        self.assertFalse(reset)
        self.assertEqual(delta["quota_pp"], 60.0, "bills 40->100, not 0->100")

    def test_cursor_predating_quota_falls_back_to_this_rollouts_first_sample(self):
        prev = {"turns": 1, "input": 100, "cache_creation": 0, "cache_read": 80, "output": 5}
        cum = st._as_summary(
            3, ["gpt-5.6-sol"], {"input": 1000, "cache_creation": 0, "cache_read": 900, "output": 25},
            quota=(40.0, 100.0),
        )
        delta, _ = st.delta_since_cursor(cum, prev)
        self.assertEqual(delta["quota_pp"], 60.0, "no invented floor when the cursor is old")

    def test_ledger_row_round_trips_quota(self):
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "led.md")
            s = st._as_summary(
                5, ["gpt-5.6-sol"], {"input": 10, "cache_creation": 0, "cache_read": 20, "output": 30},
                quota=(40.0, 100.0),
            )
            st.append_ledger(path, "ep-09-us-01-demo", "dev", "codex", s, "rollout-q.jsonl")
            rows = st.parse_ledger_rows(path)
        self.assertEqual(rows[-1]["quota_pp"], 60.0)

    def test_thirteen_cell_rows_predating_quota_still_parse_without_shifting(self):
        """`quota_pp` is APPENDED, never inserted — a row written when `minutes` was the
        last column must keep its minutes, not silently re-align by one."""
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "led.md")
            Path(path).write_text(
                _LEDGER
                + "| dev | codex | gpt-5.6-sol | 715 | 1,516,492 | 0 | 94,276,352 | "
                  "208,027 | 96,000,871 | $60.96 | `rollout-old.jsonl` | "
                  "2026-08-06 09:41 UTC · rates 2026-08-01 | 189 |\n"
            )
            rows = st.parse_ledger_rows(path)
        self.assertEqual(rows[-1]["minutes"], 189.0)
        self.assertIsNone(rows[-1]["quota_pp"], "a pre-quota row must read blank, not shifted")

    def test_real_rollout_line_is_pinned_verbatim(self):
        """A hand-copied line from a real codex-cli 0.146.0 rollout, NOT built by the
        fixture. The fixture is the author's belief about the wire shape; this is the wire
        shape. Frostvein's first cut read `info.rate_limits` — every synthetic test passed
        and the real transcript silently reported nothing."""
        line = (
            '{"timestamp":"2026-08-06T06:14:40.000Z","type":"event_msg","payload":'
            '{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,'
            '"cached_input_tokens":900,"output_tokens":25,"total_tokens":1025},'
            '"last_token_usage":{},"model_context_window":272000},'
            '"rate_limits":{"limit_id":"codex","limit_name":null,"primary":'
            '{"used_percent":40.0,"window_minutes":10080,"resets_at":1786518052},'
            '"secondary":null}}}'
        )
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rollout-real.jsonl"
            path.write_text(line + "\n")
            s = st.sum_codex_transcript(str(path))
        self.assertEqual(s["quota_last"], 40.0, "rate_limits sits beside info, not inside it")
        self.assertEqual(s["cache_read"], 900)


class NestedCodexRolloutTests(unittest.TestCase):
    """The Codex half of the fan-out defect: a `codex review` self-gate spawns its OWN
    sibling rollout rather than logging into the dev one. Same class as the invisible
    Claude subagents, different mechanism — fixing one does not fix the other."""

    def _rollout(self, d, sid, cwd, stamps, inp, cached, out):
        path = Path(d) / f"rollout-{sid}.jsonl"
        lines = [json.dumps({
            "timestamp": stamps[0], "type": "session_meta",
            "payload": {"session_id": sid, "cwd": cwd},
        })]
        for ts in stamps:
            lines.append(json.dumps({
                "timestamp": ts, "type": "event_msg",
                "payload": {"type": "token_count", "info": {"total_token_usage": {
                    "input_tokens": inp, "cached_input_tokens": cached,
                    "output_tokens": out, "total_tokens": inp + out,
                }}},
            }))
        path.write_text("\n".join(lines) + "\n")
        return path

    def test_nested_rollouts_in_the_same_cwd_and_window_are_counted(self):
        with tempfile.TemporaryDirectory() as d:
            here, there = "/workspace/projects/bifrost-client", "/workspace/projects/other"
            dev = self._rollout(d, "dev", here,
                                ["2026-08-06T06:00:00Z", "2026-08-06T09:00:00Z"], 1000, 900, 50)
            self._rollout(d, "selfgate", here,
                          ["2026-08-06T07:00:00Z", "2026-08-06T07:30:00Z"], 200, 180, 10)
            # Same window, DIFFERENT project — the contamination that cost frostvein's
            # story 3.2 quota figure a ~9pp caveat. Must not land in this story's row.
            self._rollout(d, "foreign", there,
                          ["2026-08-06T07:10:00Z", "2026-08-06T07:20:00Z"], 999, 999, 999)
            # Same project, days later — outside the window.
            self._rollout(d, "later", here,
                          ["2026-08-09T06:00:00Z", "2026-08-09T07:00:00Z"], 777, 777, 777)

            with unittest.mock.patch.object(st, "_codex_sessions_dir", return_value=d):
                nested = st.nested_codex_rollouts(str(dev))
                alone, _, n_alone = st.sum_codex_session(str(dev), include_nested=False)
                full, primary, n_full = st.sum_codex_session(str(dev))

        self.assertEqual([Path(p).name for p in nested], ["rollout-selfgate.jsonl"])
        self.assertEqual((n_alone, n_full), (0, 1))
        self.assertEqual(alone["counted_transcripts"], 1)
        self.assertEqual(full["counted_transcripts"], 2)
        self.assertEqual(full["cache_read"], 900 + 180)
        self.assertEqual(full["output"], 50 + 10)
        self.assertEqual(primary["cache_read"], 900, "primary_only stays the isolated number")

    def test_quota_is_enveloped_not_summed(self):
        """`used_percent` reads ONE account-wide counter. Adding two readings would
        double-count the same consumption."""
        zero = {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 0}
        a = st._as_summary(1, ["m"], zero, span=("2026-08-06T06:00:00Z", "2026-08-06T07:00:00Z"),
                           quota=(40.0, 55.0))
        b = st._as_summary(1, ["m"], zero, span=("2026-08-06T06:30:00Z", "2026-08-06T08:00:00Z"),
                           quota=(50.0, 70.0))
        merged = st._merge_summaries([a, b])
        self.assertEqual((merged["quota_first"], merged["quota_last"]), (40.0, 70.0))
        self.assertEqual(merged["quota_pp"], 30.0)  # NOT (55-40) + (70-50) = 35
        self.assertEqual((merged["first_ts"], merged["last_ts"]),
                         ("2026-08-06T06:00:00Z", "2026-08-06T08:00:00Z"))


class PreservationTests(unittest.TestCase):
    """A rollup accumulates hand-written analysis the generator cannot reproduce.
    `--rollup` used to overwrite it silently — the exact "rewriting recorded history"
    failure the ledgers themselves warn against. It ate a rate-correction table in
    frostvein before it was caught."""

    def test_tail_after_the_marker_survives_regeneration(self):
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "ep-09-rollup.md")
            Path(path).write_text("# old generated\n\n" + st._SENTINEL + "\n\n**Hand-written.**\n")
            merged = st._merge_preserved(path, "# fresh generated\n\n" + st._SENTINEL + "\n")
            self.assertIn("# fresh generated", merged)
            self.assertIn("**Hand-written.**", merged)
            self.assertNotIn("# old generated", merged)
            self.assertEqual(merged.count(st._SENTINEL), 1, "the marker must not accumulate")

    def test_regeneration_is_idempotent(self):
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "ep-09-rollup.md")
            gen = "# generated\n\n" + st._SENTINEL + "\n"
            Path(path).write_text(gen + "\n**Kept.**\n")
            for _ in range(3):
                merged = st._merge_preserved(path, gen)
                Path(path).write_text(merged)
            self.assertEqual(merged.count(st._SENTINEL), 1)
            self.assertEqual(merged.count("**Kept.**"), 1)

    def test_file_predating_the_marker_is_backed_up_and_announced(self):
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "ep-09-rollup.md")
            Path(path).write_text("# old\n\n**Analysis with no marker.**\n")
            said = io.StringIO()
            with contextlib.redirect_stdout(said):
                merged = st._merge_preserved(path, "# fresh\n\n" + st._SENTINEL + "\n")
            self.assertNotIn("**Analysis with no marker.**", merged)
            backup = Path(path + ".prev.md")
            self.assertTrue(backup.exists(), "unmergeable content must be backed up, never lost")
            self.assertIn("**Analysis with no marker.**", backup.read_text())
            # Backing it up silently would still lose it — nobody reads a file they were
            # never told about.
            self.assertIn("ep-09-rollup.md.prev.md", said.getvalue())


class LedgerWidthGuardTests(unittest.TestCase):
    """Ledgers legitimately carry prose tables (rate corrections, subagent-backfill
    annotations). Parsing those as metric rows invented phantom rollup phases."""

    def test_narrow_prose_tables_are_not_parsed_as_rows(self):
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "led.md")
            Path(path).write_text(
                _LEDGER
                + "\n**Rates were corrected; the row above reads 3x high.**\n\n"
                + "| row | as recorded | corrected |\n|---|---|---|\n"
                + "| review (claude, opus-5) | $66.19 | **$22.06** |\n"
            )
            rows = st.parse_ledger_rows(path)
            self.assertEqual([r["phase"] for r in rows], ["create", "dev", "review"])

    def test_legacy_twelve_cell_rows_still_parse_without_shifting(self):
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / "led.md")
            Path(path).write_text(_LEDGER)
            rows = st.parse_ledger_rows(path)
            self.assertEqual(rows[0]["est_usd"], 16.92)
            self.assertEqual(rows[0]["total"], 3455878)
            self.assertIsNone(rows[0]["minutes"], "a pre-minutes row must read blank, not shifted")


if __name__ == "__main__":
    unittest.main()


class QuotaWindowSelectionTests(unittest.TestCase):
    """`quota_pp` claims to be the 7-DAY window. Which JSON key holds it is NOT stable.

    On 2026-08-06 a real rollout carried `primary.window_minutes == 10080` (the weekly)
    with `secondary: null` — so reading `primary` was correct when this was written. By
    2026-08-31 Codex had added a 5-HOUR window as `primary` and moved the weekly to
    `secondary`, so the same code silently began recording 5h points under a weekly
    heading. It inverted a live-gate risk call on ep-16-us-03: reported "weekly at 96%,
    the gate may be starved" when the weekly actually had 85% left and only the
    self-clearing 5h window was spent.

    The fix is to select the window by `window_minutes`, never by key name — the duration
    is the meaning, the key is just where it happened to sit that month.
    """

    # Hand-copied from a real codex-cli 0.146.0 rollout, 2026-08-31. NOT fixture-built.
    TODAY_SHAPE = (
        '{"timestamp":"2026-08-31T08:36:09.010Z","type":"event_msg","payload":'
        '{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,'
        '"cached_input_tokens":900,"output_tokens":25,"total_tokens":1025}},'
        '"rate_limits":{"limit_id":"codex","limit_name":null,'
        '"primary":{"used_percent":96.0,"window_minutes":300,"resets_at":1788179782},'
        '"secondary":{"used_percent":15.0,"window_minutes":10080,"resets_at":1788766582},'
        '"plan_type":"plus"}}}'
    )

    def _sum(self, line):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "rollout-w.jsonl"
            path.write_text(line + "\n")
            return st.sum_codex_transcript(str(path))

    def test_weekly_comes_from_the_10080_minute_window_not_from_primary(self):
        s = self._sum(self.TODAY_SHAPE)
        self.assertEqual(s["quota_last"], 15.0,
                         "must read the 7-day window (secondary here), not the 5h primary")
        self.assertNotEqual(s["quota_last"], 96.0, "96% is the 5-HOUR window — the wrong axis")

    def test_weekly_still_found_when_it_sits_under_primary(self):
        """The 2026-08-06 layout. Selection must be by duration, so BOTH layouts work."""
        line = self.TODAY_SHAPE.replace(
            '"primary":{"used_percent":96.0,"window_minutes":300,"resets_at":1788179782},'
            '"secondary":{"used_percent":15.0,"window_minutes":10080,"resets_at":1788766582},',
            '"primary":{"used_percent":15.0,"window_minutes":10080,"resets_at":1788766582},'
            '"secondary":null,',
        )
        self.assertEqual(self._sum(line)["quota_last"], 15.0)

    def test_weekly_window_with_a_null_percentage_does_not_hand_the_axis_to_the_5h_one(self):
        """THE SHAPE THE FIX EXISTS FOR, and the one all three tests above miss.

        Every test above populates `used_percent` on every window, so the filter that
        drops percent-less windows never fired. Filtering on `used_percent` BEFORE
        selecting on `window_minutes` drops a weekly window that declares
        `used_percent: null` — and the 5-hour window then wins `max()` and is recorded
        under the weekly heading. That is precisely the defect a4b717d was written to
        close, and the one that already inverted a live-gate risk call.
        """
        line = self.TODAY_SHAPE.replace(
            '"secondary":{"used_percent":15.0,"window_minutes":10080,"resets_at":1788766582},',
            '"secondary":{"used_percent":null,"window_minutes":10080,"resets_at":1788766582},',
        )
        quota = self._sum(line)["quota_last"]
        self.assertIsNone(
            quota,
            "an unknown weekly percentage must be UNKNOWN, never the 5-hour window's",
        )
        self.assertNotEqual(quota, 96.0, "96% is the 5-HOUR window — the wrong axis")

    def test_the_weekly_window_is_the_one_nearest_seven_days_not_merely_the_longest(self):
        """`max(window_minutes)` codifies "largest window wins", so a future 30-day
        window would silently become "weekly" — the identical axis swap one rollout
        later."""
        line = self.TODAY_SHAPE.replace(
            '"plan_type":"plus"',
            '"tertiary":{"used_percent":3.0,"window_minutes":43200},"plan_type":"plus"',
        )
        self.assertEqual(
            self._sum(line)["quota_last"], 15.0, "must stay on the 7-day window"
        )

    def test_falls_back_to_primary_when_no_window_minutes_are_declared(self):
        """Older rollouts (and the synthetic fixture) carry no `window_minutes`. Absent a
        duration there is nothing to select on, so the historical behaviour stands."""
        line = (
            '{"type":"event_msg","payload":{"type":"token_count",'
            '"info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":900,'
            '"output_tokens":25,"total_tokens":1025}},'
            '"rate_limits":{"primary":{"used_percent":42.0}}}}'
        )
        self.assertEqual(self._sum(line)["quota_last"], 42.0)
