---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: 15b3635688ddcc2168ef9f2981b477d1490d8fa9
---

# Story 9.1: The Frame Stops Blowing Out

Status: review

## Story

As the boss,
I want the campfire to light the camp without washing out everything near it,
so that I can see what is happening at the heart of my fortress.

## The epic's premise is STALE — read this before anything else

Epic 9 says the campfire peaks at **44.8M, ~40% above what 5.4 sized against**. **It has not since
2026-08-22.** Commit `57e468e` applied Wolf's ruling (d) — base `32M → 25M`, amplitude kept at
`0.40` — landing the peak at `25.0M × 1.40 = 35.0M`, **under** the `APPROVED_PEAK = 35_520_000.0`
pin that now guards it [appearance.rs:540]. The ruling is recorded in the table's own comment
[appearance.rs:66-75].

**Wolf re-confirmed the blow-out on 2026-08-27 — five days after that fix landed, on a binary
carrying the 25M base**: *"campfire is still overblown so it hides stuff"* [deferred-work.md:943].

So the DECISION (close the blow-out) stands and the REASON in the epic text is wrong. The peak is
already at the approved ceiling and the defect survived it. **Any AC or task text repeating 44.8M
repeats a falsified fact.** Nothing in this story re-litigates ruling (d).

## What the numbers actually say — measured at story creation, 2026-08-28

Measured on the committed PNGs with the instrument's own luma (`0.2126R + 0.7152G + 0.0722B`,
[capture.rs:437-439]) and its own ground window (x 0.25–0.75, y 0.50–0.90, [capture.rs:422-423]):

| frame | ground median | p99 | area ≥ 200 | **largest ≥ 200 blob** |
| --- | --- | --- | --- | --- |
| `5-4-signoff/candidate-artifact-2026-08-15.png` (**mock**, approved) | 124.1 | 188.6 | 0.685 % | 0.4959 % |
| `5-4-signoff/boot7.png` (**5.4 converged — Bevy, Wolf-approved**) | 123.4 | 216.7 | 1.563 % | **0.6651 %** |
| `6-1-signoff/6-1-motion-after.png` | 123.4 | 216.3 | 1.559 % | 0.6546 % |
| `7-2-signoff/7-2-marks-vista.png` (**current, post ruling (d)**) | **123.4** | 225.6 | 1.840 % | **0.9883 %** |
| `7-2-signoff/7-2-marks-working.png` (working zoom) | 230.5 | 253.1 | 15.536 % | — |

**The ground median is 123.4 on every single Bevy frame in that table** — 5.4's approved one, 6.1's,
and today's — and 124.1 on the artifact. It is constant across every frame anyone has ever
complained about and every frame anyone has ever approved. **It carries no information about this
defect.** That is the whole case for AC5.

The blown pool, meanwhile, moved: the largest connected near-white region went from **0.6651 %** in
the frame Wolf approved at 5.4 to **0.9883 %** today, up 49 %.

6.2 recorded this in words on the very frame Wolf called blown: *"The ground median measured **123
on the same frame — the approved artifact's own figure, to the digit.** The field was never wrong;
a local highlight was. **No range check can see that**"* [6-2-lanterns-in-the-dark.md:864-866]. The
median is median-**by design**, so blown emitter faces cannot move it [tech-art-guidelines.md:133].

The working-zoom row reproduces 7.2's recorded 231 to within 0.5 [7-2-read-the-working-zoom.md:787],
which is what validates the measurement method rather than making it a coincidence.

### Two things this table does NOT support — read before setting a constant

- **Do not calibrate against the artifact.** It is a software isometric mock, not a Bevy capture
  [5-4-signoff/README.md]. Its distribution has a different *shape*: at threshold 250 the mock is
  **whiter** than every Bevy frame (0.267 % vs 0.107 %) while at 200 it is darker. A mock-vs-Bevy
  threshold would measure the renderer, not the defect. **Calibrate Bevy against Bevy.**
- **These are archive frames, not a controlled A/B.** They come from different stories with
  different world states, marks and dwarf positions. The 0.665 → 0.988 growth is a real signal and
  it is *not* a controlled measurement. AC13 exists to settle it under control.

## Wolf's rulings, taken at story creation 2026-08-28

| # | Question | Ruling |
| --- | --- | --- |
| W1 | Epic AC1 names the 70–180 median band as the instrument, but it reads 123 on the blown frame and passes today unchanged | **Add a local blow-out measure the median cannot see.** The 70–180 band stays, demoted to a non-regression guard. |
| W2 | The 2026-08-22 peak ruling landed and the defect survived it — which lever may 9.1 open? | **`PointLight` shadows, and only that.** No intensity, amplitude, range, or emissive change. |
| W3 | Does 9.1 need a blocking Task 0 artifact? | **No.** UX-DR22's opening half is met by the approved 5.4 artifact plus the value-floor band [epics.md:1183-1184]. Only the closing half is owed. |

**W2 is the scope, and it is narrow on purpose.** [project.rs:416-424] builds every emitter's
`PointLight` from colour, intensity and range and nothing else, so `shadows_enabled` takes Bevy's
default of `false` — **the campfire lights straight through solid terrain and through entities.**
Only the directional light casts shadows [ingest.rs:642]. This was filed at 5.4's review and never
acted on: *"Point lights cast no shadows … AC4's 'shadow' term is carried entirely by the single
250-lux directional. Perf-vs-look tradeoff for a vehicle session, not a headless call"*
[deferred-work.md:619-622]. It is the one structural cause the peak ruling could not have reached.

### The tension in W1 + W2, stated so nobody discovers it at review

**The authorised lever and the chosen instrument may not be coupled.** Shadows remove light that
passes *through* geometry; they do not dim the unoccluded pool immediately around the fire. The
near-white ceiling in AC5 may therefore not be reachable by shadows alone.

**That is a designed decision point, not a hidden risk.** If AC5 stays red with shadows correct
and NFR6 met, **STOP and report the measured numbers.** Do NOT open intensity, amplitude, range or
emissive to reach the ceiling — those are levers Wolf considered and did not authorise, and taking
one silently is the failure this project has named three times. A story that proves shadows are
insufficient, with numbers, has done its job.

## The live vehicle — unchanged, do not re-derive

**gingerspice**: cross-compiled `gui.exe` on native Windows, NVIDIA Vulkan, `simd` in WSL over
localhost. **No devpod can open a window** — measured at 5.3, re-measured twice at 8.1
[vehicle-session-runbook.md:17-19]. Consequence: **every `--capture` AC here is vehicle-bound and
will not execute anywhere else.** The headless suite is the only half that runs in a devpod, and
this story deliberately gives it real teeth (AC7).

`gui.exe` now states its own commit — `gui build <sha>` is the first line printed, before the
connect can fail [ingest.rs:91]. Compare it against `git rev-parse --short HEAD`. **Stop doing
timestamp arithmetic**; M2-7 closed at `5ff05f0` on the trap's sixth firing [deferred-work.md:909].

**Epic 9 spends one vehicle session for all four stories** [epics.md:1185-1186]. Write this
story's card so it can be merged into that sitting.

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier, not `--fast`) is green on a cold rebuild, and the diff is
   confined to this story's own commit range from `baseline_commit`. *(Not `main..HEAD` — every M2
   story is stacked and that range is wrong by default; 7.1 shipped this defect as the 10th
   instance of the AC-text class.)*

### The lever

2. The `PointLight` projected for a campfire casts shadows, so campfire light no longer reaches
   surfaces with solid terrain between them and the fire. *Mechanism is load-bearing and Wolf ruled
   it (W2): the outcome "light does not pass through rock" is not observable in any headless test,
   so the shadow-casting property is the requirement.*
3. `crates/gui/src/appearance.rs` light-table values are **byte-identical** to `baseline_commit`:
   no intensity, range, `flicker_amplitude` or `flicker_hz` change for any `LightKind`, and
   `material.emissive` [project.rs:403] is untouched. *Mechanism is load-bearing: these are the
   levers W2 withheld, and AC13's data-table discipline [5-4-the-cold-boot.md:106] means a look
   change that dodges the table is worse, not equivalent.*
4. `crates/protocol/`, `crates/sim-core/`, `crates/simd/`, `crates/client-core/` and `crates/tui/`
   are unmodified. This is a `gui`-only story; the wire carries kind identifiers only, never RGB,
   radius or flicker (AD-16).

### The instrument

5. `gui --capture` reports a **blown-pool** measure of the frame — the largest connected region of
   pixels at or above a named luminance threshold, as a fraction of the frame, plus the frame's
   p99 — and **asserts a ceiling on that fraction** at the boot vista. The numbers print before any
   assertion, as every other capture line does [capture.rs:1004]. *Largest-connected-region, not
   total area: the total moved 18 % between the frame Wolf approved and the frame he rejected while
   the pool moved 49 %, and "a ~9-tile pool blown to flat white" [appearance.rs:54-56] is a
   contiguous thing.*
6. **The ceiling is a Bevy-to-Bevy bar: the blown pool is no larger than in `boot7.png`, the
   converged frame Wolf approved at 5.4** — 0.6651 % at threshold 200, against 0.9883 % today. It
   is therefore RED at `baseline_commit`. *The approved artifact is a software mock whose
   near-white distribution has a different shape; it is context in this story, never the calibration
   source.*
7. **The measure is proven discriminating in the gate, on real committed pixels.** A headless test
   decodes `5-4-signoff/boot7.png` and `7-2-signoff/7-2-marks-vista.png` with the `image`
   dev-dependency (already present, [gui/Cargo.toml:17]) and asserts the measure separates them in
   the right direction and straddles the ceiling, while the ground median does **not** separate them
   (123.4 on both). *This is the independent oracle, and the second clause is what makes it
   non-tautological: it pins that the new measure sees something the old one cannot. An assertion
   derivable from the constant it guards is the self-referential antipattern hit at 1.1, 1.2, 1.3
   and again in 6.1's flicker band.*
8. **Seam exercised.** A failing near-white ceiling makes the process exit **non-zero**, and a test
   proves it: deleting the assertion, or discarding its result, turns that test RED. *Asserting
   that the measure was computed, or that its value is well-formed, does not satisfy this — 8.2's
   `--at-tick` wrote `AppExit::error()` into a discarded return and a run that captured nothing
   exited 0.*

### Non-regression — the guards that must still hold

9. The 70–180 valley-floor band still passes at the boot vista, unchanged in both literals
   [capture.rs:430,435]. *This is a regression guard and explicitly NOT proof of this story's fix —
   it read 123 on the frame Wolf called blown.*
10. The flicker still breathes: `flicker_amplitude` stays 0.30 / 0.40, the hand-written band
    literals `(0.70, 1.30)` / `(0.60, 1.40)` [appearance.rs:173-176] and the `torch_peak > 1.20`
    anti-vacuity guard [appearance.rs:185-192] are untouched, and `APPROVED_PEAK` still binds
    [appearance.rs:540]. *6.1's ruling — "flickering works now" [6-1-the-world-moves.md:1178] — is
    an outcome this story must not undo while cutting glare.*
11. The existing capture assertions still hold at the vista: non-black, non-uniform, warm-lit
    pixels ≥ 3,000 [capture.rs:412], and the draw-set / marks oracles.

### Measured on the vehicle

12. Sustained fps is read from the F3 overlay at working zoom and at full vista **with shadows
    enabled**, and both NFR6 bars are met (**60 fps working zoom, 30 fps full vista**). *A failed
    reading is the finding and is reported, never worked around. 8.2 measured ~140 fps, so there is
    2.3×/4.7× headroom — but shadow-casting point lights are the one change in M2 with a real
    chance of spending it, and no figure may be fabricated.*
13. The blown-pool fraction and p99 are re-measured on the vehicle at the boot vista and recorded
    beside the story-creation figures above, from a **controlled pair** — same world, same framing,
    same tick, shadows off then on — so the archive-frames caveat is closed with a real measurement
    and the AC6 ceiling is confirmed or corrected. *If it is corrected, the constant, its doc
    comment and every mutation row anchored on it move in the same commit.*

### Wolf's eye — the closing half, which no agent can check

14. Wolf has viewed the built result live on the vehicle, compared it against
    `5-4-signoff/candidate-artifact-2026-08-15.png`, and judges that the fire reads as **light on
    snow, not glare** — and that things adjacent to it (dwarves, marks, the hover slab) are
    discernible (UX-DR15, UX-DR10, UX-DR22 closing half). This closes `deferred-work.md:943` at its
    root, or reopens it as evidence this story did not finish the job.
15. The recorded observation *"the hover slab is not visible near the campfire"*
    [deferred-work.md:880-884] is re-checked at the same sitting and either closed or restated with
    what remains. *Its rendered judgement belongs to 9.2; what 9.1 owes is whether the campfire was
    the cause.*

### Evidence

16. A sabotage table for this story exists at
    `_bmad-output/implementation-artifacts/mutations/9-1-the-frame-stops-blowing-out.sh`, every row
    KILLED, zero APPLY-FAILED, re-run **after the last refactor** rather than the last feature.
    `python3 scripts/audit-mutations.py` runs clean over every table.

## Tasks / Subtasks

- [x] **Task 1 — The blown-pool measure and its ceiling (AC: 5, 6)**
  - [x] Add the measure beside `median_ground_luminance` in `crates/gui/src/capture.rs` — a pure
        function over `&[[u8; 4]]` returning the **largest connected region** of pixels at or above
        a named threshold, as a fraction of the frame, reusing the existing `luminance` helper
        [capture.rs:437-439]. Add the p99 alongside it. Four-connectivity and an iterative flood
        fill; a 1280×720 frame is 921,600 pixels, so **no recursion** — stack depth is a real
        failure mode here, not a hypothetical.
  - [x] **Threshold 200, ceiling from `boot7.png`.** At 200 the star colour `(173,196,220)` sits at
        luma 192.9 [appearance.rs:43], i.e. **just below**, so the star shell is largely excluded by
        construction — that is why 200 and not 190. The ceiling is `boot7.png`'s **0.6651 %**;
        today's frame reads **0.9883 %**, so the assertion is RED at `baseline_commit`. Confirm both
        numbers yourself before writing them into a constant.
  - [x] If you move the threshold, **re-measure every row of this story's table** and say what the
        new numbers are. A threshold change silently re-bases the ceiling.
  - [x] Name the constants in the same style as `GROUND_LUMINANCE_FLOOR` / `_CEILING` and give each
        a doc comment carrying the measured figure it came from, as those two do
        [capture.rs:425-435].
  - [x] **Hardcoded constants are fine** (technical-preferences). No threshold config, no builder,
        no trait.

- [x] **Task 2 — Wire it into the capture's range check (AC: 5, 9, 11)**
  - [x] Print the numbers in `validate_capture_ranges` alongside the existing line
        [capture.rs:1004], **before** any assertion, so a failing run still reports what it saw.
  - [x] Assert the ceiling **only where the band applies** — `range_band_applies(slice)` is
        `slice.level() >= slice.top()` [capture.rs:981-983]. A cut below the world top must keep
        skipping, with the number still printed. *7.2 ruled the skip CORRECT: lit cut-face rock is
        not the sky-lit snow 5.4 calibrated against [capture.rs:968-980]. Do not make the cut level
        assert; the working-zoom 15.5 % figure is recorded here as an inherited target for a later
        story, not this story's bar.*
  - [x] **Do not touch** `GROUND_LUMINANCE_FLOOR`, `GROUND_LUMINANCE_CEILING` or
        `WARM_PIXEL_FLOOR`.

- [x] **Task 3 — The instrument's own tests (AC: 7, 8)**
  - [x] Unit tests over hand-built pixel buffers, following
        `a_blown_out_field_fails_the_value_ceiling_that_a_midtone_one_passes` [capture.rs:1141]:
        a blown frame fails the ceiling that a clean one passes, and the cut level stands aside.
  - [x] **The discrimination test (AC7).** Decode `5-4-signoff/boot7.png` and
        `7-2-signoff/7-2-marks-vista.png` with `image` and assert **both** clauses: the blown-pool
        measure straddles the ceiling in the right direction (0.6651 % vs 0.9883 %), **and** the
        existing `median_ground_luminance` does *not* separate them (123.4 on both, well inside
        70–180). The second clause is the one that makes this test worth having — it pins that the
        new measure sees what the old one cannot, so a later edit collapsing them goes red. Paths
        are repo-relative from `CARGO_MANIFEST_DIR`; the crate already decodes PNGs this way in
        `non_background_pixels` [tests/capture.rs:37-50].
  - [x] **The exit-code test (AC8).** The band assertions panic out of a Bevy observer, so a
        failure is exit **101**, while `--at-tick` exhaustion is exit **1** via
        `AppExit::error()` consumed at [ingest.rs:100-107]. Prove the new ceiling actually bites —
        `catch_unwind` around the real `validate_capture_ranges` is the in-repo precedent
        [tests/capture.rs:464-467]. **A recipe that asserts `exit=1` for a band failure is wrong.**

- [x] **Task 4 — Campfire shadows (AC: 2, 3, 4, 10)**
  - [x] `point_light` [project.rs:416-424] sets `shadows_enabled` for the campfire. Decide and
        state whether torches and lanterns follow: the measured defect is the **campfire**, so
        YAGNI says campfire only, and each additional shadow-casting point light costs six cube-map
        faces against NFR6. If you extend it, justify it in one sentence and measure it.
  - [x] Leave a `// NOTE:` naming the limitation you chose, per the simple-over-general rule.
  - [x] A headless test asserts the projected campfire light is shadow-casting and survives the
        next reconciliation pass unchanged — the same guard AC11 gave the flicker
        [6-1-the-world-moves.md:103-105]. Reconciliation re-inserts the component
        [project.rs:583-593], so this is a real regression risk, not a hypothetical.
  - [x] Verify by diff that `appearance.rs`'s table and `project.rs:403`'s emissive are untouched.

- [x] **Task 5 — The sabotage table (AC: 16)**
  - [x] Commit first, then mutate — never `git checkout --` over an uncommitted fix.
  - [x] Rows, at minimum: (a) `shadows_enabled` reverted to the default → the AC2 test goes RED;
        (b) the blown-pool ceiling assertion deleted → AC8's test goes RED; (c) the ceiling
        constant raised past today's 0.9883 % → the calibration test goes RED; (d) the
        threshold constant moved so the measure stops separating the two PNGs → AC7 goes RED;
        (e) the print moved after the assertion → the reporting test goes RED.
  - [x] **Anchor rows on lines this story owns.** Three existing rows already sit on the campfire
        table literals — `5-4-the-cold-boot.sh:64`, `:257`, `:334` — and they survive only because
        AC3 forbids touching those values. If AC3 is ever relaxed, all three die silently. This is
        the stale-sabotage-literal class, 3rd instance and counting.
  - [x] Re-run the table **after the last refactor**, not the last feature, and run
        `python3 scripts/audit-mutations.py`. `scripts/mutate.sh` is **not concurrency-safe** —
        run it alone, and read the exit code before any pipe.

- [ ] **Task 6 — VEHICLE-BOUND: the numbers and the eye (AC: 12, 13, 14, 15)**
  - [ ] Rebuild and re-copy `gui.exe`. **Read `gui build <sha>` off the first line and compare it
        to `git rev-parse --short HEAD`.** A `-dirty` suffix means the SHA does not describe what
        is running.
  - [ ] Take the boot-vista capture, paste the `capture range check:` line, and record the
        blown-pool fraction and p99 beside this story's story-creation figures, from the controlled
        shadows-off/shadows-on pair.
  - [ ] Read sustained fps at working zoom and at full vista from F3, with shadows on.
  - [x] Write the session card for merging into Epic 9's shared sitting, from the worked example at
        `8-2-signoff/task-7-vehicle-runbook.md`. **Write it before the session**, as 8.2 did — it
        is a recipe to be corrected by the session, not a record of one.
  - [x] **Order the card so nothing erases its own evidence.** 8.2's card put the clear drag before
        the only read and destroyed what it meant to check [8-2-...md:914-916].
  - [ ] Wolf's AC13/AC14 judgement. **A dev agent cannot check these boxes.**

- [x] **Task 7 — The gate and the record (AC: 1, 16)**
  - [x] `scripts/gate.sh` full tier on a cold rebuild.
  - [x] Verify AC3/AC4 by diff over **this story's own commit range**, not `main..HEAD`.
  - [x] Update `deferred-work.md`: close `:943` (or restate it), address `:880-884`, and close
        `:619-622`'s point-light-shadow item. *8.2 closed two entries without touching the
        register and the orchestrator, not a review layer, caught it.*
  - [x] Correct `docs/tech-art-guidelines.md:56`, which still says the campfire is **32M lm** —
        stale since 2026-08-22. Also check `:46-63` and `:144-149` against the shipped table.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No intensity, amplitude, range or emissive change** (AC3). Levers Wolf considered and withheld.
  If shadows do not reach the ceiling, **stop and report** — see the stated tension above.
- **No working-zoom band.** The cut-level skip is correct and ruled; the 15.5 % figure is recorded
  as an inherited target, not this story's bar.
- **No hover-slab work.** 9.2 owns it. 9.1 only answers whether the campfire was its cause (AC14).
- **No mark-colour work** (9.3) and **no tree work** (9.4). Note 9.4 pushes valley-floor luminance
  *up* while this story pushes it *down* — the 70–180 band watches both [epics.md:1287-1290].
- **No sim-core, protocol, simd, client-core or tui changes** (AC4).
- **No new dependency.** `image` is already a dev-dependency.

### What already exists (build on it, do not re-derive)

- `median_ground_luminance`, `luminance`, `warm_lit_pixels` and the whole band mechanism
  [capture.rs:408-458, 985-1030] — the new measure is a sibling, not a replacement.
- `range_band_applies` [capture.rs:968-983] — reuse it; do not invent a second level predicate.
- The `AppExit` consumer [ingest.rs:100-107] — already fixed at 8.2's review; do not re-add it.
- `image` PNG decoding in tests [tests/capture.rs:37-50].
- `point_light` and the reconcile attach/detach path [project.rs:416-424, 583-593].
- `gui build <sha>` [ingest.rs:91] — M2-7 is closed; use it instead of mtimes.

### Key decisions & traps

- **The median cannot see this defect and never could.** Stated twice on the record
  [6-2-...md:864-866; 7-2-...md:792-794]. AC8 is a guard, AC5 is the proof.
- **`--z` below the world top silently disables the band.** A recipe that passes `--z` for the AC5
  frame judges nothing and exits 0. The vista capture takes no `--z`.
- **Exit codes differ by failure channel**: 101 for a failed assertion, 1 for `--at-tick`
  exhaustion. Do not write a recipe that expects one number for both.
- **`--capture` requires `--frames N` or `--at-tick N`** [ingest.rs:426-455], and `--frames` is a
  render-rate quantity: ticks observed = frames ÷ framerate × 10, so the same command saw 58 ticks
  on a light scene and 237 on a heavy one [7-2-...md:796-800]. Prefer `--at-tick`.
- **`--expect-work` gates the non-zero-work assertions and no default run enables it**
  [capture.rs:116-133]. Do not assume a default capture checks what you think it checks.
- **The artifact is a software mock, not a Bevy capture** [5-4-signoff/README.md]. 5.4 nevertheless
  calibrated the ground median against it and converged to 0.5 % [5-4-...md:1211-1216], so the
  method is sanctioned by precedent — but AC12 exists to close the caveat with a vehicle number.
- **A checkbox is worth only what its verification is worth.** 6.1 had four subtasks ticked without
  being delivered, and they were the seam ACs the story existed to protect.

### Previous story intelligence (deltas that change THIS story)

- **8.2 broke the inert-seam curse by requiring a test that ends at bytes on a socket.** AC7 is the
  same shape one level over: the assertion must change the process's exit, not merely compute.
- **8.2's instruments failed where its feature succeeded** — an `--at-tick` exhaustion that exited
  0, and a `--drag` that evaluated `0 == 0`. This story is *mostly* instrument work, so that
  failure mode is its central risk, not a footnote.
- **An instrument that reads state after the fact cannot see a short-lived observable**
  [8-2-...md:1024-1030]. Not a hazard here — luminance is a still-frame property — but it is why
  AC12 asks for a number rather than an impression.

### Project Structure (files to touch)

| file | NEW/UPDATE | what |
| --- | --- | --- |
| `crates/gui/src/capture.rs` | UPDATE | blown-pool measure + p99, constants, print, ceiling assertion, unit tests |
| `crates/gui/src/project.rs` | UPDATE | `point_light` gains `shadows_enabled` — **line 421 area only** |
| `crates/gui/tests/capture.rs` | UPDATE | the discrimination test (AC7) and the exit-code test (AC8) |
| `crates/gui/tests/headless.rs` | UPDATE | the shadow-casting + survives-reconcile test (AC2) |
| `mutations/9-1-the-frame-stops-blowing-out.sh` | NEW | the sabotage table |
| `docs/tech-art-guidelines.md` | UPDATE | correct the stale 32M and the light-budget prose |
| `_bmad-output/.../deferred-work.md` | UPDATE | close `:943`, `:619-622`; address `:880-884` |
| `crates/gui/src/appearance.rs` | **DO NOT TOUCH** | AC3 |

### Verification

**Executed at story creation, 2026-08-28** — the full gate on `b1cd5f9`, clean tree. **Re-run
on the rebased baseline `15b3635` the same day and green again** (77 s, full tier). Both runs are
recorded rather than the second replacing the first: the numbers throughout this story were
measured on `b1cd5f9`, and what makes them still valid is not the re-run but the fact that
`git diff b1cd5f9 15b3635 -- crates/` is **empty** — the rebase crossed scripts and process files
only, so no measurement in this story moved:

```
frostvein gate
  cargo fmt --check           ok
  cargo clippy -D warnings    ok
  cargo test                  ok
  tui has no sim-core edge                ok
  client-core has no sim-core edge        ok
  gui has no sim-core edge                ok
  metrics ledger tests        ok
  mutation tables still apply ok
GATE GREEN
```

**Also executed at story creation** — the measurement AC5/AC6/AC7 rest on, run against the
committed PNGs with the instrument's own luma. It produced the table at the top of this story. The
two figures that matter: the blown pool reads **0.6651 %** on `boot7.png`, the frame Wolf approved,
and **0.9883 %** on the current vista — while the ground median reads **123.4 on both**. A working
median of **230.5** on the working-zoom frame reproduces 7.2's recorded 231, which is what shows the
method is sound rather than the agreement being luck.

**The first draft of this story calibrated against the approved artifact and that was wrong** — it
is a software mock, whiter than every Bevy frame at threshold 250 and darker at 200, so the
comparison would have measured the renderer. Corrected before saving: the bar is Bevy against Bevy.
The remaining caveat is stated in full above and is AC13's job to close.

**Not executable at story creation — no devpod can open a window.** The obligation is inherited:
run each of these and paste the non-zero observation named beside it.

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# 1. Headless (AC 2, 5, 6, 7, 8) — name the assertions covered, not just "passed"
cargo test -p gui capture
cargo test -p gui shadow

# 2. Sabotage table (AC 16) — commit first; run alone; exit code before any pipe
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/9-1-the-frame-stops-blowing-out.sh
python3 scripts/audit-mutations.py
cargo clean -p gui

# 3. The gate (AC 1) — full tier
scripts/gate.sh
```

Vehicle side (Task 6), after the mandatory rebuild:

```bash
# WSL
cargo build -p simd -p tui
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
./target/debug/simd 7451          # port is positional; seed is fixed in the binary

# Windows, after copying target/x86_64-pc-windows-gnu/release/gui.exe across.
# NO --z: any cut below the world top disables the band AND the new ceiling.
gui.exe 7451 --capture 9-1-vista.png --at-tick 20
echo "exit=$?"

# The counter-test that proves the instrument still bites through the other channel
gui.exe 7451 --capture 9-1-should-not-exist.png --at-tick 100000 --frames 30
echo "exit=$?"     # must be 1, with the exhaustion line on stderr and no PNG
```

**Required observations, not exit 0.**

1. The first line of every run reads `gui build <sha>` and matches `git rev-parse --short HEAD`.
2. The `capture range check:` line prints **both** the existing `ground-median-luminance` and the
   new blown-pool fraction and p99. Match by **prefix** — 7.1 changed an oracle's line shape and
   older recipes quoting whole lines stopped matching.
3. The blown-pool fraction is **below the ceiling**, and the ground median is still inside 70–180.
   Record both. A run whose output shows `assertions skipped` has judged nothing.
4. **The controlled pair for AC13**: the same capture command run twice against the same daemon at
   the same `--at-tick`, once with shadows disabled and once enabled. Two blown-pool numbers from
   one world state. This is the only measurement in the story that is a real A/B.
5. Sustained fps at working zoom **and** full vista, read from F3, with shadows on. Two numbers.
6. Wolf's words on AC14 and AC15.

### Branch and commits

Branch `9-1-the-frame-stops-blowing-out`, cut from `main`. Author every commit
`Völundr <jeicei75@gmail.com>`. **Commit at minimum once per completed task, ideally on each
green** — never one squashed commit; the pre-commit hook runs `scripts/gate.sh --fast` and the
pre-push hook runs the full gate. Review-gated: **no push, no PR** until Wolf says so.

### If this overruns one session

**Split at Task 4.** Tasks 1–3 are a complete, observable slice: the instrument gains a measure
that separates the approved artifact from the shipped frame, calibrated and sabotage-proved, with
nothing rendered yet. Tasks 4–7 (the lever, the table, the vehicle, the gate) become the
continuation. **Restate the RED evidence in the continuation handoff** — 1.2 lost it across a
session boundary.

**Self-gate findings land in the Dev Agent Record, fixed or not** (M2-10). A finding that exists
only in a handback message is lost at the session boundary.

### References

- Epic + ACs: `_bmad-output/planning-artifacts/epics.md:1170-1217` (Epic 9, Story 9.1); ordering
  ruling `:313-320`; UX-DR10/15/22 `:194, :205, :218`
- The stale premise and ruling (d): commit `57e468e`; `crates/gui/src/appearance.rs:64-80`
- The surviving defect: `deferred-work.md:943-947`, `:880-884`, `:619-622`
- Why the median is blind: `6-2-lanterns-in-the-dark.md:864-866`; `7-2-read-the-working-zoom.md:787-795`
- The look baseline and its method: `5-4-the-cold-boot.md:1005-1015, :1211-1216`;
  `5-4-signoff/README.md`; `docs/tech-art-guidelines.md:126-133`
- 6.1's flicker ruling (intent to preserve): `6-1-the-world-moves.md:1124-1133, :1178`
- The vehicle: `vehicle-session-runbook.md:17-44`; `8-2-signoff/task-7-vehicle-runbook.md:41-55`
- Standing art rule: memory `art-gates-visual-judgement`; `epic-5-retro-2026-08-23.md:279-281, :399-401`

## Dev Agent Record

### Agent Model Used

gpt-5.6-terra / high

### Debug Log References

- RED Task 1: `cargo test --offline -p gui capture::tests::blown_pool_uses_the_largest_four_connected_region_and_reports_p99` failed with `E0425` for the two absent measure functions. GREEN after the iterative four-connected flood fill and p99 implementation.
- Calibration: committed PNG decoding measured boot7 at **0.66514754%** (reported 0.6651%) and p99 **216.7**; current vista at **0.98828125%** (reported 0.9883%) and p99 **225.6**. Both ground medians are independently pinned at 123 (the existing integer-rounded instrument value for the recorded 123.4).
- RED Task 2: a warm, midtone 64x64 frame with a 20x20 near-white pool returned `Ok(())` at full depth before the new ceiling; after implementation it panicked while the same cut-level frame returned `Ok(())` and printed its metrics.
- RED Task 3: compilation first failed for the absent reporting seam; the completed tests prove the committed PNG discrimination, a real `validate_capture_ranges` panic (the observer's app exit path is 101), and reporting before that panic.
- RED Task 4: Bevy 0.19 rejected the story's generic field spelling (`E0609`: `shadows_enabled` absent; its `PointLight` field is `shadow_maps_enabled`). The corrected test then failed `the campfire light must cast shadows` before implementation and passed after campfire-only shadow maps.
- Mutation runner: rows (a) and (b) were KILLED. The devpod execution layer terminated two full-table runs mid-table (first after row b; second after beginning row c/d), briefly leaving mutations that were verified and restored. `python3 scripts/audit-mutations.py` is clean (398 rows). The full table exit code was therefore not observed; Task 5 remains unchecked.
- Full cold gate: `cargo clean -p gui` removed 3,728 files / 14.9 GiB; the environment then terminated `scripts/gate.sh` after `cargo fmt --check ok`, `cargo clippy -D warnings ok`, and the start of `cargo test`. No full-gate result was observed.
- Self-gate pass 1: `codex review --base main` started (session `01a0484d-1ad2-70c0-87da-847011d5bfc9`) but the execution layer terminated it while it was inspecting the diff; it produced no final findings report. No second pass was run.

**Orchestrator (Claude) completion of Tasks 5 and 7, 2026-08-28** — Codex's three interrupted runs
were re-run here rather than left as claims:

- **Cold full gate: GREEN.** `cargo clean -p gui` (3,095 files / 13.5 GiB) then `scripts/gate.sh`
  full tier: fmt ok, clippy -D warnings ok, `cargo test` ok, all three crate-edge probes ok,
  metrics ledger ok, mutation tables still apply ok. **AC1's headless half is met.**
- **Sabotage table: 5 rows, 5 KILLED, 0 APPLY-FAILED**, `python3 scripts/audit-mutations.py`
  clean (398 rows). **The batch run was harness-killed twice** — the known `mutate.sh` failure
  mode — so rows were split one-per-file and run individually against a warm target, which is the
  recorded workaround. The tree was verified clean after each row. Which assertion caught each:
  (a) `campfire_light_casts_shadows_and_survives_a_later_reconciliation`;
  (b) `blown_pool_range_failure_is_a_real_panic_not_a_successful_capture`;
  (c) the ceiling-constant pin at `tests/capture.rs:190`;
  (d) the discrimination clause `current_pool > 0.006_651_5` at `tests/capture.rs:192`;
  (e) `capture_range_report_is_emitted_before_a_blown_pool_panic`.
  **Worth a reviewer's eye:** row (c) is killed by the constant *pin*, not by any loss of
  discrimination — the separating asserts use hard literals, so a raised ceiling changes nothing
  about them. The pin is the only guard on that constant, which is the intended discipline, but it
  is a weaker kill than (d)'s.
- **Named assertions, not "passed"**: `committed_bevy_vistas_show_the_blown_pool_that_ground_median_cannot_see` ok,
  `blown_pool_range_failure_is_a_real_panic_not_a_successful_capture` ok,
  `campfire_light_casts_shadows_and_survives_a_later_reconciliation` ok,
  `blown_pool_uses_the_largest_four_connected_region_and_reports_p99` ok,
  `blown_pool_ceiling_judges_the_boot_framing_and_stands_aside_at_a_cut` ok,
  `capture_range_report_is_emitted_before_a_blown_pool_panic` ok.
- **The calibration was measured three independent ways and all three agree.** The instrument
  prints `boot pool=0.66514754% p99=216.7; current pool=0.98828125% p99=225.6`; an orchestrator-side
  pure-Python PNG decoder written without reference to the Rust code read
  `boot7 0.6651% / p99 216.7 / ground-median 123.4` and `7-2-marks-vista 0.9883% / p99 225.6 /
  ground-median 123.4`; both reproduce the story-creation table. **The ceiling is genuinely RED at
  `baseline_commit` and the ground median genuinely cannot separate the two frames.**
- **AC3/AC4 verified by diff over `15b3635..HEAD`** (this story's own range, not `main..HEAD`):
  `crates/gui/src/appearance.rs` does not appear in the diff at all, `project.rs`'s only change is
  the three-line `shadow_maps_enabled` addition (emissive at `:403` untouched), and none of
  `protocol`, `sim-core`, `simd`, `client-core`, `tui` appear.
- **`shadow_maps_enabled` is the correct field, verified against the dependency rather than
  assumed**: Bevy 0.19.0 (this workspace's pin) gates shadow-map extraction on it at
  `bevy_pbr-0.19.0/src/render/light.rs:463`. The story's `shadows_enabled` spelling was stale;
  Codex's correction is right, and its Debug Log's "Bevy 0.16" was corrected to 0.19 here.
- **Codex's exit 0 was not trusted.** The handoff wrapper's `401` warning fired on nine false
  positives (source line numbers, a rollout session id, git blob hashes) — no auth failure. Seven
  commits, all authored `Völundr <jeicei75@gmail.com>`, one per task: the cadence floor held.
- **NOT DONE, and not claimable here: the self-gate.** Codex's single `codex review --base main`
  pass was killed before producing findings, so this story carries **zero self-gate coverage**
  against a cap of three. That is a real hole in the evidence, not a formality.

### Completion Notes List

- Tasks 1–4 complete in four local Völundr commits. The capture instrument uses hard-coded threshold 200, an iterative four-connected flood fill, p99, and the exact ceiling 0.006651476 (0.6651% rounded from boot7's pixel count). The current committed vista remains above it at 0.9883%.
- Campfire-only point-light shadows preserve every table/emissive look lever; torches and lanterns remain unshadowed by deliberate YAGNI/performance scope.
- Task 6 is vehicle-bound and intentionally unchecked. Its pre-session, evidence-preserving card is at `9-1-signoff/task-6-vehicle-runbook.md`; no fps, controlled-pair, or Wolf judgement was fabricated.
- Task 5 and Task 7 remain unchecked because the runner and cold full gate were externally interrupted. Status remains `in-progress`; AC6's shadows-enabled vehicle result is unavailable, so it is neither claimed green nor claimed red.
- **Tasks 5 and 7 were completed by the orchestrator after the handback** (see the Debug Log block
  above): cold full gate GREEN, all five sabotage rows KILLED, audit clean. Tasks 1-5 and 7 are
  therefore complete and the story is handed to review.
- **Task 6 stays open and its ACs (12, 13, 14, 15) are UNMET.** They are vehicle-bound to
  gingerspice and no devpod can open a window, so they are physically unrunnable here; Epic 9
  spends one shared sitting for all four of its stories. The story goes to review with this stated
  rather than blocked behind a session that has not happened. **AC6's on-vehicle outcome is
  therefore neither green nor red** — the ceiling is proved RED at baseline on committed pixels,
  but whether shadows alone bring it under the bar is exactly what the controlled shadows-off/on
  pair must answer.
- **The stated W1+W2 tension is still live and must not be resolved by anyone quietly.** If the
  vehicle pair shows the ceiling still exceeded with shadows correct and NFR6 met, that is the
  story's finding and it gets reported — not fixed by opening intensity, amplitude, range or
  emissive, which Wolf withheld.

### File List

- _bmad-output/implementation-artifacts/9-1-signoff/task-6-vehicle-runbook.md
- _bmad-output/implementation-artifacts/9-1-the-frame-stops-blowing-out.md
- _bmad-output/implementation-artifacts/deferred-work.md
- _bmad-output/implementation-artifacts/metrics/.session-cursors.json
- _bmad-output/implementation-artifacts/metrics/9-1-the-frame-stops-blowing-out.md
- _bmad-output/implementation-artifacts/mutations/9-1-the-frame-stops-blowing-out.sh
- _bmad-output/implementation-artifacts/sprint-status.yaml
- crates/gui/src/capture.rs
- crates/gui/src/project.rs
- crates/gui/tests/capture.rs
- crates/gui/tests/headless.rs
- docs/tech-art-guidelines.md

## Change Log

| date | change |
| --- | --- |
| 2026-08-28 | **Orchestrator verification and completion of the delegated run.** Codex's exit 0 was not trusted: the wrapper's `401` warning was nine false positives, seven commits were confirmed authored `Völundr` with the per-task cadence held, and the File List was diffed against `15b3635..HEAD`. Its three harness-killed runs were re-run here — **cold full gate GREEN**, **5 of 5 sabotage rows KILLED** with the audit clean (the batch runner was killed twice, so rows were split and run individually against a warm target), and every story test named individually rather than reported as "passed". The calibration now rests on three independent measurements that agree to the digit: the Rust instrument, an orchestrator-side pure-Python PNG decoder, and the story-creation table. Two record defects corrected: the Debug Log said Bevy 0.16 where the workspace pins 0.19 (the `shadow_maps_enabled` rename was verified against `bevy_pbr-0.19.0/src/render/light.rs:463`, so Codex was right and the story's field name was stale), and a mangled line wrap in `tech-art-guidelines.md`. **Two holes are carried into review rather than papered over: the self-gate produced ZERO passes (killed mid-run, against a cap of three), and Task 6's vehicle ACs 12-15 are unmet and unrunnable in a devpod.** Status -> review. |
| 2026-08-28 | **Rebased onto `15b3635`** after PR #38 (the build-cache reaper) merged, and `baseline_commit` moved with it. The rebase is what makes AC1 answerable: every M2 story is stacked, and an AC that names a range from a stale baseline is wrong by default — this story's own commit range must start at its true parent. Verified rather than assumed: `git diff b1cd5f9 15b3635 -- crates/` is **empty**, so the 0.6651 % / 0.9883 % / 123.4 figures measured at creation all still hold, and AC3's byte-identical claim and AC4's untouched-crates claim are unaffected. Full gate re-run on the new baseline and green. |
| 2026-08-28 | Story created. Baseline `b1cd5f9`, full gate green at creation. **The epic's premise was falsified against source**: ruling (d) already landed the peak at 35.0M under the 35.52M pin on 2026-08-22, and Wolf re-confirmed the blow-out on 2026-08-27 anyway — so the reason in the epic text is wrong while the decision stands. **The epic's AC1 was found vacuous**: the 70–180 median band reads 123 on the very frame called blown, measured on the record at 6.2. Three rulings taken from Wolf (W1 local blow-out measure, W2 shadows and only shadows, W3 no Task 0). The blown-pool measure was calibrated at creation against the committed PNGs and the ceiling rests on measurement rather than estimate: **0.6651 % on `boot7.png`, the frame Wolf approved, against 0.9883 % today**, while the ground median reads 123.4 on both. A first draft calibrated against the approved artifact and was corrected before saving — the artifact is a software mock whose near-white distribution has a different shape, so that comparison would have measured the renderer rather than the defect. The remaining caveat (archive frames, not a controlled A/B) is stated in the story and assigned to AC13. |
| 2026-08-28 | Implemented Tasks 1–4: calibrated blown-pool/p99 instrument, range ceiling, discriminating and exit seams, and campfire-only shadow maps. Added the Task 6 vehicle card and corrected the stale art/deferred records. Task 5's full mutation run and Task 7's cold full gate were interrupted by the devpod execution layer; recorded honestly and left unchecked. |
