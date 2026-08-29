# Headless bench comparison

**The question for Wolf:** does `bench-valley.png` *predict* what `gui-capture.png` shows? Not
"are they identical" — a path tracer and a rasterizer never will be. The bar is fidelity of
terrain silhouette, boot framing, palette, snow caps, spruce crowns, and the camp's warm pool.

Compare in this order: the framing (where the camp sits, where the skyline cuts), then the
terrain silhouette, then the palette and the snow caps, then the campfire's warm pool.

## Every known difference, so you are not asked to rediscover them

- **Different renderers.** The bench is Cycles, a CPU path tracer; the client is Bevy/wgpu, a
  rasterizer. Shadow softness and light falloff differ by construction.
- **Different resolutions**, same 16:9 aspect: the client frame is 1280x720, the bench 960x540.
- **The bench has no aurora, no stars, no distance fog and no `rim_level` edge treatment.** Each
  is bespoke client geometry, deliberately out of scope here. The client's green aurora band and
  its star field have no counterpart in the bench, and that is the largest visual difference
  between the two images.
- **Different ticks.** The export and the capture landed at different ticks, so dwarf positions
  differ. Terrain does not.
- **The client draws a thin extra top layer.** Its boot draw set is
  `is_exposed(..) || (z == level && solid)`, so it shows solid cells at the selected top slice
  that the bench, which draws only the exposed set, omits.

## Provenance

- Bench: `blender --background --python scripts/bench/valley_bench.py -- <snapshot.json> <out.png>`
  on the default seed. Reported `exposed_cells=44984`, `non_sky_fraction=0.674020`,
  `distinct_colors=45642`.
- Client: `gui <port> --headless --capture gui-capture.png --at-tick 100 --frames 20` against a
  live `simd`. **This command exits 101 on a devpod** — `capture.rs`'s delivered-tick floor is not
  reachable under llvmpipe, where tick delivery is erratic (measured 26 ticks, then 2, across
  `--frames` values from 20 to 200,000). The PNG is written *before* validation, so the frame is
  genuine and complete; only the integrity assertion fails. Story 9.1 already recorded that every
  `--capture` AC is vehicle-bound on a devpod. The assertion was left alone deliberately: weakening
  a client-side integrity rule to make a bench story's artifact green would ship a workaround for
  an environment limitation.
