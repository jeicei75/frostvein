---
baseline_commit: 7362850
---

# Story 2.4: The World Endures

Status: done

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

- [x] **`tui`: two keys and a live snapshot arm** (AC: 9, 10)
  - [x] `apply_key`: `KeyCode::Char('S')` → `Action::Command(Command::Save)`,
        `KeyCode::Char('L')` → `Action::Command(Command::Load)`. Uppercase arrives with SHIFT
        and `apply_key` already lets SHIFT through, so no modifier change. Extend the pinned
        keymap table test with both, asserted at all three speeds to pin that they do not
        depend on speed. `q` and the confirm path are untouched.
  - [x] Delete the "this arm is unreachable today" half of the `Msg::Snapshot` NOTE in
        `main`'s message loop — the load broadcast makes it live — and keep the rest: adopt
        the world, **keep** `state.camera` and `state.z`. Never call `initial()` there; that
        would throw away where the player was looking on every load.
  - [x] Status line and hint text are unchanged. FR21's hint bar is Story 3.1's, and the
        80-column budget pinned by 2.3's width test has no room for a save/load hint. The
        observable for `S` is the daemon's log line and the file; for `L` it is the tick
        jumping back on screen.

- [x] **Observability instrument** (AC: 11) — extend `tui --frames N --key`, do not invent a
      second channel. `--key` accepts `S` and `L` alongside `space`/`+`/`-` (update the parse
      arm and its error text).
  - [x] Two tests in `crates/tui/tests/client.rs`, real binary against a stub daemon that
        streams climbing deltas and, on reading a `{"type":"load"}` line, sends a snapshot at
        an earlier tick and then resumes climbing from it: with `--key L` the captured ticks
        jump back to that tick; with no `--key` they climb monotonically and never jump back.
        The second is the mutation guard on the first — an instrument that rendered a stale
        first snapshot forever would satisfy the first alone.
  - [x] This is also the only automated proof of AC10: `stream_frames` adopting a mid-stream
        snapshot is the same arm the interactive loop uses.

- [x] **Sabotage + mutation set** (AC: 12)
  - [x] `_bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh`, at least:
        `from_save` drops the wander cooldown; drops the wander home; reseeds the wander RNG
        from the seed instead of restoring it; `assemble` wires only `advance_tick` for the
        loaded world; `from_save` regenerates terrain from the seed instead of restoring
        tiles; `from_save` resets the id allocator; `from_save` marks every tile dirty;
        `to_save` records tick 0; the daemon's `Save` arm parses but writes nothing; the
        `Load` arm replaces the world but skips the snapshot broadcast; the `Quit` arm is
        ignored; `S` maps to `Load`; the `--key` path never writes its command; the `save`
        discriminator is renamed. Run `scripts/mutate.sh` and paste the table.
  - [x] Paste the actual RED output for every new mapping/constant test into the Dev Agent
        Record (AGENTS.md rule 1).

- [x] **Green gate** (AC: 12) — `scripts/gate.sh`, then the live check. Report what printed.

### Review Findings

- [x] [Review][Decision] **A save with duplicate dwarf ids is silently accepted and served forever** —
      `load_world` validates dims/tile-count, `MAX_LOAD_TICK` and dwarf pos/home bounds, but never
      that `save.dwarves` ids are unique; `World::from_save` spawns one entity per `SavedDwarf`
      with no collision check. Reproduced independently three times (Edge Case Hunter live, Blind
      Hunter by inspection, and by the reviewer): a save with `dwarves[1].id` rewritten to match
      `dwarves[0]` loads clean and every subsequent snapshot and delta carries ids
      `[0, 0, 2, 3, 4]` — no rejection log, no panic, indefinitely. Harmless today because `Id` is
      never a map key and `next_id` is never consulted after load, so this is a semantic gap, not
      a crash. The decision is scope, not mechanism: the fix is ~5 lines mirroring the checks
      already beside it, but those checks are themselves beyond-spec, and YAGNI is policy here
      [crates/simd/src/main.rs:259-284, crates/sim-core/src/lib.rs:313-344].
- [x] [Review][Patch] Pinned keymap table asserts `S`/`L` with `KeyModifiers::NONE`, but a real
      terminal only ever delivers uppercase with SHIFT — tighten the modifier gate and every test
      stays green while the feature dies silently [crates/tui/src/view.rs:259,317-342]
- [x] [Review][Patch] `--key S` has no automated test; only `--key L` and the no-key guard are
      covered, though AC11 names both [crates/tui/tests/client.rs:218]
- [x] [Review][Patch] Two unreachable conditions in `in_bounds` (`pos.x < i32::MAX`,
      `pos.y < i32::MAX`) with no matching `pos.z` guard — the dims comparison below already
      rejects everything they would [crates/simd/src/main.rs:249-258]
- [x] [Review][Patch] `to_save`'s `filter_map` uses `?` on each component get, so a `Dwarf` entity
      missing `Id`/`Pos`/`JobState`/`Wander` vanishes from the save with no diagnostic. Unreachable
      today (both construction sites attach the full set); wants a `// NOTE:` naming the
      precondition rather than a guard [crates/sim-core/src/lib.rs:283-299]
- [x] [Review][Defer] No in-UI affordance for `Command::Quit` — deferred to Story 3.1. Correct per
      AC9 (a shared daemon must not die from one viewer's keypress), but the status line advertises
      only `q quit` and `q` leaves the daemon running with no in-client way to stop it
      [crates/tui/src/view.rs:222]
- [x] [Review][Defer] `MAX_SAVE_BYTES` (16 MiB) and `Dims::DEFAULT` can drift apart — deferred.
      The live save is 6.9 MB, 2.4x headroom; grow the default world past that and `save_world`
      refuses every save. The suite does catch it (`saved_file_decodes_as_a_save_state` fails on a
      missing file) but with a message that never mentions the cap [crates/simd/src/main.rs:24]

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
  tile-count validation absent: inconsistent_save_is_logged_and_the_daemon_keeps_ticking ... FAILED
    unexpected inconsistent-save log:
  supported-tick-range validation absent: boundary_tick_save_is_logged_and_the_daemon_keeps_ticking ... FAILED
    unexpected boundary-tick log:
  dwarf-position validation absent: out_of_bounds_dwarf_save_is_logged_and_the_daemon_keeps_ticking ... FAILED
    unexpected out-of-bounds dwarf log:
  dwarf-home validation absent: out_of_bounds_dwarf_home_is_logged_and_the_daemon_keeps_ticking ... FAILED
    unexpected out-of-bounds dwarf-home log:
  ```
- TUI keymap RED and mapping sabotage:
  ```text
  before implementation: wrong action for Char('S') at Paused
    left: Ignore
    right: Command(Save)
  S mapped to Load: wrong action for Char('S') at Paused
    left: Command(Load)
    right: Command(Save)
  L mapped to Save: wrong action for Char('L') at Paused
    left: Command(Save)
    right: Command(Load)
  test result: FAILED. 0 passed; 1 failed
  ```
- Observability instrument RED and sabotage:
  ```text
  before --key L parsing: key_l_rewinds_captured_ticks_then_they_climb_from_the_saved_tick ... FAILED
    tui did not connect to stub daemon within 3s
  command write removed: key_l_rewinds_captured_ticks_then_they_climb_from_the_saved_tick ... FAILED
    read load command: Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }
  mid-stream snapshot ignored: assertion `left == right` failed
    left: [8, 8, 4, 5]
    right: [8, 3, 4, 5]
  CLI L mapped through S: stub command assertion failed
    left: Save
    right: Load
  streamed deltas ignored (no-key mutation guard): assertion `left == right` failed
    left: [7, 7, 7, 7]
    right: [8, 9, 10, 11]
  ```
- Mutation run (`scripts/mutate.sh
  _bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh`):
  ```text
  ================ MUTATION RESULTS ================
  from_save drops the wander cooldown                          KILLED
  from_save drops the wander home                              KILLED
  from_save reseeds the wander RNG                             KILLED
  loaded world schedule omits wander                           KILLED
  from_save regenerates terrain from the seed                  KILLED
  from_save resets the id allocator                            KILLED
  from_save marks every tile dirty                             KILLED
  to_save records tick zero                                    KILLED
  to_save leaves dwarves in ECS order                          KILLED
  daemon Save arm parses but writes nothing                    KILLED
  daemon Load arm skips snapshot broadcast                     KILLED
  daemon Quit arm is ignored                                   KILLED
  daemon save path is renamed                                  KILLED
  daemon save read limit is widened                            KILLED
  failed load panics the daemon                                KILLED
  load skips tile-count validation                             KILLED
  load widens the supported tick range                         KILLED
  load accepts an out-of-bounds dwarf                          KILLED
  load accepts an out-of-bounds dwarf home                     KILLED
  S maps to Load                                               KILLED
  L maps to Save                                               KILLED
  frames key path never writes its command                     KILLED
  frames instrument ignores load snapshots                     KILLED
  frames instrument ignores deltas                             KILLED
  save discriminator is renamed                                KILLED
  load discriminator is renamed                                KILLED
  quit discriminator is renamed                                KILLED

  All mutations killed.
  ```
- Final clean gate after mutation artifacts were removed for all four packages:
  ```text
  Removed 4633 files, 1.4GiB total
  frostvein gate
    cargo fmt --check           ok
    cargo clippy -D warnings    ok
    cargo test                  ok
    tui has no sim-core edge    ok
  GATE GREEN
  ```
- Manual live instrument observation (real `simd` + real `tui` binaries):
  ```text
  --frames 5 --key S: 110, 111, 112, 113, 114
  daemon log: saved tick 114 to frostvein.save
  --frames 20 (no key): 284 through 303
  --frames 20 --key L: 354, 355, 356, 357, 358, 114, 115 ... 128
  frostvein.save: 6,910,454 bytes
  two bounded raw clients: before [435, 435], load snapshots [114, 114]
  raw quit client: EOF after 6,910,881 buffered bytes
  daemon log: shutting down on client quit
  daemon process exit: 0
  ```
- Manual full-screen interactive TUI (`cargo run -p tui`, physical S/L keypresses) was not
  observed because this handoff terminal is non-interactive. The real binary headless key
  path and two-client raw path above were observed; the full-screen visual step remains
  explicitly unobserved.

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
  corrupt, inconsistent, boundary-scalar, out-of-bounds entity, oversized and unwritable
  failures, continued ticking, EOF, and exit 0. All 27 live-daemon tests and `simd` clippy pass.
- Added speed-independent uppercase `S`/`L` actions while preserving local `q` behavior,
  camera and z-level across live snapshots, and the unchanged 80-column status line. All
  TUI unit/integration tests and clippy pass.
- Extended `--frames --key` through the real keymap and bounded write half for `S`/`L`.
  The real-binary stub capture proves `[8, 3, 4, 5]` after load versus monotonic
  `[8, 9, 10, 11]` without a key. All 8 client integration tests pass.
- Authored and ran 27 required/expanded mutations; the final serial run reported zero
  survivors and no apply failures.
- Ran the clean final gate successfully and manually observed save, rewind, two-client
  broadcast, save-file creation, wire quit, client EOF, and daemon exit 0. The full-screen
  interactive rendering step was not observed in this non-interactive handoff.
- Ran `codex review --base main`; it identified malformed-but-decodable save dimensions as
  a daemon panic path, which is now rejected by checked tile-count validation with a
  red-then-green integration test and killed mutation. Its unbounded command-backlog finding
  was not changed because the story explicitly excludes backpressure work and the channel
  predates this story.
- Re-ran review after that patch; it identified maximum ticks and extreme dwarf coordinates
  as two remaining post-load panic paths. Maximum-tick, dwarf-position, and dwarf-home guards
  now reject those files while retaining the old world, each with red-then-green live-daemon
  evidence and an independently killed mutation.
- The next review tightened the tick finding to include near-overflow values. Loads now cap
  the supported range at `u64::MAX / 2`, leaving roughly 29 billion years of 10 Hz headroom;
  widening that boundary is independently killed by the boundary-tick test.

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
- crates/tui/src/main.rs
- crates/tui/src/view.rs
- crates/tui/tests/client.rs
- _bmad-output/implementation-artifacts/mutations/2-4-the-world-endures.sh
- _bmad-output/implementation-artifacts/deferred-work.md

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-04 | Story created |
| 2026-08-04 | Implemented deterministic save/load, daemon lifecycle commands, TUI controls, live rewind instrumentation, and zero-survivor mutation coverage. |
| 2026-08-04 | Rejected inconsistent save dimensions after adversarial review and extended the mutation proof to 24 killed cases. |
| 2026-08-04 | Rejected boundary ticks and out-of-bounds dwarf state after final review and extended the mutation proof to 27 killed cases. |
| 2026-08-04 | Capped loaded ticks to a safe operational range after review reproduced a near-overflow crash. |
| 2026-08-04 | Applied code review: reject duplicate dwarf ids on load, pin the SHIFT keymap path, add a `--key S` instrument test, cut two unreachable bounds conditions, note `to_save`'s component precondition. |
| 2026-08-04 | Fixed an intermittently red gate: the test harness reserved a port and dropped it before the daemon could bind, losing the race about one run in four. |
