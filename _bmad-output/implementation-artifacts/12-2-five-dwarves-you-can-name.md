---
baseline_commit: 16be274
model: claude-opus-5-5  # session default, same as 12.1's creation
---

# Story 12.2: Five Dwarves You Can Name

Status: in-progress

## Story

As the boss,
I want each dwarf to have a name and a colour of their own,
so that I can tell them apart at a glance and care which one is doing what.

## Not stacked: branch off `main`

`main` is `16be274` (PR #155 merged), clean. Branch `story-12-2-five-dwarves-you-can-name` exists
locally and carries this file, `12-2-signoff/` and the board edit. It closes **#136**. Epic 12's
nine standing ACs (`epics.md`, "Standing acceptance criteria") bind this story and are not restated
here. The standing wire-diff AC is satisfied by the "Wire diff" section below.

## Found at creation (2026-09-29, on `16be274`)

- **No identity exists anywhere today.** A fresh `simd 7482` snapshot's dwarves carry exactly
  `id, kind, light, pos, state`. `tui 7482 --frames 1 --z 9` ends with the status row
  `tick 15  normal  z 9/31  dwarves 5  N up`, and no name appears.
- **#136's premise is half stale.** DoF has focused the selected dwarf since `6194756`
  (`update_dof_from_camera`, `ingest.rs:2421`). He blurs anyway, for two compounding reasons
  (derived from Bevy 0.19's `dof.wgsl`, not yet measured):
  - **Wrong focus point.** The focus is his `Transform.translation`, which is his FEET: the
    spawn adds `entity_draw_offset` −0.5 Y, and the asset's origin is at min Y = 0
    (`project.rs:1797-1800`).
  - **Aperture calibrated for boot distance.** `DOF_APERTURE_F_STOPS = 0.05` (`ingest.rs:92`)
    was tuned at the boot framing, about 61.7 units away. Circle of confusion grows about as
    1/F², so at `MIN_DISTANCE` 4 (`camera.rs:7`) the in-focus band is roughly 225× thinner.
    Estimate: a head 0.33 units off the focal plane blurs about 9 px at 720p.
- **The dwarf has one material and one atlas**, `M_VoxelDwarf_r17` / `T_VoxelDwarf_r17`, 512²,
  NEAREST. There is no tunic slot. A UV→skin-joint census of the shipped GLB found the tunic
  cells: row 9 cols 0–3 and row 10 col 15 (32-px cells; see `12-2-signoff/draft.md`).
- **Look draft made at creation**: `12-2-signoff/draft.md` + `draft-crew.png` (a Blender render of
  the real GLB with the tunic cells recoloured). **Wolf approved it unchanged on 2026-09-29 (Task 0).**

## Wire diff (standing AC 4)

- NEW `protocol::DwarfName`: 16 variants, `snake_case`. The pool is `draft.md` §2.
- NEW `protocol::DwarfColour`: `Red | Gold | Green | Blue | Purple`, `snake_case`.
- NEW `protocol::Identity { name: DwarfName, colour: DwarfColour }`.
- `protocol::Entity` gains a last field, `identity: Option<Identity>`, with
  `#[serde(default, skip_serializing_if = "Option::is_none")]`.
  - A dwarf's line gains `"identity":{"name":"durin","colour":"red"}`.
  - Emitter lines are byte-identical.
  - `Entity` stays `Copy`.

## Acceptance Criteria

1. `World::generate` gives each of the five dwarves an identity: a name from the 16-name pool and a
   colour id. Names are distinct among the five, and so are colours. Both come from a NEW
   purpose-named stream, `STREAM_IDENTITY`, and never from `spawn_rng`. This is a determinism
   mechanism (AD-7), and it is load-bearing: spawn positions on every seed must stay exactly as
   they are today.
2. The identity is sim state. `save → load → tick N ≡ never-saved → tick N` compares identities, and
   seed + commands ⇒ identical identities. A test names two seeds whose name sets differ.
3. The real daemon is the judge. Its snapshot and its deltas carry each dwarf's identity; five
   distinct names and five distinct colours. After a daemon `save` then `load`, the fresh snapshot
   carries the same identity per id. A save whose dwarves repeat a name or a colour is refused
   with a log line.
4. The tui shows a roster row above the status row: the five names, ascending by id, each in its
   dwarf's colour. It adds no key, changes no status or hint text, and fits 80 columns. Under
   `NO_COLOR` the names still appear.
5. In the gui, each dwarf's tunic shows his colour id's colour. The rest of the model keeps the
   shipped atlas colours. A snapshot that gives an existing id a different identity (a load)
   re-colours that dwarf.
6. In the gui, while a dwarf is selected, a HUD line shows his name in his colour. With no
   selection, it shows nothing.
7. #136: with a dwarf selected and the camera at distance 4, his figure is sharp. The dwarf
   window's Laplacian sharpness with DoF on is ≥ 0.8× the same window under `--fx-off dof`. With
   no selection, DoF behaves exactly as today: it focuses the rig's orbit centre, and every
   existing DoF guard passes unchanged. #136 closes with that rule written on it.
8. At the seat, Wolf names each of the five dwarves by sight. Selecting one shows the name.
   Zooming in on a selected dwarf keeps him sharp. An attached tui shows the roster (success
   criterion 2).

## Tasks / Subtasks

- [x] **Task 0: draft approval (standing AC 7).** Wolf approves or amends `12-2-signoff/draft.md`
  (tunic colours, name pool, gui name line, tui roster row, the no-selection DoF rule). Record
  his words and date here. If he changes a hex or a name, update `draft.md` and use his values in
  Tasks 1, 3 and 5. Tasks 1–4 may run before approval. **Tasks 5–6 may not.**
  - **Approved 2026-09-29, unchanged (Wolf, at story creation: "1 ok 2 ok 3 ok").**
    1. The draft as written: tunic-only colour in the five hexes, the 16-name pool, the gui name
       line and the tui roster row.
    2. The no-selection DoF rule: focus stays on the rig's orbit centre.
    3. Old saves do not load: the load is refused with a log line, and there is no migration.
- [x] **Task 1: sim-core identity (AC1, AC2).**
  - [x] `lib.rs`, beside `LightKind`: `pub enum DwarfName { Durin, Dvalin, Nori, Ori, Dori, Bifur, Bofur, Gloin, Nain, Thrain, Frar, Loni, Regin, Alf, Fjalar, Frosti }`,
        `pub enum DwarfColour { Red, Gold, Green, Blue, Purple }`, and
        `#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] pub struct Identity { pub name: DwarfName, pub colour: DwarfColour }`.
        The enums derive `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize`,
        and each has a `pub const ALL: [Self; N]` in declaration order. The shuffles read `ALL`.
  - [x] `const STREAM_IDENTITY: u64 = 0x4944_454e_5449_5459; // "IDENTITY"` beside the others
        (`lib.rs:26-29`). In `generate` (`:1198`), seed `ChaCha8Rng::seed_from_u64(seed ^ STREAM_IDENTITY)`.
        Copy `DwarfName::ALL` and `DwarfColour::ALL` into arrays, `shuffle` each
        (`rand::seq::SliceRandom`, rand 0.10.2), and assign index `i` to the i-th dwarf in
        ascending id order. `spawn_dwarves` (`:1675`) takes the identities as a parameter and adds
        `Identity` to the spawn bundle (`:1699`). **Do not draw from `spawn_rng`**; that shifts
        every spawn position.
  - [x] Add `pub fn identities(&self) -> Vec<(Id, Identity)>`, sorted by id. It is a sibling
        reader, like `carrying()` (`:1617`). **Do not widen `dwarves()`**: its tuple has about 90
        destructuring call sites.
  - [x] `save.rs`: `SavedDwarf` gains `pub identity: Identity` (stays `Copy`). `to_save`
        (`:1248`) and `from_save` (`:1341`) carry it. The `filter_map` in `to_save` silently drops
        a dwarf missing any component (comment `:1240-1243`), so add the component at both spawn
        sites. Fix the 4 test literals (`save_load.rs:346,356`, `scenario.rs:1283,1293`).
        **Old saves (no `identity`) fail to decode**, and simd logs and refuses the load. That
        is intended. `// NOTE:` it on the field.
  - [x] Tests:
        - `worldgen.rs::five_dwarves_on_walkable_surface` (`:657`) also asserts 5 distinct names
          and 5 distinct colours.
        - `same_seed_produces_identical_worlds` (`:54`) also compares `identities()`.
        - NEW: `DEFAULT_SEED` and seed 42 yield different name sets.
        - `save_load_then_tick_matches_never_saved` (`save_load.rs:9`, list at `:165-176`) and
          `same_seed_and_commands_remain_deterministic` (`scenario.rs:1397`) also compare
          `identities()`.
        - The existing spawn-position tests must pass untouched.
- [x] **Task 2: protocol + simd (AC3; the wire diff above).**
  - [x] `protocol`: the three types and the field, as in "Wire diff". Fix every `Entity { .. }`
        literal (about 56; let the compiler list them). Emitters and fixture dwarves that don't
        care get `identity: None`.
  - [x] Pin tests in `protocol`:
        - the existing entity literal (`:301`) and the delta literal (`:265`) stay byte-identical;
        - NEW literal `{"id":7,"kind":"dwarf","pos":[4,5,6],"state":"idle","light":null,"identity":{"name":"durin","colour":"red"}}`
          round-trips;
        - `every_material_and_tile_variant_has_a_pinned_wire_name` (`:447`) gains blocks for
          both enums.
  - [x] `bridge.rs`: `dwarf_name` and `dwarf_colour` as exhaustive `match`es with no wildcard,
        beside `light_kind` (`:158`). Both dwarf maps (`:22`, `:81`) set
        `identity: Some(..)` from `world.identities()`, looked up by id. Keep them identical;
        extract one `dwarf_entity` fn if it keeps them so.
  - [x] `simd/src/main.rs` save validation (dwarf loop `:516`): `bail!` on a repeated name or a
        repeated colour. Add one `serve.rs` test in the shape of
        `duplicate_dwarf_id_save_is_logged_and_the_daemon_keeps_ticking` (`:652`).
  - [x] Extend `save_then_load_rewinds_every_client` (`serve.rs:528`): the connect snapshot carries
        5 distinct names and 5 distinct colours, and the post-load snapshot carries the same
        `(id, identity)` set.
- [x] **Task 3: client-core + tui (AC4).**
  - [x] `client-core`: `pub fn dwarf_name_text(name: protocol::DwarfName) -> &'static str`, an
        exhaustive `match` (`Durin => "Durin"`, …). It is the only spelling of a name, shared by
        both clients, like `refusal_text` (`lib.rs:10`).
  - [x] `tui/src/palette.rs`: `pub fn dwarf_colour(colour: protocol::DwarfColour) -> Rgb`, an
        exhaustive `match` using the approved hexes. Pin it in `every_look_is_pinned` (`:200`).
        **The `☺` glyph keeps its job-state colour**; the pinned state colours and `WALK_SGR`
        (`tests/client.rs:1353`) must not change.
  - [x] `view.rs::render` (`:228`): `map_h = h - 3`, and the roster row goes at `h-3`. Draw each
        name with `dwarf_colour` as `fg`, joined by two spaces, ascending by id, truncated to `w`.
        A dwarf with `identity: None` is skipped. The second `map_h` site (`:626`, the camera
        clamp) must match. Otherwise the cursor can scroll under the roster.
        Repoint `status_and_hint_occupy_the_bottom_two_rows` (`:1449`),
        `one_row_terminal_renders_blank` (`:981`), and the h=3 status tests (`:1440`, `:2044`).
  - [x] **The instrument is `tui --frames N` (real binary).** Add a `tests/client.rs` test with a
        stub daemon that sends a snapshot with two identified dwarves. Assert that the row above
        the `tick ` row contains both names, and that one name is wrapped in its colour's SGR.
        Then send a delta that swaps their identities and assert the row changes. Under
        `NO_COLOR=1`, the names still appear. Use `capture_walking_dwarf` (`:1359`) as the model.
- [x] **Task 4: gui instrument `--select ID`, then #136 RED → fix (AC7).**
  - [x] `parse_args_from` (`ingest.rs:1093`): `--select <id>` sets the startup `SelectedDwarf`.
        Trap: `frame_selected_dwarf` (`pick.rs:217`) sets `rig.distance = SELECT_DISTANCE`
        whenever the selection `is_changed()`, which includes the first frame, so a
        `--distance D` passed alongside would be overwritten. `--distance` must win when given.
        Add a parse test.
  - [x] Test the instrument: a pixel guard in `tests/pixel_guard.rs` captures
        `--select A` and `--select B` (two dwarves of different colour) at `--clock 12`, with
        the harness flags and one fresh daemon per capture (`:287`). The centre window must
        change between the two captures. After Task 5, each window's mean colour must be nearer
        its own dwarf's table colour than the other's.
  - [x] **RED first:** a pixel guard captures `--select 0 --distance 4` twice, once with DoF on
        and once with `--fx-off dof`. It compares `rec601_lap_mean` (`:95`) over a centre window
        sized to the dwarf. Record the failing ratio in the Debug Log. **If the RED ratio is
        already ≥ 0.8, stop and report to Wolf; #136 would then not be reproduced at the pinned
        venue.**
  - [x] Fix, selected path only. Focus on the body, not the feet: `translation` plus half the
        drawn figure height. Also make the aperture follow the focal distance so the subject
        stays in focus at close range; clamping or scaling `aperture_f_stops` are both fine.
        The no-selection path (`dof_focal_distance`, `ingest.rs:2352`) and its constants must
        not change. `dof_softens_the_far_ridge…` (`pixel_guard.rs:517`),
        `dof_keeps_depth_separation_at_distance_40` (`:572`) and the unit tests at `:3666` and
        `:4204` must pass untouched. Update
        `depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point` (`:4735`) to the
        body point.
- [x] **Task 5: gui tunic colour (AC5). Only after Task 0.**
  - [x] `appearance.rs`, beside `entity_appearance` (`:377`):
        `pub fn dwarf_tunic_color(colour: protocol::DwarfColour) -> Color`, with the approved
        hexes, exhaustive, and pinned in the palette pin test (`:590-612`).
  - [x] Build 5 recoloured atlases plus 5 `StandardMaterial`s, one per colour id, cloned from the
        GLB's material. Use the draft's rule: the tunic cells only, each texel becomes
        `colour × lum(texel) / max tunic lum`.
        - Name the cells in one const (`TUNIC_CELLS: [(u32, u32); 5]`), with a `// NOTE:` that
          they come from the r17 atlas census and a new dwarf asset must re-census them.
        - **Trap:** the glTF image may keep no CPU copy (`RenderAssetUsages`). Verify you can read
          its pixels. If you can't, load the atlas with main-world usage retained.
        - **Colour-space trap** ([[colour-space-bug-hides-as-palette-mismatch]]): the atlas is
          sRGB. Recolour in sRGB bytes, and assert in a test that a non-tunic texel is
          byte-identical after recolouring.
  - [x] Apply the material per dwarf. When a dwarf's scene instance has spawned, swap its
        mesh's `MeshMaterial3d` for its colour's material. Use `Added<MeshMaterial3d<StandardMaterial>>`
        and walk `ChildOf` up to the `WorldProjected` dwarf, as `drive_dwarf_walk` does
        (`project.rs:2077`). Leave the `AnimationPlayer` and the hierarchy untouched.
  - [x] Carry the colour on the projected dwarf as a component set at spawn (`project.rs:1795`).
        The existing-entity branch re-applies it when the mirror's identity differs, beside the
        `light` re-sync (`:1762-1776`).
  - [x] Headless tests (`tests/headless.rs`):
        - two dwarves of different colour end up with different material handles;
        - a snapshot that swaps their identities swaps the handles.
        - Unverified at creation: whether the GLB scene instance spawns under the headless
          minimal plugins. If it does not, assert on the projected dwarf's colour component and
          on the colour → material lookup instead, say so in the Debug Log, and the Task 4
          pixel guard carries the on-model evidence.
- [x] **Task 6: gui name line (AC6). Only after Task 0.** Add a HUD `Text` tagged `Hud` and
  `ClientLocal`, placed per the approved draft, using the clock-readout pattern
  (`setup_clock_readout` `ingest.rs:1702`). Its text is `dwarf_name_text` for the selected id,
  its `TextColor` is `dwarf_tunic_color`, and it is empty with no selection. Captures hide every
  HUD element, so the evidence is an in-crate test modelled on
  `the_live_clock_readout_follows_the_daemons_tick_and_speed` (`:3313`):
  - select id A → the text is A's name;
  - select B → B's name;
  - clear the selection → empty.
- [ ] **Task 7: the record.**
  - [ ] Write mutation set `_bmad-output/implementation-artifacts/mutations/12-2.sh`. Every row
        must be KILLED, by the test named:
        1. identity drawn from `spawn_rng` → a spawn-position or identity test;
        2. `from_save` ignores the saved identity → `save_load_then_tick…`;
        3. all five get one colour → `five_dwarves_on_walkable_surface`;
        4. the bridge sends `identity: None` → the `serve.rs` test;
        5. the tui never draws the roster → the client test;
        6. one material for all dwarves → the headless material test;
        7. reconcile ignores an identity change → the swap test;
        8. the HUD never shows the name → the ingest test;
        9. DoF focus reverted to `translation` → the #136 pixel guard.
  - [ ] Comment on #136: the cause as measured, the fix, and the no-selection rule. It closes via
        the PR (`Closes #136`; the whole issue is fixed).
- [ ] **Task 8: seat (AC8), then the full gate.** Write `12-2-signoff/vehicle-card.md` in the
  seat's launch form: in WSL `simd 7451`; in PowerShell
  `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`; attach `tui 7451` in WSL. Steps:
  - (a) Wolf names all five by tunic colour at working zoom, then checks by clicking each one to
    see the name;
  - (b) select one, wheel to the closest zoom, and he stays sharp;
  - (c) Escape, and DoF is back to the camp-focused look;
  - (d) the tui roster matches.
  Tell him old `frostvein.save` files will not load (Ctrl+S a fresh one first). Then run the
  full `scripts/gate.sh` at `RUST_TEST_THREADS=1` ([[gate-ooms-at-default-parallelism]]).

## Dev Notes

### Scope guardrails (do NOT)

- Do not put a `String` on the wire. The name is a closed enum (AD-6), which keeps `Entity` and
  `SavedDwarf` `Copy`. The spelling lives only in `client-core::dwarf_name_text`.
- Do not recolour the tui's `☺`, the carrier `☻` or the crowd `⚇`. The roster row is the tui's
  whole identity display (NFR10 display parity; no new tui input).
- Do not add floating name labels, outlines or a selection ring in the gui. The draft specifies
  one HUD line.
- Do not touch DoF with no selection, the boot framing, or any DoF constant the no-selection path
  reads.
- Do not edit the dwarf GLB or its Blender source. The recolour is client-side.
- Do not add rename, a colour picker, or more than five colours. FR40 (variation beyond colour) is
  12.13.
- Do not make old saves load (no `#[serde(default)]` on `identity`). A refused load is loud, which
  NFR11 allows.

### What already exists (build on it)

- `SelectedDwarf` + click select + Escape + `frame_selected_dwarf` (story 10.10, `pick.rs:68-245`).
  Selection is client-local and sends nothing.
- `update_dof_from_camera` already switches to the selected dwarf. Only the focus point and the
  close-range aperture are wrong.
- `pixel_guard.rs` has the daemon + capture harness, `rec601_lap_mean`, and the one-daemon-per-capture rule.
- `client-core::refusal_text` is the pattern for shared client text. The `Mirror` stores whole
  `protocol::Entity` values, so identity reaches both clients with no mirror change.
- The seeded-stream pattern: `seed ^ STREAM_X` constants (`lib.rs:26-29`), and worldgen-only
  streams are not saved.

### Key decisions & traps

- **`identity: Option<Identity>`, skipped when `None`:** `Entity` also carries torches and the
  campfire. Skipping `None` keeps every pinned and recorded emitter line byte-identical.
- **Five colours, exactly one per dwarf:** a permutation guarantees every pair differs in hue.
  **Sixteen names, pick five:** worlds differ.
- **A load can change the identity behind an id.** Two worlds both number their dwarves 0–4. The
  gui re-applies on change; a spawn-only apply would leave the old world's tunics on.
- **Hue evidence is a daytime capture** (`--clock 12`). Captures default to the boot hour 22, when
  firelight tints everything warm.
- **The HUD line cannot be pixel-evidenced.** Captures hide `Hud` (`ingest.rs:1775`). The in-crate
  test and the seat carry AC6.

### Verification

Deliberate RED, observed at creation on `16be274`:

```bash
cargo build -q -p simd -p tui && ./target/debug/simd 7482 &
python3 -c "import socket,json;f=socket.create_connection(('127.0.0.1',7482)).makefile();s=json.loads(f.readline());print([sorted(e) for e in s['entities'] if e['kind']=='dwarf'][0])"
#   RED (observed): ['id', 'kind', 'light', 'pos', 'state']   GREEN: 'identity' present, 5 distinct names and colours
./target/debug/tui 7482 --frames 1 --z 9 | sed 's/\x1b\[[0-9;]*[A-Za-z]//g' | tail -4
#   RED (observed): the map, then `tick 15 ... dwarves 5  N up`, then the hint; no names
#   GREEN: the row above `tick ` holds five names
pkill -x simd
```

After the fix, break it on purpose: mutation row 4 (the bridge sends `None`). The python line
shows no `identity`, and the tui roster row is empty. Restore, and both are back.

The gui instrument cannot run until Task 4 builds it. Recipe (the Task 4 guards automate it; one
fresh daemon per capture):
`gui <port> --headless --static-world --lights-steady --subdiv 4 --select 0 --distance 4 --capture /abs/on.png`,
then the same command with `--fx-off dof` → `/abs/off.png`.
- RED: the centre-window sharpness ratio is < 0.8.
- GREEN: it is ≥ 0.8.
Exit 0 is not a result; the ratio is.

### Project Structure Notes

- `crates/sim-core/src/{lib.rs,save.rs}`, `tests/{worldgen,save_load,scenario}.rs`: UPDATE
- `crates/protocol/src/lib.rs`: UPDATE (3 types, 1 field, pins)
- `crates/simd/src/{bridge.rs,main.rs}`, `tests/serve.rs`: UPDATE
- `crates/client-core/src/lib.rs`: UPDATE (`dwarf_name_text`, literals)
- `crates/tui/src/{palette.rs,view.rs}`, `tests/client.rs`: UPDATE
- `crates/gui/src/{ingest.rs,pick.rs,project.rs,appearance.rs}`, `tests/{headless,pixel_guard,capture}.rs`: UPDATE
- `_bmad-output/implementation-artifacts/mutations/12-2.sh`, `12-2-signoff/vehicle-card.md`: NEW
- `12-2-signoff/{draft.md,draft-crew.png,draft.py}`: created at story creation

### References

- `epics.md` Epic 12 intro (standing ACs) and Story 12.2; PRD `prd-frostvein-2026-09-28` FR38,
  FR39, NFR9–NFR11, success criterion 2
- Parent spine AD-6, AD-7, AD-8, AD-9, AD-11, Consistency Conventions (Vocabulary enums, Color);
  M2 spine AD-13, AD-14, AD-17
- Issue #136; commit `6194756` (DoF follows the selection); story 10.10 (selection); story 12.1
  (`refusal_text` pattern, `Delta` literal sweep)
- Memory: [[dof-inert-at-world-scale]], [[colour-space-bug-hides-as-palette-mismatch]],
  [[headless-gui-instruments]], [[freeze-point-tracks-daemon-uptime]]

### Previous story intelligence (12.1)

- A new field on a wire struct breaks every literal (12.1: about 35 `Delta`s; here about 56
  `Entity`s). It is mechanical, so let the compiler list them. `Entity` has no `Default`, and
  adding one is not this story's job.
- The full gate goes green only at `RUST_TEST_THREADS=1` (2652 s). At 2 threads, pixel guards die
  on load. This story adds pixel guards, so budget for it.
- A test assertion placed early can absorb a mutation meant for a later one (12.1's scenario
  vacuity). Put each mutation's killing assertion where only that mutation reaches it.

## Dev Agent Record

### Agent Model Used

### Debug Log References

- Task 4 RED, on `5d0cacf` (before the fix): `--select 0 --distance 4 --clock 12`, window (540,170)-(740,390), Rec.601 Laplacian mean, DoF off 5.8911, DoF on 2.2679, **ratio 0.385** (< 0.8, so no halt). Fix at `8ae5f2c`.
- Task 4 GREEN: DoF off 5.8911, DoF on 5.8893, **ratio 1.000**.
- Existing guards after the fix, unedited: `dof_softens_the_far_ridge_while_retaining_camp_focus_and_stars` (ratio 11.753) and `dof_keeps_depth_separation_at_distance_40` (ratio 3.303) pass.
- The fix: the selected subject is his body (`translation` + half of `DWARF_HEIGHT_CELLS` = 0.375), and the aperture scales by `(61.7/focal)^2` when the focal distance is under boot's (`selected_aperture`). The no-selection arm still writes `DOF_APERTURE_F_STOPS`.
- Task 4 `--select` guard: at `--clock 12` the tunics wash to about (195,160,165) (red and purple read alike), so the guard uses `--clock 8`. Chromaticity margins 0.040 (dwarf 0, purple) and 0.077 (dwarf 2, red); with every dwarf painted red dwarf 0 reads 0.020; the floor is 0.03.
- Task 5: the real GLB atlas keeps its CPU pixels (no `no readable CPU pixels` line in a capture, and the tunics render coloured on the model). The GLB scene does NOT spawn under the headless `MinimalPlugins` (no `AssetServer`), so the headless test stands in a mesh child of each projected dwarf carrying a synthetic base material and atlas; from there the production path runs. The pixel guard carries the on-model evidence.
- Task 4 mutation finding: focusing the FEET (the pre-#136 point) is NOT killed by the #136 pixel guard, because the widened aperture leaves the figure sharp either way (ratio 1.000). The unit test `depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point` kills it (tolerance tightened 0.25 to 0.03 with a discrimination assert), and an unscaled aperture is killed by the pixel guard (ratio 0.468).

### Completion Notes List

- Tasks 4-6 done (dev agent, sonnet-5.5). Commits: `5d0cacf` (`--select`), `57c9d1a` (RED guard), `06d3b94`+`d191d9b` (`--select` guard), `8ae5f2c` (the #136 fix), `8846b15` (tunic recolour), `46e9554` (focus test tightened), `9ac89c7` (HUD name).
- Re-pointed mutation rows (sabotage and killing test unchanged): `10-10-...sh` (the follow-pins-zoom row), `11-2-the-miniature.sh` (two frozen-camera DoF rows), `5-3-a-window-onto-the-valley.sh` (the reconcile find gained a fourth tuple element), `8-3-...sh` (the clock-readout row, now chained before the name readout).
- Deviations: the `--select` guard runs at `--clock 8`, not 12 (see Debug Log). Three existing HUD-count tests went 5 to 6 texts for the new name line. `DwarfColour` has no `ALL` in `protocol`, so the client keeps its own `TUNIC_COLOURS` order.

- Tasks 1-3 done (dev agent, sonnet-5.5). Commits: 1b4f8c7 (sim-core), 6a56a89 (protocol + simd), 8ad0fce and 00c9314 (tui roster).
- Re-pointed two mutation tables the change broke: `2-2-...sh` (spawn_dwarves call gained `identities`) and `6-2-...sh` (the two bridge dwarf maps are now one `dwarf_entities` fn, so both rows hit the single site).
- Deviations: `render` now blanks below 3 rows (was 2), because three rows are reserved. The cursor-pan unit test viewport went (5,5) to (5,7) so a wrong map_h in `apply_key` is killed (3 and 4 rows pan identically). A `dwarf_name_text` spelling pin test in client-core was added because the client test only checked substrings.

### File List

- `crates/gui/src/{ingest.rs,pick.rs,project.rs,appearance.rs}`, `crates/gui/tests/{headless.rs,pixel_guard.rs}`
- Mutation tables re-pointed: `mutations/{10-10-take-the-camera-where-you-want-it,11-2-the-miniature,5-3-a-window-onto-the-valley,8-3-master-of-time-and-the-skeleton-walks-in-3d}.sh`

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-29 | Task 0: Wolf approved the draft, the no-selection DoF rule and the no-old-saves rule, all unchanged. |
| 2026-09-29 | Story created on `16be274`. RED observed live: the wire carries no identity and the tui shows no names. #136 premise corrected: DoF already follows the selection, but focuses his feet with an aperture tuned for boot distance. Look draft rendered from the real GLB (`12-2-signoff/`). |
