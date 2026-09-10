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

## Where to look

- **The lake (AC9).** World ellipse centred `(28, 92)`, 181 contiguous `Ice` surface cells all at
  `z=17`, treeless. It projects to screen **~(621, 253)** — upper-left of centre. Cropped 4x in
  `lake-b7be859-detail.png`.
- **The ridges (AC6).** Along `x=0` (upper LEFT) and `y=127` (upper RIGHT), the two edges AC1
  found to be the far pair. A 6-cell band raised 4 levels.
- **The scour patches (AC5).** Ice on sloped ground, in 16-cell blocks.

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

## And one for the AC2 bound itself

The story says plainly that 25 % of baseline "is a judgement, not a measurement", made before
there was a frame to look at. The tip sits at **94,442 against a 231,905 ceiling** — roughly
2.5x of unspent headroom. If the land wants more detail, the budget is there.
