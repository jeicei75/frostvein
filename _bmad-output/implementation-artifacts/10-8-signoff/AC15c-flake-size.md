# AC15 (c) — flake size, a ladder with a measured visibility floor

Ruling 3 defect (c): *"maybe flakes could be smaller"*. Wolf's direction, 2026-09-09:
**"flake size at least half .. dwarves are small"**, with his own bound on it:
**"but if we make too small flakes then those will disappear completely"**.

That bound is what this ladder is for. A scale ladder without a visibility floor is the lantern
mistake again, where a sixth turned out to be a speck that was largely the emissive face.

Built at `cc54b5e`, one daemon, `--headless --frames 160`, 1280x720, four builds plus a
`SNOWFLAKE_COUNT = 0` baseline. Read with `flakes.py`.

## The figures

Flakes counted as bright blobs in the sky band that are NOT bright in the no-flake frame.

| candidate | `snowflake_scale` | flakes visible | flake px | largest | median | 1-px specks |
|---|---|---:|---:|---:|---:|---:|
| NOISE (no flakes twice) | — | **0** | 0 | 0 | 0 | 0 |
| current | `0.3 + 0.18` | 18 | 523 | 68 | **24** | 4 |
| **half** | `0.15 + 0.09` | 15 | 154 | 18 | **11** | 1 |
| third | `0.10 + 0.06` | 14 | 94 | 11 | **6** | 0 |
| quarter | `0.075 + 0.045` | 14 | 57 | 7 | **4** | 0 |

## What it says

**The flakes do not disappear.** The COUNT barely moves — 18, 15, 14, 14 — down to a quarter. What
changes is their SIZE: a median blob of 24 px becomes 11, then 6, then 4. Even at a quarter the
median flake is a 2x2 block and the largest is 7 px, which is a speck but not a sub-pixel one.
The four 1-px specks are at the CURRENT size and are the distant flakes; they thin out rather than
multiply as the scale drops.

So the fear is not confirmed on this evidence, and **half is comfortably safe**: 15 of 18 flakes
still resolve, at an 11-px median.

**THE VANISHING POINT MUST BE CONFIRMED ON THE VEHICLE, NOT HERE.** These are lavapipe frames, and
small bright features are precisely the class where this venue and the delivery GPU are already
KNOWN to disagree — the near-white bright tail reads +0.4 to +0.6 pp worse on the RTX 4080 (see the
story's 2026-09-09 venue entry). A flake is a small bright feature. Whether a quarter-scale flake
survives real MSAA and filtering is a vehicle question; the devpod can only say it survives here.

## Two instruments were wrong before this one, and both failures are the point

1. **A pixel diff against the no-flake baseline does not discriminate.** Under `--frames N` the
   capture fires on a FRAME count, so every run photographs a different sim tick: the dwarves have
   walked and their lantern pools have moved. Two NO-FLAKE frames differ by **19,212 px in 766
   blobs, one of them 6,575 px** — noise the size of the signal, with the two smallest candidates
   reading BELOW it. Had the first table been published it would have said the small flakes vanish,
   which is the opposite of what the corrected instrument measures.
2. **`--at-tick` would freeze the scene but cannot be reached.** Its runs end on the frame budget
   long before the tick arrives — 2 of 20 delivered at `--frames 20000`, 29 of 120. Related to #84.

The working instrument needs neither: it counts flakes directly as bright blobs on a dark sky,
inside a band that excludes the dwarves, and subtracts the stars. **The noise pair reads exactly
0 flakes and 0 px**, which is the discrimination check the first instrument failed.

## If a value is chosen

`crates/gui/src/atmosphere.rs:214` is the knob. `atmosphere.rs:625` pins
`scales >= 0.28 && <= 0.5` and must be **corrected to the ruled value, not deleted or loosened** —
the same rule AC14's flank pin was held to.
