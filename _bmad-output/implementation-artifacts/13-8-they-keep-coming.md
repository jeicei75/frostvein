---
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
baseline_commit: e2e1882d
---

# Story 13.8: They Keep Coming

Status: in-progress

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

- [x] **Task 0: Wolf's rulings (Q1–Q5).** RULED 2026-11-03: all (a); epic amended. Implementer (asked 2026-11-03): Sonnet 5.5 subagents, Opus orchestrates. Record the answers in the Change Log and amend the ACs they name. Under Q1 (a), amend the epic's Story 13.8 "Given any map size" AC with an `AMENDED 2026-11-03` line. Then ASK which implementer to use (memory: `dev-implementer-is-sonnet-not-codex`).
- [x] **Task 1: unique names and the embark guarantee (AC 1, 5).** UPDATE `crates/sim-core/src/lib.rs`.
  - [x] NEW `pub enum Epithet` (16 variants, Q5) with `pub const ALL: [Self; 16]`, beside `DwarfName` (`:206-245`). `Identity` (`:261-264`) gains `pub epithet: Epithet`.
  - [x] `generate_with_crew` (`:2913-2925`): keep the `names` and `colours` shuffles and `names[i % 16]` / `colours[i % 5]` exactly as they are. Then draw each dwarf's epithet uniformly from the epithets not yet used with its base name, continuing on `identity_rng` after the colour shuffle. Replace the "13.8 makes names unique" NOTE.
  - [x] Professions (`:2930-2943`): crew ≥ 5 keeps today's cycled `[M, M, H, H, W]` and its one shuffle. Crew 3–4 take `[M, H, W, M][..crew]` before the same shuffle.
  - [x] UPDATE `crates/simd/src/main.rs`:
    - `check_crew` (`:97`) refuses crew < 3, naming why ("the first group holds one of each trade").
    - The load validator (`:757-758`) refuses a repeated `(name, epithet)` with `save reuses dwarf name {name:?} {epithet:?}`. Colours stay free. Rewrite the NOTE.
    - Run `audit-mutations.py` BEFORE the first code commit. `13-6.sh` rows 12–13 anchor on `bail!("save reuses dwarf id …")` (memory: `deleting-code-retires-rows-in-the-same-commit`).
- [x] **Task 2: arrivals in the sim (AC 2, 3, 4, 7, 8).** UPDATE `crates/sim-core/src/lib.rs` and `src/save.rs`.
  - [x] Constants beside `WANDER_RADIUS` (`:35`), each a hardcoded tuning value (standing AC 18):
    - `ARRIVAL_PERIOD: u64 = 1_000`;
    - `ARRIVAL_GROUP: RangeInclusive<usize> = 5..=20`;
    - `CREW_CAP: usize = 200`;
    - the arrival profession weights `[(Miner, 2), (Hauler, 2), (Woodcutter, 1)]`;
    - `STREAM_ARRIVAL`, beside the other streams (`:28-34`).
  - [x] The setting:
    - NEW resource `Arrivals(bool)`. `assemble` takes it.
    - `generate_with_crew` gains no parameter. A NEW `World::generate_with_arrivals(seed, dims, crew)`, or a setter, turns it on, and the daemon's fresh game uses it.
    - `SaveState.arrivals: bool` (no `#[serde(default)]`, like `goals_completed`, `save.rs:27-29`).
  - [x] **One spawn helper.** Lift the bundle out of `spawn_dwarves` (`:3683-3699`): `Dwarf, Id, Pos, JobState, Wander, CurrentJob, Carrying, Identity, Profession`. Use the helper for the embark spawn and for arrivals, so `to_save`'s "a story that spawns dwarves a third way must keep the set intact" (`:2954-2957`) holds. Update that NOTE.
  - [x] NEW system `arrive`, in the `.chain()`ed schedule (`:2845-2860`). It is world-advancing, so it skips while paused (AD-2). On `tick % ARRIVAL_PERIOD == 0`, with arrivals on and crew < `CREW_CAP`:
    - Seed this arrival's streams from `seed ^ STREAM_ARRIVAL` mixed with the tick, so no RNG state is saved. Keep the place-and-size draws apart from the identity and profession draws.
    - Pick one edge cell uniformly from the free standable perimeter cells in the camp's component. "Free" means no dwarf, item or emitter (BFS over `astar_neighbours` from the camp, as `spawn_dwarves` does at `:3655-3665`).
    - Draw the group size, capped at `CREW_CAP − crew`. The group takes the first free cells of a BFS from that edge cell, one per tile (FR49).
    - Each dwarf gets an unused full name (uniform over the unused `(name, epithet)` pairs), a colour (repeats allowed), and a weighted profession.
    - Each dwarf also gets an `Arriving { to }` component. `to` is a free cell from a BFS from the camp, skipping occupied cells and cells already given to this group.
    - With no reachable edge cell, skip this arrival, and add a `// NOTE:` saying so.
  - [x] **The walk-in (Found C).** In `wander` (`:2681`), an idle dwarf that holds `Arriving` and no `Path` plans `astar_with_budget(…, &mut terrain.tiles.len())` to the walkable cells within `WANDER_RADIUS` of `to`.
    - Use the world budget, not `astar()`: an edge-to-camp search on 256² can pass 50,000 nodes (13.7).
    - The existing Path branch (`:2729-2753`) then walks it. Where it sets `home` at the path's end, also remove `Arriving`.
    - If no path is found, remove `Arriving` and set `home = here`.
    - `release_claim` (`:1588-1610`, the funnel for every job end) removes `Arriving`: a job took the dwarf, so it has joined the work.
    - No other dwarf's wander changes (Found B).
  - [x] Save: `SavedDwarf.arriving: Option<Pos>` with `#[serde(deserialize_with = "Option::deserialize")]`, so a missing key is refused (memory: `serde-option-is-never-required`). `to_save` and `from_save` carry it.
- [x] **Task 3: the wire and both clients (AC 6).**
  - [x] UPDATE `crates/protocol/src/lib.rs`: NEW `Epithet` (snake_case), and `Identity.epithet`. Fix the raw-wire tests (`:419`, `:443`).
  - [x] UPDATE `crates/simd/src/bridge.rs`: `fn epithet(…)` as an exhaustive `match` with no wildcard (`:241` is the shape), used at `:188-190`.
  - [x] UPDATE `crates/client-core/src/lib.rs`: `epithet_text` beside `dwarf_name_text` (`:32`), plus a one-spelling test like `every_dwarf_name_has_its_one_spelling` (`:487`).
  - [x] UPDATE `crates/tui/src/view.rs:470-475`: the roster shows `Name Epithet`. UPDATE `crates/gui/src/ingest.rs:1888-1912`: the selected slot shows `Name Epithet`. The all-dwarves list pads to the longest full name, replacing the NOTE's "padded to the longest (6)". No other gui change. Add a `name_readout_spans` unit test: a mirror whose dwarf first appears in a delta, after the snapshot, and is then selected, reads `Name Epithet`.
  - [x] Fix the struct literals the new field breaks. They are compile-driven: `tui/src/view.rs`, `tui/tests/client.rs:1696`, `gui/tests/headless.rs:4951`, `sim-core/tests/{save_load,scenario}.rs`, and `simd/tests/serve.rs`.
- [x] **Task 4: scenarios, saves, fresh game (AC 8, 9, 11).**
  - [x] UPDATE `crates/simd/src/scenario.rs`:
    - `StartFile` (`:35-46`) gains `arrivals: Option<bool>`. A seed start passes it, with absent meaning off.
    - Refuse `arrivals` with a `save` start, the way `every_unusable_scenario_fails_loudly_naming_the_cause` (`bench.rs:308`) already expects for dims/crew.
  - [x] UPDATE `crates/simd/src/main.rs:251`: the fresh game is generated with arrivals on.
  - [x] Patch `scenarios/saves/{12-11-dig-loop-patched,12-12-seat-loop,12-12-seat-slow}.save` with a throwaway script:
    - each dwarf's `identity.epithet` = the epithet that HEAD's `World::generate(DEFAULT_SEED, Dims::DEFAULT)` gives the same-named dwarf;
    - `"arriving": null` on each dwarf;
    - `"arrivals": false` at the top.

    `main`'s `simd` ignores unknown keys (only `scenario.rs` sets `deny_unknown_fields`), so `tick-bench.sh main HEAD` still reads all three. Check that, and record it in the Debug Log.
  - [x] NEW `scenarios/they-keep-coming.json`: `{"seed":4026891802,"dims":[256,256,32],"crew":5,"arrivals":true}`, no commands, `speed` normal. Set `ticks` to the last arrival's tick + 4,000, read from the run, so every walk can end. Its `about` names 13.8 and the expected end. `main` cannot read it (`deny_unknown_fields`), so its row is HEAD-only, like 13.6's. Record one run's wall time. If it is over 120 s, say so: every later story's tick-bench pays it.
- [x] **Task 5: the sim tests (AC 1–5, 7–9).** NEW tests in `crates/sim-core/tests/` (`worldgen.rs`, `scenario.rs` and `save_load.rs` hold the precedents) or in `lib.rs mod tests` where they need private state. Fast tier only: **no debug test steps a 256² world with a growing crew** (13.6's debug cost rule). Use 128² DEFAULT_SEED.
  - [x] AC 1: crews 3, 4, 5, 17 and 200 each hold all three trades. Crew 5's professions and identities equal today's: pin the five `(name, colour, profession)` triples read from `main` first.
  - [x] AC 2:
    - From crew 5, arrivals on: crew at t999 is 5. At t1,000 it is 5 + n with n in 5..=20, and every new dwarf stands on a distinct cell. The first stands on an edge cell, and every one is in the camp component (reuse `camp_component`, `tests/worldgen.rs:952`).
    - From crew 190 (embark, arrivals on): crew 200 at t1,000, still 200 at t3,000.
  - [x] AC 3, three tests:
    - (i) From the first arrival, step until every arrival has ended its walk. Assert each stands within `WANDER_RADIUS` of its `to`, within 4,000 ticks.
    - (ii) The same, with a `set_tile` on a far cell every 8 ticks plus `clear_paths` (the 13.7 churn shape, `lib.rs` test `a_far_claim_is_kept_through_terrain_churn_past_the_node_cap`). **RED first:** without the re-plan, an arrival freezes.
    - (iii) Every embark miner is set to hauler (`set_profession`), and a dig is designated near the camp. An arriving miner claims it, and it is dug.
  - [x] AC 4: same-seed worlds equal after 3,000 ticks with arrivals on. Embark spawn positions and the wander stream are unchanged by turning arrivals on: the two worlds' states up to t999 are equal.
  - [x] AC 5: `generate_with_crew(DEFAULT_SEED, DEFAULT, 200)` and the crew-190 arrivals world at t1,000 each have 200 distinct `(name, epithet)` and fewer than 200 distinct colours. In `crates/simd/tests/serve.rs`, keep `the_live_load_takes_a_save_whose_crew_repeats_names_and_colours` (`:1052`), whose base names still repeat, and assert its full names are unique. Add a refusal test: a save with two dwarves sharing a full name, refused through the live Load, with the cause in the log. Mirror both in `bench.rs:451`'s start.save path.
  - [x] AC 7: save at the first tick when some dwarf holds `Arriving` (about t1,005). Load, step 300, and compare `to_save` bytes with the never-saved world.
  - [x] AC 8: arrivals off vs on, same seed, at t3,000: crew 5 vs > 5. Save → load keeps the setting. Plus the scenario-start refusals in `bench.rs`.
  - [x] AC 9: for each new key, strip it from serialised JSON and assert the load is refused. The control is the unstripped load.
- [x] **Task 6: the instrument, RED first (AC 11).** The instrument is `simd --bench`'s end line (`crates/simd/src/scenario.rs:410-418`), which gains `crew=N arriving=K`. `N` is `world.dwarves().len()` and `K` is the dwarves holding `Arriving`, which needs a NEW `World::arriving_count()`. A human reads the crew grow with `target/release/simd --bench scenarios/they-keep-coming.json --runs 1`.
  - [x] Tests in `crates/simd/tests/bench.rs`:
    - the one-dig fixture (`:19`) reads `crew=5 arriving=0`;
    - a `variant` with `"arrivals": true` and `ticks` 1,010 reads `crew` > 5 and `arriving` > 0;
    - the same variant without the key reads `crew=5`.
  - [x] RED: on a scratch worktree, make `arrive` never fire (memory: `mutation-run-poisons-concurrent-builds`). The variant test fails at its `crew > 5` assert; paste the line. Then GREEN on HEAD. Paste `they-keep-coming`'s end line: `crew=200 arriving=0` is required. Any `arriving` > 0 is a stalled walk, not a pass.
- [x] **Task 7: bench delta (AC 10).**
  - [x] On an idle devpod: `RUNS=5 scripts/tick-bench.sh main HEAD` over the 11 committed files. Paste the table.
  - [x] Throwaway `13-8-signoff/prototypes/strip-new-fields.patch`: `#[serde(skip_serializing)]` on the three fields. `skip_serializing` changes only the hash input, not the load. Build it on a scratch worktree and run `simd --bench` on all 11 files with `--runs 1`. Each `state=` must equal `main`'s. A row that differs is a real behaviour change: trace it and record why (standing AC 11).
  - [x] Record two-hundred-on-256's mean and p99 delta against its floor, and its end line on both sides. They must match apart from `crew=`/`arriving=`, because arrivals are inert at the cap.
- [x] **Task 8: mutation table (AC 12).** NEW `_bmad-output/implementation-artifacts/mutations/13-8.sh`, in `13-7.sh`'s shape, with every anchor readable by `audit-mutations.py`. Commit first, then run it alone (`env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …`). Read each panic line (memory: `mutation-test-name-prefix-trap`). Rows:
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

Wolf picked Sonnet 5.5 subagents for the implementation (2026-11-03). Opus 5.5 orchestrates and verifies, and keeps the REDs, the mutation run, the bench and the full gate.

### Debug Log References

- **Task 1 (Sonnet subagent, `baa9ded9`, fast gate green 757 s):** REDs: `every_crew_of_three_or_more_holds_all_three_trades` → `crew 3 holds no Woodcutter`; `crew_200_holds_two_hundred_distinct_full_names` → E0609 (no `epithet`); bench `a_save_that_repeats_a_full_name_is_refused_naming_the_cause` with the validator reverted → `exited 0`; bench `crew-two` case with `check_crew` reverted → `crew-two: exited 0`. serve `the_live_load_refuses_a_save_that_repeats_a_full_name` was not run RED (mutation row 6 covers it). Crew-5 pin read from the old code: Nain Purple Woodcutter, Ori Green Hauler, Bifur Red Miner, Frar Gold Hauler, Dori Blue Miner (passes after). `audit-mutations.py` first went RED on `12-4.sh` "the pool has no hauler" (it anchors on the `[M, M, H, H, W]` literal), so the crew 3–4 case is an override after the untouched literal; audit then green (979 rows). The three saves were patched as text: Nain Flintback, Ori Hammerfall, Bifur Frostbeard, Frar Ashbrow, Dori Rimecloak.
- **Task 2 + Task 5 sim tests (Sonnet subagent, `0a586738`, fast gate green 807 s; `cargo test -p sim-core` 108 lib / 24 save_load / 87 scenario / 31 worldgen pass).** `arrive` is an exclusive system at the END of the chain (after `advance_goals`), firing on `tick % ARRIVAL_PERIOD == 0` after `advance_tick`. Its RNGs are `ChaCha8Rng::seed_from_u64(seed ^ STREAM_ARRIVAL ^ tick)` with `set_stream(0)` (edge cell, group size) and `set_stream(1)` (name pair, colour, profession), so no RNG state is saved. An arrival's initial `home` is its entry cell. REDs, with `arrive` stubbed to return early: AC 2 `the_first_group_arrives_at_the_period_on_distinct_camp_cells_from_the_edge` → `the group holds 0 dwarves`; `the_crew_stops_at_two_hundred` → `left: 190 right: 200`; AC 5 `two_hundred_by_arrivals_hold_two_hundred_distinct_full_names` → `left: 190 right: 200`; AC 4 `same_seed_arrival_worlds_are_identical_after_three_periods` → `the worlds must have grown to compare anything`; `arrivals_on_and_off_agree_until_the_first_period` → `on.dwarves().len() > 5` (its equality half passes under the stub); AC 7 `a_save_made_while_a_group_walks_in_reloads_identically` (saves 100 ticks after the first arrival, mid-walk) → `nobody ever arrived`; AC 8 `the_arrivals_setting_decides_the_crew_and_survives_a_save` → `on.dwarves().len() > 5`; AC 3 (i) `every_arrival_ends_its_walk_near_its_destination` → `the group is 0 dwarves`; AC 3 (iii) `an_arriving_miner_claims_a_dig_and_digs_it` → `the group has arrived`. AC 3 (ii) RED with the planner firing only on an edge cell (plan-once-at-spawn): `every_arrival_ends_its_walk_through_terrain_churn` → `an arrival is still walking`; (i) passes under that sabotage, so only (ii) discriminates the re-plan. No RED: `an_arrival_with_no_way_to_the_camp_ends_its_walk_where_it_stands` (passed first run; positive asserts pos == home == spawn cell when `Arriving` ends) and AC 9 `a_save_missing_an_arrivals_key_is_refused` (serde attributes; positive control). Mutation rows 5/7 cover the arrivals side.
- **Orchestrator ruling (Opus, Task 2):** the story says remove `Arriving` where the idle-Path branch sets `home` at the path's end. That branch also walks `resolve_blocked_step`/`step_aside` exit paths, which end anywhere, so `Arriving` is removed there only when the dwarf stands within `WANDER_RADIUS` (and on the same z) of `to` (`near_arrival_target`). A stepped-aside arrival keeps walking in.
- **Mutation anchors re-pointed by Task 2 (static audit green, 979 rows; NOT yet re-run, Task 8 re-runs them):** `12-11.sh` row 19b (`deserialize_with` now appears twice in save.rs; anchor includes `pub clearing:`); `2-4-the-world-endures.sh` 3 rows (`arrivals,` added to old/new); `5-1-the-world-grows-things-that-glow.sh` "wander homes return to random spawn cells" (now anchors on the embark `spawn_dwarf(&mut self.ecs, pos, camp, identity, profession);`). `13-7.sh` rows 1–2 kept their anchor by naming the walk-in budget `walk_budget`.
- **Task 3 (Sonnet subagent, `8993d7e6`; first hook run RED on two gui lib tests still expecting the bare name, fixed, second run green).** REDs (with `epithet_text` returning `""`, or the slot without the epithet): client-core `every_epithet_has_its_one_spelling` → `left: "" right: "Ironhand"`; tui `the_roster_shows_a_trade_after_the_name_and_a_dwarf_without_one_shows_his_name_only` (extended) → row `"Ori   Nori  hauler …"` fails `starts_with("Ori Ironhand  Nori Ironhand hauler  ")`; gui `a_dwarf_first_seen_in_a_delta_reads_name_and_epithet_when_selected` (snapshot with no dwarves, delta introduces him, then selected) → `left: "Thrain \n" right: "Thrain Copperbeard\n"`. The all-dwarves list pads to 18 ("Thrain Copperbeard") + 1, `{:<19}`. Mutation anchor re-pointed: `12-2.sh` "hud never shows the name" (now the `format!` block → `String::new()`), not yet re-run.
- **AC 6 recipe (release, port 7461, `--dims 128,128,32`):** at the tui's default width (100 columns when stdout is not a TTY) it prints only 3 lines (`3 Bifur Frostbeard miner / 3 Nain Flintback woodcutter / 3 Ori Hammerfall hauler`): the roster is one row cut at the frame width, and five full names plus trades need about 125 columns. With `COLUMNS=200 LINES=50` before the tui command: `3 Bifur Frostbeard miner / 3 Dori Rimecloak miner / 3 Frar Ashbrow hauler / 3 Nain Flintback woodcutter / 3 Ori Hammerfall hauler` (5 distinct, as required). RED (`epithet_text` → `""`, `COLUMNS=200`): 0 lines. **Finding, not fixed (scope: no roster UI):** a 100-column terminal now truncates the roster at crew 5. With arrivals it overflows at any crew anyway, epithets or not, so it goes to the seat as part of AC 13's crew-list question. The Verification recipe needs `COLUMNS=200 LINES=50`.
- **Tasks 4 + 6 (Sonnet subagent, `8bf7e6ae`).** `arrivals` beside `save` is refused with "start arrivals is for a seed start: a save has its own". The fresh game goes through `fresh_world` (arrivals on), and `generate_world` keeps its `catch_unwind` crew refusal. bench 11 pass + 1 release-only ignored; `simd --bin` 38 pass; full `serve.rs` 83 pass (169 s) with fresh-game arrivals on, and no test needed them off. **Task 6 RED** (scratch worktree, `arrive` returns at once): `panicked at crates/simd/tests/bench.rs:522:5: arrivals on must grow the crew: bench one-dig end tick=1010 marks=0 items=0 zones=0 goals=0/0/0 done=dig:0,cut:0 stored=0 cells=0 crew=5 arriving=0`. `a_fresh_game_grows_its_crew` (main.rs unit test) was not run RED. Mutation anchor re-pointed: `13-6.sh` row 5 (`generate_world(*seed, *dims, *crew, *arrivals)`), not yet re-run.
- **FINDING, they-keep-coming stalls (2026-11-03).** 256², seed DEFAULT_SEED, crew 5, arrivals on, release; end lines by `ticks`: 8,000 → crew=112 arriving=54; 14,000 → 177/85; 16,000 → 198/80; 17,000 → 200/82; 20,000 → 200/68; 24,000 → 200/68; 30,000 → 200/68. The crew reaches 200 between t16,000 and t17,000, and 68 walks never end. Probe (scratch worktree, release, t20,000): all 68 hold a `Path` whose next tile is occupied, 51 behind another walking arrival and 17 behind a settled dwarf (idle, no path, no job). The wait-for graph has 68 chains, every one ending at a settled dwarf, and 0 cycles. The 17 blockers are #227's crowd jammed around the camp (127,128,7): their neighbours are full, so they never step aside, and the idle-Path follower waits forever (the Dev Notes' trap). Prototypes, all measured at t20,000: (1) **stop behind a settled dwarf**, stateless: arriving 0 at 256², but `every_arrival_ends_its_walk_near_its_destination` fails on 128². At first a group member blocked its own group (a just-spawned arrival is pathless), and after excluding `Arriving` dwarves an arrival ends 5 cells short behind a camp wanderer: `ended at (58,67,10), not within WANDER_RADIUS of (63,64,9)`. (2) **re-plan around settled dwarves** (plan with their cells blocked; drop the path when one steps in): arriving **129**, all in head-on cycles, worse. (3) **wait limit**, an arrival that waits 100 ticks on an occupied tile ends its walk where it stands: arriving 0 at 256², and all of `cargo test -p sim-core` passes unchanged. **Wolf ruled (3), "Wait limit", 2026-11-03.** `SavedDwarf.arriving` becomes `Option<{to, waited}>`, still a required key.
- **Wait limit (Sonnet subagent, `e03cde78`).** `ARRIVAL_PATIENCE_TICKS = 100`, and `Arriving { to, waited }`. A blocked follower sets no cooldown, so it re-checks every tick and one wait is one tick. At the limit the walk ends where it stands (`home = here`, `Path` and `Arriving` removed), and a step resets the count. The save holds `SavedDwarf.arriving: Option<SavedArriving { to, waited }>`, still `deserialize_with = "Option::deserialize"`. The committed saves' `"arriving":null` still decodes. REDs: `an_arrival_gives_up_after_waiting_the_patience_limit_behind_a_blocker` (blocker with `Wander.cooldown = u32::MAX`; still `Arriving` at 99 waits and gone at 100 with `pos == home == start`), with the give-up made unreachable, fails `the walk ended / left: Some(100) / right: None`; `a_save_round_trip_keeps_the_arrival_wait_count`, with `from_save` using 0, fails `left: Some(0) / right: Some(40)`. No existing test changed. `cargo test -p sim-core` 110 lib / 24 / 87 / 31 pass; bench 11 + 1 ignored.
- **they-keep-coming (`23ea328c`).** The last arrival is t17,000 (16,000 → crew=198 arriving=22; 17,000 → 200/23). `arriving` reads 15 at 18,000 and 0 at every 1,000 from 19,000 to 30,000. `ticks` = 17,000 + 4,000 = 21,000. **Re-run by Opus on HEAD, release:** `bench they-keep-coming end tick=21000 marks=0 items=0 zones=0 goals=0/0/0 done=dig:0,cut:0 stored=0 cells=0 crew=200 arriving=0` (run line `mean=0.0874 p99=0.1842 max=38.358 over10=25 state=75fd7c20e91eeb0f`, the same state hash as the subagent's run). Wall time 3.04 s, well under 120 s.
- **Task 8, mutation table (Opus, on `94968b93`, run alone: `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …/13-8.sh`): 12/12 KILLED, each at its intended assert, read from the panic line** (a row that deletes or adds lines in lib.rs shifts the reported line number of a `mod tests` assert; row 5 was confirmed by re-running it in a scratch worktree). Row: test, then the assert it died at.
  1. `arrive` never fires: `the_first_group_arrives_…` at `the group holds {} dwarves`.
  2. group size uncapped: `the_crew_stops_at_two_hundred` at t1,000's `== CREW_CAP`.
  3. embark epithet always `ALL[0]`: `crew_200_holds_two_hundred_distinct_full_names` at `full.len() == 200`.
  4. re-plan gone (plan only on an edge cell): `every_arrival_ends_its_walk_through_terrain_churn` at `an arrival is still walking`.
  5. `release_claim` keeps `Arriving`: `an_arriving_miner_claims_a_dig_and_digs_it` at `Id(14) took a job and kept Arriving`.
  6. validator full-name check gone: bench `a_save_that_repeats_a_full_name_is_refused_naming_the_cause` at `exited 0`.
  7. `Arrivals` ignored: `the_arrivals_setting_decides_the_crew_and_survives_a_save` at `off.dwarves().len() == 5`.
  8. crew 3 takes `[M, M, H]`: `every_crew_of_three_or_more_holds_all_three_trades` at `crew 3 holds no …`.
  9. edge filter gone: `the_first_group_arrives_…` at the edge-cell assert.
  10. scenario key ignored: bench `a_seed_start_with_arrivals_grows_the_crew_and_without_it_does_not` at `arrivals on must grow the crew`.
  11. patience limit gone: `an_arrival_gives_up_after_waiting_the_patience_limit_behind_a_blocker` at `the walk ended`.
  12. load forgets `waited`: `a_save_round_trip_keeps_the_arrival_wait_count` at `Some(40)`.

  Exclusivity is not claimed (focused runs only). **The 7 older rows this story re-pointed were RE-RUN, not only audited, from a scratch table holding only them: 7/7 KILLED.** They are `12-11` 19b (`loading_refuses_a_pre_12_11_save_without_dwarf_clearing`), `5-1` "dwarf wander homes return to random spawn cells", `12-2` "hud never shows the name", `13-6` row 5, and `2-4` "loaded world schedule omits wander", "from_save regenerates terrain from the seed" and "from_save marks every tile dirty". `audit-mutations.py`: 991 rows, every literal matches.
- **Task 7, bench delta (Opus, idle devpod, `RUNS=5 scripts/tick-bench.sh main HEAD` on the 11 committed files; base `e2e1882d`, head `94968b93`; load before 13.25/11.74/6.34, after 1.31/1.77/2.64):**

  | scenario | base_ms | head_ms | delta | control | base_floor | head_floor | ctl_floor | state | verdict |
  |---|---|---|---|---|---|---|---|---|---|
  | big-pile | 0.1974 | 0.1988 | +0.7% | -0.6% | 2.0% | 1.1% | 2.6% | differ | within floor |
  | deep-pit | 0.0375 | 0.0377 | +0.6% | -0.2% | 4.3% | 8.5% | 7.2% | differ | within floor |
  | dig-loop | 0.0277 | 0.0274 | -0.9% | -0.2% | 5.8% | 3.6% | 3.3% | differ | within floor |
  | full-pile-crowd | 0.0316 | 0.0318 | +0.6% | +0.3% | 8.2% | 6.6% | 2.2% | differ | within floor |
  | plan-compact-piles | 0.1503 | 0.1578 | +4.9% | +3.0% | 3.0% | 2.2% | 2.5% | differ | BEYOND FLOOR |
  | plan-far-piles | 0.0879 | 0.0917 | +4.3% | +2.0% | 7.1% | 7.9% | 1.3% | differ | within floor |
  | plan-spread-piles | 0.1687 | 0.1753 | +3.9% | +3.0% | 2.1% | 2.0% | 0.8% | differ | BEYOND FLOOR |
  | plan | 0.0826 | 0.0847 | +2.5% | +2.1% | 1.9% | 1.9% | 2.0% | differ | BEYOND FLOOR |
  | seat-slow | 0.1618 | 0.1691 | +4.5% | +2.5% | 2.8% | 3.7% | 1.1% | differ | BEYOND FLOOR |
  | stuck-tunnel | 0.0268 | 0.0275 | +2.6% | +2.1% | 1.9% | 3.3% | 3.7% | differ | within floor |
  | two-hundred-on-256 | 34.1261 | 34.0631 | -0.2% | -2.0% | 0.9% | 0.5% | 0.9% | differ | within floor |

  All 11 read `differ`, as AC 10 expects (the new saved fields). Four rows are BEYOND FLOOR at +2.5% to +4.9%, each with its own layout control at +2.1% to +3.0%. Every one is under |control| × 2, which the script calls weak evidence between builds (memory `cross-build-layout-noise`). With arrivals off, these worlds pay only the `arrive` early return and the extra `Option<&mut Arriving>` fetch in `wander`. No profile was taken (no premature optimisation). **two-hundred-on-256: mean -0.2%, against floors 0.9% (base) / 0.5% (head) and control -2.0%.**
- **Reach, through the strip build (`13-8-signoff/prototypes/strip-new-fields.patch`: `#[serde(skip_serializing)]` on `Identity.epithet`, `SavedDwarf.arriving` and `SaveState.arrivals`; release builds of the strip and of `main` in scratch worktrees, `simd --bench F --runs 1`):** `state=` is **identical on all 11 files**: big-pile 81074e5c52948a13, deep-pit 8cfa0773f7a78f43, dig-loop 0337abf343bac62f, full-pile-crowd eb1112576885e4ad, plan-compact-piles ff7ac70d920a827e, plan-far-piles f07b194f92a5ec93, plan-spread-piles de631ebd8382e8e4, plan b124519bce5e6bab, seat-slow c39ed45c4ecc5ff7, stuck-tunnel a5d257cac0182a1f, two-hundred-on-256 6ba8edcc3d6c7b9c. No row is a behaviour change. `main`'s `simd` read the three patched saves (dig-loop, seat-slow and stuck-tunnel start from them), which confirms it ignores the new keys (Task 4). two-hundred-on-256's end line is `tick=2500 marks=1476 items=600 zones=1800 goals=22/0/0 done=dig:234,cut:90 stored=27 cells=2146` on `main`, on the strip build and on HEAD; HEAD and the strip build add `crew=200 arriving=0`, so arrivals are inert at the cap.
- **Full gate, `RUST_TEST_THREADS=1 scripts/gate.sh` on `71a7390c` (2026-11-03):**
  ```
  frostvein gate
    cargo fmt --check           ok  0s
    cargo clippy -D warnings    ok  2s
    cargo test                  ok  1223s
    cargo test (release real-work scenario)ok  98s
    cargo test (pixel guards)   ok  3092s
    tui has no sim-core edge                ok
    client-core has no sim-core edge        ok
    gui has no sim-core edge                ok
    metrics ledger tests        ok  0s
    bench tests                 ok  19s
    mutation tables still apply ok  11s
  reaped 18 stale build artifacts from target (0.3 GB)
  GATE GREEN  4452s
  ```
- **The seat, 2026-11-04 (Wolf, fresh game, release `simd`, `71a7390c`; record in `13-8-signoff/vehicle-card.md` § Sitting):** Q1 "yes"; Q2 "yes"; Q3 "yes the current list is too big to fit ofc so we need to have story for hud and ui"; Q4 "I think it's ok now.. ofc new groups are coming too fast atm but it helps testing... and damn it looks cool when new dwarves arrive especially during night...", so the constants stay. Unprompted: "still performance issue .. all was ok as long as I cut down a few trees and digged a little bit ..when in the end I designated bigger dig are again stuttering started", then "it's not just digging issue but eventually after a while it starts to slow down and stutter and slow down (also fps) and it happens even without designations". The slowdown grows with run time and hits the gui fps too; the crew grows 5 → 200 over the same span, so time and crew size are confounded. Candidates #225 / #224; not measured, cause not established. Then: "zooming to 200 dwarf nest will take fps to knees 6pfs or something" (close framing on the packed camp, unmeasured); "performance issue we need to solve". Q3 filed as #230 (placeholder for the HUD/UI story).
- **Probe after the seat (2026-11-04): does the sim slow down over time at constant crew?** `they-keep-coming` run to 40,000 ticks in release, `simd --bench` patched in a scratch worktree to print 500-tick windows (`13-8-signoff/prototypes/per-window-probe.patch`, output `per-window-probe-40k.txt`). `World::step` holds a **0.06 ms** mean at crew 200, flat from tick 17,000 to 40,000; whole run mean 0.076, p99 0.14, `over10=22`. Each of those 22 is a group spawn at about 36 ms, under the 100 ms period. The delta encode is 0.09 ms, but **32.9 KB every tick for an idle crew at the camp**: about 165 B per dwarf per tick, flat once the crew stops growing. **What it shows is narrow: a crew with no work history does not slow the sim over time at 200.** It has `marks=0 items=0` for the whole run and no served path, so it does NOT cover Wolf's run, which cut trees and dug before the designations stopped (hauling, piles and loose items continue after). Wolf: "slowdown is probably 2 different thigs so i would not count out sim slow down either..". Sim and gui both remain candidates. End line `crew=200 arriving=0`. Comments for #225/#224 were drafted; posting was blocked by the harness permission classifier.

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-11-03 | Story created (Opus 5.5). Creation probes: 128² and 256² edge cells are in the camp component on every probed seed; a general walk-home rule would move 152/200 dwarves on two-hundred-on-256 (#227), so the walk-in is arrivals-only; `clear_paths` would strand an arrival. Task 0 Q1–Q5 await Wolf. |
| 2026-11-03 | Task 0 RULED (Wolf, "Yes for all recommendations"): Q1 (a) arrivals are a saved world setting, off in committed scenarios, on in the fresh game; Q2 (a) the embark group spawns at the camp; Q3 (a) an arrival may take a job on the way; Q4 (a) 5–20 every 1,000 ticks, cap 200; Q5 (a) the 16 proposed epithets. Epic Story 13.8 amended (Q1). |
| 2026-11-04 | Seat (AC 13): Q1 yes, Q2 yes, Q3 yes (a crew list is needed: HUD/UI story), Q4 constants stay. Wolf reports a slowdown and stutter (tick and fps) that grows over the run, with or without designations; unmeasured. |
