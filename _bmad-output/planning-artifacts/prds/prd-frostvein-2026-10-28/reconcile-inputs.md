# M4 PRD: input reconciliation (2026-10-28)

Read-only review. Snapshot: `prd.md`, `addendum.md` and `.memlog.md` as of 10:52:10 (they changed
mid-review; line numbers are from that version, which carries memlog lines 31-33).

Abbreviations. **PRD** = `prd.md`, **ADD** = `addendum.md`, **LOG** = `.memlog.md` (all in this dir).
**RETRO** = `_bmad-output/implementation-artifacts/epic-12-retro-2026-10-21.md`.
**PROP** = `_bmad-output/implementation-artifacts/13-2-signoff/proposal.md` (raw: `scale-r1-*.txt`,
`scale-hash-pile-memo.txt`, `timer-pile-memo.txt` beside it).
**M3** = `_bmad-output/planning-artifacts/prds/prd-frostvein-2026-09-28/prd.md`; **M1** / **M2** = `-08-01` / `-08-09`.
**TP** = `docs/technical-preferences.md`. **EXT** = `extract-inputs.md` (this dir).

Tags: **GAP** (input content dropped), **CONTRA** (the docs disagree with the input or each other),
**OK** (checked, matches).

---

## 1. `.memlog.md` (decision log)

| # | Tag | Input | PRD / ADD | Finding | Fix |
|---|---|---|---|---|---|
| 1.1 | GAP (tone) | LOG:9 "Idle dwarves wander and do **stupid things**" | PRD:134-136 "small, visible things"; PRD:25-26 | Wolf's comic, slightly foolish intent is softened to "small, visible things". The quoted bar ("not boring") survives, but "stupid things" does not | Keep his words in FR66: idle dwarves "do small, stupid things" (blunder, dither, nap in odd places) |
| 1.2 | GAP (tone) | LOG:7 "Frostvein stays its own game (own systems, **icy-grim identity**)" | PRD:28-30 | Own systems is kept; the icy, grim tone is not stated anywhere | Add one clause to the Vision: the new systems keep the icy, grim feel (cold is the enemy, the fire is the life) |
| 1.3 | CONTRA | LOG:14 "warming at **the campfire** is the relief" | PRD:35, 116-117, 125-127, 176 "fires", "a lit fire", "the fires out of wood" | The camp has one campfire (`camp_emitters`, `crates/sim-core/src/lib.rs:3660-3662`), torches give no warmth (LOG:33), and building is out (PRD:163). Nothing in M4 creates a second fire, but the PRD says "fires" throughout | Either say "the campfire" (singular) everywhere, or add an FR naming where more fires come from (worldgen, the camp ring) |
| 1.4 | GAP | LOG:14 + LOG:8 (200 dwarves, one campfire) | PRD:125-129 | 200 dwarves warming and sleeping "near a fire" at one 1-cell campfire under one-dwarf-per-tile (M3 FR49) is a capacity question nobody has stated. A warm radius decides whether cold is reliefable at 200 at all | State in FR63 that warmth is a radius round the fire, and add "200 dwarves can warm at once" to FR54's scenario checks |
| 1.5 | GAP | LOG:24 "woodcutters forage; new trades hunter, fisher, cook" | PRD:116-117, 121; ADD:12 | No trade is named for stoking. "A haul of wood" implies haulers, but ADD:12 says cold becomes "demand on woodcutters" (that means cutting, not stoking) | FR61: "stoking is a hauler job" (or whichever trade Wolf names) |
| 1.6 | OK | LOG:31 terrain-change rate: no numeric target | PRD:89-90 | Reflected. See 2.1 for the instrument gap |
| 1.7 | OK | LOG:32 crew 200, no fallback floor; long-term ambition larger | PRD:152-153; ADD:69-74 | Reflected. This supersedes LOG:8's "at least 100" |
| 1.8 | OK | LOG:33 torches give no warmth | PRD:127 |
| 1.9 | OK | LOG:13 (five jobs), LOG:15-17, LOG:22-29 (A1-A8) | PRD:44-47, 61, 91-93, 97-104, 108-137, 150, 186-189 | All folded in; FR numbering matches LOG:30, superseding LOG:27's "FR66" |
| 1.10 | OK | LOG:6, 11, 18-21, 30 | — | Process-only (events and changes) |
| 1.11 | OK | LOG:12 genre is the reference, not a copy | PRD:28 |

**Note (not a gap):** the cut order LOG:28 / PRD:187-189 removes FR66's extras first. Then success
criterion 2 ("not boring", PRD:175) rests on need-driven motion plus today's wander. Today's wander is
same-z only (#186, see 3.4). That is Wolf's ruling, but the PRD should say SC2 is still judged after
the cut.

## 2. Epic 12 retro §4 and §8

| # | Tag | Input | PRD / ADD | Finding | Fix |
|---|---|---|---|---|---|
| 2.1 | GAP | RETRO:194 scale = dwarf count, map size, **terrain-change rate**; RETRO:94 (nothing is cached because terrain changes) | PRD:89-90 "the bench measures it" | The rate is now ruled "no target" (LOG:31), but no instrument backs the claim. `simd --bench` prints ms, `over10`, the state hash and marks/items/zones/goals only (`crates/simd/src/scenario.rs:319-340`). Nothing counts changed cells. M4 also adds a **sustained** source: fuel demand means continuous tree cuts, on top of foraging and fishing holes | FR54: the bench end line reports terrain cells changed (digs + cuts) per 1,000 ticks. Name fuel-driven cutting as the new steady-state source |
| 2.2 | CONTRA | RETRO:195 "The PRD is not final until he has [ruled squeeze-past and clamber-over]" | PRD:52, 180; ADD:59-60 | Squeeze is ruled and landed. Clamber is deferred into M4 (PROP:420, "decide after squeeze"), so the retro's precondition for finalising is not met as written | Add one line: the 2026-10-23 Ruling 2 replaces the retro's precondition, and clamber is ruled inside M4 (cite PROP:420) |
| 2.3 | GAP | RETRO:109-113, 196 "strict movement rules stay" (one dwarf per tile, blocking items) | PRD:54-72 | Not restated, so the new bodies on tiles are unruled: animals, sleeping dwarves "on the ground" (FR64), and 200 dwarves round the fire | What M4 changes / FR64: say whether sleepers and animals occupy a tile (block) under FR49, and that squeeze applies to them or not |
| 2.4 | GAP | RETRO:207-212 M4 inputs: design questions #199, #169 | absent | #199 ("what stops a miner digging deeper") interacts directly with FR63's "warmer the deeper". #169 (the gui mirrors the tui) interacts with NFR10's new TUI display. Neither is placed | ADD: one line each, in scope or parked, with reason |
| 2.5 | GAP | RETRO:210 feel inputs: #179 (FPS swings 20-180), #181, #175, #176, #177 | PRD:154-156 (NFR13) | #179 is the open frame-rate defect that NFR13 at 200 dwarves will hit first, and it is not cited. The others are silently dropped | NFR13: cite #179 as the known baseline. ADD: park #175-#177 and #181 explicitly |
| 2.6 | GAP | RETRO:208 movement debt includes #187 | ADD:52-55 | #187 is missing from ADD's list. Its fix was folded into 13.4, but the issue is still open (EXT:304) | Add #187 to ADD:52 with its state (fixed in 13.4, issue open, so close it or re-check) |
| 2.7 | GAP | RETRO:203 "M3-3 and M3-4 run beside it and don't count toward any cap"; carried #142, #144, #148 | PRD:186 | The counter-metric excludes 13.1-13.4 but says nothing about the tooling action items (#211, #212, #142, #144, #148). M3:166 had that clause | Add: "the retro's tooling items (M3-3, M3-4, #142, #144, #148) run alongside and do not count" |
| 2.8 | GAP | RETRO:192 FR40 "moves to M4" as a **PRD input**; round 20's quote "argues against preparing today's rig" | PRD:167 "Wolf ruled the model and rig need a rethink first" | No LOG entry records an M4 ruling on FR40, and the retro does not say Wolf ruled a rethink-first gate (it quotes round 20). With 200 dwarves and repeating colours, per-dwarf variation is newly relevant to telling dwarves apart | Ask Wolf and log it. Until then, word it as "FR40 stays parked (RETRO:192: not urgent; the rig needs a rethink)" |
| 2.9 | OK | RETRO:191 "more ambitious"; M3 out-of-scope as the menu | PRD:39-42, 159-166 | Needs, resource use and more dwarves are taken; building, threats and skills are explicitly out |
| 2.10 | OK | RETRO:87, 229-231 the seat is the feel instrument | PRD:48-50, 192 |
| 2.11 | OK | RETRO:122 one scenario file, two consumers (bench + seat `--perf-log`) | PRD:85-88 | But see 6.4 (#50) |
| 2.12 | OK | RETRO:211 parked look issues | PRD:160 (inherits M3:149) |

## 3. 13.2 proposal (§ Scale, § Recommendation, § Ruling)

Every number in ADD:23-28 matches `scale-r1-base.txt` / `scale-r1-pile-memo.txt`. ADD:34 (24%, 40-67%) matches
PROP:113-115 and 121. ADD:35 (64,300 vs 50,000) matches PROP:220. PRD:151 (8.5 ms / 65 ms) rounds 8.45 / 64.8 correctly.

| # | Tag | Input | PRD / ADD | Finding | Fix |
|---|---|---|---|---|---|
| 3.1 | GAP | PROP:374-378 "the slowest 1% of ticks sit at the budget"; `scale-hash-pile-memo.txt` 256² crew 200 p99 **81.0**, max 131.5; crew 100 p99 92.5, max 327.6 | PRD:150-152; ADD:21-28 | The PRD quotes the quietest single run (p99 65). The same point read p99 81 under load, and max is already over 100 ms in every run. NFR12 does not say how p99 is measured (runs, floor) | NFR12: quote p99 65-81 ms (load-dependent, single runs) and say NFR12 is read from `tick-bench.sh`'s interleaved runs, not one run |
| 3.2 | CONTRA | PROP:420 Ruling 2: "#207 is re-measured **on top of 13.4**; clamber-over is ruled then"; PROP:388-389 also "a hauler-placement fix closes it" | PRD:52 "decided by the scale run's #207 reading"; PRD:180-181 "on the #207 reading at scale"; ADD:59-60 | The ruling's reading is #207's 24-run sweep on 13.4 (13.4 probe: 18/24, EXT:210), not the 256² scale scenario. The PRD moves the instrument | Say "on #207's sweep re-measured on 13.4 (Ruling 2)". Optionally add "and whether the 256² scenario reproduces it". Mention the hauler-placement alternative |
| 3.3 | CONTRA | PROP:413 Ruling 1: "#186, #187, #203 and #204 **go to their own stories**, outside the movement layer" | ADD:52-55 "They enter as defect stories **when the 256² crew-200 scenario reproduces them**" | The ruling sends them to stories unconditionally, and each already has a reproducer (census replays: #186 probe 5/32, #203 marks=11, #204 seat-slow; PROP:326-330). So TP's reproduce-first rule is already met | ADD: list them as ruled stories (say whether they count toward the 12-16 cap). #186 belongs with FR66, since wander is idle life |
| 3.4 | GAP | PROP:326, 396 #186: `wander` steps on one level only | PRD:134-136 FR66 | FR66 builds idle life on wander, and wander is the cut-order survivor (PRD:188), yet #186 isn't linked to it | FR66: "wander covers more than one level (#186)", or cite it as a known limit |
| 3.5 | GAP | PROP:123-124 "A\* storage [`BTreeMap`] … is the first thing to measure if M4 goes to 256² with hundreds of dwarves" | ADD:34-37 | Dropped. This is the proposal's one named next measurement at the M4 scale | ADD: one line: the first profile at 256² crew 200 reads A\* storage (TP: profile first) |
| 3.6 | GAP (wording) | PROP:404-405, 421-422; TP:42-44 "walkable **component**" | PRD:91-93 "walkable **area**" | The term drifts from the ADR's. The component also grows as dwarves dig, so a budget "sized to" it must track that | Use "walkable component" (ADR 5 wording); add "as it grows with digging" |
| 3.7 | GAP | PROP:217-219, 128 (8 dig + 1 cut mark per dwarf; nothing reached the pile in 2,000 ticks) | PRD:85-86 "does real work" | "Real work" has no checkable bar, and the bench end line can't show it today (2.1). 13.2's trap was exactly this | FR54: the scenario must show items delivered to the pile and food/fuel/need activity on the bench end line within the bench window |
| 3.8 | GAP | Board after 13.3: "scale crew200 273.3 -> 16.8 ms mean" (EXT:183-184, SS:2769) | PRD:151 | NFR12's baseline is the spike stack (`4f57027` + patches), not main after 13.3/13.4 | ADD: re-baseline on main in the first scale story (already implied at ADD:59) |
| 3.9 | OK | PROP:393 "#159 waits for the M4 map size"; PROP:421-422 ADR 5 amended | PRD:15, 91-93; TP:42-44 |
| 3.10 | OK | PROP:414-418 squeeze ruled, idle-blocker swap via 13.4 Task 0 | PRD:52 |

## 4. M3 PRD (parent): is "What M4 changes" complete?

| # | Tag | M3 rule | PRD | Finding | Fix |
|---|---|---|---|---|---|
| 4.1 | GAP | M3:67-69 **FR39** "tell which dwarf is which **at a glance**, in the world and when selected"; M3:156 SC2 "name each of the five by sight" | PRD:66-67 (FR38 only), 101-104 | With repeating colours, "at a glance in the world" no longer holds; FR57 swaps it for "selecting one names it". That narrows FR39, and it isn't listed | Add a bullet: "M3 FR39 narrowed: at 200, a dwarf is identified by selection, not at a glance" |
| 4.2 | GAP + CONTRA | M3:83-86 **FR43** "the five always include at least one miner, one hauler and one woodcutter" | PRD:99-100 groups of "about 5-20, the first at embark … towards a working mix"; PRD:121 six professions | Not listed as reopened. A first group of 5 cannot hold one of each of six trades. "Working mix" is the only guarantee left, and per-dwarf assignment by selection at 200 dwarves is not addressed (no roster, LOG:23) | Add an FR43 bullet: which trades the embark group guarantees (e.g. ≥1 of miner, hauler, woodcutter; the rest from arrivals), and say that the minimum first group holds them |
| 4.3 | GAP | M3:101-104 **FR47** goals: progress = **current** stockpile total; targets hardcoded (`GOALS` Wood 10 → Tetrahedrite 10 → Bloodstone 1, EXT:237) | absent | Burning wood (FR61) lowers the current wood total, so the wood goal's progress can now go **down** before it completes. With 200 dwarves the chain completes almost at once | Add an FR47 bullet: stoked wood leaves the stockpile count (progress may fall until completion), and targets stay or are rescaled |
| 4.4 | CONTRA | M3:115-117 **FR50** "hauls … never leave an item on, or visibly inside, a light emitter such as the campfire" | PRD:117 "Stoking a fire is a haul of wood from the pile to the fire" | Read literally, stoking hauls an item onto the emitter | FR61: stoked wood is consumed as fuel at delivery and never rests as an item on the fire (FR50 holds) |
| 4.5 | GAP | M3:111-114 **FR49** head-on: "the dwarf with the lower id backs off" | PRD:51-52 (baseline only) | 13.4's swap (Ruling 1, plus the idle-blocker swap) replaced FR49's resolution inside the M4 epic, but no PRD records the change | Add a bullet: "M3 FR49 changed by 13.4: a timed swap, including an idle blocker; one dwarf per tile at every tick boundary holds" |
| 4.6 | GAP | M3:76-82 **FR41** "Three jobs exist"; **FR42** professions "(miner, hauler or woodcutter)" | PRD:68-70, 121 | Both closed lists are reopened (forage, hunt, fish, cook, stoke, plus eat/warm/sleep; six trades; woodcutter's job set widened) but only FR42's idle clause is listed | Widen the FR42 bullet: "FR41/FR42's job and trade lists grow (F20); woodcutters also forage" |
| 4.7 | GAP | M3:89-93 **FR45** "Dwarves visibly perform their work … animations"; art hard stop | PRD:131-137 (misery and idle only) | No FR says whether forage/hunt/fish/cook/stoke/eat/sleep are animated, or how animals, plants and a dead fire are drawn. These are new models (art, under M2-24) and they set story count | Add an FR (or an FR45 bullet): which new actions animate on the rail and which ship plain; animals and plants get placeholder or real art, under the hard stop |
| 4.8 | GAP | **M1** out of scope `prd-frostvein-2026-08-01/prd.md:214` "No fluids, **temperature**, weather, seasons"; :216-217 "No needs, moods"; :212 "no … **off-map anything**" | PRD:57-72 | Cold with depth warming (FR63) is a temperature system, and it reopens an M1 non-goal that isn't listed (only M3's "no needs" is). Arrivals and animals entering "at the edges" come from off-map | Add bullets: "M1 'no temperature' reopened for dwarf warmth only (no terrain temperature, nothing melts)" and "M1 'no off-map anything' narrowed: arrivals and animals enter at the edge" |
| 4.9 | GAP | M1:214-215 ice is a terrain material, "nothing melts, or freezes"; no fluids | PRD:114-115 FR60 | Does a fishing hole change terrain (open water? refreeze?) or is it a designation only? Undefined, and it touches the no-fluids non-goal | FR60: "a fishing hole is a designation on ice; no terrain change, no water" |
| 4.10 | GAP (minor) | M2 NFR6 bar: 60 fps working zoom, ≥30 fps full vista, **full 128×128×32 world**, on the vehicle (`epics.md:136`) | PRD:154-156 | NFR13 says "M2's NFR6 bars" without the numbers or venue. The bar was set on 128²; full vista at 256² is 4× the terrain | NFR13: quote 60/30 fps on the gingerspice vehicle, and say whether full vista at 256² is in the bar |
| 4.11 | OK | M3:145 "no fail states, threats, needs, moods or combat"; :146 skills; :147 count five; :143 resources not used; :148 cave-ins | PRD:59-65, 159-166 |
| 4.12 | OK | M3:129-137 NFR9-NFR11 | PRD:141-147 |
| 4.13 | OK | M3:170 "about two pages" vs PRD:196 "about three" | — | TP:120 says "a few pages", so three is within the rule |

## 5. `docs/technical-preferences.md`

| # | Tag | Rule | PRD / ADD | Finding | Fix |
|---|---|---|---|---|---|
| 5.1 | CONTRA | TP (determinism, scenario tests) vs PRD:84 "A 128² five-dwarf world stays reproducible **byte for byte**" | PRD:108 (worldgen places plants), 111 (animals), 116-117 (fires burn out), 141-142 | Every M4 system changes a 128² five-dwarf world's generation and outcome. Byte-for-byte can only hold for the settings story. PRD:194 already says the softer "changes outcome only with a reason on the record" | FR53: "the crew/map settings, left at default, leave today's world byte for byte". Drop the milestone-wide reading |
| 5.2 | GAP | TP:57-58 "Hardcoded constants are fine … config only when a story requires runtime change" | PRD:81-84 "world settings … settable from a scenario file"; PRD:32, 173 "a fresh 256² world" | The PRD never says what `simd` with no scenario starts: 128²/5 (today's default) or 256²/200-by-arrivals. Success criterion 1 and the done sentence need the seat to start one | FR53: name the default, and how the seat starts a 256² arrivals world (scenario file or `simd` flag) |
| 5.3 | GAP (minor) | TP:55-58 no data-driven content systems before a third use; hardcoded constants | PRD:129-133 (thresholds), 100 (cadence hardcoded) | Only the arrival cadence is said to be hardcoded. Need rates and thresholds, regrow time and fish rate are not, and three needs plus six trades invite a generic needs table | One line: need rates and thresholds, regrowth and catch rates are hardcoded constants tuned in their stories |
| 5.4 | GAP | TP:73 "A story fits one dev-agent session"; TP:68-72 vertical slices | ADD:59-60, 65, 66 | Sketch items 6 (fuel + stoking + cold + sleep) and 7 (animals + hunting + fishing + cooking) each bundle 3-4 features. Item 1 bundles settings, scenario, seat run and the #207 re-measure | Split items 6 and 7 in the sketch, and check the split count against the 12-16 cap |
| 5.5 | GAP (minor) | TP:36-38, 59-60 protocol NDJSON, optimization deferred | ADD:39-40; `crates/simd/src/scenario.rs:278-280` (the bench times `World::step` only; delta encoding is outside the window) | 200 dwarves plus animals per tick over NDJSON (memory: Nagle clumps, wire-reader overload) is unmeasured, and the bench can't see it | ADD "not yet measured": wire delta size and encode time at 200, read in FR54's seat run |
| 5.6 | OK | TP:61-62 profile first | PRD:152, 168-169 |
| 5.7 | OK | TP:95-97 nearest epic | PRD:43 |
| 5.8 | OK | TP:98-103 live gate early, reproduce first | PRD:48-50, 93, 192 |
| 5.9 | OK | TP:51-52 "not the imagined fortress of 500" | ADD:69-74 |
| 5.10 | OK | TP:120-121 length, no personas or market research | PRD (~3 pages, genre named only as reference) |

## 6. `extract-inputs.md` (index)

| # | Tag | Extract item | PRD / ADD | Finding | Fix |
|---|---|---|---|---|---|
| 6.1 | GAP | EXT:235 `MAX_DESIGNATIONS = 4096` (`lib.rs:49`); EXT:311 #170 (silent drop at the cap) | ADD:44-45 | The 13.2 scale scenario alone holds 1,800 marks at crew 200 (9 per dwarf). Forage, hunt and fishing designations are added on top. The cap and its silent-drop defect are not on the "fixed at five" list | Add `MAX_DESIGNATIONS` 4096 and #170 to ADD:44 |
| 6.2 | GAP (minor) | EXT:228-229 wander stagger "an eleventh dwarf would share dwarf 0's phase" (`lib.rs:3635-3638`); EXT:256 scale.patch cycles **professions** too | ADD:44-48 | The lockstep note will bite at 200 (visible lockstep reads as "boring"). ADD mentions identity cycling only | Add both to ADD:44-48 |
| 6.3 | GAP | EXT:322-323 #118 (seat keymap out of pressable keys), #102 (controls) | PRD:144-146 NFR10 (new Bevy-only commands: forage, hunt, fishing hole, three new professions) | New commands need bindings, and the keymap is already full. That isn't mentioned | NFR10 or ADD: new commands go through #118's rethink or the selection UI, and say which |
| 6.4 | GAP | EXT:324 #50 scenario-driven gui session mode (only `--at-tick` done) | PRD:88, 154-156 | FR54's "seat run of the same scenario" may depend on #50's open half | ADD: say whether FR54's seat run needs #50 finished |
| 6.5 | GAP (minor) | EXT:316 #219 (squeeze reads as walking through each other); EXT:321 #172 (the tui shows few trees) | absent | At 200 dwarves swaps become frequent (feel), and plants/forage add tui parity display of the same class as #172 | ADD: park or schedule each |
| 6.6 | GAP (minor) | EXT:312-313 #214 (off-map designate accepted silently); #154 | PRD:147 NFR11 | #214 is an NFR11 breach at map edges, where arrivals and animals now enter | ADD: list #214 under NFR11 known breaches |
| 6.7 | OK | EXT:232-234 `CAMP_RADIUS` 3, 7×7 camp | ADD:45 | Verified: `spawn_dwarves` draws from at most 48 cells (49 minus the campfire) and `random_range(0..0)` panics on the 49th (`lib.rs:3609-3626`). "Past about 49" is close enough |
| 6.8 | OK | EXT:274 brief: identity = palette swaps on shared geometry | PRD:101-102 (colours repeat) |

---

## Counts

| Input | GAP | CONTRA | OK |
|---|---:|---:|---:|
| 1 `.memlog.md` | 4 | 1 | 6 |
| 2 Retro §4/§8 | 7 | 1 | 4 |
| 3 13.2 proposal | 6 | 2 | 2 |
| 4 M3 PRD | 9 (one is also a CONTRA) | 1 (+1) | 3 |
| 5 Tech preferences | 4 | 1 | 5 |
| 6 Extract index | 6 | 0 | 2 |
