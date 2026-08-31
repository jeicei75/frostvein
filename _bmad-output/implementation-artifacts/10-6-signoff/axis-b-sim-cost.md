# Axis B — simulation-grid cost (not built)

The current real-world snapshot is **measured** at 7,180,286 wire bytes for 524,288 cells. Tile
payload is measured directly from that compact JSON line (7,179,427 bytes); finer snapshot figures
below repeat each actual encoded tile value, preserving real value sizes and commas. They are thus
**derived**, not guessed from an average.

| Sim k | Cells | Tile bytes | Snapshot bytes | Label |
|---:|---:|---:|---:|---|
| 1 | 524,288 | 7,179,427 | 7,180,286 | measured |
| 2 | 4,194,304 | 57,435,353 | 57,436,212 | derived from real tile encoding |
| 4 | 33,554,432 | 459,482,761 | 459,483,620 | derived from real tile encoding |

## Existing A* measurement

This is a **measured** run of the existing pathfinder, unmodified, on synthetic flat surface grids
of 128² / 256² / 512² (the horizontal form of sim k 1 / 2 / 4). It is deliberately a diagonal
query, which exercises the current aggregate node budget.

| Sim k | Surface edge | Path found | Existing-A* elapsed | Label |
|---:|---:|---:|---:|---|
| 1 | 128 | yes | 0.053594 s | measured |
| 2 | 256 | no — current node budget | 0.181869 s | measured |
| 4 | 512 | no — current node budget | 0.184344 s | measured |

Axis B is not buildable under the current wire cap (64 MiB) beyond k=2 with normal dynamic data,
and the existing A* budget already refuses this representative k=2 diagonal. No simulation-grid,
worldgen, protocol, or pathfinding change is proposed.
