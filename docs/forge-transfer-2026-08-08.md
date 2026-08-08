# Forge transfer — 2026-08-08 (Epic 3 retrospective)

Read-only note for Nidavellir, following the 2026-08-03 / 2026-08-05 convention: split by
evidence class, every claim naming the file or command that verifies it.

> **⚠️ THIS IS A TWO-WAY MERGE, NOT A PUSH — and the forge was LIVE when this was written.**
>
> `./scripts/forge-process.sh check projects/frostvein` reports **`UPSTREAM CHANGED`** on both
> shared script files. Both sides moved: the forge independently implemented the *same* Claude
> sub-agent fix, down to the function name. **A file copy in either direction destroys real work.**
>
> `/workspace/_bmad/scripts/session_tokens.py` was `md5 084310cc8e00`, mtime **10:02:02**, when this
> note was written at **10:10**. The forge session was actively editing it. **Re-diff before acting;
> the tables below are a snapshot, not a current state.**

---

## 1. What frostvein changed (all verifiable, all gate-green)

Epic 3 closed **MILESTONE 1** (FR26 walking-skeleton gate). These are the process/tooling changes
that came out of its retrospective, `_bmad-output/implementation-artifacts/epic-3-retro-2026-08-08.md`.

### 1a. The layer time-box was WRONG and is rewritten — **do not ship the 2026-08-05 version**

The rule transferred on 2026-08-05 kills a review layer on **20 minutes of wall-clock since launch**.
Epic 3 proved that cannot work: the layers run concurrently, each is *mandated* to run `cargo test`,
and they starve on one shared `target/` lock — **a lock-blocked layer emits nothing, which is
indistinguishable from a hang.**

Evidence, not inference:

| Story | Layers completing | Consequence |
|---|---|---|
| 3.1 | 1 of 4 | three killed with **zero findings between them**; one was literally reporting the cause (`sequential due to target lock`). The epic's worst defect — a designate taking a delta from 378 bytes to **16,761,209 bytes at 34.7 MB/s**, world simultaneously unsaveable — was found by the orchestrator **inline**, after all three had failed. |
| 3.2 | 4 of 4 | no holes — and the story where review found the most. |
| 3.3 | 1 of 4, then 3 of 4 | Edge Case Hunter died silent in **both** rounds; that story's loader/boundary territory is **unreviewed by any layer**. |

**The replacement (two rules, ship them together):**

1. **Kill on SILENCE, not elapsed time** — 8 minutes with no new *named method step* and no new
   finding; 45 min absolute ceiling; 90 min whole-review. Before killing, message the layer
   *"your box is up, report what you have NOW"* — a bare kill returns nothing, which is the opposite
   of the rule's own purpose. At 3.3 that salvage step recovered the Feature Auditor on the re-run,
   and it produced that review's best findings.
2. **Per-layer `CARGO_TARGET_DIR`** (`/tmp/review-<layer>/target`) — removes the contention instead
   of detecting it better. **Never ship rule 1 without rule 2**: a better detector on top of real
   contention just waits longer before killing the same starved layer.

Also delete the advisory that told each layer target-lock contention was normal and not a defect —
it was training them to sit through the exact starvation the detector then misread as a hang.

Verify: `rg 'THE MEASURE IS SILENCE' _bmad/custom/bmad-code-review.toml`.
`docs/forge-transfer-2026-08-05.md` now carries a **SUPERSEDED** banner over the old prescription so
it cannot be shipped by accident. **The diagnosis in that note is still correct; only the
prescription changed.**

**Why the forge should care beyond code review** (unchanged from 2026-08-05, and now proven twice):
this is about *any* fan-out orchestration. The failure is not "an agent hung" — it is "the
supervisor could not tell a blocked worker from a dead one, and silence was read as a clean result."

### 1b. Fan-out accounting — **the forge got the Claude half first; frostvein has the Codex half**

`session_tokens.py` counted only the primary transcript, so everything a phase spawned was invisible.
Frostvein measured the hole on both sides:

- **Review side** (what the forge already fixed): story 3.1 recorded `review=$39.18` while five
  review layers burned ~14.1M tokens and 193 turns unrecorded.
- **Dev side** (frostvein-only, and a *different mechanism*): the mandated `codex review --base main`
  self-gate does **not** log into the dev rollout — each cycle spawns its own sibling rollout. On
  story 3.2 that was **six cycles = 218 turns / 20,107,290 tokens / $18.28**, invisible.

**Independent-oracle verification, which is the part worth transferring.** Re-running 3.2's dev
rollout through the fixed tool:

| | turns | tokens | est_usd |
|---|---|---|---|
| as recorded | 715 | 96,000,871 | $60.96 |
| with nested rollouts | **933** | **116,108,161** | **$79.25** |
| delta | +218 | +20,107,290 | +$18.29 |

That reproduces a **hand-built** table (218 / 20,107,290 / $18.28) **to the token, by a different
code path** — the hand table named six rollouts; the tool found them itself by `cwd` + window
overlap. Attribution rule is deliberately narrow: same `cwd`, different session id, overlapping
window. The `cwd` test is what keeps a concurrent run in another project out of the row — the exact
contamination that cost 3.2's quota figure a ~9pp caveat.

**Frostvein took the forge's cursor-rebase design rather than inventing its own.** The forge saw a
trap frostvein's first implementation had missed: a pre-fix cursor measured the primary chain only,
so diffing it against a now-fan-out-inclusive cumulative dumps every sub-agent token that transcript
ever spent into whichever phase records next — *a confident-looking overcount replacing a known
undercount, which is worse.* Frostvein had **42 such cursors**. It now uses `_CURSOR_SCHEMA = 2` with
the forge's semantics, deliberately identical so the implementations stay mergeable.

### 1c. Reproducible captures — "exit 0 is not a result"

Story 3.3's recorded live recipe was **not reproducible and failed silently**: its leading `<`
assumed a fixed opening camera z, but the camera followed a wandering dwarf, so the auditor's run
aimed into undug rock and captured **zero of every glyph with exit 0** — indistinguishable from
"the feature is broken". It cost a false *"the feature does not work"* verdict.

Generalisable rule, now in `docs/technical-preferences.md` beside the observability-instrument rule:
**a scripted capture must be reproducible and must range-check its own output before any conclusion
is drawn.** An instrument that aims somewhere world- or time-dependent manufactures the same false
evidence as a broken one.

### 1d. Review-cost rules the forge may want

- **Fresh context as a precondition, not advice.** Story 3.3's review ran inside 3.3's dev session
  and re-read **493k context per turn** against 3.2's **213k** in its own session — **2.3×**, paid
  every turn, for carrying a dev transcript the review had no use for. The rule already existed and
  was simply not followed; it is now phrased as a precondition (*if this session did the dev, stop
  and start a new one*).
- **One verification pass per review, not one per patch.** Re-gate turns are the highest
  cost-per-turn work in a review. Accepted risk stated in the rule: a later patch can break an
  earlier verified one, and the final clean-build gate + full mutation run are what catch it.

### 1e. A finding about layer value that contradicts the obvious read

Epic 3's per-layer yield, made measurable by the *forge's own* "record which layer raised each
finding" rule (its ep-11 retro A3, taken by frostvein 2026-08-06):

| Layer | Findings | HIGH | Completed |
|---|---|---|---|
| Acceptance Auditor (Opus) | ~14 | **0** | 3 of 3 |
| Feature Auditor (Opus) | 6 (+1 noise) | **1** | 1 of 4 |
| Blind Hunter (Sonnet) | 1 | **1** | 1 of 4 |
| Edge Case Hunter (Sonnet) | 2 | 0 | 1 of 4 |

**Count and severity point in opposite directions.** Fourteen findings and zero HIGHs from one
layer; both HIGHs from the two layers that kept dying. **Trimming layers by yield-count would cut
exactly the wrong ones.** Any forge project tempted to prune a fan-out by output volume should read
this first.

---

## 2. `session_tokens.py` — reconcile, do NOT ack the file

Snapshot at 10:10 on 2026-08-08. **Re-diff before acting.**

| | Functions | Verdict |
|---|---|---|
| **Frostvein-only** | `nested_codex_rollouts`, `_rollout_meta`, `sum_codex_session`, `merge_summaries`, `_sum_one_claude_file`, `_span`, `_parse_ts`, `_minutes_between`, `_fmt_pp`, `_merge_preserved`, `_render_shape` | Codex nested rollouts, `minutes`, `quota_pp`, and rollup hand-written-section preservation. **Forge lacks all of it.** |
| **Forge-only** | `_merge_summaries`, `_price_bucket`, `sum_claude_session` | `_BUCKETS`/`_price_bucket` refactor and the `(session, main_only, n_agents)` shape. **Frostvein lacks it.** |
| **Both, written independently** | `subagent_transcripts`, `sum_claude_transcript` | ⚠️ **THE MERGE HAZARD.** Same name, same glob, same answer — arrived at twice. A copy silently picks one API shape and drops the other's callers. |
| `PRICES` | — | ✅ **No divergence.** Verified: the model-row diff is empty. |

**The API shapes differ and that is the whole difficulty:** the forge kept `sum_claude_transcript`
single-file and added `sum_claude_session` returning a 3-tuple; frostvein gave
`sum_claude_transcript` an `include_subagents=` flag and a `--no-nested` CLI escape. Pick one shape
deliberately — frostvein's `--no-nested` exists because measuring a single transcript in isolation
was a real, recurring need (it is how 3.2's caveat tables were built).

**This is the runbook's known limitation firing for the second time**: the unit that wants sharing is
the **rule**, not the **file**. The time-box is universal (fan-out orchestration, nothing to do with
Rust). That is **merge 2 of 2** against the runbook's own trigger — *"revisit at a third project, or
the first time a shared rule is hand-merged twice."* **The trigger has now fired.**

---

## 3. Sequence (unchanged, still verified — but coordinate first)

0. **Quiesce or coordinate with the running forge session** before touching `_bmad/scripts/`. It
   edited `session_tokens.py` at 10:02 today.
1. **Reconcile** `session_tokens.py` + `tests/test_session_tokens.py` as a merge, per §2.
   Do not `cp` in either direction.
2. Bump `VERSION=1.0.2` → `1.1.0` in `/workspace/forge-process.manifest` (new capability, not a fix).
3. Re-run `./scripts/forge-process.sh check projects/frostvein` — those two entries should go `ok`.
4. `scripts/forge-process.sh install projects/<name>` for every other consumer.
5. Add a History line to `/workspace/docs/forge-process-upgrade-runbook.md`.
6. Hand-merge the **rewritten** time-box + build isolation into each consumer's
   `bmad-code-review.toml` (a TEMPLATE, so `check` can only ever say *upstream moved, go look*).

**Also still pending, pre-existing:** `check` reports `adapted` on all four TEMPLATE entries — the
forge has improvements frostvein has never taken, and `bmad-create-story.toml` / `bmad-dev-story.toml`
shape how Epic 4's stories get written. Clear those **before** story 4.1a.

---

## 4. Verify every claim here

```bash
cd /workspace && bash scripts/forge-process.sh check projects/frostvein
cd /workspace/projects/frostvein && ./scripts/gate.sh                      # GATE GREEN, 36 metric tests
python3 -m unittest discover -s _bmad/scripts/tests                        # 36 tests
rg 'THE MEASURE IS SILENCE|BUILD ISOLATION|LAYER TERRITORIES' _bmad/custom/bmad-code-review.toml
diff /workspace/_bmad/scripts/session_tokens.py _bmad/scripts/session_tokens.py
```

**Gotcha found while verifying this work, worth passing on:** a same-byte-size sabotage restored
within the same second reuses Python's cached `.pyc` — the source says one thing and the import says
another, so a sabotage looks like it "survived" when the restore simply never took effect. `rm -rf
__pycache__` between sabotage cycles, or the mutation evidence is fiction. Same false-evidence class
as everything else in this note.
