# AC12 — the sitting card for 10.9

**Build:** `b7be859` (branch tip). Full gate GREEN, 452 s, run independently by the orchestrator
after Codex reported its own — `cargo test` (not the fast set) and the pixel guards both ran.

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
