# Round 8 — model the resolution up, then rig him

**Two halves, in this order: raise the figure's geometric resolution and paint what only paint can
carry, then build the rig. Deliver the first half complete before starting the second.**

Round 7 built a good figure and proved that value does work the old 16-cell palette could not. It
also left the figure at a resolution that does not match the reference, and the per-part census says
so plainly:

| part | faces | tris | | part | faces | tris |
|---|---|---|---|---|---|---|
| **`r7_head` — the whole face** | **6** | **12** | | `r7_beard` — the whole bush | 28 | 56 |
| `r7_neck` | 3 | 6 | | `r7_hair` | 52 | 104 |
| `r7_belt` | 5 | 10 | | `r7_torso` | 16 | 32 |
| `r7_leg.L` / `.R` | 8 | 16 | | `r7_skirt` | 21 | 42 |
| **whole figure, 20 parts** | **446** | **892** | | of the **30,000** ceiling (§4) | | **3 %** |

**The head is one box.** Forehead, brow, sockets, cheeks and mouth are a single flat `+Y` quad
carrying a 500 px/m image. There is no brow part, no eye part and no moustache part in the figure at
all. Meanwhile **a background pine is 3,474–5,894 triangles** — a tree in the middle distance
carries four to six times the geometry of the hero character.

**Round 7's argument against modelling those features does not hold, and this brief retracts it.**
It read: *"an eye would need cubes at 1.4 % of height — a ~70-cube-tall dwarf, far past any
budget."* That is true **only on a uniform voxel lattice**, and the lattice went away at round 4
when this asset became box-modelled; the style clauses are gone entirely since round 7. A brow ridge
is a three-box wedge and costs the rest of the figure nothing. "Detail is texture, not geometry" was
right about pupils and cloth motifs and was over-applied to **form**.

Wolf's ruling, 2026-09-13: **brows, nose, hair, beard, moustache, eyes and clothing are modelled**,
and the head is rebuilt with real form rather than gaining add-ons over a flat plane.

---

## 1. The order is a requirement, not a suggestion

**Half A is geometry-and-paint. Half B is the rig. Do not begin B until A is delivered — exported
clean, rendered, and reported.**

Why this way round: the look is judged by Wolf's eye and nothing else can judge it, whereas the rig
is judged by gates the exporter now runs (§7). If the session is killed — four of six delegated runs
in this project have been — a finished half A is a shippable round and a half-built skeleton is not.
**Save the `.blend` after every part and write a progress render.**

## 2. What geometry carries, and what texture carries

The division, and it is the round's organising idea:

- **Geometry carries FORM** — anything that must survive the light moving or the camera swinging: a
  brow ridge, the planes of a nose, the depth of an eye socket, a hem lip, a buckle, the shoulder
  line, beard locks, the steps of the hair mass.
- **Texture carries COLOUR and micro-value** — the pupil and iris, the eye white, a lip line, a
  cloth motif, cheek warmth, wear and dirt. Things that are a colour change on a plane.

**The test when you are unsure:** would the feature still read if the key light came from the other
side? If yes it can be paint; if it would vanish, it wants geometry.

This also cuts in favour of geometry at small sizes, which round 7 missed: **a modelled brow throws
a real shadow at 60 px under any light, while a painted brow can be washed out by the key.** Do not
treat the 60 px strip as an argument for flatness.

## 3. Half A — the acceptance list, judged by eye

Against `reference-sheet.jpg`, `dwarf-ortho/` and `dwarf-frames/`, each item at **full size, 100 px
and 60 px** figure height.

### 3.1 The head, rebuilt

**The head stops being a box.** Wolf's call, and the reason is that socket *depth* is what makes a
brow read at all:

1. **A brow shelf that projects**, with the forehead behind it and the eyes under it.
2. **Recessed eye sockets** — the eye sits in a hollow, not on the front plane.
3. **Cheekbones and a jaw** — the side of the head is not a single vertical plane.
4. **Brows as their own parts** (`r8_brow.L`, `r8_brow.R`), separate objects, so they can swap
   later.
5. **Eyes with form** — a lid or socket edge in geometry; **the pupil, iris and white stay
   painted**, because they are colour on a plane and geometry adds nothing.
6. **A nose with planes** — round 7's is 12 faces and reads as a pale slab from brow to moustache.
   It needs a bridge, a tip, and shaded sides; the lit front should be the narrow part.
7. **A moustache as its own part**, not paint on the beard's top quad.

**The face texture will be re-authored, and that is accepted.** Round 7 deliberately kept the head's
`+Y` quad as one island to give the face 500 px/m. Rebuilding the head splits that island, so the
painted face has to be laid out again across several islands with seams. **Keep the density where the
features are** — say what you gave each island — and do not let the rebuild cost the face the
readability it won.

### 3.2 Hair, beard, moustache

8. **The beard is a bush, not four stacked slabs.** Round 7 named this itself: right width, right
   length, right taper, and still four slabs. It wants **locks** — more and smaller steps on its own
   silhouette.
9. **The hair is a stepped mass** — a crown that steps, a fringe, side lobes with locks, not a slab.
   **Its front edge in profile stays at `y = +0.111`** (§5).

### 3.3 Clothing and gear

10. **The tunic reads as layered** — an overtunic over an under-layer with a real edge where they
    meet, in geometry, plus more than one green.
11. **Shoulder pieces read as separate from the sleeve**, with a step at the shoulder line.
12. **A hem lip at the bottom of the skirt** — a distinct band, modelled, not a painted stripe.
13. **The waist panel**, lighter than the tunic, with the square motif the sheet draws on it. Panel
    edge in geometry; the motif in paint.
14. **The belt is leather with a raised buckle.** It is five faces today.
15. **The boots have a sole and a cuff** — steps at the ground and at the top.
16. **The lantern stops being a flat orange rectangle** — a metal frame, glass lighter than the
    frame, and the flame cell. **The flame is a COLOUR and never an emitter**; that ruling stands.
17. **Skin is not uniform pale cream** — cheek warmth and a temple shadow, crisp steps, in paint.

### 3.4 The paint that goes with it

- **One material, one image**, `M_VoxelDwarf_r8` and `r8`. **256 x 256, or 512 x 512** if the new
  island count needs it — say which and why.
- **The approved palette stays the base** (`#E9D2BB, #5E4632, #FFFFFF, #5F7A6A, #474B41, #A9B2AC,
  #8B6B50, #6B5B49, #34271C, #F0A63C`); value steps on top of it are yours. **Crisp steps, no
  gradients across a part.**
- **State the texel density of every region**, as round 7 did. Round 7's body islands are 35 px/m,
  which is right for a flat colour and one pixel per hem lip.
- **The texture node's interpolation must be `Closest`** — the contract requires NEAREST filtering
  in the GLB and the exporter deliberately does not force it, because filtering changes every pixel
  of the look and is not the exporter's call to make silently.
- Backface culling on. Specular IOR Level stays at its `0.5` default: a `0` emits
  `KHR_materials_specular` and the no-extensions clause rejects the export.

## 4. Triangles: the ceiling moved to 30,000, and detail now beats thrift

**Wolf's ruling, 2026-09-13: the budget goes up, and *"it's easier to optimize than add more details
later on"*.** So `TRI_BUDGET` is **30,000**, up from 4,000.

Where the old number came from, because it is the thing that capped round 7: **4,000 was a row in
the model sheet's DERIVED LOD ladder** — a table the 2026-09-11 ruling had explicitly made
non-binding — and round 7 promoted it to a build gate. Round 7 then spent 892 of it, 22 %, and
still produced a one-box head. Meanwhile `tech-art-guidelines.md`, `dwarf_miner.py` and the MCP
brief all record **14,000–30,000** as the expectation for a dwarf at this scale, and the r3 dwarf
shipped and ran at **14,398**. The ladder's crowd argument — twenty dwarves costing twice the
terrain — constrains **LOD1 and LOD2, not LOD0**: LOD0 is a handful of close-ups, and by the sheet's
own measurement a gameplay dwarf is **8.74 px**, so the crowd never draws LOD0 at all.

**What that means for you, concretely:**

- **Spend geometry wherever the reference has a feature.** If you are choosing between a step and a
  flat plane, take the step. The figure should look under-budget by a wide margin and still read at
  the reference's resolution.
- **For scale only:** the §3 list is worth roughly 1,000–2,000 triangles if built plainly, and this
  ceiling is fifteen to thirty times that. **Running out of budget is not a risk this round. Being
  too thrifty is the risk**, and it is the one that has cost seven rounds.
- **The direction of the mistake matters.** A box model reduces by *removing boxes*, so detail built
  now is recoverable by a decimation pass later; detail not built now costs a whole round. That is
  Wolf's reason and it is the sheet's own argument for box modelling.
- **No triangle count is an acceptance criterion, in either direction.** Round 6 placed the pickaxe
  five times and segmented a tool handle to move a silhouette scalar; a number noise can satisfy
  will be satisfied by noise. Do not subdivide to look busy, and do not smooth to look expensive —
  spend on *features the reference has*, and let the census (#13) report where it went.
- **The gate's job is now to catch a MISTAKE** — an unapplied subdivision, a mirrored duplicate, a
  cull that never ran — not to shape the art. If you hit 30,000, something is wrong; say what.

The LOD ladder itself (LOD1 ≤ 800, LOD2 ≤ 150) is unchanged and **is not this round's problem**:
decimation comes after the look is signed off (§12).

**Topology, enforced by the exporter:** triangles and quads only, manifold, no loose or degenerate
geometry, consistent winding, a UV layer, no unapplied modifiers (bar the Armature one). **Judged in
review, not by machine:** no interior geometry, no overlapping UV islands bar deliberate mirrored
pairs, edge loops at the joints.

## 5. Round 7's settled facts — inherit these, do not rediscover them

Its report earned these, and three of them matter *more* now that the part count is going up.

- **Pieces within one object must overlap, never meet exactly.** Coincident rims weld under
  `remove_doubles` and produce edges with four faces, which the exporter rejects as non-manifold.
  With locks, lids and hem lips this will bite repeatedly if ignored.
- **A buried-face cull must not assume closed solids.** Round 7 lost nine faces of a hand to a
  ray-parity containment test run repeatedly against occluders it had itself opened, and paid a full
  rebuild for it. Its replacement — a six-direction enclosure test that fails safe — is the one to
  use, **once**, over closed parts. An asymmetric face count between `.L` and `.R` is the tell that
  a cull went wrong.
- **The hair's front edge in profile is `y = +0.111`** (`side-left.png` col 71), not the full depth
  of the head. Run the lobes forward and the profile becomes a featureless brown slab with no
  forehead, brow or nose. This is in no depth table.
- **The bare neck's lever is the STRAP, not the hair.** In an orthographic side view the outermost
  |x| wins, so anything shoulder-borne outboard of the neck erases it.
- **The pickaxe reads edge-on from the front because a carried pick hangs in the sagittal plane.**
  Not a defect, and not to be fixed in the mesh — it is a pose, and half B is where poses live.

Live defects in the inputs, **not yours to chase**:

- **Check 4 (silhouette step density) is WITHDRAWN** — see §4. Do not measure it.
- **Check 5 (the bare neck) is WITHDRAWN**; round 7 §7.2 settled what it was really measuring.
- **The pickaxe's four authorities disagree by 2.3x.** Round 7 chose `front.png` (shaft 0.91 H,
  blade span 0.38 H, head-span/length 0.42) and stated it. Keep that choice.
- **`dwarf-model-sheet.md` keeps a superseded video-derived table above the orthographic one.** The
  orthographic numbers win. **The sheet's printed annotations are unreliable in general; the
  drawings are orthographic and trustworthy.**
- **`session_tokens.py`'s Windows slug defect is still open.** Use `--transcript`.

## 6. Half B — the rig

**Nineteen joints, and the names are the contract** because the game binds by name:

```
root  hips  spine  chest  neck  head
shoulder.L/R  elbow.L/R  hand.L/R
hip.L/R  knee.L/R  foot.L/R
beard
```

- **One armature in the round's collection**, named `SK_VoxelDwarf_Miner01_r8`. The hierarchy is
  yours; the names are not.
- **ONE mesh, RIGID weights.** Wolf's ruling from round 3 stands: every vertex belongs to **exactly
  one** joint at weight **1.0**. No soft skinning, no separate limb meshes. Soft weights would not
  fail to render — they would quietly smooth the joints of a hard-edged figure, which is why the
  exporter counts them (§7).
- **Every new part from half A is weighted whole, to one joint:** brows, eyes, nose, moustache and
  hair to `head`; beard locks to `beard`; tunic layers, shoulder pieces and the waist panel to
  `chest` or `spine`; the belt and skirt hem to `hips`; boot cuffs and soles to `foot.L/R`. A part
  that spans a joint needs a loop at it and splits between the two.
- **Round 7's edge loops are at all fifteen sites** (its §6 lists each z and which parts carry the
  ring). **The `head` joint's loop sat on `r7_neck`** rather than on both parts, because cutting the
  head's `+Y` quad would have split the face island. **The head rebuild changes that constraint** —
  the face is multi-island now, so put the loop where the deformation needs it and say where it went.
- **The mesh ships in the NEUTRAL stance. The pose is a pose.** Round 7 named the reference's
  two-handed carry as the biggest remaining likeness gap, and a rig is the right place for it. So
  **the GLB carries the rest pose and no animation**, and the carry is delivered as renders (§9).
  Do not bake it into vertices.
- **Prove the weights hold.** A rig nobody posed is a rig nobody has checked. Bend each joint group
  and render it: shoulders and elbows, hips and knees, neck and head, the beard. Tearing, a hole at
  a joint, or geometry following the wrong bone is a defect to fix, not to report.

## 7. What changed on the orchestrator's side — all of it verified 2026-09-13

**`check_asset.py` now PASSES this asset, so from this round its output is a GATE and not a quote.**
Round 7 was told to report it verbatim pass or fail because it was broken. It is not any more.
Verified against round 7's own `.blend`, re-exported here:

```
FIGURES src-assets/export/SM_VoxelDwarf_Miner01.glb size_m=0.8x1.2x0.7 min_y_m=0.000000
  centre_x_m=0.000000 centre_z_m=0.000000 palette=<28 colours> tris=892 verts=1526
  mesh=SM_VoxelDwarf_Miner01_r7 profile=painted-map
exit 0
```

- **The palette clause understands a painted map.** It asks whether the embedded image *is* a cell
  atlas — square, a whole number of 16 px cells, every cell one flat colour — rather than whether it
  is 64x64. A painted map is read as a **census: every colour it carries, most-painted first**, with
  `profile=painted-map` printed so nothing is skipped silently. Your map will report a few dozen
  colours and that figure is what Wolf reads against the sheet, so **no stray colours**.
- **The voxel-only clauses do not apply to this family** — the 0.0125 m grid clause and the
  quad-soup clause describe the generated-voxel pipeline. Inapplicable, not waived; the profile says
  which contract ran.
- **The naming clause accepts the revision.** `SM_VoxelDwarf_Miner01_r8` inside
  `SM_VoxelDwarf_Miner01.glb` is legal; any other disagreement still fails, and `mesh=` is printed
  so a stale export announces itself.
- **`REV` is `r8`**, so **your collection must be named `SM_VoxelDwarf_Miner01_r8`**, and every
  datablock carries `r8`: object, mesh, material, image, armature.

**The exporter now supports and gates the rig** — verified on two synthetic rigged fixtures, one
complete and one deliberately short of joints:

```
  rig   joints 19   missing joints 0   unexpected joints 0   unweighted verts 0
        soft-weighted verts 0   verts weighted to a non-bone 0
        joint names in the GLB: beard, chest, elbow.L, ... spine
```

- the skin is exported and the **joint names are read back out of the written file**: a rig that
  stops at the `.blend` fails the build;
- an **Armature modifier no longer counts as an unapplied modifier**;
- **an unrigged export still works** (`rig <NONE -- unrigged figure>`), which is what makes half A
  exportable before half B exists.

**Three silent-failure traps were closed, all of which had already fired:**

- **Your texture must be PACKED into the `.blend`.** Round 7's image pointed at
  `D:/Workspace/.../T_VoxelDwarf_r7.png` with nothing packed, so off the authoring machine the
  exporter dropped the material's texture **entirely** — no images, no sampler, no
  `baseColorTexture` — while printing `texture image r7` and exiting 0, because that line reported
  the datablock's *name*. The image is packed now and **the build fails if the GLB carries no
  image**: the line reads `texture image r8   in the GLB: <name>`.
- **The GLB's sampler now declares CLAMP_TO_EDGE**, forced by the exporter. Round 7's declared no
  wrap mode, which glTF defaults to REPEAT — a bleed hazard on a packed island set, and you will
  have many more islands than round 7 did.
- **`seat_on_origin` was wrong for any part carrying an object transform.** It measured bounds in
  world space, added the shift to *local* vertex coordinates and zeroed `ob.location` — correct only
  when the object transform is already identity, which round 7's parts happened to be. A part placed
  with `primitive_cube_add(location=...)` exported geometry **0.75 m off the floor while the export
  line printed `min Z 0.000000`**. Fixed, and **a seating gate now reads the GLB's own POSITION
  bounds** and fails the build unless min Y and centre X/Z are zero. Place parts however you like.

One command, from the repo root, unchanged:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/export_dwarf.py

## 8. It must be watchable — unchanged

- **Author INSIDE Wolf's running Blender through the MCP addon. Never in a `blender --background`
  subprocess.** Only the export and the render passes run headless.
- **Prove it before your first edit:** query the live scene and report what was already in it. **If
  you cannot reach the live instance, stop and say so.**
- **One part per tool call**, named for the outliner (`r8_*`). **Save the `.blend` after every
  part** and write a numbered progress render. The `.blend` accumulates: r5, r6 and r7 stay in it,
  excluded from the view layer.
- **No generator script.** No `dwarf_r8.py`, no part-spec file as the real source. Each call
  performs one modelling operation on the live scene. The `.blend` is the deliverable. A generator
  makes you think in loops and parameters, and this round's features are hand-placed and asymmetric.
- **Every swappable feature stays its own named object** — brows, eyes, moustache, beard, hair,
  pack, belt, boots, tunic, tools — because the endpoint is *hand-author the parts, generate the
  combinations*. That costs nothing now and is a rebuild later.
- **Do not stop and wait for approval.** Work through both halves and deliver.

## 9. Renders

Copy `render_r7.py` to `render_r8.py` and keep its two passes — flat Workbench for measurement,
EEVEE key-lit with one sun and a shadowless fill for judgement. **Absolute paths, and the revision
in every path** (`src-assets/renders/r8/`): Blender resolves a relative `render.filepath` against
the `.blend`, and a fixed path across rounds once made a round analyse the previous round's images.

New this round:

- **a face close-up in THREE-QUARTER view as well as front**, because the whole point of modelled
  form is that it survives a camera move;
- **the head beside `f104` at matched scale**, as round 7 did;
- **the posed carry** from front and three-quarter, plus **one deflection render per joint group**
  (§6).

## 10. Deliverables

| # | what | path |
|---|---|---|
| 0 | the live-scene report proving you are in Wolf's Blender | in the report, before any geometry |
| 1 | the source, saved after every part, image PACKED | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture map | `src-assets/blender/textures/T_VoxelDwarf_r8.png` |
| 3 | the render script | `src-assets/blender/render_r8.py` |
| 4 | the exported GLB (gitignored scratch) | `src-assets/export/SM_VoxelDwarf_Miner01.glb` |
| 5 | progress renders, one per part | `src-assets/renders/r8/progress/NN-<part>.png` |
| 6 | five final views, flat **and** key-lit | `src-assets/renders/r8/` |
| 7 | the readability strip at 100 px and 60 px | `src-assets/renders/r8/readability.png` |
| 8 | side-by-side against `front.png` and `side-left.png` | `src-assets/renders/r8/vs-ortho-*.png` |
| 9 | the face, front and three-quarter, beside `f104` | `src-assets/renders/r8/vs-face-*.png` |
| 10 | the body beside the sheet, matched scale | `src-assets/renders/r8/vs-body.png` |
| 11 | the posed carry, front and three-quarter | `src-assets/renders/r8/pose-carry-*.png` |
| 12 | one deflection render per joint group | `src-assets/renders/r8/joint-*.png` |
| 13 | **a per-part census — name, faces, triangles, size** | in the report, as the table above |
| 14 | your report, covering every §3 item and all of §6 | `src-assets/prompts/dwarf-miner-round-8-report.md` |
| 15 | the exporter's and the checker's output, verbatim | in the report |
| 16 | your cost — `session_tokens.py --transcript`, model from your banner, row `dev-art` | in the report |

## 11. How this round is judged

By Wolf's eye on the seventeen items in §3 — **that list is half A** — and above all on whether the
figure now reads at the reference's resolution rather than as boxes. Then mechanically:

- **the head is no longer a box**, and the per-part census (#13) shows where the geometry went
- **`check_asset.py` exits 0.** It is a gate now, not a quote.
- the export passes: ≤ 30,000 triangles, topology clean, one material, one image, the seating gate,
  and the rig gate at **19 joints / 0 missing / 0 soft-weighted / 0 unweighted**
- every §3 feature still readable at 60 px, and the three-quarter face shows the form the front does
- the posed renders show no tearing and no hole at any joint
- nothing written outside `src-assets/`, and **no git command run at all** — the operator commits

## 12. What is NOT in this round

Game lighting is Epic 11's. Decimation and the LOD ladder come after the look is signed off — LOD0
is the only level this round has an opinion about. The combination layer — a dwarf as a seed, twelve
discrete axes plus four continuous dials — is a separate task after sign-off and **is not yours to
start**; all this round owes it is the named-part decomposition in §8. Animation clips are not in
this round either: the rig ships posable, with no animation in the GLB.
