# Round 13 — add the form: you inherit round 9, and you spend everything on features

**You are NOT starting from an empty file.** `SM_VoxelDwarf_Miner01.blend` already holds
`SM_VoxelDwarf_Miner01_r13` — round 9's figure, renamed. It passes every gate in this document
today, unchanged. Your job is to put form into it where it has none, and nothing else.

This reverses round 12's from-scratch rule, deliberately, on a measurement. Read §0 before anything.

---

## 0. Why the rule changed, and what the number really was

Round 12 built from nothing to a careful specification, reported honestly, and passed every gate:
19,064 triangles, **3,288 planes of form** against a 2,500 target, zero holes, rig clean. Wolf's
verdict was *"decent if we want a really low low poly model .. but it's lacking detail .. boxy look
especially in side .. missing feet"*.

**The metric was wrong, and I wrote it.** A "plane of form" is a distinct face normal, and round 12
reached 3,288 of them with a three-segment **Bevel** — its own report says so plainly: the cage at
1,418 faces carried **111 planes**, and bevelling took it to 3,273. Measured from the delivered GLB:
**84.5 % of its triangles are chamfer slivers carrying 9.6 % of the surface**, at a median width of
**1.32 mm** where §5 had asked for 8–26 mm. One source pixel of the reference is 8.571 mm, so those
chamfers are a sixth of a pixel. They move the counter and catch no light.

Counting planes **only on faces at least one source pixel (8.571 mm) wide** — the one measure that
survives subdivision *and* bevel — gives this:

| round | faces | **planes ≥ 8.571 mm** | raw planes | slivers | the verdict it got |
|---|---:|---:|---:|---:|---|
| r7 | 446 | 24 | 24 | 3 % | head one box, whole face one quad |
| r8 | 2,120 | **441** | 445 | 7 % | *"now it finally starts to look really what it should"* |
| **r9** | 5,530 | **1,436** | 2,156 | 49 % | *the form Wolf liked*; still *"away from reference images"* |
| r10 | 13,592 | **1,408** | 3,257 | 58 % | wrecked by its own tooling, not by its form |
| r11 | 12,624 | **267** | 319 | 38 % | *"the look is not quite close to what I want"* |
| r12 | 9,752 | **143** | 3,292 | 85 % | *"lacking detail .. boxy .. missing feet"* |

**Round 12 has 143 resolvable directions in it. Round 9 has 1,436.** The planes column tracks Wolf's
eye in every round; the raw column **inverts** it — r12 raw 3,292 against r9's 2,156, for the figure
he called the less detailed of the two. This is the third gameable scalar I have written in three
specs — a silhouette step at r6, millimetres of face edge at r11, a raw plane count at r12 — and the
lesson taken is that a figure-wide scalar is the defect, not the particular scalar. §3 replaces it.

**The second measurement, and the reason for the reversal.** Each round has produced roughly
3,000–5,000 triangles of real form and then run out, because ~600 operations is what a sitting holds:

| round | turns | cost | resolvable planes placed |
|---|---:|---:|---:|
| r9 | 658 | $137.72 | 1,436 |
| r10 | 585 | $131.67 | 1,408 |
| r11 | 533 | $106.72 | 267 |
| r12 | 595 | $113.94 | 143 |

Every round has spent between a third and a half of that budget re-deriving a blockout the previous
round already had — round 12's stages 1 and 2 produced **276 triangles** — and has then reached the
face and the hands with nothing left. That is why round 12's face got 121 quads and why it has no
hands at all. **Nothing has ever accumulated.** This round accumulates: you start at 1,436 and every
operation you spend goes onto a feature.

**Round 12 is not lost.** It is in git at `d137b4d` with its report and its full render set, and its
UV and texture work is the standard §5 asks you to match. It is simply not the better base.

---

## 1. The subject, the references, the measurements

A dwarf miner, **1.200 m tall**, stylised low-poly, seen at distance in play and in close-up for
marketing. Four inputs, each authoritative for a different thing:

| input | authoritative for |
|---|---|
| `references/dwarf-ortho/` (front, side-left, side-right, back) | proportion, landmark heights, widths, depths — **read its README first** |
| `references/reference-sheet.jpg` | the palette and the gear breakdown; **its printed annotations are not trustworthy** |
| `references/dwarf-frames/` (`f084`–`f164`) | **form, construction and lighting** — `f088` is the form authority, `f104` the clearest near-front face |
| `references/dwarf-model-sheet.md` | the measured tables; its **orthographic** table is the authority, the top one is superseded |

**The scale that governs everything:** the figure spans rows 7–147 in `front.png` and
`side-left.png`, so **140 source pixels = 1.00 H and one source pixel is 8.571 mm**. `front.png`
column 77 is the centre line, `side-left.png` column 58 the depth centre, **+Y is forward**, the
figure's right is **+X**, the sole sits at `z = 0`.

**The landmark table is settled. Do not re-derive it.** It is §2 of
`src-assets/prompts/dwarf-miner-round-12.md` and the figure you inherit already lands on it — round
9 was built to it and round 12 cross-checked three of its numbers against the side view. Read it,
use it to judge what you add, and spend no turns re-measuring the references.

**Two measurements that are in no table and were expensive to find:**

- **the hair's front edge in profile is `y = +0.111`** (`side-left.png` col 71). Run the side lobes
  forward past this and the profile becomes a featureless slab with no forehead, brow or nose;
- **the neck is visible only if nothing shoulder-borne is outboard of it** — in an orthographic side
  view the outermost |x| wins, so a pack strap at the shoulder erases the neck. Keep it on the
  sleeve cap.

**Known defects in the inputs. Do not rediscover them and do not obey them:** the sheet's printed
labels contradict its own drawings; the pickaxe has four conflicting authorities spanning 2.3× and
round 7's choice from `front.png` stands (shaft 0.91 H, blade span 0.38 H, head-span/length 0.42);
`gear.png` and `front.png` are at different scales with no stated conversion; the ortho crops carry
the sheet's own 1-px dimension rules on the crown and sole rows; and **the sheet is POSED** — its
arm span and two-handed carry are properties of the pose, not the body (§4).

---

## 2. What you inherit, and exactly what is wrong with it

Open the file and look at it first. What is there:

```
collection SM_VoxelDwarf_Miner01_r13
  r13_body  r13_beard  r13_moustache  r13_hair  r13_belt
  r13_boots r13_pack   r13_straps     r13_lantern r13_pickaxe
  SK_VoxelDwarf_Miner01_r13   (19 joints, rigid weights, 0 unweighted, 0 soft)
collection ref_r13
  ref.front  ref.side   (reference image planes)
  r13cam.front/side/quarter/back/head/headq/boot/bootq/bootside
```

It exports clean today — I ran it: **8,268 triangles of 100,000, 1,436 planes of form (2,156 raw, 49 % slivers), 0 exposed
holes, no inside-out mass, topology clean, rig clean, `check_asset.py` exit 0.** `r13_body` carries
a live **Mirror**; keep it, it halves your work and cannot deform anything.

**The defect list, in the order I would spend on it.** Every item is something the reference has and
this figure does not, or has wrong. Judge each against `f088` and `f104`, not against my words.

1. **The ears are flanges and they are the worst thing in the figure.** They stand **93 mm outboard
   of the cheek plane**, horizontally, at eye level — in the quarter view they read as wings through
   the skull. Tuck them to the head: an ear is a shallow shell on the side of the skull with a rim
   and a hollow, projecting a few millimetres, top at `0.850 H` and bottom at `0.729 H`.
2. **The nose projects 4 mm past the cheeks.** It needs to be unmistakable in profile — the table
   puts the nose tip **0.379 H** past the back of the head against a head depth of 0.336 H, so it
   stands roughly **50 mm** proud. It already has a bridge and shaded sides; it needs to come
   forward and to keep them.
3. **There are no feet.** The boots are a mass with a cuff and no sole break, no heel, no toe cap and
   no toe spring. Sole length is **0.214 H** with the toe projecting 0.057 H past the shin front.
4. **The hands are crude** and the close-ups land on them. They want a wrist, a palm with a back and
   an edge, a thumb, and fingers as a stepped block — not four separate fingers, but a shape that
   reads as a fist-sized hand with knuckles.
5. **The body resolves at a 38 mm median face edge while the boots resolve at 6.4 mm** — a 6× spread
   that is an accident of authoring order. The torso, the arms and the skirt want form at the same
   scale as the rest: chest and back planes, a waist that narrows in two axes, shoulder and upper-arm
   rounding, folds in the skirt, a hem lip.
6. **The side profile is the weakest view in every round so far.** Judge it against
   `side-left.png` directly: forehead, brow, the nose, the beard's front, the crown steps, the
   pack's depth. A mass of constant depth top-to-bottom is wrong unless the reference draws it so.
7. **The props are not in the hands** — the pickaxe floats free beside the figure. Put it in the
   right hand and the lantern in the left, contacting the geometry, so the neutral pose reads.
8. **The texture is 256 × 256 with 87 % of its texels a single flat fill**, and its flame carries a
   **20-step gradient ramp** (`#BC8234` → `#FFE8B4`) where §5 requires crisp steps. It is replaced
   in stage E.

**What is already good and must not be undone:** the brow ridges, the eye sockets with lids and
whites, the cheekbones and jaw, the beard's volume and steps, the hair's faceted silhouette, the
skirt's folds, the A-pose and the rig. Round 9's face is the best face this project has produced.
You are adding to it, not replacing it.

---

## 3. The metric, and why there is no triangle target

**The figure is judged on how many directions its surface faces — counted only on faces that a
viewer could resolve.**

A *plane of form* is a distinct face normal bucketed at about one degree, counted **only on faces
whose shortest edge is at least 8.571 mm**, one source pixel of the reference. A face narrower than
one reference pixel cannot be a feature of the reference. This is immune to subdivision, which splits
a flat face and adds no normal, and to bevel, whose slivers fall under the threshold.

- **you inherit 1,436. The target is 3,000. The export fails below 1,200.**
- The floor is set low on purpose: its job is to catch a figure that lost form, not to be tuned to.
- The export also prints the raw count and the sliver fraction beside it, so inflation is visible.

**THERE IS NO EXPECTED TRIANGLE COUNT AND `TRI_BUDGET` IS 100,000.** Round 12's brief said "expect
15,000–28,000 triangles" and that line is what drove the bevel: the seat could place ~3,000 triangles
of form by hand and had to inflate to show 15,000. **Spending fewer triangles is never a failure
here.** Every triangle you add must point somewhere its neighbours do not. If you finish the list in
§2 at 12,000 triangles, deliver 12,000 triangles.

**Chamfers are still wanted where §5 asks for them — at 8–26 mm, wide enough to catch light.** A
1 mm chamfer is not a chamfer, it is a counter. Bevel a silhouette edge because the edge should
read, never to reach a number.

**Feature scale, so "detail" is concrete.** One source pixel is 8.571 mm. Place a plane wherever the
reference changes direction at that scale or larger. The face's features are 1.4–5 % of figure
height (17–60 mm), so a brow ridge, an eye socket, a nose with a bridge and shaded sides, a
cheekbone and a mouth are each several planes.

---

## 4. The bind pose — unchanged, and already correct in the file

The figure is in an **A-pose**: arms out at roughly 40°, at least 20 mm of clear air at the armpit,
hands relaxed with palms toward the thighs. It is already built this way — do not close the arms to
match the sheet. Round 8 did, and its arm's inboard face sat at |x| = 0.192, exactly the torso's
half-width: zero clearance, coplanar surfaces between two objects that shimmer in the game and that
no topology gate catches, and an arm that cannot rotate without sweeping through the tunic.

Silhouette comparisons will not match the sheet at the arms. Expected. Judge torso, head, skirt and
boots against the sheet; judge the arms on clearance and form. **The props sit in neutral hands**;
the two-handed carry is a POSE, delivered as a render.

---

## 5. The work, in stages

**Do the stages in order, and run the export at every stage boundary.** It takes one command and it
is the only thing standing between you and round 10's outcome — a figure whose surfaces were missing
in dozens of places, delivered because nothing looked. The export refuses to write a figure with an
exposed hole, so a stage that ends with one is a stage that is not finished. Fix it there, while you
still remember what you cut. **Paste the `holes` and `form` lines into the report for every stage.**

**Judge each stage yourself against the reference. Do not stop and wait for approval.**

**Stage A — the head.** Ears tucked, nose brought forward, then the face densified: the brow band,
the eye sockets, the cheekbones, the jaw, a mouth under the moustache, the temple. This is the
largest single spend of the round and the close-ups land here. End with the head render set.

**Stage B — hands and feet.** Wrist, palm, thumb, knuckle block. Boot sole, heel break, toe cap,
toe spring, cuff. Then put the pickaxe in the right hand and the lantern in the left, in contact.

**Stage C — the body at one resolution.** Chest and back planes, the waist narrowing in two axes,
shoulder and upper-arm rounding, skirt folds, the hem lip, the collar, the sleeve cuffs. **Steps
advance in two axes at once** — a step that changes only z is a band, and bands are what made round
8 read as layers. **A horizontal division must be structural or it must go**: keep the belt, the boot
cuff, the sole; everything else runs vertical or diagonal.

**Stage D — the side profile.** Judge against `side-left.png` and fix what does not land: forehead,
brow, nose, beard front, crown steps, pack depth, the boot's toe.

**Stage E — UVs and paint, to round 12's standard.**

- **One material, one image**, `M_VoxelDwarf_r13` and `T_VoxelDwarf_r13`, **512 × 512**, PACKED.
- **Pack the islands properly.** Round 12 reached **67.2 % coverage** with `smart_project` at a 66°
  angle limit then `pack_islands` with rotation and concave shapes across all ten objects in one
  multi-object edit. Below ~60 % means re-pack rather than enlarge. **Report your coverage figure.**
- Round 12's texel density was **6.1–6.8 mm everywhere**; match it. Report per-region density.
- **The approved ten are the base** — `#E9D2BB` skin, `#5E4632`, `#FFFFFF`, `#5F7A6A` tunic,
  `#474B41`, `#A9B2AC` metal, `#8B6B50` wood, `#6B5B49`, `#34271C` hair, `#F0A63C` flame — and value
  steps on top of them are yours. **Crisp steps, no gradients across a part.** The 20-step flame ramp
  you inherit is exactly what this forbids.
- **Paint what geometry cannot carry**: the pupil and iris, the lip line, the waist panel's motif,
  cheek warmth, a temple shadow, cloth weave, metal wear, dirt at the hem and soles.
- **A painted feature drifts; a carved one cannot.** Wolf's words on round 12 were *"a bit misaligned
  eyes"* — its eyes were paint on a flat plane at 6.2 mm per texel over a surface with 185
  directions. The eye you inherit is a real socket. Keep it that way and let paint only colour it.
- Backface culling on. **Interpolation `Closest`.** Specular IOR Level stays at its `0.5` default —
  a `0` emits `KHR_materials_specular` and the no-extensions clause rejects the export.

**Stage F — weights and deflection.** §6.

---

## 6. The rig — you inherit it, and you only extend it

`SK_VoxelDwarf_Miner01_r13` is already built and already passes: **19 joints, 0 missing, 0
unexpected, 0 unweighted, 0 soft-weighted.** The names are the contract because the game binds by
name:

```
root  hips  spine  chest  neck  head
shoulder.L/R  elbow.L/R  hand.L/R
hip.L/R  knee.L/R  foot.L/R
beard
```

- **Do not rebuild it.** Geometry you add must be weighted, and that is the whole rigging job.
- **ONE mesh, RIGID weights: every vertex belongs to exactly one joint at weight 1.0.** No soft
  skinning. This is a style ruling, not the industry default — it keeps a hard-edged figure from
  smoothing at the joints, and its cost is a visibly hard seam at large angles, which is accepted.
- **Assign weights BY SELECTION, region by region.** Select the geometry you just added, assign it
  to its joint. **Never by a geometric test** — round 10 wrote an "is this part of the arm?"
  heuristic on distance along the arm axis, weighted the skirt to the arm bones, shredded the
  deflection renders and had to be repaired twice.
- **The props are weighted like anything else** — pickaxe to `hand.R`, lantern to `hand.L`, pack and
  straps to `chest`, belt to `spine`, hair to `head`, beard and moustache to `beard`.
- **Run the export before you rig and again after**, and compare the `holes`, `form` and
  `inside-out` lines. Rigging must not change the mesh; if the numbers move, the rigging did
  something to the geometry and that is the defect to find.
- **Prove the weights**: bend each joint group and render it. Tearing, a hole, or geometry following
  the wrong bone is a defect to fix, not to report.
- The mesh ships in the bind pose with **no animation** in the GLB.
- **DO NOT CULL BURIED FACES. At all.** Round 8's cull cost a hand and a rebuild; round 10's took
  973 faces the budget never needed and opened the crown. At a 100,000 ceiling on a figure of 8,268,
  every buried face this dwarf will ever have is free.

---

## 7. Method — what changed, and what did not

**This is the part of the specification that has been costing the most, so read it carefully.**

### What is now allowed

- **SELECTION helpers are allowed.** A function that *chooses* geometry — "select the face ring at
  z ≈ 0.88", "select the cap of the arm by `normal · axis`", "select every edge shorter than 3 mm" —
  deforms nothing and may be reused. **It must report what it selected** (count, and the bounding box
  of the selection) so the choice is verifiable before the operator runs. Round 12 lost an arm and
  had to rebuild it because it was selecting the arm's cap as "the face whose centre has the largest
  X", which after one extrusion picks the arm's *top* face. That class of error is what this
  permission is for.
- **Discrete hardware may be placed as a separate mass** rather than carved: buckles, strap fittings,
  the boot's toe cap, the lantern's frame and bail, the pickaxe's bindings. These are separate
  objects in the reference too.

### What is still banned, and why

- **No helper that EDITS geometry.** No bulk vertex moves by rule, no "setters", no bisect-by-plane
  passes, nothing in `driver_namespace` that touches vertex coordinates. Round 10 wrote those tools,
  they collapsed the pack and inverted the body, and the round was lost to repairing them. Selecting
  by rule is fine. Moving hundreds of vertices by rule is not.
- **The body is ONE continuous carved surface** — torso, arms, hands, legs, neck, head and every
  facial feature. **Carve features out of it; never stack a primitive on top of the head or torso.**
  Round 8 stacked everything and read as layers; round 9 carved and its horizontal edge fraction fell
  from 79.6 % to 61.6 % and the banding disappeared. That is why carving stays for the body, and why
  the relaxation above is limited to hardware.
- **No generator script.** The `.blend` is the deliverable. Do not write a `dwarf_r13.py` that
  rebuilds the figure and do not keep part specs in a file as the real source. Byte-identical
  regeneration was retired at round 4 and does not come back: round 3 delivered it and produced the
  worst-looking dwarf of the twelve, because a generator makes you think in loops and parameters.
- **No git commands at all.** The operator commits.
- **Nothing written outside `src-assets/`.**

### One operation per tool call

An operation is a Blender operator applied to a selection — `extrude_region`, `loopcut`, `inset`,
`bevel`, `subdivide`, a transform of the selected vertices, a modifier added. Not one finished part
per call, which was round 8's grain. A selection helper called immediately before the operator is
part of the same call.

**One rule that will bite repeatedly if ignored: pieces within one object must OVERLAP, never meet
exactly.** Coincident rims weld under `remove_doubles` and produce edges with four faces, which the
exporter rejects as non-manifold. Coincident faces between two *objects* are worse: they shimmer in
the game and no gate catches them.

---

## 8. It must be watchable, and the viewport must follow the work

**Author INSIDE Wolf's running Blender through the MCP addon. Never a `--background` subprocess.**
Only the export and the render passes run headless. **Prove it before your first edit**: query the
live scene and report what was already in it. If you cannot reach the live instance, stop and say so
— Blender never reloads a file that changed underneath it, so the work would be invisible.

**If the live file does not contain `SM_VoxelDwarf_Miner01_r13` with ten meshes and an armature,
stop and say so.** Either the wrong file is open or the base was not saved. Report the live scene's
contents first; the check costs you nothing and round 12 found exactly this.

**After every operation, make the change visible.** Round 12 ran one operation every ten seconds,
which is a comfortable watching pace — the problem was that the viewport did not follow the work:

```python
area   = [a for a in bpy.context.screen.areas if a.type == 'VIEW_3D'][0]
region = [r for r in area.regions if r.type == 'WINDOW'][0]
with bpy.context.temp_override(area=area, region=region):
    bpy.ops.view3d.view_selected()          # guard this: it throws on an empty selection
bpy.ops.wm.redraw_timer(type='DRAW_WIN_SWAP', iterations=1)
bpy.context.workspace.status_text_set("r13 · op 142 · inset nose face, 4 verts")
```

- **Drive only `areas[0]`.** Wolf keeps a second 3D viewport to navigate himself; never override into
  it.
- **Name the operation in the status bar** every call, as above.
- **Screenshot every operation** with `bpy.ops.screen.screenshot_area()` into
  `src-assets/renders/r13/live/NNNN.png`, and at the end assemble them into
  `src-assets/renders/r13/timelapse.mp4` with Blender's own FFmpeg output. `live/` is gitignored;
  the timelapse is a deliverable, and it is the record of *how* the figure was built.
- **Save the `.blend` after every call.** Four of six delegated runs in this project were killed by
  the harness; the saves are the insurance.

---

## 9. The tools, and what they will and will not accept

One command, from the repo root:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/export_dwarf.py

**`REV` is `r13`** and every datablock already carries it — object, mesh, material, image, armature,
collection. Do not edit `REV`.

**Live modifiers are allowed.** Mirror, Subdivision and Bevel may stay live and are applied on the
way out; the exporter measures the **evaluated** mesh. Only Armature is removed rather than applied.
Note that a live bevel prints its slivers in the sliver fraction, so it will not flatter you.

**Gates that fail the build** — all of them mutation-tested:

- **NO HOLE A CAMERA CAN SEE.** A boundary edge (one face where there should be two) is a hole. The
  count alone is not the gate — the exporter fires rays out of every hole and fails the build if any
  hole's rays all escape. You inherit **18 boundary edges, 0 exposed**, all sealed inside other
  masses. Zero exposed is the standard.
- **NO INSIDE-OUT MASS**: signed volume of every mass must be positive. Round 10's moustache was
  −1.34 L and its straps −2.93 L and the winding gate passed both, because it compares each face
  with its neighbours and an inverted shell's neighbours all agree.
- **PLANES OF FORM ≥ 1,200** on the joined figure, counted on faces ≥ 8.571 mm (§3).
- triangles and quads only, no n-gons; manifold; no loose verts or edges; no degenerate faces;
  consistent winding; a UV layer present;
- **≤ 100,000 triangles**;
- **the texture must reach the GLB** — an unpacked external path silently produced a GLB with no
  texture at all in round 7, so pack it;
- **seating**, read from the GLB's own POSITION bounds: min Y = 0, centre X/Z = 0;
- **the rig**: 19 joints, 0 missing, 0 unexpected, 0 unweighted, 0 soft-weighted.

**Judged in review, not by machine, because no cheap test is honest:** no interior geometry, no
overlapping UV islands bar deliberate mirrored pairs, edge loops at the joints.

**`check_asset.py` must exit 0.** It is a gate, not a quote:

    python3 scripts/bench/check_asset.py src-assets/export/SM_VoxelDwarf_Miner01.glb

It enforces one mesh / one material / one image, no glTF extensions, `doubleSided` false, NEAREST
filtering, CLAMP_TO_EDGE, UVs inside 0–1, the origin-centring clause, and a mesh name matching the
file basename bar the `_r<N>` revision. It reports the palette as a census of every colour the map
carries, so **no stray colours** — and note that round 12's report pasted a census one colour stale,
so paste the run that matches the artifact you ship.

**Feature tags.** You inherit thirteen vertex groups — `feature_nose`, `feature_eye.R`,
`feature_brow.R`, `feature_cheek.R`, `feature_ear.R`, `feature_hand.R`, `feature_crown`,
`feature_collar`, `feature_cuff.R`, `feature_hem`, `feature_overtunic`, `feature_shoulder.R`, and a
leftover `helper_arm.R`. **Tag every feature you add or rework** the same way, and drop
`helper_arm.R`. Vertex groups and FACE-domain named attributes both survive the exporter's join;
face maps do not exist any more, removed in 4.x. Tags are reported, never gated — but they are what
the next round's per-feature gate will bind to, so name them honestly.

---

## 10. Deliverables

| # | what | path |
|---|---|---|
| 0 | the live-scene report, before any geometry | in the report |
| 1 | the source, saved per call, image PACKED | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture map, 512 × 512 | `src-assets/blender/textures/T_VoxelDwarf_r13.png` |
| 3 | the render script | `src-assets/blender/render_r13.py` |
| 4 | a render set per STAGE | `src-assets/renders/r13/stage-<A–F>-*.png` |
| 5 | **the timelapse** | `src-assets/renders/r13/timelapse.mp4` |
| 6 | five final views, flat and key-lit | `src-assets/renders/r13/final-*.png` |
| 7 | the face and the hands, close up, flat and key-lit | `src-assets/renders/r13/face-*.png`, `hand-*.png` |
| 8 | beside `f088` and `f104`, matched scale | `src-assets/renders/r13/vs-frames-*.png` |
| 9 | the side profile beside `side-left.png` | `src-assets/renders/r13/vs-ortho-side-left.png` |
| 10 | **before/after against round 9, same camera, same lighting** | `src-assets/renders/r13/vs-r9-*.png` |
| 11 | the readability strip at 100 px and 60 px | `src-assets/renders/r13/readability.png` |
| 12 | the posed carry, and one deflection render per joint group | `src-assets/renders/r13/pose-*.png`, `joint-*.png` |
| 13 | **the form table: planes of form and cage faces per mass, with the sliver fraction** | in the report |
| 14 | **the export's `holes` and `form` lines at every stage boundary** | in the report |
| 15 | the UV coverage figure and per-region texel density | in the report |
| 16 | the per-part census: name, faces, tris, size | in the report |
| 17 | your report | `src-assets/prompts/dwarf-miner-round-13-report.md` |
| 18 | exporter and checker output, verbatim, matching the shipped artifact | in the report |
| 19 | cost — `session_tokens.py --transcript` (its default path is broken on Windows), row `dev-art` | in the report |

**Say in the report which of §2's eight defects you closed, which you did not, and why.** A defect
you ran out of turns for is a fine answer; a defect silently left is not.

---

## 11. How this round is judged

By Wolf's eye against `f088` and `f104` — **does it read like the reference?** Then, mechanically:

- **zero exposed holes and no inside-out mass** — the export refuses to write the figure otherwise
- **at least 3,000 planes of form** on faces ≥ 8.571 mm, up from the 1,436 you inherit
- **the sliver fraction has not ballooned** — chamfers are 8–26 mm where they exist, not 1 mm
- the ears are tucked; the nose stands ~50 mm proud; the boots have a sole, heel and toe cap; the
  hands have a wrist, palm and thumb; the props are in the hands
- **no surface coarser than 20 mm and the body is not the coarsest mass** — the 6× spread is gone
- **the UV islands cover ≥ 60 % of the map** at 6–7 mm texels, and no surface is a single flat fill
  where the reference has variation; no gradient ramps
- the body is one continuous mesh, in the bind pose, with ≥ 20 mm armpit clearance
- every feature readable at 60 px
- `check_asset.py` exits 0; the export passes topology, budget, texture, seating and rig gates
- the posed renders show no tearing at any joint
- nothing written outside `src-assets/`, and no git command run

## 12. What is NOT in this round

Animation clips — the rig ships posable with no animation in the GLB, and the two-handed carry is a
render. LODs and decimation, which come after the look is signed off; that is why the budget is
100,000 and why "push now, optimise later" is the standing instruction. Game lighting, which is
Epic 11's. The per-person combination layer, which is a separate task.
