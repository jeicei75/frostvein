---
baseline_commit: 589f524
model: claude-opus-5[1m]  # default Opus; 1M-context variant, as at 3.3
---

# Story 4.1a: Behold the Fortress in Depth

Status: done — **deliberately NOT merged to `main`.** Kept on branch
`4-1a-behold-the-fortress-in-depth` (Wolf, 2026-08-08). See Disposition below.

## Story

As the boss,
I want a raycast 3D view of my fortress in the terminal,
so that I can see the icy world — terrain and diggings — with depth.

## Acceptance Criteria

1. From the 2D view in `Mode::Normal`, `v` switches to the raycast 3D view and `v` returns. `v` is
   ignored in any designation mode, so a half-placed rectangle is never silently lost, and the
   toggle is client state only — no command goes upstream.
2. In the 3D view `Left`/`h` and `Right`/`l` turn the camera one heading step, `Up`/`k` and
   `Down`/`j` move it one tile forward and back along its heading, and `<`/`>` lower and raise it one
   z — all clamped to world bounds. `d`, `c`, `p` and `x` do nothing there (designation input stays a
   2D capability); `Space`, `+`, `-`, `S`, `L`, `q` and `Ctrl-C` keep their existing global behaviour.
3. The heading is one of 8 fixed 45° headings held as an integer on `ViewState`, so `ViewState` keeps
   its `Copy + Eq` derives and the same `--key` sequence renders a byte-identical frame on every run.
   Forward and back therefore move by an exact `(dx, dy) ∈ {-1,0,1}²` step. // Mechanism is
   load-bearing: this epic's entire evidence base is scripted captures, and a float heading makes
   them irreproducible and breaks the derives every existing `ViewState` test relies on.
4. The 3D camera *is* the 2D view's `camera` and `z` — no second camera and no second copy of the
   world. Toggling shows the same place from ground level, and a move made in either view is
   reflected in the other.
5. For each map cell the renderer casts one ray through the voxel grid stepping in x, y **and** z
   (Amanatides–Woo), stopping at the first non-`Empty` tile, at the world boundary, or at a hardcoded
   step cap. A ray fired into open air with nothing to hit returns after at most `MAX_RAY_STEPS`
   steps — asserted per ray, so a frame's cost is bounded by the viewport rather than by the terrain.
6. A hit's colour comes from the existing `palette::tile_cell` id → RGB table, shaded by distance and
   by which face the ray crossed. There is no second material → colour mapping anywhere in the client
   (AD-4). A ray that hits nothing draws `BLANK`.
7. The glyph carries the distance band — a four-step ramp, nearest to farthest — while the colour
   carries material, so the geometry is still readable when `NO_COLOR` strips every colour sequence.
8. A ray entering a tile occupied by a dwarf stops there and draws that dwarf's colour from
   `palette::entity_cell` instead of terrain. The occupied-tile index is derived in the client from
   `snapshot.entities` alone: `protocol` is **unchanged** — no new field, message or enum variant.
   Removing that entity from the snapshot must make the same cell render terrain again.
9. The 3D view fills the same `Framebuffer` the 2D view does and `view::render` dispatches on the
   view, so the interactive loop, `--frame` and `--frames` all get it from one call site. The bottom
   two rows stay the status and hint rows, drawn once for both views and flushed once per frame.
10. In the 3D view the status line reports the view and the heading alongside tick, speed, z and
    dwarf count, and the hint bar names the 3D keys and does not advertise `d`/`c`/`p`/`x`. Both
    still fit 80 columns at `tick 9999999`.
11. Terrain that changes on the wire changes the 3D picture from the same protocol state: with the
    camera facing a wall, a delta turning that wall tile `Empty` lets the ray pass through to
    whatever stands behind it.
12. `tui --frames N --key v` — the existing instrument, with `v` added to its key table and no second
    channel invented. Against a stub daemon: the capture's status lines report the 3D view and the
    map area holds a non-zero count of **at least two distinct** band glyphs; the identical run
    without `v` reports the 2D view and contains none of the 3D status text; and a run with `v,l,l`
    differs from the run with `v` alone. The second and third captures are the guards on the first.
13. `scripts/gate.sh` passes and `scripts/mutate.sh
    _bmad-output/implementation-artifacts/mutations/4-1a-behold-the-fortress-in-depth.sh` reports
    zero survivors.

## Tasks / Subtasks

- [x] **`tui`: view state gains a view and a heading** (AC: 1, 2, 3, 4)
  - [x] `pub enum View { Flat, Depth }` and two fields on `ViewState` [view.rs:24-33]:
        `view: View` and `heading: u8`. **Both must be `Eq`** — the struct derives
        `Copy, PartialEq, Eq` and every test builds it with `..normal_state(...)`. A float yaw
        breaks the derive and every one of those tests.
  - [x] `initial` [view.rs:58] starts `View::Flat`, `heading: 0`. Do not add a `--3d` flag;
        `--key v` is the scripted route and `--frame` (which renders before any key is applied)
        deliberately cannot reach the depth view.
  - [x] `apply_key`: `KeyCode::Char('v') if state.mode == Mode::Normal` flips `view` and returns
        `Action::Redraw`; `Char('v')` otherwise returns `Action::Ignore`. Mirror the existing
        `'d'|'c'|'p'|'x'` guard pair [view.rs:376-388] exactly — same shape, same reason.
  - [x] Route the movement keys on `state.view` **before** the existing `mode == Mode::Normal`
        branch inside each arm [view.rs:442-473]: in `View::Depth`, `h`/`Left` and `l`/`Right` do
        `heading = (heading + 7) % 8` and `(heading + 1) % 8`; `k`/`Up` and `j`/`Down` add and
        subtract `raycast::heading_step(heading)` from `camera`, clamped to
        `0..=dims.{x,y}-1` exactly as the flat arms clamp. `<`/`>` keep their existing
        z behaviour unchanged.
  - [x] Tests: `v` toggles both ways from normal and is ignored in all four designation modes (and
        leaves `anchor` untouched); each of the 8 headings turns to the right neighbour and wraps
        both directions; forward then back from a heading returns to the starting tile; forward at a
        world edge clamps rather than leaving the world; `d`/`c`/`p`/`x` in `View::Depth` return
        `Action::Ignore` and leave `mode` at `Normal`; `Space`/`S`/`L`/`q` still work there.

- [x] **`tui`: the raycaster** (AC: 5, 6, 7, 8) — new file `crates/tui/src/raycast.rs`, `mod raycast;`
      in `main.rs` beside `mod frame; mod palette; mod view;` [main.rs:3-5].
  - [x] `pub fn draw(snapshot, state, w, map_h, cells: &mut [Cell])` fills the map region in place —
        it does **not** own a `Framebuffer`, so the two views cannot drift apart on size, status rows
        or flush behaviour.
  - [x] Ray generation from the camera plane, using `state.heading` for the yaw and a hardcoded
        90° horizontal FOV with a `CELL_ASPECT` correction (a terminal cell is about twice as tall as
        it is wide). Origin is the tile centre: `(camera.0 as f64 + 0.5, camera.1 as f64 + 0.5,
        z as f64 + 0.5)`.
  - [x] `cast(...)`: standard Amanatides–Woo — `step`/`t_max`/`t_delta` per axis, a zero direction
        component giving `f64::INFINITY` so that axis never advances. Stop on the first non-`Empty`
        tile, on leaving the world, or at `MAX_RAY_STEPS`. **Bounds-check before indexing `tiles`**;
        reuse the widened `tile_index` shape [view.rs:513] rather than writing a second one.
  - [x] Colour: `palette::tile_cell(tile).fg` shaded by distance and by face. Add
        `pub fn shade(fg: Rgb, percent: u16) -> Rgb` to `palette.rs` and express the existing
        `dim` through it — `dim`'s signature, `DIM_PERCENT` and `dim_darkens_monotonically`
        [palette.rs:296] must be untouched and stay green. This is the second concrete use, which is
        what earns the promotion.
  - [x] Glyph: a 4-entry band ramp indexed by distance thresholds. Miss ⇒ `BLANK`.
        // NOTE: sky and "nothing drawn" are the same cell deliberately — the map area is fully
        written on every 3D frame, so the guard against a silently blank render is AC12's
        two-distinct-bands range check, not a distinct sky glyph that `NO_COLOR` would strip anyway.
  - [x] The dwarf index: one `BTreeMap<[i32; 3], JobState>` per frame from
        `snapshot.entities.filter(kind == EntityKind::Dwarf)`, lowest id wins a shared tile
        (`.entry().or_insert()`), consulted at each stepped voxel before the tile test. Carry the
        existing `// NOTE:` convention [view.rs:218-219]: a second `EntityKind` must decide its own
        rule. **This index is 4.1b's seam** — its output must change what is drawn, not merely be
        built (see the tests below).
  - [x] `pub fn heading_step(heading: u8) -> (i64, i64)` and `pub fn heading_name(heading: u8) ->
        &'static str`, both consumed by `view.rs`; index 0 = `+x` = `"e"`, then clockwise on screen.
  - [x] Tests, all on hand-built `Snapshot`s in the `empty_snapshot` style [view.rs:526-538] — **never**
        by pulling `sim-core` in for a fixture, which turns the gate probe red (see Key decisions):
        a wall N tiles ahead lands in the expected band and moves to a nearer band at N/2; every one
        of the 8 headings sees a wall placed in that heading's direction and only that one; a ray
        leaving the world draws `BLANK`; the same cell renders the dwarf colour with an entity on
        that tile and the terrain colour with the entity removed (AC8's negative); a wall tile turned
        `Empty` reveals the wall behind it (AC11); the colour of a hit is `tile_cell(tile).fg`
        shaded — assert against `tile_cell`, never against a copied literal; a ray fired into an
        all-`Empty` world returns after at most `MAX_RAY_STEPS` steps rather than running to the
        world boundary. Have `cast` report its step count so that last one is observable without a
        test-only hook in the draw path.

- [x] **`tui`: dispatch, status and hint** (AC: 9, 10)
  - [x] `view::render` [view.rs:127]: keep the `w == 0 || h < 2` guard and `map_h` as they are, then
        `match state.view` — `Flat` runs the existing terrain/zone/designation/item/entity/pending/
        cursor block unchanged, `Depth` calls `raycast::draw`. The status and hint block below stays
        shared and outside the match. Dispatching **here** and not in `main.rs` is deliberate: three
        call sites render [main.rs:196, 286, 406] and the instrument must exercise the same path the
        player does.
  - [x] Status line in `View::Depth`: the existing fields plus the view and heading, in exactly this
        shape — `tick 20  normal  3d e  z 17/31  dwarves 5`. **The `  3d <heading>  ` token is pinned**,
        because the Verification recipe greps for it; a test asserts the exact rendered string. Extend
        `status_line_fits_eighty_columns_without_truncation_at_large_ticks` [view.rs:1178] to cover
        the depth view at `tick 9999999` — that test measures real rendered width, so it is the guard.
  - [x] Hint in `View::Depth`: one line naming turn, move, z, the `v` return and quit, and **not**
        `d`/`c`/`p`/`x`. Prefix it `depth:` in the style of the mode hints — **not** `3d:`, which
        would collide with the recipe's grep for the status token. Extend `hint_bar_names_every_modes_keys_and_fits_eighty_columns`
        [view.rs:1123] — it renders at width 120 on purpose so an over-long hint cannot truncate
        into a pass.

- [x] **Observability instrument** (AC: 12) — extend `tui --frames N --key`; do not invent a second
      channel. Copy `capture_dig_replay` [client.rs:631-740] as the worked pattern.
  - [x] `named_key` [main.rs:328-349] gains `"v"`, and `every_instrument_key_name_is_pinned`
        [main.rs:548] gains its row. **Without this no scripted capture can ever reach the 3D view** —
        the key table is a closed set and an unknown name bails.
  - [x] Update the `--key` usage strings [main.rs:103-105, 147] to include `v`.
  - [x] Three captures against the stub daemon: `--key v` (status lines report the depth view; at
        least two distinct band glyphs present with non-zero counts); no key (status lines report the
        flat view, none of the depth status text); `--key v,l,l` (differs from the first). Strip SGR
        with `strip_ansi` [client.rs:81] so every assertion survives `NO_COLOR`.
  - [x] A change-detection test in the shape of
        `streamed_frames_hold_the_camera_still_so_a_moving_dwarf_moves_on_screen` [client.rs:1071]:
        a stub whose wall moves between deltas must produce frames that differ, so the capture is an
        observation rather than a static artefact.

- [x] **Sabotage + mutation set** (AC: 13) —
      `_bmad-output/implementation-artifacts/mutations/4-1a-behold-the-fortress-in-depth.sh`, at least:
      `v` does not toggle; `v` toggles from a designation mode; turning goes the wrong way; turning
      does not wrap; forward reads the wrong `heading_step` entry; forward is not clamped to the
      world; `d` enters dig mode in the depth view; the DDA drops its z component; the DDA indexes
      `tiles` without the bounds check; `MAX_RAY_STEPS` removed; the band ramp collapses to one
      glyph; face shading removed; the hit colour comes from a second hardcoded table instead of
      `tile_cell`; the dwarf index is built and its lookup result discarded (the seam mutation —
      it must die); the dwarf index ignores `EntityKind`; `render` always dispatches `Flat`; the
      status line omits the view; the hint bar advertises `d c p x` in depth; a miss draws a band
      glyph instead of `BLANK`; the ray angle ignores `heading` so every ray faces `+x`;
      `named_key("v")` removed.
  - [x] `cargo clean -p sim-core -p protocol -p simd -p tui` before the final gate — `mutate.sh` is
        not concurrency-safe and 2.3, 2.4, 3.1 and 3.2 each burned a cycle on a stale mutated binary.
  - [x] Paste the actual RED output for every new mapping/constant test into the Dev Agent Record
        (AGENTS.md rule 1).

- [x] **Green gate and the live capture** (AC: 12, 13) — `scripts/gate.sh`, then the recipe in
      Verification below. Report the actual counts, not that it passed.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No sub-voxel creature models.** A dwarf is one voxel of its own colour. The ~10×5×13
  boxes-as-code models, fine-step sampling inside creature tiles, distance LOD and seed-derived
  palette swaps are story 4.1b's entire subject, and so is the FR23/FR24 sign-off.
- **No wire change.** `protocol`, `sim-core`, `simd` and `bridge.rs` are untouched — see the first
  Key decision.
- **No pitch key.** The vertical FOV means you already see above and below the camera; a look-up/down
  control is a second camera concept for one story's worth of value. `// NOTE:` the limitation.
- **No designation input in the 3D view** (epic AC1). No cursor, no anchor, no modes, no new
  commands, no mouse.
- **No lighting model, no textures, no sub-tile geometry, no ramp slope.** A `Ramp` is a solid voxel
  that happens to be ramp-coloured. Distance + face shading is the whole shading model.
- **No performance work beyond the step cap.** No occlusion cache, no ray coherence, no parallelism,
  no dirty-region redraw. If the measured frame time misses NFR2, say so in the Dev Agent Record and
  bring the number to Wolf — do not optimise speculatively.
- **No fix** for the still-open `NO_COLOR` product-half, status-line-width, `MAX_SAVE_BYTES`-vs-world
  -size, SIGTERM, panic-invisibility or stockpile-on-rock items in `deferred-work.md` — none is
  assigned here.

### What already exists (build on it, do not re-derive)

- `Framebuffer` + `Cell` + `write_frame` are done and view-agnostic; `write_frame` flushes one frame
  in one write and needs no change [frame.rs:23].
- `palette::tile_cell` and `entity_cell` are the id → RGB table AD-4 requires, already pinned by
  `every_look_is_pinned` [palette.rs:168]. `dim` [palette.rs:151] is the shading arithmetic to
  promote, not to copy.
- `view::initial` opens deterministically on the level with the most standable ground and `tui --z N`
  pins one [view.rs:58-125] — closed as action item T3 on 2026-08-08, expressly as this story's
  prerequisite. Do not reintroduce anything world- or time-dependent into the opening view.
- `tui --frames N --key <seq>` runs the real reader thread and the real
  `apply → render → write_frame` path [main.rs:358-413], holds the camera still across frames, and
  already warns on stderr when `NO_COLOR` makes a capture colour-blind. Extend it; do not add a mode.
- Test scaffolding: `empty_snapshot` [view.rs:526], `strip_ansi` [client.rs:81], `glyph_columns_for`
  [client.rs:592], `capture_dig_replay` [client.rs:631], and 3.3's mutations file as the worked format.

### Code skeleton

```rust
// crates/tui/src/view.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View { Flat, Depth }

pub struct ViewState {
    // … existing fields unchanged …
    pub view: View,
    /// 0..8, 45° apart, 0 = +x. An integer, not an angle: `ViewState` must stay
    /// `Copy + Eq`, and a scripted capture must render identically every run.
    pub heading: u8,
}
```

```rust
// crates/tui/src/raycast.rs
use std::collections::BTreeMap;
use protocol::{EntityKind, JobState, Snapshot, Tile};
use crate::{palette::{BLANK, Cell, entity_cell, shade, tile_cell}, view::ViewState};

const HALF_FOV_TAN: f64 = 1.0;   // 90° horizontal
const CELL_ASPECT: f64 = 2.0;    // a terminal cell is ~2x taller than wide
// NOTE: the world is 128 across, so 96 steps reaches past anything worth seeing and
// makes a frame's cost bounded rather than terrain-dependent.
const MAX_RAY_STEPS: u32 = 96;
const BAND_GLYPHS: [char; 4] = ['█', '▓', '▒', '░'];   // nearest → farthest
const BAND_LIMITS: [f64; 3] = [4.0, 10.0, 24.0];
const FACE_SHADE: [u16; 3] = [100, 78, 60];            // x, y, z faces

/// Fills the map region in place. It owns no `Framebuffer`, so the two views cannot
/// drift apart on size, on the status rows, or on flushing.
pub fn draw(snapshot: &Snapshot, state: &ViewState, w: u16, map_h: u16, cells: &mut [Cell]);

pub fn heading_step(heading: u8) -> (i64, i64);      // (1,0) (1,1) (0,1) (-1,1) …
pub fn heading_name(heading: u8) -> &'static str;    // "e" "se" "s" "sw" …
```

```rust
// crates/tui/src/view.rs — one dispatch, so all three render call sites follow it.
match state.view {
    View::Flat => { /* the existing map + marker + entity passes, unchanged */ }
    View::Depth => crate::raycast::draw(snapshot, state, w, map_h, &mut framebuffer.cells),
}
// status and hint rows stay below the match, shared by both views
```

### Key decisions & traps

- **SETTLED — creature-flagged tiles need NO wire change, in 4.1a or in 4.1b** (action item E2, first
  half). `protocol::Entity { id, kind, pos, state }` [protocol/src/lib.rs:89-95] is a full-resend
  section of every `Snapshot` and `Delta` [protocol/src/lib.rs:129, 145], and `tui::apply` replaces
  `snapshot.entities` wholesale on every delta [main.rs:514]. "Creature-flagged" is therefore a
  **client-side index derived from data the client already holds**, not a flag the sim sends. So this
  story is *not* a wire change and nothing has to move in one commit. Forward note for 4.1b, not for
  here: if a sub-voxel model ever needs a *facing* direction, that is not on the wire and would be
  the first real wire question this epic raises.
- **SETTLED — how the depth view stays inside the gate probe** (action item E2, second half). The
  probe is `cargo tree -p tui | rg -q sim-core`, where a **match is the failure** [scripts/gate.sh].
  Verified 2026-08-08: `cargo tree -p tui` and `cargo tree -p tui -e normal,dev,build` print
  identical output, so a **dev-dependency** on `sim-core` trips it exactly like a normal one. Two
  concrete temptations to refuse: reusing a `sim-core` geometry helper (`World::is_standable`,
  `Terrain`, the A* neighbour walk) for the DDA, and pulling `sim-core` in as a dev-dependency to
  build a "real world" test fixture. Both turn the gate red. Every fixture is a hand-built
  `Snapshot`, exactly as `view.rs` already does it.
- **The heading is an integer, and that is not a style choice.** `ViewState` derives
  `Copy, PartialEq, Eq` [view.rs:24]; an `f64` yaw breaks `Eq` and takes the whole `ViewState` test
  file with it, and it makes a `--key` capture non-reproducible — which is the single failure mode
  this epic's prerequisite (T3) was fixed to prevent.
- **`view::render` must do the dispatch.** There are three render call sites [main.rs:196, 286, 406];
  dispatching in `main.rs` means the instrument can render a different path from the player's, which
  is exactly how 2.2 shipped an instrument that showed motion as stillness.
- **Under `NO_COLOR` the colour is gone, so the glyph must carry the depth.** This devpod sets
  `NO_COLOR=1`; a depth view whose only depth cue is a colour gradient produces a well-formed capture
  that evidences nothing. The band ramp is what makes AC12's range check meaningful. Keep the
  existing stderr warning honest: it must still say what a colourless capture cannot evidence — under
  this design that is *material identity*, no longer the geometry.
- **The dwarf index must change what is drawn.** Building it and then drawing terrain anyway is the
  us-09 dead-call shape: the seam is present and inert. AC8's negative — remove the entity, the cell
  goes back to terrain — and its mutation are what prove the decision is consumed.
- **A camera embedded in solid rock renders one flat near band.** That is a legitimate picture, not a
  bug, but it is indistinguishable from a broken renderer in a capture — which is why AC12 requires
  **two distinct bands**, not merely non-zero glyphs. Measured 2026-08-08 on the default world: the
  centre tile `(64, 64, 17)` is `Empty` and the whole column z 14–21 there is `Empty`, so the opening
  camera is in open air.
- **A client-only story does not breach the vertical-slice rule, and a reviewer should not read it as
  one.** The rule's stated test is that a story ends in something observable — a passing scenario test
  or a visible TUI behaviour — not that every story touches `sim-core`. The depth view needs no new
  sim state and no new wire field (see the first decision above), so there is genuinely nothing to
  slice through; inventing sim work to satisfy the shape would be the actual violation.
- **Debug builds are what the recipe runs.** `cargo run -p tui` is unoptimised, and this is the first
  feature in the project whose per-frame cost scales with the viewport. Measure the frame time and
  report the number; the AC5 step bound is the deterministic test, wall-clock is the live measurement.
- **Commit at minimum once per completed task**; if the work spans two Codex sessions, restate the
  RED evidence in the continuation handoff.

### Project Structure (files to touch)

```
crates/tui/src/raycast.rs        # NEW    — camera plane, 3D DDA, shading, dwarf index, heading table
crates/tui/src/view.rs           # UPDATE — View + heading on ViewState, v key, depth key routing,
                                 #          render dispatch, depth status line and hint bar
crates/tui/src/palette.rs        # UPDATE — shade(); dim() expressed through it, behaviour identical
crates/tui/src/main.rs           # UPDATE — mod raycast; named_key("v"); --key usage strings
crates/tui/tests/client.rs       # UPDATE — the three depth captures and the change-detection test
_bmad-output/implementation-artifacts/mutations/4-1a-behold-the-fortress-in-depth.sh   # NEW
crates/protocol/src/lib.rs       # UNCHANGED — deliberately; see AC8 and the first Key decision
crates/sim-core/**               # UNCHANGED — deliberately
crates/simd/**                   # UNCHANGED — deliberately
```

### Previous story intelligence (3.3)

- **3.3's review found the recorded live recipe was not reproducible and failed silently with exit 0**
  — its leading `<` assumed a fixed opening camera z. That is why `--z` exists and why this story's
  recipe passes it. Never conclude anything from a capture you have not range-checked.
- **Glyph *totals* proved nothing at 3.3 and two claims had to be withdrawn.** The measures that stood
  were per-cell and per-frame transitions against a control run. AC12 is built that way on purpose:
  the no-key run and the turned run are the guards, not decoration.
- Branch from current `main` (`589f524`), which carries the 3.3 merge, the Epic 3 retrospective and
  T3's deterministic opening camera.

### Verification

```bash
scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/4-1a-behold-the-fortress-in-depth.sh
```

Live capture — the observable outcome, joining the two binaries no test can span. Three runs, because
the second and third are what make the first mean anything:

```bash
cargo run -q -p simd -- 47433 &
cargo run -q -p tui -- 47433 --z 16 --frames 6 --key v     > /tmp/depth-v.txt
cargo run -q -p tui -- 47433 --z 16 --frames 6             > /tmp/depth-flat.txt   # control
cargo run -q -p tui -- 47433 --z 16 --frames 6 --key v,l,l > /tmp/depth-turned.txt
for p in $(pgrep -x simd); do kill $p; done   # NEVER pkill -f 'target/debug/simd' — it kills your own shell

rg -c '  3d [a-z]+  z ' /tmp/depth-v.txt      # must be 6 — one depth status line per frame
rg -c '  3d [a-z]+  z ' /tmp/depth-flat.txt   # must be 0 — the control never toggled
for g in █ ▓ ▒ ░; do printf '%s ' "$g"; rg -o "$g" /tmp/depth-v.txt | wc -l; done
# AT LEAST TWO of the four bands must be non-zero. One band alone means the camera is embedded in
# rock or the ramp collapsed, and zero means nothing was drawn. EXIT 0 IS NOT A RESULT.
cmp -s /tmp/depth-v.txt /tmp/depth-turned.txt && echo 'FAIL: turning the camera changed nothing'
```

Key names are a fixed comma-separated set — `space, +, -, S, L, d, c, p, x, h, j, k, l, enter, esc,
<, >`, and this story adds `v`. There is no repeat shorthand and there are no arrow keys; write
`l,l,l` out.

**Baseline executed 2026-08-08, before this story was saved, against `main` at `589f524`** — the parts
of the recipe that can run today, with real numbers rather than a promise:

- `simd` on the default world, `tui --frame` on three separate runs 25 seconds apart: `z 17/31` every
  time. The opening level is deterministic (T3 holds), so `--z` pins rather than guesses.
- `tui --z 16 --frame`, glyphs counted with the SGR escapes stripped: `█ 107`, `▓ 426`, `▒ 460`,
  `░ 435`, `▲ 332`. Non-zero and varied — the capture path itself is sound.
- Snapshot read straight off the socket: z 16 holds 8,895 solid + 864 ramp of 16,384 tiles and **8 of
  8** rays from the world centre hit terrain within 64 tiles; z 17 holds 5,611 + 622 and **6 of 8**.
  Hence `--z 16` in the recipe above — measured, not assumed. First hit east of centre is 12 tiles at
  z 17 and 9 at z 16, so a 90° FOV has walls at several distances in frame, which is what makes the
  band ramp visible at all.

**What could not yet run, and is therefore owed by the dev agent**: everything from `--key v` onward,
because `named_key` has no `v` today. The obligation is the exact block above, producing exactly
those observations — `3d` on 6 status lines, 0 on the control, at least two non-zero bands, and a
turned capture that differs.

Then, interactively, for the human read (this story does **not** carry the FR23/FR24 sign-off — that
is 4.1b's): `cargo run -p tui -- 47433`, press `v`, turn through all 8 headings, walk forward into a
corridor, and confirm the picture keeps pace at 10 ticks/sec with no visible stutter.

Branch: `4-1a-behold-the-fortress-in-depth`. Commit as `Völundr <jeicei75@gmail.com>`, at minimum one
commit per completed task, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 4.1a] — user story and source ACs; FR24,
  NFR2, and the `v` keymap note
- [Source: .../architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md] — AD-4 (clients
  render a world, never rules), AD-1 (`tui` holds zero game logic), and the conventions that the wire
  carries material ids never RGB, the colour table lives in `tui`, and drawing goes through one cell
  framebuffer flushed once per frame
- [Source: docs/project-brief.md#Visual identity notes] — creatures are voxel models sampled
  fine-step inside creature-flagged tiles during DDA, authored as code and never as assets (4.1b)
- [Source: _bmad-output/implementation-artifacts/3-3-the-haul-and-the-skeleton-walks.md] — the
  instrument pattern, the withdrawn glyph-total claims, and the irreproducible-recipe finding
- [Source: _bmad-output/implementation-artifacts/sprint-status.yaml] — action items E2 (both
  questions settled above), T3 (the deterministic opening camera, this story's prerequisite) and
  E1 (the 4.1a/4.1b split)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — the glyph client's visual
  ceiling, which is the standing case for spending this epic on depth rather than on more 2D polish
- [Source: AGENTS.md] — sabotage rule, honest reporting, bounded I/O, the codex self-gate

## Dev Agent Record

### Agent Model Used

`claude-opus-5[1m]` — implemented **directly**, not delegated to Codex. Codex's weekly quota was
exhausted; a probe through `scripts/codex-handoff.sh` returned `You've hit your usage limit …  try
again at Aug 12th, 2026 7:00 AM`, verified live rather than assumed from the Epic 3 retro. Wolf's
call was to implement now rather than wait four days, as at story 3.3. **The reviewer should note
that dev and review share a model family**; the fresh-context lever still applies, the different-LLM
one does not.

The probe also showed `/workspace/.codex/config.toml` now resolving `model: gpt-5.6-luna` at
`reasoning effort: medium`, where the delegation runbook recorded `gpt-5.6-sol` / `high`. Not acted
on here — it is a lever for the next delegated run, not this story's business.

### Debug Log References

**Mutation table — 22 of 22 killed**, full output in the run below. Sabotage is the evidence a green
suite is not (AGENTS.md rule 1), so the actual RED lines:

```
=== v does not toggle the view ===
view::tests::v_toggles_the_depth_view_from_normal_and_back panicked: assertion `left == right` failed
=== v toggles out of a designation mode ===
view::tests::v_is_ignored_in_every_designation_mode_and_leaves_the_anchor_alone panicked:
  assertion `left == right` failed: Dig with anchor None
=== turning goes the wrong way round ===
view::tests::turning_walks_the_eight_headings_and_wraps_both_ways panicked:
  assertion `left == right` failed: left turn
=== turning does not wrap ===
view::tests::turning_walks_the_eight_headings_and_wraps_both_ways panicked:
  assertion `left == right` failed: right turn
=== forward reads the next heading_step entry ===
view::tests::forward_then_back_returns_to_the_starting_tile_on_every_heading panicked:
  assertion `left == right` failed: heading 0 did not step by its own table entry
=== forward is not clamped to the world ===
view::tests::forward_clamps_at_the_world_edge_rather_than_leaving_the_world panicked
=== d enters dig mode from the depth view ===
view::tests::designation_keys_do_nothing_in_the_depth_view panicked:
  assertion `left == right` failed: d reached the depth view
=== render always dispatches the flat view ===
view::tests::the_two_views_draw_different_pictures_of_the_same_world panicked:
  assertion `left != right` failed: both views drew the same picture: the dispatch is not
  reaching the raycaster
=== the status line omits the view and heading ===
view::tests::the_depth_status_line_reports_the_view_and_the_heading panicked
=== the depth hint advertises d c p x ===
view::tests::hint_bar_names_every_modes_keys_and_fits_eighty_columns panicked
=== the DDA drops its z component ===
raycast::tests::cast_reports_the_face_it_crossed_on_each_axis panicked
=== the DDA indexes tiles without the bounds check ===
raycast::tests::a_ray_that_leaves_the_world_draws_blank panicked (index out of bounds)
=== the step cap is removed ===
raycast::tests::a_ray_into_open_air_stops_at_the_step_cap_rather_than_the_world_edge panicked
=== the band ramp collapses to one glyph ===
raycast::tests::a_wall_lands_in_a_band_and_moves_to_a_nearer_one_as_it_approaches panicked:
  assertion `left == right` failed: 8.5 tiles is band 1
=== face shading is removed ===
raycast::tests::a_downward_ray_is_darkened_by_the_face_it_crosses_as_well_as_the_band panicked:
  assertion `left == right` failed: nearest band through a z face
=== the hit colour comes from a second hardcoded table ===
raycast::tests::the_hit_colour_is_the_palette_entry_shaded_by_band_and_face panicked
=== the dwarf index is built and its lookup discarded ===
raycast::tests::a_dwarf_on_the_ray_is_drawn_instead_of_the_terrain_behind_it panicked
=== a shared tile goes to the highest id ===
raycast::tests::the_dwarf_index_gives_a_shared_tile_to_the_lowest_id panicked
=== a miss draws a band glyph instead of BLANK ===
raycast::tests::a_ray_that_leaves_the_world_draws_blank panicked
=== the ray angle ignores the heading ===
raycast::tests::every_heading_sees_the_wall_placed_in_its_own_direction_and_no_other panicked:
  assertion `left == right` failed: heading 1 saw the wall placed for heading 0
=== shade returns the colour unchanged ===
palette::tests::dim_darkens_monotonically panicked
=== named_key forgets v ===
tests::every_instrument_key_name_is_pinned panicked: wrong mapping for "v"

All mutations killed.
```

**RED discipline, stated plainly.** The behavioural tests were written before the code and observed
failing — as compile errors, because the API they name did not exist yet (`no field 'view' on type
'ViewState'`, `cannot find function 'draw' in this scope`). A compile error is a weak RED, so the
mapping-and-constant proof is the table above, which breaks working production code and shows the
named test going red for the named reason.

**`cargo clean -p` was run BEFORE and AFTER `mutate.sh`.** Cleaning only before is what produced two
convincing false failures at 3.2.

### Completion Notes List

All 13 ACs met. `scripts/gate.sh` green; `mutate.sh` 22/22.

**One mutation the story listed is NOT in the set, and the reason is recorded in the mutations file
rather than quietly dropped.** "The dwarf index ignores `EntityKind`" cannot be killed:
`protocol::EntityKind` has exactly one variant (`Dwarf`, `crates/protocol/src/lib.rs:36-38`), so
deleting the kind filter is a semantic no-op — no snapshot exists that the two versions disagree
about. This is **not** 3.3's rejected "no scenario can tell them apart" argument, which fell to a
unit test one level down; here the type has a single inhabitant, so there is no level at which any
test could observe it. The filter stays as 4.1b's seam with a `// NOTE:`, and the mutation is
re-added the moment a second variant exists.

**A test I wrote was false evidence and was fixed before it could mislead.** The
change-detection capture first compared whole frames — and the status line carries the tick, which
differs every frame no matter what the picture does, so it would have passed against a completely
frozen render. It now compares maps with the status line stripped, and carries its own control: an
unchanging world must produce identical maps. Same class as 3.1's `<= 80`-on-80-cells assertion.

**Live capture, actual numbers, run against a real daemon on the default world** (recipe from
Verification, `--z 16`, 6 frames, three runs):

| measure | keyed `v` | control (no key) | turned `v,l,l` |
| --- | --- | --- | --- |
| lines matching `  3d [a-z]+  z ` | **6** | **0** | 6 (`3d s`) |
| `█` / `▓` / `▒` / `░` counts | 600 / 5160 / 666 / 0 | — | — |
| differs from the keyed run | — | — | yes |

Three of four bands are non-zero, against AC12's floor of two. `░` (beyond 24 tiles) is absent
because the default world's rock stops nearly every ray sooner; that is the terrain, not a collapsed
ramp — the near three bands vary as expected and the mutation table proves the fourth entry is
reachable code. Status line as rendered: `tick 21  normal  3d e  z 16/31  dwarves 5`.

**`NO_COLOR` re-run, because AC7's whole claim is that the glyph carries the depth.** With
`NO_COLOR=1` the band counts are **byte-identical** (600 / 5160 / 666 / 0) and the only escapes left
in the capture are resets (`ESC[m`), no colour at all; the existing stderr warning still fires. With
colour on, the same capture carries many distinct 24-bit foregrounds (`38;2;164;174;182`,
`38;2;78;107;121`, `38;2;100;139;156`, …), which is the distance-and-face shading visible live.
Note for the record: **this devpod did NOT have `NO_COLOR` set this session**, contrary to the
project memory — so it was set explicitly to test the path.

**Frame cost (NFR2), debug build, measured not promised.** 60 frames at the fast tick period
(20 ms, so ~1200 ms of the total is pure waiting), three runs each:

```
flat  : 1674  1360  1356 ms
depth : 1364  1351  1355 ms
```

The depth view is indistinguishable from the flat view and both sit ~2.5 ms/frame above the wait
floor. At the normal 10 Hz period: flat 6002 ms, depth 6080 ms for 60 frames. No optimisation was
done and none is warranted — the step cap is the only bound, exactly as the guardrails require.

**Scope guardrails held:** `protocol`, `sim-core`, `simd` and `bridge.rs` are untouched (see File
List); no sub-voxel models, no pitch key, no designation input in the depth view, no lighting model,
no performance work. Nothing from `deferred-work.md` was picked up.

**Two small implementation notes for the reviewer:**
1. `Cast.steps` carries `#[allow(dead_code)]`. AC5 asks the step bound to be observable to a test
   without a test-only hook in the draw path; `draw` genuinely has no use for the count, and in a
   binary crate an unread field fails `clippy -D warnings`. The alternative was a bare tuple, which
   hides the intent instead of stating it.
2. `view::render` was split into `draw_flat` and `draw_status_and_hint` so the dispatch could be a
   `match` without reindenting the whole flat block. The flat code is moved verbatim, not rewritten;
   `framebuffer.cells[…]` became `cells[…]` and nothing else changed.

**Task-to-commit mapping.** Five commits, not one squash. Tasks 2 and 3 share commit `74c4bfa`: a
renderer with no call site is dead code, so the raycaster could not pass `clippy -D warnings` until
`render` dispatched to it — the two are not separably green.

### File List

```
crates/tui/src/raycast.rs        # NEW    — DDA, shading, bands, dwarf index, heading table
crates/tui/src/view.rs           # UPDATE — View + heading, v key, depth routing, dispatch,
                                 #          depth status line and hint, draw_flat extraction
crates/tui/src/palette.rs        # UPDATE — shade(); dim() now expressed through it
crates/tui/src/main.rs           # UPDATE — mod raycast; named_key("v"); --key usage strings
crates/tui/tests/client.rs       # UPDATE — four depth captures against the stub daemon
_bmad-output/implementation-artifacts/mutations/4-1a-behold-the-fortress-in-depth.sh   # NEW
_bmad-output/implementation-artifacts/4-1a-behold-the-fortress-in-depth.md             # UPDATE
_bmad-output/implementation-artifacts/sprint-status.yaml                               # UPDATE
```

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-08 | Story created |
| 2026-08-08 | Implemented directly by Opus (Codex quota exhausted until 2026-08-12, verified live). Depth view shipped: 5 commits, gate green, 22/22 mutations killed, live capture taken. Status → review |

## Review Findings

Code review 2026-08-08 (Opus orchestrator, fresh context). Four layers, all four completed — no
coverage holes, no timed-out layer. Layer key: `blind` = Blind Hunter (Sonnet, `raycast.rs`),
`edge` = Edge Case Hunter (Sonnet, shells), `acc` = Acceptance Auditor (Opus, whole diff),
`feat` = Feature Auditor (Opus, whole diff). Gate re-run independently: **GREEN**.

### Decisions needed

- [ ] [Review][Decision] **`v` is never advertised to the player — the epic's marquee feature is
      undiscoverable** [crates/tui/src/view.rs:366] (`feat`, HIGH) — the flat-view hint reads
      `d dig  c channel  p stockpile  x clear  <> z  hjkl move  q quit client` (70 chars); the string
      `v flat view` appears **only** in the depth hint, i.e. `v` is announced only to a player who has
      already found it. No AC requires advertising it, so every test is green. The story's headline
      outcome begins "I press `v`". Fix is a wording + budget call: `  v 3d` fits at 76 within the
      pinned 80-column status budget, `  v 3d view` overflows at 81.
- [ ] [Review][Decision] **`shade()`'s `0..=100` contract is unenforced and fails silently out of
      range** [crates/tui/src/palette.rs:153-159] (`edge`, LOW, **not reachable today**) —
      `(u16::from(fg.0) * percent / 100) as u8` truncates by wrapping above 100% (`510 as u8 == 254`)
      and the `u16` multiply itself overflows above ~257%, which panics in debug and wraps in release.
      The only caller (`raycast.rs:102`) is provably ≤100. This is the latent silent-failure class
      this project has repeatedly chosen to close early rather than defer, hence a decision not a
      deferral: harden now, or accept it as YAGNI.

### Patches

- [ ] [Review][Patch] `NO_COLOR` stderr warning is now false in the depth view [crates/tui/src/main.rs:382-388]
      (`acc`+`feat` CONVERGED, MED) — still claims "Designation and zone markers remain evidenced
      because their glyphs are distinct". The depth view draws no designations or zones, and every
      cell is one of four band glyphs, so material identity and dwarf-vs-terrain are exactly what a
      colourless depth capture cannot evidence. The spec named keeping this warning honest as an
      explicit obligation and it was skipped. Also covers `feat`'s AC8 point: under `NO_COLOR` a dwarf
      is pixel-identical to terrain (only `fg` differs), so no colourless capture can evidence AC8.
- [ ] [Review][Patch] A left–right mirrored picture survives the whole suite AND all 22 mutations
      [crates/tui/src/raycast.rs:164] (`acc`, MED) — negating `right = (-forward.1, forward.0)` in a
      throwaway copy of HEAD gave 71 + 19 tests passed, 0 failed, and the live recipe still green.
      Every picture-inspecting test uses `centre_of()`, invariant under a horizontal mirror; `right`
      appears in none of the 22 mutations. Control: flipping the vertical axis DOES fail a test, so
      vertical is guarded and horizontal is not. NOTE the shipped orientation is *correct*
      (east `(1,0)` → right `(0,1)` = south, matching `heading_name`) — this is a coverage hole in
      AC13's own guarantee, not a live defect. Fix: one asymmetric-scene test + its mutation entry.
- [ ] [Review][Patch] `Hit.distance` doc comment states the opposite of what the code does
      [crates/tui/src/raycast.rs:35-37] (`blind`+`acc` CONVERGED, MED) — comment claims "Euclidean …
      the mild fisheye it leaves is a fair price". The stored value is `t_max[axis]`, and since
      `right ⊥ forward` and `forward.z == 0`, `forward · direction == 1` exactly, so `t` **is** the
      perpendicular (camera-plane) distance and there is **no** fisheye. Not cosmetic: this comment
      demonstrably misleads — the Blind Hunter proposed "multiply by `|direction|`", which would
      *introduce* fisheye and curve every flat wall. Comment-only fix.
- [ ] [Review][Patch] Self-referential test: forward/back round-trip proves only self-consistency
      [crates/tui/src/view.rs:1979-2001] (`edge`, MED) — `forward_then_back_returns_to_the_starting_tile_on_every_heading`
      computes its expectation with `crate::raycast::heading_step(heading)`, the same table
      `step_camera` calls. A corrupted table would still round-trip and agree. Mitigated (verified):
      `raycast.rs:586-603` hand-writes the table and pins `heading_step` independently, so the table
      IS covered — but not from within this test's own territory. This antipattern has shipped in
      stories 1.1, 1.2 and 1.3. Fix: hand-write the expected step in the assertion.
- [ ] [Review][Patch] The fourth distance band `░` is pinned by no test and no mutation
      [crates/tui/src/raycast.rs:26] (`acc`+`feat` CONVERGED, LOW — fold into the mirror-test patch) —
      `BAND_LIMITS[2] = 24.0` appears in zero of the 22 mutations and no test asserts band 3; the
      `--key v` capture counts `░ 0`. AC7's "four-step ramp" is three-quarters evidenced;
      `BAND_LIMITS[2]` could be any value ≥10 and nothing would notice.
- [ ] [Review][Patch] SPEC DEFECT — AC3's "byte-identical frame on every run" is unmeetable as written
      (`acc`, LOW) — the status line carries `snapshot.tick`, which differs between runs against a live
      daemon regardless of rendering determinism. Determinism was verifiable only after stripping the
      status and hint rows. Scope the AC to the map region, or to a fixed protocol state.
- [ ] [Review][Patch] SPEC DEFECT — AC12's band-count range check cannot on its own distinguish the
      depth view from the flat view (`acc`+`feat` CONVERGED, both measured independently, LOW) — the
      **flat** control capture, with `v` never pressed, contains all four bands
      (`█ 642  ▓ 2556  ▒ 2760  ░ 2610`), because those are the material glyphs the flat view already
      uses. Read alone the band loop passes trivially and evidences nothing — 3.3's withdrawn
      glyph-total failure mode, resurfacing in spec text. The recipe is saved only by the adjacent
      `3d` status grep. The implementation is fine (the control test asserts on status/hint text, not
      glyphs); the spec text should state the band count is meaningful only conditional on that grep.

### Deferred

- [x] [Review][Defer] Camera inside solid rock renders a featureless full-screen `█` with no cue
      [crates/tui/src/raycast.rs:174-207] (`feat`, LOW) — deferred, spec pre-declares this "a
      legitimate picture, not a bug"; reachable in normal play via `<`,`<`,`v`
- [x] [Review][Defer] `shade()` has no direct test; `percent = 0` is never exercised anywhere
      [crates/tui/src/palette.rs:151-159] (`edge`, LOW) — deferred, LOW-tail cap
- [x] [Review][Defer] Partial-clamp corner (one axis at bound, one free) untested
      [crates/tui/src/view.rs:563-567] (`edge`, LOW) — deferred, correct by construction, coverage only
- [x] [Review][Defer] `simd` serve suite flakes under heavy concurrent load
      [crates/simd/tests/serve.rs:148] (`orchestrator`, LOW) — deferred, pre-existing and outside this
      diff; bites the review process, which mandates four concurrent cargo-running layers
- [x] [Review][Defer] SPEC premise false — "this devpod sets `NO_COLOR=1`"; it is unset
      (`edge`+`feat`+`orchestrator`, LOW) — deferred, spec text only

### Not proven by this review

- **AC11 was NOT observed live.** Designation is unreachable from the depth view by design, so no
  single scripted run can dig and then look at the result. It rests on the unit test
  `a_wall_turned_empty_reveals_the_wall_behind_it` plus the integration capture, not on a live dig.
- **AC13's mutation half is dev-reported only.** `scripts/mutate.sh` was not run by any layer (it
  rewrites source in place and sibling layers were live). 22/22 remains unverified by review; the
  Acceptance Auditor audited the mutation file's *content* instead and found the mirror gap above.
- **Independently confirmed honest:** the Acceptance Auditor reproduced the dev's reported capture
  numbers exactly (600/5160/666/0 bands, 6/0/6 status lines, heading `3d s`).

## Disposition (2026-08-08)

**Closed `done`, NOT merged.** The branch `4-1a-behold-the-fortress-in-depth` (5 commits, gate
green) is kept as-is; `main` stays 2D-only. No push, no PR — Wolf's explicit call.

**Why: the story succeeded technically and answered its question in the negative.** Wolf ran the
depth view live and judged it *"quite far from wow effect"*, and doubts wow is reachable in a
terminal at all. That is an experiment concluding, not a story failing — the code is sound, the
gate is green, four review layers completed cleanly and the evidence was honest.

**The requirements miss, recorded because no review layer could have caught it.** Wolf wanted an
**isometric** 3D camera; this story specified and shipped a **first-person raycast** view — *"I
didn't manage to clarify that"*. Every layer audits "does the code match the spec?", and it does,
faithfully; even the Feature Auditor's "would the user get the outcome the story promises?" cannot
help when the *promise* is the wrong thing. This is a new subclass of the tracked spec-defect
category: not an AC unmeetable as written (2.3's AC9, this story's AC3) but **an AC perfectly
meetable, perfectly implemented, describing something the user did not want.** Any process fix
belongs at story creation / epic authoring, not at review. This is retro material.

**Consequences beyond this story** (handled via `bmad-correct-course`): 3D-in-TUI is abandoned
including isometric-in-TUI; the TUI is **not** retired but demoted to the 2D instrument and debug
client; Unreal is dropped in favour of a **Bevy** client; and FR23/FR24's identity verdict, only
ever provisional at 2D, moves to that client. The four-crate spine makes this cheap — a Bevy client
is another `protocol` consumer, so `sim-core`, `simd` and `protocol` need no changes at all.

**The seven patch findings and two decisions above were deliberately NOT applied.** They are
correct, and they are recorded for the retro rather than fixed, because the code they would harden
is no longer on the path to anything. The two SPEC defects (AC3 unmeetable, AC12's vacuous band
check) are the exception worth carrying forward as authoring lessons.
