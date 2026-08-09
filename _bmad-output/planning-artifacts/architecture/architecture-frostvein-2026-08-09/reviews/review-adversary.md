# Adversarial Review — frostvein M2 Architecture Spine

- **Artifact:** `architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md` (draft)
- **Parent (binding):** `architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` (AD-1..AD-12 + conventions)
- **Method:** for each attack, construct two M2 stories/crates that each obey every AD
  to the letter yet build incompatibly. Every successful pair = a hole to close.
- **Grounding:** attacks were checked against the real `crates/protocol/src/lib.rs`
  (wire `Entity` is `{id, kind, pos, state: JobState}`; `Rect` is `{min: [i32;3],
  max: [i32;3]}` — structurally z-spanning), the PRD (FR27–FR37), and the addendum.
- **Reviewer:** adversarial pass, 2026-08-09.

## Verdict

**The spine is not story-proof as drafted.** Five constructed pairs pass every AD
and still collide; two of them (A1, A2) sit directly on the milestone's critical
path and one (A1) is an outright internal contradiction between AD-16 and the
Structural Seed. All are closable with one new AD and a handful of tightened
sentences — none requires rethinking the paradigm. Close A1–A3 before the first
story is cut; A4–A5 before their owning stories.

---

## A1 — CRITICAL — AD-16 contradicts "no shape changes": the `light` field cannot land

**The contradiction.** AD-16: "Every light source is an entity carrying a `light`
field naming a kind." The Structural Seed and FR30 (quoted into the spine):
`protocol/ # + M2: vocabulary growth only (FR30) — no shape changes`. But the wire
`Entity` struct is today `{id: u32, kind: EntityKind, pos: [i32;3], state: JobState}`
(`protocol/src/lib.rs:90`). Adding `light: Option<LightKind>` to `Entity` **is a
shape change** — a new field on an existing struct, pinned by the crate's own
wire-literal tests. AD-16 and the seed cannot both be obeyed.

**The pair.**
- *Story A (sim/worldgen, FR28–FR29)* reads AD-16 as controlling: torch, campfire,
  and lantern-carrying dwarves are entities with `light: Some(kind)`. It needs the
  `protocol::Entity` field to exist and writes scenario tests against it.
- *Story B (protocol, FR30)* reads "vocabulary growth only — no shape changes" as
  controlling: it grows `EntityKind` with `Torch` and `Campfire` variants and grows
  `Material` — pure vocabulary — and encodes "dwarves glow" nowhere, expecting
  clients to infer lantern-light from `kind == Dwarf`. That inference is a client
  inventing world state (NFR5) — but Story B never violates a letter; it followed
  the seed's own words.

Result: either two wire truths (a `light` field vs. kind-implies-light), or a
protocol story that stalls waiting for a ruling. Both stories cite the spine.

**Secondary holes inside the same AD, exposed once you try to build Story A:**
1. **`state: JobState` on a torch.** Wire `Entity` requires a `JobState`; a torch
   is not idle, walking, or working. Either `state` becomes `Option` (another shape
   change, and a TUI change), torches carry a permanent lie (`state: idle`), or
   light emitters aren't wire `Entity` at all — three implementations, all
   AD-16-compliant, mutually incompatible.
2. **Vocabulary overlap.** Is "torch" an `EntityKind`, a `LightKind`, or both? One
   story writes `EntityKind::{Torch, Campfire}` + `LightKind::{Torch, Campfire,
   Lantern}` (torch-ness owned twice); another writes `EntityKind::LightEmitter`
   with `light` carrying the distinction. Both satisfy "entity with a light field
   naming a kind."
3. **Id economy.** Torches/campfire as entities draw from the AD-9 global allocator
   and are full-resent in every delta (AD-8 small-state rule) — fine, but only if
   both stories agree they're *entities* and not a fourth full-resend list
   (`lights: Vec<...>` — which FR28's "world state with a position" invites, and
   which would be a shape change too, though a story could argue a *new* message
   section is "growth").

**Close with (tighten AD-16 + fix the seed):**
- Strike "no shape changes" from the seed line; replace with: "additive shape
  growth for AD-16 only: `Entity` gains `light: Option<LightKind>`; no other shape
  changes." Name the sanctioned diff exactly.
- Pin the vocabulary: `EntityKind` grows `Torch`, `Campfire`; `LightKind` is a new
  enum `{Torch, Campfire, Lantern}` in `sim-core`, mirrored in `protocol` (AD-6).
  There is **no** separate light-emitter list; AD-16's "entity" means the wire
  `Entity` struct, said in those words.
- Rule on `state`: decide it (e.g. "non-dwarf entities send `state: idle`; the
  field does not become optional in M2, with a `// NOTE:` naming the wart" — or
  make it `Option`, but *decide*, because both clients and the TUI parity story
  build against the answer).

---

## A2 — HIGH — Nobody owns the mirror's shape: the tui-adoption story and the gui story each build a different `client-core`

**The hole.** AD-13 creates `client-core` and says both clients consume it — but no
AD pins (a) which story *creates* the crate and its read API, (b) what that API is,
or (c) AD-15's real force. AD-15's sentence — "the mirror holds **only** states the
wire delivered (current tick and the previous one)" — parses two ways: as a **cap**
("at most those; storing one is fine") or a **mandate** ("exactly two, always").

**The pair.**
- *Story A (tui adopts client-core)* lands first, creates the crate, and builds the
  mirror the TUI needs: single current tick, flat `Vec<Tile>`, plain sorted
  `Vec<Entity>` — reading AD-15's "only" as a cap. Its tests pass; the TUI's
  in-crate state is retired. AD-13 satisfied to the letter.
- *Story B (gui projection/interpolation)* arrives needing: current **and**
  previous tick (AD-15 as mandate), entities keyed by `Id` for AD-14
  reconciliation, and a per-tick change set ("which ids moved"). It either rewrites
  Story A's finished, tested crate — or computes prev-vs-current diffs inside
  `gui`, which is letter-legal (diffing for presentation is arguably not
  "snapshot/delta application", the only thing AD-13 reserves) and re-creates
  exactly the second implementation of client-side semantics AD-13 exists to
  prevent.

**Sub-holes exposed by the same attack:**
1. **Scope of "the previous tick."** Read literally, the mirror holds two *world
   states* — including two copies of the 128×128×32 tile grid, cloned at 10 tps.
   One story clones everything (correct by the letter, absurd); another keeps
   previous *entity states* only (what interpolation actually needs). Clashing
   shapes of the same concept.
2. **Blending across a reset.** After `load`, `previous` is old-world state and
   `current` is new-world state — both "states the wire delivered", so blending
   dwarves flying across the map to their post-load positions is AD-15-compliant.
   AD-11's "snapshot is an authoritative full reset" governs the mirror's *data*,
   not the projection's *tween*.

**Close with (new AD — "the mirror's contract"):**
- `client-core` is created by its **own story, before either consumer**, and that
  story owns the read API. Name the API's shape at spine level: entities exposed
  as an `Id`-keyed map; tiles as the flat grid (parent bulk-array convention);
  the mirror retains **current tick plus previous entity states only** — tiles,
  designations, zones, items, speed are current-only.
- AD-15 tightened: the two-state retention is a *mandate* on `client-core`, not a
  cap clients may under-fill; `gui` and `tui` never compute their own
  cross-tick diffs — if a projection needs change information, `client-core`
  grows the query.
- One sentence: applying a `snapshot` clears the previous state; the projection
  never blends across a snapshot boundary.

---

## A3 — HIGH — AD-14's "only place render entities are created" is unsatisfiable the moment the aurora exists

**The hole.** AD-14: Bevy reconciliation systems "are the only place render
entities are created or despawned", and "deleting every render entity and
re-projecting must reproduce the same scene." But M2's sanctioned client-side
atmosphere (AD-15/NFR5 carve-out: sky, aurora, snowfall, flicker) and the NFR6
frame-time overlay are *also* Bevy entities — with **no mirror counterpart**.

**The pair.**
- *Story A (reconciliation)* obeys AD-14 to the letter and enforces it with the
  AD's own test: despawn everything, re-project from the mirror, assert the scene
  is identical. Green.
- *Story B (atmosphere: sky/aurora/snowfall)* spawns particle and sky entities at
  startup from no mirror data — sanctioned explicitly by AD-15's carve-out. Now
  either Story B violates AD-14's letter ("only place entities are created"), or
  Story A's delete-all-re-project invariant is false (the aurora doesn't come
  back), or Story B smuggles atmosphere *into the mirror* so it survives
  re-projection — client state inside the crate whose contents are defined as
  "wire truth only". Every resolution breaks an AD some other story relies on.

The camera entity and any UI/overlay entities fall in the same crack.

**Close with (tighten AD-14):** scope it to *world-projected* entities: every
render entity is tagged either `WorldProjected` (created/despawned only by
reconciliation, keyed by sim `Id`, delete-and-re-project must reproduce it) or
`ClientLocal` (camera, overlay, atmosphere — never keyed by sim `Id`, never
touched by reconciliation, capped to the NFR5 carve-out list). The
delete-and-re-project test quantifies over `WorldProjected` only.

---

## A4 — MEDIUM-HIGH — 3D picking can emit rect commands the sim never defined: z-spanning, unnormalized, and validated by nobody

**The hole.** The parent convention says rects are "inclusive of both corners
(`min ≤ max` per axis) on a single z-level" — but that is a *convention row*, not
an AD, written when the only client was a 2D TUI whose cursor cannot leave a
z-level. `protocol::Rect` is `{min: [i32;3], max: [i32;3]}`: a multi-z or
min>max rect **parses as a perfectly well-formed `Command`**. The parent's
malformed-input rule ("simd logs and drops the line") never fires — the line
isn't malformed.

**The pair.**
- *Story A (gui picking, FR36)* implements a 3D drag: the ray-picked start tile is
  on z=8 (a ramp top), the release tile on z=7, or the drag goes bottom-right to
  top-left. It sends the drag's raw extent: `{min:[9,4,8], max:[3,7,7]}`. AD-10 is
  obeyed (existing command, via the queue), AD-4 obeyed (no game logic), the crate
  graph obeyed. Nothing in any AD told this story the convention exists — the M2
  spine's F12 row cites only AD-10 and AD-14.
- *Story B (simd, unchanged per the seed: "unchanged structurally")* iterates the
  rect as ever. Depending on how M1 wrote the loop, a min>max rect silently
  designates nothing, or a z-range designates a cube the TUI cannot even express —
  divergent, per-client command semantics, the exact desync AD-6 was built against,
  arriving through *values* instead of *shapes*.

A second-order version of the same hole: rect **normalization** (sorting corners,
clamping to the slice's z) has no assigned home. If `gui` and `tui` each hand-roll
it, the two clients drift; `client-core` is allowed to be command-blind today.

**Close with (promote + assign):** promote the rect rule from convention to a
binding sentence in the M2 spine (it now has two independent producers): rect
commands are single-z, corner-normalized, `min ≤ max` per axis. Assign the
normalization one home — `client-core` gains the one pure helper
(`Rect::normalized(a, b, z)`-shaped) both clients call — and require `simd` to
**validate and drop** (log, per the malformed-input convention) any rect command
violating the rule, so a future client bug cannot mutate the world in undefined
ways. That is a one-line `simd` change; "unchanged structurally" survives.

---

## A5 — MEDIUM — AD-17 rung 3's self-tests have no venue: written as `cargo test`, they put a GPU inside the gate

**The hole.** Rung 2 is explicit: headless, minimal plugins, no GPU in CI. Rung 3
gives `--capture` "its own tests (file exists, not black, changes when the world
changes...)" — but never says **where they run**. Capturing via the Bevy
screenshot API requires the real render path (window/surface — on this box, WSLg's
unproven Vulkan/Dozen route the spine itself flags).

**The pair.**
- *Story A (capture instrument)* writes those self-tests the way every other test
  in this repo is written: `#[test]`s that spawn the binary with `--capture` —
  they need a live render surface. On the dev machine they pass; `scripts/gate.sh`
  now hangs or fails on the second devpod / any display-less context, and the
  gate's cache-repair contract inherits a GPU dependency.
- *Story B (NFR8 gate story)* extends `gate.sh` assuming the gate's existing
  property: everything in it runs headless (`cargo test`, probes). Both stories
  are AD-17-compliant; the gate is now flaky in exactly the way AD-17's
  "no golden-image CI" clause was written to prevent — the flake just moved from
  image comparison to surface acquisition.

**Close with (one sentence in AD-17):** rung-3 self-tests are an ignored/feature-
gated test target (or a script) invoked explicitly on the dev machine as part of
the sign-off ritual — **never** by `scripts/gate.sh` or default `cargo test`. The
gate stays display-free by rule, not by luck.

---

## A6 — MEDIUM — "Trees are Material variants" under-pins the tree: variant set, diggability, drops, and snow-cap ownership are all up for grabs

**The holes** (all inside AD-16's first sentence; exhaustive-`match` bridges catch
vocabulary *drift* at compile time, but only after someone has already chosen the
vocabulary — and these choices cross story boundaries):

1. **Variant set / composition.** One `Material::Wood` column? `{Wood, Foliage}`
   trunk-plus-canopy? A worldgen story picks one; the gui data-table story and the
   TUI glyph story (FR27: "the TUI shows them as glyphs") have each sketched
   against a guess. Compile errors force sync *late*, at integration, as rework.
2. **Dig semantics.** Trees "block pathing via existing solidity rules, mutate via
   `set_tile`" — so a tree is designatable for digging. The dig system spawns the
   only item that exists: **digging a pine drops a stone** (`Item` has no `kind`;
   `protocol/src/lib.rs:119`'s own NOTE). One story ships that absurdity silently;
   another "fixes" it by adding an item kind — an unsanctioned wire-shape change
   (see A1). Neither is wrong by the letter.
3. **Snow-capped trees.** The PRD's bar: "loaded branches", snow as "a settled
   cap". Worldgen could bake `SnowyFoliage` materials (sim truth, on the wire);
   the gui could paint snow caps procedurally (NFR5 atmosphere carve-out). Two
   owners of the same pixel — and if *both* stories do it, trees are double-capped
   and the TUI/gui disagree about world state that is nominally shared.

**Close with (tighten AD-16, three sentences):** name the variant set at spine
level (e.g. trunk + foliage, two `Material` variants, both `Solid`, both diggable);
rule that M2 digging a tree voxel behaves exactly like stone including the drop,
with a mandated `// NOTE:` naming the wart (no item-kind work in M2); assign
snow-on-branches to exactly one side (worldgen-baked material variants **or**
client cosmetic — pick one, write it down).

---

## Probed and held (no pair constructible — for completeness)

- **Crate graph:** the "no edge may be added" rule plus the two new gate probes
  (`gui`, `client-core` vs `sim-core`) close the graph tightly; I could not
  construct two stories adding different edges without one violating the mermaid
  rule outright. The one soft spot — `client-core` growing a math/util dependency
  for blend helpers — is already covered by the parent's closed-dependency-list
  rule.
- **Two Bevy versions:** the "move together, same 0.x line" convention closes the
  workspace-version attack.
- **Flicker determinism:** flicker is client-side by AD-16 (wire never carries
  it), and NFR5's carve-out names it; both clients flickering differently is
  sanctioned, not a hole.
- **Snapshot-as-reset in the render world:** AD-14's reconciliation keyed by
  never-reused AD-9 ids handles the post-`load` world correctly *once A3's
  tagging fix is in* — without it, the reset test is the thing that breaks.
- **Command acknowledgement:** the no-ack convention (effect appears in next
  delta) transfers to `gui` unchanged; no second mechanism is invitable.

## Disposition summary

| # | Severity | Hole | Fix shape |
| --- | --- | --- | --- |
| A1 | CRITICAL | AD-16 vs seed's "no shape changes"; torch `JobState`; light vocabulary ownership | Name the exact sanctioned `Entity` diff; pin `LightKind`/`EntityKind`; rule on `state` |
| A2 | HIGH | Mirror API has no owner story; AD-15 cap-vs-mandate; "previous tick" scope; blend across load | New AD: client-core's contract — own story first, Id-keyed reads, previous = entities only, mandate not cap, snapshot clears previous |
| A3 | HIGH | AD-14's "only place" + delete-and-re-project vs atmosphere/overlay entities | Scope AD-14 to `WorldProjected`-tagged entities; `ClientLocal` class for the carve-out |
| A4 | MED-HIGH | z-spanning/unnormalized rects parse as valid commands; normalization homeless; simd doesn't validate | Promote rect rule to binding; helper in `client-core`; simd validates-and-drops |
| A5 | MEDIUM | Rung-3 capture self-tests' venue unpinned → GPU inside the gate | One sentence: capture tests never run under gate.sh/default `cargo test` |
| A6 | MEDIUM | Tree variant set, dig drop absurdity, snow-cap double ownership | Pin variants; stone-drop + `// NOTE:`; assign snow caps to one side |
