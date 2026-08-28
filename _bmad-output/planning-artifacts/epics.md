---
# M1 pass (2026-08-02) completed steps 1-3. M2 pass (2026-08-09) restarts the count.
stepsCompleted: [1, 2, 3, 4]
milestone: 2
inputDocuments:
  # Milestone 1 (inherited by reference, still binding)
  - _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/prd.md
  - _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md
  - docs/architecture.md
  # Milestone 2
  - _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/prd.md
  - _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/sprint-change-proposal-2026-08-08.md
  - docs/technical-preferences.md
  - docs/narrative.md
  - docs/17d7215b-6c05-4286-b3bb-56592ca617ec.jpg
  - docs/a9d4e72b-b4c3-43f2-8a1c-e25c539fd6c1.jpg
  # Gfx pass (2026-08-28, additive — Epics 9–10 appended, nothing above regenerated)
  - _bmad-output/implementation-artifacts/deferred-work.md
  - docs/tech-art-guidelines.md
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
FR18: Commands upstream: designate dig/channel, cancel designation, place stockpile, remove stockpile, pause/resume, set tick rate, save, load, quit.
FR19: Multiple localhost clients can view the same running sim concurrently.
FR20: TUI shows a single z-level top-down view of the world, navigable between z-levels DF-style (`<`/`>`). The client contains zero game logic.
FR21: Modal, DF-familiar keyboard input: single keys enter a mode (dig, channel, stockpile), rectangles placed cursor-first with Enter-anchor / Enter-commit, `Esc` backs out, and a one-line hint bar always shows the active mode's keys (concrete keymap in the PRD addendum).
FR22: Dwarves render as `☺` glyphs colored by current job/profession; terrain and items render as distinct glyphs. 24-bit truecolor from the start; color is data (material/profession → RGB), not a fixed palette.
FR23: The visual identity is icy, gloomy, dark and grim: a cold, desaturated terrain palette with profession colors as warm accents. Acceptance instrument is Wolf's sign-off on the icy-grim look in the live TUI; palette/glyph selection happens inside existing rendering stories, not a separate art story. **Phase-one obligation MET at 3.3** (Wolf's live TUI sign-off); the icy-grim-in-depth ambition moved to Milestone 2 when 4.1b was dropped, 2026-08-08.
FR24: ~~The raycast 3D view is its own story late in the milestone.~~ **WITHDRAWN from phase one 2026-08-08**, re-homed to Milestone 2's Bevy client as an OUTCOME. Story 4.1a delivered this FR to the letter and Wolf judged the live result "quite far from wow effect"; he had wanted an isometric camera, which the FR never said — because it named a MECHANISM, not an outcome. Code kept unmerged on branch `4-1a-behold-the-fortress-in-depth`.
FR25: Scenario tests build a world from a seed, inject commands, tick N times, and assert sim state — with no client or network attached.
FR26: The walking-skeleton sentence exists as an automated scenario test (dig designation → pathfind → dig → haul to stockpile) and is the phase-one gate.

#### Milestone 2 — Bevy client (FR27–FR37)

FR IDs continue M1's global numbering; feature groups continue at F10. M1's FRs remain binding and are not restated. FR24 is delivered here, re-stated as an outcome and never as a rendering technique.

**F10 — World content that glows and grows (the one place M2 touches the sim).** All of it worldgen/sim-side, seeded, deterministic; clients render it, never invent it.

FR27: Worldgen grows pine trees on the surface — seeded, deterministic, part of world state, visible to every client (the TUI shows them as glyphs). Density and placement are worldgen tuning decisions inside the story, not FR text.
FR28: Worldgen places static warm light emitters — torches and a campfire at the dwarven starting camp. Light emitters are world state with a position; what light *looks* like is each client's concern.
FR29: Dwarves carry lanterns — a light source attached to a moving entity, deliberately the lighting system's hardest case, placed in scope as a testbed. Every dwarf simply carries one; no fuel, no pickup/drop, no economy. **First item on the M2 cut list.**
FR30: Protocol v0 vocabulary grows to carry the above (tree and light-emitter materials/entities, carried-light state) as typed world data, honouring FR17's world-not-game principle. No shape changes, only vocabulary.

**F11 — The diorama (Bevy client, the view).**

FR31: The world renders as the isometric orbitable diorama the Visual Target describes: one zoom continuum from working-close to valley-vista, camera always usable, never lost.
FR32: The cold/warm read is live — world light sources render as warm pools against the cold night palette; sky, stars, and aurora carry the far register; snow falls as pure decoration (no sim weather).
FR33: The player can slice into the mountain by z-level to see and work the underground, and can always tell which z-level they are on and what is underground vs. surface. Mechanism is chosen by testing in its story (addendum's open question), not specified here.
FR34: The world visibly lives, driven only by real sim state over the wire — dwarves move and work at the dig face, carried lanterns move with them, static lights flicker, idle dwarves wander. Zero commands issued still means visible motion (M1's FR4 aliveness, now in 3D).

**F12 — Working the fortress (input parity).**

FR35: The Bevy client reaches full TUI command parity — designate dig/channel, cancel designation, place/remove stockpile, pause/resume, tick rate, save/load, quit. Clients contain zero game logic, unchanged. **Second on the cut list: shrinks to camera + speed control if the story count runs over.**
FR36: The player can select tiles and rectangles in the 3D view with the mouse — the picking problem — including on sliced underground z-levels. M2's hardest input work and the main story-count driver. **Cut alongside FR35.**

**F13 — Client lifecycle (the boring glue).**

FR37: The Bevy client is a `protocol`-only consumer — connects, receives snapshot, applies per-tick deltas, coexists with concurrent TUI clients on the same daemon (M1's FR19). `sim-core` and `simd` need no structural change for it.

### NonFunctional Requirements

NFR1: Platform — phase one targets the WSL2 devpod and any decent terminal emulator over SSH; no other platforms. Nothing in phase one may preclude the long-term multi-machine server + client shape, and nothing builds for it.
NFR2: Feels alive — TUI keeps pace at 10 ticks/sec with no visible stutter (~100 ms frame budget, full 128×128 z-level). A player command is acknowledged in the UI within ~200 ms (one tick + one frame). Dwarf obedience is exempt (FR5 reaction delay). Even with zero commands, the view visibly moves (idle wandering). Checkable by eye; no measurement infrastructure.
NFR3: Determinism everywhere — every feature keeps seed + command log ⇒ identical state true. Any nondeterminism source (unordered iteration, wall-clock time, unseeded randomness) in `sim-core` is a bug.
NFR4: Quality gate — every story lands with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` green.

#### Milestone 2 (NFR5–NFR8)

NFR5: **No drift.** Clients never invent world state; everything visible in any client is derivable from the wire (AD-1/AD-4 restated). One deliberate carve-out: pure atmosphere — sky, aurora, snowfall, flicker animation, dig-face cosmetic chips — is client-side by design and must never acquire sim meaning silently.
NFR6: **Feels alive, Bevy bar (measured).** Sustained **60 fps at working zoom** and **≥30 fps at full vista**, full 128×128×32 world, all dwarves and all lights, **on the live vehicle — gingerspice, native-Windows `gui.exe`, NVIDIA Vulkan, `simd` in WSL over localhost** (~~on the WSLg devpod~~, corrected 2026-08-23, see the amendment below), read from the frame-time overlay.

> **NFR6 VENUE AMENDED 2026-08-23 — Milestone 2 retrospective, Wolf's ruling (action item M2-4).**
> **The numbers are unchanged and were met with headroom. The MACHINE is corrected.** Every NFR6
> clause below originally read *"on the WSLg devpod"*, and story 5.3 falsified that premise by
> measurement: **no devpod can open a window**, on any backend, stock or self-built, and both rungs
> of the fallback ladder were walked to the end. WSL2 kernel 6.18 exposes no `/dev/dri`, so wgpu-hal
> refuses the GL surface; the Vulkan rung required Mesa's Dozen built from source and then died on a
> misreported `DeviceLost` with VRAM measured flat. The one remaining lever — forcing downlevel
> limits in `gui` — was banned by 5.3's own AC9 and correctly not taken.
>
> **The bar is now set against the vehicle that exists:** `gui.exe` cross-compiled to native Windows
> on **gingerspice** (NVIDIA Vulkan), with `simd` in WSL over localhost. Clients are protocol-only
> TCP, so the crate graph is untouched.
>
> **Measured, all with headroom:** 146 fps at 5.3 (unlit envelope); **140–146 fps sustained at 5.4**
> on the full lit and snowing world (2.3× the 60-fps bar); **>143 fps at 6.1 at BOTH working zoom
> and full vista** (~2.4× and ~4.8×). **Stories 5.3, 5.4, 6.1, 6.2 and 7.1 were closed against this
> vehicle and their story files record which machine each figure came from — their AC text is
> annotated below rather than rewritten, because the ACs were met; only the venue named in the epic
> was wrong.**
 Client-agnostic ack bar: in any client, a player command's effect is visible in the issuing client within ~200 ms (one tick + one frame), met by the no-explicit-ack convention. NFR2 is TUI-specific and explicitly does **not** stretch to this client.
NFR7: **Determinism unchanged.** FR27–FR29 land inside worldgen and sim state, so seed + command log ⇒ identical state must survive them; scenario tests cover trees and light emitters like any other world state.
NFR8: **Gate grows sibling probes.** `scripts/gate.sh` gains the `gui` and `client-core` twins of the `tui` no-`sim-core`-edge probe, so the AD-1 edge stays guarded for the client that matters most.

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
- AD-10: Only world-mutating commands (`designate`, `cancel_designation`, `place_stockpile`, `remove_stockpile`) ride the command queue, consumed at loop-iteration start in arrival order. Control commands (`set_speed`, `save`, `load`, `quit`) are handled by `simd` directly. No persistent command log in phase one. // NOTE: `remove_stockpile` was added at Story 3.1 (Wolf, 2026-08-05) and this list said three until 2026-08-06 — see the spine's AD-10 amendment.
- AD-11: Save/load = explicit serde `SaveState` struct in `sim-core` (tick, RNG stream states, tiles, entities + components, jobs + claims, designations, zones, id-allocator next value) via `to_save()`/`from_save()`; `simd` owns file I/O; load triggers a fresh `snapshot` broadcast to every client. Gate test: save → load → tick N ≡ never-saved → tick N.
- AD-12: One job market: single job list with all kinds as enum variants, monotonic job ids, exactly one claiming system at a fixed schedule point; a dwarf has one `Option<CurrentJob>`. FIFO = ascending job id; dwarves considered in ascending entity id. Job-kind stories add variants and execution systems, never claiming logic.
- Conventions: z vertical (0 = lowest); rects inclusive both corners, single z-level; bulk tile arrays flat row-major (`x + y·W + z·W·H`); wire messages have a `type` field, snake_case, positions `[x, y, z]`, ticks u64, entity ids u32; wire carries material/profession ids, never RGB — the id → RGB color table is a data table in `tui`, shared by all views, never hardcoded per draw site; no explicit ack messages — a command's effect in the next delta is the ack (meets NFR2); malformed client input is logged and dropped, sim never crashes on it; `thiserror` in `sim-core`/`protocol`, `anyhow` in `simd`/`tui`; hardcoded constants at use site (`protocol` exports `DEFAULT_PORT`); `#![forbid(unsafe_code)]` in all four crates; TUI drawing = hand-rolled cell framebuffer flushed once per frame, never per-cell writes.
- Stack (closed list; new dependency = one sentence of justification in its story): Rust stable edition 2024, bevy_ecs 0.19 headless, serde/serde_json, rand + rand_chacha 0.10 (`serde` feature for RNG-state saves), crossterm 0.29, thiserror/anyhow, `std::net` + threads (no tokio/async).
- Scenario harness lives as `sim-core` integration tests, calling the lib directly — no client or network.
- Story-count counter-metric: phase one ships in 8–12 vertically sliced stories; if planning exceeds 12, the cut list starts with FR16 (save/load). FR24 (raycast view) was removed from the cut list by Wolf's override, 2026-08-01, and then **withdrawn from phase one entirely on 2026-08-08**. **Final phase-one count: 11** — the cut list was never invoked and FR16 was never at risk.

#### Milestone 2 — from the M2 Architecture Spine (AD-13…AD-18, M2 conventions, stack, structural seed)

**No starter template.** The workspace exists; M2 adds **two new crates to the existing four** — `client-core` (library, `protocol`-only dep) and `gui` (Bevy binary) — for **six** total. The M2 dependency graph is the new closed set, superseding the parent's; no edge may be added to it: `simd → sim-core`, `simd → protocol`, `client-core → protocol`, `tui → protocol`, `tui → client-core`, `gui → protocol`, `gui → client-core`.

- **AD-13 — one client mirror, in `client-core`.** A fifth crate owns the world mirror and ALL snapshot/delta application; both clients consume it and neither reimplements any of it. **`tui` adopts `client-core` in an M2 story** — its current in-crate client state is retired, not kept as a second path. That adoption story is load-bearing for this AD and **sits on no cut list**.
- **AD-14 — rendering projects the mirror; ingestion never touches the ECS.** In `gui`, wire messages mutate only the `client-core` mirror. Every render entity is exactly one of two classes: **world-projected** (terrain, dwarves, items, lights, designations, zones) or **client-local** (sky, aurora, snowfall, the NFR6 overlay, camera rigs). Reconciliation systems keyed by sim `Id` (AD-9) are the only place world-projected entities are created or despawned; deleting every world-projected entity and re-projecting must reproduce the same scene.
- **AD-15 — interpolation is presentation.** The mirror holds only states the wire delivered (current tick and the previous one). The projection layer may blend between those two for smooth motion; it never extrapolates beyond the newest tick and never predicts. A `snapshot` (connect or AD-11 load) is a world replacement: it clears previous-tick state, and nothing ever blends across it — **a rewind snaps, it is not animated.**
- **AD-16 — trees are tiles; everything that glows is an entity with a light field.** Trees are exactly two `Material` variants, `TreeTrunk` and `TreeFoliage`, occupying voxels (worldgen-seeded, blocking pathing via existing solidity rules, mutating via `set_tile`/AD-8). **Digging a tree tile removes the tile and drops no item** (Wolf's call, 2026-08-09; wood items deferred). **Snow capping is presentation**, computed by clients from material + exposure — never wire state. Every light source is an entity: `EntityKind` gains `Torch` and `Campfire`; `light: Option<LightKind>` with `LightKind = Torch | Campfire | Lantern`; a lantern-carrying dwarf is `light: Some(Lantern)` on a moving entity. The **sanctioned wire diff for all of M2 is exactly** the `light` field on `Entity` plus those enum variants — framing and mechanism unchanged. The wire carries kind identifiers only, **never RGB, radius, or flicker**. Vocabulary lands per AD-6: `sim-core` source of truth, mirrored serde enums in `protocol`, exhaustive `match` bridges.
- **AD-17 — the evidence ladder for a real renderer.** Rung 1: world-correctness proven by `client-core` asserted **headless and byte-exact in CI** (the same code `gui` renders from), with `tui` as the live cross-check on a shared daemon. Rung 2: `gui` logic (reconciliation, picking, camera, z-slice) runs **headless under minimal plugins in `cargo test`** — no GPU in CI. Rung 3: visual truth uses `gui`'s scripted capture instrument (`--capture`, Bevy screenshot API), which **has its own tests** (file exists, not black, changes when the world changes, range-checks what it came to see — exit 0 is not a result) and is **never golden-imaged in CI**; captures are the artifact for the sign-off gate's closing half, and rung 3's judge is Wolf's eye, structurally. Capture self-tests need a real render surface and are therefore **excluded from `scripts/gate.sh` and default `cargo test`** — separately invoked; the gate stays headless.
- **AD-18 — `client-core` owns the mirror's contract.** The mirror's shape is `client-core`'s API and nowhere else: world state keyed by sim `Id`, exposing current tick, **previous-tick entity states (entities only — tiles are never double-buffered)**, and per-tick change information. Providing the previous tick is a mandate on `client-core`, not a cap clients may ignore; **clients consume `client-core`'s change info and never diff wire messages themselves.** Rect handling is part of this contract: the parent's rect rule (single z-level, inclusive corners, `min ≤ max` per axis) is binding for commands, `client-core` provides the one normalization helper both clients use, and `simd` validates incoming rects and logs-and-drops violations.
- **M2 conventions:** kind → light properties (RGB, radius, flicker) is a **data table in `gui`**, sibling to `tui`'s color table, never hardcoded per draw site. `gui`'s CLI mirrors `tui`'s scripted determinism (`--capture <path>`, `--frames N`, `--z N`-style pinning) — every visual story's instrument is a command line, not a manual recipe. The NFR6 instrument is a frame-time overlay **read on screen, not felt**: `FrameTimeDiagnosticsPlugin` is a default built-in but the ready-made `FpsOverlayPlugin` needs the non-default `bevy_dev_tools` cargo feature — **the story says which**. **Exactly ONE coordinate transform pair** (`world_to_render` / `render_to_world`, sim z-up `[x,y,z]` ↔ Bevy Y-up) lives in `gui`; projection, picking, and capture all call it and a round-trip test pins it — no system does its own axis math. No unix-only code in `gui` or `client-core` (native Windows build deferred, not precluded). `bevy` and `bevy_ecs` move together on the same 0.x line, always — never two Bevy versions in one workspace.
- **Parent updates owed** (recorded, not silent): AD-6's "`tui` depends on nothing else in the workspace" is amended by AD-13; the parent's dependency-graph enumeration is superseded; the Deferred entries "Raycast 3D view" and "Mouse/touch input — confined to `tui`" and the "Unreal client" mention are stale since the 2026-08-08 pivot; `#![forbid(unsafe_code)]` now applies to all **six** crates, with `thiserror` in `client-core` and `anyhow` in `gui`.
- **Stack (M2 addition, verified 2026-08-09):** `bevy` 0.19.0 full engine, aligned with `sim-core`'s `bevy_ecs` 0.19.0 (same release train, lockfile confirmed); default features plus `bevy_dev_tools` if the ready-made FPS overlay is used; trim only on a measured problem. Frame diagnostics and the screenshot API are in default features — no third-party deps. **No other new dependencies at cold start:** meshes are built in code, no voxel crate, no asset pipeline. The closed-list rule stands — any addition needs one sentence of justification in its story.
- **Sequencing facts the structure creates (epic-planning inputs):** `client-core` must exist before either client consumes it; and **the first `gui` story proves the envelope before anything builds on it** — a Bevy window rendering at speed on this box. `glxinfo` proved GL, but wgpu prefers Vulkan via WSLg's Dozen driver, which is younger and less conformant: **unproven until run, and non-negotiable.** ~~Runtime topology is the WSL2 devpod with `gui` displaying via WSLg (D3D12 passthrough, RTX 4080 Laptop, Mesa 25.3.5).~~ **CORRECTED 2026-08-23 (M2-4): `simd` runs in the WSL2 devpod; `gui` runs as a native-Windows binary on gingerspice over localhost.** The sequencing fact itself HELD and did its job — 5.3 proved the envelope before eight stories built on it, and the answer it returned was *not here*. The venue moved; the requirement did not.
- **M2 story-count counter-metric: 10–14 stories.** Materially more means scope gets cut, not the plan extended. Cut order, decided in advance: **first FR29** (lanterns — torches and campfire still carry the warm/cold wow), **then FR35/FR36 shrink to camera + speed control**, with the TUI keeping designations until a later milestone. The `tui`-adopts-`client-core` story is explicitly not on this list.
- **"As soon as possible" has teeth:** the first boot-frame wow — world, light, aurora, no input needed — lands in the milestone's **first third**. A plan that back-loads the visual payoff is wrong, cap or no cap.
- **Parity rule, both halves:** Bevy first catches up to the TUI's features and reaches the look-and-feel bar; the TUI is not extended for Bevy-only work. But **any new sim functionality or bug fix that affects the TUI updates the TUI too** — no TUI regression ships during M2, and F10's trees and lights are rendered by both clients (TUI glyphs included).
- **Baseline:** M2 starts against today's `simd` functionality and today's seeded worldgen. More sim control is added by specific stories when a story needs it, not up front. The dwarf count stays at FR3's five (the narrative's six were scene dressing) until a story changes it.
- **Art:** procedural/code-first; no asset pipeline in the base build. Authored assets enter only when a concrete case forces the decision on the record (dwarves expected first). This **overturns, on the record**, M1's "models authored as code, never as assets, ever" — that constraint's premise no longer holds. A **tech-art-guidelines deliverable** is owed: its procedural-era half (value discipline, sky-as-illuminant, material rules) by the first `gui` visual stories; its asset-contract half arrives with the pipeline.
- **Story rules still binding (M1 `docs/technical-preferences.md`):** vertical slices, never horizontal layers; every story ends with something observable; a story fits one dev-agent session; **every story names its observability instrument in a task and tests the instrument**; a scripted capture must be reproducible and range-check its own output — **exit 0 is not a result**.
- **Decisions owed inside M2 stories (spine Deferred; the spine binds only the outcome):** the **z-slice control mechanism** (UX-DR3); the **world-edge treatment** (UX-DR12); and the **vista mountain silhouette** — should in-grid terrain give the skyline peaks backlit by the aurora within 128×128×32? M1's FR2 assumed "modest rolling hills" **for pathfinding, not for the vista register**, so this needs conscious revisiting on the record at worldgen tuning, never silent stretching. Also recorded as deferred with explicit triggers, building nothing now: native Windows build (trigger: Wolf calls for it), asset pipeline via MagicaVoxel `.vox`/`bevy_vox_scene` (trigger: a story needs authored assets — unverified against bevy 0.19, re-verify at trigger time), golden-image CI (trigger: a deterministic driver-stable render path exists; not planned), and trimming bevy features (trigger: a measured gate-time or binary-size problem).

### UX Design Requirements

No separate UX design contract exists (no `ux-designs/` run folders). The TUI's visual and interaction requirements are first-class PRD requirements (FR20–FR23) plus the concrete keymap in the PRD addendum:

- Keymap (addendum, binds FR21): `d` dig mode, `c` channel mode, `p` stockpile mode, `x` remove-designation mode, arrows/`hjkl` cursor, `Enter` anchor/commit rectangle, `Esc` back out one level, `Space` pause/resume, `+`/`-` tick rate, `<`/`>` z-level, `q` quit with confirm; one-line hint bar always shows the active mode's keys.

#### Milestone 2 — UX-DR1…UX-DR22

Still no separate UX design contract exists. The M2 PRD's **Visual Target & Game Feel** section carries that weight deliberately — it was written as the structural fix for FR24's defect (a requirement that named a mechanism instead of an outcome), so **no line below may name a rendering technique**. Each is a bar a story must clear, extracted so it becomes an acceptance criterion rather than evaporating into "make it pretty".

**The view (FR31, FR33)**

UX-DR1: A frozen mountain valley seen as an isometric diorama — you look *down into* a place, from outside, and orbit it by hand. The camera is always usable; there is no angle you get stuck in.
UX-DR2: One zoom continuum, two registers: pulled close, a working view where individual dwarves and blocks are readable; pulled out, a vista where the valley, sky, and aurora carry the frame and dwarves become warm specks. **The far register is the same view, not a mode** — pulling out changes distance, never representation.
UX-DR3: The world keeps discrete z-levels, DF-style, even in 3D; dwarves start at ground level and dig down, and the player can slice into the mountain to see and work the underground. **Open design question, decided by testing in its story:** Wolf's candidate is the mousewheel, which collides with the conventional orbit-camera zoom that UX-DR2 already claims. One wheel cannot drive both. Candidates to test — modifier+wheel for slicing, dedicated keys (`<`/`>`, TUI parity), or slice-follows-selection.

**The light — the wow mechanism (FR28, FR32)**

UX-DR4: The organising principle is **cold against warm**: a dark blue night world — snow, ice, stone, stars, a sweeping aurora — punctured by pockets of warm orange light where the dwarves are.
UX-DR5: The eye lands on the dwarven encampment **first**, and it lands there because of the warm/cold contrast, **not because of a UI marker**.
UX-DR6: Warm light sources exist *in the world* (things that glow), so the contrast is real rather than painted on.

**What the reference images bind (bars, not guidance)**

UX-DR7: **The sky is an illuminant, not a backdrop** — aurora and starlight visibly light the snow and catch on ice, and the aurora hugs the horizon rather than hanging overhead.
UX-DR8: **Snow reads as a settled cap** — white tops, bare dark flanks, loaded branches; not a uniform coat.
UX-DR9: **Work leaves evidence** — rubble and debris at the dig face (the sim's stone items, plus cosmetic chips under NFR5's carve-out), so a worked site never looks spotless.
UX-DR10: **Value discipline** — night snow stays midtone blue-grey; only emissive light approaches white. Bright moonlit snow would flatten the warm/cold read.
UX-DR11: **The cold field varies** — blue ice breaks the white expanse, so the vista reads in cold-against-cold layers rather than one white sheet.
UX-DR12: **The world reads as a miniature whose edges dissolve into the night — a raw grid edge is never visible at any zoom.** The 128×128 world shows its cut edges when the camera pulls out. **Open design question, decided by testing in the camera/atmosphere story:** fog skirt, darkness falloff at the rim, sky wrapping below the horizon line, or vignette.

**The two wow beats — both required (FR31, FR32, FR34)**

UX-DR13: **Cold boot** — the first frame is an aesthetic hit on looks alone: voxel world, dramatic lighting, aurora. No input needed.
UX-DR14: **~Thirty seconds in** — the realisation that it's *alive*: light flickers, work animates at the dig face, a dwarf picks something up and carries it. The moment a beautiful still image becomes a running simulation. **This beat is the magic; a client that only achieves UX-DR13 has failed the milestone.**

**The anti-requirements — 4.1a's six failures, inverted (each is a pass/fail bar)**

UX-DR15: *Not ugly* — the boot frame is something you'd screenshot unprompted.
UX-DR16: *Not flat* — depth reads instantly; light, shadow, and air separate near from far.
UX-DR17: *Not cluttered* — at working zoom you can tell dwarves, terrain, designations, and items apart at a glance.
UX-DR18: *Not confusing* — you always know what you're looking at, which z-level you're on, and what is underground vs. surface.
UX-DR19: *Not lifeless* — something visibly moves even when you issue nothing: work, light, weather, idle wandering.
UX-DR20: *Camera usable* — you can always reach the angle you want, and never lose the fortress.

**Interaction (FR35, FR36)**

UX-DR21: The player selects tiles and rectangles in the 3D view **with the mouse** — the picking problem — including on sliced underground z-levels, and issues the full TUI command set from the Bevy client.

**Process obligation (binds every visually subjective story)**

UX-DR22: **The sign-off gate, both halves.** *Opening:* no visually subjective story is implemented before Wolf has approved a cheap "here is what you will see" artifact for it — target frame, mock, sketch, or generated reference of *our actual world* at the framing being built, **one artifact per visual story**. *Closing:* the story is done only when Wolf has **viewed the built result live** and compared it against the approved artifact. This is the structural fix for the FR24 defect class — a spec that is meetable, implemented, and not what was wanted, which no review layer can catch by construction. **4.1a was lost at live viewing, not at spec time.** Per AD-17, `gui --capture` output serves the closing half and never replaces the opening half.

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
FR23: Epic 1 — Icy-grim visual identity. Phase-one obligation **MET** at 3.3 (Wolf's live TUI sign-off: "looks ok for 2d tui game atm"). His "need to get to the 3d first to say" was an escalation BEYOND the phase-one bar; the icy-grim-in-depth ambition moved to Milestone 2 when 4.1b was dropped, 2026-08-08. Do not read that drop as FR23 going unmet.
FR24: **WITHDRAWN from phase one 2026-08-08**, re-homed to Milestone 2 as an outcome. Epic 4 delivered 4.1a (done, kept unmerged); 4.1b dropped.
FR25: Epic 2 - Headless scenario harness (foundation; exercised throughout Epic 3)
FR26: Epic 3 - Walking-skeleton scenario test — the phase-one gate (PASSED, 2026-08-07; MILESTONE 1)

#### Milestone 2 (FR27–FR37)

FR27: Epic 5 - Seeded pine trees on the surface (sim state, rendered by both clients)
FR28: Epic 5 - Static warm light emitters — torches and campfire at the starting camp
FR29: Epic 6 - Dwarves carry lanterns (moving light source) — **first on the cut list**
FR30: Epic 5 - Protocol vocabulary growth: the `light` field plus tree/emitter enum variants (the whole sanctioned M2 wire diff)
FR31: Epic 5 - The isometric orbitable diorama: one zoom continuum, camera always usable
FR32: Epic 5 - The cold/warm read live: warm pools, night palette, sky/stars/aurora, decorative snowfall
FR33: Epic 7 - Z-slicing into the mountain, with the player always knowing their level and surface-vs-underground
FR34: Epic 6 - The world visibly lives from wire state alone: motion, work at the dig face, flicker, idle wandering
FR35: Epic 8 - Full TUI command parity from the Bevy client — **shrinks to camera + speed control if the cap is hit**
FR36: Epic 8 - Mouse picking of tiles and rectangles in 3D, including on sliced underground levels — **cut alongside FR35**
FR37: Epic 5 - The Bevy client as a `protocol`-only consumer, coexisting with concurrent TUI clients

**NFR coverage:** NFR5 (no drift) is a bar on every `gui` story, not one story's work. NFR6 lands as an instrument in Epic 5 (the envelope proof measures it; the vista bar is re-checked there) and is re-measured under full load in Epics 6 and 8. NFR7 lands with Epic 5's worldgen story. NFR8's probes land with the crates that need them — `client-core` and `gui`, both in Epic 5.

**UX-DR coverage:** Epic 5 — UX-DR1, 2, 4, 5, 6, 7, 8, 10, 11, 12, 13, 15, 16, 20. Epic 6 — UX-DR9, 14, 19. Epic 7 — UX-DR3, 17, 18. Epic 8 — UX-DR21. **UX-DR22 (the sign-off gate, both halves) binds every visually subjective story in every epic.**

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

### Epic 4: The World in Three Dimensions — **CLOSED EARLY 2026-08-08**
Delivered 4.1a (raycast depth view, `done`, deliberately **not merged**); **4.1b dropped**; **FR24 withdrawn** from phase one. Wolf judged the live depth view "quite far from wow effect" and 3D-in-TUI is abandoned. The ambition moves to a **Bevy client in Milestone 2**, which needs its own planning pass. The 2D TUI is NOT retired — it becomes the debug client and the deterministic assertion instrument.
**FRs covered:** none in phase one. FR24 withdrawn; FR23's phase-one obligation was already met at 3.3.

---

## Epic List — Milestone 2 (Bevy client)

Four epics, **11 stories** planned inside the 10–14 cap, with the slack deliberately reserved for picking (Epic 8) and for splitting either of Epic 5's two heavy crate stories (5.2, 5.3) if one overruns a dev session — split lines for both are named in Epic 5. The ordering is driven by two hard constraints, not by taste: the PRD's **first-third wow** counter-metric and the spine's two sequencing facts (`client-core` exists before either client consumes it; the `gui` envelope is proven before anything builds on it).

**CM2, stated honestly:** the boot frame lands at story 4 of 11, completing at **36%** of the milestone — inside the first-third mandate at the edge rather than comfortably, and by *effort* tighter still, since 5.2, 5.3 and 5.4 are three of the milestone's five heaviest stories and all three precede the beat. **Splitting 5.2 or 5.3 pushes the beat to story 5 of 12 (42%) and breaches CM2**, so a split is the trigger to re-check CM2 on the record, not a free move.

### Epic 5: The Cold Boot
Wolf launches the Bevy client and the first frame stops him — a frozen valley seen as an isometric diorama he can orbit, dark blue night punctured by the warm glow of the camp, aurora hugging the horizon, snow-capped pines, edges dissolving into the dark. He has issued no input and the world already looks like somewhere. Along the way the sim grows the things that glow, and both clients start reading the world through one shared mirror.
**FRs covered:** FR27, FR28, FR30, FR31, FR32, FR37
**Delivers wow beat 1 (UX-DR13).** Standalone: a beautiful viewable client; the TUI still does the commanding.

### Epic 6: The Valley Lives
Thirty seconds after the boot frame, the still image becomes a simulation. Dwarves walk and swing at the dig face, rubble accumulates where they work, torch and campfire light flickers, idle dwarves wander, and lantern light travels with the dwarf carrying it — all of it driven by real sim state over the wire, none of it invented by the client.
**FRs covered:** FR34, FR29
**Delivers wow beat 2 (UX-DR14) — the beat the PRD calls the magic.** Standalone: builds on Epic 5, needs nothing after it.

### Epic 7: Into the Mountain
Wolf slices into the mountain and sees the dig underground: he can always tell which z-level he is on and what is below ground versus on the surface, and at working zoom he can tell dwarves, terrain, designations, items, and stockpiles apart at a glance.
**FRs covered:** FR33
**Resolves the z-slice/zoom control collision by testing (UX-DR3).** Standalone: builds on Epics 5–6, needs nothing after it.

### Epic 8: The Boss Gives Orders in Three Dimensions
Wolf works the fortress from the Bevy client with the mouse — clicking and dragging rectangles to designate digs and channels, cancelling them, placing and removing stockpiles, including on sliced underground levels — plus speed, save/load, and quit. Full parity with the TUI, and the walking skeleton runs end to end in 3D.
**FRs covered:** FR35, FR36
**M2's hardest input work and the main story-count driver.** Standalone: builds on Epics 5–7; it is also the cut-list's second victim, shrinking to camera + speed control if the story cap binds.

## Epic List — the gfx pass (added 2026-08-28)

**Why this list exists:** "the gfx pass" had become a load-bearing placeholder — four deferred
items, two of 8.2's AC halves, and the standing art rule all pointed at it, and it appeared in no
plan. Meanwhile 8.3 closes M2 on Wolf's judgement of all six anti-requirement bars, a judgement
Wolf has twice refused to make against placeholder material. RULED 2026-08-28 (Wolf): the gfx
pass runs **before 8.3** — order is Epic 9, Epic 10, then 8.3 closes the milestone.

**The story cap, stated honestly:** M2 was planned at 10–14 stories and sits at 11. These two
epics add nine, which the original cap never contemplated. This is added scope ruled by Wolf, not
cap slippage: the cap governed the path to the walking skeleton, which 8.1–8.2 have delivered;
CM2's first-third wow landed long ago and is unaffected. The milestone's *close* now waits on the
look work because closing it without the look work would mean signing off UX-DR15–20 against
material every prior ruling refused to judge.

### Epic 9: A Client You Can Read
Every legibility defect on the record is closed: the campfire stops blowing out the frame, the
cell under the cursor is visible on every face, the four designation modes read apart at a
glance, and the valley has fewer trees that no longer camouflage against the ground. No new
capability — the client Wolf already has becomes one he can actually read.
**FRs covered:** none new — closes the visual halves of FR31/FR34 via UX-DR15–UX-DR20, and 8.2's
deferred AC13-rendered and AC19-reads-clearly halves.
**Every story is gated by a concrete, measured defect already in `deferred-work.md` or named by
Wolf on the record — no taste-driven tuning.** Standalone: all in today's stack, no new
dependencies. One vehicle session closes all four UX-DR22 halves.

### Epic 10: The Look Bench
Wolf designs the game's look before building it: a headless Blender bench renders "here is what
you will see" artifacts from committed scripts, BlenderMCP on gingerspice gives him a live
creative seat with Claude, the guidelines grow an asset contract — and the bench proves itself by
making the trees look right, then delivers the first authored assets: dwarves.
**FRs covered:** none new — this is the PRD's own asset-pipeline trigger firing ("dwarves are the
expected first case") plus UX-DR22's opening-artifact obligation gaining repeatable machinery.
**RULED 2026-08-28 (Wolf): the pipeline is Blender → glTF via Bevy's native loader** — this
supersedes the addendum's recorded MagicaVoxel/`bevy_vox_scene` path (note added there). Game
first; the pipeline is an outcome, not the goal. Standalone: builds on Epic 9's legible client.

---

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

// NOTE: (Epic 2 dependency sweep, 2026-08-03) nothing mutates a tile until Story 3.2's dig — 2.2's wandering moves entities, which are full-resend — so the dirty-tile section is provably always empty in Epics 2. Wolf's decision 2026-08-03: **build `World::set_tile` + the dirty set here per AD-8 and give it a real test producer** — integration tests call `set_tile` directly and assert the tile appears in that tick's delta and is gone the next. The mechanism ships proven rather than as dead code upheld by inspection (the mistake Epic 1 avoided honestly with the `tick: 0` gap). Also settled here: the TUI blocks on `event::read()` and the daemon has no client registry — see the story file.

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

// NOTE: (Epic 2 dependency sweep, 2026-08-03) this story is a WIRE CHANGE, which the text above does not make obvious. "State visible in entity wire data" adds a field to `protocol::Entity` (today `{id, kind, pos}`), so `protocol`, the `simd` bridge, 1.2's hand-written JSON-literal tests and `tui`'s `entity_cell` all move together. It is also where AD-7's purpose-named RNG streams are born: `World` retains no RNG state today (the `ChaCha8Rng` is a local in `generate()`), so 2.2 must split worldgen/wander AND persist both on `World` — see `deferred-work.md`, whose "revisit at 2.4" trigger was corrected to 2.2.

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

// NOTE: (Epic 2 dependency sweep, 2026-08-03) this is the FIRST story in which the client writes anything. 1.3 deliberately shipped a client that sends zero bytes and left a `// NOTE:` about not closing the write half; `simd` today treats every inbound line as unrecognized by definition and logs-and-drops it. So 2.3 introduces the whole command path — `protocol` has zero command types so far — plus the `Space`/`+`/`-` keys 1.3 deliberately excluded. `protocol::Speed` exists but is entirely unused until here, which is why 1.2's review dismissed "Speed::Paused/Fast unreachable" as noise.

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
**Then** the matching command (`designate` dig|channel, `place_stockpile`, `cancel_designation`, `remove_stockpile`) goes upstream, rides the AD-10 queue, and is consumed at the next loop-iteration start // NOTE: `remove_stockpile` added on Wolf's call during 3.1 (2026-08-05) and reconciled into this text 2026-08-06; `x` is one eraser key emitting `cancel_designation` then `remove_stockpile`.
**And** the designation or stockpile zone appears marked in the TUI within ~200 ms via the next delta (FR9, FR10, FR18, NFR2)
**And** rects are inclusive of both corners on a single z-level; stockpiles are only accepted on walkable floor (FR10) — a committed rect is clipped to its walkable-floor tiles (non-walkable tiles are simply not part of the zone; a rect with zero walkable tiles yields no zone). // NOTE: clip, not reject — DF-familiar and the simplest rule.

**When** I remove a designation with `x`,
**Then** the covered tiles are no longer designated, any stockpile zone under the same rect is removed, and both vanish from every client's view (FR9 — job release lands in Story 3.2 when jobs exist).

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

**EPIC CLOSED EARLY — 2026-08-08. Status `done`. FR24 is WITHDRAWN from phase one.**

Story 4.1a shipped a raycast depth view (gate green, four review layers clean, no coverage holes). Wolf ran it live and judged it **"quite far from wow effect"**, and clarified that he had wanted an **isometric** camera, not the first-person raycast the story specified — *"I didn't manage to clarify that."* **3D-in-TUI is abandoned, including isometric-in-TUI**: terminal cells staircase diagonals badly and half-blocks with truecolor cap out near Game Boy resolution, so isometric here lands on *charming*, not *wow*.

**Disposition:** 4.1a is `done` but **deliberately NOT merged** — kept on branch `4-1a-behold-the-fortress-in-depth`, `main` stays 2D-only, consistent with the PRD counter-metric *"no code exists that serves only a future phase"*. **Story 4.1b is DROPPED** (see below). The 2D TUI client is **not** retired — it becomes the debug client and the deterministic assertion instrument.

**Where the ambition went:** Unreal was dropped the same day in favour of a **Bevy client (Milestone 2)**, which needs its own planning pass. FR24 is re-stated there **as an outcome, never as a rendering technique** — naming the mechanism in the requirement is precisely what let the wrong camera get built. FR23's icy-grim-in-depth verdict follows it there as ambition; FR23's phase-one obligation was already MET at 3.3.

**Final phase-one story count: 11** (3 + 4 + 3 + 1), inside the 8–12 cap — the cut list was never invoked and FR16 (save/load) was never at risk. Verify with `rg -c '^### Story ' _bmad-output/planning-artifacts/epics.md`.

Keymap note (historical): `v` toggles the 2D ↔ 3D view. Lives only on 4.1a's unmerged branch; `main`'s keymap does not include it.

**PREREQUISITE THAT OUTLIVED THIS EPIC — the deterministic opening camera (action item T3, closed 2026-08-08).** Keep this note: `initial` used to take the whole opening view from dwarf 0, who wanders, so two clients connecting minutes apart opened on different levels and every scripted `--key` capture aimed at a different z depending on when it ran — which cost a false "the feature does not work" verdict at story 3.3's review, with exit 0. The client now opens on the level with the most standable ground and takes `tui --z N` to pin one. **This still protects every TUI capture, and the TUI is now the project's assertion instrument, so it matters more after this epic than during it.**

**PREREQUISITE TO 4.1a, not deferred work — the opening camera must be deterministic.** Closed 2026-08-08 (action item T3) before either story is written: `initial` used to take the whole opening view from dwarf 0, who wanders, so two clients connecting minutes apart opened on different levels and every scripted `--key` capture aimed at a different z depending on when it ran — which cost a false "the feature does not work" verdict at story 3.3's review, with exit 0. The client now opens on the level with the most standable ground and takes `tui --z N` to pin one. Epic 4 is a pure-camera epic whose every AC is proven by a scripted capture, so its whole evidence base rests on this.

### Story 4.1a: Behold the Fortress in Depth

As the boss,
I want a raycast 3D view of my fortress in the terminal,
So that I can see the icy world — terrain and diggings — with depth.

**Acceptance Criteria:**

**Given** the TUI attached,
**When** I press `v`,
**Then** the view toggles between the 2D top-down view and the raycast 3D view, the hint bar shows the active view's keys, and `v` returns — with designation/stockpile input remaining a 2D-view capability
**And** in the 3D view the camera can be moved and turned with the cursor keys to see the fortress from different angles.

**Given** the 3D view rendering,
**Then** it raycasts the voxel grid via DDA traversal using the same protocol state and the same `tui` id → RGB color table as the 2D view — no game logic, no second color mapping (AD-4, spine convention)
**And** it draws through the shared cell framebuffer, flushed once per frame, keeping the ~100 ms feel budget on the dev machine (NFR2).

**Given** dug corridors and ramps,
**When** the 3D view is shown,
**Then** terrain that changed in the 2D view is visible as depth in the 3D view from the same protocol state — no second copy of the world in the client.

**Given** a scripted capture,
**Then** the instrument pins the opening view with `--z` and range-checks a non-zero count of what it came to see before drawing any conclusion (exit 0 is not a result).

// NOTE (historical, from when 4.1b was still planned): dwarves in 4.1a render at whatever the simplest correct fidelity is — a single voxel is fine and expected. Sub-voxel models were 4.1b's whole subject. **4.1b was DROPPED 2026-08-08 and the sub-voxel idea moved to Milestone 2's Bevy client**, where geometry is native rather than simulated with character cells.

### ~~Story 4.1b: Dwarves in Depth~~ — **DROPPED 2026-08-08**

**Never started. Dropped at the Epic 4 closure, before any story file was written.**

**What it was:** dwarves rendered as code-authored sub-voxel models (~10×5×13 boxes-as-code —
boots, wide tunic, beard, helmet, the wide-and-short silhouette), sampled fine-step inside
creature-flagged tiles during DDA, with distance LOD down to a single voxel; seed-derived palette
swaps on shared geometry for individual identity; no sprites, no per-creature assets, ever. It also
carried **FR23's deferred icy-grim-in-depth sign-off** and an NFR2 budget check at full model
fidelity.

**Why it was dropped:** it existed to make the depth view read as inhabited and to settle the
identity verdict — both inside a **terminal** renderer that Wolf has now judged, live, as unable to
reach the effect he wants. Building sub-voxel *glyph* dwarf models to answer a visual-identity
question that a Bevy client answers far better, with a camera that can actually be chosen, is money
burned.

**Where its purpose went:**
- **FR23's icy-grim-in-depth verdict** → Milestone 2's Bevy client, as *ambition*. FR23's
  **phase-one obligation is already MET** — success criterion 2 asked for sign-off in the live TUI
  and Wolf gave it at story 3.3 ("looks ok for 2d tui game atm"). His "we need to get to the 3d
  first to say" was an escalation beyond the bar, and it is what created this story's obligation.
  **Do not read this drop as FR23 going unmet.**
- **The NFR2 fidelity budget** → re-set for the Bevy client when it is planned. NFR2 as written is
  TUI-specific and stays met for phase one.
- **The voxel-model idea itself** → still good, still deliberately unbuilt. It belongs to the 3D
  client, where geometry is native rather than simulated with character cells.

---

# Milestone 2 — The Bevy Client

## Epic 5: The Cold Boot

Wolf launches the Bevy client and the first frame stops him — a frozen valley seen as an isometric diorama he can orbit, dark blue night punctured by the warm glow of the camp, aurora hugging the horizon, snow-capped pines, edges dissolving into the dark. He has issued no input and the world already looks like somewhere.

Four stories, ordered by the two hard constraints: the sim grows the things that glow (5.1), both clients start reading one shared mirror (5.2), the render envelope is proven on this machine (5.3), and only then does the world become beautiful (5.4). **Wow beat 1 lands at story 4 of 11 — the story completes at 36% of the milestone, so CM2's first-third mandate is met at the edge, not comfortably.** Splitting either 5.2 or 5.3 moves the beat to story 5 of 12 (42%) and breaches CM2; **a split of 5.2 or 5.3 is therefore the trigger to re-check CM2 on the record, never a free move.** If a safety valve is needed, thin 5.3 (see its split line) rather than delaying 5.4.

**The sign-off gate (UX-DR22) applies to 5.4 and not to 5.1–5.3.** 5.3 is deliberately allowed to be ugly: grey boxes that orbit at speed are a pass. Naming that up front is what keeps 5.3 small enough to fit one session.

**Split line, named in advance (5.3 as well as 5.2).** The 10–14 cap's slack covers a split of either. 5.2 splits at the crate/mirror boundary. **5.3 splits into envelope + lifecycle** (crate, gate probe, window, backend recording, connect, mirror ingestion, concurrent TUI cross-check) **versus projection + instruments** (reconciliation, transform pair, camera, overlay, capture) — the first half alone is a legitimate observable story: a window on this machine showing real world state. "Allowed to be ugly" bounds 5.3's visual scope, not its structural scope.

**Contingency if the render envelope does not hold (5.3's negative finding).** Eight of the milestone's eleven stories sit downstream of 5.3, so the fallback is recorded now rather than decided under pressure: first force the GL backend `glxinfo` already proved (`WGPU_BACKEND`), then escalate to the spine's deferred **native Windows build** — a failed envelope is exactly the trigger its "Wolf calls for it" clause anticipates. Neither is chosen here; recording the ladder is what removes the panic.

### Story 5.1: The World Grows Things That Glow

As the boss,
I want the generated world to contain pine trees and the camp's torches and campfire,
So that the valley has something living in it and something warm in it before any client exists to light it.

**Acceptance Criteria:**

**Given** a world seed,
**When** `sim-core` generates the default 128×128×32 world,
**Then** pine trees stand on the surface as exactly two new `Material` variants — `TreeTrunk` and `TreeFoliage` — placed by a seeded worldgen stream and blocking pathing through the existing solidity rules (FR27, AD-16)
**And** torches and a campfire exist as entities at the dwarven starting camp, each with a position, `EntityKind::Torch` / `EntityKind::Campfire`, and `light: Some(..)` (FR28, AD-16)
**And** density and placement are tuned inside this story — the requirement does not specify them.

**Given** a designated tree tile,
**When** a dwarf digs it,
**Then** the tile is removed via `World::set_tile` and appears in the per-tick dirty set (AD-8)
**And** **no item is dropped** — stone comes from mineral materials and wood items are deferred (AD-16, Wolf's call 2026-08-09).

**Given** the wire,
**When** a snapshot or delta carries the new content,
**Then** the only additions are the `light: Option<LightKind>` field on `Entity` and the new `Material`, `EntityKind` and `LightKind` variants — `LightKind` gains `Torch` and `Campfire` here, with `Lantern` arriving only if FR29 ships (FR30, AD-16)
**And** the wire carries kind identifiers only — never RGB, radius, or flicker
**And** `sim-core` enums stay source of truth, mirrored in `protocol`, bridged in `simd` by exhaustive `match` with no wildcard arm (AD-6).

**Given** a TUI client attached,
**Then** trees and light emitters render as distinct glyphs drawn from the existing `tui` id → RGB data table — the parity rule's backward half, since this is a sim-level change (FR27, FR28).

**Given** the vista register and M1's FR2 assumption of "modest rolling hills" — made for pathfinding, never for the vista — 
**When** worldgen is tuned in this story,
**Then** the story states **on the record** whether in-grid terrain is shaped to give the skyline a mountain silhouette backlit by the aurora within 128×128×32, and why — the last of the spine's three decisions owed inside M2 stories, and the one it warned must be revisited consciously and **never silently stretched**
**And** the answer is cross-referenced from 5.4, which builds the vista on top of whatever this story decides.

**Given** the same seed,
**When** two worlds are generated and ticked N times,
**Then** trees, emitters and all other world state are identical tile-for-tile and entity-for-entity, asserted by `sim-core` scenario tests (NFR7, AD-7).

**Given** the observability instrument,
**When** `tui --frames N --z N` runs pinned to the camp's level,
**Then** it reports a non-zero count of tree glyphs and a non-zero count of emitter glyphs before any conclusion is drawn — exit 0 is not a result
**And** the instrument's own test shows those counts change when worldgen changes.

### Story 5.2: One Mirror, Two Clients

As a developer,
I want a `client-core` crate that owns the world mirror and all snapshot/delta application, with the TUI already running on it,
So that both clients read one truth, and the mirror's contract is proven against the client we can byte-assert before the Bevy client bets on it.

**Acceptance Criteria:**

**Given** the workspace,
**When** the `client-core` crate is added,
**Then** it depends on `protocol` only, carries `#![forbid(unsafe_code)]` and `thiserror`, and the closed dependency graph becomes exactly `simd → sim-core`, `simd → protocol`, `client-core → protocol`, `tui → protocol`, `tui → client-core` (AD-13)
**And** `scripts/gate.sh` gains a `client-core` sibling of the `tui` no-`sim-core`-edge probe (NFR8).

**Given** a snapshot or delta arriving,
**When** `client-core` applies it,
**Then** the mirror holds world state keyed by sim `Id` and exposes the current tick, previous-tick **entity** states (entities only — tiles are never double-buffered), and per-tick change information (AD-18)
**And** AD-8's client-side semantics live here and only here: full-resend sections are authoritative replacements, absence is deletion, and a `snapshot` is a world replacement that clears previous-tick state.

**Given** the rect contract,
**Then** `client-core` provides the single normalization helper both clients use — single z-level, inclusive corners, `min ≤ max` per axis
**And** `simd` validates incoming command rects and logs-and-drops violations while the sim keeps running, the malformed-input convention extended to well-formed JSON with invalid semantics (AD-18).

**Given** the TUI,
**When** this story is done,
**Then** it consumes `client-core` for all world state and its in-crate client state is **retired, not kept as a second path** (AD-13)
**And** it never diffs wire messages itself — it consumes `client-core`'s change information.

**Given** a recorded snapshot and delta sequence,
**When** `client-core` applies it headlessly in `cargo test`,
**Then** the resulting mirror is asserted byte-exact, including deletion-by-absence and snapshot-as-reset (AD-17 rung 1).

**Given** the observability instrument — a scripted TUI capture at a pinned level (`tui --frames N --z N`) against the same seed and command sequence,
**When** it is run before and after adoption,
**Then** the output is identical, which is what makes this story observable rather than a refactor
**And** the guard is shown to have teeth: sabotaging one mirror rule (for example, making absence stop meaning deletion) makes the comparison fail.

### Story 5.3: A Window Onto the Valley

As the boss,
I want a Bevy client that opens a window on this machine and shows the real world as voxels I can orbit,
So that the render path is proven to work here, at speed, before anything beautiful is built on it.

**This story is allowed to be ugly.** Unlit grey boxes that orbit at speed are a pass; the visual bars belong to 5.4.

**Acceptance Criteria:**

**Given** the workspace,
**When** the `gui` crate is added,
**Then** it depends on `protocol` and `client-core` only — never `sim-core` — carries `#![forbid(unsafe_code)]` and `anyhow`, and uses `bevy` 0.19.0 on the same release train as `sim-core`'s `bevy_ecs` 0.19.0 (spine stack)
**And** `scripts/gate.sh` gains the `gui` no-`sim-core`-edge probe (NFR8).

**Given** the dev machine (WSL2 devpod, WSLg, RTX 4080 Laptop, Mesa 25.3.5),
**When** `gui` launches,
**Then** a window opens and renders continuously

> **ANSWERED 2026-08-14, and the answer was NO — this AC did its job.** The window did not open on
> the devpod on any backend. That finding is the story's deliverable, not a failure: it is why the
> live vehicle is native Windows. The AC stands as written for the record; NFR6's venue amendment
> above carries the consequence.

**And** the story records **which wgpu backend actually initialised** — WSLg's Vulkan/Dozen path is younger and less conformant than the GL path `glxinfo` proved, and is unproven until run
**And** if the envelope does not hold, that is this story's finding and it is reported, never worked around silently in production code.

**Given** the daemon running,
**When** `gui` connects,
**Then** it receives the snapshot and applies per-tick deltas through `client-core`, with wire messages mutating only the mirror and never the ECS (AD-14, FR37)
**And** a `tui` client attached to the same daemon at the same time shows the same world (FR19, FR37).

**Given** the mirror,
**When** the world is projected,
**Then** terrain, dwarves, items and emitters render as world-projected entities at the simplest correct fidelity, and every render entity is exactly one of two classes — world-projected or client-local (AD-14)
**And** reconciliation systems keyed by sim `Id` are the only place world-projected entities are created or despawned
**And** despawning every world-projected entity and re-projecting reproduces the same scene, asserted headlessly under minimal plugins in `cargo test` with no GPU (AD-14, AD-17 rung 2).

**Given** the two coordinate systems,
**Then** exactly one transform pair (`world_to_render` / `render_to_world`) exists in `gui` for sim z-up `[x,y,z]` ↔ Bevy Y-up; projection and capture both call it, no system does its own axis math, and a round-trip test pins it (spine convention).

**Given** the view,
**When** I orbit and zoom,
**Then** the camera looks down into the world isometrically from outside, I can always reach the angle I want, and I never lose the fortress (FR31, UX-DR1, UX-DR20).

**Given** the NFR6 instrument,
**Then** a frame-time overlay is readable on screen — the story states whether it uses the ready-made `FpsOverlayPlugin` (which needs the non-default `bevy_dev_tools` feature) or a hand-rolled overlay — and the measured figure at this fidelity is recorded as the baseline (NFR6)
**And** the overlay is **toggleable and off by default in `--capture` output**: captures are the sign-off gate's closing artifact (AD-17 rung 3, UX-DR22) and a burnt-in fps counter both spoils the artifact and gives the instrument's "changes when the world changes" self-test a false positive for the wrong reason.

**Given** the observability instrument,
**When** `gui --capture <path> --frames N` runs,
**Then** it writes an image file, and its own tests assert the file exists, is not black, changes when the world changes, and range-check what it came to see (AD-17 rung 3)
**And** those self-tests are excluded from `scripts/gate.sh` and default `cargo test` because they need a real render surface — the gate stays headless.

### Story 5.4: The Cold Boot

As the boss,
I want the first frame of the Bevy client to be a frozen valley at night — warm camp light against cold dark, aurora on the horizon,
So that I want to keep looking at it before I have issued a single command.

**Sign-off gate, opening half (UX-DR22): Wolf approves a "here is what you will see" artifact of our actual world at this framing before implementation starts. The story is done only when he has viewed the built result live and compared it against that artifact.**

**Acceptance Criteria:**

**Given** the world at boot,
**Then** the palette is a dark blue night world — snow, ice, stone, stars — and the camp's torches and campfire read as warm orange pools of light against it (FR32, UX-DR4, UX-DR6)
**And** the eye lands on the dwarven encampment first because of the warm/cold contrast, with no UI marker doing that work (UX-DR5)
**And** depth reads instantly: light, shadow and air separate near from far (UX-DR16).

**Given** the sky,
**Then** it is an illuminant and not a backdrop — aurora and starlight visibly light the snow and catch on ice, and the aurora hugs the horizon rather than hanging overhead (UX-DR7)
**And** sky, stars, aurora and falling snow are client-local entities with no sim meaning, sanctioned by NFR5's carve-out and never acquiring sim meaning silently (FR32, AD-14, AD-15).

**Given** the terrain,
**Then** snow reads as a settled cap — white tops, bare dark flanks, loaded branches, not a uniform coat — computed by the client from material and exposure, never from wire state (UX-DR8, AD-16)
**And** blue ice breaks the white expanse so the cold field reads in layers rather than as one white sheet (UX-DR11)
**And** night snow stays midtone blue-grey while only emissive light approaches white, since bright moonlit snow would flatten the warm/cold read (UX-DR10).

**Given** the zoom continuum,
**When** I pull out to full vista,
**Then** the valley, sky and aurora carry the frame and dwarves become warm specks; pulled close, individual dwarves and blocks are readable — **the same view at a different distance, never a different representation** (FR31, UX-DR2)
**And** at no zoom is a raw grid edge visible: the world reads as a miniature whose edges dissolve into the night, by a treatment chosen by testing here from the addendum's candidates — fog skirt, darkness falloff at the rim, sky wrapping below the horizon, or vignette (UX-DR12)
**And** the vista is built on 5.1's recorded silhouette decision rather than re-opening it: if 5.1 declined the mountain skyline, this story works with the horizon it has and says so.

**Given** the light appearance,
**Then** kind → light properties (RGB, radius, flicker) is a data table in `gui` keyed by `LightKind`, sibling to `tui`'s color table, never hardcoded per draw site (AD-16, spine convention).

**Given** the full 128×128×32 world with all dwarves and all lights on the live vehicle,
**When** the frame-time overlay is read,
**Then** it shows a sustained 60 fps at working zoom and ≥30 fps at full vista (NFR6).

> **Met on the live vehicle (gingerspice / native-Windows `gui.exe` / NVIDIA Vulkan), not the WSLg devpod — see NFR6's venue amendment. The story file records the machine for its figure.**

**Given** the tech-art guidelines deliverable,
**Then** its procedural-era half — value discipline, sky-as-illuminant, and the material rules this story settles — is written down as those decisions are made, not reconstructed later (spine Deferred).

**Given** the observability instrument built at 5.3,
**When** `gui --capture <path> --frames N` runs at the boot framing,
**Then** it produces the reproducible artifact of this story's headline outcome — the frame Wolf judges — with the fps overlay off (5.3's toggle)
**And** it range-checks what it came to see before any conclusion is drawn: a non-zero count of warm-lit emitter entities in frame and a non-black, non-uniform image — **exit 0 is not a result** (AD-17 rung 3, story rules)
**And** the capture is retained beside the artifact Wolf approved at the gate's opening half, so the comparison the closing half demands is against two images rather than a memory.

**Given** the sign-off gate's closing half,
**Then** Wolf has viewed the built result live, compared it against the artifact he approved before implementation, and signed off **wow beat 1**: the boot frame is something he would screenshot unprompted (UX-DR13, UX-DR15, UX-DR22).

## Epic 6: The Valley Lives

Thirty seconds after the boot frame, the still image becomes a simulation. Dwarves walk and swing at the dig face, rubble accumulates where they work, torch and campfire light flickers, idle dwarves wander, and lantern light travels with the dwarf carrying it — all of it driven by real sim state over the wire, none of it invented by the client.

Two stories. **6.1 delivers wow beat 2 — the beat the PRD calls the magic, and the one a client that only achieves beat 1 has failed the milestone on.** 6.2 is FR29, first on the M2 cut list; cutting it leaves Epic 6 with its wow intact, because torches and the campfire already carry the warm/cold read.

**The sign-off gate (UX-DR22) applies to both stories.**

### Story 6.1: The World Moves

As the boss,
I want the valley to visibly live — dwarves walking and working, light flickering, rubble piling at the dig face — with no command from me,
So that the beautiful still image becomes a running simulation in front of my eyes.

**Acceptance Criteria:**

**Given** the mirror holding the current tick and the previous one,
**When** the projection layer draws a frame between ticks,
**Then** entity motion is blended between those two delivered states so dwarves move smoothly rather than snapping tile to tile (AD-15, FR34)
**And** it **never extrapolates beyond the newest tick and never predicts** — the client shows only states the wire delivered
**And** the blend logic is asserted headlessly under minimal plugins in `cargo test`, including that no blend factor ever reaches past the newest tick (AD-17 rung 2).

**Given** a `snapshot` arriving on connect or after an AD-11 load,
**When** the client applies it,
**Then** it is a world replacement that clears previous-tick state, and nothing blends across it — **a rewind snaps, it is not animated** (AD-15)
**And** this is asserted headlessly, not judged by eye.

**Given** a dig designated **from a TUI client on the same daemon** — the Bevy client issues no commands until Epic 8 — at a **surface-visible dig face named in the story**, since z-slicing does not arrive until 7.1 and until then this client sees the world only from outside,
**When** I watch that dig face,
**Then** a dwarf in the working state visibly works there, and the site accumulates evidence: the sim's stone items appear as world-projected entities, alongside cosmetic chips that are client-local under NFR5's carve-out — **a worked site never looks spotless** (FR34, UX-DR9, AD-14)
**And** the site is chosen so the camera can see it without slicing: a dig aimed into or under the mountain has its face occluded by the terrain being dug, which is story 3.3's false failure repeated — a capture aimed somewhere world-dependent returned zero of every glyph with exit 0, indistinguishable from a broken feature.

**Given** torches and the campfire,
**Then** their light flickers as client-side animation with no sim meaning, driven from the `gui` light data table and never from the wire (FR34, AD-15, AD-16).

**Given** a session where I issue no commands at all,
**When** I watch for thirty seconds,
**Then** something visibly moves — idle dwarves wander, work continues, light flickers — because M1's FR4 aliveness is now visible in 3D (FR34, UX-DR19).

**Given** the full world with all dwarves moving and all lights animating on the live vehicle,
**When** the frame-time overlay is read,
**Then** it still shows a sustained 60 fps at working zoom and ≥30 fps at full vista (NFR6).

> **Met on the live vehicle (gingerspice / native-Windows `gui.exe` / NVIDIA Vulkan), not the WSLg devpod — see NFR6's venue amendment. The story file records the machine for its figure.**

**Given** the observability instruments,
**When** `gui --capture <path> --frames N` runs across a span of ticks, aimed at the named surface-visible dig site,
**Then** successive captures differ where the sim changed, and the instrument range-checks what it came to see — a non-zero count of working dwarves and of accumulated rubble at that site — rather than reporting exit 0 (AD-17 rung 3)
**And** a `tui` client on the same daemon is the live cross-check that the motion reflects real sim state and not client invention (AD-17 rung 1).

**Given** the sign-off gate's closing half,
**Then** Wolf has viewed the built result live against his approved artifact and signed off **wow beat 2** — the moment the beautiful still image becomes a running simulation (UX-DR14, UX-DR22).

### Story 6.2: Lanterns in the Dark

As the boss,
I want each dwarf to carry a lantern whose warm light travels with them,
So that the dwarves are the warm thing moving through the cold, and the lighting system is proven on its hardest case.

**First item on the M2 cut list.** If the story cap binds, this is what goes; Epic 6 keeps its wow because torches and the campfire already carry the warm/cold read.

**Acceptance Criteria:**

**Given** worldgen,
**When** dwarves are placed,
**Then** every dwarf carries a lantern as `light: Some(LightKind::Lantern)` on the dwarf entity — **no fuel, no pickup or drop, no economy** (FR29, AD-16)
**And** `LightKind` gains its `Lantern` variant here, the last piece of M2's sanctioned wire diff (FR30, AD-16).

**Given** the TUI,
**Then** no TUI rendering change is required and that reasoning is recorded rather than assumed: every dwarf carries one uniformly, so a lantern glyph would distinguish nothing. The field still flows through `client-core`'s mirror to both clients (parity rule).

**Given** the same seed and command sequence,
**When** scenario tests run,
**Then** lantern state is identical run to run like any other world state (NFR7).

**Given** a dwarf walking through the dark,
**When** I watch in the Bevy client,
**Then** a warm pool of light travels with them, lighting the terrain they pass — a moving light source, deliberately the lighting system's hardest case (FR29, FR32)
**And** its appearance comes from the `gui` data table keyed by `LightKind`, never from the wire and never hardcoded per draw site (AD-16).

**Given** all five dwarves carrying moving lights plus every static emitter on the live vehicle,
**When** the frame-time overlay is read,
**Then** NFR6 still holds — 60 fps at working zoom, ≥30 fps at full vista — and if it does not, that measurement is the story's finding and is reported (NFR6).

> **Met on the live vehicle (gingerspice / native-Windows `gui.exe` / NVIDIA Vulkan), not the WSLg devpod — see NFR6's venue amendment. The story file records the machine for its figure.**

**Given** the observability instrument,
**When** `gui --capture <path> --frames N` runs across a span of ticks while a dwarf walks through dark terrain,
**Then** the lit region **moves with the dwarf** between captures — the headline outcome observed, not inferred from the light field existing on the wire
**And** it range-checks a non-zero count of lit terrain at the dwarf's successive positions before any conclusion is drawn — exit 0 is not a result (AD-17 rung 3, story rules).

**Given** the sign-off gate,
**Then** Wolf approved a "here is what you will see" artifact before implementation and has viewed the built result live against it (UX-DR22).

## Epic 7: Into the Mountain

Wolf slices into the mountain and sees the dig underground: he can always tell which z-level he is on and what is below ground versus on the surface, and at working zoom he can tell dwarves, terrain, designations, items and stockpiles apart at a glance.

Two stories. 7.1 resolves the addendum's open control-collision question by testing. 7.2 renders designations and stockpile zones — **issued from a TUI client on the same daemon**, which is why it belongs here rather than in Epic 8: it proves the Bevy client renders them with zero game logic of its own, and it means the rendering survives if Epic 8's input work is cut.

**The sign-off gate (UX-DR22) applies to both stories.**

### Story 7.1: Slice Into the Mountain

As the boss,
I want to slice into the mountain by z-level and always know which level I am on,
So that I can see and work the underground the dwarves are digging into.

**Acceptance Criteria:**

**Given** the diorama and the addendum's open design question,
**When** the control mechanism is chosen,
**Then** it is chosen **by testing, on the record** — the story states what was tried and why the winner won, from the candidates: modifier+wheel, dedicated keys (`<`/`>`, TUI parity), or slice-follows-selection (FR33, UX-DR3)
**And** the collision is resolved explicitly: the mousewheel is the conventional orbit-camera zoom that UX-DR2's continuum already claims, and **one wheel cannot drive both**
**And** behaviour *above* ground level is tested deliberately, since that is the case Wolf flagged himself.

**Given** any slice level,
**When** I look at the view,
**Then** I always know which z-level I am on and what is underground versus on the surface, without guessing (FR33, UX-DR18)
**And** I always know what I am looking at — the anti-requirement bar for *confusing* (UX-DR18).

**Given** the slice level,
**Then** it is **client-local view state and never wire state** — the daemon does not know or care which level a client is looking at, and two clients on the same daemon can sit at different levels (NFR5, AD-14).

**Given** the slice logic,
**When** it runs headlessly under minimal plugins in `cargo test`,
**Then** which tiles are shown and hidden at level N is asserted, including clamping at world bounds, with no GPU involved (AD-17 rung 2).

**Given** dug corridors and channels underground,
**When** I slice down to them,
**Then** they are visible as the dwarves left them, projected from mirror state alone (FR33, AD-14).

**Given** the observability instrument,
**When** `gui --capture <path> --frames N --z N` runs pinned to a level — the parity of `tui --z N`, and the same lesson from story 3.3's false failure,
**Then** it range-checks a non-zero count of what it came to see at that level before any conclusion is drawn (AD-17 rung 3).

**Given** the full world at any slice level on the live vehicle,
**Then** NFR6 still holds — 60 fps at working zoom, ≥30 fps at full vista (NFR6).

> **Met on the live vehicle (gingerspice / native-Windows `gui.exe` / NVIDIA Vulkan), not the WSLg devpod — see NFR6's venue amendment. The story file records the machine for its figure.**

**Given** the sign-off gate,
**Then** Wolf approved the artifact before implementation and has viewed the built result live against it (UX-DR22).

### Story 7.2: Read the Working Zoom

As the boss,
I want designations, stockpiles, items, dwarves and terrain to be tellable apart at a glance when I am zoomed in to work,
So that the view stays readable as a working instrument instead of becoming pretty clutter.

**Acceptance Criteria:**

**Given** a TUI client on the same daemon issuing designations and placing a stockpile,
**When** I look at the Bevy client,
**Then** dig designations, channel designations and stockpile zones render as world-projected entities from mirror state alone — **the Bevy client contains zero game logic and issues no commands in this story** (AD-4, AD-14, FR37).

**Given** a dig designation and a channel designation,
**Then** they are distinguishable from each other and from undesignated terrain.

**Given** a designation cancelled from the TUI,
**When** the next delta arrives,
**Then** it disappears in the Bevy client through `client-core`'s absence-is-deletion, with no special-case code in `gui` (AD-8, AD-13).

**Given** the working zoom,
**When** I look at a busy site,
**Then** I can tell dwarves, terrain, designations, items and stockpiles apart at a glance — the anti-requirement bar for *cluttered* (UX-DR17)
**And** this legibility does not cost the cold/warm read: designations and overlays must not compete with the warm light for the eye, which still lands on the encampment first (UX-DR5).

**Given** designation and zone reconciliation,
**When** it runs headlessly under minimal plugins in `cargo test`,
**Then** entities are created, updated and despawned by sim `Id` correctly, and re-projecting from scratch reproduces the same scene (AD-14, AD-17 rung 2).

**Given** the observability instrument,
**When** `gui --capture <path> --frames N --z N` runs after a scripted TUI designation,
**Then** it range-checks a non-zero count of designation and zone entities in the capture before drawing any conclusion (AD-17 rung 3).

**Given** the sign-off gate,
**Then** Wolf approved the artifact before implementation and has viewed the built result live against it (UX-DR17, UX-DR22).

## Epic 8: The Boss Gives Orders in Three Dimensions

Wolf works the fortress from the Bevy client with the mouse — clicking and dragging rectangles to designate digs and channels, cancelling them, placing and removing stockpiles, including on sliced underground levels — plus speed, save/load and quit. Full parity with the TUI, and the walking skeleton running end to end in 3D.

Three stories, and the epic carries M2's remaining cut risk. **If the cap binds, FR35/FR36 shrink to camera + speed control**: 8.1 and 8.2 drop, 8.3 keeps speed control, and the TUI keeps designations until a later milestone — which still works, because Epic 7 already renders TUI-issued designations in the Bevy client.

**If that cut fires, story 8.3's walking-skeleton AC changes with it** and must be rewritten, not silently reinterpreted: the dig is designated **from a TUI client on the same daemon** and watched in the Bevy client. The milestone's done sentence survives the cut — Wolf watches the skeleton walk in the Bevy client — but the sentence "I designate a dig in the Bevy client" does not, and an AC that cannot be met as written is this project's most frequently caught defect class.

8.1 is split from 8.2 deliberately: the spine names picking as M2's hardest input work and the main story-count driver, so the risk is isolated in a story of its own. 8.1's standalone user value is thin by design — a hover highlight — and **if picking proves easy, 8.1 and 8.2 collapse into one story and M2 lands at 10.**

**The sign-off gate (UX-DR22) applies to 8.3 and not to 8.1–8.2**, stated here for the same reason Epic 5 stated its exclusion rather than leaving it to inference. 8.1's hover highlight and 8.2's drag feedback are **legibility** work already governed by UX-DR17 and UX-DR18 — they add readable affordances to a look 5.4 and 7.2 already settled, so a full artifact cycle would be ceremony. 8.3 is different: it closes the milestone on Wolf's judgement of both wow beats and all six anti-requirement words, and that is the gate's closing half doing its real job.

### Story 8.1: Point at the World

As the boss,
I want to point at a block in the 3D view and see exactly which one I am pointing at,
So that I can trust where my orders will land before I give any.

**Acceptance Criteria:**

**Given** the cursor over the window,
**When** the client resolves what it is pointing at,
**Then** exactly one screen-ray-to-tile path exists in `gui` and it calls the existing `render_to_world` transform — no system does its own axis math (FR36, spine convention).

**Given** any orbit angle, any zoom in the continuum, and any slice level,
**When** I point at a visible block,
**Then** the tile identified is the one a player would say they are pointing at, **including on sliced underground levels** (FR36, UX-DR21)
**And** a hover highlight shows which tile is picked before any command is issued.

**Given** the edge cases,
**When** the cursor is over empty sky, over a tile hidden by the current slice, or outside the window,
**Then** nothing is picked — and specifically **not** a silent fallback to a default tile such as the origin, which would issue orders somewhere the player never pointed.

**Given** the picking logic,
**When** it runs headlessly under minimal plugins in `cargo test`,
**Then** known camera pose plus known screen coordinate resolves to the expected tile, asserted across orbit angles, zoom levels and slice levels, with the transform round-trip test extended to cover the picking path (AD-17 rung 2).

**Given** picking,
**Then** it is entirely client-local — no command is issued in this story and no picking state reaches the wire (NFR5).

**Given** the observability instrument,
**When** `gui --capture <path> --frames N --z N` runs with a scripted cursor position,
**Then** the highlight is visible in the capture and the instrument range-checks that it found one, rather than reporting exit 0 (AD-17 rung 3).

**Given** the full world with picking active on the live vehicle (gingerspice / native-Windows `gui.exe` / NVIDIA Vulkan, `simd` in WSL over localhost),
**Then** NFR6 still holds (NFR6).

> **CORRECTED 2026-08-23 (M2-4) BEFORE THIS STORY WAS WRITTEN.** The original text named the WSLg
> devpod — a premise falsified at 5.3. Caught at the M2 retrospective; had it survived into story
> creation it would have been the **4th consecutive epic** shipping a false technical premise
> (6.2's wire claim, 7.1's control collision, 7.2's sim-`Id` requirement were 3 for 3).

### Story 8.2: Designate with the Mouse

As the boss,
I want to drag out rectangles in the 3D view to designate digs and channels, cancel them, and place and remove stockpiles,
So that I can run the fortress from the client I actually want to look at.

**Acceptance Criteria:**

**Given** the Bevy client,
**When** I select a mode and drag a rectangle over the world,
**Then** dig and channel designations, **cancellation of a designation before it is dug**, stockpile placement and stockpile removal are all issued as the **existing** protocol commands — the full world-mutating set of FR35 and AD-10, no new command shapes, and the client contains zero game logic (FR35, FR9, AD-4, AD-10)
**And** a designation cancelled from the Bevy client disappears in both clients through `client-core`'s absence-is-deletion, the same path 7.2 proved for a TUI-issued cancel
**And** the interaction pattern (drag versus click-anchor-click-commit) is chosen by testing in this story, in the spirit of the TUI's cursor-first anchor/commit.

**Given** a rectangle I drew,
**When** the command is built,
**Then** it is normalized by `client-core`'s single rect helper — one z-level, inclusive corners, `min ≤ max` per axis — the same helper the TUI uses, never a second implementation (AD-18)
**And** `simd` validates the incoming rect and logs-and-drops a violation without crashing the sim (AD-18).

**Given** a sliced underground level,
**When** I designate there,
**Then** it works exactly as it does on the surface (FR36, UX-DR21).

**Given** any mode,
**Then** I always know which mode is active and how to leave it — the Bevy client's equivalent of the TUI's always-visible hint bar (UX-DR18).

**Given** a command I issue,
**When** the next delta arrives,
**Then** its effect is visible in this client within ~200 ms — one tick plus one frame, with no explicit ack message (NFR6 ack bar, parent convention).

**Given** the input logic,
**When** it runs headlessly under minimal plugins in `cargo test`,
**Then** the mode state machine and rect construction are asserted without a GPU (AD-17 rung 2).

**Given** the observability instruments,
**When** a scripted `gui` command sequence runs,
**Then** the resulting capture shows the designations it created, range-checked rather than assumed
**And** a `tui` client on the same daemon independently confirms the sim received exactly the intended designations — the cheap byte-assertable cross-check on the expensive renderer (AD-17 rung 1).

### Story 8.3: Master of Time, and the Skeleton Walks in 3D

As the boss,
I want to control speed, save, load and quit from the Bevy client, and watch the whole walking skeleton run there,
So that Milestone 2 is done: designate, pathfind, dig, haul — live, in the client worth looking at.

**Acceptance Criteria:**

**Given** the Bevy client,
**When** I pause, resume, change tick rate, save, load or quit,
**Then** each is issued as the existing control command and handled by `simd` directly, never through the world-mutating queue (FR35, AD-10)
**And** the daemon loop never stops while paused: sim time freezes, command intake does not (AD-2).

**Given** a load,
**When** the fresh `snapshot` broadcast arrives (AD-11),
**Then** the Bevy client replaces its world, previous-tick state is cleared, and **the rewind snaps rather than animating** — the AD-15 rule proven at 6.1, now exercised by the feature that actually causes it.

**Given** a TUI client and the Bevy client attached to the same daemon,
**When** either issues commands,
**Then** both show the same world and neither interferes with the other (FR19, FR37).

**Given** a fresh world,
**When** I designate a dig in the Bevy client and watch,
**Then** the walking skeleton runs end to end in front of me — designate, pathfind, dig, haul to stockpile — driven entirely by sim state over the wire (PRD success criterion 1).

**Given** the observability instrument,
**When** the end-to-end sequence is run as a scripted capture series,
**Then** it range-checks the stone reaching the stockpile rather than reporting exit 0 (AD-17 rung 3)
**And** the equivalent `tui` run on the same daemon confirms the same outcome (AD-17 rung 1).

**Given** the milestone,
**Then** Wolf signs off **both wow beats in one sitting** — the boot frame on looks alone, and the alive moment thirty seconds later — and confirms that none of the six 4.1a words is true of this client: ugly, flat, cluttered, confusing, lifeless, camera unusable (PRD success criteria 1 and 3, UX-DR13–UX-DR20).

---

## Epic 9: A Client You Can Read

Wolf looks at the client and can tell instantly what he is seeing — terrain from trees, dwarves
from designations, the cell under his cursor from the cells beside it — and the campfire no
longer hides any of it. Nothing new is built; the client he already has becomes one he can read.

**Added 2026-08-28.** The forcing observation is Wolf's own, from the 8.2 vehicle sessions: *"too
confusing still to understand what happens"* and *"campfire is still overblown so it hides
stuff"* — the client is a compromised instrument, and everything after this epic (the bench's
tree verdicts, 8.3's six-bar sign-off) is a judgement made through that instrument. Every story
below is gated by a concrete, measured defect, which is what the standing art rule (2026-08-22)
requires; none is taste-driven tuning.

**UX-DR22 applies to every story, both halves.** The opening artifact for 9.1 is the existing
approved 5.4 artifact plus the value-floor band; 9.2–9.4 each need a cheap "here is what you will
see" artifact before build. The closing halves of all four share **one vehicle session** —
Wolf's time at gingerspice is the scarcest resource this plan spends, and it is spent once.

**Sequencing inside the epic:** 9.1 first — nothing else can be judged while an emitter that
bright is in frame, and the hover slab's invisibility near the campfire is already recorded as
downstream of it. 9.2–9.4 are order-free after that.

### Story 9.1: The Frame Stops Blowing Out

As the boss,
I want the campfire to light the camp without washing out everything near it,
So that I can see what is happening at the heart of my fortress.

The defect, measured: `04e6de5` raised campfire amplitude 0.11→0.40, peaking at 44.8M, ~40% above
what 5.4's look was sized against; open since 6.2 and re-confirmed 2026-08-27 as obstructing
observation. The bar is instrument-then-eye (RULED 2026-08-28, Wolf): the `--capture` valley-floor
median-luminance band (70–180, approved artifact reads 123) must hold, and then Wolf views it
live — the instrument catches regressions cheaply, the eye stays the authority.

**Acceptance Criteria:**

**Given** the boot camp with the campfire lit,
**When** `gui --capture` renders the approved 5.4 framing,
**Then** the valley-floor median luminance sits inside the 70–180 band, range-checked by the
instrument rather than exit 0
**And** the flicker stays inside its table-defined band, deterministic as before.

**Given** the campfire at working zoom on the vehicle,
**When** Wolf views the camp live (UX-DR22 closing half),
**Then** the fire reads as light on snow, not glare — and things adjacent to it (dwarves, marks,
the hover slab) are discernible, closing the recorded "hover slab not visible near the campfire"
observation at its root, or reopening it as evidence this story did not finish the job.

### Story 9.2: The Cell Under the Cursor Is Visible on Every Face

As the boss,
I want to see which cell I am pointing at on cliff faces, corridor walls and shaft sides,
So that I can trust where my orders will land anywhere in the fortress, not only on open ground.

**CORRECTED 2026-08-28 — THE GEOMETRIC HALF ALREADY SHIPPED.** This story was written on
2026-08-28 from `deferred-work.md`'s pre-fix entry, which had not been struck when 8.2 closed it
two days earlier. What that entry described — the 0.08-thick slab at render y `z+0.51..z+0.59`
enclosed by the cube above on every vertical face — **was fixed on 2026-08-26 by commit `8782a0d`**,
which took the first of the three candidate treatments (the hit-face slab): `sync_hover_highlight`
offsets along the picked face normal and rotates to it (`project.rs:236-238`), covered by
`a_vertical_hit_face_places_the_hover_slab_outside_the_cell_side` and four sabotage rows.

**What actually remains is the LOOK question only**, and it is already deferred: 8.2's AC13
rendered half — whether the slab READS on a cliff face, stays distinct from the marks and clear of
the near-white — was **DEFERRED by Wolf 2026-08-27** under the standing art rule, *"it will get
clearer with only real gfx"*, with REOPEN TRIGGER: real game art lands. That trigger is Epic 10.
**So 9.2 has no buildable headless work and should not be picked up before Epic 10's art pass.**

**Acceptance Criteria:**

**Given** the three candidate treatments on the record,
**When** the story opens,
**Then** Wolf approves a cheap "here is what you will see" artifact of the chosen treatment on a
cliff face **before** implementation (UX-DR22 opening half — hand-made; Epic 10's bench arrives
later).

**Given** a cliff face, a corridor wall and a shaft side,
**When** Wolf points at a cell on each (vehicle, by eye),
**Then** the treatment marks **the picked cell** visibly on all three — never a hoisted
neighbour — closing 8.2's deferred AC13 rendered half.

**Given** the headless suite,
**When** the treatment's geometry is sabotaged (face assignment, rotation, offset),
**Then** tests go red — the 8.2 lesson: the highlight's geometry was sabotage-green through a
full review.

### Story 9.3: The Four Modes Read Apart

As the boss,
I want dig, channel, stockpile and clear marks to be distinguishable at a glance at working zoom,
So that I can see what I have ordered without cross-checking in the TUI.

Closes 8.2's deferred AC19 reads-clearly half; this is UX-DR17 applied to designations. The marks
must also stay distinct from the hover highlight (9.2) and clear of the near-white reserved for
stars and emitter faces. The 8.2 readout evidence (marks split across z-levels by design) makes
the vehicle check concrete: what the `tui` count proved arrived must now be tellable apart by eye.

**Acceptance Criteria:**

**CORRECTED 2026-08-28 — THIS AC IS ALREADY MET, AND WAS UNMEETABLE AS WRITTEN.**
Already met: `mark_colours_are_distinct_cold_literals` (`appearance.rs:381`) asserts pairwise
separation at a `MIN_MARK_SEPARATION = 40.0` floor across dig/channel/zone, plus separation from
every terrain colour and from the TUI's three mark colours; `hover_highlight_colour_is_a_distinct_cold_literal`
covers hover-vs-marks and hover-vs-near-white.
Unmeetable as written: it asks for **four** mark colours pairwise separated **and** distinct from
the hover. There are only **three** persistent marks — the wire's `DesignationKind` carries just
`Dig` and `Channel`, plus the stockpile zone. **Clear commits nothing** (it deletes designations),
so it has no persistent mark at all, and its drag preview deliberately *is* the hover material
(`project.rs:381-384`). The one candidate fourth element is therefore required to be simultaneously
distinct from the hover and equal to it. Spec-defect class: "AC unmeetable as written".
**What remains is the eye check only** — 8.2's AC19 reads-clearly half, recorded in that story as
*"DEFERRED to the gfx pass"*, i.e. Epic 10.

**Given** a scene holding all four mark kinds at working zoom,
**When** Wolf views it live and tells them apart at a glance, unprompted (UX-DR17; vehicle),
**Then** 8.2's deferred AC19 reads-clearly half closes.

### Story 9.4: Trees — Fewer, and Distinct from the Ground

As the boss,
I want fewer trees, coloured apart from the terrain,
So that the valley reads as a landscape with trees in it rather than a confusion of same-coloured
blocks.

Two knobs, one story, judged together. **Density** is sim-side: `worldgen.rs` places a trunk at
1-in-12 per eligible cell with a 2-cell spacing exclusion — reduce it. **Hue** is client-side and
the defect is measurable: foliage `(55,73,84)` sits within ~8 points per channel of stone
`(60,70,92)` and soil `(56,52,62)` — trees separate from ground only by snow cap and taper, the
base cubes are near-camouflage. Shift foliage brown/green (RULED 2026-08-28, Wolf), inside the
night palette's value discipline.

**Blast radius, stated at planning so the story does not rediscover it:** the density change is
the one story in this epic that touches `sim-core` — every seeded world changes, terrain-dependent
tests and capture recipes need re-checking, mutation rows anchored near worldgen literals may
APPLY-FAIL (the gate audits this), and fewer dark tree skirts push the valley-floor luminance
*up* while 9.1 pushes it *down* — the 70–180 band watches both. **NOTE:** 9.4's colour values are
a legibility patch, not settled art — Epic 10's tree pilot may refine them through the bench.

**Acceptance Criteria:**

**Given** the same world seed,
**When** the density knob is reduced,
**Then** the default world's tree count lands in a target band agreed at story start (current
count measured first — expectation before reading), deterministic across runs.

**Given** the foliage hue shifted brown/green,
**When** tested headlessly,
**Then** foliage sits a stated minimum channel distance from stone **and** soil, while staying
inside the night palette's value discipline
**And** the `SPRUCE_SNOW` exposed-crown tests still pass untouched.

**Given** 9.1 and 9.4 push valley-floor luminance in opposite directions,
**When** `--capture` runs after both land,
**Then** the 70–180 band still holds — the interaction is measured, not assumed.

**Given** the density change touches `sim-core`,
**Then** terrain-dependent tests and capture recipes are re-checked and the gate's
mutation-apply audit runs clean — the blast radius walked deliberately, not discovered.

**Given** the valley on the vehicle,
**When** Wolf looks (UX-DR22 closing, shared session with 9.1–9.3),
**Then** it reads as a landscape with trees in it — trees tellable from ground at a glance
(UX-DR17/18).

---

## Epic 10: The Look Bench

Wolf designs the game's look before building it. A headless Blender bench turns committed scripts
into "here is what you will see" artifacts; BlenderMCP on gingerspice gives him a live creative
seat with Claude in the loop; the guidelines grow an asset contract; and the bench proves itself
on the trees before the first authored assets — dwarves — go through it.

**Added 2026-08-28. The PRD's asset-pipeline trigger has fired.** The PRD holds authored assets
until "a concrete case forces the decision — dwarves are the expected first case." RULED
2026-08-28 (Wolf): the case is here — he wants authored dwarves and better trees, and the PRD
predicted the first correctly. **The pipeline is Blender → glTF via Bevy's native loader**,
superseding the addendum's MagicaVoxel/`bevy_vox_scene` path: `.vox` from Blender means lossy
voxelisation on every iteration, and the addendum's own caveat — community voxel crates lagging
each Bevy release — is precisely what the native loader deletes. (Verified 2026-08-28:
`bevy_vox_scene` 0.22 does match Bevy 0.19 today; the supersession is about iteration cost and
maintenance surface, not a present incompatibility.) A dated supersession note lives in the
addendum itself.

**Game first; the pipeline is an outcome, not the goal** (Wolf, 2026-08-28). Each story below is
pulled in by a concrete need — UX-DR22's opening artifact (owed on every visual story, hand-made
every time so far), the tree redesign, the dwarf assets. Three concrete uses is this project's
own bar for shared machinery.

**Technical premises, verified at planning (2026-08-28), to be RE-verified at story creation:**
the workspace Bevy is 0.19.0 with a deliberate feature trim (`default-features = false`, devpod
system-library constraints) that includes **neither `bevy_gltf` nor `file_watcher`** — enabling
them is explicit story work with a justification line against the trim's reasons, not a silent
edit. No devpod can open a window; BlenderMCP requires a live GUI Blender session — so MCP work
is vehicle-side (gingerspice) by construction, and headless Blender is the only Blender any
devpod can run.

**The headless venue was PROBED at planning (2026-08-28), on Wolf's doubt, not assumed:**
Blender 4.3.2 installs from this devpod's apt; **Cycles CPU** renders headless with no GPU and no
display — 960×540 at 32 samples in **1.0 s** on 32 cores — and is **pixel-deterministic** across
runs (0 of 2,073,600 values differ; byte-level diff is PNG metadata only). One quirk on the
record: the Debian build **lacks OpenImageDenoise**, so `use_denoising = False` is mandatory —
Cycles hard-fails otherwise. **Eevee and Workbench are ruled out on this hardware** (they need a
GL context; llvmpipe is the recorded dead end). Fallback venue if the real-world scene defeats
the devpod: gingerspice, which gets Blender for 10.2 anyway — costs agent-closability, not the
bench.

**Portability is a design constraint, not a preference (Wolf, 2026-08-28):** the gfx skills are
expected to eventually move out of the Nidavellir court onto their own — and the candidate venue
is named: **Wolf's GPU-capable k3s already running on gingerspice**. That target is why the
constraint has teeth: bench scripts are self-contained and world data crosses via an explicit
export file, never by reaching into this repo's internals — if the bench cannot run in a pod, it
is not portable. What the k3s venue unlocks, recorded for the court's own plan and deliberately
NOT built now: GPU Cycles speed at scale; **headless Eevee via EGL + NVIDIA runtime** — ruled out
on the devpod but viable there, and a rasterizer's artifacts sit closer to what Bevy will
actually show, shrinking the artifact-vs-reality gap UX-DR22 polices; and remote job submission,
which restores agent-closability at the better venue. Verifying the NVIDIA container runtime on
that cluster is the future court's Task 0, vehicle-side. **The court plan itself is a separate
Nidavellir-level work item, ruled out of this project's scope (Wolf, 2026-08-28)** — planned from
a forge session, per this repo's never-write-to-the-forge rule. Its tenant list already includes
**ComfyUI** alongside Blender: UX-DR22's opening half sanctions "generated reference" as an
artifact type, so ComfyUI is a second artifact source feeding the same sign-off machinery, not a
side-tenant.

### Story 10.1: The Headless Bench

As the boss,
I want a committed script to render a reviewable image of proposed look work from our actual
world data,
So that I can judge a look before anyone builds it, and the artifact UX-DR22 demands stops being
hand-made every time.

Deterministic, diffable, runs in the devpod: `blender --background --python <script>`. Output is
the UX-DR22 opening artifact for look stories from here on. The bench renders *our* world —
worldgen-exported geometry at the real palette — not generic scenes; a Task-0 artifact that draws
geometry nobody is tasked to build is the recorded 5.4 failure and is the first thing this bench
must not repeat.

**Acceptance Criteria:**

**Given** the planning-time probe (Cycles CPU, denoising off, 1.0 s, pixel-deterministic),
**When** the bench renders the real exported world instead of a test cube,
**Then** wall-time and pixel-determinism are RE-measured at that scale and recorded — the probe
proved the venue, not the workload — and denoising stays off (the Debian build has no
OpenImageDenoise; enabling it hard-fails).

**Given** a committed script and the default world seed,
**When** `blender --background --python <script>` runs in the devpod,
**Then** it produces an image of **our actual valley** — worldgen geometry crossing via an
explicit export file at the real palette, no invented geometry, no reaching into repo internals
(portability constraint above; the 5.4 spruce-sprite failure is the standing counterexample).

**Given** a look story opening after this lands,
**Then** its UX-DR22 opening artifact comes off the bench — the hand-made-every-time era ends.

**Given** the real-world scene defeats the devpod after all,
**Then** the recorded fallback fires: the same script runs on gingerspice, the venue note is
written the way NFR6's was, and the bench survives at the cost of agent-closability.

### Story 10.2: The Live Seat — BlenderMCP on Gingerspice (SPIKE)

As the boss,
I want to explore shapes and looks interactively in Blender with Claude driving alongside me,
So that starting points for assets come out of creative sessions, not cold scripts.

**A spike: its output is a decision, not a pipeline.** BlenderMCP runs against a live GUI Blender
on gingerspice with Claude Code in the Claude app; the open question is the handoff — how a look
found live becomes a committed headless script, so nothing the build depends on lives only in a
session. The spike answers: what the handoff is, what it costs, and whether MCP earns a place in
the standing workflow or stays an exploration tool. Writing a confident AC over this unknown
would be the 4.1a shape; the AC is the decision itself, recorded.

**Acceptance Criteria:**

**Given** a live GUI Blender on gingerspice with BlenderMCP connected to Claude,
**When** Wolf runs one real exploration session (a tree or dwarf blockout),
**Then** the session happens, its output is captured, and one artifact found live is carried into
a committed headless script — the handoff proven once, end to end.

**Given** the spike closes,
**Then** its output is a **recorded decision**: what the MCP-to-script handoff is, what it costs,
and whether MCP joins the standing workflow or stays an exploration tool. No pipeline AC — the
decision is the deliverable.

### Story 10.3: The Rules of the Look

As the boss,
I want the tech-art guidelines to grow two contracts — one for procedural content, one for
authored assets,
So that output from any tool, hand, script or MCP session, can be checked against the same bar.

Extends `docs/tech-art-guidelines.md`: the procedural-content rules the existing sections imply
but never state as a contract, and the asset contract the PRD says is owed when the pipeline
opens — grid scale, orientation, origin, palette/material mapping, naming, where files live.
Blocks 10.4/10.5: no asset is authored against an undefined target.

**Acceptance Criteria:**

**Given** `docs/tech-art-guidelines.md`,
**When** this story closes,
**Then** it contains a **procedural-content contract** (the rules the existing sections imply,
stated checkably) and an **asset contract** — grid scale, orientation, origin, palette/material
mapping, naming, file locations — each concrete enough that a reviewer can check an actual asset
against it line by line.

**Given** the PRD's obligation ("a tech-art-guidelines deliverable defines the asset contract
when the pipeline opens"),
**Then** it is discharged here, on the record.

### Story 10.4: The Trees Look Right (the pilot)

As the boss,
I want the trees redesigned through the bench until the valley's trees look like trees I chose,
So that the bench proves itself on the asset I am least attached to before touching the one I
care about most.

**The procedural-vs-authored decision is this story's output, made on bench evidence** — Wolf's
standing instinct is procedural (2026-08-28, "procedural is ok but we need to make them look
better"), and the bench confirms or overturns it cheaply. Hard constraint either way: the
`SPRUCE_SNOW` exposed-crown rule and the landform-not-buried result, both earned against real
captures, hold in whatever wins. Builds on 9.4's density/hue patch; supersedes its colour values
if the bench finds better ones.

**Acceptance Criteria:**

**Given** bench artifacts of at least two tree treatments (tuned-procedural mandatory; authored
optional),
**When** Wolf judges them side by side against the current trees,
**Then** the **procedural-vs-authored decision is made and recorded on that evidence** — his
standing instinct is procedural (2026-08-28); the bench confirms or overturns it cheaply.

**Given** the winning treatment lands in the client,
**Then** the `SPRUCE_SNOW` exposed-crown rule and the landform-not-buried result still hold
(existing tests stay green), 9.4's colour values are superseded or confirmed explicitly, and
Wolf views the valley live (UX-DR22 closing half).

### Story 10.5: Dwarves Worth Looking At

As the boss,
I want my dwarves to be authored models I made,
So that the creatures at the heart of the game carry my hand, not a placeholder cube's.

The PRD's predicted first authored asset, through the now-proven bench: Blender-authored, glTF
via Bevy's native loader, checked against 10.3's contract, hot-reloaded during iteration
(`file_watcher`), landing on the existing appearance-table/reconciliation seam (`EntityKind::Dwarf`,
currently colour `[151,116,96]` at scale 0.65). Enabling `bevy_gltf` + `file_watcher` happens
here with its justification against the feature trim. UX-DR22 both halves; the lantern-carrying
dwarf keeps its table-driven moving light.

**Acceptance Criteria:**

**Given** the Bevy feature trim,
**When** `bevy_gltf` and `file_watcher` are enabled,
**Then** the workspace manifest carries a justification line against the trim's recorded reasons,
and the gate stays green on all devpods.

**Given** a Blender-authored dwarf checked against 10.3's contract,
**When** the client runs against the real daemon,
**Then** every wire dwarf renders as the authored model on the existing reconciliation seam —
position blending, and the lantern-carrier's table-driven moving light, preserved — **observed
live on the vehicle**, not inferred from unit tests (the silent-inert lesson: loading green is
not standing in the world).

**Given** Wolf iterates on the model file,
**Then** the running client hot-reloads it without restart — the art-iteration loop the addendum
promised, demonstrated.

**Given** the milestone eye,
**Then** Wolf signs the dwarves off live (UX-DR22 both halves — bench artifact before, vehicle
after).

**SPLIT LINE, named now:** if this overruns a dev session, "feature enablement + a stand-in glTF
rendering on the seam" splits from "the authored dwarf itself" — the seam story first.

