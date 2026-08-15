# Story 5.4 sign-off gate artifacts (UX-DR22)

## Opening half

- `candidate-artifact-2026-08-15.png` — **APPROVED BY WOLF 2026-08-15. The gate is
  open; this is the reference the built boot frame is judged against (closing half).**
  Iterated from the parked 08-14 image on Wolf's direction across four passes: draft-2
  framing (camp foreground, sky/aurora the top third, elliptical fog dissolving the far
  valley into the sky instead of a floating diorama), trees as snow-laden spruce sprites
  with visible two-block trunks (his explicit asks: trunk at the bottom, slimmer first
  skirt so the trunk reads).
  - **Tree density is FULL wire truth: all 704 world trees (one per ~23 surface tiles).**
    A 65%-thinned variant was rendered and offered with the caveat that delivering it
    means a sim-core worldgen change re-opening 5.1's pinned terrain fingerprint; Wolf
    chose full density, so **no worldgen change rides on 5.4**. The variant is deleted;
    rerender with `--thin 0.65` if that conversation ever reopens.
  - Second wire-truth caveat: the valley surface is a per-tile snow/ice mix; the mock
    desaturates ice toward snow so it reads as mottling (the Bevy client will blend
    materials). The blue mottle pattern itself is real data.
- `candidate-artifact-2026-08-14.png` — the first candidate. **NOT APPROVED — PARKED
  2026-08-14.** Wolf's reaction on first viewing: *"quite far away from the drafts"* (the
  two concept references in `docs/`). Superseded by the 08-15 iteration; kept as the
  record of what the gate caught. Note this is the gate doing precisely its job — the
  same mismatch class that cost all of 4.1a was caught here at the price of one image.
  (Its framing parameters were replaced in `artifact_render.py`; regenerating it needs
  the pre-08-15 script.)
- **The gate stays closed: no implementation task starts** until Wolf approves an
  artifact.
- Provenance: rendered from a **live `simd` snapshot of the shipped seed** (08-14: tick
  20; 08-15: tick 21 — geometry is identical, only dwarf positions drift) by
  `artifact_render.py` (software isometric mock, kept beside it for reproducibility).
  Recapture + rerun:
  `python3 capture_snapshot.py snapshot.json` (builds nothing; runs `target/debug/simd`)
  `uv run --python 3.14 --with pillow --with numpy python artifact_render.py snapshot.json <out.png> [--thin 0.65]`.
  Geometry, camp, trees and emitters are wire-true: the FULL exposed-tile pass still
  reproduces 5.3's AC13 draw-set oracle exactly (53,365 tiles — printed on every run;
  tree tiles are then excluded from terrain drawing and re-drawn as sprites). The look
  applied is the story's proposed palette: night snow midtone blue-grey, warm pools from
  the 4 torches + campfire at z 9, aurora hugging the horizon behind the 5.1 skyline,
  snow caps, depth fog into the sky, foreground falloff, decorative snowfall.
- It is a **mock, not a Bevy capture** — it sells the look; the built result will differ in
  rendering detail but must land the same read (warm camp first, cold layered field, sky as
  illuminant).

## Closing half

- The approved artifact stays here. The story's closing `gui --capture` boot-frame PNG is
  retained beside it so Wolf's live comparison is two images, not a memory (story AC16/AC19).
