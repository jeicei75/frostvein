# Frostvein → Nidavellir transfer note (2026-08-03)

**For:** a Claude session rooted at `/workspace` (the Nidavellir forge — its own BMad, its
own memory store `-workspace`, its own `scripts/codex-handoff.sh`).
**From:** frostvein's Epic 1 retrospective,
`/workspace/projects/frostvein/_bmad-output/implementation-artifacts/epic-1-retro-2026-08-03.md`.

Frostvein owns its own process and does not write into the forge's `_bmad-output/`. This
file is a **read-only handoff**: adopt what fits, write your own memories, ignore the rest.
Every claim below names the file or command that verifies it — check rather than trust.

Split by evidence class. **Section A is fact and transfers today. Section B is hypothesis
and must not be adopted until frostvein Story 2.1 settles it.**

---

## A. Facts — verified, adopt now

### A1. `AGENTS.md` must exist at whatever directory Codex is `-C`'d into

**The forge is already correct at root, and this is not a bug report against it.**
`/workspace/AGENTS.md` is byte-identical to `/workspace/CLAUDE.md` (`diff -q` → identical),
and `/workspace/scripts/codex-handoff.sh` runs `-C /workspace`. So forge-root Codex work
reads exactly the forge's rules.

**It breaks silently the moment Codex is pointed at a sub-repo.** Codex resolves
`AGENTS.md` from its working root upward. Frostvein's handoff runs
`-C /workspace/projects/frostvein`, which has no `AGENTS.md`, so Codex walked up and found
the forge's generic copy — tool hygiene only, and `rg -c -i 'frostvein|rust|cargo'` over it
returns nothing. Across all three Epic 1 stories, Codex never read the four-crate layout,
the quality gate, YAGNI-as-policy, `#![forbid(unsafe_code)]`, the commit author,
"determinism is load-bearing", or "clients contain zero game logic". All of it existed — in
`frostvein/CLAUDE.md`, a filename Codex does not open.

**Why it matters beyond the missing rules:** everything project-specific then arrives only
through the per-story handoff prompt, and **a new Codex session re-reads `AGENTS.md` but
not the prior run's prompt.** Frostvein's story 1.2 spanned two Codex runs and the second
logged *"RED not observed because the bridge implementation already existed"* for two whole
tasks — TDD discipline lost at a session boundary, exactly as this predicts.

**Check on your side:** none of the nine repos under `/workspace/projects/` has an
`AGENTS.md`, and only frostvein has a `CLAUDE.md`. If any forge workflow ever hands Codex a
sub-repo (yggdrasil has its own memory store, so it is a plausible target), it inherits this
silently. Cheapest fix per repo: `ln -s CLAUDE.md AGENTS.md`, or a real file where the two
audiences differ.

### A2. `sandbox_workspace_write.network_access=true` — this closes YOUR open "fix-later"

Your `codex-handoff-runbook` memory records: *"sandbox (no-net) blocks ruff + aiosqlite
tests, so orchestrator MUST re-run the full gate (fix-later)."* That is the same problem
frostvein hit, and it is solvable today.

The knob exists in codex-cli 0.146.0 and unblocks loopback TCP (the sandbox denies `bind`
with `PermissionDenied`, which reads like a code bug but is not). Fixing the *sandbox
config* is legitimate — the never-work-around rule governs production code, not the harness.

**Wolf adopted it for frostvein on 2026-08-03**, baked into the handoff script by default,
because Epic 2's daemon stories all need live sockets. **Accepted trade-off, stated
plainly:** the knob is a boolean, not a per-host allowlist, so it grants full internet. The
closed-dependency-stack guarantee downgrades from *sandbox-enforced* to *prompt-instructed*
— frostvein keeps `cargo fetch` prewarm as the orchestrator's job and keeps Codex building
`--offline`. Your equivalent would be keeping dependency installation orchestrator-side.

### A3. `.git` in `writable_roots` — frostvein's handoff has it, the forge's does not

`diff /workspace/scripts/codex-handoff.sh /workspace/projects/frostvein/scripts/codex-handoff.sh`
shows frostvein added:

```
-c 'sandbox_workspace_write.writable_roots=["/workspace/projects/frostvein/.git"]'
```

`workspace-write` shields `.git` by default, so without it `git checkout -b` dies with
*"cannot lock ref … Read-only file system"* and no branch or commit is possible. There is
no `allow_git_writes` key in codex-cli 0.146.0.

`/workspace` **is** a git repo (`git@github.com:jeicei75/nidavellir.git`), and your
`codex-incremental-commits` memory says Codex is expected to commit on every green. Worth
checking whether forge handoffs are hitting this and working around it, or whether
something else in your setup already covers it. Stated as a checkable delta, not a claim.

### A4. The self-referential test antipattern, and how to detect it

Hit in **all three** frostvein stories despite being written into two consecutive story
files as an explicit warning. Root cause in one sentence: *the test ran the oracle and the
implementation through the same function*, so it proved ordering and never mapping.
Sabotages that survived a green suite included `Ice`↔`Snow` swaps, inverted terrain
layering, `panic!()` on every inbound line, and an x/y transposition.

Two detection rules worth carrying:

- **Assert against a hand-written literal**, never a round-trip through the code under
  test. A *symmetric* rename (dropping a `#[serde(rename)]`) passes a round-trip suite and
  breaks every real client.
- **Sabotage the constants, not just the mappings.** Frostvein's 1.3 had genuinely
  independent oracles and still failed: raising `PEEK_DEPTH` 3→6 left all 13 tests green,
  because the fixture lacked the *range* to express the negative case. A checked-off
  coverage subtask is a claim; sabotaging the constant is the only thing that verifies it.

This converges with your own banked ep-11 items (*"live-gate-only absent-capability
detection 3rd time"*, *"Feature Auditor must start at discovery"*). Two independent projects
finding the same class is stronger evidence than either alone.

### A5. Decompose your review cost — the method, not frostvein's numbers

Frostvein's Epic 1: create $54.40 (21%) · dev $8.76 (**3%**) · review $192.95 (**75%**) =
$256.11. Confirmed by `python3 _bmad/scripts/session_tokens.py --rollup 1`.

The useful part is the *decomposition*, which the rollup does not yet do — computed by hand
from the metrics rows across 634 review turns:

| component | share | why |
| --- | --- | --- |
| cache_read | **61%** | ~123k tokens of context re-read **per turn**, at Opus's $1.50/MTok |
| output | 26% | the model actually producing findings |
| cache_write | 14% | |

**Review was not expensive because it thought hard. It was expensive because it re-read.**
That reframes the levers as model tier, context scope and turn count — not rigor. Run
`--rollup` on your own epics and decompose the same way before assuming your shape matches;
frostvein's is a Rust project with 4-layer reviews and yours may differ.

**Caveat when reading any of these numbers:** `total` is dominated by cheap cache reads and
is misleading. `est_usd` is the only comparable column.

---

## B. Hypotheses — do NOT adopt yet

These are frostvein's Epic 2 experiments. Each has a stated measure and a revert rule. They
are unproven, and adopting an unproven cost cut is how a regression gets exported.
**Settled after frostvein Story 2.1 — re-read this section then.**

| # | Hypothesis | Evidence today | Measure that settles it |
| --- | --- | --- | --- |
| B1 | Blind Hunter + Edge Case Hunter run fine on Sonnet; Acceptance Auditor + Feature Auditor need Opus | Per-layer verdict from one story (1.3). The two auditors each found something no hunter did — a real-pty run, and a constant-sabotage. The hunters converged on the same cluster. | 2.1's mutation kill-rate vs 1.3's band (10 patches, 2 decisions, 3 deferred, 6 dismissed). Revert Lever A if it drops. |
| B2 | Making the *dev* agent sabotage-verify cuts review patch count | n=1. 1.3 is the only story where Codex sabotage-verified unprompted, and the only one whose mapping tests came through review clean. | 2.1's Opus patch count vs 1.3's ten. |
| B3 | `codex review --base main` as a pre-handback self-gate is worth its cost | Zero. Never run. Known blind spot: it marks its own homework. | Same patch-count measure as B2. |
| B4 | `gate.sh` + pre-commit hook removes the record-accuracy failure class | Mechanically obvious, unmeasured. Motivated by frostvein 1.1, where the dev record claimed a test that did not exist. | Absence of "claimed but absent" findings in 2.x reviews. |

---

## C. Reverse direction — what frostvein should take from the forge

Recorded here so the exchange is honest in both directions.

1. **You automated metrics recording; frostvein does it by hand.** Your `ep02-working-cadence`
   memory records the review `on_complete` hook in `bmad-code-review.toml` plus `--rollup`.
   Frostvein inherited the script but never wired the hook, and had not run `--rollup` at
   all until this retrospective.
2. **`codex-incremental-commits`** — your memory notes Codex squashing instead of
   committing per green, *three times*. Frostvein has not hit it yet and should watch for
   it rather than rediscover it.
3. **Shared-script bug (`session_tokens.py --rollup`):** the story column mangles
   frostvein-style story keys — `1-1-a-seeded-frozen-world-exists` renders as `seeded`,
   `1-2-...` as `daemon`, `1-3-...` as `the`. Cosmetic, but the script is shared, so the fix
   belongs upstream.

---

## D. Suggested reading order for the forge session

1. This file.
2. `_bmad-output/implementation-artifacts/epic-1-retro-2026-08-03.md` §4 (the structural
   finding) and §5 (Wolf's three decisions with their trade-offs).
3. `deferred-work.md` — only if you want the shape of how deferrals are recorded with
   revisit triggers; the content is frostvein-specific.

Then write your own memories. Do not copy frostvein's — memory stores are per-project
(`-workspace` vs `-workspace-projects-frostvein`) and nothing transfers automatically,
which is correct: the instance that owns the store should decide what it believes.
