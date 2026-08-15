---
baseline_commit: e1fef5c
model: claude-fable-5  # story-creation ran on the session model Wolf set in the harness; non-default (policy default is Opus), so the exact id is recorded per the model policy
---

# Story 5.4: The Cold Boot

Status: in-progress

## Story

As the boss,
I want the first frame of the Bevy client to be a frozen valley at night — warm camp light
against cold dark, aurora on the horizon,
so that I want to keep looking at it before I have issued a single command.

**This is wow beat 1 (UX-DR13), the story CM2's first-third mandate exists for. The sign-off
gate binds BOTH halves (UX-DR22), and the story CANNOT BE CLOSED BY A DEV AGENT** — Wolf
approves the target artifact before implementation and judges the built result live at the end.

## The sign-off gate — read before touching any code

**Opening half:** no implementation task starts before Wolf has approved one "here is what you
will see" artifact — a generated reference / mock / target frame **of our actual world at the
boot framing being built** (Task 0). This is the structural fix for the FR24/4.1a defect class:
a spec that is meetable, implemented, and not what was wanted, which no review layer can catch
by construction. 4.1a was lost at live viewing, after a full build; the artifact is what makes
that discovery cost one image instead of one story.

**Closing half:** the story is done only when Wolf has viewed the built result **live**,
compared it against the approved artifact, and signed off wow beat 1. A `--capture` PNG serves
the comparison; it never replaces the live viewing (AD-17).

## The live vehicle — where the window actually opens

Carried from 5.3's envelope finding, so nobody re-derives it: **no devpod can open a window**
(no graphics userspace, no display — measured), and the gingerspice **devpod** envelope does
NOT hold even with passthrough (wgpu refuses GL without `/dev/dri`; Dozen Vulkan dies
`DeviceLost` on the full world — both walked to the end at 5.3, AC9 honored). **The proven
vehicle is the native Windows client on gingerspice**: cross-compiled `gui.exe`, `simd` in
WSL, localhost forward, native NVIDIA Vulkan 591.74, **146 fps at 5.3's grey-box fidelity**.
The exact command sequence is in Verification. Everything else in this story — tables, snow
caps, atmosphere spawning, partition, edge-treatment logic — is headless-testable in any
devpod under `MinimalPlugins`; only the live viewing, the NFR6 reading, and the captures need
the vehicle. **Never fake the live half.**

## Acceptance Criteria

### The gate

1. Before any implementation commit, Wolf has approved the sign-off artifact, stored at
   `_bmad-output/implementation-artifacts/5-4-signoff/` beside this story's later captures.

### The boot frame

2. At boot the palette is a dark blue night world — snow, ice, stone, stars — and the camp's
   four torches and campfire read as warm orange pools of light against it (FR32, UX-DR4,
   UX-DR6).
3. The eye lands on the dwarven encampment first, and it lands there because of the warm/cold
   contrast — no UI marker does that work (UX-DR5).
4. Depth reads instantly: light, shadow and air separate near from far (UX-DR16).

### The sky

5. The sky is an illuminant, not a backdrop — aurora and starlight visibly light the snow and
   catch on ice, and the aurora hugs the horizon rather than hanging overhead (UX-DR7).
6. Sky, stars, aurora and falling snow are `ClientLocal` entities with no sim meaning
   (NFR5's carve-out), and a headless test asserts every atmosphere entity carries the
   `ClientLocal` marker — the 5.3 partition extended to the new class, not a parallel
   convention (AD-14, AD-15).

### The terrain read

7. Snow reads as a settled cap — white tops, bare dark flanks, loaded branches, not a uniform
   coat — computed client-side from material + exposure, never from wire state (UX-DR8,
   AD-16). The cap decision is a pure function with headless tests, including a hand-built
   toy world.
8. Blue ice breaks the white expanse so the cold field reads in cold-against-cold layers,
   never one white sheet (UX-DR11).
9. Value discipline: night snow stays midtone blue-grey; only emissive light approaches
   white (UX-DR10). The discipline lives in the data tables (AC13), not in per-site tuning.

### The zoom continuum and the edge

10. Pulled close, individual dwarves and blocks are readable; pulled out to full vista, the
    valley, sky and aurora carry the frame and dwarves become warm specks — the same view at
    a different distance, never a different representation (FR31, UX-DR2).
11. At no zoom is a raw grid edge visible: the world reads as a miniature whose edges
    dissolve into the night. The treatment is chosen **by testing** from the addendum's
    candidates — fog skirt, darkness falloff at the rim, sky wrapping below the horizon,
    vignette — and the story records what was tried and why the winner won (UX-DR12).

    **AMENDMENT (Wolf's ruling, 2026-08-15, review finding `[feature+auditor/MED]`):**
    "chosen **by testing**" is unmeetable headless — an edge treatment is a look, and no
    devpod can see one (seventh instance of the AC-text-defect class). The comparison moves
    to **Task 6's vehicle session**: candidates are implemented headless, then compared by
    eye on the live vehicle, and the winner plus what lost is recorded there. Until that
    session runs, `docs/tech-art-guidelines.md`'s edge section states its choice as a
    *candidate pending vehicle comparison*, never as a settled decision.
12. The vista builds on 5.1's recorded silhouette decision — **YES, shaped there**: surface
    heights span z 10–26 on the shipped seed, pinned by a `sim-core` range test. This story
    does not re-open it; the aurora backlights the skyline that exists.

### Appearance is data

13. Every appearance decision is a data table in `gui`, sibling to `tui`'s `palette.rs`:
    `LightKind` → light properties (RGB, intensity, range), `Material` → base color,
    `EntityKind` → entity appearance. No RGB literal appears at a draw site (AD-16, spine
    convention). Flicker is 6.1's animation; its table column arrives there — a `// NOTE:`
    names the limitation.

### NFR6 — measured where a window exists

14. With the full 128×128×32 world, all dwarves and all lights, the frame-time overlay reads
    a sustained **60 fps at working zoom** and **≥30 fps at full vista** on the live vehicle,
    and the figures are recorded **labelled with their machine** (native Windows client /
    gingerspice RTX 4080). The WSLg-devpod figure NFR6 names is unmeasurable there (5.3's
    envelope finding) and stays formally owed to the Epic 5 retro's bar-redefinition
    question — recorded, never blurred.

### The tech-art guidelines deliverable

15. `docs/tech-art-guidelines.md` exists with the procedural-era half — value discipline,
    sky-as-illuminant, the material rules and light-table semantics this story settles —
    written down **as the decisions are made**, not reconstructed later (spine Deferred).

### The instrument

16. `gui --capture <path> --frames N` at the boot framing produces the closing-half artifact
    with the overlay off (5.3's forcing), and range-checks before any conclusion: the image
    is non-black, non-uniform, and contains a **non-zero count of warm-lit pixels** (warm =
    red channel exceeding blue by a named threshold constant) — exit 0 is not a result
    (AD-17 rung 3). The capture is retained in `5-4-signoff/` beside the approved artifact,
    so the closing comparison is two images, not a memory.
17. **Inherited from 5.3 on Wolf's ruling:** the formal `--ignored` capture self-test
    (`gui`'s `tests/capture.rs`) executes at least once on a display-capable venue —
    candidate mechanism: cross-compile the test executable
    (`cargo test -p gui --test capture --no-run --target x86_64-pc-windows-gnu`) and run it
    on the Windows side with its env vars. If no venue can genuinely run it, the attempt and
    the blocker are recorded — the debt is never quietly re-deferred and never faked.
18. This story's live run is the **first window ever opened on the ramp-complete valley**
    (5.3's AC13 patch landed after its last live run). The startup oracle line must read
    `projected 53365 terrain cubes` on the shipped seed, and the live view shows unbroken
    slopes where `tui --z 9` shows its 444+ `▲` glyphs — confirmed by eye and stated in the
    record.

### The closing half

19. Wolf has viewed the built result live on the vehicle, compared it against the approved
    artifact, and signed off **wow beat 1**: the boot frame is something he would screenshot
    unprompted (UX-DR13, UX-DR15, UX-DR22). A dev agent cannot check this box.

### Evidence

20. A sabotage table exists at
    `_bmad-output/implementation-artifacts/mutations/5-4-the-cold-boot.sh` and every mutation
    is KILLED on a genuine assertion (`mutate.sh` now reports a non-compiling sabotage as
    `NO-COMPILE` and fails), with RED output pasted into the Dev Agent Record.
21. `scripts/gate.sh` is green, and the diff touches only `crates/gui`, `docs/`, and
    implementation-artifacts: **no wire change** (AD-16's sanctioned diff was spent at 5.1),
    no change to `sim-core`, `simd`, `protocol`, `client-core`, or `tui` (nothing sim-side
    changes, so the parity rule's backward half does not fire).

## Tasks / Subtasks

- [x] **Task 0 — The sign-off artifact (Wolf's gate, BLOCKING — no implementation before the
      checkbox)** (AC: 1)
  - [x] Produce one candidate artifact of **our actual world at the boot framing**: use a 5.3
        grey-box capture of the shipped seed (camp framing, focus ~[64,64,9]) as the geometry
        reference and the two concept images (`docs/17d7215b-*.jpg`, `docs/a9d4e72b-*.jpg`)
        as the style register — a generated reference or painted-over mock is fine; the PRD
        asks for cheap, not polished. One artifact, per the PRD's granularity ruling.
  - [x] Wolf approves or iterates. Store the approved artifact in
        `_bmad-output/implementation-artifacts/5-4-signoff/`. Only then does Task 1 start.
        **APPROVED by Wolf 2026-08-15**: `5-4-signoff/candidate-artifact-2026-08-15.png`
        (iterated from the parked 08-14 candidate: draft-2 framing, spruce trees with
        visible trunks per his two directions; FULL wire-true tree density — a thinned
        variant was offered and NOT taken, so no worldgen change rides on this story).
- [x] **Task 1 — The appearance tables** (AC: 13, 9)
  - [x] One module (suggested `crates/gui/src/appearance.rs`), three tables:
        `light_properties(LightKind) -> {color, intensity, range}`,
        `material_color(Material) -> Color` (cold, desaturated; ice visibly blue against
        snow's blue-grey; value discipline as table values),
        `entity_appearance(EntityKind) -> {color, scale}` (dwarves readable at working zoom;
        torch/campfire cubes get emissive warm material).
  - [x] Replace `project.rs`'s eight placeholder greys with table-driven materials — the
        comment at `project.rs:47-48` names this story as exactly this point.
  - [x] Tests pin each table entry with **hand-written literal oracles** (the
        self-referential-oracle trap has fired four times in this project; a test that calls
        the table to check the table proves nothing). Assert the warm/cold invariant as data:
        every `LightKind` color has R > B; every night terrain color has B ≥ R.
- [x] **Task 2 — Night lighting and warm emitters** (AC: 2, 3, 4, 9)
  - [x] Dark cold ambient + `ClearColor` night sky base; a low cool directional fill so
        flanks read (verified in the built tree: `AmbientLight`, `DirectionalLight`,
        `PointLight` in `bevy_light` 0.19; `StandardMaterial::emissive: LinearRgba`).
  - [x] Reconciliation attaches a `PointLight` (from the `LightKind` table) plus emissive
        material to every entity whose mirror state has `light: Some(..)` — driven by the
        wire field through the table, never hardcoded per kind at the draw site. On the
        shipped seed that is 4 torches + 1 campfire at the camp, z 9 (ids 5–9), verified
        live at story-creation.
  - [x] Headless test: projecting the recorded camp snapshot yields exactly 5 entities
        carrying point lights, and their colors match the table's warm side.
- [x] **Task 3 — Snow caps and the cold field** (AC: 7, 8)
  - [x] Pure cap predicate (suggested: a solid tile whose above-neighbour is empty/out of
        bounds gets the cap treatment; foliage reads as loaded branches; flanks keep the bare
        material color). Headless tests on a toy world assert capped vs flank vs enclosed.
  - [x] Wire it through the projection material choice so a capped stone top and a stone
        flank are visibly different — this is what "settled cap, not uniform coat" means in
        cubes.
- [x] **Task 4 — The sky: stars, aurora, illumination** (AC: 5, 6)
  - [x] Hand-rolled `ClientLocal` entities, no new dependencies: a star field, and aurora
        bands as emissive translucent meshes kept **low against the horizon** behind the 5.1
        skyline. (`Skybox` exists in 0.19 but wants a cubemap asset — hand-rolled geometry
        matches the no-asset-pipeline rule.)
  - [x] The illuminant half is an outcome, not a mechanism: cool green-blue light visibly
        catching on snow tops and ice from the aurora side. A tinted directional/ambient
        contribution from the table is the simplest honest candidate.
  - [x] Headless test: every entity spawned by the atmosphere systems carries `ClientLocal`
        and none carries `WorldProjected` (AC6's partition assertion).
- [x] **Task 5 — Snowfall** (AC: 6)
  - [x] Hand-rolled falling flecks (small cubes/quads), `ClientLocal`, respawning above the
        world, drifting down — pure decoration, no sim meaning, sanctioned by NFR5's
        carve-out. Density restrained: it must never obscure the camp read (AC3).
- [ ] **Task 6 — The world edge** (AC: 11)
  - [ ] Try at least two candidates cheaply before choosing: `DistanceFog`/`FogFalloff`
        (present in `bevy_pbr` 0.19 — one component on the camera, doubles as AC4's "air")
        and darkness falloff at the rim (darken material toward the boundary). Sky-wrap and
        vignette remain on the list if both disappoint. Record what was tried and why the
        winner won — the decision is owed on the record, like 7.1's control mechanism.
- [ ] **Task 7 — The boot framing and the continuum** (AC: 10, 12)
  - [ ] The window opens at the approved artifact's framing: camp in frame, warm light
        visible, no input needed. Camera focus [64,64,9] already aims at the camp; adjust
        opening yaw/pitch/distance to the artifact, as constants.
  - [ ] Verify by eye at the vehicle: working zoom readable, full vista carries valley + sky
        + aurora, edges dissolved, silhouette backlit. The zoom clamp (4.0–500.0,
        `camera.rs:32`) already spans both registers.
- [ ] **Task 8 — NFR6 on the live vehicle** (AC: 14)
  - [ ] Read the F3 overlay at working zoom and at full vista with everything lit and
        falling. Record both figures labelled `gingerspice / native Windows / NVIDIA
        591.74`. Baseline for regression sense: 146 fps at 5.3's unlit fidelity.
  - [ ] State plainly in the record that the WSLg figure is still owed and why.
- [ ] **Task 9 — Captures and the inherited AC26 debt** (AC: 16, 17, 18)
  - [ ] Two boot-framing captures at different ticks: range-checks pass (non-black,
        non-uniform, warm pixels > 0), images differ, overlay off. Store the keeper in
        `5-4-signoff/`.
  - [ ] Check the startup line: `projected 53365 terrain cubes`; confirm slopes by eye
        against `tui --frames 6 --z 9` (▲ non-zero) — the ramp-complete valley seen live for
        the first time.
  - [ ] Discharge AC26: cross-compile `tests/capture.rs` (`--no-run`, copy the test exe) and
        run it on the Windows side with `FROSTVEIN_CAPTURE_FIRST/SECOND` set. Record the
        outcome either way — a real blocker is a finding, a fake pass is a firing offence.
- [x] **Task 10 — Tech-art guidelines, procedural-era half** (AC: 15)
  - [x] `docs/tech-art-guidelines.md`: the value ladder (night snow midtone → emissive
        white), sky-as-illuminant rule, material color rules, light-table semantics, edge
        treatment choice. Write each section when its decision lands in Tasks 1–6.
- [ ] **Task 11 — Mutations, gate, closing sign-off** (AC: 19, 20, 21)
  - [x] Sabotage table, minimum set: cap predicate inverted (bare tops) → dies on Task 3's
        toy-world test; a `LightKind` table entry gone cold (R ≤ B) → dies on Task 1's
        warm-side invariant; atmosphere spawn drops `ClientLocal` → dies on Task 4's
        partition test; emitter light attachment ignores the wire's `light` field → dies on
        Task 2's five-lights test. *(5 mutations incl. the added snow-flank one — ALL KILLED
        on the orchestrator's independent full run, exit 0; Codex's own run was sandbox-cut
        at the fifth.)*
  - [x] `scripts/gate.sh` green (headless, any devpod). Run `mutate.sh` alone, never beside
        a gate. *(GATE GREEN on the orchestrator's independent run after all ten commits;
        Codex's own final gate was sandbox-cut during `cargo test` and honestly not claimed.)*
  - [x] Branch `5-4-the-cold-boot`, small commits, imperative messages, author
        `Völundr <jeicei75@gmail.com>`. Push/PR only on Wolf's explicit yes.
  - [ ] Hand the closing half to Wolf: live viewing on the vehicle against the approved
        artifact. **The story's status moves to review/done only through him.**

### Review Findings

Code review 2026-08-15 (fresh session; layers: Blind Hunter + Edge Case Hunter on Sonnet,
Acceptance + Feature Auditors on Opus, all four completed with live execution — zero coverage
holes from kills). Every finding is labelled `[layer/SEVERITY]`. The Feature Auditor's
"capture aborts with zero warm pixels" prediction was CORRECTED at triage against
`bevy_pbr-0.19.0` shader source: `emissive_exposure_weight` defaults to 0.0, so emissive
bypasses exposure — the emitter cubes will read orange and the warm-pixel check passes
(vacuously; see the third decision). Fog, however, DOES apply to `unlit` materials
(`main_pass_post_lighting_processing`, `fog_enabled` default true) — both auditors' fog
arithmetic stands.

**LIVE FALSIFICATION RUN (Wolf, 2026-08-15, native Windows vehicle, same session as the
review):** the four frame-level predictions checked are CONFIRMED BY OBSERVATION — emitters
read as orange dots with no warm pools on the snow; no snowfall visible at the boot framing;
no aurora and no stars visible; the scene is dark enough overall that the ice-vs-snow read
could not be judged. The vista/fog prediction could not be judged: the scene is too dark to
distinguish a fogged-out vista from a merely dark one (fog color is near-black). This makes
the light-scale finding the GATING patch — until the value range exists, no other visual
finding (ice, caps, aurora, fog tuning, framing-vs-artifact) is observable at the vehicle,
so the patch session should land and live-check the lighting rescale FIRST, then judge the
rest. The other four predictions are no longer arithmetic — they are observed. (An initial "crash" report resolved as the deliberate
loud-exit when launched without a reachable daemon — 5.3's disconnect fix working as
designed; started properly, gui runs stably. No stability finding.)

- [x] [Review][Decision→Patch] RULED by Wolf 2026-08-15: cap ramps too. `has_snow_cap`
      requires `Tile::Solid`, leaving the shipped seed's 3,813 exposed ramp tops (1,914 ice /
      1,899 snow) bare inside a fully capped field — slopes are what AC18's first live view
      highlights. Include `Tile::Ramp` tops under the same material-aware rules as solids
      (they already render as full cubes); pin with a toy-world test. Folds into the
      cap-predicate patch below. Three-layer convergence, probed empirically.
      [blind+edge+auditor/MED] [crates/gui/src/project.rs:294]
- [x] [Review][Decision→Patch] RULED by Wolf 2026-08-15: amend AC11 in place + soften the
      doc. AC11's "chosen **by testing**" is unmeetable headless (seventh instance of the
      AC-text-defect class): the comparison moves to Task 6's vehicle session, recorded as an
      amendment beside AC11; `docs/tech-art-guidelines.md:23-28` is reworded to "candidate,
      pending vehicle comparison" so the deliverable stops running ahead of its evidence.
      [feature+auditor/MED]
- [x] [Review][Decision→Patch] RULED by Wolf 2026-08-15: strengthen the AC16 check. The
      warm-pixel check is vacuous for its purpose — emissive emitter cubes bypass exposure
      (verified in shader source), so `warm > 0` passes even if every `PointLight` attachment
      silently failed (broken-instrument class). After the light rescale, raise the check to
      a named warm-pixel-count floor sized above what the emitter-cube faces alone can
      produce, so it detects "lights not attached".
      [feature+auditor/MED] [crates/gui/src/capture.rs:92]
- [x] [Review][Patch] Snow cap is a uniform coat that hides the ice: `has_snow_cap` ignores
      material, capping all 3,650 exposed ice tops snow-white — AC8's "ice breaks the
      expanse" is defeated on the real seed (live census: 16,992 caps = 3,650 ice + 3,817
      snow + 9,525 foliage, zero stone/soil — the toy-world test's stone case never occurs).
      Make the predicate material-aware (ice tops keep ice) per AC7's own "material +
      exposure" wording, and test against seed-shaped material cases. [feature+auditor/HIGH]
      [crates/gui/src/project.rs:294]
- [x] [Review][Patch] Warm pools of light are numerically absent: the light table is ~1/1000
      of Bevy's reference intensity (torch 900 lm → 71.6 cd vs `PointLight` default
      1,000,000 lm; campfire parity with the cold ambient+directional fill at ~0.5 world
      units, exposure-independent). AC2/AC3's warm-against-cold contrast cannot happen.
      Rescale the table lumens, re-balance ambient (110) / directional (250), and encode the
      warm-vs-cold light-budget relationship as a headless test so it is sabotage-able.
      [feature+auditor/HIGH] [crates/gui/src/appearance.rs:32]
- [x] [Review][Patch] Atmosphere is authored around the render origin, not the world/boot
      framing: all 16 snowflakes fall outside the boot frustum (43.8°–54.1° off-axis, over
      the map's far-edge strip); the star field contributes ~1 pixel (half off-map, 8/12
      beyond fog end); aurora bands sit inside the terrain volume at skyline height
      (0.03% of frame, 29–90% fogged, opaque not translucent); the "aurora" directional
      light arrives from the opposite side of the sky. Position everything relative to the
      world footprint/camp focus, aurora behind the 5.1 skyline, translucent; add
      position-pinning headless tests (today no test pins any spatial relationship between
      atmosphere, world, and camera). [feature+auditor/HIGH] [crates/gui/src/atmosphere.rs:38]
- [x] [Review][Patch] Fixed `FogFalloff::Linear { start: 85, end: 180 }` versus the 4–500
      zoom clamp: past ~250 units the whole world is beyond fog end — AC10's full vista is a
      flat sky-colored rectangle (at zoom 500, 100% of terrain pixels ≥95% fogged). Couple
      the fog range to camera distance so both registers survive; the final edge treatment
      remains Task 6's vehicle comparison. [feature+auditor/HIGH]
      [crates/gui/src/ingest.rs:190]
- [x] [Review][Patch] `SnowCap` entities carry neither `WorldProjected` nor `ClientLocal` —
      ~17,000 entities permanently outside the AD-14/15 partition (`classify_client_local`
      runs only at `PostStartup`; caps spawn in `Update`). Spawn them `ClientLocal` (adding
      `WorldProjected` would drag them into entity reconciliation's `Without<TerrainTile>`
      query and get them despawned). [feature+auditor+orchestrator/MED]
      [crates/gui/src/project.rs:284]
- [x] [Review][Patch] AC6's partition test asserts a count threshold, not "every": `>= 20`
      of 31 spawned atmosphere entities tolerates dropping the marker from 11 of them (the
      table's mutation only dies because it hits the 12-star loop; aimed at the 3 aurora
      bands it would survive), and both partition tests assert disjointness, never totality.
      Assert exact counts and totality (including caps). [blind+auditor/MED]
      [crates/gui/tests/headless.rs:505]
- [x] [Review][Patch] The warm/cold invariant is a tautology: `assert!(rgb[0] > rgb[2])`
      tests the test's own literal array, not production values — it can never fail on a
      production change (fifth sighting of the self-referential-oracle class). Assert the
      invariant over production table outputs across all enum variants.
      [auditor/MED] [crates/gui/src/appearance.rs:100]
- [x] [Review][Patch] `night_lighting()` is a fourth appearance table with no literal-oracle
      test and no mutation — the sky/ambient/aurora palette (ClearColor, fog color, ambient,
      directional tint, aurora material) is entirely unguarded. Pin it.
      [auditor/MED] [crates/gui/src/appearance.rs:24]
- [x] [Review][Patch] `Color::WHITE` literal at the star draw site (the one AC13 leak), and
      all three atmosphere materials set `emissive` that `unlit: true` never renders (dead
      code — the stars' actual color is `StandardMaterial::default()` base_color by
      omission). Fold into the atmosphere rework: table-driven `base_color`, drop the dead
      emissive fields. [feature+auditor/LOW — in a function already being patched]
      [crates/gui/src/atmosphere.rs:22]
- [x] [Review][Decision→Defer] RULED by Wolf 2026-08-15: accept cube trees and re-baseline —
      AC19 is judged on light/sky/snow/framing, NOT tree shape; tree presentation deferred to
      a later story. Original finding: trees cannot match the approved artifact:
      `artifact_render.py:7-8`
      draws trees as "snow-laden spruce sprites instead of per-tile boxes" with a visible
      trunk added per Wolf's direction (`:234`), but the wire contains foliage-skirted cube
      stacks (48 foliage vs 6 trunk tiles at camp z 9) and gui renders wire truth. No AC or
      task covers tree presentation, so the AC19 comparison fails on trees by construction —
      the artifact half of the 4.1a class: the sign-off gate approved a stylization nobody
      was tasked to build. Observed live by Wolf 2026-08-15 ("trees not like we planned"),
      confirmed against the artifact script. Options: (a) add client-side tree presentation
      to the patch cycle (expose trunks + taper foliage — touches the draw-set predicate and
      the 53,365 oracle, real scope); (b) accept cube trees for 5.4 and re-baseline the
      artifact expectation, deferring tree presentation to a later story; (c) regenerate and
      re-approve the artifact with wire-true per-tile trees so the sign-off bar is honest.
      RETRO NOTE either way: Task 0 artifact scripts must not substitute geometry the
      renderer is not tasked to produce. [wolf-live+orchestrator/HIGH]
      [_bmad-output/implementation-artifacts/5-4-signoff/artifact_render.py:7]
- [x] [Review][Defer] Entity/item id collision in `reconcile`'s `wanted` map silently erases
      kind, light, and appearance (probed: a campfire sharing an item's id renders as a bare
      stone cube). Unreachable today — `sim-core`'s single `IdAllocator` cannot collide — but
      the invariant is nowhere documented and this diff widened the blast radius.
      [edge/LOW, reachability ruled out by blind] [crates/gui/src/project.rs:219] — deferred
- [x] [Review][Defer] Point lights cast no shadows; AC4's "shadow" is carried entirely by
      the single 250-lux directional. Vehicle-judgment + perf tradeoff, not decidable
      headless. [auditor/LOW] [crates/gui/src/project.rs:118] — deferred
- [x] [Review][Defer] Degenerate-capture diagnostics mislead: an empty pixel buffer reports
      "capture is black", a 1-pixel capture always reports "capture is uniform". Unreachable
      through a real primary window. [edge/LOW] [crates/gui/src/capture.rs:76] — deferred
- [x] [Review][Defer] The live `App` built by `run()` has no test of any kind — every
      headless test builds its own `MinimalPlugins` app, so nothing catches a system dropped
      from the registration tuples or a mis-ordered resource insert. Test-architecture gap,
      pre-existing shape from 5.3. [feature/MED] [crates/gui/src/ingest.rs:81] — deferred
- [x] [Review][Defer] NFR6 headroom is stale: +16,992 cap slabs (+32% draw entities) plus a
      shadow cascade over ~70k meshes versus the 146 fps unlit baseline. Not a defect —
      recorded as Task 8's first suspect if the AC14 reading fails.
      [feature+auditor/LOW] [crates/gui/src/project.rs:284] — deferred

## Dev Notes

### Scope guardrails — do NOT build these here

- **No motion.** Interpolation/blending is 6.1's headline (`Mirror::previous_entity()` stays
  inert one more story — deferred-work.md records it). Dwarves may snap tile-to-tile here.
- **No flicker animation.** Static warm light passes; flicker is 6.1's AC. The table's
  flicker column arrives with it.
- **No lanterns.** `LightKind::Lantern` stays wire-declared and unused until 6.2 (first on
  the cut list). Dwarves carry `light: null` — do not special-case them warm.
- **No z-slicing (7.1), no designation/zone rendering (7.2), no picking or commands (8.x).**
  `gui` still issues zero commands and leaves `designations()`/`zones()` unread.
- **No wire change, no change outside `gui` + docs.** The sanctioned M2 wire diff was spent
  at 5.1. If an AC seems to need one, that is a story-spec defect — raise it, don't code it.
- **No new dependencies, no asset pipeline, no cubemap/skybox assets, no voxel or particle
  crates.** Everything above is hand-rolled geometry + the already-built Bevy features. A
  genuinely needed addition takes one sentence of justification — expected count: zero.
- **No greedy meshing / chunking / LOD.** 146 fps at 5.3 fidelity says the budget exists;
  optimize only if AC14's reading fails, and then the *measured* problem drives it.
- **No workaround for driver/envelope problems in production code** (5.3's AC9 rule stands
  for every live run here).

### What already exists (build on it, do not re-derive)

- **The whole 5.3 substrate:** mirror ingestion (`ingest.rs` — loud exits on disconnect and
  rejected snapshot), reconciliation with `WorldProjected(Id)`/`ClientLocal`/`TerrainTile`
  markers and the ramp-inclusive exposed predicate (`project.rs`), the transform pair
  (`transform.rs`, handedness pinned by a literal oracle), orbit/zoom rig clamped 4–500
  (`camera.rs`), F3 overlay forced off in captures, `--capture <path> --frames N`
  (`capture.rs`), headless suite under `MinimalPlugins`.
- **The wire already carries everything this story lights:** entities expose `kind`
  (`Torch`/`Campfire`/`Dwarf`) and `light: Option<LightKind>` through `client-core`'s
  `entities()`. No mirror change needed.
- **Placeholder seams pointing here:** `project.rs:47-48` ("leaving palette and light
  appearance to story 5.4") and the eight grey materials; entity cubes all use
  `materials[0]`.
- **Verified in the built 0.19 tree at story-creation** (registry source, not memory):
  `bevy_light`'s `PointLight {color, intensity (lumens), range, radius}`, `AmbientLight`,
  `DirectionalLight`; `bevy_pbr`'s `DistanceFog` + `FogFalloff` and
  `StandardMaterial::emissive: LinearRgba`; `bevy_core_pipeline`'s `Skybox` (cubemap-based —
  see Task 4 for why hand-rolled wins). All inside the existing feature trim; nothing new to
  enable.
- **The camp, measured live at story-creation** (shipped seed, `simd 7431`): camp at z 9
  near map centre; 4 torches + 1 campfire, ids 5–9; dwarves ids 0–4; `tui --frames 6 --z 9`
  reads `│=6 ♠=48 †=24 ♨=6 ☺=22 ▲=444`.
- **5.1's skyline:** surface span z 10–26 pinned by a range test; camp clearing is tree-free,
  7×7, flat. The vista has real peaks to backlight.
- **The live-vehicle recipe** is proven end to end in 5.3's Dev Agent Record (cross-compile
  one-liner, no code changes needed).

### Key decisions & traps

- **The gate's opening half is a hard sequence point.** Implementation commits before the
  approved artifact exists repeat the exact failure this gate was built for. Task 0 is
  cheap; 4.1a was not.
- **Appearance through tables is what makes this story testable headless.** The devpod can
  assert every color decision, the partition, the cap predicate and the light attachment
  without a GPU; the vehicle then only has to confirm what tests already pin. Hardcode one
  RGB at a draw site and that assertion surface is gone.
- **Warm/cold is an invariant, not a vibe — encode it as one.** R > B for every light-table
  entry, B ≥ R for every night terrain color. That single pair of assertions makes "the
  warm thing in the cold" sabotage-able (Task 11) instead of subjective.
- **The capture's range checks must survive animation.** Snowfall and aurora are client-local
  and move; two captures at different ticks *should* differ. Threshold counts (warm pixels,
  non-uniformity), never byte-exact comparisons, for every 5.4 check.
- **`Screenshot` timing:** captures spawn in `Update`; use a frame count comfortably past
  the first reconcile (the 5.3 recipe uses 60 — keep it) or the capture races the spawns
  (recorded LOW deferral).
- **Camera focus is hardcoded `[64,64,9]`** (recorded LOW deferral) — correct for the
  shipped seed's camp, so the boot framing can rely on it; do not generalize it here.
- **`simd` has no seed flag** — the seed is `SEED` (`simd/src/main.rs:20`), port positional.
- **`mutate.sh` rewrites source in place** — run it alone. It now fails on `NO-COMPILE`
  sabotages; if a mutation of an *older* story's table ever reports that, it is a shipped
  coverage hole needing its own decision, not a quiet rewrite.
- When torn between simple and general, pick simple and leave a `// NOTE:` naming the
  limitation.

### Project Structure (files to touch)

```
crates/gui/src/appearance.rs        NEW     the three data tables + literal-oracle tests
crates/gui/src/atmosphere.rs        NEW     stars, aurora, snowfall — ClientLocal systems
crates/gui/src/project.rs           UPDATE  table-driven materials, cap predicate, emitter lights
crates/gui/src/ingest.rs            UPDATE  app wiring: lighting resources, fog, atmosphere systems, boot framing
crates/gui/src/camera.rs            UPDATE  opening framing constants only (if needed)
crates/gui/tests/headless.rs        UPDATE  partition, five-lights, cap-predicate app-level checks
crates/gui/tests/capture.rs         UPDATE  warm-pixel range check
docs/tech-art-guidelines.md         NEW     procedural-era half (AC15)
_bmad-output/implementation-artifacts/5-4-signoff/                          NEW  artifact + captures
_bmad-output/implementation-artifacts/mutations/5-4-the-cold-boot.sh        NEW
_bmad-output/implementation-artifacts/deferred-work.md                      UPDATE if anything defers
```

### Previous story intelligence (deltas that change THIS story)

- **5.3's review found the silent-failure class twice** (invisible disconnect, swallowed
  snapshot error) — both now exit loudly. Atmosphere systems must not reintroduce it: a
  failed mesh/asset build is a loud error, never a silently-black sky.
- **The AC-text-defect class is at six instances** (latest: 5.3's vacuous "capture calls the
  transform pair" clause). If an AC here is unmeetable as written, raise it for Wolf's
  ruling and record the amendment in place — the 5.3 pattern, caught at dev, is the good
  outcome.
- **Codex handoff:** check the model banner every run; restate RED evidence across any
  session boundary; commit per green step (the recovery mechanism on a story this size).

### Verification

**Executed at story-creation (the headless half — non-zero evidence, P6 rule):** live
`simd 7431` + `tui --frames 6 --z 9` on the shipped seed produced `†=24 ♨=6` (4 torches + 1
campfire, every frame), `♠=48 │=6 ▲=444 ☺=22` — the emitters this story lights are on the
wire at the level the boot framing aims at. Bevy API surface verified against registry
source (see What already exists).

**Gate (headless, any devpod, must be green before done):**

```bash
scripts/gate.sh
```

**The live vehicle (recipe proven at 5.3; run from the devpod, launch on Windows):**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
# simd stays in WSL:  ./target/debug/simd 7431
# gui.exe runs on the Windows side against localhost:7431
```

**Boot capture (cannot run until the look exists — the obligation the dev agent inherits):**

```bash
gui.exe 7431 --capture frostvein-5-4-boot.png --frames 60
```

**Required non-zero observations:** the range checks of AC16 pass (non-black, non-uniform,
warm-lit pixel count > 0 printed by the check itself); a second capture at a later tick
differs; the startup line reads `projected 53365 terrain cubes`. **Exit 0 is not a result.**

**NFR6 reading (AC14):** F3 overlay on, read sustained fps at working zoom and at full
vista, record both labelled `gingerspice / native Windows / NVIDIA 591.74`.

**Sabotage:**

```bash
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/5-4-the-cold-boot.sh
```

### If this overruns one session

The natural seam is **look core | atmosphere finish**: Tasks 1–3 + 7 (tables, night, lights,
caps, framing — the warm/cold read exists) versus Tasks 4–6 (sky, snowfall, edge). Both
halves are gui-only, so the cut is clean — but the closing sign-off needs ALL bars, so a
split defers the gate, not part of it. Commit per green task; restate RED evidence in any
continuation handoff.

### References

- Story 5.4 epic text — `_bmad-output/planning-artifacts/epics.md:747-794`; Epic 5 rules `:609-619`
- UX-DR1–22 and the anti-requirements — `epics.md:149-194`; PRD Visual Target —
  `prds/prd-frostvein-2026-08-09/prd.md:37-131`
- Edge-treatment and silhouette context — `prds/prd-frostvein-2026-08-09/addendum.md:21-38`
- AD-14…AD-18, conventions, NFR6 —
  `architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:92-198`, `:244-253`
- Tech-art deliverable owed — `ARCHITECTURE-SPINE.md:274-280`
- 5.1's silhouette ACs — `_bmad-output/implementation-artifacts/5-1-the-world-grows-things-that-glow.md:18-58`
- 5.3's envelope finding, live-vehicle recipe, AC26 ruling —
  `5-3-a-window-onto-the-valley.md` (Dev Agent Record + Review Findings)
- The placeholder seam — `crates/gui/src/project.rs:47-52`; camera clamp — `crates/gui/src/camera.rs:32`
- Concept references (guidance, not bars) — `docs/17d7215b-6c05-4286-b3bb-56592ca617ec.jpg`,
  `docs/a9d4e72b-b4c3-43f2-8a1c-e25c539fd6c1.jpg`, `docs/narrative.md`
- Story rules, instrument rule, exit-0 — `docs/technical-preferences.md:64-101`

## Dev Agent Record

### Agent Model Used

gpt-5.6-terra

### Debug Log References

- Task 1 RED: `cargo test -p gui appearance_tables_pin_the_cold_boot_palette --offline`
  failed before table implementation: `left: [0, 0, 0]`, `right: [255, 140, 62]`
  at `crates/gui/src/appearance.rs:52`.
- Task 1 sabotage RED: changing the Torch table colour to `[62, 140, 255]` made the
  same literal-oracle test fail: `left: [62, 140, 255]`, `right: [255, 140, 62]`
  at `crates/gui/src/appearance.rs:82`. Restored before the green run.
- Task 2 RED: `cargo test -p gui recorded_camp_snapshot_projects_exactly_five_warm_point_lights --offline`
  failed before attachment: `left: 0`, `right: 5` at `crates/gui/tests/headless.rs:432`.
- Task 3 RED: the snow-cap test initially failed to compile because `has_snow_cap` did not
  exist; implementation then passed its hand-built toy-world assertions.
- Tasks 4–5 RED: `atmosphere_entities_are_client_local_and_never_world_projected` failed
  before spawning with `stars, aurora, and restrained snow must be present`.
- Task 11 mutation RED: `scripts/mutate.sh .../5-4-the-cold-boot.sh` killed all four:
  cap predicate (`project.rs:402`), cold torch table (`appearance.rs:97`), dropped atmosphere
  marker (`headless.rs:457`), and ignored wire light (`headless.rs:433`, `left: 0`, `right: 5`).
- Review-fix RED — capture: `cargo test -p gui bgra_capture_bytes_decode_before_warm_pixel_detection --offline`
  failed before the decoder existed with `error[E0425]: cannot find function decode_rgba8` at
  `crates/gui/src/capture.rs:88`; after decoding `Bgra8*` to RGBA, the hand-written oracle
  `[10, 120, 240, 255] -> [240, 120, 10, 255]` passed.
- Review-fix RED — items: `cargo test -p gui --test headless snapshot_item_receives_a_render_mesh --offline`
  failed with `a projected item must carry a mesh` at `crates/gui/tests/headless.rs:462`.
- Review-fix RED — snow flanks: `cargo test -p gui --test headless capped_stone_keeps_its_bare_cube_beneath_a_snow_cap --offline`
  failed with `left: [[118, 139, 157]]`, `right: [[40, 57, 82], [118, 139, 157]]` at
  `crates/gui/tests/headless.rs:494`; the bare stone cube was incorrectly snow material.
- Review-fix mutation run: the runner killed the cap predicate, new snow-flank, cold-torch,
  and atmosphere-marker mutations. Its sandbox execution window cut off while the fifth
  mutation was running; running that exact emitter sabotage independently produced
  `left: 0`, `right: 5` at `headless.rs:435`. Source was restored and the headless GUI suite
  passed afterwards.
- Review-patch RED — light budget: `appearance_tables_pin_the_cold_boot_palette` initially
  failed to compile because `NightLighting` lacked `ambient_brightness` and
  `directional_illuminance` (E0609); after adding the production fields the literal budget
  oracle passed. Registry sources read: `bevy_light-0.19.0/src/{point_light,ambient_light,
  directional_light}.rs`, `bevy_light-0.19.0/src/lib.rs`, `bevy_camera-0.19.0/src/camera.rs`,
  and `bevy_pbr-0.19.0/src/pbr_material.rs`.
- Review-patch RED — caps: `snow_caps_follow_material_and_exposure_in_a_seed_shaped_toy_world`
  failed at `exposed ice keeps its blue top` before the material-aware ramp-inclusive predicate.
- Review-patch RED — atmosphere: mutating the first aurora z to `-120.0` failed the position
  test with `aurora belongs beyond the far edge`.
- Review-patch RED — fog: the initial 180-unit boot fog end failed `the far terrain edge must
  remain before complete fog`; the range is now `(85,190)` at boot and `(470,900)` at full zoom.
- Review-patch RED — caps partition: `capped_stone_keeps_its_bare_cube_beneath_a_snow_cap`
  failed with `snow caps belong to the client-local partition` before spawning `ClientLocal`.
- REVIEW-PATCH round 2 — R1: chose shape (a), retaining Bevy's default `Exposure::BLENDER`
  (`exp2(-9.7) / 1.2 = 1.017e-3`) and lifting the cold fill. For snow's blue channel,
  `0.337 * 0.140 * 2,000 = 94.4 cd/m²`, or `0.0960` post-exposure; the directional term
  scales from the verified 8-lux `0.06 cd/m²` estimate to `11.25 cd/m²`, or `0.0114`
  post-exposure. Total `0.1074` linear is about `0.36` sRGB: midtone, not black. The 6M-lm
  campfire at six units is `6,000,000 / (4π * 36) = 13,263 lux`; against the 3,500-unit
  cold-fill budget it is `3.79x`, above the production-table `3x` local-contrast oracle.
  The old `60x` assertion was removed; mutating the campfire to 5,000 lm targets that
  relationship test directly.
- REVIEW-PATCH round 2 — R2: deleted the literal-laundered capture floor assertion. The
  source-face estimate remains an explicitly vehicle-pending comment, not false test evidence.
- REVIEW-PATCH round 2 — R3/R5: added the aurora-position mutation (`z = -120`); narrowed the
  ramp mutation anchor to the `has_snow_cap` predicate so it is unique again. The existing
  manual aurora mutation had produced `aurora belongs beyond the far edge`.
- REVIEW-PATCH round 2 — R4: `fall_snow` now resets below the named 9.0 render-space camp
  surface instead of below terrain at -2.0.
- REVIEW-PATCH round 2 validation blocker: `mutate.sh` was invoked alone twice from a clean
  tree, but this environment kills its parent command at roughly 30 seconds while child tests
  continue. It therefore left temporary mutation edits behind; each was inspected and restored.
  A complete clean exit-0/KILLED transcript could not be observed here. The subsequent gate
  invocation started while a detached mutation child had dirtied source, then was cut off after
  fmt and during clippy; it is not validation of the clean tree, so no green gate is claimed.
- REVIEW-PATCH round 2 self-gate: `codex review --base main` was launched once (session
  `01a005d7-1804-7751-80fb-ee2402e8db22`), but the same command-parent timeout truncated it
  during diff inspection, before findings or a verdict. No review finding was withheld.
- REVIEW-PATCH round 3 — T1/T2: `TreeFoliage` now receives no terrain snow slab, and its cube
  scale is a pure contiguous-foliage-above rule: skirt/tip `0.72`, upper crown `0.86`, mid crown
  `1.0`. This is deliberately a scale rule, not mesh generation: it exposes the terrain's own
  snow landform while keeping the wire-true cube tree draw set. `is_exposed` and
  `terrain_positions()` are untouched; the latter still calls only that predicate, so the
  previously live-confirmed `projected 53365 terrain cubes` oracle is statically preserved.
- REVIEW-PATCH round 3 — T3/T4: snowfall is a visible 6×6 grid (36 flakes at scale `0.28`) with
  independent x/z axes. The FPS text and its frame-time graph are both disabled for normal boot
  and capture configuration, and toggled together by F3.
- REVIEW-PATCH round 3 — T5/T6: aurora and stars are now just beyond the terrain edge and low
  inside the production 45° boot frustum; the test builds `CameraRig::new([64, 64, 9])` and
  checks camera basis, vertical and 16:9 horizontal bounds. `WARM_PIXEL_FLOOR` is 3,000, derived
  from the measured 17,648 warm pixels in `capture-2026-08-15T1717-boot.png` rather than the
  obsolete 100/64 estimates. No tautological test was added for that measured threshold.
- REVIEW-PATCH round 3 RED evidence: removing the foliage exclusion failed
  `snow_caps_follow_material_and_exposure_in_a_seed_shaped_toy_world` at `foliage stays dark so
  ground-level skirts cannot read as snow slabs`; changing the taper's `0 => 0.72` to `1.0`
  failed `foliage_tapers_from_wide_mid_crown_to_narrow_tip_and_skirt` (`left: 1.0`,
  `right: 0.72`, `skirt`); correlating snowfall z with x failed `a shared x column must span
  multiple z rows`; removing the capture graph disable failed `assertion failed:
  !app.world().resource::<FpsOverlayConfig>().frame_time_graph_config.enabled`; raising the first
  aurora to y=35 failed `aurora must be visible at the boot framing`.
- REVIEW-PATCH round 3 mutation RED — actual runner output:

  ```text
  snow cap leaves bare top                                     KILLED
  ice tops receive snow                                        KILLED
  foliage receives ground snow slabs                           KILLED
  spruce skirt loses its taper                                 KILLED
  ramps lose their caps                                        KILLED
  snow cap paints stone flanks                                 KILLED
  torch table goes cold                                        KILLED
  light budget collapses                                       KILLED
  night lighting goes unpinned                                 KILLED
  aurora leaves the boot frustum                               KILLED
  snowfall collapses back to a diagonal                        KILLED
  atmosphere loses client local marker                         KILLED
  snow caps lose client local marker                           KILLED
  fog stops following zoom                                     KILLED
  capture keeps its frame graph                                KILLED
  emitters ignore wire light field                             KILLED
  All mutations killed.
  ```

### Completion Notes List

- Implemented the headless-testable look core: table-driven cold materials, warm wire-driven
  emitters, snow caps, client-local atmosphere/snow, fog, boot constants, and capture warm-pixel
  threshold. Vehicle-only checks remain open.
- Not run in this devpod: Task 7 by-eye framing/zoom, Task 8 NFR6, Task 9 captures/AC26
  Windows execution, and Task 11's Wolf closing sign-off. Task 6's fog candidate is implemented;
  its required live comparison against rim darkening remains vehicle work.
- Review-fix round: capture range checks now decode RGBA/BGRA by the screenshot texture format;
  projected items receive the shared stone cube material; capped terrain keeps its bare cube and
  receives a separate thin snow-cap slab. The required self-gate pass was launched after these
  commits but its review session was cut off before reaching findings or a verdict; pass 1 found
  these three defects and pass 2 was likewise cut off before conclusion. No further self-gate
  pass was run, preserving the three-pass cap.
- The full `scripts/gate.sh` could not finish before this sandbox's command session was cut off
  during `cargo test`. Its first run did expose `clippy::too_many_arguments` in the snow-cap
  cleanup; `f33f3bf` replaced the extra query with a named combined query, then GUI headless
  tests and GUI clippy passed. The final gate tail was `cargo fmt --check ok`, `cargo clippy
  -D warnings ok`, then `cargo test` before sandbox termination. The gate checkbox remains
  unchecked and no green gate is claimed.
- **Orchestrator verification (2026-08-15, after all ten commits):** `scripts/gate.sh` GREEN
  on an independent run (fmt, clippy, full workspace tests, all three dependency probes,
  metrics ledger); `mutate.sh` full table run alone — all 5 mutations KILLED, exit 0, source
  restored. Both Task 11 sub-boxes checked on this evidence, not on Codex's cut-off runs.
- **Process findings for the review's attention:** (1) the commit-cadence hard floor was
  violated in the first session — Tasks 1–5 + 10 landed in effectively two implementation
  commits (`cd289b6`, `eccb621`); the continuation session then held the floor (one commit
  per fix). (2) `cd289b6`'s message ("Record approved cold boot sign-off artifact") is wrong
  for its content — it carries Task 1's appearance-table code; a staged-files retry under a
  stale message. (3) The first session's handback claimed completion without surfacing its
  own self-gate pass-1 findings; all three were real, verified in-tree, and fixed only in
  the continuation (`db54e77`, `ae696f5`, `31e60a1`).
- Review-patch session: rescaled torch/campfire/lantern to 250,000/500,000/180,000 lm with
  an 8 cd/m² + 8 lux cold fill; moved stars, translucent aurora, and snowfall around the
  camp/world footprint; material-aware caps preserve ice and include ramps; fog tracks zoom;
  caps and all 31 atmosphere entities are partitioned ClientLocal. Vehicle-only Tasks 6–9
  remain open. The capture floor is 100 warm pixels: a conservative floor above the estimated
  64 pixels from the five emitter faces at distance 90; it still needs native-Windows confirmation.
- REVIEW-PATCH round 2: corrected the default-exposure light budget with shape (a): 2,000
  ambient brightness, 1,500 directional lux, and 2.5M/6M/2M lm torch/campfire/lantern table
  entries. The arithmetic above predicts snow at 0.1074 linear (~0.36 sRGB) and campfire
  local contrast of 3.79x at six units. Deleted the tautological capture test, made snow
  respawn above the camp surface, and repaired the atmosphere/ramp sabotage coverage. The
  mutation-table and gate runs are environmentally incomplete; no success claim is made.
  `cargo test -p gui --offline` did complete green (18 library, 1 capture, and 11 headless
  tests; one display-required capture test ignored). The one required Codex review was started
  but cut off before reporting findings.
- **Orchestrator verification of the review-patch session (2026-08-15, after both rounds and
  all 20 commits).** Codex's own gate and mutation runs were sandbox-cut and it correctly
  claimed neither; both were re-run independently here from a clean tree:
  - `scripts/gate.sh` **GREEN**, exit 0 (fmt, clippy `-D warnings`, full workspace tests, all
    three dependency-edge probes, metrics ledger).
  - `scripts/mutate.sh .../5-4-the-cold-boot.sh` run **alone**: **all 12 mutations KILLED,
    exit 0**, source restored, tree clean. The table now covers the material-aware cap, ramp
    caps, the light-contrast *relationship*, the `night_lighting` table, aurora position, both
    `ClientLocal` partitions, and fog-follows-zoom.
  - Round 1's own run had exited **1** on `ramps lose their caps` reporting `APPLY-FAILED`
    (its anchor matched twice in `project.rs` once the predicate changed); round 2 narrowed the
    anchor and it now kills. *Orchestrator note on its own method: that first run was piped
    through `tail`, so the shell reported `tail`'s exit 0 and masked the runner's 1 — the
    failure was caught by reading the printed table, not the status. Capture the exit code
    before any pipe.*
  - Git state verified: 20 commits across the two rounds, every one authored
    `Völundr <jeicei75@gmail.com>`, working tree clean, and the diff confined to `crates/gui`,
    `docs/`, and implementation-artifacts — no wire change, nothing touched in `sim-core`,
    `simd`, `protocol`, `client-core`, or `tui` (AC21's scope half holds).
  - **Self-gate: zero usable `codex review --base main` passes.** One launch per round, both
    truncated by the sandbox command-parent timeout before findings. Under the three-pass cap
    that is permitted, but it means this patch cycle carries **no self-gate evidence** — the
    orchestrator gate/mutation runs and the next code review are the only checks on it.
  - **The R1 light values are arithmetic, not observation.** Snow's predicted 0.1074 linear
    (~0.36 sRGB) and the campfire's 3.79× local contrast were derived against the built 0.19
    tree (`bevy_camera-0.19.0/src/camera.rs:263,274` for the default `Exposure::BLENDER`
    ev100 9.7 → 1.017e-3; `bevy_pbr-0.19.0/src/render/pbr_functions.wgsl:863` for the exposure
    multiply; `pbr_ambient.wgsl:28` for the ambient term). No devpod can see whether they are
    *right* — only that they are no longer numerically impossible. **Task 7's live check is
    what settles them**, and it is the first thing to look at on the vehicle.
- **LIVE CHECK OF THE GATING PATCH (Wolf, 2026-08-15, native Windows vehicle, ~16:50).** The
  rescaled frame was viewed live and the look has visibly changed: Wolf's verdict is
  *"works now — well it's a start"*. The R1 light budget is therefore **live-confirmed as no
  longer the blocker**; the four frame-level HIGHs are no longer masked by an unreadable scene.
  **This is NOT AC19.** "A start" is not "something he would screenshot unprompted" — wow beat 1
  remains unsigned, and Tasks 6–9 remain open.
  - **Trap that cost a live run, recorded so nobody repeats it:** the first attempt at this
    check showed *no change at all*. Cause: the vehicle's cross-compiled `gui.exe` was built at
    **13:24**, while the earliest patch commit (`c69ffc4`) landed at **13:58** — the binary
    predated all 20 patch commits, so the frame was byte-identical to the falsification run.
    The cross-compile is a manual step in the vehicle recipe and **nothing in the delegated dev
    flow triggers it**. Any handback that ends "go look at it on the vehicle" must name the
    rebuild + re-copy as an explicit step, and the running build should be confirmed against
    the last commit time before any visual conclusion is drawn. Same family as the exit-0 rule:
    an unchanged frame is not evidence about the code until the binary is known to contain it.
- **FIRST REAL BOOT CAPTURE (Wolf, 2026-08-15 ~17:17, native Windows vehicle,
  `docs/frostvein-5-4-boot.png`, 1280×720).** Non-zero evidence, all of it:
  - `projected 53365 terrain cubes` — **AC18's oracle line confirmed live**, first window ever
    opened on the ramp-complete valley.
  - `capture range check: warm-lit pixels=17648` — AC16's range checks ran and passed.
    17,648 of 921,600 px = 1.9% of frame. **The warm pools are real and measured.**
  - Confirmed by eye: the warm/cold contrast works, the eye lands on the camp on light alone
    (AC3), and the field is a genuine midtone blue-grey rather than black (AC9). **The R1
    light rescale is validated at the vehicle.**
  - **The measurement makes the AC16 floor obsolete:** 17,648 with lights attached versus an
    estimated ~64 from emitter faces alone means the `WARM_PIXEL_FLOOR = 100` cannot detect
    "lights silently not attached" with any margin. Raise it against the measured value.
- **RULING REVERSED — trees and landform are back IN SCOPE (Wolf, 2026-08-15, on the capture):**
  *"let's fix trees and valley landform still."* This supersedes the morning's
  accept-cube-trees-and-defer ruling, which was made against the artifact script **before**
  anyone had seen the composed frame. The two decisions turned out to be one: with wire-true
  tree density the valley reads as a repeating waffle with no silhouette, no slope and no
  landform, so AC10/AC12/AC18's vista has nothing to carry it.
  - **Root cause, traced to worldgen (`crates/sim-core/src/worldgen.rs:206-215`):** every tree
    lays a **3×3 foliage skirt flat on the ground** around its trunk. `has_snow_cap` treats
    `TreeFoliage` exactly like terrain, so ~9,525 ground-level skirt tiles each receive a full
    bright snow slab. The repeating lump-with-a-dark-notch is a capped skirt around a dark
    trunk. The landform is buried under them. This is the very thing AC7's "loaded branches,
    **not a uniform coat**" forbids — the cap predicate met the letter and inverted the intent.
  - **Fixable entirely client-side.** Trunk tiles above the skirt are already exposed and drawn,
    so the fix does not touch `is_exposed` and **the 53,365 oracle holds**. No wire change.

- REVIEW-PATCH round 3 landed six focused visual/instrument patches in six commits. The foliage
  rule is intentionally minimal: uncapped, reduced cube branches with a 0.72/0.86/1.0 taper; no
  mesh, LOD, or draw-set change. The complete 16-entry mutation table ran alone and every
  mutation was KILLED. Vehicle viewing is still required to judge the revised trees, snowfall,
  and in-frame sky; no Tasks 6–9 or Wolf's closing sign-off box were changed.

### File List

- crates/gui/src/appearance.rs
- crates/gui/src/atmosphere.rs
- crates/gui/src/camera.rs
- crates/gui/src/capture.rs
- crates/gui/src/ingest.rs
- crates/gui/src/lib.rs
- crates/gui/src/project.rs
- crates/gui/tests/capture.rs
- crates/gui/tests/headless.rs
- docs/tech-art-guidelines.md
- _bmad-output/implementation-artifacts/mutations/5-4-the-cold-boot.sh
- _bmad-output/implementation-artifacts/5-4-signoff/README.md
- _bmad-output/implementation-artifacts/5-4-signoff/artifact_render.py
- _bmad-output/implementation-artifacts/5-4-signoff/candidate-artifact-2026-08-14.png
- _bmad-output/implementation-artifacts/5-4-signoff/candidate-artifact-2026-08-15.png
- _bmad-output/implementation-artifacts/5-4-signoff/capture_snapshot.py
- _bmad-output/implementation-artifacts/5-4-the-cold-boot.md

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-14 | Story created. Camp/emitter baseline measured live (`†=24 ♨=6` at z 9); Bevy 0.19 lighting/fog/emissive API verified against registry source; NFR6 venue corrected on the record to the proven native-Windows vehicle (5.3's envelope finding), WSLg figure kept owed; 5.3's AC26 debt and the ramp-complete first-live-view folded in as ACs 17–18. Sign-off gate encoded as blocking Task 0 + closing AC19. |
| 2026-08-15 | Implemented the headless cold-boot look core and its table, cap, emitter, and atmosphere assertions; live-vehicle verification remains open. |
| 2026-08-15 | Addressed self-gate pass 1: decode BGRA capture pixels, restore projected item rendering, and render snow as a top slab over bare terrain; added the snow-flank mutation. |
| 2026-08-15 | Orchestrator verified independently (gate GREEN, 5/5 mutations killed); Status → review. Vehicle-bound tasks and Wolf's AC19 closing sign-off remain open — review does not close this story. |
| 2026-08-15 | Code review (fresh session, 4 layers, all completed live): 3 decisions ruled by Wolf, 11 patches recorded as action items for a fresh patch session, 5 defers to deferred-work.md, 6 dismissed. Headless substrate verified solid; six frame-level HIGHs predict the composed look fails the artifact comparison (lights ~1/1000 scale, atmosphere authored at render origin, fixed fog vs zoom clamp, uniform ice-hiding cap). Status → in-progress. |
| 2026-08-15 | Landed the 11 review patches in focused commits; live vehicle tasks remain explicitly open. |
| 2026-08-15 | REVIEW-PATCH round 2: rebalanced the cold field for default exposure, removed the tautological capture assertion, added/fixed mutations, and raised snow's respawn floor. Full mutation and gate completion remain blocked by the sandbox command-parent timeout. |
| 2026-08-15 | Orchestrator verified the whole review-patch cycle independently: gate GREEN (exit 0) and all 12 mutations KILLED (exit 0) from a clean tree; scope, authorship and the 20-commit cadence confirmed. All 11 review action items closed. Zero usable self-gate passes (both truncated). Status stays **in-progress**: Tasks 6–9 are vehicle-bound and AC19 is Wolf's alone. |
| 2026-08-15 | REVIEW-PATCH round 3: stopped foliage snow slabs, tapered spruce cubes, spread visible snowfall, disabled the frame graph, frustum-pinned the aurora/stars, and raised the measured warm-pixel floor. All 16 mutations were KILLED; the final gate and self-gate follow. |
