# 13.2 scale table (Task 4, AC 6) -- base, no option patch

**Superseded for the option columns by `proposal.md` § Scale** (single-run rows for base and every option, `scale-r1-*.txt`). This
file keeps the 3-run base reading and the #159 component sizes.

From `scale-base.txt` (`scale.sh`, stack 4f57027 + census + counters + scale patches; 2000 ticks per point, `--runs 3`, state
identical across the 3 runs of every point, `commands=applied/total` full on all ten). Tick mean and p99 are the mean of the three
runs. Counter columns are run 1. "nodes expanded" is A* nodes only; `route_to_nearest` cells are a separate column. Marks left
is the `marks=` of the end line (1 mark per dig or cut, 9 per crew member at the start). Timings were taken while other agents ran
builds on the same devpod (load ~5): read the 128x128 crew 200 mean (spread 22%) with that in mind.

| map | crew | tick mean ms | p99 ms | nodes expanded | flood cells | A* calls | exhausted calls | marks left (of start) | share ticks | connectivity | squeeze | clamber |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|
| 128 | 5 | 0.405 | 6.26 | 351,729 | 148,722 | 694 | 0 | 26 (45) | 0 | | | |
| 128 | 20 | 5.25 | 22.4 | 1,336,575 | 601,219 | 2,418 | 0 | 123 (180) | 0 | | | |
| 128 | 50 | 30.9 | 100 | 2,914,662 | 1,523,151 | 8,499 | 0 | 343 (450) | 0 | | | |
| 128 | 100 | 95.7 | 315 | 6,666,627 | 2,902,763 | 17,741 | 0 | 754 (900) | 0 | | | |
| 128 | 200 | 309 | 971 | 18,905,469 | 5,915,334 | 47,571 | 0 | 1,591 (1,800) | 0 | | | |
| 256 | 5 | 0.235 | 0.51 | 514,643 | 163,737 | 363 | 10 | 35 (45) | 0 | | | |
| 256 | 20 | 0.961 | 4.52 | 587,589 | 654,776 | 1,489 | 10 | 148 (180) | 0 | | | |
| 256 | 50 | 8.93 | 62.9 | 7,234,046 | 1,549,433 | 6,866 | 131 | 390 (450) | 0 | | | |
| 256 | 100 | 21.0 | 134 | 12,929,702 | 3,021,576 | 17,893 | 218 | 796 (900) | 0 | | | |
| 256 | 200 | 31.8 | 226 | 11,920,450 | 6,471,559 | 25,468 | 138 | 1,692 (1,800) | 0 | | | |

`route_to_nearest` cells / `none` results, same runs: 128: 32,198/2, 224,873/59, 64,877/85, 241,116/126, 291,569/381; 256: 58/2,
150,170/15, 551,047/288, 651,028/77, 502,586/378 (crews 5..200 in order). `none` is not "budget": the route counter does not
separate budget from no-route.

Read with care:
- The 256 points cost LESS per tick than 128 at crews 20, 100 and 200 although the map is 4x the area: the mine and the pile are
  twice as far from the camp and from each other, so in 2000 ticks (10 ticks a step) dwarves are still walking or digging and
  nothing has reached the pile yet on 256 (200 crew: 176 items, goals 0/0/0), while the 128 crews 50-200 do reach goal progress. 256 shows what a map-spanning search costs; it does not show what a full pile costs there.
- Exhausted calls are A* calls that spent the 50,000-node budget (the "reads as no path" case). 128: none. 256: every crew,
  from 10 at crew 5 to 218 at crew 100. See #159 below.
- Share ticks are 0 everywhere: no two dwarves ever stood on one tile at a tick boundary at these crews.

## #159 on 256x256x32 (DEFAULT_SEED, `scale_generate`, neighbour rule `astar_neighbours`, fires blocked)

| map | camp | walkable component containing the camp | all walkable cells, every level | over `MAX_ASTAR_NODES` = 50,000? |
|---|---|---:|---:|---|
| 128x128x32 | (64,64,9) | 16,063 | 18,336 | no |
| 256x256x32 | (127,128,7) | 64,300 | 73,214 | **yes** (1.29x) |

So #159 is live on 256: a search whose goal is unreachable, or farther than the budget, spends 50,000 nodes and reads as "no path".
Budget-exhausted A* calls (`exhausted calls` column) appear on every 256 point (10, 10, 131, 218, 138) and on no 128 point. At crew 5
all 10 are `resolve_reroute` (500,012 nodes for 15 calls, 4 more `nopath`): a blocked dwarf rerouting to a goal across the map.
Measured by `scale_generate` (printed `component=` per point, identical for all five crews of a map).
