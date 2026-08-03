---
baseline_commit: 871a6b42d96247252b6f81a9d04331ab9e68d2f3
---

# Story 2.2: Dwarves Wander the Frost

Status: in-progress

## Story

As the boss,
I want idle dwarves to wander near their spot, visibly and deterministically,
so that the world reads as alive even when I give no orders.

## Acceptance Criteria

1. `sim-core` exports `JobState { Idle, Walk, Work }` (a `Component`); every dwarf carries
   one, `Idle` at spawn, and `World::dwarves()` returns `Vec<(Id, Pos, JobState)>` in
   ascending `Id` order.
2. Randomness runs on three purpose-named streams (AD-7), each seeded `seed ^ STREAM_*`:
   `STREAM_WORLDGEN` (terrain, unchanged), the new `STREAM_SPAWN` (dwarf placement), and the
   new `STREAM_WANDER` (a `WanderRng` resource). Terrain output for a given seed is
   byte-identical to before this story; spawn no longer reads the worldgen stream.
3. The five spawn positions for seed 42 are pinned as hand-written literals in a test, and
   adding an extra draw to `layered_terrain` leaves them unchanged — run that experiment by
   hand and record it, since it is the only evidence the streams are actually decoupled.
4. The one chained schedule becomes `(advance_tick, wander).chain()`. `wander` draws every
   random choice from `WanderRng` and processes dwarves in ascending `Id` — no wall clock,
   no unseeded randomness, no order-dependent iteration.
5. Wander rule: a dwarf with `cooldown > 0` decrements it and is `Idle`. At `0` it steps to a
   uniformly chosen orthogonally adjacent **standable** tile at the same z within
   `WANDER_RADIUS` (3) of its spawn home, becomes `Walk` for that tick, and resets
   `cooldown` to `WANDER_REST_TICKS` (10).
6. Standable = the tile is `Empty` **and** the tile directly below is `Solid` or `Ramp`.
   Across a 200-step scenario run every dwarf is on a standable tile and within radius 3 of
   its home — asserted against an oracle the test derives itself from `World::tile`, never
   by calling the production predicate.
7. Same seed, two independently generated worlds, 200 steps each → identical `dwarves()`
   (position and state, dwarf for dwarf).
8. Over a 200-step run a single dwarf takes at least two **different** step vectors. A
   constant direction choice must fail this.
9. A dwarf whose four neighbours are made non-standable via `set_tile` stays on its tile and
   reports `Idle` for the following ticks.
10. `protocol::Entity` gains `state: JobState` (`"idle"|"walk"|"work"`), wire order
    `{id, kind, pos, state}`. The hand-written JSON literals in `protocol` and `tui` carry the
    new field; `simd` bridges `sim_core::JobState → protocol::JobState` by exhaustive `match`
    with no wildcard arm, tested against an independent oracle table (AD-6).
11. Through the running daemon: across consecutive deltas at least one entity's `pos` changes
    and the entity `state` field is observed as both `"idle"` and `"walk"`.
12. `tui` colors `☺` by state — idle, walk and work each a distinct pinned RGB — and in one
    rendered frame a walking dwarf's cell color differs from an idle dwarf's.
13. `scripts/gate.sh` passes and `scripts/mutate.sh
    _bmad-output/implementation-artifacts/mutations/2-2-dwarves-wander-the-frost.sh` reports
    zero survivors.

## Tasks / Subtasks

- [x] **`sim-core`: terrain into the ECS** (AC: 4, 6)
  - [x] Add `#[derive(Resource)] struct Terrain { dims, tiles, dirty }` and move the bodies of
        `World::tile` / `set_tile` / `drain_dirty` onto `impl Terrain`, plus
        `fn is_standable(&self, p: Pos) -> bool`. `World`'s public signatures
        (`dims/tiles/tile/set_tile/drain_dirty/tick/step/seed/generate`) do not change and
        delegate to the resource — **every existing `sim-core`, `simd` and `tui` test must
        still pass unmodified except where a new field forces a construction change.**
  - [x] `spawn_dwarves` reads tiles through the resource: build the candidate `Vec` in an inner
        scope so the immutable borrow of `self.ecs` ends before the `spawn` loop.
- [x] **`sim-core`: split spawn off the worldgen stream** (AC: 2, 3)
  - [x] `STREAM_SPAWN` constant; `generate` builds a second `ChaCha8Rng::seed_from_u64(seed ^
        STREAM_SPAWN)` and hands *that* to `spawn_dwarves`. The worldgen rng keeps
        `height_field` and `layered_terrain` and nothing else.
  - [x] Test `spawn_positions_for_seed_42_are_pinned` with the five positions as literals, and
        a terrain-unchanged assertion (same seed → the tile vector still matches what the
        existing worldgen tests describe).
  - [x] **Record the decoupling experiment**: add one throwaway `rng.random::<bool>()` draw at
        the end of `layered_terrain`, run the pinned test, confirm it stays GREEN, revert, and
        paste that into the Dev Agent Record. This is inverted sabotage — the evidence is a
        test that *survives*. Without the split it goes red; that is the future bug this
        closes.
  - [x] Dwarf positions for a given seed change once, here. Nothing pins them today (verified:
        no test hardcodes a position), and 2.4's `SaveState` baselines do not exist yet — which
        is why this lands now rather than later.
- [x] **`sim-core`: wander stream, state, system** (AC: 1, 2, 4, 5)
  - [x] `STREAM_WANDER`, `WANDER_RADIUS = 3`, `WANDER_REST_TICKS = 10` as constants at their
        use site; `WanderRng` + `Terrain` inserted in `generate`, `JobState::Idle` and
        `Wander { home: pos, cooldown: id.0 % WANDER_REST_TICKS }` added to the spawn bundle.
        The modulo staggers the five dwarves without a second RNG draw.
  - [x] Register `schedule.add_systems((advance_tick, wander).chain())` — order is explicit,
        `advance_tick` first.
  - [x] `World::dwarves()` returns the 3-tuple. // NOTE: promote to a struct at the fourth
        field (3.2 adds the carried item).
- [x] **`sim-core`: scenario coverage** (AC: 6, 7, 8, 9)
  - [x] Extend `crates/sim-core/tests/scenario.rs`: `dwarves_stay_standable_and_near_home`,
        `same_seed_wanders_identically`, `wander_directions_are_not_constant`,
        `a_walled_in_dwarf_stays_idle` (build the wall with `set_tile`).
  - [x] The standability oracle in the test is written out from `World::tile` — do not call the
        production predicate on both sides.
- [x] **Wire: state on the entity** (AC: 10)
  - [x] `protocol`: `JobState` enum + `Entity.state`; add `"state": "idle"` to the `WIRE` and
        `DELTA_WIRE` literals and assert it decodes to the right variant; extend
        `every_material_and_tile_variant_has_a_pinned_wire_name` with the three state names.
  - [x] `simd/src/bridge.rs`: `fn job_state(sim_core::JobState) -> protocol::JobState`,
        exhaustive, no wildcard; both `snapshot` and `delta` fill the field. Test it against a
        restated oracle table like `expected_tile` already does.
  - [x] `tui`: add the field to the `SNAPSHOT_LINE` / `DELTA_LINE` literals in `main.rs` tests
        and to the `Entity` values built in `view.rs` tests.
- [x] **`simd`: prove it end to end** (AC: 11)
  - [x] `crates/simd/tests/serve.rs`: read a run of consecutive deltas from a live daemon;
        assert some entity's `pos` changes and that both `idle` and `walk` appear in the
        entity states across that run. Keep `bridge::delta`'s single-call-per-iteration
        discipline — it drains the dirty set.
- [x] **`tui`: color by state** (AC: 12)
  - [x] `palette::entity_cell(kind, state)` with an exhaustive `(kind, state)` match; call site
        in `view::render` passes `entity.state`. Extend `every_look_is_pinned` to all three.
  - [x] A `view` test rendering one idle and one walking dwarf in the same frame asserts their
        cells differ.
- [ ] **Observability instrument** (AC: 11, 12) — the human check for "the world visibly lives".
      Use 2.1's existing `tui --frames N` (real reader thread, real `apply` → `render`); do not
      invent a second instrument. Exact commands in Verification; paste what you saw.
- [ ] **Sabotage + mutation set** (AC: 13)
  - [ ] Write `_bmad-output/implementation-artifacts/mutations/2-2-dwarves-wander-the-frost.sh`
        with at least: wander never moves the dwarf; `random_range` replaced by a constant `0`;
        `cooldown` never reset (steps every tick); `is_standable` drops the below-tile check;
        `bridge` hardcodes `JobState::Idle`; `WANDER_RADIUS` widened to 6; `spawn_dwarves` fed
        the worldgen rng again (must kill the pinned-positions test). Run `scripts/mutate.sh`
        and paste the table.
  - [ ] Paste the actual RED output for every new mapping/constant test into the Dev Agent
        Record (AGENTS.md rule 1).
- [ ] **Green gate** (AC: 13) — `scripts/gate.sh`, then the live check. Report what printed.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No commands, in either direction.** No `set_speed`, no pause, no new keys, no command
  types in `protocol`. Story 2.3 owns the entire command path; the client still sends zero
  bytes.
- **No jobs, no claiming, no reaction delay, no A\*.** `JobState::Work` ships unreachable —
  the same situation `Speed::Paused` has been in since 1.2, and review dismissed that as
  noise. Do not invent a producer for it.
- **Flat steps only.** No ramp climbing, no z-changing moves. The climb rule belongs to
  3.2's A* (AD-5); leave a `// NOTE:` at the candidate filter saying so.
- **No `SaveState`, no `rand_chacha` `serde` feature.** 2.4 serializes what this story
  creates; adding the feature now is dead weight.
- **No status-line change.** It is already 78 columns and its overflow is a recorded deferred
  item — touching it makes that fix yours.
- **No profession colors.** The brief's miner-amber/hauler-teal identity arrives with real
  professions; this story colors by state only.
- **No designation/zone shapes** (`Vec<()>` stays) and **no change to worldgen's terrain**.
  Spawn *positions* do move once, when spawn gets its own stream — that is AC2/AC3, and it is
  the only sanctioned worldgen-output change in this story.

### What already exists (build on it, do not re-derive)

- `World { dims, tiles, dirty, ecs, schedule, ids, seed }` with the 2.1 machinery —
  `Tick` resource, `(advance_tick,).chain()`, `set_tile`/`drain_dirty`, `dwarves()` sorted by
  `Id` [crates/sim-core/src/lib.rs:84-241]. **No RNG is retained**: the `ChaCha8Rng` is a
  local in `generate` and dropped [lib.rs:111-131].
- `bridge::snapshot` / `bridge::delta` already map entities; `delta` is destructive and must
  be called once per iteration [crates/simd/src/bridge.rs:35-65]. Its tests show the required
  oracle-table pattern (`expected_tile`, `expected_material`) [bridge.rs:88-108].
- `tui` reader thread, `apply`, and the `--frames N` headless loop that runs the real client
  path and exits [crates/tui/src/main.rs:148-262]; `entity_cell` is the one dwarf-color site
  [crates/tui/src/palette.rs:59-66].
- Pinned wire literals that must move together with `Entity`: `WIRE` / `DELTA_WIRE`
  [crates/protocol/src/lib.rs:110-129] and `SNAPSHOT_LINE` / `DELTA_LINE`
  [crates/tui/src/main.rs:374-386]. `crates/tui/tests/client.rs` builds entity-free messages
  and needs no change.
- `scripts/gate.sh` and `scripts/mutate.sh` (mutations file format:
  `mutations/2-1-the-world-runs-on-its-own-clock.sh`).

### Code skeleton

```rust
// crates/sim-core/src/lib.rs
const STREAM_SPAWN: u64 = 0x5350_4157_4e5f_5f5f;  // "SPAWN___"
const STREAM_WANDER: u64 = 0x5741_4e44_4552_5f5f; // "WANDER__"
const WANDER_RADIUS: i32 = 3;      // FR4's "~3 tiles", Chebyshev from home
const WANDER_REST_TICKS: u32 = 10; // one step per second at 10 ticks/s

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum JobState { Idle, Walk, Work }

#[derive(Component)]
struct Wander { home: Pos, cooldown: u32 }

#[derive(Resource)]
struct WanderRng(ChaCha8Rng);

/// The tile grid lives in the ECS so a system can read it; `World` delegates.
#[derive(Resource)]
struct Terrain { dims: Dims, tiles: Vec<Tile>, dirty: BTreeSet<Pos> }

impl Terrain {
    fn is_standable(&self, p: Pos) -> bool {
        matches!(self.tile(p), Some(Tile::Empty))
            && matches!(
                self.tile(Pos { z: p.z - 1, ..p }),
                Some(Tile::Solid(_) | Tile::Ramp(_))
            )
    }
}

fn wander(
    mut rng: ResMut<WanderRng>,
    terrain: Res<Terrain>,
    mut dwarves: Query<(&Id, &mut Pos, &mut Wander, &mut JobState)>,
) {
    // AD-7: query iteration is archetype order, not Id order, and all five dwarves draw
    // from ONE stream — so draw order is a sim outcome. Sort before touching the rng.
    let mut dwarves: Vec<_> = dwarves.iter_mut().collect();
    dwarves.sort_by_key(|(id, ..)| **id); // match ergonomics: `id` is `&&Id`

    for (_, mut pos, mut wander, mut state) in dwarves {
        if wander.cooldown > 0 {
            wander.cooldown -= 1;
            *state = JobState::Idle;
            continue;
        }
        let here = *pos;
        // NOTE: fixed order, same z only. Ramp climbing arrives with A* in Story 3.2.
        let candidates: Vec<Pos> = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .map(|(dx, dy)| Pos { x: here.x + dx, y: here.y + dy, z: here.z })
            .filter(|p| {
                (p.x - wander.home.x).abs() <= WANDER_RADIUS
                    && (p.y - wander.home.y).abs() <= WANDER_RADIUS
                    && terrain.is_standable(*p)
            })
            .collect();
        wander.cooldown = WANDER_REST_TICKS;
        match candidates.len() {
            // Reachable once 3.2's dig can seal a dwarf in; AC8 builds that case by hand.
            0 => *state = JobState::Idle,
            n => {
                *pos = candidates[rng.0.random_range(0..n)];
                *state = JobState::Walk;
            }
        }
    }
}
```

```rust
// crates/protocol/src/lib.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState { Idle, Walk, Work }

pub struct Entity { pub id: u32, pub kind: EntityKind, pub pos: [i32; 3], pub state: JobState }
```

```rust
// crates/tui/src/palette.rs — existing amber becomes Walk; idle sits back in the cold palette
pub fn entity_cell(kind: EntityKind, state: JobState) -> Cell {
    match (kind, state) {
        (EntityKind::Dwarf, JobState::Idle) => Cell { glyph: '☺', fg: (150, 112, 62) },
        (EntityKind::Dwarf, JobState::Walk) => Cell { glyph: '☺', fg: (214, 154, 78) },
        (EntityKind::Dwarf, JobState::Work) => Cell { glyph: '☺', fg: (236, 186, 96) },
    }
}
```

Verified against the vendored bevy_ecs 0.19.0 source: `bevy_ecs::system::{Query, Res, ResMut}`
all resolve (`system_param.rs:6` re-exports `Res`/`ResMut` from `change_detection`),
`Query::iter_mut` exists (`system/query.rs:716`). No new dependency — `rand`, `rand_chacha`
and `bevy_ecs` are already `sim-core` deps.

### Key decisions & traps

- **Terrain moves into the ECS because a system cannot see `World`'s private fields.** The
  alternative — calling `wander` outside the schedule — would break AD-7's single chained
  schedule at the second system it ever gets. Keep `World`'s public API byte-identical so
  2.1's tests are the regression net for the move.
- **The dwarves' draw order is a sim outcome.** One shared `WanderRng` plus unsorted query
  iteration is a determinism bug that the same-seed test cannot catch (identical worlds have
  identical archetype order); it surfaces later when 3.2 spawns items. Sort by `Id`.
- **Walk is a one-tick pulse.** A wander step is instantaneous, so a dwarf is `Walk` on the
  tick it moves and `Idle` for the nine it rests — in the TUI that reads as a warm blink once
  a second. Sustained `Walk` arrives with real paths in 3.2.
- **Candidates are never empty in a generated world** — the tile a dwarf just left is always
  standable and in radius, so dwarves cannot freeze. AC8 constructs the empty case with
  `set_tile` rather than leaving the branch untested.
- **This is the first wire change since 1.2.** `protocol`, the `simd` bridge, the pinned JSON
  literals and `tui`'s render call site move in one commit or the suite is red between them.
- **Spawn gets its own stream, closing the 1.1 deferred item.** Verified at
  `crates/sim-core/src/lib.rs:111-131`: one `ChaCha8Rng` threads `height_field` →
  `layered_terrain` → `spawn_dwarves`, and `layered_terrain` burns exactly `dims.x * dims.y`
  bool draws before spawn reads anything. So any later change to terrain's draw count moves
  all five dwarves for every seed, silently — red in a distant scenario test, with nothing at
  the change site to explain it. Worse, a pinned-positions test *without* the split degrades
  into a ritual: terrain changes, the pin goes red, the values get pasted over, the signal is
  trained away. Split + pin together, and a change to those literals is always suspicious.
  This is the last cheap moment: 2.4's `SaveState` gate test has not recorded a baseline yet.
- **Hand-off:** 2.4's `SaveState` must carry `WanderRng`, `Wander` and `JobState` (that is
  when `rand_chacha`'s `serde` feature is needed); 3.2 gets the first production `set_tile`
  caller — `set_tile` still has none outside tests today.

### Project Structure (files to touch)

```
crates/sim-core/src/lib.rs         # UPDATE — Terrain/WanderRng resources, JobState, Wander, wander system, dwarves()
crates/sim-core/tests/scenario.rs  # UPDATE — standable/near-home, same-seed, direction diversity, walled-in
crates/sim-core/tests/worldgen.rs  # UPDATE — destructuring only, for dwarves()'s third field (lines 18, 46, 190-208)
crates/protocol/src/lib.rs         # UPDATE — JobState, Entity.state, literals
crates/simd/src/bridge.rs          # UPDATE — job_state mapping + oracle test
crates/simd/tests/serve.rs         # UPDATE — movement and idle/walk observed on the wire
crates/tui/src/palette.rs          # UPDATE — entity_cell(kind, state) + pinned looks
crates/tui/src/view.rs             # UPDATE — pass entity.state; idle-vs-walk frame test
crates/tui/src/main.rs             # UPDATE — test literals only
_bmad-output/implementation-artifacts/mutations/2-2-dwarves-wander-the-frost.sh  # NEW
```

### Previous story intelligence (2.1)

- The client instrument already exists: `tui --frames N` runs the real reader-thread loop and
  exits. `--frame` (singular) returns before the reader thread is spawned and cannot show
  motion — do not reach for it and do not add a third mode.
- 2.1's review killed a test that passed with the fix removed. Every new assertion here goes
  through `scripts/mutate.sh` before you claim it works.
- `codex review --base main` has still never actually run (`CODEX_HOME` was outside the
  writable root; `codex-handoff.sh` now lists it, unverified). You may be the first run to
  prove it — say plainly whether it ran.

### Verification

```bash
scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/2-2-dwarves-wander-the-frost.sh
```

Live instrument — the observable outcome, with zero keys pressed (AC10, AC11):

```bash
cargo run -p simd &
cargo run -p tui -- --frames 30 > /tmp/wander.txt   # real client loop, 30 frames, exits
rg -n '☺' /tmp/wander.txt | head -20                # the glyphs sit on different columns/rows across frames
cargo run -p tui                                    # watch them step and blink amber, then q -> y
```

Branch: `2-2-dwarves-wander-the-frost`. Commit as `Völundr <jeicei75@gmail.com>`, one commit
per green step, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.2] — user story, source ACs, and
  the dependency-sweep `// NOTE:` naming this a wire change and the birthplace of the streams
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md]
  — AD-1, AD-4, AD-6 (vocabulary enums bridged by exhaustive match), AD-7 (chained schedule,
  stable `Id` order, purpose-named streams), AD-8 (entities are full resend), AD-9
- [Source: _bmad-output/planning-artifacts/epics.md#Requirements Inventory] — FR4, FR15, FR22,
  FR25, NFR2, NFR3
- [Source: _bmad-output/implementation-artifacts/2-1-the-world-runs-on-its-own-clock.md] — the
  schedule/dirty machinery, the `--frames` instrument decision, `bridge::delta`'s single-call
  discipline
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — the RNG-stream entry
  (corrected trigger) and the still-open status-line overflow
- [Source: AGENTS.md] — sabotage rule, honest reporting, the codex self-gate

## Dev Agent Record

### Agent Model Used

### Debug Log References

- Terrain RED (before implementation): `error[E0432]: unresolved import super::Terrain` at
  `crates/sim-core/src/lib.rs:248:38`; `cargo test --offline -p sim-core
  terrain_identifies_standable_tiles` failed to compile.
- Standability sabotage RED (below-tile check removed):
  `assertion failed: !terrain.is_standable(Pos { x: 1, y: 0, z: 1 })`; result:
  `FAILED. 0 passed; 1 failed`. The first sabotage attempt survived because its negative fixture
  was solid rather than unsupported empty; the fixture was corrected before completing the task.
- Spawn-stream TDD RED before the split showed the old coupled positions beginning
  `[Pos { x: 80, y: 54, z: 17 }, Pos { x: 49, y: 13, z: 20 }, ...]` instead of the placeholder
  literals. After the split, the test pinned the independently seeded positions and the full tile
  vector fingerprint `0xd03e1a262b9cc19d`.
- Spawn-stream sabotage RED (worldgen RNG fed back into `spawn_dwarves`): left
  `[Pos { x: 80, y: 54, z: 17 }, Pos { x: 49, y: 13, z: 20 }, Pos { x: 39, y: 41, z: 18 },
  Pos { x: 106, y: 42, z: 15 }, Pos { x: 82, y: 47, z: 16 }]`; right was the five pinned
  spawn-stream positions; result: `FAILED. 0 passed; 1 failed`.
- AC3 inverted sabotage (manual): added `let _ = rng.random::<bool>();` at the end of
  `layered_terrain`, then ran `cargo test --offline -p sim-core
  spawn_positions_for_seed_42_are_pinned`; result stayed GREEN: `1 passed; 0 failed`. Reverted the
  throwaway draw afterward.
- Wander API RED before implementation: unresolved import `super::JobState`, tuple arity mismatch
  (`expected tuple (Id, Pos), found tuple (_, _, _)`), and missing tuple field `.2`; the targeted
  test failed to compile with seven errors.
- `WANDER_REST_TICKS` sabotage RED (10 changed to 11): `wander_rest_is_ten_ticks` failed with
  `left: Idle`, `right: Walk`; result: `FAILED. 0 passed; 1 failed`.
- Spawn-state mapping sabotage RED (`JobState::Idle` changed to `JobState::Walk`):
  `assertion failed: before.iter().all(|(_, _, state)| *state == JobState::Idle)`; result:
  `FAILED. 0 passed; 1 failed`.
- `WANDER_RADIUS` boundary sabotage RED (3 widened to 6):
  `dwarves_stay_standable_and_near_home` panicked with `dwarf Id(0) escaped in y`; result:
  `FAILED. 0 passed; 1 failed`.
- Same-seed sabotage RED (successive worlds deliberately received different wander seeds): the
  first tick differed at `Id(0)`, left `Pos { x: 115, y: 85, z: 15 }`, right
  `Pos { x: 114, y: 84, z: 15 }`; result: `FAILED. 0 passed; 1 failed`.
- Random-choice sabotage RED (`random_range` replaced by constant candidate zero): the first
  two-vector assertion initially survived because the radius forced an x-axis reversal. The
  independent step-vector oracle was strengthened to require a y-axis step; it then failed with
  `constant candidate zero only bounced on the x axis: {(-1, 0, 0), (1, 0, 0)}`; result:
  `FAILED. 0 passed; 1 failed`.
- Walled-in sabotage RED (production standability filter removed): position changed from
  `Pos { x: 115, y: 84, z: 15 }` to `Pos { x: 115, y: 85, z: 15 }`; result:
  `FAILED. 0 passed; 1 failed`.
- Wire TDD RED before implementation: `protocol::Entity` rejected `state`, `JobState` was
  undeclared, the bridge oracle could not find `protocol::JobState`/`job_state`, and TUI fixtures
  could not import or construct the new field; the workspace failed compilation.
- Protocol state-name mapping sabotage RED (`Idle` renamed to `"rest"`): left `"rest"`, right
  `"idle"`; result: `FAILED. 0 passed; 1 failed`.
- Entity wire-order sabotage RED (`state` moved before `pos`): serialized left
  `{"id":7,"kind":"dwarf","state":"idle","pos":[4,5,6]}`, right
  `{"id":7,"kind":"dwarf","pos":[4,5,6],"state":"idle"}`; result:
  `FAILED. 0 passed; 1 failed`.
- JobState bridge-oracle sabotage RED (mapping hardcoded to `Idle`): left `Idle`, right `Walk`;
  result: `FAILED. 0 passed; 1 failed`.
- Snapshot field sabotage RED (snapshot entity state hardcoded `Idle`): left
  `[Idle, Idle, Idle, Idle, Idle]`, right `[Walk, Idle, Idle, Idle, Idle]`; result:
  `FAILED. 0 passed; 1 failed`.
- Live-daemon TDD/sabotage RED (wander consumed choices and emitted `Walk` but did not assign the
  chosen position): `streamed_deltas_show_wandering_positions_and_states` failed after 30
  consecutive deltas with `no entity position changed across 30 consecutive deltas`; result:
  `FAILED. 0 passed; 1 failed`.
- TUI render RED before state-aware palette implementation: idle and walking cells were both
  `Cell { glyph: '☺', fg: (214, 154, 78) }`; `assertion left != right failed`; result:
  `FAILED. 0 passed; 1 failed`.
- Palette mapping sabotage RED (`Walk` assigned idle RGB): left
  `Cell { glyph: '☺', fg: (150, 112, 62) }`, right
  `Cell { glyph: '☺', fg: (214, 154, 78) }`; result: `FAILED. 0 passed; 1 failed`.

### Completion Notes List

- Moved terrain dimensions, tiles, and dirty tracking into an ECS `Terrain` resource; preserved
  every public `World` signature and made dwarf candidate collection end its terrain borrow before
  spawning. The full offline workspace suite passed.
- Split dwarf placement from world generation with `STREAM_SPAWN`; pinned all five seed-42 spawn
  positions as literals and pinned the entire unchanged terrain vector with a deterministic
  fingerprint. The required extra-worldgen-draw experiment stayed green.
- Added the purpose-named wander stream, `JobState`, per-dwarf home/cooldown, ascending-ID wander
  system, explicit `(advance_tick, wander).chain()`, and the three-field sorted `dwarves()` API.
  Unit tests pin idle spawn, the ID stagger, and exactly ten rest ticks.
- Added the four required 200-step/enclosure scenarios. The standability oracle is written directly
  from `World::tile`, radius is a literal 3 in the test, same-seed worlds match on every step, the
  observed movement spans multiple vectors and axes, and a four-wall fixture remains idle.
- Added protocol `JobState` and ordered `Entity.state`, updated both hand-written wire formats,
  exhaustively bridged all three sim states into snapshots/deltas against a restated oracle, and
  updated every TUI wire/entity fixture forced by the new field.
- Added a bounded live-daemon integration test that consumes 30 consecutive deltas once each and
  proves an entity moves while both `Idle` and `Walk` cross the real TCP/protocol path.
- Made the TUI entity palette exhaustive over `(Dwarf, Idle|Walk|Work)`, pinned three distinct RGB
  values, passed wire state through `view::render`, and proved idle/walking cells differ in one
  rendered frame.

### File List

- `crates/sim-core/src/lib.rs`
- `crates/sim-core/tests/scenario.rs`
- `crates/sim-core/tests/worldgen.rs`
- `crates/simd/src/bridge.rs`
- `crates/simd/tests/serve.rs`
- `crates/protocol/src/lib.rs`
- `crates/tui/src/main.rs`
- `crates/tui/src/palette.rs`
- `crates/tui/src/view.rs`
- `_bmad-output/implementation-artifacts/2-2-dwarves-wander-the-frost.md`

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-03 | Story created |
| 2026-08-03 | Wolf's call: fold the 1.1 spawn/terrain RNG-coupling deferral into this story (AC2, AC3) rather than carrying it past 2.4's save baselines. |
