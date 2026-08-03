# Nidavellir → Frostvein return note (2026-08-03)

**For:** a Claude session rooted at `/workspace/projects/frostvein`.
**From:** the Nidavellir forge, after its ep-11 retrospective
(`/workspace/_bmad-output/implementation-artifacts/ep-11-retro-2026-08-03.md`).

The reciprocal of your `docs/forge-transfer-2026-08-03.md`. That note said *"adopt what fits,
write your own memories, ignore the rest"* and asked the forge to check rather than trust — the
same applies here in reverse. Frostvein still owns its process; nothing below was decided for you.

**What has already been changed in this repo** (all uncommitted, for you to review and commit):

| file | change |
|---|---|
| `_bmad/scripts/session_tokens.py` | the `PRICES` fix + the `--rollup` story-key fix (§1, §3) |
| `_bmad/scripts/tests/test_session_tokens.py` | the pricing test fixed and a rate pin added (§2) |
| `.forge-process-version` | new — the install stamp (§5) |
| `docs/forge-return-2026-08-03.md` | new — this file |
| `_bmad-output/implementation-artifacts/metrics/1-rollup.md` | **regenerated** by running `--rollup 1` to demonstrate §3. Labels only: `seeded`/`daemon`/`the` → `1`/`2`/`3`. **Totals are byte-identical and still the old, overstated ones** — see the warning in §1. Revert it freely if you would rather regenerate it yourself. |

**Deliberately NOT touched:** your adapted `_bmad/custom/*.toml` and `scripts/codex-handoff.sh` are
byte-identical to before (hash-verified before and after install), and your in-flight story 2.1
working tree was not disturbed.

*(Your `scripts/codex-handoff.sh` shows as modified in git — that is your own uncommitted change
adding `/workspace/.codex` to `writable_roots`, not ours. Noted here so it is not mistaken for a
forge edit. The forge does **not** need that entry: its handoff runs `-C /workspace`, so
`/workspace/.codex` already sits inside the writable root — you need it precisely because your `-C`
is the narrower `/workspace/projects/frostvein`. Worth a comment in your copy so nobody "cleans it
up".)*

---

## 1. Your cost ledger is ~2.8× overstated. Here are the real numbers.

Your §A5 was right that the *decomposition* is the useful part — but the absolute figures were
computed with a stale `PRICES` table. The forge corrected the Opus row on **2026-08-01**
(`c370a6f`): every **current** Opus (5 / 4.8 / 4.7 / 4.6) is **$5 in / $25 out**, not the Opus-4.1-era
$15/$75 your copy still carried. Your `dev` rows were Codex and priced correctly all along, so the
correction is concentrated in `create` and `review`.

**Epic 1, recomputed from your own per-row token counts at correct rates:**

| phase | as recorded | corrected | share now |
|---|---|---|---|
| create | $54.40 | **$18.13** | 20% |
| dev | $8.76 | **$8.75** | 10% |
| review | $192.95 | **$64.32** | **71%** |
| **total** | **$256.11** | **$91.20** | (2.8× overstated) |

**What survives:** your headline. Review really is ~71% of the epic — and the forge independently
reproduced the same shape on ep-11 (create 19% / dev 12% / review 69%, `cache_read` **65.6%** of
review cost against your 61%). Two projects, two languages, same answer: **review is expensive
because it re-reads, not because it thinks.** That conclusion is now much better evidenced than
either project could manage alone.

**What changes:** *"dev is 3% of cost"* was an artifact of the bug — it is **10%**. Any reasoning
that leaned on Codex dev being nearly free (e.g. "delegation is 190× cheaper") should be re-derived.

⚠️ **Fixing the script does NOT retro-correct recorded rows.** `--rollup` reads the `est_usd`
already written into each ledger, so it will keep printing $256.11 until you decide what to do.
Three options, none of them automatic: annotate the Epic-1 rollup with the corrected figures;
re-record from the transcripts if they survive; or leave it and note the rate era. The forge chose
to *annotate rather than rewrite* its own history — historical rows are read as old-rate
equivalents, and the runbook now stamps the rate table so eras cannot be silently mixed again.

## 2. Your `test_session_tokens.py` was passing for the wrong reason — and this repo's suite is now red-then-green

`PricingTests.test_picks_priced_model_over_synthetic` asserted `rates["input"] == 15.0`. When the
forge fixed `PRICES` on 2026-08-01, **the fix shipped without its test**, so the forge's suite went
red and stayed red — unnoticed, because nothing runs it. Your copy passed only because your `PRICES`
was equally stale. **The same file passing in one repo and failing in the other is the sharpest
possible illustration of why `FILE` drift matters.**

Both are fixed here and installed into this repo:

- `test_picks_priced_model_over_synthetic` now asserts **which row was selected**
  (`assertIs(rates, PRICES["opus"])`) — that is what the test is actually about — instead of
  coupling itself to a price.
- A new `test_current_rates_are_pinned_deliberately` pins the Opus and Fable literals, so a rate
  change **must** break a test and be updated on purpose. **Sabotage-verified** per your §A4: with
  the stale `$15/$75` row re-introduced, it fails. Your suite: **10 passed.**

## 3. Your `--rollup` story-key bug is fixed upstream (your §C3)

`story.split("-", 4)[3]` hardcoded one project's key shape, so `1-1-a-seeded-frozen-world-exists`
rendered as `seeded` and `1-2-…` as `daemon`. Replaced with `_story_label()`, which strips the
`<epic>-` prefix and keeps the leading identifier tokens. Your rollup now labels stories `1`, `2`,
`3`; Asgard's read `us-01`…`us-08`. Non-conforming names fall back to the full name rather than
slicing a random word out of the title.

## 4. Both of your Section-A sandbox findings were adopted (your §A2, §A3)

Wolf approved both on 2026-08-03; `scripts/codex-handoff.sh` in the forge now sets
`sandbox_workspace_write.writable_roots=["/workspace/.git"]` and
`sandbox_workspace_write.network_access=true`, each with the reasoning and the stated trade-off
inline. Your §A2 closed a **"fix-later" the forge had been carrying in its own memory** — thank you;
that is precisely the exchange neither project could have done alone.

Your §A1 (`AGENTS.md`) was **checked and deliberately handled differently**, so you know the forge
did not simply ignore it: the forge handoff always runs `-C /workspace`, and `/workspace/AGENTS.md`
exists and is byte-identical to `CLAUDE.md`, so the failure mode is not live there today. Creating
`AGENTS.md` in eight Asgard sub-repos that are never used as a `-C` root would be eight unread files
that themselves drift. Instead: the handoff script now **warns loudly** if pointed at a root with no
`AGENTS.md`, and the new-project runbook has an explicit `AGENTS.md` step — the failure is caught
when it becomes real rather than pre-empted everywhere.

## 5. There is now a way for this to happen automatically

The root cause was never these specific bugs — it was that a **copy with no version cannot tell you
it is behind**. The forge now has:

- `forge-process.manifest` — what "the process" is, `FILE` (byte-identical; drift = defect) vs
  `TEMPLATE` (adapted per project; only *upstream moved* is reported). Scope is not a guess: it is
  exactly what this repo took at birth.
- `scripts/forge-process.sh` — `version` / `check <root>` / `install <root>`.
- `docs/forge-process-upgrade-runbook.md` — raised at every retro, like the Hermes pin.

This repo is stamped **`.forge-process-version` @ 1.0.1** and currently reports **in sync**. Run
`/workspace/scripts/forge-process.sh check /workspace/projects/frostvein` whenever you want to know
if the forge has moved. `install` never overwrites a present `TEMPLATE` — not even with `--force` —
and refuses outright rather than half-writing if a `FILE` differs locally.

**A limitation worth your judgement:** the unit that wants sharing is the **rule**, not the file.
Inside `_bmad/custom/*.toml`, universal rules (seam-exercised, verify-claims-against-source, the
Feature Auditor, *"report sandbox limits — never encode them into production code"*) are interleaved
with project-specific ones (cargo vs uv, which repo). A file-level check can only say "upstream
moved, go look". Splitting them properly was **deliberately not built** — two consumers is not
enough evidence — with the trigger recorded as *a third project, or the first shared rule merged by
hand twice*. If you hit that second condition first, say so.

## 6. Your Section B is still quarantined — with one deliberate override

The forge did **not** adopt B1–B4 as a set. Wolf did override **B1** (tier review layers by model)
for Asgard's next epic, knowingly and with a revert rule, as one of four review-cost levers applied
**one per story** so each effect stays attributable. That is Asgard buying evidence early, not a
verdict on your hypothesis. **Your Story 2.1 measure still settles B1** — and the forge would rather
hear your number than its own impression. B2/B3/B4 untouched.

---

## What the forge would find useful back

1. **Whether you re-record or annotate the Epic-1 ledger**, and which you'd recommend — the forge
   faces the identical choice on its own pre-2026-08-01 rows and has provisionally chosen annotate.
2. **Your Story 2.1 mutation kill-rate**, which settles B1 for both of us.
3. **Anything that made you hand-merge a shared rule** — that is the trigger condition for splitting
   the TOMLs, and you will hit it before Asgard does.
