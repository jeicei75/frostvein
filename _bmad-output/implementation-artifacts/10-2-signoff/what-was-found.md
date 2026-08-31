# What was found — story 10.2's live session, 2026-08-31

One exploration session on gingerspice: **Claude Code driving Blender 5.2.1 LTS over BlenderMCP**,
building a voxel pine from the reference sheet. This note is the durable record of what came out of
it and what a later reader must not assume.

## The artifacts in this folder

| file | what it is |
| --- | --- |
| `reference-sheet.jpg` | Gemini-generated modeling reference — **AI-generated reference, not sourced or licensed art**. Proportions are ratios against dwarf height; the colour key carries the hex values the asset was built from. |
| `dwarf-animation-reference.jpg` | Five-pose mining strike cycle (25 frames, 1 s), rigid joint rotations, inertial lantern swing. Input for a later story; nothing in this one consumes it. |
| `dwarf.mp4` + `dwarf-contact-sheet.jpg` | Motion and look target. **Generated video** — the camera moves and geometry is not consistent frame to frame, so it is a feel reference, never a spec. The contact sheet is every 10th frame, because a video is the one artifact neither review nor the gate can open. |
| `session-wip-2026-08-31T1153-tree.png` | Mid-session: snow too heavy, greens near-black, tiers not yet reading. |
| `session-final-2026-08-31T1157-tree.png` | The found look, with the session's own game-ready spec table visible. |
| `tree.glb` | The exported asset, `SM_VoxelPine_Tree02`. |

## The asset, verified at the receiving end rather than trusted

Checked on the devpod with a stdlib GLB parser and a headless Blender 5.2.1 import. **Every claim
the session made holds:**

- 1 node / 1 mesh / 1 material / 1 texture / 1 image, `extensionsUsed: []`, plain metal-rough.
- `doubleSided: false`; `magFilter 9728` (NEAREST), wrap `CLAMP_TO_EDGE`.
- 10,260 verts / 5,130 tris, one primitive, one draw call.
- 5.2 x 5.4 x 7.6 m at 0.2 m voxels (38 voxels tall); `bbox.min.y == 0.000`, so it sits on the floor.
- All UVs inside 0-1 (u 0.0625-0.9375, v 0.5625-0.9375 — inset in the atlas, no mip bleed).
- Zero degenerate triangles. Imports clean here: 1 object, 1 material, 1 image.
- Palette follows the sheet: trunk `#685B49`, needle `#364D3F`, snow `#FFFFFF`, plus a darker bark,
  two extra needle tones and a cool snow shade.

## Known differences — the things a consumer would otherwise learn the hard way

1. **The trunk is half a voxel off-centre in X.** X spans -2.700..+2.500, centre **-0.100 m**, which
   is exactly -0.5 voxels; Z is centred exactly. Every tree placed from this asset leans the same
   way. It is an even/odd voxel-count asymmetry and belongs in the generator, not in a per-instance
   offset.
2. **The mesh is not manifold — it is 2,565 disconnected quads.** 2,565 x 4 = 10,260 verts, no
   vertex sharing, V-E+F = 2565. This is CORRECT for a greedy voxel mesher and must not be "fixed"
   by welding. It does mean smooth normals, subdivision, auto-LOD and adjacency-based collision
   generation will not work as-is. It is also why **signed mesh volume** (19.112 m^3, matching the
   voxel count) is the right closure oracle: a conventional manifold check would fail a perfectly
   good asset.
3. **Greedy meshing leaves T-junctions** — 3,139 single-use edges. The volume check confirms no
   actual holes, but a renderer that is strict about T-junctions may show hairline seams. Disabling
   the merge removes them and doubles the triangle count.
4. **The scale anchor was chosen in-session and is not yet a project constant.** A 1.2 m dwarf gave
   0.2 m voxels and a 7.6 m tree. Two consequences: at that voxel size the **dwarf would be only 6
   voxels tall**, which cannot carry the beard, belt, tunic panel and lantern the sheet draws; and
   the client's cell is a unit cube (`Cuboid::default()`) while `worldgen.rs` grows trees 4-6 cells,
   against this asset's ~6.3. **Decide metres-per-voxel once, globally, from the dwarf's detail
   needs, before a second asset is built.**
5. **Version pair.** Vehicle Blender 5.2.1 LTS, blender-mcp server 1.9.0 (the addon is numbered
   separately — its panel read 1.5). The devpod now also runs 5.2.1, so `.blend` travels both ways;
   glTF remains the export of record.

## The transcript — NOT retained, deliberately

The session ran in Claude Code on gingerspice and its transcript lives nowhere else. Wolf's ruling
(2026-08-31): the two screenshots above are the record, and losing the rest is accepted.

**This is a spike finding, not an accident to apologise for.** It is measured evidence that a live
session is not a durable artifact — which is precisely why the handoff has to terminate in a
committed script rather than in a transcript someone might still have.

**No by-hand viewport edits were made** (Wolf, 2026-08-31). Confirmed against the one tweak that
happened: the trunks came out too thick and were corrected **through Claude, not the viewport**
— which is why the correction survived into the generator. The bit-exact reproduction is the
independent proof, since a mouse edit could not have reached the script. So the entire
construction lived in tool calls — and handoff candidate (a) is now not merely viable but **PROVEN, bit-exactly**.

## The handoff, proven at the receiving end

`voxel_pine.py` was emitted by the session and re-run here. Four checks, all executed:

| check | result |
| --- | --- |
| runs headless on the devpod | 1.75 s per variant; reads nothing but its two arguments |
| reproduces the session's exports | **byte-identical to all four** GLBs exported on gingerspice |
| deterministic | two local runs byte-identical (SHA-256) |
| fails when it should | voxels shifted +1 in X → `FAIL: bbox centre X is +0.200000`, exit 1 |

The sabotage matters: the check that fired is exactly the defect the hand-exported `tree.glb`
shipped with (known difference 1 above). The fix is not merely applied but **guarded** —
`centre_x` is asserted at `< 1e-6`, not merely printed.

## The render pair (AC4)

`scripts/bench/spike_pine_render.py` renders each GLB headless, Cycles CPU, on a neutral studio
backdrop — deliberately NOT the client's sky, since this image answers "did the handoff keep the
look of the ASSET", not "how does it sit in the world".

| variant | tris | subject_fraction | distinct_colors | subject_luma | run a vs run b |
| --- | --- | --- | --- | --- | --- |
| Tree01 | 4,366 | 0.156535 | 10,955 | 119.576 | 0 of 921,600 pixels differ |
| Tree02 | 5,894 | 0.127873 | 11,288 | 112.625 | 0 of 921,600 pixels differ |
| Tree03 | 3,474 | 0.088167 | 10,421 | 115.656 | 0 of 921,600 pixels differ |
| Tree04 | 5,280 | 0.077327 | 10,634 | 109.969 | 0 of 921,600 pixels differ |

Failure path observed, not assumed: a malformed GLB and a missing file both exit **1**.

**Wolf judges the pair:** `render-SM_VoxelPine_Tree02.png` against
`session-final-2026-08-31T1157-tree.png`. The renders are lit differently by design, so the
question is whether the SHAPE and PALETTE survived the handoff, not whether the two frames match.

## A defect in the render instrument, found by looking at its output

The first version of `spike_pine_render.py` rendered a frame of pure backdrop — the camera pointed
away from the asset — and its range check reported **`subject_fraction=1.000000`**, i.e. "the frame
is 100% subject", because the backdrop was held as a LINEAR triple while `image.pixels` reads back
DISPLAY-referred sRGB. Only the `distinct_colors` floor caught it; a lower colour floor would have
passed an empty picture.

`valley_bench.pixel_figures` documents that exact trap in a comment, which was read and walked into
anyway. Fixed two ways: the backdrop is now held as display-referred sRGB 0-255 and converted to
linear only at the world shader, and the fraction check is now **two-sided** — an all-subject frame
means the instrument is lying, not that the render is full. Recorded because the instrument was
believed for one full run, which is the failure mode this project keeps paying for.
