---
baseline_commit: 7371508
---

# Story 2.3: Master of Time

Status: in-progress

## Story

As the boss,
I want pause, normal, and fast-forward, with any number of terminals watching,
so that the session bends to my rhythm.

## Acceptance Criteria

1. `protocol` gains `Command`, internally tagged (`#[serde(tag = "type", rename_all =
   "snake_case")]`), with exactly one variant: `SetSpeed { speed: Speed }`. A hand-written
   literal `{"type":"set_speed","speed":"paused"}` decodes to it and re-encodes to the same
   JSON value; an unknown `type` fails to decode.
2. `simd` decodes every inbound client line as a `Command`. A decodable `set_speed` changes
   the daemon's speed; every other line keeps 1.2's behaviour exactly — logged, dropped,
   connection left open, daemon alive (including non-UTF-8 and oversized lines).
3. The daemon's real speed reaches the wire: `bridge::snapshot` and `bridge::delta` take it
   as an argument and the hardcoded `protocol::Speed::Normal` is gone from both. A client
   connecting while the sim is paused receives `"speed":"paused"` in its connect snapshot.
4. **Paused freezes sim time, not the daemon.** While paused the schedule does not run:
   across 10 consecutive deltas from a live daemon the `tick` value is identical and every
   entity `pos` is unchanged, while the deltas keep arriving at ~10/s. Resuming continues
   from the frozen tick — never a reset, never a catch-up jump.
5. Fast-forward is a loop-rate change only (20 ms period, 5×). Against a live daemon, 20
   deltas at `fast` take less than half the wall-clock of 20 deltas at `normal`. No file
   under `crates/sim-core/` changes in this story — `sim-core` learns no speed concept.
6. TUI keymap: `Space` toggles Paused↔Normal, `+` steps Paused→Normal→Fast, `-` steps
   Fast→Normal→Paused, both clamped at the ends. Each returns
   `Action::Command(Command::SetSpeed { .. })` computed from the last speed the wire
   reported; a key with nothing to change (`+` at Fast, `-` at Paused) returns
   `Action::Ignore` and sends nothing.
7. The client writes for the first time: a cloned write half sends one NDJSON command line
   per keypress, in both the interactive loop and `--frames`. No ack is expected — the next
   delta is the ack (NFR2).
8. The status line shows the current speed by its wire name and the speed keys, and fits in
   80 columns at `tick 9999999` in every speed — pinned by a test that asserts the rendered
   width, not just the text.
9. Two clients connected to one daemon both render the same speed, and a `set_speed` from
   either appears in the very next delta of both (FR19).
10. `tui --frames N --key <space|+|->` presses that key through `apply_key` before streaming
    frames. Its own tests drive the real binary: with `--key space` the tick in the captured
    status lines stops climbing and the line reads `paused`; with no `--key` the same capture
    shows it climbing.
11. `scripts/gate.sh` passes and `scripts/mutate.sh
    _bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh` reports zero
    survivors.

## Tasks / Subtasks

- [x] **`protocol`: the first upstream message** (AC: 1)
  - [x] `Command` enum, internally tagged, one `SetSpeed { speed: Speed }` variant. `Speed`
        already exists and needs no change. Add `Command` to the doc comment list of wire
        types.
  - [x] Hand-written literal test in the existing `WIRE`/`DELTA_WIRE` style: decode
        `{"type":"set_speed","speed":"paused"}`, assert the variant, assert
        `to_value(decoded) == from_str(LITERAL)`, and assert `{"type":"set_rate",...}` fails
        to decode. Extend `every_material_and_tile_variant_has_a_pinned_wire_name` with the
        command's `type` name.
- [x] **`simd`: parse inbound, drive the loop** (AC: 2, 3, 4, 5)
  - [x] `read_inbound` gains a `mpsc::Sender<protocol::Command>` (cloned per connection
        through `connect_client` → `serve`). On `serde_json::from_str::<Command>(text)` Ok →
        `send`; on Err → the existing `eprintln!("unrecognized client message: {}", excerpt(..))`
        path, unchanged. Keep the lossy-UTF-8 read and the `MAX_LINE_BYTES` cap as they are.
  - [x] `tick()` owns `let mut speed = protocol::Speed::Normal;`. At iteration start, before
        accepting new clients, drain the command channel with `try_iter()` and apply each
        `SetSpeed` in arrival order (AD-10: control commands are handled by `simd` directly,
        never a sim queue — there is no sim queue yet).
  - [x] `if speed != Speed::Paused { world.step(); }` — the whole schedule is skipped, which
        freezes the tick because `advance_tick` lives inside it. Add a `// NOTE:` that Story
        3.1's command-consuming system must run *while paused* (AD-2), which is when this
        single `if` splits into "consume commands" + "advance world".
  - [x] Period from speed: `Normal`/`Paused` → `TICK_PERIOD` (100 ms), `Fast` →
        `FAST_TICK_PERIOD` (20 ms), as an exhaustive `match` with no wildcard arm. Paused
        keeps the 100 ms cadence so deltas keep flowing. Compute the deadline from the
        *current* iteration's speed so a `set_speed fast` takes effect on that iteration's
        sleep.
  - [x] `bridge::snapshot(&world, speed)` / `bridge::delta(&mut world, speed)`; delete both
        hardcoded `Speed::Normal` literals. Keep `delta`'s single-call-per-iteration
        discipline — it drains the dirty set.
- [x] **`simd`: prove it against the live daemon** (AC: 2, 3, 4, 5, 9)
  - [x] `crates/simd/tests/serve.rs`, following the existing `Daemon` harness: a raw TCP
        client writes `{"type":"set_speed","speed":"paused"}\n`, then reads 10 consecutive
        deltas and asserts one identical `tick` and unchanged entity positions across all of
        them — **this is the seam assertion; it must fail if the decoded command's decision
        were parsed and discarded.** Then send `normal` and assert the tick resumes from the
        frozen value.
  - [x] Connect a second client while paused and assert its snapshot carries
        `"speed":"paused"` (AC3).
  - [x] Two clients, speed change sent by one, both observe it in their next delta (AC9).
  - [x] Rate test in the shape of `deltas_arrive_at_roughly_ten_per_second`: time 20 deltas
        at normal, send `fast`, time 20 more, assert the second span is under half the first.
        Compare the two measured spans rather than against an absolute clock — a loaded
        devpod must not make this flake.
  - [x] Re-run, unmodified, `malformed_input_is_dropped_and_daemon_survives`,
        `non_utf8_input_does_not_close_the_connection` and
        `oversized_line_is_refused_without_killing_the_daemon`. If any needs editing, the
        1.2 contract has been broken.
- [x] **`tui`: the keymap and the write half** (AC: 6, 7)
  - [x] `apply_key(&mut state, key, dims, speed: protocol::Speed) -> Action` — new fourth
        argument, passed `snapshot.speed` at the call site. `Action` gains
        `Command(protocol::Command)`. Every existing `apply_key` call in `view.rs` tests takes
        the new argument.
  - [x] `Space` / `+` / `-` arms returning `Action::Command`; the step tables written out as
        explicit `match speed` arms, not arithmetic on a derived ordinal.
  - [x] `main`: `stream.try_clone()` for a write half *before* `BufReader::new(stream)` — the
        reader is moved into the reader thread and cannot be borrowed back. On
        `Action::Command(c)`, `writeln!` the encoded JSON and flush. Delete 1.3's `// NOTE:`
        saying the client sends zero bytes and never closes its write half.
  - [x] No redraw is forced on send: the delta that carries the new speed is the ack and
        drives the repaint (NFR2). Add a `// NOTE:` for the known limitation — two presses
        inside one round-trip both compute from the same stale wire speed, so the second is
        a no-op. Optimistic local speed is deliberately not built.
- [x] **`tui`: the status line** (AC: 8)
  - [x] New format, dropping the camera coordinates to make room:
        `tick {t}  {speed}  z {z}/{maxz}  dwarves {n}  <>z hjkl  space +- speed  q quit`
        (74 columns at `tick 9999999`). This closes the recorded deferred overflow item —
        say so in the completion notes.
  - [x] `status_line_reports_z_camera_and_dwarf_count` is renamed and its expectation
        rewritten. Add the width test: render at `tick 9999999` for each of the three speeds
        and assert the status text is ≤ 80 columns and its last glyph is not truncated.
- [x] **Observability instrument** (AC: 10) — the human check for "the session bends to my
      rhythm". Extend the existing `tui --frames N` rather than inventing a second
      instrument: add `--key <space|+|->`, which builds the matching `KeyEvent`, runs it
      through the real `apply_key`, sends the resulting command down the real write half,
      then streams N frames as before.
  - [x] Two tests in `crates/tui/tests/client.rs` driving the real binary against a stub
        daemon that freezes its stub tick once it reads a `set_speed` line: with `--key space`
        the captured status lines stop climbing and contain `paused`; with no `--key` they
        climb. The second test is the mutation guard on the first — an instrument that always
        shows a frozen tick would pass the first alone.
  - [x] **Known limit, do not fight it:** no automated test can span both binaries.
        `CARGO_BIN_EXE_simd` is defined only for `simd`'s own tests and `CARGO_BIN_EXE_tui`
        only for `tui`'s, and `tui` may not take `simd` as a dev-dependency (gate.sh probes
        the dependency edges). The client half is proven against a stub daemon, the daemon
        half against a real daemon by raw TCP, and the two are joined by the manual live run
        in Verification. Paste what you saw.
- [x] **Sabotage + mutation set** (AC: 11)
  - [x] `_bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh` with at
        least: pause does not skip `world.step()`; the parsed command is dropped instead of
        applied; `FAST_TICK_PERIOD` equals `TICK_PERIOD`; the bridge hardcodes
        `Speed::Normal` again; `+` at `Fast` wraps to `Paused` instead of clamping; `Space`
        returns `Ignore`; the status line omits the speed. Run `scripts/mutate.sh` and paste
        the table.
  - [x] Paste the actual RED output for every new mapping/constant test into the Dev Agent
        Record (AGENTS.md rule 1).
- [x] **Green gate** (AC: 11) — `scripts/gate.sh`, then the live check. Report what printed.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No `save`, `load` or `quit` commands.** `Command` ships with exactly one variant. 2.4
  owns those keys (`S`/`L`) and the `SaveState`; `q` already quits the client locally and
  stays that way.
- **No world-mutating commands and no sim command queue.** AD-10's queue is Story 3.1's,
  arriving with `designate`. Control commands never touch it — that is the whole reason
  AD-10 splits them.
- **No `sim-core` change of any kind.** AD-2: `sim-core` never knows the wall-clock rate.
  AC5 makes "no file under `crates/sim-core/` changed" checkable.
- **No new modes, cursor, or hint bar.** FR21's modal input and the hint bar are 3.1. This
  story adds three keys to the existing flat keymap and one status-line field.
- **No optimistic client-side speed** and no ack message. The delta is the ack (spine
  convention: "no explicit ack messages").
- **No reconnect, no backpressure work.** A write to a dead daemon surfaces through the
  reader thread's existing error path.

### What already exists (build on it, do not re-derive)

- `simd`'s loop: `tick()` holds `clients`, drains `new_rx` for connects, calls
  `world.step()`, encodes one delta and broadcasts [crates/simd/src/main.rs:106-148]. The
  `mpsc` + per-connection-thread idiom for the command channel is already there in
  `new_tx`/`new_rx`.
- `read_inbound` is the log-and-drop site, with the `MAX_LINE_BYTES` cap, lossy UTF-8 and
  `excerpt()` [crates/simd/src/main.rs:227-264]. It currently treats *every* line as
  unrecognized by definition.
- `bridge::snapshot`/`bridge::delta` hardcode `protocol::Speed::Normal`
  [crates/simd/src/bridge.rs:31,65]. `delta` is destructive — one call per iteration.
- The `Daemon` test harness (spawns the real binary on port 0, parses the listening line,
  bounded reads, `next_log()` proving the daemon actually processed input)
  [crates/simd/tests/serve.rs:14-112], and the timing pattern in
  `deltas_arrive_at_roughly_ten_per_second` [serve.rs:184].
- `tui`: `apply_key` + `Action` [crates/tui/src/view.rs:152-206], the status line
  [view.rs:123-147], `stream_frames` with its fixed camera and `NO_COLOR` warning
  [crates/tui/src/main.rs:229-283], and the arg parser that already handles `--frames <n>`
  [main.rs:70-102]. `glyph_columns` in the client tests shows how to read a capture with the
  colour escapes stripped [crates/tui/tests/client.rs:102-125].
- `protocol::Speed { Paused, Normal, Fast }` with pinned wire names — it has existed and
  been entirely unused since 1.2 [crates/protocol/src/lib.rs:46-52]. This story is what it
  was for.
- `scripts/gate.sh`, `scripts/mutate.sh`, and the 2.2 mutations file as the format worked
  example.

### Code skeleton

```rust
// crates/protocol/src/lib.rs — the first client → daemon message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    SetSpeed { speed: Speed },
}
```

```rust
// crates/simd/src/main.rs
const FAST_TICK_PERIOD: Duration = Duration::from_millis(20); // 5x, AD-2

fn period(speed: protocol::Speed) -> Duration {
    match speed {
        // Paused keeps the normal cadence: the loop never stops, only the sim does.
        protocol::Speed::Paused | protocol::Speed::Normal => TICK_PERIOD,
        protocol::Speed::Fast => FAST_TICK_PERIOD,
    }
}

// inside tick(), per iteration:
for command in command_rx.try_iter() {
    match command {
        protocol::Command::SetSpeed { speed: next } => speed = next,
    }
}
let deadline = Instant::now() + period(speed);
// ... accept new clients, snapshot encoded with `speed` ...
// NOTE: the whole schedule is world-advancing today, so pausing skips all of it and the
// tick freezes with it (advance_tick is inside). Story 3.1 adds a command-consuming
// system that must run WHILE paused (AD-2) — that is when this splits in two.
if speed != protocol::Speed::Paused {
    world.step();
}
let delta_line = Arc::new(format!("{}\n", serde_json::to_string(&bridge::delta(&mut world, speed))?));
```

```rust
// crates/tui/src/view.rs — written as explicit arms, never arithmetic on an ordinal
KeyCode::Char(' ') => command(match speed {
    Speed::Paused => Speed::Normal,
    Speed::Normal | Speed::Fast => Speed::Paused,
}),
KeyCode::Char('+') => match speed {
    Speed::Paused => command(Speed::Normal),
    Speed::Normal => command(Speed::Fast),
    Speed::Fast => Action::Ignore, // clamped, sends nothing
},
KeyCode::Char('-') => match speed {
    Speed::Fast => command(Speed::Normal),
    Speed::Normal => command(Speed::Paused),
    Speed::Paused => Action::Ignore,
},
```

`+` and `>` both arrive with SHIFT held; `apply_key` already lets SHIFT through and rejects
every other modifier [view.rs:156-164], so the three new keys need no change there.

### Key decisions & traps

- **Pause is one `if` around `world.step()`, and that is the whole mechanism.** It satisfies
  AD-2 today only because every system in the schedule is world-advancing. The `// NOTE:` is
  not decoration — it is the handoff that keeps 3.1 from wiring its command consumer inside
  the skipped branch.
- **The seam is the decision, not the parse.** A `set_speed` that is decoded, logged and
  then ignored looks correct in every log line and every unit test of the parser. AC4's
  frozen-tick assertion across 10 real deltas is the only test that dies when the decision is
  discarded — write it before the loop change and watch it go red.
- **`try_clone` the socket before it enters the `BufReader`.** `main` moves `reader` into the
  reader thread [crates/tui/src/main.rs:146]; there is no way to reach the stream afterwards.
  Clone at line 110, before `BufReader::new`.
- **The client derives the next speed from the wire, and that is a deliberate limitation.**
  Two presses inside one 100 ms round-trip both read the same stale `snapshot.speed`, so the
  second is lost. Naming it in a `// NOTE:` is the requirement; building optimistic local
  state is not (AD-4, and YAGNI).
- **The status line loses the camera coordinates.** That is how the speed and its keys fit in
  80 columns, and it closes the deferred overflow item that was explicitly handed to "the
  next TUI story touching the status line". `status_line_reports_z_camera_and_dwarf_count`
  changes name and expectation — sanctioned, and the only pinned-test rewrite in this story.
- **`--frames` is an evidence channel with a history.** 2.2 shipped it re-centring the camera
  every frame, so it rendered motion as stillness and the story's live evidence was an
  artefact of the instrument. Its two failure modes now have regression tests; do not
  reintroduce a per-frame `initial()`, and keep the `NO_COLOR` warning intact. AC10's second
  test (tick climbs without `--key`) exists for the same reason: an instrument that always
  shows a frozen tick would satisfy the first test alone.
- **Timing tests are the flake risk in this story.** Compare two measured spans against each
  other, never a span against an absolute wall-clock expectation.
- **The existing stub daemons in `client.rs` never read from their socket.** That stays
  harmless at one short command line, so do not teach them to read unless the test needs the
  command. Keep a client-side write error fatal (`?`, like every other I/O path here) — Rust
  ignores SIGPIPE, so a dead peer surfaces as an `Err`, not a killed process.
- **Parse the trimmed line, not the raw buffer,** and parse the same `String::from_utf8_lossy`
  text the log already builds — one decode attempt, two outcomes, no second read of the bytes.
- **Hand-off:** 2.4 adds `Save`, `Load` and `Quit` variants to `Command` and the `S`/`L`
  keys; 3.1 adds the world-mutating variants *and* the AD-10 sim queue they ride, and owns
  the pause/command-consumption split named above.

### Project Structure (files to touch)

```
crates/protocol/src/lib.rs      # UPDATE — Command enum + hand-written literal test
crates/simd/src/main.rs         # UPDATE — command channel, speed state, pause branch, period()
crates/simd/src/bridge.rs       # UPDATE — speed parameter on snapshot/delta, drop the hardcode
crates/simd/tests/serve.rs      # UPDATE — frozen tick, resume, snapshot-while-paused, fast rate, two clients
crates/tui/src/main.rs          # UPDATE — write half, --key arg, send on Action::Command
crates/tui/src/view.rs          # UPDATE — apply_key speed arg, Action::Command, three keys, status line
crates/tui/tests/client.rs      # UPDATE — the two instrument tests
_bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh  # NEW
```

`crates/sim-core/**` — must not appear in the File List (AC5).

### Previous story intelligence (2.2)

- The `--frames` instrument shipped broken twice over and both defects were found at review,
  not by the suite. Its tests are now in `crates/tui/tests/client.rs`; extend that file
  rather than starting a new harness.
- `scripts/mutate.sh` caught a review patch whose new test passed with the fix removed. Every
  new assertion here goes through it before you claim it works.
- This devpod sets `NO_COLOR=1`. It does not affect this story's signal — the tick and the
  word `paused` are plain glyphs — but `glyph_columns`'s escape-stripping is the pattern to
  copy when reading a capture.

### Verification

```bash
scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh
```

Live instrument — the observable outcome, joining the two binaries no test can span:

```bash
cargo run -p simd &
cargo run -p tui -- --frames 20 > /tmp/normal.txt          # tick climbs
cargo run -p tui -- --frames 20 --key space > /tmp/paused.txt
rg -n 'tick [0-9]+' /tmp/paused.txt | tail -5              # same tick, and "paused" on the line
cargo run -p tui                                           # Space freezes it, + speeds it up, - slows it, q -> y
cargo run -p tui                                           # second terminal: a speed change in either shows in both
```

Branch: `2-3-master-of-time`. Commit as `Völundr <jeicei75@gmail.com>`, one commit per green
step, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.3] — user story, source ACs, and
  the dependency-sweep `// NOTE:` naming this the first story in which the client writes
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md]
  — AD-2 (loop never stops, tick freezes, fast = loop-rate change), AD-3, AD-4, AD-6
  (closed vocabularies as enums), AD-8, AD-10 (control commands handled by `simd` directly),
  and the protocol v0 message table
- [Source: _bmad-output/planning-artifacts/epics.md#Requirements Inventory] — FR14, FR18,
  FR19, NFR2
- [Source: _bmad-output/implementation-artifacts/2-2-dwarves-wander-the-frost.md] — the
  instrument's two defects and their regression tests, the oracle-table pattern
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — the status-line overflow
  item this story closes, and the still-open `NO_COLOR` product-half item it does not
- [Source: AGENTS.md] — sabotage rule, honest reporting, the codex self-gate

## Dev Agent Record

### Agent Model Used

OpenAI Codex (GPT-5)

### Debug Log References

- Protocol command RED (test written before `Command` existed):

  ```text
  error[E0425]: cannot find type `Command` in this scope
     --> crates/protocol/src/lib.rs:212:22
      |
  212 |         let command: Command = serde_json::from_str(COMMAND_WIRE)
      |                      ^^^^^^^ not found in this scope
  error: could not compile `protocol` (lib test) due to 4 previous errors
  ```

- Protocol discriminator sabotage RED (`SetSpeed` temporarily renamed to `set_rate`):

  ```text
  running 1 test
  test tests::decodes_and_reencodes_the_documented_command_wire_format ... FAILED

  thread 'tests::decodes_and_reencodes_the_documented_command_wire_format' panicked at crates/protocol/src/lib.rs:222:14:
  the documented command wire format must decode: Error("unknown variant `set_speed`, expected `set_rate`", line: 1, column: 19)
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out
  ```

- Daemon decision-seam RED before the loop handled commands:

  ```text
  running 1 test
  thread 'paused_daemon_freezes_tick_and_entities_then_resumes' panicked at crates/simd/tests/serve.rs:162:5:
  every observed delta after the command must acknowledge paused
  test paused_daemon_freezes_tick_and_entities_then_resumes ... FAILED
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out
  ```

- Parsed-command-drop sabotage RED:

  ```text
  running 1 test
  thread 'speed_change_from_either_client_reaches_both_on_the_same_delta' panicked at crates/simd/tests/serve.rs:136:5:
  daemon never reported Paused; observed [(2, Normal), (3, Normal), (4, Normal), (5, Normal), (6, Normal)]
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out
  ```

- Paused snapshot bridge sabotage RED (`snapshot` temporarily hardcoded `Normal`):

  ```text
  running 1 test
  thread 'client_connecting_while_paused_receives_paused_snapshot' panicked at crates/simd/tests/serve.rs:225:5:
  assertion `left == right` failed
    left: Normal
   right: Paused
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out
  ```

- Pause-branch sabotage RED (`world.step()` temporarily unconditional):

  ```text
  running 1 test
  thread 'paused_daemon_freezes_tick_and_entities_then_resumes' panicked at crates/simd/tests/serve.rs:196:5:
  paused ticks changed: [11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out
  ```

- Fast-period boundary sabotage RED (`FAST_TICK_PERIOD` temporarily 100 ms):

  ```text
  running 1 test
  thread 'fast_deltas_arrive_in_under_half_the_normal_span' panicked at crates/simd/tests/serve.rs:363:5:
  fast span 2.002124622s was not under half normal span 1.78344666s
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out
  ```

- Paused-period mapping sabotage RED (`Paused` temporarily used the 20 ms period):

  ```text
  running 1 test
  thread 'paused_daemon_freezes_tick_and_entities_then_resumes' panicked at crates/simd/tests/serve.rs:209:5:
  paused span 180.964739ms did not keep normal cadence relative to 688.222106ms
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out
  ```

- Delta bridge sabotage RED (`delta` temporarily hardcoded `Normal`):

  ```text
  running 1 test
  thread 'bridge::tests::delta_carries_dirty_tiles_and_full_authoritative_state' panicked at crates/simd/src/bridge.rs:352:9:
  assertion `left == right` failed
    left: Normal
   right: Fast
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out
  ```

- TUI keymap RED before `Action::Command` and the speed argument existed:

  ```text
  error[E0599]: no variant, associated function, or constant named `Command` found for enum `view::Action` in the current scope
     --> crates/tui/src/view.rs:248:25
  error[E0061]: this function takes 3 arguments but 4 arguments were supplied
     --> crates/tui/src/view.rs:302:17
  error: could not compile `tui` (bin "tui" test) due to 8 previous errors
  ```

- Fast-endpoint clamp sabotage RED (`+` at Fast temporarily wrapped to Paused):

  ```text
  running 1 test
  assertion `left == right` failed: wrong action for Char('+') at Fast
    left: Command(SetSpeed { speed: Paused })
   right: Ignore
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 22 filtered out
  ```

- Space mapping sabotage RED (Space temporarily returned `Ignore`):

  ```text
  running 1 test
  assertion `left == right` failed: wrong action for Char(' ') at Paused
    left: Ignore
   right: Command(SetSpeed { speed: Normal })
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 22 filtered out
  ```

- Status-line RED before the new format was implemented:

  ```text
  running 2 tests
  assertion `left == right` failed
    left: "tick 87  z 19/31  camera 12,34  dwarves 3  keys: <> z  arrows/hjkl pan  q quit"
   right: "tick 87  normal  z 19/31  dwarves 3  <>z hjkl  space +- speed  q quit         "
  assertion `left == right` failed
    left: 80
   right: 74
  test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 22 filtered out
  ```

- Status-speed omission sabotage RED:

  ```text
  running 1 test
  assertion `left == right` failed
    left: 66
   right: 74
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 23 filtered out
  ```

- Status-speed name mapping sabotage RED (`Paused` temporarily rendered as `normal`):

  ```text
  running 1 test
  assertion `left == right` failed
    left: "tick 9999999  normal  z 19/31  dwarves 5  <>z hjkl  space +- speed  q quit"
   right: "tick 9999999  paused  z 19/31  dwarves 5  <>z hjkl  space +- speed  q quit"
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 23 filtered out
  ```

- `--key space` instrument RED before the argument existed:

  ```text
  running 1 test
  thread '<unnamed>' panicked at crates/tui/tests/client.rs:72:17:
  tui did not connect to stub daemon within 3s
  thread 'key_space_freezes_the_streamed_frame_tick_and_reports_paused' panicked at crates/tui/tests/client.rs:174:19:
  stub daemon thread panicked: Any { .. }
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out
  ```

- Observability mutation-guard RED (delta ticks temporarily ignored):

  ```text
  running 1 test
  assertion `left == right` failed
    left: [7, 7, 7]
   right: [8, 9, 10]
  test streamed_frame_ticks_climb_when_no_key_is_sent ... FAILED
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out
  ```

- Required mutation run (`scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh`):

  ```text
  ================ MUTATION RESULTS ================
  pause no longer skips world.step                             KILLED
  parsed speed command is dropped before the loop              KILLED
  fast period equals normal period                             KILLED
  paused loop uses the fast period                             KILLED
  snapshot bridge hardcodes normal speed                       KILLED
  delta bridge hardcodes normal speed                          KILLED
  plus at fast wraps to paused                                 KILLED
  space key is ignored                                         KILLED
  status line omits the speed                                  KILLED
  frames instrument ignores incoming ticks                     KILLED
  frames key path never writes its command                     KILLED
  set_speed discriminator is renamed                           KILLED

  All mutations killed.
  ```

- Final gate after clearing stale mutation build artifacts (`cargo clean -p protocol -p simd -p tui`):

  ```text
  frostvein gate
    cargo fmt --check           ok
    cargo clippy -D warnings    ok
    cargo test                  ok
    tui has no sim-core edge    ok
  GATE GREEN
  ```

- Mutation-runner diagnostic: the first post-mutation manual `cargo run` exposed a stale
  mutated `simd` artifact (`bridge::delta` warned that `speed` was unused) even though the
  restored source and git diff were correct. `scripts/mutate.sh` restores source timestamps,
  so Cargo had reused the last mutation build. Removed only the `protocol`, `simd`, and `tui`
  package artifacts, reran the gate from a forced rebuild, and observed the green output above.

- Manual live check, real `simd` + real `tui` binaries (`NO_COLOR` warned as expected; speed
  and tick are plain glyphs and remained observable):

  ```text
  normal capture tail:
  tick 71  normal
  tick 72  normal
  tick 73  normal
  tick 74  normal
  tick 75  normal

  paused capture tail after --key space:
  tick 147  paused
  tick 147  paused
  tick 147  paused
  tick 147  paused
  tick 147  paused

  --key + from paused tail: tick 186..190  normal
  second --key + tail:      tick 206..210  fast
  --key - from fast tail:   tick 232..233  normal
  second --key - tail:      tick 238       paused (five identical frames)

  simultaneous real clients:
  observer first: tick 1338 paused (five identical frames)
  controller tail after +: tick 1349..1353 normal
  observer later: tick 1529..1538 normal
  ```

### Completion Notes List

- Added the one-variant, internally tagged `protocol::Command` wire type and pinned its literal `set_speed` JSON contract, including unknown-command rejection.
- Routed decoded control commands from every connection into the daemon loop; pause now freezes world time while deltas continue at normal cadence, fast changes only the loop period, and snapshots/deltas carry authoritative speed.
- Proved pause/resume, paused connect snapshots, same-delta two-client propagation, and relative fast timing against the real daemon. Re-ran the three named Story 1.2 input contracts unmodified; all passed.
- Added the explicit Space/+/− speed-step table, clamped endpoints, and first TUI write half. Commands are computed from authoritative wire speed, written and flushed as one NDJSON line, and rely on the next delta for repaint/ack.
- Replaced the status line with the compact authoritative speed/z/dwarf/key format and pinned all three wire speed names plus rendered width at tick 9999999. This closes the recorded deferred status-line overflow item.
- Extended the existing `--frames` evidence channel with `--key <space|+|->`, routing the synthetic key through real `apply_key` and the real socket write path before streaming. Real-binary tests prove Space freezes/reporting paused and that no-key captures still climb.
- Authored and ran twelve production sabotages covering the command wire, daemon decision seam, cadence boundaries, bridge speed, TUI mappings/status, write path, and observability mutation guard; zero survived.
- Manual cross-binary verification joined the real daemon and TUI: normal climbed, Space froze tick, + stepped through normal/fast, - stepped through normal/paused, and a persistent second client adopted the controller's speed change.

### File List

- `_bmad-output/implementation-artifacts/2-3-master-of-time.md`
- `crates/protocol/src/lib.rs`
- `crates/simd/src/bridge.rs`
- `crates/simd/src/main.rs`
- `crates/simd/tests/serve.rs`
- `crates/tui/src/main.rs`
- `crates/tui/src/view.rs`
- `crates/tui/tests/client.rs`
- `_bmad-output/implementation-artifacts/mutations/2-3-master-of-time.sh`

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-04 | Story created |
| 2026-08-04 | Implemented authoritative pause/normal/fast control across protocol, daemon, and TUI with live-daemon, real-binary instrument, mutation, gate, and manual verification. |
