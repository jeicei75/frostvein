---
baseline_commit: 5675630825df557cb829b1427ad4d54075f38ba3
---

# Story 1.2: The Daemon Serves the World

Status: review

## Story

As a player,
I want the daemon to serve the generated world over protocol v0,
so that any client can receive the full world state on connect.

## Acceptance Criteria

1. Every wire shape lives in `protocol` and nowhere else: `Snapshot`, `Dims`, `Tile`, `Material`, `Entity`, `EntityKind`, `Speed`, `MessageType` are serde types with no logic and no I/O. Neither `simd` nor `tui` declares a wire struct of its own.
2. A client connecting to the daemon over localhost TCP receives exactly one `\n`-terminated JSON line that deserializes into `protocol::Snapshot`, and receives no further bytes while it stays connected.
3. The snapshot mirrors the generated world: `dims` = 128×128×32; `tiles.len() == 524_288` in flat row-major order so `tiles[x + y*128 + z*128*128]` is the tile at `(x, y, z)`; `entities` = the 5 dwarves in ascending id with their positions; `designations` and `zones` are `[]`; `speed` is `normal`; `tick` is `0`.
4. The serialized JSON obeys the wire conventions: top-level `"type": "snapshot"`, snake_case enum values (`"empty"`, `{"solid":"stone"}`, `"dwarf"`, `"normal"`), positions as `[x, y, z]` arrays, entity ids u32, tick u64. No closed vocabulary is carried as a free-form string field.
5. `simd` converts `sim_core::Material` and `sim_core::Tile` to their `protocol` mirrors with exhaustive `match` and no wildcard arm.
6. A client line that is not valid JSON, or is JSON the daemon does not recognize, is logged to stderr and dropped; the daemon keeps running and serves a correct snapshot to a subsequent connection.
7. A client that disconnects — before reading, mid-snapshot, or after — never panics or terminates the daemon; a later connection is still served.
8. `simd` prints `listening on 127.0.0.1:<port>` to stdout after binding. The port is `protocol::DEFAULT_PORT` unless an optional first CLI argument overrides it; `0` means OS-assigned and the printed line reports the actual port.
9. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` pass, and `cargo tree -p tui | rg sim-core` still returns nothing.

## Tasks / Subtasks

- [x] **Add the three stack dependencies** (AC: 1)
  - [x] Root `Cargo.toml` `[workspace.dependencies]`: `serde = { version = "1.0.229", features = ["derive"] }`, `serde_json = "1.0.151"`, `anyhow = "1.0.104"`.
  - [x] `crates/protocol/Cargo.toml`: `serde.workspace = true` (its only dependency). `crates/simd/Cargo.toml`: add `serde_json.workspace = true`, `anyhow.workspace = true`. `tui` gains nothing in this story.
  - [x] Run `cargo fetch` **with network** before any offline work — `serde 1.0.229` is already in `Cargo.lock` and the local registry cache (bevy_ecs pulls it), but `serde_json` and `anyhow` are in neither.
- [x] **Wire types in `protocol`** (AC: 1, 3, 4)
  - [x] Replace the port-only `lib.rs` with the skeleton below, keeping `DEFAULT_PORT` and `#![forbid(unsafe_code)]`.
  - [x] `type` is a plain first field of `Snapshot` backed by the `MessageType` enum — **not** `#[serde(tag = "type")]`. Internally-tagged containers buffer the whole map through `serde::Content` on deserialize, which would balloon the 524 k-element tile array in the client.
- [x] **`tick` on `sim_core::World`** (AC: 3)
  - [x] Add a private `tick: u64` field initialised to `0` in `generate`, plus `pub fn tick(&self) -> u64`. `// NOTE:` that advancement lands in Story 2.1. Tick is sim state; `simd` must not fabricate it (AD-1).
- [x] **Bridge in `simd`** (AC: 3, 5)
  - [x] New `crates/simd/src/bridge.rs` with `pub fn snapshot(world: &sim_core::World) -> protocol::Snapshot` plus private `tile` and `material` converters, all exhaustive `match`, no `_ =>` arm.
  - [x] Entities come from `world.dwarves()` (already ascending by `Id`, AD-7) mapped to `protocol::Entity { id: id.0, kind: EntityKind::Dwarf, pos: [p.x, p.y, p.z] }`.
  - [x] `designations`/`zones` = `Vec::new()`, `speed` = `Speed::Normal`, `tick` = `world.tick()`.
- [x] **TCP server in `simd/src/main.rs`** (AC: 2, 6, 7, 8)
  - [x] Replace the 1.1 smoke print entirely. Parse the optional port arg, generate the world from `SEED`, encode the snapshot line **once** into an `Arc<String>`, bind, print the listening line, then accept forever.
  - [x] One `thread::spawn` per accepted connection. `write_all` the shared line; on write error log to stderr and return. Then read lines from the same stream and log-and-drop each one.
  - [x] All logging goes to **stderr** (`eprintln!`); stdout carries only the listening line so the test harness can parse it. No `log`/`tracing` crate — the stack is closed.
  - [x] An `Err` from `accept` is logged and the loop continues; it never propagates out of `main`.
- [x] **Bridge unit tests** — `#[cfg(test)] mod tests` inside `crates/simd/src/bridge.rs` (AC: 3, 4, 5)
  - [x] `snapshot_mirrors_world_grid` — scan `world.tiles()` for the first index of each of `Empty`, `Solid`, `Ramp`; assert all three were found, invert the index to `(x, y, z)`, and assert `snap.tiles[i] == bridge(world.tile(Pos{x,y,z}).unwrap())` for each. This is the ordering proof, not just the mapping proof.
  - [x] `entities_mirror_dwarves` — 5 entities, ids exactly `[0, 1, 2, 3, 4]` ascending, positions equal to `world.dwarves()`, every `kind` is `Dwarf`.
  - [x] `snapshot_json_obeys_wire_conventions` — via `serde_json::to_value`: `["type"] == "snapshot"`, `["speed"] == "normal"`, `["tick"] == 0`, `["designations"]` and `["zones"]` are `[]`, `["dims"] == {"x":128,"y":128,"z":32}`, `["entities"][0]["kind"] == "dwarf"` and `["pos"]` is a 3-number array, and `["tiles"]` contains both the string `"empty"` and an object keyed `"solid"` whose value is a lowercase string.
- [x] **End-to-end tests** — `crates/simd/tests/serve.rs` (AC: 2, 6, 7, 8)
  - [x] Helper spawns `env!("CARGO_BIN_EXE_simd")` with arg `0` and `Stdio::piped()` stdout, reads the listening line, parses the port. Wrap the `Child` in a struct whose `Drop` kills it so a failed assertion cannot leak a daemon.
  - [x] `snapshot_on_connect_and_nothing_more` — read one line, `serde_json::from_str::<protocol::Snapshot>` succeeds, `tiles.len() == 524_288` and `entities.len() == 5`; then `set_read_timeout(Some(200ms))` and assert the next read yields `WouldBlock`/`TimedOut`, never a second line.
  - [x] `malformed_input_is_dropped_and_daemon_survives` — after reading the snapshot, send `not json\n` and `{"type":"bogus"}\n`, drop the connection, then open a **new** connection and assert it still receives a valid `Snapshot`.
  - [x] `client_disconnect_does_not_kill_daemon` — connect and drop immediately without reading, then assert a fresh connection is still served.
- [x] **Green gate** (AC: 9) — run the four commands under Verification and fix whatever they surface.

## Dev Notes

### Scope guardrails — do NOT build these here

- No tick loop, no `bevy_ecs` schedule, no systems, no `delta` message, no dirty-tile set. Story 2.1 owns all of it.
- No client→daemon command types. `designate`/`set_speed`/`save`/`load`/`quit` arrive with the stories that give them meaning (2.3 control, 3.1 world-mutating, AD-10). In this story *every* inbound line is unrecognized by definition.
- No `Designation`/`Zone` shapes — Story 3.1 owns them. The snapshot fields exist and are always empty (see skeleton).
- No `crossterm`, no rendering, no change to `crates/tui/`. Story 1.3 owns the client.
- No save/load, no `SaveState`, no pathfinding, no jobs.
- No protocol optimization: no batching, no compression, no binary encoding, no parallel tile arrays. The ~6 MB snapshot line is sanctioned chattiness (AD-3).

### What already exists (build on it, do not re-derive)

- `sim_core::World` exposes `generate(seed, dims)`, `dims()`, `seed()`, `tiles() -> &[Tile]`, `tile(Pos) -> Option<Tile>`, `dwarves() -> Vec<(Id, Pos)>` sorted ascending by `Id` [crates/sim-core/src/lib.rs:110-147]. `Material`, `Tile`, `Pos`, `Dims`, `Id` are all public.
- `crates/simd/src/main.rs` is currently a 14-line smoke print that generates the world and prints dims + dwarf count + port. Its seed `0xF005_7E1A` is the one to keep; the rest is replaced.
- `crates/protocol/src/lib.rs` is `DEFAULT_PORT` only. `crates/tui/src/main.rs` prints that port and stays untouched.
- Dependency edges are already correct in the manifests (`simd → sim-core`, `simd → protocol`, `tui → protocol`); adding serde does not change them.
- `World::generate` **only supports `Dims::DEFAULT`-scale worlds** — `debug_assert!`s require `dims.z >= 6` and `dims.x/y >= 3`, and small footprints panic on spawn candidates [crates/sim-core/src/lib.rs:78-93]. Use `Dims::DEFAULT` in every test; do not invent a tiny world to make tests fast.

### Code skeleton (the contract — match these shapes)

```rust
// crates/protocol/src/lib.rs
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 7373;

/// Wire message discriminator. `Delta` joins in Story 2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType { Snapshot }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Material { Stone, Soil, Ice, Snow }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tile { Empty, Solid(Material), Ramp(Material) }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind { Dwarf }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Speed { Paused, Normal, Fast }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dims { pub x: u32, pub y: u32, pub z: u32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity { pub id: u32, pub kind: EntityKind, pub pos: [i32; 3] }

/// Full world state, sent on connect (AD-3). Field order is wire order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub dims: Dims,
    /// Flat row-major: index = x + y*dims.x + z*dims.x*dims.y
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    // NOTE: designation and zone shapes land in Story 3.1; `Vec<()>` keeps the
    // wire fields present and always empty without inventing their shape now.
    pub designations: Vec<()>,
    pub zones: Vec<()>,
    pub speed: Speed,
    pub tick: u64,
}
```

```rust
// crates/simd/src/main.rs — structure, not the whole file
const SEED: u64 = 0xF005_7E1A;

fn main() -> anyhow::Result<()> {
    let port: u16 = match std::env::args().nth(1) {
        Some(arg) => arg.parse()?,
        None => protocol::DEFAULT_PORT,
    };
    let world = sim_core::World::generate(SEED, sim_core::Dims::DEFAULT);
    // NOTE: the world is static in this story, so the snapshot line is encoded
    // once and shared. Story 2.1's tick loop re-encodes per connection.
    let line = Arc::new(format!("{}\n", serde_json::to_string(&bridge::snapshot(&world))?));
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("listening on 127.0.0.1:{}", listener.local_addr()?.port());
    for stream in listener.incoming() { /* log Err, spawn thread on Ok */ }
    Ok(())
}

fn serve(mut stream: TcpStream, snapshot_line: &str) {
    // write_all the line (log + return on error), then move `stream` into a
    // BufReader and log-and-drop every line until EOF or read error.
}
```

### Key decisions & traps

- **`type` as a field, never `#[serde(tag = ...)]`.** Internal tagging buffers the entire message through `serde::private::de::Content` on deserialize — with 524 288 tiles that is a memory and latency cliff the client hits in Story 1.3.
- **Bind `127.0.0.1`, not `0.0.0.0`.** Phase one is localhost-only (NFR1).
- **Rust ignores `SIGPIPE`, so a write to a dropped client returns `Err(BrokenPipe)`** — handle it, never `unwrap()`. An `unwrap` here turns AC7 into a daemon crash.
- **Log to stderr only.** The e2e helper parses stdout for the port; a stray `println!` desynchronises it.
- **The `port` argument is not config.** It is a single positional `u16` (default `DEFAULT_PORT`) that exists so the e2e tests can bind an ephemeral port instead of fighting a running dev daemon. No arg-parsing crate, no config file.
- **`Speed` is defined in `protocol` only.** Speed is a loop-rate concern owned by `simd` (AD-2); `sim-core` never learns the wall-clock rate, so there is no sim-core enum to mirror or bridge here.
- **Only `Material` and `Tile` cross the AD-6 bridge** — those are the sim-core vocabularies the wire carries. `EntityKind` has no sim-core enum yet (dwarves are a marker component); it gains variants when items arrive in Story 3.2.
- **`tui` must not gain a `sim-core` edge.** The `cargo tree` probe in Verification is the guard.
- **`Vec<()>` serializes to `[]` and deserializes only from an empty array** — exactly the guarantee wanted while the shapes are undecided.

### Project Structure (files to touch)

```
Cargo.toml                              # UPDATE — serde, serde_json, anyhow in [workspace.dependencies]
crates/protocol/Cargo.toml              # UPDATE — serde
crates/protocol/src/lib.rs              # UPDATE — wire types (keep DEFAULT_PORT)
crates/sim-core/src/lib.rs              # UPDATE — tick field + tick() accessor
crates/simd/Cargo.toml                  # UPDATE — serde_json, anyhow
crates/simd/src/main.rs                 # UPDATE — replaces the smoke print with the TCP server
crates/simd/src/bridge.rs               # NEW    — sim-core → protocol conversion + unit tests
crates/simd/tests/serve.rs              # NEW    — end-to-end TCP tests
```

### Previous story intelligence (1.1)

- Tests must be strong enough to fail under mutation — 1.1's review found four assertions that passed while the implementation was sabotaged. The tile-ordering probe and the `[0,1,2,3,4]` id assertion above exist for that reason; do not weaken them to set-membership checks.
- The 1.1 dev record wrongly implied an automated `simd` assertion existed when the step was a manual `cargo run`. This story's `crates/simd/tests/serve.rs` makes the daemon's behaviour genuinely automated — record RED/GREEN honestly, and say "manual" when a step was manual.
- The dev-agent sandbox is offline: `cargo fetch` while online, then build with `--offline`.

### Dependency versions (verified crates.io 2026-08-02)

| Crate | Version | Used by | Status |
| --- | --- | --- | --- |
| serde (derive) | 1.0.229 | protocol | already in `Cargo.lock` + registry cache via bevy_ecs |
| serde_json | 1.0.151 | simd | **not** cached — needs `cargo fetch` |
| anyhow | 1.0.104 | simd | **not** cached — needs `cargo fetch` |

All three are on the spine's closed stack, so no new-dependency justification is required.

### Verification

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo tree -p tui | rg sim-core   # must return nothing (AC9)
```

Live check (AC2, AC8) — the observable outcome, not just green tests:

```bash
cargo run -p simd &                                   # prints: listening on 127.0.0.1:7373
head -c 300 < /dev/tcp/127.0.0.1/7373                 # first bytes of the snapshot line
```

Branch: `1-2-the-daemon-serves-the-world`. Commit as `Völundr <jeicei75@gmail.com>`, one commit per green step, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.2] — user story and the three source ACs
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md] — AD-1, AD-2 (speed is a loop concern), AD-3 (NDJSON, chattiness sanctioned), AD-4, AD-6 (wire types only in `protocol`, enums never strings, exhaustive bridge), AD-8 (snapshot vs delta), AD-9 (u32 ids on the wire), AD-10 (commands arrive later), Consistency Conventions, Stack, Protocol v0 message list
- [Source: docs/technical-preferences.md#Anti-overengineering rules] — closed dependency list, YAGNI as policy, `// NOTE:` convention
- [Source: _bmad-output/implementation-artifacts/1-1-a-seeded-frozen-world-exists.md#Review Findings] — mutation-tested assertions, `Dims::DEFAULT` precondition
- [Source: crates/sim-core/src/lib.rs:78-147] — the world API this story reads and the documented dims precondition

## Dev Agent Record

### Agent Model Used

OpenAI GPT-5 Codex (Völundr)

### Debug Log References

- Tasks 1–4: completed and committed in the previous run; GREEN independently verified by the orchestrator. RED details were not available in this continuation.
- Task 5: manual AC/source inspection; GREEN — `cargo fmt --check`, `cargo test -p simd --bin simd`, and `cargo clippy -p simd --bin simd -- -D warnings`.
- Task 6: RED not observed because the bridge implementation already existed; GREEN — all four bridge tests passed, including the three new mutation-sensitive tests; targeted clippy passed.
- Task 7: RED not observed because the TCP server already existed; GREEN — all three process-level loopback tests passed; targeted clippy passed.
- Task 8: GREEN — full format, clippy, and test gate passed; `cargo tree -p tui | rg sim-core` produced no matches (expected exit 1). Live loopback check was manual and returned the listening line plus a snapshot prefix.

### Completion Notes List

- Added protocol snapshot types, world tick state, and exhaustive sim-core-to-protocol conversion across the prior and current runs.
- Replaced the daemon smoke output with a localhost TCP server that sends exactly one shared snapshot line per connection and safely drops all inbound lines.
- Added mutation-sensitive bridge coverage for tile ordering, exact dwarf ordering and positions, and serialized wire conventions.
- Added end-to-end daemon coverage for one-snapshot idleness, malformed input survival, and immediate client disconnect survival.
- Verified the complete story gate and a manual live connection without adding dependencies or changing `tui`.

### File List

- `Cargo.lock`
- `Cargo.toml`
- `crates/protocol/Cargo.toml`
- `crates/protocol/src/lib.rs`
- `crates/sim-core/src/lib.rs`
- `crates/simd/Cargo.toml`
- `crates/simd/src/bridge.rs`
- `crates/simd/src/main.rs`
- `crates/simd/tests/serve.rs`
- `_bmad-output/implementation-artifacts/1-2-the-daemon-serves-the-world.md`

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-02 | Story created |
| 2026-08-02 | Implemented protocol snapshots, localhost serving, mutation-sensitive bridge tests, and end-to-end daemon tests; full gate green and status advanced to review. |
