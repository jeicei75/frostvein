# 11.1a FXAA silhouette measurement — `c3735d1`, 2026-09-18

Boot framing, 1280x720, with `--headless --static-world --subdiv 4 --frames 160`.
The window is **east-ridge**, `(1120, 240)..(1280, 360)`: it straddles the dark eastern ridge
edge and its sky, rather than the camp or a flat open-sky area. The measure is a literal count
of pixels exactly equal to the render sky colour `[5, 12, 28]`; lower is smoother because FXAA
blends the hard sky/terrain boundary.

| capture | exact-sky pixels in east-ridge |
| --- | ---: |
| FXAA on a | 547 |
| FXAA on b | 547 |
| FXAA off a | 679 |
| FXAA off b | 679 |

The same-build floor is **0 pixels** (a–b is 547–547 with FXAA, and 679–679 without). The
hard-edge delta is **132 pixels** (`679 - 547`), above that floor. This is the required AC5
direction: the `--fx-off fxaa` frame has more exact sky-colour pixels at the ridge edge.
