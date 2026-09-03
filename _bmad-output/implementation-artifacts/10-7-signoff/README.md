# 10.7 signoff artifacts — the evidence that the sun is under the map

Captured 2026-09-03 on `e930d07`, headless, **lavapipe software rendering**, `--subdiv 1`, boot
framing, `--frames 160`, 1280x720. Each probe is a one-line source edit, built, captured, and
**reverted** — none of them is in the shipped code.

These frames cannot be regenerated identically: dwarves move between runs, which is exactly why
the noise floor below exists. They are committed rather than left in a scratch directory for that
reason.

| artifact | what was changed | mean luminance | dark (<40) | shade-band (40-89) |
|---|---|---:|---:|---:|
| `control-shipped-a-e930d07.png` | nothing — the shipped build | 87.894 | 161,492 | 223,502 |
| `control-shipped-b-e930d07.png` | nothing — **second run, THE NOISE FLOOR** | 87.973 | 161,495 | 223,412 |
| `probe-shadow-maps-disabled.png` | `shadow_maps_enabled: false` | 87.906 | 161,493 | 223,343 |
| `probe-cascade-max-500-FALSIFIED.png` | `CascadeShadowConfig` max distance 150 -> 500 | 87.865 | 161,489 | 223,492 |
| `probe-sun-illuminance-0.png` | `illuminance: 0.0` — the sun deleted | 87.815 | 161,489 | 223,560 |
| **`probe-sun-lifted-y200.png`** | **`aurora_core().with_y(200.0)`** | **101.188** | **160,432** | **198,034** |

**How to read it.** Two runs of the same build differ by **0.08 mean and 3 dark pixels in
921,600** — that is the noise. Rows 3, 4 and 5 all sit inside it: disabling shadows, extending the
cascade distance, and **deleting the sun entirely** each change the frame less than simply running
the same binary twice. Row 6 moves the mean by **13.3, about 170x noise**, and empties **25,468**
pixels out of the shade band.

The sun works. It is 67.5 units under the world, shining upward — `aurora_light_transform()` puts
it at the aurora curtain's midpoint, `(-162 + 45) / 2 = -58.5`, while the surface is at `9.0`.

**`probe-cascade-max-500-FALSIFIED.png` is kept deliberately.** Bevy's never-set
`CascadeShadowConfig { maximum_distance: 150.0 }` — 150 *cells* at this project's one-render-unit-
per-cell scale — was the first and very persuasive explanation. This frame is the measurement that
killed it. Keeping the artifact is cheaper than someone re-deriving the same wrong answer.

## The instruments

- **`lumstats.py`** — the luminance distribution. `python3 lumstats.py <file>=<label> ...`
  **This is the instrument that made the finding possible.** Use it, not a pixel diff.
- **`pixel_diff.py`** — per-pixel max-channel delta. `python3 pixel_diff.py <a> <b> <label>`.
  Kept as the *counter*-example: with dwarves moving it has a **38,989-pixel noise floor at
  delta>=4**, larger than the entire signal here, and it reported the sun probes as
  indistinguishable. When a diff is swamped, change the statistic, not the sample size.

Both are stdlib-only PNG decoders — no numpy, matching this repo's existing bench convention.
