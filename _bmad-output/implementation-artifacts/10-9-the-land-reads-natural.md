---
baseline_commit: 5133a86f683134fb7105a4f276173268d26cb92c
---

# Story 10.9: The Land Reads Natural

Status: in-progress

## Story

As the boss,
I want the ground to read as a snowfield with weather on it rather than as blocky per-voxel noise,
so that the terrain under every look judgement I sign off is terrain someone chose, not a
measurement instrument's leftovers.

## Acceptance Criteria

1. A boot-framing capture is committed under `10-9-signoff/` with each of the four world edges
   (`x=0`, `x=127`, `y=0`, `y=127`) identified in the image, and the story records which screen
   region each occupies. Ridge placement in AC6 targets the two edges this capture shows as far.
2. At `--subdiv 4` on `DEFAULT_SEED`, the `subdiv 4: ... triangles=` line reads **≤ 231,905**
   (25 % of the 927,622 baseline) **and > 60,000** (comfortably above the 31,968 flat floor). Both
   bounds are required: the upper one proves the noise is gone, the lower one proves the surface
   did not become flat.
3. A `gui` test asserts the reported `triangles=` figure differs by more than 10× between a flat
   fine layer and the shipped one, driving the real mesher. This tests the instrument of AC2.
4. In a `sim-core` test on `DEFAULT_SEED`, the mean fraction of a surface cell's four in-bounds
   neighbours sharing its material is **≥ 0.85**. An independent per-column coin flip yields ~0.50.
5. Surface material correlates with local gradient: the snow fraction on columns whose steepest
   in-bounds neighbour differs in height is strictly lower than on columns with no height
   difference, asserted on `DEFAULT_SEED`.
6. Ridges raise `heights` only within a bounded band along the two far edges from AC1. Every
   column outside the ridge and lake footprints has the same height as `height_field` produces
   before the post-pass.
7. `height_field` itself is unchanged: its output for `DEFAULT_SEED` matches values pinned in a
   `sim-core` test, and ridges and the lake are applied only after it returns.
8. `camp_origin(DEFAULT_SEED)` is `Pos { x: 64, y: 64, z: 9 }`, and a test asserts the camp
   position lies outside the lake footprint across at least 50 seeds.
9. A frozen lake exists: a contiguous region of `Material::Ice` surface cells, all at one height,
   of at least 40 cells, placed outside the camp.
10. The biome decision is **consumed, not merely produced**: a `sim-core` test drives the biome
    lookup to return the lake biome at a position that would otherwise be snowfield and asserts
    the emitted surface material changes accordingly, and the converse.
11. `--subdiv 1` still renders: a `gui` test asserts the subdiv-1 path spawns `TerrainTile`
    entities and reports a non-zero derived triangle count, unchanged in shape from today.
12. Wolf signs off on the boot frame at the closing sitting, with the fog and rim held at their
    current values so the ridge lever is judged alone.
13. `scripts/gate.sh` is green at the story tip, and the run is recorded in the Dev Agent Record.

## Tasks / Subtasks

- [x] **Task 1 — Observe the edge mapping before placing anything (AC: 1)**
  - [x] Run `gui --headless --static-world --subdiv 4 --frames 2 --capture <path>` at the boot
        framing; identify the four world edges in the resulting PNG.
        **`--frames 2` does not capture** — the frame is still black and the run dies on
        `capture is black`. Used `--frames 160` (the 10.8 signoff recipe); `triangles=` unaffected.
  - [x] Method that works when the picture alone is ambiguous: rebuild with one edge band raised
        by ~8 levels, capture again, and diff — the changed pixels name that edge. Restore after.
        **The diff method was tried and does not work at this framing** — snowfall and stars are
        animated, so same-build captures differ frame-wide and the bands are not separable
        (15,830 / 24,574 changed px, smeared across all quadrants). `--cursor` was tried too and
        cannot work headless (no `PrimaryWindow` → live pick always `None`). The mapping was taken
        from `CameraRig::project_world_point_with_depth`, the capture's own oracle, verified first
        against the picture: camp `[64,64,9]` → screen `(640.0, 561.2)`, where the campfire is.
        All probe edits reverted; restore confirmed by re-running the control to `triangles=927622`.
  - [x] Record the mapping in the story and commit the capture under `10-9-signoff/`.
  - [x] NOTE: derivation says `x=0` and `y=127` are the FAR pair (upper-left, upper-right) and the
        pinned compass agrees (`north_on_screen` = "down-left", `camera.rs:172`), but a projection
        probe at creation returned ±2 px deltas for 128-cell distances — degenerate without a real
        viewport. Treat the derivation as unconfirmed until this capture confirms it.
        **CONFIRMED.** `x=0` is upper-left (depth 95–141), `y=127` upper-right (depth 86–141), and
        they meet at world `(0,127)` = screen `(683,205)`, the silhouette apex. The creation
        probe's ±2 px does not reproduce — it was reading a default-sized window. **AC1 cannot be
        met as literally written: `x=127` and `y=0` are entirely off-screen at the boot framing
        and partly behind the camera**, so they are identified by projection, not in the image.
        Full table in `10-9-signoff/AC1-edge-mapping.md`.
- [ ] **Task 2 — Replace `detail_depth` with coherent, material-keyed relief (AC: 2, 3)**
  - [ ] Same signature, same call sites: `detail_depth(plane, u, v, subdiv) -> i32` at
        `crates/gui/src/project.rs:1165`. Sample coherent noise in world space over several coarse
        cells rather than hashing each fine voxel independently.
  - [ ] Key amplitude and wavelength to the cell's material — snow long and shallow, rock shorter
        and rougher, ice near-flat. `terrain_material(mirror, position)` already resolves it
        (`terrain_slot_at`, `project.rs:1149`).
  - [ ] Replace the docstring: it currently declares the function a measurement stand-in that 10.4
        will replace. That statement stops being true in this story.
  - [ ] Record `triangles=` and `mesh_build_ms=` before and after in the Dev Agent Record.
- [ ] **Task 3 — Kill the snow/ice coin flip (AC: 4, 5)**
  - [ ] `crates/sim-core/src/worldgen.rs:99` — replace `rng.random::<bool>()` with a rule over
        local gradient and biome. `heights` is already in scope in `layered_terrain`.
  - [ ] Keep `layered_terrain` the last consumer of `STREAM_WORLDGEN`; do not move draws into or
        out of `height_field`.
- [ ] **Task 4 — Biome map and the frozen lake (AC: 9, 10)**
  - [ ] Two or three hardcoded variants and a `match`. **No registry, no trait, no parameter
        table** — the abstraction is not earned yet and the repo policy forbids it.
  - [ ] Blend the INPUT, not the output: threshold one low-frequency field so borders come out
        soft without a border-blending system.
  - [ ] Place the lake outside the camp; carve its basin flat as a post-pass on `heights`.
- [ ] **Task 5 — Scoured ridges on the two far edges (AC: 6, 7, 8)**
  - [ ] Post-pass on `heights` after `height_field` returns, before `clamp_steps`, bounded to a
        band along the AC1 edges.
  - [ ] Re-run `clamp_steps` after the post-pass and assert the ripple stays within the band plus
        the raise amount.
- [ ] **Task 6 — Rebase the material fixtures (AC: 13)**
  - [ ] ~39 assertions across `sim-core/tests/worldgen.rs`, `sim-core/tests/scenario.rs`,
        `tui/tests/client.rs`, `client-core/tests/mirror.rs`, `gui/tests/headless.rs`,
        `gui/tests/bench_contract.rs` reference `Snow`/`Ice`. Most construct tiles rather than
        assert worldgen output — change only those that assert.
- [ ] **Task 7 — Mutation rows** — at minimum: revert `detail_depth` to the hash (AC2 must fail);
      restore the coin flip (AC4 must fail); return a constant biome (AC10 must fail); shift the
      ridge band off the far edges (AC6 must fail).
- [ ] **Task 8 — The sitting (AC: 12)** — present the boot frame with fog and rim unchanged.

## Dev Notes

### Scope guardrails — do NOT

- **Do NOT touch `height_field`'s noise rule** (`worldgen.rs:6-49`). Changing it re-rolls all
  16,384 columns and destroys the composition Wolf approves of. Ridges and the lake are post-passes.
- **Do NOT change the rim dissolve or the fog.** They are 10.8 Ruling 3 defect (d), still deferred,
  and they address the same sharp-edge defect as the ridges. Two levers on one defect must be moved
  one at a time or neither can be judged.
- **Do NOT remove `--subdiv 1`.** Wolf ruled 2026-09-10 that it stays.
- **Do NOT add a cover depth, carve mask, `Tile` variant or `protocol` field.** That is AD-19's
  terrain-state epic, not this story.
- **Do NOT take a new seed.** `DEFAULT_SEED` = 4026891802 stays.

### What already exists

- `detail_depth` (`gui/src/project.rs:1165`) and its consumer `column_heights` (`:772`) — the fine
  layer is entirely these two functions; the mesher below them does not change.
- `terrain_slot_at` / `terrain_material` (`project.rs:1149`) already resolve a cell's material in
  the mesher — material-keyed relief needs no new plumbing.
- The stats instrument is a `println!` at `project.rs:1358` and `:1467`.
- `clamp_steps` (`worldgen.rs:58`) enforces max slope 1 and is already called at the end of
  `height_field`.
- Streams are already split: `STREAM_WORLDGEN`, `STREAM_TREES`, `STREAM_SPAWN`, `STREAM_WANDER`
  (`sim-core/src/lib.rs:27-29,1138-1152`).

### Key decisions & traps

- **`triangles=` discriminates; `faces=` does not.** Measured at creation: forcing `detail_depth`
  to 0 moved faces −33 % but triangles −96.6 %. An AC keyed on faces would pass against unchanged
  noise.
- **The material rewrite has no downstream effect.** `layered_terrain` is the LAST consumer of
  `STREAM_WORLDGEN`; trees, dwarves and wander have their own streams, and `place_ramps` and
  `camp_origin` are pure functions of `heights`. Heights, ramps, camp, trees and dwarves stay
  bit-identical under Task 3 alone.
- **Material has no gameplay meaning.** `is_standable` never reads it; `Material::Snow`/`Ice`
  appear in `sim-core/src` only at `worldgen.rs:100,102`. Task 3 cannot affect pathing.
- **`camp_origin` FINDS a camp, it does not carve one** (`worldgen.rs:140`): it searches for a 7×7
  block of equal heights nearest centre and `.expect`s one exists. The camp sits at exact centre
  (distance 0) on this seed, so nothing displaces it while centre stays flat — but a flat lake near
  centre would capture the camp on a seed where it is off-centre. AC8 asserts this rather than
  assuming it.
- **`clamp_steps` ripples inward by the raise amount.** A ridge of *h* levels reaches *h* cells.
  The far edges are 60+ cells from camp, so it cannot reach — show it, do not claim it.
- **The AC2 bound is a starting bar, not a measurement.** 25 % of baseline is a judgement; Wolf may
  move it at the sitting once the frame is visible.

### Project Structure

| File | NEW/UPDATE |
|---|---|
| `crates/gui/src/project.rs` | UPDATE — `detail_depth`, its docstring |
| `crates/sim-core/src/worldgen.rs` | UPDATE — `layered_terrain` material rule, biome map, ridge and lake post-passes |
| `crates/sim-core/tests/worldgen.rs` | UPDATE — AC4, AC5, AC7, AC8, AC10 |
| `crates/gui/tests/headless.rs` | UPDATE — AC3, AC11 |
| `_bmad-output/implementation-artifacts/10-9-signoff/` | NEW — AC1 capture, AC12 frames |

## Verification

**Instrument (AC2):** the stdout line from the subdivided mesher —
`subdiv 4: projected N terrain cubes at z Z entities=E chunks=C faces=F triangles=T mesh_build_ms=M`
emitted at `crates/gui/src/project.rs:1358`.

**Recipe, EXECUTED at creation on `main` c54b793 (2026-09-10):**

```bash
cargo build -p simd -p gui
./target/debug/simd &                      # prints: listening on 127.0.0.1:7451
./target/debug/gui --headless --static-world --subdiv 4 --frames 2 | grep '^subdiv'
```

Observed, non-zero:
`subdiv 4: projected 45042 terrain cubes at z 31 entities=2164 chunks=118 faces=1155694 triangles=927622 mesh_build_ms=2516`

**The deliberate RED, OBSERVED at creation.** Condition broken: `detail_depth` (`project.rs:1165`)
forced to return `0`, flattening every cell top. Expected and observed output:
`subdiv 4: projected 40148 terrain cubes at z 31 entities=1719 chunks=118 faces=767232 triangles=31968 mesh_build_ms=1743`
— triangles fall 96.6 %. **Restore step:** revert `project.rs`, `cargo build -p gui`, re-run the
recipe and confirm `triangles=927622` returns. That confirmation was run and passed; the mutant
binary does not survive on disk.

**Contrast run**, `--subdiv 1`:
`subdiv 1: projected 40148 terrain cubes at z 31 entities=48293 chunks=0 triangles_derived=498066 mesh_build_ms=25`

**Green gate:** `scripts/gate.sh` (no arguments — the full tier, not the pre-push fast tier).

**Branch:** `10-9-the-land-reads-natural`. Commit author `Völundr <jeicei75@gmail.com>`.
Push and PR only on Wolf's explicit yes.

### References

- `crates/gui/src/project.rs:1165` (`detail_depth`), `:772` (`column_heights`), `:1358` (instrument)
- `crates/sim-core/src/worldgen.rs:6` (`NOISE_SPACING`), `:58` (`clamp_steps`), `:86`
  (`layered_terrain`), `:99` (the coin flip), `:140` (`camp_origin`)
- `crates/sim-core/src/lib.rs:27-29` (streams), `:1130-1152` (`generate` sequence)
- `crates/gui/src/camera.rs:118-172` (boot yaw, `north_on_screen`, the pinned bearing)
- `docs/architecture.md` § Terrain representation (AD-19)
- `docs/tech-art-guidelines.md:248-262` (fog vs rim; `RIM_WIDTH`, `RIM_LEVELS`)
- `_bmad-output/implementation-artifacts/10-8-...md` § Ruling 3 (defect (d), deferred)
- `_bmad-output/planning-artifacts/epics.md` § Story 10.9

## Dev Agent Record

### Agent Model Used

- Orchestration + verification: Claude Opus 5 (1M context).
- Implementation (Tasks 2-7): Codex `gpt-5.6-terra`, reasoning effort high, via
  `scripts/codex-handoff.sh`.

### Debug Log References

**Control on the clean tree, before any edit** (`5133a86`, `--subdiv 4 --frames 2`):
`subdiv 4: projected 45042 terrain cubes at z 31 entities=2164 chunks=118 faces=1155694
triangles=927622 mesh_build_ms=2527` — reproduces the story's creation baseline exactly.

**Task 1 probes (all reverted, nothing committed while sabotaged).** Three instruments were tried
before one worked; the two that failed are recorded because each fails silently in a way that
would have been believed:
1. `--cursor` headless → `pick: cursor=(640,700) no tile picked`, four screen points, all None.
2. Raise-a-band pixel diff → no noise floor at this framing (animated snowfall/stars).
3. `CameraRig::project_world_point_with_depth` → deterministic, and cross-checked against the
   committed frame before use.

**Restore verified:** `git status` clean on `crates/`, rebuild with no warnings, control re-run to
`triangles=927622` — the probe binary does not survive on disk.

### Completion Notes List

- **Task 1 (AC1) complete.** Edge mapping taken and committed
  (`10-9-signoff/AC1-edge-mapping.md`, `boot-baseline-5133a86-subdiv4.png`). Ridges target `x=0`
  and `y=127`, confirming the story's derivation.
- **AC1 defect on the record:** only two of the four world edges are in frame at the boot framing.

### File List

- `_bmad-output/implementation-artifacts/10-9-signoff/AC1-edge-mapping.md` — NEW
- `_bmad-output/implementation-artifacts/10-9-signoff/boot-baseline-5133a86-subdiv4.png` — NEW

## Change Log

| Date | Change |
|---|---|
| 2026-09-10 | Task 1 (AC1): edge mapping measured and committed. `x=0` far upper-left, `y=127` far upper-right, meeting at screen `(683,205)`; `x=127` and `y=0` are off-screen at this framing, so AC1 cannot be met as literally written. Two instruments the story proposed were falsified first (`--cursor` is dead headless; the pixel diff has no noise floor under animated snowfall). |
| 2026-09-10 | Story created. Scope settled with Wolf in conversation; baseline, deliberate RED and restore confirmation executed at creation on `main` c54b793. |
