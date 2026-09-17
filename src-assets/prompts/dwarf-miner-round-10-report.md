# Round 10 report — `SM_VoxelDwarf_Miner01`, built from an empty file at one resolution

Built 2026-09-13 in Wolf's running Blender 5.2.1 LTS through the MCP addon, in 153 numbered
operations. Every gate is green: `export_dwarf.py` exits 0, `check_asset.py` exits 0, the figure is
**24,892 triangles of the 30,000 budget**, and no mass on it resolves coarser than 21 mm.

---

## 0. The live-scene report, before any geometry

Queried through the MCP addon before the first edit:

```
BLENDER 5.2.1 LTS
FILEPATH 'D:\Workspace\frostvein\src-assets\blender\SM_VoxelDwarf_Miner01.blend'
IS_DIRTY False        BACKGROUND False        ARGV ['...\Blender 5.2\blender.exe']   windows 1
--- collections ---   Collection (3 objects)
--- objects ---       Camera CAMERA (7.359,-6.926,4.958) | Cube MESH (0,0,0) | Light LIGHT (4.076,1.005,5.904)
--- meshes ---        Cube  v 8  f 6
--- materials ---     Dots Stroke (0 users), Material (1 user)
--- armatures ---     (none)
```

`bpy.app.background` is **False** and `sys.argv` carries no `--background`, so this is the GUI
instance and not a subprocess; `bpy.data.filepath` is the round's own file. What it held was the
**default startup scene** — one 8-vertex cube, a light and a camera at Blender's factory positions,
no armature, no `SM_VoxelDwarf_Miner01_r10` collection, no previous figure. Nothing was imported,
appended, linked, or recovered from git at any point.

---

## 1. What the figure is

| | |
|---|---|
| triangles | **24,892** of 30,000 |
| vertices (GLB) | 36,310 |
| size (Blender X,Y,Z) | 1.103 x 0.768 x 1.200 m |
| masses | 10 mesh objects, joined by the exporter into one mesh / one material / one 512 image |
| rig | `SK_VoxelDwarf_Miner01_r10`, 19 joints, rigid weights, no animation in the GLB |
| feature tags | 27 `feature_*` vertex groups |
| atlas | `r10`, 512 x 512, PACKED, `Closest` interpolation, backface culling on |

The body — torso, neck, head and every carved facial feature, arms, hands, legs — is **one
continuous mesh** (`r10_body`), lofted from a single cube. The nine separate objects are exactly the
pieces §4 lists as swappable.

## 2. The resolution distribution — deliverable 11

Measured on the **evaluated** mesh (mirror applied), which is what the GLB carries.

| mass | faces | tris | median edge mm | p90 mm | area m² | faces/m² | texels/cm² | mm/texel |
|---|---|---|---|---|---|---|---|---|
| `r10_body` | 8036 | 14426 | **11.0** | 31.8 | 1.8959 | 4239 | 6.6 | 3.90 |
| `r10_pack` | 1406 | 2578 | **20.9** | 44.7 | 0.8240 | 1706 | 5.7 | 4.19 |
| `r10_beard` | 922 | 1748 | **18.8** | 69.7 | 0.5288 | 1744 | 5.7 | 4.19 |
| `r10_boots` | 824 | 1520 | **14.6** | 88.8 | 0.5791 | 1423 | 5.7 | 4.19 |
| `r10_hair` | 778 | 1432 | **18.0** | 100.8 | 0.6183 | 1258 | 5.7 | 4.17 |
| `r10_straps` | 696 | 1392 | **13.0** | 37.5 | 0.1682 | 4138 | 5.8 | 4.17 |
| `r10_belt` | 384 | 768 | **20.7** | 62.5 | 0.3200 | 1200 | 5.7 | 4.18 |
| `r10_pickaxe` | 244 | 488 | **20.0** | 50.1 | 0.2213 | 1102 | 5.7 | 4.20 |
| `r10_lantern` | 170 | 340 | **12.7** | 58.0 | 0.1290 | 1318 | 5.6 | 4.22 |
| `r10_moustache` | 132 | 200 | **15.7** | 47.5 | 0.0393 | 3361 | 5.6 | 4.23 |
| **total** | **13,592** | **24,892** | | | **5.3239** | | | |

And inside the body, by region:

| `r10_body` region | faces | median edge mm | area m² | faces/m² |
|---|---|---|---|---|
| face plate, nose, brow, eye sockets | 2226 | **7.3** | 0.1182 | 18,830 |
| arms, sleeves, hands | 3176 | **12.3** | 0.8439 | 3,763 |
| cranium, ears, jaw | 572 | **13.8** | 0.1468 | 3,896 |
| pant legs | 1046 | **13.6** | 0.1287 | 8,127 |
| torso, tunic, skirt | 1016 | **20.5** | 0.6583 | 1,543 |

**Against §10's three resolution clauses:**

* **No surface coarser than 20 mm.** The spread runs 11.0 mm to 20.9 mm. The two masses that sit
  fractionally over — the pack at 20.9 and the belt at 20.7 — are flat-panelled gear, and §3 is
  explicit that a genuinely flat panel may be one large quad; what the floor forbids is a *curve*
  described in steps bigger than 20 mm, and every curved surface on the figure (cranium, shoulders,
  waist, boot toe, beard taper) is described in steps under it.
* **The face at ~8.6 mm.** 7.3 mm — one sheet pixel, and a little finer.
* **The body is not the coarsest mass.** It is the **finest**, at 11.0 mm; the coarsest is the pack
  at 20.9 mm. That is a **1.9x spread** between finest and coarsest mass, against round 9's 6x
  (body 38 mm, boots 6.4 mm).

## 3. The atlas — deliverable 12

* **512 x 512, packed into the `.blend`**, one image (`r10`), one material (`M_VoxelDwarf_r10`),
  `Closest` interpolation, `EXTEND` (= glTF `CLAMP_TO_EDGE`), backface culling on, Specular IOR
  Level left at its `0.5` default.
* **Island coverage: 62.2 %** of the map. Over §5's 60 % floor, and it got there by **re-packing,
  not by enlarging**: the first pack used a 0.0035 gutter and covered 49.7 %. With `NEAREST`
  filtering and no mipmaps there is no bilinear bleed to guard against, so the gutter came down to
  0.0004 and the coverage to 62.2 % on the same 512 map. (Round 9's 256 map was 87 % one flat fill.)
* **Texel density.** The figure runs a uniform **5.7 texels/cm² (4.19 mm per texel)**, and the
  **face region gets 17.4 texels/cm² (2.40 mm per texel)** — 3.05x the body rate, from a 1.9x linear
  scale applied to its islands before packing. The face is 3.9 % of the map for 1.1 % of the surface.
* **Painted from 3-D, not drawn in 2-D.** Every UV triangle is rasterised and each covered texel's
  barycentric position is handed back as a 3-D point; the colour function decides from there. That
  is what puts the pupil and iris, the lip line, the waist panel's square motif, the cheek warmth,
  the temple shadow, the cloth weave, the metal wear and the dirt at the hem and soles on the same
  measurements the geometry uses.
* **Value steps are quantised to 8 % rungs centred on 1.0** — crisp steps, no gradient across a
  part. Centring the rungs on 1.0 is deliberate: the first quantiser rounded to multiples of 0.08
  and 1.0/0.08 is 12.5, so a face at full value landed on 0.96 or 1.04 and **none of the approved ten
  ever reached the map exactly**. The map now carries **53 colours: the approved ten plus 43 value
  steps**, verified against the PNG:

  `#E9D2BB #5E4632 #FFFFFF #5F7A6A #474B41 #A9B2AC #8B6B50 #6B5B49 #34271C #F0A63C` — **10/10 present.**

## 4. The per-part census — deliverable 13

| object | faces | tris | size X x Y x Z m | joint(s) | what it is |
|---|---|---|---|---|---|
| `r10_body` | 8036 | 14426 | 1.067 x 0.458 x 0.931 | 17 joints | torso, neck, head + every carved facial feature, arms, hands, legs — ONE continuous mesh |
| `r10_pack` | 1406 | 2578 | 0.524 x 0.489 x 0.370 | `chest` | body, flap, buckles, bedroll across the shoulders |
| `r10_beard` | 922 | 1748 | 0.412 x 0.290 x 0.339 | `beard` | tapered, stepped in locks, mouth recess |
| `r10_boots` | 824 | 1520 | 0.522 x 0.287 x 0.197 | `foot.L/R` | sole, welt, heel, toe cap, cuff |
| `r10_hair` | 778 | 1432 | 0.421 x 0.414 x 0.352 | `head` | cranium, four crown steps, lobes, fringe, locks |
| `r10_straps` | 696 | 1392 | 0.507 x 0.520 x 0.347 | `chest` | over the sleeve caps, with hardware |
| `r10_belt` | 384 | 768 | 0.450 x 0.388 x 0.157 | `hips` | band + raised buckle + strap end |
| `r10_pickaxe` | 244 | 488 | 0.076 x 0.707 x 1.064 | `hand.R` | facetted head, bindings, tapered shaft, butt cap |
| `r10_lantern` | 170 | 340 | 0.116 x 0.116 x 0.324 | `hand.L` | frame, glass set back, cap, bail, flame cell |
| `r10_moustache` | 132 | 200 | 0.216 x 0.072 x 0.058 | `head` | drooping, under the nose |
| **total** | **13,592** | **24,892** | 1.103 x 0.768 x 1.200 | 19 | |

`r10_belt` is the **only mass with no Mirror**, because the sheet hangs a single strap end to one
side of the buckle and a mirrored belt would grow two.

## 5. The swap boundaries

§4 asks which loops are intended as the swap boundaries. They are:

| swappable | boundary ring |
|---|---|
| `r10_hair` | the head's ring at **z = 1.125** — the skull's flat crown plate, the only skull face the hair buries. The hairline in the silhouette is the hair's own fringe edge at z = 1.118, y = +0.225. |
| `r10_beard` / `r10_moustache` | the head's **chin ring at z = 0.872** (the `head` bone's head) |
| `r10_belt` | the torso's **z = 0.412 and z = 0.505** rings — belt bottom and top, the sheet's own figures |
| `r10_boots` | the leg's **z = 0.197 ring**, the boot cuff top. The pant leg runs on down to z 0.150 inside the boot so a boot swap can never open a gap at the cuff. |
| `r10_pack` / `r10_straps` | the torso's **z = 0.470 and z = 0.800** rings |
| `r10_lantern` / `r10_pickaxe` | the **hand ring at t = 0.40** along the arm axis (the `hand` bone's head) |

## 6. The rig

Nineteen joints, names exactly as the contract spells them. Every head sits on an edge ring the
figure actually has: `hips` 0.412 (belt bottom), `spine` 0.505 (belt top), `chest` 0.700,
`neck` 0.852, `head` 0.872 (the chin), `shoulder` 0.766 (the arm socket), `elbow` 0.567,
`hand` 0.460, `hip` 0.412, `knee` 0.330, `foot` 0.197 (the boot cuff). `root` carries no geometry.
The figure's right is +X, so +X is the `.R` side.

**Weights are rigid: 0 unweighted verts, 0 soft-weighted verts** — every vertex belongs to exactly
one joint at 1.0. The boundaries sit on rings, so the faces that cross one shear rather than tear.
The `.L` half comes from the Mirror modifier with `use_mirror_vertex_groups`, which remaps `.R`
weights to `.L`; without it six joints would have shipped with nothing on them.

Deflection was proved, not assumed: `joint-*.png` bends each of the twelve joint groups and
`pose-carry-*.png` puts the figure in the two-handed carry. **No tearing, no holes, nothing following
the wrong bone** in the current set — but only after two defects those renders caught:

* the first weighting bound **the skirt's outer corner to `hand.R` and `elbow.R`**, because the test
  was "outboard and far along the arm axis" and the skirt's corner is 396 mm *along* a
  downward-and-outward axis. Bending an elbow shredded the tunic. Fixed by bounding arm membership
  by distance **from** the axis (< 115 mm) as well as along it.
* rotating the head opened the **hair's bottom rim**, which had been left uncapped to untangle a
  winding problem. Filled, re-unwrapped, re-packed.

## 7. The buried-face cull

Run **once**. Six axis rays from just off each face; a face is culled only if every ray hits and
**every first hit is on geometry weighted to the same joint**. Mirrored masses had to pass the test
on both halves, so the `.L`/`.R` counts cannot diverge. It took **847 faces** — overwhelmingly the
skull under the hair and the insides of the straps, lantern and pack. A before/after diff of the
front and quarter renders moved **756 pixels of 369,600 (0.2 %)**, all silhouette anti-aliasing: the
cull removed nothing a camera can see.

Two caveats, stated rather than hidden:

1. the cull ran **before** the weighting correction in §6, so its joint labels were the pre-fix ones.
   Re-running it now would be a different question asked of the same geometry; the deflection renders
   are the check that matters, and they are clean.
2. a later, narrower pass removed 126 more faces of the **pant leg inside the boot** (leg and boot
   are both `foot.L/R`, so in-joint), after the leg was re-resolved.

## 8. What I found that contradicts the inputs

**The tall skin column in the side views is the EAR, not the neck.** `dwarf-ortho/README.md` reads
it as "a skin column running z/H 0.793 down to 0.679, so 0.121 H of visible neck (17 source px), up
to 9 source px wide". Measured off the pixels, the column in `side-left.png` sits at **columns 57–62**
(6 px, 51 mm of depth) and **rows 28–43**, which is **z/H 0.850 down to 0.743**. §2's settled table
puts **ear top/bottom at 0.850 / 0.729**. The rows match the ear to within a pixel and miss the
README's stated neck range by eight rows; the column is 6 px deep where a neck in profile would be
11–12; and it carries a darker mark inside it, where a concha would be. §2 lists an ear and lists no
neck, and §2 is what the brief calls settled — so I built it as an ear, reaching x = 0.228
(ear-to-ear 0.383 H) against the hair's 0.2016, so **27 mm of ear shows past the hair**: tucked to
the skull, against round 9's 93 mm flanges.

The figure does have a neck (`neck` ring z 0.852, chin 0.872, collar top ~0.856) and it is bare, but
in profile it reads as roughly 16 mm rather than the README's 0.121 H. That follows from §2's own
numbers: it puts the **mouth at z/H 0.704** and the **head+hair mass ending at 0.707**, so the chin is
level with where the head silhouette gives way to the shoulders, and there is very little room for a
neck. Wolf's 2026-09-12 line that "neck is visible on side view of reference images" is satisfied —
there is bare skin between collar and jaw — but I do not think the tall column is it, and I would
rather say so than quietly build to a measurement I believe is wrong.

## 9. Where it departs from the sheet, and why

* **The arms.** A-pose, 40° from vertical, as §4 requires, so the front and side silhouettes do not
  match the sheet at the arms. Armpit clearance opens to **47 mm** within 70 mm of the socket, and
  because the arm is topologically continuous with the torso there are no coplanar surfaces between
  two objects anywhere — round 8's failure mode cannot occur here.
* **The props sit in neutral hands.** The lantern hangs from `hand.L`, the pickaxe is gripped by
  `hand.R`. The two-handed carry is delivered as a render.
* **The pickaxe is pitched back 28°.** Held bolt upright its head sat at z 1.14 — crown height — and
  read as a flat cap across the face in every three-quarter view. Pitched back it lands behind the
  shoulder, which is how `f088` carries it. Its dimensions are round 7's `front.png` choice, which
  §1 says stands: shaft 0.91 H, blade span 0.38 H (measured 0.456 m), head-span/length 0.42.
* **The crown steps in depth as well as width.** The sheet's front view gives four widths
  (0.164 / 0.229 / 0.286 / 0.336 H); its side view gives the matching depths, and the first pass
  missed them — the silhouette test caught the crown 3–4 px too deep through rows 7–14.

**Silhouette against the sheet**, measured at one rendered pixel per source pixel:

| band | front IoU | side IoU |
|---|---|---|
| head + hair (rows 7–48) | 0.625 | 0.798 |
| torso (49–95) | 0.588 | 0.807 |
| skirt (96–119) | 0.581 | 0.657 |
| boots (120–147) | 0.679 | 0.683 |
| **whole figure** | **0.608** | **0.764** |

The number that matters is **model-only**: 990 px of 8,899 in front, 199 of 6,961 in side. The
figure almost never exceeds the sheet's envelope. The ref-only pixels are the sheet's arms-down pose
and its pickaxe and lantern positions, both of which §4 says will not match.

## 10. Exporter and checker output, verbatim — deliverable 15

```
EXPORT D:\Workspace\frostvein\src-assets\export\SM_VoxelDwarf_Miner01.glb
  parts joined      10 -> 1 mesh 'SM_VoxelDwarf_Miner01_r10'
  object / mesh     SM_VoxelDwarf_Miner01_r10 / SM_VoxelDwarf_Miner01_r10
  materials         M_VoxelDwarf_r10
  texture image     r10   in the GLB: T_VoxelDwarf_r10
  triangles         24892  of 30000 budget
  size m (X,Y,Z)    1.103 x 0.768 x 1.200
  blender min Z     0.000000   (glTF min Y)
  blender centre XY 0.000000, 0.000000   (glTF centre X, Z)
  topology          n-gons 0   non-manifold edges 0   loose verts 0   loose edges 0   degenerate faces 0   flipped winding 0   missing UV layer 0
  feature tags      27 vertex groups, 0 face attributes   feature_bail, feature_beard_locks, feature_bedroll, feature_binding, feature_boot_cuff, feature_brow.R, feature_buckle, feature_butt_cap, feature_collar, feature_crown_steps, feature_cuff.R, feature_ear.R, feature_eye.R, feature_flame_cell, feature_flap, feature_glass, feature_hair_locks, feature_heel, feature_hem, feature_moustache, feature_mouth, feature_nose, feature_pick_head, feature_shoulder.R, feature_strap_end, feature_strap_hardware, feature_toe_cap
  live modifiers    r10_beard:MIRROR, r10_body:MIRROR, r10_boots:MIRROR, r10_hair:MIRROR, r10_moustache:MIRROR, r10_pack:MIRROR, r10_straps:MIRROR
  rig               joints 19   missing joints 0   unexpected joints 0   unweighted verts 0   soft-weighted verts 0   joint names in the GLB: beard, chest, elbow.L, elbow.R, foot.L, foot.R, hand.L, hand.R, head, hip.L, hip.R, hips, knee.L, knee.R, neck, root, shoulder.L, shoulder.R, spine
  GLB min/max       [-0.5513999462127686, 0, -0.3841227889060974] / [0.5513999462127686, 1.2000000476837158, 0.3841227889060974]   (glTF axes: X, Y up, Z)
  bytes             2085704
```

exit 0.

```
$ python3 scripts/bench/check_asset.py src-assets/export/SM_VoxelDwarf_Miner01.glb
FIGURES src-assets\export\SM_VoxelDwarf_Miner01.glb size_m=1.1x1.2x0.8 min_y_m=0.000000 centre_x_m=0.000000 centre_z_m=0.000000 palette=#80624A,#625443,#5F7A6A,#597263,#566E60,#281E15,#D6C1AC,#4F3B2A,#C2C2C2,#7C6A55,#9BA49E,#2C2118,#40372C,#41453C,#E9D2BB,#5A4C3D,#C4CEC8,#8B6B50,#EBEBEB,#8E9690,#3C2D20,#A9B2AC,#658170,#698675,#74624F,#473526,#6C8B79,#70907D,#C4B09D,#5E4632,#739481,#789A86,#403023,#75573E,#5D7868,#382F26,#617C6C,#A17C5D,#FCE3CA,#4A5F52,#664C36,#34271C,#475B4F,#382A1E,#474B41,#F0C8AA,#6B5B49,#403022,#FFFFFF,#56402E,#F0A63C,#291F16,#6D513A tris=24892 verts=36310 mesh=SM_VoxelDwarf_Miner01_r10 profile=painted-map
```

exit 0.

## 11. Cost — deliverable 16

`session_tokens.py` needed the explicit `--transcript` path, as §9 warns.

```
$ python _bmad/scripts/session_tokens.py --tool claude \
    --transcript ~/.claude/projects/D--Workspace-frostvein/f81b93da-....jsonl
Session token cost  (f81b93da-....jsonl, tool=claude)  (585 turns, claude-opus-5)
  input (fresh)          1,170
  cache creation     1,043,772
  cache read       205,904,073
  output               887,402
  total processed  207,836,417
  wall-clock           112 min  (elapsed, includes idle gaps)
  est. cost            $131.67  (benchmark - verify rates in PRICES)
```

Row for the ledger: **`dev-art` — 207,836,417 tokens processed, 112 min, ~$131.67.** No ledger row
was written: recording one writes into the forge's `_bmad-output/`, which CLAUDE.md forbids and §10
excludes.

## 12. Deliverables

| # | what | where |
|---|---|---|
| 0 | live-scene report | §0 above |
| 1 | the source, saved after every operation, image packed | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture map | `src-assets/blender/textures/T_VoxelDwarf_r10.png` |
| 3 | the render script | `src-assets/blender/render_r10.py` |
| 4 | stage render sets | `src-assets/renders/r10/stage-{1..4}-*.png` |
| 5 | progress renders | `src-assets/renders/r10/progress/` (79 files) |
| 6 | five final views, flat and key-lit | `src-assets/renders/r10/final-{flat,lit}-*.png` |
| 7 | beside f088 and f104 | `src-assets/renders/r10/vs-frames-f088.png`, `-f104.png` |
| 8 | side profile beside the sheet | `src-assets/renders/r10/vs-ortho-side-left.png` |
| 9 | readability strip, 100 px and 60 px | `src-assets/renders/r10/readability.png` |
| 10 | posed carry + per-joint-group deflection | `src-assets/renders/r10/pose-carry-*.png`, `joint-*.png` |
| 11 | resolution table | §2 |
| 12 | UV coverage | §3 |
| 13 | per-part census | §4 |
| 14 | this report | `src-assets/prompts/dwarf-miner-round-10-report.md` |
| 15 | exporter and checker output | §10 |
| 16 | cost | §11 |

Nothing was written outside `src-assets/`; no git command was run.

## 13. What I would fix next

* **The tunic reads more muted than the reference.** The base is the approved `#5F7A6A` exactly, but
  most faces take a step at or below 1.0 and the cloth weave subtracts another 2 %, so the whole
  garment sits a rung darker than `f104`'s. A value-step distribution biased upward would close it
  without touching the palette.
* **The hem/leg junction is ragged in close-up.** The pant leg was re-resolved three times (it came
  out of the form pass at a 68 mm median, the one real violation of the 20 mm floor) and the sequence
  left small uneven steps where the hem lip meets the leg. Invisible at 100 px, visible at 0.46 m of
  frame.
* **The pack and belt sit at 20.9 and 20.7 mm.** Flat panels, so defensible under §3, but they are
  the two masses I would subdivide first if the budget were spent differently.
* **The face plate's mirror seam.** The two halves share texels by design, and at the nose centre
  line the shared island edge shows a faint one-texel discontinuity in the carry pose.

## 14. Operations that went wrong, and what caught them

Recorded because the next round inherits the tooling, not just the file.

| op | what happened | what caught it |
|---|---|---|
| 12–13 | `bisect_plane` clipped corners off the sloped skirt and jaw, leaving n-gons that dissolving made worse | the n-gon count in the per-op audit |
| 15 | the leg inset was snapped with the centre-line setter, collapsing the hole to a line | the boundary-edge audit |
| 49 | the chin was built at z 0.950, *above* the sheet's mouth at 0.845 | re-reading §2 against the 13x front crop |
| 71–80 | subdividing a sub-region's edge set three times compounded into a 0.68 mm sliver mesh; unsubdividing it back wrecked the face, which then had to be rebuilt from the neck ring | the resolution table (1.6 mm median), then the render |
| 92 | an extrude translated by zero, leaving coincident faces and zero-area walls | the `remove_doubles` count |
| 61 / 66 | the `section` setter split on `y > 0`, collapsing the beard's lower rings and the whole pack in Y | the side render — the pack simply was not there |
| 118 | `recalc_face_normals` turned the body inside out (its cage is an open half-shell split down the mirror plane) | the head went featureless in the next render; fixed by orienting on **signed volume of the evaluated mesh** instead |
| 136 | the skirt's sides were painted as forearm skin | the flat-albedo render |
| 139 | the skirt was *weighted* to the arm bones — same root cause | the deflection renders shredded |
| 144–148 | the leg overshot to 6.4 mm, then back to 12.2 mm | the resolution table |
| 151 | unsubdivide scrambled the leg's UVs into stripes | the textured render against a clay render of the same region |

The pattern worth carrying forward: **an "is this part of the arm?" test needs the distance *from*
the arm axis, not only along it.** The same bug bit the paint and the weights independently, and in
both cases a render — not a gate — is what found it.
