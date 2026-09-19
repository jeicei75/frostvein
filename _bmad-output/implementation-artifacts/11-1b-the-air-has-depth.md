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
   `BLOWN_POOL_FRACTION_CEILING` are **not** raised and **not** asserted against a headless bloom
   frame; the ceiling clause is judged on the vehicle at the sitting (Wolf's ruling, 2026-09-18).
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
F11 is no longer the ambient-occlusion key                   KILLED
F12 is no longer the bloom key                               KILLED
the effect readout stops recording changed state             KILLED
```

### Completion Notes List

- Task 0: captured the build-specific no-AO/no-bloom controls. The camp flicker floor confirms Task 1 must pin the live flicker clock before bloom is measured.
- Task 1: implemented and mutation-proved the live `--lights-steady` path, but AC1 is blocked by residual camp-window variance after the clock is pinned. Tasks 2–6 were intentionally not started; Task 5 remains vehicle-only.
- Task 3: Bloom is installed but AC4 is blocked: its bright-tail p90 did not rise over the new build-specific Hdr/no-bloom floor. No headless ceiling was changed.
- Task 4: the three-effect command and seat controls are implemented and focused-test green; its story mutations remain deferred by the AC4 stop rule.
- Task 6: added and ran the remaining bloom, per-name `--fx-off`, F11/F12, and readout sabotages; all ten table rows KILLED. The full repository anchor audit passed after checking the legacy camera/readout rows.

### File List

- `_bmad-output/implementation-artifacts/11-1-signoff/task-0-f604b40-a.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-0-f604b40-b.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-0-f604b40-c.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-0-f604b40-d.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-1-7d13828-a.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-1-7d13828-b.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-1-7d13828-c.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-1-7d13828-d.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2-8edc62a-a.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2-8edc62a-b.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2-8edc62a-c.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2-8edc62a-d.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2-msaa-red-8edc62a.png` (new; deliberate dirty mutant evidence)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2-621ef4f-restored.png` (new; restored clean-build capture)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2-621ef4f-enclosed.png` (new; subdiv-2 enclosed-sky re-measurement)
- `_bmad-output/implementation-artifacts/mutations/11-1b-the-air-has-depth.sh` (new)
- `_bmad-output/implementation-artifacts/mutations/6-1-the-world-moves.sh` (updated; re-pointed after the flicker seam changed)
- `_bmad-output/implementation-artifacts/mutations/10-7-the-sun-lights-the-valley.sh` (updated; re-pointed after the projection resource initialization changed)
- `crates/gui/src/ingest.rs` (updated)
- `crates/gui/tests/pixel_guard.rs` (updated)
- `docs/tech-art-guidelines.md` (updated)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-3-hdr-no-bloom-67ba364-a.png` through `-d.png` (new)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-3-bloom-67ba364-a.png` (new)
- `_bmad-output/implementation-artifacts/11-1b-the-air-has-depth.md` (updated)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2b-5a1ed7b-ao-a.png` through `-d.png` (new; fresh mean-floor captures)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2b-5a1ed7b-ao-off.png` (new; fresh no-AO threshold control)
- `_bmad-output/implementation-artifacts/11-1-signoff/task-2b-a5ee674-restored.png` (new; clean post-RED restored capture)

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
