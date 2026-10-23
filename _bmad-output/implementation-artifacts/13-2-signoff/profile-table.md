# 13.2 profile table (Task 3)

Generated from `census-counters.txt` (`census.sh prototypes/counters.patch`, run 1 of 2; both runs print identical blocks)
on 4f57027 + census.patch + counters.patch. Counts only. The `mean ms` column is left EMPTY for the orchestrator's
`tick-bench` run (it must not run beside other builds).

Cost unit for `top cost`: A* **nodes expanded**, `route_to_nearest` **cells popped**, flood **cells visited**
(`pile_targets` cache hits are lookups, not cells; goal-checks are heuristic work inside A*, not extra cells).

## Summary

| scenario | ticks | mean ms | A* calls | A* nodes | A* goal-checks | route cells | flood cells | top cost (nodes+cells) |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| big-pile | 5006 |  | 3,898 | 121,371 | 268,131 | 686 | 274,861 | `flood:caging_stone` 266,549 (67% of 396,918) |
| deep-pit | 3000 |  | 1,321 | 66,654 | 147,464 | 120 | 116,140 | `flood:caging_stone` 114,393 (63% of 182,914) |
| dig-loop | 17640 |  | 23,050 | 80,858 | 162,319 | 4,763 | 206,718 | `flood:caging_stone` 203,434 (70% of 292,339) |
| full-pile-crowd | 10006 |  | 31,518 | 128,097 | 291,208 | 1,476 | 402,947 | `flood:caging_stone` 392,327 (74% of 532,520) |
| plan-compact-piles | 10006 |  | 3,930 | 147,298 | 549,905 | 745 | 380,107 | `flood:caging_stone` 354,925 (67% of 528,150) |
| plan-far-piles | 10006 |  | 4,475 | 163,946 | 473,556 | 1,002 | 399,646 | `flood:caging_stone` 380,646 (67% of 564,594) |
| plan-spread-piles | 10006 |  | 4,216 | 174,304 | 929,166 | 763 | 441,787 | `flood:caging_stone` 363,767 (59% of 616,854) |
| plan | 10006 |  | 4,475 | 163,946 | 459,228 | 1,002 | 393,386 | `flood:caging_stone` 380,646 (68% of 558,334) |
| seat-slow | 56759 |  | 33 | 19,467 | 76,098 | 16 | 30,328 | `astar:claim_goals` 18,529 (37% of 49,811) |
| stuck-tunnel | 28082 |  | 11,033 | 1,160,963 | 2,608,072 | 1,913 | 476,826 | `astar:execute_replan` 1,106,988 (68% of 1,639,702) |

### big-pile

End: tick=5006 marks=14 items=32 zones=518 goals=0/2/0

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 291 | 118 | 742 | 24 | 0 | 0 | 267 |
| claim_goals | 368 | 49,693 | 104,450 | 58 | 3 | 0 | 307 |
| clearing_kept_landing | 3,026 | 18,876 | 46,233 | 3,026 | 0 | 0 | 0 |
| execute_replan | 196 | 20,237 | 49,924 | 195 | 1 | 0 | 0 |
| resolve_reroute | 17 | 32,447 | 66,782 | 15 | 2 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 98 | 657 | 98 | 0 |
| resolve_blocker_escape | 14 | 14 | 14 | 0 |
| resolve_holder_escape | 14 | 15 | 14 | 0 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 5,847 | 266,549 | pockets=5,919 |
| nearest_free_cell | 93 | 666 |  |
| pile_targets | 51,361 | 120 | cache_hits=95,898, cache_misses=49 |
| stranded | 858 | 7,526 |  |

Guard calls: closes_last_level_exit=3,422, walls_in_a_dwarf=522. Emitter: boundaries=5,006 shares=0 swaps=0.

**Top cost:** `flood:caging_stone` 266,549 (67% of 396,918).

### deep-pit

End: tick=3000 marks=0 items=28 zones=22 goals=0/0/0

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 714 | 113 | 2,797 | 29 | 0 | 0 | 685 |
| claim_goals | 61 | 300 | 2,702 | 57 | 0 | 0 | 4 |
| clearing_kept_landing | 419 | 1,315 | 4,081 | 419 | 0 | 0 | 0 |
| execute_replan | 104 | 610 | 8,187 | 103 | 0 | 0 | 1 |
| resolve_reroute | 23 | 64,316 | 129,697 | 13 | 10 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 25 | 110 | 25 | 0 |
| resolve_blocker_escape | 3 | 3 | 3 | 0 |
| resolve_exit | 2 | 4 | 2 | 0 |
| resolve_holder_escape | 3 | 3 | 3 | 0 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 2,489 | 114,393 | pockets=2,489 |
| nearest_free_cell | 54 | 231 |  |
| pile_targets | 10,922 | 235 | cache_hits=83,050, cache_misses=235 |
| stranded | 216 | 1,281 |  |

Guard calls: closes_last_level_exit=555, walls_in_a_dwarf=147, walls_in_a_dwarf_from=32. Emitter: boundaries=3,000 shares=0 swaps=0.

**Top cost:** `flood:caging_stone` 114,393 (63% of 182,914).

### dig-loop

End: tick=17640 marks=11 items=44 zones=21 goals=0/0/0

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 22,491 | 0 | 0 | 0 | 0 | 0 | 22,491 |
| claim_goals | 455 | 80,428 | 161,276 | 106 | 112 | 0 | 237 |
| clearing_kept_landing | 97 | 373 | 833 | 97 | 0 | 0 | 0 |
| execute_replan | 7 | 57 | 210 | 6 | 1 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 4 | 26 | 4 | 0 |
| make_room | 1 | 2 | 0 | 1 |
| resolve_back_out | 1,577 | 1,577 | 0 | 1,577 |
| resolve_back_up | 1 | 2 | 0 | 1 |
| resolve_blocker_escape | 1,577 | 1,577 | 0 | 1,577 |
| resolve_exit | 1 | 2 | 0 | 1 |
| resolve_holder_escape | 1,577 | 1,577 | 0 | 1,577 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 9,727 | 203,434 | pockets=44,612 |
| nearest_free_cell | 1,414 | 2,852 |  |
| pile_targets | 22,682 | 0 |  |
| stranded | 75 | 432 |  |

Guard calls: closes_last_level_exit=121, walls_in_a_dwarf=42, walls_in_a_dwarf_from=10. Emitter: boundaries=10,000 shares=0 swaps=0.

**Top cost:** `flood:caging_stone` 203,434 (70% of 292,339).

### full-pile-crowd

End: tick=10006 marks=0 items=46 zones=4 goals=0/0/0

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 26,894 | 24 | 523 | 6 | 0 | 0 | 26,888 |
| claim_goals | 465 | 48,962 | 99,632 | 52 | 3 | 0 | 410 |
| clearing_kept_landing | 3,999 | 43,006 | 109,173 | 3,999 | 0 | 0 | 0 |
| execute_replan | 158 | 20,049 | 49,718 | 156 | 1 | 0 | 1 |
| resolve_reroute | 2 | 16,056 | 32,162 | 1 | 1 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 91 | 1,350 | 91 | 0 |
| resolve_blocker_escape | 11 | 84 | 2 | 9 |
| resolve_exit | 9 | 22 | 9 | 0 |
| resolve_holder_escape | 11 | 20 | 11 | 0 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 8,328 | 392,327 | pockets=8,346 |
| nearest_free_cell | 91 | 1,465 |  |
| pile_targets | 36,626 | 47 | cache_hits=16,433, cache_misses=33 |
| stranded | 1,120 | 9,108 |  |

Guard calls: closes_last_level_exit=4,352, walls_in_a_dwarf=632. Emitter: boundaries=10,006 shares=0 swaps=0.

**Top cost:** `flood:caging_stone` 392,327 (74% of 532,520).

### plan-compact-piles

End: tick=10006 marks=0 items=58 zones=203 goals=12/6/1

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 54 | 563 | 40,533 | 54 | 0 | 0 | 0 |
| claim_goals | 417 | 85,221 | 187,154 | 103 | 5 | 0 | 309 |
| clearing_kept_landing | 3,160 | 21,650 | 51,736 | 3,160 | 0 | 0 | 0 |
| execute_replan | 292 | 23,374 | 230,987 | 291 | 1 | 0 | 0 |
| resolve_reroute | 7 | 16,490 | 39,495 | 6 | 1 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 96 | 723 | 96 | 0 |
| resolve_blocker_escape | 11 | 11 | 11 | 0 |
| resolve_holder_escape | 11 | 11 | 11 | 0 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 7,779 | 354,925 | pockets=7,809 |
| nearest_free_cell | 94 | 816 |  |
| pile_targets | 233,692 | 15,582 | cache_hits=20,563,230, cache_misses=1,666 |
| stranded | 1,213 | 8,784 |  |

Guard calls: closes_last_level_exit=3,666, walls_in_a_dwarf=751. Emitter: boundaries=10,006 shares=0 swaps=0.

**Top cost:** `flood:caging_stone` 354,925 (67% of 528,150).

### plan-far-piles

End: tick=10006 marks=0 items=58 zones=90 goals=12/10/1

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 61 | 313 | 9,296 | 61 | 0 | 0 | 0 |
| claim_goals | 458 | 83,060 | 176,776 | 109 | 5 | 0 | 344 |
| clearing_kept_landing | 3,606 | 24,272 | 58,057 | 3,606 | 0 | 0 | 0 |
| execute_replan | 327 | 23,933 | 157,297 | 326 | 1 | 0 | 0 |
| resolve_reroute | 23 | 32,368 | 72,130 | 21 | 2 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 106 | 784 | 106 | 0 |
| resolve_blocker_escape | 27 | 139 | 11 | 16 |
| resolve_exit | 16 | 37 | 16 | 0 |
| resolve_holder_escape | 27 | 42 | 27 | 0 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 8,343 | 380,646 | pockets=8,391 |
| nearest_free_cell | 103 | 870 |  |
| pile_targets | 165,478 | 8,164 | cache_hits=5,952,309, cache_misses=881 |
| stranded | 1,312 | 9,966 |  |

Guard calls: closes_last_level_exit=4,131, walls_in_a_dwarf=800. Emitter: boundaries=10,006 shares=0 swaps=0.

**Top cost:** `flood:caging_stone` 380,646 (67% of 564,594).

### plan-spread-piles

End: tick=10006 marks=0 items=58 zones=203 goals=12/2/0

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 45 | 1,089 | 68,884 | 45 | 0 | 0 | 0 |
| claim_goals | 416 | 88,700 | 207,942 | 93 | 5 | 0 | 318 |
| clearing_kept_landing | 3,431 | 22,842 | 53,532 | 3,431 | 0 | 0 | 0 |
| execute_replan | 290 | 28,994 | 480,266 | 289 | 1 | 0 | 0 |
| resolve_reroute | 34 | 32,679 | 118,542 | 32 | 2 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 103 | 747 | 103 | 0 |
| resolve_blocker_escape | 8 | 8 | 8 | 0 |
| resolve_holder_escape | 8 | 8 | 8 | 0 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 7,912 | 363,767 | pockets=7,954 |
| nearest_free_cell | 100 | 845 |  |
| pile_targets | 281,796 | 67,443 | cache_hits=23,951,680, cache_misses=980 |
| stranded | 1,253 | 9,732 |  |

Guard calls: closes_last_level_exit=3,929, walls_in_a_dwarf=760. Emitter: boundaries=10,006 shares=0 swaps=0.

**Top cost:** `flood:caging_stone` 363,767 (59% of 616,854).

### plan

End: tick=10006 marks=0 items=58 zones=78 goals=12/10/1

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 61 | 313 | 8,648 | 61 | 0 | 0 | 0 |
| claim_goals | 458 | 83,060 | 176,776 | 109 | 5 | 0 | 344 |
| clearing_kept_landing | 3,606 | 24,272 | 58,057 | 3,606 | 0 | 0 | 0 |
| execute_replan | 327 | 23,933 | 144,193 | 326 | 1 | 0 | 0 |
| resolve_reroute | 23 | 32,368 | 71,554 | 21 | 2 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 106 | 784 | 106 | 0 |
| resolve_blocker_escape | 27 | 139 | 11 | 16 |
| resolve_exit | 16 | 37 | 16 | 0 |
| resolve_holder_escape | 27 | 42 | 27 | 0 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 8,343 | 380,646 | pockets=8,391 |
| nearest_free_cell | 103 | 870 |  |
| pile_targets | 165,478 | 1,904 | cache_hits=3,966,867, cache_misses=587 |
| stranded | 1,312 | 9,966 |  |

Guard calls: closes_last_level_exit=4,131, walls_in_a_dwarf=800. Emitter: boundaries=10,006 shares=0 swaps=0.

**Top cost:** `flood:caging_stone` 380,646 (68% of 558,334).

### seat-slow

End: tick=56759 marks=1 items=129 zones=222 goals=38/12/1

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 10 | 201 | 761 | 10 | 0 | 0 | 0 |
| claim_goals | 10 | 18,529 | 70,695 | 7 | 1 | 0 | 2 |
| execute_replan | 11 | 715 | 4,576 | 11 | 0 | 0 | 0 |
| resolve_reroute | 2 | 22 | 66 | 0 | 2 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| resolve_blocker_escape | 2 | 12 | 2 | 0 |
| resolve_holder_escape | 2 | 4 | 2 | 0 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| nearest_free_cell | 3,000 | 15,000 |  |
| pile_targets | 48,777 | 56 | cache_hits=2,005,503, cache_misses=27 |
| stranded | 6,021 | 15,272 |  |

Guard calls: closes_last_level_exit=3,015, walls_in_a_dwarf=3,018. Emitter: boundaries=3,000 shares=0 swaps=0.

**Top cost:** `astar:claim_goals` 18,529 (37% of 49,811).

### stuck-tunnel

End: tick=28082 marks=3 items=57 zones=89 goals=38/4/0

A* per caller

| caller | calls | nodes | goal-checks | found | nopath | budget | empty |
|---|---:|---:|---:|---:|---:|---:|---:|
| claim_delivery | 1,012 | 12,938 | 293,552 | 1,012 | 0 | 0 | 0 |
| claim_goals | 1,150 | 3,695 | 16,243 | 97 | 0 | 0 | 1,053 |
| clearing_kept_landing | 8,507 | 37,293 | 83,632 | 8,507 | 0 | 0 | 0 |
| execute_replan | 315 | 1,106,988 | 2,214,547 | 246 | 69 | 0 | 0 |
| resolve_reroute | 49 | 49 | 98 | 0 | 49 | 0 | 0 |

`route_to_nearest` per caller

| caller | calls | cells popped | found | none |
|---|---:|---:|---:|---:|
| clearing_landing | 305 | 1,401 | 305 | 0 |
| resolve_blocker_escape | 51 | 275 | 24 | 27 |
| resolve_exit | 1 | 2 | 1 | 0 |
| resolve_holder_escape | 51 | 235 | 27 | 24 |

Floods, and guard calls

| flood | calls | cells | other |
|---|---:|---:|---|
| caging_stone | 9,337 | 459,901 | pockets=9,985 |
| nearest_free_cell | 305 | 1,644 |  |
| pile_targets | 1,182 | 22 | cache_hits=21,261, cache_misses=15 |
| stranded | 2,003 | 15,259 |  |

Guard calls: closes_last_level_exit=9,460, walls_in_a_dwarf=1,154. Emitter: boundaries=10,000 shares=0 swaps=0.

**Top cost:** `astar:execute_replan` 1,106,988 (68% of 1,639,702).

