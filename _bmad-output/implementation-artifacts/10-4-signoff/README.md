# 10.4 signoff artifacts

Produced at story creation, 2026-09-02, on `2ef194d`, venue **Blender 5.2.1 LTS**.
Figures are NOT comparable against Blender 4.3.2 output — the venue is part of the evidence.

| File | What it is | range-check |
|---|---|---|
| `control-shipped-trees-blender-5.2.1.png` | the valley as it ships today — the control every candidate is judged against | `range-check: blender=5.2.1 exposed_cells=44984 non_sky_fraction=0.686736 distinct_colors=59191 terrain_luma=105.853 floors(non_sky_fraction=0.020000, distinct_colors=32, terrain_luma=20.000)` |
| `red-no-trees-blender-5.2.1.png` | the same world with all 5,582 tree cells removed — the deliberate RED proving the bench sees trees | `range-check: blender=5.2.1 exposed_cells=40148 non_sky_fraction=0.662805 distinct_colors=27999 terrain_luma=125.883 floors(non_sky_fraction=0.020000, distinct_colors=32, terrain_luma=20.000)` |
| `candidate-A-0.50-0.72-0.98-blender-5.2.1.png` | sharper spire, taper `0.50 / 0.72 / 0.98` | `range-check: blender=5.2.1 exposed_cells=44984 non_sky_fraction=0.684421 distinct_colors=58776 terrain_luma=105.994 floors(non_sky_fraction=0.020000, distinct_colors=32, terrain_luma=20.000)` |
| `candidate-B-0.72-0.88-0.98-blender-5.2.1.png` | fuller crown, taper `0.72 / 0.88 / 0.98` | `range-check: blender=5.2.1 exposed_cells=44984 non_sky_fraction=0.688540 distinct_colors=58679 terrain_luma=105.371 floors(non_sky_fraction=0.020000, distinct_colors=32, terrain_luma=20.000)` |
| `candidate-C-0.52-0.68-0.86-blender-5.2.1.png` | sparser crown, taper `0.52 / 0.68 / 0.86` | `range-check: blender=5.2.1 exposed_cells=44984 non_sky_fraction=0.684587 distinct_colors=58549 terrain_luma=106.916 floors(non_sky_fraction=0.020000, distinct_colors=32, terrain_luma=20.000)` |

None is an approved artifact. The control is the baseline for judging; the RED is instrument
proof. The candidates are all unapproved. The RED exited 0 and passed its range check —
the floors (0.02 / 32 / 20.0) cannot fail on a treeless world, so judgement rests on the printed
figures, not the exit code.

## AC5 client captures — REGENERATED at the 2026-09-02 code review

The two captures previously committed here were **both** renders of the pre-mesh build. The
review confirmed it three independent ways: both frames show cube foliage platters on cube trunks
under 3x zoom and to the naked eye; a rebuild of `9eba31f` re-running the recorded recipe exits 101
and writes no PNG at all; and a real HEAD capture differs from the committed "HEAD" file by
257,952 pixels at delta>=4, which is 39x the same-build noise floor. They were one treatment
photographed twice — the exact failure AC3 was written to name, landing on AC5's artifacts.

Replaced with captures from real builds, both `--headless --subdiv 2 --frames 160`, 1280x720:

| artifact | build | terrain draw | trees |
|---|---|---|---|
| `client-baseline-2ef194d-subdiv2.png` | `2ef194d` via `git archive` (stamps `unknown`: the archive carries no `.git`) | 49,933 cubes at z31 | 5,048 cube-tree cells |
| `client-head-06471d7-subdiv2.png` | `06471d7`, clean stamp, no `-dirty` | 44,885 cubes at z31 | `meshes=265 of 265 scenes_loaded=true source=embedded` |

`49,933 - 44,885 = 5,048`, exactly the tree-cell census — the two builds account for the same world.

**The difference, against a measured noise floor.** `--at-tick` cannot be used on this venue (its
tick floor demands as many OBSERVED ticks as requested and software rendering observes about a
third), so the comparison is `--frames` and the noise is stated beside the signal rather than
assumed away:

| comparison | raw | >=4 | >=16 |
|---|---|---|---|
| **baseline vs HEAD (the AC5 claim)** | **289,673** | **264,839** | **201,914** |
| same code, two runs | 62,007 | 6,620 | 1,395 |
| same code, another pair | 78,332 | 46,050 | 8,876 |

Signal is **5.7x the WORST observed noise** at delta>=4 and **22.6x** at delta>=16. Compare the
figures the story previously published — 81,101 / 36,176 / 7,939 — which sat *inside* that noise
and would have read "changed" no matter what was built.

Both captures write their PNG and then panic on the pre-existing `NEAR_WHITE_AREA_CEILING`, which
is the software-rendering condition that constant's own comment predicts. The subdiv-2 lantern
failure that ALSO panicked here was this story's own regression and is fixed: `lit terrain tiles
at dwarf positions=1870 moved=true`, where it read 0 before.

### Per-tree yaw — added after the patch pass, on Wolf's call

The client applied NO rotation to any pine, while `authored_bench.py` had always spun each one in
quarter turns and said why in its own comment: *"so 265 copies of four meshes do not all face the
camera identically."* So the frame approved as candidate D differed from the one the client draws.
A bench/client divergence, and a concrete look defect rather than taste — which is the bar for a
look change on this project.

Yaw is a SEPARATE salted draw from the same stable FNV hash; the species assignment is untouched
and pinned by its own mutation row, because FNV-1a mixes every byte and routing the variant through
a zero-salted hash silently reshuffles which pine each column gets while reading as a tidy
refactor. That nearly shipped.

**Measured, not asserted** — the same noise floor applies:

| comparison | raw | >=4 | >=16 |
|---|---|---|---|
| no-yaw `daeb2c9` vs yaw `06471d7` | 158,752 | **116,963** | **81,204** |
| worst same-code noise | 78,332 | 46,050 | 8,876 |

2.5x the worst noise at delta>=4 and 9.1x at delta>=16, so the quarter turns are genuinely visible
rather than lost on a near-symmetric conifer — which was the open question worth checking before
claiming the change does anything. Whether the forest READS better is Wolf's eye on the vehicle,
not this measurement.

## The triangle load, and the correction to this story's own text

Measured 2026-09-03 on `12da79d`, in the devpod, for the vehicle card's fps section. No frame rate
has ever been taken against the mesh trees, so the card needs to say what it is asking Wolf's GPU
to draw.

| | subdiv 1 — **what `gui.exe` draws with no flag** | subdiv 2 | subdiv 4 |
|---|---:|---:|---:|
| terrain triangles | 576,972 (48,081 cube entities) | 151,062 (118 chunks) | 926,426 (118 chunks) |
| 265 pines | ~1,187,000 | ~1,187,000 | ~1,187,000 |
| **total** | **~1,764,000** | ~1,338,000 | ~2,113,000 |
| trees' share | **67 %** | 89 % | 56 % |

Terrain figures are read straight off the client's own startup line. The tree figure is derived,
so here is the derivation rather than the number alone: the four embedded GLBs carry **4,366 /
5,894 / 3,474 / 4,424** triangles (Tree01 / Tree02 / Tree03 / Tree04R), and seed 7451's 265 trunk
columns are **86 at sim height 4, 76 at 5, 103 at 6**. Height 5 splits between Tree02 and Tree03 by
hash, so the total lies between **1,095,172 and 1,279,092** and is **1,187,132** on an even split.

Cross-check on the same run: the four GLBs sum to 307,160 + 414,136 + 244,760 + 311,284 =
**1,277,340 bytes**, exactly what `gui tree assets:` prints. The startup line is reading the blobs.

**The correction.** This story's "Still open" note reads *"~1.2 M triangles vs ~479 k terrain"*.
The 1.2 M holds. **The 479 k does not**: it matches no reading at any subdivision, and `rg` finds
it nowhere in the repo — it was written into prose with no source behind it. The table above
replaces it.

## Instruments re-verified before the vehicle card was written — 2026-09-03, `12da79d`

`gui 7451 --headless --subdiv 2 --capture … --frames 160`, so the card hands over a tested recipe
rather than a remembered one:

```
gui build 12da79d
gui tree assets: 4 of 4 embedded in this binary, 1277340 bytes
subdiv 2: projected 44885 terrain cubes at z 31 entities=2160 chunks=118 faces=227110 triangles=151062
gui trees: meshes=265 scenes_loaded=true source=embedded frames=2
slice: z 31 projected 44885 terrain cubes (265 of 265 cut-face tiles at z 31)
lantern: … lit terrain tiles at dwarf positions=1870 moved=true
capture range check: warm-lit pixels=24221 ground-median-luminance=116 near-white-area=1.8159% blown-pool=1.0809% p99-luminance=231.8
exit=101   ← the pre-existing near-white ceiling, PNG written first
```

**The `build.rs` stamp defect reproduced.** The binary would not restamp after a commit until
`crates/gui/build.rs` was touched by hand — the same failure filed at the review. The card makes
that `touch` a mandatory build step rather than a footnote, because a stamp that silently lags is
worse than no stamp: it is trusted.

## The vehicle sitting — 2026-09-03, build `3b0c43f`, RTX 4080 Laptop / NVIDIA 616.56

Sections 2 and 3 of `task-6-vehicle-runbook.md` are closed. `10-4-vista.png` (1280x720) is the
windowed capture.

**What was proven.** A lone `gui.exe` run from `$env:TEMP` with no `assets/` beside it reported
`4 of 4 embedded in this binary, 1277340 bytes`, `projected 39936 terrain cubes at z 31` and
`gui trees: meshes=265 scenes_loaded=true source=embedded`. **The delivery works on hardware** —
that is the failure this story was written around. And the WINDOWED `--capture` printed
`slice: z 31 ... (265 of 265 cut-face tiles at z 31)` and `trees: meshes=265 of 265` and wrote its
PNG, where before the review it asserted `0 == 265` and panicked before the screenshot. **That
regression is dead on the venue that hits it.**

### The near-white ceiling reads WORSE on the GPU, and the card predicted the opposite

| venue | build | near-white-area | ceiling |
|---|---|---:|---:|
| **RTX 4080, windowed** | `3b0c43f` | **2.2071 %** | 1.5630 % |
| lavapipe, headless subdiv 2 | `12da79d` | 1.8159 % | 1.5630 % |
| lavapipe, headless (story's own pair) | `2ef194d` baseline | 1.6709 % | 1.5630 % |
| lavapipe, headless (story's own pair) | HEAD | 1.6604 % | 1.5630 % |

The card said the figure "may well come in under the bar" on a real GPU. **It does not — it is the
worst reading yet, 0.64 points over.** That guess is withdrawn; the reading replaces it.

**Not attributable to the trees on this evidence, and not this story's to fix.** The constant is
9.1's, and the only controlled comparison available says trees move it the *other* way: headless
on the story's own pair, baseline 1.6709 % vs HEAD 1.6604 %, i.e. the mesh trees are marginally
BETTER than the cube trees. What is unmeasured is a **baseline GPU** reading — no `2ef194d`
capture has ever been taken on this hardware, so venue and content are still confounded. Settling
it costs one more cross-build and copy; it was not spent, because a -0.01 point content effect
cannot plausibly explain a +0.64 point venue gap.

**Do not raise the ceiling to make this go away.** It is a measurement on the venue the constant
was calibrated for, which makes it worth more than the devpod readings that preceded it.

### `--frames` guidance corrected

The card said to set the cap absurdly high because it "costs nothing". On the vehicle it costs
wall-clock: Wolf ran `--frames 2000` and the run still took ~14 s, observing 141 ticks and 877
mid-blend frames. **The run is tick-driven and ends on the capture health floor, not on the cap** —
on a vsync'd window 2000 frames is already more than the run will render. Card fixed.
