"""Tests for session_tokens.py — the per-phase/per-tool/per-model cost ledger.

Covers the three A1 fixes: Codex transcript parsing, delta-since-cursor isolation
(no whole-session mis-attribution), and per-model pricing. Builds synthetic
transcripts in tmp dirs — never touches real ledgers.

Run: python3 -m unittest discover -s _bmad/scripts/tests
"""

import json
import sys
import tempfile
import unittest
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


def _codex_token_count(model, inp, cached, out):
    return [
        json.dumps({"type": "turn_context", "payload": {"model": model}}),
        json.dumps(
            {
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "total_token_usage": {
                            "input_tokens": inp,
                            "cached_input_tokens": cached,
                            "output_tokens": out,
                            "total_tokens": inp + out,
                        }
                    },
                },
            }
        ),
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
            self.assertEqual(
                set(s), {"turns", "models", "input", "cache_creation", "cache_read", "output", "total"}
            )


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

    def test_render_marks_na_and_total(self):
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "ep-09-us-02-thin.md").write_text(_THIN)
            md = st.render_rollup(st.build_rollup(str(d), "ep-09"))
            self.assertIn("n/a", md)  # unrecoverable cell
            self.assertIn("Unrecoverable (annotated, not silent)", md)
            self.assertIn("**n/a**", md)  # all-unrecoverable review column total


if __name__ == "__main__":
    unittest.main()
