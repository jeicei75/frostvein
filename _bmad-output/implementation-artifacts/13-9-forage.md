---
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
baseline_commit: 3c6cf3b7
---

# Story 13.9: Forage

Status: in-progress

## Story

As the boss,
I want woodcutters to gather food from plants in an area I mark, again and again as the plants regrow,
so that the fortress has its first food.

## Branch

`story-13-9-forage`, cut from `main` `3c6cf3b7` (13.8b's merge). Its first commit `b99f881e` is 13.8b's close-out and rides into this PR, as `51da2d4e` did for 13.8.

**Standing ACs that bind:** Epic 13's M3 ACs 1–8 (**one full gate per story, at PR**), plus M4 ACs 10, 11, 12, 14, 16, 17 and 18. AC 13 (needs) and AC 15 (new trade) do not apply: there is no eating yet, and forage goes to the existing woodcutter.

**Wire diff (AD-6, enums snake_case):**
- `ItemKind` gains `Berries`.
- NEW `PlantState { Ripe, Growing }`, NEW `Plant { pos, state }`, NEW `PlantChange { pos, state: Option<PlantState> }` (`None` = the plant is gone), NEW `ForageCell { pos }`.
- `Snapshot` gains `plants: Vec<Plant>` and `forage: Vec<ForageCell>`.
- `Delta` gains `plants: Vec<PlantChange>`: only the plants that changed since the last delta, the way `tiles` works. It also gains `forage: Vec<ForageCell>`, resent in full like `zones`.
- `Command` gains `PlaceForageArea { rects: Vec<Rect> }` and `RemoveForageArea { rect }`. `Refusal` gains `PlaceForageArea { rect }`.

**Saved fields (two), plus one new job kind and one new item kind inside existing fields:**
- `SaveState.plants: Vec<(Pos, u64)>`, each plant's `ripe_at` tick.
- `SaveState.forage: Vec<Pos>`, the forage-area cells.
- `JobKind::Forage` goes in `jobs`, and `ItemKind::Berries` goes in `items`.

Every state hash therefore moves (memory: `state-hash-covers-save-encoding`). AC 2 reads the reach with those two fields stripped from the encoding.

## Task 0 — Wolf rules before dev

Each question has a recommendation, and the ACs and Tasks are written for it. A different answer amends the ACs it names.

- **Q1. What is a plant?**
  - (a) **Recommended:** a plant is sim state of its own, `Plants(BTreeMap<Pos, Plant { ripe_at: u64 }>)`. It is not a tile and not an entity with an `Id`. It stands on a standable cell and blocks nothing: dwarves walk through it, and it never touches `set_tile`, `clear_paths` or the id counter. Generated on its own seed stream after the trees, it shifts no tile, no spawn and no id, so a world with no forage area plays exactly as it does on `main` (AC 2). Ripe means `tick >= ripe_at`.
  - (b) A new `Material` tile at surface + 1, like a tree. It blocks standing and paths, ripples into every `Material` match in the wire and both clients, moves the terrain fingerprint and every scenario outcome, and makes regrowth a `set_tile` (path and pile caches dropped on every regrow at 256²).
- **Q2. Names and constants (standing AC 18).**
  - (a) **Recommended:**
    - The plant is a berry bush. The item is `ItemKind::Berries` (wire `berries`), the first raw food; later stories add carcass, fish and meal.
    - Placement: one in 16 eligible columns, uniform, on its own stream `STREAM_PLANTS`.
    - Eligible means: the column's top tile is `Solid(Snow)`, the cell above it is empty, and the column is outside the lake, outside the camp box (`CAMP_RADIUS + 1`, as the trees use), off the edge ring, and not a tree column. At least one side neighbour of the bush's cell must be standable at generation, so that no bush is born unforageable.
    - `FORAGE_WORK_TICKS = 30`; `REGROW_TICKS = 3_000` (5 min at 1x, 75 s at fast4x); one `Berries` per forage.
  - (b) Other numbers or names. Patches (clustered bushes) instead of uniform placement are a later look call at the seat, not this story.
- **Q3. What is a forage area?**
  - (a) **Recommended:** a saved cell set of its own, `ForageArea(BTreeSet<Pos>)`, shaped like the stockpile.
    - `PlaceForageArea { rects }` takes several rects, because plants sit on hills as the forest does. It keeps the standable cells of its rects.
    - It is refused, naming the bounding rect, when no kept cell holds a plant.
    - `RemoveForageArea { rect }` subtracts cells, and the gui's Clear sends it too.
    - An area may overlap a stockpile.
    - Forage jobs come from the area's ripe plants, not from `Designations`, so they do not count against `MAX_DESIGNATIONS`; the plant count bounds them.
  - (b) Per-plant marks, `DesignationKind::Forage`, like cut. Completing a job deletes its mark, so "again and again while the area stands" would need a second store anyway.
- **Q4. The bench scenario (standing AC 14, FR67).**
  - (a) **Recommended:** `scenarios/two-hundred-on-256.json` gains one `place_forage_area` command at `at: 1`, near the pile, covering at least 30 plants. It is "the 256² bench scenario" that 13.10 inherits "with its forage area".
    - `at: 1` puts the forage jobs ahead of the 200 cut jobs in the woodcutters' FIFO (Found B).
    - `main`'s `simd` cannot read the new command (`deny_unknown_fields`), so the delta against `main` (standing AC 10) runs on `main`'s copy of the file. The forage file is recorded as a HEAD-only row.
  - (b) A new `scenarios/forage-on-256.json`, with `two-hundred-on-256.json` left unchanged. Every later story's bench then runs two 256² files (about 20 min more per tick-bench).
- **Q5. The forage-area mode: input and the draft (standing AC 16; this is the draft for Wolf to approve).**
  - (a) **Recommended:** key **6**, which is free. Digit6–Digit0 are unbound, and #118 is about the F-keys, so it is untouched. The mode works the way stockpile does:
    ```
    HUD (None):  1 dig  2 channel  3 stockpile  4 clear  5 cut  6 forage   Space pause ...
    HUD (mode):  forage: drag over the bushes  Esc leave   |   forage: release to mark  Esc abort
    drag preview: the stockpile slab shape in a berry colour on the kept cells; bushes inside the box lit
    placed area:  the same slab, berry colour, under the bushes (a pile under it still shows as a pile)
    refused:      "forage area refused: no plant in it"  (the existing refusal line)
    bush, ripe:   a low dark-green block with red berry dots   bush, growing: the green block only, smaller
    berries item: a small red cube (the item-cube slot wood uses)
    tui:          ripe bush "  red   growing bush "  dim green   berries %  red   area cell :  berry colour
    ```
  - (b) Wolf edits the draft, or routes the mode through the selection UI.
- **Q6. The three committed seat saves.**
  - (a) **Recommended:** patch each with `"plants": []` and `"forage": []`, as data, with a throwaway script as 13.8 did. Their worlds then stay plant-free and the #203, #204 and #206 reproducers keep their outcome.
  - (b) Regenerate their plants from the seed. Their outcome is unchanged either way (Q1 a), but each save grows by about 900 entries.

## Amendment 1 — seat run 1 (2026-11-22): haulers forage on trips, one-time marks

Wolf sat build `a192ab88` (words on `13-9-signoff/vehicle-card.md`) and ruled a redesign. This section **supersedes Task 0 Q1's "blocks nothing" note only where it says so, Q3 entirely, and Q2's trade**. The ACs below are rewritten to match; the old Tasks 1–10 stand as history.

- **Q7. Who forages:** haulers, several bushes a trip (Wolf: "haulers could probably do foraging also. No need to forage and haul separately but collect some amount and carry then to pile"). Woodcutters no longer forage.
- **Q8. Marks:** one-time marks exactly like cut (Wolf: "should work like cutting"). `DesignationKind::Forage`; a mark is used up when its bush is picked; a regrown bush must be marked again. The forage AREA goes, end to end: `ForageArea`, `PlaceForageArea`/`RemoveForageArea`, `Refusal::PlaceForageArea`, `ForageCell`, `Snapshot/Delta.forage`, `SaveState.forage`, the gui slab, the tui `:`.
- **Q9. First bush of a trip:** the nearest marked bush the hauler can reach. Only forage is special-cased in the claim step; every other kind keeps AD-12's oldest-first.
- **Q10. Trip size:** `FORAGE_TRIP_BUSHES = 5`, `FORAGE_TRIP_RADIUS = 10` (cells, Manhattan in 3D, from where the hauler stands).
- **Q11. The load:** one `Berries` item with a count, as Dwarf Fortress's herbalist gathers "a stack of 1-5 plants" and carries it to the food stockpile itself (DF wiki, Herbalist, v53.17). Each item now carries a `count` (1 for every non-berries item). A load lands on ONE pile cell. `food` sums the counts.
- **Q12. Which bushes take a mark:** ripe ones only. A drag over growing bushes or bare ground marks nothing there; a drag with no ripe bush is refused.
- **Probe (point 3, "idled with forageable bushes"):** not reproduced. Fresh 256² game, crew 5 + arrivals, a 41x41 area over 32 bushes, 6,000 ticks: the woodcutter never idled more than 1 tick between bushes. "Not the nearest" reproduced: job ids followed the area's cell order, so FIFO walked him 20 cells west past a bush 6 cells from camp. Q9 answers it.

**Wire diff (amended):** keep `ItemKind::Berries`, `PlantState`, `Plant`, `PlantChange`, `Snapshot.plants`, `Delta.plants`, `DwarfJob::Forage`. ADD `DesignationKind::Forage` (wire `forage`) and `Item.count: u32`. REMOVE `ForageCell`, `Snapshot.forage`, `Delta.forage`, `Command::PlaceForageArea`, `Command::RemoveForageArea`, `Refusal::PlaceForageArea`.

**Saved fields (amended):** `SaveState.plants` stays; `SaveState.forage` goes; each saved item gains a required `count`. Forage marks save as designations.

## Found at creation (2026-11-21, `main` `3c6cf3b7`)

- **A. Nothing plant-shaped exists, and neither existing store fits an area.**
  - The only mention is the `MAX_DESIGNATIONS` comment (`lib.rs:71-72`), which says forage marks will add to the cap. Under Q3 (a) that is stale; rewrite it.
  - `Designations` is one kind per cell, and a finished job deletes its mark (`lib.rs:2853-2861`).
  - Every `Zone` is a pile: hauling (`create_haul_jobs` `:558`, "stored" `:508`), the tui's stored colour (`view.rs:325-339`) and the gui's overlap nudge (`project.rs:2296-2312`) all assume it. Reusing `Zone` would make berries "stored" where they lie.
- **B. On the bench, woodcutters never clear their cuts.** `two-hundred-on-256` ends at `done=dig:321,cut:96` (`13-8b-signoff/state-head-a28a5523.txt`): 96 of 200 cut marks by t2,500. FIFO is ascending `JobId` within a trade (`lib.rs:696-700`), so forage jobs created after tick 6 would never be claimed inside the window. Hence `at: 1` (Q4).
- **C. Haul is slow on the bench.** Only `stored=36` of `items=711` by t2,500 (#235). Berries reaching the pile inside the window is therefore plausible but not certain. If `food=0`, that is a finding for Wolf, not a reason to change haul code here (#235 owns it).
- **D. The bench end-line reader splits on `",cut:"`** (`simd/tests/bench.rs:292-295`). A field appended inside `done=` breaks its parse, so new fields go after `arriving=` (`scenario.rs:433`).
- **E. Uncarried items block cells** for claim-time reachability (#162, `lib.rs:662-663`). A forage that drops berries onto a cell where earlier berries still lie would stack items (the #154 class).
- **F. `Mirror::apply_delta` destructures `Delta { .., }`** (`client-core/src/lib.rs:244-295`), so a new `Delta` field is dropped silently unless it is added there. About 120 `Snapshot`/`Delta` struct literals across the gui, tui and client-core tests need the new fields.
- **G. The `Cut` wire name was never pinned** (`bridge.rs:473-487` lists Dig and Channel only). Pin Cut and every new variant.
- **H. The simd load validator** counts `Dig|Channel|Cut` as tile jobs against `MAX_DESIGNATIONS` and treats every other job as a haul (`main.rs:660-680`, `:713-732`). A Forage job falls into the haul arm unless it gets its own.

## Acceptance Criteria

1. **Plants grow, seeded (FR58, AD-7; mechanism AC because NFR9 is about reproducibility).**
   - Worldgen places berry bushes on the eligible cells (Q2), all ripe at tick 0, from `STREAM_PLANTS` alone.
   - Seed 42 at 128² holds a plant count inside a band the dev pins (±15% of the measured count). Same seed ⇒ identical plants; another seed differs.
   - No plant stands in the lake, the camp box, the edge ring or a tree column, or on anything but snow.
2. **Plants change no existing outcome (standing AC 11, FR53).**
   - Every pin in `tests/worldgen.rs` passes unchanged, the terrain fingerprint (`:502`) included.
   - Every committed scenario run as on `main` (bench file per Q4) has the same end line as `main`, except for the new end-line fields.
   - A throwaway build that drops `plants` and `forage` from the encoding reads `state` `same` against `main` on all of those rows.
3. **Forage marks (amended, Q8, Q12; FR58, NFR11).**
   - `Designate { kind: Forage, rect }` marks each RIPE bush whose cell is in the rect or directly above a cell of it, so a drag on the ground reaches the bush standing on it. Growing bushes and bare ground take no mark.
   - A rect with no ripe bush is refused with `Refusal::Designate { kind: Forage, rect }`; both clients show it on their refusal line.
   - Marks count against `MAX_DESIGNATIONS`. `CancelDesignation` removes a forage mark in its rect or directly above it.
4. **Haulers forage on trips (amended, Q7, Q10, Q11; FR58).**
   - A forage mark makes one `Forage` job, `trade` Hauler. The hauler works from a walkable side neighbour for `FORAGE_WORK_TICKS`; the bush turns `Growing`, its mark is removed, and the hauler now carries one `Berries` item with `count` 1.
   - While the count is under `FORAGE_TRIP_BUSHES`, the hauler goes on to the nearest other ripe marked bush within `FORAGE_TRIP_RADIUS` whose job nobody holds, and each bush adds 1 to the same item.
   - Then it carries the item to the pile and it lands on one pile cell. With no pile, or none reachable, the existing abnormal exit drops it where the hauler stands, and an ordinary haul job takes it later.
   - At all times a hauler carrying berries holds a job (the claim invariant: a free dwarf carries nothing).
5. **The first bush is the nearest (amended, Q9; mechanism AC: AD-12's single claiming system).**
   - Two marked bushes, the older one far: a free hauler goes to the near one first.
   - Kinds keep FIFO: a haul job older than a forage mark is taken first by the only hauler.
6. **Regrowth (amended).**
   - A picked bush is ripe again `REGROW_TICKS` later, unmarked, and is announced to the clients on that tick. It is foraged again only if it is marked again.
7. **A plant whose ground goes is gone.** A channel (or a dig of the tile under it) removes the plant and its mark. A held forage job is retired and its holder released (dropping any load where it stands). The clients see the bush disappear.
8. **Determinism and save (amended; NFR9, AD-11, standing AC 12).**
   - A world saved mid-trip (a hauler carrying berries with count 2 and holding a forage job, a bush mid-regrowth), loaded and stepped through the regrowth, equals the never-saved world, the `plants()` reader and item counts included.
   - A save missing `plants` or an item's `count` is refused, with a positive control.
   - The three committed seat saves are re-patched (no `forage` key; `count: 1` on every item), and their scenarios keep their end lines.
   - `simd`'s load validator refuses an out-of-bounds plant, a Forage job whose target holds no plant or no forage mark, a reused target, and an item count of 0, a non-berries count other than 1, or a berries count over `FORAGE_TRIP_BUSHES`, each naming the cause.
9. **Both clients show it (amended; NFR10).**
   - `client-core::Mirror` tracks a plant from ripe to growing to ripe to gone, and each item's count.
   - The tui draws ripe and growing bushes, berries, and a forage mark, each distinguishable, with no TUI regression and no TUI input.
10. **The gui forage mode (amended; standing AC 16, 17).**
    - Key 6 is a designation mode like cut: a drag sends `Designate { kind: Forage }` per surface rect, the preview lights the ripe bushes in the box, a refusal shows on the refusal line, and Clear cancels marks.
    - Bushes draw in two states, a marked bush shows a mark the way a marked tree does, and berries draw as an item.
11. **The fortress forages by itself on the bench (FR67, standing AC 14).**
    - `two-hundred-on-256` (Q4) ends with `foraged=K food=F`, both ≥ 1. `food` sums the counts of stored berries, by 3.3's stored rule (amended: Q11). Its opening forage order is now forage designations at tick 1 over the same box. No order is given after the scenario's opening commands.
    - The `#[ignore]` release test `the_two_hundred_on_256_scenario_does_real_work` asserts both.
    - If `food=0`, the run is recorded and Wolf rules (Found C).
12. **The bench delta is recorded (standing AC 10).** `RUNS=5 scripts/tick-bench.sh main HEAD` runs over the committed files, with `main`'s copy of the bench file (Q4). Each row's delta is recorded against its floor, and the forage file is recorded as a HEAD-only row.
13. **The instruments are tested (standing AC 8).**
    - The bench end line's `foraged=` and `food=` read `0 0` on the one-dig fixture and `≥1 ≥1` on a small forage fixture, driven through the binary.
    - The tui capture shows a ripe bush turning into a growing one.
    - The gui forage drag is pinned by unit and headless tests: `--drag` is dead headless (12.7's deferral), so no live capture.
14. **The mutation table kills** (Task 10), and `python3 scripts/audit-mutations.py` exits 0.
15. **The seat (standing AC 16, 17).**
    - Wolf marks a forage area in a fresh game and watches woodcutters forage, berries hauled, and a bush regrow.
    - He judges the plain look. His words are recorded.
    - The gui frame rate at 256² is recorded beside seat run 5's (NFR13). A miss is measured and ruled.

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings (Q1–Q6).** Record the answers in the Change Log and amend the ACs they name. Q5's answer is the approved draft. Then ASK which implementer to use (memory: `dev-implementer-is-sonnet-not-codex`).
- [x] **Task 1: plants in worldgen (AC 1, 2).** UPDATE `crates/sim-core/src/worldgen.rs` and `src/lib.rs`.
  - [x] Add `STREAM_PLANTS` (ASCII `"PLANTS__"`) beside the streams (`lib.rs:31-38`), and the constants `PLANT_ONE_IN = 16`, `FORAGE_WORK_TICKS = 30` and `REGROW_TICKS = 3_000` beside `CUT_WORK_TICKS` (`:86-93`).
  - [x] NEW `pub(crate) fn place_plants(dims, tiles: &[Tile], camp, rng) -> BTreeMap<Pos, Plant>` in `worldgen.rs`, modelled on `place_trees` (`:375-443`). Call it in `generate` after `place_trees` (`lib.rs:3392-3393`) with `ChaCha8Rng::seed_from_u64(seed ^ STREAM_PLANTS)`. Find the top tile by scanning the column, because heights are not kept. A tree column holds a `TreeTrunk`.
  - [x] NEW resource `Plants { by_pos: BTreeMap<Pos, Plant>, changed: BTreeSet<Pos> }` with `pub struct Plant { pub ripe_at: u64 }`. `assemble` (`:3305-3318`) takes it. Add the public readers `World::plants() -> Vec<(Pos, bool /*ripe*/)>` and `World::drain_plant_changes() -> Vec<(Pos, Option<bool>)>`, the second shaped like `drain_dirty` (`:3724`).
  - [x] Tests in `tests/worldgen.rs`: the AC 1 band, eligibility, and same/different seed. Do not touch the existing pins; AC 2 needs them to pass as they are.
- [x] **Task 2: the forage area (AC 3).** UPDATE `lib.rs`.
  - [x] NEW `SimCommand::PlaceForageArea { rect }` / `RemoveForageArea { rect }`, and `Refusal::PlaceForageArea { rect }` (`:152-161`). NEW `World::place_forage_area(&[Rect]) -> Option<Refusal>`, the twin of `place_stockpile` (`:3734`): refused only when no rect keeps a plant cell, naming the bounding box.
  - [x] NEW resource `ForageArea(BTreeSet<Pos>)`, beside `Zones` (`:841`). Cells are kept by the stockpile's standable test (`:3947-3960`). Add the reader `World::forage_area() -> Vec<Pos>`.
  - [x] Rewrite the stale `MAX_DESIGNATIONS` comment (`:71-72`, Found A).
- [x] **Task 3: forage jobs, regrowth, plant loss (AC 4–7).** UPDATE `lib.rs`.
  - [x] `JobKind::Forage` (`:360`). Extend every exhaustive site:
    - `trade()` (`:624`) maps it to `Woodcutter`. That is all AC 5 needs; `claim_jobs` itself does not change (AD-12).
    - `Jobs::insert/remove` (`:430`, `:450`) treat it as a tile job, unique by target.
    - `work_positions_with` (`:1593`) joins the `Dig | Cut` arm's walkable side neighbours, without the Dig-only item fallback.
    - The work-time match (`:2733-2737`) and the `unreachable!` at `:2889`.
    - The CancelDesignation job filter (`:3921`) leaves Forage alone; an area is removed by `RemoveForageArea`.
  - [x] In `create_jobs` (`:479`), after the designation loop: create a Forage job for each ripe plant in the area whose cell is not in `jobs.targets` and holds no uncarried item (Found E). Also retire, with a release, each Forage job whose plant is gone, unripe or outside every area (the `create_haul_jobs` retire shape, `:558-575`).
  - [x] Forage completion, beside cut completion (`:2853-2861`):
    - set `ripe_at = tick + REGROW_TICKS` and mark the plant changed;
    - spawn `Item(Berries)` at the plant cell with `ids.allocate()`;
    - remove the job and `release_claim`.
    - No `clear_paths`: nothing walkable changed.
  - [x] NEW system `tend_plants`, chained after `execute_jobs` (`:3337-3353`), which skips while paused (AD-2). It removes each plant whose cell is no longer standable and marks it changed, and it marks changed each plant whose `ripe_at == tick`.
  - [x] `ItemKind::Berries` (`:197`). `yield_of` (`:208`) is unaffected, because berries come from no material. `goal_kind_text` and the GOALS count by kind are unchanged.
  - [x] Scenario tests in `tests/scenario.rs` for AC 3–7, built on the timber helpers (`:2445-2510`) and `a_cut_is_hauled_to_the_pile_as_wood` (`:2579`). Find a plant near `camp_origin()` from `world.plants()`; never hardcode one.
- [x] **Task 4: save and the load validator (AC 8).** UPDATE `src/save.rs`, `lib.rs` (`to_save` `:3486-3571`, `from_save` `:3573-3591`), and `crates/simd/src/main.rs`.
  - [x] `SaveState.plants` and `SaveState.forage`, with no `#[serde(default)]`.
  - [x] Validator (`main.rs:594`): bounds checks for plants and forage cells. Forage jobs get their own arm (Found H): unique target, a plant at the target, and a count bound of the plant count, kept out of both the tile-job and the haul counts. Refusal tests go in `simd/tests/serve.rs`.
  - [x] `tests/save_load.rs`: add the two readers to the per-tick equality list (`:170-188`), a mid-regrowth variant modelled on `:298-365` with its guard, and the missing-key tests modelled on `:1222-1267`.
  - [x] Patch `scenarios/saves/*.save` per Q6 with a throwaway script, not committed.
- [x] **Task 5: the wire (AC 9).** UPDATE `crates/protocol/src/lib.rs`, `crates/simd/src/bridge.rs` and `main.rs`, and `crates/client-core/src/lib.rs`.
  - [x] The protocol types from the Wire diff, with snake_case names pinned in the wire-literal test (`protocol :346`).
  - [x] Bridge: `snapshot()` and `delta()` (`bridge.rs:1-90`). `item_kind_out`, `refusal_out` (`:348-361`). Pin Cut, Berries and the plant states in the wire-name tests (`:429-487`, Found G).
  - [x] `apply_sim_command` (`main.rs:513-552`) and `command_rects` (`:1226-1238`) take the two new commands. The scenario reader accepts them through the same path.
  - [x] `Mirror`: `plants: BTreeMap<Pos, PlantState>` and `forage`. `apply_delta` applies `plants` as changes and `forage` as a full resend (Found F). `refusal_text` gets the new arm. Add tests in `client-core/tests/mirror.rs`.
- [x] **Task 6: the tui (AC 9).** UPDATE `crates/tui/src/view.rs` and `palette.rs`.
  - [x] Draw order: zones, forage cells, designations, plants, items, then entities. The glyphs and colours are the ones Q5 approves.
  - [x] Extend `DrawTally` (`view.rs:75-135`) and the glyph-distinctness test (`palette.rs` ~`:497-520`). Plant glyphs must be ASCII (#152).
  - [x] Plants on other levels follow #172's existing peek rule. Add a `// NOTE:` naming #172; no fix here.
- [x] **Task 7: the gui (AC 10).** UPDATE `crates/gui/src/designate.rs`, `project.rs`, `appearance.rs`, `ingest.rs` and `capture.rs`.
  - [x] `DesignateMode::Forage` on `Digit6`, with the hints from Q5 (ASCII, test `:497`). `commands_for` (`:295-345`) sends `PlaceForageArea { rects: surface }`. Clear also sends `RemoveForageArea` for each surface rect. Add Digit6 to the keymap guard (`ingest.rs:5051`, Digit list `:5119-5123`).
  - [x] Preview: `sim_will_keep` (`project.rs:1079-1105`) keeps standable cells. `preview_appearance` (`:1111-1145`) uses a berry-coloured slab and lights the bushes in the box, as cut lights tree bases (`:3916-3941`).
  - [x] Render: forage cells as a slab like zones (`:2290-2380`) with a NEW `appearance::forage_color`. Each bush is one entity with ONE shared mesh per state and shared materials, so 256²'s ~3k bushes batch. Berries use the item cube (`:2267-2277`), with `item_translation`, `item_scale` (`:2383-2393`) and `appearance` extended exhaustively.
  - [x] `--drag` takes `forage` (`parse_drag` `ingest.rs:1458-1486`, `mode_key` `:1548-1557`). `assert_drag_produced_work` (`capture.rs:169`) gains a forage arm: zero forage cells fails the capture.
- [x] **Task 8: the instruments (AC 11, 13).**
  - [x] `simd --bench`'s end line (`scenario.rs:433`) appends ` foraged=K food=F` after `arriving=` (Found D). `foraged` counts forage completions, the way `done=` counts cuts. `food` counts stored `Berries`, by `split_stored`'s rule.
  - [x] Check every reader of the end line and of `next_log` before you change the line (memory: `new-log-line-breaks-next-line-readers`).
  - [x] NEW fixture `crates/simd/tests/fixtures/one-forage.json`: a small seed world, a pile, a forage area on a few plants, and enough ticks for one forage and one haul. A test in `simd/tests/bench.rs` reads `foraged=0 food=0` on `one-dig.json` and `≥1 ≥1` on `one-forage.json`.
  - [x] A tui test in `tui/tests/client.rs` drives the binary over a served world: the capture's ripe-bush count at a plant falls and its growing count rises after a forage. A gui headless test in `gui/tests/headless.rs`: every mirror plant has a projected bush in the right state.
  - [x] **RED first, for each instrument:**
    - Comment out the `foraged`/`food` increment; the fixture test must fail on `foraged=0`.
    - Draw growing bushes with the ripe glyph; the tui test must fail.
    - Make the forage arm of `commands_for` send nothing; the `--drag forage` capture must fail. (Skipped by the Task 8 agent: handled by the orchestrator.)
    - Restore each step with an edit, never `git checkout --` (memory: `sabotage-restore-trap`).
- [x] **Task 9: the bench scenario and the delta (AC 2, 11, 12).**
  - [x] Per Q4, add one `place_forage_area` at `at: 1` to `two-hundred-on-256.json`, picked by a throwaway script: the rect nearest the pile (`(143,144)`) with at least 30 plants. Record the plant count and the rect in `about`. Add a re-pin line to the doc comment of `the_two_hundred_on_256_scenario_does_real_work` (`bench.rs:266-274`), plus the AC 11 asserts.
  - [x] AC 2: run every committed scenario's end line on `main` and HEAD, and save both to `13-9-signoff/state-{main,head}-<sha>.txt`. Run the strip build in a scratch worktree (memory: `mutation-run-poisons-concurrent-builds`).
  - [x] AC 12: save `RUNS=5 scripts/tick-bench.sh main HEAD` to `13-9-signoff/tick-bench-main-<sha>.txt`. Record the delta table in the Debug Log, in 13.8's form.
- [x] **Task 10: mutation table (AC 14).** NEW `_bmad-output/implementation-artifacts/mutations/13-9.sh`, with count-guarded anchors, and save the run to `13-9-signoff/`. At least one row each for:
  - the area refusal;
  - `trade(Forage)`;
  - the item-on-cell skip;
  - the retire-on-area-removal;
  - `REGROW_TICKS` use;
  - plant loss in `tend_plants`;
  - the `changed` mark on forage;
  - `apply_delta`'s plants arm;
  - the save fields' required keys;
  - the validator's Forage arm.

  Run `audit-mutations.py` before the first code commit, so that rows against code you rewrite are re-pointed in the same commit (memory: `deleting-code-retires-rows-in-the-same-commit`).
- [ ] **Task 11: the seat card (AC 15).** NEW `13-9-signoff/vehicle-card.md` in 13.8b's skeleton:
  - the run in the seat's form (persistent fact: `launch-gui.ps1`, `simd 7451`), with `--perf-log` to an absolute `D:\Workspace\frostvein\.bin\` path;
  - what to do: mark an area at key 6, watch a forage, a haul and, at fast4x, a regrowth;
  - one question;
  - the sitting, recorded in Wolf's words.
- [x] **Task 12: the gate.** `RUST_TEST_THREADS=1 scripts/gate.sh`, once, at PR (memory: `gate-ooms-at-default-parallelism`).

- [ ] **Amendment 1 rework (Q7–Q12).** Sonnet 5.5 subagents for R1–R3 in turn on one tree; Opus does R4.
  - [ ] **R1, sim-core + save + validator (ACs 3–8).** `DesignationKind::Forage` (ripe-only, cell-or-above, refusal); remove `ForageArea` and its commands and `create_forage_jobs`' area logic; `trade(Forage)` = Hauler; nearest-first forage claim inside `claim_jobs` at a forage job's FIFO turn (one multi-goal search over the claimable forage jobs' work cells); trip continuation and the berries count on one carried item; delivery through the haul machinery; plant loss removes the mark; `SaveState.forage` removed, item `count` added; validator arms; seat saves re-patched. Rewrite the area tests as mark tests.
  - [ ] **R2, the wire and the instruments (ACs 9, 11, 13).** Protocol per the amended wire diff, bridge, `apply_sim_command`, Mirror counts, serve tests; `food` sums counts; `one-forage.json` uses forage designations.
  - [ ] **R3, the clients (ACs 9, 10).** tui forage mark (no `:` area); gui key 6 as a designation mode like cut, the mark on a marked bush, no slab.
  - [ ] **R4, Opus:** the bench file's opening order as designations, AC 2 strip build (plants + item count), tick-bench, a rewritten `13-9.sh`, the full gate, the seat card's run 2.

## Verification

Nothing here can run at creation, because the feature does not exist. Each line states the obligation the dev inherits: the exact command, the non-zero observation, and the RED that must be seen first.

```bash
# AC 1-8: sim
cargo test -p sim-core
#   expect: the new worldgen, scenario and save_load tests pass, and every existing worldgen pin passes unchanged

# AC 2, 11: end lines, main vs HEAD (release, every committed scenario)
for f in $(git ls-files 'scenarios/*.json'); do target/release/simd --bench "$f" --runs 1 | rg ' end '; done
#   expect HEAD: every row except two-hundred-on-256 equals main's line up to the new ' foraged=0 food=0'
#   expect HEAD two-hundred-on-256: foraged>=1 food>=1 (if food=0, record it and Wolf rules)
#   RED: comment out the food increment -> food=0 on one-forage.json and the fixture test fails; restore by edit

# AC 12: bench delta
RUNS=5 scripts/tick-bench.sh main HEAD 2>&1 | tee _bmad-output/implementation-artifacts/13-9-signoff/tick-bench-main-$(git rev-parse --short=8 HEAD).txt

# AC 9, 13: tui instrument (pin the level with --z; read it from the plant chosen by the test, never assume)
target/release/simd 7461 --dims 128,128,32 & sleep 3; target/release/tui 7461 --frames 3 --z <plant z> > tui.out; kill %1
sed 's/\x1b\[[0-9;?]*[A-Za-z]//g' tui.out | rg -o '"' | wc -l
#   expect: > 0 bushes on that level.  RED: draw no plants -> 0, and the tui test fails

# AC 10, 13: gui drag instrument (headless, devpod lavapipe)
target/release/gui <port> --headless --drag forage,<x0>,<y0>,<x1>,<y1> --frames 40 --capture /abs/forage.png
#   expect: the capture passes with forage cells > 0.  RED: empty forage arm in commands_for -> the capture fails

# AC 14
env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-9.sh
python3 scripts/audit-mutations.py

# gate (once, at PR)
RUST_TEST_THREADS=1 scripts/gate.sh
```

The seat run lives on the vehicle card (Task 11), in the seat's form: WSL `simd 7451`, then PowerShell `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-9-seat.csv')`.

## Dev Notes

### Scope guardrails (do NOT)
- No eating, hunger or food consumption (13.10). No cooking or meals (13.20). No new trade: forage is the hauler's (Amendment 1 Q7; was the woodcutter's).
- No change to `claim_jobs`' ordering (AD-12) except Amendment 1 Q9: at a forage job's FIFO turn a free hauler claims the NEAREST claimable forage job. Every other kind is untouched.
- No haul change, even if `food=0` on the bench (#235 owns slow hauls; Found C).
- No #172 fix, no #214 off-map fix, no #170 cap fix. No TUI input (NFR10).
- No art rounds. The bushes are plain primitives, and look tuning waits for a concrete defect judged at the seat (memory: `art-gates-visual-judgement`).
- No plant clustering, no seasons, no plant spreading.

### What already exists (build on it)
- The cut job is the forage job's template: mark, walk to a side neighbour, work, drop at the target, release (`lib.rs:1554`, `:2853-2861`). The addendum calls forage "a near copy" (`prds/prd-frostvein-2026-10-28/addendum.md:9`).
- `place_stockpile` / `Zones` / `Refusal::PlaceStockpile` is the area's template, end to end, through `bridge::refusal_out` and `client-core::refusal_text`.
- Haul is kind-agnostic: berries haul and store with no change (`lib.rs:558`, `:1644`, `:2798-2851`).
- `Terrain.dirty` / `drain_dirty` is the template for `Plants.changed` / `drain_plant_changes` (`lib.rs:848-914`, `bridge.rs:55-60`).
- 12.7 (`110cab48`, PR #171) added Wood, Cut and the gui cut mode end to end. Its file list is this story's file list.

### Key decisions & traps
- **A plant is not a tile and not an entity** (Q1 a). That is why AC 2 can demand unchanged outcomes. Giving plants `Id`s would shift every item id and move every haul order.
- **Ripeness is derived from the tick** (`tick >= ripe_at`). Only the wire needs to hear about it, through `changed`. There is no per-plant timer to tick.
- **Forage jobs are not designations.** They are re-derived each tick from area ∩ ripe plants, so a regrown plant gets a fresh job (and a fresh, later `JobId`) with no store to update.
- **The bench's forage area goes at `at: 1`** (Found B). Otherwise 104 unfinished cuts starve it.
- **New end-line fields go after `arriving=`** (Found D).
- **Add the new `Delta` fields to `apply_delta`** (Found F). The compiler will not tell you.
- **A forage job skips a cell holding an uncarried item** (Found E), so berries never stack on a bush.
- **Berries can drop under a dwarf** standing on the bush, because a bush blocks nothing. Add a scenario test that a dwarf on the bush's cell when the berries land still walks off and the berries are hauled.
- **Bushes batch:** one mesh per state and shared materials. At 256² there are about 3k of them, and NFR13 is measured at the seat (#224 is the known fps debt).
- **`tend_plants` is a full scan of the plants per tick** (about 3k at 256²). That is fine at that count; leave a `// NOTE:` naming it. AC 12 measures it.

### Previous story intelligence
- 13.8 kept committed scenarios byte-identical by making the new thing a saved setting, and proved the reach with a strip build (`13-8-they-keep-coming.md` AC 10). AC 2 reuses that proof.
- 13.8b: the bench row for `two-hundred-on-256` is the one Wolf watches. Its mean on `main` is about 5.5 ms on the devpod (`13-8b-signoff/tick-bench-main-33b4361c.txt`).
- Run the gate with `RUST_TEST_THREADS=1` (45–65 min). Build probes in a scratch worktree, never in the shared tree (memories: `probe-sabotage-leaks-into-wolfs-build`, `cargo-target-dir-builds-cwd-sources`).

### Project Structure Notes
- UPDATE:
  - sim-core: `crates/sim-core/src/lib.rs`, `src/worldgen.rs`, `src/save.rs`, `tests/worldgen.rs`, `tests/scenario.rs`, `tests/save_load.rs`
  - protocol: `crates/protocol/src/lib.rs`
  - simd: `crates/simd/src/bridge.rs`, `src/main.rs`, `src/scenario.rs`, `tests/bench.rs`, `tests/serve.rs`
  - client-core: `crates/client-core/src/lib.rs`, `tests/mirror.rs`
  - tui: `crates/tui/src/view.rs`, `src/palette.rs`, `tests/client.rs`
  - gui: `crates/gui/src/designate.rs`, `src/project.rs`, `src/appearance.rs`, `src/ingest.rs`, `src/capture.rs`, `tests/headless.rs`
  - `scenarios/two-hundred-on-256.json`; `scenarios/saves/*.save` (data patch)
- NEW:
  - `crates/simd/tests/fixtures/one-forage.json`
  - `_bmad-output/implementation-artifacts/mutations/13-9.sh`
  - `_bmad-output/implementation-artifacts/13-9-signoff/` (vehicle card, state lines, tick-bench, mutate runs)
- No new crate, no new dependency, no new crate edge.

### References
- Epic 13, Story 13.9, standing ACs 10–18: `_bmad-output/planning-artifacts/epics.md:2944-2966`, `:3186-3211`; M3 standing ACs `:2557-2570`
- FR58, FR67, NFR9–NFR13: `epics.md:152`, `:167`, `:205-219`; M4 requirements `:299-326`
- PRD addendum (forage = the Cut job shape): `prds/prd-frostvein-2026-10-28/addendum.md:9`
- 13.8 story (form, AC 10 strip-build proof): `13-8-they-keep-coming.md`; 13.8b record: `13-8b-after-the-big-dig.md`, `13-8b-signoff/`
- Issues: #118 (F-keys), #152 (wide glyphs), #154 (stacked items), #162 (items block), #172 (tui trees), #224 (gui fps), #235 (slow haul)

## Dev Agent Record

### Agent Model Used

Wolf picked Sonnet 5.5 subagents for the implementation (asked 2026-11-22). claude-opus-5-5 orchestrates and verifies, and keeps the REDs, the mutation run, the bench and the full gate.

### Debug Log References

**Task 9, AC 2 (2026-11-22, Opus 5.5):** `13-9-signoff/state-main-3c6cf3b7.txt` and `state-head-6de6fe11.txt`. All 12 committed scenarios (the 256² row on `main`'s copy of its file) end exactly as on `main` plus ` foraged=0 food=0`. A throwaway build at `6de6fe11` with `#[serde(skip_serializing)]` on `SaveState.plants` and `.forage` reads `state` equal to `main` on all 12.

**Task 9, AC 11:** HEAD's `two-hundred-on-256.json` ends `done=dig:286,cut:22 stored=17 cells=770 foraged=32 food=9` (main's file on `main`: `done=dig:321,cut:96 stored=36 cells=2343`). The forage jobs sit first in the woodcutters' FIFO (Found B), so cuts fall from 96 to 22 in the window.
- **Area pick deviates from "nearest the pile":** 54 of this world's 1,892 bushes stand on pile cells. Their berries land on the pile and count as stored with no haul, so an area holding them would make `food` vacuous. The area is the nearest 30-bush box that touches neither the pile nor a dig or cut mark: (166,151)-(195,180) on levels 11-19, 32 bushes.

**Task 9, AC 12:** `13-9-signoff/tick-bench-main-a11e21d6.txt` (RUNS=5, mean ms).

| scenario | main | HEAD | delta | control | verdict |
|---|---|---|---|---|---|
| big-pile | 0.0333 | 0.0389 | +16.9% | +4.5% | beyond floor |
| deep-pit | 0.0203 | 0.0240 | +18.2% | +3.0% | beyond floor |
| dig-loop | 0.0218 | 0.0228 | +4.8% | +4.0% | within floor |
| full-pile-crowd | 0.0256 | 0.0305 | +19.1% | +5.7% | beyond floor |
| plan-compact-piles | 0.0416 | 0.0462 | +11.3% | +0.1% | beyond floor |
| plan-far-piles | 0.0361 | 0.0410 | +13.4% | +1.9% | beyond floor |
| plan-spread-piles | 0.0418 | 0.0467 | +11.7% | +1.6% | beyond floor |
| plan | 0.0349 | 0.0398 | +14.1% | +1.9% | beyond floor |
| seat-slow | 0.0695 | 0.0709 | +2.0% | +1.1% | within floor |
| stuck-tunnel | 0.0260 | 0.0266 | +2.3% | +0.9% | within floor |
| they-keep-coming | 0.0875 | 0.1026 | +17.2% | +0.6% | beyond floor |
| two-hundred-on-256 (main's file) | 5.5499 | 5.5863 | +0.7% | -1.3% | within floor |
| two-hundred-on-256 (HEAD's file, HEAD only) | | 4.28-4.38 | | | 5 runs, spread 2.3% |

- **Reading:** the rows beyond the floor are the generated 128² worlds, each with about 480 bushes. They pay about +5 us a tick, `tend_plants`' full scan (its `// NOTE:`). The three save-loaded rows (dig-loop, seat-slow, stuck-tunnel) hold no plants (Q6) and stay within the floor.
- **At 256²** (1,892 bushes) the scan is under 1% of a 5.5 ms tick. The forage file's tick is cheaper because woodcutters cut less.

**Task 12, the gate:**
- The first full gate at `f80e6e62` was RED on one check only: the AO pixel guard's open-snow medians read LL 59 / LR 57 against controls 60/59. Four bushes stand in the LL window.
- The cause is attributed: the same build with bush drawing switched off reads 60/59.
- Wolf ruled the re-baseline (2026-11-22), committed as `5963e6d3`.
- The full gate at `5963e6d3` is GREEN in 2,311 s: cargo test, the 256² release run, all 17 pixel guards, and the mutation tables.

**Task 10, AC 14:** `13-9-signoff/mutate-13-9-cae33344.txt` has 16 of 17 rows KILLED. Row 8, "a regrown plant is never announced", SURVIVED: no test read the regrowth's `PlantChange`. Ripeness is derived from the tick, so without the announcement both clients would draw a regrown bush as growing for ever. New test `a_regrown_plant_is_announced_on_the_tick_it_ripens` (`tests/scenario.rs`). Row 8 is re-pointed to it, and the re-run is KILLED at that test's assert (`mutate-13-9-row8-rerun.txt`). `audit-mutations.py` exits 0 (1,055 rows).

### Completion Notes List

- Tasks 1-3 (sim-core), by a Sonnet 5.5 subagent. Plants are `Plants { by_pos, changed }`, the area is `ForageArea`; neither touches `set_tile`, `clear_paths` or the id counter, and every existing pin and scenario passed unchanged (AC 2's sim half).
- Forage jobs are derived by a new exclusive system `create_forage_jobs` chained right after `create_jobs` (not inside it: the retire needs `release_claim`, which `create_jobs`' `Res` signature cannot do). Designation jobs keep their ids.
- `place_forage_area(&[Rect])` is the one place logic; `SimCommand::PlaceForageArea` calls it with one rect. A refused call places nothing. `RemoveForageArea` goes through the rect prelude.
- Berries are Item(Berries) spawned at the plant cell, `ids.allocate()`d only on a completed forage; a forage whose plant is lost or unripe at completion spawns nothing.
- Lake exclusion in `place_plants` is redundant with the snow rule (lake tops are ice), so no test can kill removing it; kept because the story names it.
- A Forage job shares `Jobs.targets` with Dig/Channel/Cut, so a Channel marked on a cell whose Forage job already stands waits for that job (tile-job uniqueness, as the story says). Not changed.
- `from_save` passes empty plants and forage with a `TODO(13.9 Task 4)`; a loaded world has no plants until Task 4. simd has `TODO(13.9 Task 4/5)` arms: `bridge.rs` (`item_kind_out`, `dwarf_job`, `refusal_out`: `unreachable!`) and `main.rs` (load validator: a Forage job is refused).
- Mutation rows re-pointed in the same commit: 10-9 (lake exclusion) and 5-1 (camp clearing) now count 2 matches and mutate the first (the trees'); 12-7 row 4 (`trade`); 3-3 `carry_items runs before settle` (`tend_plants` in the schedule); 2-4 `loaded world schedule omits wander` and `from_save marks every tile dirty` (`assemble` call tail). Task 4 will move the 2-4 anchors again.
- Task 4 (save and validator), by a Sonnet 5.5 subagent. `SaveState.plants: Vec<(Pos, u64)>` and `forage: Vec<Pos>` have no `#[serde(default)]`; a missing `Vec` key errors by itself, and `a_save_missing_a_plants_or_forage_key_is_refused` proves it with a control that holds non-empty values. Plant `changed` is not saved (a load starts clean, like dirty tiles); the mid-regrowth test drains the control's pending changes and then asserts the regrow change lands on the same tick.
- The validator: bounds for plants and forage cells; a Forage arm (shares `seen_job_targets` with the tile jobs, as the sim shares `Jobs.targets`; a plant at the target; count <= plant count, message-quality like the haul bound), kept out of the tile-job and haul counts. Refusal tests in `simd/tests/serve.rs` go through a real daemon Load, with a positive control that the live Load takes a save holding a Forage job and no item (that control is what dies if Forage is counted as a haul). Not tested: Forage counted against `MAX_DESIGNATIONS` (would need 4097 plants with jobs).
- The three seat saves were patched as data (`"plants":[],"forage":[]` appended, single line, no trailing newline kept). `dig-loop`, `seat-slow` and `stuck-tunnel` bench end lines are byte-identical to 13.8b's. No test pins a state hash, so none was re-pinned.
- Mutation rows re-pointed: 2-4 (`assemble` call tail, and the `from_save` destructure anchor), 3-2 `load accepts duplicate job targets` (the Forage arm repeats the check, so the anchor now starts at the tile-job arm).
- Tests: `tests/worldgen.rs` (3), `tests/scenario.rs` (13), `lib.rs` `a_dwarf_on_the_bush_as_the_berries_land_walks_off_and_they_are_hauled` (needs ecs access: a hand-built world cannot hold plants before Task 4).

- Task 5 (the wire), by a Sonnet 5.5 subagent. Protocol: `ItemKind::Berries`, `PlantState`, `Plant`, `PlantChange`, `ForageCell`, `Snapshot.plants/forage`, `Delta.plants/forage`, `Command::PlaceForageArea/RemoveForageArea`, `Refusal::PlaceForageArea`; every name is pinned by a hand-written literal in the protocol wire tests. `PlantChange.state: None` serializes as `null`.
- GAP in the story's wire diff: it lists no job variant for a dwarf doing a forage. `DwarfJob` gains `Forage { target }` (wire `{"forage":{"target":[x,y,z]}}`), pinned in the protocol and bridge tests; without it `dwarf_job(Forage)` could only panic on a served world.
- Found G done: `Cut` (designation kind and job), `Berries` and both plant states are pinned in the bridge wire-name tests.
- `Mirror` keeps `plants: BTreeMap<[i32; 3], PlantState>` (wire positions are arrays) and `forage: Vec<ForageCell>`; accessors `plants()`, `plant(pos)`, `forage()`. `apply_delta` applies `plants` as changes (`None` removes) and `forage` as a full resend.
- The scenario reader needed no code: it deserializes `protocol::Command`, so `place_forage_area` is read through the daemon's own path. `bench.rs` has a test that a scenario places an area (state hash against a `sim_core` oracle) and a case in the unusable-scenario table for a refused one.
- TODO arms left for later tasks: tui `view.rs` item match (berries draw as wood, `TODO(13.9 Task 6)`); gui `project.rs` item material, drop and scale (berries share wood's, three `TODO(13.9 Task 7)`). The gui's `DwarfJob` uses are `matches!`, so none needed an arm.
- Mutation rows re-pointed (the Berries stand-in arms): 12-7 row 14, 3-2 `items draw above entities`, 3-3 `stone counting and stone drawing use different filters`. `audit-mutations.py` exits 0.
- Sabotage (each watched fail, restored by edit): `apply_delta` skipping plants (mirror test), bridge `Delta.plants` empty (serve test), `refusal_out` to the wrong variant (serve test), `Delta.forage` empty (serve test), `Berries` mapped to stone (bridge test), `apply_sim_command` dropping `PlaceForageArea` (bench and both serve tests).

- Task 6 (the tui), by a Sonnet 5.5 subagent. Looks (`palette.rs`): ripe bush `"` (214,62,62); growing bush `"` (58,112,66); berries `%` (222,70,84), stored berries `%` in the stockpile green; forage cell `:` (176,64,96). All ASCII. Order in `render`: zones, forage cells, designations, plants, items, entities.
- Glyph check: `"` and `:` are used by nothing else. `%` is the tetrahedrite ore TILE glyph (blue (60,90,210)); berries are an item layer over terrain and red, so no clash in a layer, and a palette test pins that no new cell equals the ore cell.
- Tally: the story's `DrawTally` is `MarkTally`. It gained `drawn_forage` and `mirror_forage` (`ForageTally { ripe, growing, berries, area }`), drawn counted on the map rows by exact cell (glyph and colour). `complete()` is unchanged. Printed by the `--frame` path on stderr (the only path that tallies; `--frames` streams frames and prints no tally): `forage: z N ripe=A of B growing=A of B berries=A of B area=A of B`.
- #172: plants use `screen_index` (same level only), as zones and items do; a `// NOTE:` names #172, not fixed.
- Tests: `view.rs` `forage_things_draw_their_glyph_and_colour`, `forage_draw_order_holds`, `the_forage_tally_counts_each_kind_and_range_checks`; `palette.rs` `the_forage_looks_are_pinned_ascii_and_distinct`. Sabotage (each watched fail, restored by edit): growing drawn ripe (3 fail), plants not drawn (3 fail), forage cells drawn after plants (2 fail).
- Mutation rows re-pointed: 3-2 `items draw above entities`, 3-3 `stone counting and stone drawing use different filters` (the Berries arms). `audit-mutations.py` exits 0.
- Task 7 (the gui), by a Sonnet 5.5 subagent. Mode `DesignateMode::Forage` on `Digit6`; HUD `6 forage` in the no-mode line, `forage: drag over the bushes  Esc leave` and `forage: release to mark  Esc abort` (ASCII). `commands_for` sends ONE `PlaceForageArea { rects: surface }` (an empty surface sends the picked rect, as the stockpile does, so the sim's refusal line shows); Clear sends `RemoveForageArea` per surface rect, after `RemoveStockpile`. `designation_target` and `preview_cells` treat Forage like the stockpile; `sim_will_keep` keeps standable, non-emitter cells.
- Look (plain primitives, no art): `forage_color` (176,64,96) slab, one entity per forage cell, raised and inset over a pile cell so the pile's rim shows. Bushes are ONE entity per plant (`ProjectedPlant { pos, state }`), with TWO shared meshes (ripe: leaf block plus four berry dots, growing: a smaller block) and ONE shared white material, the colours carried by vertex colours; so 256^2's ~3k bushes are two batches. Bushes carry no `WorldProjected`, so `blend_entities` never writes them. A state change swaps the `Mesh3d` handle in place; a removed plant is despawned. Berries: `berry_item` material (222,70,84), scale 0.24, drop on the floor, and shifted 0.3 sideways so a bush on the same cell does not swallow them (`item_translation`, `item_scale` and the spawn arm are exhaustive again; the three TODOs are gone). Preview: berry slab on kept cells, the cells that hold a bush use the berry-red item material (no tint system; the lit cell is the slab itself).
- `--drag forage` parses and presses key 6 (`mode_key`); `assert_drag_produced_work` has a forage arm over a new `DrawStats.expected_forage` (set by `with_expected_forage`, so `new`'s eight-argument callers are untouched). README key row and `--drag` row updated.
- GAP: `--drag` is dead headless (12.7's review, deferred, pre-existing): `gui <port> --headless --drag forage,600,340,700,400 --frames 40` panics `scripted --drag never completed`, in a release build too; `--frames 300` times out at 100 s under lavapipe. So the forage arm is proven by the `DrawStats` unit test, not by a live capture; the live drag is a seat check.
- Tests: `designate.rs` (forage commands, hints); `project.rs` (berry scale/drop/shift, forage keep rule and lit preview); `appearance.rs` (forage slab far from the zone slab); `ingest.rs` (`--drag forage`, keymap guard, the hint literal); `capture.rs` (the forage arm); `tests/headless.rs`: `every_mirror_plant_has_a_bush_in_its_state_and_a_forage_flips_it`, `the_forage_area_draws_a_slab_per_cell_and_a_pile_under_it_still_shows`, `a_berry_item_projects_as_a_small_red_cube_and_wood_still_as_a_log`, and `each_mode_key_...` gained key 6 and the Clear `RemoveForageArea`. Sabotage (each watched fail, restored by edit): forage arm sends nothing (2 tests), growing bushes keep the ripe mesh (headless), Clear omits `RemoveForageArea` (2 tests), the capture forage arm made vacuous (`mark_counts_are_checked_against_the_mirror_not_merely_against_zero`).
- Mutation rows re-pointed: 12-7 row 14, 8-2 (two rows), 12-10b row 13, 8-3 (the hint literal). `audit-mutations.py` exits 0.
- Task 8 (instruments), by a Sonnet 5.5 subagent. End line is now `... arriving=A foraged=K food=F`. `foraged` needs no sim-core counter: `scenario.rs` records the ripe plant set before each step and counts, after it, the plants that are still standing and not ripe (a forage completion is the only ripe-to-growing move; a lost plant is absent, a regrow goes the other way). `food` is the one sim-core addition, `World::stored_berries_count()`: `split_stored`'s stored set filtered to `ItemKind::Berries`. Readers checked: `tests/bench.rs` (`field()` splits on whitespace, `done=` split on `,cut:` untouched, no test pins the line's tail), `scripts/tick-bench.sh` (reads only the ` run 1/1: ` line: `mean=`, `state=`), no python/shell/ps1 script reads the end line, and `serve.rs`' 28 `next_log` callers read the daemon's stderr, which this change does not touch (the end line goes to `--bench` stdout only). `plan`, `they-keep-coming` and the 256² file end ` foraged=0 food=0`.
- Fixture `one-forage.json`: DEFAULT_SEED, at tick 5 a one-cell pile at (65,64,9) and a forage area over the bushes (62,58,10)-(63,58,10), 360 ticks (320 is too few: the berries are stored only by t340). It reads `foraged=2 food=1`. Test `the_end_line_counts_forages_and_stored_berries`. RED: foraged increment commented out, the test fails on `foraged=0`.
- tui test `the_forage_capture_shows_a_ripe_bush_turn_growing_and_its_berries_appear` uses the file's stub server, not simd (`CARGO_BIN_EXE_simd` is not exported to the tui package, the file header says so). `--frame` reads only the connect snapshot, so two connections stand for before and after a forage; the level is the plant's own. RED: growing bushes drawn with the ripe cell, the test fails (`ripe=3 of 2 growing=0 of 1`). Area cells under a bush read `0 of N` drawn, because the bush paints over them.
- The gui `--drag forage` RED was skipped: handled by the orchestrator.

### File List

- `crates/sim-core/src/lib.rs`, `src/worldgen.rs`, `tests/worldgen.rs`, `tests/scenario.rs`
- `crates/simd/src/bridge.rs`, `src/main.rs` (TODO arms only)
- Task 5: `crates/protocol/src/lib.rs`; `crates/simd/src/bridge.rs`, `src/main.rs`, `tests/serve.rs`, `tests/bench.rs`; `crates/client-core/src/lib.rs`, `tests/mirror.rs`; empty `plants`/`forage` added to Snapshot/Delta literals in gui `src/{capture,ingest,pick,project}.rs`, `tests/{capture,headless}.rs` and tui `src/view.rs`, `tests/client.rs`; TODO arms in tui `src/view.rs` and gui `src/project.rs`
- Task 4: `crates/sim-core/src/save.rs`, `src/lib.rs` (`to_save`, `from_save`), `tests/save_load.rs`; `crates/simd/src/main.rs` (validator), `tests/serve.rs`; `scenarios/saves/*.save` (3, data patch); mutations `2-4-the-world-endures.sh`, `3-2-the-dig.sh` (re-pointed)
- Task 5 mutations re-pointed: `12-7.sh`, `3-2-the-dig.sh`, `3-3-the-haul-and-the-skeleton-walks.sh`
- `_bmad-output/implementation-artifacts/mutations/{10-9-the-land-reads-natural,12-7,2-4-the-world-endures,3-3-the-haul-and-the-skeleton-walks,5-1-the-world-grows-things-that-glow}.sh` (re-pointed)
- Task 6: tui `src/view.rs`, `src/palette.rs`, `src/main.rs` (the `forage:` stderr line); mutations `3-2-the-dig.sh`, `3-3-the-haul-and-the-skeleton-walks.sh` (re-pointed)
- Task 7: gui `src/{designate,project,appearance,ingest,capture}.rs`, `tests/headless.rs`; `README.md`; mutations `12-7.sh`, `12-10b.sh`, `8-2-designate-with-the-mouse.sh`, `8-3-master-of-time-and-the-skeleton-walks-in-3d.sh` (re-pointed)
- Task 8: sim-core `src/lib.rs` (`stored_berries_count`); simd `src/scenario.rs`, `tests/bench.rs`, `tests/fixtures/one-forage.json`; tui `tests/client.rs`

## Change Log

| date | entry |
|---|---|
| 2026-11-21 | Story created (Opus 5.5) on `story-13-9-forage` off `main` `3c6cf3b7`. Task 0 Q1–Q6 open for Wolf; Q5 holds the forage-mode draft (standing AC 16). Status → ready-for-dev. |
| 2026-11-22 | Task 0 ruled (Wolf): Q1–Q6 all (a), as recommended; no AC amended. Q5's draft is approved as written. Implementer: Sonnet 5.5 subagents, claude-opus-5-5 orchestrates. Status → in-progress. |
| 2026-11-22 | Tasks 1–10 and 12 done: Sonnet 5.5 subagents for Tasks 1–8 (`7aaf1697`, `cb021528`, `544db973`, `c3f626b0`, `54ce2e1b`, `897b3420`), Opus for Tasks 9–12. Mutation row 8 survived and is now pinned (`f80e6e62`). Wolf ruled the AO guard re-baseline to 59/57 for the bushes (`5963e6d3`). Full gate GREEN at `5963e6d3`. The seat sitting (Task 11, AC 15) is open. |
| 2026-11-22 | Seat run 1 (Wolf, `a192ab88`): three findings. Redesign ruled as Amendment 1 (Q7–Q12): haulers forage several bushes a trip and carry one counted berries item to the pile; one-time ripe-only marks like cut replace the forage area; the first bush is the nearest. ACs 3–10, 11 and 13 are amended. Point 3's idling was probed and not reproduced; "not the nearest" was reproduced, and Q9 answers it. Status stays in-progress. |
