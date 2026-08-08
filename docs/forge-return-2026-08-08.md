# Forge return — 2026-08-08

Frostvein is **in sync at forge-process 1.1.0**. Both FILE entries pulled byte-identical, both
changed TEMPLATEs hand-merged and `ack`ed, `check` reports `in sync`.

Verified independently rather than assumed: `scripts/gate.sh` GATE GREEN, 42 tests, and a
**seven-sabotage pass** on the merged `session_tokens.py` — each killed by a distinct test,
`__pycache__` cleared between cycles.

---

## 1. The `live-gate` flag — **keep it. It is not noise here.**

You asked me to judge it and to tell you rather than edit locally. My answer is the opposite of
what I expected when I started looking.

`--rollup 3` now reports:

```
**No `live-gate` row:** 3-1-give-the-order, 3-2-the-dig, 3-3-the-haul-and-the-skeleton-walks
```

**That is correct, and it is the second of your two branches — "the gate ran and its cost was never
recorded."** Every Epic 3 story had a live run: a real `simd` + `tui` against a real daemon at 3.1
and 3.2, and Wolf watching the full haul loop for 3.3's AC17 sign-off. None of it was billed to a
row of its own. The flag is pointing at a real recording gap, not an Asgard phase leaking into a
project that has no equivalent.

**It also cannot be acted on here yet**, and I want that recorded rather than discovered later:
frostvein's live checks run *inside* the review session, so they cannot be separated into their own
row until `--mark` at phase boundaries exists (our open item T2). Until then the flag is a standing
true statement, not a false alarm — and your message already offers the annotate-and-move-on escape
for a story that genuinely had none, so it does not become a warning that always fires and gets
ignored.

**No change requested.** Frostvein has taken an action item to adopt a `live-gate` phase once T2
lands.

**One line of it is Asgard-specific, if you ever tighten the wording:** *"it is among the most
expensive rows in this epic"* is not true here — frostvein's live check is minutes inside a larger
session, not a costly phase of its own. The *recording* argument is what carries universally; the
*cost* argument is yours. Not worth a change on its own.

## 2. Your ep-06 A2 preflight rule closed our open P6

`EVERY LIVE-GATE STORY CARRIES AN EXECUTABLE PREFLIGHT BLOCK` and frostvein's open action item P6
turned out to be **the same rule in two courts**. We had written P6 independently at the Epic 3 retro
after two recipes lied to us; your version states the principle better than ours did — *prose
preconditions do not count and never did.*

Taken as the principle, **not** the mechanism: the `[preflight]` toml block, `preflight.sh`, court /
`skill_role` / Hermes toolset resolution are Asgard machinery and frostvein has none of it. Our form
is *the Verification recipe must be executed during story-creation and shown to produce non-zero
evidence before the story is saved*, with three rules — range-check the output never the exit code
(**exit 0 is not a result**); pin anything world- or time-dependent; and a recipe that cannot yet run
must state the exact command and the exact non-zero observation the dev agent owes.

Evidence behind ours, in case it is useful upstream: story 3.2's documented recipe designated at the
opening view level where the map is air, so it marked nothing and the feature read as broken; story
3.3's recipe assumed a fixed opening camera z, aimed into undug rock, captured **zero of every glyph
and exited 0** — and a review layer duly reported the feature did not work.

## 3. What I kept adapted, and why

- **`REVIEWS ARE READ-ONLY`** — adopted, with a carve-out. Your git prohibitions are kept verbatim
  and are the dangerous half. But frostvein *mandates* that every layer run the binaries, so
  building, `cargo test`, starting/killing a daemon and scratch under `/tmp` are expected and are not
  repo mutations — each layer builds into its own `CARGO_TARGET_DIR` under `/tmp`, so build output
  never touches the tree. Added one prohibition of our own: **never run `scripts/mutate.sh` while
  layers are live** — it rewrites source in place, so a layer running `cargo test` reads mutated
  source and reports fiction.
- **`LAYER TIME-BOX`** — took your rewrite; it is better organised than the version we sent. Flipped
  the coupling note: you record that build isolation is deliberately not ported because Asgard's
  layers are read-only and run no build. Here the cause is live, so the note says the two ship
  together and that your decision is not permission to drop it here.
- **`REVIEW RUNS IN A FRESH CONTEXT`** — adopted verbatim, and **deleted the duplicate** from our
  `REVIEW-COST DISCIPLINE` rules (3) and (4). The rule is now stated once, in your fact, so the two
  copies cannot drift apart the way they were about to.
- **`REVIEW FRAMING`** — kept ours. *"An observable-outcome AC stays OPEN until it has actually been
  observed"* is broader than *"if a story has a live/devpod AC"*, and breadth is the point here.
- **`LAYER TERRITORIES`** — kept, frostvein-only, exactly as you recorded it. Noted that your
  convergence data reads both ways.
- **`bmad-dev-story.toml`** — untouched and still correctly `adapted`. `install` never overwrites a
  present TEMPLATE, so the dev-metric text was never at risk.

## 4. A correction we owed our own config

While merging, `REVIEW-COST DISCIPLINE` still asserted *"~$22/story is the settled floor."* **Epic 3
falsified that** — $45.52/story over 862 turns, a 73% rise — and every ledger figure before
2026-08-08 is additionally a known undercount, because it omitted exactly the fan-out this release
now counts. The fact now says to quote no floor at all until a full epic is measured with fan-out
accounting on. Worth checking whether any forge-side text carries a similar stale floor.

## 5. Confirmed on real data, not just fixtures

Frostvein's story 3.2 dev rollout through the merged tool:

| | turns | tokens | est_usd |
|---|---|---|---|
| `--no-nested` | 715 | 96,000,871 | $60.96 |
| default | **933** | **116,108,161** | **$79.25** |

Same numbers our pre-merge implementation produced, and the same as the table built by hand months
apart. One difference worth knowing: yours reports **14** nested rollouts where we counted **6**
self-gate cycles. The token totals are identical, so the extra 8 are the 0-turn app-server companion
rollouts — correct, and harmless. Your `incl. N nested rollout(s): … (17.3% of the session)` line is
a real improvement on what we sent.
