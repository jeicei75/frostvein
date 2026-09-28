---
title: Frostvein Milestone 3 — Game Functionality PRD
status: final
created: 2026-09-28
updated: 2026-09-28
---

# Frostvein Milestone 3 — Game Functionality

Inherits the Milestone 1 and 2 PRDs (`prd-frostvein-2026-08-01`, `prd-frostvein-2026-08-09`,
both final) by reference: the four-crate spine, determinism, YAGNI-as-policy, NFR5 (no drift),
the parity rule, and M2's Visual Target all stand, except where **What M3 changes** below says
otherwise.

## Vision

Milestone 2 made the valley worth looking at. Milestone 3 makes it worth *playing*: a more
immersive game with more to do, and a reason to do it. The five dwarves stop being
interchangeable. Each has a name, a look and a trade, and they work side by side: one digs,
one hauls, one fells pines. A short chain of goals gives the first session a direction without
ending the game.

**The done sentence:** Milestone 3 is done when Wolf can start a fresh world, tell his dwarves
apart, give them their trades, and watch them dig, haul and cut *at the same time* until the
tutorial chain completes. Play then carries on.

## Scope shape

- **In:** dwarf identity; three concurrent jobs driven by professions; three work animations;
  new diggable resources and a wood resource; a chained tutorial of collection goals;
  one-dwarf-per-tile movement; the three sim defects Wolf found at the M2 close.
- **Planning shape (retro ruling):** one M3 epic, which grows by stories. A new epic only if the
  milestone goal changes.
- **Instrument per kind of work (retro ruling):** sim work is judged by deterministic scenario
  tests (seed → commands → tick N → assert). Look and feel work (identity, animations, profession
  UI) reaches Wolf's seat early, rather than being argued from screenshots.
- **Baseline:** today's five dwarves and today's worldgen.

## What M3 changes

These are the inherited rules M3 reopens or narrows, stated here so that nothing overrides them
silently:

- **M1 FR5** — FIFO job claiming still holds, but each dwarf claims only from jobs of its own
  profession (FR42).
- **M1 FR12** — Items gain kinds: stone, wood, minerals and gems. There is still no quality,
  stacking or containers.
- **M1 non-goal "no production beyond dig → stone → stockpile"** — reopened for collecting wood,
  minerals and gems. Using them stays out.
- **M1 non-goal "nothing falls"** — reopened for items and trees only (FR52). Unsupported terrain
  and cave-ins stay out.
- **M2 non-goal "no mine crystals"** — reopened as gems (FR46).
- **The parity rule** — narrowed by NFR10.
- **M1 FR17 ("a world, not a dwarf game")** — goals travel over the wire as typed data: target,
  progress and completed. The rules that advance them live only in the sim.

## Features & Functional Requirements — Milestone 3

These describe capabilities, not implementation. FR IDs continue the global numbering (M2 ended
at FR37). Feature groups continue at F14.

### F14. Dwarves you can tell apart

- **FR38** — Every dwarf has a name and a distinct colouring. Both are part of world state:
  seeded, deterministic, sent over the wire, and saved. Clients render identity; they never
  invent it (NFR5).
- **FR39** — The player can tell which dwarf is which at a glance in the Bevy client, both
  in the world and when a dwarf is selected. A selected dwarf stays sharp when the camera zooms
  in (#136). The TUI shows names.
- **FR40** *(stretch)* — The dwarf model is prepared for per-dwarf variation beyond colour.
  This is preparation only. It ships no new variants, and it is the first item cut under
  cap pressure.

### F15. Trades and concurrent work

- **FR41** — Three jobs exist: **dig**, **haul** and **cut tree**. Cutting a tree removes the
  whole tree from the world and yields wood items at its base. Dig no longer takes tree tiles.
  Today a dig designation removes trees one tile at a time and yields nothing.
- **FR42** — Each dwarf has one profession (miner, hauler or woodcutter) and takes only jobs of
  that profession. With the professions spread across dwarves, dig, haul and cut run at the same
  time, and hauls no longer wait behind a dig backlog. A dwarf with no work in its trade idles
  and wanders; it does not help with other trades.
- **FR43** — The player assigns and changes professions in the Bevy client by selecting a dwarf.
  This is new selection and UI work. Each dwarf starts with a seeded default profession, and the
  five always include at least one miner, one hauler and one woodcutter. A fresh world is
  therefore playable before any assignment.
- **FR44** — The player can designate trees for cutting in the Bevy client, the same way they
  designate digging.
- **FR45** — Dwarves visibly perform their work: a dig, a haul (carrying) and a cut animation,
  each driven only by real sim state and timed to the sim's work. Art follows the retro's hard
  stop (M2-24): every round gets a ledger row, and after two rounds that Wolf's eye has not judged
  as converging, the loop returns to Wolf. He decides whether the animation ships plain or is
  parked.

### F16. Resources and goals

- **FR46** — Worldgen places minerals and gems inside the rock: seeded and deterministic, with
  gems rarer than minerals. Digging a mineral or gem cell yields an item of that kind, which is
  hauled like stone. The set of kinds is small and fixed: one or two minerals and one gem. The
  exact kinds and placement are tuned in the story.
- **FR47** — A chained tutorial of goals runs in the sim: collect N wood, then N minerals, then
  N gems. A resource counts only once it is in a stockpile. Progress is the current total of that kind
  across all stockpiles, so items stockpiled before the goal started count too. A goal that has
  completed stays completed. Goals and progress are world state, sent over the wire and saved. The targets are hardcoded constants.
- **FR48** — Both clients show the current goal and its progress. When a goal completes the
  player sees it, and when the chain completes the game says so. There is no fail state, and
  play continues after the chain.

### F17. Movement and sim defects

- **FR49** (#133) — At most one dwarf stands on a tile. A dwarf whose next step is occupied
  waits or routes around the other dwarf. Two dwarves meeting head-on in a one-wide tunnel both
  get moving again within a bounded number of ticks, which the scenario test names: the dwarf
  with the lower id backs off or reroutes. If it has nowhere to go, the other dwarf backs off.
- **FR50** (#134) — Stockpile placement, hauls and falling items never leave an item on, or
  visibly inside, a light emitter such as the campfire. The story first checks whether today's
  stones sit on the emitter's own cell or on a neighbouring cell that its model overlaps.
- **FR51** (#132) — No dwarf stays stuck after digging. A dwarf that cannot reach its job, or
  whose job has lost its support, releases the job within a bounded number of ticks and goes on
  with other work. The designation stays queued and retried, and is never silently dropped (M1
  FR8 holds). This is reproduced first as a red scenario test.
- **FR52** (#135) — Nothing is left floating above a dug-out block: items fall to the next
  supporting surface, and trees fall or are removed. No dwarf is left standing on air. The
  story picks the simplest rule that reads right. This is new sim ground and the largest of the
  three defects.

## Cross-cutting NFRs — Milestone 3

- **NFR9 — Determinism holds.** Identity, professions, resources, goals and one-per-tile movement
  all live in sim state. Seed plus command log gives identical state, and scenario tests cover
  each of them.
- **NFR10 — Parity rule, narrowed.** This is a change on the record. Every sim change reaches the
  TUI as display: names, new items and goal progress. Profession assignment and tree designation
  are Bevy-only commands in M3, and the TUI gains no new input.
- **NFR11 — Refuse loudly.** A command the sim refuses (a stockpile on rock, a cut on a tile that
  is not a tree, and so on) is visible to the player, not silently discarded. M3 adds several
  new filters, and this is the known trap for them.

## Out of scope — Milestone 3

M1 and M2's non-goals stand, except where **What M3 changes** reopens them. Additionally:

- Resources are not *used*. There is no crafting, building or trading with wood, minerals or gems.
  They are collected and counted.
- No fail states, threats, needs, moods or combat.
- No per-dwarf skills, levels or experience. A profession is a label that filters jobs.
- No new dwarves. The count stays five.
- No cave-ins and no unsupported-terrain physics.
- The look issues stay parked until one bites (#120, #126, #127, #128, #130, #137).

## Success criteria — Milestone 3

1. On a fresh world, Wolf assigns professions and watches dig, haul and cut happen concurrently,
   **in one sitting at the seat**. Each animation reads, unless Wolf ruled under FR45 that it
   ships plain or is parked.
2. Wolf can name each of the five dwarves by sight.
3. The tutorial chain completes in play, the game announces it, and play continues.
4. #132, #133, #134 and #135 each close with a scenario test that was red first and is green now.
5. The gate is green. The determinism scenario tests cover everything NFR9 lists.

**Counter-metrics** (what success must not cost):

- **M3 targets 10–14 stories.** This cap is soft: stretching is allowed when it is called out on
  the record, not drifted into. If the cap bites, cut in this order: first FR40 (model
  preparation); then FR52's trees (keep falling items); then the gem tier of FR46/FR47.
  The retro's tooling action items (M2-28 to M2-31) run alongside and do not count toward the cap.
- **Look work is not back-loaded.** Identity and the first work animation reach
  the seat in the milestone's first half.
- **No TUI regression ships.**
- The PRD stays about two pages.
