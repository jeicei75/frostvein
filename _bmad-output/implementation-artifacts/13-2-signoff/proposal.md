# 13.2 proposal: the way through

Story 13.2 (M3-1, #209). A spike: nothing here lands under `crates/`. Every number below names its stack and the command that
printed it. The stacks are throwaway patches under `prototypes/`, applied in order to `4f57027` (`main`, 13.1 merged):

| stack | patches on `4f57027` | used for |
|---|---|---|
| **base** | none | the tick mean of § Profile (`tick-bench`'s base column) |
| **C0** | `census` + `counters` | every count; state hash equal to base on all 10 scenarios |
| **S** | C0 + `scale` | § Scale (`scale.patch` is inert on the committed scenarios: hashes equal C0) |
| **conn** | C0 + `connectivity` (`FV_CONN=t` terrain-only, `FV_CONN=i` terrain + loose items) | option 1 |
| **squeeze** | C0 + `squeeze` (variant B, head-on movers swap) | option 2 |
| **clamber** | C0 + `clamber` (step onto a stone costs 4) | option 3 |
| **pile-memo** | C0 + `pile-memo` (keep `pile_targets`' last answer) | option 4, the cost fix the profile justifies |
| **timer** | S + `timer` (per-system and per-function wall time) | where the ms goes (§ Profile) |

Commands: `census.sh STACK...` (every reproducer's `end` line, state hash and counter block over `--runs 2`, plus the sweep lists),
`scale.sh [OPTION]` (the ten `scale/*.json` points), `scripts/tick-bench.sh A B` (5 interleaved runs per build plus a
shuffled-layout control). Raw outputs sit beside this file (`census-*.txt`, `scale-*.txt`, `tickbench.txt`); the per-option notes
`proto-connectivity.md`, `proto-squeeze.md` and `proto-clamber.md` carry the long tables this file summarises.

**Reading rule.** Counts decide between builds, because they are deterministic and identical across runs (10/10 blocks
`IDENTICAL across 2 runs` in every census). ms supports: cross-build noise is about 6% here (13.1), and the tick-bench control
column puts a floor under each delta.

## Survey

Retro §4.1's leads, checked on the web (Task 2). Only fetched pages are cited; the DFHack quote was re-fetched and matches.

| # | lead | verdict | source | what it means here |
|---|---|---|---|---|
| 1 | DF runs A\* over the whole map | VERIFIED (wiki level) | [DF wiki, DF2014:Path](https://dwarffortresswiki.org/index.php/DF2014:Path) | Plain A\* on a big grid is what DF ships ("in edge cases, will search every single tile"), but DF pairs it with lead 2, so it is not A\* alone. ADR 5 matches DF's search, not its reachability layer. |
| 2 | DF keeps a per-tile walkable-group id; reachable = id compare | VERIFIED | DFHack [`Maps.cpp` L1076-1087](https://raw.githubusercontent.com/DFHack/dfhack/develop/library/modules/Maps.cpp) (`canWalkBetween` = `tile1 && tile1 == tile2`), [`df.block.xml` L288-289](https://raw.githubusercontent.com/DFHack/df-structures/master/df.block.xml), [Lua API](https://raw.githubusercontent.com/DFHack/dfhack/develop/docs/dev/Lua%20API.rst) | A component id answers "unreachable" in O(1). DF's is **terrain only**: it "cannot possibly take into account anything that depends on the actual units", and it is stale while paused. That is this spike's variant T. Frostvein's cages are made of items and dwarves, which T cannot see (§ Prototypes). |
| 3 | Factorio FFF #317 replaced the heuristic with an abstract one | VERIFIED | [FFF #317](https://www.factorio.com/blog/post/fff-317) | A coarse graph (each 32×32 chunk split into connected components) gives the heuristic for the unchanged fine A\*. Entities are ignored, so placing things never rebuilds it. A heuristic cannot make an answer wrong, only slower: the cheapest hierarchical idea, and the one aimed at #159's long-search case. |
| 4 | Going Medieval: items on the floor slow travel to 70% | CORRECTED: player-reported only | [one Steam comment, 30 Jun 2021](https://steamcommunity.com/app/1029780/discussions/0/3055112065882407017) | No dev, wiki or patch source. Precedent at most for "an item costs, it does not block" (clamber). Do not cite 70% as a fact. |
| 5 | Hierarchical A\* (HPA\*) | VERIFIED from a secondary source | [DHPA\*/SHPA\* paper, AIIDE-10](https://cdn.aaai.org/ojs/12397/12397-52-15925-1-2-20201228.pdf) (the Botea paper itself would not load) | Clusters plus a cache of path lengths; a terrain change rebuilds only the damaged clusters. Upkeep per dig is local, but the layer is far more code than any bug here, and it is not built for z-levels with ramps. |
| 6 | Cooperative pathfinding (Silver's CA\*/WHCA\*) | VERIFIED | [Silver 2005, AAAI](https://cdn.aaai.org/ojs/18726/18726-52-22369-1-10-20210928.pdf) | A space-time reservation table stops head-on deadlock in corridors (under 2% failure vs about 20% for "reroute when blocked" at 100 agents), at about 50 ms per turn on 32×32 mazes. Frostvein's `resolve_blocked_step` is the "reroute when blocked" baseline. A whole cooperative planner is a different cost class; a local swap rule (squeeze) targets the same failure. |
| E | RimWorld regions | EXTRA, weakly sourced | [fan article](https://rimworldhub.com/post/ai_paths_unpacked_rimworld_routing_quirks__tactics) | Region graph answers NO_PATH for a sealed area before tile search: the same idea as lead 2. "Pawns pass through each other" was NOT confirmed. |

Fetch gaps: the HPA\* original (404s), raw `df.map.xml` (404; the field is in `df.block.xml`). Quotes from non-PDF HTML pages
(DF wiki, FFF, Steam, rimworldhub) came through a summarising fetcher; `Maps.cpp`, `df.block.xml`, the Lua `.rst` and both papers
were read as raw text.

## Profile

**Counts** (`census.sh prototypes/counters.patch` → `census-counters.txt`, stack C0; both runs of the build print identical
blocks, 10/10). The per-caller tables (A\* calls, nodes, goal-checks and found/nopath/budget/empty per caller; `route_to_nearest`
per caller; cells per flood; guard calls) are in **`profile-table.md`**, which is part of this section. Summary, with the tick mean
from `tickbench.txt` section 1 (`scripts/tick-bench.sh 4f57027 ec1478b4`, quiet devpod, 5 interleaved runs; base, then C0 in
brackets):

| scenario | ticks | mean ms base (C0) | A\* calls | A\* nodes | goal-checks | flood cells | top cost by count | top cost by time (timer) |
|---|---:|---:|---:|---:|---:|---:|---|---|
| big-pile | 5006 | 1.054 (1.035) | 3,898 | 121,371 | 268,131 | 274,861 | `caging_stone` 67% | `pile_targets` 92% |
| deep-pit | 3000 | 0.038 (0.041) | 1,321 | 66,654 | 147,464 | 116,140 | `caging_stone` 63% | A\* 26% (`resolve_reroute`) |
| dig-loop (#203 save) | 17640 | 0.035 (0.037) | 23,050 | 80,858 | 162,319 | 206,718 | `caging_stone` 70% | `caging_stone` and A\* 11% each; spread over systems |
| full-pile-crowd | 10006 | 0.031 (0.034) | 31,518 | 128,097 | 291,208 | 402,947 | `caging_stone` 74% | `caging_stone` 17% |
| plan-compact-piles | 10006 | 1.393 (1.369) | 3,930 | 147,298 | 549,905 | 380,107 | `caging_stone` 67% | `pile_targets` 91% |
| plan-far-piles | 10006 | 0.222 (0.223) | 4,475 | 163,946 | 473,556 | 399,646 | `caging_stone` 67% | `pile_targets` 73% |
| plan-spread-piles | 10006 | 1.683 (1.650) | 4,216 | 174,304 | 929,166 | 441,787 | `caging_stone` 59% | `pile_targets` 92% |
| plan | 10006 | 0.187 (0.184) | 4,475 | 163,946 | 459,228 | 393,386 | `caging_stone` 68% | `pile_targets` 69% |
| seat-slow | 56759 | 0.472 (0.452) | 33 | 19,467 | 76,098 | 30,328 | A\* `claim_goals` 37% | `pile_targets` 75% |
| stuck-tunnel (#206 save) | 28082 | 0.097 (0.100) | 11,033 | 1,160,963 | 2,608,072 | 476,826 | A\* `execute_replan` 68% | A\* 56% (the livelock's replans) |

Counter overhead (C0 vs base): within the noise floor on the six heavier scenarios, +3% to +11% on the four under 0.1 ms; hashes
identical. RED: heuristic-twice (`counters-red-heuristic-twice.patch`) doubles every goal-check count (plan `claim_goals` 176,776 →
353,552) and moves no node count and no hash.

**Counts ranked the wrong thing.** plan-compact (1.39 ms) and plan-far (0.22 ms) have nearly the same A\* and flood counts. What
differs is the zones (203 against 90) and `pile_targets`' cache hits (20.5M against 6.0M), and a cache hit was not counted as work.
So a throwaway **timer** stack (`timer.patch` on S: a lap system between each pair of chained systems, and a drop-timer on each
search and flood function, inclusive) was run once per scenario and at five scale points (`simd --bench FILE --runs 1`, several at
once, so the shares are the reading and not the ms; state hashes equal base). Share of tick time:

| scenario / point | ticks | mean ms | claim_jobs | execute_jobs | other systems | `pile_targets` | A\* (`astar_with_budget_inner`) | `caging_stone` | other floods+guards | `tick_boundary` (counters) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| big-pile | 5006 | 1.12 | 80% | 19% | 2% | **92%** | 1% | 2% | 0% | 0% |
| deep-pit | 3000 | 0.0478 | 18% | 58% | 23% | **17%** | 26% | 13% | 1% | 2% |
| dig-loop | 10000 | 0.0416 | 28% | 30% | 40% | **2%** | 11% | 11% | 0% | 2% |
| full-pile-crowd | 10006 | 0.0386 | 24% | 41% | 33% | **2%** | 14% | 17% | 2% | 2% |
| plan-compact-piles | 10006 | 1.83 | 87% | 11% | 2% | **91%** | 0% | 1% | 0% | 0% |
| plan-far-piles | 10006 | 0.336 | 74% | 18% | 7% | **73%** | 3% | 3% | 0% | 0% |
| plan-spread-piles | 10006 | 2.27 | 89% | 9% | 2% | **92%** | 1% | 1% | 0% | 0% |
| plan | 10006 | 0.3 | 72% | 19% | 8% | **69%** | 3% | 4% | 0% | 0% |
| seat-slow | 3000 | 0.622 | 71% | 17% | 12% | **75%** | 1% | 0% | 3% | 0% |
| stuck-tunnel | 10000 | 0.127 | 6% | 73% | 21% | **2%** | 56% | 7% | 1% | 1% |
| scale-128-crew050 | 2000 | 33.9 | 84% | 15% | 0% | **92%** | 3% | 1% | 0% | 0% |
| scale-128-crew100 | 2000 | 102 | 81% | 19% | 0% | **94%** | 2% | 0% | 0% | 0% |
| scale-128-crew200 | 2000 | 302 | 78% | 21% | 0% | **94%** | 2% | 0% | 0% | 1% |
| scale-256-crew100 | 2000 | 24.9 | 62% | 37% | 0% | **74%** | 18% | 2% | 1% | 1% |
| scale-256-crew200 | 2000 | 37.9 | 35% | 58% | 0% | **75%** | 11% | 2% | 0% | 6% |

Raw: `timer.txt`. Columns are shares of the summed system time. `pile_targets` can exceed `claim_jobs` because the delivery in
`execute_jobs` calls it too. Function times are inclusive (`walls_in_a_dwarf` counted net of the `stranded` it calls). The
timer's own ms read higher than tick-bench's (plan-compact 1.83 against 1.39) because six runs shared the devpod.

`pile_targets` is called from `work_positions` (each haul job a hauler weighs, every tick) and from the delivery in `execute_jobs`.
On a hit it skips the entrance floods but still rebuilds the whole depth map: `is_walkable` and four side neighbours per zone cell,
then a BFS into a fresh `BTreeMap`. That is about 57 µs a call at 203 zone cells, 233,692 calls in plan-compact. **Pathfinding is not
the tick cost wherever the tick is expensive:** in the six scenarios over 0.1 ms and at every 128² scale point, `pile_targets` is
69–94% and A\* 0–3%. A\* leads only in two cheap scenarios: stuck-tunnel (56%, the livelock's 69 no-path replans of about 16k nodes
each) and deep-pit (26%).

**After pile-memo** (`timer.patch` stacked on S + `pile-memo`, raw `timer-pile-memo.txt`; hashes and end lines equal base on all six):

| scenario / point | ticks | mean ms | claim_jobs | execute_jobs | other systems | `pile_targets` | A\* (`astar_with_budget_inner`) | `caging_stone` | other floods+guards | `tick_boundary` (counters) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| plan-compact-piles | 10006 | 0.159 | 69% | 20% | 11% | **43%** | 4% | 4% | 1% | 1% |
| big-pile | 5006 | 0.0956 | 54% | 34% | 11% | **42%** | 12% | 9% | 2% | 1% |
| seat-slow | 3000 | 0.177 | 51% | 23% | 25% | **46%** | 2% | 0% | 5% | 1% |
| scale-128-crew200 | 2000 | 18.9 | 40% | 48% | 1% | **25%** | 24% | 4% | 0% | 11% |
| scale-256-crew100 | 2000 | 5.5 | 40% | 54% | 1% | **7%** | 67% | 6% | 2% | 5% |
| scale-256-crew200 | 2000 | 8.1 | 15% | 59% | 2% | **7%** | 40% | 8% | 1% | 25% |

`pile_targets` keeps 42–46% of the (now much smaller) tick on the pile scenarios: a memo hit still builds its key and clones the
map, so there is more to take if it is ever needed. At scale **A\* becomes the next cost**: 24% at 128² crew 200, 67% and 40% at
256² crews 100 and 200. The tick is far under budget at all three (5.5–18.9 ms, the counters' own `tick_boundary`, which does not
ship, included). This is where the epic's `BTreeMap` storage question lands. A\* storage is not worth a story on 128², but it is
the first thing to measure if M4 goes to 256² with hundreds of dwarves.

## Scale

`scale.sh [OPTION]` with `SCALE_RUNS=1` on the ten `scale/*.json` points (2,000 ticks each, DEFAULT_SEED; 8 dig marks and 1 cut mark per dwarf,
the mine and the pile in opposite quadrants so the haul crosses the map; see `scale/README.md`), stack S plus the option. Raw: `scale-r1-base.txt`, `scale-r1-connectivity-i.txt`
(`FV_CONN=i`), `scale-r1-squeeze.txt`, `scale-r1-clamber.txt`, `scale-r1-pile-memo.txt`. The four option runs went at once (four
single-threaded `simd`s on 32 cores), and pile-memo beside one census, so ms carries that load: base 128² crew 5 read 0.59 ms here
and 0.41 ms in the quieter 3-run `scale-base.txt`. Counts are deterministic and match `scale-base.txt` exactly for base. Conn T is
left out: it answered nothing at the three 128² points it ran (`scale-connectivity.txt`, `scale-connectivity-sub.txt`).

**Tick mean, ms**

| map | crew | base | conn I | squeeze | clamber | pile-memo |
|---|---:|---:|---:|---:|---:|---:|
| 128² | 5 | 0.592 | 0.707 | 0.529 | 0.392 | 0.191 |
| 128² | 20 | 5.71 | 5.56 | 5.02 | 5.62 | 0.942 |
| 128² | 50 | 30.8 | 30.2 | 30.6 | 31.8 | 2.75 |
| 128² | 100 | 95.5 | 92.7 | 95.7 | 100 | 7.23 |
| 128² | 200 | 313 | 303 | 327 | 334 | 20.8 |
| 256² | 5 | 0.244 | 0.445 | 0.234 | 0.324 | 0.179 |
| 256² | 20 | 1.06 | 1.55 | 1.23 | 1.05 | 0.38 |
| 256² | 50 | 9.8 | 10.4 | 10 | 10.7 | 2.99 |
| 256² | 100 | 23.1 | 22.3 | 22.5 | 21.5 | 5.69 |
| 256² | 200 | 34.1 | 34.5 | 35.8 | 36 | 8.45 |

**A\* nodes expanded** (2,000 ticks)

| map | crew | base | conn I | squeeze | clamber | pile-memo |
|---|---:|---:|---:|---:|---:|---:|
| 128² | 5 | 351,729 | 239,352 | 271,479 | 50,002 | 351,729 |
| 128² | 20 | 1,336,575 | 1,336,575 | 851,719 | 1,212,579 | 1,336,575 |
| 128² | 50 | 2,914,662 | 2,898,673 | 2,062,684 | 2,205,107 | 2,914,662 |
| 128² | 100 | 6,666,627 | 6,570,895 | 7,305,834 | 6,040,269 | 6,666,627 |
| 128² | 200 | 18,905,469 | 18,282,688 | 19,661,592 | 17,949,426 | 18,905,469 |
| 256² | 5 | 514,643 | 514,643 | 414,652 | 565,148 | 514,643 |
| 256² | 20 | 587,589 | 587,589 | 886,283 | 487,261 | 587,589 |
| 256² | 50 | 7,234,046 | 7,234,046 | 4,950,601 | 7,996,743 | 7,234,046 |
| 256² | 100 | 12,929,702 | 6,903,257 | 8,031,032 | 5,843,628 | 12,929,702 |
| 256² | 200 | 11,920,450 | 11,670,450 | 13,335,618 | 12,458,790 | 11,920,450 |

**Flood cells**, all floods

| map | crew | base | conn I | squeeze | clamber | pile-memo |
|---|---:|---:|---:|---:|---:|---:|
| 128² | 5 | 148,722 | 1,948 | 151,162 | 146,691 | 148,722 |
| 128² | 20 | 601,219 | 7,366 | 553,681 | 609,472 | 601,219 |
| 128² | 50 | 1,523,151 | 18,178 | 1,460,470 | 1,459,071 | 1,523,151 |
| 128² | 100 | 2,902,763 | 51,906 | 3,032,956 | 3,029,326 | 2,902,763 |
| 128² | 200 | 5,915,334 | 89,814 | 5,993,164 | 5,726,777 | 5,915,334 |
| 256² | 5 | 163,737 | 603 | 163,677 | 161,561 | 163,737 |
| 256² | 20 | 654,776 | 3,418 | 653,472 | 652,601 | 654,776 |
| 256² | 50 | 1,549,433 | 10,935 | 1,506,419 | 1,534,903 | 1,549,433 |
| 256² | 100 | 3,021,576 | 31,785 | 3,006,396 | 2,977,258 | 3,021,576 |
| 256² | 200 | 6,471,559 | 63,039 | 6,493,358 | 6,566,204 | 6,471,559 |

**Budget-exhausted A\* calls** (#159)

| map | crew | base | conn I | squeeze | clamber | pile-memo |
|---|---:|---:|---:|---:|---:|---:|
| 128² | 5 | 0 | 0 | 0 | 0 | 0 |
| 128² | 20 | 0 | 0 | 0 | 0 | 0 |
| 128² | 50 | 0 | 0 | 0 | 0 | 0 |
| 128² | 100 | 0 | 0 | 0 | 0 | 0 |
| 128² | 200 | 0 | 0 | 0 | 0 | 0 |
| 256² | 5 | 10 | 10 | 8 | 11 | 10 |
| 256² | 20 | 10 | 10 | 16 | 8 | 10 |
| 256² | 50 | 131 | 131 | 74 | 142 | 131 |
| 256² | 100 | 218 | 97 | 118 | 74 | 218 |
| 256² | 200 | 138 | 133 | 141 | 126 | 138 |

**Marks left** at tick 2,000 (start: 9 per dwarf)

| map | crew | base | conn I | squeeze | clamber | pile-memo |
|---|---:|---:|---:|---:|---:|---:|
| 128² | 5 | 26 | 26 | 24 | 21 | 26 |
| 128² | 20 | 123 | 123 | 126 | 118 | 123 |
| 128² | 50 | 343 | 343 | 338 | 336 | 343 |
| 128² | 100 | 754 | 754 | 748 | 749 | 754 |
| 128² | 200 | 1591 | 1591 | 1576 | 1567 | 1591 |
| 256² | 5 | 35 | 35 | 33 | 34 | 35 |
| 256² | 20 | 148 | 148 | 148 | 149 | 148 |
| 256² | 50 | 390 | 390 | 374 | 381 | 390 |
| 256² | 100 | 796 | 794 | 794 | 785 | 796 |
| 256² | 200 | 1692 | 1692 | 1682 | 1669 | 1692 |

Reading:
- **The tick budget is 100 ms (10 ticks a second).** Base passes it at crew 50 on 128² (31 ms), reaches it at crew 100 (96 ms)
  and is 3× over at crew 200 (313 ms). No movement option moves that: conn I removes 98% of the flood cells at crew 200
  (5.9M → 90k) and the tick reads 303 ms. **pile-memo, with every count equal to base, reads 20.8 ms** (15× cheaper). Timer at
  128² crew 50, 100 and 200: `pile_targets` is 92%, 94% and 94% of the tick.
- **256² is cheaper per tick than 128²** at crews 20, 100 and 200: in 2,000 ticks nothing reaches the pile there (the walk is
  longer), so those points measure map-spanning searches, not a full pile. A\* is 18% and 11% of the tick at 256² crew 100 and 200
  (timer), against 2–3% on 128².
- **#159 is live on 256² and only there.** The camp's walkable component is 64,300 cells against the 50,000-node budget
  (16,063 on 128²; `scale-table.md`), and every 256² point has budget-exhausted searches. Conn I cuts them only at crew 100
  (218 → 97) and not at crew 50 (131 → 131), so most are **reachable goals farther than the budget**, not unreachable ones. A
  connectivity map cannot fix that; a budget that scales with the component, or a better heuristic (survey lead 3), can.
- Share ticks are 0 at every point of every option.

## Prototypes

Four prototypes, each a crude patch on C0: the three movement families (Tasks 5–7) and the cost fix § Profile justifies. Ghost (`ghost.patch`, `dwarf_tiles` returns nothing) was the
census RED, not an option: it made 3 of 6 reproducers worse, because occupancy feeds the guards and jobs as well as movement.

1. **Connectivity map** (`connectivity.patch`, lib.rs). A component id per walkable cell, labelled lazily one component at a time
   and modelled on `pile_ground`. A\* returns "no path" for free when start and every goal differ in id. Variant **T** keys on
   terrain + emitters and is rebuilt by `set_tile`. Variant **I** also keys on loose items (rebuilt on each pick-up/drop a query
   follows), and it answers `caging_stone` as well. The soundness RED (ramps never joined) moved plan from `marks=0` to `marks=49`,
   so the hash check sees a wrong map.
2. **Squeeze-past** (`squeeze.patch`, variant B). A holder blocked by the same dwarf for `SQUEEZE_WAIT_TICKS = 10` (one step
   period) trades tiles with him in one write. Two head-on movers swap instead of #196's yield, and an idle blocker with no way
   aside swaps too. Both positions change inside one call, so `occupied` holds the same two tiles before and after. Variant A
   (swap only where the ladder says "wait") was nearly inert: only dig-loop moved.
3. **Clamber-over** (`clamber.patch`). A\* and `route_to_nearest` may step onto a loose stone's cell at cost 4 (the heuristic
   stays admissible, since the cheapest step is still 1). `blocked` is unchanged, so every guard and flood still reads a stone as
   a wall, and fires stay walls (sabotage RED: 407 dwarf-on-fire ticks; real patch 0). Cost 8 was measured as well and is no better.
   A fourth patch, `clamber-guards.patch`, also took stones out of the wall-in guards. It broke outcomes (below).
4. **pile-memo** (`pile-memo.patch`, +20/−5 in lib.rs). Not a movement option: it is the cost fix § Profile points at.
   `pile_targets`' answer depends only on terrain, the zones, and `blocked` on the zone cells and their side neighbours, so
   `PileGround` keeps the last answer keyed on that blocked subset. `set_tile` and a zones change drop it, as they already drop
   the entrance-flood answers. It changes no rule, so the hash must not move. Census (`census.sh prototypes/counters.patch
   prototypes/pile-memo.patch` → `census-pile-memo.txt`): **10/10 state hashes and end lines equal C0, every sweep line equal**,
   and 233,593 of 233,692 calls are memo hits in plan-compact. RED: a key that ignores `blocked` moves all four hashes tried
   (plan goals 12/10/1 → 12/5/1, big-pile marks 14 → 15). Tick-bench: see the table below.

**Every reproducer** (`census-*.txt`; end line `marks / items / goals`, then the option's tick-bench delta against C0 from
`tickbench.txt` sections 2–6). "same" means the state hash equals base. Every option row has a shuffled-layout control within
−2.2% … +6.6% (read each scenario's `control` column there).

| scenario | base end | base ms | conn T | conn I | squeeze | clamber 4 | pile-memo |
|---|---|---:|---|---|---|---|---|
| big-pile | 14 / 32 / 0/2/0 | 1.054 | same, +3.4% | same, +11.5% | 12 / 34 / 0/1/0, +33.8% | 13 / 33 / 0/2/0, −4.4% | same, **−91.0%** |
| deep-pit | 0 / 28 / 0/0/0 | 0.038 | same, +118.7% | same, +421.5% | 0 / 27, +8.1% | 0 / 26, +34.5% | same, −10.6% |
| dig-loop (#203 save) | 11 / 44 / 0/0/0 | 0.035 | same, +17.9% | same, +14.8% | 10 / 44, −27.2% | 14 / 42, +6.2% | same, +4.3% |
| full-pile-crowd | 0 / 46 / 0/0/0 | 0.031 | same, +57.7% | same, +159.3% | 0 / 46 (hash differs), −2.7%† | 0 / 46 (hash differs), +7.2% | same, +3.2% |
| plan-compact-piles | 0 / 58 / 12/6/1 | 1.393 | same, +9.1% | same, +12.6% | 0 / 58 / 12/6/1 (hash differs), +4.1%† | 0 / 58 / 12/6/1, +17.9% | same, **−89.1%** |
| plan-far-piles | 0 / 58 / 12/10/1 | 0.222 | same, +21.5% | same, +54.2% | 0 / 58 / 12/10/1 (hash differs), +15.9% | 0 / 58 / 12/10/1, +17.5% | same, **−60.5%** |
| plan-spread-piles | 0 / 58 / 12/2/0 | 1.683 | same, +9.4% | same, +12.0% | 0 / 58 / 12/2/0 (hash differs), +7.7% | 0 / 58 / 12/2/**1**, +16.8% | same, **−89.8%** |
| plan | 0 / 58 / 12/10/1 | 0.187 | same, +20.7% | same, +62.0% | 0 / 58 / 12/10/1 (hash differs), +13.8% | 0 / 58 / 12/10/1, +18.5% | same, **−55.5%** |
| seat-slow | 1 / 129 / 38/12/1 | 0.472 | same, +3.2%† | same, +8.6% | 1 / 129 / 38/**11**/1, +3.1%† | 1 / 129 / 38/12/1 (hash differs), +14.0% | same, **−62.5%** |
| stuck-tunnel (#206 save) | 3 / 57 / 38/4/0 | 0.097 | same, +1.8% | same, +37.4% | **0** / 60 / 38/4/**1**, −71.1% | 3 / 57 / 38/4/0, −4.5% | same, −1.9%† |

† within the noise floor. Squeeze and clamber change outcomes, so their ms compares two different worlds: stuck-tunnel's −71%
is the livelock gone, and big-pile's +34% is a different stone layout leaving more jobs to scan (`claim_goals` nopath 3 → 49).
Connectivity changes no hash, so its deltas are pure cost. **Every scenario got slower under both T and I**, while I cut stuck-tunnel's
`execute_replan` from 1,106,988 nodes to 1,528 and `caging_stone` from about 380k cells to 2.4k on the plan family. The relabelling
costs more than that: about 16k cells per rebuild, 3.5–3.9M labelled per 10k ticks (I) to save about 0.5M (`proto-connectivity.md`).

**Sweeps** (`census …` lines, same files):

| reading | base | conn T and I | squeeze | clamber 4 |
|---|---|---|---|---|
| cage `frozen` (#186) | 4: (10,6,0) (12,4,1) (12,4,4) (12,6,3) | same | 5, 2 of base's kept | 5, none of base's kept |
| cage `unfinished` @4000 (#187) | (11,6,1) | same | 5 entries; 4 still stuck at 16,000 ticks (base 0) | (13,6,1) |
| cage `landed` / runs / worked | [] / 32 / 32 | same | [] / 32 / 32 | [] / 32 / 32 (`landed_standing` []) |
| two-wide staircase worked | 24/24 | same | 24/24 | 24/24 |
| one-wide staircase | failed [] 96/96 | same | failed [] 96/96, 101 swaps | failed [] 96/96 |
| head-on | 4/4 | same | 4/4, 1 swap | 4/4 |
| campfire-189 | 23/24, 80 pick-ups after t2500 | same | **24/24, 0** | **24/24, 0** |
| ring 207 (`census_207_four_bars_round_a_hole`) | 14/24 | same | 18/24 | **24/24** |
| shares (two dwarves on one tile at a boundary) | 0 | 0 | 0 | 0 |

**AC 5 holds in every prototype.** `shares=0` on every scenario block and every sweep line, and the counter has a RED:
`shares > 0` under ghost on 9 of 10 scenarios. It also fired under a squeeze sabotage (holder's `Pos` not written: stuck-tunnel
shares=64) and under clamber's first cut (two dwarves on (52,71,12) in big-pile, which `DWARF_CLOSED` then fixed).

**Unit tests pinned on today's rules** (`cargo test --release -p sim-core --lib` on each stack): conn fails
`a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on` (#159's fix showing, see § Issues; the other six budget tests pass); squeeze fails 4 of 98,
each on the trace helper's "swapped tiles" assert; clamber fails 1
(`a_dwarf_with_a_goal_past_a_stone_in_a_corridor_goes_round_it`). In each case the failing assert is the old rule.

**Guards as one question** (Task 5). Not built, for two measured reasons. First, the wall-in guards were never the cost:
`stranded` visits 432–15,272 cells per scenario against 114k–460k for `caging_stone`, and conn I already answers `caging_stone`.
Second, the guards ask a delta ("does closing this cell strand someone not already stranded"), while "does every dwarf still reach
work" is a level. That level is already false at **3,018 of 3,018** `walls_in_a_dwarf` calls in seat-slow and 28 of 28 in dig-loop,
so a level guard would refuse every drop forever there. `clamber-guards.patch` measured the nearby idea (stones never wall anyone in):
`caging_stone` fell to about 1% (big-pile 274,861 → 2,613 cells) and stuck-tunnel finished (marks 3 → 0), but stones landed on
standing dwarves 3 times, the two-wide staircase fell to **12/24**, and full-pile-crowd was left with 23 marks.

## Issues

One cell per issue per option, from the census and scale files named above. Variant I for conn (T answers none of these).

| issue | conn (I) | squeeze | clamber 4 | pile-memo | kind |
|---|---|---|---|---|---|
| #159 budget starvation | **fixed** in the unit shape: the lib test stamps `[120 ×10, 0]` (the unreachable ten answered for free, the reachable job claimed). Scenarios: unchanged, since no budget runs out (`budget=0` on every row). 256²: see § Scale | unchanged (not movement: claim_jobs budget) | unchanged: 256² budget-exhausted calls 10/10/131/218/138 → 11/8/142/74/126 (crews 5…200) | unchanged (hash equal) | A\* budget, live on 256² only |
| #186 idle on a one-level island | unchanged (frozen identical) | worse/reshuffled: frozen 4 → 5, 2 kept | reshuffled: 4 → 5, none kept | unchanged (hash equal) | **not movement**: `wander` steps on one level only (the issue's own probe: 5/32 → 1/32) |
| #187 refused channel loops | unchanged `[(11,6,1)]` | **worse**: unfinished 1 → 5, 4 stuck at 16k ticks | unchanged count, `[(11,6,1)]` → `[(13,6,1)]` | unchanged (hash equal) | **not movement**: let-go returns to the same refusal (job logic) |
| #189 let-go hauler beside a boxed-in dwarf | unchanged 23/24 | **fixed** 24/24, pick-ups 80 → 0 | **fixed** 24/24, pick-ups 80 → 0 | unchanged (hash equal) | movement: both a swap and a clamber let the boxed-in dwarf out |
| #203 a dig seals the miner | unchanged marks=11 (I: `claim_goals` 80,428 → 162 nodes, cost only) | marks 11 → 10 | **worse** 11 → 14 | unchanged (hash equal) | **not movement**: the dig takes the floor from under his only exit; terrain seals him |
| #204 stones seal two miners (pile full) | unchanged (seat-slow 1 / 38/12/1, full-pile-crowd 0 / 46) | unchanged (seat-slow goals 12 → 11) | unchanged (same end lines) | unchanged (hash equal) | stone placement with the pile full; no option opens the four-stone pocket in the replay |
| #206 stuck in a narrow tunnel | unchanged marks=3 (I: replan 1,106,988 → 1,528 nodes) | **fixed** marks 3 → 0, goals 38/4/0 → 38/4/1 | unchanged marks=3 (A\* 1.16M → 733k nodes, same livelock) | unchanged (hash equal) | movement: dwarf blocks dwarf head-on |
| #207 haulers fill a pocket's way out | unchanged 14/24 | improved 14 → 18/24 | **fixed** 24/24 (retry ticks 1964 → 20) | unchanged (hash equal) | movement (where the haulers stand), fixed by crossing the stones |

Base column for every cell: `census-counters.txt` (C0), which equals `census-base.txt` line for line apart from the counter blocks.

## Rulings needed

Each option breaks one of today's rules (12.9 Task 0 Q1b: one dwarf per tile, no swap; #162: a loose stone walls its tile like a
fire). Wolf ruled on 2026-10-21 that the strict rules stay, so each departure below is put to him and none was assumed. One dwarf
per tile at every tick boundary holds in all of them.

1. **A swap (squeeze).** Two dwarves trade tiles in one step after a one-step-period wait (10 ticks), and two head-on job-holders
   swap instead of #196's yield. *Buys:* #206 and #189 fixed, #207 14 → 18 of 24. *Costs:* #187's sweep regresses (4 runs stuck at
   16k ticks against base's 0); the #194/#196 back-out goes; `step_and_watch` and the 4 lib tests that pin "no swap" are rewritten to
   accept a swap and still fail on a shared tile. The gui would sell the swap with an animation (out of scope).
2. **A dwarf may step onto and stand on a loose stone (clamber).** The planner cost is 4; the walk time is the same as any step.
   *Buys:* #189 and #207 fixed; A\* nodes −37% to −69% in 8 of 10 scenarios. *Costs:* #203's replay worse (11 → 14 marks); dwarves end
   441 of 5,006 boundaries on a stone in big-pile, which the gui must draw; one lib pin changes; `wander` still never steps on a
   stone. At cost 8 a stone placed earlier in the same tick can be stepped onto, so `landed` needs a definition (cost 4 reads `[]`).
3. **An unreachable job is stamped without spending budget (connectivity).** This is #159's fix, and its lib pin changes. No other
   rule moves, because no hash moved.
4. **No guard change is proposed.** The level guard and `clamber-guards` were both measured worse (§ Prototypes).

## Recommendation

**What the spike found, in one line each.**
- The tick cost is not pathfinding. `pile_targets` is 69–94% of the tick in every scenario over 0.1 ms and at every 128² scale
  point, and a memo that changes no outcome takes 128² crew 200 from 313 to 21 ms.
- The eight cage issues are not one movement problem. Two are dwarf-blocks-dwarf (#206, #189) and two are stone-blocks-dwarf
  (#189, #207; #189 is both), four are not movement (#186 wander, #187 job refusal, #203 dig, #204 stone placement), and one is the A\* budget
  on a map bigger than today's (#159).
- No single option fixes the movement set. Squeeze fixes #206 and #189, clamber fixes #189 and #207, and connectivity fixes
  none in the replays.

**Options, in order, and the stories they become** (expected cost from this file's numbers):

1. **pile-memo, no ruling needed** (story 13.3, small: the 25-line patch plus a test that a changed blocked cell beside the pile
   gives the fresh answer). Expected (`tickbench.txt` section 6, C0 against pile-memo, state the same
   on all 10): the tick falls 55–91% on the seven pile scenarios (plan-compact 1.39 → 0.15 ms, big-pile 1.05 → 0.09), moves −11% to
   +4% on the other three (dig-loop +4.3% and full-pile-crowd +3.2% are inside the 6% cross-build noise). At scale 128² crews 50/100/200 read 2.75/7.23/20.8 ms against 30.8/95.5/313,
   which puts a 200-dwarf crew at a fifth of the 100 ms budget. No hash moves. It also makes later A/B rows readable, because
   today a pile scenario's ms is mostly this one function.
2. **Squeeze-past, if Wolf rules the swap in** (Ruling 1; story 13.4, medium). It is the only option that fixes #206, the seat
   freeze, and it fixes #189 and takes #207 from 14 to 18 of 24. Expected cost: the swap adds no search (one lookup per blocked
   look). The ms moves only because the worlds differ (stuck-tunnel −71%, big-pile +34%). **The story must clear the #187
   regression before it lands**: 4 cage-sweep runs stuck at 16,000 ticks where base has none. It rewrites `step_and_watch` and the 4
   lib tests that pin "no swap".
3. **Clamber-over, only if Wolf rules a dwarf may stand on a stone** (Ruling 2; story 13.5, medium, after 13.4). It is the only
   option that fixes #207 fully. Expected cost: A\* nodes −37% to −69% in 8 of 10 scenarios, tick +6% to +35% in the changed
   worlds, #203's replay 11 → 14 marks, and the gui must draw a dwarf on a stone. Re-measure #207 on top of squeeze first: if 18 of
   24 is acceptable, or a hauler-placement fix closes it (#207 is "where the haulers stand"), clamber is not needed.
4. **No connectivity map now.** DF's terrain-only kind (T) answered 377 cheap searches and nothing expensive, because frostvein's
   cages are items and dwarves. The item-keyed kind (I) answers the expensive ones but costs more than it saves (every scenario
   slower, +9% to +421%) until it updates incrementally. It also does not fix #159 on 256², where most exhausted searches have a
   reachable goal past the budget. **#159 waits for the M4 map size.** If M4 goes past 128², its story sizes the A\* budget to the
   walkable component, or takes FFF #317's chunk-component heuristic (survey lead 3). That is the cheapest hierarchical idea, since it
   cannot make an answer wrong.
5. **The four non-movement issues go to their own stories, outside the movement layer:** #186 (`wander` on more than one level;
   the issue's probe already cut the sweep from 5/32 to 1/32), #187 (the let-go/refusal loop), #203 (a dig that removes a miner's
   only exit) and #204 (stone landings with the pile full). None of the three movement options fixed any of them.

**ADR 5** (`docs/technical-preferences.md`: "Plain A\* pathfinding on the voxel grid for milestone 1. Hierarchical pathfinding is
acknowledged future work, not scaffolded now."). The recommendation **keeps plain A\*** through M4 on 128². In every scenario over 0.1 ms and at every
128² scale point A\* is 0–3% of the tick, and the scale curve fits the budget once pile-memo lands. What it changes is the scope line. ADR 5
should say "plain A\* with a fixed node budget holds while the walkable component is under `MAX_ASTAR_NODES`", because past that
(256²: 64,300 cells) #159 is live. The first step past it is a budget or a heuristic, not a hierarchy. Hierarchical pathfinding stays
unscaffolded.

## Ruling

Pending Wolf.
