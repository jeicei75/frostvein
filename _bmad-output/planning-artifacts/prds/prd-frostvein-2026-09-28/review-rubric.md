# PRD Quality Review — Frostvein Milestone 3 — Game Functionality

## Overall verdict
The PRD has a real thesis. M3 makes the valley worth *playing* through identity, concurrent trades and a goal chain, and it is checked by a concrete done sentence, a cut order and counter-metrics. It is the right weight for a solo hobby milestone. The main risk is the opening line, "all stand unchanged". M3 in fact overrides at least five inherited M1 requirements and non-goals, the job-retry rule (FR8) among them, and it names none of them. Several FRs also defer the one semantic that makes them testable: how goal progress counts, how long "forever" is, and what happens to an abandoned job. Fix those before epic and story creation and the PRD is ready to build from.

## Decision-readiness — adequate
Where the PRD makes a decision, it states it plainly and names what was given up:
- An idle dwarf "does not help with other trades" (FR42). This accepts idle dwarves in exchange for clear professions.
- Profession assignment and tree designation are "Bevy-only commands in M3; the TUI gains no new input" (NFR10). This is a named exception to the parity rule.
- "There is no fail state" (FR48).
- The cut order is spelled out: FR40, then FR52's trees, then the gem tier.

These are real decisions that someone pushing back could argue with.

The gap is the tensions the PRD does not surface. The header says the inherited PRDs "all stand unchanged". Scope shape says "the only reopening of M1/M2's non-goals is gems and minerals". Both claims are false (see Scope honesty). Separately, putting a tutorial goal chain on the wire conflicts with M1 FR17's design principle, and nothing in the PRD says so. A reader of the architecture pass will find these contradictions downstream instead of here.

### Findings
- **medium** Goals on the wire conflict with FR17's "world, not a dwarf game" principle (§F16 FR47) — M1 FR17 says messages carry "state and typed data … never game rules or narrative interpretation". FR47 puts "Goals and progress" ("collect N wood, then N minerals, then N gems") on the wire as world state, which is a game rule by any reading. The PRD may still be right to do this, but it has to be a decision on the record, not an accident. *Fix:* in FR47 or NFR10, add one line saying FR17's principle is relaxed for the goal chain, or reshape the wire form as neutral data (for example, counters and a goal index) and leave the "tutorial" framing to the client.

## Substance over theater — strong
There are no personas, no innovation section and no boilerplate NFRs. NFR9 and NFR10 each name the specific state they govern ("Identity, professions, resources, goals and one-per-tile movement"). The Vision could not be swapped into another PRD: "one digs, one hauls, one fells pines" is this game. The only furniture-like item is FR40.

### Findings
- **medium** FR40 is speculative preparation that the project's own policy bans, and it cannot be tested (§F14 FR40) — "The dwarf model is prepared for per-dwarf variation beyond colour. This is preparation only. It ships no new variants". `technical-preferences.md`/CLAUDE.md make YAGNI binding: "No abstraction with a single implementation". "Prepared" also has no observable done condition. The stretch label and first-in-cut-order placement contain the damage but do not fix it. *Fix:* delete FR40, or restate it as a concrete, observable capability (for example, "two dwarves render with different beard meshes"). Otherwise park it as an issue.

## Strategic coherence — strong
The thesis is explicit: "Milestone 2 made the valley worth looking at. Milestone 3 makes it worth *playing*". F14–F16 serve it directly: identity to tell dwarves apart, trades to make them work side by side, and goals for direction. The done sentence ties them into one observable session. F17 (defects #132–#135) reads at first like backlog riding along. It is defensible, though, because concurrent work multiplies collisions, stuck dwarves and floating items, so the defects become visible exactly when the thesis succeeds. One sentence making that link would help. The success criteria check the thesis (criteria 1–3), not activity. The counter-metrics are real: a story cap with an ordered cut list, "Look work is not back-loaded", and "No TUI regression ships".

### Findings
- **low** No counter-metric protects sim tick cost (§Counter-metrics) — one-per-tile routing (FR49), re-planning (FR51) and falling (FR52) all add per-tick work. The inherited NFR2 (10 ticks/sec, TUI keeps pace) covers this implicitly, but no M3 criterion re-checks it. *Fix:* add "NFR2's tick rate still holds with five dwarves working concurrently" to the counter-metrics, or say explicitly that NFR2 is inherited unchanged.

## Done-ness clarity — thin
About half the FRs have a clean testable consequence: FR38, FR41, FR42, FR44, FR46 and FR50. The rest hand the defining semantic to "the story" or use an unbounded word. That is acceptable where the knob is truly tuning (the mineral kinds in FR46). It is not acceptable where the story writer would have to invent the requirement, as in the counting rule for FR47, the fate of an abandoned job in FR51, or the bound behind "forever" in FR49 and FR51.

### Findings
- **high** FR47's counting rule is ambiguous (§F16 FR47) — "A resource counts only once it is in a stockpile" leaves several questions open:
  - Is progress the current stockpiled total, or a cumulative count of deliveries?
  - Do minerals stockpiled before the mineral goal became active count toward it?
  - Can progress go down if a stockpile is removed (M1 FR10/FR18 allow removal) or an item falls out (FR52)?
  - Is "N" the same for all three goals?

  Each answer produces a different scenario test and a different player experience. *Fix:* state the rule in one line, for example "progress is the count of that kind currently in stockpiles; it can go down; items stockpiled before the goal activates count", and give each goal's target.
- **high** FR51's "abandoned" conflicts with the inherited FR8 and does not say what the player sees (§F17 FR51) — M1 FR8 says "A designation that is currently unreachable stays queued and is retried; it is never silently dropped." FR51 says an unreachable job "is abandoned or re-planned instead of being retried forever". The PRD does not say whether an abandoned job drops the designation or returns it to the queue, or whether the player can see it. This is exactly the accept-and-discard shape the project has been burned by before. *Fix:* say explicitly that FR51 supersedes FR8 for this case, and choose one outcome, for example "the designation stays, marked unreachable in both clients, and is retried only after the world changes near it".
- **medium** "Forever" is not a testable bound (§F17 FR49, FR51) — FR49 says dwarves "must not lock each other forever", and FR51 says jobs are not "retried forever". A scenario test cannot assert "not forever". *Fix:* give a tick bound, for example "a head-on meeting in a one-wide tunnel resolves within 50 ticks", and the same for FR51's re-plan or abandon decision.
- **medium** FR43's default professions do not guarantee a playable fresh world, and it contradicts §Out of scope (§F15 FR43) — "new dwarves start with a seeded default profession, so a fresh world is playable before any assignment." There are two problems:
  - A seeded draw over five dwarves can produce no woodcutter, which blocks the wood goal until the player reassigns. "Playable" therefore depends on luck with the seed.
  - "New dwarves" conflicts with "No new dwarves. The count stays five."

  *Fix:* "each of the five dwarves starts with a default profession; every profession is held by at least one dwarf".
- **medium** FR45's escape clause contradicts Success criterion 1 (§F15 FR45, §Success criteria 1) — FR45 allows an animation to "ship plain or [be] parked" after two unconverged rounds. SC1 requires dig, haul and cut "with each animation readable". A parked animation would fail the milestone on a path the PRD itself permits. "Unconverged" is also undefined. *Fix:* reword SC1 to "each work is visibly distinguishable (animated, or plain if FR45's hard stop fired)", or add parked animations to the cut order.
- **medium** FR52 leaves out dwarves and conflicts with FR49 and FR50 (§F17 FR52) — the rule is "items fall to the next supporting surface, and trees fall or are removed". There are three gaps:
  - It says nothing about a *dwarf* standing on a block that is channelled out.
  - A falling item can land on a light emitter's cell, which FR50 forbids.
  - It can land on a tile where a dwarf stands, which interacts with FR49.

  "The simplest rule that reads right" is subjective, although placing it in the story is honest. *Fix:* add dwarves to the scope (or explicitly exclude them) and one line on where an item lands when its landing cell is an emitter.
- **low** FR50 does not say how a rejected placement behaves (§F17 FR50) — "Stockpile placement … never put items on a light emitter's cell". Does placing a stockpile over the campfire fail visibly, or quietly leave that cell out? The sim's silent-filter history makes this worth one clause. *Fix:* "placement over an emitter cell excludes that cell and the client shows the gap", or "is rejected with a visible reason".

## Scope honesty — adequate
§Out of scope does real work: "Resources are not *used*", "A profession is a label that filters jobs", "No new dwarves", and the parked look issues are listed by number. The cut order is an honest plan for de-scoping. Several choices are openly deferred to the story ("tuned in the story", "chosen in the story"), which suits a hobby milestone. The PRD carries no `[ASSUMPTION]` tags and no index, although M1 and M2 both had them. The lowercase sentence starts in FR42, FR43, FR46, FR52 and NFR10 look like tags or lead-ins removed during editing, and they leave unmarked inferences.

The real failure is the inheritance claim.

### Findings
- **high** The "Stand unchanged" and "only reopening" claims are false; five inherited items are overridden without a note (§preamble, §Scope shape "Baseline") — the preamble says the M1/M2 PRDs "all stand unchanged". The Baseline says "The only reopening of M1/M2's non-goals is gems and minerals". M3 in fact overrides these:
  - **M1 FR5** — idle dwarves "claim the oldest unclaimed job (FIFO)" becomes profession-filtered claiming (FR42).
  - **M1 FR8** — "never silently dropped" becomes "abandoned" (FR51).
  - **M1 FR12** — "No materials system" becomes wood, mineral and gem item kinds (FR41, FR46).
  - **M1 FR22** — TUI glyphs "colored by current job/profession" becomes per-dwarf "distinct colouring" (FR38). The PRD does not say which colour the TUI glyph now carries.
  - **M1 non-goals** — "nothing melts, freezes, or falls" and "No … production beyond dig → stone → stockpile" become falling (FR52) and wood production (FR41).

  A downstream agent that trusts the preamble will implement against contradictory rules. *Fix:* add a short "Supersedes" list under Scope shape naming each inherited ID or non-goal and its M3 replacement, and correct the Baseline sentence.
- **low** Inferences are not tagged, and the rewording left fragments (§FR42, FR43, FR46, FR52, NFR10) — "a small fixed set of kinds (one or two minerals, one gem)." is a fragment. "a dwarf with no work…", "new dwarves start…", "the simplest rule…" and "profession assignment…" all start lowercase mid-paragraph. *Fix:* repair the sentences. Either restore `[ASSUMPTION]` tags with a short index, or state that the light PRD deliberately drops the index.

## Downstream usability — adequate
This PRD feeds straight into epic and story creation, with no UX or architecture pass in between for most of it. That makes FR-level clarity matter more than traceability. The IDs are clean and every issue reference resolves (#132–#137). There is no glossary. Terms drift in ways a story writer will have to guess at.

### Findings
- **medium** The TUI's display duties are underspecified (§NFR10, §F14 FR39) — NFR10 lists "names, new items, goal progress". It leaves out professions, the per-dwarf colouring, trees designated for cutting, and tree removal and falling. FR39 says only "The TUI shows names". Because M1 FR22 tied TUI colour to profession, a story writer cannot tell what the TUI dwarf glyph should look like in M3. *Fix:* extend NFR10's list to "names, profession, new item kinds, tree-cut designations, goal progress" and state the glyph's colour source.
- **low** Terms drift with no glossary — "trade"/"profession"/"job" (Vision and FR42), "cut tree"/"fells"/"cut" (Vision and FR41), and "tile"/"cell"/"block" (FR49, FR50 and FR52 describe the same grid unit three ways). *Fix:* use one noun each ("profession", "cut tree", "tile") or add a three-line glossary.

## Shape fit — strong
This is a solo hobby milestone PRD, and it is shaped like one. It has no user journeys, which is right for a single-operator game whose "user" is Wolf at the seat. Its success criteria are observational where the retro ruled that the seat is the instrument (SC1, SC2) and scenario-test based where the sim is the instrument (SC4, SC5). The "Instrument per kind of work" ruling makes this split explicit, which is unusually good. At roughly two pages, the brevity is justified, and the counter-metric "The PRD stays about two pages" keeps it that way. Brownfield accuracy is the one weak spot, covered under Scope honesty.

## Mechanical notes
- **ID continuity: clean.** M2 ended at FR37, F13 and NFR8, confirmed against `prd-frostvein-2026-08-09/prd.md`. M3 continues at FR38–FR52, F14–F17 and NFR9–NFR10, contiguous with no duplicates.
- **Inherited references resolve.** NFR5 ("no drift") matches M2's NFR5. "Mine crystals" matches M2's out-of-scope wording. The parity rule matches M2's Scope shape. The Visual Target section exists in M2. The "Inherits" line names both folders correctly.
- **Assumptions Index: absent.** There are no inline tags, so nothing needs to roundtrip. See the Scope honesty finding on the stripped fragments.
- **Criterion 4 vs F17:** SC4 lists #132–#135 and the FRs map one to one (FR49↔#133, FR50↔#134, FR51↔#132, FR52↔#135). It resolves.
- **Frontmatter:** `status: draft`. Flip it to final once the high findings are resolved, matching the two inherited PRDs.
