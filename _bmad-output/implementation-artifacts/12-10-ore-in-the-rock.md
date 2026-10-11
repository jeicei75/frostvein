---
baseline_commit: 32565fd
model: claude-opus-5-5  # session default, same as 12.1-12.9's creation
---

# Story 12.10: Ore in the Rock

Status: in-progress

## Story

As the boss,
I want my miners to find minerals and gems in the mountain,
so that digging deeper is worth it.

## Not stacked: branch off `main`

`main` is `32565fd` (PR #190 merged), clean. Branch `story-12-10-ore-in-the-rock` carries this file,
`12-10-signoff/` and the board edit. **Its first commit flips 12.9 to `done` in its story file**
(`12-9-one-dwarf-per-tile.md:8` still reads `in-progress`; the board already reads `done`). Epic 12's nine
standing ACs (`epics.md:2422-2434`) bind this story and are not restated. "Wire diff" below satisfies
standing AC 4. Standing AC 7: Wolf approves `12-10-signoff/draft.md` at Task 0, before any gui or tui
display is built.

## Found at creation (2026-10-11, on `32565fd`)

- **No miner can reach anything below the surface, on any generated world.** This is the story's premise
  defect.
  - Dig marks only `Tile::Solid(_)` (`sim-core/src/lib.rs:2326-2329`) and completes only on `Solid`
    (`:1639-1641`).
  - `place_ramps` (`worldgen.rs:233-261`) turns the surface of **every column with a higher neighbour** into
    a `Ramp`, so every hill is ringed by undiggable ramps.
  - Digging is sideways only: dig work positions are the walkable same-z side neighbours
    (`work_positions`, `lib.rs:1080-1083`), and a channel opens no way down (#163).
  - Measured on DEFAULT_SEED by a fewest-digs search (walk by `astar_neighbours`' rules, dig at the
    digger's level): every reachable diggable cell is at depth 0 (`[(0, 10569)]`). With ramps diggable the
    same search reaches depth 3 in 4 digs and depth 21 in 27, and 2,877 depth-≥3 cells lie within 20 tiles
    (Manhattan) of the camp. Ore "inside the rock" is unreachable unless the dig rule changes (Task 0.2).
- **A fresh world exposes no rock below the soil.** Steps between columns are at most 1 (DEFAULT_SEED step
  histogram `{0: 25000, 1: 7512}`), and no cell with `z + 2 < height` has an empty neighbour (0 of ~250k).
  So ore placed there is invisible until it is dug or sliced, and it cannot move any exposed-face count:
  the bench pins (`resolution_bench.py:21-22`, `test_resolution_bench.py:261`) stay as they are.
- **Every dig yields `ItemKind::Stone`.** `execute_jobs` throws the material away (`Some(Tile::Solid(_))`,
  `lib.rs:1640`) and spawns `Item(ItemKind::Stone)` unconditionally (`:1695`). Snow, ice and soil also
  yield stone today; that stays.
- **A channel keeps the material.** It turns `Solid(m)` under the target into `Ramp(m)` (`:1643-1651`) and
  spawns a stone on the target. Over ore it would leave `Ramp(ore)` and yield stone (Task 0.3).
- **Hauling is kind-blind.** `uncarried_stones` (`:1139-1154`), `create_haul_jobs` (`:362-426`),
  `pile_targets`/`drop_cell` (`:732`, `:766`) never read the kind. Ore hauls with no new haul code.
- **sim-core has no exhaustive match on `Material` or `ItemKind`.** It compiles with new variants. The
  exhaustive matches are `simd/src/bridge.rs:109-114, 124-133`, the clients (below), and the tests
  `sim-core/tests/worldgen.rs:458-472` and `simd/src/bridge.rs:347-356`.
- **The gui preview mirrors the dig rule.** `sim_will_keep` keeps `Tile::Solid(_)` for dig
  (`gui/src/project.rs:1067-1076`). It must change with the sim, or ramp cells light as "not kept".
- **Saves need no change.** `Material` has no `rename_all` (writes `"Tetrahedrite"`), and new variants are
  additive, so a pre-12.10 save decodes and loads ore-less (`from_save` never regenerates). There is no
  version field; 12.7 and 12.9 refused old saves only because a field went missing (`save.rs:23-48`).
- **No instrument counts materials or items.** simd takes only `[port] [--pause-at TICK]`
  (`main.rs:63-98`). `12-10-signoff/ore_wire.py` is written at creation (Verification).
- **Live RED** (`ore_wire.py`, fresh release `simd`, DEFAULT_SEED): `census: no ore kinds …`, `ore kinds in
  the rock: 0`, `ORE WIRE RED`, exit 1. **Its CONTROL** (`--control`, plain interior stone) is RED on
  `main` (`stone: target dug NEVER …`) and **GREEN on a scratch build with ramps diggable** (two lines,
  `lib.rs:1640` and `:2328`): `plan: stone target [62, 77, 12] (depth 3) via 4 digs from [62, 66, 9]`,
  `target dug tick 933; stone item on it tick 933; on a pile cell tick 1303`, `ORE WIRE OK (CONTROL)`.

## Wire diff (standing AC 4)

- `protocol::Material` (`protocol/src/lib.rs:25-32`) gains `Tetrahedrite`, `Bloodstone`, giving `"tetrahedrite"` and
  `"bloodstone"`. `Tile` is unchanged: `{"solid":"tetrahedrite"}`, and `{"ramp":"tetrahedrite"}` exists on the wire but no
  rule produces it (Task 0.3).
- `protocol::ItemKind` (`:258-263`) gains `Tetrahedrite`, `Bloodstone`, with the **same wire names as the material**:
  `{"id":12,"pos":[1,2,3],"kind":"bloodstone"}`. `ore_wire.py` relies on that equality.
- `Item`, `Entity`, `Snapshot`, `Delta`, `Command`, `Refusal` keep their shape.
- **Save:** the same two variants in sim-core's `Material` and `ItemKind`, through the existing derives. A
  pre-12.10 save loads (Task 0.4).

## Acceptance Criteria

1. The real daemon is the judge: on DEFAULT_SEED, `ore_wire.py` reports 2 ore kinds, none exposed. It digs
   to the nearest cell of each, gets an item of that kind on the cell, and sees it reach a pile:
   `ORE WIRE OK`.
2. Worldgen places one mineral and one gem (Task 0.1 names) only in `Solid(Stone)` cells with
   `z + 2 < height`, from its own purpose-named stream. Gems are rarer (at most 1 per 10 mineral cells) and
   only at depth ≥ `GEM_MIN_DEPTH` (Task 0.5). For DEFAULT_SEED and seed 42, heights, ramps, trees, camp and
   dwarf spawn are unchanged from `32565fd`.
3. A dig mark lands on a `Ramp` as on a `Solid` non-tree tile. A dug ramp becomes `Empty` and yields its
   material's item, like a solid (Task 0.2).
4. Digging a mineral or gem cell leaves `Empty` and one item of that kind on the cell. Digging any other
   non-tree material still yields `stone`. A channel whose ramped cell is ore yields that ore's item and
   leaves `Ramp(Stone)` (Task 0.3).
5. An ore item is hauled to a stockpile like stone, with no haul change.
6. Determinism (standing AC 3): same seed gives identical ore, and a different seed gives different ore. A
   world saved with ore in the rock and ore items on the ground and in transit loads and steps identically.
   A pre-12.10 save loads (Task 0.4).
7. gui (`draft.md`): ore cells draw in their own colours on cut faces and on exposed walls. An ore item is
   the stone cube in its ore's colour. The dig preview lights ramp cells.
8. TUI (NFR10): ore cells and ore items draw with their own glyphs and colours (`draft.md` §1). A stored ore
   item keeps its glyph in zone green. No new input, no regression.
9. Instrument: `ore_wire.py` GREEN, plus a `serve.rs` test that the snapshot carries both ore names. The
   deliberate RED is mutation row 1 (dig yields stone whatever the material): `FIRST ITEM THERE WAS THE WRONG
   KIND` and `ORE WIRE RED`.
10. At the seat, Wolf slices into a hill and sees veins and a bloodstone, digs a tunnel to a vein, and watches
    the ore come out and reach the pile. He sees the same in an attached tui.

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings, 2026-10-11, at creation.**
  1. **Look draft APPROVED** (`12-10-signoff/draft.md`), with option (b)'s colours under Dwarf Fortress
     names: the mineral is **tetrahedrite** (40,64,176) and the gem is **bloodstone** (120,196,176). The
     names are flavour, the colours are (b)'s: Wolf kept them knowing real tetrahedrite is steel-grey and
     bloodstone dark green. One mineral, not two. The blue-at-or-above-red rule stands.
  2. **Dig takes ramps: all diggable ground digs the same way, hillsides included.** #163 (channel down a
     level) is NOT folded in. It becomes its own story, **12.10b**, right after this one (board and
     `epics.md` updated at creation).
  3. **Channel over ore** yields the ore item and leaves `Ramp(Stone)`, so ore never yields twice and
     `Ramp(ore)` is never produced.
  4. **A pre-12.10 save loads, ore-less, with no refusal.** There is no version field.
  5. **Placement starting point** (not changed by Wolf; tuned in Task 2, census recorded):
     - tetrahedrite in random-walk veins, about 2% of interior stone, veins of 4-12 cells;
     - bloodstone as single cells at depth ≥ 5 (`GEM_MIN_DEPTH`), about 1 per 20 tetrahedrite cells.

- [ ] **Task 1: protocol** (AC 1, 4)
  - [ ] `Material` += `Tetrahedrite`, `Bloodstone`; `ItemKind` += `Tetrahedrite`, `Bloodstone` (snake_case, wire diff above).
  - [ ] Extend the literal pins: the material list in `every_material_and_tile_variant_has_a_pinned_wire_name`
        (`protocol/src/lib.rs:620-631`), and an ore item JSON pin beside `:732-752`.

- [ ] **Task 2: sim-core** (AC 2-6). RED first: write 2.1's tests and record each failing on the
      unchanged rule before 2.2-2.4.
  - [ ] 2.1 Scenario tests (`tests/scenario.rs`, fixture style of `dig_one_stone` `:1327-1358`, always with a
        pile, because 12.9 items block):
        - `digging_ore_yields_an_ore_item_that_reaches_the_pile`, one tetrahedrite and one bloodstone cell;
        - `a_dig_mark_takes_a_ramp_and_the_ramp_is_dug_out`;
        - `channelling_over_ore_yields_the_ore_and_leaves_a_stone_ramp`.

        Use `kinds_at` (`:2396-2403`) for kinds.
  - [ ] 2.2 `Material` (`lib.rs:55-63`) += `Tetrahedrite`, `Bloodstone`; `ItemKind` (`:131-135`) += same.
        `fn yield_of(material: Material) -> ItemKind`: ore maps to its own kind, everything else to `Stone`.
  - [ ] 2.3 Dig filter (`:2326-2329`) keeps `Solid(_) | Ramp(_)` (still no tree). The completion arm (`:1640`)
        takes `Solid(m) | Ramp(m)` and returns `m`. Capture the material **before** `set_tile` (`:1688`),
        and spawn `Item(yield_of(m))` at `:1695`. Channel (`:1643-1651`): if `m` is ore, produce
        `Ramp(Stone)` and spawn `yield_of(m)`.
  - [ ] 2.4 `const STREAM_ORE: u64 = 0x4f52_455f_5f5f_5f5f; // "ORE_____"` beside `lib.rs:27-32`.
        `pub(crate) fn place_ore(dims, heights, tiles, rng)` in `worldgen.rs`, called in `generate` after
        `place_ramps` with `ChaCha8Rng::seed_from_u64(seed ^ STREAM_ORE)`. It replaces only
        `Solid(Stone)` with `z + 2 < height`. Constants are hardcoded (`VEIN_COUNT`, `VEIN_LEN`,
        `GEM_COUNT`, `GEM_MIN_DEPTH`).
  - [ ] 2.5 `tests/worldgen.rs`:
        - Extend the exhaustive match `:458-472`. Re-pin the fingerprint `:491` once, with the reason
          "12.10 ore" in the comment block `:475-490` and the commit.
        - Let `surface_is_icy` (`:573-581`) accept ore below the soil.
        - The dwarf, height, camp and tree pins (`:423-452`, `worldgen.rs:368-378`, `:421`) must stay
          **untouched**. If any moves, ore is reading the wrong stream.
        - New: same-seed equality and different-seed difference of the ore cells.
        - New: every ore cell is `z + 2 < height` and has no empty neighbour, over 16 seeds.
        - New: bloodstone depth ≥ `GEM_MIN_DEPTH` and bloodstone ≤ tetrahedrite / 10.
        - New: the DEFAULT_SEED census as a printed and asserted band. Record the figures in the
          Completion Notes.
  - [ ] 2.6 `tests/save_load.rs`: a world with ore items lying and carried saves, loads and steps
        identically. Per Task 0.4, a world saved before 12.10 (one without ore tiles) loads and ticks.

- [ ] **Task 3: simd** (AC 1, 9)
  - [ ] `bridge.rs`: `item_kind_out` `:109-114`, `material` `:124-133`, the oracle `expected_material`
        `:347-356`, and the named list in `snapshot_json_obeys_wire_conventions` `:649`. Add the ore arms to
        a unit test of `item_kind_out` (12.7 left it unpinned, `:375-377`).
  - [ ] `serve.rs`: `the_default_world_holds_tetrahedrite_and_bloodstone_on_the_wire`, which reads the snapshot of a
        real daemon and counts both names, with none exposed.

- [ ] **Task 4: gui**, per the approved draft (AC 7)
  - [ ] `appearance.rs` `material_color` `:350-359`. Extend `appearance_tables_pin_the_cold_boot_palette`
        (table `:588-595`, the blue-at-or-above-red check `:602-605`, which stands) and
        `mark_colours_are_distinct_cold_literals` (terrain list `:714-721`, TUI colour list `:760-776`).
  - [ ] `project.rs`:
        - `TerrainSlot` `:209-219` and `TERRAIN_SLOTS` `:229-238` in the **same order** (`slot as usize`
          indexes the array, `:3043-3045`);
        - `base_color` `:241-252` and `TerrainSlot::of` `:254-263`;
        - `material_detail_depth` `:1729-1735`, where ore takes Stone's arm;
        - the item material `:2235-2244`, where ore uses its slot at rim 0, and `item_translation` /
          `item_scale` `:2349-2363`, where ore matches Stone;
        - `sim_will_keep` `:1067-1076`, where dig keeps `Ramp`, with its doc and `designate.rs:219-221`.

        `has_snow_cap` (`:3029-3040`) and `rests_on_the_ground` (`:2976-2981`) stay as they are, with ore
        treated like stone. Note it.
  - [ ] Bench parity, or the gate's bench tier goes red the moment a real export holds ore:
        - `scripts/bench/resolution_bench.py` `KNOWN_MATERIALS` / `_material_detail_depth` `:140-156` and its
          test `:341-350`;
        - `scripts/bench/valley_bench.py` `TERRAIN_RGB` `:35-42`;
        - the matching anchors in `crates/gui/tests/bench_contract.rs:40-202`.

        `CONTROL_FACES`/`CONTROL_QUADS`/`24_644` must NOT move (no ore is exposed).
  - [ ] `headless.rs`, on the `material_rgb` pattern (`:6018-6026`, `:6030-6069`):
        - `an_ore_cell_on_the_cut_face_draws_in_its_own_colour`, both kinds and stone, all distinct;
        - `an_ore_item_projects_as_a_stone_cube_in_its_ore_colour`;
        - `the_dig_preview_keeps_a_ramp_cell`.
  - [ ] `docs/tech-art-guidelines.md` materials table `:24-44`: add rows for both ores (#62).

- [ ] **Task 5: tui** (AC 8)
  - [ ] `palette.rs` `tile_cell` `:23-75`: a `Solid` and a `Ramp` arm per ore (a ramp is `▲` in the ore's
        colour, as today). Add item cells beside `wood_item_cell` / `stored_wood_item_cell` `:155-169`.
  - [ ] `view.rs:319-331`: the four new `(ItemKind, stored)` arms.
  - [ ] `every_look_is_pinned`: add the tile rows `:235-249`. Add the item glyphs to the marker set
        `:333-404`, which asserts they are unique and clear of `existing_glyphs` `:406`. Pin the stored
        variants with their own `assert_eq!` (12.7's deviation).
  - [ ] `tests/client.rs`, on `capture_growing_world` (`:930-1065`): a `--frames 2 --z 1` capture with
        ore cells and items gives a count > 0 for each new glyph, and the control 0.

- [ ] **Task 6: the record**
  - [ ] NEW `mutations/12-10.sh`. Each row is named against its killing test and run alone after commit:
        1. dig yields `Stone` whatever the material (`ore_wire` deliberate RED, plus 2.1);
        2. the dig filter refuses ramps again (2.1 ramp test);
        3. ore seeds from `STREAM_WORLDGEN` (the spawn or height pins);
        4. ore may replace the surface (the never-exposed test);
        5. gems at any depth (the gem-depth test);
        6. the channel leaves `Ramp(ore)` (2.1 channel test);
        7. `sim_will_keep` drops ramps (the headless preview test);
        8. tui bloodstone glyph `$` becomes `█` (`every_look_is_pinned`);
        9. gui bloodstone colour equals stone's (the headless colour test).
  - [ ] Run `scripts/audit-mutations.py` after `cargo fmt`. `3-2-the-dig.sh` and its kin quote the dig
        filter `Some(Tile::Solid(_)))` and the stone spawn `:1695`. Re-point or retire each with a reason.
        An APPLY-FAILED is a finding.
  - [ ] Spine (2026-08-09) AD-16: add an amendment paragraph after `:144-157` covering `Material` and
        `ItemKind` growing by the ore pair, dig taking ramps, and the channel-over-ore rule. Add README rows
        for the new glyphs and colours.

- [ ] **Task 7: live proof and gate** (AC 1, 9, 10)
  - [ ] The Verification recipes: GREEN, the deliberate RED, and the tui count. Record every figure.
  - [ ] NEW `12-10-signoff/vehicle-card.md` from the Verification seat block.
  - [ ] Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` (~55 min; default parallelism OOMs).

## Dev Notes

### Scope guardrails (do NOT)

- No second mineral, no glowing or emissive ore, no ore in soil or on the surface, and no use for ore.
- No goals or stock counting by kind (12.12). No falling items or floating fixes (12.11).
- No channel-down rule (#163 is story 12.10b), and no change to channel beyond the ore case.
- No new `Command`, `Entity` field, tui input, or simd CLI flag. The instrument is the Python wire client.
- No touching soil, snow or ice yields (they stay `stone`). No fix for the gui preview lighting trees for
  dig (pre-existing).

### What already exists (build on it)

- 12.7's item-kind path end to end: `ItemKind` → `item_kind_out` → `Item.kind` → gui item material and
  scale, and tui `(kind, stored)` cells.
- Kind-blind hauling and 12.9's pile fill and item blocking: ore rides them unchanged.
- The purpose-named stream pattern: `seed ^ STREAM_X` into `ChaCha8Rng` (`lib.rs:1972-2006`).
- `ore_wire.py`, the daemon-judged recipe, RED at creation, with a `--control` that is proven GREEN.

### Key decisions & traps

1. **Ore must not consume any other stream's draws.** Ore gets its own RNG, created after `place_ramps`. The
   untouched spawn and height pins are the guard (`deferred-work.md:92-114` is why).
2. **"Interior" is `z + 2 < height`**, which is exactly `layered_terrain`'s Stone rule
   (`worldgen.rs:208-231`). It excludes scoured surface stone and every ramp, so nothing is exposed and no
   `Ramp(ore)` comes from worldgen.
3. **Making ramps diggable changes old outcomes.** A dig rect over ramps only was refused before
   (`dig refused: nothing to dig`) and now applies. Expect scenario re-pins; disclose each beside the pin
   and in the commit (12.9 rule).
4. **Digging a ramp can leave a dwarf standing on air**, as digging any solid already can. That is 12.11's
   AC; do not fix it here.
5. **Glyph counts must not collide with text.** The status and hint rows hold `o` ×6 and `/` (12.7's trap),
   so recipes count by the ANSI colour plus glyph.
6. **A new gui `TerrainSlot` shifts every later slot index.** Keep the enum and `TERRAIN_SLOTS` in step.
   The palette tests' lists are literal arrays, not compiler-checked.

### Project Structure Notes

- UPDATE `crates/protocol/src/lib.rs`; `crates/sim-core/src/{lib.rs, worldgen.rs}`;
  `crates/sim-core/tests/{scenario.rs, worldgen.rs, save_load.rs}`
- UPDATE `crates/simd/src/bridge.rs`, `crates/simd/tests/serve.rs`
- UPDATE `crates/gui/src/{appearance.rs, project.rs, designate.rs}`, `crates/gui/tests/{headless.rs,
  bench_contract.rs}`; `scripts/bench/{resolution_bench.py, test_resolution_bench.py, valley_bench.py}`
- UPDATE `crates/tui/src/{palette.rs, view.rs}`, `crates/tui/tests/client.rs`
- UPDATE `ARCHITECTURE-SPINE.md` (2026-08-09, AD-16), `README.md`, `docs/tech-art-guidelines.md`, old
  mutation tables per Task 6
- NEW `mutations/12-10.sh`, `12-10-signoff/vehicle-card.md`
- EXISTING (creation) `12-10-signoff/ore_wire.py`, `12-10-signoff/draft.md`
- `client-core` needs no change (no `Material`/`ItemKind` branch outside tests).

### Previous story intelligence

- 12.9: every uncarried item blocks its cell, so a one-wide tunnel waits at each dug item until it is hauled
  and stops with no pile (`lib.rs:1078-1079`). Every fixture and recipe places a pile.
- 12.7: the agent split that worked was Sonnet 5.5 subagents in sequence (A: Tasks 1-3; B: Tasks 4-5),
  with the orchestrator on Tasks 6-7. `cargo test -p gui` does not rebuild `simd`.
- Kill test daemons by PID; `pkill -f simd …` matches its own shell and exits 144. Build release probes in a
  scratch worktree while `mutate.sh` runs.

### References

- `epics.md:2676-2695` (Story 12.10), `:2422-2434` (standing ACs), `:218` (AD-7 ore streams), `:2747`
  (12.12 needs reachable gems); PRD `prds/prd-frostvein-2026-09-28/prd.md:46-52, 97-104, 132-137`
- `ARCHITECTURE-SPINE.md` (2026-08-01) AD-6 `:91-107`, AD-7 `:109-121`, AD-11 `:185-198`, colour
  convention `:227`; (2026-08-09) AD-16 `:121-157`
- `docs/tech-art-guidelines.md:24-44, 164-200`; `deferred-work.md:533-539` (glyph guard),
  `:1103-1110` (no cross-client terrain check)
- Issues #163 (channel-down), #152 (wide glyphs), #62 (tech-art doc pins)
- No new dependency. No version research needed: no crate changes, and Bevy stays on 0.19.

## Verification

**Wire (runs today; RED observed at creation):**

```bash
target/release/simd 7711 &   # fresh daemon, DEFAULT_SEED; kill it by PID afterwards
python3 _bmad-output/implementation-artifacts/12-10-signoff/ore_wire.py 7711        # the story
python3 _bmad-output/implementation-artifacts/12-10-signoff/ore_wire.py 7712 4000 --control  # fresh simd 7712
```

- **RED at creation**, on `main` `32565fd`, release `simd`: `census: no ore kinds (every solid tile is one
  of the six pre-12.10 materials)`, `crew can walk to 16063 cells from 5 dwarves; 201693 cells are within
  digging reach`, `ore kinds in the rock: 0 (12.10 wants 2 or 3 …)`, `ORE WIRE RED`, exit 1.
- **CONTROL at creation:** on `main` it is RED, with `plan: stone target [62, 77, 12] (depth 3) via 4 digs
  from [62, 66, 9]` and `stone: target dug NEVER; …`. With the two-line ramp patch (scratch worktree) it is
  `target dug tick 933; stone item on it tick 933; on a pile cell tick 1303` and `ORE WIRE OK (CONTROL:
  plain stone)`, exit 0. The instrument can go green, and it tells the dig rule apart.
- **GREEN, required of dev:**
  - one `census:` line per ore kind with `exposed=0 ramps=0`, the bloodstone's depth starting at ≥ 5;
  - a `plan:` line per kind;
  - per kind, `target dug tick N; <kind> item on it tick N; on a pile cell tick M`;
  - `ore kinds in the rock: 2`, `ORE WIRE OK`, exit 0;
  - the control still `OK`;
  - record every line.
- **Deliberate RED, required of dev:** row 1 in a scratch-worktree release `simd` gives `<kind> item on it
  NEVER; … FIRST ITEM THERE WAS THE WRONG KIND (tick N kind='stone')` and `ORE WIRE RED`.
- Bloodstone is depth ≥ 5, which is 6 digs at the nearest (creation search). If its `plan:` line needs more
  than ~12 digs, raise `TICKS` (the 2nd argument) and record the digs and ticks: 12.12 needs reachable gems.
- `ORE WIRE BROKEN` (exit 2) means no snapshot, a daemon that closed or stalled, or no plan. It is never a
  pass or a fail of the feature.

**tui (runs today; the ore half is RED until the feature exists).** On a fresh daemon, take `Z` from the
tetrahedrite `plan:` line's target:

```bash
target/release/tui 7711 --frame --z Z > "$SCRATCH/f.txt" 2> /dev/null
grep -o '38;2;60;90;210m%' "$SCRATCH/f.txt" | wc -l     # the tetrahedrite literal Task 0.1 rules
```

- **Observed at creation** (`--z 6`, fresh `simd 7741`): `█` 1585, `▓` 95, `%` 0, `$` 0. A bare `o` count
  is 6 with no item on screen (status and hint text).
- **Required of dev:** at least one tetrahedrite `%` by colour escape at `Z`, and one bloodstone `$` at the bloodstone
  target's `Z` (if it is outside the frame, record that and show it in the gui slice instead). Zero is a
  failure, whatever the exit code.

**Seat (AC10), in the seat's launch form:**

- WSL: `simd 7451`, and in a second WSL terminal `tui 7451 --z Z` for parity (Z from a `plan:` line run
  against a separate fresh daemon).
- PowerShell: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`.
- Press `,` to step the slice down into the hill east of the camp until the cut face shows tetrahedrite veins.
  Keep going for a bloodstone.
- Press `3` and drag a 3×3 stockpile beside the fire, then `Esc`. Press `1` and drag a tunnel line from the
  hillside to a vein. The slope cells light in the preview and take marks.
- Watch a miner dig in, the tetrahedrite come out as a blue cube, and a hauler bring it to the pile. In the tui,
  the `•` goes green on the pile.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-10-11 | Created on `32565fd`. Premise defect found: no dig reaches below the surface (ramp-ringed hills), so Task 0.2 proposes ramp digging. `ore_wire.py` RED, its control proven GREEN on a ramp-diggable scratch build. Draft written. Task 0 ruled by Wolf at creation: (b) colours as tetrahedrite + bloodstone, dig takes ramps, #163 split out as 12.10b, channel over ore gives ore + `Ramp(Stone)`, old saves load. |
