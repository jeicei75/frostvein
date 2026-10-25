# 13.2 Task 7: clamber-over (`prototypes/clamber.patch`, `prototypes/clamber-guards.patch`)

Stacks (all `git apply --check` clean in this order on `4f57027`):
- **A (the prototype):** `census.patch` + `counters.patch` + `clamber.patch`
  `census.sh prototypes/counters.patch prototypes/clamber.patch` -> `census-clamber.txt` (step cost 4)
  `FV_CLAMBER_COST=8 census.sh prototypes/counters.patch prototypes/clamber.patch` -> `census-clamber-cost8.txt`
- **B (guards agree):** A + `clamber-guards.patch`
  `census.sh prototypes/counters.patch prototypes/clamber.patch prototypes/clamber-guards.patch` -> `census-clamber-guards.txt`
- Scale: `scale.sh prototypes/clamber.patch` -> `scale-clamber.txt` (see "Scale" for what finished).
Base column = `census-counters.txt` (census + counters, no prototype). Counts and outcomes only; no ms (other builds ran beside me).

## What the patch does (lib.rs +119/-11, tests/scenario.rs +7 census lines)

- A thread-local `CLAMBER` set = the uncarried item cells, minus emitter cells. It is refreshed wherever `blocked` is built
  (`claim_jobs`, `execute_jobs` per dwarf, `world_blocked`, `wander`). `blocked` itself is UNCHANGED: every guard and flood
  (`walls_in_a_dwarf*`, `closes_last_level_exit`, `caging_stone`, `stranded`, `nearest_free_cell`, `pile_targets`,
  `work_positions`) still reads stones as walls, because they still call `astar_neighbours`. Emitters are in `blocked` and never
  in `CLAMBER`, so fires stay walls.
- New `move_neighbours` = `astar_neighbours` plus the stone cells. Only `astar_with_budget_inner` and `route_to_nearest_inner` use it.
- **A\* step cost** onto a stone cell is `clamber_cost()` = 4 (env `FV_CLAMBER_COST` overrides, for the 4-vs-8 A/B only; a shipped rule
  would be a const). Any other step costs 1. The stale-entry check `queued_f != cost + h` is unchanged.
- **Heuristic stays admissible.** The cheapest step is still 1, and `astar_heuristic` is Chebyshev-like with step >= 1, so it never
  over-estimates; it is untouched. Callers that read `path.len()` as cost: none in sim code (`path.len()` appears only in two unit tests
  on stone-free terrain, `lib.rs` ~:5749/:5929, where the set is empty and cost == steps). The stored `Path` is consumed one cell per step.
- **`route_to_nearest`** was a BFS. It is now a uniform-cost search (`(cost, seq)` heap) that crosses stone cells at the same cost but never
  stops on one (`accept` is not asked about a stone cell). With no stone in play the order is the old FIFO order (cost 1 everywhere,
  `seq` breaks ties), and a cell is accepted on discovery as before. `settle`'s landing search (`settle_landing`) is excluded from clamber:
  it places a dwarf after a fall, it is not a walk.
- **Dwarf-closed tiles stay closed.** `closed_to_dwarves` also records its closed dwarf tiles in a second thread-local (`DWARF_CLOSED`)
  and a stone cell in it is not crossable. Without this a rerouting search walked a holder into a tile another dwarf stood on (below, RED).
- Walking the stored path: `execute_jobs` replans only if `path[0]` is blocked AND not a stone cell; `wander`'s idle-exit route does the same.
  `wander`'s own random moves still use `is_walkable` and never step on a stone.
- Counters (new rows in `counters:` blocks): `clamber:steps execute=N wander=N` (steps onto a stone cell);
  `clamber:boundaries dwarf_on_stone_ticks` (tick boundaries that end with >= 1 dwarf on a loose item's cell) and `dwarf_on_stone_dwarfticks`
  (sum over dwarves); `dwarf_on_emitter_ticks` (a dwarf on a fire's cell; must be 0). Census test: `census cage landed_standing` (below).

## AC 5 and the instruments' REDs (observed)

- **shares=0 everywhere**: 16 of 16 `emitter:ticks ... shares=0` lines in `census-clamber.txt` (10 scenarios + 4 sweeps counters + head-on/one-wide
  `levels` lines), likewise 16/16 in the cost-8 and guards files. Determinism: 10 of 10 `IDENTICAL across 2 runs` in each file.
- **RED, one dwarf per tile.** First cut of the patch (no `DWARF_CLOSED`): `counters big-pile ... emitter:ticks boundaries=5006 shares=11 swaps=0`
  (dwarf 3 and 4 both on (52,71,12) from tick 4558). Final patch: `shares=0` on the same run.
- **RED, fires stay walls.** Sabotage: `set_clamber` includes the emitter cells. Observed `census cage counters clamber:boundaries
  dwarf_on_emitter_ticks=407` (dwarves walking across the campfire), against `dwarf_on_emitter_ticks=0` on every line of the real patch.
  (The 10 bench scenarios read 0 even under the sabotage: no search there routes across the fire, so only the cage sweep discriminates.)
- **RED, no stone placed on a dwarf.** `landed` (existing) counts an item that moved this tick onto a dwarf's end-of-tick tile. Under
  clamber a dwarf can step onto a stone that a writer put down EARLIER IN THE SAME TICK (execute_jobs walks dwarves in id order), which
  `landed` reads as a landing although no writer placed anything on him. So the patch adds `landed_standing`: the same list restricted to a
  dwarf who already stood on that cell at the previous boundary (a writer's fault). Sabotage: delete `|| occupied.contains(&cell)` from
  `clearing_set_down`'s `refused`: `landed` AND `landed_standing` both read 9 entries (`[(4,6,1362,28,4), (9,6,1667,32,0), ...]`).
  Real patch, cost 4: `landed=[]`, `landed_standing=[]`.

## Census: base vs clamber (step cost 4, cost 8) vs clamber + guards

Reproducer end lines, `marks / items / goals(a/b/c)`; the state hash changed in every row of every column (all 10 x 3).

| scenario | base | clamber 4 | clamber 8 | + guards |
|---|---|---|---|---|
| big-pile | 14 / 32 / 0/2/0 | 13 / 33 / 0/2/0 | 7 / 39 / 0/0/0 | 18 / 28 / 0/0/0 |
| deep-pit | 0 / 28 | 0 / 26 | 0 / 28 | 0 / 28 |
| dig-loop (#203) | 11 / 44 | **14** / 42 | 14 / 42 | **18** / 39 |
| full-pile-crowd | 0 / 46 | 0 / 46 | 0 / 46 | **23** / 23 |
| plan-compact-piles | 0 / 58 / 12/6/1 | same | same | **5** / 53 / 12/6/1 |
| plan-far-piles | 0 / 58 / 12/10/1 | same | same | 0 / 58 / 12/**9**/1 |
| plan-spread-piles | 0 / 58 / 12/2/0 | 0 / 58 / 12/2/**1** | same as 4 | **6** / 52 / 12/2/1 |
| plan | 0 / 58 / 12/10/1 | same | same | 0 / 58 / 12/**9**/1 |
| seat-slow | 1 / 129 / 38/12/1 | same | same | 0 / 130 / 38/12/1 |
| stuck-tunnel (#206) | 3 / 57 / 38/4/0 | 3 / 57 / 38/4/0 | same | 0 / 60 / 38/4/1 |

Sweeps:

| reading | base | clamber 4 | clamber 8 | + guards |
|---|---|---|---|---|
| cage `frozen` | [(10,6,0),(12,4,1),(12,4,4),(12,6,3)] | [(0,6,0),(2,6,4),(14,6,4),(15,4,2),(16,6,1)] (0 kept) | [(11,6,4),(12,6,2),(13,6,3),(16,4,3)] | 12 entries |
| cage `unfinished` | [(11,6,1)] | [(13,6,1)] | [(0,4,1),(4,6,1)] | [(4,6,1),(6,6,1)] |
| cage `landed` | [] | **[]** | **[(15,6,1133,17,4)]** (a step onto a fresh stone, `landed_standing=[]`) | 3 entries, all `landed_standing` (real) |
| cage runs / worked | 32 / 32 | 32 / 32 | 32 / 32 | 32 / 32 |
| two-wide staircase `worked` | 24/24 | 24/24 | 24/24 | **12/24**, 12 stalled at level 3 |
| one-wide staircase `failed` | [] 96/96 | [] 96/96 | [] 96/96 | [] 96/96 |
| head-on | 4/4, marks_left=0 | same | same | same |
| campfire-189 | pickups_after_t2500=80 stored=23/24 | **0, 24/24** | 0, 24/24 | 0, 24/24 |
| ring (4 readings) | 20/20, 20/20, 14/24 (last rise 1748, retry_ticks 1964), 19/19 | 20/20, 20/20, **24/24** (last rise 2304, retry_ticks 20), 19/19 | same but 207 last rise 3096, retry_ticks 206 | 207: 24/24 (2375, retry 0) |
| pocket207 | 24/24, 12/12, 12/12 | same | same | same |

Counts (sum over callers; base / clamber 4 / clamber 8 / + guards):

| scenario | A* calls | A* nodes | route cells | flood cells | A* nopath |
|---|---|---|---|---|---|
| big-pile | 3,898 / 4,380 / 3,791 / 2,363 | 121,371 / 59,128 / 55,530 / 31,132 | 686 / 800 / 209,841 / 273 | 274,861 / 277,219 / 256,920 / 2,613 | 6 / 3 / 4 / 2 |
| deep-pit | 1,321 / 905 / 1,208 / 1,239 | 66,654 / 114,633 / 2,507 / 99,141 | 120 / 126 / 137 / 152 | 116,140 / 105,932 / 107,869 / 1,000 | 10 / 8 / 0 / 7 |
| dig-loop | 23,050 / 31,768 / 31,631 / 18,855 | 80,858 / 28,354 / 27,984 / 1,246 | 4,763 / 1,372 / 1,350 / 7 | 206,718 / 74,421 / 80,604 / 148,447 | 113 / 434 / 432 / 20 |
| full-pile-crowd | 31,518 / 32,159 / 32,096 / 16,430 | 128,097 / 39,568 / 44,287 / 25,081 | 1,476 / 34,008 / 1,439 / 332 | 402,947 / 358,342 / 362,890 / 4,047 | 5 / 0 / 0 / 0 |
| plan-compact-piles | 3,930 / 5,077 / 4,989 / 3,191 | 147,298 / 76,330 / 39,417 / 36,382 | 745 / 1,074 / 1,003 / 485 | 380,107 / 353,906 / 360,445 / 23,758 | 7 / 2 / 0 / 0 |
| plan-far-piles | 4,475 / 4,033 / 4,959 / 3,280 | 163,946 / 78,927 / 36,768 / 44,542 | 1,002 / 975 / 1,103 / 846 | 399,646 / 344,507 / 351,368 / 17,003 | 8 / 4 / 0 / 1 |
| plan-spread-piles | 4,216 / 3,966 / 4,630 / 3,594 | 174,304 / 54,155 / 44,682 / 63,355 | 763 / 844 / 941 / 457 | 441,787 / 403,529 / 412,331 / 82,870 | 8 / 1 / 0 / 1 |
| plan | 4,475 / 4,033 / 4,959 / 3,280 | 163,946 / 78,927 / 36,768 / 44,542 | 1,002 / 975 / 1,103 / 846 | 393,386 / 338,235 / 345,096 / 10,743 | 8 / 4 / 0 / 1 |
| seat-slow | 33 / 212 / 69 / 42 | 19,467 / 20,173 / 4,192 / 19,464 | 16 / 62 / 35 / 33 | 30,328 / 29,970 / 31,305 / 22,554 | 3 / 3 / 2 / 3 |
| stuck-tunnel | 11,033 / 11,171 / 11,171 / 296 | 1,160,963 / 732,630 / 731,075 / 68,128 | 1,913 / 1,750 / 1,750 / 273 | 476,826 / 530,886 / 530,886 / 558 | 118 / 93 / 93 / 11 |

Reading the counts: nodes expanded fall 37-69% in 8 of 10 scenarios (a stone is a crossing, not a wall, so fewer searches end in "no
path" and flood the whole component), but deep-pit 66,654 -> 114,633 (+72%) and seat-slow's calls 33 -> 212 go the other way; the world is
different, so these are outcomes, not pure cost. `full-pile-crowd` route cells 1,476 -> 34,008 (23x) at cost 4 and `big-pile` route
cells 686 -> 209,841 at cost 8: `route_to_nearest` is now a heap search over a larger reachable set. The flood (`caging_stone`) cost
does NOT move under clamber alone (guards unchanged): 274,861 -> 277,219 in big-pile. It collapses only with `clamber-guards`
(big-pile 2,613, plan 10,743, full-pile-crowd 4,047, stuck-tunnel 558), and that is the variant that breaks outcomes (below).

## Dwarves on stones (the rule departure, counted)

`clamber:boundaries` per scenario (clamber 4 / 8 / guards): ticks that END with a dwarf on a loose stone's cell, and steps onto one.

| scenario | boundaries ending on a stone (c4 / c8 / guards) | steps onto a stone (c4 / c8 / guards) |
|---|---|---|
| big-pile (5,006) | 441 / 498 / 668 | 43 / 45 / 65 |
| deep-pit (3,000) | 88 / 106 / 178 | 6 / 4 / 16 |
| dig-loop (10,000) | 44 / 22 / 55 | 4 / 2 / 5 |
| full-pile-crowd (10,006) | 157 / 102 / 1,336 | 15 / 10 / 123 |
| plan-compact-piles | 264 / 99 / 517 | 23 / 9 / 47 |
| plan-far-piles = plan | 396 / 143 / 352 | 36 / 13 / 32 |
| plan-spread-piles | 225 / 110 / 407 | 20 / 10 / 37 |
| seat-slow (3,000 bench ticks) | 242 / 55 / 252 | 22 / 5 / 22 |
| stuck-tunnel (10,000) | 221 / 221 / 143 | 62 / 62 / 13 |
| cage sweep (128,000 boundaries) | 4,399 (c4) | 401 execute + 2 wander |
| one-wide staircase (30,027) | 741 | 59 + 3 wander |
| two-wide staircase (51,230) | 3,142 | 301 |
| head-on (1,232) | 66 | 6 |

Each stone crossing keeps the dwarf on the cell for one step period (`STEP_REST_TICKS` = 10 ticks), so about 10 boundaries per step.
The step costs the same TIME as any other step; only the planner's cost differs. A slower real step (Going Medieval's 70% is a single
unverified Steam comment, survey lead 4) would be a second rule on top of this one and was not built.

## Issue cells (clamber 4 unless said)

| issue | cell | number |
|---|---|---|
| #159 | unchanged | live only at 256x256: budget-exhausted A* calls base 10 / 10 / 131 / 218 / 138 (crews 5..200) -> clamber 11 / 8 / 142 / 74 / 126; 0 at every 128x128 point in both (`scale-base.txt`, `scale-clamber.txt`) |
| #186 | unchanged in kind / reshuffled | cage `frozen` 4 -> 5 entries, none of the base four remain; `unfinished` [(11,6,1)] -> [(13,6,1)]. Idle dwarf on a one-level island is `wander` (same z): wander never steps on a stone, so a stone ring around an idle dwarf still strands him in `wander`. NOT movement-by-A\* |
| #187 | unchanged | cage `unfinished` 1 -> 1 (seed 11 -> seed 13); cost 8: 1 -> 2 |
| #189 | **fixed** | campfire-189 pickups_after_t2500 80 -> 0, stored 23/24 -> 24/24 (boxed-in dwarf now clambers out) |
| #203 | worse (not movement) | dig-loop marks 11 -> 14. The hole is sealed by terrain; a stone inside it is crossed now, which changes who digs what, not the seal |
| #204 | unchanged | seat-slow marks 1 -> 1, goals 38/12/1 same; full-pile-crowd 0 / 46 same. The four-stone pocket is crossable in principle, but the bench replay is unchanged |
| #206 | unchanged | stuck-tunnel marks 3 -> 3, goals 38/4/0 same, A\* nodes 1.16M -> 733k (fewer failed searches, same livelock): a dwarf-blocks-dwarf head-on, not a stone |
| #207 | **fixed** | census_207 ring stored 14/24 -> 24/24, free 10 -> 0, last rise t1748 -> t2304, retry_ticks 1964 -> 20 (cost 8: 24/24, last rise t3096, retry_ticks 206) |

## clamber-guards (B): a stone no longer walls anyone in

`clamber-guards.patch` (52 lines): `walls_in_a_dwarf`, `walls_in_a_dwarf_from` and `sides_of` strip the existing stone cells from
`blocked` before flooding (the candidate cell being filled still closes); `caging_stone` returns `None` (no stone cages a dwarf).
`closes_last_level_exit` stays: it protects `wander`, which does not clamber. It is a different rule and it shows:

- Cost: `caging_stone` flood cells 274,861 -> 2,613 (big-pile), 393,386 -> 10,743 (plan), 476,826 -> 558 (stuck-tunnel); `stuck-tunnel`
  A\* nodes 1.16M -> 68k and **marks 3 -> 0** (goals 38/4/1: #206 completes, because the pile no longer fills the tunnel behind him).
- Outcome damage: `landed_standing` = 3 real entries (stones DO land on standing dwarves once the guard is gone), cage `frozen` 4 -> 12
  entries, two-wide staircase `worked` 24/24 -> **12/24** (12 stalled at level 3), full-pile-crowd marks 0 -> 23, big-pile 14 -> 18,
  dig-loop 11 -> 18, plan-compact 0 -> 5, plan-spread 0 -> 6. `shares=0` still holds.
- So guards that "agree" with clamber are a net loss as built: the wall-in guard is also what keeps stones off dwarves and out of the
  staircase. A real version would have to keep the "never under a dwarf" half and drop only the "walls in" half. Not built.

## Rule departures (candidate "Rulings needed" lines)

1. **A dwarf may walk onto and across a loose stone's cell** (a step over an item), at planner cost 4 (cost 8 measured). Today (#162, 12.9):
   a stone walls the tile for walking like a fire. Buys: #189 and #207 fixed; A\* nodes -37% to -69% in 8 of 10 scenarios (deep-pit +72%, seat-slow +4%).
2. **A dwarf can stand on a stone at a tick boundary** (the table above), ending ~441 of 5,006 boundaries in big-pile. The gui must draw it.
3. **Pinned test that must change:** `a_dwarf_with_a_goal_past_a_stone_in_a_corridor_goes_round_it` (`lib.rs` ~:7512,
   "the miner stood in the stone's cell") fails: `cargo test --release -p sim-core --lib` -> 96 passed, 1 failed. The full scenario binary
   passes 87/87 only because `census.patch` already turned the cage and staircase asserts into prints; read those lines above.
4. **A stone can land in a cell a dwarf then walks onto in the same tick.** The cage sweep's `landed` is NOT empty at cost 8 (one entry,
   seed 15 width 6 tick 1133: miner placed the stone, dwarf 4 stepped onto it later in the same tick, `landed_standing=[]`). It is a
   consequence of departure 1, not a writer placing a stone on a dwarf. Cost 4 reads `landed=[]`. A shipped rule needs either "a path step
   onto a stone placed this tick replans" or `landed` redefined; the census cannot say which Wolf wants.
5. **`wander` still cannot step on a stone** (only its idle-exit route can), so the #186 shape is not changed by this.
6. **Not changed in A:** every wall-in guard, `nearest_free_cell`, `pile_targets`, `work_positions`, emitters (still walls: 0 dwarf-on-fire
   ticks in 16 of 16 readings, 407 under the sabotage RED).
7. B only: `walls_in_a_dwarf*`/`caging_stone` weakened as above.

## Scale (`scale.sh prototypes/clamber.patch` -> `scale-clamber.txt`)

All ten points ran to completion (`scale.sh prototypes/clamber.patch`, `--runs 3`, state identical across the 3 runs at every point,
`share_ticks=0` at every point). Base column = `scale-base.txt` (`scale.sh` with no option). Counts from run 1; "marks/items/goals" is the end line.

| point | base marks/items/goals | clamber | A* nodes base -> clamber | route cells | flood cells | budget-exhausted A* calls (#159) |
|---|---|---|---|---|---|---|
| 128 crew 5 | 26/41/0/0/0 | 21/46/0/0/0 | 351,729 -> 50,002 | 32,198 -> 96,486 | 148,722 -> 146,691 | 0 -> 0 |
| 128 crew 20 | 123/116/0/0/0 | 118/121/0/0/0 | 1,336,575 -> 1,212,579 | 224,873 -> 80,827 | 601,219 -> 609,472 | 0 -> 0 |
| 128 crew 50 | 343/221/6/0/0 | 336/222/6/0/0 | 2,914,662 -> 2,205,107 | 64,877 -> 161,611 | 1,523,151 -> 1,459,071 | 0 -> 0 |
| 128 crew 100 | 754/288/10/0/0 | 749/309/10/0/0 | 6,666,627 -> 6,040,269 | 241,116 -> 324,330 | 2,902,763 -> 3,029,326 | 0 -> 0 |
| 128 crew 200 | 1591/425/3/0/0 | 1567/463/3/0/0 | 18,905,469 -> 17,949,426 | 291,569 -> 521,264 | 5,915,334 -> 5,726,777 | 0 -> 0 |
| 256 crew 5 | 35/24/0/0/0 | 34/25/0/0/0 | 514,643 -> 565,148 | 58 -> 150,065 | 163,737 -> 161,561 | 10 -> 11 |
| 256 crew 20 | 148/62/0/0/0 | 149/61/0/0/0 | 587,589 -> 487,261 | 150,170 -> 50,156 | 654,776 -> 652,601 | 10 -> 8 |
| 256 crew 50 | 390/126/0/0/0 | 381/135/0/0/0 | 7,234,046 -> 7,996,743 | 551,047 -> 850,658 | 1,549,433 -> 1,534,903 | 131 -> 142 |
| 256 crew 100 | 796/162/0/0/0 | 785/173/0/0/0 | 12,929,702 -> 5,843,628 | 651,028 -> 651,043 | 3,021,576 -> 2,977,258 | 218 -> 74 |
| 256 crew 200 | 1692/176/0/0/0 | 1669/207/0/0/0 | 11,920,450 -> 12,458,790 | 502,586 -> 452,495 | 6,471,559 -> 6,566,204 | 138 -> 126 |

Reading: at 128x128 the clamber saves A* nodes at 5, 20, 50, 100, 200 crews by 86%, 9%, 24%, 9%, 5%: the 5-crew point drops because
the base's repeated failed floods go away, and the saving shrinks with the crew because the work (each dwarf's own path) dominates.
At 256x256 the effect is mixed: -55% at crew 100, +10% at 5, -17% at 20, +11% at 50, +4.5% at 200. The flood cost (`caging_stone` etc.)
barely moves at any point (within 5%), as at the census, since the guards are unchanged. The marks left are 0-19% below base at
128 crew 5 and within 3% elsewhere; items are equal or higher at 9 of 10 points (up to +18% at 256 crew 200). **#159 is live on 256x256 in the base** (budget-exhausted A* calls 10, 10, 131,
218, 138) and clamber does not remove it (11, 8, 142, 74, 126): it is a different count of the same starvation, not a fix. Wall-clock:
`scale-clamber.txt` carries the means (e.g. 128 crew 200 296 ms, crew 100 89 ms) but other agents' builds and test runs
shared the machine, so they are not evidence; Opus measures ms on a quiet machine.

## Judgement calls

- Thread-locals instead of changing ~25 call signatures: crude on purpose; a real version would carry the stone set in the `blocked`
  argument (e.g. a small struct). Bevy runs each system on one thread, so the per-system refresh is consistent.
- Dwarf tiles closed by `closed_to_dwarves` stay closed even on a stone cell; the other closed sets (plain `blocked`) let a search cross a
  stone a dwarf stands on, as jobs already walk "through the other dwarves" (they step aside or yield, and `occupied` still stops the move).
- `route_to_nearest` gets the cost too, so an escape route does not prefer a clamber to a short walk. CONTROL (observed): the patch with
  `set_clamber` forced to an empty set (stone cells never crossable, everything else as patched, the BFS -> heap rewrite included) gives
  the base state hash on all 10 scenarios (big-pile `d32911a55fd27058`, deep-pit `14627076fa965c39`, dig-loop `ab78f0f971e0326f`,
  full-pile-crowd `89f558d4764d66fb`, plan-compact `862f0e8814e04fb7`, plan-far `90471fafc3169ca6`, plan-spread `5688e4a4e8105d21`,
  plan `2479b66a35ac887e`, seat-slow `17b53b99e9161018`, stuck-tunnel `8365ab4d3aafed06`) and the base `route:` cell counts
  (e.g. big-pile clearing_landing cells=657). So every delta in the tables is the clamber rule, not the rewrite.
- Cost 4 vs 8: 8 is no better (landed non-empty, cage `frozen` reshuffled again, `unfinished` 2). I would carry 4.
