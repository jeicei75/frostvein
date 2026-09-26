---
baseline_commit: bfe27e4
---

# Story 8.3: Master of Time, and the Skeleton Walks in 3D

Status: in-progress

## Story

As the boss,
I want to save and load from the Bevy client and watch the whole walking skeleton run there,
so that Milestone 2 is done: designate, pathfind, dig, haul — live, in the client worth looking at.

## Not stacked — branch off `main`

`main` is `bfe27e4` (11.3 merged as #129), clean. Branch slug `story-8-3-master-of-time`. Every
"X did not change" check is `git diff bfe27e4..HEAD`.

## What the epic assumed, checked against `bfe27e4`

Most of 8.3's epic text is already true. This story builds only the gap.

| Epic premise | State at `bfe27e4` | This story |
|---|---|---|
| gui pauses / resumes | **Done** — `Space`, `command.rs:285` (`toggle_pause`), refused under `--static-world` | nothing |
| gui changes tick rate | **Done** — `+`/`-`, `command.rs:312` (`step_speed`), five tiers since 11.3 | nothing |
| gui saves / loads | **Missing** — no `Command::Save`/`Command::Load` anywhere in `crates/gui` | **Task 1** |
| gui quits "as the existing control command" | **Premise wrong.** `Command::Quit` shuts the DAEMON down (`simd/src/main.rs:219`), and no client sends it; tui `q`→`y` exits the client only. A gui that sent it would kill a shared tui's daemon, breaking the epic's own two-client AC. Closing the window already exits the gui alone. | nothing; see Key decisions |
| load snaps, not animates (AD-15) | **Done** — `apply_wire_snapshot` resets clock/headings (`ingest.rs:2338`); proven by `snapshot_rewind_snaps_at_a_mid_blend_clock` (`tests/headless.rs:1396`) | exercised live in Task 4 |
| tui + gui on one daemon | **Done** at the wire — `designation_and_stockpile_changes_reach_both_clients` (`serve.rs:1454`) | seat check, Task 4 |
| end-to-end dig → haul | **Done** at the daemon — `a_designated_dig_and_a_stockpile_stream_a_stone_onto_a_zone_tile` (`serve.rs:1702`) | **Task 3**: the gui capture must range-check it |
| hint of the keys (hop zero) | hint bar exists (8.2 AC9, `designate.rs:49`) but names only the designate keys | **Task 2** |

## Acceptance Criteria

1. `Ctrl+S` in the gui puts exactly `{"type":"save"}\n` on the daemon socket, and `Ctrl+L` puts
   exactly `{"type":"load"}\n`. `S` or `L` without `Ctrl` sends nothing.
2. `Ctrl+L` under `--static-world` sends nothing and says so on stderr, like `Space` does today.
3. While `Ctrl` is held, `W`/`A`/`S`/`D`/`Q`/`E` do not move the camera, so `Ctrl+S` saves without pitching.
4. With no designate mode active, the hint bar names pause, speed, save and load as well as the
   designate keys, and it stays ASCII.
5. `gui --capture <png> --expect-haul` prints the largest number of stones seen on stockpile tiles
   during the run and exits non-zero when that number is 0.
6. At the seat, on one daemon with a gui and a tui attached, Wolf designates a dig and a stockpile in
   the gui and sees a stone reach the stockpile in both clients; a load after a save snaps both back.
7. Wolf signs off both wow beats in one sitting (boot frame on looks, the alive moment ~30 s later)
   and confirms none of the six 4.1a words is true: ugly, flat, cluttered, confusing, lifeless,
   camera unusable.
8. `crates/protocol/` and `crates/simd/` are unchanged (`git diff bfe27e4..HEAD`).
9. Every new test has a mutation row, all KILLED; `scripts/gate.sh` (full tier) is green.

## Tasks / Subtasks

- [x] **Task 1 — save and load keys (AC1-3).**
  - [x] `crates/gui/src/command.rs` UPDATE: add `pub fn save_load_keys(keys: Res<ButtonInput<KeyCode>>, static_world: Res<StaticWorld>, mut pending: ResMut<PendingCommands>)`.
        `Ctrl` (either side) + `just_pressed(KeyS)` → `pending.push(Command::Save)`; `Ctrl` +
        `just_pressed(KeyL)` → `Command::Load`, except under `--static-world`: print
        `sim NOT LOADED: --static-world holds the world frozen for the whole run` and send nothing.
        Register it beside `toggle_pause` in `ingest.rs`.
  - [x] `crates/gui/src/ingest.rs` UPDATE `camera_controls` (~`:2104`): when `Ctrl` is held, the
        key-driven `yaw`/`pitch`/`zoom` are 0. Mouse orbit, pan and wheel are untouched.
  - [x] Add `(KeyCode::KeyL, "load, with Ctrl (command.rs)")` to the keymap guard list (`ingest.rs:~3690`)
        and note `KeyS` there as "pitch; save with Ctrl".
  - [x] Tests, over a real loopback socket like `command.rs:~470` (assert the BYTES):
        Ctrl+S → save line; Ctrl+L → load line; plain `S`/`L` → nothing; Ctrl+L under
        `--static-world` → nothing; Ctrl+S held for a frame leaves the rig's pitch unchanged.
- [x] **Task 2 — the hint names the controls (AC4).** `crates/gui/src/designate.rs:50` UPDATE the
      `DesignateMode::None` string to
      `"1 dig  2 channel  3 stockpile  4 clear   Space pause  +/- speed  Ctrl+S save  Ctrl+L load"`.
      Extend `every_hint_is_ascii`'s neighbour test (or add one) to assert the four new key names are in it.
- [x] **Task 3 — the haul instrument (AC5).**
  - [x] `crates/gui/src/capture.rs` UPDATE `MotionStats`: add `pub items_on_stockpile: usize`,
        a running max like `item_count`. At the `observe` call (`capture.rs:881`) pass the count of
        `mirror.items()` whose `pos` is in `mirror.zones()` (both are cell positions, `protocol/src/lib.rs:158,163`).
  - [x] Print it in the existing `motion:` line as `items on stockpile=N` (before any assertion, as
        that line already does).
  - [x] `--expect-haul` flag in `parse_args_from` (`ingest.rs:~1142`, next to `--expect-work`): requires
        `--capture`; when set, assert `items_on_stockpile >= 1` with the message
        `capture observed no stone on a stockpile`.
  - [x] Test the instrument: `observe` with a stone on a zone tile counts 1, off it counts 0; the
        flag's assertion panics at 0 and passes at 1. Mutation rows for the count and the flag.
- [ ] **Task 4 — seat recipe and sign-off (AC6-7).** ~~Write~~ WRITTEN (`8-3-signoff/vehicle-card.md`); **the sitting is pending.** Write `8-3-signoff/vehicle-card.md` in the
      launch form (`launch-gui.ps1 -GuiArgs @(...)`, `simd 7451` in WSL). Steps: tui and gui on one
      daemon; in the gui, `1` + drag a SMALL dig (2-3 tiles) beside a SMALL stockpile (`3`); `+` to
      Fast; watch a stone land on the pile in both clients; `Ctrl+S`; dig more; `Ctrl+L` → both snap
      back; then the two wow beats and the six words. Record Wolf's answers verbatim in the story.
- [x] **Task 6 — Wolf's HUD tweaks (2026-09-26: *"1) time of the day 2) how long (sim time) sim has been
      running 3 game speed 4 hud on/off (i mean all fps and hints and modes hidden/visible)"*).** `08406e5`.
  - [x] A top-right readout `HH:MM   elapsed Nd HH:MM   speed <name>` (`ingest.rs` `clock_readout`). The hour
        is the RENDERED hour (`clock::current_hour`, so it follows `--clock`), elapsed is the daemon tick (a
        load rewinds it), and the speed is the daemon's own.
  - [x] `H` hides or shows every `Hud`-marked text (slice, lighting, clock, designate hint) plus Bevy's fps
        overlay and graph. `H` was unbound; it is added to the keymap guard.
  - [x] Tests: format at two ticks; the live readout follows a snapshot that arrives after Startup; `H` hides
        all 4 texts and the overlay, and a second `H` restores them. The wiring test pins the readout's boot text.
- [x] **Task 5 — docs.** README `### Controls`: add `ctrl + S` / `ctrl + L` rows and the Ctrl camera
      rule. README flags list: `--expect-haul`.

## Verification

```bash
# RED first (AC5): a fresh daemon has no stockpile, so the instrument must fail.
target/debug/simd 7471 &
target/debug/gui 7471 --headless --capture /tmp/haul-red.png --frames 400 --expect-haul
#   EXPECTED: motion line "... items on stockpile=0", panic "capture observed no stone on a stockpile", exit 101

# GREEN: lay work from the snapshot, run at Fast4x, then capture.
uv run scripts/task6-designate.py 7471          # dig, channel, stockpile computed from the snapshot
# send {"type":"set_speed","speed":"fast4x"} (any client, e.g. tui `+` x4), wait ~60 s
target/debug/gui 7471 --headless --capture /tmp/haul.png --frames 400 --expect-haul
#   EXPECTED: "items on stockpile=N" with N >= 1, exit 0 (or the known 101 only for an unrelated
#   capture-validator reason, which must be named)
```

**Measured at creation** (`bfe27e4`, devpod, the daemon's own snapshot read by a probe client
after `task6-designate.py`): items on stockpile tiles **0** at ticks 234 → 1,244 (Normal), then
**7** by tick 4,567 at Fast4x, of 74 stones on 4 zone tiles. So the observation can occur and the
count discriminates. `--drag` could NOT be used here: scripted input never completes headless on
the devpod (3 attempts, "scripted --drag never completed"), which is why the seat does the drags.

## Dev Notes

### Scope guardrails — do NOT
- Do NOT touch `crates/protocol/` or `crates/simd/` (AC8). No new command, no ack message.
- Do NOT send `Command::Quit` from the gui, and add no quit key: window close is the quit.
- Do NOT change the snapshot/blend path — AD-15's snap is already proven (`headless.rs:1396`).
- Do NOT add a tui instrument: the tui cross-check is Wolf's eye at the seat (AD-17 rung 1's live half).
- Do NOT tune looks. Wolf, 2026-09-26: *"need to soon move forward or we tweak this story forever"*.

### What already exists — build on it
- `PendingCommands` + `send_commands` put commands on the socket (`command.rs`); pause/speed are the pattern.
- `--expect-work` (`capture.rs:140`, `ingest.rs:1142`) is the pattern for a work-asserting flag.
- `MotionStats::item_count` (`capture.rs:406`) is already a running max of stones; add the stockpile subset beside it.
- The daemon logs `saved tick N to frostvein.save` on save; a load broadcasts a snapshot to every client.
- The keymap guard `the_client_keymap_avoids_keys_other_plugins_have_claimed` must list every bound key.

### Key decisions & traps
- **Quit is the window, not `Command::Quit`.** That command stops the daemon for every client.
- **`Ctrl` gates the keyboard camera** so `Ctrl+S` does not pitch; `Ctrl` is otherwise only the pan multiplier with the mouse (`ingest.rs:2110`).
- **Hauls queue behind digs.** 174 designations delayed the first delivery by thousands of ticks. The seat recipe uses a small dig beside a small pile, at Fast.
- **Items can stack past the tile count** (7 stones on 4 tiles was measured): count stones on zone tiles, not tiles.
- **Load under `--static-world` is refused**, for the same reason as `Space`: a capture's frozen world must not move.

### Project Structure
- `crates/gui/src/command.rs` — UPDATE (save/load system + tests)
- `crates/gui/src/ingest.rs` — UPDATE (register system, Ctrl camera gate, `--expect-haul` parse, keymap guard)
- `crates/gui/src/designate.rs` — UPDATE (hint string + test)
- `crates/gui/src/capture.rs` — UPDATE (stockpile count, print, assertion)
- `README.md` — UPDATE (controls, flag)
- `_bmad-output/implementation-artifacts/mutations/8-3-master-of-time-and-the-skeleton-walks-in-3d.sh` — NEW
- `_bmad-output/implementation-artifacts/8-3-signoff/vehicle-card.md` — NEW

### References
- `epics.md` Story 8.3; AD-2, AD-10, AD-11, AD-15, AD-17 (in `epics.md`).
- `8-2-designate-with-the-mouse.md` AC5 (bytes-at-the-socket seam test), AC9 (hint bar).
- `11-3-night-falls-day-breaks.md` Task 9c (speed tiers, gui queue 256).

### Previous-story intelligence
- 11.3: gui `+`/`-` and five speed tiers exist; the gui message queue is 256 because Fast4x evicted a 16-queue gui on lavapipe.
- 8.2: every command test asserts the bytes on a loopback socket, not that a command was built.
- Commits authored `Völundr <jeicei75@gmail.com>`; no Claude trailers (CLAUDE.md §8).

## Dev Agent Record

### Agent Model Used

claude-opus-5-5 (orchestrator implemented directly; no Codex delegation for this size).

### Debug Log References

- RED (`2d3f12d`, fresh daemon): `motion: ... item count=0 items on stockpile=0`, panic
  `capture observed no stone on a stockpile` at `capture.rs:452`, **exit 101**.
- GREEN (`task6-designate.py`, then 60 s at Fast4x, snapshot probe `items_on_stockpile=7`): `motion: ... item
  count=74 items on stockpile=7`, **exit 0**.
- Mutations: 12 of 12 KILLED (11 new, plus the re-pointed 10.10 delta-time row).

### Completion Notes List

- `Ctrl+S`/`Ctrl+L` → `{"type":"save"}` / `{"type":"load"}` on the live socket. Load is refused under
  `--static-world`. While Ctrl is held the keyboard camera scale is 0 (mouse orbit, pan and wheel are untouched).
- The hint's no-mode text names Space, +/-, Ctrl+S and Ctrl+L. **Captures hide the UI**, so whether it fits
  is a seat check (card §1).
- `--expect-haul`: `MotionStats::items_on_stockpile` is a running max of STONES on zone cells, printed on the
  `motion:` line. `expect_haul_fails_a_world_with_no_stone_on_a_stockpile` (full tier) drives the real binary through the RED.
- `crates/protocol/` and `crates/simd/` are untouched (AC8).
- The 10.10 row "delta-time scaling leaves the key rates per-frame" was re-pointed, because the Ctrl gate
  restructured its line; it was re-KILLED.

### Seat sitting (Wolf, 2026-09-26, build `c46ae49`, verbatim)

*"cool.. I think all I wanted works ..also save, load"*: the HUD tweaks (readout, `H`) and `Ctrl+S`/`Ctrl+L` confirmed at the seat.

### File List

- `crates/gui/src/command.rs` (`save_load_keys`)
- `crates/gui/src/ingest.rs` (registration, Ctrl camera gate, `--expect-haul`, keymap guard, tests)
- `crates/gui/src/designate.rs` (hint)
- `crates/gui/src/capture.rs` (stockpile count, print, assertion, test)
- `crates/gui/tests/pixel_guard.rs` (`expect_haul_fails_a_world_with_no_stone_on_a_stockpile`)
- `README.md` (controls, `--expect-haul`)
- `_bmad-output/implementation-artifacts/mutations/8-3-master-of-time-and-the-skeleton-walks-in-3d.sh` (new)
- `_bmad-output/implementation-artifacts/mutations/10-10-take-the-camera-where-you-want-it.sh` (one row re-pointed)
- `_bmad-output/implementation-artifacts/8-3-signoff/vehicle-card.md` (new)

## Change Log

| Date | Change |
|---|---|
| 2026-09-26 | Story created. Premises checked at `bfe27e4`: pause/speed/snap/two-client/daemon haul done; save/load keys, hint, gui haul instrument and seat sign-off remain. Quit premise corrected (window close, not `Command::Quit`). Haul observation measured via the daemon's snapshot: 0 → 7 on stockpile. |
| 2026-09-26 | Dev: Tasks 1-3 and 5 built (`2d3f12d`); RED/GREEN recipe observed (0 → exit 101; 7 → exit 0); 12/12 mutations KILLED; vehicle card written. The seat sitting (AC6-7) is pending. |
| 2026-09-26 | Task 6 (Wolf's HUD tweaks): clock/elapsed/speed readout and `H` HUD toggle (`08406e5`). The 7.1 and M2-1 mutation rows were re-anchored. |
