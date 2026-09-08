# AC3 — Per-emitter marginals, at the shipped k=4 default

**Executed 2026-09-08.** Build stamped `gui build c7bfb00`, equal to `git rev-parse --short HEAD`,
no `-dirty`. One daemon (`simd 0`, port 43593) for all eight captures. Boot framing, `--frames 160`,
**no `--subdiv` flag** — every figure here is taken at the shipped default that Task 0 made `k = 4`,
as AC13 requires. No `--z`, so the motion instrument stays live (issue #77).

```
./target/debug/gui <port> --headless --frames 160 --capture <png> [--lights-off <list>]
python3 _bmad-output/implementation-artifacts/10-7-signoff/lumstats.py <png>=<label>
```

Frames: `AC3-<label>-c7bfb00.png`, one per row, committed beside this file.

## Noise floor — two all-on runs of this build, WORST

| | warm-lit | ground median | near-white | blown pool | mean |
|---|---:|---:|---:|---:|---:|
| all-on a | 27,664 | 126 | 2.4398 % | 1.2082 % | 94.472 |
| all-on b | 27,500 | 126 | 2.4872 % | 1.2735 % | 94.588 |
| **floor** | **164 px** | **0** | **0.0474 pp** | **0.0653 pp** | **0.116** |

Reference used for every Δ below is the pair's midpoint: warm-lit 27,582 · median 126 ·
near-white 2.4635 % · blown 1.2409 % · mean 94.530.

## The table

Blown pool is printed and never asserted on headless frames (`capture.rs:596`). ×noise is |Δ|
over the floor above.

| source off | warm-lit | ground median | near-white | blown pool | p99 | mean | Δmean | ×noise (mean) | exit |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| — (all-on a) | 27,664 | 126 | 2.4398 % | 1.2082 % | 232.4 | 94.472 | — | — | 101 |
| — (all-on b) | 27,500 | 126 | 2.4872 % | 1.2735 % | 233.7 | 94.588 | — | — | 101 |
| `sun` | 28,370 | 117 | 2.1296 % | 1.2203 % | 233.6 | 87.689 | −6.841 | 59.0× | 101 |
| `ambient` | 71,855 | **69** | 1.7812 % | 1.1502 % | 229.7 | **47.616** | **−46.914** | **404.4×** | 101 |
| `campfire` | 26,643 | 123 | 2.2784 % | 1.2615 % | 229.9 | 94.142 | −0.388 | 3.3× | 101 |
| `torches` | 9,449 | 117 | 1.6046 % | 0.7369 % | 211.8 | 92.584 | −1.946 | 16.8× | 101 |
| `lanterns` | 30,197 | 125 | 2.1650 % | 1.0310 % | 223.1 | 94.187 | −0.343 | 3.0× | 101 |
| `campfire,torches` | 6,257 | 115 | 1.3102 % | 0.5679 % | 207.8 | 91.904 | −2.626 | 22.6× | **0** |

**Marginals with the confounder OFF** (Premise 4: with torches lit, the campfire's contribution
reads as nil):

| marginal | warm-lit | near-white | blown pool | mean |
|---|---:|---:|---:|---:|
| campfire, torches already off | −3,192 | −0.2944 pp | −0.1690 pp | −0.680 |
| torches, campfire already off | −20,386 | −0.9682 pp | −0.6936 pp | −2.238 |
| **ratio, torches : campfire** | **6.4 : 1** | **3.3 : 1** | 4.1 : 1 | 3.3 : 1 |

At k=1 on `main` the same ratio read 4.5:1 on warm-lit. The torch ring dominates the camp's warm
signature at both resolutions, and more so at the shipped one.

## Two findings that change what the story does next

### 1. Premise 8 no longer holds at the shipped default

The story's Premise 8 and its Verification table recorded that **switching the four torches off
alone takes the shipped guard GREEN** — near-white 1.3342 % against the 1.5630 % ceiling, exit 0.
That was measured at `--subdiv 1`. **At the k=4 default it is false:**

```
torches off, k=4:            near-white 1.6046 %   ABOVE the 1.5630 % ceiling   exit 101
campfire + torches off, k=4: near-white 1.3102 %   under it                     exit 0
```

k=4 raises near-white by roughly 0.28 pp across the board (the all-on control moved 2.1686 % →
2.4635 %), which is about six times the noise floor, and that is enough to carry the torches-off
frame back over the bar. **The one-emitter answer to the guard's red is gone**; the smallest
switch-off that clears the shipped ceiling is now campfire AND torches together. Premise 8 is left
standing in the story text as written, because premises record what was true when measured; this
file is the correction, and no lighting decision should cite Premise 8's exit-0 claim.

### 2. The ambient, not the directional, is what lights this valley

This is the measurement that matters for Ruling 1, and it was not the question anyone asked.

| switched off | Δ frame mean | ×noise |
|---|---:|---:|
| `ambient` (brightness 4,500) | **−46.914** | **404×** |
| `sun` (directional, 22,000 lux) | −6.841 | 59× |

Removing the ambient costs the frame **half its light** and drives dark pixels from 17.5 % to
54.95 % of the frame. Removing the directional costs about a seventh as much. The story's Premise 2
correctly says the directional's 22,000 lux has never been judged as light on the world — this adds
that **it is not the dominant term either.** Re-deriving the table toward the PRD's dark night under
Ruling 1 is therefore mostly an ambient decision, and a candidate that only moves
`directional_illuminance` will move the picture far less than its number suggests. Same shape as the
standing lesson that the loudest knob is often not the one being tuned.

### Instrument caveat: warm-lit counts colour balance, not warm light

`warm_lit_pixels` (`capture.rs:548`) counts pixels where `red − blue > 30`. It is a purely
relative test, so **switching off a COOL source inflates it**: ambient off reads 71,855 warm-lit
pixels, 2.6× the all-on figure, and lanterns off reads 30,197, above all-on. Nothing got warmer in
either frame; the blue fill was removed. Warm-lit is therefore only meaningful for judging warm
emitters while the cool fill is held constant, and its ×noise figures for `sun`, `ambient` and
`lanterns` above must not be read as contribution. Ground median and frame mean are the honest
columns for a cool source.

### Incidental: this is the first evidence for the issue #72 fix in the field

These eight frames are the first captures written by the synchronous encoder that replaced Bevy's
queued `save_to_disk` observer. `lumstats.py` refuses any PNG that is not colour type 2
(`UnsupportedPngColourType`, `lumstats.py:16`), and it read all eight, so the new writer produces
the same RGB format the old one did. Seven of the eight exited 101 and every one of them left its
PNG on disk, which is the ordering AC16 asks for — though it is not the concurrent-load race that
AC16's five full-tier runs exist to test, and that remains unreproduced.
