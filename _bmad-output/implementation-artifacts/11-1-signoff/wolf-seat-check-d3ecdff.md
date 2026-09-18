# Wolf's seat check — gingerspice, build `d3ecdff`, 2026-09-18

The branch was pushed so Wolf could build and look at it. He did, on the correct build.

**Build confirmed: `d3ecdff`** — the tip of `story-11-1a-exposure-and-clean-edge`, which carries
`Msaa::Off`, `Exposure { ev100: 9.7 }` and `Fxaa::default()`. Confirmed by Wolf before this was
recorded, because a seat verdict against a stale `gui.exe` closes nothing: the walk-cycle defect
cost most of a session to a client three commits behind, and issue #46 (automate the
rebuild-and-copy) is still open.

## What he judged

> "edges are fine"

> "ridge is better than fading terrain atm"

> "I checked client already in gingerspice.. it's fine"

## What that CLOSES

**AC11's first clause — the edges.** The FXAA frame reads acceptably at the boot framing on the
delivery GPU. **The standing caveat that FXAA might read blurrier than MSAA 4x is withdrawn**; it
was a real risk (FXAA is a spatial post-filter, MSAA 4x supersamples the edge) and the seat answered
it. No further edge evidence is owed.

## What it does NOT close — do not read this as a full AC11 sign-off

- **AC10 — `--perf-log` p50 against NFR6's 60 fps bar**, with all effects on and each off. Not
  measured. No fps figure exists for this build on the vehicle, and none has been fabricated.
- **AC11's second clause — "the exposure is the one he wants to judge 11.1b's effects under."**
  Not addressed. Note this is really a question about the CURRENT brightness, not about the change:
  EV100 9.7 is Bevy's own fallback when the component is absent
  (`bevy_render-0.19.0/src/camera.rs:639`), so 11.1a changed zero pixels of exposure. Moving it off
  9.7 is a look decision and Wolf's, and #75 says every look constant predating 10.7 was tuned with
  the sun under the map.

Task 5 therefore stays UNCHECKED. `task-5-vehicle-card.md` still holds the two `--perf-log` runs
that are owed.

## A ruling recorded for Epic 11's later stories

> "when extending world then we can think about how to solve it"

The terrain fading at the world edge is **deferred to the world-extension work**, not to a look
story. It is not a defect to fix under Epic 11.

> "volumetric fog was in the plans at least"

**Volumetric fog stays in plan for 11.2.** "ridge is better than fading terrain" was a preference
about the world EDGE, not an objection to aerial haze — checked with Wolf rather than inferred,
because the opposite reading would have contradicted 11.2's whole premise ("the far valley
softening into air") before it was written.
