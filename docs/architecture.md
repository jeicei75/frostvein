# Frostvein — Architecture

The ten-minute read. The binding contracts are the architecture spines at
`_bmad-output/planning-artifacts/architecture/` — `architecture-frostvein-2026-08-01/`
(phase one, AD-1…12) and `architecture-frostvein-2026-08-09/` (Milestone 2,
AD-13…18) — decision rationale in the `.memlog.md` beside each; if this doc
and a spine ever disagree, the spine wins.

## The shape

Core–shell: a pure, deterministic simulation library wrapped by thin
imperative shells, speaking a shared wire language. Clients are
mirror-then-project: a plain world mirror holds wire truth; rendering
projects it.

```
sim-core ◄── simd ──► protocol ◄── client-core ◄── tui
                          ▲              ▲
                          └──── gui ─────┘
```

One Cargo workspace, six crates (`client-core` and `gui` land in M2
stories). No other dependency edge is ever added.

| Crate | What it is |
| --- | --- |
| `sim-core` | pure library: world, ECS (headless bevy_ecs), jobs, A*, `SaveState`. Zero I/O — no net, fs, clock, or terminal. Scenario tests live here as integration tests. |
| `protocol` | serde wire types only. The single home of every message shape and closed vocabulary (materials, professions, …) — drift between daemon and client is a compile error. |
| `simd` | daemon binary: fixed-timestep loop (10 ticks/s), TCP server (`std::net` + threads), command queue, delta assembly, save-file I/O. |
| `tui` | client binary: crossterm cell framebuffer (flushed once per frame), modal DF-style input, the id → RGB color table. Zero game logic. Also the deterministic assertion instrument the review evidence discipline rests on. |
| `client-core` | (M2) the shared client library: world mirror + ALL snapshot/delta application, previous-tick entity states, rect normalization. Depends on `protocol` only; both clients consume it, neither reimplements it. |
| `gui` | (M2) Bevy 0.19 client binary: projects the mirror into an isometric voxel diorama — reconciliation, picking, camera, the kind → light/appearance tables, `--capture` instrument. Zero game logic, runs via WSLg. |

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

## Milestone 2 — the Bevy client (AD-13…18, condensed)

**One mirror.** `client-core` owns the world mirror and all snapshot/delta
application; the mirror's shape is its API — Id-keyed state, current +
previous-tick entities, per-tick change info. Clients never diff wire
messages themselves. `tui` adopts it in an M2 story and retires its in-crate
state (AD-13, AD-18 — amends AD-6's "`tui` depends on nothing else").

**Projection.** In `gui`, wire messages mutate only the mirror. Render
entities are world-projected (created/despawned solely by reconciliation,
keyed by sim id; delete-all-and-re-project must reproduce the scene) or
client-local (sky, aurora, snowfall, overlay — never world state) (AD-14).

**World-projected does not imply Id-keyed** (story 7.2 — amends AD-14's
"keyed by sim id"). AD-14 names designations and zones as world-projected in
the same sentence that says reconciliation is keyed by sim `Id`, and those two
halves cannot both hold: the wire carries no id for a designation or a zone,
only a position, and AD-8 full-resends both lists every tick. Marks are
therefore reconciled by POSITION, with their own marker components rather than
`WorldProjected(u32)` — that id space already mixes sim ids with synthetic
`terrain_id(pos, dims)` values, and a position keyed into it would collide with
terrain immediately. Everything else AD-14 asserts is unchanged and still binds
marks: they are created and despawned solely by reconciliation, they are never
`ClientLocal`, and delete-all-and-re-project must reproduce them. Recorded here
rather than left to drift, on the AD-13/AD-6 precedent above.

**Interpolation is presentation.** The projection may blend between the two
mirrored ticks; it never extrapolates or predicts. A snapshot (connect or
load) clears the previous tick — a rewind snaps, never animates (AD-15).

**M2 world content.** Trees are tiles (`TreeTrunk`/`TreeFoliage`; digging
one drops no item). Everything that glows is an entity: `kind` names the
object (+`Torch`, +`Campfire`), `light: Option<LightKind>` names the
emission (`Torch | Campfire | Lantern`) — a dwarf's lantern is the same
concept moving. That field + those variants are the entire sanctioned M2
wire diff. Appearance (RGB, radius, flicker) is a `gui` data table; the
wire never carries it (AD-16).

**Evidence ladder.** World-correctness proven headless in CI through
`client-core` (same code `gui` renders from) with the TUI as live
cross-check; `gui` logic tests run without GPU (minimal plugins); visual
truth uses the scripted `--capture` instrument — tested itself, never
golden-imaged in CI, judged by Wolf's eye against the pre-approved sign-off
artifact (AD-17). Capture tests need a render surface and stay out of
`gate.sh`.

**The bar (NFR6).** 60 fps at working zoom, ≥30 fps at full vista, full
world + all dwarves + all lights, on the WSLg devpod, read from the
frame-time overlay. Any client: command effect visible within ~200 ms.

## Conventions worth memorizing

- z is vertical, 0 = lowest. Rects are inclusive of both corners, one
  z-level — binding for commands: `client-core` holds the one normalization
  helper, `simd` validates and drops violators. Bulk tile arrays are flat
  row-major: `x + y·W + z·W·H`.
- Sim is z-up, Bevy is Y-up: exactly one `world_to_render`/`render_to_world`
  pair in `gui`, used by projection, picking, and capture; round-trip
  tested. No system does its own axis math.
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
crossterm 0.29 · thiserror/anyhow · `std::net` + threads. M2 adds: bevy
0.19.0 (full engine, `gui` only; same release train as bevy_ecs — the two
always move together; + `bevy_dev_tools` feature if the ready-made FPS
overlay is used). Versions verified on crates.io 2026-08-01 / 2026-08-09.
The list is closed: a new dependency needs one sentence of justification in
its story.

## Deferred, with triggers

Hierarchical pathfinding & chunking (map size + profile) · binary protocol /
interest management (stable shapes + measured problem) · protocol
generalization for external producers (second producer exists — Asgard) ·
LLM whimsy sidecar (phase 2+; enters through the command queue, and brings
the persistent command log with it) · tokio (a story that needs it) ·
touch input (post-M2; lands in `gui` — mouse picking made it the input
client, 2026-08-09) · parallel ECS scheduling (profiled problem) · native
Windows `gui` build (Wolf calls for it; no unix-only code in `gui`/
`client-core` keeps it reachable) · asset pipeline, MagicaVoxel via
bevy_vox_scene (a story needs authored assets — dwarves expected first;
re-verify the crate against current bevy then) · z-slice control & world-edge
treatment (story-level design-and-test, PRD addendum) · golden-image CI
(a driver-stable render path; not planned).

*(The raycast 3D view entry is gone: withdrawn with FR24 at the 2026-08-08
pivot — the 3D client is Bevy, above.)*
