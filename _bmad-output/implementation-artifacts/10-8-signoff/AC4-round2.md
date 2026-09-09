# AC4 round 2 — Wolf's refinements, and the knob that was never varied

**Executed 2026-09-08** on branch commit `4d45ca9`, shipped `k = 4` default, boot framing,
`--frames 160`, no `--z`. All edits **uncommitted and reverted**; the tree was verified clean after
every candidate. Nothing has landed.

Wolf's direction after round 1 (verbatim): *"what comes to decisions C but we could take campfire
and torches down still .. lanterns I need to see separately.. ambient feeling should be still bit
more darker except .. campfire + torches area is way too overblown still or maybe distance how far
light goes is much too long for those"* — and then *"maybe you could distribute torches more far
away from campfire to see them easier together but separated"*.

That last sentence named the lever that turned out to matter most.

## What was tested

All three candidates take C's colours and drop the ambient from 2,400 to **1,500** (Wolf's "bit
more darker"). They differ only in how the camp is brought down.

| | torch | campfire | lantern | torch reach | campfire reach | lantern reach |
|---|---:|---:|---:|---:|---:|---:|
| shipped | 14.0M | 25.0M | 5.0M | 20 | 28 | 14 |
| C (round 1) | 9.0M | 18.0M | 3.5M | 20 | 28 | 14 |
| **D** dimmer | 5.5M | 11.0M | 2.5M | 20 | 28 | 14 |
| **E** nearer | 9.0M | 18.0M | 3.5M | **12** | **17** | **9** |
| **F** both | 7.0M | 14.0M | 3.0M | **14** | **20** | **10** |

Every campfire peak stays under the `APPROVED_PEAK` of 35.52M that Wolf ruled at 6.2.

## Figures

| | run | warm-lit | ground median | near-white | blown pool | p99 | mean | exit |
|---|---|---:|---:|---:|---:|---:|---:|---|
| **D** | a | 31,206 | 79 | 1.1780 % | 0.7964 % | 209.1 | 65.510 | **0** |
| **D** | b | 32,187 | 79 | 1.1706 % | 0.7881 % | 209.1 | 65.510 | **0** |
| **D** | lanterns off | 33,037 | 79 | 0.8901 % | 0.6272 % | 194.3 | 65.098 | 0 |
| **E** | a | 21,010 | **68** | 1.2853 % | 0.9296 % | 216.8 | 64.177 | 101 |
| **E** | b | 22,164 | **68** | 1.2324 % | 0.8639 % | 214.0 | 64.147 | 101 |
| **E** | lanterns off | 23,776 | 68 | 1.1425 % | 0.8162 % | 208.3 | 64.084 | 101 |
| **F** | a | 24,447 | **69** | 1.2759 % | 0.8882 % | 215.5 | 64.628 | 101 |
| **F** | b | 24,836 | **69** | 1.1734 % | 0.8438 % | 210.7 | 64.482 | 101 |
| **F** | lanterns off | 26,282 | 69 | 1.0412 % | 0.7279 % | 202.3 | 64.316 | 101 |

**D is the first candidate to exit 0 on two consecutive runs**, satisfying AC5 against the shipped
ceiling without it even being re-derived.

**E and F do NOT fail on near-white — they fail on the FLOOR.** Both sit comfortably under the
1.5630 % ceiling. What they breach is `GROUND_LUMINANCE_FLOOR = 70`, the guard that says a frame is
*"a black field, not a lit night"* (`capture.rs:1404`): E reads 68 and F reads 69. **Pulling the
emitters' reach in starves the valley floor faster than it tames the camp**, because those lights
were lighting far more ground than their role suggested. Reach is a real and separate knob, and on
its own it is the wrong one.

**Lanterns, seen separately** (Wolf's second ask). At D's levels, switching the lanterns off takes
near-white from 1.1780 % to 0.8901 % — **a quarter of the entire near-white budget from the
smallest emitter.** At the shipped table the same switch-off was worth 3× noise on frame mean and
looked like a rounding error (`AC3-marginals.md`). As the camp comes down, the lanterns stop being
negligible. They are worth a deliberate decision rather than being carried along.

## The finding: spreading the torches beats dimming them

`camp_emitters` (`crates/sim-core/src/lib.rs:1614`) places the four torches at **±2 cells
diagonally** from the campfire. At 1.6 m per cell that is **4.53 m**, sitting inside a campfire
whose light reaches 28 m. They cannot read as separate sources at any intensity — which is exactly
why "campfire + torches area" reads as one overblown pool.

Probed at F's lighting held constant, moving ONLY the spacing:

| torches at | distance | warm-lit | ground median | near-white | blown pool | p99 | mean | exit |
|---|---|---:|---:|---:|---:|---:|---:|---|
| ±2 cells (ships today) | 4.5 m | 23,954 | **69** | 1.2713 % | 0.8993 % | 215.1 | 64.593 | 101 |
| **±5 cells** | 11.3 m | 33,906 | **77** | 1.0727 % | 0.6108 % | 203.0 | 65.310 | **0** |
| **±8 cells** | 18.1 m | 32,249 | **82** | 0.7235 % | 0.4069 % | 185.6 | 65.266 | **0** |

**Every other lever traded the peak against the floor. This one moves both the right way.**
Dimming (D) lowered the blown highlights and lowered the ground with them. Shortening the reach
(E, F) lowered the highlights and starved the ground below its floor. Spreading the torches
**drops near-white from 1.2713 % to 0.7235 % while RAISING the ground median from 69 to 82** — and
it takes F's lighting from exit 101 to exit 0 without changing a single lighting constant.

The mechanism is straightforward once the geometry is visible: four torches stacked on top of the
fire were adding their pools together into one saturated core while contributing almost nothing to
the surrounding floor. Spread them and the same total light covers the camp instead of piling up
in one place.

By eye: at ±5 the torches read as distinct warm patches at the camp's edge with the fire still the
bright core; at ±8 they are clearly four separate lights around a lit clearing, and the terraced
dig reads as a place rather than a glare. Frames `probe-torch-spacing-{2,5,8}cells-4d45ca9.png`.

## This probe is out of 10.8's scope as written, and that is Wolf's call

The spacing lives in **`sim-core`**, not the client. Story 10.8's guardrails say *"Do not touch
camera or composition"*, and the story's Project-structure table names no simulation file. Making
it permanent would also re-pin `generated_world_has_sorted_camp_emitters`
(`sim-core/src/lib.rs:1706`), which asserts the four offsets exactly.

It is a small change — one function and one test — but it moves the story into the simulation and
changes what every future capture of the camp looks like, including the figures already taken in
`AC3-marginals.md` and round 1. **Recommendation: if Wolf wants it, add it to 10.8 as an explicit
acceptance criterion rather than letting it in quietly, and re-take the marginals after it lands.**
If he would rather keep 10.8 to the client, it belongs in its own small story and the lighting is
then chosen against today's camp.
