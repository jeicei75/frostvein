# Modelling brief — the dwarf miner, via Blender MCP

**For:** a Claude Code session with the Blender MCP server attached, Blender **5.2.1**.
**Produces:** story 10.5 Part B's first input — the authored dwarf.
**Status:** DRAFT. Three decisions at the bottom are Wolf's and are not yours to take.

Paste everything between the rules into the modelling session. It is written to be read
without this repo open, so it repeats the contract rather than citing it.

---

## What you are making

A **voxel dwarf miner** for frostvein, a Rust voxel-colony sim. He is one entity in a
snowy valley seen from an isometric camera, standing about a fifth as tall as the pine
trees beside him. He carries a lantern and a pickaxe.

## Where you may write — `src-assets/` and nowhere else

**Ruled by Wolf, 2026-09-06. This session writes nothing outside `src-assets/`.**

| | |
|---|---|
| the generator | `src-assets/blender/<name>.py` |
| the editable source | `src-assets/blender/<name>.blend` |
| the exported `.glb` | `src-assets/export/<published-name>.glb` |
| your report | `src-assets/prompts/<brief-name>-report.md` |

`src-assets/export/` is **gitignored scratch**, which is deliberate: the asset contract puts
the runtime glTF at `assets/gltf/<published-name>.glb`, and **promoting it there is a separate,
deliberate act taken from the forge side — not yours.** Two committed copies of one asset is
how a bench and a client come to draw different trees, and this repo has already paid for that
once.

**Why this rule exists**, because the alternative sounds harmless: the previous session wrote
its `.glb` to `assets/gltf/` — correct per the contract, outside `src-assets/` in practice. The
commit was made from inside `src-assets/`, so `git add .` could not reach it, and the `-a` that
swept up an unrelated tracked deletion does not stage untracked files. The deliverable was
reported as delivered and was not in the repo. Nothing was ignored and nothing errored.

**Before you claim anything is committed:** run `git status` from the **repository root**, not
from your working directory, and read the untracked list. A scoped `status` will tell you the
tree is clean when it is not.

## Reference — use these, and only these

In `src-assets/references/`:

- **`reference-sheet.jpg`, Section A** — the approved modelling sheet. Front, both side
  and back orthographic views, two 3/4 poses, a gear breakdown for the pickaxe and the
  lantern, and the palette. **This is your primary source.**
- **`dwarf-contact-sheet.jpg`** — 24 rendered frames of the intended result in situ.
  Read the silhouette, the colour under warm light, and how the gear is carried.
- **`dwarf-animation-reference.jpg`** — the 5-pose mining-strike cycle. **Reference only.**
  You are not rigging or animating anything; it tells you which joints later need to
  rotate rigidly, which is why the model must not weld across them.

**Do NOT model from `17d7215b-….jpg` or `a9d4e72b-….jpg`.** Those are mood/vista art —
atmospheric shots of the valley and the fortress. They are not orthographic, they are
lit and post-processed, and nothing may be measured or colour-picked off them.

**Where the sheet and the dimensions below disagree, the dimensions win.** The sheet is
authoritative for silhouette, gear and palette. Its numeric labels are known to be
unreliable: Section B's second row still reads `X.x dwarf height` (an unfilled
placeholder), Section B's Type 4 labels contradict each other outright, and Section A
labels the dwarf's own height `0.6x dwarf height`, which is circular. Do not derive a
dimension from a label on that image.

## The dimensions — non-negotiable

| | |
|---|---|
| Voxel | **0.1 m** — the project grid, and for the dwarf exactly 1× it. Not finer. |
| Height | **12 voxels = 1.20 m** — one simulation cell is 1.6 m, so the dwarf is 0.75 cells |
| Up axis | **+Y** |
| Units | **metres**, all transforms applied |
| Origin | foot base at **`min Y = 0`**; bounding box centred on X and Z to **±0.000001** |

Every voxel centre lands on the 0.1 m lattice. The 1.20 m figure is the resolution
contract's, not the sheet's.

**12 voxels is a hard budget and it is tight.** Head, beard, torso, legs, lantern and
pickaxe all have to read inside it. Spend the budget on what the contact sheet makes
recognisable — the beard mass, the wide torso, the short legs, the lantern's warm cell —
and let detail go. **If the silhouette genuinely cannot be read at 12 voxels, stop and
report that as a finding.** It is a ruling for Wolf (raise the dwarf height, or accept a
coarser read); it is not licence to model finer than 0.1 m.

## The asset contract — pass it unchanged

The repo's checker is `scripts/bench/check_asset.py`. It was written for the pines and
enforces a "V1 voxel asset" shape. **Target that shape exactly**, because an asset that
passes it needs no code change to ship:

- **One mesh, one material, one primitive** — one draw call.
- **One embedded texture**: a single PNG atlas of exactly **64×64** texels, a 4×4 grid of
  16 px cells, UVs inset to each cell's centre.
- `magFilter = NEAREST`, `wrapS`/`wrapT` = `CLAMP_TO_EDGE`, every `TEXCOORD_0` inside 0–1.
- Flat-shaded, **single-sided** (`doubleSided` absent or false), metallic-roughness,
  `metallic 0`, roughness ~0.92.
- **Neither `extensionsUsed` nor `extensionsRequired`.**
- **Greedy-meshed, unwelded quad soup**: merge co-planar same-colour faces, and give every
  quad its own four vertices. `tris == quads × 2` and `verts == quads × 4`, exactly. This
  is also what keeps the mining-strike joints separable later.
- **Naming:** the file basename, the glTF mesh name and the glTF node name must all be the
  **same** string. Proposed: **`SM_VoxelDwarf_Miner01`**. Confirm the name before you build;
  it becomes a compiled-in string and renaming it later touches the client.

If some clause turns out to be genuinely impossible for a character — a second material,
say — **report it and stop.** Do not "fix" a conforming asset to satisfy a pine rule, and
do not quietly break a clause. The checker's V1 clauses are known to be tree-shaped, and
generalising them is story 10.5 Part B's job, taken as a decision with the real asset in
hand. Your finding is the input to that decision.

## Palette

From the sheet's own breakdown. Lay these into the 64×64 atlas as byte-exact texels:

The sheet prints these as text beside a swatch. The scan is 1024 px wide and JPEG-
compressed, so the label text is **not** reliable on its own: `B` and `8` render as the
same glyph at that size. Every value below was cross-checked by sampling the swatch itself
and, where possible, against the shipped pine atlas.

| Role | Hex | Confidence |
|---|---|---|
| Skin | `#E9D2BB` | label legible, swatch agrees |
| Beard | `#5E4632` | label legible, swatch agrees |
| Snow | `#FFFFFF` | **confirmed** — same value in the shipped pine atlas |
| Tunic | `#5F7A6A` | label legible, swatch agrees |
| Pants | `#474B41` | label ambiguous (`474841` / `474B41`); swatch channel order G>R>B picks `474B41` |
| Metal | `#A9B2AC` | label legible, swatch agrees |
| Wood | `#8B6B50` | label ambiguous (`8B6850` / `8B6B50`); swatch ratio favours `8B6B50` — **verify at full resolution** |
| Wood Trunk | `#6B5B49` | **confirmed** — same value in the shipped pine atlas |

Two of the eight are pinned by an asset that already ships, which is what makes the glyph
substitution above provable rather than a guess: `Wood Trunk`'s label reads `685849` on
this scan and the pines demonstrably use `#6B5B49`.

**Before you build, re-read the "GEAR & PROP BREAKDOWN" panel from a full-resolution copy
of the sheet if one exists, and correct `Wood` and `Pants` if it disagrees.** Publish the
cell-to-role map with your figures either way.

**Two colour bugs shipped silently on the pines. Both are in your path:**

- `Image.pack()` on a `GENERATED` image re-encodes the generated source and discards
  whatever you assigned to `.pixels`; the exporter then writes those bytes into the GLB.
  Encode the PNG yourself and pack the exact bytes.
- `bpy.data.images.new()` returns a **byte** image, so `.pixels` is display-referred.
  Linearising the hex before writing there bakes a second sRGB decode in and ships a
  visibly too-dark asset.

Verify by decoding the PNG back out of the **finished GLB** and comparing each cell to its
hex. Not out of the Blender datablock — out of the file you wrote.

## Do not

- **Do not make anything emissive**, the lantern flame included. The client owns lighting:
  the dwarf's lantern arrives over the wire as a point light, and a pixel guard asserts
  that with every light source switched off **zero** warm-lit pixels remain. A baked
  emissive face is a light nobody can switch off, and it has already been found from the
  seat once.
- **Do not rig, skin or animate.** No armature, no shape keys, no actions in the GLB.
- **Do not smooth, subdivide, decimate or auto-LOD.** The mesh has no shared adjacency;
  these either do nothing or shred it. Coarser LODs come from re-running at a coarser
  voxel size.
- **Do not add a collider, a camera, a light or an empty** to the exported scene.
- **Do not colour-match the mood art.** Those frames are lit and graded; the palette above
  is the unlit surface colour.

## The deliverable is a SCRIPT, not a session

This is the project's standing clause and it is the reason this brief exists:

> An asset ships three things: the editable source (`.blend`), the exported glTF, and a
> **standalone headless generator script** that reproduces it. **The script is the durable
> record; the session is not** — a live MCP session that produced an asset without leaving
> a runnable script has not delivered one.

So: use MCP to explore, judge and iterate freely — that is what it is good for — but the
session is finished only when **`src-assets/blender/dwarf_miner.py`** exists and

```
blender --background --python src-assets/blender/dwarf_miner.py -- src-assets/export/SM_VoxelDwarf_Miner01.glb
```

writes a **byte-identical** GLB from a cold Blender, with no MCP and no hand steps. Prove
it: run it twice into two paths and compare SHA-256. The pines do this and it is what let
this repo regenerate all four of them, byte-for-byte, a month after they shipped.

Model it on the sibling `src-assets/blender/voxel_pine.py`, which already contains the
greedy mesher, the atlas builder, the GLB reader and the checks. **Import it, or copy the
parts you need — do not edit it.**

### Self-verification the script must carry

Print one `FIGURES` line, then assert it and exit non-zero on any failure:

- bbox centre X and Z are 0, `min Y` is 0, height is 1.20 m
- `tris == quads × 2`, `verts == quads × 4`
- exactly 1 material, 1 primitive, 1 embedded image, no glTF extensions
- material single-sided; all UVs inside 0–1
- every palette cell decodes to its hex; `magFilter` is `NEAREST`
- **read every figure back out of the GLB you just wrote**, never off the spec you asked
  for. That is the only check that catches a generator silently ignoring a field.

Two guards that are not optional, because both were found the hard way here:

- **`--voxel` must be > 0.** Zero collapses the mesh to a point *and* scales the expected
  height and volume by the same zero, so every closure check passes vacuously and the
  script reports OK on nothing. An oracle must not be scaled by the input it is checking.
- **Convert any uncaught exception to a non-zero exit.** Blender's `--background` runner
  prints a traceback for an uncaught exception and **still exits 0**, so without this a
  bad argument or an unwritable path reports success having written nothing. Exit 0 with
  no output is not a result.

## When you are done, report

1. The `FIGURES` line, and the SHA-256 of the GLB from two independent runs.
2. `scripts/bench/check_asset.py`'s verdict on the GLB — including a rejection, verbatim,
   if it rejects it.
3. Four orthographic renders (front, both sides, back) and one 3/4, beside the sheet.
4. Anything the 12-voxel budget forced you to drop, and anything in the contract you could
   not satisfy. **A limitation reported is the deliverable; a limitation worked around
   silently is a defect.**

---

## Open, and Wolf's to decide before this is handed over

1. **The published name.** `SM_VoxelDwarf_Miner01` is a proposal that matches
   `SM_VoxelPine_TreeNN`. It becomes a compiled-in string in the client's asset table.
2. **Is the gear part of the mesh?** The sheet and the contact sheet show the lantern and
   the pickaxe carried. One mesh with the gear baked in is the simplest thing that draws
   today — the client has no attachment mechanism and the entity path draws one static
   scene. But the animation reference implies the pickaxe and the lantern eventually move
   independently of the body. Baking them in now is the YAGNI answer and costs a re-cut
   later. **Recommendation: bake them in, and keep them as separate unwelded voxel groups
   so a later cut is a selection, not a remodel.**
3. **The 1.20 m dwarf anchor is not ratified anywhere.** It reached the resolution contract
   by being copied out of story 10.2's tree notes and has never been ruled on, yet it sets
   the dwarf's height here, the client's `scale` value, and every tree's dwarf-multiple on
   the reference sheet. If it is going to move, it moves before this model is built.
