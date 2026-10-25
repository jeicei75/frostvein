# 13.2 Task 5: the connectivity map (`prototypes/connectivity.patch`)

Stack: `4f57027` + `census.patch` + `counters.patch` + `connectivity.patch` (`git apply --check` clean in that order).
Census: `census.sh prototypes/counters.patch prototypes/connectivity.patch` -> `census-connectivity.txt` (variant T, the
default) and `FV_CONN=i census.sh ...` -> `census-connectivity-i.txt` (variant I). Per-scenario tables below come from
`simd --bench scenarios/*.json --runs 2` of the same binary with `FV_CONN=off|t|i` (`off` equals base + counters:
all 10 state hashes equal `census-counters.txt`). Scale: `scale-connectivity-sub.txt` (see the end).

## What the patch does

`Terrain` gets `conn: Mutex<Connectivity>`, modelled on `pile_ground`. The map is a component id per walkable cell over
`astar_neighbours`' rule with a *key* set closed, labelled lazily: the first question about a start cell floods that
cell's whole component and labels it. Goals are never flooded: "goal has the start's id" is a lookup.
- Invalidated by `set_tile` (counter `rebuild_tile`) and when the key changes (`rebuild_key`). A zones change needs no
  hook: zones never enter the connectivity rule (`pile_ground` needs it because its answers depend on zones).
- Key is set by `conn_sync` where the world's `blocked` is built (`claim_jobs`, and per dwarf in `execute_jobs`).
  Variant T: key = emitters. Variant I: key = emitters + uncarried items (`FV_CONN=i`). `FV_CONN=off` disables it.
- Use 1: `astar_with_budget_inner` returns "no path, completed, no budget spent" when start and every goal differ in id.
  This covers `claim_jobs` (delivery + goals), `execute_replan`, `resolve_reroute`, `clearing_kept_landing`. Only used
  when key is a subset of the caller's `blocked` (sound: more closed cells never connect anything); else it is silent.
- Use 2 (I only, key == `blocked` exactly): `caging_stone` returns `None` at once when the miner's component has more
  than 64 cells (the answer its first `pocket` flood gives after 65 cells).
- Probe (counts only, I): at each `walls_in_a_dwarf` call, is some dwarf already outside the largest dwarf-component?

## Behaviour: no hash moved

All 10 scenarios: state hash identical to base in T and in I (`d32911a5...`, `14627076...`, `ab78f0f9...`, `89f558d4...`,
`862f0e8d...`, `90471faf...`, `5688e4a4...`, `2479b66a...`, `17b53b99...`, `8365ab4d...`). End lines (marks/items/zones/goals)
therefore equal base: big-pile 14/32/518, deep-pit 0/28/22, dig-loop 11/44/21, full-pile-crowd 0/46/4, plan-compact 0/58/203
goals 12/6/1, plan-far 0/58/90 12/10/1, plan-spread 0/58/203 12/2/0, plan 0/58/78 12/10/1, seat-slow 1/129/222 38/12/1,
stuck-tunnel 3/57/89 38/4/0. Sweeps (T and I) equal base line for line: cage frozen `[(10,6,0),(12,4,1),(12,4,4),(12,6,3)]`
pinned, unfinished `[(11,6,1)]` pinned, landed `[]`, runs=32 worked=32, campfire-189 stored=23/24, headon 4/4 shares=0 swaps=0,
staircase1 failed=[] 96/96, staircase2 24/24, rings 20/20, 20/20, **14/24**, 19/19, pocket207 24/24. `shares=0` on every
block and sweep line; every counters block IDENTICAL across 2 runs (10/10 in both censuses).
Why none moved: the budget never runs out in these worlds (`budget=0` on every A* row), and the map only short-circuits a
search whose answer ("completed, no path") it knows, so no later search sees a different budget.

Where it does change behaviour (not reachable in these scenarios, shown by a unit test): lib test
`a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on` FAILS under T and I (passes under `FV_CONN=off`):
retry stamps `[120 x10, 0]` instead of `[120, 0 x10]`: the ten unreachable jobs are answered for free and stamped, the
reachable one behind them is claimed. That IS #159's fix (a budget-exhausted unreachable job no longer starves the queue)
and the test's pin is the old rule. The other six `claim_jobs`/budget lib tests pass.

## Soundness RED

Sabotage: labelling restricted to same-z neighbours (ramps never joined). T binary, `plan`: state `2479b66a...` ->
`26ff1eb0...`, end `marks=0 goals 12/10/1` -> `marks=49 items=0 goals=0/0/0`, `conn:claim_goals answered=6493`; `stuck-tunnel`
`8365ab4d...` -> `0bdf70f7...`; `deep-pit` refuses a Channel designation and aborts. So the hash check does catch a wrong map.

## Invalidation numbers (the Task 5 trap)

Rebuild = labels thrown away. "labelled" = cells flooded to relabel (cost unit: one BFS cell). Per 1,000 ticks = (tile + key
rebuilds) x 1000 / ticks.

| scenario | ticks | T rebuilds (tile) | T /1k | T cells labelled | I rebuilds (tile+key) | I /1k | I cells labelled | I cells/labelling |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| big-pile | 5006 | 32 | 6.4 | 530,433 | 32+120 | 30.4 | 2,440,924 | 14,793 |
| deep-pit | 3000 | 32 | 10.7 | 514,016 | 32+88 | 40.0 | 1,925,881 | 16,049 |
| dig-loop | 17640 | 12 | 0.7 | 208,871 | 12+10 | 1.2 | 304,641 | 6,923 |
| full-pile-crowd | 10006 | 46 | 4.6 | 739,662 | 46+94 | 14.0 | 2,247,739 | 15,718 |
| plan-compact-piles | 10006 | 49 | 4.9 | 804,218 | 49+174 | 22.3 | 3,595,912 | 15,702 |
| plan-far-piles | 10006 | 49 | 4.9 | 804,218 | 49+196 | 24.5 | 3,932,994 | 15,545 |
| plan-spread-piles | 10006 | 49 | 4.9 | 804,218 | 49+159 | 20.8 | 3,355,176 | 15,533 |
| plan | 10006 | 49 | 4.9 | 804,218 | 49+196 | 24.5 | 3,932,994 | 15,545 |
| seat-slow | 56759 | 0 | 0.0 | 16,144 | 0+14 | 0.2 | 240,021 | 8,000 |
| stuck-tunnel | 28082 | 0 | 0.0 | 16,083 | 0+216 | 7.7 | 3,477,751 | 10,700 |

T is invalidated only by digs/channels (`set_tile`): 0 to 10.7 per 1,000 ticks, about 16k cells each (the walkable
component). I adds one key rebuild per item pick-up/drop that a query follows: 7.7 to 40 per 1,000 ticks.

## What it answered, and what it cost

| scenario | A* nodes base | T | I | caging cells base | I | A* calls answered T | I | caging answered by I |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| big-pile | 121,371 | 121,371 | 57,143 | 266,549 | 3,695 | 0 | 4 | 5775/5847 |
| deep-pit | 66,654 | 66,654 | 66,654 | 114,393 | 0 | 0 | 0 | 2489/2489 |
| dig-loop | 80,858 | 80,752 | 591 | 203,434 | 157,943 | 377 | 397 | 973/9727 |
| full-pile-crowd | 128,097 | 128,097 | 63,865 | 392,327 | 972 | 0 | 4 | 8310/8328 |
| plan-compact-piles | 147,298 | 147,298 | 50,967 | 354,925 | 1,530 | 0 | 6 | 7749/7779 |
| plan-far-piles | 163,946 | 163,946 | 67,615 | 380,646 | 2,394 | 0 | 6 | 8295/8343 |
| plan-spread-piles | 174,304 | 174,304 | 77,973 | 363,767 | 2,088 | 0 | 6 | 7870/7912 |
| plan | 163,946 | 163,946 | 67,615 | 380,646 | 2,394 | 0 | 6 | 8295/8343 |
| seat-slow | 19,467 | 19,467 | 3,543 | 0 | 0 | 0 | 2 | 0/0 |
| stuck-tunnel | 1,160,963 | 1,160,963 | 55,503 | 459,901 | 34,476 | 0 | 69 | 8689/9337 |

(`A* calls answered` counts only calls the map short-circuited; the `A* calls` rows also count them, now with 0 nodes. In
claim_jobs the old per-call `components` skip is bypassed for answered searches, so call counts rise: dig-loop
`claim_goals` 455 -> 861 calls, 376 of them free.)

Reading the numbers:
- **T answers almost nothing.** Only dig-loop: 377 free "no" answers, but they were the cheap searches (small caged pocket);
  its nodes barely move (80,858 -> 80,752). stuck-tunnel `execute_replan`: 69 nopath, all `passed` by T. Those searches fail
  because of **loose stones**, not terrain (each floods the 16k-cell component to learn "no"). T cannot see them.
- **I answers the expensive ones.** stuck-tunnel `execute_replan` 1,106,988 -> 1,528 nodes (all 69 nopath free), dig-loop
  claim_goals 80,428 -> 162 nodes; staircase2 sweep `claim_delivery` 1,767,544 -> 3,784 nodes; cage sweep `caging_stone`
  5,531,714 -> 5,730 cells. `caging_stone` is answered 8,689/9,337 (stuck-tunnel) and ~99% on the plan family; dig-loop only
  10% (the miners sit in small pockets, which the map can't shortcut, the base flood is already tiny there).
- **The price of I is larger than what it saves, at this scale.** stuck-tunnel: 3.48M cells labelled to save 1.10M nodes +
  0.43M flood cells. plan: 3.93M labelled to save 96k nodes + 378k cells. Roughly 8x more cells labelled than searched, and
  a labelled cell (BTreeMap insert + `astar_neighbours`) costs about what an expanded node does. T's price: 0.5 to 0.8M cells
  labelled per 10k ticks (about 80 cells/tick) for nothing, in every scenario except dig-loop.
  The cause is whole-component relabelling on each key change (about 16k cells), on a world whose total search work is only
  16 to 60 cells/tick. The map is lazy per component; a fix would need incremental update (a removal is a union-find
  merge; an ADD can split a component, the hard decremental case), which I did not build. ms are Opus's to measure.

## The decision: T or I

Neither as built. T is sound, rebuilds rarely (0 to 11/1k ticks) and would be near-free if labelling were cheap, but on
every reproducer the failures that cost anything are item-caused, so it answers 377 cheap searches and nothing expensive.
I answers the expensive failures and the 65-cell cage floods exactly, but is invalidated 8 to 40 times per 1,000 ticks and
costs ~8x its saving in relabelling. If a map is built for M4, it must be I (items are what cages dwarves) and incremental.
Which way I went: both are in the patch (`FV_CONN`, default t); the recommendation is I-with-incremental-update or nothing.

## Guards: one question?

Not built (`connectivity-guards.patch` does not exist). Numbers:
- Cost: `stranded` (the flood behind `walls_in_a_dwarf`/`_from`) visits 432 to 15,272 cells per scenario (seat-slow, stuck-tunnel
  top out), against 114k to 460k for `caging_stone`. The map (I) already removes `caging_stone`'s cost; the wall-in guards
  were never the cost. `closes_last_level_exit` floods nothing.
- Rule: the guards ask a delta ("does closing this cell strand a dwarf who is not already stranded": `after` not a subset of
  `before`). "Does every dwarf still reach work" is a level. Probe (I, at every `walls_in_a_dwarf` call with key == blocked):
  `already_split` = some dwarf is outside the largest dwarf component before the cell is closed: seat-slow **3,018 of 3,018
  calls**, dig-loop 28 of 28, cage sweep 5 of 2,499, staircase2 2 of 2,901; zero elsewhere. In the frozen worlds (#203, #204 seat-slow)
  the level is already false, so a level guard would refuse every drop forever. It is a different rule and a worse one.
- `caging_stone` is not a guard in that sense (it finds the stone that cages a miner) and the map answers it exactly (above).

## Rule departures (candidate "Rulings needed")

1. None in the 10 scenarios or the sweeps (hashes and lists equal).
2. #159 shape: a budget-exhausting unreachable job is answered without spending budget, so the reachable job behind it is claimed
   and the unreachable ones are stamped (`retry_after`). The lib pin `a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on` had to
   change (observed `[120 x10, 0]`). Buys: #159 closed for terrain-unreachable jobs (T) and item-blocked ones (I).
3. The old "jobs sat out stay unstamped" NOTE in `claim_jobs` no longer describes unreachable jobs.

## Issue cells

| issue | cell | number |
|---|---|---|
| #159 | fixed in the unit shape (T and I); unseen in the scenarios (no budget ever exhausted); needs scale for the live 256² case | lib test stamps `[120 x10, 0]`; `exhausted=0` on every row |
| #186 | unchanged | cage frozen list identical (4 runs) |
| #187 | unchanged | unfinished `[(11,6,1)]` |
| #189 | unchanged | campfire-189 stored=23/24 |
| #203 | unchanged (cost only) | dig-loop marks=11, hash equal; I: claim_goals 80,428 -> 162 nodes |
| #204 | unchanged | seat-slow marks=1 goals 38/12/1; full-pile-crowd marks=0 items=46 |
| #206 | unchanged (cost only) | stuck-tunnel marks=3, hash equal; I: execute_replan 1,106,988 -> 1,528 nodes |
| #207 | unchanged / not movement | census_207 ring 14/24 |

## Determinism / traps

No HashMap anywhere (`BTreeMap`/`BTreeSet` only). The map is derived state: `Mutex`, not serialized, rebuilt after load.
A label is only trusted when `key` is a subset of the caller's `blocked`; non-subset calls count `skipped_not_subset`.
`caging_stone` and the probe additionally need `key == blocked`; counted `silent` otherwise (guard probe silent is 190 to 433
of ~500 to 1,150 calls: the guards often run against a `blocked` set built after a mid-tick item move).
The counters' `calls` for answered A* searches include free ones; read `nodes`.

## Scale

See the section appended below by the scale subset run (this agent ran `scale.sh` for 3 points in T and a subset in all three modes).

### Scale subset (what was run)

`scale.sh prototypes/connectivity.patch` (T, 3 runs each) finished only 128-crew005/020/050 (`scale-connectivity.txt`, I killed it
after that: ~2 min per point and growing; the 256² and crew100/200 points were not run by me, Opus runs them). I then ran the
same scale binary directly, `--runs 1`, with `FV_CONN=off|t|i` (`scale-connectivity-sub.txt`; `off` = base + counters). The
sub-run was stopped at 128-crew050 / `i`, which had not finished while `off` and `t` had: I mode is by far the slowest at 50 dwarves.
Counts only (`simd --bench scale/scale-128-crewNNN.json --runs 1`), state hashes equal across off/t/i at crew005 and crew020:

| point | mode | A* nodes | flood cells | map cells labelled | rebuilds | exhausted A* |
|---|---|---:|---:|---:|---:|---:|
| 128-crew005 | off | 351,729 | 148,722 | 0 | 0 | 0 |
| 128-crew005 | t | 351,729 | 148,722 | 321,327 | 19 | 0 |
| 128-crew005 | i | 239,352 | 1,948 | 593,990 | 19 tile + 17 key | 0 |
| 128-crew020 | off | 1,336,575 | 601,219 | 0 | 0 | 0 |
| 128-crew020 | t | 1,336,575 | 601,219 | see file | | 0 |
| 128-crew020 | i | 1,336,575 | 7,366 | see file | | 0 |
| 128-crew050 | off | 2,914,662 | 1,523,151 | 0 | 0 | 0 |
| 128-crew050 | t | 2,914,662 | 1,523,151 | 1,737,217 | 107 | 0 |

At scale the story repeats: T saves nothing; I removes the `caging_stone` floods (601k -> 7k cells at crew020) and some A* nodes but
labels more cells than it saves. `exhausted=0` everywhere, so #159 is not live at these points; 256² crews were not run by me.
