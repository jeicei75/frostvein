# 11.2 creation probe — the control, the instrument, and what the two mechanisms actually do

Run 2026-09-21 on `main` @ `7442174`, devpod lavapipe (llvmpipe / Mesa 25.0.7, wgpu Vulkan).
Every capture: `--headless --static-world --lights-steady --subdiv 4 --frames 160`, **a fresh
`simd` per capture** ([[freeze-point-tracks-daemon-uptime]] — captures sharing a daemon freeze
worlds tens of ticks apart).

## 1. The control, and its same-build floor

Four captures, `control-7442174-{a,b,c,d}.png`, all exit 0:

```
capture range check: warm-lit pixels=26976..27032 ground-median-luminance=69
  near-white-area=0.4656..0.4676% blown-pool=0.4270..0.4322% p99-luminance=163.0..163.3
```

`--lights-steady` plus the repaired `--static-world` makes this build far quieter than 11.1b's:
near-white spread is **0.0020 pp** where 11.1b measured **1.4423 pp** unpinned at the camp.

## 2. The instrument: `sharpness.py`

**Depth of field is invisible to every statistic 11.1 built.** `creases.py` and `campstats.py`
report LEVEL statistics (p10 / median / p90 / mean / near-white area) and a blur preserves a
window's mean almost exactly. `sharpness.py` reports LOCAL CONTRAST instead: the mean and p90 of
the 4-neighbour Laplacian of integer Rec.601 luma, with mean |∇| as a cross-check.

Same-build floor across the four controls:

| window | rect | `lap_mean` | spread | `lap_p90` | `grad_mean` spread |
| --- | --- | --- | ---: | --- | ---: |
| `far-ridge` | 450,120 → 900,250 | 13.4539 … 13.4721 | **0.0182** | 41 (0) | 0.0085 |
| `camp-focus` | 500,400 → 760,620 | 18.1468 … 18.2653 | **0.1185** | 59–60 (1) | 0.0859 |
| `near-foreground` | 140,620 → 640,716 | 8.2646 … 8.2729 | **0.0083** | 28 (0) | 0.0210 |

### The instrument is proved both ways

`blur_proof.py` box-blurs the luma plane of `control-7442174-a.png` and runs `sharpness.py`'s own
`statistics()` on the result:

| window | sharp | box r=1 | box r=2 | box r=4 |
| --- | ---: | ---: | ---: | ---: |
| `far-ridge` | 13.4539 | **4.1482** | 2.2702 | 1.3161 |
| `camp-focus` | 18.1899 | **6.0457** | 3.4762 | 1.9027 |
| `near-foreground` | 8.2646 | **2.7534** | 1.6542 | 1.0063 |

A **one-pixel** blur moves `far-ridge` by 9.31 against a **0.0182** floor — **512× the floor**.

## 3. Depth of field: it renders here, and the two obvious settings are both wrong

Probed by adding `DepthOfField` to the camera tuple temporarily (reverted; `git diff HEAD --
crates/` is empty and `gui --version` re-checked).

### 3a. `DepthOfField::default()` is a silent no-op at this world scale

`focal_distance: rig.distance` (90), `aperture_f_stops: 1.0` — every window inside its floor:

| window | control | DoF on, f/1.0 | vs floor |
| --- | ---: | ---: | --- |
| `far-ridge` | 13.4539 | 13.4654 | inside 0.0182 |
| `camp-focus` | 18.1899 | 18.1222 | inside 0.1185 |
| `near-foreground` | 8.2646 | 8.2654 | inside 0.0083 |

The shader (`dof.wgsl:134`) computes
`coc = scale·|depth − focus| / (depth · (focus − f))`, `scale = f²/(sensor_height · N)`,
`f = 0.5·sensor_height / tan(fov/2)`. At `fov = 45°`, `sensor_height = 18.66 mm`, `N = 1.0`:
`f = 0.022524 m`, `scale = 0.027188`, and the far ridge (~150 units) lands at
`coc ≈ 0.087 px` of a 720-px frame. **The mechanism is live and the number is 0.09 px.**
`N = 0.02` gives `coc ≈ 4.4 px`, which is what reads. **The exit code, the log and every level
statistic are identical between "working" and "doing nothing".**

### 3b. `focal_distance = rig.distance` misses the camp by ~46 %

Sweep at `N = 0.02`, `max_depth = ∞`. **`camp-focus` peaks between 55 and 70, not at 90:**

| `focal_distance` | `far-ridge` | `camp-focus` | `near-foreground` |
| ---: | ---: | ---: | ---: |
| control (no DoF) | 13.4539 | 18.1899 | 8.2646 |
| 55 | 2.6853 | 11.3665 | 2.9014 |
| 70 | 5.9610 | 13.7148 | 1.9060 |
| **61.7** | **3.6980** | **15.3410** | 2.2196 |
| 90 (`= rig.distance`) | 10.7585 | 5.9583 | 1.7029 |
| 110 | 11.3293 | 4.0414 | 1.6405 |

**Cause, in the rig:** `CameraRig::transform` (`camera.rs:108-127`) aims at
`composition_target() = world_to_render(focus) + composition_push()`, and
`boot_composition_offset()` pushes that target **`BOOT_COMPOSITION_FORWARD = 33.0`** units along
the view direction, with `BOOT_COMPOSITION_LIFT = -0.5`. The camera sits `rig.distance` from the
*pushed* target, so the distance to the **camp** at boot is
`|48.1·forward_h + 38.7·Y| ≈ 61.7`, not 90. The sweep's peak confirms it.

**So `focal_distance` must be derived from the camera transform and the aim point**
(`camera.translation.distance(world_to_render_f32(rig.focus))`), which tracks orbit, zoom AND
the push — the push itself scales with zoom, `composition_push()` line 127.

### 3c. `max_depth: f32::INFINITY` (the default) blurs the sky

As `depth → ∞`, `coc → scale/focus`: at `N = 0.02`, `focus = 61.7` that is **15.9 px** on the
stars and the aurora, capped only by `max_circle_of_confusion_diameter: 64`. Measured at
`max_depth = 120` the far ridge reads 3.4937 against 3.6980 at `∞`. **The sky needs a finite
`max_depth`, and that — not `fog_enabled` — is what keeps the stars.**

Frames filed: `probe-dof-focal90-f1.0-maxdepth-inf.png` (indistinguishable from the control),
`probe-dof-focal61.7-f0.02-maxdepth-inf.png` (reads as a miniature).

### 3d. The aperture bracket, and the tension Wolf has to rule on

`focal_distance = 61.7`, `max_depth = 120`, sweeping `aperture_f_stops`. `far-ridge` and
`camp-focus` are `lap_mean`; the star columns count pixels at or above that Rec.601 luma inside
the pinned sky window `(60,10)-(460,110)`, whose same-build floor is **0.0004** on `lap_mean`
and **0** on every count.

| `N` | `far-ridge` | Δ | `camp-focus` | Δ | `near-foreground` | stars ≥150 | sky peak |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| control | 13.4539 | — | 18.1899 | — | 8.2646 | 78 | 158 |
| 0.02 | 3.4937 | −74.0 % | 15.3214 | −15.8 % | 2.2171 | **0** | 138 |
| 0.03 | 5.4806 | −59.3 % | 16.9519 | −6.8 % | 2.8370 | 8 | 156 |
| **0.05** | **8.5994** | **−36.1 %** | **17.6085** | **−3.2 %** | 4.2918 | **32** | 157 |
| 0.07 | 11.0019 | −18.2 % | 18.0195 | −0.9 % | 5.4029 | 38 | 158 |
| 0.10 | 13.0579 | −2.9 % | 18.1671 | −0.1 % | 6.6392 | 67 | 158 |

**THE TENSION, stated as a number:** the aperture that softens the far ridge is the aperture that
takes the stars' bright cores. At `N = 0.02` the ridge loses three quarters of its edge contrast
and **not one star pixel reaches 150**. At `N = 0.1` every star is intact and the ridge has not
moved. `N = 0.05` keeps 32 of 78 star cores with the sky's peak unchanged at 157 while the ridge
loses 36 %, and the ridge's fractional loss is **11.3x** the camp's — the separation the effect
exists to produce.

`probe-dof-focal61.7-f0.05-maxdepth120.png` is that frame. **The value is Wolf's to rule; the
bracket is the story's to hand him.**

## 4. Volumetric fog: it renders, it DARKENS, and it trips a shipped floor

Probed with `VolumetricFog::default()` on the camera, `VolumetricLight` on the sun (which already
has `shadow_maps_enabled: true`, `ingest.rs:1486`), and one `FogVolume { density_factor: 0.06 }`
scaled 160×48×160 over the valley.

```
capture range check: warm-lit pixels=17542 ground-median-luminance=30
  near-white-area=0.1029% blown-pool=0.0901% p99-luminance=87.6
thread 'main' panicked at crates/gui/src/capture.rs:1498:5:
  the valley floor reads 30, below the 55 value floor — the frame is a black field, not a lit night
```

Three things, all in one capture (`probe-volfog-density0.06-defaults.png`, exit 101, PNG still written):

1. **It renders under lavapipe.** The epic's unknown is answered for this mechanism too.
2. **It absorbs far more than it scatters at night.** `FogVolume`'s defaults are
   `absorption: 0.3`, `scattering: 0.3`, and `VolumetricFog`'s are `ambient_color: WHITE`,
   `ambient_intensity: 0.1` — calibrated for a daylit scene. Against this scene's night sun
   the result is a dimmer frame, not aerial perspective: ground median **69 → 30**, p99
   **163 → 88**.
3. ~~`capture.rs`'s ground-median floor of 55 is the binding constraint on haze density.~~
   **STRUCK 2026-09-21, same session, by further probing — see section 6. It was not a guard
   bounding an art decision; it was three wrong settings.** At the corrected settings the haze
   reads and the ground median is **69, the control's own value**, with 14 of headroom.

### And the epic's sky clause names the wrong mechanism

`StandardMaterial::fog_enabled` is read **only** inside `#ifdef DISTANCE_FOG` in
`pbr_functions.wgsl:1002`, gating `apply_fog` alone. It has **no effect on volumetric fog**,
which is a separate pass over the `FogVolume`'s geometry. The stars and the aurora already set
`fog_enabled: false` (`atmosphere.rs:270, 279`) and the probe frame dims them anyway. **What
protects the sky is the fog volume's EXTENT and DoF's `max_depth`, not a material flag.**

## 5. What is NOT pinned, and will be this story's noise

`fall_snow` (`atmosphere.rs:321`) moves 96 flakes by `time.delta_secs()` — **wall clock**.
Neither `--lights-steady` nor `--static-world` pins it. The four controls above happen to be
tight because every capture stops at the same frame count from a fresh daemon; a flake field is
still high-contrast structure sitting in exactly the windows this story measures, and it is in
DoF's near field. Re-measure the floor before leaning on any figure.


## 6. The haze, corrected — three findings, and the floor collision dissolves

Same venue, same controls. Each row adds one correction to the row above.

| step | ground median | far ridge median | far ridge `lap_mean` | sky median | stars ≥150 | seam? |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| control, no haze | 69 | 49 | 13.45 | 24 | 78 | — |
| Bevy defaults, `density 0.06` | **30 — PANIC** | — | — | — | — | — |
| + `ambient_color`/`intensity` from night, `density 0.015` | 67 | 74 | 5.60 | 60 | **0** | no |
| + box volume bounded to top y 28 | 67 | 65 | 9.59 | 25 | 78 | **YES** |
| **+ vertical density texture** | **69** | **69** | **9.30** | **25** | **78** | **no** |

**Finding 1 — `ambient_intensity` is relative to Bevy's ambient default, not to this scene's.**
`VolumetricFog::ambient_intensity` defaults to `0.1` to match
`AmbientLight::default().brightness = 80.0` (`bevy_light-0.19.0/src/ambient_light.rs:36`). This
client runs ambient at **1500.0**, so the fog was lit at 1/19th of the scene's own ambient while
its extinction term ran at full strength — `volumetric_fog.wgsl:212` is
`exp(-ray_length * (absorption + scattering)) * ambient_color * ambient_intensity`. Derived value
`0.1 × 1500/80 = 1.875`, with `ambient_color = night_lighting().ambient`.

**Finding 2 — `fog_color` must stay WHITE; the tech-art doc's rule is about the other fog.**
`docs/tech-art-guidelines.md` requires the fog colour to be exactly the sky colour. That is a
`DistanceFog` rule: distance fog BLENDS a surface toward a colour, so matching the sky is right.
Volumetric fog **adds in-scattered light**, so a sky-coloured (5,12,28) fog scatters near-black.
Measured with `fog_color: night_lighting().sky`, the far ridge went **49 → 31** — darker, the
opposite of aerial perspective. At the default white it goes **49 → 69**.

**Finding 3 — a bounded `FogVolume` shows its top face, and every statistic missed it.**
Bounding the box to just above `SKYLINE_MAX = 26.0` gave the best numbers in the table: sky median
25 against a control of 24, all 78 star cores intact, far ridge up 16 with 29 % of its contrast
gone. **And the frame has a hard bright band ruled across the sky**, where the camera sees the
volume's top plane. `probe-haze-d0.015-volume-top28.png` — it is unmistakable by eye and invisible
to every window in this instrument. Raising the box only moves the band up the frame
(`fogd-7602.png`, top y 34).

The fix is `FogVolume::density_texture` — an `R8Unorm` 3D texture, 1 x 64 x 1, sampled as `.r` in
UVW (`volumetric_fog.wgsl:268`), full to v 0.54 and linearly zero by v 0.83, which over a volume
spanning world y -6..42 fades the fog out between y 20 and y 34. The fog then thins into the air
instead of ending at a plane. **`probe-haze-d0.015-density-ramp.png` is that frame**: no seam, the
best far-ridge figures of any variant, the sky and every star untouched, and a ground median
identical to the no-haze control.

**Method note for the record:** findings 2 and 3 were both introduced BY me during this probe —
finding 2 by "helpfully" matching the documented fog colour, finding 3 by acting on a table of
windows without opening the PNG. The second cost a wrong recommendation to Wolf, twice.
