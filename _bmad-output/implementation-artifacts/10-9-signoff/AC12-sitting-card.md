# AC12 — the sitting card for 10.9

**Build:** `b7be859`. Full gate GREEN, 452 s, run independently by the orchestrator
after Codex reported its own — `cargo test` (not the fast set) and the pixel guards both ran.

**The frame still speaks for the branch tip, re-checked 2026-09-11.** `b7be859` is no longer the
tip — the review patches (`665ccdf`) landed after it and touched `worldgen.rs`, `project.rs` and
`ingest.rs`. None of them can move the look: the `worldgen` edit drops a parameter that was already
unused (`_rng`), `project.rs` extracts `derived_triangle_count` from an inline expression, and the
`ingest.rs` change is a test. Re-measured on a fresh build at `9cea5b0`: `entities=1702`,
`faces=830908`, `triangles=94442`, all three bit-identical to the tip column below. Not asserted by
pixel diff — the snowfall animates, so a frame-to-frame diff here has no noise floor (see #90).

**The frame:** `boot-b7be859-subdiv4.png` — boot framing, `--subdiv 4`, `--frames 160`, 1280x720.
Fog and rim are UNCHANGED (10.8 Ruling 3 defect (d) stays deferred), so the ridge lever is being
judged alone, as AC12 requires.
**Before:** `boot-baseline-5133a86-subdiv4.png` is the same framing before the story.

## What the numbers say

| | baseline `5133a86` | tip `b7be859` | AC2 bound |
|---|---|---|---|
| `triangles=` | 927,622 | **94,442** | `<= 231,905` and `> 60,000` — both met |
| `faces=` | 1,155,694 | 830,908 | (deliberately not an AC) |
| `mesh_build_ms=` | 2,516 | 1,715 | — |
| entities | 2,164 | 1,702 | — |

`--subdiv 1` still renders (AC11): 55,167 entities, `triangles_derived=522,304`.

**One number the story did not carry, added at review.** `near-white-area` across five runs of one
build: 0.8181 / 0.6398 / 0.6917 / 0.7868 / 0.4890 % — mean ~0.685 %, spread **0.329 pp**, against a
0.946072 % guard ceiling that budgets a 0.117 pp swing. `triangles=` was identical on every run, so
that spread is the animated snowfall, not the land. Two consequences for this sitting: the frame you
are judging is one draw from that spread rather than a fixed look, and the pixel guard can go red on
a gate run where nothing has changed.

## Where to look

- **The lake (AC9).** World ellipse centred `(28, 92)`, 181 contiguous `Ice` surface cells all at
  `z=17`, treeless. It projects to screen **~(621, 253)** — upper-left of centre. Cropped 4x in
  `lake-b7be859-detail.png`.
  **It is NOT the only ice in the world, and it is not the biggest.** Corrected at review: the
  world holds 2,085 ice surface cells and **eleven** connected regions of >= 40 cells. The lake is
  the FIFTH largest; the biggest is 247 cells at `x[80,95] y[0,15]`. The other ten are AC5 scour
  lattice (below). Anything ice-coloured that is not at upper-left of centre is scour, not lake.
- **The ridges (AC6).** Along `x=0` (upper LEFT) and `y=127` (upper RIGHT), the two edges AC1
  found to be the far pair. A 6-cell band raised 4 levels.
- **The scour patches (AC5).** Ice on sloped ground, in 16-cell blocks — **measured at review as
  the loudest thing in the frame after the terrain itself**, so look here first. They are
  reinforced THREE ways, not one: ice takes a darker, bluer albedo (104,128,170 against snow-cap
  146,158,184), is EXCLUDED from `has_snow_cap` so it misses the brighter cap its neighbours get,
  AND takes detail depth 0 so it renders dead flat beside drifted snow. Sampled plate at screen
  (250,540) reads (78,104,157), R/B 0.497, against adjacent snow at (430,600) (99,120,165), R/B
  0.60. Visible bottom-left and left of camp.

## Which ice is which — every region placed on the frame

Asked by Wolf at the sitting, 2026-09-11: *what are the same-coloured blocks as the lake, in the
big hill and elsewhere?* **They are ice, the same material the lake is made of — AC5 scour, not
lake.** The material rule gives ice ONE appearance, so a scour plate and the frozen lake are the
same pixels by construction. Marked on `ice-regions-b7be859-annotated.png` (green = lake, red =
scour); each box is the projected CENTROID cell of a region, not its outline.

Every ice region of >= 40 cells, projected through the capture's own oracle
(`CameraRig::project_world_point_with_depth`, camp `(64,64,9)` -> `(640.0, 561.2)` reproduced as
the control):

| rank | cells | world | heights | screen | where |
|---|---|---|---|---|---|
| 1 | 247 | `x[80,95] y[0,15]` | z 5–19 | `(-1693, 3236)` | off-screen, near-left |
| 2 | 237 | `x[64,79] y[96,111]` | z 16–27 | `(1071, 284)` | **the big hill, upper right** |
| 3 | 225 | `x[48,63] y[112,127]` | z 13–20 | `(978, 272)` | **the big hill, upper right** |
| 4 | 217 | `x[80,95] y[80,95]` | z 11–19 | `(1176, 515)` | right flank, below the hill |
| **5** | **213** | **`x[19,37] y[86,99]`** | **z 17 flat** | **`(624, 243)`** | **THE LAKE** |
| 6 | 166 | `x[96,111] y[68,79]` | z 18–24 | `(1483, 701)` | off-screen right |
| 7 | 154 | `x[0,15] y[80,95]` | z 16–23 | `(476, 203)` | upper left, on the skyline |
| 8 | 119 | `x[0,15] y[0,15]` | z 13–19 | `(-414, 421)` | off-screen left |
| 9 | 64 | `x[48,63] y[37,42]` | z 15–17 | `(201, 576)` | bottom left |
| 10 | 53 | `x[48,63] y[44,47]` | z 13–15 | `(313, 560)` | bottom left |
| 11 | 53 | `x[64,76] y[16,21]` | z 20–23 | `(-382, 934)` | off-screen, below-left |

Re-derived independently of the review, from a live snapshot at `9cea5b0` (`export_world.py`,
topmost non-tree tile per column, 4-neighbour components): 2,085 ice surface cells, 954 `Solid` +
1,131 `Ramp`, 28 regions of which 11 are >= 40 cells. The review's figures reproduce exactly.

**What separates the lake from the rest, in the data and not by eye.** The lake is the only region
that is FLAT (all 213 cells at `z=17`; every other region spans 3 to 12 height levels) and the only
one the lattice did not make — its cells sit at `(x/16 + y/16) % 5 ∈ {1,2,3}`, while all ten others
are `% 5 == 0`, the scour rule's own residue. So the biome pilot's signature is *flatness*, and
nothing about its material.

**The consequence for ruling 3 below.** Six scour regions are in frame and **three of them are
BIGGER than the lake** (237, 225, 217 cells against 213) — the lake is the fifth largest in the
world and the fourth largest in shot. The frozen lake AC9 introduced as a biome pilot is not
visually distinguishable from a by-product of the scour lattice; it is one of seven identical-
looking ice patches, and not the largest.

## Three things the orchestrator will not sign off, for Wolf to rule on

1. **AC1 is UNMET AS WRITTEN and no framing change was made to chase it.** Only `x=0` and
   `y=127` are in frame; `x=127` and `y=0` are off-screen and partly BEHIND the camera. They are
   identified by projection in `AC1-edge-mapping.md` instead. Changing the boot framing to get
   all four in shot would invalidate every look judgement since 10.7, so it was not done.
2. **AC9's letter is met but the lake does not READ as ice.** It is flat, treeless and
   contiguous — structurally a lake — but at this lighting its material reads as a flat snow
   clearing rather than as ice. Whether that is a frozen lake or just a plain is a look call.
3. **AC5's rule is a REGULAR PATTERN and may show.** `surface_material` scours snow where
   `gradient > 0 && (x / 16 + y / 16) % 5 == 0` (`worldgen.rs`). That is diagonal stripes of
   16-cell blocks on a fixed lattice, not a natural process. It passes its test; the question is
   whether the repeat is visible at this framing.
   **Quantified at review, and it is bigger than "may show" suggested.** Only **19.0 % of sloped
   columns (1,872 of 9,843)** are scoured, so 81 % of sloped ground is materially identical to
   flat ground — the rule reads as a lattice laid OVER the terrain rather than as a response to
   it. The lattice is what produces the ten non-lake ice regions above, four of them LARGER than
   the lake. Also a semantic oddity worth a ruling: "scoured" ground emits **ice**, though Task
   2's own text calls it "exposed rock", and the rock relief branch never executes at boot
   (stone 0 / soil 0 drawn top faces) — so nothing in the shipped frame reads as exposed rock.

## And one for the AC2 bound itself

The story says plainly that 25 % of baseline "is a judgement, not a measurement", made before
there was a frame to look at. The tip sits at **94,442 against a 231,905 ceiling** — roughly
2.5x of unspent headroom. If the land wants more detail, the budget is there.
