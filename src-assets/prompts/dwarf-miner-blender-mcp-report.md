# Report — the dwarf miner, version 2 (96 voxels)

**Session:** Claude Code with Blender MCP, Blender 5.2.1 LTS, Windows.
**Brief:** `src-assets/prompts/dwarf-miner-blender-mcp.md` v2, 2026-09-06.
**Supersedes:** the v1 report previously at this path.

---

## 1. Figures, and two independent cold runs

```
FIGURES name=SM_VoxelDwarf_Miner01 voxel=0.0125 height_voxels=96 voxels=154557
groups=backpack:24050,belt:6422,body:112717,lantern:7628,pickaxe:3740
centring=-5,+8 quads=7199 verts=28796 tris=14398
bbox=1.2500x1.2000x0.8000 centre_x=+0.000000 centre_z=+0.000000 min_y=+0.000000
volume=0.301869 expected_volume=0.301869 materials=1 primitives=1 images=1
palette=#E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50,#6B5B49,#34271C,#F0A63C
glb_bytes=1009476
```

Two cold `blender --background` runs into two different paths, no MCP, no hand steps:

```
205493a41f64d9008fbfb18fb2e46771fa66b0410c23b6bf2dec79c450cebb5f  runA/SM_VoxelDwarf_Miner01.glb
205493a41f64d9008fbfb18fb2e46771fa66b0410c23b6bf2dec79c450cebb5f  runB/SM_VoxelDwarf_Miner01.glb
IDENTICAL
```

He is **96 voxels of 0.0125 m = 1.20 m**, bbox centred on X and Z to 0.000000, feet at
`min Y = 0`, and the signed volume equals the voxel volume to the last printed digit, which
is the closure oracle: no hole, no duplicated face, no inverted normal.

**Guards, exercised independently** (each exits non-zero and writes nothing):

| | |
|---|---|
| wrong basename, `--voxel 0` | `error: basename must be 'SM_VoxelDwarf_Miner01', got 'wrong_name.glb'` — the naming guard fires **first**, so the voxel guard below is reached only on a correct basename |
| correct basename, `--voxel 0` | `error: --voxel must be greater than 0 (got 0.0)` |
| unwritable output path | `generator failed: PermissionError: ...` — exit 1, not Blender's usual exit 0 after a traceback |

## 2. `check_asset.py`, verbatim

```
FIGURES src-assets\export\SM_VoxelDwarf_Miner01.glb size_m=1.2x1.2x0.8 min_y_m=0.000000
centre_x_m=0.000000 centre_z_m=0.000000
palette=#E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50 tris=14398 verts=28796
```

Exit 0. **No rejection.** Every clause holds and the checker needed no change from this session.

The four shipped pines and the v1 dwarf still pass the same checker unchanged, so nothing
regressed:

```
FIGURES assets\trees\SM_VoxelPine_Tree01.glb  size=5.0x6.4x5.0 tris=4366
FIGURES assets\trees\SM_VoxelPine_Tree02.glb  size=5.0x8.0x5.4 tris=5894
FIGURES assets\trees\SM_VoxelPine_Tree03.glb  size=3.8x8.0x3.4 tris=3474
FIGURES assets\trees\SM_VoxelPine_Tree04R.glb size=4.2x9.6x4.2 tris=4424
```

### The one thing the checker's verdict does NOT cover

Its `palette=` field lists **seven** cells and stops, exactly as the brief warned:
`PALETTE_HEX` is hardcoded to the pine's seven and it iterates `range(len(PALETTE_HEX))`.
So **Wood Trunk, Hair and Lantern flame — cells 7, 8 and 9 — are invisible to it**, and it
does not compare the seven it does read against anything either (`contract_data` treats the
colour-to-role map as a signoff comparison, not a clause). All ten cells are read back out of
the finished GLB and checked against the module's hex by the generator's own
`shipped_palette == PALETTE_HEX` check. **That check is the only mechanical verification
those cells get anywhere in the repo.** Do not read the checker's exit 0 as validating them.

## 3. Renders

In `src-assets/renders/`, all produced by `src-assets/blender/render_dwarf.py`, which imports
the generator and builds the model rather than loading the `.blend` — so a render can never
show geometry the generator does not ship.

| | |
|---|---|
| `dwarf-{lit,flat}-front.png` | front orthographic |
| `dwarf-{lit,flat}-side-right.png` | his right, the pickaxe side |
| `dwarf-{lit,flat}-side-left.png` | his left, the lantern side |
| `dwarf-{lit,flat}-back.png` | back orthographic |
| `dwarf-{lit,flat}-three-quarter.png` | the ¾ |
| **`dwarf-vs-contact-sheet.png`** | **all five, beside contact-sheet frame r3c5** — the comparison the brief asks for |

Two passes, because one is not honest on its own: `flat` is unlit albedo, the only way to read
the palette; `lit` is studio-lit, the only way to see that the forms are stepped at all.
Neither is a claim about how the client will light him.

## 4. What v1 could not do, and where this landed

| required | delivered |
|---|---|
| Eyes, whites and darker pupil, set apart with a bridge | 6 voxels of white each, a 4-voxel pupil sunk one voxel behind it, held apart by a 4-voxel nose bridge |
| A brow ledge stepped proud of the face | 3 voxels tall, 2 proud, and **narrower than the face** so skin carries past its ends at the temples |
| A nose | projects 4 voxels at the tip, 3 at the bridge — past the brow, and visible in profile |
| Beard separated from the hair by **value** | Beard `#5E4632` against Hair `#34271C`, a luminance ratio of 0.56 |
| A lantern that reads as a lantern | dark iron cage — four corner posts and a centre mullion — standing in front of warm `#F0A63C` panes |
| A pickaxe that reads in profile *and* head-on | small boss the haft passes through, two wings stepping out and **down** to points |

## 5. Limitations — reported, not worked around

1. **The tunic is one green where the reference has two.** The contact sheet and the front
   ortho both show a lighter yellow-green front panel over darker green sleeves and shoulders.
   The ruled palette has one Tunic `#5F7A6A` and one Pants `#474B41`, and no lighter green;
   I used Pants for the sleeves and shoulders to get a two-tone, but the *lighter* panel is
   not reproducible. Six atlas cells are free, so an eleventh colour would fit mechanically —
   **that is a palette ruling and not mine to take.**

2. **The bounding box is centred; his body is not.** The centring line reads `centring=-5,+8`:
   the lattice is translated 5 voxels in X and 8 in Y to land the box on the origin. In X that
   is the pickaxe reaching further out than the lantern; in Y it is the backpack. So his feet
   stand ~6 cm from the origin in X and ~10 cm in Y. The contract measures the box and the box
   is exact, but a placement system that assumes the origin is under his boots will see that
   offset. Cannot be removed without either unbalancing the gear or dropping it.

3. **Gear is baked in and touches the body, so a later cut is not free.** Decision 2 rules the
   lantern and pickaxe into the mesh, kept as separate voxel groups, and `part_of` tracks them
   (`groups=` on the FIGURES line). But the greedy mesher culls the faces between adjacent
   voxels regardless of group, and the fists necessarily touch their gear — so deleting a group
   from the finished mesh would leave a hole where the hand met it. **The cut is a re-run of
   the generator with the part excluded, not a selection-and-delete in the mesh.** The code is
   structured for that; the mesh is not.

4. **Cells 8 and 9 are DERIVED, not sampled clean**, and I want that on the record because
   the brief said "sample … do not invent it" and neither could be sampled straight.

   - **Hair `#34271C`.** The two sources disagree by a factor of two: the reference sheet's
     front ortho gives hair:beard = **0.87**, the contact sheet's frame r3c5 gives **0.42**.
     Both are compromised, in opposite directions — the sheet's figure is 139 px tall and
     JPEG-compressed, so a 5/255 gap is inside its noise; the contact sheet is torch-lit from
     above and behind, so the hair mass is usually the shadow side while the beard catches the
     key. I took the **midpoint, 0.56**, applied to the Beard albedo with its hue unchanged.
     The *direction* is certain and is the point the brief makes; the exact value is a judgement
     between two bad measurements.
   - **Lantern flame `#F0A63C`.** The pane **cores** cannot be used: they measure `#F7DFA6`
     (contact) and `#FDECAB` (sheet), a pale cream a hair off Skin `#E9D2BB`, because they are
     blown out by the light the client puts inside the lantern. Shipping that would have
     repeated v1's exact failure. I took the hue from the pane **edge**, where the glow falls
     off and the glass's own colour survives (`#533517`, R:G:B 1 : 0.64 : 0.28), and raised it
     to a value a lit pane wants.

   Both are **albedo only. Nothing is emissive** — the material carries no emission input and
   the generator asserts no `emissive*` key reaches the GLB material. Per the brief, a bright
   warm albedo is safe under the darkness guard because with every light off the scene renders
   near-black; an emissive material would be a light nobody can switch off.

5. **Pants `#474B41` and Wood `#8B6B50` remain UNRESOLVED**, carried from the brief unchanged.
   The sheet's `8` and `B` are the same glyph at 1024 px and the swatches are JPEG-noisy.
   Only the original art file settles them. Untouched by this round.

### On triangle count, which is a symptom and not a budget

The first cut of this body was **451 quads / 902 triangles** from 149,528 voxels — because a
voxel figure built from rectangular slabs greedy-meshes into almost nothing. A flat slab face
merges into one enormous quad however many voxels sit behind it, so the count is not a cost to
manage, it is the *measurement* of whether the surface has any shape. Three levers took it to
14,398, and every one of them is also a fidelity improvement:

| | quads |
|---|---|
| rectangular slabs (first cut) | 451 |
| superelliptic columns whose profile varies with z | 4,471 |
| strand grooves down the beard and hair, which the contact sheet has | 6,003 |
| fittings: soles, toe caps, lacing, rivets, seams, cuffs, studs, grip wrap, sheath, bedroll binding | 6,911 |
| rounder exponents on torso, head, beard and pack | **7,199** |

A rivet, a seam and a strand channel are all the same thing to the mesher: a **colour or
surface boundary that breaks a co-planar run**. Detail and triangle count are one lever here,
not two competing ones.

**Two traps in the groove pass, both found in a render and neither by reasoning:**

- The brow ledge, the pupils and the mouth are all painted in **Hair**, proud of the skin. A
  Hair groove over the face therefore does not cut a strand, it **deletes the feature and
  exposes the skin behind it** — at depth 2 the brow went and the face came back wearing
  goggles; even at depth 1 the pupils went, because they are only one voxel proud. Hair grooves
  are now confined to the crown (z 84+) and the nape, away from the face band.
- The sideburns and moustache are two voxels proud, and **14 is a multiple of 7**, so the
  coarse step-7 beard groove ate them at exactly x ±14. That groove now stops below the
  moustache.
- Grooves cut from +Y, which is wrong for anything on his back: the frontmost Wood voxel of a
  backpack column is the face buried inside his torso, so cutting there carves a **cavity
  nobody can see** and pays for it in hidden quads. `groove()` takes a `side` for that.

## 6. Two decisions I took, and why

Both are within the brief's grant to model from the reference; naming them because either
could reasonably have gone the other way.

1. **The pose is the contact sheet's, not the front orthographic's.** The sheet's front view is
   a display pose: the pickaxe lies diagonally across the body with its head and the lantern
   **both** out on his left. The bounding box would still centre — the contract only measures
   the box — but it would centre on a point ~14 voxels off his feet, and every instance placed
   from the asset would stand beside its own origin. The contact sheet, which the brief makes
   the primary source, carries the pickaxe in the right hand and the lantern in the left; that
   is the in-situ pose *and* the one that balances. (The front and back orthos are mutually
   consistent — the haft butt swaps sides between them — so this is a pose choice, not a
   contradiction in the reference.)

2. **Both arms hang; the pickaxe is carried upright rather than raised.** The contact sheet's
   frames are mining-strike poses. Nothing here is rigged, so a neutral carry is the right base
   for the rig that follows, and it keeps the pick head level with the top of his hair rather
   than above it — a tool that outtopped him would make a "1.20 m dwarf" 1.10 m of dwarf under
   a raised pick, since the bbox is his height.

## 7. On the grid clause

The brief as first read told this session to pass the contract unchanged while ruling a voxel
size that could not: `check_asset.py` enforced a **0.1 m** grid, and 1.20 m / 96 = 0.0125 m
lands off it on seven planes in eight. That was verified rather than argued — v1's geometry
built at `--voxel 0.0125` was rejected, `grid clause: POSITION values must use the 0.1 m
project grid` — and raised before any modelling. Wolf amended the checker
(`ce1a033`, `aae8701`) to `PROJECT_GRID_METRES = 0.0125`, with the tolerance moved from grid
steps to metres. **This session wrote nothing outside `src-assets/`.**

## 8. Files

| | |
|---|---|
| generator | `src-assets/blender/dwarf_miner.py` — replaced, not extended |
| render script | `src-assets/blender/render_dwarf.py` — new |
| editable source | `src-assets/blender/dwarf.blend` — 7,199 polys, matches the shipped GLB |
| renders | `src-assets/renders/` — 11 PNGs |
| this report | `src-assets/prompts/dwarf-miner-blender-mcp-report.md` |
| exported GLB | `src-assets/export/SM_VoxelDwarf_Miner01.glb` — **gitignored scratch, deliberately** |

`voxel_pine.py` is imported and **unedited**: the greedy mesher, material, exporter, GLB
reader, PNG decoder and volume oracle all come from it. The even-width fix still applies —
the dwarf is even-width in X and Y and the mesher's half-voxel shift is undone after meshing.

**Promoting the GLB to `assets/gltf/SM_VoxelDwarf_Miner01.glb` is a separate act from the
forge side and was not taken here.** The file currently there is still v1 (216 triangles).
