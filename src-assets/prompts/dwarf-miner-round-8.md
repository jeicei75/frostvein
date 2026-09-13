# Round 8 — paint the rest of him, then rig him

**Two halves, in this order: finish the texture on the body and the gear, then build the rig.
Deliver the first half complete before starting the second.**

Round 7 settled the argument the previous six rounds were having. Given complete freedom to bevel,
subdivide and smooth-shade, it built a hard-edged flat-faced figure anyway and **painted** the
features instead — forehead, brows, eyes, nose, moustache, mouth, ears — at 892 of 4,000 triangles.
Wolf's verdict: *"now we are getting there"*. The lesson is on the record: the face was never a
geometry problem.

**That lesson is only half applied.** The face got a 500 px/m island and eight painted features. The
tunic, skirt, belt and boots got **one flat colour each** at 35 px/m — which is exactly the state the
face was in at the end of round 6. The reference's body is not flat: it has a layered overtunic,
shoulder pieces, a hem lip, a lighter waist panel with a square motif, and several greens. Round 8
does to the body what round 7 did to the head.

**And then the rig**, which round 7 named as the largest remaining likeness gap and which its §6
already prepared: edge loops exist at all fifteen joint sites, measured on the built meshes.

---

## 1. The order is a requirement, not a suggestion

**Half A is the texture. Half B is the rig. Do not begin B until A is delivered — exported clean,
rendered, and reported.**

Why this way round: the look is judged by Wolf's eye and nothing else can judge it, whereas the rig
is judged by gates the exporter now runs (§5). If the session is killed — four of six delegated runs
in this project have been — a finished texture half is a shippable round and a half-built skeleton is
not. **Save after every part and write a progress render, as always.**

**Do not touch the geometry to fix a look problem in half A** unless the report says why the paint
could not carry it. Round 7's own finding is that value does the work; 892 triangles is 22 % of the
budget and the temptation to spend the rest is the trap this round is arranged against. Geometry
changes in half B are a different matter: see §4.

## 2. Half A — the body and the gear, as acceptance items

Judged by eye against `reference-sheet.jpg` and `dwarf-frames/`, each at **full size, 100 px and
60 px** figure height. Seven items:

1. **The tunic reads as layered, not as one green.** The reference has an overtunic over an
   under-layer with a visible edge where they meet, and more than one green in play.
2. **Shoulder pieces read as separate from the sleeve** — a value step at the shoulder line, which
   is where a viewer reads the silhouette's width.
3. **A hem lip at the bottom of the skirt** — the reference's hem is a distinct band, lighter or
   darker than the skirt above it, not the same colour running off the edge.
4. **The waist panel**, lighter than the tunic, with the square motif the sheet draws on it.
5. **The belt reads as leather with a buckle**, not as a dark stripe: the buckle is already a
   230 px/m island, so this is paint, not geometry.
6. **The boots have a sole and a cuff** — two value steps, at the ground and at the top.
7. **The lantern stops being a flat orange rectangle** — a metal frame, a glass面 lighter than the
   frame, and the flame cell. **The flame is a COLOUR and never an emitter**, that ruling stands.

And three corrections carried over from round 7's own report, which named them without being asked:

8. **The nose.** Round 7's is a large pale slab from brow to moustache, lighter than everything
   around it, and it reads as a snout at every size. Shade its sides; the lit front should be the
   narrow part, not the whole nose.
9. **The mouth at 60 px.** Round 7 §12.3's answer is the right one and is not to be re-derived: the
   fix is **a darker beard immediately under the moustache**, so the mouth reads as the bottom of a
   two-value shape rather than as a 1 px line of its own. Do not make the mouth bigger.
10. **Skin is near-uniform pale cream.** Cheek warmth and a temple shadow, both crisp steps.

**Texel density.** Round 7's body islands are 35 px/m, which is the correct density for a flat
colour and far too coarse for any of the items above — at 35 px/m the hem lip is one pixel. **Every
region that carries a feature in this list needs the density to resolve it, and the report must
state the density it gave each region and why.** The map may grow to **512 x 512** if the packing
needs it; say so if it does.

**What must not change:** the approved palette stays the base (`#E9D2BB, #5E4632, #FFFFFF, #5F7A6A,
#474B41, #A9B2AC, #8B6B50, #6B5B49, #34271C, #F0A63C`), value steps on top of it are yours, crisp
steps not gradients, one material, one image, backface culling on, Specular IOR Level at its `0.5`
default. **The texture node's interpolation must be `Closest`** — the asset contract requires
NEAREST filtering in the GLB and the exporter does not force it for you, because filtering changes
every pixel of the look and is not the exporter's call to make silently.

## 3. Round 7's settled facts — inherit these, do not rediscover them

Its report earned these; a round that re-derives them has wasted the money that found them.

- **The hair's front edge in profile is `y = +0.111`** (`side-left.png` col 71), not the full depth
  of the head. Run the side lobes forward and the profile becomes a featureless brown slab with no
  forehead, brow or nose. This is in no depth table.
- **Pieces within one object must overlap, never meet exactly.** Coincident rims weld under
  `remove_doubles` and produce edges with four faces, which the exporter rejects as non-manifold.
- **The bare neck's lever is the STRAP, not the hair.** In an orthographic side view the outermost
  |x| wins, so anything shoulder-borne outboard of the neck erases it. Round 7 delivered 18 px of
  bare skin (0.027 H) by dropping the strap onto the sleeve cap and raising the head's floor.
- **A buried-face cull must not assume closed solids.** Round 7 lost nine faces of a hand to a
  ray-parity containment test run repeatedly against occluders it had itself opened. Its
  replacement — a six-direction enclosure test that fails safe — cost a full rebuild to find. If
  you cull, use that, and run it **once**, over closed parts.
- **The pickaxe reads edge-on from the front because a carried pick hangs in the sagittal plane.**
  It is not a defect and it is not yours to fix in the mesh — see §4.

Still-live defects in the inputs, unchanged from round 7 and **not yours to chase**:

- **Check 4 (silhouette step density) is WITHDRAWN.** A scalar that noise can satisfy was satisfied
  by noise. Do not measure it.
- **Check 5 (the bare neck) is WITHDRAWN.** Round 7 §7.2 settled what it was really measuring.
- **The pickaxe's four authorities disagree by 2.3x.** Round 7 chose `front.png` (shaft 0.91 H,
  blade span 0.38 H, head-span/length 0.42) and stated it. Keep that choice.
- **`dwarf-model-sheet.md` keeps a superseded video-derived table above the orthographic one.** The
  orthographic numbers win. **The sheet's printed annotations are unreliable in general; the
  drawings are orthographic and trustworthy.**
- **`session_tokens.py`'s Windows slug defect is still open.** Use `--transcript`.

## 4. Half B — the rig

**Nineteen joints, and the names are the contract** because the game binds by name:

```
root  hips  spine  chest  neck  head
shoulder.L/R  elbow.L/R  hand.L/R
hip.L/R  knee.L/R  foot.L/R
beard
```

- **One armature, in the round's collection**, named `SK_VoxelDwarf_Miner01_r8`. The hierarchy is
  yours; the names are not.
- **ONE mesh, RIGID weights.** Wolf's ruling from round 3 stands: every vertex belongs to **exactly
  one** joint at weight **1.0**. No soft skinning, no separate limb meshes. Soft weights would not
  fail to render — they would quietly smooth the joints of a hard-edged figure, which is why the
  exporter now counts them (§5).
- **Round 7's edge loops are already at all fifteen sites** (its §6 lists the z of each and which
  parts carry the ring). The `head` joint's loop deliberately sits on `r7_neck` rather than on both
  parts, because cutting the head's `+Y` quad would split the face's texture island. **Keep that.**
- **The mesh ships in the NEUTRAL stance. The pose is a pose.** Round 7's §12.1 is right that the
  reference's two-handed carry across the body is the biggest remaining likeness gap — and the right
  place for it is now available, because a rig can hold it. So: **the GLB carries the rest pose and
  no animation**, and the pose is delivered as renders (§7). Do not bake it into vertices.
- **Prove the weights hold.** A rig nobody posed is a rig nobody has checked. Bend each joint group
  and render it: shoulders and elbows, hips and knees, neck and head, the beard. Tearing, a hole at
  a joint, or geometry that follows the wrong bone is a defect to fix, not to report.
- **Geometry may change in half B** where a joint cannot deform without it — an extra loop, a split
  that lets the arm clear the body in the carry. Say what you changed and why, and re-run half A's
  60 px readability strip afterwards: a loop through the face island would cost the face.

## 5. What changed on the orchestrator's side — all of it verified today

**`check_asset.py` now PASSES this asset, so from this round its output is a GATE and not a quote.**
Round 7 was told to report it verbatim pass or fail because it was broken in three clauses. It is
not any more. Verified against round 7's own `.blend`, re-exported here:

```
FIGURES src-assets/export/SM_VoxelDwarf_Miner01.glb size_m=0.8x1.2x0.7 min_y_m=0.000000
  centre_x_m=0.000000 centre_z_m=0.000000 palette=<28 colours> tris=892 verts=1526
  mesh=SM_VoxelDwarf_Miner01_r7 profile=painted-map
exit 0
```

What was changed to get there, and what it means for you:

- **The palette clause understands a painted map.** It asks whether the embedded image *is* a cell
  atlas — square, a whole number of 16 px cells, every cell one flat colour — rather than whether it
  is 64x64. A painted map is read as a **census: every colour it carries, most-painted first**, and
  `profile=painted-map` is printed so the skip is never silent. Your map will report ~30 colours;
  that is the figure Wolf reads against the sheet, so **no stray colours**.
- **The voxel-only clauses no longer apply to this family.** The 0.0125 m grid clause and the
  quad-soup clause describe the generated-voxel pipeline. You are off-lattice by construction and
  you share vertices. They are inapplicable, not waived — the profile says which contract ran.
- **The naming clause accepts the revision.** `SM_VoxelDwarf_Miner01_r8` inside
  `SM_VoxelDwarf_Miner01.glb` is legal; any other disagreement still fails. The mesh name is now
  printed as `mesh=`, so a stale export announces itself.
- **`REV` is `r8`**, so **your collection must be named `SM_VoxelDwarf_Miner01_r8`** and every
  datablock carries `r8`: object, mesh, material `M_VoxelDwarf_r8`, image `r8`, armature
  `SK_VoxelDwarf_Miner01_r8`.

**The exporter now supports and gates the rig** — verified on two synthetic rigged fixtures, one
complete and one deliberately short of joints:

```
  rig   joints 19   missing joints 0   unexpected joints 0   unweighted verts 0
        soft-weighted verts 0   verts weighted to a non-bone 0
        joint names in the GLB: beard, chest, elbow.L, ... spine
```

- the skin is exported (`export_skins=True`) and the **joint names are read back out of the written
  file**, not out of the scene: a rig that stops at the `.blend` fails the build;
- an **Armature modifier no longer counts as an unapplied modifier** — it is the one modifier whose
  presence is correct, and it made the fixture fail the topology gate before this change;
- **an unrigged export still works** (the line reads `rig <NONE -- unrigged figure>`), which is what
  makes half A exportable before half B exists.

**Three silent-failure traps were closed, all of which had already fired:**

- **Your texture must be PACKED into the `.blend`.** Round 7's image `r7` pointed at
  `D:/Workspace/.../T_VoxelDwarf_r7.png` with nothing packed, so off the authoring machine the
  glTF exporter dropped the material's texture **entirely** — no images, no sampler, no
  `baseColorTexture` — while the export printed `texture image r7` and exited 0, because that line
  reported the datablock's *name*. The image is now packed (and its path made relative) and the
  exporter **fails the build if the GLB carries no image**. The line now reads
  `texture image r7   in the GLB: T_VoxelDwarf_r7`.
- **The GLB's sampler now declares CLAMP_TO_EDGE.** Round 7's declared no wrap mode at all, which
  glTF defaults to REPEAT — a bleed hazard at the border of a packed island set. The exporter forces
  it; you do not need to.
- **`seat_on_origin` was wrong for any part carrying an object transform**, and the rig is what
  would have exposed it. It measured bounds in world space, added the shift to *local* vertex
  coordinates, and zeroed `ob.location` — correct only when the object transform is already
  identity, which round 7's parts happened to be. A fixture built with
  `primitive_cube_add(location=...)` exported geometry **0.75 m off the floor while the export line
  printed `min Z 0.000000`**, because the line measured the scene before the location was discarded.
  The transform is now baked into the vertices, and **a seating gate reads the GLB's own POSITION
  bounds** and fails the build unless min Y and centre X/Z are zero. The line
  `GLB min/max` prints them. **This is a real risk for you**: place parts however you like, but the
  figure is seated from the artifact now, not from the intention.

One command, from the repo root, unchanged:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/export_dwarf.py

## 6. The limits — same three as round 7

1. **4,000 triangles for LOD0**, a build gate. Round 7 spent 892. Half B may need a few more for
   deformable joints; the headroom is 4.5x and the budget is a ceiling, not a target.
2. **Topology quality, enforced by the exporter:** triangles and quads only, manifold, no loose or
   degenerate geometry, consistent winding, a UV layer, no unapplied modifiers (bar the Armature
   one). Judged in review, not by machine: **no interior geometry**, **no overlapping UV islands**
   bar deliberate mirrored pairs, **edge loops at the joints**.
3. **One material, one texture map.** 256 x 256, or 512 x 512 if §2's densities need it.

No style clauses. No axis-aligned clause, no flatness clause, no silhouette metric. Round 7 chose
hard-edged flat faces with all of that lifted and declared the choice; **that choice is settled and
is not being re-litigated in either direction** — build what the reference shows.

## 7. It must be watchable — unchanged

- **Author INSIDE Wolf's running Blender through the MCP addon. Never in a `blender --background`
  subprocess.** Only the export and the render passes run headless.
- **Prove it before your first edit:** query the live scene and report what was already in it. **If
  you cannot reach the live instance, stop and say so.**
- **One part per tool call**, named for the outliner (`r8_*`). **Save the `.blend` after every part**
  and write a numbered progress render. The `.blend` accumulates: r5, r6 and r7 are still in it and
  stay there, excluded from the view layer.
- **No generator script.** No `dwarf_r8.py`, no part-spec file as the real source. Each call performs
  one modelling operation on the live scene. The `.blend` is the deliverable.
- **Every swappable feature stays its own named object** — beard, hair, pack, belt, boots, tunic,
  tools — because the endpoint is *hand-author the parts, generate the combinations*. That costs
  nothing now and is a rebuild later.
- **Do not stop and wait for approval.** Work through both halves and deliver.

## 8. Renders

Copy `render_r7.py` to `render_r8.py` and keep its two passes — flat Workbench for measurement,
EEVEE key-lit with one sun and a shadowless fill for judgement. **Absolute paths, and the revision
in every path** (`src-assets/renders/r8/`): Blender resolves a relative `render.filepath` against
the `.blend`, and a fixed path across rounds once made a round analyse the previous round's images.

New this round: **the posed renders** (§4) — the two-handed carry from the front and three-quarter,
plus one per joint group showing the weights hold.

## 9. Deliverables

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
| 9 | the body beside the sheet, matched scale, for §2 | `src-assets/renders/r8/vs-body.png` |
| 10 | the posed carry, front and three-quarter | `src-assets/renders/r8/pose-carry-*.png` |
| 11 | one deflection render per joint group | `src-assets/renders/r8/joint-*.png` |
| 12 | your report, covering every §2 item and all of §4 | `src-assets/prompts/dwarf-miner-round-8-report.md` |
| 13 | the exporter's output and the checker's output, verbatim | in the report |
| 14 | your cost — `session_tokens.py --transcript`, model from your banner, row `dev-art` | in the report |

## 10. How this round is judged

By Wolf's eye on the ten items in §2 — **that list is half A**. Then mechanically:

- **`check_asset.py` exits 0.** It is a gate now, not a quote.
- the export passes: ≤ 4,000 triangles, topology clean, one material, one image, the seating gate,
  and the rig gate at **19 joints / 0 missing / 0 soft-weighted / 0 unweighted**
- every §2 feature still readable at 60 px
- the posed renders show no tearing and no hole at any joint
- nothing written outside `src-assets/`, and **no git command run at all** — the operator commits

## 11. What is NOT in this round

Game lighting is Epic 11's. Decimation and LODs come after the look is signed off. The combination
layer — a dwarf as a seed, twelve discrete axes plus four continuous dials — is a separate task
after sign-off and **is not yours to start**; all this round owes it is the named-part decomposition
in §7. Animation clips are not in this round either: the rig ships posable, with no animation in the
GLB.
