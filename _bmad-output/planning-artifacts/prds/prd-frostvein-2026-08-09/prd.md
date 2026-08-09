---
title: Frostvein Milestone 2 — Bevy Client PRD
status: final
created: 2026-08-09
updated: 2026-08-09
---

# Frostvein Milestone 2 — Bevy Client

Inherits the Milestone 1 PRD (`prd-frostvein-2026-08-01/prd.md`, final) by
reference: the four-crate spine, determinism, YAGNI-as-policy, and the TUI's
load-bearing role as deterministic assertion instrument all stand unchanged.

## Vision

Milestone 2 gives Frostvein a client worth *looking* at. The sim has been
provably alive since Milestone 1, but only ever legible — never beautiful,
never something you'd sit and watch. The Bevy client makes the frozen valley
a place: an isometric voxel diorama you look down into and orbit by hand,
where a cold, dark blue, indifferent world is punctured by pockets of warm
light, and your eye is pulled to the dwarves because they are the warm thing
in the cold. The world must be rich and beautiful — and visually pleasing
**as soon as possible**, not as the milestone's final payoff.

Frostvein is a **procedural game** — that is identity, not implementation
detail. The world is seeded and generated; richness comes from light, sky,
and atmosphere over procedural geometry, never from designed levels (the
addendum's Valheim lesson). Concept
references (two images + `docs/narrative.md`) are guidance,
not acceptance bars.

**The done sentence:** Milestone 2 is done when Wolf can watch the walking
skeleton — designate, pathfind, dig, haul — live in the Bevy client, in a
world rich and beautiful enough that he keeps watching after the stone lands
in the stockpile.

## Visual Target & Game Feel

The section FR24's failure demanded. Every line here describes what the boss
sees, feels, or can tell at a glance; **no line may name a rendering
technique.** If a sentence could be satisfied two different ways in code, it
is written correctly.

### The view

- A frozen mountain valley seen as an isometric diorama — you look *down
  into* a place, from outside, and orbit it by hand. The camera is always
  usable; there is no angle you get stuck in.
- One zoom continuum, two registers: pulled close, a working view where
  individual dwarves and blocks are readable; pulled out, a vista where the
  valley, sky, and aurora carry the frame and dwarves become warm specks.
  The far register is the same view, not a mode — pulling out changes
  distance, never representation.
- The world keeps discrete z-levels, DF-style, even in 3D. Dwarves start at
  ground level and dig down; the player can slice into the mountain to see
  and work the underground. The slicing mechanism is a story-level
  design-and-test question (Wolf's candidate: mousewheel; known collision
  with zoom — see addendum).

### The light (the wow mechanism)

- The organising principle is cold against warm: a dark blue night world —
  snow, ice, stone, stars, a sweeping aurora — punctured by pockets of warm
  orange light where the dwarves are.
- The eye lands on the dwarven encampment first, and it lands there because
  of the warm/cold contrast, not because of a UI marker.
- Warm light sources exist *in the world* (things that glow), so the
  contrast is real, not painted on.

### What the references bind (bars, not guidance)

The concept images are guidance, but reconciliation showed they carry
qualities a build could miss while passing the text above. These are bars:

- The sky is an illuminant, not a backdrop: aurora and starlight visibly
  light the snow and catch on ice, and the aurora hugs the horizon rather
  than hanging overhead.
- Snow reads as a settled cap: white tops, bare dark flanks, loaded
  branches — not a uniform coat.
- Work leaves evidence: rubble and debris at the dig face (the sim's stone
  items, plus cosmetic chips under NFR5's carve-out), so a worked site
  never looks spotless.
- Value discipline: night snow stays midtone blue-grey; only emissive
  light approaches white. Bright moonlit snow would flatten the warm/cold
  read.
- The cold field varies: blue ice breaks the white expanse, so the vista
  reads in cold-against-cold layers, not one white sheet.
- The world reads as a miniature whose edges dissolve into the night — a
  raw grid edge is never visible at any zoom. Treatment is a
  design-and-test question (addendum).

Depth for these (values, material rules) belongs to the tech-art
guidelines deliverable.

### The two wow beats (both required)

1. **Cold boot:** the first frame is an aesthetic hit on looks alone —
   voxel world, dramatic lighting, aurora.
2. **~Thirty seconds in:** the realisation that it's *alive* — light
   flickers, work animates at the dig face, a dwarf picks something up and
   carries it. The moment a beautiful still image becomes a running
   simulation. This beat is the magic; a client that only achieves beat 1
   has failed the milestone.

### The anti-requirements (4.1a's six failures, inverted)

The 3D TUI view was judged: ugly, flat, cluttered, confusing, lifeless, with
an unusable camera. Each inverts into a bar this client must clear:

| 4.1a failure | M2 bar |
| --- | --- |
| Ugly | The boot frame is something you'd screenshot unprompted. |
| Flat | Depth reads instantly — light, shadow, and air separate near from far. |
| Cluttered | At working zoom, you can tell dwarves, terrain, designations, and items apart at a glance. |
| Confusing | You always know what you're looking at, which z-level you're on, and what's underground vs. surface. |
| Lifeless | Something visibly moves even when you issue nothing — work, light, weather, idle wandering. |
| Camera unusable | You can always reach the angle you want, and never lose the fortress. |

### Sign-off gate

No visually subjective story is implemented before Wolf has approved a cheap
"here is what you will see" artifact for it (target frame, mock, sketch, or
generated reference of *our actual world* at the framing being built). This
is the structural fix for the FR24 defect class — a spec that is meetable,
implemented, and not what was wanted, which no review layer can catch.
The gate has a closing half: the story is done only when Wolf has viewed
the built result live and compared it against the approved artifact —
4.1a was lost at live viewing, not at spec time.
`[ASSUMPTION]` granularity: one artifact per visual story, not one for the
milestone.

## Scope shape (agreed, pre-FR)

- **In:** the Bevy client itself; full TUI input parity (designate
  dig/channel, cancel, stockpiles, speed, save/load); the *minimum*
  sim/protocol content that makes the light real — things that glow warm,
  and trees; atmosphere (night sky, stars, aurora, cosmetic snowfall).
- **Out (unchanged M1 non-goals, not reopened):** minecarts, built walls,
  mine crystals, flowing water, off-map anything (distant peaks, second
  outpost), construction, fluids, weather-as-simulation.
- **Baseline:** M2 starts against today's simd functionality and today's
  seeded worldgen. More sim control gets added by specific stories when a
  story needs it, not up front. The narrative's six dwarves were scene
  dressing — the count stays at FR3's five until a story changes it.
- **Art:** procedural/code-first. No asset pipeline in the base build;
  authored assets (Wolf + AI tooling) enter when a concrete case forces the
  decision — dwarves are the expected first case. This **overturns, on the
  record,** the M1 brief's "models authored as code, never as assets, ever"
  — that constraint's premise (no artist, no pipeline) no longer holds:
  Wolf is an artist with AI asset tooling. A tech-art-guidelines deliverable
  defines the asset contract when the pipeline opens.
- **Parity rule:** Bevy first catches up to the TUI's features and reaches
  the look-and-feel bar; the TUI is not extended for Bevy-only work. But any
  new sim functionality or bug fix that affects the TUI updates the TUI too
  — no regression, no stagnation on sim-level change.

## Features & Functional Requirements — Milestone 2

Capabilities, not implementation. FR IDs continue Milestone 1's global
numbering (M1 ended at FR26); feature groups continue at F10. FR24's
re-homed outcome — *what the boss should see* — is delivered by F11, stated
as outcome throughout.

### F10. World content that glows and grows (the one place M2 touches the sim)

All of this is worldgen/sim-side, seeded, and deterministic — clients render
it, never invent it.

- **FR27** — Worldgen grows pine trees on the surface: seeded, deterministic,
  part of world state, visible to every client (the TUI shows them as
  glyphs). `[ASSUMPTION]` density and placement are worldgen tuning
  decisions inside the story, not FR text.
- **FR28** — Worldgen places static warm light emitters: torches and a
  campfire at the dwarven starting camp. Light emitters are world state with
  a position; what light *looks* like is each client's concern.
- **FR29** — Dwarves carry lanterns: a light source attached to a moving
  entity. This is deliberately the lighting system's hardest case — a moving
  warm light — placed in scope as a testbed. `[ASSUMPTION]` every dwarf
  simply carries one; no fuel, no pickup/drop, no economy.
- **FR30** — Protocol v0 vocabulary grows to carry the above (tree and
  light-emitter materials/entities, carried-light state) as typed world
  data, honouring FR17's world-not-game principle. No shape changes, only
  vocabulary.

### F11. The diorama (Bevy client — the view)

- **FR31** — The world renders as the isometric orbitable diorama the Visual
  Target describes: one zoom continuum from working-close to valley-vista,
  camera always usable, never lost.
- **FR32** — The cold/warm read is live: world light sources render as warm
  pools against the cold night palette; sky, stars, and aurora carry the far
  register; snow falls as pure decoration (no sim weather).
- **FR33** — The player can slice into the mountain by z-level to see and
  work the underground, and can always tell which z-level they are on and
  what is underground vs. surface. Mechanism per the addendum's open
  question — chosen by testing in its story, not here.
- **FR34** — The world visibly lives, driven only by real sim state over the
  wire: dwarves move and work at the dig face, carried lanterns move with
  them, static lights flicker, idle dwarves wander. Zero commands issued
  still means visible motion (M1's FR4 aliveness, now in 3D).

### F12. Working the fortress (input parity)

- **FR35** — The Bevy client reaches full TUI command parity: designate
  dig/channel, cancel designation, place/remove stockpile, pause/resume,
  tick rate, save/load, quit. Clients contain zero game logic, unchanged.
- **FR36** — The player can select tiles and rectangles in the 3D view with
  the mouse — the picking problem — including on sliced underground z-levels.
  Acknowledged as M2's hardest input work and the main story-count driver.

### F13. Client lifecycle (the boring glue)

- **FR37** — The Bevy client is a `protocol`-only consumer: connects,
  receives snapshot, applies per-tick deltas, coexists with concurrent TUI
  clients on the same daemon (M1's FR19). `sim-core` and `simd` need no
  structural change for it.

## Cross-cutting NFRs — Milestone 2

- **NFR5 — No drift.** Clients never invent world state; everything visible
  in any client is derivable from the wire (Wolf's rule; AD-1/AD-4 in
  `docs/architecture.md`, restated). One deliberate carve-out: pure atmosphere —
  the sky, aurora, snowfall — is client-side by design and must never
  acquire sim meaning silently.
- **NFR6 — Feels alive, Bevy bar.** The client sustains the diorama at
  interactive framerates on the dev machine with the full 128×128×32 world,
  all dwarves, and all lights, with command acknowledgement comparable to
  the TUI's (~200 ms). `[ASSUMPTION]` a measured number (e.g. 60 fps) is
  set at architecture time as a blocking item, per the sprint change
  proposal (`_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-08.md`)
  — NFR2 explicitly does not stretch to cover this client.
- **NFR7 — Determinism unchanged.** FR27–FR29 land inside worldgen and sim
  state, so seed + command log ⇒ identical state must survive them;
  scenario tests cover trees and light emitters like any other world state.
- **NFR8 — Gate grows a sibling probe.** `scripts/gate.sh` gains the
  bevy-client twin of the `tui` probe (no `sim-core` edge), per the sprint
  change proposal — the AD-1 edge stays guarded for the client that matters
  most.

## Out of scope — Milestone 2 (silence is not permission)

M1's non-goals stand except where an FR above explicitly reopens one (trees
and light emitters are the only reopenings). Additionally out for M2:

- No minecarts, built walls, or mine crystals; no construction of any kind.
- No flowing water; no fluids. Frozen river terrain, if worldgen ever makes
  any, is just ice material.
- No off-map anything: no distant peaks beyond the world grid, no second
  outpost.
- No sim weather: snowfall is client-side decoration (NFR5's carve-out).
- No asset pipeline in the base build; no authored per-creature assets until
  a story forces the decision on the record.
- No new sim mechanics beyond F10: no needs, moods, combat, farming,
  crafting — the M1 list, unchanged.
- No larger maps, chunking, or hierarchical pathfinding; no protocol
  optimisation. The world stays 128×128×32.
- No TUI feature work chasing Bevy-only capabilities (the parity rule's
  other half).

## Success criteria — Milestone 2

1. The walking-skeleton scenario — designate, pathfind, dig, haul — runs
   live in the Bevy client, and Wolf signs off **both wow beats in one
   sitting**: the boot frame on looks alone, and the alive moment thirty
   seconds later.
2. Every visually subjective story shipped only after its sign-off artifact
   was approved — zero FR24-class misses (spec meetable, implemented,
   unwanted).
3. The anti-requirements table holds: none of the six 4.1a words — ugly,
   flat, cluttered, confusing, lifeless, camera unusable — is true of this
   client.
4. The quality gate is green across the workspace, including the new client
   probe (NFR8), and determinism scenario tests cover trees and light
   emitters (NFR7).
5. Total planning docs (this PRD + the M2 architecture pass) remain
   re-readable in one sitting.

**Counter-metrics** (what success must not cost):

- **M2 ships in 10–14 stories.** Materially more means scope gets cut, not
  the plan extended. The cut list is decided now, in order: first **FR29**
  (lanterns — torches and campfire still carry the warm/cold wow); then
  parity narrows — **FR35/FR36 shrink to camera + speed control**, and the
  TUI keeps designations until a later milestone.
- **"As soon as possible" has teeth:** the first boot-frame wow — world,
  light, aurora, no input needed — lands in the milestone's **first third**.
  A plan that back-loads the visual payoff is wrong, cap or no cap.
- **No TUI regression ships during M2**, and sim-level changes and bug fixes
  that affect the TUI update the TUI (the parity rule, enforced).
- Criterion 5 doubles as a counter-metric, as in M1: thoroughness that
  bloats the docs is a failure, not a virtue.

## Assumptions index

Inline `[ASSUMPTION]` tags, collected for the architecture pass:

- **Visual Target / sign-off gate** — one artifact per visual story, not one
  per milestone.
- **FR27** — tree density/placement is worldgen tuning inside the story.
- **FR29** — every dwarf simply carries a lantern; no fuel, pickup/drop, or
  economy.
- **NFR6** — the measured Bevy feel bar (fps number) is set at architecture
  time; NFR2 explicitly does not stretch to this client.
