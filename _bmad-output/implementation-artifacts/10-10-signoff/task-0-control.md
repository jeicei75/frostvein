# Task 0 control — the boot framing, and what an instrument can actually see here

> **CORRECTION, 2026-09-17 (issue #98). Two figures below are wrong about what they measure, and
> one of them is not a noise floor. Do not re-derive from this document without reading this box.**
>
> 1. **"mean luminance" here is a plain unweighted RGB channel average, not luminance.** The
>    `76.1236` below reproduces to four decimal places as the RGB mean of `control-boot-5452c4d-a.png`.
>    That frame's actual Rec.601 luminance — the statistic `10-7-signoff/lumstats.py` computes and
>    `crates/gui/tests/pixel_guard.rs` asserts on — is **71.1931**. The two differ by ~5.0 on this
>    frame, which is enough to look exactly like a look regression to anyone comparing across tools.
>    It did that once already: the figure was read as a 5-point baseline move on 2026-09-17.
> 2. **The `0.0048` swing is not the same-build noise floor.** It is one two-sample pair that
>    happened to land tight. Re-measured on build `8eb1d1d` (no production change between the two —
>    only the build-stamp fix #96 and a docs commit #97), the same-build pair swing is **0.0724**
>    on Rec.601 and **0.0684** on the RGB average: **14–20x** this figure. Across five same-build
>    captures in that run, `near-white-area` spread 0.5655 %–0.7922 % (cf. issue #90).
>    A two-sample pair cannot establish a floor; it can only produce a lucky-tight one.
> 3. **Consequence:** story 10.10's AC1 gated on "within 10x the 0.0048 swing" = a 0.048 tolerance,
>    which is *narrower than the measured same-build noise*. Wolf struck the clause on 2026-09-17;
>    the hand-written-literal rig guard is the whole of AC1.
> 4. **The 766x RED headroom below is inflated for the same reason** — it divides an RGB-average
>    delta by a lucky-tight pair swing. The RED's *ranking* conclusion survives (mean luminance
>    discriminates, changed-pixel count does not); its multiplier does not.
>
> The level itself never moved: creation → `8eb1d1d` is 0.0670 (Rec.601) / 0.0589 (RGB average).
> All four control PNGs are committed beside this file, so this was settled off the images in hand.

**Build:** `gui build 5452c4d` (HEAD `5452c4d`, clean tree — the stamp was verified, see the trap
at the bottom). Daemon `./target/debug/simd 7451`.

```bash
./target/debug/simd 7451 &
./target/debug/gui 7451 --headless --static-world --subdiv 4 --frames 160 \
  --capture _bmad-output/implementation-artifacts/10-10-signoff/control-boot-5452c4d-<a|b>.png
```

| Frame | `capture range check:` | Exit |
| --- | --- | ---: |
| `control-boot-5452c4d-a.png` | `warm-lit pixels=27852 ground-median-luminance=81 near-white-area=0.6597% blown-pool=0.5044% p99-luminance=181.7` | 0 |
| `control-boot-5452c4d-b.png` | `warm-lit pixels=27918 ground-median-luminance=81 near-white-area=0.6488% blown-pool=0.5123% p99-luminance=181.8` | 0 |

Both frames report `subdiv 4: projected 45861 terrain cubes ... faces=838770 triangles=123644`.

## The same-build noise floor, measured on this build

Two runs of the SAME binary against the SAME world, diffed per pixel on max channel delta:

| statistic | same-build swing (a vs b) |
| --- | ---: |
| changed pixels (delta > 0) | **55,284 px — 5.999 % of the frame** |
| changed pixels (delta > 8) | 13,039 px — 1.415 % |
| max channel delta | 203 |
| **mean luminance** | **0.0048** |

**The changed-pixel count is not a usable guard at this framing.** Snowfall and stars animate, so
6 % of the frame differs between two runs of an unchanged binary. An AC keyed to "changed pixels
at or below the noise floor" would tolerate a regression that moved up to 55,284 pixels.

## The deliberate RED, and what it corrects

The condition broken: the camera itself, via the existing `--distance 80` (boot is
`BOOT_DISTANCE = 90.0`). Restore: drop the flag. No source was modified, so no mutant binary
survives this measurement.

| statistic | boot (d=90) | RED (d=80) | RED / same-build noise |
| --- | ---: | ---: | ---: |
| **mean luminance** | 76.1236 | 79.8025 | **766x** |
| changed pixels (delta > 0) | — | 88.54 % | 14.8x |

**Ranking the fields, which is the point of the RED:** mean luminance carries 766x headroom over
its own noise; the changed-pixel count carries 14.8x over a 6 % floor. Mean luminance is the
discriminating statistic at this framing and the changed-pixel count is the nearly-inert one.
Story 10.10's AC1 is written against the float-exact rig comparison first, and mean luminance
second; it does not key off changed pixels.

## Two traps this measurement surfaced

1. **The capture's near-white ceiling is calibrated for the BOOT framing only.** The RED run at
   `--distance 80` exits **101** on `near-white-area=1.1134%`. A `--camera` flag makes non-boot
   framings routine, so captures that trip the ceiling become routine with it. Per 10.8's standing
   rule the ceiling is measured, **not raised** — 10.10 must decide what a non-boot capture does
   about it rather than discovering it at gate time. Related: issue #90.
2. **The build stamp went stale across a commit.** `./target/debug/gui --version` reported
   `bd5a9df-dirty` on a CLEAN tree at HEAD `5452c4d`, after a `cargo build` that recompiled the
   `gui` crate. `crates/gui/build.rs` declares `rerun-if-changed` on `../../.git/HEAD` and
   `../../.git/index`, and the script did not re-run; `touch crates/gui/build.rs` then produced the
   correct `gui build 5452c4d`. Every frame above was captured AFTER that correction. The stamp is
   the mechanism that catches a stale or mutant binary, so a stamp that silently names the wrong
   commit — and falsely claims `-dirty` — defeats the guard it exists to be.
