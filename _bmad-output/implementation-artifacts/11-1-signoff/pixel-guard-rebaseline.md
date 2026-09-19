# 11.1a pixel-guard re-baseline — re-measured at EV100 10.5, 2026-09-19

Both real rendered guards ran after the render-path change, with their output enabled. Re-measured
at the code review, after Wolf ruled the exposure to 10.5 EV100 and the ground-median floor to 55.

| guard | measured result | threshold | outcome |
| --- | ---: | ---: | --- |
| all lights off | on 60.701, off 11.993, drop **48.707**, 0 warm pixels | drop > 40.0 | passes, **8.707** headroom |
| fine mesher (`--subdiv 2`) | **11** enclosed-sky px in **7** blobs | ≤ 2,300 px, ≤ 20 blobs | passes |

**No threshold was moved by this story.** `ALL_OFF_DROP_FLOOR` remains 40.0 and
`ENCLOSED_SKY_CEILING` / `BLOB_CEILING` remain 2,300 / 20. (`GROUND_LUMINANCE_FLOOR` in
`capture.rs` WAS ruled 70 -> 55 at the same sitting — that is a separate, explicitly recorded
ruling with its own evidence, not a guard moved to make a run pass. Nothing was failing when it
moved.)

## Two corrections to the previous version of this file

**1. The "2,042-pixel calibration" was already stale, and AC8 reasoned from it.** AC8 states
"`ENCLOSED_SKY_CEILING` (2,300, residual 2,042) … Headroom is only 258 px, so expect this guard to
move". That 2,042 is quoted from a `pixel_guard.rs` comment dating to 10.7. Measured at the code
review on a **rebuilt `41b3f02` baseline** — the untouched pre-story build — the real figure is
**43 px / 12 blobs**. Pre-story headroom was therefore 2,257 px, not 258. The premise the whole AC8
exercise was reasoned from was false before this story began.

**2. The attributed cause was wrong, and backwards.** This file previously said the low residual
was "because `Msaa::Off` plus FXAA changes the exact-colour edge measure". Measured separately at
the guard's own conditions (`--subdiv 2`):

| build / flags | enclosed-sky px | blobs |
| --- | ---: | ---: |
| baseline `41b3f02` (MSAA 4x, no FXAA) | 43 | 12 |
| 11.1a `--fx-off fxaa` (`Msaa::Off`, no FXAA) | 61 | **18** |
| 11.1a default (`Msaa::Off` + FXAA) | 11 | 7 |
| ceiling | 2,300 | **20** |

`Msaa::Off` **raised** the count (43 -> 61); only FXAA lowered it. The two do not act in the same
direction and must not be attributed jointly.

## What that table means, and why it is filed rather than fixed

`Msaa::Off` alone takes the blob count from 12 to **18 against a ceiling of 20** — two blobs of
margin on the bar `pixel_guard.rs:464` calls "THE PRIMARY BAR, because it is where the separation
actually is". The shipped default reads 7 only because FXAA blends those edges back down. Measured
on this story's own committed same-build frames, FXAA erases **76% of blobs and 81% of the ≤4 px
blobs**; the guard's own comment records 10.7's trunk-hole family as "38 separate holes but only 135
pixels", ~3.5 px each, so that family re-introduced today would present ~7 blobs and read 14 against
a ceiling of 20 — **the guard would go green on the exact defect it was built for.**

Found independently by the Feature Auditor and the Acceptance Auditor at the 2026-09-19 code
review. Wolf ruled it **folded into issue #108**, which already tracks the same guard going fully
blind under `Hdr` in 11.1b (its exact `== [5,12,28]` predicate matches nothing post-`Hdr`, so it
returns 0 for a frame with the terrain entirely missing). One fix closes both; a re-baseline would
not, because the oracle's premise — that the night sky is a single exact colour over a large flat
region — is what has gone.

The exposure ruling did not move this guard: 11 px / 7 blobs is identical at 9.7 and 10.5 EV100,
because the sky colour is not exposure-scaled.

## The all-off drop lost most of its headroom to the exposure ruling

The drop fell **61.707 -> 48.707** against an unchanged 40.0 floor, so headroom went **21.707 ->
8.707**. This guard measures the lit half, which the darker exposure moves directly. It still passes
and no floor was touched, but it is now the tightest of the three and 11.1b's SSAO and Bloom both
darken the frame further. Worth watching rather than acting on today.

`crates/gui/tests/capture.rs` has no story diff and ran green: 8 passed, 1 intentionally ignored.
The full `--ignored` guard suite ran 7 passed, 0 failed, 459s, on build `77d5056` + review patches.
