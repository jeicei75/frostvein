---
baseline_commit: 56bfbe5
model: claude-opus-5-5  # session default, same as 12.1-12.10b's creation
---

# Story 12.11: Nothing Floats

Status: ready-for-dev

## Story

As the boss,
I want things above a dug-out block to come down,
so that the world never shows stones hanging or trees standing on air.

## Not stacked: branch off `main`

`main` is `56bfbe5` (PR #202 merged), clean. Branch `story-12-11-nothing-floats` carries this file,
`12-11-signoff/` and the board edit. Epic 12's standing ACs (`epics.md:2422-2434`) bind this story and are
not restated. **No wire change** (standing AC 4): an item's `pos` is resent in full on every delta
(`protocol/src/lib.rs:254-258, 296`; `simd/src/bridge.rs:80`), so a fall is a changed `pos`; a felled tree
is `Empty` tiles through `Terrain::set_tile`'s dirty set. **No new saved state**: a fall changes positions
and tiles only.

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
    tree, no debris) and the miner carrying stones out of his own way, in the gui and an attached tui.
13. #135 and #180 close with their red-then-green tests.
14. With no stockpile, a miner digs a one-wide tunnel of 4 marks into rock to the end: each stone that
    blocks his next face is carried off by him, and no hauler is involved (D6, #180). RED today: he stops at
    the first stone (`lib.rs:1142-1143`).

## Tasks / Subtasks

- [ ] **Task 1: RED tests first** (AC 1)
  - [ ] `crates/sim-core/tests/scenario.rs` (UPDATE), from the probe's tests: the channel-under-a-block case
    (AC 3, 4), the dig-under-an-item case (AC 2), the hillside tree dig (AC 5; DEFAULT_SEED `[103,2,8]` or the
    nearest such tree to the camp), the one-wide tunnel with no pile (AC 14), and a float counter added to
    `a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay` (`scenario.rs:3441`) that
    asserts 0 floating items and the AC 6 dwarf rule every tick (AC 9). Items placed off the natural flow go
    in through `to_save`/`from_save` (`SaveState.items`, `save.rs:25`), as the probe does.
  - [ ] Run each on unchanged `56bfbe5` code and paste the failing assert lines into Debug Log References.
    Commit them with `#[ignore = "RED until Task N"]`, un-ignored by the commit that greens them.
- [ ] **Task 2: items fall** (AC 2, 3, 6)
  - [ ] In `settle` (`lib.rs:1909-1952`), after the dwarves, in ascending item id: each uncarried item that
    is `Empty` over `Empty` drops to its landing (D1, D2). The landing search is `nearest_free_cell`
    (`lib.rs:1232`) from the column's first standable cell, refusing taken cells, emitter tiles, dwarf tiles,
    `walls_in_a_dwarf` and any cell above the start.
  - [ ] One helper for "the first standable cell down this column", used by the items AND the channel's
    occupant check (`lib.rs:1861-1877`, today an inline loop). Don't add a third copy.
  - [ ] `work_positions`' haul leg (`lib.rs:1189-1199`): rewrite the "items never fall" NOTE. Keep the
    `is_standable` gate: it now covers only D2's no-free-cell case.
  - [ ] Rewrite `settle`'s two NOTEs (`:1941-1942`, `:1951`) to say what is now true.
  - [ ] Flip the unit test `a_stone_on_unstandable_ground_leaves_its_haul_job_queued_and_retried`
    (`lib.rs:3416`), which pins the old behaviour; disclose it.
  - [ ] Fixtures that say "items never fall" (`scenario.rs:1120-1121`, `simd/tests/serve.rs:2155-2158`):
    fix the comment, keep the fixture.
- [ ] **Task 3: the miner clears his way (D6; Q1, #180)** (AC 4, 9, 14)
  - [ ] Make a blocked dig/channel job claimable and workable by its miner: today a job whose every work
    position holds an item has an EMPTY work set (`work_positions`, `lib.rs:1144-1147`, `:1155-1162`), so
    `claim_jobs` never claims it (reachability is checked at claim, `:479-620`). The blocking items' lift
    cells must count, and a channel's target item must be lifted before the cut. Decide where the trigger
    lives, and say so in a `// NOTE:`; no trade or claim-filter change (AD-12).
  - [ ] The lift, walk and set-down reuse the haul's mechanics (`Carrying`, `WORK_TICKS` per lift and
    set-down, `carry_items`), with the D6 landing rule. After the set-down the miner goes back to the same
    job; a channel then cuts with its target clear, so 12.10b's landing code is unchanged.
  - [ ] Every place that assumes "a dwarf carries only the stone of the haul job he holds" now also allows a
    miner holding a dig/channel job: the `debug_assert` in `work_positions` (`lib.rs:1165-1168`),
    `release_claim` (`:1257`, a cleared stone a released miner holds is dropped by its rule), and **simd's
    save validation** (`simd/src/main.rs:568-577`, which today refuses such a save). A save taken mid-clear
    loads and finishes the clear (standing AC 3's save/load gate).
  - [ ] A hauler holding the lifted item's haul job lets go when the miner lifts it (D6); pin that the item
    is later delivered to the pile (AC 3) and that no item ever has two holders.
  - [ ] The judge is AC 9 and AC 14: the staircase sweep 24 of 24 with 0 floats, the one-wide sweep, and the
    no-pile tunnel. Then the cage sweep: name the mechanism of each newly frozen run from a dump (as 12.10b
    Task 4 did) before you re-pin `ONE_LEVEL_ISLANDS` (`scenario.rs:2951`). If one is a dwarf caged by a
    moved item, fix it, don't pin it.
- [ ] **Task 4: trees, by Q2** (AC 5)
  - [ ] If (a): extract the Cut arm's felling (`lib.rs:1753-1771`) into one helper. The Dig arm calls it
    when the cell above a completed dig target is `TreeTrunk`. The logs then fall by Task 2 in the same tick.
  - [ ] A Cut mark or held cut on that tree then completes as today's no-tree no-op (`tree_of` → `None`,
    `:1772-1775`); pin that the woodcutter is released.
- [ ] **Task 5: dig filter under an emitter** (AC 8)
  - [ ] `apply_command`'s Dig filter (`lib.rs:2533-2538`): refuse the cell under an emitter's tile
    (`emitter_tiles`, `:2519`). A rect left with zero cells is a `Refusal`, as today.
  - [ ] gui `sim_will_keep` (`crates/gui/src/project.rs:1079-1089`): mirror it for `DesignateMode::Dig`.
- [ ] **Task 6: gui debris** (AC 10)
  - [ ] In `reconcile`'s dirty-tile loop (`project.rs:2131-2160`): a dirty tile that is now `Empty` also
    despawns the `DigChip`s of the cell above it. Headless test beside `headless.rs:990-1006`: a channel's
    chips, then a channel under them, leaves 0 chips at that cell.
- [ ] **Task 7: the instrument, its RED and the mutation table** (AC 11)
  - [ ] `12-11-signoff/floats_wire.py` (EXISTING, creation) is the instrument. Run it on 5 fresh release
    daemons, each started from a directory with no `frostvein.save`, at most 4 at a time. Paste each verdict
    block. Do the same with `12-10b-signoff/way_down_wire.py` (its pile was sized while blocks floated,
    `deferred-work.md:2664`; resize it only if it goes RED for want of pile).
  - [ ] Deliberate RED of the instrument on the finished branch: rerun it with row "fall removed" applied
    in a scratch worktree build and paste the `FLOATS RED` block (the creation RED was on `main`).
  - [ ] NEW `mutations/12-11.sh`, each row run alone after commit and named against its killing test: the
    fall removed (AC 2); a landing on an emitter allowed (AC 2, FR50); a landing on a dwarf allowed (AC 2);
    "never higher" dropped (AC 2); the miner waits instead of clearing (AC 4, 14); the set-down allowed in
    the target's column (AC 4); the Q2 rule reverted (AC 5); the D4 filter
    dropped (AC 8); the chip despawn dropped (AC 10). Re-point any old row your NOTE edits break
    (`scripts/audit-mutations.py`).
- [ ] **Task 8: seat card** (AC 12, if Q3 keeps it)
  - [ ] NEW `12-11-signoff/vehicle-card.md`, in the seat's launch form (12.10b's card is the template):
    the staircase at `[61..65, 62..63, 9]` with a BIG pile, and the tree at `[67,53,12]` (dig its ground
    `[67,53,11]`, workable from its own level), and a one-wide tunnel 4 long with NO pile (give the cells).
    Note the compass mirror (#169).
- [ ] **Task 9: spine and gate**
  - [ ] `ARCHITECTURE-SPINE.md` (2026-08-09, beside its 12.10/12.10b bullets): one bullet for the fall rule, Q1 and Q2 as ruled.
  - [ ] `RUST_TEST_THREADS=1 scripts/gate.sh`, full and green, before review.

## Dev Notes

### Scope guardrails (do NOT)

- No cave-ins, and no falling terrain of any kind (AC 7). Emitters never fall (D4 prevents the case).
- No wire, protocol, bridge, tui or client-core change; no new saved field; no new command or key.
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
6. **The gui swing.** `dwarf_clip` (`gui/src/project.rs:464-479`) plays the Dig swing for a miner in
   `Work` on a Dig/Channel job, carrying or not, so his 5-tick lift and set-down read as a swing; the walk
   between reads as Carry. Check it at the seat; change the clip choice only if Wolf asks.
7. **Old mutation anchors.** Rewording the NOTEs at `lib.rs:1189`, `:1941`, `:1951` may break old rows;
   `audit-mutations.py` blocks the commit until they are re-pointed.

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs`, `crates/sim-core/tests/scenario.rs`
- UPDATE `crates/gui/src/project.rs` (`sim_will_keep`, chip despawn), `crates/gui/tests/headless.rs`
- UPDATE `crates/simd/src/main.rs` (save validation, Task 3), `crates/simd/tests/serve.rs` (comment), `ARCHITECTURE-SPINE.md` (2026-08-09), old mutation tables per Task 7
- NEW `mutations/12-11.sh`, `12-11-signoff/vehicle-card.md` (if Q3)
- EXISTING (creation) `12-11-signoff/floats_wire.py`, `12-11-signoff/creation-prototype.patch`
- `protocol`, `tui`, `client-core`, `simd/src/bridge.rs`: no change.

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

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-10-15 | Task 0 RULED (Wolf): Q1 (b) done BY THE MINER, who moves the stone aside himself and goes on digging, never to the pile; #180 folded in (D6, AC 14, Task 3 rewritten). Q2 (a). Q3 short seat. Traps found: simd's save validation refuses a carrying miner; the gui plays the dig swing for his lift. |
| 2026-10-15 | Created on `56bfbe5`. #135 reproduced: the 12.10b staircase leaves 4-10 floating blocks in 24 of 24 sweep runs; a channel under a block and a dig under a lying stone hang it with its haul retried forever; digging a tree's ground leaves the trunk on air (92 of 259 trees are exposed). Dwarves never stay on air. `floats_wire.py` written: RED 3 of 3 on main, OK 3 of 3 on a throwaway prototype. Pure gravity regresses 12.10b (staircase 17 of 24), so Task 0 Q1 is open. Defaults D1-D5. |
