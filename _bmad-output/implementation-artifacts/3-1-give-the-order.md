---
baseline_commit: 8bf4548
---

# Story 3.1: Give the Order

Status: in-progress

## Story

As the boss,
I want to mark rectangles for digging or channeling and place stockpiles with DF-familiar modal keys,
so that my directives are recorded in the world, visibly, the moment I issue them.

## Acceptance Criteria

1. `sim-core` gains `DesignationKind { Dig, Channel }`, `Rect { min: Pos, max: Pos }` and
   `SimCommand { Designate { kind, rect }, CancelDesignation { rect }, PlaceStockpile { rect },
   RemoveStockpile { rect } }`, plus `World::apply_command(&mut self, SimCommand)` and the readers
   `World::designations() -> Vec<(Pos, DesignationKind)>` and `World::zones() -> Vec<Pos>`, both
   ascending by `Pos`. `apply_command` is a plain `&mut self` method, not a system: nothing is added
   to the schedule.
2. `apply_command` normalizes a rect componentwise (`min`/`max` swapped per axis where needed) and
   clips it to world bounds; a rect entirely outside the world is a no-op. `Designate` records every
   in-bounds tile of the rect at the given kind, overwriting any existing kind on those tiles;
   `CancelDesignation` removes designations on those tiles and leaves zones untouched.
3. `PlaceStockpile` records only the rect tiles where `Terrain::is_standable` holds — the existing
   predicate, not a second one. Non-standable tiles are silently not part of the zone; a rect with
   zero standable tiles yields no zone and changes nothing. `RemoveStockpile` removes the rect's zone
   tiles and leaves designations untouched — the mirror of AC2's cancel, so each of the two erasures
   is independently assertable.
4. **The seam is consumed while paused.** With `simd` paused, a `designate` command sent by a client
   appears in a `delta` within a bounded number of deltas, and **every** delta from the pause onward
   carries the same frozen `tick` — including the one that first carries the designation. (Not "the
   next delta": the command crosses TCP and lands on a later loop iteration, so a delta already in
   flight may arrive first. 2.3 shipped an AC that made exactly that mistake.) `apply_command`
   mutates designation and zone state only, and never touches `Tick`, entity positions or `JobState`.
5. `protocol` replaces the `Vec<()>` placeholders with `designations: Vec<Designation>` and
   `zones: Vec<Zone>` in both `Snapshot` and `Delta`, and gains `DesignationKind`, `Rect`,
   `Designation { pos, kind }`, `Zone { pos }` and the `Command` variants `Designate { kind, rect }`,
   `CancelDesignation { rect }`, `PlaceStockpile { rect }`, `RemoveStockpile { rect }`. These
   hand-written literals each decode and re-encode to the same JSON value, and
   `{"type":"designate","kind":"mine","rect":{"min":[0,0,0],"max":[0,0,0]}}` fails to decode:
   ```json
   {"type":"designate","kind":"dig","rect":{"min":[1,2,3],"max":[4,5,3]}}
   {"type":"cancel_designation","rect":{"min":[1,2,3],"max":[4,5,3]}}
   {"type":"place_stockpile","rect":{"min":[1,2,3],"max":[4,5,3]}}
   {"type":"remove_stockpile","rect":{"min":[1,2,3],"max":[4,5,3]}}
   ```
   and a snapshot/delta carries `"designations":[{"pos":[1,2,3],"kind":"dig"}]`, `"zones":[{"pos":[1,2,4]}]`.
6. `simd` consumes the four world-mutating commands in its existing iteration-top drain, in arrival
   order, by calling `world.apply_command(...)` — above and independent of the `if speed != Paused`
   guard (AD-10). `bridge` maps `protocol::DesignationKind` and `sim_core::DesignationKind` in both
   directions by exhaustive `match` with no wildcard arm, and `bridge::snapshot`/`bridge::delta` emit
   the world's real designations and zones instead of `Vec::new()`.
7. Against the live daemon: a client sends `designate` dig over a rect, then `place_stockpile` over a
   rect, then `cancel_designation` over part of the dig rect, then `remove_stockpile` over part of
   the zone rect. Within a bounded number of deltas the designations list grows to the rect's tile
   count, the zones list carries only standable tiles, and after each erasure the covered tiles are
   absent from **every** connected client's delta while the other list is unaffected. Two clients are
   connected and both observe the same lists.
8. `SaveState` gains `designations` and `zones` (sorted ascending by `Pos`). The AD-11 gate test is
   extended: designate, place a stockpile, tick, save, load, then tick the loaded world and a
   never-saved control 200 further times asserting equal `tick()`, `dwarves()`, `designations()` and
   `zones()` at every step.
9. TUI modal input: from the main view `d`, `c`, `p`, `x` enter dig / channel / stockpile / remove
   mode and place a cursor at the current view centre. In a mode, arrows and `hjkl` move the
   **cursor** (clamped to world bounds, camera panning only as far as needed to keep the cursor on
   screen), the first `Enter` anchors a corner, the second commits the rectangle and emits the
   matching `Command` for the inclusive rect at the current z-level, leaving the mode active with no
   anchor. `Esc` backs out one level: anchored → un-anchored → normal view.
   **`x` is the eraser and commits two commands** for its rect, `CancelDesignation` then
   `RemoveStockpile`, so one key clears both kinds of mark while each wire command keeps an honest
   name and stays independently injectable in the harness.
10. A one-line hint bar occupies the bottom row and always shows the active mode's keys; the status
    line sits directly above it and reports tick, speed, z and dwarf count. Every mode's hint text is
    at most 80 columns, and the normal-mode hint names `q quit client` — `q` still closes only this
    client and no key in the client ever sends `Command::Quit`.
11. Designations, zones, the cursor and the pending rectangle render as pinned, mutually distinct
    glyph + colour pairs, layered terrain → zones → designations → entities → pending rect → cursor.
    Each marker is distinguishable by **glyph alone**, so the view still carries the information when
    `NO_COLOR` strips every colour sequence.
12. **The optimistic-speed fix** (deferred from 2.3): the next speed is computed from client-side
    state, not from the last delta. Pressing `+` then `-` at `Normal` inside one round-trip emits
    `SetSpeed{Fast}` then `SetSpeed{Normal}` — never `SetSpeed{Paused}`. A wire update overwrites the
    local value, so the daemon stays authoritative.
13. **`read_inbound` reports a partial line honestly** (deferred from 2.3): a client that sends a
    truncated command and closes is no longer logged as a 64 KB overflow. The overflow log fires only
    when the line actually reached `MAX_LINE_BYTES`, matching the split already used at
    `crates/tui/src/main.rs:370-374`.
14. `tui --frames N --key <sequence>` presses a comma-separated key sequence through the real
    `apply_key` and the real write half before streaming. Its tests drive the real binary against a
    stub daemon: the sequence that designates a rect produces a capture containing the dig marker
    glyph, and the identical run **without** the sequence produces a capture containing none.
15. `scripts/gate.sh` passes and `scripts/mutate.sh
    _bmad-output/implementation-artifacts/mutations/3-1-give-the-order.sh` reports zero survivors.

## Tasks / Subtasks

- [x] **`sim-core`: designation and zone state, and the AD-10 consumer** (AC: 1, 2, 3, 4)
  - [x] Add `DesignationKind`, `Rect` and `SimCommand` to `crates/sim-core/src/lib.rs` (skeleton
        below). No new dependency.
  - [x] Two new resources inserted by `assemble`: `Designations(BTreeMap<Pos, DesignationKind>)` and
        `Zones(BTreeSet<Pos>)`. `BTreeMap`/`BTreeSet`, not hash containers — AD-7 forbids iteration
        order affecting outcomes, and `designations()`/`zones()` must be ascending by `Pos`.
  - [x] `assemble` is still the ONE assembly site: both new resources go in there so `generate` and
        `from_save` cannot diverge. Do not touch `schedule.add_systems`.
  - [x] `apply_command` normalizes then clips: `min = componentwise min`, `max = componentwise max`,
        then intersect with `0..dims`. Iterate `z`, then `y`, then `x`.
  - [x] `PlaceStockpile` filters on the existing `Terrain::is_standable`. Do not write a second
        walkability predicate and do not make `is_standable` public if a private call suffices.
        `RemoveStockpile` needs no predicate — it removes whatever zone tiles the rect covers.
  - [x] Tests in `crates/sim-core/tests/scenario.rs` (extend, do not replace): a reversed rect
        designates the same tiles as its normalized form; a rect straddling the world edge designates
        only the in-bounds part; a fully out-of-bounds rect is a no-op; `Designate` over an existing
        designation overwrites the kind; `CancelDesignation` removes only designations and leaves
        zones; `RemoveStockpile` removes only zones and leaves designations; a stockpile rect over
        mixed terrain keeps exactly the standable tiles; a stockpile rect with no standable tile
        yields no zone. The two "leaves the other alone" assertions are a matched pair — set up one
        overlapping rect carrying both a designation and a zone, and erase with each command in turn.
  - [x] Determinism test: `apply_command` then 200 `step()`s from seed 42 twice yields identical
        `dwarves()`, `designations()` and `zones()`.

- [x] **`sim-core`: designations and zones survive save/load** (AC: 8)
  - [x] `SaveState` gains `designations: Vec<(Pos, DesignationKind)>` and `zones: Vec<Pos>`, both
        sorted ascending. `DesignationKind` derives `Serialize`/`Deserialize` like the other sim
        types in the save.
  - [x] Extend `crates/sim-core/tests/save_load.rs`'s gate test: designate a rect, place a stockpile,
        tick 37, `set_tile` one tile, save, load, then compare against a never-saved control for 200
        steps asserting `tick()`, `dwarves()`, the mutated tile, `designations()` and `zones()` after
        **each** step. Do not add a save-format literal test (format stability is a project non-goal).

- [x] **`protocol`: the wire change** (AC: 5)
  - [x] Add `DesignationKind`, `Rect`, `Designation`, `Zone`; replace `designations: Vec<()>` and
        `zones: Vec<()>` in `Snapshot` and `Delta`; delete the two `Vec<()>` NOTEs they made
        obsolete. Add the four `Command` variants.
  - [x] Extend the hand-written literal tests (`WIRE`, `DELTA_WIRE`, the command table, and
        `every_material_and_tile_variant_has_a_pinned_wire_name`) with the new shapes and the three
        new `type` names, and keep the existing `{"type":"store"}` rejection alongside a new
        `"kind":"mine"` rejection. Literals, not round-trips — a symmetric rename must stay red.

- [x] **`simd`: consume the queue at iteration top, emit the real lists** (AC: 6, 7, 13)
  - [x] Extend the iteration-top `match` with the four arms, each bridging into
        `world.apply_command(...)` (skeleton below). It sits above the `if speed != Paused` guard, so
        pause never blocks command intake — that placement is the AD-2/AD-10 contract, not a detail.
  - [x] Rewrite the NOTE at `crates/simd/src/main.rs:177-180`, which predicts this split: say what
        now holds (commands apply while paused; the schedule stays entirely world-advancing; job
        conversion and reaction delays are 3.2's and belong in the schedule).
  - [x] `bridge.rs`: `designation_kind` in both directions plus `rect`/`pos` conversions; exhaustive
        `match`, no wildcard. `snapshot()` and `delta()` map `world.designations()` and
        `world.zones()`. Add the independent-oracle test in the existing style — hand-written wire
        names decoded, never a second copy of the production match.
  - [x] Fix `read_inbound`'s partial-line report [main.rs:401-405]: log the overflow only when the
        read actually hit `MAX_LINE_BYTES`; otherwise treat the unterminated tail as a closed
        connection. Mirror the split at `crates/tui/src/main.rs:370-374`. This is the one piece of
        adjacent code this story is authorized to touch — it is assigned here in
        `deferred-work.md`.
  - [x] Live-daemon tests in `crates/simd/tests/serve.rs`, extending the `Daemon` harness: the
        AC7 designate → stockpile → cancel → remove-stockpile sequence with two clients; the paused-intake test (send
        `set_speed` paused, read two deltas to confirm the tick is frozen, send `designate`, assert
        the next delta carries it and the tick is still frozen); and a partial-line test asserting the
        log line is NOT the overflow message.

- [x] **`tui`: modes, cursor, hint bar, optimistic speed** (AC: 9, 10, 11, 12)
  - [x] `ViewState` gains `mode: Mode`, `cursor: (i64, i64)`, `anchor: Option<(i64, i64)>` and
        `speed: Speed`. `apply_key`'s signature becomes
        `apply_key(&mut ViewState, KeyEvent, Dims, viewport: (u16, u16)) -> Action` — the `speed`
        parameter is **replaced** by `state.speed`, and the viewport is what lets the camera follow
        the cursor. Both signature changes land together; every existing keymap test updates
        mechanically.
  - [x] `initial(&snapshot)` sets the new fields: `Mode::Normal`, `cursor` = the camera it just
        computed, `anchor: None`, `speed` = `snapshot.speed`. Its existing test pins the whole
        `ViewState`, so it fails until this is done — that is the intended order.
  - [x] Markers draw only on the viewed z-level, the same guard entities already use
        [view.rs:107-115]. A designation one level down must not appear on this one.
  - [x] Mode entry (`d`/`c`/`p`/`x`) only from `Mode::Normal`; in a mode those keys are ignored and
        `Esc` is the way out. Cursor starts at the camera position. Speed keys, `S`/`L` and `q`
        remain global and work in every mode.
  - [x] `<`/`>` change z (and the cursor with it) while un-anchored; while anchored they are ignored
        — the rect is single-z by construction rather than by validation.
  - [x] Camera follow: after a cursor move, pan the camera by the minimum needed to keep the cursor
        inside the visible window (`w` columns, `h - 2` map rows, centred on the camera). Test the
        boundary: a cursor walked to the window edge and one step past it moves the camera by exactly
        one.
  - [x] Commit on the second `Enter`: emit `Command::Designate { kind, rect }` or
        `PlaceStockpile { rect }` with `min`/`max` from anchor and cursor at `state.z`, then clear
        the anchor and stay in the mode. `x` mode emits **two** commands for the same rect,
        `CancelDesignation` then `RemoveStockpile`.
  - [x] `Action` carries one command today. Give `x` its second without inventing a general
        multi-command mechanism: the narrowest change is an `Action::Commands([Command; 2])` variant
        or a second `Action` returned alongside — pick one, keep the single-command path untouched,
        and `// NOTE:` that two is the only arity anything needs. Do not build a command queue in the
        client.
  - [x] Optimistic speed: on emitting a `SetSpeed`, write the requested speed into `state.speed`
        immediately; on every applied snapshot/delta, overwrite `state.speed` from the wire. Test the
        exact 2.3 trap: at `Normal`, `+` then `-` with no wire update in between yields `Fast` then
        `Normal`. That assertion fails if the fix is reverted — it is the point of the test.
  - [x] Render: `map_h = h - 2`; row `h-2` is the status line (tick, speed, z, dwarves — the key
        hints move out of it), row `h-1` is the hint bar. Layer order terrain → zones → designations
        → entities → pending rect → cursor.
  - [x] `palette.rs`: `designation_cell(DesignationKind)`, `zone_cell()`, `cursor_cell()` and the
        pending-rect look, each pinned in `every_look_is_pinned`. Suggested glyphs — dig `×`, channel
        `▼`, stockpile `≡`, cursor `+`, remove-mode preview `-`; the exact RGB is yours, but every
        glyph must be absent from the existing tile/entity table and the test must assert the whole
        set is pairwise distinct.
  - [x] Hint-bar test: for every mode and both anchor states the hint is ≤ 80 columns, names that
        mode's keys, and the normal-mode hint contains `q quit client`. `x` mode's hint must say it
        clears both marks and stockpiles — one key doing two things is only discoverable if the hint
        bar says so.

- [x] **Observability instrument** (AC: 14) — extend `tui --frames N --key`, do not invent a second
      channel. `--key` accepts a comma-separated sequence and presses each through the real
      `apply_key`; single values (`space`, `+`, `-`, `S`, `L`) keep working unchanged. Add the names
      `d`, `c`, `p`, `x`, `h`, `j`, `k`, `l`, `enter`, `esc`, `<`, `>` and update the error text.
  - [x] Two tests in `crates/tui/tests/client.rs`, real binary against a stub daemon that echoes the
        designated rect back in its deltas: with `--key d,enter,l,l,enter` the capture contains the
        dig marker glyph at the expected columns; with no `--key` the identical capture contains
        none. The second is the mutation guard on the first — an instrument that rendered the connect
        snapshot forever would satisfy the first alone.
  - [x] Keep the `NO_COLOR` warning. Markers are glyph-distinct by design (AC11), so this capture is
        real evidence even under `NO_COLOR` — say so where the warning is emitted.

- [ ] **Sabotage + mutation set** (AC: 15)
  - [x] `_bmad-output/implementation-artifacts/mutations/3-1-give-the-order.sh`, at least:
        `apply_command` skips normalization; skips bounds clipping; `PlaceStockpile` ignores
        `is_standable`; `CancelDesignation` also clears zones; `RemoveStockpile` also clears
        designations; `RemoveStockpile` is a no-op; `x` commits only its first command;
        `Designate` refuses to overwrite an existing kind; the daemon's designate arm decodes but
        never calls `apply_command`; the daemon's remove-stockpile arm decodes but never calls it; the
        designate arm moves *below* the pause guard; `bridge` swaps dig and channel; `bridge` swaps
        place- and remove-stockpile; `bridge::delta`
        emits `Vec::new()` for designations; for zones; `to_save` drops designations; drops zones;
        `from_save` discards them; the `designate` / `cancel_designation` / `place_stockpile` /
        `remove_stockpile` discriminators are renamed; `kind` is renamed; the second `Enter` commits without emitting;
        `Esc` exits the mode instead of releasing the anchor; the cursor ignores the camera clamp;
        optimistic speed is reverted to reading the wire value; the hint bar is dropped; the
        designation layer is drawn under the terrain; `read_inbound` reports every unterminated line
        as an overflow.
  - [ ] `cargo clean -p protocol -p simd -p tui` before the final gate — `mutate.sh` is not
        concurrency-safe and both 2.3 and 2.4 hit a stale mutated binary afterwards.
  - [x] Paste the actual RED output for every new mapping/constant test into the Dev Agent Record
        (AGENTS.md rule 1).

- [ ] **Green gate** (AC: 15) — `scripts/gate.sh`, then the live check. Report what printed.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No jobs, no job market, no claiming, no reaction delay, no `CurrentJob`.** A designation is a
  mark on the world and nothing more until 3.2. Do not add a system to the schedule.
- **No diggability filtering.** A dig rect records every in-bounds tile, air included. The epic's
  clip rule is written for stockpiles only; deciding which tiles are *diggable* is 3.2's, together
  with FR8's unreachable-retry. Leave a `// NOTE:` saying so rather than inventing the rule here.
- **No A*, no pathfinding, no stone items, no tile mutation.** `set_tile` still has no production
  caller after this story.
- **No second eraser mode.** `x` clears both mark kinds. Do not add a separate stockpile-removal key
  or a "remove zone" mode — one eraser, two wire commands.
- **No mouse input, no rect drag, no multi-z rects from the client.**
- **No quit key that sends `Command::Quit`.** 2.4's AC9 stands: a shared daemon must not die from
  one viewer's keypress. This story only makes the status/hint text honest about it.
- **No fix for the status-line-width or `NO_COLOR`-product-half items** in `deferred-work.md` —
  neither is assigned here.
- **No reconnect, no backpressure, no protocol optimization.**

### What already exists (build on it, do not re-derive)

- `assemble(seed, dims, tiles, tick, wander_rng, ids)` is the single world-assembly site and both
  `generate` and `from_save` go through it [crates/sim-core/src/lib.rs:222-246]. `Terrain` owns
  `dims`/`tiles`/`dirty` and already has `is_standable` [lib.rs:137-143].
- `simd` drains `command_rx.try_iter()` at iteration top, then accepts clients, then steps only when
  not paused, then encodes one delta and broadcasts [crates/simd/src/main.rs:123-191]. `read_inbound`
  decodes every inbound line as `protocol::Command` and forwards it, so new variants need no parser
  change [main.rs:390-428].
- `tui`'s `apply()` already replaces `designations` and `zones` wholesale from each delta
  [crates/tui/src/main.rs:447-448] — the types change, that code does not. `apply_key` +
  `Action::Command`, the `--frames`/`--key` plumbing and `send_command` are all in place
  [view.rs:157-228, main.rs:63-155,262-271].
- Test scaffolding to extend: the `Daemon` harness, `next_log`, `send_literal`, `read_delta`
  [crates/simd/tests/serve.rs:19-220]; the stub-daemon + `strip_ansi` + `glyph_columns` capture
  pattern [crates/tui/tests/client.rs:79-135,488]; and 2.4's mutations file as the worked format.

### Code skeleton

```rust
// crates/sim-core/src/lib.rs — sim-side vocabulary and the AD-10 consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesignationKind { Dig, Channel }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect { pub min: Pos, pub max: Pos }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimCommand {
    Designate { kind: DesignationKind, rect: Rect },
    CancelDesignation { rect: Rect },
    PlaceStockpile { rect: Rect },
    RemoveStockpile { rect: Rect },
}

#[derive(Resource, Default)]
struct Designations(BTreeMap<Pos, DesignationKind>);
#[derive(Resource, Default)]
struct Zones(BTreeSet<Pos>);

impl World {
    /// AD-10: consumed by `simd` at loop-iteration start, in arrival order, and it applies
    /// while paused — designation intake is not world advancement.
    pub fn apply_command(&mut self, command: SimCommand) { /* normalize -> clip -> apply */ }
}
```

```rust
// crates/protocol/src/lib.rs — the wire change. `Rect` earns its own type: four commands use it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignationKind { Dig, Channel }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect { pub min: [i32; 3], pub max: [i32; 3] }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Designation { pub pos: [i32; 3], pub kind: DesignationKind }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone { pub pos: [i32; 3] }
// Zone deliberately carries no `kind`: stockpile is the only zone in phase one, and a
// single-variant enum is the abstraction YAGNI forbids. NOTE: a second zone kind adds the field.
```

```rust
// crates/simd/src/main.rs — the existing iteration-top drain, ABOVE the pause guard.
for command in command_rx.try_iter() {
    match command {
        protocol::Command::SetSpeed { speed: next } => speed = next,
        protocol::Command::Save => save_world(&world),
        protocol::Command::Load => { /* unchanged */ }
        protocol::Command::Quit => { /* unchanged */ }
        protocol::Command::Designate { kind, rect } => world.apply_command(
            sim_core::SimCommand::Designate {
                kind: bridge::designation_kind_in(kind),
                rect: bridge::rect_in(rect),
            },
        ),
        protocol::Command::CancelDesignation { rect } => world
            .apply_command(sim_core::SimCommand::CancelDesignation { rect: bridge::rect_in(rect) }),
        protocol::Command::PlaceStockpile { rect } => world
            .apply_command(sim_core::SimCommand::PlaceStockpile { rect: bridge::rect_in(rect) }),
        protocol::Command::RemoveStockpile { rect } => world
            .apply_command(sim_core::SimCommand::RemoveStockpile { rect: bridge::rect_in(rect) }),
    }
}
```

### Key decisions & traps

- **Option C is Wolf's ruling, not one of several readings** (2026-08-05). The AD-10 consumer is a
  plain `&mut self` method called by `simd` at iteration top, *before* the conditional `world.step()`.
  Rejected: a second always-run schedule (breaks AD-7's single chained schedule and gives `assemble`
  two things to keep in step — the exact divergence it exists to prevent) and bevy run-conditions on
  a `Paused` resource (makes `sim-core` learn about pause, which 2.3 forbade). Precedent:
  `World::set_tile` already mutates sim state as a plain method [lib.rs:375]. The cost, which wants a
  `// NOTE:`, is that command ordering is now explicit by call site rather than by `.chain()`.
- **`remove_stockpile` is a fourth world-mutating command, added on Wolf's call (2026-08-05), and it
  is scope beyond `epics.md`.** Story 3.1's epic text gives stockpiles no removal affordance, so
  without it a misplaced stockpile is permanent until 3.2's save/load or a daemon restart. It obeys
  AD-10 — that rule is the world-mutating/control *split*, and this is unambiguously world-mutating —
  but AD-10's prose enumerates three commands, so the spine's command table and `docs/architecture.md`
  now under-list by one. Do not silently reconcile them from inside this story: flag it in the Dev
  Agent Record so the epic text and the spine are corrected together, once.
- **One eraser key, two honest commands.** The alternative — teaching `cancel_designation` to also
  delete zones — keeps the spine's list at three but leaves a wire command whose name lies about
  half of what it does, in the one crate whose whole job is being the single source of message
  shapes. Two precise commands cost one extra `match` arm and buy independently assertable erasures
  (AC2 and AC3 are mirrors of each other) plus a harness that can inject either alone.
- **Draw the pause line here, or 3.2 draws it by accident.** Designation *intake* applies while
  paused (AC4). Everything designation-*derived* — turning a designation into a job, ticking the
  reaction delay — is world-advancing, belongs in the schedule, and skips while paused. 3.2 adds both;
  this story adds nothing to the schedule, which is what makes that line hold.
- **This is a wire change and the epic text does not say so.** `designations` and `zones` are
  `Vec<()>` today [crates/protocol/src/lib.rs:100-101, 115-116]. `protocol`, `bridge`, the pinned JSON
  literals in `protocol`, `simd`'s tests and `tui` must move in **one commit** or the suite is red
  between them — exactly as 2.2 had to for `Entity.state`.
- **`Vec<()>` was never an empty-array guarantee.** It accepts `[null,null]`. Nothing depends on that,
  but do not read the old NOTE as a contract you must preserve; delete it with the type.
- **The seam test must assert the negative path.** A daemon arm that decodes `designate`, logs it and
  never calls `apply_command` passes every parser test. AC7's "the designation appears in the next
  delta" and AC4's "and the tick did not move" are the assertions that die when the decision is
  discarded — write them first and watch them go red.
- **The instrument's negative control is the point.** 2.2 shipped an instrument that rendered motion
  as stillness and 2.4 needed a no-key run to prove the `--key` path did anything. Here the no-key
  capture must contain **no** marker glyph; without that, a client that drew a marker unconditionally
  would pass.
- **Glyph-distinct markers, deliberately.** This devpod sets `NO_COLOR=1` and crossterm then drops
  every colour sequence. 2.2's job-state signal was colour-only and its evidence was vacuous until
  rerun. Designation markers differ by glyph, so AC14's capture is real evidence either way.
- **`apply_key` changes shape twice in one story** — the `speed` parameter leaves, the `viewport`
  parameter arrives. Do both in one edit; every existing keymap test then updates once, mechanically.
- **Optimistic speed is client-side command state, not game logic.** Speed is `simd` state by AD-10,
  never sim state, so holding a local pending value does not breach AD-4. The wire remains
  authoritative: every applied message overwrites it.
- **80 columns is a real budget.** The status line already reaches exactly 80 at seven-digit ticks
  [view.rs:595-644]. Moving the key hints into the hint bar is what buys the room; do not re-add them
  to the status line.
- **Two rows, not one.** `map_h = h - 2` shifts every existing render test's expected framebuffer.
  That is mechanical, but it is the largest single source of churn in this story — do it first and
  get the suite green again before adding markers.
- **`mutate.sh` is not concurrency-safe** and rebuilds mutated artifacts; both 2.3 and 2.4 hit stale
  mutated binaries. Budget the `cargo clean -p protocol -p simd -p tui` step before the final gate.
- **A story this size may span two Codex sessions.** If it does, restate the RED evidence in the
  continuation handoff — Epic 1 lost TDD discipline at exactly that boundary. Commit per green step;
  that is the recovery mechanism, not a style preference.

### Project Structure (files to touch)

```
crates/sim-core/src/lib.rs          # UPDATE — DesignationKind, Rect, SimCommand, resources, apply_command, readers
crates/sim-core/src/save.rs         # UPDATE — designations + zones in SaveState
crates/sim-core/tests/scenario.rs   # UPDATE — rect normalization, clipping, clip-to-standable, cancel, determinism
crates/sim-core/tests/save_load.rs  # UPDATE — gate test now covers designations and zones
crates/protocol/src/lib.rs          # UPDATE — the wire change + literal tests
crates/simd/src/bridge.rs           # UPDATE — designation_kind both ways, rect, real lists in snapshot/delta
crates/simd/src/main.rs             # UPDATE — three drain arms, rewritten pause NOTE, read_inbound partial-line fix
crates/simd/tests/serve.rs          # UPDATE — designate/stockpile/cancel two-client test, paused intake, partial line
crates/tui/src/view.rs              # UPDATE — Mode, cursor, anchor, optimistic speed, hint bar, marker layers
crates/tui/src/palette.rs           # UPDATE — designation/zone/cursor cells, pinned and distinct
crates/tui/src/main.rs              # UPDATE — --key sequence parsing, state.speed from the wire
crates/tui/tests/client.rs          # UPDATE — the two instrument tests
_bmad-output/implementation-artifacts/mutations/3-1-give-the-order.sh  # NEW
_bmad-output/implementation-artifacts/deferred-work.md                # UPDATE — close the three items this story owns
```

### Previous story intelligence (2.4)

- Copy 2.4's daemon-side pattern exactly: command decoded in `read_inbound`, applied in the
  iteration-top drain, proven by a live-daemon test asserting the *consequence* over several real
  deltas. Parser-level tests proved nothing there and will prove nothing here.
- 2.4's hermetic `Daemon` harness (per-daemon temp cwd, removed on `Drop`) is in place — extend it,
  and keep every new test out of the repo tree.
- 2.4 landed on `main` via PR #10; branch from current `main` (`8bf4548`), which already carries the
  duplicate-dwarf-id load rejection and the SHIFT keymap pin.

### Verification

```bash
scripts/gate.sh
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/3-1-give-the-order.sh
```

Live instrument — the observable outcome, joining the two binaries no test can span:

```bash
cargo run -p simd &
cargo run -p tui -- --frames 4                            > /tmp/before.txt   # no markers
cargo run -p tui -- --frames 4 --key d,enter,l,l,j,enter  > /tmp/dig.txt      # dig markers appear
cargo run -p tui -- --frames 4 --key p,enter,l,l,enter    > /tmp/pile.txt     # stockpile markers appear
cargo run -p tui -- --frames 4 --key x,enter,l,l,j,enter  > /tmp/clear.txt    # both marks gone
rg -c '×' /tmp/before.txt /tmp/dig.txt /tmp/clear.txt
rg -c '≡' /tmp/pile.txt /tmp/clear.txt                                       # the eraser took the zone too
cargo run -p tui                                          # d, move, Enter, move, Enter — the mark appears live
```

Then, with the daemon paused (`space` first), designate again and confirm the mark still appears
while the tick is frozen — AC4's observable. Finally, with two clients attached, confirm a
designation from one shows in both.

Branch: `3-1-give-the-order`. Commit as `Völundr <jeicei75@gmail.com>`, one commit per green step,
imperative messages. Review-gated: no push, no PR.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.1] — user story, source ACs, and the
  `// NOTE:` recording clip-not-reject for stockpiles
- [Source: _bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md]
  — AD-10 (world-mutating commands ride the queue, consumed at loop-iteration start in arrival
  order), AD-2 (the loop never stops; commands apply while paused), AD-8 (designations and zones are
  full-resend, absence is deletion), AD-6 (vocabularies are enums, bridged by exhaustive match),
  AD-11 (SaveState carries designations and zones), AD-7, AD-9, plus the geometry convention that
  rects are inclusive of both corners on a single z-level
- [Source: _bmad-output/implementation-artifacts/epic-2-retro-2026-08-05.md] — Wolf's Option C ruling
  for the AD-10 consumer, the "3.1 is a wire change" finding, and the three deferred items scheduled
  into this story
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — the stale-speed compose trap
  [crates/tui/src/view.rs:180-195], `read_inbound`'s partial-line misreport (recorded as main.rs:270,
  now at main.rs:401-405 after 2.4), and the missing `Command::Quit` affordance [view.rs:222]
- [Source: _bmad-output/implementation-artifacts/2-4-the-world-endures.md] — the `assemble` single-site
  rule, the hermetic daemon harness, and the instrument's failure history
- [Source: AGENTS.md] — sabotage rule, honest reporting, bounded I/O, the codex self-gate

## Dev Agent Record

### Agent Model Used

OpenAI Codex (GPT-5), acting as Völundr.

### Debug Log References

- Initial RED, before sim vocabulary existed (`cargo test --offline -p sim-core --test scenario`):
  ```text
  error[E0432]: unresolved imports `sim_core::DesignationKind`, `sim_core::Rect`, `sim_core::SimCommand`
  error[E0599]: no method named `apply_command` found for struct `World`
  error: could not compile `sim-core` (test "scenario") due to 35 previous errors
  ```
- Normalization sabotage RED:
  ```text
  test reversed_rect_designates_the_normalized_inclusive_tiles ... FAILED
  assertion `left == right` failed
    left: []
   right: [(Pos { x: 1, y: 2, z: 4 }, Dig), (Pos { x: 1, y: 3, z: 4 }, Dig), (Pos { x: 2, y: 2, z: 4 }, Dig), (Pos { x: 2, y: 3, z: 4 }, Dig)]
  ```
- Bounds-clipping sabotage RED:
  ```text
  test designation_rect_clips_to_world_bounds ... FAILED
  assertion `left == right` failed
    left: [(Pos { x: -1, y: -1, z: -1 }, Channel), ..., (Pos { x: 1, y: 0, z: 0 }, Channel)]
   right: [(Pos { x: 0, y: 0, z: 0 }, Channel), (Pos { x: 1, y: 0, z: 0 }, Channel)]
  ```
- Overwrite-mapping sabotage RED:
  ```text
  test designate_overwrites_the_existing_kind ... FAILED
  assertion `left == right` failed
    left: [(Pos { x: 7, y: 8, z: 9 }, Dig)]
   right: [(Pos { x: 7, y: 8, z: 9 }, Channel)]
  ```
- Standability-filter sabotage RED:
  ```text
  test stockpile_keeps_exactly_the_standable_tiles ... FAILED
  assertion `left == right` failed
    left: [Pos { x: 10, y: 10, z: 10 }, Pos { x: 11, y: 10, z: 10 }, Pos { x: 12, y: 10, z: 10 }]
   right: [Pos { x: 10, y: 10, z: 10 }, Pos { x: 11, y: 10, z: 10 }]
  ```
- Cancel-isolation sabotage RED:
  ```text
  test each_eraser_leaves_the_other_mark_kind_untouched ... FAILED
  assertion `left == right` failed
    left: []
   right: [Pos { x: 10, y: 10, z: 10 }]
  ```
- Remove-isolation and remove-no-op sabotage REDs:
  ```text
  test each_eraser_leaves_the_other_mark_kind_untouched ... FAILED
  assertion `left == right` failed
    left: []
   right: [(Pos { x: 10, y: 10, z: 10 }, Channel)]

  test each_eraser_leaves_the_other_mark_kind_untouched ... FAILED
  assertion failed: world.zones().is_empty()
  ```
- Save/load initial RED and four persistence-seam sabotages:
  ```text
  test save_load_then_tick_matches_never_saved ... FAILED
  assertion `left == right` failed
    left: []
   right: [(Pos { x: 2, y: 1, z: 2 }, Channel), ..., (Pos { x: 4, y: 3, z: 2 }, Channel)]

  # to_save drops zones / from_save discards zones (each run independently)
  assertion `left == right` failed
    left: []
   right: [Pos { x: 115, y: 84, z: 15 }]

  # to_save drops designations / from_save discards designations (each run independently)
  assertion `left == right` failed
    left: []
   right: [(Pos { x: 2, y: 1, z: 2 }, Channel), ..., (Pos { x: 4, y: 3, z: 2 }, Channel)]
  ```
- Protocol wire initial RED:
  ```text
  error[E0422]: cannot find struct, variant or union type `Designation` in this scope
  error[E0599]: no variant named `Designate` found for enum `Command`
  error: could not compile `protocol` (lib test) due to 26 previous errors
  ```
- Atomic wire-migration seam RED:
  ```text
  error[E0004]: non-exhaustive patterns: `protocol::Command::Designate { .. }`,
  `protocol::Command::CancelDesignation { .. }`, `protocol::Command::PlaceStockpile { .. }`
  and 1 more not covered
  error[E0308]: mismatched types: expected `Designation`, found `()`
  ```
- Protocol designation-kind mapping sabotage RED:
  ```text
  test tests::every_material_and_tile_variant_has_a_pinned_wire_name ... FAILED
  assertion `left == right` failed
    left: "\"channel\""
   right: "\"dig\""
  ```
- Protocol command discriminator sabotages RED (each rename run independently):
  ```text
  unknown variant `designate`, expected ... `mark` ...
  unknown variant `cancel_designation`, expected ... `erase` ...
  unknown variant `place_stockpile`, expected ... `store` ...
  unknown variant `remove_stockpile`, expected ... `unstore`
  missing field `mode`
  ```
- Bridge direction sabotages RED:
  ```text
  test bridge::tests::every_designation_kind_maps_to_its_named_wire_variant ... FAILED
    left: "\"channel\""
   right: "\"dig\""

  test bridge::tests::every_designation_kind_maps_to_its_named_wire_variant ... FAILED
    left: Channel
   right: Dig
  ```
- Daemon decision-seam REDs with decoded commands deliberately discarded:
  ```text
  test designation_and_stockpile_changes_reach_both_clients ... FAILED
  daemon never emitted expected marks; observed [(1, [], []), (2, [], []), (3, [], []),
  (4, [], []), (5, [], []), (6, [], []), (7, [], []), (8, [], []), (9, [], []), (10, [], [])]

  test designation_is_applied_while_tick_is_paused ... FAILED
  paused daemon discarded designation
  ```
- Partial-line reporting RED:
  ```text
  test unterminated_partial_line_is_not_reported_as_overflow ... FAILED
  partial line was falsely reported as overflow: client line exceeded 65536 bytes; closing connection
  ```
- Two-row layout RED:
  ```text
  test view::tests::status_and_hint_occupy_the_bottom_two_rows ... FAILED
  assertion failed: status.starts_with("tick 0  normal  z 0/0  dwarves 0")
  ```
- Initial view-state RED:
  ```text
  error[E0560]: struct `view::ViewState` has no field named `mode`
  error[E0560]: struct `view::ViewState` has no field named `cursor`
  error[E0560]: struct `view::ViewState` has no field named `anchor`
  error[E0560]: struct `view::ViewState` has no field named `speed`
  ```
- Marker z-filter RED:
  ```text
  test view::tests::marks_draw_only_on_the_viewed_level ... FAILED
    left: ' '
   right: '×'
  ```
- Mode/anchor and z-lock REDs:
  ```text
  test view::tests::mode_keys_enter_only_from_normal_and_escape_backs_out_one_level ... FAILED
    left: Ignore
   right: Redraw

  test view::tests::z_keys_work_unanchored_and_are_ignored_while_anchored ... FAILED
    left: Redraw
   right: Ignore
  ```
- Cursor/camera boundary RED:
  ```text
  test view::tests::cursor_moves_clamps_and_pans_camera_only_after_crossing_the_window_edge ... FAILED
    left: (5, 5)
   right: (7, 5)
  ```
- Commit/action REDs:
  ```text
  test view::tests::second_enter_commits_each_single_command_mode_and_stays_in_mode ... FAILED
    left: Ignore
   right: Redraw

  error[E0599]: no variant, associated function, or constant named `Commands` found for enum `view::Action`
  ```
- Optimistic-speed sabotage RED:
  ```text
  test view::tests::optimistic_speed_keys_compose_before_a_wire_update ... FAILED
    left: Normal
   right: Fast
  ```
- Layer-order RED:
  ```text
  test view::tests::marker_layers_follow_terrain_zone_designation_entity_pending_cursor_order ... FAILED
    left: '☺'
   right: 'd'
  ```
- Palette collision sabotage RED:
  ```text
  test palette::tests::every_look_is_pinned ... FAILED
  left: [... Cell { glyph: '×', fg: (246, 242, 226) }, ...]
  right: [... Cell { glyph: '+', fg: (246, 242, 226) }, ...]
  ```
- Hint/global-key REDs:
  ```text
  test view::tests::hint_bar_names_every_modes_keys_and_fits_eighty_columns ... FAILED
  assertion failed: hint.starts_with("dig:")

  test view::tests::speed_save_load_and_quit_keys_remain_global_in_every_mode ... FAILED
    left: Ignore
  right: Command(Save)
  ```
- Instrument sequence RED:
  ```text
  test key_sequence_designates_and_the_echoed_marker_reaches_expected_columns ... FAILED
  tui did not connect to stub daemon within 3s
  ```
- Instrument key-name mapping sabotage RED:
  ```text
  test tests::every_instrument_key_name_is_pinned ... FAILED
  assertion `left == right` failed: wrong mapping for "h"
    left: Some(Char('l'))
   right: Some(Char('h'))
  ```
- Live-instrument queued-delta RED, reproduced first against the real daemon and then pinned in the
  stub with an aligned large snapshot and the bounded 17-message pre-command window. Setting the
  production drain bound from 17 to 0 reproduced the same RED:
  ```text
  test key_sequence_designates_and_the_echoed_marker_reaches_expected_columns ... FAILED
  send mark delta: Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }
  mark stub daemon thread panicked: Any { .. }
  test result: FAILED. 0 passed; 1 failed; 10 filtered out
  ```

### Completion Notes List

- AC1–AC4: added ordered designation/zone resources, normalized and clipped inclusive rectangle
  application, standability-filtered stockpiles, independent erasers, readers, pause-safe plain
  `World::apply_command`, and deterministic scenario coverage. `cargo test --offline -p sim-core`
  and sim-core clippy are green.
- AC8: `SaveState` now carries sorted designation and zone lists through the single `assemble`
  path; the AD-11 gate compares both lists against a never-saved control after each of 200 steps.
- AC5–AC7/AC13: replaced wire placeholders with pinned typed shapes and four commands; added
  exhaustive bridge conversions, iteration-top command consumption, authoritative mark lists,
  two-client consequence tests, paused-intake coverage, and honest partial-line EOF handling.
- AC9–AC12: added modal rectangle input, clamped cursor/camera follow, one-level escape, narrow
  two-command erasure, optimistic local speed with wire overwrite, two-row status/hints, ordered
  marker layers, and a glyph-distinct pinned palette. TUI tests and clippy are green.
- AC14: `--key` now replays comma-separated named sequences through the real keymap/write path;
  the stub-daemon positive capture verifies the exact command and echoed marker columns, while the
  no-key negative control proves markers are not unconditional. The `NO_COLOR` warning now states
  that glyph-distinct markers remain valid evidence.
- AC15 development mutation pass: all 32 authored sabotages were killed. The prescribed clean,
  final gate, final mutation pass, live checks, and self-review remain before review status.
- Live verification found that four deltas already buffered with the large connect snapshot could
  consume the entire keyed capture before the command consequence arrived. The headless instrument
  now nonblockingly drains up to simd's bounded 16-message client queue plus one in-flight write
  before replaying world-mutating key sequences; it stops immediately on `WouldBlock` or a partial
  line, and its existing save/load and speed capture semantics remain unchanged. The strengthened
  positive test and a 32nd mutation pin the bound.
- AD-10 under-listing: `remove_stockpile` is a fourth world-mutating command added on Wolf's call,
  so `epics.md` and the architecture spine's command table now list three where there are four.
  Those two planning documents were deliberately not reconciled inside this implementation story;
  they need one coordinated correction.

### File List

- crates/sim-core/src/lib.rs
- crates/sim-core/src/save.rs
- crates/sim-core/tests/save_load.rs
- crates/sim-core/tests/scenario.rs
- crates/protocol/src/lib.rs
- crates/simd/src/bridge.rs
- crates/simd/src/main.rs
- crates/simd/tests/serve.rs
- crates/tui/src/main.rs
- crates/tui/src/palette.rs
- crates/tui/src/view.rs
- crates/tui/tests/client.rs
- _bmad-output/implementation-artifacts/mutations/3-1-give-the-order.sh
- _bmad-output/implementation-artifacts/deferred-work.md
- _bmad-output/implementation-artifacts/3-1-give-the-order.md

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-05 | Story created |
| 2026-08-05 | Added stockpile removal on Wolf's call: `remove_stockpile` as a fourth world-mutating command, `x` becomes a two-command eraser. Scope beyond epics.md — the epic text and the spine's command table still list three. |
