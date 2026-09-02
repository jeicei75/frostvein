# Story 10.4 Task 2 — the procedural-vs-authored decision

**Decided by Wolf, 2026-09-02.** Artifact it rests on:
`candidate-D-authored-pines-blender-5.2.1.png` (height-exact revision), venue **Blender 5.2.1 LTS**.

> **Ruling: the mesh path wins, and story 10.4 lands it in the client rather than deferring it.**
>
> Wolf was shown Task 2's stop-if-authored instruction and the option to scope the client work as
> a separate story, and chose to absorb it here. The override is deliberate and recorded as such.

## The decision UX-DR22's opening half required

Wolf approved a bench artifact **before** any client change was implemented. At the time of this
ruling the branch contained only Task 0's documentation correction and the Task 1/Task 2 bench
artifacts; `crates/` was untouched. Verified: `git diff 8f5d0c1..6d737e8 -- crates/` is empty.

## What was judged

| artifact | treatment | range-check (blender=5.2.1) | mean Δ vs control |
|---|---|---|---|
| `control-shipped-trees` | shipped cube trees, taper 0.62/0.78/0.95 | `exposed_cells=44984 non_sky_fraction=0.686736 distinct_colors=59191 terrain_luma=105.853` | — |
| `candidate-A-0.50-0.72-0.98` | sharper spire | `exposed_cells=44984 non_sky_fraction=0.684421 distinct_colors=58776 terrain_luma=105.994` | 5.27 |
| `candidate-B-0.72-0.88-0.98` | fuller crown | `exposed_cells=44984 non_sky_fraction=0.688540 distinct_colors=58679 terrain_luma=105.371` | 5.76 |
| `candidate-C-0.52-0.68-0.86` | sparser crown | `exposed_cells=44984 non_sky_fraction=0.684587 distinct_colors=58549 terrain_luma=106.916` | 5.61 |
| **`candidate-D-authored-pines`** | **10.2 voxel pines, height-exact** | `exposed_cells=40148 non_sky_fraction=0.678709 distinct_colors=69944 terrain_luma=117.770` | **21.38** |
| `red-no-trees` | every tree cell removed (instrument RED) | `exposed_cells=40148 non_sky_fraction=0.662805 distinct_colors=27999 terrain_luma=125.883` | 26.07 |

"mean Δ" is the mean per-pixel maximum channel difference against the control, 0–255, measured
under Blender on the committed PNGs.

## Why the taper lost

**The knob cannot reach the goal, and this is arithmetic rather than taste.** `foliage_scale`
shrinks each foliage cube *within its own cell*. It cannot move a cube, add one, or change the
crown's shape. The crown's shape is fixed by `place_trees`: a tip cell plus two 3×3-minus-centre
rings — a flat disc with a one-cell spike, about 19 exposed cells per tree.

- The whole taper sweep (A→B, the extremes tested) moves the frame **5.27–5.76** mean Δ.
  Deleting every tree moves it **26.07**. The knob is worth **~1/5 of the trees' own existence**.
- Measured resolution gap: the bench draws **103 triangles per tree**; an authored pine is
  **3,474–5,894**. That is **34–57×**. A tiered conifer silhouette needs more primitives than a
  tree has cells, so no triple of scale factors produces one.

## Why "procedural vs authored" was the wrong frame

`10-2-signoff/voxel_pine.py` **is a procedural generator** — 717 lines, deterministic, seeded, all
variation from an integer hash of `(seed, x, y, z, tag)`, no clock, no filesystem, no `random`,
sorted iteration, byte-identical GLB for identical arguments. The determinism and
no-hand-modelling properties Wolf's standing procedural instinct (2026-08-28) was protecting are
**already fully present in the mesh path**.

The real difference is **where the generator runs and at what resolution**: in the client at one
cube per 1.6 m cell, or offline in Blender at 0.2 m voxels — 8× finer linearly — baked to a mesh.
The instinct is therefore **confirmed, not overturned**; what changes is the venue.

## Two defects found in the shipped taper while judging

1. **The `0.95` arm renders on zero cells.** Counting foliage by the `above` index `foliage_scale`
   switches on, straight from the exported snapshot: `above=0` → 2,385 cells, `above=1` → 2,120,
   **`above=2` → 0**. Provable from `place_trees`: trunk rejection forces trees ≥3 apart in
   Chebyshev while a crown ring spans radius 1, so no column ever receives foliage from two trees,
   and a ring column holds exactly two consecutive foliage cells. The shipped taper is two-step.
   `foliage_tapers_from_wide_mid_crown_to_narrow_tip_and_skirt` passes only because its fixture is
   a synthetic 1×1×6 column with three stacked foliage cells — a shape worldgen cannot produce.
   `bench_contract.rs` pins `0.95` in both Rust and Python: a guard holding a value nothing renders.
2. **53% of foliage renders as snow, not green** — 2,385 of 4,505 cells are snow-laden crowns, and
   they are exactly the top-of-column set. The shipped tree reads as a grey platter over a thin
   green band.

Neither defect is fixed by this story's client change; both are recorded so the mesh work does not
inherit them silently.

## Upstream defect: the reference sheet contradicts itself on Type 4

`10-2-signoff/reference-sheet.jpg` Section B labels every variant twice. For Type 4 the labels
disagree, and only for Type 4:

| variant | cell label | dwarf label | dwarf label × 1.20 m | in cells | agree? |
|---|---|---|---|---|---|
| Type 1 | 4 CELLS | 5.32× | 6.38 m | 3.99 | yes |
| Type 2 | 5 CELLS | 6.67× | 8.00 m | 5.00 | yes |
| Type 3 | 5 CELLS | 6.67× | 8.00 m | 5.00 | yes |
| **Type 4** | **6 CELLS** | **8.80×** | **10.56 m** | **6.60** | **NO** |

The cell label is the half the simulation can honour — `place_trees` has a hard ceiling of 6 —
so it wins. `pine_6cell.py` regenerates Tree04 scaled uniformly to **9.6 m = exactly 6 cells**
(48 voxels at 0.2 m), restoring the 8-voxels-per-cell alignment the other three already have
(32 = 4×8, 40 = 5×8, 48 = 6×8; the shipped 53 is 6.625×8). Height-overshooting placements fall
from **103 of 265 to 0**. `SM_VoxelPine_Tree04R.glb` passes `check_asset.py`.

**Left open for the client work, deliberately not decided here:** whether Tree04R supersedes
`SM_VoxelPine_Tree04` as 10.2's deliverable, and whether the reference sheet itself is corrected.
This story generated Tree04R as a 10.4 scratch asset and did not overwrite a signed-off artifact.

## What the ruling costs, measured

- **1,187,132 triangles** for 265 pines — **2.20× the entire current terrain** (539,808 = 44,984
  exposed cells × 12) and **19.6×** today's cube trees (60,576). Whether that is affordable is an
  **fps question this venue cannot answer**: lavapipe renders but does not clock. It is a hole
  carried into review, not a solved problem.
- **`exposed_cells` reads 40,148, not 44,984 — and that is correct.** The cube oracle cannot see
  mesh trees. A mesh-tree world retires 44,984 as a description of the trees, which is why Task 4
  is a re-derivation rather than a renumber.
- The client loads **zero** external meshes today: no `AssetServer`, no `SceneRoot`, no `assets/`
  directory. `bevy_gltf` is already compiled in transitively via `3d_bevy_render`, so the feature
  is not the missing piece — the plumbing is.

## Method note, so the comparison can be trusted

`authored_bench.py` **imports** `valley_bench` rather than forking it, so the terrain, camera,
lights and materials in candidate D are the same code that produced the control. Trees are
stripped from the snapshot exactly as the story's RED recipe does, then one pine is instanced per
trunk column, scaled by 1/1.6 to the resolution contract and **not stretched to fit**.

Two silent-failure traps were hit and are recorded because both would have produced a
confident-looking wrong artifact:

- The glTF importer leaves `rotation_mode` on `QUATERNION`, so assigning `rotation_euler` is
  **ignored with no error**. The pines rendered unrotated and nothing complained.
- The first orientation guard read `obj.dimensions`, which is the **local** bounding box and
  ignores rotation entirely — it would have passed a mis-oriented tree. The check now measures the
  **world** bounding box, and additionally asserts the base sits at Y=0. This is 10.3's
  leaf-only-check defect in a new costume.
