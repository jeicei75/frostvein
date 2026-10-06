---
baseline_commit: fd9ca98
model: claude-opus-5-5  # session default, same as 12.1-12.6's creation
---

# Story 12.7: Timber

Status: in-progress

## Story

As the boss,
I want to mark trees for cutting and have my woodcutters fell them for wood,
so that the pines become something my fortress collects.

## Not stacked: branch off `main`

`main` is `fd9ca98` (PR #167 merged), clean. Branch `story-12-7-timber` carries this file, `12-7-signoff/`
and the board edit. Its first commit flips 12.6 to `done` in both its story file and the board, because 12.6
merged at `review`. Epic 12's nine standing ACs (`epics.md:2422-2434`) bind this story and are not restated.
"Wire diff" below satisfies standing AC 4. Standing AC 7 applies: Wolf approves `12-7-signoff/draft.md` at
Task 0, before any gui or tui display is built. Wolf ruled Task 0 at creation.

## Found at creation (2026-10-05, on `fd9ca98`)

- **Live RED** (`12-7-signoff/timber_wire.py`, fresh release `simd 7703`, DEFAULT_SEED):
  - The daemon logs `unrecognized client message` for both `designate kind cut` commands.
  - Per finding, the script reports: `cut mark at A's base: NEVER`, `a woodcutter holds the cut: NEVER`,
    `every tile of A empty: NEVER`, `wood item … NEVER` (both), and `cut over no tree refused: NEVER`.
  - `dig mark placed on tree D: tick 15` and `tree D changed: tick 208`: a dig mark lands on a trunk tile
    and a miner digs it away, dropping nothing.
  - Verdict `TIMBER WIRE RED`, exit 1.
- **Dig takes tree tiles today.** The `Designate` filter keeps any `Tile::Solid(_)` for dig and any standable
  cell for channel (`sim-core/src/lib.rs:1670-1680`), so trunks, foliage and treetops all take marks.
  `execute_jobs` then empties the tile and sets `yields_stone = false` for tree materials (`:1112-1158`).
  AD-16 records that "for nothing" as Wolf's 2026-08-09 call; this story supersedes it.
- **Trees have no identity, only tiles.** `place_trees` (`worldgen.rs:290-358`) builds each tree from:
  - a trunk column of 3-5 `TreeTrunk` cells (`surface+1 ..= crown_top-1`);
  - a `TreeFoliage` tip at `crown_top`;
  - `TreeFoliage` rings in the 3×3 around the trunk at `crown_top-2 ..= crown_top-1`, written only into
    `Empty` cells.

  Trunks are at least 3 apart (Chebyshev, `:310-316`), so a crown's ring is distance 1 from its own trunk and
  at least 2 from any other. **Crowns can TOUCH**: DEFAULT_SEED has 259 trees, and 22 of the 57
  distance-3 pairs share a face, among them A (73,59) / B (73,56), about 11 tiles from camp. A flood fill
  over tree tiles fells both. The gui already uses the same 3×3-around-the-trunk rule to retire a tree's
  mesh (`incremental_tree_cover`, `gui/src/project.rs:2966`), so a felled pine vanishes with no new
  tree-mesh code.
- **Tree A is reachable.** Its base is (73,59,12), and its standable work cells are (72,59,12) and
  (73,60,12). A probe channel on each was worked by a miner walking from camp (fresh `simd 7704`).
- **Items have no kind.** `struct Item;` (`lib.rs:128`) is a marker; `items() -> Vec<(Id, Pos)>`
  (`:1864`). On the wire `protocol::Item` is `{id,pos}`, and its comment at `protocol/src/lib.rs:249`
  reads "a second item kind adds a kind field". Hauling is already kind-blind (`create_haul_jobs` `:350`,
  `uncarried_stones` `:865`), so wood hauls with no new haul code.
- **A designation that marks nothing is silent.** `simd` discards `apply_command`'s result for
  `Designate` and `CancelDesignation` (`simd/src/main.rs:224-234`). Only `PlaceStockpile` and
  `SetProfession` push refusals (`:235-253`).
- **The woodcutter has no work.** `trade` (`lib.rs:435-440`) maps nothing to `Woodcutter`. On
  DEFAULT_SEED the one woodcutter is Nain (id 0).
- **A cutting woodcutter would play Walk.** `dwarf_clip` (`gui/src/project.rs:425-437`) returns Dig only for
  a `Dig`/`Channel` job in Work, else Carry if carrying, else Walk (Task 0.1).
- **Over a forest, gui drags never reach a trunk's foot.**
  - Dig uses the single-z rect at the picked cell (`designate.rs:183`). Picking skips foliage
    (`pick.rs:337-340`), so a ground hit gives the ground level, one below every trunk.
  - Channel, stockpile and clear follow the surface (`client_core::surface_targets`,
    `client-core/src/lib.rs:269`). In a trunk column the standable cell is the one above the crown tip.

  So neither path reaches the cell a cut mark sits on, and the cut targeting in Task 4 is new.

## Wire diff (standing AC 4)

- `protocol::DesignationKind::Cut` (`:135-138`) gives wire `"cut"`. The command is the existing `designate`:
  `{"type":"designate","kind":"cut","rect":{"min":[73,59,12],"max":[73,59,12]}}`. No new `Command`
  variant; AD-10's enumeration is unchanged.
- `protocol::DwarfJob::Cut { target }` (`:107-111`) gives `{"cut":{"target":[73,59,12]}}`. The bridge's
  `dwarf_job` is exhaustive, so 12.7 must carry it; 12.8's clip reads it.
- NEW `protocol::ItemKind { Stone, Wood }` (snake_case). `Item` gains `kind: ItemKind`, always serialized,
  after `pos`: `{"id":12,"pos":[1,2,3],"kind":"stone"}`. **Every item literal pin changes on purpose**
  (`WIRE` `:295-305`, `DELTA_WIRE` `:307-316`, the delta string at `:338`, the decode asserts `:449-450`, the
  Item literal table `:685-700`). Replace the `:249` NOTE.
- NEW `protocol::Refusal::Designate { kind: DesignationKind, rect: Rect }` gives
  `{"command":"designate","kind":"cut","rect":{…}}`, for a cut, dig or channel rect that marks nothing (Task 0.3).
- `Entity`, `Snapshot` and `Delta` keep their shape.
- **Save:** `SaveState.items` becomes `Vec<(u32, Pos, ItemKind)>`, and `DesignationKind::Cut` and
  `JobKind::Cut` serialize through the existing derives. A pre-12.7 save fails to decode, and simd refuses it
  (precedent `save.rs:35-40`, `serve.rs:939`).

## Acceptance Criteria

1. The real daemon is the judge. A tree marked for cutting is claimed only by a woodcutter. When he finishes,
   every tile of that tree is `empty` on the wire, and one wood item per trunk cell lies at its base
   (`timber_wire.py`: GREEN).
2. One tree is its trunk column plus the `TreeFoliage` in the 3×3 column around it, from the base to one
   above the top trunk cell. A scenario test cuts A and asserts every tile of B (whose crown touches A's)
   unchanged.
3. Items carry a kind (stone, wood) in the sim, on the wire and in the save. Wood is hauled to a stockpile
   like stone. A dug stone still has kind `stone`.
4. A `designate cut` marks every tree with a tile in the rect, with one mark at the trunk's base. Marks
   count toward `MAX_DESIGNATIONS`. A cut rect that marks no tree is refused on the next delta (12.1's shape).
5. A dig mark never lands on a `TreeTrunk`/`TreeFoliage` tile, and a channel mark never lands on a cell
   standing on one. A dig or channel rect that marks nothing is refused like a cut (Task 0.3).
6. A `cancel_designation` rect that holds the mark's base or any tile of the marked tree removes the cut mark
   and its job. A woodcutter holding that job lets go.
7. Determinism (standing AC 3): seed plus a command log with a cut gives identical state, `item_kinds()`
   included. A world saved mid-cut, with wood on the ground, loads and steps identically. A pre-12.7 save is
   refused.
8. gui: `5` arms cut mode. A drag sends exactly one `designate cut`, at the level Task 4 defines, and the
   preview lights the tree cells it will catch. A cut mark draws at the trunk's base in its own colour, and
   `4` (clear) removes it. A refusal shows in the refusal slot (`draft.md` §1).
9. gui: wood draws as a log, and a carried log sits at the carry offset. A woodcutter working a cut plays the
   clip Task 0.1 rules (`draft.md` §2-§3).
10. TUI (NFR10): a cut mark draws as `/`, wood as `=` (green on a stockpile cell), and the `marks:` line
    counts cut marks. No new input, no regression.
11. Instrument: `timber_wire.py` against a real daemon prints `TIMBER WIRE OK`. A `serve.rs` test asserts the
    same findings. The deliberate RED is mutation row 3 (the cut grabs the neighbour's crown): `tree B
    changed` and `TIMBER WIRE RED`.
12. At the seat, Wolf marks pines with `5`, watches Nain fell one whole and a hauler bring its log to a pile,
    sees the same in an attached tui, and clears a mark with `4`.

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings, 2026-10-05, at creation.**
  1. **Look draft APPROVED as drafted** (`12-7-signoff/draft.md`), with **(a)**: while a woodcutter works a
     cut, he plays the **Dig clip, facing the trunk**, as a placeholder until 12.8.
  2. **One log per trunk cell (3-5)**, all at the trunk's base. Wolf picked this over the recommended single
     log. Tree A has 4 trunk cells (z 12-15), so it leaves 4 logs. The gui stacks items that share a cell
     (Task 4); the tui still shows one `=` per cell.
  3. **A dig, channel or cut rect that marks nothing is REFUSED:** one `Refusal::Designate { kind, rect }`
     for all three kinds (standing AC 6). The side effect is accepted: over a forest edge, the treetop rows
     of a gui channel drag show `channel refused: nothing to channel` while the ground rows apply. A rect
     wholly out of bounds stays as today: it is dropped in the prelude (`lib.rs:1623-1668`), and no client
     can produce one.
  4. **Refusal texts as written:** `cut refused: no tree`, `dig refused: nothing to dig`, `channel refused:
     nothing to channel`. Each is a `&'static str` in `client_core::refusal_text`.
- [x] **Task 1: protocol (wire diff).** RED first: write the literal pins below before the types.
  - [x] `DesignationKind::Cut`, `DwarfJob::Cut { target: [i32; 3] }`, `ItemKind { Stone, Wood }`,
    `Item.kind`, `Refusal::Designate { kind, rect }`. `Refusal` stays `Copy`.
  - [x] Pins, as hand literals:
    - each new wire form parses and re-serialises byte-identical;
    - the item pins change to carry `"kind":"stone"`, and one wood literal is added;
    - `"kind":"cut"` joins the `designate` command table (`:510`) and the `DesignationKind` table (`:679`);
    - `cut` joins the `DwarfJob` table (`:636-649`).
- [x] **Task 2: sim-core (AC2-AC7). RED first: write the scenario tests against today's code and record the
  failures.**
  - [x] Vocabulary: `DesignationKind::Cut` (`:79`), `JobKind::Cut` (`:226`), and `trade(Cut) => Woodcutter`
    (`:435`). `pub enum ItemKind { Stone, Wood }` derives Serialize, Deserialize, Copy and Ord.
    `struct Item;` becomes `struct Item(ItemKind);`, so a spawn without a kind does not compile; fix every
    `(Item, …)` spawn (`:1154`, `:1533`, and the unit-test spawns `:2235-2997`). Add a new reader,
    `pub fn item_kinds(&self) -> Vec<(Id, ItemKind)>`, sorted by `Id`. `items()` keeps its signature: it
    has 56 call sites.
  - [x] The tree rule, in one private fn beside `work_positions`:
    `fn tree_of(terrain: &Terrain, pos: Pos) -> Option<(Pos, Vec<Pos>)>`. It returns the base and every tile.
    - A `TreeTrunk` at `pos` gives that column.
    - A `TreeFoliage` at `pos` gives the column of the `TreeTrunk` at the same z within Chebyshev 1, or else
      the one directly below (the tip). Anything else gives `None`.
    - The base is the lowest contiguous `TreeTrunk` of the column, and the top is the highest.
    - The tiles are the trunk cells, plus every `TreeFoliage` in the 3×3 around the column for
      `z in base.z ..= top.z + 1`.
    - `// NOTE:` the rule relies on `place_trees` keeping trunks 3 apart, and the gui's
      `incremental_tree_cover` uses the same 3×3 box. Foliage with no trunk in reach is no tree.
  - [x] `apply_command` `Designate` (`:1670-1690`):
    - Dig keeps `Solid(m)` only where `m` is not a tree material.
    - Channel keeps a standable cell only where the tile below is not a tree material.
    - Cut maps each in-rect tile through `tree_of` and marks each distinct base once, under the existing
      cap check (`:1684`).
    - When zero cells are applied, for any of the three kinds, return
      `Some(Refusal::Designate { kind, rect })` (Task 0.3). A re-mark of an existing mark counts as applied.
      Existing tests that designate over nothing and expect `None` flip to the refusal.
    - Leave the rect prelude (`:1623-1668`) byte-identical: rows in `3-1-give-the-order.sh` quote it.
  - [x] `CancelDesignation` (`:1692-1732`): also remove any `Cut` mark whose base is in the rect, or whose
    `tree_of(base)` has a tile in it. Add `JobKind::Cut` to the tile-job filter (`:1709`), so its job goes
    and its holder is released.
  - [x] `create_jobs` (`:333`): `Cut => JobKind::Cut`. `work_positions` (`:801`): `Cut` uses Dig's
    orthogonal-standable rule on the base.
  - [x] `execute_jobs`: NEW `pub const CUT_WORK_TICKS: u32 = 50;` beside `DIG_WORK_TICKS` (`:50`), with
    `// NOTE: tuned with 12.8's clip`. When the cut completes:
    - if `tree_of(target)` is `None`, take the existing remove-job/mark/release branch (`:1140-1145`);
    - otherwise `set_tile(Empty)` every tile, then `clear_paths`, then spawn one `Item(ItemKind::Wood)`
      per trunk cell at the base (Task 0.2), then remove the job and the mark, then `release_claim`.
  - [x] Dig and channel completion: tree targets can no longer arrive there, so remove the
    `yields_stone`/tree branch (`:1119`, `:1132`, `:1152`), and spawn `Item(ItemKind::Stone)`. Re-point or
    retire the mutation rows that quote it (Dev Notes, trap 1).
  - [x] Tests that pin the old behaviour flip:
    - `execute_jobs_digs_tree_materials_without_spawning_items` (`lib.rs:4083`) and
      `execute_jobs_channels_tree_materials_without_spawning_items` (`:4123`) become designation-filter
      tests: no dig mark on trunk or foliage, and no channel mark standing on either;
    - `trees_do_not_enclose_the_camp_from_outside_dig_work` (`scenario.rs:229`) becomes a cut of that same
      trunk by the woodcutter;
    - check `scenario.rs:63` (its 30 digs exclude trunks but not foliage, and it asserts the pile fills).
  - [x] NEW `sim-core/tests/scenario.rs` tests (DEFAULT_SEED, whose tree pair is measured):
    - `cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing`: mark A (73,59); step
      until A's tiles are all empty; assert every tile of B (73,56) equals its pre-cut tile, that exactly 4 items
      of kind `Wood` (A's trunk cells) lie at A's base, and that only Nain ever held the job (assert `claims()` each step);
    - `a_cut_is_hauled_to_the_pile_as_wood`;
    - `a_cut_over_no_tree_is_refused_and_a_dig_never_marks_a_tree` (also channel standing on a tree);
    - `a_cancel_touching_a_marked_tree_removes_its_mark_and_releases_the_woodcutter`;
    - a cut against a full cap (`scenario.rs:584` is the pattern): refused, with no mark added.
  - [x] Determinism: `same_seed_and_commands_remain_deterministic` (`scenario.rs:1970`) gains a cut, and
    asserts `item_kinds()` each step. `save_load.rs`: the gate test (`:9-180`) compares `item_kinds()`. A new
    round trip saves while Nain holds a cut and wood lies on the ground.
- [x] **Task 3: simd + client-core (AC1, AC3, AC4, AC7).**
  - [x] `bridge.rs`, all exhaustive `match`es with no wildcard:
    - `designation_kind_in/out` (`:257`, `:264`): `Cut`;
    - `dwarf_job` (`:175`): `Cut { target }`;
    - NEW `item_kind_out`, which the item lists use (`:35-42`, `:87-94`, via `item_kinds()`);
    - `refusal_out` (`:285`): `Designate`.
  - [x] `main.rs` `Designate` arm (`:224-229`): extend `refusals` like `PlaceStockpile` (`:235-238`). Load
    validation (`:404-485`, `:636-653`): `JobKind::Cut` and `DesignationKind::Cut` arms, checked as Dig is.
  - [x] `simd/tests/serve.rs`: NEW `a_cut_fells_one_whole_tree_into_wood_and_a_cut_over_nothing_is_refused`.
    It mirrors `timber_wire.py`: the same commands through `send_literal` (`:171`), with assertions on
    deltas. A save without item kinds is refused (copy `serve.rs:939`).
  - [x] `client-core` `refusal_text` (`:10`): the `Designate` arms (Task 0.4), pinned in
    `every_refusal_has_its_one_text` (`:376`). Fix the `Item { id, pos }` fixtures (`:425-428`,
    `client-core/tests/mirror.rs`).
- [x] **Task 4: gui (AC8, AC9). Look per Task 0.1.**
  - [x] `designate.rs`: `DesignateMode::Cut` on `Digit5`, plus `mode_key` (`ingest.rs:1540`), `parse_drag`'s
    `cut` (`:1456`), the keymap test table (`:4809-4812` area), the hint strings (`:83-97`) and
    `every_hint_is_ascii` (`:405`).
  - [x] The cut target, in `designation_target` (`:222`): the picked tile if the mirror holds
    `TreeTrunk`/`TreeFoliage` there, else the cell above it. `commands_for` (`:258`): `Cut` gives ONE
    `Designate { Cut, rect_on_level(anchor xy, end xy, target z of the anchor) }`. `Clear` gains one more
    `CancelDesignation` on that same cut-target rect, so a clear drag at a pine's foot reaches its trunk.
    - **One command per cut drag**, so a drag can raise at most one refusal (12.1's split-drag lesson).
  - [x] Preview (`project.rs:913-983`): `sim_will_keep` for `Cut` is "the cell is a tree tile". Mark:
    `designation_color` (`appearance.rs:202`) gets a cold `Cut` literal that passes
    `mark_colours_are_distinct_cold_literals` (`:667`). Add it to that test's `marks` array, keep it at
    least 50 from every TUI mark colour (`:189-201`), and add `designation_material` /
    `designation_mark_transform` arms (`project.rs:2236`, `:2813`, assets `:714-722`). Slab at the base cell.
  - [x] `capture.rs:170` `assert_drag_produced_work`: `Cut` asserts `expected_designations > 0`.
  - [x] Items (`project.rs:2091-2097`): branch on `item.kind`. Wood is a log (NEW `WOOD_ITEM_SCALE: Vec3`
    and a wood colour, both in `appearance.rs` beside `STONE_ITEM_SCALE`, `:233`). The carry branch
    (`:533-547`) uses the same per-kind scale.
  - [x] Stacking (Task 0.2): items that share a cell draw stacked by ascending id, the n-th one log height
    above the first (`item_translation`, `:2202`). This applies to every item kind; for stones it only
    changes #154's two-on-one-cell case.
  - [x] Clip (Task 0.1): `dwarf_clip` (`:425-437`) returns Dig for a `DwarfJob::Cut` in Work, and
    `dig_yaw` (`:2458-2467`) faces its target. `// NOTE: placeholder until 12.8's Cut clip.`
  - [x] Headless tests (`tests/headless.rs`):
    - a `Digit5` drag over a trunk writes exactly one `designate` with `"kind":"cut"` at the trunk's
      level, and a ground-level drag at a tree's foot writes it one level up (`drag_one_tile`, `:4020`);
    - a clear drag at the foot writes a cancel rect that holds the base;
    - a cut mark projects at the base with the cut material;
    - a wood item projects as a log, and a stone still as a stone (`snapshot_item_receives_a_render_mesh`,
      `:2041`, is the pattern); four logs on one cell draw at four distinct heights;
    - a woodcutter on a `cut` job in Work gets the Dig clip.
- [x] **Task 5: tui (AC10).**
  - [x] `palette.rs`: `designation_cell(Cut)` is `/` (226,96,64). Add a `wood_item_cell` (`=`,
    (164,116,66)) and its stored twin (zone green), and add both to `every_look_is_pinned`'s `markers`
    (`:298-359`).
  - [x] `view.rs`: the item layer (`:315-324`) picks the cell by kind. `tally_marks` (`:99-103`) counts the
    cut glyph.
  - [x] Tests:
    - a view test that a wood item draws `=` and a stored one draws it green;
    - `the_mark_tally_reports_what_the_frame_could_not_show` (`:725`), or a sibling, with a cut mark;
    - a `tui/tests/client.rs` capture with a `Designate` refusal on the status row (the
      `capture_refusal_frames` pattern, `:205`).
- [x] **Task 6: the record.**
  - [x] `mutations/12-7.sh`. Each row is KILLED by the test named:
    1. the cut does nothing → `cutting_one_tree_fells_it_whole…`;
    2. the cut removes the trunk only → the same test's every-tile-empty assert;
    3. the crown box is 5×5 (it takes B's crown) → the same test's neighbour assert, and the live recipe's RED;
    4. `trade(Cut) => Miner` → the only-Nain assert;
    5. the wood spawns as `Stone` → the kind assert, and the serve test;
    5b. one log per tree, not per trunk cell → the exactly-4 assert;
    6. the dig filter takes tree tiles again → the filter test;
    7. the channel filter takes cells on trees → the filter test;
    8. a no-tree cut is not refused → the refusal test, and the serve test;
    9. cancel leaves the `Cut` job → the cancel test;
    10. the bridge sends wood as `stone` → the serve test;
    11. the save drops the kind (wood reloads as stone) → the save round trip;
    12. gui cut mode sends `dig` → the drag test;
    13. gui clear omits the cut-target rect → the clear test;
    14. gui wood draws as a stone → the projection test;
    14b. gui items sharing a cell all draw at the same height → the stacking test;
    15. tui wood uses the stone glyph → the view test.
  - [x] Spine `architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md` AD-16 (`:121-144`): add an "Amended
    2026-10-xx (Story 12.7)" paragraph.
    - Dig and channel never take tree tiles.
    - A cut job removes one whole tree (the rule above) and yields wood.
    - `protocol::Item` gains `kind`, an M3 vocabulary growth.
    - The 2026-08-09 "drops no item / wood deferred" sentence is superseded, not deleted.
  - [x] README: the gui key row (`:202`) gains `5` cut. The flags table gains `--drag … cut` (`--drag`
    itself is missing there, `:266-286`). The tui legend (`:64-73`) gains `/` and `=`.
- [x] **Task 7: the live recipe, the seat (AC12), then the full gate.**
  - [x] Run the Verification recipe: GREEN, then the deliberate RED (row 3). Record both outputs.
  - [x] Write `12-7-signoff/vehicle-card.md` in the seat's launch form. Say where to look: the hint bar is
    bottom-left, the marks are at the pines' feet, and the log is at the foot of the felled pine.
  - [x] Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` (about 55 min, [[gate-ooms-at-default-parallelism]]).

### Review Findings

Code review run 1, 2026-10-05, on `fb726ce` (diff `fd9ca98..HEAD`, 19 code files). There were four layers
and none timed out. Blind Hunter and Edge Case Hunter ran on Sonnet with R1 territories. The Acceptance and
Feature Auditors ran on Opus over the whole diff. Every layer's cargo ran (1.97.1), and each built in its own
target dir. Live runs:
- `timber_wire.py` gave `TIMBER WIRE OK` on two fresh daemons. A empty at t272, 4 wood on the pile, B
  unchanged.
- The row-3 mutant, rebuilt from a source copy under `/tmp`, gave `TIMBER WIRE RED` (`tree B … changed:
  tick 272`).
- Save/load mid-cut with wood on the ground resumed and felled B. A kind-less save was refused.
- The real gui, paused mid-cut, printed `gui dwarf 0 clip dig`.
- A Blind Hunter probe felled every tree on seeds 42/7/99. Logs equalled the trunk cells cut, every one wood.
- `fmt`, `clippy -D warnings` and every non-ignored test were green on HEAD.

The pixel guards were NOT re-run on HEAD. The full gate is last green on `a7b1ded`, before `aed8c5b`
(#168), and must re-run before the PR. No HIGH or MED code defect was found. Tally: 2 decision-needed
(both resolved), 3 patch, 8 defer, 12 dismissed. Layer and severity are in brackets.

- [x] [Review][Decision] **At a full designation cap, the refusal names the wrong cause** (blind + feature,
  LOW-MED; a misreported error, the latent class). Cap-skipped cells count as not applied
  (`sim-core/src/lib.rs:1794`). So at `MAX_DESIGNATIONS` (4096; one 64×64 dig drag reaches it), any drag
  over real trees or rock reads `cut refused: no tree` / `dig refused: nothing to dig`. The texts are Task
  0.4's ruling, and `a_cut_against_a_full_cap_is_refused…` pins the refusal but not what it says.
  Options: (a) widen the three texts (`… : no tree, or the mark limit is reached`); (b) a distinct cause on
  the wire (a refusal field, i.e. a protocol change); (c) accept.
  — RESOLVED (Wolf, option a): widen the three texts. Becomes the patch below.
- [x] [Review][Decision] **AC12's record does not itemise tui parity or the `4` clear** (acceptance, LOW;
  record). The pass rests on "works .. but cut trees have some blocks in the air" and then "that works now".
  The second visit was "for the floating blocks only". Confirm whether both were seen at the seat.
  — RESOLVED (Wolf, 2026-10-05): he ran the tui beside the gui, both with compasses. The parity check
  found that the gui is a MIRROR image of the tui: north is up in both, but the hauled logs are left in
  the gui and right in the tui. Measured with the gui's own projection: screen `perp_dot(east, north)` is
  -1.000 in the tui and +0.87..+0.99 in the gui at all 8 yaws, so the handedness is opposite at every
  angle. `world_to_render` `(x,z,-y)` is a proper rotation of a left-handed sim frame. This predates 12.7
  (the gui transform and the north-only compass); filed as **#169** (bug, route:story, route:undecided).
  It is not a 12.7 defect: both clients show the same felled tree, mark and log. The `4` clear was NOT
  observed (Wolf: "no because got confused"), so AC12's "clears a mark with `4`" stays OPEN until it is
  seen at the seat.

Patches: Wolf chose to LEAVE THEM AS ACTION ITEMS (2026-10-05), for a `/bmad-review-patch` session. Before
the PR: the 3 patches, the `4` clear at the seat, and the full gate on the final HEAD.
Review cost $26.45 / 483 turns (subagents 78.1%; it also carries the launcher fix's few turns). Reaped
140.7 GB of /tmp layer caches (70.7 GB of free space reclaimed).
- [x] [Review][Patch] **Widen the three `Refusal::Designate` texts to name the cap** (from D1; blind +
  feature, LOW-MED, misreported error). `cut refused: no tree, or the mark limit is reached` and its dig and
  channel twins, re-pinned in `every_refusal_has_its_one_text`. [`crates/client-core/src/lib.rs:14`]
  — LANDED `660c8f4`. The tui status row clips the tail below ~95 (cut) to ~110 (channel) columns. Wolf
  ruled on 2026-10-06 to land the texts as ruled and record the limit in a `// NOTE:` at `refusal_text`.
- [x] [Review][Patch] **The story's tui recipe `/` count cannot fail** (acceptance, LOW; a broken
  instrument, the latent class, CONFIRMED). `grep -o '/' f2.txt | wc -l` gives 2 with a mark and 1 with
  none: the status row's `z 13/31` carries a `/`. The dev fixed the same collision in `tally_marks` but not
  in the recipe. Count the mark colour (`38;2;226;96;64m/`) instead.
  [`12-7-timber.md` Verification, the tui recipe's `grep -o '/'` line] — LANDED `ce737d6`.
- [x] [Review][Patch] **`items_out` pairs ids with kinds by position and checks only with `debug_assert_eq!`**
  (edge, LOW; latent silent trap). `items()` keeps an item only if it has a `Pos`; `item_kinds()` keeps
  every `Item`. One item without a `Pos` would shift every later kind in a release daemon, with no log.
  Unreachable today. Look the kind up by id instead of zipping. [`crates/simd/src/bridge.rs:88`]
  — LANDED `9d96a7d`.

**Review patch pass 1, 2026-10-06** (`660c8f4`..`47c2f5c`). Every patch was RED before its fix. Mutation
rows 18 and 19 were run as a focused scratch table: both KILLED; exclusivity is unverified. The full gate
(`RUST_TEST_THREADS=1 scripts/gate.sh`) was GREEN on `47c2f5c`, exit 0, in 3284 s, pixel guards
included. Closure table (no REWORK rows: this is round 1's first patch pass):

| Item | Written for | Then tested | Pre-existing-state fixture |
| --- | --- | --- | --- |
| Refusal texts | the text source (`refusal_text`) | the display side: gui HUD `ingest.rs:4018` asserts HUD == `refusal_text` exactly; tui `a_designate_refusal_shows_on_the_status_row` run with the full text pinned | the tui's 80-column headless frame. It went RED with the full text pinned (`…or the mark limit ` clipped), so that test keeps its old `contains("cut refused: no tree")` claim and the clip is recorded as a NOTE, not hidden |
| `items_out` join | a world holding an item with no `Pos` | release AND debug builds; release showed the silent mispairing (item 3 → `Wood`) | `an_item_without_a_position_does_not_shift_later_kinds`: ids 1 and 3 positioned, id 2 kind-only. The normal side (every item positioned) is serve.rs plus rows 5/8/10, all in the green full gate |
| tui recipe count | a frame WITH a cut mark | a frame with NO mark, on a fresh release `simd 7713` (DEFAULT_SEED) | the status row's `z 13/31`: bare `/` count 1 unmarked / 2 marked, mark-colour count 0 / 1. The same line's `=` count was checked too: 0 `=` in the chrome, so it is sound |
- [x] [Review][Defer] **Stacked items hop one step while a carrier stands on their cell** (blind + feature
  + acceptance, LOW, cosmetic; new in 12.7). `item_stacks` counts carried items, whose wire `pos` is the
  carrier's cell, so a hauler crossing a pile cell lifts a higher-id log by 0.28. The gui knows `carrying`
  and could skip those items. [`crates/gui/src/project.rs:2262`] — deferred: LOW cosmetic; the LOW-tail cap.
- [x] [Review][Defer] **A pile of 4-5 logs pokes out of its cell** (blind, LOW, cosmetic). The 4th log's top
  is 1.12 cells above the floor, inside the cell the felled trunk vacated. Seen and passed at the seat.
  [`crates/gui/src/appearance.rs:251`] — deferred: LOW cosmetic.
- [x] [Review][Defer] **`item_stacks` re-sorts all items every frame, and once per released item**
  (blind, LOW). n is in the hundreds. [`crates/gui/src/project.rs:566,2625`] — deferred: no measured cost.
- [x] [Review][Defer] **The loader accepts a cut mark that is not a tree base** (edge, LOW, RAN). With a
  hand-edited save, a mark on air is "worked" and vanishes with no wood, and a mid-trunk or foliage mark
  idles the woodcutter forever. The `JobKind::Cut` arm checks the kind only, as Dig's does.
  [`crates/simd/src/main.rs:641`] — deferred: only a hand-edited save reaches it.
- [x] [Review][Defer] **The bridge's "every designation kind" table lacks `Cut`, and nothing unit-pins
  `refusal_out(Designate)` or `item_kind_out`** (acceptance, LOW). Covered end to end by the serve test and
  rows 5/8/10. [`crates/simd/src/bridge.rs:381`] — deferred: covered by serve.rs.
- [x] [Review][Defer] **`--drag` is dead headless, and its new README row does not say it needs a window**
  (feature, LOW, RAN; pre-existing). `--drag cut` and `--drag dig` both panic `scripted --drag never
  completed`, so `assert_drag_produced_work(Cut)` runs only at the seat. It is loud, not silent.
  [`README.md:271`] — deferred: pre-existing, loud.
- [x] [Review][Defer] **A lagging walker can miss the whole cut swing** (feature, LOW, RAN in lavapipe).
  Nain never printed `clip dig` during his 50 work ticks at ~1 fps, though the cut completed. He does when
  the world is paused. This is #164's gating (`DwarfClip::Dig if !arrived`), and it is 0.25 s at fast4x.
  [`crates/gui/src/project.rs:509`] — deferred: #164 / 12.8's clip tuning.
- [x] [Review][Defer] **The approved draft says felling takes "10 s … like a dig"** (acceptance, LOW;
  record). `CUT_WORK_TICKS = DIG_WORK_TICKS = 50` is 5 s at Normal, and the vehicle card says ~5 s.
  [`_bmad-output/implementation-artifacts/12-7-signoff/draft.md`] — deferred: signed-off artifact; the code
  matches "like a dig".

Dismissed (12):
- An out-of-bounds single-z rect is silent. Task 0.3 ruled it, and the gui cannot reach it, because
  worldgen clamps ground at `dims.z - 2`.
- Multi-z and inverted rects are stderr-only. Pre-existing.
- The new tally test misses the `take(map_cells)` guard. The older tally test kills it.
- A cut with no woodcutter is silent. A design limit, as for dig.
- A cut drag takes one z from the anchor. Design; the preview shows it.
- `Item.kind` is required on the wire. The spec says "always serialized".
- Dig and channel now refuse. Task 0.3.
- A log is carried crosswise. The approved look.
- The roster note. Not a defect.
- The cancel note. Not a defect.
- The gate on HEAD. A known pre-PR action, not a finding.
- The protocol NOTE was dropped, not replaced. Moot, since `kind` now exists.

**Code review run 2, 2026-10-06** — the patch-pass diff `8c3764f..7499534` only (round 2 for this diff).
Four layers, none timed out: blind (Sonnet, client-core), edge (Sonnet, simd + tests + mutation rows),
acceptance and feature (Opus, whole diff). Each ran cargo in its own target dir.

**Delta against run 1:** 3 NEW findings (all LOW, records), 1 REWORK (the closure table's refusal-texts row
overclaims its display-side test; the code fix itself holds), 1 pre-existing defect found live and filed
as **#170**. Severity: 0 HIGH, 0 MED. **Stopping rule:** none of the new findings is HIGH, so the static
audit ENDS here. The next spend goes to the seat.

**Observed live this run** (feature, release `simd`, DEFAULT_SEED):
- D1's outcome is now observed. With 4096 marks, a cut on tree B (73,56,13) is refused on the wire and the
  tui shows `cut refused: no tree, or the mark lim` (clipped at 80 columns). Control: cancel one mark (4095)
  and the identical cut is accepted.
- After felling A beside the 12.5 channel block, items 10-13 are `stone` and 14-17 are `wood` on the wire,
  with none mispaired.
- Re-proven by acceptance: both REDs (old cut text, zip join in release) and the recipe's 0/1 and 1/2 counts.
- Mutation rows 18 and 19 apply once each to the current source (edge, read-only count).
- NOT observed: the gui HUD showing the widened text (vehicle card step (d) will be the first time), and
  AC12's `4` clear.

- [ ] [Review][Patch] **The closure table's refusal-texts row overclaims its display-side test, and
  misnames the 80 columns** (acceptance + blind, LOW, REWORK; a misreported verification).
  - `ingest.rs:4018` sends a `Refusal::PlaceStockpile`, not a `Designate`, and compares the HUD against
    `refusal_text(&expected)` itself, so it pins no wording.
  - The tui test keeps `contains("cut refused: no tree")`, which mutation row 18 would leave green.
  - So the new texts are pinned only at their source, and the NOTE's "the gui HUD shows it in full" has no
    test behind it.
  - The 80 columns are this devpod's `tput cols`, not the tui: `frame_size()` falls back to 100×40, and the
    full-text pin goes GREEN at `COLUMNS=120` or with `TERM` unset. Keeping the prefix is right, because a
    full pin would be venue-flaky.
  - Correct the row and the NOTE wording.
  [`12-7-timber.md:357`, `crates/client-core/src/lib.rs:14`]
- [ ] [Review][Patch] **Task 0.3, Task 0.4 and the draft still quote the old texts, with no pointer to D1**
  (acceptance, LOW; the partial-doc-update shape). They are records of rulings, so the old texts stay, but
  each needs a one-line "superseded by D1 (2026-10-05)" note.
  [`12-7-timber.md:130,133`, `12-7-signoff/draft.md:46`]
- [x] [Review][Defer] **At the mark cap, a designate that overlaps existing marks drops its new cells
  silently** (feature + blind, LOW, RAN; pre-existing since 3.2's cap). An already-marked cell counts as
  applied, so `applied > 0`, there is no refusal, and the new cells are dropped. Live: a dig over
  (0..3,0,0..1) at 4096 returned `refusals []` and marked 0 new z 1 cells. D1 only surfaces the all-new
  case. [`crates/sim-core/src/lib.rs:1794`] — deferred, pre-existing; filed **#170** (route:undecided:
  reporting a partial apply is a wire change).

Dismissed (5):
- The tui clips the cap clause at 80 columns. Already ruled: land as ruled, plus a NOTE.
- `kinds[&id]` could panic. `items()` ids are a strict subset of `item_kinds()` ids
  (`lib.rs:2005-2026`), and a panic is louder than the old silent mispair.
- Mixed-kind items are only covered transitively on the wire. `serve.rs:2779` asserts `Wood` on the wire.
- Text casing and formatter-split arms. Cosmetic.
- `tui --frame` cannot show a refusal. That was the orchestrator's prompt error; `tui/src/main.rs:409`
  documents it, and `--frames N` does show one.

Patches: Wolf chose to LEAVE THEM AS ACTION ITEMS (2026-10-06). Review cost $7.23 / 170 turns (subagents
58.5%). Reaped 5.6 GB of /tmp layer caches (3.1 GB of free space reclaimed).

## Dev Notes

### Scope guardrails (do NOT)

- No falling trees, and no fix for trees floating over a dug block (12.11, #135).
- No cut animation: 12.8 makes it. Task 0.1's clip is a placeholder.
- No new `Command` variant, no new tui input (NFR10), and no new `Entity` field.
- No minerals or gems (12.10). `ItemKind` gets exactly `Stone` and `Wood`.
- No change to claiming (AD-12): `Cut` adds its variant, its execution and its `trade` arm only.
- No fix for #162 (items never block), #154 (two items on one pile cell) or #164 (visuals trail the walker).
  Wood inherits all three.

### What already exists (build on it)

- 12.1's refusal path: `Option<Refusal>` → `bridge::refusal_out` → `Delta.refusals` (broadcast) → gui
  `LastRefusal` / tui status row → `client_core::refusal_text`.
- Hauling, the pile's "stored" rule, `carry_items` and `release_claim` are kind-blind: wood rides them
  unchanged.
- The gui retires and respawns a tree mesh per dirty trunk column (`incremental_tree_cover`), so a felled
  pine vanishes with no new mesh code. Its `tree_mesh_might_cover` test (`project.rs:5061`) uses the same
  3×3 box.
- `release_claim`, `MAX_DESIGNATIONS` and the 12.5 `dwarf_clip` stderr line (`gui dwarf N clip …`,
  `project.rs:530`).
- `timber_wire.py`, the daemon-judged recipe, RED at creation.

### Key decisions & traps

1. **The dig change rots old mutation anchors.**
   - `3-2-the-dig.sh` quotes the `yields_stone` tuple, the stone spawn, the Dig filter `Some(Tile::Solid(_)))`
     and `DesignationKind::Channel => terrain.is_standable`.
   - `5-1-the-world-grows-things-that-glow.sh` row 1 is `if yields_stone || true`; its behaviour is
     superseded, so retire it with a reason line.
   - `3-3`/`12-5` quote `JobKind::Dig | JobKind::Channel`.
   - Run `scripts/audit-mutations.py` (after `cargo fmt`). Re-point each row to the new seam and re-kill it
     alone, or retire it with a reason. An APPLY-FAILED is a finding, not noise ([[stale-sabotage-literal]]).
2. **One mark per tree, at its base.** The base is a `TreeTrunk` tile, so no dig or channel mark can share
   its `Pos`, and `Jobs.targets` stays unique.
3. **Mark and cancel use one rule:** "the rect holds the base or any tile of the tree". If cancel matched
   only the base, a gui clear drag, which targets surface cells, would never reach a mark.
4. **A crown flood fill is wrong.** It passes on an isolated tree and fells B with A. Row 3 holds the line,
   with B measured.
5. **The live recipe is pinned to DEFAULT_SEED's trees** (A (73,59), B (73,56), D (65,56)) and asserts the
   trunks exist. A worldgen change that moves them makes it fail loudly.
6. **An untracked draft mutation table blocks every commit** (12.4). Draft rows outside `mutations/` until
   they are ready.

### Project Structure Notes

- UPDATE `crates/protocol/src/lib.rs`; `crates/sim-core/src/{lib.rs, save.rs}`;
  `crates/sim-core/tests/{scenario.rs, save_load.rs}`
- UPDATE `crates/simd/src/{main.rs, bridge.rs}`, `crates/simd/tests/serve.rs`
- UPDATE `crates/client-core/src/lib.rs`, `crates/client-core/tests/mirror.rs`
- UPDATE `crates/gui/src/{designate.rs, project.rs, appearance.rs, ingest.rs, capture.rs}`,
  `crates/gui/tests/headless.rs` (and the `Item` literals in `tests/bench_contract.rs`, if any)
- UPDATE `crates/tui/src/{palette.rs, view.rs}`, `crates/tui/tests/client.rs`
- UPDATE `ARCHITECTURE-SPINE.md` (2026-08-09, AD-16), `README.md`
- UPDATE the old mutation tables per trap 1; NEW `mutations/12-7.sh`, `12-7-signoff/vehicle-card.md`
- EXISTING (creation) `12-7-signoff/timber_wire.py`, `12-7-signoff/draft.md`

### Previous story intelligence

- 12.6's patterns carry over: hand-literal wire pins, exhaustive bridge `match`es, and a `serve.rs` test that
  mirrors the creation-time Python recipe. Its live recipe's first GREEN failed on the INSTRUMENT: a command
  lands a few ticks after the send (AD-10). Judge from the first delta that shows the effect, never from the
  send tick.
- `cargo test -p gui` does not rebuild `simd`. A daemon-side RED for a gui real-binary test is a
  scratch-worktree run, not a `mutate.sh` row. Kill test daemons by PID, never `pkill -x simd`.
- Branch pushes use `push.sh --fast`. The FULL gate runs only before the PR
  ([[push-branch-for-vehicle-testing]]).

### References

- `epics.md:2590-2615` (Story 12.7), `:2422-2434` (standing ACs); PRD
  `prds/prd-frostvein-2026-09-28/prd.md:76-78` (FR41), `:87-88` (FR44), `:132-137` (NFR10, NFR11)
- `ARCHITECTURE-SPINE.md` (2026-08-09) AD-16 `:121-144`; (2026-08-01) AD-6, AD-8 (`set_tile` dirty
  tiles), AD-10 `:151`, AD-11, AD-12 (amended 12.4); the "Command acknowledgement" convention
- Stories 12.1 (refusal shape, one command per drag), 12.4 (trades), 12.5 (`DwarfJob`, clips), 12.6
  (record format)
- No new dependency. No version research needed: no crate changes, and Bevy stays on 0.19.

## Verification

**Wire (runs today; RED observed at creation):**

```bash
target/release/simd 7703 &   # fresh daemon, DEFAULT_SEED; kill it by PID afterwards
python3 _bmad-output/implementation-artifacts/12-7-signoff/timber_wire.py 7703
```

- **RED** at creation, on `main` `fd9ca98`, release `simd`:
  - the header: `tree A base [73, 59, 12] (21 tiles), tree B base [73, 56, 13] (20 tiles), crowns touch:
    True; dig over D's trunk [65, 56, 10]`;
  - `cut mark at A's base: NEVER`, `a woodcutter holds the cut: NEVER`, `every tile of A empty: NEVER`,
    `wood item at A's base: NEVER`, `wood item on a pile cell: NEVER`;
  - `tree B (touching crown) changed: no`, `cut over no tree refused: NEVER`;
  - `dig mark placed on tree D: tick 15`, `tree D changed: tick 208`;
  - `TIMBER WIRE RED`, exit 1. The daemon log shows `unrecognized client message` for both cut commands.
- **GREEN, required of dev:**
  - every NEVER becomes a tick: the cut mark, Nain holding the cut, every tile of A empty, wood at the base,
    wood on the pile, and the no-tree refusal. Record how many wood items appeared at A's base: 4, one per
    trunk cell (Task 0.2);
  - `tree B … changed: no`, `dig mark placed on tree D: no`, `tree D changed: no`;
  - `TIMBER WIRE OK`, exit 0;
  - record every tick.
- **Deliberate RED, required of dev:** row 3 (the 5×5 crown box), with release `simd` built in a scratch
  worktree, gives `tree B (touching crown) changed: tick N` and `TIMBER WIRE RED`.

**tui (runs today; the cut half is RED until the feature exists).** Run it after the GREEN run, on the same
daemon. `--frame` (one frame) is the mode that prints the `marks:` tally to stderr; `--frames N` does not.

```bash
target/release/tui 7703 --frame --z 13 > "$SCRATCH/f1.txt" 2> "$SCRATCH/m1.txt"   # before: 0 of 0
python3 -c 'import socket,json; s=socket.create_connection(("127.0.0.1",7703)); f=s.makefile(); f.readline(); s.sendall(b"{\"type\":\"set_speed\",\"speed\":\"paused\"}\n{\"type\":\"designate\",\"kind\":\"cut\",\"rect\":{\"min\":[73,56,13],\"max\":[73,56,13]}}\n"); f.readline(); f.readline()'
target/release/tui 7703 --frame --z 13 > "$SCRATCH/f2.txt" 2> "$SCRATCH/m2.txt"   # B marked
target/release/tui 7703 --frame --z 9  > "$SCRATCH/f3.txt" 2> /dev/null            # the pile
grep -o '38;2;226;96;64m/' "$SCRATCH/f2.txt" | wc -l; grep -o '=' "$SCRATCH/f3.txt" | wc -l
```

- **Observed at creation** (fresh release `simd 7706`, `fd9ca98`):
  - before: `marks: z 13 designations=0 of 0`;
  - after the cut: still `0 of 0`, because the daemon logs `unrecognized client message`;
  - the instrument's CONTROL: a dig mark on B's base (accepted today) gives `designations=1 of 1`,
    `span x[73..73] y[56..56]` and one `×` in the frame. The tally sees a mark at that cell, so `0 of 0`
    after the cut is the feature missing, not the instrument.
- **Required of dev:**
  - after the cut, `designations=1 of 1`, with `span x[73..73] y[56..56]` and exactly one cut-mark `/`
    (`38;2;226;96;64m/`) in `f2.txt`. A bare `/` count cannot fail: the status row's `z 13/31` carries one
    (review patch, 2026-10-06: bare count 1 unmarked / 2 marked, mark-colour count 0 / 1);
  - at z 9, at least one `=` in `f3.txt` (in stockpile green): the log the GREEN run delivered;
  - zero of either is a failure, whatever the exit code.

**Seat (AC12), in the seat's launch form:**

- WSL: `simd 7451`, and in a second WSL terminal `tui 7451` for parity.
- PowerShell: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`.
- Press `3` and drag a 3×3 stockpile west of the fire, then `Esc`. Press `5` and drag across the foot of the
  pines nearest the fire (A and B stand at x 73, y 56-59, about 11 tiles out; the cursor readout gives the
cell). Read the marks at their feet. Nain (purple)
  walks out, works, and one whole pine is gone; a log lies at its foot, and a hauler brings it to the pile.
- Press `4` and drag over the foot of a still-marked pine: its mark goes.

## Dev Agent Record

### Agent Model Used

- Orchestrator + verifier: Claude Opus 5.5 (`claude-opus-5-5[1m]`). Wolf picked Sonnet 5.5 subagents
  for the implementation (2026-10-05), as in 12.2-12.6. They ran one after the other, on one tree.
- Agent A (Sonnet 5.5): Tasks 1, 2 and 3, plus re-pointing the old mutation rows its changes broke.
  Agent B (Sonnet 5.5): Tasks 4 and 5, plus the rows its changes broke.
- The orchestrator wrote `mutations/12-7.sh`, the spine and README edits and the vehicle card. It ran
  every mutation, the live recipe and its deliberate RED, and the full gate.

### Debug Log References

- **Task 1 RED** (Agent A): the hand-literal pins did not compile before the types existed (13
  errors: `ItemKind` undeclared, `Item` has no field `kind`, …).
- **Task 2 RED** (Agent A, against the vocabulary alone: `Cut` filtering to nothing, `JobKind::Cut`
  completing as a no-op):
  - `cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing`: `one mark, at the
    base  left: []  right: [(Pos{73,59,12}, Cut)]`;
  - `a_cut_is_hauled_to_the_pile_as_wood`: `all four logs reach the pile  left: 0 right: 4`;
  - `a_cut_over_no_tree_is_refused_and_a_dig_never_marks_a_tree`: `left: None right:
    Some(Designate{kind: Cut, rect: (62,62,9)-(63,63,9)})`. It failed on its first assert, so its dig
    and channel asserts were not RED-exercised there; rows 6 and 7 kill the filter tests instead;
  - `a_cancel_touching_a_marked_tree_removes_its_mark_and_releases_the_woodcutter`: `left: 0 right: 2`;
  - `a_cut_against_a_full_cap_is_refused_and_adds_no_mark`: `left: None right: Some(Designate{Cut, …})`;
  - flipped: `execute_jobs_digs/channels_tree_materials_without_spawning_items` failed as expected
    after the filter change (`dug TreeTrunk left: 1 right: 0`) and became
    `a_dig_mark_never_lands_on_a_tree_tile` / `a_channel_mark_never_lands_on_a_cell_standing_on_a_tree_tile`;
    `trees_do_not_enclose_the_camp_from_outside_dig_work` failed (`left: Some(Solid(TreeTrunk)) right:
    Some(Empty)`) and became `…_outside_cut_work`.
- **Task 3 RED** (Agent A): `a_cut_fells_one_whole_tree_into_wood_and_a_cut_over_nothing_is_refused`
  against a bridge that mapped `Cut` to `Dig` and dropped the refusal: `no cut mark at A's base`.
- **Task 4 RED** (Agent B), gui lib: `a_cut_drag_is_one_designate_cut_on_the_cut_target_rect` (`left:
  []`); `clear_reaches_both_the_picked_cell_and_the_standable_one` (fourth cancel missing);
  `mark_colours_are_distinct_cold_literals` (`cut must retain its named colour left: [56,132,250]`);
  `mark_counts_are_checked_against_the_mirror_not_merely_against_zero` (`a cut drag that produced no
  designation must fail`); `a_scripted_drag_can_name_the_cut_mode` (`invalid --drag mode`);
  `the_cut_preview_lights_only_tree_tiles_at_the_cut_level` (`empty air beside a trunk must not light`).
  Headless: `a_cut_drag_over_a_trunk_writes_one_designate_cut_at_the_trunks_level` (`left: []`);
  `a_cut_drag_at_a_trees_foot_writes_it_one_level_up` (`left: []`);
  `a_clear_drag_at_a_trees_foot_writes_a_cancel_that_holds_the_base` (last cancel `[1,1,1]`, not
  `[1,1,2]`); `a_cut_mark_projects_at_the_base_with_the_cut_material` (`got 1.54`, Dig's placement);
  `a_wood_item_projects_as_a_log_and_a_stone_still_as_a_stone` (`a log, not a cube left:
  Vec3(0.4,0.4,0.4)`); `four_logs_on_one_cell_draw_at_four_distinct_heights` (`log 10 … sits at -0.3,
  wanted -0.36`); `a_carried_log_is_drawn_at_the_log_scale`; `a_woodcutter_on_a_cut_job_in_work_gets_the_dig_clip`
  (`left: Walk right: Dig`); `a_woodcutter_working_a_cut_faces_the_trunk` (`drew Quat(0,0,0,1)`).
- **Task 5 RED** (Agent B): `every_look_is_pinned` (`glyph '*' … right: '='`);
  `a_wood_item_draws_as_a_log_and_a_stored_one_in_the_stockpile_colour` (`left: '*' right: '='`);
  `the_mark_tally_counts_a_cut_mark` (no `/`). `a_designate_refusal_shows_on_the_status_row` had NO
  RED: Agent A's `refusal_text` already carried the text, so it is a positive pin only.
- **Live recipe GREEN** (orchestrator, fresh release `simd 7721` on `2d6fd95`, then again `simd 7723`
  on `c0e7b0e`, identical ticks): `cut mark at A's base: tick 9`; `a woodcutter holds the cut (dwarf,
  tick): (0, 20)` (Nain); `every tile of A empty: tick 250`; `wood item at A's base: tick 250`; `wood
  item on a pile cell: tick 591`; `tree B (touching crown) changed: no`; `cut over no tree refused:
  tick 9`; `dig mark placed on tree D: no`; `tree D changed: no`; `TIMBER WIRE OK`, exit 0. A snapshot
  afterwards held exactly **4** items, all `wood`, on four pile cells: one per trunk cell of A.
- **Deliberate RED** (row 3, 5×5 crown box, release `simd` built in a scratch worktree off `2d6fd95`,
  `simd 7722`): every line as GREEN except `tree B (touching crown) changed: tick 250`; `TIMBER WIRE
  RED`, exit 1.
- **tui recipe** (same daemon as the GREEN run): before, `marks: z 13 designations=0 of 0`; after the
  cut on B, `designations=1 of 1`, `span x[73..73] y[56..56]`, and exactly one `/` (fg 226,96,64) in
  the map rows. At z 9: on `2d6fd95` (before Task 5) **zero** `=`, which was Task 5's live RED; on
  `c0e7b0e`, **four** `=`, drawn in zone green (88,190,118).
- **Mutations, `12-7.sh`: 20/20 KILLED** (`RUST_TEST_THREADS=1 scripts/mutate.sh`, after commit
  `e52c98b`). Each row died on the assertion it names. Rows 1 and 2: `Pos{72,58,14} still stands`
  (every tile empty). Row 3: `tree B changed at Pos{72,57,14}`. Row 4: `only the woodcutter holds a
  cut`. Rows 5 and 5b: `one log per trunk cell, at the base`. Rows 6 and 7: `dig/channel over
  TreeTrunk`. Row 8: the refusal assert. Row 9: `all(|job| job.kind != JobKind::Cut)`. Rows 5 (serve)
  and 10: `serve.rs:2785`, the all-`Wood` assert. Row 8 (serve): `serve.rs:2817`, `a cut over no tree
  was never refused`. Row 11: `save_load.rs:287`, `item_kinds()` after load. Row 13: `the last cancel
  must hold the trunk's base`. Row 14: `a log, not a cube`. Rows 12, 14b, 15 and 16 died on their
  named tests' first asserts. Row 16 (the placeholder clip) is beyond the story's list.
- **Seat finding, #168** (Wolf, 2026-10-05, at `900d557`): "works .. but cut trees have some blocks in the
  air" (`12-7-signoff/gui 05_10_2026 19.23.10.png`). Cause: `project.rs` spawned `CHIPS_PER_TILE` dig-debris
  chips on every tile a delta emptied, and a felled pine empties ~20 tiles over open air (tree A: 80 floating
  chips, 4 at its foot). RED: `a_felled_tree_leaves_debris_at_its_foot_and_none_in_the_air` gave `left: {[0,0,1]:
  4, [0,0,2]: 4, [0,0,3]: 4} right: {[0,0,1]: 4}`. Fix: chips only where the tile below is `Solid` or `Ramp`.
  `empty_tile_delta_leaves_deterministic_client_local_chips_and_snapshot_clears_them` emptied z 0 (the
  world's floor, nothing under it), so its fixture gained a floor; its assertions are unchanged. Row 17 KILLED.
  Fixed in `aed8c5b`; gui headless 112/112, gui lib 231 pass, clippy clean.
- **AC12 PASSED at Wolf's seat** (2026-10-05, `4be19a0`): "ok .. that works now". He also saw a log
  leave the ground as the hauler walks into it: that is #164 effect 1 (pick-up not gated on the drawn
  body), out of 12.7's scope by its guardrail, and recorded on #164.
- **Full gate GREEN** on `a7b1ded` (before the #168 fix): `RUST_TEST_THREADS=1 scripts/gate.sh`, 3251 s (cargo test 237 s,
  pixel guards 2978 s), `GATE GREEN`, exit 0.

### Completion Notes List

- A `cut` designation marks one whole tree at its trunk's base. `tree_of` is the 3×3 column box
  (base ..= top + 1), not a flood fill, with the `// NOTE:` the spec asks for. Nain fells it in
  `CUT_WORK_TICKS = 50` and leaves one `Wood` item per trunk cell. Tree A gives 4 logs; B has 3 trunk
  cells.
- Dig and channel never take tree tiles. A zero-cell dig, channel or cut rect returns
  `Refusal::Designate { kind, rect }`, and simd now pushes the `Designate` refusal. Cancel removes a
  cut mark whose tree has any tile in the rect, and releases its holder.
- `ItemKind { Stone, Wood }` is carried in the sim (`item_kinds()`), on the wire (`Item.kind`) and in
  the save. A pre-12.7 save is refused (`a_save_from_before_item_kinds_is_logged_and_the_daemon_keeps_ticking`).
- gui: `5` arms cut mode; one `designate cut` per drag at the cut-target level; clear adds a fourth
  command, a cancel on the cut-target rect. The cut mark is (30,190,150), nearest TUI mark the zone
  green at 66.2 (floor 50), and `mark_colours_are_distinct_cold_literals` now checks against all 13 TUI
  mark colours. Logs are `WOOD_ITEM_SCALE = (0.7, 0.28, 0.28)` from draft §3, coloured (164,116,66),
  stacked by ascending id with a step of one log height (0.28) for every kind. A woodcutter in Work on
  a cut plays the Dig clip facing the trunk (`// NOTE: placeholder until 12.8's Cut clip.`).
- tui: cut `/` (226,96,64), wood `=` (164,116,66), stored wood `=` in zone green. `tally_marks` now
  counts only the map rows: the `/` of the status row's `z 1/2` was being counted as a cut mark
  (found by `the_mark_tally_reports_what_the_frame_could_not_show` going red).
- **Deviations, recorded:**
  - `stored_wood_item_cell` is not in `every_look_is_pinned`'s `markers`, because that test asserts
    unique glyphs and it shares `=` with the plain log. It is pinned by its own `assert_eq!`, like the
    stored stone.
  - `same_seed_and_commands_remain_deterministic` runs 400 ticks (was 200) so the cut completes. Seed
    42's woodcutter is reassigned to hauler at tick 60, so the test hands him the axe back at tick 120.
  - `bridge.rs` gained one helper, `items_out`, shared by the snapshot and the delta.
  - The cut mark sits on the ground at the trunk's foot and is deliberately not lifted by
    `dig_mark_level` (`// NOTE:`).
- **Old mutation rows:** Agent A re-pointed 15 rows (3-1 ×2, 3-2 ×12, 3-3 ×1) and retired 5-1's `if
  yields_stone || true` row with a reason line. Agent B re-pointed 9 (12-1, 3-2, 3-3, 12-5 ×3, 6-1,
  8-2, 8-3). `audit-mutations.py`: 786 rows match. All 24 re-pointed rows were extracted into one
  scratch table and RUN (`mutate.sh`, after commit): **24/24 KILLED**, so none is audit-only.
- `README.md`: the key row has `5` cut; the flags table gains `--drag` (it was missing); the tui legend
  gains `/` and `=`. Spine AD-16 has its 2026-10-05 amendment. `12-7-signoff/vehicle-card.md` is
  written in the seat's launch form.

### File List

- `crates/protocol/src/lib.rs`
- `crates/sim-core/src/lib.rs`, `crates/sim-core/src/save.rs`
- `crates/sim-core/tests/scenario.rs`, `crates/sim-core/tests/save_load.rs`
- `crates/simd/src/bridge.rs`, `crates/simd/src/main.rs`, `crates/simd/tests/serve.rs`
- `crates/client-core/src/lib.rs`, `crates/client-core/tests/mirror.rs`
- `crates/gui/src/appearance.rs`, `crates/gui/src/capture.rs`, `crates/gui/src/designate.rs`,
  `crates/gui/src/ingest.rs`, `crates/gui/src/project.rs`, `crates/gui/tests/headless.rs`
- `crates/tui/src/palette.rs`, `crates/tui/src/view.rs`, `crates/tui/tests/client.rs`
- `README.md`
- `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`
- `_bmad-output/implementation-artifacts/mutations/12-7.sh` (new)
- `_bmad-output/implementation-artifacts/mutations/{3-1-give-the-order, 3-2-the-dig,
  3-3-the-haul-and-the-skeleton-walks, 5-1-the-world-grows-things-that-glow, 6-1-the-world-moves,
  8-2-designate-with-the-mouse, 8-3-master-of-time-and-the-skeleton-walks-in-3d, 12-1, 12-5}.sh`
- `_bmad-output/implementation-artifacts/12-7-signoff/vehicle-card.md` (new)
- `_bmad-output/implementation-artifacts/12-7-timber.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-05 | Story created on `fd9ca98`. RED on the live wire (`timber_wire.py`: `designate kind cut` unrecognized; a dig mark lands on a trunk and digs it away, dropping nothing). Look draft `12-7-signoff/draft.md` written; Task 0 open. |
| 2026-10-05 | Task 0 ruled by Wolf: draft approved with the Dig clip as the cut placeholder; one log per trunk cell (over the recommended one per pine), stacked in the gui; zero-cell dig/channel/cut rects all refused; refusal texts as written. |
| 2026-10-05 | Dev (Sonnet 5.5 agents A and B, Opus verifying): `designate cut`, `DwarfJob::Cut`, `ItemKind` and `Refusal::Designate` on the wire; whole-tree felling into one log per trunk cell; dig/channel never take trees; zero-cell rects refused; gui cut mode `5`, cut mark, stacked logs, Dig clip placeholder; tui `/` and `=`. Live wire OK (4 logs) and the deliberate RED shown; 20/20 + 24/24 re-pointed mutations killed; spine AD-16 amended; README; seat card. Full gate GREEN 3251 s on `a7b1ded`. Awaiting Wolf's seat (AC12). |
| 2026-10-05 | Seat (AC12): Wolf "works .. but cut trees have some blocks in the air". #168 filed; dig-debris chips spawned over air on every emptied tree tile. Fixed (`aed8c5b`): chips only over a solid or ramp floor. Mutation row 17 added and KILLED. Back to the seat for the floating blocks only. |
| 2026-10-05 | AC12 PASSED at Wolf's seat ("that works now", `4be19a0`). The log popping into the hauler's hands is #164 (out of scope; recorded there). Status -> review. Full gate last green on `a7b1ded`; re-run before the PR. |
| 2026-10-05 | Code review run 1 on `fb726ce` (4 layers, none timed out): no HIGH/MED code defect; 2 decisions resolved, 3 patches left as action items, 8 deferred, 12 dismissed. D2's tui check found the gui is a mirror image of the tui (pre-existing): #169. The `4` clear was not observed. Status -> in-progress. The launcher no longer pulls (`6126e9e`, #143, Wolf's request during the review). |
| 2026-10-06 | Review patch pass 1: the three refusal texts name the mark cap (`660c8f4`; tui clips them below ~95-110 columns, Wolf: land as ruled); `items_out` joins kinds by id (`9d96a7d`); the tui recipe counts the cut-mark colour (`ce737d6`); mutation rows 18-19 KILLED (`47c2f5c`). Full gate GREEN 3284 s on `47c2f5c`. Still open: AC12's `4` clear at the seat. Status stays in-progress. |
| 2026-10-06 | Code review run 2 on the patch diff `8c3764f..7499534` (4 layers, none timed out): 0 HIGH/MED, so the static audit ends. D1's cap refusal observed live. 2 LOW record patches left as action items; the partial-cap silent drop deferred as #170. Status stays in-progress. |
