# Addendum — Frostvein Milestone 4 PRD

Depth that informs the PRD but doesn't belong in it. Source extract: `extract-inputs.md`.

## Why these jobs (Wolf asked for a recommendation, 2026-10-28)

| Job | Built on | Why it fits now |
|---|---|---|
| Forage | the `Cut` job shape (mark, walk, take, haul) | cheapest food source; a near copy of an existing job |
| Hunt | the movement layer (13.2–13.4) | animals give unordered motion and stress pathfinding at scale for real |
| Cook at the fire | the campfire, the pile | a raw → meal step gives haulers reasons to move between places |
| Stoke the fire | wood (`Cut`), the campfire, the torches | cold becomes demand on woodcutters, so the trades start depending on each other |
| Ice fishing | the frozen lake (one of the ice patches) | flavour that is frostvein's own (Wolf: include it) |

Considered and left for later milestones, because each is a whole system: building and
construction, crafting workshops, trading, combat, moods.

Placed fires (FR68) are the one exception to "no building" so that 200 dwarves can warm
themselves somewhere other than the camp.

Idle life comes mostly from the needs. A cold dwarf drifts to the fire, a hungry one goes to
the pile, and a tired one lies down. Idle-only behaviours are small additions on top.

## Scale data the targets rest on (13.2 § Scale, one run each, ms)

| map | crew | base mean | base p99 | pile-memo mean | pile-memo p99 | pile-memo max |
|---|---:|---:|---:|---:|---:|---:|
| 128² | 100 | 95.5 | 277.1 | 7.23 | 36.5 | 57.2 |
| 128² | 200 | 313 | 929.7 | 20.8 | 97.1 | 134.1 |
| 256² | 100 | 23.1 | 150.1 | 5.69 | 71.3 | 241.6 |
| 256² | 200 | 34.1 | 241.7 | 8.45 | 64.8 | 110.9 |

**Caveat:** at 256² nothing reached the pile within 2,000 ticks, so those points measure
map-spanning searches, not a full working fortress. The real 256² cost is unknown until FR54's
scenario does real work. That is why the scale story comes first.

### Measurement notes behind FR54 and NFR12

- **Why FR54 asks for real work:** 13.2's 256² points failed that test, because nothing reached
  the pile in 2,000 ticks.
- **Why NFR12 takes p99 as the median across runs:** a single run is not a reading. 13.2's p99
  for one point ranged 85–142 ms across runs.
- **Where the NFR12 bar starts from:** at 256² crew 200, 13.2 measured 8.5 ms mean and 65 ms p99
  with pile-memo, but before any work reached the pile.

After pile-memo, A* is the next cost at scale: 24% of the tick at 128² crew 200, and 40–67% at
256². On 256², the camp's walkable component is 64,300 cells against `MAX_ASTAR_NODES` 50,000.
Searches run out of budget at every 256² point (#159), and most of them are reachable goals
farther than the budget allows.

**Not yet measured anywhere:** the gui frame rate with more than five dwarves, and the cost of the
new per-tick systems (needs, animals, fires) at 200.

## What the code fixes at five today (main, 2026-10-28)

`Dims::DEFAULT` 128×128×32; `[Identity; 5]` and `[Profession; 5]`; 16 names and 5 colours;
`CAMP_RADIUS` 3 (a 7×7 camp, which panics past about 49 spawns); `MAX_ASTAR_NODES` 50,000.
13.2's throwaway `prototypes/scale.patch` (crew and dims settable from a scenario, identities
cycling `i % 5`, a BFS spawn spill) is the starting sketch for FR53. Its identity cycling falls
short of FR57.

## Movement and sim debt at 200 dwarves

- **Own stories already (13.2 Ruling 1, Wolf 2026-10-23):** #186 (wander stays on one z level),
  #187 (a refused channel loops; 13.4 folded in a fix, but the issue is still open, so check it
  first), #203 (a dig seals a miner in), #204 (stones seal two miners). Each already has a
  reproducer, so they do not wait for the scale run. They count toward the cap.
- **Decided by the #207 re-measure (Ruling 2):** #207 and 13.5 clamber. That ruling (Wolf,
  2026-10-23) replaces the M3-close retro's "the PRD is not final until clamber is ruled".
- **From 13.4, open:** #218 (a full pile seals pits), #220 (the squeeze timer carries across a
  change of case).
- **Scale-specific traps to check in the first story:** `MAX_DESIGNATIONS` is 4096, and the 13.2
  crew-200 scenario alone uses 1,800 marks. #170 drops cells silently at the cap, and #214
  silently accepts an off-map rect. Wander phases repeat past ten dwarves (`lib.rs:3626` NOTE).
  Professions cycle `i % 5` in `scale.patch`.
- **The next measurement at this scale:** the A* `BTreeMap` storage (13.2 proposal :121–124: "the
  first thing to measure if M4 goes to 256² with hundreds of dwarves").
- **Seat and UI:** the seat keymap is out of physically pressable keys (#118), and M4 adds Bevy
  commands. FR54's scripted seat run may need #50's unfinished half.

## Sketch of story order (input to epics, not binding)

Items 6 and 7 bundle several features each. Epics splits them into one-session stories.

1. Scale settings plus the 256² crew-200 bench scenario and a seat frame-rate run (FR53, FR54),
   re-measuring #207 for the 13.5 ruling.
2. #159 on 256² (FR55).
3. Arrivals and identity at scale (FR56, FR57).
4. Hunger, eating, forage (FR58, FR62): the first need with a food source.
5. Misery, shown at the seat (FR65), early for feel.
6. Fires that burn wood, stoking, cold (FR61's fuel half, FR63); sleep (FR64).
7. Animals and hunting (FR59); ice fishing (FR60); cooking (FR61's meal half).
8. Idle life (FR66), tuned at the seat.

FR67 (the fortress runs itself) is checked by every food and fuel story, not built once. FR68 (place a fire) joins item 6.

## Beyond M4's crew (direction, not scope)

Wolf, 2026-10-28: "let's try 200 and see where we end up … in the end we need to have even
more". M4 builds for 200 (tech preferences: build for the current need, "not the imagined
fortress of 500 dwarves"). What the 200 run measures, the tick tail and the frame rate, is the
evidence a later milestone uses to decide how much further the crew can grow.
