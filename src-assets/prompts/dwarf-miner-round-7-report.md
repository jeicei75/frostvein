# Round 7 report — a face, painted, on a figure that matches the sheet

**Result: a complete figure at 892 triangles over 20 named parts, built part by part inside
Wolf's open Blender, with the eight §3 features carried by a hand-placed 256x256 texture and
a head that measures 0.296 H against the sheet's 0.293 H.** The export passes every topology
clause at 0/0/0/0/0/0/0/0. `check_asset.py` fails, on a clause this round's own brief made
impossible to satisfy — §8.

**Six findings are in §7**, three in the input documents, two in tooling and one in my own method. The one
that matters most to the next round is §7.1: **the checker can no longer reach its geometry
clauses at all for this asset**, so "the checker's grid clause fails a box model" is no longer
the whole story.

---

## 0. The live-scene report — proof this ran in Wolf's Blender

First call, before any geometry, `get_scene_info` then a direct query of the running instance:

```
blender_version: 5.2.1 LTS
filepath:  ''
is_dirty:  False
pid:       29036
collections: ['Collection']
objects:   [('Camera','CAMERA',(7.359,-6.926,4.958)),
            ('Cube','MESH',(0.0,0.0,0.0)),
            ('Light','LIGHT',(4.076,1.005,5.904))]
meshes:    ['Cube']
materials: ['Dots Stroke', 'Material']
images:    [('Render Result',(0,0)), ('Viewer Node',(256,256))]
render_engine: BLENDER_EEVEE
cwd:       C:\Program Files\Blender Foundation\Blender 5.2
```

**What was already in it: the default startup scene** — Cube, Light, Camera, unsaved, not
dirty — in a GUI process whose working directory is the Blender install, not the repo. A
`--background` subprocess cannot be reached by the MCP addon at all, and would not carry a
`Viewer Node` image or a `Dots Stroke` material.

Second call opened the deliverable and reported what that file already held:

```
now filepath: D:\Workspace\frostvein\src-assets\blender\SM_VoxelDwarf_Miner01.blend
collections:  [('SM_VoxelDwarf_Miner01_r5', 20), ('SM_VoxelDwarf_Miner01_r6', 17)]
materials:    ['M_VoxelDwarf_r5', 'M_VoxelDwarf_r6']
images:       [('T_VoxelDwarf_Palette_r5',(64,64)), ('T_VoxelDwarf_Palette_r6',(64,64))]
object_count: 38
```

**r5 and r6 are still in the file and untouched** — the `.blend` accumulates, as §5 asks. They
are excluded from the view layer so only r7 is on screen and only r7 renders. Every one of the
33 modelling calls ended in `wm.save_mainfile()` plus a progress render, so the file on disk
never trailed the screen by more than one part.

**There was one wipe-and-rebuild, announced here as §5 requires, and it was forced by a defect
of mine** — every part was rebuilt in four calls near the end because the buried-face cull had
been run repeatedly against occluders it had itself opened, and had eaten nine faces of the
left hand. The full account is §7.7. Everything before that was incremental: parts appeared one
per call, and seven were *edited* in place afterwards — hair (four times), head, beard, neck,
straps, pickaxe, lantern — each as its own call with the reason stated in it.

## 1. The measurement frame, and what was measured before anything was built

`H = 1.200 m`, sole at `z = 0`, centre line `x = 0`, **`+Y` forward**, his right is `+X`. One
source pixel of `dwarf-ortho/` is `H/140 = 8.571 mm`. `front.png` column 77.5 is the centre
line, `side-left.png` column 58 is the depth centre.

Rather than trust the model sheet's table, I re-read the pixels. Both ortho crops were loaded
into Blender, decimated 5:1 back to source pixels, and printed as a classified character map.
That is where the face numbers in §2 come from, and it found the features the sheet's table
does not list at all:

| feature | read off `front.png` | as z/H | as z |
|---|---|---|---|
| hairline, top of the skin | row 18 | 0.921 | 1.106 |
| brow band | rows 24–25 | 0.879–0.871 | 1.055–1.045 |
| eye whites | rows 32–34 | 0.821–0.807 | 0.985–0.968 |
| nose, lit column | cols 73–81, rows 29–42 | 0.843–0.750 | 1.011–0.900 |
| moustache | rows 43–45 | 0.743–0.729 | 0.891–0.875 |
| mouth | rows 46–47 | 0.721–0.714 | 0.866–0.857 |
| beard taper, half-width | rows 44–61 / 62–67 / 68–77 / 78–82 | — | ±0.197 / ±0.171 / ±0.137 / ±0.111 |
| hair front edge in profile | `side-left` col 71 | — | y = +0.111 |

The beard taper is the only one of these the sheet quantifies (0.343 H at its widest); the
four-step profile and the hair's front edge in profile are new measurements, and both turned
out to be load-bearing — see §7.3.

## 2. The texture, which is the round

**One image datablock `r7`, 256 x 256, one material `M_VoxelDwarf_r7`, backface culling on,
Specular IOR Level left at its `0.5` default.** `src-assets/blender/textures/T_VoxelDwarf_r7.png`.

**How it was authored.** Not as a palette atlas and not as a generated pattern: every island is
painted by a rule expressed in the **same world coordinates the geometry is built from**, so
"the brow band runs z 1.046–1.072 at |x| 0.098–0.128" is one statement that places the paint
where the measurement said the brow is. The texture cannot drift from the model because they
are addressed in one space.

**Texel density.** Every face is projected along its dominant axis at a single scale and
shelf-packed into 186 islands:

| island set | density | why |
|---|---|---|
| the whole body — 183 islands | **35 px/m**, uniform | every one of them is one flat colour; there is nothing on them to resolve |
| the head's `+Y` quad — **the face** | **500 px/m** | the 0.274 x 0.246 m face gets a 137 px island, four times the resolution the reference's own 140-px-tall sheet gives it |
| the beard's `+Y` top quad — moustache and mouth | **320 px/m** | |
| the belt's `+Y` quad — the buckle | **230 px/m** | |

§2 permits the face its own denser island; I took that for three islands rather than one, and
the reason is that the moustache, the mouth and the buckle are painted, not modelled, so they
need pixels for the same reason the face does. **The body's density is even by construction** —
one scale, one projection rule, no exceptions — and no body island carries a gradient, so 35
px/m is not a compromise, it is the resolution a flat colour needs.

**Colours.** The approved ten are the base and every one of them round-trips through the flat
pass **exactly**, verified by sampling the render at known world coordinates (`#5E4632` beard,
`#5F7A6A` tunic, `#473C31` belt, `#FFFFFF` eye white, `#34271C` brow, `#A9B2AC` metal, all
exact; skin and three value steps come back 1/255 low, which is PNG rounding). On top of them
are the value steps the reference paints and the flat palette could not hold:

```
skin      #E9D2BB   cheek #DCBCA0   socket/flank #C4A88F   lid #9E8571   nose front #F6E6D6
hair/beard#5E4632   crown top #664C36   under-hair #47341F   brow #34271C   beard foot #523D2B
moustache #7A5F46   under-lip #6B5039   mouth #241A12   pupil #231A12
tunic     #5F7A6A   hem #4C6355   pants #474B41   leather #6B5B49   dark leather #473C31
boot cuff #7A6752   sole #322A22   metal #A9B2AC   dark metal #707572   wood #8B6B50
bedroll   #63695B   lantern flame #F0A63C
```

No gradients. Every step is a hard boundary between two flat values, which is what the
reference's steps are.

## 3. The eight items §3 asks for, and how each is built

| # | item | how | measured |
|---|---|---|---|
| 1 | **a forehead** | painted skin between the fringe's underside and the brow band | 0.060 m of bare skin, **0.050 H**; the sheet's is rows 18–24 = **0.043 H** |
| 2 | **brows separate from the hair** | two painted steps that slope down toward the centre, stopping at \|x\| 0.128 against a head half-width of 0.137 — **9 mm of skin between brow and hairline on each side** | brow band 0.026 m tall = 0.022 H; sheet 2 px = 0.014 H |
| 3 | **eyes, white and dark pupil, in a socket** | painted: white 0.078–0.128, pupil 0.088–0.113, a `#C4A88F` socket around and a `#9E8571` lower lid under | white 0.042 H wide x 0.025 H; sheet 0.036 H x 0.021 H |
| 4 | **a nose with form** | **geometry**: a two-step box, narrow bridge stepping out to a wider tip, projecting 0.063 m past the face plane. Front face `#F6E6D6`, sides `#C4A88F`, underside `#9E8571`, and the face plane behind it carries the flank shadow | projects **0.053 H**; visible as a step in both side views |
| 5 | **a moustache AND a mouth** | moustache painted across the join — upper half on the face plane (z 0.890–0.916), lower half with its droop on the beard's front face (z 0.861–0.890); the mouth is a `#241A12` slot cut into it at z 0.856–0.876 | moustache 0.036 H tall; mouth 0.070 H x 0.017 H |
| 6 | **a beard that tapers and steps** | **geometry**: four measured steps, ±0.197 → ±0.171 → ±0.137 → ±0.111, tip at 0.464 H | tip **0.464 H** against the sheet's **0.464 H**; widest 0.328 H against 0.343 H |
| 7 | **ears** | geometry, tapered boxes standing proud of the hair, between the eye line and the nose tip | ear-to-ear **0.383 H** against the sheet's **0.383 H** |
| 8 | **a head that is not oversized** | see below | **0.296 H** against the sheet's **0.293 H**. r6 was 0.350 H |

**On item 8, and the measurement it forced.** The sheet's landmark is "head + hair mass ends
0.707 H", which it derives from **`back.png` row 44** as well as front row 48. It cannot be
measured on the front view of a figure whose beard is as wide as its head — the silhouette does
not step there, in mine or in the reference's. So the block was measured the way `back.png`
measures it: crown to the lowest hair. First build came in at **0.314 H**, all of the excess
being the back lobe hanging to z 0.823; the lobe now stops at 0.845 and the block is
**0.296 H, within 1 % of the sheet**.

The face itself is independently right: skin from the fringe to the beard is **0.180 H** against
the sheet's **0.186 H**.

## 4. Readability at 100 px and 60 px

`src-assets/renders/r7/readability.png` — the front and the three-quarter, key-lit, at 100 px
and at 60 px of figure height, nearest-neighbour, no resampling.

At **100 px** all eight items read: the forehead, the sloped brows, both eyes with their
pupils, the nose's lit front against its shaded flanks, the moustache, the mouth, the stepped
beard, the ears.

At **60 px**, honestly:

| feature | height at 60 px | reads? |
|---|---|---|
| brows | 1.3 px | yes — they are the darkest thing on the face |
| eye white + pupil | 1.5 px | yes, as a light/dark/light triplet |
| nose | 3.2 px of profile projection | yes |
| moustache | 2.2 px | yes |
| **mouth** | **1.0 px** | **marginal** — it reads only because the moustache above it is lighter than the beard below it. Under about 50 px it merges into the beard |
| beard taper, ears | — | yes |

**The mouth is the weakest of the eight and I am not going to claim otherwise.** The reference's
own mouth is 1.5 source pixels on a 140 px figure — 0.011 H against my 0.017 H — so it is not
physically available at 60 px in the reference either. I made mine 1.5x the drawn size to buy
that one pixel; making it larger again would start to read as a cartoon mouth at 100 px, which
is the resolution Wolf is actually judging.

## 5. The parts, and what the combination layer gets for free

20 objects, every swappable feature already its own named object, as §5 requires:

```
r7_head    r7_hair    r7_beard   r7_nose    r7_ear.L/R  r7_neck
r7_torso   r7_skirt   r7_belt    r7_arm.L/R r7_leg.L/R  r7_boot.L/R
r7_pack    r7_straps  r7_pickaxe r7_lantern
```

Nothing is welded into a shared mesh; the exporter joins copies at export time and the
authored parts stay separate in the `.blend`. Swapping a beard, a hair, a pack, a belt, a pair
of boots, a tunic or a tool is an object substitution.

**Triangles: 892 of the 4,000 budget**, counted by the exporter's own loop-triangle count.
There is 4.5x headroom; I did not spend it, because §1 says the budget is a ceiling and because
every triangle I did not spend on a bevel is one the LOD chain will not have to remove.

**Proportions against the sheet, all measured on the built geometry:**

| | r7 | sheet | |
|---|---|---|---|
| head + hair block | 0.296 H | 0.293 H | +1 % |
| head width with hair | 0.337 H | 0.336 H | +0.3 % |
| ear to ear | 0.383 H | 0.383 H | exact |
| shoulders over the caps | 0.528 H | 0.528 H | exact |
| stance, boot outer to outer | 0.400 H | 0.398 H | +0.5 % |
| beard tip | 0.464 H | 0.464 H | exact |
| beard widest | 0.328 H | 0.343 H | −4 % |
| face skin, fringe to beard | 0.180 H | 0.186 H | −3 % |

## 6. Edge loops at every joint — the rig is next round and is not foreclosed

Verified on the built meshes, not asserted: a ring of four vertices at each joint's z, in every
part that spans it.

```
root        0.000  the figure's own origin; not a deform joint
hips        0.440  r7_torso:4  r7_skirt:4  r7_leg.L:4  r7_leg.R:4
spine       0.640  r7_torso:4
chest       0.720  r7_torso:4
neck        0.800  r7_torso:4  r7_neck:4
head        0.886  r7_neck:4
shoulder.L  0.820  r7_arm.L:4          shoulder.R  0.820  r7_arm.R:4
elbow.L     0.620  r7_arm.L:4          elbow.R     0.620  r7_arm.R:4
hand.L      0.420  r7_arm.L:4          hand.R      0.420  r7_arm.R:4
hip.L       0.440  r7_leg.L:4          hip.R       0.440  r7_leg.R:4
knee.L      0.330  r7_leg.L:4          knee.R      0.330  r7_leg.R:4
foot.L      0.190  r7_boot.L:4 r7_leg.L:4   foot.R  0.190  r7_boot.R:4 r7_leg.R:4
beard       0.875  r7_beard:4
```

The head joint's loop lives on `r7_neck`, because the head is rigid above it and its `+Y` quad
is the face's single texture island — cutting it would split the island and put a seam through
the face. That is a deliberate choice and it is the only joint site where the loop sits on the
part below rather than on both.

**The stance is neutral and stays neutral.** The sheet's 0.772 H arm span is a property of its
pose, and the reference's pickaxe is held across the body for the same reason; §7 rules both to
the rig. The consequence is in §7.5.

## 7. Conflicts, defects and departures

**7.1 `check_asset.py` can no longer reach its geometry clauses for this asset, and that is
new.** The brief names three broken clauses and says none are mine to chase. What actually
happens now is narrower and more serious: the palette clause raises **`expected a 64x64 V1
atlas`** at `check_asset.py:178`, which is *before* the grid clause, the naming clause and the
figures line. **The brief itself mandates a 256 x 256 map and retires the 16-cell atlas**, so
the contract as written and the checker as written cannot both be satisfied — this is not a box
model failing a grid clause, it is the checker rejecting the round's own deliverable format at
the door. Round 6 reported the grid clause as the single line of output; for r7 even that clause
never runs. Not chased, not edited, reported verbatim in §8.

**7.2 The bare neck is buildable, but the lever is the STRAP, not the hair.** Round 6 reported
check 5 as unbuildable because head mass ends at 0.707 H and the shoulder line sits at 0.700 H.
That reasoning is right about the hair and wrong about what covers the neck. In an orthographic
side view the outermost \|x\| wins, so **anything shoulder-borne outboard of the neck erases
it** — and my first build's pack strap, at x 0.100–0.168 and z 0.845–0.864, did exactly that
while the hair was innocent. Dropping the strap onto the sleeve cap (z 0.838–0.852) and raising
the head's floor to 0.886 opens the band. Measured on the delivered flat renders, classified to
palette cells and walked row by row: **18 px of bare skin, 0.027 H, in `dwarf-flat-side-left`
and `dwarf-flat-side-right` alike.** That is below check 5's withdrawn 0.08 H gate and well
below round 6's reported 0.211 H — which I believe was measuring the ear, exactly as round 6's
own §7.3 says the sheet's rows do. The neck is visible; the number check 5 asks for is not
reachable with a shoulder at 0.700 H.

**7.3 The hair's front edge in profile is a measurement nobody had taken, and it decides whether
the figure has a face from the side.** My first build ran the hair's side lobes forward to
y +0.214, the full depth of the head, and the profile came back as a featureless brown slab —
no forehead, no brow, no nose. `side-left.png` puts the hair's front edge at **col 71 = y
+0.111**, with bare skin ahead of it from col 72 to the nose tip at col 87. With the lobes cut
back to +0.111 the profile carries forehead, brow line, nose step and cheek. **This is not in
the model sheet's depth table**, which lists only "head, back to front of the hair 0.336".

**7.4 The exporter's non-manifold clause catches coincident rims inside a single part, and it
was right to.** The hair's crown piece began at z 1.132, exactly where the back slab and the
fringe ended; `remove_doubles` welded the rims and two edges came out with **four** faces each.
The export failed with `non-manifold edges 2` and named it. Fixed by dropping the crown 3 mm so
the pieces overlap instead of abutting. **The rule this establishes for the parts library:
pieces within one object must overlap, never meet exactly.**

**7.5 The reference's pickaxe is broadside because the reference is POSED, and I did not chase
it.** Measured on **`front.png`** — stated, because §4 says the four authorities disagree by
2.3x and to pick one and move on: shaft **0.91 H** corner to corner (127 px), blade span
**0.38 H** (53 px), so head-span/length **0.42**. In a neutral hang the blade lies in the
sagittal plane, which is the plane a carried pick actually hangs in, so from the front it reads
edge-on — a shaft with a small head. Rotating it broadside would drive the 0.46 m blade through
the skull at z 1.0. The two-handed carry across the body is a pose and belongs to the rig.

**7.6 `session_tokens.py`'s Windows slug defect (round 6 §7.7) is still open, unchanged.** The
default path still produces `C:\Users\suihk/.claude/projects/D:\Workspace\frostvein` and
reports `no transcript found`. Worked around with `--transcript`. Not fixed, not mine.

**7.7 MY OWN, and it cost this session a full rebuild: the buried-face cull was iterative, and
its containment test was only sound on closed solids.** "No interior geometry" is a review item
with no machine gate, so I gave it one: sample each face at its centre and its pulled-in
corners, test each sample for containment inside every *other* part, delete the faces that are
buried. The test was **ray parity** — count the hits along +X, odd means inside. That is correct
for a closed solid and **meaningless for an open one**, and the cull itself opens solids. Run
once it was fine; run again after each geometry edit, it was testing against its own leftovers.
The damage was visible: **`r7_arm.L` lost all nine faces of the hand segment**, judged buried in
a lantern that by then was missing seven of its own faces, and a close render of the left hand
showed straight through it. It was asymmetric — `r7_arm.R` kept 39 faces against `.L`'s 30 —
which is the tell I should have looked for earlier, because a symmetric figure whose two sides
cull differently has been culled wrongly somewhere.

Fixed properly rather than patched: every part rebuilt closed (§0), and the test replaced with
a **six-direction enclosure test** — a point is inside a part only if a ray along each of ±X,
±Y, ±Z leaves through one of that part's own faces *from the inside* (`normal · direction > 0`).
It needs no parity and no closedness assumption, and when a solid *is* open the ray simply
misses and the answer is "not inside", which keeps the face. It fails safe. One pass over closed
parts now removes **80 faces**, perfectly symmetric — `arm.L` and `arm.R` 3 each, `leg.L` and
`leg.R` 10 each — against the old test's 151 across five passes, and the delivered figure is
892 triangles rather than 822. **Seventy of those triangles are the price of the bug being
found; all of them were faces the old test should never have taken.**

Rebuilding closed also re-exposed two welds of the §7.4 family that the culling had been hiding:
the boot's toe step changed only one coordinate, so two of its four ledge corners were the same
point, and the lantern's bail drop shared a corner column with the bail. Both now step in every
coordinate, and all 20 parts are closed and 2-manifold before the cull runs.

**On "no interior geometry", as delivered.** **80 faces removed in one pass**, which is why the
delivered neck is 3 faces of a box's 14 and the torso 16 of 30. What remains
is contact area between abutting parts, which is unavoidable when parts abut and is not what the
clause is about. Nothing is sealed inside the figure, and off-axis renders — from below left,
below right, above-behind, and a close-up of each hand — were checked for see-through holes
before delivery.

**A departure, declared as §1 asks.** **The figure is hard-edged and flat-faced by choice.** No
bevel, no subdivision, no smooth normals, no rotated boxes — all four are permitted this round
and none is used. The reason is that `dwarf-frames/` is the reference's own 3D interpretation
and it is hard-edged and flat-faced, sitting in a world of voxel pines; the thing that was
missing from rounds 1–6 was never curvature, it was a face. `vs-face.png` and
`vs-ortho-front.png` are the evidence for the choice, and the 4.5x triangle headroom means
reversing it later costs nothing that has been spent.

## 8. The export and the checker, verbatim

```
EXPORT D:\Workspace\frostvein\src-assets\export\SM_VoxelDwarf_Miner01.glb
  parts joined      20 -> 1 mesh 'SM_VoxelDwarf_Miner01_r7'
  object / mesh     SM_VoxelDwarf_Miner01_r7 / SM_VoxelDwarf_Miner01_r7
  materials         M_VoxelDwarf_r7
  texture image     r7
  triangles         892  of 4000 budget
  size m (X,Y,Z)    0.789 x 0.657 x 1.200
  blender min Z     0.000000   (glTF min Y)
  blender centre XY 0.000000, 0.000000   (glTF centre X, Z)
  topology          n-gons 0   non-manifold edges 0   loose verts 0   loose edges 0
                    degenerate faces 0   flipped winding 0   missing UV layer 0
                    unapplied modifiers 0
  bytes             59748
```

```
FAIL src-assets\export\SM_VoxelDwarf_Miner01.glb: palette/material clause: expected a 64x64 V1 atlas
```

Exit code 1. **This is the checker's entire output** — the clause raises before the figures
line, so nothing further exists to quote. See §7.1: the model was not changed to satisfy it and
the checker was not edited.

## 9. The renders

The lit pass is rebuilt, per §8 of the brief. `render_r5.py`'s Workbench STUDIO "lit" is gone:

- **flat** stays Workbench `FLAT` + `TEXTURE` — no sampler noise, and every measurement in this
  report is taken off it;
- **key-lit** is EEVEE with **one sun, shadows on, 4.2 W, a 3° cone**, aimed high from the
  figure's front left along `f088`'s key, plus a **1.1 W shadowless fill** from behind the
  opposite shoulder so that the two side views are not judged against a single key. Raytracing
  is off deliberately: with one sun and flat albedo it adds nothing but grain, and grain on a
  flat-shaded asset reads as texture that is not there;
- **every path is absolute**, and the revision is in all of them (`renders/r7/`). Round 5's
  renders escaped to `C:\src-assets\renders\` because Blender resolves a relative
  `render.filepath` against the `.blend`;
- the world background is driven through the **background node**, not `world.color`:
  `World.use_nodes` is deprecated in 5.x and setting the colour no longer reaches EEVEE, which
  is why the first lit pass came back on a near-black ground.

## 10. Cost

```
Session token cost  (e696f294-c1c6-4542-8e9b-8706c4af8b3c.jsonl, tool=claude)  (281 turns, claude-opus-5)
  input (fresh)            562
  cache creation       671,591
  cache read        66,692,799
  output               505,666
  total processed   67,870,618
  wall-clock            56 min  (elapsed, includes idle gaps)
  est. cost             $50.19  (benchmark - verify rates in PRICES)
```

Model from the banner: **Opus 5 (1M context)**, `claude-opus-5[1m]`. Row: `dev-art`. Printed
with `--transcript` because of §7.6; no ledger row was written and no cursor advanced. The
figure above is the reading taken while writing this report, so it excludes the report itself.

## 11. Deliverables

| # | what | path |
|---|---|---|
| 0 | the live-scene report | §0 above |
| 1 | the source, saved after every part | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture map | `src-assets/blender/textures/T_VoxelDwarf_r7.png` |
| 3 | the render script | `src-assets/blender/render_r7.py` |
| 4 | the exported GLB (gitignored scratch) | `src-assets/export/SM_VoxelDwarf_Miner01.glb` |
| 5 | progress renders, one per part | `src-assets/renders/r7/progress/01..33-*.png` |
| 6 | five final views, flat and key-lit | `src-assets/renders/r7/dwarf-{flat,lit}-*.png` |
| 7 | the readability strip, 100 px and 60 px | `src-assets/renders/r7/readability.png` |
| 8 | side by side against the ortho sheet | `src-assets/renders/r7/vs-ortho-front.png`, `vs-ortho-side-left.png` |
| 9 | the face beside `f104`, matched scale | `src-assets/renders/r7/vs-face.png` |
| 10 | this report | `src-assets/prompts/dwarf-miner-round-7-report.md` |
| 11 | exporter and checker output, verbatim | §8 above |
| 12 | cost | §10 above |

**No generator script was written.** There is no `dwarf_r7.py` and no part-spec file: every
part's dimensions exist only in the `.blend` and in the modelling call that placed it. The
primitives used to author — the stepped-solid builder, the loop-cut helper, the island packer,
the paint pass — lived in the Blender session's `driver_namespace` for the length of the
session and were never written to disk, so there is no artifact that can regenerate the figure
and the `.blend` is not disposable.

**Nothing was written outside `src-assets/`**, verified by timestamp over the working tree
rather than by `git status`: **no git command was run at all.** The seven changed paths are
`blender/render_r7.py` (new), `blender/SM_VoxelDwarf_Miner01.blend` (+`.blend1`),
`blender/textures/T_VoxelDwarf_r7.png` (new), `export/SM_VoxelDwarf_Miner01.glb` (gitignored),
`renders/r7/**` (new) and this report.

## 12. What I would change next, in order

1. **The rig, which is correctly next.** The single biggest remaining likeness gap is the pose,
   and `vs-ortho-front.png` makes it plain: the reference holds the pick across the body in both
   hands and everything about its upper body follows from that.
2. **Rule on `check_asset.py`'s palette clause** (§7.1). It now rejects the format this brief
   mandates, so either the clause learns about painted maps or the asset contract says that
   revisioned meshes skip it. Leaving it failing means the checker certifies nothing for this
   asset while still costing a line in every report.
3. **The mouth at 60 px** (§4). If it must survive the small silhouette, the answer is not a
   bigger mouth — it is a darker beard immediately under the moustache, so the mouth reads as
   the bottom of a two-value shape rather than as a 1 px line of its own.
4. **The beard's mass.** Right width, right length, right taper, and still four stacked slabs
   where the reference's is a bush. That wants more and smaller steps on its own silhouette —
   and at 892 of 4,000 triangles there is room for them ten times over.
