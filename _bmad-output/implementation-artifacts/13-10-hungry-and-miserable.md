---
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
baseline_commit: 0465969f
---

# Story 13.10: Hungry and Miserable

Status: in-progress

## Story

As the boss,
I want dwarves to grow hungry and leave their work to eat, and to turn visibly miserable if I let the food run out,
so that the fortress has needs I can see, and neglect has a cost.

## Branch

`story-13-10-hungry-and-miserable`, cut from `main` `0465969f` (13.9's merge as PR #243). Its first commit, `de1940cf`, is 13.9's close-out and goes into this PR, as `b99f881e` did for 13.9.

**Standing ACs that apply:** Epic 13's M3 ACs 1–8 (**one full gate per story, at PR**), plus M4 ACs 10–14 and 16–18.
- AC 13 (needs before trades) applies for the first time.
- AC 15 does not apply: there is no new trade.
- AC 16: the posture is a plain pose, so it goes **straight to the seat** with no draft (epics.md:3231). The tui look and the selection-panel words are in Q6 for Wolf to approve.

**Wire diff (AD-6, snake_case, Q4 (a)):**
- `protocol::Entity` gains `hungry: Option<bool>` and `miserable: Option<bool>`. Both take `#[serde(default, skip_serializing_if = "Option::is_none")]`, like `profession`, so torch and campfire entities stay byte-identical. Every dwarf carries `Some(..)` for both.
- `DwarfJob` gains `Eat { target }`. `target` is the berries item's cell or the bush's cell.
- No new `Snapshot`/`Delta` field and no new command.

**Saved fields:**
- `SaveState.hunger: bool`, the world setting (Q1). No `#[serde(default)]`.
- `SavedDwarf.hunger: u32`. No `#[serde(default)]`.
- `JobKind::Eat { item }` and `JobKind::EatPlant` go in `jobs`.

Every state hash therefore moves (memory: `state-hash-covers-save-encoding`). AC 2 reads the reach with a strip build.

## Task 0: Wolf rules before dev

Each question has a recommendation, and the ACs and Tasks are written for it. A different answer amends the ACs it names.

**Ruled 2026-11-26 (Wolf):** Q1, Q2, Q3, Q5, Q6 (a) ("otherwise I trust your recommendations"). Q4 (a), with the rule widened: "they should find food and eat if there are some available". Q7 (a): "should dwarves just find food if nothing is in storage? Can be also later AI functionality". Together these give one food rule for every hungry dwarf, not just a miserable one: **stored berries first, else the nearest reachable ripe bush with no forage job.** Misery now changes only the work rule. Its one difference is that it gives up its trade job even when no food is found. The ACs, Tasks and Dev Notes below are amended to match. Restocking the pile on their own, with no mark, is filed as #246 (route:undecided).

- **Q1. Hunger is a world setting, like arrivals (Found A).**
  - (a) **Recommended:**
    - `Hunger(bool)` is a saved setting, on in the fresh game (`fresh_world`) and in any scenario whose `start` says `"hunger": true`.
    - It is off for every other `generate*` world, every committed scenario and the three seat saves. With it off, hunger never rises, so no committed 128² five-dwarf scenario or long test changes outcome (standing AC 11).
    - The bench file `two-hundred-on-256.json` turns it on (Q3).
  - (b) Always on, with every committed scenario and long test re-pinned. None of the 128² scenarios holds any food (no forage marks), so every dwarf in them would end up miserable and stop working.
- **Q2. Constants and the rule (standing AC 18).**
  - (a) **Recommended:**
    - `hunger` counts up by 1 per tick while the setting is on, saturating.
    - `EAT_AT = 12_000`: 20 min at 1x, 60 s at fast4x.
    - `MISERABLE_AT = 18_000`: 10 min later at 1x, 30 s later at fast4x.
    - `EAT_WORK_TICKS = 20`. One berry (one unit of an item's `count`) is one meal, and a meal sets hunger to 0.
    - Starting hunger is staggered, so 200 dwarves do not all get hungry on one tick: `start_hunger(seed, id) = fnv(seed, id) % EAT_AT`, the `reaction_delay` hash shape. It draws on no RNG stream, so no other draw shifts. It applies to the embark crew when the setting is turned on, and to each arrival at spawn.
    - **Hungry** means `hunger >= EAT_AT`. A hungry dwarf eats a reachable stored berries item, or else the nearest reachable ripe bush that **has no forage job** (ruled Q4/Q7: it never takes a bush the player marked).
      - With neither, it keeps working and looks again later.
      - A dwarf **carrying** something is not interrupted until his hands are empty, so hunger drops no stone or load on the ground. This applies to a delivery, a clearing set-down, or the end of a forage trip.
    - **Miserable** means `hunger >= MISERABLE_AT`.
      - It holds no trade job and claims none, even with no food found.
      - It eats by the hungry rule above.
      - Eating a bush regrows it as a forage does, for any eater.
      - It clears the moment hunger falls below `MISERABLE_AT`, which is at the meal.
    - The misery rule is one function, `fn miserable(hunger: u32) -> bool`, with a `// NOTE:` that cold (13.13) and tiredness (13.14) join it. There is no needs framework.
  - (b) Other numbers. With (a), no dwarf can be miserable inside the 2,500-tick bench window, because every starting hunger is below `EAT_AT` (Q3).
- **Q3. Food on the bench (FR67, standing AC 14; Found B).**
  - Today's bench forages 32 bushes for `food=31` by t2,500. That is recorded in `13-9-signoff/state-head-7afc35d8.txt` and was re-read live at creation.
  - With Q2 (a), about 200 × 2,500 / 12,000 ≈ 42 dwarves get hungry inside the window, most of them before the first berries are stored.
  - (a) **Recommended:** the bench file gains `"hunger": true` and a larger opening forage order. The dev widens it until the end line reads `eaten>=1 food>=1 miserable=0`, then records the box, the bush count and the reason in `about` and in the doc comment of `the_two_hundred_on_256_scenario_does_real_work` (standing AC 11).
    - If no box near the pile reaches `food>=1`, the run is recorded and Wolf rules. No haul change is made (#235 owns slow hauls).
    - `main`'s `simd` cannot read the new `start` key (`deny_unknown_fields`, `scenario.rs:38`), so the delta against `main` runs on `main`'s copy of the file, as in 13.9.
  - (b) A separate `hungry-on-256.json`, with the bench file left as it is. Every later story's tick-bench then runs two 256² files.
- **Q4. How a dwarf eats (mechanism; AD-12's one claim market).**
  - (a) **Recommended:** eating is a **personal job**.
    - `JobKind::Eat { item }` (a stored berries item) and `JobKind::EatPlant` (`target` is the bush's cell).
    - It is created by a new system and **handed straight to the hungry dwarf**, as a forage trip's Haul job is handed to its hauler (`lib.rs:3160-3176`).
    - It is never in the claim market: `trade` returns `Option<Profession>`, `None` for both Eat kinds, so `can_try` (`:779`) never matches one and `set_profession` (`:4437`) never releases one.
    - It walks, works and completes in `execute_jobs`, with the haul pick-up's work cells for an item and Forage's for a bush. An Eat job nobody holds is retired.
    - One Eat job per item at a time, so a load of 5 feeds its eaters one by one. Others choose another item or wait.
  - (b) A per-dwarf component with its own walking. That duplicates `execute_jobs`' path, step and squeeze rules.
- **Q5. The gui posture (FR65, standing AC 16: plain, straight to the seat).**
  - (a) **Recommended:** a procedural slump on the armature node, the `nudge_dwarf_for_cut` precedent (`project.rs:2810`).
    - A miserable dwarf is drawn pitched forward `SLUMP_PITCH = 20°` about his feet and sunk `SLUMP_SINK = 0.05` cells. Both are plain constants, judged at the seat.
    - It applies over any clip and pops on and off with no ease.
    - The selection panel appends `hungry` or `miserable` after the trade.
    - The gui prints `gui dwarf {id} miserable on|off` on stderr at each change, the `trade_change_lines` precedent (`ingest.rs:1843`).
  - (b) A new `Miserable` clip authored in Blender. That is an art round under the M2-24 hard stop.
- **Q6. The tui look (NFR10; this is the draft for Wolf to approve).**
  - (a) **Recommended:**
    ```
    map:     a miserable dwarf's ☻ in MISERY_COLOUR (110,120,150, a cold grey-blue) instead of his tunic
             (a hungry dwarf keeps his tunic)
    roster:  "Bifur Ironhand  miner  miserable"   /  "...  miner  hungry"   (the word in STATUS_TEXT grey)
    status:  "dwarves 5  hungry 2  miserable 1  ..."   (survives NO_COLOR; the map colour does not)
    ```
  - (b) Wolf edits the draft.
- **Q7. FR67 over a long run (Found C).**
  - Forage marks are one-time (13.9 Q8), so a regrown bush feeds nobody until it is marked again, and a fortress left alone runs out of berries.
  - The bench window does not reach that.
  - (a) **Recommended:** file it as an issue (`route:undecided`). This story changes no mark rule.
  - **Ruled (a), amended:** a hungry dwarf eats wild bushes when storage is empty (Q2, Q4). A regrown bush is unmarked, so it feeds an eater again, and a fortress left alone does not starve while it has reachable bushes. What runs out is only the stored food. Restocking it with no mark is the later AI, filed as #246.
  - (b) Fold it into this story.
- Then **ASK which implementer** (memory: `dev-implementer-is-sonnet-not-codex`).

## Found at creation (2026-11-26, `main` `0465969f`)

- **A. No 128² committed world holds food.**
  - The plan rows (10,006 ticks), full-pile-crowd, big-pile and deep-pit order no forage, and the three seat saves have `plants: []`.
  - The long tests run past any sensible eat threshold: the goal chain finishes at 9,828 (`tests/scenario.rs:5184-5187`), the cage sweep runs 16,000 × 32 (`:3006-3280`), the staircase sweeps run up to 6,000 per level (`:4825`, `:4922`), and arrivals run to 21,000 (`they-keep-coming.json`).
  - With hunger always on, every one of them ends with miserable, idle dwarves (Q1).
- **B. The bench food line:** `bench two-hundred-on-256 end tick=2500 … goals=29/0/0/31 … foraged=32 food=31`. Re-read live at creation in release, at `de1940cf`: `state=b9f67ecc34f14bcc`, equal to the committed `state-head-7afc35d8.txt:23`. The crew mix is miner : hauler : woodcutter = 2 : 2 : 1 (`lib.rs:48-52`).
- **C. Forage marks are used up** (`remove_forage_mark` on completion, `lib.rs:3128`), and a regrown bush is unmarked (13.9 AC 6).
- **D. No item is ever consumed today.**
  - Despawning one is new.
  - Its readers are `split_stored`/`uncarried_stones` (`:536`, `:1893`), `Carrying`, `Clearing.item`, haul jobs, and simd's validator.
  - Items are resent in full on every delta (`protocol :343`), so a despawn reaches both clients as a removal.
- **E. `trade()` is exhaustive** (`:713`) and read by `can_try` (`:780`) and `set_profession` (`:4437`). Making it `Option` is the one change that keeps Eat jobs out of the market (Q4).
- **F. `release_claim`** (`:2011`) is the one let-go funnel. It drops a carried item, resets `home`, and ends an arrival's walk-in (`:2105`). It does not remove the job, so a released trade job stays queued: "retried, never dropped" is already the behaviour.
- **G. `protocol::Entity` has no `Default`.** About 75 struct literals list every field: tui `view.rs` 19, gui `headless.rs` 15, gui `ingest.rs` 9, client-core 7, tui `client.rs` 6, and the rest in protocol, bridge, `project.rs` and `capture.rs`.
  - `Mirror` stores whole entities (`client-core/src/lib.rs:271-294`), so a new `Entity` field reaches it with no code. Its `Delta { .., }` destructure (`:251-262`) matters only for new top-level fields, and there are none.
- **H. The tui's identity override** (`view.rs:453-459`) paints every named dwarf in his tunic colour after every other choice. The misery colour has to win there.
- **I. The bench end line** (`simd/src/scenario.rs:443-455`) is split on `",cut:"` (`bench.rs:200-203`), so new fields go after `food=`.
- **J. The bench's per-tick dwarf join** (`bridge.rs:197-240`) finds each sibling reader's row with `.iter().find`. A `hunger()` reader joins it the same way.

## Acceptance Criteria

1. **Hunger rises and a hungry dwarf eats (FR62, standing AC 13).**
   - With the setting on, each dwarf's hunger rises 1 a tick from its staggered start (Q2).
   - At `EAT_AT`, a dwarf that carries nothing lets go of its trade job, which stays queued and is later done by someone. It walks to a reachable stored berries item, eats for `EAT_WORK_TICKS`, and the item's count falls by 1 (the item is gone at 0). Its hunger is then 0.
   - With no reachable stored berries, it walks instead to the nearest reachable ripe bush with no forage job, eats there for `EAT_WORK_TICKS`, and the bush regrows (`ripe_at = tick + REGROW_TICKS`). Its hunger is then 0. Stored berries win over a nearer bush (ruled Q4/Q7).
   - With neither, it keeps its trade job and keeps working.
   - A dwarf carrying an item is interrupted only once its hands are empty.
2. **Off changes nothing (standing AC 11, FR53).**
   - Every committed scenario run as on `main` (bench file per Q3) has `main`'s end line, apart from the new end-line fields.
   - A throwaway build that drops `SaveState.hunger` and `SavedDwarf.hunger` from the encoding (13.8's strip patch) reads `state` `same` against `main` on all of them.
   - Every existing test passes unchanged.
3. **Misery refuses trade work, never self-care (FR65, standing AC 13).**
   - At `MISERABLE_AT` a dwarf is miserable. It lets go of its trade job (once its hands are empty) and claims none until relieved.
   - It eats by AC 1's rule. With no food, it holds no trade job anyway.
   - A marked bush is never eaten, by a hungry or a miserable dwarf.
   - Its misery clears at the meal.
4. **No food anywhere, then food back (FR65, success criterion 3).**
   - A world with the setting on, no plant and no berries starts from a sim-written save with every dwarf's hunger set to `MISERABLE_AT - 1`, and runs 5,000 ticks. Then: every dwarf is still in the world, the tick has advanced every step with no panic, every dwarf is miserable, and no miserable dwarf holds a trade job. (Stepping to `MISERABLE_AT` from a fresh start is about 18,000 debug ticks, so the save edit replaces them.)
   - Stored berries are then put on the pile, through a sim-written save edited and reloaded. Every dwarf's misery clears within `RECOVERY_BOUND` ticks, a constant the test names from its measured reading plus a margin.
5. **Eat jobs are personal (mechanism AC, Q4: AD-12's one market).**
   - No dwarf ever claims another's Eat job.
   - `set_profession` never releases an Eat job.
   - An Eat job nobody holds is gone the next tick.
   - Two hungry dwarves and one item of count 1: one eats it, and the other does not walk on to a vanished item.
6. **Determinism and save (NFR9, AD-11, standing AC 12).**
   - Same seed and commands give identical hunger every tick, with the setting on.
   - A world saved mid-meal (a dwarf holding an Eat job with `WorkProgress > 0`, another miserable), loaded and stepped past the meal, equals the never-saved world, the `hunger()` reader included.
   - A save missing `hunger` (either key) is refused, with a positive control.
   - The three seat saves are patched as data (`"hunger": false`, `"hunger": 0` per dwarf), and their scenarios keep their end lines.
   - `simd`'s load validator refuses, each naming the cause:
     - an Eat job nobody holds;
     - an Eat job whose item is not a stored berries item;
     - an `EatPlant` whose target holds no ripe plant, or holds a forage job;
     - two Eat jobs on one item, or two `EatPlant` on one bush;
     - a dwarf with hunger above 0 on a world with the setting off.
7. **Both clients show it (NFR10).**
   - Each dwarf `Entity` carries `hungry` and `miserable`. The flags change on the threshold ticks only, not every tick.
   - The tui draws Q6's look, with no TUI regression and no TUI input.
   - The gui draws Q5's slump on a miserable dwarf only, and shows the selection-panel word.
8. **The bench (FR67, standing ACs 10 and 14).**
   - `two-hundred-on-256` (Q3) ends with `eaten=E food=F miserable=0`, where `E >= 1` and `F >= 1`, and no order is given after its opening commands. The `#[ignore]` release test asserts all three.
   - `miserable=0` holds by Q2's numbers, not by tuning. The discriminating readings are `eaten` and `food`.
9. **The bench delta (standing AC 10).** `RUNS=5 scripts/tick-bench.sh main HEAD` runs over the committed files, with `main`'s copy of the bench file. Two HEAD-only rows give the need's cost at 200: HEAD's file, and HEAD's file with `"hunger": false`, both on HEAD's build.
10. **The instruments are tested (standing AC 8).**
    - The end line's `eaten=` reads `0` on a sim-written save with hunger off and `>= 1` on the same save with one dwarf's hunger edited to `EAT_AT - 5` and a stored berries item, driven through the binary.
    - The tui status count follows a dwarf turning miserable in a capture.
    - The gui headless test shows the slump appear and go with the mirror flag.
11. **The mutation table kills** (Task 9), and `python3 scripts/audit-mutations.py` exits 0.
12. **The seat (standing AC 16, 17; success criterion 3, hunger half).**
    - In a fresh game with no forage marks, Wolf sees hungry dwarves leave their work, eat at unmarked bushes and go back to work.
    - He then marks bushes and sees stored berries eaten at the pile.
    - On the no-food save (Task 10), he sees dwarves turn miserable, slump and stop work.
    - His words on the plain slump and the tui look are recorded.
    - The gui frame rate is recorded beside 13.9's seat reading (NFR13).

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings (Q1–Q7).** Ruled 2026-11-26: see the Ruled line under Task 0 and the Change Log. #246 filed. Then ASK which implementer.
- [ ] **Task 1: hunger state and the setting (AC 1, 2, 6).** UPDATE `crates/sim-core/src/lib.rs`, `src/save.rs`.
  - [ ] Constants beside `FORAGE_*` (`lib.rs:97-104`): `EAT_AT`, `MISERABLE_AT`, `EAT_WORK_TICKS`.
  - [ ] NEW `#[derive(Component)] struct Hunger(u32)` on every dwarf, at all three spawn sites: `spawn_dwarf` (`:4708`), `from_save`'s spawn (`:3970-3987`) and nothing else (`arrive` uses `spawn_dwarf`). It is a required component, not optional: see the `Carrying` NOTE (`:410-413`).
  - [ ] NEW resource `struct HungerOn(bool)` beside `Arrivals` (`:1011-1014`), passed through `assemble` (`:3636`). NEW `pub fn with_hunger(mut self) -> World`: it sets the resource and gives each dwarf `start_hunger(seed, id)`. `arrive` gives each arrival its start hunger when the setting is on.
  - [ ] NEW `fn start_hunger(seed: u64, dwarf: Id) -> u32`, shaped like `reaction_delay` (`:695`), and `fn miserable(hunger: u32) -> bool` with the cold/tiredness `// NOTE:`.
  - [ ] Readers: `pub fn hunger(&self) -> Vec<(Id, u32)>`, `pub fn hungry(&self) -> Vec<(Id, bool)>` and `pub fn miserable(&self) -> Vec<(Id, bool)>`, sorted by `Id` like `professions()` (`:4558`). `pub fn hunger_on(&self) -> bool`.
  - [ ] Save: `SaveState.hunger: bool` and `SavedDwarf.hunger: u32`, with no `#[serde(default)]` (`save.rs:32-34` NOTE shape). Update `to_save` (`:3841`) and `from_save`, and the `SavedDwarf { .. }` literals (`tests/save_load.rs:547,567,680,706,966`; `tests/scenario.rs:1624,1644,4626`).
- [ ] **Task 2: eating and misery (AC 1, 3, 4, 5).** UPDATE `lib.rs`.
  - [ ] `JobKind::Eat { item: u32 }` and `JobKind::EatPlant` (`:383`). `trade` becomes `fn trade(kind: JobKind) -> Option<Profession>`, `None` for both; update `can_try` (`:780`) and `set_profession` (`:4437`).
  - [ ] Extend every exhaustive site:
    - `Jobs::insert`/`remove` (`:453-486`): `EatPlant` shares `targets`; `Eat` is unique by item through a NEW `eat_items` set.
    - `work_positions_with` (`:1795`): `Eat` uses the haul pick-up's cells for the item, or empty if it is not stored; `EatPlant` uses Forage's side neighbours.
    - The work-time match (`:2987-2992`).
    - `JobKind` matches in simd.
  - [ ] NEW exclusive system `tend_hunger`, chained after `advance_tick`, which skips while paused (AD-2) and does nothing with the setting off:
    1. Retire every Eat job nobody holds.
    2. Add 1 to every dwarf's hunger, saturating.
    3. For each dwarf, ascending `Id`, that is hungry, holds no Eat job and carries nothing, on its look tick (`(tick + id) % RETRY_COOLDOWN == 0`, stateless): run ONE search to the union of the work cells of the stored berries items with no Eat job. If that fails, run one search to the side cells of ripe bushes with no job, the forage turn's union-search shape (`:848-883`). This applies to every hungry dwarf, miserable or not (ruled Q4/Q7).
    4. On a hit, `release_claim` its trade job, create the Eat job (`next_job_id`) and hand it over with its `Path`.
    - `// NOTE:` the search cost: two floods per hungry dwarf per look when no food is reachable. AC 9 measures it.
  - [ ] Misery in the claim: `can_try` is false for a miserable dwarf. `tend_hunger` releases a miserable dwarf's trade job once its hands are empty, whether or not food is found.
  - [ ] Completion in `execute_jobs`, beside Forage (`:3108`), dispatched before the terrain-change arm:
    - `Eat`: if the item is still a stored Berries item, take 1 from its `ItemCount` and despawn it at 0, setting `stale = true`. Then hunger goes to 0.
    - `EatPlant`: if the bush is ripe and has no other job, set `ripe_at = tick + REGROW_TICKS`, mark the plant `changed`, and set hunger to 0.
    - In both: remove the job and `release_claim`.
  - [ ] Scenario tests in `tests/scenario.rs` for AC 1, 3, 4, 5. Build the hungry state from a sim-written save with edited hunger (`to_save` → edit → `from_save`), never by stepping 12,000 ticks in debug. Find a pile cell and a bush from the world's readers; never hardcode them.
- [ ] **Task 3: the load validator (AC 6).** UPDATE `crates/simd/src/main.rs`, `tests/serve.rs`.
  - [ ] In `try_load_world_from` (`:594`), Eat arms beside the Forage arm (`:766`): the AC 6 refusals. `EatPlant` shares `seen_job_targets`. Both are kept out of the tile-job and haul counts (`:701-704`). The per-dwarf loop (`:878`) gets the hunger-with-setting-off check.
  - [ ] Refusal tests through a real daemon Load, each with a positive control written by the sim (13.9 review's lesson: prove it with a save the sim wrote).
  - [ ] Patch `scenarios/saves/*.save` per AC 6 with a throwaway script, not committed.
- [ ] **Task 4: the wire (AC 7).** UPDATE `crates/protocol/src/lib.rs`, `crates/simd/src/bridge.rs`, `scenario.rs`, `main.rs`.
  - [ ] `Entity.hungry`/`.miserable` and `DwarfJob::Eat { target }`, pinned in the wire-literal tests (`protocol :487`, `:512`, `:761`). Torches and the campfire stay byte-identical (the `:487` test).
  - [ ] `dwarf_entities` (`bridge.rs:197-240`) joins `hungry()`/`miserable()`. `emitter_entity` (`:181`) gets `None`. `dwarf_job` (`:242`) maps both Eat kinds. Add the first per-dwarf-field bridge test.
  - [ ] Scenario `StartFile.hunger: Option<bool>` (`scenario.rs:38-47`). A save start with `hunger` is refused, as `arrivals` is (`:150-155`). `generate_world` (`main.rs:111`) takes it, and `fresh_world` (`:134`) turns it on.
  - [ ] Every `Entity { .. }` literal (Found G).
- [ ] **Task 5: the tui (AC 7).** UPDATE `crates/tui/src/view.rs`, `palette.rs`.
  - [ ] `MISERY_COLOUR` in `palette.rs`, pinned in `every_look_is_pinned` (`:326`). The identity override (`view.rs:453-459`, Found H) uses it for a miserable dwarf.
  - [ ] The roster word (`:537-570`) and the status counts (`:488-519`), per Q6.
  - [ ] Tests beside `a_named_dwarf_is_drawn_in_his_tunic_colour…` (`:1391`) and `status_line_reports_speed_z_and_dwarf_count` (`:2075`).
- [ ] **Task 6: the gui (AC 7).** UPDATE `crates/gui/src/project.rs`, `ingest.rs`.
  - [ ] NEW system `slump_miserable_dwarf`, modelled on `nudge_dwarf_for_cut` (`project.rs:2810`) and composed with it so neither overwrites the other's armature offset. Register it after `nudge_dwarf_for_cut` in the chain (`ingest.rs:797-807`). It reads `miserable` from the mirror entity.
  - [ ] The selection word in `name_readout_spans` (`ingest.rs:1880-1932`), and the stderr `miserable on|off` line beside `report_trade_changes` (`:1843-1878`).
  - [ ] Headless tests in `gui/tests/headless.rs`, modelled on `a_cutting_dwarfs_armature_is_nudged_toward_the_trunk_and_his_entity_is_not` (`:6872`): the slump appears and goes with the flag, the entity transform is untouched, and a cutting miserable dwarf keeps both offsets.
- [ ] **Task 7: the instruments (AC 8, 10).**
  - [ ] The end line (`scenario.rs:443-455`) appends ` eaten=E hungry=H miserable=M` after `food=` (Found I). `eaten` counts completed meals (Eat and EatPlant). Check every end-line and `next_log` reader before changing the line (memory: `new-log-line-breaks-next-line-readers`).
  - [ ] A test in `simd/tests/bench.rs`: a save written by `sim_core` in the test with hunger off reads `eaten=0`. The same save with the setting on, one dwarf's hunger set to `EAT_AT - 5` and a stored berries item reads `eaten>=1`.
  - [ ] A tui test in `tui/tests/client.rs`, on the file's stub server: the status line's `miserable` count goes from 0 to 1 when one entity's `miserable` turns `Some(true)`.
  - [ ] **RED first, for each instrument.** Restore each step with an edit, never `git checkout --` (memory: `sabotage-restore-trap`).
    - Comment out the `eaten` increment: the bench test must fail on `eaten=0`.
    - Make the status line ignore `miserable`: the tui test must fail.
    - Leave `slump_miserable_dwarf` unregistered: the headless test must fail.
- [ ] **Task 8: the bench file and the delta (AC 2, 8, 9).**
  - [ ] Per Q3, set `"hunger": true` in `two-hundred-on-256.json` and widen the opening forage order with a throwaway script until AC 8 holds. Record the box, the bush count and the reason in `about`, and add a re-pin line to the `bench.rs:294-307` doc comment, plus the AC 8 asserts.
  - [ ] AC 2: run the end lines on `main` and HEAD, saved to `13-10-signoff/state-{main,head}-<sha>.txt`. Run the strip build in a scratch worktree (memory: `mutation-run-poisons-concurrent-builds`).
  - [ ] AC 9: `RUNS=5 scripts/tick-bench.sh main HEAD`, saved to `13-10-signoff/tick-bench-main-<sha>.txt`, plus the two HEAD-only rows. Record the delta table in the Debug Log, in 13.9's form.
- [ ] **Task 9: the mutation table (AC 11).** NEW `_bmad-output/implementation-artifacts/mutations/13-10.sh`, with count-guarded anchors, run saved to `13-10-signoff/`. At least one row each for:
  - the hunger increment;
  - the `HungerOn` gate;
  - `start_hunger`'s stagger;
  - the `EAT_AT` comparison;
  - the empty-hands condition;
  - `can_try`'s misery refusal;
  - the marked-bush exclusion;
  - the unheld-Eat retire;
  - `trade(Eat) = None`;
  - the count decrement;
  - the despawn at 0;
  - hunger reset at the meal;
  - each validator arm;
  - the save keys;
  - the bridge's `miserable` join;
  - the tui misery colour;
  - the gui slump registration.

  Run `audit-mutations.py` before the first code commit (memory: `deleting-code-retires-rows-in-the-same-commit`). Rows on `trade`'s match (13-9 row 2, 12-7 row 4) will need re-pointing.
- [ ] **Task 10: the seat card (AC 12).** NEW `13-10-signoff/vehicle-card.md` in 13.9's skeleton:
  - the run in the seat's form (WSL `simd 7451`; PowerShell `launch-gui.ps1` with `--perf-log` to an absolute `D:\Workspace\frostvein\.bin\` path);
  - what to do at fast4x in a fresh game: mark nothing, and watch hungry dwarves eat at unmarked bushes and go back to work (the tui `hungry` count). Then mark bushes and watch stored berries being eaten at the pile;
  - the misery half: NEW committed `scenarios/saves/no-food.save`, a copy of a seat save (`plants: []`) with `"hunger": true` and each dwarf's hunger a few hundred ticks below `MISERABLE_AT`, made by a throwaway script from a sim-written save. Watch dwarves turn miserable, slump and stop work (the tui `miserable` count);
  - one question each on the slump and the tui look;
  - the sitting in Wolf's words.
- [ ] **Task 11: the gate.** `RUST_TEST_THREADS=1 scripts/gate.sh`, once, at PR (memory: `gate-ooms-at-default-parallelism`).

## Verification

Only the baseline can run at creation, and it was run (Found B). Each line below states what the dev must produce: the exact command, the non-zero observation, and the RED to see first.

```bash
# Baseline, run at creation (release, de1940cf):
target/release/simd --bench scenarios/two-hundred-on-256.json --runs 1 | rg ' end '
#   got: ... goals=29/0/0/31 done=dig:287,cut:96 stored=66 cells=2304 crew=200 arriving=0 foraged=32 food=31

# AC 1-6: sim
cargo test -p sim-core
#   expect: the new hunger tests pass and every existing test passes unchanged (AC 2)

# AC 2, 8: end lines, main vs HEAD (release, every committed scenario; the bench row on main's copy)
for f in $(git ls-files 'scenarios/*.json'); do target/release/simd --bench "$f" --runs 1 | rg ' end '; done
#   expect HEAD: main's line plus ' eaten=0 hungry=0 miserable=0'
#   expect HEAD two-hundred-on-256 (hunger on): eaten>=1 food>=1 miserable=0
#   RED: comment out the eaten increment -> eaten=0 and the bench.rs instrument test fails; restore by edit

# AC 9: bench delta
RUNS=5 scripts/tick-bench.sh main HEAD 2>&1 | tee _bmad-output/implementation-artifacts/13-10-signoff/tick-bench-main-$(git rev-parse --short=8 HEAD).txt

# AC 7, 10: tui (status count survives NO_COLOR; read the level from the test's dwarf, never assume)
cargo test -p tui --test client miserable
#   RED: the status line ignores `miserable` -> the test fails

# AC 7, 10: gui slump (headless, devpod lavapipe)
cargo test -p gui --test headless slump
#   RED: slump_miserable_dwarf left unregistered -> the test fails

# AC 11
env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-10.sh
python3 scripts/audit-mutations.py

# gate (once, at PR)
RUST_TEST_THREADS=1 scripts/gate.sh
```

The seat run lives on the vehicle card (Task 10), in the seat's form: WSL `simd 7451`, then PowerShell `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-10-seat.csv')`.

## Dev Notes

### Scope guardrails (do NOT)
- No cold, sleep, meals or cooking (13.13, 13.14, 13.20). No moods beyond misery, and no death (FR65).
- No change to forage marks or regrowth (Q7), and no haul change, even if `food=0` (#235).
- No needs framework, trait or event bus: one `Hunger` component and one `miserable()` function.
- No art round. The slump is a plain transform (memory: `art-gates-visual-judgement`).
- No TUI input (NFR10). No #230 crew-list work: the roster stays what it is.
- No change to `claim_jobs`' order (AD-12): Eat jobs never enter it.

### What already exists (build on it)
- **The forage trip's handed-over Haul job** (`lib.rs:3160-3176`) is the template for handing a job straight to one dwarf.
- **The forage turn's one union search** (`:848-883`) is the template for "nearest food by path".
- **`release_claim`** already keeps the released job queued and resets `home` (Found F).
- **13.8's `Arrivals`** is the template for the setting, end to end: the resource, `assemble`, `generate_with_arrivals`, `StartFile.arrivals`, `generate_world`, `fresh_world`, the save key, and the strip-build proof (`13-8-signoff/prototypes/strip-new-fields.patch`).
- **`nudge_dwarf_for_cut`** (`project.rs:2810`) is the armature-offset template.

### Key decisions & traps
- **Hunger is a counter, misery is derived.** Only `hunger` is saved, and both wire flags are computed from it in sim-core: clients hold no threshold (ADR 4).
- **The look tick is stateless** (`(tick + id) % RETRY_COOLDOWN`), so nothing new is saved for it.
- **A hungry dwarf with empty hands and no reachable food keeps its trade job.** Only misery takes the job without food.
- **Stored berries win over a nearer bush** (ruled Q4/Q7), so marking bushes keeps eaters near the pile. The two searches run in that order, not as one union.
- **Eating an item despawns it at count 0.** Set `stale = true` in `execute_jobs` (`:2704`), or the kept stone map goes wrong and its debug asserts (`:2748-2768`) fire.
- **No eater takes a marked bush.** Its job sits in `Jobs.targets`, so `EatPlant` uniqueness enforces this too.
- **Pixel guards may move in the fresh game.** Starting hungers are staggered below `EAT_AT`, so some dwarves turn hungry within the first ticks and walk to a bush. Nobody is miserable before tick 6,000. A guard that captures a fresh game with a frame count can therefore see a dwarf elsewhere. Attribute any moved guard to this before re-baselining (13.9 Task 12).
- **`dwarf_clip` has no Eat arm** (`project.rs:484-500`), so an eating dwarf holds the Walk clip, standing still. That is plain and intended; no eat animation (no art round).
- **The cage sweep hardcodes dwarves 0..5 and `STILL_BOUND = 500`** (`scenario.rs:3199-3207`). The setting is off there, so it must not move.

### Previous story intelligence
- 13.9 review: a validator arm must be proven with a save **the sim wrote** in that state, through simd's live Load. A hand-built save once passed while a sim-written one was refused (#244).
- 13.9: `two-hundred-on-256`'s 54 bushes on pile cells count as stored with no haul. The forage box widened for Q3 must still avoid the pile and the dig and cut marks.
- Run the gate with `RUST_TEST_THREADS=1` (45–65 min). Build probes in a scratch worktree, never in the shared tree (memories: `probe-sabotage-leaks-into-wolfs-build`, `cargo-target-dir-builds-cwd-sources`).

### Project Structure Notes
- UPDATE:
  - sim-core: `crates/sim-core/src/lib.rs`, `src/save.rs`, `tests/scenario.rs`, `tests/save_load.rs`
  - protocol: `crates/protocol/src/lib.rs`
  - simd: `crates/simd/src/bridge.rs`, `src/main.rs`, `src/scenario.rs`, `tests/bench.rs`, `tests/serve.rs`
  - client-core: `crates/client-core/src/lib.rs` (entity literals only; a shared `need_text` beside `profession_text` `:80` if both clients spell the words)
  - tui: `crates/tui/src/view.rs`, `src/palette.rs`, `tests/client.rs`
  - gui: `crates/gui/src/project.rs`, `src/ingest.rs`, `tests/headless.rs`, plus the `Entity` literals in `src/capture.rs` and `tests/capture.rs`
  - data: `scenarios/two-hundred-on-256.json`; `scenarios/saves/*.save` (data patch); NEW `scenarios/saves/no-food.save` (Task 10, seat only)
- NEW:
  - `_bmad-output/implementation-artifacts/mutations/13-10.sh`
  - `_bmad-output/implementation-artifacts/13-10-signoff/` (vehicle card, state lines, tick-bench, mutate runs)
- No new crate, no new dependency, no new crate edge.

### References
- Story 13.10 and M4 standing ACs: `_bmad-output/planning-artifacts/epics.md:3213-3240`, `:2950-2962`. M3 standing ACs: `:2557-2570`.
- FR62, FR65, FR67: `epics.md:156`, `:162`, `:167`. M4 PRD: `prds/prd-frostvein-2026-10-28/prd.md:141-169` (FRs), `:211` (NFRs), `:214-226` (success criterion 3).
- 13.8 (setting pattern, strip build): `13-8-they-keep-coming.md:37-38`, `:183-189`, `:441`. 13.9 (forage, bench file, review lessons): `13-9-forage.md`.
- Issues: #224 (gui fps), #230 (crew list), #235 (slow haul), #245 (dig fps).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-11-26 | Created (claude-opus-5-5) from epics.md 13.10, on `main` `0465969f`. Baseline bench line re-read live (Found B). Task 0 Q1–Q7 open for Wolf. |
| 2026-11-26 | Task 0 ruled by Wolf. Q1, Q2, Q3, Q5, Q6 (a). Q4 (a), widened: "they should find food and eat if there are some available". Q7 (a): "should dwarves just find food if nothing is in storage? Can be also later AI functionality". One food rule now covers every hungry dwarf: stored berries first, else the nearest unmarked ripe bush. Amended Q2, Q7, AC 1, 3, 6 (two `EatPlant` on one bush), 12, Tasks 2 and 10 (NEW `no-food.save` for the seat's misery half), and the Dev Notes. Self-restocking filed as #246 (route:undecided). |
