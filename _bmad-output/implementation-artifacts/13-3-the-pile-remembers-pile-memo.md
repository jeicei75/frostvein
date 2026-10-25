---
baseline_commit: ca862749
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
---

# Story 13.3: The Pile Remembers — pile-memo

Status: ready-for-dev

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

- [ ] **Task 1: port pile-memo into `sim-core` (AC 1, 4).** UPDATE `crates/sim-core/src/lib.rs`. Port `13-2-signoff/prototypes/pile-memo.patch`; do not `git apply` it. It applies on `main`, but it calls `counters::add`, and `main` has no `counters` module, so it does not compile.
  - [ ] `PileGround` (`lib.rs:713-717`) gains two fields: `near: BTreeSet<Pos>` (the zone cells and their `side_neighbours`, which are the only cells whose `blocked` the depth map reads) and `memo: Option<(BTreeSet<Pos>, BTreeMap<Pos, u32>)>` (the key, which is the blocked cells among `near`, and the last answer).
  - [ ] `Terrain::set_tile` (`lib.rs:742-746`) sets `memo = None` beside `open.clear()`.
  - [ ] `pile_targets` (`lib.rs:851`). The zones-change reset (`:868-873`) also builds `near` and sets `memo: None`. Before the floods, build `key = blocked ∩ near`. If it equals the memo's key, return the memo's answer. Otherwise compute as today and store `(key, depth.clone())` before returning. No `counters::add` line.
  - [ ] `#[cfg(test)] thread_local! static MEMO_HITS` beside `FLOODED` (`lib.rs:818-822`), incremented on a hit. Task 2's hit test reads it.
  - [ ] Rewrite the doc comments on `PileGround` (`:710-712`) and the last paragraph of `pile_targets`' (`:847-850`): what is kept, the key, and what drops it. Add `// NOTE: a hit still builds its key and clones the map (13.2: pile_targets is still 42–46% of a pile scenario's tick after the memo); not optimised further.`
- [ ] **Task 2: lib tests (AC 1, 2, 3).** In `lib.rs`'s test module, beside `the_entrance_floods_are_kept_until_the_terrain_or_the_zones_change` (`:7405`). Use the same helpers (`flat_terrain`, `at1`), and change tiles through `set_tile` only.
  - [ ] `a_blocked_cell_beside_the_pile_gives_the_fresh_answer` (AC 2). Fixture: one zone cell `c` with rock on three of its side neighbours, so its fourth, `door`, is its only way in (the creation probe's fixture: `flat_terrain(12, 12)`, `c = (5,5,1)`, rock at `(6,5,1)` `(5,4,1)` `(5,6,1)`, `door = (4,5,1)`). (i) With `blocked` empty, assert `{c: 1}`. (ii) With `blocked = {door}` on the SAME terrain, assert the answer equals a fresh terrain's answer for `{door}` (the oracle: rebuild the fixture, so its `pile_ground` is empty) and is not `{c: 1}`. (iii) With `blocked` empty again, assert `{c: 1}`. (iv) Repeat (i)–(iii) with the blocked cell on a zone cell: a two-cell zone, the far cell blocked.
  - [ ] `the_pile_answer_is_kept_until_a_tile_or_the_zones_change` (AC 3). Same inputs twice: `MEMO_HITS` goes up by 1 and the answers are equal. After a `set_tile` (any cell): no hit on the next call. After a zones change: no hit on the next call.
- [ ] **Task 3: the mutation table (AC 2, 3).** NEW `_bmad-output/implementation-artifacts/mutations/13-3.sh`, in `13-1.sh`'s shape (python heredoc, `assert s.count(old) == 1`). Run it alone, after commit: `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-3.sh`. Every row must be KILLED at its own assert (memory: strengthened test needs re-mutation). Re-express the REDs as rows, because their hunks carry the `counters::add` context line and will not apply:
  1. the key ignores `blocked` (`let key: BTreeSet<Pos> = BTreeSet::new();`, from `pile-memo-red-key-ignores-blocked.patch`) → `a_blocked_cell_beside_the_pile_gives_the_fresh_answer`;
  2. `near` reads the zone cells only (`near: zones.clone()`, from `pile-memo-red-near-zone-only.patch`) → the same test, at case (ii);
  3. `set_tile` keeps the memo → `the_pile_answer_is_kept_until_a_tile_or_the_zones_change`;
  4. a zones change keeps the memo → the same test;
  5. the memo is never stored → the same test (no hit).
- [ ] **Task 4: the instrument, RED first (AC 4, 5).** The instrument is 13.1's `scripts/tick-bench.sh`. Its `state` column (`tick-bench.sh:147`, from `--bench`'s `state=` at `crates/simd/src/scenario.rs:320-321`) judges AC 4, and its delta and control columns judge AC 5. It is tested by `crates/simd/tests/bench.rs` and `mutations/13-1.sh`. Run nothing else at the same time (no gate, no `mutate.sh`).
  - [ ] **RED.** In a scratch worktree off HEAD, apply row 1's sabotage and commit it `--no-verify` (memory: worktree commit runs the gate). Run `RUNS=3 scripts/tick-bench.sh main <red-sha> scenarios/plan.json scenarios/big-pile.json`. Expected: `differ` on both rows (creation: `differ`, `differ`). Paste it, then remove the worktree.
  - [ ] **Green.** `RUNS=5 scripts/tick-bench.sh main HEAD`. Expected: `same` ×10, the six rows at −55% or better and BEYOND FLOOR, and the other four as measured (creation figures above). Paste the table in the Debug Log.
  - [ ] **Scale point (AC 5).** `crew` exists only in 13.2's `prototypes/scale.patch`, which applies on `main` alone (checked at creation). Make two throwaway `--no-verify` commits in scratch worktrees: `main` + `scale.patch`, and HEAD + `scale.patch`. Build each in release from its own worktree (memory: CARGO_TARGET_DIR builds cwd sources, `-j 8`). Run `simd --bench _bmad-output/implementation-artifacts/13-2-signoff/scale/scale-128-crew200.json`, `--runs 3` on HEAD and `--runs 1` on `main` (about 10 min a run). Paste both summary lines. Expected: identical `state=` (creation: `e0b2df85768ab3de` on both), with HEAD's mean, p99, max and over10 beside `main`'s. Remove both worktrees.
- [ ] **Task 5: `launch-gui.ps1` compares full SHAs (AC 6).** UPDATE `scripts/launch-gui.ps1`.
  - [ ] `:133-140`: read `$headFull = git -C $Checkout rev-parse HEAD`, and keep `$head = ... rev-parse --short HEAD` for display only. Rewrite the `:133-134` comment. Its premise, "`build.rs` uses `--short`, so this must too", is what broke: `--short`'s length depends on the repo's object count (7 characters below about 16k objects, 8 above), and the devpod build and the Windows checkout differ.
  - [ ] Replace `:201`'s `$stamp -ne $head` string compare. Run `$stampFull = git -C $Checkout rev-parse --verify --quiet "$stamp^{commit}"`, and check `$LASTEXITCODE`, as the `:146-149` comment explains. If it fails or is empty, `Fail` with: the checkout has no single commit for stamp `$stamp` (not fetched, or an ambiguous prefix); rebuild `gui.exe` at `$head`. If `$stampFull -ne $headFull`, keep the existing MISMATCH text, which shows the short forms.
  - [ ] Do not touch `crates/gui/build.rs`. The stamp stays short.
  - [ ] There is no `pwsh` on the devpod, so this cannot run here. The seat sitting (Task 6) is its only test. Say so in the commit message.
- [ ] **Task 6: the seat card (AC 6).** NEW `_bmad-output/implementation-artifacts/13-3-signoff/vehicle-card.md`. Give short inline steps, with no wrapper script (Wolf rejected one at 13.1). Push first: `scripts/push.sh --fast` (standing-authorised for vehicle testing).
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
- [ ] **Task 7: gate and close.** Fast tier on every commit (pre-commit hook). ONE full gate at PR: `RUST_TEST_THREADS=1 scripts/gate.sh` (about 45–65 min). Paste the block under Debug Log References. Then the cost row (`python3 _bmad/scripts/session_tokens.py --phase dev --story 13-3-the-pile-remembers-pile-memo`), `phase_state.py clear`, and Status → review on the story and the board.

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

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-10-25 | Story created (Opus 5.5) from Epic 13 Story 13.3 and the 13.2 proposal, with the launch-gui full-SHA fix folded in per Wolf's ruling. The creation probe ran pile-memo ported onto `main` against `main` (see Found at creation). Status ready-for-dev. |
