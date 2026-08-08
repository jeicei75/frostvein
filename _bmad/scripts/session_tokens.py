#!/usr/bin/env python3
"""Measure token consumption for an agent session and record it to a per-story
metrics ledger, attributed **per phase, per tool, and per model**.

Why: a story is built across separate sessions and separate *tools* — Opus (this
Claude Code session) authors and reviews; Codex/Völundr (a separate `codex exec`
run, its own transcript) does the dev. Cost has to be captured per phase and per
tool and accumulated in a durable per-story ledger that survives restarts.

Two transcript sources:
  * ``--tool claude`` — Claude Code logs each turn's ``usage`` to
    ``~/.claude/projects/<forge-slug>/<id>.jsonl``.
  * ``--tool codex``  — Codex logs cumulative ``token_count`` events to
    ``$CODEX_HOME/sessions/YYYY/MM/DD/rollout-*.jsonl`` (CODEX_HOME=/workspace/.codex).

**Delta accounting (no whole-session mis-attribution).** A single transcript holds
many phases (Opus authors, then later reviews, then patches post-review — all in
one session). Summing the whole transcript on each ``--phase`` would bill every
later phase the entire cumulative session. So recording is *delta-based*: a small
cursor file (``metrics/.session-cursors.json``) remembers the cumulative total
already recorded for each transcript, and a new row bills only what is new since
the last record. The first record on a fresh transcript bills the whole thing; a
fresh Codex rollout (one per `codex exec`) likewise bills its whole dev run.

Phase conventions (compose the columns to get the retro's required lines):
  * ``--tool codex  --phase dev``          → the `codex-dev` line (real Codex cost)
  * ``--tool claude --phase create``       → story authoring
  * ``--tool claude --phase review``       → the review gate itself
  * ``--tool claude --phase review-patch`` → the `opus-review-patch` line: rework
    the review required *after* Codex finished — the quality signal for the loop.

Usage:
    python3 session_tokens.py                                        # print this Claude session's cost
    python3 session_tokens.py --tool codex                           # print newest Codex run's cost
    python3 session_tokens.py --story ep-03-us-02 --phase review-patch          # record (delta) row
    python3 session_tokens.py --tool codex --story ep-03-us-02 --phase dev      # record Codex dev row
"""

from __future__ import annotations

import argparse
import glob
import json
import os
from datetime import datetime, timezone


# Estimated rates in USD per million tokens. EDIT THESE to match current pricing —
# they drive the only cross-phase/cross-tool comparable number (est_usd). Keyed by a
# substring matched against the model id. cache_creation priced at the 5m-write rate;
# cache_read at the cached-input rate. ``gpt-5`` covers Codex (gpt-5.5 / gpt-5-codex),
# which exposes no separate cache-write tier, so cache_write == input there.
# Bump this whenever a rate in PRICES changes, and stamp it into every row written
# afterwards. Without it a ledger silently mixes pricing eras and stops being
# comparable to itself: every review row recorded before 2026-08-01 was computed at
# the retired Opus-4.1 $15/$75 and reads ~3x high, which was not discovered until the
# ep-11 retro re-priced the epic by hand. A row that cannot say which rate table
# produced it cannot be trusted against its neighbours.
PRICES_VERSION = "2026-08-01"

PRICES: dict[str, dict[str, float]] = {
    # model-substring: {input, cache_write, cache_read, output} $/Mtok
    #
    # ORDER MATTERS. `_rates_for` returns the FIRST key that is a substring of the
    # model name, so specific keys must precede general ones — "gpt-5" matches
    # "gpt-5.6-sol" too, and would silently bill it at a quarter of its real input
    # rate if it came first.
    # Claude Fable 5 (story-creation/orchestration model since 2026-08-01): $10 in /
    # $50 out, cache read 0.1x, cache write at the 5m 1.25x rate per file convention.
    # (Claude Code sessions actually cache at the 1h TTL = 2x write; writes are a
    # small share of these sessions, so the understatement is minor — noted, not modeled.)
    "fable": {"input": 10.0, "cache_write": 12.50, "cache_read": 1.0, "output": 50.0},
    # Opus (review model). Every CURRENT Opus — 5 / 4.8 / 4.7 / 4.6 — is $5 in / $25
    # out (cache read 0.1x, write 1.25x). This row previously carried the Opus
    # 4.1-era $15/$75, so every review figure recorded before 2026-08-01 (e.g. the
    # us-02 review's $81.60) is OVER-stated ~3x. Historical rows are NOT
    # retro-corrected — read them as old-opus-rate equivalents.
    "opus": {"input": 5.0, "cache_write": 6.25, "cache_read": 0.50, "output": 25.0},
    "sonnet": {"input": 3.0, "cache_write": 3.75, "cache_read": 0.30, "output": 15.0},
    "haiku": {"input": 1.0, "cache_write": 1.25, "cache_read": 0.10, "output": 5.0},
    # gpt-5.6 Sol (Codex dev model since 2026-08-01): $5 in / $30 out, cache read at
    # a 90% discount and cache writes at 1.25x input.
    # NOTE: the other 5.6 tiers are cheaper (Terra ~$2.50 in) and are NOT listed, so
    # they would fall through to the "gpt-5" row and be over-priced. Add them here if
    # a court is ever moved onto one.
    "gpt-5.6-sol": {"input": 5.0, "cache_write": 6.25, "cache_read": 0.50, "output": 30.0},
    # gpt-5.5 (the Codex dev model up to and including ep-11-us-03) is priced the
    # SAME as 5.6 Sol: $5 in / $0.50 cached / $30 out. It had no row, so it fell
    # through to "gpt-5" and every Codex dev figure recorded before 2026-08-01 is
    # under-stated by ~3.8x. Those historical rows are NOT retro-corrected — read
    # them as gpt-5-rate equivalents.
    "gpt-5.5": {"input": 5.0, "cache_write": 6.25, "cache_read": 0.50, "output": 30.0},
    "gpt-5": {"input": 1.25, "cache_write": 1.25, "cache_read": 0.125, "output": 10.0},
}


def _rates_for(models: list[str]) -> dict[str, float] | None:
    """First *priced* model wins. Skips Claude Code's ``<synthetic>`` pseudo-model
    (no price key) so a session that briefly used it still prices at the real model."""
    for m in models:
        for key, rates in PRICES.items():
            if key in m:
                return rates
    return None


def estimate_usd(s: dict) -> float | None:
    rates = _rates_for(s["models"])
    if rates is None:
        return None
    return (
        s["input"] / 1e6 * rates["input"]
        + s["cache_creation"] / 1e6 * rates["cache_write"]
        + s["cache_read"] / 1e6 * rates["cache_read"]
        + s["output"] / 1e6 * rates["output"]
    )


def _forge_root() -> str:
    """The Nidavellir forge root — anchors transcript + ledger lookups regardless of cwd.

    Stories are implemented inside the per-court sub-repos under ``projects/`` (a
    different cwd each run), but the Claude Code session is rooted at the forge root
    and the metrics ledger lives there too. Derive the root from this script's own
    location (``<root>/_bmad/scripts/session_tokens.py``) so the result never depends
    on where the script was invoked from.
    """
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def _claude_project_dir() -> str:
    """Transcript dir for the forge session (``~/.claude/projects/<forge-root-slug>``).

    Claude Code logs to a slug derived from its launch directory (the forge root),
    NOT the shell cwd — so anchor to the forge root, never ``os.getcwd()`` (which is
    a ``projects/<sub-repo>`` dir during story work and would miss the transcript).
    """
    slug = _forge_root().replace("/", "-")
    return os.path.expanduser(f"~/.claude/projects/{slug}")


def _codex_sessions_dir() -> str:
    """Codex rollout root. Codex's auth/config live in the workspace-local CODEX_HOME
    (``/workspace/.codex``, per ``scripts/codex-handoff.sh``), not the default ``~/.codex``."""
    return os.path.join(os.environ.get("CODEX_HOME", "/workspace/.codex"), "sessions")


def _newest_transcript(directory: str, pattern: str = "*.jsonl") -> str | None:
    files = glob.glob(os.path.join(directory, "**", pattern), recursive=True)
    files += glob.glob(os.path.join(directory, pattern))
    if not files:
        return None
    return max(set(files), key=os.path.getmtime)


def subagent_transcripts(path: str) -> list[str]:
    """Sub-agent transcripts belonging to the Claude session at ``path``.

    Claude Code writes each sub-agent (a review layer, an Explore run) to its own
    JSONL under ``<dir>/<session-id>/subagents/agent-*.jsonl``, a sibling tree of
    the parent's ``<dir>/<session-id>.jsonl``. The parent transcript records the
    tool-use that spawned them but NOT their token usage, so a ledger row built
    from the parent alone silently omits everything the fan-out cost.

    Measured hole this closes: story 3.1 recorded review=$39.18 while its five
    review layers burned a further ~14.1M tokens and 193 turns unrecorded; story
    3.2's four layers burned ~495k tokens across ~111 tool-uses, none of it in the
    row. Three of 3.1's five layers produced NOTHING, so a real fraction of review
    spend bought coverage holes the ledger could not show.

    NOTE: ``agent-*.meta.json`` files live in the same directory and carry no
    usage; the ``.jsonl`` glob excludes them. Returns [] when the directory is
    absent, which is the normal case for a session that spawned nothing.
    """
    base, ext = os.path.splitext(path)
    if ext != ".jsonl":
        return []
    return sorted(glob.glob(os.path.join(base, "subagents", "agent-*.jsonl")))


def sum_claude_transcript(path: str, include_subagents: bool = True) -> dict[str, object]:
    """Sum per-turn usage across a Claude Code transcript JSONL (cumulative).

    Includes the session's sub-agent transcripts by default — see
    ``subagent_transcripts``. Pass ``include_subagents=False`` to measure one file
    in isolation.
    """
    files = [path] + (subagent_transcripts(path) if include_subagents else [])
    parts = [_sum_one_claude_file(f) for f in files]
    merged = merge_summaries(parts)
    merged["counted_transcripts"] = len(files)
    return merged


def _sum_one_claude_file(path: str) -> dict[str, object]:
    totals = {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 0}
    turns = 0
    models: set[str] = set()
    stamps: list[str] = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            msg = obj.get("message")
            if not isinstance(msg, dict):
                continue
            if msg.get("model"):
                models.add(str(msg["model"]))
            usage = msg.get("usage")
            if not isinstance(usage, dict):
                continue
            turns += 1
            if obj.get("timestamp"):
                stamps.append(str(obj["timestamp"]))
            totals["input"] += int(usage.get("input_tokens", 0) or 0)
            totals["cache_creation"] += int(usage.get("cache_creation_input_tokens", 0) or 0)
            totals["cache_read"] += int(usage.get("cache_read_input_tokens", 0) or 0)
            totals["output"] += int(usage.get("output_tokens", 0) or 0)
    return _as_summary(turns, sorted(models), totals, _span(stamps))


def sum_codex_transcript(path: str) -> dict[str, object]:
    """Sum a Codex ``rollout-*.jsonl`` (cumulative). Codex emits ``token_count``
    events whose ``info.total_token_usage`` is *already cumulative*, so the session
    total is the LAST such event — not a sum over events. ``input_tokens`` there is
    the full prompt count *including* cached, so fresh = input - cached.

    Also captures ``payload.rate_limits.primary.used_percent`` — Codex bills a weekly
    QUOTA, not metered tokens, so this is the axis that actually binds. See the ledger
    header for why it is not interchangeable with ``est_usd``.

    NOTE: ``rate_limits`` is a SIBLING of ``info`` under ``payload``, not a member of it
    (verified against a real codex-cli 0.146.0 rollout). Reading it from ``info`` yields
    silence, not an error — and a synthetic fixture that puts it there will happily pass
    while the real thing reports nothing."""
    last: dict | None = None
    events = 0
    models: set[str] = set()
    stamps: list[str] = []
    quota: list[float] = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            payload = obj.get("payload")
            if not isinstance(payload, dict):
                continue
            if payload.get("model"):  # session_meta / turn_context carry the model id
                models.add(str(payload["model"]))
            if payload.get("type") == "token_count":
                pct = ((payload.get("rate_limits") or {}).get("primary") or {}).get("used_percent")
                if isinstance(pct, (int, float)):
                    quota.append(float(pct))
                info = payload.get("info")
                if not isinstance(info, dict):
                    continue
                if isinstance(info.get("total_token_usage"), dict):
                    last = info["total_token_usage"]
                    events += 1
                    if obj.get("timestamp"):
                        stamps.append(str(obj["timestamp"]))
    if last is None:
        return _as_summary(0, sorted(models), {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 0})
    cached = int(last.get("cached_input_tokens", 0) or 0)
    totals = {
        "input": int(last.get("input_tokens", 0) or 0) - cached,  # fresh (non-cached) prompt
        "cache_creation": 0,  # Codex exposes no separate cache-write tier
        "cache_read": cached,
        "output": int(last.get("output_tokens", 0) or 0),  # already includes reasoning_output_tokens
    }
    return _as_summary(
        events, sorted(models), totals, _span(stamps),
        (quota[0], quota[-1]) if quota else (None, None),
    )


def merge_summaries(parts: list[dict[str, object]]) -> dict[str, object]:
    """Fold several transcript summaries into one.

    Turns and token buckets add. The span becomes the outer envelope, so
    ``minutes`` still reads as elapsed wall-clock over everything counted.

    QUOTA IS NOT SUMMED. ``used_percent`` is a reading of one account-wide counter,
    not a per-session quantity, so adding two readings would double-count. The
    merged quota is the envelope — the lowest first reading to the highest last —
    which is the true movement of the counter across the whole window.
    """
    parts = [p for p in parts if p]
    if not parts:
        return _as_summary(0, [], {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 0})
    totals = {b: sum(int(p[b]) for p in parts) for b in ("input", "cache_creation", "cache_read", "output")}
    models = sorted({m for p in parts for m in p["models"]})
    firsts = sorted(str(p["first_ts"]) for p in parts if p.get("first_ts"))
    lasts = sorted(str(p["last_ts"]) for p in parts if p.get("last_ts"))
    q_first = [float(p["quota_first"]) for p in parts if p.get("quota_first") is not None]
    q_last = [float(p["quota_last"]) for p in parts if p.get("quota_last") is not None]
    return _as_summary(
        sum(int(p["turns"]) for p in parts),
        models,
        totals,
        (firsts[0] if firsts else None, lasts[-1] if lasts else None),
        (min(q_first) if q_first else None, max(q_last) if q_last else None),
    )


def nested_codex_rollouts(primary: str) -> list[str]:
    """Rollouts that ran *inside* ``primary``'s window from the same directory.

    Codex's mandated ``codex review --base main`` pre-handback self-gate does NOT
    log into the dev rollout — each cycle spawns its own sibling rollout under the
    sessions tree, so a ledger row built from the dev rollout alone omits it. On
    story 3.2 that was **six cycles, $18.28 / 218 turns / 20.1M tokens**, and it
    understated the story's dev cost by 23% of dollars — and by far more on quota,
    the axis that actually binds.

    Attribution rule, deliberately narrow: same ``cwd``, different ``session_id``,
    and a time span that OVERLAPS the primary's. The ``cwd`` test is what keeps a
    concurrent run in another project out of this story's row — the exact
    contamination that cost story 3.2's quota figure a ~9pp caveat. There is no
    parent/child link in a rollout to use instead; this was checked against a real
    codex-cli 0.146.0 ``session_meta``, which carries ``cwd``, ``session_id`` and
    ``originator`` but nothing naming a parent.

    Companion 0-turn app-server rollouts are written alongside each self-gate pair;
    they contribute nothing and fall out harmlessly.
    """
    meta = _rollout_meta(primary)
    if not meta or not meta.get("cwd") or not meta.get("first_ts"):
        return []
    found = []
    for path in glob.glob(os.path.join(_codex_sessions_dir(), "**", "rollout-*.jsonl"), recursive=True):
        if os.path.abspath(path) == os.path.abspath(primary):
            continue
        other = _rollout_meta(path)
        if not other or other.get("cwd") != meta["cwd"] or not other.get("first_ts"):
            continue
        if other["session_id"] == meta["session_id"]:
            continue
        # Overlap, not containment: a self-gate started inside the dev window may
        # finish after it.
        if other["first_ts"] <= meta["last_ts"] and other["last_ts"] >= meta["first_ts"]:
            found.append(path)
    return sorted(found)


def _rollout_meta(path: str) -> dict | None:
    """(session_id, cwd, first_ts, last_ts) for a Codex rollout, or None."""
    session_id = cwd = None
    first_ts = last_ts = None
    try:
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                except json.JSONDecodeError:
                    continue
                payload = obj.get("payload")
                if isinstance(payload, dict) and obj.get("type") == "session_meta":
                    session_id = payload.get("session_id") or payload.get("id")
                    cwd = payload.get("cwd")
                ts = obj.get("timestamp")
                if ts:
                    first_ts = first_ts or str(ts)
                    last_ts = str(ts)
    except OSError:
        return None
    if not session_id:
        return None
    return {"session_id": session_id, "cwd": cwd, "first_ts": first_ts, "last_ts": last_ts}


def sum_codex_session(path: str, include_nested: bool = True) -> dict[str, object]:
    """A Codex rollout plus the sibling rollouts nested in its window."""
    files = [path] + (nested_codex_rollouts(path) if include_nested else [])
    merged = merge_summaries([sum_codex_transcript(f) for f in files])
    merged["counted_transcripts"] = len(files)
    return merged


def _span(stamps: list[str]) -> tuple[str | None, str | None]:
    """(first, last) ISO timestamp of the rows actually counted, or (None, None)."""
    return (stamps[0], stamps[-1]) if stamps else (None, None)


def _parse_ts(ts: str | None) -> datetime | None:
    if not ts:
        return None
    try:
        return datetime.fromisoformat(str(ts).replace("Z", "+00:00"))
    except ValueError:
        return None


def _minutes_between(start: str | None, end: str | None) -> float | None:
    a, b = _parse_ts(start), _parse_ts(end)
    if a is None or b is None or b < a:
        return None
    return (b - a).total_seconds() / 60.0


def _as_summary(
    turns: int,
    models: list[str],
    totals: dict[str, int],
    span: tuple[str | None, str | None] = (None, None),
    quota: tuple[float | None, float | None] = (None, None),
) -> dict[str, object]:
    grand = totals["input"] + totals["cache_creation"] + totals["cache_read"] + totals["output"]
    # A weekly window RESET inside the span makes `last < first`; report `—` rather than a
    # negative or a wrapped number, because the true consumption spans two windows and this
    # tool cannot see the pre-reset ceiling.
    pp = None
    if quota[0] is not None and quota[1] is not None and quota[1] >= quota[0]:
        pp = quota[1] - quota[0]
    return {
        "turns": turns,
        "models": models,
        "input": totals["input"],
        "cache_creation": totals["cache_creation"],
        "cache_read": totals["cache_read"],
        "output": totals["output"],
        "total": grand,
        "first_ts": span[0],
        "last_ts": span[1],
        "quota_first": quota[0],
        "quota_last": quota[1],
        "quota_pp": pp,
        # How many transcript files this summary counted. 1 for a single file; more
        # once sub-agent transcripts or nested rollouts are folded in. Part of the
        # shared shape so a row is comparable across tools.
        "counted_transcripts": 1,
    }


# --- delta accounting -------------------------------------------------------

_CURSOR_BUCKETS = ("turns", "input", "cache_creation", "cache_read", "output")

# Cursors written before fan-out accounting (T1, 2026-08-08) measured the PRIMARY
# transcript only. Diffing one of those against a now-fan-out-inclusive cumulative
# would dump every sub-agent token and every nested-rollout token that transcript
# ever spent into whichever phase happens to record next — a fresh wrong number in
# place of the old missing one, which is strictly worse because it looks precise.
# `main` rebases such a cursor instead, and says so out loud.
#
# NOTE: schema 2 and this mechanism are deliberately identical to the forge's, which
# solved the Claude half of this independently and got here first. Keeping the
# semantics aligned is what makes the two implementations mergeable rather than
# merely similar.
_CURSOR_SCHEMA = 2


def _cursor_path() -> str:
    return os.path.join(
        _forge_root(), "_bmad-output", "implementation-artifacts", "metrics", ".session-cursors.json"
    )


def _load_cursors() -> dict:
    try:
        with open(_cursor_path(), encoding="utf-8") as fh:
            return json.load(fh)
    except (FileNotFoundError, json.JSONDecodeError):
        return {}


def _save_cursors(cursors: dict) -> None:
    path = _cursor_path()
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(cursors, fh, indent=2, sort_keys=True)
        fh.write("\n")


def delta_since_cursor(cumulative: dict, prev: dict | None) -> tuple[dict, bool]:
    """Bill only what is new since ``prev`` (the cumulative already recorded for this
    transcript). Returns (delta_summary, reset) — ``reset`` is True when ``prev`` looked
    stale (any bucket would go negative, e.g. a reused/rotated transcript id), in which
    case we fall back to billing the full cumulative."""
    if not prev:
        return cumulative, False
    if any(int(cumulative.get(b, 0)) < int(prev.get(b, 0)) for b in _CURSOR_BUCKETS):
        return cumulative, True
    totals = {b: int(cumulative[b]) - int(prev.get(b, 0)) for b in ("input", "cache_creation", "cache_read", "output")}
    # The window starts where the last record stopped, so elapsed time is billed per phase
    # the same way tokens are. A cursor written before `last_ts` existed leaves it None,
    # which surfaces as `—` rather than a wrong duration.
    # Quota bills over the same window as tokens: from where the last record stopped to
    # here. A cursor written before quota existed leaves it None, so we fall back to this
    # transcript's own first sample rather than inventing a floor.
    q_from = prev.get("quota_last")
    if q_from is None:
        q_from = cumulative.get("quota_first")
    delta = _as_summary(
        int(cumulative["turns"]) - int(prev.get("turns", 0)),
        cumulative["models"],
        totals,
        (prev.get("last_ts"), cumulative.get("last_ts")),
        (q_from, cumulative.get("quota_last")),
    )
    return delta, False


# --- ledger -----------------------------------------------------------------

_LEDGER_HEADER = (
    "# Token metrics — {story}\n\n"
    "Per-phase, per-tool, per-model cost for this story. Rows are **deltas** — each "
    "records only the tokens spent since the prior record on the same transcript (so a "
    "multi-phase session is not billed whole to every phase). `total` = all tokens "
    "processed (input + cache_create + cache_read + output); it is dominated by cheap "
    "cache reads, so **`est_usd` is the comparable benchmark** (weights each component "
    "by its model rate — edit `PRICES` in `_bmad/scripts/session_tokens.py`). The "
    "`codex-dev` (tool=codex) vs `review-patch` (tool=claude) rows separate Codex's dev "
    "cost from the rework the review required after. `minutes` is wall-clock across the "
    "same delta window (first counted turn of the window to its last), so a phase that "
    "stalled reads differently from one that was merely expensive — it is the third axis, "
    "alongside tokens and cost, and it INCLUDES any human gap inside the window. It is "
    "a late column so rows written before it existed still parse; those show `—`. "
    "Generated by that script.\n\n"
    "**`quota_pp` is the axis that actually binds for Codex, and `est_usd` is NOT a "
    "substitute for it.** Codex runs on a subscription with a weekly quota, so no dollars "
    "are literally spent on a `tool=codex` row — `est_usd` weights tokens by `PRICES` "
    "purely as a cross-tool comparability benchmark. `quota_pp` is percentage points of "
    "the 7-day window consumed over the same delta window, read from "
    "`rate_limits.primary.used_percent` in the rollout. Two caveats that decide whether a "
    "number is trustworthy: the percentage is **account-wide**, so a concurrent run in "
    "another project inflates it (check each rollout's `cwd` before attributing), and a "
    "weekly reset inside the window shows `—` rather than a negative. Claude rows are "
    "always `—`. Story 3.2 is the worked example: $18.28 of self-gate priced at 23% of "
    "dollars was ~1/3–1/2 of the quota.\n\n"
    "**A row COUNTS ITS FAN-OUT** (Epic 3 retrospective, action item T1). A phase that "
    "spawns work pays for it, so the row includes it: for `tool=claude`, every sub-agent "
    "transcript under `<session-id>/subagents/agent-*.jsonl`; for `tool=codex`, every "
    "sibling rollout sharing the primary's `cwd` whose window overlaps it — which is how "
    "the mandated `codex review` self-gate is caught, since each cycle spawns its own "
    "rollout rather than logging into the dev transcript. Before this, both were invisible: "
    "story 3.1 recorded review=$39.18 while five review layers burned ~14.1M tokens and "
    "193 turns unrecorded, and story 3.2's dev row understated by **$18.28 / 218 turns / "
    "20.1M tokens**. The `cwd` test is what keeps a concurrent run in another project out "
    "of this story's row. Pass `--no-nested` to measure one transcript in isolation. "
    "**Rows recorded before 2026-08-08 do NOT include fan-out and are undercounts**; they "
    "are not retro-edited, because the ledger records what a transcript spent and a "
    "hand-adjusted number would be less trustworthy than a stated caveat.\n\n"
    "| phase | tool | model | turns | input | cache_create | cache_read | output | total | est_usd | transcript | recorded | minutes | quota_pp |\n"
    "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n"
)


def _fmt(n: int) -> str:
    return f"{n:,}"


def _fmt_usd(cost: float | None) -> str:
    return f"${cost:.2f}" if cost is not None else "—"


def _fmt_pp(pp: float | None) -> str:
    return f"{pp:.0f}pp" if pp is not None else "—"


def append_ledger(metrics_file: str, story: str, phase: str, tool: str, s: dict, transcript_id: str) -> None:
    os.makedirs(os.path.dirname(metrics_file), exist_ok=True)
    if not os.path.exists(metrics_file):
        with open(metrics_file, "w", encoding="utf-8") as fh:
            fh.write(_LEDGER_HEADER.format(story=story))
    model = ", ".join(s["models"]) or "—"
    when = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    mins = _minutes_between(s.get("first_ts"), s.get("last_ts"))
    row = (
        f"| {phase} | {tool} | {model} | {s['turns']} | {_fmt(s['input'])} | "
        f"{_fmt(s['cache_creation'])} | {_fmt(s['cache_read'])} | {_fmt(s['output'])} | "
        f"{_fmt(s['total'])} | {_fmt_usd(estimate_usd(s))} | `{transcript_id}` | "
        f"{when} · rates {PRICES_VERSION} | {'—' if mins is None else f'{mins:.0f}'} | "
        f"{_fmt_pp(s.get('quota_pp'))} |\n"
    )
    with open(metrics_file, "a", encoding="utf-8") as fh:
        fh.write(row)


# --- rollup -----------------------------------------------------------------

# The phases every non-trivial story should carry; the rollup flags any that are absent
# so a gap is loud (the ep-03->04->05 slip was silent-missing review/dev rows).
_TRIAD = ("create", "dev", "review")


def parse_ledger_rows(path: str) -> list[dict]:
    """Parse the data rows of a per-story ledger markdown table into dicts.

    Skips the header and the ``|---|`` separator; tolerates the prose/header above
    the table. Numeric cells drop thousands-commas; ``est_usd`` parses ``$x.yz`` (and
    ``—`` -> None) so an unpriced row still counts toward token totals but not cost."""
    # New columns are APPENDED, never inserted: ledgers written before `minutes` carry 12
    # cells and before `quota_pp` carry 13; `zip` stops at the shorter side, so those rows
    # simply lack the key. Inserting anywhere earlier silently re-aligns every historical
    # row by one column.
    cols = (
        "phase", "tool", "model", "turns", "input", "cache_create", "cache_read",
        "output", "total", "est_usd", "transcript", "recorded", "minutes", "quota_pp",
    )
    rows: list[dict] = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line.startswith("|"):
                continue
            cells = [c.strip() for c in line.strip("|").split("|")]
            if not cells or cells[0] in ("phase", "") or set(cells[0]) <= {"-", ":"}:
                continue
            # A ledger row carries every column through `transcript`/`recorded` (12, or 13
            # once `minutes` exists). Narrower tables in the same file are PROSE — the
            # rate-correction and annotation tables 2.1 and 1-rollup carry — and parsing
            # them invented phantom phases ("row", "dev (codex, gpt-5.6-sol)") that showed
            # up as extra rollup columns. Require the full width.
            if len(cells) < 12:
                continue
            row: dict = dict(zip(cols, cells))
            for b in ("turns", "input", "cache_create", "cache_read", "output", "total"):
                raw = str(row.get(b, "0")).replace(",", "")
                row[b] = int(raw) if raw.lstrip("-").isdigit() else 0  # annotation rows use "—"
            usd = str(row.get("est_usd", "—")).lstrip("$")
            row["est_usd"] = float(usd) if usd not in ("—", "-", "") else None
            mins = str(row.get("minutes", "—")).strip()
            row["minutes"] = float(mins) if mins.replace(".", "", 1).isdigit() else None
            pp = str(row.get("quota_pp", "—")).strip().rstrip("p")
            row["quota_pp"] = float(pp) if pp.replace(".", "", 1).isdigit() else None
            rows.append(row)
    return rows


def build_rollup(metrics_dir: str, epic: str) -> dict:
    """Aggregate every ``<epic>-*.md`` ledger into per-story / per-phase totals + a gap list."""
    files = sorted(
        f for f in glob.glob(os.path.join(metrics_dir, f"{epic}-*.md")) if not f.endswith("-rollup.md")
    )
    if not files:
        # Fail loudly. This used to fall through and write a rollup with zero rows whose
        # summary line read "No gaps — every story carries a priced create/dev/review",
        # i.e. a green verdict over an empty set. `--rollup 11` instead of `--rollup ep-11`
        # is an easy slip, and the reassuring output is exactly what stops you noticing.
        known = sorted({
            os.path.basename(f).split("-us-")[0]
            for f in glob.glob(os.path.join(metrics_dir, "*.md"))
            if "-us-" in os.path.basename(f)
        })
        hint = f" Known epic prefixes here: {', '.join(known)}." if known else ""
        raise SystemExit(
            f"no story ledgers match '{epic}-*' in {metrics_dir} — nothing to roll up.{hint}"
        )
    per_story: dict[str, list[dict]] = {}
    for f in files:
        story = os.path.basename(f)[: -len(".md")]
        per_story[story] = parse_ledger_rows(f)
    gaps, unrecoverable = [], []
    for story, rows in per_story.items():
        for phase in _TRIAD:
            prows = [r for r in rows if r["phase"] == phase]
            if not prows:
                gaps.append((story, phase))  # no row at all = a silent gap
            elif all(r["est_usd"] is None for r in prows):
                unrecoverable.append((story, phase))  # row present, cost annotated unrecoverable
    return {"epic": epic, "per_story": per_story, "gaps": gaps, "unrecoverable": unrecoverable}


def _sum_usd(rows: list[dict]) -> float:
    return sum(r["est_usd"] for r in rows if r["est_usd"] is not None)


def _story_label(story: str, epic: str) -> str:
    """Short per-row label for the rollup table: the story's *identifier*, not words from its slug.

    This used to be ``story.split("-", 4)[3]``, which silently assumed one project's key shape
    (``ep-NN-us-MM-slug`` → ``01``). On a project keyed ``E-S-slug`` it sliced a word out of the
    title instead: ``1-1-a-seeded-frozen-world-exists`` → ``seeded``, ``1-2-...`` → ``daemon``.
    Cosmetic, but the script is shared across projects, so the fix belongs here rather than in a
    fork (ep-11 retro A2; reported in frostvein's transfer note §C3).

    Rule: drop the leading ``<epic>-`` prefix, then keep the leading identifier tokens — numbers
    and the ``us`` marker — and stop at the first word of the human slug.
    ``ep-11-us-01-brain-design-decision-spike`` → ``us-01``;  ``1-1-a-seeded-...`` → ``1``.
    """
    rest = story[len(epic) + 1:] if story.startswith(f"{epic}-") else story
    tokens: list[str] = []
    for tok in rest.split("-"):
        if tok.isdigit() or tok == "us":
            tokens.append(tok)
        else:
            break
    return "-".join(tokens) or rest


def render_rollup(roll: dict) -> str:
    per_story: dict[str, list[dict]] = roll["per_story"]
    phases = list(_TRIAD) + sorted(
        {r["phase"] for rows in per_story.values() for r in rows} - set(_TRIAD)
    )
    when = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    out = [
        f"# Token metrics rollup — {roll['epic']}",
        "",
        f"Per-story cost by phase (`est_usd`), summed across the epic. `$x` = recorded; "
        f"`n/a` = a row exists but its cost is annotated unrecoverable in the ledger; `—` = no row "
        f"at all (a silent gap to fix). Generated by "
        f"`_bmad/scripts/session_tokens.py --rollup {roll['epic']}` on {when}; re-run to refresh.",
        "",
        "| story | " + " | ".join(phases) + " | total |",
        "|---" * (len(phases) + 2) + "|",
    ]
    phase_tot: dict[str, float] = {p: 0.0 for p in phases}
    phase_priced: dict[str, int] = {p: 0 for p in phases}
    grand = 0.0
    for story, rows in per_story.items():
        cells = []
        for p in phases:
            prows = [r for r in rows if r["phase"] == p]
            if not prows:
                cells.append("—")
            elif all(r["est_usd"] is None for r in prows):
                cells.append("n/a")  # present but unrecoverable
            else:
                usd = _sum_usd(prows)
                phase_tot[p] += usd
                phase_priced[p] += 1
                cells.append(f"${usd:.2f}")
        stot = _sum_usd(rows)
        grand += stot
        short = _story_label(story, roll["epic"])
        out.append(f"| {short} | " + " | ".join(cells) + f" | ${stot:.2f} |")
    tot_cells = [f"**${phase_tot[p]:.2f}**" if phase_priced[p] else "**n/a**" for p in phases]
    out.append("| **total** | " + " | ".join(tot_cells) + f" | **${grand:.2f}** |")
    out.append("")
    out.extend(_render_shape(per_story, roll["epic"]))
    if roll["gaps"]:
        out.append("**Silent gaps (no row at all — fix):** " + ", ".join(f"{s} `{p}`" for s, p in roll["gaps"]))
        out.append("")
        out.append(
            "Each gap is either backfillable (transcript on disk) or must be annotated "
            "unrecoverable in that story's ledger — no silent blanks."
        )
        out.append("")
    if roll.get("unrecoverable"):
        out.append(
            "**Unrecoverable (annotated, not silent):** "
            + ", ".join(f"{s} `{p}`" for s, p in roll["unrecoverable"])
        )
        out.append("")
        out.append(
            "These ran in mixed multi-story/multi-phase sessions with no per-phase cursor baseline, "
            "so the delta can't be reconstructed post-hoc. The `on_complete` review hook "
            "(`_bmad/custom/bmad-code-review.toml`) records the review delta at review-time going "
            "forward, so this class of gap does not recur."
        )
        out.append("")
    if not roll["gaps"] and not roll.get("unrecoverable"):
        out.append("**No gaps** — every story carries a priced create/dev/review.")
        out.append("")
    out.append(_SENTINEL)
    out.append("")
    return "\n".join(out)


# Everything after this line in a rollup survives regeneration. A rollup accumulates
# hand-written analysis that the generator cannot reproduce (rate-era corrections, why a
# row is annotated unrecoverable, cross-project comparisons), and `--rollup` used to
# silently overwrite all of it — which is precisely the "silently rewriting recorded
# history" failure the ledgers themselves warn against. Cost frostvein one rate-correction
# table before it was caught.
_SENTINEL = "<!-- HAND-WRITTEN BELOW — `--rollup` regenerates above this line and preserves everything after it. -->"


def _merge_preserved(path: str, generated: str) -> str:
    """Carry a previous rollup's hand-written tail across a regeneration.

    A pre-sentinel file cannot be split safely, so it is backed up rather than parsed —
    losing the analysis silently is the one outcome not on the table."""
    if not os.path.exists(path):
        return generated
    with open(path, encoding="utf-8") as fh:
        old = fh.read()
    if _SENTINEL in old:
        tail = old.split(_SENTINEL, 1)[1].lstrip("\n")
        return generated.rstrip("\n") + "\n" + ("\n" + tail if tail else "")
    backup = path + ".prev.md"
    with open(backup, "w", encoding="utf-8") as fh:
        fh.write(old)
    print(
        f"  NOTE: {os.path.basename(path)} predates the preserve marker; any hand-written "
        f"analysis in it is NOT carried over.\n"
        f"        Previous version saved to {os.path.basename(backup)} — move anything worth "
        f"keeping below the marker in the new file."
    )
    return generated


def _render_shape(per_story: dict[str, list[dict]], epic: str) -> list[str]:
    """The two axes cost alone hides: tokens (and how much of them is re-read context)
    and wall-clock. Cost answers 'how much'; these answer 'why', and they are what the
    Epic 2 retrospective could not read off the ledger."""
    out = [
        "## Spend shape — tokens and wall-clock",
        "",
        "Cost alone cannot tell a phase that thought hard from one that re-read the same "
        "context 200 times, nor an expensive phase from a stalled one. `cache-read` is "
        "context already paid for and re-sent; a high share means the levers are turn count "
        "and context scope, not model tier or rigor. `minutes` is wall-clock across the "
        "recorded windows and **includes any human gap inside them**, so read it as elapsed, "
        "not effort. `—` = rows recorded before these columns existed. `quota` is Codex "
        "weekly-window percentage points — the resource that actually rations delegated "
        "dev, which `est_usd` cannot express because Codex bills a subscription, not "
        "tokens. Summing it across stories is only meaningful inside one 7-day window; "
        "across an epic read it as relative weight, not as a percentage of anything.",
        "",
        "| story | turns | tokens | cache-read | cache-read % | output | minutes | quota |",
        "|---|---|---|---|---|---|---|---|",
    ]
    agg = {"turns": 0, "total": 0, "cache_read": 0, "output": 0}
    mins_total, any_mins = 0.0, False
    pp_total, any_pp = 0.0, False
    for story, rows in per_story.items():
        t = {k: sum(int(r.get(k, 0) or 0) for r in rows) for k in agg}
        for k in agg:
            agg[k] += t[k]
        smins = [r["minutes"] for r in rows if r.get("minutes") is not None]
        if smins:
            any_mins = True
            mins_total += sum(smins)
        spp = [r["quota_pp"] for r in rows if r.get("quota_pp") is not None]
        if spp:
            any_pp = True
            pp_total += sum(spp)
        share = f"{100 * t['cache_read'] / t['total']:.0f}%" if t["total"] else "—"
        out.append(
            f"| {_story_label(story, epic)} | {_fmt(t['turns'])} | {_fmt(t['total'])} | "
            f"{_fmt(t['cache_read'])} | {share} | {_fmt(t['output'])} | "
            f"{f'{sum(smins):.0f}' if smins else '—'} | "
            f"{f'{sum(spp):.0f}pp' if spp else '—'} |"
        )
    gshare = f"{100 * agg['cache_read'] / agg['total']:.0f}%" if agg["total"] else "—"
    out.append(
        f"| **total** | **{_fmt(agg['turns'])}** | **{_fmt(agg['total'])}** | "
        f"**{_fmt(agg['cache_read'])}** | **{gshare}** | **{_fmt(agg['output'])}** | "
        f"**{f'{mins_total:.0f}' if any_mins else '—'}** | "
        f"**{f'{pp_total:.0f}pp' if any_pp else '—'}** |"
    )
    out.append("")
    return out


def _print_summary(label: str, s: dict) -> None:
    model = ", ".join(s["models"]) or "unknown"
    print(f"{label}  ({s['turns']} turns, {model})")
    print(f"  input (fresh)   {_fmt(s['input']):>12}")
    print(f"  cache creation  {_fmt(s['cache_creation']):>12}")
    print(f"  cache read      {_fmt(s['cache_read']):>12}")
    print(f"  output          {_fmt(s['output']):>12}")
    print(f"  total processed {_fmt(s['total']):>12}")
    mins = _minutes_between(s.get("first_ts"), s.get("last_ts"))
    print(f"  wall-clock      {(f'{mins:.0f} min' if mins is not None else '—'):>12}  (elapsed, includes idle gaps)")
    if s.get("quota_pp") is not None:
        print(
            f"  weekly quota    {_fmt_pp(s['quota_pp']):>12}  "
            f"({s['quota_first']:.0f}% → {s['quota_last']:.0f}% of the 7-day window; account-wide)"
        )
    cost = estimate_usd(s)
    if cost is not None:
        print(f"  est. cost       {_fmt_usd(cost):>12}  (benchmark — verify rates in PRICES)")
    else:
        print("  est. cost                —  (no rate for this model in PRICES)")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--tool", choices=["claude", "codex"], default="claude")
    ap.add_argument("--transcript", help="explicit transcript path (default: newest for the tool)")
    ap.add_argument("--story", help="story key, e.g. ep-03-us-02-... (enables ledger recording)")
    ap.add_argument("--phase", default="", help="create | dev | review | review-patch")
    ap.add_argument(
        "--no-nested",
        action="store_true",
        help="measure ONE transcript in isolation: skip Claude sub-agent transcripts and nested "
        "Codex rollouts. Off by default — the honest number for a phase includes its fan-out.",
    )
    ap.add_argument(
        "--metrics-file",
        help="ledger path (default: <forge>/_bmad-output/implementation-artifacts/metrics/<story>.md)",
    )
    ap.add_argument(
        "--rollup",
        metavar="EPIC",
        help="build a per-story/per-phase cost rollup for EPIC (e.g. ep-05) and write <EPIC>-rollup.md",
    )
    args = ap.parse_args()

    if args.rollup:
        metrics_dir = os.path.join(_forge_root(), "_bmad-output", "implementation-artifacts", "metrics")
        roll = build_rollup(metrics_dir, args.rollup)
        out_path = os.path.join(metrics_dir, f"{args.rollup}-rollup.md")
        md = _merge_preserved(out_path, render_rollup(roll))
        with open(out_path, "w", encoding="utf-8") as fh:
            fh.write(md)
        print(md)
        print(f"  wrote {os.path.relpath(out_path, _forge_root())}")
        if roll["gaps"]:
            print(f"  {len(roll['gaps'])} gap(s) — backfill or annotate unrecoverable.")
        return 0

    if args.tool == "codex":
        path = args.transcript or _newest_transcript(_codex_sessions_dir(), "rollout-*.jsonl")
        if not path or not os.path.exists(path):
            print(f"session_tokens: no Codex rollout found in {_codex_sessions_dir()}")
            return 1
        s = sum_codex_session(path, include_nested=not args.no_nested)
        primary_only = None if args.no_nested else sum_codex_session(path, include_nested=False)
    else:
        path = args.transcript or _newest_transcript(_claude_project_dir())
        if not path or not os.path.exists(path):
            print(f"session_tokens: no transcript found in {_claude_project_dir()}")
            return 1
        s = sum_claude_transcript(path, include_subagents=not args.no_nested)
        primary_only = None if args.no_nested else sum_claude_transcript(path, include_subagents=False)

    transcript_id = os.path.basename(path)
    _print_summary(f"Session token cost  ({transcript_id}, tool={args.tool})", s)

    if not (args.story and args.phase):
        if args.story or args.phase:
            print("  (pass BOTH --story and --phase to record a ledger row)")
        return 0

    cursors = _load_cursors()
    prev = cursors.get(transcript_id)
    rebased = 0
    if prev is not None and int(prev.get("schema", 1)) < _CURSOR_SCHEMA and primary_only is not None:
        # Bill the primary-chain delta — exactly what the old meter would have billed —
        # then rebase the cursor to the full session so every LATER row is complete.
        # What is skipped is NAMED, not hidden: backfilling it is a deliberate re-price,
        # not something to smuggle into an unrelated row.
        rebased = int(s["total"]) - int(primary_only["total"])
        delta, reset = delta_since_cursor(primary_only, prev)
    else:
        delta, reset = delta_since_cursor(s, prev)
    if reset:
        print("  (cursor looked stale — billing full cumulative; cursor reset)")
    if rebased:
        print(
            f"  (pre-fan-out cursor rebased: this row bills the primary chain only; "
            f"{_fmt(rebased)} fan-out tokens already spent on this transcript are NOT "
            f"backfilled here — later rows on it are complete)"
        )
    metrics_file = args.metrics_file or os.path.join(
        _forge_root(), "_bmad-output", "implementation-artifacts", "metrics", f"{args.story}.md"
    )
    append_ledger(metrics_file, args.story, args.phase, args.tool, delta, transcript_id)
    cursors[transcript_id] = {b: int(s[b]) for b in _CURSOR_BUCKETS} | {
        "last_ts": s.get("last_ts"),
        "quota_last": s.get("quota_last"),
        "schema": _CURSOR_SCHEMA,
    }
    _save_cursors(cursors)
    print(
        f"  recorded delta → {os.path.relpath(metrics_file, _forge_root())} "
        f"(phase={args.phase}, tool={args.tool}, {delta['turns']} new turns, "
        f"{_fmt_usd(estimate_usd(delta))})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
