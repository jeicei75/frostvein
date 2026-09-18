# What is left in the camp window after the flicker is pinned — build `b50235b`, 2026-09-18

Task 1 added `--lights-steady` and AC1 still failed. This is the orchestrator's independent
verification of **why**, run after Codex handed back at the story's own stop condition.

**It is not the flicker, and it is not snow. It is the dwarves, and they carry the lanterns.**
Filed as issue **#105**.

## The flicker pin itself works

Four same-build captures with `--lights-steady` (Codex, build `7d13828`), camp window
`(500,400)..(760,620)`, Rec.601 integer luma:

| statistic | un-pinned spread | pinned spread |
| --- | ---: | ---: |
| median | 4 | 1 |
| p90 | 17 | **0** |
| near-white (≥230) area | 1.7658 pp | 0.4125 pp |

`p90` collapsing to **exactly 0** across four captures is the proof: the emitter tail is pinned.
Whatever remains has another cause.

## Naming the residual

`residual.py` clusters the pixels that differ between two pinned captures; `delta.py` measures how
far they move; `markdiff.py` paints them back onto the frame so the feature can be named by looking
at it rather than inferred from a count.

Pairwise, `task-1-7d13828-a` vs `-b`, camp window:

| measurement | value |
| --- | ---: |
| camp pixels changed | 19,581 of 57,200 (**34.23 %**) |
| changed pixels moving > 9 luma levels | 25.17 % |
| signed moves ≤ −3 / ≥ +3 | 5,299 / 5,608 |
| largest contiguous blob | 12,884 px @ (623, 545) |

Balanced large moves over a third of the window is the signature of something **moving**, not of a
light dimming. A uniform intensity change would be one-signed.

`residual-b50235b-camp-changed-pixels.png` paints those pixels red: the **entire warm-lit pool**
changed and the saturated core did not — a light that moved.

`residual-b50235b-dwarf-region-steady-{a,b}.png` crop the same 80×70 region from both captures at
6×. The dwarves are in visibly different cells, and the bright pool follows them.

## Why the world was moving at all

`--static-world` pauses nothing. It is parsed at `crates/gui/src/ingest.rs:933` and its only use is
`ingest.rs:564` → `.with_static_world(...)`, a flag read only in `capture.rs` to **skip the motion
assertions**. No pause command, no tick-rate change, no effect on `blend_projection`.

The gui's own instrument says so in the same run that carries the flag:

```
lantern: dwarf positions observed={8 distinct cells} ... moved=true
motion:  ticks observed=36 dwarf position changes=36 mid-blend frames=21
```

`blend_entities` (`project.rs:2096`) advances drawn positions from the wall clock every frame, and
lights ride on projected entities (`project.rs:1716`, `:1770`). A moving dwarf moves a light.

## What this costs the story

- **AC1 is unsatisfiable as written.** The floor it must beat is set by dwarf motion, which
  `--lights-steady` is not meant to touch and `--static-world` does not stop.
- **AC4 (bloom) inherits the same problem**, because it is measured through the same window.
- **AC2 and AC3 (ambient occlusion, the MSAA guard) are unaffected.** They are measured on the
  crease windows, which exclude the camp and measured a spread of **exactly 0** on this build with
  the dwarves moving throughout (Task 0: terrace 31/65, LL 68/117, LR 69/117).

The prior attribution was not baseless — pinning the flicker did move near-white from 1.7658 pp to
0.4125 pp. It was incomplete, and the part it missed is the larger one.

## Probe that did not settle it

`--at-tick N` against a freshly started daemon should replay the sim deterministically from the
seed. Four attempts at `--at-tick 100 --frames 600` reached only ~35 ticks before the frame budget
ran out; at `--at-tick 25 --frames 900` the capture reported 2 delivered ticks and panicked. The
frame budget and tick delivery do not line up in an obvious way, and the client-side walk is still
wall-clock paced even at a fixed tick. **Unproven, not disproven** — it needs its own measurement
before anything is built on it.
