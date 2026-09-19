---
baseline_commit: d3ecdff2df8ae1d1284de21c4223ec3e7f0d8587
---

# Story 11.1b: The Air Has Depth

Status: in-progress

## Story

As the boss,
I want creases and contacts to darken and emitters to glow,
so that the diorama reads as lit objects in a space, not as coloured cubes on a plane.

## Stacking — read this first

**This story is STACKED on `story-11-1a-exposure-and-clean-edge`, which is PUSHED at `d3ecdff` but
NOT MERGED and NOT REVIEWED.** Branch off that branch, not `main`.

Consequence, and it is a standing defect class here: **any AC of the form "X did not change" must be
proved with THIS story's own commit range, never `git diff main...HEAD`** — a range against `main`
silently attributes all of 11.1a's changes to this story. Use `git diff d3ecdff..HEAD`.

If 11.1a is merged or amended while this story is in flight, re-check the branch point before every
commit and push, and retarget the PR. That has caught this project out seven times.

## What 11.1a already built for you

11.1a exists because **SSAO is a silent no-op while MSAA is on** — `extract_ssao_settings`
(`bevy_pbr-0.19.0/src/ssao/mod.rs:479-494`) logs an `error!` and then **`return`s out of the whole
loop**, skipping every camera, while the frame still renders and the process exits 0. So:

- `Msaa::Off`, `Exposure { ev100: 9.7 }` and `Fxaa::default()` are on the camera tuple
  (`crates/gui/src/ingest.rs:1308-1335`). **The prerequisite is already satisfied — do not re-add it.**
- `--fx-off <list>` + `F10` + a readout line exist, with the toggle implemented as component
  **insert/remove** (`fxaa_controls`, `ingest.rs:1440`), not an `enabled` flag. Follow that pattern.
- **`creases.py`** (`11-1-signoff/creases.py`) measures Rec.601 `p10`/`median`/`p90`/`mean` over
  pinned windows and is **proved both ways** — see `creases-proof.md`.

## The instrument you inherit, and the one place it does not reach

`creases.py`'s three pinned windows have a **same-build noise floor of ZERO on `p10` and `median`**
(measured across two builds now):

| window | rect | p10 | median |
| --- | --- | --- | --- |
| `terrace-creases` | 860,190 → 1060,290 | 31 | 65 |
| `open-snow-LL` | 180,620 → 380,700 | 68 | 117 |
| `open-snow-LR` | 950,590 → 1150,670 | 69 | 117 |

**That is an excellent instrument for AO and a useless one for bloom**, because it deliberately
excludes the camp — and the camp is where every emitter is.

**Measured at creation, four same-build controls, camp window `(500,400)..(760,620)`:**

| statistic | across ctl-a..d | spread |
| --- | --- | ---: |
| median | 82, 82, 83, 84 | **2** |
| p90 | 194, 203, 204, 212 | **18** |
| p99 | 248, 250, 250, 251 | **3** |
| mean | 110.421 … 114.959 | **4.538** |
| near-white (≥230) area | 4.2308 … 5.6731 % | **1.4423 pp** |

**Bloom acts on exactly the bright tail that moves.** The cause is not the sim: `flicker_lights`
is driven by `time.elapsed_secs()` (`ingest.rs:1890`), Bevy's wall clock, so `--static-world`
pauses the world and **not** the flicker. A bloom figure taken at the camp without fixing this has
a 1.4423 pp floor — for scale, the whole-frame near-white ceiling's entire headroom is 0.13 pp.
Task 1 fixes the instrument before Task 3 leans on it.

## Acceptance Criteria

1. `--lights-steady` pins the flicker term to a fixed, deterministic phase, and the camp window's
   same-build spread collapses: across at least four captures with the flag, `median`, `p90` and
   `near-white area` each move by less than a tenth of the un-pinned spreads recorded above. The
   before/after floors are both recorded.
2. Ambient occlusion is on the camera, and **its output is consumed, not merely produced**: with AO
   on, `terrace-creases` `mean` drops by more than the window's same-build floor, while
   `open-snow-LL` and `open-snow-LR` `median` stay at their post-stack control value of 116
   exactly. Figures and floor recorded.

   **Amended 2026-09-19 (Wolf).** Two changes, both forced by measurement and neither a loosening.
   `p10` became `mean` because `Hdr` erased p10's signal entirely -- it reads 38 with AO on and 38
   with AO off, so the original statistic can no longer answer the question this AC asks. And the
   open-snow control moved 117 -> 116. That shift was first recorded as **`Hdr`'s** doing; it is
   **bloom's**. `Hdr` and `Bloom` arrived in one commit (`Bloom` `#[require]`s `Hdr`) and were never
   separated, so the attribution was never tested. `--fx-off bloom` leaves `Hdr` in place -- removing
   a component does not remove what required it -- and reads 117/117, while all-effects-on reads
   116/116. The rendered guard runs all effects on, so 116 is its control.
3. **A guard fails if MSAA is ever re-enabled while AO is on.** It must assert the rendered
   consequence, not the component: a test that only checks `Msaa::Off` is present does not satisfy
   this, because the defect it guards is silent at every level above the pixels.
4. Bloom is on the camera, and only emitters and their immediate halo brighten: with
   `--lights-steady` and `--static-world`, the camp window's **halo statistics** (`median` and
   `mean`) rise by more than their pinned floor, while `open-snow-LL` and `open-snow-LR` `median`
   do not BRIGHTEN. Figures and floor recorded.

   **Amended 2026-09-19 (Wolf).** The original clause asked the camp's **bright tail** (`p90`) to
   rise. `Bloom::default()` is `NATURAL`/`EnergyConserving`, which REDISTRIBUTES energy out of bright
   cores into the surround rather than adding any, so it cannot raise the bright tail -- measurement
   confirms it lowers it slightly. The AC was asking for a signature the chosen composite mode does
   not possess. The **halo** this AC also names IS that signature, it is large, and it is what the AC
   now measures. The preset is unchanged: the story still tunes no look.
5. **Headless area figures are compared only headless-to-headless.** `NEAR_WHITE_AREA_CEILING` and
   `BLOWN_POOL_FRACTION_CEILING` are **not** raised and **not judged against** a headless bloom
   frame; the ceiling clause is judged on the vehicle at the sitting (Wolf's ruling, 2026-09-18).

   **Amended 2026-09-19 (Wolf, at the review).** "not **asserted** against" -> "not **judged**
   against". The original wording was factually contradicted by shipped code: `capture.rs:1479`
   asserts `near_white <= NEAR_WHITE_AREA_CEILING` inside `range_check`, which runs on EVERY
   headless capture, so both ceilings were asserted against every bloom frame this story took.
   Nothing about the intent changes -- neither ceiling is raised, both constants and their pins are
   intact, and the VERDICT still belongs to the vehicle. The amendment makes the AC say what the
   2026-09-18 ruling meant and what the story actually did. Silencing the range check instead would
   mean editing `capture.rs`'s assertion for the convenience of an AC, which is the wrong direction.
6. `--fx-off` accepts `ao` and `bloom` beside `fxaa` as a **set**, each reaching the spawned camera
   rather than only `Args`: naming an effect removes its component, omitting it leaves it present.
   An unknown name errors naming all accepted names.
7. Each new effect has a seat toggle beside `F10`, and the readout names each one's state. A test
   presses the real keys and asserts the recorded readout changes.
8. On the vehicle, `--perf-log` at boot framing is read with all effects on and with each off, and
   the p50 frame times recorded. NFR6's 60 fps bar is re-read; if an effect costs it, the figures
   are recorded and Wolf rules rather than the story tuning anything.
9. Wolf signs off both halves at the sitting against the reference art, with the frame pair filed.
10. `crates/gui/tests/capture.rs` stays unchanged and green, and `git diff d3ecdff..HEAD --stat` on
    `crates/protocol`, `crates/sim-core` and `crates/client-core` is empty.
11. Every mutation row in `mutations/11-1b-the-air-has-depth.sh` is shown to KILL, and the table's
    output is pasted into the Dev Agent Record.

## Tasks / Subtasks

- [x] **Task 0 — control, on a clean tree.** (AC: 1, 2, 4)
  - [x] `./target/debug/gui --version` named the current HEAD's tree before capture.
  - [x] Took four same-build captures and recomputed BOTH floors on this build — the crease
        windows and the camp window. **Floors are build-specific**; they moved 3.3x across one story
        and 20x across another. The creation figures above are a control to compare against, never a
        threshold to reuse.
- [x] **Task 1 — make the emitter window measurable.** (AC: 1)
  - [x] Add `--lights-steady`, pinning the `seconds` passed to `flicker_lights` (`ingest.rs:1890`)
        to a constant so every capture sees the same flicker phase. One branch; do not rewrite
        `flicker_scale`, whose determinism is already pinned by
        `flicker_is_bounded_distinct_and_deterministic` (`appearance.rs:175`).
  - [x] **Instrument test:** the flag reaches the live system, not merely parse — copied
        `fx_off_reaches_the_live_camera_and_rejects_unknown_effects` (`ingest.rs:2171`).
  - [x] Re-measured the camp window with the flag; its spread did not collapse. **Stopped as required:
        stop and say so** — every bloom figure in this story depends on it.
  - [x] **Resolved 2026-09-19.** The residual was never flicker. `--static-world` is documented as
        "freeze the sim" (`README.md:201`) and froze nothing: it set a bool that silenced the
        capture's motion assertions while the daemon kept ticking, so the dwarves kept walking and
        carrying their lanterns through the camp window. Wired it to the `SetSpeed { Paused }` that
        `command.rs` has sent since 10.5 — `crates/gui` only, no wire or sim change. The camp spread
        then collapsed to 0/0/0.0017 pp and **AC1 is met as written**.
- [x] **Task 2 — ambient occlusion.** (AC: 2, 3)
  - [x] Add `ScreenSpaceAmbientOcclusion` to the camera tuple. `DepthPrepass` and `NormalPrepass`
        arrive automatically via `#[require(...)]` (`ssao/mod.rs:113`) and `PbrPlugin` already
        registers the plugin (`bevy_pbr-0.19.0/src/lib.rs:227`, in `DefaultPlugins` at
        `bevy_internal-0.19.0/src/default_plugins.rs:79`). **No `add_plugins`, no feature change.**
  - [x] Measure with `creases.py` against Task 0's floor. Record the figures.
  - [x] **The MSAA guard (AC3).** Assert the rendered consequence. The deliberate RED is in
        Verification below and is the whole reason this story was split out — run it.
- [x] **Task 3 — bloom.** (AC: 4, 5)
  - [x] Add `Bloom` to the camera tuple. `Hdr` arrives via `#[require(Hdr)]`
        (`bloom/settings.rs:32`) and `PostProcessPlugin` is in `DefaultPlugins`
        (`default_plugins.rs:61`). `Bloom::default()` is `NATURAL` — `intensity: 0.15`,
        `low_frequency_boost: 0.7`, `composite_mode: EnergyConserving` (`settings.rs:132-141`);
        `OLD_SCHOOL` (0.05, Additive) and `SCREEN_BLUR` (1.0) are the other presets. Record which
        you chose and why in one line.
  - [x] **`Hdr` changes the whole frame's pipeline, not just the emitters.** Expect every figure to
        move, including the open-snow windows. If they move, AC4's "does not move" clause is about
        bloom's *marginal* contribution — measure bloom on/off with `Hdr` present in both, not
        against a pre-`Hdr` control, or you will attribute the pipeline change to bloom.
  - [x] Record the headless area figures as a DELTA only (AC5). Do not assert a ceiling on them.
- [x] **Task 4 — the switches.** (AC: 6, 7)
  - [x] Generalise `FxaaOff(bool)` (`ingest.rs:152`) into a set, mirroring `LightSource`/
        `LightingToggles` (`ingest.rs:91-196`) — this is the third concrete effect, so a small enum
        earns its place now and not before. Keep insert/remove, not an `enabled` field.
  - [x] Pick the two keys and extend the readout (`lighting_readout`, `ingest.rs:1371`). `F5`–`F10`
        are taken; note the existing order is not alphabetical (F7 lanterns, F8 ambient, F9 torches).
  - [x] Update the three tests pinning the full readout string (`ingest.rs:2489`, `:2508`, `:2594`).
        **Do not weaken them to substring checks** — they pin the whole line on purpose.
- [ ] **Task 5 — the vehicle read.** (AC: 8, 9) — **cannot be done on a devpod.** No devpod can open
      a window. Write `11-1-signoff/task-5b-vehicle-card.md` naming the exact `--perf-log` runs and
      the frame pair Wolf judges, leave this task UNCHECKED, and hand it to the gingerspice sitting.
      **Invent no fps figure.**
- [x] **Task 6 — sabotage.** (AC: 11)
  - [x] Scope-only rows: AO omitted; AO present with MSAA re-enabled; strengthened
        `--lights-steady` consequence. The bloom, effect-switch, key, and readout rows remain
        deferred with Tasks 3–4.
  - [x] Rows for: AO omitted; AO present but MSAA re-enabled (AC3's guard — this row must KILL, and
        it is the most important row in the table); bloom omitted; each `--fx-off` name discarded;
        each new key moved; `--lights-steady` value discarded; the readout not recording.
  - [x] `rg` all of `mutations/` for rows quoting `ingest.rs` camera-tuple or readout literals and
        re-point any this story breaks. **11.1a broke a 10.7 row exactly this way and it failed the
        gate** — budget for it. APPLY-FAILED is not noise.
  - [x] **Never `exec` a mutation payload.** And per **issue #104**, `mutate.sh` does NOT restore
        tracked non-Rust targets: after any run touching `creases.py` or a doc, check
        `git status --porcelain` and restore. It reports KILLED while leaving the file sabotaged.

### Review Findings

Code review 2026-09-19, fresh context, four layers (Blind Hunter + Edge Case Hunter on Sonnet,
Acceptance Auditor + Feature Auditor on Opus), all four ran the binaries, **no coverage holes and
no layer timed out**. This review carried the adversarial load alone: no self-gate ran on this
story (four codex handoffs died on the read-only `/tmp` mount-registry lock, three-pass cap spent,
issue #49). Severities below are the orchestrator's, re-rated from source; layer severities do not
carry. Layer attribution is recorded on each item.

**Settled and NOT reopened:** both 2026-09-19 amendments are measurement-forced corrections, not
lowered bars — the Acceptance Auditor reproduced both independently (terrace `p10` reads 38 with AO
on and 38 with AO off, so the retired statistic is genuinely dead; `Bloom::default() == NATURAL ==
EnergyConserving` at `bloom/settings.rs:139,185-189`, so the original bright-tail clause was
unsatisfiable by construction). Wolf's #106 ruling — AO accepted as wired, its look deferred to
11.3 — also stands and is not relitigated here. All nine Dev Notes scope guardrails verified CLEAN,
including commit hygiene: 31/31 commits authored `Völundr`, zero Claude trailers.

#### Wolf's rulings on the decision items (2026-09-19)

All six resolved at the review sitting. Each became a PATCH; none was deferred or dismissed.

1. **Space under `--static-world`: REFUSE IT.** `toggle_pause` consults `StaticWorld` and ignores
   the press, with a readout line saying why. Same fix shape as the permanent-pause HIGH, so the
   two close together: `--static-world` becomes a sustained invariant instead of a startup event.
2. **The pause race: FIX IT, then re-measure.** Wait for the daemon's pause to be acknowledged
   before the capture countdown starts, so the frozen tick is deterministic. AC1's and AC4's camp
   figures are then re-measured on the stable world.

   **CORRECTION, made during the patch pass.** This item was first recorded as "the fix lives in
   `crates/gui`'s startup path — AC10 forbids touching `capture.rs`". That was WRONG and it
   narrowed the fix for no reason: AC10 pins `crates/gui/**tests**/capture.rs`, the integration
   test. `crates/gui/**src**/capture.rs` is a different file, is not named by any AC, and is where
   the frame countdown lives. The fix may live there.

   **AND THE HANDSHAKE IS AVAILABLE, contrary to a comment that says it is not.** `SimPaused`'s doc
   comment (`command.rs:47-50`) states "the wire carries no speed in the snapshot; this is
   presentation state that mirrors what we last ASKED for, not what the daemon reports." That is
   FALSE: `protocol::Snapshot` carries `pub speed: Speed` (`protocol/src/lib.rs:152`) and so does
   `protocol::Delta` (`:168`). The daemon has been reporting its speed on every message all along.
   That false comment is the stated justification for the client mirroring its own request instead
   of observing the daemon — which is exactly the design that produced #105 and then this race. It
   was found in the patch pass, by no review layer, and it is the same latent-doc-lie class as the
   `README.md:201` item below.
3. **AC5: AMEND THE WORDING** from "not asserted against" to "not judged against", which is what the
   2026-09-18 ruling meant and what the story actually did. No code change; `capture.rs:1479` keeps
   asserting the ceiling on every headless capture, and AC10 forbids changing that here.
4. **The open-snow control: ADD TO #106, do not touch the ACs.** #106 is already open for exactly
   this ("AC2 as written cannot distinguish 'consumed' from 'visible'"). Record that the same defect
   applies to AC4's "does not brighten" clause. No AC is rewritten mid-story.
5. **AC4's bloom guard: WRITE IT NOW.** The rendered bloom/halo guard the Project Structure table
   promised, using `--fx-off bloom` as the control, built against the floor that ruling 2 produces.
6. **Prepasses: REMOVE WITH REQUIRES.** `--fx-off ao` and F11 use `remove_with_requires` so "ao off"
   is genuinely off and AC8's vehicle cost read is honest. This CHANGES what `--fx-off ao` renders,
   so the AO-off control and the delta-based AC3 guard must be re-baselined after it.

**ORDERING IS LOAD-BEARING** and is not the order above: fix the race (2) FIRST, because it
stabilises the frozen tick every camp figure depends on; then re-measure AC1 and AC4; then write the
bloom guard (5) against that floor; then land `remove_with_requires` (6) LAST and re-baseline the
AO-off reading and the AC3 guard once. Taking (6) earlier baselines the guard twice.

- [x] [Review][Decision] **Should Space be allowed to defeat `--static-world`?** — `toggle_pause`
      (`command.rs:99-117`) runs every frame in `PostUpdate`, never consults `Res<StaticWorld>`, and
      unconditionally queues `SetSpeed{Normal}` on any Space press. Proved by the Edge Case Hunter
      with a standalone `MinimalPlugins` app against the crate's public API (startup `paused=true`
      → after Space `paused=false`). Interactive seat only; `apply_scripted_input` never presses
      Space, so no headless path is affected. Options: refuse Space under the flag; allow it but
      print a readout line saying the flag's guarantee just broke; or leave it. MED. [edge]
- [x] [Review][Decision] **The pause is RACED, so AC1's floor is not reproducible and the frozen
      world state is non-deterministic.** — `pause_static_world` runs in `Startup` and the command
      drains in the first frame's `PostUpdate`, so the daemon keeps ticking until the pause lands:
      every `--static-world` run prints `ticks observed=3 dwarf position changes=8 ... moved=true`.
      The frozen state is whichever tick won the race, which is scheduling- and load-dependent.
      Measured camp near-white spread of the SAME statistic, same build `46f47c7`, same flags:
      record 0.0017 pp (n=4) · Feature Auditor 0.0385 pp (n=3) · Acceptance Auditor **0.5210 pp**
      (n=5, four tight at 0.028 pp and one outlier departing 0.493 pp and dropping p90 a level).
      AC1's bar is <0.17658 pp, so the reproduction **fails it by 2.95x** while the recorded run
      clears it by 100x. On a quiet box the pause lands on 3 ticks every time, which is why four
      consecutive dev captures looked like a zero floor; the layers were run against a box building
      four Bevy trees at once, which is when the outlier appeared. This bears on whether AC1 is MET
      and on every camp figure in the story, AC4's halo included. Wolf's call: re-measure AC1 under
      load, wait for a daemon pause acknowledgement before the capture countdown, or accept the
      figures as quiet-box measurements and say so. Note AC10 forbids touching `capture.rs`, so the
      `SKIPPED` silencer whose false confidence caused #105 necessarily stays. HIGH.
      [acceptance + feature]
- [x] [Review][Decision] **AC5's "not asserted against a headless bloom frame" is false against
      shipped code.** — `capture.rs:1479` asserts `near_white <= NEAR_WHITE_AREA_CEILING` inside
      `range_check`, which runs on EVERY headless capture including every bloom frame this story
      took. Verified in source by the orchestrator. AC5's important half is MET — neither ceiling
      was raised, both constants and their pins are intact. But the "not asserted" half cannot be
      satisfied without changing `capture.rs`, which AC10 forbids. Compounding it: measured headless
      headroom is 0.196 pp near-white / 0.033 pp blown-pool, against a recorded delivery-GPU penalty
      of **0.4-0.6 pp worse near-white** — i.e. larger than the entire headroom, so the ceiling is
      likely to trip at the sitting. Options: amend AC5's wording to "not judged against", exempt
      bloom frames in a later story, or carry it to the sitting as a known risk. MED.
      [acceptance + feature]
- [x] [Review][Decision] **AC2's and AC4's open-snow "control" is near-inert — a median on an
      integer plateau.** — Three layers measured the same thing independently. AO darkens open snow
      by MORE than it darkens the creases the guard treats as signal: terrace -0.599 vs LL -0.687,
      LR -0.627 (Acceptance Auditor); Feature Auditor read -0.628 vs -0.669/-0.622. Bloom changed
      **99.66% of open-snow pixels**, mean luma -0.667, 2.92% of them by >9 levels (Blind Hunter,
      `delta.py`) — and open-snow-LL's MEAN **brightened by 0.686** while its median fell, so AC4's
      "only emitters and their immediate halo brighten" is unsupported on the sensitive statistic
      and survives only because the clause names the quantisation-blind one. Both clauses would pass
      a flat ~0.6-luma global darkening and a flat frame dim. This is exactly the AC-bar half #106
      was deliberately left open for, now shown to apply to AC4 as well as AC2. Recommend adding it
      to **#106** rather than opening a new issue. MED. [feature + acceptance + blind]
- [x] [Review][Decision] **AC4's promised bloom pixel guard was never written.** — Project Structure
      (`:258`) says `pixel_guard.rs` UPDATE for "AC2's crease pair, AC3's MSAA guard, **AC4's bloom
      pair**". The diff adds exactly one test; `rg` finds no rendered bloom/halo/camp guard anywhere
      in `crates/gui/tests/`. Bloom's only regression net is a component-presence assertion, so a
      preset change or a silent post-process skip would be caught by no pixel. Writing one is real
      work with a real obstacle — the camp is the flicker-noisy window this story had to build
      `--lights-steady` to measure at all, and the race above still moves it. Wolf's call: write it
      now, or file it as its own story beside #108's oracle rebuild. MED. [acceptance + feature]
- [x] [Review][Decision] **`--fx-off ao` and F11 leave `DepthPrepass` + `NormalPrepass` running.** —
      Bevy's `remove::<T>()` does not remove `#[require]`d components (hence the separate
      `remove_with_requires`), so "ao off" still pays two full-scene GPU passes for the rest of the
      session while the readout says off. There is a genuine tension, which is why this is a
      decision and not a patch: leaving them makes AC2's -0.628 delta purely SSAO's shading, which
      is GOOD for the guard; removing them makes AC8's vehicle cost read honest. Today AC8 will
      under-report AO's true cost and Wolf could rule AO cheap on a delta that excludes its
      expensive half. Minimum action either way: the vehicle card must say which cost its column
      measures. MED. [feature + blind]

- [ ] [Review][Patch] `--static-world` leaves the daemon paused permanently and daemon-wide; nothing
      ever sends `Normal` [crates/gui/src/command.rs:74-92] — HIGH. [feature]
- [ ] [Review][Patch] AC3's guard passes with SSAO absent once bloom is off; assert the AO-on/AO-off
      delta, not an absolute level [crates/gui/tests/pixel_guard.rs:165-196] — HIGH. [feature]
- [ ] [Review][Patch] AC11 is NOT MET: the table holds 12 rows and 10 are pasted; the two
      `--static-world` pause-path rows are missing, and the Completion Notes still say "ten"
      [_bmad-output/implementation-artifacts/11-1b-the-air-has-depth.md:398-409] — MED.
      [acceptance + orchestrator]
- [ ] [Review][Patch] Task 5's card OVERWROTE 11.1a's card, destroying a sibling story's live
      deliverable [_bmad-output/implementation-artifacts/11-1-signoff/task-5-vehicle-card.md] — MED.
      [acceptance + orchestrator]
- [ ] [Review][Patch] Two mutation rows are WEAK KILLS: retargeting the key to `F13` panics
      `lighting_readout`'s `unreachable!()` arm before any key assertion discriminates; swap
      F11<->F12 instead [_bmad-output/implementation-artifacts/mutations/11-1b-the-air-has-depth.sh:59-73]
      — MED. [blind + orchestrator]
- [ ] [Review][Patch] A re-pointed 11.1a mutation row now sabotages the readout LABEL, not its
      STATE, so nothing anywhere sabotages the on/off literal and AC7's state clause is unprotected
      [_bmad-output/implementation-artifacts/mutations/11-1a-a-chosen-exposure-and-a-clean-edge.sh]
      — MED. [acceptance]
- [ ] [Review][Patch] `README.md:201` still promises `--static-world` makes "two captures differ
      only by what you changed"; the fix pauses the DAEMON while `fall_snow` and `flicker_projection`
      run on the wall clock [README.md:201, crates/gui/src/atmosphere.rs:321] — MED, latent
      silent-failure trap. [orchestrator + feature]
- [ ] [Review][Patch] Published noise floors are lucky-tight pairs: the "600x" margin divides by a
      TWO-sample 0.001 floor, and three independent layers read 69.452-69.487 (spread 0.035, >=35x
      it) — one of them BELOW the guard comment's own stated range
      [crates/gui/tests/pixel_guard.rs:165-172] — MED. [orchestrator + feature + edge]
- [ ] [Review][Patch] Wolf's #106 ruling (AO accepted as wired, NOT visible from the seat, look
      deferred to 11.3) is recorded nowhere but the issue — not in this story, not on the board
      [_bmad-output/implementation-artifacts/11-1b-the-air-has-depth.md] — MED, latent: 11.2/11.3
      inherit a false premise. [orchestrator]
- [ ] [Review][Patch] `docs/tech-art-guidelines.md` ships two wrong rows: bloom says "AC4 remains
      blocked" after AC4 was met, and AO claims a creases-concentrated signature #106 measured as
      broadly uniform [docs/tech-art-guidelines.md:68-69] — MED. [orchestrator + feature + acceptance]
- [ ] [Review][Patch] The File List is badly stale — ~25 changed files unlisted, including
      `command.rs` which carries the headline change, four new instrument scripts and 16 PNGs; and
      `tests/headless.rs` is listed UPDATE but was never touched
      [_bmad-output/implementation-artifacts/11-1b-the-air-has-depth.md] — MED.
      [acceptance + orchestrator]
- [ ] [Review][Patch] `sprint-status.yaml` promises "two open items" and lists one [_bmad-output/implementation-artifacts/sprint-status.yaml]
      — LOW, folded in with the board update. [acceptance + orchestrator]
- [ ] [Review][Patch] `--lights-steady` is documented nowhere — absent from README's flag table and
      all of `docs/` [README.md:190-204] — LOW, folded in with the README patch above.
      [orchestrator + feature + acceptance]

- [x] [Review][Defer] `11-1-signoff/campstats.py:12` hardcodes an absolute repo path where its
      sibling `creases.py:14-17` derives it from `__file__` — deferred, LOW tail; fails loudly, not
      silently. [acceptance]
- [x] [Review][Defer] `lighting_readout`'s key->string match carries a live `unreachable!()` arm
      [crates/gui/src/ingest.rs:1490-1496] — deferred, LOW tail; a maintenance landmine for whoever
      extends `CameraEffect::ALL`, not a live bug. [blind]
- [x] [Review][Defer] `CameraEffect::from_name`'s error text hardcodes the accepted-name list rather
      than deriving it from `CameraEffect::ALL` [crates/gui/src/ingest.rs:185] — deferred, LOW tail;
      correct today, lies on a fourth effect. [feature]

## Dev Notes

### Scope guardrails — do NOT

- Do NOT re-add `Msaa::Off`, `Exposure` or `Fxaa` — 11.1a owns them. Do NOT change the EV100.
- Do NOT add depth of field, volumetric fog, a day/night cycle, a moon or a clock. Those are 11.2
  and 11.3.
- Do NOT add a dependency or change a `bevy` feature. `3d_bevy_render` already carries
  `bevy_pbr`, `bevy_post_process` and `bevy_anti_alias`.
- Do NOT touch `protocol`, `sim-core`, `client-core` or `simd` (AC10).
- Do NOT change a light constant, a colour, or a `BOOT_*` constant. This story adds MECHANISMS; the
  look re-judgement under them is Wolf's at the sitting. `bench_contract.rs:128-162` greps
  `camera.rs` for those literals.
- Do NOT raise `NEAR_WHITE_AREA_CEILING` or `BLOWN_POOL_FRACTION_CEILING` (AC5).
- Do NOT build a general post-effect registry. A set of three named effects, mirroring the existing
  light toggles, is the whole of it.

### Key decisions & traps

- **The lavapipe question is ANSWERED for both mechanisms, and it stays answered.** Probed through
  wgpu 29.0.4 on this devpod (`llvmpipe (LLVM 19.1.7, 256 bits)`, Vulkan, Mesa 25.0.7):
  `max_storage_textures_per_shader_stage = 48`, so SSAO's only GPU gate (`>= 5`,
  `ssao/mod.rs:61-70`) passes; and `Rgba16Float` reports `RENDER_ATTACHMENT`, `STORAGE_BINDING`,
  `TEXTURE_BINDING` and `FILTERABLE`, so `Hdr` has its format. **This proves the plugins load and
  the formats exist — not that either effect looks right.**
- **AO's failure mode is silence, and it is the reason this story exists.** If AO appears to do
  nothing, check `Msaa` before blaming lavapipe. The `error!` goes to the log, the capture is clean,
  the exit code is 0.
- **Exit 0 is not a result.** Range-check what the instrument printed.
- **`--frames 160`** — `--frames 2` never captures (dies on `capture is black`, `capture.rs:1419`).
  `--cursor` is dead headless.
- **The PNG is written before the range check asserts** (`save_then_validate`, `capture.rs:1276`),
  so a capture that exits 101 still leaves a measurable frame. Both creation controls used this.
- **Rec.601 vs Rec.709.** `creases.py` and `pixel_guard.rs:35-41` are Rec.601 integer;
  `capture.rs:604` is Rec.709. Say which any new figure is in.
- **A mutant binary outlives a source restore.** Rebuild and re-check `--version` after any
  mutation run before taking a capture.
- **Two samples are not a floor.** Four minimum. Issue #98 retracted a floor that was one
  lucky-tight pair.

### Open question inherited from 11.1a, and it may land on this story

**The `enclosed_sky` guard has gone vacuous and Wolf has not yet ruled.** 11.1a moved its residual
from 2,042 px to **11** px against an unchanged `ENCLOSED_SKY_CEILING` of 2,300
(`pixel_guard.rs:478`), because FXAA blends the edge pixels that used to match the sky colour
exactly. It passes, and it now tolerates 209x its own value — it has stopped discriminating.
**RULED AND CLOSED 2026-09-19 — see below.** (Original instruction: do not move it without Wolf's explicit ruling; if he rules while this story is in
flight, the change belongs wherever he says. Re-measure it either way and record the figure, because
`Hdr` will move it again.)

**Wolf ruled 2026-09-19: RETIRE it, and rebuild the oracle as its own story (issue #108).** The
guard was not vacuous, it was **inert**. `const SKY = [5,12,28]` is an EXACT RGB match; `Hdr` moved
the rendered sky to `[7,15,31]`, so **zero** pixels classified as sky where **8,434** did pre-`Hdr`.
It returned 0 holes / 0 blobs because it could not SEE sky — it would have passed with the terrain
entirely absent. Repair is a story, not a re-baseline: post-stack, dark shadowed terrain and night
sky **overlap in colour space**, which is the discrimination the unique sky colour gave for free.
Two measured attempts were rejected (a self-calibrated exact match fragments the fill's
connectivity; a tolerant rule calibrated from the frame's own top rows reports ~21,000 false holes
in ~790 blobs). The flood-fill port itself is sound — it reproduces the recorded 11 px / 7 blobs
pre-`Hdr`, and a synthetic 40x40 sky patch painted into the terrain raises the count by exactly
1,600 px / 1 blob. The test and its `enclosed_sky` helper are removed, with a ledger comment in
`pixel_guard.rs` naming what is still covered (the geometric mask tests) and what is not.

### Project Structure

| File | Change |
| --- | --- |
| `crates/gui/src/ingest.rs` | UPDATE — SSAO + Bloom on the camera tuple; `FxaaOff` → an effect set; `--lights-steady`; two keys; readout |
| `crates/gui/tests/headless.rs` | UPDATE — live-entity and key-path tests |
| `crates/gui/tests/pixel_guard.rs` | UPDATE — AC2's crease pair, AC3's MSAA guard, AC4's bloom pair |
| `crates/gui/tests/capture.rs` | UNCHANGED — AC10 asserts this |
| `docs/tech-art-guidelines.md` | UPDATE — the AO and bloom rows beside 11.1a's exposure row |
| `_bmad-output/implementation-artifacts/11-1-signoff/task-5b-vehicle-card.md` | NEW |
| `_bmad-output/implementation-artifacts/mutations/11-1b-the-air-has-depth.sh` | NEW |

### Verification

From the repo root. **`scripts/gate.sh` has no thread knob and its Bevy test apps exhaust 23 GB —
export `RUST_TEST_THREADS=6` and run it in the FOREGROUND.** The full gate is ~420s; the pre-commit
hook is the FAST tier at ~50-66s. **Never `git commit --no-verify`** — 11.1a did it once and put a
red commit on the branch. If the gate is red, fix the cause or say it is red.

```bash
RUST_TEST_THREADS=6 scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/11-1b-the-air-has-depth.sh
git status --porcelain     # issue #104: mutate.sh leaves tracked non-Rust targets sabotaged
```

**The control, which RAN at creation** (figures in the table above):

```bash
./target/debug/simd 7466 &
./target/debug/gui 7466 --headless --static-world --subdiv 4 --frames 160 \
  --capture 11-1-signoff/control-<sha>-a.png
# creation observation on 41b3f02, exit 0:
#   capture range check: warm-lit pixels=28772 ground-median-luminance=81
#     near-white-area=0.8584% blown-pool=0.5872% p99-luminance=192.5 resolution=1280x720
```

**THE DELIBERATE RED — this is the one that matters, and it is AC3's whole purpose.** With AO on,
put MSAA back:

```bash
# RED: change `Msaa::Off` to `Msaa::Sample4` in the camera tuple (ingest.rs:1308-1335).
# EXPECTED: AC3's guard goes RED, and `terrace-creases` p10 returns to its no-AO value —
#           because extract_ssao_settings returns out of the loop and skips EVERY camera.
#           The capture still succeeds and still exits 0. THAT is the point: nothing else
#           in the system notices.
# RESTORE: revert to `Msaa::Off`, rebuild, confirm `gui --version` shows no -dirty, re-capture.
```

**If that RED does not redden the guard, the guard is worthless and AC3 is unmet** — no matter how
green the suite is. A guard that only checks the `Msaa::Off` component would pass this RED trivially
while AO silently does nothing, which is the exact defect.

### References

- `_bmad-output/planning-artifacts/epics.md` § Epic 11 / Story 11.1 — source; carries the split, the
  FXAA ruling, the struck bloom clause and the lavapipe note
- `11-1a-a-chosen-exposure-and-a-clean-edge.md` — the render path this story stands on
- `11-1-signoff/creases-proof.md` (instrument GREEN/RED), `fxaa-silhouette.md` (the exact-sky
  method), `task-0-control.md` (both builds' control figures and the ceiling-headroom finding)
- Pinned Bevy 0.19.0: `bevy_pbr/src/ssao/mod.rs:61-70, 113, 479-494` ·
  `bevy_post_process/src/bloom/settings.rs:32, 132-175` · `bevy_internal/src/default_plugins.rs:61, 79`
- `deferred-work.md:1237-1246` — llvmpipe under-reads near-white by ~16 %; headless area figures are
  deltas only
- NFR6 — 60 fps at working zoom, ≥30 at full vista, read on the vehicle with `--perf-log`
- Open issues: **#104** (`mutate.sh` leaves tracked targets sabotaged — hit this story's `creases.py`
  directly), **#90** (near-white swing), **#75** (lighting re-judgement), **#72** (pixel-guard race)

### Previous-story intelligence

- 11.1a's `--fx-off` is **insert/remove**, not an `enabled` flag, and its reaches-the-camera test
  (`ingest.rs:2171`) is the shape to copy. `let _ = value;` left all 106 tests green once here.
- 11.1a broke a **10.7** mutation row by inserting one line into `projection_systems`, and the gate
  caught it. Expect the same and budget for the re-point.
- 11.1a's dev was delegated across three Codex runs; the ones that handed back honestly at their
  context limit were resumable **because each task was committed green**. Commit per task.
- Branch off `story-11-1a-exposure-and-clean-edge` at `d3ecdff`, slug
  `story-11-1b-the-air-has-depth`. Commit as `Völundr <jeicei75@gmail.com>` on the **first** commit —
  this clone's git config defaults to `jeicei75`. No `Co-Authored-By: Claude` trailer and no
  "Generated with Claude" footer (CLAUDE.md §8).

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- Task 0 (2026-09-18): `gui build f604b40` before capture. Four `--headless --static-world --subdiv 4 --frames 160` controls: crease windows were Rec.601 p10/median-stable (terrace 31/65; LL 68/117; LR 69/117). Camp `(500,400)..(760,620)` was Rec.601: median 82/81/85/84 (floor 4), p90 200/195/212/199 (floor 17), near-white >=230 4.7552/4.2587/6.0245/4.4563% (floor 1.7658 pp).
- Task 1 RED: `cargo test --offline -p gui lights_steady_reaches_the_live_flicker_system` failed before implementation with `error[E0425]: cannot find type LightsSteady in this scope` at `ingest.rs:2204` and `:2208`. GREEN: the focused test passed after the resource, parser branch, app wiring, and fixed 0.0-second branch were added. The first hook run also exposed standalone `projection_systems` test apps lacking the resource; `projection_systems` now initializes it alongside its other resources.
- Task 1 mutation (2026-09-18), `the --lights-steady value is discarded before the live flicker system`: KILLED. Output: `thread 'ingest::tests::lights_steady_reaches_the_live_flicker_system' ... panicked at crates/gui/src/ingest.rs:2227:9: assertion failed: steady.world().resource::<super::LightsSteady>().0`; `test result: FAILED. 0 passed; 1 failed`.
- Task 1 measure (2026-09-18): `gui build 7d13828` before capture. Four `--headless --static-world --lights-steady --subdiv 4 --frames 160` samples, all successful headless captures. Camp `(500,400)..(760,620)`, Rec.601: median 82/83/83/82 (spread 1), p90 209/209/209/209 (spread 0), near-white >=230 5.4441/5.5227/5.5839/5.8566% (spread 0.4125 pp). AC1 requires each spread below one tenth of Task 0's 4/17/1.7658 floors: <0.4 median, <1.7 p90, <0.17658 pp near-white. Median and near-white fail; #90 already tracks this same-build near-white instability. Stopped before Task 2 as the story directs.
- Task 2 GREEN (2026-09-18): `gui build 8edc62a` before four `--headless --static-world --subdiv 4 --frames 160` captures. `creases.py` Rec.601 figures were stable: terrace p10/median = 30/64 (Task 0 control 31/65, p10 floor 0); open-snow-LL median = 117 and open-snow-LR median = 117 (both exactly their Task 0 controls). The rendered guard `ambient_occlusion_darkens_terrace_creases_and_msaa_cannot_silently_disable_it` passed.
- Task 2 deliberate RED: changing only `Msaa::Off` to `Msaa::Sample4` made the rendered guard fail (`terrace p10=32`, open-snow medians 117/117) at `pixel_guard.rs:264`, while the process still rendered. A dirty standalone mutant logged Bevy's SSAO/MSAA error and `creases.py` measured terrace p10=32, LL/LR medians 117/117 (Rec.601). Thus MSAA restored the no-AO rendered consequence and the guard reddened as required. Source was restored before the post-RED rebuild.
- Task 2 post-RED restore (2026-09-18): rebuilt source and confirmed `gui build 621ef4f` with no `-dirty` before capture. Rec.601 `creases.py`: terrace p10/median 30/64, open-snow LL/LR medians 117/117. The re-measured `enclosed.py` residual at subdiv 2 was 11 px in 7 blobs; the unchanged 2,300-px ceiling was not moved.
- Task 6 scope rows (2026-09-18): first mutation-table run correctly reported both ignored AO rows NOT-RUN because their targets omitted the runner's required `ignored` tier; no result was claimed. After adding that tier, rerun output was all KILLED: strengthened steady consequence (`assertion left == right failed: --lights-steady must pin the PointLight intensity the live flicker system writes`, `ingest.rs:2273`); AO omitted and MSAA re-enabled (both rendered guard failures at `pixel_guard.rs:264`, 64.19 s and 78.24 s respectively). `scripts/mutate.sh` restored Rust source; only this table and newly recorded PNGs remained changed afterward.
- Final verification for this handoff: `RUST_TEST_THREADS=6 scripts/gate.sh` was GREEN in 461 s, including rendered pixel guards and mutation-table audit. Three `codex review --base 77d5056` passes were attempted after the gate (the latter two after record-only commits); none could inspect any diff because every shell invocation hit the known read-only `/tmp` mount-registry lock before execution. None produced a finding. The three-pass cap is now exhausted; no fourth review was run.
- Self-gate: one `codex review --base 77d5056` pass was attempted. It did not start review work because its sandbox reported every command (including `git diff`) blocked by a read-only mount-registry lock under `/tmp`. No second pass was run; this is environmental, not a review result.
- Task 3/4 (2026-09-18): `gui build 67ba364` (clean, no `-dirty`) before five headless captures. `Bloom::default()` / Natural was selected because it is the requested default energy-conserving mechanism, not a look-tuned preset. Hdr/no-bloom four-capture Rec.601 camp floor: median 82/82/81/81 (spread 1), p90 208/208/203/204 (spread 5), near-white >=230 4.9231/5.3409/4.8864/5.1364% (spread 0.4545 pp). The Hdr+bloom sample was median 95, p90 207, near-white 5.2255%: p90 fell one level rather than rising beyond the measured floor, so AC4 is unmet and work stopped as directed. Bloom's headless whole-frame near-white delta was -0.0236 pp (0.7879% no-bloom a to 0.7643% bloom); no ceiling was asserted or changed. Rec.601 `creases.py` snow medians moved from 117/117 (Hdr/no-bloom) to 116/116 (bloom), so the marginal contribution was -1/-1. The requested enclosed-sky re-measurement was not completed before the AC4 stop condition.
- Task 4 (2026-09-18): replaced `FxaaOff(bool)` with the fixed three-member `CameraEffect` / `EffectsOff` set. `--fx-off` accepts `fxaa`, `ao`, and `bloom`; F10/F11/F12 toggle them with live component insertion/removal and the full readout. Focused tests for the live `--fx-off` and real-key paths passed. Legacy 10.7 and 11.1a mutation anchors were re-pointed; the pre-commit mutation audit then passed. The story's new Task 6 rows were not added or run because AC4 invoked its explicit stop rule.
- Task 2 guard repair (2026-09-18): `Hdr` erased the former p10 signal: fresh Rec.601 terrace p10 was 38 with AO on (four samples) and 38 with AO off, matching the all-effects-on and bloom-off checks. Four clean `gui build 5a1ed7b` AO-on captures measured terrace means 69.515/69.505/69.489/69.466 (floor 0.049); a fresh AO-off capture measured 70.016. The rendered guard now uses the terrace mean with a 69.75 ceiling (0.234 below the ceiling at the highest AO-on sample; 0.266 above it when AO is off). Hdr also moved both stable open-snow medians to 116, which is their new exact Hdr control.
- Task 2 guard repair RED (2026-09-18): after changing only the camera tuple from `Msaa::Off` to `Msaa::Sample4`, the mean-based rendered guard failed in 78.51 s: `AC2/AC3 pixel guard (Rec.601): terrace mean=70.756; open-snow LL/LR median=117/117`; `SSAO must visibly darken terrace creases ... mean=70.756, but AO-on must stay below 69.75` at `pixel_guard.rs:264`. The source was restored to `Msaa::Off` before the clean rebuild.
- Task 2 guard repair restore (2026-09-18): after committing and rebuilding, `gui build a5ee674` named a clean tree before the required post-RED capture. `creases.py` measured terrace p10/median/p90/mean 38/65/114/69.441 Rec.601; open-snow LL/LR medians were 116/116. The focused enclosed-sky guard re-measured 0 pixels in 0 blobs; its 2,300-pixel ceiling was not changed.
- Task 5 card (2026-09-18): wrote the gingerspice sitting recipe for all-effects-on plus each individual `--fx-off` perf-log run, and named `effects-on.png` / `bloom-off.png` as Wolf's frame pair. No vehicle window was opened and no frame-time figure was observed.
- Task 6 full table (2026-09-18): after committing the table before mutation, `scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/11-1b-the-air-has-depth.sh` reported all ten rows KILLED. The two ignored rendered guards ran in their required ignored tier; neither had an APPLY-FAILED, NOT-RUN, or NO-COMPILE result. The harness restored its Rust target, and `git status --porcelain` was clean immediately after it. Rebuilt restored source and confirmed `gui build f8271de` before the later full gate. `rg` found the existing 11.1a `Msaa::Off` / `Fxaa::default()` and 10.7 readout anchors as well as this story's new tuple/readout rows; `scripts/audit-mutations.py` confirmed all 585 repository rows still apply, so no additional re-point was needed.
- Final verification (2026-09-18): `RUST_TEST_THREADS=6 scripts/gate.sh` ran in the foreground and was GREEN in 480 s: fmt 0 s, clippy 2 s, workspace tests 118 s, rendered pixel guards 328 s, metrics 1 s, bench 18 s, and mutation-anchor audit 4 s. No additional `codex review --base 77d5056` pass was run: the Dev Agent Record already documents the story's three allowed attempts as environmental `/tmp` mount-registry failures, so the three-pass cap was exhausted before this handoff.

- Task 1b resolution (2026-09-19), issue #105: `gui build 164ab55`, clean tree, before four
  `--headless --static-world --lights-steady --subdiv 4 --frames 160` captures. Camp
  `(500,400)..(760,620)`, Rec.601: median 95/95/95/95 (spread **0**), p90 208/208/208/208 (spread
  **0**), p99 250 x4, mean 121.309/121.292/121.307/121.298 (spread 0.017), near-white >=230
  5.9545/5.9528/5.9528/5.9528 % (spread **0.0017 pp**). AC1 required each below a tenth of Task 0's
  4 / 17 / 1.7658 floors, i.e. <0.4, <1.7, <0.17658 pp. **All three met, by margins of 235x-100x.**
  The cause of the old residual was NOT flicker: `--static-world` silenced the capture's motion
  assertions and never paused anything, so the dwarves kept walking their lanterns through the camp.
- Task 3b (2026-09-19), issue #107: bloom-off control is `--fx-off bloom`, which leaves `Hdr` in
  place (removing a component does not remove what `#[require]`d it), so this is bloom's MARGINAL
  contribution and not the pipeline's. Four captures each, same build, Rec.601 camp window:

  | statistic | bloom OFF | bloom ON | floor | delta |
  | --- | ---: | ---: | ---: | ---: |
  | median (halo) | 82 | 95 | 0 | **+13** |
  | mean (halo) | 114.320 | 121.302 | 0.035 | **+6.98** |
  | p90 (bright tail) | 210 | 208 | 0 | -2 |
  | p99 (bright tail) | 251 | 250 | 0 | -1 |
  | near-white % | 6.16 | 5.95 | 0.0455 pp | -0.21 pp |

  This is the `EnergyConserving` signature read against a zero floor: the halo rises hard, the bright
  tail falls slightly because the energy came OUT of the cores. The original AC4 clause asked the
  bright tail to rise and was therefore unsatisfiable by construction. Open snow moved 117 -> 116
  (LL and LR), i.e. it DARKENED by one level; it did not brighten, which is what the amended AC asks.
- Attribution correction (2026-09-19): the open-snow 117 -> 116 shift was recorded three times as
  `Hdr`'s. It is **bloom's**. `Hdr` and `Bloom` landed in one commit and no control ever separated
  them; `--fx-off bloom` does, and reads 117/117 with `Hdr` still on.
- Task 2c re-measure (2026-09-19) on the paused world, Rec.601 `terrace-creases` mean: AO on
  69.485/69.486 (floor 0.001), AO off 70.087/70.086 (floor 0.001), **delta -0.601 against a 0.001
  floor -- 600x**. `p10` reads 38 on both sides, which is why AC2/AC3 moved to the mean. The 69.75
  guard ceiling sits 0.265 below the AO-on reading and 0.337 above the AO-off one.
- `--static-world` test shape (2026-09-19): the focused test reads the SOCKET, not the resource.
  Throughout the defect's whole life the resource was set correctly and simply went nowhere, so a
  resource assertion would have passed the entire time. RED proved by unregistering the startup
  system: `--static-world must write a command to the daemon: Os { code: 11, kind: WouldBlock }`.
- Task 6 (2026-09-19): all **twelve** rows KILLED, including the two new pause-path rows covering
  both failure modes (never sent; sent as `Normal` instead of `Paused`). `git status --porcelain`
  after the run showed no tracked file left sabotaged (issue #104).

```
the strengthened --lights-steady consequence is discarded before the live flicker system KILLED
ambient occlusion is omitted from the live camera            KILLED
MSAA is re-enabled while ambient occlusion is present        KILLED
bloom is omitted from the live camera                        KILLED
--fx-off fxaa discards the named FXAA effect                 KILLED
--fx-off ao discards the named ambient-occlusion effect      KILLED
--fx-off bloom discards the named bloom effect               KILLED
F11 toggles bloom instead of ambient occlusion               KILLED
F12 toggles ambient occlusion instead of bloom               KILLED
the effect readout stops recording changed state             KILLED
--static-world never reaches the daemon                      KILLED
--static-world asks the daemon to run instead of pause       KILLED
the --static-world pause is sent before its chosen tick      KILLED
the pause is believed from our own request, not the daemon report KILLED
Space resumes a --static-world run                           KILLED
a --static-world run never hands the daemon back             KILLED
--fx-off ao leaves AO required prepasses running             KILLED
--fx-off bloom takes Hdr with it, destroying AC4 control     KILLED
bloom is retuned to the OLD_SCHOOL preset                    KILLED

All mutations killed.
```

**AC11 (2026-09-19, after the review patches).** NINETEEN rows, all KILLED, `git status --porcelain`
clean immediately after (issue #104). The pre-review record pasted a TEN-row block while the table
held twelve and the prose claimed twelve; the block above is this table's real output.

The table is re-run **after** every guard change, not before. Two rows earned their place by
failing first:

- `bloom is retuned to the OLD_SCHOOL preset` **SURVIVED** the first run of AC4's new guard.
  `OLD_SCHOOL` leaves the `Bloom` component present and is `Additive`, so it ADDS energy and still
  cleared the halo-rise floors. The row was right and the guard's own comment had overclaimed. The
  near-white clause that now separates the composite modes was written because of it, and the kill
  lands on THAT assertion (`pixel_guard.rs:350`), not on an earlier one absorbing the mutation.
- The AO and bloom guards both gained `--lights-steady` after the near-white clause failed a live
  run reading bloom RAISING camp near-white by 1.03 pp. Every row targeting either guard was
  re-mutated against the changed guards; the 18-row result taken before those edits is evidence for
  the old guards only and is not the table above.

### Review patch pass (2026-09-19) — measurements, and two corrections to the review itself

All measurements below: build `6140ca3`, clean tree (`gui --version` checked), headless llvmpipe,
Rec.601 integer luma, `--static-world --lights-steady --subdiv 4 --frames 160`.

**THE RACE IS REAL AND THE FIRST FIX FOR IT DID NOT WORK.** Wolf ruled "fix the race, then
re-measure". The first attempt targeted a fixed tick client-side. It failed, and the failure is the
finding: the pause fired at ticks **30, 66, 101, 136, 171** across five runs, never at the target 8,
because the daemon had been running before each client connected and the world was already past the
gate at connect. The frozen worlds were ~35 ticks apart -- WORSE than the 3-tick spread it was meant
to remove. A client-side target tick cannot work when the world is already past it.

**WHAT DOES WORK IS ONE DAEMON PER CAPTURE**, and the difference is not marginal:

| recipe | camp median | p90 | p99 | mean | near-white |
| --- | ---: | ---: | ---: | ---: | ---: |
| fresh `simd` per capture (n=4) | 0 | 0 | 1 | 0.020 | **0.0070 pp** |
| one shared `simd` (n=5) | 1 | 2 | 1 | 0.867 | **0.3619 pp** |

AC1's bar is <0.4 median, <1.7 p90, <0.17658 pp near-white. Fresh daemons clear it by **25x**; a
shared daemon **fails it by 2.05x**. With fresh daemons the pause landed at **tick 40 in four runs
out of four**. This exactly reproduces the code review's independent 0.5210 pp figure, which was
taken across twelve captures against one daemon -- that reading was right, and it was measuring the
recipe, not the flag. **AC1 stands, with a precondition nobody had written down.** The recipe is now
in `README.md`, in the vehicle card, and enforced inside both rendered guards.

**AC2/AC3 re-measured after the prepass change**, fresh daemon per capture:
terrace mean AO-on 69.499 / 69.511 (floor 0.012), AO-off 70.142 / 70.125 (floor 0.017),
**darkening 0.62-0.64 against a 0.017 floor, about 36x**. Guard bar 0.30.

**AC4 re-measured**, camp window, fresh daemon per capture: bloom-on median 95/95 mean
120.334/120.288; bloom-off median 82/82 mean 113.427/113.435. **Median +13, mean +6.88**, against
same-build floors of 0 and 0.046. Independently reproduces the 2026-09-19 figures.

**CORRECTION 1 — the review's AC3 finding was overstated, and this is the review correcting
itself.** It was reported as "the guard passes with SSAO absent once bloom is off". Measured:
`--fx-off ao,bloom` gives terrace mean **68.403**, which does clear the old 69.75 ceiling -- but the
same capture reads open-snow LL/LR median **117** against the guard's `== 116`, so **the guard still
goes red**. The true finding is narrower and still worth the fix: the guard's STATED mechanism (a
terrace level) stopped discriminating once bloom was off, and what actually caught AO's absence was
the open-snow clause the story calls an unrelated control, via a one-level quantisation boundary.
The guard is a delta now, so the stated mechanism is the one doing the work.

**CORRECTION 2 — the published margins were overstated and are now measured.** "delta -0.601
against a 0.001 floor -- 600x" divided by a TWO-sample floor, against this story's own Dev Note
("Two samples are not a floor. Four minimum. Issue #98"). Three independent review layers read the
same statistic on one build at 69.452 / 69.485 / 69.486 / 69.487 / 69.487 -- a spread of **0.035,
at least 35x the published floor**, and one of those readings falls BELOW the range the guard's own
doc comment stated. The real margin is about **36x**, not 600x. Nothing was failing; the number was.

**`remove_with_requires` CRASHES and was reverted to naming the two prepasses explicitly.** Every
`--fx-off ao` run died in `bevy_render::sync_world` with "Attempting to synchronize an entity that
has already been synchronized!": it removes the whole transitive require closure, which overlaps
components the camera needs for its own render-world sync. The unit test passed throughout, because
`MinimalPlugins` has no render world -- a component assertion cannot see a client that cannot draw.

**Issue #106's ruling, recorded here because it lived only on the issue.** Wolf ruled 2026-09-18,
re-measured at `164ab55`: **AO is accepted as WIRED AND PROVED, it is NOT visible from the seat, and
its look judgement moves to 11.3.** SSAO attenuates only the ambient term and this scene tunes
ambient deliberately small (1,500 against a 7,000 directional and 7,000,000 lm point lights,
`appearance.rs:37-48`), so AO's strongest effect anywhere in frame is ~2% of base luminance and is
broadly uniform rather than concentrated at contacts. #106 stays OPEN for its second half: AC2 as
written cannot distinguish "consumed" from "visible". The review measured that the same defect
applies to **AC4's** "does not brighten" clause -- open-snow LL's MEAN rises 0.686 under bloom while
its MEDIAN falls -- and Wolf ruled 2026-09-19 to record that on #106 rather than rewrite either AC
mid-story. **AC2 and AC3 are a MECHANISM proof and are not a claim that AO reads from the seat.**

**FULL GATE GREEN, 524s, run in the foreground of the review session on `9609274`, clean tree.**
fmt 0s · clippy 2s · workspace tests 134s · rendered pixel guards 357s · crate-edge probes ok ·
metrics 0s · bench 19s · mutation-anchor audit 4s (595 rows). The rendered tier grew from 328s to
357s because AC4's guard is new and both guards now take two captures against two daemons.

**Review scaffolding reaped**: `scripts/reap-build-caches.sh --tmp-only --force` removed 8
directories under /tmp and reclaimed **51.5 GB**. The repository's own `target/` was not touched.

**Review cost**: $100.61 over 800 turns, 148.8M cache-read tokens (98.0% of everything processed),
four subagent transcripts accounting for 33.6%. That is well above Epic 3's $45.52/story baseline,
and the reason is visible rather than mysterious: this review had no self-gate to lean on (#49), ran
four layers that each built and ran the binaries, and then carried a nineteen-item patch pass with
three rendered-guard rebuilds, two full mutation-table runs and a full gate inside the same session.

### Completion Notes List

- Task 0: captured the build-specific no-AO/no-bloom controls. The camp flicker floor confirms Task 1 must pin the live flicker clock before bloom is measured.
- Task 1: implemented and mutation-proved the live `--lights-steady` path, but AC1 is blocked by residual camp-window variance after the clock is pinned. Tasks 2–6 were intentionally not started; Task 5 remains vehicle-only.
- Task 3: Bloom is installed but AC4 is blocked: its bright-tail p90 did not rise over the new build-specific Hdr/no-bloom floor. No headless ceiling was changed.
- Task 4: the three-effect command and seat controls are implemented and focused-test green; its story mutations remain deferred by the AC4 stop rule.
- Task 6: added and ran the remaining bloom, per-name `--fx-off`, F11/F12, and readout sabotages; all ten table rows KILLED. The full repository anchor audit passed after checking the legacy camera/readout rows.

### File List

REBUILT 2026-09-19 at the code review, which found the previous list omitted about 25 changed
files -- including `crates/gui/src/command.rs`, which carries this story's headline change -- and
listed `crates/gui/tests/headless.rs` as UPDATE when it was never touched. Enumerated from
`git diff --name-only d3ecdff..HEAD`, not from memory. 67 files; the 41 signoff PNGs are named by
their families rather than one line each.

**Source**

- `crates/gui/src/ingest.rs` (updated) -- SSAO + Bloom on the camera tuple; `FxaaOff` -> the
  `CameraEffect`/`EffectsOff` set; `--lights-steady`; `--static-world` wiring; F11/F12; readout;
  the review's pause-handshake systems and the `toggle_pause` -> `send_commands` ordering edge
- `crates/gui/src/command.rs` (updated) -- `StaticWorld`, `StaticWorldPause`, `pause_static_world`,
  `confirm_static_world_pause`, `restore_speed_on_exit`, and `toggle_pause`'s `--static-world` guard
- `crates/gui/src/capture.rs` (updated, at the review) -- the capture countdown holds until the
  daemon reports itself paused; `ScriptedInput` bundles two params to make room for it
- `crates/gui/tests/pixel_guard.rs` (updated) -- AC2/AC3's rendered guard (a DELTA since the
  review), AC4's rendered bloom guard (NEW at the review), and the retired `enclosed_sky` helper
- `crates/gui/tests/capture.rs` -- UNCHANGED, and AC10 asserts it

**Docs**

- `README.md` (updated, at the review) -- `--static-world` says what it actually freezes;
  `--lights-steady` and `--fx-off` documented for the first time
- `docs/tech-art-guidelines.md` (updated) -- the AO and Bloom rows, corrected at the review

**Process and evidence**

- `_bmad-output/implementation-artifacts/11-1b-the-air-has-depth.md` (updated)
- `_bmad-output/implementation-artifacts/11-1a-a-chosen-exposure-and-a-clean-edge.md` (updated)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (updated)
- `_bmad-output/implementation-artifacts/deferred-work.md` (updated, at the review)
- `_bmad-output/implementation-artifacts/metrics/11-1b-the-air-has-depth.md`,
  `metrics/.session-cursors.json` (updated)
- `mutations/11-1b-the-air-has-depth.sh` (new; 18 rows after the review added six and fixed two
  weak kills), `mutations/11-1a-a-chosen-exposure-and-a-clean-edge.sh` (updated; readout row split
  and F10 row de-weakened), `mutations/10-7-the-sun-lights-the-valley.sh`,
  `mutations/6-1-the-world-moves.sh` (updated; re-pointed)
- `11-1-signoff/task-5-vehicle-card.md` -- RESTORED to 11.1a's card at the review
- `11-1-signoff/task-5b-vehicle-card.md` (new, at the review) -- 11.1b's card, at the path the
  Project Structure table always named
- `11-1-signoff/campstats.py`, `delta.py`, `markdiff.py`, `residual.py` (new instruments)
- `11-1-signoff/emitter-window-flicker-floor.md`,
  `11-1-signoff/residual-after-the-flicker-pin.md`, `11-1-signoff/wolf-seat-check-d3ecdff.md` (new)
- 41 new signoff PNGs under `11-1-signoff/`: `task-0-f604b40-*`, `task-1-7d13828-*`,
  `task-2-8edc62a-*`, `task-2-621ef4f-*`, `task-2b-5a1ed7b-*`, `task-2b-a5ee674-restored`,
  `task-3-hdr-no-bloom-67ba364-*`, `task-3-bloom-67ba364-a`, `task-3b-164ab55-nobloom-*`,
  `task-1b-164ab55-paused-*`, `task-2c-164ab55-noao-*`, `task-2d-*`, `residual-b50235b-*`

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-18 | Created, stacked on 11.1a. Lavapipe answered for both mechanisms (48 storage textures; `Rgba16Float` is a filterable render attachment). Creation measured the camp window's flicker floor at 1.4423 pp near-white / 18 levels p90 and found `flicker_lights` runs off the wall clock, so `--static-world` does not stop it — Task 1 fixes the instrument before bloom leans on it. |
| 2026-09-18 | Task 0: captured the f604b40 build controls. Crease p10/median floor was 0; the camp's unpinned Rec.601 floor was 4 median levels, 17 p90 levels, and 1.7658 pp near-white. |
| 2026-09-18 | Task 1: added and mutation-proved `--lights-steady`, but the pinned camp capture still spread by 1 Rec.601 median level and 0.4125 pp near-white (AC1 requires <0.4 and <0.17658 pp). Stopped before AO/bloom; #90 already records the matching near-white instability. |
| 2026-09-18 | Task 2: added default SSAO and a rendered Rec.601 guard. Four 8edc62a captures put terrace p10 at 30 against the Task 0 control 31 (floor 0), with both open-snow medians unchanged at 117. The deliberate MSAA-on RED failed the guard and returned terrace p10 to 32; bloom remains blocked by #105. |
| 2026-09-18 | Task 2 restored capture again read terrace p10 30 and both open-snow medians 117 on clean 621ef4f; enclosed sky was 11 px / 7 blobs and its ceiling stayed untouched. Task 6's three in-scope rows were KILLED after correcting the initial ignored-test target omission. |
| 2026-09-18 | Full foreground gate GREEN (461 s). Three self-gate attempts were blocked before diff inspection by the known read-only `/tmp` mount-registry lock; no review finding was produced and the hard cap precluded a fourth. |
| 2026-09-18 | Task 3: added default Natural Bloom and measured Hdr-on bloom-off controls before judging bloom. Camp p90 floor was 5 Rec.601 levels (203–208); the bloom sample was 207, so it did not clear the floor and AC4 remains unmet. Task 4's three-effect switches were implemented and focused-test green; further sabotage and gate work stopped at AC4's explicit stop condition. |
| 2026-09-18 | Repaired the AC3 rendered guard after Hdr made terrace p10 non-discriminating (38 both with and without AO). Fresh Rec.601 terrace-mean samples set a 69.75 ceiling from a 0.049 AO-on floor and 70.016 AO-off control. The focused guard passed; the required MSAA-on RED failed at mean 70.756, then source was restored. |
| 2026-09-18 | Rebuilt the restored source as clean `a5ee674`, captured terrace mean 69.441 Rec.601, and re-measured enclosed sky at 0 px / 0 blobs without moving its ceiling. Added the Task 5 vehicle card; vehicle-only Task 5 remains unchecked and no FPS figure was claimed. |
| 2026-09-18 | Completed Task 6: all ten 11.1b mutations KILLED, including the rendered AO/MSAA rows; the restored-source audit found all 585 repository mutation anchors current. |
| 2026-09-18 | Full foreground gate GREEN in 480 s after the guard repair and completed mutation table. The story's three self-gate attempts were already exhausted by prior environmental failures, so no fourth review was attempted. |
| 2026-09-19 | **AC1 met.** `--static-world` never froze anything despite `README.md:201`; wired it to the `SetSpeed { Paused }` `command.rs` has sent since 10.5 (gui-only, AC10 intact). Camp spread collapsed to 0 median / 0 p90 / 0.0017 pp near-white, clearing AC1's bar by 100x-235x. Issue #105's diagnosis confirmed: the residual was walking, lantern-carrying dwarves. |
| 2026-09-19 | **AC4 amended and met.** Bloom's marginal contribution measured against `--fx-off bloom` with `Hdr` held on: halo median +13, mean +6.98 against floors of 0 and 0.035; bright tail -2 p90. The original bright-tail clause was unsatisfiable under `EnergyConserving`. Preset unchanged. |
| 2026-09-19 | Corrected a three-times-repeated attribution: the open-snow 117 -> 116 shift is **bloom's**, not `Hdr`'s. `Hdr` and `Bloom` arrived in one commit and were never separated until `--fx-off bloom` did it. |
| 2026-09-19 | **The `enclosed_sky` guard is INERT, not merely vacuous.** Its `const SKY = [5,12,28]` is an exact RGB match; `Hdr` moved the sky to `[7,15,31]`, so **zero** pixels now classify as sky (8,434 did pre-Hdr). It reports 0 holes / 0 blobs because it cannot see sky at all, and would pass with the terrain entirely absent. Raised for ruling; see issue #108. |
