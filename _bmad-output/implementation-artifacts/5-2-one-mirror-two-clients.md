---
baseline_commit: 305aa03
model: claude-opus-5[1m]  # default Opus; 1M-context variant, as at 5.1
---

# Story 5.2: One Mirror, Two Clients

Status: done

## Story

As a developer,
I want a `client-core` crate that owns the world mirror and all snapshot/delta application, with the TUI already running on it,
so that both clients read one truth, and the mirror's contract is proven against the client we can byte-assert before the Bevy client bets on it.

## Acceptance Criteria

### The crate and the graph

1. A fifth crate `client-core` exists at `crates/client-core`, is a workspace member, has
   `protocol` and `thiserror` as its only **normal** dependencies, and carries
   `#![forbid(unsafe_code)]`. It contains no I/O of any kind — no `std::net`, no `std::fs`, no clock.
   *(Wording corrected at review, 2026-08-11. As first written this AC said "depends on `protocol`
   and `thiserror` only", which contradicts AC17: wire-JSON-literal tests need a parser in test
   scope, and "no change to `protocol`" blocks putting one there. A `serde_json` dev-dependency is
   the resolution — the same pattern `protocol` itself uses. Dev-dependencies do not enter the
   shipped graph, so AC2 and the gate probe are unaffected.)*
2. The workspace dependency graph is exactly `simd → sim-core`, `simd → protocol`,
   `client-core → protocol`, `tui → protocol`, `tui → client-core`. No other edge exists, read off
   the five `Cargo.toml` files and the two gate probes. (`gui` arrives in 5.3; this story adds no
   `gui` anything.)
3. `scripts/gate.sh` gains a `client-core` no-`sim-core`-edge probe, a sibling of the existing `tui`
   probe: a `cargo tree -p client-core` match on `sim-core` is a gate failure. Do **not** add the
   `gui` probe — it lands with that crate in 5.3.

### The mirror's contract (AD-18)

4. The mirror's shape is `client-core`'s API and is defined nowhere else. It exposes: `dims()`,
   `tick()`, `speed()`, `tile(pos)`, `entities()`, `items()`, `designations()`, `zones()`,
   `previous_entity(id)`, and `changes()`. Entities and items are keyed by sim `Id` and iterate in
   **ascending id order**.
5. AD-8's client-side semantics live in `client-core` and only there. A delta's `tiles` are a dirty
   set applied in place; `entities`, `items`, `designations`, `zones` and `speed` are authoritative
   full replacements, so **an id absent from a delta's list is deleted from the mirror**.
6. A snapshot is a world replacement: it replaces tiles, every collection, tick and speed, and
   **clears previous-tick state** — after applying a snapshot, `previous_entity(id)` is `None` for
   every id, including ids the snapshot itself carries.
7. Previous-tick state covers **entities only**. There is no previous-tick tile, item, designation
   or zone state on the mirror, and no API that would return one. After a delta,
   `previous_entity(id)` returns that entity as it stood at the tick before, for every id present in
   both ticks.
8. `changes()` reports, for the most recently applied message: dirty tile positions, and entity ids
   partitioned into spawned / despawned / changed. The partition is exclusive — an id appears in at
   most one of the three — and ids are ascending in each. An entity present in both ticks with an
   identical `Entity` value appears in **none** of them.
9. Ingesting a snapshot whose `tiles` length disagrees with its `dims` fails with a typed
   `client-core` error naming both numbers; it never panics and never produces a mirror. This is the
   only error the crate defines.

### The rect contract (AD-18)

10. `client-core` provides the single rect normalization helper both clients use. It produces
    inclusive corners with `min ≤ max` per axis on **one z-level**, and single-z is structural in the
    signature rather than checked at runtime. *(A prescribed mechanism, deliberately: AD-18 makes
    this helper the one shared contract point between two clients, and a signature that cannot
    express a multi-z rect is what stops the second client re-deriving the rule.)*
11. `simd` validates every rect arriving on the wire and **logs-and-drops** violations — a rect whose
    `min.z != max.z`, or whose `min > max` on any axis, is logged to stderr and never reaches
    `World::apply_command`. The daemon keeps ticking and the client keeps its connection.
12. `sim-core::World::apply_command`'s existing min/max normalization and dims clipping are **not
    modified**. A test shows a rejected rect leaves designations and zones unchanged, and that a
    well-formed rect still applies exactly as it does today.

### The TUI adoption (AD-13)

13. `tui` consumes `client-core` for all world state. Its in-crate client state is **retired, not
    kept as a second path**: `tui::apply`, `tui::validate_snapshot` and `view::tile_index` no longer
    exist, and `protocol::Snapshot` is no longer held as mutable client state anywhere in `tui`.
14. `view::initial`, `view::opening_z` and `view::render` take the mirror instead of `&Snapshot`.
    `tui` never diffs wire messages itself.
15. The rendered frame is unchanged. Every existing `tui` test that pins rendered output — the
    `view` unit tests and every real-binary capture test in `crates/tui/tests/client.rs` — passes
    with its **assertions unmodified**. Signature and construction changes to reach the new API are
    expected; a changed expected value is not.
16. `tui` builds its designate/cancel/stockpile rects through the `client-core` helper. No rect is
    constructed inline in `tui` any more.

### Evidence

17. `client-core` is asserted headless and byte-exact in `cargo test` against a recorded snapshot and
    delta sequence: the resulting mirror equals a hand-written expected mirror, including
    deletion-by-absence and snapshot-as-reset. The recorded sequence is written as wire JSON
    literals, not built from `protocol` structs, so a wire-shape change breaks it.
18. The live cross-check: `tui --frames 6 --z 9` against a real `simd` reports non-zero counts of
    every glyph the baseline reports, in the same terrain figures. See Verification for the executed
    baseline and for exactly which half of that capture is byte-stable and which is not.
19. A sabotage table exists at
    `_bmad-output/implementation-artifacts/mutations/5-2-one-mirror-two-clients.sh` and every
    mutation in it is KILLED, with the RED output pasted verbatim into the Dev Agent Record.

## Tasks / Subtasks

- [x] **Task 1 — Scaffold the crate and the gate probe** (AC: 1, 2, 3)
  - [x] `crates/client-core/Cargo.toml`: `protocol = { path = "../protocol" }` and
        `thiserror.workspace = true`. Add `thiserror = "2.0.19"` to the root
        `[workspace.dependencies]` — **justification for the closed stack:** the mirror rejects an
        inconsistent snapshot, and a typed error is what lets `tui` and `gui` (both `anyhow`) add
        context without string-matching; the spine already names `thiserror` for `client-core`, and
        2.0.19 is already in `Cargo.lock` as a transitive dep so the offline sandbox needs no fetch.
  - [x] Add `"crates/client-core"` to the root `members` list.
  - [x] `scripts/gate.sh`: copy the `tui has no sim-core edge` block (`gate.sh:73-88`) as
        `client-core has no sim-core edge`. It is **inverted** — an `rg` match is the FAILURE.
        Add it, do not generalise the existing one into a loop; a third probe arrives in 5.3 and
        that is when a loop earns itself.
  - [x] Test: `cargo tree -p client-core` shows `protocol` and `thiserror` and nothing else.

- [x] **Task 2 — The mirror: shape, ingestion, AD-8 semantics** (AC: 4, 5, 6, 9)
  - [x] `Mirror::from_snapshot(Snapshot) -> Result<Mirror, MirrorError>` and
        `apply_snapshot(&mut self, Snapshot) -> Result<(), MirrorError>`. The tiles-vs-dims check
        from `tui/src/main.rs:462-479` **moves here** — copy its error text, it already names both
        numbers.
  - [x] `apply_delta(&mut self, Delta)`. Port the dirty-tile bounds guard from
        `tui/src/main.rs:495-513` verbatim in behaviour: an out-of-bounds tile change is skipped, not
        an error.
  - [x] Entities and items in `BTreeMap<u32, _>`; designations and zones as `Vec` (they have no ids
        and are keyed by position — do not invent one).
  - [x] `tile(pos: [i32; 3]) -> Option<Tile>`, bounds-checked, owning the flat row-major index rule
        (`x + y*dims.x + z*dims.x*dims.y`). This retires `view::tile_index`.
  - [x] Tests for AC5 and AC6 as **negative** assertions: an entity present in tick N and absent from
        tick N+1's delta is gone from `entities()`; an item likewise; a snapshot applied over a
        populated mirror leaves nothing of the old world behind.

- [x] **Task 3 — Previous-tick entities and change info** (AC: 7, 8)
  - [x] `previous_entity(id) -> Option<&Entity>`, populated only by `apply_delta`, cleared by
        `apply_snapshot`. Exactly one generation is retained — the mirror must not grow per tick.
  - [x] `changes() -> &Changes` with `tiles: Vec<[i32;3]>`, `spawned/despawned/changed: Vec<u32>`,
        all ascending, the three id lists mutually exclusive.
  - [x] **These two have no live caller in this story** — the TUI re-renders whole frames and
        must keep doing so (see Key decisions). Their ACs are therefore written against the seam's
        own decision surface, and the tests must assert the branch-changing negatives: an unchanged
        entity is in none of the three lists; a despawned id is in `despawned` and **not** in
        `spawned`; after a snapshot `previous_entity` is `None` even for ids the snapshot carries.
  - [x] Add the Deferred note naming 5.3 as the wiring story (Task 8).

- [x] **Task 4 — The rect helper and simd's validation** (AC: 10, 11, 12)
  - [x] `client_core::rect_on_level(a: (i32, i32), b: (i32, i32), z: i32) -> protocol::Rect` —
        two corners plus one level, so a multi-z rect is **unrepresentable** rather than rejected.
  - [x] `simd`: validate in the client reader thread at `main.rs:675-683`, immediately after
        `from_str::<protocol::Command>` succeeds and **before** `command_tx.send`. Reuse the existing
        malformed-input convention there (`eprintln!` + `excerpt`) — it is already the site for
        "well-formed bytes, invalid content".
  - [x] Violations are `min.z != max.z` or `min[i] > max[i]` on any axis. Commands with no rect
        (`SetSpeed`, `Save`, `Load`, `Quit`) pass through untouched.
  - [x] Do **not** touch `sim_core::World::apply_command` (`lib.rs:1290-1320`). Its own min/max swap
        and dims clipping stay as defence in depth; a `simd` test asserts a rejected rect adds zero
        designations and zero zones, and that today's well-formed rects still apply unchanged.
  - [x] Test in `crates/simd/tests/serve.rs`: send an inverted rect and a two-z rect over a real
        socket, assert the daemon logs, keeps ticking and keeps the client connected, and that the
        following delta carries no new mark.

- [x] **Task 5 — Adopt the mirror in `tui`** (AC: 13, 14, 15, 16)
  - [x] `crates/tui/Cargo.toml`: add `client-core = { path = "../client-core" }`.
  - [x] `main.rs`: delete `apply` (`:495-520`) and `validate_snapshot` (`:462-479`). `read_message`
        keeps decoding and keeps `MAX_SNAPSHOT_BYTES` — **networking stays in `tui`**; the mirror is
        built from the decoded snapshot at `main.rs:164-165` and fed by the interactive loop
        (`:259-268`) and `stream_frames` (`:395-402`).
  - [x] The error site moves: today a tiles/dims mismatch fails inside `read_snapshot` in the reader
        thread; after the move it fails when the mirror is built. Relocate the unit test
        `rejects_a_snapshot_whose_tiles_do_not_match_dims` (`main.rs:679-695`) into `client-core`,
        and keep a `tui` test proving the connect path still surfaces that error rather than
        rendering. **Do not simply delete it.**
  - [x] Same for `applies_dirty_tiles_and_replaces_authoritative_fields` (`main.rs:609-662`): it is
        the existing test of the function being moved, so it **relocates into `client-core`** and
        grows the absence-is-deletion cases of Task 2. Deleting it alongside `apply` would remove the
        only coverage the behaviour has ever had. `reads_one_snapshot_line`, `reads_one_delta_line`
        and the garbage/EOF cases stay in `tui` — `read_message` is not moving.
  - [x] `view.rs`: `initial`, `opening_z` and `render` take `&Mirror`. Delete `tile_index`
        (`:524-526`) and route every read through `mirror.tile(..)`. `ViewState` is camera/cursor/
        mode and does not move — it is client-local presentation, not world state.
  - [x] `view.rs:406-417`: replace the inline `Rect` construction with `rect_on_level`.
  - [x] Keep the four draw passes and their order exactly as they stand (`view.rs:193-251`):
        zones, designations, items, emitters, dwarves; crowd/carrier contention and the status line's
        `dwarves N` count stay dwarf-only.

- [x] **Task 6 — Byte-exact mirror tests** (AC: 17)
  - [x] `crates/client-core/tests/` (or `#[cfg(test)]` in the crate — either is fine, pick one):
        a recorded sequence of one snapshot line and three delta lines as **wire JSON string
        literals**, modelled on `protocol/src/lib.rs:175-196` and `tui/src/main.rs:533-545`. Deserialize
        them, apply in order, assert the whole mirror against hand-written expected values.
  - [x] The sequence must exercise, in this order: an entity moving; an entity **disappearing** from
        a delta's list; an item appearing and disappearing; a dirty tile landing; and a second
        snapshot resetting the lot.
  - [x] Assert previous-tick and `changes()` at each step, including the negatives from Task 3.

- [x] **Task 7 — The observability instrument** (AC: 18)
  - [x] The instrument is the existing `tui --frames N --z N`. **Do not build a second one.** Its
        exact command, the executed baseline numbers, and which half of the capture is byte-stable
        are in Verification below.
  - [x] Run it before (on `main`) and after adoption, in the **same shell and same terminal** — the
        viewport is the operator's real terminal size, so counts scale with it and two runs in
        different terminals are not comparable. Record both outputs in the Dev Agent Record with the
        actual numbers.
  - [x] The byte-exact half of AC18 is carried by AC15: the real-binary capture tests in
        `crates/tui/tests/client.rs` drive a deterministic stub daemon and already pin glyph columns,
        positions and tick sequences. They are the reproducible "identical before and after" guard.
        The live run is a range check, and the story says so rather than claiming byte identity it
        cannot have.

- [x] **Task 8 — Deferred note and the gate** (AC: 19)
  - [x] Append a `## Deferred from: story 5.2` section to
        `_bmad-output/implementation-artifacts/deferred-work.md` recording that `previous_entity()`
        and `changes()` ship with no live caller, that their decision surface is tested, and that
        **5.3 is the wiring story** (`gui` reconciliation and AD-15 interpolation).
  - [x] Write `_bmad-output/implementation-artifacts/mutations/5-2-one-mirror-two-clients.sh`
        following the existing files' shape and run `scripts/mutate.sh` against it. Minimum set:
        absence stops meaning deletion in `apply_delta` (an entity omitted from a delta survives);
        `apply_snapshot` stops clearing previous-tick state; `changes()` reports an unchanged entity
        as changed; the `simd` rect validation is removed (an inverted rect reaches the sim); the
        mirror's entity iteration is reversed. Paste the RED output verbatim.
  - [x] `scripts/gate.sh` green — including the new probe. Branch `5-2-one-mirror-two-clients`,
        small commits, imperative messages, author `Völundr <jeicei75@gmail.com>`. Push/PR only on
        Wolf's explicit yes.

### Review Findings

Code review 2026-08-11. Four layers, all four reported, no coverage holes. Zero HIGH findings.
Layer attribution is in brackets: `[blind]` Blind Hunter (Sonnet, `client-core`), `[edge]` Edge Case
Hunter (Sonnet, shells), `[auditor]` Acceptance Auditor (Opus), `[feature]` Feature Auditor (Opus).

- [x] [Review][Decision — RESOLVED: keep as-is, document the contract] Wolf's ruling 2026-08-11.
      `changes()` stays delta-only; no production code changed. The contract is now recorded in
      `deferred-work.md` under "Deferred from: story 5.2" so 5.3 reads it before writing a
      reconciler. Rejected: populating `spawned` on a snapshot (asymmetric with tiles), and
      populating `tiles` with every position (~6MB per snapshot on the shipped world, for a
      consumer that does not exist yet). Original finding: `changes()` reports nothing after a
      snapshot, and under-reports dropped dirty tiles — [auditor+feature+blind, MED] `apply_snapshot` resets `changes` to `Default`
      (`client-core/src/lib.rs:52,166`), pinned by `tests/mirror.rs:139-142`: after a reset snapshot
      carrying entity 99, all four lists are empty. AC8 says `changes()` reports "for the most
      recently applied message" and a snapshot IS a message, so empty is only correct under a
      delta-only reading the AC never states. A 5.3 gui reconciler driven by `changes()` would create
      no nodes for a world it just received in full — an empty Bevy window beside a correct TUI. The
      consumer also has no positive signal that a snapshot happened; `previous_entity == None`
      everywhere is the only tell, and it costs a full scan. Same seam, second half: an out-of-bounds
      dirty tile is skipped (`lib.rs:68-73`) AND omitted from `changes().tiles`, so a consumer
      repainting only `changes().tiles` silently diverges from one repainting everything. The skip
      itself is spec-mandated (Task 2, "skipped, not an error") and is NOT the finding; the
      unreported-ness is new with `changes()`. Options: (a) after a snapshot populate `spawned` with
      every id and `tiles` with every position; (b) add an explicit reset signal; (c) keep as-is and
      document the delta-only contract in the deferral so 5.3 does not discover it by redesign.
- [x] [Review][Decision — RESOLVED: AC1 means normal dependencies] Wolf's ruling 2026-08-11. The
      `serde_json` dev-dependency stands; AC1's wording has been corrected above to say "only
      **normal** dependencies", with the AC1/AC17 conflict recorded inline so the next reader does
      not re-derive it. No code changed. Original finding: AC1 and AC17 cannot both be met
      literally — [auditor, MED]
      `crates/client-core/Cargo.toml:10-11` carries `serde_json` as a **dev**-dependency. AC1 says the
      crate "depends on `protocol` and `thiserror` only" and Task 1's verification command says
      `cargo tree` must show "nothing else"; AC17 requires the recorded sequence be wire JSON string
      literals, which needs a parser in test scope, and "no change to `protocol`" forbids putting a
      helper there. The dev chose the dev-dependency (the same pattern `protocol` itself uses) and
      reworded the completion note to "normal dependencies" rather than surfacing the conflict.
      Options: (a) rule that AC1 means normal dependencies and correct the AC's wording;
      (b) require the dev-dep be removed and AC17 met another way.

- [x] [Review][Patch] APPLIED — Mutation evidence re-run against final source; a sixth mutation
      (`rect helper stops normalizing corners`) added. All six KILLED, output pasted in the Dev
      Agent Record; the stale dev-time block is kept beneath it, labelled superseded.
- [x] [Review][Patch] APPLIED — Self-referential rect assertions replaced with literal oracles
      [crates/tui/src/view.rs:1430, :1476], each carrying a `// NOTE:` saying why it is a literal.
      The new mutation proves they now kill a normalization change; the version they replaced would
      have survived it.
- [x] [Review][Patch] APPLIED — `tile_index` now compares as `u32` via `u32::try_from` instead of
      casting dims to `i32` [crates/client-core/src/lib.rs:170-185], with a `// NOTE:` naming the
      wrap it avoids, plus a covering test for negative and out-of-range coordinates.
      **`render`'s `unwrap_or(Tile::Empty)` deliberately left alone** — with the cast fixed the two
      bounds checks agree so it cannot fire, it is load-bearing for the peek-below path (which
      relies on `z = -1` reading as out-of-range), and touching `render` would put AC15 at risk for
      no behavioural gain.
- [x] [Review][Patch] APPLIED — Instrument-stability caveat corrected: the entity glyphs and tick
      ARE stable while the daemon-start-to-connect delay and frame count are held fixed, so all
      five counts are a regression guard rather than three of them being noise to ignore.

- [x] [Review][Defer] Duplicate entity/item ids silently collapse to last-one-wins
      [crates/client-core/src/lib.rs:75-78, 101, 151-162] — deferred; the obvious fix adds a second
      `MirrorError` variant, which AC9 forbids ("the only error the crate defines")
- [x] [Review][Defer] `rect_is_valid` is a structural gate, not a dims gate
      [crates/simd/src/main.rs:714-716] — deferred, AC11 as written; bounds safety still lives in
      `sim-core`'s unchanged clamp
- [x] [Review][Defer] Ascending `items()` order has no test that would fail if it flipped
      [crates/client-core/tests/mirror.rs:82-85] — deferred, low tail
- [x] [Review][Defer] A NOTE documenting a still-live limitation was deleted with code that did not
      describe it [crates/tui/src/main.rs:255-258] — deferred, comment-only, behaviour matches `main`
- [x] [Review][Defer] Pre-existing `simd` test flakes under parallel test execution
      [crates/simd/tests/serve.rs] — deferred, pre-existing and not attributable to this diff
- [x] [Review][Defer] AC17's "equals a hand-written expected mirror" is unmeetable from `tests/`
      [crates/client-core/tests/mirror.rs:45-143] — deferred, AC phrasing; the accessor assertions
      are thorough and the ordering mutant died via this file

Dismissed as noise (2): the out-of-bounds dirty-tile **skip** itself, which Task 2 mandates verbatim;
and the inline `protocol::Rect` literals at `crates/tui/tests/client.rs:671,979`, which are
independent wire-level oracles and the correct pattern — AC16 governs production rect construction,
and `rg 'Rect\s*\{' crates/tui/src/` returns zero hits.

**Layer convergence** (Epic 2 action item; R1 territory-split measurement): 3 of 12 merged findings
were raised independently by two layers — `changes()`-after-snapshot (auditor+feature), the `i32`
dims cast (blind+auditor), and the out-of-bounds dirty tile (blind+feature). Against Epic 3's clean
four-layer story (3.2), which produced 1 convergence in 8.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No `gui`, no Bevy, no `gui` gate probe.** 5.3 adds the crate, the window and its probe.
- **No lighting, no interpolation, no blending.** The mirror *provides* previous-tick entities;
  consuming them is 5.3/5.4. Writing a blend here would be building the consumer this story
  deliberately defers.
- **No change to `protocol`.** This story touches no wire shape and no wire vocabulary. AD-16's
  sanctioned diff was spent in full at 5.1.
- **No change to `sim-core`.** `apply_command`'s normalization stays exactly as it is.
- **No new instrument.** `tui --frames N --z N` exists and is the instrument.
- **No performance work, no caching, no dirty-rect rendering.** Nothing has been measured, and a
  render that skips frames breaks the capture instrument this story is judged by.
- **No sign-off gate.** UX-DR22 binds 5.4, explicitly not 5.1–5.3.
- **Nothing else from `deferred-work.md`.** None of the open 5.1 entries fire here.

### What already exists (build on it, do not re-derive)

- **There is no client world type today.** `tui` holds a `protocol::Snapshot` as mutable state and
  mutates it in `main.rs::apply` (`:495-520`) — 25 lines. That function IS the thing being moved into
  `client-core`, and its dirty-tile bounds guard is already correct; port the behaviour, do not
  redesign it.
- **`validate_snapshot` (`main.rs:462-479`) is the only ingestion validation that exists.** It is the
  whole content of `MirrorError`.
- **The wire's entity order is already globally ascending by id** — measured live at story-creation:
  `0..4` dwarves, `5..9` emitters. But `bridge::snapshot` builds it as
  `dwarves().chain(emitters())` (`bridge.rs:29,84`), so the ordering is an accident of id allocation,
  not a guarantee. Keying by id makes it structural. See Key decisions for why the TUI is
  insensitive to it anyway.
- **`glyph_columns_for` / `glyph_positions` in `tui/tests/client.rs` derive expected columns from the
  captured line width** (`client.rs:592-596, 608`), which is why those tests survive a varying
  terminal size. Keep that property when touching them.
- **`view.rs`'s whole test module builds its world through one helper**, `empty_snapshot(dims)`
  (`view.rs:537`), and there are only two `Snapshot { .. }` literals in the file. Point that helper
  at the mirror and the rest of AC15 falls out — this is why "assertions unmodified" is a meetable
  bar here rather than a wish.
- **The `simd` client reader already has the log-and-drop site** (`main.rs:675-683`, with `excerpt`
  at `:697`). Rect validation belongs there, not in a new layer.

### Key decisions & traps

- **`tui` keeps all networking; `client-core` is protocol-only and has zero I/O.** The reader thread,
  `MAX_SNAPSHOT_BYTES`, the timeouts and the channel all stay in `tui/src/main.rs`. A `std::net`
  import in `client-core` is an AD-13 breach, not a convenience.
- **The mirror is not `Snapshot` with methods.** Keying entities and items by `Id` is the point of
  AD-18 — a `Vec<Entity>` with a lookup helper reproduces the shape the gui cannot use and leaves the
  "keyed by sim Id" AC met only in prose.
- **Ascending-id iteration is a change of *guarantee*, not of behaviour.** The TUI draws emitters and
  dwarves in separate kind-filtered passes (`view.rs:218-251`), so intra-list order only decides
  last-write-wins between two entities of the same kind on one cell — and both orders are ascending
  within kind today. Pin the ascending order with a test; do not assume the wire will keep supplying
  it.
- **`previous_entity()` and `changes()` have no live caller in this story, by design.** The obvious
  candidate — driving `needs_redraw` from `changes()` — is a trap: `--frames N` emits one frame per
  server message, so a client that skipped "unchanged" frames would change the instrument's output
  and break AC15/AC18. Leave the seam inert, test its decisions, record the deferral.
- **AD-15 is enforced by clearing, not by a flag.** After a snapshot `previous_entity` is `None`, so a
  future interpolating client has nothing to blend from and structurally snaps. Do not add a
  `reset: bool` — the absence is the signal.
- **A live before/after capture cannot be byte-identical, and claiming it would be false evidence.**
  Measured at story-creation: terrain glyph counts are stable across runs (`│=6 ♠=48`) because
  terrain is seeded; dwarf and emitter counts are not (`☺` 22 vs 30, `†` 24 vs 21, `♨` 6 vs 3 across
  two runs of the identical command) because the client connects at a wall-clock-dependent tick — the
  capture opened at tick 31 here and tick 0 never appears. The status line carries that tick. The
  byte-exact guard is the stub-daemon capture suite (AC15); the live run is a range check (AC18).
- **Counting glyphs with `tr -cd` lies quietly** — `tr` works on bytes and the box glyphs share
  leading UTF-8 bytes. Use `grep -o '<glyph>' | wc -l`.
- **`--z` must be pinned at 9.** The camp is at z 9 on the shipped seed and `opening_z` picks z 19,
  which shows no camp, no dwarves and no lights while the status line still reads `dwarves 5`. This is
  a recorded 5.1 decision, not a bug to fix here.
- **`simd` has no seed flag.** The seed is the constant `SEED` (`simd/src/main.rs:20`) and the port is
  positional only (`simd 7413`).
- **Adding a gate probe is cheap to get subtly wrong** — the existing block is *inverted* (a match is
  the failure) and lives outside the `run` helper for that reason. Copy its shape.
- When torn between simple and general, pick simple and leave a `// NOTE:` naming the limitation.

### Project Structure (files to touch)

```
Cargo.toml                              UPDATE  members += client-core; workspace dep thiserror 2.0.19
crates/client-core/Cargo.toml           NEW     protocol + thiserror only
crates/client-core/src/lib.rs           NEW     Mirror, Changes, MirrorError, rect_on_level
crates/client-core/tests/mirror.rs      NEW     byte-exact recorded snapshot + delta sequence
crates/tui/Cargo.toml                   UPDATE  += client-core
crates/tui/src/main.rs                  UPDATE  delete apply + validate_snapshot; build/feed the mirror
crates/tui/src/view.rs                  UPDATE  initial/opening_z/render take &Mirror; delete tile_index;
                                                rect_on_level at :406-417
crates/tui/tests/client.rs              UPDATE  only if a signature forces it — assertions unchanged (AC15)
crates/simd/src/main.rs                 UPDATE  rect validation at the reader's log-and-drop site
crates/simd/tests/serve.rs              UPDATE  inverted-rect and two-z-rect over a real socket
scripts/gate.sh                         UPDATE  client-core no-sim-core-edge probe
_bmad-output/implementation-artifacts/deferred-work.md                          UPDATE  the 5.2 deferral
_bmad-output/implementation-artifacts/mutations/5-2-one-mirror-two-clients.sh   NEW
```

### Previous story intelligence

- **5.1's terrain and camp are your baseline and the recipe depends on them**: camp at z 9, five
  dwarves clustered, five emitters (ids 5–9). Pin `--z 9` or the capture aims at empty sky.
- **5.1 was signed off on evidence that was partly an artefact of the instrument** — a `--frames 2`
  capture can report `♨=0` on a correct build because dwarves walk over the campfire (emitters draw
  below dwarves, by AC17). Keep captures at `--frames 6` or higher.
- **The mutation runner rewrites source in place and is not concurrency-safe** — run
  `scripts/mutate.sh` alone, never while a gate or a review is running.

### Verification

**Gate (must be green before done):**

```bash
scripts/gate.sh
```

**The instrument recipe.** Run from the repo root:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --bins
./target/debug/simd 7413 &            # port is POSITIONAL; there is no --seed flag
sleep 3
out=$(./target/debug/tui 7413 --frames 6 --z 9 2>/dev/null | sed -e $'s/\x1b\\[[0-9;]*m//g')
for g in '│' '♠' '†' '♨' '☺'; do
  printf '%s = %s\n' "$g" "$(printf '%s' "$out" | grep -o "$g" | wc -l)"
done
```

**Executed at story-creation time against `main`, and what it proved.** This is the "before" half of
AC18, run live rather than described:

```
│ = 6     (tree trunk)
♠ = 48    (tree foliage)
† = 24    (torch)
♨ = 6     (campfire)
☺ = 22    (dwarf)
status:   tick 31  normal  z 9/31  dwarves 5   ... through tick 36
```

**The required observation after adoption: every one of those five counts is non-zero, and `│` and
`♠` are exactly 6 and 48.** The two terrain figures are byte-stable because terrain is seeded and
static; they are the part of this capture that a mirror regression would move.

**CORRECTED AT REVIEW, 2026-08-11 — this instrument is stronger than the paragraph that used to
stand here claimed.** The original text said the three entity counts and the tick are "not stable"
and told the reader not to treat a difference in them as a regression. That is **false for this
recipe**, and following it discards a real regression signal. The Feature Auditor ran the recipe
twice from a fresh daemon and got `†=24 ♨=6 ☺=22`, ticks 31→36, both times — byte-identical to the
`main` baseline recorded above. The tick at connect is a deterministic function of the fixed
`sleep 3` against a seed-deterministic sim, so **a fixed-delay capture IS a byte comparison across
the commit, entity glyphs included.** The 5.1 figures (`†=21 ♨=3 ☺=30`) differ because that capture
used a different sleep and frame count, not because the instrument is nondeterministic.

The honest caveat is narrower: the entity counts and tick are stable **only while the
daemon-start-to-connect delay and the frame count are held fixed**. Vary either — or connect to a
long-lived daemon (one run at tick 963 gave `†=12 ♨=6 ☺=30`) — and only `│=6 ♠=48` survive. Hold
them fixed and all five are a regression guard. **Exit 0 is not a result.** Use `grep -o`, never
`tr -cd`.

**Sabotage:**

```bash
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/5-2-one-mirror-two-clients.sh
```

### If this overruns one session

The split line named in advance by Epic 5 is **the crate/mirror boundary**: Tasks 1–4 and 6
(`client-core` exists, is byte-asserted headless, owns the rect helper, and `simd` validates) versus
Tasks 5, 7 (`tui` adopts it). The first half alone is observable as a passing headless byte-exact
suite. **A split of 5.2 breaches CM2** — it moves wow beat 1 to story 5 of 12 (42%, against the
first-third mandate met at 36%). Epic 5 records that a split of 5.2 or 5.3 is therefore the trigger to
re-check CM2 **on the record**, never a free move: if you take it, say so and re-check.

### References

- Epic 5 and story 5.2 ACs — `_bmad-output/planning-artifacts/epics.md:663-698`; CM2 and the split
  rule at `:613-617`
- AD-13, AD-14, AD-15, AD-17, AD-18, the M2 dependency graph and conventions —
  `architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:62-198`
- NFR8 (gate sibling probes), FR19/FR37 — `epics.md:98, 84`
- Story rules, the instrument rule, "exit 0 is not a result" — `docs/technical-preferences.md:64-101`
- The code being moved — `crates/tui/src/main.rs:462-520`; the render reads it feeds —
  `crates/tui/src/view.rs:127-317`
- The rect path today — `crates/tui/src/view.rs:406-417` → `crates/simd/src/main.rs:151-173` →
  `crates/simd/src/bridge.rs:182-187` → `crates/sim-core/src/lib.rs:1290-1320`
- The log-and-drop convention — `crates/simd/src/main.rs:675-702`
- The gate probe to copy — `scripts/gate.sh:73-88`
- 5.1's camp, `--z 9` decision and instrument traps —
  `_bmad-output/implementation-artifacts/5-1-the-world-grows-things-that-glow.md:237-256, 384-403`

## Dev Agent Record

### Agent Model Used

`gpt-5.6-terra`, reasoning effort `high` (read off the run banner, not the agent's self-report —
the record said "gpt-5.6", and a family nickname is exactly what makes an old ledger row
unreadable). **First delegated dev run on `terra` rather than `sol`** — one variable changed,
effort held at `high`, which is the comparison the runbook asks for.

### Debug Log References

- Task 1 RED (`cargo tree --offline -p client-core` before scaffolding):
  ```text
  error: package ID specification `client-core` did not match any packages
  ```
- Task 2 RED (`cargo test --offline -p client-core` before `Mirror`):
  ```text
  error[E0433]: cannot find type `Mirror` in this scope
  ```
- Task 3 RED (intentional unchanged-entity comparison sabotage):
  ```text
  assertion `left == right` failed
    left: [2]
   right: [7]
  ```
- Task 4 RED (`cargo test --offline -p simd --test serve invalid_rects_are_logged_dropped_and_leave_the_client_connected` before validation):
  ```text
  daemon logged nothing within 10s
  ```
- Task 5 RED (connection-path test before mirror construction existed):
  ```text
  error[E0425]: cannot find function `read_mirror` in this scope
  ```
- Task 6 RED (recorded wire fixture before correcting its deliberately wrong tick expectation):
  ```text
  assertion `left == right` failed
    left: 9
   right: 999
  ```
- Task 7 live instrument, run in the same shell/terminal context. Baseline at `305aa03`:
  ```text
  │ = 6
  ♠ = 48
  † = 24
  ♨ = 6
  ☺ = 22
  tick 36  normal  z 9/31  dwarves 5
  ```
  After mirror adoption:
  ```text
  │ = 6
  ♠ = 48
  † = 24
  ♨ = 6
  ☺ = 22
  tick 36  normal  z 9/31  dwarves 5
  ```
  `NO_COLOR` was set, so the binary correctly emitted its existing warning that this capture cannot evidence colours; glyph-count evidence is unaffected.
- Task 8 mutation RED output, RE-RUN AT REVIEW 2026-08-11 against final source (verbatim). The
  block below it is the original dev-time run, kept for the record — the review found it had been
  captured at `c80e0d6`, before `2925485` changed `client-core/src/lib.rs`, and never re-run, so its
  line numbers no longer matched the source. This run is against HEAD and adds a sixth mutation
  (`rect helper stops normalizing corners`) that did not exist at dev time; it is the proof that the
  restored literal oracle in `view.rs` kills a normalization change, which the self-referential
  version it replaced would have survived.
  ```text
  === delta absence no longer deletes entities ===
  thread 'tests::delta_deletes_entities_and_items_absent_from_authoritative_lists' (447063) panicked at crates/client-core/src/lib.rs:261:9:
  assertion failed: mirror.entities().next().is_none()
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

  === snapshot retains previous-tick state ===
  thread 'tests::changes_partition_entities_and_keep_one_previous_generation' (447470) panicked at crates/client-core/src/lib.rs:386:9:
  assertion failed: mirror.previous_entity(7).is_none()
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

  === unchanged entity is reported as changed ===
  thread 'tests::changes_partition_entities_and_keep_one_previous_generation' (447853) panicked at crates/client-core/src/lib.rs:376:9:
  assertion `left == right` failed
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

  === daemon accepts inverted rectangles ===
  thread 'invalid_rects_are_logged_dropped_and_leave_the_client_connected' (448397) panicked at crates/simd/tests/serve.rs:114:47:
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 60 filtered out; finished in 10.41s

  === rect helper stops normalizing corners ===
  thread 'view::tests::second_enter_commits_each_single_command_mode_and_stays_in_mode' (449282) panicked at crates/tui/src/view.rs:1466:13:
  assertion `left == right` failed
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 44 filtered out; finished in 0.00s

  === mirror entity iteration reverses ids ===
  thread 'recorded_wire_messages_build_the_expected_mirror' (449726) panicked at crates/client-core/tests/mirror.rs:55:5:
  assertion `left == right` failed
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ================ MUTATION RESULTS ================
  delta absence no longer deletes entities                     KILLED
  snapshot retains previous-tick state                         KILLED
  unchanged entity is reported as changed                      KILLED
  daemon accepts inverted rectangles                           KILLED
  rect helper stops normalizing corners                        KILLED
  mirror entity iteration reverses ids                         KILLED

  All mutations killed.
  ```
- Task 8 mutation RED output, ORIGINAL DEV-TIME RUN (superseded, see above):
  ```text
  === delta absence no longer deletes entities ===
  thread 'tests::delta_deletes_entities_and_items_absent_from_authoritative_lists' (272) panicked at crates/client-core/src/lib.rs:261:9:
  assertion failed: mirror.entities().next().is_none()
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

  === snapshot retains previous-tick state ===
  thread 'tests::changes_partition_entities_and_keep_one_previous_generation' (544) panicked at crates/client-core/src/lib.rs:371:9:
  assertion failed: mirror.previous_entity(7).is_none()
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

  === unchanged entity is reported as changed ===
  thread 'tests::changes_partition_entities_and_keep_one_previous_generation' (814) panicked at crates/client-core/src/lib.rs:361:9:
  assertion `left == right` failed
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

  === daemon accepts inverted rectangles ===
  thread 'invalid_rects_are_logged_dropped_and_leave_the_client_connected' (1197) panicked at crates/simd/tests/serve.rs:114:47:
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 60 filtered out; finished in 10.40s

  === mirror entity iteration reverses ids ===
  thread 'recorded_wire_messages_build_the_expected_mirror' (1476) panicked at crates/client-core/tests/mirror.rs:55:5:
  assertion `left == right` failed
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ================ MUTATION RESULTS ================
  delta absence no longer deletes entities                     KILLED
  snapshot retains previous-tick state                         KILLED
  unchanged entity is reported as changed                      KILLED
  daemon accepts inverted rectangles                           KILLED
  mirror entity iteration reverses ids                         KILLED

  All mutations killed.
  ```
- Self-gate pass 1 RED (Codex review):
  ```text
  attempt to multiply with overflow
  ```
  The review identified overflow in malformed snapshot dimension products. The regression test
  `rejects_dimension_products_beyond_u64_without_panicking` first failed with that panic; validation
  now computes in `u128` and returns `MirrorError` instead.

### Completion Notes List

- Task 1: Added the `client-core` workspace crate, `thiserror` workspace dependency, and its inverted no-`sim-core` gate probe. `cargo tree --offline -p client-core` shows normal dependencies `protocol` and `thiserror` only.
- Task 2: Added the validated authoritative `Mirror`; dirty tiles apply in place, all other delta collections replace, and snapshot replacement clears old world state.
- Task 3: Added exactly-one-generation entity history and ascending, exclusive entity change lists. These remain intentionally unwired until Story 5.3.
- Task 4: Added the single-level shared rect helper and daemon log-and-drop validation for inverted or multi-z rectangles; real-socket coverage confirms the client stays connected and valid rectangles still apply.
- Task 5: Retired TUI-local snapshot mutation/validation/indexing. The TUI builds and feeds `Mirror`, preserves its networking bounds, and all 45 view/unit plus 16 binary-capture assertions pass unchanged in expected output.
- Task 6: Added headless wire-literal coverage for movement, deletion-by-absence, item lifecycle, dirty tiles, and snapshot reset.
- Task 7: Ran the prescribed baseline and after-adoption captures; all five reported glyphs are non-zero and terrain is exactly stable (`│=6`, `♠=48`).
- Task 8: Added the 5.3 deferred wiring note and five killed mutations. Full offline workspace tests and clippy passed; `scripts/gate.sh` completed green, including both no-`sim-core` probes.
- Self-gate pass 1: fixed the review's malformed-dimension overflow finding, added a regression test, and reran the full gate green.
- Self-gate pass 2: no actionable findings. Its nested sandbox could not bind loopback sockets for existing `simd`/`tui` tests; the direct workspace gate had already run those tests green.

### File List

- Cargo.toml
- Cargo.lock
- crates/client-core/Cargo.toml
- crates/client-core/src/lib.rs
- crates/client-core/tests/mirror.rs
- crates/simd/src/main.rs
- crates/simd/tests/serve.rs
- crates/tui/Cargo.toml
- crates/tui/src/main.rs
- crates/tui/src/view.rs
- scripts/gate.sh
- _bmad-output/implementation-artifacts/deferred-work.md
- _bmad-output/implementation-artifacts/mutations/5-2-one-mirror-two-clients.sh
- _bmad-output/implementation-artifacts/5-2-one-mirror-two-clients.md

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-10 | Story created. Baseline instrument recipe executed live against `main`; terrain counts pinned (`│=6 ♠=48`) and the entity/tick half recorded as deliberately not byte-stable. |
| 2026-08-10 | Implemented client-core mirror adoption, daemon rect validation, wire-literal tests, mutation evidence, and the 5.3 deferral; status set to review. |
| 2026-08-10 | Self-gate pass 1 fixed overflow-safe snapshot dimension validation and added its regression test. |
| 2026-08-10 | Self-gate pass 2 found no actionable issue; its local gate could not bind loopback sockets in the nested sandbox. |
| 2026-08-11 | Four-layer code review, all layers reporting, no coverage holes, zero HIGH findings. Two decisions taken by Wolf; four patches applied; six items deferred; two dismissed. Gate green, all six mutations killed, live instrument re-run at `│=6 ♠=48 †=24 ♨=6 ☺=22` ticks 31→36. |
