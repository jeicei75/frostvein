# Round 9 — build it like a low-poly character: base mesh first, details carved after

**Round 8's figure is right in content and wrong in construction. This round changes HOW it is
built: one base mesh, blocked out whole, shaped, then carved — not fifty-four boxes stacked
together.**

Wolf, 2026-09-13, on the delivered r8: *"looks like we have visible horizontal seams now"*, and
*"it still looks like extruded paperdoll more than model in video reference"*, and then the
instruction that decides this round: *"create base mesh and start carving details with Blender
modelling best low poly workflows"*.

---

## 1. The defect, measured — one cause behind both complaints

Every edge in r8 binned by orientation:

| | horizontal edge length | vertical |
|---|---|---|
| **whole figure** | **79.6 %** | 19.0 % |
| `r8_hair` | **93.0 %** | 5.8 % |
| `r8_overtunic` | 92.5 % | 6.7 % |
| `r8_skirt` | 90.5 % | 9.5 % |
| `r8_skull` | 88.5 % | 9.5 % |
| `r8_beard` | 83.0 % | 16.5 % |

A plain axis-aligned box is already two-thirds horizontal by edge length, so **two-thirds is the
baseline, not zero** — but at 79.6 % overall and 93 % on the hair, nearly every division r8 added
runs level. That is the extruded-paperdoll signature as arithmetic: **the form changes in stacked
slices up Z**, which is what a front silhouette pushed backwards looks like. The seams are the same
defect seen from the front — **the layers ARE the seams.**

**The cause is the construction method, and the previous brief prescribed it.** Round 8 said "one
part per tool call" and asked for "a crown that steps", "an overtunic over an under-layer with a
real edge in geometry", "a hem lip", "a sole, a heel, a toe cap and a cuff". Read literally — fairly
— that produces one primitive per feature, each finished in isolation and stacked on its neighbour.
Fifty-four boxes overlapping cannot express a curved cranium or a rounded shoulder, and every
boundary between two of them is a visible line.

## 2. How to build it — the workflow, and it is the round

Standard low-poly character practice. **Do the stages in order, and do not start a stage until the
previous one reads right.**

### Stage 1 — reference planes, then a base mesh of the WHOLE figure

- Load `dwarf-ortho/front.png` and `side-left.png` as image planes, scaled so the figure is
  **1.200 m** tall, and model against them.
- **One primitive, extruded into the whole figure**: a cube for the torso, extruded up into the
  neck and head, out into shoulders and arms, down into hips and legs. **Use a Mirror modifier and
  model one half** — see §4, mirrors may now stay live.
- **No features at all.** No brow, no nose, no hem, no buckle. Masses only.
- **Target 200–500 triangles for the whole block-out**, then stop and render it from front, side,
  three-quarter and back against the planes. **The silhouette is the deliverable of this stage.**
  If it is wrong here, nothing added later fixes it.

### Stage 2 — the form pass: loops, taper, round

- **Loop cuts where the form changes**: chest, waist, hips, knee, elbow, wrist, ankle, brow, jaw,
  crown. Then **move and scale those loops** to taper the limbs, narrow the waist, curve the
  cranium back, round the shoulder and the upper back.
- **The side profile is as binding as the front.** r8 was built from the front and given depth
  afterwards, which is literally an extrusion. A mass of constant depth top-to-bottom is wrong
  unless the reference draws it that way.
- **Curvature comes from several small steps, not one big one.** The sheet resolves 8.571 mm per
  source pixel (140 px = 1.00 H), so a curve drawn over three pixels is a **~26 mm** step. r8
  stepped 50–100 mm at a time, which is why its masses read as plates.
- The loops at the fifteen joint sites are the rig's deform loops. Put them where §6 needs them.
- **Still no features.** End this stage with a render set and the edge-orientation table (§7).

### Stage 3 — carve the features OUT of the surface

- **Inset and extrude**, never stack: the brow ridge is an extruded band of the forehead, the nose
  is extruded from the face, the eye sockets are inset, the hem lip is an extruded rim of the
  skirt, the collar an inset of the neck opening, the cuff an extrusion of the sleeve end.
- **Bevel the silhouette edges that should read** — a chamfer at 8–26 mm catches light and reads at
  60 px where a hard corner does not.
- **A horizontal division must be structural or it must go.** Keep the belt, the boot cuff, the
  sole — the reference has those, ringing the figure. Everything else that ringed r8 (the hair's
  stacked crown, the overtunic's chest band, the skirt's bands) is either re-cut to run **vertically
  or diagonally** — beard locks, hair locks, folds — or removed.

### Stage 4 — separate objects, and ONLY for what must swap

**The body is ONE continuous mesh**: torso, arms, hands, legs, neck, head, and the head's carved
features — brow, nose, sockets, cheekbones, ears. Not one object per feature.

Separate named objects, because the per-person combination layer swaps them: **hair, beard,
moustache, belt + buckle, pack + flap + bedroll, straps, lantern, pickaxe, boots.**

**This reverses one thing round 8 was told.** Round 8 made the brows, eyes and nose their own
objects "so they can swap later", and that is why the seams exist: a nose that is a separate box has
a boundary, a nose extruded from the face does not.

**But carving them in does NOT cost the combination layer, because a feature can be named inside one
mesh.** Wolf's point, 2026-09-13, and verified in this Blender rather than assumed:

- **Vertex groups work and survive the exporter's join** with their names and membership intact.
  This is the mechanism to use: `Select -> Select All by Trait -> Vertex Group` finds the region
  again, and the group is visible in the UI.
- **FACE-domain named attributes also work** (`mesh.attributes.new("feature_nose", 'BOOLEAN',
  'FACE')`) and survive the join — the right choice when the region is better described by faces
  than by vertices.
- **Face maps do NOT exist any more** — removed in Blender 4.x, confirmed absent in 5.2. Do not
  reach for them.

**So tag every carved feature.** Name the groups `feature_*` — `feature_nose`, `feature_brow.L`,
`feature_hem` — and the exporter reports them on its own line (`feature tags N vertex groups, M face
attributes`). It is reported, never gated: an untagged figure still exports, it is just a figure the
combination layer cannot take apart.

**And this is the part that is free now and a rebuild later: give each swappable feature a STABLE
BOUNDARY LOOP.** Tagging finds a region; swapping it needs its rim to be a fixed ring of vertices at
known positions, so a variant can be stitched into the same hole. Carve each feature so its
surrounding loop is a clean ring you would be willing to freeze, and say in the report which loops
you intend as the swap boundaries.

**One trap was removed from the exporter today so this is safe:** the rig gate judged rigid weights
over EVERY vertex group, so a vertex carrying `head` at 1.0 plus `feature_nose` at 1.0 read as
**soft-weighted and failed the build** — which would have pushed you to delete your own tags to get
an export. Only bone-named groups count as weights now. Verified on a fixture whose vertices carry
both.

### Stage 5 — UVs and paint

Unwrap after the form is final, mirror-share the symmetric halves, and re-author the islands the new
topology needs. Keep r8's density decisions where the masses did not change; **the face keeps its
high-density island**, and say what you gave each region. One material, one image, `Closest`
interpolation, backface culling on, Specular IOR Level at `0.5`.

### Stage 6 — rig, weights, deflection

As r8, and its rig is a good rig: 19 joints, contract names, **rigid weights at exactly 1.0**, no
animation in the GLB, the carry delivered as renders. One continuous body mesh makes this easier
than r8's did — the deform loops are already in the surface from stage 2.

## 3. The reference is the authority for construction — look at `f088` first

`src-assets/references/dwarf-frames/f088.png`, above the orthographic sheet for *form*:

- curvature from **many small steps** — the cranium curves back, the shoulder and upper back are
  round, each a run of steps advancing in **two axes at once**, never a plate on a plate;
- masses **taper and tilt** — the sleeve narrows to the cuff, the boot is a wedge, the tunic's hem
  runs at an angle, the forms sit off-axis;
- divisions that ring a mass are **few and structural**; the rest of the detail runs vertical or
  diagonal — locks, folds, straps.

## 4. What was limiting you, and is not any more

**The exporter forbade every modifier and measured the wrong mesh.** Fixed 2026-09-13, verified on a
fixture built for this workflow:

- **`triangles` and the topology gates now read the EVALUATED mesh** — the geometry the GLB will
  actually carry. Before, they read the cage, so a live Subdivision made the triangle figure a lie
  and the gates inspect geometry that was never exported.
- **`flatten` applies each part's modifier stack BEFORE the join.** `object.join()` keeps only the
  active object's modifiers, so a Subdivision on any part but the first was silently discarded: the
  export printed `live modifiers r8_torso:SUBSURF` and wrote a GLB with none in it.
- **The "unapplied modifiers" gate is gone**, replaced by a reported `live modifiers` line. It had
  to exist while the counts read the cage; it does not now, and it was the thing that made an
  iterative cage-and-refine workflow impossible.
- **Seating reads the evaluated bounds**, so a Catmull-Clark surface that shrinks inside its cage
  still lands on the floor.

**So: Mirror, Subdivision and Bevel may stay LIVE on the parts while you work, and are applied on
the way out.** Model one half with a Mirror. Keep a subdivision cage adjustable. The only modifier
the exporter removes rather than applies is Armature, because applying it would bake the rest pose
and throw the skin away.

Measured on the fixture: a torso with a live Subsurf level 2 exported **204 triangles** and 16,108
bytes, against the cage's 24 triangles and 3,496 bytes before the fix.

## 5. The budget is not your constraint

`TRI_BUDGET` is **30,000** and r8 spent **3,955** — 13 %. A properly looped and rounded low-poly
character at this scale lands somewhere around **8,000–15,000**; if it reads like `f088` at 20,000,
spend it. Wolf's standing ruling: *"it's easier to optimize than add more details later on"*, and a
decimation pass can take it back later.

**Being thrifty is the failure mode this round.** A mass that could curve and does not is the defect.

## 6. What to keep from round 8 — do not re-derive any of it

- **The content is right.** Every feature on r8's list stays: brow, eyes, nose, moustache, tapering
  beard, ears, layered tunic, shoulder pieces, hem, waist panel, belt with buckle, boot sole and
  cuff, pack with flap and bedroll, lantern with frame and glass, pickaxe with bindings.
- **The rig**, and its finding: a **buried-face cull must be scoped BY JOINT** — a face buried inside
  a part on a *different* joint is exposed the moment that joint bends. With one continuous body
  mesh there is far less to cull.
- **The settled facts:** the hair's front edge in profile at `y = +0.111`; pieces within one object
  must overlap and never abut exactly; the bare neck's lever is the strap; the pickaxe reads edge-on
  from the front because a carried pick hangs in the sagittal plane; the lantern flame is a COLOUR
  and never an emitter.
- **The gates are gates.** `check_asset.py` exits 0 today and must still exit 0; the export's
  topology, seating and rig gates are green and must stay green.
- **`REV` is `r9`** — collection `SM_VoxelDwarf_Miner01_r9`, every datablock carrying `r9`, and the
  export exits with `collection ... not found` until you create it, which is expected. **Do not edit
  `REV`.** r5–r8 stay in the file, excluded from the view layer. **PACK the texture image.**

## 7. The instrument — check yourself instead of guessing

Bin every edge by orientation and report the §1 table for your own figure:

```python
horiz = math.degrees(math.asin(abs((b - a).normalized().z)))   # < 30 deg = horizontal
```

**Target: the whole figure under 70 % horizontal, and no mass above 80 %.** It is a check, not a
score: it is satisfied by looping and rounding masses the way the reference does, and it can be
gamed by chopping a shaft into rings — which round 6 already did to a different scalar. **If the
number moves and the figure does not look more like `f088`, the number is lying and the figure is
the truth.** Report it per stage, so the form pass can be judged before features land.

## 8. It must be watchable — unchanged in principle, changed in grain

- **Author INSIDE Wolf's running Blender through the MCP addon**, never a `--background` subprocess.
  Prove it first by reporting the live scene. Only the export and renders run headless.
- **One OPERATION per tool call** — an extrude, a loop cut, a loop moved, an inset, a bevel — not
  one finished part per call. That is the change: r8's grain was "a part per call", which is what
  built it out of separate finished boxes.
- **Save the `.blend` after every call and write a numbered progress render at every stage
  boundary**, plus the stage's render set. Four of six delegated runs in this project were killed by
  the harness; the saves are the insurance.
- **No generator script.** The `.blend` is the deliverable. **No git commands at all** — the operator
  commits. **Do not stop and wait for approval**: judge each stage against the reference planes
  yourself and carry on.

## 9. Deliverables

| # | what | path |
|---|---|---|
| 0 | the live-scene report | in the report, before any geometry |
| 1 | the source, saved per call, image PACKED | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture map | `src-assets/blender/textures/T_VoxelDwarf_r9.png` |
| 3 | the render script | `src-assets/blender/render_r9.py` |
| 4 | **a render set per STAGE** — block-out, form, carved, textured | `src-assets/renders/r9/stage-N-*.png` |
| 5 | progress renders | `src-assets/renders/r9/progress/NN-<operation>.png` |
| 6 | five final views, flat and key-lit | `src-assets/renders/r9/` |
| 7 | **beside `f088` and `f104`, matched scale** | `src-assets/renders/r9/vs-frames-*.png` |
| 8 | **the side profile beside `side-left.png`** | `src-assets/renders/r9/vs-ortho-side-left.png` |
| 9 | the readability strip at 100 px and 60 px | `src-assets/renders/r9/readability.png` |
| 10 | the posed carry, and one deflection render per joint group | `src-assets/renders/r9/pose-*.png`, `joint-*.png` |
| 11 | the per-part census: name, faces, tris, size | in the report |
| 12 | **the edge-orientation table, per stage and per mass** | in the report |
| 13 | your report | `src-assets/prompts/dwarf-miner-round-9-report.md` |
| 14 | exporter and checker output, verbatim | in the report |
| 15 | cost — `session_tokens.py --transcript`, row `dev-art` | in the report |

## 10. How this round is judged

By Wolf's eye against `f088` — **does it read as a model rather than as layers?** Then:

- **the body is one continuous mesh**, and the census shows it
- **the horizontal banding is gone**: no stacked-plate crown, no chest band, no skirt bands
- **under 70 % horizontal edge length overall, no mass above 80 %**
- **the side profile matches `side-left.png`**; no mass is a constant-depth extrusion
- the export passes every gate, `check_asset.py` exits 0, the rig gate reads 19 / 0 / 0 / 0
- the posed renders show no tearing at any joint, and every feature still reads at 60 px
- nothing written outside `src-assets/`, and no git command run

## 11. What is NOT in this round

The pose and animation clips (the rig ships posable, no animation in the GLB). LODs and decimation.
Game lighting, which is Epic 11's. The per-person combination layer, which is a separate task after
the look is signed off — all this round owes it is stage 4's named swappable objects.
