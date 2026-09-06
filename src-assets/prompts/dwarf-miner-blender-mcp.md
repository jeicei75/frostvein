# Modelling brief — the dwarf miner, via Blender MCP

**Version 2, 2026-09-06.** Version 1 was executed and delivered a working, reproducible,
contract-passing asset that **did not read as a dwarf**. The cause was not the modelling: v1's
budget was twelve voxels, and at twelve voxels a face is three voxels wide. This version raises
the budget eightfold and rules every decision v1 left open.

**For:** a Claude Code session with the Blender MCP server attached, Blender **5.2.1**.
**Produces:** story 10.5 Part B's first input — the authored dwarf.

---

## Start from scratch

`src-assets/blender/dwarf_miner.py` exists and works. **Do not extend it.** Its 463 lines place
239 voxels by explicit rule at a 12-voxel height; at 96 voxels not one of those placements
survives, and subdividing v1's blocks would produce a smoother version of the thing that did not
read.

**Read it for its machinery and reuse that**, which is the genuinely good half:

- how it imports `voxel_pine.py`'s greedy mesher without editing it
- the **even-width fix**: the pine mesher subtracts half a voxel in X and Y so an odd-width trunk
  centres on the origin. That shift is off-grid at any voxel size finer than the pine's, so the
  dwarf is even-width in X and Z and the shift is undone after meshing. **This still applies.**
- the atlas builder, the GLB read-back, the `FIGURES` line, and the argument guards

Model the geometry from the reference, not from v1.

## Where you may write — `src-assets/` and nowhere else

**Ruled by Wolf, 2026-09-06. This session writes nothing outside `src-assets/`.**

| | |
|---|---|
| the generator | `src-assets/blender/dwarf_miner.py` (replace it) |
| the editable source | `src-assets/blender/dwarf.blend` |
| the exported `.glb` | `src-assets/export/SM_VoxelDwarf_Miner01.glb` |
| your renders | `src-assets/renders/` |
| your report | `src-assets/prompts/dwarf-miner-blender-mcp-report.md` |

`src-assets/export/` is **gitignored scratch**, deliberately: the asset contract puts the runtime
glTF at `assets/gltf/<published-name>.glb`, and **promoting it there is a separate act taken from
the forge side — not yours.** Two committed copies of one asset is how a bench and a client come
to draw different trees.

**Why this rule exists**, because the alternative sounds harmless: v1 wrote its `.glb` to
`assets/gltf/` — correct per the contract, outside `src-assets/` in practice. The commit was made
from inside `src-assets/`, so `git add .` could not reach it and the `-a` that swept up an
unrelated tracked deletion does not stage untracked files. The asset was reported as delivered
and was not in the repo. Nothing was ignored and nothing errored.

**Before claiming anything is committed:** run `git status` from the **repository root**, not
from your working directory. A scoped `status` reports a clean tree while an untracked
deliverable sits one level up.

## Reference — use these, and only these

In `src-assets/references/`:

- **`dwarf-contact-sheet.jpg`** — 24 rendered frames of the intended result in situ. **This is
  now your primary source**, ahead of the line sheet, because it shows the dwarf as geometry:
  stepped hair, a brow ledge, eyes with whites, a beard distinct in value from the hair, a
  lantern with a cage and warm panes.
- **`reference-sheet.jpg`, Section A** — front, both side and back orthographic views, the gear
  breakdown, and the palette. Authoritative for **silhouette, gear and palette**.
- **`dwarf-animation-reference.jpg`** — the 5-pose mining-strike cycle. Reference only; you are
  not rigging. It tells you which joints later rotate rigidly, which is why the model must not
  weld across them.

**Do NOT model from `17d7215b-….jpg` or `a9d4e72b-….jpg`** — mood/vista art, lit and graded, not
orthographic.

**Do not try to measure the voxel count off any of them.** All three are too small to resolve it:
the line sheet is 1024 px, each contact-sheet frame is 320×180, and the dwarf is ~93 px tall in
the mood art. Measurements taken off them bound the count from **below** only, and reading them
as a target is how v1's budget ended up where it did. **The count is given below. Use it.**

## The dimensions — ruled, not negotiable

| | |
|---|---|
| Height | **96 voxels = 1.20 m** |
| Voxel | **0.0125 m** (1.20 / 96) |
| Up axis | **+Y**, facing **−Z** (Bevy's forward) |
| Units | **metres**, all transforms applied |
| Origin | foot base at **`min Y = 0`**; bounding box centred on X and Z to **±0.000001** |
| Width | **even** in X and Z — see the even-width fix above |

**He is 1.20 m tall and that has not changed.** Only the voxels got smaller. v1 was 12 voxels of
0.1 m; you have 96 voxels of 0.0125 m in the *same* silhouette. `--voxel` rescales the model, it
does not resample it — the count is authored, not a flag.

### Why 96, so you can spend it correctly

Detail is **voxels across the object**, not metres per voxel. Stated that way the shipped world is:

| | voxels tall | voxel | metres tall |
|---|---|---|---|
| pine Tree04R | 48 | 0.2 m | 9.6 m |
| pine Tree02 / Tree03 | 40 | 0.2 m | 8.0 m |
| **dwarf, v1** | **12** | 0.1 m | 1.20 m |
| **dwarf, you** | **96** | 0.0125 m | 1.20 m |

v1's dwarf had a quarter of a pine's silhouette resolution and a twentieth of its geometry —
background scenery twenty times more detailed than the character the game is about. At 96 he is
twice a pine's resolution, which is right: a pine is a cone on a column and almost all its shape
is silhouette, while a dwarf carries a face, hands, gear and an asymmetric pose in the same space.

**Expect 14,000–30,000 triangles.** That is not a limit to stay under, it is what the budget
implies — do not economise into austerity. The world holds **five** dwarves against 265 pines, so
even the top of that range is ~13% of the forest's ~1.19 M triangles. Cost is not your constraint.

## What v1 could not do, and you must

v1 correctly reported these as unavailable at 12 voxels. At 96 they are required, and the contact
sheet shows all of them:

- **Eyes**, with whites and a darker pupil, set apart with a bridge between them.
- **A brow ledge** above them, stepped proud of the face.
- **A nose**.
- **A beard separated from the hair by value, not only by shape.** In v1 both used `#5E4632`, so
  the entire head read as one brown mass with a thin skin band across it. This is the single
  biggest reason v1 does not read as a face.
- **A lantern that reads as a lantern** — a cage with warm panes. v1's was `Metal` + `Snow` and
  reads as a pale brick, because there is no flame colour in the palette. There is now.
- **A pickaxe that reads as a pickaxe** in profile *and* head-on. v1's reads as a bar with a cap.

## Palette

The eight from the sheet, plus two new cells this asset needs. The atlas is a 4×4 grid of sixteen
cells, so there is room; eight were used, ten now are.

| Cell | Role | Hex | Note |
|---|---|---|---|
| 0 | Skin | `#E9D2BB` | |
| 1 | Beard | `#5E4632` | |
| 2 | Snow / eye white | `#FFFFFF` | **confirmed** — same value in the shipped pine atlas |
| 3 | Tunic | `#5F7A6A` | |
| 4 | Pants | `#474B41` | label ambiguous (`474841`/`474B41`); carried, unconfirmed |
| 5 | Metal | `#A9B2AC` | |
| 6 | Wood | `#8B6B50` | label ambiguous (`8B6850`/`8B6B50`); carried, unconfirmed |
| 7 | Wood Trunk | `#6B5B49` | **confirmed** — same value in the shipped pine atlas |
| **8** | **Hair** | **you choose** | darker than Beard. Sample the contact-sheet head; do not invent it. |
| **9** | **Lantern flame** | **you choose** | a warm amber. Sample the contact-sheet lantern panes. |

The sheet's `B` and `8` are the same glyph at 1024 px — provably, because `Wood Trunk` reads
`685849` there while the pines demonstrably ship `#6B5B49`. Cells 4 and 6 could not be resolved
by sampling either; the residuals do not clear the noise floor. They ship as-is. **Only the
original art file settles them.**

**`check_asset.py` reads only SEVEN palette cells** — `PALETTE_HEX` is hardcoded to the pine's
seven and it iterates `range(len(PALETTE_HEX))`. Cells 8 and 9 are **not mechanically checked by
anything but your own generator.** Do not read its acceptance as validation of them; say so in
your report.

### The flame is a colour, never an emitter

A pixel guard asserts that with all five light sources off, **zero** pixels in the frame satisfy
`R − B > 30`. Four palette colours already exceed that as raw albedo — `wood` +59, `skin` +46,
`beard` +44, `trunk` +34 — and the guard passes, because with nothing lighting the scene every
surface renders near-black. **So a bright amber albedo is safe. An emissive material is not**, at
any colour: it is a light nobody can switch off, and it has been found from the seat once already.

### Two colour bugs that shipped silently on the pines

- `Image.pack()` on a `GENERATED` image re-encodes the generated source and discards whatever you
  assigned to `.pixels`; the exporter then writes those bytes into the GLB. Encode the PNG
  yourself and pack the exact bytes.
- `bpy.data.images.new()` returns a **byte** image, so `.pixels` is display-referred. Linearising
  the hex before writing there bakes a second sRGB decode in and ships a visibly too-dark asset.

Verify by decoding the PNG back out of the **finished GLB**, not out of the Blender datablock.

## The asset contract — pass it unchanged

`scripts/bench/check_asset.py` enforces a "V1 voxel asset" shape. Hold every clause.

**The grid clause was amended for you on 2026-09-06** — `PROJECT_GRID_METRES` moved 0.1 → 0.0125,
because no 96-voxel 1.20 m dwarf can land on a 0.1 m grid. **Pull before you start.** With that
change the 96-voxel asset passes the contract cleanly and unchanged on your side, so a grid-clause
rejection is now a real defect in your geometry, not an expected limitation.

- **One mesh, one material, one primitive** — one draw call.
- **One embedded PNG atlas of exactly 64×64 texels**, 4×4 grid of 16 px cells, UVs inset to cell
  centres.
- `magFilter = NEAREST`, `wrapS`/`wrapT` = `CLAMP_TO_EDGE`, every `TEXCOORD_0` inside 0–1.
- Flat-shaded, **single-sided**, metallic-roughness, `metallic 0`, roughness ~0.92.
- **Neither `extensionsUsed` nor `extensionsRequired`.**
- **Greedy-meshed, unwelded quad soup**: `tris == quads × 2` and `verts == quads × 4`, exactly.
- **Naming:** file basename, glTF mesh name and glTF node name are all **`SM_VoxelDwarf_Miner01`**.

If a clause proves genuinely impossible, **report it and stop.** Do not break it quietly and do
not "fix" a conforming asset to satisfy a pine rule.

## Do not

- **Do not make anything emissive**, the lantern included. See above.
- **Do not rig, skin or animate.** No armature, no shape keys, no actions in the GLB.
- **Do not smooth, subdivide, decimate or auto-LOD.** The mesh has no shared adjacency.
- **Do not add a collider, camera, light or empty** to the exported scene.
- **Do not colour-match the mood art.** Those frames are lit and graded.
- **Do not edit `voxel_pine.py`.** Import it.

## The deliverable is a SCRIPT, not a session

> An asset ships three things: the editable source, the exported glTF, and a **standalone
> headless generator script** that reproduces it. **The script is the durable record; the session
> is not.**

This clause paid for itself on v1: the `.glb` never reached the repo, and the asset was
regenerated byte-for-byte from the committed script on a different operating system.

### How to work — the script IS the live session, not a write-up of it

**Do not build the model by hand in Blender and write the generator afterwards**, and do not go
the other way either and develop the script blind against headless renders. Both lose something:
the first makes the script a reconstruction that may not reproduce what you actually shaped, and
the second means nobody — you or Wolf — sees the dwarf until it is finished.

Work like this instead:

1. Edit `dwarf_miner.py`.
2. **Execute it inside the running Blender instance** through MCP, so the result appears in the
   viewport. The generator opens with `wipe_scene()`, so re-running it is always a clean rebuild
   and is the correct way to iterate.
3. Look at the viewport. Change the script. Go to 2.

The script is then the source at every moment, never a reconstruction, and **Wolf can watch the
dwarf take shape** — which is the point of doing this in a visible Blender rather than headless.
When something looks wrong, the fix is a change to the script, never a hand edit to the mesh; a
hand edit is a change that the script does not contain and that the next run will silently discard.

The cold headless run stays the proof, because a GUI session carries context and add-on state that
a `--background` run does not. So the session is finished only when

```
blender --background --python src-assets/blender/dwarf_miner.py -- src-assets/export/SM_VoxelDwarf_Miner01.glb
```

writes a byte-identical GLB from a cold Blender, with no MCP and no hand steps. Run it twice into
two paths and compare SHA-256.

### Self-verification the script must carry

One `FIGURES` line, then assert it and exit non-zero on any failure. **Read every figure back out
of the GLB you just wrote**, never off the spec you asked for.

- bbox centre X and Z are 0, `min Y` is 0, height is 1.20 m, and the model is **96 voxels tall**
- `tris == quads × 2`, `verts == quads × 4`
- exactly 1 material, 1 primitive, 1 embedded image, no glTF extensions
- material single-sided; all UVs inside 0–1
- every palette cell decodes to its hex — **all ten, including the two the checker cannot see**
- `magFilter` is `NEAREST`

Two guards that are not optional, both kept from v1, which had them right:

- **`--voxel` must be > 0.** Zero collapses the mesh to a point *and* scales the expected height
  and volume by the same zero, so every closure check passes vacuously and the script reports OK
  on nothing. An oracle must not be scaled by the input it is checking.
- **Convert any uncaught exception to a non-zero exit.** Blender's `--background` runner prints a
  traceback and **still exits 0**, so without this a bad argument or an unwritable path reports
  success having written nothing.

v1 also added a **basename guard** — it refuses to write a GLB the checker would reject on the
naming clause. Keep it. Note it fires *before* the voxel check, so exercise the two independently.

## When you are done, report

1. The `FIGURES` line, and the SHA-256 from two independent cold runs.
2. `check_asset.py`'s verdict, verbatim, including a rejection if it rejects.
3. Renders in `src-assets/renders/`: front, both sides and back orthographic, plus one ¾ — and
   **place them beside the contact sheet**, which is the thing they have to match.
4. What you could not achieve at 96 voxels, and any contract clause you could not satisfy.
   **A limitation reported is the deliverable; a limitation worked around silently is a defect.**

## Decisions — all ruled. None of these are yours.

v1's report presented these as "ruled" having taken them itself. They are ruled now, by Wolf, on
2026-09-06, and there is nothing left here for a session to decide:

1. **Published name:** `SM_VoxelDwarf_Miner01`.
2. **Gear is part of the mesh** — bake the lantern and pickaxe in, kept as separate unwelded
   voxel groups so a later cut is a selection rather than a remodel.
3. **Height stays 1.20 m.** The anchor remains formally unratified and is a separate open
   question; it is **not** reopened by this work and you must not adjust it.
4. **Resolution is 96 voxels.**

If something here proves wrong, **say so and stop** — do not rule on it yourself.
