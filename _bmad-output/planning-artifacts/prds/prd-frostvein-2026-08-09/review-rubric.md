# PRD Quality Review — Frostvein Milestone 2 (Bevy Client)

Reviewed: `prd.md` + `addendum.md` (2026-08-09), against the M1 PRD they inherit
(`prd-frostvein-2026-08-01/prd.md`, final). Calibration: hobby/solo project with
documentation restraint as explicit policy; Milestone 2 PRD inheriting a final M1
PRD by reference; central design intent is *outcomes, never rendering techniques*
(the FR24 lesson). Absent personas/market sections and un-restated M1 content are
policy compliance, not gaps, and are not penalized.

## Overall verdict

This is a strong, unusually honest PRD: it has a real thesis (cold-against-warm
as the organizing principle, two required wow beats), it converts a concrete
prior failure (4.1a's six words) into testable bars, and its decisions are made
on the record — including a pre-committed cut list and an explicit overturn of an
M1 constraint. The one structural risk is that the sign-off gate closes the FR24
defect class *before* implementation (artifact approval) but never states a
per-story check of the *implemented* result against that artifact — the same gap
through which 4.1a passed a green gate and four review layers and still failed at
live viewing. Everything else is small.

## Decision-readiness — strong

Decisions are stated as decisions throughout. The asset-pipeline reversal is
exemplary: "This **overturns, on the record,** the M1 brief's 'models authored as
code, never as assets, ever'" (§ Scope shape / Art), with the premise that
expired named. The cut list is decided *now*, in order (§ Success criteria,
counter-metrics: "first **FR29** … then parity narrows"), so a scope crunch needs
no new negotiation. The z-slice open question is genuinely open — FR33 states the
outcome and the addendum names the mousewheel/zoom collision plus three candidate
resolutions to test, with no smuggled answer. The parity rule (§ Scope shape)
names both what is given up (no TUI feature growth) and what is protected (no TUI
regression). Nothing here is smoothed to neutral.

### Findings

- **low** Reference-game guidance lives only in the addendum (§ addendum,
  Valheim) — the "budget goes to light, sky, and atmosphere" cost-profile lesson
  is quietly load-bearing for how F11 stories will spend effort, but the PRD body
  never points at it. *Fix:* one pointer sentence in the Vision or Visual Target
  ("cost profile per the addendum's Valheim note").

## Substance over theater — strong

No furniture. There are no personas (policy), no market section (policy), and
the sections that exist all do work. The NFRs are product-specific: NFR6 names
the exact world (128×128×32, all dwarves, all lights) and the ~200 ms
acknowledgement bar; NFR8 names a specific gate probe. The Vision could not be
swapped into another product — "your eye is pulled to the dwarves because they
are the warm thing in the cold" is this game and no other. The anti-requirements
table (§ Visual Target) is the opposite of theater: six bars earned from a real
judged failure. FR29 even declares its own motive honestly ("deliberately the
lighting system's hardest case … placed in scope as a testbed").

### Findings

*(none — nothing reads as furniture)*

## Strategic coherence — strong

The PRD has a thesis and bets on it twice over: (1) state visual outcomes, never
mechanisms — enforced by the Visual Target's standing rule ("**no line may name a
rendering technique**… If a sentence could be satisfied two different ways in
code, it is written correctly"); (2) the wow is warm-against-cold plus aliveness,
and every F10 FR exists to make that light *real in the sim* rather than painted
on. Prioritization follows the thesis, not ease: the counter-metric "'As soon as
possible' has teeth" forces the boot-frame wow into the first third, explicitly
rejecting a back-loaded plan; the cut list cuts FR29 first *because* torches and
campfire still carry the warm/cold wow — a cut ordered by thesis. Success
criteria validate the thesis (both wow beats in one sitting, zero FR24-class
misses), not activity, and counter-metrics exist and bite (story cap, first-third
rule, no-regression, doc length).

### Findings

*(none)*

## Done-ness clarity — adequate

The sim-side work is unambiguous: FR27–FR30 are seeded, deterministic world
state with scenario-test coverage mandated by NFR7 — an engineer knows exactly
what done is. The deliberately subjective client work is handled about as well as
subjective work can be: the anti-requirements table turns six adjectives into
observable checks ("you can tell dwarves, terrain, designations, and items apart
at a glance"), FR34 has a crisp falsifiable consequence ("Zero commands issued
still means visible motion"), and the sign-off gate routes taste through Wolf's
approval of a concrete artifact. But the acceptance loop has a hole (finding 1
below), and two bars remain adjectives until architecture time.

### Findings

- **medium** The sign-off gate is pre-implementation only (§ Visual Target /
  Sign-off gate) — it requires Wolf's approval of a "here is what you will see"
  artifact *before* a visual story is implemented, but no requirement anywhere
  says the *implemented* result is checked live against that artifact per story.
  The only post-implementation checks are milestone-level (success criteria 1
  and 3). 4.1a shipped gate-green through four review layers and failed only at
  live viewing; as written, an M2 story could do the same and the miss would
  surface at milestone end, exactly when it's most expensive. *Fix:* one
  sentence in the sign-off gate: a visually-subjective story is done only when
  Wolf has seen the live result against the approved artifact — the
  per-story twin of success criterion 1.
- **medium** NFR6's feel bar is "interactive framerates" — an adjective — with
  the measured number explicitly deferred to architecture time via
  `[ASSUMPTION]`. The deferral is on the record, indexed, and instructed by the
  sprint change proposal, so this is a tracked open item rather than a silent
  gap; but until the number exists, M2's central promise (a client worth
  watching) has no objective floor, and the anti-requirement bars all assume the
  frame rate is already acceptable. *Fix:* none needed in the PRD itself —
  flagged so the architecture pass treats setting the number as blocking, not
  optional.
- **low** "The far register degrades gracefully" (§ Visual Target / The view) is
  the rubric's flagged phrase-shape. The following clause ("it is the same view,
  not a mode") does give a testable condition — no discrete mode switch — but
  "degrades gracefully" itself adds only an adjective. *Fix:* lean on the
  testable clause: e.g. "the far register is reached by zoom alone — same view,
  no mode switch."

## Scope honesty — strong

Best dimension in the document. Omissions are explicit under a heading that says
so ("silence is not permission"), reopenings are enumerated and minimal ("trees
and light emitters are the only reopenings"), and the atmosphere carve-out in
NFR5 pre-empts the likeliest silent scope creep ("must never acquire sim meaning
silently"). All four `[ASSUMPTION]` tags are inline at real inferences and
round-trip cleanly to the index. The one genuinely unresolved design question
(z-slicing) is honestly held open with its collision named. Open-items density is
low and matches hobby stakes. De-scoping is pre-negotiated, not deferred to a
future argument.

### Findings

*(none)*

## Downstream usability — adequate

This is a chain-top PRD (it feeds the M2 architecture pass, then stories), so
this dimension matters. IDs are clean: FR27–FR37 contiguous from M1's FR26,
F10–F13 from F9, NFR5–NFR8 from NFR4 — no gaps, no duplicates. Cross-references
into M1 all resolve (FR4, FR17, FR18-derived parity list in FR35, FR19, FR24,
NFR2). Terms ("diorama," "register," "z-level," "slice") are used consistently.
There is no glossary, consistent with M1 and the restraint policy; at this
vocabulary size that costs little.

### Findings

- **low** NFR5 cites "AD-1/AD-4" and NFR6/NFR8 cite "the sprint change
  proposal" with no path — neither resolves from the PRD pair alone; a reader
  (or a source-extracting subagent) must already know where the M1 architecture
  spine and the proposal live. *Fix:* parenthetical paths at first mention of
  each.

## Shape fit — strong

Correctly shaped: hobby/solo capability spec, rigor light, substance high. No
UJs and no personas — right for a single-operator project where the operator is
also the acceptance instrument. Inheritance by reference instead of restatement
is exactly what the "few pages / one sitting" policy demands, and success
criterion 5 makes doc length a criterion *and* a counter-metric, as in M1. The
document practices its own central rule: I found no line in the Visual Target or
F11 that names a rendering technique — mechanism words appear only where
mechanisms are legitimately at issue (the addendum's asset-pipeline sketch,
explicitly filed as "depth that belongs downstream"). The addendum split is used
correctly.

### Findings

*(none)*

## Mechanical notes

- Assumptions Index roundtrip: clean. Four inline `[ASSUMPTION]` tags (sign-off
  gate, FR27, FR29, NFR6); all four indexed; no orphan index entries.
- ID continuity: FR27–FR37, F10–F13, NFR5–NFR8 — contiguous with M1, no
  duplicates.
- Cross-references into M1 verified against `prd-frostvein-2026-08-01/prd.md`:
  FR4 (idle aliveness), FR17 (world-not-game), FR19 (concurrent clients), FR24
  (withdrawal), NFR2 (TUI-specific scope note) — all accurate as characterized.
- FR35's parity list matches M1 FR18's command list (designate dig/channel,
  cancel, place/remove stockpile, pause/resume, tick rate, save, load, quit).
- Unresolvable-from-here references: "AD-1/AD-4" (NFR5), "the sprint change
  proposal" (NFR6, NFR8) — see Downstream usability finding.
- UJ protagonist check: n/a — no UJs, appropriate for this shape.
- Frontmatter status is `draft`, correct for a PRD entering review.
