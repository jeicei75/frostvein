---
baseline_commit: 1f262d8
model: claude-opus-5[1m]  # the policy default (Opus); recorded because 5.4 ran on claude-fable-5 and an unlabelled neighbour row is exactly the ledger ambiguity the model policy exists to prevent
---

# Story 6.1: The World Moves

Status: in-progress

<!-- The HEADLESS half only. Tasks 6 and 9 and ACs 13/16/17 stay OPEN and are vehicle- and
     human-bound; review does not close this story, only Wolf does (5.4's precedent). -->


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

7. **The named dig site is `[55,62,9]`–`[56,65,9]`** — 8 mineral tiles (ice/snow), every one
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

- [x] **Task 1 — The blend clock and the blend** (AC: 2, 3, 5, 6) — reopened by orchestrator
      verification 2026-08-17 (two subtasks ticked but not delivered), **CLOSED for real the same
      day** after the continuation run added the app-level tests and the sabotage was re-run and
      now kills. Evidence in "Orchestrator verification" below.
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
  - [x] Extract the Update registration into one function
        (e.g. `pub fn projection_systems(app)`) called by `run()` **and** by the headless tests, so
        a dropped system is a red test. *(**Inert on the first run** — the function existed at
        `ingest.rs:131` but only `run()` called it, while `headless_app()` built its own wiring. The
        continuation wired it in at `tests/headless.rs:38`, so live and test now register through
        the same object. **AC6 MET, proven by sabotage: the same deletion that left 54/54 GREEN now
        turns 4 tests RED.**)*
  - [x] Tests: strictly-between at factor 0.5 (assert against a hand-written
        expected midpoint, not against the production lerp); factor 1 lands exactly on the current
        position; elapsed far past the interval still lands exactly on the current position and
        never beyond; an entity with no previous state snaps; a re-run of reconcile after the blend
        does not move the entity back to its snapped position (AC5).
        *(The pure-function half was already good — literal-oracle tests on `blended_translation`
        and `TickClock` (`blend.rs:66,75`). **The app-level halves were missing on the first run and
        the continuation added them:** `projection_pipeline_blends_at_a_midpoint`
        (`headless.rs:144`) and `later_production_reconciliation_does_not_clobber_a_blended_
        translation` (`:166`) — the latter is AC5's sole-writer assertion. The old `headless.rs:24`
        duplicate of the `blend.rs` unit test is gone, replaced by the real app-level test.
        **AC5 MET**, mutation `reconciliation overwrites blended translation` KILLED.)*

- [x] **Task 2 — A rewind snaps** (AC: 4, 6) — reopened 2026-08-17 (the required test did not
      exist), **CLOSED for real** by the continuation.
  - [x] Verify (do not re-implement) that `Mirror::apply_snapshot` clears `previous_entities`
        (`crates/client-core/src/lib.rs:50-54`, `:147-168`) and that the blend therefore cannot
        cross a snapshot. Reset the blend clock on a snapshot as well, so the first post-snapshot
        frame has no stale elapsed time. *(`TickClock::reset` exists at `blend.rs:41`.)*
  - [x] Test: run frames with a blend in progress, apply a snapshot placing the
        same entity far away, run one frame at a mid-range clock, assert the transform equals the
        snapshot position exactly. *(Missing on the first run; delivered by the continuation as
        `snapshot_rewind_snaps_at_a_mid_blend_clock` (`headless.rs:193`). **AC4 MET**, mutation
        `snapshot rewind no longer snaps` KILLED.)*

- [x] **Task 3 — Flicker** (AC: 10, 11, 6) — reopened 2026-08-17 (AC11's app-level test did not
      exist), **CLOSED for real** by the continuation.
  - [x] Extend `LightProperties` with the flicker columns the 5.4 `// NOTE:` at
        `crates/gui/src/appearance.rs:50` promises, and delete that NOTE. Add `flicker_scale(kind,
        id, seconds)` — pure, no RNG, no wire input (skeleton in Dev Notes).
  - [x] Add a `ProjectedLight(LightKind)` component written by `reconcile` on spawn and only when
        the kind changes, so reconcile stops re-inserting `point_light()` every frame
        (`crates/gui/src/project.rs:293-297`); add `flicker_lights` writing
        `PointLight.intensity` each frame from the table × `flicker_scale`.
  - [x] Tests: the scale stays inside the named band across a time sweep; two
        ids of the same kind diverge at the same instant; the two kinds diverge; the function is
        deterministic for the same `(id, seconds)`; and an app-level test that runs reconcile after
        the flicker and finds the flickered intensity intact (AC11).
        *(Pure-function half was already delivered and mutation-killed (`appearance.rs:102`). The
        app-level AC11 test was missing on the first run and the continuation added it as
        `flickered_light_survives_a_later_production_reconciliation` (`headless.rs:225`).
        **AC11 MET**, mutation `reconciliation resets flickered light` KILLED.)*
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

### Review Findings — code review 2026-08-18 (4 layers, all live)

Four layers, none a coverage hole: every layer verified `cargo 1.97.1` and executed code rather
than reading it. Territory note: R1's split names `sim-core` / `simd`+`tui`+`protocol`, none of
which this gui-only diff touches, so it was adapted for this story — Blind Hunter took the pure
logic (`blend.rs`, `appearance.rs`), Edge Case Hunter the shell (`project.rs`, `ingest.rs`,
`capture.rs`, `tests/`). Both Opus auditors kept whole-diff scope. **R1 has no mapping for the M2
crates and needs one at the Epic 5/6 retro.**

Convergences (the signal this project tracks): the unordered `accumulate_motion` was raised
independently by **three** layers (edge + auditor + feature); the self-referential flicker band by
two (auditor + blind); the capture-recipe/print-order defect by two (auditor + feature).

- [x] [Review][Decision] **The story's own capture recipe cannot pass its own instrument** — `simd`
      ticks at 10 Hz (`crates/simd/src/main.rs:20`) and 5.4 measured >135 fps on gingerspice, so
      `--frames 600` ≈ 4.4 s ≈ **~44 delivered ticks** against `assert!(ticks.len() >= 100)`
      (`crates/gui/src/capture.rs:67`). The **before** run panics *before* `Screenshot::primary_window()`
      is spawned — the operator gets a panic and **no PNG at all**, on the first command of Task 6.
      The `--frames 2000` after-run is fine (~148 ticks). The recipe appears twice, and one copy is
      inside the artifact **Wolf approved**: this story's Verification block and
      `6-1-signoff/what-you-will-see.md`. Needs ≥1,400 frames at 135 fps. Wolf's call because it
      amends an approved artifact.

- [x] [Review][Patch] **HIGH — the three "drive" lines are untested; each is a one-line deletion that
      kills wow beat 2 with a green suite** [`crates/gui/src/ingest.rs:355`, `:379`, `:422`] —
      independently re-verified by the orchestrator on a throwaway copy under `/tmp`: deleting
      `clock.observe_tick(...)` (clock never re-bases, factor pins at 1.0, **every dwarf snaps**),
      or `time.delta_secs()` → `0.0` (factor stays 0, client renders permanently one tick behind),
      or `time.elapsed_secs()` → `0.0` (**nothing breathes**) each leaves **57/57 GREEN**. This is the
      same defect class as the first Codex run's inert `projection_systems`, one level below where
      AC6 looks: AC6 proves a system is *in the tuple* and says nothing about the tuple's *inputs*.
      The four new app-level tests all call `TickClock::advance(0.01)` by hand
      (`tests/headless.rs:154`, `:176`, `:203`), so they pass whether or not production ever drives
      the clock. Fix: app-level tests that let the production systems drive time across
      `app.update()` and assert a transform actually moves and an intensity actually changes between
      two frames; plus the three matching entries in the mutation table.
- [x] [Review][Patch] **The motion line is printed *after* the assertion, so a failing run prints no
      motion line** [`crates/gui/src/capture.rs:204-212`] — AC14 requires it "before any conclusion".
      On the vehicle run that fails, the operator gets one panic and none of the five numbers needed
      to diagnose it. One-line reorder.
- [x] [Review][Patch] **The "mid-blend frames" counter never reads a rendered position**
      [`crates/gui/src/capture.rs:180-185`] — it is `factor ∈ (0,1) && any entity has a previous
      state`, and `Mirror::apply_delta` stores a previous entry for *every surviving entity*, moving
      or not, so it degenerates to "the clock is mid-interval" and is true in a frozen world. It
      observes no `Transform`. Measured: under the sabotaged clock above it still logged 23
      strictly-between frames, so **AC14's blend assertion passes with the blend dead** — inert
      against this story's most likely live failure.
- [x] [Review][Patch] **`accumulate_motion` is ordered against nothing** [`crates/gui/src/ingest.rs:123`]
      — registered as its own `Update` chain with no `.after()`/`.before()` edge to
      `projection_systems`. It takes `Res<TickClock>` while `blend_projection` takes `ResMut<TickClock>`;
      Bevy's `ambiguity_detection` defaults to `LogLevel::Ignore` and this repo never overrides it,
      so the conflict is resolved silently. The story calls that ordering load-bearing; the
      instrument reading it sits outside the chain. **Raised by three layers.**
- [x] [Review][Patch] **The flicker band assertion is a tautology and cannot fail**
      [`crates/gui/src/appearance.rs:83-90`] — `flicker_scale` is
      `1.0 + amplitude * (primary + secondary) / 1.3` with the bracket normalised to ±1.3, so
      containment within `1.0 ± amplitude` holds **by construction for any amplitude**, and the test
      reads `flicker_amplitude` from the same table it is validating. The `torch flicker band widens`
      mutation (0.07 → 0.70) is killed only by the separate constant-pinning
      `assert_eq!(light_properties(Torch).flicker_amplitude, 0.07)`. AC10's "named band" has no test
      that can go red. This is the self-referential-test antipattern already hit at 1.1, 1.2 and 1.3.
      Fix: assert against hardcoded literals.
- [x] [Review][Patch] **Two distinct-tick deltas in one frame collapse the measured interval to the
      0.02 s floor** [`crates/gui/src/blend.rs:34-36`] — `observe_tick` sets
      `interval = elapsed.clamp(MIN, MAX)` and the first delta of the pair already zeroed `elapsed`,
      so the second measures `0.0` → `MIN_TICK_INTERVAL`, and `factor()` saturates to 1.0 for the
      rest of that interval. The Dev Notes accepted "the entity covers two tiles in one interval";
      they did not accept a clock that stays corrupted into later frames. No test exercises a
      distinct-tick burst — the existing queued-delta test reuses `tick: 1`, which the
      `tick > last_tick` guard makes a no-op.
- [x] [Review][Patch] **AC15's windowed capture comparison is a near-vacuous `> 0`**
      [`crates/gui/tests/capture.rs:88-91`] — measured against the real approved pair with the test's
      own window math: 1,651 changed pixels inside the window vs an expected ~5 from atmosphere alone
      at the outside density, so `> 0` passes on snowfall with ≈99.5% probability. The AC exists
      *because* whole-frame inequality is vacuous; a `> 0` threshold in an 8,165 px window barely
      improves on it. A threshold in the low hundreds would bite.
- [x] [Review][Patch] **`item_count` overwrites instead of taking a running maximum**
      [`crates/gui/src/capture.rs:56-62`] — `max_working` uses `.max(...)`, `item_count` uses plain
      assignment, so items present during the run but gone at the final observed frame fail
      `--expect-work`'s `item_count >= 1`. A false negative on a run the instrument should accept,
      inconsistent with how the sibling counter answers the same "did it happen at any point"
      question.

- [x] [Review][Defer] NaN passes through `f32::clamp`, so the "clamped to [0,1]" guarantee has a hole [`crates/gui/src/blend.rs:47`, `:55`] — deferred, unreachable (Bevy never emits a NaN `delta_secs()`) and guarding it is error handling for an impossible scenario, which ground rule 1 makes a defect in itself
- [x] [Review][Defer] `flicker_scale` phase collides exactly for ids ≥ 2^24, so two emitters pulse in perfect sync [`crates/gui/src/appearance.rs:84`] — deferred, needs ~16.7M `IdAllocator` allocations
- [x] [Review][Defer] Flicker aliases from ~3.2 days of client uptime and freezes entirely at ~11.6 days (f32 `elapsed_secs` precision) [`crates/gui/src/appearance.rs:85-88`] — deferred, out of reach for a demo client
- [x] [Review][Defer] `observe_tick(0)` is a silent no-op (`>` not `>=`) [`crates/gui/src/blend.rs:33`] — deferred, not reachable given reset-before-delta ordering
- [x] [Review][Defer] `TickClock::reset` leaves a stale `interval` across a snapshot boundary [`crates/gui/src/blend.rs:41-44`] — deferred, snaps early rather than smearing, never extrapolates
- [x] [Review][Defer] `DigChip` entities accumulate without bound across many distinct dug tiles with no rebuild [`crates/gui/src/project.rs:294-318`] — deferred, pre-existing YAGNI boundary
- [x] [Review][Defer] The dig-chip test asserts count and markers only — no position, no determinism, and Task 4's two negative cases untested; `chip_offsets()` is a fixed array, not "derived from the tile position" as Task 4's text says [`crates/gui/tests/headless.rs:684-722`, `crates/gui/src/project.rs:382-389`] — deferred
- [x] [Review][Defer] AC3's mutation targets the redundant clamp in `blended_translation`, not the load-bearing one in `TickClock::factor` [`mutations/6-1-the-world-moves.sh:3-9`] — deferred
- [x] [Review][Defer] Making `--expect-work` a no-op leaves 57/57 green — the flag's effect is untested [`crates/gui/src/ingest.rs:189`, `:201`] — deferred
- [x] [Review][Defer] `MIN_TICK_INTERVAL` is never exercised by any test [`crates/gui/src/blend.rs:35`] — deferred
- [x] [Review][Defer] Reconcile no longer refreshes an existing entity's scale, so an entity spawned before `ProjectionAssets` exists keeps scale 1.0 permanently and a wire kind change no longer restyles it [`crates/gui/src/project.rs:335-350`] — deferred, not live under `run()`
- [x] [Review][Defer] `MotionStats` counter fields and both interval constants are `pub` with no reader outside their module [`crates/gui/src/capture.rs:31-34`, `crates/gui/src/blend.rs:7-8`] — deferred
- [x] [Review][Defer] Deleting `clock.reset` on snapshot leaves 57/57 green [`crates/gui/src/ingest.rs:344`] — deferred, harmless today only because `apply_snapshot` clears `previous_entities`
- [x] [Review][Defer] Three `// NOTE:`s the story's tasks asked for are absent — the per-frame id map, the two-deltas-in-one-frame limitation, and point-light-not-emitter-material — deferred
- [x] [Review][Defer] AC18's "RED output pasted into the Dev Agent Record" is half-satisfied: the KILLED table and suite counts are pasted, no RED assertion text for the four new mutations — deferred, format not fabrication
- [x] [Review][Defer] `deferred-work.md` was not updated by this story, so entries it closed still read as open — 5.3's "`previous_entity()` remains without a live caller" and 5.3's review's incidental-`.chain()` finding are both closed here, and 5.4's "`run()` has no test of any kind" is partly closed by AC6 and needs narrowing rather than deletion — deferred *(raised by the orchestrator, not by a layer)*

**Carried into Task 6, not a defect:** the flicker band actually reached is torch ±7.0% and campfire
±11.0% (14% / 22% peak-to-peak) at 1.7 Hz / 0.9 Hz — comfortably inside 5.4's warm-pixel floor
(17,648 measured against a 3,000 floor), but a *subtle* breath. Given Wolf's "did not see the
difference" reaction to the Task 0 pair, if he judges it too faint at the live viewing the amplitude
column (`crates/gui/src/appearance.rs:61`, `:68`) is the single-number knob and widening it will not
endanger the capture range checks.


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
- **AMENDED 2026-08-18 (Wolf's ruling at the live viewing). The named dig site is now
  `[55,62,9]`–`[56,65,9]`: a 2x4 rect of 8 tiles, ALL of them solid.** The original
  `[58,68,9]`–`[64,69,9]` is superseded — it straddled a slope, and slope tiles are `Tile::Ramp`,
  which is not diggable, so four of them stood as a contiguous wall through the middle of the
  finished excavation. The site was re-verified live end to end before any vehicle run: 8
  designations, first dwarf in `Work` at ~24 ticks, **all 8 tiles dug in 52 ticks — the same figure
  as the original** — 8 stone items left standing, max 2 dwarves working at once, and **nothing left
  standing inside the site**. Projection `(0.424,0.692)`–`(0.455,0.721)`, the same v-band as the
  original, so it reads at the same place in the composition; 9.2 tiles from the campfire (range 28)
  and 7 from the nearest torch (range 20). It is the ONLY rect near the camp that is all-solid,
  sky-exposed, unoccluded from the boot camera AND in frame — 19 tiles in the whole neighbourhood
  meet all four constraints, so the choice was forced, not preferred. **AC7's "8 mineral tiles"
  needs no amendment.** Superseded original, kept for the record:
- **The superseded site `[58,68,9]`–`[64,69,9]` designated exactly 8 tiles** (ice + snow, all
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
  --key d,h,h,h,h,h,h,h,h,h,k,k,enter,l,j,j,j,enter
# d = dig mode (cursor resets to 64,64) · 9x h and 2x k reach [55,62] · enter anchors
# 1x l and 3x j reach [56,65] · enter commits the rect [55,62,9]-[56,65,9]
# EXECUTED LIVE 2026-08-18: 8 designations, all 8 tiles dug in 52 ticks, 8 items, nothing standing.
```

**The motion capture (the obligation the dev agent inherits — it cannot run until the blend
exists):**

```bash
# 1. before the dig. --frames 1500, NOT 600: simd ticks at 10 Hz and the vehicle runs >135 fps,
#    so 600 frames is ~4.4 s ~= 44 ticks and the instrument's >=100-tick floor panics BEFORE the
#    screenshot is spawned — a failed command and no PNG at all. 1500 frames ~= 11 s ~= 110 ticks.
gui.exe 7451 --capture 6-1-motion-before.png --frames 1500
# 2. designate (command above)
# 3. across the dig — size --frames so the run spans >=100 ticks; the instrument asserts it
gui.exe 7451 --capture 6-1-motion-after.png --frames 2000 --expect-work
```

**Do not reuse the Task 0 filenames.** `6-1-before.png` and `6-1-after.png` are the pair Wolf
approved and are the baseline this run is compared against; writing over them destroys the
comparison. Hence the `6-1-motion-*` names above.

**Required non-zero observations** (paste both printed lines into the record): the motion line
reports ticks observed ≥ 100, dwarf position changes > 0, blend frames > 0, max working ≥ 1 and
items ≥ 1; the 5.4 range-check line still reports its warm-pixel and ground-luminance figures inside
their bands; the startup line still reads `projected 53365 terrain cubes`. **Exit 0 is not a
result.**

**Capture self-test on the vehicle (AC15):**

```bash
cargo test -p gui --test capture --no-run --target x86_64-pc-windows-gnu
# copy the test exe to the Windows side, then run it with
# FROSTVEIN_CAPTURE_FIRST=6-1-motion-before.png FROSTVEIN_CAPTURE_SECOND=6-1-motion-after.png \
#   <exe> --ignored
# The window comparison now has a floor of 200 changed pixels, not >0: the approved pair measured
# 1,651 inside the window against ~5 expected from snowfall and aurora alone.
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

### Orchestrator verification of the Codex dev run (2026-08-17)

Codex (`gpt-5.6-terra`, effort high, rollout `01a00f06-ac14-7d21-8992-ba08c966669f`) exited 0 and
reported Tasks 1–5, 7, 8 complete. **Exit 0 was not trusted.** What holds and what does not:

**Verified GOOD, independently:**

- **No auth failure.** All 14 log matches for `401|Missing bearer|Unauthorized` are the handoff
  prompt's own text and git blob hashes. No real 401.
- **AC19 scope holds exactly.** `git diff --name-only main..HEAD` touches only `crates/gui/`,
  `docs/` and `_bmad-output/` — no wire change, nothing in `sim-core`, `simd`, `protocol`,
  `client-core` or `tui`. The file list matches the story's Project Structure section precisely.
- **`scripts/gate.sh` GREEN** on my own run (fmt, clippy `-D warnings`, tests, all three
  dependency-edge probes, metrics ledger tests).
- **`scripts/mutate.sh` 4/4 KILLED**, run alone, each on a genuine assertion failure, tree clean
  afterwards.
- **Full `gui` suite 54/54 green** after a forced rebuild (see the stale-artifact note below).
- Four commits on the branch, **all authored `Völundr <jeicei75@gmail.com>`**, nothing pushed.
- The pure-function work is genuinely well done: `blended_translation` and `TickClock` are tested
  against **hand-written literal** expectations rather than the production lerp, and `flicker_scale`
  likewise. That is the literal-oracle discipline this project asks for, and the mutations confirm
  the tests bite.

**FALSIFIED — four subtasks were ticked without being delivered, and they are the story's own
declared "whole implementation risk".** The story says of AC5 and AC11: *"AC5 and AC11 exist to make
that failure a red test rather than a live discovery."* They do not.

- **AC6 UNMET — proven by sabotage, not by reading.** I removed **both** `blend_projection` and
  `flicker_projection` from the live `Update` tuple in `projection_systems` — i.e. a client that
  does no blending and no flickering at all, the entire headline outcome of the story gone — and ran
  the full suite:

  ```
  SABOTAGE APPLIED: both blend_projection and flicker_projection removed from the LIVE tuple
  test result: ok. 39 passed; 0 failed  (lib)
  test result: ok. 14 passed; 0 failed  (headless)
  test result: ok.  1 passed; 1 ignored (capture)
  ```

  **54/54 green with the feature deleted.** `projection_systems` is called only by `run()`
  (`ingest.rs:118`); `headless_app()` builds its own wiring and never calls it, so the shared-
  registration mechanism AC6 requires is present but inert — the exact defect class AC6 was written
  to prevent. Its doc comment asserting "for both the live app and headless tests" makes it read as
  satisfied, which is worse than an obvious gap.
- **AC5 UNTESTED.** No test re-runs reconcile after the blend, so nothing asserts the blend is the
  sole writer of translation. `blend_projection` appears in no test file.
- **AC11 UNTESTED.** No test runs `flicker_projection`, so nothing asserts the flickered
  `PointLight` survives reconciliation.
- **AC4 UNTESTED.** No app-level rewind-snaps test exists; `TickClock::reset` is implemented but
  never exercised through an app.
- **`tests/headless.rs:24` is a duplicate of the `blend.rs` unit test** — it calls the pure function
  and constructs no app, so it looks like the app-level assertion and is not one.
- Consistent with the above, **the mutation table has no mutation for AC4, AC5, AC6 or AC11** —
  there was no test to mutate. 4 mutations for 18 ACs is thin on its face; the gap is precisely the
  seam ACs.

The **production code for these ACs looks correct** (reconcile no longer re-inserts `Transform` —
`project.rs:338` carries the `// NOTE:`; `ProjectedLight` gates light re-insertion). What is missing
is the evidence that it is correct and stays correct. That distinction matters for scoping the fix:
this is four tests plus one wiring change, not a redesign.

**Two process findings, recorded rather than buried:**

- **The self-gate is a COVERAGE HOLE, not a clean result.** Codex ran `codex review --base main`
  **once**; it "did not return a review conclusion — the spawned review session stopped after
  initialization", and no second pass was run (cap is three). Per the review-cost-discipline rule a
  layer that returns nothing is recorded as a coverage hole with zero findings, never as a pass.
  It found nothing because it did not run, and it is worth noting that a working self-gate is
  exactly what should have caught the inert `projection_systems`.
- **Commit cadence below the hard floor:** 3 commits for 7 completed tasks
  (`d7a3d94`, `5f2027c`, `e790b57`), against a floor of one per task. Better than a single squash,
  still short of what was asked in the prompt.

**One trap re-fired and cost me a false alarm, worth not rediscovering:** after `mutate.sh`
completed and restored the source, `cargo test -p gui` failed
`motion_instrument_rejects_stillness_and_accepts_the_required_observation` with *"capture observed
only 100 delivered ticks"* — the signature of the mutated `>= 101` still compiled in, while
`git diff` showed the source clean at `>= 100`. `cargo clean -p gui` then gave 39/39. **`mutate.sh`
restores source but leaves a stale build artifact; clean the crate AFTER the mutation run, not only
before.** This is the same trap recorded at 3.1's review and it is still live.

**One defect in my own handoff prompt:** it told Codex to read `docs/dev-workflow.md`, which does
not exist in this repo (it is a forge-root path). Codex correctly reported it rather than inventing
content. The applicable rules are in `docs/technical-preferences.md`.

### Continuation run and its verification (2026-08-17, same day)

A continuation handoff was issued for the four falsified ACs only, carrying the RED evidence
verbatim and requiring the sabotage itself as a pasted deliverable. Codex
(`gpt-5.6-terra`/high, rollout `01a00f1e-650f-7510-8d02-289c4f9fee78`) closed all four in **five
commits, one per AC plus the mutations** — the cadence floor met this time.

**AC6 MET, and the proof is the same sabotage that falsified it.** Removing both
`blend_projection` and `flicker_projection` from the live tuple:

```
BEFORE the continuation:  54 passed; 0 failed          <- feature deleted, suite green
AFTER  the continuation:  13 passed; 4 failed          <- feature deleted, suite RED
  projection_pipeline_blends_at_a_midpoint                          headless.rs:159
  later_production_reconciliation_does_not_clobber_a_blended_...    headless.rs:180
  snapshot_rewind_snaps_at_a_mid_blend_clock                        headless.rs:207
  flickered_light_survives_a_later_production_reconciliation        headless.rs:243
```

`headless_app()` now registers through `projection_systems` (`tests/headless.rs:38`), so the live
wiring and the test wiring are one object. The old duplicate-unit-test-masquerading-as-integration
at `headless.rs:24` is gone.

**Mutation table doubled, 4 → 8, and all 8 KILLED** (run alone, tree clean after,
`cargo clean -p gui` run afterwards per the stale-artifact trap):

```
blend extrapolates beyond delivered state              KILLED
torch flicker band widens                              KILLED
dig chips lose client-local ownership                  KILLED
motion capture requires too many ticks                 KILLED
snapshot rewind no longer snaps                        KILLED   <- new, AC4
reconciliation overwrites blended translation          KILLED   <- new, AC5
live projection omits the blend                        KILLED   <- new, AC6
reconciliation resets flickered light                  KILLED   <- new, AC11
```

**`scripts/gate.sh` GREEN** on my own run; **306 workspace tests**, 57 in `gui`.

**Codex's honesty on this run is worth recording as the behaviour to keep.** Its sandbox detached
output before the gate and the mutation run emitted their conclusions, and rather than claim them it
wrote: *"I cannot honestly claim a green gate or completed self-review… I also did not update the
Dev Agent Record with a fabricated mutation RED table."* It also did not run the self-gate rather
than half-run it. That is the correct call — and it is the direct inverse of the first run's four
ticked-but-undelivered boxes, from the same agent on the same story. **The lesson is not "Codex is
unreliable"; it is that a checkbox is only worth what its verification is worth, which is why the
orchestrator re-runs everything.** The self-gate remains a coverage hole for this story: it produced
no conclusion on either run.


### Code review and patch round (2026-08-18, orchestrator = Claude Opus, fresh context)

Four layers, none a coverage hole — every layer verified `cargo 1.97.1` and executed code. The
Feature Auditor ran the real pipeline against a live daemon with a real TUI designation and
**watched the feature work**: 592 distinct sub-tile positions per dwarf, 2,095 of 5,594 frames
strictly between delivered positions, 32 `ClientLocal` chips for 8 dug tiles with **zero**
mis-marked as `WorldProjected`, 8 wire stone items persisting, and 5,582 distinct intensities per
emitter across the table band. AC6 is genuinely closed and was verified structurally.

**The review's headline finding, and it is the same defect class as the first Codex run's inert
`projection_systems`, one level below where AC6 looks.** AC6 proves a system is *in the tuple*; it
says nothing about the tuple's *inputs*. Three one-line deletions each killed wow beat 2 with the
suite fully green. Re-verified independently by the orchestrator on a `git archive` copy under
`/tmp` (the repo was never mutated):

```
delete  clock.observe_tick(mirror.0.tick())   ingest.rs:355 -> 57/57 GREEN (every dwarf snaps)
        time.delta_secs()          -> 0.0     ingest.rs:379 -> 57/57 GREEN (one tick behind, forever)
        time.elapsed_secs()        -> 0.0     ingest.rs:422 -> 57/57 GREEN (nothing breathes)
```

The four seam tests added by the continuation run all call `TickClock::advance(0.01)` **by hand**,
so they pass whether or not production ever drives the clock. Closed by three tests that let
production drive time — `ingesting_a_delta_rebases_the_blend_clock_from_the_wire` (`ingest.rs`,
through the real `IngestReceiver`), `production_drives_the_blend_clock_from_frame_time` and
`production_drives_the_flicker_from_elapsed_time` (`headless.rs`, real `Time` across `app.update()`)
— plus three matching mutations. All three deletions now turn a named test RED.

**Nine changes applied (1 decision + 8 patches), all verified in one pass:**

1. The three untested drive lines above — three new tests, three new mutations.
2. `capture.rs` — the motion line now prints **before** `assert_valid`, per AC14's "before any
   conclusion". A failing vehicle run previously produced a panic and none of the five numbers.
3. `capture.rs` — the mid-blend counter now reads **actual rendered `Transform`s**: a frame counts
   only if an entity that genuinely moved between the two delivered ticks is drawn away from both
   endpoints. It previously read only the clock, and `apply_delta` stores a previous entry for every
   surviving entity, so it was true in a frozen world — measured to still log 23 "strictly-between"
   frames with the blend sabotaged. AC14's blend assertion was inert against this story's most
   likely live failure.
4. `ingest.rs` — new `ProjectionSet`; the capture chain is `.after(ProjectionSet)`. It previously
   read `Res<TickClock>` with no ordering edge to the `ResMut` writer, and Bevy's
   `ambiguity_detection` defaults to `LogLevel::Ignore`, so the conflict resolved silently. **Raised
   independently by three layers.**
5. `appearance.rs` — the flicker band is asserted against **hand-written literals**. It read
   `flicker_amplitude` from the same table it validated, and since `flicker_scale` is
   `1.0 + amplitude * (..)/1.3` with the bracket normalised to ±1.3, containment held by
   construction for *any* amplitude: the assertion could not fail. A peak-reached assertion was
   added too, or a zero-amplitude flicker would satisfy the band. This is the self-referential-test
   antipattern already hit at 1.1, 1.2 and 1.3.
6. `blend.rs` — a same-frame burst of distinct-tick deltas no longer collapses `interval` to its
   0.02 s floor. The first delta zeroes `elapsed`, so the second measured ~0 and saturated the
   blend for the rest of the frame. The Dev Notes accepted "two tiles in one interval"; they did
   not accept a clock that stays corrupted afterwards.
7. `capture.rs` — `item_count` is a running maximum, matching `max_working`. Items hauled away
   before the final frame previously failed `--expect-work` on a run that should pass.
8. `tests/capture.rs` — AC15's window comparison now has a floor of **200 changed pixels**, not
   `> 0`. Measured on the approved pair: 1,651 changed inside the window against ~5 expected from
   snowfall and aurora alone, so `> 0` passed on atmosphere with ~99.5% probability.
9. **Wolf's ruling on the capture recipe.** `simd` ticks at 10 Hz and the vehicle runs >135 fps, so
   the story's own `--frames 600` before-run is ~44 ticks against the instrument's ≥100 floor — it
   would have **panicked before writing any PNG, on the first command of Task 6**. Raised to 1,500.
   The Task 0 recipe was deliberately **left untouched**: that pair was taken with the shipped 5.4
   binary, which has no motion assertions, and rewriting it would falsify a record of what was run.
   A second trap found in the same block and fixed: the Task 6 capture used the *same filenames* as
   the approved Task 0 pair and would have overwritten the baseline it is compared against — the
   outputs are now `6-1-motion-before.png` / `6-1-motion-after.png`.

**Verification (one pass, after all nine):** `scripts/gate.sh` **GREEN** on a cold rebuild
(`cargo clean -p gui` run after the mutation round, per the stale-artifact trap). **311 workspace
tests**, `gui` 42 lib + 19 headless + 1 capture. **Mutation table 8 → 14, ALL 14 KILLED**, run
alone, each on a genuine named assertion, tree verified restored afterwards.

**Sixteen LOW findings went straight to `deferred-work.md`** with file:line per the cap-the-low-tail
rule — including three real-but-unreachable ones from the Blind Hunter (NaN through `f32::clamp`;
`flicker_scale` colliding exactly for ids ≥ 2^24; the flicker aliasing from ~3.2 days of uptime and
freezing at ~11.6), and one raised by the orchestrator rather than a layer: **this story closed two
`deferred-work.md` entries without updating the register**, so 5.3's "`previous_entity()` has no live
caller" and its review's incidental-`.chain()` finding still read as open.

**Process findings for the Epic 5/6 retro.** (1) **R1's layer-territory split has no mapping for the
M2 crates** — it names `sim-core` / `simd`+`tui`+`protocol`, none of which a gui-only diff touches,
so a literal reading would have left both hunters idle and the diff to the auditors alone. It was
adapted for this story (Blind → pure logic, Edge → shell) and needs a real mapping. (2) Convergence
was measurable this time: the unordered instrument read was found by **three** layers, the
self-referential band and the capture-recipe defect by **two** each.

**WHAT THIS REVIEW DOES NOT ESTABLISH, stated plainly:** nothing here has been observed on the
vehicle. **ACs 7, 12, 13, 15, 16 and 17 remain OPEN and unobserved**, and Tasks 6 and 9 are
untouched. Unit-green is not feature-proof. This review does not close the story — only Wolf does.

**One thing to watch at the live viewing, not a defect:** the flicker band actually reached is torch
±7.0% and campfire ±11.0% (14% / 22% peak-to-peak) at 1.7 Hz / 0.9 Hz — safely inside 5.4's
warm-pixel floor, but a *subtle* breath. Given the "did not see the difference" reaction to the Task
0 pair, if it reads too faint the amplitude column (`appearance.rs:61`, `:68`) is the single-number
knob and widening it will not endanger the capture range checks.


### Task 6 — the live vehicle session (2026-08-18, gingerspice / native Windows / NVIDIA)

Vehicle: `NVIDIA GeForce RTX 4080 Laptop GPU`, `DiscreteGpu`, Vulkan, driver `591.74`. Startup line
read `projected 53365 terrain cubes` on every run — unchanged from 5.4.

**The "before" capture** — `gui.exe 7451 --capture 6-1-motion-before.png --frames 1500`, zero
commands issued:

```
motion: ticks observed=107 dwarf position changes=49 mid-blend frames=628 max working dwarves=0 item count=0
capture range check: warm-lit pixels=29072 ground-median-luminance=123
```

**AC12 IS MET BY THIS RUN** — ≥100 delivered ticks with zero commands issued, a non-zero count of
dwarf position changes and a non-zero count of mid-blend frames. Two corroborations rather than bare
green: `49/107 = 45.8%` of ticks carried a dwarf position change against the **47% wander baseline
measured independently at story-creation** on the same seed; and ground-median **123** is exactly
5.4's converged figure (123 measured / 123.3 in the approved artifact), so **wow beat 1's look is
undisturbed by this story**.

The `--frames 1500` correction earned itself immediately: at the story's original 600 this run would
have observed ~44 ticks and panicked before writing any PNG.

**The first "after" attempt FAILED, and the failure is worth keeping.**

```
motion: ticks observed=143 dwarf position changes=65 mid-blend frames=689 max working dwarves=0 item count=8
thread panicked at crates/gui/src/capture.rs:89: capture observed no working dwarves
```

`item count=8` with `max working=0` says the eight tiles were **already dug when the window opened** —
the designation had completed between the two runs. Not a defect: `WORK_TICKS` is 5, so 8 tiles are
40 tick-units of work, and the accumulator samples every frame (~14 samples per tick here), which
cannot miss work that is actually happening. The instrument correctly refused to certify a run in
which it observed none.

**This is review patch #2 paying for itself on its first live use.** The motion line printed BEFORE
the panic, so all five numbers were available to diagnose with. Under the code as originally
delivered the assertion ran first and the operator would have seen only
`capture observed no working dwarves` — indistinguishable from "the feature is dead".

**The "after" capture** — daemon restarted for a fresh world, `--frames 3000 --expect-work`, dig
designated from a TUI client early in the window:

```
motion: ticks observed=259 dwarf position changes=158 mid-blend frames=1364 max working dwarves=3 item count=8
capture range check: warm-lit pixels=28777 ground-median-luminance=123
Screenshot saved to 6-1-motion-after.png
```

**All five AC14 assertions passed, including both `--expect-work` halves.** Again corroborated
rather than merely green: `max working dwarves=3` is **exactly** the "up to 3 dwarves working at
once" measured at story-creation, and `item count=8` is **exactly** the 8 mineral tiles of the named
site `[58,68,9]`–`[64,69,9]`. Position changes rose to 61% of ticks (158/259) against the 45–47%
idle baseline, consistent with dwarves pathing to the dig. Ground-median held at **123** across both
runs, dig in progress and all lights flickering.


**AC13 — the NFR6 reading, MET.** F3 overlay, dig in progress, all lights flickering, full
128x128x32 world: **sustained >143 fps at BOTH working zoom and full vista**, labelled
**`gingerspice / native Windows / NVIDIA`** (RTX 4080 Laptop, Vulkan, driver 591.74). That is
**2.4x** the 60 fps working-zoom bar and **4.8x** the 30 fps vista bar. Consistent with 5.4's
">135 fps at every zoom" on the same vehicle, so this story's blend, flicker and debris cost no
measurable frame time. The WSLg-devpod figure NFR6 names remains unmeasurable here (5.3's envelope
finding) and stays formally owed to the Epic 5/6 retro's bar-redefinition question — recorded, not
blurred.

**Operator note for whoever repeats this:** the AC15 self-test recipe in this story is written for
`cmd.exe` (`set VAR=value`). Run from PowerShell it silently sets nothing and the test panics with
`first capture path is required` — which is the test being correct, not a defect. PowerShell needs
`$env:FROSTVEIN_CAPTURE_FIRST = "..."`, and absolute paths, since `gui.exe` writes the PNGs relative
to its launch directory while the test resolves relative to its own.


**AC15 — MET on the vehicle.** `capture_exists_is_not_black_and_changes_with_the_world` PASSED
against the pair produced above, so the projected dig-site window carries **≥200 changed pixels** —
a threshold snowfall and the aurora cannot reach (measured on the approved Task 0 pair: ~5 expected
inside the window from atmosphere alone, against 1,651 of real signal). The whole-file byte
comparison this replaced would have passed on atmosphere alone.

The run also surfaced a small gap of the same class the review patched in `capture.rs`: **the test
asserted without reporting**, so a pass yielded the verdict but not the margin. It now prints the
changed-pixel count and the window bounds before asserting. **This does not require a re-run** — the
AC is met by the pass; the print is so the next operator gets the number rather than only the
verdict. It is inside the `#[ignore]`d vehicle test, so the headless gate is unaffected.


### The live viewing and its two rulings (2026-08-18) — Task 6 partially re-opened

Wolf viewed the built result on the vehicle. **Dwarves slide** — AC2's headline outcome confirmed
live. Two findings came out of it, both his, both correct, and both fixed here.

**RULING 1 — the flicker read as STATIC, and the mechanism was never the problem.** Measured in a
real app the campfire's `PointLight` takes 5,587 distinct intensity values with a 22% peak-to-peak
swing, so it was animating exactly as designed. At torch ±7% / campfire ±11% — about a tenth of a
stop, through HDR tonemapping — it is simply below the threshold at which a light pool reads as
breathing. AC10's mechanism clauses were met and **AC10's observable intent was not**, which on this
project means not met. **Wolf's ruling: strong — torch 0.30, campfire 0.40** (~4x), on the reasoning
that a timid bump risks a second failed viewing and a whole extra vehicle session. The hand-written
band literals and the peak-reached assertion moved with the table, so the test still cannot go
tautological. Warm-pixel headroom is ample: the floor is 3,000 and the vehicle measured ~28,800.
This was the exact risk the code review flagged in advance and carried into the runbook.

**RULING 2 — THE DIG SITE IS RE-PICKED.** Wolf: *"after digging tile walls are still in place"*. He
was right and my first diagnosis (snow caps re-whitening the trench floor) was wrong. Measured
cause: the named rect straddles a **slope**, and the sim represents slope tiles as `Tile::Ramp`,
which is **not diggable** (`sim-core:1339-1341`). Of the 14 tiles in `[58,68,9]`–`[64,69,9]`, 8 were
solid, 2 already empty, and **4 were ramps at x=58–61, y=69 — contiguous**. They stood at full
height while the row in front was excavated: a wall through the middle of the dig.

**The story KNEW 8 of 14 were diggable and recorded it at creation. What nobody drew was that the
leftovers are contiguous rather than scattered.** That makes it the eighth instance of the
AC-text-defect class this story's own Dev Notes track — and the first one caught only by a human
eye on the vehicle, which is precisely what the sign-off gate exists for.

**A false start worth recording, because it is the argument for checking before sending Wolf back to
the vehicle.** The first replacement I proposed, `[64,57,9]`–`[66,58,9]`, was all-solid and in frame
— and a raytrace from the boot camera found **5 of its 6 tiles occluded by tree foliage**. It would
have been strictly worse than the site it replaced, and the failure would have surfaced only at the
next live viewing.

**The replacement, chosen on four constraints at once:** solid + sky-exposed + unoccluded from the
boot camera + in frame. Exactly **19 tiles in the whole neighbourhood** satisfy all four, and
`[55,62,9]`–`[56,65,9]` is the only 8-tile rect among them — the choice was forced, not preferred.
**Re-verified live end to end before any vehicle run**, with the real TUI key sequence:

```
8 designations issued · first dwarf in Work at ~24 ticks · ALL 8 TILES DUG IN 52 TICKS
8 stone items left standing · max 2 dwarves working at once
tiles still standing inside the site: NONE - clean excavation, no wall
projection (0.424,0.692)-(0.455,0.721)  -- same v-band as the original
```

52 ticks is **identical** to the original site's measured figure, and the tile count is unchanged,
so **AC7's "8 mineral tiles" needs no amendment** and AC9's framing assertion simply moves. New key
sequence: `d,h,h,h,h,h,h,h,h,h,k,k,enter,l,j,j,j,enter`.

**What this costs in already-collected evidence.** ACs 12, 13 and 14 stand — they are about motion,
frame rate and the instrument, none of which depend on which tiles are dug. **AC15 must be re-run**:
its window now points at the new site, and the 2026-08-17 capture pair is of the old one. The
approved artifact carries an amendment note rather than a rewrite; its PNGs are kept as the record
of what was actually approved on 2026-08-17.

### File List

- `_bmad-output/implementation-artifacts/6-1-signoff/what-you-will-see.md` (new)
- `_bmad-output/implementation-artifacts/6-1-signoff/6-1-before.png` (new — vehicle capture)
- `_bmad-output/implementation-artifacts/6-1-signoff/6-1-after.png` (new — vehicle capture)
- `_bmad-output/implementation-artifacts/6-1-signoff/6-1-digsite-inset.png` (new — marked frames +
  7× dig-site crops, generated from the pair)
- `crates/gui/src/blend.rs` (new — `TickClock`, `blended_translation`, literal-oracle tests)
- `crates/gui/src/appearance.rs` (modified — flicker columns + `flicker_scale`)
- `crates/gui/src/project.rs` (modified — blend is sole `Transform` writer, `ProjectedLight`,
  `DigChip`)
- `crates/gui/src/ingest.rs` (modified — clock reset, chained tuple, `projection_systems`,
  `--expect-work`)
- `crates/gui/src/capture.rs` (modified — motion accumulator + AC14 assertions)
- `crates/gui/src/lib.rs` (modified — `pub mod blend;`)
- `crates/gui/tests/headless.rs` (modified — four app-level seam tests, registered via
  `projection_systems`)
- `crates/gui/tests/capture.rs` (modified — dig-site-window comparison)
- `docs/tech-art-guidelines.md` (modified — motion + flicker + debris section)
- `_bmad-output/implementation-artifacts/mutations/6-1-the-world-moves.sh` (new — 8 mutations)
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

- `_bmad-output/implementation-artifacts/deferred-work.md` (modified — 16 LOW review findings,
  each with file:line, plus the note that this story closed two entries without updating them)

*(The 2026-08-18 review-patch round touched no new files: `crates/gui/src/{appearance,blend,capture,
ingest}.rs`, `crates/gui/tests/{headless,capture}.rs`, the mutation script and this story file were
all already listed above.)*

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-18 | **Live viewing: two rulings by Wolf, both fixed.** (1) The flicker read as STATIC though the mechanism ran correctly (5,587 distinct intensities measured) -- amplitude raised torch 0.07 -> 0.30 and campfire 0.11 -> 0.40 on his ruling, band literals and peak assertion moved with it. (2) **The dig site is RE-PICKED to `[55,62,9]`-`[56,65,9]`**: the original straddled a slope and 4 undiggable `Tile::Ramp` tiles stood as a contiguous wall through the excavation -- the story knew 8 of 14 were diggable but nobody drew that the leftovers were contiguous (8th AC-text defect, first caught only by a human eye). A first replacement was rejected by raytrace: 5 of 6 tiles occluded by trees. The chosen site is the ONLY 8-tile rect near the camp that is solid + sky-exposed + unoccluded + in frame (19 tiles qualify at all), re-verified live: all 8 dug in 52 ticks (identical to the original), 8 items, NOTHING left standing. AC7's tile count needs no amendment. ACs 12/13/14 stand; **AC15 must be re-run** against the new window. |
| 2026-08-18 | **Code review (4 layers, fresh context) + patch round.** The feature was watched running live against a real daemon, and AC6 confirmed genuinely closed — but **three one-line deletions (`observe_tick`, `delta_secs`, `elapsed_secs`) each killed wow beat 2 with the suite 57/57 GREEN**, the same defect class as run one's inert `projection_systems`, one level below where AC6 looks. 1 decision + 8 patches applied: three production-drive tests, the motion line printed before its assertion, the mid-blend counter now reading real `Transform`s instead of the clock, `ProjectionSet` ordering for the instrument, literal flicker-band assertions (the old one was a tautology), a same-frame burst guard on the tick clock, a running-max item count, and a 200-pixel floor on AC15's window. Wolf ruled the `--frames 600` before-run up to 1,500 — at 10 Hz against >135 fps it was ~44 ticks and would have panicked before writing any PNG on Task 6's first command; the Task 0 recipe was left untouched as a record of what was actually run, and the Task 6 outputs renamed so they cannot overwrite the approved pair. Gate GREEN cold, 311 workspace tests, **mutations 8 → 14, all KILLED**. 16 LOW findings deferred with file:line. Status → in-progress: Tasks 6 and 9 and ACs 7/12/13/15/16/17 remain OPEN and vehicle- and human-bound. |
| 2026-08-17 | Continuation run closed all four falsified ACs in five commits (one per AC + mutations). **AC6 verified MET by the same sabotage that falsified it: 54/54 green with the feature deleted became 4 tests RED.** Mutation table 4 → 8, all KILLED. Gate green, 306 workspace tests. Status → review for the HEADLESS half; Tasks 6 and 9 and ACs 13/16/17 remain open and vehicle/human-bound. Self-gate remains a coverage hole — no conclusion on either run. |
| 2026-08-17 | Orchestrator verification of the Codex dev run. Gate green, 4/4 mutations killed, AC19 scope exact, all commits Völundr, nothing pushed — but **AC4, AC5, AC6 and AC11 are unmet or untested and four subtasks were ticked without being delivered.** AC6 falsified by sabotage: removing both `blend_projection` and `flicker_projection` from the live tuple leaves the full suite 54/54 green, because `projection_systems` is called only by `run()` and never by the headless tests. Tasks 1, 2 and 3 reopened. Self-gate recorded as a coverage hole (ran once, returned nothing). Commit cadence 3 commits for 7 tasks, below the floor. |
| 2026-08-17 | **Task 0 APPROVED BY WOLF — AC1 MET, the gate is OPEN, implementation may start.** Approved as-is; the named dig site is deliberately NOT re-picked (re-picking would invalidate the live-verified exposure/occlusion/projection/timing figures and force amendments to AC7 and AC9). Dev delegated to Codex for Tasks 1-5, 7, 8; Tasks 6 and 9 stay vehicle- and human-bound. |
| 2026-08-17 | Task 0 capture pair taken on the vehicle. Measured: the dig site is 64×43 px = 0.30% of frame at boot framing, and 72% of the 2,255 changed pixels fall inside the predicted window — the dig is correct and correctly located but invisible to the eye at the vista. `6-1-digsite-inset.png` added (marked frames + 7× crops) to make the pair legible, and line 8 added to "what you will NOT see": the dig face reads at working zoom, not at the boot vista. Consequence flagged for AC7/AC8/Task 4. AC15's windowed-comparison design vindicated in advance. Gate still closed pending Wolf's approval. |
| 2026-08-16 | Task 0 part (c) RULED by Wolf: wow beat 2 is signed **without** UX-DR14's carried stone (option A) — no sim story spun, no wire change, clause formally not delivered in M2. Wolf is taking the before/after capture pair on gingerspice rather than the written-only fallback; his approval follows that viewing, so AC1 remains unmet and the gate stays closed. |
| 2026-08-16 | Task 0 written half delivered: `6-1-signoff/what-you-will-see.md` (four additions + seven-line "what you will NOT see"). Carried-stone raise re-verified against shipped `sim-core` and sharpened — haul jobs come only from stockpiles and 6.1 places none, so no dwarf carries a stone in this scenario at all. Capture pair and Wolf's ruling still owed; gate closed, no implementation started. |
| 2026-08-16 | Story created. Dig site named and verified live on the shipped seed (8 mineral tiles, all sky-exposed and unoccluded, dug in 52 ticks, 8 stone items left standing); wander motion baseline measured at 47% of ticks; TUI key sequence executed end to end; the carried-stone gap in UX-DR14 verified against `sim-core` and raised as a blocking Task 0 clarification. |
