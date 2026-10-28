# M4 PRD — extracted inputs (2026-10-28)

Read-only extraction. Paths are relative to `/workspace/projects/frostvein`. No recommendations.

Abbreviations: **RETRO** = `_bmad-output/implementation-artifacts/epic-12-retro-2026-10-21.md`;
**M3PRD** = `_bmad-output/planning-artifacts/prds/prd-frostvein-2026-09-28/prd.md`;
**PROP** = `_bmad-output/implementation-artifacts/13-2-signoff/proposal.md`;
**SS** = `_bmad-output/implementation-artifacts/sprint-status.yaml`; **EPICS** = `_bmad-output/planning-artifacts/epics.md`.

---

## 1. Epic 12 retro (M3 close) — M4 content

**Outcome** (RETRO:10-12): M3 closed 2026-10-21. 12.13 dropped, FR40 moves to M4 as a PRD input. "M4 starts from a PRD with a
more ambitious goal, written in parallel with a pathfinding spike."

**Goal guidance — §8 Wolf's rulings 2026-10-21** (RETRO:190-196):
- "M4's goal is left to the M4 PRD, and it should be **more ambitious** than M3's. M3 built a playable loop with a direction;
  M4 gives it something to be for. M3's out-of-scope list is the natural menu: using resources, building, needs, threats,
  skills, more dwarves." (RETRO:191)
- FR40 moves to M4: *"it's not that urgent right now"*; round 20's *"have to rethink the model and rig anyway"* argues against
  preparing today's rig (RETRO:192).
- Spike (M3-1) is first M4 work, in parallel with the PRD (RETRO:193).
- **Scale / "more dwarves":** "The PRD feeds the spike its load target. The PRD's first draft names the scale (dwarf count, map
  size, terrain-change rate), and the spike designs for it, not for today's 5-dwarf valley." (RETRO:194)
- "The spike feeds the PRD its movement rules ... The PRD is not final until he has [ruled]." (RETRO:195)
- Strict movement rules stay (RETRO:196; §4.1 RETRO:109).

**§8 critical path** (RETRO:198-203): 1. M3-2 tick bench + scripted scenarios; 2. in parallel: M4 PRD draft (goal + scale
numbers) and M3-1 spike; 3. Wolf rules the spike's proposal; 4. M4 PRD finalized, then the M4 epic; 5. M3-3/M3-4 beside it,
outside any cap. Discovery "movement layer is ad-hoc" becomes an M4 PRD input carried by M3-1 (RETRO:205).

**§8 M4 inputs list** (RETRO:207-212):
- Movement debt: #159 #186 #187 #189 #203 #204 #206 #207.
- Design questions: #199 (what stops a miner digging deeper), #169 (gui mirrors the tui).
- Feel: #179 (FPS swings 20–180), #181 (suction pick-up), #175, #176, #177.
- Parked look issues: #120 #126 #127 #128 #130 #137.
- FR40.

**§4 What was hard** (Wolf's ranking: movement layer, performance, gate; seat findings explicitly NOT on the list) (RETRO:85-87):
- §4.1 movement layer ad-hoc: stuck/caged/looping dwarves in 8 of 13 stories; eight open issues in the family (RETRO:91).
  No reachability kept between ticks; every reachability question is a fresh A*/flood bounded by `MAX_ASTAR_NODES = 50_000`;
  the bound produces #159 (RETRO:94). Plain A* with `BTreeMap` costs, heuristic loops every goal cell per expansion (RETRO:95).
  Four local guards answer one cell each and conflict (RETRO:96-102). Genre leads DF / Factorio / Going Medieval (RETRO:104-107).
- Wolf: *"I still want to keep stricter rules like one-dwarf per tile and blocking items when it is natural .. ofc one option is
  to solve it with animations."* Squeeze-past and clamber-over named as the genre trick (RETRO:109-113).
- §4.2 performance invisible to tests: 12.12 entrance rule 17x tick with tests green; 12.9 spikes 9–17.6 ms vs 5 ms fast4x
  period (#179); 12.10b Nagle (#200) (RETRO:117-122).
- §4.3 gate 3,700–5,000 s, pixel guards 80%; Wolf ruled: optimize guards (M3-3), one full gate per story at PR (M3-4)
  (RETRO:126-133).
- Seat is the feel instrument; a seat finding is the loop working (RETRO:87, 229-231).

**Key insights** (RETRO:182-186): groundwork pays — "Build M4 on them" (animation rail, extension pattern, refusal shape)
(RETRO:182); the sim extension pattern = purpose-named stream, component, wire, save, exhaustive bridge (RETRO:75).

**Action items carried** (RETRO:218-227), current state from `gh`:
| ID | Item | State now |
|---|---|---|
| M3-1 #209 | pathfinding/reachability spike | closed 2026-10-25 (story 13.2 done) |
| M3-2 #210 | scripted scenarios + tick bench | closed 2026-10-23 (story 13.1 done) |
| M3-3 #211 | pixel guards: time then cut (folds #145) | open |
| M3-4 #212 | one full gate per story, at PR | open |
| M3-5 / M3-6 | FR40 move; hygiene | done in retro |
| carried from M2 | #142 push re-gating, #144 vehicle script, #148 records | all open |

Retro readiness: "Blockers into the M4 PRD: None" (RETRO:241).

---

## 2. M3 PRD (parent) — `prd-frostvein-2026-09-28/prd.md`

**Inheritance** (M3PRD:10-13): inherits M1/M2 PRDs by reference — four-crate spine, determinism, YAGNI-as-policy, NFR5 (no drift),
the parity rule, M2's Visual Target.

**Vision / goal** (M3PRD:17-25): "Milestone 2 made the valley worth looking at. Milestone 3 makes it worth *playing*: a more
immersive game with more to do, and a reason to do it." Done sentence: "Milestone 3 is done when Wolf can start a fresh world,
tell his dwarves apart, give them their trades, and watch them dig, haul and cut *at the same time* until the tutorial chain
completes. Play then carries on." (M3PRD:23-25)

**Scope shape** (M3PRD:29-37): one M3 epic growing by stories; sim judged by deterministic scenario tests, look/feel at the seat
early; baseline "today's five dwarves and today's worldgen".

**What M3 changes** (M3PRD:39-55): FR5 FIFO per profession; FR12 items gain kinds (no quality/stacking/containers); M1 non-goal
"no production beyond dig→stone→stockpile" reopened for collecting only ("Using them stays out"); "nothing falls" reopened for
items/trees only; gems allowed; parity rule narrowed (NFR10); goals over the wire as typed data.

**FRs, one line each** (M3PRD:62-125):
- FR38 — every dwarf has a seeded name + distinct colouring, in world state, wire, save (M3PRD:64).
- FR39 — dwarves distinguishable at a glance in Bevy client; selected dwarf sharp on zoom (#136); TUI shows names (M3PRD:68).
- FR40 *(stretch)* — dwarf model prepared for per-dwarf variation beyond colour; preparation only, no new variants, first cut (M3PRD:70).
- FR41 — three jobs: dig, haul, cut tree; cut removes whole tree, yields wood at base; dig no longer takes trees (M3PRD:76).
- FR42 — one profession per dwarf (miner/hauler/woodcutter), takes only its trade's jobs; idles otherwise (M3PRD:79).
- FR43 — player assigns/changes profession in Bevy client; seeded defaults include ≥1 of each (M3PRD:83).
- FR44 — designate trees for cutting in Bevy client (M3PRD:87).
- FR45 — dig, haul, cut animations driven by sim state; art hard stop M2-24 (M3PRD:89).
- FR46 — worldgen places minerals and gems in rock; small fixed set (1–2 minerals, 1 gem) (M3PRD:97).
- FR47 — chained tutorial goals in the sim: N wood → N minerals → N gems; stored count; hardcoded targets (M3PRD:102).
- FR48 — both clients show current goal/progress and completion; no fail state, play continues (M3PRD:105).
- FR49 (#133) — at most one dwarf per tile; head-on in one-wide tunnel resolves in bounded ticks (M3PRD:111).
- FR50 (#134) — no item left on/inside a light emitter (M3PRD:116).
- FR51 (#132) — no dwarf stays stuck after digging; unreachable job released in bounded ticks, designation retried (M3PRD:118).
- FR52 (#135) — nothing floats above a dug-out block; items fall, trees fall or are removed (M3PRD:122).

**NFRs** (M3PRD:129-137):
- NFR9 — Determinism holds: identity, professions, resources, goals, one-per-tile movement in sim state; seed + command log =
  identical state; scenario tests cover each.
- NFR10 — Parity rule narrowed: every sim change reaches the TUI as display; profession assignment and tree designation are
  Bevy-only commands; TUI gains no new input.
- NFR11 — Refuse loudly: a refused command is visible to the player, not silently discarded.

**Out of scope — Milestone 3 (verbatim, M3PRD:139-149)** — the suggested M4 menu:

> M1 and M2's non-goals stand, except where **What M3 changes** reopens them. Additionally:
>
> - Resources are not *used*. There is no crafting, building or trading with wood, minerals or gems.
>   They are collected and counted.
> - No fail states, threats, needs, moods or combat.
> - No per-dwarf skills, levels or experience. A profession is a label that filters jobs.
> - No new dwarves. The count stays five.
> - No cave-ins and no unsupported-terrain physics.
> - The look issues stay parked until one bites (#120, #126, #127, #128, #130, #137).

**Success criteria / counter-metrics** (M3PRD:151-170): 5 criteria (seat concurrent work, name the five, chain completes, four
defects close red-first, gate green + determinism). Counter-metrics: 10–14 stories soft cap with named cut order; look work not
back-loaded; no TUI regression; "The PRD stays about two pages." (M3PRD:170).

---

## 3. FR40 / Story 12.13 "Ready for More Faces"

- **What:** "The dwarf model is prepared for per-dwarf variation beyond colour. This is preparation only. It ships no new
  variants, and it is the first item cut under cap pressure." (M3PRD:70-72; EPICS:103). Story: "As the boss, I want the dwarf
  model prepared for per-dwarf variation beyond colour, So that later milestones can give each dwarf a look of their own without
  redoing the model." ACs: prepared, ships **no new variants**; art is Wolf's hands or a watched seat under the hard stop (M2-24);
  first cut under cap pressure (EPICS:2795-2815).
- **Why dropped:** Wolf at the retro 2026-10-21: *"move it to M4 .. it's not that urgent right now"* (SS:2727-2729); round 20's
  *"have to rethink the model and rig anyway"* argues against preparing today's rig (EPICS:2797-2800; RETRO:192). Epic 12 closed
  with 13 stories (EPICS:2799).
- **M4 status:** "FR40 is an M4 PRD input, not a cancelled requirement." (EPICS:2798; SS:2729). Epic 13 header: "FR40 (12.13,
  dropped from M3) is an M4 PRD input." (EPICS:481). Board: `12-13-ready-for-more-faces: dropped` (SS:2727).
- No separate 12-13 story file exists (only the epics.md entry and the board line).

---

## 4. Story 13.2 spike — Scale, Recommendation, Ruling (PROP)

**Setup** (PROP:128-133): `scale.sh`, 2,000 ticks/point, `DEFAULT_SEED`, 8 dig + 1 cut mark per dwarf, mine and pile in opposite
quadrants so hauls cross the map. Stack S = `4f57027` + census + counters + scale patches. Single runs (`SCALE_RUNS=1`) under
concurrent load. 256² is cheaper per tick than 128² at several crews because in 2,000 ticks nothing reaches the pile (longer walk)
— it measures map-spanning searches, not a full pile (PROP:217-219; `scale-table.md`).

**Tick budget: 100 ms (10 ticks/s)** (PROP:211).

**Tick cost table** — base vs pile-memo; mean from PROP:137-148, p99/max from raw `scale-r1-base.txt` lines 9-54 and
`scale-r1-pile-memo.txt` lines 10-55 (1 run each, ms):

| map | crew | base mean | base p99 | base max | pile-memo mean | pile-memo p99 | pile-memo max |
|---|---:|---:|---:|---:|---:|---:|---:|
| 128² | 5 | 0.592 | 10.10 | 18.7 | 0.191 | 6.23 | 11.3 |
| 128² | 20 | 5.71 | 22.35 | 35.5 | 0.942 | 11.05 | 29.8 |
| 128² | 50 | 30.8 | 100.8 | 119.8 | 2.75 | 15.75 | 33.1 |
| 128² | 100 | 95.5 | 277.1 | 304.0 | 7.23 | 36.47 | 57.2 |
| 128² | 200 | 313 | 929.7 | 1022.8 | 20.8 | 97.09 | 134.1 |
| 256² | 5 | 0.244 | 0.51 | 32.1 | 0.179 | 0.21 | 33.3 |
| 256² | 20 | 1.06 | 5.18 | 64.0 | 0.380 | 1.29 | 61.0 |
| 256² | 50 | 9.8 | 70.4 | 96.2 | 2.99 | 36.95 | 69.3 |
| 256² | 100 | 23.1 | 150.1 | 299.8 | 5.69 | 71.31 | 241.6 |
| 256² | 200 | 34.1 | 241.7 | 307.5 | 8.45 | 64.78 | 110.9 |

(3-run base means/p99 in `scale-table.md` agree within load noise, e.g. 128² crew 200 mean 309, p99 971.)

**Readings**:
- Base passes the budget at crew 50 on 128² (31 ms), reaches it at crew 100 (96 ms), 3x over at crew 200 (313 ms). No movement
  option moves that; pile-memo (every count equal to base) reads 20.8 ms, 15x cheaper; state hash equal at all ten points
  (PROP:211-216).
- Tail does not fit: pile-memo 128² crew 200 p99 97.1, max 134 ms; later runs p99 85.4 / max 116.7 with 1,030 of 2,000 ticks over
  10 ms, and p99 141.6 / max 189.6 / 1,081 over 10 ms under heavier load. "So the slowest 1% of ticks sit at the budget, not a
  fifth of it." (PROP:374-378).
- After pile-memo, **A\* becomes the next cost at scale**: 24% at 128² crew 200, 67% / 40% at 256² crews 100 / 200 (27%, 70%,
  54% of system time alone). "A\* storage is not worth a story on 128², but it is the first thing to measure if M4 goes to 256²
  with hundreds of dwarves." (PROP:121-124).
- `pile_targets` is 69–94% of the tick in every scenario over 0.1 ms and at 128² crews 50/100/200 (PROP:100-104, 357-358).
- On main after 13.3: board records "scale crew200 273.3 -> 16.8 ms mean" (SS:2769) and creation probe "128^2 crew 200 640 ->
  31.7 ms mean" (SS:2767).

**#159 (A\* node budget on 256²)**:
- Camp walkable component: 16,063 cells on 128² vs **64,300 on 256²** against `MAX_ASTAR_NODES = 50,000` (1.29x)
  (`scale-table.md` § #159; PROP:220-221).
- Budget-exhausted A\* calls: 0 at every 128² point; 256² crews 5/20/50/100/200 = 10/10/131/218/138 (PROP:182-193).
- Most are "reachable goals farther than the budget, not unreachable ones. A connectivity map cannot fix that; a budget that
  scales with the component, or a better heuristic (survey lead 3), can." (PROP:222-223).
- Recommendation item 4: "**#159 waits for the M4 map size.** If M4 goes past 128², its story sizes the A\* budget to the walkable
  component, or takes FFF #317's chunk-component heuristic" (PROP:393-395).
- ADR 5 amended (Ruling 4, PROP:421-422; `docs/technical-preferences.md:42-44`): plain A\* with a fixed node budget holds while the
  walkable component is under `MAX_ASTAR_NODES`; past that, a budget sized to the component or a better heuristic, not a hierarchy.

**Recommendation summary** (PROP:356-398): tick cost is not pathfinding; the eight cage issues are not one movement problem —
#206/#189 dwarf-blocks-dwarf, #189/#207 stone-blocks-dwarf, four not movement (#186 wander, #187 job refusal, #203 dig, #204 stone
placement), #159 A\* budget on a bigger map. Options in order: 13.3 pile-memo; 13.4 squeeze-past; 13.5 clamber-over only if ruled;
no connectivity map now; the four non-movement issues go to their own stories outside the movement layer (PROP:396-398).

**Ruling, Wolf 2026-10-23** (PROP:410-422): order as recommended; #159 waits for M4 map size; #186/#187/#203/#204 to their own
stories; Ruling 1 swap allowed (squeeze B); Ruling 2 "decide after squeeze" (#207 re-measured on top of 13.4, clamber ruled then);
ADR 5 amended.

**What remains of the movement layer**:
- 13.3 pile-memo: **done**, merged PR #217 (c5164d93), closing no issue (SS:2764-2775).
- 13.4 squeeze-past: **done**, merged 2026-10-28 PR #221 (0a8b4462), closing no issue; #187 fix folded in; idle-blocker swap
  ruled in; seat Q2 → #219; 16k cage runs pinned → #218; post-review seat card Q4 not recorded before merge (SS:2776-2799).
  13.4 probe: the idle-blocker swap moves #207 13 → 18 of 24 (SS:2779).
- **13.5 clamber-over: `backlog`**, "only if ruled (13.2 Ruling 2): #207 is re-measured on top of 13.4 first; the story may be
  dropped" (SS:2800-2801). Story text: basis is `prototypes/clamber.patch` (step cost 4), to be ported; ACs: lands or is dropped
  with ruling recorded; emitters stay blocking, no writer puts a stone on a dwarf, gui draws a dwarf standing on a stone
  (EPICS:2915-2929). Spike costs for clamber: #207 24/24, A\* nodes −37% to −69% in 8 of 10 scenarios, #203 replay worse 11 → 14
  marks, gui must draw dwarf on stone (PROP:386-389).
- **#207 open.** #189 and #206 are closed (2026-10-25, before 13.4 landed; memory `closing-keyword-closes-whole-issue.md` records
  PR #216 prose closing them). `epic-13-retrospective: optional` (SS:2802).

---

## 5. Current hardcoded limits on main (relevant to more dwarves / jobs)

All in `crates/sim-core/src/lib.rs` unless noted.

- **World size** `Dims::DEFAULT = 128 × 128 × 32` (lib.rs:141-145). `World::generate` doc: "Only `Dims::DEFAULT`-scale worlds are
  supported" (lib.rs:2860-2865). No chunking.
- **Crew = 5, fixed by type**: `spawn_dwarves(..., identities: [Identity; 5], professions: [Profession; 5])` (lib.rs:3598-3604).
  Professions seeded as fixed array `[Miner, Miner, Hauler, Hauler, Woodcutter]`, shuffled (lib.rs:2911-2919). Wander cooldown
  stagger note: "an eleventh dwarf would share dwarf 0's phase — harmless at five" (lib.rs:3626-3628).
- **Identity pools**: `DwarfName::ALL` = 16 names (lib.rs:192-230); `DwarfColour::ALL` = 5 colours (Red, Gold, Green, Blue,
  Purple) (lib.rs:233-243). Identity = name + colour, shuffled from `STREAM_IDENTITY` (lib.rs:2900-2908).
- **Camp**: `CAMP_RADIUS: u32 = 3` (`crates/sim-core/src/worldgen.rs:7`) → a 7x7 flat clearing required ("worldgen requires one
  7x7 flat camp clearing", worldgen.rs:364); spawn candidates are standable cells in that square minus emitters (lib.rs:3609-3621).
  `camp_emitters` returns 5 fixed emitters (lib.rs:3660); `TORCH_RING_OFFSET = 8` (lib.rs:3658).
- **Other limits**: `MAX_ASTAR_NODES = 50_000` (lib.rs:50); `MAX_DESIGNATIONS = 4096` (lib.rs:49); `RETRY_COOLDOWN = 20`
  (lib.rs:59); `WORK_TICKS = 5`, `DIG_WORK_TICKS = 50`, `CUT_WORK_TICKS = 50` (lib.rs:52-58); `SQUEEZE_WAIT_TICKS` = one step
  period (lib.rs:48); `GOALS` = Wood 10 → Tetrahedrite 10 → Bloodstone 1 (lib.rs:62-66).
- **Enums that exist today**:
  - `JobKind { Dig, Channel, Cut, Haul { item } }` (lib.rs:277-286); `trade()` maps Dig/Channel → Miner, Cut → Woodcutter,
    Haul → Hauler, exhaustive (lib.rs:539-546). Wire: `DwarfJob { Dig, Channel, Cut, Haul }` (`crates/protocol/src/lib.rs:109`).
  - `Profession { Miner, Hauler, Woodcutter }` (lib.rs:253-258).
  - `ItemKind { Stone, Wood, Tetrahedrite, Bloodstone }` (lib.rs:156-161); `yield_of` exhaustive (lib.rs:166-178).
  - `Material { Stone, Soil, Ice, Snow, TreeTrunk, TreeFoliage, Tetrahedrite, Bloodstone }` (lib.rs:78-87).
  - `DesignationKind { Dig, Channel, Cut }` (lib.rs:105-109).
  - `SimCommand { Designate, CancelDesignation, PlaceStockpile, RemoveStockpile, SetProfession }` (lib.rs:118-124).
  - `Refusal { PlaceStockpile, SetProfession, Designate }` (lib.rs:127-131).
  - `LightKind { Torch, Campfire, Lantern }` (lib.rs:185-189); `JobState { Idle, Walk, Work }` (lib.rs:267-271).
  - Ore constants hardcoded: one vein per 360 interior cells, 4–12 cells; 1 bloodstone per 20 tetrahedrite; gems ≥ depth 5
    (worldgen.rs:10-18).
- **Scale is not on main**: no `generate_scaled` / `crew` / `dims` in `crates/` (checked with `rg`).

**`prototypes/scale.patch` (13.2, throwaway, 396 lines)** — what it changes:
- sim-core: adds `World::generate_scaled(seed, dims, crew)`; `generate` calls it with crew 5 (byte-for-byte today).
  `spawn_dwarves` gains `crew`; if the 7x7 camp ring has fewer standable cells than `crew` (~48), it takes the `crew` walkable
  cells nearest the camp by BFS over `astar_neighbours` (ramps included), asserting enough ground. Identities and professions
  **cycle past five** (`i % 5`), so a big crew repeats names/colours/professions.
- sim-core test module `scale_gen` (ignored test `scale_generate`): writes the ten `scale/*.json` scenarios (128² and 256² ×
  crews 5/20/50/100/200), 8 dig + 1 cut mark per dwarf near (edge/4, edge/4), a pile of one cell per mark near (3·edge/4, 3·edge/4),
  and prints the camp's walkable-component size (#159).
- simd `scenario.rs`: the `start` block may add `dims: [x,y,z]` and `crew` to a seed start (defaults `Dims::DEFAULT`, 5).
- Inert on the committed scenarios (hashes equal C0) (PROP:10).

---

## 6. Project brief + technical preferences — rules a PRD must respect

Brief (`docs/project-brief.md`, M1-era; still says "Unreal client" — Bevy replaced it; #148 open to update the brief):
- Vision: "A Dwarf Fortress–inspired voxel colony simulation, built as a headless simulation daemon with thin display clients ...
  The sim must be playable and enjoyable early, and remain fully testable headless." (brief:4-7)
- "Solo hobby project developed primarily by AI agents ... Restraint and shipped increments are valued over completeness and
  generality." (brief:9-11)
- Roadmap sketch (context only): "needs/moods, more jobs and items, larger maps with chunking, hierarchical pathfinding,
  FlatBuffers protocol, interest management" (brief:72-75).
- Identity: seed-derived, palette swaps on shared geometry, no per-creature assets (brief:92-95).

Technical preferences (`docs/technical-preferences.md`):
- Four crates; sim-core pure, no I/O; protocol is the single home of message shapes; clients zero game logic (tp:9-15, 39).
- All randomness from the world seed; scenario tests build → command → tick N → assert (tp:25-28).
- Fixed-timestep 10 ticks/s (tp:32). Protocol: NDJSON, optimization deferred (tp:36-38, 59-60).
- ADR 5 + 13.2 amendment: plain A\*, budget/heuristic before hierarchy (tp:40-44).
- "YAGNI is policy. Build for the current story's needs, not the imagined fortress of 500 dwarves." (tp:51-52)
- No single-implementation abstractions; no config/plugin/event/data-driven content systems before a third concrete use;
  hardcoded constants fine (tp:53-58).
- "No premature performance work. Optimize only after a profile shows a problem in a realistic scenario." (tp:61-62)
- New dependency needs one sentence of justification (tp:63-64).
- Stories: vertical slices; observable end; one dev session; each names and tests its instrument (tp:68-94).
- New work joins the nearest epic; new epic only for a new milestone goal (tp:95-97).
- Live gate early: look/UX to Wolf's seat before review; sim by scenario tests (tp:98-100).
- Reproduce before a story exists (tp:101-103). Art hard stop (tp:104-107).
- "PRD: a few pages, derived from the project brief. No personas, no market analysis, no competitive research." (tp:120-121)
- M3 precedent: "The PRD stays about two pages" (M3PRD:170); one epic per milestone, Epic 13 is the M4 epic and is renamed to the
  PRD's goal (EPICS:481-489; SS:2735-2738).

---

## 7. Open GitHub issues — gameplay / sim features and known sim defects

From `gh issue list --state open --limit 100` (2026-10-28). Tooling, look-only and instrument issues omitted.

**Movement / sim defects**
- #159 — A dwarf in one walkable area over `MAX_ASTAR_NODES` starves a reachable job behind > `RETRY_COOLDOWN` unreachable ones
  (bug; live on 256² only; waits for M4 map size).
- #186 — Idle dwarf stands forever on a one-level island: wander is same-z only (bug).
- #187 — A refused channel completion loops forever: let-go returns to the same refusal (bug; 13.4 Task 0 ruled "#187 fix folded in", SS:2781, issue still open).
- #203 — A dig can take the floor out from under a miner's only exit and seal him in a hole.
- #204 — With the pile full, stones build up round the pit until they seal two miners in a pocket.
- #207 — A two-deep pile round a pocket stalls: haulers fill its last way out and the drop is refused (13.5 clamber's target).
- #218 — Full pile seals pits: channel cut refused forever, or never claimed (16k cage sweep, 4 shapes) (bug, from 13.4).
- #220 — Squeeze timer carries across a change of case: a cancel mid-stand-off swaps on the first look (bug, from 13.4).
- #154 — Two loose stones on one stockpile cell: a channel's stone, and a pile placed over a heap (bug).
- #170 — At the mark cap, a designate that overlaps existing marks drops its new cells silently (bug).
- #214 — An off-map designate/cancel/remove rect is accepted silently instead of refused (bug).

**Design questions / gameplay feel**
- #199 — Decide what should stop a miner digging deeper: gems should not, ore is undecided.
- #219 — Squeeze-past reads as two dwarves walking through each other (no swap animation) (enhancement, from 13.4 seat Q2).
- #181 — Pick-up reads as suction and drop as a slide (carry look after 12.9).
- #176 — Woodcutter walks with the axe in hand, not strapped to the pack.
- #179 — Gui frame rate swings 20–180 FPS with haze off (12.9 seat).
- #169 — gui world is a mirror image of the tui (east/west flip).
- #172 — The tui shows a few trees where the gui shows a forest; nothing says so.
- #102 — Look at the controls overall: discoverability first, slice-on-scroll as a maybe.
- #118 — Seat keymap is out of physically-pressable function keys; scheme needs a rethink.
- #50 — M2-15 scenario-driven gui session mode (the `--at-tick` half is done) (action-item).

**Closed members of the retro's movement list**: #189, #206 (closed 2026-10-25); #209, #210 closed.
