---
baseline_commit: a96ab48
model: claude-opus-5-5  # session default, same as 11.3's creation
---

# Story 12.1: Stones Stay Out of the Fire

Status: review

## Story

As the boss,
I want hauled stones to land in the stockpile and never in the campfire, and to be told when a
stockpile I place is refused,
so that my camp looks right and I never wonder whether an order was ignored.

## Not stacked — branch off `main`

`main` is `a96ab48` (PR #151 merged), clean. Branch `story-12-1-stones-stay-out-of-the-fire`
already exists and carries the sprint-planning board edit (Epic 12 added). Closes **#134**. Epic 12's
nine standing ACs (`epics.md`, "Standing acceptance criteria") bind this story and are not restated.

## Found at creation — #134's case, reproduced (AC1's record, to be confirmed at the seat)

A throwaway scenario probe on `DEFAULT_SEED` (camp `(64,64,9)`) settled which case it is. It was
run and then deleted. **No stone ever reaches the emitter's own cell.** Movement already refuses
it (`is_walkable`, `lib.rs:530`), and a delivery is a drop where the carrier stands. The real
defect is one level up:

- `PlaceStockpile` accepts the fire's cell (`lib.rs:1480-1488`, `is_standable` only), and the haul
  `free` set counts it as a free tile (`lib.rs:697-702`). It is **free forever and unreachable**.
- The pick-up leg is gated only on `!free.is_empty()` (`lib.rs:712`), so dwarves keep picking up
  a stone they can never deliver. A* fails, `retry_claim` → `release_claim` drops the stone wherever
  the carrier stands, and 20 ticks later the loop repeats.
- **Measured.** Case A is a 5×5 stockpile centred on the fire plus 30 digs. The pile was full by
  t≈2000. Pick-ups kept climbing: 109 → 216 → 330 at t=2000/3000/4000. Three stones ended **stacked
  on one zone cell diagonal to the fire** (`(+1,+1)`), and every one of the fire's 8 neighbours held
  stones. `max stones on an emitter cell = 0` throughout. Case B is a stockpile on the fire's cell
  only plus 1 dig. The same stone was picked up 98 times in 3,000 ticks and never delivered.
- The gui draws the campfire as a 0.55-cell cube at its cell centre (`appearance.rs:397`,
  `project.rs:1818-1821`), and stones at their own cell centre (`project.rs:1937`). A stone on a
  neighbour cell does **not** geometrically intersect the fire. It can only overlap it in
  projection from a low camera. **Wolf's seat answer to "is this what you saw" is recorded in
  Task 6.**

Live RED of the refusal half, taken on `a96ab48` (`simd 7481`, fresh):
`tui 7481 --frames 12 --z 9 --key p,enter,enter` → the daemon's zones became `[[64,64,9]]`, which
is the campfire cell, accepted. `tui 7481 --frames 12 --z 8 --key p,enter,enter` (solid rock) →
zones unchanged, and the status line read `tick 39  normal  z 8/31  dwarves 5  N up`, with no word
of the refusal.

## Acceptance Criteria

1. The story's Debug Log records #134's case as found above: stones never on the emitter cell;
   the emitter cell accepted as an unreachable stockpile tile drives an endless pick-up/drop loop
   and stacks stones beside the fire. Wolf's seat answer is added in Task 6.
2. A `PlaceStockpile` rect never makes a light emitter's cell a stockpile cell, and no haul's
   delivery goal ever includes an emitter cell, **including a zone loaded from an older save**.
3. A scenario test on `DEFAULT_SEED`, **red before the fix and recorded as red**, places a 5×5
   stockpile centred on the campfire, digs nearby, and asserts all three of these: no zone on an
   emitter cell; no stone on an emitter cell at any tick; zero pick-ups once the reachable
   stockpile is full.
4. A `PlaceStockpile` rect that yields zero valid cells is refused by the sim. That covers all rock,
   all emitter, or entirely off the map. The refusal reaches every attached client in the **next
   delta** as a typed entry. A rect with at least one valid cell is not a refusal.
5. Refusal wire shape (AD-4/AD-6, **the pattern every later M3 filter follows**): `Delta` gains
   `refusals`, a list of `protocol::Refusal`. That type is an enum internally tagged by `command`,
   and its one variant is `PlaceStockpile { rect }`. The list is omitted from the JSON when empty,
   so every existing delta line is byte-identical. No client pre-checks the rule.
6. Both clients show `stockpile refused: no valid cells` after a refused stockpile: the tui on its
   status row, the gui as a HUD line. The text stays until that client sends its next world
   command. The text comes from one function in `client-core`.
7. The parent spine's "Command acknowledgement" convention is amended on the record. Accepted
   commands still acknowledge through their effect. Refused ones are reported in the next delta's
   `refusals`.
8. At the seat, Wolf places a stockpile around the campfire, hauls into it, and sees no stone on or
   in the fire. A single-cell stockpile on the fire shows the refusal in the gui and in an attached
   tui.

9. **(Added 2026-09-28, Wolf's ruling on #153.)** No stockpile cell ever holds more than one
   uncarried stone. When a carrier's delivery fails, it drops the stone where it stands unless that
   tile is a stockpile cell already holding a stone. In that case it drops it on the nearest
   walkable tile that is not an occupied stockpile cell. Heaps on open ground stay allowed, as
   today.

## Tasks / Subtasks

- [x] **Task 0 — RED first (AC1, AC3).** Write the scenario test (skeleton below) in
  `crates/sim-core/tests/scenario.rs` and run it on the unfixed code. Paste the failing assertion
  messages and the measured pick-up count into the Debug Log. The emitter-zone and the pick-up
  asserts must fail. The stone-on-emitter assert is expected to pass today; it is a regression
  guard, and the log says so. Record the AC1 finding in the Debug Log.
- [x] **Task 1 — sim-core: the filter and the refusal (AC2, AC4).**
  - [x] `Refusal` enum (NEW, `lib.rs` beside `SimCommand`):
        `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Refusal { PlaceStockpile { rect: Rect } }`.
  - [x] `apply_command(&mut self, command: SimCommand) -> Option<Refusal>`. It returns
        **`Option`, not `Result`**: 58 call sites outside `lib.rs` ignore the return, and
        `Result` is `#[must_use]`, so `clippy -D warnings` would fail on every one of them.
  - [x] `PlaceStockpile` keeps a position only if `is_walkable(&terrain, &blocked, pos)`, the same
        predicate movement uses. Build `blocked` from the emitter positions (`blocked_cells` on
        `self.emitters()`). Zero kept → return `Some(Refusal::PlaceStockpile { rect })`, where `rect`
        is the command's rect as received. The out-of-bounds early return (`lib.rs:1391-1399`)
        returns that same refusal for `PlaceStockpile` and `None` for the other three commands.
  - [x] `work_positions` gains `blocked: &BTreeSet<Pos>`. The haul `free` set filters with
        `is_walkable(terrain, blocked, *pos)` instead of `terrain.is_standable(*pos)`. Both callers
        (`lib.rs:401`, `:836`) already hold `blocked`. Update the five unit-test calls
        (`lib.rs:2289-2312`) and add one unit test: zones `{fire}`, blocked `{fire}`, stone
        elsewhere → both legs empty. This is the old-save case.
  - [x] Unit tests: an all-emitter rect → `Some(refusal)` and zones unchanged; an all-rock rect →
        `Some`; an off-map rect → `Some`; a 3×3 around the fire → `None` with 8 zones added;
        `Designate`/`Cancel`/`Remove` → `None`.
- [x] **Task 2 — protocol + simd: the wire (AC4, AC5).**
  - [x] `protocol` (NEW type):
        `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] #[serde(tag = "command", rename_all = "snake_case")] pub enum Refusal { PlaceStockpile { rect: Rect } }`.
        On `Delta`, add a last field:
        `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub refusals: Vec<Refusal>`.
        Pin the wire with a literal test:
        `"refusals":[{"command":"place_stockpile","rect":{"min":[64,64,8],"max":[64,64,8]}}]`
        round-trips, and `DELTA_WIRE` (no refusals) still round-trips byte-identical.
  - [x] `bridge.rs`: add `rect_out` (the mirror of `rect_in`, `:182`) and
        `pub(crate) fn refusal_out(sim_core::Refusal) -> protocol::Refusal`, an exhaustive `match`
        with no wildcard. `delta(world, speed, refusals: Vec<protocol::Refusal>)`.
  - [x] `simd/src/main.rs` `tick`: `let mut refusals = Vec::new();` before the command loop
        (`:169`). The `PlaceStockpile` arm (`:234`) does
        `refusals.extend(world.apply_command(..).map(bridge::refusal_out))`. The other arms discard
        the `None`. Pass `refusals` to `bridge::delta` (`:292`).
  - [x] Extend `serve.rs::the_daemon_keeps_channels_and_stockpiles_only_at_standable_cells`
        (`:417-438`). The rejected (solid) stockpile must appear in a delta's `refusals` with its
        rect, and the accepted one must not. The real daemon is the judge.
  - [x] Every `protocol::Delta { .. }` struct literal gains `refusals: Vec::new()`. There are about
        35 of them across `gui/tests/{capture,headless}.rs`, `gui/src/ingest.rs`,
        `tui/tests/client.rs`, `client-core/src/lib.rs`, `simd/tests/serve.rs` and `bridge.rs`.
        Let the compiler list them.
- [x] **Task 3 — both clients show it (AC6).**
  - [x] `client-core`: `pub fn refusal_text(refusal: &protocol::Refusal) -> &'static str`, an
        exhaustive `match` returning `"stockpile refused: no valid cells"`. It is the only
        definition of the text.
  - [x] tui: `ViewState` gets `pub refusal: Option<protocol::Refusal>`. Both `Msg::Delta` arms
        (`main.rs:312-329` interactive, `:450-462` `stream_frames`) set it from
        `delta.refusals.last()` when non-empty and never clear it on an empty delta. `apply_key`
        clears it when it returns a world command (the dig, channel, stockpile or clear commit).
        `render` (`view.rs:377-410`) appends two spaces plus `refusal_text(..)` to the status row
        when `Some`; the quit prompt still wins. Update the `ViewState` literal in
        `initial_view_opens_on_the_level_with_the_most_standable_ground` (`view.rs:1884`).
  - [x] gui: a `LastRefusal(Option<protocol::Refusal>)` resource. The `WireMessage::Delta` arm of
        `ingest_messages` (`ingest.rs:2492-2540`) sets it the same way. A HUD `Text` tagged `Hud`
        and `ClientLocal`, one line above the designate hint (`designate.rs:65`), shows
        `refusal_text` or nothing. `designate.rs`'s release handler sets `LastRefusal(None)` when
        it pushes commands.
  - [x] gui, stockpile only: when a stockpile drag's `surface` is empty, push
        `PlaceStockpile { rect: picked_rect }` so the sim judges and refuses it
        (`commands_for`, `designate.rs:221-240`). Today it sends nothing, which is the silent no-op
        NFR11 forbids. Channel is unchanged.
- [x] **Task 4 — the instrument, tested (AC6).** The instrument is `tui --frames N --key ...`
  against the real daemon (recipe below); the story extends its status row.
  - [x] Test the instrument through the real binary in `tui/tests/client.rs`. A stub daemon sends
        one delta carrying a refusal, then plain deltas. Assert that the streamed status rows
        contain the text from that frame on. A second stub run with no refusal must show no
        `refused` in any row.
  - [x] gui: an in-crate test in `ingest.rs` modelled on
        `the_live_clock_readout_follows_the_daemons_tick_and_speed` (`:3298`). Inject a `Delta`
        with a refusal and the HUD text equals `refusal_text`. Inject a later plain delta and it is
        unchanged. Push a command and it is empty.
- [x] **Task 5 — the record (AC7, AC1).**
  - [x] Amend the "Command acknowledgement" row in
        `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md`
        with an `Amended 2026-MM-DD (Story 12.1)` note. It covers: refusals ride the next delta's
        `refusals` and go to every client; the sim decides; every later filter adds a `Refusal`
        variant.
  - [x] `deferred-work.md:406` ("stockpile on solid rock is a silent no-op"): mark it
        **CLOSED in Story 12.1**. Its dig twin (a dig rect hitting nothing diggable) stays open;
        say so on the entry.
  - [x] Mutation set `_bmad-output/implementation-artifacts/mutations/12-1.sh`, every row KILLED:
        (1) drop the `is_walkable` filter in `PlaceStockpile` → the scenario test fails;
        (2) revert the `free` filter to `is_standable` → the old-save unit test fails;
        (3) simd discards the refusal → the `serve.rs` test fails;
        (4) the tui never renders `refusal` → the client test fails;
        (5) the gui never sets `LastRefusal` → the ingest test fails.
- [ ] **Task 6 — seat (AC8), then the full gate.** Write `12-1-signoff/vehicle-card.md` in the seat's
  launch form. In WSL: `simd 7451`. In PowerShell from the Windows checkout:
  `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`. Attach a tui in WSL with `tui 7451`.
  The card has three steps: (a) `3`, then drag a 5×5 stockpile centred on the campfire;
  (b) `1`, then dig a few blocks nearby, `+` to speed up, and watch the ring fill with nothing in
  the fire; (c) `3`, then drag one cell on the fire, and the refusal shows in both clients. Ask
  Wolf "is this the case you saw?" and record the answer under AC1. Then run the full
  `scripts/gate.sh` (see [[gate-ooms-at-default-parallelism]]: `RUST_TEST_THREADS=2`).

- [x] **Task 7 — #153: one stone per stockpile cell when the pile fills (AC9). Added 2026-09-28 at
  the seat, by Wolf's ruling.** Found by running after Task 1's fix: 26 stones on 24 cells, 3 on one
  corner. The orchestrator's probe (worktree, since deleted) found the mechanism. With 24/24 cells
  full, two carriers still walking to the last free cell `(65,66,9)` arrived and dropped on it, at
  ticks 1808 and 1883. The comment at the haul `free` set ("self-healing ... it repaths") is false
  once `free` is empty.
  - [x] RED first: a scenario test on `DEFAULT_SEED` that places a 5×5 stockpile centred on the
        camp and designates a 15×15 dig centred on the camp (one `Designate` rect,
        `camp ± 7` at `camp.z`). Tick 8,000 and assert at EVERY tick that no stockpile cell holds
        more than one UNCARRIED stone. A carried stone reports its carrier's position, so exclude
        stones in `world.carrying()`. Add a positive assertion too: by the end, every zone cell
        holds exactly one stone. Record the RED message (the tick and cell) in the Debug Log.
  - [x] Fix in `sim-core` at the drop site: `release_claim`, or whatever the RED trace shows
        actually drops the stone. Drop at the carrier's tile unless it is an occupied stockpile cell;
        otherwise at the nearest `is_walkable` tile that is not an occupied stockpile cell.
        Choose "nearest" deterministically (a BFS with a fixed neighbour order, or ties broken by
        `Pos` order). Correct the false "self-healing" comment.
  - [x] Unit test: a carrier standing on an occupied stockpile cell releases its claim → the stone
        lands on a non-zone neighbour, and the stockpile cell still holds one stone.
  - [x] Add both tests to `mutations/12-1.sh` (revert the drop rule → both reddens), run them, and
        record them KILLED.
  - [x] Determinism: an existing scenario/determinism test must still pass. The drop site changes
        only in the occupied-stockpile case.

- [x] **Task 8 — the tui shows a stored stone as stored (Wolf's ruling 2026-09-28, at the seat).**
  Wolf read a full pile in the tui as "the fire is inside the stockpile". A stone draws `*` over
  the zone's `≡` in the same grey as a loose stone, so a full pile leaves no trace of the stockpile.
  Implemented by Claude directly, because Codex hit its 5-hour limit (Wolf's choice).
  - [x] `palette::stored_item_cell()`: `*` in the stockpile green, `zone_cell().fg`. `render` draws
        it for a stone on a zone cell; a loose stone stays `item_cell()`.
  - [x] View test, RED first: a stone on a zone cell renders `stored_item_cell()`, a stone off the
        zone renders `item_cell()`. Add a mutation row to `mutations/12-1.sh` and record it KILLED.

### Scenario test skeleton (Task 0)

```rust
#[test]
fn a_stockpile_around_the_campfire_never_zones_or_receives_the_fire() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    let camp = world.camp_origin(); // (64,64,9) on DEFAULT_SEED
    let emitters: BTreeSet<Pos> = world.emitters().into_iter().map(|(_, p, _)| p).collect();
    world.apply_command(SimCommand::PlaceStockpile { rect: rect(
        Pos { x: camp.x - 2, y: camp.y - 2, ..camp }, Pos { x: camp.x + 2, y: camp.y + 2, ..camp }) });
    assert!(world.zones().iter().all(|z| !emitters.contains(z)), "zone on an emitter: {:?}", world.zones());
    // Designate ~30 single-cell digs: non-tree Solid cells at camp.z with a standable 4-neighbour,
    // nearest the camp first (the probe found 30 within r<=7 on DEFAULT_SEED).
    // Tick 4,000, counting per-dwarf carrying None->Some transitions:
    //   - every tick: no item on an emitter cell;
    //   - in a window after the pile is full (measure when; pre-fix it was full by ~2,000): 0 pick-ups.
}
```

## Dev Notes

### Scope guardrails (do NOT)

- Do not address a refusal to only the issuing connection. The delta is broadcast, and the refusal
  rides it. `// NOTE:` on `Delta::refusals`: every attached client sees every refusal.
- Do not add a refusal for `Designate`, `CancelDesignation` or `RemoveStockpile`. A dig on nothing
  stays a silent no-op (deferred-work, now its own open entry). The enum is the extension point.
- Do not add a `reason` field or a second variant. One variant means one reason ("no valid cells").
- Do not teach the gui drag preview (`client_core::surface_targets`) about emitters. The preview
  over the fire shows one cell the sim will not keep. `// NOTE:` it; that is a client restating a
  sim rule, which is AD-4's line.
- Do not fix the general "only free tile is unreachable" loop, where a standable zone cell is
  sealed off. It is the same pick-up/drop mechanism without a fire. It belongs with #132 (12.3,
  FR51). Add it there as a comment, or ask Wolf.
- Do not add a new server message type. Both clients `bail!` on an unknown `type` (tui
  `main.rs:517`, gui `ingest.rs:2665`), and every extra tui message costs a `--frames` frame.

### What already exists (build on it)

- `is_walkable` + `blocked_cells` (`lib.rs:530-545`): the one "can a dwarf stand here" rule (#74).
  The stockpile and the free set now ask it too.
- `spawn_dwarves` already excludes emitter cells (`lib.rs:1594-1606`), using the same idea at spawn.
- `serve.rs:318` already sends a rejected stockpile before an accepted one and asserts the drop.
  Extend it; do not write a second daemon test.
- Delta assembly happens once per loop iteration and after paused command intake (AD-2), so a
  refusal is in the very next delta (NFR2 ~200 ms).
- `tui --frames N --z Z --key ...` streams real frames; its cursor opens at the map centre
  `(64,64)`, which is the campfire on `DEFAULT_SEED`.

### Key decisions & traps

- **`Option<Refusal>`, not `Result`**: see Task 1. This is a compile-clean choice, not a style
  choice.
- **`skip_serializing_if` keeps the wire byte-identical when nothing is refused.** Pinned wire
  literals and every recorded capture stay valid.
- **Adding a `Delta` field breaks every struct literal** (~35). This is mechanical. Do not reach for
  `..Default::default()`, because `Delta` has no `Default`, and adding one is not this story's job.
- **A partial stockpile is not a refusal.** A 5×5 over the fire keeps 24 cells and reports nothing.
- **Refusal text lives only in `client-core::refusal_text`**, like the colour table is data. A
  second draw site would drift.
- **The gui can only produce an all-rock stockpile through the new empty-surface path.**
  `surface_targets` follows the ground, so its common refusal is the all-emitter drag.

### Verification (recipe; the RED half was run at creation, see "Found at creation")

```bash
cargo build -q -p simd -p tui
./target/debug/simd 7481 &                       # fresh daemon, DEFAULT_SEED
# A. all-emitter: the cursor opens on the campfire at z 9
./target/debug/tui 7481 --frames 12 --z 9 --key p,enter,enter | sed 's/\x1b\[[0-9;]*m//g' | rg 'tick '
#   GREEN: the last status rows contain "stockpile refused: no valid cells"; daemon zones stay []
# B. all-rock: z 8 under the camp is solid
./target/debug/tui 7481 --frames 12 --z 8 --key p,enter,enter | sed 's/\x1b\[[0-9;]*m//g' | rg 'tick '
#   GREEN: same text. RED (a96ab48, observed): A left zone [[64,64,9]]; B's rows had no "refused".
# Zones check: read one snapshot line from the port; `zones` must not contain [64,64,9].
pkill -x simd
```

Deliberate RED after the fix: mutation row (3) (simd discards the refusal) → recipe B's rows show
no `refused`. Restore it, and B shows it again. Exit 0 is not a result; the text in the row is.

### Project Structure Notes

- `crates/sim-core/src/lib.rs`: UPDATE (`Refusal`, `apply_command`, `work_positions`, unit tests)
- `crates/sim-core/tests/scenario.rs`: UPDATE (the red scenario test)
- `crates/protocol/src/lib.rs`: UPDATE (`Refusal`, `Delta::refusals`, wire test)
- `crates/simd/src/{bridge.rs,main.rs}`, `crates/simd/tests/serve.rs`: UPDATE
- `crates/client-core/src/lib.rs`: UPDATE (`refusal_text`, Delta literals)
- `crates/tui/src/{main.rs,view.rs}`, `crates/tui/tests/client.rs`: UPDATE
- `crates/gui/src/{ingest.rs,designate.rs}`, `crates/gui/tests/{headless,capture}.rs`: UPDATE
- `_bmad-output/implementation-artifacts/mutations/12-1.sh`, `12-1-signoff/vehicle-card.md`: NEW
- Parent `ARCHITECTURE-SPINE.md`, `deferred-work.md`: UPDATE (the record)

### References

- `epics.md` Epic 12 intro (order, standing ACs) and Story 12.1; PRD `prd-frostvein-2026-09-28`
  FR50, NFR11
- Parent spine `architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md`: AD-2, AD-4, AD-6, AD-10,
  and Consistency Conventions ("Command acknowledgement", "Vocabulary enums")
- Issue #134; `deferred-work.md:406`
- `docs/technical-preferences.md` (M2-26 scenario tests for sim, M2-27 reproduce first)

## Dev Agent Record

### Agent Model Used

gpt-6-sol

### Debug Log References

- Task 8 (Claude, directly; Codex was at its 5-hour limit): RED on ``d13df15``, where `view::tests::a_stone_on_a_stockpile_cell_draws_in_the_stockpile_colour` failed at `view.rs:1169` with `left: Cell { glyph: '*', fg: (176, 172, 160) }` and `right: Cell { glyph: '*', fg: (88, 190, 118) }`. GREEN at `2fcfa21`: tui 47 + 17 passed. Mutation `tui draws stored stones grey` KILLED. The full `12-1.sh` table, rerun by the orchestrator: 8/8 KILLED.
- Task 8 rotted two older rows' anchors, and the pre-commit hook caught it (`mutation tables still apply FAILED`, commit refused): 3.2's `items draw above entities` and 3.3's `stone counting and stone drawing use different filters`. I re-pointed both at the new draw expression, `audit-mutations.py` reported 697/697 applicable, and both re-ran ALONE and were KILLED.
- Task 7, verified by the orchestrator: Codex hit its 5-hour usage limit after its two code commits (`47561ec`, `3d6d2ad`), with its record still uncommitted. The orchestrator committed that record as `d13df15` and reran `12-1.sh` independently: 7/7 KILLED.
- Task 7 RED on current code (`cargo test --offline -p sim-core --test scenario a_full_stockpile_never_stacks_uncarried_stones -- --nocapture`): `tick 1809: Pos { x: 65, y: 66, z: 9 } holds 2 uncarried stones`; `test result: FAILED. 0 passed; 1 failed`. The test excludes IDs in `world.carrying()` and asserts all 24 zone cells hold exactly one uncarried stone at tick 8,000.
- Task 7 drop trace (temporary manual `eprintln!`, removed before commit): `TRACE retry tick=1809 pos=Some(Pos { x: 65, y: 66, z: 9 }) carrying=Some(Carrying(Some(34)))`. No delivery trace fired in that window. The stale goal reaches `retry_claim` → `release_claim`, which wrote stone 34 onto the occupied zone cell.
- Task 7 mutation verification (`scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-1.sh`, run alone after fix commits): all seven rows KILLED. New rows: `retry drop stacks a full stockpile` → `a_full_stockpile_never_stacks_uncarried_stones` FAILED at `scenario.rs:167`; `release drop stacks an occupied stockpile` → `release_claim_avoids_an_occupied_stockpile_cell` FAILED at `lib.rs:2702` (`left != right`). Existing five rows also KILLED: `stockpile keeps emitter zones`, `old save haul goals include emitters`, `daemon discards stockpile refusal`, `tui omits refusal status`, `gui drops last refusal`.
- Task 7 restored-source verification: `cargo test --offline -p sim-core` → 118 passed, 0 failed, 1 ignored (60 unit, 10 save/load, 32 scenario, 16 worldgen); `RUST_TEST_THREADS=2 cargo test --offline -p simd --test serve` → 66 passed, 0 failed. The existing `same_seed_and_commands_remain_deterministic` and race scenarios passed. The pre-commit fast gate passed on both code commits; full `scripts/gate.sh` was not run by request.
- Task 7 required an update to the existing race scenario: it now asserts the second carrier drops off the occupied stockpile cell, then hauls to a newly opened cell. Two Story 3.3 mutation anchors were re-pointed after the drop code changed; `scripts/audit-mutations.py` reported all 694 rows applicable before the new Task 7 rows were added.
- Orchestrator verification (Claude, 2026-09-28): FULL `scripts/gate.sh` GREEN on `a812bbe` at `RUST_TEST_THREADS=2`, 1882 s (cargo test 138 s, pixel guards 1710 s, mutation tables still apply). 10 commits all author Völundr / committer jeicei75, no trailers, no `--no-verify`. Codex dev $9.05 (326 turns, gpt-6-sol/high, 13pp quota).
- Additional post-commit boundary sabotage (`scripts/mutate.sh /tmp/12-1-boundary-red.sh`): `stockpile_refuses_only_when_every_cell_is_invalid` failed at `crates/sim-core/src/lib.rs:2365:9` with `assertion left == right failed` (KILLED); `empty_stockpile_surface_still_reaches_the_sim` failed at `crates/gui/src/designate.rs:315:9` with `assertion left == right failed` (KILLED). Both restored targeted tests passed.
- Protocol wire compatibility: the test pins the exact compact no-refusal delta JSON line. Separate post-commit sabotage (`scripts/mutate.sh /tmp/12-1-wire-red.sh`) removed `skip_serializing_if`; `refusal_wire_is_literal_and_empty_delta_wire_is_unchanged` failed at `crates/protocol/src/lib.rs:259:9` with `assertion left == right failed` (0 passed, 1 failed), KILLED. Restored targeted test passed.
- Post-strengthening verification: the five-row mutation table was rerun after committing the second-client daemon assertion; all five rows were KILLED again. `RUST_TEST_THREADS=2 cargo test --offline -p simd --test serve --quiet` was rerun on restored source: 66 passed, 0 failed.
- Task 2 broadcast check: the daemon test attached a second client before the refused command and asserted its delta carried the same refusal; targeted test passed.
- Task 5 mutation verification (`scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-1.sh`, run alone after commit):
  - `stockpile keeps emitter zones` — `a_stockpile_around_the_campfire_never_zones_or_receives_the_fire` — KILLED: `panicked at crates/sim-core/tests/scenario.rs:113:5`; test result FAILED, 0 passed, 1 failed.
  - `old save haul goals include emitters` — `an_old_save_zone_on_an_emitter_is_no_haul_goal` — KILLED: `assertion failed: super::work_positions(&terrain, &blocked, &zones, &items, job, ...)`; test result FAILED, 0 passed, 1 failed.
  - `daemon discards stockpile refusal` — `the_daemon_keeps_channels_and_stockpiles_only_at_standable_cells` — KILLED: `assertion left == right failed: the rejected stockpile must be broadcast in the next delta`; test result FAILED, 0 passed, 1 failed.
  - `tui omits refusal status` — `streamed_refusal_stays_on_the_status_row_across_plain_deltas` — KILLED: `panicked at crates/tui/tests/client.rs:276:9`; test result FAILED, 0 passed, 1 failed.
  - `gui drops last refusal` — `refusal_hud_follows_wire_and_clears_on_the_next_world_command` — KILLED: `assertion left == right failed` at `crates/gui/src/ingest.rs:3382:9`; test result FAILED, 0 passed, 1 failed.
- Task 6 daemon tests: `RUST_TEST_THREADS=2 cargo test --offline -p simd --test serve --quiet` → 66 passed, 0 failed.
- Task 6 manual live verification on fresh `simd` port 43767, `tui --frames 12 --key p,enter,enter`:
  - A emitter, `--z 9`: 12 status rows, 7 with refusal; last row `tick 14  normal  z 9/31  dwarves 5  N up  stockpile refused: no valid cells`; snapshot zones `[]`.
  - B rock, `--z 8`: 12 status rows, 7 with refusal; last row `tick 26  normal  z 8/31  dwarves 5  N up  stockpile refused: no valid cells`; final snapshot zones `[]`.
- The pre-commit fast gate was green on each successful commit. The full `scripts/gate.sh` without arguments was not run by request; seat verification remains pending.
- Task 4 GREEN: `streamed_refusal_stays_on_the_status_row_across_plain_deltas` and `refusal_hud_follows_wire_and_clears_on_the_next_world_command` passed. The TUI stub sent one refused delta then plain deltas; the GUI test drove a world command after the plain delta.
- Task 0 RED on unfixed code: `zone on an emitter: [Pos { x: 64, y: 64, z: 9 }]; pick-ups after t=2000: 221 (expected 0)`. The stone-on-emitter assertion passed before this failure; maximum observed on an emitter was 0. This confirms #134's unreachable emitter-zone pickup/drop loop and beside-fire pile, not a stone entering the fire cell.
- Task 0/1 GREEN: scenario, old-save haul unit test, and all-invalid/partial stockpile unit test passed with `cargo test --offline -p sim-core` targeted invocations.

### Completion Notes List

- Task 8: a stone on a stockpile cell draws `*` in the stockpile green (`stored_item_cell`); a loose stone stays grey. This is a tui-only change, and the gui is unchanged.
- Task 7: occupied stockpile drops relocate to the nearest walkable nonoccupied cell in deterministic `Pos` order; open-ground heaps remain possible. The 8,000-tick scenario fills all 24 zone cells without stacking. Task 6's Wolf seat check remains pending.
- Task 5: amended the acknowledgement convention and closed the stockpile feedback item. All five mutation rows KILLED. Task 6 card and real-daemon verification are done; Wolf seat check remains pending.
- Task 4: test both visible status instruments through the streaming TUI binary and GUI ingest/HUD systems.
- Task 3: clients retain the shared refusal text until their next world command; GUI empty-surface stockpile drags reach the sim.
- Task 2: broadcast typed refusals in the next delta; empty lists preserve the old JSON shape. Protocol literal and real daemon acceptance/refusal tests pass.
- Task 0 and Task 1: exclude emitters from placed zones and haul goals; return typed refusal for empty stockpiles.

### File List

- `crates/tui/src/palette.rs`
- `_bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh`
- `_bmad-output/implementation-artifacts/12-1-signoff/vehicle-card.md`
- `_bmad-output/implementation-artifacts/12-1-stones-stay-out-of-the-fire.md`
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `_bmad-output/implementation-artifacts/metrics/.session-cursors.json`
- `_bmad-output/implementation-artifacts/metrics/12-1-stones-stay-out-of-the-fire.md`
- `_bmad-output/implementation-artifacts/mutations/12-1.sh`
- `_bmad-output/implementation-artifacts/mutations/2-1-the-world-runs-on-its-own-clock.sh`
- `_bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh`
- `_bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh`
- `_bmad-output/implementation-artifacts/mutations/3-1-give-the-order.sh`
- `_bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh`
- `_bmad-output/implementation-artifacts/mutations/8-3-master-of-time-and-the-skeleton-walks-in-3d.sh`
- `_bmad-output/implementation-artifacts/mutations/m2-1-live-app-systems.sh`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md`
- `crates/client-core/src/lib.rs`
- `crates/gui/src/designate.rs`
- `crates/gui/src/ingest.rs`
- `crates/gui/tests/capture.rs`
- `crates/gui/tests/headless.rs`
- `crates/protocol/src/lib.rs`
- `crates/sim-core/src/lib.rs`
- `crates/sim-core/tests/scenario.rs`
- `crates/simd/src/bridge.rs`
- `crates/simd/src/main.rs`
- `crates/simd/tests/serve.rs`
- `crates/tui/src/main.rs`
- `crates/tui/src/view.rs`
- `crates/tui/tests/client.rs`

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-28 | Task 8: stored stones draw in the stockpile green in the tui (Wolf's seat ruling), implemented by Claude directly. Two older mutation anchors re-pointed and re-killed. |
| 2026-09-28 | Task 7: prevent retry drops from stacking on occupied stockpile cells; pin the race, kill seven mutations, and pass sim-core and daemon suites. |
| 2026-09-28 | Task 5: amend acknowledgement record, close deferred stockpile item, and kill all five mutations. Task 6 live recipe green; seat pending. |
| 2026-09-28 | Task 4: pin refusal persistence and clearing with client instrument tests. |
| 2026-09-28 | Task 3: show persistent refusals in TUI and GUI; empty-surface GUI stockpile sends a command. |
| 2026-09-28 | Task 2: add typed broadcast refusals to delta; protocol and daemon tests GREEN. |
| 2026-09-28 | Tasks 0–1: scenario RED recorded; emitter filtering and sim refusal GREEN. |
| 2026-09-28 | Story created on `a96ab48`. #134 reproduced by a sim probe: never on the emitter cell; the emitter zone cell drives an endless pick-up/drop loop and stacks stones beside the fire. Refusal RED observed live in the tui. Refusal shape: typed `refusals` on the next delta. |
