# Round 8 report — the resolution raised everywhere, then rigged

**Result: 54 named parts, 3,955 triangles, one 512 x 512 painted map, and a 19-joint rigid rig,
built part by part inside Wolf's open Blender.** The export passes every topology clause at
0/0/0/0/0/0/0/0 and the rig gate at **19 joints / 0 missing / 0 unexpected / 0 unweighted /
0 soft-weighted / 0 non-bone**. **`check_asset.py` exits 0** — it is a gate this round and it
passed.

**No part of this figure is a plain box.** The head that was six faces is 62; the belt that was
five is 32 across three objects; each leg that was eight is 80; the pickaxe that was 42 is 226
across three objects. The per-part census is §5 and every row names the reference feature it
carries.

**Six findings are in §7.** The two that matter to the next round are **§7.1** — my buried-face
cull was scoped wrongly for a figure that will be posed, and the knee deflection render is what
caught it — and **§7.4**, a *new* silent-failure trap of exactly r7's family: **`Image.pack()`
does not replace an already-packed copy**, so the `.blend` shipped a two-repacks-stale texture
while the PNG on disk was correct and every report line said "packed".

---

## 0. The live-scene report — proof this ran in Wolf's Blender

`get_scene_info` first, before any geometry. The first two calls failed with
`Could not connect to Blender`; `blender.exe` was running as **pid 16296 since 07:57** with **no
listening socket**, so the addon server was not started. That is reported here because it is the
honest first line of the log: Wolf started it, and the session continued.

The first successful query, of the running instance:

```
FILE: 'D:\Workspace\frostvein\src-assets\blender\SM_VoxelDwarf_Miner01.blend'
blender 5.2.1 LTS
scene=Scene  objects_in_file=59
- Scene Collection  objs=2  [visible]
  - SM_VoxelDwarf_Miner01_r5  objs=20  [EXCLUDED]
  - SM_VoxelDwarf_Miner01_r6  objs=17  [EXCLUDED]
  - SM_VoxelDwarf_Miner01_r7  objs=20  [visible]
MATERIALS: M_VoxelDwarf_r5, M_VoxelDwarf_r6, M_VoxelDwarf_r7
IMAGES:
  r7  size=(256, 256)  packed=False  filepath=D:\...\textures\T_VoxelDwarf_r7.png
  T_VoxelDwarf_Palette_r5  size=(64, 64)  packed=True
  T_VoxelDwarf_Palette_r6  size=(64, 64)  packed=True
ARMATURES: <none>
CAMERAS: R6Cam, R7Cam        LIGHTS: <none>
```

**What was already in it: the r7 deliverable, open and unrigged**, with r5 and r6 excluded from
the view layer and r7 on screen. A `--background` subprocess cannot be reached by the MCP addon
at all.

**One thing that report line contradicts the brief, and it is worth recording.** §7 of the brief
says of round 7's texture "The image is packed now". In the live file **it was not**:
`r7 packed=False`, pointing at an absolute `D:/` path. The exporter's new clause is therefore
still load-bearing, and §7.4 below is the same bug from the other side.

Final state of the file, verified after the last save:

```
SM_VoxelDwarf_Miner01_r5     objs=20  excluded=True
SM_VoxelDwarf_Miner01_r6     objs=17  excluded=True
SM_VoxelDwarf_Miner01_r7     objs=20  excluded=True
SM_VoxelDwarf_Miner01_r8     objs=55  excluded=False     (54 meshes + 1 armature)
image r8 (512,512) packed=True packed_bytes=11389 file_bytes=11389 equal=True
material backface_cull=True interpolation=Closest extension=EXTEND SpecIOR=0.50
armatures=['SK_VoxelDwarf_Miner01_r8']  actions=[]
```

**r5, r6 and r7 are untouched.** 53 numbered progress renders were written, one per modelling
call, each preceded by `wm.save_mainfile()`, so the file on disk never trailed the screen by more
than one part. **No generator script exists**: there is no `dwarf_r8.py` and no part-spec file.
The authoring primitives (`loft`, `prism`, `wedge`, `hexa`, `mirror`, the island packer, the
paint pass) lived in the session's `driver_namespace` and were never written to disk.

**Seven of the 53 calls are declared repair calls** that re-author several parts at once rather
than one. That is a departure from §8's "one part per tool call" and the reason is §7.1: the
cull had to be undone across 40 parts, and re-running the identical authoring specs is a repair
pass, not new authoring. Every one of them says so in the call.

## 1. The measurement frame

`H = 1.200 m`, sole at `z = 0`, centre line `x = 0`, **`+Y` forward**, his right is `+X`. One
source pixel of `dwarf-ortho/` is `H/140 = 8.571 mm`. `front.png` column 77.5 is the centre line
and **row 146.94 is z = 0**, so `z = (146.94 − row) × 8.571 mm`; `side-left.png` column 58 is the
depth centre.

Both ortho crops were re-read as classified character maps at source resolution rather than
taken from any table. That read produced four numbers that changed the model, and all four are
things the sheet's own table does not carry:

| read off | what it says | what I had done first |
|---|---|---|
| `front.png` rows **24–31**, cols 60–72 | the dark mass at the eye is **69 mm tall**, not the 26 mm brow band r7 painted — brow *plus* a socket nearly as dark | brow shelf cut 13 mm deep; **widened to 36 mm and the socket dropped to meet the cheek at 0.962** |
| `front.png` row 26 → row 31 | the brow's **underside falls 43 mm** from temple to nose and thickens 26 → 69 mm | my first brow fell 22 mm and read as a level bar; **rebuilt** |
| `side-left.png` rows 29–41, cols **57–62** | the ear is **43 mm deep** | I had inherited r7's 100 mm; **corrected** (§7.5) |
| `front.png`, the shaft corner to corner | the pickaxe lies about **17° off HORIZONTAL** | every carry pose I guessed was near-vertical and put the blade across the face; **measured, then solved** (§7.7) |

## 2. What geometry carries and what texture carries

The division is the round's organising idea and it decided every call:

- **Geometry carries FORM.** The brow shelf, the socket depth, the nose's planes, every hem lip,
  buckle frame, lock, binding, sole, heel, toe cap, cuff and step in the crown.
- **Texture carries COLOUR and micro-value.** The pupil, the eye white, the lip line, the waist
  panel's motif, the cheek warmth, the temple and socket shadow, metal wear.

**The test the brief gives — would it survive the key moving — decided three arguments against
me.** All three were cases where I had reached for paint and the sheet wanted form: the socket
(painted flat, rebuilt as a 30 mm recess), the brow (painted as a bar, rebuilt as a three-segment
wedge that falls 43 mm), and the nose (painted at the lightest value on the figure, which is
**not** what the sheet does — see §3.1 item 6).

### 2.1 The resolution floor, and where it bit

**One source pixel is 8.571 mm.** The floor is not a triangle count and I did not treat it as
one. Concretely, the features that exist *because* the sheet resolves them at one or two pixels:
the overtunic's hem edge (7 mm proud), the skirt's three panel ledges, the hem lip (6 mm), the
boot welt (4 mm) and toe cap, the cuff lips top and bottom, the collar's rolled rim, the ear's
helix notch, the pickaxe's three binding bands and two grooves, the lantern's four corner posts.
None of those is 8.571 mm because a number said so; each is there because the drawing has it.

## 3. The twenty-five items of §3

### 3.1 The head, rebuilt — items 1–7

**The head is no longer a box.** `r8_skull` is a 13-ring loft, 62 faces after the cull, and its
front face is a sequence of measured ledges rather than one plane:

| z | what the ring does | measured against |
|---|---|---|
| 1.106 → 1.058 | forehead, set **back** at y +0.178 | hairline row 18, brow row 24 |
| **1.058** | **LEDGE — the brow shelf steps out to y +0.196** | rows 24–27 |
| **1.020** | **LEDGE — the socket steps back to y +0.166, 30 mm deep** | rows 26–31, the dark block |
| **0.962** | **LEDGE — the cheek comes forward to y +0.190** | eye whites rows 33–34 sit on this shelf |
| 0.930 → 0.886 | jaw and chin, width ±0.130 → ±0.106 | — |

1. **A brow shelf that projects** — 36 mm of it, at y +0.196 against a socket floor at +0.166.
2. **Recessed eye sockets** — the eye plane sits **30 mm inside** the shelf, and the brow part
   on top of it reaches y +0.213, so the total overhang over the eye is **47 mm**. At 60 px
   figure height that is **2.3 px of cast shadow**, which is the §2 argument for geometry at
   small sizes and it holds up in `readability.png`.
3. **Cheekbones and a jaw** — width runs ±0.137 at the temple to ±0.106 at the chin and the
   corner chamfer opens 19 → 30 mm top and bottom. **No side of this head is a single vertical
   plane.**
4. **Brows as their own parts** — `r8_brow.L` / `.R`, three overlapping wedge segments each, 17
   faces. Thin (14 mm) at the temple, **48 mm at the nose**, underside falling 43 mm across the
   span. Separate objects, so they swap.
5. **Eyes with form** — `r8_eye.L` / `.R`: the eye plane is set back 6 mm with a **lid stepping
   8 mm forward above and below it**, so the lid throws its own shadow independently of the brow.
   **The pupil, iris and white stay painted**, as ruled. The map puts the white outboard
   (|x| 0.102–0.126) and the dark inboard, which is what row 33 reads outward-to-inward:
   `S S S m S W W W m # # #`.
6. **A nose with planes** — 23 faces: a bridge that starts buried between the brows and walks
   70 mm forward to the tip, a **ledge at the tip where the nostril wings step out and the front
   steps back**, an underside sloping to the moustache. It stands **59 mm proud of the cheek**.
   **Its paint was corrected twice.** I first gave the whole front `#F6E6D6`, the lightest value
   on the figure; rows 34–42 classify as ordinary skin `S`, the same value as the cheeks. **In
   the reference the nose reads by the shadows either side of it, not by being bright.** The
   light value is now the 41 mm tip only.
7. **A moustache as its own part** — `r8_moustache`, a centre block under the nostrils plus two
   ends that **droop**, lower and further back as they go outboard. It stands clear of the beard
   front so the lit lip band at rows 49–51 stays visible below it.

**The face texture was re-authored, and here is what each island got.** The head's `+Y` quad is
gone; the face is now **14 islands at 400 px/m**. That is 80 % of r7's 500 px/m and the trade is
deliberate: the features r7 needed 500 px/m for — the brow, the socket, the nose's form — are
**geometry now**, and what is still painted there is the eye white (15 px across), the pupil, and
four value steps. The face did not lose readability; `vs-face-front.png` and
`vs-face-tq.png` are the evidence, and the three-quarter plate is the one that matters, because
it shows form the front cannot.

### 3.2 Hair, beard, moustache — items 8–9

8. **The beard is a bush.** r7 named its own defect: right width, right length, right taper, four
   slabs. `r8_beard` is **96 faces**: the mass carries **six crisp ledges** instead of four, and
   **seven tapered locks** break the outline. **The tip stays at z 0.557 = 0.464 H, the sheet's
   figure exactly** — the locks ragged the silhouette *above* the measured length rather than
   lengthening the beard.
9. **The hair is a stepped mass**, four named objects: `r8_hair` (a crown with **four hard
   ledges**, from ±0.202 at z 1.140 up to ±0.104 at 1.200, which is the dome `front.png` rows
   10–16 and `side-left.png` rows 7–16 draw), `r8_fringe` (a flat-bottomed band at z 1.104 with
   an 8 mm cut edge — row 18 is flat across cols 63–92, so a flat fringe is faithful), and
   `r8_hair_lobe.L`/`.R` (the lobe plus **two locks ending at different heights**, 0.848 and
   0.860). **Its front edge in profile holds at y = +0.111.** That is the single most expensive
   number in the file and it was not rediscovered — it was inherited from r7 §7.3 and obeyed.

### 3.3 The body — items 10–13

10. **The torso has a chest, a waist and a collar.** [was 16 faces] `r8_torso` is a 9-ring loft:
    narrowest at **z 0.480, the belt line**, widest at **0.730**, stepping in at 0.766 to a
    shoulder shelf. **The neck opening is a real hole with a rim**: `r8_collar` is four
    overlapping bars around a genuine aperture, inner ±0.080 x / ±0.068 y against a neck of
    ±0.075 / ±0.060 — **you can see down into it**.
11. **The legs are not single boxes.** [was 8 faces each] 80 faces each: thigh, **a knee that
    bulges out at z 0.330 and back in at 0.372**, and a calf. 0.330 is the rig's `knee.L/R`
    site, so the deform loop and the silhouette step are the same ring.
12. **The arms have a cuff and a hand that reads as a hand.** `r8_arm` is octagonal with an
    **elbow ledge at z 0.612** (the rig's elbow); `r8_cuff` is a turned band with a lip top and
    bottom; `r8_hand` has a **wrist step at 0.424**, knuckles stepping back in at 0.348, and a
    **thumb as its own tapered shell** on the front-inboard corner where a hanging fist puts it.
13. **The neck is not three faces.** [was 3] 48 faces: an **octagonal** column, chamfered so it
    is cylinder-ish, tapering into the jaw, with a **ledge at 0.872 that is the jaw shadow** —
    the head's underside sits on it.

### 3.4 The clothing — items 14–19

14. **The tunic reads as layered.** `r8_overtunic` is a vest 5–8 mm proud of the under-tunic
    whose bottom is a **modelled ledge**, not a painted stripe, and whose top steps in at its own
    shoulder shelf. **Two greens**: `#5F7A6A` under, `#4C6355` over.
15. **Shoulder pieces read as separate from the sleeve**, with an **overhanging lip and a step
    under it** at the shoulder line. Outer face at |x| 0.317 → **0.634 m = 0.528 H, the sheet's
    figure exactly**.
16. **A hem lip** — `r8_hem`, its own object, 6 mm proud with a ledge top and bottom. **The
    skirt's panels are stepped**: three hard ledges on the way down, not a smooth flare.
17. **The waist panel** — `r8_waistpanel`, 6 mm proud with a modelled edge, carrying the square
    motif **in paint**, which is exactly the division §3.17 asks for. The sheet draws **two**
    such panels and both are delivered: the buckle plate at rows 91–103 and the lighter skirt
    panel at rows 110–118.
18. **The belt is leather with a raised buckle and a strap end.** [was 5 faces — a slab] Four
    objects: `r8_belt` (band with a ledge at each edge), `r8_buckle` (a **recessed centre plate,
    four frame bars 12 mm proud of it, and a pin 10 mm proud of those**), `r8_belt_tail` (laps
    the band and hangs 54 mm below it, tapering to a cut tip).
19. **The boots have a sole, a heel, a toe cap and a cuff** — all four, across three objects.
    The **toe cap** is a ledge across the instep at z 0.056; the **cuff** is the widening
    `front.png` rows 126–132 actually draw (cols 47–75 against the boot's 49–73); the **sole** is
    a welted plate; the **heel** is a second shell standing to z 0.062 and 12 mm proud at the
    back, where a profile reads it.

### 3.5 The gear — items 20–23

20. **The pickaxe** [was 42 faces, 1.08 m of prop] is **226 faces over three objects**. A head
    with **real facets** — each blade is three segments losing thickness, height and altitude
    together so the tip is an **edge**, not a cut-off block — a **poll** behind the eye, **three
    binding bands and two grooves** where the head meets the shaft, a shaft **tapering** 40 → 30
    mm, and a flared **butt cap**. Blade span 0.460 m = 0.383 H against `front.png`'s 0.38.
    **It still reads edge-on from the front**, and per §5 that is a pose, not a defect.
21. **The pack has a flap, buckles and a roll.** `r8_pack` (narrowed base, chamfered gusset),
    `r8_pack_flap` (14 mm proud with a hard **flap edge** — the strongest line on `back.png`),
    `r8_pack_buckle` (strap, buckle plate, keeper), and **`r8_bedroll`** — a lashed roll across
    the top with two straps, which `side-left.png` draws and **no round has built before**.
22. **The straps have buckles and keepers**, and they **sit on the shoulder**: three faceted
    segments off the pack top, over the cap, converging inboard down the chest as real straps do.
23. **The lantern is not a flat orange rectangle.** [was 29 faces] Five objects, 92 faces: a
    **frame with four corner posts** and two rails, a flared foot, **glass set back 8 mm inside
    the posts**, a **cap** in two hard steps, and a **bail with a real square hole** whose
    crossbar runs up into `r8_hand.L`. **The flame is a COLOUR and never an emitter** — it is a
    band 4 mm proud of the glass and inside the posts, so it is visible from every side with no
    interior geometry and no light source.

### 3.6 What stays paint — items 24–25

24. **Skin is not uniform.** Cheek warmth `#DCBCA0` at 0.060 < |x| < 0.112, temple shadow
    `#C4A88F` beyond |x| 0.112, deep socket `#7A5F46` at z ≥ 0.985, eye shelf `#9E8571` below it,
    lower cheek in shadow above the beard. Crisp steps, no gradient across any part.
25. **The pupil, the eye white, the lip line, the waist panel's motif, metal wear.** The lip is
    a `#241A12` slot with a `#6B5039` lip below it and `#47341F` under that — which is r7 §12.3's
    own prescription: *the mouth reads as the bottom of a two-value shape rather than a 1 px
    line*. **Cloth weave was deliberately not painted**: §3.7 forbids gradients and a weave at
    64 px/m is noise, which §4 says a number can be satisfied by and a reviewer cannot.

### 3.7 The paint itself

**One material `M_VoxelDwarf_r8`, one image `r8`, 512 x 512, PACKED, `Closest` interpolation,
backface culling on, Specular IOR Level at its 0.5 default.** Roughness is 0.9 — my choice, so a
flat albedo under one sun has no sheen; it emits no extension.

**512 and not 256, and here is why.** The island set's raw area is **84,797 px²** even with the
body tier at 44 px/m. 256 x 256 holds 65,536. **256 cannot hold this figure**, so 512 is forced
rather than chosen. Having been forced to it I raised the body tier **44 → 64 px/m** so the extra
area is used rather than wasted: the final pack is **1,259 islands ending at row 292 of 512**.

**Texel density of every region:**

| region | density | why |
|---|---|---|
| every flat-coloured face — the body, clothing, gear, hair, beard | **64 px/m** | nothing on them to resolve; this is 1.8x r7's 35 px/m and buys the boundaries between painted regions inside a face |
| the skull's `+Y` faces — **the face** | **400 px/m** | the eye white is 15 px across; the form that needed 500 px/m in r7 is geometry now |
| the eyes' `+Y` faces | **400 px/m** | white and pupil |
| the lantern's glass and flame, all faces | **300 px/m** | |
| the buckle's and waist panel's `+Y` faces | **300 px/m** | the square motifs |
| the nose's `+Y` faces | **260 px/m** | the tip/bridge step |
| the brows' and moustache's `+Y` faces | **220 px/m** | |

Every island is projected along its face's dominant axis and painted **by a rule expressed in the
same world coordinates the geometry is built from** — "the socket runs z 0.962–1.020" is one
statement that places paint where the measurement put form. The texture cannot drift from the
model because they are addressed in one space. **The map's unused area is filled with the
approved skin value**, deliberately, so the checker's colour census carries no stray.

**29 colours, and the checker printed all of them.** The approved ten are the base; everything
else is a declared value step, and 24 of the 29 are carried over verbatim from r7 §2 because
those round-tripped exactly. New this round: `#4C6355` overtunic (the second green §3.14 asks
for), `#94A08F` waist panel, `#B87C2C` lantern glass.

## 4. Readability at 100 px and 60 px

`src-assets/renders/r8/readability.png` — front and three-quarter, key-lit, at 100 px and 60 px
of figure height, nearest-neighbour.

At **100 px** every §3 feature reads: brows, eye whites and pupils, the nose's lit front against
its shaded flanks, the moustache and the mouth, the stepped beard and its locks, the ears, the
collar, the belt and buckle, the waist panel, the hem, the boot cuffs, the lantern, the pick.

At **60 px**, honestly:

| feature | reads? | note |
|---|---|---|
| brows | **yes** | the darkest thing on the face, and now the *shape* reads, not just the value |
| the socket under the brow | **yes** | this is new — 47 mm of overhang is 2.3 px of shadow, and it survives the key moving |
| eye white + pupil | yes | a light/dark pair |
| nose | yes | 3 px of profile projection |
| moustache, mouth | yes | the two-value shape holds where r7's 1 px line was marginal |
| beard taper and locks | yes | |
| belt buckle, waist panel, hem | yes | as value steps, because they are modelled steps |
| **the pickaxe's bindings** | **no** | 3 bands in 100 mm is below one pixel at 60 px. They are for the gear close-up and the 100 px read, and I am not going to claim otherwise |
| **the lantern's corner posts** | **marginal** | the frame reads as a dark surround; the individual posts do not |

## 5. The parts, and the census

**54 objects, 2,120 faces, 3,955 triangles.** Every swappable feature is its own named object, so
the combination layer gets the decomposition for free. `bone` is the §6 weight assignment.

| part | faces | tris | size m (X,Y,Z) | reference feature it carries | bone |
|---|---|---|---|---|---|
| `r8_skull` | 62 | 119 | 0.274 x 0.327 x 0.246 | brow shelf, socket, cheekbone, jaw | head |
| `r8_brow.L` / `.R` | 17 | 34 | 0.102 x 0.027 x 0.059 | the brow, falling 43 mm to the nose | head |
| `r8_eye.L` / `.R` | 11 | 22 | 0.060 x 0.032 x 0.042 | lid and socket edge | head |
| `r8_nose` | 23 | 46 | 0.084 x 0.099 x 0.146 | bridge, tip, nostril wings | head |
| `r8_moustache` | 22 | 44 | 0.216 x 0.054 x 0.058 | the moustache and its droop | head |
| `r8_ear.L` / `.R` | 17 | 34 | 0.038 x 0.056 x 0.092 | the helix rim and its notch | head |
| `r8_hair` | 56 | 112 | 0.404 x 0.381 x 0.355 | the stepped, domed crown | head |
| `r8_fringe` | 11 | 22 | 0.264 x 0.110 x 0.064 | the hairline at row 18 | head |
| `r8_hair_lobe.L` / `.R` | 36 | 72 | 0.078 x 0.300 x 0.264 | side lobe + two locks, front edge +0.111 | head |
| `r8_beard` | 96 | 192 | 0.396 x 0.228 x 0.333 | six taper ledges + seven locks | beard |
| `r8_neck` | 48 | 88 | 0.150 x 0.118 x 0.154 | the column and the jaw shadow | chest+neck+head |
| `r8_collar` | 56 | 112 | 0.252 x 0.224 x 0.048 | the neck opening's rim | chest |
| `r8_torso` | 29 | 58 | 0.384 x 0.346 x 0.320 | chest, waist, shoulder shelf | hips+spine+chest |
| `r8_overtunic` | 34 | 68 | 0.396 x 0.358 x 0.245 | the layer edge | spine+chest |
| `r8_shoulder.L` / `.R` | 56 | 96 | 0.181 x 0.204 x 0.062 | the shoulder line's lip | shoulder.L/R |
| `r8_arm.L` / `.R` | 51 | 102 | 0.124 x 0.180 x 0.383 | the sleeve and its elbow | shoulder+elbow |
| `r8_cuff.L` / `.R` | 48 | 80 | 0.118 x 0.182 x 0.050 | the turned sleeve cuff | elbow.L/R |
| `r8_hand.L` / `.R` | 62 | 108 | 0.102 x 0.207 x 0.108 | wrist step, knuckles, thumb | hand.L/R |
| `r8_belt` | 32 | 56 | 0.398 x 0.350 x 0.076 | the leather band's two edges | hips |
| `r8_buckle` | 33 | 66 | 0.156 x 0.038 x 0.094 | frame, recessed plate, pin | hips |
| `r8_belt_tail` | 9 | 18 | 0.036 x 0.018 x 0.094 | the strap end | hips |
| `r8_waistpanel` | 10 | 20 | 0.232 x 0.038 x 0.084 | the lighter skirt panel | hips |
| `r8_skirt` | 48 | 96 | 0.520 x 0.442 x 0.152 | three stepped panels | hips |
| `r8_hem` | 32 | 56 | 0.532 x 0.456 x 0.038 | the hem lip | hips |
| `r8_leg.L` / `.R` | 80 | 144 | 0.178 x 0.192 x 0.322 | thigh, knee bulge, calf | hip+knee |
| `r8_boot.L` / `.R` | 48 | 88 | 0.194 x 0.250 x 0.124 | the toe cap | foot.L/R |
| `r8_boot_cuff.L` / `.R` | 40 | 72 | 0.218 x 0.228 x 0.068 | the turned cuff | foot.L/R |
| `r8_boot_sole.L` / `.R` | 25 | 50 | 0.202 x 0.270 x 0.062 | welted sole + heel | foot.L/R |
| `r8_pickaxe_head` | 76 | 136 | 0.052 x 0.460 x 0.134 | faceted blades, poll | hand.R |
| `r8_pickaxe_binding` | 70 | 124 | 0.046 x 0.042 x 0.100 | three bands, two grooves | hand.R |
| `r8_pickaxe_shaft` | 80 | 128 | 0.044 x 0.044 x 1.050 | taper + butt cap | hand.R |
| `r8_pack` | 32 | 56 | 0.332 x 0.216 x 0.336 | narrowed base, gusset | chest |
| `r8_pack_flap` | 39 | 70 | 0.352 x 0.230 x 0.250 | the flap edge | chest |
| `r8_pack_buckle` | 18 | 36 | 0.076 x 0.026 x 0.246 | strap, buckle, keeper | chest |
| `r8_bedroll` | 50 | 84 | 0.372 x 0.116 x 0.116 | the lashed roll | chest |
| `r8_strap.L` / `.R` | 40 | 80 | 0.120 x 0.524 x 0.254 | buckle and keeper, on the shoulder | chest |
| `r8_lantern_frame` | 47 | 94 | 0.156 x 0.156 x 0.206 | four corner posts, rails, foot | hand.L |
| `r8_lantern_glass` | 5 | 10 | 0.116 x 0.116 x 0.132 | glass set back | hand.L |
| `r8_lantern_flame` | 4 | 8 | 0.124 x 0.124 x 0.052 | the flame cell, a colour | hand.L |
| `r8_lantern_cap` | 21 | 42 | 0.140 x 0.140 x 0.060 | the cap's two steps | hand.L |
| `r8_lantern_bail` | 15 | 30 | 0.076 x 0.024 x 0.076 | the bail's square hole | hand.L |
| **whole figure, 54 parts** | **2,120** | **3,955** | 0.766 x 0.677 x 1.200 | **13 % of the 30,000 ceiling** | |

**Proportions against the sheet, measured on the built geometry:**

| | r8 | sheet | |
|---|---|---|---|
| head + hair block | 0.296 H | 0.293 | +1 % |
| head width with hair | 0.337 H | 0.336 | +0.3 % |
| ear to ear | 0.380 H | 0.383 | −0.8 % |
| shoulders over the caps | 0.528 H | 0.528 | exact |
| stance, boot outer to outer | 0.397 H | 0.398 | −0.3 % |
| beard tip | 0.464 H | 0.464 | exact |
| beard widest | 0.330 H | 0.343 | −4 % |
| face skin, fringe to beard | 0.178 H | 0.186 | −4 % |

**On the triangle budget, since §4 says being thrifty is the risk.** 3,955 is 13 % of the
ceiling and sits inside the brief's own 2,000–5,000 estimate for the §3 list. I did not stop
because of a number — every ledge the sheet draws is in, and the parts where I could most easily
have spent more are named in §12. What I did *not* do is subdivide to look busy: §4 warns that a
count noise can satisfy will be satisfied by noise, and round 6 is the precedent.

## 6. The rig

**`SK_VoxelDwarf_Miner01_r8`, one armature in the round's collection, nineteen joints, the
contract names exactly**, read back out of the written GLB by the exporter:

```
beard, chest, elbow.L, elbow.R, foot.L, foot.R, hand.L, hand.R, head,
hip.L, hip.R, hips, knee.L, knee.R, neck, root, shoulder.L, shoulder.R, spine
```

Hierarchy: `root → hips → spine → chest → {neck → head → beard, shoulder.L/R → elbow → hand}`,
`hips → hip.L/R → knee → foot`. **Every joint site sits on an edge ring this figure actually
has**, taken from its own ledges rather than copied from r7: elbow **0.612** is the sleeve's
elbow ledge, knee **0.330** is the leg's knee bulge, hand **0.424** is the wrist step, shoulder
**0.806** the arm's shoulder line, head **0.886** the skull's floor, beard **0.875** the beard's
top ring. `root` carries no geometry — it is the figure's origin, not a deform joint.

**ONE mesh, RIGID weights: 0 vertices are anything other than exactly one joint at 1.0.**
Verified in the scene and again by the exporter on the joined mesh.

**Every part is weighted whole except the five that genuinely span a joint**, and each of those
splits on a ring that already existed: `r8_torso` at 0.480 and 0.680, `r8_overtunic` at 0.640,
`r8_arm` at 0.612, `r8_leg` at 0.330, `r8_neck` at 0.800 and 0.872.

**The head loop moved, and here is where it went.** r7 put it on `r8_neck` alone because cutting
the head's `+Y` quad would have split the single face island. The face is multi-island now, so
the loop sits where the deformation needs it: **the neck's top segment, z ≥ 0.872, rides with
`head`.** That keeps the skull/neck junction sealed when the head turns instead of opening a gap
at the jaw — `joint-neck-head.png` is the evidence, at 0.35 rad of neck and 0.70 rad of head
turn.

**The props are weighted like anything else**: the pickaxe's three objects to `hand.R`, the
lantern's five to `hand.L`, the pack and bedroll and both straps to `chest`. A posed arm takes
its prop with it — visible in `pose-carry-front.png`, where the pick swings with the right arm
and the lantern hangs from the left.

**The mesh ships in the NEUTRAL stance.** The GLB carries the rest pose and **no animation**;
`bpy.data.actions` is empty and every pose bone's `matrix_basis` is identity, checked after the
last save. **The carry is a pose and is delivered as renders.**

**Proving the weights.** `joint-*.png` is one deflection render per joint group, each from the
camera that actually shows its bend. **No tearing, no hole at any joint, nothing following the
wrong bone.** The check that mattered is in §7.1: the first attempt *did* show a hole, it was a
defect in my cull rather than in the weights, and it is fixed rather than reported as a caveat. I
re-verified with backface culling on against a **magenta world** — zero magenta pixels anywhere
inside the silhouette in three views at full deflection.

## 7. Conflicts, defects and departures

**7.1 MY OWN, and it is the round's real finding: a buried-face cull must be scoped BY JOINT,
not by part.** §5 handed me the six-direction enclosure test and I used it correctly — and still
got it wrong, because I asked the wrong question. I culled every face buried inside *any* other
part. **A face buried inside an occluder on the same joint can never be exposed; a face buried
inside an occluder on a different joint is exposed the moment that joint moves.** 57 of
`r8_leg`'s 80 faces were inside the skirt (`hips`) and the boot (`foot`), and the first
hip-and-knee deflection render showed **straight through the raised thigh**.

This is not a small distinction. The "no interior geometry" clause is about geometry sealed where
nobody can see it, and **for a rigged figure "where nobody can see it" is a property of the
skeleton, not of the rest pose.** The fix cost 40 parts re-authored and the re-scoped cull
removes **320 faces instead of 705**, perfectly symmetric, with the figure verified solid at full
deflection. The rule for the parts library: **cull only against occluders that share the face's
joint, and never cull a face that itself spans one.**

**7.2 MY OWN, and it is r7 §7.7 wearing a different hat: my first cull mutated meshes while
iterating.** Each part was culled against occluders the cull had already opened, so the result
was correct for whatever ran first and conservative for everything after. **The tell was the one
r7 told me to look for** — `r8_shoulder.L` came out at 50 faces and `.R` at 52 — and rebuilding
the pair showed the test itself agrees exactly on both sides. The re-scoped pass computes every
removal against the closed geometry and applies them afterwards. **r7's lesson was "do not run it
repeatedly"; the sharper statement is "never let it read its own output", and a single pass over
a mutating scene does exactly that.**

**7.3 A zero-height ledge produces a COLLINEAR triangle when the step moves one coordinate.**
The exporter failed with `degenerate faces 6`. A ledge is two rings at the same z, so any side
wall between them has zero height; when the step moves both x and y the collapsed quad vanishes
harmlessly, but when it moves **one** coordinate it collapses to a triangle whose three points
share an x and a z. Only the skull's three face ledges and the boots' toe cap move a single
coordinate — exactly the faces reported. **Fixed by giving those ledges 0.4 mm of real height**,
which is 1/21 of one source pixel: the step reads identically and the topology is honest.

**7.4 A NEW silent-failure trap, and it is r7's inverted: `Image.pack()` does not replace an
already-packed copy.** After the first paint I repainted three times. Each time `img.save()`
wrote a correct PNG to disk and `img.pack()` returned without error — and **the bytes embedded in
the `.blend` never changed**. The `.blend` therefore shipped the *first* paint's packing against
UVs that had been repacked twice since, and the delivered flat render came back with the belt
buckle sampling the lantern's island. Caught by eye on `vs-ortho-front.png`, then confirmed
numerically: **packed 11,667 bytes against a 10,543-byte file.**

**This is exactly the family the exporter's new clause was written for and it slips past that
clause**, because the GLB *does* carry an image and the line *does* print a name — the image is
simply the wrong one. **It is not the exporter's bug and I am not proposing a change to it
blind**, but the cheapest gate I can see is to compare the packed size against the file on disk,
or to hash the image buffer into the export line. My own fix was to `unpack(method='REMOVE')`
before every save-and-pack, and the final state is verified equal.

**7.5 The ear's depth was inherited, not measured, and was 2.3x wrong.** I took r7's
y −0.030..+0.070 (100 mm). `side-left.png` rows 29–41 put the ear's pale column at **cols 57–62,
43 mm**. The side view is the authority for every Y dimension and this is the one number r7
carried that it had not read off the profile. Corrected to 56 mm including the rim.

**7.6 The sheet's own annotations stayed unreliable and the drawings stayed trustworthy.** Every
number in this report is read off pixels. The two places the drawing corrected me outright are
the brow's fall (§1) and the pickaxe's 17° (§7.7).

**7.7 A prop's orientation cannot be dialled in through a bone chain with Euler angles, and I
wasted four iterations proving it.** The pick is rigidly bound to `hand.R`, so its final
direction is the composition of shoulder, elbow and hand rotations; every attempt to steepen the
shaft also swung it behind the beard. **The fix is to place the hand with the arm and then set
the hand's WORLD matrix** so its −Y axis — the shaft's direction in the hand's rest frame — lies
along a measured `SHAFT_DIR`. The same one line, used the other way, keeps the lantern **hanging
vertical** from `hand.L` regardless of what the left arm does. Both are in `render_r8.py` with
the reasoning attached. **And the reason four guesses all failed is §1's last row: I had the pick
near-vertical when `front.png` draws it 17° off horizontal.**

**7.8 The bare neck in profile measures 0.009 H, and r7 measured 0.027 H. I am reporting the
smaller number.** Check 5 is withdrawn (§5) so this is not a gate, and r7 §7.2 already settled
that the figure the check asks for is unreachable with a shoulder at 0.700 H. What I did control
for: **nothing shoulder-borne on this figure goes above z 0.854** — the arm's top was dropped to
0.838 and the shoulder cap to 0.848 specifically so the strap could crest at 0.854 and leave
32 mm of band under the head floor at 0.886. **What closes it anyway is the hair lobe**: in an
orthographic profile the outermost |x| wins, the lobe reaches |x| 0.202 against the neck's 0.075,
and it hangs to z 0.848 because the head-block measurement (0.296 H) requires the lowest hair to
be there. **The lever r7 identified was the strap; the remaining lever is the lobe, and it is
load-bearing for a proportion that matches the sheet exactly.** I did not trade a measured
proportion for a withdrawn check.

**7.9 `session_tokens.py`'s Windows slug defect is still open**, unchanged from r6 §7.7 and r7
§7.6. Worked around with `--transcript`. Not mine, not chased.

**7.10 `check_asset.py` passes, and its palette census is genuinely useful.** r7 could not reach
the geometry clauses at all. This round it exits 0 and prints all 29 colours most-painted first,
which is the line Wolf reads against the sheet — and it is what let me confirm there are **no
stray colours**, because the census and my declared palette are the same list.

**A departure, declared as §1 asks. The figure stays hard-edged and flat-shaded.** No bevel, no
subdivision, no smooth normals; rotated and sheared hexahedra *are* used where a taper needs them
(brows, beard locks, pickaxe blades, strap segments), which the 2026-09-11 ruling permits. The
reason is unchanged from r7: `dwarf-frames/` is the reference's own 3D interpretation and it is
hard-edged, sitting in a world of voxel pines. At 13 % of the ceiling, reversing this costs
nothing that has been spent.

## 8. The export and the checker, verbatim

```
EXPORT D:\Workspace\frostvein\src-assets\export\SM_VoxelDwarf_Miner01.glb
  parts joined      54 -> 1 mesh 'SM_VoxelDwarf_Miner01_r8'
  object / mesh     SM_VoxelDwarf_Miner01_r8 / SM_VoxelDwarf_Miner01_r8
  materials         M_VoxelDwarf_r8
  texture image     r8   in the GLB: T_VoxelDwarf_r8
  triangles         3955  of 30000 budget
  size m (X,Y,Z)    0.766 x 0.677 x 1.200
  blender min Z     0.000000   (glTF min Y)
  blender centre XY 0.000000, 0.000000   (glTF centre X, Z)
  topology          n-gons 0   non-manifold edges 0   loose verts 0   loose edges 0
                    degenerate faces 0   flipped winding 0   missing UV layer 0
                    unapplied modifiers 0
  rig               joints 19   missing joints 0   unexpected joints 0   unweighted verts 0
                    soft-weighted verts 0   verts weighted to a non-bone 0
                    joint names in the GLB: beard, chest, elbow.L, elbow.R, foot.L, foot.R,
                    hand.L, hand.R, head, hip.L, hip.R, hips, knee.L, knee.R, neck, root,
                    shoulder.L, shoulder.R, spine
  GLB min/max       [-0.383, 0, -0.3385] / [0.383, 1.2, 0.3385]   (glTF axes: X, Y up, Z)
  bytes             401832
```

```
FIGURES src-assets\export\SM_VoxelDwarf_Miner01.glb size_m=0.8x1.2x0.7 min_y_m=0.000000
  centre_x_m=0.000000 centre_z_m=0.000000
  palette=#E9D2BB,#5F7A6A,#4C6355,#C4A88F,#B87C2C,#5E4632,#6B5B49,#474B41,#7A6752,#473C31,
          #7A5F46,#A9B2AC,#47341F,#9E8571,#664C36,#707572,#F0A63C,#322A22,#94A08F,#63695B,
          #DCBCA0,#8B6B50,#523D2B,#34271C,#FFFFFF,#231A12,#6B5039,#F6E6D6,#241A12
  tris=3955 verts=6922 mesh=SM_VoxelDwarf_Miner01_r8 profile=painted-map
exit 0
```

## 9. The renders

`render_r8.py`, copied from `render_r7.py` and keeping both its passes: **flat Workbench for
measurement** (every number in §5 and §7.8 is taken off it) and **EEVEE key-lit for judgement** —
one sun at 4.2 W with a 3° cone along `f088`'s key, plus a 1.1 W shadowless fill. Absolute paths,
revision in every path, background driven through the background node.

New this round, as §9 asks: the face plate in **three-quarter as well as front**; the head beside
`f104` at matched scale; a **gear plate** with the pickaxe, lantern and pack framed individually
beside the sheet's own breakdown; the **posed carry**; **one deflection render per joint group**,
each from the camera that shows its bend.

**One fix to the comparison sheets that r7 did not have.** `_ref_figure()` crops the ortho
reference to the **figure's own rows (7..147 = z 1.200..0.000) before trimming**. r7 trimmed on
colour alone, which lets the sheet's grid lines and dimension arrows into the crop — and any
pixel of those above the crown or below the sole silently rescales a comparison that is supposed
to be matched on height.

## 10. Cost

```
Session token cost  (548e6c76-f965-42e5-841e-15a6f576ec2b.jsonl, tool=claude)  (372 turns, claude-opus-5)
  input (fresh)            744
  cache creation       797,859
  cache read       103,259,061
  output               777,418
  total processed  104,835,082
  wall-clock            84 min  (elapsed, includes idle gaps)
  est. cost             $76.06  (benchmark - verify rates in PRICES)
```

Model from the banner: **Opus 5 (1M context)**, `claude-opus-5[1m]`. Row: `dev-art`. Printed with
`--transcript` because of §7.9; no ledger row written and no cursor advanced. The reading was
taken while writing this report, so it excludes the report itself.

## 11. Deliverables

| # | what | path |
|---|---|---|
| 0 | the live-scene report | §0 above |
| 1 | the source, saved after every part, image PACKED | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture map | `src-assets/blender/textures/T_VoxelDwarf_r8.png` |
| 3 | the render script | `src-assets/blender/render_r8.py` |
| 4 | the exported GLB (gitignored scratch) | `src-assets/export/SM_VoxelDwarf_Miner01.glb` |
| 5 | progress renders, one per call | `src-assets/renders/r8/progress/01..53-*.png` |
| 6 | five final views, flat **and** key-lit | `src-assets/renders/r8/dwarf-{flat,lit}-*.png` |
| 7 | the readability strip at 100 px and 60 px | `src-assets/renders/r8/readability.png` |
| 8 | side by side against the ortho sheet | `src-assets/renders/r8/vs-ortho-{front,side-left}.png` |
| 9 | the face, front and three-quarter, beside `f104` | `src-assets/renders/r8/vs-face-{front,tq}.png` |
| 10 | the body beside the sheet, matched scale | `src-assets/renders/r8/vs-body.png` |
| 10b | the gear close-up beside the breakdown | `src-assets/renders/r8/vs-gear.png` |
| 11 | the posed carry, front and three-quarter | `src-assets/renders/r8/pose-carry-{front,tq}.png` |
| 12 | one deflection render per joint group | `src-assets/renders/r8/joint-*.png` |
| 13 | the per-part census | §5 above |
| 14 | this report | `src-assets/prompts/dwarf-miner-round-8-report.md` |
| 15 | exporter and checker output, verbatim | §8 above |
| 16 | cost | §10 above |

**Nothing was written outside `src-assets/`**, verified by timestamp over the working tree rather
than by `git status`: **no git command was run at all.** Four files outside it carry a
2026-09-13 timestamp — `scripts/bench/check_asset.py`, `scripts/tests/test_check_asset.py` and
two `_bmad-output/` mutation scripts — and all four, together with `export_dwarf.py`, are stamped
**09:36:34**, which is before this session's first write at **09:53:42**. They are the
operator's verified changes, not mine. The changed paths are
`blender/render_r8.py` (new), `blender/SM_VoxelDwarf_Miner01.blend` (+`.blend1`),
`blender/textures/T_VoxelDwarf_r8.png` (new), `export/SM_VoxelDwarf_Miner01.glb` (gitignored),
`renders/r8/**` (new) and this report.

## 12. What I would change next, in order

1. **Close the `Image.pack()` trap in the exporter** (§7.4). It is the third silent texture
   failure in two rounds and the first one the current gate cannot see. Comparing the packed
   size against the file on disk is a two-line clause; hashing the buffer into the export line
   is better. This is a ruling, not something for me to change unasked.
2. **The cull rule belongs in the brief, not in a report** (§7.1). "Cull only against occluders
   that share the face's joint" is the sentence that would have saved this round a 40-part
   rebuild, and it only becomes visible once a figure is rigged — which is to say, it could not
   have been learned before this round and should not have to be learned again.
3. **The pickaxe's bindings and the lantern's posts do not survive 60 px** (§4). If they must,
   the answer is not more geometry — it is a value step in paint on the shaft and the frame, so
   they read as a light/dark pair rather than as three 1 px bands.
4. **The beard is 4 % narrow and the face 4 % short** against the sheet. Both are inherited from
   r7's block and both are inside the sheet's own ±4 % caveat, but they are the two largest
   remaining proportion gaps and they are on the part of the figure the eye goes to first.
5. **The bare neck** (§7.8). If it matters, the lever is the hair lobe's bottom at z 0.848, and
   raising it trades against a head-block measurement that currently matches the sheet to 1 %.
   That is a call for Wolf, not for me.
