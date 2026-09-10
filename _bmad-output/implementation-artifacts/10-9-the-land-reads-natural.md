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
- [x] **Task 2 — Replace `detail_depth` with coherent, material-keyed relief (AC: 2, 3)**
  - [x] Same signature, same call sites: `detail_depth(plane, u, v, subdiv) -> i32` at
        `crates/gui/src/project.rs:1165`. Sample coherent noise in world space over several coarse
        cells rather than hashing each fine voxel independently.
  - [x] Key amplitude and wavelength to the cell's material — snow long and shallow, rock shorter
        and rougher, ice near-flat. `terrain_material(mirror, position)` already resolves it
        (`terrain_slot_at`, `project.rs:1149`).
  - [x] Replace the docstring: it currently declares the function a measurement stand-in that 10.4
        will replace. That statement stops being true in this story.
  - [x] Record `triangles=` and `mesh_build_ms=` before and after in the Dev Agent Record.
- [x] **Task 3 — Kill the snow/ice coin flip (AC: 4, 5)**
  - [x] `crates/sim-core/src/worldgen.rs:99` — replace `rng.random::<bool>()` with a rule over
        local gradient and biome. `heights` is already in scope in `layered_terrain`.
  - [x] Keep `layered_terrain` the last consumer of `STREAM_WORLDGEN`; do not move draws into or
        out of `height_field`.
- [x] **Task 4 — Biome map and the frozen lake (AC: 9, 10)**
  - [x] Two or three hardcoded variants and a `match`. **No registry, no trait, no parameter
        table** — the abstraction is not earned yet and the repo policy forbids it.
  - [x] Blend the INPUT, not the output: threshold one low-frequency field so borders come out
        soft without a border-blending system.
  - [x] Place the lake outside the camp; carve its basin flat as a post-pass on `heights`.
- [x] **Task 5 — Scoured ridges on the two far edges (AC: 6, 7, 8)**
  - [x] Post-pass on `heights` after `height_field` returns, before `clamp_steps`, bounded to a
        band along the AC1 edges.
  - [x] Re-run `clamp_steps` after the post-pass and assert the ripple stays within the band plus
        the raise amount.
- [x] **Task 6 — Rebase the material fixtures (AC: 13)**
  - [x] ~39 assertions across `sim-core/tests/worldgen.rs`, `sim-core/tests/scenario.rs`,
        `tui/tests/client.rs`, `client-core/tests/mirror.rs`, `gui/tests/headless.rs`,
        `gui/tests/bench_contract.rs` reference `Snow`/`Ice`. Most construct tiles rather than
        assert worldgen output — change only those that assert.
- [x] **Task 7 — Mutation rows** — at minimum: revert `detail_depth` to the hash (AC2 must fail);
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

**Task 5 RED then green.** Before the ridge post-pass existed, the new unit tests did not compile:
`cannot find function apply_ridges in this scope` and `cannot find function
in_ridge_footprint in this scope`. With the minimal post-pass in place, all three passed. The
default `height_field` samples are independently pinned at `(0,0)=15`, `(64,64)=8`,
`(127,127)=5`, and `(20,92)=18` before either post-pass. Ridges raise only the confirmed far
edges `x < 6` / `y >= 122` by four levels; the direct test permits the resulting clamp ripple only
through `x < 10` / `y >= 118` (plus the lake core and its four-cell stepped shore), and found no
outside change. The camp is `Pos { x: 64, y: 64, z: 9 }` on `DEFAULT_SEED`, and stayed outside the
lake biome for `DEFAULT_SEED..DEFAULT_SEED + 51`.

**Task 6 RED then green.** The first full `sim-core` run correctly failed its world fingerprint:
`left: 628768409094364363`, `right: 2166155576459420686`. The real-world bench control then
failed with `exposed_faces=61152 (expected 61142), greedy_quads=12132 (expected 19264)`. Rebased
only those terrain-derived literals: fingerprint `0x08b9_d589_660e_9ccb`, control 61,152 faces /
12,132 greedy quads / 24,264 triangles, and its per-class census. `cargo test -p sim-core --test
worldgen` (16 tests) and `python3 -m unittest discover -s scripts/tests` (68 tests) then passed.

**Task 5 AC2 re-measurement (after rebuilding following mutation):**
`subdiv 4: projected 43458 terrain cubes at z 31 entities=1719 chunks=118 faces=802668
triangles=94208 mesh_build_ms=2600`. Against the `5133a86` baseline (`triangles=927622`,
`mesh_build_ms=2527`), 94,208 is `<= 231,905` and `> 60,000`; AC2 remains met. `faces=` is
recorded only as diagnostic data, not an AC result.

**Task 7 mutation evidence.** `scripts/mutate.sh
_bmad-output/implementation-artifacts/mutations/10-9-the-land-reads-natural.sh` executed every
row serially. KILLED: per-voxel relief hash (the AC2 budget test at `project.rs:2802`), snow/ice
coin flip (`surface_materials_are_coherent_and_snow_prefers_flat_ground`), constant lake biome
(`biome_decision_is_consumed_by_the_surface_material_rule`), shifted ridge band (outside-footprint
failure at `(10,0)`), `NOISE_SPACING` 32→31 (the height-field pin), lake footprint moved over camp
(`assertion left != right failed`), and the stale real-world quad control. The initial lake-basin
row SURVIVED because the old rectangle test allowed the unflattened smooth field. It was
strengthened with `lake_post_pass_flattens_its_entire_ice_core`, committed, and re-run alone via
`scripts/mutate.sh`: KILLED with `lake core heights were {17, 18}`. The source was restored and
`cargo build --offline -p simd -p gui` completed before the AC2 measurement. The mutation audit
also exposed a stale 10.6 control row; it was re-pointed before mutation and now audits cleanly
(527 rows).

**Finishing verification (Tasks 3, 4, and 6).** A fresh live-daemon export after the lake-tree
exclusion measured `exposed_faces=62,586`, `greedy_quads=12,322`, `triangles=24,644`, `chunks=128`,
and `cells=45,920`; its independent census measured `tree_cells=4,930`, `tree_faces=13,339`,
`terrain_cells=40,990`, `terrain_faces=49,247`, and `trees=259`. These replace the stale Task 6
control. The control comment names 10.9's terrain changes plus lake tree exclusion and records
the +2.35% faces / +1.57% quads movement. The material-parity fixture was first RED (`ice` at
`k=4`: 80 faces, expected 96), then fixed and its mutation was KILLED. The lake-tree test was RED
at lake column `(26,88)` before the exclusion; its mutation was KILLED after restoration. The
biome-emission-path test was added after review pass 3; forcing every biome to lake produced
`left: Solid(Ice)`, `right: Solid(Snow)` at `(64,64)`, and the mutation was KILLED.

**Final gate and self-review.** `scripts/gate.sh` with no arguments completed **GREEN in 480s**
after the final review fix (Cargo 66s, pixel guards 373s, bench 18s, mutation audit 3s). Three
`codex review --base main` passes ran, the permitted maximum: pass 1 found benchmark material
parity (fixed in `7e3ef20`); pass 2 found lake tree trunks (fixed in `a0c612c`); pass 3 found the
biome test bypassing the production emission path (fixed in `81430cf`). The reviewer sandbox could
not bind local sockets (`Operation not permitted`), so its socket-test failures were environmental;
the full gate ran those checks green. No fourth review was run.

### Completion Notes List

- **Task 1 (AC1) complete.** Edge mapping taken and committed
  (`10-9-signoff/AC1-edge-mapping.md`, `boot-baseline-5133a86-subdiv4.png`). Ridges target `x=0`
  and `y=127`, confirming the story's derivation.
- **AC1 defect on the record:** only two of the four world edges are in frame at the boot framing.
- **Task 2 (AC2, AC3) complete — `d8cf573`.** `detail_depth` now interpolates a noise field whose
  corners span three coarse cells, keyed by material (snow long and shallow, rock a shorter
  wavelength, ice flat). Fast gate green at the commit.
- **AC2 MEASURED AND MET**, on the real recipe with the daemon running:

  | | baseline `5133a86` | after `d8cf573` | AC2 bound |
  |---|---|---|---|
  | `triangles=` | 927,622 | **95,422** | `<= 231,905` and `> 60,000` — both met |
  | `faces=` | 1,155,694 | 832,224 | (not an AC — does not discriminate) |
  | `mesh_build_ms=` | 2,516 | 1,768 | — |
  | entities | 2,164 | 2,098 | — |

  89.7 % below baseline and 3.0x above the 31,968 flat floor. Frame committed as
  `10-9-signoff/task2-relief-d8cf573.png` (`--frames 160`; near-white 0.3908 %, exit 0).
- **The bench seam was nearly lost and is not.** The staged Task 2 deleted
  `the_detail_rule_matches_the_benchs_pinned_vector`, the test holding
  `crates/gui/src/project.rs` and `scripts/bench/resolution_bench.py` to ONE rule. Python still
  pinned the OLD vector against itself, so it stayed green while the two implementations
  diverged. `scripts/audit-mutations.py` caught it in the gate (row in
  `mutations/10-6-how-fine-can-we-go.sh` naming a test that no longer existed). The new rule is
  now ported to the bench, both sides re-pinned to the same new vector, and the row re-pointed.
- **Task 5 (AC6, AC7, AC8) complete — `27965eb`.** A six-cell, four-level post-pass raises only
  `x=0` and `y=127` edge bands, then re-runs `clamp_steps`. Unit pins show no height change beyond
  the band plus its four-cell ripple and the lake footprint; they also pin `height_field` before
  post-passes and camp safety across 51 seeds.
- **Tasks 3–4 (AC4, AC5, AC9, AC10) complete.** The gradient/biome material rule replaces the
  coin flip; the lake is a hardcoded biome with a flat post-pass. The final review strengthened
  the AC10 test to observe tiles emitted through `layered_terrain`, not a direct helper call.
- **Task 6 (AC13 fixture work) complete — `7dc83f8`, `c3ee6a6`, `a0c612c`.** The final live-world
  control is 62,586 exposed faces, 12,322 greedy quads, and 24,644 triangles; its census is 4,930
  tree cells / 13,339 tree faces / 40,990 terrain cells / 49,247 terrain faces / 259 trees.
- **Task 7 complete — `7e9fadd`, `db0186c`, `032f557`.** Ten story mutation rows executed;
  seven killed on the first run. The lake row initially survived, so its test was strengthened in
  `3a58eb4` and the row re-targeted in `032f557`; the re-run killed it. The 10.6 control row was
  re-pointed because Task 6 changed its measured literal.
- **AC12 remains unmet by design.** Task 8 is Wolf’s in-person sign-off and was not attempted.

### File List

- `_bmad-output/implementation-artifacts/10-9-signoff/AC1-edge-mapping.md` — NEW
- `_bmad-output/implementation-artifacts/10-9-signoff/boot-baseline-5133a86-subdiv4.png` — NEW
- `_bmad-output/implementation-artifacts/10-9-signoff/task2-relief-d8cf573.png` — NEW
- `crates/gui/src/project.rs` — UPDATE (Task 2)
- `scripts/bench/resolution_bench.py` — UPDATE (Tasks 2 and 6; material parity and final live control)
- `scripts/tests/test_resolution_bench.py` — UPDATE (Tasks 2 and 6; independent parity and control pins)
- `_bmad-output/implementation-artifacts/mutations/10-6-how-fine-can-we-go.sh` — UPDATE (row re-pointed)
- `crates/sim-core/src/lib.rs` — UPDATE (Task 5 ridge post-pass sequence)
- `crates/sim-core/src/worldgen.rs` — UPDATE (Tasks 3–5 and 7; ridge, camp, lake, biome, and tree exclusion)
- `crates/sim-core/tests/worldgen.rs` — UPDATE (Task 6 terrain fingerprint)
- `crates/gui/src/project.rs` — UPDATE (Task 7 AC2 upper-budget assertion)
- `scripts/bench/resolution_bench.py` — UPDATE (Task 6 final real-world control)
- `scripts/tests/test_resolution_bench.py` — UPDATE (Task 6 final control/census literals)
- `_bmad-output/implementation-artifacts/mutations/10-9-the-land-reads-natural.sh` — NEW (Task 7)

## Change Log

| Date | Change |
|---|---|
| 2026-09-10 | Finishing verification: rebased the live export control to 62,586 faces / 12,322 quads / 24,644 triangles with its remeasured census and reason comment. Three self-review findings were fixed (benchmark material parity, lake tree exclusion, and biome emission-path coverage); the final full gate is green in 480s. |
| 2026-09-10 | Task 2 (AC2, AC3): coherent material-keyed relief replaces the per-voxel hash placeholder. `triangles=` 927,622 -> **95,422**, inside both AC2 bounds; `mesh_build_ms=` 2,516 -> 1,768. The Rust/Python rule pin was nearly lost in the rewrite and was restored — the gate's mutation audit caught it. |
| 2026-09-10 | Tasks 5–7: scoured ridges now occupy the confirmed far `x=0` / `y=127` edges; direct pins prove height-field preservation, bounded clamp ripple, lake flatness, and camp separation. Terrain-derived controls were rebased, every new/changed terrain test received executed mutation evidence, and AC2 re-measured at **94,208** triangles / 2,600 ms. Task 8/AC12 remains a human sitting and is intentionally uncompleted. |
| 2026-09-10 | Task 1 (AC1): edge mapping measured and committed. `x=0` far upper-left, `y=127` far upper-right, meeting at screen `(683,205)`; `x=127` and `y=0` are off-screen at this framing, so AC1 cannot be met as literally written. Two instruments the story proposed were falsified first (`--cursor` is dead headless; the pixel diff has no noise floor under animated snowfall). |
| 2026-09-10 | Story created. Scope settled with Wolf in conversation; baseline, deliberate RED and restore confirmation executed at creation on `main` c54b793. |
