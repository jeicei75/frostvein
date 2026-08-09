---
baseline_commit: 36b7de3
model: claude-opus-5[1m]  # default Opus; 1M-context variant, as at 4.1a
---

# Story 5.1: The World Grows Things That Glow

Status: in-progress

## Story

As the boss,
I want the generated world to contain pine trees, a real dwarven camp with torches and a campfire, and a skyline worth looking at,
so that the valley has something living in it and something warm in it before any client exists to light it.

## Acceptance Criteria

### Terrain shape — the recorded silhouette decision

1. **The silhouette decision is YES and is implemented here.** On the default seed, surface
   heights span at least 16 z-levels within `Dims::DEFAULT` (128×128×32): the lowest column is
   at or below z 10 and the highest at or above z 26. A `sim-core` test asserts the span as a
   range, not as pinned values. (Today: 12–20, an 8-level span — measured, see Dev Notes.)
2. The reshaping preserves the traversal invariants: no two 4-neighbour columns differ by more
   than one z-level, and every one-level step still has a ramp. `tests/worldgen.rs`
   `height_varies_and_steps_are_at_most_one` and `ramps_connect_every_step` pass **unmodified**.
3. No worldgen write leaves the grid at any height: every surface tile, ramp and tree tile is
   written inside `0..dims.z`, asserted by a test that generates the default world and checks the
   topmost written z against `dims.z`.

### The camp

4. Worldgen chooses exactly one camp site, deterministically from the seed: the flat standable
   area nearest the map centre, large enough to hold five dwarves and the emitters with room to
   wander. The chosen origin is exposed to tests.
5. All five dwarves spawn inside the camp clearing. Each spawn position is standable and has at
   least one standable 4-neighbour.
6. The camp clearing contains no tree tiles.

### Trees

7. `sim-core::Material` gains exactly two variants, `TreeTrunk` and `TreeFoliage`. Pine trees
   stand on the surface, placed from a new purpose-named worldgen RNG stream; density, height and
   crown shape are tuned inside this story.
8. Tree tiles block pathing through the existing solidity rule — `Terrain::is_standable` and
   `astar_neighbours` are **not modified**, and a test shows a dwarf routing around a trunk.
9. Trees do not enclose the camp: a scenario test designates a diggable tile outside the clearing,
   ticks, and the dig completes.

### Digging a tree

10. Digging a `TreeTrunk` or `TreeFoliage` tile removes the tile via `World::set_tile`, records it
    in the per-tick dirty set, and spawns **no item**. Digging any mineral material still spawns
    exactly one stone.
11. Channelling a tile whose material is a tree material likewise spawns no item.

### The wire

12. `protocol` grows exactly this and nothing else: `Material::TreeTrunk`, `Material::TreeFoliage`,
    `EntityKind::Torch`, `EntityKind::Campfire`, `enum LightKind { Torch, Campfire, Lantern }`, and
    the field `Entity.light: Option<LightKind>`. `LightKind::Lantern` is declared here and unused —
    it lands live only if FR29 ships.
13. Emitters appear in `Snapshot.entities` and `Delta.entities` with `kind` naming the object,
    `state: JobState::Idle`, and `light: Some(..)`. Dwarves carry `light: None`.
14. `light` is always serialized — a dwarf's entity JSON carries `"light":null`. The golden wire
    literals in `protocol` and `tui` are updated to the new byte-exact form and stay pinned.
15. Every `sim-core` → `protocol` enum bridge covering the new variants is an exhaustive `match`
    with no wildcard arm, and the bridge's independent test oracle is extended to match.

### The TUI (the parity rule's backward half)

16. `tui` renders `TreeTrunk`, `TreeFoliage`, `Torch` and `Campfire` as four glyphs from
    `palette.rs`, each distinct from every glyph already in use. No glyph or RGB literal appears
    outside `palette.rs`.
17. Emitters draw beneath dwarves in the layer order; the crowd/carrier cell-contention rule and
    the status line's `dwarves N` count remain dwarf-only.

### Determinism

18. Two worlds generated from the same seed are identical tile-for-tile and entity-for-entity —
    trees and emitters included — both at generation and after N ticks in lockstep.
19. Save → load → tick N is state-identical to never-saved → tick N with emitters present in the
    world.

### The instrument

20. A capture from `tui --frames N --z <camp z>` reports a **non-zero** count of tree glyphs and a
    non-zero count of **each** emitter glyph before any conclusion is drawn about the feature.
21. The instrument's own test drives the real `tui` binary against the stub daemon and shows those
    counts **change** when the world changes — a capture that is well-formed but empty must fail it.

## Tasks / Subtasks

- [x] **Task 1 — Reshape the terrain for a skyline** (AC: 1, 2, 3)
  - [x] Widen the height field's amplitude in `crates/sim-core/src/worldgen.rs:height_field`.
        **Amplitude alone will not do it** — see the arithmetic in Key decisions; `NOISE_SPACING`
        must move with it or `clamp_steps` grinds the peaks back down.
  - [x] Keep the height clamp inside `[3, dims.z - 2]` and leave headroom for tree crowns.
  - [x] Add a test asserting the height span (≥16 levels, min ≤ 10, max ≥ 26) as a range.
  - [x] Add a test asserting no tile is written at or above `dims.z`.
  - [x] Re-pin `tests/worldgen.rs`'s terrain fingerprint and spawn literals — they **will** change,
        loudly, which is what they are for. Extend the fingerprint's exhaustive `Tile` match with
        the two new materials by **appending** codes; do not renumber existing ones.

- [x] **Task 2 — Choose the camp and spawn the dwarves into it** (AC: 4, 5, 6)
  - [x] Add a camp-site rule to worldgen: scan flat standable columns, take the one nearest the
        map centre with a large enough flat neighbourhood, tie-broken deterministically.
  - [x] Rewrite `World::spawn_dwarves` to draw its five positions from inside the clearing rather
        than from the whole map. `STREAM_SPAWN` stays the spawn stream (AD-7).
  - [x] Set each dwarf's `Wander { home }` to its camp position so wandering keeps them there.
  - [x] Close `deferred-work.md:46-53` (border-biased spawn) — this is the real embark-site rule
        that entry named as its revisit trigger. Add the closing note to `deferred-work.md`.

- [x] **Task 3 — Grow the trees** (AC: 7, 8, 9)
  - [x] Add `Material::TreeTrunk` and `Material::TreeFoliage` to `sim-core`.
  - [x] Add a fourth stream constant `STREAM_TREES` beside the existing three and build the tree
        RNG in `World::generate`. Place trees **after** ramps and **before** `spawn_dwarves`, and
        skip the camp clearing.
  - [x] Truncate or skip a tree where the crown would not fit under `dims.z`.
  - [x] Tests: a dwarf routes around a trunk; the camp is tree-free; a dig outside the clearing
        completes (proves the camp is not enclosed).

- [x] **Task 4 — A dug tree drops nothing** (AC: 10, 11)
  - [x] In `execute_jobs`, bind the material at `crates/sim-core/src/lib.rs:836` instead of `_` and
        carry a yield flag through the `change` tuple; guard the `ecs.spawn((Item, ...))` at
        `lib.rs:864-865` on it. Leave `set_tile`, `clear_paths`, job retirement, designation
        clearing and `release_claim` untouched.
  - [x] This is the **first place in `sim-core` that matches on `Material`** — write it as one
        `matches!` on the tree variants, not a new trait or table.
  - [x] Tests: dig a trunk → tile empty, dirty set carries it, item count unchanged; dig stone →
        exactly one stone, unchanged from today.

- [x] **Task 5 — Grow the wire vocabulary** (AC: 12, 13, 14, 15)
  - [x] `sim-core`: a marker component for emitters plus the light kind, a sorted public reader
        (`emitters()`) modelled on `items()`, and a `SaveState` field modelled on
        `SaveState.items`. Both `to_save` and `from_save` must be updated in lockstep —
        `to_save`'s `filter_map` **silently skips** an entity missing a component it reads.
  - [x] `simd`'s save loader (`main.rs:483-498`) must add emitter ids to the `seen_ids` uniqueness
        check and the `next_id` bound, or a corrupt save loads silently.
  - [x] `protocol`: the five additions of AC12, mirroring `sim-core` (AD-6). `LightKind` must be
        `Copy + Eq` — `protocol::Entity` derives both and is passed by value throughout `tui`.
  - [x] `simd/src/bridge.rs`: extend `fn material`, add an `entity_kind`/light bridge, and emit
        emitters into both `snapshot()` and `delta()` entity lists. Extend the independent test
        oracle `expected_material` (`bridge.rs:184-191`) — it is a deliberate second copy, keep it
        independent.
  - [x] Update the runtime material allow-list at `bridge.rs:381` (`["stone","soil","ice","snow"]`)
        — it is a **runtime** check that fails the moment worldgen emits a tree.
  - [x] Update the golden literals: `protocol/src/lib.rs:162,174` (`WIRE`, `DELTA_WIRE`),
        `protocol/src/lib.rs:211-213` (the `to_string` entity assertion), the variant-name pinning
        table at `protocol/src/lib.rs:356-395`, and `tui/src/main.rs:533-545`
        (`SNAPSHOT_LINE`, `DELTA_LINE`).

- [x] **Task 6 — Render them in the TUI** (AC: 16, 17)
  - [x] `palette.rs:23-59` `tile_cell`: four new arms (`Solid` and `Ramp` for each tree material).
        Ramp arms take the ramp glyph `▲` with the tree colour, like every other ramp.
  - [x] `palette.rs:61-76` `entity_cell`: match the emitter kinds on kind alone
        (`(EntityKind::Torch, _) =>`), since an emitter has no meaningful job state.
  - [x] `view.rs`: the entity draw loop at `view.rs:228-240` filters `kind == EntityKind::Dwarf`
        and would **silently drop** emitters — this is `deferred-work.md:317-324`, and this story
        owns the rule it names. Draw emitters in their own pass **below** dwarves; leave the crowd
        count (`view.rs:220-227`) and the status-line dwarf count (`view.rs:269-273`) dwarf-only.
  - [x] Extend the layer-order test `marker_layers_follow_terrain_zone_designation_item_entity_pending_cursor_order`
        (`view.rs:967-1025`) with the emitter layer.
  - [x] Extend `palette.rs:167-294` `every_look_is_pinned`, including the `existing_glyphs`
        distinctness set — four new glyphs must be added there or the distinctness claim is a lie
        by omission.
  - [x] Close `deferred-work.md:317-324` with the rule this story chose.

- [x] **Task 7 — Determinism and save/load** (AC: 18, 19)
  - [x] Extend `tests/scenario.rs:1203-1241` `same_seed_and_commands_remain_deterministic` with the
        new `emitters()` reader in its per-tick assertion list.
  - [x] Extend the `save_load.rs` round-trip to carry emitters.
  - [x] `frostvein.save` at the repo root holds the old vocabulary and will no longer load. v0 has
        no save-format stability guarantee — delete it or regenerate it, and say which.

- [x] **Task 8 — The observability instrument** (AC: 20, 21)
  - [x] The instrument is the existing `tui --frames N --z N`; **do not build a second one.**
        Its exact command and the required non-zero observation are in Verification below.
  - [x] Add an integration test in `crates/tui/tests/client.rs` that spawns the real `tui` binary
        against the stub daemon with a world containing trees and emitters, counts the four new
        glyphs in the ANSI-stripped capture, and asserts each count is non-zero.
  - [x] The same test carries its **control**: a stub world with no trees and no emitters produces
        zero of those glyphs. Without it, a test asserting only "non-zero" would pass against a
        renderer that painted the glyphs unconditionally.
  - [x] Note the harness trap recorded in Key decisions: `glyph_positions` in that file records
        only the first occurrence of a glyph per line (`deferred-work.md:326-330`) — a counting
        assertion must not be built on it.

- [ ] **Task 9 — Sabotage table** (project rule: a green suite is not evidence)
  - [ ] Write `_bmad-output/implementation-artifacts/mutations/5-1-the-world-grows-things-that-glow.sh`
        following the existing files' shape, and run `scripts/mutate.sh` against it.
  - [ ] Minimum set: the yield guard is removed (a dug tree drops stone); the camp clearing filter
        is dropped from tree placement; the tree stream is replaced by the worldgen stream (a
        determinism change that must move the pinned values); the emitter draw pass is deleted; the
        emitter list is dropped from `delta()` but kept in `snapshot()`; the amplitude change is
        reverted (the height-span test must go red).
  - [ ] Paste the RED output verbatim into the Dev Agent Record.

- [ ] **Task 10 — Gate and commit**
  - [ ] `scripts/gate.sh` green. Do **not** add gate probes here — NFR8's `client-core` and `gui`
        probes land with those crates in 5.2 and 5.3.
  - [ ] Branch `5-1-the-world-grows-things-that-glow`. Small commits, imperative messages, author
        `Völundr <jeicei75@gmail.com>`. Push/PR only on Wolf's explicit yes.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No `client-core`, no `gui`, no Bevy.** 5.2 adds the mirror crate, 5.3 the Bevy window.
- **No lighting model, no colour temperature, no flicker, no radius.** The wire carries kind
  identifiers only. Appearance lives in `gui`'s data table, in a later story.
- **No lanterns.** `LightKind::Lantern` is declared and left unused; the moving-light case is FR29,
  first on the M2 cut list.
- **No wood items, no chopping job, no tree growth over time.** Digging a tree yields nothing.
- **No snow-capping.** Snow on trunks and crowns is presentation computed by clients from material
  and exposure (AD-16) — never wire state, and not this story.
- **No sign-off gate.** UX-DR22 binds 5.4, explicitly not 5.1–5.3. Nothing here is judged on looks.
- **No new dependencies.** The stack is closed; a new crate needs one sentence of justification.
- **No gate probes, no `scripts/gate.sh` change.**
- **No performance work.** Nothing has been measured.
- **Nothing else from `deferred-work.md`** beyond the two entries whose triggers this story
  actually fires (`:46-53` border-biased spawn, `:317-324` the non-`Dwarf` draw filter).

### What already exists (build on it, do not re-derive)

- **Nothing matches on `Material` in `sim-core` today.** Solidity is decided on the `Tile`
  constructor, so `Tile::Solid(Material::TreeTrunk)` blocks pathing with **zero code change** —
  that is exactly what AC8's "via existing solidity rules" means. The single gate is
  `Terrain::is_standable` (`lib.rs:478-485`): a tile is enterable only if it is `Tile::Empty`.
- **Worldgen is already a three-stage pipeline** — `height_field` → `layered_terrain` →
  `place_ramps` (`worldgen.rs`), driven from `World::generate` (`lib.rs:1047-1074`). Tree placement
  is a fourth stage, not a rewrite.
- **Purpose-named RNG streams already exist**: `STREAM_WORLDGEN`, `STREAM_SPAWN`, `STREAM_WANDER`
  (`lib.rs:26-28`), all `ChaCha8Rng::seed_from_u64(seed ^ STREAM_X)`. A generate-time tree stream is
  a fourth constant and needs **no** `SaveState` change — the tiles it produced are already saved.
- **`items()` (`lib.rs:1408-1418`) is the model for `emitters()`**: `iter_entities()` + marker
  filter + `sort_by_key(Id)`. `carrying()` is the precedent for adding a **new sorted reader**
  rather than widening `dwarves()`.
- **`World::set_tile` already records the dirty set** (`lib.rs:1222-1232`, `Terrain::set_tile`
  `lib.rs:455-476`); a `BTreeSet<Pos>` drained sorted, re-reading current values at drain time.

### Key decisions & traps

- **Amplitude alone will not make a skyline; `NOISE_SPACING` must move with it.** Today
  `height_field` computes `dims.z/2 + (noise*2-1)*4` → heights 12–20 (measured), and
  `clamp_steps` then iterates to a fixpoint capping any 4-neighbour delta at 1. With
  `NOISE_SPACING = 16`, a 20-level swing across one lattice cell is 1.25 levels/tile — `clamp_steps`
  grinds it flat and you get a wider hill, not a peak. **Raise the amplitude and the lattice spacing
  together** so the gradient stays under one level per tile; larger landforms are what the vista
  register wants anyway.
- **The height clamp must leave crown headroom.** `layered_terrain` writes the surface tile *at*
  `index(dims, x, y, height)`, and heights clamp to `[3, dims.z - 2]` = `[3, 30]` — one free level
  above the tallest column. A crown on a peak clips straight out of the grid. Either lower the top
  clamp by the maximum tree height or skip/truncate trees without headroom. AC3 exists for this.
- **`heights[i]` is the z of the topmost SOLID tile**, so the first standable z of a column is
  `height + 1` (see `spawn_dwarves`, `lib.rs:1462`). Off-by-one here puts trees inside the ground.
- **The `light` field is always serialized (`"light":null` for dwarves).** No
  `skip_serializing_if`, no `#[serde(default)]`. Rationale: v0 is deliberately chatty and
  unoptimised, and an omitted field would introduce an absence-means-something convention next to
  AD-8's section-level "absence is deletion" — two different meanings for a missing thing on one
  wire. The cost is updating four golden literals, which is a one-time, visible edit.
- **`Entity.state` stays required; emitters carry `JobState::Idle`.** Making it optional would
  exceed AD-16's sanctioned wire diff, which is exactly the `light` field plus the enum variants.
- **`protocol::Entity` derives `Copy` and `Eq`** (`protocol/src/lib.rs:89`) and is passed by value
  throughout `tui/src/view.rs`. `LightKind` must derive both or the derives cascade.
- **Adding an `EntityKind` variant is only half-caught by the compiler.** `palette::entity_cell` is
  an exhaustive `match` and will fail to compile — good. But `view.rs`'s three
  `kind == EntityKind::Dwarf` equality checks compile fine and **silently drop** the new entities.
  `deferred-work.md:317-324` predicted this precisely and named this story's trigger.
- **`bridge.rs:381` is a runtime allow-list**, not a `match`: `["stone","soil","ice","snow"]`. It
  will not fail to compile; it will fail at test time with a confusing message once worldgen emits a
  tree. Update it deliberately.
- **`protocol/src/lib.rs:356-395` pins variant wire names from a hand-written array** — it passes
  silently while leaving new variants unpinned. Add all five.
- **The terrain fingerprint at `tests/worldgen.rs:56-73` will change, and that is correct.** Its
  exhaustive `Tile` match must gain codes for the new materials — **append** new codes, never
  renumber existing ones, or the fingerprint changes for a second, invisible reason.
- **`to_save`'s `filter_map` silently skips an entity missing a component it reads**
  (`lib.rs:1082-1085`). An emitter spawned with a component set that `to_save` does not expect
  vanishes from the save with no error.
- **`MAX_SAVE_BYTES` is a hand-picked constant, not derived** (`deferred-work.md:196-209`), and it
  has already been broken once by a story that added state rather than dimensions. Emitters are a
  handful of entities so this should not fire — but confirm a save still succeeds rather than
  assuming it.
- **Counting glyphs with `tr -cd '█'` is wrong and lies quietly.** `tr` works on bytes; the box
  glyphs share leading UTF-8 bytes, so `tr` reports inflated counts for every glyph including ones
  that are absent. I hit this while validating the recipe below — first run reported 3520 dwarf
  glyphs for a five-dwarf world. **Use `grep -o '<glyph>' | wc -l`.**
- **`simd` has no seed flag.** The seed is the constant `SEED = 0xF005_7E1A`
  (`crates/simd/src/main.rs:20`) and the port is positional only (`simd 7413`). Any recipe runs
  against that one world.
- **Under a pipe the TUI viewport is 100×40** (`frame_size()`, `main.rs:321-326`) with the camera
  fixed at the map centre, so a capture shows only x 14..113, y 45..82. **A camp outside that
  window is invisible to the recipe.** Camp-nearest-map-centre (AC4) keeps it inside; if the site
  rule ever moves, the recipe needs `--key` panning.
- **`--z` sets only the opening level and is clamped to `0..=dims.z-1`** (`view.rs:58-78`). Without
  it the client picks the level with the most standable ground — deterministic but world-dependent,
  and today that is z 17 (measured). Reshaped terrain will move it: **pin `--z` explicitly.**
- When torn between simple and general, pick simple and leave a `// NOTE:` naming the limitation.

### Measured facts about today's world (baseline `36b7de3`)

Taken live from a running daemon, not inferred:

| fact | value |
| --- | --- |
| default seed | `0xF005_7E1A` (`simd/src/main.rs:20`) |
| dims | 128 × 128 × 32 |
| surface heights | min 12, max 20 (mode 16) — **8 levels of relief on a 32-level world** |
| dwarf spawns | `(96,68,20) (78,48,17) (16,96,15) (120,48,20) (75,33,15)` — up to ~104 tiles apart |
| height at map centre (64,64) | 13 |
| `opening_z` with no `--z` | 17 |

The scatter in row four is why AC4–6 exist: **there is no camp today**, and FR28 places torches "at
the dwarven starting camp" as though one existed.

### Project Structure (files to touch)

```
crates/sim-core/src/worldgen.rs        UPDATE  height amplitude + lattice spacing; tree placement stage
crates/sim-core/src/lib.rs             UPDATE  Material variants; STREAM_TREES; camp site rule;
                                               spawn_dwarves; emitter component + emitters();
                                               execute_jobs yield flag; to_save/from_save
crates/sim-core/src/save.rs            UPDATE  SaveState emitter field
crates/sim-core/tests/worldgen.rs      UPDATE  height span, bounds, re-pinned fingerprint + spawns
crates/sim-core/tests/scenario.rs      UPDATE  determinism list; tree dig; camp not enclosed
crates/sim-core/tests/save_load.rs     UPDATE  emitters round-trip
crates/protocol/src/lib.rs             UPDATE  Material/EntityKind variants, LightKind, Entity.light,
                                               golden literals, variant-name pinning table
crates/simd/src/bridge.rs              UPDATE  material bridge, entity/light bridge, emitters in
                                               snapshot() + delta(), test oracle, allow-list
crates/simd/src/main.rs                UPDATE  save-load validation: emitter ids in seen_ids/next_id
crates/tui/src/palette.rs              UPDATE  tile_cell + entity_cell arms; every_look_is_pinned
crates/tui/src/view.rs                 UPDATE  emitter draw pass below dwarves; layer-order test
crates/tui/src/main.rs                 UPDATE  golden SNAPSHOT_LINE / DELTA_LINE
crates/tui/tests/client.rs             UPDATE  instrument test: glyph counts + empty-world control
_bmad-output/implementation-artifacts/deferred-work.md   UPDATE  close :46-53 and :317-324
_bmad-output/implementation-artifacts/mutations/5-1-the-world-grows-things-that-glow.sh   NEW
frostvein.save                         DELETE or regenerate (old vocabulary; v0 has no format guarantee)
```

### Previous story intelligence

- **4.1a's depth view is on an unmerged branch and is not your baseline.** Work from `main` at
  `36b7de3`; `crates/tui/src/raycast.rs` does not exist here.
- **4.1a could not kill its `EntityKind` filter mutation** because `protocol::EntityKind` had
  exactly one inhabitant, and recorded that the mutation is re-added the moment a second variant
  exists. This story creates that second variant — the filter mutation is now killable and belongs
  in Task 9's set.
- **A change-detection capture that compares whole frames is false evidence** (4.1a): the status
  line carries the tick, which differs every frame regardless of what the picture does. Strip the
  status line before comparing, and carry an unchanging-world control.

### Verification

**Gate (must be green before done):**

```bash
scripts/gate.sh
```

**The instrument recipe.** Run from the repo root, in two shells or with a background daemon:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --bins
./target/debug/simd 7413 &            # port is POSITIONAL; there is no --seed flag
sleep 2
Z=<the camp's z>                      # pin it; do NOT rely on the opening level
out=$(./target/debug/tui 7413 --frames 6 --z "$Z" 2>/dev/null | sed -e $'s/\x1b\\[[0-9;]*m//g')
for g in '<trunk>' '<foliage>' '<torch>' '<campfire>' '☺'; do
  printf '%s = %s\n' "$g" "$(printf '%s' "$out" | grep -o "$g" | wc -l)"
done
```

**The required observation: every one of the four new glyph counts is non-zero, and so is `☺`.**
A zero in any of them means the capture aimed at the wrong level or the feature is not there — the
two are indistinguishable from the exit code. **Exit 0 is not a result.** Use `grep -o`, never
`tr -cd`: `tr` counts bytes and inflates every box-glyph count (see Key decisions).

**What was executed at story-creation time, and what it proved.** The harness half of this recipe
was run live against `36b7de3`, because the recipe's shape — not just its wording — is the thing
that has failed twice on this project. Daemon on port 7413, `tui --frames 2 --z N`, SGR stripped,
`grep -o` counts:

```
z=14  stone=1066  soil=1290  ice=440   snow=426   ramp=298   dwarf=0
z=16  stone=214   soil=852   ice=920   snow=870   ramp=664   dwarf=0
z=17  stone=4     soil=574   ice=874   snow=864   ramp=664   dwarf=0
z=18  stone=0     soil=214   ice=778   snow=794   ramp=570   dwarf=0
z=20  stone=0     soil=0     ice=378   snow=470   ramp=216   dwarf=2
```

Non-zero, varying with `--z`, and it caught two real traps before they reached this file: the first
run of this probe passed `--port`/`--seed` flags that `simd` does not accept and captured nothing
while exiting 0, and the second used `tr -cd` and reported 3520 dwarf glyphs. **The half that cannot
run yet** is the four new glyph counts, because the feature does not exist — so the exact command
and the exact non-zero observation are stated above, and producing them is the dev agent's
obligation, recorded in the Dev Agent Record with the actual numbers.

**Sabotage:**

```bash
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/5-1-the-world-grows-things-that-glow.sh
```

### If this overruns one session

The split line is **terrain shape + camp** (Tasks 1, 2) versus **trees, emitters, wire, TUI**
(Tasks 3–8). The first half alone is observable: a reshaped valley with the five dwarves gathered
in one place, visible in the TUI capture. Note that Epic 5's plan records wow beat 1 landing at
story 4 of 11 — 36%, meeting CM2 at the edge — and that a split of 5.2 or 5.3 breaches it. **A split
of 5.1 does not**, because it happens before the beat rather than pushing it back; but say so on the
record if you take it.

### References

- Epic 5 and story 5.1 ACs — `_bmad-output/planning-artifacts/epics.md:609-661`
- AD-13…AD-18, M2 conventions, stack, sequencing — `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`; AD-16 at `:121-144`
- FR27/FR28/FR30, NFR5/NFR7/NFR8 — `_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/prd.md:164-183, 220-238`
- The silhouette question's origin and the constraint set — `prds/prd-frostvein-2026-08-09/addendum.md:34-38`; `reconcile-narrative.md:101-119`; readiness fix #5 at `implementation-readiness-report-2026-08-09.md:276-300`
- M1 FR2 "modest rolling hills", and its stated purpose (pathfinding, not the vista) — `prds/prd-frostvein-2026-08-01/prd.md:52-55`
- UX-DR4, 5, 6, 8, 10, 11, 13 (the warm/cold read the camp serves) — `epics.md:161-176`
- Story rules, the instrument rule, "exit 0 is not a result" — `docs/technical-preferences.md:64-101`
- Deferred entries this story closes — `_bmad-output/implementation-artifacts/deferred-work.md:46-53, 317-324`
- Save-cap and glyph-counting traps — `deferred-work.md:196-209, 326-330`
- 5.4 builds on this story's silhouette decision — `epics.md:771-775`

### Decisions recorded on the record (Wolf, 2026-08-09)

1. **The vista mountain silhouette: YES, and shaped here.** In-grid terrain within 128×128×32 is
   tuned to give the skyline peaks the aurora can backlight. This is the last of the three
   decisions the M2 spine owed inside M2 stories, and the one it warned must be revisited
   consciously and never silently stretched.
   **Why:** FR33's "slice into the mountain" presumes in-grid mountains exist; the narrative's
   signature frame is peaks backlit by aurora; and 5.4 is the wow gate with CM2 met at the edge, so
   discovering this there would mean re-opening worldgen two stories back — precisely the failure
   the readiness report named. Today's 8 levels of relief on a 32-level world read as a plain from
   an isometric vista. Judgment of the *look* still belongs to 5.4 under the sign-off gate; what
   5.1 owes is a headless-testable shape (AC1–3), not an aesthetic verdict.
   **5.4 builds on this rather than re-opening it** (`epics.md:771-775`).
2. **The camp is real: the dwarves are clustered into it.** 5.1 defines a deterministic camp site
   and spawns all five dwarves there, with the campfire and torches around them.
   **Why:** UX-DR5 requires the eye to land on the encampment through warm/cold contrast alone. A
   campfire burning where no dwarf lives is not an encampment, and 5.4's entire wow rests on that
   read. The pinned spawn positions and terrain fingerprint change loudly, which is what those pins
   are for.

## Dev Agent Record

### Agent Model Used

GPT-5.6 Codex (Völundr)

### Debug Log References

- Task 1 RED: `default_world_has_mountainous_height_span` failed with `minimum surface height was 12` before the height-field change.
- Task 2 RED: the camp tests failed to compile because `World::camp_origin` did not exist before the camp rule was implemented.
- Task 3 RED: tree tests failed to compile with 12 `no variant ... TreeTrunk/TreeFoliage` errors before the materials and generator existed.
- Task 4 RED: `execute_jobs_digs_tree_materials_without_spawning_items` failed `left: 1, right: 0` for `TreeTrunk` before the yield guard.
- Task 5 RED: protocol tests failed with missing `Entity.light`, both tree materials, both emitter kinds, and `LightKind`; bridge tests also referenced the not-yet-existing emitter reader.
- Task 6 RED: workspace checking failed on non-exhaustive TUI matches for all four tree tile shapes and both emitter kinds before palette/render support.
- Task 7 RED (controlled sabotage): removing emitter restoration made `save_round_trip_preserves_emitters` fail with `left: []` versus the five expected `(Id, Pos, LightKind)` tuples; restoration was then reinstated.
- Task 8 RED (controlled sabotage): deleting the emitter draw pass made `growing_world_instrument_counts_change_with_trees_and_emitters` fail with `feature capture contained zero †`; the pass was restored.
- Task 8 manual instrument tuning: z=25 aimed at seed 42 and produced `│=30 ♠=222 †=0 ♨=0 ☺=0`; the live default-seed snapshot identified camp z=9. At z=9 the first run exposed absent tree slices, and later a fully occluded campfire; density/crown air-only placement and emitter-cell-free dwarf spawns closed both evidence failures.

### Completion Notes List

- Task 1: widened terrain amplitude and lattice spacing together, preserved the height clamp and traversal invariants, added range/bounds tests, and re-pinned the seed-42 literals. `cargo test --offline -p sim-core` passed (91 tests).
- Task 2: selected the nearest deterministic 7x7 flat central clearing, persisted/exposed its origin, restricted all five seeded spawn draws to it, and independently tested nearest-site and standability/mobility properties.
- Task 3: added a purpose-seeded pine pass after ramps, bounded every crown below `dims.z`, kept foliage out of the whole camp clearing, and proved pathfinding detours plus reachable outside digging. All 96 `sim-core` tests pass offline.
- Task 4: carried a tree/mineral yield flag through Dig and Channel changes; both tree materials mutate and dirty their tiles without items, while existing mineral yield tests stay pinned. All 98 `sim-core` tests pass offline.
- Task 5: added five fixed camp emitters, sorted/persisted emitter state, exact wire vocabulary and null-light goldens, exhaustive independent bridges, and save-id/bounds validation. Protocol (5), sim-core (99), simd unit (14), and simd serve (60) tests pass offline.
- Task 6: pinned four distinct tree/emitter glyphs in the palette, added an emitter pass between items and dwarves, preserved dwarf-only contention/status rules, and closed the non-dwarf draw deferral. All 61 TUI tests pass offline.
- Task 7: added emitter assertions to per-tick determinism and save/load lockstep, added an explicit emitter round-trip test, and deleted the obsolete 6.9 MB v0 `frostvein.save`. All 100 `sim-core` tests pass offline.
- Task 8: added a bounded real-binary feature/control capture that counts every ANSI-stripped glyph occurrence. Manual `tui 7413 --frames 6 --z 9` observation: `│=6`, `♠=48`, `†=21`, `♨=3`, `☺=30` (all non-zero; `NO_COLOR` warning observed, so this manual run evidences glyphs, not colours). All 100 sim-core and 62 TUI tests pass offline.

### File List

- `_bmad-output/implementation-artifacts/5-1-the-world-grows-things-that-glow.md`
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `crates/sim-core/src/lib.rs`
- `crates/sim-core/src/save.rs`
- `crates/sim-core/src/worldgen.rs`
- `crates/sim-core/tests/worldgen.rs`
- `crates/sim-core/tests/scenario.rs`
- `crates/sim-core/tests/save_load.rs`
- `crates/protocol/src/lib.rs`
- `crates/simd/src/bridge.rs`
- `crates/simd/src/main.rs`
- `crates/simd/tests/serve.rs`
- `crates/tui/src/main.rs`
- `crates/tui/src/palette.rs`
- `crates/tui/src/view.rs`
- `crates/tui/tests/client.rs`
- `frostvein.save` (deleted)

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-09 | Story created. Two decisions taken on the record with Wolf: the vista mountain silhouette (yes, shaped here) and the camp (dwarves clustered into it). |
| 2026-08-09 | Task 1 reshaped the terrain skyline and added range and bounds coverage. |
| 2026-08-09 | Task 2 established the deterministic central camp and clustered dwarf spawns. |
| 2026-08-09 | Task 3 added deterministic pine trees with camp, bounds, pathing, and reachability coverage. |
| 2026-08-09 | Task 4 made dug and channelled tree materials yield no stone. |
| 2026-08-09 | Task 5 added persisted camp emitters and the exact light-aware wire vocabulary. |
| 2026-08-09 | Task 6 rendered trees and fixed emitters with the specified layer and parity rules. |
| 2026-08-09 | Task 7 pinned emitter determinism/save-load behavior and removed the obsolete save. |
| 2026-08-09 | Task 8 proved the real capture instrument with a zero-feature control and non-zero live counts. |
