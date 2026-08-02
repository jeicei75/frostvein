---
stepsCompleted: [1, 2, 3]
inputDocuments:
  - _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/prd.md
  - _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md
  - docs/architecture.md
---

# frostvein - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for frostvein, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: The world is a fixed-size 3D voxel grid (default 128×128×32) with layered terrain — stone, soil, ice, snow, air — generated from a world seed; snow surfaces and ice appear in ordinary generated terrain so the world reads as frozen from first boot. No chunking or streaming.
FR2: Terrain generation produces surface height variation with walkable ramps/slopes (modest rolling height, a few z-levels), enough to exercise climb pathfinding and channel digging.
FR3: A handful of dwarves (5) spawn on the surface at world generation.
FR4: Each dwarf runs a simple job state machine: idle → walk → work, with current state visible to clients. Idle dwarves wander nearby tiles (seeded, deterministic, ~3-tile radius) so the world visibly lives with no orders given. No needs, moods, or personalities.
FR5: Idle dwarves claim the oldest unclaimed job (FIFO). One dwarf per job; a claimed job is released if the dwarf cannot complete it. A seeded per-dwarf reaction delay (~5–30 ticks, seeded per dwarf per job) passes before a job is claimed.
FR6: Dig job: a dwarf adjacent to a designated tile removes it (dig: wall becomes open floor; channel: floor is dug out leaving a ramp below) and a stone item appears at the dug location.
FR7: Haul job: a dwarf carries a loose stone item to a stockpile tile and drops it there.
FR8: A designation that is currently unreachable stays queued and is retried; it is never silently dropped (naive retry acceptable in phase one).
FR9: The player can designate tiles for digging as rectangles in two modes — dig (same-level) and channel (dig down, leaving a ramp) — and can cancel a designation before it is dug, releasing any unclaimed or in-progress job on it.
FR10: The player can place a stockpile zone as a rectangle on walkable floor.
FR11: Dwarves pathfind with plain A* on the voxel grid: walking on floors and climbing ramps/stairs between z-levels. No hierarchical pathfinding, no caching.
FR12: Stone exists as a haulable item with a world position. No materials system, quality, stacking, or containers.
FR13: The daemon runs a fixed-timestep tick loop (default 10 ticks/sec), fully decoupled from clients; the sim advances with zero clients attached.
FR14: Speed control: pause, normal (1×), and one fast-forward step (≈5×), implemented as tick-rate changes.
FR15: Determinism: identical world seed + identical command sequence produces identical sim state, tick for tick.
FR16: Dev save/load of the full sim state, plus clean quit. No save-format stability guarantees.
FR17: Protocol v0: newline-delimited JSON over localhost TCP. Full world snapshot on connect, per-tick delta messages thereafter. Messages describe a world, not a dwarf game — state and typed data, never rules or narrative.
FR18: Commands upstream: designate dig/channel, cancel designation, place stockpile, pause/resume, set tick rate, save, load, quit.
FR19: Multiple localhost clients can view the same running sim concurrently.
FR20: TUI shows a single z-level top-down view of the world, navigable between z-levels DF-style (`<`/`>`). The client contains zero game logic.
FR21: Modal, DF-familiar keyboard input: single keys enter a mode (dig, channel, stockpile), rectangles placed cursor-first with Enter-anchor / Enter-commit, `Esc` backs out, and a one-line hint bar always shows the active mode's keys (concrete keymap in the PRD addendum).
FR22: Dwarves render as `☺` glyphs colored by current job/profession; terrain and items render as distinct glyphs. 24-bit truecolor from the start; color is data (material/profession → RGB), not a fixed palette.
FR23: The visual identity is icy, gloomy, dark and grim: a cold, desaturated terrain palette with profession colors as warm accents. Acceptance instrument is Wolf's sign-off on the icy-grim look in the live TUI; palette/glyph selection happens inside existing rendering stories, not a separate art story.
FR24: The raycast 3D view is its own story late in the milestone. Required for phase one — Wolf's override (2026-08-01) of the PRD's may-slip clause; it no longer slips and is off the cut list.
FR25: Scenario tests build a world from a seed, inject commands, tick N times, and assert sim state — with no client or network attached.
FR26: The walking-skeleton sentence exists as an automated scenario test (dig designation → pathfind → dig → haul to stockpile) and is the phase-one gate.

### NonFunctional Requirements

NFR1: Platform — phase one targets the WSL2 devpod and any decent terminal emulator over SSH; no other platforms. Nothing in phase one may preclude the long-term multi-machine server + client shape, and nothing builds for it.
NFR2: Feels alive — TUI keeps pace at 10 ticks/sec with no visible stutter (~100 ms frame budget, full 128×128 z-level). A player command is acknowledged in the UI within ~200 ms (one tick + one frame). Dwarf obedience is exempt (FR5 reaction delay). Even with zero commands, the view visibly moves (idle wandering). Checkable by eye; no measurement infrastructure.
NFR3: Determinism everywhere — every feature keeps seed + command log ⇒ identical state true. Any nondeterminism source (unordered iteration, wall-clock time, unseeded randomness) in `sim-core` is a bug.
NFR4: Quality gate — every story lands with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` green.

### Additional Requirements

From the Architecture Spine (AD-1…AD-12, conventions, stack, structural seed):

- **No starter template** — greenfield: one Cargo workspace, four crates (`sim-core`, `protocol`, `simd`, `tui`). Epic 1 Story 1 must scaffold this. Fixed dependency edges only: `simd → sim-core`, `simd → protocol`, `tui → protocol`.
- AD-1: `sim-core` is a pure library — zero I/O (no net, fs, clock, terminal). Shells (`simd`, `tui`) contain zero game rules.
- AD-2: Fixed-timestep loop in `simd` that never stops: while paused, queued commands still apply and a delta still emits each iteration; only world-advancing systems skip and the tick counter freezes. Fast-forward is a loop-rate change.
- AD-3/AD-4: Protocol v0 = one JSON object per line over localhost TCP; snapshot on connect, one delta per loop iteration, commands upstream. Messages carry state and typed data, never rules.
- AD-5: Plain A* only; hierarchical pathfinding is a future sim-core-internal swap, never scaffolded now.
- AD-6: All wire types live in `protocol` and only there; closed vocabularies (materials, professions, job kinds, speeds, command types) are Rust enums, never strings. `sim-core` enums are source of truth, mirrored in `protocol`, bridged in `simd` by exhaustive `match` with no wildcard arm.
- AD-7: Determinism enforced structurally: single-threaded explicitly `.chain()`ed schedule; order-sensitive logic iterates in stable entity-id order; no `HashMap`/`HashSet` iteration affects outcomes; all randomness from the world seed via purpose-named streams (worldgen, wander); reaction delay = `hash(seed, dwarf id, job id)` with a fixed named hash, never `RandomState`.
- AD-8: Deltas = dirty tiles + ALL small state in full (entities, designations, zones, speed/pause, tick). Tiles mutate only via `World::set_tile`, which records the per-tick dirty set. Full-resend sections are authoritative replacements — absence is deletion.
- AD-9: One global monotonic `u32` id allocator for all entity kinds (never per-kind counters); ids never reused, survive save/load (allocator next-value is part of `SaveState`); job ids are a separate named space; `bevy_ecs::Entity` never leaves `sim-core`.
- AD-10: Only world-mutating commands (`designate`, `cancel_designation`, `place_stockpile`) ride the command queue, consumed at loop-iteration start in arrival order. Control commands (`set_speed`, `save`, `load`, `quit`) are handled by `simd` directly. No persistent command log in phase one.
- AD-11: Save/load = explicit serde `SaveState` struct in `sim-core` (tick, RNG stream states, tiles, entities + components, jobs + claims, designations, zones, id-allocator next value) via `to_save()`/`from_save()`; `simd` owns file I/O; load triggers a fresh `snapshot` broadcast to every client. Gate test: save → load → tick N ≡ never-saved → tick N.
- AD-12: One job market: single job list with all kinds as enum variants, monotonic job ids, exactly one claiming system at a fixed schedule point; a dwarf has one `Option<CurrentJob>`. FIFO = ascending job id; dwarves considered in ascending entity id. Job-kind stories add variants and execution systems, never claiming logic.
- Conventions: z vertical (0 = lowest); rects inclusive both corners, single z-level; bulk tile arrays flat row-major (`x + y·W + z·W·H`); wire messages have a `type` field, snake_case, positions `[x, y, z]`, ticks u64, entity ids u32; wire carries material/profession ids, never RGB — the id → RGB color table is a data table in `tui`, shared by all views, never hardcoded per draw site; no explicit ack messages — a command's effect in the next delta is the ack (meets NFR2); malformed client input is logged and dropped, sim never crashes on it; `thiserror` in `sim-core`/`protocol`, `anyhow` in `simd`/`tui`; hardcoded constants at use site (`protocol` exports `DEFAULT_PORT`); `#![forbid(unsafe_code)]` in all four crates; TUI drawing = hand-rolled cell framebuffer flushed once per frame, never per-cell writes.
- Stack (closed list; new dependency = one sentence of justification in its story): Rust stable edition 2024, bevy_ecs 0.19 headless, serde/serde_json, rand + rand_chacha 0.10 (`serde` feature for RNG-state saves), crossterm 0.29, thiserror/anyhow, `std::net` + threads (no tokio/async).
- Scenario harness lives as `sim-core` integration tests, calling the lib directly — no client or network.
- Story-count counter-metric: phase one ships in 8–12 vertically sliced stories; if planning exceeds 12, the cut list starts with FR16 (save/load). FR24 (raycast view) was removed from the cut list by Wolf's override, 2026-08-01.

### UX Design Requirements

No separate UX design contract exists (no `ux-designs/` run folders). The TUI's visual and interaction requirements are first-class PRD requirements (FR20–FR23) plus the concrete keymap in the PRD addendum:

- Keymap (addendum, binds FR21): `d` dig mode, `c` channel mode, `p` stockpile mode, `x` remove-designation mode, arrows/`hjkl` cursor, `Enter` anchor/commit rectangle, `Esc` back out one level, `Space` pause/resume, `+`/`-` tick rate, `<`/`>` z-level, `q` quit with confirm; one-line hint bar always shows the active mode's keys.

### FR Coverage Map

FR1: Epic 1 - Fixed-size seeded voxel world with icy layered terrain
FR2: Epic 1 - Surface height variation with walkable ramps/slopes
FR3: Epic 1 - 5 dwarves spawn on the surface at worldgen
FR4: Epic 2 - Job state machine idle → walk → work; seeded idle wandering
FR5: Epic 3 - FIFO job claiming with seeded per-dwarf reaction delay
FR6: Epic 3 - Dig job (dig and channel execution, stone item appears)
FR7: Epic 3 - Haul job (carry stone to stockpile)
FR8: Epic 3 - Unreachable designations stay queued and retry
FR9: Epic 3 - Rectangle dig/channel designation and cancellation
FR10: Epic 3 - Stockpile zone placement on walkable floor
FR11: Epic 3 - Plain A* pathfinding (floors, ramps/stairs across z)
FR12: Epic 3 - Stone as haulable item with world position
FR13: Epic 2 - Fixed-timestep tick loop decoupled from clients
FR14: Epic 2 - Speed control: pause, normal, fast-forward
FR15: Epic 2 - Determinism: seed + commands ⇒ identical state (established here, held by every later story)
FR16: Epic 2 - Dev save/load of full sim state, clean quit
FR17: Epic 1 - Protocol v0: NDJSON over TCP, snapshot on connect (deltas activate in Epic 2)
FR18: Epics 1–3 - Commands arrive with the feature that gives them meaning: control commands (pause/speed/save/load/quit) in Epic 2, world-mutating commands (designate/cancel/stockpile) in Epic 3
FR19: Epic 2 - Multiple localhost clients view the same sim
FR20: Epic 1 - Single z-level top-down TUI view with z-navigation
FR21: Epic 3 - Modal DF-familiar keyboard input with hint bar
FR22: Epic 1 - Glyph rendering, 24-bit truecolor, color-as-data
FR23: Epic 1 - Icy-grim visual identity (Wolf sign-off; re-checked live in Epic 2)
FR24: Epic 4 - Raycast 3D view (firm scope per Wolf's override)
FR25: Epic 2 - Headless scenario harness (foundation; exercised throughout Epic 3)
FR26: Epic 3 - Walking-skeleton scenario test — the phase-one gate

## Epic List

### Epic 1: The Frozen World on Screen
Generate the world and behold it: workspace scaffold, seeded worldgen with icy terrain and rolling height, dwarves spawned (static for now), daemon serving a snapshot on connect, TUI rendering the z-level view in the icy-grim palette with z-navigation. Earliest possible feedback on the FR23 look.
**FRs covered:** FR1, FR2, FR3, FR17 (snapshot), FR20, FR22, FR23

### Epic 2: The World Breathes
The sim lives and the session is yours: fixed-timestep tick loop with per-tick deltas, idle dwarves wandering, pause/normal/fast-forward, determinism locked in with the scenario-harness foundation, save/load/quit, multiple clients watching the same sim.
**FRs covered:** FR4, FR13, FR14, FR15, FR16, FR19, FR25, FR18 (control commands)

### Epic 3: The Boss Gives Orders
Designate a dig, watch dwarves obey in their own time, see stone reach the stockpile: modal input, designations with cancel, stockpiles, job market with reaction delays, A*, dig, haul, retry — capped by the walking-skeleton scenario test (FR26), the phase-one gate.
**FRs covered:** FR5, FR6, FR7, FR8, FR9, FR10, FR11, FR12, FR21, FR26, FR18 (world-mutating commands)

### Epic 4: The World in Three Dimensions
See the fortress with depth: the raycast 3D view, firm scope per Wolf's FR24 override.
**FRs covered:** FR24

## Epic 1: The Frozen World on Screen

Generate the world and behold it: workspace scaffold, seeded worldgen with icy terrain, dwarves spawned, daemon serving a snapshot on connect, TUI rendering the z-level view in the icy-grim palette.

### Story 1.1: A Seeded Frozen World Exists

As a developer,
I want the Cargo workspace scaffolded and a seeded voxel world generated in `sim-core`,
So that every later story builds on a deterministic frozen world with the quality gate already enforced.

**Acceptance Criteria:**

**Given** a fresh checkout,
**When** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` run,
**Then** all pass on a workspace of four crates (`sim-core`, `protocol`, `simd`, `tui`) with only the sanctioned dependency edges (`simd → sim-core`, `simd → protocol`, `tui → protocol`)
**And** all four crates carry `#![forbid(unsafe_code)]`.

**Given** a world seed,
**When** `sim-core` generates the default 128×128×32 world,
**Then** terrain is layered stone/soil/ice/snow/air with snow and ice present in ordinary surface terrain (FR1)
**And** surface height varies across a few z-levels with walkable ramps connecting them (FR2)
**And** 5 dwarves are placed on walkable surface tiles with `u32` ids from the single global monotonic allocator (FR3, AD-9).

**Given** the same seed,
**When** two worlds are generated independently,
**Then** they are tile-for-tile and entity-for-entity identical (AD-7 worldgen stream) — asserted by a `sim-core` integration test.

### Story 1.2: The Daemon Serves the World

As a player,
I want the daemon to serve the generated world over protocol v0,
So that any client can receive the full world state on connect.

**Acceptance Criteria:**

**Given** `simd` running with a seeded world,
**When** a client connects over localhost TCP (port = `protocol::DEFAULT_PORT`),
**Then** it receives exactly one `snapshot` JSON line — dims, tiles (flat row-major `x + y·W + z·W·H`), entities, designations (empty), zones (empty), speed, tick — with `type` field, snake_case values, positions as `[x, y, z]`, and closed vocabularies as enums, never strings (FR17, AD-3, AD-6).

**Given** every wire message shape,
**Then** it is defined in the `protocol` crate and nowhere else, with `sim-core` vocabulary enums bridged in `simd` by exhaustive `match` with no wildcard arm (AD-6).

**Given** a malformed line sent by a client,
**When** `simd` reads it,
**Then** it logs and drops the line and the daemon keeps running.

### Story 1.3: Behold the Frozen World

As the boss,
I want to see the icy world in my terminal in truecolor and walk its z-levels,
So that I can behold the fortress site and judge the icy-grim look.

**Acceptance Criteria:**

**Given** the daemon running,
**When** I launch `tui`,
**Then** it connects, receives the snapshot, and renders a single z-level top-down view: distinct glyphs per terrain material, dwarves as `☺` colored by profession/job state, in 24-bit truecolor (FR20, FR22).

**Given** the view,
**When** I press `<` / `>`,
**Then** the view moves one z-level down/up, clamped at world bounds
**And** `q` exits the client cleanly (with confirm).

**Given** the rendering implementation,
**Then** the id → RGB mapping is one data table in `tui` (wire carries ids, never RGB) and drawing goes through a cell framebuffer flushed once per frame — no per-cell terminal writes.

**Given** a live session on the dev machine,
**Then** the terrain palette reads cold, desaturated, icy-grim, and Wolf signs off on the look direction in the terminal (FR23 — final sign-off recurs in Epic 2 with motion).

## Epic 2: The World Breathes

The sim lives and the session is yours: fixed-timestep tick loop with per-tick deltas, idle dwarves wandering, speed control, determinism locked in with the scenario-harness foundation, save/load/quit, multiple clients.

Keymap note: the addendum keymap had no save/load keys; `S` (save) and `L` (load) were agreed with Wolf during story design (uppercase, so a slipped finger doesn't trigger them).

### Story 2.1: The World Runs on Its Own Clock

As the boss,
I want the daemon to advance the sim on a fixed timestep and stream every tick,
So that the world exists and moves independent of anyone watching.

**Acceptance Criteria:**

**Given** `simd` running,
**When** no client is attached,
**Then** the sim advances at 10 ticks/sec on a fixed timestep, driven by a single-threaded, explicitly `.chain()`ed `sim-core` schedule (FR13, AD-7).

**Given** a connected client,
**When** each loop iteration completes,
**Then** exactly one `delta` line is emitted — dirty tiles (recorded only via `World::set_tile`) plus ALL small state in full: entities, designations, zones, speed, tick (AD-8)
**And** full-resend sections are authoritative replacements — the client's set becomes exactly the list sent, absence is deletion.

**Given** the TUI receiving deltas,
**Then** a status line shows the current tick climbing live, and the frame keeps pace with no visible stutter on the full 128×128 z-level (NFR2).

### Story 2.2: Dwarves Wander the Frost

As the boss,
I want idle dwarves to wander near their spot, visibly and deterministically,
So that the world reads as alive even when I give no orders.

**Acceptance Criteria:**

**Given** a running sim,
**When** a dwarf is idle,
**Then** it wanders walkable tiles within ~3 tiles of its position, driven by the seeded wander stream — never wall clock, never unseeded randomness (FR4, AD-7)
**And** each dwarf carries the idle → walk → work state machine with its current state visible in entity wire data (FR4).

**Given** the TUI attached,
**When** dwarves wander,
**Then** their `☺` glyphs move between frames — the view visibly moves with zero commands issued (NFR2).

**Given** the scenario-harness foundation (FR25),
**When** an integration test builds a world from a seed and ticks N times — no client, no network,
**Then** running it twice yields identical sim state tick-for-tick (FR15, NFR3)
**And** the harness API (build world → inject commands → tick N → assert) is in place for later stories.

### Story 2.3: Master of Time

As the boss,
I want pause, normal, and fast-forward, with any number of terminals watching,
So that the session bends to my rhythm.

**Acceptance Criteria:**

**Given** the TUI attached,
**When** I press `Space` (pause/resume) or `+`/`-` (rate step),
**Then** a `set_speed` command goes upstream, `simd` handles it directly (never the sim queue, AD-10), and the change shows in the next delta within ~200 ms (FR14, FR18, NFR2 — the delta is the ack).

**Given** the sim paused,
**Then** the loop keeps running: deltas keep flowing, queued commands still apply, only world-advancing systems skip and the tick counter freezes (AD-2)
**And** fast-forward (≈5×) is a loop-rate change invisible to `sim-core`.

**Given** two TUI clients connected to the same daemon,
**Then** both render the same world live and a speed change from either is reflected in both (FR19).

### Story 2.4: The World Endures

As the boss,
I want to save the world, load it back, and quit cleanly,
So that a session can end without the fortress being lost.

**Acceptance Criteria:**

**Given** a running sim,
**When** I press `S` (save),
**Then** `simd` writes a `SaveState` serialized from `World::to_save()` — tick, RNG stream states, tiles, entities + components, jobs + claims, designations, zones, and the id-allocator next value — to a local file (FR16, AD-11). // NOTE: jobs + claims sections join in Story 3.2 when the job market exists — recorded epic-design trade-off; every sim-state-adding story extends SaveState.

**When** I press `L` (load),
**Then** `simd` replaces the world wholesale via `from_save()` and broadcasts a fresh `snapshot` to every connected client, which each treat as an authoritative full reset (AD-11)
**And** entity ids continue from the saved allocator value — never reused (AD-9).

**Given** the scenario harness,
**When** a test runs save → load → tick N against never-saved → tick N from the same seed,
**Then** the resulting states are identical (AD-11 gate test).

**When** I press `q` in the TUI (with confirm) or send `quit`,
**Then** the daemon and client shut down cleanly — no panic, terminal restored.

## Epic 3: The Boss Gives Orders

Designate a dig, watch dwarves obey in their own time, see stone reach the stockpile: modal input, designations with cancel, stockpiles, job market with reaction delays, A*, dig, haul — capped by the walking-skeleton scenario test (FR26), the phase-one gate.

### Story 3.1: Give the Order

As the boss,
I want to mark rectangles for digging or channeling and place stockpiles with DF-familiar modal keys,
So that my directives are recorded in the world, visibly, the moment I issue them.

**Acceptance Criteria:**

**Given** the main view,
**When** I press `d` (dig), `c` (channel), `p` (stockpile), or `x` (remove designation),
**Then** the TUI enters that mode, the cursor moves with arrows/`hjkl`, `Enter` anchors the first corner then commits the rectangle, `Esc` backs out one level, and the one-line hint bar always shows the active mode's keys (FR21, addendum keymap).

**When** a rectangle is committed,
**Then** the matching command (`designate` dig|channel, `place_stockpile`, `cancel_designation`) goes upstream, rides the AD-10 queue, and is consumed at the next loop-iteration start
**And** the designation or stockpile zone appears marked in the TUI within ~200 ms via the next delta (FR9, FR10, FR18, NFR2)
**And** rects are inclusive of both corners on a single z-level; stockpiles are only accepted on walkable floor (FR10) — a committed rect is clipped to its walkable-floor tiles (non-walkable tiles are simply not part of the zone; a rect with zero walkable tiles yields no zone). // NOTE: clip, not reject — DF-familiar and the simplest rule.

**When** I remove a designation with `x`,
**Then** the covered tiles are no longer designated and vanish from every client's view (FR9 — job release lands in Story 3.2 when jobs exist).

**Given** the scenario harness,
**Then** injected designate/place/cancel commands produce the same sim-state designations and zones as the TUI path (FR25).

### Story 3.2: The Dig

As the boss,
I want dwarves to claim dig orders in their own time, walk to the site, and dig,
So that the mountain yields stone at my command — through workers, not a remote control.

**Acceptance Criteria:**

**Given** designated tiles,
**When** each is consumed by the sim,
**Then** it becomes a job in the single job market — one list, all kinds as enum variants, monotonic job ids in their own id space (AD-12, AD-9).

**Given** idle dwarves and unclaimed jobs,
**When** the one claiming system runs at its fixed schedule point,
**Then** the oldest unclaimed job (lowest job id) goes to the eligible dwarf, dwarves considered in ascending entity id, one job per dwarf via `Option<CurrentJob>` (FR5, AD-12)
**And** a claim only happens after the dwarf's reaction delay of 5–30 ticks = `hash(seed, dwarf id, job id)` with a fixed named hash (FR5, AD-7).

**Given** a claimed dig job,
**When** the dwarf pathfinds with plain A* — walking floors, climbing ramps/stairs across z-levels, no hierarchy, no caching (FR11, AD-5) —
**Then** it walks adjacent to the tile and works it: dig turns wall into open floor; channel digs out the floor leaving a ramp below; a stone item appears at the dug location with a world position and its own entity id (FR6, FR12)
**And** its job state (idle → walk → work) and profession color change live in the TUI (FR4, FR22).

**Given** an unreachable designation,
**Then** its job stays queued and is retried — never silently dropped; a dwarf that cannot complete a claimed job releases it for reclaiming (FR8, FR5, naive retry sanctioned)
**And** cancelling a designation mid-dig releases any claimed or queued job on it (FR9).

**Given** the scenario harness,
**Then** designate → delay → claim → walk → dig is asserted headless, including the unreachable-retry and cancel-mid-dig cases (FR25).

### Story 3.3: The Haul — and the Skeleton Walks

As the boss,
I want dug stone carried to my stockpile, completing the loop I ordered,
So that the walking-skeleton sentence is true — live on screen and proven headless.

**Acceptance Criteria:**

**Given** a loose stone item and a stockpile zone,
**When** the sim generates a haul job for it in the same job market,
**Then** a dwarf claims it FIFO after its reaction delay, walks to the stone, carries it to a stockpile tile, and drops it there (FR7 — haul added as a job-kind variant + execution system, never touching claiming logic, AD-12).

**Given** the scenario harness,
**When** the walking-skeleton scenario runs — build world from seed, inject a dig designation and stockpile placement, tick N —
**Then** it asserts the full sentence: designation → pathfind → dig → stone appears → haul → stone on stockpile tile, with no client or network attached (FR26 — the phase-one gate)
**And** the same scenario replayed yields identical state (FR15).

**Given** the daemon running the same scenario live,
**When** Wolf watches it in the TUI,
**Then** the whole loop is visible end to end, meets the feel floor (NFR2), and Wolf signs off on the icy-grim look in motion (FR23, success criterion 2).

## Epic 4: The World in Three Dimensions

See the fortress with depth: the raycast 3D view, firm scope per Wolf's FR24 override.

Keymap note: `v` toggles the 2D ↔ 3D view (agreed with Wolf during story design; a plain letter passes through tmux/SSH with zero risk and matches the DF-style letter keymap — can be rebound later).

### Story 4.1: Behold the Fortress in Depth

As the boss,
I want a raycast 3D view of my fortress in the terminal,
So that I can see the icy world — terrain, dwarves, and diggings — with depth.

**Acceptance Criteria:**

**Given** the TUI attached,
**When** I press `v`,
**Then** the view toggles between the 2D top-down view and the raycast 3D view, the hint bar shows the active view's keys, and `v` returns — with designation/stockpile input remaining a 2D-view capability
**And** in the 3D view the camera can be moved and turned with the cursor keys to see the fortress from different angles.

**Given** the 3D view rendering,
**Then** it raycasts the voxel grid via DDA traversal using the same protocol state and the same `tui` id → RGB color table as the 2D view — no game logic, no second color mapping (AD-4, spine convention)
**And** it draws through the shared cell framebuffer, flushed once per frame, keeping the ~100 ms feel budget on the dev machine (NFR2).

**Given** dwarves in view,
**Then** they render as code-authored sub-voxel models (~10×5×13 boxes-as-code: boots, wide tunic, beard, helmet — the wide-and-short silhouette), sampled fine-step inside creature-flagged tiles during DDA, with distance LOD down to a single voxel far away (addendum decision)
**And** individual identity (beard/hair color) derives from the world seed as palette swaps on shared geometry — no sprites, no per-creature assets, ever.

**Given** a live session,
**When** Wolf watches the fortress in 3D — dug corridors, ramps, wandering and working dwarves,
**Then** the icy-grim identity holds in depth and Wolf signs off on the 3D look (FR23 spirit applied to FR24).
