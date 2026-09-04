---
baseline_commit: 834f105
model: claude-opus-5[1m]  # Opus default; the 1M-context variant, recorded so the ledger row is readable
---

# Story 10.5: Dwarves Worth Looking At — Part A, the seam

Status: ready-for-dev

**THE EPIC'S OWN SPLIT LINE IS TAKEN, and this story is the first half.** `epics.md:1591-1593`
names it: *"if this overruns a dev session, 'feature enablement + a stand-in glTF rendering on the
seam' splits from 'the authored dwarf itself' — the seam story first."* It overruns. The evidence is
in the premises below: the entity path needs an origin change, a scale change pinned by a
source-text grep across two languages, a partition invariant that scene children break, and a
hot-reload question nobody has ruled on — before any authored geometry exists.

**Part B — the authored dwarf, generalising `check_asset.py`, and UX-DR22's two halves — is a
separate story, and it cannot start until Wolf has a model.** That is not a scheduling preference:
UX-DR22's opening half requires an approved artifact of the real asset before implementation, so
Part B's first input is Wolf's `.blend`, not a dev agent's.

## Story

As the boss,
I want the dwarf's rendering seam to draw an authored glTF model instead of a coloured cube,
so that when I hand over a dwarf I have made, the client already knows how to stand it in the world.

## Premises re-verified against source — 2026-09-04, on `834f105`

**Six of nine were wrong, stale, or absent from the epic.** Read this before the Acceptance Criteria.

1. **`bevy_gltf` IS ALREADY ENABLED — half of the epic's AC1 is a no-op.** `epics.md:1380-1382` and
   `:1571-1574` say the trim carries "neither `bevy_gltf` nor `file_watcher`". `Cargo.toml:21-24`
   enables `3d_bevy_render`, which pulls `bevy_gltf` and `gltf_animation` (`Cargo.lock:754`,
   `:872`). **Only `file_watcher` is off** — nothing in any manifest mentions it and
   `notify-debouncer-full` is absent from the lock file entirely. This was corrected in three
   downstream artifacts (`sprint-status.yaml:1415`, `10-4-...md:26-37`, `10-1-...md:42`) and
   **never in `epics.md`**. There is no feature to enable for glTF; the pines already load through it.

2. **THE LANTERN IS NOT CLIENT-TABLE-DRIVEN, and the epic's phrasing will mislead a dev agent.**
   `epics.md:1566-1567` says "the lantern-carrying dwarf keeps its table-driven moving light".
   `entity_light_kind(EntityKind::Dwarf)` returns **`None`** (`project.rs:501`) — the client
   deliberately refuses to light a dwarf by kind, and `an_unlit_dwarf_gets_no_point_light`
   (`headless.rs:1088`) pins that refusal. The lantern arrives **over the wire** as `entity.light`,
   originating at `sim-core/src/lib.rs:120` (`DWARF_LIGHT`). What *is* table-driven is the light's
   own colour/intensity/range (`light_properties`, `appearance.rs:52-89`, lantern row `:81-87`).
   **The thing to preserve is the wire path, not a kind lookup.**

3. **HOT-RELOAD AND THE EMBEDDED DELIVERY COLLIDE, AND NO DOCUMENT SAYS SO.** The pines are
   delivered by a hand-rolled `include_bytes!` table (`ingest.rs:229-247`) registered with
   `EmbeddedAssetRegistry::insert_asset(PathBuf::new(), ...)` (`ingest.rs:265-272`) — **not** Bevy's
   `embedded_asset!` macro and not a build script. That was deliberate: `resolve_asset_root` and
   `GUI_WORKSPACE_ROOT` were deleted when the pines were embedded, because they stamped *this
   machine's absolute Linux path* into a binary that gets copied to Windows (`crates/gui/build.rs:21-25`).

   Bevy's bridge between the two is `embedded_watcher` (`bevy_asset-0.19.0/Cargo.toml:39`,
   `embedded_watcher = ["file_watcher"]`), which watches the **source file** and overwrites the
   embedded bytes in the running binary. It resolves that source through `get_base_path()`, the
   **compile-time** path (`bevy_asset-0.19.0/src/io/embedded/embedded_watcher.rs:34`), and it maps
   embedded paths back via the `full_path` argument this project passes as **`PathBuf::new()`**.

   **Consequence, and it is the story's central question:** the machine that can *show* a dwarf (the
   Windows vehicle) has no source tree at that path, and the machine that has the source tree (the
   devpod) cannot open a window. Hot-reload as the addendum promised it does not have a venue.
   **See "Wolf's ruling required" below. Do not pick one of these silently.**

4. **AN ASSET-CONTRACT DWARF WILL FLOAT HALF A CELL.** Entities are placed at the cell **centre** —
   `world_to_render(position)` with no offset (`project.rs:1480`, `:1491`). Trees are placed at the
   cell **floor** — `world_to_render(base) - Vec3::Y * 0.5` (`project.rs:2275`). The cube works today
   only because a unit cube's centre is its middle. The asset contract requires `min Y = 0`
   (`docs/tech-art-guidelines.md:437-439`), so a conforming model dropped onto the entity path
   stands half a cell in the air. **This is a task, not a surprise at the sitting.**

5. **`scale: 0.65` IS PINNED TWICE, AND ONE PIN IS A SOURCE-TEXT GREP ACROSS TWO LANGUAGES.**
   `appearance_tables_pin_the_cold_boot_palette` asserts the exact value (`appearance.rs:410`,
   `:414-418`). `bench_literals_match_the_client_palette_lights_and_boot_camera`
   (`bench_contract.rs:111-115`) greps the **literal source text** of `appearance.rs` for
   `EntityKind::Dwarf => EntityAppearance { color: Color::srgb_u8(151, 116, 96), scale: 0.65,` AND
   requires `"dwarf": ((151, 116, 96), 0.65)` in `scripts/bench/valley_bench.py:54` — each matching
   **exactly once**. The contract owes 10.5 the correction to **0.75** (1.20 m / 1.6 m,
   `tech-art-guidelines.md:353-357`). **Client and bench move in one commit or the contract goes red.**

6. **`WorldAssetRoot` SPAWNS CHILDREN, AND TWO PARTITION TESTS FORBID UNMARKED ENTITIES.**
   `WorldAssetRoot` spawns the scene as children of its entity
   (`bevy_world_serialization-0.19.0/src/components.rs:12-13`), and `#[require(Transform)]` gives
   each child a `Transform`. `world_and_client_local_markers_are_a_structural_partition`
   (`headless.rs:1586-1601`) and `the_classification_pass_leaves_no_entity_outside_the_partition`
   (`headless.rs:2718-2734`) both assert **zero** entities carrying `Transform` with neither
   `WorldProjected` nor `ClientLocal`. `classify_client_local` runs **once, at `PostStartup`**
   (`ingest.rs:1193-1200`, registered `:514`), so anything spawned later is never classified.
   This is latent today only because the tree entities are equally unmarked and no fixture has both
   an `AssetServer` and a scene. **A scene-drawn dwarf in a fixture with an `AssetServer` fails both.**

7. **CONFIRMED, and it is what makes Part A possible without any art:** the glTF loading seam already
   exists end to end. `TREE_SCENE_PATHS` (`project.rs:250-256`) →
   `asset_server.load(format!("embedded://{path}#Scene0"))` (`project.rs:292-296`) →
   `WorldAssetRoot(assets.tree_scene(variant))` (`project.rs:2260-2279`). Part A rides this path with
   an existing pine `.glb` as the stand-in.

8. **CONFIRMED — the headless fallback is load-bearing.** With no `AssetServer` (every
   `MinimalPlugins` test, `headless.rs:57-71`) `ProjectionAssets` fills scene handles with
   `Handle::default()` via `map_or_else` (`project.rs:292-296`) and nothing loads. **Preserve this
   exactly** or every headless test panics.

9. **CONFIRMED — `check_asset.py` is NOT Part A's problem.** It rejects any asset that is not a
   single-mesh/single-material/single-image 64×64-atlas V1 voxel asset, on its first clause and
   before any grid or origin check (`check_asset.py:229-234`), and the contract says so in terms:
   *"an authored dwarf, for instance — will be REJECTED … That is a scope mismatch, not a contract
   violation; story 10.5 introduces the second asset family and owns generalising these clauses"*
   (`tech-art-guidelines.md:452-458`). **Part A's stand-in is an existing pine, which already passes.
   Generalising the checker belongs to Part B with the real asset in hand.**

## Wolf's ruling required — Task 1, and the story stops here without it

**The hot-reload venue.** AC3 of the epic demands *"the running client hot-reloads it without
restart — the art-iteration loop the addendum promised, demonstrated"*. Premise 3 shows that loop has
no venue as the client is built today. The options, costed:

| # | Option | Cost | What it gives up |
|---|---|---|---|
| A | **Dev-only disk loading**: a `--assets <dir>` flag that overrides the embedded source, plus `file_watcher`, and ship `assets/` beside `gui.exe` on the vehicle | one flag, one `AssetPlugin` config, `notify` enters the graph | the lone-`gui.exe` property, for dev builds only. This is the arrangement `resolve_asset_root` was deleted for — the difference is that the path becomes an explicit argument rather than a compile-time stamp, which is what made the old one fail |
| B | **Accept rebuild-and-copy**, as the pines do today, and drop the hot-reload AC | nothing | the addendum's art-iteration promise. Iteration cost is a full cross-compile per tweak |
| C | **Build natively on Windows** so source tree and window are the same machine | a Windows Rust toolchain and a second build path; the never-two-Bevy-versions rule applies | the single build venue |

**Recommendation: A, deferred to Part B.** Part A does not need hot-reload to prove the seam, and A's
value is only realised once there is a model worth iterating on. Enabling `file_watcher` in Part A
would add `notify` to the graph for a capability nothing exercises — the inert-mechanism shape this
project has shipped before. **Wolf decides; record the ruling in this file with the date.**

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier) is green, and the diff is confined to this story's own
   commit range from `baseline_commit` — not `main..HEAD`.

### The seam

2. **`EntityKind::Dwarf` is drawn from a glTF scene, not the shared cube.** With the real daemon
   running, a `gui --headless --capture` frame differs from the committed
   `10-5-signoff/control-cube-dwarves-a.png` **inside a window containing at least one wire dwarf**,
   by at least **10x the same-build noise floor measured in that same window** — two runs of one
   binary, the **worst** taken. Publish the window's coordinates and both figures.
   **NOT a whole-frame bar. The authoring run measured the whole-frame same-build noise at
   `raw=64,851 / >=4=24,243 / >=16=8,982` on an unchanged binary** — the snow is animated, so a
   whole-frame 10x bar would demand a quarter of the frame change and five dwarf-sized silhouettes
   cannot reach it. This is 10.7's AC11 lesson arriving one story later: a whole-frame bar is the
   wrong instrument for a local change, and it fails in the direction that looks like the feature
   is broken.
3. **The dwarf stands on the ground.** Its rendered base sits at the cell floor, not its centre: the
   entity spawn applies the same `- Vec3::Y * 0.5` drop the tree path uses (`project.rs:2275`), and a
   test asserts the spawned `Transform`'s translation against a hand-written expected value that is
   **not** derived from the expression under test.
4. **Scale is corrected to 0.75 in client and bench in ONE commit.** `appearance.rs` and
   `scripts/bench/valley_bench.py:54` both carry it, `bench_contract.rs`'s dwarf anchor still matches
   **exactly once** on each side, and `appearance_tables_pin_the_cold_boot_palette` is updated rather
   than loosened.

### What must not break

5. **Position blending survives.** The update arm still never re-inserts a `Transform` on an existing
   entity (`project.rs:1464-1476`), and `later_production_reconciliation_does_not_clobber_a_blended_translation`
   (`headless.rs:988`) stays green.
6. **The lantern still rides the dwarf's own transform.** The `PointLight` remains on the
   `WorldProjected` entity, not on a scene child, so `LanternStats` samples the right translation
   (`capture.rs:799`, `:833-841`); `a_dwarf_lantern_stays_on_its_blended_projection_transform`
   (`headless.rs:1052`) stays green.
7. **The marker partition still holds with scene children.** Both
   `world_and_client_local_markers_are_a_structural_partition` (`headless.rs:1586`) and
   `the_classification_pass_leaves_no_entity_outside_the_partition` (`headless.rs:2725`) pass with a
   scene-drawn dwarf **in a fixture that has an `AssetServer`** — a fixture without one does not
   exercise this and does not satisfy the AC.
8. **No new emissive.** `switching_every_light_off_darkens_the_frame_and_leaves_no_emitter_glowing`
   (`pixel_guard.rs:222`) still measures `warm_lit_pixels == 0` with all five sources off. The
   stand-in's material must not introduce an emissive nothing can switch off
   (`emissive_materials()` is Torch + Campfire only, `project.rs:479-484`).
9. **Headless tests keep working without an `AssetServer`.** The `Handle::default()` fallback
   (`project.rs:292-296`) is preserved for the dwarf scene handle.

### The instrument

10. **A startup line reports the dwarf scene as the tree line does**, naming how many dwarf scenes
    loaded and from which source — the counterpart of
    `gui trees: meshes=… scenes_loaded=… source=embedded` (`project.rs:2239`). **The line is tested by
    driving the real binary**, and the test asserts the line CHANGES when the underlying state does,
    not merely that it is well-formed.

### Scope

11. **`file_watcher` is not enabled unless Wolf's Task 1 ruling says so**, and whatever the ruling is,
    it is recorded in this file with its date and its justification line against the trim's recorded
    reasons (`Cargo.toml:19-20` shows the existing comment style).
12. `_bmad-output/implementation-artifacts/mutations/10-5-dwarves-worth-looking-at.sh` carries at
    least **three rows the mutation run kills**, one of them AC3's floor-offset row.

## Tasks / Subtasks

- [ ] **Task 1 — Wolf's ruling on the hot-reload venue (AC11).** Put the table above in front of Wolf,
      take the decision, record it here with the date. **Nothing else starts until this is recorded**;
      the answer decides whether `file_watcher` and a `--assets` flag are in scope at all.
- [ ] **Task 2 — the seam draws a scene (AC2, AC9).**
  - [ ] Extend `ProjectionAssets` (`project.rs:229-245`) with a dwarf scene handle, loaded in
        `setup_projection_assets` (`project.rs:257-296`) through the **same `map_or_else` fallback**
        the trees use, so a missing `AssetServer` yields `Handle::default()`.
  - [ ] At the spawn arm (`project.rs:1485-1493`) draw `EntityKind::Dwarf` with
        `WorldAssetRoot(...)` instead of `Mesh3d`/`MeshMaterial3d`. Every other kind keeps the cube.
  - [ ] Stand-in asset: an existing `assets/trees/*.glb`, chosen because it already passes
        `check_asset.py` and is **obviously not a dwarf**, so no frame from Part A can be mistaken for
        a finished dwarf.
- [ ] **Task 3 — the floor offset (AC3).** Apply the tree path's `- Vec3::Y * 0.5` to the entity
      spawn; test the resulting translation against a hand-written expected value.
- [ ] **Task 4 — scale 0.65 → 0.75 (AC4).** `appearance.rs` and `valley_bench.py:54` in one commit;
      update `appearance_tables_pin_the_cold_boot_palette`; confirm both `bench_contract.rs` anchors
      still match exactly once.
- [ ] **Task 5 — the partition (AC7).** Decide how scene children are classified. Note
      `classify_client_local` runs only at `PostStartup` (`ingest.rs:1193-1200`), so a runtime-spawned
      child is never reached. Add the `AssetServer`-bearing fixture that actually exercises it.
- [ ] **Task 6 — regressions (AC5, AC6, AC8).** Blending, lantern transform, emissive. Run the pixel
      guards, which the fast gate skips.
- [ ] **Task 7 — the instrument and its test (AC10).**
- [ ] **Task 8 — mutations (AC12), then the full gate (AC1).**

## Dev Notes

### Scope guardrails — do NOT

- **Do not author or commit a dwarf model.** That is Part B and it is Wolf's hand, not a generated
  stand-in. A generated "placeholder dwarf" would become the thing everyone judges — see
  [placeholder sets the budget]: 96.8% of 10.6's triangle budget was the measurement stand-in.
- **Do not touch `check_asset.py`.** Its V1-voxel clauses are correct for what exists; generalising
  them without the real asset in hand means guessing the second family's shape.
- **Do not enable `file_watcher`** ahead of Task 1's ruling.
- **Do not re-tune any look constant.** Not the sun (10.7), not the palette, not
  `NEAR_WHITE_AREA_CEILING` — `gui --headless --capture` already exits 101 on `main`
  (`pixel_guard.rs:173-177`), so a red capture exit is not evidence about dwarves.
- **Do not re-insert a `Transform` on the reconcile update arm** (`project.rs:1464-1465`).

### What already exists — build on it, do not rebuild it

- The whole embedded-glTF path: `include_bytes!` table (`ingest.rs:229-247`) → `insert_asset`
  (`ingest.rs:265-272`) → `asset_server.load("embedded://…#Scene0")` (`project.rs:292-296`) →
  `WorldAssetRoot` (`project.rs:2260-2279`).
- `authored_bench.py` — the shipped mesh bench that renders authored assets **in situ** in our
  valley, with the 1/1.6 cell-to-metre scaling already reasoned out. Part B's UX-DR22 opening
  artifact should extend this rather than start over.
- `pixel_diff.py`, `lumstats.py`, `enclosed.py` in `10-7-signoff/` — the measured-frame instruments,
  each with a colour-type guard and a known noise floor.

### Key decisions and traps

- **A delta is not a level.** 10.7 closed an AC on a metric whose delta tracked the defect and whose
  level never could, and shipped 54 holes under a green guard. Any "the dwarf changed" measurement
  must be able to answer "and nothing else did".
- **Blender's glTF importer leaves `rotation_mode` on `QUATERNION`; assigning `rotation_euler` is
  then silently ignored** (10.4). If Part A or B touches orientation in a bench script, set the mode.
- **`obj.dimensions` is the LOCAL bounding box and ignores rotation** (10.4) — measure the world
  bounding box, and assert the base sits at Y=0.
- **A mesh dwarf is invisible to any cube-counting oracle**, exactly as mesh trees made
  `exposed_cells` read 40,148 instead of 44,984.
- **The 1.20 m dwarf anchor is not ratified anywhere.** `tech-art-guidelines.md:353` carries it, and
  `10-3-...md:39-42` records that it was copied from `10-2-signoff/ASSET_NOTES.md:36` and never
  ruled on. AC4 changes the client to match the contract; if Wolf wants a different dwarf height,
  Part B is where that lands, and this AC's number moves with it.

### Project structure — files to touch

| Path | NEW/UPDATE | Why |
|---|---|---|
| `crates/gui/src/project.rs` | UPDATE | `ProjectionAssets` (:229), `setup_projection_assets` (:257), spawn arm (:1485), floor offset |
| `crates/gui/src/appearance.rs` | UPDATE | dwarf `scale` 0.65 → 0.75 (:280), table test (:410) |
| `scripts/bench/valley_bench.py` | UPDATE | dwarf tuple (:54) — same commit as `appearance.rs` |
| `crates/gui/tests/bench_contract.rs` | UPDATE | dwarf anchors (:111-115) |
| `crates/gui/tests/headless.rs` | UPDATE | partition fixture with an `AssetServer`; blending/lantern regressions |
| `_bmad-output/implementation-artifacts/mutations/10-5-dwarves-worth-looking-at.sh` | NEW | AC12 |

### References

- `epics.md:1355-1362` execution order · `:1556-1593` story 10.5 · `:218` UX-DR22
- `docs/tech-art-guidelines.md:337-358` resolution contract · `:423-493` asset contract · `:452-458`
  the V1-only scope trap
- `_bmad-output/implementation-artifacts/10-4-the-trees-look-right-the-pilot.md:26-57` the premise
  corrections · `:363-373` what was deleted when the pines were embedded · `:423-433` the two
  importer traps
- `_bmad-output/implementation-artifacts/10-3-the-rules-of-the-look.md:452-459` why an authored dwarf
  fails the checker
- `bevy_asset-0.19.0/Cargo.toml:39-44`, `src/io/embedded/embedded_watcher.rs:26-44` hot-reload mechanics

## Verification

**Executed at story creation, 2026-09-04, on `834f105`.** The instrument is `10-7-signoff/pixel_diff.py`,
reused rather than reinvented, and the control frames it grades are committed by this story.

Control frames captured on a CLEAN tree so their stamp is trustworthy: the first attempt read
`gui build 87f3bdc-dirty` — a binary built at the previous commit — and was discarded and re-taken
rather than committed. The committed pair carries **`gui build 4b01a58`**, this story's own commit.

```bash
./target/debug/simd 7491 &
./target/debug/gui 7491 --headless --subdiv 1 --frames 160 \
  --capture _bmad-output/implementation-artifacts/10-5-signoff/control-cube-dwarves-a.png
# ...and again to -b.png. Exits 101 on the near-white ceiling; the PNG is still written.
```

### The deliberate RED, observed before any green was accepted

The instrument can fail in two directions — by dying, and by lying — so both were driven:

```
RED 1  a truncated frame
       zlib.error: Error -5 while decompressing data: incomplete or truncated stream
RED 2  a frame diffed against ITSELF
       RED2-self       raw=      0  >=4=      0  >=16=      0
```

RED 1 proves it dies loudly on a bad frame rather than printing plausible numbers. RED 2 proves it
can say **nothing changed** — without which "the dwarf changed" means nothing. Restore: none, both
REDs are scratch files.

### The noise floor, and why AC2 is a LOCAL bar

```
NOISE-a-vs-b            raw= 64,851   >=4= 24,243   >=16=  8,982
```

**That is two captures of ONE unchanged binary.** The snow is animated, so tens of thousands of
pixels differ frame to frame with no code change at all. A whole-frame 10x bar is therefore
unreachable by five dwarf-sized silhouettes, which is why AC2 asks for a windowed measurement and
its own windowed floor. The dev agent must re-measure both on their build; these figures are the
shape of the problem, not a number to inherit.

**The obligation this recipe cannot yet discharge:** the "after" frame does not exist at authoring
time. The dev agent must produce it with the same command against the same daemon port and framing,
and paste both the noise floor measured on **their** build and the change, per AC2.

## Branch and commits

Branch `10-5-dwarves-worth-looking-at`, off `main` at `834f105` (which carries 10.7 and its closure). Author `Völundr <jeicei75@gmail.com>`.
Small commits, imperative messages. Push and PR only on Wolf's explicit yes.

## Change Log

| Date | Change |
|---|---|
| 2026-09-04 | Story created. **The epic's named split line is taken and this is Part A, the seam.** Nine premises re-verified on `834f105`: six were wrong, stale or absent — `bevy_gltf` is already enabled (AC1 half-moot), the lantern is wire-driven not client-table-driven, hot-reload has no venue under the embedded-delivery decision, an asset-contract dwarf floats half a cell on the entity path, `scale: 0.65` is pinned by a two-language source-text grep, and `WorldAssetRoot`'s scene children break two partition tests. Wolf's ruling required on the hot-reload venue before Task 2. |
| 2026-09-04 | **Verification recipe EXECUTED, and it falsified this story's own AC2.** Two REDs observed (a truncated frame dies with `zlib.error`; a frame against itself reads exactly 0/0/0). The same-build noise floor on an unchanged binary is `raw=64,851 / >=4=24,243 / >=16=8,982` — the snow is animated — so the whole-frame 10x bar AC2 originally carried was **unreachable by five dwarf-sized silhouettes**, the same defect 10.7 hit with its AC11 whole-frame bar. AC2 is now a WINDOWED bar against a windowed floor. Control frames re-taken on a clean tree after the first pair came out stamped `87f3bdc-dirty`; the committed pair carries `4b01a58`. |

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Review Findings
