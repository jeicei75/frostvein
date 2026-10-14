---
baseline_commit: 015548f
model: claude-opus-5-5  # session default, same as 12.1-12.10's creation
---

# Story 12.10b: A Way Down

Status: review

## Story

As the boss,
I want a channel to open the level below, and my crew to keep working at the bottom of what they dig,
so that my miners can dig down into the mountain, not only sideways into hillsides.

## Not stacked: branch off `main`

`main` is `015548f` (PR #192 merged), clean. Branch `story-12-10b-a-way-down` carries this file,
`12-10b-signoff/` and the board edit. Epic 12's standing ACs (`epics.md:2422-2434`) bind this story and
are not restated. **No wire change** (standing AC 4): `Tile`, `Item` and every message keep their shape;
`{"ramp":"tetrahedrite"}` is already on the wire (12.10). **No new saved state.**

## Found at creation (2026-10-13, on `015548f`)

Why this story exists: at the 12.10 seat (pass 2) Wolf could not reach a bloodstone, because nothing digs a
level down, and all dwarves but one jammed in a one-wide trench (#185). He did not know what separates dig
from channel: **dig** removes a block at the miner's own level (a tunnel), **channel** is meant to go down.

- **#163 holds in source.** A channel turns the solid under its target into `Ramp(material)` and spawns the
  stone on the target (`lib.rs:1665-1680`, `:1720-1727`). On a cell already standing on a ramp it is a
  no-op (`execute_jobs_removes_a_channel_job_when_the_support_is_already_a_ramp`, `lib.rs:5090`). So a
  channel never opens anything and channels never go deeper.
- **The movement rule fixes the shape of the answer.** A vertical step is legal when the LOWER of the two
  cells stands on a `Ramp` (`astar_neighbours`, `lib.rs:874-913`). A walkable step down from `z` therefore
  needs `z-1` open (`Empty`) and `z-2` a `Ramp`.
- **Probe** (`12-10b-signoff/creation-probe.patch`: the ruled rule in a scratch copy, plus
  `crates/sim-core/tests/way_down_probe.rs`; its `PROBE_*` env toggles are probe-only, never ship them).
  DEFAULT_SEED, site `[61,62,9]` (camp `[64,64,9]`):
  - one channel: `z-1` → `Empty`, `z-2` → `Ramp(Soil)`; a miner digs `[61,63,8]` from inside the pit
    (tick 181) and stands at z 9 again by tick 191;
  - a **two-wide** staircase (rows `y 62..63`), 4 levels, big pile: z 9 → z 5, a dwarf digs at z 5;
  - drop EITHER work-position change (Task 3) and it stalls at level 1. A **one-wide** staircase jams at
    level 2 (#185). With no pile it stalls (never-drop, FR8). Skipping the wall-in check broke level 3.
  - 4 levels two wide dig **28 blocks**; one item fits per pile cell (12.9), so a 22-cell pile fills.
  - each finished staircase leaves ~3 **floating blocks**: one landed on a cell a later level opened
    under. Falling is 12.11's (`#135`); not here.
- **The jam (Wolf: fold it in).** Under the ruled rule plus Task 3, the staircase still stalls at the dig
  face, a two-by-two dead end crowded with dwarves walking in place and dug blocks:
  - `way_down_wire.py`, 5 fresh daemons: **1 of 5** OK (4 RED at level 3);
  - the deterministic probe sweep (designate each level `delay` ticks after the previous one is worked,
    `delay` 0..24): **16 of 24** OK; RED at delays 5, 6, 7, 8, 16, 21, 22 (level 3) and 20 (level 2).
  - On `015548f` the wire reads `level 0: NEVER WORKED ... the floor under [61, 62, 9] is {"ramp": "snow"}`.

### Task 0: RULED (Wolf, at creation, 2026-10-13)

- **Q1, the channel rule:** a channel **opens a step down**: the cell under the target is dug out to
  `Empty`, and the cell under THAT becomes a ramp. (Not "a hole, no ramp", not a new dig-down order.)
- **Q2, the dug block:** it lands **up top, beside the miner**, on the nearest free cell by 12.9's drop
  rule; never in the hole.
- **Q3, the jam:** **folded in.** What reaches the seat must work: the sweep 24 of 24 and the wire 5 of 5.

### Defaults taken at creation (Wolf may overrule; listed in the hand-off)

- **D1:** the new ramp keeps its own material, ore included. It was not dug out, so nothing is yielded
  twice; a later dig of `Ramp(ore)` yields the ore (12.10's ramp-dig rule). This retires 12.10's
  "channel over ore leaves `Ramp(Stone)`" and the spine's "the sim never produces `Ramp(ore)`".
- **D2:** `z-2` already a `Ramp` stays one; `z-2` `Empty` stays `Empty` (the channel opens a hole into the
  space below, with no ramp). A channel mark is refused where `z-2` is outside the world (`z < 2`).
- **D3 (#188):** a channel mark on an emitter's tile (campfire, torch) is refused at designation. Under the
  new rule it would leave the fire hanging over a hole.

## Acceptance Criteria

1. RED first: the walk-down scenario (AC 3) and the staircase sweep (AC 7) are recorded RED on `015548f`
   (the floor under a channel is a ramp) before any rule change; #185's own fixture is recorded RED.
2. A completed channel turns the cell under its target into `Empty` and yields one item of that cell's
   material; a `Solid(m)` cell under that becomes `Ramp(m)` (D1, D2). A channel on a cell standing on a ramp
   opens it the same way.
3. On DEFAULT_SEED after one channel at `[61,62,9]`, a miner works a dig at `[61,63,8]` (workable only from
   the pit) and later stands at z 9 again.
4. The channel's item lands on the worker's level, on the nearest free walkable cell by 12.9's drop rule
   (no dwarf on it, not a taken pile cell, walls no dwarf in); never on the target and never in its column.
   **Amended (Wolf, at dev, 2026-10-13; `4df25a3`): the worker's level OR ABOVE, never below it.** The
   worker's level only stalled the staircase sweep at 16 of 24; landings reach +4 levels.
5. A channel is worked from any walkable cell a dwarf could step onto the target from, ramp steps included;
   never from the target itself.
6. A loose item is picked up from any walkable cell a dwarf could step onto the item's cell from, ramp steps
   included. Delivery is unchanged.
7. On DEFAULT_SEED with the creation pile, a two-wide staircase 4 levels deep is fully worked (every mark
   gone, every level-3 target's floor open) for EVERY designation delay 0..24 (the creation sweep).
8. #185's fixture (`room_and_tunnel`, the dead-end miner behind four idle woodcutters) finishes its dig.
9. A channel mark is refused on an emitter's tile and where `z-2` is outside the world (D2, D3), and the gui
   preview lights none of those cells.
10. `way_down_wire.py` prints `WAY DOWN OK` on 5 of 5 fresh daemons, and each mutation row named in Task 7
    turns it, or its scenario test, RED.
11. 12.9's channel tests, 12.10's ore-channel test and the walking-skeleton test stay green or each re-pin
    is disclosed beside the pin. #163, #185 and #188 close.
12. At the seat, Wolf channels a two-wide staircase at least 3 levels down by the camp, with a big pile,
    and watches miners walk down and haulers carry blocks back up, in the gui and in an attached tui.

## Tasks / Subtasks

- [x] **Task 1: RED tests first** (AC 1, 3, 7, 8)
  - [x] `crates/sim-core/tests/scenario.rs` (UPDATE):
    `a_channel_opens_a_step_down_and_a_dwarf_walks_down_and_back` (AC 3; the probe's `walk_down_and_back`)
    and `a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay` (AC 7; the probe's
    `staircase_delay_sweep`, asserting 24/24 and printing the stalled delays on failure). Take the site
    finder, pile rect and stair geometry from the probe; drop every `PROBE_*` toggle.
  - [x] #185's fixture as a test in `lib.rs`'s test module beside `room_and_tunnel` (`lib.rs:5532`):
    the dig at `ROOM_WALL` done within a stated bound.
  - [x] Run each on `015548f` and paste the failing assert lines into Debug Log References. Commit the RED
    tests with `#[ignore = "RED until Task N"]`, un-ignored by the commit that greens them.
- [x] **Task 2: the channel rule** (AC 2, 4)
  - [x] `execute_jobs`' `JobKind::Channel` arm (`lib.rs:1665-1680`): `below = target.z-1`, `base = z-2`.
    `below` `Solid(m) | Ramp(m)` → `Empty`, yield `yield_of(m)`; `base` `Solid(m)` → `Ramp(m)`; `Ramp` and
    `Empty` unchanged. Both tiles change in the same tick, through `Terrain::set_tile` (both dirty).
  - [x] Landing: extract the breadth-first "nearest free walkable cell" search from `release_claim`
    (`lib.rs:1188-1231`) into one helper and call it from both; the channel's item lands by it from the
    worker's position. Remove the target-occupant `step_aside` block (`:1699-1713`) if nothing spawns on the
    target any more, and its NOTE.
  - [x] Keep the `walls_in_a_dwarf` check (the probe broke without it). Its NOTE (`:1698`, "reads the
    terrain before the ramp is cut") is now about the pit: re-state what it reads, and confirm in a test
    that it does not refuse the staircase's own cells forever (AC 7 covers this).
- [x] **Task 3: work positions across a ramp step** (AC 5, 6)
  - [x] `work_positions` (`lib.rs:1089-1158`): `JobKind::Channel` uses `astar_neighbours(terrain, blocked,
    job.target)` in place of same-z `side_neighbours`; the haul pick-up leg uses
    `astar_neighbours(terrain, blocked, *pos)`. Keep the "target standable" gate and the delivery leg.
  - [x] Check every consumer that assumed same-z: the gui's facing (`a_channelling_miner_faces_the_target_cell`,
    `headless.rs:5313`) and `serve.rs`'s "same z, Manhattan distance 1" assert from 12.9 (`0fccc74`).
- [x] **Task 4: the dig-face jam** (AC 7, 8)
  - [x] Reproduce from the sweep's RED delays first: dump positions, `claims()`, `jobs()` and items at the
    stall (the probe's dump code) and name the mechanism before changing any rule.
  - [x] Fix inside `resolve_blocked_step` / `step_aside` / claiming (`lib.rs:1285-1449`, `:479-560`).
    #185's candidate directions: the job holder wins the yield over idle dwarves; a dwarf moved aside does
    not keep a home inside a one-wide passage. **No swap** (Wolf, 12.9 Task 0 Q1b): if the only fix is a
    swap, stop and ask Wolf.
  - [x] The judge is AC 7's 24/24 plus AC 8. Then run the 12.9 sweep over seeds `0..=16` and record every
    pinned outcome that moved.
- [x] **Task 5: designation filter** (AC 9)
  - [x] `apply_command`'s `DesignationKind::Channel` filter (`lib.rs:2365-2371`): also refuse an emitter's
    tile and `pos.z < 2`. A rect left with zero cells is a `Refusal` (standing AC 6), as today.
  - [x] gui `sim_will_keep` (`crates/gui/src/project.rs:1076-1086`): mirror both, so the preview lights none
    of them.
- [x] **Task 6: re-pins and docs** (AC 11)
  - [x] Expect to move (code half done, Agent A): `execute_jobs_channels_a_material_preserving_ramp_and_spawns_stone` (`lib.rs:5039`),
    `execute_jobs_removes_a_channel_job_when_the_support_is_already_a_ramp` (`:5090`), scenario
    `a_channel_is_worked_from_the_next_tile_and_the_stone_lands_on_the_target` (`scenario.rs:949`),
    `a_channel_worker_whose_support_is_removed_lets_go_and_the_crew_goes_on` (`:854`),
    `channelling_over_ore_yields_the_ore_and_leaves_a_stone_ramp` (`:3219`), and any 12.9 sweep pin.
    Disclose each beside the pin and in its commit (12.9 rule).
  - [x] `ARCHITECTURE-SPINE.md` (2026-08-09): amend the 12.10 bullet (`:157-166`) for D1 and add the channel
    rule. README: one line under the controls saying dig = sideways at your level, channel = a step down.
- [x] **Task 7: the instrument and its RED** (AC 10)
  - [x] `12-10b-signoff/way_down_wire.py` (EXISTING, creation) is the instrument. Run it on 5 fresh release
    daemons; paste each verdict block.
  - [x] NEW `mutations/12-10b.sh`, each row named against its killing test, run alone after commit:
    channel worked from the same level only (kills AC 7; the wire goes RED at level 1); pick-up from the
    same level only (AC 7; wire RED at level 2); `base` left `Solid` (AC 3); the item spawned on the target
    (AC 4); the emitter refusal dropped (AC 9); the Task 4 fix reverted (AC 7 and AC 8). Re-point any old
    row the change breaks (`audit-mutations.py`).
- [x] **Task 8: the seat card** (AC 12)
  - [x] NEW `12-10b-signoff/vehicle-card.md`, in the seat's launch form. Teach: dig vs channel in one
    sentence each; drag a BIG pile first (4 levels two wide = 28 blocks); channel a 2-wide row of 5 at the
    camp level, then each next row one shorter and one level down inside it. Give the site, the z per row,
    the compass bearing at boot, and the tui `--z` and keys. Known, by design: blocks can float until
    12.11; one-wide passages are slower.
- [x] **Task 9: gate** — `RUST_TEST_THREADS=1 scripts/gate.sh`, full, green, before review.

### Review Findings

Code review run 1, 2026-10-14, on `1ab1703` (diff `015548f...HEAD`, 18 files, +1,901/-211; code 4 files, +804/-112).
This was a fresh session. Four layers ran and none timed out. All ran cargo 1.97.1, each in its own
`/tmp/review-<layer>` target dir with `CARGO_BUILD_JOBS=6`.
- Blind Hunter (Sonnet) took sim-core `src` plus gui `project.rs`.
- Edge Case Hunter (Sonnet) took sim-core `tests/`, simd `tests/`, the five mutation tables and `way_down_wire.py`.
- The Acceptance and Feature Auditors (Opus) took the whole diff.

| Layer | Findings | Severity | Ran |
| --- | --- | --- | --- |
| Blind Hunter | 6 | 1 MED, 5 LOW | `cargo test -p sim-core --lib` filters (8 green); 4 throwaway probes in a `git archive` copy (item on target, refusal undo, dwarf on target) |
| Edge Case Hunter | 7 | 1 MED, 6 LOW | the full scenario suite (58 green); the wire on 2 fresh daemons (2 OK); a daemon killed mid-run (RED, not BROKEN); every mutation anchor matched by `rg` |
| Acceptance Auditor | 6 | 1 MED, 5 LOW | the full sim-core suite with `--include-ignored`; the gui preview test; `serve.rs`; the 3 RED tests re-run on `dbbebd0` (all RED as recorded); the wire on 2 fresh daemons (2 OK) |
| Feature Auditor | 6 | 1 HIGH, 2 MED, 3 LOW | hop trace (every hop WIRED except the gui channel preview); the wire on 3 fresh daemons (3 OK); variants on fresh daemons: card geometry OK, `west` OK, `three` OK, **`same` RED 2/2, `onewide` RED 2/2**; tui `--frame --z 9..5` on a dug stair; a `preview_cells` probe in a copy |

Convergences:
- the garbled landing NOTE: blind + acceptance;
- no-op dirty deltas on a refused cut: blind + acceptance;
- AC 3's `pit_dwarf` is any dwarf at z 8: edge + acceptance.

Live bugs filed at the moment they were found: **#193** (the pit cage, HIGH) and **#194** (the one-wide stair jam,
`route:undecided`).

Live: the wire reproduces GREEN on 7 independent fresh daemons across three layers, and the card's own geometry is
OK on the wire for 3,000-5,000 ticks (haulers lift 9 of 15 pit blocks to the pile, and 6 float). Two realistic
departures from the card's geometry are RED: the same rect reused one level down (#193), and a one-wide staircase
(#194). **Not proven live:** AC 12 in full (the seat: the gui watch, haulers seen carrying, tui parity and ramp contrast
on Wolf's terminal, the card's compass and pick claims).

Cost: $16.15 over 315 turns (subagents 70.9% of tokens). Reaped after triage: 10 `/tmp` build dirs, 33.8 GB
(17.9 GB reclaimed on disk). The #194 decision was RULED by Wolf (fix in 12.10b). Patches LEFT AS ACTION ITEMS
(Wolf, 2026-10-14) for a `/bmad-review-patch` session. Status -> in-progress.

- [x] [Review][Patch] (decision resolved) **One-wide staircases jam (#194); #185's closure (feature, MED, RAN)** — Run 1: level 3 is
  never worked, with three dwarves frozen on the stair for ~11,000 ticks. Run 2: all four levels are worked, but an
  idle dwarf parks on the only stair cell and nobody comes back up. This is the inverse of #185's fixture: holders
  coming DOWN meet an idle dwarf at the dead end, and passing it would need the banned swap. Two idle dwarves never
  call `resolve_blocked_step`. AC 8 (the fixture) is met, but #185's title shape ("never gets out past idle dwarves")
  still occurs on a one-wide stair. The card calls it "slower" (`vehicle-card.md:88`).
  **RULED 2026-10-14 (Wolf): fix it in 12.10b.** First, name the mechanism from a dump at the stall (positions,
  `Path`, `claims()`, idle homes) on both runs. Then pin it RED with a deterministic fixture (the one-wide stair, or
  the `onewide` wire variant: `level_cells` geometry in #194's body; #193's body has the `same` variant). **No swap** (12.9 Task 0 Q1b,
  scope guardrails): if the only fix is a swap, stop and ask Wolf. Judge: a one-wide staircase 4 levels down OK on
  the wire 5/5, AC 7 and AC 8 still green, and the 12.9 seed sweep pins re-run. Then the card's one-wide line
  follows, and the PR closes #185 and #194.
- [x] [Review][Patch] **HIGH (#193): a channel that severs a pit from its rim cages the dwarves inside (feature, RAN
  2/2)** [`crates/sim-core/src/lib.rs:1783-1800`, `:807-860`] — Channelling the SAME rect one level down
  (`x 61..65 y 62..63`) leaves 2 dwarves at z 6 with no way up, 3 marks never worked, 14 blocks stranded, and
  `back up: NEVER`. The post-cut check (Deviation 1) seeds `walls_in_a_dwarf`'s floods from `astar_neighbours(target)`
  on the CUT terrain. The target now stands on `Empty` and has no upward step, so the rim is never a side and the split
  is invisible (orchestrator confirmed this by reading). Fix: take the sides from the PRE-cut terrain (the rim
  included) and flood the CUT terrain. Then the severing cut is refused and its cell stays as the step. The NOTE says
  it covers this. RED first: a unit fixture of a pit whose last link to the rim is the target, plus the `same` wire
  variant. Re-judge AC 7 24/24, the 12.9 seed sweep pins (Deviation 1's own reason: a pit worker refused forever) and
  the wire 5/5. Add a mutation row (sides from the cut terrain) against the new test.
- [x] [Review][Patch] **MED: AC 9's "the gui preview lights none of those cells" is not wired (feature, RAN)**
  [`crates/gui/src/project.rs:1046`] — `preview_cells` sends Channel through `client_core::surface_targets` with no
  `sim_will_keep` filter, so the new Channel arm (`:1080-1086`) is reached only by its own unit test. A probe of
  `preview_cells(.., Channel)` on the test's campfire mirror lit `[1,0,2]`, the fire. The dev's RED (mirror reverted)
  tested the arm, not the preview. Fix: filter the Channel and Stockpile branch through `sim_will_keep`. Pin it through
  `preview_cells`, then re-mutate (the arm's own test cannot kill a dropped filter call).
- [x] [Review][Patch] **MED: the spine states the landing the ruling replaced, and AC 4's text is unannotated
  (acceptance)** [`ARCHITECTURE-SPINE.md:173`; story AC 4 line 81, Key decision 3 line 188] — The spine says "the dug
  item lands on the worker's level". It was written in `e89c3f4`, before `4df25a3`. The code is "level or above"
  (`cell.z < pos.z` refused), and landings reach +4. Fix the spine bullet, and annotate AC 4 and Key decision 3 inline
  with the ruling (the "partial doc update immunises" shape).
- [x] [Review][Patch] **MED (instrument): `way_down_wire.py` reports a dead daemon as RED (edge, RAN)**
  [`12-10b-signoff/way_down_wire.py:41-47,116-178`] — A simd killed 4 s in printed `level 1: NEVER WORKED ... WAY DOWN
  RED`, exit 1. EOF ends `for line in f` silently. A refused connect, a first-read timeout or a bad JSON line also exit
  1, with a traceback. The docstring promises `BROKEN` (exit 2) for "the daemon stalled or closed". This is a latent
  silent failure in an instrument (frostvein exception), so it is patched.
- [x] [Review][Patch] **LOW: the seat card understates floating blocks and needs the shrinking-stair rule (feature)**
  [`12-10b-signoff/vehicle-card.md:86-88`] — The card says about 3 floating blocks; card geometry measured 6 (2 runs).
  After the #193 patch, a reused rect leaves one mark that is never worked (it stays as the step), so say "each row
  one shorter". The one-wide line follows the #194 ruling.
- [x] [Review][Patch] **LOW: the landing NOTE is spliced mid-sentence into the refusal note, a 127-column line (blind +
  acceptance)** [`crates/sim-core/src/lib.rs:1779-1782`] — It is in the function the #193 patch edits, so it is fixed
  there. Restate the post-cut NOTE to match the #193 fix.

**Review patch pass (2026-10-14, fresh session).** Every item above landed; commits `bab56ad`..`ab90c52`.
Closure table (both-sides rule; REWORK = re-closes an earlier round's half-closed fix):

| Item | Written for | Then tested | Pre-existing-state fixture | |
| --- | --- | --- | --- | --- |
| #194 one-wide jam (`e000392`) | holders coming DOWN a one-wide stair, an idle dwarf at the dead end, a job-holding line behind | the #185 side (an idle line AHEAD of a holder in a dead end) and the stuck head-on with no room to back out (#183) | `room_and_tunnel` (`a_dead_end_miner_behind_idle_woodcutters_finishes_its_dig`), 12.9's `head_on` fixture (`a_head_on_with_no_escape_drops_the_idle_blockers_stale_path`), the DEFAULT_SEED two-wide sweep (AC 7 24/24), seeds `0..=16` sweep pins unchanged | REWORK (Task 4) |
| #193 pit cage (`d6b3beb`) | a cut that takes a pit's last step up to its rim | a cut beside a second way out (worked), and Deviation 1's world (a pit worker whose way up the cut itself opens) | `severing_pit(true)` (`a_channel_beside_a_second_way_out_of_the_pit_is_worked`); AC 7 24/24 and the seeds `0..=16` sweep pins unchanged | REWORK (Deviation 1) |
| AC 9 gui preview (`1de2889`) | a channel drag over the campfire | a stockpile drag over the same row still lights the fire's tile; the dig/cut branch untouched | 12.10b's 3x1 campfire mirror; `the_cut_preview_covers_every_rect_cell_and_only_tree_cells_are_in_the_cut_style` green | REWORK (dev RED tested the arm) |
| Spine + AC 4 / KD 3 (`83b3784`) | the spine and story text | read against the code: the landing refusal `cell.z < pos.z` and `a_channel_never_lands_its_stone_below_the_worker` | none: a doc change | |
| Wire BROKEN (`bab56ad`) | a daemon that dies or drops the client | a live daemon still judges: OK and RED both print | a daemon killed 4 s in (the review's repro, now `BROKEN ... closed the connection at tick 687`, exit 2); 12 concurrent daemons dropping a slow reader (`client delta queue full`), 4 of 4 BROKEN; pre-fix `same` RED and post-fix OK runs | |
| Seat card (`ab90c52`) | the card's text | the wire's own output: `onewide` 5/5 OK, `same` leaves exactly 1 of 10 marks | none: a doc change | |
| Landing NOTE (`d6b3beb`) | the spliced NOTE | read in place, restated for #193 | none: a comment | |

- [x] [Review][Defer] **MED: a channel over a tile holding a stone (or a pile cell) leaves it in mid-air (blind, RAN)**
  [`crates/sim-core/src/lib.rs` Channel arm; filter `:2459`] — deferred, ruled 12.11's at creation (#135, "floating
  blocks"). This is a new route into it: before 12.10b the target stayed standable. The stranded stone's haul job
  retries forever (the existing FR8 NOTE).
- [x] [Review][Defer] **LOW-MED: a refused channel completion retries forever, a full `DIG_WORK_TICKS` per cycle, with
  no feedback (blind)** [`lib.rs` `let Some(found) = found else`] — deferred, pre-existing (#187). It becomes the
  normal outcome of a stair-severing mark after the #193 patch.
- [x] [Review][Defer] **LOW: a refused cut's undo dirties both tiles, sending no-op deltas every retry (blind +
  acceptance, RAN)** [`lib.rs:1791-1805`] — deferred: harmless, since the delta carries the current tile.
- [x] [Review][Defer] **LOW: a dwarf on the target falls into the pit, and through an `Empty` base (D2) into a cavity
  the wall-in check never measured (blind, RAN for the ramp case)** [`lib.rs` Channel arm] — deferred: rare (a cave
  under a standable target), and the occupant is outside the check's closed set.
- [x] [Review][Defer] **LOW: `make_room_down_the_idle_chain`'s route may pass the holder's tile, and a second holder
  can overwrite a leaving dwarf's `Path` (blind, read)** [`lib.rs:1324-1364`] — deferred: no swap and no stall of the
  holder found.
- [x] [Review][Defer] **LOW: AC 3's test credits any dwarf seen at z 8, and its walk-back leg is "reaches z 9 at all"
  (edge + acceptance, read)** [`crates/sim-core/tests/scenario.rs` `a_channel_opens_a_step_down_...`] — deferred:
  fixture robustness, deterministic and green.
- [x] [Review][Defer] **LOW: the staircase sweep asserts open floors, never a dwarf on the new level (edge, read)** —
  deferred: the wire carries "a dwarf stood at z 5".
- [x] [Review][Defer] **LOW: the wire's OK does not need the deepest dwarf to be the one that came back (edge, read)** —
  deferred: the docstring does not claim it, and the miner-down, hauler-up reading matches AC 12.
- [x] [Review][Defer] **LOW: the wire's pile keeps 22 cells for 28 blocks and passes because about 6 float (feature,
  RAN)** — deferred to 12.11: once blocks fall, the wire's pile is too small.
- [x] [Review][Defer] **LOW: tui ramps one or more levels below the view are dimmed to about rgb(15-47) (feature,
  RAN)** — deferred to the seat (AC 12 judges it on Wolf's terminal).
- [x] [Review][Defer] **LOW: record nits (acceptance)** — the `ONE_LEVEL_ISLANDS` comment keeps interim narrative
  ahead of the final disclosure, and the `3-2-the-dig.sh` re-point dropped older provenance lines. Deferred.

Dismissed (4): the landing "omits the taken-pile test" (items are in `blocked`); mutation rows 2 and 6 killed by tests
Task 7 did not name (disclosed beside the rows and in the Debug Log); `serve.rs`'s `|dz| <= 1` (disclosed, and
`apart == 1` still holds); rows killed only by the seed sweep (the record shows them run alone and KILLED).

#### Code review run 2, 2026-10-14, on `ec93ff8` (diff `dc31a55..HEAD`, the patch pass; 12 files, +506/-73)

Fresh session. Four layers ran and none timed out. All ran cargo 1.97.1, each in its own `/tmp/review-<layer>` target dir.
The territories were the same as in run 1.

**Delta vs run 1:**
- **NEW: 14.** 2 HIGH, 3 MED, 9 LOW.
- **REWORK: 3**, all half-closed rows of the closure table.
  - #193: its fix never counted a dwarf standing on the target (#195).
  - #194: "`onewide` 5/5" does not hold at a bigger sample, and a line of job holders still jams (#196).
  - AC 9: the "other side" the row tested is a stockpile preview that lights the fire, which the sim drops.
- **Stopping rule:** NEW HIGHs exist, so another round is authorised after the patches.

| Layer | Findings | Severity (layer's) | Ran |
| --- | --- | --- | --- |
| Blind Hunter (Sonnet) | 3 | 1 MED, 2 LOW | sim-core lib 86 green; a 600-combo #194 permutation probe on HEAD and on `dc31a55` (0 swaps/shared tiles; the same 108 multi-miner failures on both, so no regression) |
| Edge Case Hunter (Sonnet) | 5 | 5 LOW | wire BROKEN on 8 death/stall shapes (kill -9, SIGSTOP, RST, bad JSON: all exit 2); `two`/`onewide` OK, `same` RED; 7 mutation rows run in archive copies, all KILLED by the intended assert |
| Acceptance Auditor (Opus) | 4 | 1 MED, 3 LOW | closure table re-run row by row; wire `two` on 16 fresh daemons: **15 OK, 1 RED** (`back up: NEVER`); 6 mutation rows KILLED; seed sweep, AC 7, #185, #193 pair green |
| Feature Auditor (Opus) | 6 | 2 HIGH, 1 MED-HIGH, 1 MED, 2 LOW | hop trace: all WIRED; 14 live runs (an instrumented wire copy that logs per-tick positions, jobs and reversals); a unit probe of a dwarf on the severing target |

**Convergences:**
- Miners stay at the bottom (M2): acceptance + feature + edge.
- The card's "reuse the whole rect" sentence: feature (RAN) + edge (the lit-mark half confirmed).

**Live bugs filed when found:**
- **#195**: a dwarf standing on the target is caged.
- **#196**: job holders head-on in a one-wide line, `route:undecided`.
- The M2 measurement was added to **#186**.

**Not proven live:** AC 12 in full, the seat.
- The gui watch, tui parity and ramp contrast on Wolf's terminal.
- The card's compass, pick and `,` claims.
- The new preview filter was reached only by its unit test.

- [x] [Review][Patch] (decision resolved) **HIGH (#196): job holders head-on in a one-wide line wait forever (feature, RAN), REWORK of #194**
  **RULED 2026-10-14 (Wolf): (a), fix it in 12.10b.** Dump the stall first (positions, `Path`, `claims()`, jobs), then
  RED, then the fix. **No swap**: if only a swap would do, stop and ask Wolf. Judge: the one-wide wire at a sample
  bigger than 5, the one-wide sweep, AC 7 24/24, AC 8, and the seeds `0..=16` pins.
  [`crates/sim-core/src/lib.rs` `resolve_blocked_step`]
  - **One-wide staircase, 2 of 9 runs:** jammed at level 2. Miner 2 at (61,62,8), hauler 3 at (62,62,7) and miner 4 at
    (63,62,7) all hold jobs. Nothing moved from tick ~660 to 12012.
  - **Same rect reused, 2 of 5 runs:** the #193 refusals fall diagonally and narrow the two-wide pit to one-wide. Four
    holders stayed frozen from tick 2500 to 14012.
  - The #194 arm handles an IDLE blocker only. Holder vs holder waits by design (the `resolve_blocked_step` NOTE).
  - #194's judge, "one-wide 4 levels OK on the wire 5/5", was met on a sample too small to show this.
  - No dump yet.
  - Choose:
    - (a) fix it in 12.10b, with no swap (dump first, then RED, then the fix; ask Wolf if only a swap would do);
    - (b) move it to a new story, have the card say a one-wide stair can jam, and stop the PR closing #194 as fixed.
- [x] [Review][Patch] (decision resolved) **RULED 2026-10-14 (Wolf): (a), by design for now.** Add a card line (mark
  something up top to bring the miners up), give `two` the same up-top call as `onewide`, and have the PR name M2 as
  accepted, not fixed, under #194, pointing at #186.
  **MED: miners stay at the bottom of a finished staircase; the wire's `two` verdict is not
  deterministic (acceptance + feature + edge, RAN)** [`way_down_wire.py:176-179`; `vehicle-card.md`]
  - `two` on fresh daemons: 1 of 16 RED, `back up: NEVER`.
  - In every OK run a HAULER came up. Both miners idled at z 5-6. On the card's geometry they stayed there for 6,000
    ticks.
  - `wander` is same-z only (#186). The patch pass added the up-top call only to `onewide` and its test.
  - The card never tells Wolf that miners stay down until work is marked up top.
  - Choose:
    - (a) by design: add a card line and give `two` the same up-top call; the PR names M2 as accepted, not fixed, under
      #194;
    - (b) fix it in the sim (idle dwarves climb to the camp level); new behaviour, out of scope, a new story.
- [x] [Review][Patch] **HIGH (#195): a dwarf standing ON the channel target is caged in the pit the cut severs
  (feature, RAN), REWORK of #193** [`crates/sim-core/src/lib.rs:832-841`, the Channel arm]
  - `walls_in_a_dwarf_from` closes `cell`, so a dwarf on the target is in no flood piece. `settle` then drops it into
    the severed pit.
  - Unit probe, `severing_pit(false)` with the dwarf on the target: the cut was not refused; the dwarf ended at z 3 with
    no route to the rim.
  - Live: hauler 3 was cut off from the step at tick 1297 and caged until tick 8012.
  - Run 1 deferred this as LOW/rare. After #193 it is the remaining route into #193's outcome.
  - Fix: count a dwarf on the target at the cell it will settle to.
  - RED first, as a twin of the #193 fixture, plus a mutation row.
- [x] [Review][Patch] **MED: the card invites "reuse the whole rect one level down"; doing so fails 4 of 5 (feature,
  RAN)** [`12-10b-signoff/vehicle-card.md` row-rule paragraph]
  - Failures: 2 jams (#196), 1 cage (#195), 1 burn (a miner put 5,614 of ~6,500 ticks into two refused marks, #187).
  - "Never worked and stays lit" is not always true either: in run 4 both refused marks were worked later.
  - Strike the sentence and keep "each row one shorter".
- [x] [Review][Patch] **MED: the Stockpile preview lights an emitter's tile that the sim drops, and the new test pins
  the mismatch (blind, read; orchestrator confirmed), REWORK of the AC 9 row** [`crates/gui/src/project.rs:1091`,
  the test `a_channel_drag_over_the_campfire_does_not_light_its_tile`]
  - The sim's `PlaceStockpile` filters `is_walkable(.., blocked_cells(emitters))`. The mirror's Stockpile arm is
    `is_standable` only.
  - The test asserts `[1,0,2]`, the fire, is lit for Stockpile.
  - The `preview_cells` and `sim_will_keep` docs claim both modes drop what the sim refuses.
  - Fix: mirror the emitter rule in the Stockpile arm, flip the assert, re-mutate. This is a latent silent failure.
- [x] [Review][Patch] **LOW: `sim_will_keep(Channel)` lacks the sim's "no tree at z-1" rule (blind, read)**
  [`project.rs` Channel arm; sim `lib.rs:2504-2508`] — It is in the function being edited, so it is fixed there.
- [x] [Review][Patch] **LOW: the card's floating-block count reads as covering both shapes (edge, RAN)**
  [`vehicle-card.md`] — The 6 is for two-wide; one-wide measured 4. Fix it when the card is edited.
- [x] [Review][Patch] **LOW: a stale Completion Note says AC 4's text is "left as written" (acceptance)** [this file,
  Completion Notes, first bullet] — `83b3784` annotated AC 4 and KD 3 inline. This is the partial-doc-update shape.
- [x] [Review][Patch] **LOW: `step_and_watch`'s swap and shared-tile detector has never been shown firing
  (acceptance)** [`crates/sim-core/tests/scenario.rs` `step_and_watch`] — It is the only per-tick enforcement of the
  No Swap ruling. Show it RED once, for example with a mutant that teleports two adjacent dwarves.
**Review patch pass, run 2 (2026-10-14, fresh session).** All 9 items landed; commits `31322f8`..`2b4db37`.
Closure table (both-sides rule; REWORK = re-closes an earlier round's half-closed fix):

| Item | Written for | Then tested | Pre-existing-state fixture | |
| --- | --- | --- | --- | --- |
| #196 holders head-on (`e6fd5b5`) | two JOB HOLDERS head-on in a one-wide line, neither with an escape (the #194 back-out ran for an idle blocker only) | #194's side, an IDLE blocker at the dead end, and #185's idle line ahead, and 12.9's stuck head-on with no room to back out; plus the two-wide shape at long delays | `job_holders_head_on_in_a_one_wide_staircase_do_not_wait_forever` (DEFAULT_SEED, level 0 at tick 190, then +8: the state shipped code froze in, level 2 never worked); the one-wide sweep 24/24 (#194's RED delays 0-3, 8), `room_and_tunnel` (AC 8), `a_head_on_with_no_escape_drops_the_idle_blockers_stale_path`, AC 7 24/24, the seeds `0..=16` pins unchanged; a release probe of 1,200 runs (below) | REWORK (#194) |
| M2 accepted (`497ff9b`) | the wire's `two` shape and the card | `two` AND `onewide` on fresh daemons | 10 + 10 fresh DEFAULT_SEED release daemons: `two` 10/10 OK (8 via the up-top call), `onewide` 10/10 OK | |
| #195 dwarf on the target (`31322f8`) | a dwarf standing ON the channel target, dropped into the pit the cut severs | the other side: the same dwarf on the target over a pit with a second step to the rim (worked, and it settles into that pit); #193's own pair (dwarf in the pit) unchanged | `severing_pit(false, true)` RED (the target opened); `severing_pit(true, true)`; `severing_pit(false/true, false)` green | REWORK (#193) |
| Card "reuse the rect" (`497ff9b`) | the card's text | the probe's `same` shape: 180 runs never finish, 50 in a refusal loop (#187) | none: a doc change | |
| Stockpile preview (`fd0d208`) | the Stockpile arm lighting an emitter's tile | the Channel arm still refuses the fire; a non-emitter stockpile cell still lit (`[0,0,2]`) | 12.10b's 3x1 campfire mirror (both tests); the flipped asserts RED first | REWORK (AC 9 row) |
| Channel over a tree (`fd0d208`) | a channel cell standing on a trunk | the stone cell beside it stays lit | `the_channel_preview_skips_a_cell_standing_on_a_tree` (2x1 mirror, trunk at `[1,0,1]`), RED first | |
| Card floating count (`497ff9b`) | the card's text | read against run 2's measurements (two-wide 6, one-wide 4) | none: a doc change | |
| Stale Completion Note (`497ff9b`) | the note | read against AC 4 and Key decision 3 inline | none: a doc change | |
| `step_and_watch` fires (`190cd8d`, `2b4db37`) | the detector, never seen RED | both halves: swap (row 18) and shared tile (row 19) | the one-wide sweep (row 18: `dwarves 3 and 4 swapped tiles at tick 278`); the #196 fixture in release (row 19: `two dwarves share a tile at tick 402`) | |

**Found while patching (not a finding of run 2):** in every debug run `World::step`'s own `debug_assert` ("two
dwarves share a tile after tick") fires before `step_and_watch`'s shared-tile assert can, so that half of the
detector is live in release runs only. The swap half is unique to `step_and_watch`. Recorded beside rows 18-19.

**For the PR body (Wolf's M2 ruling):** under #194, name M2 (miners stay at the bottom of a finished staircase) as
ACCEPTED, NOT FIXED: idle dwarves wander on their own level only (#186). The card tells the seat to mark something
up top; the wire calls a dwarf up on every shape. #194 and #196 are fixed.

- [x] [Review][Defer] **LOW: the #193 refusal loop burns a miner, 20-72 work entries per lit mark, up to 2,964 work
  ticks (feature, RAN)** — deferred, pre-existing (#187). The live figure is in this run.
- [x] [Review][Defer] **LOW: the first sends in `way_down_wire.py` sit outside the `OSError` try, and a non-object JSON
  line raises `AttributeError` (edge, read)** [`way_down_wire.py:109-131`] — deferred: a microsecond window, and the
  traceback is visible, not silent.
- [x] [Review][Defer] **LOW: `same`'s verdict is always RED and cannot tell a #193 refusal from another stall (edge,
  RAN)** — deferred: the docstring says to read the lines.
- [x] [Review][Defer] **LOW: the #194 sweep's mutants take 465-676 s in debug (edge, RAN)** — deferred: they still
  report KILLED.
- [x] [Review][Defer] **LOW: the case-1 fallback can hand an idle blocker a `Path` whose first tile is a third dwarf,
  so it freezes until that tile frees (blind, read)** [`lib.rs:~1462-1467`] — deferred: bounded, no swap, and no
  worse than the old `return false`.

Dismissed (1): `sides_of` runs for every job kind (one extra `astar_neighbours` call per dig; the dig path is
otherwise bit-identical).

**Run 2 outcome:**
- Cost: $20.86 over 326 turns, with subagents at 76.2% of tokens.
- Reaped after triage: 13 `/tmp` build dirs, 46.2 GB (28.3 GB reclaimed on disk).
- Both decisions were RULED by Wolf.
- 8 patches were LEFT AS ACTION ITEMS for a `/bmad-review-patch` session.
- Status -> in-progress.
- Round 3 is authorised (new HIGHs). It audits this round's closure table.

## Dev Notes

### Scope guardrails (do NOT)

- No falling items or trees (12.11). No new designation, `Command`, key, or tui input. No swap (ask Wolf).
- No change to dig, cut, delivery, or the pile fill order. No new saved state, no wire change.
- No channel-rule change beyond D1-D3. No look work: ramps and open cells already draw.

### What already exists (build on it)

- `astar_neighbours` (`lib.rs:874-913`): THE step rule, ramps included. Tasks 3 and 4 reuse it.
- `release_claim`'s drop search (`lib.rs:1183-1262`): 12.9's "never under a dwarf, never walls one in".
- `resolve_blocked_step` (`lib.rs:1304-1449`): 12.9's four blocked-step cases and its dead-end escape.
- `way_down_wire.py`: daemon-judged, RED on `015548f`, OK on the scratch rule (1 of 5, the jam).
- `creation-probe.patch`: the scratch rule, both work-position changes, the probe tests and the sweep.

### Key decisions & traps

1. **The ramp goes under the opened cell, two below the target.** A dwarf stands ON a ramp from the cell
   above it; `Ramp` at `z-1` is today's bug.
2. **Both work-position changes are load-bearing.** The probe stalls at level 1 without either one.
3. **A dug block lands on the miner's level, which is the staircase.** Haulers must lift it across a ramp
   step (Task 3), and the pile must hold it: every fixture places the creation pile. **Superseded by the
   AC 4 ruling (`4df25a3`): the miner's level or above, never below; a pit worker's block lands up the
   ramp.**
4. **Wire timing is a seed.** The same rule went OK 1 of 5 over the wire; only the delay sweep makes the
   jam deterministic. Never accept a single green wire run as proof.
5. **The jam is #185's family, at a dead end that is two wide.** Find the mechanism from a dump before
   choosing a fix; 12.9 found that a per-step rule livelocks where an escape rule does not.
6. **zsh does not word-split `$v`.** `env $v cmd` sets only the first variable; creation lost a matrix to
   it. Use `env ${=v} cmd` or separate arguments.

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs`; `crates/sim-core/tests/scenario.rs`
- UPDATE `crates/gui/src/project.rs` (`sim_will_keep`); any re-pinned `crates/gui/tests/headless.rs`,
  `crates/simd/tests/serve.rs` assert
- UPDATE `ARCHITECTURE-SPINE.md` (2026-08-09), `README.md`, old mutation tables per Task 7
- NEW `mutations/12-10b.sh`, `12-10b-signoff/vehicle-card.md`
- EXISTING (creation) `12-10b-signoff/way_down_wire.py`, `12-10b-signoff/creation-probe.patch`
- `protocol`, `tui`, `client-core`, `simd/src`: no change expected.

### Previous story intelligence

- 12.10: dig takes ramps; a channel over ore yielded ore + `Ramp(Stone)` (D1 retires that). Seat pass 2's
  trench jam is this story's AC 7/8 (`12-10-signoff/seat-pass-2-trench.png`).
- 12.9: every uncarried item blocks its cell; one dwarf per tile; channel worked from the next tile (its
  AC13, kept: never the target). Its seed sweep `0..=16` pins still dwarves and unfinished runs.
- Build probes in a scratch worktree, never the shared tree; kill daemons by PID.

### References

- `epics.md:2702-2727` (Story 12.10b), `:2422-2434` (standing ACs); issues #163, #185, #188
- `lib.rs:874-913` (`astar_neighbours`), `:1089-1158` (`work_positions`), `:1183-1262` (`release_claim`),
  `:1285-1449` (`step_aside`, `resolve_blocked_step`), `:1665-1727` (channel completion), `:2365-2371`
  (channel filter), `:5532` (`room_and_tunnel`)
- `crates/gui/src/project.rs:1076-1086`; `ARCHITECTURE-SPINE.md` (2026-08-09) `:157-166`

## Verification

**Instrument (Task 7), daemon-judged, against a FRESH release `simd`:**

```bash
simd 7451 &   # fresh, DEFAULT_SEED; one daemon per run
python3 _bmad-output/implementation-artifacts/12-10b-signoff/way_down_wire.py 7451 12000
```

- **Deliberate RED (observed at creation):** on `015548f`: `level 0: NEVER WORKED by tick 12102 (10 of 10
  channels left; the floor under [61, 62, 9] is {"ramp": "snow"})` and `WAY DOWN RED`. With the ruled rule
  but channels worked from the same level only: `level 1: NEVER WORKED ... {"ramp": "soil"}`, `WAY DOWN RED`.
  With pick-up from the same level only: `level 2: NEVER WORKED`, `WAY DOWN RED`.
- **Green required of dev:** 5 runs on 5 fresh daemons, each `level 0..3: worked`, `deepest dwarf z 5`,
  `back up: dwarf N at tick T`, `WAY DOWN OK`. Observed at creation on the ruled rule WITHOUT Task 4: 1 of
  5 (4 RED at level 3, the jam).
- **Sim:** `cargo test -p sim-core --test scenario a_two_wide_staircase` prints the stalled delays on
  failure; required 24/24.

**Seat (AC 12), in the seat's launch form:**

- WSL: `simd 7451`, and in a second WSL terminal `tui 7451 --z 8` for parity.
- PowerShell: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`.
- Follow `12-10b-signoff/vehicle-card.md` (Task 8).

## Dev Agent Record

### Agent Model Used

- Orchestrator + verifier: Claude Opus 5.5 (`claude-opus-5-5`). Wolf picked Sonnet 5.5 subagents for the
  implementation (2026-10-13), as in 12.2-12.10; the skill config still names Codex.
- Agent A (Sonnet 5.5): Tasks 1, 2, 3, 5 and the code half of 6. Agent B (Sonnet 5.5): Task 4. One after
  the other, on one tree.
- The orchestrator: the AC 4 landing rule after Wolf's ruling (`4df25a3`), AC 6's direct test, the spine,
  README and seat card, `mutations/12-10b.sh`, every mutation run, every wire run and the full gate.

### Debug Log References

- (A) Task 1 RED, run on unchanged `015548f` code:
  - `a_channel_opens_a_step_down_and_a_dwarf_walks_down_and_back`: `scenario.rs:3353` `assertion left == right failed  left: Some(Ramp(Snow))  right: Some(Empty)` (z-1 under the site is a ramp, not open).
  - `a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay`: `assertion left == right failed: staircase worked for 0 of 24 designation delays; stalled (delay, levels done): [(0, 0), (1, 0), ... (23, 0)]`. (First draft judged only "marks gone" and passed vacuously on 015548f, since a ramping channel also clears its mark; the judge now also requires every floor under the level to be `Empty`, per AC 7.)
  - `a_dead_end_miner_behind_idle_woodcutters_finishes_its_dig` (lib.rs, beside `room_and_tunnel`): `not done by tick 1000` (panic from `watch`, bound 1,000 ticks from clock 100).
- (A) Task 5 RED (before the filter): `a_channel_mark_is_refused_on_an_emitters_tile_only`: `left: None  right: Some(Designate { kind: Channel, rect: ... z: 25 })` (lib.rs); `a_channel_mark_is_refused_where_z_is_below_two`: `channel at z 1  left: None  right: Some(Designate ...)`; gui `the_channel_preview_skips_an_emitters_tile_and_cells_below_z_two`: panicked `the campfire's own tile` (project.rs, with the `sim_will_keep` mirror reverted).
- (A) After Task 2 + 3 (rule, landing helper, both work-position changes) and BEFORE the wall-check fix below: AC 3 walk-down GREEN; AC 7 sweep 22 of 24, stalled `[(8, 3), (23, 3)]`; the 12.9 seed sweep showed 4 frozen dwarves and unfinished runs `(0, 4, 1)` and `(4, 4, 1)`.
- (A) Mechanism of those (traced with a temporary `eprintln!`, removed): the wall-in check read the terrain BEFORE the step was cut, with the target counted as closed. A worker standing in the pit of the level above (it works the next level from below, Task 3) has the target as its only way up, so the check said "walls a dwarf in" every cooldown, forever (seed 0: `walls target [68,66,12] worker [67,66,11]`, repeated every ~72 ticks). Fix: for a channel the step is cut first, the check and the landing read the cut terrain, and the two tile changes are undone if either refuses (`undo`). A dig keeps the old order.
- (A) After the fix: AC 7 sweep **24 of 24** (creation saw 16 of 24 with the probe's rule; the probe also carried the block in `Carrying` and refused MARKED cells on drops, and read its wall check before the cut: I believe the jam creation saw at levels 2-3 was largely this pre-cut refusal, not only the dig-face crowd). `#185` fixture still RED (`not done by tick 1000`), the sweep is NOT the judge for it. 12.9 seed sweep: frozen dwarves `[(4, 4, 4), (10, 6, 4)]`, unfinished `[]`.
- (B) Task 4 RED: `a_dead_end_miner_behind_idle_woodcutters_finishes_its_dig` panics `not done by tick 1000` (dump: no dwarf moves for 900 ticks).
- (B) Mechanism of #185's fixture (dump of positions, `Wander.home`, `Path`, `claims()` every tick from 101): the miner (id 0, x 16, `Path` of 11 to the room) is blocked by idle dwarf 1 at x 15. Dwarves 1-4 stand at x 15..12 and the tunnel is one wide. In `resolve_blocked_step` case 1 the idle blocker has no free neighbour off the holder's path (x 14 is occupied, x 16 is the holder), and its exit search treated every OTHER dwarf as closed, so it found no route and returned `false`: the miner waits. The idle line cannot dissolve itself: only the dwarf at x 12 has a free tile ahead, nothing asks it to move, and idle dwarves wander only near a home (here none moves at all). Not the yield rule, not a home inside the passage: the idle chain was never asked to shift.
- (B) After the fix: the fixture finishes at about tick 400 (bound 1,000 from clock 100). AC 7 sweep 24 of 24, AC 3 green, all sim-core tests green, the 12.9 seed sweep unchanged (`ONE_LEVEL_ISLANDS` `[(4, 4, 4)]`, `UNFINISHED` `[]`): no pin moved.

- (O) Verified independently at `4a877a4` (after A): AC 3 and AC 7 green; #185 fixture RED `not done by
  tick 1000`. Wire on 5 fresh release daemons (scratch-worktree build): **5 of 5 `WAY DOWN OK`** (level 3
  worked by tick 1246-1599, deepest dwarf z 5, a dwarf back up by tick 1141-1528).
- (O) **AC 4 finding.** A temporary `eprintln!` at the channel landing over the AC 7 sweep (672 landings,
  `4a877a4` + B): landing z minus worker z = `{0: 406, +1: 172, +2: 19, +3: 22, +4: 9, -1: 44}`. 40% off
  the worker's level; 44 dropped one level DOWN into an earlier pit. Variants, AC 7 sweep: refuse below
  the worker 24/24; worker's level only **16/24**, stalled `[(2, 3), (3, 3), (4, 3), (5, 3), (6, 3), (7,
  3), (8, 3), (10, 3)]`. **Wolf ruled (b), never below the worker** (2026-10-13).
- (O) AC 4 RED: `a_channel_never_lands_its_stone_below_the_worker` panicked `the stone lands at the
  worker's level or above, never in the pit: worker Pos { x: 65, y: 65, z: 25 }, stone Pos { x: 64, y: 65,
  z: 24 }`. Green with `|| cell.z < pos.z` in the landing's refusal. The 12.9 seed sweep moved:
  `ONE_LEVEL_ISLANDS` `[(4, 4, 4)]` -> `[(10, 4, 4)]`, read by dump at tick 4,000: idle at `[67, 76, 28]`
  between two loose blocks and two drops, 0 marks, every claim `None` (the #186 shape). `UNFINISHED` `[]`.
- (O) `mutations/12-10b.sh`, first run on `dfe6718` (`RUST_TEST_THREADS=1`, alone): 9 KILLED, **2
  SURVIVED** the staircase sweep: row 2 (lift from the same level only) and row 8 (the channel's wall-in
  check also read before the cut). Full sim-core suite under each mutant (scratch worktree): row 2 only
  moved the `ONE_LEVEL_ISLANDS` trajectory pin (weak), so AC 6 had no direct test; row 8 failed
  `UNFINISHED` (marks left after 4,000 ticks, a real stall). Fixed in `79a210b`: AC 6's own test
  `a_stone_at_a_pit_rim_is_lifted_from_the_ramp_below` (a positive assertion: the work set is exactly the
  pit cell); row 2 re-pointed to it, row 8 to `no_dwarf_is_caged_by_stones_over_the_probe_seeds`. Re-run
  alone: row 2 KILLED `the stone is lifted from the pit cell, one ramp step below it`; row 8 KILLED `runs
  with marks left after 4000 ticks`. **All 11 rows KILLED.**
- (O) Final wire at `79a210b`, 5 fresh release daemons (scratch-worktree build): **5 of 5 `WAY DOWN OK`**
  (each: level 0..3 worked, level 3 by tick 1219-1583, `deepest dwarf z 5`, `back up: dwarf 1 at tick
  1402` x4 and `dwarf 3 at tick 1396` x1).
- (O) Full gate, `RUST_TEST_THREADS=1 scripts/gate.sh` on `79a210b`: **GATE GREEN 3508 s** (fmt, clippy,
  cargo test 422 s, pixel guards 3046 s, edge probes, metrics, bench, mutation tables still apply).
- (O) Agent A's 13 re-pointed old rows (`12-1` x1, `12-10` x2, `12-9` x5, `3-2-the-dig` x5), run alone after
  the gate: 12 KILLED, **1 SURVIVED**: 12-9 row 28 "a channel stone may wall a dwarf in" against the seed
  sweep (a channel's stone no longer lands on its target, so the sweep never reaches that refusal). New
  direct test `a_channel_never_lands_its_stone_where_it_walls_a_dwarf_in` (worker at the mouth of a
  one-wide dead end, a dwarf at its far end; positive assert: the stone lands on the open tile); row 28
  re-pointed to it (`1ab5d3b`) and re-run alone: KILLED `a stone at the mouth walls the dwarf at the dead
  end in`. **Every 12.10b row and every re-pointed old row now KILLED (24 of 24).**
- (O) `1ab5d3b` is after the full gate: it adds one unit test and re-points one mutation row, no
  production code. Its pre-commit fast gate (everything but `serve.rs` and the pixel guards) is green; the
  full gate was NOT re-run on it.

- (P) Review patch pass, 2026-10-14 (fresh session, Opus 5.5):
  - #193 RED: `a_channel_that_would_cut_a_pit_off_from_its_rim_is_refused` panicked `the target's ramp stays:
    cutting it cages the dwarf in the pit  left: Some(Empty)  right: Some(Ramp(Stone))`; its twin
    `a_channel_beside_a_second_way_out_of_the_pit_is_worked` green before and after. Wire `same` on a pre-fix
    release daemon: `level 3: NEVER WORKED ... (2 of 10 channels left)`, two dwarves at z 5, `back up: NEVER`,
    `WAY DOWN RED`. Fix: the target's sides are read before the cut and the cut terrain is flooded.
  - #194 mechanism (dumped positions, `Path`, `Wander.home`, claims and jobs at the stall, probe in a scratch
    worktree): **M1**, delays 0-3 and 8 of a one-wide sweep, level 3 never worked: idle miner d4 finished a
    channel on `(63,62,6)`, the stair's last step, and parked there (home there). Holder d2 at `(62,62,7)` needs
    `(64,62,5)`, the only work cell, past it; hauler d3 (holding a job) is behind d2 at `(61,62,8)`. Case 1 found
    no aside, no exit (every other dwarf closed), no idle chain (d3 holds a job); the head-on was `(None, None)`;
    all waited forever. **M2**, delays 9-10, levels all worked but nobody back up: NOT a cage. Both miners idle
    at z 5 with routes of 4-5 steps to z 9; `wander` is same-z only and nothing asked them up, and no block
    needed lifting from that deep. Marking a channel up top brought one up within ~1,000 ticks.
  - #194 RED: `a_one_wide_staircase_reaches_four_levels_down_and_a_dwarf_climbs_back_for_every_delay` (every
    tick checked for a swap or shared tile) `left: [(0, 3, false), (1, 3, true), (2, 3, true), (3, 3, true),
    (8, 3, false)]  right: []`. Fix (no swap): with no exit and no idle shift, the idle blocker's way out runs
    back up the line; in the stuck head-on the holder backs out past the dwarves behind it, which yield to it
    head-on in turn. After: 24/24; AC 7 24/24; AC 8 green; the seeds `0..=16` pins unchanged (no pin moved).
  - gui RED: `a_channel_drag_over_the_campfire_does_not_light_its_tile`: `left: [[0, 0, 2], [1, 0, 2], [2, 0,
    2]]  right: [[0, 0, 2], [2, 0, 2]]` (the fire lit).
  - Mutations, run alone after commit (`RUST_TEST_THREADS=1`, one extracted table): 12-10b rows 6 (re-pointed),
    10, 11, 12, 13 (new); 12-9 rows 22, 28, 32 (re-pointed): **8 of 8 KILLED**.
  - Wire at `e000392` (scratch-worktree release build, one fresh daemon per run): `two` 5 of 5 OK; `onewide`
    5 of 5 OK (one after the up-top call); `same` 2 of 2 stop at `level 1: NEVER WORKED (1 of 10 channels left;
    the floor under [65, 63, 8] is {"ramp": "soil"})`, the refused cut kept as the step, nobody below z 7.
    Twelve daemons at once overloaded the readers (`client delta queue full; disconnecting client`): 4 runs
    printed `BROKEN`, not RED, and were re-run at four at a time.
  - Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` on `ab90c52`: **GATE GREEN 3565 s**, exit 0 (fmt, clippy, cargo test 460 s, pixel guards 3065 s, metrics, bench, mutation tables still apply).
- **Review patch pass, run 2 (2026-10-14).**
  - #195 RED: `a_channel_that_would_drop_the_dwarf_on_its_target_into_a_cut_off_pit_is_refused`: `the target's ramp
    stays: cutting it drops the dwarf into a caged pit  left: Some(Empty)  right: Some(Ramp(Stone))`. The
    second-link twin was green before and after.
  - #196 dump (release probe, DEFAULT_SEED, one-wide, level 0 marked at tick 190, each later level 8 after the last
    opens), level 2 frozen from tick ~885: hauler 1 at (62,62,7) `Haul` of the block at (65,62,7), path
    `[(63,62,7), (64,62,6)]`; miner 4 at (63,62,7), standing ON its own mark, path `[(62,62,7)]`; miner 2 at
    (61,62,8) behind the hauler. Head-on, both holding jobs, neither with an escape off the other's path, so
    `(None, None)`; the #194 back-out ran only `if idle`. The review's tiles exactly.
  - #196 RED: `job_holders_head_on_in_a_one_wide_staircase_do_not_wait_forever`: `levels worked; still marked at
    tick 6719: [(63,62,7), (65,62,7)]  left: 2  right: 4`.
  - #196 probe (scratch, release; first delay 0..300 step 10 x between-level delay 0..40, 1,200 runs per shape;
    a run is FROZEN when no claimed dwarf moved, no mark went and no item moved for 1,500 ticks):
    - before (`a197b90` and `31322f8`, identical): one-wide 4 frozen, two-wide 13 frozen;
    - re-route only (case 2's A* around every dwarf, tried first): one-wide 0, two-wide 10;
    - back-out for any blocker, with or without the re-route: one-wide 0, two-wide 0, and every run worked all 4
      levels by tick 12,000. The re-route added nothing, so it did not ship (YAGNI).
    - `same` (180 runs, never finishes by design): walking-line freezes 93 -> 1; refusal loops (a miner working a
      refused mark, #187) 18 -> 50, as lines no longer freeze before reaching the marks.
  - gui RED: `a_channel_drag_over_the_campfire_does_not_light_its_tile` `left: [[0,0,2],[1,0,2],[2,0,2]]  right:
    [[0,0,2],[2,0,2]]`; `the_channel_preview_skips_an_emitters_tile_and_cells_below_z_two` `the sim's stockpile
    filter drops the campfire's tile`; `the_channel_preview_skips_a_cell_standing_on_a_tree` `the cell standing on
    the trunk`.
  - Wire on `e6fd5b5` (release, fresh daemon per run, 4 at a time): `onewide` **10 of 10 OK**, 6 via the up-top
    call; `two` **10 of 10 OK**, 8 via the up-top call.
  - Mutations, run alone after commit (`RUST_TEST_THREADS=1`, extracted tables): 12-10b rows 5c, 10, 12 (re-pointed),
    14, 15, 16, 17, 18, 19 (new); 12-9 rows 22, 32 (re-pointed): **11 of 11 KILLED**, each on its intended assert.
    Row 18 first SURVIVED against the #196 fixture (no head-on yield there) and was re-pointed at the one-wide
    sweep. Row 19 is KILLED in debug by `World::step`'s debug_assert, not by `step_and_watch` (see above).
  - Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` on `2b4db37`: **GATE GREEN 4809 s**, exit 0 (fmt, clippy, cargo
    test 461 s, pixel guards 4305 s, edge probes, metrics, bench 20 s, mutation tables still apply).
  - Cost: $16.96 over 264 turns (`session_tokens.py --phase review-patch`).

### Completion Notes List
- (O) **AC 4 as shipped (Wolf's ruling at dev):** the block lands on the nearest free tile at the worker's
  level OR ABOVE, never below it, never on the target or in its column. "The worker's level" taken
  literally stalls the staircase (16/24); a pit worker lands his block up the ramp. AC 4 and Key decision 3
  carry the amendment inline (`83b3784`). The NOTE beside the rule names the limitation.
- (O) The premise "both work-position changes are load-bearing" (Key decision 2) no longer holds for the
  staircase after the AC 4 ruling: with blocks landing up top, same-level lifts still reach 24/24. The
  cross-ramp lift stays (AC 6 asks for it) and now has its own test.
- (O) `dbbebd0` "Start 12.10b: mark it in-progress" also carries Agent A's Task 1 RED tests: they were
  staged when the orchestrator committed. Not rewritten (the record says so instead).
- (O) Two direct tests added for rules no instrument reached after the change: AC 6's cross-ramp lift
  and the channel landing's wall-in refusal (both found by surviving mutations).
- (O) Docs: the spine's 12.10 bullet keeps its text, its last sentence marked superseded, and a 12.10b
  bullet states the channel rule, D1, the landing and the refusals. README: one paragraph, dig vs channel.
  Seat card: `12-10b-signoff/vehicle-card.md`.
- (B) Task 4: new `make_room_down_the_idle_chain` (lib.rs, beside `step_aside`), called from `resolve_blocked_step` case 1 only when the idle blocker has neither a free tile aside nor an exit route. It searches through the other IDLE dwarves (never the holder, never a dwarf with a job) for the nearest free tile off the holder's path and gives the idle dwarf nearest that tile the rest of the route as its `Path`; the holder waits meanwhile and the line shifts a dwarf at a time. No swap, no new state, no change to the holder-wins/yield rules. No mutation row re-pointed (`audit-mutations.py` green); a row for this fix is Task 7's.
- (A) Tasks 1, 2, 3, 5 and the code half of 6 done. Task 1's RED commit was swept into the orchestrator's `dbbebd0` (it committed while my files were staged); the content is mine.
- (A) **Deviation 1, the wall-in check moved after the cut (channel only).** The story said keep the check and re-state its NOTE. Kept, but it reads the terrain AFTER the step is cut (see Debug Log): read before the cut it refused a pit worker forever (12.9 seed sweep seeds 0 and 4, AC 7 delays 8 and 23). The tiles are undone if the check or the landing refuses. Dig is unchanged. No test names this alone: AC 7 (24/24) and the sweep pin `UNFINISHED = []` kill it.
- (A) **Deviation 2, none on marked cells.** The probe's drop refused DESIGNATED cells; I did NOT add that, to the shared helper or to the channel landing (the story's ruled shape). 12.9's ordinary drop is unchanged. A block can land on a cell a later level will mark; the probe's "floating blocks" are the same family (12.11).
- (A) The landing is `nearest_free_cell(terrain, blocked, worker_pos, refused)`, shared by `release_claim` and the channel; the channel refuses a dwarf's tile, a cell that walls a dwarf in, and the target's column (the pit is reachable once cut). Item cells are in `blocked`, so a taken pile cell is never chosen. The item is spawned at the landing directly (no `Carrying`).
- (A) The `step_aside` target-occupant block is gone. A dwarf standing on a channel target now falls into the pit with `settle`. `step_aside` is still used by `resolve_blocked_step`.
- (A) Task 3 consumers: `a_channelling_miner_faces_the_target_cell` is a synthetic same-z fixture and `dig_yaw` is yaw-only, so it is unaffected and unchanged. `serve.rs` `deltas_label_a_miners_dig...`: the "same z" assert is loosened to `|dz| <= 1` (re-pin, disclosed beside the pin).
- (A) Re-pins (each disclosed beside the pin): lib.rs `execute_jobs_channels_a_material_preserving_ramp_and_spawns_stone` -> `execute_jobs_opens_a_step_down_and_lands_the_stone_up_top` (z-1 Empty, z-2 Ramp, stone beside the worker); `execute_jobs_removes_a_channel_job_when_the_support_is_already_a_ramp` -> `execute_jobs_opens_a_channel_whose_support_is_already_a_ramp` (a ramp is dug out, yields; z-2 ramp stays); scenario `a_channel_is_worked_from_the_next_tile_and_the_stone_lands_on_the_target` -> `..._lands_beside_the_worker`; `a_channel_worker_whose_support_is_removed_lets_go_and_the_crew_goes_on` (last assert: worked means z-1 Empty, was Ramp); `channelling_over_ore_yields_the_ore_and_leaves_a_stone_ramp` -> `..._opens_a_step_with_its_own_ramp` (D1: the ramp keeps its ore material, retiring "channel over ore leaves Ramp(Stone)"); `no_dwarf_is_caged_by_stones_over_the_probe_seeds` pins `ONE_LEVEL_ISLANDS` [(10,4,2),(10,6,3),(16,4,0)] -> [(4,4,4),(10,6,4)] and `UNFINISHED` (14 runs) -> []; serve.rs as above.
- (A) Task 5 re-pins (found by the fast gate, not by me beforehand): `designation_rect_clips_to_world_bounds` (channel rect at z 1 -> z 2, since z < 2 is refused); `a_cut_over_no_tree_is_refused_and_a_dig_never_marks_a_tree` (its "ordinary ground" channel was the camp cell, the campfire's tile, now refused: uses the cell beside it); the 12.9 seed sweep `ONE_LEVEL_ISLANDS` loses (10, 6, 4) once the campfire tile is refused (both sweep rects cover it), final pin `[(4, 4, 4)]`, `UNFINISHED` still `[]`. `3-2-the-dig.sh` "Designate ignores the diggability filter" re-pointed to the new trailing channel filter.
- (A) Mutation rows the change broke (the pre-commit audit blocked the commit; `--no-verify` was refused) are re-pointed in `mutations/12-1.sh`, `12-10.sh`, `12-9.sh`, `3-2-the-dig.sh` (rows: drop search walks through rock; 12-10 #1, #6 renamed "the channel's ramp loses its ore"; 12-9 #12, #15, #25, #28, #32; 3-2 channel writes Empty / loses the material / stone is not spawned / "stale channel job" renamed "a channel over a ramp is a no-op"). They APPLY (audit green) but were NOT run: Task 7 must run them.
- (A) Not re-pinned and still green: the walking-skeleton test and every other sim-core, gui lib and `headless::a_channelling_miner_faces_the_target_cell` test.

### File List

- (O) `crates/sim-core/src/lib.rs` (AC 4 refusal + its test, AC 6 test), `crates/sim-core/tests/scenario.rs`
  (island re-pin), `README.md`, `_bmad-output/implementation-artifacts/mutations/12-9.sh` (row 28 re-pointed),
  `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`,
  `_bmad-output/implementation-artifacts/mutations/12-10b.sh` (NEW),
  `_bmad-output/implementation-artifacts/12-10b-signoff/vehicle-card.md` (NEW),
  `_bmad-output/implementation-artifacts/12-10b-a-way-down.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

- (B) `crates/sim-core/src/lib.rs` (Task 4: `make_room_down_the_idle_chain`, fixture un-ignored with a bound comment)

- `crates/sim-core/src/lib.rs`, `crates/sim-core/tests/scenario.rs`, `crates/simd/tests/serve.rs`, `crates/gui/src/project.rs`
- `_bmad-output/implementation-artifacts/mutations/{12-1,12-10,12-9,3-2-the-dig}.sh` (re-pointed)
- Run 2 patch pass: `crates/sim-core/src/lib.rs`, `crates/sim-core/tests/scenario.rs`, `crates/gui/src/project.rs`,
  `mutations/12-10b.sh`, `mutations/12-9.sh` (rows 22, 32 re-pointed), `12-10b-signoff/{vehicle-card.md,way_down_wire.py}`

## Change Log

| Date | Change |
|---|---|
| 2026-10-14 | Review patch pass, run 2: all 9 items landed (`31322f8`..`2b4db37`). #195 (a dwarf on the target counted where it falls), #196 (the #194 back-out runs for a busy blocker too; release probe 1,200 runs per shape, one-wide 4 -> 0 and two-wide 13 -> 0 frozen; wire `onewide` 10/10), M2 accepted (card line, `two` up-top call; wire `two` 10/10), the gui stockpile/tree mirror, the card, `step_and_watch` shown firing. 11 mutation rows KILLED. Full gate GREEN 4809 s. Status -> review for round 3; AC 12 (the seat) still unrun. |
| 2026-10-14 | Code review run 2 (four layers, none timed out, all ran): 2 HIGH (#195, a dwarf on the target is caged; #196, a one-wide line of job holders jams), 3 REWORK rows of the closure table. Wolf ruled #196 fixed in 12.10b, and miners staying at the bottom accepted (card line + `two` up-top call). 8 patches left as action items, 5 deferred, 1 dismissed. Status -> in-progress. |
| 2026-10-14 | Review patch pass: all 7 patches landed. #193 (a pit-severing cut is refused), #194 (a one-wide line backs out past an idle dwarf, no swap; one-wide sweep 24/24, wire 5/5), the gui preview filter, the wire's BROKEN path, spine, AC 4 and the seat card. 8 mutation rows KILLED. Status -> review for round 2; AC 12 (the seat) still unrun. |
| 2026-10-14 | Code review run 1 (four layers, none timed out): 1 HIGH (#193, a channel cages dwarves in a pit), #194 ruled into the story, 7 patches left as action items, 11 deferred, 4 dismissed. Status -> in-progress. |
| 2026-10-13 | Agent A: Tasks 1, 2, 3, 5 and the code half of 6 (channel rule, shared landing helper, work positions across ramp steps, designation filter, re-pins). AC 3 green; AC 7 sweep 24 of 24; #185 fixture still RED for Task 4. |
| 2026-10-13 | Orchestrator: full gate GREEN on `79a210b` (3508 s); old re-pointed rows 12 of 13 KILLED, row 28 pinned by a new direct test and KILLED. Status -> review. |
| 2026-10-13 | Orchestrator: wire 5 of 5 OK; AC 4 measured (40% of landings off the worker's level) and ruled by Wolf: never below the worker (`4df25a3`); AC 6 direct test; `mutations/12-10b.sh` 11 of 11 KILLED after re-pointing 2 survivors; spine, README, seat card. |
| 2026-10-13 | Agent B: Task 4. The #185 fixture jam was an idle line in a one-wide passage that nothing asked to shift; `make_room_down_the_idle_chain` sends the end of the line out. AC 8 green, AC 7 24 of 24, no sweep pin moved. |
| 2026-10-13 | Created on `015548f`. #163 confirmed in source. Task 0 ruled by Wolf: a channel opens a step down (`z-1` Empty, `z-2` Ramp), the block lands up top beside the miner, and the dig-face jam (#185) is folded in. Probe: the rule plus two work-position changes reaches 4 levels on a two-wide stair, but jams at the dig face (wire 1 of 5, sweep 16 of 24). `way_down_wire.py` written, RED on `015548f`. Defaults D1-D3 (ramp keeps material, z<2 and emitter marks refused; #188). |
