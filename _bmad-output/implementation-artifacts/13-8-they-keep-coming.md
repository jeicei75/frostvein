---
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
baseline_commit: e2e1882d
---

# Story 13.8: They Keep Coming

Status: ready-for-dev

## Story

As the boss,
I want new dwarves to arrive at the valley edge over time until there are two hundred, each with a name of its own,
so that the fortress grows while I watch.

## Branch: off local `main`, which is ahead of `origin/main`

Branch `story-13-8-they-keep-coming` was cut from local `main` `e2e1882d`, which carries the unpushed 13.7 done-mark. The mark rides into this PR, as `483b95b1` did for 13.7.

**Standing ACs that bind:** Epic 13's M3 ACs 1–8 (**one full gate per story, at PR**), plus M4 standing ACs 10, 11, 12, 15 and 18. AC 16 binds only its closing half: Wolf views the result live (AC 13). There is no new UI, only longer name text in the existing slots. AC 13 (needs) and AC 14 (stock) do not apply. AC 17 does not apply either: this story adds no new kind of rendering, and 13.6 measured 200 dwarves.

**Wire diff:**
- `protocol::Identity` gains `epithet: Epithet`, a new closed enum, snake_case (AD-6).
- No new message, and no other field.

**Saved fields (three):**
- `Identity.epithet`.
- `SavedDwarf.arriving: Option<Pos>`.
- `SaveState.arrivals: bool` (under Q1 a).

Every state hash therefore moves (memory: `state-hash-covers-save-encoding`). AC 10 reads the reach with those fields stripped from the encoding.

## Task 0 — Wolf rules before dev

Each question has a recommendation. The ACs and Tasks are written for it, and a different answer amends the ACs it names. **RULED 2026-11-03 (Wolf, "Yes for all recommendations"): Q1–Q5 all (a).** The epic's Story 13.8 is amended under Q1 (a).

- **Q1. Do the committed scenarios get arrivals?** The 128² rows run 3,000–10,006 ticks. Three of them start from Wolf's seat saves: `dig-loop` (#203), `stuck-tunnel` (#206) and `seat-slow` (#204). 13.17 and 13.18 need those three as reproducers.
  - (a) **Recommended (← RULED 2026-11-03):** arrivals are a saved world setting. A scenario `start` with no `"arrivals"` key has none: it is the world the file was recorded on, as 13.6 Q2 (a) did for `dims`/`crew`. `"arrivals": true` turns them on. A fresh game (`simd` with no scenario) always has them. This keeps every committed file byte-for-byte unchanged, so `main`'s `simd` still reads them and all 11 rows can read `same` under AC 10. Amend the epic's "Given any map size" AC to: "arrivals run on any map size, 128² included, in a fresh game and in any scenario that asks for them; committed scenarios keep their recorded world."
  - (b) As the epic is written: arrivals run in every world. On each 128² row the crew then grows from 5 to about 50–100 inside the window, so every row is re-pinned (standing AC 11), and each row's cost and meaning change. Whether #203, #204 and #206 still reproduce is unmeasured. No saved `arrivals` field is needed.
- **Q2. What is "the first group arrives at embark"?**
  - (a) **Recommended (← RULED 2026-11-03):** today's starting crew, spawned at the camp as it is now. Every spawn, identity and profession pin passes unchanged, and the embark guarantee is AC 1.
  - (b) The embark group also spawns at an edge and walks in. That moves every scenario's outcome and every spawn pin.
- **Q3. Does a walking arrival take work before it reaches the camp?**
  - (a) **Recommended (← RULED 2026-11-03):** yes. An arrival is an idle dwarf from the moment it appears, so `claim_jobs` can give it a job of its trade on the way. That is "joins the work". With no such job, it walks to the camp. `claim_jobs` itself is unchanged (AD-12).
  - (b) No. `claim_jobs` skips a dwarf that is still arriving, so it reaches the camp first.
- **Q4. Cadence and group size (standing AC 18).**
  - (a) **Recommended (← RULED 2026-11-03):** one group every `ARRIVAL_PERIOD = 1_000` ticks (100 s at 1x), the first at tick 1,000. Group size is uniform in 5..=20 and capped at `CREW_CAP − crew`, with `CREW_CAP = 200`. From crew 5 this averages about 16 groups, so the crew reaches 200 near t16,000: about 27 min at 1x, or 7 at fast4x. The seat may retune the numbers (AC 13).
  - (b) Other numbers.
- **Q5. The epithets (FR57).** 16 epithets × 16 names = 256 full names, so 200 always fit. A full name reads `Durin Frostbeard`.
  - (a) **Recommended list (← RULED 2026-11-03):** Ironhand, Stonefoot, Frostbeard, Coldforge, Deepdelver, Greyhelm, Ashbrow, Flintback, Rimecloak, Hammerfall, Copperbeard, Icevein, Stoutheart, Blackanvil, Emberkeep, Snowmantle. None is a Tolkien epithet: Oakenshield, Ironfoot, Stonehelm and Longbeard are left out on purpose.
  - (b) Wolf edits the list. Keep at least 13 entries, because 16 × 13 = 208 ≥ 200.

## Found at creation (2026-11-03, `main` code `e2e1882d`)

The probes ran in a throwaway scratch worktree on release builds. Raw lines are in `13-8-signoff/creation-probes.txt`, and the patches in `13-8-signoff/prototypes/`.

- **A. Edge cells reach the camp.** On DEFAULT_SEED and 8 + 4 other seeds, 507–510 of the 128² perimeter cells and 1,016–1,022 of the 256² perimeter cells are in the camp's walkable component, on all four sides. So the AC can happen.
  - Walk length at the step pace (about 11 ticks a step):
    - 128²: median 95 steps (≈ 1,050 ticks), longest 163.
    - 256²: median 191 steps (≈ 2,100 ticks), longest 295.
- **B. A blanket "walk home when far" rule is wrong.** Idle, pathless dwarves standing more than `WANDER_RADIUS` from their `home`:
  - The ten 128² rows have 0.
  - two-hundred-on-256 has 152 of its 200 (697 wander ticks). They are 13.6's BFS-spill spawns, whose home is the camp. That crowd is #227, which is routed to 13.11.

  So the walk-in rule must apply to arrivals only (the `Arriving` marker, Task 2).
- **C. `clear_paths` would strand an arrival.**
  - `clear_paths` (`lib.rs:1622`) removes every `Path`, idle exit paths included.
  - It runs on every dig and cut completion (`:1348`, `:2552`) and at `:3167`.
  - An arrival walking in on an idle `Path` therefore loses it at the first dig anywhere.
  - Pathless, with its home at the camp, it has no wander candidate, so it freezes: the `release_claim` NOTE (`:1596-1606`) describes the same freeze.
- **D. What already exists:**
  - Names already travel and show, but only the 16 base names:
    - wire `Identity` (`protocol/src/lib.rs:92-96`), bridged at `simd/src/bridge.rs:188-190`;
    - the TUI roster row (`tui/src/view.rs:462-490`);
    - the gui name slot, both the selected dwarf and the all-dwarves list (`gui/src/ingest.rs:1878-1915`).
  - `simd`'s load validator carries the NOTE "13.8 makes names unique and restores a name check here" (`simd/src/main.rs:757-758`).
  - The three committed seat saves are JSON, crew 5, all on DEFAULT_SEED's identities (Nain, Ori, Bifur, Frar, Dori).

## Acceptance Criteria

1. **The embark group holds every trade (FR56, standing AC 15).**
   - Every generated world with a crew of 3 or more holds at least one miner, one hauler and one woodcutter.
   - `simd` refuses a crew below 3, naming the cause, both for `--crew` and for a scenario's `crew`.
   - A crew of 5 or more draws exactly as today: `crew_200_holds_the_two_two_one_mix` and every spawn and identity pin in `tests/worldgen.rs` pass unchanged.
2. **Groups arrive on the cadence, and the crew stops at 200 (FR56).**
   - In a world with arrivals on, a group of 5–20 dwarves appears every `ARRIVAL_PERIOD` ticks, the first at `ARRIVAL_PERIOD`.
   - The group stands on distinct free cells, starting at an edge cell (x ∈ {0, W−1} or y ∈ {0, H−1}) in the camp's walkable component.
   - A scenario test counts the crew at fixed ticks and shows it never passes 200.
3. **An arrival walks in, survives terrain churn, and joins the work (FR56, Q3 a).**
   - With no job of its trade, every arrival ends its walk standing within `WANDER_RADIUS` of its destination near the camp, within 4,000 ticks of appearing. That holds while a cell somewhere else is dug every 8 ticks (`clear_paths` churn).
   - With a dig queued that only arrivals can take, an arriving miner claims it and digs it.
   - An arrival with no way to the camp ends its walk where it stands, and is never left walking forever.
4. **Identities and professions come from purpose-named seed streams (FR56, AD-7; mechanism AC, because NFR9 is about reproducibility).**
   - An arrival's identity and profession are drawn from streams derived from the world seed, and none of those draws shifts a worldgen, spawn, wander or embark draw.
   - The profession weights are 2:2:1 miner:hauler:woodcutter, a hardcoded constant.
   - Two worlds with the same seed, dims, crew and setting are identical (`to_save` bytes) after 3 arrival periods.
5. **Every full name is unique; colours repeat (FR57).**
   - At 200 dwarves, whether they come from `generate_with_crew(…, 200)` or from arrivals up to the cap, no two dwarves share a full name (name + epithet).
   - `simd` refuses a save holding a repeated full name, naming the cause.
   - A save whose base names repeat, but whose full names are unique, loads.
6. **Names reach both clients (FR57, NFR10).**
   - The wire `Identity` carries the epithet.
   - The TUI roster row shows `Name Epithet` for each dwarf (emitter `tui/src/view.rs:470-475`).
   - The Bevy client names a selected dwarf `Name Epithet` (emitter `gui/src/ingest.rs:1888-1898`), and that includes a dwarf that arrived after the snapshot.
7. **A save made while a group walks in reloads identically (NFR9, AD-11).** A world saved at a tick when at least one dwarf holds `Arriving`, loaded and stepped 300 ticks, equals the never-saved world (`to_save` bytes).
8. **The arrivals setting decides (Q1 a; seam exercised).**
   - A world with arrivals off holds its starting crew through 3 arrival periods.
   - The same seed with arrivals on grows.
   - A scenario `start` of `"arrivals": true` turns them on, and its absence turns them off.
   - A fresh `simd` game has them on.
   - A save → load keeps the setting.
   - `"arrivals"` beside a `"save"` start is refused, naming the cause.
9. **Old saves (AD-11, no migration).**
   - The three committed saves are patched (an epithet for each dwarf, `arriving: null`, `"arrivals": false`), and their scenarios still load.
   - A save with the `epithet`, `arriving` or `arrivals` key removed is refused, with a positive control.
10. **Bench delta (standing ACs 10–12).**
    - `RUNS=5 scripts/tick-bench.sh main HEAD` over all 11 committed files reads `differ` on every row, because of the new saved fields.
    - A throwaway build that drops those fields from the encoding reads `same` on all 11 against `main`.
    - The two-hundred row's mean and p99 delta are recorded against the floor.
11. **The instrument shows the crew (Task 6).**
    - `simd --bench`'s end line gains `crew=N arriving=K`.
    - `scenarios/they-keep-coming.json` ends at `crew=200 arriving=0`.
    - A test drives the binary: the one-dig fixture reads `crew=5 arriving=0`, and an arrivals variant past `ARRIVAL_PERIOD` reads `crew>5`.
12. **The mutation table kills** (Task 8), and `python3 scripts/audit-mutations.py` exits 0.
13. **The seat (FR57).** Wolf watches a fresh game's crew grow, selects an arrival and reads its full name, then rules whether 200 dwarves need a crew list. His answer is recorded in his words. A "yes" becomes an issue or a story, not part of this one.

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings (Q1–Q5).** RULED 2026-11-03: all (a); epic amended; implementer still to be asked. Record the answers in the Change Log and amend the ACs they name. Under Q1 (a), amend the epic's Story 13.8 "Given any map size" AC with an `AMENDED 2026-11-03` line. Then ASK which implementer to use (memory: `dev-implementer-is-sonnet-not-codex`).
- [ ] **Task 1: unique names and the embark guarantee (AC 1, 5).** UPDATE `crates/sim-core/src/lib.rs`.
  - [ ] NEW `pub enum Epithet` (16 variants, Q5) with `pub const ALL: [Self; 16]`, beside `DwarfName` (`:206-245`). `Identity` (`:261-264`) gains `pub epithet: Epithet`.
  - [ ] `generate_with_crew` (`:2913-2925`): keep the `names` and `colours` shuffles and `names[i % 16]` / `colours[i % 5]` exactly as they are. Then draw each dwarf's epithet uniformly from the epithets not yet used with its base name, continuing on `identity_rng` after the colour shuffle. Replace the "13.8 makes names unique" NOTE.
  - [ ] Professions (`:2930-2943`): crew ≥ 5 keeps today's cycled `[M, M, H, H, W]` and its one shuffle. Crew 3–4 take `[M, H, W, M][..crew]` before the same shuffle.
  - [ ] UPDATE `crates/simd/src/main.rs`:
    - `check_crew` (`:97`) refuses crew < 3, naming why ("the first group holds one of each trade").
    - The load validator (`:757-758`) refuses a repeated `(name, epithet)` with `save reuses dwarf name {name:?} {epithet:?}`. Colours stay free. Rewrite the NOTE.
    - Run `audit-mutations.py` BEFORE the first code commit. `13-6.sh` rows 12–13 anchor on `bail!("save reuses dwarf id …")` (memory: `deleting-code-retires-rows-in-the-same-commit`).
- [ ] **Task 2: arrivals in the sim (AC 2, 3, 4, 7, 8).** UPDATE `crates/sim-core/src/lib.rs` and `src/save.rs`.
  - [ ] Constants beside `WANDER_RADIUS` (`:35`), each a hardcoded tuning value (standing AC 18):
    - `ARRIVAL_PERIOD: u64 = 1_000`;
    - `ARRIVAL_GROUP: RangeInclusive<usize> = 5..=20`;
    - `CREW_CAP: usize = 200`;
    - the arrival profession weights `[(Miner, 2), (Hauler, 2), (Woodcutter, 1)]`;
    - `STREAM_ARRIVAL`, beside the other streams (`:28-34`).
  - [ ] The setting:
    - NEW resource `Arrivals(bool)`. `assemble` takes it.
    - `generate_with_crew` gains no parameter. A NEW `World::generate_with_arrivals(seed, dims, crew)`, or a setter, turns it on, and the daemon's fresh game uses it.
    - `SaveState.arrivals: bool` (no `#[serde(default)]`, like `goals_completed`, `save.rs:27-29`).
  - [ ] **One spawn helper.** Lift the bundle out of `spawn_dwarves` (`:3683-3699`): `Dwarf, Id, Pos, JobState, Wander, CurrentJob, Carrying, Identity, Profession`. Use the helper for the embark spawn and for arrivals, so `to_save`'s "a story that spawns dwarves a third way must keep the set intact" (`:2954-2957`) holds. Update that NOTE.
  - [ ] NEW system `arrive`, in the `.chain()`ed schedule (`:2845-2860`). It is world-advancing, so it skips while paused (AD-2). On `tick % ARRIVAL_PERIOD == 0`, with arrivals on and crew < `CREW_CAP`:
    - Seed this arrival's streams from `seed ^ STREAM_ARRIVAL` mixed with the tick, so no RNG state is saved. Keep the place-and-size draws apart from the identity and profession draws.
    - Pick one edge cell uniformly from the free standable perimeter cells in the camp's component. "Free" means no dwarf, item or emitter (BFS over `astar_neighbours` from the camp, as `spawn_dwarves` does at `:3655-3665`).
    - Draw the group size, capped at `CREW_CAP − crew`. The group takes the first free cells of a BFS from that edge cell, one per tile (FR49).
    - Each dwarf gets an unused full name (uniform over the unused `(name, epithet)` pairs), a colour (repeats allowed), and a weighted profession.
    - Each dwarf also gets an `Arriving { to }` component. `to` is a free cell from a BFS from the camp, skipping occupied cells and cells already given to this group.
    - With no reachable edge cell, skip this arrival, and add a `// NOTE:` saying so.
  - [ ] **The walk-in (Found C).** In `wander` (`:2681`), an idle dwarf that holds `Arriving` and no `Path` plans `astar_with_budget(…, &mut terrain.tiles.len())` to the walkable cells within `WANDER_RADIUS` of `to`.
    - Use the world budget, not `astar()`: an edge-to-camp search on 256² can pass 50,000 nodes (13.7).
    - The existing Path branch (`:2729-2753`) then walks it. Where it sets `home` at the path's end, also remove `Arriving`.
    - If no path is found, remove `Arriving` and set `home = here`.
    - `release_claim` (`:1588-1610`, the funnel for every job end) removes `Arriving`: a job took the dwarf, so it has joined the work.
    - No other dwarf's wander changes (Found B).
  - [ ] Save: `SavedDwarf.arriving: Option<Pos>` with `#[serde(deserialize_with = "Option::deserialize")]`, so a missing key is refused (memory: `serde-option-is-never-required`). `to_save` and `from_save` carry it.
- [ ] **Task 3: the wire and both clients (AC 6).**
  - [ ] UPDATE `crates/protocol/src/lib.rs`: NEW `Epithet` (snake_case), and `Identity.epithet`. Fix the raw-wire tests (`:419`, `:443`).
  - [ ] UPDATE `crates/simd/src/bridge.rs`: `fn epithet(…)` as an exhaustive `match` with no wildcard (`:241` is the shape), used at `:188-190`.
  - [ ] UPDATE `crates/client-core/src/lib.rs`: `epithet_text` beside `dwarf_name_text` (`:32`), plus a one-spelling test like `every_dwarf_name_has_its_one_spelling` (`:487`).
  - [ ] UPDATE `crates/tui/src/view.rs:470-475`: the roster shows `Name Epithet`. UPDATE `crates/gui/src/ingest.rs:1888-1912`: the selected slot shows `Name Epithet`. The all-dwarves list pads to the longest full name, replacing the NOTE's "padded to the longest (6)". No other gui change. Add a `name_readout_spans` unit test: a mirror whose dwarf first appears in a delta, after the snapshot, and is then selected, reads `Name Epithet`.
  - [ ] Fix the struct literals the new field breaks. They are compile-driven: `tui/src/view.rs`, `tui/tests/client.rs:1696`, `gui/tests/headless.rs:4951`, `sim-core/tests/{save_load,scenario}.rs`, and `simd/tests/serve.rs`.
- [ ] **Task 4: scenarios, saves, fresh game (AC 8, 9, 11).**
  - [ ] UPDATE `crates/simd/src/scenario.rs`:
    - `StartFile` (`:35-46`) gains `arrivals: Option<bool>`. A seed start passes it, with absent meaning off.
    - Refuse `arrivals` with a `save` start, the way `every_unusable_scenario_fails_loudly_naming_the_cause` (`bench.rs:308`) already expects for dims/crew.
  - [ ] UPDATE `crates/simd/src/main.rs:251`: the fresh game is generated with arrivals on.
  - [ ] Patch `scenarios/saves/{12-11-dig-loop-patched,12-12-seat-loop,12-12-seat-slow}.save` with a throwaway script:
    - each dwarf's `identity.epithet` = the epithet that HEAD's `World::generate(DEFAULT_SEED, Dims::DEFAULT)` gives the same-named dwarf;
    - `"arriving": null` on each dwarf;
    - `"arrivals": false` at the top.

    `main`'s `simd` ignores unknown keys (only `scenario.rs` sets `deny_unknown_fields`), so `tick-bench.sh main HEAD` still reads all three. Check that, and record it in the Debug Log.
  - [ ] NEW `scenarios/they-keep-coming.json`: `{"seed":4026891802,"dims":[256,256,32],"crew":5,"arrivals":true}`, no commands, `speed` normal. Set `ticks` to the last arrival's tick + 4,000, read from the run, so every walk can end. Its `about` names 13.8 and the expected end. `main` cannot read it (`deny_unknown_fields`), so its row is HEAD-only, like 13.6's. Record one run's wall time. If it is over 120 s, say so: every later story's tick-bench pays it.
- [ ] **Task 5: the sim tests (AC 1–5, 7–9).** NEW tests in `crates/sim-core/tests/` (`worldgen.rs`, `scenario.rs` and `save_load.rs` hold the precedents) or in `lib.rs mod tests` where they need private state. Fast tier only: **no debug test steps a 256² world with a growing crew** (13.6's debug cost rule). Use 128² DEFAULT_SEED.
  - [ ] AC 1: crews 3, 4, 5, 17 and 200 each hold all three trades. Crew 5's professions and identities equal today's: pin the five `(name, colour, profession)` triples read from `main` first.
  - [ ] AC 2:
    - From crew 5, arrivals on: crew at t999 is 5. At t1,000 it is 5 + n with n in 5..=20, and every new dwarf stands on a distinct cell. The first stands on an edge cell, and every one is in the camp component (reuse `camp_component`, `tests/worldgen.rs:952`).
    - From crew 190 (embark, arrivals on): crew 200 at t1,000, still 200 at t3,000.
  - [ ] AC 3, three tests:
    - (i) From the first arrival, step until every arrival has ended its walk. Assert each stands within `WANDER_RADIUS` of its `to`, within 4,000 ticks.
    - (ii) The same, with a `set_tile` on a far cell every 8 ticks plus `clear_paths` (the 13.7 churn shape, `lib.rs` test `a_far_claim_is_kept_through_terrain_churn_past_the_node_cap`). **RED first:** without the re-plan, an arrival freezes.
    - (iii) Every embark miner is set to hauler (`set_profession`), and a dig is designated near the camp. An arriving miner claims it, and it is dug.
  - [ ] AC 4: same-seed worlds equal after 3,000 ticks with arrivals on. Embark spawn positions and the wander stream are unchanged by turning arrivals on: the two worlds' states up to t999 are equal.
  - [ ] AC 5: `generate_with_crew(DEFAULT_SEED, DEFAULT, 200)` and the crew-190 arrivals world at t1,000 each have 200 distinct `(name, epithet)` and fewer than 200 distinct colours. In `crates/simd/tests/serve.rs`, keep `the_live_load_takes_a_save_whose_crew_repeats_names_and_colours` (`:1052`), whose base names still repeat, and assert its full names are unique. Add a refusal test: a save with two dwarves sharing a full name, refused through the live Load, with the cause in the log. Mirror both in `bench.rs:451`'s start.save path.
  - [ ] AC 7: save at the first tick when some dwarf holds `Arriving` (about t1,005). Load, step 300, and compare `to_save` bytes with the never-saved world.
  - [ ] AC 8: arrivals off vs on, same seed, at t3,000: crew 5 vs > 5. Save → load keeps the setting. Plus the scenario-start refusals in `bench.rs`.
  - [ ] AC 9: for each new key, strip it from serialised JSON and assert the load is refused. The control is the unstripped load.
- [ ] **Task 6: the instrument, RED first (AC 11).** The instrument is `simd --bench`'s end line (`crates/simd/src/scenario.rs:410-418`), which gains `crew=N arriving=K`. `N` is `world.dwarves().len()` and `K` is the dwarves holding `Arriving`, which needs a NEW `World::arriving_count()`. A human reads the crew grow with `target/release/simd --bench scenarios/they-keep-coming.json --runs 1`.
  - [ ] Tests in `crates/simd/tests/bench.rs`:
    - the one-dig fixture (`:19`) reads `crew=5 arriving=0`;
    - a `variant` with `"arrivals": true` and `ticks` 1,010 reads `crew` > 5 and `arriving` > 0;
    - the same variant without the key reads `crew=5`.
  - [ ] RED: on a scratch worktree, make `arrive` never fire (memory: `mutation-run-poisons-concurrent-builds`). The variant test fails at its `crew > 5` assert; paste the line. Then GREEN on HEAD. Paste `they-keep-coming`'s end line: `crew=200 arriving=0` is required. Any `arriving` > 0 is a stalled walk, not a pass.
- [ ] **Task 7: bench delta (AC 10).**
  - [ ] On an idle devpod: `RUNS=5 scripts/tick-bench.sh main HEAD` over the 11 committed files. Paste the table.
  - [ ] Throwaway `13-8-signoff/prototypes/strip-new-fields.patch`: `#[serde(skip_serializing)]` on the three fields. `skip_serializing` changes only the hash input, not the load. Build it on a scratch worktree and run `simd --bench` on all 11 files with `--runs 1`. Each `state=` must equal `main`'s. A row that differs is a real behaviour change: trace it and record why (standing AC 11).
  - [ ] Record two-hundred-on-256's mean and p99 delta against its floor, and its end line on both sides. They must match apart from `crew=`/`arriving=`, because arrivals are inert at the cap.
- [ ] **Task 8: mutation table (AC 12).** NEW `_bmad-output/implementation-artifacts/mutations/13-8.sh`, in `13-7.sh`'s shape, with every anchor readable by `audit-mutations.py`. Commit first, then run it alone (`env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …`). Read each panic line (memory: `mutation-test-name-prefix-trap`). Rows:
  1. `arrive` never fires → AC 2's growth test.
  2. The group size is not capped → the crew-190 test at its t1,000 assert.
  3. The epithet is always `ALL[0]` → AC 5's uniqueness test.
  4. The `Arriving` re-plan is removed → AC 3 (ii) at its arrival assert.
  5. `release_claim` keeps `Arriving` → AC 3 (iii), or say which test kills it. If none can, say so in the record rather than inventing a test.
  6. The full-name check is removed from the validator → the refusal test.
  7. The `Arrivals` flag is ignored (always on) → AC 8's off-world assert.
  8. Crew 3 takes `[M, M, H]` → AC 1.
  9. The edge filter is removed (the group spawns at the camp) → AC 2's edge assert.
  10. The scenario `arrivals` key is ignored → the bench variant test.
- [ ] **Task 9: the seat (AC 13).** The draft is skipped: there is no new UI. Write `13-8-signoff/vehicle-card.md` before handing the seat over. Push first (memory: `push-branch-for-vehicle-testing`, `push-before-seat-handoff`). Commands are written as the seat runs them:
  - WSL: `simd 7451` (a fresh 256² game, crew 5, arrivals on).
  - PowerShell, from the Windows checkout: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`.
  - Questions: Q1, do groups appear at the edge and walk in? Q2, does selecting an arrival name it in full? Q3, at around 200, does Wolf need a crew list (FR57)? Q4, do the cadence and group size feel right (Q4's constants)?

  Wolf speeds the game up with the speed keys. Record his answers in his words. A "yes" to Q3 becomes an issue (`gh issue create`, on Wolf's OK).
- [ ] **Task 10: gate and close.** Fast tier on every commit (pre-commit hook). ONE full gate at PR: `RUST_TEST_THREADS=1 scripts/gate.sh`, about 75 min (memory: `gate-ooms-at-default-parallelism`). Paste the block. Then:
  - the cost row: `python3 _bmad/scripts/session_tokens.py --phase dev --story 13-8-they-keep-coming`, then `phase_state.py clear`;
  - Status → review, on the story and on the board.

  The PR closes no issue: check `closingIssuesReferences` (memory: `closing-keyword-closes-whole-issue`).

## Verification

```bash
# AC 1-5, 7-9 (fast tier)
cargo test -p sim-core
cargo test -p simd --test bench --test serve
# AC 11: RED first (scratch worktree, `arrive` disabled), then GREEN; the human instrument:
cargo build --release -p simd && target/release/simd --bench scenarios/they-keep-coming.json --runs 1
#   expect the end line to read: ... crew=200 arriving=0     (crew=5 under the RED)
# AC 6: names on the TUI roster (fresh 128² daemon; tui takes a bare PORT)
target/release/simd 7461 --dims 128,128,32 & sleep 3
target/release/tui 7461 --frames 3 --z 9 > tui.out; kill %1
sed 's/\x1b\[[0-9;?]*[A-Za-z]//g' tui.out | tr -s ' ' \
  | rg -o '[A-Z][a-z]+ [A-Z][a-z]+ (miner|hauler|woodcutter)' | sort | uniq -c
#   expect 5 distinct `<Name> <Epithet> <trade>` lines, 3 each.
#   Run at creation on main e2e1882d (no epithets yet), with the pattern '[A-Z][a-z]+ (miner|hauler|woodcutter)':
#   3 Bifur miner / 3 Dori miner / 3 Frar hauler / 3 Nain woodcutter / 3 Ori hauler (exit 0)
# AC 10: bench delta, idle devpod
RUNS=5 scripts/tick-bench.sh main HEAD $(git ls-files 'scenarios/*.json' | rg -v they-keep-coming)
# AC 12
env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-8.sh
python3 scripts/audit-mutations.py
# gate (once, at PR)
RUST_TEST_THREADS=1 scripts/gate.sh
```

AC 6's epithet pattern cannot match before Task 3. The capture command itself ran at creation and printed the five bare names shown above. The dev agent must paste the epithet run, which needs 5 distinct lines. Run its RED as well: with `epithet_text` returning `""`, the epithet pattern counts 0, so the instrument says so. Task 6's recipe has its RED built in.

## Dev Notes

### Scope guardrails (do NOT)

- No crew list or roster UI: that is the seat's ruling, and its own story. No change to the TUI's input (NFR10). No gui change beyond the name text.
- No fix for #227: the spill spawns' `home` and the wander radius stay as they are. Only `Arriving` dwarves walk in (Found B). No change to `claim_jobs` (AD-12, Q3 a).
- No new trade, no colour change (colours repeat), and no new dependency. No optimization.
- No migration code for old saves: they are refused (AD-11 precedent). The three committed saves are patched as data.

### What already exists (build on it)

- The idle `Path` follower in `wander` (`lib.rs:2729-2753`): it walks at the step pace, waits on an occupied tile, gives up on an item, and sets `home` at the end. Arrivals ride it.
- `spawn_dwarves`' BFS spill (`:3651-3671`) is the free-cell walk to reuse for the group and for `to`. Plan the walk with `astar_with_budget` and `terrain.tiles.len()` (13.7: a world-sized budget cannot run out).
- Test helpers: `camp_component` (`tests/worldgen.rs:952`), `set_profession`, `stand_miners_at`, the churn fixture in `a_far_claim_is_kept_through_terrain_churn_past_the_node_cap`, and bench.rs's `variant` and `field`.
- The 13.6 `--dims`/`--crew` path and `check_crew` (`simd/src/main.rs:97-118`).
- Both clients already take a dwarf that first appears in a delta. The gui spawns a model for any new id (`gui/src/project.rs:2213-2263`), and entity sections are full resends (AD-8). So arrivals need no client spawn code.

### Key decisions & traps

- **`clear_paths` wipes an arrival's walk on every dig.** `Arriving` is what re-plans it (Found C). Test (ii) is the RED.
- **Do not make "far from home walks home" general.** On 256² it would move 152 of 200 dwarves (Found B, #227).
- **An arrival's walk can stall on an occupied tile,** because the follower waits forever. The walk ends anywhere within `WANDER_RADIUS` of `to`, not on `to` itself, so a crowd at the camp does not hold it. The end line's `arriving=0` (AC 11) is the alarm if it stalls.
- **Crew-5 draws must not move.** Draw epithets after the colour shuffle on `identity_rng`, and draw arrival randomness from its own streams. Otherwise every name, spawn and pixel pin moves.
- **A new saved field reads `differ` everywhere.** End lines are too coarse to show reach, so read it through Task 7's strip build.
- **`Option` save fields are never required by default.** Use `deserialize_with`, plus a strip-the-key test (AC 9).
- **The schedule must stay deterministic.** `arrive` iterates and tie-breaks by `Id`. It never iterates a hash container, and it uses one allocator (AD-9).
- **Debug cost:** no fast-tier test steps 256² past a few dozen ticks, and the 200-crew growth is read only from the release scenario (Task 4/6).
- **Probes and REDs go in scratch worktrees** built from inside them (memory: `cargo-target-dir-builds-cwd-sources`). Never run `git checkout --` over an uncommitted fix (memory: `sabotage-restore-trap`).

### Previous story intelligence

- 13.7: claim searches already use the world budget, so an arrival far out claims normally. Give the walk-in plan the same budget, since `astar()` stays capped at 50,000.
- 13.6: `generate_with_crew`'s draw order is pinned (`crew_200_holds_the_two_two_one_mix`). The #226 tests (`serve.rs:1052`, `bench.rs:451`) load crew-17 saves with repeated base names, and they must keep passing under the restored full-name check.
- 13.4: once a backstop lands, old rows can go inert. Re-run any re-pointed row; do not only audit it (memory: `backstop-makes-old-rows-inert`).

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs`: `Epithet`, `Identity`, constants, the `Arrivals` resource, the spawn helper, `arrive`, `wander`, `release_claim`, `to_save`/`from_save`, `arriving_count`, tests.
- UPDATE `crates/sim-core/src/save.rs` (`arrivals`, `arriving`).
- UPDATE `crates/protocol/src/lib.rs`, `crates/simd/src/bridge.rs`, `crates/client-core/src/lib.rs`, `crates/tui/src/view.rs`, `crates/gui/src/ingest.rs`.
- UPDATE `crates/simd/src/main.rs` (`check_crew`, validator, fresh game) and `crates/simd/src/scenario.rs` (`StartFile.arrivals`, end line).
- UPDATE tests: `crates/sim-core/tests/{worldgen,scenario,save_load}.rs`, `crates/simd/tests/{bench,serve}.rs`, `crates/tui/tests/client.rs`, `crates/gui/tests/headless.rs`.
- UPDATE `scenarios/saves/*.save` (3, patched). NEW `scenarios/they-keep-coming.json`.
- NEW `_bmad-output/implementation-artifacts/mutations/13-8.sh`.
- `13-8-signoff/`:
  - at creation: `creation-probes.txt`, `prototypes/{probe-edge-cells,probe-far-idle}.patch`;
  - in dev: `prototypes/strip-new-fields.patch`, `vehicle-card.md`.
- UPDATE `_bmad-output/planning-artifacts/epics.md` Story 13.8 (Q1 a).

### References

- `_bmad-output/planning-artifacts/epics.md`: Story 13.8 (`:3133-3160`), Epic 13 standing ACs 10–18 (`:2950-2962`), FR53/FR56/FR57 (`:141-150`), M4 architecture notes (`:299-330`, AD-7/AD-11 lines)
- `docs/technical-preferences.md` (anti-overengineering; hardcoded constants)
- `crates/sim-core/src/lib.rs:27-48`, `:206-264`, `:567-687`, `:1588-1631`, `:2681-2790`, `:2845-2945`, `:2945-2960`, `:3627-3700`; `src/save.rs:10-60`
- `crates/simd/src/main.rs:76-118`, `:223-251`, `:564`, `:745-790`; `crates/simd/src/scenario.rs:35-60`, `:400-420`; `crates/simd/tests/bench.rs`, `serve.rs:1052`
- `crates/protocol/src/lib.rs:60-100`, `:227`; `crates/simd/src/bridge.rs:180-260`; `crates/client-core/src/lib.rs:32-50`; `crates/tui/src/view.rs:462-490`; `crates/gui/src/ingest.rs:1878-1915`
- `_bmad-output/implementation-artifacts/13-6-two-hundred-on-256.md` (Q2/Q3, #226), `13-7-the-long-way.md` (world budget, churn test)
- GitHub #227 (idle spill crowd, 13.11), #226 (closed: repeated identities load)

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-11-03 | Story created (Opus 5.5). Creation probes: 128² and 256² edge cells are in the camp component on every probed seed; a general walk-home rule would move 152/200 dwarves on two-hundred-on-256 (#227), so the walk-in is arrivals-only; `clear_paths` would strand an arrival. Task 0 Q1–Q5 await Wolf. |
| 2026-11-03 | Task 0 RULED (Wolf, "Yes for all recommendations"): Q1 (a) arrivals are a saved world setting, off in committed scenarios, on in the fresh game; Q2 (a) the embark group spawns at the camp; Q3 (a) an arrival may take a job on the way; Q4 (a) 5–20 every 1,000 ticks, cap 200; Q5 (a) the 16 proposed epithets. Epic Story 13.8 amended (Q1). |
