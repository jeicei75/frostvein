# Voxel Pine Trees — asset notes

Four snow-laden voxel pines from Section B of `references/reference-sheet.jpg`.
Everything here is produced by `voxel_pine.py`; there is no manual modelling
step and no hand-authored `.blend` in the pipeline.

## Generating

```
blender --background --python voxel_pine.py -- <type> <out.glb> [--seed N] [--voxel M]
```

```
blender --background --python voxel_pine.py -- 1 export/SM_VoxelPine_Tree01.glb
blender --background --python voxel_pine.py -- 2 export/SM_VoxelPine_Tree02.glb
blender --background --python voxel_pine.py -- 3 export/SM_VoxelPine_Tree03.glb
blender --background --python voxel_pine.py -- 4 export/SM_VoxelPine_Tree04.glb
```

The script prints one `FIGURES` line, then checks it and exits non-zero if any
check fails. Same arguments give a byte-identical GLB (verified with SHA-256).

## Variants

| Type | Cells | Sheet height | Size (m, XYZ) | Tris | Verts | GLB |
|---|---|---|---|---|---|---|
| 1 | 4 | 5.32× dwarf | 5.0 × 6.4 × 5.0 | 4,366 | 8,732 | 300 KB |
| 2 | 5 | 6.67× dwarf | 5.0 × 8.0 × 5.4 | 5,894 | 11,788 | 404 KB |
| 3 | 5 | 6.67× dwarf | 3.8 × 8.0 × 3.4 | 3,474 | 6,948 | 239 KB |
| 4 | 6 | 8.80× dwarf | 4.6 × 10.6 × 4.6 | 5,280 | 10,560 | 363 KB |

Heights follow the sheet's `N × dwarf height` labels with the dwarf anchored at
**1.20 m** and a **0.2 m** voxel. If the dwarf is a different height, rescale
with `--voxel`; every dimension is derived from it.

Type 4 is drawn on the sheet as three trunks. It is modelled here as a single
tall 6-cell tree — one asset per file. A three-trunk grove would be a separate
prop that instances this one.

## Trunk proportions

The trunk column and the root plate are sized independently, on purpose: a
**wide plate under a slender column** is the proportion the sheet reads with.

`trunk_r=(base, tip)` is a float radius on a radial cross-section, so thickness
tunes in sub-voxel steps instead of jumping 3×3 → 5×5:

| radius | cells | reads as |
|---|---|---|
| ≤ 1.0 | 5 | plus/cross, 3 wide with notched corners |
| ≤ 1.5 | 9 | full 3×3 |
| ≤ 2.0 | 13 | 13-cell cross, 5 wide at the cardinals |
| ≤ 2.3 | 21 | rounded 5×5 |
| ≤ 2.9 | 25 | full 5×5 |

Tuned values give every type a solid **3-voxel (0.6 m) column** through the
visible run, with only the bottom row or two widening to blend into the plate.
Types 3 and 4 carry the long exposed trunks, so they get slightly more base.
Values below ~1.2 taper to the plus section too early and the trunk looks
pinched; 5×5 throughout (the first pass) reads as a stone pillar.

`flare=(z0, z1)` sizes the root plate and is left at its original width.

## Per-asset properties

- One mesh, one material, one primitive → **1 draw call**.
- Origin at the trunk base centre, sitting on `y = 0`; bbox centred on X and Z
  to 0.000000 so instances do not lean.
- Transforms applied, +Y up, metres.
- Flat shaded, single-sided (`doubleSided` absent → false).
- No glTF extensions — plain metallic-roughness, `metallic 0`, `roughness 0.92`.

## Palette

A single 64×64 PNG atlas embedded in the GLB, 4×4 grid of 16 px cells, UVs
inset to each cell centre. Exported with `magFilter NEAREST` — **do not let the
importer switch it to linear or force mips-only sampling**, or cells bleed.

| Cell | Hex | Role |
|---|---|---|
| 0 | `#4A3B2E` | Trunk Brown (sheet) |
| 1 | `#6B5B49` | Wood Trunk (sheet) |
| 2 | `#2A3E34` | Needle Green, shaded |
| 3 | `#364D3F` | Needle Green (sheet) |
| 4 | `#52715B` | Needle Green, lit |
| 5 | `#FFFFFF` | Snow (sheet) |
| 6 | `#D8E4EC` | Snow, shaded |

Texels are byte-exact to these values; the script decodes the PNG back out of
the finished GLB and fails if any cell differs. That check exists because two
separate colour bugs shipped silently before it did:

- `Image.pack()` on a `GENERATED` image re-encodes the generated source and
  discards whatever was assigned to `.pixels`; the exporter then copies those
  black packed bytes into the GLB. The script now encodes the PNG itself and
  packs the exact bytes.
- `bpy.data.images.new()` returns a **byte** image, so `.pixels` is
  display-referred. Linearising the hex before writing it there bakes a second
  sRGB decode into the texture and ships a visibly too-dark tree.

## Mesh topology — read before running any adjacency tool

The mesh is a **greedy-meshed voxel hull, deliberately left unwelded**. Every
quad carries its own four vertices and **no vertex is shared between quads**
(`verts == quads × 4` exactly, asserted on every build). This is correct output
for a voxel mesher — merging co-planar same-colour faces is what keeps the
triangle count roughly half of naïve per-face meshing — but it means the asset
**does not support**:

- **Smooth or averaged vertex normals.** There is no adjacency to average
  across, so smoothing does nothing (or splits into visible facets). The asset
  is flat-shaded by design; keep it that way.
- **Subdivision surfaces.**
- **Adjacency-based auto-LOD or decimation.** Anything that needs a connected
  manifold — edge-collapse decimators, most auto-LOD tools — will either refuse
  the mesh or shred it. Generate LODs by re-running the script at a coarser
  `--voxel` instead.
- **Collision generation that walks shared edges.** Convex decomposition and
  box/voxel colliders are fine. Prefer a separate primitive collider: a capsule
  on the trunk plus a cone or box on the canopy.

Greedy merging also leaves **T-junctions** where a large quad abuts smaller
ones (single-use edges in the thousands). Harmless for rendering here, but it
is the other reason not to run adjacency algorithms over this mesh.

Because the mesh is a quad soup, Euler characteristic and edge-manifold tests
are meaningless on it. The correct closure oracle is the **signed volume**,
which must equal `voxel_count × voxel_size³`; it is only satisfied when every
exposed face is present, none is duplicated, and all normals face outward. The
script asserts this on every build.

## What the script checks

Printed as one `FIGURES` line, then asserted; any failure exits 1.

- bbox centre X and Z are 0, bbox min Y is 0, height matches the variant
- signed volume equals the voxel volume
- `tris == quads × 2`, `verts == quads × 4`
- exactly 1 material, 1 primitive, 1 embedded image, no glTF extensions
- material single-sided; all UVs inside 0–1
- every palette cell decodes to its sheet hex; `magFilter` is `NEAREST`

## Repo state

- `voxel_pine.py` — the generator; the only source of truth for these assets.
- `export/*.glb` — the four current builds.
- `trees.blend` — working file holding the current set.
- `tree.blend` / `tree.blend1` — **stale.** They hold the interactive first
  pass, with the off-centre canopy, the too-dark texture and the thick 5×5
  trunks, and are not used by the pipeline. `tree.blend` also overwrote the
  file that was in the project before this work, and `tree.blend1` is a backup
  of that same session rather than the original. Safe to delete.

Note that `trees.blend` was saved before the trunk pass, so its meshes are one
revision behind `export/`. Re-import the GLBs (or regenerate) before building
on it.
