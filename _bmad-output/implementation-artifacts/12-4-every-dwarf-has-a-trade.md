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
- [ ] **Task 1: sim-core profession (AC1, AC6).**
  - [ ] `lib.rs`, beside `Identity`:
        `#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)] pub enum Profession { Miner, Hauler, Woodcutter }`.
        Its own component, not a field of `Identity`, because 12.6 makes it mutable.
  - [ ] `const STREAM_PROFESSION: u64 = 0x5052_4f46_4553_534e; // "PROFESSN"` beside `STREAM_IDENTITY`
        (`lib.rs:31`). In `generate` (`:1309`):
        - seed `ChaCha8Rng::seed_from_u64(seed ^ STREAM_PROFESSION)`;
        - `shuffle` the pool `[Miner, Miner, Hauler, Hauler, Woodcutter]`;
        - pass it into `spawn_dwarves` (`:1810`) beside `identities`, and add it to the spawn bundle
          (`:1834`).
        Draw nothing from `spawn_rng` or `identity_rng`.
  - [ ] `pub fn professions(&self) -> Vec<(Id, Profession)>`, sorted by id: a sibling reader like
        `identities()` (`:1752`). Do not widen `dwarves()`.
  - [ ] `save.rs`: `SavedDwarf` gains `pub profession: Profession`, with no `#[serde(default)]` and a
        `// NOTE:` like `identity`'s. `to_save` (`:1365`) and `from_save` (`:1463`) carry it.
        - **Trap:** a dwarf without `Profession` drops silently out of `to_save`'s `filter_map` AND
          out of `claim_jobs`' query. Insert it at BOTH spawn sites.
        - Fix the 6 `SavedDwarf { .. }` literals. A literal whose dwarf holds a job gets that job's
          trade: `two_carriers_racing…` → both `Hauler`.
  - [ ] Tests:
        - NEW `worldgen.rs` test over seeds `0..64`: each world's `professions()` is exactly 2/2/1.
        - NEW: `DEFAULT_SEED` and 42 assign differently. The prototype's constant gave
          `[M,M,W,H,H]` vs `[M,H,H,W,M]`; yours will differ, so pick two seeds that do.
        - `same_seed_produces_identical_worlds` (`worldgen.rs:54`), `save_load_then_tick_matches_never_saved`
          (`save_load.rs:9`, list at `:173`) and `same_seed_and_commands_remain_deterministic`
          (`scenario.rs:1612`, beside `:1644`) also compare `professions()`.
        - `spawn_positions_for_seed_42_are_pinned` (`worldgen.rs:386`) and the identity tests pass
          untouched.
- [ ] **Task 2: the claim filter (AC2, AC3).**
  - [ ] `fn trade(kind: JobKind) -> Profession`: an exhaustive `match` with no wildcard
        (`Dig | Channel => Miner`, `Haul { .. } => Hauler`). 12.7's `Cut` then fails to compile until
        it is given a trade.
  - [ ] `claim_jobs` (`:422`): add `&Profession` to the dwarf query (`:431`), and fix the three tuple
        patterns (`:435`, `:438`, `:442`). In the dwarf loop (`:482`), `continue` when
        `trade(job.kind) != profession`.
        - **Do this BEFORE `attempted = true` (`:493`) and before the component check (`:495`).**
          After it, every idle miner would stamp a 20-tick `retry_after` on each haul it skips, and
          hauls would lag a whole cooldown behind a free hauler (AC3).
  - [ ] Rewrite the AD-12 comment (`:418-420`). It says "one shared node budget"; it must now say
        claiming filters by trade and spends one budget per dwarf (Task 2b).
  - [ ] Amend AD-12 in `planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md:194`
        under the "Amended YYYY-MM-DD:" convention AD-10 uses (`:168`). The amendment says:
        - claiming considers a dwarf only for jobs whose trade (`trade(JobKind)`) is its profession;
        - FIFO and id order hold within a trade;
        - "Job-kind stories add variants and execution systems — never claiming logic" now reads
          "…their variant, its execution and its `trade` arm — never a second claiming system".
- [ ] **Task 2b: #159, a budget per dwarf (AC11, AC12). RED first:** write both tests (Task 3) before
  this change, run them, and record the failures in the Debug Log. AC11 expects `None` and AC12
  expects every stamp at 0 (M2-27).
  - [ ] In `claim_jobs`:
        - replace `let mut astar_nodes_remaining = MAX_ASTAR_NODES;` (`:449`) with
          `let mut budgets = vec![MAX_ASTAR_NODES; dwarves.len()];`, indexed like the sorted `dwarves`;
        - delete the per-job `if astar_nodes_remaining == 0 { break; }` (`:459-461`);
        - iterate the dwarves with `enumerate()` and pass `&mut budgets[slot]` to both
          `astar_with_budget` calls (`:503-509`, `:521-527`);
        - change both `(None, true, _) => break 'jobs` arms (`:515`, `:533`) to `continue`.
  - [ ] **The sat-out rule.** Add `let mut sat_out = false;` per job. Inside the eligible-dwarf branch
        (after the trade filter and the reaction-delay check), and BEFORE `attempted = true`:
        `if budgets[slot] == 0 { sat_out = true; continue; }`. The stamp at `:547` becomes
        `if attempted && !assigned && !sat_out`.
        - A dwarf that exhausts DURING a search did attempt the job, so that job IS stamped. Without
          that, he would re-exhaust on the same job every tick.
  - [ ] Rewrite the `// NOTE:` at `:450-454`:
        - the bound is now (idle dwarves) × `MAX_ASTAR_NODES` per tick;
        - the residual is one area over the budget with more than `RETRY_COOLDOWN` unreachable jobs
          ahead of a reachable one (#159).
  - [ ] **Replace** `claim_jobs_bounds_aggregate_astar_expansions_per_tick` (`lib.rs:3210`). It pins
        #159's broken shape: its first assert IS the bug. Its five-plate fixture becomes AC11's test.
        Re-point mutation row `3-2-the-dig.sh:938` ("each claim search gets a fresh node budget") to
        AC12's test, which kills that sabotage at tick 100 vs 110.
- [ ] **Task 3: sim tests (AC2-AC5, AC11, AC12).** Existing first:
  - [ ] Fix the three unit tests named in "Found at creation". Add a `set_profession(world, id, p)`
        helper in the `tests` mod; in-crate tests may `insert` the component directly. Give the
        expected claimant the right trade, and keep each test's assertion as it is. **No public
        setter**: 12.6 adds the command.
  - [ ] New unit tests (`lib.rs` tests mod):
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
  - [ ] New scenario tests (`tests/scenario.rs`), skeleton below:
        - `hauling_starts_while_the_dig_backlog_is_still_queued` (AC4);
        - `each_trade_holds_only_its_own_jobs_and_the_woodcutter_wanders` (AC5): the same world, run
          to the first delivery plus 200 ticks. Check `claims()` × `jobs()` × `professions()` every
          tick, and count the woodcutter's position changes.
  - [ ] Run the whole sim-core suite. Any other test that now fails because of WHICH dwarf claims
        gets a fixture fix, not a weakened assert. List each one in the Debug Log.
- [ ] **Task 4: protocol + simd (AC7; the wire diff above).**
  - [ ] `protocol`: the enum and the field. Fix every `Entity { .. }` literal (about 63; let the
        compiler list them) with `profession: None`, except the dwarf fixtures a test reads.
  - [ ] Pin tests in `protocol`:
        - the existing entity and delta literals stay byte-identical;
        - the identity literal at `:320` gains `,"profession":"miner"` and round-trips;
        - `every_material_and_tile_variant_has_a_pinned_wire_name` (`:510`) gains a `Profession`
          block.
  - [ ] `bridge.rs`: `fn profession(v: sim_core::Profession) -> protocol::Profession`, an exhaustive
        `match` beside `dwarf_colour` (`:191`). `dwarf_entities` (`:133`) sets it from
        `world.professions()`, looked up by id the way `identities` is.
  - [ ] Extend `save_then_load_rewinds_every_client` (`serve.rs:544`): the connect snapshot gives all
        five dwarves `Some(profession)` with each trade present, and the post-load snapshot carries
        the same `(id, profession)` set. No new simd save validation: any trade mix is a valid save
        (12.6).
- [ ] **Task 5: client-core + tui (AC8). Format per Task 0.**
  - [ ] `client-core`: `pub fn profession_text(p: protocol::Profession) -> &'static str`, an exhaustive
        `match` beside `dwarf_name_text` (`lib.rs:17`). It is the only spelling, and 12.6's gui reads
        it too. Pin the three spellings in a unit test.
  - [ ] `view.rs` roster (`:429-443`): after each name, a blank, then `profession_text` in `STATUS_TEXT`.
        A dwarf with `profession: None` shows his name only. No layout row moves.
  - [ ] **The instrument is `tui --frames N` (real binary).** Extend `capture_roster` (`client.rs:1555`):
        - the two stub dwarves get professions;
        - its second frame changes one dwarf's profession as well as swapping identities;
        - a new test asserts each trade word sits after its own dwarf's name, and that the changed
          trade shows in frame 2;
        - the `NO_COLOR` test (`:1699`) also asserts the trade words.
  - [ ] README tui paragraph (`README.md:70`): the roster also names each dwarf's trade.
- [ ] **Task 6: the record.**
  - [ ] Write `_bmad-output/implementation-artifacts/mutations/12-4.sh`. Every row must be KILLED, by
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
        11. stamp-and-stop (the issue's candidate) → AC11's test (25 > 20);
        12. the sat-out rule dropped (`&& !sat_out` removed) → AC12's test (never claimed).
        A fresh budget per search is `3-2-the-dig.sh:938`'s row, re-pointed to AC12's test (claimed
        at tick 100 instead of 110).
        Put each killing assertion where only its mutation reaches it ([[strengthened-test-needs-remutation]]).
  - [ ] Re-point any older mutation row the change breaks (12.2 re-pointed four tables), and list them.
        `3-2-the-dig.sh:938` is known (Task 2b). Check 12-3.sh's rows too; they sabotage the
        component cache that `sat_out` sits beside.
  - [ ] Comment on #159: the measured cause, fix B′, the AC11/AC12 tests, and the residual that stays
        open. Retitle it to the residual: "A dwarf in one walkable area over MAX_ASTAR_NODES starves a
        reachable job behind more than RETRY_COOLDOWN unreachable ones". The PR body says
        `Refs #159`, never `Closes`.
- [ ] **Task 7: the live recipe and the seat (AC9, AC10), then the full gate.**
  - [ ] Run the Verification recipe on the branch, GREEN then the deliberate RED, and record both
        outputs in the Debug Log.
  - [ ] Write `12-4-signoff/vehicle-card.md` in the seat's launch form:
        - in WSL, `simd 7451`;
        - in PowerShell, `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`;
        - in WSL, attach `tui 7451`.
        Steps:
        - (a) the roster names a trade for each dwarf;
        - (b) channel a block of the camp floor and drag a stockpile west of the fire;
        - (c) a stone reaches the pile while channel marks remain.
        Tell him old `frostvein.save` files will not load.
  - [ ] Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` (~75 min, [[gate-ooms-at-default-parallelism]]).

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

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-02 | Task 0 ruled by Wolf: 2M/2H/1W pool, the grey-trade roster, old saves refused, and #159 FOLDED IN with fix B (a budget per dwarf). #159 reproduced on `main` (never claimed at N = 10/25/60). Prototypes measured A (fails N > 20) and B. B's sat-out rule was found while writing AC12 and verified (claim at tick 110; sabotages give never / tick 100). AC11, AC12 and Task 2b added. |
| 2026-10-02 | Story created on `f4d9ba5`. RED reproduced in the sim (seed 42: 0 marks left at the first delivery) and on the live daemon (`first_delivery.py`: 0 of 25). A throwaway prototype went GREEN (12 marks left in the sim, 19 of 25 live) and mapped the fixture fallout. |
