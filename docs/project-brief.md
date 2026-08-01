# Project Brief: Voxelheim (working title)

## Vision
A Dwarf Fortress–inspired voxel colony simulation, built as a headless simulation
daemon with thin display clients. Rich modern graphics come later via a separate
Unreal client; the first client is a terminal (TUI) renderer. The sim must be
playable and enjoyable early, and remain fully testable headless.

This is a solo hobby project developed primarily by AI agents (Claude Code) under
the owner's direction, inside a WSL2 devpod. Restraint and shipped increments are
valued over completeness and generality.

## Core architectural decisions (already made — do not re-litigate)
- **Sim core**: pure Rust library. No rendering, no terminal, no networking inside it.
- **Daemon**: thin wrapper around the sim core; owns the tick loop and a TCP server.
- **Clients**: dumb displays. TUI client first. Unreal client later, out of scope now.
- **Protocol v0**: newline-delimited JSON over TCP on localhost. Full snapshot on
  connect, per-tick delta messages after, commands upstream. Chatty is fine.
- **Tick loop**: fixed timestep (default 10 ticks/sec), decoupled from any client.
  Supports pause and fast-forward by changing tick rate. Sim is deterministic given
  a seed and a command sequence.
- **ECS**: Bevy ECS used headless (bevy_ecs crate, not the full Bevy engine).

## Milestone 1 — the walking skeleton (the entire current scope)
One sentence defines done:

> A dwarf in a procedurally generated voxel map receives a dig designation,
> pathfinds to it, digs the tile, and hauls the resulting stone to a stockpile —
> all visible live in the TUI client.

Everything in milestone 1 exists to serve that sentence. Target: 8–12 vertically
sliced stories. If planning produces materially more, cut scope rather than
extending the plan.

### Milestone 1 scope (minimal versions only)
- Voxel world: fixed-size 3D grid (e.g. 128x128x32), simple layered terrain gen
  (stone/soil/air), no chunking or streaming.
- Entities: a handful of dwarves with position and a simple job state machine
  (idle → walk → work). No needs, moods, or personalities yet.
- Designations: player marks tiles to dig; a stockpile zone can be placed.
- Jobs: dig and haul, claimed by idle dwarves. Simple priority = FIFO.
- Pathfinding: plain A* on the voxel grid (walk + climb ramps/stairs is enough).
  No hierarchical pathfinding, no caching.
- Items: stone as a haulable item. No materials system, no quality, no containers.
- Daemon + protocol v0 as decided above.
- TUI client: single z-level top-down view to start; dwarves render as `☺`
  glyphs colored by current job/profession; the raycast 3D view is its own
  story late in the milestone, and may slip to milestone 2 without ceremony.
- Headless test harness: sim scenarios asserted without any client attached
  (e.g. "after N ticks the designated tile is dug and stone is in the stockpile").

## Explicit non-goals (silence is not permission — these are out)
- No Unreal or any graphical client.
- No world generation history, civilizations, or off-map anything.
- No combat, health, injuries, or body parts.
- No fluids, temperature, weather, seasons, or cave-ins.
- No needs/moods/personalities, no social systems.
- No farming, crafting chains, or production beyond dig → stone → stockpile.
- No save-format stability guarantees (a dev save/load is fine if trivial).
- No multiplayer beyond multiple localhost clients viewing the same sim.
- No performance optimization before a measured problem exists.
- No mod support, scripting, or data-driven content systems.
- No binary protocol, compression, or interest management (protocol v0 only).

## Success criteria for milestone 1
1. The walking-skeleton sentence passes as an automated headless scenario test.
2. The same scenario is watchable live in the TUI attached to the running daemon.
3. `cargo test` and `cargo clippy` are clean.
4. Total docs (PRD + architecture) remain short enough to re-read in one sitting.

## Roadmap sketch (context only — do NOT plan or design these now)
Milestone 2+: TUI raycast 3D view, needs/moods, more jobs and items, larger maps
with chunking, hierarchical pathfinding, FlatBuffers protocol, interest
management, Unreal client. Listed only so agents don't accidentally foreclose
them; no story, ADR, or abstraction should be created for them yet.

### Visual identity notes (context only — no milestone 1 work)
These decisions are made; they exist here so agents don't foreclose them:
- In the 2D top-down view, creatures are single glyphs (`☺`) colored by
  profession (e.g. miner amber, hauler teal). Color-by-profession is the
  identity system across all views.
- In future 3D views, creatures are voxel models at a finer resolution than
  terrain: each occupies one map tile, but the tile is sampled against a small
  sub-voxel character model (~10x5x13 for a dwarf: boots, wide tunic, arms,
  beard covering the chest, eyes and nose above it, helmet). Wide-and-short
  silhouette is the read.
- Character models are authored as code (box-fill commands producing a small
  3D array), never as image or sprite assets. The same model data serves the
  TUI raycaster (fine-step sampling inside creature-flagged tiles during DDA
  traversal), distance LODs (down to a single voxel far away), and, much
  later, mesh generation for the Unreal client.
- Individual identity (beard/hair color) derives from the world seed; palette
  swaps on shared geometry, no per-creature assets.
Implication for milestone 1: none, except that no design choice should assume
sprites or per-creature art assets exist.
