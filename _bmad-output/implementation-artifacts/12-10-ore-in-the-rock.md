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

- [x] **Task 1: protocol** (AC 1, 4)
  - [x] `Material` += `Tetrahedrite`, `Bloodstone`; `ItemKind` += `Tetrahedrite`, `Bloodstone` (snake_case, wire diff above).
  - [x] Extend the literal pins: the material list in `every_material_and_tile_variant_has_a_pinned_wire_name`
        (`protocol/src/lib.rs:620-631`), and an ore item JSON pin beside `:732-752`.

- [x] **Task 2: sim-core** (AC 2-6). RED first: write 2.1's tests and record each failing on the
      unchanged rule before 2.2-2.4.
  - [x] 2.1 Scenario tests (`tests/scenario.rs`, fixture style of `dig_one_stone` `:1327-1358`, always with a
        pile, because 12.9 items block):
        - `digging_ore_yields_an_ore_item_that_reaches_the_pile`, one tetrahedrite and one bloodstone cell;
        - `a_dig_mark_takes_a_ramp_and_the_ramp_is_dug_out`;
        - `channelling_over_ore_yields_the_ore_and_leaves_a_stone_ramp`.

        Use `kinds_at` (`:2396-2403`) for kinds.
  - [x] 2.2 `Material` (`lib.rs:55-63`) += `Tetrahedrite`, `Bloodstone`; `ItemKind` (`:131-135`) += same.
        `fn yield_of(material: Material) -> ItemKind`: ore maps to its own kind, everything else to `Stone`.
  - [x] 2.3 Dig filter (`:2326-2329`) keeps `Solid(_) | Ramp(_)` (still no tree). The completion arm (`:1640`)
        takes `Solid(m) | Ramp(m)` and returns `m`. Capture the material **before** `set_tile` (`:1688`),
        and spawn `Item(yield_of(m))` at `:1695`. Channel (`:1643-1651`): if `m` is ore, produce
        `Ramp(Stone)` and spawn `yield_of(m)`.
  - [x] 2.4 `const STREAM_ORE: u64 = 0x4f52_455f_5f5f_5f5f; // "ORE_____"` beside `lib.rs:27-32`.
        `pub(crate) fn place_ore(dims, heights, tiles, rng)` in `worldgen.rs`, called in `generate` after
        `place_ramps` with `ChaCha8Rng::seed_from_u64(seed ^ STREAM_ORE)`. It replaces only
        `Solid(Stone)` with `z + 2 < height`. Constants are hardcoded (`VEIN_COUNT`, `VEIN_LEN`,
        `GEM_COUNT`, `GEM_MIN_DEPTH`).
  - [x] 2.5 `tests/worldgen.rs`:
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
  - [x] 2.6 `tests/save_load.rs`: a world with ore items lying and carried saves, loads and steps
        identically. Per Task 0.4, a world saved before 12.10 (one without ore tiles) loads and ticks.

- [x] **Task 3: simd** (AC 1, 9)
  - [x] `bridge.rs`: `item_kind_out` `:109-114`, `material` `:124-133`, the oracle `expected_material`
        `:347-356`, and the named list in `snapshot_json_obeys_wire_conventions` `:649`. Add the ore arms to
        a unit test of `item_kind_out` (12.7 left it unpinned, `:375-377`).
  - [x] `serve.rs`: `the_default_world_holds_tetrahedrite_and_bloodstone_on_the_wire`, which reads the snapshot of a
        real daemon and counts both names, with none exposed.

- [x] **Task 4: gui**, per the approved draft (AC 7)
  - [x] `appearance.rs` `material_color` `:350-359`. Extend `appearance_tables_pin_the_cold_boot_palette`
        (table `:588-595`, the blue-at-or-above-red check `:602-605`, which stands) and
        `mark_colours_are_distinct_cold_literals` (terrain list `:714-721`, TUI colour list `:760-776`).
  - [x] `project.rs`:
        - `TerrainSlot` `:209-219` and `TERRAIN_SLOTS` `:229-238` in the **same order** (`slot as usize`
          indexes the array, `:3043-3045`);
        - `base_color` `:241-252` and `TerrainSlot::of` `:254-263`;
        - `material_detail_depth` `:1729-1735`, where ore takes Stone's arm;
        - the item material `:2235-2244`, where ore uses its slot at rim 0, and `item_translation` /
          `item_scale` `:2349-2363`, where ore matches Stone;
        - `sim_will_keep` `:1067-1076`, where dig keeps `Ramp`, with its doc and `designate.rs:219-221`.

        `has_snow_cap` (`:3029-3040`) and `rests_on_the_ground` (`:2976-2981`) stay as they are, with ore
        treated like stone. Note it.
  - [x] Bench parity, or the gate's bench tier goes red the moment a real export holds ore:
        - `scripts/bench/resolution_bench.py` `KNOWN_MATERIALS` / `_material_detail_depth` `:140-156` and its
          test `:341-350`;
        - `scripts/bench/valley_bench.py` `TERRAIN_RGB` `:35-42`;
        - the matching anchors in `crates/gui/tests/bench_contract.rs:40-202`.

        `CONTROL_FACES`/`CONTROL_QUADS`/`24_644` must NOT move (no ore is exposed).
  - [x] `headless.rs`, on the `material_rgb` pattern (`:6018-6026`, `:6030-6069`):
        - `an_ore_cell_on_the_cut_face_draws_in_its_own_colour`, both kinds and stone, all distinct;
        - `an_ore_item_projects_as_a_stone_cube_in_its_ore_colour`;
        - `the_dig_preview_keeps_a_ramp_cell`.
  - [x] `docs/tech-art-guidelines.md` materials table `:24-44`: add rows for both ores (#62).

- [x] **Task 5: tui** (AC 8)
  - [x] `palette.rs` `tile_cell` `:23-75`: a `Solid` and a `Ramp` arm per ore (a ramp is `▲` in the ore's
        colour, as today). Add item cells beside `wood_item_cell` / `stored_wood_item_cell` `:155-169`.
  - [x] `view.rs:319-331`: the four new `(ItemKind, stored)` arms.
  - [x] `every_look_is_pinned`: add the tile rows `:235-249`. Add the item glyphs to the marker set
        `:333-404`, which asserts they are unique and clear of `existing_glyphs` `:406`. Pin the stored
        variants with their own `assert_eq!` (12.7's deviation).
  - [x] `tests/client.rs`, on `capture_growing_world` (`:930-1065`): a `--frames 2 --z 1` capture with
        ore cells and items gives a count > 0 for each new glyph, and the control 0.

- [x] **Task 6: the record**
  - [x] NEW `mutations/12-10.sh`. Each row is named against its killing test and run alone after commit:
        1. dig yields `Stone` whatever the material (`ore_wire` deliberate RED, plus 2.1);
        2. the dig filter refuses ramps again (2.1 ramp test);
        3. ore seeds from `STREAM_WORLDGEN` (the spawn or height pins);
        4. ore may replace the surface (the never-exposed test);
        5. gems at any depth (the gem-depth test);
        6. the channel leaves `Ramp(ore)` (2.1 channel test);
        7. `sim_will_keep` drops ramps (the headless preview test);
        8. tui bloodstone glyph `$` becomes `█` (`every_look_is_pinned`);
        9. gui bloodstone colour equals stone's (the headless colour test).
  - [x] Run `scripts/audit-mutations.py` after `cargo fmt`. `3-2-the-dig.sh` and its kin quote the dig
        filter `Some(Tile::Solid(_)))` and the stone spawn `:1695`. Re-point or retire each with a reason.
        An APPLY-FAILED is a finding.
  - [x] Spine (2026-08-09) AD-16: add an amendment paragraph after `:144-157` covering `Material` and
        `ItemKind` growing by the ore pair, dig taking ramps, and the channel-over-ore rule. Add README rows
        for the new glyphs and colours.

- [x] **Task 7: live proof and gate** (AC 1, 9, 10)
  - [x] The Verification recipes: GREEN, the deliberate RED, and the tui count. Record every figure.
  - [x] NEW `12-10-signoff/vehicle-card.md` from the Verification seat block.
  - [x] Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` (~55 min; default parallelism OOMs).

### Review Findings

Code review run 1, 2026-10-11, on `6e4e8c6` (diff `32565fd...HEAD`, 35 files, +2,311/-50; code 19 files,
+1,116/-30). This was a fresh session. Four layers ran and none timed out. All ran cargo 1.97.1, each in its own
`/tmp/review-<layer>` target dir with `CARGO_BUILD_JOBS=3`.
- Blind Hunter (Sonnet) took sim-core `src` plus gui `appearance.rs` and `project.rs`.
- Edge Case Hunter (Sonnet) took simd, tui, protocol, gui `designate.rs`, every `tests/`, the bench scripts and
  `ore_wire.py`.
- The Acceptance and Feature Auditors (Opus) took the whole diff.

| Layer | Findings | Severity | Ran |
| --- | --- | --- | --- |
| Blind Hunter | 2 | 1 MED, 1 LOW | the sim-core suite on a `git archive` copy; three dig probes in the copy |
| Edge Case Hunter | 5 (+2 notes) | 5 LOW | protocol/simd/tui suites; new sim-core, gui and bench tests by name; `ore_wire.py 8000` OK; tui `--z 11` count |
| Acceptance Auditor | 5 | 2 MED, 3 LOW | every named test; `ore_wire.py` ×4 (2 RED at 8000, OK at 16000, row 1 RED); a `32565fd` vs HEAD snapshot diff; a real `main`-written save loaded live; all 10 rows applied in copies; a soil/snow/ice yield mutant |
| Feature Auditor | 5 | 5 LOW | hop trace (every hop WIRED); `ore_wire.py 8000` OK; the seat card replayed over the wire with ONE pile (3 runs); tui frames at z 9-13 |

Convergences:
- the snow cap on dug ore: blind + feature;
- `rests_on_the_ground` NOTE: blind + acceptance;
- seat card landmarks and tui steps: feature + acceptance;
- cut-face test coverage: feature + acceptance.

Live: `ore_wire.py` reproduces GREEN on three independent fresh daemons at 8000 ticks when it connects at once, and
the row-1 RED reproduces. The seat card's own tunnel was replayed headless with one 3×3 pile: the ramp cell takes its
mark and is dug, and the tetrahedrite reaches the pile (tick ~1,927). AC 2 was checked against `32565fd`: every one of
the 5,308 changed cells is ore over `{"solid":"stone"}`, and entities are identical. Seed 42 folded back to stone gives
main's fingerprint. **Not proven live:** AC 10 in full (the seat), the visual half of AC 7 (ore reads as ore under the
real lighting at `--subdiv 4`), and AC 8's glyph width in Windows Terminal.

Cost: $19.13 over 395 turns (subagents 81.4% of tokens). Reaped after triage: 10 `/tmp` build dirs, 64.6 GB
(33.7 GB reclaimed on disk). Patches LEFT AS ACTION ITEMS (Wolf, 2026-10-11) for a `/bmad-review-patch` session.

**Seat pass 1 (Wolf, 2026-10-11, after the review):** ore FOUND. Bloodstone is too difficult to find, so it is
NOT verified at the seat (AC 10's bloodstone half stays open). "Digging is messy": Wolf wants it fixed properly in
the NEXT story, not in this patch pass.

- [x] [Review][Patch] (decision resolved) **MED: a tunnel floor of ore draws a snow cap (blind + feature)** — `has_snow_cap`
  (`crates/gui/src/project.rs:3047-3058`) checks only the cell directly above. Any ore cell with `Empty` above therefore
  draws a SnowCap top face (`:1492`), even under a roofed tunnel. On the seat card's own tunnel, `[77,64,10]` is
  tetrahedrite directly under the target `[77,64,11]`, so once the target is dug the vein's continuation in the floor
  reads white. Task 4 kept `has_snow_cap` unchanged ("ore treated like stone"), and stone tunnel floors already cap the
  same way, so this follows the spec. But the function's own doc gives the reason soil was excluded: "an excavation
  erased itself the moment it finished". The 12.10 NOTE's justification ("never exposed at boot") ignores digging, which
  is how ore is found.
  **RULED 2026-10-11 (Wolf): option 1, exclude ore from the cap as soil is.** Fix the NOTE, and add a headless test that
  a dug ore floor draws in its own colour.
- [x] [Review][Patch] **MED (#191): `ore_wire.py` goes RED at the recorded 8000 ticks if it connects a few seconds late
  (acceptance)** [`_bmad-output/implementation-artifacts/12-10-signoff/ore_wire.py:104-108,227-230`] — The planner
  starts from the dwarves' positions in the connect snapshot. Two fresh daemons, connected late, planned `bloodstone
  target [95, 23, 17] via 6 digs from [66, 61, 9]`: `on a pile cell NEVER`, `ORE WIRE RED`. At 16000 the same plan piled
  at tick 8717, `OK`. Fix: stop once every kind is piled, raise the recorded TICKS to 16000, and record the dependence.
  Same file, folded in (edge, LOW, a latent silent failure): the exposure census (`:90`) counts only `"empty"`
  neighbours, so ore on the grid edge would print `exposed=0`. Count an off-grid neighbour as exposed, as `serve.rs` and
  the worldgen test do.
- [x] [Review][Patch] **MED: the seat card misplaces the fire, ignores the gui mirror, and its tui steps cannot show the
  pile or the bloodstone (feature + acceptance)** [`_bmad-output/implementation-artifacts/12-10-signoff/vehicle-card.md`]
  - The campfire is at `[64,64,9]`, not "near `[62,66,9]`" (a dwarf spawn).
  - #169 is open: "east of the camp" is on the LEFT in the gui.
  - The tui at `--z 11` shows 0 green `•`, because the pile is at z 9; say "press `<` twice".
  - At `--z 13`, the bloodstone at `[55,83]` is 19 rows south, off an 80×24 frame. Name the scroll, or point at the
    bloodstones visible at z 10/11 near the east hill (`[80,72,10]`, `[89,77,11]`).
- [x] [Review][Patch] **MED: AC 4's "any other non-tree material still yields stone" has no test: a mutant survives
  the whole sim-core suite (acceptance)** [`crates/sim-core/src/lib.rs:146-158`, `crates/sim-core/tests/scenario.rs:3107`]
  — With `Soil | Snow | Ice => ItemKind::Tetrahedrite`, `cargo test -p sim-core --no-fail-fast` was 0 failed. Add a
  scenario test that digs soil, snow and ice and asserts one `Stone`, plus a `12-10.sh` row naming it.
- [x] [Review][Patch] **LOW: the `rests_on_the_ground` NOTE says ore counts as stone, and the code excludes ore (blind +
  acceptance)** [`crates/gui/src/project.rs:2991-2997`] — The match is `Stone | Soil | Ice | Snow`. Task 4 rules ore
  "treated like stone", so add the two ore arms (unreachable today). Patched despite LOW because it is a comment
  misdescribing the code and a spec deviation.
- [x] [Review][Defer] **LOW: the "cut face" headless test never slices and never runs `--subdiv 4` (feature +
  acceptance)** [`crates/gui/tests/headless.rs:6126`] — deferred. It builds a 2-tall world with ore on top and queries
  `TerrainTile` (subdiv 1 only). Both paths read `TerrainSlot::of` (traced, wired), so a `TERRAIN_SLOTS` drift in the
  chunk path would pass it. The seat is the real cut-face evidence.
- [x] [Review][Defer] **LOW: the pre-12.10 save test is built by the new code (edge)**
  [`crates/sim-core/tests/save_load.rs`, `a_world_saved_before_12_10_loads_ore_less_and_ticks`] — deferred. It
  rewrites a new save's ore to stone, so it cannot see a representation change. The acceptance layer loaded a real
  `main`-written `frostvein.save` in the HEAD daemon live (ore 0, ticks on), so AC 6's old-save clause is observed.
- [x] [Review][Defer] **LOW: the tui ore-capture test is a lower-bound presence check (edge)**
  [`crates/tui/tests/client.rs:1462-1490`] — deferred. It checks `found >= 1` per colour escape; the literal palette
  pins and the view test's `stored_*` asserts cover glyph and arm swaps.
- [x] [Review][Defer] **LOW: `•` and `◊` are East-Asian-Width Ambiguous (edge)** [`crates/tui/src/palette.rs`] —
  deferred to the seat (AC 8). Neither is emoji-capable. `◊` already carries a NOTE.

Dismissed (2):
- `ore_wire.py` "piled while still carried" (edge): a carried item has no `Pos` and drops out of `items()`
  (`simd/src/bridge.rs:91`), so the wire never reports it on a pile cell.
- the census band pinned to a measured value (edge): intended, per its NOTE.

#### Patch pass 1 (2026-10-11, fresh session, on `571c15e`)

All 5 patch items landed in 4 commits: `d3a3d12` (AC 4 test), `e2bc89f` (snow cap + `rests_on_the_ground`),
`d3bbfb3` (`ore_wire.py`), `b2657c5` (seat card). **Full gate GREEN at `b2657c5`**
(`RUST_TEST_THREADS=1 scripts/gate.sh`, exit 0, 3502 s: cargo test 414 s, pixel guards 3044 s). Mutations
for the rows this pass touched: **3/3 KILLED** (12-10 rows 10 and 11, and 5.4's "snow cap leaves bare top",
re-pointed because the cap guard grew two lines; the first commit attempt went RED on exactly that APPLY-FAILED).

| Item | Fix written for | Then tested | Pre-existing-state fixture | Mutation |
| --- | --- | --- | --- | --- |
| Snow cap on ore (ruling) | ore opened to the sky by a dig delta | ore ALREADY open in the boot snapshot (a save or mid-dig world), plus open stone beside it still capped (non-vacuity) | `a_dug_ore_floor_draws_in_its_own_colour_not_a_snow_cap`: `[30,30,0]`/`[32,30,0]` open at snapshot, `[30,34,0]`/`[32,34,0]` roofed then dug | row 11 KILLED. RED before the fix on the snapshot side, and on the dug side alone with the loop cut to `dug` (`Tetrahedrite at [30, 34, 0] … must not draw a snow cap`) |
| #191 `ore_wire.py` late connect | a late connect (crew wandered) | a prompt connect, the old path, still OK and now stops early | live fresh `simd`: A at once → OK, piled 5538, 33 s; B 5 s late → plan `[95,23,17]` from `[65,61,9]`, piled 9102, OK; C the OLD script at 8000, 5 s late → `on a pile cell NEVER`, RED (#191 reproduced) | hand mutant: drop `and not all(piled)` from the BROKEN check → `ORE WIRE BROKEN: the daemon closed at tick 5670, before TICKS=16000`, exit 2 |
| `ore_wire.py` edge census (folded) | ore on the grid edge | fully enclosed interior ore (the only shape worldgen makes) | fake daemon (scratch `edge_daemon.py`, 3×3×3 stone): tetrahedrite at x=0 `(0,1,1)`, bloodstone enclosed `(1,1,1)` | the old script is the mutant: edge `exposed=0`; new `exposed=1`; enclosed `exposed=0` in both. The live DEFAULT_SEED census is unchanged (`exposed=0` for both kinds) |
| Seat card | a reader at the seat | every claim re-read on a fresh daemon and a tui frame | fresh DEFAULT_SEED snapshot: campfire `[64,64,9]`; `[77,64,11]` tetrahedrite, `[80,72,10]` bloodstone (depth 6). `tui --frame` per z: bloodstone `$` count z9 0, z10 1 (row 18 col 56 = `[80,72]`), z11 1, z13 0 | n/a (prose). **Review's evidence partly wrong:** z 11's visible `$` is `[96,55,11]` (depth 14), and the suggested `[89,77,11]` falls on row 23, off the map. The card names `[80,72,10]` only |
| AC 4 other materials | soil/snow/ice dug (the pre-12.10 rule, unchanged) | the ore arms, already pinned by `digging_ore_yields_…` | `digging_soil_snow_and_ice_still_yields_stone`: generated seed-42 world, each material set on the cell beside dwarf 2 | row 10 (ICE ONLY, the loop's last material) KILLED: `digging Ice must yield one stone, got [(Id(10), Tetrahedrite)]` |
| `rests_on_the_ground` arms | the NOTE's claim (ore counts as stone) | no test: unreachable, since no tree stands on interior ore | none; a comment and arm fix | none (no new test) |

No REWORK rows: none re-closes a prior round's fix.

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
- **TICKS depends on when you connect (#191, review run 1).** The planner starts from where the connect
  snapshot finds the dwarves. Connected 5 s late, the bloodstone plan becomes `[95, 23, 17] via 6 digs from
  [65, 61, 9]` and piles at tick 9102, so a run at 8000 reads `on a pile cell NEVER`, `ORE WIRE RED`. The
  default is now **16000**, and the script stops as soon as every kind is piled (connected at once: done at
  tick 5538, 33 s wall).
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

- Orchestrator + verifier: Claude Opus 5.5 (`claude-opus-5-5`). Wolf picked Sonnet 5.5 subagents for the
  implementation (2026-10-11), as in 12.2-12.9. The skill config still names Codex; Wolf chose against it.
- Agent A (Sonnet 5.5): Tasks 1, 2 and 3, the gui/tui compile-only stubs, and the old mutation rows its
  changes un-anchored. Agent B (Sonnet 5.5): Tasks 4 and 5, replacing every stub, and the rows its changes
  un-anchored. They ran one after the other, on one tree.
- The orchestrator made `yield_of` exhaustive (`796b450`). It wrote `mutations/12-10.sh`, the spine, README
  and seat card, and fixed the instrument's pile. It ran every mutation, every live recipe and the full gate.

### Debug Log References

- **Task 1 RED** (A): the protocol pins did not compile, with four `E0599 no variant … named
  Tetrahedrite/Bloodstone` errors (`Material`, `ItemKind`).
- **Task 2 RED** (A). The enum variants were added and the rule left unchanged:
  - `digging_ore_yields_an_ore_item_that_reaches_the_pile` failed with `digging Tetrahedrite must yield one
    Tetrahedrite item, got [(Id(10), Stone)]`.
  - `a_dig_mark_takes_a_ramp_and_the_ramp_is_dug_out` failed with `a dig over a ramp must not be refused`
    (`left: Some(Designate{Dig…}) right: None`).
  - `channelling_over_ore_yields_the_ore_and_leaves_a_stone_ramp` failed with `left:
    Some(Ramp(Tetrahedrite)) right: Some(Ramp(Stone))`.
  - The fingerprint failed alone after `place_ore`. Every dwarf, height, camp and tree pin passed untouched.
- **Task 3 RED** (A): `E0004 non-exhaustive patterns` in `bridge.rs`: 2 in the bin, 3 in the test bin (the
  `expected_material` oracle).
- **Task 4 RED** (B). Each test was run against A's stone-look stubs:
  - `an_ore_cell_on_the_cut_face_draws_in_its_own_colour` failed with `no tetrahedrite [40, 64, 176] face
    drawn`. Its stone assertion passed, so the test has a positive half.
  - `an_ore_item_projects_as_a_stone_cube_in_its_ore_colour` failed with `left: [60, 70, 92] right: [40, 64,
    176]`.
  - `the_dig_preview_keeps_a_ramp_cell` failed with `left: [[1, 1, 1]] right: [[1, 1, 1], [2, 1, 1]]`.
  - `appearance_tables_pin_the_cold_boot_palette` was taken RED after the change, by setting tetrahedrite
    back to stone.
  - `bench_literals_match_the_client_palette_lights_and_boot_camera` failed with `matched 0:
    "tetrahedrite": (40, 64, 176)`.
  - `test_ore_takes_stones_relief_rule` failed with `'tetrahedrite' not found`.
  - The blue≥red check and `mark_colours_are_distinct_cold_literals` were extended and pass. They had no
    separate RED.
- **Task 5 RED** (B):
  - `every_look_is_pinned` failed with `left: '█' (86,92,104) right: '%' (60,90,210)`.
  - `an_ore_item_draws_its_own_glyph_and_a_stored_one_in_the_stockpile_colour` failed with `left: '*'
    right: '•'`, both in zone green.
  - `ore_cells_and_ore_items_reach_the_capture_in_their_own_glyph_and_colour` failed with `no % in colour
    60;90;210`.
- **Live recipe: instrument defect found and fixed** (`bad0002`). The first GREEN attempt was RED:
  `bloodstone: target dug NEVER`, at 4000 ticks and again at 12000. A probe showed the cause:
  - the one 3×3 pile was full at 9 items (8 stone, 1 tetrahedrite);
  - the two tunnels dig 15 cells;
  - a stone at `[55,78,13]` blocked the one-wide bloodstone tunnel (12.9), and the marks `[55,79..83,13]`
    were left.

  No 5×5 or 4×4 is standable near the camp, so the recipe now places **two disjoint 3×3 piles** (18
  cells). Bloodstone needs ~5,700 ticks, so the recorded run uses `TICKS=8000`, as the Verification allows.
  A probe on two piles at 8000 had every planned cell dug and the bloodstone on a pile cell at `[61,64,9]`.
- **Live GREEN** (orchestrator): release `simd` built in a scratch worktree at `796b450`, fresh `simd
  7718`, `ore_wire.py 7718 8000`:
  - `census: bloodstone cells=252 depth=5..25 exposed=0 ramps=0`;
  - `census: tetrahedrite cells=5056 depth=3..25 exposed=0 ramps=0`;
  - `crew can walk to 16063 cells from 5 dwarves; 201693 cells are within digging reach`;
  - `plan: tetrahedrite target [77, 64, 11] (depth 3) via 5 digs from [67, 64, 9]`;
  - `plan: bloodstone target [55, 83, 13] (depth 6) via 10 digs from [62, 67, 9]`;
  - `piles: [[59, 64, 9], [64, 59, 9]] (18 cells)`;
  - `bloodstone: target dug tick 4938; bloodstone item on it tick 4938; on a pile cell tick 5670`;
  - `tetrahedrite: target dug tick 1924; tetrahedrite item on it tick 1924; on a pile cell tick 2326`;
  - `ore kinds in the rock: 2`, `ORE WIRE OK`, exit 0.

  The bloodstone plan is 10 digs, under the ~12 the Verification flags.
- **Control** (fresh `simd 7719`, `4000 --control`): `plan: stone target [62, 77, 12] (depth 3) via 4 digs
  from [62, 67, 9]`; `target dug tick 918; stone item on it tick 918; on a pile cell tick 1341`; `ORE WIRE
  OK (CONTROL: plain stone)`, exit 0.
- **Deliberate RED** (row 1, release `simd` in a scratch worktree off `796b450`, `simd 7720`, `8000`):
  - `bloodstone: target dug tick 4806; bloodstone item on it NEVER; on a pile cell NEVER; FIRST ITEM THERE
    WAS THE WRONG KIND (tick 4806 kind='stone')`;
  - `tetrahedrite: target dug tick 1874; … FIRST ITEM THERE WAS THE WRONG KIND (tick 1874 kind='stone')`;
  - `ORE WIRE RED`, exit 1.

  Ticks vary by ~±250 between runs: commands land on whatever tick the daemon is at when they arrive.
- **tui recipe** (release `tui`/`simd` at `a69bf7c`, fresh `simd 7741`, `--frame`). Counts are by colour
  escape, which the renderer emits only when the colour changes, so they are lower bounds:
  - `--z 11` (the tetrahedrite target): `38;2;60;90;210m%` 88, `38;2;130;220;190m$` 5, `█` 5458;
  - `--z 13` (the bloodstone target): `%` 80, `$` 4.

  At creation, on `main`, `%` and `$` were 0 (`--z 6`).
- **Mutations, `12-10.sh`** (`RUST_TEST_THREADS=1 scripts/mutate.sh`, after commit `a69bf7c`): 9 of 10
  killed on the first run.
  - Row 1 killed at `digging Tetrahedrite must yield …`. Row 2 killed at the ramp test. Row 3a killed at
    `spawn_positions_for_seed_42_are_pinned` (the fingerprint). Row 4 killed at `worldgen.rs:806` (`within
    the soil layer or surface`). Row 5 killed at `worldgen.rs:833` (`bloodstone … is only depth`). Row 6
    killed at `scenario.rs:3207` (`never a ramp of the ore`). Row 7 killed at `headless.rs:6203`. Row 8
    killed at `palette.rs:304`. Row 9 killed at `headless.rs:6159`.
  - **Row 3b SURVIVED** against `tree_density_for_seed_42_is_deterministic_and_in_target_band`. Ore drawing
    from the tree stream moves every tree, but the trunk count stays in its 230..=300 band. Run by hand
    under that mutant, with `--no-fail-fast` across sim-core, it was caught by 7 scenario tests (the 12.7
    tree pin `no trunk at (73, 59): pinned to DEFAULT_SEED`, among them), one `save_load` test and the
    fingerprint. Re-pointed to `cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing`
    (`ec7777a`). Run alone after that commit, it was **KILLED**. 10/10.
  - Row 3a is the story's literal row. Ore seeded from another fresh stream consumes no other stream's
    draws, so only the fingerprint sees it. That pin was re-pinned in this story, so the row is weaker than
    3b, which is the real hazard.
- Exclusivity is not claimed. Each row was run against its named test only.
- **Full gate** (`RUST_TEST_THREADS=1 scripts/gate.sh`, at `ec7777a`): `GATE GREEN 3393s`, exit 0.
  The steps took: fmt 0s, clippy 3s, cargo test 384s, pixel guards 2970s, the three no-sim-core-edge
  probes ok, metrics ledger 0s, bench tests 19s, and mutation tables still apply 9s.

### Completion Notes List

- **Ore placement** (`worldgen.rs`, `place_ore`, own `STREAM_ORE`, after `place_ramps`):
  - placement is proportional, not a fixed `VEIN_COUNT`/`GEM_COUNT`: one random-walk vein of 4-12 cells
    per `INTERIOR_PER_VEIN = 360` interior cells, then one bloodstone per `TETRAHEDRITE_PER_BLOODSTONE = 20`
    tetrahedrite, at depth ≥ `GEM_MIN_DEPTH = 5`;
  - small test worlds are not saturated. Depth is `height - z`, so the surface cell is depth 0.
- **DEFAULT_SEED census:**
  - 5,056 tetrahedrite in 554 vein components (9.1 per vein), 2.02% of 250,849 interior stone;
  - 252 bloodstone at depth 5..=25, 1 per 20.1;
  - the asserted band is 4,550-5,550 cells, 500-620 veins, 1.5-2.5%.
- **Deviation: ore keeps one cell off the world edge** (`fa9c42c`). The resolution bench counts a grid-edge
  face as exposed, and ore on the edge split greedy quads (`greedy_quads=12845 (expected 12322)`). Exposed
  faces did not move. The story's "ore cannot move a bench count" held for faces, not for quads. With the
  edge skipped, `CONTROL_FACES 62_586`, `CONTROL_QUADS 12_322` and `24_644` are unchanged. The never-exposed
  test treats the grid edge as air.
- **Re-pins:** only the terrain fingerprint, `0xe5fa_e10f_6708_cc79` (main) → `0x9bb0_34d6_e3cd_0a93`.
  It moved twice in-branch (`eb3dc1c`, then `fa9c42c`), and the first value was never released. No
  scenario pin moved from ramps becoming diggable.
- `yield_of` is an exhaustive `match` (`796b450`; Agent A had a `_ => Stone` fallback), so a new material
  must choose its yield.
- **Look:** every draft literal passed the palette tests unchanged.
  - gui: tetrahedrite (40,64,176), bloodstone (120,196,176).
  - tui rock: `%` (60,90,210), `$` (130,220,190), and ramps `▲` in the same colours.
  - tui items: `•` and `◊`, in zone green when stored.
  - `TerrainSlot` puts the ores right after Stone, with `TERRAIN_SLOTS` (10) in step. `has_snow_cap` and
    `rests_on_the_ground` are unchanged, each with a `// NOTE:` that ore counts as stone.
- **Old mutation rows re-pointed:**
  - `3-2-the-dig.sh`, 7 rows: the dig filter, the change tuple, the channel arm, the spawn, the stale-job
    row, and "items draw above entities" (twice: A's stubs, then B's arms);
  - `3-3` "stone counting and stone drawing use different filters";
  - `12-7.sh` row 6.

  None was retired. `audit-mutations.py`: 850 rows, all anchored.
- **The commit `bad0002`** (the Python instrument only) was made with `--no-verify`. The next commit,
  `a69bf7c`, ran the fast gate over the same tree, green.
- **AC 10 (the seat) is Wolf's.** The card is `12-10-signoff/vehicle-card.md`. Its tetrahedrite and
  bloodstone coordinates and z-levels come from the live run.

### File List

- `crates/protocol/src/lib.rs`
- `crates/sim-core/src/lib.rs`, `crates/sim-core/src/worldgen.rs`
- `crates/sim-core/tests/scenario.rs`, `crates/sim-core/tests/worldgen.rs`, `crates/sim-core/tests/save_load.rs`
- `crates/simd/src/bridge.rs`, `crates/simd/tests/serve.rs`
- `crates/gui/src/appearance.rs`, `crates/gui/src/project.rs`, `crates/gui/src/designate.rs`
- `crates/gui/tests/headless.rs`, `crates/gui/tests/bench_contract.rs`
- `crates/tui/src/palette.rs`, `crates/tui/src/view.rs`, `crates/tui/tests/client.rs`
- `scripts/bench/resolution_bench.py`, `scripts/bench/valley_bench.py`, `scripts/tests/test_resolution_bench.py`
- `docs/tech-art-guidelines.md`, `README.md`
- `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`
- `_bmad-output/implementation-artifacts/mutations/12-10.sh` (NEW), `mutations/3-2-the-dig.sh`,
  `mutations/3-3-the-haul-and-the-skeleton-walks.sh`, `mutations/12-7.sh`
- `_bmad-output/implementation-artifacts/12-10-signoff/ore_wire.py` (pile fix),
  `12-10-signoff/vehicle-card.md` (NEW)
- `_bmad-output/implementation-artifacts/12-10-ore-in-the-rock.md`, `sprint-status.yaml`

## Change Log

| Date | Change |
|---|---|
| 2026-10-11 | Created on `32565fd`. Premise defect found: no dig reaches below the surface (ramp-ringed hills), so Task 0.2 proposes ramp digging. `ore_wire.py` RED, its control proven GREEN on a ramp-diggable scratch build. Draft written. Task 0 ruled by Wolf at creation: (b) colours as tetrahedrite + bloodstone, dig takes ramps, #163 split out as 12.10b, channel over ore gives ore + `Ramp(Stone)`, old saves load. |
| 2026-10-11 | Dev (Sonnet 5.5 subagents A and B, orchestrated and verified by Opus 5.5). Ore veins (tetrahedrite) and rare deep gems (bloodstone) are placed in interior stone from `STREAM_ORE`. Dig takes ramps; a dig or channel over ore yields the ore item, and the channel leaves `Ramp(Stone)`. Wire, gui and tui draw both. Fingerprint re-pinned to `0x9bb0_34d6_e3cd_0a93`. `ore_wire.py` GREEN (two piles, 8000 ticks), and the deliberate RED is red. Mutations 10/10 after row 3b was re-pointed. Full gate green at `ec7777a`. Status → review; AC 10 (the seat) is Wolf's. |
| 2026-10-11 | Review patch pass 1: all 5 patch items landed (`d3a3d12`, `e2bc89f`, `d3bbfb3`, `b2657c5`). Ore excluded from the snow cap (Wolf's ruling); AC 4 soil/snow/ice yield test; `ore_wire.py` to 16000 ticks with early stop and an edge-aware census; seat card fixed. Full gate GREEN at `b2657c5` (3502 s); touched mutation rows 3/3 killed. Status stays in-progress: AC 10 (the seat) is unrun. |
