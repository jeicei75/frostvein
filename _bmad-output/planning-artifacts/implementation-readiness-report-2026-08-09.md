---
stepsCompleted: [1, 2, 3, 4, 5, 6]
documentsUnderAssessment:
  prd:
    - _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/prd.md
    - _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/addendum.md
  architecture:
    - _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md
    - _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md  # inherited parent, AD-1..12
  epics:
    - _bmad-output/planning-artifacts/epics.md  # Epics 5-8 (Milestone 2)
  ux:
    - docs/narrative.md            # accepted as UX input per Wolf, 2026-08-09
    - docs/17d7215b-6c05-4286-b3bb-56592ca617ec.jpg
    - docs/a9d4e72b-b4c3-43f2-8a1c-e25c539fd6c1.jpg
  supporting:
    - docs/project-brief.md
    - docs/technical-preferences.md
    - _bmad-output/planning-artifacts/sprint-change-proposal-2026-08-08.md
    - _bmad-output/implementation-artifacts/deferred-work.md
---

# Implementation Readiness Assessment Report

**Date:** 2026-08-09
**Project:** frostvein
**Scope:** Milestone 2 — the Bevy client (Epics 5–8)

## Step 1 — Document Inventory

### PRD

- **Under assessment:** `prds/prd-frostvein-2026-08-09/` — `prd.md` (16k), `addendum.md` (3.2k), plus reconcile passes against the narrative and both reference images, and a review rubric.
- **Historical:** `prds/prd-frostvein-2026-08-01/` — Milestone 1. Retained as parent context, not assessed.

### Architecture

- **Under assessment:** `architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md` (16k, AD-13…18) with `reviews/` (reconcile-prd, reconcile-parent, review-adversary, review-versions, review-rubric).
- **Inherited:** `architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` (AD-1…12), amended 2026-08-09.
- **Projection, not a source:** `docs/architecture.md` — the ten-minute read; self-declares subordinate to the spines.

### Epics & Stories

- `epics.md` (96k) — one whole document carrying both milestones. M1 = Epics 1–4 (closed), **M2 = Epics 5–8, 11 stories** — the assessment target. No sharded variant.

### UX

- **No UX design document exists.** Per Wolf's decision (2026-08-09), `docs/narrative.md` and the two reference images are accepted as the UX input, and epics will be traced against *them*. This matches the repo's documentation-restraint policy.

### Issues Recorded

| Severity | Issue | Resolution |
|---|---|---|
| — | 08-01 / 08-09 doc pairs | **Not duplicates.** Milestone generations; M2 is additive over M1. No action. |
| WARNING | No UX document for a visual milestone | Accepted narrative + reference images as UX input (Wolf, option a). |
| INFO | No `project-context.md` in this repo | Used `docs/technical-preferences.md` in its place, per repo CLAUDE.md. |
| INFO | M2 plan is uncommitted (`epics.md`, `sprint-status.yaml` modified) | Assessment reads the working tree. |

## Step 2 — PRD Analysis

Source: `prds/prd-frostvein-2026-08-09/prd.md` (status: final) + `addendum.md`, read
in full. The M2 PRD **inherits the M1 PRD by reference** (FR1–FR26, NFR1–NFR4), so
M2 numbering continues the global sequence rather than restarting.

### Functional Requirements

**F10. World content that glows and grows** *(the one place M2 touches the sim — all seeded, deterministic, world state)*

- **FR27** — Worldgen grows pine trees on the surface: seeded, deterministic, part of world state, visible to every client (the TUI shows them as glyphs). `[ASSUMPTION]` density and placement are worldgen tuning decisions inside the story, not FR text.
- **FR28** — Worldgen places static warm light emitters: torches and a campfire at the dwarven starting camp. Light emitters are world state with a position; what light *looks* like is each client's concern.
- **FR29** — Dwarves carry lanterns: a light source attached to a moving entity. Deliberately the lighting system's hardest case — a moving warm light — placed in scope as a testbed. `[ASSUMPTION]` every dwarf simply carries one; no fuel, no pickup/drop, no economy.
- **FR30** — Protocol v0 vocabulary grows to carry the above (tree and light-emitter materials/entities, carried-light state) as typed world data, honouring FR17's world-not-game principle. No shape changes, only vocabulary.

**F11. The diorama (Bevy client — the view)**

- **FR31** — The world renders as the isometric orbitable diorama the Visual Target describes: one zoom continuum from working-close to valley-vista, camera always usable, never lost.
- **FR32** — The cold/warm read is live: world light sources render as warm pools against the cold night palette; sky, stars, and aurora carry the far register; snow falls as pure decoration (no sim weather).
- **FR33** — The player can slice into the mountain by z-level to see and work the underground, and can always tell which z-level they are on and what is underground vs. surface. Mechanism per the addendum's open question — chosen by testing in its story, not here.
- **FR34** — The world visibly lives, driven only by real sim state over the wire: dwarves move and work at the dig face, carried lanterns move with them, static lights flicker, idle dwarves wander. Zero commands issued still means visible motion (M1's FR4 aliveness, now in 3D).

**F12. Working the fortress (input parity)**

- **FR35** — The Bevy client reaches full TUI command parity: designate dig/channel, cancel designation, place/remove stockpile, pause/resume, tick rate, save/load, quit. Clients contain zero game logic, unchanged.
- **FR36** — The player can select tiles and rectangles in the 3D view with the mouse — the picking problem — including on sliced underground z-levels. Acknowledged as M2's hardest input work and the main story-count driver.

**F13. Client lifecycle (the boring glue)**

- **FR37** — The Bevy client is a `protocol`-only consumer: connects, receives snapshot, applies per-tick deltas, coexists with concurrent TUI clients on the same daemon (M1's FR19). `sim-core` and `simd` need no structural change for it.

**Total M2 FRs: 11 (FR27–FR37).** Inherited and still binding: FR1–FR26.

### Non-Functional Requirements

- **NFR5 — No drift.** Clients never invent world state; everything visible in any client is derivable from the wire (AD-1/AD-4 restated). One deliberate carve-out: pure atmosphere — sky, aurora, snowfall — is client-side by design and must never acquire sim meaning silently.
- **NFR6 — Feels alive, Bevy bar.** The client sustains the diorama at interactive framerates on the dev machine with the full 128×128×32 world, all dwarves, and all lights, with command acknowledgement comparable to the TUI's (~200 ms). `[ASSUMPTION]` a measured fps number is set **at architecture time as a blocking item**; NFR2 explicitly does not stretch to this client.
- **NFR7 — Determinism unchanged.** FR27–FR29 land inside worldgen and sim state, so seed + command log ⇒ identical state must survive them; scenario tests cover trees and light emitters like any other world state.
- **NFR8 — Gate grows a sibling probe.** `scripts/gate.sh` gains the bevy-client twin of the `tui` probe (no `sim-core` edge) — the AD-1 edge stays guarded for the client that matters most.

**Total M2 NFRs: 4 (NFR5–NFR8).** Inherited: NFR1–NFR4.

### Additional Requirements (unnumbered but binding)

The PRD carries a large body of requirement-grade text outside FR/NFR numbering. For
traceability I have enumerated it, since **unnumbered requirements are exactly what
epics silently drop.**

**Visual Target bars — "What the references bind (bars, not guidance)" — explicitly promoted from guidance to bars by the PRD:**

- **VT1** — The sky is an illuminant, not a backdrop: aurora and starlight visibly light the snow and catch on ice; the aurora hugs the horizon rather than hanging overhead.
- **VT2** — Snow reads as a settled cap: white tops, bare dark flanks, loaded branches — not a uniform coat.
- **VT3** — Work leaves evidence: rubble/debris at the dig face (sim stone items + cosmetic chips under NFR5's carve-out); a worked site never looks spotless.
- **VT4** — Value discipline: night snow stays midtone blue-grey; only emissive light approaches white.
- **VT5** — The cold field varies: blue ice breaks the white expanse; the vista reads in cold-against-cold layers, not one white sheet.
- **VT6** — The world reads as a miniature whose edges dissolve into the night — a raw grid edge is never visible at any zoom. (Treatment is a design-and-test question, addendum.)

**Wow beats (both required):**

- **WB1** — Cold boot: the first frame is an aesthetic hit on looks alone (voxel world, dramatic lighting, aurora).
- **WB2** — ~Thirty seconds in: the realisation that it's alive — light flickers, work animates at the dig face, a dwarf picks something up and carries it. **"A client that only achieves beat 1 has failed the milestone."**

**Anti-requirements (4.1a's six failures, inverted — each a bar):**

- **AR1 Ugly** → the boot frame is something you'd screenshot unprompted.
- **AR2 Flat** → depth reads instantly; light, shadow, and air separate near from far.
- **AR3 Cluttered** → at working zoom you can tell dwarves, terrain, designations, and items apart at a glance.
- **AR4 Confusing** → you always know what you're looking at, which z-level you're on, and what is underground vs. surface.
- **AR5 Lifeless** → something visibly moves even when you issue nothing.
- **AR6 Camera unusable** → you can always reach the angle you want and never lose the fortress.

**Process requirements:**

- **PR1 — Sign-off gate (opening half).** No visually subjective story is implemented before Wolf approves a cheap "here is what you will see" artifact for it, showing *our actual world* at the framing being built. One artifact per visual story.
- **PR2 — Sign-off gate (closing half).** The story is done only when Wolf has viewed the built result **live** and compared it against the approved artifact. "4.1a was lost at live viewing, not at spec time."
- **PR3 — Parity rule.** Bevy catches up to the TUI; the TUI is not extended for Bevy-only work, **but** any new sim functionality or bug fix affecting the TUI updates the TUI too. No TUI regression ships during M2.
- **PR4 — Tech-art guidelines deliverable.** Named twice as the home for visual depth (values, material rules) and as the asset contract when the pipeline opens.
- **PR5 — Art is procedural/code-first.** No asset pipeline in the base build; authored assets enter only when a concrete case forces the decision **on the record** (dwarves the expected first case). This overturns M1's "never as assets, ever" on the record.

**Open design questions deferred to story level (addendum):**

- **OQ1** — Z-slice control mechanism; known collision with mousewheel zoom. Chosen by testing in its story.
- **OQ2** — World-edge treatment at vista zoom (fog skirt / darkness falloff / sky wrap / vignette). Chosen by testing in the camera/atmosphere story.
- **OQ3** — Should in-grid terrain give the vista a mountain silhouette within 128×128×32? M1's FR2 "modest rolling hills" assumption was made for pathfinding, not the vista register — "may need conscious revisiting, not silent stretching." To be decided **on the record** at worldgen tuning.

**Counter-metrics (success must not cost these):**

- **CM1** — M2 ships in **10–14 stories**. Materially more ⇒ cut scope, not extend the plan. Pre-decided cut order: **FR29 (lanterns) first**, then parity narrows — **FR35/FR36 shrink to camera + speed control**.
- **CM2** — "As soon as possible" has teeth: the first boot-frame wow (world, light, aurora, no input needed) lands in the milestone's **first third**. A plan that back-loads the visual payoff is wrong, cap or no cap.
- **CM3** — No TUI regression during M2; sim-level changes update the TUI.
- **CM4** — Planning docs stay re-readable in one sitting.

**Scope baseline constraints:** M2 starts against today's simd and today's seeded
worldgen; dwarf count stays at FR3's five; world stays 128×128×32. M1 non-goals stand
except trees and light emitters, the only reopenings.

### PRD Completeness Assessment

**Strong.** This is an unusually disciplined PRD for a visual milestone, and it is
self-aware about the failure mode that produced it. Specific strengths:

- Every Visual Target line is written as an outcome, and the PRD polices its own rule
  ("no line may name a rendering technique"). That is the correct register for FRs.
- The FR24 defect class — *spec meetable, implemented, and not what was wanted* — is
  addressed structurally (PR1/PR2) rather than by adding more spec text. The closing
  half of the gate is the part most PRDs omit, and it is the half that failed in 4.1a.
- Cut order is pre-decided (CM1) rather than left to panic at the end.
- Out-of-scope is explicit and states "silence is not permission."

Two completeness risks carried into the epic-coverage step:

1. **Ratio of unnumbered to numbered requirements is high.** 11 FRs and 4 NFRs against
   6 VT bars, 6 anti-requirements, 5 process requirements and 4 counter-metrics. The
   unnumbered ones carry the milestone's actual definition of success (WB1/WB2 are the
   done sentence) yet have no IDs to trace against. **Step 3 will trace them anyway.**
2. **NFR6's fps number is marked a blocking item to be set at architecture time.**
   Whether the architecture pass actually set it is a step-4 check, not an assumption.

## Step 3 — Epic Coverage Validation

Source: `epics.md` (frontmatter `stepsCompleted: [1,2,3,4]`, `milestone: 2`), read in
full for Epics 5–8 plus the M2 requirements inventory and coverage maps. The document
carries its own **FR Coverage Map**, **NFR coverage** and **UX-DR coverage** statements
— I validated those claims against the story acceptance criteria rather than trusting
them.

### Coverage Matrix — Functional Requirements

| FR | PRD requirement (abbrev.) | Epic coverage | Story-level AC | Status |
|---|---|---|---|---|
| FR27 | Seeded pine trees on the surface | Epic 5 | 5.1 — `TreeTrunk`/`TreeFoliage` materials, seeded stream, pathing, TUI glyphs, determinism | ✓ Covered |
| FR28 | Static warm light emitters (torches, campfire) | Epic 5 | 5.1 — `EntityKind::Torch`/`Campfire`, `light: Some(..)`, TUI glyphs | ✓ Covered |
| FR29 | Dwarves carry lanterns | Epic 6 | 6.2 — `light: Some(Lantern)`, no fuel/pickup, moving light | ✓ Covered *(cut-list #1)* |
| FR30 | Protocol vocabulary growth | Epic 5 | 5.1 — the `light` field + Material/EntityKind/LightKind variants; `Lantern` variant lands in 6.2 | ✓ Covered |
| FR31 | Isometric orbitable diorama, one zoom continuum | Epic 5 | 5.3 — orbit/zoom, never lost; 5.4 — the two registers, same view not a mode | ✓ Covered |
| FR32 | Cold/warm read live; sky, stars, aurora, snowfall | Epic 5 | 5.4 — palette, warm pools, sky-as-illuminant, client-local atmosphere; 6.2 — lantern pools | ✓ Covered |
| FR33 | Slice into the mountain; always know your z-level | Epic 7 | 7.1 — mechanism chosen by testing, level legibility, client-local slice state, headless assertions | ⚠️ Covered with a caveat *(see Gap 3)* |
| FR34 | The world visibly lives from wire state alone | Epic 6 | 6.1 — interpolation, dig-face work, rubble, flicker, idle wandering, 30-second no-command watch | ✓ Covered |
| FR35 | Full TUI command parity from the Bevy client | Epic 8 | 8.2 — designate dig/channel, place/remove stockpile; 8.3 — pause/resume, tick rate, save/load, quit | ⚠️ **Partial — `cancel designation` has no AC** *(Gap 1)* |
| FR36 | Mouse picking of tiles and rectangles in 3D | Epic 8 | 8.1 — ray-to-tile, hover highlight, edge cases, headless assertions; 8.2 — rect drag on sliced levels | ✓ Covered |
| FR37 | Bevy client is a `protocol`-only consumer | Epic 5 | 5.3 — dependency edges, gate probe, snapshot/delta via `client-core`, concurrent TUI; re-exercised 7.2, 8.3 | ✓ Covered |

**No FRs appear in the epics that are absent from the PRD.** The M2 inventory in
`epics.md` reproduces FR27–FR37 faithfully; the only additions are cut-list annotations,
which trace to the PRD's counter-metrics.

### Coverage Matrix — Non-Functional Requirements

| NFR | Claimed coverage | Verified | Status |
|---|---|---|---|
| NFR5 (no drift) | "a bar on every `gui` story" | Enforced as an explicit AC in 5.4 (atmosphere carve-out), 6.1 (cosmetic chips), 7.1 (slice level is client-local), 8.1 (no picking state on the wire) | ✓ Covered |
| NFR6 (Bevy feel bar) | Instrument in Epic 5, re-measured in 6 and 8 | **The PRD's blocking item was discharged:** the number is now measured — 60 fps at working zoom, ≥30 fps at full vista, read from a frame-time overlay. Re-asserted in 5.3, 5.4, 6.1, 6.2, 7.1, 8.1, and as the ~200 ms ack bar in 8.2 | ✓ Covered |
| NFR7 (determinism) | Epic 5 worldgen story | 5.1 (trees + emitters identical tile-for-tile) and 6.2 (lantern state) | ✓ Covered |
| NFR8 (gate sibling probes) | `client-core` and `gui`, both Epic 5 | 5.2 (`client-core` probe) and 5.3 (`gui` probe) | ✓ Covered |

### Coverage Matrix — UX Design Requirements (UX-DR1…22)

The epics document promotes the PRD's Visual Target prose into 22 numbered UX-DRs —
which is precisely the traceability move step 2 flagged as necessary. I checked all 22
against story ACs:

| Claimed | Verified in ACs |
|---|---|
| Epic 5 — UX-DR1, 2, 4, 5, 6, 7, 8, 10, 11, 12, 13, 15, 16, 20 | All 14 present (UX-DR1/20 in 5.3; the other twelve in 5.4) ✓ |
| Epic 6 — UX-DR9, 14, 19 | All 3 present in 6.1 ✓ |
| Epic 7 — UX-DR3, 17, 18 | UX-DR3 + 18 in 7.1, UX-DR17 + 5 in 7.2 ✓ |
| Epic 8 — UX-DR21 | Present in 8.1 and 8.2 ✓ |
| UX-DR22 — "binds every visually subjective story in every epic" | ⚠️ **Not applied anywhere in Epic 8** *(Gap 2)* |

### Missing Requirements

#### Gap 1 — FR35's `cancel designation` has no acceptance criterion (HIGH)

**FR35 (PRD):** "designate dig/channel, **cancel designation**, place/remove stockpile,
pause/resume, tick rate, save/load, quit."

Cancel-designation appears in Epic 8's narrative ("cancelling them") and in story 8.2's
*"I want"* sentence ("designate digs and channels, **cancel them**, and place and remove
stockpiles") — but **it is absent from 8.2's acceptance criteria**, which enumerate only
"dig and channel designations, stockpile placement and stockpile removal". No other M2
story issues a cancel from the Bevy client. Story 7.2 renders a *TUI-issued* cancel, which
is not the same requirement.

- **Impact:** The one command in the parity list with no AC anywhere. A dev agent works
  the ACs; parity ships one command short and the miss surfaces at 8.3's "full parity"
  claim or, worse, at Wolf's hands. This project's own record says an AC that cannot be
  met as written is its most frequently caught defect class — this is the mirror case,
  a requirement with no AC at all, which no review layer catches because nothing is wrong
  with what *is* written.
- **Recommendation:** Add `cancel_designation` to 8.2's first AC alongside the other
  world-mutating commands. One clause; it needs no new story. (`remove_stockpile` and
  `cancel_designation` are both already AD-10 queue commands, so the plumbing is shared.)

#### Gap 2 — UX-DR22's sign-off gate is unstated for all of Epic 8 (MEDIUM)

Epics 5, 6 and 7 each carry an explicit gate statement, and Epic 5 goes further by naming
the exclusion on the record: *"The sign-off gate applies to 5.4 and not to 5.1–5.3. 5.3 is
deliberately allowed to be ugly."* That is exactly right, and it sets the precedent.

**Epic 8 contains no reference to UX-DR22 at all** — no epic-level statement, no story AC
in 8.1, 8.2 or 8.3. Yet 8.1 ships a *hover highlight* and 8.2 ships *drag-rectangle
feedback*, both of which are visible artifacts the player looks at while working, and 8.3's
closing AC asks Wolf to sign off both wow beats and the six anti-requirement words — a
live-viewing judgement with no approved artifact to compare against.

- **Impact:** Silence is ambiguous where Epic 5 was explicit. Either Epic 8 needs the gate
  and it was dropped, or it is deliberately exempt and the reasoning is unrecorded. The
  PRD calls this gate "the structural fix for the FR24 defect class" — leaving its
  applicability implicit for the epic that ends the milestone weakens the fix at the
  point of highest consequence.
- **Recommendation:** Add one line to Epic 8 in the Epic 5 pattern, stating which stories
  the gate binds and which it does not, with the reason. My read: 8.1's highlight and
  8.2's drag feedback are *legibility* work already governed by UX-DR17/18 rather than
  fresh visual composition, so a full artifact cycle is likely overkill — but that is a
  judgement to record, not to leave to inference. 8.3's milestone sign-off should
  explicitly cite UX-DR22's closing half.

#### Gap 3 — the vista mountain silhouette question has no owning story (MEDIUM)

The spine records three "decisions owed inside M2 stories": the z-slice control mechanism
(UX-DR3), the world-edge treatment (UX-DR12), and **the vista mountain silhouette** —
should in-grid terrain give the skyline peaks backlit by the aurora within 128×128×32?
The spine is emphatic that M1's FR2 "modest rolling hills" assumption *"was made for
pathfinding, not for the vista register, so this needs conscious revisiting on the record
at worldgen tuning, never silent stretching."*

The first two landed in stories: UX-DR3 → 7.1's opening AC, UX-DR12 → 5.4's zoom-continuum
AC. **The third appears nowhere outside the requirements inventory** — the string
`silhouette` occurs exactly once in the M2 material, at line 141, and never in a story.
Story 5.1 is the worldgen-tuning story and its ACs cover trees and emitters only; story
5.4 is the vista story and its ACs cover edge dissolve but not skyline terrain.

- **Impact:** This is the "silent stretching" the spine names by that phrase. The vista
  register is half of FR31 and the backdrop for the whole cold-boot wow; if the answer is
  "yes, tune terrain for a skyline", it is a worldgen change belonging to 5.1 — before
  5.4 builds the vista on top of it. Discovering it at 5.4 means re-opening a story two
  slots back, or accepting a flat horizon by default, which is a decision made by
  omission rather than on the record.
- **Recommendation:** Add the question to **5.1** as a decision-on-the-record AC ("Given
  the vista register, then the story states on the record whether in-grid terrain is
  tuned for a skyline silhouette, and why"), and cross-reference it from 5.4. It costs
  one clause and closes the last of the spine's three owed decisions.

### Coverage Statistics

- **Total PRD M2 FRs:** 11 (FR27–FR37)
- **FRs mapped to an epic:** 11 — **100%**
- **FRs fully covered by story-level acceptance criteria:** 10 — **91%** (FR35 partial)
- **Total M2 NFRs:** 4 — **4 covered, 100%**, and NFR6's PRD-blocking fps number was set
- **Total UX-DRs:** 22 — **21 applied in story ACs, 95%** (UX-DR22 unstated for Epic 8)
- **Spine "decisions owed inside M2 stories":** 3 — **2 assigned, 67%**
- **Story count:** 11, inside the 10–14 counter-metric ✓
- **M1 FRs re-opened without authority:** none. Trees and light emitters are the only
  reopenings and both are sanctioned by the PRD.

**Headline:** epic-level coverage is complete and the coverage maps in `epics.md` are
honest. Every gap found is at the acceptance-criteria layer, in requirements that the
document's own prose acknowledges — the plan says the right thing in narrative and then
does not always bind it in an AC. All three are one- or two-clause fixes; none requires
a new story or re-planning.

## Step 4 — UX Alignment Assessment

### UX Document Status

**Not found** — no `*ux*.md` and no `ux-designs/` run folder exists, for M1 or M2. This
is a deliberate, documented choice, not an oversight: the epics document states it twice,
and `docs/technical-preferences.md` makes documentation restraint policy.

**UX is unquestionably implied** — M2 is a visual milestone whose entire deliverable is
what a human sees on screen. So the warning stands on the record, but the substance is
covered by a **de-facto UX contract**: `epics.md` promotes the PRD's *Visual Target &
Game Feel* prose into **UX-DR1…UX-DR22**, and Wolf accepted `docs/narrative.md` plus the
two reference images as the UX input for this assessment.

### UX ↔ PRD Alignment

**Complete and faithful. 22 of 22 trace cleanly**, with no UX requirement invented and
none of the PRD's visual prose dropped:

| PRD Visual Target section | UX-DRs | Verdict |
|---|---|---|
| The view (3 bullets) | UX-DR1–3 | ✓ verbatim in substance |
| The light — the wow mechanism (3 bullets) | UX-DR4–6 | ✓ |
| What the references bind (6 bars) | UX-DR7–12 | ✓ one-for-one with my step-2 VT1–VT6 |
| The two wow beats | UX-DR13–14 | ✓ including "beat 1 alone has failed the milestone" |
| The anti-requirements (6) | UX-DR15–20 | ✓ one-for-one with AR1–AR6 |
| Sign-off gate | UX-DR22 | ✓ both halves preserved |
| *(derived from FR35/FR36)* | UX-DR21 | ✓ traceable, not invented |

This is the single strongest thing in the M2 plan. The PRD's own diagnosis of the FR24
failure was that a *mechanism* got specified where an *outcome* was wanted; the UX-DR
extraction holds that line — I could not find a UX-DR that names a rendering technique,
and each one is written so a story can fail it.

### UX ↔ Architecture Alignment

The spine supports every UX-DR that needs a mechanism, and in several cases addresses
one directly and by name:

| UX-DR | Architectural support | Verdict |
|---|---|---|
| UX-DR1, 2, 20 (isometric orbit, zoom continuum, camera usable) | AD-14 camera rigs as client-local; the single `world_to_render`/`render_to_world` transform pair; NFR6 at both registers | ✓ |
| UX-DR3 (z-slice) | Spine Deferred — binds the outcome (FR33), leaves mechanism to the story | ✓ correct deferral |
| UX-DR4, 5, 6 (cold against warm) | AD-16 light-as-entity + `gui` data table keyed by `LightKind`; wire never carries RGB/radius/flicker | ✓ |
| UX-DR7 (sky is an illuminant) | AD-14/AD-15 client-local atmosphere sanctioned by NFR5's carve-out | ✓ |
| UX-DR8 (snow as a settled cap) | **AD-16 names it explicitly**: snow capping is presentation, computed from material + exposure, never wire state | ✓ direct |
| UX-DR9 (work leaves evidence) | **AD-15 names dig-face cosmetic chips** in the carve-out list | ✓ direct |
| UX-DR10, 11 (value discipline, cold field varies) | No mechanism needed; `gui` data tables + the tech-art guidelines deliverable | ✓ |
| UX-DR12 (edges dissolve) | Spine Deferred — binds the no-raw-edge bar only | ✓ correct deferral |
| UX-DR13, 14 (wow beats) | AD-17 rung 3: "rung 3's judge is Wolf's eye, structurally" | ✓ |
| UX-DR15–19 (anti-requirements) | AD-17 evidence ladder + NFR6 | ✓ |
| UX-DR21 (mouse picking, full command set) | AD-18 rect helper (one implementation, both clients), transform pair, AD-10 commands unchanged | ✓ |
| UX-DR22 (sign-off gate) | **AD-17 draws the boundary explicitly**: captures serve the *closing* half and never replace the *opening* half | ✓ direct |

No UI capability in the UX set is unsupported by the architecture, and no architectural
decision contradicts a UX-DR. NFR6 was set as a measured number in the spine exactly as
the PRD demanded, so the performance side of the UX bars is enforceable rather than felt.

### Alignment Issues

#### Gap 4 — the NFR6 frame-time overlay will be burned into the sign-off artifacts (MEDIUM)

AD-14 classifies the NFR6 overlay as a client-local render entity, and stories 5.3, 5.4,
6.1, 6.2, 7.1 and 8.1 each require it to be **read on screen**. Separately, AD-17 rung 3
makes `gui --capture` output the artifact for **UX-DR22's closing half**, and UX-DR13/15
require the boot frame to be "something you'd screenshot unprompted."

**Nothing anywhere says the overlay can be turned off.** Not the spine convention, not
the stack row, not any story AC. As written, the frame that Wolf judges wow beat 1 on —
and every capture offered as sign-off evidence — carries an fps counter in the corner.

- **Impact:** Low severity, high embarrassment, and it lands precisely on the milestone's
  two highest-stakes judgements. It also makes captures weaker as artifacts: an overlay
  changes between runs, so "changes when the world changes" (AD-17's own instrument test)
  becomes trivially true for the wrong reason.
- **Recommendation:** One clause in **5.3**, where the overlay is introduced: the overlay
  is toggleable and off in `--capture` output by default. This also protects AD-17's
  instrument self-test from a false positive.

#### Gap 5 — story 6.1 needs a visible dig face before the client can see underground (MEDIUM)

Story 6.1's central AC is *"Given dwarves working a designated dig, When I watch the dig
face, Then a dwarf in the working state visibly works there, and the site accumulates
evidence."* Two things are unstated:

1. **Who designates.** The Bevy client cannot issue commands until Epic 8, and story 7.2
   — one epic later — is where TUI-issued designations are first rendered. The dig in 6.1
   must therefore come from a TUI client on the same daemon. Story 7.2 states that setup
   explicitly for itself; 6.1 does not.
2. **Where the dig is.** Z-slicing arrives in **7.1, after this story**. Until then the
   Bevy client sees the world from outside only. A dig that goes into or under the
   mountain has its dig face occluded by the very terrain being dug.

- **Impact:** This is the story-3.3 failure mode with the serial numbers filed off, and
  the repo already paid for that lesson: a capture aimed somewhere world-dependent
  returned **zero of every glyph with exit 0**, indistinguishable from a broken feature.
  6.1's instrument is `gui --capture <path> --frames N` — note it carries **no `--z N`
  pinning**, unlike 7.1's and 7.2's, because z-slicing does not exist yet. So 6.1 has the
  weakest aim of any capture in the milestone and the strongest dependency on the dig
  being where the camera is looking. `docs/technical-preferences.md` states the rule this
  violates: *"a scripted capture must be reproducible and must range-check its own
  output… Exit 0 is not a result."*
- **Recommendation:** Add to 6.1: the dig is designated from a TUI client on the same
  daemon at a **surface-visible face**, named in the story, and the capture range-checks a
  non-zero count of working dwarves and rubble at that site. Two clauses, no new story.

### Warnings

⚠️ **No UX document exists for a milestone whose deliverable is entirely visual.**
Mitigated as described — UX-DR1–22 is a real contract and the reference images were
formally reconciled against the PRD. Recorded rather than escalated, per Wolf's decision.

⚠️ **The Bevy client has no interaction spec — no keymap, no mode model, no hint-bar
design — anywhere in the planning set.** M1's equivalent was pinned in the PRD addendum
(`d` dig, `c` channel, `Esc` backs out, and so on). For M2, every interaction decision is
deferred into stories: the z-slice control (7.1), drag-versus-anchor-commit (8.2), and
"the Bevy client's equivalent of the TUI's always-visible hint bar" (8.2, unspecified).

This is **defensible and probably right** — the FR24 lesson was that specifying mechanism
ahead of testing is what caused the milestone's worst failure, and the PRD's
outcome-only discipline is deliberate. But it concentrates unbudgeted design work in
**Epic 8, which is also the cut-risk epic and the story-count driver**. If 8.1/8.2 are
cut, the Bevy client ships with a camera and no interaction model at all, and the M2
interaction question simply rolls to a later milestone undocumented. Worth Wolf's
awareness; it is a scheduling risk, not a planning defect.

ℹ️ **No architectural gap found in the other direction** — there is no UI component in
the UX set that the architecture cannot support, and no spine decision that a UX-DR
contradicts.

## Step 5 — Epic Quality Review

Validated Epics 5–8 and all 11 stories against create-epics-and-stories standards, plus
this repo's own binding story rules in `docs/technical-preferences.md`.

### Epic Structure Validation

**User value focus — all four epics pass.** No technical milestones, no "Setup X" epics.

| Epic | Title is user-centric | Goal states a user outcome | Value standalone |
|---|---|---|---|
| 5 — The Cold Boot | ✓ | "Wolf launches the Bevy client and the first frame stops him" | ✓ "a beautiful viewable client; the TUI still does the commanding" |
| 6 — The Valley Lives | ✓ | "the still image becomes a simulation" | ✓ |
| 7 — Into the Mountain | ✓ | "Wolf slices into the mountain and sees the dig underground" | ✓ |
| 8 — The Boss Gives Orders in Three Dimensions | ✓ | "Wolf works the fortress from the Bevy client with the mouse" | ✓ |

Note that `client-core` — the piece most likely to have become a "Create the shared client
library" technical epic — was correctly placed as a *story inside* a user-value epic
rather than promoted to an epic of its own. That is the right call.

**Epic independence — no violations.** Each epic explicitly declares its position:
Epic 5 standalone; Epic 6 "builds on Epic 5, needs nothing after it"; Epic 7 "builds on
Epics 5–6, needs nothing after it"; Epic 8 "builds on Epics 5–7". I checked every story AC
for references to later work and **found no forward dependencies at any level.**

Two places deserve credit for actively avoiding one:

- **Story 7.2 renders designations issued from a TUI client**, and the epic states the
  reasoning: *"which is why it belongs here rather than in Epic 8: it proves the Bevy
  client renders them with zero game logic of its own, and it means the rendering survives
  if Epic 8's input work is cut."* That is dependency-aware planning against the cut list.
- **Story 8.3 cites AD-15 as "proven at 6.1, now exercised by the feature that actually
  causes it"** — a backward reference to earned ground, which is the correct direction.

**Vocabulary created when needed, not up front.** The analogue of the database-tables
check: story 5.1 adds `LightKind::Torch` and `Campfire` and explicitly defers
`LightKind::Lantern` to 6.2, *"arriving only if FR29 ships."* Textbook. It also means the
cut of FR29 removes a wire variant cleanly rather than leaving a dead one behind.

**Starter template:** architecture specifies none (the workspace exists; M2 adds two
crates). 5.2 and 5.3 scaffold `client-core` and `gui` respectively, each with dependency
edges, `#![forbid(unsafe_code)]`, error crate, and the NFR8 gate probe. ✓ Compliant.

### Acceptance Criteria Review

**Format and traceability: excellent.** Every AC across all 11 stories uses
Given/When/Then, and essentially every one cites its FR, UX-DR and AD. I found no vague
"user can do X" criteria of the kind this check exists to catch.

**Error and edge conditions — better than typical.** Specific commendations:

- **8.1** carries a dedicated edge-case AC: cursor over empty sky, over a slice-hidden
  tile, or outside the window picks *nothing* — *"and specifically not a silent fallback
  to a default tile such as the origin, which would issue orders somewhere the player
  never pointed."* That is the failure mode named before it happens.
- **5.2** requires the guard to be shown to have teeth: sabotaging a mirror rule must make
  the comparison fail. This repo has been bitten by self-referential tests in stories 1.1,
  1.2 and 1.3; the sabotage requirement is the learned countermeasure, correctly applied.
- **7.1** asserts clamping at world bounds; **5.1** asserts determinism tile-for-tile.
- **8.3's cut contingency is written down**: if FR35/FR36 are cut, *"story 8.3's
  walking-skeleton AC changes with it and must be rewritten, not silently reinterpreted."*
  The plan names its own most-caught defect class and pre-empts it.

### Findings by Severity

#### 🔴 Critical Violations

**None.** No technical epics, no forward dependencies, no story that cannot be completed
as scoped. This is an unusually clean structural result and I want it stated plainly
rather than padded with manufactured criticism.

#### 🟠 Major Issues

**M1 — Stories 5.4 and 6.2 name no observability instrument (violates a binding repo rule).**

`docs/technical-preferences.md` — restated verbatim in the epics document's own M2
inventory — requires that *"every story names its observability instrument in a task and
tests the instrument"*, and the rule's rationale is that an untested instrument
"manufactures false evidence rather than merely missing true evidence — which is worse
than having none, because it is believed."

Counting instrument/capture references per story: 5.1 ✓, 5.2 ✓, 5.3 ✓, **5.4 ✗ (zero)**,
6.1 ✓, **6.2 ✗ (zero)**, 7.1 ✓, 7.2 ✓, 8.1 ✓, 8.2 ✓, 8.3 ✓.

- **5.4 is the wow-beat-1 story.** Its only evidence channel is the frame-time overlay
  (a number, not a picture) and Wolf's live viewing. Yet AD-17 rung 3 states that
  `gui --capture` output *is* the artifact for the sign-off gate's closing half — so 5.4
  is the story that most needs the capture instrument and is the one story that never
  invokes it. Story 5.3 builds the instrument; 5.4 then does not use it.
- **6.2's** ACs are worldgen, TUI reasoning, determinism, a live watch, NFR6 and sign-off
  — nothing scripted or reproducible showing a lantern's light moving with a dwarf.
- **Recommendation:** add a capture AC to both, in the pattern the other stories already
  use. For 5.4: `gui --capture <path> --frames N` at the boot framing, range-checking a
  non-zero count of warm-lit pixels or emitter entities, retained as the artifact
  alongside Wolf's live sign-off. For 6.2: a capture across a span of ticks showing the
  lit region translating with the dwarf.

**M2 — Story 5.3 is the largest and riskiest story in the milestone with no named split contingency.**

The epic reserves slack explicitly for *"splitting Epic 5's crate story if it overruns one
dev session"* — that is 5.2. But **5.3 is at least as large and carries the milestone's
only unproven-until-run risk.** Its ACs require: the `gui` crate + dependency edges + gate
probe, the bevy 0.19 dependency, a window that renders on WSLg, recording which wgpu
backend initialised, connect + snapshot + delta via `client-core`, world projection of
terrain/dwarves/items/emitters, reconciliation keyed by sim `Id` with a full
re-projection equivalence test, the single transform pair with a round-trip test, an
orbit camera meeting UX-DR1/20, the frame-time overlay with a baseline measurement, and
the `--capture` instrument *with its own self-tests*.

"Allowed to be ugly" bounds 5.3's *visual* scope, which is a real and clever saving — but
it does not bound its *structural* scope, and none of the above is optional.

- **Recommendation:** name a split line in advance, as was done for 5.2. The natural cut
  is **envelope + lifecycle** (crate, probe, window, backend recording, connect, mirror
  ingestion, concurrent TUI) versus **projection + instruments** (reconciliation, transform
  pair, camera, overlay, capture). The first half alone is a legitimate observable story:
  a window on this machine showing real world state.

**M3 — The highest-consequence unknown in M2 has no stated contingency.**

The spine is emphatic: wgpu prefers Vulkan via WSLg's Dozen driver, *"younger and less
conformant"* than the GL path `glxinfo` proved — *"unproven until run, and
non-negotiable."* Story 5.3 handles this correctly at the reporting level (*"if the
envelope does not hold, that is this story's finding and it is reported, never worked
around silently in production code"* — exactly right, and consistent with this repo's
sandbox rule).

But **8 of 11 stories sit downstream of that finding**, and the plan says nothing about
what happens if it comes back negative. The milestone has no Plan B on the record.

- **Impact:** If 5.3 fails at story 3 of 11, M2 stalls with no pre-agreed next move, and
  the decision gets made under time pressure rather than now while it is cheap.
- **Recommendation:** record the fallback ladder in Epic 5, one sentence: force the GL
  backend that `glxinfo` proved (`WGPU_BACKEND`), then the spine's already-Deferred
  **native Windows build** whose trigger is currently only "Wolf calls for it" — a failed
  envelope is precisely such a call. Deciding nothing is fine; *recording* that these are
  the two candidates costs a line and removes the panic.

#### 🟡 Minor Concerns

**m1 — Story 5.2 is a developer-facing story, and the standard treats that as a red flag.**
*"As a developer, I want a `client-core` crate…"* with a success condition of **identical
output before and after** is, by construction, a refactor with zero user-visible change.
The repo's own rule bans pure-refactoring stories, though it was written for milestone 1.

I am **not** calling this a violation, because the plan answers it well: AD-13 makes
`client-core` load-bearing, the story is explicitly on no cut list, and the observability
answer (identical scripted capture before/after, plus a sabotage check that must fail) is
a genuinely good instrument for a refactor. Recorded so the judgement is visible, not
inherited silently.

**m2 — Story 5.1 does TUI work in a layer that story 5.2 then retires.**
5.1 renders trees and emitters in the TUI; 5.2 retires the TUI's in-crate client state.
The glyph draw sites survive; the state-application work does not. Small, real rework.
Reversing the order would trade it for a different cost (a mirror built before there is
new vocabulary to exercise it), so the current order is defensible — but a dev agent
working 5.1 should know the state layer it touches is scheduled for demolition.

**m3 — Two stories have completion conditions that no automated check can close.**
5.4 ("the eye lands on the encampment first", "something you'd screenshot unprompted") and
6.1's wow-beat-2 sign-off end on human judgement. This is **by design** — AD-17 makes
"Wolf's eye" rung 3's judge structurally, and the whole sign-off gate exists because no
review layer can catch the FR24 defect class. The consequence to plan around: **5.4 and
6.1 cannot be closed by a dev agent autonomously**, and 5.4 has an unbounded iteration
count sitting directly on the first-third-wow critical path. Worth Wolf's calendar
awareness, not a plan change.

**m4 — Story sizing across the milestone is uneven.** Largest: 5.2, 5.3, 5.4, 6.1, 8.3.
Smallest: 6.2. Given a hard 10–14 cap, the count is honest — but the *effort* behind the
count is not uniform, and three of the five heaviest stories are consecutive (5.2–5.4).

### Best Practices Compliance Checklist

| Check | Epic 5 | Epic 6 | Epic 7 | Epic 8 |
|---|---|---|---|---|
| Epic delivers user value | ✓ | ✓ | ✓ | ✓ |
| Epic functions independently of later epics | ✓ | ✓ | ✓ | ✓ |
| Stories appropriately sized | ⚠️ 5.2/5.3/5.4 heavy | ⚠️ 6.1 heavy | ✓ | ⚠️ 8.3 heavy |
| No forward dependencies | ✓ | ✓ | ✓ | ✓ |
| Structures/vocabulary created when needed | ✓ exemplary | ✓ | ✓ | ✓ |
| Clear acceptance criteria | ✓ | ✓ | ✓ | ⚠️ FR35 cancel missing |
| Observability instrument named and tested | ⚠️ 5.4 missing | ⚠️ 6.2 missing | ✓ | ✓ |
| Traceability to FRs maintained | ✓ | ✓ | ✓ | ✓ |

## Step 6 — Summary and Recommendations

### One More Finding: the first-third counter-metric is claimed more confidently than it holds

Not attributable to a single step, so recorded here.

CM2 states: *"the first boot-frame wow lands in the milestone's first third. A plan that
back-loads the visual payoff is wrong, cap or no cap."* The epics document answers:
*"Wow beat 1 lands at story 4 of 11 — inside the PRD's first-third mandate."*

Story 4 of 11 **completes at 36%**, not inside the first third by count. More importantly,
the plan contains two documented paths that push it further out and neither is reconciled
with the claim:

- The epic reserves slack for **splitting 5.2** if it overruns a session → 5.4 becomes
  story 5 of 12 = **42%**.
- Finding M2 recommends a split line for **5.3** as well → story 5 or 6 of 12–13.

And by *effort* rather than story count the picture is worse: **5.2, 5.3 and 5.4 are three
of the milestone's five heaviest stories, consecutive, and all three precede the wow
beat.** Counting stories flatters the calendar.

- **Impact:** CM2 is the counter-metric Wolf wrote to stop exactly this. The plan is not
  violating it today, but it is closer to the edge than the document's confident sentence
  suggests, and its own contingencies breach it.
- **Recommendation:** no re-plan. Replace the claim with the honest version — *"beat 1
  lands at story 4 of 11; if 5.2 or 5.3 splits, it moves to 5 and CM2 is at risk"* — and
  treat a split of 5.2 or 5.3 as the trigger to re-check CM2 rather than a free move. If
  the milestone needs a genuine safety valve, the candidate is thinning 5.3 further
  (M2's split), not delaying 5.4.

### Overall Readiness Status

## **NEEDS WORK — light.**

Every finding is an acceptance-criteria edit to one file (`epics.md`). **Nothing upstream
moves:** the PRD needs no change, the architecture spine needs no change, no story is
added or removed, and no epic is resequenced. Realistically this is under an hour of
editing before story 5.1 can be created.

That is a good result for a milestone this visual, and it is worth naming why. The PRD's
structural response to the FR24 failure — outcomes not mechanisms, a two-halved sign-off
gate, a pre-decided cut order — survived intact through the spine and into the stories.
The recurring shape of what I *did* find is narrow and consistent: **the plan states the
right thing in prose and then does not always bind it in an acceptance criterion.**
Cancel-designation, the sign-off gate in Epic 8, the mountain silhouette, and the missing
instruments in 5.4/6.2 are all the same failure — narrative coverage without AC coverage.
A dev agent works the ACs.

### Critical Issues Requiring Immediate Action

None are blocking in the "stop, re-plan" sense. Ranked by consequence:

| # | Finding | Severity | Fix |
|---|---|---|---|
| 1 | **FR35's `cancel designation` has no AC anywhere** (Gap 1) | HIGH | One clause in 8.2's first AC |
| 2 | **5.4 and 6.2 name no observability instrument** — violates a binding repo rule, and 5.4 is the wow-beat-1 story whose sign-off artifact *is* a capture (M1) | HIGH | One capture AC each |
| 3 | **No contingency if the WSLg render envelope fails** — 8 of 11 stories are downstream of story 5.3 (M3) | HIGH | One sentence in Epic 5 naming the fallback ladder |
| 4 | **6.1's dig face may not be visible** — z-slicing arrives in 7.1, and 6.1's capture has no `--z` pinning; this is the story-3.3 failure exactly (Gap 5) | MEDIUM | Pin a surface-visible dig site + name the TUI as its source |
| 5 | **The vista mountain silhouette has no owning story** — the last of the spine's three "decisions owed", and the one it warned against stretching silently (Gap 3) | MEDIUM | Decision-on-the-record AC in 5.1 |
| 6 | **The fps overlay will be burned into the sign-off artifacts** (Gap 4) | MEDIUM | Toggle, off in `--capture`, stated in 5.3 |
| 7 | **UX-DR22's sign-off gate is unstated for all of Epic 8** (Gap 2) | MEDIUM | One line in Epic 5's pattern, saying which stories it binds and why |
| 8 | **5.3 has no split contingency despite being the largest, riskiest story** (M2) | MEDIUM | Name the split line now |
| 9 | **CM2's first-third claim is at 36% and both split paths breach it** | MEDIUM | Restate honestly; make a split the trigger to re-check |

Plus four minor concerns (m1–m4) and two standing warnings (no UX document; no Bevy
interaction spec, deliberately deferred into Epic 8).

### Recommended Next Steps

1. **Apply fixes 1–9 to `epics.md`.** All are AC-level; none touches the PRD or the spine.
   Fixes 1, 2, 4, 5 and 6 are the ones that change what a dev agent actually builds — do
   those even if the rest are skipped.
2. **Re-check nothing upstream.** Explicitly: the PRD and the M2 spine passed this
   assessment with no findings against them. NFR6's blocking fps number was set as
   required; every UX-DR has architectural support; there are no forward dependencies and
   no technical epics.
3. **Commit the M2 planning set.** The PRD and spine are committed; `epics.md`,
   `sprint-status.yaml` and this report are not. The whole M2 plan currently exists only
   in the working tree of `m2-bevy-client-planning`.
4. **Then run sprint planning, then create story 5.1.** Story 5.1 is a clean starting
   point: sim-side, deterministic, testable headlessly, observable through the existing
   TUI instrument, and it carries only one of the fixes above (#5, the silhouette
   decision).
5. **Wolf's calendar item, not a plan change:** 5.4 and 6.1 cannot be closed by a dev
   agent — they end on his eye, by design (AD-17 rung 3). 5.4 also needs its "here is
   what you will see" artifact approved *before* implementation starts. Both sit on the
   critical path to the first-third wow.

### Final Note

This assessment identified **9 substantive issues plus 4 minor concerns across 5
categories** (FR coverage, UX alignment, epic structure, story quality, counter-metric
integrity). None requires re-planning; all are acceptance-criteria edits to `epics.md`.

The M2 plan is materially stronger than the M1 plan was at the equivalent gate — the
FR24 post-mortem visibly shaped it, and the sign-off gate, the outcome-only Visual
Target, the UX-DR extraction and the pre-decided cut order are all real structural work
rather than documentation. Fix the nine, commit, and this is ready to build.

---

**Assessed:** 2026-08-09
**Assessor:** Product Manager (implementation-readiness workflow, steps 1–6)
**Scope:** Milestone 2 — Bevy client (Epics 5–8, 11 stories, FR27–FR37, NFR5–NFR8, UX-DR1–22)

---

## Resolution — all nine fixes applied, 2026-08-09

Wolf authorised the fixes immediately on delivery of this report. All nine were applied
to `epics.md` in the same session and committed alongside it. **The report's NEEDS WORK
status above is the finding at assessment time; the status after remediation is READY.**

Nothing upstream was touched, as predicted: the PRD and the M2 architecture spine are
unchanged, no story was added or removed, and no epic was resequenced.

| # | Finding | Applied as |
|---|---|---|
| 1 | FR35's `cancel designation` had no AC | 8.2's first AC now names cancellation in the world-mutating set (FR35, FR9, AD-10), plus a second clause requiring it to disappear in both clients through absence-is-deletion — the path 7.2 proved for a TUI-issued cancel |
| 2 | 5.4 and 6.2 named no observability instrument | 5.4 gains a `gui --capture` AC at the boot framing (overlay off, range-checks warm-lit emitters, non-black and non-uniform, retained beside the approved artifact). 6.2 gains one showing the lit region **moves with the dwarf** across captures |
| 3 | No contingency if the render envelope fails | Epic 5 records the fallback ladder: force the GL backend `glxinfo` proved (`WGPU_BACKEND`), then escalate to the spine's deferred native Windows build — a failed envelope being exactly the trigger its "Wolf calls for it" clause anticipates |
| 4 | 6.1's dig face might not be visible | 6.1's AC now pins the dig to a **surface-visible face named in the story**, designated **from a TUI client**, and cites story 3.3's false failure by name; its capture AC range-checks working dwarves and rubble at that site |
| 5 | The vista mountain silhouette had no owning story | 5.1 gains a decision-on-the-record AC; 5.4's vista AC now builds on that decision rather than re-opening it |
| 6 | The fps overlay would be burned into sign-off artifacts | 5.3's overlay AC now requires it toggleable and off by default in `--capture`, with the instrument-self-test false positive named as the second reason |
| 7 | UX-DR22 unstated for Epic 8 | Epic 8 now states the gate applies to 8.3 and **not** to 8.1–8.2, with the reason: 8.1/8.2 are legibility work under UX-DR17/18 on a look 5.4 and 7.2 already settled |
| 8 | 5.3 had no split contingency | Epic 5 names split lines for **both** heavy stories — 5.3 splits into envelope + lifecycle versus projection + instruments, the first half standing alone as an observable story |
| 9 | CM2's first-third claim overstated | Both the M2 epic-list preamble and Epic 5 now state the honest position: beat 1 completes at 36%, at the edge; a split of 5.2 or 5.3 pushes it to 42% and breaches CM2, making a split the trigger to re-check CM2 rather than a free move |

**Verification:** all eleven M2 stories now reference an observability instrument
(previously 5.4 and 6.2 scored zero), and each fix was confirmed present in the file by
string match after editing.

**Not changed, deliberately:** the four minor concerns (m1–m4) and the two standing
warnings. m1 (5.2 is developer-facing) and m3 (5.4 and 6.1 close on Wolf's judgement) are
recorded judgements rather than defects; m2 (5.1 touches a layer 5.2 retires) and m4
(uneven sizing) are consequences of an ordering that is correct on balance. The two
warnings — no UX document, no Bevy interaction spec — are deliberate choices of this
project's documentation-restraint policy and the PRD's outcome-only discipline.

**Status after remediation: READY.** Next: sprint planning, then create story 5.1.
