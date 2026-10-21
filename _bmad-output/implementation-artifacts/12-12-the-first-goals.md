---
baseline_commit: 60955ef
model: claude-opus-5-5  # session default, same as 12.1-12.11's creation
---

# Story 12.12: The First Goals

Status: done

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
- [x] [Review][Patch] **Close AC 11 for holes that touch rock, a drop or the map edge** [`crates/sim-core/src/lib.rs:842-846`, NOTE `:812-821`] — Severity MED, from Decision 1 (a).
  - Change the rule so a hole is a pocket when its flood runs dry, whatever borders it. Candidate cheap exit: "open" once the flood leaves the zone's x/y bounding box or hits the cap.
  - RED first: the Acceptance Auditor's boulder probe (the committed ring setup plus one `Solid(Stone)` at the fire's west neighbour), committed as a scenario test.
  - Re-measure the release tick cost against 0.265 ms mean / 0 ticks > 10 ms.
  - Strengthen `a_pile_round_enclosed_ground_still_fills_every_cell` to count by `split_stored` and to assert AC 11's no-retry clause (the deferred ring-test item).
  - Add a mutation row and re-mutate.
  - Bring the NOTE, the spine bullet (`ARCHITECTURE-SPINE.md:212`) and the record into line.
- [x] [Review][Patch] **The seat card leaves no spare tetrahedrite** [`12-12-signoff/vehicle-card.md:71-74`] —
  Severity MED. Source: Feature (RAN, card replayed over the wire).
  - The card digs exactly 10 ore cells (3+2+1+4) for a target of 10. "The next one at `[69,79,14]`" is one of the
    10.
  - The replay's tick 10000 read tetrahedrite 9, loose 0, marks 0; the 10th was in a hauler's hands.
  - One missed drag cell or one stranded ore stops the chain at 9/10 for good, and the card names no next step.
    Name one or two spare tetrahedrite cells, or the dig to make if it sits at 9.
- [x] [Review][Patch] **The card's pile figures come from single-z rects; the gui's stockpile drag follows the
  surface** [`12-12-signoff/vehicle-card.md`; `crates/gui/src/designate.rs:193`] — Severity LOW, in the file
  patched above. Source: Feature (read).
  - The replay matched the card's 67 cells only because it also used single-z rects.
  - Say on the card that the cell counts are approximate at the seat.
- [x] [Review][Patch] **Two record claims do not reproduce**
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

Code review run 2 (2026-10-19, diff `813c218..6aa424d`, session c09dd276). Four layers: Blind Hunter and Edge Case
Hunter on Sonnet, Acceptance Auditor and Feature Auditor on Opus. **No layer timed out**, every layer ran
`cargo 1.97.1` in its own `/tmp/review-<layer>`, and no layer touched the repo.
- **Blind (sim-core):** release probe comparing the new and old `pile_targets` on synthetic layouts.
- **Edge (tests, mutations):** both ring tests instrumented; rows 13 and 14 applied by hand in a `/tmp` copy, with
  and without the free-cell assert stubbed.
- **Acceptance:** audited the closure table; the boulder test on `f7df25b` (RED at `scenario.rs:5272`); trajectory
  hashes on main, `f7df25b` and HEAD; tick cost; the spare census; sitting B and `goals_wire.py` on HEAD.
- **Feature:** traced every hop and found all WIRED; ran live A/B daemons (HEAD against `813c218`) for sitting B,
  a boulder loaded from a save, a pit in the hole, `goals_wire.py`, and spread multi-pile layouts.

**Delta against run 1:** run 1 had 0 HIGH, 1 MED decision, 3 patch, 10 defer. Run 2 NEW: **1 HIGH** (decision), 1
MED patch, 1 LOW patch, 2 defer, 1 dismissed. **REWORK: closure row 1 (AC 11 beside rock).** The fill is closed for
its fixture, but the patch item's "re-measure the tick cost" was measured on the compact layout only. The rule's
NOTE and spine bullet also claim more than the code does. Rows 2-5 are CLOSED (Acceptance RAN each one: 4569 retry
ticks on the old rule with the free-cell assert stubbed; row 14 kills at `:5272`; plan plus spare gives 11
tetrahedrite and 12/11/1; both record claims corrected). **The stopping rule does NOT fire: a HIGH is among the new
findings.**

**Coverage holes:** no gui run, so AC 8's visual half and the seat drag geometry for AC 11 are unverified; the
mutation table was applied by hand to rows 13 and 14 only; AC 10 rests on the recorded seat.
After triage, `scripts/reap-build-caches.sh --tmp-only --force` reaped 11 directories under /tmp (7.8 GB listed)
and reclaimed 6.0 GB of free space. Review cost row: $16.81, 290 turns (`87de117`).

- [x] [Review][Decision] **The box exit makes `pile_targets` several times slower once piles are spread out** —
  RESOLVED (Wolf, 2026-10-19): option (a), fix now; became the first Patch below. **RE-RULED at the patch pass
  (Wolf, 2026-10-19, "b"): per-pile boxes (a) cut the cost but resealed the campfire ring in a compact layout; the
  fix landed instead as one box plus a cache of the floods' answers, see the run 2 patch pass.** Severity HIGH. Sources: Acceptance (RAN), Feature (RAN), Blind (RAN, synthetic): a three-way convergence.
  - **Where:** `crates/sim-core/src/lib.rs:828-832` takes one box over the union of ALL zones. `:845` exits only
    outside it or at the cap, `4 x all zone cells + 16`. The memo short-circuits on the START cell only
    (`:837`). It is called twice per claimable haul job per tick (`claim_jobs` `:582`/`:585`, via `work_positions`
    `:1334`) and on every delivery (`:2211`).
  - **Why:** once piles sit apart, ground round each pile lies inside the box, so every inward flood runs to the
    cap. The old rule stopped at the first cell beside rock, which real terrain is full of.
  - **Measured, same fill trajectory in every A/B (cost only, not behaviour):**
    - Acceptance: `plan-10-10-1.json` plus three 2x2 piles far off. HEAD **2.39 ms** mean (p99 4.6), old rule
      0.67, main 0.24.
    - Feature, live daemon: 5 x 25-cell piles near the corners. **30.95 against 6.14 cpu-ms/tick mean; fast4x 17
      t/s against 95.**
    - Feature: the same 5 piles in a compact box. 13.81 against 7.15 mean, 48-66 t/s against 104-128.
  - **Why it shipped:** the patch pass re-measured `plan-10-10-1` (+13%) and one contiguous 518-cell pile (faster)
    only, the two layouts where the box exit fires at once. This is the 17x class from Task 2b again, with every
    test green.
  - **Options:**
    - **(a) Fix now.** Give each flood the x/y box of the pile COMPONENT it borders (4-connected same-z pile
      cells), memoised per component, and end a flood when it reaches a cell this call already classified. The
      RED is a measurement: the far-pile and 5 x 25 layouts against the old rule, plus both ring tests and rows 13
      and 14. Two piles jointly ringing a hole need a test. UNTESTED.
    - **(b) Cache `pile_targets` once per tick.** A bigger change: `blocked` moves within a tick as stones land.
    - **(c) Accept and file.** Fill is correct; file a `bug` + `route:story` issue with these numbers for the
      performance story Wolf has pencilled in (#204/#206), and state the cost in the NOTE, spine and card.
- [x] [Review][Patch] **Bound each entrance flood by its own pile's box, not one box over every pile**
  [`crates/sim-core/src/lib.rs:828-857`] — Severity HIGH, from Decision 1 (a). LANDED as re-ruled (b): one box plus
  the floods' answers kept until a tile or the zones change (`25fe2da`).
  - Give each flood the x/y box of the pile component it borders (4-connected same-z pile cells), memoised per
    component. End a flood when it reaches a cell this call already classified.
  - RED first, as a measurement: the far-pile layout (`plan-10-10-1.json` plus three 2x2 piles at `[104,104,13]`,
    `[24,104,19]`, `[94,64,25]`) and 5 x 25-cell piles spread and compact, against the old rule (`f7df25b`) and
    HEAD. Re-measure the compact plan against 0.296 ms too.
  - Both ring tests and rows 13 and 14 must stay green or KILLED. Add a test for two piles that jointly ring a hole,
    and a mutation row that puts back the single union box.
- [x] [Review][Patch] **The NOTE and spine bullet overstate the entrance rule**
  [`crates/sim-core/src/lib.rs:814-821`, `ARCHITECTURE-SPINE.md:213-215`] — Severity MED. Sources: Acceptance (RAN),
  Blind (read), Edge (read).
  - **"A hole the pile rings cannot leave the box" / "only a ringed hole bigger than the cap reads as open" is
    false.** The box ignores z and the flood follows ramps. Acceptance RAN the fire ring with the west hole cell as
    a ramp down to a CLOSED one-wide cellar running west out of the box: HEAD seals at **16/20**, 4 free cells, 4569
    retry ticks. With the cellar ending inside the box it fills 20/20. The layout is contrived; the claim is the
    defect.
  - **"One that runs dry is a pocket, whatever borders it"** misses a pile-plus-rock pocket that reaches past the
    box. Acceptance RAN a 4-cell pile across a one-wide dead-end tunnel ending one cell past the box: 1/4 stored,
    5753 retry ticks, the same on HEAD, `f7df25b` and main (not a regression).
  - **"About 17 cells across"** is wrong: a one-wide square ring hits the cap from outer side 20 (hole 18 across),
    and the cap counts every pile on the map, not the ring alone.
  - State the rule as it is, with the limits. If Decision 1 is (a), write the NOTE for the new rule instead.
- [x] [Review][Patch] **The seat record and the card cite stale evidence** [`12-12-the-first-goals.md:626`;
  `12-12-signoff/vehicle-card.md:132`, `:82-83`] — Severity LOW. Patched rather than deferred: a stale record
  invents scope later. Sources: Acceptance (RAN), Feature (RAN).
  - The seat block rests AC 11 on "the review's wire replay of sitting B (20/20 at tick 2763)". That replay ran on
    `813c218`, before `b3a9753`. Cite run 2's HEAD replays: scenario path 20/20 at tick 2859 with 0 retry ticks
    (Acceptance), and live wire 20/20 at tick 2881 (Feature).
  - The card's "about 3.5 ms a tick, before and after 12.12" is stale. The patch pass measured the 518-cell pile
    at 1.62 ms, and the card says nothing of spread piles (Decision 1).
  - The spare's "the only tetrahedrite within four cells of any run" holds on the run's own level only. In 3D,
    ore touches run cells directly, e.g. `[46,83,13]` above the run's last cell.
- [x] [Review][Defer] **The ring test's retry bound is checked only until the ring fills, and its message
  overclaims** [`crates/sim-core/tests/scenario.rs:5248-5253`, `:5280-5283`] — deferred. Source: Edge (RAN).
  Passing runs read 0 and 20, the old rule 4569. The counter does not check that a cell is free ("while a pile cell
  was free"), and the loop stops at fill. With 26 stones for 20 cells, haul jobs may legitimately retry after fill.
- [x] [Review][Defer] **Only one boulder position is tested, and no ring touching a drop or the map edge**
  [`crates/sim-core/tests/scenario.rs:5296-5301`] — deferred. Source: Edge (read). Feature RAN a pit in the hole
  live (20/20). The map edge and the other three hole cells stay unprobed.
- Dismissed (1): Blind's "a rock-bounded alcove inside the box now fills in a different order". An alcove closed by
  pile and rock is an enclosure under Decision 1 (a), "a pocket whatever borders it". Acceptance's trajectory
  hashes over 10,000 ticks of the plan are identical on main, `f7df25b` and HEAD.

Code review run 3 (2026-10-19, diff `6aa424d..bdac2d4`, session 51ce71e4). Four layers: Blind Hunter and Edge Case
Hunter on Sonnet, Acceptance Auditor and Feature Auditor on Opus. **No layer timed out**, every layer ran
`cargo 1.97.1` in its own `/tmp/review-<layer>`, and no layer touched the repo.
- **Blind (sim-core):** read start-independence, tile writers and callers; a release probe (items+dwarves hash every
  25 ticks, plan digs and cuts) on `6aa424d` and HEAD for compact/spread3/spread5 piles, plus a far tile flipped every
  1/5/20/100 ticks. Hashes identical old and new in every layout; spread5 56.0 -> 0.66 ms, a flip every tick 48.9 -> 1.70.
- **Edge (tests, mutations, shells):** fixture geometry checked; rows 15-18 applied by hand in a `/tmp` copy, each dies
  at its own assert; `save_load` 17/17; every production tile write goes through `Terrain::set_tile`.
- **Acceptance:** audited the closure table; per-tick goal + items/dwarves logs byte-identical on `6aa424d` and HEAD
  for plan, far and a new four-3x3-pile layout (6.95 -> 0.45 ms); reload mid-run at t2000/t5000 identical to
  never-saved; an instrumented run counted 165,478 calls, 881 floods, 48 clears; put 54b35bd's rule back on HEAD.
- **Feature:** every hop WIRED (both callers pass the whole `Zones`; every tile writer goes through `set_tile`); live
  release daemons, HEAD vs `6aa424d` vs main: five spread piles 2.49 ms 193 t/s vs 21.27 ms 52 t/s; dig-heavy (+250
  digs) 3.79 vs 32.60 ms, main 3.54; stored/loose counts identical; `goals_wire.py` 5/5 GOALS OK; sitting B ring 20/20
  at t2881 (82 tile changes during the fill).

**Delta against run 2:** run 2 had 1 HIGH decision, 1 MED patch, 1 LOW patch, 2 defer, 1 dismissed. Run 3 NEW: **0
HIGH**, 1 MED patch, 1 LOW patch, 5 defer, 4 dismissed. **REWORK: closure row 1 (spread-pile cost)**, its both-sides
half only: the cost fix is real and keeps the fill, but the compact reseal that made Wolf re-rule (a) to (b) is pinned
by nothing. Rows 2 and 3 are CLOSED (row 2 with the LOW below). **The stopping rule FIRES: every new finding is below
HIGH, so there is no round 4.** The live gate already ran this round (Feature's daemons). The seat (AC 10, on
`a4d07eb`) predates the cache, and the trajectories are byte-identical across it.

**Coverage holes:** no gui or tui run this round (no client code changed); AC 7/8/10 rest on earlier runs and the seat.
After triage, `scripts/reap-build-caches.sh --tmp-only --force` reaped 19 directories under /tmp (21.8 GB listed)
and reclaimed 16.1 GB of free space. Review cost row: $12.59, 307 turns (`e1ab594`).

- [x] [Review][Patch] **The rule Wolf re-ruled away from (outside EVERY pile component's box, `54b35bd`) is pinned
  by no test or row** [`crates/sim-core/tests/scenario.rs:5296-5344`; `mutations/12-12.sh` row 18] — Severity MED.
  Source: Acceptance (RAN). Put back on HEAD with the cache kept, every sim-core test stays green (lib 96, save_load
  17, scenario 86, 22). Row 18 and the joint-ring test guard the Patch's literal "own pile's box" only. The compact5
  layout that showed the reseal (tet 1/10, 33 hauls retrying) was a measurement, and its coordinates are not recorded.
  - Add a ring test whose hole lies outside each pile component's own box: e.g. four one-side bars round a hole,
    rock at the corners, two deep so the band can seal; RED under the every-box rule, green on HEAD.
  - Add mutation row 19 that puts back `54b35bd`'s every-box rule; it must die at the new test's assert.
  - Record the layout's cells (and compact5's, if recovered from the patch pass scratch) in the record.
  - Riding along (Edge, LOW, same fixture code): put `assert_eq!(cells.len(), 20)` back inside `ring_pile`
    (`scenario.rs:5188`). `assert_ring_fills` lost it, and the two `ring_pile` tests now run any shape silently.
  - LANDED `e18c542` as a DEPTH test, Wolf's ruling 2026-10-20: the hauled four-bar fixture stalls on HEAD itself
    (14/24, haulers stand in the hole to fill its last way out), filed as #207. Row 19 KILLED. See the run 3 patch pass.
- [x] [Review][Patch] **The NOTE's and spine's cap example cannot seal** [`crates/sim-core/src/lib.rs:842`;
  `ARCHITECTURE-SPINE.md:216-219`] — Severity LOW. Patched rather than deferred: this is the third round correcting this
  text, and the spine is the binding record. Source: Acceptance (RAN in a scratch lib test). The arithmetic is now
  true (side 19 pocket, side 20 open), but a one-wide ring has no band two deep, so it reads as open and still fills.
  The smallest lone square band that seals through the cap is two deep and 39 on a side (hole 35: cap 1200 < 1225;
  side 38 is a pocket, cap 1168 > 1156). The spine also contradicts itself: "a thick band round it seals ... (a lone
  one-wide ring 20 across)". State the one-wide ring as reading open, and the 39-side two-deep band as the sealing case.
  - LANDED `a91e40f`; sides 38/39 and 19/20 re-measured by depth (run 3 patch pass).
- [x] [Review][Defer] **No row kills the flood's stop at an already-answered cell** [`crates/sim-core/src/lib.rs:881-884`]
  — deferred. Sources: Blind (read), Edge (read): a two-layer convergence. Removing it changes cost and the `FLOODED`
  count only; the unit test asserts `flooded == 0` on a repeat call, which still holds.
- [x] [Review][Defer] **spread5 is not reproducible from the record** [`12-12-the-first-goals.md`, run 2 patch pass
  RED table] — deferred. Source: Acceptance (read). No pile cells for spread5 (or the 518-cell pile beyond "camp +-20,
  z +-3"), and its 12/0/0 end goes uncommented.
- [x] [Review][Defer] **The card's cost line sits under the "big pile" heading** [`12-12-signoff/vehicle-card.md:133-134`]
  — deferred. Source: Acceptance (read). Five 25-cell piles are not "several hundred cells", and 2.86 ms is `World::step`
  for plan + spread5. Feature measured 2.4-2.9 ms live, so the figure holds.
- [x] [Review][Defer] **Ticks grow with loose items when haulers are short** [`crates/sim-core/src/lib.rs:583-593`] —
  deferred, pre-existing. Source: Feature (RAN). 0 haulers, 66 loose stones: 11.17 ms/tick on HEAD, 11.70 on main
  (90 t/s at fast4x). The per-haul-job `work_positions` in `claim_jobs`, not the floods. Candidate for #206.
- [x] [Review][Defer] **`Terrain.tiles` is not guarded, so a future direct writer would skip the invalidation**
  [`crates/sim-core/src/lib.rs:704-712`] — deferred. Source: Blind (read). Every production write goes through
  `set_tile` today; only test fixtures assign `.tiles` directly, before use.
- Dismissed (4): Edge's "the joint-ring test never asserts the hole is walkable and unzoned" (row 18 dies on it, so
  a broken fixture would show as a SURVIVED row); Blind's "the cache is hidden state, not saved" (a pure function of
  tiles and zones, rebuilt on load; AC 3 RAN identical); Feature's "every `set_tile` drops every answer" (by design,
  stated in the NOTE, measured harmless at play dig rates); Feature's "far-corner piles fill slowly" (long walks,
  identical on `6aa424d`).

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
  - **Full gate on `9fe4f89`, `RUST_TEST_THREADS=1 scripts/gate.sh`, rc 0:**

    ```
    cargo fmt --check           ok  0s
    cargo clippy -D warnings    ok  2s
    cargo test                  ok  719s
    cargo test (pixel guards)   ok  3008s
    tui has no sim-core edge                ok
    client-core has no sim-core edge        ok
    gui has no sim-core edge                ok
    metrics ledger tests        ok  1s
    bench tests                 ok  19s
    mutation tables still apply ok  9s
    GATE GREEN  3766s
    ```
  - **Closure table** (no row is REWORK; nothing re-closes an earlier round's fix):

    | Item | Written for | Tested from | Pre-existing-state fixture |
    |---|---|---|---|
    | AC 11 beside rock | a ringed hole touching rock | the plain ring (must still fill, 0 retry ticks) and the OLD rule (must fail both asserts) | `a_pile_round_ground_beside_rock_still_fills_every_cell` RED on `f7df25b`; the stubbed-assert run on `f7df25b` (4569 retry ticks) |
    | Ring test no-retry clause | AC 11's "no haul job retries forever" | the old rule with the earlier assert stubbed | the same `f7df25b` worktree |
    | Spare tetrahedrite | a chain stuck at 9/10 | the plan WITHOUT the spare (exactly 10) | `plan-10-10-1.json` on DEFAULT_SEED |
    | Approximate counts | card text | n/a (prose) | n/a |
    | Two record claims | the record | the reviewers' measured figures | n/a (prose) |

- **Seat, AC 10 (Wolf, 2026-10-19, fresh world on `a4d07eb`, gui + tui, sitting A of the card): PASS.**
  - **Q1:** wood ticked at 10/10. Bloodstone read 1/1 for a long time without ticking, as the card says, and
    completed with tetrahedrite.
  - **Q2:** the title turned gold, every goal was done, and play went on.
  - **Q3:** Wolf cleared the west pile: "yes dropped but goal stays marked as done".
  - **Sitting B (the ring round the fire, the card's AC 11 step): NOT RUN, Wolf's call** ("too complicated test").
    AC 11 stands on `a_pile_round_enclosed_ground_still_fills_every_cell`,
    `a_pile_round_ground_beside_rock_still_fills_every_cell`, rows 13 and 14, and review run 2's replays of
    sitting B on `6aa424d`: the scenario path fills 20/20 at tick 2859 with 0 retry ticks (Acceptance), and the
    live wire fills 20/20 at tick 2881 (Feature). (Corrected at the run 2 patch pass: this line cited run 1's wire
    replay, 20/20 at tick 2763, which ran on `813c218`, before the box rule of `b3a9753`. Run 2's replays
    predate that pass's flood cache, which keeps the floods' answers and changes none of them; the ring tests ran
    on it, see below.)
  - **Seen at the seat, not 12.12's:**
    - Miners stuck in narrow pits. First, a one-wide tunnel at `77,70`: a clearing miner lifts and sets down one
      stone 32 times while the other retries her dig. Later, a miner frozen with a stone in a stone-walled pit at
      `68,52`.
    - Dwarf movement went chunky at fast4x while they were stuck ("move pause move pause"), then recovered. Wolf
      reads it as the loops, more visible at speed.
    - Both saves replayed offline (release) loop or freeze identically on `main` `60955ef`. The first save's ticks
      over 10 ms (14-20 per 3,000, up to 28 ms; a fast4x tick has 5 ms) fall on the loop's set-down and retry
      ticks, also on `main`. The later save's sim tick averaged 0.82 ms.
    - Both stuck miners got going again once Wolf designated more digs. Posted on #206 (the tunnel loop and the
      slow ticks) and #204 (the frozen miner).

- **Review run 2 patch pass (2026-10-19, Opus, fresh session).** All 3 `[Review][Patch]` items; the HIGH one landed
  as Wolf re-ruled it mid-pass.
  - **First ruling, (a) per-pile boxes** (`54b35bd`, `5b112cf`). Wolf took my variant of the Patch's wording
    ("a"): a flood is open once it reaches a cell outside EVERY pile component's x/y box, the cap, or a cell this
    call already found open (the Patch's literal "its own pile's box" depends on visiting order). That cut
    spread5 from 21.5 to 7.7 ms, but **resealed a ring in the compact layout**: the plan's campfire ring (two z9
    components) and the added squares, with rock between them, enclose ground that no one pile's box holds. It
    read as open, the ring sealed as on the old rule, 33 haul jobs retried for good, and tetrahedrite stopped at
    1/10 (HEAD 10/10). It also cost +20% on the plan alone. I stopped and asked again.
  - **Re-ruled (b), landed** (`25fe2da`): back to HEAD's one box over every pile. The floods' answers are kept in
    `Terrain::pile_ground` (a `Mutex` beside the tiles, so no signature changes) for the zones they were found
    for. `set_tile` drops them, and a change in the zones replaces them. The floods read terrain and zones only, not
    `blocked`, so moving stones do not stale them. An answer is a property of the ground's component within the
    box (it leaves the box or reaches the cap from any start), so a kept answer is the answer a new flood would
    give: fill is HEAD's. A flood also stops at a cell already answered.
  - **RED, as the Patch asked, a measurement** (release, every `World::step` over 10,000 ticks of
    `plan-10-10-1.json` from tick 6, scratch worktrees, 4 at a time; mean ms; goals as wood/tet/blood stored):

    | Layout | old rule `f7df25b` | HEAD `502be2b` | per-pile `54b35bd` | cache `25fe2da` |
    |---|---|---|---|---|
    | plan | 0.300 | 0.368 | 0.424 | **0.206** |
    | plan + three far 2x2 piles | 0.674 | 2.208 | 0.604 | **0.240** |
    | plan + five 5x5 spread | 4.97 | 19.84 (9,539 ticks over 10 ms) | 7.65 | **2.86** (9 over 10 ms) |
    | plan + five 5x5 compact | 3.71, tet 1/10 | 2.41, 12/10/0 | 4.09, tet 1/10 | **0.77**, 12/10/0 |

    Goals end the same on HEAD and the cache in every layout (12/10/1 on plan and far; 12/0/0 spread5; 12/10/0
    compact5). The 518-cell pile (camp +-20, z +-3, 5,000 ticks, 3 at a time): old 3.95, HEAD 2.01, cache 1.94
    ms; the floods are not its cost.
  - **Tests.** `the_entrance_floods_are_kept_until_the_terrain_or_the_zones_change` (lib.rs, replaces this pass's
    `spread_piles_flood_only_the_ground_beside_them`): a 5x5 pile with a hole closed by a rock spur. A second call
    floods 0 cells; digging the spur opens the hole (`(7,6)` depth 2 to 1); taking the two cells north of the hole
    out of the zone opens it (`(6,7)` 2 to 1). `two_piles_that_ring_ground_together_still_fill_every_cell`
    (scenario.rs, `5b112cf`): two piles and rock jointly ring a hole; fills 19/19 on HEAD and the cache, 16/19 on
    the per-pile rule as first worded. It needs a two-column bar: a one-column bar's cells find their west
    neighbour open first and `.any()` never floods the hole, so the first fixture survived row 16 (my checkpoint
    example to Wolf was wrong in that detail).
  - **Mutation table** (`env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh`, on `25fe2da`): **20/20 KILLED**.
    Rows 15-18 are new: 15 the answers dropped on every call (killed at "unchanged terrain and zones flood
    nothing"), 16 a tile change keeps them ("the dug spur opens the hole"), 17 a zone change keeps them ("the hole
    opens north once those cells leave the zone"), 18 a flood bounded by its own pile's box (the joint-ring test,
    `scenario.rs:5271`). Each dies at its own assert. Rows 13 and 14 still KILLED.
  - **NOTE and spine** (`lib.rs` `pile_targets`, `ARCHITECTURE-SPINE.md:213`): the rule as it is, with its limits.
    Enclosed ground reads as open when it reaches past the box (the box ignores z: a ramp to a closed cellar; a
    pile-and-rock pocket running past it) or past the cap (a lone one-wide square ring 20 on a side, hole 18; the
    cap counts every pile). The cost paragraph names the cache.
  - **Card:** the spare is the only tetrahedrite within four cells on that run's own level (`[46, 83, 13]` sits
    above the run's last cell); the pile cost line now gives 2 ms for the 518-cell pile and 3 ms for five spread
    piles. **Record:** the seat block cites run 2's replays (above).
  - **Full gate on `12febce`, `RUST_TEST_THREADS=1 scripts/gate.sh`, rc 0:**

    ```
    cargo fmt --check           ok  1s
    cargo clippy -D warnings    ok  2s
    cargo test                  ok  750s
    cargo test (pixel guards)   ok  3210s
    tui has no sim-core edge                ok
    client-core has no sim-core edge        ok
    gui has no sim-core edge                ok
    metrics ledger tests        ok  1s
    bench tests                 ok  19s
    mutation tables still apply ok  11s
    GATE GREEN  4002s
    ```
  - **Closure table:**

    | Item | Written for | Tested from | Pre-existing-state fixture | REWORK |
    |---|---|---|---|---|
    | Spread-pile cost (HIGH) | ground inside the box between spread piles | the compact layout where piles and rock ring ground together (must still fill), and unchanged/changed terrain and zones | far, spread5, compact5 on HEAD `502be2b`; `two_piles_that_ring_ground_together_still_fill_every_cell` | **yes**: re-closes run 1's box rule (`b3a9753`), and this pass's own `54b35bd` |
    | NOTE and spine overstate the rule | the doc text | the reviewers' ramp-cellar and tunnel runs | n/a (prose) | no |
    | Stale seat record and card | the record and the card | the run 2 replays; a probe of `[46,83,13]` (`Solid(Tetrahedrite)`) | DEFAULT_SEED | no |

- **Review run 3 patch pass (2026-10-20, Opus, fresh session).** Both `[Review][Patch]` items.
  - **The every-box rule pinned (MED), as a depth test** (`e18c542`). The suggested ring test did not work as a
    scenario: four bars two deep round the campfire's 3x3 square, with rock on the 16 corner cells (Chebyshev 2..=3
    from the fire on both axes), run through `assert_ring_fills`. It **does not fill on HEAD**: 14/24 stored, the
    last at t1748, and 12 haul jobs retrying until t6000. Traced: the last free inner cell (64,62) is the hole's
    only way out. Haulers 1 and 3 are sent to its neighbour (64,63) in the hole, and from there the drop walls them in
    (#182/#201), so it is refused. Idle dwarf 4, also in the hole, makes a fill from outside refused too. The
    depth-1 outer cells get nothing while a deeper free cell exists. On main `60955ef`, and with the every-box rule
    put back (row 19's payload, scratch worktree), the same fixture truly seals at 12/24 (a different free set:
    the inner row is cut off). A plus-shaped variant (the hole's 4 corners rock too) also stalls on HEAD at 14/24.
    I stopped there rather than tune a third. **Wolf ruled (2026-10-20): pin by depth and file the stall: #207.**
    `a_hole_outside_each_piles_own_box_is_still_a_pocket` (lib.rs): the same four bars and corner rock round a
    3x3 hole at (8,8) on `flat_terrain(16, 16)`. On HEAD the outer row is depth 1 and the inner row depth 2 (the
    hole is a pocket). Row 19 (the every-box rule put back, flood-answer cache kept) **KILLED** at `lib.rs:7500`
    ("(7,6) lies inside: the hole is a pocket"), run alone. `ring_pile` asserts its 20 cells again. That is a
    fixture guard, not a test, so it has no row.
  - **compact5, recovered** from the run 2 patch pass probe (`tp-cache/src/main.rs` in that session's scratch):
    `plan-10-10-1.json`'s digs, cuts and piles, applied at tick 6. Then five 5x5 squares, each a `PlaceStockpile`
    rect x/y +-2 round camp + (-8,-8), (8,-8), (-8,8), (8,8), (0,-16), and z from the surface -1 to +1 (the
    surface is the highest `Empty` over `Solid`/`Ramp` at the centre). The four-bar layout is the one above.
  - **Cap example (LOW)** (`a91e40f`), in the NOTE and the spine. Re-measured by depth in a scratch lib test on
    `e18c542`. A two-deep square band of side 38 (288 cells, cap 1,168, hole 1,156): inner row depth 2, a pocket.
    Side 39 (296 cells, cap 1,200, hole 1,225): inner row depth 1, read as open (inner corners depth 2, as they
    touch no hole cell). One-wide rings of side 19 and 20 are depth 1 throughout, so they fill either way.
  - **Full gate on `a91e40f`** (`RUST_TEST_THREADS=1 scripts/gate.sh`):
    ```
    cargo fmt --check           ok  1s
    cargo clippy -D warnings    ok  4s
    cargo test                  ok  725s
    cargo test (pixel guards)   ok  3011s
    tui has no sim-core edge                ok
    client-core has no sim-core edge        ok
    gui has no sim-core edge                ok
    metrics ledger tests        ok  0s
    bench tests                 ok  19s
    mutation tables still apply ok  10s
    GATE GREEN  3779s
    ```
  - **Closure table:**

    | Item | Written for | Tested from | Pre-existing-state fixture | REWORK |
    |---|---|---|---|---|
    | Every-box rule unpinned (MED) | the rule Wolf re-ruled away from (`54b35bd`) | HEAD (depths 1/2, green); the every-box rule put back with the cache kept (row 19, RED at `:7500`); the hauled fixture on main, on the every-box rule (both seal 12/24) and on HEAD (stalls 14/24, #207) | the four-bar layout: two-deep bars round a 3x3 hole, rock corners, so no pile's own box holds the hole | **yes**: re-closes run 2's closure row 1 (its both-sides half) |
    | Cap example cannot seal (LOW) | the NOTE and spine text | depth probes of two-deep bands 38/39 and one-wide rings 19/20 on `e18c542` | n/a (prose) | **yes**: the third correction of this text (run 1 and run 2 patches) |

### File List

- `crates/sim-core/src/lib.rs` (goals, stored helper, `pile_targets` entrance rule, unit tests), `crates/sim-core/src/save.rs`, `crates/sim-core/tests/scenario.rs` (chain test, ring test, determinism reader), `crates/sim-core/tests/save_load.rs`
- `crates/simd/src/main.rs` (load bound and two tests)
- (Agent B, Task 4) `crates/protocol/src/lib.rs`, `crates/simd/src/bridge.rs`, `crates/simd/tests/serve.rs`, `_bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh`, and the `Snapshot`/`Delta` literals in `crates/client-core`, `crates/tui`, `crates/gui` (src and tests)
- (Agent B, Task 5) `crates/client-core/src/lib.rs`, `crates/client-core/tests/mirror.rs`, `crates/tui/src/view.rs`, `crates/tui/src/main.rs`, `crates/tui/tests/client.rs`, `crates/gui/src/ingest.rs`, `crates/gui/src/appearance.rs`
- (Agent B, Task 4, literals) also `crates/gui/src/capture.rs`, `crates/gui/src/pick.rs`, `crates/gui/src/project.rs`, `crates/gui/tests/capture.rs`, `crates/gui/tests/headless.rs`
- (Agents A and B, re-pointed rows) `_bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh`, `_bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh`
- `_bmad-output/implementation-artifacts/mutations/12-12.sh` (new, 15 rows; 16 after review run 1; 20 after review run 2; 21 after review run 3)
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
| 2026-10-19 | Review run 1 patch pass (Opus, fresh session): all 4 patches landed (`b3a9753` `97f153d` `09139bb` `9fe4f89`); boulder RED to green, row 14 KILLED; full gate GREEN on `9fe4f89` (3766 s). Status -> review for a round-2 review; AC 10 seat still open. |
| 2026-10-19 | Seat (AC 10) PASS on `a4d07eb`, sitting A, Q1-Q3 (Wolf). Sitting B not run (Wolf's call); AC 11 rests on the ring tests and the review's wire replay. Stuck-miner loops and fast4x stalls seen; pre-existing on `main`, posted on #206 and #204. |
| 2026-10-19 | Code review run 2 on `6aa424d` (diff `813c218..6aa424d`, 4 layers, none timed out): 1 HIGH decision, the single box over every pile makes `pile_targets` several times slower once piles are spread out (fill unchanged). Wolf ruled (a), fix now. 3 patches left as action items; 2 defer; 1 dismissed. Stopping rule did not fire. Status -> in-progress. |
| 2026-10-19 | Code review run 3 on `bdac2d4` (diff `6aa424d..bdac2d4`, 4 layers, none timed out): 0 HIGH. The flood-answer cache holds (identical trajectories, spread piles 8-10x faster live). REWORK closure row 1: the every-box rule is unpinned (MED patch); LOW patch on the cap example. 5 defer; 4 dismissed. Stopping rule fired: no round 4. Patches left as action items. Status -> in-progress. |
| 2026-10-19 | Review run 2 patch pass (Opus, fresh session): per-pile boxes (`54b35bd` `5b112cf`) resealed the campfire ring in a compact layout; Wolf re-ruled (b), landed as one box plus kept flood answers (`25fe2da`): spread piles 19.84 to 2.86 ms, fill unchanged. NOTE, spine, card and record (`12febce`). Mutation table 20/20 KILLED; full gate GREEN on `12febce` (4002 s). Status -> review for a round-3 review. |
| 2026-10-20 | Review run 3 patch pass (Opus, fresh session): both patches landed. The every-box rule is pinned by a depth test plus row 19 (KILLED) (`e18c542`); the hauled four-bar ring stalls on HEAD itself (14/24), so Wolf ruled depth + issue (#207). Cap example corrected in the NOTE and spine (`a91e40f`). Full gate GREEN on `a91e40f` (3779 s). No round 4 (run 3's stopping rule). Status -> review: PR on Wolf's yes. |
| 2026-10-20 | Pushed (`f833d0c`) and merged as PR #208 (`ac0a2f8`); no issue closed, #207 stays open. Status -> done. |
