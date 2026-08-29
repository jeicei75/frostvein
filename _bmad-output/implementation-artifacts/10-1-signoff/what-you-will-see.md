# Headless bench comparison

**The question for Wolf:** does `bench-valley.png` *predict* what `gui-capture.png` shows? Not
"are they identical" — a path tracer and a rasterizer never will be. The bar is fidelity of
terrain silhouette, boot framing, palette, snow caps, spruce crowns, and the camp's warm pool.

Compare in this order: the framing (where the camp sits, where the skyline cuts), then the
terrain silhouette, then the palette and the snow caps, then the campfire's warm pool.

> **Re-rendered 2026-08-29 after the code review.** The first pair was rendered with no ambient
> light, a key light aimed 122 degrees away from the client's, and full-size cube foliage. All
> three are fixed, so this artifact is materially brighter and its trees read as sparse crowns
> rather than solid slabs. The `gui-capture.png` half is unchanged — the review changed no crate
> behaviour, only a test.

## Every known difference, so you are not asked to rediscover them

Ordered by how much of the frame each one moves.

- **The bench has no aurora and no stars.** Each is bespoke client geometry, deliberately out of
  scope. The client's green aurora band and its star field have no counterpart in the bench, and
  this remains the largest single visual difference between the two images. It also means the
  client's *sky* carries light that the bench's flat sky does not.
- **The camp's warm pool is much larger and more blown out in the client.** The client drives its
  campfire at 25,000,000 lm and its torches at 14,000,000 [`appearance.rs:52-86`]; the bench uses
  Cycles point energies of 1,500 and 750. Only the light *colours* are pinned to the client — the
  intensities are not, because Bevy and Cycles units do not correspond. **Judge the camp's
  position and hue, not the size of its white core.** Deferred as an open calibration question.
- **Different renderers.** The bench is Cycles, a CPU path tracer; the client is Bevy/wgpu, a
  rasterizer. Shadow softness and light falloff differ by construction.
- **The bench frame carries visible sampling grain**, most obvious on flat snow faces. It renders
  at 32 samples with denoising off — Cycles here has no OpenImageDenoise support and hard-fails if
  asked. The client's rasterized surfaces are flat and clean. Grain is the first thing an eye
  reads as "different material"; it is not one.
- **Overall exposure is calibrated, not converted.** The bench's ambient strength and sun energy
  have no unit correspondence with Bevy's `brightness: 4_500` and `22,000` lux, so they are tuned
  against one objective target: mean luma over the bottom 65% of the frame. Client 105.7, bench
  103.6. Expect the same *level*, not the same histogram.
- **The bench has no distance fog and no `rim_level` edge treatment.** Both are client-side and
  out of scope; far terrain in the bench is slightly crisper than in the client.
- **Snow caps are a different object.** The client spawns a separate raised slab,
  `Cuboid::new(1.02, 0.08, 1.02)` at `+Y*0.54` [`project.rs:195, 894-906`], so its caps overhang
  with their own visible edge. The bench recolours the top face of the cube itself. The *rule* for
  which cells get a cap is identical.
- **Ramps are drawn as full cubes.** The bench has no sloped geometry; a `ramp` tile occludes and
  draws exactly like a solid one. The excavated camp ramps therefore read as steps in the bench
  and as slopes in the client.
- **Different ticks.** The export and the capture landed at different ticks, so dwarf positions
  differ. Terrain does not.
- **The client draws a thin extra top layer.** Its boot draw set is
  `is_exposed(..) || (z == level && solid)`, so it shows solid cells at the selected top slice
  that the bench, which draws only the exposed set, omits.
- **Different resolutions**, same 16:9 aspect: the client frame is 1280x720, the bench 960x540.

## What is pinned, and therefore should NOT differ

A test fails if any of these drift apart: the six terrain colours, the snow-cap and snow-laden
foliage colours, the sky, the ambient and directional colours, the three light colours *in their
own arms*, the three entity colours and scales, all five boot-camera constants, the vertical FOV
and aspect ratio, the four aurora constants that aim the key light, and the foliage crown scales.

## Provenance

- Bench: `blender --background --python scripts/bench/valley_bench.py -- <snapshot.json> <out.png>`
  on the default seed. Reported `exposed_cells=44984`, `non_sky_fraction=0.686815`,
  `distinct_colors=58993`, `terrain_luma=106.260`. Cycles internal 1.42 s, whole process 4.34 s,
  and 0 of 2,073,600 RGBA values differ across two runs.
- Client: `gui <port> --headless --capture gui-capture.png --at-tick 100 --frames 20` against a
  live `simd`. **This command exits 101 on a devpod** — `capture.rs`'s delivered-tick floor is not
  reachable under llvmpipe, where tick delivery is erratic (measured 26 ticks, then 2, across
  `--frames` values from 20 to 200,000). The PNG is written *before* validation, so the frame is
  genuine and complete; only the integrity assertion fails. Story 9.1 already recorded that every
  `--capture` AC is vehicle-bound on a devpod. The assertion was left alone deliberately: weakening
  a client-side integrity rule to make a bench story's artifact green would ship a workaround for
  an environment limitation.
