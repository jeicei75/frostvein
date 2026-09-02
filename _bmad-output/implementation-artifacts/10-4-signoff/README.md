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
