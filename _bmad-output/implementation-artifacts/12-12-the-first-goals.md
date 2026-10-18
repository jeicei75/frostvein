---
baseline_commit: 60955ef
model: claude-opus-5-5  # session default, same as 12.1-12.11's creation
---

# Story 12.12: The First Goals

Status: in-progress

## Story

As the boss,
I want the game to give me a short chain of goals and tell me when I meet them,
so that my first session has a direction without ever ending the game.

## Not stacked: branch off `main`

`main` is `60955ef` (PR #205 merged), clean. Branch `story-12-12-the-first-goals` carries this file,
`12-12-signoff/` and the board edit. Epic 12's standing ACs (`epics.md:2422-2434`) bind this story and
are not restated. Numeric order is the ruled execution order for Epic 12 (`sprint-status.yaml:2486`), and
12.12 is the last non-stretch story.

**Wire diff (standing AC 4):** a new `protocol::Goal { kind: ItemKind, target: u32, progress: u32,
completed: bool }`, and a `goals: Vec<Goal>` field, always serialized, on BOTH `Snapshot` (after `tick`) and
`Delta` (after `speed`, before `refusals`). It holds all three goals, in chain order, on every message (AD-8
full resend). For example:
`"goals":[{"kind":"wood","target":10,"progress":3,"completed":false},{"kind":"tetrahedrite",...},{"kind":"bloodstone",...}]`.
`kind` reuses `ItemKind`, so there is no new enum. No new command and no new refusal.

## Found at creation (2026-10-18, on `60955ef`)

- **Nothing exists.** There is no goal, tutorial or announce code in any crate, and no count of stored
  items anywhere. The only definition of "in a stockpile" is inline in `create_haul_jobs`
  (`lib.rs:389-410`, 3.3's rule): an item is STORED iff it is uncarried, on a zone cell, and the lowest-id
  uncarried item on that cell. There is no flag.
- **The chain is reachable on DEFAULT_SEED, judged by the wire.** `12-12-signoff/goals_wire.py`
  (written at creation) plans the work from the snapshot: the cheapest trees to cut, the cheapest
  tunnels to tetrahedrite and then bloodstone, and non-enclosing piles. It sends the commands and counts
  stored items per kind from the wire alone (an oracle that is independent of the sim). Release `simd`, fresh
  daemons, fast4x. Each target set below was met by every one of its runs. The final instrument pauses the daemon while
  it plans, so the run is identical to the tick on every fresh daemon; the rows other than 10/10/1 were
  measured by its earlier, unpaused form:

  | wood / tetrahedrite / bloodstone | all stored by tick | runs |
  |---|---|---|
  | 5 / 5 / 1 | 6,042 | 1 |
  | **10 / 10 / 1** | **9,828** (final instrument; the unpaused form gave 10,454 in 3 runs, and 11,383 before the pile slack) | 3 identical |
  | 8 / 8 / 2 | 18,176 | 1 |
  | 10 / 10 / 3 | 24,583 | 1 |

  For 10/10/1, the plan was byte-identical on 3 of 3 daemons (snapshot at tick 6) and is kept as
  `12-12-signoff/plan-10-10-1.json`: 46 digs, cuts at `[65,56,10] [61,74,13] [65,73,12]` (12 logs), 3
  piles of 30/30/18 cells. The first bloodstone is `[55,83,13]` (depth 6), stored at tick 6,352. The 2nd and 3rd
  are `[95,23,17]` and `[75,96,18]`, about 40 cells from camp.
- **Deliberate RED (3 of 3 on `60955ef`, final instrument):** `goals: the daemon sent NO goals field (last tick
  10028)`, `GOALS RED`, exit 1. Each run printed its oracle lines first (`oracle: 10 wood stored by tick 1847`,
  `oracle: 1 bloodstone stored by tick 6352`, `oracle: 10 tetrahedrite stored by tick 9828`), and phase 2
  dropped the stock (wood 12→9, tetrahedrite 10→5), so the run is not vacuous. An earlier form that planned
  while the daemon ran was once dropped by it (`client delta queue full`, a truncated line); the final form
  pauses on connect and drains deltas on a thread, and reports a dropped reader as `GOALS BROKEN`.
- **A pile ringing the campfire seals itself (new defect, FOLDED IN by Task 0 Q3: Task 2b, AC 11).** The probe's first
  planner laid the pile round the fire and left the fire's 4 neighbours out. The middle of the band filled
  first and cut 8 free pile cells and the 4 fire-side cells off from every hauler. Delivery A* finished
  with "no path" over 16,037 cells, with no budget exhaustion. 8 haul jobs were then retried forever, both
  haulers idled, and the chain stalled at 6 of 10 tetrahedrite, 4 runs out of 4. The root is that `pile_targets`
  (`lib.rs:759-771`) counts any walkable non-zone neighbour as a way in, including ground the zone
  surrounds. A throwaway fix in `pile_targets` (an entrance must connect to ground outside the zone)
  refilled 54 of 58 on that geometry, against 50 without it (from a re-fill of the save's pile; the 4 left
  loose were the probe's own misplacement on an unreachable ledge). The instrument lays compact piles that
  enclose nothing, so it judges the goals, not this defect; Task 2b's test judges the defect.
  Creation geometry (z 9, `F` the campfire `[64,64,9]`, `#` a filled pile cell, `o` a free one, `.` walkable;
  rows y 62-66): `..#####o###.` / `.#####o.o##.` / `.####o.F.o#.` / `.#####o.o##.` / ` .#####o##.`
- **Extra logs.** A planned dig under a trunk fells that tree too (12.11 Q2), so 17 logs reach the pile where
  12 were planned. Size piles with slack.

### Task 0: RULED (Wolf, 2026-10-18)

- **Q1 RULED (b): 10 wood, 10 tetrahedrite, 1 bloodstone** ("1 ok"). `plan-10-10-1.json` is the plan.
- **Q2 RULED (A): the draft as written** ("2 ok"). No banner.
- **Q3 RULED: fold it in** ("fold it"), against the creation recommendation to file it. It fails silently
  (idle haulers, haul jobs retried forever) and it sits on AC 10's seat path. Task 2b, AC 11.

The options as offered at creation, kept for the record:

- **Q1, the targets** (hardcoded constants, FR47). Measured above. **(b) is recommended:** 10 wood, 10
  tetrahedrite, 1 bloodstone, at about 10,000 scripted ticks (about 4 min at fast4x). The one gem is the
  nearest, depth 6, and it is the 12.10 card's own gem. (a) 5/5/1, about 6,000 ticks. (c) 8/8/2, about 18,000. (d)
  10/10/3, about 25,000, with two gems about 40 cells out.
- **Q2, the look:** approve `12-12-signoff/draft.md`. The recommended option is **(A)**: a goal list
  bottom-right in the gui, `[x]` and green when done, and a gold `all goals done - play on` title; the tui
  gets a fourth text row. The other option is **(B)**: the same, plus a client-timed banner of about 5 s on each
  completion.
- **Q3, the pile-ring defect:** file it as an issue (recommended; it is not on this story's path once the
  card says "don't ring the fire"), or fold it in here.

### Defaults taken at creation (Wolf may overrule)

- **D1, progress is live and completion is per tick.** `progress` is the STORED count of the goal's kind,
  computed when read (bridge and `World::goals()`), so it always matches what the pile holds.
  `completed` is decided by the sim at the end of each tick. While the daemon is paused nothing completes
  (the schedule doesn't run): a pile placed over 10 logs while paused reads `10/10` unticked until the
  next tick. Leave a `// NOTE:`.
- **D2, the chain cascades in one tick.** When goal k completes and goal k+1's stock already meets its
  target (stock counts before a goal starts, FR47), k+1 completes in the same tick, and so on.
- **D3, pre-12.12 saves are refused** (no `#[serde(default)]`), as 12.2/12.4/12.9/12.11 ruled. simd
  refuses a save whose `goals_completed` exceeds the chain length.
- **D4, "the chain is done" is derived:** every goal `completed`. There is no separate wire flag.

## Acceptance Criteria

1. On a fresh world, `World::goals()` and the first delta's `goals` both list wood, tetrahedrite and
   bloodstone in that order, with the Task 0 Q1 targets, progress 0 and none completed (FR47).
2. At the end of every tick, each goal's `progress` equals the number of STORED items of its kind (3.3's
   rule). A carried item, a loose item and the second item on one pile cell do not count. Items stored
   before the goal became current do count (FR47).
3. A goal completes at the end of the first tick on which every goal ahead of it is completed and its
   progress meets its target, and never earlier. It never goes back to incomplete: when stock drops below
   target (a pile removed; a stored item moved off its cell, as a clearing miner can) the goal stays
   completed and its progress follows the stock down (FR47, D2).
4. Seed + command log ⇒ identical `goals()`, and save → load → tick N ≡ never-saved covers a world with one goal
   completed and the next one in progress (standing AC 3). A pre-12.12 save, and a save with
   `goals_completed` beyond the chain, are refused by simd with a logged reason (D3).
5. Every `Snapshot` and `Delta` carries `goals` exactly as the wire diff above, pinned by byte-exact
   literals in `protocol`. On a real `simd` (`serve.rs`), a loaded world with items on a pile sends
   `progress` above 0 that equals the stored count in that same delta.
6. On DEFAULT_SEED, a scenario test applies the creation plan (`plan-10-10-1.json`, or Q1's) and the chain
   completes within a pinned tick bound. On every tick of that run, no goal completes before the one
   ahead of it. 200 ticks after the chain the tick has climbed and every goal is still completed.
7. The tui shows the goal row as drafted (Task 0 Q2): the current goal and its count, the done goals, and
   the chain-done line. It changes on the delta that changes `goals`, in `tui --frames` against a stub
   server (NFR10).
8. The gui shows the goal list as drafted: done goals ticked, the current one distinct, the gold
   announcement once all are done. It is a `Hud` text (`H` hides it; captures hide it), and a headless
   test shows its text change when the mirror's goals change.
9. `goals_wire.py` prints `GOALS OK` on 5 of 5 fresh release daemons, and each mutation row of Task 7 turns
   its named test, or the wire, RED.
10. At the seat (Task 0 Q2 approved first), Wolf completes the chain in play on a fresh world, in the gui with an
    attached tui. He sees each goal complete and the chain announced, and play goes on (success
    criterion 3, UX-DR22 closing half).
11. A pile that surrounds walkable ground outside the zone (the creation ring round the fire) still fills
    every pile cell a hauler can deliver to: with at least as many items as pile cells, no free pile cell is
    left while an item lies loose, and no haul job retries forever. Pile layouts that enclose nothing fill in
    the same order as today (Q3).

## Tasks / Subtasks

- [x] **Task 0: Wolf rules Q1-Q3** and approves the draft (AC 10's opening half). RULED 2026-10-18:
  Q1 10/10/1, Q2 (A), Q3 fold in.
- [x] **Task 1: RED tests first** (standing AC 2)
  - [x] `crates/sim-core/src/lib.rs` unit tests (they can reach private `Zones`, as `lib.rs:4480` does), in a
    world with hand-placed items on a pile: progress counts stored only (carried, loose and a second item on
    a cell excluded); a completed goal stays completed after its pile is removed; the cascade (D2); order
    (bloodstone stored first does not complete bloodstone).
  - [x] `crates/sim-core/tests/scenario.rs`: `the_default_world_completes_the_goal_chain_with_scripted_commands`
    (AC 6). It steps a fresh DEFAULT_SEED world to the plan's `snapshot_tick` and applies the plan's digs
    (one `Designate` dig per cell), cuts and pile cells (one `PlaceStockpile` of 1-cell rects), as literals
    ported from the JSON. Then `run_until` the chain is done. On EVERY tick it checks the goals against an
    oracle built in the test from public readers only (`items()`, `item_kinds()`, `carrying()`, `zones()`;
    lowest id per cell). Pin the bound at the measured completion tick plus a disclosed margin, and say how
    long the test takes in debug.
  - [x] Each is committed RED with `#[ignore = "RED until Task N"]`, with its failing line pasted into the Debug
    Log; it is un-ignored by the commit that greens it.
- [ ] **Task 2b: a pile that encloses ground still fills (Q3)** (AC 11)
  - [x] RED first (Task 1's rule): a scenario test that lays a pile band round a walkable hole that is not
    zoned, with haulers outside the band and at least as many items as pile cells. The creation shape (the
    map above) is the reference: the campfire's ring with its 4 neighbours left out of the zone. Build it
    with `make_standable` (`scenario.rs:12`) or on DEFAULT_SEED's camp. Its failing assert must show the
    seal, i.e. a free pile cell left with a loose item whose haul job keeps retrying, not merely a slow fill.
  - [x] Fix in `pile_targets` (`lib.rs:759-787`): a walkable non-zone neighbour counts as a way into the
    pile only if it connects to walkable ground outside the zone without crossing it. Ground that the zone
    surrounds then seeds no depth-1 layer, so the cells next to the hole are the deepest and fill first. The
    creation's throwaway did this with a capped flood (4 × zone size + 16 cells) over non-zone walkable
    ground. Bound the flood the same way and state the cap in a `// NOTE:`.
  - [ ] `pile_targets` runs on every haul claim and delivery (`lib.rs:1238`, `:2115`). Measure the max tick
    time before and after, with a scratch probe in release on a quiet machine, as 12.9 did (32 runs, max
    8.54 ms, 0 over 10 ms; `12-9-one-dwarf-per-tile.md:689`, probe not committed). Paste both.
  - [x] Non-enclosing piles must not change: the two-wide and one-wide staircase sweeps
    (`scenario.rs:4660`, `:4766`), the cage sweep and the walking skeleton stay green with NO re-pin
    (expected, because every entrance of such a pile already reaches open ground; inferred, not measured).
    Any pin that moves is a finding: name its mechanism from a dump before re-pinning it.
- [x] **Task 2: goals in the sim** (AC 1-3)
  - [x] Extract the stored rule from `create_haul_jobs` (`lib.rs:389-410`) into ONE helper that both
    callers use, e.g. `fn stored_items(ecs: &EcsWorld) -> BTreeMap<u32, Pos>` over `uncarried_stones`.
    `create_haul_jobs` keeps its `loose` list from the same pass or the complement. Do not make a second
    copy of the lowest-id clause.
  - [x] `pub const GOALS: [(ItemKind, u32); 3]` holds Q1's targets in chain order; `#[derive(Resource)] struct
    GoalsCompleted(u32)` counts the goals done, in order;
    `pub struct Goal { pub kind: ItemKind, pub target: u32, pub progress: u32, pub completed: bool }`
    (Debug, Clone, Copy, PartialEq, Eq); `pub fn goals(&self) -> Vec<Goal>`, with progress computed live (D1).
  - [x] `fn advance_goals(ecs: &mut EcsWorld)`: while the next goal's stored count meets its target, mark it
    completed (D2). Chain it AFTER `carry_items` (`lib.rs:2556-2569`), because positions are final only
    then, and rewrite `carry_items`' "Exclusive and LAST" doc (`:2365-2366`).
- [x] **Task 3: save** (AC 4)
  - [x] `SaveState.goals_completed: u32` (`save.rs:9-27`), with no serde default and a NOTE like the items
    one (`:23-24`). Fill it in `to_save` (`lib.rs:2640-2713`), destructure it in `from_save` (`:2715-2731`)
    and insert the resource beside `assemble` (`:2527-2571`).
  - [x] simd `load_world_from` (`main.rs:344-716`): `bail!` when `goals_completed > GOALS.len()`. Add two
    tests beside `loading_refuses_a_pre_12_11_save_without_dwarf_clearing` (`main.rs:1014`): pre-12.12 and
    out-of-range.
  - [x] `save_load.rs`: a sibling of `save_load_then_tick_matches_never_saved` (`:10-187`) saved with one
    goal completed and the next one's progress above 0. Assert both before saving, so the test is not
    vacuous, and compare `goals()` for 200 ticks. Add `goals()` to the reader list of
    `same_seed_and_commands_remain_deterministic` (`scenario.rs:2057-2129`); its window must show progress
    above 0 (assert it).
- [ ] **Task 4: wire** (AC 5)
  - [ ] `protocol`: `Goal`, then `goals` on `Snapshot` (after `tick`) and on `Delta` (after `speed`). Re-pin
    `WIRE`/`DELTA_WIRE` (`protocol/src/lib.rs:313-334`) and the byte-exact delta at `:370`, and add one goal
    literal test.
  - [ ] `simd/src/bridge.rs`: `snapshot()` (`:1-39`) and `delta()` (`:46-85`) fill `goals` from
    `world.goals()` through `item_kind_out` (`:109-116`). A bridge unit test checks goals against
    hand-written wire values.
  - [ ] `serve.rs`: write a save with items on pile cells (a stacked pair on one cell included) and load it
    (`read_snapshot_after_load`, `:197`). The snapshot and the next delta carry `progress` above 0, and that
    equals the stored count the test computes from the same message's `items`, `zones` and `carrying`.
  - [ ] Every `Snapshot {`/`Delta {` literal in the workspace gains the field. That is mechanical, about 66 Snapshot sites, mostly in gui
    tests. A shared test helper is fine; a `Default` derive on the wire types is not.
- [ ] **Task 5: clients** (AC 7, 8)
  - [ ] `client-core`: `Mirror` stores `goals` and gets a `goals()` getter. NAME `goals` in `apply_delta`'s
    destructure (`client-core/src/lib.rs:106-115`): its `..` would drop the field silently. Also set it in
    `replace` (`:195-217`). The goal text the two clients share lives here, next to `refusal_text`
    (`:10-27`), e.g. `goal_kind_text(ItemKind) -> &'static str` (`wood`, `tetrahedrite`, `bloodstone`) plus
    the line builders the draft needs. Add a test that a delta's goals reach the mirror.
  - [ ] tui (`view.rs:235-489`): the goal row above the roster. Replace the three `3`s (`:249-250` map height,
    `tally_marks` `:107-109`, `move_cursor` `:682`) and the `h < 3` guard (`:245`) with ONE named constant.
    Re-pin `one_row_terminal_renders_blank` (`:1037`) and the pinned status strings, and disclose them. A
    `tui --frames` stub test in `tests/client.rs` (pattern: `capture_frames_refusing`, `:214-275`): goal row
    text before and after a delta that completes wood, and after the chain.
  - [ ] gui: a `GoalReadout` Text modelled on `setup_name_readout` (`ingest.rs:1774-1791`, 22 px, `Hud`,
    `ClientLocal`, `GlobalZIndex(i32::MAX - 16)`) at bottom 16 px, right 16 px. It is updated from
    `mirror.0.goals()` with span colours as drafted (green 88,190,118; white; grey `TRADE_TEXT_COLOR`
    `appearance.rs:454`; gold `#D6A42C`). Re-pin the HUD census (`ingest.rs:4075`, `:4119`: 6 → 7) and the
    production text set (`:6111-6142`). Add a headless test that the spans change when the mirror's goals
    change (pattern: roster `:3721`). If Q2 is (B), add the banner and its own test.
- [ ] **Task 6: the instrument** (AC 9)
  - [ ] `12-12-signoff/goals_wire.py` (EXISTING, creation) is the instrument. Run it on 5 fresh release
    daemons, each from a directory with no `frostvein.save`, at most 4 at a time, with Q1's targets. Paste
    each verdict block, including `goal: ... completed at tick`, `phase 2` and `stock dropped after the
    removal for: [...]`.
  - [ ] `tui --frames` is the human's view of the same thing: `tui 7461 --frames 2 --z 9` against a live
    daemon prints the goal row (`goal 1 of 3: wood 0/10` at boot).
- [ ] **Task 7: deliberate RED and the mutation table**
  - [ ] NEW `mutations/12-12.sh`. Run each row alone after commit and name its killing test or the wire.
    Rows: progress counts loose items too (AC 2; the wire's `progress ... oracle` FAULT); progress counts a
    second item on a cell (AC 2 unit); progress counts carried items (AC 2 unit); a goal un-completes when
    stock drops (AC 3; the wire's phase 2 FAULT); order ignored (AC 3, AC 6; bloodstone is stored at tick
    6,352, before tetrahedrite completes at 9,828); no cascade (AC 3 unit); `to_save` drops `goals_completed` (AC 4);
    the load bound dropped (AC 4); the bridge reports `progress: 0` (AC 5); the Mirror drops `goals` (Task 5
    test); the tui row ignores `completed`; the gui readout ignores `completed`; the Task 2b entrance rule
    reverted (AC 11, the ring test).
  - [ ] One deliberate wire RED on the finished branch: rerun `goals_wire.py` with the "un-completes" row
    applied in a scratch worktree build, and paste its `FAULT` lines and `GOALS RED`.
- [ ] **Task 8: seat card** (AC 10)
  - [ ] NEW `12-12-signoff/vehicle-card.md`, in the seat's launch form. Use a fresh world, with fast4x allowed.
    Give the trees `[65,56,10] [61,74,13] [65,73,12]`, the tetrahedrite vein at `[77,64,11]` (its tunnel cells
    from the plan file) and the next one at `[69,79,14]`, and the bloodstone `[55,83,13]` with its tunnel. Use two
    or three compact piles near the camp, plus one short step where Wolf rings the fire with a pile (its 4
    neighbours left out) and sees it fill to the last cell (AC 11). Note the compass mirror (#169) and that a
    pre-12.12 save is refused.
- [ ] **Task 9: spine and gate**
  - [ ] `ARCHITECTURE-SPINE.md` (2026-08-09): one bullet, next to the 12.11 one: goals are sim state,
    progress is the stored count by 3.3's rule, the chain is sticky and cascades, and the wire carries
    `Goal` on snapshot and delta.
  - [ ] `RUST_TEST_THREADS=1 scripts/gate.sh`, full and green, before review.

## Dev Notes

### Scope guardrails (do NOT)

- No new command, refusal, key or tui input. Goals cannot be skipped, reset or reordered.
- No reward and no use of resources (PRD non-goal). No fail state; play never pauses or ends.
- No change to hauling, the pile fill order, digging or cutting beyond Task 2b's entrance rule in
  `pile_targets`. #186/#187/#199/#203/#204/#206 stay where they are.
- No goal logic in a client: clients print the wire's `goals`; "chain done" is "every goal completed"
  (D4), which is display, not a rule.
- No `Default` on wire types and no `#[serde(default)]` on `goals` or `goals_completed`.
- No new dependency.

### What already exists (build on it)

- The stored rule: `create_haul_jobs` (`lib.rs:389-410`). `uncarried_stones` (`:1271`) covers every kind
  and is ascending by id.
- Full-resend world state on the delta: `speed` (bridge `:83`); the refusal path through client-core is
  the client pattern (`refusal_text`, tui status `view.rs:435-438`, gui `designate.rs:53-82`).
- The HUD: `Hud` (`ingest.rs:1762`), hidden in captures (`hide_hud_for_capture`, `:2021-2028`).
- Save refusal pattern and tests: `save.rs:39-54`, `simd/src/main.rs:987-1038`.
- The plan and the oracle: `goals_wire.py`, `plan-10-10-1.json`.

### Key decisions & traps

1. **Count by the stored rule, not "on a pile cell".** A carrier standing on a free pile cell has his item's
   `Pos` there, and fell_tree drops several logs on one base cell. A naive `zones.contains(pos)` count (as
   `scenario.rs:2551-2555` does) over-counts.
2. **`Mirror::apply_delta`'s `..` swallows new fields.** It is why refusals never reach the Mirror. Name
   `goals` in the destructure.
3. **Order of the tick.** `advance_goals` after `carry_items`, so that a delivery's set-down in
   `execute_jobs` and the carry follow are both final before completion is judged; the wire is assembled after `step()`.
4. **Paused deltas** show live progress and no completion (D1). `goals_wire.py` never pauses, so its
   "completed on the first delta" check holds; any test that pauses must expect the lag.
5. **Stock leaves a pile in natural play:** a clearing miner may lift a stored item (`clearing_item`,
   `lib.rs:1852-1901`, picks among all uncarried items), and `settle` drops items whose floor is dug. This is
   why AC 3 is load-bearing.
6. **Two piles, two judges.** The chain test and `goals_wire.py` use non-enclosing piles (the plan's), so a
   regression of Task 2b cannot masquerade as a goals failure; only the ring test (AC 11) judges the seal.
7. **Snapshot literals.** About 66 sites; the field is required on purpose (AD-6 strictness). Budget for
   the edit; don't dodge it.

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs`, `crates/sim-core/src/save.rs`, `crates/sim-core/tests/scenario.rs`,
  `crates/sim-core/tests/save_load.rs`
- UPDATE `crates/protocol/src/lib.rs`, `crates/simd/src/bridge.rs`, `crates/simd/src/main.rs`,
  `crates/simd/tests/serve.rs`
- UPDATE `crates/client-core/src/lib.rs`, `crates/tui/src/view.rs`, `crates/tui/tests/client.rs`,
  `crates/gui/src/ingest.rs` (readout + census), `crates/gui/tests/headless.rs`, every other
  `Snapshot {`/`Delta {` literal
- UPDATE `ARCHITECTURE-SPINE.md` (2026-08-09)
- NEW `_bmad-output/implementation-artifacts/mutations/12-12.sh`, `12-12-signoff/vehicle-card.md`
- EXISTING (creation) `12-12-signoff/goals_wire.py`, `12-12-signoff/plan-10-10-1.json`, `12-12-signoff/draft.md`

### Previous story intelligence

- 12.11: a clearing miner moves items, the pile included, and a dig under a trunk fells it (extra logs).
  Disclose every moved pin; build probes in a scratch worktree; daemons run outside the repo (`simd` writes
  `frostvein.save` in its cwd) and are killed by PID.
- 12.10: the planner and the bloodstone `[55,83,13]` come from `ore_wire.py`; connect at once (#191).
- The gate needs `RUST_TEST_THREADS=1` on this box (about an hour).

### References

- `epics.md:2757-2782` (Story 12.12), `:2422-2434` (standing ACs), FR47/FR48 (`epics.md:116-117`), AD-8/AD-11
  (`epics.md:212-217`); PRD `prds/prd-frostvein-2026-09-28/prd.md:101-106, 151-158` (success criterion 3)
- `lib.rs:389-410` (stored rule), `:759-771` (`pile_targets`, Q3), `:2365-2387` (`carry_items`), `:2527-2571`
  (`assemble`, schedule), `:2640-2731` (save), `:3142, :3221, :3233` (readers)
- `protocol/src/lib.rs:262-301, 313-376`; `simd/src/bridge.rs:1-116`; `simd/src/main.rs:344-716, 987-1038`
- `client-core/src/lib.rs:10-27, 106-217`; `tui/src/view.rs:107-109, 235-509, 682, 1037`;
  `gui/src/ingest.rs:1762-1791, 4075, 4119, 6111-6142`; `gui/src/appearance.rs:454-466`

## Verification

**Instrument (Task 6), daemon-judged, against a FRESH release `simd`, run from outside the repo:**

```bash
simd 7461 &   # fresh, DEFAULT_SEED; cwd with no frostvein.save; connect at once
python3 _bmad-output/implementation-artifacts/12-12-signoff/goals_wire.py 7461 40000
```

- **Deliberate RED (observed at creation, 3 of 3 fresh daemons on `60955ef`, default targets 10/10/1):**
  `oracle: 10 wood stored by tick 1847` / `oracle: 1 bloodstone stored by tick 6352` /
  `oracle: 10 tetrahedrite stored by tick 9828` / `phase 2 at tick 9828: ... removing the pile round [67, 63,
  9] (8 counted items on it)` / `stock dropped after the removal for: ['wood', 'tetrahedrite']` /
  `goals: the daemon sent NO goals field (last tick 10028)` / `GOALS RED`, exit 1. Restore: none (main is
  the RED). The run is identical on every fresh daemon.
- **Required of dev:** 5 of 5 `GOALS OK`, each with three `goal: ... completed at tick` lines in chain
  order, `goal: chain completed at tick`, `goals: N deltas carried goals; 0 faults`, and a non-empty `stock
  dropped` line. Plus one `GOALS RED` from the "un-completes" mutation (Task 7) showing its `FAULT ...
  un-completed` line.
- **Sim:** `cargo test -p sim-core --test scenario the_default_world_completes_the_goal_chain`.
- **tui:** `tui 7461 --frames 2 --z 9` prints the goal row; at boot `goal 1 of 3: wood 0/10` (with Q1's
  numbers).

**Seat (AC 10), in the seat's launch form:**

- WSL: `simd 7451`, and in a second WSL terminal `tui 7451 --z 9` for parity.
- PowerShell: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`.
- Follow `12-12-signoff/vehicle-card.md` (Task 8).

## Dev Agent Record

### Agent Model Used

- Agent A (Claude Sonnet 5.5, claude-sonnet-5-5), Wolf's pick (2026-10-18): Tasks 1 (sim), 2b, 2, 3.

### Debug Log References

RED lines (run with `--ignored` on a stub `World::goals()` that returns the three goals with progress 0, none completed; the stub is replaced by Task 2):

- `goal_progress_counts_stored_items_only` (lib.rs unit): `left: [(0, false), (0, false), (0, false)]` / `right: [(2, false), (1, false), (0, false)]`, "progress is the stored count of each kind"
- `a_completed_goal_stays_completed_when_its_stock_leaves` (lib.rs unit): `left: (0, false)` / `right: (10, true)`
- `a_goal_completes_only_after_the_ones_ahead_of_it` (lib.rs unit): `left: [(0, false), (0, false), (0, false)]` / `right: [(0, false), (10, false), (1, false)]`, "bloodstone stored first must not complete"
- `the_chain_cascades_in_one_tick` (lib.rs unit): `left: [(0, false), (0, false), (0, false)]` / `right: [(9, false), (10, false), (1, false)]`
- `the_default_world_completes_the_goal_chain_with_scripted_commands` (scenario.rs): `tick 333: Wood progress 0 but 1 are stored`
- `same_seed_and_commands_remain_deterministic` (scenario.rs, the new `goals()` guard), on the original 21 x 21 x 21 pile: `no wood was ever stored in 400 ticks` (also none in 1,500)
- `loading_refuses_goals_completed_beyond_the_chain` (simd main.rs): `a save beyond the chain was loaded`
- `save_load_with_a_goal_done_and_the_next_in_progress_matches_never_saved` (save_load.rs): green on arrival (the field did not exist before Task 3); sabotage `to_save` writing `goals_completed: 0` turns it RED at `assertion left == right` (`completed: false` against `completed: true` on the loaded world)
- `loading_refuses_a_pre_12_12_save_without_goals_completed` (simd main.rs): green on arrival, for the same reason (the serde refusal needs the new field); its positive control loads the current save
- `a_pile_round_enclosed_ground_still_fills_every_cell` (scenario.rs, Task 2b, on `6dffa0d` pile_targets): `the ring sealed itself: 35 of 44 cells filled, last at tick 4245 (now 12000); 9 free pile cells [...] while 17 stones lie loose and 17 haul jobs retry` (a stall of 7,755 ticks, not a slow fill). The committed ring is the smaller one (Chebyshev 1..=2, 20 cells, 26 loose stones; the 44-cell ring took 195 s in debug): on `6dffa0d` it reads `the ring sealed itself: 16 of 20 cells filled, last at tick 1378 (now 9000); 4 free pile cells [(63,63,9) (65,63,9) (63,65,9) (65,65,9)] while 10 stones lie loose and 10 haul jobs retry`; fixed, 20 of 20 at tick 1,864 (16 s in debug).

### Completion Notes List

- **Task 2, goals.** `split_stored(ecs)` is the ONE copy of 3.3's stored rule (`create_haul_jobs` and the goals both call it); `stored_by_kind` counts it per kind; `advance_goals` runs after `carry_items` and cascades (D2); `World::goals()` reads progress live (D1, with the `// NOTE:`). `GoalsCompleted` is a resource, `assemble` takes `goals_completed` (0 from `generate`). `carry_items`' doc now says "LAST of the movers ... Only `advance_goals` follows it".
- **Task 1 / chain test (AC 6), numbers.** `the_default_world_completes_the_goal_chain_with_scripted_commands` measured completion at wood tick 1,847; tetrahedrite tick 9,828; bloodstone tick 9,828 (it was stored at 6,352, so it completes by the cascade in the same tick). These equal the wire oracle's ticks (`oracle: 10 wood stored by tick 1847`, `10 tetrahedrite ... 9828`). Bound pinned at 10,500: a margin of 672 ticks (6.8%), chosen because the unpaused wire form drifted to 10,454. The test takes about 206 s in debug (about 10,030 ticks plus 200). It checks, every tick, progress == the oracle's stored count for all three goals, no goal completed before the one ahead, no goal un-completing, and each completion at a stock that meets its target; 200 ticks after the chain the tick has climbed and all three are still done.
- **`same_seed_and_commands_remain_deterministic` changed beyond the `goals()` reader.** Its 21 x 21 x 21 pile from the origin stored no log in the 400-tick window (nor in 1,500), so the required "progress above 0" guard could not hold. I replaced that pile with a 3 x 3 pile at camp + (3..5, 3..5): wood is stored from tick 290 (4 logs by 432) and the test now runs in 2 s instead of 22 s. The test lost its giant-pile coverage; nothing else asserted on that pile in this test.
- **Task 3.** `SaveState.goals_completed` (no serde default), filled in `to_save`, destructured in `from_save`; simd refuses `goals_completed > GOALS.len()` before reading positions.
- **Task 2b tick cost: NOT measured here (the orchestrator measures it).** One thing to look at: `pile_targets` floods up to `4 x zone + 16` cells per distinct neighbouring ground component, so a very large zone (the 9,261-cell cube that `same_seed...` used to place) makes each call cost a flood of up to ~37,000 cells.

- **Task 2b, a finding: the first two forms of the entrance rule moved pins.** The creation's throwaway (a flood over same-z non-zone walkable ground, items blocking, cap 4 x zone + 16) moved 8 of 84 scenario tests (e.g. `a_stockpile_around_the_campfire_never_zones_or_receives_the_fire`: 0 pick-ups after t=2500 against the pinned 80; `a_full_stockpile_never_stacks_uncarried_stones`: zone (62,62,9) not full). Mechanism, from a dump of the 24-cell pile: the camp is a small hollow, walled by rock and trees, with fewer than 112 same-z walkable cells outside the zone, so every real entrance flooded dry under the cap and read as a pocket. Following ramps (`astar_neighbours`) fixed 7 of 8. The 8th (`a_carried_stone_is_always_set_down_beside_its_carrier`, delay 20, two stones landing 3 cells from the carrier at ticks 2337/2574) came from loose stones: they were walls in the flood, so ground fenced by loose stones read as a pocket and lost its depth-1 seeds, which changed the fill order (grids at tick 2300/2336 differ old vs new on row y=66). A third failure was the pre-commit gate's two `lib.rs` unit tests (`haul_work_positions_gate_both_legs_on_a_free_standable_pile_tile`, `a_stone_at_a_pit_rim_is_lifted_from_the_ramp_below`): on a tiny fixture every walkable cell outside the one-cell zone is fewer than the cap, so a plain cap reads ordinary ground as a pocket. The final rule floods with terrain only (no items, no dwarves; they move), over ramps, cap 4 x zone + 16, and a flood that runs dry is a pocket only if every cell beside it is a pile cell (rock, a drop or the map edge beside it makes it open ground). With it the lib unit tests (91 passed, 5 ignored = the RED goal tests) and the whole `scenario` suite is green (83 passed, 1 ignored = the chain test) and NO pin was re-pinned.

### File List

- `crates/sim-core/src/lib.rs` (goals, stored helper, `pile_targets` entrance rule, unit tests), `crates/sim-core/src/save.rs`, `crates/sim-core/tests/scenario.rs` (chain test, ring test, determinism reader), `crates/sim-core/tests/save_load.rs`
- `crates/simd/src/main.rs` (load bound and two tests)
- `_bmad-output/implementation-artifacts/12-12-the-first-goals.md`

## Change Log

| Date | Change |
|---|---|
| 2026-10-18 | Story created on `60955ef`: chain reachability measured on the wire (4 target sets), `goals_wire.py` RED 3/3 on main, look draft written, pile-ring defect found; Task 0 Q1-Q3 open. |
| 2026-10-18 | Task 0 Q1 RULED 10/10/1 and Q2 RULED (A), Wolf: "1 ok 2 ok". Q3 open. |
| 2026-10-18 | Task 0 Q3 RULED: fold the pile-ring defect in (Wolf: "fold it"); Task 2b, AC 11, a mutation row, a seat step. |
