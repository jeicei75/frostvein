---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: 538e1f8
---

# Story 6.2: Lanterns in the Dark

Status: done

<!-- FIRST ITEM ON THE M2 CUT LIST. If the story cap binds, this is what goes — Epic 6 keeps its
     wow because torches and the campfire already carry the warm/cold read. Do not treat that as
     licence to thin it: while it is in the plan it ships whole. -->

## Story

As the boss,
I want each dwarf to carry a lantern whose warm light travels with them,
so that the dwarves are the warm thing moving through the cold, and the lighting system is proven
on its hardest case — a light that moves.

## The sign-off gate — read before touching any code

**Opening half (Task 0, BLOCKING):** no implementation commit before Wolf has approved one "here is
what you will see" artifact, stored at `_bmad-output/implementation-artifacts/6-2-signoff/`. Three
parts: (a) a before capture of the camp at the boot framing from the **shipped 6.1 binary**, (b) the
written list of what this story adds, (c) an explicit **"what you will NOT see"** list. Part (c) is
not optional — 5.4's artifact drew geometry the renderer was never tasked to produce, and 6.1's
artifact under-stated what would read at the boot vista.

**Closing half (AC17):** done only when Wolf has viewed the built result live on the vehicle and
signed off. A capture serves the comparison; it never replaces the live viewing (AD-17). *(This
line read "AC14" until the 2026-08-19 review: AC14 is the 5.4 capture range checks, a separate open
obligation, and filing both under one number hid one of them.)*

## The live vehicle — unchanged, do not re-derive

**No devpod can open a window** (no graphics userspace, measured at 5.3). The proven vehicle is the
native Windows client on **gingerspice**: cross-compiled `gui.exe`, `simd` in WSL, localhost, native
NVIDIA Vulkan. 6.1 measured **>143 fps at both working zoom and full vista** there with all lights
flickering, so this story starts with large headroom over NFR6. Everything except the live viewing,
the NFR6 reading and the captures is headless-testable in any devpod under `MinimalPlugins`.
**Never fake the live half.**

## Acceptance Criteria

### The gate

1. Before any implementation commit, Wolf has approved the sign-off artifact at
   `_bmad-output/implementation-artifacts/6-2-signoff/`.

### The lantern reaches the wire

2. Every dwarf on the wire carries `light: Some(LightKind::Lantern)`, in **both** the snapshot and
   the delta paths — today both hardcode `light: None`
   (`crates/simd/src/bridge.rs:27`, `:82`). No dwarf is ever emitted without it, and no dwarf is
   emitted twice.
3. **No protocol change.** `protocol::LightKind::Lantern` and `Entity.light` already exist
   (`crates/protocol/src/lib.rs:46-50`, `:107`), so the diff adds no wire variant and no field:
   `git diff --stat main..HEAD -- crates/protocol` is empty. *(Mechanism is the requirement: AD-16's
   sanctioned M2 wire diff was already spent, and the epic's claim that this story spends "the last
   piece" of it is stale — see Dev Notes.)*
4. The lantern is a uniform property of being a dwarf — **no fuel, no pickup, no drop, no economy,
   and nothing per-dwarf to persist.** A save round-trip produces the same lantern state without
   `SaveState` gaining a lantern field (FR29, AD-16).
5. Same seed and command sequence → identical lantern state on every run, asserted by a scenario
   test like any other world state (NFR7).

### The two existing lantern guards stay honest

6. `emitter_entity`'s `unreachable!("lanterns are not live emitters")`
   (`crates/simd/src/bridge.rs:148`) is still true after this story: dwarf lanterns do **not** flow
   through the static-emitter path, and a test proves a dwarf lantern cannot reach it. Either the
   guard remains valid and is left alone, or it is removed *and* the reason recorded — silently
   leaving a reachable `unreachable!` is a defect.
7. `load_world_from`'s rejection of lantern emitters (`crates/simd/src/main.rs:501-503`) and its test
   `loading_rejects_lantern_emitters_before_the_wire_bridge` are re-ruled explicitly: either they
   stay correct (a *static* lantern emitter in a save is still bogus) with the test renamed to say
   so, or they go with the reason recorded. **Do not leave a test whose name asserts a state of the
   world that this story just changed.**

### The TUI

8. `tui` needs no rendering change, and **that reasoning is recorded in the story rather than
   assumed**: every dwarf carries one uniformly, so a lantern glyph distinguishes nothing. The field
   still flows through `client-core`'s mirror to both clients unchanged (parity rule).

### The moving light

9. A dwarf's lantern renders as a warm pool that **travels with the dwarf**, lighting the terrain it
   passes (FR29, FR32).
10. The lantern's appearance comes from the `gui` table keyed by `LightKind`
    (`crates/gui/src/appearance.rs:71-77`), never from the wire and never hardcoded at a draw site
    (AD-16). Dwarves are **not** special-cased warm anywhere in `gui`.
11. The lantern pool follows the **blended** transform, not the snapped wire position: at a
    mid-blend clock the light's translation equals the dwarf's rendered translation, so the pool
    slides with the dwarf rather than jumping tile to tile (AD-15, interaction with 6.1).
12. AC9–AC11 are asserted headlessly under `MinimalPlugins` in `cargo test`, driving the same
    `projection_systems` registration the live `App` uses (AD-17 rung 2).

### Measured where a window exists

13. With all five dwarves carrying moving lights plus every static emitter, the frame-time overlay
    reads a sustained **60 fps at working zoom** and **≥30 fps at full vista** on the live vehicle,
    recorded **labelled with its machine**. If a reading fails, that measurement is the story's
    finding and is reported (NFR6).
14. **The 5.4 capture range checks still pass**, re-measured with lanterns lit: warm-lit pixels
    ≥ `WARM_PIXEL_FLOOR` and ground-median luminance inside `[GROUND_LUMINANCE_FLOOR,
    GROUND_LUMINANCE_CEILING]` = `[70, 180]` (`crates/gui/src/capture.rs`). *(This is the story's
    real look risk, not NFR6 — see Dev Notes. A ceiling breach is a finding to report with the
    lantern intensity named as the knob, never a silently widened band.)*

### The instrument

15. `gui <port> --capture <path> --frames N` accumulates across the run and prints one lantern line
    before any conclusion — the dwarf positions observed and the count of **lit terrain tiles at the
    dwarf's successive positions** — and **asserts** that count is non-zero and that the lit region
    **moved** between the first and last observation. The 6.1 motion line and the 5.4 range checks
    are retained unchanged. **Exit 0 is not a result** (AD-17 rung 3).
16. The instrument has its own test, driving a hand-built sequence of mirror states: a world whose
    dwarves never move produces no lit-region movement and **fails**; a moving one passes.

### The closing half

17. Wolf has viewed the built result live on the vehicle, compared it against the approved artifact,
    and signed off. **A dev agent cannot check this box.**

### Evidence

18. A sabotage table exists at
    `_bmad-output/implementation-artifacts/mutations/6-2-lanterns-in-the-dark.sh` and every mutation
    is KILLED on a genuine assertion, with the RED table pasted into the Dev Agent Record.
19. `scripts/gate.sh` is green and the diff touches only `crates/sim-core`, `crates/simd`,
    `crates/gui`, `docs/` and implementation-artifacts: **no `crates/protocol` change**, and no
    change to `client-core` or `tui`.

## Tasks / Subtasks

- [x] **Task 0 — The sign-off artifact (Wolf's gate)** (AC: 1) — **GATE OPENED 2026-08-18 ON THE
      WRITTEN-ONLY FALLBACK.** Wolf, travelling and away from the vehicle: *"Well let's Start dev
      then"*. Recorded honestly rather than upgraded: he approved **proceeding**, not the artifact
      line by line, and the before-capture was **not taken** because no vehicle session was
      available. Task 0's own fallback clause covers exactly this — the written
      `what-you-will-see.md` approved on its own, with the skipped pair and the reason recorded.
      **AC1 MET on that basis. The consequences stay open, not blessed:** the ground-luminance
      ceiling risk (line 6 of the "will NOT see" list) is UNRULED, so if AC14's re-measure breaches
      `[70,180]` it is reported as the story's finding with the lantern intensity named as the knob —
      never a widened band. Wolf still closes the story at Task 9.
  - [x] *(N/A — no vehicle session available; the pair was SKIPPED and the written-only fallback
        taken. The command stands for whoever runs Task 6.)* Take a before capture on the vehicle
        with the **shipped 6.1 binary** (no code change):
        `gui.exe 7451 --capture 6-2-before.png --frames 1500`. Store in `6-2-signoff/`.
        *(1500, not 600: `simd` ticks at 10 Hz and the vehicle runs >143 fps, so 600 frames is ~44
        ticks against the instrument's ≥100-tick floor and panics before writing a PNG — 6.1's
        lesson, already paid for once.)*
  - [x] Write `6-2-signoff/what-you-will-see.md`: the addition — a warm pool travelling with each of
        the five dwarves, lighting terrain as they pass — with the one-sentence look it aims for.
  - [x] Write the **"what you will NOT see"** list and get each line ruled on: no fuel, no pickup or
        drop, no lantern economy; dwarves remain scaled cubes and do not themselves glow (the pool
        is a light, the dwarf is not emissive); no lantern glyph in the TUI; no z-slicing (7.1); no
        commands from `gui` (8.x). **Raise explicitly:** with five moving lights added to a camp
        that already has a campfire and four torches, the camp may read brighter than the frame
        Wolf approved at 5.4 — name the ground-median ceiling as the measured bar and the lantern
        intensity as the knob.

- [x] **Task 1 — The lantern in the world** (AC: 2, 4, 5)
  - [x] Give every dwarf a lantern in `sim-core` and expose it to the bridge. **Do NOT attach the
        existing `Emitter` component** — see Dev Notes; it would double-emit the dwarf and drive it
        into the static-emitter path's `unreachable!`.
  - [x] Set `light` on the dwarf arm of **both** bridge paths (`crates/simd/src/bridge.rs:24-28`
        snapshot, `:79-83` delta). Both are currently `light: None`; changing one is the obvious
        half-fix and AC2 exists to catch it.
  - [x] Tests: every dwarf on a fresh snapshot and on a delta carries `Some(Lantern)`; the dwarf
        count on the wire is unchanged (no double-emission); a save round-trip preserves lantern
        state with no new `SaveState` field; a seeded scenario run is identical twice.

- [x] **Task 2 — Re-rule the two lantern guards** (AC: 6, 7)
  - [x] Decide and record: does a dwarf lantern reach `emitter_entity`? Add the test that proves it
        does not (or remove the `unreachable!` with the reason).
  - [x] Decide and record: is a *static* lantern emitter in a save still bogus? If yes, rename
        `loading_rejects_lantern_emitters_before_the_wire_bridge` — its name asserts a world state
        this story changes. If no, delete guard and test together with the reason.

- [x] **Task 3 — The TUI ruling** (AC: 8)
  - [x] Confirm by reading `crates/tui/src/view.rs` that nothing renders `Entity.light`, and record
        the one-paragraph reasoning for "no TUI change" in the Dev Agent Record. No `tui` code
        change is expected; if one proves necessary, that is a story-spec defect — raise it.

- [x] **Task 4 — The moving light in `gui`** (AC: 9, 10, 11, 12)
  - [x] Verify (do not re-implement) that `reconcile` already spawns a `PointLight` and a
        `ProjectedLight` for any entity carrying a light, and that 6.1's `flicker_lights` already
        animates it from the table. **The expected `gui` diff is close to zero** — if you find
        yourself special-casing dwarves, stop: AC10 forbids it.
  - [x] Confirm the light rides the blended transform (6.1's `blend_entities` owns translation for
        every non-terrain `WorldProjected` entity, and the `PointLight` is on that same entity).
  - [x] Tests: a dwarf entity carrying `Some(Lantern)` gets a `PointLight` whose colour, intensity
        and range come from the table row; at a mid-blend clock the light entity's translation
        equals the dwarf's rendered translation (AC11); a dwarf with `light: None` gets no
        `PointLight` (the negative case that proves the light is wire-driven, not kind-driven).

- [x] **Task 5 — The instrument** (AC: 15, 16)
  - [x] Extend `CaptureState` (`crates/gui/src/capture.rs`) to accumulate lit-terrain counts at the
        dwarves' successive positions, print one lantern line **before** any assertion (6.1's
        lesson: a failing run must still print its numbers), then assert non-zero and moved.
  - [x] Keep 6.1's motion line and 5.4's range checks exactly as they are.
  - [x] Unit-test the accumulator against a hand-built mirror sequence — a still world fails, a
        moving one passes.

- [x] **Task 6 — The live vehicle session** (AC: 9, 13, 14, 15)
  - [x] Cross-compile and launch per Verification; capture, paste the printed lantern line, motion
        line and range-check line into the Dev Agent Record.
  - [x] Read the F3 overlay at working zoom and at full vista with all five lanterns moving; record
        both labelled `gingerspice / native Windows / NVIDIA`.
  - [x] Confirm by eye and state in the record: a warm pool travels with each dwarf and lights the
        terrain it passes; the camp does not read blown out against the 5.4 frame. **Half of this
        one came back a finding — see the Dev Agent Record.**

- [x] **Task 7 — Tech-art guidelines** (AC: 10 supporting)
  - [x] Add one short section to `docs/tech-art-guidelines.md`: a moving light is the same table
        lookup as a static one, and dwarves are lit by carrying a light on the wire rather than by
        being dwarves.

- [x] **Task 8 — Evidence and the gate** (AC: 18, 19)
  - [x] Write the sabotage table following 6.1's format; run `scripts/mutate.sh` **alone** and paste
        the RED table. Run `cargo clean -p gui` **after** the mutation round (the stale-artifact trap
        has fired twice).
  - [x] `scripts/gate.sh` green; confirm `crates/protocol` is untouched.

- [x] **Task 9 — Wolf's closing sign-off** (AC: 17)
  - [x] Wolf views live against the approved artifact and signs off. **A dev agent cannot check
        this box.** Signed 2026-08-20: *"i think we are done with these stories"* — given with the
        campfire finding below open and knowingly carried.

### Review Findings — code review 2026-08-19 (4 layers, all live, fresh context)

Four layers, **none a coverage hole**: every layer verified `cargo 1.97.1 (c980f4866 2026-06-30)` and
executed code rather than reading it. Diff range `538e1f8..9f4c806` (the frontmatter
`baseline_commit`), read at HEAD `ec308bd` with 7.1 stacked on top — layers took 6.2-era source via
`git show 9f4c806:<path>` and confirmed each finding sits inside the range. Review ran in its own
session; no dev context inherited.

**R1 territory mapping — the open item 6.1's review raised is now answered.** 6.1's review recorded
"R1 has no mapping for the M2 crates and needs one at the Epic 5/6 retro". Ruled by Wolf at this
review: `crates/gui` and `crates/client-core` belong to the **Edge Case Hunter**, as client shells of
the same class as `tui`. Without that ruling `gui` — 244 of this story's 369 changed code lines —
would have belonged to no hunter at all. Both Opus auditors kept whole-diff scope.

**Convergence, the evidence R1 rests on: 5 of 15 deduped findings were raised independently by two
layers** — the empty-first-region latch (auditor + feature), the untested `accumulate_motion`
(auditor + feature), the dark-lantern blind spot (auditor + feature), AC4/AC5's unfalsifiable
assertions (auditor + feature), and `moved()`'s weak oracle (edge + feature). That is **1-in-3
against Epic 3's 1-in-8**. Every convergence involves the Feature Auditor; the Blind Hunter's
territory (`sim-core`, 83 lines) yielded one LOW. **No finding sat in a hunter's excluded territory,
so R1's revert rule is not triggered.**

**The theme, stated plainly: the wire half is proven and the evidence apparatus is not.** Both bridge
arms were verified by reading bytes off a live daemon — 10 entities, dwarves 0–4 all `lantern`, three
consecutive deltas, ids unique, no duplication. What has never run even once is the instrument built
to prove the *rendered* half: `accumulate_motion` has exactly one production caller, zero test
callers, and no devpod can execute it (`Failed to build event loop: neither WAYLAND_DISPLAY nor
WAYLAND_SOCKET nor DISPLAY is set`). `scripts/gate.sh` was run by the Acceptance Auditor and is
**green**.

**AC14's band was NOT widened** — `WARM_PIXEL_FLOOR = 3_000`, `GROUND_LUMINANCE_FLOOR = 70`,
`GROUND_LUMINANCE_CEILING = 180` are byte-identical to their pre-6.2 values, verified by
`git diff 538e1f8..9f4c806 -- crates/gui/src/capture.rs | grep 'FLOOR\|CEILING'` printing nothing.
The one thing the review was told to assume had been quietly done, was not done. It was, however,
never re-measured with lanterns lit.

- [x] [Review][Decision — RULED BY WOLF 2026-08-19: APPROVED, AC1 CLOSED] **AC1 — the sign-off artifact still says AC1 is unmet** — `6-2-signoff/`
      holds one file, `what-you-will-see.md`, whose own header reads *"Status: WRITTEN HALVES
      DRAFTED 2026-08-18, AWAITING WOLF … Until Wolf approves this file as a whole, AC1 is unmet, no
      implementation commit may land."* It was never updated after Task 0 opened the gate on the
      written-only fallback, so the artifact of record contradicts the story's MET claim. Part (a),
      the before-capture, is absent (honestly recorded as skipped — no devpod can render one).
      **Wolf's call:** does "Well let's Start dev then" stand as approval of the written-only
      artifact, closing AC1, or does AC1 stay open? Only he can rule it, and the artifact's header
      needs correcting either way.
      **RULING (Wolf, 2026-08-19): APPROVED.** "Well let's Start dev then" stands as approval of
      the written-only artifact; **AC1 is MET**. Carried to the patch list: update the artifact
      header to record the approval and its date, and note part (a) as **waived** because no
      devpod can render a before-capture.
- [x] [Review][Decision — RULED BY WOLF 2026-08-19: AC14 STAYS OPEN, STORY RETURNS TO in-progress] **AC14 — the story's own named real look risk is UNRULED and unmeasured** —
      the band is intact, but **no capture has ever been taken with lanterns lit**: Task 6 is `[ ]`,
      `6-2-signoff/` holds no PNG, and the capture cannot run in any devpod. Five lights were added
      inside a camp whose approved frame measured ground-median **123** against a ceiling of **180**,
      and line 6 of the "what you will NOT see" list records the ceiling as explicitly UNRULED.
      **Wolf's call:** does 6.2 leave review with AC14 open pending a gingerspice session, or does
      the live capture block the story? This is the single largest open item and no automated
      evidence exists in either direction.
      **RULING (Wolf, 2026-08-19): the vehicle session is owed.** 6.2 returns to `in-progress`
      with **AC13, AC14, AC17 and the rendered halves of AC9 and AC15 OPEN** pending gingerspice.
      The band being intact is not evidence that five new lights respect it. Patches land first,
      so the live session runs against an instrument that can actually fail.

- [x] [Review][Patch] **HIGH — an empty first observation latches an empty region and makes AC15's
      `moved()` assertion permanently, silently true** [`crates/gui/src/capture.rs:126`] —
      `observe()` early-returns only when `first_region.is_some()`, so a first call whose computed
      region is empty falls through and `first_region.get_or_insert_with(|| region.clone())` latches
      `Some({})`. From then on `moved()` — `first != last_region` — holds regardless of whether any
      dwarf ever moves, and `lit_terrain_tiles > 0` and `!last_region.is_empty()` are both satisfied
      by later frames. The entire AC15 movement assertion goes vacuous. The dev's own self-review
      found and fixed the **mirror image** of this — the empty *last* region, guarded at
      `assert_valid` — and did not add the symmetric guard, which reads as oversight rather than
      choice. Orchestrator correction to the layers' reachability reasoning: the mirror is **not**
      empty at startup (`run()` blocks on `read_snapshot` and builds the `Mirror` before
      `App::new()`, `ingest.rs:71-95`), so the trigger is not "no entities yet" but **any first frame
      whose `terrain` query is not yet populated** — the lit region is computed against that query,
      so lanterns can be present while every `lit_region` comes back empty. Reachability is
      **UNPROVEN because the code has never executed anywhere**, which is itself the point. Rated
      HIGH regardless: this is the broken-instrument class that made 2.2's live evidence an artefact,
      against an AC whose whole text is "exit 0 is not a result". Fix is unambiguous — only latch a
      non-empty first region.
- [x] [Review][Patch] **HIGH — `accumulate_motion`, the production code that feeds the instrument, has
      zero test callers and has never executed once** [`crates/gui/src/capture.rs:265-330`,
      `crates/gui/src/ingest.rs:137`] — `rg -n accumulate_motion crates/` returns three hits: the
      definition, the import, and the single registration inside `if let Some(capture) = args.capture`
      in `run()`, which panics without a display. The ~35 lines that actually *derive* the lit region
      on the live path — the `EntityKind::Dwarf && light == Some(Lantern)` filter, the `light.range`
      read, the `distance <= range` terrain sweep, and the `needs_observation` gate — are exercised by
      nothing. **Deleting the whole `if capture.lantern.needs_observation(...) { ... }` block leaves
      the entire suite green.** AC16 asks for a test "driving a hand-built sequence of **mirror
      states**"; the shipped test drives hand-built `BTreeSet` region literals, so the mirror →
      transforms → region derivation is never touched, and the mutation table has three mutations on
      `LanternStats` and **zero** on the extraction block. This is the same defect class as 6.1's
      inert `projection_systems` and it is what leaves the finding above unknowable. Fix: a headless
      test that drives `accumulate_motion` through mirror states + transforms, plus a matching
      mutation.
- [x] [Review][Patch] **MED — nothing in the automated suite can tell a lit lantern from a dark one**
      [`crates/gui/tests/headless.rs:354-356`, `crates/gui/src/capture.rs:294-300`] — the intensity
      assertion is `(0.95 * expected.intensity ..= 1.05 * expected.intensity).contains(&light.intensity)`
      where `expected = light_properties(Lantern)`, i.e. the table is checked against itself; with the
      row zeroed the band becomes `0.0..=0.0` and `0.0` sits inside it. Its comment calls this an
      "independently named ±5% table band", which is false — nothing about it is independent. The
      assertion is legitimate for **AC10** (the value came from the table, not a draw site) and
      useless for **AC9** (the pool is visible). Meanwhile the instrument reads `light.range` and
      **never `intensity` or `color`**, so `range: 0.0` would be caught and `intensity: 0.0` would
      not; and `WARM_PIXEL_FLOOR = 3_000` was baselined at 17,648 warm pixels **from torches and the
      campfire alone**, before lanterns existed, so a dead lantern clears it with 5x headroom. The
      real backstop is AC17 — Wolf's eyes — which costs a scarce vehicle session to discover a defect
      a literal-valued assertion catches for free. This is the self-referential-test antipattern, hit
      at 1.1, 1.2, 1.3 and 6.1's flicker band, now a fifth time. Fix: pin a literal minimum intensity.
- [x] [Review][Patch] **MED — the lantern line never samples a mid-blend frame, so it cannot tell a
      sliding pool from a snapping one** [`crates/gui/src/capture.rs:311`] — `needs_observation`
      gates on the **wire** position map, so `observe()` runs only on frames where a delivered
      position just changed, i.e. exactly the frames where `TickClock::factor()` has reset to ~0.
      Every mid-blend frame is skipped. The assertion therefore proves "the union of lit terrain
      differs between two *wire* positions", which would pass unchanged with `blend_entities` deleted
      and the light snapping tile to tile. The story's own trap note — "the instrument must measure
      the pool MOVING, not the light existing" — is half-satisfied: it catches the light existing.
      Mitigated globally by 6.1's `mid_blend_frames > 0`, and AC11 has headless evidence, so this is
      a hole in *this* instrument rather than an undetectable regression.
- [x] [Review][Patch] **MED — `recorded_camp_snapshot_projects_exactly_five_warm_point_lights` is now
      a stale oracle, which is exactly what AC7 forbids, one test over**
      [`crates/gui/tests/headless.rs:831`] — the dev correctly renamed the `simd` test per AC7, then
      left this one, whose name claims to be *the recorded camp*, hand-building its dwarf with
      `light: None` (`:852`) and asserting the camp projects **exactly five** point lights. The live
      daemon emits **ten** in that camp. The test is not wrong as a wire-driven-not-kind-driven check,
      but its name asserts a state of the camp this story just changed — AC7's rule verbatim — and the
      suite consequently holds no headless picture of the camp as it now ships. Fix: rename to what it
      actually checks, and add a camp-as-shipped case.
- [x] [Review][Patch] **MED — `moved()` is a weak oracle in both directions and will produce a false
      failure on the vehicle** [`crates/gui/src/capture.rs:158-162`] — it compares only the **first**
      and **last** observation, retaining nothing in between, and it unions all five dwarves' lit
      tiles into a single aggregate set rather than tracking per-dwarf regions. Two consequences: a
      dwarf that wanders away and returns, or five dwarves whose aggregate union coincides at the two
      sampled instants, panics on a perfectly working feature; and a single stuck dwarf is masked by
      its moving neighbours. Improbable, not impossible — and a false failure burns a gingerspice
      session and reads as a real defect, which is this project's documented 3.2/3.3 false-failure
      class. Fix: track per-dwarf regions, or assert movement against any observation rather than the
      last.
- [x] [Review][Patch] **MED — 6.2's lantern assertion is not slice-aware and will panic on a correct
      run once 7.1 lands** [`crates/gui/src/capture.rs:367`] — 7.1's `reconcile` drops wire entities
      above the cut (`project.rs:326`, `.filter(|entity| entity.pos[2] <= slice.level())`) and dwarves
      stand at z 9, so at HEAD `gui <port> --capture <p> --frames N --z 5` observes zero lantern
      dwarves and `capture.lantern.assert_valid()` — called unconditionally — panics with *"capture
      observed no terrain lit by dwarf lanterns"*, reporting a defect when the operator merely asked
      for a lower slice. 7.1's own `DrawStats::assert_valid` **is** level-aware; 6.2's is not.
      Whoever runs Task 6 with `--z` hits this, and 7.1's story notes explicitly instruct pinning
      `--z 10`. Found by the Feature Auditor as a cross-story interaction. Fix: skip or gate the
      lantern asserts when no lantern entity is projected at the requested cut.
- [x] [Review][Patch] **LOW (silent-failure exception) — `lit terrain tiles at dwarf positions=N` is a
      running sum, not a tile count** [`crates/gui/src/capture.rs:148`] — `self.lit_terrain_tiles +=
      region.len()` accumulates every observation's region size, so on the vehicle (5 dwarves, range
      16, 100+ observations) the line prints a six- or seven-figure number that reads as a tile count,
      cannot be compared against any run or threshold, and is not what AC15 asks for. Measured by the
      Feature Auditor's probe: 10 observations of a 5-tile pool printed `50` where distinct tiles ever
      lit was `14`. Patched despite LOW under the frostvein silent-failure exception — a wrong number
      on an observability instrument that no test and no human will catch — and it sits in the same
      struct as the two HIGHs.
- [x] [Review][Patch] **LOW — a doc comment this story falsified still claims `dwarves()` is a
      three-tuple** [`crates/sim-core/src/lib.rs:1452-1454`] — the comment on `carrying()` reads *"A
      sibling reader to claims() and items(), which is why dwarves() keeps its three-tuple shape and
      the clients need no new arm."* 6.2 widened `dwarves()` to a four-tuple (`:1494`). The comment
      pre-dates the story and 6.2 never touched it, but 6.2's own change is what makes it false. Same
      class as AC7's "do not leave a test whose name asserts a state of the world this story just
      changed", one level down at the comment. One line.
- [x] [Review][Patch] **LOW (record) — the gate section points at the wrong AC number** [story file,
      "The sign-off gate"] — *"Closing half (AC14): done only when Wolf has viewed the built result
      live"*. The closing sign-off is **AC17**; AC14 is the 5.4 capture range checks. Two different
      open obligations filed under one number, in the section every agent reads first.
- [x] [Review][Patch] **LOW (record) — the 9th mutation has no pasted RED row** [`mutations/6-2-lanterns-in-the-dark.sh`]
      — the table declares 9 mutations; the RED table in the Dev Agent Record has 8 rows. The 9th,
      `reconciliation lights a dwarf the wire left unlit` (added by the orchestrator after it found
      the spawn-arm-only gap), is described in prose as "now KILLS" with no result line. AC18
      requires the RED table pasted.

- [x] [Review][Defer] **Two of AC11's three assertions are tautological** [`crates/gui/tests/headless.rs`,
      `a_dwarf_lantern_stays_on_its_blended_projection_transform`] — `assert_eq!(translation,
      projected_translation(&mut app, id))` compares a value to itself, since the `PointLight` is a
      component on the same entity as the `WorldProjected`/`Transform` it is compared against, so
      there is only one `Transform`; `assert!(has_light, …)` is likewise structural. AC11 as written
      ("the light's translation equals the dwarf's rendered translation") is unfalsifiable given the
      chosen architecture — an AC-text defect, not an implementation one. **Deferred because the third
      assertion is genuinely falsifiable** (translation strictly between the two endpoint x's, which
      dies if the blend is deleted), so AC11 retains real coverage. `[auditor/LOW]`
- [x] [Review][Defer] **AC4's and AC5's lantern assertions cannot fail, and AC5's scenario test does
      not exist** [`crates/sim-core/src/lib.rs` `DWARF_LIGHT`, `crates/sim-core/tests/scenario.rs`] —
      `dwarves()` appends a compile-time constant to every tuple, so the round-trip comparison can
      never disagree on the light and the determinism comparison has no random input to vary.
      `scenario.rs` gained **no new test function** — only mechanical `(_, _, _, _)` destructuring
      updates — while AC5 asks for a scenario test by name. The oracles that *do* carry weight are the
      literal `*light == LightKind::Lantern` check and the diff proving `SavedDwarf` gained no field
      (8 fields, confirmed). **Deferred as an AC-text defect**: this is the sanctioned consequence of
      the story's own "simplest encoding" decision, and satisfied-by-construction is the honest,
      YAGNI-correct outcome. The ACs should be re-worded rather than the code changed.
      `[auditor+feature/LOW]`
- [x] [Review][Defer] **The mutation table has no coverage of AC11's blend, `DWARF_LIGHT`, or the save
      round-trip** [`mutations/6-2-lanterns-in-the-dark.sh`] — the 9 mutations cover both bridge arms,
      both guards, both reconcile light arms and three `LanternStats` assertions. Nothing sabotages
      the blend that AC11's "the pool slides" depends on, nor the lantern constant, nor the save path.
      AC11 is the story's headline interaction and its one falsifiable assertion is unsabotaged.
      **Deferred as the residue** once the extraction-block mutation from the second HIGH is added.
      `[auditor/LOW]`


## Dev Notes

### The epic's wire claim is STALE — verified against source

The epic text says this story is where "`LightKind` gains its `Lantern` variant … the last piece of
M2's sanctioned wire diff (FR30, AD-16)". **That is not true and has not been true since 5.1.**
Verified at story-creation:

- `protocol::LightKind` already has `Lantern` (`crates/protocol/src/lib.rs:46-50`).
- `sim_core::LightKind` already has `Lantern` (`crates/sim-core/src/lib.rs:112`).
- `Entity.light: Option<LightKind>` already exists (`crates/protocol/src/lib.rs:107`).
- The bridge already maps `sim_core::LightKind::Lantern → protocol::LightKind::Lantern`
  (`crates/simd/src/bridge.rs:152-157`).

So **AD-16's sanctioned wire diff is already spent and this story must not spend it again.** The
work is not a wire change; it is switching two hardcoded `light: None` values and making the world
say every dwarf carries one. This materially shrinks the story and AC3 pins it.

### Scope guardrails — do NOT build these here

- **No fuel, pickup, drop or economy.** The lantern is uniform and permanent. If it ever becomes
  droppable, *that* is when it becomes a component with saved state — not now.
- **No `crates/protocol` change** (AC3), and no `client-core` or `tui` change.
- **No special-casing dwarves warm in `gui`.** 6.1's guardrail stands inverted: 6.1 said "dwarves
  carry `light: null` — do not special-case them warm"; 6.2 makes the *wire* say they are lit, and
  `gui` must still learn it only from the wire.
- **No z-slicing (7.1), no designation/zone rendering (7.2), no picking or commands (8.x).**
- **No new dependencies, no shadow work, no light-culling or clustering.** 6.1 measured >143 fps at
  every zoom; optimise only if AC13's reading fails, and then the *measured* problem drives it.
- **No workaround for driver or envelope problems in production code** (5.3's AC9 rule stands).

### What already exists (build on it, do not re-derive)

- **The whole light path is built and fenced off.** `reconcile` spawns a `PointLight` from the table
  for any entity whose wire `light` is `Some`, and tags it `ProjectedLight(kind)` so reconciliation
  stops re-inserting it (`crates/gui/src/project.rs`). 6.1's `flicker_lights` then animates it every
  frame. **A dwarf carrying a lantern on the wire should light up with no `gui` change at all.**
- **The `gui` table already has the Lantern row, unused:** colour `(255,195,110)`, intensity
  `11_000_000`, range `16.0`, `flicker_amplitude 0.05`, `flicker_hz 1.3`
  (`crates/gui/src/appearance.rs:71-77`). Torch and campfire amplitudes were raised to 0.30/0.40 at
  6.1's live viewing; **the lantern row was deliberately left at 0.05** — a carried lamp is steadier
  than a fire. Revisit only if Wolf rules it at the viewing.
- **The light rides the blend for free.** 6.1 made `blend_entities` the sole writer of translation
  for every non-terrain `WorldProjected` entity, and the `PointLight` lives on that same entity — so
  the pool interpolates with the dwarf rather than snapping. AC11 asserts it rather than assuming it.
- **Measured live at story-creation** (`simd` on the shipped seed, snapshot read off the wire):
  5 dwarves, ids 0–4, all `light: None` today; 5 static emitters — campfire id 5 at `[64,64,9]`,
  torches ids 6–9 at `[62,62]`, `[66,62]`, `[62,66]`, `[66,66]`. Dwarves stand at z 9.

### Key decisions & traps

- **Do NOT give dwarves the existing `Emitter` component.** `World::emitters()` filters on it
  (`crates/sim-core/src/lib.rs:1474`) and the bridge chains `emitters()` onto the dwarf list, so a
  dwarf with `Emitter` would be **emitted twice** on the wire *and* driven into
  `entity_kind()`'s `unreachable!("lanterns are not live emitters")`
  (`crates/simd/src/bridge.rs:148`) — a panic in the daemon. Carry the fact some other way.
- **Simplest encoding that satisfies AC4:** the lantern is a constant property of `Dwarf`, so it
  needs no per-dwarf storage and nothing in `SaveState`. A named constant in `sim-core` plus a
  widened `dwarves()` return is enough; the project's YAGNI policy explicitly allows hardcoded
  constants. `// NOTE:` that it becomes a component the day lanterns can be dropped.
- **Two bridge sites, not one.** `light: None` is hardcoded at `bridge.rs:27` (snapshot) and `:82`
  (delta). Fixing only the snapshot yields a client that lights dwarves on connect and drops the
  light on the next tick — a failure mode that looks like a flicker bug.
- **THE REAL LOOK RISK IS THE GROUND-LUMINANCE CEILING, NOT NFR6.** 6.1 measured >143 fps with 4.8x
  headroom, so five more lights are very unlikely to breach NFR6. But 5.4's approved frame measured
  ground-median **123** against a **ceiling of 180**, and this story adds five lights of 11M lm
  inside the camp where the dwarves stand. That is the largest lighting change since 5.4 and AC14 is
  where it will show. If the ceiling is breached, the finding is reported with the lantern intensity
  named as the knob — **never widen the band to make the capture pass.**
- **The instrument must measure the pool MOVING, not the light existing.** 6.1's review found a
  mid-blend counter that read the clock instead of a rendered position and would have passed with
  the feature dead. The same trap is available here: counting lit tiles once proves a light exists;
  AC15 requires the lit region to have *moved* between observations.
- **`mutate.sh` rewrites source in place** — run it alone, and `cargo clean -p gui` afterwards.
- **`simd` has no seed flag** — the seed is `SEED` (`crates/simd/src/main.rs:20`), port positional.
- When torn between simple and general, pick simple and leave a `// NOTE:` naming the limitation.

### Project Structure (files to touch)

```
crates/sim-core/src/lib.rs        UPDATE  dwarves carry a lantern; dwarves() exposes it
crates/simd/src/bridge.rs         UPDATE  both dwarf arms set light (snapshot + delta)
crates/simd/src/main.rs           UPDATE  re-rule the load guard + rename or remove its test
crates/gui/src/capture.rs         UPDATE  lantern accumulator + assertions
crates/gui/tests/headless.rs      UPDATE  moving-light tests (AC9-AC12)
docs/tech-art-guidelines.md       UPDATE  moving light section
_bmad-output/implementation-artifacts/6-2-signoff/                        NEW  artifact + capture
_bmad-output/implementation-artifacts/mutations/6-2-lanterns-in-the-dark.sh  NEW
_bmad-output/implementation-artifacts/deferred-work.md                    UPDATE if anything defers
```

`crates/protocol` and `crates/client-core` and `crates/tui`: **no change** (AC3, AC19).

### Previous story intelligence (deltas that change THIS story)

- **6.1 is not merged.** Branch from `6-1-the-world-moves`, not `main` — the whole moving-light
  interaction (AC11) depends on 6.1's blend, and `main` does not have it. If 6.1 has merged by the
  time this starts, branch from `main` instead and say so in the record.
- **6.1's live viewing produced two findings that a green suite could not see** — a flicker that ran
  correctly but was invisible at ±7%, and a dig site whose undiggable ramps left a wall. Both were
  presentation truths only a human eye caught. Expect the same class here: "the pool moves" is a
  mechanism test; "it reads as a lantern" is Wolf's call at Task 9.
- **6.1's review found three untested drive lines** — deleting `observe_tick`, `delta_secs()` or
  `elapsed_secs()` each killed the feature with a green suite. When writing AC12's tests, assert the
  production systems *drive* the outcome, not that a hand-advanced value produces it.

### Verification

**Executed at story-creation (the headless half — non-zero evidence, P6 rule).** A live `simd 7796`
on the shipped seed was read off the wire: all five dwarves report `light: None` today and the five
static emitters report `campfire`/`torch`, confirming both the starting state AC2 changes and that
no dwarf currently reaches the emitter path. The two hardcoded `light: None` sites and the two
lantern guards were read at source and are cited above.

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

**The lantern capture (the obligation the dev agent inherits — it cannot run until the wire carries
the lantern):**

```bash
gui.exe 7451 --capture 6-2-lantern.png --frames 3000
```

**Required non-zero observations** (paste all three printed lines into the record): the lantern line
reports a non-zero lit-terrain count at the dwarves' positions and a lit region that MOVED; 6.1's
motion line still reports ticks ≥ 100, position changes > 0 and blend frames > 0; the range-check
line reports warm-lit pixels above the floor and ground-median inside `[70,180]`. The startup line
still reads `projected 53365 terrain cubes`. **Exit 0 is not a result.**

**NFR6 reading (AC13):** F3 overlay on, all five lanterns moving, read sustained fps at working zoom
and at full vista; record both labelled `gingerspice / native Windows / NVIDIA`.

**Sabotage:**

```bash
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/6-2-lanterns-in-the-dark.sh
```

### If this overruns one session

The seam is **wire | instrument**: Tasks 1–4 (the lantern reaches the wire and lights the dwarf —
the headline outcome exists and is visible) versus Tasks 5 (the instrument that measures it). Both
halves are needed for the gate, so a split defers the sign-off, not part of it. Commit per green
task; restate RED evidence in any continuation handoff.

### References

- Story 6.2 epic text — `_bmad-output/planning-artifacts/epics.md:847-878`
- FR29/FR30/FR32, NFR6, NFR7, AD-16 — `epics.md`, `architecture/…/ARCHITECTURE-SPINE.md`
- The two hardcoded dwarf `light: None` sites — `crates/simd/src/bridge.rs:24-28`, `:79-83`
- The static-emitter path and its guard — `crates/simd/src/bridge.rs:132-157`
- The save-load lantern guard and its test — `crates/simd/src/main.rs:501-503`, `:744-756`
- `emitters()` and the `Emitter` component — `crates/sim-core/src/lib.rs:109`, `:1474-1486`
- `dwarves()` — `crates/sim-core/src/lib.rs:1441-1456`; dwarf spawn — `:1214-1226`
- The gui light table incl. the unused Lantern row — `crates/gui/src/appearance.rs:52-79`
- Reconcile's light spawn + `ProjectedLight` gate, and the blend — `crates/gui/src/project.rs`
- Capture range checks and their bands — `crates/gui/src/capture.rs`
- 6.1's story, its review and its two live findings — `6-1-the-world-moves.md`
- Story rules, instrument rule, exit-0 — `docs/technical-preferences.md:64-101`

## Dev Agent Record

### Agent Model Used

`gpt-5.6-terra` (Codex/Völundr, reasoning effort high) — dev.
`claude-opus-5[1m]` — orchestration, verification and the sabotage round.
*(Exact ids per the model policy: a family nickname is what makes an old ledger row unreadable.)*

### Debug Log References

- Task 1 RED, `cargo test --offline -p simd every_dwarf_carries_a_lantern_in_snapshot_and_delta_without_duplication`:
  `every snapshot dwarf must carry the lantern wire value` at `crates/simd/src/bridge.rs:406`;
  0 passed, 1 failed. The test independently expected `Some(protocol::LightKind::Lantern)` on each
  of the exactly five dwarf entities in both outputs.
- Task 2 guard sabotage RED, after temporarily replacing `entity_kind(Lantern)`'s guard with
  `EntityKind::Dwarf`, `cargo test --offline -p simd static_lantern_emitters_remain_rejected_by_the_bridge_guard`:
  `test did not panic as expected at crates/simd/src/bridge.rs:431:8`; 0 passed, 1 failed. The
  `unreachable!` was restored immediately after the observed failure.
- Task 4 projection sabotage RED, after temporarily changing the generic initial-spawn light
  branch to `None`, `cargo test --offline -p gui --test headless
  a_wire_declared_dwarf_lantern_uses_the_shared_appearance_table` failed at
  `crates/gui/tests/headless.rs:224`: `a wire-declared lantern must project a point light`; 0
  passed, 1 failed. The generic reconciliation branch was restored immediately.
- Task 5 RED, before implementation, `cargo test --offline -p gui
  lantern_instrument_requires_a_lit_region_to_move` failed to compile with `E0433: cannot find
  type LanternStats in this scope` at `crates/gui/src/capture.rs:372` and `:378`; the test named
  the missing accumulator before it was added.
- Task 5 review-fix RED, after adding the vanished-final-region case, `cargo test --offline -p gui
  lantern_instrument_requires_a_lit_region_to_move` failed at `crates/gui/src/capture.rs:471`:
  `assertion failed: std::panic::catch_unwind(|| vanished.assert_valid()).is_err()`; the old
  accumulator incorrectly accepted a non-empty first region followed by an empty final region.
- Task 8 mutation RED table (run alone; all KILLED on genuine assertions):

  | Mutation | Result |
  | --- | --- |
  | snapshot dwarf arm drops lanterns | `bridge.rs:403` assertion; 0 passed, 1 failed |
  | delta dwarf arm drops lanterns | `bridge.rs:403` assertion; 0 passed, 1 failed |
  | static emitter bridge accepts lanterns | expected-panic test failed; 0 passed, 1 failed |
  | saved static lantern emitters load | `main.rs:756` assertion; 0 passed, 1 failed |
  | wire-declared lanterns no longer create lights | `headless.rs:224` assertion; 0 passed, 1 failed |
  | lantern capture accepts an unmoved region | `catch_unwind(...).is_err()` assertion; 0 passed, 1 failed |
  | lantern capture loses its lit-terrain count | `capture.rs:138` assertion; 0 passed, 1 failed |
  | lantern capture accepts an empty final region | `catch_unwind(...).is_err()` assertion; 0 passed, 1 failed |

### Completion Notes List

- Task 1: Added `sim_core::DWARF_LIGHT`, a uniform `LightKind::Lantern` value returned by the
  widened `World::dwarves()` reader. It is not an ECS `Emitter` or saved per-dwarf state. Both
  bridge dwarf arms now map that value to the existing protocol variant. `cargo test --offline -p
  sim-core -p simd` passed (49 sim-core unit, 10 save/load, 30 scenario, 12 worldgen, 16 simd unit,
  and 61 simd serve tests).
- Task 2: Dwarf lanterns stay exclusively on the dwarf bridge arm; the static-emitter reader
  contains only torches and the campfire, so the existing `emitter_entity` lantern guard remains
  true. A malformed saved *static* lantern emitter remains invalid; renamed its test to
  `loading_rejects_static_lantern_emitters_before_the_wire_bridge` to state that distinction.
  `cargo test --offline -p simd` passed (18 unit and 61 serve tests).
- Task 3: No `tui` change. `tui::view::render` selects cells only by `EntityKind`, job state,
  position, and crowd/item contention; it never reads `Entity.light`. A lantern glyph would not
  distinguish any dwarf because every dwarf carries one. `client-core::Mirror` retains the wire
  entity (including its light field) unchanged for both clients, preserving parity.
- Task 4: No `gui` production change. Existing generic reconciliation already creates a
  `PointLight` and `ProjectedLight` for every wire `Some(light)`, and the shared schedule already
  flickers it from the `LightKind` table. Headless tests now prove a lantern dwarf gets the
  lantern table's colour/range/intensity band, an unlit dwarf gets no light, and the lantern lives
  on the same mid-blended `WorldProjected` transform as the dwarf. `cargo test --offline -p gui`
  passed (42 library, 24 headless integration, and 1 non-ignored capture test).
- Task 5: Added the capture-only `LanternStats` accumulator. After the shared projection set, it
  derives each lantern dwarf's lit terrain set from the same rendered `PointLight` transform and
  terrain transforms, records only successive delivered dwarf positions, prints the lantern line
  before either assertion, and requires non-zero lit terrain plus a changed first/last region.
  The existing motion line and image range checks are unchanged. `cargo test --offline -p gui`
  passed (43 library, 24 headless integration, and 1 non-ignored capture test).
- Task 7: Added the short moving-lights guideline: wire-carried moving lights use the same
  `LightKind` table lookup as static lights, and dwarves are never special-cased warm.
- Task 8: `scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/6-2-lanterns-in-the-dark.sh`
  ran alone again after the review fix: all eight mutations were KILLED, then `cargo clean -p gui` ran afterwards as required.
  The subsequent `scripts/gate.sh` was GREEN. `git diff --stat 6-1-the-world-moves...HEAD --
  crates/protocol` is empty; there is no protocol, client-core, or tui change.
- Self-review pass 1 (`codex review --base 6-1-the-world-moves`) raised two P2 findings in the
  lantern instrument: an empty final region could falsely count as movement, and all terrain was
  scanned on every capture frame. Fixed both: `assert_valid` now requires a non-empty final region,
  with a RED regression test and mutation; `accumulate_motion` first compares the five delivered
  lantern positions and only builds terrain regions after they change. A post-fix full gate passed.
- Self-review pass 2 (`codex review --base 6-1-the-world-moves`) completed with no actionable
  findings. Review stopped after two passes, below the three-pass cap.
- Tasks 6 and 9 remain unchecked: this devpod cannot run the required gingerspice native Windows /
  NVIDIA live session, and Wolf alone supplies the closing sign-off. Status is `review` for the
  completed headless implementation and evidence.


### Orchestrator verification of the Codex dev run (2026-08-18)

Codex (`gpt-5.6-terra`, reasoning effort **high**, session `01a0150d-45ad-71e0-8627-2cd6dd87730f`)
exited 0. **Exit 0 was not trusted.**

**Verified GOOD, independently:**

- **No auth failure.** All 8 log matches for `401|Missing bearer|Unauthorized` are the handoff
  prompt's own text and `:401` source line numbers. No real 401.
- **Scope holds exactly.** `git diff --stat 6-1-the-world-moves..HEAD` over `crates/protocol`,
  `crates/client-core`, `crates/tui`, `Cargo.toml` and `Cargo.lock` is **empty** — AC3 and AC19
  hold, and the story's central finding (the wire diff was already spent) is borne out: **this story
  changed no wire type at all.**
- **`scripts/gate.sh` GREEN** on my own run, and again after the mutation round with
  `cargo clean -p gui` in between.
- **Commit cadence MET — 9 commits for 8 dev tasks**, all authored `Völundr <jeicei75@gmail.com>`,
  nothing pushed (`refs/remotes` has no `6-2` branch). This is the first story where the floor was
  actually asked for in the handoff prompt rather than only in AGENTS.md, and it is the first story
  that met it.
- **Self-gate ran and CONCLUDED** — two passes, the first raising two P2 capture issues (both
  fixed), the second returning nothing actionable, so it stopped at two of the three allowed. That
  is a real result rather than 6.1's coverage hole, where the self-gate produced no conclusion on
  either run.
- **`gui` needed almost nothing, exactly as the story predicted.** The only `crates/gui/src` file
  touched is `capture.rs` — the instrument. `project.rs` and `appearance.rs` are **untouched**, so
  the moving light came free from 6.1's reconcile + blend, and AC10's "no dwarf special-casing"
  holds structurally rather than by assertion.
- **END TO END ON A LIVE DAEMON, which no test can fake.** Built `simd` from this branch, ran it,
  and read the wire:

  ```
  SNAPSHOT path: ids 0-4 dwarf light=lantern   (5/5)   ids 5-9 campfire/torch unchanged
  DELTA    path: ids 0-4 dwarf light=lantern   (5/5)   ids 5-9 campfire/torch unchanged
  entity count still exactly 10 -- no double-emission through the Emitter trap
  ```

  Both bridge arms carry it, so the half-fix failure mode the story warned about did not occur.
- **9/9 mutations KILLED** (8 from the dev run + 1 I added, below), run alone, tree verified clean
  afterwards. The table includes **separate** mutations for the snapshot arm and the delta arm,
  which is the right shape for AC2.

**ONE GAP I FOUND BY SABOTAGE, and closed.** AC10 says dwarves must not be special-cased warm
*anywhere* in `gui`. Reconciliation has **two** light-insertion arms — the spawn arm
(`crates/gui/src/project.rs:368`) and the existing-entity arm (`:340`). Sabotaging each separately:

```
SPAWN arm lights every dwarf regardless of the wire   -> an_unlit_dwarf_gets_no_point_light RED  ✅
EXISTING-ENTITY arm, same sabotage                    -> 68/68 GREEN                          ❌
```

The negative case was asserted only on the **spawn frame**. The identical defect on a later
reconcile pass was invisible — **6.1's defect class exactly**, where reconcile misbehaved on frames
the spawn-frame tests never reached. Closed by extending `an_unlit_dwarf_gets_no_point_light` to run
a second `app.update()` plus a production `reconcile_projection` pass and re-assert, and by adding
the mutation `reconciliation lights a dwarf the wire left unlit`, which now KILLS.

**Recorded, not fixed:** the instrument identifies its subjects as
`kind == Dwarf && light == Some(Lantern)` (`crates/gui/src/capture.rs:131`). That is measurement
rather than rendering, so it does not violate AC10 — but it does hardcode the dwarf/lantern pairing
in `gui`, and it is worth a review layer's opinion.

**Still OPEN and not closable by any agent:** Task 6 (the live vehicle session) and Task 9 (Wolf's
sign-off), and with them **AC9's look, AC13's NFR6 reading and AC14's re-measured range checks —
including the ground-luminance ceiling, which is this story's real look risk and remains UNRULED**
because the artifact was approved on the written-only fallback.

### Code review and patch round (2026-08-19, orchestrator = Claude Opus, fresh context)

Four layers, none a coverage hole; findings and their triage are in **Review Findings** above. This
records what the patch round actually did and what it cost to be sure of it.

**Applied: 12 patches** — 2 HIGH, 5 MED, 5 LOW (three of them silent-failure or record traps, two of
them story-file record fixes). The three LOW AC-text defects were deferred to `deferred-work.md`
rather than patched, per the review-cost rule; they are AC wording to fix at re-authoring, not code.

**One verification pass, not one per patch:** `scripts/gate.sh` **GREEN** (fmt, clippy `-D warnings`,
full workspace tests, all three no-`sim-core`-edge probes, metrics ledger), 52 `gui` unit tests and 30
headless tests green.

**THE SABOTAGE TABLE CAUGHT THE REVIEW'S OWN PATCH, TWICE.** First run: **10 KILLED / 2 SURVIVED / 1
APPLY-FAILED**. This is the entry worth carrying forward, because both survivors were *new tests
written by this review* that passed for the wrong reason:

- `lantern capture latches an empty first region` SURVIVED. The new test held its dwarf at ONE
  position across all three observations, so `needs_observation`'s early return swallowed
  observations two and three and the test passed on an unrelated assert. The dwarf has to keep
  changing position while its lit region stays constant for the sabotage to bite. Fixed.
- `lantern capture accepts an empty final region` SURVIVED. Making `moved()` per-dwarf and sticky
  made it catch the vanished case on its own, which left `!last_region.is_empty()` pinned by
  nothing. Fixed with a dedicated test where a dwarf moves and *then* the lanterns go dark.
- `lantern capture accepts an unmoved region` APPLY-FAILED — an ORIGINAL mutation that still pointed
  at the pre-patch `moved()` body. A patch that rewrites a mutated line silently breaks its own
  sabotage; re-pointed.

Second run: **13/13 KILLED.** Mutations went 9 → 13.

**A finding proved itself during the patch round.** The new headless test would not go green until it
explicitly advanced `TickClock` before the observing frame — because `accumulate_motion` samples on a
delivered POSITION change, which lands at factor ~0 with the light still rendered at the old tile.
That is direct empirical confirmation of the mid-blend sampling finding, stronger than the static
trace that raised it. The narrow fix was taken deliberately: sampling every frame would run the
terrain sweep across the whole draw set each frame and corrupt the very fps reading AC13 asks for, so
the gate stays and a `// NOTE:` now records what the lantern line does and does not evidence. **This
is the one patch where the limitation was documented rather than removed** — flagged rather than
buried.

**Not covered by any test, and said plainly:** the slice-awareness fix (`capture.rs`, skipping the
lantern asserts when a requested cut hides every dwarf) has **no test**, because `capture_after_frames`
needs a window and no devpod has one. It is a one-branch guard verified by reading only.

**What this round did NOT prove.** Nothing here rendered a pixel. The windowed client cannot start in
any devpod (`Failed to build event loop: neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is
set`), so **AC13, AC14, AC17 and the rendered halves of AC9 and AC15 remain OPEN** pending a
gingerspice session. The instrument is now considerably harder to fool than it was, which is the point
of doing this before the vehicle session rather than after — but a trustworthy instrument that has
still never run is not evidence that the feature works.

### File List

- crates/sim-core/src/lib.rs
- crates/sim-core/tests/save_load.rs
- crates/sim-core/tests/scenario.rs
- crates/sim-core/tests/worldgen.rs
- crates/simd/src/bridge.rs
- crates/gui/tests/headless.rs
- crates/gui/src/capture.rs
- docs/tech-art-guidelines.md
- _bmad-output/implementation-artifacts/mutations/6-2-lanterns-in-the-dark.sh
- _bmad-output/implementation-artifacts/6-2-lanterns-in-the-dark.md
- _bmad-output/implementation-artifacts/sprint-status.yaml


### Task 6 + Task 9 — the live vehicle session (2026-08-20, gingerspice / native Windows / NVIDIA)

6.2 rode the same binary and the same `simd` as 6.1 and 7.1, per `vehicle-session-runbook.md`, so
the lantern instrument printed on every capture of the sitting rather than only its own.

**AC15 — the lantern line, at full depth and at a cut:**

```
lantern: dwarf positions observed={...58 positions...} lit terrain tiles at dwarf positions=3171 moved=true
lantern: dwarf positions observed={...26 positions...} lit terrain tiles at dwarf positions=1718 moved=true
```

`moved=true` on every run — the assertion this instrument exists for. The lit-tile count falling
3171 -> 1718 between full depth and the z 9 cut is the slice removing terrain from under the pools,
not a lantern regression: `lantern_assertions_apply` asks the mirror whether a dwarf sits at or below
the cut, and at z 9 all five do, so the assertions ran rather than being skipped.

**AC13:** sustained **>140 fps** at working zoom and full vista, `gingerspice / native Windows /
NVIDIA`, with five moving lanterns on the stacked 6.1 + 6.2 + 7.1 binary. NFR6's working-zoom bar is
60. Five moving point lights on top of five static emitters cost nothing measurable.

**AC9 / AC14 — by eye.** Wolf: *"I can see light moving with dwarves"* — the pool travels with the
dwarf and lights the terrain it crosses, which is the half of AC9 that separates a real moving light
from a glow stuck to a cube. Confirmed.

### The finding this story's own runbook predicted, and what was done about it

The runbook named the risk in advance: *"the range check only guards the ground median inside
[70,180] — it cannot tell you whether the camp LOOKS over-lit. That judgement is yours and there is
no instrument for it."* It came back exactly there. Wolf, at full depth: **"Camp is too blown out"**,
and after the first drop, **"campfire light is maybe still too blown"**.

The ground median measured **123 on the same frame — the approved artifact's own figure, to the
digit.** The field was never wrong; a local highlight was. No range check can see that, and this is
the second time in this epic that the only instrument capable of catching a defect was Wolf's eye.

**Applied:** lantern **11,000,000 / range 16 -> 5,000,000 / range 14**. The white-clip radius scales
as sqrt(intensity), so the blown pool shrinks by about a third.

**NOT applied, and carried open: the campfire.** After the lantern drop Wolf still reads the campfire
as blown, and the diagnosis points away from this story:

- `flicker_lights` multiplies the base intensity by a band of `1 +/- amplitude`.
- Commit **04e6de5** took the campfire amplitude **0.11 -> 0.40**, so its peak went 35.5M -> **44.8M**
  — 40% above the value story 5.4 sized against the artifact Wolf approved, on a light whose own
  comment records that 72M "blew a ~9-tile pool to flat white".

So the blow-out is most likely **6.1's amplitude raise reaching past 5.4's approved ceiling**, not
6.2's lanterns and not 5.4's calibration. Two fixes were put to Wolf — (a) base 32M -> 23M so the
*peak* lands on the approved brightness while the fire keeps breathing, or (b) amplitude 0.40 -> 0.25
so the still frame matches and the breathing calms — and **neither was chosen before sign-off**.
Recorded here as open rather than closed: **the campfire reads over-lit at full depth as shipped.**

### The cost of tuning a look-constant here

Dropping one intensity required three hand-written literals to move with it: the palette pin in
`appearance::tests`, this story's `the lantern goes dark but stays present` sabotage row, and a
doc comment in `headless.rs` naming the shipped figure. The palette pin went red immediately and
caught the change, which is the pin working. The sabotage row would have gone **silently dead** —
the same failure 6.1's `torch flicker band widens` row suffered from 04e6de5 until 2026-08-20.

**A sabotage row naming a tuned literal dies the moment the knob moves, and dies quietly.** Any
future look change must retarget its rows in the same commit.

### Sabotage — 13 of 13 KILLED

`the lantern goes dark but stays present` retargeted to the new intensity and re-verified:

```
=== the lantern goes dark but stays present ===
thread 'a_wire_declared_dwarf_lantern_uses_the_shared_appearance_table' panicked at crates/gui/tests/headless.rs:466:5
```

`scripts/gate.sh` GREEN cold.

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-18 | Orchestrator verification of the Codex dev run. Gate green on my own run, scope exact (**no `crates/protocol` change at all** — the story's stale-premise finding borne out), 9 commits all Völundr, nothing pushed, self-gate concluded in 2 of 3 passes. **Verified end to end on a live daemon: all 5 dwarves carry `lantern` on BOTH wire paths, entity count still 10, no double-emission.** `gui` needed only `capture.rs` — the moving light came free from 6.1. **One gap found by sabotage and closed:** AC10's negative case was asserted only on the spawn frame; the same dwarf-special-casing applied to reconciliation's existing-entity arm left 68/68 green. Test extended across a reconcile pass, mutation added, table now 9/9 KILLED. |
| 2026-08-18 | Story created. **The epic's central wire claim was falsified against source: `protocol::LightKind::Lantern`, `sim_core::LightKind::Lantern` and `Entity.light` all already exist, so AD-16's sanctioned wire diff is already spent and this story adds no protocol change** — AC3 pins that. Verified live off the wire that all five dwarves carry `light: None` today. Found two existing lantern guards that this story must re-rule rather than trip over: `emitter_entity`'s `unreachable!` and `load_world_from`'s rejection with its now-misnamed test. Identified the `Emitter`-component trap (double-emission plus a daemon panic) and the two-site `light: None` half-fix. Flagged the ground-luminance ceiling — not NFR6 — as the story's real look risk. |
| 2026-08-18 | Task 1 complete: every dwarf now carries the uniform, non-persisted lantern through both bridge paths; no protocol change. |
| 2026-08-18 | Task 2 complete: re-ruled static lantern emitters as invalid while proving dwarf lanterns bypass that path. |
| 2026-08-18 | Task 3 complete: recorded the no-TUI-change ruling. |
| 2026-08-18 | Task 4 complete: verified the existing generic moving-light path and added its lantern guardrail tests. |
| 2026-08-18 | Task 5 complete: added the moving-lantern capture accumulator and its still-versus-moving test. |
| 2026-08-18 | Task 7 complete: documented the generic moving-light rule. |
| 2026-08-18 | Task 8 complete: eight sabotage mutations killed after self-review fixes; post-mutation GUI clean and full gate green. Status set to review with the vehicle and Wolf sign-off tasks explicitly open. |
| 2026-08-18 | Self-review pass 2 found no actionable findings; review stopped after two passes. |
