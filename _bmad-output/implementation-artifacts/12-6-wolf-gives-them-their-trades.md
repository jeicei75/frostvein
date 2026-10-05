---
baseline_commit: 98149e1
model: claude-opus-5-5  # session default, same as 12.1-12.5's creation
---

# Story 12.6: Wolf Gives Them Their Trades

Status: review

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
- NEW `protocol::Refusal::SetProfession { dwarf: u32 }`. With the existing `#[serde(tag = "command")]` it
  reads `{"command":"set_profession","dwarf":999}`. Its only cause is an unknown dwarf id, so it carries no
  reason field (Task 0.2: no last-of-trade refusal). A second cause adds one.
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
3. The sim refuses a `set_profession` for an unknown dwarf id, and the refusal reaches every attached
   client on the next delta (12.1's shape). Any trade change for an existing dwarf is accepted, including
   one that leaves a trade with nobody in it (Task 0.2).
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
    the other miner.

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings, 2026-10-05, at creation.**
  1. **Look draft APPROVED as drafted** (`12-6-signoff/draft.md`): the roster in the name slot, the
     selected dwarf's name and trade line with `T: change trade`, and `T` cycling miner → hauler →
     woodcutter. Its §4 last-of-trade line is void under ruling 2.
  2. **An empty trade is ALLOWED.** No last-of-trade refusal: Wolf may make all five miners. The roster is
     the only signal (no name says "hauler"), and that trade's jobs wait silently. Recommended was
     "refuse the last of any trade"; he chose "allow anything" over it and over "protect miner and hauler
     only". Task 2 adds a `// NOTE:` at `claim_jobs`.
  3. **The refusal text is fixed:** `"trade refused: no such dwarf"`, a `&'static str` like 12.1's.
- [x] **Task 1: protocol (wire diff).**
  - [x] `protocol/src/lib.rs`: `Refusal::SetProfession { dwarf: u32 }` (`:148`);
    `Command::SetProfession` (`:154`). `Command` stays `Clone` (not `Copy` since 12.1); `Refusal` stays
    `Copy`.
  - [x] Pins, written as hand literals like `refusal_wire_is_literal_and_empty_delta_wire_is_unchanged`
    (`:309`): the command literal and each refusal literal parse and re-serialise byte-identical. The
    existing pins are untouched.
- [x] **Task 2: sim-core (AC2-AC4). RED first: write the scenario tests against a `set_profession` that
  does nothing, and record the failures.**
  - [x] `SimCommand::SetProfession { dwarf: Id, profession: Profession }` (`lib.rs:91`). `sim_core::Refusal`
    (`:99`) gains `SetProfession { dwarf: Id }` (AD-6: sim-core is the source of truth).
  - [x] `apply_command` (`:1614`): dispatch `SetProfession` **before** the rect prelude (`:1616-1621`) to a
    private `fn set_profession(&mut self, dwarf: Id, profession: Profession) -> Option<Refusal>`.
    - Leave the four rect arms byte-identical: rows in `3-1-give-the-order.sh` and `3-2-the-dig.sh`
      quote them.
    - Rule order: unknown id → `Refusal::SetProfession`; same trade → `None`, nothing changes; else
      insert the new `Profession`, even if it empties his old trade (Task 0.2). If he holds a job with
      `trade(job.kind) != profession`, call `release_claim` (`:883`). Do NOT use `retry_claim`: a cooldown
      would delay the right-trade claim for no reason.
    - `// NOTE:` the released job's `WorkProgress` is lost; the next miner starts the dig from 0.
    - `// NOTE:` at `claim_jobs`' trade filter (`:508`): a trade with no dwarf leaves its jobs unclaimed
      and silent, by Wolf's ruling (12.6 Task 0.2); the roster is the player's signal.
  - [x] `sim-core/tests/scenario.rs`, new tests (DEFAULT_SEED or 42, as the neighbours do):
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
    - `an_unknown_dwarf_is_refused_and_an_emptied_trade_is_allowed`: id 999 returns
      `Refusal::SetProfession` with `professions()` unchanged; making both haulers miners is accepted, and
      with a loose stone and a pile no haul job is ever claimed over N ticks (Task 0.2's silent wait,
      pinned so a later "fix" is a visible decision).
  - [x] Determinism: `same_seed_and_commands_remain_deterministic` (`scenario.rs:1755`) gains a
    `SetProfession` in its command list and asserts `professions()` each step. `save_load.rs` gains a
    round trip after a reassignment, asserting `professions()` (pattern at `:150-180`).
- [x] **Task 3: simd + client-core (AC1, AC3, AC8).**
  - [x] `simd/src/main.rs`: a `Command::SetProfession` arm in the loop (`:224-243`) that pushes the sim's
    refusal into `refusals`, like `PlaceStockpile` (`:235-237`). `command_rects` (`:798`) returns `None`
    for it.
  - [x] `simd/src/bridge.rs`: `profession_in` (an exhaustive `match`, beside `profession_out`, `:233`) and
    the `refusal_out` (`:277`) arm for `SetProfession`. No wildcard arms.
  - [x] `simd/tests/serve.rs`: a real daemon. A test client designates the channel block and the pile, as
    `trade_wire.py` does. When a miner holds a channel job it sends `set_profession` to hauler and to dwarf
    999. On deltas, assert: his `profession` is `hauler`; he never again holds a `dig`/`channel` job;
    another miner later holds that target; and a refusal `{"command":"set_profession","dwarf":999}`
    arrives.
  - [x] `client-core/src/lib.rs` `refusal_text` (`:10`): the `SetProfession` arm, `"trade refused: no such
    dwarf"` (Task 0.3). A unit test pins every variant's text, like `every_profession_has_its_one_spelling` (`:375`).
- [x] **Task 4: gui roster, readout and the `T` key (AC5-AC8). Look per Task 0.1.**
  - [x] `ingest.rs` `update_name_readout` (`:1760`):
    - nothing selected → the roster;
    - selected → name, then trade and `T: change trade`.
    - Colours: name in `dwarf_tunic_color`, trade and key text grey `(150,160,170)`, as a named const
      beside the tunic table in `appearance.rs`. Use `TextSpan` children for the per-name colours.
    - The 12.2 row `hud never shows the name` (`mutations/12-2.sh:64`) quotes `:1774-1775`. If those lines
      change, re-point the row and re-kill it.
  - [x] A `T` system next to `save_load_keys` (`command.rs:316`): with `SelectedDwarf` (`pick.rs:67`) set
    and his mirror profession known, push `Command::SetProfession` with the next trade into
    `PendingCommands` (`command.rs:21`). Clear `LastRefusal` (`ingest.rs:2326`) as `designation_input` does
    (`designate.rs:196`).
    - Put the "next trade" order in one pure fn, which `--trade` does not need.
    - Add `KeyT` to the keymap test table (`ingest.rs:4346-4384`).
  - [x] `--trade <miner|hauler|woodcutter>` in `Args` (`:1008`) and the parser (beside `--select`,
    `:1256`): it requires `--select`, else a parse error; an unknown trade name is a parse error. Push it
    once, after the snapshot. `refuse_select_of_a_missing_dwarf` (`:2549`) already fails a bad id.
  - [x] The two stderr lines in AC9. The `trade` line compares each dwarf's profession with
    `Mirror::previous_entity` (`client-core/src/lib.rs:176`) or with a `Local` map; it prints nothing for
    the snapshot. `// NOTE:` chatty by design, as 12.5's clip line is.
  - [x] Tests (ingest.rs unit tests, `configured_app` with its real TCP pair, `:2975`; or `headless.rs`):
    - the roster lists five `Name trade` lines in id order, the names in tunic colour, and follows a delta
      that changes one trade;
    - selected: name line plus `miner   T: change trade`; Escape brings the roster back;
    - `T` pressed: the bytes on the socket are exactly one `set_profession` line with the next trade.
      Pressing `T` with nothing selected writes nothing. The readout still says the OLD trade until a
      delta changes it;
    - a `SetProfession` refusal in a delta shows in the refusal slot, and `T` clears it;
    - update `the_name_hud_shows_the_selected_dwarfs_name_in_his_colour_and_clears` (`:3504`) to the
      two-line form.
- [x] **Task 5: the instrument (AC9).**
  - [x] `gui/tests/pixel_guard.rs` (`Daemon::spawn`, `:154`): NEW
    `a_trade_set_from_the_gui_comes_back_on_the_wire`. Run `gui <port> --headless --select 2 --trade hauler
    --frames N --capture <tmp>`, and assert stderr has `gui sent set_profession dwarf 2 hauler` followed by
    `gui dwarf 2 trade hauler`. Set N from a measured run ([[frame-counts-are-venue-calibrated]]).
    Dwarf 2 is Bifur, a miner.
  - [x] Its deliberate RED is the simd `SetProfession` arm doing nothing: no `trade` line, and the test fails
    by name. It cannot be a `mutate.sh` row, because `cargo test -p gui` does not rebuild `simd`; run it in
    a scratch worktree with its own target dir, as 12.5 did. Record the output in the Debug Log.
- [x] **Task 6: tui (AC10).** No code change is expected beyond the `refusal_text` arms. Add a
  `tui/tests/client.rs` capture with a `SetProfession` refusal on a delta and assert the status row text
  (`capture_refusal_frames`, `:205`, is the pattern). The existing trade-change test (`:1766`) covers the
  roster.
- [x] **Task 7: the record.**
  - [x] `_bmad-output/implementation-artifacts/mutations/12-6.sh`. Each row is KILLED by the test named:
    1. `set_profession` changes nothing → the reassigned-miner scenario test;
    2. a reassignment keeps the held job (no `release_claim`) → the reassigned-miner scenario test;
    3. the released job is removed from `Jobs` instead → the reassigned-miner test's "same `JobId`" assert;
    4. `retry_claim` instead of `release_claim` → the reassigned-miner test's one-step claim assert;
    5. the hauler keeps carrying when reassigned → the reassigned-hauler test;
    6. the unknown id is not refused → the refusal test, and the serve test;
    7. the bridge maps `hauler` to `miner` in `profession_in` → the serve test;
    8. `T` sends the current trade, not the next → the `T` socket test;
    9. the roster drops the trade word → the roster test;
    10. the readout shows the requested trade before the wire says so → the `T` socket test's "old trade"
        assert;
    11. `--trade` pushes nothing → `a_trade_set_from_the_gui_comes_back_on_the_wire`.
  - [x] Spine `architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` AD-10 (`:155-156`): add
    `set_profession` to the list of world-mutating commands, with an "Amended 2026-10-xx (Story 12.6)"
    line. This is the first selection-driven command.
  - [x] README gui section: the roster, `T`, `--trade`, and the two stderr lines. Tui section: trade
    refusals show on the status row.
- [x] **Task 8: the live recipe, the seat (AC11), then the full gate.**
  - [x] Run the Verification recipe: GREEN, then the deliberate RED. Record both outputs.
  - [x] Write `12-6-signoff/vehicle-card.md` in the seat's launch form. Say where to look: the roster is
    top-right under the clock, and refusals are bottom-left.
  - [x] Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` (~55 min, [[gate-ooms-at-default-parallelism]]).

### Review Findings

Code review run 1, 2026-10-05, on `9d6ce0a` (diff `98149e1..HEAD`, 24 files). All four layers ran the
binaries and none timed out. Blind Hunter and Edge Case Hunter ran on Sonnet, with R1 territories; the
Acceptance and Feature Auditors ran on Opus over the whole diff. Every layer's cargo ran (1.97.1), and every
suite was green in its own target dir. Live runs:
- `trade_wire.py` gave `TRADES WIRE OK` three times; reclaim by miner 4 at tick 289 each time.
- The real gui `--select 2 --trade hauler` printed the `sent` line and then the `trade` line, three times.
- `a_trade_set_from_the_gui_comes_back_on_the_wire` passed.
- An attached tui followed the trade change and showed `trade refused: no such dwarf`.
- The hauler half of AC2 was observed on the wire: stone 11 put down at tick 165 and delivered by Frar at
  tick 379.

No code defect was found at HIGH or MED. Every finding is in a record or doc, or is a LOW test or instrument
gap. Tally: 0 decision-needed, 4 patch, 5 defer, 5 dismissed. Layer and severity are in brackets.

- [x] [Review][Patch] **AD-10's Rule line still lists four commands** (acceptance, MED for the doc reader).
  The new amendment says "only the enumeration grows", and Task 7 says to add `set_profession` to the list.
  The binding Rule line was not edited (`ARCHITECTURE-SPINE.md:155-156`); 2026-08-06's precedent did edit
  it. [`_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md:155`]
- [x] [Review][Patch] **The seat card's step 3 cannot select Bifur as written** (feature, MED for the card
  reader). Step 2 arms stockpile mode, and nothing disarms it. `select_dwarf` returns early on
  `*mode != DesignateMode::None` (`pick.rs:112`), and only Escape resets the mode (`designate.rs:162`). A
  literal click starts a 1x1 stockpile drag, and then `T` does nothing. Add "press `Esc`" before the click.
  [`_bmad-output/implementation-artifacts/12-6-signoff/vehicle-card.md:51`]
- [x] [Review][Patch] **The seat card still reads `Seat result: _pending_`** (feature + acceptance, LOW;
  same file as the patch above). The story and the board record AC11 PASSED ("works", `a5eb634`).
  [`_bmad-output/implementation-artifacts/12-6-signoff/vehicle-card.md:65`]
- [x] [Review][Patch] **The seat card implies Dori takes the released mark at once** (feature, LOW; same
  file). Measured: 272 ticks, about 27 s at Normal. The mark waits behind Dori's earlier marks in FIFO
  order. [`_bmad-output/implementation-artifacts/12-6-signoff/vehicle-card.md:55`]
- [x] [Review][Defer] **`--select <missing id> --trade` still sends `set_profession`** (edge + feature +
  acceptance, LOW, RAN; `crates/gui/src/ingest.rs:1791` against `:2677`). The run exits 1. Before it does,
  `send_commands` has written the command, and every attached client shows `trade refused: no such dwarf`.
  The world is unchanged. — deferred: LOW; it is loud (exit 1), and the world is unchanged. The GitHub issue
  was drafted, but filing it was blocked by the session's permission classifier (see the review summary).
- [x] [Review][Defer] **The roster test reaches "Escape brings the roster back" with `select(None)`, not the
  Escape key** (acceptance, LOW; `crates/gui/src/ingest.rs:3787`). The Escape→deselect path is 12.2 code that
  12.6 did not change. — deferred: test gap, LOW
- [x] [Review][Defer] **The roster's id-order assert cannot fail on order** (acceptance, LOW;
  `crates/gui/src/ingest.rs:3682`). `crew_snapshot()` lists ids already sorted, so the order holds only
  because `Mirror.entities` is a `BTreeMap`. — deferred: test gap, LOW
- [x] [Review][Defer] **The determinism test's `SetProfession` hits an idle dwarf** (acceptance, LOW, RAN;
  `same_seed_and_commands_remain_deterministic`). At seed 42, tick 60, Id(0) is an idle woodcutter, so
  `release_claim`'s determinism is covered only by the save/load round trip. AC4's letter is met.
  — deferred: test gap, LOW
- [x] [Review][Defer] **`trade_change_lines` is silent for a dwarf that first appears, or reappears, in a
  delta** (edge, LOW, READ; `crates/gui/src/ingest.rs:1808`). The case is unreachable today, because no
  dwarf spawns after connect. It becomes a silent instrument hole the day one does. — deferred: latent and
  unreachable, LOW

Dismissed (5):
- Edge's MED "`T` twice before the delta sends a duplicate". By spec, `T` cycles from the mirror (AC7, Key
  decisions), and a same-trade set is a no-op. A delta follows every command even when paused (feature,
  RAN).
- "A selected dwarf that disappears blanks the slot". Dwarves never despawn, and `--select` of a non-dwarf
  is refused by `refuse_select_of_a_missing_dwarf`.
- `update_refusal_hint` is not ordered after `trade_key`. At worst the slot clears one frame late, which is
  cosmetic.
- Blind's notes that a reassign while holding a vanished job id was not exercised, and that a woodcutter
  idles. Those are coverage notes and ruled behaviour.

**Patch pass 1, 2026-10-05.** All 4 patches landed: `e64bdbb` (AD-10) and `aa771b6` (seat card). Every
patch is a doc edit. `git diff --name-only 9d6ce0a..HEAD` lists two `.md` files and nothing else. No test
or script reads either file, so no new test or mutation row exists. **The full gate was not re-run (Wolf,
at the checkpoint).** The doc-only diff leaves every gate input unchanged since `72be96c`'s green 3413 s.
This is not a new green. The pre-commit fast gate passed on both commits.

| Item | Side written for | Side tested | Pre-existing state checked |
|---|---|---|---|
| AD-10 Rule line | a reader of the binding Rule line | the 2026-10-05 amendment ("only the enumeration grows") and the 2026-08-06 precedent, read against the edited line | the four-command Rule line at `9d6ce0a` |
| Seat card: `Esc` before the click | a seat reader following step 3 literally | the code path: `pick.rs:112` returns early off `DesignateMode::None`, and with no drag in progress one `Esc` resets the mode (`designate.rs:162-164`) | the card at `9d6ce0a`, with stockpile mode still armed after step 2 |
| Seat card: seat result | a reader of the card alone | the story and board record of AC11 ("works", `a5eb634`) | the `_pending_` line at `9d6ce0a` |
| Seat card: Dori's wait | a seat reader timing step 5 | the review's live measurement (272 ticks, about 27 s at Normal, FIFO behind her marks); not re-measured here | the "takes the mark" wording at `9d6ce0a` |

None of these rows is REWORK. This is the first patch pass.

## Dev Notes

### Scope guardrails (do NOT)

- No new tui input (NFR10). No tui display change beyond the refusal text.
- No mouse UI, buttons or menus: one key, `T`.
- No new wire field on `Entity`, `Snapshot` or `Delta`. No save format change.
- No claiming change: `claim_jobs` stays as is (AD-12), plus one `// NOTE:` (Task 0.2).
- No last-of-trade refusal and no empty-trade warning (Task 0.2).
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
  row 10 holds this.
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
  999 appears; `TRADES WIRE OK`, exit 0.
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
- Press Escape: the roster reads `Bifur hauler`, in the gui and in the tui.

## Dev Agent Record

### Agent Model Used

- Orchestrator + verifier: Claude Opus 5.5 (`claude-opus-5-5[1m]`). Wolf picked Sonnet 5.5 subagents
  for the implementation (2026-10-05), as in 12.2-12.5. They ran one after the other, on one tree.
- Agent A (Sonnet 5.5): Tasks 1, 2, 3 and 6. Agent B (Sonnet 5.5): Tasks 4 and 5 (not 5's RED).
- The orchestrator wrote the mutation table, the spine and README edits, ran every mutation and every
  deliberate RED, and did Task 8.

### Debug Log References

- **Task 2 RED** (Agent A, against a `set_profession` stub that returns `None` and changes nothing):
  - `a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job` (`scenario.rs:1813`): `the
    reassigned miner still holds the channel: [(Id(0), None), (Id(1), None), (Id(2), Some(JobId(0))),
    (Id(3), None), (Id(4), None)]`;
  - `a_reassigned_hauler_puts_the_stone_down_and_another_hauler_delivers_it` (`:1865`): `the reassigned
    hauler still carries`;
  - `an_unknown_dwarf_is_refused_and_an_emptied_trade_is_allowed` (`:1917`): `left: None right:
    Some(SetProfession { dwarf: Id(999) })`;
  - `a_world_saved_after_a_reassignment_loads_with_the_new_trade_and_steps_identically`
    (`save_load.rs:220`): `assertion failed: saved.professions().contains(&(holder,
    Profession::Hauler))`;
  - `a_trade_change_to_the_same_trade_keeps_the_job` and the `same_seed_and_commands_remain_deterministic`
    extension pass against the stub. A no-op keeps the job, and two identical no-op worlds agree. They
    guard against a wrong implementation, not a missing one; no RED is possible.
- **Task 3 RED** (Agent A, daemon arm ignoring the command, restored before commit):
  `a_reassigned_miner_lets_go_on_the_wire_and_an_unknown_dwarf_is_refused` (`serve.rs:801`): `1500
  deltas: released Some([65, 63, 9]), profession hauler false, target reclaimed false, refusal for 999
  false; need all`. `every_refusal_has_its_one_text` (client-core) was written with its arm: no RED.
- **Task 6 RED** (Agent A): a temporary text change failed `a_set_profession_refusal_shows_on_the_status_row`
  at `client.rs:298` on `contains("trade refused: no such dwarf")`.
- **Task 4 REDs** (Agent B):
  - `the_roster_lists_every_dwarf_in_id_order_and_follows_a_trade_change`: `left: [] right: [("Nain   ",
    [128, 76, 168]), ("woodcutter\n", [150, 160, 170]), …` (no spans yet);
  - `the_name_hud_shows_the_selected_dwarfs_name_in_his_colour_and_clears` (two-line form): `no selection
    shows the roster left: "" right: "Durin  miner\nBifur  hauler"`;
  - `the_next_trade_goes_miner_hauler_woodcutter_and_back` (`command.rs`): `left: Miner right: Hauler`;
  - `t_sends_one_set_profession_with_the_next_trade_and_the_readout_waits_for_the_wire`: `left: "" right:
    "{\"type\":\"set_profession\",\"dwarf\":3,\"profession\":\"hauler\"}"`;
  - `a_trade_refusal_shows_in_the_refusal_slot_and_t_clears_it`: `T is a world command and clears the
    refusal left: "trade refused: no such dwarf" right: ""`;
  - `the_trade_flag_needs_a_selection_and_a_known_trade`, `the_trade_flag_sends_one_set_profession_for_the_selected_dwarf`,
    `a_trade_line_is_reported_only_for_a_change_the_wire_made`: compile RED (`E0609 no field 'trade' on
    type 'ingest::Args'`, `E0425 cannot find function 'trade_change_lines'`).
- **Mutation table `mutations/12-6.sh`** (12 entries; the story's row 6 is 6a + 6b), run alone through
  `scripts/mutate.sh` on `95bf097` with a fresh debug `simd`, 2026-10-05: **12/12 KILLED**, each at the
  assert it targets:
  1. "set_profession changes nothing": `scenario.rs:1813` (still holds the channel);
  2. "a reassignment keeps the held job": `scenario.rs:1813`;
  3. "the released job is removed from Jobs": `scenario.rs:1819`, the same-`JobId`-in-the-queue assert;
  4. "retry_claim instead of release_claim": `scenario.rs:1823`, the one-step claim by the other miner;
  5. "a reassigned hauler keeps carrying": `scenario.rs:1865`;
  6a. "an unknown dwarf is not refused (sim)": `scenario.rs:1917`;
  6b. "an unknown dwarf is not refused (wire)": `serve.rs:801`, the closing `need all` panic. Re-run
      in a scratch worktree to read its flags: `profession hauler true, target reclaimed true, refusal
      for 999 false`. Only the refusal fails;
  7. "the bridge reads hauler as miner": `serve.rs:801`, the same panic. Its flags (scratch worktree):
      `profession hauler false, target reclaimed false, refusal for 999 true`. The command lands as
      "same trade", so he keeps the job;
  8. "T sends the current trade, not the next": `ingest.rs:3825`;
  9. "the roster drops the trade word": `ingest.rs:3719`;
  10. "the readout shows the requested trade, not the mirror's": `ingest.rs:3837`, `the readout must
      still say the OLD trade until the wire changes it`. It is the first readout assert in that test, so
      no earlier assert absorbs it;
  11. "--trade pushes nothing" (`ignored`): `pixel_guard.rs:1249`, `--trade must send exactly one
      set_profession`.

- **12.2 row re-kill:** "hud never shows the name", re-pointed by Agent B to `let name =
  client_core::dwarf_name_text(identity.name);`, run alone through `mutate.sh`: KILLED at
  `ingest.rs:3772`, the selected dwarf's name spans.
- **Task 5 deliberate RED** (orchestrator, scratch worktree with its own target dir, removed after): the
  `simd` `SetProfession` arm made to `continue` (drop the command), debug `simd` + `gui` rebuilt, then
  `a_trade_set_from_the_gui_comes_back_on_the_wire` failed by name at `pixel_guard.rs:1255`: `the
  daemon's word must come back as exactly one `gui dwarf 2 trade hauler` line, printed from the mirror`.
  Its stderr held `gui sent set_profession dwarf 2 hauler` and no `trade` line. GREEN at `--frames 100`
  (Agent B measured both lines at `--frames 1`. Each run is ~53 s because the capture waits for its
  delivered-tick floor, so the frame count does not bind).
- **Live wire recipe** (`trade_wire.py`, release `simd` on `95bf097`, fresh daemon per run, fast4x):
  - first GREEN attempt read `TRADES WIRE RED`. The cause was the INSTRUMENT. It counted a held channel
    from `sent_at + 2`, but the command lands on a later loop iteration (AD-10). At fast4x the profession
    first read `hauler` at tick 23, five ticks after the client's send at 18. The "held" sighting at tick
    21 was before the daemon had applied it. Fixed: the hold is judged from the first delta that shows
    the new trade, which is AC2's own claim (the trade change and the release are one command);
  - **GREEN:** `tick 18: dwarf 2 holds channel [65, 63, 9]; sent both commands`, `profession reads hauler
    on the wire: tick 23`, `old channel job still held after the command: no`, `released target [65, 63,
    9] reclaimed by a miner: (4, 289)`, `refusal for dwarf 999: tick 24`, `TRADES WIRE OK`, exit 0;
  - **the reclaim at tick 289 is FIFO, not a stall.** A trace of miner 4 shows him busy the whole time:
    `[66,66]` (walk 19, work 61), `[65,61]` (112-227), `[65,62]` (228-288), then `[65,63]` at 289. The
    released job waits its turn among the open marks. Claiming is unchanged (AD-12);
  - **deliberate RED, row 1** (sim ignores the command; release `simd` built in a scratch worktree):
    `profession … NEVER`, `reclaimed … NEVER`, `refusal … NEVER`, `TRADES WIRE RED`, exit 1;
  - **the corrected criterion fires**, row 2 (trade changes, job kept; scratch worktree): `profession
    reads hauler on the wire: tick 23`, `old channel job still held after the command: tick 23`,
    `reclaimed … NEVER`, `TRADES WIRE RED`, exit 1.
- **gui recipe** (release `gui` + `simd`, `--select 2 --trade hauler --frames 300`): exactly one `gui
  sent set_profession dwarf 2 hauler`, then exactly one `gui dwarf 2 trade hauler`. **gui exited 101**
  on the capture's near-white ceiling (1.6948% vs 0.9461%, boot-calibrated). It reproduces WITHOUT
  `--trade` (1.6118%, exit 101): at frame 300 the camera on Bifur sits by the fire. This predates 12.6
  and is filed as **#166** (`bug`, `route:story`). The recipe's criterion is the two lines ("whatever the
  exit code"), and they hold.

- **Full gate GREEN** on `72be96c`, `RUST_TEST_THREADS=1 scripts/gate.sh`, 3413 s (cargo test 231 s, pixel
  guards 3143 s, mutation tables still apply). **AC11 PASSED at Wolf's seat 2026-10-05** ("works"), branch at `a5eb634`.

### Completion Notes List

- `set_profession` is dispatched before the rect prelude. Because the prelude's and the closing
  `match`es are exhaustive, each gains one `SetProfession { .. } => unreachable!(…)` arm. The four rect
  arms are byte-identical, and `audit-mutations.py` (767 rows) still applies every literal.
- `rustfmt` reflowed `protocol::Refusal::PlaceStockpile { rect: Rect }` onto three lines once a variant
  with a doc comment joined it. The wire form is unchanged and no mutation row quotes it.
- Agent B re-pointed the 12.2 row "hud never shows the name" (`mutations/12-2.sh:64`) to
  `let name = client_core::dwarf_name_text(identity.name);`.
- `push_startup_trade` is registered on its own (`.add_systems(Startup, …)`), not inside the
  `client_systems` Startup tuple, because `m2-1-live-app-systems.sh` quotes that tuple.
- The `sent` line prints in `send_commands` after the socket write succeeds. The `trade` line comes from
  `report_trade_changes`, which compares a `Local` map with the mirror and records the snapshot silently.
- `T` sends nothing when the mirror holds no profession for the selected dwarf, and clears `LastRefusal`
  only when it sends.

### File List

- `crates/protocol/src/lib.rs`
- `crates/sim-core/src/lib.rs`, `crates/sim-core/tests/scenario.rs`, `crates/sim-core/tests/save_load.rs`
- `crates/simd/src/main.rs`, `crates/simd/src/bridge.rs`, `crates/simd/tests/serve.rs`
- `crates/client-core/src/lib.rs`
- `crates/gui/src/ingest.rs`, `crates/gui/src/command.rs`, `crates/gui/src/appearance.rs`,
  `crates/gui/tests/pixel_guard.rs`
- `crates/tui/tests/client.rs` (test only)
- `README.md`
- `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md`
- `_bmad-output/implementation-artifacts/mutations/12-6.sh` (new), `mutations/12-2.sh` (one row re-pointed)
- `_bmad-output/implementation-artifacts/12-6-signoff/vehicle-card.md` (new), `12-6-signoff/trade_wire.py`
  (the held-job criterion now dates from the trade change)
- `_bmad-output/implementation-artifacts/12-6-wolf-gives-them-their-trades.md`,
  `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-05 | Story created on `98149e1`. RED on the live wire (`trade_wire.py`: `set_profession` unrecognized, profession never changes, no refusal). Look draft `12-6-signoff/draft.md` written. |
| 2026-10-05 | Task 0 ruled by Wolf: draft approved as drafted; an emptied trade is allowed (no last-of-trade refusal, `TradeRefusal` dropped, refusal is `{dwarf}` only); fixed refusal text. |
| 2026-10-05 | Dev (Sonnet 5.5 agents A and B, Opus verifying): `set_profession` on the wire, in the sim (`release_claim` of an old-trade job), in simd, both clients' refusal text, gui roster, `T` and `--trade`. 12/12 mutations killed; live wire OK and the deliberate RED shown; `trade_wire.py`'s held-job check corrected; #166 filed (pre-existing `--select` capture ceiling). Full gate GREEN 3413 s on `72be96c`. Awaiting Wolf's seat (AC11). |
| 2026-10-05 | AC11 PASSED at Wolf's seat ("works", `a5eb634`). Status -> review. |
| 2026-10-05 | Code review run 1 on `9d6ce0a`: 4 layers, all ran the binaries; no code HIGH/MED. 4 doc/record patches left as action items (Wolf), 5 deferred, 5 dismissed. Status -> in-progress. |
| 2026-10-05 | Patch pass 1: 4/4 doc patches landed (`e64bdbb` AD-10 Rule line; `aa771b6` seat card: Esc, Dori's FIFO wait, seat result). Doc-only; full gate not re-run (Wolf); fast gate green on both commits. Status -> review for round 2. |
