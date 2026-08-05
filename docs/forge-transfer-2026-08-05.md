# Frostvein → Nidavellir transfer note (2026-08-05)

**For:** a Claude session rooted at `/workspace` (the Nidavellir forge — its own BMad, its
own memory store `-workspace`, its own `scripts/codex-handoff.sh`).
**From:** frostvein's Epic 2 retrospective,
`/workspace/projects/frostvein/_bmad-output/implementation-artifacts/epic-2-retro-2026-08-05.md`.
**Predecessor:** `docs/forge-transfer-2026-08-03.md` (Epic 1). Same contract as that one.

Frostvein owns its own process and does not write into the forge's `_bmad-output/`. This
file is a **read-only handoff**: adopt what fits, write your own memories, ignore the rest.
Every claim names the file or command that verifies it — check rather than trust.

Split by evidence class. **Section A is fact and transfers today. Section B is hypothesis
and must not be adopted until frostvein Epic 3 settles it. Section C is a defect in a script
the forge and frostvein share.**

---

## A. Facts — verified, adopt now

### A1. A hung sub-agent is indistinguishable from a working one, and nothing detects it

**The incident.** Frostvein story 2.4's code review fans out four layers concurrently. One
hung for **~2.5 hours** — 43% of that story's total wall-clock — and the run sat on it until
Wolf came back from an evening out and noticed. Nothing in the orchestration had any concept
of "too long".

**The trap that makes this hard to spot, and the part worth transferring:**

> **A growing transcript is not progress.**

A hung layer keeps emitting tokens. Its log keeps growing. File size, token count, and
"still running" all look *identical* to healthy work. Every intuitive progress signal is a
false one. The only signals that count are a layer completing a **named step of its method**
or **emitting a finding**; if neither has advanced inside the budget, it is hung.

**The fix, now encoded** in `_bmad/custom/bmad-code-review.toml` as the `LAYER TIME-BOX`
persistent fact (verify: `rg 'LAYER TIME-BOX' _bmad/custom/bmad-code-review.toml`):

- **20 minutes hard budget per layer.** The orchestrator kills and continues. No extensions
  because it "seems close".
- **A timed-out layer is a coverage hole, not a clean result.** It must be reported as such
  in the summary — *"Edge Case Hunter timed out at 20 min; its territory is unreviewed"* —
  so its silence can never be read as "found nothing". This is the half most likely to be
  skipped, and it is the half that matters.
- **60-minute ceiling on the whole review.** Past that: stop, report what exists, ask. An
  unattended run must degrade into a **partial report**, never into an open-ended wait.

**Why the forge should care beyond code review:** this applies to *any* fan-out
orchestration — parallel agents, workflow stages, background tasks. The frostvein incident
happened to be a review layer; the failure mode is structural.

**The meta-lesson, which is Epic 1's lesson recurring in a new place:** the in-session fixes
applied at 2.4 (port hygiene, prompt-level time-boxing) were **never written into any config
file**. Verified at the retro by searching for them. Epic 3 would have inherited the trap
unchanged. *An ad-hoc fix that is not encoded did not happen.*

### A2. Cost alone is the wrong instrument. You need tokens and wall-clock beside it

**The finding.** Frostvein's Epic 1 retrospective adopted three cost levers and predicted
review spend would fall from $193 to $39. Measured at correct rates against correct rates,
review went from **$21.44/story to $26.30/story** — it *rose*. Nobody could see this,
because the ledger recorded only dollars.

**What the token axis then showed immediately:**

| | Epic 1 (3 stories) | Epic 2 (4 stories) |
| --- | --- | --- |
| Turns | 971 | 2,146 |
| Tokens processed | 102.8M | 290.8M |
| **Of which cache reads** | **99.1M (96%)** | **280.6M (96%)** |
| Output tokens | 973,843 | 1,738,460 |

**96% of every token processed is re-read context, in both epics.** That invariant survived
three cost levers untouched, because none of them addressed turn count or orchestrator
context — which is where the mass is.

**Be careful comparing this to ep-11's number.** The forge's ep-11 figure (65.6% cache reads)
and frostvein's Epic 1 figure (61%) are **shares of cost**; the 96% above is a **share of
tokens**. Both are correct and they are different metrics — cache reads are cheap per token,
so they dominate volume far more than they dominate the bill. State which denominator you
mean or the two projects will appear to disagree.

**Adopt:** the axes, not the numbers. Cost answers *how much*; tokens answer *why*; wall-clock
distinguishes an expensive phase from a stalled one (A1 is invisible on the cost axis — a
hung layer's 2.5 hours cost almost nothing).

### A3. Naming the trap by name, in the story, is what killed a four-story recurring class

Frostvein's dominant defect class was self-referential tests — the test running the oracle and
the implementation through the same function. It recurred in **1.1, 1.2, 1.3, 2.1, and 2.2**
despite being written down as a warning in two consecutive story files.

What finally ended it (zero instances in 2.3 and 2.4) was not "write stronger tests" and not a
process step. It was a **literal, specific sentence in the story's Dev Notes naming the exact
shape of the trap for that story**:

> *"The self-referential save test is this story's headline trap. `assert_eq!(a.to_save(),
> b.to_save())` is green for exactly the bug it should catch: a field missing from `SaveState`
> is missing from both sides. The oracle is `tick()` / `dwarves()` / `tile()` after ticking
> forward… This class has now been hit in 1.1, 1.2 and 1.3 — do not add a fourth."*

(Verify: `_bmad-output/implementation-artifacts/2-4-the-world-endures.md`, "Key decisions &
traps".) The dev agent walked around it. **Generic warnings transfer knowledge; a named,
story-specific trap changes behaviour.**

### A4. An untested evidence channel manufactures false evidence

Three consecutive frostvein stories shipped or inherited a broken observability instrument:

- **2.1**: the instrument had to be *invented at review* because the composed client loop was
  invisible to the gate.
- **2.2**: the instrument recomputed the camera every frame, so a moving entity stayed pinned
  to screen centre while the world scrolled underneath. It **rendered motion as stillness**,
  and every piece of "live evidence" taken through it was an artefact of the instrument. In
  parallel, `NO_COLOR=1` in the environment meant a colour capture containing zero colour
  information read as proof the colours worked.
- **2.4**: an AC named two instrument invocations; only one had a test.

> **An untested evidence channel manufactures false evidence rather than merely missing true
> evidence — which is strictly worse than having no instrument, because it is believed.**

The rule, now encoded in `_bmad/custom/bmad-create-story.toml` (verify:
`rg 'TESTED-INSTRUMENT' _bmad/custom/bmad-create-story.toml`): the task that names a story's
instrument must also require **a test of the instrument itself**, driving the real binary,
carrying a mutation like any other — asserting that the observable *changes* when the
underlying state changes, and that the instrument *says so* when the environment can silently
suppress its signal.

### A5. Spec-text defects are a distinct class, and nothing guards them

Acceptance criteria describing an outcome that cannot occur, or misstating an earlier story's
contract. Four instances in seven frostvein stories, every one authored in story-creation and
every one caught only at review:

- an AC promising a fallback that is unreachable on the platform;
- an AC amended at review because it described the wrong invariant;
- an AC demanding an effect appear "in the very next message" when the transport guarantees it
  lands on the second;
- an AC that both claimed behaviour X and demanded "keeps the previous story's behaviour
  exactly", where the previous story does not-X.

Cheap guard, adopted by frostvein for Epic 3: at authoring time, every AC must name an
observation that **can actually occur**, and any AC restating a prior contract must be checked
against that contract rather than remembered.

### A7. The shared script's tests had no runner — check yours

`_bmad/scripts/tests/test_session_tokens.py` is a manifest `FILE`, thorough, and **nothing ran
it**. Its docstring records that it went red after the 2026-08-01 `PRICES` fix and stayed red
unnoticed, because no gate invoked it. Frostvein has now wired it into `scripts/gate.sh` as a
fifth check — stdlib `unittest`, no pytest and no venv, so a pre-commit hook cannot break on a
missing dev dependency:

```bash
run "metrics ledger tests" python3 -m unittest discover -s _bmad/scripts/tests
```

Verify before trusting it, as frostvein did: revert `PRICES` to the retired `$15/$75` Opus row and
confirm the gate goes RED. **Worth checking whether the forge runs its own copy** — a defect in a
`FILE` propagates to every consumer by design, which is exactly what makes an untested one
expensive.

### A6. A deferred UX finding can *be* the product outcome

Frostvein's Epic 2 goal was *"the world reads as alive."* A review layer wrote, and the project
deferred:

> *"At a default 80×24 terminal only one of the five dwarves is ever visible… so the headline
> outcome reads as one dwarf twitching once a second."*

Three stories later, asked whether he signed off on the epic's headline outcome, Wolf said the
world *"didn't change after that, so it was a bit boring after all."* The deferred sentence was
the verdict.

**Transferable rule:** when a review layer says *"the headline outcome reads as X"* and X is
bad, that is a **product finding**, not a nice-to-have. Triage it against the epic's goal
sentence, not against the story's ACs — no AC was violated in the frostvein case, which is
exactly why it was deferred.

---

## B. Hypothesis — do NOT adopt until frostvein Epic 3 settles it

### B1. Review restructure: disjoint territories + one verification pass

**The diagnosis** (fact, per A2): review costs `turns × context × rate`, is 96% re-read
context, and no lever so far has touched turns or orchestrator context.

**The unadopted proposal:**

- **R1 — disjoint territories.** All four layers currently read the same diff under the same
  instruction ("find problems"), and they converge: in frostvein two separate stories had a
  single defect found independently by **three** layers. Give each hunter a slice (one layer
  gets the pure-logic core, one gets the I/O shells) while the two auditors keep whole-diff
  scope so cross-cutting defects stay covered.
- **R2 — one verification pass per review**, not one per patch. Apply all patches, then run
  the gate and mutation set once.

**Control:** mutation kill-rate against the established baseline; plus **record which layer
raised each finding**, so convergence becomes measured rather than inferred (frostvein only
saw it because three layers happened to be named in two story files — that one is cheap and
transferable on its own).

**Revert rule:** if a defect is later found whose site sat inside a hunter's *excluded*
territory, drop R1 and keep R2.

**Why it is Section B and not Section A:** frostvein's previous cost prediction was confidently
wrong in the same way this one might be (A2). It is measured on turns and cache-read tokens,
not promised in dollars, and it has not run yet.

---

## C. A defect in the script the forge and frostvein share

`session_tokens.py` exists in both trees. Two issues found while adding the token/wall-clock
axis; frostvein's copy is fixed as of 2026-08-05.

### C1. Annotation tables inside a ledger were parsed as data rows

`parse_ledger_rows` accepted **any** markdown table row in the file. Ledgers legitimately carry
prose tables — frostvein's carry rate-correction tables explaining why a historical row is
overstated — and those were parsed as metric rows, inventing phantom phases (`row`,
`dev (codex, gpt-5.6-sol)`, `review (claude, opus-5)`) that appeared as **extra columns in the
rollup**. Cost totals stayed correct (the phantom rows priced as `None`), so it was cosmetic —
but it made the rollup look wrong, which erodes the trust the ledger exists to create.

**Fix:** require full row width (`len(cells) >= 12`) before treating a row as data. A ledger row
carries every column through `transcript`/`recorded`; a prose table does not.

### C2. Adding a column to the ledger is a backward-compatibility trap

`parse_ledger_rows` uses `dict(zip(cols, cells))`, so **inserting** a column anywhere but the end
silently re-aligns every historical row by one position — an entirely wrong ledger that still
parses. The new `minutes` column is therefore appended **last**; `zip` stops at the shorter side,
so pre-existing rows simply have no `minutes` key and render as `—`.

Worth knowing before the forge adds its own column.

### C3. Still open in both copies (unchanged from the 2026-08-03 note)

The delta model assumes **one transcript ≈ one phase**. A single orchestrator session spanning
retro → sweep → story-creation cannot be split by the script, so the whole session lands on
whichever phase records first. Frostvein annotates such rows as unrecoverable rather than
guessing (see the `create` row in
`_bmad-output/implementation-artifacts/metrics/2-1-the-world-runs-on-its-own-clock.md`). The fix
would be a `--mark` call at each phase boundary within a session; recorded rather than built,
since no story needs it yet.

---

## D. What frostvein changed, in one list

For a forge session deciding what to copy:

| Change | File | Class |
| --- | --- | --- |
| Per-layer wall-clock budget, kill-and-continue, timed-out = coverage hole | `_bmad/custom/bmad-code-review.toml` | A1 — adopt |
| `minutes` column on ledger rows; cursor stores `last_ts` so each phase's duration is billed over its own window | `_bmad/scripts/session_tokens.py` | A2 — adopt |
| Rollup "Spend shape" table: turns, tokens, cache-read, cache-read %, output, minutes | `_bmad/scripts/session_tokens.py` | A2 — adopt |
| Full-width guard on ledger row parsing | `_bmad/scripts/session_tokens.py` | C1 — adopt |
| Preserve marker so `--rollup` stops destroying hand-written analysis | `_bmad/scripts/session_tokens.py` | C2 — adopt |
| Metrics tests wired into the quality gate | `scripts/gate.sh` (per-project) | A7 — adopt |
| Tested-instrument addendum | `_bmad/custom/bmad-create-story.toml` (since 2026-08-03) | A4 — adopt |
| Review territories + single verification pass | not yet applied | B1 — **wait** |
