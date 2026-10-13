---
baseline_commit: 015548f
model: claude-opus-5-5  # session default, same as 12.1-12.10's creation
---

# Story 12.10b: A Way Down

Status: ready-for-dev

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

- [ ] **Task 1: RED tests first** (AC 1, 3, 7, 8)
  - [ ] `crates/sim-core/tests/scenario.rs` (UPDATE):
    `a_channel_opens_a_step_down_and_a_dwarf_walks_down_and_back` (AC 3; the probe's `walk_down_and_back`)
    and `a_two_wide_staircase_reaches_four_levels_down_for_every_designation_delay` (AC 7; the probe's
    `staircase_delay_sweep`, asserting 24/24 and printing the stalled delays on failure). Take the site
    finder, pile rect and stair geometry from the probe; drop every `PROBE_*` toggle.
  - [ ] #185's fixture as a test in `lib.rs`'s test module beside `room_and_tunnel` (`lib.rs:5532`):
    the dig at `ROOM_WALL` done within a stated bound.
  - [ ] Run each on `015548f` and paste the failing assert lines into Debug Log References. Commit the RED
    tests with `#[ignore = "RED until Task N"]`, un-ignored by the commit that greens them.
- [ ] **Task 2: the channel rule** (AC 2, 4)
  - [ ] `execute_jobs`' `JobKind::Channel` arm (`lib.rs:1665-1680`): `below = target.z-1`, `base = z-2`.
    `below` `Solid(m) | Ramp(m)` → `Empty`, yield `yield_of(m)`; `base` `Solid(m)` → `Ramp(m)`; `Ramp` and
    `Empty` unchanged. Both tiles change in the same tick, through `Terrain::set_tile` (both dirty).
  - [ ] Landing: extract the breadth-first "nearest free walkable cell" search from `release_claim`
    (`lib.rs:1188-1231`) into one helper and call it from both; the channel's item lands by it from the
    worker's position. Remove the target-occupant `step_aside` block (`:1699-1713`) if nothing spawns on the
    target any more, and its NOTE.
  - [ ] Keep the `walls_in_a_dwarf` check (the probe broke without it). Its NOTE (`:1698`, "reads the
    terrain before the ramp is cut") is now about the pit: re-state what it reads, and confirm in a test
    that it does not refuse the staircase's own cells forever (AC 7 covers this).
- [ ] **Task 3: work positions across a ramp step** (AC 5, 6)
  - [ ] `work_positions` (`lib.rs:1089-1158`): `JobKind::Channel` uses `astar_neighbours(terrain, blocked,
    job.target)` in place of same-z `side_neighbours`; the haul pick-up leg uses
    `astar_neighbours(terrain, blocked, *pos)`. Keep the "target standable" gate and the delivery leg.
  - [ ] Check every consumer that assumed same-z: the gui's facing (`a_channelling_miner_faces_the_target_cell`,
    `headless.rs:5313`) and `serve.rs`'s "same z, Manhattan distance 1" assert from 12.9 (`0fccc74`).
- [ ] **Task 4: the dig-face jam** (AC 7, 8)
  - [ ] Reproduce from the sweep's RED delays first: dump positions, `claims()`, `jobs()` and items at the
    stall (the probe's dump code) and name the mechanism before changing any rule.
  - [ ] Fix inside `resolve_blocked_step` / `step_aside` / claiming (`lib.rs:1285-1449`, `:479-560`).
    #185's candidate directions: the job holder wins the yield over idle dwarves; a dwarf moved aside does
    not keep a home inside a one-wide passage. **No swap** (Wolf, 12.9 Task 0 Q1b): if the only fix is a
    swap, stop and ask Wolf.
  - [ ] The judge is AC 7's 24/24 plus AC 8. Then run the 12.9 sweep over seeds `0..=16` and record every
    pinned outcome that moved.
- [ ] **Task 5: designation filter** (AC 9)
  - [ ] `apply_command`'s `DesignationKind::Channel` filter (`lib.rs:2365-2371`): also refuse an emitter's
    tile and `pos.z < 2`. A rect left with zero cells is a `Refusal` (standing AC 6), as today.
  - [ ] gui `sim_will_keep` (`crates/gui/src/project.rs:1076-1086`): mirror both, so the preview lights none
    of them.
- [ ] **Task 6: re-pins and docs** (AC 11)
  - [ ] Expect to move: `execute_jobs_channels_a_material_preserving_ramp_and_spawns_stone` (`lib.rs:5039`),
    `execute_jobs_removes_a_channel_job_when_the_support_is_already_a_ramp` (`:5090`), scenario
    `a_channel_is_worked_from_the_next_tile_and_the_stone_lands_on_the_target` (`scenario.rs:949`),
    `a_channel_worker_whose_support_is_removed_lets_go_and_the_crew_goes_on` (`:854`),
    `channelling_over_ore_yields_the_ore_and_leaves_a_stone_ramp` (`:3219`), and any 12.9 sweep pin.
    Disclose each beside the pin and in its commit (12.9 rule).
  - [ ] `ARCHITECTURE-SPINE.md` (2026-08-09): amend the 12.10 bullet (`:157-166`) for D1 and add the channel
    rule. README: one line under the controls saying dig = sideways at your level, channel = a step down.
- [ ] **Task 7: the instrument and its RED** (AC 10)
  - [ ] `12-10b-signoff/way_down_wire.py` (EXISTING, creation) is the instrument. Run it on 5 fresh release
    daemons; paste each verdict block.
  - [ ] NEW `mutations/12-10b.sh`, each row named against its killing test, run alone after commit:
    channel worked from the same level only (kills AC 7; the wire goes RED at level 1); pick-up from the
    same level only (AC 7; wire RED at level 2); `base` left `Solid` (AC 3); the item spawned on the target
    (AC 4); the emitter refusal dropped (AC 9); the Task 4 fix reverted (AC 7 and AC 8). Re-point any old
    row the change breaks (`audit-mutations.py`).
- [ ] **Task 8: the seat card** (AC 12)
  - [ ] NEW `12-10b-signoff/vehicle-card.md`, in the seat's launch form. Teach: dig vs channel in one
    sentence each; drag a BIG pile first (4 levels two wide = 28 blocks); channel a 2-wide row of 5 at the
    camp level, then each next row one shorter and one level down inside it. Give the site, the z per row,
    the compass bearing at boot, and the tui `--z` and keys. Known, by design: blocks can float until
    12.11; one-wide passages are slower.
- [ ] **Task 9: gate** — `RUST_TEST_THREADS=1 scripts/gate.sh`, full, green, before review.

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
   step (Task 3), and the pile must hold it: every fixture places the creation pile.
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

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-10-13 | Created on `015548f`. #163 confirmed in source. Task 0 ruled by Wolf: a channel opens a step down (`z-1` Empty, `z-2` Ramp), the block lands up top beside the miner, and the dig-face jam (#185) is folded in. Probe: the rule plus two work-position changes reaches 4 levels on a two-wide stair, but jams at the dig face (wire 1 of 5, sweep 16 of 24). `way_down_wire.py` written, RED on `015548f`. Defaults D1-D3 (ramp keeps material, z<2 and emitter marks refused; #188). |
