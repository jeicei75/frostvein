---
baseline_commit: 74421747
---

# Story 11.2: The Miniature

Status: review

## Story

As the boss,
I want the valley to read as a small lit world in a lens,
so that the frame looks like the art shots — a focus plane on the camp, the far valley softening
into air — instead of a uniformly sharp render.

## Not stacked — branch off `main`

11.1a and 11.1b are both merged. `main` is `7442174`, clean, and the binaries in `target/` are
built from it. Branch slug `story-11-2-the-miniature`. Every "X did not change" AC is proved with
`git diff 7442174..HEAD`, never against a branch tip.

## What the creation probe already settled — READ THIS BEFORE WRITING ANY CODE

Both mechanisms were added to the camera temporarily on `7442174`, measured, and reverted
(`git diff HEAD -- crates/` empty, `gui --version` re-checked). Full figures and method:
`11-2-signoff/creation-probe.md`. **Four things it found, each of which would have cost a session:**

1. **Both mechanisms render under lavapipe.** The epic's unknown is answered for depth of field
   and for volumetric fog. That proves the plugins run, not that either looks right.
2. **`DepthOfField::default()` is a silent no-op at this world scale.** At `aperture_f_stops: 1.0`
   the far ridge's circle of confusion is **0.09 px** of a 720-px frame. Every window, every level
   statistic, the log and the exit code are identical to no-DoF. `N ≈ 0.05` is what reads.
3. **`focal_distance = rig.distance` misses the camp by 46 %.** `CameraRig::transform`
   (`camera.rs:108-127`) aims at the camp pushed `BOOT_COMPOSITION_FORWARD = 33.0` units along the
   view direction, so at boot the camera is **≈ 61.7** units from the camp while `rig.distance` is
   **90**. Measured: `camp-focus` sharpness peaks between 55 and 70, and reads *worse* at 90 than
   at 55.
4. **Volumetric fog at defaults darkens instead of hazing — and the settings that fix it are
   derived, not tuned.** `FogVolume { density_factor: 0.06 }` at Bevy's defaults took the valley
   floor from **69 to 30** and panicked `capture.rs:1498`. Three corrections, all found by probe,
   turn that into real aerial perspective at **ground median 69 — identical to the control**:
   a night-scaled `ambient_intensity`, a much lower density, and a **vertical density texture**
   so the volume has no visible top face. Settings and figures below; it also dimmed the aurora,
   so **`fog_enabled: false` does not protect the sky from volumetric fog** (see the trap list).

## The instrument you inherit

**`11-2-signoff/sharpness.py`, built and proved at creation.** Depth of field is invisible to
every statistic 11.1 built: `creases.py` and `campstats.py` report LEVEL statistics and a blur
preserves a window's mean almost exactly. `sharpness.py` reports LOCAL CONTRAST — the mean and p90
of the 4-neighbour Laplacian of integer Rec.601 luma, with mean |∇| as a cross-check.

Same-build floor, four controls on `7442174`
(`--headless --static-world --lights-steady --subdiv 4 --frames 160`, **fresh `simd` per capture**):

| window | rect | `lap_mean` | spread |
| --- | --- | --- | ---: |
| `far-ridge` | 450,120 → 900,250 | 13.4539 … 13.4721 | **0.0182** |
| `camp-focus` | 500,400 → 760,620 | 18.1468 … 18.2653 | **0.1185** |
| `near-foreground` | 140,620 → 640,716 | 8.2646 … 8.2729 | **0.0083** |
| `sky-stars` | 60,10 → 460,110 | 2.2727 … 2.2731 | **0.0004** |

**Proved both ways** (`blur_proof.py`, which runs `sharpness.py`'s own `statistics()`): a
**one-pixel** box blur moves `far-ridge` from 13.4539 to 4.1482 — **512× the floor**.

## The aperture — RULED f/0.05 (Wolf, 2026-09-21)

At `focal_distance = 61.7`, `max_depth = 120`, sweeping the aperture:

| `N` | `far-ridge` | Δ | `camp-focus` | Δ | stars ≥150 | sky peak |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| control | 13.4539 | — | 18.1899 | — | 78 | 158 |
| 0.02 | 3.4937 | −74.0 % | 15.3214 | −15.8 % | **0** | 138 |
| 0.03 | 5.4806 | −59.3 % | 16.9519 | −6.8 % | 8 | 156 |
| **0.05** | **8.5994** | **−36.1 %** | **17.6085** | **−3.2 %** | **32** | 157 |
| 0.07 | 11.0019 | −18.2 % | 18.0195 | −0.9 % | 38 | 158 |
| 0.10 | 13.0579 | −2.9 % | 18.1671 | −0.1 % | 67 | 158 |

**The aperture that softens the far ridge is the aperture that takes the stars' bright cores**,
because the stars are behind the ridge and one full-screen pass cannot treat them separately.

**Wolf ruled `0.05` on 2026-09-21, at story creation, against this table and the filed frame
`probe-dof-focal61.7-f0.05-maxdepth120.png`.** It is `DOF_APERTURE_F_STOPS` and the story does not
move it. Re-judged on the vehicle at the sitting like every other look value (AC14); if it moves
there, the ratio in AC1 has a bar of 3 against a measured 11.3, so a re-ruling anywhere in the
0.02–0.10 bracket still satisfies the guard.

## The haze settings, derived at creation — and why there is no split

An earlier draft of this story recommended splitting it, because volumetric fog at Bevy's defaults
drove the valley floor under `GROUND_LUMINANCE_FLOOR` and that looked like a guard bounding an art
decision. **It was not. It was three wrong settings, and all three have derived corrections.**

| step | setting | ground median | far ridge median | far ridge `lap_mean` | stars ≥150 |
| --- | --- | ---: | ---: | ---: | ---: |
| control, no haze | — | 69 | 49 | 13.45 | 78 |
| Bevy defaults | `density 0.06`, `ambient_intensity 0.1`, white | **30 — PANIC** | — | — | — |
| + night ambient, `density 0.015` | box volume, top y 42 | 67 | 74 | 5.60 | **0** |
| + volume bounded to the skyline | box volume, top y 28 | 67 | 65 | 9.59 | 78 |
| **+ vertical density texture** | **the recipe below** | **69** | **69** | **9.30** | **78** |

**The three corrections:**

1. **`ambient_intensity` must be scaled to this scene's ambient.** Bevy's default `0.1` is stated
   to match `AmbientLight::default().brightness = 80.0` (`bevy_light/src/ambient_light.rs:36`).
   **This client runs ambient at 1500.0**, so the fog was lit at 1/19th of the scene's own ambient
   while its extinction ran at full strength. Derived value: `0.1 × 1500/80 = **1.875**`, with
   `ambient_color = night_lighting().ambient`. **This is a derivation, not a tuned number — if the
   ambient constant moves, this moves with it.**
2. **`fog_color` stays WHITE. Do NOT set it to the sky colour.** The tech-art doc's rule that
   "the fog colour and the rim's target colour MUST both be exactly the sky colour" is a rule about
   **`DistanceFog`**, which BLENDS toward a colour. Volumetric fog **adds in-scattered light**, so a
   sky-coloured (5,12,28) fog scatters near-black and darkens the valley: measured, it took the far
   ridge from 49 down to 31 instead of up to 69.
3. **A bounded box has a VISIBLE TOP FACE, and no statistic caught it.** With the volume ending at
   y 28 the numbers looked ideal — sky median 25, all 78 star cores intact — and the frame has a
   hard bright band ruled across the sky where the camera sees the volume's top plane. It is
   obvious to the eye and invisible to every window. The fix is `FogVolume::density_texture`: a
   small 3D texture with a vertical ramp, so the fog thins out instead of ending at a plane. The
   client already builds a procedural `Image` for the aurora gradient (`atmosphere.rs`,
   `aurora_gradient_image()`) — same pattern.

**The proven recipe** (`probe-haze-d0.015-density-ramp.png`, exit 0, ground median 69):

```
VolumetricFog { ambient_color: night_lighting().ambient, ambient_intensity: 1.875, ..default() }
VolumetricLight on the sun (already shadow_maps_enabled: true)
FogVolume { density_factor: 0.015, density_texture: Some(<vertical ramp>), ..default() }
  Transform::from_xyz(64.0, 18.0, -64.0).with_scale(Vec3::new(160.0, 48.0, 160.0))
  ramp: R8Unorm 3D, 1 x 64 x 1, full to v 0.54, linear to zero by v 0.83
        (v maps 0..1 over world y -6..42, so the fade runs y 20 -> 34, around SKYLINE_MAX = 26)
```

**So `GROUND_LUMINANCE_FLOOR` is not touched, not approached, and not a decision.** It reads 69 —
the control's own value — with 14 of headroom. No split, no ruling, one story.

**What is still Wolf's, at the sitting:** re-judging the ruled aperture and the haze strength
(`density_factor` and the ramp's fade band) against the reference art on the vehicle. Both start
from filed frames, and the story tunes neither.

## Acceptance Criteria

1. Depth of field is on the camera and its effect is **separated by depth**, measured with
   `sharpness.py` on two captures that differ only by `--fx-off dof`: `far-ridge` `lap_mean` falls
   by more than its same-build floor, and its **fractional** fall is at least **3×**
   `camp-focus`'s. The measured ratio, both windows' figures and the floor are recorded. (At the
   creation values the ratio is 11.3; the bar is 3 so a Wolf ruling on aperture cannot make a
   correct implementation fail this AC.)
2. `camp-focus` `lap_mean` retains at least **90 %** of its no-DoF control value on the same pair.
   The focal plane is on the camp, not merely somewhere.
3. The focal distance is **derived from the camera's own transform and the rig's aim point**, not
   from `rig.distance`. A unit test asserts it reads ≈ 61.7 (±1.0) at boot framing where
   `rig.distance` is 90, and that it changes when the rig orbits and when it zooms.
   *(Mechanism AC, deliberately: the epic asks that orbit and zoom "do not lose focus on the
   world", and `rig.distance` is 46 % wrong at boot while satisfying every outcome test taken at
   one framing. The mechanism is the requirement.)*
4. The outcome of AC3 at a second framing: with `--distance 40`, AC1's ratio test still passes on a
   fresh capture pair. Figures recorded.
5. The aperture is `DOF_APERTURE_F_STOPS = 0.05` as ruled, and `max_depth` is a named finite
   constant rather than `f32::INFINITY`, with the value and the reason recorded. With DoF on, the `sky-stars` window's **peak** luma stays within 3 of its control value
   (157–158 at creation) — the stars may soften, they may not be erased.
6. A guard fails if the aperture is ever returned to a physically-plausible value. It must assert
   the **rendered** consequence: a test that only checks the `DepthOfField` component is present
   does not satisfy this, because `DepthOfField::default()` renders a frame indistinguishable from
   no depth of field at all and exits 0.
7. Volumetric haze is on: `VolumetricFog` on the camera, `VolumetricLight` on the sun, and one
   `FogVolume` over the valley carrying a **vertical density texture**. Measured on-vs-off with
   `--fx-off haze`, the far valley gains aerial depth the existing `DistanceFog` did not:
   `far-ridge`'s **median rises by at least 10** (creation recipe: 49 → 69) **and** its `lap_mean`
   **falls by at least 15 %** (creation: 13.45 → 9.30). Level alone does not satisfy this — a
   bounded box raised the level while leaving contrast untouched, which is haze you cannot see.
8. **The haze does not swallow the sky, and leaves no seam.** With haze on, the `sky-stars`
   window's count of pixels ≥ 150 is **unchanged** from its control (78 at creation, and the
   creation recipe holds all 78), and its median moves by at most 1. The sky materials keep
   `fog_enabled: false` and the story records that this flag is **not** what protects them.
   **Plus a human check that no statistic can replace:** the capture is opened and looked at, and
   the frame carries no hard edge where the fog volume ends. A bounded box passed every window in
   this AC while ruling a bright band across the sky.
9. **No ceiling and no floor is moved.** `GROUND_LUMINANCE_FLOOR` (55, `capture.rs:584`, asserted
   at `:1498`) and both near-white ceilings are unchanged, and the haze capture's ground median
   stays at or above the no-haze control's value minus its own floor (creation: 69 on both). If a
   density Wolf later asks for drives the floor red, the story records both figures and **Wolf
   rules** — it does not tune the guard and does not silence it.
10. The rim dissolve still owns the world edge: `appearance.rs`'s `rim_dissolved_color` and
    `RIM_LEVELS` are unchanged, and the world-edge tiles still close on the sky colour.
11. `--fx-off` accepts `dof` and `haze` beside `fxaa`, `ao` and `bloom` as a set, each reaching the
    **spawned camera** rather than only `Args`; naming an effect removes its component, omitting it
    leaves it present. An unknown name errors naming all five accepted names.
12. Each new effect has a seat toggle beside `F10`/`F11`/`F12`, and the readout names each one's
    state. A test presses the real keys and asserts the recorded readout changes.
13. On the vehicle, `--perf-log` at boot framing is read with all effects on and with each of the
    two new ones off, and the p50 frame times recorded. NFR6's 60 fps bar is re-read; if an effect
    costs it, the figures are recorded and Wolf rules rather than the story tuning anything.
14. Wolf signs off both halves at the sitting against the reference art, with the frame pair filed.
15. `git diff 7442174..HEAD --stat` on `crates/protocol`, `crates/sim-core` and
    `crates/client-core` is empty.
16. Every mutation row in `mutations/11-2-the-miniature.sh` is shown to KILL, and the table's
    output is pasted into the Dev Agent Record.

## Tasks / Subtasks

- [x] **Task 0 — read the creation probe, then confirm the control on YOUR build** (AC1)
  - [x] Read `11-2-signoff/creation-probe.md` end to end. The lavapipe probe the epic asks for as
        every story's first task is **already done for both mechanisms** — do not repeat it.
  - [x] Take four controls on your own branch point and re-run `sharpness.py`. The floors above are
        for `7442174`; a floor is build-specific ([[delta-needs-a-noise-floor]]). Four minimum.
- [x] **Task 1 — depth of field on the camera, with a derived focal distance** (AC1, AC2, AC3, AC5)
  - [x] Add `DepthOfField` to the camera tuple at `ingest.rs:1408-1437`, beside `Bloom::default()`.
  - [x] Add `update_dof_from_camera`, mirroring `update_fog_from_camera` (`ingest.rs:1877-1882`):
        query `(&GlobalTransform, &CameraRig, &mut DepthOfField)` and set
        `focal_distance = transform.translation().distance(world_to_render_f32(rig.focus))`.
        Set the same value at spawn so frame 0 is not focused at the default 10.0.
  - [x] `DOF_APERTURE_F_STOPS = 0.05` — **Wolf's ruling, 2026-09-21**; do not move it. Name
        `DOF_MAX_DEPTH` beside it. One line each saying they are non-physical on purpose and why
        (see the trap list), and that the aperture is a ruled value.
  - [x] Unit test for AC3 on the derivation, driven from a `CameraRig`, not from a rendered frame.
- [x] **Task 2 — the rendered guard, and the RED that proves it** (AC1, AC2, AC5, AC6)
  - [x] Port `sharpness.py`'s Laplacian statistic into `crates/gui/tests/pixel_guard.rs` beside the
        existing Rec.601 helpers and assert AC1's ratio, AC2's retention and AC5's sky peak.
  - [x] Run the deliberate RED in Verification below **before** accepting any green.
- [x] **Task 3 — the switches** (AC11, AC12)
  - [x] Extend `CameraEffect` (`ingest.rs:157-186`) with `Dof` and `Haze`; keys F7/F8 or the next
        free pair — check `designate.rs` and `slice.rs` for collisions before choosing.
  - [x] Follow 11.1a's **insert/remove** pattern, not an `enabled` flag. `DepthOfField` and
        `VolumetricFog` have **no `#[require]`s**, so a plain `remove` is correct for both and
        `remove_with_requires` must not be used ([[remove-with-requires-strips-render-sync]]).
  - [x] The reaches-the-camera test at `ingest.rs:2577-2620` is the shape to copy.
  - [x] **Collapse the two effect sites into one helper** (Wolf ruled 2026-09-21 — this de-duplication
        IS in scope). `--fx-off` at spawn (`ingest.rs:1438-1470`) and the live key toggles
        (`ingest.rs:1596-1616`) each hand-write an insert/remove arm per effect; with this story that
        is **five effects across two sites**. One `apply_effect(camera, effect, on)` called by both is
        a second concrete caller, not an abstraction with one.
  - [x] **Three asymmetries MUST survive the collapse, and each needs a test that fails if it does
        not** — they are the whole risk of merging these sites:
        (a) AO takes `DepthPrepass` and `NormalPrepass` WITH it, named explicitly;
        (b) `Bloom` is removed WITHOUT its required `Hdr`, deliberately, so `--fx-off bloom` stays a
        measure of bloom's marginal contribution and not an Hdr+bloom comparison;
        (c) `remove_with_requires` is never used — it crashes the render-world sync and
        `MinimalPlugins` tests pass on a client that cannot draw.
        If the helper cannot express all three without a per-effect branch, keep the branch — a
        helper that flattens them is worse than the duplication it removes.
- [x] **Task 4 — volumetric haze** (AC7, AC8, AC9, AC10)
  - [x] Build the recipe above as written — it is proved, not proposed. `VolumetricFog` on the
        camera, `VolumetricLight` on the sun (`ingest.rs:1483-1490`, already
        `shadow_maps_enabled: true`), one `FogVolume` as a `ClientLocal` entity.
  - [x] `ambient_intensity` is **derived, not hardcoded**: write it as
        `0.1 * night_lighting().ambient_brightness / 80.0` with a comment naming Bevy's
        `AmbientLight::default().brightness` (`bevy_light/src/ambient_light.rs:36`) as the 80.0, so
        it follows the ambient constant instead of silently going stale. A mutation row pins it.
  - [x] Build the density ramp as a procedural `Image`, following `aurora_gradient_image()`
        (`atmosphere.rs`). A unit test asserts the ramp is full at the valley floor and zero above
        the fade band — a uniform texture would restore the visible top face and every pixel window
        would still pass.
  - [x] Leave `fog_color` at its default. Do **not** set it to the sky colour (see the traps).
- [x] **Task 5 — the instrument task** (AC1, AC7, AC8)
  - [x] The human-visible instrument for this story is
        `gui <port> --headless --static-world --lights-steady --subdiv 4 --frames 160 --capture <png>`
        read through `sharpness.py`. It exists and is proved both ways; the story's own obligation
        is the **test of the instrument** — `blur_proof.py` must be re-run on this branch's control
        and its table pasted into the Dev Agent Record.
- [x] **Task 6 — the tech-art doc rows** (AC5, AC9)
  - [x] Add the depth-of-field and haze rows to `docs/tech-art-guidelines.md` beside 11.1's
        exposure, AO and bloom rows (`:69-71`), each carrying its measured figure, the aperture
        marked as Wolf's ruling of 2026-09-21, and `ambient_intensity` shown as the derivation
        `0.1 × ambient_brightness / 80` rather than as the number 1.875.
  - [x] Add one line to the same doc's edge-treatment section recording that the sky bypasses
        `Exposure` by three routes and does NOT bypass volumetric fog, citing #113. The doc
        currently says only `Sky materials MUST set fog_enabled: false` (`:211`), which is true
        of `DistanceFog` and irrelevant to the haze this story adds.
- [x] **Task 7 — the sitting** (AC13, AC14)
  - [x] Write `11-2-signoff/vehicle-card.md` naming the exact commands, the frames to capture and
        the questions Wolf is being asked — the aperture from the bracket above, and the haze
        density against the floor.
- [x] **Task 8 — mutations and the gate** (AC15, AC16)
  - [x] `mutations/11-2-the-miniature.sh`: at minimum the aperture constant, `max_depth`, the focal
        derivation, the fog density and the `VolumetricLight` on the sun.
  - [x] Every row `assert s.count(old) == 1`, not `assert old in s` — eleven existing rows carry no
        count guard and this story does not add a twelfth.
  - [x] Run `audit-mutations.py` **after** `cargo fmt`; formatting alone has orphaned six rows.

## Dev Notes

### Scope guardrails — do NOT

- Do NOT change `Msaa`, `Exposure`, `Fxaa`, `ScreenSpaceAmbientOcclusion` or `Bloom`. 11.1a and
  11.1b own them and the EV100 is Wolf's ruling of 2026-09-19.
- Do NOT add a day/night cycle, a moon, a clock or a sky turn. That is 11.3, and issues **#113**
  (the sky is a `ClearColor` and bypasses exposure) and **#108** (the enclosed-sky oracle) are
  unrouted and belong to it.
- Do NOT touch `protocol`, `sim-core`, `client-core` or `simd` (AC15).
- Do NOT raise, lower or silence `NEAR_WHITE_AREA_CEILING`, `BLOWN_POOL_FRACTION_CEILING` or
  `capture.rs:1498`'s ground-median floor (AC9).
- Do NOT change a light constant, a colour, a `BOOT_*` constant or the rim dissolve. This story adds
  MECHANISMS; the look judgement under them is Wolf's. `bench_contract.rs:128-162` greps `camera.rs`
  for those literals and will catch you.
- Do NOT add a dependency or change a `bevy` feature. `3d_bevy_render` already carries
  `bevy_post_process` (`DepthOfFieldPlugin`, `bevy_post_process/src/lib.rs:35`) and `bevy_pbr`
  (`VolumetricFogPlugin`, `bevy_pbr/src/lib.rs:236`). Crossing a feature boundary rebuilds ~400
  crates and has nearly OOMed this devpod.
- Do NOT build a general post-effect registry, and do NOT extract a camera-settings bundle or a
  camera preset system — Wolf ruled 2026-09-21 (YAGNI, one camera, no second one planned), filed
  as **#117**. Five named effects mirroring the light toggles is the whole of it. **Collapsing the
  two effect-toggle sites into one helper is a separate thing and IS allowed — see Task 3.**

### What already exists — build on it, do not rebuild it

- The camera tuple, `ingest.rs:1408-1437`: `Msaa::Off`, `Exposure { ev100: 10.5 }`, `Fxaa`, SSAO,
  `Bloom`, `Projection::Perspective { fov: BOOT_VERTICAL_FOV }`, the `CameraRig`, `AmbientLight`
  and `DistanceFog`.
- `update_fog_from_camera` (`ingest.rs:1877-1882`) — **the exact shape Task 1 copies**: one system
  reading the rig and writing one field of an effect component every frame.
- `--fx-off` as a set with per-effect insert/remove and a readout (`ingest.rs:157-215, 1438-1470,
  1596-1616`); `--lights-steady` (`:1125`, consumed at `:2044-2052`); `--static-world`, repaired
  2026-09-19 so it actually freezes.
- The sun already carries `shadow_maps_enabled: true` (`ingest.rs:1486`), which is
  `VolumetricLight`'s stated prerequisite.
- The stars and the aurora already set `fog_enabled: false` and `unlit: true`
  (`atmosphere.rs:267-282`).

### Key decisions & traps

- **`DepthOfField::default()` renders a frame identical to no depth of field, and says nothing.**
  The shader (`dof.wgsl:117-137`) computes
  `coc = scale·|depth − focus| / (depth·(focus − f))`, `scale = f²/(sensor_height·N)`. At the
  client's 45° FOV and this world's scale that is **0.09 px** at `N = 1.0`. There is no log line,
  no warning and no exit code. **Check the measured `lap_mean` before believing anything.**
- **`max_depth` defaults to `f32::INFINITY`, and the sky is at infinity.** As `depth → ∞`,
  `coc → scale/focus`: at `N = 0.02` that is **15.9 px** on the stars, and measurably **zero** star
  pixels survived at luma 150. A finite `max_depth` is mandatory, not a refinement.
- **`rig.distance` is not the distance to anything you can see.** `composition_target()`
  (`camera.rs:117-127`) pushes the aim point 33 units down the view axis, scaled by
  `(distance / BOOT_DISTANCE).min(1.0)`. Deriving the focal distance from the camera transform gets
  the push, the zoom and the orbit for free; restating `33.0` anywhere is how a framing solve drifts
  from the framing it inverts, and `camera.rs:124` says so in its own words.
- **Volumetric fog's failure mode is total and SILENT.** `extract_volumetric_fog`
  (`bevy_pbr/src/volumetric_fog/render.rs:240-257`) begins `if volumetric_lights.is_empty()` and
  then **removes `VolumetricFog`, `ViewVolumetricFog` and every `FogVolume` from the render world
  and returns** — no `error!`, no `warn!`, nothing. Forget `VolumetricLight` on the sun and you get
  a clean frame, exit 0, and no fog. This is the same class as 11.1's MSAA/SSAO trap and it is
  worse, because SSAO at least logs.
- **`fog_enabled: false` does not protect the sky from volumetric fog.** That flag is read **only**
  inside `#ifdef DISTANCE_FOG` at `bevy_pbr/src/render/pbr_functions.wgsl:1002`, gating `apply_fog`
  alone. The probe frame dimmed the aurora with both sky materials' flags already false. **What
  bounds the haze is the `FogVolume`'s extent; what bounds the blur is `max_depth`.** The epic's
  parenthetical names the wrong mechanism — keep the flag false, and say in the record why it is not
  the reason.
- **The sky bypasses `Exposure` by THREE routes, and does NOT bypass volumetric fog.** `view.exposure`
  is applied only in `apply_pbr_lighting` (`pbr_functions.wgsl:863`), which `pbr.wgsl:81-84` calls
  only on the LIT path — so the `ClearColor` background (`ingest.rs:574`), the stars and the aurora
  (both `unlit: true`) are all exposure-invisible, and a `Skybox` would be a fourth bypass
  (`skybox.wgsl:80`). Haze, by contrast, reaches the sky — the probe dimmed the aurora. **#113 is
  routed to 11.3; this story's two sky figures (AC5, AC8) are PROVISIONAL against whichever fix
  11.3 takes, and re-baselining them is one capture pair.** Record that, do not act on it here.
- **`FogVolume`'s defaults are daylight figures**, and the probe at them read as a dimmer rather
  than haze: ground median **69 → 30**, p99 **163 → 88**. All three corrections are in the recipe
  above. The two that are easy to get wrong a second time:
  - **`fog_color` stays WHITE.** The tech-art doc's "fog colour MUST be exactly the sky colour"
    rule is about `DistanceFog`, which BLENDS toward a colour. Volumetric fog ADDS in-scattered
    light, so a sky-coloured fog scatters near-black: measured, the far ridge went **49 → 31**
    instead of 49 → 69. Same word, opposite operation.
  - **A bounded `FogVolume` box has a visible top face and NO STATISTIC CAUGHT IT.** With the
    volume ending at y 28 the sky window read median 25 and all 78 star cores intact — and the
    frame has a hard bright band ruled across the sky. Found by opening the PNG. Use the density
    texture, and **look at the frame** before believing a window.
- **`ambient_intensity` is relative to `AmbientLight::default().brightness = 80.0`**, not to this
  scene's 1500.0. Bevy's `0.1` default therefore lights the fog at 1/19th of the scene's own
  ambient while extinction runs at full strength. Derive it; do not pick it.
- **`--lights-off sun` does not kill the fog.** `apply_lighting_toggles` (`ingest.rs:1638-1643`)
  sets the sun's `illuminance` to 0 rather than removing the light, so `VolumetricLight` survives
  and `extract_volumetric_fog` still runs. The shafts go, the volume stays. Say which you measured.
- **`fall_snow` is not pinned by anything.** `atmosphere.rs:321` moves 96 flakes by
  `time.delta_secs()` — wall clock. Neither `--lights-steady` nor `--static-world` touches it, and a
  flake field is high-contrast structure sitting in every window this story measures, in DoF's near
  field. The creation controls came out tight anyway; **measure the floor on your branch before
  leaning on any figure**, and if it is not tight, that is the finding, not an obstacle.
- **Exit 0 is not a result, and exit 101 still leaves a PNG.** `save_then_validate`
  (`capture.rs:1317`) writes before it asserts, so a frame that trips the ground-median floor is
  still measurable. Both halves of AC9 are read that way.
- **A mutant binary outlives a source restore.** `mutate.sh` restores the source and leaves the last
  mutant build on disk. Rebuild and check `gui --version` shows no `-dirty` before any capture.
- **Rec.601 vs Rec.709.** `sharpness.py`, `creases.py`, `campstats.py` and `pixel_guard.rs:38-87`
  are Rec.601 integer; `capture.rs:625` is Rec.709. Say which any new figure is in.
- **`--frames 2` never captures** (dies on `capture is black`, `capture.rs:1478`); `--cursor` is
  dead headless; use `--frames 160`.
- **One fresh `simd` per capture.** `--static-world` freezes at the tick reached at connect, so two
  captures sharing a daemon froze worlds 35 ticks apart.

### Project Structure

| File | Change |
| --- | --- |
| `crates/gui/src/ingest.rs` | UPDATE — `DepthOfField` + `VolumetricFog` on the camera; `update_dof_from_camera`; `FogVolume` spawn; `VolumetricLight` on the sun; two `CameraEffect` variants; two keys; readout |
| `crates/gui/src/camera.rs` | UNCHANGED — read `composition_target`/`composition_push`, do not restate them |
| `crates/gui/tests/pixel_guard.rs` | UPDATE — AC1/AC2 ratio guard, AC5 sky peak, AC6's aperture RED, AC7/AC8 haze pair |
| `crates/gui/tests/headless.rs` | UPDATE — live-entity and key-path tests |
| `crates/gui/tests/capture.rs` | UNCHANGED — AC9 depends on its floors being untouched |
| `docs/tech-art-guidelines.md` | UPDATE — the DoF and haze rows beside 11.1's |
| `_bmad-output/implementation-artifacts/11-2-signoff/vehicle-card.md` | NEW |
| `_bmad-output/implementation-artifacts/mutations/11-2-the-miniature.sh` | NEW |

### Verification

From the repo root. **`scripts/gate.sh` has no thread knob and its Bevy test apps exhaust 23 GB —
export `RUST_TEST_THREADS=6` and run it in the FOREGROUND.** Full gate ~420 s; the pre-commit hook
is the fast tier only, so a successful push is not full-gate evidence. **Never
`git commit --no-verify`.**

```bash
RUST_TEST_THREADS=6 scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/11-2-the-miniature.sh
git status --porcelain     # issue #104: mutate.sh leaves tracked non-Rust targets sabotaged
```

**The control, which RAN at creation on `7442174`** — one fresh daemon per capture:

```bash
./target/debug/simd 7501 &
./target/debug/gui 7501 --headless --static-world --lights-steady --subdiv 4 --frames 160 \
  --capture _bmad-output/implementation-artifacts/11-2-signoff/control-<sha>-a.png
# creation observation, exit 0, four times:
#   capture range check: warm-lit pixels=26976..27032 ground-median-luminance=69
#     near-white-area=0.4656..0.4676% blown-pool=0.4270..0.4322% p99-luminance=163.0..163.3
python3 _bmad-output/implementation-artifacts/11-2-signoff/sharpness.py control-*.png
#   far-ridge lap_mean 13.4539..13.4721 · camp-focus 18.1468..18.2653
#   near-foreground 8.2646..8.2729 · sky-stars 2.2727..2.2731
```

**THE DELIBERATE RED — this is AC6's whole purpose.** With depth of field on, put the aperture back
to a physically plausible value:

```bash
# RED: set DOF_APERTURE_F_STOPS to 1.0 (Bevy's own default, via PhysicalCameraParameters).
# EXPECTED: AC1's ratio guard goes RED and `far-ridge` lap_mean returns to its no-DoF value,
#           because the circle of confusion falls to 0.09 px of a 720-px frame. The capture
#           still succeeds, still exits 0, and the log says nothing. THAT is the point.
# RESTORE: revert the constant, rebuild, confirm `gui --version` shows no -dirty, re-capture.
```

**A second RED, for AC7's half:** remove `VolumetricLight` from the sun. Expected: the fog vanishes
entirely with **no log line at all** (`render.rs:240-257` returns silently), the frame reads as the
no-haze control, and AC7's guard goes red. **If either RED leaves its guard green, that guard is
worthless no matter how green the suite is.**

### References

- `_bmad-output/planning-artifacts/epics.md` § Epic 11 / Story 11.2 — source ACs and the execution
  order ruling (11.1 → 11.2 → 11.3)
- `11-2-signoff/creation-probe.md` — the control, the instrument proof, both mechanisms probed
  live, the aperture bracket, the floor collision
- `11-1b-the-air-has-depth.md` — the render path this story stands on, and the `--fx-off` pattern
- Pinned Bevy 0.19.0: `bevy_post_process/src/dof/mod.rs:74-115` (component), `:308-320` (defaults),
  `:663-672` (extract; `DEPTH_PREPASS_TEXTURE_SUPPORTED` is `true` on native) ·
  `bevy_post_process/src/dof/dof.wgsl:117-137` (the CoC maths) ·
  `bevy_light/src/volumetric.rs:10-16, 18-58, 60-130` ·
  `bevy_pbr/src/volumetric_fog/render.rs:240-257` (the silent early return) ·
  `bevy_pbr/src/render/pbr_functions.wgsl:1002` (`fog_enabled` is DistanceFog only) ·
  `bevy_pbr/src/lib.rs:236`, `bevy_post_process/src/lib.rs:35` (both plugins in `DefaultPlugins`)
- `deferred-work.md:1237-1246` — llvmpipe under-reads near-white by ~16 %; headless area figures are
  deltas only
- NFR6 — 60 fps at working zoom, ≥30 at full vista, read on the vehicle with `--perf-log`
- Open issues: **#104** (`mutate.sh` leaves tracked targets sabotaged), **#90** (near-white swing),
  **#75** (lighting re-judgement), **#72** (pixel-guard race), **#84** (`--frames` ignored without
  `--capture`);
  **#113** (the sky and exposure — routed to 11.3), **#117** (camera bundle — filed, not to be built; its two-site duplication IS cleaned here)

### Previous-story intelligence

- 11.1b's thirteen unticked `[Review][Patch]` items were a **false backlog** — eleven were already
  fixed and only the boxes had not moved. Reconcile a merged review list against the tree before
  treating it as scope.
- 11.1a's `--fx-off` is insert/remove, and its reaches-the-camera test (`ingest.rs:2577-2620`) is the
  shape to copy. Removing a component does **not** remove what required it — that is deliberate for
  `Bloom`/`Hdr` and must not be "fixed".
- Commit **per task**, green. 11.1a's dev was delegated across three Codex runs and the ones that
  handed back at their context limit were resumable only because of that.
- Author as `Völundr <jeicei75@gmail.com>` on the **first** commit — this clone's git config
  defaults to `jeicei75`, so pass `--author` every time. No `Co-Authored-By: Claude` trailer and no
  "Generated with Claude" footer (CLAUDE.md §8).
- Wolf merges mid-session: re-check the branch point and staging before every commit and push, and
  verify `git merge-base --is-ancestor origin/main HEAD`.

## Dev Agent Record

### Agent Model Used

GPT-5.6-Codex

### Debug Log References

- Render DoF green (Rec.601): far-ridge 9.3720 → 6.4426 (31.3%), camp 16.6325 → 16.1498 (2.9%), ratio 10.771, sky peak 158 → 157.
- Render haze green (Rec.601): far median 50 → 69, `lap_mean` 8.6568 → 6.4470 (25.5%), sky median 24 → 25, stars ≥150 32 → 32.
- AC6 RED at f/1.0: exit 101 only from guard; far 9.4230 → 9.3725 (0.5%), camp 16.4759 → 16.4086 (0.4%), ratio 1.314, sky 158 → 158; no render warning observed.
- AC7 RED without `VolumetricLight`: exit 101 only from guard; far median 50 → 50, `lap_mean` 8.6573 → 8.6352 (0.3%), sky 24 → 24, stars ≥150 32 → 32; no fog log line observed.
- Mutation table: all six KILLED; audit: 601 rows apply (11 legacy rows unguarded, none added here).
- Full gate RED after 1320s only in the previous AO post-stack control: expected LL/LR 93/93, observed 91/93; darkening 0.425. Filed #119.

### Completion Notes List

- Implemented ruled f/0.05 finite-depth DoF from camera transform to aim point; vertical-ramped density-0.015 volumetric haze with derived ambient intensity; `dof`/`haze` fx-off switches and F1/F2 readout toggles. (Dev first chose F13/F14; a standard
  keyboard stops at F12, so those toggles were unreachable at the seat while AC12's synthetic
  key-press test stayed green. Corrected to F1/F2 at orchestrator verification, Wolf's ruling
  2026-09-21; the keymap is now full and issue #118 carries the rethink.)
- `apply_effect` retains AO prepasses, Bloom's Hdr, and plain removal. All six named mutations kill their named tests.
- Blocked: do not tune previous AO control under this story. #119 needs Wolf's decision. AC8 eye check, AC13 vehicle performance, AC14 signoff, four independent controls, and blur proof remain outstanding.

### Orchestrator verification (Claude Opus 5, 2026-09-21)

Codex exited 0; verified rather than trusted. No 401/auth line in the run log, three commits all
authored `Völundr <jeicei75@gmail.com>`, claimed files present, and the gate red exactly as it
reported. **`codex review --base main` never ran** — its sandbox could not open `/tmp`'s bubblewrap
mount-registry lock — so the self-gate contributed nothing this story and the code review carries
that weight alone.

**F13/F14 → F1/F2 (`db4d30b`).** Dev walked upward from F12 for the two new toggles. A standard
keyboard stops at F12, so both were unreachable at the seat while AC12's synthetic key-press test
stayed green — a live mechanism no hand can operate, green. `rg` over `crates/gui/src/` put F3–F12
already in use, leaving F1 and F2 as the only free keys. **The keymap is now fully allocated**;
issue **#118** carries the rethink Wolf asked for, with the inventory and the constraint that a
test pressing a `KeyCode` proves nothing about reachability.

**#119 re-attributed by measurement, and one reading corrected.** Codex blamed the haze; the
failing window `OPEN_SNOW_LL` sits inside the region DoF blurs, so both were tested. Fresh daemon
per capture, this branch's build:

| capture | open-snow-LL | LR | terrace mean |
| --- | ---: | ---: | ---: |
| all effects on | **91** | 93 | 60.423 |
| `--fx-off dof` | **91** | 93 | 60.301 |
| `--fx-off haze` | **93** | 93 | 54.954 |
| `--fx-off dof,haze` | **93** | 93 | 54.700 |

DoF is exonerated; the haze moves it. **91 is stable: 91/91/91/91 across four fresh captures,
spread 0** — the same standard used when this control was last re-baselined.

I also suspected the haze was eating the AO guard's headroom and **that was overstated**: AO
darkening reads **~0.44 with haze on** (ao-off 60.860/60.851 against ao-on 60.403/60.403/60.424)
and **~0.48 with haze off**, against a 0.30 floor. Haze's own marginal cost is ~0.04; the erosion
from the historical 0.62–0.64 predates this story.

**AC8's human half — DONE.** `seam-check-6194756-all-on.png` opened and compared against
`seam-check-6194756-haze-off.png`. **No hard edge and no band where the fog volume ends**: the sky
gradient is continuous from horizon to top, the star field is intact, and the haze reads as aerial
depth on the far ridge rather than as a dimmer. This is the check a bounded box passed on every
pixel window while ruling a bright band across the sky, so it was done by eye on the frame.

**Wolf's two findings from the seat, 2026-09-21:**

1. *"focal point should move to dwarf when selected"* — **REAL, fixed in `6194756`.**
   `frame_selected_dwarf` centres a picked dwarf via `CameraRig::frame_render_point`, which writes
   the rig focus OFFSET from him by the composition push. Focusing the aim point therefore focused
   `33 × 20/90 ≈ 7.3` units short of him at `SELECT_DISTANCE`, putting the figure just picked
   outside the focal plane at f/0.05. RED first: `focal_distance=8.133` while the dwarf stood
   `1.118` away. `dof_subject` now makes a selected dwarf the focal subject; with no selection the
   subject is still the rig's aim point, so **every boot-framing figure in this record stands
   unchanged** and AC3's guard still passes. Mutation row KILLED.
2. *"turning dof on off has issues"* — **NOT REPRODUCED, and I cannot reproduce it here.** The ECS
   path is sound: `apply_effect` re-inserts the ruled component (not `DepthOfField::default()`),
   and `update_dof_from_camera.after(effect_controls)` earns its ordering — Bevy's auto-inserted
   sync point means the re-inserted component is corrected in the SAME frame, so there is no focus
   pop. Pinned by a new test and its mutation row (removing `.after(effect_controls)` KILLS it).
   Headless cannot press keys, so the seat symptom is unreproducible on the devpod. **No issue
   filed: I have neither a reproduction nor a measurement, and this tracker is a measurement
   archive.** Wolf: what does "issues" look like — a one-frame flash, no visible change at all, or
   artefacts that persist?

**Full gate re-run independently on `a726e75` — RED at 1248s, ONE failure, and it is #119.**
All ten other rendered-frame guards pass, including both of this story's own
(`dof_softens_the_far_ridge_while_retaining_camp_focus_and_stars` and
`haze_lifts_and_softens_the_far_valley_without_swallowing_the_sky`). The gate's own line reads
`terrace mean AO-on=60.363 AO-off=60.799 darkening=0.436; open-snow LL/LR median=91/93` — matching
the independent measurements above. **So #119 is the single thing between this story and a green
gate, and it is a decision, not a defect to fix here.**

Recommended, NOT applied: re-baseline `CONTROL_OPEN_SNOW_MEDIAN` 93 -> 91 with a comment naming
11.2's haze, exactly as it went 116 -> 93 on 2026-09-19 for Wolf's EV100 ruling. That constant's own
comment sets the precedent and the reasoning: the assertion is EQUALITY, so a re-baseline is
"neither weaker nor stronger", and a control must track a deliberate change to the frame. The
alternative — tuning `FOG_DENSITY_FACTOR` until the old control passes — is the
[[guard-bounds-the-art-decision]] trap and would let a guard calibrated under a superseded frame
bound Wolf's ruled haze. **Wolf rules; the story does not move it.**

**A note on reading the gate:** the run was invoked through a pipe, and the harness reported
`exit code 0` because that was `tail`'s status, not `gate.sh`'s. The gate itself printed
`GATE RED 1248s`. Read the verdict line, never the pipeline's exit code.

**A harness trap found while testing, worth knowing:** `configured_app` runs no input-clearing
system, so `ButtonInput::just_pressed` is **sticky** — every `app.update()` with a stale press
toggles the effect again. A two-tap sequence silently becomes four toggles. Every existing key
test presses exactly once, so none of them could see it. Both new tests clear the input after each
tap and say why.

### Orchestrator verification, day 2 (Claude Opus 5, 2026-09-22)

Both things this story was blocked on came back from Wolf, and one of them turned the
"unreproduced" symptom into a real defect with a root cause.

**Wolf's second seat finding REPRODUCED, root-caused and FIXED. It was never a DoF-toggle bug.**
Asked what "issues" looked like, Wolf described it exactly: *"first off disabled and black screen,
turn on depth overlay (green, red, blue). Then I need to cycle it to get dof back."* The mechanism:

- `ScreenSpaceAmbientOcclusion` is `#[require(DepthPrepass, NormalPrepass)]`. **`DepthOfField` and
  `VolumetricFog` declare NOTHING** — verified in the Bevy 0.19.0 sources
  (`bevy_post_process-0.19.0/src/dof/mod.rs`, `bevy_light-0.19.0/src/volumetric.rs`). Both simply
  read whatever depth prepass the camera happens to carry.
- So both of 11.2's new effects were silently borrowing **AO's** depth buffer, and
  `apply_effect`'s AO-off branch removed both prepasses unconditionally. Before 11.2 that was
  harmless — AO was the only consumer. **11.2 is what made it bite**, which is why it is fixed
  here rather than filed.
- Measured through a synthetic key sequence, `(depth, normal, dof, haze)`:

| step | depth | normal | dof | haze |
| --- | ---: | ---: | ---: | ---: |
| boot, everything on | 1 | 1 | 1 | 1 |
| **F11 ambient occlusion off** | **0** | **0** | 1 | 1 |
| F1 dof off, then F1 dof on | **0** | 0 | 1 | 1 |
| **F11 ambient occlusion back on** | **1** | 1 | 1 | 1 |
| boot with `--fx-off ao` | **0** | **0** | 1 | 1 |

  Cycling **F1 never restores it**; only **F11** does. That is precisely "I need to cycle it to get
  dof back", and it is why the readout kept reporting `F1 dof on` throughout — the component was
  there, the buffer it samples was not. **`--fx-off ao` shipped the same depth-less DoF at boot.**

**Why every instrument here said it was fine.** Captured the broken state headlessly
(`--fx-off ao`, which reproduces it at boot) against a control, one fresh daemon per capture, with
a same-build noise floor: floor 1.65–5.26% of pixels, DoF-on-vs-off signal 25.75–28.12%. The frame
renders **correctly on lavapipe** — no black screen, no overlay. The visual damage is
**venue-sited**, the same devpod-vs-vehicle split as [[capture-ceiling-is-venue-sited]], so no
amount of headless capture on this box was ever going to show it. The seat was the only instrument
that could see this, and it did.

**The fix.** Prepass ownership is now computed from the whole effect set in one place
(`sync_prepasses`) instead of being guessed at a single toggle site: depth is inserted when **any**
of AO, DoF or haze is on and removed only when all three are off; normals stay AO-only, because
nothing else samples them. Called from both the boot and live-toggle sites. RED first — the new
test failed at `(0, 0)` against an expected `(1, 0)` — then green.

One existing expectation changed with it, and it was the buggy one:
`fx_off_reaches_the_live_camera_and_rejects_unknown_effects` asserted `--fx-off ao` leaves
`(0, 0, 1)`. It now asserts `(1, 0, 1)`, plus a new `--fx-off ao,dof,haze` case for `(0, 0, 1)`.
**This weakens nothing.** Its stated purpose was that "ao off" must not pay for a pass nothing
samples — since 11.2 the depth pass *is* sampled by two other effects, so the pass AO can still be
charged for is the normal one. AO's marginal cost is measured with DoF and haze on, and a pass the
other two would pay anyway was never part of it. **AC8's vehicle cost delta should be read that
way**: `--fx-off ao` now measures SSAO + the normal prepass, not SSAO + both prepasses.

**#119 RULED by Wolf and APPLIED: re-baseline, not a tuned constant.** Re-measured on this build
first — `terrace mean AO-on=60.417 AO-off=60.848 darkening=0.431; open-snow LL/LR median=91/93`,
matching yesterday's independent figures. **The two windows no longer share a value**, so the
single `CONTROL_OPEN_SNOW_MEDIAN` was split into `CONTROL_OPEN_SNOW_LL_MEDIAN = 91` (inside the fog
volume's depth) and `CONTROL_OPEN_SNOW_LR_MEDIAN = 93` (outside it, unmoved). A single constant
would now have to be wrong about one of them. `FOG_DENSITY_FACTOR` was **not** touched — tuning the
art to satisfy a guard is the [[guard-bounds-the-art-decision]] trap Wolf's ruling avoids.

The prepass fix does **not** disturb any figure in this record: at boot every effect is on, so the
camera carries both prepasses before and after the change. Only `--fx-off ao` captures move, and
#119's measurements never used one.

**Haze strength — OPEN, and deliberately not tuned.** Wolf, on the same pass: *"haze could be
stronger.. cannot see the difference between on/off."* The measurement agrees it is faint — the
haze moves the one window it reaches by **2 levels out of 255** (93 → 91). But the comparison may
also have been made in the broken state above: **if F11 had been pressed at any point first, the
haze had no depth buffer when it was judged.** Strengthening it now would risk tuning the art to
compensate for a bug that is now fixed. **Wolf's call: re-check F2 on/off at the seat on a build
carrying the prepass fix, before any value moves.** If it still reads flat, the density is chosen
from his reading — and the LL control is re-baselined again, by the same reasoning as #119.

### The seat recordings, and what they overturned (2026-09-22, afternoon)

Wolf recorded the toggles at the seat. Two recordings settled three things that no instrument on
this devpod could have reached, and overturned two of my own conclusions.

**1. F1 AND F2 WERE NEVER OURS. This is the whole of "turning dof on off has issues".**

`DefaultPlugins` pulls in `bevy_dev_tools::render_debug::RenderDebugOverlayPlugin` whenever the
`bevy_dev_tools` and `bevy_pbr` features are both on (`bevy_internal-0.19.0/src/default_plugins.rs:95`),
and its `handle_input` hardcodes **F1 -> cycle the depth/normal debug overlay** and **F2 -> cycle
that overlay's opacity** (`bevy_dev_tools-0.19.0/src/render_debug.rs:107` and `:110`). `db4d30b` had
moved dof and haze onto exactly those keys. Every press therefore drove BOTH handlers.

Wolf's own log lines name it outright:

```
09:04:56  Debug Overlay: Depth
09:04:58  Debug Overlay: Normal
09:05:00  Debug Overlay Disabled
09:05:02  Debug Overlay: Depth
09:05:05  Debug Overlay Opacity: 0.5
09:05:08  Debug Overlay Opacity: 0.8
```

Measured frame statistics match every one of those events to within half a second, and explain
every state I had been calling corruption:

| what the frame did | mean luminance | what it actually was |
| --- | ---: | --- |
| healthy | 77.6 | no overlay |
| black | 12.9 | `Debug Overlay: Depth` |
| green/pink/blue | 156.7 (green excess +22.8) | `Debug Overlay: Normal` |
| dimmed plateaus | 58.4, then 40.6 | `Opacity 0.5`, then `0.8` |

**There was never any renderer corruption, and my prepass fix was never implicated.** I had
reasoned from a step-per-toggle pattern to "my `sync_prepasses` churn is a prime suspect"; a
baseline recording on `74f8cd3` (the commit BEFORE that fix) reproduced every plateau identically,
which is what exonerated it. Wolf had named the cause in his first sentence -- "depth overlay
(green, red, blue)" was Bevy's own feature name, and I read it as a description rather than a label.

**The fix, and the reason no test caught it.** Every test here builds on `MinimalPlugins`, which
does not include that plugin, so AC12's synthetic key-press test handed F1 straight to our handler
with nothing to shadow it -- green, on a keymap that was unusable at the seat. A test cannot see a
collision with a plugin it never loads, so the reserved keys are now written down instead:
`the_client_keymap_avoids_keys_other_plugins_have_claimed` fails if any of our controls lands on a
key Bevy has claimed, or if two of our own controls collide. Mutation-killed, with the message
naming both claimants.

Wolf ruled: keep Bevy's overlay (it is a useful instrument), move ours, and free a key because
"fxaa switch is not needed in F10 right now"; then, seeing the result, **reorganise the whole row --
"bevy F1, F2, effects, lights maybe from biggest to smallest"**. The row now reads:

| key | control |
| --- | --- |
| F1 / F2 | Bevy's debug overlay -- reserved |
| F3 | perf-log frame mark |
| F4 F5 F6 F7 | effects: haze, dof, bloom, ao -- widest-acting first |
| F8 F9 F10 F11 F12 | lights: sun, ambient, campfire, torches, lanterns -- biggest reach first |
| *(none)* | fps overlay -- always on (Wolf: "does not harm now"), and `--capture` forces it off |
| *(none)* | fxaa, `--fx-off fxaa` only |

The overlay giving up its toggle is what made the row fit: the perf mark had been exiled to `M`
because F3 was taken, and it came straight back. `the_interactive_overlay_is_on_and_has_no_key_to_restore_it`
pins the new default, because with no toggle left there is nothing to recover it with.

**The first attempt at that reorganisation put haze on F3 -- already the fps overlay's key.** The
guard did not catch it, because the guard I had just written enumerated only the lights, the effects
and the perf mark. A guard that knows about part of the keymap certifies the rest. It now lists
every key the client binds, each with its site, and the sabotage row for it moves haze onto F3
specifically. Before choosing, every `KeyCode` binding inside the Bevy crates was checked:
`picking_debug`'s F3 is only a doc-comment example and its plugin is not in `DefaultPlugins`, and
`easy_screenshot` (Space, PrintScreen) is not either. **F1 and F2 are the only keys Bevy takes.**
The keymap still wants the rethink in #118.

**2. THE HAZE DOES NOTHING ON THE VEHICLE'S GPU. Wolf was right and my "faint" reading was wrong.**

> **CORRECTED 2026-09-23 -- the conclusion below is WRONG; the measurement is not.** It was never
> the GPU. **F4 could not turn the haze off live on any machine**: Bevy 0.19 only inserts
> `VolumetricFog` on the render-world camera and clears it only when no light is volumetric, so
> both halves of this recording had haze ON. A headless `--fx-off haze` pair on the 4080 showed the
> real difference. Fixed in `cf5e008`; Wolf confirmed F4 at the seat. Kept below as the record.

Measured inside his first recording, static camera, 9 frames averaged per state, HUD rows excluded,
during a window where the debug overlay was still disabled (its first event is the F1 press that
follows):

| | haze OFF | haze ON |
| --- | ---: | ---: |
| mean luminance | 77.720 | 77.650 |

**Delta -0.070 levels**, and the across-state difference (mean |d| 0.437) is SMALLER than the
frame-to-frame noise inside each state (0.486 / 0.430). Flat across every row eighth.

On this devpod the same toggle, measured against a same-build noise floor of 0.30-0.95% of pixels,
moves the whole frame **+3.96 levels (+6.34%)**, changes **71.7%** of pixels, and lifts the
mid-distance band by about **+10**. So the fog renders under lavapipe and not on the vehicle.

**My earlier "2 levels out of 255, genuinely faint" was wrong**, and wrong in an instructive way: it
was the `OPEN_SNOW_LL` control window only, and that window sits in the lower-left of the frame --
precisely where the row profile shows the haze does nothing (`-0.23`). I generalised a whole-effect
judgement from a window that is nearly blind to the effect. **Tuning `FOG_DENSITY_FACTOR` is now
explicitly the wrong move**: a larger number multiplied by zero is still zero.

**3. Depth of field DOES work at the seat.** At equal luminance (77.1), mid-valley Laplacian energy
is **180 with dof off and 106 with dof on** -- a 41% blur. That half of the story is doing its job
on the vehicle.

### Close-out (Claude Opus 5.5, 2026-09-23) -- the sitting, the outstanding evidence, and the deviations

**AC14 -- SIGNED OFF by Wolf at the seat, 2026-09-23:** *"I think we are done now what comes to
AC14 also.. we will tweak it when we have more content etc but overall already now gui starts to
look damn cool."* Both halves accepted at their shipped values (f/0.05, `FOG_DENSITY_FACTOR`
0.015 with the ramp); retuning waits for real content. Frame pair filed:
`11-2-signoff/all-on-51b1db3.png` beside the controls below.

**AC13 -- READ at the seat by Wolf, fullscreen:** haze on ~60 fps, haze off ~140 fps (a small window
runs ~140 either way, so the cost is per-pixel). Ruled: accepted, no optimisation now. The card's
headless `--perf-log` runs were not usable for ranking effects -- frame time drifted ~35% within
each 2.7-6.8 s run -- and the seat reading supersedes them.

**Task 0 -- four controls on this build, and the floor is now ZERO.** `control-51b1db3-{a,b,c,d}.png`
(`--fx-off dof,haze`, one fresh `simd --pause-at 120` each), all exit 0, read by `sharpness.py`:

| window | `lap_mean` a = b = c = d | spread | all effects on |
| --- | ---: | ---: | ---: |
| `far-ridge` | 13.4970 | **0** | 6.4110 |
| `camp-focus` | 18.3981 | **0** | 16.1508 |
| `near-foreground` | 8.2366 | **0** | 4.0617 |
| `sky-stars` | 2.2697 | **0** | 1.8812 |

At creation the same-build spreads were 0.0083-0.1185. They are zero because `b5a0e2a` removed the
three timing leaks from a frozen capture (snow, walk phase, per-frame facing); a floor of zero is a
PROPERTY of this build, not a lucky draw, and two same-build captures are now bit-identical.

**Task 5 -- `blur_proof.py` on this branch's control** (`control-51b1db3-a.png`):

| window | sharp | box r=1 | box r=2 | box r=4 |
| --- | ---: | ---: | ---: | ---: |
| `far-ridge` | 13.4970 | 4.1554 | 2.2755 | 1.3204 |
| `camp-focus` | 18.3981 | 6.1253 | 3.5133 | 1.9127 |
| `near-foreground` | 8.2366 | 2.7415 | 1.6500 | 1.0054 |
| `sky-stars` | 2.2697 | 1.0113 | 0.8325 | 0.7391 |

**AC16 -- the full table, 18 rows, run on `51b1db3`:**

```
a physically plausible aperture silently disables miniature blur KILLED
infinite DoF depth erases the star cores                     KILLED
focus returns to the rig orbit radius instead of the aim point KILLED
haze returns to Bevy daylight density                        KILLED
sun stops participating in the volumetric pass               KILLED
fog ambient stops following the night ambient budget         KILLED
a selected dwarf stops being the focal subject               KILLED
re-inserted DoF keeps boot framing for a frame               KILLED
ao off takes the depth prepass dof and haze sample           KILLED
an effect sits on a key bevy_dev_tools already binds         KILLED
two controls of ours land on one key                         KILLED
F4 leaves the sun volumetric, so the fog never leaves the render world KILLED
a frozen world lets its snow fall on the wall clock          KILLED
a frozen world leaves each stride where frame timing put it  KILLED
facing is read once per frame, so a batched step never turns the dwarf KILLED
the static-world pause timeout counts frames again           KILLED
a plain capture fires on its frames with its ticks still missing KILLED
simd drops --pause-at on the floor                           KILLED

All mutations killed.
```

"sun stops participating" was RE-POINTED first: `sync_haze_light` re-inserts `VolumetricLight` every
frame, so removing only the spawn's copy would have survived. Four rows in other stories' tables
(6-1 x3, 10-5 x1) were re-pointed at moved seams and shown to KILL.

**AC15** -- `git diff 7442174..HEAD --stat` on `crates/protocol`, `crates/sim-core` and
`crates/client-core` is empty (checked 2026-09-23).

**DEVIATIONS the review must see:**
1. **`crates/simd` was touched** (`b09d03a`, `simd --pause-at`), against the Dev Notes guardrail
   "Do NOT touch ... `simd`". AC15 itself does not name `simd`. Wolf approved it explicitly as an
   instrument fix: a hand-started client at the vehicle could not reach the tick-120 freeze. No
   wire or sim change -- it queues the existing `SetSpeed { at_tick }` command at startup.
2. **Out-of-epic work landed on this branch**, all Wolf-approved and driven by the sitting:
   F4 render-world leak (`cf5e008`); static-world determinism (`b5a0e2a`, including
   `ingest::apply_wire_delta`, which the headless tests now share with production); tick-waiting
   captures (`6743fb1`); wall-clock pause timeout (`53b50d2`); `push.sh --fast` (`08f7aee`); vehicle
   docs and create/dev-story overrides in the `launch-gui.ps1` form (`e2fa878`).
3. **Keys differ from Task 3's F7/F8 suggestion**: effects are F4-F7, lights F8-F12 -- the reasoning
   is in the seat-recordings section above.
4. **Earlier record corrected, not rewritten**: "haze inert on the vehicle GPU" is withdrawn in place.
   The fog renders identically on the 4080 headless (+3.93 vs +3.95 levels on lavapipe).

**Gate:** full gate GREEN on `b09d03a` (1708 s, `RUST_TEST_THREADS=2`); every later commit is docs,
mutation rows, or this record, each fast-tier green. The full gate runs again before the PR.

### File List


- `crates/gui/src/ingest.rs`
- `crates/gui/tests/pixel_guard.rs`
- `docs/tech-art-guidelines.md`
- `_bmad-output/implementation-artifacts/11-2-signoff/vehicle-card.md`
- `_bmad-output/implementation-artifacts/mutations/11-2-the-miniature.sh`
- `_bmad-output/implementation-artifacts/mutations/10-7-the-sun-lights-the-valley.sh`
- `_bmad-output/implementation-artifacts/mutations/m2-1-live-app-systems.sh`
- `crates/gui/src/pick.rs` (orchestrator: `DrawnEntities` widened to `pub(crate)` for reuse)
- `_bmad-output/implementation-artifacts/11-2-signoff/seam-check-6194756-all-on.png` (NEW)
- `_bmad-output/implementation-artifacts/11-2-signoff/seam-check-6194756-haze-off.png` (NEW)
- 2026-09-23 close-out: `crates/gui/src/{atmosphere,capture,command,project}.rs`,
  `crates/gui/tests/headless.rs`, `crates/simd/src/main.rs` (deviation 1), `README.md`,
  `scripts/push.sh`, `scripts/launch-gui.ps1`, `_bmad/custom/bmad-{create,dev}-story.toml`,
  mutation rows in `6-1-the-world-moves.sh` and `10-5-dwarves-worth-looking-at.sh`,
  `11-2-signoff/control-51b1db3-{a,b,c,d}.png` and `all-on-51b1db3.png` (NEW)

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-21 | Aperture RULED `0.05` (Wolf); haze recipe proved and the floor collision struck — no split. |
| 2026-09-21 | Story created. Both mechanisms probed live on `7442174` and reverted; `sharpness.py` built and proved both ways; aperture bracket measured; the `capture.rs:1498` floor collision and the recommended split raised for Wolf. |
| 2026-09-21 | Implemented 11.2 mechanisms and evidence; full gate blocked by #119's previous AO control, so status remains in-progress. |
| 2026-09-21 | Orchestrator verification: F13/F14 → F1/F2 (unreachable keys, #118); #119 re-attributed to haze by measurement, DoF exonerated, LL stable at 91 spread 0; AC8 eye check done, no seam; Wolf's selected-dwarf focus defect fixed and mutation-killed; toggle symptom not reproduced. |
| 2026-09-22 | Wolf's toggle symptom reproduced and root-caused: DoF and haze silently borrowed AO's depth prepass, which AO-off removed. Fixed by `sync_prepasses`, RED first. #119 ruled by Wolf and applied as a re-baseline, the control splitting into LL 91 / LR 93. Haze strength left untouched pending a seat re-check on the fixed build. |
| 2026-09-22 | Seat recordings: F1/F2 were bevy_dev_tools' debug-overlay keys, which is the entire toggle symptom -- dof -> F10, haze -> F3, fxaa unbound, collision guard added. Haze measured INERT on the vehicle GPU (-0.07 levels) while moving +3.96 on lavapipe; my "faint" reading came from a control window blind to the effect and is withdrawn. |
| 2026-09-23 | Haze "inert on the vehicle GPU" withdrawn: F4 never removed the fog from Bevy's render world (`cf5e008`, seat-confirmed). Capture instruments made vehicle-safe: static-world timeout in wall clock (`53b50d2`), deterministic frozen captures -- snow, walk phase, per-delta facing -- which un-flaked the distance-40 DoF guard (`b5a0e2a`), tick-waiting plain captures (`6743fb1`), `simd --pause-at` (`b09d03a`). `push.sh --fast` for branch pushes (`08f7aee`). Full gate GREEN on `b09d03a`. Vehicle card and README rewritten to the seat's `launch-gui.ps1` form. |
| 2026-09-23 | AC13 read at the seat by Wolf, fullscreen: **haze on ~60 fps, haze off ~140 fps** (~9.5 ms/frame for the volumetric fog; a small window runs ~140 either way, so the cost is per-pixel). Wolf's ruling: accepted as is, **no optimisation now**. If it is revisited, the first knob is `VolumetricFog::step_count` (default 64). The card's headless AC13 runs could not rank the effects (35% within-run drift, unequal run lengths, 720p) and are superseded by this seat reading. |
| 2026-09-23 | AC14 signed off by Wolf at the seat. Task 0 controls re-taken on `51b1db3` (same-build floor now 0), Task 5 blur proof re-run, AC16 table 18/18 KILLED after re-pointing the sun-marker row, AC15 empty. `simd` deviation and out-of-epic fixes recorded. Status -> review. |
