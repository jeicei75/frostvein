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
prompts/     briefs for MCP/agent modelling sessions, and their session records
references/  the approved reference sheets, the mood art and dwarf.mp4
renders/     the committed turnarounds, the zoom strip and the reference comparisons
candidates/  authored GLBs that have NOT been promoted to assets/ -- see the dwarf below
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

**Revision r3 (2026-09-11) is the current authored round, and it is a CANDIDATE.** It is
rigged, so it is not the same shape of asset r2 shipped; the session record is
`prompts/dwarf-miner-round-3-report.md` and it is the thing to read before touching any of
this. What follows describes r3. `assets/gltf/SM_VoxelDwarf_Miner01.glb` still holds r2.

```
blender --background --python blender/dwarf_miner.py -- <out.glb>
        [--voxel M] [--pose neutral|swing] [--beard NAME] [--hair NAME] [--blend P]
```

It draws **no random numbers at all** — every voxel is placed by an explicit rule — so there
is no `--seed` and determinism is by construction rather than by discipline.

### Candidates, and the naming scheme (ruled by Wolf, round 3 §C)

**The file path in `assets/gltf/` is a SLOT and does not churn.** `SM_VoxelDwarf_Miner01.glb`
is what the client loads (`DWARF_SCENE_PATH`), and `01` denotes the ROLE — miner, as opposed
to a future smith — never a version. It is never bumped to `Miner02` to mean "second attempt".

**Internal names carry the revision, because they are what a stale binary shows about
itself:** node and mesh `SM_VoxelDwarf_Miner01_r3`, material `M_VoxelDwarf_r3`, image
`T_VoxelDwarf_Palette_r3`. The generator declares the same token as `REVISION` and prints it
on the `FIGURES` line, so source and binary can be compared without opening Blender.

**Candidates never enter `assets/gltf/`.** They live in `candidates/` and carry the revision
in the basename too, so basename, mesh and node still agree and `check_asset.py`'s naming
clause holds. Promotion is a copy plus the revision bump — and see the report §5 for what the
checker will say about the copy, because that clause has **not** caught up with this scheme.

```
blender --background --python blender/dwarf_miner.py -- ../src-assets/candidates/SM_VoxelDwarf_Miner01_r3.glb
blender --background --python blender/dwarf_miner.py -- ../src-assets/candidates/SM_VoxelDwarf_Miner01_r3_swing.glb
```

The basename is refused unless it names a pose of this revision: a posed GLB and a neutral one
are different assets and must not share a name. `--pose` must agree with the basename.

**Verified 2026-09-11** under Blender 5.2.1: two independent cold runs (with
`blender/__pycache__` removed between them) and the committed files are byte-identical.

| GLB | sha256 (first 16) | bytes | Size (m, XYZ) | Voxels | Tris | Verts |
|---|---|---|---|---|---|---|
| `candidates/SM_VoxelDwarf_Miner01_r3.glb` | `dd34c1b3154a4cba` | 1,399,040 | 1.25 × 1.20 × 0.80 | 150,340 | 11,058 | 22,116 |
| `candidates/SM_VoxelDwarf_Miner01_r3_swing.glb` | `1c3a5a35242b41ea` | 1,400,936 | same | same | same | same |

The two poses share one mesh: a pose is node transforms, so the `POSITION` accessor is the
rest mesh in both and both satisfy the grid, centring and volume clauses identically.

### The skeleton, and the one topology rule

Nineteen joints, rigid weights, **one mesh** — Wolf's ruling, round 3 §A. The names are the
contract between this asset and every future one, because one animation clip has to drive
every variant:

```
root  hips  spine  chest  neck  head  beard
shoulder.L/R  elbow.L/R  hand.L/R
hip.L/R  knee.L/R  foot.L/R
```

**THE RULE: no quad may cross a joint.** Greedy-merge within a part, never across one, so
every quad's four vertices belong to exactly one joint. A quad spanning shoulder to wrist
cannot bend — it can only shear into a parallelogram. It holds by construction (the mask a
greedy run merges over is keyed by `(colour, joint)`) and is checked anyway against the
written GLB, along with rigid weights and the joint-name set.

**Weights are rigid: every vertex 1.0 to one joint, no blending.** That is not a downgrade
from soft skinning, it is what the reference does; soft weights on cubes smear the voxel read.
The trade, stated because it is a decision: greedy meshing removes the interior vertices soft
deformation needs, so choosing greedy quads and rigid weights is choosing rigid animation. A
part that ever wants genuine soft motion needs welded, denser topology and will stop reading
as voxels.

**Do not weld this mesh.** Measured on r2's shipped GLB: of 28,796 vertices, the number
sharing position AND normal AND UV with another is **zero** — greedy meshing already
performed every legal weld. Welding by position alone averages normals across hard voxel
edges (the voxel read is the look), breaks the per-cell UVs, and makes rigid joints
impossible, because one welded vertex can follow only one bone and drags the other part with
it.

### Palette: 23 cells, 8×8 grid

The atlas is still the contracted **64×64** image but is now an **8×8 grid of 8 px cells**,
because 4×4 holds sixteen and this palette needs twenty-three. **`check_asset.py` samples the
4×4 grid and therefore mis-reads this atlas — it reports 4 cells of 23 and exits 0.** That is
a real, reproducible defect in a clause that has not caught up; see the report §5. Do not
"fix" it by rearranging the atlas.

The ten ruled hexes are carried verbatim from r2 with their provenance unchanged. The thirteen
new ones are **value steps computed from them by a stated factor** — `step()` scales a hex,
keeping hue exactly, and raises rather than clamping; `tint()` mixes toward white and is used
once, where a scale would clamp. They are derivations, not measurements, and the source says
so.

| Cell | Hex | Role | Cell | Hex | Role |
|---|---|---|---|---|---|
| 0 | `#E9D2BB` | Skin | 12 | `#CBD6CE` | Metal, highlight |
| 1 | `#BAA896` | Skin, in shadow | 13 | `#707572` | Metal, underside |
| 2 | `#5E4632` | Beard, mid | 14 | `#8B6B50` | Wood |
| 3 | `#826145` | Beard, lit lock | 15 | `#AF8765` | Wood, lit face |
| 4 | `#423123` | Beard, outline lock | 16 | `#6B5B49` | Leather, mid |
| 5 | `#FFFFFF` | Snow / eye white | 17 | `#8F7A62` | Leather, cuff and top edge |
| 6 | `#5F7A6A` | Tunic, mid field | 18 | `#493E32` | Leather, sole and shadow |
| 7 | `#7DA18C` | Tunic, hem and lit band | 19 | `#34271C` | Hair / dark iron |
| 8 | `#44584C` | Tunic, fold and sleeve | 20 | `#513C2B` | Hair, crown |
| 9 | `#474B41` | Trouser / lantern iron | 21 | `#F0A63C` | Lantern flame |
| 10 | `#63695B` | Trouser, lit | 22 | `#F7CE94` | Lantern flame, hot core |
| 11 | `#A9B2AC` | Metal | | | |

`Pants` and `Wood` remain **carried from the brief and not confirmed** — `reference-sheet.jpg`
is 1024×558 at full resolution and its `8` and `B` render as the same glyph. `Hair` and
`Flame` were derived in r2, not sampled clean.

### The lantern flame is EMISSIVE, and nothing else is

Wolf ruled it in for round 3 (§B). glTF emission is `emissiveFactor × emissiveTexture`, so a
per-texel mask has to be black where nothing glows — which the base-colour atlas is not. The
mask is therefore a **second UV set** (`TEXCOORD_1`) into the *same* image: the pane quads
sample the flame cells, everything else samples cell 63, which is left black. One mesh, one
material, one image all survive. Emission strength is held at **exactly 1.0**, because
anything else makes the exporter write `KHR_materials_emissive_strength` and the asset would
carry an extension.

Costs 176,928 bytes, the whole second UV accessor. The rig costs 442,320 more (`JOINTS_0` +
`WEIGHTS_0`), which is why the file grew to 1.37 MiB while the triangle count *fell* to
11,058 — removing the grooves let big co-planar runs merge again.

**Making the lantern LIGHT THE SCENE is a point light in the client and is not art work.**

### Sockets and the variation scheme

`SOCKETS` declares four anchor voxel coordinates and the four swappable parts are authored
against them and nothing else: `beard` (0, 4, 74), `hair` (0, 0, 63), `hand.R` (31, 2, 33),
`hand.L` (−31, 2, 33). The pickaxe hangs on `hand.R` and the lantern on `hand.L`, each
weighted entirely to that hand, so splitting them into their own assets later is a file move.

`BEARDS` and `HAIRS` are registries of builder functions keyed by name (`--beard`, `--hair`).
One entry each; a second is a function with the same signature and a new key.

**Removability is checked, not claimed.** `build_voxels()` takes `beard=None` / `hair=None`,
and every build runs the body twice more — once without each part — and requires that no
voxel outside the part goes missing or changes colour. The skull is a complete skin head to
z 88; the hair is placed only into air, never over skin; the beard's moustache and sideburns
are painted proud of the skull and do not replace it. That check found three real defects in
this round, listed in the report §6.

The remaining groups (torso, legs, belt, pack) are still absolute. They are not swap axes yet.

### Rendering

```
blender --background --python blender/render_dwarf.py -- ../src-assets/renders
        [--engine workbench|cycles] [--reference FRAME.png]
```

Builds the model by importing `dwarf_miner` — including the skeleton and the pose — so the
renders cannot drift from what the generator ships. Writes five views × (flat, lit) neutral,
five lit in the swing pose, and the **zoom strip**.

**The zoom strip, `dwarf-zoom-strip.png`, is a round-3 deliverable and not a convenience.**
The lit front view at 10, 30, 100 and 700 px, nearest-neighbour, each blown back up by an
integer factor with the true-size frame inset. 10 px is the wide end measured through the
client's own projection oracle — a 1.20 m dwarf draws 8.74 px at the shipped boot framing —
and full height is the marketing shot. With free zoom the camera passes through everything
between, so **a detail that dissolves into speckle at 10 px is a defect, not a lost luxury.**

`--reference` takes a still, because nothing in the script can decode a video. The frame the
report compares against:

```
ffmpeg -ss 9 -i references/dwarf.mp4 -frames:v 1 -vf "crop=340:560:400:120" dwarf-mp4-t9.png
```

`-ss 10` returns nothing: the file is 10.0 s at 24 fps, so there is no frame at t = 10.

The flat pass proves itself every run — `FLAT-CHECK all 23 palette colours reach the PNG
exactly`. Verified under **Workbench** on the art seat 2026-09-11, which is what round 2 left
owed from this side.

### What r3 changed about the body, and one flag for Wolf

The groove passes are **gone** (`groove()` deleted, and a build-time check scans for any
one-voxel channel repeating at a fixed stride). Detail moved into the outline: beard locks, a
stepped crown, a stepped hairline, a stepped tunic hem, stepped boot cuffs, and a buckle that
is a plate standing proud rather than a painted face.

**The mid-body bands MOVED, and they are bands these notes previously recorded as measured
from the approved reference sheet.** Measured off `dwarf.mp4` at t = 9 s against its own
~10.3 px voxel pitch: the reference's belt centres at z 40 and its tunic hem lands at z 29.
r2 had the belt at 20–26 and the hem at 19, so the belt sat *on* the hem and no trouser showed
at all. The sheet and the video disagree here and round 3 resolved it to the video, on the
brief's statement that the video is that round's authority. The boots already agreed and did
not move. Recorded because it is a ratified number being changed — see the report §7A.

Height is **held at 96** and cannot usefully move: the grid clause needs `1.20 / N` to be a
multiple of 0.0125 m, so N must divide 96. The reference's ~60–65 is off the grid entirely,
and 48 loses the eyes, the brow ledge and the nose.


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
- every palette cell decodes to its declared hex; `magFilter` is `NEAREST`; sampler wrap is
  `CLAMP_TO_EDGE`
- **the dwarf, from r3, additionally:** exactly one skin whose joint names are the standing
  list; every vertex weighted 1.0 to a single joint; **no quad crossing a joint**, checked
  against the written GLB rather than trusted from the mesher; an `emissiveTexture` on UV
  set 1 with `emissiveFactor [1,1,1]` covering the lantern panes and nothing else; every
  internal name carrying the revision token; basename, mesh and node agreeing on the pose;
  no one-voxel channel repeating at a fixed stride anywhere on the body; and the beard and
  the hair each removable without changing a voxel that is not theirs
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
- `blender/dwarf_miner.py` — the dwarf generator, **r3**. Still imports `voxel_pine` and
  still **does not edit it**, but owns two things it used to borrow: the **mesher**, because
  greedy runs must break at a joint boundary and because the dwarf needs a second UV set and
  an 8 px cell grid; and the **exporter**, because it must select the armature, pass
  `export_skins`, and pass `export_rest_position_armature=False` for a posed GLB. The
  material scaffolding, the GLB reader, the PNG decoder and the volume oracle are still the
  pine's.
- `blender/render_dwarf.py` — the turnaround, both poses, and the zoom strip. Builds by
  importing the generator, so a render cannot drift from what ships.
- `candidates/SM_VoxelDwarf_Miner01_r3.glb`, `candidates/SM_VoxelDwarf_Miner01_r3_swing.glb`
  — r3's deliverable. **Not promoted**; `assets/gltf/` still holds r2.
- `prompts/dwarf-miner-round-3-report.md` — r3's session record: figures, the byte-identical
  proof, what `check_asset.py` actually said, and the flags Wolf is owed a ruling on.
- `blender/dwarf.blend` — the dwarf's editable source, saved by the generator's `--blend`.
  Regenerated, never hand-edited: the `.glb` is the deliverable and the script is the
  durable record.
- `prompts/dwarf-miner-blender-mcp-report.md` — the v1 session record: figures, reproduction
  proof, the checker's verdict and the three findings. Superseded on the asset, kept for the
  provenance of the palette.
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
