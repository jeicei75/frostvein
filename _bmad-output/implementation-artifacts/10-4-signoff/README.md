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
