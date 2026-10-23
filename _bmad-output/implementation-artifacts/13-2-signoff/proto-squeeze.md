# 13.2 Task 6: squeeze-past (`prototypes/squeeze.patch`)

Stack: `4f57027` + `census.patch` + `counters.patch` + `squeeze.patch` (`git apply --check` clean in that order).
Command: `13-2-signoff/census.sh prototypes/counters.patch prototypes/squeeze.patch` -> `census-squeeze.txt`.
Base column = `census-counters.txt` (same stack without squeeze). Counts only; no ms (other builds were running).

## What the patch does (lib.rs only, +113 lines)

- `try_squeeze` in `resolve_blocked_step`: a holder stuck behind the same blocker for `SQUEEZE_WAIT_TICKS = 10`
  (= `STEP_REST_TICKS`, one step period: the holder's second look) swaps with it. Both `Pos` are written inside one
  call, so no tick boundary has two dwarves on a tile. Per-holder timer in a throwaway `Squeeze` resource (not saved,
  not hashed).
- Who may swap (const `SQUEEZE_HEAD_ON_MOVERS`):
  - **B (the patch as committed, `true`)**: two head-on movers (blocker holds a job and its path starts on the
    holder's tile) swap INSTEAD of the escape ladder (#196's yield); an idle blocker that has no tile aside and no
    exit swaps after the wait; head-on with neither escape (#194/#196 back-out) swaps after the wait, and the back-out is
    skipped.
  - **A (`false`)**: swap only where today's ladder returns "wait". Measured too (below).
  - Not swapped: a blocker that holds a job and is NOT coming the other way (a worker or a follower). He would be
    pulled off his work tile / out of line. Left as is: reroute, else wait.
- Both rest `STEP_REST_TICKS` after the swap (the caller's wait path stores the holder's advanced `Path` and cooldown;
  the blocker's cooldown and `Path` are set in `try_squeeze`). An idle blocker's `home` becomes the new tile.
- Swap works across z (ramps): `astar_neighbours` is symmetric.

### Every writer of a dwarf's `Pos`, and its `occupied`

| writer | `occupied` | with the swap |
|---|---|---|
| `execute_jobs` ordinary step (`:2343`) | `remove(pos)`, `insert(next)` | unchanged. A swap makes `resolve_blocked_step` return `false` (wait), so this step is skipped and `occupied` is not touched |
| `execute_jobs` via `step_aside` / the blocker-escape step in `resolve_blocked_step` | updated there | unchanged |
| `execute_jobs` via `try_squeeze` (NEW) | none needed: the same two tiles stay taken | the set is the same before and after |
| `settle` (fall landing) | own set built from components at its start | sees the swapped positions |
| `wander` (idle step, idle exit path) | own set built from components at its start | sees the swapped positions |
| `carry_items` | items only | follows the carrier last, as today |

Occupancy as the guards (`walls_in_a_dwarf`, `closes_last_level_exit`, `caging_stone`) and jobs read it is unchanged:
the same set of tiles is taken before and after. No guard, job rule or claim was edited.

## AC 5 emitter: shares=0 everywhere, RED run

- Every counters block and every sweep line: `shares=0` (14 of 14 `emitter:ticks` lines in `census-squeeze.txt`).
- `swaps > 0` (that is the point): big-pile 8, deep-pit 6, dig-loop 2, full-pile-crowd 2, plan 9, plan-compact 14,
  plan-far 9, plan-spread 9, seat-slow 3, stuck-tunnel 7; sweeps: cage 195, one-wide staircase 101, two-wide staircase
  196, head-on 1.
- RED (the emitter detects a shared tile): sabotage = `try_squeeze` writes the blocker's `Pos` but not the holder's.
  Observed: `stuck-tunnel ... shares=64 swaps=0`, `plan ... shares=127 swaps=0`; unsabotaged `shares=0 swaps=6` / `shares=0 swaps=9` (first cut of the patch; the final census below has shares=0 everywhere).
- Determinism: every counters block `IDENTICAL across 2 runs` (10 of 10).

## Census: base vs squeeze (B)

Reproducer end lines (`bench NAME end ...`) and state hash:

| scenario | base end | squeeze end | state hash |
|---|---|---|---|
| big-pile | marks=14 items=32 goals=0/2/0 | marks=**12** items=34 goals=0/**1**/0 | changed |
| deep-pit | marks=0 items=28 | marks=0 items=**27** | changed |
| dig-loop (#203) | marks=11 items=44 | marks=**10** items=44 | changed |
| full-pile-crowd | marks=0 items=46 | marks=0 items=46 | changed |
| plan | marks=0 items=58 goals=12/10/1 | same | changed |
| plan-compact-piles | marks=0 items=58 goals=12/6/1 | same | changed |
| plan-far-piles | marks=0 items=58 goals=12/10/1 | same | changed |
| plan-spread-piles | marks=0 items=58 goals=12/2/0 | same | changed |
| seat-slow | marks=1 items=129 goals=38/12/1 | marks=1 items=129 goals=38/**11**/1 | changed |
| stuck-tunnel (#206) | marks=**3** items=57 goals=38/4/**0** | marks=**0** items=60 goals=38/4/**1** | changed |

Sweeps (from the `census ...` lines):

| reading | base | squeeze B |
|---|---|---|
| cage `frozen` | [(10,6,0), (12,4,1), (12,4,4), (12,6,3)] | [(2,6,1), (7,6,3), (8,6,4), (12,4,4), (12,6,3)] (2 kept, 2 gone, 3 new) |
| cage `unfinished` @4000 | [(11,6,1)] | [(0,4,1), (6,6,1), (9,6,1), (12,6,6), (13,6,8)] |
| cage `unfinished` @16000 (scratch run, TICKS=16_000) | [] | [(0,4,1), (6,6,1), (9,6,1), (13,6,1)]: stuck, not slow |
| cage `landed` / runs / worked | [] / 32 / 32 | [] / 32 / 32 |
| two-wide staircase `worked` | 24/24, stalled [] | 24/24, stalled [] |
| one-wide staircase `failed` | [] levels 96/96, swaps=0 | [] levels 96/96, **swaps=101** |
| head-on levels | 4/4 marks_left=0 swaps=0 | 4/4 marks_left=0 **swaps=1** |
| campfire-189 | pickups_after_t2500=80 stored=23/24 | pickups **0** stored **24/24** |
| ring (4 readings) | 20/20, 20/20, 14/24 (retry_ticks 1964), 19/19 | 20/20, 20/20, **18/24** (free=6, last rise t5989), 19/19 |
| pocket207 | 24/24, 12/12, 12/12 | same |

Counts (sum over callers; base -> squeeze):

| scenario | A* nodes | A* calls | route cells | flood cells |
|---|---|---|---|---|
| big-pile | 121,371 -> **904,982** | 3,898 -> 3,376 | 686 -> 551 | 274,861 -> 251,902 |
| deep-pit | 66,654 -> 82,661 | 1,321 -> 1,190 | 120 -> 112 | 116,140 -> 116,394 |
| dig-loop | 80,858 -> 82,072 | 23,050 -> 27,714 | 4,763 -> 44 | 206,718 -> 52,740 |
| full-pile-crowd | 128,097 -> 111,884 | 31,518 -> 31,498 | 1,476 -> 1,465 | 402,947 -> 403,488 |
| plan | 163,946 -> 127,708 | 4,475 -> 3,843 | 1,002 -> 1,005 | 393,386 -> 375,730 |
| plan-compact | 147,298 -> 152,789 | 3,930 -> 5,897 | 745 -> 1,311 | 380,107 -> 391,122 |
| plan-far | 163,946 -> 127,708 | 4,475 -> 3,843 | 1,002 -> 1,005 | 399,646 -> 382,002 |
| plan-spread | 174,304 -> 135,306 | 4,216 -> 3,670 | 763 -> 855 | 441,787 -> 438,981 |
| seat-slow | 19,467 -> 19,317 | 33 -> 25 | 16 -> 0 | 30,328 -> 30,292 |
| stuck-tunnel | 1,160,963 -> **20,073** | 11,033 -> 392 | 1,913 -> 220 | 476,826 -> 32,580 |

Reading the counts: the swap itself adds no search (one hash lookup per blocked look). Every delta is the world
being different. stuck-tunnel's collapse (-98% A* nodes) is the livelock no longer running: `execute_replan` was 1.1M
nodes of the base's 1.16M. big-pile's 7x comes from `claim_goals` (49,693 -> 789,149 nodes, nopath 3 -> 49): a
different stone layout leaves more jobs unreachable to scan. That is an outcome change, not swap cost.

### Variant A (`SQUEEZE_HEAD_ON_MOVERS = false`) (measured on the first cut of the patch, before the #183 stale-path fix, which only touches the head-on-with-no-escape arm), same command with the flag flipped (`/tmp/.../census-squeeze-fallback.txt`, not committed)

Nearly inert: only dig-loop moves (marks 11 -> 10, state changed); stuck-tunnel state hash = base (marks=3); cage,
campfire-189, ring, head-on, staircases identical to base except one-wide staircase swaps=4. The 4 swaps and dig-loop's 2
show where the ladder waits today. #206 is not a "ladder says wait" case. Its two job-holders head-on in the tunnel
keep resolving through the yield (`resolve_blocked_step` returns true), 1.1M A* nodes of replanning, and never get past.
So the head-on-mover swap (B) is the one that matters.

## stuck-tunnel (#206)

Base: marks=3, both job-holders (ids 2 and 4) walk the one-wide tunnel x=77, y=64..71 back and forth forever (t20000
at y=67/66, t22000 y=69/68, t26000 y=71/70, t28000 y=71/67). B: marks=0, goals 38/4/1, and at t24000/26000/28000 all
five dwarves are idle (id 2 at (77,64,10)/(76,63,10), id 4 at (79,71,10)/(78,71,10)/(79,72,10), the other three at z15 and z9).
7 swaps in the run.

## Issue cells (B)

| issue | cell | number |
|---|---|---|
| #159 | not movement (claim_jobs A* budget starvation) | needs scale; no change expected |
| #186 | worse / reshuffled | cage `frozen` 4 -> 5, only 2 of the 4 base entries remain. The shape is an idle dwarf on a one-level island (wander is same-z): not movement. Different timing moves which runs hit it |
| #187 | worse | cage `unfinished` @4000 1 -> 5 entries; @16000 0 -> 4 runs with 1 mark left (stuck). The one I read, seed 0 width 4, is a pit with the pile full: dwarf 1 `Work` at (68,65,11) beside the mark (68,64,12), stones banked at (67,64..66,11) and (66,x,11). I did not prove which swap led there |
| #189 | **fixed** | campfire-189 pickups_after_t2500 80 -> 0, stored 23/24 -> 24/24. The boxed-in dwarf now trades out with the hauler |
| #203 | unchanged in kind | dig-loop marks 11 -> 10. Still sealed in a hole by terrain, no movement fix |
| #204 | unchanged / slightly worse | seat-slow marks 1 -> 1 with goals 38/12/1 -> 38/11/1; full-pile-crowd marks 0 -> 0 items 46 -> 46. The issue's four-stone pocket is a seal, and a swap does not open it |
| #206 | **fixed** | stuck-tunnel marks 3 -> 0, goals 38/4/0 -> 38/4/1, A* nodes 1.16M -> 20.1k |
| #207 | improved, not fixed | ring census_207 14/24 -> 18/24, last rise t1748 -> t5989, free cells 10 -> 6 (haulers still wall the hole's last way out) |

## Rule departures (candidate "Rulings needed" lines)

1. **A swap.** Two dwarves trade tiles in one step after a one-step-period wait (10 ticks). Today: "no swap" (12.9
   Task 0 Q1b, `resolve_blocked_step` NOTE). Buys: #206 fixed, #189 fixed, #207 14 -> 18 of 24.
2. **`step_and_watch` and the strict one-per-tile / no-swap checks must permit a swap.** `census.patch` already turned the
   `assert_eq!` on a swap into a count. Shipping needs that check rewritten to accept a swap and still fail on two on a
   tile (`shares` stays 0). Measured: `cargo test --release -p sim-core --lib` with the patch fails 4 of 98 (`head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_{dead,open}_end`, `a_dead_end_miner_behind_idle_woodcutters_finishes_its_dig`, `a_3x3_pile_fills_from_the_inside_out`), each on the shared trace helper's `dwarves 0 and 1 swapped tiles` assert (`lib.rs` ~:7198). Affected tests: the one-wide staircase (101 swaps) and head-on (1 swap), plus the unit tests that
   pin "no swap" in `lib.rs`.
3. **Head-on movers swap instead of yielding** (B). Replaces #196's holder/blocker escape for two job-holders. The escape
   ladder still handles an idle blocker who can step aside or leave.
4. **The #194/#196 back-out is skipped** when both are stuck head-on (the arm keeps #183's drop of an idle blocker's stale exit path; the back-out code is deleted). The squeeze takes its place.
5. **Idle blocker with no way out swaps** after the wait, and its `home` moves to the holder's old tile.
6. **Not changed:** the wall-in guards, job logic, claim logic, occupancy as they read it.

## Cost of the ruling (judgement)

B fixes the swap family's two reproducers (#206, #189) and half-moves #207, but regresses the cage sweep: 4 runs
stuck at 16k ticks where base has 0, and `frozen` reshuffles. I did not isolate whether a swap causes those four or whether
any timing change would (a timing change that large on a 32-run sweep that was re-pinned five times in Epic 12 is
plausible), but A leaves the sweep byte-identical, so B's swaps are what perturb it. A ruling would need that sweep read
again with the swap rule in.

## Blockers / notes

- No blocker. The 16k cage readings used a scratch edit (`TICKS = 16_000`) that is not in the patch.
- `scale.sh` exists: see `scale-squeeze.txt` (appended when the run finished).

## Scale (`scale.sh prototypes/squeeze.patch` -> `scale-squeeze.txt`, PARTIAL)

Killed after 4 of 10 points (crew 200 and the 256 maps not run; ms is noise while other builds run, so only counts are quoted). share_ticks=0 at all four.
128x128: crew005 A* nodes 271,479 marks=24; crew020 851,719 marks=126; crew050 2,062,684 marks=338 goals=5/0/0; crew100 7,305,834 marks=748 goals=6/0/0. Opus must run base and the rest.
