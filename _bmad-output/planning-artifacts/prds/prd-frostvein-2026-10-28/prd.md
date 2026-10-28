---
title: Frostvein Milestone 4 — A Living Fortress PRD
status: final
created: 2026-10-28
updated: 2026-10-28
---

# Frostvein Milestone 4 — A Living Fortress

*(Working title; Epic 13 takes this name when the PRD is final.)*

Inherits the Milestone 1, 2 and 3 PRDs (`prd-frostvein-2026-08-01`, `-08-09`, `-09-28`, all
final) by reference: the four-crate spine, determinism, YAGNI-as-policy, NFR5 (no drift), the
parity rule as NFR10 narrowed it, NFR11 (refuse loudly), and M2's Visual Target all stand, except
where **What M4 changes** below says otherwise. ADR 5 as amended by the 13.2 ruling stands.

## Vision

Milestone 3 made the valley worth playing: five dwarves with names and trades, working side by
side towards a short chain of goals. Milestone 4 makes it **alive**: a crowded fortress on a
valley four times the size, which keeps moving when the player stops giving orders. Dwarves
arrive over time until there are two hundred. The frozen valley's cold and hunger, and the need
to sleep, give them reasons to act: they forage, hunt, fish through the lake ice, cook at the
fire, carry wood to keep it burning, and bed down when they are spent. A dwarf left cold, hungry
or exhausted is visibly miserable and stops working. An idle dwarf is not a statue. It does
stupid things: it wanders, huddles at the fire, follows an animal, and dozes in the snow.

The genre is the reference, but frostvein stays its own game, icy and grim. Every new system is
chosen because it grows out of something the valley already has: trees, the frozen lake, the
campfire and the pile.

**The done sentence:** Milestone 4 is done when Wolf can start a fresh 256² world, set up its
standing work once, watch the crew grow by arrivals to two hundred, and sit at the seat **without
giving another order** while the fortress feeds and warms itself, visibly and without being
boring. Neglect shows: starve the pile or let the fires die, and the dwarves turn miserable and
stop work until that is put right.

## Scope shape

- **In:** crew and map size as real world settings, with arrivals up to 200 dwarves on 256²;
  hunger, cold and sleep; food from foraging, hunting and ice fishing, cooked at the fire; fires
  that burn wood and are stoked; misery that refuses work; idle life; the movement and
  pathfinding work that the target scale makes live (#159 first).
- **Planning shape:** one M4 epic (Epic 13), growing by stories. 13.1–13.4 are already in it.
- **Scale first (Wolf, 2026-10-28):** performance is tested at the target scale **from the first
  M4 game story on**, not checked at the end. That story adds the crew and map settings and the
  256² crew-200 bench scenario (FR53, FR54). Every later story runs against it.
- **Instrument per kind of work:** sim work is judged by deterministic scenario tests and the
  tick bench. Feel (misery, idle life, "not boring") reaches Wolf's seat early instead of being
  argued from screenshots.
- **Baseline:** today's worldgen grown to 256², today's three trades, and today's movement layer
  (pile-memo and squeeze-past merged). Clamber (13.5) is decided as 13.2's Ruling 2 says (Wolf,
  2026-10-23): #207 is re-measured on its own sweep, on top of 13.4.
- **Art: plain first (Wolf, 2026-10-28).** Animals, plants, fishing, sleep and misery get simple
  models and readable poses, driven by sim state. Every round follows the M2-24 hard stop. Look
  tuning waits for a concrete defect judged at the seat.
- **Tuning:** every need rate, threshold, regrowth time and arrival cadence is a hardcoded
  constant, tuned in its story.

## What M4 changes

These are the inherited rules M4 reopens or narrows, stated here so that nothing overrides them
silently:

- **M3 out-of-scope "no new dwarves, the count stays five"**: reopened. The crew grows by
  arrivals up to 200 (FR56).
- **World size `Dims::DEFAULT` 128×128×32** (M1 baseline): the M4 target is 256×256×32 (FR53).
- **M3 out-of-scope "no needs"**: reopened for **hunger, cold and sleep only**. There are still no
  moods, no fail state and no death (FR65).
- **M3 out-of-scope "resources are not used"**: reopened for **eating food and burning wood
  only**. There is still no crafting or trading, and no building except placing a fire (FR68).
- **M3 FR38 "a distinct colouring" per dwarf, and FR39 "tell which dwarf is which at a
  glance"**: narrowed. At 200 dwarves, names stay unique within the crew, but colours repeat.
  A dwarf is identified by selecting it (FR57).
- **M3 FR43 "the five always include at least one of each trade"**: becomes six trades. The
  embark group includes at least one dwarf of each of the six (FR56).
- **M3 FR47 goal progress**: progress is the current stockpile total, so burning wood can lower
  the wood goal's progress. A completed goal stays completed.
- **M3 FR49** stands as amended by 13.4: two dwarves meeting head-on may swap tiles.
- **M3 FR50 "no item left on a light emitter"**: stands. Stoked wood is consumed into the fire and
  is never left on it as an item.
- **M1 non-goals "no temperature" and "nothing from off the map"**: reopened, for cold (FR63) and
  for arrivals and animals entering at the valley edge (FR56, FR59).
- **M3 FR42 "an idle dwarf … does not help with other trades"**: needs come before trades. A
  hungry, cold or tired dwarf of any trade stops to eat, warm itself or sleep (FR62, FR63, FR64).
  Idle time gains behaviours beyond wandering (FR66).
- **The campfire is always lit** (M2 lighting): a fire now burns wood and goes out without it
  (FR61).

## Features & Functional Requirements — Milestone 4

These describe capabilities, not implementation. FR IDs continue the global numbering (M3 ended
at FR52). Feature groups continue at F18.

### F18. A crowd on a bigger valley (first)

- **FR53**: The starting crew size and the map size are world settings: seeded, deterministic,
  saved, and settable from a scenario file. Worldgen produces a playable 256×256×32 valley, and a
  fresh game defaults to it. The starting crew spawns on walkable ground near the camp even when
  there are more dwarves than the camp ring holds. Arrivals (FR56) run on any map size. The story
  that adds the settings keeps every committed scenario's outcome unchanged. Later stories that
  add worldgen content (plants, animals) re-pin old scenarios, each with its reason on the record.
- **FR54**: The committed scenario set gains a **256² crew-200 scenario that does real work**.
  The test is checkable: within the bench window, items are delivered to the pile and every job
  kind in the scenario completes at least once. The tick bench reads the scenario from the first
  M4 game story, and every later story reports its delta against `main` there. A seat run of the
  same scenario records the gui frame rate with 200 dwarves, with the daemon's wire traffic in
  the loop. No terrain-change rate is set as a target: the bench reports whatever rate 200
  dwarves in the working mix produce, as cells changed per window.
- **FR55** (#159): On 256², a reachable job is never starved because the walkable area exceeds
  the path-search budget. ADR 5's amendment applies: the budget is sized to the walkable
  component or the heuristic is improved, with no hierarchy. Red first, as a scenario test.

### F19. Arrivals

- **FR56**: New dwarves arrive at the valley edge during play, in groups of about 5–20, until the
  crew reaches 200. The first group arrives at embark. Each arrival walks to the camp, has an
  identity and a profession seeded towards a working mix, and joins the work. The cadence is a
  hardcoded constant tuned in the story.
- **FR57**: Every dwarf has a name that is unique within the crew. Names can be combinations
  such as a first name and an epithet, so 200 never run out. Colours repeat. The player can still
  find a given dwarf: selecting one names it, and the TUI shows names. No crew list or roster is
  planned. Whether 200 dwarves need one is judged at the seat once arrivals land (Wolf).

### F20. Food

Three trades are added (hunter, fisher and cook), for six professions in all.

- **FR58**: Worldgen places forageable plants on the surface, seeded. Woodcutters forage the
  plants in a designated forage area, again and again as they regrow. Each foraged plant yields
  raw food, which is hauled to the pile.
- **FR59**: Animals live in the valley. They wander, and they enter at the edges over time, so
  game is renewable. A hunter tracks a designated animal or the nearest animal, kills it, and
  the carcass is hauled to the pile. Animals are harmless and do not fight back.
- **FR60**: On the frozen lake, a fisher works a fishing hole designated on lake ice and catches
  fish at a slow, steady rate. The hole stays until the player removes it. Fishing off the ice is
  refused loudly (NFR11).
- **FR61**: A cook turns raw food (plants, carcasses, fish) into meals at a lit fire. Fires burn
  wood and go out when they run dry. Stoking a fire is a hauler's job: a haul of wood from the
  pile to the fire.
- **FR62**: A hungry dwarf of any trade leaves its work to eat from the pile, preferring a meal
  to raw food. Eating relieves hunger.

### F21. Cold, sleep, misery and idle life

- **FR63**: Dwarves grow cold on the surface and away from a lit fire, and warm up near one.
  Underground is warmer the deeper it goes, so a deep working is never cold. A cold dwarf of any
  trade goes to a fire to warm up. Torches give light, not warmth.
- **FR64**: Dwarves tire as they work and need to sleep. A tired dwarf of any trade lies down
  to sleep. With no shelters, that means on the ground, near a fire if one is close. Sleep
  relieves tiredness, and a sleeping dwarf keeps losing warmth like any other.
- **FR65**: A dwarf whose hunger, cold or tiredness passes a threshold becomes **miserable**.
  It is visibly miserable in the Bevy client (posture or animation, driven by sim state) and
  shown in the TUI, and it refuses trade work until relieved. It never refuses **self-care**:
  a needy dwarf relieves itself by the nearest means. It eats from the pile, or straight from a
  plant if the pile has no food. It warms at a lit fire, or deep underground if no fire is lit.
  It sleeps wherever it is. Neglect therefore never locks the fortress up. No dwarf dies.
- **FR66**: An idle dwarf does small, visible things that come from sim state: it wanders,
  gathers at the fire, follows a nearby animal, and naps. All of it is deterministic. The exact
  set is tuned at the seat, and the bar is Wolf's "not boring when not actively doing anything".
  Art follows the M2-24 hard stop.

### F22. A fortress that runs itself

- **FR67**: Once the standing work is set up (a pile, a forage area, a fishing hole, and trees
  marked for cutting), the fortress runs with no further orders. Hunters pick the nearest
  animal, cooks cook whatever raw food the pile holds, and haulers stoke whichever fire is
  lowest. On the bench scenario, the food and wood stock never runs down to nothing over the
  window.
- **FR68**: The player can place a fire the same way they place a stockpile: a marked spot that
  haulers bring wood to, which then burns and is stoked like the campfire (FR61). This is the one
  exception to "no building". A fire placed where it cannot burn (on rock, on lake ice, on a
  stockpile) is refused loudly (NFR11).

## Cross-cutting NFRs — Milestone 4

- **NFR9: determinism holds**, extended to crew and map settings, arrivals, animals, plants and
  regrowth, fires and fuel, placed fires, hunger, cold, sleep and misery. Seed plus command log
  gives identical state, and save → load → tick N equals never-saved → tick N for each.
- **NFR10: parity rule, as narrowed.** Every new sim state reaches the TUI as display (animals,
  food, fires, sleep, misery). New commands (forage area, hunt, fishing hole, place fire, new
  professions) are Bevy-only. The TUI gains no new input.
- **NFR11: refuse loudly**, for every new filter.
- **NFR12: the tick fits at the target scale.** On the 256² crew-200 bench scenario, the tick
  mean is at most 25 ms, and p99 stays under the 100 ms budget (Wolf). It is measured by
  `tick-bench.sh`: a release build, interleaved runs, on a machine running nothing else, with p99
  taken as the median across runs. A single run is not a reading. The bar gates the milestone
  close. Each story reports its delta from the first story on, and a regression beyond the
  recorded noise floor needs Wolf's ruling. Optimization still waits for a profile (tech
  preferences). 200 is the target, with no fallback floor set in advance: a miss is measured and
  ruled by Wolf (2026-10-28).
- **NFR13: the seat holds at the target scale.** M2's NFR6 frame-rate bars hold with 200 dwarves
  on 256², or the miss is measured and ruled by Wolf. This is measured early (FR54), not at the
  close. #179 (the gui's frame rate swings 20–180) is the known defect this bar meets first.

## Out of scope — Milestone 4

M1–M3's non-goals stand, except where **What M4 changes** reopens them. Additionally:

- No death, starvation, freezing to death or other fail state. Misery is the consequence.
- No shelters, building or construction except placing a fire (FR68); no crafting, workshops or
  trading.
- No threats or combat. Animals are harmless.
- No moods beyond misery, and no skills. No beds or bedrooms: dwarves sleep where they are.
- No cave-ins.
- FR40 (per-dwarf model variation) stays parked. Wolf said at the M3-close retro that it is "not
  that urgent", and that the model and rig need a rethink first.
- No chunking, no interest management, and no protocol redesign (NDJSON stays; new state gets new
  message fields), unless the bench or the seat profile shows one is needed (tech preferences:
  profile first).
- Still parked: #199 (what stops a miner digging deeper), #169 (the gui mirrors the tui), and the
  look issues (#120, #126, #127, #128, #130, #137, #175–#177, #181).

## Success criteria — Milestone 4

1. On a fresh 256² world, the crew grows by arrivals to 200, **in one sitting at the seat**, while
   dwarves forage, hunt, fish, cook, stoke, eat, warm themselves and sleep.
2. With the standing work set up, Wolf watches for **one full day–night cycle** without giving any
   order and rules it **not boring**.
3. Neglect reads: with the pile empty of food, or the fires out of wood, dwarves visibly turn
   miserable and stop work. Putting it right restores them.
4. The 256² crew-200 bench scenario meets NFR12 at the close, and every M4 story's record carries
   its delta there.
5. #159 closes with a scenario test that was red first. 13.5 (clamber) is landed or dropped on
   the #207 reading at scale.
6. The gate is green. The determinism scenario tests cover everything NFR9 lists.

**Counter-metrics** (what success must not cost):

- **M4 targets 12–16 new stories.** 13.1–13.4 do not count. Defect stories count. The tooling
  action items (#142, #144, #148, #211, #212) run alongside and do not. This cap is soft:
  stretching is allowed when it is called out on the record. If the cap bites, cut in this order
  (Wolf): first FR66's extra idle behaviours (wandering and the fire stay); then hunting and
  cooking together (food comes from foraging and fishing, eaten raw); then regrowth (plants are a
  one-off harvest).
- **Performance is not back-loaded.** F18 comes first; no game story merges without its
  bench delta.
- **Feel is not back-loaded.** Misery and the first idle behaviour reach the seat in the first
  half.
- **No TUI regression ships**, and no 128² five-dwarf scenario changes outcome without a reason
  on the record.
- The PRD stays about five pages, because M4 adds four needs systems. The detail goes in
  `addendum.md`.
