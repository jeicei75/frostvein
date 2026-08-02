---
baseline_commit: 1506d05aa4946cceb590431ae536d2eb451944ae
---

# Story 1.1: A Seeded Frozen World Exists

Status: review

## Story

As a developer,
I want the Cargo workspace scaffolded and a seeded voxel world generated in `sim-core`,
so that every later story builds on a deterministic frozen world with the quality gate already enforced.

## Acceptance Criteria

1. The workspace holds four crates — `sim-core`, `protocol`, `simd`, `tui` — under `crates/`, edition 2024, and the only workspace dependency edges are `simd → sim-core`, `simd → protocol`, `tui → protocol`. `cargo tree -p tui | rg sim-core` returns no match.
2. All four crate roots carry `#![forbid(unsafe_code)]`.
3. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass from a clean checkout.
4. `World::generate(seed, Dims::DEFAULT)` produces a 128×128×32 world whose terrain is layered stone → soil → surface, with both `Snow` and `Ice` present among surface tiles and `Air` above.
5. Column surface heights span at least 3 distinct z-levels, and no two 4-adjacent columns differ in height by more than 1.
6. Every 4-adjacent column pair whose heights differ by exactly 1 has a `Ramp` tile at the top of the lower column.
7. Exactly 5 dwarves spawn at distinct positions, each standing in an `Empty` tile directly above a `Solid` tile, each carrying a unique `u32` `Id` issued by the world's single monotonic allocator.
8. Two worlds generated independently from the same seed are equal tile-for-tile and dwarf-for-dwarf (id and position).
9. Two worlds generated from *different* seeds differ in their tile arrays.

## Tasks / Subtasks

- [x] **Scaffold the workspace** (AC: 1, 2, 3)
  - [x] Root `Cargo.toml` as a virtual manifest: `resolver = "3"`, members `crates/sim-core`, `crates/protocol`, `crates/simd`, `crates/tui`; `[workspace.package]` sets `edition = "2024"`, `version = "0.1.0"`.
  - [x] Declare only the dependencies this story uses (see Key decisions & traps). `serde`, `serde_json`, `crossterm`, `thiserror`, `anyhow` are **not** added in this story — they arrive with the story that first needs them.
  - [x] `#![forbid(unsafe_code)]` at the top of `sim-core/src/lib.rs`, `protocol/src/lib.rs`, `simd/src/main.rs`, `tui/src/main.rs`.
  - [x] `protocol` contains exactly `pub const DEFAULT_PORT: u16 = 7373;` — wire types land in Story 1.2.
  - [x] `tui/src/main.rs` is a stub that prints the port it will connect to (proves the `tui → protocol` edge, nothing more).
  - [x] `.gitignore` covers `/target`.
- [x] **Terrain generation in `sim-core`** (AC: 4, 5, 6)
  - [x] Define `Material`, `Tile`, `Pos`, `Dims`, `Id` per the skeleton below.
  - [x] Seed the worldgen RNG as `ChaCha8Rng::seed_from_u64(seed ^ STREAM_WORLDGEN)` with `STREAM_WORLDGEN` a hardcoded `u64` const. No other RNG source anywhere.
  - [x] Build a per-column height field from hand-rolled seeded value noise (lattice of seeded values + bilinear interpolation), centred near z=16 with amplitude of a few levels; then run a smoothing/clamp pass so no 4-adjacent columns differ by more than 1 (AC5 depends on this).
  - [x] Fill each column: `Stone` at depth, `Soil` for the few tiles below the surface, the top tile `Snow` or `Ice` chosen from the worldgen stream, `Empty` above.
  - [x] Convert the top tile of a column to `Ramp(material)` when any 4-neighbour column is exactly 1 higher.
- [x] **Dwarf spawning** (AC: 7)
  - [x] Implement the single monotonic `IdAllocator` on `World`; every entity kind draws from it.
  - [x] Spawn 5 `bevy_ecs` entities with `(Dwarf, Id, Pos)`, positions chosen from the worldgen stream among flat `Solid`-topped columns, dwarf occupying the `Empty` tile above. Reject duplicates so all 5 positions are distinct.
- [x] **`simd` smoke binary** (AC: 1)
  - [x] `main()` generates the default world from a hardcoded seed and prints dims + dwarf count, then exits. This exercises both `simd` edges rather than merely declaring them.
- [x] **Integration tests** — `crates/sim-core/tests/worldgen.rs` (AC: 4–9)
  - [x] `same_seed_produces_identical_worlds` — assert `tiles()` equal and dwarf `(Id, Pos)` lists equal.
  - [x] `different_seed_produces_different_world` — assert `tiles()` differ.
  - [x] `surface_is_icy` — `Snow` and `Ice` both appear among top tiles; `Stone` and `Soil` both appear.
  - [x] `height_varies_and_steps_are_at_most_one` — ≥3 distinct column heights; no 4-adjacent pair differs by >1.
  - [x] `ramps_connect_every_step` — every 1-step adjacency has a `Ramp` at the lower column's top.
  - [x] `five_dwarves_on_walkable_surface` — count, distinctness, unique ids, `Empty` tile with `Solid` below.
- [x] **Green gate** — run all three gate commands, fix anything they surface.

### Review Findings

Code review 2026-08-02 (4 layers: Blind Hunter, Edge Case Hunter, Acceptance Auditor,
Feature Auditor). Production code is correct at `Dims::DEFAULT` and every AC holds when
observed. All findings below concern **test strength** and **non-default `Dims`**, not
shipped behaviour. Mutation results were re-verified independently by the orchestrator.

- [x] [Review][Decision→Patch] RESOLVED (Wolf, 2026-08-02): document the precondition rather than support small worlds. Added `debug_assert!`s for `dims.z >= 6` and `dims.x/y >= 3` plus a `# Panics` doc and `// NOTE:` on `World::generate`. Original finding: public `World::generate` panics or silently degrades on non-`DEFAULT` dims — three reachable sites, all confirmed by execution: `dims.z <= 4` panics in `f64::clamp` (`min > max`) at `crates/sim-core/src/worldgen.rs:36`; fewer than 5 flat spawn candidates (e.g. `Dims{x:2,y:2}`, or any zero axis) panics `cannot sample empty range` at `crates/sim-core/src/lib.rs:166`; `dims.z == 5` returns a *silently* flat world with zero ramps, making AC4–AC6 false with no error. No current caller passes non-`DEFAULT` dims. DECISION: does `generate` support small worlds (fix the height-clamp maths + spawn guard), or is `Dims::DEFAULT` the documented precondition (`debug_assert!` + `// NOTE:`)? Story 2.x scenario tests may want tiny fast worlds, which is why this is Wolf's call and not an unambiguous patch.
- [x] [Review][Patch] FIXED — verified: the mutation now FAILS `different_seed_produces_different_world` (was passing). AC9's seed-sensitivity test did not do the job the Dev Notes assign it [crates/sim-core/tests/worldgen.rs:22] — Dev Notes state "AC9 exists precisely to catch an implementation that ignores the seed". VERIFIED by mutation: drawing the height lattice from a hardcoded `ChaCha8Rng::seed_from_u64(1234)` (stream alignment preserved) makes terrain shape identical for every seed, and all 6 tests still pass. `assert_ne!` on the whole tile array passes if any 1 of 524,288 tiles differs — the per-column Snow/Ice coin flip alone satisfies it. Fix: compare height fields across seeds and assert dwarf positions differ too.
- [x] [Review][Patch] FIXED — verified: the inverted-layering mutation now FAILS `surface_is_icy` (was passing). AC4's layering order and "Air above" were never asserted [crates/sim-core/tests/worldgen.rs:37] — VERIFIED by mutation: inverting the stone/soil predicate at `worldgen.rs:85` (Soil at bedrock, Stone under the snow) keeps all 6 tests green. `surface_is_icy` proves only set-membership, never vertical order; nothing asserts `Empty` above the surface, and the `surface_height` helper defines the surface as the topmost non-`Empty` tile, making that clause tautological. Also unasserted: `tiles().len() == 524288`.
- [x] [Review][Patch] PARTIALLY FIXED — see note below. AC7's "single monotonic allocator" clause was unverified [crates/sim-core/tests/worldgen.rs:120] — VERIFIED by mutation: replacing the spawn draw with a constant index 0 (dwarves no longer seed-dependent) keeps all 6 tests green; a separate run with random `Id`s also passes. The test asserts only distinctness. Fix: assert `ids == [Id(0), Id(1), Id(2), Id(3), Id(4)]`.
- [x] [Review][Patch] FIXED — widened to `usize` before multiplying in `index()`, the tile `Vec` allocation, the lattice range, and the heights `with_capacity`. Grid index and volume arithmetic was evaluated in `u32` before the `usize` cast [crates/sim-core/src/worldgen.rs:9, :19, :80] — `(dims.x * dims.y * dims.z) as usize` and `(x + y*dims.x + z*dims.x*dims.y) as usize` overflow in debug and wrap **silently in release**, allocating a wrong-sized `Vec` or addressing the wrong tile. Unreachable at `Dims::DEFAULT` (524,288). Fix is unambiguous and behaviour-preserving: widen to `usize` before multiplying. Note the story deliberately chose `i32` positions "so neighbour arithmetic cannot underflow" — this is the one place that care was not applied.
- [x] [Review][Patch] FIXED — `height_varies_and_steps_are_at_most_one` now runs at seeds 0, 1, 42, 7777. Previously every property test ran at exactly one seed (42) [crates/sim-core/tests/worldgen.rs] — AC5's "≥3 distinct z-levels" is a statistical claim sampled once. Independently swept seeds 0–40: minimum 9 distinct heights, Snow+Ice present at every seed, so the properties are robust — but the suite would not notice if they stopped being.
- [x] [Review][Patch] FIXED — `// NOTE:` added at the lattice draw naming the float determinism contract. Float-based determinism carried no `// NOTE:` [crates/sim-core/src/worldgen.rs:16, :27-36] — `rng.random::<f64>()`, `lerp`, `smooth`, `.round()` are the only floating point in the codebase and sit directly on the seed⇒identical-world contract. It is in fact safe (Rust does no FMA contraction; these are correctly-rounded IEEE ops), but house convention applied a `// NOTE:` to the far less consequential ramp limitation and skipped it here.
- [x] [Review][Patch] FIXED — Debug Log reworded to say the simd RED step was a manual `cargo run`, not an automated assertion. It cited a `simd` smoke assertion that does not exist [story Dev Agent Record] — "simd RED: the smoke assertion saw the original port-only output" implies an automated test; there is no `crates/simd/tests/` in the diff or File List. The RED step was a manual `cargo run -p simd`. Correct the wording so the record is accurate.
**Post-patch mutation re-run (orchestrator, 2026-08-02).** Four mutations against the
strengthened suite — 3 of 4 now caught, 1 deliberately left open:

| Mutation | Before | After |
| --- | --- | --- |
| Terrain shape ignores the seed (hardcoded lattice RNG) | 6/6 pass | **FAILS** `different_seed_produces_different_world` |
| Layering inverted (soil at bedrock, stone under snow) | 6/6 pass | **FAILS** `surface_is_icy` |
| Ids bypass the allocator (random `u32`) | 6/6 pass | **FAILS** `five_dwarves_on_walkable_surface` |
| Spawn ignores the RNG draw (constant index 0) | 6/6 pass | **still 6/6 pass — OPEN** |

- [x] [Review][Defer] KNOWN OPEN, accepted: a spawn that ignores the RNG draw is still not detected [crates/sim-core/src/lib.rs:166] — the cross-seed dwarf-position assertion added above does NOT catch it, because the candidate *list* is itself terrain-derived and therefore seed-dependent: with a constant index the positions still differ between seeds. Accepted rather than fixed because it violates **no AC** — AC7 requires distinct positions, valid tiles and allocator ids; AC8/AC9 still hold — and contradicts only the task wording "positions chosen from the worldgen stream". Any test that would catch it must assert scan-order properties, which means either duplicating production candidate logic in the test or a brittle clustering heuristic; both are worse than the gap under the project's simplicity policy. Revisit if spawn placement ever becomes gameplay-relevant.
- [x] [Review][Defer] Single RNG stream couples dwarf positions to terrain's exact draw count [crates/sim-core/src/lib.rs:79-91] — deferred, architectural. `layered_terrain` consumes exactly `dims.x*dims.y` bool draws before spawn reads anything, so any future change to surface-material selection silently relocates all five dwarves and invalidates recorded baselines. The story mandated the single `STREAM_WORLDGEN` stream, so this is compliant; AD-7's "purpose-named streams" is the relevant future decision. Revisit at Story 2.4 (`SaveState` persists RNG state).
- [x] [Review][Defer] Spawn distribution is biased toward the map border [crates/sim-core/src/lib.rs:143-153] — deferred, no AC. `is_flat` filters out-of-bounds neighbours before `.all()`, so corner columns are judged on 2 neighbours and interior columns on 4, making border columns systematically likelier to qualify. Observed: seed 0 spawns a dwarf at `Pos{x:0,y:26,z:20}`, hard against the wall. Unintended distribution rather than a chosen one.
- [x] [Review][Defer] Story commits reference five planning docs that are untracked [story References] — deferred, pre-existing. `_bmad-output/planning-artifacts/` and `docs/architecture.md` are untracked in the working tree, so this branch commits an artifact whose entire evidence chain dangles in a fresh clone. Repo-hygiene item for Wolf, not caused by this story.

## Dev Notes

### Scope guardrails — do NOT build these here

- No tick loop, no `bevy_ecs` schedule, no systems. Story 2.1 introduces the chained schedule.
- No `World::set_tile` and no dirty-tile set. AD-8 exempts world *construction* from dirty tracking; the dirty set arrives with deltas in Story 2.1.
- No wire types, no serde, no TCP. Story 1.2 owns the `protocol` message shapes and the snapshot.
- No pathfinding, no jobs, no designations, no `SaveState`, no walkability query beyond what the spawn rule needs.
- No `crossterm`, no rendering. Story 1.3 owns the TUI.
- Dwarves are static data in this story. They do not move.

### What already exists

- The repo has **no Rust code at all** — this story creates the first line of it. `git ls-files` shows only docs, `CLAUDE.md`, `scripts/codex-handoff.sh`, and `_bmad/` process files.
- Toolchain is installed and current: `rustc 1.97.1`, `clippy 0.1.97`, `rustfmt 1.9.0-stable`. Edition 2024 needs ≥1.85, so this is comfortably clear.
- All eight stack versions were re-verified against crates.io on 2026-08-02 and match the spine exactly.

### Code skeleton (the contract — match these shapes)

```rust
// crates/sim-core/src/lib.rs
#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material { Stone, Soil, Ice, Snow }

/// A voxel. `Empty` is air; `Solid` is wall/floor; `Ramp` is a walkable slope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile { Empty, Solid(Material), Ramp(Material) }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos { pub x: i32, pub y: i32, pub z: i32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims { pub x: u32, pub y: u32, pub z: u32 }
impl Dims { pub const DEFAULT: Dims = Dims { x: 128, y: 128, z: 32 }; }

/// Sim-assigned stable entity id (AD-9). One allocator for every entity kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub u32);

pub struct World { /* dims, tiles: Vec<Tile>, ecs: bevy_ecs::world::World, ids, seed */ }

impl World {
    pub fn generate(seed: u64, dims: Dims) -> World;
    pub fn dims(&self) -> Dims;
    pub fn seed(&self) -> u64;
    /// Flat row-major: index = x + y*dims.x + z*dims.x*dims.y
    pub fn tiles(&self) -> &[Tile];
    pub fn tile(&self, p: Pos) -> Option<Tile>;
    /// Sorted ascending by `Id` — stable order is required by AD-7.
    pub fn dwarves(&self) -> Vec<(Id, Pos)>;
}
```

### Key decisions & traps

- **`sim-core::World` is not `bevy_ecs::World`.** Our `World` *owns* a private `bevy_ecs::world::World` field. Import the bevy one as `use bevy_ecs::world::World as EcsWorld;` — the name collision is the single most likely source of confusion in this story.
- **`bevy_ecs` default features are correct; never enable `multi_threaded`.** It is off by default in 0.19.0 (defaults are `async_executor`, `backtrace`, `bevy_reflect`, `std`). Enabling it would violate AD-7's single-threaded requirement.
- **The stack is closed — do not add a noise crate.** Value noise is hand-rolled from the seeded `ChaCha8Rng`, roughly 30 lines. Any new dependency requires a written one-sentence justification, and none is warranted here.
- **Coordinates are `i32`, not `u32`.** Grid dimensions are `u32`, but positions are `i32` so that neighbour arithmetic in Story 3.2's A* cannot underflow. Bounds-check on conversion.
- **Every random draw comes from the single worldgen stream.** No `thread_rng`, no `RandomState`, no wall clock, no `HashMap`/`HashSet` iteration feeding a sim outcome. AC9 exists precisely to catch an implementation that ignores the seed and would otherwise pass AC8.
- **`rand_chacha` needs no `serde` feature yet.** It is added in Story 2.4 when `SaveState` must persist RNG stream state.
- **Dwarf position semantics:** a dwarf occupies the `Empty` tile *above* the solid it stands on. Spawn only on flat `Solid` tops in this story — not on `Ramp` tiles.
- Height smoothing is what makes AC5 and AC6 satisfiable. Generate, then clamp neighbours to a maximum 1-level step *before* placing ramps. `// NOTE:` the limitation that ramps only bridge single-level steps.

### Project Structure (files to touch — all NEW)

```
Cargo.toml                              # virtual workspace manifest
.gitignore                              # /target
crates/sim-core/Cargo.toml              # deps: bevy_ecs, rand, rand_chacha
crates/sim-core/src/lib.rs              # Material, Tile, Pos, Dims, Id, World
crates/sim-core/src/worldgen.rs         # height field, layering, ramps, spawns
crates/sim-core/tests/worldgen.rs       # the six integration tests
crates/protocol/Cargo.toml              # no dependencies
crates/protocol/src/lib.rs              # DEFAULT_PORT only
crates/simd/Cargo.toml                  # deps: sim-core, protocol
crates/simd/src/main.rs                 # generate + print smoke
crates/tui/Cargo.toml                   # dep: protocol
crates/tui/src/main.rs                  # stub printing DEFAULT_PORT
```

Splitting `worldgen.rs` out of `lib.rs` is optional; keep it if `lib.rs` gets long.

### Dependency versions (verified crates.io 2026-08-02)

| Crate | Version | Used by |
| --- | --- | --- |
| bevy_ecs | 0.19.0 | sim-core (headless; default features) |
| rand | 0.10.2 | sim-core |
| rand_chacha | 0.10.0 | sim-core |

### Verification

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo tree -p tui | rg sim-core   # must return nothing (AC1)
```

Branch: `1-1-a-seeded-frozen-world-exists`. Commit as `Völundr <jeicei75@gmail.com>`, one commit per green step, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.1] — user story and the three source ACs
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md] — AD-1 (pure core), AD-7 (structural determinism), AD-8 (construction exempt from dirty tracking), AD-9 (single id allocator), Consistency Conventions (row-major index, `forbid(unsafe_code)`, geometry), Stack, Structural Seed
- [Source: docs/architecture.md#Conventions worth memorizing] — the ten-minute restatement
- [Source: docs/technical-preferences.md#Anti-overengineering rules] — closed dependency list, YAGNI as policy, `// NOTE:` convention
- [Source: _bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-01/prd.md] — FR1, FR2, FR3, NFR3, NFR4
- [Source: _bmad-output/planning-artifacts/implementation-readiness-report-2026-08-02.md#Findings] — no open finding affects this story; the developer-voiced framing is explicitly sanctioned

## Dev Agent Record

### Agent Model Used

OpenAI Codex (GPT-5)

### Debug Log References

- Scaffold RED: `cargo build --offline` failed because the root manifest did not exist.
- Terrain RED: the three initial worldgen tests failed to compile because the contracted world API did not exist.
- Spawn RED: `five_dwarves_on_walkable_surface` failed with 0 dwarves instead of 5.
- simd RED: a manual `cargo run -p simd` printed the original port-only output instead of generated world dimensions and dwarf count. (No automated `simd` test exists; this step was verified by running the binary.)
- GREEN: `cargo fmt --check`, `cargo clippy --all-targets --offline -- -D warnings`, `cargo test --offline`, and the `tui` dependency-edge probe all passed.

### Completion Notes List

- Scaffolded the edition-2024 four-crate Cargo workspace with only the three approved external dependencies and required internal edges.
- Implemented deterministic ChaCha8-seeded value-noise terrain, layered frozen materials, bounded height steps, and connecting ramps.
- Spawned five deterministic ECS dwarves at distinct flat solid-topped positions using a single monotonic ID allocator.
- Added the simd generation smoke path and all six named worldgen integration tests; the complete offline quality gate passes.

### File List

- `.gitignore`
- `Cargo.lock`
- `Cargo.toml`
- `_bmad-output/implementation-artifacts/1-1-a-seeded-frozen-world-exists.md`
- `_bmad-output/implementation-artifacts/deferred-work.md` (new — code review deferrals)
- `crates/protocol/Cargo.toml`
- `crates/protocol/src/lib.rs`
- `crates/sim-core/Cargo.toml`
- `crates/sim-core/src/lib.rs`
- `crates/sim-core/src/worldgen.rs`
- `crates/sim-core/tests/worldgen.rs`
- `crates/simd/Cargo.toml`
- `crates/simd/src/main.rs`
- `crates/tui/Cargo.toml`
- `crates/tui/src/main.rs`

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-02 | Story created |
| 2026-08-02 | Implemented and verified the seeded frozen world walking scaffold. |
| 2026-08-02 | Code review (4 layers). 8 patches applied: strengthened AC4/AC7/AC9 test assertions, multi-seed property check, `usize` index widening, `debug_assert!` + `// NOTE:` on the supported dims range, float-determinism note, Debug Log correction. 3 items deferred, 4 dismissed. Verified by re-running mutations: 3 of 4 previously-undetected mutations now fail the suite; the fourth is documented as knowingly open. |
