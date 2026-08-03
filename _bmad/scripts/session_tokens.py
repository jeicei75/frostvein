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


def sum_claude_transcript(path: str) -> dict[str, object]:
    """Sum per-turn usage across a Claude Code transcript JSONL (cumulative)."""
    totals = {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 0}
    turns = 0
    models: set[str] = set()
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
            totals["input"] += int(usage.get("input_tokens", 0) or 0)
            totals["cache_creation"] += int(usage.get("cache_creation_input_tokens", 0) or 0)
            totals["cache_read"] += int(usage.get("cache_read_input_tokens", 0) or 0)
            totals["output"] += int(usage.get("output_tokens", 0) or 0)
    return _as_summary(turns, sorted(models), totals)


def sum_codex_transcript(path: str) -> dict[str, object]:
    """Sum a Codex ``rollout-*.jsonl`` (cumulative). Codex emits ``token_count``
    events whose ``info.total_token_usage`` is *already cumulative*, so the session
    total is the LAST such event — not a sum over events. ``input_tokens`` there is
    the full prompt count *including* cached, so fresh = input - cached."""
    last: dict | None = None
    events = 0
    models: set[str] = set()
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
                info = payload.get("info")
                if isinstance(info, dict) and isinstance(info.get("total_token_usage"), dict):
                    last = info["total_token_usage"]
                    events += 1
    if last is None:
        return _as_summary(0, sorted(models), {"input": 0, "cache_creation": 0, "cache_read": 0, "output": 0})
    cached = int(last.get("cached_input_tokens", 0) or 0)
    totals = {
        "input": int(last.get("input_tokens", 0) or 0) - cached,  # fresh (non-cached) prompt
        "cache_creation": 0,  # Codex exposes no separate cache-write tier
        "cache_read": cached,
        "output": int(last.get("output_tokens", 0) or 0),  # already includes reasoning_output_tokens
    }
    return _as_summary(events, sorted(models), totals)


def _as_summary(turns: int, models: list[str], totals: dict[str, int]) -> dict[str, object]:
    grand = totals["input"] + totals["cache_creation"] + totals["cache_read"] + totals["output"]
    return {
        "turns": turns,
        "models": models,
        "input": totals["input"],
        "cache_creation": totals["cache_creation"],
        "cache_read": totals["cache_read"],
        "output": totals["output"],
        "total": grand,
    }


# --- delta accounting -------------------------------------------------------

_CURSOR_BUCKETS = ("turns", "input", "cache_creation", "cache_read", "output")


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
    delta = _as_summary(int(cumulative["turns"]) - int(prev.get("turns", 0)), cumulative["models"], totals)
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
    "cost from the rework the review required after. Generated by that script.\n\n"
    "| phase | tool | model | turns | input | cache_create | cache_read | output | total | est_usd | transcript | recorded |\n"
    "|---|---|---|---|---|---|---|---|---|---|---|---|\n"
)


def _fmt(n: int) -> str:
    return f"{n:,}"


def _fmt_usd(cost: float | None) -> str:
    return f"${cost:.2f}" if cost is not None else "—"


def append_ledger(metrics_file: str, story: str, phase: str, tool: str, s: dict, transcript_id: str) -> None:
    os.makedirs(os.path.dirname(metrics_file), exist_ok=True)
    if not os.path.exists(metrics_file):
        with open(metrics_file, "w", encoding="utf-8") as fh:
            fh.write(_LEDGER_HEADER.format(story=story))
    model = ", ".join(s["models"]) or "—"
    when = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    row = (
        f"| {phase} | {tool} | {model} | {s['turns']} | {_fmt(s['input'])} | "
        f"{_fmt(s['cache_creation'])} | {_fmt(s['cache_read'])} | {_fmt(s['output'])} | "
        f"{_fmt(s['total'])} | {_fmt_usd(estimate_usd(s))} | `{transcript_id}` | {when} |\n"
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
    cols = ("phase", "tool", "model", "turns", "input", "cache_create", "cache_read", "output", "total", "est_usd")
    rows: list[dict] = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line.startswith("|"):
                continue
            cells = [c.strip() for c in line.strip("|").split("|")]
            if not cells or cells[0] in ("phase", "") or set(cells[0]) <= {"-", ":"}:
                continue
            row: dict = dict(zip(cols, cells))
            for b in ("turns", "input", "cache_create", "cache_read", "output", "total"):
                raw = str(row.get(b, "0")).replace(",", "")
                row[b] = int(raw) if raw.lstrip("-").isdigit() else 0  # annotation rows use "—"
            usd = str(row.get("est_usd", "—")).lstrip("$")
            row["est_usd"] = float(usd) if usd not in ("—", "-", "") else None
            rows.append(row)
    return rows


def build_rollup(metrics_dir: str, epic: str) -> dict:
    """Aggregate every ``<epic>-*.md`` ledger into per-story / per-phase totals + a gap list."""
    files = sorted(
        f for f in glob.glob(os.path.join(metrics_dir, f"{epic}-*.md")) if not f.endswith("-rollup.md")
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
    return "\n".join(out)


def _print_summary(label: str, s: dict) -> None:
    model = ", ".join(s["models"]) or "unknown"
    print(f"{label}  ({s['turns']} turns, {model})")
    print(f"  input (fresh)   {_fmt(s['input']):>12}")
    print(f"  cache creation  {_fmt(s['cache_creation']):>12}")
    print(f"  cache read      {_fmt(s['cache_read']):>12}")
    print(f"  output          {_fmt(s['output']):>12}")
    print(f"  total processed {_fmt(s['total']):>12}")
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
        md = render_rollup(roll)
        out_path = os.path.join(metrics_dir, f"{args.rollup}-rollup.md")
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
        s = sum_codex_transcript(path)
    else:
        path = args.transcript or _newest_transcript(_claude_project_dir())
        if not path or not os.path.exists(path):
            print(f"session_tokens: no transcript found in {_claude_project_dir()}")
            return 1
        s = sum_claude_transcript(path)

    transcript_id = os.path.basename(path)
    _print_summary(f"Session token cost  ({transcript_id}, tool={args.tool})", s)

    if not (args.story and args.phase):
        if args.story or args.phase:
            print("  (pass BOTH --story and --phase to record a ledger row)")
        return 0

    cursors = _load_cursors()
    delta, reset = delta_since_cursor(s, cursors.get(transcript_id))
    if reset:
        print("  (cursor looked stale — billing full cumulative; cursor reset)")
    metrics_file = args.metrics_file or os.path.join(
        _forge_root(), "_bmad-output", "implementation-artifacts", "metrics", f"{args.story}.md"
    )
    append_ledger(metrics_file, args.story, args.phase, args.tool, delta, transcript_id)
    cursors[transcript_id] = {b: int(s[b]) for b in _CURSOR_BUCKETS}
    _save_cursors(cursors)
    print(
        f"  recorded delta → {os.path.relpath(metrics_file, _forge_root())} "
        f"(phase={args.phase}, tool={args.tool}, {delta['turns']} new turns, "
        f"{_fmt_usd(estimate_usd(delta))})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
