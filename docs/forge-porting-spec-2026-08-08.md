# Porting spec — frostvein → Nidavellir, 2026-08-08

**For the forge session doing the merge.** The forge is the single point of truth; nothing here
asks you to revert forge work. Everything below is **additive** to what the forge already has.

Companion note with the evidence and rationale: `docs/forge-transfer-2026-08-08.md`.
Frostvein's side is committed on branch `epic-3-retrospective` — read the real code there rather
than from this spec, which describes intent and verification, not implementation.

> **Prerequisite:** the forge's own `_bmad/scripts/session_tokens.py`,
> `_bmad/scripts/tests/test_session_tokens.py` and `_bmad/custom/bmad-code-review.toml` were
> uncommitted working-tree changes when this was written. **Commit them first** — the last step of
> this process (`forge-process.sh install`) overwrites consumer copies, and you want a restore point
> behind everything.

---

## Part 1 — `session_tokens.py`

**Base = the forge's version. Do not take frostvein's file wholesale**; it lacks `_price_bucket`,
`_BUCKETS` and `by_model`, which are forge-only and must survive.

The two feature sets are disjoint on a shared spine. There is exactly **one** structural merge point.

### 1.0 The merge point: `_as_summary`

Union the keys. Neither side's are optional.

| Side | Keys |
|---|---|
| forge | `turns models input cache_creation cache_read output total` + **`by_model`** |
| frostvein | same 7 + **`first_ts last_ts quota_first quota_last quota_pp counted_transcripts`** |

Both suites assert the exact key set (frostvein: `ClaudeParsingTests.test_sum_and_shared_shape`), so
a missed key fails loudly rather than silently. That is deliberate — keep it.

### 1.1 Wall-clock (`minutes`)

**Port:** `_span`, `_parse_ts`, `_minutes_between`, and the `minutes` ledger column.

**Why it exists:** cost alone cannot distinguish a phase that thought hard from one that re-read the
same context 200 times, nor an expensive phase from a **stalled** one. A layer that hung for 2.5
hours at story 2.4 was nearly *free* in dollars. `minutes` is the third axis.

**Two properties that are load-bearing, not incidental:**
- It measures the *delta window* (first counted turn of the window to its last), so a phase recorded
  mid-session is not billed the whole session's elapsed time.
- It **includes human gaps** inside the window. Read it as elapsed, not effort — the ledger header
  says so, and that sentence should survive the merge.
- The column is appended **last** so pre-existing rows still parse (they read `—` rather than being
  re-aligned by a column). If the forge's ledgers have historical rows, preserve this ordering
  discipline.

### 1.2 Codex weekly quota (`quota_pp`) — **the axis `est_usd` cannot express**

**Port:** quota capture in `sum_codex_transcript`, `_fmt_pp`, the `quota_pp` ledger column, and quota
handling in `delta_since_cursor`.

**Why it exists:** Codex bills a **subscription with a weekly quota**, not metered tokens. No dollars
are literally spent on a `tool=codex` row — `est_usd` is only a cross-tool comparability benchmark.
Story 3.2 is the worked example: the self-gate was **23% of dollars** but **~1/3 to 1/2 of a full
week's quota**, and it **exhausted the quota outright**, forcing the next story onto Claude at
roughly double the cost. A decision made on the dollar column would have been made on the wrong axis.

**Three traps, each already paid for once:**
1. `rate_limits` is a **sibling of `info` under `payload`**, not a member of it. Reading it from
   `info` yields *silence, not an error* — and a fixture that nests it inside `info` passes happily
   while the real thing reports nothing. `test_real_rollout_line_is_pinned_verbatim` pins a real
   line for exactly this reason. **Port that test.**
2. `used_percent` is **account-wide**. A concurrent run in another project inflates it — check each
   rollout's `cwd` before attributing. This cost story 3.2's figure a ~9pp caveat.
3. A weekly **reset inside the window** makes `last < first`. Report `—`, never a negative or a
   wrapped number: the true consumption spans two windows and the tool cannot see the pre-reset
   ceiling.

### 1.3 Nested Codex rollouts — **the half the forge does not have**

**Port:** `_rollout_meta`, `nested_codex_rollouts`, `sum_codex_session`.

**Why it exists:** the mandated `codex review --base main` pre-handback self-gate does **not** log
into the dev rollout. Each cycle spawns its **own sibling rollout**, so a row built from the dev
rollout alone omits it entirely. On story 3.2 that was six cycles = **218 turns / 20,107,290 tokens
/ $18.28**, invisible.

This is the **same defect class** as the forge's sub-agent fix but a **different mechanism** — nested
`codex exec` sessions rather than Claude sub-agents. Fixing one does not fix the other. The forge fixed
the Claude side; this is the Codex side.

**Attribution rule — deliberately narrow, and the narrowness is the point:**
same `cwd`, different `session_id`, **time span overlaps** the primary's.
- `cwd` is what keeps a concurrent run in another project out of the row (trap 2 above).
- **Overlap, not containment** — a self-gate started inside the dev window may finish after it.
- There is **no parent/child link in a rollout to use instead.** Checked against a real codex-cli
  0.146.0 `session_meta`: it carries `cwd`, `session_id` and `originator`, and nothing naming a
  parent. Do not go looking for one.
- Companion 0-turn app-server rollouts are written beside each self-gate pair; they contribute
  nothing and fall out harmlessly.

**Verification that this is right, not merely plausible** — and this is the strongest evidence in the
whole transfer. Re-running 3.2's dev rollout:

| | turns | tokens | est_usd |
|---|---|---|---|
| primary only | 715 | 96,000,871 | $60.96 |
| with nested | **933** | **116,108,161** | **$79.25** |
| delta | +218 | +20,107,290 | +$18.29 |

The hand-built table in `_bmad-output/implementation-artifacts/metrics/3-2-the-dig.md` says
**218 / 20,107,290 / $18.28**, derived months apart by measuring six *named* rollouts one at a time.
The tool found them itself by `cwd` + window overlap. **Two independent derivations, one answer.**

### 1.4 Reconcile `subagent_transcripts` and pick ONE API shape

Both sides wrote `subagent_transcripts` independently: same glob
(`<dir>/<session-id>/subagents/agent-*.jsonl`), same answer. **Keep the forge's.** The independent
agreement is itself confirmation the layout assumption is right.

What must be **decided**, not merged by accident:

| | forge | frostvein |
|---|---|---|
| API | `sum_claude_session(path) -> (session, main_only, n_agents)` | `sum_claude_transcript(path, include_subagents=True)` |
| CLI escape | none | `--no-nested` |

**Recommendation: keep the forge's function shape, and add `--no-nested`.** The flag is not
speculative — isolating a single transcript is how 3.2's caveat tables were built, and it is the only
way to answer "what did *this one* rollout cost?" once nesting is on by default. It maps cleanly onto
the forge's shape: `--no-nested` simply reports `main_only`.

Whichever shape you pick, **nesting must be ON by default.** The honest number for a phase includes
the work it spawned; the isolated number is the special case.

### 1.5 Cursor rebase — **already aligned, do not re-derive**

Frostvein **adopted the forge's** `_CURSOR_SCHEMA = 2` and its rebase semantics verbatim, after the
forge caught a trap frostvein's first implementation had missed: a pre-fix cursor measured the
primary chain only, so diffing it against a fan-out-inclusive cumulative dumps every historical
sub-agent token into whichever phase records next — *a confident-looking overcount replacing a known
undercount, which is worse.* Frostvein had **42 such cursors**.

**One extension to carry back:** frostvein applies the same rebase to the **Codex** side too, since a
pre-fix codex cursor is primary-rollout-only for the same reason. Generalise the forge's `main_only`
to `primary_only` across both tools.

Keep the printed notice. What is skipped must be **named, not hidden** — backfilling belongs to a
deliberate re-price, not to an unrelated row.

### 1.6 Rollup preservation

**Port:** `_SENTINEL`, `_merge_preserved`, `_render_shape`, and the `Spend shape` table.

**Why it exists:** `--rollup` regenerates a file that also carries **hand-written retrospective
analysis** below a sentinel marker. Without preservation, re-running the rollup destroys it. There is
a test that a file predating the marker is **backed up and announced** rather than silently rewritten.

`_render_shape` produces the turns / tokens / cache-read% / output / minutes table. Its finding, in
both epics measured: **96–98% of every token processed is a cache read** — which is what proves the
cost levers are turn count and context scope, not model tier or rigor. That conclusion is unavailable
without this table.

### 1.7 Tests

Merge both suites: forge 20 + frostvein 36. They overlap little.

Frostvein classes worth taking whole: `QuotaTests` (including the verbatim real-rollout line),
`DurationTests`, `PreservationTests`, `LedgerWidthGuardTests` (legacy 12- and 13-cell rows must still
parse without shifting), `FanOutAccountingTests`, `CursorRebaseTests`.

**`test_current_rates_are_pinned_deliberately` must survive in some form.** Prices are a *decision*:
changing one must break a test so it is updated on purpose. Every review row recorded before
2026-08-01 is ~3× overstated precisely because a rate changed with nothing watching. Assert
hand-written literals — **never assert `PRICES` against itself.**

`PRICES` rows themselves are **identical** on both sides. Verified: the model-row diff is empty.

---

## Part 2 — `bmad-code-review.toml` (TEMPLATE, hand-merge)

The forge has `REVIEW-COST DISCIPLINE`. It has **no time-box, no build isolation, no territories** —
all three are net-new upstream. `check` can only ever say *upstream moved, go look*, because this is
a TEMPLATE.

### 2.1 Ship these two together, never separately

1. **`LAYER TIME-BOX` — kill on SILENCE, not elapsed time.** 8 minutes with no new *named method
   step* and no new finding; 45 min absolute ceiling; 90 min whole review.
2. **`BUILD ISOLATION` — per-layer `CARGO_TARGET_DIR`** under `/tmp/review-<layer>/target`.

**Why never separately:** the failure was never "an agent hung". Four concurrent layers, each
*mandated* to execute the binaries, starve on one shared build lock — and **a lock-blocked layer
emits nothing, which is indistinguishable from a hang.** A better detector on top of live contention
just waits longer before killing the same starved layer. Rule 2 removes the cause; rule 1 stops the
wrong kills.

Also **delete any advisory telling layers that build contention is normal and not a defect** — it
trains them to sit through the exact starvation the detector then misreads.

**Two sub-rules that are easy to drop and were expensive to learn:**
- **Ask before you kill.** Message the layer *"your box is up, start nothing new, report what you
  have NOW"*, and kill only if it stays silent. A bare kill returns **nothing**, which is the
  opposite of the rule's own stated purpose — an unattended review must degrade into a *partial
  report*, never an empty one. At 3.3 this salvaged the layer that produced the review's best findings.
- **A growing transcript is not progress.** A hung layer keeps emitting tokens and its log keeps
  growing, so file size, token count and "still running" all look identical to healthy work. The only
  signals that count are a named step completed or a finding emitted.

**This generalises well beyond code review** — it is about any fan-out orchestration where a
supervisor must distinguish a blocked worker from a dead one, and where silence gets read as a clean
result.

### 2.2 Optional, and honestly weaker: `LAYER TERRITORIES`

Disjoint hunter territories (adversarial hunter = core library, edge-case hunter = the shells; both
auditors keep whole-diff scope). **Take this only if the forge has a measured convergence problem of
its own.** Frostvein's evidence thinned to **1 convergence in 8 findings** on the only story with a
clean four-layer run, and it was approved anyway as a cost lever, sequenced strictly *after* the two
reliability fixes. Carry its revert rule if you take it: revert if a defect is later found whose site
sat inside an excluded territory.

### 2.3 Cheap and general

- **Fresh context as a precondition, not advice.** Review must not run in the session that did the
  dev. Measured: 3.3's review re-read **493k context per turn** inheriting the dev session, against
  3.2's **213k** in its own — **2.3×**, paid every turn, for a transcript it had no use for. The rule
  already existed and was simply not followed, which is why it is now phrased as a precondition.
- **One verification pass per review, not one per patch.** Re-gate turns are the highest
  cost-per-turn work in a review. Stated risk: a later patch can break an earlier verified one, and
  the final clean-build gate plus full mutation run are what catch it.

---

## Part 3 — findings that are neither code nor config

These need no merge; they are worth a forge-side retro note.

1. **Count and severity point in opposite directions.** Epic 3's per-layer yield: one auditor
   produced ~14 findings and **zero HIGHs**; both HIGHs came from the two layers that kept dying
   (1 and 6 findings respectively). **Pruning a fan-out by output volume cuts exactly the wrong
   workers.** Any forge project tempted to trim agents by yield should read this first.
2. **"Encoded" and "correct" are different claims.** The time-box was encoded, was wrong, and its
   tracking row read `done` for an entire epic while it cost coverage on two of three stories. A rule
   that has misfired in production gets re-verified against the failure, not marked closed against
   its own text.
3. **Exit 0 is not a result.** A scripted capture must be reproducible and must range-check its own
   output. 3.3's recipe aimed at a world-dependent camera level, captured **zero of everything**, and
   exited 0 — indistinguishable from "the feature is broken". An instrument that aims somewhere
   world- or time-dependent manufactures the same false evidence as a broken one.
4. **The `.pyc` trap, found while verifying this work.** A same-byte-size sabotage restored within
   the same second reuses Python's cached bytecode: the source says one thing and the import says
   another, so a sabotage reads as "survived" when the restore simply never took effect.
   `rm -rf __pycache__` between mutation cycles or the evidence is fiction.
5. **The runbook's known limitation has fired.** The unit that wants sharing is the **rule**, not the
   **file** — the time-box is universal (fan-out orchestration, nothing to do with Rust) and this is
   the **second** hand-merge of a shared rule, which is the runbook's own stated trigger to revisit.

---

## Part 4 — sequence and verification

```bash
# 0. restore point first — the forge's own changes were uncommitted
cd /workspace && git add _bmad/ && git commit          # forge's finished work

# 1. merge Part 1 into the forge's session_tokens.py + tests
# 2. merge Part 2 into the forge's bmad-code-review.toml

# 3. verify in the forge
cd /workspace && python3 -m unittest discover -s _bmad/scripts/tests   # union of both suites

# 4. version + propagate
#    VERSION=1.0.2 -> 1.1.0 in /workspace/forge-process.manifest  (new capability, not a fix)
cd /workspace && bash scripts/forge-process.sh check projects/frostvein   # both FILE entries -> ok
cd /workspace && bash scripts/forge-process.sh install projects/<name>    # every consumer
#    add a History line to /workspace/docs/forge-process-upgrade-runbook.md
```

**Sabotage-verify before believing any of it.** The metrics suite is wired into frostvein's
`scripts/gate.sh` precisely because it once went red after a `PRICES` fix and **stayed red,
unnoticed, because nothing ran it**. A green check is not evidence; break each ported behaviour on
purpose and confirm the suite goes red.

Frostvein verified this merge's own pieces that way — six sabotages on the fan-out accounting, four
on the camera fix, each killed by a distinct test.

**Also still pending, pre-existing and unrelated to this transfer:** `check` reports `adapted` on all
four TEMPLATE entries. `bmad-create-story.toml` and `bmad-dev-story.toml` shape how frostvein's Epic 4
stories get written, so those want reconciling **before** story 4.1a.
