---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: db1c8475902a9822aec2b07052a56d2a8f6568e8
---

# Story 7.1: Slice Into the Mountain

Status: done

<!-- First story of Epic 7. Not on any cut list. It is also the story that gives the dig DEPTH:
     6.1's excavation is one voxel deep because a designation is a 2D rect at one z, and Wolf's
     live viewing found it barely reads. Slicing is how the underground becomes legible. -->

## Story

As the boss,
I want to slice into the mountain by z-level and always know which level I am on,
so that I can see and work the underground the dwarves are digging into.

## The sign-off gate — read before touching any code

**Opening half (Task 0, BLOCKING):** no implementation commit before Wolf has approved one "here is
what you will see" artifact at `_bmad-output/implementation-artifacts/7-1-signoff/`, in three parts:
(a) a before capture at the boot framing from the **shipped 6.1 binary**, (b) the written list of
what slicing adds, (c) an explicit **"what you will NOT see"** list.

**Closing half (AC15):** done only when Wolf has viewed the built result live on the vehicle and
signed off. A capture serves the comparison; it never replaces the live viewing (AD-17).

## The live vehicle — unchanged, do not re-derive

**No devpod can open a window** (no graphics userspace, measured at 5.3). The vehicle is the native
Windows client on **gingerspice**: cross-compiled `gui.exe`, `simd` in WSL, localhost, native NVIDIA
Vulkan. 6.1 measured **>143 fps at both working zoom and full vista**. Everything except the live
viewing, the NFR6 reading and the captures is headless-testable under `MinimalPlugins`.
**Never fake the live half.**

## Acceptance Criteria

### The gate

1. Before any implementation commit, Wolf has approved the sign-off artifact at
   `_bmad-output/implementation-artifacts/7-1-signoff/`.

### The control, chosen by testing

2. The slice control is chosen **by testing, on the record**: the story states what was tried, what
   each felt like, and why the winner won, from the candidates — modifier+wheel, dedicated keys
   (`<`/`>`, TUI parity), or slice-follows-selection (FR33, UX-DR3).
3. **The collision is resolved explicitly and against the code as it actually is.** Verified at
   story-creation: `gui` binds **no mouse input at all** today and camera zoom sits on the `Q`/`E`
   keys (`crates/gui/src/ingest.rs`, `camera_controls`). So the wheel is *unclaimed in code* while
   UX-DR2's zoom continuum *intends* to claim it. The story records that distinction rather than
   repeating the epic's "the mousewheel is already claimed" framing, and states what happens to the
   chosen control when the wheel does become zoom.
4. Behaviour **above ground level** is tested deliberately — the case Wolf flagged himself: at the
   topmost level the view is exactly today's full view, and slicing above the world's top is not
   possible.

### The slice

5. At slice level N, tiles above N are not drawn, and the **cut face at N is drawn** — a tile at z=N
   that was fully buried at full depth becomes visible as the floor of the cut. Which tiles are
   shown and hidden at level N is asserted headlessly under `MinimalPlugins` in `cargo test`, with
   no GPU involved (AD-17 rung 2).
6. The slice level **clamps at world bounds** `[0, dims.z-1]`; no input can drive it outside, and
   the clamp is asserted at both ends.
7. World-projected **entities** above the slice are hidden with the terrain, so a dwarf on the
   surface does not float over a sliced-open interior. *(If the ruling at Task 1 is that entities
   stay visible, that is legitimate — but it is a decision to record and test, not to leave
   undefined.)*
8. Dug corridors and channels underground are visible as the dwarves left them when sliced down to,
   projected from mirror state alone (FR33, AD-14).

### Knowing where you are

9. The current z-level is **always readable on screen** without guessing — the anti-requirement bar
   for *confusing* (UX-DR18). It is legible at the boot framing, not only when the F3 diagnostic
   overlay is enabled.
10. It is unambiguous whether what you are looking at is underground or the surface (FR33, UX-DR18).

### Client-local, never wire

11. The slice level is **client-local view state and never wire state**: the daemon does not know or
    care which level a client is looking at, `gui` sends nothing when it changes, and **two clients
    on the same daemon can sit at different levels simultaneously** (NFR5, AD-14). Asserted, not
    assumed: **both of these are empty** —
    `git diff --stat db1c847..3a5abb1 -- crates/protocol crates/simd crates/sim-core crates/client-core`
    (7.1's own commit range) and `git diff --stat -- crates/protocol crates/simd crates/sim-core
    crates/client-core` (the review patches, uncommitted). Verified empty 2026-08-19.
    *(Corrected at code review 2026-08-19. The original text named `main..HEAD`, which returns 6
    files — all inherited from the unmerged 6.1/6.2 predecessors this branch is stacked on. The
    review's first correction, `6-2-lanterns-in-the-dark..HEAD`, was ALSO wrong: 6.2's own patch
    commit `75f04a2` landed after 7.1's commits and touches `sim-core`, so a branch-tip range can
    never be clean here. On a stacked branch only the story's explicit commit range answers this
    question, and anyone pasting an "empty" result from the original command would have been
    recording false evidence.)*

### The instrument

12. `gui <port> --capture <path> --frames N --z N` pins the client to a level — the parity of
    `tui --z N` — and **range-checks a non-zero count of what it came to see at that level before
    any conclusion**: the drawn-tile count at that level, printed before any assertion. The 6.1
    motion line and the 5.4 range checks are retained unchanged. **Exit 0 is not a result**
    (AD-17 rung 3).
13. The startup draw-set oracle stays honest across slicing. Today it prints
    `projected 53365 terrain cubes` at full depth (5.4's pinned figure). At a slice the count is
    necessarily different, so the line states the level it counted, and the full-depth figure is
    still reproducible.

### Measured where a window exists

14. With the full world at a slice level, the frame-time overlay reads a sustained **60 fps at
    working zoom** and **≥30 fps at full vista** on the live vehicle, recorded **labelled with its
    machine**. If a reading fails, that measurement is the story's finding and is reported (NFR6).

### The closing half

15. Wolf has viewed the built result live on the vehicle, compared it against the approved artifact,
    and signed off. **A dev agent cannot check this box.**

### Evidence

16. A sabotage table exists at
    `_bmad-output/implementation-artifacts/mutations/7-1-slice-into-the-mountain.sh` and every
    mutation is KILLED on a genuine assertion, with the RED table pasted into the Dev Agent Record.
17. `scripts/gate.sh` is green and **7.1's own commits** touch only `crates/gui`, `docs/` and
    implementation-artifacts: **no wire change**, and no change to `sim-core`, `simd`, `protocol`,
    `client-core` or `tui`. *(Same correction as AC11: scope the check to this story's commit
    range, not to `main..HEAD`, while the branch is stacked on unmerged predecessors.)*

## Tasks / Subtasks

- [x] **Task 0 — The sign-off artifact (Wolf's gate)** (AC: 1) — **GATE OPENED 2026-08-18 ON THE
      WRITTEN-ONLY FALLBACK**, on the same basis as 6.2: Wolf, travelling, said *"Well let's Start
      dev then"* to a message covering both stories. He approved **proceeding**, not the artifact
      line by line, and no before-capture was taken because no vehicle session was available. AC1
      MET on that basis; Wolf still closes the story at Task 8.
      **THE CONTROL RULING IS PROVISIONAL, NOT WOLF'S.** He has not answered which control drives
      the slice. Build `<` / `>` — TUI parity with `tui --z N`, no collision with the currently
      unbound wheel, and nothing to migrate when UX-DR2 later takes the wheel for zoom. Record it as
      **provisional**: it is one key binding and is cheap for Wolf to reverse at the viewing.
  - [x] *(N/A — no vehicle session available; pair SKIPPED, written-only fallback taken.)*
        Take a before capture on the vehicle with the **shipped 6.1 binary**:
        `gui.exe 7451 --capture 7-1-before.png --frames 1500`. Store in `7-1-signoff/`.
        *(1500, not 600 — see 6.1's tick-floor lesson.)*
  - [x] Write `7-1-signoff/what-you-will-see.md`: slicing down into the mountain, the cut face, the
        level readout, and the dug site seen from below.
  - [x] Write the **"what you will NOT see"** list and get each line ruled on: no designation or
        zone rendering (7.2); no commands or picking from `gui` (8.x); no cutaway shading or
        cross-section hatching — the cut face is the same terrain material, not a new look; dwarves
        remain scaled cubes. **Raise explicitly:** which control won and why, so Wolf rules on the
        control *before* it is built rather than at the viewing.

- [x] **Task 1 — Choose the control by testing** (AC: 2, 3, 4)
  - [x] Tried the candidates against the actual client path. The wheel is **unbound in code** today,
        but modifier+wheel would need to move when UX-DR2 gives the wheel to continuous zoom; it was
        therefore rejected before adding a migration. Slice-follows-selection was rejected because
        `gui` has no picking or selection in 7.1. Dedicated `<`/`>` is a discrete, immediate
        keyboard step in the headless production schedule and matches `tui --z N`. No live vehicle
        "feel" observation was possible in this devpod; the input/rebuild behaviour was tested
        headlessly and the live feel remains Task 5/Wolf's call.
  - [x] **PROVISIONAL (Wolf has not confirmed):** `<` lowers and `>` raises the client-local slice.
        It wins TUI parity, has no current binding collision (`Q`/`E` retain zoom), and is one cheap
        binding to reverse at the viewing.
  - [x] When UX-DR2 later gives the wheel to zoom, `<`/`>` remain unchanged; no wheel slice binding
        needs migration.

- [x] **Task 2 — The slice** (AC: 5, 6, 7, 8)
  - [x] Add the slice level as a `ClientLocal` resource (the `TickClock` precedent in
        `crates/gui/src/blend.rs` is the shape to copy), defaulting to the world top so the boot
        frame is unchanged.
  - [x] Change the terrain draw set so a tile is drawn when `z <= level` **and** (`is_exposed` at
        full depth **or** `z == level`) — the second arm is what makes the cut face appear. Keep
        `is_exposed` itself intact; slicing is a view filter over it, not a new exposure rule.
  - [x] Hide world-projected entities above the level (or record the opposite ruling).
  - [x] Rebuild the terrain when the level changes, reusing the existing snapshot-rebuild path
        rather than adding a second one.
  - [x] Tests: shown/hidden sets at a mid level; the cut face appears at `z == level`; clamping at
        `0` and at `dims.z-1`; at the top level the drawn set is **identical** to today's full-depth
        set (AC4); an entity above the level is hidden and one below is not.

- [x] **Task 3 — Knowing where you are** (AC: 9, 10)
  - [x] Draw the current level on screen, legible at the boot framing and independent of the F3
        diagnostic overlay.
  - [x] Test what can be tested headlessly (the value the readout is fed), and state plainly in the
        record which half is only confirmable by eye at Task 5.

- [x] **Task 4 — The instrument** (AC: 12, 13)
  - [x] Add `--z N` to `gui`'s arg parsing, rejecting it without `--capture` the way `--expect-work`
        already is (`crates/gui/src/ingest.rs`).
  - [x] Print the drawn-tile count **and the level it counted** before any assertion; assert the
        count is non-zero at the requested level.
  - [x] Keep 6.1's motion line and 5.4's range checks exactly as they are.
  - [x] Unit-test the accumulator: a level with nothing to draw fails, a level with terrain passes.

- [x] **Task 5 — The live vehicle session** (AC: 8, 9, 10, 14)
  - [x] Cross-compile and launch per Verification; slice down to the 6.1 dig site and confirm the
        excavation is visible from below.
  - [x] Read the F3 overlay at working zoom and at full vista **at a slice level**; record both
        labelled `gingerspice / native Windows / NVIDIA`.
  - [x] Confirm by eye and state in the record: the level readout is legible; underground reads as
        underground; the cut face is not confusing. **AC10's `surface`/`underground` ruling was not
        given — see the Dev Agent Record.**

- [x] **Task 6 — Tech-art guidelines** (AC: 5 supporting)
  - [x] Add one short section to `docs/tech-art-guidelines.md`: slicing is a view filter over the
        existing exposure rule, the cut face is the same material, and the level is client-local.

- [x] **Task 7 — Evidence and the gate** (AC: 16, 17)
  - [x] Write the sabotage table following 6.1's format; run `scripts/mutate.sh` **alone** and paste
        the RED table. Run `cargo clean -p gui` **after** the mutation round.
  - [x] `scripts/gate.sh` green; confirm the diff touches no crate but `gui`.

- [x] **Task 8 — Wolf's closing sign-off** (AC: 15)
  - [x] Wolf views live against the approved artifact and signs off. **A dev agent cannot check
        this box.** Signed 2026-08-20: *"i think we are done with these stories"*.

### Review Findings

Four-layer code review, 2026-08-19, fresh context. No layer was a coverage hole: all four verified
`cargo 1.97.1`, built in isolated `CARGO_TARGET_DIR`s, and executed code. R1 territories were
extended to `crates/gui` on Wolf's ruling (gui predates neither hunter's named territory): Blind
Hunter took `slice.rs`/`project.rs`/`ingest.rs`, Edge Case Hunter took `capture.rs`/`lib.rs`/
`tests/headless.rs`/`docs/`; both Opus auditors kept whole-diff scope.

**The cut itself is proven.** Blind Hunter fuzzed the real `terrain_positions_at`/`is_exposed`/
`SliceLevel` across ~1.1M position checks and 4,000 ECS steps with zero failures; the Feature
Auditor drove a live `simd` and measured 15,316 of 16,071 cut-face tiles supplied only by the
`z == level` arm. The hollow-shell trap named at story creation is genuinely closed. Every defect
below sits in the layer that *observes* the feature, not the feature.

- [x] [Review][Patch] **RULED 2026-08-19 (Wolf): label keys off terrain above the cut** — "underground"
      iff any solid/ramp tile exists strictly above the cut level. The surface/underground label is
      positional, not content-derived —
      `label()` is `if level == top { "surface" } else { "underground" }`, with no relation to where
      terrain actually is. Measured against the live world: z 31 and z 30 draw an identical 53,365
      cubes with zero terrain at z 30, yet one `<` press flips the readout to "underground"; z 10-30
      all read "underground" while the cut plane sits in open sky above a mountain whose camp is at
      z 9. AC10 requires it be unambiguous whether you are looking at underground or the surface.
      This is a mechanism defect, not a legibility question for Wolf — but what the label should key
      off (terrain above the cut? cut plane intersecting solid? highest solid in view?) is a design
      call. `[feature+auditor/MED]` `crates/gui/src/slice.rs:56-62`
- [x] [Review][Patch] **RULED 2026-08-19 (Wolf): allow `--z` without `--capture`** — drop the `bail!`,
      which fixes the broken recipe line and lets the client boot pinned. No page-step key. The client
      cannot boot at a level, and AC11's own recipe line would error — `--z` is gated behind `bail!("--z requires --capture")`, so the interactive client
      always starts at the top; reaching the 6.1 dig site is 22 discrete `just_pressed` taps with no
      key-repeat, no page-step and no jump-to-N. The AC-extract's own recipe `gui.exe 7451 --z 9 ...`
      carries no `--capture` and would exit with an error as written. Decide whether `--z` should be
      allowed without `--capture` (which fixes both) before Task 5. `[feature/MED]`
      `crates/gui/src/ingest.rs:241`, `:363-368`
- [x] [Review][Patch] **RULED 2026-08-19 (Wolf): fix it** — add `Res<MirrorResource>` to
      `capture_after_frames` and assert lanterns iff at least one dwarf sits at or below the cut.
      Lantern assertions are skipped at every non-top level — the 6.2 guard
      `if capture.lantern.observed() || slice.level() == slice.top()` cannot distinguish "the
      operator sliced below the dwarves" from "the slice filter broke entity projection entirely",
      so at any level below top a regression projecting zero lanterns exits 0. Every 7.1 capture is
      at z 9, so this is the common case, not the edge. It does print a line, so it is not silent.
      The fix needs `capture_after_frames` to read the mirror (does any dwarf sit at or below the
      cut?), which is a new resource dependency in 6.2's reviewed code — Wolf's call whether to
      spend it. `[auditor/MED]` `crates/gui/src/capture.rs:414-425`
- [x] [Review][Patch] The on-screen readout is registered outside `projection_systems` and has no
      test; deleting both its systems leaves 84/84 green — AC9's entire mechanism is undefended,
      and `projection_systems`' own doc comment states the rule that was skipped
      `[feature+auditor/HIGH]` `crates/gui/src/ingest.rs:110`, `:122`, `:295-317`
- [x] [Review][Patch] `--z` pinning is untested and can be made completely inert with a green suite
      — replacing `SliceLevel::pinned` with `at_world_top` passes 84/84, so `gui --z 9 --capture`
      would silently photograph the full-depth view and manufacture false evidence for AC8/AC12
      `[auditor/HIGH]` `crates/gui/src/ingest.rs:85-88`
- [x] [Review][Patch] `DrawStats::assert_valid` is level-blind — `terrain_tiles > 0` over a global
      count, and world-boundary tiles are always exposed, so a hollow-shell cut with no floor passes
      identically to a correct one (measured 209 vs 258 on a 9x9x9 block, 49 floor tiles missing).
      The doc comment claims a guarantee the code does not deliver `[edge/MED]`
      `crates/gui/src/capture.rs:36-42`
- [x] [Review][Patch] `draw.assert_valid()` was inserted ahead of the lantern, motion and 5.4 range
      diagnostics, inverting the convention the comment directly above it states — a capture whose
      slice draws nothing now panics with none of the other five numbers printed `[auditor/MED]`
      `crates/gui/src/capture.rs:399`
- [x] [Review][Patch] The readout has no `GlobalZIndex` (defaults to 0) and sits under Bevy's FPS
      overlay, which uses `GlobalZIndex(i32::MAX - 32)` at font 32 from the origin — the overlap
      covers the level number itself, in exactly the session AC14 requires reading both
      `[feature/MED]` `crates/gui/src/ingest.rs:300-307`
- [x] [Review][Patch] Item visibility above the slice has no oracle — removing both
      `item.pos[2] <= slice.level()` filters leaves 84/84 green; the sabotage table mutates only the
      entity filter `[auditor/MED]` `crates/gui/src/project.rs:331-341`
- [x] [Review][Patch] AC11/AC17's stated proof command is false on a stacked branch —
      `git diff --stat main..HEAD -- crates/protocol crates/simd crates/sim-core` returns 6 files,
      all inherited from the unmerged 6.1/6.2 predecessors. The substance is clean (a per-commit
      audit of 7.1's 13 commits shows only `crates/gui/`, `docs/`, `_bmad-output/`), but the AC's
      own check cannot pass and will mislead anyone who runs it. 10th instance of the AC-text-defect
      class `[feature+auditor/MED]` story AC11 + AC17
- [x] [Review][Patch] The sign-off artifact contradicts the code — it promises "The boot frame is
      unchanged" while the implementation adds a permanent 22px readout that
      `force_capture_overlay_off` does not suppress, so it will appear in `7-1-slice.png` and every
      future 5.4/6.1/6.2 capture. It also names no keys, so Wolf must be told them out of band at
      Task 5, and the binding is physically `,`/`.` (bare comma steps too), not literally `<`/`>`.
      Verified harmless to the 5.4 range checks: the readout is blue-dominant and outside the
      luminance sample region `[auditor+feature/MED]`
      `_bmad-output/implementation-artifacts/7-1-signoff/what-you-will-see.md`
- [x] [Review][Patch] `MinimalPlugins` never runs `ButtonInput::clear()`, so a pressed key stays
      just-pressed forever — the Feature Auditor's probe took two level steps from one press. The
      three new tests survive by luck; the next one to press a slice key and update twice will
      assert against the wrong world. Production is unaffected `[feature/LOW, patched as it sits in
      a file this round already edits]` `crates/gui/tests/headless.rs:32`
- [x] [Review][Patch] `mirror.items()` is filtered by `pos[2] <= slice.level()` twice — once for
      `item_ids`, again in `wanted.extend(...)` `[blind/LOW, patched as it sits in a function this
      round already edits]` `crates/gui/src/project.rs:331-341`
- [x] [Review][Defer] `SliceLevel::rebind` is untested and speculative — replacing it with `false`
      passes 84/84; it guards a snapshot changing world dims, which cannot happen while
      `Dims::DEFAULT` is a constant `[auditor/LOW]` `crates/gui/src/slice.rs:44-47` — deferred
- [x] [Review][Defer] The capture's own slice line is untested (deleting the `println!` leaves
      84/84 green) and its format disagrees with the startup oracle's — `slice: z 9 projected 36788
      terrain cubes` vs `projected 36788 terrain cubes at z 9`, so a recipe grepping one shape
      misses the other `[auditor/LOW]` `crates/gui/src/capture.rs:395-398` — deferred
- [x] [Review][Defer] An out-of-range `--z` clamps silently with no diagnostic (`--z 999` becomes
      31, `--z -5` becomes 0); the printed line is honest about the level actually used
      `[auditor/LOW]` `crates/gui/src/slice.rs:17-21` — deferred
- [x] [Review][Defer] `update_slice_readout` rebuilds the whole `Text` every frame with no
      `is_changed()` guard `[feature/LOW]` `crates/gui/src/ingest.rs:313-317` — deferred
- [x] [Review][Defer] 6.1's vehicle runbook still quotes the pre-slice oracle string exactly;
      it now reads `projected 53365 terrain cubes at z 31` (a prefix match, so a human is fine)
      `[auditor/LOW]` `_bmad-output/implementation-artifacts/6-1-signoff/task-6-vehicle-runbook.md:86`
      — deferred

**Patch round applied 2026-08-19 (same session, batched — one verification pass, not one per fix).**
All 13 patches landed. Suite 84 -> 90 tests. The sabotage table grew 7 -> 13 mutations and
**13/13 KILLED**; `scripts/gate.sh` **GATE GREEN**, run and observed, after the mandated
`cargo clean -p gui` (the mutation round restores file contents without bumping mtimes, so the
first post-mutation test run was still executing the last mutant — the exact trap Task 7 warns of).

Two corrections made during the patch round, recorded because both were mine:
1. **AC11's first correction was also wrong.** Rescoping to `6-2-lanterns-in-the-dark..HEAD` still
   catches `75f04a2`, 6.2's patch commit, which landed *after* 7.1's commits and touches
   `sim-core`. On a stacked branch only the story's explicit commit range answers the question.
2. **The review's own diff baseline hid a coupled hunk.** `75f04a2` made the lantern assertion read
   `slice.level()`, so it belongs to this story's surface; it was excluded as already-reviewed and
   recovered only because the layers were told to read HEAD, not just the diff.

**Dismissed as noise (1):** the Acceptance Auditor's confirmation that the guardrail's performance
premise holds (slicing reduces the draw set at every level below the top, measured across all 32
levels, never above 53,365) — informational, not a defect.

**Convergence between layers (4 of 19 findings):** the untested readout (feature+auditor), the
label defect (feature+auditor), AC11's false proof command (feature+auditor), and the `,`/`.`
binding (feature+auditor). Notably better than Epic 3's 1-in-8, and both convergent pairs crossed
the Opus auditors rather than the territorialised hunters.

**Scope note:** the review baseline was `db1c847` (the story's own `baseline_commit`), which
excluded 6.2's later patch commits. `75f04a2` made the lantern assertion slice-conditional and is
therefore coupled to this story — the Acceptance Auditor caught it by reading HEAD rather than only
the diff, and it is carried above as a decision item.


## Dev Notes

### The epic's control-collision claim is STALE — verified against source

The epic frames 7.1 as resolving "the addendum's open control collision" and asserts "the mousewheel
is already claimed by the zoom continuum and one wheel cannot drive both". **Against the code as it
stands, no wheel is claimed by anything.** Verified at story-creation: `rg` over `crates/gui/src`
finds **no `MouseWheel`, no `MouseButton`, no `CursorMoved` and no mouse handling of any kind**, and
`camera_controls` (`crates/gui/src/ingest.rs`) binds yaw to `A`/`D`, pitch to `W`/`S` and zoom to
`Q`/`E` — keyboard only.

The collision is therefore **planned, not implemented**: UX-DR2's zoom continuum *intends* the wheel.
That does not dissolve the decision — it changes what "resolved by testing" means. Choosing the
wheel now costs a migration when zoom arrives; choosing `<`/`>` costs nothing and matches the TUI.
AC3 requires the story to say which, and why, against this reality rather than the epic's.

### Scope guardrails — do NOT build these here

- **No designation or zone rendering (7.2), no picking or commands (8.x).** `gui` still issues zero
  commands and leaves `designations()` / `zones()` unread.
- **No wire change and no change outside `gui` + docs.** The slice is client-local by AC11; if it
  seems to need wire state, that is a story-spec defect — raise it, don't code it.
- **No cutaway shading, no cross-section hatching, no separate cut-face material.** The cut face is
  the same terrain material as everything else; a new look is a story of its own.
- **No greedy meshing, chunking or LOD.** 6.1 measured >143 fps; slicing *reduces* the draw set at
  every level below the top, so the performance direction is favourable. Optimise only if AC14's
  reading fails, and then the *measured* problem drives it.
- **No multi-level designation.** Digging deeper is a sim/designation concern, not this story.
- **No workaround for driver or envelope problems in production code** (5.3's AC9 rule stands).

### What already exists (build on it, do not re-derive)

- **The draw set is one function.** `terrain_positions(mirror)` walks every position and keeps the
  ones `is_exposed` returns true for (`crates/gui/src/project.rs`). Slicing is a filter layered over
  that, plus the cut-face arm — **do not rewrite `is_exposed`**; it is load-bearing for the 53,365
  oracle and for ramp handling.
- **The terrain rebuild path already exists** and is driven by the snapshot branch of `reconcile`.
  A level change is a rebuild of exactly the same shape; reuse it.
- **The `ClientLocal` resource precedent is `TickClock`** (`crates/gui/src/blend.rs`), registered
  through `projection_systems` so live and headless wiring are one object — 6.1's AC6. Any new
  system this story adds goes in that same registration or it is invisible to the suite.
- **`tui` already takes `--z N`** (`crates/tui/src/main.rs:129`) and pins a level; AC12 is the
  parity of that flag, and story 3.3's false failure is why it exists.
- **The arg parser already rejects invalid combinations** (`--capture` requires `--frames`,
  `--expect-work` requires `--capture`) — follow that shape for `--z`.
- **6.1's dig site is `[55,62,9]`–`[56,65,9]`**, 8 tiles one voxel deep at z 9, with the floor at
  z 8 rendering as bare soil. That is the concrete thing to slice down to at Task 5.

### Key decisions & traps

- **The cut face is the whole point and it is the part `is_exposed` cannot give you.** A tile buried
  under rock is not "exposed" at full depth, so a naive `z <= level` filter over the existing draw
  set produces a hollow shell with holes where the mountain's interior should be. The `z == level`
  arm is what fills the floor of the cut. Expect this to be the first thing a mutation kills.
- **The 53,365 oracle is a full-depth figure.** It is pinned in 5.4's and 6.1's verification
  recipes. Slicing changes the count by construction, so the line must name the level it counted or
  every future recipe reads as broken. This is exactly the "instrument manufactures false evidence"
  class the project has been bitten by.
- **Default to the world top so the boot frame is untouched.** 5.4's approved composition and its
  capture range checks assume the full view; a client that boots sliced would change the frame Wolf
  signed off.
- **Entities above the slice are a real decision, not an oversight.** Hiding them is the natural
  reading of "underground versus surface"; leaving them visible makes dwarves float over a cut.
  AC7 requires whichever is chosen to be tested, so the suite records the ruling.
- **Two clients at different levels is the NFR5 proof (AC11).** It is cheap to demonstrate — two
  `gui` processes, or one `gui` and one `tui --z N` — and it is the assertion that keeps the slice
  out of the wire.
- **`mutate.sh` rewrites source in place** — run it alone, and `cargo clean -p gui` afterwards.
- When torn between simple and general, pick simple and leave a `// NOTE:` naming the limitation.

### Project Structure (files to touch)

```
crates/gui/src/slice.rs           NEW     the level resource + the draw-set filter + tests
crates/gui/src/project.rs         UPDATE  terrain draw set honours the level; entity visibility
crates/gui/src/ingest.rs          UPDATE  --z parsing, the control binding, systems registration
crates/gui/src/capture.rs         UPDATE  level-aware draw count + assertion
crates/gui/src/lib.rs             UPDATE  `pub mod slice;`
crates/gui/tests/headless.rs      UPDATE  shown/hidden sets, cut face, clamping, entity visibility
docs/tech-art-guidelines.md       UPDATE  slicing section
_bmad-output/implementation-artifacts/7-1-signoff/                          NEW  artifact + capture
_bmad-output/implementation-artifacts/mutations/7-1-slice-into-the-mountain.sh  NEW
_bmad-output/implementation-artifacts/deferred-work.md                      UPDATE if anything defers
```

### Previous story intelligence (deltas that change THIS story)

- **6.1 is not merged.** Branch from `6-1-the-world-moves`, not `main` — the blend, the flicker and
  the shared `projection_systems` registration all live there. If 6.1 has merged by the time this
  starts, branch from `main` and say so. **If 6.2 is dev'd first, both stories touch
  `crates/gui/src/capture.rs`** — rebase rather than resolving by hand at the end.
- **6.1's review found three untested "drive" lines**: deleting `observe_tick`, `delta_secs()` or
  `elapsed_secs()` each killed the headline feature with a green suite, because every seam test
  hand-fed the input. When writing AC5's tests, drive the level through the production path, not by
  poking the resource — the same defect is available here.
- **6.1's live viewing found two things a green suite could not see**: a flicker that ran correctly
  but was invisible, and a dig site whose undiggable ramps left a wall. Both were presentation
  truths. Expect "the level readout is legible" and "the cut face is not confusing" to be the same
  class — mechanism is testable, legibility is Wolf's call at Task 8.

### Verification

**Executed at story-creation (the headless half — non-zero evidence, P6 rule).** `rg` over
`crates/gui/src` returned **no mouse handling of any kind**, and `camera_controls` was read at source
confirming yaw/pitch/zoom are bound to `A`/`D`, `W`/`S`, `Q`/`E`. `terrain_positions` and
`is_exposed` were read at source and are cited above. A live `simd` snapshot was read off the wire to
confirm the dig-site column materials this story will slice down to (`[55,62,9]`–`[56,65,9]`, floor
soil at z 8).

**Gate (headless, any devpod, must be green before done):**

```bash
scripts/gate.sh
```

**The live vehicle (recipe proven at 5.3, 5.4 and 6.1):**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
# simd stays in WSL:  ./target/debug/simd 7451
# gui.exe runs on the Windows side against localhost:7451
```

**The slice capture (the obligation the dev agent inherits — it cannot run until the slice exists):**

```bash
# pinned to the dig level, so the excavation is what the capture came to see
gui.exe 7451 --capture 7-1-slice.png --frames 1500 --z 9
```

**Required non-zero observations** (paste all printed lines into the record): the startup line names
the level it counted and a non-zero drawn-tile count at that level; 6.1's motion line still reports
ticks ≥ 100, position changes > 0 and blend frames > 0; the range-check line reports warm-lit pixels
above the floor and ground-median inside `[70,180]`. **Exit 0 is not a result.**

**Two clients at different levels (AC11):**

```bash
./target/debug/tui 7451 --z 4 --frames 30      # one client at level 4
# gui.exe 7451 --z 9 ... on the Windows side   # the other at level 9, same daemon
```

**NFR6 reading (AC14):** F3 overlay on, at a slice level, read sustained fps at working zoom and at
full vista; record both labelled `gingerspice / native Windows / NVIDIA`.

**Sabotage:**

```bash
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/7-1-slice-into-the-mountain.sh
```

### If this overruns one session

The seam is **slice | readout**: Tasks 1–2 and 4 (the control, the cut, the instrument — the
headline outcome exists and is measurable) versus Task 3 (the on-screen level readout). Both halves
are needed for the gate, so a split defers the sign-off, not part of it. Commit per green task;
restate RED evidence in any continuation handoff.

### References

- Story 7.1 epic text — `_bmad-output/planning-artifacts/epics.md:894-932`; Epic 7 framing `:880-892`
- FR33, UX-DR2, UX-DR3, UX-DR18, NFR5, NFR6, AD-14, AD-17 — `epics.md`,
  `architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`
- The draw set and the exposure rule — `crates/gui/src/project.rs` (`terrain_positions`,
  `is_exposed`)
- Camera controls and the absence of mouse input — `crates/gui/src/ingest.rs` (`camera_controls`)
- The `ClientLocal` resource precedent and shared registration — `crates/gui/src/blend.rs`,
  `crates/gui/src/ingest.rs` (`projection_systems`)
- `tui --z N` parity — `crates/tui/src/main.rs:129`
- Capture range checks and the draw-set oracle — `crates/gui/src/capture.rs`
- 6.1's story, its review, its two live findings and the dig site — `6-1-the-world-moves.md`
- Story rules, instrument rule, exit-0 — `docs/technical-preferences.md:64-101`

## Dev Agent Record

### Agent Model Used

gpt-5.6 (Codex)

### Debug Log References

- **RED — Task 1/2 production-path test, before implementation:**
  ```text
  error[E0432]: unresolved import `gui::slice`
    --> crates/gui/tests/headless.rs:23:5
  ```
- **RED — Task 3 readout contract, before implementation:**
  ```text
  error[E0599]: no method named `readout` found for struct `SliceLevel` in the current scope
    --> crates/gui/src/slice.rs:79:26
  ```
- **RED — Task 4 capture instrument, before implementation:**
  ```text
  error[E0433]: cannot find type `DrawStats` in this scope
    --> crates/gui/src/capture.rs:427:21
  ```
- **RED — self-review regression, before the live-delta fix:**
  ```text
  assertion `left == right` failed: a later dig above the selected level must not leave floating debris
    left: 4
   right: 0
  ```
- **Mutation RED evidence — 2026-08-18:** `scripts/mutate.sh` was run alone, followed by
  `cargo clean -p gui`. All seven mutations were killed by a genuine assertion:

  | Mutation | Result | Assertion that went red |
  | --- | --- | --- |
  | cut face no longer fills buried terrain | KILLED | `keyboard_slice_rebuilds_the_cut_face_and_hides_surface_entities`: shown set mismatch |
  | slice no longer hides surface entities | KILLED | same production-path test: surface dwarf remained projected |
  | slice input stops requesting the established rebuild path | KILLED | same production-path test: old terrain remained after input |
  | slice can rise above the world top | KILLED | `the_slice_starts_at_the_top_and_clamps_at_both_world_bounds` |
  | slice readout loses its underground state | KILLED | `the_readout_names_the_current_level_and_whether_it_is_surface_or_underground` |
  | capture accepts an empty requested slice | KILLED | `draw_count_instrument_rejects_an_empty_level_and_accepts_terrain` |
  | capture z works without capture mode | KILLED | `capture_slice_level_requires_capture_and_is_retained_for_pinning` |

- **Gate — 2026-08-18:** `scripts/gate.sh` reached `GATE GREEN` after the cold mutation cleanup:
  fmt, clippy, workspace tests, all three `sim-core` dependency probes, and metrics-ledger tests
  passed.
- **Self-review pass 1 — 2026-08-18:** `codex review --base 6-2-lanterns-in-the-dark` found one
  actionable P2: a dirty `Empty` tile above the selected slice still spawned `DigChip` debris. Added
  the level guard and the production-path regression above; `cargo test --offline -p gui` then passed.
- **Follow-up gate — 2026-08-18:** after repeating the mutation pass and mandated `cargo clean -p
  gui`, `scripts/gate.sh` again reached `GATE GREEN`.
- **Self-review pass 2 — 2026-08-18:** `codex review --base 6-2-lanterns-in-the-dark` returned
  no actionable correctness issues, so the three-pass cap was not approached.

### Completion Notes List

- Added a client-local `SliceLevel`, defaulted to the snapshot world's top, with provisional
  `<`/`>` controls (comma/period physical keys), explicit world-bound clamps, and no wire activity.
- Terrain filtering is `z <= level && (is_exposed || z == level terrain)`, preserving the existing
  exposure rule and drawing the cut floor. A control change marks the existing snapshot rebuild;
  entities, items, and later dig-chip debris above the level are removed from the projection.
- Added `Slice: z N/top — surface|underground` as an always-on UI readout. The value and its
  surface/underground wording are headlessly tested; legibility and whether the cut reads clearly
  still require the Task 5 vehicle viewing.
- `gui --capture … --z N` is capture-only, pins the client-local level, prints `slice: z N
  projected COUNT terrain cubes` before assertions, and rejects an empty draw count. Existing motion
  and pixel-range checks remain unchanged.
- **Task 5 remains unchecked:** no GPU/native-Windows vehicle session is available to this agent;
  no excavation, FPS, or visual-legibility observation was fabricated. **Task 8 remains unchecked:**
  Wolf's live sign-off is required.
- Scope check: `git diff --stat 6-2-lanterns-in-the-dark..HEAD -- crates/protocol crates/simd
  crates/sim-core crates/client-core crates/tui` is empty.


### Orchestrator verification of the Codex dev run (2026-08-18)

Codex (`gpt-5.6-terra`, reasoning effort **high**, session `01a01541-e593-7210-a7c7-18a3e14a6314`)
exited 0. **Exit 0 was not trusted.**

**Verified GOOD, independently:**

- **No auth failure** — every `401` match in the log is a source line number, not an error.
- **Scope holds exactly (AC17).** `git diff --stat 6-2-lanterns-in-the-dark..HEAD` over `sim-core`,
  `simd`, `protocol`, `client-core`, `tui`, `Cargo.toml` and `Cargo.lock` is **empty**. The slice is
  `gui`-only, which is AC11's client-local requirement proven structurally rather than asserted.
- **`scripts/gate.sh` GREEN** on my own run, and again after the mutation round with
  `cargo clean -p gui` between.
- **Commit cadence MET — 10 commits for 7 dev tasks**, all `Völundr`, nothing pushed. Second story
  running to the floor since it started being asked for in the prompt.
- **Self-gate CONCLUDED in two of three passes** — pass 1 found and fixed floating dig chips above
  the cut (a genuine defect: client-local debris ignoring the slice), pass 2 clean.
- **7/7 mutations KILLED**, run alone, tree clean afterwards — including
  `cut face no longer fills buried terrain`, which is the trap the story was written around, and
  `slice input stops requesting the established rebuild path`.
- **AC13 satisfied:** the draw-set oracle now prints `projected {} terrain cubes at z {}`, so it
  names the level it counted instead of silently changing the pinned figure.

**THE SABOTAGE THAT MATTERED — and this time it held.** Both 6.1 and 6.2 were caught by tests that
hand-fed their inputs, so I attacked the same seam here: is the slice driven through the
**production control path**, or poked into the resource by the tests?

```
slice_controls REMOVED from the live projection tuple  -> 2 headless tests RED  ✅
both `<` / `>` key bindings deleted outright           -> 2 headless tests RED  ✅
```

The tests press real keys through the registered system, so an unbound or unregistered control is a
red suite rather than a silent hole. That is the discipline 6.1's review had to add by hand; it
arrived built-in here. `keyboard_slice_rebuilds_the_cut_face_and_hides_surface_entities` is the test
carrying it.

**Recorded, not a defect:** the oracle line changed from `projected 53365 terrain cubes` to
`projected 53365 terrain cubes at z 31`. That is AC13 working as specified, but any older recipe
grepping the exact former string will no longer match — 5.4's and 6.1's verification blocks quote
it. Anyone re-running those should match the prefix, not the whole line.

**PROVISIONAL AND OWED TO WOLF:** the `<` / `>` binding is **my** call, not his — he was travelling
and did not answer. AC2 requires the ruling recorded, and it is recorded as provisional. It is one
key binding and cheap to reverse at the viewing.

**Still OPEN and not closable by any agent:** Task 5 (the live vehicle session) and Task 8 (Wolf's
sign-off), and with them AC8's dug corridors seen from below, AC9/AC10's legibility — *is the cut
face confusing?* — and AC14's NFR6 reading at a slice level.

### File List

- `_bmad-output/implementation-artifacts/7-1-slice-into-the-mountain.md` — task record, evidence,
  file list, and review status.
- `_bmad-output/implementation-artifacts/mutations/7-1-slice-into-the-mountain.sh` — seven-kill
  sabotage table.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — story tracking to `review`.
- `crates/gui/src/slice.rs` — client-local level, clamp/readout, and unit tests.
- `crates/gui/src/project.rs` — level-aware terrain and dynamic-projection filtering.
- `crates/gui/src/ingest.rs` — keyboard controls, on-screen readout, capture parsing, and schedule
  wiring.
- `crates/gui/src/capture.rs` — level-aware non-zero draw-count oracle.
- `crates/gui/src/lib.rs` — slice module export.
- `crates/gui/tests/headless.rs` — production-path cut-face, clamp, top-level, and visibility tests.
- `docs/tech-art-guidelines.md` — mountain slicing art constraints.

### Task 5 + Task 8 — the live vehicle session (2026-08-20, gingerspice / native Windows / NVIDIA)

Before this session **not one pixel of the slice had ever been seen by a human**. It had been
measured against a live daemon and never watched.

**AC12 — the capture at the dig, `gui.exe 7451 --capture 7-1-slice.png --frames 1500 --z 9`:**

```
projected 36788 terrain cubes at z 9
slice: z 9 projected 36788 terrain cubes (16063 of 16063 cut-face tiles at z 9)
motion: ticks observed=108 dwarf position changes=48 mid-blend frames=521 max working dwarves=0 item count=8
```

The two cut-face figures are counted independently — one from what was drawn, one from what the
mirror says is there — and they match. The cut is filled, not hollow.

**The number is better evidence than the story knew.** The story premeasured **16071**; the vehicle
read **16063 — exactly eight fewer, exactly the eight tiles 6.1 dug.** The story's figure was taken
on an undug world. Nothing in the client knows about a dig-site rectangle, so the cut face can only
have been derived from live world state. That is a stronger result than the equality check itself.

**AC8:** confirmed by eye — the dug tiles read as an excavation seen from inside the mountain.
**AC9:** confirmed — the level readout is legible at the boot framing and clear of the F3 overlay,
which is the review fix of 2026-08-19 verified live. **AC11:** the TUI held z 4 while the Bevy client
sat at z 9 on the same daemon, neither disturbing the other; a uniform field of `#` at z 4 is correct
— worldgen fills every column solid from z 0 to its height, so z 4 is the inside of the mountain.
**AC14:** sustained **>140 fps** at both zooms at a slice level. A transient hitch on level change is
the draw-set rebuild and not a sustained-rate failure; recorded as an observation.

### Two instrument defects the slice exposed, both fixed here

**1. A failing capture destroyed its own evidence.** The first z 9 capture panicked and wrote **no
PNG at all**. `save_to_disk` and `validate_capture_ranges` were two observers on one event; Bevy runs
entity observers for one event in an unspecified order and consistently ran validation first. So the
run whose frame most needed looking at was the one run that produced no frame — the exact inverse of
this instrument's "exit 0 is not a result" rule. Visible in every passing log too, where
`capture range check:` printed *above* `Screenshot saved to`.

Sequenced inside one observer via `save_before_validate`, which exists as its own function so the
ordering is testable at all: the live saver needs a render surface, so a test can only reach it if
the sequence exists apart from the Bevy plumbing. Same mechanism-is-the-requirement justification as
6.1's AC5 and AC6. Confirmed live — the log now reads `Screenshot saved to 7-1-slice.png` **above**
the range check, and the PNG is on disk.

**2. The range band was judging a scene it was never calibrated against.** The z 9 capture read
`ground-median-luminance=67` against a floor of 70 and panicked with "the frame is a black field".
It was not. The floor and ceiling were measured on the approved artifact at the boot framing, and
their own wording says what they watch — *"the valley floor"*, *"night snow stays midtone"*. A cut
removes everything above it, so the sample window stops showing sky-lit snow and starts showing the
interior rock the cut exposes: darker by **material**, not by any light regression. Wolf confirmed by
eye that the z 9 picture reads fine.

This is the same correction the 2026-08-19 review made one assertion higher up — it taught the
lantern checks to ask the mirror whether a dwarf sits at or below the cut, precisely so that *"the
operator merely asked for a lower slice"* could not read as a defect — and then stopped, leaving the
range checks below it unconditional. **The hole relocated one level down**, which is this project's
recorded `verification-defect-relocates` pattern arriving on schedule.

`range_band_applies` now scopes the calibrated band to cuts at the world top. Below it the numbers
still print, with a line naming why they were skipped. `capture is black` and `capture is uniform`
stay unconditional, so a slice capture is never ungated. Full-depth behaviour is byte-identical,
which is what keeps 6.1's AC14 ("5.4's pixel range checks are retained unchanged") true as written.

Confirmed live:

```
capture range check: warm-lit pixels=3645 ground-median-luminance=67
capture range check: the cut at z 9 is below the world top, where 5.4's band was measured on
sky-lit snow - warm and ground assertions skipped
```

### Open, and carried knowingly past sign-off

- **AC10's ruling was never given.** The readout decides `surface` / `underground` by asking whether
  any rock sits above the cut. The known residue stands: at z 30 the world's 17-cube peak still
  counts as "above", so it reads `underground` while the picture is indistinguishable from the
  surface. One line either way, and Wolf did not rule it.
- **The control binding is still PROVISIONAL.** `,` and `.` ship unchosen. The mousewheel was
  confirmed unclaimed — there is no `MouseWheel` handling anywhere in `gui` — and was not claimed
  here. **The next story that wants the wheel inherits the decision rather than finding it made,**
  and claiming it later costs a migration when UX-DR2 brings wheel zoom.

### Sabotage — 16 of 16 KILLED (14 -> 16)

```
a failing range check destroys the frame that would explain it  KILLED
the calibrated band is skipped at full depth too                KILLED
a cut is still judged against the boot-framing band             KILLED
```

The second and third are a pair on purpose: one proves the band still bites at full depth, the other
that it stands aside at a cut. A single row could not tell "correctly scoped" from "switched off".

`scripts/gate.sh` GREEN cold.

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-18 | Orchestrator verification of the Codex dev run. Gate green on my own run, scope exact (`gui`-only, so AC11's client-local rule holds structurally), 10 commits all Völundr, nothing pushed, self-gate concluded in 2 of 3 passes after finding floating dig chips above the cut. 7/7 mutations KILLED including the cut-face trap. **The seam that caught 6.1 and 6.2 held here:** removing `slice_controls` from the live tuple, and deleting both key bindings, each turn 2 headless tests RED — the tests drive the slice through the production control path rather than poking the resource. Recorded: the draw-set oracle now reads `...terrain cubes at z N`, so older recipes quoting the exact old string must match the prefix. The `<`/`>` ruling remains PROVISIONAL — mine, not Wolf's. |
| 2026-08-18 | Story created. **The epic's control-collision premise was falsified against source: `gui` binds no mouse input of any kind and camera zoom sits on `Q`/`E` keys, so the wheel is unclaimed in code** — the collision is planned (UX-DR2 intends the wheel) rather than implemented, and AC3 requires the story to choose against that reality. Identified the cut-face trap: a naive `z <= level` filter over the existing exposure rule yields a hollow shell, because a buried tile is not "exposed" — the `z == level` arm is the whole feature. Flagged that the pinned 53,365-cube draw-set oracle is a full-depth figure that slicing necessarily changes, so the line must name its level or every inherited recipe reads as broken. Raised entity visibility above the slice as a decision to rule and test rather than leave undefined. |
| 2026-08-18 | Implemented the headless slice, capture instrument, always-on level readout, and tech-art rule. `<`/`>` is explicitly **PROVISIONAL (Wolf has not confirmed)**. Seven mutations killed and the gate is green; vehicle-only Task 5 and Wolf-only Task 8 remain open. |
| 2026-08-18 | Self-review pass 1 caught dig chips from a later empty-tile delta floating above the cut. Added the slice-level guard and production-path regression; the repeated mutation run and follow-up gate are green. |
| 2026-08-18 | Self-review pass 2 returned no actionable correctness issues; stopped at two passes. |
