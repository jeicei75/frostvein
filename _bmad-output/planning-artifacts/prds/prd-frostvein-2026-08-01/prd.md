---
title: Frostvein PRD
status: final
created: 2026-08-01
updated: 2026-08-02
---

# Frostvein — Product Requirements Document

## Vision

Frostvein is a Dwarf Fortress–inspired voxel colony simulation set in an icy,
gloomy, dark and grim world, built as a headless, deterministic simulation
daemon with thin display clients — TUI first, an Unreal client as the
long-term visual target. The world must look delicious even in the TUI: the
icy-grim identity is carried by color and glyph choices from day one, not
deferred to future graphics. The sim must be playable and enjoyable early and
remain fully testable headless: the fun is the whole loop — issuing an order,
watching dwarves obey it live, and feeling the sim breathe under pause and
fast-forward.

The player is the boss — a distant god issuing directives, not a hand on a
remote control. Dwarves obey in their own time, and the world visibly lives
even when no orders are given. Dwarven unpredictability and whimsy are part
of the intended fun, and an LLM-driven flavor layer is on the roadmap to
deepen it — always outside the deterministic sim core (see Future phases).

Frostvein is a solo hobby project, developed primarily by AI agents under the
owner's direction. Restraint and shipped increments are valued over
completeness and generality — that stance is a requirement of this document,
not background color.

**Phase one (Milestone 1) delivers the walking skeleton:** a dwarf in a
procedurally generated voxel map receives a dig designation, pathfinds to it,
digs the tile, and hauls the resulting stone to a stockpile — all visible live
in the TUI client. This PRD gives FR-level depth to phase one only; later
phases are sketched for direction and deliberately left shallow, to be
iterated into this document when their time comes.

## Features & Functional Requirements — Phase One (Milestone 1)

Capabilities, not implementation. FR IDs are stable and globally numbered.

### F1. World

- **FR1** — The world is a fixed-size 3D voxel grid (default 128×128×32) with
  simple layered terrain — stone, soil, ice, snow, air — generated from a
  world seed. The icy materials are not decoration: snow surfaces and ice
  appear in ordinary generated terrain, so the world reads as frozen from the
  first boot. No chunking or streaming.
- **FR2** — Terrain generation produces surface height variation with
  walkable ramps/slopes, so vertical traversal exists in a naturally
  generated map. `[ASSUMPTION]` modest rolling height (a few z-levels), not
  mountains — enough to exercise climb pathfinding and channel digging.

### F2. Dwarves & jobs

- **FR3** — A handful of dwarves spawn on the surface at world generation.
  `[ASSUMPTION]` 5 dwarves.
- **FR4** — Each dwarf runs a simple job state machine: idle → walk → work.
  Current state is visible to clients. No needs, moods, or personalities.
  Idle is not frozen: idle dwarves wander nearby tiles (seeded,
  deterministic) so the world visibly lives even with no orders given.
  `[ASSUMPTION]` wandering stays within ~3 tiles of the dwarf's current spot.
- **FR5** — Idle dwarves claim the oldest unclaimed job (FIFO). One dwarf per
  job; a claimed job is released if the dwarf cannot complete it. Dwarves
  react in their own time: a seeded per-dwarf reaction delay passes before a
  job is claimed, so orders feel like directives to workers, not remote
  control. `[ASSUMPTION]` delay of roughly 0.5–3 s (5–30 ticks), seeded per
  dwarf per job.
- **FR6** — Dig job: a dwarf adjacent to a designated tile removes it (dig:
  wall becomes open floor; channel: floor is dug out leaving a ramp below)
  and a stone item appears at the dug location.
- **FR7** — Haul job: a dwarf carries a loose stone item to a stockpile tile
  and drops it there.
- **FR8** — A designation that is currently unreachable stays queued and is
  retried; it is never silently dropped. // NOTE: naive retry is acceptable
  in phase one.

### F3. Player intents

- **FR9** — The player can designate tiles for digging, as rectangles, in two
  modes: **dig** (same-level excavation) and **channel** (dig down, leaving a
  ramp). A designation can be cancelled before it is dug (remove
  designation), releasing any unclaimed or in-progress job on it.
- **FR10** — The player can place a stockpile zone as a rectangle on walkable
  floor.

### F4. Pathfinding

- **FR11** — Dwarves pathfind with plain A* on the voxel grid: walking on
  floors and climbing ramps/stairs between z-levels. No hierarchical
  pathfinding, no caching.

### F5. Items

- **FR12** — Stone exists as a haulable item with a world position. No
  materials system, no quality, no stacking, no containers.

### F6. Daemon & tick loop

- **FR13** — The daemon runs a fixed-timestep tick loop (default 10
  ticks/sec), fully decoupled from clients; the sim advances with zero
  clients attached.
- **FR14** — Speed control: pause, normal (1×), and fast-forward implemented
  as tick-rate changes. `[ASSUMPTION]` one fast step (≈5×) is enough for
  phase one.
- **FR15** — Determinism: identical world seed + identical command sequence
  produces identical sim state, tick for tick. This is load-bearing for the
  scenario harness (F9).
- **FR16** — Dev save/load of the full sim state, plus clean quit. No save
  format stability guarantees.

### F7. Protocol v0

- **FR17** — Newline-delimited JSON over localhost TCP. On connect a client
  receives a full world snapshot; thereafter per-tick delta messages.
  Chattiness is acceptable; no batching, compression, or interest management.
  Design principle: messages describe *a world, not a dwarf game* — state and
  typed data (positions, materials, professions), never game rules or
  narrative interpretation. This lets the channel carry any realm a future
  producer feeds it (see Future phases wild card) at zero extra cost now.
- **FR18** — Commands upstream: designate dig/channel, cancel designation,
  place stockpile, pause/resume, set tick rate, save, load, quit.
- **FR19** — Multiple localhost clients can view the same running sim
  concurrently.

### F8. TUI client

- **FR20** — Single z-level top-down view of the world, navigable between
  z-levels DF-style (`<`/`>`). The client contains zero game logic.
- **FR21** — Modal, DF-familiar keyboard input: single keys enter a mode
  (dig, channel, stockpile), rectangles are placed cursor-first with
  Enter-anchor / Enter-commit, `Esc` backs out, and a one-line hint bar
  always shows the active mode's keys. Concrete keymap in the addendum.
  Mouse/touch input is phase two (see addendum).
- **FR22** — Dwarves render as `☺` glyphs colored by current job/profession
  (e.g. miner amber, hauler teal); terrain and items render as distinct
  glyphs. 24-bit truecolor from the start; color is data (material/profession
  → RGB), not a fixed palette.
- **FR23** — The visual identity is icy, gloomy, dark and grim: a cold,
  desaturated terrain palette with profession colors as warm accents. The
  world should look delicious in the terminal, within phase-one scope. The
  acceptance instrument is Wolf's eye: success criterion 2 includes sign-off
  on the icy-grim look in the live TUI. `[ASSUMPTION]` this is palette/glyph
  selection work inside existing rendering stories, not a separate art story.
- **FR24** — The raycast 3D view is its own story late in the milestone.
  Required for phase one — Wolf's override (2026-08-01) of this FR's earlier
  may-slip clause; it no longer slips and is off the cut list.

### F9. Headless scenario harness

- **FR25** — Scenario tests build a world from a seed, inject commands, tick
  N times, and assert sim state — with no client or network attached.
- **FR26** — The walking-skeleton sentence exists as an automated scenario
  test (dig designation → pathfind → dig → haul to stockpile) and is the
  phase-one gate.

## Cross-cutting NFRs

- **NFR1 — Platform.** Phase one targets the WSL2 devpod and any decent
  terminal emulator over SSH. No other platforms. (Long term this is a true
  server + client game across machines; nothing in phase one may preclude
  that, and nothing in phase one builds for it.)
- **NFR2 — Feels alive.** The TUI keeps pace with the sim at 10 ticks/sec
  with no visible stutter (~100 ms frame budget on the dev machine, full
  128×128 z-level). A player command is *acknowledged* in the UI within
  ~200 ms (one tick + one frame) — the designation appears marked, the mode
  responds. Dwarf obedience is explicitly exempt from this bar: dwarves
  react in their own time (FR5). Even with zero commands issued, the view
  visibly moves (idle wandering, FR4). Checkable by eye; no measurement
  infrastructure in phase one.
- **NFR3 — Determinism everywhere.** FR15 is cross-cutting: every feature
  must keep seed + command log ⇒ identical state true. Any source of
  nondeterminism (unordered iteration, wall-clock time, unseeded randomness)
  in `sim-core` is a bug.
- **NFR4 — Quality gate.** Every story lands with `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo test` green.

## Out of scope — phase one (silence is not permission)

Carried over from the project brief; listing them here keeps this PRD safe to
hand downstream on its own. None of these exist in phase one:

- No Unreal or any graphical client.
- No world-generation history, civilizations, or off-map anything.
- No combat, health, injuries, or body parts.
- No fluids, temperature, weather, seasons, or cave-ins. Ice and snow (FR1)
  are terrain *materials*, not simulated processes — nothing melts, freezes,
  or falls in phase one.
- No needs, moods, or personalities (idle wandering and reaction delays are
  seeded behavior, not a mood system), and no social systems.
- No farming, crafting chains, or production beyond dig → stone → stockpile.
- No save-format stability guarantees (FR16's dev save/load is enough).
- No multiplayer beyond multiple localhost clients viewing the same sim.
- No performance optimization before a measured problem exists.
- No mod support, scripting, or data-driven content systems.
- No binary protocol, compression, or interest management (protocol v0 only).

## Success criteria — phase one

1. The walking-skeleton sentence passes as an automated headless scenario
   test (FR26).
2. The same scenario is watchable live in the TUI attached to the running
   daemon, it meets the feel floor (NFR2), and Wolf signs off on the
   icy-grim look (FR23) in the same session.
3. The quality gate (NFR4) is green across the workspace.
4. Total planning docs (PRD + architecture) remain short enough to re-read in
   one sitting.

**Counter-metrics** (what success must *not* cost):

- Phase one ships in 8–12 vertically sliced stories. Materially more means
  scope gets cut, not the plan extended.
- No code exists that serves only a future phase (YAGNI is policy).
- Criterion 4 doubles as a counter-metric: thoroughness that bloats the docs
  is a failure, not a virtue.
- FR count is not story count — FRs pack into vertical slices. If story
  planning still exceeds 12, the cut list starts with FR16 (save/load).
  FR24 (raycast view) was removed from the cut list by Wolf's override,
  2026-08-01.
- The story rules in `docs/technical-preferences.md` apply unchanged:
  vertical slices only, every story ends in something observable, each fits
  one dev-agent session.

## Future phases (direction only — deliberately shallow)

Sketched so nothing in phase one forecloses them; no FRs, stories, or
abstractions may be created for them yet.

- **Phase 2 candidates:** mouse/touch input (iPad over SSH — see addendum),
  needs/moods, more jobs and items.
- **LLM dwarf whimsy (named phase-2+ candidate):** a local-LLM flavor layer
  that makes dwarves funnier and less predictable — quirks, impulses, saga
  narration. The architectural boundary is firm: never inside `sim-core`.
  The LLM runs as an async sidecar/client; its outputs enter the sim only as
  ordinary logged inputs (commands/impulses), so replay and scenario tests
  stay deterministic — tests substitute scripted stand-ins. The "dwarves
  obey in their own time" gap that phase one creates (FR5) is exactly the
  space this layer later occupies. Mechanism sketch in the addendum.
- **Later:** larger maps with chunking, hierarchical pathfinding, FlatBuffers
  protocol with interest management, true multi-machine server + client play,
  and the Unreal client — the long-term visual target for the icy-grim world.
- **Pathfinding vs. future GUI clients:** nothing to prepare now. Clients
  never path — they render the per-tick positions the sim reports (a GUI
  client may interpolate between ticks visually, which is pure presentation).
  Hierarchical pathfinding remains a sim-core-internal swap triggered by map
  size, not by client type; plain A* (FR11) forecloses nothing.
- **Visual identity (decided, not built):** creatures stay single
  profession-colored glyphs in 2D; future 3D views sample code-authored
  sub-voxel character models — no sprites or per-creature art assets, ever.
  Model dimensions, LOD strategy, and identity mechanics are recorded in the
  addendum.
- **Wild card (unscheduled, one line by design):** visualizing external live
  systems — e.g. the Asgard realm — by feeding their state through an adapter
  that speaks protocol v0; every client then renders them unchanged.

## Assumptions index

Inline `[ASSUMPTION]` tags, collected for the architecture agent's
confirm-or-override pass:

- **FR2** — terrain height variation is modest rolling hills, a few z-levels.
- **FR3** — "a handful of dwarves" = 5.
- **FR4** — idle wander radius ~3 tiles.
- **FR5** — reaction delay ~0.5–3 s (5–30 ticks), seeded per dwarf per job.
- **FR14** — a single fast-forward step (≈5×) besides pause and 1×.
- **FR23** — icy-grim look is palette/glyph work inside rendering stories,
  not a separate art story.
