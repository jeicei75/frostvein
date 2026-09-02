# 10.4 signoff artifacts

Produced at story creation, 2026-09-02, on `2ef194d`, venue **Blender 5.2.1 LTS**.
Figures are NOT comparable against Blender 4.3.2 output — the venue is part of the evidence.

| File | What it is | range-check |
|---|---|---|
| `control-shipped-trees-blender-5.2.1.png` | the valley as it ships today — the control every candidate is judged against | `exposed_cells=44984 non_sky_fraction=0.686736 distinct_colors=59191 terrain_luma=105.853` |
| `red-no-trees-blender-5.2.1.png` | the same world with all 5,582 tree cells removed — the deliberate RED proving the bench sees trees | `exposed_cells=40148 non_sky_fraction=0.662805 distinct_colors=27999 terrain_luma=125.883` |

Neither is an approved artifact. The control is the baseline for judging; the RED is instrument
proof. **The RED exited 0 and passed its range check** — the floors (0.02 / 32 / 20.0) cannot fail
on a treeless world, so judgement rests on the printed figures, not the exit code.
