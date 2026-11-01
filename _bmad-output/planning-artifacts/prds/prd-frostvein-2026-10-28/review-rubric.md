# PRD Quality Review — Frostvein Milestone 4: A Living Fortress

Calibration: solo hobby project, one judge (Wolf), technical preferences cap a PRD at "a few
pages, no personas, no market analysis". This review therefore does not ask for personas, journeys,
market or competitive sections, a glossary section or an Assumptions Index. It judges whether
story creation for Epic 13 can work from the PRD without inventing rules.

## Overall verdict

The PRD is honest and tightly scoped. **What M4 changes** names every inherited rule it reopens,
the cut order is Wolf's and explicit, the perf bar has numbers, and every FR traces to a step in the
addendum's story sketch. What is at risk is the needs loop, which is the thesis. Nothing in the FRs
owns "the fortress feeds and warms itself without an order". Recovery from neglect can deadlock. One
campfire cannot warm 200 dwarves when only one dwarf may stand on a tile. And the bench scenario's
"real work" is glossed in the same terms that produced 13.2's trap. Fix the six highs before
marking the PRD final. None of them needs more than a sentence or two.

## Decision-readiness — adequate

**What M4 changes** (§ What M4 changes) is the PRD's best section: each reopened rule is quoted,
with its narrowing and the FR it lands in. Trade-offs that Wolf ruled are stated as rulings: the
cut order, the 25 ms / 100 ms bar, and the roster question parked for the seat (FR57). Three real
tensions are left unnamed. Each will come up as a surprise inside a story.

### Findings
- **[medium] FR55's fix pulls against NFR12** (§ F18 FR55, § NFR12). FR55 says "the budget is sized to the walkable area". Today's per-tick bound is idle dwarves × `MAX_ASTAR_NODES` (`lib.rs:584` NOTE). Sizing the budget to a component of 64,300+ cells with 200 dwarves raises the worst-case tick, and the worst case is what NFR12's p99 measures. The addendum puts A* at 40–67% of the 256² tick already. *Fix:* one sentence in FR55. Either the budget-sizing option must show its p99 on the FR54 bench, or the heuristic option is preferred because of NFR12.
- **[medium] Needs change every existing scenario, and no strategy is stated** (§ F21, § Counter-metrics "no 128² five-dwarf scenario changes outcome without a reason on the record"). Once hunger, cold and sleep land, every committed scenario changes: the M3 red-first tests for FR49–FR52, the cage sweeps, the state hashes. Dwarves stop work to eat and sleep, and the campfire can go out. The counter-metric allows this "with a reason", but it does not say whether those scenarios are re-pinned, or whether needs can be disabled per scenario. The second option is a setting, and settings run into YAGNI. The first needs-story will have to decide. *Fix:* state the ruling: re-pin with a reason, or needs off in pre-M4 scenarios.
- **[medium] The 13.5 decision instrument is undefined** (§ Scope shape "decided by the scale run's #207 reading", § SC5). 13.2 Ruling 2 said "#207 re-measured on top of 13.4". The PRD moves this to "the scale run", but #207 is a 24-case pocket-pile sweep (13.4 moved it from 13 to 18 of 24), not a reading the FR54 tick bench produces. Nothing says what result lands clamber and what result drops it. *Fix:* name the instrument (the #207 sweep at 256²? the bench?) and the threshold, or say "Wolf rules on the sweep count".

## Substance over theater — strong

There is no furniture. The Vision is specific to frostvein: lake-ice fishing, the campfire, "dozes
in the snow". The addendum's "Why these jobs" table justifies each job from something that already
exists. The NFRs are product-specific and carry numbers. The PRD is about three pages, as its own
counter-metric asks.

### Findings
- **[low] An adjective comes before the number** (§ NFR12 "stays well inside the 100 ms budget … The bar is a mean of at most 25 ms"). "Well inside" adds nothing once 25 ms is stated, and a reader could take it as a second, looser bar. *Fix:* drop the adjective and lead with the numbers.

## Strategic coherence — adequate

The thesis is clear and well bet: **alive** means the fortress "keeps moving when the player stops
giving orders". It is backed by needs that create demand between trades (addendum: "cold becomes
demand on woodcutters"). SC2 and SC3 validate the thesis rather than measuring activity, and the
counter-metrics are real. The weak point is that the thesis has no owner among the FRs.

### Findings
- **[high] No FR owns the self-sustaining loop the done sentence promises** (§ Vision done sentence "without giving an order while the fortress feeds and warms itself", § SC2, § FR58, § FR61, § FR44 inherited). Several food and fuel sources are order-driven. Forage is on "a designated plant" (FR58). Wood comes from trees, which M3 FR44 makes designation-only. Fishing needs "a fishing hole designated". Stoking and cooking are implied to be automatic only because NFR10's command list leaves them out. No FR says that production keeps up with consumption at 200 dwarves with the seeded profession mix. That balance is the hardest tuning problem in M4, and no story owns it. SC2's "for a stretch" has no length. *Fix:* add one FR (or a clause to FR61/FR62). It should say which jobs generate themselves (stoke, cook, hunt-nearest, and any auto-forage or auto-cut) and that with N standing designations a crew-200 fortress holds for a stretch of X sim minutes without misery. SC2 should then name the stretch.
- **[medium] A counter-metric contradicts the story sketch** (§ Counter-metrics "Misery and the first idle behaviour reach the seat in the first half" vs addendum § Sketch item 8 "Idle life (FR66)", last). Wandering already exists (M3 FR42), so "first idle behaviour" is either already met or means an FR66 behaviour that the sketch schedules last. *Fix:* name which behaviour is meant (e.g., "gathers at the fire", which falls out of FR63 at sketch step 6) or move one FR66 behaviour earlier in the sketch.
- **[low] Cut step 2 leaves animals ambiguous** (§ Counter-metrics "then hunting and cooking together"). If hunting is cut, does FR59's animal life stay, for idle "follows an animal" and for the addendum's "stress pathfinding at scale for real"? *Fix:* say whether animals stay without hunters.

## Done-ness clarity — thin

Several FRs are sharply testable: FR55 (red-first scenario), FR60's loud refusal, FR62's
meal-over-raw preference, NFR9's save → load → tick N equality, and FR57's unique names. Reliance
on seat judgment ("not boring", "visibly") is legitimate at this stakes level, because Wolf is the
named judge and the PRD says so. The weakness is in the edges of the needs loop and in the scale
instrument. These are the parts every F20–F21 story and the first F18 story will need first.

### Findings
- **[high] Recovery from neglect can deadlock, and an unlit fire has no defined behaviour** (§ FR61, § FR63, § FR65, § SC3 "Putting it right restores them"). Here is the sequence. A miserable dwarf "refuses work until relieved". Stoking is work (a haul). Let the fires die, and the haulers turn cold and miserable, so they refuse to stoke. The fires stay out, and nobody recovers. FR63's "a cold dwarf goes to a fire to warm up" is undefined when no fire is lit. FR61 does not say whether stoking relights a dead fire. The player cannot "put it right", because orders are the player's only tool. A permanent misery lock is a fail state, which § Out of scope forbids. *Fix:* state the recovery path. Examples: relief-seeking (eating, warming, sleeping, and maybe stoking) is never refused; a dead fire relights on its first stoke; cold dwarves with no lit fire go underground (FR63 already makes depth warm).
- **[high] One campfire cannot warm 200 dwarves under one-dwarf-per-tile** (§ FR61 "Fires burn wood", § FR63, § FR64 "near a fire if one is close", § Out of scope "no building or construction"). The PRD says "fires" in the plural, but no FR creates a fire. There is no placement command (NFR10 lists none), and building is out of scope. Today's camp ring holds about 48 standable cells (`CAMP_RADIUS` 3). With M3 FR49's one-dwarf-per-tile rule, a warmth radius around a single campfire fits a few dozen dwarves at most. Cold, as specified, cannot be satisfied at 200. *Fix:* rule how many fires exist and how they come to be (worldgen places K, or a Bevy-only "place fire" command that NFR10 lists), and give a warmth radius so FR63 is testable.
- **[high] Crew size as a setting, arrivals, and the default world have conflicting meanings** (§ FR53, § FR54, § FR56, § What M4 changes "the M4 target is 256×256×32", § Done sentence "start a fresh 256² world"). FR53 makes crew size a world setting and needs a spill spawn for "more of them than the camp ring holds". FR56 says arrivals run "until the crew reaches 200", starting with a 5–20 group at embark that always fits the ring. The inferred reading is that the setting is for the bench and arrivals are for play, but the PRD does not say so. It also leaves open whether crew size is the starting count or the cap, whether arrivals run in 128² five-dwarf worlds, whether `Dims::DEFAULT` changes, and how Wolf starts a "fresh 256² world" (simd flag? default?). Separately, FR53's "A 128² five-dwarf world stays reproducible byte for byte" can only hold until FR58/FR59 add plants and animals to worldgen. *Fix:* define crew size = starting crew; arrivals cap = 200, hardcoded, and say in which worlds arrivals run; state the default dims; scope "byte for byte" to FR53's own story.
- **[high] FR54's "real work" has no checkable condition, and its gloss repeats the trap** (§ FR54 "(marks and a pile far enough apart that paths and floods span the map; 13.2's trap)", addendum § Scale data caveat). Marks and a pile far apart is exactly 13.2's setup, and in it "nothing reached the pile within 2,000 ticks". As written, the gloss describes the trap rather than its guard. FR54 also does not say whether the bench scenario takes up later systems (needs, animals, fires) as they land. Without them, SC4's close-time NFR12 pass could come from a fortress that never gets hungry inside the bench window. *Fix:* give an observable condition, e.g. "≥ N hauls delivered and ≥ M jobs completed inside the bench window", with the window length stated, and add "each F20–F21 story extends the bench scenario so its system runs inside the window".
- **[high] NFR12 has a bar but no measurement protocol** (§ NFR12, § SC4, § Counter-metrics "no game story merges without its bench delta"). The PRD does not give the run count, the tick window, the build profile, or the load. It also does not say whether the bar gates each story or only the close: "stays" implies each story, while SC4 says "at the close". This matters because p99 is noisy. 13.2's own data has the 128² crew-200 pile-memo p99 at 85.4, 97.1 and 141.6 ms on different runs under different load. A bar of p99 < 100 ms judged on one run can flip with load alone. *Fix:* fix the protocol (e.g., release build, idle devpod, 3 runs, median of p99s, N ticks) and say whether a mid-milestone breach blocks a merge or only gets recorded.
- **[medium] Need priority and interruption rules are not specified** (§ FR62–FR65). When a dwarf is hungry and cold, which need wins? Does a sleeping dwarf that gets cold (FR64: it "keeps losing warmth") wake? Does a miserable dwarf still seek relief? Is eating "work" for FR65's refusal? These are sim rules under NFR9's determinism, and each story will otherwise invent its own. *Fix:* one ordering sentence (e.g., cold > hunger > sleep, and relief-seeking is never refused), tuned in the story.
- **[medium] SC1's "in one sitting" is undefined, and it binds the arrival cadence** (§ SC1, § FR56 "The cadence is a hardcoded constant tuned in the story"). The cadence must deliver about 10–40 groups to reach 200 within a sitting. The PRD gives no wall-clock length and does not say whether fast-forward is allowed. *Fix:* give a sitting length and sim speed (e.g., "≤ 60 min at fast4x"), which sets the cadence constant's range.
- **[medium] NFR13 does not restate the bars, and the venue is unclear** (§ NFR13 "M2's NFR6 frame-rate bars hold"). The bars (60 fps at working zoom, ≥ 30 fps at full vista, `docs/architecture.md:127`) were set for a 128² world. On 256², "full vista" covers four times the area, and the PRD does not say whether it means the whole map. Open issue #179 (frame rate swings 20–180 fps at five dwarves) is not mentioned, though it makes a miss likely before any M4 load. *Fix:* restate the numbers, name the camera framing on 256², and name #179 as a known risk.
- **[low] "Playable 256² valley" is undefined** (§ FR53). FR60 needs a lake, FR58 needs surface plants, and FR56 needs reachable valley edges. *Fix:* list what "playable" requires: camp clearing, lake, trees, walkable edge.
- **[low] Adjectives in FR60** (§ FR60 "a slow, steady rate", "lake ice"). The rate is fine as a tuned constant. Whether any of the seven ice patches counts as "lake ice", or only the lake, decides FR60's refusal test. *Fix:* say which.
- **[low] "A working mix" is vaguer than M3's guarantee** (§ FR56 "seeded towards a working mix"). M3 FR43 guaranteed at least one of each profession. *Fix:* state the floor per arrival group, or for the crew.

## Scope honesty — adequate

§ Out of scope is explicit and does real work: no death, no beds, no shelters, no threats, FR40
parked with Wolf's reason, chunking only on evidence. The cut order is ruled. Open items are few
and real (the roster is judged at the seat). Three omissions hide stories, and one exclusion
contradicts an NFR.

### Findings
- **[medium] Art and animation scope is not stated for the new jobs, sleep and misery** (§ FR65 "posture or animation", § FR64 "lies down", § FR58–FR61; the M2-24 hard stop is cited only in FR66). M3 FR45 required animations for each job. M4 does not say whether forage, hunt, fish, cook and stoke need one, or whether they ship plain. FR65's visible misery and FR64's lying down are art work, and SC3 depends on "visibly". These are hidden stories inside the cap, with no fallback if the art loop is parked. *Fix:* one line: new jobs ship plain unless Wolf asks; misery and sleep get a pose under the M2-24 stop, with a named fallback (e.g., a tint or marker driven by sim state).
- **[medium] Cap accounting omits defect stories, 13.5 and art rounds** (§ Counter-metrics "12–16 new stories. 13.1–13.4 do not count", addendum § Movement debt). The addendum expects #186, #203, #204, #207, #218 and #220 to "enter as defect stories when the 256² crew-200 scenario reproduces them". It does not say whether those stories, or 13.5, count against the cap. M3 said so for its tooling items. *Fix:* state whether they count.
- **[medium] "No protocol change" contradicts NFR10** (§ Out of scope "No chunking, interest management or protocol change unless…", § NFR10 "Every new sim state reaches the TUI as display"). New wire fields are a protocol change, and the M4 FRs need them. A literal-minded story agent would treat them as out of scope. *Fix:* "no protocol *format or transport* change (compression, batching, FlatBuffers)".
- **[low] Torch fuel is unstated** (§ FR61 "Fires burn wood", § FR63 "Torches give light, not warmth"). Do torches burn wood? FR63 rules out warmth, but not fuel. *Fix:* say "torches stay always lit" or say otherwise.
- **[low] Save compatibility is unstated** (§ FR57's combined names, § F20–F21 new state). Names move from `DwarfName::ALL` to combinations, and new saved state appears. The PRD does not say whether M3 saves load, or are refused loudly (NFR11). The memory record shows serde `Option` fields never refuse an old save. *Fix:* one line: M3 saves are refused loudly.

## Downstream usability — adequate

This PRD feeds Epic 13's story creation directly, so traceability matters. The FR and F IDs are
contiguous, every cross-reference resolves, and the addendum's sketch covers every FR. There is no
glossary, which is acceptable at this length, but a few nouns drift in ways a story agent could
misread.

### Findings
- **[medium] FR61 and FR59 each hide two or more stories** (§ FR61; addendum § Sketch item 6 "FR61's fuel half" and item 7 "FR61's meal half"). The addendum already splits FR61 into halves that land in different stories at different times, which shows the FR should be two. FR59 bundles animal life and edge spawning with the hunter job and carcass hauling. *Fix:* split FR61 into fuel/stoke and cook, and consider splitting FR59 into animals and hunting. Story ACs can then cite single FRs.
- **[low] Glossary drift** (throughout). "Pile" and "stockpile" (M3's term) are used interchangeably. So are "trade" and "profession", and "fire", "fires" and "campfire". "Work" is ambiguous for FR65: does eating, warming or stoking count? *Fix:* pick one term each, and define "work" in FR65.

## Shape fit — strong

The shape is right for a solo hobby milestone. It is a capability spec with no personas or
journeys, Wolf is the single judge named where seat judgment is the test, and the detail sits in
the addendum. It neither over- nor under-formalizes. No findings.

## Mechanical notes

- **IDs:** FR53–FR66 are contiguous and unique; F18–F21 are contiguous. Every FR referenced in § What M4 changes (FR53, FR56, FR57, FR61, FR62–FR66) and § NFR13 (FR54) resolves to the matching content.
- **Reused NFR IDs:** NFR9, NFR10 and NFR11 are M3's IDs, redefined here with M4 content. NFR9 says "extended", NFR10 says "as narrowed", NFR11 has no marker. A story citing "NFR10" is ambiguous between the M3 and M4 texts. Mark all three "(M4 extension)", or cite them as M4-NFR10.
- **Undefined in the PRD body:** "13.2's trap" (FR54) is explained only in the addendum caveat, and "M2-24" (FR66) is a retro ID. Both are acceptable, but the PRD should not read alone without the addendum.
- **Code facts checked against `main`:** `Dims::DEFAULT` 128×128×32 (`lib.rs:141`), `MAX_ASTAR_NODES` 50,000 (`lib.rs:50`) and `CAMP_RADIUS` 3 (`worldgen.rs:7`) all match the addendum. The scale table and the "8.5 ms mean / 65 ms p99" figure match 13.2's raw numbers (8.45 / 64.78).
- **Assumptions Index:** there are no `[ASSUMPTION]` tags and no index. That is acceptable at this stakes level. FR57's roster question is the only open item and is assigned to Wolf.
- **Status:** `draft`. Per epics.md, the PRD can become final now that 13.2 is ruled. The header's "Epic 13 takes this name when the PRD is final" agrees with epics.md:483.
