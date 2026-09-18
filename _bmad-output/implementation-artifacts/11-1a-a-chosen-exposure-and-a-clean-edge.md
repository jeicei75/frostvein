---
baseline_commit: 41b3f02613018857d5ad7b121d046b7209a1773c
---

# Story 11.1a: A Chosen Exposure and a Clean Edge

Status: in-progress

## Story

As the boss,
I want the client's camera to render at an exposure I chose and with edges that are not stair-stepped,
so that the frame I judge the art-shot effects against is a frame whose render path I control.

## Why this story exists, and why it is 11.1a

Epic 11's story 11.1 bundled occlusion, bloom and exposure. Three things found at creation split it
(Wolf ruled, 2026-09-18):

1. **SSAO cannot render at all while MSAA is on, and fails silently.** `extract_ssao_settings`
   (`bevy_pbr-0.19.0/src/ssao/mod.rs:479-494`) logs an `error!` and then **`return`s out of the whole
   loop** — not `continue` — so *every* camera is skipped. The frame renders, the capture writes, the
   process exits 0. Bevy's `Msaa` default is `Sample4` (`bevy_render-0.19.0/src/view/mod.rs:243-244`)
   and this client never sets `Msaa` anywhere, so AO added naively is inert.
2. **`Msaa::Off` and `Hdr` each move every pixel on their own**, before any effect is judged. The
   live-frame guards must be re-baselined against that change, not against it *plus* two effects.
3. **The epic's bloom AC was unsatisfiable headless** — see the ruling below.

So this story owns the render path and the measuring instrument; **11.1b** puts SSAO and Bloom on top
of it and judges them with the instrument this story builds and proves.

**Rulings inherited (Wolf, 2026-09-18, at creation):**

- **FXAA, not TAA.** TAA needs `MotionVectorPrepass` + jitter and accumulates across frames; falling
  snow, flickering light and a 160-frame capture settle make that unproven here. `Fxaa`
  (`bevy_anti_alias-0.19.0/src/fxaa/mod.rs:57`) needs no prepass and no `bevy` feature change. SMAA
  was rejected: it needs the `smaa_luts` feature, which is not in the root feature list, and crossing
  a `bevy` feature boundary rebuilds ~400 crates and has nearly OOMed this devpod.
- **Bloom's acceptance is a headless DELTA plus a vehicle ceiling** (carried to 11.1b, recorded here
  so it is not re-derived): a headless area figure must never be judged against
  `NEAR_WHITE_AREA_CEILING` (`deferred-work.md:1237-1246` — llvmpipe under-reads near-white by ~16 %,
  and the ceiling is calibrated on a GPU frame). Measured at creation, the margin is gone anyway:
  blown-pool headroom to its ceiling is **0.0154 pp against a 0.1568 pp same-build spread**.
- **No ceiling is raised to make a run pass.** 10.8's standing rule holds (AC8).

## Acceptance Criteria

1. The camera entity spawns carrying `Msaa::Off`. A test asserts it on the **live spawned entity**
   (through `configured_app`), against a hand-written literal — not read back from a constant.
2. The camera entity spawns carrying exactly one hardcoded `Exposure`. A test asserts its `ev100` on
   the live spawned entity against a hand-written literal. The story records the EV100 chosen and the
   reason, and `docs/tech-art-guidelines.md` § Lights carries it as a row.
3. `--fx-off <a,b,...>` parses the name `fxaa` and **reaches the spawned camera, not merely `Args`**:
   with `fxaa` in the list the camera entity has no `Fxaa` component; with the flag absent it has one.
   The flag does not require `--capture`. An unknown name is an error naming the accepted names.
4. `F10` toggles FXAA at the seat, and the on-screen readout names its state beside the F5–F9 lights.
   A test presses `F10` through the real key path and asserts the recorded readout string changes.
5. FXAA visibly softens the terrain silhouette: at the boot framing, a headless pair `--fx-off fxaa`
   against the default differs, inside a pinned window straddling the ridge-line silhouette, by more
   than that window's own same-build floor — measured as the count of pixels exactly equal to the sky
   colour `[5, 12, 28]`, which the hard-edged frame has more of. The figure and its floor are recorded.
6. `_bmad-output/implementation-artifacts/11-1-signoff/creases.py` is committed and reports, per pinned
   window, `p10`, `median`, `p90` and `mean` Rec.601 luminance. Its windows are pinned in the script and
   **none of them sits on the camp** (the camp window's mean moves 1.19 between same-build runs from
   light flicker; the three non-camp windows move 0).
7. The crease instrument is proved both ways before any figure taken with it is believed: run on two
   same-build captures it reports **delta 0 on `p10` and `median` for every pinned window**; run on a
   control against a deliberately darkened capture (`--lights-off ambient`) it reports a crease-window
   `p10` delta of at least 25 levels. Both runs are pasted into the Dev Agent Record.
8. The live-frame guards are re-baselined against the new render path and **no ceiling is raised merely
   to make a run pass**: `pixel_guard.rs`'s `ENCLOSED_SKY_CEILING` (2,300, residual 2,042) and
   `ALL_OFF_DROP_FLOOR` (40.0) are re-measured and their new figures recorded. If either must move, the
   story states the measured residual and the evidence that the count is not holes — the argument
   `pixel_guard.rs:466-476` makes — and Wolf rules before it moves.
9. `crates/gui/tests/capture.rs` is unchanged and still green. It measures **committed PNGs on disk**,
   not live renders, so the render-path change cannot reach it; if it goes red, something else did.
10. On the vehicle, `--perf-log` at the boot framing is read with FXAA on and off, and the two p50
    frame times are recorded. NFR6's 60 fps bar is re-read; if FXAA costs it, the figures are recorded
    and Wolf rules rather than the story tuning anything.
11. Wolf signs off the frame at the sitting: the edges read cleaner than the MSAA-4x control at the same
    framing, and the exposure is the one he wants to judge 11.1b's effects under. The pair is filed.
12. Every mutation row in `mutations/11-1a-a-chosen-exposure-and-a-clean-edge.sh` is shown to KILL, and
    the table's output is pasted into the Dev Agent Record.

**Deferred, by design, and named:** `Msaa::Off` exists in this story for a consumer that is not built
yet — `ScreenSpaceAmbientOcclusion` in **11.1b**. AC1 covers the decision surface (the component is on
the camera) because the seam has no live consumer here. 11.1b owes the AC that proves AO's output is
consumed rather than silently skipped.

## Tasks / Subtasks

- [x] **Task 0 — take the control, on a clean tree.** (AC: 5, 7, 8)
  - [x] Confirm `./target/debug/gui --version` names the current HEAD before capturing anything. On a
        merge commit the stamp names the merged parent; check `git rev-parse HEAD^{tree}` matches that
        commit's tree rather than assuming `-dirty` means stale. If the tree differs, `touch
        crates/gui/build.rs` and rebuild.
  - [x] Re-take at least four same-build captures and recompute the floors below on THIS build. The
        floor is build-specific — it moved 3.3x across one story before, and 20x across another. Do not
        inherit the creation figures; they are a control to compare against, not a floor to reuse.
- [x] **Task 1 — the render path.** (AC: 1, 2)
  - [x] Add `Msaa::Off` and one `Exposure` to the camera tuple at `ingest.rs:1284-1308`. Both are plain
        components beside the existing `AmbientLight` / `DistanceFog`; nothing else in the tuple moves.
  - [x] Pick the EV100 and say why in one line. `Exposure::default()` is `BLENDER` = **9.7**
        (`bevy_camera-0.19.0/src/camera.rs:263`, `impl Default` at `:279-283`); the other named
        constants are `EV100_SUNLIGHT` 15.0, `EV100_OVERCAST` 12.0, `EV100_INDOOR` 7.0 (`:255-257`).
        Setting the component explicitly at 9.7 changes no pixel and is a legitimate choice — it makes
        the value ours rather than Bevy's. Any other value must be justified by a frame.
  - [x] Add the row to `docs/tech-art-guidelines.md` § Lights (table rows `:56-67`). Read `:19-22`
        first: the tables are a view of the sections below, so rule the value into the section, then
        the table — do not fill the table in from the code.
- [ ] **Task 2 — FXAA, switchable.** (AC: 3, 4, 5)
  - [ ] Add `Fxaa` to the camera tuple unless switched off. `Fxaa` needs no prepass and no plugin add:
        `FxaaPlugin` ships inside `AntiAliasPlugin` (`bevy_anti_alias-0.19.0/src/lib.rs:28`), which
        `DefaultPlugins` already carries (`bevy_internal-0.19.0/src/default_plugins.rs:63`).
  - [ ] Add `--fx-off <a,b,...>`, copying `--lights-off` end to end (parse `ingest.rs:998`, name
        resolution `LightSource::from_name` `ingest.rs:129`, starting state `with_off` `ingest.rs:189`,
        `ALL` `ingest.rs:96`). Do **not** copy `--distance`'s `requires --capture` gate at
        `ingest.rs:1019`; this flag is interactive too, the way `--camera` is (`ingest.rs:1021-1023`
        records that deliberate non-gate).
  - [ ] Add the `F10` toggle and extend the readout beside `lighting_readout` (`ingest.rs:1344`,
        literals `:1348-1352`, driver `light_controls` `:1393`). Note the existing F-key order is NOT
        alphabetical — F7 is lanterns, F8 ambient, F9 torches (`ingest.rs:105-111`).
  - [ ] **Instrument test:** copy the shape of
        `the_distance_flag_reaches_the_camera_rig_rather_than_merely_parsing` (`ingest.rs:3313`). Its
        own docstring at `ingest.rs:3309` records why: replacing the assignment with `let _ = distance;`
        left all 106 tests green.
- [ ] **Task 3 — the crease instrument.** (AC: 6, 7)
  - [ ] Write `11-1-signoff/creases.py`. Stdlib only — import `load` from
        `10-7-signoff/lumstats.py` the way `10-5-signoff/window_diff.py:19-20` imports from
        `pixel_diff.py`. Use the **Rec.601** integer luma `(r*299 + g*587 + b*114)//1000`, the same
        statistic as `lumstats.py:38` and `pixel_guard.rs:35-41`, so figures compare against the record.
        Note `capture.rs:604` is Rec.709 — a different number; say which one the output is.
  - [ ] Pin the windows. Measured at creation on `41b3f02` (1280x720), same-build `a` vs `d`:

        | window | rect (x0,y0,x1,y1) | p10 | median | mean a → d |
        | --- | --- | --- | --- | --- |
        | `terrace-creases` | 860,190,1060,290 | 30 | 65 | 68.129 → 68.139 |
        | `open-snow-LL` | 180,620,380,700 | 68 | 117 | 104.052 → 104.079 |
        | `open-snow-LR` | 950,590,1150,670 | 68 | 117 | 107.785 → 107.785 |
        | `camp-terraces` | 500,400,700,500 | 67 | 68 | 92.586 → 91.398 |

        `p10` and `median` are **identical** across the pair on all four; the camp window's *mean*
        moves 1.19 and the others move ≤ 0.027. Keep `camp-terraces` only as a reported diagnostic and
        never as an assertion — AC6 forbids measuring in it.
  - [ ] Re-derive the rects against THIS build's control before pinning. They were chosen by eye off
        the creation frame; if the boot framing has moved, they name the wrong pixels.
- [ ] **Task 4 — re-baseline the live guards.** (AC: 8, 9)
  - [ ] `enclosed_sky` (`pixel_guard.rs:58-60`) matches the sky colour **exactly** (`== [5,12,28]`), so
        it is directly sensitive to this story's change: `Msaa::Off` hardens edges and should raise the
        exact-match count; FXAA blends them back down. Headroom is only 258 px (residual 2,042 against
        a 2,300 ceiling), so expect this guard to move and measure it before assuming a direction.
  - [ ] Re-read `ALL_OFF_DROP_FLOOR` (`pixel_guard.rs:425`). Its recorded basis is all-on 101.1 /
        all-off 13.2 against a 0.16 noise floor; exposure and FXAA both touch the all-on half.
  - [ ] Confirm `tests/capture.rs` is untouched and green (AC9).
- [ ] **Task 5 — the vehicle read.** (AC: 10, 11) — **cannot be done on a devpod.** No devpod can open
      a window; this task is a card for the gingerspice sitting. Write it into
      `11-1-signoff/task-5-vehicle-card.md` naming the two `--perf-log` runs and the frame pair Wolf
      judges, and hand it over rather than fabricating a figure.
- [ ] **Task 6 — sabotage.** (AC: 12)
  - [ ] Rows for: `Msaa::Off` dropped from the tuple; the `Exposure` value discarded; the `--fx-off`
        value discarded (`let _ = fx_off;`); the `F10` keycode moved; the readout not recording; the
        `Fxaa` component never inserted; and the crease instrument's window rects swapped.
  - [ ] `rg` the whole `mutations/` directory for rows quoting `ingest.rs` camera-tuple or
        `pixel_guard.rs` threshold literals and re-point any this story breaks —
        `scripts/audit-mutations.py` fails the gate on a stale row, and 10.9 and 10.10 both paid for it.
  - [ ] **Never `exec` a mutation payload to test whether it applies.** A payload reports whether it
        applies by applying it; `import pathlib` inside `exec` rebinds to the real module and defeats a
        monkeypatched `Path`. That sabotaged five tracked source files in one 5 ms burst mid-review on
        10.10, with three sibling layers compiling against the tree.

## Dev Notes

### Scope guardrails — do NOT

- **Do NOT add `ScreenSpaceAmbientOcclusion` or `Bloom`.** They are 11.1b. Adding `Hdr` (which `Bloom`
  requires) moves every pixel and would contaminate this story's re-baseline.
- Do NOT add a dependency or change a `bevy` feature. The stack is closed at `bevy` 0.19.0, and
  `3d_bevy_render` already carries everything this story and 11.1b need (verified below).
- Do NOT touch `protocol`, `sim-core`, `client-core` or `simd`. This is a client render-path change;
  AD-16 closes the M2 wire diff.
- Do NOT change a light, a colour, a `BOOT_*` constant or the composition. `bench_contract.rs:128-162`
  greps `camera.rs` for those literals and requires `scripts/bench/resolution_bench.py` to move in
  lockstep.
- Do NOT raise `NEAR_WHITE_AREA_CEILING` or `BLOWN_POOL_FRACTION_CEILING` (AC8).
- Do NOT build a general effect registry. `--fx-off` copies `--lights-off`'s shape and carries one
  name today; 11.1b and 11.2 add theirs.

### What already exists

- Camera spawn, `ingest.rs:1284-1308`: `Camera3d`, `Projection::Perspective`, `rig.transform()`,
  `CameraRig`, `AmbientLight`, `DistanceFog`, `ClientLocal`. No `Camera` component is inserted
  explicitly — it arrives through `Camera3d`'s required components.
- `--lights-off` is the whole pattern to copy for `--fx-off`, including the F-key toggle, the readout
  and the `with_off` starting state.
- Test harness: `configured_app` (`ingest.rs:1813`), `live_app` (`tests/headless.rs:2324`),
  `press_once` (`tests/headless.rs:78` — it releases AND clears, because `MinimalPlugins` has no
  `InputPlugin`).
- `pixel_guard.rs`'s `Daemon::capture(label, extra)` (`:179`) is exactly the on/off harness this story
  needs; `switching_every_light_off_darkens_the_frame_and_leaves_no_emitter_glowing` (`:399`) is the
  on/off comparison shape, floor and all.

### Key decisions & traps

- **The premises were verified at creation, against the pinned 0.19.0 source. They hold:**
  `3d_bevy_render` expands to include `bevy_anti_alias`, `bevy_pbr` and `bevy_post_process`
  (`bevy-0.19.0/Cargo.toml` `[features]`), so **no feature change is needed** for FXAA now or for
  SSAO/Bloom in 11.1b. Both plugins are auto-registered — SSAO through `PbrPlugin`
  (`bevy_pbr-0.19.0/src/lib.rs:227`), Bloom through `PostProcessPlugin`
  (`bevy_post_process-0.19.0/src/lib.rs:33`), and both of those are in `DefaultPlugins`
  (`bevy_internal-0.19.0/src/default_plugins.rs:61, 79`) — so a component alone is enough and no
  `add_plugins` is owed. `Bloom` carries `#[require(Hdr)]` and `ScreenSpaceAmbientOcclusion` carries
  `#[require(DepthPrepass, NormalPrepass)]`, so those prerequisites are automatic. **`Msaa::Off` is the
  only one that is not**, and it is the silent one.
- **Lavapipe supports SSAO here — this was Epic 11's stated unknown and it is now answered.** The
  plugin's only GPU gate is `max_storage_textures_per_shader_stage >= 5`
  (`bevy_pbr-0.19.0/src/ssao/mod.rs:61-70`, a `warn!` then `return`). Probed on this devpod:
  `llvmpipe (LLVM 19.1.7, 256 bits)`, Vulkan, Mesa 25.0.7 — **48**. The plugin will load. That proves
  the plugin loads, not that the effect looks right; it removes the "cannot render headless at all"
  branch for 11.1b.
- **Exit 0 is not a result, and neither is a rendered frame.** The whole reason this story exists is
  that a silently-skipped effect still produces a clean capture and exit 0.
- **The PNG is written before the range check asserts** (`capture.rs:1276` `save_then_validate`), so a
  capture that exits 101 still leaves a measurable frame. The creation controls were measured this way.
- **`--frames 2` never captures** (the frame is still black and the run dies on `capture is black`,
  `capture.rs:1419`) and **`--cursor` is dead headless** (no `PrimaryWindow`). Captures need
  `--frames 160`.
- **Two luminance definitions live in this repo.** `capture.rs:604` is Rec.709; `lumstats.py:38` and
  `pixel_guard.rs:35-41` are Rec.601 integer. Every figure in this story is Rec.601. Say which one any
  new number is in.
- **A two-sample pair is not a floor.** The documented 0.0048 floor was one lucky-tight pair and is
  retracted (issue #98). Four samples at creation gave a mean-luminance spread of 0.252 — 3.5x the
  0.0724 recorded one build earlier.
- **A mutant binary outlives a source restore.** `mutate.sh` restores the source and leaves the last
  mutant *build* on disk. Rebuild and re-check `--version` before taking any capture after a mutation
  run.

### Creation control — build `41b3f02`, boot framing, `--headless --static-world --subdiv 4 --frames 160`

Four same-build captures, all exit 0:

| run | warm-lit | ground-median | near-white % | blown-pool % | p99 | mean (Rec.601) |
| --- | --- | --- | --- | --- | --- | --- |
| a | 27,958 | 81 | 0.7656 | 0.6084 | 188.3 | 71.337 |
| b | 28,562 | 81 | 0.5789 | 0.4516 | 178.3 | 71.144 |
| c | 28,746 | 81 | 0.7601 | 0.5312 | 188.4 | 71.376 |
| d | 28,760 | 81 | 0.8177 | 0.5768 | 191.3 | 71.396 |
| **spread** | **802** | **0** | **0.2388 pp** | **0.1568 pp** | **13.0** | **0.252** |

`ground-median-luminance` is the one whole-frame band that does not move at all. **The two area
statistics have less headroom to their ceilings than their own noise**: near-white 0.1284 pp of
headroom against a 0.2388 pp spread, blown-pool 0.0154 pp against 0.1568 pp. This is issue #90
confirmed at four samples, and it is why 11.1b's bloom AC is a delta rather than a ceiling.

### Project Structure

| File | Change |
| --- | --- |
| `crates/gui/src/ingest.rs` | UPDATE — `Msaa::Off` + `Exposure` + `Fxaa` in the camera tuple; `--fx-off` parse/resource/camera; `F10`; readout |
| `crates/gui/tests/headless.rs` | UPDATE — live-entity assertions for AC1–AC4 |
| `crates/gui/tests/pixel_guard.rs` | UPDATE — AC5's silhouette pair; re-baselined thresholds (AC8) |
| `crates/gui/tests/capture.rs` | UNCHANGED — AC9 asserts this |
| `docs/tech-art-guidelines.md` | UPDATE — § Lights gains the exposure row |
| `_bmad-output/implementation-artifacts/11-1-signoff/creases.py` | NEW |
| `_bmad-output/implementation-artifacts/11-1-signoff/task-5-vehicle-card.md` | NEW |
| `_bmad-output/implementation-artifacts/mutations/11-1a-a-chosen-exposure-and-a-clean-edge.sh` | NEW |

### Verification

Run from the repo root. **`scripts/gate.sh` has no thread knob and its Bevy test apps exhaust 23 GB —
export `RUST_TEST_THREADS=6` and run it in the FOREGROUND** (a background job gets killed on memory
pressure). The pre-push hook is only the FAST tier, so a successful push is not full-gate evidence.

```bash
RUST_TEST_THREADS=6 scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/11-1a-a-chosen-exposure-and-a-clean-edge.sh
```

**The control, which RAN at creation and must be re-run here** (figures above):

```bash
./target/debug/gui --version          # must name the current HEAD's tree
./target/debug/simd 7466 &
./target/debug/gui 7466 --headless --static-world --subdiv 4 --frames 160 \
  --capture _bmad-output/implementation-artifacts/11-1-signoff/control-<sha>-a.png
# creation observation on 41b3f02, exit 0:
#   capture range check: warm-lit pixels=27958 ground-median-luminance=81
#     near-white-area=0.7656% blown-pool=0.6084% p99-luminance=188.3 resolution=1280x720
```

**The crease instrument's two proofs, both RAN at creation on the prototype** (AC7). The GREEN:

```bash
python3 _bmad-output/implementation-artifacts/11-1-signoff/creases.py \
  control-<sha>-a.png=control-a control-<sha>-d.png=control-d
# creation observation: p10 and median IDENTICAL across the pair on all four windows;
#   terrace-creases p10=30 median=65, open-snow-LL/LR p10=68 median=117.
```

The RED — a deliberately darkened frame, with its restore step (there is none: it is a flag, not an
edit, which is why this is the RED to prefer):

```bash
./target/debug/gui 7466 --headless --static-world --subdiv 4 --frames 160 \
  --lights-off ambient --capture /tmp/amboff.png        # exits 101 on GROUND_LUMINANCE_FLOOR; PNG is written first
python3 _bmad-output/implementation-artifacts/11-1-signoff/creases.py \
  control-<sha>-a.png=control /tmp/amboff.png=ambient-off
# creation observation: terrace-creases p10 30 -> 0, median 65 -> 1; open-snow-LL median 117 -> 104.
# EXPECTED FAILURE if the instrument is broken: the two runs report the same numbers.
```

**The `--fx-off` RED**, which cannot run until the flag exists. The required non-zero observation, to
be produced by dev: replace the parsed value with `let _ = fx_off;` and confirm the reaches-the-camera
test of AC3 goes **RED**. A test that only checks the flag parses does not satisfy AC3 — that is the
exact hole `--distance` had when `let _ = distance;` left all 106 tests green (`ingest.rs:3309`).

### References

- `_bmad-output/planning-artifacts/epics.md` § Epic 11 / Story 11.1 — story source; amended at creation
  to record this split and the three rulings
- Pinned Bevy 0.19.0 source, all verified at creation: `bevy_pbr/src/ssao/mod.rs:61-70, 113, 479-494` ·
  `bevy_render/src/view/mod.rs:240-246` · `bevy_anti_alias/src/fxaa/mod.rs:57` ·
  `bevy_anti_alias/src/taa/mod.rs:113, 152-154` · `bevy_camera/src/camera.rs:232-263` ·
  `bevy_post_process/src/bloom/settings.rs:32` · `bevy/Cargo.toml` `[features] 3d_bevy_render`
- `_bmad-output/implementation-artifacts/deferred-work.md:1237-1246` — the llvmpipe near-white venue rule
- `crates/gui/src/capture.rs:542-602` — every capture constant and its calibration provenance
- `docs/technical-preferences.md:47-62` anti-overengineering · `:72-91` the instrument rule
- NFR6 — 60 fps at working zoom, ≥30 at full vista, read on the vehicle with `--perf-log`
- Open issues this story touches: **#90** (near-white swing — confirmed at four samples here), **#75**
  (the lighting re-judgement; its "capture exits 101 on main" premise is stale — the boot framing exits
  0 today), **#72** (pixel-guard PNG race), **#84** (`--frames` ignored without `--capture`), **#62**
  (pin the tech-art doc values against the code — AC2 adds a row it will have to cover)

### Previous-story intelligence

- 10.10 shipped the camera rig this story's captures are framed by; `--camera` and `--distance` now
  `bail!` together, so a pasted readout line must not be combined with `--distance`.
- 10.10's review found **six** seams that were present but observable by nothing — a readout key, a
  pan branch, a live-rig formatter. Every one was green. AC3 and AC4 exist in that shape on purpose:
  assert through the real key path and the real spawned entity, never the formatter alone.
- Branch off `main` at `41b3f02`, which is where PRs #100 and #103 merged and nothing is in flight.
  Suggested slug `story-11-1a-exposure-and-clean-edge`. Commit as `Völundr <jeicei75@gmail.com>` on the
  **first** commit — this clone's git config defaults to `jeicei75`, so pass `--author` every time. No
  `Co-Authored-By: Claude` trailer and no "Generated with Claude" footer (CLAUDE.md §8).

## Dev Agent Record

### Agent Model Used

gpt-5.6-terra, reasoning effort high

### Debug Log References

- Task 0: `cargo build -p gui --offline`; four foreground `--headless --static-world --subdiv 4 --frames 160` captures; Rec.601 statistics are recorded in `11-1-signoff/task-0-control.md`.

### Completion Notes List

- Task 0 complete: rebuilt after checking the merge-tree stamp, then re-took four same-build controls. The control spreads are recorded; no prior-build floor was reused.
- Task 1 complete: the live camera explicitly uses `Msaa::Off` and 9.7 EV100. 9.7 is Bevy's Blender-calibrated default, selected to preserve the approved frame while making the art decision explicit.

### File List

- `_bmad-output/implementation-artifacts/11-1a-a-chosen-exposure-and-a-clean-edge.md` — Task 0 bookkeeping.
- `_bmad-output/implementation-artifacts/11-1-signoff/task-0-control.md` — Task 0 re-take figures.
- `_bmad-output/implementation-artifacts/11-1-signoff/control-41b3f02-a.png` — Task 0 control capture.
- `_bmad-output/implementation-artifacts/11-1-signoff/control-41b3f02-b.png` — Task 0 control capture.
- `_bmad-output/implementation-artifacts/11-1-signoff/control-41b3f02-c.png` — Task 0 control capture.
- `_bmad-output/implementation-artifacts/11-1-signoff/control-41b3f02-d.png` — Task 0 control capture.
- `_bmad-output/implementation-artifacts/mutations/11-1a-a-chosen-exposure-and-a-clean-edge.sh` — Task 1 mutation rows.
- `crates/gui/src/ingest.rs` — explicit camera MSAA/exposure and live-rig tests.
- `docs/tech-art-guidelines.md` — exposure ruling and Lights table row.

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-18 | Created. Epic 11's 11.1 split into 11.1a (render path + instrument) and 11.1b (SSAO + Bloom) on Wolf's ruling, after creation-time verification found the SSAO/MSAA silent skip, answered the lavapipe question (48 storage textures, ≥ 5), and measured the area ceilings' headroom to be smaller than their own noise. FXAA ruled over TAA and SMAA. |
| 2026-09-18 | Re-took Task 0's four same-build controls and recorded the build-specific floors. |
| 2026-09-18 | Made the camera's MSAA and exposure explicit, with live-entity tests and the tech-art ruling. |
