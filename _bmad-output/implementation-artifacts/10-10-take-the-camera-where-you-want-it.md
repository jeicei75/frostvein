---
baseline_commit: c5f96caa470e457c874001d9df2ccc58355efe94
---

# Story 10.10: Take the Camera Where You Want It

Status: in-progress

## Story

As the boss,
I want to fly the camera where I like, return to a view I have already judged, and frame a single dwarf,
so that comparing a look change costs one command instead of a hand-flown approximation.

## Acceptance Criteria

1. With no camera input and no `--camera`, the boot rig's yaw, pitch, distance and focus and the `Transform` it produces equal hand-written literals in the test — NOT values read back from the `BOOT_*` constants, so the test fails when a constant moves (the deliberate pattern at `ingest.rs:3313`). The boot capture's mean luminance stays within 10x the 0.0048 control swing in `10-10-signoff/task-0-control.md`.
2. `CameraRig.focus` is a world-space `Vec3` that pan moves, clamped so the focus cannot leave the world bounds; orbit and zoom clamps are unchanged.
3. MMB-drag orbits, shift+MMB-drag pans, the mouse wheel zooms, and holding shift multiplies the rate. RMB and LMB behaviour is unchanged.
4. Orbit, pan and zoom rates are per-second: stepping the systems at two different frame deltas that total the same elapsed time leaves yaw and pitch within 1e-4 rad and distance within 1e-3 of each other.
5. `--camera <yaw>,<pitch>,<distance>,<fx>,<fy>,<fz>` sets the rig at startup and reaches the spawned `CameraRig`, not merely `Args`. It does not require `--capture`.
6. The readout key prints one line carrying yaw, pitch, distance and focus; passing that line's values back as `--camera` reproduces the same rig by exact float comparison.
7. The printed line differs when the rig differs: two rigs that are not equal never print the same line.
8. LMB with `DesignateMode::None` selects the nearest dwarf whose projected screen position lies within the pick radius of the cursor, and frames him: he projects within 0.05 of screen centre and the focus tracks him while selected.
9. Escape releases the selection; LMB on empty ground with no dwarf in radius selects nothing and leaves the rig untouched.
10. Selection and camera state are client-local: no wire message is sent, and `protocol` and `client-core` are unchanged.
11. A non-boot `--camera` capture that trips `NEAR_WHITE_AREA_CEILING` reports which framing it was taken at. The ceiling value is not raised.
12. Every mutation row in `mutations/10-10-take-the-camera-where-you-want-it.sh` is shown to KILL, and the table's output is pasted into the Dev Agent Record.

## Tasks / Subtasks

- [ ] **Task 0 — re-take the control.** (AC: 1)
  - [x] Confirm `./target/debug/gui --version` prints the CURRENT clean HEAD before capturing anything. If it does not, `touch crates/gui/build.rs` and rebuild — see the stamp trap in Dev Notes.
  - [ ] Re-run the control pair from `10-10-signoff/task-0-control.md` on the story branch and confirm mean luminance still lands near 76.12 and the pair swing near 0.0048. A moved baseline is a finding, not a nuisance.
- [x] **Task 1 — make the focus movable.** (AC: 2)
  - [x] Change `CameraRig.focus` from `[i32; 3]` to `Vec3` (world space). Add `world_to_render_f32(Vec3) -> Vec3` in `crates/gui/src/transform.rs` and make the existing `world_to_render([i32; 3])` delegate to it, so exactly one transform pair survives.
  - [x] Add `CameraRig::pan(&mut self, right: f32, forward: f32)`, moving the focus in the camera's own ground plane and clamping each axis to the world bounds.
  - [x] Update `north_on_screen` (`camera.rs:123`) and `CameraRig::new` for the new focus type.
- [ ] **Task 2 — seat controls.** (AC: 3, 4)
  - [ ] Extend `camera_controls` (`ingest.rs:1432`) to read `ButtonInput<MouseButton>`, `MouseMotion` and `MouseWheel`; multiply every rate by `Res<Time>::delta_secs()` and by the shift multiplier.
  - [ ] Strike the "remains unclaimed until UX-DR2 lands" comment at `ingest.rs:1446`; the wheel is claimed here.
  - [ ] `configured_app` does NOT insert `ButtonInput<MouseButton>` (`ingest.rs:1845-1866`). Insert it there, or the new tests panic on a missing resource.
- [ ] **Task 3 — the instrument: readout and `--camera`.** (AC: 5, 6, 7, 11)
  - [ ] Parse `--camera` beside `--distance` (`ingest.rs:935`). Do NOT copy `--distance`'s `requires --capture` gate at `ingest.rs:980`; this flag is interactive too.
  - [ ] Carry it to the rig in `setup_camera` (`ingest.rs:1179`) the way `CaptureDistance` is carried (`ingest.rs:1147`).
  - [ ] Add the readout key printing one line, and make the capture's ceiling failure name the framing it was taken at.
  - [ ] **Instrument test:** the printed line must CHANGE when the rig changes (AC7), and `--camera` must reach the rig rather than parse — copy the shape of `the_distance_flag_reaches_the_camera_rig_rather_than_merely_parsing` (`ingest.rs:3313`).
- [ ] **Task 4 — select and frame a dwarf.** (AC: 8, 9, 10)
  - [ ] Project each mirrored dwarf with `project_world_point_with_depth` and take the nearest within the pick radius. Read positions from the `client-core` mirror, never from wire messages.
  - [ ] Solve the framing so the dwarf lands centred: `composition_target()` still adds the boot push scaled by `distance/90`, so aiming the focus AT the dwarf leaves him off-centre.
  - [ ] Follow the AD-15 blended position; never extrapolate, and snap rather than animate across a `snapshot`.
- [ ] **Task 5 — sabotage.** (AC: 12)
  - [ ] Write `mutations/10-10-take-the-camera-where-you-want-it.sh` with rows for: delta-time scaling removed, the pan clamp removed, the `--camera` value discarded (`let _ = camera;`), and the boot-rig guard.
  - [ ] `rg` the whole `mutations/` directory for rows quoting `camera.rs`/`setup_camera`/`camera_controls` literals and re-point any this story breaks — `scripts/audit-mutations.py` fails the gate on a stale row.

## Dev Notes

### Scope guardrails — do NOT

- Do NOT add rebindable keys, a viewpoint registry, a camera-path recorder, or a `CameraMode` enum with one real mode. The printed line is the save format.
- Do NOT change `protocol` or `client-core`. AD-16 closes the M2 wire diff; camera and selection send nothing.
- Do NOT change the look, the light, or any `BOOT_*` constant. `bench_contract.rs:128-162` greps `camera.rs` for those literals and requires `scripts/bench/resolution_bench.py` to move in lockstep.
- Do NOT raise `NEAR_WHITE_AREA_CEILING`. 10.8's rule: measure, do not raise.
- Do NOT add a dependency. The M2 stack is closed at `bevy` 0.19.0.

### What already exists

- `CameraRig` with `orbit`/`zoom`/`transform`/`project_*` (`camera.rs:32-111`); `focus` is written once at `CameraRig::new([64, 64, 9])` (`ingest.rs:1179`) and never again.
- `camera_controls` (`ingest.rs:1432`): WASD/QE, `0.02` rad and `1.0` distance **per frame**, no delta-time.
- `--distance`: parse `ingest.rs:935`, gate `ingest.rs:980`, resource `ingest.rs:1147`, rig `ingest.rs:1179`. Its two tests are the pattern to copy.
- Test harness: `configured_app` (`ingest.rs:1813`), `live_app` (`tests/headless.rs:2324`), `press_once` (`tests/headless.rs:78`), and `install_pick_camera` (`tests/headless.rs:2357`) which fakes a viewport so `viewport_to_world` resolves. `mouse_drag_uses_the_anchor_level_and_clears_its_anchor_on_release` (`tests/headless.rs:2745`) is the click-and-assert shape.
- `camera_controls_drive_the_rig` (`tests/headless.rs:2976`) already asserts a key moves `rig.distance`; extend it.

### Key decisions & traps

- **The boot composition rides a moved focus.** `composition_target()` (`camera.rs:66`) adds `boot_composition_offset() * (distance/90).min(1.0)` to the focus. Framing a dwarf by pointing the focus at him leaves him off-centre; solve for the offset.
- **`--cursor` is dead headless** — no `PrimaryWindow`, so the live pick is always `None`. AC8/AC9 must be pinned by `live_app` + `install_pick_camera`, never by a headless capture.
- **`--frames 2` never captures** — the frame is still black and the run dies on `capture is black` (`capture.rs:1419`). Captures need `--frames 160`.
- **The changed-pixel count cannot guard this framing.** Same-build noise is 55,284 px (6.0 %). Mean luminance swings 0.0048 and moved 766x that under the RED. Use mean luminance; see `10-10-signoff/task-0-control.md`.
- **The build stamp can go stale.** On a clean tree at `5452c4d`, after a rebuild that recompiled `gui`, `--version` still said `bd5a9df-dirty`; `touch crates/gui/build.rs` fixed it. Check the stamp before trusting any frame.
- **`camera_controls` and `update_fog_from_camera` are unordered** (`ingest.rs:1487`), so a test on the pair needs two `app.update()`s.
- **`press_once` releases AND clears** (`tests/headless.rs:78`) — `MinimalPlugins` has no `InputPlugin`, so a pressed key otherwise stays just-pressed forever.

### Project Structure

| File | Change |
| --- | --- |
| `crates/gui/src/camera.rs` | UPDATE — `focus: Vec3`, `pan()`, `north_on_screen`, composition |
| `crates/gui/src/transform.rs` | UPDATE — add `world_to_render_f32`, delegate the integer form |
| `crates/gui/src/ingest.rs` | UPDATE — `camera_controls`, `--camera` parse/resource/rig, readout key, `configured_app` mouse resource |
| `crates/gui/src/pick.rs` | UPDATE — dwarf selection beside the existing tile pick |
| `crates/gui/src/capture.rs` | UPDATE — ceiling failure names its framing |
| `crates/gui/tests/headless.rs` | UPDATE — control, instrument, select-and-frame tests |
| `_bmad-output/implementation-artifacts/mutations/10-10-take-the-camera-where-you-want-it.sh` | NEW |

### Verification

Run from the repo root. **`scripts/gate.sh` has no thread knob and its Bevy test apps exhaust 23 GB — export `RUST_TEST_THREADS=6` and run it in the FOREGROUND.**

```bash
RUST_TEST_THREADS=6 scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/10-10-take-the-camera-where-you-want-it.sh
```

Instrument recipe — the capture half, which RAN at creation and must be re-run here:

```bash
./target/debug/gui --version          # must print the current clean HEAD, no -dirty
./target/debug/simd 7451 &
./target/debug/gui 7451 --headless --static-world --subdiv 4 --frames 160 \
  --capture 10-10-signoff/boot-<sha>.png
# observed at creation on 5452c4d, exit 0:
#   capture range check: warm-lit pixels=27852 ground-median-luminance=81
#     near-white-area=0.6597% blown-pool=0.5044% p99-luminance=181.7 resolution=1280x720
```

The `--camera` half cannot run until the flag exists. The required non-zero observation, to be produced by dev:

```bash
./target/debug/gui 7451 --headless --static-world --subdiv 4 --frames 160 \
  --camera 0.7,0.45,90,64,64,9 --capture 10-10-signoff/camera-boot-<sha>.png
# REQUIRED: mean luminance within 10x of 0.0048 of the boot control — the flag at boot
#           values must reproduce the boot frame, not merely be accepted.
```

**The deliberate RED, with its restore step.** Whatever the readout prints, break it before trusting it:

```bash
# RED: in setup_camera, discard the parsed value -- `let _ = camera;`
# EXPECTED: `--camera 0.7,0.45,20,64,64,9` produces a frame whose mean luminance matches the
#           BOOT control instead of the near-framing, and the reaches-the-rig test goes red.
# RESTORE: revert the line, rebuild, and confirm --version shows no -dirty before re-capturing.
```
Precedent for why this RED is mandatory: `--distance`'s own docstring (`ingest.rs:3309`) records that replacing its assignment with `let _ = distance;` left all 106 tests green.

### References

- `_bmad-output/planning-artifacts/epics.md` § Story 10.10 — story source, creation measurements
- `_bmad-output/implementation-artifacts/10-10-signoff/task-0-control.md` — control pair, noise floor, RED
- M2 architecture spine — AD-13 (mirror in `client-core`), AD-14 (camera rigs are client-local), AD-15 (blend, never extrapolate; a snapshot snaps), AD-16 (M2 wire diff closed), AD-17 rung 2 (gui camera logic runs headless under minimal plugins), and the one-transform-pair rule
- `docs/technical-preferences.md:47-62` — anti-overengineering policy; `:72-91` — instrument rule
- UX-DR1 (no angle you get stuck in), UX-DR2 (the wheel is the camera's), `epics.md:989` (7.1 resolved the collision)
- NFR6 — 60 fps at working zoom, ≥30 fps at full vista, read on the vehicle with `--perf-log`
- Open issues touching this story's instruments: #90 (near-white swing), #84 (`--frames` ignored without `--capture`), #72 (pixel-guard PNG race)

### Previous-story intelligence

- 10.9 had to re-point a row in an OLDER mutation table because a renamed test broke it; `scripts/audit-mutations.py` fails the gate on a stale row. Budget the re-point (Task 5).
- 10.9 proved `--cursor` cannot work headless and `--frames 2` never captures — both already folded into the traps above, at the cost of a wasted Task 1 there.
- Branch `story-10-10-camera-control` is one commit past the `main` merge of #95; 10.9's PR #91 is merged. Commit as `Völundr <jeicei75@gmail.com>`; no `Co-Authored-By` trailer (CLAUDE.md §8).

## Dev Agent Record

### Agent Model Used

GPT-5.6-Codex

### Debug Log References

- Task 0 build stamp: `cargo build --offline -p gui -p simd -j 8`; `./target/debug/gui --version` printed `gui build 8eb1d1d`, matching clean `HEAD 8eb1d1d`.
- Task 0 control re-take: `control-boot-8eb1d1d-a.png` mean luminance 71.1261; `...-b.png` 71.1985; swing 0.0724. This does not meet the 76.12 / 0.0048 control. Filed live bug #98 with the commands and range measurements.
- RED — `pan_moves_focus_on_the_camera_ground_plane_and_stays_inside_the_world`: before the implementation, `cargo test --offline -p gui pan_moves_focus_on_the_camera_ground_plane_and_stays_inside_the_world -j 8` failed with `no method named pan` and `[i32; 3] == Vec3` type errors.
- RED — `boot_rig_and_transform_are_pinned_by_literals`: initial literal-pin test failed and printed `Transform { translation: Vec3(100.74321, 47.646896, -33.05163), rotation: Quat(-0.202291, 0.41114035, 0.094099894, 0.8838479), scale: Vec3(1.0, 1.0, 1.0) }`; the final test compares those hand-written literals and passed.

### Completion Notes List

- Task 1: `CameraRig.focus` is now a world-space `Vec3`; panning follows camera right/forward on the ground plane and clamps to the fixed 128×128×32 world bounds. Integer and fractional world-to-render conversion now share `world_to_render_f32`.
- Task 0: stamp was verified, but the manual control pair moved. The task remains open pending a ruling/fix for #98; the baseline was not changed.

### File List

- `crates/gui/src/camera.rs` — world-space focus, pan, boot literal guard.
- `crates/gui/src/transform.rs` — fractional world-to-render conversion.
- `crates/gui/src/pick.rs` — test rig updated for `Vec3` focus.
- `crates/gui/tests/headless.rs` — test rig literals updated for `Vec3` focus.
- `_bmad-output/implementation-artifacts/10-10-signoff/control-boot-8eb1d1d-a.png` — Task 0 control evidence.
- `_bmad-output/implementation-artifacts/10-10-signoff/control-boot-8eb1d1d-b.png` — Task 0 control evidence.
- `_bmad-output/implementation-artifacts/10-10-take-the-camera-where-you-want-it.md` — Task 0/1 evidence and status.

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-17 | Created. Control pair, same-build noise floor and the `--distance 80` RED measured at creation on `5452c4d`; AC1 rewritten against mean luminance after the changed-pixel statistic was shown to have a 6 % floor. |
| 2026-09-17 | Task 1: made camera focus movable in world space with a literal boot-rig guard; re-took Task 0 control and filed #98 for the moved baseline. |
