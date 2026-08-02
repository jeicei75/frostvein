# Reconciliation — technical-preferences.md vs PRD + addendum

Source: `/workspace/projects/frostvein/docs/technical-preferences.md`
Scope note: stack/ADR content in the source deliberately belongs in the future
architecture doc, not the PRD. Absent stack details are therefore NOT reported
as gaps; only contradictions, restraint/story-rule violations, FR-forced
anti-overengineering violations, and genuinely PRD-relevant dropped constraints.

## Covered (one line)

All five pre-made ADRs are faithfully reflected as capabilities (FR11 A*, FR13
fixed 10/s tick, FR14 speed = tick-rate, FR15/NFR3 seed+command-log
determinism, FR17 NDJSON snapshot/delta/commands, FR20 zero client logic);
anti-overengineering is carried (YAGNI counter-metric, FR17 no
batching/compression, FR11 no hierarchy/caching, FR1 no chunking); story rules
(8–12 vertical slices) and the quality gate (NFR4 exact command match) are in;
no personas or market analysis appear; color-as-data/truecolor (FR22) and the
scenario harness (FR25/26 = the milestone gate) match the source verbatim in
spirit.

## Gaps

1. **"Every story ends with something observable … No pure-infrastructure or
   pure-refactoring stories in milestone 1" and "A story fits one dev-agent
   session"** (source, Story rules) — the PRD counter-metrics carry "8–12
   vertically sliced stories" but drop the observable-ending / no-infra-story
   / one-session-fit constraints. These are planning constraints the
   epics/story step will need; if the PRD's counter-metrics section is the
   place story rules are being restated, it is restating them incompletely.
   Severity: **low** (legitimately downstream to the SM, but the PRD chose to
   restate the sibling rules and omitted these).

2. No other PRD-relevant source constraint is missing. Dev-workflow items
   (dev writes own tests, no QA gate, small commits) and dependency/unsafe
   rules are process/architecture concerns, correctly absent from a
   capabilities-only PRD.

## Contradictions / Violations

1. **Raycast 3D view scheduled inside milestone 1.**
   - Source quote: "the **future** raycast view reuses this" (Stack → TUI) —
     the source's only mention of raycast frames it as future work; nothing in
     the source puts it in milestone 1.
   - PRD location: FR24 ("The raycast 3D view is its own story late in the
     milestone and may slip to phase two without ceremony"); also Future
     phases ("TUI raycast 3D view (if slipped)").
   - A raycast renderer is a whole additional presentation of the world inside
     an 8–12-story milestone whose gate is the walking skeleton. The "may slip
     without ceremony" hedge softens it, but as written it consumes a story
     slot the source's story cap ("Milestone 1 is 8–12 stories. More than that
     means scope creep — cut, don't plan") did not budget for.
   - Severity: **medium**.

2. **FR footprint pressures the 8–12 story cap.**
   - Source quote: "Milestone 1 is 8–12 stories. More than that means scope
     creep — cut, don't plan." (Story rules)
   - PRD location: 26 FRs across 9 feature groups, including several with no
     source basis beyond the walking skeleton: FR16 (dev save/load of full sim
     state), FR19 (concurrent multi-client), FR24 (raycast), channel-dig as a
     second designation mode (FR6/FR9), plus the addendum's `x`
     remove-designation mode. The PRD's own counter-metric restates the cap,
     but the FR list is what the cap must fit, and it is generous. The source
     says the resolution is "cut, don't plan" — the PRD plans and defers the
     cutting to story time.
   - Severity: **medium** (not a textual contradiction; a scope-rule tension
     the epics step will inherit).

3. **FR16 dev save/load — capability with no source basis and hidden cost.**
   - Source quote: "Build for the current story's needs, not the imagined
     fortress of 500 dwarves." (Anti-overengineering); the source's ADR 3
     command list and testing story (seed + command log ⇒ replay) make
     save/load unnecessary for determinism or the milestone gate.
   - PRD location: FR16, and FR18 lists `save`/`load` as protocol commands.
   - Full-sim-state serialization is exactly the kind of horizontal
     infrastructure the source bans as a story ("never a horizontal layer")
     and it duplicates what seed+command-log replay already provides for dev
     purposes. If kept, it must still end in something observable to be a
     legal story.
   - Severity: **low** (explicitly "dev" scoped, no format guarantees — but it
     is the FR most likely to force a YAGNI violation).

4. **FR17's "a world, not a dwarf game" design principle serves only a future
   phase.**
   - Source quote: "No … data-driven content systems until a third concrete
     use case exists in shipped code" / "Introduce the abstraction when the
     second concrete case exists, not before." (Anti-overengineering)
   - PRD location: FR17 ("Design principle: messages describe *a world, not a
     dwarf game* … keeps the channel able to carry any realm a future producer
     feeds it (see Future phases wild card) at zero extra cost now"); PRD
     counter-metric simultaneously states "No code exists that serves only a
     future phase."
   - Mitigation on record: the addendum's Asgard section explicitly rules
     "do NOT design for this now" and sets the second-producer trigger, and
     the principle constrains message *content* rather than adding an
     abstraction. But an FR whose stated justification is the unscheduled wild
     card is in tension with the PRD's own YAGNI counter-metric; the principle
     should be justified by ADR 4 (clients render data, not rules) alone, or
     moved out of FR text.
   - Severity: **low**.

5. **Addendum depth vs documentation restraint.**
   - Source quote: "PRD: a few pages … Default answer to 'should I also
     document X?' is no." (Documentation restraint)
   - PRD location: the addendum's keymap table (UX/story detail), LLM-sidecar
     mechanism sketch, and Asgard generalization design notes are downstream
     depth attached to the PRD artifact. The addendum self-identifies as such
     ("Depth that belongs downstream … not in the PRD narrative"), which is
     honest, but the combined artifact is drifting past "a few pages", and
     success criterion 4 counts "PRD + architecture" doc length.
   - Severity: **low** (parking downstream detail in a clearly-labeled
     addendum is a reasonable compromise; flagging so it isn't grown further).

No high-severity findings: nothing in the PRD or addendum contradicts an ADR,
the determinism rule, the quality gate, or the crate/client-logic boundaries.
