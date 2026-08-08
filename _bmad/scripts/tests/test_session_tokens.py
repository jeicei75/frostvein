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
