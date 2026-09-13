# Round 10 — build the dwarf from scratch, at one resolution, to the reference

**This document is the complete specification. You are building `SM_VoxelDwarf_Miner01` from
nothing: an empty collection, a primitive, and the references. You inherit no previous `.blend`, no
previous texture and no previous rig — everything you need is here or in `src-assets/references/`.**

It exists because the process has to be reproducible. Nine rounds produced a figure and a great deal
of hard-won knowledge, and most of that knowledge lived in round-to-round briefs that each assumed
the last round's file. This one assumes nothing.

**What round 9 proved, and why you are starting over anyway.** Round 9 changed the construction
method — base mesh, form pass, carved features — and it worked: the figure's horizontal edge
fraction fell from 79.6 % to 61.6 %, the banding disappeared, and the masses gained real volume.
What it did not fix is **resolution**: its body resolves at a 38 mm median face edge while its boots
resolve at 6.4 mm, a 6× spread that is an accident of authoring order rather than a decision. Wolf,
2026-09-13: *"we could still have a lot more detail overall everywhere .. it's still away from
reference images"*, and *"resolution should be pretty much same for whole model"*. Starting from a
clean base mesh at one resolution is cheaper than re-resolving a figure built at six.

---

## 1. The subject and the references

A dwarf miner, **1.200 m tall**, stylised low-poly in a world of voxel pines, seen mostly at
distance in play and in close-up for marketing. Four inputs, and each is authoritative for a
different thing:

| input | authoritative for | notes |
|---|---|---|
| `references/dwarf-ortho/` (front, side-left, side-right, back) | **proportion, landmark heights, widths, depths** | 5x nearest-neighbour crops; **read its README first** |
| `references/reference-sheet.jpg` | the palette and the gear breakdown | the drawings are trustworthy; **its printed annotations are not** |
| `references/dwarf-frames/` (`f084`–`f164`) | **form, construction and lighting** — how it reads in 3D | `f088` is the form authority, `f104` the clearest near-front face |
| `references/dwarf-model-sheet.md` | the measured tables, transcribed | its **top** table is superseded video data; the **orthographic** table below it is the authority |

**The scale that governs everything:** the figure spans rows 7–147 in `front.png` and
`side-left.png`, so **140 source pixels = 1.00 H, and one source pixel is 8.571 mm**. `front.png`
column 77 is the centre line, `side-left.png` column 58 is the depth centre, **+Y is forward**, the
figure's right is **+X**, the sole sits at **z = 0**.

**Known defects in these inputs. Do not rediscover them and do not obey them:**

- the sheet's printed labels contradict its own drawings (one reads `0.6x dwarf height` for the
  dwarf's own height); **the pickaxe has four conflicting authorities spanning 2.3x** — round 7 chose
  `front.png` (shaft 0.91 H, blade span 0.38 H, head-span/length 0.42) and that choice stands;
- `gear.png` and `front.png` are at different scales with no stated conversion;
- the ortho crops contain the sheet's own 1-px dimension rules on the crown and sole rows — exclude
  them from any automatic silhouette read;
- **the sheet is POSED.** Its arm span (0.772 H) and its two-handed carry are properties of the pose,
  not of the body. See §4.

## 2. The measurements — settled, do not re-derive

Heights as a fraction of H, measured up from the sole:

| landmark | z/H | | landmark | z/H |
|---|---|---|---|---|
| crown | 1.000 | | beard tip | **0.464** |
| crown steps, top down | 0.979 / 0.964 / 0.943 | | belt top / bottom | 0.421 / 0.343 |
| brow | 0.879 | | hand bottom | 0.330 |
| eye line | 0.807 | | tunic hem | **0.207** |
| ear top / bottom | 0.850 / 0.729 | | boot cuff top / bottom | 0.164 / 0.107 |
| nose tip (lowest) | 0.750 | | sole | 0.000 |
| head + hair mass ends | **0.707** | | sleeve cuff, forearm begins | 0.566 |
| shoulder line (sleeve cap top) | **0.700** | | | |

Widths as a fraction of H: head with hair **0.336**, ear to ear 0.383, crown steps top down
0.164 / 0.229 / 0.286 / 0.336, beard widest 0.343, chest (tunic only) 0.317, **shoulders over the
sleeve caps 0.528**, waist / skirt 0.439, stance boot-to-boot 0.398, one boot 0.164, boot cuff 0.200.

Depths as a fraction of H: head back-to-hair-front 0.336 (**the head is very nearly a cube**), nose
tip past the back of the head 0.379, beard front 0.357, torso 0.286, skirt 0.300, pack behind the
torso 0.186, shin 0.164, **boot sole length 0.214** (the toe projects 0.057 H past the shin front),
whole figure pack-to-nose 0.550.

**Two measurements that are in no table and were expensive to find:**

- **the hair's front edge in profile is `y = +0.111`** (`side-left.png` col 71). Run the side lobes
  forward past this and the profile becomes a featureless slab with no forehead, brow or nose;
- **the neck is visible only if nothing shoulder-borne is outboard of it.** In an orthographic side
  view the outermost |x| wins, so a pack strap at the shoulder erases the neck. Drop the strap onto
  the sleeve cap.

## 3. Resolution — the heart of this round

**One granularity for the whole figure: no surface resolves coarser than 20 mm** — roughly the
reference video figure's own voxel size, and 2.3 source pixels of the sheet.

- **It is a FLOOR on step size, not a target face count.** A genuinely flat panel may be one large
  quad, because there is nothing on it to resolve. What the floor forbids is a *curve or a feature*
  described in steps bigger than 20 mm.
- **The face is the one exception, and it goes FINER: ~8.6 mm**, one sheet pixel. Its features are
  1.4–5 % of figure height (17–60 mm), so a 20 mm step physically cannot express an eyelid or a
  nostril.
- **No mass may be the coarsest by a wide margin, and the body may not be the coarsest at all.**
  Round 9's failure in one line: body 38 mm, boots 6.4 mm.
- **Expect 10,000–14,000 faces and 20,000–28,000 triangles.** `TRI_BUDGET` is **30,000** and it is a
  real constraint at this resolution — the first time it has been. If you need to choose, spend on
  the body, the head and the hands; the boots do not need 6 mm faces.

**Report the distribution** (§9): median face edge in mm per mass, and faces per m². That table is
how this round is judged on resolution.

## 4. The bind pose — the one place the reference must be disobeyed

**Model in an A-pose: arms out at roughly 40° from the body, with at least 20 mm of clear air at the
armpit.** Hands hang relaxed, palms toward the thighs. This is the standard bind pose for a rigged
character, and the sheet's arms-down pose is a pose.

Round 8 closed the arms to match the sheet and the cost was measured: its arm's inboard face sat at
|x| = 0.192, **exactly** the torso's half-width — zero armpit clearance, coplanar surfaces between
two objects (which shimmer in the game and which no topology gate catches), an arm that cannot
rotate without sweeping through the tunic, and the worst possible case for a buried-face cull.

- **Stage 1–3 silhouette comparisons will not match the sheet at the arms.** Expected. Judge torso,
  head, skirt and boots against the sheet; judge the arms on clearance and form.
- **The props sit in neutral hands**, not across the chest. **The two-handed carry is a POSE**,
  delivered as a render once the rig exists — that is where the arms are finally compared.
- A T-pose is not wanted: it distorts the shoulder mass.

## 5. How to build it — the workflow, proven in round 9

**Do the stages in order. Do not start a stage until the previous one reads right. Judge each stage
yourself against the reference planes — do not stop and wait for approval.**

**Stage 1 — reference planes, then a base mesh of the WHOLE figure.** Load `front.png` and
`side-left.png` as image planes scaled to 1.200 m. One cube, extruded into the whole figure: torso
up into neck and head, out into shoulders and arms, down into hips and legs. **Mirror modifier, model
one half** (it may stay live — §7). In the bind pose. **No features.** 200–500 triangles, then render
front / side / three-quarter / back against the planes. **The silhouette is this stage's
deliverable** — if it is wrong here, nothing added later fixes it.

**Stage 2 — the form pass.** Loop cuts where the form changes: chest, waist, hips, knee, elbow,
wrist, ankle, brow, jaw, crown. Move and scale the loops to taper the limbs, narrow the waist, curve
the cranium back, round the shoulder and the upper back. **The side profile is as binding as the
front** — a mass of constant depth top-to-bottom is wrong unless the reference draws it so. Steps
advance in **two axes at once**; a step that changes only z is a band, and bands are what made round
8 read as layers. Put the deform loops at §6's joint sites. Still no features. End with a render set.

**Stage 3 — carve the features OUT of the surface.** Inset and extrude; never stack a primitive on
top. The brow ridge is an extruded band of the forehead, the nose is extruded from the face, the eye
sockets are inset, the hem lip is an extruded rim, the collar an inset of the neck opening, the cuff
an extrusion of the sleeve end. **Bevel the silhouette edges that should read** — a chamfer of
8–26 mm catches light where a hard corner does not. **A horizontal division must be structural or it
must go**: keep the belt, the boot cuff, the sole; everything else runs vertical or diagonal —
beard locks, hair locks, folds.

**The features, all of which must be present and must read at 100 px and 60 px figure height:**
forehead; brows separate from the hair; eyes with a white and a dark pupil in a socket; **a nose with
form — a lit front and shaded sides, and it must project past the cheeks by more than 4 mm, which is
all round 9 managed**; moustache and mouth; a beard that tapers and steps in **locks**, not slabs;
ears **tucked to the skull, not flanges** (round 9's stood 93 mm outboard of the cheek plane and read
as handles); cheekbones and a jaw; a layered tunic with a visible edge; shoulder pieces stepped at
the shoulder line; a hem lip; a lighter waist panel with the square motif; a leather belt with a
raised buckle and a strap end; boots with sole, heel, toe cap and cuff; a pack with flap, buckles and
a bedroll; straps with hardware, sitting on the sleeve cap; a lantern with a metal frame, glass set
back, a cap, a bail and the flame cell — **the flame is a COLOUR and never an emitter**; a pickaxe
with a facetted head, bindings where head meets shaft, a tapered shaft and a butt cap.

**Stage 4 — separate objects, and only for what must swap.** **The body is ONE continuous mesh**:
torso, arms, hands, legs, neck, head and all the head's carved features. Separate named objects only
for what the per-person combination layer swaps: **hair, beard, moustache, belt, pack, straps,
lantern, pickaxe, boots.**

**Tag every carved feature** with a vertex group named `feature_*` — `feature_nose`,
`feature_brow.L`, `feature_hem`. Verified in Blender 5.2: vertex groups survive the exporter's join
with names and membership intact, and FACE-domain named attributes do too
(`mesh.attributes.new("feature_nose", 'BOOLEAN', 'FACE')`). **Face maps do not exist any more** —
removed in 4.x. Tags are reported by the exporter, never gated.

**And give each swappable feature a stable boundary loop** — a clean ring you would be willing to
freeze, so a variant can be stitched into the same hole later. Say in the report which loops you
intend as the swap boundaries. Free now, a rebuild afterwards.

**Stage 5 — UVs and paint.**

- **One material, one image**, `M_VoxelDwarf_r10` and `r10`, **512 x 512**, PACKED into the `.blend`.
- **PACK THE ISLANDS PROPERLY.** Round 9's map was 256 x 256 with **87 % of its texels a single flat
  fill** — the islands were so sparse that most of the map was wasted, which is why its surfaces read
  flat. **Report the fraction of the map your islands actually cover**; below ~60 % means re-pack
  rather than enlarge.
- **The approved ten are the base** — `#E9D2BB` skin, `#5E4632`, `#FFFFFF`, `#5F7A6A` tunic,
  `#474B41`, `#A9B2AC` metal, `#8B6B50` wood, `#6B5B49`, `#34271C` hair, `#F0A63C` flame — and value
  steps on top of them are yours. **Crisp steps, no gradients across a part.**
- **The face gets its own dense island**; state the texel density you gave each region.
- **Paint what geometry cannot carry**: the pupil and iris, the lip line, the waist panel's motif,
  cheek warmth, a temple shadow, cloth weave, metal wear, dirt at the hem and soles. Round 9's
  surfaces are flat fills; the reference's are not.
- Backface culling on. **Interpolation `Closest`** — the contract requires NEAREST filtering and the
  exporter deliberately does not force it. Specular IOR Level stays at its `0.5` default: a `0` emits
  `KHR_materials_specular` and the no-extensions clause rejects the export.

**Stage 6 — rig, weights, deflection.** §6.

## 6. The rig

**Nineteen joints, and the names are the contract** because the game binds by name:

```
root  hips  spine  chest  neck  head
shoulder.L/R  elbow.L/R  hand.L/R
hip.L/R  knee.L/R  foot.L/R
beard
```

- **One armature** in the round's collection, named `SK_VoxelDwarf_Miner01_r10`. The hierarchy is
  yours; the names are not. `root` carries no geometry.
- **ONE mesh, RIGID weights: every vertex belongs to exactly one joint at weight 1.0.** No soft
  skinning, no separate limb meshes. This is a style ruling, not the industry default — it keeps a
  hard-edged figure from smoothing at the joints.
- **Put each joint on an edge ring the figure actually has**, from stage 2. Round 9's, for reference:
  elbow 0.612, knee 0.330, hand 0.424, shoulder 0.806, head 0.886, beard 0.875.
- **The props are weighted like anything else** — the pickaxe to the hand that carries it, the
  lantern and pack to what they hang from — so a posed arm takes its prop with it.
- **The mesh ships in the bind pose with NO animation** in the GLB.
- **Prove the weights**: bend each joint group and render it. Tearing, a hole, or geometry following
  the wrong bone is a defect to fix, not to report.
- **If you cull buried faces, scope the cull BY JOINT.** A face buried inside a part on a *different*
  joint is exposed the moment that joint bends. Round 8 lost nine faces of a hand to a cull that
  asked the wrong question, and paid a full rebuild. Use a six-direction enclosure test that fails
  safe, run **once**, over closed parts. An asymmetric face count between `.L` and `.R` is the tell.

## 7. The tools, and what they will and will not accept

One command, from the repo root:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/export_dwarf.py

**`REV` is `r10`**, so your collection must be `SM_VoxelDwarf_Miner01_r10` and every datablock
carries `r10`: object, mesh, material, image, armature. **Until that collection exists the export
exits with `collection ... not found`, which is expected — do not edit `REV`.** Earlier revisions
stay in the file, excluded from the view layer.

**Live modifiers are allowed.** Mirror, Subdivision and Bevel may stay live and are applied on the
way out; the exporter measures the **evaluated** mesh, so the triangle count and the topology gates
describe what the file carries. Only Armature is removed rather than applied.

**Gates that fail the build** — all of them mutation-tested:

- triangles and quads only, no n-gons; manifold; no loose verts or edges; no degenerate faces;
  consistent winding; a UV layer present;
- **≤ 30,000 triangles**;
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
carries, so **no stray colours**.

**One rule that will bite repeatedly if ignored: pieces within one object must OVERLAP, never meet
exactly.** Coincident rims weld under `remove_doubles` and produce edges with four faces, which the
exporter rejects as non-manifold. Coincident faces between two *objects* are worse: they shimmer in
the game and no gate catches them.

## 8. It must be watchable

- **Author INSIDE Wolf's running Blender through the MCP addon. Never a `--background` subprocess.**
  Only the export and the render passes run headless.
- **Prove it before your first edit**: query the live scene and report what was already in it. A
  subprocess starts from a default scene. **If you cannot reach the live instance, stop and say so** —
  Blender never reloads a file that changed underneath it, so the work would be invisible.
- **One OPERATION per tool call** — an extrude, a loop cut, a loop moved, an inset, a bevel. Not one
  finished part per call: that grain is what built round 8 out of fifty-four separate boxes.
- **Save the `.blend` after every call**, and write a numbered progress render plus the stage's render
  set at every stage boundary. Four of six delegated runs in this project were killed by the harness;
  the saves are the insurance.
- **No generator script.** The `.blend` is the deliverable. Do not write a `dwarf_r10.py` that
  rebuilds the figure and do not keep part specs in a file as the real source. Byte-identical
  regeneration was retired at round 4 and does not come back: round 3 delivered it and produced the
  worst-looking dwarf of the nine, because a generator makes you think in loops and parameters.
- **No git commands at all.** The operator commits.

## 9. Deliverables

| # | what | path |
|---|---|---|
| 0 | the live-scene report | in the report, before any geometry |
| 1 | the source, saved per call, image PACKED | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture map | `src-assets/blender/textures/T_VoxelDwarf_r10.png` |
| 3 | the render script | `src-assets/blender/render_r10.py` |
| 4 | a render set per STAGE — block-out, form, carved, textured | `src-assets/renders/r10/stage-N-*.png` |
| 5 | progress renders | `src-assets/renders/r10/progress/NN-<operation>.png` |
| 6 | five final views, flat and key-lit | `src-assets/renders/r10/` |
| 7 | beside `f088` and `f104`, matched scale | `src-assets/renders/r10/vs-frames-*.png` |
| 8 | the side profile beside `side-left.png` | `src-assets/renders/r10/vs-ortho-side-left.png` |
| 9 | the readability strip at 100 px and 60 px | `src-assets/renders/r10/readability.png` |
| 10 | the posed carry, and one deflection render per joint group | `src-assets/renders/r10/pose-*.png`, `joint-*.png` |
| 11 | **the resolution table: median face edge (mm) and faces/m² per mass** | in the report |
| 12 | **the UV coverage figure: what fraction of the map the islands occupy** | in the report |
| 13 | the per-part census: name, faces, tris, size | in the report |
| 14 | your report | `src-assets/prompts/dwarf-miner-round-10-report.md` |
| 15 | exporter and checker output, verbatim | in the report |
| 16 | cost — `session_tokens.py --transcript` (its default path is broken on Windows), row `dev-art` | in the report |

## 10. How this round is judged

By Wolf's eye against `f088` and `f104` — **does it read like the reference, at one resolution?**
Then, mechanically:

- **no surface coarser than 20 mm; the face at ~8.6 mm; the body is not the coarsest mass**
- **the UV islands cover ≥ 60 % of the map**, and no surface is a single flat fill where the
  reference has variation
- the body is one continuous mesh, in the bind pose, with ≥ 20 mm armpit clearance
- every §5 feature present and readable at 60 px; the nose projects well past the cheeks; the ears
  are tucked to the skull
- `check_asset.py` exits 0; the export passes topology, budget, texture, seating and rig gates
- the posed renders show no tearing at any joint
- nothing written outside `src-assets/`, and no git command run

## 11. What is NOT in this round

Animation clips — the rig ships posable with no animation in the GLB, and the two-handed carry is a
render. LODs and decimation, which come after the look is signed off. Game lighting, which is Epic
11's. The per-person combination layer, which is a separate task; all this round owes it is stage 4's
named objects, feature tags and stable boundary loops.
