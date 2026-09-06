# src-assets — the working art tree

Editable art sources for frostvein: Blender generators, working `.blend` files, and the
approved reference material. **This folder is the source; it is not the delivery.** The
runtime assets the client compiles in live at `assets/trees/` (and, from story 10.5,
`assets/gltf/`), and the asset contract in `docs/tech-art-guidelines.md` is the authority
over everything here.

**Reconciled 2026-09-06.** The folder arrived as a snapshot of a Windows Blender session
that predated story 10.2's review hardening and story 10.4's Tree04 rescale, so its
generator and its notes were both a revision behind the repo. The forge side won, per
Wolf's ruling; what follows is the reconciled record.

## Layout

```
blender/     the generators, and the working .blend files
prompts/     briefs for MCP/agent modelling sessions
references/  the approved reference sheets and the mood art
export/      LOCAL SCRATCH, gitignored -- see below
```

## Generating the pines

Use **Blender 5.2.1** — `/opt/blender-5.2/blender` in the devpod, matching the vehicle.
The apt build at `/usr/bin/blender` is 4.3.2 and the generator does not run under it.

```
blender --background --python blender/voxel_pine.py -- <type 1-4> <out.glb> [--seed N] [--voxel M]
```

**One generator, four types.** Type 4 builds `Tree04R`; see below for why.

**Write straight to `assets/trees/`.** A second copy of a shipped `.glb` is exactly how a
bench and a client come to draw different trees, which is the reasoning
`scripts/bench/authored_bench.py:55-58` already carries ("never a signoff copy").

> **`export/` is gitignored local scratch as of 2026-09-06 and is no longer tracked.** It
> had held a duplicate of `Tree0{1,2,3}` plus the **superseded 10.6 m `Tree04`** that ships
> nowhere. Exporting there from a Blender session is fine; nothing in the repo reads it, and
> nothing in it can become a second committed copy of a shipped asset.

```
blender --background --python blender/voxel_pine.py -- 1 ../assets/trees/SM_VoxelPine_Tree01.glb
blender --background --python blender/voxel_pine.py -- 2 ../assets/trees/SM_VoxelPine_Tree02.glb
blender --background --python blender/voxel_pine.py -- 3 ../assets/trees/SM_VoxelPine_Tree03.glb
blender --background --python blender/voxel_pine.py -- 4 ../assets/trees/SM_VoxelPine_Tree04R.glb
```

The output basename must be the variant's published name: `check_asset.py` requires the file
basename, the glTF mesh name and the node name to agree, and type 4's is **`Tree04R`**.

The script prints one `FIGURES` line, then checks it and exits non-zero if any check fails.
Same arguments give a byte-identical GLB.

**Verified 2026-09-06** on this devpod under Blender 5.2.1, and re-verified after type 4 was
folded into this script: all four regenerate byte-for-byte identical to the shipped files.

| GLB | sha256 (first 16) | bytes |
|---|---|---|
| `SM_VoxelPine_Tree01.glb` | `2fc387f1397996b4` | 307,160 |
| `SM_VoxelPine_Tree02.glb` | `830c84ba5ff1ff97` | 414,136 |
| `SM_VoxelPine_Tree03.glb` | `492607b290eda2e3` | 244,760 |
| `SM_VoxelPine_Tree04R.glb` | `1c618419dbfa1452` | 311,284 |

## Variants

| Type | Label | Cells | Height | Size (m, XYZ) | Tris | Verts |
|---|---|---|---|---|---|---|
| 1 | `Tree01` | 4 | 6.4 m | 5.0 × 6.4 × 5.0 | 4,366 | 8,732 |
| 2 | `Tree02` | 5 | 8.0 m | 5.0 × 8.0 × 5.4 | 5,894 | 11,788 |
| 3 | `Tree03` | 5 | 8.0 m | 3.8 × 8.0 × 3.4 | 3,474 | 6,948 |
| 4 | **`Tree04R`** | **6** | **9.6 m** | **4.2 × 9.6 × 4.2** | **4,424** | **8,848** |

Every dimension derives from the **0.2 m** authored voxel and the dwarf anchored at
**1.20 m**. Rescale with `--voxel` if the dwarf height moves.

### Type 4 is 4R, and why

`voxel_pine.py -- 4` builds **`Tree04R`, the 9.6 m tree that ships**. The superseded 10.6 m
`Tree04` is no longer buildable from this script: it survives only as `_TREE04_PRE_RESCALE`,
the documented *input* to the rescale arithmetic.

The approved `references/reference-sheet.jpg` labels each Section B variant twice, and for
Type 4 the two labels contradict each other:

| Type | cell label | dwarf-multiple label | ⇒ | |
|---|---|---|---|---|
| 1 | 4 CELLS | 5.32× dwarf | 6.38 m = 3.99 cells | agree |
| 2 | 5 CELLS | 6.67× dwarf | 8.00 m = 5.00 cells | agree |
| 3 | 5 CELLS | 6.67× dwarf | 8.00 m = 5.00 cells | agree |
| 4 | 6 CELLS | 8.80× dwarf | 10.56 m = 6.60 cells | **CLASH** |

Story 10.4 resolved it to the **cell** label: `place_trees` has a hard ceiling of 6 cells,
so a 6.625-cell tree fits no tree the simulation can generate — it overshot on 103 of 265
placements, every one of them 1.0 m too tall. The spec is rescaled **uniformly** by 48/53
(`factor 0.90566`), because scaling height alone would squat the silhouette into a different
tree, and 10.4 was comparing the asset rather than proposing a new design.

`_rescale_tree04()` **computes** that spec at import rather than carrying a table of rounded
numbers. That is load-bearing: `SM_VoxelPine_Tree04R.glb` reproduces byte-for-byte only if
these floats are bit-identical to the ones 10.4 produced, and transcribing them would change
the bytes. It arrived here as a separate `pine_6cell.py` importing `voxel_pine.py` across an
absolute path; folding it in on 2026-09-06 removed both the second script and the path.

`references/dwarf-animation-reference.jpg` is a second, separate reference sheet — the
5-pose mining-strike cycle. It is a GFX reference, not a modelling dimension source.

## Generating the dwarf

```
blender --background --python blender/dwarf_miner.py -- <out.glb> [--voxel M] [--blend P]
```

**One generator, one variant**, and it refuses any basename but the published one, because
the contract requires file basename == glTF mesh name == glTF node name and that string is
compiled into the client. Write straight to `assets/gltf/`, the location
`docs/tech-art-guidelines.md:463` contracts for a runtime glTF:

```
blender --background --python blender/dwarf_miner.py -- ../assets/gltf/SM_VoxelDwarf_Miner01.glb
```

It draws **no random numbers at all** — every voxel is placed by an explicit rule — so there
is no `--seed` and determinism is by construction rather than by discipline.

**Verified 2026-09-06** under Blender 5.2.1: two independent cold runs and the shipped file
are byte-identical, and `scripts/bench/check_asset.py` accepts it with **no code change**.

| GLB | sha256 (first 16) | bytes | Size (m, XYZ) | Voxels | Tris | Verts |
|---|---|---|---|---|---|---|
| `SM_VoxelDwarf_Miner01.glb` | `3a87cbbd03660316` | 16,688 | 1.0 × 1.2 × 0.6 | 239 | 216 | 432 |

Gear is **baked into the one mesh** (Wolf's ruling, 2026-09-06) as voxel groups that touch
the body only at the hands — body 216, pickaxe 11, lantern 12, reported on the `FIGURES`
line. Since the mesh is unwelded quad soup, cutting the gear out for the mining-strike cycle
later is a selection, not a remodel.

### Palette

Section A's swatch column, the sheet's `Needle Green` dropped (it is the tree's).

| Cell | Hex | Role | Provenance |
|---|---|---|---|
| 0 | `#E9D2BB` | Skin | label legible, swatch agrees |
| 1 | `#5E4632` | Beard — also the hair | label legible, swatch agrees |
| 2 | `#FFFFFF` | Snow — **used as the lantern's glass pane** | confirmed: same value in the pine atlas |
| 3 | `#5F7A6A` | Tunic | label legible, swatch agrees |
| 4 | `#474B41` | Pants | **unresolved, see below** |
| 5 | `#A9B2AC` | Metal | label legible, swatch agrees |
| 6 | `#8B6B50` | Wood | **unresolved, see below** |
| 7 | `#6B5B49` | Wood Trunk — belt, boots, backpack | confirmed: same value in the pine atlas |

The lantern's pane is **plain albedo and not emissive**. The client owns the lantern's light;
a baked emissive face is a light nobody can switch off, which the pixel guards assert against.

### Three findings

**1. `Pants` and `Wood` cannot be settled from the sheet, and were carried from the brief.**
`reference-sheet.jpg` is 1024×558 — that *is* its full resolution, not a downscale, so there
is no better scan to re-read. At that size the sheet's `8` and `B` render as the same glyph:
`Wood Trunk` reads `#685849` there while the pines demonstrably ship `#6B5B49`. Sampling the
swatches cannot break the tie either. Absolute luminance is useless (measured bias −17..+1
across the swatches), and channel differences, which cancel most of it, still carry up to
**8.8/255** of error when calibrated against the three swatches whose truth is known — well
over the **3/255** that separates `#474B41` from `#474841` and `#8B6B50` from `#8B6850`. Both
values above are the brief's, **carried and not confirmed**; the residuals actually lean
`#474841` for Pants and split for Wood. If the original art file exists, one look settles it.

**2. The pine's half-voxel lattice shift is off the project grid at a 0.1 m voxel.**
`voxel_pine.greedy_mesh()` subtracts half a voxel in X and Y so an ODD-width trunk centres on
the origin. At the pine's 0.2 m voxel that shift is 0.1 m and lands on the grid; at the
dwarf's 0.1 m voxel it is **0.05 m**, and `check_asset.py`'s grid clause rejects every vertex.
The dwarf is therefore **even-width in X and Y** and undoes the shift after meshing — a rigid
translation, which is why the mesher itself is imported unedited. A consequence worth
recording: the resolution contract's phrasing "every voxel centre lands on the 0.1 m lattice"
and the checker's clause (POSITION on the 0.1 m grid) **cannot both hold at a 0.1 m voxel**.
The checker is the mechanical gate, so it wins here; voxel centres land on the half-grid.

**3. The sheet's Section A depicts roughly twice the resolution the contract allows, and
that is a ruling, not a defect.** The contract fixes the dwarf at 12 voxels of 0.1 m
(`docs/tech-art-guidelines.md:352`). Measured off the sheet's own back view — silhouette
bounding box 29 × 92 px, modal edge-step 4–5 px — the **drawn** figure is about **20–23
voxels tall and 7 wide**, and it is not voxel art at all: anti-aliased outlines, gradient
shading and a curved organic pickaxe head. At 12 voxels the following are simply unavailable
and were dropped:

- **eyes** — once hair frames it the face is 2 voxels wide, so a separated pair cannot be
  drawn and an adjacent pair reads as a brow band
- the moustache as a mass distinct from the beard; bracers; the tunic's layered panels;
  boot cuffs
- the pickaxe's **curved** pick and its hammer poll — it is a straight bar with two
  turned-down ends, which is the least that still reads as a pickaxe rather than a hook
- the lantern's frame and mullions — it is base / pane / cap, three voxels

Nothing here is fixable by modelling harder. **0.1 m is the floor** the brief and the
contract both set, so the only exits are to accept the coarse read (what ships today), or to
raise the 1.20 m anchor — which re-labels every tree's dwarf-multiple on the sheet and moves
`gui`'s `scale`. Wolf ratified 1.20 m on 2026-09-06, **before** this measurement existed.

The full session record is `prompts/dwarf-miner-blender-mcp-report.md`.

## Trunk proportions

The trunk column and the root plate are sized independently, on purpose: a **wide plate
under a slender column** is the proportion the sheet reads with.

`trunk_r=(base, tip)` is a float radius on a radial cross-section, so thickness tunes in
sub-voxel steps instead of jumping 3×3 → 5×5:

| radius | cells | reads as |
|---|---|---|
| ≤ 1.0 | 5 | plus/cross, 3 wide with notched corners |
| ≤ 1.5 | 9 | full 3×3 |
| ≤ 2.0 | 13 | 13-cell cross, 5 wide at the cardinals |
| ≤ 2.3 | 21 | rounded 5×5 |
| ≤ 2.9 | 25 | full 5×5 |

Tuned values give every type a solid **3-voxel (0.6 m) column** through the visible run,
with only the bottom row or two widening to blend into the plate. Types 3 and 4 carry the
long exposed trunks, so they get slightly more base. Values below ~1.2 taper to the plus
section too early and the trunk looks pinched; 5×5 throughout (the first pass) reads as a
stone pillar. `flare=(z0, z1)` sizes the root plate and is left at its original width.

## Palette

A single 64×64 PNG atlas embedded in the GLB, 4×4 grid of 16 px cells, UVs inset to each
cell centre. Exported with `magFilter NEAREST` — **do not let the importer switch it to
linear**, or cells bleed.

| Cell | Hex | Role |
|---|---|---|
| 0 | `#4A3B2E` | Trunk Brown (sheet) |
| 1 | `#6B5B49` | Wood Trunk (sheet) |
| 2 | `#2A3E34` | Needle Green, shaded |
| 3 | `#364D3F` | Needle Green (sheet) |
| 4 | `#52715B` | Needle Green, lit |
| 5 | `#FFFFFF` | Snow (sheet) |
| 6 | `#D8E4EC` | Snow, shaded |

Texels are byte-exact; the script decodes the PNG back out of the finished GLB and fails
if any cell differs. That check exists because two separate colour bugs shipped silently
before it did:

- `Image.pack()` on a `GENERATED` image re-encodes the generated source and discards
  whatever was assigned to `.pixels`; the exporter then copies those black packed bytes
  into the GLB. The script now encodes the PNG itself and packs the exact bytes.
- `bpy.data.images.new()` returns a **byte** image, so `.pixels` is display-referred.
  Linearising the hex before writing it there bakes a second sRGB decode into the texture
  and ships a visibly too-dark tree.

## Mesh topology — read before running any adjacency tool

The mesh is a **greedy-meshed voxel hull, deliberately left unwelded**. Every quad carries
its own four vertices and **no vertex is shared between quads** (`verts == quads × 4`
exactly, asserted on every build). Correct output for a voxel mesher — merging co-planar
same-colour faces is what keeps the triangle count roughly half of naïve per-face meshing
— but it means the asset **does not support**:

- **Smooth or averaged vertex normals.** There is no adjacency to average across. The
  asset is flat-shaded by design; keep it that way.
- **Subdivision surfaces.**
- **Adjacency-based auto-LOD or decimation.** Edge-collapse decimators and most auto-LOD
  tools will refuse the mesh or shred it. Generate LODs by re-running at a coarser
  `--voxel` instead.
- **Collision generation that walks shared edges.** Prefer a separate primitive collider:
  a capsule on the trunk plus a cone or box on the canopy.

Greedy merging also leaves **T-junctions** where a large quad abuts smaller ones. Harmless
for rendering, and the other reason not to run adjacency algorithms over this mesh.

Because the mesh is a quad soup, Euler characteristic and edge-manifold tests are
meaningless on it. The correct closure oracle is the **signed volume**, which must equal
`voxel_count × voxel_size³`; it is only satisfied when every exposed face is present, none
is duplicated, and all normals face outward. The script asserts this on every build.

## What the generators check

Printed as one `FIGURES` line, then asserted; any failure exits non-zero.

- bbox centre X and Z are 0, bbox min Y is 0, height matches the variant
- signed volume equals the voxel volume
- `tris == quads × 2`, `verts == quads × 4`
- exactly 1 material, 1 primitive, 1 embedded image, no glTF extensions
- material single-sided; all UVs inside 0–1
- every palette cell decodes to its sheet hex; `magFilter` is `NEAREST`
- `--voxel` must be > 0. Zero is the dangerous one: it collapses the mesh to a point AND
  scales the expected height and volume by the same zero, so every closure check passes
  vacuously and the script reports OK on nothing.
- any uncaught exception is converted to a non-zero exit. Blender's `--background` runner
  prints a traceback for an uncaught exception and **still exits 0**, so without this a
  bad argument or an unwritable path reports success having written nothing.

**The last two are why this folder's generator was replaced on 2026-09-06.** The copy that
arrived here predated both guards.

## Repo state

**Tracked and current:**

- `blender/voxel_pine.py` — **the** generator, and the only source of truth for all four
  tree variants.
- `blender/dwarf_miner.py` — the dwarf generator. Imports `voxel_pine` for the greedy
  mesher, the material, the exporter, the GLB reader and the volume oracle, and **does not
  edit it**; only the body, the palette and the atlas are its own.
- `blender/dwarf.blend` — the dwarf's editable source, saved by the generator's `--blend`.
  Regenerated, never hand-edited: the `.glb` is the deliverable and the script is the
  durable record.
- `prompts/dwarf-miner-blender-mcp-report.md` — the session record answering that folder's
  dwarf brief: figures, reproduction proof, the checker's verdict and the three findings.
- `references/reference-sheet.jpg` — the **approved** modelling sheet, Sections A (dwarf)
  and B (trees). See the caveat below.
- `references/dwarf-animation-reference.jpg`, `references/dwarf-contact-sheet.jpg`,
  `references/dwarf.mp4` — dwarf reference.
- `prompts/` — modelling-session briefs.

**Tracked, and one revision behind:**

- `blender/trees.blend` — the working file holding the tree set. Saved *before* the trunk
  pass, so its meshes are one revision behind what ships. Re-import the GLBs, or
  regenerate, before building on it. The `.glb` is the deliverable; the `.blend` is not.

**Removed 2026-09-06, on Wolf's yes — do not restore:**

- `blender/tree.blend` and `blender/tree.blend1` — the interactive **first pass**, with the
  off-centre canopy, the too-dark texture and the thick 5×5 trunks. Nothing in the pipeline
  ever read them. Story 10.2's signoff record had already ruled on this pair in terms —
  *"Safe to delete"* — and additionally notes that `tree.blend` **overwrote** the file that
  was in the project before that work, and that `tree.blend1` was an autosave of that same
  stale session rather than the original.

  Recorded here because a deletion leaves no trace where a reader will look. That session's
  sibling `tree.glb` carries the glTF mesh and node name `SM_VoxelPine_Tree02` — the
  *deliverable's* name — while being a different asset: 5,130 tris and centre X −0.100
  against the deliverable's 5,894 and +0.000. An asset's identity is its published name
  **and** its published figures, never its internal name alone, and this pair is the
  measured counterexample `docs/tech-art-guidelines.md` cites. If a `tree.blend` reappears
  here, it is that one, and it is not a source.

  `.gitignore` now excludes `*.blend[0-9]`, so no future autosave can be committed. Both
  files remain recoverable from commit `5ff1ed7`.

**Tracked, and MOOD ART — not a dimension source:**

- `references/17d7215b-….jpg`, `references/a9d4e72b-….jpg` — the valley and fortress
  vista shots. They set the mood; they are not orthographic and nothing may be measured
  off them.

### The reference sheet is a silhouette and palette authority, not a dimension authority

`reference-sheet.jpg` was approved and is the upstream reference the project settles
questions against — but it carries labels that are provably wrong, and two of them have
already cost a story:

- Section B, Type 4: the cell/dwarf-multiple clash above.
- Section B, second row: the labels read literally `X.x dwarf height` and `Y.y dwarf
  height` — unfilled placeholders.
- Section A: the dwarf's own height is labelled `0.6x dwarf height`, which is circular,
  and the `12 Voxels` arrow is drawn across the figure's **width** while the resolution
  contract reads 12 voxels as its **height** (`docs/tech-art-guidelines.md:352`,
  "dwarves target 0.1 m (12 voxels = 1.20 m = 0.75 cells)").

**The contract wins on dimensions; the sheet wins on silhouette, gear and palette.** Where
they disagree, say so in the story rather than picking silently.
