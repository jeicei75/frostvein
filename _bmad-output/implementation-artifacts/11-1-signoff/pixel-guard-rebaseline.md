# 11.1a pixel-guard re-baseline — `c3735d1`, 2026-09-18

Both real rendered guards ran after the FXAA/MSAA render-path change, with their output enabled.

| guard | measured result | current threshold | outcome |
| --- | ---: | ---: | --- |
| all lights off | on 73.705, off 11.998, drop **61.707**, 0 warm pixels | drop > 40.0 | passes with 21.707 levels headroom |
| fine mesher (`--subdiv 2`) | **11** enclosed-sky pixels in **7** blobs | ≤ 2,300 pixels, ≤ 20 blobs | passes |

Neither guard needs a threshold move: `ALL_OFF_DROP_FLOOR` remains 40.0 and
`ENCLOSED_SKY_CEILING` remains 2,300. The new exact-sky residual is far lower than the prior
2,042-pixel calibration because `Msaa::Off` plus FXAA changes the exact-colour edge measure; it
does not justify silently tightening a guard. The 11 pixels in 7 blobs remain below the existing
topological allowance, whose source comment records the whole-world face oracle as the evidence
that this residual is not terrain holes. Any threshold change requires Wolf's ruling.

`crates/gui/tests/capture.rs` has no story diff and ran green: 8 passed, 1 intentionally ignored.
