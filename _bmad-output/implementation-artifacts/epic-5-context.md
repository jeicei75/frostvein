# Epic 5 Context: The Cold Boot

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Deliver the first Bevy-client vertical slice: a real, orbitable window onto the seeded valley and then a compelling cold-night boot frame. The epic first adds the world content that makes warmth real, establishes one shared client mirror, proves the renderer and its evidence envelope on the development machine, and only then builds the visual result. It matters because this is Milestone 2’s first wow beat and the foundation for every later graphical-client story while retaining the TUI as the dependable command and assertion client.

## Stories

- Story 5.1: The World Grows Things That Glow
- Story 5.2: One Mirror, Two Clients
- Story 5.3: A Window Onto the Valley
- Story 5.4: The Cold Boot

## Requirements & Constraints

The generated world gains deterministic, solid pine-tree tiles and camp torches/campfire as positioned light-emitter entities; these are genuine sim and wire state, visible consistently to both clients. Tree removal uses the existing tile mutation path and produces no wood item. The wire grows only typed vocabulary for the new material, entity-kind, and light-kind values plus an optional light field; it never carries RGB, radius, flicker, or other presentation data.

The GUI is a protocol-only consumer that connects, accepts snapshot then deltas, and can coexist with a TUI client against one daemon. Rendering must use the shared mirror and present real terrain, dwarves, items, and emitters; clients must not invent world state. Atmosphere without sim meaning is allowed later, but must stay client-local. The initial rendering-envelope story may use plain grey boxes: visual polish belongs only to the cold-boot story.

The final boot frame is an isometric, orbitable diorama with a continuous close-to-vista zoom, a camera that remains usable and keeps the fortress in view, and a dark-blue cold world punctuated by real warm camp light. The first visual sign-off is Wolf’s approval of an artifact before visual implementation; completion requires live comparison to that approved artifact. Full 128×128×32-world performance is measured with a readable frame-time overlay, rather than inferred. The cold-boot story’s visual bar includes atmosphere, depth, settled-looking snow, varied ice, and no raw world edge; exact edge treatment remains a story-level decision.

Every story retains determinism, the existing client/sim separation, Rust’s unsafe prohibition, and the closed dependency policy. No asset pipeline, voxel library, golden-image CI, simulated weather, or speculative renderer framework is introduced. Any new dependency needs an explicit story justification; Bevy feature trimming waits for a measured gate-time or binary-size problem.

## Technical Decisions

Clients use mirror-then-project architecture. `client-core` is a protocol-only library that exclusively owns snapshot/delta application, current tick, previous-tick entity state, and per-tick changes. Snapshot reset and full-resend replacement semantics live there; neither client diffs wire messages or retains a second mirror path. Tile state is not double-buffered. The shared inclusive, single-z rectangle normalization helper belongs in `client-core` for later commands.

`gui` depends only on `protocol` and `client-core`, uses `anyhow`, and has `#![forbid(unsafe_code)]`; `client-core` uses `thiserror` and the same unsafe prohibition. The permitted workspace graph additionally includes `tui → client-core`; no client may depend on `sim-core`. Gate probes enforce the absence of `sim-core` edges for `tui`, `client-core`, and `gui`. Full Bevy and `sim-core`’s `bevy_ecs` stay on the same 0.x release line; use Bevy 0.19.0 in this epic. Build meshes procedurally in code and add no render dependency beyond Bevy.

In `gui`, ingestion mutates only the mirror. Reconciliation systems are the sole creators/despawners of world-projected ECS entities, keyed by simulation `Id`; all render entities are either world-projected or client-local. Deleting all world-projected entities and reconciling again must regenerate the same scene. World correctness is covered by mirror tests; reconciliation, camera, and coordinate logic run headlessly under minimal plugins; capture tests need a real render surface and remain separate from default tests and the headless gate.

There is exactly one conversion pair for sim z-up coordinates and Bevy Y-up coordinates: `world_to_render` and `render_to_world`. Projection, later picking, and capture reuse it, and a round-trip test pins it. Appearance is data in GUI tables keyed by kind, never wire data or per-draw hardcoding. The GUI exposes deterministic scripted capture flags and a frame-time overlay; capture output has the overlay disabled and self-tests range-check meaningful, non-black, changed output before claiming success.

## UX & Interaction Patterns

The view is outside and looking down into the valley, with manual orbit and a single zoom continuum rather than view modes. At close working range, blocks and dwarves are legible; at vista range, the valley, sky, and aurora carry the composition. Cold/warm contrast—not a UI marker—must direct attention to the encampment. The gui capture command is the reproducible visual-evidence instrument, while the overlay is a toggleable diagnostic rather than capture content.

## Cross-Story Dependencies

Story 5.1 supplies trees and lights that 5.3/5.4 render. Story 5.2 must establish `client-core` and retire the TUI’s duplicate state before `gui` consumes the mirror in 5.3. Story 5.3 proves the window, lifecycle, projection, camera, and evidence instruments before 5.4 invests in visual quality. Epic 6 relies on this projection base for real movement, work, and flicker; Epic 7 adds slicing; Epic 8 adds graphical input. Splitting 5.2 or 5.3 requires an explicit re-check of the milestone’s first-third wow timing.
