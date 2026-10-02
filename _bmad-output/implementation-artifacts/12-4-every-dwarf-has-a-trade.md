---
baseline_commit: f4d9ba5
model: claude-opus-5-5  # session default, same as 12.1-12.3's creation
---

# Story 12.4: Every Dwarf Has a Trade

Status: in-progress

## Story

As the boss,
I want each dwarf to have a trade and to work only at it,
so that digging, hauling and felling can all go on at once instead of queueing behind each other.

## Not stacked: branch off `main`

`main` is `f4d9ba5` (PR #160 merged), clean. Branch `story-12-4-every-dwarf-has-a-trade` carries this
file, `12-4-signoff/` and the board edit. Epic 12's nine standing ACs (`epics.md`, "Standing acceptance
criteria") bind this story and are not restated. The "Wire diff" section below satisfies standing AC 4.

**#159 is folded in (Wolf, Task 0).** Fix B closes its summed-components half. Its single-area half
stays open, so the PR says `Refs #159` and never `Closes`
([[closing-keyword-closes-whole-issue]]).

## Found at creation (2026-10-02, on `f4d9ba5`)

- **The symptom reproduces only with a backlog that is reachable from the start.** Claiming runs FIFO
  by `JobId`, and every dig id is lower than every haul id. So a free dwarf always takes a claimable
  dig before any haul (`claim_jobs`, `lib.rs:458-546`).
  - **Sim RED** (probe, seed 42, 20 digs beside a corridor, all reachable; skeleton below): first
    delivery at tick 342 with **0** marks left. The same probe on DEFAULT_SEED: tick 292, 0 left.
  - **Live RED** (`12-4-signoff/first_delivery.py`, fresh `simd`, 25 channel marks on the camp floor):
    the backlog runs out at tick 220 and the first stone lands at tick 299, with **0 of 25** marks left.
  - **Not a repro:** a 49-mark dig block on the bowl wall. Its inner tiles are unreachable at first,
    so they go on retry cooldown, and the free dwarves take hauls in the meantime. First delivery came
    at tick 212 with 39 of 49 left. The scenario test must use an all-reachable backlog.
- **Prototype GREEN** (a throwaway worktree, not committed): a `Profession` component, the claim
  filter below, and a seeded 2-miner/2-hauler/1-woodcutter pool.
  - Sim probe: 12 marks left at the first delivery (seed 42) and 9 (DEFAULT_SEED).
  - Live recipe: tick 153, 19 of 25 left.
- **What the prototype broke in sim-core.** All five failures are fixture shape; none is a logic
  regression.
  - Three in-crate unit tests assume dwarf 2 claims a dig: `claim_jobs_waits_for_the_reaction_delay`,
    `an_unreachable_lower_id_does_not_starve_a_reachable_dwarf`,
    `claim_jobs_bounds_aggregate_astar_expansions_per_tick`.
  - Two `SavedDwarf`-literal tests failed only because the prototype's `from_save` gave every dwarf
    `Miner`: `save_load_preserves_in_progress_work` and
    `two_carriers_racing_for_the_last_tile_do_not_leave_a_permanent_stack`. The real field fixes them.
  - Everything else in sim-core (worldgen, the walking skeleton, 12.3's tests) stayed green.
- **Boot frames are unaffected.** Professions change nothing until a job exists: wander draws are
  untouched, and the spawn and identity streams are untouched. The gui pixel guards (no designations)
  have no reason to move.
- **#159 REPRODUCED** with an in-crate probe on `main`. Five dwarves stand on five separate
  11,000-cell plates (the fixture of `claim_jobs_bounds_aggregate_astar_expansions_per_tick`), with N
  unreachable digs queued ahead of one reachable dig on plate 4. **The reachable dig is never claimed
  in 300 ticks, at N = 10, 25 and 60.** Two throwaway prototypes:
  - **A, the issue's candidate** (stamp the exhausted job, then `break`): claimed at tick 110 for
    N = 10, and **never** for N = 25 or 60. Each tick stamps one job, so once more than
    `RETRY_COOLDOWN` (20) unreachable jobs are queued, the first ones come off cooldown before the
    loop reaches the reachable job.
  - **B, a budget per dwarf** (Wolf's choice): claimed on the first tick (tick 100) for all three N.
    The release time of one tick on this fixture (about 55k nodes) is 25 ms, against 22 ms on main.
    The worst-case ceiling rises from 50k nodes per tick to (idle dwarves) × 50k, about 110 ms here.
    Of the sim-core suite, only the bug-pinning test above fails under B.
  - **B needs one more rule, found while writing AC12 and verified.** If an over-budget dwarf's
    sitting out still counts, the sealed dwarves' 12.3 cache hits stamp the reachable job in step with
    the unreachable ones, and it is never claimed. So a job is stamped only when no dwarf of its trade
    sat it out for budget. This is "B′"; the summed results above are unchanged under it.
  - **A single area over 50k**, from five plates joined by ramp staircases (55,003 cells; skeleton
    below), with one dwarf in it, four in sealed one-cell pockets, and 10 unreachable digs. After
    one tick on `main`, every `retry_after` is 0 (`break 'jobs`). Under B every job is stamped 120.
  - **What B′ leaves open (stays on #159):** a dwarf in one area over 50k nodes still exhausts on
    each unreachable job, one per tick. With more than 20 of them ahead of a reachable job, that
    dwarf never reaches it (prototype, N = 25: never claimed). Today's worlds cannot trigger this
    (#159: DEFAULT_SEED's area is about 16k).

## Wire diff (standing AC 4)

- NEW `protocol::Profession`: `Miner | Hauler | Woodcutter`, `snake_case`.
- `protocol::Entity` gains a last field, `profession: Option<Profession>`, with
  `#[serde(default, skip_serializing_if = "Option::is_none")]`.
  - A dwarf's line gains `"profession":"miner"` after `identity`.
  - Emitter lines are byte-identical.
  - `Entity` stays `Copy`.

## Acceptance Criteria

1. `World::generate` gives each of the five dwarves a profession. The draw shuffles the fixed pool
   `[Miner, Miner, Hauler, Hauler, Woodcutter]` and assigns it in ascending id order.
   - It uses a NEW purpose-named stream, `STREAM_PROFESSION`, and never `spawn_rng` or the identity
     stream. This is a load-bearing determinism mechanism (AD-7): spawn positions and identities on
     every seed stay exactly as they are today.
   - A test over 64 seeds shows every world has exactly two miners, two haulers and one woodcutter.
   - A test names two seeds whose assignments differ.
2. `claim_jobs` considers a dwarf for a job only when the job's trade is the dwarf's profession:
   `Dig | Channel` → miner, `Haul` → hauler. No job kind maps to woodcutter until 12.7.
   - Claiming stays FIFO within a trade: a free miner takes the lowest-id dig or channel even when a
     lower-id haul is queued.
   - Mechanism, load-bearing (AD-12): there is still exactly one claiming system, and only its filter
     grows. The AD-12 amendment is recorded on the spine.
3. A job with no free dwarf of its trade is never put on retry cooldown by dwarves of other trades.
   Its `retry_after` is unchanged after the tick.
4. Scenario test (skeleton below): seed 42, 20 reachable digs and a 2-cell pile. At the tick the first
   stone lands on the pile, **more than 5** dig marks remain. RED at creation was 0.
5. In that same run, on every tick:
   - no miner holds a haul, and no hauler holds a dig or channel;
   - the woodcutter holds no job.
   His position changes at least once during the run (he wanders).
6. Profession is sim state.
   - `save → load → tick N ≡ never-saved → tick N` compares `professions()`.
   - Seed + commands give identical professions.
   - A save with no `profession` fails to decode, and simd refuses the load with a log line.
7. The real daemon is the judge.
   - Its connect snapshot and its deltas carry a profession on each of the five dwarves, with at least
     one of each trade.
   - After a daemon `save` then `load`, the fresh snapshot carries the same `(id, profession)` set.
8. The tui roster row shows each dwarf's trade after his name, in the format Wolf approves at Task 0.
   - Under `NO_COLOR` the trades still appear.
   - A delta that changes a dwarf's profession changes the row.
   - No key, status text or hint text changes.
9. `first_delivery.py` against a fresh `simd` prints `FIRST DELIVERY … marks_left N of 25` with N > 5.
   RED at creation was 0 of 25.
10. At the seat, Wolf reads the trades in an attached tui's roster. In the gui he watches a dwarf carry
    a stone to a pile while channel marks are still being worked.
11. #159: five idle miners stand in five separate areas, each under 50k nodes but summing past it.
    With 25 unreachable digs queued ahead of one reachable dig, the reachable dig is claimed on the
    first claim tick. RED on `main`: never claimed in 300 ticks. 25 is deliberately more than
    `RETRY_COOLDOWN`, so the issue's stamp-and-stop fix fails this AC.
12. Each dwarf has his own budget of `MAX_ASTAR_NODES` per tick. A dwarf whose search runs out of it
    sits out the rest of that tick, and the other dwarves go on claiming. A job is put on cooldown
    only when no dwarf of its trade sat it out. The fixture: one miner in the 55,003-cell area, four
    in sealed pockets, 10 unreachable digs, then one reachable dig in his area.
    - First tick (100): only dig 0, the one he exhausted on, is stamped (`120`). Every other job keeps
      `retry_after == 0`.
    - **He claims the reachable dig at tick 110**: one exhausted dig per tick, then the reachable one.
    Verified on a prototype at creation. `main` never claims it (`break 'jobs`, every stamp 0).
    Stamping the jobs he sat out never claims it (the pocket dwarves' 12.3 cache stamps it in step
    with the unreachable ones). A fresh budget per search claims it at tick 100.
    Mechanism, load-bearing: the per-dwarf budget is the cost bound Wolf ruled (2026-10-02).

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings, 2026-10-02, at creation.**
  1. **Pool: fixed 2 miners / 2 haulers / 1 woodcutter, assigned by seed.** He chose this over one of
     each plus two random picks.
  2. **Roster: "Name + grey trade", approved as mocked.** The mock used DEFAULT_SEED's real names; the
     trades in it are illustrative:
     `Nain miner  Ori hauler  Bifur woodcutter  Frar miner  Dori hauler`
     - The name stays in tunic colour, then one space, then the trade word in full in `STATUS_TEXT`
       grey. Two spaces separate dwarves.
     - The 2/2/1 worst case is 75 columns. Once 12.6 allows 3+ woodcutters the row can reach 93, and
       it truncates as it does today.
  3. **#159: FOLD IT IN.** The fix shape, chosen from two measured prototypes, is **B: a budget per
     dwarf** (not the issue's stamp-and-stop; not "B, and close #159"). So #159 stays open for the
     single-area residual. B′ (the sat-out rule in "Found at creation") is the verified form of B.
  4. **Old saves are refused, with no migration**, as ruled in 12.2.
- [x] **Task 1: sim-core profession (AC1, AC6).**
  - [x] `lib.rs`, beside `Identity`:
        `#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)] pub enum Profession { Miner, Hauler, Woodcutter }`.
        Its own component, not a field of `Identity`, because 12.6 makes it mutable.
  - [x] `const STREAM_PROFESSION: u64 = 0x5052_4f46_4553_534e; // "PROFESSN"` beside `STREAM_IDENTITY`
        (`lib.rs:31`). In `generate` (`:1309`):
        - seed `ChaCha8Rng::seed_from_u64(seed ^ STREAM_PROFESSION)`;
        - `shuffle` the pool `[Miner, Miner, Hauler, Hauler, Woodcutter]`;
        - pass it into `spawn_dwarves` (`:1810`) beside `identities`, and add it to the spawn bundle
          (`:1834`).
        Draw nothing from `spawn_rng` or `identity_rng`.
  - [x] `pub fn professions(&self) -> Vec<(Id, Profession)>`, sorted by id: a sibling reader like
        `identities()` (`:1752`). Do not widen `dwarves()`.
  - [x] `save.rs`: `SavedDwarf` gains `pub profession: Profession`, with no `#[serde(default)]` and a
        `// NOTE:` like `identity`'s. `to_save` (`:1365`) and `from_save` (`:1463`) carry it.
        - **Trap:** a dwarf without `Profession` drops silently out of `to_save`'s `filter_map` AND
          out of `claim_jobs`' query. Insert it at BOTH spawn sites.
        - Fix the 6 `SavedDwarf { .. }` literals. A literal whose dwarf holds a job gets that job's
          trade: `two_carriers_racing…` → both `Hauler`.
  - [x] Tests:
        - NEW `worldgen.rs` test over seeds `0..64`: each world's `professions()` is exactly 2/2/1.
        - NEW: `DEFAULT_SEED` and 42 assign differently. The prototype's constant gave
          `[M,M,W,H,H]` vs `[M,H,H,W,M]`; yours will differ, so pick two seeds that do.
        - `same_seed_produces_identical_worlds` (`worldgen.rs:54`), `save_load_then_tick_matches_never_saved`
          (`save_load.rs:9`, list at `:173`) and `same_seed_and_commands_remain_deterministic`
          (`scenario.rs:1612`, beside `:1644`) also compare `professions()`.
        - `spawn_positions_for_seed_42_are_pinned` (`worldgen.rs:386`) and the identity tests pass
          untouched.
- [x] **Task 2: the claim filter (AC2, AC3).**
  - [x] `fn trade(kind: JobKind) -> Profession`: an exhaustive `match` with no wildcard
        (`Dig | Channel => Miner`, `Haul { .. } => Hauler`). 12.7's `Cut` then fails to compile until
        it is given a trade.
  - [x] `claim_jobs` (`:422`): add `&Profession` to the dwarf query (`:431`), and fix the three tuple
        patterns (`:435`, `:438`, `:442`). In the dwarf loop (`:482`), `continue` when
        `trade(job.kind) != profession`.
        - **Do this BEFORE `attempted = true` (`:493`) and before the component check (`:495`).**
          After it, every idle miner would stamp a 20-tick `retry_after` on each haul it skips, and
          hauls would lag a whole cooldown behind a free hauler (AC3).
  - [x] Rewrite the AD-12 comment (`:418-420`). It says "one shared node budget"; it must now say
        claiming filters by trade and spends one budget per dwarf (Task 2b).
  - [x] Amend AD-12 in `planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md:194`
        under the "Amended YYYY-MM-DD:" convention AD-10 uses (`:168`). The amendment says:
        - claiming considers a dwarf only for jobs whose trade (`trade(JobKind)`) is its profession;
        - FIFO and id order hold within a trade;
        - "Job-kind stories add variants and execution systems — never claiming logic" now reads
          "…their variant, its execution and its `trade` arm — never a second claiming system".
- [x] **Task 2b: #159, a budget per dwarf (AC11, AC12). RED first:** write both tests (Task 3) before
  this change, run them, and record the failures in the Debug Log. AC11 expects `None` and AC12
  expects every stamp at 0 (M2-27).
  - [x] In `claim_jobs`:
        - replace `let mut astar_nodes_remaining = MAX_ASTAR_NODES;` (`:449`) with
          `let mut budgets = vec![MAX_ASTAR_NODES; dwarves.len()];`, indexed like the sorted `dwarves`;
        - delete the per-job `if astar_nodes_remaining == 0 { break; }` (`:459-461`);
        - iterate the dwarves with `enumerate()` and pass `&mut budgets[slot]` to both
          `astar_with_budget` calls (`:503-509`, `:521-527`);
        - change both `(None, true, _) => break 'jobs` arms (`:515`, `:533`) to `continue`.
  - [x] **The sat-out rule.** Add `let mut sat_out = false;` per job. Inside the eligible-dwarf branch
        (after the trade filter and the reaction-delay check), and BEFORE `attempted = true`:
        `if budgets[slot] == 0 { sat_out = true; continue; }`. The stamp at `:547` becomes
        `if attempted && !assigned && !sat_out`.
        - A dwarf that exhausts DURING a search did attempt the job, so that job IS stamped. Without
          that, he would re-exhaust on the same job every tick.
  - [x] Rewrite the `// NOTE:` at `:450-454`:
        - the bound is now (idle dwarves) × `MAX_ASTAR_NODES` per tick;
        - the residual is one area over the budget with more than `RETRY_COOLDOWN` unreachable jobs
          ahead of a reachable one (#159).
  - [x] **Replace** `claim_jobs_bounds_aggregate_astar_expansions_per_tick` (`lib.rs:3210`). It pins
        #159's broken shape: its first assert IS the bug. Its five-plate fixture becomes AC11's test.
        Re-point mutation row `3-2-the-dig.sh:938` ("each claim search gets a fresh node budget") to
        AC12's test, which kills that sabotage at tick 100 vs 110.
- [x] **Task 3: sim tests (AC2-AC5, AC11, AC12).** Existing first:
  - [x] Fix the three unit tests named in "Found at creation". Add a `set_profession(world, id, p)`
        helper in the `tests` mod; in-crate tests may `insert` the component directly. Give the
        expected claimant the right trade, and keep each test's assertion as it is. **No public
        setter**: 12.6 adds the command.
  - [x] New unit tests (`lib.rs` tests mod):
        - `claim_jobs_takes_fifo_within_a_trade`: queue haul id 0 then dig id 1, with one free miner
          and one free hauler. The miner holds dig 1 and the hauler holds haul 0.
        - `a_job_with_no_free_dwarf_of_its_trade_gets_no_retry_stamp`: only miners are free, past the
          reaction delay, and a haul is queued. After the step, its `retry_after` is still 0.
        - `a_reachable_job_behind_unreachable_ones_is_claimed_when_areas_sum_past_the_budget` (AC11):
          the five-plate fixture of the test it replaces, all five dwarves `Miner`, 25 unreachable
          digs, then one reachable dig on plate 4 at `(50,50,9)`. Run `claim_jobs` alone from tick
          100; the reachable dig is held at tick 100.
        - `a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on` (AC12): the joined-plates fixture
          (skeleton below), all five `Miner`, 10 unreachable digs, then the reachable dig at
          `(5,5,1)`. Assert the tick-100 stamps (`[120, 0, …, 0]`, 11 jobs) and the claim at exactly
          tick 110.
  - [x] New scenario tests (`tests/scenario.rs`), skeleton below:
        - `hauling_starts_while_the_dig_backlog_is_still_queued` (AC4);
        - `each_trade_holds_only_its_own_jobs_and_the_woodcutter_wanders` (AC5): the same world, run
          to the first delivery plus 200 ticks. Check `claims()` × `jobs()` × `professions()` every
          tick, and count the woodcutter's position changes.
  - [x] Run the whole sim-core suite. Any other test that now fails because of WHICH dwarf claims
        gets a fixture fix, not a weakened assert. List each one in the Debug Log.
- [x] **Task 4: protocol + simd (AC7; the wire diff above).**
  - [x] `protocol`: the enum and the field. Fix every `Entity { .. }` literal (about 63; let the
        compiler list them) with `profession: None`, except the dwarf fixtures a test reads.
  - [x] Pin tests in `protocol`:
        - the existing entity and delta literals stay byte-identical;
        - the identity literal at `:320` gains `,"profession":"miner"` and round-trips;
        - `every_material_and_tile_variant_has_a_pinned_wire_name` (`:510`) gains a `Profession`
          block.
  - [x] `bridge.rs`: `fn profession(v: sim_core::Profession) -> protocol::Profession`, an exhaustive
        `match` beside `dwarf_colour` (`:191`). `dwarf_entities` (`:133`) sets it from
        `world.professions()`, looked up by id the way `identities` is.
  - [x] Extend `save_then_load_rewinds_every_client` (`serve.rs:544`): the connect snapshot gives all
        five dwarves `Some(profession)` with each trade present, and the post-load snapshot carries
        the same `(id, profession)` set. No new simd save validation: any trade mix is a valid save
        (12.6).
- [x] **Task 5: client-core + tui (AC8). Format per Task 0.**
  - [x] `client-core`: `pub fn profession_text(p: protocol::Profession) -> &'static str`, an exhaustive
        `match` beside `dwarf_name_text` (`lib.rs:17`). It is the only spelling, and 12.6's gui reads
        it too. Pin the three spellings in a unit test.
  - [x] `view.rs` roster (`:429-443`): after each name, a blank, then `profession_text` in `STATUS_TEXT`.
        A dwarf with `profession: None` shows his name only. No layout row moves.
  - [x] **The instrument is `tui --frames N` (real binary).** Extend `capture_roster` (`client.rs:1555`):
        - the two stub dwarves get professions;
        - its second frame changes one dwarf's profession as well as swapping identities;
        - a new test asserts each trade word sits after its own dwarf's name, and that the changed
          trade shows in frame 2;
        - the `NO_COLOR` test (`:1699`) also asserts the trade words.
  - [x] README tui paragraph (`README.md:70`): the roster also names each dwarf's trade.
- [x] **Task 6: the record.**
  - [x] Write `_bmad-output/implementation-artifacts/mutations/12-4.sh`. Every row must be KILLED, by
        the test named:
        1. the trade check removed → `hauling_starts_while_the_dig_backlog_is_still_queued`;
        2. the trade check moved after `attempted = true` → `a_job_with_no_free_dwarf_of_its_trade_gets_no_retry_stamp`;
        3. `trade` maps `Haul` to `Miner` → `each_trade_holds_only_its_own_jobs…`;
        4. professions drawn from `spawn_rng` → `spawn_positions_for_seed_42_are_pinned`;
        5. the pool has no hauler → the 64-seed test;
        6. `from_save` ignores the saved profession → `save_load_then_tick_matches_never_saved`;
        7. the bridge sends `profession: None` → `save_then_load_rewinds_every_client`;
        8. the roster drops the trade word → the new client test;
        9. exhaustion `continue` → `break 'jobs` (`main`'s shape) → AC11's test (it is never claimed);
        10. one shared budget again (`budgets[slot]` → a single counter) → AC11's test;
        11. stamp-and-stop (the issue's candidate) → AC11's test (25 > 20). **Superseded by patch pass 1:**
            row 11 is now mutA (stamp-and-stop on a budget per dwarf) against
            `a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp`, and rows 13 and
            14 were added (14/14). Patch pass 2 added rows 15 and 16 (16/16);
        12. the sat-out rule dropped (`&& !sat_out` removed) → AC12's test (never claimed).
        A fresh budget per search is `3-2-the-dig.sh:938`'s row, re-pointed to AC12's test (claimed
        at tick 100 instead of 110).
        Put each killing assertion where only its mutation reaches it ([[strengthened-test-needs-remutation]]).
  - [x] Re-point any older mutation row the change breaks (12.2 re-pointed four tables), and list them.
        `3-2-the-dig.sh:938` is known (Task 2b). Check 12-3.sh's rows too; they sabotage the
        component cache that `sat_out` sits beside.
  - [x] Comment on #159: the measured cause, fix B′, the AC11/AC12 tests, and the residual that stays
        open. Retitle it to the residual: "A dwarf in one walkable area over MAX_ASTAR_NODES starves a
        reachable job behind more than RETRY_COOLDOWN unreachable ones". The PR body says
        `Refs #159`, never `Closes`.
- [x] **Task 7: the live recipe and the seat (AC9, AC10), then the full gate.**
  - [x] Run the Verification recipe on the branch, GREEN then the deliberate RED, and record both
        outputs in the Debug Log.
  - [x] Write `12-4-signoff/vehicle-card.md` in the seat's launch form:
        - in WSL, `simd 7451`;
        - in PowerShell, `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`;
        - in WSL, attach `tui 7451`.
        Steps:
        - (a) the roster names a trade for each dwarf;
        - (b) channel a block of the camp floor and drag a stockpile west of the fire;
        - (c) a stone reaches the pile while channel marks remain.
        Tell him old `frostvein.save` files will not load.
  - [x] Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` (~75 min, [[gate-ooms-at-default-parallelism]]).

### Review Findings

Review run 1 (2026-10-02) on `05a16a4` vs `f4d9ba5`. Four layers ran, and every one ran cargo with its own
`CARGO_TARGET_DIR`. None timed out and none was a coverage hole.
- Blind Hunter (Sonnet) took `crates/sim-core/src` and `crates/client-core/src`.
- Edge Case Hunter (Sonnet) took simd, tui, protocol, gui and every `tests/` directory.
- The Acceptance Auditor and Feature Auditor (Opus) took the whole diff.

Severity was set at triage. There were 0 HIGH findings.

Live, confirmed independently by three layers: the roster reads
`Nain woodcutter  Ori hauler  Bifur miner  Frar hauler  Dori miner`, and `first_delivery.py` gives
`marks_left 17 of 25` on four fresh daemons. The feature layer also observed live:
- snapshots and deltas carry the trades;
- save and load keep the set;
- a save with `profession` stripped is refused with `missing field 'profession'`.

The gui was not built by any layer, so AC10 rests on Wolf's seat ("1 ok").

13 findings were dismissed:
- Two findings were by design: the woodcutter is idle until 12.7, and old saves are refused (ruled).
- Three cost findings were dismissed because the per-dwarf bound is Wolf's ruled cost bound: the 5× worst-case search ceiling, exhausted floods going uncached, and expensive reachable jobs being stamped. The last is the open #159 residual.
- "No 2/2/1 test across seeds" is false: AC1's 64-seed test covers it.
- "`professions()` has no caller" is false: `bridge.rs` calls it.
- A dwarf without a `Profession` cannot be spawned by any path, and the same holds for `identity`.
- Re-stamping the cached-out case is correct.
- Roster overflow past 80 columns cannot happen: with the fixed 2/2/1 pool and names of at most 6 characters, the worst row is 75 columns.
- Offset-brittle colour asserts.
- The gui stores the trade without reading it; that is the scoped intent (12.6).
- One `simd` serve-test failure was caused by the review: a sibling layer ran `pkill -x simd` during that run. The test passes alone.

| # | Layer | Sev | Route | Finding |
| --- | --- | --- | --- | --- |
| 1 | feature | MED | decision -> accepted | Dig backlog clears ~2.3x slower under the ruled 2/2/1 pool |
| 2 | accept | MED | patch | AC12's "the other dwarves go on claiming" is untested; two sabotages survive |
| 3 | accept | LOW | patch (same test) | "Trade filter before the budget check" ordering is unpinned |
| 4 | accept | LOW | patch (silent instrument) | Mutation row 11 duplicates row 10 and does not test stamp-and-stop |
| 5 | accept | LOW | patch (record) | Board and Change Log still say "AC10 awaits the seat" |
| 6 | accept | LOW | defer | Determinism test named in the spec does not compare `professions()` |
| 7 | accept | LOW | defer | Dev Agent Record line refs point into mutated sources |
| 8 | accept | LOW | defer | AC7's "deltas carry a profession" has no automated test |
| 9 | accept | LOW | defer | AC4's RED was never re-measured on the committed fixture |
| 10 | blind | LOW | defer | A job with no free dwarf of its trade re-plans its work positions every tick |
| 11 | feature | LOW | defer | A trade with zero dwarves leaves its jobs unclaimed silently |

- [x] [Review][Decision] **RESOLVED (Wolf, option 1): accepted as ruled. The numbers are recorded here and as a NOTE on 12.6 in `epics.md`, together with finding 11's zero-trade trap.** Dig backlog clears ~2.3x slower under the ruled 2/2/1 pool. The feature layer
  ran the same 25-channel recipe until every mark was gone.
  - `main` (`f4d9ba5`, built from a read-only `git archive`) cleared it in **218 ticks**, with or without a pile.
  - This branch took **495 ticks with a pile** and **529 without**.
  - Only 2 of 5 dwarves can dig now. Without a pile, the haulers logged 964 idle samples against 96 walking.
  - The story measured only "marks left at the first delivery" (AC4/AC9) and never the time to clear
    the backlog, so nothing records the cost. It follows directly from Wolf's Task 0 pool ruling. 12.6
    (reassigning trades) and 12.7 (woodcutter work) are where it eases.
- [x] [Review][Patch] AC12's "the other dwarves go on claiming" is untested [crates/sim-core/src/lib.rs:538,556; test at :3506].
  - The AC12 fixture's four other miners are sealed in one-cell pockets and can claim nothing.
  - AC11's plates never exhaust, so no test shows a second dwarf claiming in the same tick after another exhausted.
  - Two sabotages survive the whole sim-core suite, run by the auditor in a scratch copy:
    - **mutA:** both `(None, true, _) => continue` arms become "stamp, then `return`" (the issue's stamp-and-stop on per-dwarf budgets).
    - **mutB:** both arms become `break`, which leaves the dwarf loop.
  - Fix: add a fixture in which a higher-id miner can reach the very job a lower-id miner exhausted on,
    and assert that the higher-id miner holds it on the same tick. One assertion kills both mutations.
- [x] [Review][Patch] "Trade filter before the budget check" ordering is unpinned [crates/sim-core/src/lib.rs:503,517].
  - **mutC** (the auditor's) moves the `budgets[slot] == 0` sat-out check above the trade filter. The whole sim-core suite stays green.
  - Under it, an exhausted miner marks an unreachable haul as sat-out, so the haul is never stamped and is re-searched every tick.
  - Fix, in the same test as above: after a miner exhausts, an unreachable haul with a free hauler must still be stamped `tick + 20`.
- [x] [Review][Patch] Mutation row 11 duplicates row 10 [_bmad-output/implementation-artifacts/mutations/12-4.sh:95-97].
  - Row 11 restores the SHARED budget, which is row 10's exact sabotage, then adds stamp-and-stop. It
    dies at the same AC11 tick-100 assert that the shared budget alone already fails.
  - The record still says it covers "the issue's candidate". Stamp-and-stop on per-dwarf budgets is
    mutA, and mutA survives.
  - Fix: re-point row 11 to mutA against the new assertion, and add rows for mutB and mutC. Run all
    three and record the killing assertion.
- [x] [Review][Patch] Board and Change Log are stale after the seat
  [_bmad-output/implementation-artifacts/sprint-status.yaml:2514; Change Log above].
  - Both still say "AC10 awaits Wolf's seat". `40b6535` recorded the pass only in the Completion Notes and `vehicle-card.md`.
- [x] [Review][Defer] Determinism test named in the spec does not compare `professions()`
  [crates/sim-core/tests/scenario.rs:1755] — deferred.
  - The dev added the compare to `designate_dig_stockpile_haul_and_the_stone_reaches_the_pile_headlessly` (`:1089`) instead.
  - That test is also seed + commands, so AC6 is met. The swap is undocumented.
- [x] [Review][Defer] Dev Agent Record line refs point into mutated sources — deferred.
  - `lib.rs:3401`, `:3500`, `:3562` and `:3261` are mutate.sh panic sites in the mutated files.
  - In the committed tree they are blank lines or fields. The kills are real (rows 2 and 11 reproduced).
- [x] [Review][Defer] AC7's "deltas carry a profession" has no automated test [crates/simd/tests/serve.rs] — deferred.
  - `save_then_load_rewinds_every_client` checks only snapshots.
  - Deltas share `dwarf_entities` (`bridge.rs:69`), and two layers observed them carrying trades live.
- [x] [Review][Defer] AC4's RED was never re-measured on the committed fixture — deferred.
  - The test anchors on the first Miner rather than `dwarves()[2]`, a documented deviation.
  - Mutation row 1 stands in for the RED.
- [x] [Review][Defer] A job with no free dwarf of its trade re-plans its work positions every tick [crates/sim-core/src/lib.rs:483-495] — deferred.
  - The goal set is computed before the trade filter, and such a job is never stamped.
  - It now happens whenever both haulers are busy, not only when all five dwarves are.
  - The cost was not measured.
- [x] [Review][Defer] A trade with zero dwarves leaves its jobs unclaimed silently [crates/sim-core/src/lib.rs:503,570] — deferred to 12.6.
  - `attempted` stays false, so nothing is stamped or logged.
  - It is unreachable today (always 2/2/1, saves only from `to_save`).
  - It becomes reachable when 12.6 lets the player reassign the last hauler or miner, so 12.6 should decide what the player sees.

**Patch pass 1 (2026-10-02, fresh session).** Landed in `6232659` (review records), `6938841` (test), `ef7f523`
(mutation rows) and `0b38aca` (records). The FULL gate (`RUST_TEST_THREADS=1`) was GREEN, 3093 s, exit 0, on `0b38aca`.
`12-4.sh` gave 14/14 KILLED. The new test is
`a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp`, beside AC12's test. Its terrain
comes from the new `joined_plates` helper, which AC12's test now shares.

| Item | Side the fix was written for | Side tested | Pre-existing-state fixture | Rework? |
| --- | --- | --- | --- | --- |
| 2: same-tick handoff after exhaustion | correct code: a higher-id miner claims the dig a lower-id miner exhausted on | the old shapes the review named: mutA (row 11) and mutB (row 13) both fail `"miner 1 claims the dig miner 0 exhausted on, on the same tick"` | `joined_plates` (AC12's 55,003-cell area, miner 0 at (0,0,1)) + miner 1's pocket (120,120,20), the only work position of dig 0 at (121,120,20) | no |
| 3: trade filter before the budget check | correct order: an exhausted miner does not mark a haul sat out | the reordered code: mutC (row 14) fails `stamps == [0, 120]` (the haul is left at 0) | same fixture, with miner 0 at budget 0 when haul 1 comes up. Hauler 2 is sealed at (124,120,20); the stone (124,124,20) and the pile (126,124,20) are in other pockets | no |
| 4: row 11 duplicated row 10 | the record: row 11 now names mutA on a budget per dwarf | the run: rows 11, 13 and 14 each died at the assertion named above, not at row 10's AC11 tick-100 assert | the committed `12-4.sh` against `ef7f523` | no |
| 5: stale board / Change Log | the board comment and a new Change Log row | `rg "awaits"` in the board: no live claim is left. The old dev row is kept as history | `sprint-status.yaml` at `05a16a4` | no |

Exclusivity: each mutation was run against its one named test, as `mutate.sh` does. Before this pass, review run 1 had
mutA, mutB and mutC survive the WHOLE sim-core suite. So the new test is the only sim-core test that kills them.

**Review run 2 (2026-10-02, fresh session)** on `05a16a4..b0c1bd0`, which is patch pass 1. Test code and records only: every
`crates` hunk is inside `#[cfg(test)]`.
- Four layers ran, each running cargo with its own `CARGO_TARGET_DIR`. None timed out and none was a coverage hole.
- The shells had no diff, so the Edge Case Hunter was reassigned `mutations/12-4.sh` and the boundaries of the new test.
- The Blind Hunter took the sim-core test diff. Both auditors took the whole diff.

**Delta vs run 1: 4 NEW, 0 REWORK. Severity 0 HIGH, 1 MED, 3 LOW.** The stopping rule applies: no new finding is HIGH,
so the static audit ends here.

**The closure table was audited, and all of it holds.** Three layers re-ran rows 11, 13 and 14 independently in
scratch copies, and each died at the assertion the table names:
- mutA at `lib.rs:3647`;
- mutB at `:3641`;
- mutC at `:3648` (`[0, 0]` vs `[0, 120]`), after passing the earlier claim assertion.

The Edge Case Hunter applied rows 9-14 without an APPLY-FAILED. Row 11 no longer restores the shared budget.

**Live, by the Feature Auditor:**
- `first_delivery.py` gives `FIRST DELIVERY tick 186 marks_left 17 of 25` on two fresh daemons. The roster is unchanged.
- A probe-instrumented `simd` logged **0 exhaustions and 0 sat-outs** across the whole live recipe. The lowest budget
  left at any claim was 49,978.
- So the AC11/AC12 paths (`lib.rs:517-519`, `:538`, `:556`) never fire on DEFAULT_SEED. AC11/AC12 rest on the sim-core
  fixtures and mutation rows alone, which drive the real `claim_jobs` with the real `MAX_ASTAR_NODES`.
- The live GREEN says nothing about AC12. AC10 rests on Wolf's seat.

Four findings were dismissed:
- "Ordering pinned only by mutC": this is a description, not a defect.
- `sat_out = false` survives the new test: row 12 and AC12's test kill it, and the closure table never claimed it.
- The `[0, 120]` expectation depends on the reaction delay: it holds, and the edge layer walked it.
- The old run-1 "Next" comment in the board: the newer line above it supersedes it.

| # | Layer | Sev | Route | Finding |
| --- | --- | --- | --- | --- |
| 1 | blind (confirmed by orchestrator) | MED | patch | Sat-out `continue` -> `break` survives the whole sim-core suite |
| 2 | blind+edge+accept | LOW | patch (latent silent failure, same test) | The new test never asserts miner 0 exhausted |
| 3 | accept | LOW | patch (record) | Debug Log / Task 6 still describe the old row 11 and 12/12; the File List is missing two files |
| 4 | accept | LOW | patch (record) | Two run-1 deferrals in `deferred-work.md` lack a usable file:line |

- [ ] [Review][Patch] Sat-out `continue` -> `break` survives the whole sim-core suite [crates/sim-core/src/lib.rs:519; test at :3604].
  - **mutD** is `budgets[slot] == 0 { sat_out = true; break; }`. The orchestrator re-ran it in a scratch copy: sim-core passes 67 + 10 + 39 + 19 tests.
  - The new test's comment says "no leaving the dwarf loop", but only the two exhaustion arms (`:538`, `:556`) are pinned.
  - In the new test, miner 0 exhausts on the FIRST job, so the `budgets[slot] == 0` branch is never reached ahead of another miner on a later miner job.
  - Live effect under mutD: once the lowest-id miner exhausts in a tick, every later miner job that tick breaks at him. The other miners then get one job per tick: the #159 slowdown, through the sibling site.
  - Fix: add a third job, a dig beside miner 3's pocket (126,120,20), for example at (127,120,20), with an id after haul 1. Assert `claims()[3] == (Id(3), Some(JobId(2)))`. Add a `12-4.sh` row 15 for mutD against this test, run it, and record the killing assertion.
- [ ] [Review][Patch] The new test never asserts that miner 0 exhausted [crates/sim-core/src/lib.rs:3524 (`joined_plates`), :3604].
  - The fixture's exhaustion rests only on 55,003 > `MAX_ASTAR_NODES` (50,000), a 10% margin.
  - The accept layer applied mutC and raised `MAX_ASTAR_NODES` to 60,000: the new test PASSED with the mutant still in place. Only AC12's test failed (`:3577`).
  - The blind layer saw the same thing at 5,000,000 with no mutant.
  - A budget bump would make the new test vacuous silently.
  - Fix: one precondition assert in the new test. For example, call `astar_with_budget` from (0,0,1) to dig 0's work position with a fresh `MAX_ASTAR_NODES` budget and assert it exhausts. Or assert the joined area exceeds `MAX_ASTAR_NODES`. Re-run rows 11, 13 and 14 after the change (strengthened-test rule).
- [ ] [Review][Patch] Stale Dev Agent Record for row 11 [this file: Task 6 row 11 (~:294), Debug Log (~:626-640), File List].
  - The Debug Log still says `12-4.sh` 12/12 KILLED and describes row 11 as "stamp-and-stop on a shared budget -> AC11's tick-100 claim". Task 6 maps row 11 to AC11's test.
  - Mark both as superseded by patch pass 1 (row 11 is mutA; rows 13 and 14 were added; 14/14).
  - Add `deferred-work.md` and `planning-artifacts/epics.md` to the File List.
- [ ] [Review][Patch] Two run-1 deferrals lack a usable file:line [_bmad-output/implementation-artifacts/deferred-work.md:2411-2429].
  - "AC4's RED was never re-measured" should cite `crates/sim-core/tests/scenario.rs:1672` (assertion at `:1687`).
  - "Dev Agent Record line refs" should give the crate path for `lib.rs:3401/...`.

### Scenario test skeleton (Task 3; the creation probe, which ran RED on main)

```rust
// seed 42: a corridor east/west of dwarf 2, 10 digs on each side of it (all reachable), pile on
// cells 1-2. The first delivery must leave > 5 marks (RED on main: 0 at tick 342).
let mut world = World::generate(42, Dims::DEFAULT);
let worker = world.dwarves()[2].1;
let dx = if worker.x + 16 < world.dims().x as i32 { 1 } else { -1 };
let cell = |s: i32, dy: i32| Pos { x: worker.x + dx * s, y: worker.y + dy, ..worker };
for s in 1..=13 { make_standable(&mut world, cell(s, 0)); }
let mut targets = Vec::new();
for s in 3..13 { for dy in [-1, 1] {
    let t = cell(s, dy);
    world.set_tile(Pos { z: t.z - 1, ..t }, Tile::Solid(Material::Stone)); // floor: the stone lands standable
    world.set_tile(t, Tile::Solid(Material::Stone));
    targets.push(t);
}}
world.drain_dirty();
for t in &targets { world.apply_command(SimCommand::Designate { kind: DesignationKind::Dig, rect: rect(*t, *t) }); }
world.apply_command(SimCommand::PlaceStockpile { rect: rect(cell(1, 0), cell(2, 0)) });
let zones: BTreeSet<Pos> = world.zones().into_iter().collect();
// step (tick guard 5000) until world.items() has one on `zones`; then assert designations().len() > 5
```

### Joined-plates skeleton (Task 3, AC12; verified at creation, the area is 55,003 cells)

```rust
// In-crate (lib.rs tests mod). Five 110x100 plates at z = 1,3,5,7,9 as in the replaced test, then a
// ramp staircase at row y=99 joins plate z to plate z+2. A move up needs a Ramp under the LOWER cell.
let idx = |x: i32, y: i32, z: i32| super::worldgen::index(dims, x as u32, y as u32, z as u32);
for k in 0..4_i32 {
    let z = 1 + 2 * k;
    tiles[idx(108, 99, z - 1)] = Tile::Ramp(Material::Stone); // under A=(108,99,z)
    tiles[idx(109, 99, z + 1)] = Tile::Empty;                 // B=(109,99,z+1), carved
    tiles[idx(109, 99, z)] = Tile::Ramp(Material::Stone);     // B's floor, under the lower of B->C
    tiles[idx(110, 99, z + 2)] = Tile::Empty;                 // C=(110,99,z+2), on the next plate
}
// Four one-cell pockets for dwarves 1-4: (120|122|124|126, 120, 20) Empty; dwarf 0 at (0,0,1).
// Unreachable digs as the replaced test: work cell (3+2j, 2, 20) Empty, target (2+2j, 2, 20).
// Reachable dig: tiles[idx(5, 5, 1)] = Solid(Stone), job target (5,5,1), id = unreachable count.
// Run `claim_jobs` alone, ticks 100.. ; read world.jobs() retry_after after tick 100.
```

## Dev Notes

### Scope guardrails (do NOT)

- No set-profession command, setter or gui UI (12.6). No `Cut` job or woodcutter work (12.7).
- No second claiming system or per-trade pass. One loop, one filter (AD-12).
- Do not change 12.3's component cache, `reaction_delay`, `RETRY_COOLDOWN` or `MAX_ASTAR_NODES`.
  No cross-tick caching of searches (AD-5). The #159 residual stays open.
- No profession glyph colour in the tui (the `☻` stays in tunic colour, 12.2 Task 9). No gui display:
  the gui only gains `profession: None` in its literals and passes the field through the mirror.
- No load-time trade validation, and no migration of old saves.

### What already exists (build on it)

- 12.2's identity is the exact pattern: a purpose-named stream, a component at both spawn sites, a
  sibling reader, a `SavedDwarf` field, an exhaustive bridge `match`, a `client-core` spelling fn, and
  the roster row.
- Idle dwarves already wander (`wander`, `lib.rs:1181`, skips any dwarf holding a job), so a jobless
  woodcutter wanders with no new code.
- The `Mirror` stores whole `protocol::Entity` values, so the field reaches both clients with no mirror
  change.

### Key decisions & traps

- **A fixed pool, shuffled.** The ≥1-of-each guarantee is structural, and the roster fits 80 columns.
- **A separate `profession` field, not inside `Identity`.** 12.6 mutates it, and gui code compares
  `identity` to decide on a tunic re-colour.
- **The filter goes before `attempted`.** See Task 2. This is the one place a correct-looking filter
  silently delays every haul.
- **A loaded dwarf may hold an off-trade job** (a hand-written save). It finishes the job, because
  `execute_jobs` never reads the trade. 12.6 owns release-on-reassign.
- **Pick test dwarves by `professions()`, never by index.** Seeds assign trades in different orders.
- **Each dwarf's budget is HIS, and sitting out is not attempting.** An exhaustion mid-search stamps
  the job, so he does not re-exhaust on it every tick. Sitting out stamps nothing, so a reachable job
  is not put on cooldown in step with the unreachable ones. Both halves have a mutation row.
- **The per-tick cost ceiling is now (idle dwarves) × 50k nodes**, about 110 ms of A* on the devpod.
  It is reachable only once digging opens five huge separate areas. Wolf ruled it (2026-10-02).
- **The trade filter, then the budget check, then `attempted`.** A sat-out dwarf of the wrong trade
  must not block a stamp, so the trade filter comes first.

### Verification

The live recipe lives in `12-4-signoff/first_delivery.py`. Read its docstring before running it. It
is pinned to DEFAULT_SEED's camp.

```bash
cargo build -q -p simd
./target/debug/simd 7530 >/dev/null 2>&1 &          # FRESH daemon every run
python3 _bmad-output/implementation-artifacts/12-4-signoff/first_delivery.py 7530
pkill -x simd
#   RED (observed at creation on f4d9ba5, twice, identical):
#     backlog: tick 13 marks 25 pile cells 9
#     BACKLOG EXHAUSTED tick 220, no stone on the pile yet
#     FIRST DELIVERY tick 299 marks_left 0 of 25
#   GREEN (required): FIRST DELIVERY ... marks_left N of 25 with N > 5 (prototype: tick 153, 19 of 25)
```

Deliberate RED after the fix: apply mutation row 1, rebuild `simd`, and rerun. The output must return
to `BACKLOG EXHAUSTED` and `marks_left 0`. Restore, rebuild, and rerun for GREEN again. Exit 0 is not
a result; `marks_left` is. Do NOT run the sabotage in the shared working tree while Wolf may build
([[probe-sabotage-leaks-into-wolfs-build]]). Use `scripts/mutate.sh`'s copy, or commit first
([[sabotage-restore-trap]]).

Roster (after Task 5): `./target/debug/tui 7530 --frames 1 --z 9 | sed 's/\x1b\[[0-9;]*[A-Za-z]//g' | tail -3`.
The first of the three rows must show five names, each followed by a trade word. RED on `f4d9ba5`:
the row holds names only.

#159 has no live instrument. Today's worlds cannot reach it: DEFAULT_SEED's walkable area is about
16k nodes, and the trigger needs more than 50k. The fixtures are its evidence:
`cargo test -p sim-core --lib -- a_reachable_job_behind a_dwarf_over_his_budget`.
- RED, observed on prototypes at creation: AC11 `None` at N = 25; AC12 stamps all 0.
- GREEN: AC11 claimed at tick 100; AC12 `[120, 0, …]`, then claimed at tick 110.
- Mutation rows 9-12 are each fixture's deliberate RED.

### Project Structure Notes

- `crates/sim-core/src/{lib.rs,save.rs}`, `tests/{worldgen,save_load,scenario}.rs`: UPDATE
- `crates/protocol/src/lib.rs`: UPDATE (1 enum, 1 field, pins)
- `crates/simd/src/bridge.rs`, `tests/serve.rs`: UPDATE
- `crates/client-core/src/lib.rs`: UPDATE (`profession_text`, literals)
- `crates/tui/src/view.rs`, `tests/client.rs`: UPDATE
- `crates/gui/{src,tests}/*`: UPDATE (`Entity` literals only)
- `planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` (AD-12), `README.md`: UPDATE
- `_bmad-output/implementation-artifacts/mutations/12-4.sh`, `12-4-signoff/vehicle-card.md`: NEW
- `_bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh` (row `:938` re-pointed): UPDATE
- `12-4-signoff/first_delivery.py`: created at story creation

### References

- `epics.md` Epic 12 intro (order, standing ACs) and Story 12.4; PRD `prd-frostvein-2026-09-28`
  FR42, FR43, NFR9-NFR11
- Parent spine AD-6, AD-7, AD-9, AD-11, AD-12; Consistency Conventions (Vocabulary enums, Color)
- 8.3's symptom: `8-3-…md:141`, `8-3-signoff/vehicle-card.md:41`; `scripts/task6-designate.py`
  docstring (channels are always reachable, digs settle at a floor)
- Issue #159; `lib.rs:452` NOTE
- Memory: [[strengthened-test-needs-remutation]], [[gate-ooms-at-default-parallelism]],
  [[probe-sabotage-leaks-into-wolfs-build]], [[sabotage-restore-trap]]

### Previous story intelligence (12.2, 12.3)

- 12.2's `Entity` sweep took about 56 literals, and it is now about 63. It is mechanical: let the
  compiler list them. 12.2 also re-pointed mutation rows in four older tables, so expect the same.
- 12.3 changed `claim_jobs`' failure paths: the component cache and `break 'jobs` on budget
  exhaustion. The trade filter must skip a dwarf before either one sees him.
- The full gate is green only at `RUST_TEST_THREADS=1`: 4221 s and 4458 s on 12.3.

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5.5 subagents x2 (Tasks 1-3; Tasks 4-5), orchestrated and verified by Claude Opus 5.5, which also wrote Task 6's table, ran every mutation and Task 7.

### Debug Log References

- **RED, AC11 and AC12, on the unchanged claim code** (after Task 2's filter, before Task 2b's budget change):
  - `a_reachable_job_behind_unreachable_ones_is_claimed_when_areas_sum_past_the_budget`:
    `left: (Id(4), None)`, `right: (Id(4), Some(JobId(25)))`, "the plate-4 miner must claim the reachable dig on the first claim tick".
  - `a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on`:
    `left: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`, `right: [120, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`.
  - GREEN after the change: both pass (AC12 also pins no claim on ticks 101-109 and the claim at tick 110).
- **Fixtures fixed because of WHICH dwarf claims** (seed 42 assigns `[Woodcutter, Miner, Hauler, Hauler, Miner]`; assertions untouched):
  - `claim_jobs_waits_for_the_reaction_delay`: dwarf 2 (a hauler) set to Miner.
  - `claim_jobs_takes_fifo_and_skips_busy_dwarves_and_claimed_jobs`: dwarf 0 (a woodcutter) set to Miner in both worlds.
  - `claim_jobs_prefers_the_lowest_free_dwarf_id`: all five set to Miner.
  - `an_unreachable_lower_id_does_not_starve_a_reachable_dwarf`: dwarves 0 and 1 set to Miner. It still passed, but dwarf 0 would have been a vacuous woodcutter.
  - `save_load_preserves_in_progress_work` (`tests/save_load.rs`): the worker is the first Miner from `professions()`, not `dwarves()[2]`.
  - `claim_jobs_bounds_aggregate_astar_expansions_per_tick`: all five set to Miner as an interim fix in Task 2, then replaced in Task 2b.
  - `SavedDwarf` literals: `save_load.rs` (2 digs) `Miner`; `scenario.rs` `two_carriers_racing...` both `Hauler`.
- **AC4 measured:** seed 42, first stone on the pile at tick 241 with 8 of 20 marks left.
- **Mutation rows re-pointed (anchor text only unless noted):** `12-2.sh` "from_save ignores the saved identity";
  `2-2-dwarves-wander-the-frost.sh` "spawn consumes the worldgen stream again"; `3-2-the-dig.sh` "claim_jobs walks dwarves descending"
  (6-tuple), "unreachable lower id starves a reachable dwarf" (`budgets[slot]`, `continue`), and "each claim search gets a fresh
  node budget" (now pointed at AC12's test, Task 2b).
- **Task 4 RED:** `save_then_load_rewinds_every_client` panicked "every dwarf carries a profession" with the bridge sending `None`;
  green after `profession_out` and the `professions()` lookup. `Profession` pin test and the identity literal (now `,"profession":"miner"`) added in `protocol`.
- **Task 5 RED:** `each_trade_word_sits_after_its_own_dwarfs_name_in_grey_and_follows_a_profession_change` and `the_trade_words_survive_no_color`
  failed with names only (`"Durin  Nori"`); green after the `view.rs` roster change.
- **Orchestrator re-point, Task 5:** `12-2.sh` "tui never draws the roster" (the roster now pairs identity with profession; the
  sabotage still empties the identity stream).
- **Mutations, run by the orchestrator** (`RUST_TEST_THREADS=1 scripts/mutate.sh`, on committed `5f4ad04`): `12-4.sh` **12/12 KILLED**,
  and the six re-pointed older rows (extracted into a scratch table) **6/6 KILLED**. Killing assertion per row:
  1. trade check removed -> `hauling_starts_...` `marks_left > 5` (scenario.rs:1686);
  2. trade check moved after attempted -> AC3's "no miner may stamp a haul" (lib.rs:3401);
  3. `Haul` -> `Miner` -> AC5's per-tick holder check (scenario.rs:1731);
  4. professions from `spawn_rng` -> `spawn_positions_for_seed_42_are_pinned` (worldgen.rs:423);
  5. no hauler in the pool -> the 64-seed test, "seed 0" (worldgen.rs:70);
  6. `from_save` ignores the profession -> `save_load_then_tick_matches_never_saved` professions compare (save_load.rs:174);
  7. bridge sends `None` -> `save_then_load_rewinds_every_client` "every dwarf carries a profession" (serve.rs:551);
  8. roster drops the trade -> `each_trade_word_sits_after_its_own_dwarfs_name_...` (client.rs:1736);
  9. exhaustion `return`s (`main`'s `break 'jobs`) -> AC12's tick-100 stamps (lib.rs:3567);
  10. one shared budget -> AC11's tick-100 claim (lib.rs:3494);
  11. stamp-and-stop on a shared budget -> AC11's tick-100 claim (lib.rs:3500). **Superseded by patch pass 1:** row 11 is mutA on a
      budget per dwarf and dies at "miner 1 claims the dig miner 0 exhausted on". mutB and mutC were added as rows 13 and 14, giving 14/14.
      Patch pass 2 added rows 15 and 16, giving 16/16. See Review Findings for the current killing assertions;
  12. sat-out rule dropped -> AC12's tick-100 stamps (lib.rs:3567).
  Re-pointed: fresh budget per search -> AC12's stamps (lib.rs:3562); unreachable lower id -> lib.rs:3261; walks descending -> lib.rs:3200;
  from_save ignores identity -> save_load.rs:173; tui never draws the roster -> client.rs:1688; spawn consumes worldgen -> worldgen.rs:423.
- **Rows 9 and 11 deviate from the story's mapping, deliberately.** Under a budget per dwarf nobody in AC11's 11k-cell plates ever
  exhausts, so an exhaustion-arm sabotage is unreachable there and would SURVIVE. Row 9 is pointed at AC12, whose 55k area exhausts.
  Row 11 restores the shared budget the issue's candidate was written against, then applies stamp-and-stop. (Superseded by patch
  pass 1: that row duplicated row 10, and it is now mutA.)
- **AC12's later assertions, shown to fail on their own** (trap 1): rows 9, 12 and the fresh-budget row all die at the tick-100 stamp
  assert, so in a scratch worktree (never the shared tree) that assert was removed and each sabotage re-run. Control passed; sat-out
  dropped -> "tick 110 reaches the reachable dig" (never claimed); exhaustion `return` -> the same; fresh budget per search ->
  "tick 101: one exhausted dig per tick" (claimed early). Worktree removed after.
- **Live recipe (AC9),** fresh `simd 7530` each run, on `5f4ad04`:
  - GREEN, three runs, identical: `backlog: tick 13 marks 25 pile cells 9` / `FIRST DELIVERY tick 186 marks_left 17 of 25`.
  - Deliberate RED (row 1 applied in a scratch worktree, its own target dir): `BACKLOG EXHAUSTED tick 220, no stone on the pile yet` /
    `FIRST DELIVERY tick 299 marks_left 0 of 25`, exactly the creation RED. GREEN re-run on the clean build after: 17 of 25.
- **Roster instrument (AC8),** fresh daemon: `tui 7530 --frames 1 --z 9` row reads
  `Nain woodcutter  Ori hauler  Bifur miner  Frar hauler  Dori miner` (DEFAULT_SEED `[W,H,M,H,M]`); the same row under `NO_COLOR`.
- **#159 commented and retitled** (Wolf: post now): https://github.com/jeicei75/frostvein/issues/159#issuecomment-5947062459 ; title is now the single-area residual; the issue stays OPEN.
- **FULL GATE GREEN** on `3f705b3`, `RUST_TEST_THREADS=1 scripts/gate.sh`, 3161 s (pixel guards 2906 s). Only the story record changed after it.

### Completion Notes List

- Task 1: `Profession` component, `STREAM_PROFESSION`, shuffled 2/2/1 pool in `generate`, `professions()`, `SavedDwarf.profession`
  (no serde default). Tests: `every_world_has_two_miners_two_haulers_and_one_woodcutter` (AC1),
  `different_seeds_assign_professions_differently` (AC1; DEFAULT_SEED `[W,H,M,H,M]` vs 42 `[W,M,H,H,M]`), and `professions()` added to
  three determinism/save tests (AC6).
- Task 2: `fn trade(JobKind)` (exhaustive), the trade `continue` first in the dwarf loop, AD-12 comment and spine amendment.
  Tests: `claim_jobs_takes_fifo_within_a_trade` (AC2), `a_job_with_no_free_dwarf_of_its_trade_gets_no_retry_stamp` (AC3).
- Task 2b: `budgets: Vec` per dwarf, `sat_out` rule, exhaustion `continue`s, label `'jobs` removed (unused), NOTE rewritten.
  The RED tests were committed together with the fix, not before it.
- Task 3: AC11/AC12 unit tests, and scenario tests `hauling_starts_while_the_dig_backlog_is_still_queued` (AC4) and
  `each_trade_holds_only_its_own_jobs_and_the_woodcutter_wanders` (AC5).
- Deviations: the story's three failing unit tests differed from reality (see Debug Log). `make_standable`-style fixtures in AC4/AC5
  anchor on the first Miner from `professions()` instead of `dwarves()[2]`. AC12 uses `Dims::DEFAULT` (128x128x32), which fits the skeleton.
- Tasks 4-5: `Entity` literals got `profession: None` by compiler-driven sweep: 45 in the first pass (gui 9, tui 24, client-core 7, simd 3, protocol 2)
  and 16 gui tests, 61 in all. Tests: `an_identified_dwarf_round_trips...` and `every_material_and_tile_variant_has_a_pinned_wire_name` (protocol),
  `save_then_load_rewinds_every_client` and NEW `save_without_a_profession_is_logged_and_the_daemon_keeps_ticking` (AC6, AC7; serve.rs),
  `every_profession_has_its_one_spelling` (client-core), `the_roster_shows_a_trade_after_the_name_and_a_dwarf_without_one_shows_his_name_only`
  (view.rs), `each_trade_word_sits_after_its_own_dwarfs_name_...` and `the_trade_words_survive_no_color` (AC8; client.rs).
  The pre-commit hook failed on the untracked `12-4.sh` (row 11 anchor matches 2, not 3), which is Task 6's.
- **AC10 CLOSED at the seat** (Wolf, 2026-10-02: "1 ok" to the seat step, `12-4-signoff/vehicle-card.md` a-c). He then asked for the roster and trades in the gui too; not built here (story scope: no gui display), routed by his ruling.

### File List

- crates/sim-core/src/lib.rs
- crates/sim-core/src/save.rs
- crates/sim-core/tests/worldgen.rs
- crates/sim-core/tests/save_load.rs
- crates/sim-core/tests/scenario.rs
- _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md
- _bmad-output/implementation-artifacts/mutations/12-2.sh
- _bmad-output/implementation-artifacts/mutations/2-2-dwarves-wander-the-frost.sh
- _bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh
- _bmad-output/implementation-artifacts/12-4-every-dwarf-has-a-trade.md
- crates/protocol/src/lib.rs
- crates/simd/src/bridge.rs
- crates/simd/tests/serve.rs
- crates/client-core/src/lib.rs
- crates/tui/src/view.rs
- crates/tui/tests/client.rs
- crates/gui/src/capture.rs
- crates/gui/src/ingest.rs
- crates/gui/tests/capture.rs
- crates/gui/tests/headless.rs
- README.md
- _bmad-output/implementation-artifacts/mutations/12-4.sh
- _bmad-output/implementation-artifacts/12-4-signoff/vehicle-card.md
- _bmad-output/implementation-artifacts/sprint-status.yaml
- _bmad-output/implementation-artifacts/deferred-work.md
- _bmad-output/planning-artifacts/epics.md

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-02 | Code review run 2 on `05a16a4..b0c1bd0` (4 layers, none timed out). 0 REWORK: the patch pass 1 closure table held, and mutA/mutB/mutC were re-killed at the named assertions by three layers. 4 NEW findings (0 HIGH, 1 MED, 3 LOW), so the stopping rule ends the static audit. All 4 were left as action items (Wolf: option 2): mutD (sat-out `continue` -> `break`) survives sim-core, the exhaustion precondition is unasserted, and two record fixes. 4 dismissed. Live 17 of 25 re-observed; AC11/AC12 never fire on DEFAULT_SEED. Status in-progress. Review cost $7.51 over 165 turns (subagents 52.0% of tokens). Review caches reaped from /tmp: 21.5 GB, 14.2 GB freed. |
| 2026-10-02 | Review patch pass 1 (fresh session): all 4 patches landed (`6938841`, `ef7f523`, `0b38aca`). 12-4.sh 14/14 KILLED (row 11 re-pointed to mutA; mutB/mutC added as rows 13/14). FULL GATE GREEN 3093 s on `0b38aca`. Status review, for code review run 2. Patch cost $2.76 over 62 turns. |
| 2026-10-02 | Code review run 1 (4 layers, none timed out, 0 HIGH). Decision 1 (2/2/1 digs ~2.3x slower) accepted by Wolf, with a NOTE on 12.6. 4 patches left as action items for a fresh-session patch pass. 6 deferred to `deferred-work.md`, 13 dismissed. Status in-progress. Review cost $11.85 over 232 turns (subagents 69.6% of tokens). The review build caches were reaped from /tmp: 20.0 GB, 13.8 GB of it freed. |
| 2026-10-02 | AC10 passed at Wolf's seat ("1 ok", `12-4-signoff/vehicle-card.md` a-c, recorded in `40b6535`). He asked for the roster and trades in the gui too; that went to 12.6 (`05a16a4`). |
| 2026-10-02 | Dev done (Sonnet 5.5 subagents + Opus orchestrator): professions (2M/2H/1W, own stream), trade filter, #159 fix B′ (budget per dwarf + sat-out rule), wire field, tui roster trades. 12-4.sh 12/12 and 6 re-pointed rows KILLED; live recipe GREEN 17 of 25 / RED 0 of 25; #159 commented + retitled; full gate GREEN 3161 s on `3f705b3`. Status review; AC10 awaits the seat. |
| 2026-10-02 | Task 0 ruled by Wolf: 2M/2H/1W pool, the grey-trade roster, old saves refused, and #159 FOLDED IN with fix B (a budget per dwarf). #159 reproduced on `main` (never claimed at N = 10/25/60). Prototypes measured A (fails N > 20) and B. B's sat-out rule was found while writing AC12 and verified (claim at tick 110; sabotages give never / tick 100). AC11, AC12 and Task 2b added. |
| 2026-10-02 | Story created on `f4d9ba5`. RED reproduced in the sim (seed 42: 0 marks left at the first delivery) and on the live daemon (`first_delivery.py`: 0 of 25). A throwaway prototype went GREEN (12 marks left in the sim, 19 of 25 live) and mapped the fixture fallout. |
