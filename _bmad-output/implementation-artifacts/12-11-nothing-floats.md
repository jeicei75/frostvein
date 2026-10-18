---
baseline_commit: 56bfbe5
model: claude-opus-5-5  # session default, same as 12.1-12.10b's creation
---

# Story 12.11: Nothing Floats

Status: in-progress

## Story

As the boss,
I want things above a dug-out block to come down,
so that the world never shows stones hanging or trees standing on air.

## Not stacked: branch off `main`

`main` is `56bfbe5` (PR #202 merged), clean. Branch `story-12-11-nothing-floats` carries this file,
`12-11-signoff/` and the board edit. Epic 12's standing ACs (`epics.md:2422-2434`) bind this story and are
not restated. **No wire shape change** (standing AC 4; D7 reuses `DwarfJob::Haul` for a clearing miner): an item's `pos` is resent in full on every delta
(`protocol/src/lib.rs:254-258, 296`; `simd/src/bridge.rs:80`), so a fall is a changed `pos`; a felled tree
is `Empty` tiles through `Terrain::set_tile`'s dirty set. **No new saved state for a fall**: it changes positions
and tiles only. The one new saved field is `SavedDwarf.clearing` (D6, `31c0b16`), required by Wolf's review run 1
ruling (b), so a pre-12.11 save is refused.

## Found at creation (2026-10-15, on `56bfbe5`)

**The epic's "#135 is unreproduced" is stale.** 12.10b measured floating blocks and deferred them here
(`12-10b-a-way-down.md:45-46, 262-265`; `deferred-work.md:2649-2651`). The epic's `lib.rs:709` cite is now
`lib.rs:1189-1190`. Probe: `12-11-signoff/creation-prototype.patch` (its `probe_floats.rs` and the
throwaway rule; never ship either).

- **Items, by channel (natural play).** 12.10b lands a channel's block up top beside the worker, often on a
  cell a later mark channels under. The 12.10b two-wide staircase sweep (pile placed, delays 0..24)
  leaves **4-10 floating blocks in 24 of 24 runs, still floating 3,000 ticks later** (longest 4,525 ticks).
  Channel the cell one block landed on, and it hangs for the whole 4,000-tick run with its haul job queued
  and retried (`retry_after` pushed forever, the `is_standable` gate at `lib.rs:1194`).
- **Items, by dig.** A stone lying on a dig target's top hangs one cell above the dig's own new stone
  (`[61,62,9]` over `[61,62,8]`, 3,903 ticks). This is 12.10's seat-pass-2 "floating rubbish"
  (`12-10-signoff/seat-pass-2-trench.png`).
- **Trees.** Dig and channel refuse tree tiles (`lib.rs:2533-2549`), but a dig of the GROUND cell under a
  trunk base is accepted. **92 of DEFAULT_SEED's 259 trees** have a ground cell workable from its own level;
  digging `[103,2,8]` leaves the trunk at `[103,2,9]` over air. The wire picks `[67,53,11]` under the base
  `[67,53,12]`.
- **Dwarves.** None left on air: the longest non-standable streak over all 24 sweep runs is **0 ticks**
  (`settle`, `lib.rs:1909-1952`). Its one hole: a faller whose landing is taken and who has no free tile
  stays put (NOTE `:1941-1942`).
- **Emitters (by reading, not run):** the Dig filter has no emitter check (only Channel, D3 of 12.10b), so
  a dig under a torch or the fire, worked from a pit, would leave it hanging.
- **gui (by reading, not run):** a `DigChip` is despawned only when ITS cell is dirty
  (`gui/src/project.rs:2131-2137`). A channel empties the cell UNDER a cell an earlier channel left chips
  on, without dirtying it, so every staircase level's chips hang over the next level's hole.
- **Wire, `12-11-signoff/floats_wire.py` (written at creation):** on `56bfbe5`, **3 of 3** fresh daemons
  `items floated >= 30 ticks: 9`, `tree: base [67, 53, 12] over air`, `FLOATS RED`. On the prototype,
  **3 of 3** `items floated >= 30 ticks: 0`, `tree: felled or supported`, `FLOATS OK`. 12.10b's
  `way_down_wire.py` stays `WAY DOWN OK` on the prototype (2 of 2).
- **Pure gravity regresses 12.10b.** With every item falling straight down (a channel's lying block drops
  into the opened step), the two-wide staircase sweep falls to **17 of 24** (stalls at level 3, delays 5, 6,
  7, 9, 17, 18, 19), and the cage sweep freezes **17** runs against the 1 pinned. With option Q1(b) below,
  the staircase is **24 of 24** and the cage sweep freezes **5**: `(0,4,2) (6,4,4) (10,4,0) (10,6,1)
  (15,6,1)` against the pinned `(10,4,4)` (`scenario.rs:2951`). Why those 5 freeze is NOT established.

### Task 0: RULED (Wolf, 2026-10-15)

- **Q1 (b), and the miner does it himself, #180 FOLDED IN.** Wolf: "miner seems to be lazy so would be nice
  if miner would first move stone because he is the nearest one and then continue digging anyway in any
  case", and: "no need to carry the stone to stockpile but move it aside .. then hauler can pick it up and
  move it to stockpile". So a stone in a miner's way is moved by **that miner**, as a visible lift-carry-set-down step of his
  own job, and then he digs. This covers a stone on a channel target (Q1) AND a stone that blocks his dig
  face, e.g. in a one-wide tunnel (#180), which today waits for a hauler, and with no pile stops for good.
  See D6. The creation prototype's Q1(b) moved the stone in the cut's own tick (no walk), so its 24 of 24
  and wire numbers are evidence for the RULE, not for this mechanism: re-measure.
- **Q2 (a):** the dig fells the tree as a cut does and its logs fall.
- **Q3:** keep a SHORT seat pass (AC 12).
- **Q4 (Wolf, 2026-10-15, after creation): he looks like a hauler while he clears.** While a miner moves a
  stone aside, the wire reports his job as `Haul`, so both clients show the hauler's lift, carry and
  set-down, not the dig swing. See D7.

The options as offered at creation, kept for the record:

- **Q1, an item lying on a channel's target.** (a) It falls into the opened step like any item. Simplest,
  but it blocks the way down until hauled (the trap 12.10b's "block lands up top" ruling avoided):
  staircase 17 of 24. (b) **Recommended:** the channel lifts it out with its own block, landing it up top
  by 12.10b's rule (nearest free cell from the worker; never in the target's column, never below the worker,
  no dwarf on it, walls no dwarf in); with no free cell the cut is undone and the job retries, as today.
  Staircase 24 of 24, wire OK. (c) The channel waits until the item is hauled; with no pile the mark never
  finishes (FR8), and the staircase leaves a block on most next-level cells. Not prototyped.
- **Q2, a tree whose ground is dug.** (a) **Recommended:** the dig fells it as a cut does (every tree tile
  `Empty`, one log per trunk cell at the base, `lib.rs:1753-1775`), and the logs then fall by the item rule.
  Prototype: tree gone, logs beside the hole. (b) It is removed and yields nothing. (c) A dig mark is refused
  on the cell under a trunk base (prevention; "a tree whose support is dug out" then never happens).
- **Q3, the seat.** AC 12 below is a short seat pass (staircase + one tree dig). This is not a look story,
  but the defect came from the seat and Task 6 changes what the gui draws. Keep it, or judge by the wire
  and the scenario tests only?

### Defaults taken at creation (Wolf may overrule; listed in the hand-off)

- **D1:** an uncarried item whose cell is `Empty` over `Empty` drops **in the same tick** to the first
  standable cell down its column. Dwarves keep one level per tick. The gui snaps items every delta
  (`project.rs:2879-2892`), so a one-level fall reads as a fall either way.
- **D2:** where that cell is taken (an item, an emitter's tile, a dwarf, or filling it walls a dwarf in),
  the item rests on the nearest free cell from it by 12.9's drop rule, **never higher than the cell it fell
  from** and never stacked on another item (#154's shape). With no free cell it stays and retries next tick
  (`// NOTE:`).
- **D3:** a tree crown is ground: `is_standable` counts any `Solid`, foliage included, so an item can come
  to rest on a crown. `// NOTE:` only.
- **D4:** a dig mark is refused on the cell under an emitter's tile (12.10b D3's twin), and the gui preview
  mirrors it.
- **D5:** gui debris whose floor is emptied is removed (Task 6). Client-local, no wire.
- **D6 (the miner clears his way, Q1 + #180):** a miner holding a dig or channel job whose way is blocked by
  an uncarried item (one lies on his channel's target, or every work position of his job is blocked by
  items) lifts the nearest such item (`WORK_TICKS`, like a pick-up), carries it to the nearest free cell by
  the 12.9/12.10b drop rule from where he stands (never in the target's column, never a work position of his
  job, never below him, no dwarf on it, walls no dwarf in), sets it down (`WORK_TICKS`), and goes back to
  the same job. It is a step of HIS job: no haul job is claimed, his trade does not change (AD-12), and no
  pile is needed; the pile stays the hauler's, who later hauls the stone from where it was set aside. A
  hauler already holding that item's haul job is not waited for: when the miner lifts it, that hauler lets go
  (`release_claim`, carrying nothing), and the haul job is claimed again once the stone is down (AC 3). One
  owner per item at a time.
- **D7 (Q4, the wire while clearing):** from the start of the lift run to the end of the set-down run, the
  clearing miner's `Entity.job` is `DwarfJob::Haul`; before and after, his own `Dig`/`Channel`. **Wire diff
  (standing AC 4): no shape change.** `DwarfJob::Haul` gains a second meaning ("a dwarf carrying a stone",
  not only "a hauler's haul job"); `profession` stays `miner`, so a client can still tell them apart.

## Acceptance Criteria

1. RED first: the tests of Task 1 are recorded RED on `56bfbe5` (the fall rule, the channel item, the tree
   and the staircase float count), with their failing assert lines, before any rule change.
2. At the end of every tick, no uncarried item lies on an `Empty` cell over an `Empty` cell, except where
   D2 leaves it with no free cell. A fallen item rests on a standable cell, never on an emitter's tile, a
   dwarf's tile or another item's tile, never higher than it started, and never walls a dwarf in (D1, D2,
   FR50).
3. A fallen item with a haul job is delivered to the pile by a hauler (the job follows the item's live
   position); no fallen item's haul job retries forever.
4. An item lying on a channel's target is carried off it by the channel's own miner before he cuts (the
   wire shows him `carrying` it while holding the channel), and it never floats and never lands in the
   opened step (D6).
5. A dug-out tree ground cell leaves no trunk base over air at the end of the tick, by the rule Task 0 Q2
   rules, and the story records the rule in the spine.
6. Over every sweep in this story, at the end of every tick every dwarf is on a standable cell or falling
   (the cell under him `Empty`), and no dwarf is on air longer than his fall.
7. Unsupported terrain does not change: a `Solid` cell over an `Empty` cell stays where it is.
8. A dig mark is refused on the cell under an emitter's tile, and the gui preview does not light it (D4).
9. On DEFAULT_SEED, 12.10b's two-wide staircase sweep stays **24 of 24** and leaves **0** floating items
   for every delay. The cage sweep (`no_dwarf_is_caged_by_stones_over_the_probe_seeds`), the one-wide sweep
   and the walking-skeleton test stay green, or each re-pin is disclosed beside the pin.
10. In the gui, no dig debris remains over a cell that has been emptied under it (D5).
11. `floats_wire.py` prints `FLOATS OK` on 5 of 5 fresh daemons, `way_down_wire.py` prints `WAY DOWN OK`
    on 5 of 5, and each mutation row of Task 7 turns its named test, or the wire, RED.
12. *(Task 0 Q3, short)* At the seat, Wolf channels a two-wide staircase 3 levels down, digs out the ground
    under one tree, and digs a one-wide tunnel 4 long with no pile. He sees nothing hanging (no block, no
    tree, no debris) and the miner carrying stones out of his own way like a hauler (lift, carry, set down,
    no pick swing), in the gui and an attached tui.
13. #135 and #180 close with their red-then-green tests.
14. With no stockpile, a miner digs a one-wide tunnel of 4 marks into rock to the end: each stone that
    blocks his next face is carried off by him, and no hauler is involved (D6, #180). RED today: he stops at
    the first stone (`lib.rs:1142-1143`).
15. While a miner clears a stone (lift, walk, set-down) the daemon reports his `job` as `haul`, and before
    and after it as his `dig`/`channel` (D7); the gui then plays the hauler's lift and Carry clip, never the
    dig swing, for that stretch.

## Tasks / Subtasks

- [x] **Task 1: RED tests first** (AC 1)
  - [x] `crates/sim-core/tests/scenario.rs` (UPDATE), from the probe's tests: the channel-under-a-block case
    (AC 3, 4), the dig-under-an-item case (AC 2), the hillside tree dig (AC 5; DEFAULT_SEED `[103,2,8]` or the
    nearest such tree to the camp), the one-wide tunnel with no pile (AC 14), and a float counter added to
    `a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay` (`scenario.rs:3441`) that
    asserts 0 floating items and the AC 6 dwarf rule every tick (AC 9). Items placed off the natural flow go
    in through `to_save`/`from_save` (`SaveState.items`, `save.rs:25`), as the probe does.
  - [x] Run each on unchanged `56bfbe5` code and paste the failing assert lines into Debug Log References.
    Commit them with `#[ignore = "RED until Task N"]`, un-ignored by the commit that greens them.
- [x] **Task 2: items fall** (AC 2, 3, 6)
  - [x] In `settle` (`lib.rs:1909-1952`), after the dwarves, in ascending item id: each uncarried item that
    is `Empty` over `Empty` drops to its landing (D1, D2). The landing search is `nearest_free_cell`
    (`lib.rs:1232`) from the column's first standable cell, refusing taken cells, emitter tiles, dwarf tiles,
    `walls_in_a_dwarf` and any cell above the start.
  - [x] One helper for "the first standable cell down this column", used by the items AND the channel's
    occupant check (`lib.rs:1861-1877`, today an inline loop). Don't add a third copy.
  - [x] `work_positions`' haul leg (`lib.rs:1189-1199`): rewrite the "items never fall" NOTE. Keep the
    `is_standable` gate: it now covers only D2's no-free-cell case.
  - [x] Rewrite `settle`'s two NOTEs (`:1941-1942`, `:1951`) to say what is now true.
  - [x] Flip the unit test `a_stone_on_unstandable_ground_leaves_its_haul_job_queued_and_retried`
    (`lib.rs:3416`), which pins the old behaviour; disclose it.
  - [x] Fixtures that say "items never fall" (`scenario.rs:1120-1121`, `simd/tests/serve.rs:2155-2158`):
    fix the comment, keep the fixture.
- [x] **Task 3: the miner clears his way (D6, D7; Q1, Q4, #180)** (AC 4, 9, 14, 15)
  - [x] Make a blocked dig/channel job claimable and workable by its miner: today a job whose every work
    position holds an item has an EMPTY work set (`work_positions`, `lib.rs:1144-1147`, `:1155-1162`), so
    `claim_jobs` never claims it (reachability is checked at claim, `:479-620`). The blocking items' lift
    cells must count, and a channel's target item must be lifted before the cut. Decide where the trigger
    lives, and say so in a `// NOTE:`; no trade or claim-filter change (AD-12).
  - [x] The lift, walk and set-down reuse the haul's mechanics (`Carrying`, `WORK_TICKS` per lift and
    set-down, `carry_items`), with the D6 landing rule. After the set-down the miner goes back to the same
    job; a channel then cuts with its target clear, so 12.10b's landing code is unchanged.
  - [x] Every place that assumes "a dwarf carries only the stone of the haul job he holds" now also allows a
    miner holding a dig/channel job: the `debug_assert` in `work_positions` (`lib.rs:1165-1168`),
    `release_claim` (`:1257`, a cleared stone a released miner holds is dropped by its rule), and **simd's
    save validation** (`simd/src/main.rs:568-577`, which today refuses such a save). A save taken mid-clear
    loads and finishes the clear (standing AC 3's save/load gate).
  - [x] D7: a public sim reader that names a clearing miner (and the item), and in `simd/src/bridge.rs`
    `dwarf_entities` (`:181-186`) report `DwarfJob::Haul` for him while he clears (exhaustive `match`
    kept). Tests: `serve.rs` reads `haul` on the wire for a clearing miner and `dig`/`channel` before and
    after; a gui headless test that a miner entity with `job: Haul` + `carrying` gets `DwarfClip::Carry`
    from `dwarf_clip` (`gui/src/project.rs:464-479`), so no clip change is needed.
  - [x] A hauler holding the lifted item's haul job lets go when the miner lifts it (D6); pin that the item
    is later delivered to the pile (AC 3) and that no item ever has two holders.
  - [x] The judge is AC 9 and AC 14: the staircase sweep 24 of 24 with 0 floats, the one-wide sweep, and the
    no-pile tunnel. Then the cage sweep: name the mechanism of each newly frozen run from a dump (as 12.10b
    Task 4 did) before you re-pin `ONE_LEVEL_ISLANDS` (`scenario.rs:2951`). If one is a dwarf caged by a
    moved item, fix it, don't pin it.
- [x] **Task 4: trees, by Q2** (AC 5)
  - [x] If (a): extract the Cut arm's felling (`lib.rs:1753-1771`) into one helper. The Dig arm calls it
    when the cell above a completed dig target is `TreeTrunk`. The logs then fall by Task 2 in the same tick.
  - [x] A Cut mark or held cut on that tree then completes as today's no-tree no-op (`tree_of` → `None`,
    `:1772-1775`); pin that the woodcutter is released.
- [x] **Task 5: dig filter under an emitter** (AC 8)
  - [x] `apply_command`'s Dig filter (`lib.rs:2533-2538`): refuse the cell under an emitter's tile
    (`emitter_tiles`, `:2519`). A rect left with zero cells is a `Refusal`, as today.
  - [x] gui `sim_will_keep` (`crates/gui/src/project.rs:1079-1089`): mirror it for `DesignateMode::Dig`.
- [x] **Task 6: gui debris** (AC 10)
  - [x] In `reconcile`'s dirty-tile loop (`project.rs:2131-2160`): a dirty tile that is now `Empty` also
    despawns the `DigChip`s of the cell above it. Headless test beside `headless.rs:990-1006`: a channel's
    chips, then a channel under them, leaves 0 chips at that cell.
- [x] **Task 7: the instrument, its RED and the mutation table** (AC 11)
  - [x] `12-11-signoff/floats_wire.py` (EXISTING, creation) is the instrument. Run it on 5 fresh release
    daemons, each started from a directory with no `frostvein.save`, at most 4 at a time. Paste each verdict
    block. Do the same with `12-10b-signoff/way_down_wire.py` (its pile was sized while blocks floated,
    `deferred-work.md:2664`; resize it only if it goes RED for want of pile).
  - [x] Deliberate RED of the instrument on the finished branch: rerun it with row "fall removed" applied
    in a scratch worktree build and paste the `FLOATS RED` block (the creation RED was on `main`).
  - [x] NEW `mutations/12-11.sh`, each row run alone after commit and named against its killing test: the
    fall removed (AC 2); a landing on an emitter allowed (AC 2, FR50); a landing on a dwarf allowed (AC 2);
    "never higher" dropped (AC 2); the miner waits instead of clearing (AC 4, 14); the bridge reports `Dig` while clearing (AC 15); the set-down allowed in
    the target's column (AC 4); the Q2 rule reverted (AC 5); the D4 filter
    dropped (AC 8); the chip despawn dropped (AC 10). Re-point any old row your NOTE edits break
    (`scripts/audit-mutations.py`).
- [x] **Task 8: seat card** (AC 12, if Q3 keeps it)
  - [x] NEW `12-11-signoff/vehicle-card.md`, in the seat's launch form (12.10b's card is the template):
    the staircase at `[61..65, 62..63, 9]` with a BIG pile, and the tree at `[67,53,12]` (dig its ground
    `[67,53,11]`, workable from its own level), and a one-wide tunnel 4 long with NO pile (give the cells).
    Note the compass mirror (#169).
- [x] **Task 9: spine and gate**
  - [x] `ARCHITECTURE-SPINE.md` (2026-08-09, beside its 12.10/12.10b bullets): one bullet for the fall rule, Q1 and Q2 as ruled.
  - [x] `RUST_TEST_THREADS=1 scripts/gate.sh`, full and green, before review.

### Review Findings

Code review run 1, 2026-10-17, on `1b1b99b` (diff `56bfbe5..HEAD`, 21 files, +3,143/-124; code 7 files,
+1,667/-94). This was a fresh session. Four layers ran and none timed out. All ran cargo 1.97.1, each in its
own `/tmp/review-<layer>` target dir.
- Blind Hunter (Sonnet) took `sim-core/src` and gui `project.rs`.
- Edge Case Hunter (Sonnet) took `simd`, gui `tests/` and sim-core `tests/`.
- The Acceptance and Feature Auditors (Opus) took the whole diff, without the story file or the creation prototype.

| Layer | Findings | Severity | Ran |
| --- | --- | --- | --- |
| Blind Hunter | 10 | 4 MED, 6 LOW | `cargo test -p sim-core` (green); **no probe**, so every finding was reasoned |
| Edge Case Hunter | 12 | 3 MED, 9 LOW | `simd` (75 green), the scenario suite (76 green, cage sweep included); **no probe or daemon**, so every finding was reasoned |
| Acceptance Auditor | 10 | 1 HIGH, 2 MED, 7 LOW | sim-core and simd full suites (green); a save → load → step equivalence probe at every tick of a clear (**RED**); per-tick probes over the 32 cage runs and 24 staircase delays (AC 2/6 clean); floats and way-down wire, 1 fresh daemon each (OK) |
| Feature Auditor | 4 | 1 MED, 3 LOW | hop trace (every hop WIRED); `floats_wire.py` on a fresh daemon (OK); the tunnel with no pile on the wire (4 of 4); 16 clearing stretches on the wire; tree logs probe; dig-under-emitter refusals live; tui `--z 13`; the gui chip and clip headless tests |

Convergence: the cancel mid-clear drop (blind + feature + acceptance) and `haul` before the lift (edge +
acceptance). After triage: 3 decision (all ruled, now patches), 7 patch, 9 defer, 12 dismissed. Unproven by any run: AC 10, AC 15's gui
half and AC 8's gui half at the seat (headless only); the seat record quotes the dig-loop save sitting, not the
card's questions 1-3, on the final code. AC 13 is a merge-time event; watch the closing-keyword trap.

**Patch pass, 2026-10-18 (fresh session, Opus 5.5 by Wolf's pick): all 10 landed** in `31c0b16`, `240a525`,
`ad2d183`, `c0f6c63`, `a2876db`, `c051bf0`. The closure table, REDs, mutation rows and wire runs are in the Debug
Log's "Review run 1 patch pass" entry.

- [x] [Review][Patch] **A save taken mid-clear does not step like the never-saved world** (acceptance, HIGH,
  RAN) — standing AC 3 ("new sim state is deterministic and saved"). `Clearing { item, landing, drop_from }`
  (`lib.rs:287`) is not in `SavedDwarf` (`save.rs:30`). On load the trigger re-inserts it with `WorkProgress(0)`
  (`lib.rs:1901-1912`), so the lift or set-down restarts and the kept landing is lost. A probe that saves at every
  tick of a clear diverges at save ticks 48-52 and 64-80. No `save_load.rs` test was added; the serve test proves
  only that the clear finishes. The story's "No new saved state" holds for the fall, not for D6. The choice is the
  save format: (a) `clearing: Option<SavedClearing>` with `#[serde(default)]`, because a pre-12.11 save has no
  clearing miner, so `None` is exact and Wolf's kept saves still load; (b) the same field with no default, by house
  precedent, which refuses every pre-12.11 save, `~/frostvein-saves/12-11-dig-loop.save` included; (c) file an
  issue instead. **RULED (Wolf, 2026-10-17): (b)**, no `#[serde(default)]`; pre-12.11 saves are refused.
- [x] [Review][Patch] **The wire never judges AC 3: `floats_wire.py`'s pile holds 22 cells** (feature, MED,
  RAN; acceptance + blind) — the docstring's "big enough for all 28 blocks" is false
  (`12-11-signoff/floats_wire.py:12`). "22 of 32 on the pile" is the pile's capacity, and the other 10 stones
  have haul jobs retrying for good. 12.10b's review deferred "resize it at 12.11", and this story did not. No sim
  test pins AC 3 either: the flipped unit test asserts only that `z` fell. With pile layers at z 10-12, all 32 are
  delivered. (a) a scenario test of a fallen stone delivered to the pile, and the docstring corrected; (b) (a), plus
  the wire's pile resized so that OK needs every item on the pile (the seat card is untouched). **RULED (Wolf,
  2026-10-17): (b).**
- [x] [Review][Patch] **Cage pin `(12,4,4)` is a moved-stone cage, re-pinned without a ruling** (acceptance,
  LOW) — Task 3 says "fix it, don't pin it". Its own comment reads: his own clearing set-down took one exit, and a
  later cut took the other. That is the same "closed by a cut" shape Wolf ruled for `(12,4,1)` (#186), but no
  ruling names it. (a) extend the `(12,4,1)` ruling and note it on #186; (b) fix it. **RULED (Wolf, 2026-10-17):
  (a)**, the record beside the pin and a note on #186.
- [x] [Review][Patch] **A cancel or a vanished job mid-clear drops the stone at any distance**
  [`crates/sim-core/src/lib.rs:1347`] (blind + feature + acceptance, MED) — `apply_command`'s cancel removes the job
  before `release_claim` (`lib.rs:3033-3050`), and so does the vanished-job path (`:1835`). `clearing_job` is then
  `None`, so Wolf's "lets go only beside himself" guard is skipped and `nearest_free_cell` searches at any
  distance. That is the "throwing stones" path. Fix: key the beside-him rule on `Clearing` alone, and the
  target-column refusals on the job when it still exists.
- [x] [Review][Patch] **The wire reads `haul` while the miner walks to the stone empty-handed**
  [`crates/sim-core/src/lib.rs:3160`] (edge + acceptance, MED) — D7 says "from the start of the lift run".
  `Clearing` is inserted when the trigger fires (`:1905`), so in the probe he read `haul` about 40 ticks before the
  lift began. A stone on his only mark means he never reads `channel` first. Fix: `World::clearing()` names a miner
  only while he carries or works the lift (`JobState::Work`). The bridge then needs no change.
- [x] [Review][Patch] **A one-delta pick swing after every set-down** [`crates/sim-core/src/lib.rs:2076`] (feature,
  MED, RAN) — this breaks AC 15's "never the dig swing". The set-down removes `Clearing` but leaves `JobState::Work`,
  so the next delta reads `channel`/`dig` + `work`, which is `DwarfClip::Dig` (16 of 16 stretches on the wire). Fix:
  set `JobState::Walk` at the set-down, as the haul pick-up does.
- [x] [Review][Patch] **simd's relaxed carry rule has no refusal test and accepts two holders**
  [`crates/simd/src/main.rs:568`] (edge, MED) — dropping `profession == Miner`, or widening the `matches!`, keeps
  every test green. A save in which miner A carries X while hauler B holds `Haul{X}` is accepted, which is the
  two-holders state D6 forbids. Fix: refuse when another dwarf holds the item's haul job, and add serve tests for
  the refusal side: a hauler with a dig job carrying, a miner on a cut job carrying, and the two-holders save. If
  decision 1 saves `Clearing`, also require the carried item to equal the saved clearing item.
- [x] [Review][Patch] **The no-throw sweep can pass with nothing carried** [`crates/sim-core/tests/scenario.rs:4296`]
  (edge, LOW; latent vacuity) — `run_until`'s result is ignored and nothing asserts that a stone was ever carried.
  Fix: assert each level is worked, and assert a carried-stone count > 0.
- [x] [Review][Patch] **Row 7's "the column alone is equivalent" is unproven for Dig**
  [`_bmad-output/implementation-artifacts/mutations/12-11.sh`] (acceptance, LOW, read) — for a Dig target, the cell
  on its top is refused only by the column clause. A stone set there falls into the dug cell. Fix: add a Dig-fixture
  test that kills the column-only mutation, or correct the Debug Log's claim if the mutation proves equivalent.
- [x] [Review][Patch] **The spine bullet omits `caging_stone` and the seat guards**
  [`_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:182`]
  (acceptance, LOW) — the clearing trigger now has three cases, and the three guards are lift only with a set-down,
  let go only beside himself, and wait while holding.
- [x] [Review][Defer] **AC 6's "no dwarf on air longer than his fall" is asserted nowhere, and standable-or-falling
  only in the staircase sweep's `run_until`** [`crates/sim-core/tests/scenario.rs:4320`] — deferred. A per-tick
  probe over all 56 runs found 0 violations.
- [x] [Review][Defer] **AC 7 (terrain never falls) has no test or row** — deferred; no terrain-fall code exists.
- [x] [Review][Defer] **A clearing set-down may land on a free pile cell** [`crates/sim-core/src/lib.rs:1234`] —
  deferred; harmless, but the guardrail and the record do not cover it.
- [x] [Review][Defer] **`closes_last_level_exit` counts only same-level sides** [`crates/sim-core/src/lib.rs:833`] —
  deferred. A fallen stone whose one candidate is refused hangs until the dwarf moves (D2's NOTE; Wolf ruled the
  check kept everywhere; it is the #186 shape for an idle dwarf).
- [x] [Review][Defer] **A one-tick flap of the trigger resets dig progress** [`crates/sim-core/src/lib.rs:1914`] —
  deferred; reasoned only, and the kept-stone rule covers the observed ping-pong.
- [x] [Review][Defer] **`caging_stone`'s per-tick BFS cost is unmeasured** [`crates/sim-core/src/lib.rs:1863`] —
  deferred; the BFS is capped at 64 cells, with a few miners.
- [x] [Review][Defer] **A dig's own spawned stone landing has no `closes_last_level_exit`** [`crates/sim-core/src/lib.rs:2240`]
  — deferred, beside #203 (the dig's wall-in check reads pre-dig terrain).
- [x] [Review][Defer] **The serve D7 test does not pin one dwarf across its three stages and relies on seed-fixed
  cells** [`crates/simd/tests/serve.rs:2935`] — deferred.
- [x] [Review][Defer] **The serve mid-clear load test's "opened" check does not pin the carrier as the cutter**
  [`crates/simd/tests/serve.rs:1271`] — deferred.

**Code review run 2, 2026-10-18, on `5a8fbaf`** (fresh session; diff `1b1b99b..5a8fbaf`, the run-1 patch pass, 13
files, +967/-33; code 3 files). It used the same layer setup as run 1: cargo 1.97.1 in each layer's own
`/tmp/review-<layer>` target dir. Blind Hunter took `sim-core/src`. Edge Case Hunter took simd, sim-core `tests/`,
`mutations/12-11.sh` and `floats_wire.py`. The two Opus auditors took the whole diff, with the story file.

**Delta vs run 1.**
- NEW findings after triage: 0 HIGH, 1 MED, 2 LOW to patch, 2 LOW to defer.
- REWORK: 1. Closure-table row 4 (cancel mid-clear) is HALF-CLOSED. "Beside him" is pinned, but "never below him" is not.
- The other 9 rows were audited CLOSED by the acceptance layer. It ran their tests and four sabotages, and forked 2,163
  save→load copies of a live world with 0 divergences.
- Raw findings: 13 (blind 3, edge 3 plus 3 wire hypotheses, acceptance 4, feature 3). 8 were dismissed.
- **The stopping rule fires**: no new finding is HIGH, so the audit ends here. The next spend goes to the seat, not
  to round 3.

| Layer | Findings | Severity (after triage) | Ran |
| --- | --- | --- | --- |
| Blind Hunter | 3 | 0 kept (all dismissed) | a save→load→step probe at every tick t = 1..399 of a channel-mark clear, full `to_save` compared for 60 ticks (green) |
| Edge Case Hunter | 3 + 3 wire hypotheses | 1 LOW defer, 1 LOW defer folded in; rest dismissed | simd `load_world_from` on a real save at every tick of a clear (73 clearing, 28 carry ticks, 0 refused); every `12-11.sh` payload applied to a scratch copy (all apply). **Partial coverage**: it went silent 8 min, answered the check-in by reporting early, and ran no wire and no mutation kill. The other layers ran both. |
| Acceptance Auditor | 4 | 1 MED patch (REWORK), 1 LOW patch, 1 LOW defer, 1 to the live gate | closure-table audit; 2,163 save→load forks (0 diverged; the probe goes RED at 48-52/64-80 when the load drops `Clearing`); 4 sabotages; Wolf's real pre-12.11 save refused live (`missing field 'clearing'`), positive control with `clearing: null` loads; `floats_wire.py` OK 32 of 32; `way_down_wire.py` OK |
| Feature Auditor | 3 | 1 LOW patch (3 merged) | hop table, every hop WIRED; live per-delta traces of the tunnel, 3 staircases, staircase plus tree; a mid-clear save through the daemon loaded into a fresh daemon (identical to the never-saved run); a cancel mid-carry (stone one cell from him); `floats_wire.py` OK 32 of 32; the same tracer on a `1b1b99b` binary goes RED (3 of 3 set-down swings, 12 empty-handed `haul` deltas); gui headless clip tests 4 of 4 |

What the runs prove: on the final code, live on the wire, a miner walks to the stone as `dig`/`channel`. He reads
`haul` from the lift to the set-down and walks after it, and he never sets a stone down more than a cell away (cancel
included). A mid-clear save resumes exactly, and a pre-12.11 save is refused.

What they do not prove:
- **AC 12** and the gui half of **AC 15**: the seat PASS (2026-10-17, `e098f71`) predates this patch pass. It ran on
  a save that is now refused by design, so a re-sit must start from a fresh world.
- **AC 10** and the gui half of **AC 8**: headless only.
- **AC 13** happens at merge.

- [ ] [Review][Patch] **The "never below him" let-go clause is pinned by no test** [`crates/sim-core/src/lib.rs:1380`]
  (acceptance, MED, RAN; REWORK of row 4)
  - Deleting `|| (clearing && cell.z < pos.z)` keeps the whole scenario suite green, 83 of 83, and no
    `12-11.sh` row targets it.
  - The cancel fixture refuses every side, so the stone always lands on his own tile.
  - `astar_neighbours(pos)` includes ramp-down cells. A miner on a pit step who loses his job mid-clear (cancel,
    vanished job, trade change) could drop the stone one level down into the opened step, breaking D6, AC 4 and the
    spine bullet, and nothing would notice.
  - Fix: a fixture where the cancelled miner has a free lower neighbour and every same-level side refused, RED with
    the clause deleted, plus a mutation row.
- [ ] [Review][Patch] **The story record still says "no new saved state"**
  [`_bmad-output/implementation-artifacts/12-11-nothing-floats.md:22`, `:351`] (acceptance, LOW, read)
  - Since `31c0b16`, `SavedDwarf.clearing` is a required field by Wolf's ruling (b).
  - Correct both lines. The guardrail line "no new saved field" should name the one exception.
- [ ] [Review][Patch] **Refresh the seat card for the re-sit**
  [`_bmad-output/implementation-artifacts/12-11-signoff/vehicle-card.md:44`, `:92-106`] (feature, LOW, RAN; three
  findings merged)
  - Add a "Known, by design" line for #187: in a staircase-plus-tree run, the last row-1 cut was refused for about
    1,800 ticks while an idle dwarf wandered. `1b1b99b` behaves the same, so this patch did not cause it.
  - Re-measure the tunnel figure. It says "all four marks dug by tick 713" on `86d60f4`, but HEAD finishes at t735.
  - Say that sitting A's one-level pile (`x 56..72, y 66..73, z 9`) holds about 22 of 32 stones, so stones left
    lying there are not an AC 3 failure. The ruling left the card's pile as it was.
  - State that the re-sit starts from a fresh world, since the kept save is refused.
- [x] [Review][Defer] **A kept clearing landing is re-checked against only part of the set-down rules**
  [`crates/sim-core/src/lib.rs:1799-1807`] (acceptance + edge, LOW, read) — deferred, pre-existing (12.11 dev)
  - The kept filter skips `walls_in_a_dwarf`, the target column, `job_positions`, marked cells and `z < pos.z`. A
    landing that turns illegal between lift and set-down is used anyway: a dwarf steps into a pocket it closes, or
    Wolf marks the cell.
  - simd validates neither `landing` nor `drop_from`.
  - The sweeps found no violation.
- [x] [Review][Defer] **simd's mid-clear positive control covers only the carry tick**
  [`crates/simd/tests/serve.rs:3138`] (edge, LOW, RAN) — deferred
  - A validator that refused the walk-to-stone or lift phase would stay green.
  - The edge probe loaded a real save at every tick of a clear, and 0 of 73 were refused, so there is no defect today.

After triage, `scripts/reap-build-caches.sh --tmp-only --force` reaped 19 `/tmp` directories (37.3 GB listed, 20.5 GB
free space reclaimed), run 1's leftover layer caches included.

## Dev Notes

### Scope guardrails (do NOT)

- No cave-ins, and no falling terrain of any kind (AC 7). Emitters never fall (D4 prevents the case).
- No protocol, tui or client-core change; the only bridge change is D7; no new saved field but `SavedDwarf.clearing`
  (review run 1, ruling (b)); no new command or key.
- No fall animation in the gui; items keep snapping to their wire position.
- No change to the channel rule, the dig rule, the cut rule or the pile fill order beyond Q1, Q2, D4, D6.
- The miner never carries to the pile, never claims a haul job and never changes trade (D6).
- No swap (12.9 Task 0 Q1b). #186, #187, #189, #199 stay where they are.

### What already exists (build on it)

- Dwarf gravity: `settle` (`lib.rs:1909-1952`), one level per tick, landing beside a taken cell.
- The drop search: `nearest_free_cell` (`lib.rs:1232-1255`), called by `release_claim` (`:1257-1316`) and
  the channel landing (`:1883-1888`) with 12.9's refusals.
- `walls_in_a_dwarf` (`lib.rs:812-824`, #201: only a NEWLY stranded dwarf counts).
- Haul jobs are already by item id and read the live `Pos` every tick (`lib.rs:1164-1200`); the only thing
  keeping a hanging item's job dead is the `is_standable` gate at `:1194`.
- `creation-prototype.patch`: a throwaway fall in `settle`, Q1(b) and Q2(a), plus `probe_floats.rs`.

### Key decisions & traps

1. **Gravity alone breaks the way down.** A block that falls into the step blocks the staircase (17 of 24).
   Q1 exists for this. The prototype's 24 of 24 moved the stone without a walk; D6's walk changes timing,
   so the sweeps are the judge, not the creation numbers.
2. **`nearest_free_cell` checks `from` first and climbs ramps.** Your `refused` must reject a blocked
   `from` and any cell above the start, or a fallen stone walks back up to the rim (the prototype's dig
   probe landed it at `[62,62,9]`, the level it fell from).
3. **The dig's own stone and the fallen item want the same cell.** Dig under a lying stone: the dig spawns
   its stone on the target, and the lying stone falls onto that cell. D2 sends it beside.
4. **Order in the tick.** `execute_jobs` runs before `settle` (`lib.rs:2146-2157`), so a spawn and its fall
   land in the same tick and no delta shows a hanging item. Logs from Q2(a) spawn at the old base, over air,
   and fall the same way.
5. **The cage sweep's 5 new freezes are unexplained.** A changed landing changes every later trajectory
   (12.9's sweep is chaotic in this sense), but "unexplained" is not "harmless". Dump before you pin.
6. **The gui picks the clip from `job` before `carrying`** (`dwarf_clip`, `gui/src/project.rs:464-479`):
   `Work` on `Dig`/`Channel` is the swing whatever he holds. That is why D7 changes what the WIRE says
   rather than the clip order: reordering would still swing through the 5-tick lift (nothing carried yet).
   Any reader that keys on `job == haul` meaning "a hauler" must now also check `profession`.
7. **Old mutation anchors.** Rewording the NOTEs at `lib.rs:1189`, `:1941`, `:1951` may break old rows;
   `audit-mutations.py` blocks the commit until they are re-pointed.

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs`, `crates/sim-core/tests/scenario.rs`
- UPDATE `crates/gui/src/project.rs` (`sim_will_keep`, chip despawn), `crates/gui/tests/headless.rs`
- UPDATE `crates/simd/src/main.rs` (save validation, Task 3), `crates/simd/tests/serve.rs` (comment + D7 test), `ARCHITECTURE-SPINE.md` (2026-08-09), old mutation tables per Task 7
- NEW `mutations/12-11.sh`, `12-11-signoff/vehicle-card.md` (if Q3)
- EXISTING (creation) `12-11-signoff/floats_wire.py`, `12-11-signoff/creation-prototype.patch`
- UPDATE `crates/simd/src/bridge.rs` (D7)
- `protocol`, `tui`, `client-core`: no change.

### Previous story intelligence

- 12.10b: the channel lands its block up top, never below the worker (`4df25a3`); a cut that cages a pit is
  undone and retried; median 11 floating blocks per card staircase, deferred here.
- 12.9: every uncarried item blocks its cell; never drop under a dwarf; the cage sweep over seeds `0..=16`
  pins frozen dwarves and unfinished runs. Disclose every moved pin.
- Build probes and wire daemons in a scratch worktree, run daemons outside the repo (`simd` writes
  `frostvein.save` in its cwd), kill them by PID.

### References

- `epics.md:2729-2755` (Story 12.11), `:2422-2434` (standing ACs), FR50/FR52 (`epics.md:122-124`); #135,
  #180, #154; `deferred-work.md:2649-2651, 2664`
- `lib.rs:1164-1200` (`work_positions`), `:1232-1316` (drop search, `release_claim`), `:1753-1775` (cut),
  `:1778-1906` (dig, channel), `:1909-1952` (`settle`), `:2519-2549` (designation filters), `:3416`
- `scenario.rs:2933-3080` (cage sweep), `:3441` (staircase sweep), `:3526` (one-wide sweep), `:1101`
  (walking skeleton)
- `gui/src/project.rs:1079-1101` (`sim_will_keep`), `:2131-2160` (chips), `:2879-2892` (item snap)

## Verification

**Instrument (Task 7), daemon-judged, against a FRESH release `simd` run from outside the repo:**

```bash
simd 7461 &   # fresh, DEFAULT_SEED; one daemon per run; cwd with no frostvein.save
python3 _bmad-output/implementation-artifacts/12-11-signoff/floats_wire.py 7461 12000
```

- **Deliberate RED (observed at creation, 3 of 3 fresh daemons on `56bfbe5`):** `item 10 floating at
  [62, 62, 9] since tick 109` ... `items floated >= 30 ticks: 9; floating at the end: 9`, `tree: base [67,
  53, 12] over air`, `FLOATS RED`, exit 1. Restore: none (main is the RED).
- **GREEN observed at creation on the prototype (3 of 3):** `levels worked: 4 of 4`, `items floated >= 30
  ticks: 0; floating at the end: 0 []`, `tree: felled or supported`, `items on the pile: 22 of 32`,
  `FLOATS OK`.
- **Required of dev:** 5 of 5 `FLOATS OK` on the branch, with every count line pasted; one `FLOATS RED`
  with row "fall removed" applied (Task 7). `way_down_wire.py` 5 of 5 `WAY DOWN OK`.
- **Sim:** `cargo test -p sim-core --test scenario a_two_wide_staircase` prints stalled delays and float
  counts on failure; required 24 of 24 with 0 floats.

**Seat (AC 12, if Q3 keeps it), in the seat's launch form:**

- WSL: `simd 7451`, and in a second WSL terminal `tui 7451 --z 8` for parity.
- PowerShell: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`.
- Follow `12-11-signoff/vehicle-card.md` (Task 8).

## Dev Agent Record

### Agent Model Used

- Orchestrator + verifier: Claude Opus 5.5 (`claude-opus-5-5`).
- Codex `gpt-6-sol` / high (run banner), launched from the stale skill-config default WITHOUT asking Wolf which agent to use (the orchestrator's miss). It committed Task 1 (`11b7925`) and wrote a first pass of Tasks 2-8, uncommitted, before its usage limit cut it off; the work was pinned at `refs/wip/12-11-run1`.
- Agent A (Claude Sonnet 5.5, `claude-sonnet-5-5`), Wolf's usual pick (as in 12.2-12.10b): took the Codex tree over and finished, fixed and committed Tasks 2-6; then the seed-12 fix attempt (reverted, see below).
- The orchestrator: Task 7 (the mutation table and every mutation run, every wire run, the deliberate RED, the emitter-landing test), Task 8 (seat card rewritten, its tunnel cells found and checked live), Task 9 (spine, full gate).

### Debug Log References

Task 1 focused RED runs against unchanged production code (baseline `56bfbe5`):

```text
a_loose_stone_falls_to_standable_ground_in_one_tick: left Pos { x: 61, y: 62, z: 9 }, right Pos { x: 61, y: 62, z: 8 }
a_channel_miner_carries_its_target_stone_aside_before_cutting: channel miner never carried the target stone
a_dig_under_a_stone_does_not_leave_it_floating: dig left floating items: [(Id(10), Pos { x: 61, y: 62, z: 9 })]
digging_hillside_tree_ground_fells_the_tree: tree base remains over the dug ground; left Some(Solid(TreeTrunk)), right Some(Empty)
a_miner_clears_a_four_mark_tunnel_without_a_pile: miner stopped before the fourth mark; remaining [(Pos { x: 68, y: 66, z: 9 }, Dig), (Pos { x: 69, y: 66, z: 9 }, Dig), (Pos { x: 70, y: 66, z: 9 }, Dig)]
a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay: floating items by delay: [(0, 8), (1, 8), (2, 8), (3, 6), (4, 4), (5, 5), (6, 5), (7, 5), (8, 5), (9, 9), (10, 9), (11, 10), (12, 8), (13, 8), (14, 8), (15, 8), (16, 8), (17, 8), (18, 9), (19, 8), (20, 8), (21, 8), (22, 8), (23, 6)]
```

Tasks 2-3 busy-crew stall and the cage sweep, read by dump and per-item trace (Claude Sonnet 5.5):

- `a_busy_crew_never_shares_a_tile_and_still_works` stalled on its last channel mark (68,65,9) with `retry_after` pushed forever. The refusal was the existing wall-in check: both miners stood in the pit, its west exits were clogged by 12 loose blocks and the cut removes the east ramp, the one exit left. This hazard is older than the story: on `11b7925` with no change, the same fixture started 9..15 ticks later already leaves one mark, and with only the fall rule it fails at delays 11..15. With the clearing step on, three more faults made it systematic (all fixed in `6e4f9ea`): a miner re-picked the NEAREST stone every tick and turned back between two equidistant stones (seed 8 ping-pong); the set-down was re-planned when he stood on his own landing cell on the way to its side, so he walked back and forth (seed 1, two carriers); and a stone set down on another channel mark was lifted again or dropped into its step. The last is the one NOTE'd refusal in `refused` (a marked cell, the own target included). Without it the cage sweep freezes eight runs, not five.
- Cage sweep `no_dwarf_is_caged_by_stones_over_the_probe_seeds` after the fixes. Frozen (was `[(10, 4, 4)]`): `(8,4,4) (10,6,0) (12,4,1) (12,4,4) (16,6,3)`. Each was read with a per-item trace (spawn, set-down, fall, with the dwarf and whether he was clearing): all are the #186 shape, an idle dwarf whose other same-level sides are rock, a drop or a loose block, with no mark and no claim left. `(8,4,4)` [69,67,7]: a block he set down himself at t1204 (clearing), a channel block at t1428, after he walked in at t1377. `(12,4,4)` [69,65,10]: his own clearing set-down at t1029 took one of two exits; a later cut took the other. `(12,4,1)` [66,60,10]: a two-cell pocket `{[66,60,10],[66,61,10]}` bordered by three blocks, two set aside by ANOTHER miner while clearing (t565, t614) and one a hauler's drop (t180); he walked in up a ramp at t630, and a CHANNEL CUT of `[66,61,10]` at t720 closed his last same-level exit (the opened step keeps him unstranded, so the cut check allows it). Closed by a cut, not a placement. `(16,6,3)` [99,62,16]: his own set-down (t3420) and a channel block (t3471) after he stopped at t3408. `(10,6,0)` [62,78,28] stood 521 ticks with nothing beside it and moved again at t3996: a pause, not a cage. No stone landed on a dwarf (the sweep's first assert). RULED (Wolf, 2026-10-16): `(12,4,1)` stays pinned. A fix was tried first (generalising `closes_last_level_exit` to every open side of the placement cell, marks counted as no exit, three variants): each one that freed `(12,4,1)` left at least one run permanently unfinished (seed 13 width 6 with its last mark `[98,60,22]` sealed by set-aside blocks, #182 mode ii; seeds 1 and 6 in the third variant), so all were reverted. The cut case is recorded on #186.
- Unfinished (was none): `(6,6,2) (11,6,3) (12,6,6)`. SLOW, not stuck: the same runs to 16,000 ticks finish at t4,080, t4,350 and t4,444. A clearing miner walks a block aside before he cuts.
- Mutation checks run by me after the fix was committed (each restored by re-editing the same text): fall removed (`settle` skips every item) kills `a_loose_stone_falls_to_standable_ground_in_one_tick`, `a_dig_under_a_stone_does_not_leave_it_floating` and the lib test `a_stone_on_unstandable_ground_falls_while_its_haul_job_remains`, and does NOT kill the two-wide staircase sweep (with the miner clearing, nothing is left hanging on the staircase, so its float counter does not see the fall rule). Miner waits (`clearing_item` forced `None`) kills `a_channel_miner_carries_its_target_stone_aside_before_cutting`, `a_miner_clears_a_four_mark_tunnel_without_a_pile` and `a_hauler_holding_the_lifted_stones_job_lets_go_and_the_stone_is_delivered_later`, and does NOT kill either staircase sweep. Q2 reverted (no `fell_tree` in the Dig arm) kills `digging_hillside_tree_ground_fells_the_tree`; `digging_a_marked_trees_ground_releases_its_woodcutter` survives it (a cut on a standing tree also releases its woodcutter). Hauler keeps the job at lift kills the hauler test. Bridge reporting `Dig`/`Channel` while clearing kills `a_clearing_miner_reads_haul_on_the_wire_and_his_own_job_before_and_after`. Save validation refusing a clearing miner kills `a_save_taken_mid_clear_loads_and_the_clear_finishes`. Landing on a dwarf allowed kills `a_fallen_stone_never_lands_on_a_dwarf`; "never higher" dropped kills `a_fallen_stone_never_lands_above_where_it_started`; both SURVIVED the whole suite before those two tests were written. Set-down allowed in the target's column survives alone (the marked-cell refusal covers the own target); with both clauses dropped `a_clearing_miner_never_sets_the_stone_down_in_the_target_column` fails (the mark never completes). The marked-cell clause alone dropped is killed by the cage sweep (eight frozen runs). Dig filter dropped kills `a_dig_mark_is_refused_beneath_an_emitter_only`; the gui `sim_will_keep` dig clause dropped kills `the_channel_preview_skips_an_emitters_tile_and_cells_below_z_two`; the chip clause dropped kills `channel_under_debris_removes_chips_above_the_opened_floor`.

Task 7 (orchestrator), on `2fb5f84` built release in a scratch worktree (`simd -p simd --release`, cwd inside the worktree), each daemon from a fresh scratch dir, at most 4 at a time:

```text
floats_wire.py, 5 of 5 (ports 7461-7465), every block identical:
levels worked: 4 of 4
items floated >= 30 ticks: 0; floating at the end: 0 []
tree: felled or supported
items on the pile: 22 of 32
FLOATS OK

way_down_wire.py, 5 of 5 OK (7471-7474, 7476), e.g.:
level 0: worked by tick 473 (10 channels) / level 1: 1529 / level 2: 1855 / level 3: 2099
deepest dwarf z 5 (camp floor 9; the way down needs 5 or lower)
back up: dwarf 4 at tick 1987
WAY DOWN OK
(port 7475, run beside the mutation build, printed `WAY DOWN BROKEN: the daemon closed the connection at tick 5433`;
its daemon log says `client delta queue full; disconnecting client`, the slow-reader overload, NOT a sim result.
Re-run alone on 7476: WAY DOWN OK, level 3 at 2376.)

Deliberate RED, row "fall removed" applied in the scratch worktree only:
levels worked: 4 of 4
items floated >= 30 ticks: 3; floating at the end: 3 [[67, 53, 12], [67, 53, 12], [67, 53, 12]]
FLOATS RED
```

The RED is carried by the felled tree's three logs alone: with the miner clearing his own targets, no staircase block floats even with the fall removed (agent A's finding for the sim sweep, now seen on the wire). Without the tree leg the instrument would be blind to the fall rule.

`mutations/12-11.sh` (16 rows), `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh`, after commit, alone: 15 KILLED on the first run; row 2 (a fall may land on an emitter's tile) SURVIVED: no test pinned it. NEW `a_fallen_stone_never_lands_on_an_emitters_tile` (a stone over the campfire): green, and row 2 re-pointed at it, re-run alone: KILLED. Final: every row KILLED. Killers: 1 `a_loose_stone_falls_to_standable_ground_in_one_tick`, 1b `a_dig_under_a_stone_does_not_leave_it_floating`, 2 `a_fallen_stone_never_lands_on_an_emitters_tile`, 3 `a_fallen_stone_never_lands_on_a_dwarf`, 4 `a_fallen_stone_never_lands_above_where_it_started`, 5 `a_miner_clears_a_four_mark_tunnel_without_a_pile`, 5b `a_channel_miner_carries_its_target_stone_aside_before_cutting`, 6 serve `a_clearing_miner_reads_haul_on_the_wire_and_his_own_job_before_and_after`, 7 `a_clearing_miner_never_sets_the_stone_down_in_the_target_column` (column + marked-cell clauses; the column alone is equivalent: **FALSE for a dig, corrected 2026-10-18**, see row 7c in the review run 1 patch pass), 7b the cage sweep (marked-cell clause), 8 `digging_hillside_tree_ground_fells_the_tree`, 9 `a_dig_mark_is_refused_beneath_an_emitter_only`, 9b gui `the_channel_preview_skips_an_emitters_tile_and_cells_below_z_two`, 10 gui `channel_under_debris_removes_chips_above_the_opened_floor`, 11 `a_hauler_holding_the_lifted_stones_job_lets_go_and_the_stone_is_delivered_later`, 12 serve `a_save_taken_mid_clear_loads_and_the_clear_finishes`. `audit-mutations.py`: 894 rows, every literal matches.

Task 8 (orchestrator): Codex's draft card named a tunnel at `x 67..70, y 66, z 9` that was never checked (the scenario test builds its tunnel with `set_tile`). A snapshot search for a natural one-wide 4-cell run (solid, solid floor, solid non-tree sides, reachable from the camp) found none in rock at z 9; the nearest is a snowbank west of the camp, `[40..37, 63, 13]` (stone, then three snow; soil floor). Checked live on the final build (`2fb5f84`, fresh daemon, NO pile, `fast4x`, one mark at a time): marks dug by ticks 340, 479, 574, 735, a miner reported `haul` while carrying in 78 deltas.

Task 9 (orchestrator), full gate on `40195d5`, `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/gate.sh`:

```
cargo fmt --check           ok  0s
cargo clippy -D warnings    ok  2s
cargo test                  ok  517s
cargo test (pixel guards)   ok  3131s
tui has no sim-core edge                ok
client-core has no sim-core edge        ok
gui has no sim-core edge                ok
metrics ledger tests        ok  1s
bench tests                 ok  20s
mutation tables still apply ok  9s
GATE GREEN  3689s
```

Seat finding (Wolf, 2026-10-16, after the gate above): "miners are throwing stones quite wildly".
A probe of the seat card's staircase (pile placed, delays 0, 3, 7, 12) on `3d9488b` found 1-3 stones per
run moved 4-5 cells and 2-3 levels UP in ONE tick, from the pit floor onto the rim, the miner still in
the pit. Mechanism (instrumented): the miner lifted, the next tick his set-down search found a cell
(usually on the rim) but no route to its side, because the route was CLOSED TO THE OTHER DWARVES in the
pit (207 of 251 failures: "from None", a route existed ignoring dwarves); he retried, and
`release_claim` dropped the stone on the nearest free cell at ANY distance. A second cause: a kept
landing was never re-searched once a stone closed its only side. No test caught it: every test read
where stones end up, none how far one moves in a tick.

Wolf ruled (AskUserQuestion) "Both guards", then, told the two causes fix it alone and that no sweep
reaches the guards, "Fixes + all three guards", each with its own fixture:
- Cause 1: the set-down route runs through the crew, as every job walk does (`clearing_set_down`).
- Cause 2: a kept landing with no reachable side is searched again.
- Guard 1: he lifts only with a set-down (else the job retries, the stone stays).
- Guard 2: a clearing miner lets go only beside himself (`astar_neighbours`), else on his own tile.
- Guard 3: a miner holding his stone whose set-down is taken waits and looks again, not retry. Guard 1
  plus guard 2 alone stalled delays 7 and 12 and caged seed 13 (a stone dropped under the miner, the
  12.9 rule); guard 3 alone held seed 13's miner still for good until cause 2 was fixed.
- The set-down search moved into one helper, `clearing_set_down`, used before the lift and while carrying.

Tests (`86d60f4`): NEW `a_carried_stone_is_always_set_down_beside_its_carrier` (the staircase, 24 delays,
every carrier, every tick): RED on `3d9488b` from delay 0, GREEN on main `56bfbe5` and on the fix. Its
first version left its record stale over the designation delays and so blamed haulers for jumps they
never made (corrected in `86d60f4`; haulers set stones down beside them on main too). NEW, one per guard,
in a walled corridor (`walled_clearing`): `a_miner_with_nowhere_to_set_a_stone_down_never_lifts_it`,
`a_miner_who_lets_go_mid_clear_sets_the_stone_down_beside_him` (his trade changes while he carries; the
old drop landed it 3 cells off), `a_miner_whose_set_down_side_is_taken_keeps_the_stone` (a save reloaded
with a hauler on his only set-down side).

Cage sweep re-pinned (each new run read by dump): ONE_LEVEL_ISLANDS `[(10,6,0),(12,4,1),(12,4,4),(12,6,3)]`
(was five: (8,4,4) and (16,6,3) are freed; (12,6,3) is new, idle at [64,69,10] with no item beside him,
his one free side 4 cells from his home, past WANDER_RADIUS: the #186 shape). UNFINISHED
`[(6,6,2),(11,6,1)]` (was three); run to 16,000 ticks they finish at t4,080 and t4,100, slow not stuck.

```text
mutations/12-11.sh, 21 rows, env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh, after commit, alone:
every row KILLED. New: 13 route closed to dwarves (two-wide staircase sweep), 14 a stale landing kept
(cage sweep), 15 lift with nowhere to go, 16 let-go at any distance, 17 give up when the set-down is
taken (each its own fixture, each read on its own assertion). Rows 7 and 7b re-pointed (the clauses
moved into `clearing_set_down`, 12 spaces shallower).

Wire on 86d60f4 (release, scratch worktree, fresh daemon each, at most 4 at a time):
floats_wire.py 5 of 5 FLOATS OK (level 3 at tick 2714, 0 floated, 22 of 32 on the pile)
way_down_wire.py 5 of 5 WAY DOWN OK (level 3 at tick 2192, deepest z 5, back up at tick 1388)
(the first 7481 run printed FLOATS BROKEN: no site; its daemon could not bind, the port held by an orphan
dev-phase `simd-nofall` daemon, so the script read that old world. Twelve orphan dev-phase daemons
stopped; re-run on the new build: FLOATS OK.)
Seat card sitting B (tunnel, no pile): marks dug by ticks 318, 457, 552, 713; a miner read `haul` while
carrying in 78 deltas; the farthest a carried stone landed from its carrier: 1 cell.
```

Full gate on `86d60f4`, `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/gate.sh`:

```
cargo fmt --check           ok  0s
cargo clippy -D warnings    ok  3s
cargo test                  ok  616s
cargo test (pixel guards)   ok  3243s
tui has no sim-core edge                ok
client-core has no sim-core edge        ok
gui has no sim-core edge                ok
metrics ledger tests        ok  1s
bench tests                 ok  20s
mutation tables still apply ok  9s
GATE GREEN  3901s
```

Seat finding (Wolf, 2026-10-16 15:04 UTC, re-sitting the card): "dwarf still got in to the loop of digging",
at 52,64,7. His save (tick 7640) is kept at `~/frostvein-saves/12-11-dig-loop.save`. Loaded offline
(`World::from_save`, release probe, scratch worktree) and run 3000 ticks on `97b93ac`: the pit is FROZEN,
0 of 20 marks done and no stone moved; the pile is full (21 of 21), so no hauler moves anything. Two miners,
two shapes, both caged:
- Dori (id 4), in the two-cell pocket 52,63,8 / 52,64,8, is shut in by ONE stone, 43 on 53,63,8 (without it
  he reaches 6118 cells and the camp). Stone 43 moved there from 55,63,8 (haul job 65's creation target,
  t5466); what moved it is not in a save. His loop is the #187 shape: ten swings on channel 67 or 69, no
  cell for the block, undo, retry, alternating. Clearing never fired: it looked only at stones on the
  TARGET's faces, not at his own way out. This is Task 3's "a dwarf caged by a moved item: fix it".
- Bifur (id 2), at 54,64,6, is caged by TERRAIN: a two-cell hole with no ramp up (stone 46 in the other
  cell; without it he reaches 2 cells). Loop: a 5-tick lift, no set-down, let go (the seat guard 1,
  correct). Likely a Dig of 53,64,6 took the floor out from under his only exit (inferred). Filed #203.

Wolf ruled "fix it now", then "land and file". Fix (`44ca3fd`): `caging_stone`. When a miner's dig/channel
faces are clear, he clears the lowest-id stone that alone shuts his pocket (<= 64 cells) in, with the same
lift/carry/set-down as a face stone. NOTE in the helper: a pocket over 64 cells, or one shut by two stones,
is not seen. NEW `a_miner_caged_by_one_stone_carries_it_out_and_cuts` (Dori's shape built from tiles, no
pile): RED on `97b93ac` (the miner never lifts), GREEN on the fix. Replay of the save on the fix: Dori lifts
43 at t7646, cuts at t7791, 9 marks done by t8948, and Bifur climbs out through a cut Dori opens. Then at
~t8955 both miners stand head-on in a two-cell pocket that FOUR stones seal (42, 43, 52, 53; 43 is Dori's
own set-down): the noted no-swap case, and a full pile piling stones round the pit. Filed #204.

Cage sweep re-pinned: UNFINISHED `[(11,6,1)]` (was `[(6,6,2),(11,6,1)]`; seed 6 now finishes inside 4,000).
ONE_LEVEL_ISLANDS unchanged. Every other sim-core test unchanged. Mutation row 18 (the trigger returns
`None`) KILLED, run alone after commit.

Full gate on `44ca3fd` (plus row 18 and this record), `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/gate.sh`:

```
cargo fmt --check           ok  1s
cargo clippy -D warnings    ok  3s
cargo test                  ok  632s
cargo test (pixel guards)   ok  3019s
tui has no sim-core edge                ok
client-core has no sim-core edge        ok
gui has no sim-core edge                ok
metrics ledger tests        ok  0s
bench tests                 ok  20s
mutation tables still apply ok  9s
GATE GREEN  3693s
```

Wire on `7d1e77c` (release, scratch worktree, fresh daemon in an empty directory each, at most 4 at a time,
each daemon's own bind checked in its log): floats_wire.py 5 of 5 FLOATS OK (22 of 32 on the pile);
way_down_wire.py 5 of 5 WAY DOWN OK (deepest z 5, back up at tick 1276, was 1388 on `86d60f4`).

**Review run 1 patch pass (2026-10-18, fresh session, Opus 5.5 by Wolf's pick).** Base `38a66bb`. Landed in five
groups: the save format first (it changes the file), then the clearing semantics, the tests, AC 3 and the docs.

REDs, each on the unpatched code:

- `a_world_saved_at_every_tick_of_a_clear_loads_and_steps_identically`: `saves taken mid-clear diverged from the
  never-saved world (save tick, first diverged tick): [(48, 53), (49, 53), (50, 53), (51, 53), (52, 53), (64, 76),
  ... (80, 81)]`, the review's windows exactly.
- `loading_refuses_a_pre_12_11_save_without_dwarf_clearing`: `a pre-12.11 save without clearing was loaded`, and
  this RED was AFTER the field was added with no `#[serde(default)]`. serde_derive reads a missing `Option` field as
  `None` regardless, so Wolf's (b) would not have held. `#[serde(deserialize_with = "Option::deserialize")]` makes
  the field required (NOTE in `save.rs`).
- The four serve refusals: the hauler and cut-job saves were refused by the old generic text (`carries item 10
  without holding its haul job`); the other-stone and two-holder saves LOADED (the daemon then logged
  `client delta queue full`, which comes only after a load).
- `a_miner_whose_channel_is_cancelled_mid_clear_sets_the_stone_down_beside_him`: `the stone landed at Pos { x: 61,
  y: 62, z: 9 }, more than a cell from the miner on Pos { x: 63, y: 62, z: 9 }`. Its first fixture (a hauler shut
  in a pocket, the twin of the trade-change test) did NOT go RED: without the job the free mark cell beside him
  was the nearest anyway. The hauler now stands on the mark, his only side.
- `a_miner_reads_as_clearing_only_while_he_lifts_or_carries`: `tick 8: dwarf Id(2) reads as clearing, holding
  nothing and not lifting`.
- `a_miner_who_sets_his_stone_down_is_walking_not_working`: `tick 81: set down while working`.

`JobState` is read by no sim logic (only the readers and tests), so Walk at the set-down moves no pin.

**Closure table** (both sides; REWORK = re-closes an earlier round's half-closed fix):

| Item | Fix written for | Then tested from | Pre-existing-state fixture | Rework |
|---|---|---|---|---|
| 1 save `Clearing` (HIGH) | a save taken mid-clear | the OLD world: a pre-12.11 save is refused; and the never-saved world at every tick of a real clear | `loading_refuses_a_pre_12_11_save_without_dwarf_clearing` (a current save with `clearing` stripped from every dwarf's JSON; positive control: the same save unstripped loads); `a_world_saved_at_every_tick_of_a_clear_loads_and_steps_identically` (DEFAULT_SEED, stone on the [61,62,9] channel mark, saved at every tick from claim to completion) | no |
| 7 simd carry rule (MED) | the refusal side | the ACCEPT side: a real sim-made mid-clear save still loads and finishes (`a_save_taken_mid_clear_loads_and_the_clear_finishes`, unchanged, green); a live world never makes two holders (`claim_jobs` skips carried stones, so a carried stone's haul has no work set) | `mid_clear_save()`: DEFAULT_SEED stepped to a real lift, then edited to a hauler carrier, a cut job, another stone, a held haul job | no |
| 4 cancel drops anywhere (MED) | the job-gone path (cancel, vanished job) | the job-held path: `a_miner_who_lets_go_mid_clear_sets_the_stone_down_beside_him` (trade change) unchanged and green; row 16 re-pointed and still kills it | `a_miner_whose_channel_is_cancelled_mid_clear_sets_the_stone_down_beside_him`: miner lifted in the walled corridor, then a hauler put on his only side by save | **REWORK** (Wolf's seat guard `fab02df` was closed for the job-held path only) |
| 5 `haul` before the lift (MED) | the empty-handed walk | the lift run still reads as clearing (row 21b), the carry reads as clearing, and serve's D7 wire test (channel, haul, channel) unchanged and green | `a_stone_on_a_channel_mark()`: DEFAULT_SEED, a stone on a one-cell channel mark at the staircase site (the review probe's shape) | no |
| 6 pick swing after set-down (MED) | the set-down tick | his real channel work afterwards still reads `Work` (`a_channel_miner_carries_its_target_stone_aside_before_cutting` green) | same `a_stone_on_a_channel_mark()` | no |
| 2 AC 3 / wire pile (MED) | a fallen stone's delivery | the OLD world: a haul job made while the stone lay on its ground, target = that ground; and the wire with the OLD one-level pile goes RED | `a_fallen_stone_with_a_haul_job_is_delivered_to_the_pile` (job asserted made on the ground and unheld, then the floor removed); scratch copy of `floats_wire.py` with the pile at cz only: `items on the pile: 22 of 32`, `FLOATS RED` | **REWORK** (12.10b review deferred "resize at 12.11") |
| 8 no-throw sweep vacuity (LOW) | nothing carried / a level never worked | green on the real crew, 24 delays: every level worked, carries > 0 every delay | rows 23 (the reader goes blind to carrying: fires at `no stone was ever carried`) and 23b (no claims: fires at `level 0 was never worked`) | no |
| 9 row 7 for a dig (LOW) | the column clause alone, dig | the channel side: row 7 (channel fixture) still kills | `a_clearing_miner_never_sets_the_stone_down_on_his_dig_targets_top`: stone on the dig's one face, ramp floors, the target's top the nearest cell only the column refuses | **REWORK** (corrects 12.11 dev's "the column alone is equivalent") |
| 3 (12,4,4) pin (LOW) | the record | n/a: no code; Wolf's (a) | recorded beside `ONE_LEVEL_ISLANDS`; #186 comment 6246215160 | no |
| 10 spine bullet (LOW) | the record | n/a: no code | `caging_stone` and the three seat guards named; `haul` from the lift to the set-down | no |

**Mutations** (`mutations/12-11.sh`, rows 19-24b and 7c/16b; run after commit with `env -u NO_COLOR
RUST_TEST_THREADS=1 scripts/mutate.sh` from a scratch table of only the new rows; each kill read at its panic line,
not only the test name): 19 (load drops the clearing) `save_load.rs` divergence assert; 19b (the
`deserialize_with` removed) simd `loading_refuses_a_pre_12_11_...`; 20/20b/20c/20d (hauler clears, cut job clears,
carries another stone, two holders accepted) each its serve refusal; 16b (let-go keyed on the job again) the
cancel test's landed assert; 21 (named while walking) and 21b (named only while carrying) both sides of
`a_miner_reads_as_clearing_...`; 22 (Work left at set-down); 23 and 23b as above; 24 (the fall removed) at the AC 3
test's fall assert; 24b (the haul reads the stale job target) at its delivery assert. Row 16 re-pointed at the new
seam. **First run: 13 of 14 KILLED, 7c SURVIVED.** Read by trace in a scratch worktree: in the first dig fixture
the cell beyond the target's top was a miner's only same-level exit, so `closes_last_level_exit` refused the top
before the column clause could. `beyond` and `aside` got second exits and the fixture moved one cell west off
dwarf 4's spawn (`a2876db`); 7c then fails at `tick 46: the stone lies at Pos { x: 63, y: 62, z: 10 }, in the dig
target's column`. Also on the first run, 23b's first payload (no clearing at all) was killed by the THROW assert,
not the level assert: haulers worked every level anyway. Re-pointed to "no dwarf ever claims". Re-run of 7c, 23b,
24b: all KILLED at their own lines. `audit-mutations.py`: 915 rows, every literal matches.

Wire on `c0f6c63` (release, scratch worktree, one fresh daemon per run from an empty directory, at most 4 at a
time, every daemon log free of `queue full`; sim code unchanged since): `floats_wire.py` **5 of 5 FLOATS OK**, each
`levels worked: 4 of 4`, `items floated >= 30 ticks: 0`, `tree: felled or supported`, `items on the pile: 32 of 32`.
Deliberate RED of the resized OK condition: the one-level pile copy, `items on the pile: 22 of 32`, `FLOATS RED`.
`way_down_wire.py` **5 of 5 WAY DOWN OK** (levels by 1067/1659/2179, back up at tick 1276).

Full gate on `c051bf0`, `RUST_TEST_THREADS=1 scripts/gate.sh`:

```
frostvein gate
  cargo fmt --check           ok  0s
  cargo clippy -D warnings    ok  2s
  cargo test                  ok  629s
  cargo test (pixel guards)   ok  3004s
  tui has no sim-core edge                ok
  client-core has no sim-core edge        ok
  gui has no sim-core edge                ok
  metrics ledger tests        ok  0s
  bench tests                 ok  19s
  mutation tables still apply ok  10s
GATE GREEN  3673s
```

### Completion Notes List

- Task 2: uncarried items fall after dwarves in ascending id order, to the nearest free cell from the column's first standable cell, never above the cell they fell from. The channel's dwarf landing check and item gravity share `first_standable_down_column`. The old unit test `a_stone_on_unstandable_ground_leaves_its_haul_job_queued_and_retried` is flipped to `a_stone_on_unstandable_ground_falls_while_its_haul_job_remains` (the premise "items never fall" is gone).
- Task 3: a miner whose dig/channel way is blocked by an uncarried item lifts the nearest one (kept until set down), carries it to a free cell by the 12.9 drop rule (never in the target's column, on a marked cell, a work position, below him, under a dwarf, or where it walls a dwarf in or closes an idle dwarf's last same-level exit), sets it down and returns to the same job. The trigger lives in `execute_jobs` (NOTE there). simd's save validation allows a miner carrying while holding a dig/channel job (the original refusal text is kept for every other save). `World::clearing()` feeds the bridge: a clearing miner reads `Haul` on the wire, `profession` stays `miner`. The two-wide staircase sweep is 24 of 24 with 0 floating items, the one-wide sweep and the walking skeleton pass, the no-pile 4-mark tunnel finishes (all in `cargo test -p sim-core --test scenario`, 70 passed). Wolf ruled 2026-10-16: `closes_last_level_exit` is kept everywhere as built. NOTE: the pile-drop site can change which pile cell a hauler fills. Added beyond the first pass: `closes_last_level_exit` (a clearing set-down, a fallen item, a hauler's pile drop and a released carrier's drop never take an idle dwarf's last same-level exit), disclosed in the cage pins.
- Task 4: `fell_tree` is the one helper for the Cut arm and the Dig arm (a completed dig whose cell above is a trunk fells the tree; its logs fall in the same tick). A held cut on that tree completes as the no-tree no-op and releases the woodcutter.
- Task 5: the Dig filter refuses a cell whose upper neighbour is an emitter's tile; the gui `sim_will_keep` mirrors it. The unit test `stockpile_refuses_only_when_every_cell_is_invalid` used the cell under the fire for its unrelated designate/cancel check; it now uses the rock beside it (D4 makes the cell under the fire undiggable).
- Task 6: a dirty tile that is now `Empty` despawns the dig chips of the cell above it.
- The first pre-commit attempt failed only under `NO_COLOR=1`: `ore_cells_and_ore_items_reach_the_capture_in_their_own_glyph_and_colour` expected ANSI escapes. It passed unchanged with `NO_COLOR` unset; every commit hook ran with `env -u NO_COLOR RUST_TEST_THREADS=1`.
- Commits: Tasks 2, 3 and 4 are ONE commit (`6e4f9ea`): their `lib.rs` hunks interleave (the fall rule, the clearing step and `fell_tree` share `settle`, `execute_jobs` and `release_claim`). The mid-run attempt to split them was impractical because the fast gate runs `a_busy_crew_...` and the cage sweep on the working tree. No `--no-verify` anywhere.
- `serve.rs` is NOT in the fast gate; the whole file (73 tests) was run by hand with `RUST_TEST_THREADS=2` and passed, then the two tests added afterwards were run alone.
- Seat fix (Wolf's finding, after the first green gate): a clearing miner's set-down route runs through the crew and a stale kept landing is searched again (the two causes); he lifts only with a set-down, lets go only beside himself, and waits while holding when his set-down is taken (Wolf's guards). See the Debug Log's seat-finding entry.
- Dig-loop fix (Wolf's second seat finding): a miner whose pocket one stone shuts carries that stone out (`caging_stone`). The terrain hole (#203) and the many-stone pocket on a full pile (#204) are filed, not fixed (Wolf: "land and file").

### File List

- `crates/sim-core/src/lib.rs`
- `crates/sim-core/src/save.rs` (review run 1: `SavedClearing`)
- `crates/sim-core/tests/save_load.rs` (review run 1)
- `crates/sim-core/tests/scenario.rs`
- `crates/simd/src/bridge.rs`
- `crates/simd/src/main.rs`
- `crates/simd/tests/serve.rs`
- `crates/gui/src/project.rs`
- `crates/gui/tests/headless.rs`
- `_bmad-output/implementation-artifacts/mutations/{12-5,12-7,12-9,12-10b,3-2-the-dig,3-3-the-haul-and-the-skeleton-walks}.sh` (old rows re-pointed)
- `_bmad-output/implementation-artifacts/mutations/12-11.sh` (NEW)
- `_bmad-output/implementation-artifacts/12-11-signoff/vehicle-card.md` (NEW)
- `_bmad-output/implementation-artifacts/12-11-signoff/floats_wire.py` (review run 1: pile over four levels, OK needs every item)
- `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`
- `_bmad-output/implementation-artifacts/12-11-nothing-floats.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

| Date | Change |
|---|---|
| 2026-10-18 | Code review run 2 on `5a8fbaf` (fresh session, 4 layers, diff `1b1b99b..5a8fbaf`): 9 of 10 closure rows audited CLOSED, row 4 REWORK (the 'never below him' let-go clause is unpinned: deleting it keeps 83 of 83 scenario tests green). 0 HIGH, 1 MED + 2 LOW patch, 2 defer, 8 dismissed. Stopping rule fired: no round 3; next is a fresh-world seat re-sit. 3 patches left as action items. Status -> in-progress. |
| 2026-10-18 | Review run 1 patch pass (fresh session, Opus 5.5): all 10 `[Review][Patch]` items landed (`31c0b16`..`c051bf0`). `Clearing` saved as a REQUIRED field (Wolf's (b)); serde read a missing `Option` as `None`, so `deserialize_with` makes pre-12.11 saves refused. simd refuses non-miner, cut-job, wrong-stone and two-holder clearing saves. Cancel mid-clear drops beside him; `haul` only from the lift; Walk after the set-down. AC 3 pinned in the sim and judged by the wire (pile over four levels, 32 of 32). Row 7's "column alone is equivalent" was FALSE for a dig (row 7c). (12,4,4) pinned under the (12,4,1) ruling, #186 noted. 16 new/changed mutation rows KILLED; wire 5/5 + 5/5 OK, deliberate RED 22 of 32. Status -> review (round 2). |
| 2026-10-17 | Seat re-sit (Wolf) on `e098f71`: "no stuck anymore with old save and no flying stones .. well we need to tweak details in later stories still most likely but now it's ok". Seat PASS; detail tuning left to later stories (#203, #204 already filed). Next: code review in a fresh session. |
| 2026-10-17 | Seat finding (Wolf): a dig loop at 52,64,7. His save froze the pit: one miner caged by one moved stone, one in a terrain hole. Fixed the first (`caging_stone`, `44ca3fd`); new test RED before, GREEN after; cage UNFINISHED pin moved; row 18 KILLED. Filed #203 (terrain hole) and #204 (many-stone pocket on a full pile). Full gate GREEN (3693 s, one test thread). Status -> review. |
| 2026-10-16 | Orchestrator: full gate GREEN on `86d60f4` (3901 s, one test thread). Status -> review. |
| 2026-10-16 | Seat finding (Wolf): miners threw stones up to five cells onto the pit rim. Two causes fixed (the set-down route closed to the crew, a stale kept landing) plus Wolf's three guards, each pinned; new no-throw test RED before, GREEN on main and the fix; cage pins moved; 21/21 mutations KILLED; wire 5/5 + 5/5 OK. Status back to in-progress until the gate. |
| 2026-10-16 | Orchestrator: full gate GREEN on `40195d5` (3689 s, one test thread). Status -> review. |
| 2026-10-16 | Orchestrator: Task 7 (16-row mutation table, all KILLED after a new emitter-landing test closed row 2; wire 5/5 FLOATS OK, 5/5 WAY DOWN OK, deliberate FLOATS RED), Task 8 (seat card rewritten, tunnel cells found and checked live), spine bullet. Wolf ruled the seed-12 pin stays (closed by a cut, #186). |
| 2026-10-16 | Wolf ruled: `closes_last_level_exit` kept everywhere as built (fall landing, clearing set-down, released carrier's drop, hauler pile drop); the pile-drop site can change which pile cell a hauler fills. |
| 2026-10-16 | Dev (Claude Sonnet 5.5): Tasks 2-6 done and committed. Clearing step made stable (kept stone, landing route, marked cells refused); busy-crew stall explained; cage pins `ONE_LEVEL_ISLANDS` and `UNFINISHED` re-pinned with per-run mechanisms; new tests for the fall rule, the hauler's let-go, the wire job and a mid-clear save. |
| 2026-10-15 | Q4 RULED (Wolf): a clearing miner is reported as `Haul` on the wire, so he looks like a hauler (D7, AC 15). No protocol shape change. |
| 2026-10-15 | Task 0 RULED (Wolf): Q1 (b) done BY THE MINER, who moves the stone aside himself and goes on digging, never to the pile; #180 folded in (D6, AC 14, Task 3 rewritten). Q2 (a). Q3 short seat. Traps found: simd's save validation refuses a carrying miner; the gui plays the dig swing for his lift. |
| 2026-10-15 | Created on `56bfbe5`. #135 reproduced: the 12.10b staircase leaves 4-10 floating blocks in 24 of 24 sweep runs; a channel under a block and a dig under a lying stone hang it with its haul retried forever; digging a tree's ground leaves the trunk on air (92 of 259 trees are exposed). Dwarves never stay on air. `floats_wire.py` written: RED 3 of 3 on main, OK 3 of 3 on a throwaway prototype. Pure gravity regresses 12.10b (staircase 17 of 24), so Task 0 Q1 is open. Defaults D1-D5. |
