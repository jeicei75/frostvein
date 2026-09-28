---
title: M3 PRD — input reconciliation
date: 2026-09-28
target: prd.md (draft, 2026-09-28)
inputs: epic-8-retro-2026-09-28.md; prd-frostvein-2026-08-09 (M2); prd-frostvein-2026-08-01 (M1, for non-goals); deferred-work.md; issues #132-#136; sim-core/protocol code
---

# M3 PRD — reconciliation against its inputs

Tags: **GAP** means the PRD should change or say something. **STORY** means real, but it belongs at
story creation, not in a light PRD. **OK** means covered, or correctly left out.

## 0. Code premises (sim-core / protocol at `d3f1693`)

| Premise the PRD relies on | Verdict | Evidence |
| --- | --- | --- |
| Five dwarves | **Holds** | `lib.rs:1613` `for _ in 0..5` in the dwarf spawn |
| No dwarf identity or name | **Holds** | `pub struct Dwarf;` (`lib.rs:111`), a unit marker. Protocol `Entity` is `{id, kind, pos, state, light}` (`protocol/src/lib.rs:137`) |
| Items are stone only | **Holds** | Protocol `Item` has no kind field: `// NOTE: a second item kind adds a kind field; stone is the only item in phase one.` (`protocol/src/lib.rs:167`). FR41/FR46 therefore change the wire shape |
| Trees are Solid tiles | **Holds** | `Tile::Solid(Material::TreeTrunk / TreeFoliage)` (`worldgen.rs:326,330,350`) |
| No gem/mineral materials | **Holds** | `Material { Stone, Soil, Ice, Snow, TreeTrunk, TreeFoliage }` (`lib.rs:47-54`, mirrored in protocol) |
| **Not stated, but it matters:** a Dig mark already accepts tree tiles | **GAP (see 1.4)** | The Dig filter is `matches!(tile, Some(Tile::Solid(_)))` (`lib.rs:1422`). A dug trunk or foliage tile becomes `Empty` and yields **no** item (`lib.rs:931`). So trees can be dug one tile at a time today, for nothing, which leaves the rest of the tree floating |

## 1. Epic 8 / M2-close retro

### 1.1 #133: "must not lock" cannot be met with only "wait or route around" — GAP (highest)

Retro §7: *"It reshapes pathfinding, so the PRD must fix what 'collision' means before any story does."*
#133 lists three options (*"tile occupancy (one dwarf per tile), letting dwarves swap places, or local
avoidance only"*) and warns: *"must not cause deadlocks in 1-wide corridors or around dig faces."*

PRD FR49: *"A dwarf whose next step is occupied waits or routes around the other dwarf. Two dwarves
meeting head-on in a one-wide tunnel must not lock each other forever."*

In a one-wide tunnel there is no way around, and two dwarves who both wait wait forever. The only two
behaviours FR49 names therefore produce exactly the lock it forbids. The PRD has to name the tie-break
(a swap, or a deterministic yield or back-off such as "lower id backs off"). Otherwise the meaning of
"collision" is still open, and the retro asked for it to be settled here. Wolf's memlog wording is
*"wait or route around; enough for now"*, so the tie-break is a small addition, not a reversal.

Also not covered, at STORY level: `settle` dropping a dwarf onto an occupied tile; spawn placement;
a dwarf idling on the only access cell to a dig face (#133 names dig faces explicitly).

### 1.2 Where #133 goes in the order was delegated to the PRD, and the PRD gives none — GAP (small)

Retro §7: *"Suggested order: #134 → #132 → #135, with #133 placed by the PRD."* The critical path puts
*"Red repro tests for #134 and #132"* third, right after the PRD and the epic. The PRD gives no order.
It matters: #133 creates new ways to get stuck. If it lands before #132 is reproduced and fixed, a
collision deadlock cannot be told apart from #132's unknown stall. One line would settle it:
*"#132 fixed before #133 lands."*

### 1.3 #134's premise is unconfirmed, and FR50 states the fix as if it were known — GAP

#134: *"It's unconfirmed whether the stones are actually on the fire's tile, or on a neighbouring tile
that the 3D campfire model visually overlaps. Check which first."*

FR50 fixes only the cell case: *"never put items on a light emitter's cell, including the campfire."*
If the cause is the visual overlap, FR50 ships green and Wolf still sees stones in the fire. The retro's
M2-27 (#141) says *"Every defect story starts from a red scenario test or an observed live repro"*. The
same applies to #135 (*"hasn't been reproduced"*), which SC4 treats as reproducible. The cell rule is
still correct either way (*"A fire's tile shouldn't be a valid stockpile tile either way"*), but FR50
should cover what Wolf saw, not only the cell.

### 1.4 The retro's art hard stop is paraphrased wrongly — GAP (small)

Retro M2-24 (#138): *"Hand-authored art is Wolf's hands or a seat he can watch live. A delegated art
loop returns to Wolf after 2 rounds that his eye has not judged converging, and every round gets a
ledger row."*

PRD FR45: *"two unconverged rounds and the animation ships plain or is parked."*

This has two problems:
- **It drops two of the rule's three parts:** who authors the art (Wolf's hands, or a seat he can
  watch live), and the ledger row for every round. The ledger row is exactly what the $891 dwarf loop
  lacked.
- **It changes the outcome.** After two rounds the retro returns the decision to Wolf. The PRD instead
  has the animation ship plain or be parked automatically.

FR40 (model prep for variation) is art work too and should say the same.

### 1.5 A parked look issue is one M3 itself triggers — GAP

PRD: *"The look issues stay parked until one bites (#120, #126, #127, #128, #130, #136, #137)."*

#136 is *"DoF doesn't focus on the selected dwarf … after selecting a dwarf and zooming in on it, the
dwarf is completely blurred."* FR39 ("which dwarf is which … when a dwarf is selected"), FR43 (assign
professions "by selecting a dwarf") and SC2 ("name each of the five dwarves by sight") all depend on
looking at a selected dwarf up close. #136 bites the moment F14 lands. It should come off the parked
list and join the identity or profession-UI story.

### 1.6 Tooling stories and the cap — GAP (small)

The retro's critical path step 4 is *"Tooling M2-28 to M2-31, in parallel (they speed up every M3
sitting)"*. M2-33 and M2-34 are routed as `story` too. The PRD's *"M3 targets 10–14 stories"* does
not say whether these six count. M2 closed at 28 against 10–14, so the cap's accounting should be
stated.

### 1.7 Covered — OK

- One epic that grows by stories (M2-25), and a new epic only if the milestone goal changes.
- A light PRD (goal, success criteria, cap).
- The instrument per kind of work (§6.1, M2-26).
- Red-first for #132 (FR51, SC4).
- The retro names two suspects for #132: *"an unreachable designation retried forever (no abandon path);
  a channel job orphaned when its support is dug."* FR51's *"cannot reach, or that has lost its
  support, is abandoned or re-planned"* covers both. It also covers retro §4.4's open item
  (`lib.rs:44,442`).
- *"items never fall — so its job retries forever"* (`lib.rs:709`, cited in #135) is covered by
  FR52's falling items.
- The parked look list matches the retro's exactly, apart from #136 (1.5).

## 2. M2 PRD (and the M1 non-goals it carries)

### 2.1 Non-goals reopened without saying so — GAP

PRD: *"The only reopening of M1/M2's non-goals is gems and minerals, which M2 kept out as 'mine
crystals'."* That is not true. Two more are reopened:

- **M1:** *"No farming, crafting chains, or production beyond dig → stone → stockpile."* FR41 (cut
  tree → wood) and FR46 (mineral and gem items) are production beyond dig → stone → stockpile. The
  PRD's own out-of-scope list (*"Resources are not used"*) shows the intent, but the reopening should
  be named.
- **M1:** *"No fluids, temperature, weather, seasons, or cave-ins. … nothing melts, freezes, or falls
  in phase one."* FR52 makes items fall and *"trees fall or are removed."* #135 also asks for a rule
  on *"unsupported terrain in general"*, which is cave-in territory. The PRD should say that falling
  things are reopened, and whether unsupported **terrain** (cave-ins) stays out. As written, FR52's
  "the simplest rule that reads right is chosen in the story" could quietly include cave-ins.

### 2.2 Goals on the wire and FR17's "world, not game" principle — GAP (one line)

M1 FR17: *"messages describe a world, not a dwarf game — state and typed data (positions, materials,
professions), never game rules or narrative interpretation."*

FR47 puts a *"chained tutorial of goals"* on the wire, and FR48 has *"the game says so"*. A tutorial
chain is game rules and narrative. This can square with FR17 if the wire carries only typed state
(the goal kind, target and count) and the clients word it. The PRD should say so, or say that it is
bending FR17. Professions and names were anticipated by FR17 ("professions" is in its own list).

### 2.3 NFR10 narrows the parity rule while saying it applies it — GAP (labelling)

M2: *"the TUI is not extended for Bevy-only work. But any new sim functionality or bug fix that
affects the TUI updates the TUI too — no regression, no stagnation on sim-level change."*

NFR10 is titled *"Parity rule, applied"* but says *"profession assignment and tree designation are
Bevy-only commands in M3; the TUI gains no new input."* These are new **sim** commands, not Bevy-only
work. For the first time the TUI can no longer issue every sim command, which M2 had just brought to
full parity (FR35). The memlog shows that Wolf confirmed this assumption, so it is a legitimate ruling.
It should be labelled as an amendment to the parity rule, not as an application of it.

A side effect: the TUI, M1's *"deterministic assertion instrument"*, can no longer drive a cut or a
profession change end-to-end. Scenario tests cover the sim, so the effect is small, but it should be
stated.

### 2.4 FR42 changes M1 FR5 silently — STORY/OK

M1 FR5: *"Idle dwarves claim the oldest unclaimed job (FIFO)."* FR42 changes this to FIFO within a
trade. That is fine, but FR5 is not cited as amended.

### 2.5 Numbering and inheritance — OK

- FR38–FR52 continue from FR37; F14–F17 continue from F13; NFR9–NFR10 continue from NFR8. All correct.
- The inheritance of NFR5 (no drift), the parity rule, determinism and YAGNI is stated.
- FR38 correctly routes identity through NFR5 (*"Clients render identity; they never invent it"*).
- The five-dwarf count matches M1 FR3 and M2's *"the count stays at FR3's five"*.
- The cap-with-cut-order counter-metric mirrors M2's. A soft cap matches Wolf's scope ruling (retro
  §4.6).

## 3. deferred-work.md items the retro names

| Item | PRD | Verdict |
| --- | --- | --- |
| **Stockpile on rock is a silent no-op** (`deferred-work.md:406`). The retro says: *"the sim should refuse loudly rather than silently discard"* | Dropped | **GAP.** FR50 changes exactly this command's filter by adding emitter cells, and FR44 adds a new designation that will filter the same way (a cut mark over non-tree tiles). That is the silent-sim-filter class that hit 8.2. One line would cover it: *"a command that takes zero tiles says so."* The deferred entry's own trigger is *"Revisit when a story touches command feedback"*, which FR43 and FR44 do |
| **Haul jobs not bounded by `MAX_DESIGNATIONS`** (`:379`) | Not mentioned | **OK.** The entry itself says *"WHY IT IS NOT A DEFECT"*, with the trigger "save items make ticks slow". M3 adds item kinds, so item counts grow, but that is not yet measured |
| **Clear cannot reach a second standable cell** (`:1025`) | Not mentioned | **OK.** The reopen trigger is *"caves, overhangs or multi-level interiors become reachable by a drag"*. Underground mineral mining may make this more common. That is a STORY-level watch item, not a PRD line |
| **Hauls queue behind digs** (the 8.3 story and `vehicle-card.md:41`; not actually in deferred-work.md) | FR42: *"hauls no longer wait behind a dig backlog"* | **OK**, but it depends on at least one hauler existing. See 4.1 |
| **AD-8 full-resend designation amplifier** (`:285`) | Not mentioned | **OK/STORY.** It was closed by 3.2's 4,096 cap. FR44's cut marks must go under the same cap, and the new per-entity names and goals ride every delta. Both are story-level |

## 4. Other internal gaps found during reconciliation

### 4.1 Seeded default professions must cover all three trades — GAP (small)

FR43: *"new dwarves start with a seeded default profession, so a fresh world is playable before any
assignment."* FR42 adds that a dwarf *"does not help with other trades."* A seeded draw over five
dwarves can produce no hauler. Then nothing ever reaches a stockpile, and FR47's goals can never
progress, which contradicts "playable before any assignment". The PRD should require that the defaults
cover all three trades. Wolf accepted FR43 *"with hesitation ('hmm ok')"*, so this is worth raising
with him.

### 4.2 Existing Dig on trees versus the new cut job — GAP

This is the code fact from §0: a Dig mark already accepts `TreeTrunk` and `TreeFoliage` tiles and yields
nothing. With professions:
- a miner will dig trees tile by tile for no wood;
- FR44's *"the same way they designate digging"* leaves the overlap open;
- piecemeal tree digging is the likely source of #135's floating trees.

FR41 or FR44 should say that Dig no longer takes tree tiles, because trees are cut.

### 4.3 Professions are not said to be saved or sent on the wire — STORY

FR38 says names and colours are *"sent over the wire, and saved"*, and FR47 says the same for goals.
FR42 and FR43 say neither for professions. NFR9 lists professions in sim state, but save and load are
a separate path. The TUI's list of new display items (NFR10: *"names, new items, goal progress"*)
leaves out professions, although M1 FR22 anticipated *"colored by current job/profession (e.g. miner
amber, hauler teal)"*.

### 4.4 How goals count — STORY

Several questions about FR47's counting are open:
- Is "in a stockpile" the current stock or cumulative deliveries?
- Does removing a stockpile lower the count?
- Do gems delivered before the gem goal is active count toward it?

These are fine to settle in the story.

### 4.5 Copy — trivial

Several sentences start in lowercase where `[ASSUMPTION]` tags were removed: FR42 "a dwarf with no work",
FR43 "new dwarves start", FR46 "a small fixed set", FR52 "the simplest rule", NFR10 "profession
assignment".

## Ranked summary

1. **FR49's head-on case can't be met:** in a one-wide tunnel, "wait or route around" is a deadlock.
   Name a swap or yield tie-break (1.1).
2. **"The only reopening" is false:** FR41/FR46 reopen M1's "no production beyond dig → stone →
   stockpile", and FR52 reopens "nothing … falls / no cave-ins". Say whether terrain collapse stays
   out (2.1).
3. **Dig already takes tree tiles, for no item.** With professions, miners fell trees piecemeal for no
   wood. That is the likely #135 source (4.2).
4. **#136 bites under F14/FR43:** a selected, zoomed dwarf is blurred, yet the PRD parks it (1.5).
5. **Silent command filters:** the deferred "stockpile on rock is a silent no-op" (refuse loudly) is
   dropped while FR50 and FR44 add more of the same filtering. And #134's premise (fire cell versus
   visual overlap) is unconfirmed while FR50 asserts the cell fix (3, 1.3).
6. **Smaller items:**
   - the art hard stop is misquoted: the ledger row and "who authors" are dropped, and the outcome
     becomes auto-park (1.4);
   - seeded defaults must include a hauler (4.1);
   - NFR10 amends the parity rule rather than applying it (2.3);
   - goals on the wire need to square with FR17 (2.2);
   - #133 is not ordered after #132 (1.2);
   - the cap does not say whether tooling stories count (1.6).
