# PRD Quality Review — Frostvein PRD (2026-08-01)

## Overall verdict

This is a strong PRD that does exactly what its own restraint policy demands: a few pages that an architecture agent and a story agent can act on without asking questions. The thesis (headless deterministic sim, thin clients, "player is the boss, dwarves obey in their own time") is genuinely load-bearing — it shows up in FRs (FR5's reaction delay), NFRs (NFR2's explicit exemption of dwarf obedience from the responsiveness bar), and the future-phase LLM sketch. The two real risks are small: the "look delicious" visual-identity requirement (FR23) has no verification hook anywhere in the success criteria, and designation *removal* exists only as a keymap row in the addendum with no FR — which, under the PRD's own "silence is not permission" rule, leaves a story agent guessing.

## Decision-readiness — strong

Decisions read as decisions throughout. The cut list is pre-ordered ("the cut list starts with FR24 … and FR16, in that order" — Success criteria, counter-metrics), FR24 is explicitly slippable "without ceremony," and the addendum records two decisions with triggers rather than hedges: keyboard-first for milestone 1 ("Decision: keyboard-first for milestone 1"), and the Asgard adapter ("Decision: do NOT design for this now" with a recorded trigger — "the second concrete producer exists"). Trade-offs name what was given up: "Chattiness is acceptable; no batching, compression, or interest management" (FR17), "naive retry is acceptable in phase one" (FR8), "No save format stability guarantees" (FR16). There are no `[NOTE FOR PM]` callouts, but I could not find a dodged tension that needed one — the shallow-by-design future phases are declared shallow, not smoothed over.

## Substance over theater — strong

No personas, no market analysis, no differentiation section — correctly absent per `docs/technical-preferences.md`, not missing. The Vision could not swap into another colony-sim PRD: "the icy-grim identity is carried by color and glyph choices from day one, not deferred to future graphics" and "a distant god issuing directives, not a hand on a remote control" are specific bets that later sections cash (FR1's "icy materials are not decoration," FR5, FR23, NFR2). NFRs carry product-specific thresholds, not boilerplate: NFR2 gives "~100 ms frame budget on the dev machine, full 128×128 z-level" and "acknowledged in the UI within ~200 ms (one tick + one frame)."

### Findings
- **low** Future-phases visual-identity bullet exceeds its own "deliberately shallow" rule (§ Future phases, "Visual identity (decided, not built)") — sub-voxel model dimensions ("~10×5×13 for a dwarf"), LOD strategy, and palette-swap identity are architecture/art-pipeline depth inside a section that opens with "no FRs, stories, or abstractions may be created for them yet." It is decided direction, not furniture, but it is the one place the PRD's discipline slips. *Fix:* compress to the decision ("single profession-colored glyphs in 2D; code-authored sub-voxel models for future 3D, no sprites ever") and move the dimensions/LOD detail to the addendum with the other downstream sketches.

## Strategic coherence — strong

The thesis is stated and everything hangs off it: fun-is-the-whole-loop plus determinism-as-harness. Feature order follows the walking-skeleton sentence (Vision, "Phase one delivers the walking skeleton"), and the phase-one gate is that sentence as an automated test (FR26). Success criteria validate the thesis rather than measure activity — criterion 2 requires the same scenario be "watchable live in the TUI … and it meets the feel floor," pairing headless proof with the experienced loop. Counter-metrics are real and self-referential: "8–12 vertically sliced stories … materially more means scope gets cut, not the plan extended," and criterion 4 (docs re-readable in one sitting) "doubles as a counter-metric." MVP scope kind is coherent: an experience-proving vertical slice, and the scope logic matches it.

## Done-ness clarity — adequate

Most FRs carry a testable consequence: FR6 states the exact state transition ("wall becomes open floor; channel: floor is dug out leaving a ramp below") plus the item spawn; FR8 gives a falsifiable property ("never silently dropped"); FR15/NFR3 define determinism as an assertable equality; FR26 turns the milestone into a single automated test. NFR2 converts "feels alive" into numbers and honestly bounds verification ("Checkable by eye; no measurement infrastructure in phase one"). The weak spots are the aesthetic and the soft quantifiers below — few, but this is the dimension story creation leans on hardest.

### Findings
- **medium** FR23's acceptance is unverifiable and unowned (§ F8, FR23) — "The world should look delicious in the terminal" is the PRD's only phase-one requirement with no testable consequence and no reviewer hook; the success criteria cover the scenario test, the feel floor (NFR2), and the quality gate, but nothing gates the visual identity the Vision calls day-one load-bearing. *Fix:* add one line to Success criteria or FR23 making Wolf the acceptance instrument — e.g. "criterion 2 includes Wolf's eyeball sign-off on the icy-grim palette in the live TUI" — so the story that implements FR23 has a named done condition.
- **low** Unpinned quantifiers in FR2 and FR4 (§ F1 FR2, § F2 FR4) — "a few z-levels" and "wander nearby tiles" leave the dev agent to pick numbers. Both are low-stakes and FR2's is `[ASSUMPTION]`-tagged, but a scenario test needs concrete values. *Fix:* let the architecture/story layer pin them; optionally tag FR4's "nearby" with an `[ASSUMPTION]` (e.g. within ~3 tiles) to match the FR2/FR5 pattern.

## Scope honesty — strong

The out-of-scope section is titled "silence is not permission" and does real work — it doesn't just list absences, it disambiguates the ones a reader would plausibly infer wrong: "Ice and snow (FR1) are terrain *materials*, not simulated processes — nothing melts, freezes, or falls," and "idle wandering and reaction delays are seeded behavior, not a mood system." Five inline `[ASSUMPTION]` tags (FR2, FR3, FR5, FR14, FR23) sit exactly where the PM inferred rather than confirmed, each with a concrete default. De-scoping is proposed in the open (the FR24/FR16 cut list). Open-items density is right for the stakes: a handful of tagged assumptions on a solo hobby PRD, none of them blocking.

## Downstream usability — strong

This is a chain-top PRD (feeds architecture then epics/stories) and it extracts cleanly. FR1–FR26 are contiguous and unique; NFR1–NFR4 likewise. Every cross-reference I chased resolves: FR5↔NFR2, FR15↔F9/FR26, FR21→addendum keymap, FR17→Future-phases wild card→addendum Asgard section. The out-of-scope section explicitly exists so the PRD is "safe to hand downstream on its own." No glossary, but at this length every domain noun (dig vs. channel, designation, stockpile, walking skeleton) is defined at first use and used consistently. No UJs — see Shape fit.

### Findings
- **medium** Designation removal exists only in the addendum keymap (addendum § Keymap sketch, row `x` "remove-designation mode (DF parity; confirm in story)") — there is no FR for cancelling a designation, and it isn't in the out-of-scope list either. Under the PRD's own "silence is not permission" rule that makes it out of scope, yet the keymap ships it, and FR18's upstream command list has no remove-designation command to carry it. A story agent hits a genuine contradiction. *Fix:* one line resolves it either way — add "cancel designation" to FR9/FR18, or add "No designation removal (the `x` key is a phase-two candidate)" to out-of-scope and mark the keymap row phase 2.

## Shape fit — strong

Hobby/solo, single-operator, capability-spec shape — exactly what the rubric prescribes for this product type. UJs would be overhead here: there is one protagonist (Wolf as player), and the Vision paragraph carries the experience arc ("issuing an order, watching dwarves obey it live, and feeling the sim breathe under pause and fast-forward") that a UJ section would only restate. The addendum is the right pressure valve — mechanism depth (crossterm mouse pipeline, LLM sidecar determinism argument, Asgard generalization trigger) lives there instead of bloating FR text. Rigor is light where the rubric permits and firm where the project needs it (determinism, the quality gate, the story-count counter-metric). Not over-formalized, not under-formalized.

## Mechanical notes

- **No Assumptions Index.** Five inline `[ASSUMPTION]` tags (FR2, FR3, FR5, FR14, FR23) with no roundtrip index at the end. At this document length the cost of scanning is near zero, but a six-line index would make the architecture agent's confirm-or-override pass mechanical.
- **ID continuity:** FR1–FR26 contiguous, no gaps or duplicates; NFR1–NFR4 clean. Success criteria are numbered 1–4 without SM-style IDs — fine at this scale, nothing references them by ID.
- **Glossary drift:** none found. "Designation," "stockpile," "channel," "tick," "walking skeleton" are used identically across FRs, NFRs, success criteria, and addendum.
- **External reference:** the addendum cites "ADR 4" (Asgard section), which resolves to the pre-made ADRs in `docs/technical-preferences.md`, not to anything in the PRD. Correct content, but a path mention would spare a downstream agent the hunt.
- **Internal consistency spot-checks:** NFR2's "~200 ms (one tick + one frame)" is arithmetically consistent with FR13's 10 ticks/sec and NFR2's own ~100 ms frame budget. FR14's three speeds (pause, 1×, fast) match the keymap's `+`/`-` annotation. FR18's command list covers every keymap action except `x` (see the medium finding above) and cursor movement (client-local, correctly absent).
