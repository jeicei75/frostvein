# Axis B — simulation-grid cost (not built)

The current real-world snapshot is **measured** at 7,180,286 wire bytes for 524,288 cells. Tile
payload is measured directly from that compact JSON line (7,179,427 bytes); finer snapshot figures
below repeat each actual encoded tile value, preserving real value sizes and commas. They are thus
**derived**, not guessed from an average.

| Sim k | Cells | Tile bytes | Snapshot bytes | Label |
|---:|---:|---:|---:|---|
| 1 | 524,288 | 7,179,427 | 7,180,286 | measured |
| 2 | 4,194,304 | 57,435,353 | 57,436,242 | derived from real tile encoding and scaled wire fields |
| 4 | 33,554,432 | 459,482,761 | 459,483,651 | derived from real tile encoding and scaled wire fields |

## Existing A* measurement

This is a **measured** run of the existing pathfinder, unmodified, on synthetic flat surface grids
of 128² / 256² / 512² (the horizontal form of sim k 1 / 2 / 4). It is deliberately a diagonal
query, which exercises the current aggregate node budget.

| Sim k | Surface edge | Path found | Debug elapsed (3 runs) | Release elapsed (3 runs) | Label |
|---:|---:|---:|---:|---:|---|
| 1 | 128 | yes | 0.0524 / 0.0532 / 0.0536 s | 0.0058 / 0.0063 / 0.0060 s | measured |
| 2 | 256 | no — current node budget | 0.1766 / 0.1804 / 0.1781 s | 0.0196 / 0.0198 / 0.0192 s | measured |
| 4 | 512 | no — current node budget | 0.1745 / 0.1763 / 0.1769 s | 0.0188 / 0.0200 / 0.0191 s | measured |

**Read the path-found column, not the seconds.** Path-found reproduces exactly on every run and
in both build profiles: that is the finding. The seconds do not survive a change of venue — the
same instrument returned 0.0852 / 0.3380 / 0.3487 s during the 2026-08-31 code review, roughly
2× these debug figures, on a host under load, and release is ~9× faster than debug again. A
number that moves by 9× with a compiler flag and 2× with host load is not a budget, and nothing
downstream may treat it as one.

Axis B is not buildable under the current wire cap (64 MiB) beyond k=2 with normal dynamic data,
and the existing A* budget already refuses this representative k=2 diagonal. No simulation-grid,
worldgen, protocol, or pathfinding change is proposed.
