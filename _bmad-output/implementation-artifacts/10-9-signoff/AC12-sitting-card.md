# AC12 — the sitting card for 10.9

**Build:** `df0f722`. Full gate GREEN, **505 s**, run in the foreground at
`RUST_TEST_THREADS=6` — `cargo test` (the full arm, 80 s) and the pixel guards (383 s) both ran.

**The frame:** `boot-df0f722-subdiv4.png` — boot framing, `--subdiv 4`, `--frames 160`, 1280x720.
Fog and rim are UNCHANGED (10.8 Ruling 3 defect (d) stays deferred), so the ridge lever is being
judged alone, as AC12 requires.
**Before the story:** `boot-baseline-5133a86-subdiv4.png` is the same framing on `main`.

**This card has been through two sittings, and both frames are kept.**

- `boot-b7be859-subdiv4.png` is the frame of the FIRST sitting, 2026-09-11, with scoured ground
  still emitting ice. Wolf's verdict on it was that the scour patches read as artificial plates —
  see ruling 3 below. It is superseded by a RULING, not by a defect, and it is what the ice
  columns in the tables below measure. (That frame was itself re-checked against its own tip
  before the sitting: the review patches in `665ccdf` moved `worldgen.rs`, `project.rs` and
  `ingest.rs` after it was captured, and a rebuild at `9cea5b0` reproduced `entities=1702`,
  `faces=830908`, `triangles=94442` bit-identical, so it spoke for the tip it was judged as.)
- `boot-df0f722-subdiv4.png` is this frame, with scour emitting stone. Nothing else moved: the
  heights, ramps, camp, trees, dwarves and the lake are bit-identical, and the world export proves
  it — the ten scour regions occupy the same cells at the same heights, relabelled.

## What the numbers say

| | baseline `5133a86` | first sitting `b7be859` | **tip `df0f722`** | AC2 bound |
|---|---|---|---|---|
| `triangles=` | 927,622 | 94,442 | **123,644** | `<= 231,905` and `> 60,000` — both met |
| `faces=` | 1,155,694 | 830,908 | 838,770 | (deliberately not an AC) |
| `mesh_build_ms=` | 2,516 | 1,715 | 1,773 | — |
| entities | 2,164 | 1,702 | 1,768 | — |
| `ground-median-luminance=` | — | 82 | 81 | (not an AC) |

The tip's +31 % of triangles over the first sitting is the ROCK RELIEF BRANCH, executing at boot
for the first time: `material_detail_depth` gives `Ice` relief depth 0 and `Stone | Soil` depth 1,
so re-labelling 1,872 scour columns turns flat slabs into relief. It spends a quarter of AC2's
unspent headroom and leaves ~1.9x of it.

`--subdiv 1` still renders (AC11): 55,167 entities, `triangles_derived=522,304`.

**One number the story did not carry, added at review.** `near-white-area` across five runs of one
build: 0.8181 / 0.6398 / 0.6917 / 0.7868 / 0.4890 % — mean ~0.685 %, spread **0.329 pp**, against a
0.946072 % guard ceiling that budgets a 0.117 pp swing. `triangles=` was identical on every run, so
that spread is the animated snowfall, not the land. Two consequences for this sitting: the frame you
are judging is one draw from that spread rather than a fixed look, and the pixel guard can go red on
a gate run where nothing has changed.

## Where to look

- **The lake (AC9).** World ellipse centred `(28, 92)`, 213 contiguous `Ice` surface cells (181
  `Solid` + 32 `Ramp`) all at `z=17`, treeless. It projects to screen **~(624, 243)** — upper-left
  of centre. Cropped 4x in `lake-df0f722-detail.png`.
  **It is now the ONLY ice in the world**, and that is new at this tip. At the first sitting the
  world held 2,085 ice cells in eleven regions and the lake was the FIFTH largest — the scour
  lattice was emitting ice too. With scour on stone the export finds exactly one ice region: 213
  cells, flat, the lake. Anything ice-coloured anywhere else is now a defect, and a test asserts
  it world-wide.
- **The ridges (AC6).** Along `x=0` (upper LEFT) and `y=127` (upper RIGHT), the two edges AC1
  found to be the far pair. A 6-cell band raised 4 levels.
- **The scour patches (AC5), now rock.** Stone on sloped ground, in 16-cell blocks. At the first
  sitting these were ice and were measured as the loudest thing in the frame after the terrain
  itself; Wolf ruled them artificial plates, and stone with the snow cap KEPT is what replaced
  them (ruling 3 below).
  **What the change did, sampled in both frames at the same pixels.** The plate at screen
  (250,540) read (78,104,157) as ice and now reads **(99,120,165) — pixel-identical to the
  adjacent snow** at (430,600). Same story at (201,576): (77,103,156) -> (99,120,164). The lake at
  (624,243) is unmoved at (94,114,159), as ruled.
  **So the patches no longer read as ice, and they do not read as rock either.** Two rules act at
  once: `Stone` takes relief depth 1, which breaks the slab into drifted geometry, and `Stone` is
  NOT excluded from `has_snow_cap`, so every top face with open sky above gets a snow cap and the
  rock albedo (60,70,92) shows only in the gaps a cap cannot reach. What you see in those boxes is
  snow-covered broken ground with dark flecks, not exposed stone. The half of ruling 3 that says
  **nothing in the shipped frame reads as exposed rock is therefore still true** — the lattice's
  regularity is what got fixed, not the semantics.

## Which patch is which — every region placed on the frame

Asked by Wolf at the first sitting, 2026-09-11: *what are the same-coloured blocks as the lake, in
the big hill and elsewhere?* The answer then was **they are ice, the same material the lake is made
of — AC5 scour, not lake**, because the material rule gave ice one appearance and a scour plate and
the frozen lake were the same pixels by construction. That is what his ruling changed.

Two annotated frames, same boxes, same coordinates — each box is a region's projected CENTROID
cell, not its outline:

- `ice-regions-b7be859-annotated.png` — the first sitting. Green = lake, red = scour, all ice.
- `ice-regions-df0f722-annotated.png` — this tip. Green = lake, still ice. Red = the same regions,
  now stone.

The geometry below did not move between the two: the ten scour regions hold the same cells at the
same heights, and only the material changed. Projected through the capture's own oracle
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

**What separated the lake from the rest, in the data and not by eye.** The lake was the only region
that is FLAT (all 213 cells at `z=17`; every other region spans 3 to 12 height levels) and the only
one the lattice did not make — its cells sit at `(x/16 + y/16) % 5 ∈ {1,2,3}`, while all ten others
are `% 5 == 0`, the scour rule's own residue. So the biome pilot's signature was *flatness*, and
nothing about its material.

**That is what made ruling 3 urgent, and the ruling settled it.** Six scour regions are in frame
and three of them were BIGGER than the lake (237, 225, 217 cells against 213) — the lake was the
fifth largest in the world and the fourth largest in shot, so the biome pilot was not visually
distinguishable from a by-product of the lattice. Re-measured at this tip from a fresh export:
**snow 14,299 columns, stone 1,872, ice 213**, and ice forms exactly ONE region. The lake now owns
the material outright, which is the uniqueness AC9 claimed and had to retract at review.

## Three things the orchestrator will not sign off, for Wolf to rule on

**Two of the three were ruled at the first sitting, 2026-09-11. Rulings are recorded verbatim
under each item; only item 1 is still open.**

1. **AC1 is UNMET AS WRITTEN and no framing change was made to chase it.** Only `x=0` and
   `y=127` are in frame; `x=127` and `y=0` are off-screen and partly BEHIND the camera. They are
   identified by projection in `AC1-edge-mapping.md` instead. Changing the boot framing to get
   all four in shot would invalidate every look judgement since 10.7, so it was not done.
2. **AC9's letter is met but the lake does not READ as ice.** It is flat, treeless and
   contiguous — structurally a lake — but at this lighting its material reads as a flat snow
   clearing rather than as ice. Whether that is a frozen lake or just a plain is a look call.
   **RULED 2026-09-11 — Wolf: "keep the lake don't touch it".** Accepted as it stands; the lake's
   code path was not touched, and the sampled lake pixel is unmoved at (94,114,159) between the
   two frames. Item closed, no follow-up filed.
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
   **RULED 2026-09-11 — Wolf: "areas of that ice scour are too flat (looks like artificial plate
   more) and yes maybe it could be stone instead of ice.. keep the lake don't touch it", then,
   shown both variants, "capped stone is probably ok at this point".** Implemented in `df0f722`:
   `surface_material`'s snowfield arm emits `Material::Stone`, and `has_snow_cap` was deliberately
   left alone so the cap stays on the rock — the variant with `Stone` excluded from that rule was
   built, captured and rejected. What this fixed is the FLATNESS: relief depth 1 instead of 0.
   **What it did not fix, and the record should not pretend otherwise:** the lattice's regularity
   is unchanged (still `% 5 == 0` on a fixed 16-cell grid, still 19.0 % of sloped columns), and
   with the cap on, nothing reads as exposed rock — the patches now read as snow-covered broken
   ground. Neither remainder is filed as an issue; both are visible in this card by choice.

## And one for the AC2 bound itself

The story says plainly that 25 % of baseline "is a judgement, not a measurement", made before
there was a frame to look at. The tip sits at **94,442 against a 231,905 ceiling** — roughly
2.5x of unspent headroom. If the land wants more detail, the budget is there.
