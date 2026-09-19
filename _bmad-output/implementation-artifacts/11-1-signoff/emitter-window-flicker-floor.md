# The emitter window's flicker floor — measured at 11.1b's creation, build `41b3f02`, 2026-09-18

Story 11.1b has to prove that bloom brightens emitters and their halo. The instrument 11.1a built
(`creases.py`) **deliberately excludes the camp**, and the camp is where every emitter is. So before
writing 11.1b's bloom criterion, the camp window was measured to find out what it costs.

## Method

The four committed same-build controls `control-41b3f02-{a,b,c,d}.png` (boot framing,
`--headless --static-world --subdiv 4 --frames 160`, all exit 0), over the camp window
`(500, 400)..(760, 620)` — the campfire, torches, lanterns and their pools. Rec.601 integer luma,
the same statistic as `creases.py` and `pixel_guard.rs:35-41`.

## Result

| statistic | ctl-a | ctl-b | ctl-c | ctl-d | spread |
| --- | ---: | ---: | ---: | ---: | ---: |
| median | 83 | 82 | 84 | 82 | **2** |
| p90 | 212 | 194 | 204 | 203 | **18** |
| p99 | 251 | 248 | 250 | 250 | **3** |
| mean | 114.959 | 110.421 | 113.535 | 112.597 | **4.538** |
| near-white (≥230) area | 5.6731 % | 4.2308 % | 4.5804 % | 4.8899 % | **1.4423 pp** |

Compare the non-camp windows on the same frames, where `p10` and `median` have a spread of
**exactly 0**.

## Why it matters, and it is not the sim

**Bloom acts on the bright tail, and the bright tail is the part that moves.** `p90` swings 18
levels and near-white area swings **1.4423 pp** between two runs of an unchanged binary. For scale,
the whole frame's entire headroom to `NEAR_WHITE_AREA_CEILING` is 0.13 pp.

The cause is the light flicker, and `--static-world` does **not** stop it: `flicker_lights` is
called with `time.elapsed_secs()` (`crates/gui/src/ingest.rs:1890`), Bevy's wall clock, not sim
time. Pausing the world pauses the dwarves and leaves the campfire flickering. The frames therefore
sample different flicker phases, and `flicker_scale` (`appearance.rs:105-112`) puts 30–40 % of a
torch's or campfire's intensity into that term.

**Consequence for 11.1b:** a bloom figure taken at the camp without pinning the phase carries a
1.4423 pp floor, which is the same shape as the defect issue #90 records and the one 10.4's AC5
published through — 36,176 changed pixels reported as proof while the same-build noise was 38,087.
So 11.1b's Task 1 pins the flicker phase (`--lights-steady`) and proves the spread collapses
**before** Task 3 measures bloom through the same window.

The median (spread 2) and p99 (spread 3) are comparatively steady; they are also the statistics
bloom moves least. The discriminating statistic and the noisy statistic are the same one, which is
why this needed fixing rather than working around.
