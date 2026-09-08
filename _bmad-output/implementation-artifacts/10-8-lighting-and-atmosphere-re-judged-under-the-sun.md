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

9. **The subdiv default lives in two lines.** `args.subdiv.unwrap_or(1)` (`ingest.rs:458`) and the
   guarded `TerrainSubdivision` insert at `:433-434`; `MAX_SUBDIV = 16` (`project.rs:167`). k=4 was
   RULED 2026-09-01 for dig smoothness (5–13 ms a dig vs 38–78 at k=8; `10-6-signoff/decision.md`),
   the k=16 reopening WITHDRAWN there, and the vehicle read >130 fps at k=4 (`vehicle-fps.md`).
   Every pixel-guard capture passes `--subdiv 1` explicitly (`pixel_guard.rs:380-388, 280, 336,
   496-497`), so the default flip changes no existing test's path — verify with
   `rg -n '"--capture"' crates/gui/tests` before trusting that.
10. **The two flank paths.** k=1 draws the whole cell in its material plus a `snow_cap_mesh` slab
    (`project.rs:335, 1937`); k>1 paints snow on top faces only, pinned by
    `a_capped_cell_paints_snow_on_its_top_faces_and_rock_everywhere_else` (`project.rs:3015`) down
    to "covered terrain keeps its dark flank" (`:3633`). The fine path's detail carving is a
    self-labelled MEASUREMENT STAND-IN (`project.rs:1159`), so part of what the eye reads at k=4 is
    placeholder by its own admission — say so on the card.

## The frame, read at creation

The control frame reads as a **bright, hard-shadowed daylight snowfield under a night sky**: pale
blue ground, long tree shadows, a camp floor blown to flat white. The PRD's frame is *"a dark blue
night world"* whose sky is the illuminant and whose snow *"stays midtone blue-grey; only emissive
light approaches white"*. The tech-art doc still describes the key as a *"green-blue directional
light [that lets] the aurora catch snow and ice"* — a night key, not a sun. Premise 2 says why: the
key's strength was never chosen as light.

## Rulings — 2026-09-08

**RULED at creation (Wolf, 2026-09-08), extending the story:** the snow-and-rock flank rule is IN
(Ruling 2); issue #62 is IN (Ruling 3); sky, aurora, fog, rim and snowfall are OPEN for change on
named defects (Ruling 4); **the shipped terrain default becomes `--subdiv 4` and lands FIRST**;
issues #72 and #77 are fixed here. Day/night and the art-shot post stack are NOT here — they are
Epic 11 (`epics.md` § Epic 11), which runs after this story and before 8.3; the cycle boots at
night, so this story's approved frame is the night Epic 11 builds on.

**Still owed at the opening sitting, BEFORE any table is drafted:**

1. **Is the key light a SUN or NIGHT LIGHT (aurora/moon), and how bright may a night key be?**
   This decides whether the table is re-derived toward the PRD's dark night or toward the daylit
   read the control shows. Also: intensities or one `Exposure` component — pick ONE knob family.
   Do not inherit "sun" from 10.7's title as a decision; 10.7 ruled elevation only. Epic 11.3
   will make this light a sun by day and a moon by night — so what is ruled here is the NIGHT key.
2. **Which winter?** From the k=1 / k=4 same-framing side-by-side (Task 0): snow on vertical faces
   or stone flanks under a snow cap. The losing path is made to match.
3. **Which sky, aurora, fog, rim or snowfall defects, if any, does Wolf name on the k=4 control
   frame?** Each named defect opens that constant, with the frame that shows it.

**Fixed, not owed:** the sun's elevation `+17.66°` and azimuth `40.0398°` stay
(`atmosphere.rs:35,39`; Wolf, 2026-09-03). `the_approved_sun_lights_downward` stays as is.

### RECORDED at the opening sitting — Wolf, 2026-09-08

Handed to Wolf: the k=4 control frame and its range-check line, the k=1/k=4 flank pair, the
figures with their noise floor, Premise 2's dates, and "The frame, read at creation". Answers
given in two messages while the Task 6c mutation run was live, **verbatim**:

> *"while mutation runs answers to ruling: 1) Night and moonlight 2) How snow works really.. in
> dream case there would be a separate layer of snow growing to some extend over time ..if digged
> then ofc under that should be stone.. so maybe yes flank-k4 is closer to target but maybe snow
> layer it bit too thick? 3) cannot see fog, snowfall does not start from the top of the screen
> depending view angle and maybe flakes could be smaller, the rim dissolve?"*

> *"1 intensities 3 actually maybe rim is connected to fog.. visible terrain cut off is too sharp
> so could it fanish to fog so that it looks like world continues and is not just diorama visible
> atm? but visible area of terrain is ok right now until we will get other things in shape at
> least"*

**Ruling 1 — NIGHT KEY, MOONLIGHT. Knob family: INTENSITIES.** The key is not a sun. The table
is re-derived toward the PRD's dark night. The knob is the light table
(`night_lighting()` / `light_properties()`); **no `Exposure` component is added** — that stays
Epic 11.1's mechanism. How bright the night key may be is deliberately NOT ruled in words: Wolf
chooses it from Task 3's candidate frames, which is what AC4 exists for.

**Ruling 2 — THE k=4 FLANK RULE WINS.** Stone flanks under a snow cap. The k=1 slab path is the
loser and is made to match (AC14). Wolf's *"maybe snow layer it bit too thick"* is answered by
mechanism, not by a constant: at k=4 settled snow is **paint on the top faces with no thickness at
all** (`project.rs:940-975`), so the apparent thickness is the detail carving, which
`project.rs:1159` labels a **MEASUREMENT STAND-IN** for 10.4's authored terrain. **Nothing in
10.8 tunes it.** Wolf's dream case — a settled snow LAYER that accretes over time and reveals
stone when dug — is a sim-side feature, recorded in `deferred-work.md`, not built here.

**Ruling 3 — FOUR DEFECTS NAMED**, each opening exactly one thing and nothing else:

| # | Defect, in Wolf's words | Opens |
|---|---|---|
| a | *"cannot see fog"* | `fog_falloff` (`ingest.rs:1412`): 70→210 at the boot framing |
| b | *"snowfall does not start from the top of the screen depending view angle"* | the flake spawn band: height `11.0 + SNOWFLAKE_FALL_SPAN 20.0` over `SNOWFLAKE_DISC_RADIUS 48.0` (`atmosphere.rs:182-196`) |
| c | *"maybe flakes could be smaller"* | `snowflake_scale` (`atmosphere.rs:214`), today `0.3 + 0.18` |
| d | *"visible terrain cut off is too sharp … not just diorama"* | the rim dissolve (`RIM_WIDTH 26`, `RIM_LEVELS 13`, `rim_level` `project.rs:1948`, `rim_dissolved_color` `appearance.rs:266`) **together with** the fog |

**NOT opened by Ruling 3:** how much terrain is visible. Wolf: *"visible area of terrain is ok
right now until we will get other things in shape at least"*. The sky colour, the aurora and the
star shell carry no named defect and do not move. Ruling 4's scope is these four rows.

**ORCHESTRATOR FINDING, derived from source at the sitting — NOT yet measured, Task 6b must test
it against frames before acting on it.** Defects (a) and (d) plausibly share ONE cause, which is
why Wolf's *"maybe rim is connected to fog"* is likely right. Both the fog colour and the rim's
target colour are pinned to `night_lighting().sky` = `srgb_u8(5, 12, 28)`, a very dark navy. But
the sky actually VISIBLE at the horizon is the aurora curtain, `srgb_u8(73, 157, 144)` at up to
`AURORA_PEAK_ALPHA 0.55`. So distant terrain and the world edge both fade toward a colour DARKER
than what is behind them: the dissolve cannot make the edge vanish, it can only turn it into a
dark band against a bright horizon, which is exactly a diorama edge. `docs/tech-art-guidelines.md`
§ Edge treatment currently forbids splitting them — *"The fog colour and the rim's target colour
MUST both be exactly the sky colour. A haze colour only becomes available once the sky itself
carries a vertical gradient; until then these three colours move together or not at all."* The
aurora curtain gives the horizon a gradient in APPEARANCE but not in the sky CONSTANT. If the
frames confirm the mechanism, **the doc rule is what changes**, in the same commit, with the
before/after pair that showed it.

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

### The extensions (ruled 2026-09-08)

13. **`--subdiv 4` is the shipped default and lands first.** A client started with no `--subdiv`
    builds terrain at k=4 (`ingest.rs:433-434, 458`); every figure in this story is taken at that
    default; every test that meant k=1 says `--subdiv 1`; the tech-art doc's "shipped default"
    row and the `deferred-work.md` "no owner" entry are corrected in the same commit. The vehicle
    card re-reads fps and one dig's cost at the default against 10.6's figures.
14. **The flank rule.** After Ruling 2, both meshers paint one pinned snow-capped cell's vertical
    faces the same material; a test compares the two paths' face materials for that cell and is
    shown RED against the unfixed loser; `a_capped_cell_paints_snow_on_its_top_faces_and_rock_
    everywhere_else` is corrected to the ruling, not deleted.
15. **Atmosphere constants change only on Ruling 3's named defects**, each change citing its
    frame; the pins (`the_aurora_curtain_hugs_the_horizon_beyond_the_world`,
    `the_star_shell_fills_the_visible_sky_wedge`,
    `the_rim_dissolve_runs_from_the_untouched_material_to_the_bare_sky`) are corrected, not
    loosened; UX-DR10 holds (night snow midtone, only emissive approaches white).
16. **Issue #72 fixed:** the capture's PNG is on disk before any range-check panic can end the
    process — reproduced RED first with the all-off pixel guard, then green on five consecutive
    full-tier runs.
17. **Issue #77 fixed:** `motion_assertions_apply` asks whether any dwarf is DRAWN in the captured
    slice, not whether the mirror holds one; a below-the-cut capture (`--z` under the dwarves)
    exits without a motion panic — reproduced RED first.

### Sign-off

12. **Closing half (UX-DR22).** Wolf views the built result live on the vehicle against the
    approved frame, states whether *"lighting is still way off"* is still true, and the two
    inherited eye-checks (hover slab on a vertical face near the fire; mark modes apart at a
    glance) are each closed or reopened on the record. If "way off" is still true, that is this
    story's finding and 8.3 does not start.

## Tasks / Subtasks

- [x] **Task 0 — Ship k=4, then re-take the control** (AC: 13; FIRST)
  - [x] `ingest.rs:458` `unwrap_or(1)` → `4`; `:433-434` insert `TerrainSubdivision` unconditionally.
        `rg -n '"--capture"' crates/gui/tests` and make every k=1 caller explicit.
  - [x] Re-capture the control ×2 at the new default (same recipe as Verification); these replace
        the k=1 control as this story's noise floor and as the `creation-control` in AC6. Keep the
        k=1 frames; the AC6 discrimination test may use either as the "above ceiling" frame.
  - [x] Also capture the k=1 / k=4 side-by-side for Ruling 2, same framing, both filed.
  - [x] Doc row + deferred-work entry corrected in the same commit.

- [x] **Task 1 — The rulings sitting** (AC: 2) — Rulings 1–4 recorded above, 2026-09-08
  - [ ] Hand Wolf: the k=4 control frame, its range-check line, Premise 2's dates, "The frame, read
        at creation", the k=1/k=4 pair, and Rulings 1–3 as questions. Record the answers verbatim.
  - [ ] **Stop here until Ruling 1 is recorded.** No lighting constant moves before it.

- [x] **Task 2 — Per-emitter marginals** (AC: 3)
  - [x] Check the stamp first: `target/debug/gui --version` must print the branch HEAD with no
        `-dirty`; `touch crates/gui/build.rs` before the build if it lags.
  - [x] Captures, one daemon (`simd 0`, read the port from its `listening on` line):
        all-on ×2, then `--lights-off` for `sun`, `ambient`, `campfire`, `torches`, `lanterns`,
        `campfire,torches`. Read each with
        `python3 _bmad-output/implementation-artifacts/10-7-signoff/lumstats.py <png>=<label>`.
  - [x] Table: source off · warm-lit · ground median · near-white · blown pool (diagnostic only) ·
        mean · Δmean vs all-on · ×noise. Torches and campfire rows state the marginal with the
        other OFF. Commit as `10-8-signoff/AC3-marginals.md` with the PNGs.

- [~] **Task 3 — Candidates, captured by the client** (AC: 4) — candidates built and filed;
      **AWAITING WOLF'S CHOICE**
  - [x] Each candidate = an edit to `night_lighting()` / `light_properties()`, built, captured at
        the boot framing ×2, **reverted** (tree verified clean between candidates; nothing landed).
        Ruling 1 chose intensities, so no `Exposure` component was added. PNGs named by what
        changed, never by verdict: `candidate-{A-cold-fill-dimmed,B-moon-coloured-key,
        C-emitters-trimmed}-d04e59f-{a,b}.png`.
  - [x] Every PNG filed with its range-check line and lumstats figures, beside the control's two
        runs as the noise floor. Presented side by side in `10-8-signoff/AC4-candidates.md`.
  - [ ] **STOPPED HERE — Wolf has not chosen.** Record the choice with filename and figures.
        Tasks 4, 5, 6, 6b and 8 all wait on it.

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

- [ ] **Task 6b — The flank rule and the atmosphere defects** (AC: 14, 15)
  - [ ] Make the losing path match Ruling 2 (`project.rs:335/1937` slab path or `:3015` top-face
        rule); write the two-path agreement test RED first; correct the existing pin.
  - [ ] For each Ruling 3 defect: change the constant, file the before/after pair with figures,
        correct the pin that names it.

- [~] **Task 6c — The instrument's own defects** (AC: 16, 17) — code complete, AC16's five-run
      evidence OWED (see Completion Notes)
  - [~] #72: the ordering fix landed and is pinned by a deterministic test, but the RACE ITSELF was
        never reproduced and the five green full-tier runs were NOT taken — the dev session was
        cut off by a Codex quota exhaustion mid-run. AC16 is NOT satisfied.
  - [x] #77: reproduced with `--z 5 --capture` (below the dwarves) → motion panic, exit 101; made
        `motion_assertions_apply` read the captured slice; RED then green.
  - [x] Do not close the issues from a commit keyword — no commit on this branch names a closing
        keyword; `git log main..HEAD --grep='Closes #\|Fixes #'` is empty.

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
- **Do not build any Epic 11 mechanism here** — no AO, bloom, exposure component, depth of field,
  volumetric fog, clock or moon. Ruling 1 may pick `Exposure` as the ONE knob; that is the only
  exception, and it is one constant.
- **Do not touch camera or composition**; `fog_falloff` and the rim only on a Ruling 3 defect.
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
| `crates/gui/src/ingest.rs` | UPDATE | `:433-434, 458` subdiv default → 4; `motion_assertions_apply` for #77; one `Exposure` at `:1124` only if Ruling 1 chose it |
| `crates/gui/src/project.rs` | UPDATE | the losing flank path (`:335/1937` or `:3015`), per Ruling 2 |
| `crates/gui/tests/pixel_guard.rs` | UPDATE | #72 reproduction and the fixed write ordering |
| `scripts/bench/valley_bench.py` | UPDATE if a colour changes | lockstep with the client |
| `crates/gui/tests/bench_contract.rs` | UPDATE | anchors move; #62 rows if Ruling 3 IN |
| `docs/tech-art-guidelines.md` | UPDATE | Lights, Value ladder, Sky and lights |
| `_bmad-output/implementation-artifacts/10-8-signoff/` | UPDATE | marginals, candidates, approved frame, vehicle card |
| `_bmad-output/implementation-artifacts/mutations/10-8-lighting-and-atmosphere-re-judged-under-the-sun.sh` | NEW | ≥4 rows |
| `_bmad-output/implementation-artifacts/deferred-work.md` | UPDATE | strike the near-white calibration, the k=4 "no owner" and the snow-flank entries as each lands |

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
| 2026-09-08 | **Task 3 / AC4: three candidate night-key tables built, captured ×2 each, and REVERTED** — nothing landed. A one-variable ladder (cold fill dimmed → moon-coloured key → emitters trimmed) filed with frames and figures in `AC4-candidates.md`. **Dimming the cold fill is the whole picture** (149× noise on frame mean); **recolouring the key moves near-white 0.6× noise, below the floor**, so B's case is an eye case and cannot be made on metrics, which matters because a colour move forces AC8's bench lockstep. Awaiting Wolf's choice; Tasks 4, 5, 6, 6b and 8 wait on it. |
| 2026-09-08 | **Task 2 / AC3 done: per-emitter marginals at the k=4 shipped default.** Eight captures, one daemon, stamp `c7bfb00`. Torches out-weigh the campfire **6.4:1** on warm-lit with the other off. **Two findings:** Premise 8 is FALSE at the shipped default — torches off alone now reads 1.6046 % and stays RED, and only campfire+torches together clears the ceiling; and the **ambient is the dominant illuminant**, costing the frame 404× noise against the directional's 59×, so Ruling 1's re-derivation is mostly an ambient decision. Instrument caveat filed: warm-lit counts red-over-blue, so switching off a COOL source inflates it. |
| 2026-09-08 | **Rulings 1–4 recorded at the opening sitting** (AC2), verbatim: night moonlight key with the INTENSITIES knob, the k=4 flank rule wins, and four named atmosphere defects (fog invisible, snowfall not reaching the top of frame, flakes too large, the terrain cut-off too sharp / diorama). Orchestrator finding filed with them: fog and rim both fade toward the dark sky constant while the visible horizon is the bright aurora, which would explain two of the four defects at once — unmeasured, Task 6b must test it. |
| 2026-09-08 | **Task 6c part done.** Issue #77 fixed: `motion_assertions_apply` now asks whether a dwarf lies within the CAPTURED SLICE, on both arms; RED reproduced first, `--static-world` workaround removed. Issue #72's ordering fix landed and is pinned deterministically, but **AC16 is NOT met** — the race was never reproduced and the five full-tier runs were not taken, the dev session dying on Codex quota exhaustion. Mutation table opened with four rows; one SURVIVED because the test compared against the constant the sabotage moved, fixed by pinning the literal, then 4/4 KILLED. |
| 2026-09-08 | **Task 0 done: `--subdiv 4` is the shipped default.** One `DEFAULT_TERRAIN_SUBDIV` constant feeds both the resource and the perf provenance; every k=1 test caller is now explicit. Control re-taken ×2 at the new default (near-white 2.5000 / 2.4554 %, exit 101, noise floor 0.0446 pp) plus the k=1/k=4 flank pair for Ruling 2. Doc rows superseded and the `deferred-work.md` "no owner" entry closed in the same commit. |
| 2026-09-08 | **Extended on Wolf's rulings at creation:** `--subdiv 4` becomes the shipped default and lands first (Task 0); the snow-flank rule, atmosphere constants (on named defects), issue #62 and the instrument bugs #72/#77 are IN (ACs 13–17). Day/night and the art-shot post stack are split into **Epic 11** (three stories, after this one, before 8.3), booting at night. Tonemapping premise verified: LUTs are on through `3d_api`. |
| 2026-09-08 | Story created on Wolf's instruction ("lighting and overall atmosphere/style story first"), ahead of 8.3. Seven premises verified against source; the control frame captured twice on `3ed269c` (near-white 2.1686 / 2.2088 %, exit 101) plus two probes — torches off alone reads 1.3342 % and EXITS 0, sun off stays red; the dated finding that the directional's 22,000 lux was set the day after the light stopped reaching any surface. Epic entry corrected at creation: the capture ceilings are re-calibrated on the approved frame, not frozen. Status → ready-for-dev. |

## Dev Agent Record

### Agent Model Used

- **Dev (delegated):** `gpt-5.6-terra`, `model_reasoning_effort: high` — read off the run banners,
  not self-reported. Two sessions: `01a07f75-e501-7970-b2a0-7fc145ea0851` (Task 0) and
  `01a07f91-8992-71c2-8181-822c5cf527a1` (Task 6c, mutations).
- **Orchestrator:** `claude-fable-5-1` for the handoffs and Task 0 verification; the harness
  switched the session to `claude-opus-5[1m]` mid-run, which wrote the rulings and this record.

### Debug Log References

**RED — Task 0, the shipped default (AC13).** The resource test before the default existed:

```
test ingest::tests::absent_subdiv_flag_installs_the_shipped_default_four ... FAILED
thread '...' panicked at crates/gui/src/ingest.rs:2419:25
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 144 filtered out
```

**Task 0 captures.** Build stamped `gui build 25f217b`, HEAD `25f217b`, no `-dirty`. Boot framing,
`--frames 160`, no `--z`. Full figures in `10-8-signoff/task-0-control.md`:

```
control-k4-25f217b-a  warm-lit=27338 ground-median=126 near-white=2.5000% blown-pool=1.2994% p99=233.6  exit 101
control-k4-25f217b-b  warm-lit=26944 ground-median=126 near-white=2.4554% blown-pool=1.2551% p99=233.4  exit 101
flank-k1-25f217b      warm-lit=21690 ground-median=135 near-white=2.1978% blown-pool=1.1287% p99=229.9  exit 101
flank-k4-25f217b      warm-lit=27301 ground-median=125 near-white=2.4263% blown-pool=1.2428% p99=231.4  exit 101
```

Noise floor, worst of the k=4 control pair: mean `0.074` · near-white `0.0446 pp` · warm-lit
`394 px`. Exit 101 throughout is the shipped near-white ceiling, still red as Premise 1 recorded;
the PNG is written first, so no evidence is lost. **k=4 is darker whole-frame (mean −6.7, ~90×
the floor) yet hotter at the camp (warm-lit +5,600 px, near-white +0.23 pp).**

**RED — issue #77 (AC17).** `gui <port> --headless --capture <png> --subdiv 1 --z 5 --frames 700`:

```
thread 'Compute Task Pool' panicked at crates/gui/src/capture.rs:445:9:
capture rendered no mid-blend entities
a capture below every dwarf must exit cleanly; it exited Some(101)
test a_capture_below_the_dwarves_skips_motion_but_still_writes_a_png ... FAILED
```

Then green after narrowing `motion_assertions_apply` to dwarves at or below the cut, and the
`--static-world` workaround was removed from `the_dwarf_startup_line_reports_what_was_actually_drawn`.

**Mutation round (AC11), `scripts/mutate.sh` run alone.** First pass:

```
restore the shipped terrain subdivision default to one       SURVIVED
make motion assertions ignore the captured slice again       KILLED
validate a captured frame before its PNG is written          KILLED
demand motion below a dwarf-free captured slice again        KILLED
1 mutation(s) did not KILL.
```

**The SURVIVED row is the finding, and it is [[sabotage-blind-when-fixture-matches-constant]]
again:** the test asserted `resource == DEFAULT_TERRAIN_SUBDIV`, so a sabotage that moves the
constant moves the expectation with it and pins nothing. Fixed in `ab33fa9` by asserting the
literal `4`. Re-run, all four KILLED:

```
restore the shipped terrain subdivision default to one       KILLED
make motion assertions ignore the captured slice again       KILLED
validate a captured frame before its PNG is written          KILLED
demand motion below a dwarf-free captured slice again        KILLED
```

Binary rebuilt to the clean `31f1417` stamp afterwards — a mutant build outlives the source
restore.

**Task 2 / AC3 — per-emitter marginals, at the shipped k=4 default.** Build stamped
`gui build c7bfb00` = HEAD, no `-dirty`; one daemon, port 43593; eight captures at the boot
framing with no `--subdiv` flag. Full table, marginals and frames in `10-8-signoff/AC3-marginals.md`.
Noise floor, worst of the all-on pair: warm-lit `164 px` · near-white `0.0474 pp` · mean `0.116`.

```
all-on a           warm-lit=27664 ground-median=126 near-white=2.4398% blown-pool=1.2082% p99=232.4  mean 94.472  exit 101
all-on b           warm-lit=27500 ground-median=126 near-white=2.4872% blown-pool=1.2735% p99=233.7  mean 94.588  exit 101
sun off            warm-lit=28370 ground-median=117 near-white=2.1296% blown-pool=1.2203% p99=233.6  mean 87.689  exit 101
ambient off        warm-lit=71855 ground-median= 69 near-white=1.7812% blown-pool=1.1502% p99=229.7  mean 47.616  exit 101
campfire off       warm-lit=26643 ground-median=123 near-white=2.2784% blown-pool=1.2615% p99=229.9  mean 94.142  exit 101
torches off        warm-lit= 9449 ground-median=117 near-white=1.6046% blown-pool=0.7369% p99=211.8  mean 92.584  exit 101
lanterns off       warm-lit=30197 ground-median=125 near-white=2.1650% blown-pool=1.0310% p99=223.1  mean 94.187  exit 101
campfire+torches   warm-lit= 6257 ground-median=115 near-white=1.3102% blown-pool=0.5679% p99=207.8  mean 91.904  exit 0
```

**TWO FINDINGS THE REST OF THE STORY MUST BE READ AGAINST.**

1. **PREMISE 8 IS FALSE AT THE SHIPPED DEFAULT.** The story records that torches off ALONE takes
   the guard to exit 0 (near-white 1.3342 % vs the 1.5630 % ceiling). That was `--subdiv 1`. At
   k=4 torches off alone reads **1.6046 %, still above the ceiling, exit 101**; the smallest
   switch-off that clears it is now **campfire AND torches together** (1.3102 %). k=4 lifts
   near-white by ~0.28 pp across the board, about 6× the floor. Premise 8's text is left as
   written because a premise records what was true when measured — `AC3-marginals.md` is the
   correction, and no lighting decision may cite Premise 8's exit-0 claim.
2. **THE AMBIENT, NOT THE DIRECTIONAL, LIGHTS THIS VALLEY.** Ambient off moves the frame mean
   −46.914 (**404× noise**, dark pixels 17.5 % → 54.95 %); the directional off moves it −6.841
   (59×). Premise 2 says the directional's 22,000 lux was never judged as light; this adds that it
   is **not the dominant term either**. Under Ruling 1 the night-key re-derivation is mostly an
   AMBIENT decision, and a candidate that moves only `directional_illuminance` will move the
   picture far less than its number suggests.

**INSTRUMENT CAVEAT recorded with them:** `warm_lit_pixels` (`capture.rs:548`) counts
`red − blue > 30`, a purely relative test, so switching off a COOL source INFLATES it — ambient
off reads 71,855 warm-lit (2.6× all-on) and lanterns off reads above all-on. Warm-lit is only
meaningful for warm emitters with the cool fill held constant; judge a cool source on ground
median and frame mean.

**Task 3 / AC4 — three candidate tables, built, captured ×2 each, and REVERTED.** Ruling 1's
knob only: ambient and directional strength, plus emitter intensities in C. Sky, aurora and star
shell untouched. Full table, per-knob decomposition and the eye reading in
`10-8-signoff/AC4-candidates.md`. A one-variable-at-a-time ladder — A dims the cold fill, B adds a
moon-coloured key, C adds trimmed emitters:

```
control   near-white 2.4635 %   mean 94.530   ground median 126
A         near-white 1.9529 %   mean 77.183   ground median 100.5
B         near-white 1.9240 %   mean 74.900   ground median  97
C         near-white 1.5766 %   mean 74.148   ground median  94
noise floor from the control pair: near-white 0.0474 pp, mean 0.116
```

**What each knob did, and one of the three answers is a surprise:**

```
control -> A  cold fill dimmed     near-white -0.5105 pp (10.8x)   mean -17.347 (149.5x)
A -> B        key recoloured       near-white -0.0289 pp ( 0.6x)   mean  -2.282 ( 19.7x)
B -> C        emitters trimmed     near-white -0.3475 pp ( 7.3x)   mean  -0.753 (  6.5x)
```

**RECOLOURING THE KEY IS BELOW THE NOISE FLOOR on near-white (0.6×)** — the instrument cannot see
it. It is a value change (mean, ground median), not a highlight change. Since a colour move forces
lockstep edits to `valley_bench.py` and both `bench_contract.rs` anchors under AC8, **B's case has
to be made by eye or not at all**; no capture metric here supports it.

**The exit column is informational, not a verdict.** Candidates were judged against the SHIPPED
`NEAR_WHITE_AREA_CEILING = 1.5630 %`, which Premise 3 records as calibrated on a sun-under-the-map
frame. AC6 re-derives it from the APPROVED pair. What AC6 requires is that the control still sits
ABOVE the re-derived ceiling, and all three satisfy that (A 2.0479 %, B 1.9656 %, C 1.6775 %,
control 2.4635 %). C is the only one that would also clear the OLD ceiling, and it STRADDLES it —
1.5429 % then 1.6102 % across a 0.0673 pp swing — which is exactly the run-to-run behaviour AC6
exists to absorb.

**Open for Wolf's eye, and no metric here covers it:** at 7,000 lux the moon casts almost no
modelling on the snow. A key that reads as a key wants a fourth candidate between 7,000 and 22,000.

### Completion Notes List

**DONE**

- **Task 0 / AC13.** `DEFAULT_TERRAIN_SUBDIV = 4` in `ingest.rs`; `TerrainSubdivision` is now
  inserted unconditionally and the perf-provenance line reads the same constant, so the two sites
  cannot disagree. Every test caller that meant k=1 says `--subdiv 1`. Control re-taken ×2 at the
  new default, flank pair filed, `docs/tech-art-guidelines.md` rows superseded in the doc's own
  style and the `deferred-work.md` "no constant and no owner" entry closed, all in the same commit.
- **Task 1 / AC2.** Rulings 1–4 recorded above with Wolf's words verbatim and the date. **No
  lighting constant was changed before it** — the only code commits before the sitting were the
  subdiv default and the two instrument fixes, none of which touch `appearance.rs`.
- **Issue #77 / AC17.** RED reproduced, predicate narrowed to the captured slice on BOTH arms,
  unit test covers dwarf-at-cut / dwarf-above-cut / empty-mirror, integration test drives the real
  binary below the cut. The `--static-world` workaround is gone.

**NOT DONE, and why**

- **AC16 is NOT satisfied.** The #72 ordering fix landed (`f25f337`: the PNG is encoded and written
  synchronously inside `save_then_validate` instead of relying on Bevy's async `save_to_disk`
  observer) and is pinned by a deterministic test that panics the validator and then asserts a
  decodable PNG on disk. But **the race itself was never reproduced**, and **the five consecutive
  green full-tier runs were not taken** — the Codex session was cut off mid-run by quota
  exhaustion (`You've hit your usage limit … try again at 10:20 AM`). AC16 asks for both. Owed.
  Note `crates/gui/Cargo.toml` moves `image` from a dev-dependency to a dependency for this,
  same version and features, no lockfile change.
- **Task 2 / AC3 is DONE** and **Task 3 / AC4's candidates are BUILT AND FILED** (see the Debug
  Log). **Task 3 is STOPPED at its own gate: Wolf has not chosen a table.** Tasks 4, 5, 6, 6b and
  8 all depend on that choice and are not started. They were blocked on Ruling 1 at handoff time
  and are now unblocked: the key is a night moonlight and the knob is the light table. Task 6b
  additionally carries Ruling 3's four named defects and the orchestrator's fog/rim colour finding
  above, which must be tested against frames before the doc rule is touched.
- **Status stays `in-progress`.** This is a partial story.

### File List

| File | Change |
|---|---|
| `crates/gui/src/ingest.rs` | `DEFAULT_TERRAIN_SUBDIV = 4`, unconditional `TerrainSubdivision`, provenance reads the constant, new + corrected tests |
| `crates/gui/src/capture.rs` | #72 synchronous PNG write before validation; #77 slice-aware `motion_assertions_apply` on both arms; two new unit tests |
| `crates/gui/tests/pixel_guard.rs` | explicit `--subdiv 1` callers, corrected `capture()` comment, `--static-world` workaround removed, below-the-cut integration test |
| `crates/gui/Cargo.toml` | `image` moved dev-dependency → dependency (PNG encode now ships) |
| `docs/tech-art-guidelines.md` | "Terrain, shipped default" row and "Adopted is not shipped" bullet superseded to k=4 |
| `_bmad-output/implementation-artifacts/deferred-work.md` | the k=4 "no constant and no owner" entry closed |
| `_bmad-output/implementation-artifacts/mutations/10-8-…-under-the-sun.sh` | NEW, four rows, all KILLED |
| `_bmad-output/implementation-artifacts/mutations/10-6-how-fine-can-we-go.sh` | renamed test reference |
| `_bmad-output/implementation-artifacts/10-8-signoff/control-k4-25f217b-{a,b}.png` | NEW, the k=4 control pair |
| `_bmad-output/implementation-artifacts/10-8-signoff/flank-k{1,4}-25f217b.png` | NEW, the Ruling 2 pair |
| `_bmad-output/implementation-artifacts/10-8-signoff/task-0-control.md` | NEW, figures and noise floor |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | 10.8 → in-progress |
| `_bmad-output/implementation-artifacts/10-8-…-under-the-sun.md` | this file |
