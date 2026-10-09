---
baseline_commit: 26185a0
model: claude-opus-5-5  # session default
---

# Story 12.9: One Dwarf per Tile

Status: in-progress

## Story

As the boss,
I want dwarves to step around each other instead of through each other,
so that my crew looks like bodies in a real place.

## Not stacked: branch off `main`

`main` is `26185a0` (PR #178, 12.8, merged), clean. Branch: `story-12-9-one-dwarf-per-tile`.
Closes **#133** and **#162** (folded in at Task 0, Wolf 2026-10-08). #132 (12.3) is fixed, as the epic order requires. Epic 12's nine standing ACs
(`epics.md`, "Standing acceptance criteria") bind this story and are not restated.
**Wire diff: none.** **New saved state: `SavedDwarf.path`** (Task 4).

## Found at creation (2026-10-08, on `26185a0`)

**#133 reproduced in the sim and on the live wire.** A throwaway probe was run and deleted:

- Idle crew, no orders, 3,000 ticks: two dwarves share a tile on **494** ticks (`DEFAULT_SEED`;
  first at tick 2, dwarves 1 and 3 on `(65,63,9)`), **667** (seed 42) and **526** (seed 7451).
- Busy crew on `DEFAULT_SEED`: a 4x7 channel `(65..68, 61..67, z9)` and a 3x3 pile `(59..61, 64..66, z9)`
  at tick 0, run for 3,000 ticks. **332** shared ticks. This baseline sets AC1's bounds:
  - marks: 25 → 0 by tick **1,112**;
  - first stone on the pile at tick **217**, and the pile full (9) by ~1,000;
  - **1,241** dwarf moves.
- Live RED on a release `simd`, with the story's instrument (Verification): `shared ticks 186`,
  `OCCUPANCY RED`.

**The cause is by design.** Occupancy is not a movement rule (`lib.rs:1339-1340`). Three production
systems write a dwarf's `Pos`, and none of them checks for other dwarves:

- `execute_jobs`, the job step (`lib.rs:1124-1126`);
- `settle` (`lib.rs:1266-1268`);
- `wander` (`lib.rs:1351`).

`spawn_dwarves` already places the five on distinct tiles (`candidates.swap_remove`, `lib.rs:2091`).

**FR49's head-on rule, read per step, livelocks.** Hand-traced in a dead-end tunnel `x1..x6` (rock
at `x7`, open ground west of `x1`):
- Lower-id A is at `x5`, walking out. Higher-id B is at `x4`, walking in to `x6`.
- A backs off to `x6`, and B steps to `x5`. A now has nowhere to go, so B backs off to `x4` and A
  steps to `x5`. A has a free cell behind it again, so A backs off.
- This repeats forever.

"Nowhere to go" must therefore mean **no escape**: no free cell, reachable without passing the
other dwarf, that lies off the other dwarf's remaining path. It cannot mean "no free neighbour this
tick" (Key decisions).

**`Path` is not saved** (`SavedDwarf`, `save.rs:30-45`). A loaded dwarf recomputes a fresh A* path.
Two consequences:
- A detour taken to avoid a dwarf is not what a fresh A* returns, so a world saved mid-detour
  diverges after load.
- A head-on test that reads the blocker's next step sees nothing after a load.

Both break standing AC3, so the path joins `SaveState`.

**#162 (stones never block) names 12.9 as one place to decide it** (its option 3). Task 0 Q3:
**folded in, and every uncarried item blocks, on a pile cell too** (Wolf). See "Stones block" below.

## Acceptance Criteria

1. **RED first.** A scenario test on `DEFAULT_SEED` places 12.9's channel and pile (the busy crew
   above) and runs 3,000 ticks. It asserts that **no tick** has two dwarves on one tile. It is red
   on `26185a0`, and the red is recorded. It also asserts the crew still works:
   - all 25 channel marks are cleared by tick **2,500**;
   - a stone is on the pile by tick **600**;
   - dwarf moves are at least **600**.
2. Spawn places the five dwarves on distinct tiles for every seed in `0..64`. This guard is green
   on `26185a0`.
3. **Head-on, both orientations.** Two miners meet head-on in a one-wide dead-end tunnel that opens
   onto open ground at one end. Each holds a job whose work position lies past the other. There are
   two fixtures:
   - **(a)** the lower id is nearer the open end;
   - **(b)** the lower id is nearer the dead end, and the higher id's work position is the tunnel's
     last cell. The lower then has no escape, and the other must back off.

   In each fixture, both reach their work positions and complete their jobs within a tick bound the
   test names. No tick has them on one tile, and neither job is released.
4. **Settle.** A dwarf falls onto a tile another dwarf stands on (overhang dug over a cave). It
   comes to rest on a free standable tile within 3 ticks of the support's removal. No tick has two
   dwarves on one tile.
5. **Dig-face access (amended at Task 0, Q1(b)).** An idle dwarf stands on the only work position
   of a dig at the end of a one-wide dead-end tunnel. A miner holds that dig and stands in the
   tunnel between the idle dwarf and the open end. The miner backs out, and the idle dwarf follows
   it out. The dig then completes within a tick bound the test names. The test asserts:
   - no tick has two dwarves on one tile;
   - **no tick has two dwarves swap tiles** (A from `a` to `b` while B goes from `b` to `a`);
   - the idle dwarf ends on a tile off the miner's route, with `home` equal to that tile and no
     `Path`.
6. **Occupancy never makes a job unreachable at claim time.** A dwarf stands in the only passage to
   a reachable job. The job is still claimed after its reaction delay and completed. Its
   `retry_after` is never stamped.
7. **Save → load → tick N ≡ never-saved → tick N** holds for a world saved while a dwarf follows a
   path that a fresh A* from its position would not return (a detour or a back-off). The test
   asserts that precondition. A pre-12.9 save is refused, per Task 0 Q2.
8. These existing guards stay green, unchanged in intent:
   - the walking skeleton (`scenario.rs:1036`);
   - 12.3's `unreachable_digs_never_starve_a_reachable_one` (`scenario.rs:723`) and
     `a_channel_worker_whose_support_is_removed_lets_go_and_the_crew_goes_on` (`scenario.rs:845`);
   - #74's `a_dwarf_never_stands_in_a_fire` (`lib.rs:4707`);
   - `save_load_then_tick_matches_never_saved` (`save_load.rs:10`).

   Any pinned figure that moves is re-pinned, and the move is disclosed in the commit and in
   Completion Notes.
9. The live instrument (Task 6) reads `OCCUPANCY OK` on the fixed build and `OCCUPANCY RED` with a
   mutation applied. It also reads `stone entries 0` on the fixed build. #133 and #162 close with
   AC1's and AC10's red-then-green records.

**Stones block (#162, ACs 10–12 added at Task 0).** "Item" below means an uncarried stone or log.
A carried item occupies no tile, which is unchanged.

10. **No dwarf enters a cell holding an item.** In AC1's busy run, no dwarf move from `a` to `b`
    between two ticks lands on a `b` that held an item at the end of the earlier tick. The test is
    red on `26185a0`, and the red is recorded. A fixture also places one item in a one-wide
    corridor that has a way round. A dwarf with a goal past the item takes the way round and never
    enters the item's cell.
11. **Pick up and drop from the next tile.** In a haul fixture:
    - at the pick-up tick, the hauler stands on a 4-neighbour (same z) of the item, never on it;
    - at the delivery tick, the hauler stands on a 4-neighbour of the pile cell the item lands on,
      never on that cell.
12. **The pile fills from the inside out.** A 3×3 pile with at least 9 reachable loose items fills
    all nine cells, the centre included, within a tick bound the test names. The test asserts the
    centre's fill tick is earlier than every edge cell's.

**From Wolf's seat pass 1 (2026-10-08, ACs 13–14 added):**

13. **Channel from the next tile.** A channel's work positions are the walkable same-z 4-neighbours of a standable
    target, like a dig's. The miner never stands on the cell it channels. The stone lands on the target cell.
    In the gui, a channelling miner faces the target cell.
14. **Pick-up and drop are drawn as a reach, not a jump** (gui only):
    - an item the wire has just put in a hauler's hands stays drawn on its own cell until the gui parents it, and it
      never slides along with the hauler's wire tile;
    - once parented it rises from its cell to the hands over a short lift, about 0.3 s;
    - when released it moves from the hands down to its new cell over a short set-down, about 0.3 s. It never
      appears on the cell in one frame.

## Tasks / Subtasks

- [x] **Task 0: Wolf rules** (record each answer in the Change Log). The recommended option is first.
  **RULED 2026-10-08:** Q1 **(b)**, over the rec. Q2: "old saves are not important", so **(a) refuse**.
  Q3 **(b) fold in**, and after the consequences were laid out: "taken pile cells should be
  impassable" too, fold confirmed over a new-story rec. Q4 **(a)**. Dev mode: Sonnet subagents.
  - **Q1.** In a dead end, an idle dwarf on the mover's next tile may have no free cell off the
    mover's path (AC5's case). (a) **The two swap tiles.** This is the only swap in the story, and
    the gui draws them passing once. (b) The mover backs out of the tunnel and the idle dwarf
    follows. That needs multi-step state for an idle dwarf.
  - **Q2.** Pre-12.9 saves have no `path`. (a) **Refuse them.** No `#[serde(default)]`, matching
    identity, profession and item kind. A 12.4–12.8 save can also hold two dwarves on one tile.
    (b) Default to an empty path.
  - **Q3.** #162, stones never block. (a) **Not folded.** It stays its own issue: a stone that
    blocks conflicts with picking up and dropping on a pile. (b) Fold it in.
  - **Q4.** Seat look after the gate. (a) **Yes, about 5 minutes.** #133 was reported at the
    seat, and back-off reads as feel. (b) No: scenario tests and the wire instrument only.
- [x] **Task 1: RED first (AC1, AC3, AC4, AC5).**
  - Add the AC1 test to `crates/sim-core/tests/scenario.rs`. Put the vacuity asserts (marks cleared,
    pile, moves) **last**, so a frozen-crew mutant dies on them and not on the occupancy assert.
  - Add the AC3 (a)/(b), AC4 and AC5 tests. These are fixtures, so `lib.rs` `mod tests` is fine;
    `stand_miners_at` (`lib.rs:3670`) and `insert_dig` (`:3686`) already exist.
  - Run them on unfixed code and paste each failing assertion into the Debug Log. Each fails on its
    shared-tile assert, because today the dwarves walk through each other.
  - Add the AC2 guard (green now).
- [x] **Task 2: occupancy in every writer.**
  - One helper answers "is this tile free of other dwarves", from a `BTreeSet<Pos>` of live dwarf
    positions. `execute_jobs`, `settle` and `wander` each build that set once and update it as each
    dwarf moves, in ascending `Id` order (AD-7).
  - `wander` drops occupied candidates. Replace the NOTE at `lib.rs:1339-1340`.
  - `settle`: if `below` holds a dwarf, the faller comes to rest on the nearest free walkable tile.
    Use a breadth-first search from `below` over `astar_neighbours`, in the shape of
    `release_claim`'s drop search (`lib.rs:975-997`). If no free tile exists, it stays put this tick
    (`// NOTE:`; 12.11 owns "no dwarf on air").
  - Add `debug_assert!` that no two dwarves share a tile, at the end of `World::step`. It turns
    every existing scenario test into an occupancy check.
- [x] **Task 3: the blocked step** (`execute_jobs`, the step branch `lib.rs:1095-1134`). The
  recommended shape is in Key decisions. When a holder's next tile holds another dwarf:
  1. **The blocker is idle** (`CurrentJob(None)`). It steps to a free walkable neighbour that is
     not on the holder's remaining path, in fixed `astar_neighbours` order. Its wander `cooldown`
     becomes `STEP_REST_TICKS` and its `home` becomes the new tile. The holder then steps.
     **If it has no such neighbour (Q1(b), no swap):** the idle dwarf gets an exit `Path`, which is
     an A* route to the nearest walkable tile off the holder's remaining path. The holder's tile
     counts as passable for that search, and every other dwarf blocks. Its first step is now the
     holder's tile, so this is case 3, head-on. The idle dwarf has no escape, because the holder
     blocks the only way out, so the holder yields and backs out. `wander` follows a dwarf's `Path`
     when it has one: at `STEP_REST_TICKS` pace, waiting when the next tile is occupied, with no
     RNG draw. When the path runs out, `wander` removes `Path` and sets `home` to the tile reached.
     `clear_paths` dropping the exit path is fine: the next blocked step re-derives it.
  2. **The blocker holds a job.** Re-route with A* to the holder's goals, with every other dwarf's
     tile added to `blocked`. If a path exists, store it and step.
  3. **Head-on.** The blocker's `Path` starts with the holder's tile. Compute each side's escape.
     The lower id yields if it has an escape; otherwise the other yields. The yielder's `Path`
     becomes its route to the escape, and it takes the first step.
  4. Otherwise wait. Hold position, set `JobState::Walk`, and set the wander cooldown to
     `STEP_REST_TICKS`, so a blocked dwarf searches once per step period and not every tick.
- [x] **Task 4: save the path (AC7).**
  - `SavedDwarf` gains `pub path: Vec<Pos>`, with no `#[serde(default)]` (per Q2). `to_save` writes
    it (`lib.rs:1501`), and `from_save` inserts `Path` when it is non-empty (`lib.rs:1593-1617`).
  - Fix the `SavedDwarf` literals at `save_load.rs:483,498` and `scenario.rs:1495,1510`.
  - Add the AC7 test to `crates/sim-core/tests/save_load.rs`. Compare `dwarves()`, `claims()` and
    `jobs()` on every tick. Check that `save_load_recomputes_every_path_invalidated_by_another_dig`
    (`save_load.rs:460`) still holds.
- [x] **Task 5: guards (AC6, AC8).**
  - Add the AC6 test.
  - Run `cargo test -p sim-core` and `cargo test -p simd`. Occupancy changes how many cells `wander`
    can choose from, so every later RNG draw moves.
  - Re-pin a moved figure only after confirming that the move is occupancy and not a defect. Say so
    beside the pin and in the commit.
- [x] **Task 10: stones block (#162, AC10–AC12). Do this after Task 5 and before Task 6.**
  - `blocked_cells` takes emitters **and every uncarried item's tile**, so A*, `wander`, claim
    reachability and the drop search all see items through the one rule (#74's shape). Every caller
    passes both. `execute_jobs` must not use a once-per-call set for items. A pick-up, a drop or a
    spawn changes them mid-loop, so it rebuilds them per dwarf (`uncarried_stones` already is).
  - `work_positions`:
    - Dig and Cut: filter neighbours with `is_walkable`, not `is_standable`.
    - Haul pick-up: the walkable same-z 4-neighbours of the item's tile. The item's tile must still
      be standable, as today.
    - Haul delivery: the walkable same-z 4-neighbours of the **deepest** free pile cells. Depth is
      the breadth-first layer from the walkable non-pile cells around the zone. Pile cells it never
      reaches are not delivery targets (`// NOTE:`).
  - Delivery drops the item onto the deepest free pile cell 4-adjacent to the hauler, lowest `Pos`
    on a tie. This is the same rule `work_positions` used, from one helper, so the two cannot
    disagree. `release_claim` keeps its abnormal-exit drop at the dwarf's own tile. Add a `// NOTE:`
    that the dwarf stands in the item until it walks off, which is rare.
  - Spawn sites are unchanged: a dig's stone goes in the dug cell, logs at the trunk base, and a
    channel's stone on the target. A channel miner stands in its own stone until it walks off
    (`// NOTE:`; Wolf accepted). A one-wide tunnel waits at each stone until it is hauled, and
    without a pile it stops (`// NOTE:`, FR8 never-drop).
  - Write the AC10 occupancy-of-items assert into the AC1 test, before the vacuity asserts. Write
    the AC10 corridor fixture and the AC11 and AC12 fixtures RED first, and paste the failing asserts.
  - Expect re-pins from the old rule, such as
    `haul_work_positions_gate_both_legs_on_a_free_standable_pile_tile` and the release-claim drop
    tests. Each intended change is disclosed beside the pin and in the commit (Task 5's rule).
  - If AC1's bounds (2,500 / 600 / 600) stop holding because of blocking items, **STOP and report**
    with the measured figures. Do not loosen them.
- [x] **Task 6: instrument (AC9).** `12-9-signoff/occupancy_wire.py` (NEW at creation; RED recorded
  in Verification).
  - It reads every delta from a fresh daemon running the busy-crew orders at fast4x. It counts
    shared ticks, and it range-checks five dwarves per delta, more than 200 moves, marks cleared and
    a stone on the pile. **Also count `stone entries`** (AC10 on the wire): a dwarf whose tile changed
    onto a tile that held an uncarried item in the previous delta (the wire's `carrying` says which
    items are held). Green needs `stone entries 0`; record the 26185a0 figure as its RED.
  - Run it three times, and paste all three outputs into the Debug Log:
    - GREEN on the fixed build;
    - deliberate RED: apply 12-9.sh row 1 (`wander` ignores occupancy), rebuild a release `simd`, and
      expect `OCCUPANCY RED`; then restore;
    - instrument self-test: apply row 7 (no dwarf ever steps), rebuild, and expect exit 2,
      `CREW DID NOT WORK`. A frozen crew shares nothing, and the instrument must not call that
      green.
- [x] **Task 7: mutations.** Create `_bmad-output/implementation-artifacts/mutations/12-9.sh` (NEW).
  Run it alone, after commit, with `RUST_TEST_THREADS=1 scripts/mutate.sh`. Read the kill line for
  every row.
  1. `wander` ignores occupancy → AC1.
  2. The job step ignores occupancy → AC1, AC3.
  3. `settle` ignores occupancy → AC4.
  4. The per-step flip rule: the lower id yields whenever it has any free neighbour → AC3(b) times
     out (the livelock).
  5. The higher id never yields → AC3(b).
  6. An idle blocker never makes way (the holder waits) → AC5.
  7. No dwarf ever steps → AC1's vacuity asserts. This is also the instrument self-test.
  8. `to_save` drops `path` → AC7.
  9. Occupied tiles join claim-time `blocked` in `claim_jobs` → AC6.
  10. An idle blocker with no free neighbour swaps tiles with the holder (the old Q1(a)) → AC5's
      no-swap assert.
  11. `blocked_cells` ignores items → AC10.
  12. Haul pick-up's work position is the item's own tile (the old rule) → AC11.
  13. Delivery picks the shallowest free pile cell → AC12.
  14. Delivery drops at the hauler's own tile → AC11.
  15. Channel works from the target itself (the old rule) → AC13 (sim).
  16. `dig_yaw` ignores Channel → AC13 (gui).
  17. `blend_entities` moves a carried, unparented item → AC14 hold.
  18. The lift is instant (`LIFT_SECONDS` to 0, or the jump restored) → AC14 rise.
  19. Release snaps to the cell → AC14 set-down.

  `74-dwarves-path-through-fire.sh` row 2 anchors on `wander`'s `is_walkable` line, so re-point it
  if Task 2 edits that line. `scripts/audit-mutations.py` reports rot.
- [x] **Task 11: channel from the next tile (AC13).** `work_positions` Channel uses the same rule as Dig and Cut
  (`side_neighbours(target)` filtered by `is_walkable`), still only for a standable target. The stone still spawns
  at `job.target`, which the miner no longer stands on. Remove the "channel miner stands in its own stone" NOTE.
  gui `dig_yaw` also takes `DwarfJob::Channel { target }`; fix its doc ("`None` for a channel"). RED first: a sim
  test that the channel holder works from a 4-neighbour and never stands on the target, and a gui test that a
  channelling miner gets the yaw toward the target. 12.3's
  `a_channel_worker_whose_support_is_removed_lets_go_and_the_crew_goes_on` (an AC8 guard) assumes the worker
  stands ON the target. Keep its intent: remove the support under the tile the worker actually stands on.
  Disclose the change.
- [x] **Task 12: lift and set-down (AC14, gui only, `crates/gui/src/project.rs`).**
  - `blend_entities` does not move an unparented item that the wire says is carried. It stays where it was
    drawn.
  - `sync_dwarf_work` parents it as today, then lifts it from its drawn position to `CARRY_OFFSET` over
    `LIFT_SECONDS`.
  - Release moves it from its drawn position to `item_translation(...)` over `LIFT_SECONDS`.
    `blend_entities` skips an item mid set-down.
  - One small presentation component holds the motion: start, end and elapsed. That is animation state, not
    game logic.
  - Pace it with the same frame time the walker uses. NOTE that a paused world still finishes a lift.
  - Tests are headless gui tests over a mirror:
    - the item holds its cell while the hauler walks in;
    - it rises, never in one frame, and arrives at `CARRY_OFFSET`;
    - set-down is never one frame, and it arrives at the cell.
  - Existing pick-up and drop gate tests stay green.
- [x] **Task 8: record.**
  - The PR body says `Closes #133` and `Closes #162`. Both issues are whole: #162 was folded in.
  - Comment on #162 with the ruling (all items block, pile cells too) and the AC10 RED/GREEN.
    Done: issuecomment-6066099669 (138 -> 0 stone entries).
  - Run the full gate, `RUST_TEST_THREADS=1 scripts/gate.sh`, and get it green before review.
    Done: `GATE GREEN 3281s` on `ac99c2d`. Only the story record has changed since.
- [x] **Task 9: seat look (per Q4).** Use a short vehicle card in `12-9-signoff/` in the canonical
  launch form. Wolf drags the channel and pile and watches the crew at Normal and at Fast: no two
  dwarves overlap, and blocked dwarves step aside or back off. Haulers pick up and drop from the
  next tile, nobody walks through a stone, and the pile fills from the centre. Record his words.
  **Pass 1 (2026-10-08), Wolf:** "when channeling dwarf channels directly under not next block" (→ Task 11);
  "sometimes dwarves are sucking stones from bit too far away and before starting to carry and throwing them,
  sometimes logs are pushed before dwarf instead of carrying (well it's kind of funny actually)" (→ Task 12);
  one-wide digging waits on the hauler, "maybe that's ok" (→ #180, idea); "performance variation is now big even
  when haze is off.. 20 - 180 FPS" (→ #179; the sim tick is ruled out there: release p99 34 us, max 10 ms).
  Pass 2 checks Tasks 11 and 12.
  **Pass 2 (2026-10-08, `7d69e19`), Wolf:** channel, "1 yes"; carry, "2 better.. dwarves are still sucking the stone
  not really picking up and also when dropping the stone slides.. but not going to tweak it now more" (-> **#181**,
  bug, look parked; AC14's headless tests hold (corrected at review run 1: this said "the sim side of AC14", and AC14 is
  gui-only), and the gui blend reads as a slide rather than a reach).

### Review Findings

Code review run 1, 2026-10-09, on `34b6783` (diff `origin/main...HEAD`, `26185a0..34b6783`, 20 files, +2,933/-185).
This was a fresh session. Four layers ran and none timed out. All ran cargo 1.97.1, each in its own `/tmp/review-<layer>`
target dir with `CARGO_BUILD_JOBS=6`.
- Blind Hunter (Sonnet) took sim-core `src` plus gui `project.rs`.
- Edge Case Hunter (Sonnet) took simd, gui `ingest.rs`, every `tests/`, the mutation tables and `occupancy_wire.py`.
- The Acceptance and Feature Auditors (Opus) took the whole diff.

| Layer | Findings | Severity | Ran |
| --- | --- | --- | --- |
| Blind Hunter | 5 | 2 MED, 3 LOW | sim-core suite; probes in a copy (delivery or channel onto a dwarf, chain deadlock, save round-trip at 20 cut ticks) |
| Edge Case Hunter | 5 | 5 LOW | sim-core, simd, five gui tests one at a time, `audit-mutations.py` (827 rows match), live instrument, row 8 in a copy (KILLED) |
| Acceptance Auditor | 11 | 2 MED, 9 LOW | every suite, fmt and clippy, REDs reproduced on 26185a0, instrument GREEN, row 1 RED, row 7 exit 2 |
| Feature Auditor | 6 | 2 HIGH, 4 LOW | 30 release runs; a multi-seed probe at HEAD and on base; live big-channel and trench runs; save, refused-save and bad-path live |

Convergences:
- stones landing on dwarves: blind + feature;
- unvalidated `path` on load: acceptance + feature;
- AC7 precondition: edge + acceptance;
- AC8 guard: edge + acceptance;
- tick cost: blind + feature.

The orchestrator reproduced D1, P1 and P2 itself. Probe sources were checked byte-identical to HEAD and `26185a0`.

The live wire stays green on the story recipe. Three independent runs read `shared 0, stone entries 0, OCCUPANCY OK / STONES
OK`. **That green does not cover D1:** the instrument counts dwarves moving onto items, not items landing on dwarves.
Not proven live: the AC13 facing and the AC14 lift/set-down look (seat; #181 parked) and the back-off feel (Task 9).

Three issues filed at discovery: **#182** (D1), **#183** (P1), **#184** (P2).

- [x] [Review][Patch] (decision resolved) **HIGH: stones land on dwarves; a caged dwarf never moves again and the channel never finishes (#182)**
  (feature + blind) — Since AC13 a channel target is no longer the miner's own tile. `execute_jobs` still spawns the stone
  at `job.target` (`lib.rs:1553`) with no occupancy check. Delivery's `drop_cell(pile_targets(..))` (`lib.rs:731-770,
  1467`) never consults dwarves either. A dwarf ends up standing inside a stone. If its other neighbours are stones, fire,
  trunk or air, `wander` has no candidate. Once the pile is full nobody hauls those stones, so the dwarf stays caged and its
  channel mark has no work position left.

  Multi-seed release probe, seeds 0..16, 4x7 and 6x10 channels, 4,000 ticks, 28 runs with a pile:

  | | HEAD | `26185a0` |
  | --- | --- | --- |
  | Runs with marks unfinished | 9/28 | 0/28 |
  | Runs with a dwarf frozen ≥2,251 ticks | 8/28 | 0/28 (longest still ≤60) |

  Live on DEFAULT_SEED with a 6x10 channel: stone 30 spawned under idle dwarf 0 at t1371, and the dwarf did not move
  again. Two sub-modes sit beside the cage:
  - (i) a delivery seals a dwarf in a dead end whose only exit is a pile cell (seed 13);
  - (ii) in 3 of the 9 runs, a mark is walled in by its own unhauled stones once the pile is full, with no dwarf caged.
    This is the #180 / FR8 never-drop shape already ruled for a one-wide tunnel.

  **RULED 2026-10-09 (Wolf): option 2, fix in 12.9's patch pass, now a patch:**
  - A delivery's `drop_cell` skips dwarf-occupied pile cells; the hauler holds and retries next tick. No claim-time change
    (AD-12).
  - A completing channel first steps any dwarf off its target with the case-1 sidestep. If it cannot, the miner holds at
    Work and retries.
  - **Prevent (i):** a delivery refuses a cell whose filling would wall a dwarf in. This is a connectivity check per drop.
  - `occupancy_wire.py` gains an `items landed on a dwarf` count, and green needs 0.
  - A scenario test over the probe's seeds asserts no dwarf freezes.
  - (ii) stays the ruled #180 / FR8 shape and gets a `// NOTE:`.

  **Landed** in `0b8322d` and `aa94a5d`. Wolf ruled option 1 (2026-10-09): chase the shapes, then fall back to landing what
  holds. As ruled, the fix was worse than HEAD on the sweep: 28 of 32 runs had a frozen dwarf. Two changes from the ruled
  form fixed that:
  - a refused worker or hauler LETS GO (`retry_claim`) instead of holding;
  - the wall-in check is "a dwarf, the filler included, is left in a piece smaller than the largest".

  The abnormal drop is guarded too. Results:
  - Sweep: 0 landings over 32 runs.
  - Frozen runs: 9 (as found), 5 after the fix. All 5 are one-level islands: wander is same-z only, A* still reaches the
    dwarf, and no stone is on it. They're pinned by name and filed as **#186**.
  - Live wire: GREEN, with landings 0. A deliberate RED (row 25) reads 6 landings.
  - Rows 23–28 KILLED.

  The "no dwarf freezes" sub-bullet is met except for #186.
- [x] [Review][Patch] **MED: head-on with no escape on either side waits forever when the idle blocker follows a stale exit
  `Path` (#183)** (blind) [crates/sim-core/src/lib.rs:1295] — Drop the idle blocker's `Path` in the `(None, None)` arm so
  the next blocked step re-derives it. RED: the probe's chain fixture (`done=None` after 40,000 ticks; mirrored layout 370).

  **Landed** in `c9311f1`, with a mechanism test (`a_head_on_with_no_escape_drops_the_idle_blockers_stale_path`) and row 22
  KILLED. The chain fixture still fails with the drop in place: it is a livelock, because tunnel-homed idle dwarves wander back
  and the miner (id 0) always yields. Wolf ruled option 1: land the drop and file the chain as **#185**.
- [x] [Review][Patch] **MED: a walled-in deepest free pile cell stops the whole pile (#184; this is record flag 2, and the
  record understated it)** (acceptance) [crates/sim-core/src/lib.rs:731] — Layer depth through free cells only. RED: centre
  free, four edge-middles taken, one loose stone gives `delivered=None, retry_after 3118`; the control delivers at t312. Also
  correct the #162 comment's "never walled out of the empty middle".

  **Landed** in `6dbdafe`. `a_walled_in_free_cell_does_not_stop_the_pile` (delivers at t312), row 20 KILLED, and four old
  rows re-pointed (all KILLED). The #162 comment was edited in place (Wolf: edit).
- [x] [Review][Patch] **MED: the new saved `SavedDwarf.path` is not validated on load** (acceptance + feature)
  [crates/simd/src/main.rs:344] — `load_world_from` range-checks every other saved position, but not `path`. A save with
  `path=[(9000,9000,9000)]` loads, and the dwarf stands off the map, on the wire. `wander` follows a path with no adjacency
  check (`lib.rs:1707`). Bounds-check every path tile, the same way `pos` and `home` are checked.

  **Landed** in `0f8a5b2`. RED first; an in-bounds control loads. Row 21 KILLED.
- [x] [Review][Patch] **LOW (silent-failure exception): `occupancy_wire.py` reports a malformed delta as RED** (edge)
  [_bmad-output/implementation-artifacts/12-9-signoff/occupancy_wire.py:111] — Empty `entities` makes `max()` raise, and a
  missing `items` or `designations` raises `KeyError`. The traceback exits 1, the same code as RED; it should exit 2, RUN
  PROVES NOTHING.
- [x] [Review][Patch] **LOW (silent-failure exception): `occupancy_wire.py` accepts tick gaps** (edge)
  [_bmad-output/implementation-artifacts/12-9-signoff/occupancy_wire.py:135] — Only `limit // 2` ticks are required, and gaps
  after the first delta are never counted, so a shared tick between two read deltas is invisible. Count the gaps after the
  first tick, print them, and treat any gap as RUN PROVES NOTHING. Runs read 1,479 and 1,435 of 1,500 ticks.

  **Both LOWs landed** in `f5cfd32`, together with the `items landed on a dwarf` count. A fake-daemon self-test covered
  empty entities, a missing `items` key and a skipped tick: each exits 2.
- [ ] [Review][Patch] **LOW: record corrections** (acceptance) — AC8 wants each re-pin disclosed in the commit, and
  `5b17f72`, `d93f32d` and `0fccc74` are subject-only. Disclose them in the patch commit body and the PR body. Correct
  these record lines:
  - flag 4 is false: the AC6 fixture passes at cooldown 0 and 10, and only 10,000 stalls;
  - the AC10 sim RED on `26185a0` was never run there. It is `shared=332 item_entries=329`, and the occupancy assert fires
    first;
  - AC1 at HEAD reads 1,223 / 184 / 1,199, not the Task 10 figures;
  - the #74 anchor is row **3**, not row 2;
  - "the sim side of AC14" should read "AC14's headless tests"; AC14 is gui-only.

  **Record lines corrected, and the re-pins disclosed in the `48ac6b0` commit body. Left unchecked:** the PR body
  disclosure waits for the PR.
- [x] [Review][Defer] AC10 can break within one tick: a hauler lifts a stone in `execute_jobs` and `wander` moves an idle
  dwarf onto that cell in the same tick (seed 5, t834). The gui draws it walking into the stone (feature)
  [crates/sim-core/src/lib.rs:1664] — deferred, 1/28 runs, cosmetic
- [x] [Review][Defer] Job holders now stall up to ~65 ticks behind a blocker (≤11 at base). An exit `Path` waits on the
  wander cooldown (feature; record flag 4) [crates/sim-core/src/lib.rs:1695] — deferred, reads as hesitation, not stranding
- [x] [Review][Defer] Tick-cost spikes: max 9–17.6 ms in 8/28 runs vs ≤0.41 ms at base. p99 30–156 µs, inside NFR2's fast4x
  budget (feature + blind) [crates/sim-core/src/lib.rs:1204] — deferred, within budget; relevant to #179
- [x] [Review][Defer] A sidestep or yield can move a blocker a second cell in the tick it already moved (blind)
  [crates/sim-core/src/lib.rs:1240] — deferred, unverified one-tick jump
- [x] [Review][Defer] gui lift takes `from` as world space when the item is still another dwarf's child (two-carrier hand-off
  in one frame) (blind) [crates/gui/src/project.rs] — deferred, rare, unverified
- [x] [Review][Defer] The `serve.rs` channel-from-the-next-tile assert may never run: the test returns before requiring a
  channel Work delta (edge) [crates/simd/tests/serve.rs:646] — deferred, the sim test covers AC13
- [x] [Review][Defer] The AC8 channel guard passes only through its Ramp branch now; another miner finishes the channel
  (edge + acceptance, flag 1) [crates/sim-core/tests/scenario.rs] — deferred, intent holds
- [x] [Review][Defer] The AC7 precondition is asserted indirectly (three west steps), not as `!path.is_empty()` at save (edge +
  acceptance) [crates/sim-core/tests/save_load.rs] — deferred, row 8 kills it
- [x] [Review][Defer] The campfire re-pin comment says full by ~2,250; measured 2,149 (acceptance)
  [crates/sim-core/tests/scenario.rs] — deferred, the bound holds
- [x] [Review][Defer] AC1's "stone on the pile" counts a carried stone: 184 carried vs 201 loose (acceptance)
  [crates/sim-core/tests/scenario.rs] — deferred, both far inside 600
- [x] [Review][Defer] `dwarf_tiles` spells `.cloned()` to dodge #74 row 3's anchor, uncommented (acceptance, flag 3)
  [crates/sim-core/src/lib.rs] — deferred, a cleanup would show as BROKEN in the audit, loudly
- [x] [Review][Defer] Five 3-3 and 12-1 mutation rows were already dead on `26185a0`; two spot-checked SURVIVED there
  (acceptance, flag 5) [_bmad-output/implementation-artifacts/mutations/] — deferred, pre-existing

#### Review run 2 (2026-10-09, diff `2a10797..02e313c`, the run-1 patch pass)

This run used a fresh session. Diff: 11 files, +736/−72 (code: 3 files, +490/−33). Four layers ran and none timed out. Each ran
cargo 1.97.1 in its own `/tmp/review-<layer>` target dir with `CARGO_BUILD_JOBS=6`.
- Blind Hunter (Sonnet) took sim-core `src`.
- Edge Case Hunter (Sonnet) took simd, `scenario.rs`, `occupancy_wire.py` and the mutation tables.
- The Acceptance and Feature Auditors (Opus) took the whole diff.

**Delta against run 1:**
- **NEW:** 17 findings: 1 HIGH, 3 MED, 13 LOW. 3 were dismissed and 14 survive: 2 decisions, 5 patches and 7 deferred.
- **REWORK:** #182 is HALF-CLOSED.
  - 0 landings on another dwarf: verified, RED on old.
  - But a refused completion now loops forever (D1).
  - A let-go drops its stone under the hauler, and the instrument would call that RED (D2).
- **Closed:** #183, #184, the path bounds-check and both instrument LOWs. Each new test is RED on the pre-patch code (the acceptance
  auditor ran each against `2a10797` with only the test hunks applied).
- **Stopping rule:** there is a HIGH among the new findings, so another round is authorised after the patches.

| Layer | Findings | Severity (layer's own) | Ran |
| --- | --- | --- | --- |
| Blind Hunter | 6 | 1 MED, 2 LOW-MED, 3 LOW | sim-core suite; a `release_claim` own-tile probe in a copy |
| Edge Case Hunter | 8 | 3 MED, 5 LOW | scenario + simd suites; live wire (0 landings); 10 fake-daemon inputs; every row 20-28 and the re-pointed rows in a copy |
| Acceptance Auditor | 9 (3 positive) | 2 MED, 4 LOW | all suites, fmt, clippy; 5 new tests RED on `2a10797`; AC1/AC10 on `26185a0`; live wire 1,500 and 3,000 ticks |
| Feature Auditor | 4 | 1 HIGH, 1 MED, 2 LOW | 60-run release probe on HEAD, `2a10797` and `26185a0`; an instrumented trace; live wire 1,500 and 4,000 ticks |

Convergences:
- the own-tile let-go drop: all four layers;
- tick cost: blind + acceptance + feature;
- the sweep pin hides freezes: edge + acceptance;
- the sidestep moves a working dwarf, and work positions ignore refusal: blind + feature (folded into D1).

The orchestrator read the Feature Auditor's per-seed rows across all three revisions and confirmed D1's 4x7 regression.

Filed at discovery: **#187** (D1) and **#188** (campfire).

Not proven live:
- AC13 facing and the AC14 lift/set-down (seat);
- the back-off feel;
- how a sidestep, a let-go and the D1 loop look;
- a stone on the campfire.

The live wire is GREEN on the default recipe, which never lets go: 0 own-tile drops at 1,500 and 4,000 ticks.

Review cost: $16.16 over 316 turns; the 4 subagents were 73.6% of tokens. The run reaped 20.0 GB of `/tmp` caches (12.4 GB
reclaimed).

- [ ] [Review][Patch] (decision resolved) **HIGH: a refused channel completion loops forever; the let-go changes nothing (#187)** (feature;
  blind 4 and 6 fold in) [crates/sim-core/src/lib.rs:1652-1670] — The worker returns to the same refusal every ~116-144 ticks
  until the run ends. A mark never finishes. The dwarves keep moving, so the sweep's `STILL_BOUND` stays green. Three shapes:
  - (a) no aside tile: an idle dwarf homed on the target in a dead end whose only exit is the miner's work tile (seeds 21/6,
    2/6, 19/6);
  - (b) `step_aside` takes the FIRST free neighbour, a one-tile pocket the wall-in check then refuses (seed 5/4: 3 marks left,
    base 0). It also shoves a dwarf that is working its own job;
  - (c) the miner's work tile is a pocket only the target opens. `release_claim` re-homes the miner there and
    `work_positions` picks it again, although (68,67) is valid (seed 22/6; seed 29/4 finishes at t2765 vs 1,253 on base).
  
  | Channel size | HEAD | `2a10797` | base |
  | --- | --- | --- | --- |
  | 4x7 runs unfinished, seeds 0..16 | 5/16 | 3/16 | 0/16 |
  | 4x7 mean finish | 1,351 | 1,293 | 1,173 |
  
  Landings on another dwarf: 0 / 330 / 1,200, so HEAD still beats `2a10797`.

  **RULED 2026-10-09 (Wolf): option 1, fix all three shapes in 12.9:**
  - (b) `step_aside` picks the aside tile WITH the wall-in check, not the first free neighbour.
  - (c) `work_positions` skips a work tile the fill would seal.
  - (a) when the occupant has no tile aside, it leaves by an exit `Path` instead of the miner letting go.
  - The sweep pins each run's marks left, so a loop goes red.
  - Same time-box as #182: land what holds, keep the rest on #187.
- [ ] [Review][Patch] (decision resolved) **MED: a let-go drops its stone under the hauler, and the instrument and the sweep disagree on whether
  that is a landing** (blind + edge + acceptance + feature) [crates/sim-core/src/lib.rs:1160-1168;
  12-9-signoff/occupancy_wire.py:133] — `release_claim`'s `refused` exempts the carrier's own tile, so a let-go or abnormal
  drop lands at the hauler's feet.
  - The sweep exempts that (`carriers_before`); `occupancy_wire.py` counts it, so it reads STONES RED.
  - Probe: 51 own-tile drops in 42 of 60 runs; 32 over the sweep's 32 runs.
  - The default wire recipe never lets go, which is the only reason the wire is green.
  - Related (blind 2): `walls_in_a_dwarf` never tests a dwarf standing ON the cell, so a carrier can seal itself in with its
    own drop by stepping into the smaller side.

  **RULED 2026-10-09 (Wolf): option 2, never drop under any dwarf.**
  - `release_claim`'s `refused` stops exempting the carrier's own tile. A let-go or abnormal drop sets the stone on the nearest
    reachable tile that is allowed: not a taken pile cell, not under any dwarf, and walls nobody in. The carrier is included.
  - `occupancy_wire.py` stays strict.
  - The sweep's `carriers_before` exemption goes.
  - The `release_claim` NOTE ("the dwarf stands in the item until it walks off") is replaced.
  - Tests pinning an own-tile drop are re-pinned, and each re-pin is disclosed in the commit body.
- [ ] [Review][Patch] **MED: the per-drop wall-in check made tick cost materially worse** (feature + blind + acceptance)
  [crates/sim-core/src/lib.rs:765-776, 1160-1191]
  - `drop_cell` runs `refused` before the target filter, so `walls_in_a_dwarf` runs on all four neighbours of the hauler, pile
    or not (50-109 calls against ~10 deliveries).
  - Each call clones `blocked` and floods up to 50,000 tiles.
  - The 60-run probe: 37 runs with a tick over 10 ms (`2a10797`: 15); max 25.0 ms, the whole fast4x budget. Run 1's deferral
    measured 9-17.6 ms in 8/28.
  - Fix: filter to targets first, then re-measure max and p99 on the probe. If the max still exceeds the budget, it rejoins
    the tick-cost deferral and #179 with the number.
- [ ] [Review][Patch] **MED: the #182 sweep cannot see the defects it exists for** (edge + acceptance + feature)
  [crates/sim-core/tests/scenario.rs:3020-3045]
  - `max_by_key` keeps only the longest-still dwarf per run. Seed 12/6 has two frozen dwarves (0 and 4) and dwarf 0 is
    invisible, so a new cage in a pinned run is hidden.
  - No assert covers marks left, so D1's loops pass.
  - Fix: pin every dwarf over `STILL_BOUND`, and pin each run's marks left (after D1's ruling), so a new loop goes red.
- [ ] [Review][Patch] **LOW (process): the run-1 patch pass emitted no per-item closure table** (acceptance)
  [12-9-one-dwarf-per-tile.md, Review Findings] — The substance holds: every new test is RED on `2a10797` (acceptance, RAN). Write
  the table, one row per item: side written for, side tested, and the named fixture or row.
- [ ] [Review][Patch] **LOW: the `48ac6b0` re-pin disclosure is incomplete** (acceptance) — Missing:
  - `5b17f72`'s `lib.rs` pin moves: `release_claim_drops_where_the_carrier_can_walk` `pocket[2]`→`pocket[1]`, the settle
    landing, and the six old-rule tests;
  - `d93f32d`'s two channel unit-test targets and the `headless.rs` step label.
  
  Fold them into the pending PR-body disclosure.
- [ ] [Review][Patch] **LOW (silent-failure exception): `occupancy_wire.py` exits 1 (RED) on a connection reset** (edge)
  [12-9-signoff/occupancy_wire.py, :37-38, socket loop] — An uncaught `ConnectionResetError` gives a traceback and exit 1. Treat it as
  RUN PROVES NOTHING, exit 2, as run 1 did for malformed deltas.
- [x] [Review][Defer] A saved `path` tile is bounds-checked but not checked for adjacency or walkability; `wander` can step a
  dwarf into rock from a hand-edited save (edge) [crates/simd/src/main.rs:604] — deferred, matches the ruling (as `pos`/`home`);
  corrupted saves only
- [x] [Review][Defer] The sweep's "crew worked" check is satisfiable by a channel stone on an overlapping pile cell (edge)
  [crates/sim-core/tests/scenario.rs:~439] — deferred, vacuity guard only
- [x] [Review][Defer] Any exit 2 masks a RED in `occupancy_wire.py` (the counts still print) (edge) — deferred, by design
- [x] [Review][Defer] `occupancy_wire.py` cannot see an item under a dwarf in the first delta or from tick 0 (edge) —
  deferred, no writer does that
- [x] [Review][Defer] `walls_in_a_dwarf` treats equal-sized pieces as no wall-in (`< largest`), so two dwarves in two equal
  pockets can both be sealed (blind) [crates/sim-core/src/lib.rs:812] — deferred, unobserved
- [x] [Review][Defer] The abnormal-drop BFS has no node bound and calls `walls_in_a_dwarf` per cell (blind + acceptance)
  [crates/sim-core/src/lib.rs:1163-1188] — deferred, the "nowhere" fallback fired 0 times in 60 runs; revisit with the tick-cost patch
- [x] [Review][Defer] A channel mark on the campfire tile now completes: stone on the camp tile, fire over a ramp (feature)
  — deferred to **#188** (AC13, round-1 code; needs a ruling)

Dismissed (3):
- #186's framing undersells it. The issue's own table lists the stones, and Wolf ruled to file it.
- `retry_after` stamped at completion is the recorded let-go deviation.
- The 12-1 `retry drop stacks a full stockpile` row SURVIVES. That is already deferred in run 1, pre-existing.

## Dev Notes

### Scope guardrails (do NOT)

- Do not change `claim_jobs`, its filter, or claim-time reachability (AD-12). Occupancy is a
  step-time rule. A dwarf in a doorway at claim time would otherwise stamp a 20-tick cooldown, and
  `components` would record a false split (12.3). AC6 and mutation 9 pin this.
- Do not put other dwarves into `is_walkable` or `blocked_cells` globally. Those are the terrain
  and fire rule (#74), and both A* and claim reachability read them. Add dwarf tiles only to the
  blocked set of the step-time re-route.
- No new pathfinder, cost function, reservation table or cross-tick cache (AD-5). Plain A* with an
  enlarged `blocked` set is the re-route.
- No new component or resource holding wait counters or "yielding" flags. The yield lives in the
  saved `Path`. If a design needs more, it goes into `SaveState` with a test, and the reason goes in
  Completion Notes.
- No wire, `protocol`, `client-core`, `tui` or `gui` change. The tui crowd glyph
  (`tui/src/view.rs:345-360`, `palette.rs:181`) stays: it is client rendering of whatever the wire
  says.
- **Items DO block (#162, folded at Q3), dwarves do NOT join `blocked_cells`.** An uncarried item
  is world state, static within a system call, so it belongs in the terrain-and-fire rule. A dwarf
  moves within the call, so it does not. Falling items and "no dwarf on air" stay in 12.11.
- Before seat pass 1: no gui change. Its pick-up and drop gates key on the hauler's own wire cell
  (`gui/src/project.rs:520-530`), which still holds when the item is one tile away. **The seat
  disproved the look:** the items jump a cell. Task 12 (Wolf, 2026-10-08) is the one gui change, and it is
  presentation only. Task 11 adds one `dig_yaw` arm. No other client change.

### What already exists (build on it)

- `is_walkable` and `blocked_cells` (`lib.rs:683-698`) are the one terrain-and-fire rule for every
  writer. #74's `a_dwarf_never_stands_in_a_fire` (`lib.rs:4707`) tests the running world, not one
  writer. Follow that shape for AC1.
- `astar(terrain, blocked, from, goals)` (`lib.rs:802`) already takes a blocked set. Pass
  `emitters ∪ other dwarves` for the re-route.
- `release_claim`'s drop search (`lib.rs:975-997`) is the breadth-first "nearest free cell" shape
  for `settle`. Its home reset (`lib.rs:1024-1027`) is why a displaced idle dwarf's `home` must
  move too.
- Step pacing: `Wander.cooldown` paces both walkers (`lib.rs:1108-1129`), and `STEP_REST_TICKS`
  (`:42`) is the step period, 11 ticks per cell.
- The gui's walk phase advances by ground covered (`gui/src/project.rs:390-401`), so a waiting
  dwarf freezes mid-stride rather than walking on the spot. No gui change is needed.

### Key decisions & traps

- **Escape.** A dwarf's escape is the nearest tile reached by a breadth-first search over walkable
  tiles, with every other dwarf's tile blocked, that is not on the other dwarf's remaining `Path`.
  - Escapes are what keep the yield stable. As the yielder retreats, its escape gets nearer, and the
    other dwarf never gains one by advancing into tiles on the yielder's path. That is why fixture
    (b) resolves rather than flipping.
  - Bound the search at `MAX_ASTAR_NODES`.
  - If neither dwarf has an escape (both sealed in one pocket), both wait. Leave a `// NOTE:`.
- **The yield is the path.** A yielder's `Path` becomes its escape route.
  - The other dwarf then sees a non-head-on job holder ahead of it, so it waits or follows.
  - When the yielder arrives, its path is empty, so `execute_jobs` recomputes A* to its goals
    (`lib.rs:1100-1106`) and it resumes.
  - `clear_paths` on any dig re-derives all of this from saved state, so no flag is needed.
- **Process order is ascending `Id`**, and each writer updates its occupied set as dwarves move. A
  blocker processed later in the same tick is judged by its previous-tick `Path`. That is
  deterministic and save-exact, because `Path` is saved.
- **Never strand a displaced idle dwarf.** `wander` only accepts tiles within `WANDER_RADIUS` of
  `home`. If a dwarf is pushed 5 or more tiles out, it never moves again (`release_claim` NOTE,
  `lib.rs:1014-1019`). So moving `home` with it is required.
- **A displaced idle dwarf must not also wander in the same tick.** Its cooldown is reset, or
  `wander` (which runs after) moves it a second cell, and it is drawn jumping.
- **Re-pins are expected and must be disclosed.** Occupancy changes `wander`'s candidate count, so
  the RNG draws, and with them every idle trajectory after the first avoided collision, change.
  9.4's review graded "the record said a pin was not at risk when it had moved" as HIGH.
- **Tick cost (NFR2).** Re-routes and escapes run only on a blocked step, at most once per step
  period per dwarf. That is at most 5 × `MAX_ASTAR_NODES` a tick, the same order as `claim_jobs`.
  12.3 measured ~0.16 s/tick (debug) at a full burn. Keep AC1's loop bounded by its tick numbers.
- **Put the vacuity asserts last** (12.3 trap 1). A mutant that freezes the crew must die on "the
  crew still works", not on the occupancy assert it trivially passes.

### Verification (recipe; the RED half was run at creation on `26185a0`)

```bash
cargo build -q --release -p simd
./target/release/simd 7491 >/dev/null 2>&1 &            # FRESH daemon, DEFAULT_SEED (camp [64,64,9])
sleep 2
python3 _bmad-output/implementation-artifacts/12-9-signoff/occupancy_wire.py 7491 1500; echo "exit $?"
pkill -x simd
#   RED (observed, 26185a0, 2026-10-08):
#     camp [64, 64, 9] channel [65, 61, 9]..[68, 67, 9] pile [59, 64, 9]..[61, 66, 9]
#     ticks read 1479  deltas without exactly 5 dwarves 0
#     shared ticks 186  max dwarves on one tile 2  first shared (101, {(65, 62, 9): [1, 3]})
#     dwarf moves 551  channel marks 25 -> 0  items on the pile 9
#     OCCUPANCY RED                                   exit 1
#   GREEN (required): shared ticks 0, max dwarves on one tile 1, moves >= 200, marks -> 0,
#     items on the pile > 0, OCCUPANCY OK, exit 0
#   Deliberate RED (required): 12-9.sh row 1 applied, release simd rebuilt -> OCCUPANCY RED, exit 1
#   Instrument self-test (required): row 7 applied -> CREW DID NOT WORK, exit 2
#   Review patch pass (0b8322d/aa94a5d, 2026-10-09). The instrument now also counts items landing on a dwarf (#182):
#     GREEN: ticks read 1479, shared 0, moves 433, marks 25 -> 0, items on the pile 8, stone entries 0,
#       items landed on a dwarf 0, tick gaps 0, OCCUPANCY OK, STONES OK, exit 0
#     Before the fix (6dbdafe): items landed on a dwarf 8, first (192, {12: (65, 62, 9)}), STONES RED, exit 1
#     Deliberate RED (required): 12-9.sh row 25 applied, release simd rebuilt -> items landed on a dwarf 6,
#       first (192, {12: (65, 62, 9)}), STONES RED, exit 1
#   GREEN now also requires items landed on a dwarf 0 and tick gaps 0. A malformed delta or any tick gap exits 2.
```

Restart the daemon before every run, because the recipe is pinned to a fresh world. Exit 0 is not a
result; the `shared ticks` and `dwarf moves` lines are. `RUN PROVES NOTHING` (exit 2) means too few
ticks were read, or a delta lacked five dwarves. Fix the run before reading anything else.

### Project Structure Notes

- `crates/sim-core/src/lib.rs`: UPDATE. Changes: `execute_jobs` blocked step; `settle`; `wander`;
  the occupancy helper, escape and re-route; the `World::step` debug assert; `to_save`/`from_save`
  path. Tests for AC3–AC6.
- `crates/sim-core/src/save.rs`: UPDATE (`SavedDwarf.path`). No new saved state for #162:
  item positions are already saved, and blocking is derived from them.
- `crates/sim-core/tests/scenario.rs`: UPDATE (AC1, AC2; `SavedDwarf` literals)
- `crates/sim-core/tests/save_load.rs`: UPDATE (AC7; `SavedDwarf` literals)
- `_bmad-output/implementation-artifacts/mutations/12-9.sh`: NEW.
  `mutations/74-dwarves-path-through-fire.sh`: UPDATE if re-pointed.
- `_bmad-output/implementation-artifacts/12-9-signoff/occupancy_wire.py`: NEW (at creation).
  `12-9-signoff/vehicle-card.md`: NEW, per Q4.

### Previous story intelligence

- 12.3: `claim_jobs` keeps a per-call list of flooded components (`lib.rs:494`), and occupancy at
  claim time would poison it. Keep the step and the claim apart. Its AC tests were slow while red,
  so keep tick loops bounded by the AC numbers.
- 12.8 (#164): the gui gates a pick-up on the drawn body reaching its wire tile. A dwarf that waits
  keeps its wire tile, so nothing changes there. The Q1 swap is the only case that draws two bodies
  crossing.
- The full gate is green only at `RUST_TEST_THREADS=1` (about 55 min). Run `mutate.sh` alone.

### References

- `epics.md` Epic 12, Story 12.9 and the order rules ("#132 is fixed before #133"). M3 deltas:
  "AD-5 (plain A\*) stands for FR49"; AD-7 "FR49's tie-break is by entity id".
- PRD `prd-frostvein-2026-09-28/prd.md:111` (FR49), and `reconcile-inputs.md` §1.1 (the tie-break
  gap and the three edge cases). `review-rubric.md:30` covers the NFR2 tick-cost watch.
- Issues #133 and #162. Issue #74 and `mutations/74-dwarves-path-through-fire.sh` give the
  one-rule-every-writer shape.
- Story 12.3 (`12-3-no-dwarf-stuck-after-digging.md`): components, budgets and the bounded-test
  discipline.

## Dev Agent Record

### Agent Model Used

Sonnet 5.5 subagents x3, one at a time on one tree (A: Tasks 1-3, B: Tasks 4-5, C: Task 10); Opus 5.5 orchestrator
(Tasks 0, 6, 7, 9; verified each phase, wrote the mutation table and ran every mutation and instrument run).

### Debug Log References

**Task 1 REDs, run on the unfixed movement code (Task 2/3 not yet written), `cargo test -p sim-core`:**

- AC1 `a_busy_crew_never_shares_a_tile_and_still_works` (`scenario.rs`):
  `332 ticks had two dwarves on one tile; first Some((2, [(62,65,9), (65,63,9), (61,66,9), (65,63,9), (66,62,9)]))`.
  332 is the story's baseline. 1.2 s in debug.
- AC2 `spawn_places_five_dwarves_on_distinct_tiles_for_every_small_seed`: green on 26185a0, as required.
- AC3(a) `head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_open_end` (`lib.rs`):
  `two dwarves share a tile at tick 102: [(13,20,5), (13,20,5), ...]`.
- AC3(b) `head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_dead_end`:
  `two dwarves share a tile at tick 102: [(13,20,5), (13,20,5), ...]`.
- AC4 `a_dwarf_that_falls_onto_another_comes_to_rest_on_a_free_tile`:
  `two dwarves share a tile at tick 2: [(10,10,2), (10,10,2), ...]`.
- AC5 `an_idle_dwarf_on_the_dig_face_follows_the_miner_out_and_the_dig_completes`:
  `two dwarves share a tile at tick 123: [(16,20,5), (16,20,5), ...]`.

All five fixtures fail on their shared-tile assert (the `watch` helper), not on a later one.

**Task 2 alone (wait when the next tile is occupied, no yielding), measured before Task 3:** AC3(a), AC3(b)
and AC5 timed out (`not done by tick 1000/1200`) and 8 existing scenario tests stalled (walking skeleton,
every haul test): a plain wait deadlocks in any one-wide passage and on any idle dwarf standing on a work
position. That is why Task 2 and Task 3 are one commit.

**AC3(b) resolves, it does not flip.** Trace of the fixture (lower id 0 at x=14, higher id 1 at x=12, tunnel
x 11..16, room x<=10): tick 102 head-on at x=13/12; dwarf 0 has no escape (every cell behind it is on dwarf
1's path), dwarf 1 retreats to the room and dwarf 0 walks out, then dwarf 1 goes back in. Both digs
complete well inside the 1,000-tick bound.

**AC5 trace:** the miner at x=13 walks to x=15, finds the idle dwarf on x=16. The idle dwarf has no free
neighbour off the miner's path, so it gets an exit `Path`; head-on, the miner has an escape and backs out;
the idle dwarf follows it out one or two cells per cycle ("snowplow", x=14 -> 12 -> 10), steps aside at the
room mouth (case 1, no `Path`), and the miner walks back in. Dig done at tick 306.

**Task 4/5 REDs and records (Agent B):**

- AC7 `save_load_keeps_the_exit_path_of_an_idle_dwarf_leaving_a_dead_end` (`save_load.rs`), with `from_save`
  temporarily not inserting `Path` (restored before commit):
  `assertion left == right failed  left: [(Id(0), (9,20,5), Walk, Lantern), (Id(1), (10,20,5), ...)]  right: [(Id(0), (10,19,5), ...), (Id(1), (10,20,5), ...)]`
  (loaded vs never-saved, at the first compared tick after the save).
- A FIRST AC7 fixture (head-on, dwarf 1 backing out west) was green without `Path` in the save: a loaded
  yielder hits the same blocked step again and re-derives the same yield, so a back-off alone does not
  need the saved path. A scan over every save tick 0..400 showed no divergence. It was dropped. The idle
  dwarf's exit path is what the save must carry: a scan of the AC5 fixture diverged at save ticks 12-22
  (idle dwarf at x 14, miner at x 13), and the committed test saves at the first tick the idle dwarf is at x 14.
- Pre-12.9 refusal `loading_refuses_a_pre_12_9_save_without_dwarf_paths` (`simd/src/main.rs`): green by
  construction (no `#[serde(default)]`); it strips `path` from every dwarf of a real save and expects
  `load_world_from` to return `None`, with a positive control (the same save with paths loads).
- AC6 `a_dwarf_in_the_only_passage_does_not_make_a_reachable_job_unreachable_at_claim_time` (`lib.rs`):
  green on the code as it stands (a guard, claim-time code is untouched). With the idle dwarf's wander
  cooldown at 10,000 the fixture timed out (`not done by tick 1000`). Not diagnosed to the line, but the
  reading is: the exit path `resolve_blocked_step` gives the idle blocker does not reset its wander cooldown,
  and `wander` follows a `Path` only when the cooldown reaches 0, so a 10,000-tick rest stalls the follow.
  No real dwarf rests that long (`WANDER_REST_TICKS` is small), so it is the fixture's artefact. Cooldown 40 outlasts the claim and the test passes.
  **Corrected at review run 1:** the fixture does not NEED 40. It passes at cooldown 0 and at 10; only 10,000 stalls.

**Task 10 REDs (Agent C), on the unfixed movement code (items not blocking):**

- AC1+AC10 `a_busy_crew_never_shares_a_tile_and_still_works` (`scenario.rs`, new assert before the vacuity asserts):
  `404 dwarf moves entered an item's cell; first (tick, dwarf, from, to) Some((112, 1, (67,66,9), (66,66,9)))`.
  **Corrected at review run 1:** that RED is on 12.9's occupancy code with items not blocking, NOT on `26185a0` as AC10
  asks; it was never run there. On `26185a0` the test reads `shared=332 item_entries=329`, and the occupancy assert fires
  first.
- AC10 corridor `a_dwarf_with_a_goal_past_a_stone_in_a_corridor_goes_round_it` (`lib.rs`):
  `the miner stood in the stone's cell at ticks [123, 124, ..., 133]`.
- AC11 `a_hauler_picks_up_and_drops_from_the_next_tile`:
  `tick 229: the hauler picked up from (10,20,5), not a 4-neighbour of (10,20,5)`.
- AC12 `a_3x3_pile_fills_from_the_inside_out`: `centre filled at 1071 but edge (17,19,5) at 290`.
- (A first AC11 draft failed with "never picked up" because `watch` returned at once on an empty job list and only dwarf 0 was
  tracked; fixed in the test, not the code.)

**Task 10 measured (green):** AC1 marks cleared t=1,205 (bound 2,500; baseline 1,112), first stone on pile t=228 (600),
moves 1,213 (600): all hold. AC12 pile full at tick 1,257 (named bound 2,000), centre at 212. No deadlock in the AC1 crew.
**Corrected at review run 1:** AC1 at `34b6783` (after Tasks 11-12) reads marks cleared 1,223, first stone 184, moves 1,199,
not the Task 10 figures above. All still inside 2,500 / 600 / 600.

**Task 6, the wire instrument (orchestrator).** The instrument gained a `stone entries` count (AC10 on the wire) and a
second verdict line, `STONES OK|RED`, committed `1569c93`. Every run used a fresh release `simd 7491`, DEFAULT_SEED, 1,500 ticks:

```
26185a0 (scratch worktree, the RED for both rules):
  ticks read 1479  deltas without exactly 5 dwarves 0
  shared ticks 186  max dwarves on one tile 2  first shared (101, {(65, 62, 9): [1, 3]})
  dwarf moves 551  channel marks 25 -> 0  items on the pile 9
  stone entries 138  first entry (156, {1: (65, 63, 9)})
  OCCUPANCY RED / STONES RED   exit 1
GREEN, 1569c93:
  ticks read 1479  deltas without exactly 5 dwarves 0
  shared ticks 0  max dwarves on one tile 1  first shared None
  dwarf moves 544  channel marks 25 -> 0  items on the pile 9
  stone entries 0  first entry None
  OCCUPANCY OK / STONES OK   exit 0
DELIBERATE RED, 12-9.sh row 1 (wander ignores occupancy) applied, release simd rebuilt:
  shared ticks 145  max dwarves on one tile 2  first shared (101, {(65, 62, 9): [1, 3]})
  dwarf moves 549  channel marks 25 -> 0  items on the pile 9  stone entries 0
  OCCUPANCY RED / STONES OK   exit 1
SELF-TEST, row 7 (no dwarf ever steps) applied, rebuilt:
  shared ticks 0  dwarf moves 2  channel marks 25 -> 25  items on the pile 0  stone entries 0
  CREW DID NOT WORK -- a frozen crew shares no tile; this is not a green   exit 2
```
The source was restored after each mutation, and a clean release `simd` was rebuilt afterwards.

**Task 7, mutations (orchestrator).** `mutations/12-9.sh`, 14 rows, committed `1569c93` before the run.
`RUST_TEST_THREADS=1 scripts/mutate.sh` was run alone: **14/14 KILLED**. The kill site of each row was read from
its panic line:

| Row | Test | Dies on |
| --- | --- | --- |
| 1 wander ignores occupancy | AC1 | the `World::step` one-dwarf-per-tile `debug_assert!` (lib.rs:2041) |
| 2 the job step ignores occupancy | AC3(a) | the `World::step` `debug_assert!` |
| 3 settle ignores occupancy | AC4 | the `World::step` `debug_assert!` |
| 4 per-step flip rule | AC3(b) | `watch`'s `not done by tick` bound: the livelock |
| 5 higher id never yields | AC3(b) | `not done by tick` |
| 6 idle blocker never makes way | AC5 | `not done by tick` |
| 7 no dwarf ever steps | AC1 | the vacuity assert `channel marks cleared ... bound 2500` (scenario.rs:2832), not the occupancy assert |
| 8 to_save drops path | AC7 | the per-tick `assert_eq` (save_load.rs:671) |
| 9 dwarf tiles join claim-time blocked | AC6 | `not done by tick` |
| 10 idle blocker swaps tiles | AC5 | `watch`'s no-swap assert |
| 11 blocked_cells ignores items | AC10 corridor | its stood-in-the-stone assert (lib.rs:5566) |
| 12 pick-up stands on the item | AC11 | `not done by tick` (the item tile is blocked, so the pick-up is unreachable) |
| 13 shallowest pile cell first | AC12 | `not done by tick` (the centre is walled in, so the pile never fills) |
| 14 drop at the hauler's own tile | AC11 | `the stone must land on the pile cell` |

**Trap 1 for rows 1-3:** the `debug_assert!` pre-empts each test's own shared-tile assert. Those asserts were RED
on 26185a0 (above), and AC1's was re-shown to kill independently: row 1 applied, `cargo test --release` (debug
asserts compiled out) -> `489 ticks had two dwarves on one tile; first Some((2, ...))` at scenario.rs:2816. Restored.
`scripts/audit-mutations.py`: 822 rows, every literal still matches.

**Rows 15-19 and a full re-run (orchestrator, `06a235e`).** After Tasks 11 and 12, the whole `12-9.sh` (rows 1-19)
reads **19/19 KILLED**. Each new row died on its own assert:
- 15: `the channel miner stood on the cell it channels` (scenario.rs:953).
- 16: the yaw assert in `a_channelling_miner_faces_the_target_cell`.
- 17: `frame 0: the stone slid off its own cell`.
- 18: `the first frame of the lift is the end of it`.
- 19: `the set-down is a one-frame snap`.

`12-5.sh` (Agent E re-pointed two of its rows) reads **15/15 KILLED**.

**Old tables Agent C re-pointed, RUN (orchestrator, on 036f43a).** C's re-pointing passed the audit but the rows
had not been run. Running `3-3-the-haul-and-the-skeleton-walks.sh` and `12-1.sh` found 9 rows not killing:
7 SURVIVED and 1 NO-COMPILE in 3-3, and 2 SURVIVED in 12-1. Agent D triaged each row against its own 26185a0 copy, run in
a scratch worktree with that tree's `mutate.sh`:
- **12.9 blinded 4 rows, all KILLED at 26185a0 and re-armed in `ca80219` (tests and table only, no production change):**
  - `the pick-up leg ignores standability`: a new assert in `haul_work_positions_gate_...` covers a stone with no
    floor beside standable ground.
  - `the pick-up leg uses job.target instead of the live position`: the stale target in
    `haul_execution_reads_the_stones_live_position_...` moved to a cell that shares no neighbour with the stone.
  - `the drop does not move the stone`: re-pointed at delivery's `= landing` write. Its old anchor matched only
    `release_claim`'s abnormal drop.
  - 12-1 `drop search walks through rock`: a loose stone now seals the pocket in
    `release_claim_drops_where_the_carrier_can_walk`.
  All 4 KILLED after the fix.
- **5 rows did not kill at 26185a0 either.** They are pre-existing dead rows, not caused by 12.9, and are left alone:
  - 3-3 `free stockpile tiles ignore standability`, `the pick-up leg drops the free-tile gate` and
    `every stone on a zone tile counts as stored`: SURVIVED.
  - 3-3 `load_world accepts two dwarves carrying one item`: NO-COMPILE (E0282, the payload leaves a set's type
    uninferrable).
  - 12-1 `retry drop stacks a full stockpile`: SURVIVED.

- **Task 11 RED (Agent E).** `a_channel_is_worked_from_the_next_tile_and_the_stone_lands_on_the_target` (scenario.rs, AC13 sim):
  `assertion left != right failed: the channel miner stood on the cell it channels (left == right == Pos { x: 63, y: 65, z: 9 })`.
  `a_channelling_miner_faces_the_target_cell` (gui headless.rs, AC13 gui): `a channelling miner faces his target (east), drew
  Quat(0.0, 0.70710677, 0.0, 0.70710677)` (west: `dig_yaw` ignored Channel).

- **Task 12 RED (Agent E, gui `headless.rs`, AC14).** `a_stone_the_wire_puts_in_his_hands_holds_its_cell_while_the_hauler_walks_in`:
  `frame 0: the stone slid off its own cell, left Vec3(1.0, -0.3, 0.0) right Vec3(2.0, -0.3, 0.0)`.
  `a_stone_picked_up_from_the_next_tile_rises_to_his_hands_over_a_lift`: `the first frame of the lift is the end of it`
  (`Vec3(0.0, 0.705, -0.58)` == `CARRY_OFFSET` in frame 1). `a_stone_released_onto_the_next_tile_is_set_down_over_a_lift`:
  `the set-down is a one-frame snap` (`Vec3(1.0, -0.3, 0.0)` on the cell in frame 1).

- **Full gate on `ac99c2d`** (`RUST_TEST_THREADS=1 scripts/gate.sh`): `GATE GREEN 3281s`. cargo test 250 s,
  pixel guards 2,994 s, the three no-sim-core-edge probes ok, mutation tables still apply.

### Completion Notes List

- Tasks 1-3 done. `execute_jobs`, `settle` and `wander` each build a `BTreeSet<Pos>` of dwarf tiles once
  (`dwarf_tiles`) and keep it current as their dwarves move, in ascending `Id`. `settle` finds the landing
  tile with `route_to_nearest` (breadth-first, `astar_neighbours`, bounded by `MAX_ASTAR_NODES`). The same
  helper gives the escape and the exit path. `World::step` ends in a `debug_assert!` of one dwarf per tile.
- The blocked step is `resolve_blocked_step` (cases 1-4 as in Task 3). No swap anywhere. `wander` follows an
  idle dwarf's `Path` at `STEP_REST_TICKS` pace, waits on an occupied tile, draws no RNG, and on arrival
  removes `Path` and sets `home`.
- A blocker that is idle but already has an exit `Path` starting on the holder's tile goes straight to the
  head-on case. A case-1 sidestep removes any stale `Path` from the blocker.
- **Re-pins.** `save_load.rs` `save_load_then_tick_matches_never_saved`: the loop guard `saved.tick() < 600`
  became `< 1_000`. The test's corridor is one wide and a dead end, an idle dwarf wanders into it, and the
  hauler now backs out and walks back in rather than passing through it. The pick-up now lands at tick 691
  (the dwarf the hauler displaces is the pre-existing wanderer). Cause is occupancy, not a defect: the same
  test's save/load comparison is unchanged and green. No other figure moved: all 50 scenario and all 72 lib
  tests (including AC8's guards) pass unchanged, and `cargo test -p simd` is green (22 + 72).
- `Path` is still not saved (Task 4). An idle dwarf's exit path or a yielder's path is therefore lost on
  save/load until Task 4 lands. No existing save/load test went red on that.
- `74-dwarves-path-through-fire.sh` row 3 (corrected at review run 1; this said row 2) anchors on `positions.copied().collect()`, which the new
  `dwarf_tiles` would have duplicated; `dwarf_tiles` spells it `.cloned()` so the audit stays clean
  (`scripts/audit-mutations.py`: 808 rows, all match).
- The `wander` system gained `#[allow(clippy::type_complexity)]` (one query over the dwarf row, plus `Path`).
- Tasks 4-5 done (Agent B). `SavedDwarf.path: Vec<Pos>` has no serde default (NOTE added); it is no longer
  `Copy`. `to_save` writes every dwarf's `Path` (idle exit paths included) and `from_save` inserts `Path` when
  non-empty. The Task 1-3 agent reported no open problem in this territory; `Path` not being saved was its
  known gap, closed here. No existing save/load test moved, `save_load_recomputes_every_path_invalidated_by_another_dig`
  still passes, and no figure was re-pinned in Tasks 4-5.
- AC7's precondition ("a path a fresh A* would not return") is asserted through the public API: the idle
  woodcutter is idle (`claims()` None) and must walk out of the dead end WEST, one cell per step, away from
  the face, which neither A* nor a wander roll reproduces. The saved yielder's back-off is NOT a sensitive
  case (see Debug Log): a loaded yielder re-derives it.
- AC6 added in `lib.rs` `mod tests` (the fixtures need `ecs`).
- The pre-12.9 refusal test lives in `simd` (`serde_json` is simd's dependency, sim-core has none).
- Not done (not mine): `occupancy_wire.py` shows as modified in the working tree; it was not touched here
  and is not staged.
- Task 10 done (Agent C). `blocked_cells` now = emitters + every uncarried item's tile in claim_jobs, execute_jobs (rebuilt per
  dwarf), wander (carried items excluded), settle, release_claim's drop search and the Task 3 helpers (their `emitters` param renamed
  `blocked`). `PlaceStockpile` still uses emitters only (a stockpile may be placed over a loose stone). New: `pile_targets`
  (free pile cells by BFS depth), `drop_cell` (deepest free adjacent cell, lowest Pos), `world_blocked`, `side_neighbours`.
  `work_positions`: Dig/Cut via `is_walkable`; pick-up = walkable 4-neighbours of the item; delivery = walkable 4-neighbours of
  the deepest free pile cells. Delivery lands the stone on `drop_cell`; release_claim keeps its own-tile drop (NOTE).
  Two additions beyond the text, both needed for AC10: execute_jobs re-plans when a stored path's next tile became an item
  (a delivery changes no terrain, so clear_paths never runs), and wander drops an idle exit path whose next tile became an item.
  `settle` does not land a faller on an item cell (nearest free tile instead; with none it hangs, 12.11).
- **Intended changes to old-rule tests (disclosed):** `haul_work_positions_gate_both_legs_on_a_free_standable_pile_tile` (goal
  sets are now neighbours, `blocked` carries the items); `a_haul_walks_picks_up_walks_and_drops_in_two_work_runs` (first walk
  1 step not 2); `pickup_sets_carrying_...` (hauler stands on cell(1), not the stone); `release_claim_drops_where_the_carrier_can_walk`
  (second pile cell now free, since a full cell walls the pocket; drop expected at pocket[1]);
  `claimed_dwarf_settles_before_moving_from_newly_unsupported_ground` (a free landing tile added, faller does not land on the dig's stone);
  scenario `two_deep_dig_advances_from_the_exposed_face` (a one-cell pile added and loop 500 -> 1,500: the outer stone blocks the
  one-wide tunnel until hauled; final asserts: stone on pile and at inner).
- **Re-pin:** scenario `a_stockpile_around_the_campfire_never_zones_or_receives_the_fire` "full by" 2,000 -> 2,500. Measured on the
  old and new code: old 24/24 on the pile at t=2,000, new 21 at 2,000 and 24 at ~2,250, then no further pick-up. Cause: items block
  and deliveries go deepest-first; not a defect.
- Results: `cargo test -p sim-core` (75+13+50+19 green), `-p simd` with RUST_TEST_THREADS=1 (23 + serve 72), `-p tui`, `-p gui`: all green, no client change.
- Task 11 done (Agent E). `work_positions` Channel = walkable same-z 4-neighbours of a standable target; the NOTE about standing in
  its own stone is gone. `dig_yaw` takes `DwarfJob::Channel`. **Intended changes to old-rule tests (disclosed):** the two `lib.rs`
  channel unit tests (`execute_jobs_channels_a_material_preserving_ramp_and_spawns_stone`, `..._removes_a_channel_job_when_the_support_is_already_a_ramp`)
  now put the target beside dwarf 0 (they stood him on it). AC8 guard `a_channel_worker_whose_support_is_removed_lets_go_and_the_crew_goes_on`
  now finds the Work-state dwarf on a 4-neighbour of the target and removes the support under HIS tile; the "lets go within 2 ticks" asserts are
  unchanged. Its last assert changed: the target keeps its own support and other neighbours stay walkable, so another miner can now finish the
  channel (before, the holder's tile was the only work position and the order stayed forever). It now asserts the designation is present OR the
  below tile is a Ramp (never vanished unworked). `headless.rs` `a_digging_dwarf_faces_his_target_and_a_channel_keeps_his_heading`: only the
  step label changed (a channel aimed at his own cell has no direction); the name stays because mutation row 12-5 names it. No pinned figure
  moved; AC1's bounds hold (all 51 scenario tests green). `simd/tests/serve.rs` `deltas_label_a_miners_dig_a_haulers_haul_and_the_stone_he_carries` asserted `target == dwarf.pos` for a channel (the old rule; failed `[66,66,9]` vs `[66,65,9]`): it now asserts the target is a same-z 4-neighbour of the miner. Mutation row `12-5.sh` "a digging dwarf no longer faces his target"
  was re-pointed at the new five-line `dig_yaw` pattern and RUN alone (temp table in the scratchpad): KILLED.
- Task 12 done (Agent E, gui only). One component, `ItemMotion { from, to, elapsed }`, and `advance_item_motion` (chained after
  `sync_dwarf_work`, paced by `Time::delta_secs()` like the walker; `LIFT_SECONDS = 0.3` serves the lift and the set-down; NOTE: a paused
  world still finishes one). `sync_dwarf_work` parents as before but starts the child at the stone's drawn position in the dwarf's local space and
  lifts it to `CARRY_OFFSET`; on release it unparents and sets down from the hands (dwarf transform applied to the local translation) to
  `item_translation(...)`. `blend_entities` now skips an unparented item that any entity carries (the wire moved it to the hauler's tile) and one with
  an `ItemMotion`; `BlendQuery` gained `Option<&ItemMotion>`. No existing test moved. Mutation row `12-5.sh` "the blend writer moves a carried stone" was
  re-pointed (the filter line changed; the sabotage now drops the whole guard with `.filter(|_| true)`) and RUN alone: KILLED. `audit-mutations.py`: 822 rows
  all apply. Not run (orchestrator): gui pixel guards. No row yet pins the new hold/rise/set-down (rows 17-19 of the story's list are for the orchestrator).
- **Summary (orchestrator, for review).** ACs 1-14 met. Wire GREEN: 0 shared ticks and 0 stone entries (baseline 186 / 138).
  12-9.sh 19/19 KILLED; 12-5.sh 15/15 KILLED; full gate GREEN on `ac99c2d`. Seat: pass 1 → Tasks 11-12; pass 2 channel yes,
  carry look parked as #181. **Flags for the reviewer:**
  - (1) the AC8 guard's last assert is weaker (designation OR ramp), see above;
  - (2) an out-of-order abnormal drop (`release_claim`) can wall in a deeper free pile cell, because `pile_targets` depth passes
    through taken cells;
  - (3) Agent A wrote `dwarf_tiles` with `.cloned()`, which does not match #74's sabotage anchor `positions.copied().collect()`;
  - (4) ~~the AC6 fixture needs an idle cooldown of 40~~ FALSE (review run 1): it passes at cooldown 0 and 10; only 10,000
    stalls, because an exit `Path` waits on the wander cooldown;
  - (5) five 3-3 and 12-1 rows were already dead on 26185a0, and nothing here repairs them.

### File List

- `crates/sim-core/src/lib.rs` (occupancy in the three writers, `route_to_nearest`, `resolve_blocked_step`,
  `World::step` assert, fixtures and AC3-AC5 tests)
- `crates/sim-core/tests/scenario.rs` (AC1, AC2; `path` on the two `SavedDwarf` literals)
- `crates/sim-core/tests/save_load.rs` (guard re-pin; AC7 test; `path` on the two `SavedDwarf` literals)
- `crates/sim-core/src/save.rs` (`SavedDwarf.path`, no longer `Copy`)
- `crates/simd/src/main.rs` (pre-12.9 save refusal test)
- Task 10: `crates/sim-core/src/lib.rs` (blocking items, pile depth, drop cell, fixtures AC10 corridor / AC11 / AC12), `crates/sim-core/tests/scenario.rs` (AC10 assert, two disclosed test changes)
- `_bmad-output/implementation-artifacts/mutations/12-9.sh` (NEW, rows 1-14)
- `_bmad-output/implementation-artifacts/mutations/12-1.sh`, `mutations/3-3-the-haul-and-the-skeleton-walks.sh` (9 rows re-pointed by Agent C; both tables were then run: 4 rows re-armed by Agent D in `ca80219`, 5 were already dead on 26185a0, see Debug Log)
- `_bmad-output/implementation-artifacts/12-9-signoff/occupancy_wire.py` (`stone entries`, `STONES` verdict)
- `_bmad-output/implementation-artifacts/12-9-signoff/vehicle-card.md` (NEW, Task 9)
- `_bmad-output/planning-artifacts/epics.md` (Task 0 ruling note on Story 12.9)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- Task 11/12 (Agent E): `crates/sim-core/src/lib.rs`, `crates/sim-core/tests/scenario.rs`, `crates/gui/src/project.rs`, `crates/gui/src/ingest.rs`, `crates/gui/tests/headless.rs`, `crates/simd/tests/serve.rs` (channel target is a 4-neighbour of the miner), `_bmad-output/implementation-artifacts/mutations/12-5.sh` (two rows re-pointed), `mutations/12-9.sh` (rows 15-19)
- `_bmad-output/implementation-artifacts/12-9-one-dwarf-per-tile.md` (this record); `metrics/12-9-one-dwarf-per-tile.md` and `metrics/.session-cursors.json` (cost ledger)

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-08 | Story created on `26185a0`. #133 reproduced: sim probe (494/3,000 idle ticks shared on `DEFAULT_SEED`, 332 busy) and live wire (`occupancy_wire.py`, 186 shared ticks, `OCCUPANCY RED`). The per-step head-on rule was traced to a livelock, and the escape rule replaces it. `Path` joins `SaveState`. Task 0 (Q1–Q4) is open |
| 2026-10-08 | **Task 0 ruled (Wolf).** Q1 (b): no swap; the idle dwarf gets an exit `Path` and the miner backs out (AC5 and Task 3.1 amended, mutation row 10 added). Q2: refuse pre-12.9 saves ("old saves are not important"). Q3 (b): **#162 folded in**. Every uncarried item blocks, on pile cells too ("taken pile cells should be impassable"), confirmed over the rec to split it into its own story. Pick-up and drop from the next tile, pile fills inside out: AC10–AC12, Task 10, rows 11–14, instrument `stone entries`. Q4 (a): seat look. Dev mode: Sonnet 5.5 subagents |
| 2026-10-08 | Tasks 1-3 (Agent A): RED fixtures AC1-AC5, occupancy in `execute_jobs`/`settle`/`wander`, blocked step with escape/yield, idle exit path, no swap. One re-pin (`save_load` guard 600 -> 1,000) |
| 2026-10-08 | Tasks 4-5 (Agent B): `SavedDwarf.path` saved and restored, AC7 and AC6 tests, pre-12.9 refusal test |
| 2026-10-08 | Task 10 (Agent C): every uncarried item blocks; pick-up/drop from the next tile; pile fills inside out. AC10-AC12 tests, one re-pin, six old-rule tests updated |
| 2026-10-08 | Tasks 6-7 (orchestrator): instrument `stone entries`; GREEN 0/0, deliberate RED (row 1) 145 shared, self-test exit 2; 26185a0 baseline 186 shared / 138 stone entries. `12-9.sh` 14/14 KILLED, kill sites recorded; AC1's own assert shown to kill in `--release` |
| 2026-10-08 | Ran the two old tables Agent C re-pointed: 9 rows were not killing. 4 had been blinded by 12.9 and are re-armed in `ca80219` (tests only). 5 were already dead on 26185a0 and are left alone, recorded |
| 2026-10-08 | Seat pass 1 (Wolf): channel from the next tile → Task 11 / AC13; pick-up and drop drawn as a reach → Task 12 / AC14 (both ruled into 12.9, as recommended); self-haul idea → #180; FPS swing → #179. Rows 15–19 added |
| 2026-10-08 | Tasks 11-12 (Agent E): channel from the next tile, `dig_yaw` Channel arm, lift and set-down (`ItemMotion`). Two 12-5 mutation rows re-pointed and run, both KILLED |
| 2026-10-08 | Tasks 11-12 (Agent E, `d93f32d` `bfa6568` `0fccc74`). Rows 15-19 added; 12-9.sh 19/19 KILLED and 12-5.sh 15/15 KILLED on `06a235e` |
| 2026-10-08 | Full gate green on `ac99c2d` (3,281 s) |
| 2026-10-08 | #162 commented. Seat pass 2 (Wolf): channel yes; carry better but still reads as suction and a slide, parked as #181. Tasks 8-9 done, Status review |
| 2026-10-09 | Review run 1 patch pass (fresh session, Wolf ruled #182 option 1 let-go and #183 option 1): `6dbdafe` `0f8a5b2` `f5cfd32` `48ac6b0` `c9311f1` `0b8322d` `aa94a5d`. 6 of 7 patches closed; record corrections waits on the PR-body disclosure. Rows 22-28 KILLED (27 via its own corridor test). Sweep: 0 landings, 32/32 runs worked, 5 one-level islands frozen as #186; chain livelock filed #185. Live wire GREEN on `aa94a5d` (0 shared, 0 stone entries, 0 landings), deliberate RED row 25 = 6 landings. Full gate green on `aa94a5d` (3,654 s). Seat items still open, Status in-progress |
