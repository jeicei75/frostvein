---
baseline_commit: ca862749
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
---

# Story 13.3: The Pile Remembers — pile-memo

Status: in-progress

## Story

As the boss,
I want `pile_targets` to keep its last answer until something it reads changes,
so that a pile scenario's tick is no longer mostly one function, and a 200-dwarf crew's **mean** tick fits the budget on today's map (13.2 measured 20.8 ms mean at 128² crew 200; its tail does not fit: p99 97.1 ms, max 134 ms).

## Not stacked: branch off `main`

`main` is `ca862749` (PR #216, 13.2). Branch `story-13-3-the-pile-remembers-pile-memo` carries `b245ba5f` (13.2's done-mark) and this file. Epic 13's standing ACs bind: M3's ACs 1–8, with **one full gate per story, at PR** (M3-4, #212). Wolf ruled the 13.2 proposal on 2026-10-23: this story is first and needs no rule change (`13-2-signoff/proposal.md` § Recommendation item 1, § Ruling).

**Wire diff (standing AC 4): none.** The memo is a cache inside `Terrain`. **Save format: unchanged.** `pile_ground` is never saved, and every `Terrain` starts with an empty one (`lib.rs:2698`).

**Folded in (Wolf, 2026-10-25):** the `launch-gui.ps1` false MISMATCH at the seat after #216 (`gui.exe ca862749 / checkout ca86274`, same commit). Tasks 5–6.

## Found at creation (2026-10-25, on `main` `ca862749`)

Throwaway probe, never pushed: `pile-memo.patch` applied on `main`, with its one `counters::add` line deleted. That is commit `9b5a1a24` in a scratch worktree, and it builds and runs. `RUNS=3 scripts/tick-bench.sh main 9b5a1a24`, load average about 2–6:

| scenario | base ms | head ms | delta | control | floors (base / head / ctl) | state | verdict |
|---|---:|---:|---:|---:|---|---|---|
| big-pile | 1.6058 | 0.1485 | −90.8% | +3.6% | 10.4 / 13.3 / 4.3% | same | BEYOND FLOOR |
| plan-compact-piles | 3.8535 | 0.3965 | −89.7% | −15.4% | 16.5 / 12.3 / 46.8% | same | BEYOND FLOOR |
| plan-spread-piles | 3.5158 | 0.3433 | −90.2% | +7.0% | 49.2 / 45.4 / 10.1% | same | BEYOND FLOOR |
| plan-far-piles | 0.3996 | 0.1487 | −62.8% | −0.1% | 64.4 / 55.9 / 54.7% | same | within floor |
| plan | 0.4670 | 0.2071 | −55.6% | +9.7% | 1.4 / 28.2 / 27.5% | same | BEYOND FLOOR |
| seat-slow | 1.4297 | 0.4775 | −66.6% | −2.6% | 10.6 / 19.3 / 23.7% | same | BEYOND FLOOR |
| deep-pit | 0.0681 | 0.0629 | −7.6% | +0.7% | 22.0 / 18.1 / 8.6% | same | within floor |
| dig-loop | 0.0551 | 0.0545 | −1.2% | −10.1% | 37.5 / 43.1 / 42.6% | same | within floor |
| full-pile-crowd | 0.0497 | 0.0583 | +17.4% | +17.4% | 42.9 / 36.0 / 20.8% | same | within floor |
| stuck-tunnel | 0.3070 | 0.2783 | −9.3% | −17.9% | 6.0 / 4.9 / 11.2% | same | within floor |

What the probe established:
- **The port is hash-equal on all 10 scenarios** (10 `same` cells counted), and the six pile/seat deltas match 13.2's C0 run (−55.5…−91.0%, `13-2-signoff/tickbench.txt` § "C0 vs pile-memo").
- **The `state` column discriminates.** RED: the key always empty, on top (`5ace696a`). `RUNS=3 scripts/tick-bench.sh main 5ace696a scenarios/plan.json scenarios/big-pile.json` reads `differ` on both rows (plan −65.5%, big-pile −91.8%).
- **This devpod was noisy at creation.** Floors reached 64%, against 13.2's ≤5%, and absolute ms read about 1.5–3× 13.2's. plan-far-piles' −62.8% therefore read "within floor". AC 5 says how to handle that.
- **AC 2's fixture kills the near RED.** A probe test passes on `9b5a1a24` and fails under `near: zones.clone()` at its own assert ("a blocked door gives the fresh answer"). Its fixture: `flat_terrain(12, 12)`, zone `{(5,5,1)}`, rock at `(6,5,1)`, `(5,4,1)` and `(5,6,1)`, door `(4,5,1)`. Unblocked gives `{c: 1}`, a blocked door gives `{}`, and the stale answer under the RED is `{c: 1}`.
- **`scale.patch` applies on `main` alone**, without `census` or `counters`. Throwaway commits `ab108c8e` (`main` + scale) and `5b3a2ef8` (`9b5a1a24` + scale), `simd --bench 13-2-signoff/scale/scale-128-crew200.json --runs 1`:
  - `main`: `mean=640.4329 p99=2501.1753 max=3219.522 over10=1396 state=e0b2df85768ab3de`
  - memo: `mean=31.7005 p99=191.9440 max=325.600 over10=1081 state=e0b2df85768ab3de`

  The states are identical, and both equal 13.2's C0 base hash (`scale-hash-base.txt:29`), so the `census`/`counters` stack did not move state either. The mean falls 95%. This devpod runs about 2× 13.2's absolute ms (13.2: 300.5 → 20.8), so compare deltas, not ms. The tail still exceeds the 100 ms budget, as 13.2 said.

## Acceptance Criteria

1. Every `pile_targets` call returns exactly what a fresh computation returns for the same terrain, zones and `blocked`. "Fresh" means a `Terrain` built from the same tiles with an empty `pile_ground`.
2. A lib test changes one `blocked` cell on a **non-zone side neighbour** of the pile, with terrain and zones unchanged. It gets the fresh answer, and asserts that this answer differs from the previous one. A second case does the same on a zone cell. Both 13.2 REDs, re-expressed as mutation rows, fail it.
3. A lib test shows that a repeated call with unchanged inputs is a memo hit. A `set_tile` or a zones change makes the next call a miss. This is the fast-tier guard on the cost (mechanism by design: unit tests are otherwise blind to it). A mutation that never stores the memo fails it.
4. No outcome changes. `scripts/tick-bench.sh main HEAD` prints `same` in the `state` column for all 10 scenarios. Every scenario test in the full gate passes with no pin edited, the cage, ring and two-wide sweeps included.
5. The same run reports the cost. The five pile layouts and seat-slow read −55% or better, each `BEYOND FLOOR`. If a row reads within floor because its floors exceed 10%, rerun it once with load average under 2, and record both runs and the load. deep-pit, dig-loop, full-pile-crowd and stuck-tunnel are reported as measured, with their control column. The 13.2 scale point 128² crew 200 is reported against `main` with mean, p99, max and over-10 ms, and with an identical state hash.
6. `launch-gui.ps1` accepts a `gui.exe` whose stamp abbreviates the checkout's HEAD to a different length. It still refuses a stamp from another commit, and a stamp the checkout cannot resolve to exactly one commit. Wolf sees the first two at the seat (Task 6). The third is seen only if he has a binary from an unfetched commit at hand; otherwise the card records it as unseen.

## Tasks / Subtasks

- [x] **Task 1: port pile-memo into `sim-core` (AC 1, 4).** UPDATE `crates/sim-core/src/lib.rs`. Port `13-2-signoff/prototypes/pile-memo.patch`; do not `git apply` it. It applies on `main`, but it calls `counters::add`, and `main` has no `counters` module, so it does not compile.
  - [x] `PileGround` (`lib.rs:713-717`) gains two fields: `near: BTreeSet<Pos>` (the zone cells and their `side_neighbours`, which are the only cells whose `blocked` the depth map reads) and `memo: Option<(BTreeSet<Pos>, BTreeMap<Pos, u32>)>` (the key, which is the blocked cells among `near`, and the last answer).
  - [x] `Terrain::set_tile` (`lib.rs:742-746`) sets `memo = None` beside `open.clear()`.
  - [x] `pile_targets` (`lib.rs:851`). The zones-change reset (`:868-873`) also builds `near` and sets `memo: None`. Before the floods, build `key = blocked ∩ near`. If it equals the memo's key, return the memo's answer. Otherwise compute as today and store `(key, depth.clone())` before returning. No `counters::add` line.
  - [x] `#[cfg(test)] thread_local! static MEMO_HITS` beside `FLOODED` (`lib.rs:818-822`), incremented on a hit. Task 2's hit test reads it.
  - [x] Rewrite the doc comments on `PileGround` (`:710-712`) and the last paragraph of `pile_targets`' (`:847-850`): what is kept, the key, and what drops it. Add `// NOTE: a hit still builds its key and clones the map (13.2: pile_targets is still 42–46% of a pile scenario's tick after the memo); not optimised further.`
- [x] **Task 2: lib tests (AC 1, 2, 3).** In `lib.rs`'s test module, beside `the_entrance_floods_are_kept_until_the_terrain_or_the_zones_change` (`:7405`). Use the same helpers (`flat_terrain`, `at1`), and change tiles through `set_tile` only.
  - [x] `a_blocked_cell_beside_the_pile_gives_the_fresh_answer` (AC 2). Fixture: one zone cell `c` with rock on three of its side neighbours, so its fourth, `door`, is its only way in (the creation probe's fixture: `flat_terrain(12, 12)`, `c = (5,5,1)`, rock at `(6,5,1)` `(5,4,1)` `(5,6,1)`, `door = (4,5,1)`). (i) With `blocked` empty, assert `{c: 1}`. (ii) With `blocked = {door}` on the SAME terrain, assert the answer equals a fresh terrain's answer for `{door}` (the oracle: rebuild the fixture, so its `pile_ground` is empty) and is not `{c: 1}`. (iii) With `blocked` empty again, assert `{c: 1}`. (iv) Repeat (i)–(iii) with the blocked cell on a zone cell: a two-cell zone, the far cell blocked.
  - [x] `the_pile_answer_is_kept_until_a_tile_or_the_zones_change` (AC 3). Same inputs twice: `MEMO_HITS` goes up by 1 and the answers are equal. After a `set_tile` (any cell): no hit on the next call. After a zones change: no hit on the next call.
- [x] **Task 3: the mutation table (AC 2, 3).** NEW `_bmad-output/implementation-artifacts/mutations/13-3.sh`, in `13-1.sh`'s shape (python heredoc, `assert s.count(old) == 1`). Run it alone, after commit: `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-3.sh`. Every row must be KILLED at its own assert (memory: strengthened test needs re-mutation). Re-express the REDs as rows, because their hunks carry the `counters::add` context line and will not apply:
  1. the key ignores `blocked` (`let key: BTreeSet<Pos> = BTreeSet::new();`, from `pile-memo-red-key-ignores-blocked.patch`) → `a_blocked_cell_beside_the_pile_gives_the_fresh_answer`;
  2. `near` reads the zone cells only (`near: zones.clone()`, from `pile-memo-red-near-zone-only.patch`) → the same test, at case (ii);
  3. `set_tile` keeps the memo → `the_pile_answer_is_kept_until_a_tile_or_the_zones_change`;
  4. a zones change keeps the memo → the same test;
  5. the memo is never stored → the same test (no hit).
- [x] **Task 4: the instrument, RED first (AC 4, 5).** The instrument is 13.1's `scripts/tick-bench.sh`. Its `state` column (`tick-bench.sh:147`, from `--bench`'s `state=` at `crates/simd/src/scenario.rs:320-321`) judges AC 4, and its delta and control columns judge AC 5. It is tested by `crates/simd/tests/bench.rs` and `mutations/13-1.sh`. Run nothing else at the same time (no gate, no `mutate.sh`).
  - [x] **RED.** In a scratch worktree off HEAD, apply row 1's sabotage and commit it `--no-verify` (memory: worktree commit runs the gate). Run `RUNS=3 scripts/tick-bench.sh main <red-sha> scenarios/plan.json scenarios/big-pile.json`. Expected: `differ` on both rows (creation: `differ`, `differ`). Paste it, then remove the worktree.
  - [x] **Green.** `RUNS=5 scripts/tick-bench.sh main HEAD`. Expected: `same` ×10, the six rows at −55% or better and BEYOND FLOOR, and the other four as measured (creation figures above). Paste the table in the Debug Log.
  - [x] **Scale point (AC 5).** `crew` exists only in 13.2's `prototypes/scale.patch`, which applies on `main` alone (checked at creation). Make two throwaway `--no-verify` commits in scratch worktrees: `main` + `scale.patch`, and HEAD + `scale.patch`. Build each in release from its own worktree (memory: CARGO_TARGET_DIR builds cwd sources, `-j 8`). Run `simd --bench _bmad-output/implementation-artifacts/13-2-signoff/scale/scale-128-crew200.json`, `--runs 3` on HEAD and `--runs 1` on `main` (about 10 min a run). Paste both summary lines. Expected: identical `state=` (creation: `e0b2df85768ab3de` on both), with HEAD's mean, p99, max and over10 beside `main`'s. Remove both worktrees.
- [x] **Task 5: `launch-gui.ps1` compares full SHAs (AC 6).** UPDATE `scripts/launch-gui.ps1`.
  - [x] `:133-140`: read `$headFull = git -C $Checkout rev-parse HEAD`, and keep `$head = ... rev-parse --short HEAD` for display only. Rewrite the `:133-134` comment. Its premise, "`build.rs` uses `--short`, so this must too", is what broke: `--short`'s length depends on the repo's object count (7 characters below about 16k objects, 8 above), and the devpod build and the Windows checkout differ.
  - [x] Replace `:201`'s `$stamp -ne $head` string compare. Run `$stampFull = git -C $Checkout rev-parse --verify --quiet "$stamp^{commit}"`, and check `$LASTEXITCODE`, as the `:146-149` comment explains. If it fails or is empty, `Fail` with: the checkout has no single commit for stamp `$stamp` (not fetched, or an ambiguous prefix); rebuild `gui.exe` at `$head`. If `$stampFull -ne $headFull`, keep the existing MISMATCH text, which shows the short forms.
  - [x] Do not touch `crates/gui/build.rs`. The stamp stays short.
  - [x] There is no `pwsh` on the devpod, so this cannot run here. The seat sitting (Task 6) is its only test. Say so in the commit message.
- [x] **Task 6: the seat card (AC 6).** NEW `_bmad-output/implementation-artifacts/13-3-signoff/vehicle-card.md`. Give short inline steps, with no wrapper script (Wolf rejected one at 13.1). Push first: `scripts/push.sh --fast` (standing-authorised for vehicle testing).
  ```powershell
  # PowerShell, Windows checkout. Undo the 2026-10-25 workaround so the lengths differ again:
  git -C D:\Workspace\frostvein config --unset core.abbrev
  git -C D:\Workspace\frostvein rev-parse --short HEAD     # expect 7 characters
  # pull this branch, then, with the OLD gui.exe (stamped ca862749) still in .bin\:
  .\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')        # RED: expect MISMATCH (a real stale binary)
  # rebuild gui.exe at the branch HEAD the usual way, then:
  .\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')        # expect "verified — gui.exe <8 chars>, checkout <7 chars>"
  ```
  Questions for Wolf: (1) Did the old binary read MISMATCH? (2) Did the rebuilt one read `verified` with two different lengths? If both print the same length, the lengths did not differ, so the pass proves nothing; say so. (3) The unresolvable-stamp branch (AC 6's third case) is shown at the seat by `-Exe` with a binary built from a commit the Windows checkout has not fetched. If none is at hand, record it as unseen. Record the answers in the card. No sim seat is needed: sim work is judged by scenario tests (M2-26).
- [x] **Task 7: gate and close.** Fast tier on every commit (pre-commit hook). ONE full gate at PR: `RUST_TEST_THREADS=1 scripts/gate.sh` (about 45–65 min). Paste the block under Debug Log References. Then the cost row (`python3 _bmad/scripts/session_tokens.py --phase dev --story 13-3-the-pile-remembers-pile-memo`), `phase_state.py clear`, and Status → review on the story and the board.

### Review Findings

Code review run 1 (2026-10-25, a fresh session), on `ca862749..f284a0dd`. Four layers, each ran `cargo 1.97.1` in its own `CARGO_TARGET_DIR` on trees extracted with `git archive`; none timed out or failed. Territories: Blind Hunter (Sonnet) `crates/sim-core`; Edge Case Hunter (Sonnet) `launch-gui.ps1` + `mutations/`; Acceptance and Feature Auditors (Opus) the whole diff. Nothing under `/workspace` was modified.

What the layers re-ran: all five `13-3.sh` rows and `12-9.sh` row 20, applied by hand on copies, each KILLED at its recorded assert (edge + acceptance); row 1 also dies at case (iv) alone (acceptance). `--bench` state on all 10 scenarios HEAD vs `ca862749`: `same` 10/10 (feature + acceptance, independently); row 1's sabotage in release reads `differ` on plan/big-pile/seat-slow. Memo hit rate on the live path (feature, instrumented copy): 99.5-100% on all 10 scenarios, at most 1 miss per tick; live daemon on big-pile with `tui --frames 30` 99.88%; scale 128² crew 200 at most 2 misses per tick. Scale point: `main` 291.9 / 289.0 ms mean vs HEAD 19.0 / 19.7 ms, state `e0b2df85768ab3de` on both (feature / acceptance; HEAD tail p99 98-106 ms, max 150-161 ms under sibling load). `git rev-parse --verify --quiet "<stamp>^{commit}"` on Linux git: 7, 8 and 40-char forms of HEAD resolve; another commit resolves to a different SHA; unknown, too-short and ambiguous prefixes give rc 1 with empty output (edge + feature + acceptance). No layer could run `pwsh`.

- [x] [Review][Decision] AC 6 cases 1-2 rest on one unitemised sentence (acceptance MED + feature; severity MED) — Task 6 asks Q1 "did the old binary read MISMATCH?" and Q2 "did the rebuilt one read `verified` with two different lengths?", and says a same-length pass proves nothing. The card records only "so script fix works now", and itself says the lengths were not separately read; the seat had `core.abbrev 8` set before the sitting. Task 6 is ticked, and the Completion Notes and Change Log treat AC 6 as seated. Options: (a) Wolf answers Q1/Q2 now and the card records them; (b) record cases 1-2 as "asserted, not itemised" and keep Task 6 ticked; (c) re-sit.
  **Resolved 2026-10-25: (a).** Asked Q1 (old binary MISMATCH?) and Q2 (rebuilt binary `verified` with two different lengths, `core.abbrev` unset?), Wolf: "yes it's ok". Becomes the patch below.
- [ ] [Review][Patch] Record Wolf's Q1/Q2 answers in the card's § Sitting [13-3-signoff/vehicle-card.md § Sitting] — replace "Q1 and Q2 were not answered one by one" with the itemised answers and their wording; Q3 stays unseen.
- [ ] [Review][Patch] The Debug Log's cause for full-pile-crowd's +5.5% is contradicted by measurement (feature; LOW, patched under the frostvein exception: a wrong recorded cause steers a future optimisation) [13-3-the-pile-remembers-pile-memo.md § Debug Log, Task 4 green paragraph] — it says `blocked ∩ near` "changes on most calls", so every miss pays key + clone; measured 4 key misses in 36,626 calls (99.9% hits). If the cost is real (+2-4% in review, inside the ~6% cross-build resolution), it is the hit path itself: the key scans all of `blocked` (46 loose stones) and clones the map, on a 4-cell pile whose fresh compute is cheap.
- [ ] [Review][Patch] Record text stale after the seat sitting (acceptance; LOW, same lines this record touches) [sprint-status.yaml:2770, story § Completion Notes Task 6 bullet, § File List] — the board comment still says "AC 6 seat card pending Wolf"; the Task 6 completion note keeps "Status goes to review … with AC 6 unseen; the sitting is recorded in the card when it happens"; the File List omits `metrics/13-3-the-pile-remembers-pile-memo.md`, `metrics/.session-cursors.json` and the 13.2 story file (done-mark `b245ba5f`).
- [x] [Review][Defer] Test fixtures assign `Terrain.tiles` directly, bypassing `set_tile` [crates/sim-core/src/lib.rs:5290, :5369, :5446, :6862] — deferred, pre-existing (blind + feature + acceptance, read). The memo widens the `open` cache's existing exposure: a future test that rewrites `.tiles` after a `pile_targets` call reads a stale answer. No current instance (each writes before any tick); production writes go through `set_tile` only.
- [x] [Review][Defer] `launch-gui.ps1` accepts a non-hex stamp that git resolves [scripts/launch-gui.ps1:212] — deferred, unreachable from `build.rs` (edge, RAN git): `gui build HEAD` or `main` resolves and verifies; a stamp starting with `-` (not `-dirty`) reaches git as an option. `build.rs` emits only hex, `unknown` or a suffixed hex form, so only a hand-faked binary gets there. Guard if ever needed: `$stamp -notmatch '^[0-9a-f]{4,40}$'`.

Cost: review row $15.76 (276 turns; 4 subagents 71.0% of tokens). Reap after triage: 15 /tmp directories, 16.4 GB (11.8 GB free space reclaimed).

Dismissed (5): hit-path key build + clone cost (blind; the story's own NOTE and scope guardrail); one-entry memo thrash between the two callers (blind; measured 99.9% hits); a refused channel cut drops the memo with terrain unchanged (feature; cost only, 1-49 drops per run); the scale headline exists only through the throwaway `scale.patch` (feature; by design, M2-26); the scale tail is load-sensitive (acceptance; the record already says the tail does not fit).

## Verification

```bash
# AC 1-3: the lib tests (fast tier)
cargo test -p sim-core --lib pile_
# AC 2-3: the sabotage table; every row KILLED at its own assert
env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-3.sh
# AC 4-5: RED first (Task 4), then
RUNS=5 scripts/tick-bench.sh main HEAD      # state: same x10; six rows <= -55% BEYOND FLOOR
# AC 5: the scale point, Task 4's two throwaway scale.patch builds
simd --bench _bmad-output/implementation-artifacts/13-2-signoff/scale/scale-128-crew200.json --runs 3
# AC 6: Task 6's seat card (Windows; no pwsh on the devpod)
```

## Dev Notes

### Scope guardrails (do NOT)

- Do not add a `counters` module, or port `census.patch`, `counters.patch` or `timer.patch`.
- Do not change a rule. If any scenario hash, sweep list or pinned assert moves, the memo is wrong: stop and find out why. Do not re-pin.
- Do not optimise the hit path (key building, the clone), A\*, or `caging_stone`. That is a later measured story, if one is ever needed.
- Do not land `crew` or `generate_scaled`. `scale.patch` stays a throwaway for Task 4.
- Do not touch squeeze-past or clamber-over (13.4, 13.5), `protocol`, the save format, or `build.rs`.

### What already exists (build on it)

- `PileGround` already keeps the entrance-flood answers (`open`), dropped by `set_tile` and reset on a zones change (`lib.rs:710-748`, `:864-873`, 12.12 review run 2). The memo extends that same struct and the same two drops. Do not build a second cache.
- `pile_targets` has two callers, each once per tick per hauler: `work_positions` (`lib.rs:1378`) and the delivery in `execute_jobs` (`lib.rs:2255`). Both pass `world_blocked`-style sets (emitters plus uncarried items).
- The test to copy is `the_entrance_floods_are_kept_until_the_terrain_or_the_zones_change` (`lib.rs:7405`), which uses the `FLOODED` counter pattern (`:818-822`, `:884`).
- `scripts/tick-bench.sh` (13.1) builds base, head and a shuffled-layout control in scratch worktrees and prints `state same|differ` per row.
- 13.2's evidence: `census-pile-memo.txt` (10/10 hashes equal), `census-pile-memo-red-{key,near}.txt`, `tickbench.txt` § "C0 vs pile-memo", `scale-r1-pile-memo.txt`, `timer-pile-memo.txt`.

### Key decisions & traps

- **The answer depends only on terrain, zones, and `blocked` ∩ (zone cells ∪ their side neighbours).** `is_walkable` is asked only of those cells (`:907-910`, `:919`), and `open_ground` reads terrain with `no_items` (`:863`, `:893`). So that key is exact, not a heuristic.
- **The near RED is invisible to every scenario hash.** Only the AC 2 side-neighbour case catches it. The full gate catches it only incidentally, through the cage sweep's `frozen` pin (`tests/scenario.rs:2988`, `:3118`; 13.2 `census-pile-memo-red-near.txt`).
- **Direct `terrain.tiles[...]` writes bypass the drop.** Test fixtures that write tiles directly (`lib.rs:5557`, `:5596`, …) do so before any `pile_targets` call. New tests use `set_tile`.
- **The existing kept-floods test (`:7405`) must stay green unedited.** Its repeated call is now a memo hit, which floods nothing, so `flooded == 0` still holds. The probe built in release but never ran clippy. If `-D warnings` flags `type_complexity` on `memo`, a local type alias is fine.
- **Cross-build resolution is about 6%** (13.1 review run 2). The six expected drops clear it about ten times over. The other four rows are reported, not judged.
- **`git rev-parse --short` length is per repo.** The devpod (about 17.5k objects) prints 8, and the seat checkout printed 7 until `core.abbrev 8` was set as a workaround. A MISMATCH where one SHA is a prefix of the other is this bug, not a stale binary.

### Previous story intelligence

- 13.2's REDs and patches are C0-stack artefacts (on `4f57027` + `census` + `counters`). Port them, and re-express the REDs as mutation rows. Its `census.sh` and `scale.sh` hardcode `BASE=4f57027`, so do not use them as this story's instrument.
- 13.2's first hash check was vacuous: the regex matched 0 lines on both sides. Count the `same` cells (10) before calling the states equal.
- **PR #216's body closed #189 and #206** through prose ("fixes #206 and #189"). Write this story's PR body with `Closes`/`Fixes` only where meant. This story closes no issue.

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs` (`PileGround`, `Terrain::set_tile`, `pile_targets`, the test module)
- UPDATE `scripts/launch-gui.ps1`
- NEW `_bmad-output/implementation-artifacts/mutations/13-3.sh`
- NEW `_bmad-output/implementation-artifacts/13-3-signoff/vehicle-card.md`
- No new dependency.

### References

- `_bmad-output/planning-artifacts/epics.md` Story 13.3; Epic 13 standing ACs
- `_bmad-output/implementation-artifacts/13-2-signoff/proposal.md` § Profile ("After pile-memo"), § Prototypes item 4, § Recommendation item 1, § Ruling
- `_bmad-output/implementation-artifacts/13-2-signoff/prototypes/pile-memo.patch`, `pile-memo-red-key-ignores-blocked.patch`, `pile-memo-red-near-zone-only.patch`, `scale.patch`
- `_bmad-output/implementation-artifacts/13-2-signoff/scale/scale-128-crew200.json`
- `crates/sim-core/src/lib.rs:703-748`, `:818-926`, `:1378`, `:2255`, `:7405`
- `scripts/tick-bench.sh`; `crates/simd/src/scenario.rs:320-321`
- `scripts/launch-gui.ps1:133-140`, `:187-209`; `crates/gui/build.rs:40-55`
- `docs/technical-preferences.md` (ADR 5 scope line, anti-overengineering rules)

## Dev Agent Record

### Agent Model Used

Implementer: Sonnet 5.5 subagents (`claude-sonnet-5-5`), Opus 5.5 orchestrating (Wolf, 2026-10-25).

### Debug Log References

**Task 3, mutation table** (`mutations/13-3.sh`, run alone after commit `2a5a86d8`): 5/5 KILLED, each at its own assert.

| row | sabotage | killed by | at |
|---|---|---|---|
| 1 | the key ignores `blocked` (`.filter(\|_\| false)`) | `a_blocked_cell_beside_the_pile_gives_the_fresh_answer` | `lib.rs:7521`, "(ii) a blocked (4,5,1) gives the fresh answer" |
| 2 | `near` reads the zone cells only (`.copied()`) | the same test | `lib.rs:7521`, the same (ii) assert |
| 3 | `set_tile` keeps the memo | `the_pile_answer_is_kept_until_a_tile_or_the_zones_change` | `lib.rs:7553`, "a set_tile drops the memo" |
| 4 | a zones change keeps the memo (`memo: kept.memo.take()`) | the same test | `lib.rs:7563`, "a zones change drops the memo" |
| 5 | the memo is never stored | the same test | `lib.rs:7547`, "unchanged inputs are a hit" |

Rows 1-2 stop at the one-cell fixture, so case (iv) (the zone-cell case) has no row of its own; rows 3-4 also fail the unedited `the_entrance_floods_are_kept_until_the_terrain_or_the_zones_change` (implementer's hand check). **`12-9.sh` row 20 re-pointed:** its second anchor (`}` `}` `depth`) ran into the new memo store, so the fast gate's "mutation tables still apply" check went red at the first commit attempt. The retain now lands BEFORE `kept.memo = Some(...)`, so the kept answer is the sabotaged one too. Re-run alone: KILLED, `a_walled_in_free_cell_does_not_stop_the_pile` at its `watch` bound (`lib.rs:6925`, the delivery never lands).

**Task 4 RED** (row 1's sabotage as throwaway `f7ed06fb` in a scratch worktree, removed): `RUNS=3 scripts/tick-bench.sh main f7ed06fb scenarios/plan.json scenarios/big-pile.json`, load 4.77 before / 6.69 after:

```
scenario                  base_ms    head_ms    delta  control base_floor head_floor  ctl_floor  state  verdict
plan                       0.2409     0.0980   -59.3%    -6.4%      30.2%      48.1%      41.4%  differ BEYOND FLOOR
big-pile                   1.1440     0.1272   -88.9%   +10.3%       7.9%      79.5%      33.4%  differ BEYOND FLOOR
```

**Task 4 green:** `RUNS=5 scripts/tick-bench.sh main HEAD` (HEAD `2a5a86d8`), load 4.51 before / 1.36 after. `same` counted: 10 of 10.

```
scenario                  base_ms    head_ms    delta  control base_floor head_floor  ctl_floor  state  verdict
big-pile                   1.0573     0.0887   -91.6%    -2.7%       1.7%       2.6%       2.1%  same   BEYOND FLOOR
deep-pit                   0.0379     0.0338   -10.8%    -1.5%       1.8%       2.4%       2.9%  same   BEYOND FLOOR
dig-loop                   0.0349     0.0357    +2.2%    +1.1%       3.2%       2.5%       2.8%  same   within floor
full-pile-crowd            0.0311     0.0328    +5.5%    +0.6%       3.5%       3.4%       5.4%  same   BEYOND FLOOR
plan-compact-piles         1.4132     0.1456   -89.7%    -2.3%       1.5%       5.4%       5.0%  same   BEYOND FLOOR
plan-far-piles             0.2250     0.0834   -62.9%    -3.4%       2.3%       1.7%       1.8%  same   BEYOND FLOOR
plan-spread-piles          1.6895     0.1614   -90.4%    -2.7%       3.4%       3.0%       1.8%  same   BEYOND FLOOR
plan                       0.1874     0.0780   -58.4%    -3.1%       0.9%       1.3%       0.8%  same   BEYOND FLOOR
seat-slow                  0.4687     0.1613   -65.6%    -6.3%       2.3%       3.3%       2.8%  same   BEYOND FLOOR
stuck-tunnel               0.0964     0.0940    -2.4%    +0.5%       0.7%       0.5%       1.2%  same   BEYOND FLOOR
```

The six pile/seat rows read -58.4..-91.6%, all BEYOND FLOOR, every floor under 10% (quiet devpod; no rerun needed). The other four, as measured: deep-pit -10.8%, stuck-tunnel -2.4%, dig-loop +2.2% (within floor), and **full-pile-crowd +5.5% BEYOND FLOOR** (control +0.6%). That is inside the ~6% cross-build resolution (13.1 review run 2), so it is not established. Corrected at code review run 1: the cause first written here, that `blocked ∩ near` changes on most calls so every miss pays the key build and the clone, is wrong. Measured, 4 key misses in 36,626 calls (99.9% hits). If the cost is real (+2-4% in review), it is the hit path itself: the key scans all of `blocked` (46 loose stones) and clones the map, on a 4-cell pile whose fresh compute is cheap. Not optimised (scope guardrail).

**Task 4 scale point** (`scale.patch` on `main` = `8a75da2e`, on HEAD = `0a2d1455`, throwaway, each built in release from its own worktree, removed), `simd --bench 13-2-signoff/scale/scale-128-crew200.json`:

```
HEAD run 1/3: ticks=2000 commands=1801/1801 mean=16.6793 p99=85.5127 max=116.787 over10=947 state=e0b2df85768ab3de
HEAD run 2/3: ticks=2000 commands=1801/1801 mean=16.8790 p99=86.6331 max=114.000 over10=951 state=e0b2df85768ab3de
HEAD run 3/3: ticks=2000 commands=1801/1801 mean=16.8892 p99=85.5163 max=117.399 over10=948 state=e0b2df85768ab3de
main run 1/1: ticks=2000 commands=1801/1801 mean=273.2888 p99=794.6823 max=863.505 over10=1305 state=e0b2df85768ab3de
```

Identical state (`e0b2df85768ab3de`, = creation and 13.2's C0 base). Mean -93.9% (273.3 -> 16.7-16.9 ms, spread 1.2%), p99 794.7 -> ~86 ms, over-10 ms ticks 1305 -> ~949. The tail does NOT fit the 100 ms budget: max 114-117 ms, as 13.2 said.

**Task 7, full gate** on `46f9afa1` (`RUST_TEST_THREADS=1 scripts/gate.sh`):

```
frostvein gate
  cargo fmt --check           ok  1s
  cargo clippy -D warnings    ok  2s
  cargo test                  ok  683s
  cargo test (pixel guards)   ok  3057s
  tui has no sim-core edge                ok
  client-core has no sim-core edge        ok
  gui has no sim-core edge                ok
  metrics ledger tests        ok  0s
  bench tests                 ok  19s
  mutation tables still apply ok  13s
reaped 16 stale build artifacts from target (0.0 GB)
GATE GREEN  3785s
```

No pin edited; every scenario test, the cage, ring and two-wide sweeps included, passed (AC 4).

### Completion Notes List

- Implementer: one Sonnet 5.5 subagent wrote Tasks 1, 2 and 5 (lib.rs port + two tests, launch-gui.ps1), checking each of the five sabotages by hand. Opus orchestrated, reviewed the diff, re-pointed `12-9.sh` row 20, wrote and ran the mutation table, ran every tick-bench and scale run, and wrote the card.
- The hit check is a let-chain (`if let ... && *memo_key == key`): clippy's `collapsible_if` flagged the nested form. No `type_complexity` warning, so no alias.
- AC 1-3: lib tests + 5/5 KILLED. AC 4: `same` 10/10 on tick-bench and an identical state at the scale point; the full gate (scenario pins, sweeps) is Task 7. AC 5: the table and scale lines above. AC 6: `launch-gui.ps1` is unrun here (no `pwsh`); the seat card is its only test.
- **Task 6 (seat, 2026-10-25):** Wolf: "so script fix works now"; Q1/Q2 answered one by one at code review run 1, "yes it's ok" (recorded in the card; Q3 unseen). The card was written and the branch pushed (`push.sh --fast` VERIFIED `46f9afa1`) before the sitting.

### File List

- `crates/sim-core/src/lib.rs` (modified)
- `scripts/launch-gui.ps1` (modified)
- `_bmad-output/implementation-artifacts/mutations/13-3.sh` (new)
- `_bmad-output/implementation-artifacts/mutations/12-9.sh` (modified: row 20 re-pointed)
- `_bmad-output/implementation-artifacts/13-3-signoff/vehicle-card.md` (new)
- `_bmad-output/implementation-artifacts/13-3-the-pile-remembers-pile-memo.md` (this record)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (status)
- `_bmad-output/implementation-artifacts/13-2-the-way-through-pathfinding-and-reachability-spike.md` (modified: done-mark `b245ba5f`)
- `_bmad-output/implementation-artifacts/metrics/13-3-the-pile-remembers-pile-memo.md` (new: cost rows)
- `_bmad-output/implementation-artifacts/metrics/.session-cursors.json` (modified: metric cursor)

## Change Log

| Date | Change |
|---|---|
| 2026-10-25 | Code review run 1 (fresh session, 4 layers, none timed out) on `ca862749..f284a0dd`: 0 HIGH. Memo hits 99.5-100% live, state `same` 10/10 re-verified, scale 128^2 crew200 ~290 -> ~19 ms mean same state. Decision (AC 6 seat itemised) resolved: Wolf, Q1/Q2 "yes it's ok". 3 record patches LEFT AS ACTION ITEMS; 2 deferred; 5 dismissed. Status in-progress. |
| 2026-10-25 | AC 6 seat: Wolf, "so script fix works now"; Q3 (unfetched stamp) unseen. Task 6 checked. |
| 2026-10-25 | Dev (Sonnet 5.5 subagent + Opus): pile-memo in `pile_targets`, two lib tests, `mutations/13-3.sh` 5/5 KILLED, `12-9.sh` row 20 re-pointed, launch-gui full-SHA compare, seat card. tick-bench `same` 10/10, six rows -58..-92%; scale 128^2 crew 200 mean 273.3 -> 16.8 ms, same state. Full gate GREEN on `46f9afa1` (3785 s). Status review; AC 6 seat pending. |
| 2026-10-25 | Story created (Opus 5.5) from Epic 13 Story 13.3 and the 13.2 proposal, with the launch-gui full-SHA fix folded in per Wolf's ruling. The creation probe ran pile-memo ported onto `main` against `main` (see Found at creation). Status ready-for-dev. |
