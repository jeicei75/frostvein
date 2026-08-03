---
baseline_commit: 0882d0641f8aba67281d1d87817ba7423643a0f9
---

# Story 2.1: The World Runs on Its Own Clock

Status: ready-for-dev

## Story

As the boss,
I want the daemon to advance the sim on a fixed timestep and stream every tick,
so that the world exists and moves independent of anyone watching.

## Acceptance Criteria

1. `sim-core` owns exactly one `bevy_ecs::schedule::Schedule`, built with an explicit
   `.chain()`, and `World::step()` runs it. `tick` is an ECS resource advanced by a system
   *inside* that schedule; `World::tick()` reports N after N `step()` calls.
2. `World::set_tile(pos, tile)` replaces that tile and records `pos` in a per-tick dirty
   set. An out-of-bounds `pos` mutates nothing and returns `false`. `tiles` has no other
   mutator.
3. `World::drain_dirty()` returns `Vec<(Pos, Tile)>` in ascending `Pos` order and empties
   the set, so one change is never reported twice.
4. A `sim-core` scenario test that calls `set_tile` then `step()` finds exactly that tile
   in the drained dirty set, and finds it empty after the next `step()`.
5. `protocol` gains `MessageType::Delta`, `TileChange { pos, tile }`, and
   `Delta { type, tick, tiles, entities, designations, zones, speed }`. Neither `simd` nor
   `tui` declares a delta shape of its own.
6. Per AD-8, `Delta.tiles` carries **only** tiles dirtied that iteration, while `entities`,
   `designations`, `zones`, `speed` and `tick` are full authoritative resends every
   iteration.
7. `simd` runs a fixed 10 ticks/sec timestep that steps the world and emits exactly one
   `delta` line per iteration to every connected client, and it keeps advancing with zero
   clients attached.
8. A client connecting mid-run receives exactly one `snapshot` encoded from the world *at
   connect*, then one `delta` per iteration thereafter. Its snapshot `tick` equals the
   sim's tick at that moment, not `0`.
9. A client whose bounded delta queue overflows, or whose socket write fails, is
   disconnected and removed from the registry. The daemon and every other client keep
   running.
10. `tui` applies deltas from a dedicated reader thread while staying responsive to keys,
    and its status line shows the tick climbing live.
11. `scripts/gate.sh` passes.

## Tasks / Subtasks

- [ ] **Schedule + tick as sim state** (AC: 1)
  - [ ] `#[derive(bevy_ecs::resource::Resource)] struct Tick(pub u64);` inserted in
        `generate`. Delete the plain `tick: u64` field — `World::tick()` reads the resource.
  - [ ] `fn advance_tick(mut t: ResMut<Tick>) { t.0 += 1; }`, registered as
        `schedule.add_systems((advance_tick,).chain())`. Keep the tuple + `.chain()` even
        with one system: AD-7 requires the ordering to be explicit, and 2.2 appends to it.
  - [ ] `World` gains `schedule: Schedule` (use `Schedule::default()`; we never insert it
        into a `Schedules` resource, so the label does not matter). `pub fn step(&mut self)`
        calls `self.schedule.run(&mut self.ecs)` — disjoint field borrows, this compiles.
  - [ ] Remove the `// NOTE: tick advancement lands in Story 2.1` at
        `crates/sim-core/src/lib.rs:120`.
- [ ] **`set_tile` + dirty set** (AC: 2, 3, 4)
  - [ ] `dirty: BTreeSet<Pos>` on `World`. **`BTreeSet`, not `HashSet`** — AD-7 forbids
        unordered iteration reaching a sim outcome, and this one reaches the wire.
  - [ ] `pub fn set_tile(&mut self, p: Pos, t: Tile) -> bool` — bounds-check via the same
        path as `tile()`, write, insert into `dirty`, return `true`; `false` and no
        mutation when out of bounds.
  - [ ] `pub fn drain_dirty(&mut self) -> Vec<(Pos, Tile)>` — ascending `Pos` order
        (`BTreeSet` iteration already gives this), clearing as it goes.
  - [ ] Scenario test `set_tile_shows_up_once_in_the_dirty_set` per AC4. **This is the only
        producer the dirty path has until Story 3.2's dig** — see Key decisions.
- [ ] **Delta wire types** (AC: 5, 6)
  - [ ] `protocol`: add `Delta` variant to `MessageType`; add `TileChange` and `Delta` per
        the skeleton. Keep `type` a plain first field backed by `MessageType` — **never**
        `#[serde(tag = "type")]`, for the reason in `crates/protocol/src/lib.rs`.
  - [ ] Test decoding a **hand-written JSON literal** of a delta, as 1.2 did for the
        snapshot. A symmetric rename passes a round-trip and breaks every client.
- [ ] **`simd`: tick loop, client registry, broadcast** (AC: 7, 8, 9)
  - [ ] Move the accept loop to its own thread handing `TcpStream`s to the tick loop over
        an `mpsc::channel`. **The tick loop owns the `World` outright** — no `Mutex`, no
        sharing.
  - [ ] Per iteration: admit new clients (encode a snapshot from the *current* world for
        each), `world.step()`, encode one delta, `try_send` it to every client, drop those
        that fail, then sleep to the deadline.
  - [ ] Registry entry holds a **bounded** `SyncSender<Arc<String>>` (`CLIENT_QUEUE = 16`).
        `TrySendError::Full` → the client is too slow, disconnect it; `Disconnected` → its
        thread already died, remove it. Unbounded queueing here would reintroduce exactly
        the class 1.2's review made us fix.
  - [ ] Connection thread: drain its receiver and `write_all` each line; keep the existing
        read-and-drop of inbound lines and all 1.2 bounds (`MAX_LINE_BYTES`,
        `WRITE_TIMEOUT`, `MAX_CONNECTIONS`, `ACCEPT_BACKOFF`, `ConnectionGuard`).
  - [ ] **Read EOF must no longer close the connection** — now that a write path exists,
        a half-closing client keeps receiving deltas. Replace the 1.2 `// NOTE:` at
        `crates/simd/src/main.rs` that hands this to 2.1; the thread now ends when the
        *write* fails or the channel closes.
  - [ ] Delete the encode-once `Arc<String>` snapshot: the world is no longer static.
  - [ ] e2e in `crates/simd/tests/serve.rs`: assert three consecutive deltas arrive with
        strictly increasing `tick`; assert a client connecting later gets a snapshot whose
        `tick > 0`; assert one client dropped mid-stream leaves a second client still
        receiving.
- [ ] **`tui`: reader thread + live tick** (AC: 10)
  - [ ] Spawn a reader thread owning the `BufReader`; it does blocking `read_line`, decodes
        into an `enum Msg { Snapshot(Box<Snapshot>), Delta(Box<Delta>) }`, sends on an
        `mpsc::channel`, and exits on EOF or decode error.
  - [ ] Main loop: `if event::poll(POLL_INTERVAL)? { … }`, then drain the channel with
        `try_recv()`, then redraw only if a key or a message changed something.
  - [ ] `fn apply(snapshot: &mut Snapshot, delta: Delta)` — write each `TileChange` into
        `tiles`, then replace `entities`, `designations`, `zones`, `speed`, `tick`
        wholesale. Keep `Snapshot` as the client's world model; do not invent a second type.
  - [ ] Status line gains the tick. Extend the existing pinned status-line assertion in
        `crates/tui/src/view.rs` rather than loosening it.
  - [ ] Keep the snapshot read timeout for the *first* line only; the reader thread blocks
        indefinitely afterwards by design.
- [ ] **Green gate** (AC: 11) — `scripts/gate.sh`, then the live check below; report what it
      printed.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No commands upstream, in either direction.** No `set_speed`, no pause, no `Space`/`+`/`-`
  keys — Story 2.3 owns the entire command path and `protocol` gains no command type here.
  The client still sends zero bytes.
- **No wander, no job state, no `JobState` component, no second RNG stream.** Story 2.2 owns
  those, and it is 2.2 that splits `STREAM_WORLDGEN` into purpose-named streams.
- **No change to `protocol::Entity`.** 2.2 adds the job-state field; adding it now would
  break 1.2's pinned JSON literals for no gain in this story.
- **No `SaveState`, no pathfinding, no designations/zones shapes** (they stay `Vec<()>`).
- **No reconnect logic in `tui`** — if the reader thread hits EOF, the client exits with an
  `anyhow` error. Reconnect is not in any Epic 2 story.
- **Do not move `tiles` into the ECS.** Only `tick` becomes a resource here; see below.

### What already exists (build on it, do not re-derive)

- `World` = `dims, tiles, ecs: EcsWorld, ids, seed, tick` with `generate/dims/seed/tick/
  tiles/tile/dwarves` [crates/sim-core/src/lib.rs:69-155]. **No `set_tile`, no `Schedule`,
  no systems, no retained RNG** — the `ChaCha8Rng` is a local in `generate` and is dropped.
- `simd` encodes the snapshot **once** into a shared `Arc<String>` and `serve()` writes it
  then reads-and-drops forever [crates/simd/src/main.rs:55,93]. All 1.2 bounds are in place
  and must survive. `bridge::snapshot(&World) -> protocol::Snapshot` is reusable as-is.
- `tui` blocks on `event::read()` at [crates/tui/src/main.rs:121] and holds one immutable
  `Snapshot`. `view::render(&Snapshot, &ViewState, w, h)` and `apply_key` are unchanged by
  this story beyond the status line.
- `protocol::MessageType` has the single variant `Snapshot`; `Speed` exists and is unused
  until 2.3.

### Verified API shapes (compile-probed against the vendored bevy_ecs 0.19.0, 2026-08-03)

```rust
use bevy_ecs::{resource::Resource, system::ResMut, schedule::{IntoScheduleConfigs, Schedule}};

#[derive(Resource)]
pub struct Tick(pub u64);                       // Resource: Component — the derive covers both

fn advance_tick(mut t: ResMut<Tick>) { t.0 += 1; }

let mut schedule = Schedule::default();
schedule.add_systems((advance_tick,).chain());  // trait is IntoScheduleConfigs, NOT IntoSystemConfigs
schedule.run(&mut ecs);                         // ecs: bevy_ecs::world::World
let n = ecs.resource::<Tick>().0;
```

The single-threaded executor is already the default (`multi_threaded` is off, and must stay
off — AD-7). `.chain()` is still required: it makes the *order* explicit rather than
graph-incidental.

```rust
// crates/protocol/src/lib.rs — additions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileChange { pub pos: [i32; 3], pub tile: Tile }

/// One per loop iteration (AD-8). `tiles` is the dirty set; everything else is a
/// full authoritative resend — absence is deletion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub tick: u64,
    pub tiles: Vec<TileChange>,
    pub entities: Vec<Entity>,
    pub designations: Vec<()>,
    pub zones: Vec<()>,
    pub speed: Speed,
}
```

```rust
// crates/simd/src/main.rs — structure, not the whole file
const TICK_PERIOD: Duration = Duration::from_millis(100);   // 10 Hz (FR13)
const CLIENT_QUEUE: usize = 16;

loop {
    let deadline = Instant::now() + TICK_PERIOD;
    for stream in new_rx.try_iter() { /* register + send this client its snapshot */ }
    world.step();
    let line = Arc::new(format!("{}\n", serde_json::to_string(&bridge::delta(&mut world))?));
    clients.retain(|c| c.tx.try_send(Arc::clone(&line)).is_ok());
    thread::sleep(deadline.saturating_duration_since(Instant::now()));
}
```

### Key decisions & traps

- **The TUI must use a reader thread, not a socket read timeout.** `BufRead::read_line`
  documents its buffer contents as *unspecified* when it returns an error, so a read that
  times out mid-line cannot be safely resumed — timeout-driven line framing is unsound.
  A dedicated blocking reader plus `event::poll` for keys avoids the problem entirely.
- **Dirty tiles have no gameplay producer until Story 3.2.** 2.2's wandering moves
  entities, which are full-resend. Wolf's decision (2026-08-03): build the mechanism here
  per AD-8 **and prove it with AC4's direct `set_tile` test**, so it ships exercised rather
  than as dead code upheld by inspection. Do not skip AC4 because "nothing calls it yet" —
  AC4 *is* the caller.
- **The tick loop owns the `World`; nothing else touches it.** New connections arrive as
  raw sockets over a channel and the loop encodes their snapshot itself. This is why no
  `Mutex<World>` appears anywhere — do not add one.
- **Bound the per-client queue.** A slow client must be dropped, never buffered without
  limit. `try_send` + `retain` is the whole mechanism.
- **`Instant`/`sleep` live in `simd` only.** `sim-core` has zero I/O including the clock
  (AD-1); it never learns the wall-clock rate (AD-2).
- **Use `saturating_duration_since` for the sleep.** A tick that overruns its budget must
  not panic on a negative duration; it just runs the next one immediately.
- **`World::tick()` keeps its signature.** `bridge::snapshot` already reads it, and AD-1
  says `simd` must never fabricate the tick — that mutation finally becomes killable here,
  so make sure a test would fail if `bridge` hardcoded `0`.

### Project Structure (files to touch)

```
crates/sim-core/src/lib.rs        # UPDATE — Tick resource, Schedule, step, set_tile, drain_dirty
crates/sim-core/tests/scenario.rs # NEW    — AC4 dirty-set scenario test
crates/protocol/src/lib.rs        # UPDATE — MessageType::Delta, TileChange, Delta (+ literal test)
crates/simd/src/bridge.rs         # UPDATE — add `delta(&mut World) -> protocol::Delta`
crates/simd/src/main.rs           # UPDATE — accept thread, tick loop, client registry, broadcast
crates/simd/tests/serve.rs        # UPDATE — delta stream, mid-run connect, one-client-drop
crates/tui/src/main.rs            # UPDATE — reader thread, poll loop, apply()
crates/tui/src/view.rs            # UPDATE — tick in the status line (extend the pinned test)
```

### Previous story intelligence (1.3)

- Every mapping/pinning test is verified **by sabotage** — break the code, watch the named
  test go red, restore, and paste the failure output into the Dev Agent Record. This is now
  a standing rule in `AGENTS.md`; 1.3 is the story where doing it produced clean mapping
  tests on the first review pass.
- Sabotage the **constants** too: 1.3 shipped a green suite in which widening `PEEK_DEPTH`
  3→6 changed nothing. Here that means `TICK_PERIOD` and `CLIENT_QUEUE`.
- The sandbox now has network access, so you can run the loopback tests yourself — the
  blocker that stopped 1.3's gate is gone. Still build `--offline`.

### Verification

```bash
scripts/gate.sh
```

Live check (AC7, AC8, AC10 — the observable outcome, not just green tests):

```bash
cargo run -p simd &                        # tick loop starts with no client attached
cargo run -p tui                           # status line tick climbs ~10/sec; < > and panning still work
cargo run -p tui                           # a SECOND client mid-run: snapshot tick > 0, then it tracks
```

Branch: `2-1-the-world-runs-on-its-own-clock`. Commit as `Völundr <jeicei75@gmail.com>`,
one commit per green step, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.1] — user story, source ACs, and
  the dependency-sweep `// NOTE:` recording Wolf's dirty-tile decision
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md]
  — AD-1 (pure core, no clock), AD-2 (loop never stops), AD-3/AD-4 (NDJSON, one message per
  line), AD-7 (single-threaded `.chain()`ed schedule, no unordered iteration), AD-8 (deltas =
  dirty tiles + all small state in full; `set_tile` records the dirty set), AD-9 (u32 ids)
- [Source: _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/prd.md] — FR13, FR17,
  FR19, NFR2, NFR3
- [Source: _bmad-output/implementation-artifacts/1-2-the-daemon-serves-the-world.md#Review Findings]
  — the daemon bounds this story must preserve, and the half-close `// NOTE:` handed to 2.1
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — the RNG-stream deferral
  now corrected to Story 2.2, not this one
- [Source: crates/sim-core/src/lib.rs:69-155, crates/simd/src/main.rs:38-138,
  crates/tui/src/main.rs:69-138] — the code this story updates
- [Source: AGENTS.md] — standing dev-agent rules, including sabotage-verification

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-03 | Story created |
