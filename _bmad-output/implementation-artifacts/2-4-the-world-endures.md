---
baseline_commit: 7362850
---

# Story 2.4: The World Endures

Status: in-progress

## Story

As the boss,
I want to save the world, load it back, and quit cleanly,
so that a session can end without the fortress being lost.

## Acceptance Criteria

1. `sim-core` gains `SaveState` (with `SavedDwarf`), a plain serde struct in
   `crates/sim-core/src/save.rs` carrying exactly: seed, tick, dims, tiles, the live wander
   RNG state, the id-allocator next value, and each dwarf's id, pos, job state, wander home
   and wander cooldown. `World::to_save()` produces it; `World::from_save()` builds a world
   from it. `sim-core` still performs no file I/O of any kind (AD-1).
2. **The AD-11 gate test.** From seed 42: tick 37, mutate one tile with `set_tile`, save,
   load. Then tick the loaded world and a never-saved control 200 further times, asserting
   equal `tick()`, equal `dwarves()` and the mutated tile **at every step**, not only at the
   end. The oracle is the public API — comparing `to_save()` against `to_save()` does not
   satisfy this AC (see the self-referential trap below).
3. Load reuses no ids and dirties no tiles: after save → load the allocator's next value is
   still 5 and `drain_dirty()` is empty (AD-9, AD-8 — construction does not dirty-track).
4. `protocol::Command` gains the unit variants `Save`, `Load` and `Quit`. The literals
   `{"type":"save"}`, `{"type":"load"}` and `{"type":"quit"}` each decode and re-encode to
   the same JSON value; `{"type":"store"}` fails to decode.
5. `simd` handles all three itself at iteration top, never through a sim queue (AD-10).
   `save` serializes `world.to_save()` to `frostvein.save` (relative to the daemon's working
   directory) by writing `frostvein.save.tmp` and renaming it, then logs
   `saved tick {tick} to {path}`. The file on disk decodes back into a `sim_core::SaveState`
   whose tick equals the logged tick.
6. `load` replaces the world wholesale via `from_save` and broadcasts one fresh `snapshot`
   line to **every** connected client before that iteration's delta. With two clients
   connected and one sending `load`, both receive that snapshot, and its tick equals the tick
   named in the earlier `saved tick ...` log line — a tick strictly lower than the last delta
   each client had seen. A client's tick only ever decreases across a snapshot (AD-11).
7. A failed `save` or `load` (missing file, undecodable file, unwritable path) is logged and
   dropped: the daemon keeps ticking, keeps streaming deltas, sends no snapshot, and does not
   panic.
8. `quit` shuts the daemon down cleanly — the process exits 0 within the harness timeout,
   connected clients see EOF, and nothing panics.
9. TUI keymap: `S` returns `Action::Command(Command::Save)` and `L` returns
   `Action::Command(Command::Load)`, at every wire speed and independent of it. `q` still
   opens the local quit confirm and quits the client only; **no key in the client ever sends
   `Command::Quit`** (a shared daemon must not die from one viewer's keypress).
10. The client treats a mid-stream `snapshot` as an authoritative full reset while keeping
    its camera and z-level — the arm 1.3 wrote and marked unreachable is now live in both the
    interactive loop and `--frames`.
11. `tui --frames N --key <S|L>` presses that key through the real `apply_key` and the real
    write half before streaming. Its tests drive the real binary against a stub daemon: with
    `--key L` the captured status-line ticks jump **back** to the stub's saved tick and then
    climb from there; with no `--key` the same capture climbs monotonically and never jumps
    back.
12. `scripts/gate.sh` passes and `scripts/mutate.sh
    _bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh` reports zero
    survivors.

## Tasks / Subtasks

- [x] **Dependencies: turn on what AD-11 needs** (AC: 1)
  - [x] Workspace `Cargo.toml`: `rand_chacha = { version = "0.10.0", features = ["serde"] }`.
        The feature is optional and currently off; `ChaCha8Rng` only derives
        `Serialize`/`Deserialize` with it (verified: `rand_chacha-0.10.0/Cargo.toml:53`,
        and the crate's own `test_chacha_serde_roundtrip`). This closes the standing
        rand_chacha debt carried from Epic 1's retrospective.
  - [x] `crates/sim-core/Cargo.toml`: add `serde.workspace = true` (derive). No `serde_json`
        — `sim-core` produces a struct, `simd` produces bytes. No other new dependency; do
        not reach for `tempfile`.
  - [x] `.gitignore`: add `/frostvein.save` and `/frostvein.save.tmp` — a live run drops the
        save beside `Cargo.toml`.

- [x] **`sim-core`: `SaveState`, `to_save`, `from_save`** (AC: 1, 3)
  - [x] New `crates/sim-core/src/save.rs` with `SaveState` + `SavedDwarf` (skeleton below).
        Derive `Serialize`/`Deserialize` on the sim types it carries: `Material`, `Tile`,
        `Pos`, `Dims`, `JobState`. These are **not** wire types — AD-6 governs the protocol,
        and the save file is `sim-core`'s own format with no external consumer. Do not import
        `protocol` into `sim-core` and do not build the save out of `protocol::Snapshot`
        (AD-11 names it "a separate, lossy client projection", never used for saving).
  - [x] `dwarves` in the save are sorted ascending by id, and `from_save` spawns them in that
        order (AD-7 stable order).
  - [x] **Extract the one place the world is assembled.** `generate` and `from_save` must not
        each wire the schedule: a system added to one chain and not the other diverges
        silently and only ever shows up as a scenario mismatch nobody can explain. Both call
        a private `assemble(...)` that inserts `Tick`, `WanderRng`, `Terrain` (with an empty
        dirty set) and chains `(advance_tick, wander)`.
  - [x] `to_save` reads the live `WanderRng` (clone the `ChaCha8Rng`, do not reseed), the
        `Terrain` tiles, `Tick`, `ids.next`, and each dwarf's `Wander { home, cooldown }` —
        `World::dwarves()` does not expose `Wander`, so iterate entities directly the way it
        does.

- [x] **`sim-core`: the gate test** (AC: 2, 3) — new `crates/sim-core/tests/save_load.rs`
  - [x] `save_load_then_tick_matches_never_saved`: seed 42, **tick 37 before saving** and
        `set_tile` one tile first. Ticking first is not decoration — at tick 0 a `from_save`
        that reseeded the wander stream from the seed and reset every cooldown to its spawn
        value would pass. Compare against a never-saved control for 200 further steps,
        asserting `tick()`, `dwarves()` and the mutated tile after **each** step.
  - [x] `loading_does_not_reuse_entity_ids`: after save → load, `to_save().next_id == 5`.
  - [x] `loading_starts_with_no_dirty_tiles`: after save → load, `drain_dirty()` is empty.
  - [x] Do NOT add a hand-written save-format literal test. Save-format stability is an
        explicit project non-goal; the round-trip through the real types is the contract.

- [x] **`protocol`: three unit commands** (AC: 4)
  - [x] `Command` gains `Save`, `Load`, `Quit` (internally tagged unit variants encode as
        `{"type":"save"}`). Extend the existing literal test and
        `every_material_and_tile_variant_has_a_pinned_wire_name` with the three new `type`
        names, and keep the `{"type":"store"}` rejection assertion.

- [x] **`simd`: file I/O, broadcast-on-load, clean exit** (AC: 5, 6, 7, 8)
  - [x] `const SAVE_PATH: &str = "frostvein.save";` at use site (spine convention). The path
        is working-directory-relative and never comes from the client — a client-supplied
        path is a write-anywhere primitive.
  - [x] Extend the command drain's `match` with the three arms (skeleton below). `Quit`
        returns `Ok(())` from `tick()`; `main` returns and the process exits 0, closing every
        client socket with it. Log `shutting down on client quit` first so `next_log()` has a
        signal.
  - [x] `save`: encode `world.to_save()`, write `{SAVE_PATH}.tmp`, `fs::rename` onto
        `SAVE_PATH`. Rename is atomic on one filesystem, so a daemon killed mid-write leaves
        the previous save intact rather than a truncated file that decodes as nothing.
  - [x] `load`: decode the file into `sim_core::SaveState`, `World::from_save`, assign over
        `world`, then encode one snapshot and `broadcast` it to `clients` immediately —
        before the accept loop and before the delta. New clients admitted later in the same
        iteration already encode from the new world (the accept loop runs after the drain).
  - [x] Both are fallible and neither may kill the daemon: on any error `eprintln!` and carry
        on with the world untouched. Never `?` a save/load error out of `tick()`.
  - [x] `// NOTE:` the two known costs: encoding ~524k tiles stalls that one iteration
        (~0.2 s, the same order as the connect snapshot), and speed is `simd` state so a load
        while paused leaves the loop paused on the loaded tick.

- [x] **`simd`: prove it against the live daemon** (AC: 5, 6, 7, 8) — `crates/simd/tests/serve.rs`
  - [x] Make the `Daemon` harness hermetic: spawn with `current_dir` set to a fresh unique
        directory under `std::env::temp_dir()`, keep the path on `Daemon`, remove it in
        `Drop`. No test may write into the repo tree, and two tests must never share one save
        file. Build the unique name from the bound port plus `process::id()` — no new crate.
  - [x] `save_then_load_rewinds_every_client`: two clients; one sends `save`; read the
        `saved tick {t} to ...` log line and parse `t`; read deltas past `t + 10`; send
        `load`; assert **both** clients then receive a line whose `type` is `snapshot` with
        `tick == t`, strictly below the last delta tick each had seen. The logged tick is
        what makes this exact instead of racy.
  - [x] `saved_file_decodes_as_a_save_state`: read `frostvein.save` from the daemon's temp
        cwd and `serde_json::from_str::<sim_core::SaveState>`; its tick equals the logged
        tick. (`simd`'s tests may use `sim_core` — the package depends on it.)
  - [x] `load_without_a_save_file_is_logged_and_the_daemon_keeps_ticking`: send `load` first
        thing in a fresh cwd; assert an error log, then that deltas keep arriving with the
        tick still climbing and no snapshot line in between.
  - [x] `quit_exits_the_daemon_cleanly`: send `quit`; assert the client's socket reaches EOF
        and the child exits with success within `IO_TIMEOUT` (add a `wait_for_exit` helper;
        `Drop`'s `kill()` on an already-exited child is a harmless `Err`).
  - [x] Re-run unmodified: `malformed_input_is_dropped_and_daemon_survives`,
        `non_utf8_input_does_not_close_the_connection`,
        `oversized_line_is_refused_without_killing_the_daemon`, and 2.3's speed tests. If any
        needs editing, an earlier contract has been broken.

- [ ] **`tui`: two keys and a live snapshot arm** (AC: 9, 10)
  - [ ] `apply_key`: `KeyCode::Char('S')` → `Action::Command(Command::Save)`,
        `KeyCode::Char('L')` → `Action::Command(Command::Load)`. Uppercase arrives with SHIFT
        and `apply_key` already lets SHIFT through, so no modifier change. Extend the pinned
        keymap table test with both, asserted at all three speeds to pin that they do not
        depend on speed. `q` and the confirm path are untouched.
  - [ ] Delete the "this arm is unreachable today" half of the `Msg::Snapshot` NOTE in
        `main`'s message loop — the load broadcast makes it live — and keep the rest: adopt
        the world, **keep** `state.camera` and `state.z`. Never call `initial()` there; that
        would throw away where the player was looking on every load.
  - [ ] Status line and hint text are unchanged. FR21's hint bar is Story 3.1's, and the
        80-column budget pinned by 2.3's width test has no room for a save/load hint. The
        observable for `S` is the daemon's log line and the file; for `L` it is the tick
        jumping back on screen.

- [ ] **Observability instrument** (AC: 11) — extend `tui --frames N --key`, do not invent a
      second channel. `--key` accepts `S` and `L` alongside `space`/`+`/`-` (update the parse
      arm and its error text).
  - [ ] Two tests in `crates/tui/tests/client.rs`, real binary against a stub daemon that
        streams climbing deltas and, on reading a `{"type":"load"}` line, sends a snapshot at
        an earlier tick and then resumes climbing from it: with `--key L` the captured ticks
        jump back to that tick; with no `--key` they climb monotonically and never jump back.
        The second is the mutation guard on the first — an instrument that rendered a stale
        first snapshot forever would satisfy the first alone.
  - [ ] This is also the only automated proof of AC10: `stream_frames` adopting a mid-stream
        snapshot is the same arm the interactive loop uses.

- [ ] **Sabotage + mutation set** (AC: 12)
  - [ ] `_bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh`, at least:
        `from_save` drops the wander cooldown; drops the wander home; reseeds the wander RNG
        from the seed instead of restoring it; `assemble` wires only `advance_tick` for the
        loaded world; `from_save` regenerates terrain from the seed instead of restoring
        tiles; `from_save` resets the id allocator; `from_save` marks every tile dirty;
        `to_save` records tick 0; the daemon's `Save` arm parses but writes nothing; the
        `Load` arm replaces the world but skips the snapshot broadcast; the `Quit` arm is
        ignored; `S` maps to `Load`; the `--key` path never writes its command; the `save`
        discriminator is renamed. Run `scripts/mutate.sh` and paste the table.
  - [ ] Paste the actual RED output for every new mapping/constant test into the Dev Agent
        Record (AGENTS.md rule 1).

- [ ] **Green gate** (AC: 12) — `scripts/gate.sh`, then the live check. Report what printed.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No jobs, claims, designations or zones in `SaveState`.** They do not exist yet; 3.1 and
  3.2 extend the struct when they do. Do not add empty placeholder fields.
- **No save-format versioning, migration, or compatibility shim.** "No save-format stability
  guarantees" is an explicit project non-goal.
- **No autosave, no save slots, no timestamped files, no save-on-quit, no client-supplied
  path.** One file, one key each way.
- **No compression and no binary format** (AD-3's spirit; chatty and plain is sanctioned).
- **No quit key in the client.** `q` is client-local, as shipped. Nothing in `tui` sends
  `Command::Quit`.
- **No fix for `read_inbound`'s partial-line-as-overflow log** — recorded in
  `deferred-work.md`, owned by 3.1, and this story does not touch that function.
- **No optimistic client-side speed, no new modes, no hint bar, no status-line change** —
  all 3.1's.
- **No `sim-core` I/O.** `to_save()` hands back a struct; `simd` owns every byte written.
- **No reconnect, no backpressure work.**

### What already exists (build on it, do not re-derive)

- `World` holds `ecs`, `schedule`, `ids: IdAllocator`, `seed`; the retained sim RNG is the
  `WanderRng` resource, and `Wander { home, cooldown }` is a **private** component
  [crates/sim-core/src/lib.rs:76-93,211-260]. The worldgen and spawn RNGs are locals inside
  `generate()` and are dropped when it returns — nothing retains them, so nothing serializes
  them; their output is already materialized in `tiles` and the dwarf rows.
- `Terrain` owns `dims`, `tiles` and the `dirty` `BTreeSet`; `set_tile` is the only mutator
  and `drain_dirty` the only drain [lib.rs:87-131].
- `simd`'s loop drains `command_rx` at iteration top, then accepts clients (encoding the
  connect snapshot lazily, once), then steps, then encodes one delta and broadcasts
  [crates/simd/src/main.rs:117-165]. `broadcast` already prunes dead clients [main.rs:175].
- `read_inbound` already decodes every inbound line as `protocol::Command` and forwards it —
  new variants need no change there [main.rs:258-296].
- `tui`'s `apply_key` + `Action::Command`, the arg parser's `--frames`/`--key` handling, the
  write half and `send_command` [crates/tui/src/view.rs:157-226,
  crates/tui/src/main.rs:63-153,263-282], and the `Msg::Snapshot` arm that has been waiting
  since 1.3 [main.rs:224-227, and `stream_frames` at main.rs:334].
- Test scaffolding to extend rather than replace: the `Daemon` harness and `next_log()`
  [crates/simd/tests/serve.rs:14-96], the stub-daemon + `strip_ansi` capture pattern
  [crates/tui/tests/client.rs:96-135], and the 2.3 mutations file as the worked format.

### Code skeleton

```rust
// crates/sim-core/src/save.rs — sim-core's own format. Not a wire type; `protocol` is
// not imported here and never will be.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveState {
    pub seed: u64,
    pub tick: u64,
    pub dims: Dims,
    pub tiles: Vec<Tile>,
    pub wander_rng: ChaCha8Rng,
    pub next_id: u32,
    pub dwarves: Vec<SavedDwarf>, // ascending by id
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SavedDwarf {
    pub id: u32,
    pub pos: Pos,
    pub state: JobState,
    pub home: Pos,     // private `Wander` field — the one most easily forgotten
    pub cooldown: u32, // ditto
}
```

```rust
// crates/sim-core/src/lib.rs — ONE assembly site, so the schedule cannot diverge between
// a generated world and a loaded one.
fn assemble(seed: u64, dims: Dims, tiles: Vec<Tile>, tick: u64, wander_rng: ChaCha8Rng,
            ids: IdAllocator) -> World {
    let mut ecs = EcsWorld::new();
    ecs.insert_resource(Tick(tick));
    ecs.insert_resource(WanderRng(wander_rng));
    ecs.insert_resource(Terrain { dims, tiles, dirty: BTreeSet::new() });
    let mut schedule = Schedule::default();
    schedule.add_systems((advance_tick, wander).chain());
    World { ecs, schedule, ids, seed }
}
// `generate` keeps its worldgen calls and its `spawn_dwarves`; `from_save` spawns the
// saved rows instead. Neither builds a schedule of its own. Do not rename the `wander`
// system to dodge the parameter shadow — name the parameter around it, as above.
```

```rust
// crates/simd/src/main.rs — inside the existing iteration-top drain
for command in command_rx.try_iter() {
    match command {
        protocol::Command::SetSpeed { speed: next } => speed = next,
        protocol::Command::Save => save_world(&world),   // logs on success AND on failure
        protocol::Command::Load => {
            if let Some(loaded) = load_world() {
                world = loaded;
                let line = Arc::new(format!(
                    "{}\n", serde_json::to_string(&bridge::snapshot(&world, speed))?));
                broadcast(&mut clients, &line);
            }
        }
        protocol::Command::Quit => {
            eprintln!("shutting down on client quit");
            return Ok(());
        }
    }
}
```

### Key decisions & traps

- **The self-referential save test is this story's headline trap.** `assert_eq!(a.to_save(),
  b.to_save())` is green for exactly the bug it should catch: a field missing from
  `SaveState` is missing from both sides. The oracle is `tick()` / `dwarves()` / `tile()`
  **after ticking forward**, because hidden state (the wander cooldown, the wander home, the
  RNG stream position) is invisible the instant after a load and only diverges once the sim
  runs. This class has now been hit in 1.1, 1.2 and 1.3 — do not add a fourth.
- **Tick before saving, in every determinism test.** A save taken at tick 0 hides a
  `from_save` that reseeds the wander stream from the seed and resets every cooldown to its
  spawn value (`id.0 % 10`), because those *are* the tick-0 values.
- **`Wander` is private and `World::dwarves()` cannot see it.** Restoring a dwarf's `pos`
  but not its `home` gives a dwarf a new 3-tile box centred wherever it happened to be
  standing; restoring `pos` but not `cooldown` re-phases its step. Both look perfect in a
  snapshot and diverge within ten ticks.
- **`from_save` must not call `generate()`.** Terrain comes off the save; a world rebuilt
  from the seed silently discards every dug tile — which in this story means silently
  discarding the fortress. The `set_tile`-before-save assertion in AC2 is the guard.
- **One `assemble`, one schedule.** Two hand-wired schedules is the failure mode that a
  green suite cannot see: 3.2 adds a system, wires it into `generate` only, and every save
  loaded thereafter runs a different sim.
- **The seam is the decision, not the parse.** A `save` that is decoded and logged but writes
  nothing, or a `load` that replaces the world but skips the broadcast, passes every parser
  test and every log assertion. AC5's file-decodes assertion and AC6's two-client snapshot
  assertion are the tests that die when the decision is discarded — write them first and
  watch them go red.
- **The logged tick is what makes the load test exact.** A client cannot know which iteration
  consumed its `save`, so timing-based expectations flake. Read `saved tick {t} to ...` off
  stderr and assert the post-load snapshot carries exactly `t`.
- **Save cost lands on one iteration.** Encoding 524k tiles is the same order of work as the
  connect snapshot (measured ~175 ms there), so the tick that handles `S` runs long. That is
  acceptable for an explicit operator action; do not add a worker thread, a channel or an
  async write to hide it. `// NOTE:` it and move on.
- **Speed is not sim state.** It stays out of `SaveState` (AD-11's field list, AD-10's
  split). Loading while paused leaves the loop paused, now sitting on the loaded tick.
- **Atomic rename, not truncate-in-place.** `write .tmp` + `rename` costs two lines and is
  the difference between "the daemon died mid-save" and "the previous save is gone too".
- **Keep the mid-stream snapshot arm's camera.** Adopt the world, keep `state.camera` and
  `state.z`. Calling `initial()` on every load would yank the view back to entity 0.
- **`--frames` is an evidence channel with a history.** 2.2 shipped it re-centring the camera
  per frame and silently colourless under `NO_COLOR`; both were found at review, after the
  story read as done. Do not reintroduce a per-frame `initial()`, keep the `NO_COLOR`
  warning, and keep the no-key mutation-guard test beside the `--key L` one.
- **The daemon's tests must not write into the repo.** `frostvein.save` is
  working-directory-relative; without a per-daemon temp cwd the suite drops save files beside
  `Cargo.toml` and two concurrent tests fight over one file.

### Project Structure (files to touch)

```
Cargo.toml                        # UPDATE — rand_chacha "serde" feature
.gitignore                        # UPDATE — /frostvein.save, /frostvein.save.tmp
crates/sim-core/Cargo.toml        # UPDATE — serde
crates/sim-core/src/save.rs       # NEW    — SaveState, SavedDwarf
crates/sim-core/src/lib.rs        # UPDATE — serde derives, assemble(), to_save/from_save
crates/sim-core/tests/save_load.rs# NEW    — the AD-11 gate test, ids, dirty set
crates/protocol/src/lib.rs        # UPDATE — Save/Load/Quit variants + literal tests
crates/simd/src/main.rs           # UPDATE — SAVE_PATH, save/load/quit arms, broadcast on load
crates/simd/tests/serve.rs        # UPDATE — temp-cwd harness, rewind, file decode, failed load, quit
crates/tui/src/view.rs            # UPDATE — S and L keys + keymap table test
crates/tui/src/main.rs            # UPDATE — --key S|L, live snapshot arm NOTE
crates/tui/tests/client.rs        # UPDATE — the two instrument tests
_bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh  # NEW
```

`crates/simd/src/bridge.rs` should not need changing — `snapshot()` already takes the world
and the speed.

### Previous story intelligence (2.3)

- The daemon-side pattern to copy exactly: command decoded in `read_inbound`, applied in the
  iteration-top drain, proven by a live-daemon test that asserts the *consequence* over
  several real deltas. Parser-level tests proved nothing there and will prove nothing here.
- `scripts/mutate.sh` is not concurrency-safe and rebuilds mutated artifacts; 2.3 hit a stale
  mutated binary on the first manual `cargo run` afterwards and had to
  `cargo clean -p protocol -p simd -p tui` before the final gate. Expect the same.
- The client's write half, `--key` plumbing and the `KeyModifiers::SHIFT` allowance all
  landed in 2.3 — extend them, do not rebuild them.

### Verification

```bash
scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh
```

Live instrument — the observable outcome, joining the two binaries no test can span:

```bash
cargo run -p simd &                                       # save lands in the repo root (gitignored)
cargo run -p tui -- --frames 5  --key S > /tmp/save.txt   # note the daemon's "saved tick N"
cargo run -p tui -- --frames 20         > /tmp/run.txt    # tick climbs well past N
cargo run -p tui -- --frames 20 --key L > /tmp/load.txt
rg -n 'tick [0-9]+' /tmp/load.txt | head -5               # tick jumps back to N, then climbs
ls -l frostvein.save
cargo run -p tui                                          # S, then let it run, then L: the world rewinds on screen
```

Then, in a second terminal against the same daemon, confirm a load from one client rewinds
both. Finish with `printf '{"type":"quit"}\n' | nc 127.0.0.1 7373` (or the raw-TCP
equivalent) and confirm the daemon exits without a panic and the clients report the closed
connection.

Branch: `2-4-the-world-endures`. Commit as `Völundr <jeicei75@gmail.com>`, one commit per
green step, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.4] — user story, source ACs, and
  the `// NOTE:` recording that jobs + claims join `SaveState` at 3.2
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md]
  — AD-11 (SaveState field list, `simd` owns file I/O, snapshot broadcast on load, the
  save → load → tick N gate), AD-9 (allocator next value survives load), AD-8 (construction
  does not dirty-track), AD-10 (control commands handled by `simd` directly), AD-1, AD-6
- [Source: _bmad-output/implementation-artifacts/2-3-master-of-time.md] — the command path
  this story extends, and the instrument's failure history
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — the RNG-stream-coupling
  item, resolved at 2.2 (`STREAM_WORLDGEN`/`STREAM_SPAWN`/`STREAM_WANDER`, verified at
  crates/sim-core/src/lib.rs:17-19), and the `read_inbound` item this story must not touch
- [Source: AGENTS.md] — sabotage rule, honest reporting, bounded I/O, the codex self-gate

## Dev Agent Record

### Agent Model Used

OpenAI GPT-5 Codex

### Debug Log References

- Initial save API RED (`cargo test --offline -p sim-core --test save_load`):
  ```text
  error[E0599]: no associated function or constant named `from_save` found for struct `World`
  error[E0599]: no method named `to_save` found for struct `World` in the current scope
  error: could not compile `sim-core` (test "save_load") due to 6 previous errors
  ```
- Save restoration sabotage REDs (`save_load_then_tick_matches_never_saved` unless noted):
  ```text
  cooldown=0: test save_load_then_tick_matches_never_saved ... FAILED
    left:  [(Id(0), Pos { x: 113, y: 85, z: 15 }, Walk), ...]
    right: [(Id(0), Pos { x: 113, y: 86, z: 15 }, Idle), ...]
  home=position: test save_load_then_tick_matches_never_saved ... FAILED
    left:  [(Id(0), Pos { x: 113, y: 88, z: 15 }, Walk), ...]
    right: [(Id(0), Pos { x: 113, y: 86, z: 15 }, Walk), ...]
  reseed wander RNG: test save_load_then_tick_matches_never_saved ... FAILED
    left dwarf 4:  Pos { x: 101, y: 123, z: 17 }
    right dwarf 4: Pos { x: 101, y: 121, z: 17 }
  loaded schedule omits wander: test save_load_then_tick_matches_never_saved ... FAILED
    left dwarf 4:  Pos { x: 101, y: 122, z: 17 }, Idle
    right dwarf 4: Pos { x: 101, y: 121, z: 17 }, Walk
  regenerated terrain: assertion `left == right` failed
    left: Some(Solid(Stone))
    right: Some(Empty)
  tick=0: assertion `left == right` failed
    left: 1
    right: 38
  loading_does_not_reuse_entity_ids with allocator reset: FAILED
    left: 0
    right: 5
  loading_starts_with_no_dirty_tiles with a restored dirty entry: FAILED
    assertion failed: loaded.drain_dirty().is_empty()
  save_orders_dwarves_by_id with sorting removed: FAILED
    left: [4, 3, 2, 1, 0]
    right: [0, 1, 2, 3, 4]
  ```
- Protocol command API RED (`cargo test --offline -p protocol
  decodes_and_reencodes_the_documented_command_wire_format`):
  ```text
  error[E0599]: no variant, associated function, or constant named `Save` found for enum `Command`
  error[E0599]: no variant, associated function, or constant named `Load` found for enum `Command`
  error[E0599]: no variant, associated function, or constant named `Quit` found for enum `Command`
  error: could not compile `protocol` (lib test) due to 6 previous errors
  ```
- Wire-discriminator sabotage REDs:
  ```text
  Save renamed to store: unknown variant `save`, expected one of `set_speed`, `store`, `load`, `quit`
  Load renamed to restore: unknown variant `load`, expected one of `set_speed`, `save`, `restore`, `quit`
  Quit renamed to exit: unknown variant `quit`, expected one of `set_speed`, `save`, `load`, `exit`
  test result: FAILED. 0 passed; 1 failed
  ```
- Daemon behavior RED and decision-seam sabotage:
  ```text
  decoded-but-ignored Save: saved_file_decodes_as_a_save_state ... FAILED
    daemon logged nothing within 10s
  Load without snapshot broadcast: save_then_load_rewinds_every_client ... FAILED
    daemon did not broadcast a snapshot within four lines
  ignored Quit: quit_exits_the_daemon_cleanly ... FAILED
    daemon logged nothing within 10s
  panicking corrupt-load path: undecodable_save_is_logged_and_the_daemon_keeps_ticking ... FAILED
    unexpected corrupt-save log
  panicking unwritable-save path: unwritable_save_is_logged_and_the_daemon_keeps_ticking ... FAILED
    unexpected save error log
  SAVE_PATH changed to other.save: saved_file_decodes_as_a_save_state ... FAILED
    saved file must exist: Os { code: 2, kind: NotFound, message: "No such file or directory" }
  MAX_SAVE_BYTES widened 16 MiB -> 17 MiB: oversized_save_is_logged_and_the_daemon_keeps_ticking ... FAILED
    unexpected oversized-save log: ... EOF while parsing a value at line 1 column 16777217
  ```

### Completion Notes List

- Enabled `ChaCha8Rng` serde support, added serde derive support to `sim-core`, and ignored
  the daemon's working-directory save artifacts. `cargo check --offline -p sim-core` passed.
- Added the exact `SaveState`/`SavedDwarf` state, a shared world assembly path, and faithful
  save/load restoration. The public-API 200-tick scenario, allocator, clean-dirty-set, and
  stable dwarf ordering tests pass; every hidden restoration seam was sabotaged and went RED.
- Added the literal `save`, `load`, and `quit` command discriminators plus explicit `store`
  rejection. All three discriminator mappings were independently sabotaged and went RED.
- Added bounded atomic daemon saves, bounded loads, authoritative load snapshots to every
  client, and clean quit. A hermetic temp-cwd harness proves rewind, file decode, missing,
  corrupt, oversized and unwritable failures, continued ticking, EOF, and exit 0. All 23
  live-daemon tests and `simd` clippy pass.

### File List

- .gitignore
- Cargo.lock
- Cargo.toml
- _bmad-output/implementation-artifacts/2-4-the-world-endures.md
- crates/sim-core/Cargo.toml
- crates/sim-core/src/lib.rs
- crates/sim-core/src/save.rs
- crates/sim-core/tests/save_load.rs
- crates/protocol/src/lib.rs
- crates/simd/src/main.rs
- crates/simd/tests/serve.rs

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-04 | Story created |
