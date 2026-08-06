---
baseline_commit: c7a32fa
model: claude-opus-5[1m]  # default Opus; 1M-context variant chosen for a story spanning all four crates
---

# Story 3.2: The Dig

Status: in-progress

## Story

As the boss,
I want dwarves to claim dig orders in their own time, walk to the site, and dig,
so that the mountain yields stone at my command — through workers, not a remote control.

## Acceptance Criteria

1. `sim-core` gains `JobId(pub u32)`, `JobKind { Dig, Channel }` and
   `Job { id, kind, target: Pos, created_tick: u64, retry_after: u64 }`, plus a `Jobs` resource
   holding a `BTreeMap<JobId, Job>`, a `BTreeSet<Pos>` target index and `next_id: u32`, mutated only
   through its own `insert`/`remove` so the map and the index cannot drift. Job ids are their own
   space and never appear where an entity `Id` is expected (AD-9). Readers: `World::jobs()` ascending
   by `JobId`, `World::claims() -> Vec<(Id, Option<JobId>)>` ascending by `Id`, and
   `World::items() -> Vec<(Id, Pos)>` ascending by `Id`.
   **`IdAllocator` moves from a `World` field into the ECS as a `Resource`**, inserted by `assemble`
   and read by `to_save`. The single global allocator and its no-reuse-across-load guarantee are
   unchanged (AD-9); only its home moves — a system cannot spawn a stone with a sim-assigned id while
   the allocator lives outside the ECS.
2. **Diggability — this supersedes Story 3.1's AC2.** `apply_command`'s `Designate` arm records a
   mark only where that kind can be worked: `Dig` only on `Tile::Solid(_)`, `Channel` only where
   `is_standable` holds. Other tiles in the rect are silently not marked, exactly as `PlaceStockpile`
   already treats non-standable tiles; a rect with no workable tile changes nothing. 3.1 recorded
   every in-bounds tile and left a `// NOTE:` handing this rule here.
3. The mark set never exceeds `MAX_DESIGNATIONS = 4096` positions. Once full, a rect adds no new
   position but still overwrites the kind of positions already marked — so the arm `continue`s past a
   refused position rather than `break`ing out of the rect. New positions are taken in the existing
   `z`, then `y`, then `x` order, so the same command against the same world always yields the same
   mark set.
4. A `create_jobs` system turns every designated tile that has no job into exactly one `Job` of the
   matching kind, in ascending `Pos` order, stamped with the current tick. Running it on an unchanged
   world creates no second job for a tile. It lives in the chained schedule, so it does not run while
   paused — designation *intake* applies while paused (3.1 AC4), designation-*derived* work does not.
5. Exactly one claiming system, `claim_jobs`, at a fixed schedule point: jobs considered ascending by
   `JobId`, dwarves ascending by entity `Id`. A dwarf holding a job is not eligible; a claimed job is
   skipped. Dwarf `D` may claim job `J` only when `tick >= J.created_tick + reaction_delay(seed, D, J)`
   **and** `tick >= J.retry_after`. One job per dwarf, one dwarf per job (FR5, AD-12).
6. `reaction_delay(seed: u64, dwarf: Id, job: JobId) -> u64` returns 5..=30 from a hand-written,
   named FNV-1a over the three values — never `RandomState`, never an RNG stream, never wall clock
   (AD-7). The delays for seed 42, dwarves 0..=4 and job ids 0..=2 are pinned as literals in a test,
   so changing the hash is visible rather than silent.
7. `sim-core` gains a private A* over standable positions. Neighbours are the four horizontal steps
   at the same z, plus those four at z±1 when the tile beneath the **lower** of the two positions is
   a `Ramp` — the same convention `place_ramps` already builds (FR11, AD-5). Expansion is capped at
   `MAX_ASTAR_NODES = 50_000`; exceeding it yields no path. Ties break on `Pos`, so a path is a pure
   function of terrain and endpoints, identical across runs and processes.
8. A claimed dwarf takes one step per tick along its path reporting `JobState::Walk`, then holds
   `JobState::Work` for `WORK_TICKS = 5` at a work position before the job completes. Work positions
   are: for `Dig`, the standable tiles horizontally 4-adjacent to the target at the target's z; for
   `Channel`, the target tile itself. `wander` skips any dwarf holding a job, so a working dwarf
   never drifts; a dwarf is `Idle` exactly when it holds no job.
9. Completing a `Dig` sets the target from `Solid(m)` to `Empty`. Completing a `Channel` sets the
   tile **below** the target from `Solid(m)` to `Ramp(m)`. Both mutate through `World::set_tile`, so
   both appear in that tick's delta `tiles` — the first production producer of the AD-8 dirty set. A
   stone item is spawned at the target position with an id from the single global allocator (AD-9),
   and the completed job and its designation are both removed.
10. A `settle` system, chained after job execution and before `wander`, moves any dwarf whose current
    tile is no longer standable down one z when the tile below is standable, dwarves in ascending
    `Id`, and discards that dwarf's path so it re-paths. A dwarf standing on a tile dug out from
    under it is one level lower in the same tick and never renders hovering on air.
11. A dwarf that can reach no work position releases its claim and the job's `retry_after` becomes
    `tick + RETRY_COOLDOWN = 20`; the job stays in the market and is retried, never dropped (FR8).
    `CancelDesignation` over a rect removes the designations, removes any job on those tiles whether
    queued or claimed, and clears the `CurrentJob` of any dwarf holding one (FR9).
12. `protocol` gains `Item { id: u32, pos: [i32; 3] }` and an `items` section in both `Snapshot` and
    `Delta`, full-resend like `zones` — absence is deletion. `Item` carries no kind: stone is the
    only item in phase one, mirroring `Zone`'s existing `// NOTE:`. `Entity` is unchanged, so every
    pinned dwarf literal stays byte-identical. The hand-written literal `{"id":12,"pos":[1,2,3]}`
    decodes and re-encodes to the same JSON value, and `bridge::snapshot`/`bridge::delta` emit the
    world's real items.
13. `SaveState` gains `jobs`, `next_job_id` and `items` (all sorted ascending), and `SavedDwarf`
    gains `current_job: Option<u32>`. Paths are **not** saved — they are recomputed. The AD-11 gate
    test is extended: designate a dig, then step until `claims()` first shows a dwarf holding the job
    and step a few more so it is mid-walk (a stepped condition, never a magic tick number — the
    reaction delay is 5–30 ticks and a hardcoded tick would be seed-fragile), save, load, then tick
    the loaded world and a never-saved control 200 further times asserting equal `tick()`,
    `dwarves()`, `jobs()`, `claims()`, `items()`, `designations()` and `zones()` after **each** step.
14. TUI layer order is terrain → zones → designations → **items** → entities → pending rect →
    cursor. A stone renders as a pinned glyph absent from every existing table. Where two or more
    **dwarves** occupy one cell a distinct crowd glyph is drawn instead of either dwarf, so no dwarf
    is ever silently overwritten by a higher `Id` (reproduced at 2.2, seed 133). Every new look is
    distinguishable by glyph alone, so the view still carries the information under `NO_COLOR`.
15. `tui --frames N --key <sequence>` — the existing instrument, extended in no new direction —
    shows the whole outcome. Driven against a stub daemon that replays a dig, the capture must show
    **two glyph-level transitions at the target cell**: the designation glyph present in an early
    frame and absent in a later one, and the stone glyph absent early and present late. The identical
    run against a stub whose world never changes shows neither. The second test is the guard on the
    first. Assert on those two glyphs, **not** on the wall-to-floor change: `render` peeks up to
    `PEEK_DEPTH` levels below an `Empty` tile and redraws the tile it finds with `tile_cell`
    [view.rs:126-138], so digging stone that sits on stone re-renders the identical `█` in a dimmed
    colour — and `NO_COLOR` strips the only thing that differs. That is the 2.2 failure exactly.
16. `crates/sim-core/tests/scenario.rs` asserts headless, with no client and no network: designate →
    delay → claim → walk → dig, ending with the tile changed and a stone at the target; the
    unreachable case (a walled-off target — the job is still present after 200 ticks and no dwarf is
    stuck claiming it); and cancel-mid-dig (cancel while a dwarf holds the job — the job is gone, the
    dwarf is `Idle`, the tile unchanged). The same seed and command sequence replayed twice yields
    identical `dwarves()`, `jobs()`, `claims()`, `items()` and tiles.
17. `scripts/gate.sh` passes and `scripts/mutate.sh
    _bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh` reports zero survivors.

## Tasks / Subtasks

- [x] **`sim-core`: job market vocabulary, resource and readers** (AC: 1)
  - [x] Add `JobId`, `JobKind`, `Job`, `CurrentJob` and the `Jobs` resource to
        `crates/sim-core/src/lib.rs` (skeleton below). No new dependency.
  - [x] `Jobs` keeps its `BTreeMap<JobId, Job>` and `BTreeSet<Pos>` index private and exposes
        `insert`/`remove`/`get_mut`/`iter` — the pairing is enforced in one place, not at each call
        site. `BTree*`, not hash containers: AD-7 forbids iteration order affecting outcomes.
  - [x] `Jobs` and the new components go in through `assemble`, still the ONE assembly site, so
        `generate` and `from_save` cannot diverge.
  - [x] `CurrentJob(Option<JobId>)` is a component on every dwarf, present from spawn. **The dwarf
        owns the claim and it is the only source of truth** — do not also store `claimed_by` on the
        `Job`, or the two will drift and every later story pays for it.
  - [x] **Do this refactor FIRST, before anything else in the story.** `IdAllocator` is a plain field
        on `World` [lib.rs:244] and `allocate` is called only from `spawn_dwarves` [lib.rs:572]; a
        bevy system cannot reach it, so `execute_jobs` could not give a stone an id. Make it
        `#[derive(Resource)]`, insert it in `assemble` alongside the other resources, and repoint
        `to_save` [lib.rs:347] and `spawn_dwarves`. Small, mechanical, and every later task depends
        on it — the whole suite should be green again before you continue.

- [x] **`sim-core`: item entities** (AC: 1, 9)
  - [x] `#[derive(Component)] struct Item;` — the marker mirroring `Dwarf` [lib.rs:89-90]. A stone is
        `(Item, Id, Pos)`. No `ItemKind`, no components a stone does not have.
  - [x] `World::items()` filters on `Item` the way `dwarves()` filters on `Dwarf` [lib.rs:520-535],
        sorted ascending by `Id`.
  - [x] `to_save`'s dwarf filter is `entity.contains::<Dwarf>()`, so items are excluded from
        `dwarves` automatically — but re-read its `filter_map` NOTE [lib.rs:324-327] before adding a
        second entity kind, and give items their own save list rather than widening that one.

- [x] **`sim-core`: diggability and the mark budget** (AC: 2, 3)
  - [x] Filter in the existing `Designate` arm of `apply_command`; delete the `// NOTE: Story 3.2
        owns diggability` at `lib.rs:473` that this closes.
  - [x] `MAX_DESIGNATIONS` check in the same loop: `break` once the map is full and the position is
        not already present, so an over-large rect marks a deterministic prefix.
  - [x] **Existing tests will go red and that is expected, not a regression.** `save_load.rs`
        designates `Channel` at z=2 (solid rock — nothing is standable there) and several
        `scenario.rs` cases designate arbitrary positions. Repoint each to terrain the kind can
        actually work, using the existing `make_standable` helper [scenario.rs:9-20] and a
        `set_tile(.., Solid)` for dig targets. Do not weaken the filter to keep old tests green.
  - [x] Tests: dig marks only solid tiles of a mixed rect; channel marks only standable ones; a rect
        with no workable tile changes nothing; the 4097th distinct position is refused while
        re-designating an existing one still flips its kind.

- [x] **`sim-core`: designations become jobs** (AC: 4)
  - [x] `create_jobs` system: for each designation in ascending `Pos` with no entry in the target
        index, insert a `Job` with `created_tick = tick`, `retry_after = 0`.
  - [x] Add to the chained schedule — order is fixed and load-bearing (AC4, AC5, AC10):
        `(advance_tick, create_jobs, claim_jobs, execute_jobs, settle, wander).chain()`.
  - [x] Tests: N designated tiles yield N jobs with ascending ids; a second `step()` adds none; a
        paused daemon creates no jobs (assert via the schedule not running, i.e. `apply_command` then
        no `step()` leaves `jobs()` empty).

- [ ] **`sim-core`: claiming and the reaction delay** (AC: 5, 6)
  - [ ] `reaction_delay` as a named, hand-written FNV-1a — offset basis `0xcbf29ce484222325`, prime
        `0x100000001b3`, folding the LE bytes of seed, dwarf id and job id in that order, then
        `5 + (hash % 26)`. One function, no dependency, no RNG stream.
  - [ ] `claim_jobs`: build the claimed-`JobId` set from dwarves ascending `Id` (five entries), then
        walk `jobs.iter()` ascending `JobId`; for each unclaimed, eligible job assign the first
        eligible dwarf in ascending `Id`.
  - [ ] Tests: FIFO — with two jobs and one free dwarf the lower `JobId` is taken; a dwarf holding a
        job is skipped; a job is not claimed before `created_tick + delay`; the pinned delay table
        for seed 42 × dwarves 0..=4 × jobs 0..=2.

- [ ] **`sim-core`: A\*** (AC: 7)
  - [ ] Private `astar(terrain, from: Pos, goals: &BTreeSet<Pos>) -> Option<Vec<Pos>>` — a goal
        **set**, so one search serves all four work positions of a dig target rather than four.
  - [ ] `BinaryHeap<Reverse<(u32 /*f*/, Pos)>>` with `BTreeMap` for `came_from`/`g`; `Pos` in the
        heap key is what makes ties deterministic. Heuristic = Manhattan to the nearest goal.
  - [ ] Neighbours in the fixed order `[(-1,0), (1,0), (0,-1), (0,1)]`, matching `wander`
        [lib.rs:200]: same-z when standable; z±1 when standable **and** the tile below the lower of
        the two positions is a `Ramp`.
  - [ ] Tests: a straight corridor yields the shortest path; a ramp built by `place_ramps` is
        crossed; a walled-off goal yields `None`; the node cap yields `None` rather than scanning the
        world; the same query twice yields the identical path.

- [ ] **`sim-core`: walk, work, dig and channel** (AC: 8, 9)
  - [ ] `execute_jobs`: no path → compute one to the job's work positions (none reachable → release
        per AC11); path non-empty → take one step, `JobState::Walk`; at a work position → count
        `WORK_TICKS` with `JobState::Work`, then complete.
  - [ ] `Path(Vec<Pos>)` and `WorkProgress(u32)` components, attached on claim and removed on
        release/completion. **Neither is saved** (AC13) — `Path` is a pure function of terrain and
        endpoints, so a loaded world recomputes the identical one. `WorkProgress` IS saved, folded
        into the job as a field rather than a component if that reads simpler — pick one and say so.
  - [ ] Completion: `set_tile` (dig: target → `Empty`; channel: target-below → `Ramp(m)` keeping the
        material it had), spawn the stone `(Item, Id, Pos)` with `self.ids.allocate()`, then
        `jobs.remove(..)` and drop the designation.
  - [ ] `wander` gains one filter: skip dwarves whose `CurrentJob` is `Some`. Rewrite its
        `// NOTE:` at `lib.rs:187-191` — the resting-dwarf-on-mutated-terrain case is now handled by
        `settle`, not deferred.
  - [ ] Tests: state machine `Idle → Walk → Work → Idle` observed over one job; a dig turns the wall
        to `Empty` and puts a stone at the target with a fresh id; a channel turns the tile below
        into a `Ramp` of the same material; the dug tile appears in that tick's `drain_dirty`.

- [ ] **`sim-core`: settle, retry and cancel** (AC: 10, 11)
  - [ ] `settle` between `execute_jobs` and `wander`: ascending `Id`, if `!is_standable(pos)` and
        `is_standable(pos.z - 1)` then move down one and clear `Path`. One level per tick, no loop —
        a deeper shaft settles over successive ticks and that is deliberate.
  - [ ] Release path: clear `CurrentJob`, drop `Path`/`WorkProgress`, set `retry_after = tick +
        RETRY_COOLDOWN`. **Without the cooldown this livelocks**: eligibility is computed from
        `created_tick`, which does not change, so the same dwarf re-claims and re-runs A* every tick.
  - [ ] `CancelDesignation` in `apply_command` also removes jobs whose target is in the rect and
        clears the `CurrentJob` of any dwarf holding one. `RemoveStockpile` still touches zones only.
  - [ ] Tests: a dwarf standing on a dug tile is one z lower the same tick; a walled-off target's job
        is still present after 200 ticks and no dwarf is permanently stuck on it; cancelling while
        claimed leaves the job gone, the dwarf `Idle` and the tile unchanged.

- [ ] **`protocol` + `simd`: items on the wire** (AC: 12)
  - [ ] Add `Item` and the `items` field to `Snapshot` and `Delta`. Do not touch `Entity` — keeping
        it unchanged is why every existing pinned dwarf literal stays valid.
  - [ ] Extend `WIRE`, `DELTA_WIRE` and `every_material_and_tile_variant_has_a_pinned_wire_name`
        with the new shape. Literals, not round-trips.
  - [ ] `bridge.rs`: `World::items()` mapped in `snapshot()` and `delta()` alongside zones. Rewrite
        the amplification `// NOTE:` at `bridge.rs:74-80` — it now names the `MAX_DESIGNATIONS`
        bound instead of pointing forward to this story.
  - [ ] Live-daemon test in `crates/simd/tests/serve.rs` extending the existing `Daemon` harness: a
        designate over a solid tile, then read deltas until one carries a non-empty `tiles` and a
        non-empty `items` — the AD-8 dirty path proven end to end for the first time.

- [ ] **`sim-core`: jobs, claims and items survive save/load** (AC: 13)
  - [ ] `SaveState` gains `jobs: Vec<Job>`, `next_job_id: u32`, `items: Vec<(u32, Pos)>`;
        `SavedDwarf` gains `current_job: Option<u32>`. `Job`, `JobKind` and `JobId` derive
        `Serialize`/`Deserialize` like the other saved sim types.
  - [ ] Extend `crates/sim-core/tests/save_load.rs`'s gate test per AC13. Do not add a save-format
        literal test — format stability is a project non-goal.
  - [ ] `load_world` in `simd` already validates persisted mark positions; extend the same
        `in_bounds` check to job targets and item positions, matching the pattern at
        `crates/simd/src/main.rs:274-307`.

- [ ] **`tui`: stone, crowds and the layer order** (AC: 14)
  - [ ] `palette.rs`: `item_cell()` and `crowd_cell()`, both added to `every_look_is_pinned` and to
        its pairwise-distinct and no-collision assertions. Suggested `*` for stone and `⚇` for a
        crowd; exact RGB is yours, the distinctness is not optional.
  - [ ] `view.rs render`: draw `snapshot.items` after designations and before entities. For the
        entity layer, count dwarves per screen cell first, then draw `crowd_cell()` where the count
        exceeds one — do not draw a dwarf and then overwrite it.
  - [ ] Tests: an item renders on the viewed level only (the same z-guard entities already use
        [view.rs:170-178]); a dwarf standing on a stone shows the dwarf; two dwarves on one cell show
        the crowd glyph and neither `☺`; layer order extended in the existing
        `marker_layers_follow_...` test.

- [ ] **Observability instrument** (AC: 15) — extend `tui --frames N --key`; do not invent a second
      channel. The existing `--key` sequence parser and stub-daemon capture pattern already carry it
      [3.1 AC14, `crates/tui/tests/client.rs`].
  - [ ] Two tests driving the real binary against a stub daemon: the stub replays a wall tile
        becoming empty plus an item appearing, and the capture must show the cell's glyph **change
        between frames** and the stone glyph arrive. The control stub sends deltas whose world never
        changes and its capture must show neither.
  - [ ] The change-between-frames assertion is the point: 2.2 shipped an instrument that rendered
        motion as stillness and its evidence was an artefact. A capture that merely *contains* a
        stone glyph would pass even if the client drew it unconditionally.

- [ ] **Scenario harness** (AC: 16) — extend `crates/sim-core/tests/scenario.rs`, do not replace.
      Build the terrain the case needs with `make_standable` and `set_tile` rather than hunting for
      generated terrain that happens to suit.

- [ ] **Sabotage + mutation set** (AC: 17)
  - [ ] `_bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh`, at least: `Designate`
        ignores the diggability filter; ignores the `MAX_DESIGNATIONS` cap; `break`s out of the rect
        when full instead of `continue`ing; the stone takes its id from a second per-kind counter
        rather than the global allocator; `create_jobs` creates a
        duplicate job per tick; `create_jobs` runs on paused (moved out of the schedule);
        `claim_jobs` walks jobs descending; walks dwarves descending; ignores the reaction delay;
        ignores `retry_after`; claims a job already claimed; `reaction_delay` returns a constant;
        drops the job id from the hash; drops the dwarf id from the hash; A\* neighbour order
        reversed; A\* allows a z-change with no ramp below; A\* ignores the node cap; A\* ties break
        on insertion order; dig sets the wrong tile; dig writes the tile without `set_tile`; channel
        writes `Empty` instead of `Ramp`; channel loses the material; the stone is not spawned; the
        stone reuses an id; the completed job is not removed; the designation is not removed;
        `settle` moves up instead of down; `settle` does not clear the path; release skips the retry
        cooldown; an unreachable job is dropped; cancel leaves the job in place; cancel leaves
        `CurrentJob` set; `wander` moves a dwarf holding a job; `to_save` drops jobs; drops items;
        drops `current_job`; `from_save` discards them; `bridge` drops items from the delta; the
        `items` field is renamed on the wire; the crowd glyph is not drawn; items draw above
        entities.
  - [ ] `cargo clean -p sim-core -p protocol -p simd -p tui` before the final gate — `mutate.sh` is
        not concurrency-safe and 2.3, 2.4 and 3.1 all hit stale mutated binaries.
  - [ ] Paste the actual RED output for every new mapping/constant test into the Dev Agent Record
        (AGENTS.md rule 1).

- [ ] **Green gate** (AC: 17) — `scripts/gate.sh`, then the live check. Report what printed.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No haul job, no stockpile delivery, no carrying.** Stone is spawned and left where it falls.
  `JobKind` gets `Dig` and `Channel` only; 3.3 adds `Haul` as a variant plus its execution system and
  **must not touch claiming logic** (AD-12) — leave that seam clean.
- **No gravity beyond AC10's single level.** `settle` moves a dwarf down one z per tick when it has
  no floor. Items never fall, a dwarf never falls more than one level per tick, and nothing else in
  the world is support-checked. `// NOTE:` the limitation.
- **No occupancy rule, no tile reservation, no path re-planning around other dwarves.** Wolf's call
  2026-08-06: the 2.2 defect was that a dwarf silently *vanished from view*, and that is a rendering
  fault, fixed by AC14's crowd glyph. Dwarves may share a tile. Do not filter A\* on occupancy — it
  buys physical correctness at the price of a deadlock class this story cannot afford.
- **No hierarchical pathfinding, no path caching between jobs, no flow fields** (AD-5). One A\* per
  claim, recomputed when the path is missing.
- **No job priorities, no job cancellation UI, no per-dwarf skills or professions.** FIFO by job id
  is the whole scheduling rule.
- **No second claiming system, ever** (AD-12). One system, one schedule point.
- **No new wire message types.** Items ride an added section on the existing `snapshot`/`delta`.
- **No changes to `Entity`** — leaving it alone is what keeps every pinned dwarf literal valid.
- **No reconnect, no backpressure, no protocol optimization**, and no fix for the still-open
  `NO_COLOR` product-half, status-line-width, `MAX_SAVE_BYTES`-vs-world-size or SIGTERM items in
  `deferred-work.md` — none is assigned here.

### What already exists (build on it, do not re-derive)

- `assemble(seed, dims, tiles, tick, wander_rng, ids, designations, zones)` is the single
  world-assembly site; `generate` and `from_save` both go through it [lib.rs:250-279]. The schedule
  is built there too, `.chain()`ed [lib.rs:271-272].
- `Terrain` owns `dims`/`tiles`/`dirty` and already has `is_standable` and the `set_tile` →
  dirty-set path [lib.rs:119-169]. `World::set_tile`/`drain_dirty` delegate [lib.rs:416-422].
- `apply_command` already normalizes, clips and iterates `z`→`y`→`x` [lib.rs:427-501]; the
  `positions()` closure is the loop to filter, and both new rules go inside the existing `Designate`
  arm.
- `simd` drains commands at iteration top above the pause guard, then steps, then encodes one delta
  [main.rs:129-215]. `read_inbound` decodes any `protocol::Command`, so nothing new is needed there.
- `tui`'s `apply()` replaces wire sections wholesale each delta [main.rs:447-448] and `--frames
  N --key <seq>` already presses named keys through the real `apply_key` before streaming.
- Test scaffolding to extend: `make_standable` [scenario.rs:9-20], the hermetic `Daemon` harness
  [serve.rs:19-220], the stub-daemon + `strip_ansi` + `glyph_columns` capture pattern
  [client.rs:79-135], and 3.1's mutations file as the worked format.

### Code skeleton

```rust
// crates/sim-core/src/lib.rs — the job market. Job ids are their own space (AD-9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JobId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind { Dig, Channel }   // NOTE: Haul joins in Story 3.3 as a variant only.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub kind: JobKind,
    pub target: Pos,
    pub created_tick: u64,
    pub retry_after: u64,
}

/// The map and the target index are private so they can only move together.
#[derive(Resource, Default)]
struct Jobs {
    by_id: BTreeMap<JobId, Job>,
    targets: BTreeSet<Pos>,
    next_id: u32,
}

/// The ONLY record of who holds what. Present on every dwarf from spawn.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct CurrentJob(Option<JobId>);

#[derive(Component)]
struct Path(Vec<Pos>);          // transient: never saved, recomputed when missing

const MAX_DESIGNATIONS: usize = 4096;
/// Generous: the reachable component of a default world is ~16k standable tiles, so an
/// unreachable goal exhausts the frontier long before this. It is a hang guard, not a budget.
const MAX_ASTAR_NODES: usize = 50_000;
const WORK_TICKS: u32 = 5;
const RETRY_COOLDOWN: u64 = 20;

/// Exclusive: it spawns a stone (needs the id allocator and immediate visibility to the rest of
/// the tick) and mutates Terrain. `Commands` would defer the spawn to a sync point and put the
/// item's existence at the mercy of schedule internals — determinism is not worth that.
fn execute_jobs(ecs: &mut EcsWorld) { /* … */ }

/// FR5 / AD-7: a fixed NAMED hash, never RandomState, never an RNG stream.
fn reaction_delay(seed: u64, dwarf: Id, job: JobId) -> u64 { /* FNV-1a; 5 + (h % 26) */ }

/// Goal SET, so one search serves all four work positions of a dig target.
fn astar(terrain: &Terrain, from: Pos, goals: &BTreeSet<Pos>) -> Option<Vec<Pos>> { /* … */ }
```

```rust
// crates/sim-core/src/lib.rs — schedule order is load-bearing, not stylistic.
schedule.add_systems(
    (advance_tick, create_jobs, claim_jobs, execute_jobs, settle, wander).chain(),
);
```

```rust
// crates/protocol/src/lib.rs — one added section; Entity is untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item { pub id: u32, pub pos: [i32; 3] }
// Item deliberately carries no `kind`: stone is the only item in phase one and a
// single-variant enum is the abstraction YAGNI forbids — the same call Zone made.
// NOTE: a second item kind adds the field.
```

### Key decisions & traps

- **Four rulings from Wolf, 2026-08-06, are settled and are not open readings.** (1) The AD-8
  amplification is bounded by `MAX_DESIGNATIONS` in `sim-core` — not delta-encoding, not another
  deferral. (2) Occupancy is a **rendering** fix, not a reservation model. (3) Channel turns the
  supporting tile into a `Ramp`, so nobody ever loses support to a channel. (4) Support is handled by
  a real one-level fall, not by blocking the job.
- **A single channelled tile does nothing useful, and that is correct.** After `P.z-1` becomes a
  `Ramp`, `is_standable(P)` is still true — the ramp is what lets A\* cross one z-level to an
  adjacent column that is one lower. You channel a *run* to descend, exactly as in DF. Do not
  "fix" this by also emptying the tile; that is the rejected alternative.
- **Diggability moving into `apply_command` is a deliberate change to 3.1's contract**, sanctioned by
  the `// NOTE:` 3.1 left at `lib.rs:473`. It is also half the amplification answer: a surface dig
  rect now marks almost nothing. Existing tests break; repoint them, do not soften the rule.
- **The `IdAllocator` move is a prerequisite, not a nicety.** It is the one change that unblocks
  every other task, and it touches `assemble`, `to_save` and `spawn_dwarves` — code three stories
  have relied on. Do it first, get the suite green, commit, then build on it.
- **The dwarf owns the claim.** One source of truth. A `claimed_by` on the `Job` as well is the
  obvious next thought and it is wrong — the two drift the first time a dwarf is despawned or a job
  removed, and nothing fails loudly.
- **Only exposed faces are diggable, and that is correct.** A `Dig` job's work positions are the
  standable tiles adjacent to the target, so a tile buried inside rock has none and is unreachable
  until a neighbour is dug away. Designating a solid volume therefore yields one workable face that
  advances inward as each tile falls, which is exactly DF's behaviour and emerges free from AC8 +
  AC11. Do not add a special case for it; do add a scenario test that a two-deep dig eventually
  reaches the second tile.
- **`retry_after` is not optional.** Eligibility is computed from `created_tick`, which never
  changes, so without the cooldown a dwarf that fails to path re-claims and re-runs a 20,000-node A\*
  on the very next tick, forever. This is the single most likely way this story ships a hang.
- **The seam here is the schedule position, not a function call.** `create_jobs` and `claim_jobs`
  live in the chained schedule precisely so they do NOT run while paused, while `apply_command` stays
  outside it and does. A test that only checks jobs get created does not prove that line: assert the
  negative — `apply_command` while paused, no `step()`, `jobs()` empty; then one `step()` and the job
  exists.
- **AD-8's dirty path goes live here.** Every delta in Epics 1–2 carried `tiles: []` because
  `set_tile` had no production caller; `deferred-work.md` explicitly says not to read 2.1's AC6 as
  evidence that tile streaming works. The live-daemon test in AC12's task is the first real proof.
- **Determinism is the whole point of the tie-break.** `BinaryHeap` pops equal-`f` nodes in
  insertion-dependent order; putting `Pos` in the key is what makes the path reproducible. A
  `HashMap` for `came_from` is fine for lookups but use `BTreeMap` so no reviewer has to reason about
  whether it is ever iterated.
- **`Item` carries no kind on purpose.** `Zone` made the same call in 3.1 and left the NOTE; follow
  the local precedent rather than inventing an `ItemKind` enum with one variant.
- **Both `entity_cell` and the crowd rule are exhaustive-match territory** (AD-6). Count dwarves per
  cell before drawing; drawing then overwriting reintroduces exactly the 2.2 defect one layer up.
- **A story this size will likely span two Codex sessions.** Restate the RED evidence in the
  continuation handoff — Epic 1 lost TDD discipline at precisely that boundary. Commit at minimum
  once per completed task; on this story that is the recovery mechanism, not a style preference.
- **`mutate.sh` is not concurrency-safe.** Budget the `cargo clean -p …` step before the final gate;
  2.3, 2.4 and 3.1 each burned a cycle on a stale mutated binary.

### Project Structure (files to touch)

```
crates/sim-core/src/lib.rs          # UPDATE — job market, diggability + cap, A*, create/claim/execute/settle, readers
crates/sim-core/src/save.rs         # UPDATE — jobs, next_job_id, items, SavedDwarf.current_job
crates/sim-core/tests/scenario.rs   # UPDATE — repoint designations to workable terrain; the three AC16 scenarios
crates/sim-core/tests/save_load.rs  # UPDATE — repoint the channel rect; gate test covers jobs/claims/items
crates/protocol/src/lib.rs          # UPDATE — Item + items section + pinned literals
crates/simd/src/bridge.rs           # UPDATE — items in snapshot/delta; rewrite the amplification NOTE
crates/simd/src/main.rs             # UPDATE — load_world validates job targets and item positions
crates/simd/tests/serve.rs          # UPDATE — live dig: a delta carrying real tiles and items
crates/tui/src/palette.rs           # UPDATE — item_cell, crowd_cell, pinned and distinct
crates/tui/src/view.rs              # UPDATE — items layer, crowd counting, layer-order test
crates/tui/tests/client.rs          # UPDATE — the two instrument tests
_bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh   # NEW
_bmad-output/implementation-artifacts/deferred-work.md           # UPDATE — close the amplification and dirty-path items
```

### Previous story intelligence (3.1)

- 3.1's live-daemon pattern is the one to copy: decode in `read_inbound`, apply in the iteration-top
  drain, prove by asserting the *consequence* over several real deltas. Parser-level tests proved
  nothing there and will prove nothing here.
- 3.1 shipped `remove_stockpile` as a fourth world-mutating command while six planning documents
  still listed three. **Reconciled 2026-08-06, before this story starts** — the spine's AD-10 rule
  and message table, `docs/architecture.md`, `epics.md`'s AD-10 line and 3.1 text, and FR18 in both
  `epics.md` and the PRD. So AD-10 now reads four commands and agrees with the code; you are not
  looking at drift.
- Branch from current `main` (`c7a32fa`), which carries 3.1 plus the forge-process reconcile.

### Verification

```bash
scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh
```

Live instrument — the observable outcome, joining the two binaries no test can span. A dig takes the
reaction delay (5–30 ticks) plus the walk plus `WORK_TICKS`, so ask for enough frames:

```bash
cargo run -p simd &
cargo run -p tui -- --frames 200                              > /tmp/dig-live.txt
# In the capture: the dig mark appears, a ☺ turns Walk-coloured and moves toward it,
# the wall glyph at the target becomes open floor, and a * remains at that cell.
rg -c '×' /tmp/dig-live.txt      # marks placed
rg -c '\*' /tmp/dig-live.txt     # stone exists after the dig, and only after
cargo run -p tui                                              # d, move, Enter, move, Enter — watch it live
```

Then, interactively: designate a **channel** run of three tiles along flat ground and confirm a ramp
appears below and a dwarf can walk down it; cancel a dig mid-walk and confirm the dwarf returns to
idle wandering; and, with the daemon paused, confirm a mark still appears while no job is created
until you resume.

Branch: `3-2-the-dig`. Commit as `Völundr <jeicei75@gmail.com>`, at minimum one commit per completed
task, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.2] — user story and source ACs
- [Source: .../architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md] — AD-12 (one job
  market, FIFO by ascending job id, dwarves in ascending entity Id, job-kind stories add variants
  never claiming logic), AD-5 (plain A\*), AD-7 (chained schedule, stable id order, fixed named hash
  for the reaction delay), AD-8 (dirty tiles via `set_tile`, everything small full-resend), AD-9
  (one allocator, job ids a separate space), AD-10 (the pause line), AD-11 (`SaveState` carries jobs
  and claims)
- [Source: _bmad-output/implementation-artifacts/3-1-give-the-order.md] — `apply_command`'s shape,
  the diggability deferral, the wire-change pattern, the instrument's failure history
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — the AD-8 amplification
  measurement (owner: this story) and the inert dirty-tile path (revisit: this story)
- [Source: _bmad-output/implementation-artifacts/sprint-status.yaml] — the Epic 2 action items this
  story closes: occupancy AC, and the five size mitigations folded into the tasks above
- [Source: AGENTS.md] — sabotage rule, honest reporting, bounded I/O, the codex self-gate

## Dev Agent Record

### Agent Model Used

### Debug Log References

- `MAX_DESIGNATIONS` sabotage (`4096 -> 4097`):
  `designation_budget_refuses_new_tiles_but_updates_existing_tiles_after_them` failed at
  `scenario.rs:405` with `assertion failed: !world.designations().iter().any(|(pos, _)| *pos == extra)`.

### Completion Notes List

### File List

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-06 | Story created |
