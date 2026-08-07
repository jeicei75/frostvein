---
baseline_commit: bf1f5c0
model: claude-opus-5[1m]  # default Opus; 1M-context variant chosen for a story spanning sim-core, simd and tui
---

# Story 3.3: The Haul — and the Skeleton Walks

Status: review

## Story

As the boss,
I want dug stone carried to my stockpile, completing the loop I ordered,
so that the walking-skeleton sentence is true — live on screen and proven headless.

## Acceptance Criteria

1. `JobKind` gains a third variant `Haul { item: u32 }` (`u32`, matching `SaveState.items` and
   `SavedDwarf.current_job`, not `Id`). `Jobs` gains a private `haul_items: BTreeSet<u32>` index
   maintained by the same `insert`/`remove` as `targets`, so tile jobs stay unique by `target` and
   haul jobs are unique by `item`. Job ids come from one `Jobs::next_job_id()` using
   `saturating_add`, called by both creation systems. The item index is load-bearing rather than
   stylistic: `Jobs::insert` refuses a duplicate `target`, so keying a haul job by position would
   silently refuse a dig job on the tile a stone happens to sit on.
2. `Carrying(Option<u32>)` is a component on **every** dwarf from spawn, mirroring
   `CurrentJob(Option<JobId>)` — an optional component would sit in `to_save`'s required-component
   filter, which silently drops a dwarf missing one. `World::carrying() -> Vec<(Id, Option<u32>)>`
   ascending by `Id` joins `claims()` and `items()` as a reader; `dwarves()` keeps its three-tuple
   shape, so `bridge.rs` is untouched.
3. A stone is **stored** iff it is not carried and its position is a stockpile tile. A stone that is
   neither stored nor carried is **loose**.
4. `create_haul_jobs`, chained after `create_jobs` and before `claim_jobs`, creates exactly one
   `Haul` job per loose stone that has none, in ascending item id, whenever at least one stockpile
   tile exists; and removes any haul job whose stone has become stored, releasing its holder.
   Running it on an unchanged world creates no second job for a stone.
5. A **free** stockpile tile is a standable zone tile holding no stored stone — one stone per
   stockpile tile. Haul work positions: a carrying dwarf walks onto a free stockpile tile; a dwarf
   not carrying walks **onto** the stone's current position, but only while a free stockpile tile
   exists. With no free tile the goal set is empty at both legs, so the job is never claimed rather
   than claimed into a pick-up-and-drop cycle.
6. A haul job whose stockpile is full or gone stays queued and is retried, never dropped (FR8),
   matching 3.2's never-drop ruling; no dwarf holds it and none carries its stone. When a free tile
   appears the job becomes claimable again with no further intervention.
7. Claiming logic is unchanged (AD-12): `claim_jobs` keeps its FIFO-by-`JobId`, ascending-dwarf-`Id`,
   reaction-delay, `retry_after` and shared-`MAX_ASTAR_NODES` rules, and only `work_positions` learns
   `Haul`. A haul job's reaction delay comes from the existing `reaction_delay(seed, dwarf, job)`
   because its `JobId` is distinct.
8. Haul execution keeps the existing walk → `WORK_TICKS` of `JobState::Work` → effect shape. The
   effect when the dwarf is not carrying is **pickup**: `Carrying` is set, `WorkProgress` resets to
   0, `Path` is dropped and the job is *not* completed. The effect when carrying is **drop**: the
   stone's position becomes the dwarf's tile, `Carrying` clears, the job is removed and the claim
   released. A haul completion mutates no tile and removes no designation.
9. `carry_items`, last in the chained schedule, makes a carried stone's position equal its carrier's
   position at the end of every tick — whichever system moved the carrier.
10. A dwarf carries a stone only while holding that stone's haul job. `release_claim` drops any
    carried stone at the dwarf's current position, so every abnormal exit (job vanished, retry,
    cancel, retire) leaves a loose stone rather than one welded to an idle dwarf.
11. `SavedDwarf` gains `carrying: Option<u32>` and the `Haul` variant serializes its item. The AD-11
    gate test covers a mid-haul save — stepped until `carrying()` first shows a dwarf holding a
    stone, never a magic tick — then load, then tick the loaded world and a never-saved control 200
    further times asserting equal `tick()`, `dwarves()`, `jobs()`, `claims()`, `carrying()`,
    `items()`, `designations()` and `zones()` after **each** step.
12. `simd`'s `load_world` scopes its existing tile-job rules — unique `target`, a matching
    designation of the same kind, the `MAX_DESIGNATIONS` count cap — to `Dig`/`Channel` only. A
    `Haul` job must name an item present in the save, haul jobs are unique by item, and their count
    is bounded by the save's item count. A dwarf's `carrying` must name an existing item, no two
    dwarves may carry the same item, and a carrying dwarf's `current_job` must be that stone's haul
    job. Every rejection logs and leaves the daemon ticking.
13. `protocol` is **unchanged** — no new message, field, or enum variant — so every pinned wire
    literal stays byte-identical and `bridge.rs` needs no new arm. The only wire-visible effect of
    this story is a stone's `pos` moving.
14. Where a single dwarf shares a screen cell with one or more stones, `tui` draws a distinct pinned
    glyph instead of `☺`; two or more dwarves still win with the crowd glyph. Counting and drawing
    use the same entity filter, so they cannot disagree. Every look stays distinguishable by glyph
    alone under `NO_COLOR`. // NOTE: the glyph states co-location, which is a carry in every case
    the sim produces except a dwarf standing on a loose stone it does not hold.
15. `tui --frames N --key <sequence>` — the existing instrument, extended in no new direction —
    shows the whole outcome. Driven against a stub daemon replaying a haul, the capture must show
    **three glyph-level transitions**: the stone glyph present at the source cell early and absent
    later; the co-location glyph present in some middle frame and absent in the first; and the stone
    glyph absent at the stockpile cell early and present late. The identical run against a stub
    whose world never changes shows none of the three. The second test is the guard on the first.
16. `crates/sim-core/tests/scenario.rs` asserts the walking-skeleton sentence headless, with no
    client and no network: build a world from a seed, inject a dig designation and a stockpile
    placement, tick within a bounded budget, and assert the target tile changed, a stone was spawned,
    and that stone's final position is a stockpile tile with no job left in the market. The same seed
    and command sequence replayed twice yields identical `dwarves()`, `jobs()`, `claims()`,
    `carrying()`, `items()` and tiles (FR26, FR15).
17. Wolf watches the live daemon and client run the whole loop — designate, dig, carry, stockpile —
    and signs off that it holds the feel floor (NFR2) and that the icy-grim identity reads in motion
    (FR23, success criterion 2). This closes the FR23 motion half left open at the Epic 2 retro.
18. `scripts/gate.sh` passes and `scripts/mutate.sh
    _bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh` reports
    zero survivors.

## Tasks / Subtasks

- [x] **`sim-core`: the haul job variant and its index** (AC: 1)
  - [x] `JobKind::Haul { item: u32 }`. `Job`, `JobId` and `JobKind` keep their existing derives —
        `Job` stays `Copy`, so the payload must stay a scalar.
  - [x] `Jobs` gains `haul_items: BTreeSet<u32>`; `insert` and `remove` switch on `job.kind` and
        maintain exactly one index each. **`Jobs::insert` rejects a duplicate `target`** today
        [lib.rs:146-153], so indexing a haul job by position would silently refuse a dig job on the
        tile a stone happens to sit on. Index haul jobs by item, never by `target`.
  - [x] `Job.target` for a `Haul` is the stone's position at creation, kept only so `load_world`'s
        in-bounds check still applies to every job. **Execution and claiming always read the stone's
        live `Pos`, never `target`** — say so in a `// NOTE:` at the variant.
  - [x] `fn next_job_id(&mut self) -> JobId` on `Jobs` using `saturating_add`, replacing
        `jobs.next_id += 1` [lib.rs:176] and serving both creation systems. Closes the
        `jobs.next_id` plain-add item deferred at 3.2's review; a second allocation site is what
        makes it worth one function rather than two copies.
  - [x] Update `from_save`'s `debug_assert!` message [lib.rs:866] — saves are now validated for
        unique job ids, unique tile targets **and** unique haul items.

- [x] **`sim-core`: carrying** (AC: 2, 9, 10)
  - [x] `#[derive(Component)] struct Carrying(Option<u32>);` attached at **both** dwarf spawn sites
        [lib.rs:1180 and lib.rs:892], present on every dwarf whether or not it carries. Mirroring
        `CurrentJob` is deliberate: an optional component would have to be threaded through
        `to_save`'s `filter_map`, which **silently skips a dwarf missing any required component**
        [lib.rs:795-802] — every non-carrying dwarf would vanish from the save with nothing failing.
  - [x] `pub fn carrying(&self) -> Vec<(Id, Option<u32>)>` ascending by `Id`, built exactly like
        `claims()`. Then rewrite the stale `// NOTE: promote this tuple to a struct at the fourth
        field (Story 3.2 adds carried item)` above `dwarves()` [lib.rs:1113-1115] — the fourth field
        was deliberately not taken; a sibling reader keeps `dwarves()` and therefore `bridge.rs`
        untouched.
  - [x] `carry_items` exclusive system, **last** in the chain: for each dwarf with
        `Carrying(Some(id))`, set that item's `Pos` to the dwarf's `Pos`. Last, not after `settle`,
        so the end-of-tick invariant holds no matter which system moved the carrier.
  - [x] `release_claim` [lib.rs:473-497] drops a carried stone at the dwarf's current `Pos` and
        clears `Carrying`, alongside the `Wander::home` re-homing added by commit `db42285`. **This
        funnel is the whole reason the story has no stranded-stone class** — completion, no-op
        completion, a vanished job, a retry and cancel all pass through it.
  - [x] Tests: a carried stone's position tracks its carrier every tick including through a `settle`
        fall; releasing a claim mid-carry leaves the stone loose at the dwarf's tile; no dwarf ever
        has `Carrying(Some(_))` while `CurrentJob` is `None`.

- [x] **`sim-core`: haul jobs appear and retire** (AC: 3, 4, 6)
  - [x] `create_haul_jobs` exclusive system between `create_jobs` and `claim_jobs` — exclusive
        because retiring a claimed job calls `release_claim`. Schedule:
        `(advance_tick, create_jobs, create_haul_jobs, claim_jobs, execute_jobs, settle, wander,
        carry_items).chain()`.
  - [x] Create: for each stone in ascending `Id` that is loose and has no haul job, when
        `Zones` is non-empty, insert `Job { kind: Haul { item }, target: <stone pos>, created_tick:
        tick, retry_after: 0 }`.
  - [x] Retire: remove any haul job whose stone is stored, and `release_claim` its holder if it had
        one. The only reachable case is a stockpile placed over a loose stone while a dwarf walks to
        it — the holder is by definition not yet carrying, so nothing is dropped.
  - [x] Tests: N loose stones with a stockpile yield N jobs with ascending ids; a second `step()`
        adds none; **no stockpile ⇒ no haul jobs at all** (the negative that keeps every Epic-2 and
        3.2 scenario unchanged); placing a stockpile over a loose stone retires its job and idles
        its claimant; removing every stockpile leaves the job queued, unclaimed and its stone
        uncarried, and placing a stockpile again makes it complete.

- [x] **`sim-core`: work positions and execution** (AC: 5, 7, 8)
  - [x] `work_positions` grows the context it needs — proposed
        `fn work_positions(terrain: &Terrain, zones: &BTreeSet<Pos>, items: &BTreeMap<u32, Pos>,
        job: Job, carrying: Option<u32>) -> BTreeSet<Pos>`. `Dig` and `Channel` arms are unchanged
        and ignore the new arguments.
  - [x] `claim_jobs` gains `Res<Zones>` and an item query to build that map, and passes
        `carrying = None`: a claimable dwarf holds no job, and AC10 means it therefore carries
        nothing. Assert that rather than assuming it.
  - [x] `Haul` arm, carrying: the free stockpile tiles — standable zone tiles holding no **stored**
        stone, so a stone being carried across the pile by another dwarf never blocks a tile. Not
        carrying: `{ stone pos }` if `is_standable` **and** a free tile exists, else empty.
        Both legs read the same free-tile set; that shared gate is what keeps AC6 quiescent.
  - [x] `execute_jobs`: dispatch on `job.kind` **before** the existing
        `change: Option<(Pos, Tile)>` computation [lib.rs:576-600]. A haul must never reach the
        no-op-completion arm, and above all must never reach
        `Designations.0.remove(&job.target)` — a designation may legitimately exist at a stone's
        position and removing it would delete an order the player gave.
  - [x] Pickup sets `Carrying(Some(item))`, re-inserts `WorkProgress(0)`, drops `Path`, leaves the
        job claimed. Drop sets the stone's `Pos` to the dwarf's tile, clears `Carrying`, removes the
        job and calls `release_claim`. No `set_tile`, so no `clear_paths`.
  - [x] Tests: the state machine over one haul is `Idle → Walk → Work → Walk → Work → Idle` with
        exactly `WORK_TICKS` in each `Work` run; a stone whose tile is not standable leaves its job
        queued and retried; a stockpile whose every tile already holds a stone leaves the job queued
        and retried; the drop lands the stone on a zone tile and removes the job.

- [x] **`sim-core`: save/load** (AC: 11)
  - [x] `SavedDwarf.carrying: Option<u32>`; `to_save` reads the component, `from_save` restores it.
        `Path` stays unsaved and recomputed, as at 3.2.
  - [x] Extend `save_load.rs`'s `save_load_then_tick_matches_never_saved` per AC11 with a mid-haul
        save point reached by a stepped condition. Do not add a save-format literal test — format
        stability is a project non-goal.

- [x] **`simd`: the loader learns that a job need not have a designation** (AC: 12)
  - [x] The `JobKind` match at `main.rs:439-442` stops compiling; that is the point. Split the job
        gauntlet [main.rs:303-347, 438-452]: tile jobs keep unique-`target`, matching-designation
        and the `MAX_DESIGNATIONS` cap; haul jobs get unique-`item`, item-exists, and a count
        bounded by `save.items.len()`.
  - [x] Dwarf validation [main.rs:374-421] gains the three `carrying` rules. Do **not** validate that
        a carried stone's saved position equals its carrier's — `carry_items` re-establishes that on
        the next tick, so rejecting the save would refuse a file over something self-healing.
  - [x] Repoint the existing tests this re-scopes: `over_budget_job_save…` [serve.rs:561],
        `duplicate_job_target_save…` [:736], `job_without_matching_designation_save…` [:696],
        `job_with_mismatched_designation_kind_save…` [:714] must still hold for `Dig`/`Channel`.
  - [x] New rejection tests in the same shape: a haul job naming an absent item; two haul jobs on one
        item; more haul jobs than items; a `carrying` naming an absent item; two dwarves carrying one
        item; a carrying dwarf whose `current_job` is not that stone's haul job.
  - [x] Live-daemon test extending the `Daemon` harness: designate a dig and place a stockpile, then
        read deltas until one carries an item whose `pos` equals a zone `pos`. Copy the bounded-poll
        shape of `completed_dig_streams_dirty_tile_and_item_in_the_same_delta` [serve.rs:1178].

- [x] **`tui`: the carried stone becomes visible** (AC: 14)
  - [x] `palette.rs`: a `carrier_cell()` beside `item_cell` [palette.rs:98-103], added to
        `every_look_is_pinned` and to its pairwise-distinct and no-collision assertions. Suggested
        `☻` (it reads as the loaded twin of `☺`); exact RGB is yours, the distinctness is not
        optional — glyph alone must carry it, because this devpod sets `NO_COLOR=1`.
  - [x] `view.rs render`: build the per-cell item count in the same pass shape as `dwarf_counts`
        [view.rs:178-185], then in the single entity draw loop pick, in order: crowd glyph when the
        cell holds two or more dwarves, `carrier_cell()` when it holds one dwarf and one or more
        items, `entity_cell` otherwise. Count and draw over the **same** filter — the 3.2 review
        deferred a mismatch here (`dwarf_counts` counts only `EntityKind::Dwarf` while the draw loop
        crowd-glyphs any entity), and this rewrite is where it closes.
  - [x] `items_draw_only_on_the_viewed_level_and_under_dwarves` [view.rs:781-809] pins the old
        behaviour and **will go red — that is the story, not a regression.** Repoint it: a dwarf
        sharing a cell with a stone now draws `carrier_cell()`, and an item with no dwarf still
        draws `*`. Extend the layer-order test [view.rs:885-941] rather than replacing it.
  - [x] Tests: one dwarf + one stone on a cell draws the carrier glyph and neither `☺` nor `*`; two
        dwarves + a stone still draw the crowd glyph; a stone alone still draws `*`; off-screen and
        wrong-z items are still discarded before indexing.

- [x] **Observability instrument** (AC: 15) — extend `tui --frames N --key`; do not invent a second
      channel. Copy `capture_dig_replay` [client.rs:631-740] as the worked pattern.
  - [x] Stub replay: a stone at cell A with a dwarf elsewhere; then the dwarf on A with the stone's
        `pos` following it across two or three cells; then the stone at the stockpile cell B with the
        dwarf moved off B. Assert the three AC15 transitions by column, using `strip_ansi` and
        `glyph_columns_for` so the assertions survive `NO_COLOR`.
  - [x] The control stub sends deltas whose world never changes; its capture must show none of the
        three. **The change-between-frames assertion is the point**: a capture that merely *contains*
        a carrier glyph would pass even if the client drew it unconditionally, which is the 2.2
        false-evidence failure exactly.

- [x] **Walking-skeleton scenario** (AC: 16) — extend `crates/sim-core/tests/scenario.rs`, do not
      replace. Sculpt the terrain with `make_standable` [scenario.rs:11-20] and `set_tile` rather
      than hunting for generated terrain that happens to suit.
  - [x] Name it in the file's existing style, e.g.
        `designate_dig_stockpile_haul_and_the_stone_reaches_the_pile_headlessly`, mirroring
        `designate_delay_claim_walk_work_and_dig_complete_headlessly` [scenario.rs:560].
  - [x] Bound the loop with an `assert!(world.tick() < N)` guard rather than a fixed step count. Two
        reaction delays of 5..=30 plus two walks plus two `WORK_TICKS` runs is the budget to size
        against; 3.2's cross-map cases used 300–900.
  - [x] Extend `same_seed_and_commands_remain_deterministic` [scenario.rs:750] with the stockpile
        command and `carrying()`.

- [x] **Sabotage + mutation set** (AC: 18)
  - [x] `_bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh`, at
        least: `create_haul_jobs` runs with no stockpile present; creates a duplicate job per tick;
        walks items descending; indexes haul jobs by `target` instead of `item`; does not retire a
        stored stone's job; retires a job without releasing its holder; `next_job_id` wraps instead
        of saturating; haul work positions ignore standability; ignore stored-stone occupancy so two
        stones share a tile; drop the free-tile gate on the not-carrying leg so a job with nowhere
        to deliver is still claimed; use `job.target` instead of the stone's live position; a carrying dwarf
        is given the stone's tile as its goal instead of the stockpile; pickup completes the job;
        pickup does not reset `WorkProgress`; pickup does not clear `Path`; the drop does not move
        the stone; the drop removes a designation at `job.target`; `carry_items` is removed from the
        schedule; `carry_items` runs before `settle`; `release_claim` does not drop the carried
        stone; `to_save` drops `carrying`; `from_save` discards it; `load_world` accepts a haul job
        naming an absent item; accepts two dwarves carrying one item; applies the
        matching-designation rule to haul jobs; the carrier glyph is not drawn; the carrier glyph
        wins over the crowd glyph; item counting and drawing use different filters.
  - [x] `cargo clean -p sim-core -p protocol -p simd -p tui` before the final gate — `mutate.sh` is
        not concurrency-safe and 2.3, 2.4 and 3.1 all hit stale mutated binaries.
  - [x] Paste the actual RED output for every new mapping/constant test into the Dev Agent Record
        (AGENTS.md rule 1).

- [x] **Green gate and the live loop** (AC: 17, 18) — `scripts/gate.sh`, then the live check below.
      Report what printed, including the key sequence that actually worked and the glyph counts.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No wire change.** `protocol` is untouched: no `carrying` field on `Entity`, no `JobState::Haul`,
  no `kind` on `Item`, no new `Command`. Carrying is visible because a carried stone's `pos` moves
  and `tui` draws co-location. This is Wolf's call, 2026-08-07, over adding `Entity.carrying`, which
  would move every pinned literal in `protocol`, `bridge.rs` and `serve.rs` on the milestone-gate
  story.
- **No second claiming system, ever** (AD-12). `claim_jobs` keeps every ordering rule; only
  `work_positions` learns `Haul`.
- **No item gravity, no item stacking model, no containers, no materials.** A stone is still
  `(Item, Id, Pos)`. Two stones can share a non-stockpile tile and will render as one `*`; leave it
  and `// NOTE:` it.
- **No occupancy rule for dwarves, no tile reservation, no path re-planning around other dwarves**
  — Wolf's 3.2 ruling stands. The one-stone-per-stockpile-tile rule in AC6 is about *items*, not
  dwarves, and is a goal-set filter, not a reservation.
- **No job priorities, no haul-specific UI, no new keys or modes.** The modal machine is already
  complete for designate → dig → haul; hauling is autonomous.
- **No hierarchical pathfinding, no path caching between legs** (AD-5). The pickup and the delivery
  are two separate A* searches, and `Path` is dropped between them.
- **No fix** for the still-open `NO_COLOR` product-half, status-line-width, `MAX_SAVE_BYTES`-vs-world
  -size, SIGTERM, `--frame`-has-no-colour-warning or channel-orphan items in `deferred-work.md` —
  none is assigned here.

### What already exists (build on it, do not re-derive)

- `execute_jobs` is already exclusive and already has the walk → work → effect shape this story
  reuses [lib.rs:519-613]; `settle` and the `Path`/`WorkProgress` lifecycle are 3.2's and unchanged.
- `release_claim` [lib.rs:473-497] is the single funnel out of holding a job — it clears
  `CurrentJob`, sets `Idle`, re-homes `Wander::home` (commit `db42285`) and drops `Path`/
  `WorkProgress`. Every new exit path this story adds must go through it.
- `astar(terrain, from, goals: &BTreeSet<Pos>)` [lib.rs:447-450] already takes a goal **set** and
  returns the path to the nearest, so "walk to any free stockpile tile" is one call. It returns
  `None` on an empty goal set before spending any node budget.
- `Zones(BTreeSet<Pos>)` [lib.rs:289-290] is already exactly the shape `astar` wants for goals. Its
  tiles are validated standable at command time only and never re-checked, so filter on
  `is_standable` when building haul goals.
- `Item` marker, `World::items()` [lib.rs:1101-1111] and `protocol::Item` all shipped at 3.2, and
  `tui`'s `apply()` already replaces the `items` section wholesale each delta.
- Test scaffolding to extend: `make_standable` [scenario.rs:11-20], the hermetic `Daemon` harness
  [serve.rs:49-270], `capture_dig_replay` + `strip_ansi` + `glyph_columns_for`
  [client.rs:81, 592, 631], and 3.2's mutations file as the worked format.

### Code skeleton

```rust
// crates/sim-core/src/lib.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    Dig,
    Channel,
    // NOTE: `item` is the identity; `Job.target` for a Haul is only the stone's position at
    // creation, kept so load validation can bounds-check every job. Claiming and execution read
    // the stone's live `Pos` — never `target`, which is stale the moment the stone is picked up.
    Haul { item: u32 },
}

#[derive(Resource, Default)]
struct Jobs {
    by_id: BTreeMap<JobId, Job>,
    targets: BTreeSet<Pos>,      // Dig/Channel only — uniqueness by tile
    haul_items: BTreeSet<u32>,   // Haul only — uniqueness by stone, and the reservation
    next_id: u32,
}

/// Present on every dwarf from spawn, exactly like `CurrentJob`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct Carrying(Option<u32>);

/// Exclusive: retiring a stored stone's job may need `release_claim`.
fn create_haul_jobs(ecs: &mut EcsWorld) { /* … */ }

/// Exclusive, last in the chain: end-of-tick invariant, whoever moved the carrier.
fn carry_items(ecs: &mut EcsWorld) { /* … */ }

fn work_positions(
    terrain: &Terrain,
    zones: &BTreeSet<Pos>,
    items: &BTreeMap<u32, Pos>,
    job: Job,
    carrying: Option<u32>,
) -> BTreeSet<Pos> { /* Dig and Channel arms unchanged */ }
```

```rust
// crates/sim-core/src/lib.rs — order is load-bearing, not stylistic (AC4, AC7, AC9).
schedule.add_systems(
    (advance_tick, create_jobs, create_haul_jobs, claim_jobs,
     execute_jobs, settle, wander, carry_items).chain(),
);
```

### Key decisions & traps

- **Two rulings from Wolf, 2026-08-07, are settled and are not open readings.** (1) One stone per
  stockpile tile — chosen over unlimited stacking because with stacking every stone routes to the
  same nearest tile and five hauls render as one `*`, so the milestone demo would look like the loop
  ran once. (2) Carrying is shown by a client-side co-location glyph, not by a wire field.
- **The haul index is the reservation.** One job per stone, one dwarf per job, so nothing else is
  needed to stop two dwarves converging on one rock. Do not add a `claimed_by` on the item — the
  same drift argument that kept the claim solely on the dwarf at 3.2 applies here.
- **`Jobs::insert` keys tile jobs on `Pos` and silently returns `false` on a duplicate.** Indexing
  haul jobs by position would therefore make a dig designation on the tile a stone sits on silently
  never become a job. This is the single sharpest trap in the story.
- **A haul completion must not touch `Designations`.** The dig path removes the designation at
  `job.target`; a haul's `target` is a stone position where a real, unrelated designation may exist.
  Dispatch on kind before that block, not inside it.
- **`Carrying` is `Option` on every dwarf, not an optional component.** `to_save`'s `filter_map`
  skips a dwarf missing any required component [lib.rs:795-802] — an optional `Carrying` in that
  required set would silently drop every non-carrying dwarf from the save, and nothing would fail.
- **The seam here is the schedule position and the funnel, not a function call.** `create_haul_jobs`
  sits inside the chained schedule so it does not run while paused (3.2's designation-intake-yes,
  designation-derived-work-no line extends to stones unchanged). `release_claim` is the one place a
  stone is dropped. Assert the negatives: paused + a stone + a stockpile ⇒ no haul job; a released
  claim mid-carry ⇒ a loose stone at the dwarf's tile.
- **`work_positions` is the AD-12 seam and it is heavier than 3.2's guardrail predicted** —
  flagged by 3.2's acceptance auditor. `claim_jobs` now computes work positions and paths at claim
  time, so a `Haul` variant is not a pure `JobKind` addition; it needs `Zones` and item positions in
  the claiming system. This is budgeted, not a discovery. Claiming *logic* still does not change.
- **Never cache the destination tile.** `work_positions` is recomputed every tick inside
  `execute_jobs` [lib.rs:541] and that is what makes two carriers converging on one free tile
  self-healing: the moment the first drops, the tile leaves the second's goal set and it repaths.
  Storing a chosen destination on the job or the dwarf reintroduces the race.
- **A stone on an unsupported tile is unreachable and its job retries forever.** Items never fall
  [lib.rs:639], so a stone whose floor is dug away has no standable position. This is the same shape
  as the channel-orphan case Wolf ruled on at 3.2 — retry is nearly free and the never-drop rule
  wins. `// NOTE:` it; do not add detection.
- **The instrument's stub must move the dwarf off the stockpile cell** before the final frames, or
  the assertion is about the carrier glyph rather than the stone arriving. In the live sim the
  dwarf drops and then wanders (re-homed by `release_claim`), so the stone becomes visible within a
  few ticks — expect a brief carrier glyph at the pile before the `*` settles.
- **Colour cannot be evidence in this devpod.** `NO_COLOR=1` strips every SGR sequence; the carrier
  glyph must be readable without it. Use `.env_remove("NO_COLOR")` only where colour is genuinely
  the assertion, as `capture_dig_replay` does [client.rs:643].
- **`mutate.sh` is not concurrency-safe.** Budget the `cargo clean -p …` step before the final gate;
  2.3, 2.4, 3.1 and 3.2 each burned a cycle on a stale mutated binary.
- **Commit at minimum once per completed task.** On a story this size that is the recovery
  mechanism, not a style preference; if it spans two Codex sessions, restate the RED evidence in the
  continuation handoff.

### Project Structure (files to touch)

```
crates/sim-core/src/lib.rs          # UPDATE — Haul variant + index, Carrying, create_haul_jobs,
                                    #          carry_items, work_positions, execute/release paths
crates/sim-core/src/save.rs         # UPDATE — SavedDwarf.carrying
crates/sim-core/tests/scenario.rs   # UPDATE — the walking skeleton, haul lifecycle, determinism
crates/sim-core/tests/save_load.rs  # UPDATE — mid-haul gate test, carrying() in the assertions
crates/simd/src/main.rs             # UPDATE — load_world: tile-job vs haul-job rules, carrying
crates/simd/tests/serve.rs          # UPDATE — repointed + new save rejections; live haul delta
crates/tui/src/palette.rs           # UPDATE — carrier_cell, pinned and distinct
crates/tui/src/view.rs              # UPDATE — cell contention rule, repointed occlusion test
crates/tui/tests/client.rs          # UPDATE — the haul instrument tests
_bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh   # NEW
_bmad-output/implementation-artifacts/deferred-work.md   # UPDATE — close the next_id and
                                                         #          crowd-filter items
crates/protocol/src/lib.rs          # UNCHANGED — deliberately; see AC13
crates/simd/src/bridge.rs           # UNCHANGED — deliberately; items already bridged
```

### Previous story intelligence (3.2)

- **The one defect 3.2 shipped was found by Wolf playing, not by any of four review layers**
  (`db42285`): a dwarf that walked to a distant job could never wander again because `Wander::home`
  was written once at spawn. The fix put the re-home in `release_claim`. This story adds the second
  thing that funnel must do, and it is the same class of bug — state left behind on a dwarf that
  stopped holding a job.
- 3.2's live-daemon and stub-capture patterns are the ones to copy; parser-level tests proved
  nothing at 3.1 and will prove nothing here. Assert the *consequence* over several real deltas.
- Branch from current `main` (`bf1f5c0`), which carries 3.2 plus the stranding fix.

### Verification

```bash
scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh
```

Live instrument — the observable outcome, joining the two binaries no test can span. The full loop
is two reaction delays (5–30 ticks each) plus two walks plus two `WORK_TICKS` runs, so ask for
plenty of frames:

```bash
cargo run -p simd -- 47411 &
# Place the stockpile FIRST on open low ground, then dig into the face beside it — both at the
# same z, so the stone never needs a ramp. Adjust the cursor moves against the real terrain and
# record the sequence that worked.
cargo run -p tui -- 47411 --frames 400 \
  --key '<,p,enter,l,l,enter,esc,d,enter,l,l,l,l,l,l,j,j,j,enter' > /tmp/haul-live.txt
rg -c '≡' /tmp/haul-live.txt     # stockpile placed
rg -c '×' /tmp/haul-live.txt     # dig marks
rg -c '☻' /tmp/haul-live.txt     # a dwarf carrying a stone
rg -c '\*' /tmp/haul-live.txt    # loose and stored stone
for p in $(pgrep -x simd); do kill $p; done   # NEVER pkill -f 'target/debug/simd' — it kills your own shell
cargo run -p tui                              # then interactively, for AC17
```

Three things this recipe gets right that the obvious one gets wrong, carried from 3.2's review:
`--key` is required (without it no command is ever sent and the capture is empty however many frames
you ask for); `<` must come first (the opening view level is air, where the diggability filter
correctly marks nothing); and do not look for the wall becoming floor (`render` peeks below an
`Empty` tile and redraws a near-identical glyph, which `NO_COLOR` then strips). Assert on the
`≡`, `×`, `☻` and `*` glyphs.

Key names are a fixed comma-separated set — `space, +, -, S, L, d, c, p, x, h, j, k, l, enter, esc,
<, >`. There is no repeat shorthand and there are no arrow keys; write `l,l,l` out.

Then, interactively, for Wolf's AC17 sign-off: designate a run of several dig tiles and watch the
stockpile fill one tile at a time; cancel a designation mid-haul and confirm the dwarf keeps
delivering the stone it already carries; remove the stockpile while a dwarf is carrying and confirm
it drops the stone where it stands rather than freezing; and confirm the whole loop keeps pace at
10 ticks/sec with no visible stutter.

Branch: `3-3-the-haul-and-the-skeleton-walks`. Commit as `Völundr <jeicei75@gmail.com>`, at minimum
one commit per completed task, imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.3] — user story and source ACs; FR5,
  FR7, FR8, FR11, FR12, FR15, FR23, FR25, FR26
- [Source: .../architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md] — AD-12 (one
  job market; job-kind stories add variants and execution systems, never claiming logic), AD-5
  (plain A*), AD-7 (chained schedule, stable id order), AD-8 (dirty tiles, everything small
  full-resend), AD-9 (one allocator, job ids a separate space), AD-11 (`SaveState` carries jobs and
  claims)
- [Source: _bmad-output/implementation-artifacts/3-2-the-dig.md] — the job market, A*, execution and
  release machinery this story extends, and its four deferred items
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — `jobs.next_id` plain add and
  the crowd count/draw filter mismatch, both closed here
- [Source: _bmad-output/implementation-artifacts/sprint-status.yaml] — the open Epic 2 action item
  "Re-check FR23's motion sign-off at Story 3.3" (owner: Wolf), closed by AC17
- [Source: AGENTS.md] — sabotage rule, honest reporting, bounded I/O, the codex self-gate

## Dev Agent Record

### Agent Model Used

`claude-opus-5[1m]` — implemented **directly**, not delegated to Codex. Wolf's call, 2026-08-07:
the Codex weekly quota is exhausted until Wednesday next week, so the standing delegation protocol
in `_bmad/custom/bmad-dev-story.toml` fell back to direct implementation as its point (7) allows.
One commit per completed task, eleven commits, each with a green `scripts/gate.sh` behind the
pre-commit hook.

### Debug Log References

RED evidence, in the order it was observed.

**Task 1 — the variant and the allocator did not exist** (`cargo test -p sim-core --lib jobs_index_haul`):

```
error[E0599]: no variant named `Haul` found for enum `JobKind`
error[E0609]: no field `haul_items` on type `Jobs`
error[E0599]: no method named `next_job_id` found for struct `Jobs` in the current scope
error: could not compile `sim-core` (lib test) due to 8 previous errors
```

**Task 4 — behavioural red before haul execution existed.** The two-leg state machine test written
first, run against the interim stub that dispatched a haul to `continue`:

```
assertion `left == right` failed: two walks and exactly WORK_TICKS of work in each of the two legs
  left: [Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle, Idle]
 right: [Walk, Walk, Work, Work, Work, Work, Work, Walk, Walk, Walk, Work, Work, Work, Work, Work, Idle]
```

**Task 7 — the two `tui` tests the story predicted would go red, went red** on the new cell
contention rule, before either was repointed:

```
test view::tests::items_draw_only_on_the_viewed_level_and_under_dwarves ... FAILED
test view::tests::marker_layers_follow_terrain_zone_designation_item_entity_pending_cursor_order ... FAILED
assertion `left == right` failed
  left: '☻'
 right: '☺'
```

**Task 10 — the mutation table, 31 sabotages, all killed** (`scripts/mutate.sh …3-3….sh`, run from
a `cargo clean -p` of the four packages). Representative kills:

```
=== create_haul_jobs walks the stones descending ===
thread 'tests::create_haul_jobs_makes_one_job_per_loose_stone_in_ascending_item_order' panicked
=== next_job_id wraps instead of saturating ===
assertion `left == right` failed: a saturated allocator must repeat its last id, never wrap onto a reusable one
=== the pick-up leg uses job.target instead of the live position ===
thread 'tests::haul_execution_reads_the_stones_live_position_not_the_jobs_target' panicked
=== the drop removes a designation at job.target ===
assertion `left == right` failed: a haul completion removed a designation at its stale target
=== load_world applies the matching-designation rule to haul jobs ===
thread 'a_mid_haul_save_loads_and_the_daemon_keeps_ticking' panicked
=== the carrier glyph is never drawn ===
thread 'view::tests::items_draw_only_on_the_viewed_level_and_a_shared_cell_draws_the_carrier' panicked

All mutations killed.
```

Three mutations survived the first run and were fixed by strengthening the tests, not by weakening
the sabotage — recorded because each was a real hole:

- *free stockpile tiles ignore standability* survived because the test asserted only that nobody
  **carried**; the dwarf had claimed the job and was still walking after 80 ticks. The test now
  asserts the job is never **claimed**, which is AC6's actual shape and lands within a few ticks.
- *the pick-up leg uses `job.target`* survived twice. First because a two-tile displacement measured
  in Manhattan distance can be one step in Chebyshev; then because `execute_jobs` only recomputes a
  path when the cached one runs out, so a carrier whose goal set empties **finishes the walk it is
  on** and drops at the end of it — which in the original fixture was the pile tile itself, making
  the rest of the scenario vacuous. It is now pinned by a dedicated unit test that inserts a haul
  job with a deliberately stale target; with `target` substituted for the live position the stone
  teleports to meet the dwarf and the test dies.
- *the drop does not move the stone* twice failed to apply, because the same `Pos` write appears in
  both `release_claim` and `carry_items`; the sabotage now anchors on the surrounding `if let`.

**Live loop** — daemon and client, real binaries, `seed 0xF005_7E1A`, port 47411. The recipe in
Verification needed two corrections against the real terrain and they are worth recording:

1. At the opening view level (the dwarf's own z, 20) **every** cell is `Empty`, so `render` draws the
   dimmed peek of the terrain below and the dig filter correctly marks nothing: `× 0` for any rect.
   Digging happens one level DOWN (`<`), where the surface is solid.
2. A stockpile cannot go on that lower level *first* — the surface there is `Solid`, so
   `PlaceStockpile`'s standability filter drops every tile. Two client runs against the one running
   daemon: dig a 7×7 at z 19, then place the pile over the same area once it is dug floor.

```bash
cargo run -q -p simd -- 47411 &
# run A — dig a 7x7 one level down, at fast speed
cargo run -q -p tui -- 47411 --frames 400 \
  --key '<,+,d,h,h,h,k,k,k,enter,l,l,l,l,l,l,j,j,j,j,j,j,enter,esc'      # × 3384   * 6614
# run B — the pile goes on the dug floor, same rect
cargo run -q -p tui -- 47411 --frames 400 \
  --key '<,+,p,h,h,h,k,k,k,enter,l,l,l,l,l,l,j,j,j,j,j,j,enter,esc'      # ≡ 4151  ☻ 1033  * 7923
for p in $(pgrep -x simd); do kill $p; done
```

Run B, per frame rather than per capture, is the evidence: **28 distinct cells showed the stockpile
glyph early, and 21 of them hold a stone in the final frame** — the pile filling one tile at a time.
A carrier glyph appears in 365 of the 400 frames, first at frame 26. The whole run was at `fast`
speed with no visible stutter and the status line kept advancing (`tick 21428 … dwarves 5`).

`NO_COLOR` was NOT set in this session's shell — the captures carry real SGR sequences — so unlike
2.2's devpod the colour half of the instrument was live here. The glyph assertions in the suite are
still colour-independent on purpose.

### Completion Notes List

- **AC17 is NOT satisfied and cannot be by me.** It is Wolf watching the loop and signing off on the
  feel floor (NFR2) and the icy-grim identity in motion (FR23). Everything it needs is in place and
  the live run above is the invitation, not the sign-off. The sprint action item "Re-check FR23's
  motion sign-off at Story 3.3" stays open until Wolf answers.
- Every other AC is implemented and covered: 1–13 in `sim-core`/`simd`, 14 in `tui`, 15 by the stub
  replay pair, 16 by the headless walking-skeleton scenario, 18 by a green gate from a clean build
  plus a 31/31 mutation table.
- `protocol` and `bridge.rs` are untouched, as AC13 requires. `dwarves()` kept its three-tuple shape
  and `carrying()` is a sibling reader, so no pinned wire literal moved.
- **Two traps found that the story had not named**, both fixed here:
  - `CancelDesignation` matched jobs by `target` for every kind, so cancelling marks over the tile a
    stone happens to sit on would silently delete that stone's haul job — the same shape as the
    designation trap AC8 warns about, one layer up. It is now scoped to tile jobs, with a scenario
    test and a mutation.
  - A `Query` naming `&Carrying` skips a dwarf that lacks the component, exactly as `to_save`'s
    `filter_map` does. `from_save` therefore had to attach `Carrying` in the same commit that
    introduced it, not in the later save/load commit, or every loaded dwarf would have gone
    invisible to `claim_jobs` and `carry_items` with nothing failing.
- **One behaviour worth a reviewer's eye, deliberately left alone:** `execute_jobs` recomputes
  `work_positions` every tick but only recomputes a PATH when the cached one is exhausted. A carrier
  whose goal set empties mid-walk therefore finishes its current walk before retrying and dropping.
  It cannot drop in the wrong place — the effect requires standing on a current work position — so
  the invariant holds and the story's "never cache the destination" rule is satisfied; only the
  reaction is a walk late. Named in a `// NOTE:` in the scenario test.
- Two lines in the story's mutation list are documented in the mutations file as **not
  independently killable**, with reasons, rather than left as survivors: the pick-up leg's
  `is_standable` (a non-standable stone is unreachable either way, so the line only skips a doomed
  A* search) and `remove::<Path>()` at pick-up (the pick-up leg has a single goal, so the path is
  always already exhausted on arrival).
- `deferred-work.md`: the `jobs.next_id` plain-add item and the crowd count/draw filter mismatch are
  both closed here, and marked as such in that file.

### File List

- `crates/sim-core/src/lib.rs` — `JobKind::Haul`, `haul_items`, `next_job_id`, `Carrying`,
  `carrying()`, `create_haul_jobs`, `carry_items`, `uncarried_stones`, `item_entity`,
  `work_positions` haul arm, `claim_jobs` context, haul execution, `release_claim` drop,
  cancel scoping, save/load fields, and the new unit tests
- `crates/sim-core/src/save.rs` — `SavedDwarf.carrying`
- `crates/sim-core/tests/scenario.rs` — walking skeleton, no-work-while-paused, cancel-keeps-haul,
  remove-and-replace-the-pile, the no-teleport step helper, determinism extended with `carrying()`
- `crates/sim-core/tests/save_load.rs` — mid-haul save point by stepped condition, `carrying()` in
  the AD-11 comparison, mid-haul round trip, `SavedDwarf` literals
- `crates/simd/src/main.rs` — tile-job vs haul-job load rules, `carrying` validation
- `crates/simd/tests/serve.rs` — six haul/carrying rejection tests, one positive mid-haul load, the
  live-daemon stone-reaches-a-zone-tile test
- `crates/tui/src/palette.rs` — `carrier_cell()`, pinned and distinct
- `crates/tui/src/view.rs` — cell contention rule, one filter for counting and drawing, repointed
  occlusion and layer-order tests
- `crates/tui/tests/client.rs` — the haul replay instrument and its unchanged-world control
- `_bmad-output/implementation-artifacts/mutations/3-3-the-haul-and-the-skeleton-walks.sh` — NEW
- `_bmad-output/implementation-artifacts/deferred-work.md` — two items closed
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status transitions

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-07 | Story created |
| 2026-08-07 | Implemented directly (Codex quota exhausted): haul jobs, carrying, execution, save/load, loader rules, carrier glyph, instrument, walking-skeleton scenario, 31-mutation set. Gate green from a clean build; live loop run and reported. AC17 awaits Wolf. |
