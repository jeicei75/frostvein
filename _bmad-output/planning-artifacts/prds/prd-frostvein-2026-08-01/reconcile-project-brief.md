# Reconciliation: project-brief.md → PRD + addendum

Input: `/workspace/projects/frostvein/docs/project-brief.md` (SOURCE)
Against: `prd.md` + `addendum.md` in this directory.
Approved extensions (ice/snow terrain, reaction delays, idle wandering, LLM
layer, Asgard wild card, product-PRD framing) were treated as non-gaps per the
reconciliation instructions.

## Covered

Vision, headless-daemon + thin-client architecture, tick loop / pause / fast-forward, determinism, protocol v0 (snapshot + deltas, no compression/interest management), walking-skeleton milestone definition, world grid + terrain, dwarf state machine, FIFO jobs, dig/haul, A* pathfinding, stone item, dev save/load, multiple localhost viewers, TUI glyph/profession-color identity, raycast-view slip permission, headless scenario harness, all four success criteria, 8–12-story cap, roadmap sketch, and the core visual-identity rules (glyph-in-2D, code-authored models, seed-derived identity, no sprite assets ever) are all captured in the PRD/addendum.

## Gaps

### G1 — The explicit non-goals section is largely dropped (HIGH)

Source (lines 52–63): `## Explicit non-goals (silence is not permission — these are out)` followed by eleven exclusions.

The PRD has **no out-of-scope / non-goals section at all**, and the brief's framing — that silence is *not* permission — is exactly the stance the PRD's silence violates. Individually missing from PRD + addendum:

- "No world generation history, civilizations, or off-map anything" — absent.
- "No combat, health, injuries, or body parts" — absent.
- "No fluids, temperature, weather, seasons, or cave-ins" — absent, and **more load-bearing now than in the brief**: the PRD's approved ice/snow terrain (FR1) actively invites melting/freezing/temperature scope creep that this non-goal was guarding against.
- "No farming, crafting chains, or production beyond dig → stone → stockpile" — absent.
- "No mod support, scripting, or data-driven content systems" — absent.
- "No performance optimization before a measured problem exists" — absent as a stated rule (NFR2's "no measurement infrastructure in phase one" gestures at it but does not state the prohibition).

Partially covered elsewhere and *not* counted as gaps: no needs/moods/personalities (FR4), no save-format stability (FR16), no binary protocol/compression/interest management (FR17), no multiplayer beyond localhost viewers (FR19 + NFR1, though the exclusion is implied rather than stated — see G5).

**Severity: high.** These were explicit constraints; FR structures silently dropped them, and the PRD's own future-phases section (needs/moods, more jobs) makes an explicit phase-one exclusion list more necessary, not less.

### G2 — Decided visual-identity details thinned to a one-liner (MED)

Source (lines 77–95, "Visual identity notes … These decisions are made"):

- "~10x5x13 for a dwarf: boots, wide tunic, arms, beard covering the chest, eyes and nose above it, helmet" and "**Wide-and-short silhouette is the read**" — the concrete silhouette/character read, stated as a made decision, is gone. The PRD keeps only "code-authored sub-voxel character models".
- "authored as code (box-fill commands producing a small 3D array)" — the authoring mechanism is dropped.
- "The same model data serves the TUI raycaster (fine-step sampling inside creature-flagged tiles during DDA traversal), distance LODs (down to a single voxel far away), and … mesh generation for the Unreal client" — the one-model-many-renderers decision is dropped.

**Severity: medium.** These are qualitative identity decisions the brief says exist "so agents don't foreclose them"; the PRD's compressed bullet preserves the prohibition (no sprites) but loses the positive decisions a future 3D-view story would need to honor.

### G3 — Decided architecture facts absent: Bevy ECS and the pure-lib sim core (LOW-MED)

Source (lines 13–22, "Core architectural decisions (already made — do not re-litigate)"):

- "**ECS**: Bevy ECS used headless (bevy_ecs crate, not the full Bevy engine)" — nowhere in PRD or addendum.
- "**Sim core**: pure Rust library. No rendering, no terminal, no networking inside it" — only implied (NFR3 names `sim-core`; FR20 says clients hold zero game logic; the LLM sketch says "never inside sim-core"); the positive "pure library, zero I/O" decision is never stated.

**Severity: low-med.** Likely deliberate ("capabilities, not implementation") and duplicated in `docs/technical-preferences.md` / repo CLAUDE.md — but if the PRD+addendum are meant to be self-sufficient planning inputs, these do-not-re-litigate decisions are not captured in them.

### G4 — Project-identity context dropped: solo hobby, AI-agent-built (LOW)

Source (lines 9–11): "This is a solo hobby project developed primarily by AI agents (Claude Code) under the owner's direction, inside a WSL2 devpod. Restraint and shipped increments are valued over completeness and generality."

WSL2 survives (NFR1) and restraint survives (counter-metrics/YAGNI), but the "solo hobby, built by AI agents under the owner's direction" identity — which calibrates every effort/scope judgment downstream — appears nowhere.

**Severity: low.**

### G5 — Multiplayer exclusion implied, not stated (LOW)

Source (line 60): "No multiplayer beyond multiple localhost clients viewing the same sim." FR19 states the positive (multiple localhost viewers) and NFR1 defers multi-machine play, but the exclusion itself ("no multiplayer beyond this") is never written. Folds into G1's missing non-goals section.

**Severity: low.**

## Contradictions

None found. The PRD contradicts no brief statement; all divergences are the approved extensions.

Two observations (not contradictions, recorded for completeness):

1. **Title:** brief says "Voxelheim (working title)"; PRD is "Frostvein". Consistent with the repo name — a deliberate rename, not a conflict.
2. **Unlisted extensions:** the **channel** dig mode (FR2/FR6/FR9) and the `x` remove-designation key (addendum keymap) go beyond the brief's "player marks tiles to dig" and are not on the stated approved-extension list. They read as natural companions to the approved terrain-height extension, but Wolf's approval of them specifically is not on record in the inputs given.
3. **Mild tension, not contradiction:** the brief's "no performance optimization before a measured problem exists" vs NFR2's ~100 ms frame / ~200 ms ack numbers — NFR2 self-limits to "checkable by eye; no measurement infrastructure", which keeps it on the right side of the non-goal, but the non-goal itself should still be restated (see G1).
