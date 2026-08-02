---
stepsCompleted: [1, 2, 3, 4, 5, 6]
documentsIncluded:
  prd: _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/prd.md
  prdAddendum: _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/addendum.md
  architecture: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md
  architectureProjection: docs/architecture.md
  epics: _bmad-output/planning-artifacts/epics.md
  ux: null
---

# Implementation Readiness Assessment Report

**Date:** 2026-08-02
**Project:** frostvein

## Document Inventory

| Type | File | Status |
|---|---|---|
| PRD | `prds/prd-frostvein-2026-08-01/prd.md` (+ `addendum.md`) | Found — single version |
| Architecture | `architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` | Found — primary (binding contract) |
| Architecture (projection) | `docs/architecture.md` | Found — supporting "ten-minute read"; spine wins on conflict |
| Epics & Stories | `epics.md` | Found — single whole document |
| UX Design | — | **Not found** (TUI project; may be intentional) |

**Duplicates:** none requiring resolution — the architecture two-tier arrangement is self-declaring (projection defers to spine).

**Warnings:** No UX design document; assessment will flag UI-facing requirements lacking design coverage.

## PRD Analysis

Source: `prds/prd-frostvein-2026-08-01/prd.md` (status: final, 2026-08-01) + `addendum.md`.
PRD gives FR-level depth to **phase one (Milestone 1) only**, by design.

### Functional Requirements

**F1. World**
- FR1: The world is a fixed-size 3D voxel grid (default 128×128×32) with simple layered terrain — stone, soil, ice, snow, air — generated from a world seed. Icy materials are not decoration: snow surfaces and ice appear in ordinary generated terrain so the world reads as frozen from first boot. No chunking or streaming.
- FR2: Terrain generation produces surface height variation with walkable ramps/slopes, so vertical traversal exists in a naturally generated map. [ASSUMPTION] modest rolling height (a few z-levels) — enough to exercise climb pathfinding and channel digging.

**F2. Dwarves & jobs**
- FR3: A handful of dwarves spawn on the surface at world generation. [ASSUMPTION] 5 dwarves.
- FR4: Each dwarf runs a simple job state machine: idle → walk → work; current state visible to clients. No needs, moods, or personalities. Idle dwarves wander nearby tiles (seeded, deterministic) so the world visibly lives with no orders. [ASSUMPTION] wander within ~3 tiles.
- FR5: Idle dwarves claim the oldest unclaimed job (FIFO). One dwarf per job; a claimed job is released if the dwarf cannot complete it. A seeded per-dwarf reaction delay passes before a job is claimed (orders are directives, not remote control). [ASSUMPTION] delay ~0.5–3 s (5–30 ticks), seeded per dwarf per job.
- FR6: Dig job: a dwarf adjacent to a designated tile removes it (dig: wall → open floor; channel: floor dug out leaving a ramp below) and a stone item appears at the dug location.
- FR7: Haul job: a dwarf carries a loose stone item to a stockpile tile and drops it there.
- FR8: An unreachable designation stays queued and is retried; never silently dropped. // NOTE: naive retry acceptable in phase one.

**F3. Player intents**
- FR9: Player designates tiles for digging as rectangles in two modes: dig (same-level) and channel (dig down, leaving a ramp). A designation can be cancelled before it is dug, releasing any unclaimed or in-progress job on it.
- FR10: Player can place a stockpile zone as a rectangle on walkable floor.

**F4. Pathfinding**
- FR11: Dwarves pathfind with plain A* on the voxel grid: walking on floors, climbing ramps/stairs between z-levels. No hierarchical pathfinding, no caching.

**F5. Items**
- FR12: Stone exists as a haulable item with a world position. No materials system, quality, stacking, or containers.

**F6. Daemon & tick loop**
- FR13: The daemon runs a fixed-timestep tick loop (default 10 ticks/sec), fully decoupled from clients; the sim advances with zero clients attached.
- FR14: Speed control: pause, normal (1×), fast-forward as tick-rate changes. [ASSUMPTION] one fast step (≈5×) is enough.
- FR15: Determinism: identical world seed + identical command sequence produces identical sim state, tick for tick. Load-bearing for the scenario harness (F9).
- FR16: Dev save/load of full sim state, plus clean quit. No save-format stability guarantees.

**F7. Protocol v0**
- FR17: Newline-delimited JSON over localhost TCP. On connect a client receives a full world snapshot; thereafter per-tick delta messages. Chattiness acceptable; no batching, compression, or interest management. Design principle: messages describe a world, not a dwarf game — state and typed data, never game rules or narrative interpretation.
- FR18: Commands upstream: designate dig/channel, cancel designation, place stockpile, pause/resume, set tick rate, save, load, quit.
- FR19: Multiple localhost clients can view the same running sim concurrently.

**F8. TUI client**
- FR20: Single z-level top-down view, navigable between z-levels DF-style (`<`/`>`). Client contains zero game logic.
- FR21: Modal, DF-familiar keyboard input: single keys enter a mode (dig, channel, stockpile), rectangles placed cursor-first with Enter-anchor/Enter-commit, `Esc` backs out, one-line hint bar always shows active mode's keys. Concrete keymap in the addendum. Mouse/touch is phase two.
- FR22: Dwarves render as `☺` glyphs colored by current job/profession (miner amber, hauler teal); terrain and items render as distinct glyphs. 24-bit truecolor from the start; color is data (material/profession → RGB), not a fixed palette.
- FR23: Visual identity is icy, gloomy, dark and grim: cold desaturated terrain palette with profession colors as warm accents. Acceptance instrument is Wolf's eye: success criterion 2 includes sign-off on the icy-grim look in the live TUI. [ASSUMPTION] palette/glyph selection inside existing rendering stories, not a separate art story.
- FR24: The raycast 3D view is its own story late in the milestone and may slip to phase two without ceremony.

**F9. Headless scenario harness**
- FR25: Scenario tests build a world from a seed, inject commands, tick N times, and assert sim state — no client or network attached.
- FR26: The walking-skeleton sentence exists as an automated scenario test (dig designation → pathfind → dig → haul to stockpile) and is the phase-one gate.

Total FRs: 26

### Non-Functional Requirements

- NFR1 — Platform: Phase one targets the WSL2 devpod and any decent terminal emulator over SSH. No other platforms. Long term this is a true server+client game across machines; nothing in phase one may preclude that, and nothing in phase one builds for it.
- NFR2 — Feels alive: TUI keeps pace at 10 ticks/sec with no visible stutter (~100 ms frame budget, full 128×128 z-level). Player commands acknowledged in UI within ~200 ms (one tick + one frame). Dwarf obedience explicitly exempt (FR5). Even with zero commands, the view visibly moves (FR4). Checkable by eye; no measurement infrastructure.
- NFR3 — Determinism everywhere: FR15 is cross-cutting; any nondeterminism (unordered iteration, wall-clock time, unseeded randomness) in `sim-core` is a bug.
- NFR4 — Quality gate: every story lands with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` green.

Total NFRs: 4

### Additional Requirements & Constraints

- **Out of scope (explicit, "silence is not permission")**: no graphical client, no worldgen history/civs, no combat/health, no fluids/temperature/weather/seasons/cave-ins (ice/snow are materials, not processes), no needs/moods/social, no farming/crafting beyond dig→stone→stockpile, no save-format stability, no multiplayer beyond localhost viewers, no perf optimization without a measured problem, no mod/scripting/data-driven systems, no binary protocol.
- **Success criteria**: (1) walking-skeleton scenario test passes headless (FR26); (2) same scenario watchable live in TUI, meets feel floor (NFR2), Wolf signs off on icy-grim look (FR23); (3) quality gate green; (4) planning docs stay re-readable in one sitting.
- **Counter-metrics**: phase one ships in 8–12 vertically sliced stories — materially more means scope gets cut (cut list starts FR24, then FR16); no code serving only a future phase; story rules from `docs/technical-preferences.md` apply unchanged.
- **Assumptions index** (for architecture confirm-or-override): FR2 modest hills; FR3 = 5 dwarves; FR4 wander ~3 tiles; FR5 delay 5–30 ticks; FR14 single ≈5× fast step; FR23 palette work inside rendering stories.
- **Addendum contents**: concrete DF-style keymap for FR21; mouse/iPad-touch input mechanism (deferred phase 2, client-side only); LLM whimsy sidecar mechanism sketch (phase 2+, never inside sim-core); Asgard adapter generalization trigger (second concrete producer, post-frostvein); sub-voxel character model decisions (future 3D views).

### PRD Completeness Assessment

The PRD is unusually well-formed for traceability: FR IDs are stable and globally numbered, NFRs are explicit, assumptions are tagged inline AND collected in an index, out-of-scope is enumerated ("silence is not permission"), and success criteria + counter-metrics are testable. Deliberate shallowness of future phases is a documented policy, not a gap. The one structural note: FR23's acceptance rests on subjective sign-off (Wolf's eye) — acceptable and explicitly acknowledged in the document. No UX document exists; FR20–FR24 plus the addendum keymap carry the UX burden for phase one.

## Epic Coverage Validation

Source: `epics.md` (4 epics, 11 stories: Epic 1 ×3, Epic 2 ×4, Epic 3 ×3, Epic 4 ×1).
The epics document contains its own Requirements Inventory (FR1–FR26, NFR1–NFR4) and an explicit FR Coverage Map. Each claim below was verified against actual story acceptance criteria, not just the map.

### Coverage Matrix

| FR | Requirement (short) | Epic/Story coverage | Status |
| --- | --- | --- | --- |
| FR1 | Fixed-size seeded voxel world, icy layered terrain | Story 1.1 AC2 | ✓ Covered |
| FR2 | Surface height variation, walkable ramps | Story 1.1 AC2 | ✓ Covered |
| FR3 | 5 dwarves spawn at worldgen | Story 1.1 AC2 | ✓ Covered |
| FR4 | Job state machine + seeded idle wandering | Story 2.2 (state visible on wire); live color change in 3.2 | ✓ Covered |
| FR5 | FIFO claiming + seeded reaction delay | Story 3.2 AC2 | ✓ Covered |
| FR6 | Dig job (dig/channel execution, stone appears) | Story 3.2 AC3 | ✓ Covered |
| FR7 | Haul job to stockpile | Story 3.3 AC1 | ✓ Covered |
| FR8 | Unreachable designation queued + retried | Story 3.2 AC4 | ✓ Covered |
| FR9 | Rect dig/channel designation + cancel | Story 3.1 (designate/cancel UI+commands); 3.2 (job release on cancel) | ✓ Covered |
| FR10 | Stockpile zone on walkable floor | Story 3.1 AC2 | ✓ Covered |
| FR11 | Plain A* across floors/ramps/z | Story 3.2 AC3 | ✓ Covered |
| FR12 | Stone as haulable item with position | Story 3.2 AC3 | ✓ Covered |
| FR13 | Fixed-timestep loop, client-decoupled | Story 2.1 AC1 | ✓ Covered |
| FR14 | Pause / 1× / fast-forward | Story 2.3 | ✓ Covered |
| FR15 | Determinism seed+commands ⇒ identical state | Story 2.2 AC3 (established); re-asserted 3.3 AC2 | ✓ Covered |
| FR16 | Dev save/load + clean quit | Story 2.4 | ✓ Covered |
| FR17 | Protocol v0: NDJSON, snapshot, deltas | Story 1.2 (snapshot); Story 2.1 (deltas) | ✓ Covered |
| FR18 | Command set upstream | 2.3 (speed), 2.4 (save/load/quit), 3.1 (designate/cancel/stockpile) | ✓ Covered |
| FR19 | Multiple localhost viewers | Story 2.3 AC3 | ✓ Covered |
| FR20 | Z-level top-down view + z-nav | Story 1.3 AC1–2 | ✓ Covered |
| FR21 | Modal DF keyboard input + hint bar | Story 3.1 AC1 | ✓ Covered |
| FR22 | Glyph rendering, truecolor, color-as-data | Story 1.3 AC1, AC3 | ✓ Covered |
| FR23 | Icy-grim identity, Wolf sign-off | Story 1.3 (static), 3.3 (in motion), 4.1 (3D) | ✓ Covered |
| FR24 | Raycast 3D view | Story 4.1 | ✓ Covered — see divergence note |
| FR25 | Headless scenario harness | Story 2.2 AC3 (foundation); exercised in 3.1/3.2/3.3 | ✓ Covered |
| FR26 | Walking-skeleton scenario test (phase gate) | Story 3.3 AC2 | ✓ Covered |

### Missing Requirements

None. All 26 PRD FRs trace to at least one story with acceptance criteria; no epic FRs exist that are absent from the PRD.

### Divergences (epics vs. PRD — documented, needs PRD sync)

1. **FR24 scope change.** PRD (status: final): raycast view "may slip to phase two without ceremony" and heads the cut list. Epics: "Required for phase one — Wolf's override (2026-08-01)... off the cut list", with the cut list now starting at FR16. The override is attributed and dated in the epics doc, but the PRD was not updated — the two final documents now disagree on phase-one scope and cut-list order.
2. **Keymap additions beyond the addendum.** `S`/`L` (save/load, Story 2.4 preamble) and `v` (2D↔3D toggle, Epic 4 preamble) were agreed with Wolf during story design and documented in the epics file, but do not appear in the addendum keymap table.

### Coverage Statistics

- Total PRD FRs: 26
- FRs covered in epics/stories: 26
- Coverage: **100%**
- Story count: 11 — within the 8–12 counter-metric.

## UX Alignment Assessment

### UX Document Status

**Not found** — and this is a documented, deliberate arrangement, not an oversight. The epics file states it explicitly: "No separate UX design contract exists... The TUI's visual and interaction requirements are first-class PRD requirements (FR20–FR23) plus the concrete keymap in the PRD addendum." UX is clearly implied (user-facing TUI client), so the check proceeds against the PRD-as-UX-spec.

### UX ↔ PRD Alignment

- Interaction model: fully specified — modal DF-familiar input (FR21) with a concrete keymap table in the addendum, hint-bar discoverability, Enter-anchor/Enter-commit rectangles, Esc back-out.
- Visual design: specified at requirement level — glyph vocabulary and truecolor (FR22), icy-grim palette direction with named accents (FR23), z-level navigation (FR20).
- Acceptance: subjective sign-off (Wolf's eye) is the named instrument for FR23 — deliberate, testable in a live session, and staged sensibly across stories (static look in 1.3, motion in 3.3, 3D in 4.1).

### UX ↔ Architecture Alignment

The spine directly supports every UX-bearing requirement:

- Responsiveness (NFR2 ~200 ms ack): the "no explicit ack" convention — a command's effect in the next delta is the ack — plus AD-2's always-running loop (deltas keep flowing while paused, so designations placed while paused are still acknowledged).
- Frame budget (~100 ms): TUI drawing convention — hand-rolled cell framebuffer flushed once per frame, never per-cell writes.
- Color/identity (FR22/FR23): wire carries ids, never RGB; the id → RGB table is one data table in `tui` shared by 2D and raycast views.
- Input (FR21): entirely client-side in `tui`; mouse/touch deferral is recorded with a concrete phase-2 mechanism (crossterm mouse capture) in the addendum.

### Alignment Issues

1. **Keymap drift (minor):** `S` (save), `L` (load), and `v` (2D↔3D toggle) were agreed with Wolf during story design and are documented in epics preambles, but the addendum keymap table — the declared canonical keymap ("concrete keymap in the addendum", FR21) — was not updated. Two sources now describe the keymap, one incomplete.
2. **FR24 in the spine's Deferred list:** the spine still records the raycast view as "may slip to phase two without ceremony (FR24)" — pre-dating Wolf's 2026-08-01 override that made it firm phase-one scope. Same divergence family as the PRD's FR24 text (already logged in Epic Coverage Validation).

### Warnings

- No standalone UX document: acceptable for a solo-developer TUI with this level of FR/keymap specification and the "docs re-readable in one sitting" counter-metric. No blocking gap. The absence is mitigated, not ignored — the epics restate the full keymap under "UX Design Requirements."

## Epic Quality Review

Standards applied: create-epics-and-stories best practices, plus the project's own story rules from `docs/technical-preferences.md` (vertical slices only; every story ends in something observable; one dev-agent session per story; 8–12 stories for milestone 1), which the PRD binds "unchanged."

### Epic Structure

| Epic | User value | Independent? | Verdict |
| --- | --- | --- | --- |
| 1 The Frozen World on Screen | See and judge the generated icy world in the terminal | Stands alone | ✓ |
| 2 The World Breathes | Watch a living sim; control time; keep a session | Needs only Epic 1 | ✓ |
| 3 The Boss Gives Orders | Issue orders, watch them fulfilled — the game loop | Needs only Epics 1–2 | ✓ |
| 4 The World in Three Dimensions | See the fortress in depth | Needs only Epics 1–3 | ✓ |

No technical-milestone epics: all four are outcome-titled and player-voiced ("As the boss..."). No epic requires a later epic; no circular dependencies. Sequencing is deliberately value-first — the icy-grim look (the project's identity bet) gets feedback in Epic 1, the phase gate (FR26) caps Epic 3.

### Dependency Analysis

Within-epic ordering is clean: 1.1→1.2→1.3, 2.1→(2.2, 2.3, 2.4), 3.1→3.2→3.3 — each story uses only earlier outputs. Two **forward-reference notes** exist and both are documented trade-offs, not blocking dependencies:

- Story 2.4: SaveState's "jobs + claims" sections "join in Story 3.2 when the job market exists" — 2.4 is completable without 3.2; the note records that every sim-state-adding story extends SaveState.
- Story 3.1: designation removal ships, but "job release lands in Story 3.2 when jobs exist" — 3.1 is completable alone (no jobs exist yet to release).

Entity/state creation timing follows the create-when-needed rule (jobs appear in 3.2, not scaffolded in 1.1; SaveState grows incrementally). Architecture specifies **no starter template** (greenfield) and requires Epic 1 Story 1 to scaffold the workspace — Story 1.1 does exactly that, with the quality gate enforced from the first commit.

### Acceptance Criteria Quality

Consistent Given/When/Then. Strong points: FR/AD ids cited inline in nearly every AC (traceability is enforced at the AC level, not just the coverage map); error/negative paths present (malformed input in 1.2, unreachable-retry and cancel-mid-dig in 3.2, clean shutdown in 2.4); determinism asserted as tests, not intent (1.1 twin-worldgen, 2.2 twin-run, 2.4 save/load-vs-never-saved, 3.3 replay).

### Findings

#### 🔴 Critical Violations

None.

#### 🟠 Major Issues

1. **Story 3.2 sizing risk ("The Dig").** One story carries: designation→job conversion, the whole job market + single claiming system (AD-12), seeded reaction delay, plain A* across z-levels, dig/channel execution with item spawn, unreachable-retry, cancel-mid-dig release, and headless scenarios for all of it. A* alone is a session-sized chunk. This strains the project's own "fits one dev-agent session" rule — the only story in the set that clearly does. *Remediation:* pre-split vertically (e.g., 3.2a "claim and walk": job market + delay + A* + walk-to-site asserted headless; 3.2b "the dig lands": dig/channel execution + retry + cancel release). 12 stories total — still within the 8–12 counter-metric.
2. **Story 4.1 sizing risk ("Behold the Fortress in Depth").** DDA raycaster + camera movement + sub-voxel character models with LOD + seed-derived identity in one story is ambitious for one session. Unlike 3.2 it has no hard gate behind it. *Remediation options:* accept as-is with the model/LOD ACs marked as the flex zone, or split — but a split lands at 13 stories, breaching the counter-metric, whose cut list (post-override) starts at FR16. Flag for Wolf rather than silently splitting.

#### 🟡 Minor Concerns

3. **Story 1.1 is developer-voiced** ("As a developer..."). Under generic standards a setup story is a red flag; here it is sanctioned by the project's own rules — it ends in something observable (passing worldgen/determinism integration tests), and the architecture mandates Epic 1 Story 1 scaffold the workspace. Compliant locally; noted for the record.
4. **Stockpile placement edge unspecified.** Story 3.1: "stockpiles are only accepted on walkable floor" — behavior for a rectangle partially on walkable floor (clip? reject whole rect?) is undefined. One sentence in the story fixes it.
5. **Keymap drift** (`S`/`L`/`v` absent from the addendum's canonical keymap table) — logged in UX Alignment; belongs on the fix list.
6. **FR24 doc sync** (PRD + spine still carry the may-slip clause overridden 2026-08-01) — logged in Epic Coverage Validation; belongs on the fix list.

### Best Practices Compliance Summary

- [x] Epics deliver user value
- [x] Epics function independently, no forward epic dependencies
- [x] No blocking forward story dependencies (2 documented forward notes)
- [x] State created when needed, no upfront scaffolding beyond the mandated workspace story
- [x] Clear, testable, FR-traced acceptance criteria
- [ ] Story sizing: 2 of 11 stories at risk of exceeding one dev-agent session (3.2, 4.1)

## Summary and Recommendations

### Overall Readiness Status

**READY** — with a short pre-flight fix list. No critical violations. FR coverage is 100% (26/26) with AC-level traceability, epics are value-sequenced and independent, the architecture spine directly supports every UX-bearing and cross-cutting requirement, and determinism — the load-bearing property — is asserted by tests at four separate points in the story flow. The issues found are sizing risks and document-sync drift, not planning gaps.

### Critical Issues Requiring Immediate Action

None. The two 🟠 items below are the closest to blocking and are decisions, not rework:

1. **Story 3.2 is oversized** for the "one dev-agent session" rule (job market + claiming + reaction delay + A* + dig execution + retry + cancel-release + scenarios). Recommend a vertical pre-split into 3.2a (claim & walk) / 3.2b (dig lands) — total becomes 12 stories, still within the counter-metric.
2. **Story 4.1 sizing** (raycaster + camera + sub-voxel models + LOD). A split would breach the 12-story cap, so this is Wolf's call: accept as-is with model/LOD ACs as the flex zone, or split and invoke the cut list (which now starts at FR16).

### Recommended Next Steps

1. Decide the Story 3.2 split (recommended: yes) and the Story 4.1 question — a 5-minute `correct-course`-scale edit to `epics.md`.
2. Sync the FR24 override into the two stale documents: PRD ("may slip" clause + cut-list order) and spine (Deferred list entry) — both still contradict the epics' firm-scope wording despite `status: final`.
3. Add `S`, `L`, `v` to the addendum keymap table so the declared canonical keymap (FR21) is actually complete.
4. Add one sentence to Story 3.1 defining partially-walkable stockpile rectangles (clip vs. reject).
5. Then proceed: sprint planning → story creation → dev, starting with Story 1.1.

### Final Note

This assessment identified **6 issues across 3 categories** (2 major sizing risks, 4 minor documentation/specification gaps) — and zero coverage or dependency defects. Address items 1–2 before implementation; items 3–4 can ride along with their stories. These findings can be used to improve the artifacts, or you may choose to proceed as-is.

---
*Assessed 2026-08-02 by the BMad implementation-readiness workflow (facilitated by Claude for Wolf).*

## Decision Log (2026-08-02, post-assessment)

- **Stories kept as-is (Wolf's call):** no split of Story 3.2 or 4.1 — the sizing risks (major issues 1–2) are accepted; the story set stays at 11.
- **Doc-sync items fixed:** PRD FR24 text, cut-list order, and phase-2 candidates list updated to Wolf's 2026-08-01 override; spine Deferred entry likewise; addendum keymap now includes `S`/`L`/`v`; Story 3.1 defines partially-walkable stockpile rects as **clip to walkable tiles** (zero-walkable rect yields no zone).
