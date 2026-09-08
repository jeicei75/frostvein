---
baseline_commit: 3ed269c
model: claude-fable-5-1  # the session's harness model, not the Opus default; recorded so the ledger row is readable
---

# Story 10.8: Lighting and Atmosphere, Re-judged Under the Sun

Status: ready-for-dev

**RUNS BEFORE 8.3.** Wolf, 2026-09-08: *"I think we will create lighting and overall
atmosphere/style story first."* The 2026-08-28 ruling (look work ahead of the six-bar sign-off)
still governs; the board's numeric order does not. 8.3 stays `backlog` behind this story.

Created 2026-09-08 and **added to `epics.md` § Epic 10 the same day** (the last section).

## Story

As the boss,
I want the valley's light and air re-judged under the sun the game actually ships with,
so that the look I sign Milestone 2 off against was tuned for this lighting, and not for a world
lit by something else.

## Premises verified at creation — 2026-09-08, on `main` 3ed269c

**Executed, not read.** The control capture was run on a build stamped `gui build 3ed269c`.

1. **The near-white guard is RED on today's `main`, and the frame is filed.**
   `gui <port> --headless --frames 160 --subdiv 1 --capture` at the boot framing:
   ```
   capture range check: warm-lit pixels=21037 ground-median-luminance=135 near-white-area=2.1686% blown-pool=1.1077% p99-luminance=229.4
   near-white area is 2.1686%, above the 1.5630% ceiling calibrated on boot7.png      EXIT=101
   ```
   Frame: `10-8-signoff/creation-control-main-3ed269c-boot-a.png` (run b beside it). The PNG is written before the
   check panics (`capture.rs:1261`, `save_then_validate`), so exit 101 costs no evidence.
2. **The directional's strength has never been judged as light on the world.** The below-horizon
   aim landed 2026-08-15 (`7e08862`); `directional_illuminance: 22_000` landed the NEXT DAY
   (`10c06e1`). Between then and 10.7 (2026-09-03, `5269abf`) that light reached no surface —
   deleting it moved the frame less than same-build noise (`10-7-signoff/README.md`). 10.7 raised
   it and, by ruling, re-tuned nothing. **22,000 lux is an unjudged number.**
3. **The capture ceilings' calibration frame no longer represents the game.** `boot7.png`
   (`5-4-signoff/`) was rendered with the sun under the map; `NEAR_WHITE_AREA_CEILING`,
   `BLOWN_POOL_FRACTION_CEILING` and the ground band are its figures to the digit
   (`capture.rs:560-596`). Filed in `deferred-work.md` § "10.7, the near-white ceiling's
   calibration frame is gone": *re-deriving means picking a new reference frame under the sun, not
   scaling the old number.* That is this story.
4. **The emitters have never been sized separately, and now can be.** `--lights-off <list>`
   (`ingest.rs:897-905`, `LightSource::from_name` `:121`) starts a headless capture with named
   sources dark; `apply_lighting_toggles` (`ingest.rs:1239-1290`) zeroes the light AND blacks the
   emissive face (`project.rs:598-620`). Measured 2026-09-03 with it (`deferred-work.md` § "THE
   CAMP'S WARM SIGNATURE IS MOSTLY TORCHES"): four torches at 56M lm outweigh the 25M campfire
   **4.5:1 on warm-lit pixels**; the blown white core is the campfire's 2:1; the near-white breach
   splits ~0.40 pp torches / ~0.34 pp campfire. A bad source name is refused, observed today:
   `Error: unknown light source "bogus"; expected sun, campfire, torches, lanterns, or ambient`.
5. **Exposure and tonemapping are Bevy's defaults, set by nobody.** The camera is
   `Camera3d::default()` (`ingest.rs:1124`) with no `Exposure` or `Tonemapping` component, so the
   frame runs at `Exposure::BLENDER` = EV100 9.7 (`bevy_camera-0.19.0/src/camera.rs:263,279-283`)
   through `Tonemapping::TonyMcMapface` (`bevy_core_pipeline-0.19.0/src/tonemapping/mod.rs:157`)
   with real LUTs — `tonemapping_luts` is not in the workspace feature list (`Cargo.toml:32-36`)
   but `cargo tree -p gui -e features -i bevy_core_pipeline` shows it enabled transitively through `3d_api`, and no
   frame is magenta (the placeholder LUT Bevy would substitute, `tonemapping/mod.rs:446`). Lux
   reaches a pixel only through exposure and the tone curve; a lighting table, an exposure and a
   tonemapper are three knobs for one picture, and this story must say which it turns (Ruling 1).
   Do NOT add `tonemapping_luts` to `Cargo.toml` "to be safe" — it is already on, and any change
   to the `bevy` feature list rebuilds ~400 crates.
6. **The Blender bench cannot be the artifact venue for intensities.** `valley_bench.py:83-97`
   records that Bevy's brightness/illuminance and Cycles' strength/energy *"share no units"*; the
   bench's `AMBIENT_STRENGTH = 3.3` / `SUN_ENERGY = 21.0` were set by eye. The bench stays pinned
   for COLOURS and AIM through `bench_contract.rs:88-108, 170-176, 216`, and those anchors move in
   lockstep when a colour does.
7. **Epic 10's two inherited eye-checks were never taken.** No 10.x story record carries them
   (`rg -i 'inherited eye-check' _bmad-output/implementation-artifacts/10-*.md` → only 10.3's
   reference). They were parked on *"the withheld-levers story runs first; if it dims the fire,
   this may close for free"*. The hover slab is `(80,220,210)` (`appearance.rs:138`), luminance
   189.5, against a camp pool above 200.
8. **The four torches alone account for the guard's red.** Probed today (Verification table):
   `--lights-off torches` reads near-white 1.3342 % and blown pool 0.2936 % — under BOTH shipped
   ceilings, exit 0 — while `--lights-off sun` stays red at 1.7378 %. 10.7's 2026-09-03 torches-off
   reading (1.85 %, still red) predates the emissive-follows-the-toggle fix, so it measured torch
   lights off with torch FACES still glowing; today's figure is the first with both dark. The
   campfire's own share is not isolated here (AC3 does it, torches off). Start the table at the
   torch ring, as `deferred-work.md` already advised.

## The frame, read at creation

The control frame reads as a **bright, hard-shadowed daylight snowfield under a night sky**: pale
blue ground, long tree shadows, a camp floor blown to flat white. The PRD's frame is *"a dark blue
night world"* whose sky is the illuminant and whose snow *"stays midtone blue-grey; only emissive
light approaches white"*. The tech-art doc still describes the key as a *"green-blue directional
light [that lets] the aurora catch snow and ice"* — a night key, not a sun. Premise 2 says why: the
key's strength was never chosen as light.

## Rulings owed from Wolf — the opening sitting, BEFORE any table is drafted

1. **Is the key light a SUN or NIGHT LIGHT (aurora/moon), and how bright may a night key be?**
   This decides whether the table is re-derived toward the PRD's dark night or toward the daylit
   read the control shows. Also: intensities or one `Exposure` component — pick ONE knob family.
   Do not inherit "sun" from 10.7's title as a decision; 10.7 ruled elevation only.
2. **The snow-flank question** (`deferred-work.md` § "THE TWO RENDER PATHS DISAGREE ABOUT SNOW'S
   FLANKS"): IN or OUT. It is a material rule, not a light. Recommended OUT unless Wolf has decided
   what he wants.
3. **Issue #62** (pin `docs/tech-art-guidelines.md` Critical values in `bench_contract.rs`, ~10
   anchored rows): IN or OUT. Recommended IN — this story rewrites exactly those rows.
4. **Sky, fog, rim: which defects, if any, does Wolf name on the control frame?** Only a named
   defect with the frame that shows it opens one of these for change (AC9).

**Fixed, not owed:** the sun's elevation `+17.66°` and azimuth `40.0398°` stay
(`atmosphere.rs:35,39`; Wolf, 2026-09-03). `the_approved_sun_lights_downward` stays as is.

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier) is green, and the diff is confined to this story's own
   commit range `3ed269c..HEAD` — the branch is cut from `main`, not stacked.

### The rulings and the artifact

2. Rulings 1–4 above are recorded in this file with date and Wolf's words, and **no lighting
   constant is changed before Ruling 1 is recorded.**
3. **Per-emitter marginal table.** For each of sun, ambient, campfire, torches, lanterns: one
   headless capture at the boot framing with that source alone off via `--lights-off`, plus
   campfire off with torches ALSO off; each row carries the `capture range check:` line
   (`capture.rs:1353`) and `lumstats.py` mean, beside the same-build noise floor (two all-on
   runs, WORST). Filed as `10-8-signoff/AC3-marginals.md`.
4. **Opening artifact (UX-DR22 opening half).** At least two candidate lighting tables and the
   shipped control, each captured by `gui --headless` at the boot framing on THIS story's build,
   each PNG filed in `10-8-signoff/` with its range-check line and lumstats figures. A candidate's
   figures differ from the control's beyond the noise floor. Wolf chooses; the choice is recorded
   with artifact filename, figures, who and when. Candidate edits are **uncommitted and reverted**
   — nothing lands before the choice.

### The fix

5. The chosen table lands; `gui <port> --headless --frames 160 --subdiv 1 --capture <png>` at the
   boot framing **exits 0 on two consecutive runs** (near-white swings ~0.1 pp between runs).
6. **Every capture constant that moves is derived from the approved treatment's own two
   same-build runs — the WORST reading plus that pair's measured swing — in the same commit as the
   table, with both readings in the constant's doc comment.** This deliberately departs from 9.1's
   "the approved frame sits exactly AT the bar": headless near-white swings ~0.04–0.1 pp between
   runs of one build (2.1686 % / 2.2088 % today), so a to-the-digit ceiling fails AC5 by
   construction. Both approved-run frames are committed under `10-8-signoff/` and
   `committed_bevy_vistas_show_the_blown_pool_that_ground_median_cannot_see`
   (`crates/gui/tests/capture.rs:158`) is extended: both approved runs at or under each ceiling,
   `creation-control-main-3ed269c-boot-a.png` ABOVE `NEAR_WHITE_AREA_CEILING`, behavioural asserts
   first and the pin second — 9.1's order. If the control does NOT sit above the new ceiling, the
   chosen table did not move this metric and the story says so rather than widening the gap by
   hand. A constant raised without a frame is the defect 10.7's AC7 forbids.
7. `appearance_tables_pin_the_cold_boot_palette` and
   `campfire_keeps_local_contrast_over_the_midtone_cold_fill` (`appearance.rs:317, 571`) are
   corrected to the new values, not loosened: the 1.2×–6.0× band and the R/B ≥ 2× ambient term
   still hold. If the campfire peak changes, `APPROVED_PEAK` (`appearance.rs:591`) is re-ruled by
   Wolf and re-pinned, never deleted.
8. Every changed colour changes in `scripts/bench/valley_bench.py` and both `bench_contract.rs`
   anchors in the SAME commit; `bench_contract.rs` stays green with each anchor matching exactly
   once.
9. Sky `(5,12,28)`, aurora, `fog_falloff` (`ingest.rs:1412`) and the rim dissolve change only
   where Ruling 4 named a defect, each change citing the frame that showed it.
10. `docs/tech-art-guidelines.md` § Lights, § Value ladder and § Sky and lights state the new
    values and the rule that changed, in the same commit as the code. If Ruling 3 is IN: the
    Critical values rows are anchored in `bench_contract.rs`, exactly once each, and one row is
    shown to fail when the doc value drifts.
11. `_bmad-output/implementation-artifacts/mutations/10-8-lighting-and-atmosphere-re-judged-under-the-sun.sh`
    carries at least **four rows the mutation run kills**, including: restore the shipped
    `directional_illuminance` (killed by the corrected palette pin); move `NEAR_WHITE_AREA_CEILING`
    off the approved figure (killed by AC6's test); diverge a bench colour from the client (killed
    by `bench_contract.rs`).

### Sign-off

12. **Closing half (UX-DR22).** Wolf views the built result live on the vehicle against the
    approved frame, states whether *"lighting is still way off"* is still true, and the two
    inherited eye-checks (hover slab on a vertical face near the fire; mark modes apart at a
    glance) are each closed or reopened on the record. If "way off" is still true, that is this
    story's finding and 8.3 does not start.

## Tasks / Subtasks

- [ ] **Task 1 — The rulings sitting** (AC: 2)
  - [ ] Hand Wolf: the control frame, its range-check line, Premise 2's dates, "The frame, read at
        creation", and Rulings 1–4 as questions. Record the answers verbatim with the date.
  - [ ] **Stop here until Ruling 1 is recorded.** No lighting constant moves before it.

- [ ] **Task 2 — Per-emitter marginals** (AC: 3)
  - [ ] Check the stamp first: `target/debug/gui --version` must print the branch HEAD with no
        `-dirty`; `touch crates/gui/build.rs` before the build if it lags.
  - [ ] Captures, one daemon (`simd 0`, read the port from its `listening on` line):
        all-on ×2, then `--lights-off` for `sun`, `ambient`, `campfire`, `torches`, `lanterns`,
        `campfire,torches`. Read each with
        `python3 _bmad-output/implementation-artifacts/10-7-signoff/lumstats.py <png>=<label>`.
  - [ ] Table: source off · warm-lit · ground median · near-white · blown pool (diagnostic only) ·
        mean · Δmean vs all-on · ×noise. Torches and campfire rows state the marginal with the
        other OFF. Commit as `10-8-signoff/AC3-marginals.md` with the PNGs.

- [ ] **Task 3 — Candidates, captured by the client** (AC: 4)
  - [ ] Each candidate = an edit to `night_lighting()` / `light_properties()` (and, if Ruling 1
        chose exposure, one `Exposure { ev100 }` on the camera at `ingest.rs:1124`), built,
        captured at the boot framing ×2, **reverted**. Name PNGs by what changed
        (`candidate-<what>-a.png`), never by verdict.
  - [ ] Every PNG filed with its range-check line and lumstats figures; the control's two runs give
        the noise floor. Present side by side; record Wolf's choice with filename and figures.
  - [ ] **Stop here until Wolf has chosen.**

- [ ] **Task 4 — Land the table, client and bench together** (AC: 5, 7, 8)
  - [ ] `crates/gui/src/appearance.rs`: `night_lighting()` `:40-50`, `light_properties()` `:52-90`.
        Correct the two tests at `:317` and `:571` to the new values. If a colour changes:
        `scripts/bench/valley_bench.py:46-47` and the light-colour literals, plus
        `crates/gui/tests/bench_contract.rs:88-108` — ONE commit.
  - [ ] Two consecutive boot captures exit 0; paste both range-check lines.

- [ ] **Task 5 — Re-calibrate the capture constants on the approved frame** (AC: 6)
  - [ ] Commit both approved-treatment runs as `10-8-signoff/approved-<what>-<sha>-{a,b}.png`.
        Measure them with the production functions (`near_white_area_fraction`,
        `largest_blown_pool_fraction`, the ground median); set each constant in `capture.rs:560-596`
        to worst-of-two plus the pair's swing, and write both readings into the doc comment.
  - [ ] Extend `crates/gui/tests/capture.rs:158`: approved ≤ ceiling, creation control > ceiling,
        then the pin. Run it RED first by leaving the constant at the boot7 figure.

- [ ] **Task 6 — The docs move with the code** (AC: 9, 10)
  - [ ] `docs/tech-art-guidelines.md` § Lights table, § Value ladder table, § Sky and lights prose.
        Supersede rows in the existing style (`↳ before …` **(superseded)**), do not delete history.
  - [ ] If Ruling 3 IN: anchored rows in `bench_contract.rs` reusing `assert_anchor`, exactly once
        each; show one RED by editing a doc value.

- [ ] **Task 7 — Mutation table** (AC: 11)
  - [ ] ≥4 rows, format per `mutations/10-7-the-sun-lights-the-valley.sh`. **Commit the fix before
        mutating**; run `scripts/mutate.sh` ALONE; re-mutate after any strengthening; record KILLED
        per row naming the mutation. Run `gui --version` afterwards — a mutant build outlives the
        source restore.

- [ ] **Task 8 — Verification and the closing sitting** (AC: 1, 12)
  - [ ] Execute the Verification recipe, RED first; paste outputs into the Dev Agent Record.
  - [ ] Full `scripts/gate.sh` green, pasted. If `pixel_guard.rs` fails with "wrote no PNG", that
        is issue #72 — re-run the guard alone and record both; never read around it.
  - [ ] Vehicle card in the shape of `10-4-signoff/task-6-vehicle-runbook.md`, launched via
        `scripts/launch-gui.ps1` (do NOT use its `--` forwarding — issue #81). It asks for: the
        six-word check on lighting, the hover slab on a cliff face near the fire, the three marks at
        working zoom.

## Dev Notes

### Scope guardrails — do NOT

- **Do not move the sun.** Elevation, azimuth, `the_approved_sun_lights_downward` stay.
- **Do not touch the snow-flank rule unless Ruling 2 is IN**; do not touch camera, composition or
  `fog_falloff` unless Ruling 4 named a defect.
- **Do not raise a capture ceiling to clear a panic.** A constant moves to a frame's figure or not
  at all (AC6).
- **Do not use the Blender bench as the artifact** for intensities (Premise 6). Colours and aim
  stay pinned to it.
- **No config, no CLI flag, no second toggle scheme.** `--lights-off` and F5–F9 are the instrument;
  an `Exposure` component, if ruled, is one hardcoded constant.
- **Do not pause the world for captures** — the motion-health floor panics before the PNG. Do not
  use `--at-tick`. Live with dwarf-motion noise and measure it.
- **Do not weaken a test to make a capture exit 0.**

### What already exists — build on it

- **Light table and tint:** `night_lighting()` `appearance.rs:40-50`; `light_properties()` `:52-90`
  (campfire 25M/0.40/0.9 Hz, torch 14M/0.30, lantern 5M/0.05); `flicker_scale` `:93`.
- **Instrument:** `--lights-off` + F5–F9 (`ingest.rs:90-200, 897-905, 1239-1290`); range-check
  emitter `capture.rs:1352`; `10-7-signoff/lumstats.py` (mean / dark / shade-band, RGB PNG only);
  the capture recipe in `crates/gui/tests/pixel_guard.rs:130-200` (`Daemon::spawn`, `capture`).
- **Guards:** `capture.rs:538-596` constants and their calibration notes;
  `crates/gui/tests/capture.rs:158` decodes committed frames against the constants.
- **Bench lockstep:** `bench_contract.rs:88-108` (ambient/directional/light RGB), `:170-176`
  (sun), `:216` (`AMBIENT_STRENGTH` use); `valley_bench.py:46-47, 80-81, 96-97`.
- **Docs:** `docs/tech-art-guidelines.md` § Critical values (Lights, Value ladder), § Sky and
  lights, § Edge treatment.

### Key decisions and traps

- **Deltas are not levels.** Every figure is a level with its noise floor beside it; the WORST of
  two same-build runs is the floor (10.4 published a delta inside its noise; 10.7 shipped holes
  reading a delta as a level).
- **Near-white swings ~0.1 pp run to run; the blown POOL is diagnostic only on headless frames**
  (`capture.rs:596` doc comment) — assert AREA, print the pool.
- **Whole-frame mean cannot see a point light.** 10.7's AC11 measured campfire-off at 1.8× noise
  on frame mean while sun-off read 128×. Judge emitters on warm-lit pixels, near-white and ground
  median, not frame mean.
- **Measure torches with the campfire OFF and vice versa.** Premise 4: with torches lit the
  campfire's contribution reads as +300 px — nil.
- **`campfire_keeps_local_contrast…` sums `ambient_brightness + directional_illuminance` as the
  cold fill.** Changing EITHER moves its ratio; correct the expectation with the cause stated, do
  not widen the band.
- **Exposure multiplies everything.** Under EV100 9.7 a table halved and an EV raised one stop are
  the same picture; Ruling 1 fixes which knob turns so the record is comparable.
- **`bench_contract.rs` is a text grep.** It cannot see a wrong number that matches on both sides;
  check the maths.
- **Build parallelism OOMs this devpod** (32 cores / 23 GB); `mutate.sh` caps jobs — never run it
  beside a build or a review layer.
- **A mutant build outlives the source restore.** `gui --version` before every capture; the stamp
  must equal `git rev-parse --short HEAD` with no `-dirty`.
- **Frame-file names carry what changed and the commit, never a verdict** (10.7 committed
  "after-fix" frames that were the rejected fix).
- **Issue #77:** a capture cut below the dwarves panics on motion assertions — keep boot framing
  (`--z` unset) for every figure here.

### Project structure

| Path | NEW/UPDATE | Note |
|---|---|---|
| `crates/gui/src/appearance.rs` | UPDATE | `night_lighting()`, `light_properties()`, the two pin tests |
| `crates/gui/src/capture.rs` | UPDATE | ceilings re-calibrated on the approved frame, doc comments name it |
| `crates/gui/tests/capture.rs` | UPDATE | `:158` extended: approved ≤, control >, then pin |
| `crates/gui/src/ingest.rs` | UPDATE only if Ruling 1 chose exposure | one `Exposure` on the camera at `:1124` |
| `scripts/bench/valley_bench.py` | UPDATE if a colour changes | lockstep with the client |
| `crates/gui/tests/bench_contract.rs` | UPDATE | anchors move; #62 rows if Ruling 3 IN |
| `docs/tech-art-guidelines.md` | UPDATE | Lights, Value ladder, Sky and lights |
| `_bmad-output/implementation-artifacts/10-8-signoff/` | UPDATE | marginals, candidates, approved frame, vehicle card |
| `_bmad-output/implementation-artifacts/mutations/10-8-lighting-and-atmosphere-re-judged-under-the-sun.sh` | NEW | ≥4 rows |
| `_bmad-output/implementation-artifacts/deferred-work.md` | UPDATE | strike the near-white calibration entry when AC6 lands |

### References

- `_bmad-output/planning-artifacts/epics.md` § Story 10.8; § Inherited eye-checks; § UX-DR22
- `_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/prd.md` § The light, § What the
  references bind, § The anti-requirements
- `_bmad-output/implementation-artifacts/deferred-work.md` § "10.7, the near-white ceiling's
  calibration frame is gone" · § "THE CAMP'S WARM SIGNATURE IS MOSTLY TORCHES" · § "THE TWO RENDER
  PATHS DISAGREE ABOUT SNOW'S FLANKS"
- `_bmad-output/implementation-artifacts/10-7-the-sun-lights-the-valley.md` — the bench-then-judge-
  then-land shape, AC11's measurements, the toggles' history; `10-7-signoff/README.md`
- `_bmad-output/implementation-artifacts/9-1-the-frame-stops-blowing-out.md` — how the ceilings
  were calibrated on `boot7.png`
- GitHub issues #75 (the defect), #62 (doc pins), #72 and #77 (capture instrument), #81 (launcher)
- `bevy_camera-0.19.0/src/camera.rs:232-283`; `bevy_core_pipeline-0.19.0/src/tonemapping/mod.rs:157`
- `CLAUDE.md`, `docs/technical-preferences.md`

## Verification

**Executed at story creation, 2026-09-08, on `main` 3ed269c, build stamped `gui build 3ed269c`.**

**RED first — the guard seen failing on the shipped frame** (this is the defect, and it is also the
instrument proving it can say no):

```bash
./target/debug/simd 0 &                     # prints: listening on 127.0.0.1:<port>
./target/debug/gui <port> --headless --frames 160 --subdiv 1 --capture /tmp/control-a.png
```
```
capture range check: warm-lit pixels=21037 ground-median-luminance=135 near-white-area=2.1686% blown-pool=1.1077% p99-luminance=229.4
thread 'main' panicked at crates/gui/src/capture.rs:1393:5:
near-white area is 2.1686%, above the 1.5630% ceiling calibrated on boot7.png        EXIT=101
```

**RED for the per-emitter instrument** — a bad source name is refused, not ignored:
```
./target/debug/gui 1 --headless --lights-off bogus --capture /dev/null --frames 1
Error: unknown light source "bogus"; expected sun, campfire, torches, lanterns, or ambient
```

**GREEN — the instrument discriminates.** Same daemon, same build, boot framing:

| run | `--lights-off` | warm-lit | ground median | near-white | blown pool | p99 | mean | exit |
|---|---|---:|---:|---:|---:|---:|---:|---|
| control a | — | 21,037 | 135 | 2.1686 % | 1.1077 % | 229.4 | 101.114 | 101 |
| control b (noise) | — | 21,988 | 136 | 2.2088 % | 1.1196 % | 230.3 | 101.218 | 101 |
| torches off | `torches` | **9,702** | 122 | **1.3342 %** | 0.2936 % | 208.7 | 99.768 | **0** |
| sun off | `sun` | 21,219 | 117 | 1.7378 % | 1.0318 % | 228.6 | 87.800 | 101 |

**Noise floor, worst of the two controls:** mean 0.104 · near-white 0.040 pp · warm-lit 951 px.
Every probe row moves far outside it, so the instrument discriminates in both directions: the sun
carries the frame's MEAN (−13.3, 128× noise, the ground median 135 → 117) and barely touches
warm-lit; the torches carry the WARM figures (warm-lit −11,335, near-white −0.83 pp) and barely
touch the mean. **Switching the four torches off alone takes the shipped guard GREEN — exit 0 —
with every ceiling at its shipped value.** Frames: `10-8-signoff/creation-control-main-3ed269c-boot-{a,b}.png`,
`creation-probe-torches-off-3ed269c.png`, `creation-probe-sun-off-3ed269c.png`. These are probes
for the record, not candidates: a candidate is a TABLE Wolf chooses from, and dark torches are a
decision he has not made.

Read with `python3 _bmad-output/implementation-artifacts/10-7-signoff/lumstats.py <png>=<label>`.

**Obligations this recipe cannot yet discharge:** AC5's two consecutive exit-0 runs and AC6's
RED (constant left at the boot7 figure, the extended test fails on the approved frame) exist only
after the table lands. The dev pastes both, RED then GREEN.

## Branch and commits

Branch `10-8-lighting-and-atmosphere-re-judged-under-the-sun`, cut from `main` at `3ed269c`; not
stacked, so AC1's range is `3ed269c..HEAD`. Author every commit `Völundr <jeicei75@gmail.com>`,
at least one per completed task. Review-gated: **no push, no PR** until Wolf says so; after any
push, `git ls-remote` confirms it landed (issue #76).

## Change Log

| Date | Change |
|---|---|
| 2026-09-08 | Story created on Wolf's instruction ("lighting and overall atmosphere/style story first"), ahead of 8.3. Seven premises verified against source; the control frame captured twice on `3ed269c` (near-white 2.1686 / 2.2088 %, exit 101) plus two probes — torches off alone reads 1.3342 % and EXITS 0, sun off stays red; the dated finding that the directional's 22,000 lux was set the day after the light stopped reaching any surface. Epic entry corrected at creation: the capture ceilings are re-calibrated on the approved frame, not frozen. Status → ready-for-dev. |

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
