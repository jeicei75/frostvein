# PRD Addendum — frostvein

Depth that belongs downstream (architecture, UX, story work), not in the PRD
narrative.

## Keymap sketch — modal DF-style input (FR21, UX detail)

Input detail for the downstream UX/story work; not FR text.

| Key | Context | Action |
| --- | --- | --- |
| `d` | main view | enter dig-designation mode |
| `c` | main view | enter channel-designation mode |
| `p` | main view | enter stockpile-placement mode |
| arrows / `hjkl` | any mode | move cursor |
| `Enter` | in a mode | anchor first corner / commit rectangle |
| `Esc` | in a mode | back out one level |
| `Space` | anywhere | pause / resume |
| `+` / `-` | anywhere | tick rate up / down (pause, 1×, fast) |
| `<` / `>` | anywhere | z-level down / up |
| `x` | main view | remove-designation mode (FR9) |
| `S` | anywhere | save (uppercase, so a slipped finger doesn't trigger it) |
| `L` | anywhere | load (uppercase, likewise) |
| `v` | main view (2D or 3D) | toggle 2D ↔ 3D view |
| `q` | main view | quit (with confirm) |

A one-line hint bar at the bottom always shows the active mode's keys —
DF's discoverability trick, and the only new scope this adds.

(`S`/`L` and `v` were agreed with Wolf during story design, 2026-08-01, and
folded back into this table 2026-08-02 so it stays the one canonical keymap.)

## Mouse / iPad-touch input mechanism (deferred to phase 2)

Wolf asked whether the TUI can work with mouse or touch on an iPad. Answer:
yes, with no protocol or sim changes — input handling is entirely client-side.

- crossterm (the chosen terminal crate) supports mouse-event capture
  (`EnableMouseCapture`, SGR mouse protocol).
- Over SSH from an iPad, terminal apps that support mouse reporting (Blink
  Shell notably) translate taps into terminal mouse escape sequences.
- Pipeline: touch → SSH client app → mouse escape codes → crossterm event →
  same designation code path as the keyboard cursor.
- Decision: keyboard-first for milestone 1; mouse/touch is a phase-2 TUI input
  story confined to the `tui` crate's input layer.

## LLM whimsy sidecar — mechanism sketch (phase 2+, architecture input)

How an LLM layer coexists with load-bearing determinism.

- The LLM never runs inside `sim-core`. It is an async sidecar (or plain
  client) that observes world state via the normal protocol.
- Its outputs enter the sim exclusively as ordinary inputs on the command
  channel — dwarf impulses, quirk events, whatever the future schema allows.
  Because all inputs are logged, replay reproduces the exact run without the
  LLM present; determinism (seed + input log ⇒ identical state) holds.
- Scenario tests never invoke the LLM: they inject scripted stand-in impulses
  through the same channel, or none at all.
- Latency is a non-issue by design: dwarves obey in their own time (FR5's
  seeded reaction delay), so a sidecar that takes seconds to produce an
  impulse fits the fiction instead of fighting the tick budget.
- Read-only narration (chronicler: grim saga text from the delta stream) is
  an even cheaper first form — zero sim impact.

## Asgard adapter — when protocol generalization is allowed (post-frostvein)

Wolf wants the engine to eventually visualize external live systems (the
Asgard realm). Decision: do NOT design for this now. The recorded trigger and
scope for later:

- **Trigger:** the second concrete producer exists (an Asgard adapter that
  emits snapshots/deltas). That is the project's abstraction rule applied to
  the protocol — generalize at the second use, not before.
- **What generalization would touch, when its time comes:** entity/material
  type vocabulary beyond dwarf/stone (likely namespaced type ids), a
  per-producer command vocabulary (a visualized realm may accept different or
  no commands), and possibly schema versioning. None of this is scaffolded in
  phase 1.
- **What phase 1 already guarantees:** FR17's world-not-game principle +
  ADR 4 (clients render data, never rules) + color-as-data mean protocol v0
  messages contain nothing dwarf-specific in *shape* — only in vocabulary,
  which is the cheap part to extend later. (The pre-made ADRs live in
  `docs/technical-preferences.md`.)

## Visual identity details (decided — future 3D views)

Moved from the PRD's Future phases per its own shallowness rule; these are
decisions, not options:

- Sub-voxel character models at finer resolution than terrain; a dwarf is
  ~10×5×13 (boots, wide tunic, arms, beard covering the chest, eyes and nose
  above it, helmet). Wide-and-short silhouette is the read.
- Models are authored as code (box-fill commands producing a small 3D
  array), never image or sprite assets.
- One model serves the TUI raycaster (fine-step sampling inside
  creature-flagged tiles during DDA traversal), distance LODs (down to a
  single voxel far away), and much later, mesh generation for the Unreal
  client.
- Individual identity (beard/hair color) derives from the world seed —
  palette swaps on shared geometry, no per-creature assets.
