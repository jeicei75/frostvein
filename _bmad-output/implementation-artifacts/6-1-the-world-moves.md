---
baseline_commit: 1f262d8
model: claude-opus-5[1m]  # the policy default (Opus); recorded because 5.4 ran on claude-fable-5 and an unlabelled neighbour row is exactly the ledger ambiguity the model policy exists to prevent
---

# Story 6.1: The World Moves

Status: in-progress

## Story

As the boss,
I want the valley to visibly live — dwarves walking and working, light flickering, rubble piling
at the dig face — with no command from me,
so that the beautiful still image becomes a running simulation in front of my eyes.

**This is wow beat 2 (UX-DR14) — the beat the PRD calls the magic, and a client that achieves only
beat 1 has FAILED the milestone. The sign-off gate binds BOTH halves (UX-DR22) and the story
CANNOT BE CLOSED BY A DEV AGENT.**

## The sign-off gate — read before touching any code

**Opening half (Task 0, blocking):** no implementation commit before Wolf has approved one "here is
what you will see" artifact. For a *motion* story a still frame cannot carry the bar, so the
artifact is three parts: (a) a real before/after capture pair of our actual world at the boot
framing, produced on the vehicle by the **already-shipped 5.4 binary** — no new code, the dig and
its rubble already work; (b) a written list of the four things this story adds on top of that pair;
(c) an explicit **"what you will NOT see"** list. Part (c) is not optional: 5.4's artifact drew
spruce sprites the renderer was never tasked to produce, and the mismatch surfaced only at Wolf's
live viewing (deferred-work.md:635-642 — "Task 0 artifact scripts must not substitute geometry the
renderer is not tasked to produce").

**Closing half (AC17):** done only when Wolf has viewed the built result **live** on the vehicle,
compared it against the approved artifact, and signed off wow beat 2. A capture serves the
comparison; it never replaces the live viewing (AD-17).

## The live vehicle — where the window actually opens

Unchanged from 5.4 and not to be re-derived: **no devpod can open a window** (no graphics
userspace — measured at 5.3, both fallbacks walked to the end). **The proven vehicle is the native
Windows client on gingerspice**: cross-compiled `gui.exe`, `simd` in WSL, localhost, native NVIDIA
Vulkan. 5.4 measured **>135 fps at every zoom including full vista** there, so this story starts
with 4.5x headroom over NFR6's vista bar. Everything in this story except the live viewing, the
NFR6 reading and the captures is headless-testable in any devpod under `MinimalPlugins`.
**Never fake the live half.**

## Acceptance Criteria

### The gate

1. Before any implementation commit, Wolf has approved the sign-off artifact, stored at
   `_bmad-output/implementation-artifacts/6-1-signoff/`.

### Motion between ticks

2. A world-projected entity whose mirror position changed renders **strictly between** its
   previous-tick and current-tick render positions while the blend factor is in `(0,1)`, and
   exactly at the current-tick position at factor `1` — dwarves move smoothly instead of snapping
   tile to tile (AD-15, FR34).
3. The client never shows a state the wire did not deliver: at any elapsed time at or beyond one
   tick interval the render position equals the current-tick position **exactly**, and no render
   position ever lies beyond it along the previous→current segment. There is no prediction and no
   extrapolation path in the code (AD-15).
4. **A rewind snaps.** On the first frame after a `snapshot` (connect or AD-11 load) every
   world-projected entity renders exactly at its snapshot position whatever the blend clock reads
   — nothing blends across a snapshot (AD-15).
5. The blend is the **sole writer** of world-projected entity translation after spawn:
   reconciliation no longer re-inserts a snapped `Transform` every frame, and a headless test fails
   if it does. *(Mechanism is the requirement here: a blend that is computed and then overwritten
   by the next reconcile pass is the present-but-inert seam — the exact defect the seam-exercised
   rule exists for.)*
6. AC2–AC5 are asserted headlessly under `MinimalPlugins` in `cargo test` (AD-17 rung 2), and the
   blend and flicker systems are registered by **one function that the live `App` and the headless
   test both call**, so dropping a system from the live wiring turns a test red. *(Mechanism, same
   justification: `run()` has no test of any kind today — deferred-work.md:626-630 — so a system
   silently missing from the live tuple is invisible to the whole suite.)*

### Work leaves evidence at the dig face

7. **The named dig site is `[58,68,9]`–`[64,69,9]`** — 8 mineral tiles (ice/snow), every one
   sky-exposed and unoccluded at the boot framing, designated **from a TUI client on the same
   daemon** (the Bevy client issues no commands until Epic 8). When it is dug, the sim's stone
   items render as world-projected entities at the dug tiles and stay there — no stockpile is
   placed, so the rubble accumulates and a worked site never looks spotless (FR34, UX-DR9, AD-14).
8. Each wire tile change that leaves a tile `Empty` also spawns deterministic **client-local**
   debris chips at that position: `ClientLocal`, never `WorldProjected`, no sim meaning, cleared by
   a snapshot rebuild, and identical for the same position on every run (NFR5's carve-out, AD-15).
9. The named dig site projects **inside the frame** at the boot camera rig, asserted headlessly via
   `CameraRig::project_world_point` — so a capture aimed there cannot repeat 3.3's zero-of-every-
   glyph-with-exit-0 failure.

### Flicker

10. Torch and campfire light flickers as **client-side animation driven from the `gui` light table
    and never from the wire**: the flicker value is a pure deterministic function of the emitter's
    simulation id and the client's elapsed seconds, stays inside a named band around the table
    intensity, and differs both between the two kinds and between two emitters of the same kind —
    no synchronised pulse (FR34, AD-15, AD-16).
11. The flickered `PointLight` survives the next reconciliation pass unchanged. *(Mechanism, same
    seam-exercised justification as AC5: reconcile re-inserts `point_light()` on every frame today,
    which would silently erase every flicker.)*

### Aliveness with no commands

12. With zero commands issued, motion is measurable, not asserted by eye: over an observation of at
    least 100 delivered ticks the instrument counts a non-zero number of dwarf position changes and
    a non-zero number of frames that rendered a mid-blend entity (FR34, UX-DR19; M1's FR4 aliveness
    now visible in 3D).

### NFR6 — measured where a window exists

13. With the full 128×128×32 world, all dwarves moving and all lights animating, the frame-time
    overlay reads a sustained **60 fps at working zoom** and **≥30 fps at full vista** on the live
    vehicle, recorded **labelled with its machine**. The WSLg-devpod figure NFR6 names is
    unmeasurable there (5.3's envelope finding) and stays formally owed to the Epic 5/6 retro's
    bar-redefinition question — recorded, never blurred.

### The instrument

14. `gui <port> --capture <path> --frames N [--expect-work]` accumulates over the whole run and
    prints one motion line before any conclusion — ticks observed, dwarf position changes, frames
    with a mid-blend entity, maximum concurrent working dwarves, item count — and **asserts**:
    ticks observed ≥ 100, position changes > 0, blend frames > 0, and, with `--expect-work`,
    maximum working dwarves ≥ 1 and items ≥ 1. 5.4's pixel range checks are retained unchanged.
    **Exit 0 is not a result** (AD-17 rung 3).
15. The before/after capture pair differs **inside the dig site's projected screen window**, not
    merely in whole-file bytes: snowfall and the aurora animate client-locally every frame, so
    whole-frame inequality is satisfied by atmosphere alone and proves nothing about the sim. The
    `--ignored` capture self-test (`crates/gui/tests/capture.rs`) is updated to compare that window
    and is executed on the vehicle, as it was for 5.4.
16. A `tui` client on the same daemon is run beside the Bevy client during the live session as the
    cross-check that the motion reflects real sim state and not client invention (AD-17 rung 1).

### The closing half

17. Wolf has viewed the built result live on the vehicle, compared it against the approved
    artifact, and signed off **wow beat 2** — the moment the beautiful still image becomes a running
    simulation (UX-DR14, UX-DR22). A dev agent cannot check this box.

### Evidence

18. A sabotage table exists at
    `_bmad-output/implementation-artifacts/mutations/6-1-the-world-moves.sh` and every mutation is
    KILLED on a genuine assertion, with RED output pasted into the Dev Agent Record.
19. `scripts/gate.sh` is green and the diff touches only `crates/gui`, `docs/` and
    implementation-artifacts: **no wire change** (AD-16's sanctioned M2 diff was spent at 5.1), and
    no change to `sim-core`, `simd`, `protocol`, `client-core` or `tui`. Nothing sim-side changes,
    so the parity rule's backward half does not fire.

## Tasks / Subtasks

- [x] **Task 0 — The sign-off artifact (Wolf's gate, BLOCKING — no implementation before the
      checkbox)** (AC: 1) — **APPROVED BY WOLF 2026-08-17. AC1 MET. The gate is OPEN and
      implementation may start.** He approved the artifact as-is after the pair's finding was
      measured and recorded: the dig face is judged at working zoom, the boot frame for composition
      and motion, and the named dig site is NOT re-picked.
  - [x] Produce the before/after capture pair on the vehicle with the **shipped 5.4 binary** (no
        code change — the dig, its rubble and the boot framing all work today):
        `gui.exe 7451 --capture 6-1-before.png --frames 600`, then designate the site from the TUI
        with the exact key sequence in Verification, then
        `gui.exe 7451 --capture 6-1-after.png --frames 600`. Store both in `6-1-signoff/`.
  - [x] Write `6-1-signoff/what-you-will-see.md`: the four additions — (1) dwarves slide between
        tiles instead of snapping, (2) torch and campfire pools breathe, (3) chips of debris at
        each dug tile, (4) stone rubble that stays (already in the "after" capture) — each with the
        one-sentence look it is aiming for.
  - [x] Write the **"what you will NOT see"** list in the same file, and get each line ruled on:
        *(RULED by Wolf 2026-08-16 — the carried stone: **sign beat 2 without it**, no sim story
        spun, UX-DR14's carried-stone clause formally not delivered in M2. Other six lines drew no
        objection. A seventh line was added and stated: dwarves remain rigid cubes with no walk
        cycle.)*
        no lantern light (6.2, first on the cut list); no z-slicing (7.1); no mouse or commands from
        `gui` (8.x); dwarves remain scaled cubes; trees remain wire-true cube stacks (Wolf's 5.4
        ruling); and — **raise this one explicitly** — **a carried stone does not travel with its
        dwarf**. UX-DR14's wording includes "a dwarf picks something up and carries it", and that is
        not achievable in any client today: `World::items()` reports every item at its last resting
        position and a carried stone keeps its pickup tile until it is dropped
        (`crates/sim-core/src/lib.rs:1462-1471`, `:674-687`, `:891`); the TUI's "carrier" glyph is
        only "a dwarf standing on a tile that has an item" (`crates/tui/src/view.rs:240-251`).
        Making it visible is a sim + wire change and is therefore **out of this story's scope by
        AC19**. Wolf's call: sign beat 2 without it, or spin a separate sim story.
  - [x] *(N/A — the pair WAS taken, so the fallback never fired. Wolf declined it explicitly.)*
        If no vehicle session is available for the pair, the fallback artifact is the written
        `what-you-will-see.md` approved on its own — record that the pair was skipped and why. Do
        not substitute a hand-drawn render for the renderer's own output.

- [x] **Task 1 — The blend clock and the blend** (AC: 2, 3, 5, 6)
  - [x] Add `TickClock` (a `ClientLocal` resource, see the skeleton in Dev Notes) advanced each
        frame from `Time` and reset **only when a delta advances the mirror's tick** — a paused
        daemon keeps emitting deltas at loop rate (AD-2) and resetting on those would corrupt the
        measured interval. Clamp the measured interval to `[MIN_TICK_INTERVAL, MAX_TICK_INTERVAL]`
        so a stalled server cannot produce a huge denominator, and clamp the factor to `[0,1]`.
  - [x] Add `blend_entities`: build the id → `&Entity` map once per frame from `mirror.entities()`,
        then set `transform.translation` for every non-terrain `WorldProjected` entity —
        `previous_entity(id)` present → lerp previous → current by the factor; absent (spawned this
        tick, or after a snapshot) → snap; an id that is an item and not an entity → snap to its
        item position. `// NOTE:` the per-frame map: there are ~10 dynamic entities, so this stays
        cheaper than adding a lookup to `client-core` and keeps the diff gui-only.
  - [x] Stop `reconcile` re-inserting a `Transform` for entities it did not just spawn
        (`crates/gui/src/project.rs:286-297`) — spawn sets translation and scale once, the blend
        owns translation thereafter.
  - [x] `.chain()` the Update systems in the load-bearing order `ingest_messages →
        reconcile_projection → blend_entities → flicker_lights` (this also closes the incidental-
        ordering finding at deferred-work.md:605-609, which is now load-bearing rather than
        incidental).
  - [x] Extract the Update registration into one function (e.g. `pub fn projection_systems(app)`)
        called by `run()` **and** by the headless tests, so a dropped system is a red test.
  - [x] Tests: strictly-between at factor 0.5 (assert against a hand-written expected midpoint, not
        against the production lerp); factor 1 lands exactly on the current position; elapsed far
        past the interval still lands exactly on the current position and never beyond; an entity
        with no previous state snaps; a re-run of reconcile after the blend does not move the
        entity back to its snapped position (AC5).

- [x] **Task 2 — A rewind snaps** (AC: 4, 6)
  - [x] Verify (do not re-implement) that `Mirror::apply_snapshot` clears `previous_entities`
        (`crates/client-core/src/lib.rs:50-54`, `:147-168`) and that the blend therefore cannot
        cross a snapshot. Reset the blend clock on a snapshot as well, so the first post-snapshot
        frame has no stale elapsed time.
  - [x] Test: run frames with a blend in progress, apply a snapshot placing the same entity far
        away, run one frame at a mid-range clock, assert the transform equals the snapshot position
        exactly.

- [x] **Task 3 — Flicker** (AC: 10, 11, 6)
  - [x] Extend `LightProperties` with the flicker columns the 5.4 `// NOTE:` at
        `crates/gui/src/appearance.rs:50` promises, and delete that NOTE. Add `flicker_scale(kind,
        id, seconds)` — pure, no RNG, no wire input (skeleton in Dev Notes).
  - [x] Add a `ProjectedLight(LightKind)` component written by `reconcile` on spawn and only when
        the kind changes, so reconcile stops re-inserting `point_light()` every frame
        (`crates/gui/src/project.rs:293-297`); add `flicker_lights` writing
        `PointLight.intensity` each frame from the table × `flicker_scale`.
  - [x] Tests: the scale stays inside the named band across a time sweep; two ids of the same kind
        diverge at the same instant; the two kinds diverge; the function is deterministic for the
        same `(id, seconds)`; and an app-level test that runs reconcile after the flicker and finds
        the flickered intensity intact (AC11).
  - [x] `// NOTE:` that only the point light flickers, not the emitter's emissive material — per-
        entity materials would mean one `StandardMaterial` handle per emitter.

- [x] **Task 4 — Debris chips at the dig face** (AC: 8, 9)
  - [x] In `reconcile`'s dirty-tile branch, when the updated tile is `Empty`, despawn any existing
        chips at that position and spawn `CHIPS_PER_TILE` small `ClientLocal` `DigChip` cubes at
        deterministic offsets derived from the tile position (no RNG). Despawn all chips on a
        snapshot rebuild alongside terrain. Chips take a debris colour from the appearance table:
        the removed tile's material is **not** available — AD-18 forbids double-buffering tiles, so
        the mirror keeps no previous tile.
  - [x] Test: a delta emptying a solid tile spawns exactly `CHIPS_PER_TILE` chips near that
        position, all `ClientLocal` and none `WorldProjected`; a delta that changes a tile to
        something solid spawns none; the same position twice does not double the chips; a snapshot
        rebuild clears them.
  - [x] Test (AC9): `CameraRig::new([64,64,9]).project_world_point(p)` is inside `[0,1]²` for every
        tile of the named dig site.

- [x] **Task 5 — The instrument** (AC: 14, 15)
  - [x] Extend `CaptureState` (`crates/gui/src/capture.rs`) to accumulate across the `--frames N`
        run: ticks observed (distinct mirror ticks), dwarf position changes, frames in which at
        least one entity rendered mid-blend, max concurrent dwarves in `JobState::Work`, and the
        item count at capture time. Print one line, then assert per AC14. Add the `--expect-work`
        flag (and reject it without `--capture`, the way `--capture` already requires `--frames`).
  - [x] Keep the 5.4 warm-pixel and ground-luminance checks exactly as they are — they still guard
        the beat-1 look this story must not regress.
  - [x] Unit-test the accumulator itself against a hand-built sequence of mirror states (a
        stationary world produces zero position changes and fails; a moving one passes) — the
        instrument is an evidence channel and an untested one manufactures false evidence.
  - [x] Update the `--ignored` capture self-test to compare the **dig-site window** computed from
        `CameraRig::project_world_point` plus a margin, replacing the whole-file byte comparison,
        and say in its doc comment why (snowfall alone satisfies byte inequality).

- [ ] **Task 6 — The live vehicle session** (AC: 7, 12, 13, 15, 16)
  - [ ] Cross-compile and launch per Verification. Run the TUI designation with the exact key
        sequence; keep a `tui` client open beside the Bevy window as the AD-17 rung-1 cross-check.
  - [ ] Capture the pair, paste the printed motion line and the printed range-check line into the
        Dev Agent Record, and run the `--ignored` capture self-test on the vehicle.
  - [ ] Read the F3 overlay at working zoom and at full vista **with the dig in progress and all
        lights flickering**; record both figures labelled `gingerspice / native Windows / NVIDIA`.
        If a reading fails NFR6, that measurement is the story's finding and is reported — the
        first suspect is 5.4's cap-slab count (deferred-work.md:631-634), not the blend.
  - [ ] Confirm by eye and state in the record: dwarves slide rather than snap; light breathes;
        chips and rubble sit at the dug tiles; nothing else changed about the beat-1 frame.

- [x] **Task 7 — Tech-art guidelines** (AC: 10 supporting)
  - [x] Add one short section to `docs/tech-art-guidelines.md`: motion is presentation (blend
        between delivered ticks, never predict), flicker semantics and the band the table uses,
        and debris as client-local evidence. Written as the decisions are made, not reconstructed.

- [x] **Task 8 — Evidence and the gate** (AC: 18, 19)
  - [x] Write `_bmad-output/implementation-artifacts/mutations/6-1-the-world-moves.sh` following
        5.4's format; run `scripts/mutate.sh` **alone** (it rewrites source in place) and paste the
        RED table.
  - [x] `scripts/gate.sh` green; confirm the diff touches no crate but `gui`.

- [ ] **Task 9 — Wolf's closing sign-off** (AC: 17)
  - [ ] Wolf views live against the approved artifact and signs off wow beat 2. **A dev agent
        cannot check this box.**

## Dev Notes

### Scope guardrails — do NOT build these here

- **No lanterns.** `LightKind::Lantern` keeps its table row and stays unused until 6.2 (first on
  the M2 cut list). Dwarves carry `light: null` — do not special-case them warm.
- **No z-slicing (7.1), no designation/zone rendering (7.2), no picking or commands (8.x).** `gui`
  still issues zero commands and leaves `designations()` / `zones()` unread.
- **No carried-stone motion.** It is not on the wire (see Task 0). Do not synthesise it in the
  client — that is exactly the drift NFR5 forbids.
- **No wire change and no change outside `gui` + docs.** If an AC seems to need one, that is a
  story-spec defect — raise it, don't code it.
- **No new dependencies, no asset pipeline, no particle crate.** Chips are cubes; flicker is a
  sine. Expected new-dependency count: zero.
- **No greedy meshing / chunking / LOD.** 5.4 measured >135 fps at every zoom; optimise only if
  AC13's reading fails, and then the *measured* problem drives it.
- **No workaround for driver or envelope problems in production code** (5.3's AC9 rule stands).

### What already exists (build on it, do not re-derive)

- **The mirror already holds everything the blend needs.** `apply_delta` fills
  `previous_entities` with each surviving entity's state at the previous tick and clears it on
  `apply_snapshot` (`crates/client-core/src/lib.rs:56-105`, `:139-145`). `previous_entity()` has had
  no live caller since 5.3 and this story is the one that wires it (deferred-work.md:573-576).
- **Per-frame client-local animation already has a precedent to copy:** `fall_snow`
  (`crates/gui/src/atmosphere.rs:291-301`) — `Res<Time>`, direct `&mut Transform`, per-entity phase
  so the field never re-synchronises.
- **The whole 5.4 substrate:** reconciliation with `WorldProjected(Id)` / `ClientLocal` /
  `TerrainTile` / `SnowCap` markers, the appearance tables with literal-oracle tests, the transform
  pair, the orbit rig with `project_world_point`, `--capture <path> --frames N` with its range
  checks, and the headless suite under `MinimalPlugins`.
- **Digging already produces rubble on the wire.** A dig removes the tile and spawns a stone item
  at the dug position unless the material is a tree (`crates/sim-core/src/lib.rs:849-892`), and
  `gui` already projects items as stone cubes (`crates/gui/src/project.rs:318-323`). With no
  stockpile placed, nothing hauls it away.
- **The TUI is already a scriptable command client:** `tui <port> --z N --frames N --key <seq>`
  sends the keyed commands and then streams frames (`crates/tui/src/main.rs:97-192`); the cursor
  opens at `(dims.x/2, dims.y/2)` = `(64,64)` and `d` resets it there
  (`crates/tui/src/view.rs:59-79`, `:387-398`).

### Measured at story-creation (shipped seed, live daemon — not estimates)

- Camp at z 9 near map centre: campfire id 5 at `[64,64,9]`, torches ids 6–9 at `[62,62]`,
  `[66,62]`, `[62,66]`, `[66,66]`; dwarves ids 0–4. The camp floor is z 8, so dwarves stand at z 9.
- **The named dig site `[58,68,9]`–`[64,69,9]` designates exactly 8 tiles** (ice + snow, all
  `Tile::Solid` — `Tile::Ramp` is **not** diggable, `crates/sim-core/src/lib.rs:1339-1341`), all of
  them the top of the world at that column and all unoccluded from the boot camera. Projected
  screen positions run `(0.49,0.70)`–`(0.53,0.73)` — lower centre of frame, inside the camp's light.
- **Timings, measured end to end:** the designation appears on the wire ~2 ticks after the TUI
  command; the first dwarf enters `Work` ~24 ticks later; **all 8 tiles are dug within 52 ticks
  (~5 s)** with up to 3 dwarves working at once; 8 stone items then sit at the site permanently.
  WORK_TICKS is 5 (`crates/sim-core/src/lib.rs:35`), so a single tile's work phase is half a second
  — **an instantaneous sample of "is a dwarf working?" is a coin flip, which is why AC14 asserts a
  maximum over the run and not a value at the shot.**
- **Wander baseline: 47% of ticks (327 of 701) contain at least one dwarf position change** with
  zero commands issued. That is the floor under AC12 and the reason its window is ≥100 ticks.

### Key decisions & traps

- **The gate's opening half is a hard sequence point.** Task 0 is one vehicle session and one
  markdown file; 4.1a was a whole story.
- **Reconcile currently overwrites both things this story adds.** It re-inserts a snapped
  `Transform` (`project.rs:286-292`) and re-inserts `point_light()` (`:293-297`) on **every frame**
  for every existing entity. Left alone, the blend and the flicker are computed and thrown away —
  present-but-inert. AC5 and AC11 exist to make that failure a red test rather than a live
  discovery.
- **Alpha comes from the wire's own cadence, not from a hardcoded tick rate.** The protocol carries
  `Speed` but no tick rate (`crates/protocol/src/lib.rs:62-67`), and a hardcoded 10 tps would be a
  sim rule living in a client. Measure the gap between ticks instead — it then tracks pause and
  fast-forward for free.
- **Paused is not still.** AD-2's loop keeps emitting a delta per iteration while the tick counter
  freezes, so `previous == current` and the blend is the identity. Do not add a pause branch.
- **Two deltas can land in one frame** (`ingest_messages` drains the queue): the blend then runs
  between the last two applied ticks and the entity covers two tiles in one interval. Acceptable;
  do not build a delta queue for it — `// NOTE:` the limitation.
- **Whole-frame "the capture changed" is a vacuous check in this story.** Snowfall and the aurora
  animate every frame, so two captures always differ in bytes. Every motion claim must be anchored
  to sim-derived counts (AC14) or to the dig-site window (AC15).
- **Global counts are the site counts here, deliberately.** The instrument counts working dwarves
  and items world-wide rather than taking a site rectangle: the named dig is the only work and the
  only source of items in the world, so the counts are equivalent — and it keeps world knowledge
  out of the client. `// NOTE:` this at the assertion.
- **Item ids and entity ids share `reconcile`'s `wanted` map.** They cannot collide today (one
  `IdAllocator`) but nothing asserts it (deferred-work.md:613-618). The blend must not widen that:
  look entities up by id from `mirror.entities()`, never assume a `WorldProjected` id is an entity.
- **The capture accumulator needs the mirror, which is private today.** `MirrorResource` is a
  private newtype in `ingest.rs` (`:52-53`). Accumulate in a normal `Update` system that writes
  into `CaptureState`, and let the `ScreenshotCaptured` observer only *read* what was accumulated —
  do not try to reach the mirror from the observer. Widen the resource's visibility if that is the
  smaller change; it is `gui`-internal either way.
- **`mutate.sh` rewrites source in place** — run it alone; it fails a `NO-COMPILE` sabotage.
- **Camera focus is hardcoded `[64,64,9]`** (recorded LOW deferral) — correct for the shipped seed
  and load-bearing for AC9's framing test. Do not generalise it here.
- **`simd` has no seed flag** — the seed is `SEED` (`crates/simd/src/main.rs:20`), port positional.
- When torn between simple and general, pick simple and leave a `// NOTE:` naming the limitation.

### Code skeletons (shape, not prescription)

```rust
// crates/gui/src/blend.rs  (NEW)

/// Client-local playback clock for AD-15 blending. Presentation only: it holds no sim state
/// and the wire carries no tick rate, so the cadence is MEASURED from delta arrivals.
#[derive(Resource, Debug)]
pub struct TickClock { elapsed: f32, interval: f32 }

/// Fast-forward is 5x of 10 tps, so 0.02 s is the floor a real cadence can reach; a gap
/// longer than the ceiling is a stalled server, not a slow tick, and must not stretch a blend.
pub const MIN_TICK_INTERVAL: f32 = 0.02;
pub const MAX_TICK_INTERVAL: f32 = 0.50;

impl TickClock {
    /// 0 at the delta that delivered the current tick, 1 at the next expected one.
    /// CLAMPED — the client shows only states the wire delivered (AD-15).
    pub fn factor(&self) -> f32 { (self.elapsed / self.interval).clamp(0.0, 1.0) }
}

/// Where a projected entity is drawn this frame. `previous` absent => it spawned this tick or a
/// snapshot just landed: snap.
pub fn blended_translation(previous: Option<[i32; 3]>, current: [i32; 3], factor: f32) -> Vec3 {
    match previous {
        Some(previous) => world_to_render(previous).lerp(world_to_render(current), factor),
        None => world_to_render(current),
    }
}
```

```rust
// crates/gui/src/appearance.rs  (UPDATE)

pub struct LightProperties {
    pub color: Color, pub intensity: f32, pub range: f32,
    /// Fraction of `intensity` the flicker may swing either way.
    pub flicker_amplitude: f32,
    pub flicker_hz: f32,
}

/// Client-side flicker (NFR5's carve-out): a pure function of the emitter's simulation id and
/// the client's elapsed seconds. No RNG, no wire data, no sim meaning.
pub fn flicker_scale(kind: LightKind, id: u32, seconds: f32) -> f32 { /* two incommensurate
    sines, per-id phase offset, amplitude from the table */ }
```

### Project Structure (files to touch)

```
crates/gui/src/blend.rs             NEW     TickClock + blended_translation + tests
crates/gui/src/appearance.rs        UPDATE  flicker columns, flicker_scale, literal-oracle tests
crates/gui/src/project.rs           UPDATE  stop re-snapping transforms; ProjectedLight; DigChip
crates/gui/src/ingest.rs            UPDATE  clock reset on tick advance, chained system tuple,
                                            shared registration fn, --expect-work parsing
crates/gui/src/capture.rs           UPDATE  motion accumulator + assertions
crates/gui/src/lib.rs               UPDATE  `pub mod blend;`
crates/gui/tests/headless.rs        UPDATE  blend, snapshot-snap, flicker, chips, framing tests
crates/gui/tests/capture.rs         UPDATE  dig-site-window comparison
docs/tech-art-guidelines.md         UPDATE  motion + flicker section
_bmad-output/implementation-artifacts/6-1-signoff/                          NEW  artifact + captures
_bmad-output/implementation-artifacts/mutations/6-1-the-world-moves.sh      NEW
_bmad-output/implementation-artifacts/deferred-work.md                      UPDATE if anything defers
```

### Previous story intelligence (deltas that change THIS story)

- **5.4's shipped look is the baseline this story must not disturb** — the light table, ground
  luminance floor/ceiling and warm-pixel floor were converged over 8 review-patch rounds against a
  measured artifact. Keep the existing capture range checks green; a flicker band wide enough to
  break the warm-pixel floor is too wide.
- **The AC-text-defect class stands at seven instances.** If an AC here proves unmeetable as
  written, raise it for Wolf's ruling and record the amendment in place — caught at dev is the good
  outcome.
- **Codex handoff:** check the model banner every run; restate RED evidence across any session
  boundary; commit per green task.

### Verification

**Executed at story-creation (the headless half — non-zero evidence, P6 rule).** Live `simd 7452`
on the shipped seed, designation issued by the real `tui` key sequence below: designations appeared
at tick 46 (8 tiles), dwarves reached `Work` by tick 70, all 8 tiles were `empty` by tick 98, and 8
stone items remained at `[58..64, 68..69, 9]` for the rest of the 70-second observation.
Over 701 ticks, 327 (47%) contained a dwarf position change with zero commands issued. Camera
projection of the site computed from the shipped `CameraRig` constants: `(0.49,0.70)`–`(0.53,0.73)`.

**Gate (headless, any devpod, must be green before done):**

```bash
scripts/gate.sh
```

**The live vehicle (recipe proven at 5.3 and 5.4; build in the devpod, launch on Windows):**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
# simd stays in WSL:  ./target/debug/simd 7451
# gui.exe runs on the Windows side against localhost:7451
```

**Designate the named dig site from a TUI client (WSL, any time after the daemon starts):**

```bash
./target/debug/tui 7451 --z 9 --frames 3 \
  --key d,h,h,h,h,h,h,j,j,j,j,enter,l,l,l,l,l,l,j,enter
# d = dig mode (cursor resets to 64,64) · 6x h and 4x j reach [58,68] · enter anchors
# 6x l and 1x j reach [64,69] · enter commits the rect [58,68,9]-[64,69,9]
```

**The motion capture (the obligation the dev agent inherits — it cannot run until the blend
exists):**

```bash
# 1. before the dig
gui.exe 7451 --capture 6-1-before.png --frames 600
# 2. designate (command above)
# 3. across the dig — size --frames so the run spans >=100 ticks; the instrument asserts it
gui.exe 7451 --capture 6-1-after.png --frames 2000 --expect-work
```

**Required non-zero observations** (paste both printed lines into the record): the motion line
reports ticks observed ≥ 100, dwarf position changes > 0, blend frames > 0, max working ≥ 1 and
items ≥ 1; the 5.4 range-check line still reports its warm-pixel and ground-luminance figures inside
their bands; the startup line still reads `projected 53365 terrain cubes`. **Exit 0 is not a
result.**

**Capture self-test on the vehicle (AC15):**

```bash
cargo test -p gui --test capture --no-run --target x86_64-pc-windows-gnu
# copy the test exe to the Windows side, then run it with
# FROSTVEIN_CAPTURE_FIRST=6-1-before.png FROSTVEIN_CAPTURE_SECOND=6-1-after.png <exe> --ignored
```

**NFR6 reading (AC13):** F3 overlay on, dig in progress, read sustained fps at working zoom and at
full vista; record both labelled `gingerspice / native Windows / NVIDIA`.

**Sabotage:**

```bash
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/6-1-the-world-moves.sh
```

### If this overruns one session

The seam is **motion | ambience**: Tasks 1, 2 and 5 (clock, blend, snapshot snap, instrument — the
headline outcome exists and is measurable) versus Tasks 3 and 4 (flicker, chips). Both halves are
gui-only, so the cut is clean — but the closing sign-off needs every bar, so a split defers the
gate, not part of it. Commit per green task; restate RED evidence in any continuation handoff.

### References

- Story 6.1 epic text — `_bmad-output/planning-artifacts/epics.md:804-845`; Epic 6 rules `:796-802`
- UX-DR1–22 and the anti-requirements — `epics.md:149-194`; FR34 `:75`; NFR5–NFR6 `:95-96`
- AD-14…AD-18, M2 conventions, NFR6 bar —
  `architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:92-198`, `:244-253`
- Mirror contract and the previous-tick generation — `crates/client-core/src/lib.rs:56-105`, `:139-145`
- The two clobber sites — `crates/gui/src/project.rs:286-297`; the Update tuple —
  `crates/gui/src/ingest.rs:107-117`; the light-table NOTE — `crates/gui/src/appearance.rs:50`
- Client-local animation precedent — `crates/gui/src/atmosphere.rs:291-301`
- Instrument to extend — `crates/gui/src/capture.rs:101-154`; framing helper —
  `crates/gui/src/camera.rs:76-95`
- Dig semantics, work duration, rubble, carried items —
  `crates/sim-core/src/lib.rs:35`, `:849-892`, `:1339-1341`, `:1462-1471`, `:674-687`
- Scripted TUI commands — `crates/tui/src/main.rs:97-192`, `crates/tui/src/view.rs:59-79`, `:387-420`
- 5.4's shipped look, review findings and vehicle record — `5-4-the-cold-boot.md`
- Open deferrals this story touches — `deferred-work.md:573-576`, `:605-609`, `:613-618`, `:626-634`
- Story rules, instrument rule, exit-0 — `docs/technical-preferences.md:64-101`

## Dev Agent Record

### Agent Model Used

`claude-opus-5[1m]` (orchestrator) — Task 0 only. Implementation is delegated to Codex (Völundr)
and **has not started**: the gate is closed.

### Debug Log References

Task 0 verification, re-run against shipped `sim-core` this session rather than inherited from
story-creation:

- `World::items()` (`crates/sim-core/src/lib.rs:1462-1471`) reports **every** item at its stored
  `Pos`, carried or not — this is the accessor that feeds the wire.
- A stone's `Pos` is rewritten in exactly one place, `release_claim` (`:696-707`), i.e. **at the
  drop**. Confirmed by reading the pick-up arm (`:825-836`), which sets `Carrying` and mutates no
  item position. So a carried stone holds its pickup tile on the wire and then teleports.
- `uncarried_stones` (`:674-687`) proves the sim knows which stones are in transit; that
  knowledge never reaches the wire.
- **New at Task 0 (not known at story-creation):** haul jobs are derived **only from stockpile
  tiles** (`:319`, `:260`). 6.1 places no stockpile, so **no dwarf ever carries a stone in this
  story's scenario** — the UX-DR14 gap does not merely go unrendered here, it does not occur.
  This materially reframes the Task 0 ruling and is recorded in the artifact.
- Shipped 5.4 vehicle binary confirmed present and reusable for the capture pair —
  `target/x86_64-pc-windows-gnu/release/gui.exe`, built 2026-08-16 08:30. The pair needs no
  rebuild and no new code.

### Completion Notes List

- **Task 0 written half COMPLETE.** `6-1-signoff/what-you-will-see.md` authored: part (b) the
  four additions with the look each is aiming for, and part (c) the "what you will NOT see" list
  as seven ruled lines. Line 4 (**dwarves remain rigid cubes — smooth motion draws the eye to
  what is moving**) was added beyond the six the story named, on the 5.4 lesson that an
  unstated absence surfaces at the live viewing.
- **Task 0 BLOCKING RULING CLOSED (Wolf, 2026-08-16): the carried stone — sign wow beat 2
  WITHOUT it.** Option A taken; no sim story spun; no wire change rides on 6.1. **UX-DR14's
  "picks something up and carries it" clause is formally NOT DELIVERED in M2** — recorded as a
  decision, not blurred into the beat. It is unobservable in this scenario in any case (no
  stockpile → no haul job → no carrying).
- **Task 0 capture pair TAKEN 2026-08-17** by Wolf on gingerspice off the shipped 5.4 binary:
  `6-1-signoff/6-1-before.png`, `6-1-after.png`. **The pair produced a finding, and it is the kind
  the gate exists to catch.** Wolf's first reaction was *"did not see the difference"*, and he was
  right. Measured, not argued:
  - The dig-site window from `CameraRig::project_world_point` is `u 0.492–0.541`, `v 0.689–0.747`
    = **64×43 px = 0.30% of a 1280×720 frame**.
  - Pixels differing between the pair (channel sum > 30): **2,255 = 0.245% of the frame**, of which
    **1,625 (72%) fall inside that window**; the remainder is snowfall and aurora.
  - **So the dig is correct and correctly located** — designation landed, 8 tiles emptied, stone
    items rendered, change concentrated exactly where the camera math predicted. It is simply too
    small to see at the boot vista.
  - `6-1-signoff/6-1-digsite-inset.png` was generated to make the pair legible: both full frames
    with the window marked, plus 7× crops of the window side by side. At 7× the pale shelf is
    visibly cut into a dark trench with a stone item at lower centre.
  - **Recorded as line 8 of the artifact's "what you will NOT see":** the dig face does not read at
    the boot vista, it reads at working zoom. **This has a direct consequence for AC7, AC8 and
    Task 4** — the rubble and the debris chips are sub-legible at boot framing, so wow beat 2's
    visible weight at the opening frame rests on **dwarf motion and light flicker**, which are
    camp-scale. Judge the dig at working zoom (where AC13's reading is taken anyway).
  - **It is also a live preview of why AC15 exists:** whole-frame inequality here is satisfied by
    snowfall alone; only the windowed comparison says anything true. AC15's design is vindicated
    before a line of it is written.
- **AC1 MET — WOLF APPROVED THE ARTIFACT 2026-08-17 and the gate is OPEN.** He approved it as-is
  after the finding was measured, choosing **not** to re-pick the dig site: the alternative would
  have invalidated the live-verified sky-exposure, occlusion, projection and 52-tick timing figures
  and required amending AC7 and AC9, and the dig is legible at working zoom regardless.
  Implementation is delegated to Codex from here; Task 0 is closed and is not to be re-opened or
  re-edited by the dev agent.

### Completion Notes — 2026-08-17 implementation

- Tasks 1–5 and 7 are implemented headlessly. The vehicle-only `--ignored` capture test was
  written and compiled but deliberately not executed; Task 6 and Task 9 remain unchecked.
- RED evidence before implementation: `cargo test -p gui --test headless
  blend_midpoint_is_strictly_between_the_delivered_positions --offline` failed with
  `error[E0583]: file not found for module 'blend'` at `crates/gui/src/lib.rs:5:1`.
- Sabotage was run alone. The first mutation produced: `assertion 'left == right' failed` in
  `blend::tests::midpoint_and_snap_are_literal_wire_positions` after interpolation was allowed
  to extrapolate. The remaining table entries were also killed by their named assertions.
- The requested `docs/dev-workflow.md` path is absent; its applicable workflow rules are present
  in `docs/technical-preferences.md`.

### File List

- `_bmad-output/implementation-artifacts/6-1-signoff/what-you-will-see.md` (new)
- `_bmad-output/implementation-artifacts/6-1-signoff/6-1-before.png` (new — vehicle capture)
- `_bmad-output/implementation-artifacts/6-1-signoff/6-1-after.png` (new — vehicle capture)
- `_bmad-output/implementation-artifacts/6-1-signoff/6-1-digsite-inset.png` (new — marked frames +
  7× dig-site crops, generated from the pair)
- `_bmad-output/implementation-artifacts/6-1-the-world-moves.md` (modified — Status, Task 0
  checkbox, Dev Agent Record, Change Log)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified — 6.1 → in-progress)
- `crates/gui/src/blend.rs` (new — delivered-tick interpolation clock and transform blend)
- `crates/gui/src/appearance.rs`, `capture.rs`, `ingest.rs`, `lib.rs`, `project.rs` (modified —
  flicker, capture evidence, shared projection wiring, client-local debris)
- `crates/gui/tests/headless.rs`, `tests/capture.rs` (modified — motion/debris/framing coverage and
  projected-window capture comparison)
- `docs/tech-art-guidelines.md` (modified — presentation-motion decisions)
- `_bmad-output/implementation-artifacts/mutations/6-1-the-world-moves.sh` (new)

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-17 | **Task 0 APPROVED BY WOLF — AC1 MET, the gate is OPEN, implementation may start.** Approved as-is; the named dig site is deliberately NOT re-picked (re-picking would invalidate the live-verified exposure/occlusion/projection/timing figures and force amendments to AC7 and AC9). Dev delegated to Codex for Tasks 1-5, 7, 8; Tasks 6 and 9 stay vehicle- and human-bound. |
| 2026-08-17 | Task 0 capture pair taken on the vehicle. Measured: the dig site is 64×43 px = 0.30% of frame at boot framing, and 72% of the 2,255 changed pixels fall inside the predicted window — the dig is correct and correctly located but invisible to the eye at the vista. `6-1-digsite-inset.png` added (marked frames + 7× crops) to make the pair legible, and line 8 added to "what you will NOT see": the dig face reads at working zoom, not at the boot vista. Consequence flagged for AC7/AC8/Task 4. AC15's windowed-comparison design vindicated in advance. Gate still closed pending Wolf's approval. |
| 2026-08-16 | Task 0 part (c) RULED by Wolf: wow beat 2 is signed **without** UX-DR14's carried stone (option A) — no sim story spun, no wire change, clause formally not delivered in M2. Wolf is taking the before/after capture pair on gingerspice rather than the written-only fallback; his approval follows that viewing, so AC1 remains unmet and the gate stays closed. |
| 2026-08-16 | Task 0 written half delivered: `6-1-signoff/what-you-will-see.md` (four additions + seven-line "what you will NOT see"). Carried-stone raise re-verified against shipped `sim-core` and sharpened — haul jobs come only from stockpiles and 6.1 places none, so no dwarf carries a stone in this scenario at all. Capture pair and Wolf's ruling still owed; gate closed, no implementation started. |
| 2026-08-16 | Story created. Dig site named and verified live on the shipped seed (8 mineral tiles, all sky-exposed and unoccluded, dug in 52 ticks, 8 stone items left standing); wander motion baseline measured at 47% of ticks; TUI key sequence executed end to end; the carried-stone gap in UX-DR14 verified against `sim-core` and raised as a blocking Task 0 clarification. |
