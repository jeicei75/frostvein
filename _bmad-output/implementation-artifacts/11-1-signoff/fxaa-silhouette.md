# 11.1a FXAA silhouette measurement — re-measured at EV100 10.5, 2026-09-19

Re-taken at the code review after Wolf ruled the exposure to **10.5 EV100** and the capture
ground-median floor to **55**. Build `77d5056` plus the review patches; boot framing, 1280x720,
`--headless --static-world --subdiv 4 --frames 160`, a **fresh daemon per capture** (`--static-world`
freezes at the tick reached AT CONNECT, so captures sharing a daemon freeze worlds tens of ticks
apart).

The window is **east-ridge**, `(1120, 240)..(1280, 360)`: it straddles the dark eastern ridge edge
and its sky rather than the camp or a flat open-sky area. The measure is a literal count of pixels
exactly equal to the render sky colour `[5, 12, 28]`; lower is smoother, because FXAA blends the
hard sky/terrain boundary away.

**FOUR captures per condition, not two.** The previous version of this file derived its floor from
two, which Task 0's own rule ("re-take at least four") and the Dev Notes ("a two-sample pair is not
a floor", issue #98) both forbid — and this is the statistic AC5 actually judges. Raised by the
Acceptance Auditor at the 2026-09-19 code review.

| condition | a | b | c | d | same-build spread |
| --- | ---: | ---: | ---: | ---: | ---: |
| FXAA on (shipped) | 556 | 556 | 556 | 556 | **0** |
| `--fx-off fxaa` | 679 | 679 | 679 | 679 | **0** |

- **Same-build floor: 0 pixels**, over four samples in each condition, not two.
- **Hard-edge delta: 123 pixels** (`679 - 556`), far above that floor.
- Direction is AC5's: the `--fx-off fxaa` frame has MORE exact sky-colour pixels at the ridge edge.

**The filter is not blind, and that was checked rather than assumed.** A count of zero can mean
"clean" or "the predicate matches nothing" — issue #108 is exactly that failure on a sibling guard,
where `Hdr` moved the sky's value and emptied an exact-colour filter. Measured here, the three
darkest colours present in the window are `(5,12,28)x556`, `(5,13,28)x128`, `(6,13,28)x32` with FXAA
on and `(5,12,28)x679`, `(5,13,28)x142`, `(6,13,28)x33` with it off. The exact sky colour is present
in quantity in both, so the count is reading something.

**Exposure does not move this statistic.** The `--fx-off fxaa` count is 679 at both 9.7 and 10.5 EV100
— identical — because the sky colour is not exposure-scaled. Only the FXAA-on count moved (547 at
9.7, 556 at 10.5), since FXAA blends sky against terrain and the terrain darkened. This is why the
delta fell 132 -> 123 while the floor stayed 0.

**Superseded:** `fxaa-on-c3735d1-{a,b}.png` and `fxaa-off-c3735d1-{a,b}.png` were taken at 9.7 EV100
and no longer show the shipped exposure. They are kept as the pre-ruling record; the frames that
show the shipped look are `ev10.5-fxaa-{on,off}-{a..d}.png`.
