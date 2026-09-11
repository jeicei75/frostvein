# AC1 — which world edge occupies which screen region at the boot framing

**Build:** `5133a86` (story branch tip, clean tree — verified by re-running the control to
`triangles=927622` after the probes below were reverted).
**Frame:** `boot-baseline-5133a86-subdiv4.png`, 1280x720.

```bash
./target/debug/simd &
./target/debug/gui --headless --static-world --subdiv 4 --frames 160 \
  --capture 10-9-signoff/boot-baseline-5133a86-subdiv4.png
# subdiv 4: ... faces=1155694 triangles=927622 mesh_build_ms=2575
# capture range check: warm-lit pixels=33899 ground-median-luminance=82
#   near-white-area=0.4542% blown-pool=0.2776% p99-luminance=173.0 resolution=1280x720
```

**The story's Task 1 capture command does not work as written.** `--frames 2` is the
triangle-count recipe; a capture at frame 2 has not rendered yet and the run dies on
`capture is black` (`capture.rs:1419`). Captures need `--frames 160`, the recipe every 10.8
signoff frame used. The `triangles=` figure is identical either way.

## Instrument

Not the pixel diff the story proposed, and not `--cursor`. Both were tried and both fail here:

- **`--cursor` cannot work under `--headless`.** `PickedTile` is written from
  `window.cursor_position()` (`pick.rs:70`) and `apply_scripted_input` writes that position only
  if a `PrimaryWindow` resolves (`ingest.rs:1012`). Headless has no window, so the live pick is
  always `None`, `expected` is `None` too, and the run dies on the `picked.is_some()` assertion
  at `capture.rs:960`: `pick: cursor=(640,700) no tile picked`. Confirmed at four screen points.
- **A raise-a-band pixel diff has no usable noise floor at this framing.** Snowfall and stars
  are animated, so two captures of the *same* build differ frame-wide. A 4-cell band at `x=0`
  and a 12-cell band at `y=0` produced 15,830 and 24,574 changed pixels smeared across all
  quadrants — the bands were not separable from the animation.

**What was used instead:** `CameraRig::project_world_point_with_depth` (`camera.rs:110`) — the
capture's own projection oracle. It is deterministic, needs no window, and carries its aspect
ratio as a constant (`BOOT_ASPECT_RATIO`, `camera.rs:30`), so normalized coordinates times
`(1280, 720)` are screen pixels directly. The creation-time note that a projection probe returned
"±2 px deltas for 128-cell distances — degenerate without a real viewport" does not reproduce;
that probe was reading a default-sized window, not this function.

**The oracle was verified against the picture before it was believed:** `camp_origin` at
`[64, 64, 9]` projects to screen `(640.0, 561.2)`, and the campfire in the committed frame sits
exactly there.

## The mapping

| World edge | Depth | Where it is on screen |
|---|---|---|
| **`x=0`** | 95–141 | **FAR — upper LEFT.** On screen from `y≈40` (left margin) running up-right to the frame's top-centre. `(0,48,9)→(150,333)`, `(0,64,9)→(296,298)`, `(0,96,9)→(521,244)`, `(0,127,9)→(683,205)`. Off-screen left below `y≈40`. |
| **`y=127`** | 86–141 | **FAR — upper RIGHT.** On screen from the same top-centre corner running down-right to the right margin at `x≈85`. `(0,127,9)→(683,205)`, `(32,127,9)→(842,255)`, `(64,127,9)→(1074,326)`, `(80,127,9)→(1234,376)`. Off-screen right beyond `x≈88`. |
| **`x=127`** | 12–53 | **NEAR — entirely OFF SCREEN**, low and to the right. `(127,64,9)→(2755,2177)`; `(127,0,·)` and `(127,32,·)` are *behind the camera*. |
| **`y=0`** | 1–67 | **NEAR — entirely OFF SCREEN**, low and to the left. `(64,0,9)→(-1214,1566)`; `(127,0,·)` is behind the camera. |

The two far edges meet at world corner **`(0,127)`**, screen **`(683, 205)`** — the apex of the
terrain silhouette at the top-centre of the frame. Sampling at `z=20` instead of `z=9` moves each
sample up by 60–70 px and changes no edge's classification.

## What this settles for the rest of the story

- **AC6's ridges go on `x=0` and `y=127`** — the derivation in the story's Task 1 note is
  CONFIRMED, and so is the pinned compass (`north_on_screen` = "down-left": north is world `-y`,
  so `+y` is up-right and `x=0` is up-left).
- **AC1 cannot be met as literally written.** Only two of the four world edges are in frame at the
  boot framing; `x=127` and `y=0` are off-screen and partly behind the camera. They are identified
  here by projection, not "in the image". No framing change was made to chase them — the boot
  framing is what every look judgement since 10.7 has been signed off against.
