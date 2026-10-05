---
baseline_commit: 98149e1
model: claude-opus-5-5  # session default, same as 12.1-12.5's creation
---

# Story 12.6: Wolf Gives Them Their Trades

Status: ready-for-dev

## Story

As the boss,
I want to select a dwarf and change their trade,
so that I can shape my crew to the work I want done.

## Not stacked: branch off `main`

`main` is `98149e1` (PR #165 merged), clean. Branch `story-12-6-wolf-gives-them-their-trades` carries
this file, `12-6-signoff/` and the board edit. 12.5 is already `done` on `main`, so the board edit flips
nothing else. Epic 12's nine standing ACs (`epics.md`, "Standing acceptance criteria") bind this story and
are not restated. "Wire diff" below satisfies standing AC 4. Standing AC 7 applies: Wolf approves the look
draft (`12-6-signoff/draft.md`) at Task 0, before any gui display is built.

## Found at creation (2026-10-05, on `98149e1`)

- **No command can change a trade.** `Profession` is its own component "because 12.6 makes it mutable"
  (`sim-core/src/lib.rs:198-204`). The only writer after spawn is the test helper `set_profession`
  (`:1984`, "no public setter until 12.6 adds the command").
- **Live RED** (`12-6-signoff/trade_wire.py`, fresh release `simd 7495`, DEFAULT_SEED, fast4x): at tick 18
  miner 2 holds channel `[65, 63, 9]`, and the script sends `set_profession` for dwarf 2 (→ hauler) and for
  dwarf 999. The daemon logs `unrecognized client message: {"type": "set_profession", …}` twice. Then:
  `profession reads hauler on the wire: NEVER`, `old channel job still held after the command: tick 21`,
  `released target … reclaimed by a miner: NEVER`, `refusal for dwarf 999: NEVER`, `TRADES WIRE RED`,
  exit 1.
- **A released job goes back to the queue already.** `release_claim` (`lib.rs:883`) clears `CurrentJob`,
  drops a carried stone where the dwarf stands (avoiding an occupied pile cell), sets `Idle`, resets
  `Wander.home`, and removes `Path` and `WorkProgress`. It never removes the job from `Jobs`. It sets no
  `retry_after`: that is `retry_claim` (`:963`). So the next `claim_jobs` (`:444`) can hand the job to any
  free dwarf whose trade matches (`:508`).
- **A held job is always of the holder's own trade**, because `claim_jobs` filters on
  `trade(job.kind) != profession` (`:508`). A trade change therefore always lets go of a held job.
- **A trade with zero dwarves is silent.** No dwarf passes the filter, so `attempted` stays false: no
  cooldown stamp, no log, and the jobs wait forever (12.4 review NOTE, `epics.md:2585`). The spawn pool is
  2 miners, 2 haulers and 1 woodcutter (`lib.rs:1384-1391`). Task 0.2 decides what the player sees.
- **The gui has no profession display at all.** Every gui fixture sets `profession: None`. The name slot
  (`NameReadout`, `ingest.rs:1737-1787`, top 48 px, right 16 px) is empty when nothing is selected. The
  capture hides every `Hud` text (`hide_hud_for_capture`, `:1888`), so a `--capture` PNG can never show the
  roster. The real-binary instrument has to be stderr.
- **`T` is free.** `the_client_keymap_avoids_keys_other_plugins_have_claimed` (`ingest.rs:4316`, table
  `:4346-4384`) binds F3, C, Comma, Period, Space, Equal, Minus, the numpad +/-, A, D, W, S, L, H, E, Q,
  Shift, Ctrl, Escape and Digit1-4. A per-trade letter cannot work: `H` is the HUD toggle.
- **The tui already follows a trade change.** `each_trade_word_sits_after_its_own_dwarfs_name_in_grey_and_follows_a_profession_change`
  (`tui/tests/client.rs:1766`) pins the roster changing on a delta. Its refusal row shows
  `client_core::refusal_text` (`view.rs:423-426`), so a new `Refusal` variant reaches it through that one
  function.

## Wire diff (standing AC 4)

- NEW `protocol::Command::SetProfession { dwarf: u32, profession: Profession }`. The wire form is
  `{"type":"set_profession","dwarf":2,"profession":"hauler"}`. It is world-mutating and rides the AD-10
  queue. It carries no rect: `command_rects` (`simd/src/main.rs:798`) returns `None` for it.
- NEW `protocol::TradeRefusal { NoSuchDwarf, LastOfTrade }` (snake_case), and NEW
  `protocol::Refusal::SetProfession { dwarf: u32, reason: TradeRefusal }`. With the existing
  `#[serde(tag = "command")]` it reads `{"command":"set_profession","dwarf":999,"reason":"no_such_dwarf"}`.
  - **If Task 0.2 rules option 3**, `LastOfTrade` is not added.
- `Entity`, `Snapshot` and `Delta` do not change. The trade is already `Entity.profession` (12.4), and a
  change shows on the next delta. Every existing pin stays byte-identical.
- **No save change.** `Profession` is already saved (`SavedDwarf.profession`, `lib.rs:1424`).

## Acceptance Criteria

1. The real daemon is the judge. `set_profession` for an existing dwarf changes his `profession` on the
   next delta. A refusal never changes it.
2. A dwarf reassigned while he holds a job lets go of it in the same command.
   - The job stays in the queue with the same `JobId` and is never dropped (M1 FR8). A free dwarf of the
     job's trade claims it within the bound the scenario test names.
   - A reassigned hauler carrying a stone puts it down: it is no longer carried, and his haul job still
     names that stone. Another hauler later delivers it to the pile.
   - A command that leaves his trade unchanged lets him keep his job.
3. The sim refuses a `set_profession` it cannot apply, and the refusal reaches every attached client on
   the next delta (12.1's shape):
   - an unknown dwarf id gives `no_such_dwarf`;
   - per Task 0.2's ruling (options 1 and 2), the last dwarf of a protected trade gives `last_of_trade`,
     and his trade does not change.
4. Determinism (standing AC 3): seed plus a command log that includes `set_profession` gives identical
   state, `professions()` included. A world saved after a reassignment loads with the new trade and steps
   identically to the never-saved world.
5. The gui, with no dwarf selected, shows the crew roster in the name slot: one line per dwarf in id order,
   his name in his tunic colour and his trade in grey (`draft.md` §1).
6. The gui, with a dwarf selected, shows his name and his trade, and says which key changes it
   (`draft.md` §2).
7. `T` with a dwarf selected sends exactly one `set_profession` for that dwarf, with the next trade in the
   order miner → hauler → woodcutter → miner. `T` with nothing selected sends nothing. The readout shows the
   trade the mirror holds, never the requested one.
8. A `set_profession` refusal shows in the gui's refusal slot and in the tui status row, with the text
   `client_core::refusal_text` gives it. In the gui it clears on the next world command, `T` included.
9. The instrument is the real `gui` binary, judged by a real `simd`.
   - `--select ID --trade TRADE` sends the same command `T` builds, once, after the snapshot. It prints
     `gui sent set_profession dwarf {id} {trade}`.
   - Every change of a dwarf's wire profession after the snapshot prints `gui dwarf {id} trade {trade}`.
     It is printed from the mirror, so it only ever reports the daemon's word.
   - A real-binary test sees both lines for dwarf 2 → hauler. Its deliberate RED is the daemon ignoring
     the command: no `trade` line.
10. TUI: no new input and no regression (NFR10). Its roster follows a trade change, and its status row
    shows a trade refusal.
11. At the seat, Wolf sees the roster with nothing selected. He selects a miner who is digging, presses
    `T`, and sees the trade change in the gui and in an attached tui. The mark he was digging is taken by
    the other miner. Per Task 0.2, he also sees the refusal for the last of a trade.

## Tasks / Subtasks

- [ ] **Task 0: Wolf's rulings, at creation. The dev does not start Tasks 4-5 before 0.1 is ruled.**
  1. **The look draft** `12-6-signoff/draft.md`: roster in the name slot, the selected dwarf's name and
     trade line, `T` cycles, refusal text. Approve, or say what to change.
  2. **What the player sees when a trade is left empty.** Recommended: **option 1**.
     - **Option 1: refuse the last of any trade.** "The crew always has every trade": the FR43 spawn
       guarantee becomes a rule of play. It allows 3/1/1, never 4/1/0. The woodcutter stays a woodcutter
       and idles until 12.7. One rule, and it is loud.
     - **Option 2: refuse the last miner or hauler only.** The woodcutter can be made a miner now (3/2/0).
       12.7 must then decide what an empty woodcutter trade does, and extend the rule.
     - **Option 3: allow anything.** The roster is the only signal: no name says "miner", and the digs
       wait silently. `claim_jobs` stays silent, with a `// NOTE:`.
  3. **The refusal text is fixed** (`"trade refused: last of his trade"`, `"trade refused: no such
     dwarf"`), because `refusal_text` returns `&'static str`. Naming the dwarf would change that function
     for both clients. Recommended: keep it fixed.
- [ ] **Task 1: protocol (wire diff).**
  - [ ] `protocol/src/lib.rs`: `TradeRefusal` beside `Refusal` (`:148`); `Refusal::SetProfession`;
    `Command::SetProfession` (`:154`). `Command` stays `Clone` (not `Copy` since 12.1); `Refusal` stays
    `Copy`.
  - [ ] Pins, written as hand literals like `refusal_wire_is_literal_and_empty_delta_wire_is_unchanged`
    (`:309`): the command literal and each refusal literal parse and re-serialise byte-identical. The
    existing pins are untouched. `every_material_and_tile_variant_has_a_pinned_wire_name` gains a
    `TradeRefusal` block.
- [ ] **Task 2: sim-core (AC2-AC4). RED first: write the scenario tests against a `set_profession` that
  does nothing, and record the failures.**
  - [ ] `SimCommand::SetProfession { dwarf: Id, profession: Profession }` (`lib.rs:91`). `sim_core::Refusal`
    (`:99`) gains `SetProfession { dwarf: Id, reason: TradeRefusal }`, and sim-core gains `TradeRefusal`
    (AD-6: sim-core is the source of truth).
  - [ ] `apply_command` (`:1614`): dispatch `SetProfession` **before** the rect prelude (`:1616-1621`) to a
    private `fn set_profession(&mut self, dwarf: Id, profession: Profession) -> Option<Refusal>`.
    - Leave the four rect arms byte-identical: rows in `3-1-give-the-order.sh` and `3-2-the-dig.sh`
      quote them.
    - Rule order: unknown id → `NoSuchDwarf`; same trade → `None`, nothing changes; last of a protected
      trade (Task 0.2) → `LastOfTrade`; else insert the new `Profession`. If he holds a job with
      `trade(job.kind) != profession`, call `release_claim` (`:883`). Do NOT use `retry_claim`: a cooldown
      would delay the right-trade claim for no reason.
    - `// NOTE:` the released job's `WorkProgress` is lost; the next miner starts the dig from 0.
  - [ ] `sim-core/tests/scenario.rs`, new tests (DEFAULT_SEED or 42, as the neighbours do):
    - `a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job`: one reachable channel mark;
      step until a miner holds it AND `tick >= job.created_tick + 31` (every reaction delay is 5-30 ticks
      from `created_tick`, `lib.rs:415-430`, so the other miner is already eligible; assert he is idle
      first). Reassign the holder to hauler. After **one** step, assert that the holder has no
      `Dig`/`Channel` job, that the `JobId` is still in `jobs()`, and that the other miner holds that same
      `JobId`. Then assert the tile is eventually channelled. A one-step bound is what lets row 4 (a
      20-tick `retry_claim` cooldown) die.
    - `a_reassigned_hauler_puts_the_stone_down_and_another_hauler_delivers_it`: step until a hauler
      carries; reassign him to miner; assert `carrying()` is `None` for him, the stone is in `items()`, the
      haul job with that item is still in `jobs()`, and the stone later reaches the pile (reuse
      `stone_on_the_pile`, `:1667`).
    - `a_trade_change_to_the_same_trade_keeps_the_job`.
    - `unknown_dwarf_and_last_of_a_trade_are_refused_and_change_nothing` (options 1/2): the returned
      `Refusal`, and `professions()` unchanged. Under option 1 the woodcutter is refused, the first miner
      is accepted and the second is refused.
  - [ ] Determinism: `same_seed_and_commands_remain_deterministic` (`scenario.rs:1755`) gains a
    `SetProfession` in its command list and asserts `professions()` each step. `save_load.rs` gains a
    round trip after a reassignment, asserting `professions()` (pattern at `:150-180`).
- [ ] **Task 3: simd + client-core (AC1, AC3, AC8).**
  - [ ] `simd/src/main.rs`: a `Command::SetProfession` arm in the loop (`:224-243`) that pushes the sim's
    refusal into `refusals`, like `PlaceStockpile` (`:235-237`). `command_rects` (`:798`) returns `None`
    for it.
  - [ ] `simd/src/bridge.rs`: `profession_in` (an exhaustive `match`, beside `profession_out`, `:233`) and
    `refusal_out` (`:277`) arms for `SetProfession`, plus `trade_refusal_out`. No wildcard arms.
  - [ ] `simd/tests/serve.rs`: a real daemon. A test client designates the channel block and the pile, as
    `trade_wire.py` does. When a miner holds a channel job it sends `set_profession` to hauler and to dwarf
    999. On deltas, assert: his `profession` is `hauler`; he never again holds a `dig`/`channel` job;
    another miner later holds that target; and a refusal `{"command":"set_profession","dwarf":999,
    "reason":"no_such_dwarf"}` arrives.
  - [ ] `client-core/src/lib.rs` `refusal_text` (`:10`): arms for both reasons, using the texts in Task
    0.3. A unit test pins every variant's text, like `every_profession_has_its_one_spelling` (`:375`).
- [ ] **Task 4: gui roster, readout and the `T` key (AC5-AC8). Look per Task 0.1.**
  - [ ] `ingest.rs` `update_name_readout` (`:1760`):
    - nothing selected → the roster;
    - selected → name, then trade and `T: change trade`.
    - Colours: name in `dwarf_tunic_color`, trade and key text grey `(150,160,170)`, as a named const
      beside the tunic table in `appearance.rs`. Use `TextSpan` children for the per-name colours.
    - The 12.2 row `hud never shows the name` (`mutations/12-2.sh:64`) quotes `:1774-1775`. If those lines
      change, re-point the row and re-kill it.
  - [ ] A `T` system next to `save_load_keys` (`command.rs:316`): with `SelectedDwarf` (`pick.rs:67`) set
    and his mirror profession known, push `Command::SetProfession` with the next trade into
    `PendingCommands` (`command.rs:21`). Clear `LastRefusal` (`ingest.rs:2326`) as `designation_input` does
    (`designate.rs:196`).
    - Put the "next trade" order in one pure fn, which `--trade` does not need.
    - Add `KeyT` to the keymap test table (`ingest.rs:4346-4384`).
  - [ ] `--trade <miner|hauler|woodcutter>` in `Args` (`:1008`) and the parser (beside `--select`,
    `:1256`): it requires `--select`, else a parse error; an unknown trade name is a parse error. Push it
    once, after the snapshot. `refuse_select_of_a_missing_dwarf` (`:2549`) already fails a bad id.
  - [ ] The two stderr lines in AC9. The `trade` line compares each dwarf's profession with
    `Mirror::previous_entity` (`client-core/src/lib.rs:176`) or with a `Local` map; it prints nothing for
    the snapshot. `// NOTE:` chatty by design, as 12.5's clip line is.
  - [ ] Tests (ingest.rs unit tests, `configured_app` with its real TCP pair, `:2975`; or `headless.rs`):
    - the roster lists five `Name trade` lines in id order, the names in tunic colour, and follows a delta
      that changes one trade;
    - selected: name line plus `miner   T: change trade`; Escape brings the roster back;
    - `T` pressed: the bytes on the socket are exactly one `set_profession` line with the next trade.
      Pressing `T` with nothing selected writes nothing. The readout still says the OLD trade until a
      delta changes it;
    - a `SetProfession` refusal in a delta shows in the refusal slot, and `T` clears it;
    - update `the_name_hud_shows_the_selected_dwarfs_name_in_his_colour_and_clears` (`:3504`) to the
      two-line form.
- [ ] **Task 5: the instrument (AC9).**
  - [ ] `gui/tests/pixel_guard.rs` (`Daemon::spawn`, `:154`): NEW
    `a_trade_set_from_the_gui_comes_back_on_the_wire`. Run `gui <port> --headless --select 2 --trade hauler
    --frames N --capture <tmp>`, and assert stderr has `gui sent set_profession dwarf 2 hauler` followed by
    `gui dwarf 2 trade hauler`. Set N from a measured run ([[frame-counts-are-venue-calibrated]]).
    Dwarf 2 is Bifur, a miner, and one of two, so options 1 and 2 accept the change.
  - [ ] Its deliberate RED is the simd `SetProfession` arm doing nothing: no `trade` line, and the test fails
    by name. It cannot be a `mutate.sh` row, because `cargo test -p gui` does not rebuild `simd`; run it in
    a scratch worktree with its own target dir, as 12.5 did. Record the output in the Debug Log.
- [ ] **Task 6: tui (AC10).** No code change is expected beyond the `refusal_text` arms. Add a
  `tui/tests/client.rs` capture with a `SetProfession` refusal on a delta and assert the status row text
  (`capture_refusal_frames`, `:205`, is the pattern). The existing trade-change test (`:1766`) covers the
  roster.
- [ ] **Task 7: the record.**
  - [ ] `_bmad-output/implementation-artifacts/mutations/12-6.sh`. Each row is KILLED by the test named:
    1. `set_profession` changes nothing → the reassigned-miner scenario test;
    2. a reassignment keeps the held job (no `release_claim`) → the reassigned-miner scenario test;
    3. the released job is removed from `Jobs` instead → the reassigned-miner test's "same `JobId`" assert;
    4. `retry_claim` instead of `release_claim` → the reassigned-miner test's one-step claim assert;
    5. the hauler keeps carrying when reassigned → the reassigned-hauler test;
    6. the unknown id is not refused → the refusal test, and the serve test;
    7. the last of a trade is not refused (options 1/2) → the refusal test;
    8. the bridge maps `hauler` to `miner` in `profession_in` → the serve test;
    9. `T` sends the current trade, not the next → the `T` socket test;
    10. the roster drops the trade word → the roster test;
    11. the readout shows the requested trade before the wire says so → the `T` socket test's "old trade"
        assert;
    12. `--trade` pushes nothing → `a_trade_set_from_the_gui_comes_back_on_the_wire`.
  - [ ] Spine `architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` AD-10 (`:155-156`): add
    `set_profession` to the list of world-mutating commands, with an "Amended 2026-10-xx (Story 12.6)"
    line. This is the first selection-driven command.
  - [ ] README gui section: the roster, `T`, `--trade`, and the two stderr lines. Tui section: trade
    refusals show on the status row.
- [ ] **Task 8: the live recipe, the seat (AC11), then the full gate.**
  - [ ] Run the Verification recipe: GREEN, then the deliberate RED. Record both outputs.
  - [ ] Write `12-6-signoff/vehicle-card.md` in the seat's launch form. Say where to look: the roster is
    top-right under the clock, and refusals are bottom-left.
  - [ ] Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` (~55 min, [[gate-ooms-at-default-parallelism]]).

## Dev Notes

### Scope guardrails (do NOT)

- No new tui input (NFR10). No tui display change beyond the refusal text.
- No mouse UI, buttons or menus: one key, `T`.
- No new wire field on `Entity`, `Snapshot` or `Delta`. No save format change.
- No claiming change: `claim_jobs` stays as is (AD-12). Option 3 adds only a `// NOTE:`.
- No cut job: woodcutters stay jobless until 12.7.
- No fix for #164 (work visuals trailing the walker) or #162 (stones without collision).

### What already exists (build on it)

- `release_claim` (`lib.rs:883`) is the one funnel for letting go of a job. It drops a carried stone
  correctly and resets wander home.
- 12.1's refusal path: `Option<Refusal>` from the sim → `bridge::refusal_out` → `Delta.refusals`
  (broadcast) → `LastRefusal` in the gui and `ViewState.refusal` in the tui → `refusal_text`.
- `SelectedDwarf`, `--select` and `refuse_select_of_a_missing_dwarf` (12.2). `PendingCommands` and
  `send_commands` (`command.rs:12-43`, `:374`).
- `client_core::profession_text` (`:39`) and `dwarf_name_text` (`:17`): one spelling for both clients.
- `trade_wire.py`, the daemon-judged recipe, RED at creation.

### Key decisions & traps

- **The readout reads the mirror, not the key.** If it showed the requested trade, a daemon that ignored
  the command would look like it worked: the silent-filter trap ([[silent-sim-filter-trap]]). Mutation
  row 11 holds this.
- **The `trade` stderr line must come from the mirror.** Printed at send time, it would pass with a dead
  daemon arm. That is why the `sent` line and the `trade` line are separate.
- **`release_claim`, not `retry_claim`.** The job did not fail. A cooldown would delay the other miner by
  20 ticks.
- **Dispatch `SetProfession` before the rect prelude** in `apply_command`, or the rect `match` at
  `:1616-1621` has no rect to take, and every existing arm-quoting mutation row stays valid.
- **Same-trade commands are not refusals.** NFR11 is about refused commands; this one is satisfied.
- **An untracked draft mutation table blocks every commit** (12.4). Draft rows outside `mutations/`
  until they are ready.

### Project Structure Notes

- UPDATE `crates/protocol/src/lib.rs`, `crates/sim-core/src/lib.rs`, `crates/sim-core/tests/{scenario.rs,
  save_load.rs}`
- UPDATE `crates/simd/src/{main.rs, bridge.rs}`, `crates/simd/tests/serve.rs`
- UPDATE `crates/client-core/src/lib.rs`
- UPDATE `crates/gui/src/{ingest.rs, command.rs, appearance.rs}`, `crates/gui/tests/{headless.rs,
  pixel_guard.rs}`
- UPDATE `crates/tui/tests/client.rs` (test only)
- UPDATE `ARCHITECTURE-SPINE.md` (AD-10), `README.md`
- NEW `_bmad-output/implementation-artifacts/mutations/12-6.sh`, `12-6-signoff/vehicle-card.md`
- EXISTING (creation) `12-6-signoff/trade_wire.py`, `12-6-signoff/draft.md`

### Previous story intelligence

- 12.5's protocol pattern carries over: an exhaustive bridge `match` with no wildcard, hand-literal pins
  in `protocol`.
- `cargo test -p gui` does not rebuild `simd`, so a daemon-side RED for a gui real-binary test is a
  scratch-worktree run, not a `mutate.sh` row. Kill test daemons by PID, never `pkill -x simd`.
- Branch pushes use `push.sh --fast`. The FULL gate runs only before the PR
  ([[push-branch-for-vehicle-testing]]).

### References

- `_bmad-output/planning-artifacts/epics.md:2560-2588` (Story 12.6, incl. the 12.4 review NOTE `:2585`);
  PRD `prds/prd-frostvein-2026-09-28/prd.md:79-86` (FR42, FR43), `:132-137` (NFR10, NFR11)
- `ARCHITECTURE-SPINE.md` (2026-08-01): AD-4 (`:75`), AD-10 (`:151`), AD-12 (`:194`), conventions row
  "Command acknowledgement" (`:222`)
- Story 12.1 (refusal shape), 12.2 (selection, name HUD), 12.4 (professions, tui roster), 12.5 (wire
  pattern, real-binary stderr instrument)

## Verification

**Wire (runs today; RED observed at creation):**

```bash
target/release/simd 7495 &   # fresh daemon, DEFAULT_SEED; kill it by PID afterwards
python3 _bmad-output/implementation-artifacts/12-6-signoff/trade_wire.py 7495
```

- **RED** at creation, on `main` `98149e1`, release `simd`: `tick 18: dwarf 2 holds channel [65, 63, 9];
  sent both commands`, then `profession reads hauler on the wire: NEVER`, `old channel job still held
  after the command: tick 21`, `released target [65, 63, 9] reclaimed by a miner: NEVER`, `refusal for
  dwarf 999: NEVER`, `TRADES WIRE RED`, exit 1. The daemon's stderr shows `unrecognized client message`
  for both commands.
- **GREEN, required of dev:** the profession reads `hauler` within a tick or two of the send; the old job
  is `no`, no longer held; the target is reclaimed by miner 4 at a tick the dev records; the refusal for
  999 appears; `TRADES WIRE OK`, exit 0. Under option 1 or 2 the run is unchanged: dwarf 2 is one of two
  miners.
- **Deliberate RED, required of dev:** mutation row 1 (the sim ignores the command) gives `NEVER` on the
  profession line and `TRADES WIRE RED` again. Restore it before going on.

**gui (the feature does not exist yet; the obligation is inherited):**

```bash
target/release/simd 7496 &
target/release/gui 7496 --headless --subdiv 4 --select 2 --trade hauler --frames 300 \
  --capture "$SCRATCH/trade.png" 2>&1 | rg 'gui sent set_profession|gui dwarf [0-9] trade'
```

- **Required of dev:** exactly one `gui sent set_profession dwarf 2 hauler`, then `gui dwarf 2 trade
  hauler`. Zero of either is a failure, whatever the exit code. If 300 frames does not reach it, raise it
  and record the figure.
- **Deliberate RED:** the simd arm doing nothing (scratch worktree) gives the `sent` line and no `trade`
  line.

**Seat (AC11), in the seat's launch form:**

- WSL: `simd 7451`, and in a second WSL terminal `tui 7451` for the parity check.
- PowerShell: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`. With nothing selected, read the roster
  top-right under the clock.
- Press `2` and drag channel marks east of the fire, then press `3` and drag a 3×3 stockpile west of it.
  Click Bifur (red) while he digs, press `T`, and watch his trade line and the tui roster say `hauler`.
  Watch Dori (blue) take his mark.
- Per Task 0.2 (option 1): select Nain (purple, the only woodcutter) and press `T`. The bottom-left line
  says `trade refused: last of his trade`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-05 | Story created on `98149e1`. RED on the live wire (`trade_wire.py`: `set_profession` unrecognized, profession never changes, no refusal). Look draft `12-6-signoff/draft.md` written. Task 0 (draft, empty-trade rule, refusal text) awaits Wolf. |
