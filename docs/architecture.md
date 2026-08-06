# Frostvein — Architecture (phase one)

The ten-minute read. The binding contract is the architecture spine at
`_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md`
(decision rationale in `.memlog.md` beside it); if this doc and the spine ever
disagree, the spine wins.

## The shape

Core–shell: a pure, deterministic simulation library wrapped by two thin
imperative shells, speaking a shared wire language.

```
sim-core ◄── simd ──► protocol ◄── tui
```

One Cargo workspace, four crates. No other dependency edge is ever added.

| Crate | What it is |
| --- | --- |
| `sim-core` | pure library: world, ECS (headless bevy_ecs), jobs, A*, `SaveState`. Zero I/O — no net, fs, clock, or terminal. Scenario tests live here as integration tests. |
| `protocol` | serde wire types only. The single home of every message shape and closed vocabulary (materials, professions, …) — drift between daemon and client is a compile error. |
| `simd` | daemon binary: fixed-timestep loop (10 ticks/s), TCP server (`std::net` + threads), command queue, delta assembly, save-file I/O. |
| `tui` | client binary: crossterm cell framebuffer (flushed once per frame), modal DF-style input, the id → RGB color table. Zero game logic. |

## The decisions (AD-1…12, condensed)

**Boundaries.** The sim is a pure library; shells own all I/O (AD-1). Clients
render a *world*, not a dwarf game: the wire carries typed state — positions,
materials, professions — never rules (AD-4).

**Time.** The daemon loop never stops. Each iteration applies queued
world-mutating commands, then — only when not paused — runs the sim systems
and advances the tick counter; a delta goes out every iteration. So pause
freezes sim time but not command intake: designations placed while paused
appear immediately (AD-2, AD-10). Fast-forward is a loop-rate change. Control
commands (`set_speed`, `save`, `load`, `quit`) are handled by the daemon
directly and never enter the sim queue.

**Determinism** (load-bearing for the scenario harness). Single-threaded,
explicitly chained system schedule; order-sensitive logic iterates in stable
entity-id order; no `HashMap` iteration may affect outcomes; all randomness
flows from the world seed via named streams (reaction delay =
`hash(seed, dwarf, job)` with a fixed hash, never `RandomState`); no wall
clock in `sim-core` (AD-7).

**Identity.** One global monotonic `u32` id allocator for all entity kinds —
never per-kind counters. Ids are never reused, survive save/load, and are the
only entity identity on the wire; `bevy_ecs::Entity` never leaves `sim-core`
(AD-9).

**Jobs.** One job market: a single list of all job kinds, one claiming system
at a fixed schedule point, one `Option<CurrentJob>` per dwarf. FIFO means
ascending job id. Job stories add variants and execution systems, never
claiming logic (AD-12).

**Protocol.** Newline-delimited JSON over localhost TCP: snapshot on connect,
one delta per loop iteration, commands upstream; no batching or compression
until shapes stabilize (AD-3, AD-6). Deltas = dirty tiles (the grid mutates
only via `World::set_tile`, which records them) + *everything small in full* —
entities, designations, zones, speed, tick. Full-resend sections are
authoritative replacements: absence is deletion (AD-8).

**Save/load.** An explicit serde `SaveState` in `sim-core` (tiles, entities,
jobs, RNG states, id allocator, tick) via `to_save()`/`from_save()`; the
daemon owns the file I/O. Load is a wholesale replacement, so the daemon
rebroadcasts a full snapshot to every client. Gate test:
save → load → tick N ≡ never-saved → tick N (AD-11).

**Pathfinding.** Plain A* on the voxel grid, walk + ramps/stairs. Nothing
hierarchical, nothing cached (AD-5).

## Conventions worth memorizing

- z is vertical, 0 = lowest. Rects are inclusive of both corners, one
  z-level. Bulk tile arrays are flat row-major: `x + y·W + z·W·H`.
- Wire: one JSON object per line, `type` field, snake_case; positions
  `[x, y, z]`; closed vocabularies are enums, never strings; the wire carries
  material/profession ids, never RGB — the color table lives in `tui`.
- No ack messages: a command's effect in the next delta *is* the ack.
- Malformed client input: log and drop; the sim never crashes on it.
- `thiserror` in libraries, `anyhow` in binaries;
  `#![forbid(unsafe_code)]` everywhere; no single-implementation
  abstractions; hardcoded constants are fine (`protocol` exports
  `DEFAULT_PORT`).

## Protocol v0 messages

| Direction | Messages |
| --- | --- |
| client → daemon | `designate` (dig \| channel, rect) · `cancel_designation` (rect) · `place_stockpile` (rect) · `remove_stockpile` (rect) · `set_speed` (pause \| normal \| fast) · `save` · `load` · `quit` |
| daemon → client | `snapshot` (connect + after load: dims, tiles, entities, designations, zones, speed, tick) · `delta` (each iteration: dirty tiles + all small state) |

## Stack

Rust stable (edition 2024) · bevy_ecs 0.19 (headless) · serde/serde_json ·
rand + rand_chacha 0.10 (`serde` feature for RNG-state saves) ·
crossterm 0.29 · thiserror/anyhow · `std::net` + threads. Versions verified
on crates.io 2026-08-01. The list is closed: a new dependency needs one
sentence of justification in its story.

## Deferred, with triggers

Hierarchical pathfinding & chunking (map size + profile) · binary protocol /
interest management (stable shapes + measured problem) · protocol
generalization for external producers (second producer exists — Asgard) ·
LLM whimsy sidecar (phase 2+; enters through the command queue, and brings
the persistent command log with it) · tokio (a story that needs it) ·
mouse/touch input (phase 2, `tui` input layer) · raycast 3D view (late
milestone story, firm phase-one scope per Wolf's override 2026-08-01;
sub-voxel code-authored models, never sprites) ·
parallel ECS scheduling (profiled problem).
