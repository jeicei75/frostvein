# Round 5 report — rebuilt from the orthographic sheet

Run through without stopping, as the brief asked. Twelve modelling steps, each saved and rendered.
Headless throughout: Blender 5.2.1 LTS in `--background`, no live MCP session, so the whole round is
reproducible from the committed files.

**Nothing was written outside `src-assets/`, and no git command was run at any point** — with one
accident that I caused and cleaned up, recorded in section 4.7 rather than buried.

---

## 0. The re-measured proportion table

Measured off `src-assets/references/dwarf-ortho/`, before any geometry. **Every number in
`dwarf-model-sheet.md` that came from the video is superseded by these**, and that file has been
corrected in place with the same table plus a SUPERSEDED box over the old one.

### The frame

The figure spans **rows 7 (crown) to 147 (sole)** in both `front.png` and `side-left.png`, so
**140 source pixels = 1.00 H**; on a 1.20 m dwarf one source pixel is **8.571 mm**. `back.png` is
123 px tall for the same figure, so back-view pixels are scaled by 140/123 before use. `front.png`
column 77 is the centre line (X); `side-left.png` column 58 is the depth centre, +Y forward.

Front and back were measured independently and agree: head width 0.336 vs 0.341 H, tunic hem 0.207
vs 0.197 H, stance 0.398 vs 0.398 H. That agreement is the reason I trust the rest.

### Heights, as z/H measured up from the sole

| landmark | z / H | where |
|---|---|---|
| crown | 1.000 | front row 7 |
| crown step 2 / step 1 / main skull top | 0.979 / 0.964 / 0.943 | front rows 10 / 12 / 15 |
| brow | 0.879 | front row 24 |
| eye line | 0.807 | front rows 30–34 |
| ear top / ear bottom | 0.850 / 0.729 | front rows 28 / 45 |
| nose tip, lowest | 0.750 | front row 42 |
| head + hair mass ends | **0.707** | front row 48; back row 44 gives 0.705 |
| shoulder line, top of the sleeve cap | **0.700** | back row 45 |
| sleeve cuff, bare forearm begins | 0.566 | back row 61 |
| **beard tip** | **0.464** | front row 82 |
| belt top / bottom | 0.421 / 0.343 | front rows 88 / 99 |
| pack bottom | ~0.35 | side row 95, back row 90 |
| hand bottom | 0.330 | back row 90 |
| **tunic hem** | **0.207** | front row 118; back row 106 gives 0.197 |
| boot cuff top / bottom | 0.164 / 0.107 | front rows 124 / 132 |
| sole | 0.000 | front row 147 |

### Widths, as width/H

| feature | width / H | where |
|---|---|---|
| head, with hair | **0.336** | front cols 54–100 |
| head, ear to ear | 0.383 | front cols 51–104 |
| crown steps, top down | 0.164 / 0.229 / 0.286 / 0.336 | front rows 7 / 10 / 12 / 15 |
| beard, widest | 0.343 | front rows 44–56 |
| chest, tunic only | 0.317 | back, between the sleeve seams |
| **shoulders, over the sleeve caps** | **0.528** | back cols 26–90 |
| waist / tunic skirt | 0.439 | back cols 32–85 |
| arm span, hands out — the sheet is POSED | 0.772 | back cols 11–105 |
| stance, boot outer to boot outer | 0.398 | front cols 50–105 |
| one leg / one boot | 0.164 | front |
| boot cuff | 0.200 | front rows 125–132 |

### Depths, as depth/H, +Y forward, on `side-left.png`

| feature | depth / H | where |
|---|---|---|
| head, back of hair to front of hair | 0.336 | cols 36–83 — the head is very nearly a cube |
| nose tip, from the back of the head | 0.379 | col 89 |
| beard front | 0.357 | col 86 |
| torso | 0.286 | cols 38–78 |
| tunic skirt | 0.300 | cols 37–79 |
| pack, behind the torso back | 0.186 | cols 12–38 |
| shin | 0.164 | cols 47–70 |
| **boot sole length** | **0.214** | cols 47–76; the toe projects **0.057 H past the shin front** |
| whole figure, pack to nose | 0.550 | cols 12–89 |

### The five defects, answered by measurement

| # | the defect | what the sheet says | what r5 built |
|---|---|---|---|
| 1 | head is one big block | crown steps in at the sides AND at both front and back over rows 7–15 | four stepped boxes, 0.164 → 0.229 → 0.286 → 0.336 H; the **skull under the hair is stepped too**, so lifting the hair leaves a domed head, not a box |
| 2 | side view is a flat blocky stick | head 0.336 H deep, nose to 0.379 H, torso 0.286 H, pack 0.186 H behind it | every depth in the model is one of these numbers; nothing is estimated |
| 3 | neck is missing | **the sheet does not show one** — hair and beard hide it in all four views | built as its own part, and the head raised 3 px so a band of it reads. See 9.2 |
| 4 | shoulders | **0.528 H over the caps against a 0.336 H head** | 0.528 H, and the over-shoulder straps finally have somewhere to run |
| 5 | feet could be longer | boot 0.214 H deep, toe 0.057 H past the shin | 0.214 H, toe 0.057 H proud |

---

## 1. Deliverables

| # | what | path | state |
|---|---|---|---|
| 0 | re-measured proportion table | section 0 above | — |
| 1 | the source, saved after every step | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` | 12 saves |
| 2 | the exporter, one command | `src-assets/blender/export_dwarf.py` | inherited; **one line changed**, `REV = "r5"` |
| 3 | exported GLB (gitignored scratch) | `src-assets/export/SM_VoxelDwarf_Miner01.glb` | 80,060 bytes |
| 4 | progress renders, one per step | `src-assets/renders/progress/01..12-*.png` | 12 sheets |
| 5 | the five final views | `src-assets/renders/dwarf-{flat,lit}-*.png` | 10 files |
| 6 | zoom strip with the speckle metric | `src-assets/renders/zoom-strip.png` | 18.3 / 7.9 / 2.3 |
| 7 | side-by-side against the orthographic | `src-assets/renders/vs-ortho-side.png` | — |
| 8 | this report | `src-assets/prompts/dwarf-miner-round-5-report.md` | — |
| 9 | checker output, verbatim | section 6 | FAIL, as predicted, plus one new failure |
| 10 | cost | section 8 | verbatim |

Authoring helpers, all new, all inside `src-assets/`:

- `src-assets/blender/dwarf_r5.py` — the box specs, in SOURCE-PIXEL coordinates so every number can
  be checked against the sheet by counting 5x5 blocks. Round 4 kept the equivalent as text
  datablocks inside the `.blend`; on disk they are diffable and runnable headless, which is what let
  this round rebuild the whole figure twelve times in under a minute when a convention changed.
- `src-assets/blender/render_r5.py` — round 4's render settings, unchanged, with the view list.
- `src-assets/blender/sheet_r5.py` — progress sheets, the zoom strip, the side-by-side. Blender's
  bundled Python has numpy but no PIL, so composition lives outside it.

The export command, from the repo root, unchanged from round 4:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/export_dwarf.py

---

## 2. What the model is

**1,116 triangles** (r4: 1,214; r3: 14,398) against an LOD0 ceiling of 4,000 — an outcome, not a
target, and nothing was optimised. Fifteen parts joined at export into one mesh, one material
(`M_VoxelDwarf_r5`), one packed 64x64 atlas (`T_VoxelDwarf_Palette_r5`). Height **1.2000 m**, min
Z 0.000000, centred in X and Y. **Zero non-axis-aligned faces, zero smooth-shaded faces**, verified
per part after every step and again at export, where it is a build failure.

Parts: `Torso Neck Head Beard Hair Arm.L Arm.R Leg.L Leg.R Boot.L Boot.R Belt Pack Lantern Pickaxe`.
Sockets: `socket.hair socket.beard socket.pack socket.hand.L socket.hand.R` — the pack socket is new,
because the pack is exactly the kind of thing a seeded variant will want to swap.

Beard and hair lift off and leave a complete head: brow ridges, eyes, nose, cheeks, ears, jaw, and
now a neck. **Progress sheet `03-head.png` is the proof** — it renders the whole figure with the head
and neck on and neither hair nor beard built yet, and the neck is plainly visible in all four views.

What is new this round, beyond the five defects: the skull is stepped under the hair; the hair stops
short of the cheek so the face reads in profile; the pack carries a real flap, buckle, bedroll and
two strap runs taken from `back.png` instead of being invented; the boots have a cuff.

---

## 3. Inherited without change, as instructed

- **Palette**: round 4's atlas and cell coordinates, copied verbatim from its `.blend`; only the
  image datablock was renamed to `…_r5`. No hex was re-derived and none was sampled from the video.
- **Material**: round 4's `M_VoxelDwarf` setup, copied. Backface culling on, so `doubleSided` is
  false. **Specular IOR Level left at its 0.5 default** — the fix that matters; the export carries no
  `KHR_materials_specular` and passes the no-extensions clause.
- **Exporter**: `export_dwarf.py` as it stands. The only edit is `REV = "r4"` → `REV = "r5"`, which
  the naming rule requires.
- **Render settings**: Workbench, flat and studio-lit passes on `#6F7073`, `Standard` view transform,
  same paths and names.

Time spent rebuilding scaffolding: none. The seed step opens round 4's `.blend`, deletes every mesh,
object, collection and text datablock, renames the material and image, and saves. It is one command
and it is repeatable.

---

## 4. Conflicts and defects found in the input documents

All reported in writing, none blocking, and in every case the reference won.

**4.1 The sheet's side-view labels are swapped relative to anatomy.** `side-left.png` —
"Side View (Left Orthographic)" — draws him facing image-right with the pack behind him on the
image-left. Standing on a figure's left and looking at them puts their face on *your* left, so that
drawing is a view of his **right** side. Our render naming follows anatomy (`side-left` is the camera
on his left, matching round 4's and `render_dwarf.py`'s convention), which means **deliverable 7
compares `dwarf-flat-side-right.png` against `side-left.png`** — the two that actually share an
orientation. Labelled as such on the image itself.

**4.2 The front and back views disagree about which hand holds what.** The front view puts the
lantern at the viewer's right, which is his left hand. The back view puts it at the viewer's left,
which is also his… left only if that drawing is mirrored. One of the two is flipped. Not reconciled;
r5 keeps round 4's and the shipped asset's convention — **pickaxe on his right (+X), lantern on his
left (−X)** — and the fixing round can pick a side if Wolf cares.

**4.3 The gear breakdown's labels disagree with the gear as drawn on the figure.** The brief told me
to take the gear proportions from the sheet's own labels, and I did, but they do not match the art:

| | breakdown label | the same tool, measured in `front.png` |
|---|---|---|
| pickaxe, overall length | 0.83 H | ~1.04 H |
| pickaxe, head span ÷ length | 0.89 | ~0.54 |
| lantern, height | 0.33 H | 0.26 H |

Followed the labels for size (0.83 H and 0.33 H) and the in-situ drawing for the pick head's
proportion, because a head 0.89 × a 116 px shaft is 103 px wide and would be as broad as his arm
span. The consequence of the 0.83 H length is section 7.3.

**4.4 `check_asset.py` has a second failing clause, and it is a real contradiction with the brief.**
The naming clause requires the GLB's **file basename to equal the published mesh name**. The brief
fixes the filename as `SM_VoxelDwarf_Miner01.glb` and requires the mesh to carry `r5`, so
`'SM_VoxelDwarf_Miner01'` ≠ `'SM_VoxelDwarf_Miner01_r5'` and the clause fails. Round 4 never reached
it — the grid clause fails first and the checker stops there — so this is newly surfaced, not newly
caused. **Not worked around; the checker was not edited.** Either the revision token moves out of the
mesh name or the clause learns about revisions; that is an orchestrator decision.

**4.5 "6 spare slots" was not reachable.** The brief says the approved ten leave six spare under the
16-cell ceiling, and in the same section says to copy round 4's atlas verbatim — which already spends
five of them. Copying won, so there was exactly **one** spare. It is unspent; see section 5.

**4.6 The sheet's decorative labels, confirmed.** The README's warning is right and I can put numbers
on it: the front view's vertical `0.8x dwarf height` labels the dwarf's own height, and `12 Voxels`
across the body implies a 15-voxel dwarf the art exceeds. The *horizontal* labels are sound — the
front view including the pickaxe measures 1.079 H against `1.0x`, each side view 0.643 H against
`0.67x` — so they are a usable sanity check even though the vertical one is not.

**4.7 My own defect, reported because it broke the round's own rule.** Blender resolves a *relative*
`render.filepath` against the .blend, not the shell's working directory, so the first "final" render
pass silently wrote ten PNGs to `C:\src-assets\renders\` — outside `src-assets/`, outside the repo.
I found it because the files I was measuring were round 4's, deleted `C:\src-assets` entirely, and
`render_r5.py` now calls `os.path.abspath` on the output directory. It also cost me a detour:
several paragraphs of analysis in this session were measurements of round 4's renders. Recorded
because "nothing written outside `src-assets/`" is a hard rule and it was broken for about fifteen
minutes.

One more, smaller: `uv run` installed `pillow` and `numpy` into the repo's pre-existing, gitignored
`.venv/` before I redirected `UV_PROJECT_ENVIRONMENT` to the scratchpad. No tracked file changed.

---

## 5. The palette — 15 of 16 cells, cell 15 still unspent

Copied verbatim, in atlas order:

    #E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50,#6B5B49,#34271C,#F0A63C,
    #BAA896,#826145,#7DA18C,#44584C,#63695B

**Nothing this round needed a sixteenth value.** Everything new is carried by a cell that already
exists, reusing approved cells cross-purpose exactly as round 4 did: `#34271C` is hair, beard shadow,
boot body and buckle interior; `#5E4632` is the hair's highlight and the boot's lit plane; `#8B6B50`
and `#6B5B49` carry the belt, pack, flap and straps; `#63695B` carries the bedroll as well as the
trouser lit plane. Boots, belt, pack, bedroll, straps, lantern and pickaxe cost **zero** slots.

The only thing that wanted a new cell is still the warm buckle round 4 identified (`#F7CE94`), and
that is a **hue** change rather than a value step, so the slot stays unspent and the buckle stays
grey iron. That is one decision, not a habit — if Wolf wants it, it is a one-line change.

**One convention did change, and it is worth flagging.** Round 4's value scheme painted the
turned-away (−Y) planes dark as well as the undersides. That reads well from the front and turns the
entire back view into mud — which is precisely the view `back.png` was added to fix. r5's scheme is
**top plane lit, underside dark, every vertical face at base value**: the albedo encodes FORM, not a
light direction. The back render is the evidence.

---

## 6. `check_asset.py` — verbatim

    $ python scripts/bench/check_asset.py src-assets/export/SM_VoxelDwarf_Miner01.glb
    FAIL src-assets\export\SM_VoxelDwarf_Miner01.glb: grid clause: POSITION values must use the 0.0125 m project grid
    EXIT=1

The failure the brief predicted. **Not quantised to satisfy it; the checker was not edited.** Worst
off-lattice offset **6.071 mm** against a 0.010 mm tolerance. The replacement guarantee on this side
is the axis-aligned-normal rule, which the exporter enforces and fails the build on.

The checker stops at the first failure, so the clauses after it were probed separately using the
checker's own module, unedited:

| clause | result | |
|---|---|---|
| grid | **FAIL** | worst off-lattice 6.071 mm vs 0.010 mm |
| quad-soup | **PASS** | 1,116 tris, 2,232 verts, required 2,232 — no vertex shared between quads |
| origin-centring | **PASS** | min Y 0.000000, centre X 0.000000, centre Z 0.000000 |
| naming | **FAIL** | file stem `SM_VoxelDwarf_Miner01` vs mesh `SM_VoxelDwarf_Miner01_r5` — see 4.4 |
| palette / material | PASS | 64x64 atlas, 15 cells, no holes, NEAREST, CLAMP_TO_EDGE, UVs in 0–1 |

Everything before the grid clause passed on the way through: file, one-mesh/material/image,
one-primitive, no-extensions, single-sided, palette/material, mesh-and-node names agreeing, applied
identity transform, finite positions.

Exporter output, verbatim:

    EXPORT D:\Workspace\frostvein\src-assets\export\SM_VoxelDwarf_Miner01.glb
      parts joined      15 -> 1 mesh 'SM_VoxelDwarf_Miner01_r5'
      object / mesh     SM_VoxelDwarf_Miner01_r5 / SM_VoxelDwarf_Miner01_r5
      materials         M_VoxelDwarf_r5
      palette image     T_VoxelDwarf_Palette_r5
      triangles         1116
      size m (X,Y,Z)    1.020 x 0.686 x 1.200
      blender min Z     0.000000   (glTF min Y)
      blender centre XY 0.000000, 0.000000   (glTF centre X, Z)
      non-axis-aligned  0   smooth-shaded 0
      bytes             80060

---

## 7. The three numeric checks, and the zoom strip

Measured on the **flat albedo** pass, classified to the **nearest palette cell**, which is the
measurement definition the brief fixed this round.

| check | target | r4 | r5 |
|---|---|---|---|
| 1 — beard no wider than the head | — | 0.25 m vs 0.336 m, PASS | **0.315 H vs 0.334 H — PASS** |
| 2 — tunic wins ≥ 4 body-band tenths | ≥ 4 | 4, PASS | **2 — MISS, deliberately** |
| 3 — visible skin | ≥ 10 % | 13.9 % re-measured, PASS | **13.0 % with the tools, 16.1 % without — PASS** |

Whole-figure shares, tools included: tunic 17.5 %, hair 17.2 %, beard 15.7 %, skin 8.1 %, trunk
7.7 %, wood 6.9 %, metal 5.1 %, skin-shadow 4.9 %, beard-highlight 4.6 %.

**Check 2 misses, and I did not chase it.** Brown against tunic, per tenth measured up from the feet,
held tools excluded, shares of the band window:

    0-10 58.9/0.3   10-20 45.2/0.2   20-30 0.0/72.3   30-40 29.1/33.1   40-50 35.3/26.9
    50-60 54.9/27.4 60-70 59.2/39.1  70-80 42.0/8.7   80-90 31.4/0.6    90-100 48.8/0.5

The brown in bands 40–70 % is **the beard**, and the beard is where the sheet puts it: 0.315 H wide
against a 0.334 H head — already *narrower* than the sheet's own 0.343 H — with its tip at 0.464 H,
the sheet's own figure. Winning four tenths from here means a beard narrower or shorter than the
sheet draws. This is the same shape of conflict round 4 hit on check 3, with the roles reversed, and
it resolves the same way: **the reference wins and the metric is logged as a miss.** Check 2 is a
chest-coverage measure and this dwarf's chest is covered by his beard, which is what the sheet shows.

**Zoom strip**, round 4's metric kept — `delta = mean |nearest − mip|` on the flat pass — but with
one method change: the figure is now resampled to a given **height** with its aspect preserved, where
the square resize distorts him. At 10 / 30 / 100 px tall: **18.3 / 7.9 / 2.3**. Under round 4's
apparent square resize the same figure gives 17.2 / 7.1 / 2.2 against its 14.1 / 7.3 / 2.1, so the
two sets are close at 30 and 100 px and not strictly comparable at 10. **Nothing turns to speckle at
10 px**: he reads as a dark-topped, green-bodied, brown-bearded block with a tool beside him.

---

## 8. Cost

`_bmad/scripts/session_tokens.py`, print mode, run without `--story` so nothing was written outside
`src-assets/`. Row label `dev-art`. Model from the session banner: **claude-opus-5**.

    Session token cost  (943d20d1-9bc8-4bdc-93c9-c610e6fc3974.jsonl, tool=claude)  (268 turns, claude-opus-5)
      input (fresh)            536
      cache creation       796,667
      cache read        70,812,771
      output               526,370
      total processed   72,136,344
      wall-clock            67 min  (elapsed, includes idle gaps)
      est. cost             $53.55  (benchmark — verify rates in PRICES)

**The transcript-discovery bug round 4 reported is still there.** `session_tokens.py --tool claude`
builds the project directory as `C:\Users\suihk/.claude/projects/D:\Workspace\frostvein` instead of
the escaped slug `D--Workspace-frostvein`, so `--transcript` had to be passed explicitly again. Owed
on the orchestrator's side; not fixed here.

---

## 9. Where the model departs from the sheet, and why

Ordered by how much each is likely to matter to Wolf's eye.

**9.1 The sheet is POSED and this is a neutral base.** Every view has the arms held out and forward —
the back view puts the hands at ±0.386 H from the centre line, half again the shoulder's 0.264 H.
r5 hangs them below the sleeve cap at ±0.264 H. That single choice is most of what makes the side
comparison look emptier than the sheet: with the arm forward, the sheet's beard, belt and tunic all
show in profile; with it hanging, the sleeve covers them for 0.24 H of depth. Nothing is missing —
it is occluded, and a rig and a pose will uncover it.

**9.2 The neck is the one part of this figure that is inferred, not measured.** The sheet does not
show a neck anywhere: hair covers it at the back and sides, beard at the front, in all four views.
"A neck exists" is one of the round's own criteria, so I built one — 0.129 H wide, 0.164 H deep,
skin, its own part — and made two deliberate departures so it reads: the head sits **3 px (2 % of
figure height) higher** than the sheet's reading, and the **hair's side locks stop above the collar**
instead of running to the shoulders. In the assembled figure the beard still covers it from the front
and the sides, exactly as the sheet does; `03-head.png` is where you can see it. If Wolf wants the
neck visible in the finished figure, the lever is the beard's width, and that trades directly against
fidelity — say the word and I will take the beard in.

**9.3 The pickaxe is planted, not held, and it stands forward of him.** At the sheet's own 0.83 H it
is 116 px long — it cannot hang from a hand at 0.33 H without going through the floor, so it stands
butt-on-ground. That puts its head level with his ears, and a broadside head there passes straight
through them: the figure is 0.383 H wide at the ears and the head is 0.343 H across. Round 4 solved
the same collision by turning the head fore-and-aft; that reads as a plank in the side view, which is
the view this round is judged on. r5 keeps the sheet's **broadside** head and buys the clearance by
planting the tool 0.22 H forward of the figure instead. It reads correctly in the front, back and
3/4 views and as a narrow vertical bar in profile — which is exactly what the sheet's own side views
show. It is still the weakest thing in the figure and it is a socketed prop, so it is cheap to move.

**9.4 The hair stops short of the cheek.** The sheet's hair runs forward over the temple; r5's side
locks stop at side-view column 70, leaving 11 px of lit cheek exposed in profile. Without that the
side view is a slab of hair with an ear on it — round 4's silhouette, and Wolf's "flat blocky stick".
This is a departure that fixes the complaint rather than the reference.

**9.5 The pick head is a staircase and the bedroll's ends are steps.** The sheet's pick sweeps in a
smooth arc and its bedroll is a coil. A box model has neither. This is the idiom, not a compromise
of it.

**9.6 The shoulder straps sit outboard of where the sheet draws them.** `back.png` puts them at about
±0.02 to ±0.09 H from the centre line; r5 runs them at ±0.11 to ±0.16 H so they clear the beard and
are visible on the chest at all. They are only possible in any position because the shoulder caps
exist: round 4 had 7 mm of shoulder and terminated its straps at the seam.

**9.7 The buckle is grey iron, not brass.** Unchanged from round 4 and for the same reason — a hue
change, not a value step. Section 5.

---

## 10. What limited the likeness

**10.1 Lighting is still out of scope and still carries half the reference's appeal.** Unchanged from
round 4 and still true: the sheet's artwork is *shaded*, and a flat-albedo asset on grey `#6F7073`
is a structurally different object. Worth one concrete number this round: a cheek that the palette
calls `#E9D2BB` renders on the sheet at about `#AA846F`. That is why colour on the sheet cannot be
sampled directly and why nearest-RGB classification of the sheet mislabels whole regions — I had to
compare chromaticity instead to read it at all. Epic 11's lighting is the point at which the
remaining colour questions become answerable.

**10.2 Flat per-face colour still cannot do gradation.** Form has to be quantised into planes and the
line between "value step" and "corduroy" is narrow. The r5 convention — lit top, dark underside, base
sides — is the version of this that survives being seen from behind.

**10.3 The 16-cell ceiling, but it did not bind this round.** Fifteen used, one spare, nothing
wanting a sixteenth value. What the ceiling still forecloses is hue variation: a brass buckle, a
warm/cool split, eye colour. The clause bug that makes 16 the ceiling is still owed.

**10.4 "No rotated boxes" blocked four specific reference features**, the same wall round 4 hit: the
pick head's arc, the bedroll's coil, the diagonal strap run, and the stance. The reference is not
itself a strict box model, so "match it with axis-aligned boxes" has a built-in ceiling.

**10.5 The orthographic sheet removed the single biggest limit and introduced a smaller one.** Every
depth in this model is measured; the back is drawn rather than invented; front and back agree to
within 1.5 % on every shared dimension. What remains is resolution: the sheet is a 1024x558 JPEG, so
each view is only 110–160 px tall in the original. Proportions as fractions are solid to about half a
percent; single-pixel features are not, and colour is JPEG-smeared at every edge.

**10.6 The authoring interface still cannot model by eye.** Every change is compute-coordinates, run,
render, look, adjust. Writing the part specs as a file on disk instead of a text datablock made the
loop much faster — a convention change rebuilt the whole figure in about forty seconds, and I used
that three times — but it is still a generator in the sense round 4 meant, and the "hand-modelled"
premise is only partly realised. Genuinely hand-tuned geometry needs Wolf nudging vertices and the
seat being told which parts to stop regenerating.

**What did NOT limit anything**: no triangle pressure (1,116 of 4,000), no file-size pressure
(80,060 bytes of 16 MB), and the grid-clause failure remains expected and harmless.

---

## 11. Owed work and notes for whoever picks this up

- **The exported figure's body is 39 mm off the origin in X.** `export_dwarf.py` centres the joined
  mesh on its bounding box, and that box now includes a lantern reaching −0.471 m and a pick head
  reaching +0.549 m. The contract's "centred in X and Z" is satisfied; the *dwarf* is not centred.
  This matters the moment there is a rig, and the cleanest fix is for the exporter to centre on the
  body parts and let the props sit where they sit. Not changed here, because the brief said to reuse
  the exporter as it stands.
- **The naming clause in `check_asset.py` (4.4)** needs a decision before anything ships.
- **`render_dwarf.py` still imports the retired `dwarf_miner`** and would render the OLD dwarf.
  Round 4 reported this and it is still true. `render_r5.py` is the working replacement and reads the
  `.blend`; repointing or retiring the old script sits outside this round.
- **Facing.** He faces +Y in Blender, matching the shipped asset. That maps to −Z in glTF, backwards
  from the glTF convention. Round 4 flagged it; still worth a decision before animation.
- **No rig, no LODs, no decimation** — all explicitly out of scope and all untouched.
- **One tracked file was deleted by accident and I did not put it back**, because putting it back
  means running git and this round does not run git.
  `src-assets/renders/dwarf-vs-contact-sheet.png` — round 4's comparison against the video contact
  sheet — was caught by a `rm src-assets/renders/dwarf-*.png` while I was clearing the stale r4
  renders described in 4.7. It is not in round 5's manifest and round 5 does not replace it.
  **`git checkout -- src-assets/renders/dwarf-vs-contact-sheet.png` restores it**, and that is the
  operator's call, not mine.
- **Nothing else was deleted.** `dwarf.blend`, `dwarf_miner.py`, `voxel_pine.py`, `render_dwarf.py`
  and `trees.blend` are all left exactly as they were. The r4 progress sheets and final renders at
  the delivered paths were overwritten or renamed by r5's, which is what the manifest asks for;
  `git checkout` recovers those too if Wolf wants them side by side.

## 12. Recommended next, in order of expected gain

1. **A rig and a pose.** Round 4 said stance was the largest remaining likeness gap and measurement
   has now made that concrete: the sheet's hands are at ±0.386 H and ours are at ±0.264 H, and that
   one difference accounts for most of what the side-by-side shows. Joint names are already fixed.
2. **A fixing round on the figure**, if Wolf wants one before the rig. My own list, shortest first:
   move or re-mount the pickaxe (9.3); decide whether the neck should be visible at the cost of beard
   width (9.2); decide the handedness question (4.2).
3. **Epic 11's lighting, before any further palette judgement** — unchanged, and the palette revisit
   is already gated on it.
4. **Fix `check_asset.py`'s palette clause and its naming clause**, which is what would let the
   tunic and leathers carry the gradation 10.2 cannot afford and let a revisioned mesh pass.
