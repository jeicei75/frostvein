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
- [x] **Task 2b: a pile that encloses ground still fills (Q3)** (AC 11)
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
  - [x] `pile_targets` runs on every haul claim and delivery (`lib.rs:1238`, `:2115`). Measure the max tick
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
- [x] **Task 4: wire** (AC 5)
  - [x] `protocol`: `Goal`, then `goals` on `Snapshot` (after `tick`) and on `Delta` (after `speed`). Re-pin
    `WIRE`/`DELTA_WIRE` (`protocol/src/lib.rs:313-334`) and the byte-exact delta at `:370`, and add one goal
    literal test.
  - [x] `simd/src/bridge.rs`: `snapshot()` (`:1-39`) and `delta()` (`:46-85`) fill `goals` from
    `world.goals()` through `item_kind_out` (`:109-116`). A bridge unit test checks goals against
    hand-written wire values.
  - [x] `serve.rs`: write a save with items on pile cells (a stacked pair on one cell included) and load it
    (`read_snapshot_after_load`, `:197`). The snapshot and the next delta carry `progress` above 0, and that
    equals the stored count the test computes from the same message's `items`, `zones` and `carrying`.
  - [x] Every `Snapshot {`/`Delta {` literal in the workspace gains the field. That is mechanical, about 66 Snapshot sites, mostly in gui
    tests. A shared test helper is fine; a `Default` derive on the wire types is not.
- [x] **Task 5: clients** (AC 7, 8)
  - [x] `client-core`: `Mirror` stores `goals` and gets a `goals()` getter. NAME `goals` in `apply_delta`'s
    destructure (`client-core/src/lib.rs:106-115`): its `..` would drop the field silently. Also set it in
    `replace` (`:195-217`). The goal text the two clients share lives here, next to `refusal_text`
    (`:10-27`), e.g. `goal_kind_text(ItemKind) -> &'static str` (`wood`, `tetrahedrite`, `bloodstone`) plus
    the line builders the draft needs. Add a test that a delta's goals reach the mirror.
  - [x] tui (`view.rs:235-489`): the goal row above the roster. Replace the three `3`s (`:249-250` map height,
    `tally_marks` `:107-109`, `move_cursor` `:682`) and the `h < 3` guard (`:245`) with ONE named constant.
    Re-pin `one_row_terminal_renders_blank` (`:1037`) and the pinned status strings, and disclose them. A
    `tui --frames` stub test in `tests/client.rs` (pattern: `capture_frames_refusing`, `:214-275`): goal row
    text before and after a delta that completes wood, and after the chain.
  - [x] gui: a `GoalReadout` Text modelled on `setup_name_readout` (`ingest.rs:1774-1791`, 22 px, `Hud`,
    `ClientLocal`, `GlobalZIndex(i32::MAX - 16)`) at bottom 16 px, right 16 px. It is updated from
    `mirror.0.goals()` with span colours as drafted (green 88,190,118; white; grey `TRADE_TEXT_COLOR`
    `appearance.rs:454`; gold `#D6A42C`). Re-pin the HUD census (`ingest.rs:4075`, `:4119`: 6 → 7) and the
    production text set (`:6111-6142`). Add a headless test that the spans change when the mirror's goals
    change (pattern: roster `:3721`). If Q2 is (B), add the banner and its own test.
- [x] **Task 6: the instrument** (AC 9)
  - [x] `12-12-signoff/goals_wire.py` (EXISTING, creation) is the instrument. Run it on 5 fresh release
    daemons, each from a directory with no `frostvein.save`, at most 4 at a time, with Q1's targets. Paste
    each verdict block, including `goal: ... completed at tick`, `phase 2` and `stock dropped after the
    removal for: [...]`.
  - [x] `tui --frames` is the human's view of the same thing: `tui 7461 --frames 2 --z 9` against a live
    daemon prints the goal row (`goal 1 of 3: wood 0/10` at boot).
- [x] **Task 7: deliberate RED and the mutation table**
  - [x] NEW `mutations/12-12.sh`. Run each row alone after commit and name its killing test or the wire.
    Rows: progress counts loose items too (AC 2; the wire's `progress ... oracle` FAULT); progress counts a
    second item on a cell (AC 2 unit); progress counts carried items (AC 2 unit); a goal un-completes when
    stock drops (AC 3; the wire's phase 2 FAULT); order ignored (AC 3, AC 6; bloodstone is stored at tick
    6,352, before tetrahedrite completes at 9,828); no cascade (AC 3 unit); `to_save` drops `goals_completed` (AC 4);
    the load bound dropped (AC 4); the bridge reports `progress: 0` (AC 5); the Mirror drops `goals` (Task 5
    test); the tui row ignores `completed`; the gui readout ignores `completed`; the Task 2b entrance rule
    reverted (AC 11, the ring test).
  - [x] One deliberate wire RED on the finished branch: rerun `goals_wire.py` with the "un-completes" row
    applied in a scratch worktree build, and paste its `FAULT` lines and `GOALS RED`.
- [x] **Task 8: seat card** (AC 10)
  - [x] NEW `12-12-signoff/vehicle-card.md`, in the seat's launch form. Use a fresh world, with fast4x allowed.
    Give the trees `[65,56,10] [61,74,13] [65,73,12]`, the tetrahedrite vein at `[77,64,11]` (its tunnel cells
    from the plan file) and the next one at `[69,79,14]`, and the bloodstone `[55,83,13]` with its tunnel. Use two
    or three compact piles near the camp, plus one short step where Wolf rings the fire with a pile (its 4
    neighbours left out) and sees it fill to the last cell (AC 11). Note the compass mirror (#169) and that a
    pre-12.12 save is refused.
- [x] **Task 9: spine and gate**
  - [x] `ARCHITECTURE-SPINE.md` (2026-08-09): one bullet, next to the 12.11 one: goals are sim state,
    progress is the stored count by 3.3's rule, the chain is sticky and cascades, and the wire carries
    `Goal` on snapshot and delta.
  - [x] `RUST_TEST_THREADS=1 scripts/gate.sh`, full and green, before review.

### Review Findings

Code review run 1 (2026-10-18, diff `60955ef..813c218`). There were four layers: Blind Hunter and Edge Case
Hunter on Sonnet, Acceptance Auditor and Feature Auditor on Opus. **No layer timed out.** Each layer built in
its own `/tmp/review-<layer>`.
- **Blind (sim-core, client-core):** ran every sim-core and client-core test, `scenario` included (84 ok).
- **Edge:** ran the protocol, simd, tui and client-core tests, a live daemon with `tui --frames`, and live save
  refusals at `goals_completed` = 4, -1, null, 2^32 and missing.
- **Acceptance:** ran every AC's pinning tests, `goals_wire.py` (5/5 `GOALS OK`), live AC 1 and AC 4 probes, and a
  scratch boulder probe.
- **Feature:** traced every hop and found all WIRED; ran `goals_wire.py` (3/3 `GOALS OK`), a save/load across
  daemons, and a replay of the seat card over the wire (sitting A completes, sitting B's ring fills 20/20 at tick
  2763).

**Coverage holes:**
- No layer ran the full gui headless/capture suites or the full simd serve suite. 8 gui goal/HUD tests and simd's
  goal and refusal tests ran; the full gate covers the rest on `c53c050`, and no code changed after it.
- The mutation table was not executed (reviews may not run payloads).
- AC 10 (seat) is open.
- After triage, `scripts/reap-build-caches.sh --tmp-only --force` reaped 9 directories under /tmp (39.1 GB listed) and reclaimed 20.3 GB of free space. Review cost row: $18.58, 404 turns (`49ee8e6`).

- [x] [Review][Decision] **AC 11 holds only while the enclosed hole touches no rock, drop or map edge** — RESOLVED (Wolf, 2026-10-18): option (a), close it now; became the Patch below. Severity
  MED. Sources: Acceptance (RAN) and Blind (read): a convergence.
  - **The rule:** `pile_targets`' flood (`crates/sim-core/src/lib.rs:842-846`) calls a non-pile ground cell "open"
    as soon as any flooded cell has a same-z side neighbour that is neither pile nor standable.
  - **What it does:** a hole inside a pile ring that also touches ONE rock cell, a pit or a drop is treated as a
    way in. The middle of the band fills first and seals the free cells, as on `6dffa0d`.
  - **Measured** (scratch probe, release): the committed ring test's setup, plus one `Solid(Stone)` at the fire's
    west neighbour inside the unzoned hole. Without the boulder: `free []`, 6 haul jobs. With it: `free
    [(63,63,9),(65,63,9),(63,65,9),(65,65,9)]`, `loose 10`, `haul jobs 10`, retrying forever. The failure is
    silent, the same class Q3 folded in.
  - **Disclosure:** the NOTE (`lib.rs:812-817`) discloses only "a hole in a pile on a ledge". The spine bullet
    (`ARCHITECTURE-SPINE.md:212`) and the Completion Notes state the fix with no limit.
  - **Not a regression:** on `main` every such ring sealed. The limit comes from the rule's design (agent A's form
    had it), not from the 17x-tick early exit.
  - **Options:**
    - **(a) Close it now.** A hole is a pocket when its flood runs dry, whatever borders it. A cheap early exit
      that keeps that: "open" once the flood leaves the zone's x/y bounding box, or hits the cap. A hole ringed
      by the pile cannot leave the box. Use the boulder probe as the RED, re-measure the tick cost against
      0.265 ms, and strengthen the ring test while there (see the deferred ring-test item).
    - **(b) Narrow and file.** Correct the NOTE, the spine bullet and the record to state the rock/drop limit, and
      file a `bug` + `route:story` issue.
- [ ] [Review][Patch] **Close AC 11 for holes that touch rock, a drop or the map edge** [`crates/sim-core/src/lib.rs:842-846`, NOTE `:812-821`] — Severity MED, from Decision 1 (a).
  - Change the rule so a hole is a pocket when its flood runs dry, whatever borders it. Candidate cheap exit: "open" once the flood leaves the zone's x/y bounding box or hits the cap.
  - RED first: the Acceptance Auditor's boulder probe (the committed ring setup plus one `Solid(Stone)` at the fire's west neighbour), committed as a scenario test.
  - Re-measure the release tick cost against 0.265 ms mean / 0 ticks > 10 ms.
  - Strengthen `a_pile_round_enclosed_ground_still_fills_every_cell` to count by `split_stored` and to assert AC 11's no-retry clause (the deferred ring-test item).
  - Add a mutation row and re-mutate.
  - Bring the NOTE, the spine bullet (`ARCHITECTURE-SPINE.md:212`) and the record into line.
- [ ] [Review][Patch] **The seat card leaves no spare tetrahedrite** [`12-12-signoff/vehicle-card.md:71-74`] —
  Severity MED. Source: Feature (RAN, card replayed over the wire).
  - The card digs exactly 10 ore cells (3+2+1+4) for a target of 10. "The next one at `[69,79,14]`" is one of the
    10.
  - The replay's tick 10000 read tetrahedrite 9, loose 0, marks 0; the 10th was in a hauler's hands.
  - One missed drag cell or one stranded ore stops the chain at 9/10 for good, and the card names no next step.
    Name one or two spare tetrahedrite cells, or the dig to make if it sits at 9.
- [ ] [Review][Patch] **The card's pile figures come from single-z rects; the gui's stockpile drag follows the
  surface** [`12-12-signoff/vehicle-card.md`; `crates/gui/src/designate.rs:193`] — Severity LOW, in the file
  patched above. Source: Feature (read).
  - The replay matched the card's 67 cells only because it also used single-z rects.
  - Say on the card that the cell counts are approximate at the seat.
- [ ] [Review][Patch] **Two record claims do not reproduce**
  [`12-12-the-first-goals.md:412`, `:431`] — Severity LOW. Patched rather than deferred: a false record invents
  scope later. Source: Acceptance (RAN) and Feature (RAN).
  - **The "runs in 2 s instead of 22 s" claim for the determinism test:** with the old 21x21x21 pile restored in a
    scratch copy, it runs 5.3 s on HEAD against 6.4 s on main (debug). Dropping the pile hid no flood cost; the
    stated reason is wrong.
  - **"Identical on all five ... 9817 deltas":** the 4 connect-at-once re-runs give 9814, with identical tick
    lines. The completion ticks depend on the connect tick: 2056/9618 at connect tick 1, and 1847/9828 at connect
    tick 6 (Feature's 3/3).
- [x] [Review][Defer] **AC 1 has no pinning test** [`crates/sim-core/tests/scenario.rs:5101`] — deferred.
  - Acceptance RAN a live probe: the snapshot and the first deltas carry progress 0 and none completed.
  - The chain test checks kind/target only, after 6 ticks with piles placed.
- [x] [Review][Defer] **AC 4's "logged reason" is not asserted** [`crates/simd/src/main.rs:1050`, `:1076`] —
  deferred. The tests check `is_none()` only. Edge and Acceptance both saw the log lines live (missing field; "save
  has 4 goals completed; the chain has 3").
- [x] [Review][Defer] **The Snapshot is pinned by an order-insensitive `Value` compare, not byte-exact**
  [`crates/protocol/src/lib.rs:523-527`] — deferred, pre-existing practice. The Delta is byte-exact.
- [x] [Review][Defer] **The real-daemon goal test never checks `completed`**
  [`crates/simd/tests/serve.rs:3243`] — deferred. `completed` is pinned in `bridge.rs` and in sim-core
  save_load.
- [x] [Review][Defer] **The ring test counts naively and skips AC 11's "no haul job retries forever"**
  [`crates/sim-core/tests/scenario.rs:5192`] — deferred.
  - It counts any item on a zone cell, carried items and stacks included, against Key decision 1.
  - Row 13 still kills it.
  - If Decision 1 is (a), the patch pass takes this with it.
- [x] [Review][Defer] **The tui goal row clips silently at narrow widths** [`crates/tui/src/view.rs:445`] —
  deferred. The longest line is 72 columns and fits 80.
- [x] [Review][Defer] **The gui goal readout may overlap the designate hint in a narrow window**
  [`crates/gui/src/ingest.rs:1932`] — deferred. Not observed; the readout is bottom-right and the hint is
  bottom-left.
- [x] [Review][Defer] **Per-tick item scans are unmeasured** [`crates/sim-core/src/lib.rs:409`, `:444`] —
  deferred. `split_stored` runs in `create_haul_jobs` and again in `advance_goals`, plus once per Snapshot/Delta
  via `goals()`. All of it is linear in items.
- [x] [Review][Defer] **`World::from_save` does not bound `goals_completed`** [`crates/sim-core/src/lib.rs:2818`]
  — deferred. simd is the only loader and bounds it at `crates/simd/src/main.rs:375`.
- [x] [Review][Defer] **`goal_lines`' `{:>2}` misaligns the `/` at progress 100 or more**
  [`crates/client-core/src/lib.rs:112`] — deferred, cosmetic.

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
- Agent B (Claude Sonnet 5.5, claude-sonnet-5-5), Wolf's pick (2026-10-18): Tasks 4, 5.

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

Task 4 (Agent B). None of these can be RED on unchanged code: each needs the new `goals` field to compile. Each is shown RED by a one-line sabotage of COMMITTED code (restored with `git checkout --`):

- `refusal_wire_is_literal_and_empty_delta_wire_is_unchanged` (protocol; WIRE/DELTA_WIRE/byte-exact delta re-pinned, plus a goals-before-refusals order assert): moving `goals` after `refusals` in `Delta` turns it RED at `goals sit after speed and before refusals` (the byte-exact delta alone could NOT see it: an empty `refusals` is skipped, so the bytes are identical; the order assert was added for that reason)
- `a_goal_is_literal_on_the_wire` (protocol): green on arrival; it shares `Goal` with the WIRE literals above, which carry the same four keys
- `snapshot_and_delta_carry_the_goal_chain_as_hand_written_wire_values` (simd bridge): `target: 1` in `goals_out` -> `left: [Goal { kind: Wood, target: 1, ...}, ...]` / `right: [Goal { kind: Wood, target: 10, ...}, ...]`
- `a_loaded_pile_reports_its_stored_count_as_goal_progress_in_the_snapshot_and_the_next_delta` (simd serve.rs): `progress: 0` in `goals_out` -> `assertion left == right failed: the stacked pair counts once` / `left: 0` / `right: 1`

Task 5 (Agent B). The text builders, the Mirror field and the gui list were stubbed (compiling, returning empty) and run RED first; the tui row test was RED on the unchanged render; the frames test needs the row, so it is green on arrival and shown RED by sabotage of committed code:

- `every_goal_kind_has_its_one_spelling` (client-core): `left: ""` / `right: "wood"`
- `the_goal_row_reads_as_the_approved_draft` (client-core): `left: ""` / `right: "goal 1 of 3: wood 4/10"`
- `the_goal_list_reads_as_the_approved_draft` (client-core): `left: []` / `right: [(Done, "[x] wood          10 / 10"), (Current, "[ ] tetrahedrite   3 / 10"), ...]`
- `a_deltas_goals_reach_the_mirror_and_a_snapshot_resets_them` (client-core): `left: []` / `right: [Goal { kind: Wood, target: 10, progress: 0, completed: false }, ...]` (stub Mirror); also RED by sabotage `self.goals = Vec::new()` in `apply_delta`: `left: []` / `right: [Goal { kind: Wood, target: 10, progress: 10, completed: true }, ...]`
- `the_goal_row_sits_above_the_roster_in_grey_and_pile_green` (tui view.rs), on the unchanged render: the row at y 4 read `"                                                            "` (blank, panicked at the first `starts_with` assert)
- `the_goal_row_changes_on_the_delta_that_changes_goals` (tui tests/client.rs): sabotage, `.filter(|goal| !goal.completed)` in `goal_row_text`: `left: ["goal 1 of 3: wood 4/10   done: wood 4/10  tetrahedrite 0/10  bloodstone 0/1", "goal 2 of 3: tetrahedrite 3/10   done: tetrahedrite 3/10  bloodstone 0/1", "all goals done - play on:"]` / `right: ["goal 1 of 3: wood 4/10", "goal 2 of 3: tetrahedrite 3/10   done: wood 10/10", "all goals done - play on: wood 12/10  tetrahedrite 10/10  bloodstone 1/1"]`
- `the_goal_list_follows_the_mirrors_goals_in_the_approved_colours` (gui ingest.rs lib test): stub spans `left: []` / `right: [("goals\n", [255, 255, 255]), ("[ ] wood           4 / 10\n", ...)]`; also RED by sabotage `GoalState::Done => Color::WHITE`: `left: [..., ("[x] wood          10 / 10\n", [255, 255, 255]), ...]` / `right: [..., ("[x] wood          10 / 10\n", [88, 190, 118]), ...]`

Re-pinned existing tests (Task 5), all disclosed: the tui footer grew from 3 rows to `FOOTER_ROWS` = 4, so a terminal `h` now leaves `h - 4` map rows. In `tui/src/view.rs`: `one_row_terminal_renders_blank` (blank at h 0..2, drawn at 3 -> blank at h 0..3, drawn at 4; a terminal under 4 rows is blank); 13 render tests bumped by one row of `h` to keep the same map rows (`a_named_dwarf_is_drawn_in_his_tunic_colour_and_a_nameless_one_in_his_job_colour`, `a_stone_on_a_stockpile_cell_draws_in_the_stockpile_colour`, `a_wood_item_draws_as_a_log_and_a_stored_one_in_the_stockpile_colour`, `an_ore_item_draws_its_own_glyph_and_a_stored_one_in_the_stockpile_colour`, `items_draw_only_on_the_viewed_level_and_a_shared_cell_draws_the_carrier`, `two_dwarves_on_one_cell_draw_the_crowd_glyph`, `entities_draw_only_on_the_viewed_level`, `out_of_world_cells_are_blank`, `peek_below_stops_at_three_levels`, `walking_and_idle_dwarves_render_different_colors`, and the three below), none changing an expected value. Three also moved a row index: `hint_bar_names_every_modes_keys_and_fits_eighty_columns` (hint read at row 2 -> 3), `status_line_fits_eighty_columns_without_truncation_at_large_ticks` (status read at row 1 -> 2), `renders_the_viewed_level` (its 35-cell expected framebuffer gained a 7-cell blank goal row after the map rows). In `gui/src/ingest.rs`: `h_hides_and_shows_every_hud_text` and `a_capture_hides_the_hud_and_h_cannot_restore_it` (HUD census 6 -> 7), `the_production_wiring_runs_every_call_run_makes_after_its_plugins` (production text set gained one more `""`, the goal root).

### Completion Notes List

- **Task 2, goals.** `split_stored(ecs)` is the ONE copy of 3.3's stored rule (`create_haul_jobs` and the goals both call it); `stored_by_kind` counts it per kind; `advance_goals` runs after `carry_items` and cascades (D2); `World::goals()` reads progress live (D1, with the `// NOTE:`). `GoalsCompleted` is a resource, `assemble` takes `goals_completed` (0 from `generate`). `carry_items`' doc now says "LAST of the movers ... Only `advance_goals` follows it".
- **Task 1 / chain test (AC 6), numbers.** `the_default_world_completes_the_goal_chain_with_scripted_commands` measured completion at wood tick 1,847; tetrahedrite tick 9,828; bloodstone tick 9,828 (it was stored at 6,352, so it completes by the cascade in the same tick). These equal the wire oracle's ticks (`oracle: 10 wood stored by tick 1847`, `10 tetrahedrite ... 9828`). Bound pinned at 10,500: a margin of 672 ticks (6.8%), chosen because the unpaused wire form drifted to 10,454. The test takes about 206 s in debug (about 10,030 ticks plus 200). It checks, every tick, progress == the oracle's stored count for all three goals, no goal completed before the one ahead, no goal un-completing, and each completion at a stock that meets its target; 200 ticks after the chain the tick has climbed and all three are still done.
- **`same_seed_and_commands_remain_deterministic` changed beyond the `goals()` reader.** Its 21 x 21 x 21 pile from the origin stored no log in the 400-tick window (nor in 1,500), so the required "progress above 0" guard could not hold. I replaced that pile with a 3 x 3 pile at camp + (3..5, 3..5): wood is stored from tick 290 (4 logs by 432). (Corrected at review run 1: this said "the test now runs in 2 s instead of 22 s", which does not reproduce. With the old pile restored in a scratch copy the test runs 5.3 s on HEAD against 6.4 s on `main`, debug, so dropping the pile hid no flood cost; the reason for the change is the "progress above 0" guard alone.) The test lost its giant-pile coverage; nothing else asserted on that pile in this test.
- **Task 3.** `SaveState.goals_completed` (no serde default), filled in `to_save`, destructured in `from_save`; simd refuses `goals_completed > GOALS.len()` before reading positions.
- **Task 2b tick cost: NOT measured here (the orchestrator measures it).** One thing to look at: `pile_targets` floods up to `4 x zone + 16` cells per distinct neighbouring ground component, so a very large zone (the 9,261-cell cube that `same_seed...` used to place) makes each call cost a flood of up to ~37,000 cells.
- **Task 2b tick cost (orchestrator, measured): agent A's rule cost 17x the tick; an early exit restores it.** Release, a scratch probe (not committed) timing every `World::step` over 10,000 ticks of `plan-10-10-1.json` from tick 6, each build from its own scratch worktree:
  - before (`60955ef`): mean 0.170 / 0.171 ms, p99 0.423 / 0.415 ms, max 8.97 / 9.09 ms, 0 ticks over 10 ms (2 runs; 3 earlier runs: max 8.87-9.12 ms, 0 over 10 ms).
  - agent A's rule (`736da92`): mean 2.88 / 2.85 ms, p99 5.15 / 5.10 ms, max 13.1 / 13.0 ms, **8 ticks over 10 ms** per run. Every haul claim and delivery re-flooded each ground component to the cap.
  - fix (orchestrator): the flood stops at the first cell beside a same-z neighbour that is neither pile nor standable. Same answer as agent A's rule (a same-z standable neighbour is always reached by the terrain-only flood, so "beside a non-pile cell the flood never reached" IS "beside a non-pile, non-standable cell"), decided early. After: mean 0.265 ms, p99 0.615 ms, max 8.83 ms, 0 over 10 ms.
  - worst case, a 518-cell pile (camp +-20, z +-3) on top of the plan, 5,000 ticks: before mean 3.68 ms, p99 7.11, max 14.8, 32 over 10 ms; after 3.42 / 6.76 / 15.3, 31 over 10 ms. A big pile is ALREADY that slow on `main`; this story neither causes nor fixes it.
- **Task 2b on real terrain (orchestrator).** The ring test flattens the ground round the camp, so the fix was also checked on the real shape: the 58-cell zone of the creation run that sealed (run 7476's save) placed fresh on DEFAULT_SEED with the plan's digs and cuts. `60955ef`: 50 of 58 filled, 8 items loose, unchanged from tick 9,006 to 12,006. `736da92` and the early-exit fix: 58 of 58 at tick 10,006, 0 loose. (Loading the sealed SAVE itself stays at 50/58 on every rule: its 8 free cells are reachable only through filled cells, the documented no-target case; no entrance rule can reopen a pile that already sealed.)

- **Task 2b, a finding: the first two forms of the entrance rule moved pins.** The creation's throwaway (a flood over same-z non-zone walkable ground, items blocking, cap 4 x zone + 16) moved 8 of 84 scenario tests (e.g. `a_stockpile_around_the_campfire_never_zones_or_receives_the_fire`: 0 pick-ups after t=2500 against the pinned 80; `a_full_stockpile_never_stacks_uncarried_stones`: zone (62,62,9) not full). Mechanism, from a dump of the 24-cell pile: the camp is a small hollow, walled by rock and trees, with fewer than 112 same-z walkable cells outside the zone, so every real entrance flooded dry under the cap and read as a pocket. Following ramps (`astar_neighbours`) fixed 7 of 8. The 8th (`a_carried_stone_is_always_set_down_beside_its_carrier`, delay 20, two stones landing 3 cells from the carrier at ticks 2337/2574) came from loose stones: they were walls in the flood, so ground fenced by loose stones read as a pocket and lost its depth-1 seeds, which changed the fill order (grids at tick 2300/2336 differ old vs new on row y=66). A third failure was the pre-commit gate's two `lib.rs` unit tests (`haul_work_positions_gate_both_legs_on_a_free_standable_pile_tile`, `a_stone_at_a_pit_rim_is_lifted_from_the_ramp_below`): on a tiny fixture every walkable cell outside the one-cell zone is fewer than the cap, so a plain cap reads ordinary ground as a pocket. The final rule floods with terrain only (no items, no dwarves; they move), over ramps, cap 4 x zone + 16, and a flood that runs dry is a pocket only if every cell beside it is a pile cell (rock, a drop or the map edge beside it makes it open ground). With it the lib unit tests (91 passed, 5 ignored = the RED goal tests) and the whole `scenario` suite is green (83 passed, 1 ignored = the chain test) and NO pin was re-pinned.
- **Task 4 (Agent B), the wire.** `protocol::Goal` plus `goals` on `Snapshot` (after `tick`) and `Delta` (after `speed`, before `refusals`), no `Default`, no serde default, always serialized. `bridge::goals_out` is the one mapper, used by `snapshot()` and `delta()`. 90 `Snapshot {`/`Delta {` literals gained `goals: Vec::new()` (a script that matched braces; `cargo check --workspace --all-targets` then passed), and the JSON fixtures that are parsed (client-core `tests/mirror.rs`, tui `main.rs`, gui `ingest.rs`) gained `"goals":[]`; the python bench fixtures are not parsed by Rust and were left. The serve test uses two zone cells at the dwarves' tiles (a pair of wood on one, tetrahedrite alone on the other) and computes the stored count from the SAME message's `items`/`zones`/`carrying`. Mutation row re-pointed: `2-3-master-of-time.sh` "delta bridge hardcodes normal speed" (its anchor `speed,\n\n        refusals` became `speed,\n        goals: goals_out(world),\n        refusals`).
- **Task 5 (Agent B), the clients.** client-core: `Mirror` stores `goals` (named in `apply_delta`'s destructure and set in `replace`) with a `goals()` getter; `goal_kind_text`, `goals_all_done`, `goals_title_text`, `goal_lines` (the gui list, with a `GoalState`) and `goal_row_text` (the tui row as a grey `head` and a green `done`) sit above `Changes`, beside `refusal_text`. They only format the daemon's `goals`; "all done" is every `completed` (display, D4), and the "current" goal is the first not completed. tui: `FOOTER_ROWS` (4) replaces the three `3`s (`tally_marks`, `render`'s `h < 3` guard and `map_h`, `move_cursor`); the goal row sits at `h - 4`. Two choices the draft left open: the chain-done line is `all goals done - play on: ` in grey and the counts in the pile green (the whole done part is green), and the gui title `goals` is white. gui: `GoalReadout` (bottom 16 px, right 16 px, `Hud`, `ClientLocal`, `GlobalZIndex(i32::MAX - 16)`), spans rebuilt only when they change, colours in `appearance.rs` (`GOAL_DONE_COLOR` 88,190,118; `GOAL_ALL_DONE_COLOR` D6A42C; current white; later `TRADE_TEXT_COLOR`). Numbers in the lines follow the draft's columns exactly (`{kind:<12}  {progress:>2} / {target}`). The gui test lives in `ingest.rs` beside the roster test (the "headless" pattern of `:3721`), not in `tests/headless.rs`.

- **Task 6, the instrument (orchestrator): `GOALS OK` on 5 of 5 fresh release daemons** (`985b7b7`, built in a scratch worktree, each daemon in its own empty directory, 4 then 1 at a time, ports 7511-7515, every daemon reaped by PID, `delta queue full` 0 times in every daemon log). The verdict block, identical on all five:
  ```
  wood: oracle reached 10 at tick 2056; goal completed at tick 2056
  tetrahedrite: oracle reached 10 at tick 9618; goal completed at tick 9618
  bloodstone: oracle reached 1 at tick 6566; goal completed at tick 9618
  goals: 9817 deltas carried goals; 0 faults
  play continued to tick 9818 (200 ticks after the chain)
  stock dropped after the removal for: ['wood', 'tetrahedrite', 'bloodstone']
  GOALS OK
  ```
  Only the end-state lines vary (two variants): runs 7511/7515 leave 4 tetrahedrite loose, 7512-7514 leave 3, with the haulers at different cells. The wire's ticks differ from the chain test's (1,847 / 9,828) because the daemon's connect snapshot is now tick 1 (tick 6 at creation): the crew stands elsewhere, so 6 of the 78 pile cells differ (`--plan-out` diffed against `plan-10-10-1.json`: digs and cuts identical). The sim itself did not move: a release probe of the scenario path on `985b7b7` and on `736da92` both give wood 1,847, tetrahedrite 9,828, bloodstone 9,828 (stored 6,352). (Corrected at review run 1: the `deltas carried goals` count is not fixed across runs either. The reviewers' 4 connect-at-once re-runs gave 9814 with identical tick lines, and the completion ticks follow the connect tick: 2,056 / 9,618 at connect tick 1, 1,847 / 9,828 at connect tick 6, the Feature Auditor's 3 of 3.)
  `tui 7521 --frames 2 --z 9` against a fresh daemon prints `goal 1 of 3: wood 0/10` in the trade grey (150,160,170) at tick 1 and tick 2.
- **Orchestrator, Task 5:** the goal row was drawn at `h - 4`, a second literal beside `FOOTER_ROWS`; now `h - FOOTER_ROWS`.

- **Task 7, the mutation table (orchestrator): `mutations/12-12.sh`, 15 rows, all KILLED** (run alone after commit, `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh`). The first run on `86f369a` killed 13 and found TWO SURVIVORS, both weak tests, both fixed in `1ff0a19` and re-mutated alone:
  - row 3 "progress counts carried items": `goal_progress_counts_stored_items_only`'s carried clause was vacuous, because the step's `carry_items` moved the carried item onto its carrier, off the pile. The carrier now stands on the pile cell and the test reads progress before the step; KILLED at the new assert `a carried item on a pile cell is not stored` (not by an earlier one).
  - row 9 "the bridge reports progress 0": the bridge unit test read a fresh world, where every progress is 0 and nothing is completed. It now loads a save with 10 wood and 1 tetrahedrite stored and `goals_completed: 1`, expecting (10, done) (1, not) (0, not); KILLED. (9b, the same sabotage against `serve.rs`, was KILLED on the first run.)
  Killers: 1, 2, 3 `goal_progress_counts_stored_items_only`; 4 `a_completed_goal_stays_completed_when_its_stock_leaves`; 5 `a_goal_completes_only_after_the_ones_ahead_of_it`; 5b the chain test; 6 `the_chain_cascades_in_one_tick`; 7 `save_load_with_a_goal_done_and_the_next_in_progress_matches_never_saved`; 8 `loading_refuses_goals_completed_beyond_the_chain`; 9 the bridge test; 9b `a_loaded_pile_reports_its_stored_count_as_goal_progress_in_the_snapshot_and_the_next_delta`; 10 `a_deltas_goals_reach_the_mirror_and_a_snapshot_resets_them`; 11 the tui `--frames` test `the_goal_row_changes_on_the_delta_that_changes_goals`; 12 `the_goal_list_follows_the_mirrors_goals_in_the_approved_colours`; 13 `a_pile_round_enclosed_ground_still_fills_every_cell`.
- **The five older rows re-pointed by agents A and B, re-run alone:** 4 KILLED (2.4 x3, 2.3 "delta bridge hardcodes normal speed"). **3.3's "every stone on a zone tile counts as stored" SURVIVED, and it ALREADY survives on `main`:** the same sabotage, applied by hand to `60955ef` and to `86f369a`, leaves `two_carriers_racing_for_the_last_tile_do_not_leave_a_permanent_stack` green (1 passed on both). Not a 12.12 regression and not fixed here (3.3's test, out of scope); the clause it sabotages is now killed by 12.12 row 2.
- **Task 7, deliberate wire RED (orchestrator):** row 4 (`let mut done = 0;`) built in a scratch worktree at `86f369a`, one fresh release daemon: the chain completes at tick 9,618, then phase 2 removes a pile and `FAULT tick 9624: wood un-completed (completed at 2056)`, `FAULT tick 9624: tetrahedrite un-completed (completed at 9618)`, `FAULT tick 9624: bloodstone un-completed (completed at 9618)` (and every tick after), `GOALS RED`, exit 1, `delta queue full` 0.
- **Task 8, seat card (orchestrator):** `12-12-signoff/vehicle-card.md`, the plan's digs as 10 row drags, 3 rect piles near the fire (67 cells), the 3 cuts, and a sitting B ring (5 x 5 round the fire, its 4 side neighbours cleared: 20 cells). A scratch release probe of exactly those drags from tick 30: sitting A wood at tick 1,957, tetrahedrite and bloodstone at 10,294, every mark gone; sitting B 20 of 20 filled by tick 3,030.

- **Task 9, the full gate (orchestrator), on `c53c050`, `RUST_TEST_THREADS=1 scripts/gate.sh`, rc 0:**

  ```
  cargo fmt --check           ok  0s
  cargo clippy -D warnings    ok  2s
  cargo test                  ok  773s
  cargo test (pixel guards)   ok  3101s
  tui has no sim-core edge                ok
  client-core has no sim-core edge        ok
  gui has no sim-core edge                ok
  metrics ledger tests        ok  0s
  bench tests                 ok  19s
  mutation tables still apply ok  10s
  GATE GREEN  3914s
  ```

- **Review run 1 patch pass (2026-10-19, Opus, fresh session).** All 4 `[Review][Patch]` items.
  - **AC 11 closed for holes beside rock, a drop or the map edge** (`b3a9753`, `97f153d`). `pile_targets`' flood no
    longer calls a hole open when a flooded cell touches a cell that is neither pile nor standable. It is open only
    when it reaches a cell outside the zones' x/y bounding box, or the cap (4 x zone + 16). A hole the pile rings
    cannot leave the box, so it is a pocket whatever else borders it. The NOTE states the one limit left: a ringed
    hole bigger than the cap (a one-cell-wide ring about 17 cells across) still reads as open.
  - **RED:** `a_pile_round_ground_beside_rock_still_fills_every_cell` (scenario.rs; the ring of
    `a_pile_round_enclosed_ground_still_fills_every_cell` plus one `Solid(Stone)` at the fire's west neighbour) on
    `f7df25b`: `the ring sealed itself: 16 of 20 cells stored, last at tick 1451 (now 6050); 4 free pile cells
    [(63,63,9), (65,63,9), (63,65,9), (65,65,9)] while 10 haul jobs retry`, the Acceptance Auditor's figures.
  - **Ring test strengthened** (both ring tests share `assert_ring_fills`): it counts stored items by 3.3's rule
    (one uncarried item per pile cell, `uncarried_item_tiles`), and it asserts AC 11's no-retry clause directly:
    at most 100 ticks with a haul job in retry cooldown while a pile cell is free. On the fix: 0 ticks (plain ring)
    and 20 (boulder ring). On the old rule with the earlier `free.is_empty()` assert stubbed (scratch worktree):
    `haul jobs sat in retry cooldown for 4569 ticks while a pile cell was free`. That shows the new assert kills on
    its own.
  - **Mutation row 14** "a hole beside rock reads as open again" (the reviewed rule restored): KILLED by the boulder
    test at `scenario.rs:5272`. Row 13 still KILLED by the plain ring test at the same line. 16 rows in the table.
  - **Tick cost** (release, scratch worktrees, every `World::step` over 10,000 ticks of `plan-10-10-1.json` from
    tick 6, 2 runs each). Old rule (`f7df25b`): mean 0.261 / 0.262 ms, p99 0.600 / 0.624, max 9.09 / 9.88,
    0 ticks over 10 ms. New rule (`b3a9753`): mean 0.297 / 0.295 ms, p99 0.734 / 0.674, max 9.06 / 9.61, 0 over
    10 ms. That is +13% on the mean (0.296 against 0.262 ms; the dev record's reference was 0.265), with p99 and
    max about the same, still 0 over 10 ms, and far from the 17x of agent A's rule. Goals end the same (12/10/1, all done). Worst case, the 518-cell pile (camp +-20, z +-3) over 5,000
    ticks: old 3.35 ms mean, p99 6.64, max 14.8, 31 over 10 ms; new 1.62 ms, p99 4.65, max 12.7, 16 over 10 ms.
  - **Seat card** (`09139bb`): the spare tetrahedrite is `[45, 83, 12]`, just west of the South-west run's last cell.
    It is the only one within 4 cells of any run (scratch probe on DEFAULT_SEED). The plan's 46 digs plus that cell
    give 11 tetrahedrite items with every mark dug, against 10 without it. The card says its pile cell counts come
    from one-level rects and are approximate at the seat.
  - **Record:** the two claims that did not reproduce are corrected in place (the determinism test's run time, the
    deltas count). The spine bullet and the `pile_targets` NOTE state the rule as it now is.

### File List

- `crates/sim-core/src/lib.rs` (goals, stored helper, `pile_targets` entrance rule, unit tests), `crates/sim-core/src/save.rs`, `crates/sim-core/tests/scenario.rs` (chain test, ring test, determinism reader), `crates/sim-core/tests/save_load.rs`
- `crates/simd/src/main.rs` (load bound and two tests)
- (Agent B, Task 4) `crates/protocol/src/lib.rs`, `crates/simd/src/bridge.rs`, `crates/simd/tests/serve.rs`, `_bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh`, and the `Snapshot`/`Delta` literals in `crates/client-core`, `crates/tui`, `crates/gui` (src and tests)
- (Agent B, Task 5) `crates/client-core/src/lib.rs`, `crates/client-core/tests/mirror.rs`, `crates/tui/src/view.rs`, `crates/tui/src/main.rs`, `crates/tui/tests/client.rs`, `crates/gui/src/ingest.rs`, `crates/gui/src/appearance.rs`
- (Agent B, Task 4, literals) also `crates/gui/src/capture.rs`, `crates/gui/src/pick.rs`, `crates/gui/src/project.rs`, `crates/gui/tests/capture.rs`, `crates/gui/tests/headless.rs`
- (Agents A and B, re-pointed rows) `_bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh`, `_bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh`
- `_bmad-output/implementation-artifacts/mutations/12-12.sh` (new, 15 rows; 16 after review run 1)
- `_bmad-output/implementation-artifacts/12-12-signoff/vehicle-card.md` (new)
- `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/metrics/12-12-the-first-goals.md`
- `_bmad-output/implementation-artifacts/12-12-the-first-goals.md`

## Change Log

| Date | Change |
|---|---|
| 2026-10-18 | Story created on `60955ef`: chain reachability measured on the wire (4 target sets), `goals_wire.py` RED 3/3 on main, look draft written, pile-ring defect found; Task 0 Q1-Q3 open. |
| 2026-10-18 | Task 0 Q1 RULED 10/10/1 and Q2 RULED (A), Wolf: "1 ok 2 ok". Q3 open. |
| 2026-10-18 | Task 0 Q3 RULED: fold the pile-ring defect in (Wolf: "fold it"); Task 2b, AC 11, a mutation row, a seat step. |
| 2026-10-18 | Dev (Sonnet 5.5 agents A and B, Opus orchestrating): Tasks 1-9 done. Goals in sim-core, on the wire, in both clients; pile-ring fix (early-exit flood after a 17x tick regression); mutation table 15/15 KILLED; wire 5/5 OK and a deliberate RED; seat card. Full gate GREEN on `c53c050`. Status -> review. |
| 2026-10-18 | Code review run 1 on `813c218` (4 layers, none timed out): 0 HIGH. 1 MED decision: AC 11 seals again when the hole touches rock or a drop. Wolf ruled (a), close it now. 4 patches left as action items; 10 defer. Status -> in-progress. |
