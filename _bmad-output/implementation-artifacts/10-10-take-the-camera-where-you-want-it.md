---
baseline_commit: c5f96caa470e457c874001d9df2ccc58355efe94
---

# Story 10.10: Take the Camera Where You Want It

Status: review

## Story

As the boss,
I want to fly the camera where I like, return to a view I have already judged, and frame a single dwarf,
so that comparing a look change costs one command instead of a hand-flown approximation.

## Acceptance Criteria

1. With no camera input and no `--camera`, the boot rig's yaw, pitch, distance and focus and the `Transform` it produces equal hand-written literals in the test — NOT values read back from the `BOOT_*` constants, so the test fails when a constant moves (the deliberate pattern at `ingest.rs:3313`). **The luminance clause is STRUCK (Wolf's ruling, 2026-09-17, issue #98): the hand-written-literal rig guard is the whole of AC1, and there is no capture-based guard on the boot frame.**
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

- [x] **Task 0 — re-take the control.** (AC: 1) — **done; its outcome was a finding, and the ruling that struck AC1's luminance clause. See issue #98.**
  - [x] Confirm `./target/debug/gui --version` prints the CURRENT clean HEAD before capturing anything. If it does not, `touch crates/gui/build.rs` and rebuild — see the stamp trap in Dev Notes.
  - [x] Re-run the control pair. **The baseline did NOT move.** The documented 76.12 is a plain RGB channel average mislabelled "mean luminance" — it reproduces to 4 d.p. as the RGB mean of the committed creation frame, whose Rec.601 luminance (`10-7-signoff/lumstats.py`, the statistic `pixel_guard.rs` uses) is 71.19. The reported "5 points lower" was that statistic gap, not a look change; creation→now is 0.0670 (Rec.601) / 0.0589 (RGB), i.e. noise-sized. **What did NOT hold is the noise floor: this build's same-build pair swing is 0.0724 against the documented 0.0048 — 20x, and larger than AC1's own 0.048 tolerance, so the clause was unsatisfiable by noise alone.** Settled off the four committed control PNGs; no recapture needed.
- [x] **Task 1 — make the focus movable.** (AC: 2)
  - [x] Change `CameraRig.focus` from `[i32; 3]` to `Vec3` (world space). Add `world_to_render_f32(Vec3) -> Vec3` in `crates/gui/src/transform.rs` and make the existing `world_to_render([i32; 3])` delegate to it, so exactly one transform pair survives.
  - [x] Add `CameraRig::pan(&mut self, right: f32, forward: f32)`, moving the focus in the camera's own ground plane and clamping each axis to the world bounds.
  - [x] Update `north_on_screen` (`camera.rs:123`) and `CameraRig::new` for the new focus type.
- [x] **Task 2 — seat controls.** (AC: 3, 4)
  - [x] Extend `camera_controls` (`ingest.rs:1432`) to read `ButtonInput<MouseButton>`, `MouseMotion` and `MouseWheel`; multiply every rate by `Res<Time>::delta_secs()` and by the shift multiplier.
  - [x] Strike the "remains unclaimed until UX-DR2 lands" comment at `ingest.rs:1446`; the wheel is claimed here.
  - [x] `configured_app` does NOT insert `ButtonInput<MouseButton>` (`ingest.rs:1845-1866`). Insert it there, or the new tests panic on a missing resource.
- [x] **Task 3 — the instrument: readout and `--camera`.** (AC: 5, 6, 7, 11)
  - [x] Parse `--camera` beside `--distance` (`ingest.rs:935`). Do NOT copy `--distance`'s `requires --capture` gate at `ingest.rs:980`; this flag is interactive too.
  - [x] Carry it to the rig in `setup_camera` (`ingest.rs:1179`) the way `CaptureDistance` is carried (`ingest.rs:1147`).
  - [x] Add the readout key printing one line, and make the capture's ceiling failure name the framing it was taken at.
  - [x] **Instrument test:** the printed line must CHANGE when the rig changes (AC7), and `--camera` must reach the rig rather than parse — copy the shape of `the_distance_flag_reaches_the_camera_rig_rather_than_merely_parsing` (`ingest.rs:3313`).
- [x] **Task 4 — select and frame a dwarf.** (AC: 8, 9, 10)
  - [x] Project each mirrored dwarf with `project_world_point_with_depth` and take the nearest within the pick radius. Read positions from the `client-core` mirror, never from wire messages.
  - [x] Solve the framing so the dwarf lands centred: `composition_target()` still adds the boot push scaled by `distance/90`, so aiming the focus AT the dwarf leaves him off-centre.
  - [x] Follow the AD-15 blended position; never extrapolate, and snap rather than animate across a `snapshot`.
- [x] **Task 5 — sabotage.** (AC: 12)
  - [x] Write `mutations/10-10-take-the-camera-where-you-want-it.sh` with rows for: delta-time scaling removed, the pan clamp removed, the `--camera` value discarded (`let _ = camera;`), and the boot-rig guard.
  - [x] `rg` the whole `mutations/` directory for rows quoting `camera.rs`/`setup_camera`/`camera_controls` literals and re-point any this story breaks — `scripts/audit-mutations.py` fails the gate on a stale row.

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
- **No capture statistic guards this framing.** Changed pixels have a 6.0 % same-build floor (55,284 px), and the mean-luminance floor documented as 0.0048 measured **0.0724** on this build — see issue #98. Both are struck as gates. Note also that `task-0-control.md`'s "mean luminance" is a plain RGB average, NOT the project's Rec.601 (`10-7-signoff/lumstats.py`); its 766x RED headroom is inflated for the same reason. Captures are observations here, not assertions.
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
# SUPERSEDED (Wolf, 2026-09-17, issue #98): the luminance tolerance is struck — this build's
#   same-build swing (0.0724) exceeds it. Record the capture's range-check line as an
#   observation; the reaches-the-rig TEST carries the proof that the flag reaches the rig.
#   Do not assert a luminance threshold.
```

**The deliberate RED, with its restore step.** Whatever the readout prints, break it before trusting it:

```bash
# RED: in setup_camera, discard the parsed value -- `let _ = camera;`
# EXPECTED: the reaches-the-rig test goes RED. (The luminance half is struck with AC1's clause —
#           issue #98. The test going red IS the proof, and is exactly what `--distance` lacked
#           when `let _ = distance;` left all 106 tests green.)
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

- **Tasks 0 and 1 — Codex `gpt-5.6-terra`**, reasoning effort high, via `scripts/codex-handoff.sh`.
  Two runs. The first handed back honestly at 193,967 tokens with Task 1 committed and Task 2
  part-written; the second died at 90,208 tokens on `You've hit your usage limit ... try again at
  1:58 PM` — a 5-hour-window exhaustion, confirmed by a probe. No last-message file was written at
  all, which is how a quota death differs from a harness kill (that leaves an EMPTY one).
- **Tasks 2 to 5 — Claude `claude-opus-5[1m]`**, as the direct-implementation fallback on Wolf's
  explicit ruling after being told the cost: the dev/review model split this project relies on is
  lost for that work, so the review will be the same model that wrote it.

The sections below keep Codex's own record for Tasks 0-1 verbatim; Claude's follows under each
heading.

### Debug Log References

- Task 0 build stamp: `cargo build --offline -p gui -p simd -j 8`; `./target/debug/gui --version` printed `gui build 8eb1d1d`, matching clean `HEAD 8eb1d1d`.
- Task 0 control re-take: `control-boot-8eb1d1d-a.png` mean luminance 71.1261; `...-b.png` 71.1985; swing 0.0724. This does not meet the 76.12 / 0.0048 control. Filed live bug #98 with the commands and range measurements.
- RED — `pan_moves_focus_on_the_camera_ground_plane_and_stays_inside_the_world`: before the implementation, `cargo test --offline -p gui pan_moves_focus_on_the_camera_ground_plane_and_stays_inside_the_world -j 8` failed with `no method named pan` and `[i32; 3] == Vec3` type errors.
- RED — `boot_rig_and_transform_are_pinned_by_literals`: initial literal-pin test failed and printed `Transform { translation: Vec3(100.74321, 47.646896, -33.05163), rotation: Quat(-0.202291, 0.41114035, 0.094099894, 0.8838479), scale: Vec3(1.0, 1.0, 1.0) }`; the final test compares those hand-written literals and passed.

**Claude, Tasks 2-5.**

- Full gate at `4b3f75b`: **GATE GREEN 511s** — fmt, clippy `-D warnings`, `cargo test` 126s,
  **pixel guards 351s**, three dependency-edge probes, metrics, bench, mutation audit.
  `RUST_TEST_THREADS=6`, foreground.
- Mutation run: `scripts/mutate.sh .../10-10-take-the-camera-where-you-want-it.sh` — **13/13
  KILLED**. Four re-pointed older rows re-run separately — **4/4 KILLED**. Audit passes at 551 rows.
- Build stamp verified `gui build 4b3f75b` on a clean tree before every capture. It earned its keep:
  straight after the mutation run it read `4b3f75b-dirty`, catching the mutant build that outlives
  the source restore. Every frame below was taken after the rebuild.
- Issues filed from running the system: **#98** (control statistic — corrected, retitled),
  **#99** (the near-white ceiling is unreachable via zoom).

### Completion Notes List

- Task 1: `CameraRig.focus` is now a world-space `Vec3`; panning follows camera right/forward on the ground plane and clamps to the fixed 128×128×32 world bounds. Integer and fractional world-to-render conversion now share `world_to_render_f32`.
- Task 0: stamp was verified, but the manual control pair moved. The task remains open pending a ruling/fix for #98; the baseline was not changed.
  - **RESOLVED (orchestrator, same day).** The pair had not moved; the DOCUMENTED figure was a different statistic. See the Task 0 note below and #98's correction. Task 0 is closed.

**Claude, Tasks 2-5.**

**Task 0's outcome, since it reshaped AC1.** The re-take did not confirm the baseline; it falsified
the record. `task-0-control.md`'s "mean luminance 76.1236" is a plain unweighted RGB channel
average, not luminance — it reproduces to four decimal places as the RGB mean of the committed
creation frame, whose Rec.601 value (`10-7-signoff/lumstats.py`, the statistic `pixel_guard.rs`
asserts on) is **71.1931**. Codex measured 71.13 with the canonical decoder, compared it against the
documented 76.12, and read a 5-point regression. There was none: creation to this build is **0.0670**
(Rec.601) / **0.0589** (RGB). Settled off the four committed control PNGs — no rebuild, no recapture.

The real defect was the floor. The documented **0.0048** swing is one two-sample pair that landed
tight; this build's pair swings **0.0724**, 20x, and five same-build captures spread
`near-white-area` over 0.5655-0.7922%. AC1 gated on 10x 0.0048 = a **0.048** tolerance, *narrower
than the measured noise*, so it would have failed on noise alone. Wolf struck the clause.
`task-0-control.md` now carries a correction box, because its 766x RED headroom is inflated the same
way and the next reader would have re-derived the phantom regression from it.

**Task 2 — the seat controls.** Held keys scale by delta time; mouse and wheel deltas do NOT, because
an event delta is already this frame's movement and scaling it by `dt` makes one physical sweep
depend on the frame rate — the opposite of what "per-second" is for. Three defects in the
half-finished work were fixed rather than inherited: the elapsed-time test could never pass
(`TimePlugin` overwrites a manually advanced `Time` in `First`, so it measured real microsecond frame
time — the arithmetic gave it away, 1.2 rad/s must move yaw 1.2 rad in a second and the runs moved
0.088 and 0.049); its pitch assertion was **vacuous**, both rigs reading the boot 0.45 because only
`KeyD`/`KeyE` were pressed; and the mouse path carried `dt`.

**Task 3 — the instrument.** `--camera` places the rig through the same clamps the live controls use,
and deliberately does NOT copy `--distance`'s `requires --capture` gate. The readout key (`C`) prints
one line ending in the `--camera` argument that reproduces the rig, so the line IS the save format.
`camera_readout_line` is the ONLY formatter, and the capture's near-white failure names its framing
with the same function, so a readout that disagreed with a failing capture cannot happen.
`LastCameraReadout` exists because a bare `println!` is unreachable by a test, and an untested
evidence channel manufactures false evidence rather than merely missing true evidence.

**Task 4 — select and frame.** The framing SOLVES the composition push rather than approximating it:
`transform()` looks at `composition_target()`, so aiming the focus at a dwarf leaves him 33 cells
off-centre; setting the composition target TO the dwarf makes the look-at point the dwarf, so he
projects at exactly (0.5, 0.5) at any yaw, pitch or distance. The follow reads his DRAWN translation
— the one `blend_entities` wrote this frame — so it inherits AD-15 rather than re-deriving it.
Ordering is explicit: `select_dwarf` after `camera_controls`, `frame_selected_dwarf` after
`ProjectionSet`, both in `Update` so the camera Transform is propagated this frame instead of
trailing by one. NOTE: the follow writes the focus without pan's clamp — centring a dwarf needs a
focus offset from him by the push, which near a corner lands outside the world, and clamping would
decentre exactly the dwarves hardest to see.

Two defects in my own first draft of these tests, both found by running them: a second
`ButtonInput::press` on an already-pressed button records NO `just_pressed` under `MinimalPlugins`,
so the second click in a test silently did nothing (`click_once` now releases AND clears); and the
tracking test gave the walk no time, so the dwarf travelled 0.06 cells across 40 frames and a broken
follow was indistinguishable from a working one.

**Task 5 — sabotage. 13 of 13 KILLED**, plus 4 re-pointed older rows re-run and KILLED.

| mutation | test that died | result |
| --- | --- | --- |
| delta-time scaling leaves the key rates per-frame | `camera_controls_are_scaled_by_elapsed_time` | KILLED |
| mouse deltas are scaled by delta time as well | `mouse_drag_maps_the_same_motion_at_every_frame_rate` | KILLED |
| the pan clamp lets the focus leave the world | `pan_moves_focus_on_the_camera_ground_plane_and_stays_inside_the_world` | KILLED |
| the boot yaw moves out from under the pinned framing | `boot_rig_and_transform_are_pinned_by_literals` | KILLED |
| the fractional inverse transform mirrors an axis | `the_fractional_transform_pair_round_trips_and_pins_its_handedness` | KILLED |
| the `--camera` value is discarded after parsing | `the_camera_flag_reaches_the_camera_rig_rather_than_merely_parsing` | KILLED |
| the readout rounds the framing it prints | `the_camera_readout_round_trips_through_the_flag_exactly` | KILLED |
| the readout prints the same line for every rig | `the_camera_readout_differs_whenever_the_rig_differs` | KILLED |
| the near-white ceiling stops naming its framing | `blown_pool_range_failure_is_a_real_panic_not_a_successful_capture` | KILLED |
| the framing skips the composition push | `a_left_click_selects_the_nearest_dwarf_and_frames_him_at_screen_centre` | KILLED |
| the pick radius is removed | `escape_releases_the_selection_and_an_empty_click_leaves_the_rig_untouched` | KILLED |
| escape stops releasing the selection | `escape_releases_the_selection_and_an_empty_click_leaves_the_rig_untouched` | KILLED |
| the follow solves the framing once instead of tracking | `the_focus_tracks_the_selected_dwarf_as_he_walks` | KILLED |

Re-pointed by this story's edits and **re-run to prove they still kill** (applying is not killing):
`9-1` "near-white area ceiling assertion is deleted" KILLED, `9-1` "capture reports after the
blown-pool assertion" KILLED, `m2-1` "camera controls drop out of the update tuple" KILLED, `5-4`
"close zoom loses the camp" KILLED.

**RED output, per test, observed before each green** (restored from file copies, never
`git checkout --`, because the fix was uncommitted at the time):

- `key_scale = multiplier` -> yaw **72.700005 vs 36.700012** for the same 0.25 s elapsed. Worth
  recording: under this sabotage pitch and distance BOTH saturate at their clamps in both runs
  (1.4207964 and 500.0), so the run-to-run equality assertions are blind to it and **only yaw
  discriminates**.
- mouse deltas scaled by `dt` -> yaw **0.6994993 vs 0.6990004**, and the sweep barely moves the rig.
- `setup_camera: let _ = start;` -> yaw left **0.7**, right **1.25**, with **159 other tests still
  green** — the same shape as `--distance` at 7.2, where the identical sabotage left all 106 green.
  Run against the whole lib, so this is exclusivity, not a focused invocation.
- readout formatted `{:.2}` -> killed the round-trip AND the differs test.
- ceiling drops its framing -> `"near-white area is 9.7656%, above the 0.9461% ceiling calibrated on
  boot7.png"`, no framing named.
- `focus = target`, push unsolved -> the dwarf projects at **y 0.7794** instead of 0.5, 28% of screen
  height out.
- pick radius removed -> a corner click selected the dwarf across the frame.
- Escape release removed -> the selection survived Escape.
- follow gated on `is_changed` -> focus **identical** before and after, `Vec3(27.239794, -19.259184,
  1.0)`.

**Instrument observations** — observations, not assertions, since AC1's tolerance was struck:

```
boot, no flag:  warm-lit pixels=28881 ground-median-luminance=81 near-white-area=0.8173%
                blown-pool=0.5808% p99-luminance=190.8 resolution=1280x720   exit 0
--camera 0.7,0.45,90,64,64,9:
                warm-lit pixels=27029 ground-median-luminance=81 near-white-area=0.7019%
                blown-pool=0.5007% p99-luminance=186.2 resolution=1280x720   exit 0
```

Rec.601 mean luminance **71.409** (no flag) against **71.238** (flag at boot values) — the flag
reproduces the boot frame. The 0.171 gap is 2.4x the two-sample swing this story's own control
published, which is the reason the clause was struck rather than a defect in the flag.

**What I could NOT demonstrate, and why.** AC11's near-white path could not be reached on the LIVE
instrument. `validate_capture_ranges` asserts the ground-median ceiling (180) BEFORE near-white, and
zooming raises both, so `--camera 0.7,0.45,20,64,64,9` exits 101 on `the valley floor reads 202`
without ever reaching near-white. The recorded `--distance 80` trigger no longer trips either:
**0.7897% this run against 1.1134% in the control record, exit 0**. AC11 is closed by the unit test
(RED observed, mutation-killed); the live gap is filed as **#99**. The readout KEY likewise cannot be
exercised headlessly — there is no window to press a key into — which is precisely what
`LastCameraReadout` and the formatter tests exist for.

### File List

- `crates/gui/src/camera.rs` — world-space focus, pan, boot literal guard.
- `crates/gui/src/transform.rs` — fractional world-to-render conversion.
- `crates/gui/src/pick.rs` — test rig updated for `Vec3` focus.
- `crates/gui/tests/headless.rs` — test rig literals updated for `Vec3` focus.
- `_bmad-output/implementation-artifacts/10-10-signoff/control-boot-8eb1d1d-a.png` — Task 0 control evidence.
- `_bmad-output/implementation-artifacts/10-10-signoff/control-boot-8eb1d1d-b.png` — Task 0 control evidence.
- `_bmad-output/implementation-artifacts/10-10-take-the-camera-where-you-want-it.md` — Task 0/1 evidence and status.

**Claude, Tasks 2-5** (production)
- `crates/gui/src/camera.rs` — `place`, `composition_push`, `frame_render_point`,
  `camera_readout_line`, clamp constants
- `crates/gui/src/transform.rs` — `render_to_world_f32`, integer form delegating to it
- `crates/gui/src/ingest.rs` — `camera_controls` (mouse, wheel, delta time, shift), `--camera`
  parse / `CameraStart` / rig, `camera_readout` + `LastCameraReadout`, `configured_app` mouse
  resource, selection-system registration and ordering
- `crates/gui/src/pick.rs` — `SelectedDwarf`, `select_dwarf`, `nearest_dwarf_to`,
  `frame_selected_dwarf`, `DrawnEntities`
- `crates/gui/src/capture.rs` — the near-white failure names its framing

**Tests**
- `crates/gui/tests/headless.rs` — elapsed-time, mouse-drag, select / frame / track / release /
  no-wire, plus `click_once` and `drawn_translation`
- `crates/gui/tests/capture.rs` — the ceiling failure names its framing

**Artifacts and records**
- `_bmad-output/implementation-artifacts/mutations/10-10-take-the-camera-where-you-want-it.sh` — NEW
- `_bmad-output/implementation-artifacts/mutations/9-1-the-frame-stops-blowing-out.sh` — 2 rows re-pointed
- `_bmad-output/implementation-artifacts/mutations/m2-1-live-app-systems.sh` — 1 row re-pointed
- `_bmad-output/implementation-artifacts/mutations/5-4-the-cold-boot.sh` — 1 row re-pointed
- `_bmad-output/implementation-artifacts/mutations/8-2-designate-with-the-mouse.sh`,
  `mutations/5-3-a-window-onto-the-valley.sh` — re-pointed during Task 1
- `_bmad-output/implementation-artifacts/10-10-signoff/task-0-control.md` — correction box
- `_bmad-output/implementation-artifacts/10-10-signoff/boot-4b3f75b.png`,
  `camera-boot-4b3f75b.png`, `camera-d80-4b3f75b.png`, `camera-near-4b3f75b.png` — NEW.
  `camera-near` is the close-zoom frame that trips the GROUND ceiling (#99), NOT a near-white trip;
  `camera-d80` is the framing the control record says should trip near-white and does not.
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-17 | **Tasks 2-5 implemented directly by Claude** after Codex exhausted its 5-hour usage window mid-Task-2 — Wolf's ruling, taking the loss of the dev/review model split. Mouse orbit/pan, wheel zoom and a shift multiplier, with per-second scaling on HELD KEYS ONLY (an event delta is already this frame's movement). `--camera` and the readout key, sharing one formatter with the capture's ceiling message. Dwarf selection and framing that SOLVES the composition push, following the AD-15 blended position. 13-row sabotage table, 13/13 KILLED, plus 4 re-pointed older rows re-run and killed. Full gate GREEN 511s including the pixel-guard tier. Filed #99: the near-white ceiling is unreachable via zoom because the ground-median ceiling fires first, and the control record's `--distance 80` trigger no longer trips. |
| 2026-09-17 | **AC1's luminance clause STRUCK on Wolf's ruling (issue #98).** Task 0's control re-take found the documented control statistic was a plain RGB channel average mislabelled "mean luminance" (76.1236 reproduces exactly as the RGB mean of the committed creation frame; its Rec.601 luminance is 71.19), so the reported "baseline moved 5 points" was a statistic mismatch — the look never moved. The real defect: the documented 0.0048 same-build swing is a lucky-tight two-sample pair, and this build's swing is 0.0724, larger than AC1's own 0.048 tolerance, making the clause unsatisfiable by noise. Ruling: drop the pixel clause, keep the hand-written-literal rig guard as the whole of AC1. Task 3's required luminance observation and the RED's luminance half are superseded with it; the reaches-the-rig test carries that proof. **This edit changes an Acceptance Criterion, outside the dev workflow's normally permitted sections — made on an explicit ruling and logged here for that reason.** |
| 2026-09-17 | Created. Control pair, same-build noise floor and the `--distance 80` RED measured at creation on `5452c4d`; AC1 rewritten against mean luminance after the changed-pixel statistic was shown to have a 6 % floor. |
| 2026-09-17 | Task 1: made camera focus movable in world space with a literal boot-rig guard; re-took Task 0 control and filed #98 for the moved baseline. |
