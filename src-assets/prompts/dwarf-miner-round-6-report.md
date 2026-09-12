# Round 6 report — built in Wolf's Blender, rotated boxes, and the resolution number

**Result: a complete figure at 2,900 triangles, 17 parts, built part-by-part inside Wolf's
running Blender.** Every mechanical clause in §9 of the brief passes. The three things this
round was asked to change are all in: rotated boxes are used (ten places, each listed in §4),
the silhouette step density is roughly **2.7x round 5's** and now sits at or above the
reference on three of four views, and the whole figure appeared in the viewport one part at a
time over 37 saved steps.

**Four defects were found in the input documents and one in my own work; all five are in §7.**
The largest is that **check 4's published density numbers cannot be reproduced from round 5's
own delivered renders**, so the "10.3 / 11.2" targets in the brief are not a scale this round
could aim at. §3 explains what replaced them and why the replacement is stricter, not looser.

---

## 0. The live-scene report — proof this ran in Wolf's Blender

Queried before any geometry was created, through the MCP addon:

```
BLENDER 5.2.1 LTS
FILEPATH ''
IS_DIRTY False
BACKGROUND False                      <- a GUI instance, not `blender --background`
WINDOWS 1
  screen: Layout areas: ['PROPERTIES', 'OUTLINER', 'DOPESHEET_EDITOR', 'VIEW_3D']
SCENES ['Scene']
COLLECTIONS ['Collection']
OBJECTS [('Camera', 'CAMERA'), ('Cube', 'MESH'), ('Light', 'LIGHT')]
MATERIALS ['Dots Stroke', 'Material']
CWD C:\Program Files\Blender Foundation\Blender 5.2
RENDER_ENGINE BLENDER_EEVEE
```

**What was already in it: the default startup scene** — Cube, Light, Camera, unsaved, no
filepath. So Wolf had launched Blender fresh and had not opened the dwarf. `BACKGROUND False`
with one window carrying a `VIEW_3D` area is the part that matters: a `--background`
subprocess has no window manager and no screen, so this could not have been one.

`src-assets/blender/SM_VoxelDwarf_Miner01.blend` was then opened **in that instance** and
everything after happened there. The r5 collection was left in the file as reference and
excluded from the view layer so the viewport started clear.

**Wolf interrupted after the setup calls with *"i can't see in blender anything atm?"* — and he
was right.** At that point the r6 collection existed but was empty and r5 was parked, so the
viewport was blank. The whole figure was blocked out in the next call, 156 triangles, and was
on screen from then on. That exchange is the round's own evidence that the watchability clause
is load-bearing: three minutes of invisible setup was already too much.

Only two things ran headless, both sanctioned: the final `export_dwarf.py`, and `render_r6.py`
for the delivered stills — rendering in the GUI instance would have stolen Wolf's window on
every pass.

## 1. What was built, and the triangle count

| part | tris | part | tris |
|---|---|---|---|
| `r6_head` (stepped crown, 11 slabs) | 132 | `r6_belt` | 96 |
| `r6_face` (brow, eyes, nose, ears) | 252 | `r6_pack` | 168 |
| `r6_neck` | 48 | `r6_straps` | 84 |
| `r6_beard` | 384 | `r6_boot.L` / `.R` | 144 / 144 |
| `r6_hair` | 384 | `r6_leg.L` / `.R` | 48 / 48 |
| `r6_torso` | 144 | `r6_lantern` | 152 |
| `r6_arm.L` / `.R` | 204 / 216 | `r6_pickaxe` | 252 |

**Total 2,900 triangles over 17 parts**, against round 5's 1,116 and an eventual LOD0 ceiling
of 4,000. Nothing was optimised, per the standing ruling. The spend went where the brief said
to put it: the crown is 11 slabs, the beard 10 courses plus 16 lock boxes, each arm 16 courses,
each boot 12 layers.

Sockets are declared and wired: `socket.hair`, `socket.beard`, `socket.pack`, `socket.hand.L`,
`socket.hand.R`. The beard and hair lift off and leave a complete head underneath.

## 2. The export — the flat-and-planar gate

```
EXPORT D:\Workspace\frostvein\src-assets\export\SM_VoxelDwarf_Miner01.glb
  parts joined      17 -> 1 mesh 'SM_VoxelDwarf_Miner01_r6'
  object / mesh     SM_VoxelDwarf_Miner01_r6 / SM_VoxelDwarf_Miner01_r6
  materials         M_VoxelDwarf_r6
  palette image     T_VoxelDwarf_Palette_r6
  triangles         2900
  size m (X,Y,Z)    0.871 x 0.708 x 1.324
  blender min Z     0.000000   (glTF min Y)
  blender centre XY 0.000000, 0.000000   (glTF centre X, Z)
  smooth-shaded     0   non-planar 0   custom normals 0   modifiers 0
  bytes             204948
```

**`smooth-shaded 0  non-planar 0  custom normals 0  modifiers 0`** — the line the brief asks
for, clean. Rotation does not threaten it and this is why: a rigid rotation is an isometry, so
a planar quad stays planar to float precision. The Z extent is 1.324 m rather than 1.200
because the pickaxe rides above the crown; **the dwarf himself is 1.200 m**.

## 3. Check 4 — resolution, the round's headline

### The instrument had to be rebuilt, and this is the round's most important finding

**Check 4's published numbers cannot be reproduced from round 5's own delivered renders.** Its
method is "count rows where the silhouette edge moves, divide by the figure's height in that
image's pixels". Re-running exactly that on `src-assets/renders/dwarf-flat-front.png` gives
**L14 / R75** at a tolerant background threshold — against the **L13 / R27** check 4 published.
The R edge is off by a factor of three.

The cause is antialiasing: the flat pass is antialiased, so a nominally straight edge wobbles
by a pixel and every wobble counts as a step. The result depends entirely on the threshold
chosen, and no threshold is recorded. Two further problems compound it:

- **the metric shrinks as the image grows.** Steps are counted per image row, so the reference
  crop measured at 700 rows and round 5's render at 635 are only accidentally comparable, and
  a 1400-row render of the same model would score half as much;
- the reference's steps were counted on a **5x nearest-neighbour blow-up**, where each source
  pixel is five identical rows, so its step count is at source granularity while its divisor
  is five times larger.

**What replaced it** (`src-assets/blender/measure_r6.py`): every silhouette is resampled to
**exactly 140 rows** — the sheet's own source-pixel height, where one row is 8.571 mm — with
area averaging and a 0.5 coverage threshold, *before* any step is counted. That is scale-free,
immune to antialiasing, and puts the reference and our render on one axis. It is **stricter**
than check 4, not looser: a feature under about three source pixels does not survive the
resample, so it cannot be counted.

Read on this instrument:

| view | reference | round 5 | **round 6** |
|---|---|---|---|
| front | 60.0 steps / 100 rows, balance 0.95 | 20.0, balance 0.65 | **52.9, balance 0.76** |
| side-left | 47.1, balance 0.94 | 17.9, balance 0.47 | **53.6, balance 0.97** |
| side-right | 48.6, balance 0.84 | 15.7, balance 0.57 | **51.4, balance 0.95** |
| back | 59.7, balance 0.77 | — | **62.1, balance 0.93** |

**Round 5 carried a third of the reference's detail, not the 60 % check 4 reported** — the
published comparison flattered it, because its antialiasing noise inflated the model's count
while the reference's five-fold divisor deflated the target's.

**Round 6 exceeds the reference on the side and back views and reaches 88 % of it on the
front.** Against the brief's stated gates — ≥ 10 steps per 100 rows and neither edge below 70 %
of the other — every view passes, the weakest by 5.3x.

The asymmetry finding is fixed outright: round 5's worst edge ratio was 0.47 with a body that
was *symmetric*, because a held prop owned one edge. Round 6's worst is 0.76.

### What the front view cost, and it is worth stating plainly

The front is the one view below the reference, and the reason is the pickaxe. The measured
trade is in §5 — a tool anywhere outboard of the body becomes the whole right edge and hides
every step the body has. 52.9 is the best of the five placements measured.

## 4. Rotated boxes — every one, and why

Axis-aligned is still the default. Counted in world space off the finished mesh: **1,450 faces,
of which 1,252 — 86.3 % — are axis-aligned**, or about **208 of 241 boxes**. The 33 that are not
belong to ten features, and every one of them is a thing the old clause blocked:

(The pickaxe accounts for 106 of the 198 rotated faces on its own, because the whole *object*
carries a 12° tilt: its shaft and bindings are axis-aligned boxes in their own object space and
only the head's arc is rotated within it. Counted per-box in the tool's own frame, 11 of its 20
boxes are axis-aligned.)

| # | part | rotation | why |
|---|---|---|---|
| 1 | brow ridges (2) | ±13° about Y | the sheet draws them **angled down toward the centre**; square, they are a shelf — which is what round 5 had |
| 2 | ears (4) | ∓12° about Z | so they lie against the skull instead of standing square off it |
| 3 | sleeve caps (2) | ∓11° about Y | the shoulder **slopes away from the neck**; square it reads as a crate corner |
| 4 | bedroll (5) | +17° about Y | **the slung bedroll** — named in the brief as blocked. Square on top of the pack it is one more box on a stack; slung, it reads as something tied on |
| 5 | chest + back straps (6) | ∓19° about Y | **the diagonal straps** — named as blocked. A strap from sleeve cap to belt is a diagonal in XZ; axis-aligned it can only be a staircase, and rounds 4 and 5 both left it off |
| 6 | lantern bail (1) | +52° about Y | carries the load from the lantern back in to the hand |
| 7 | right-hand fingers (2) | −14° about X | curl **forward** round the shaft rather than sticking out square |
| 8 | left-hand fingers (1) | +16° about Y | close on the bail |
| 9 | pick head, spike (6) | +4 / 12 / 21 / 30 / 39° about Y | **the pick head's arc** — named as blocked. A staircase of rotated boxes |
| 10 | pick head, adze (5) | −4 / −11 / −18° about Y | the arc's short side |
| 11 | the whole pickaxe | +12° about X | see §5 — it is what keeps the shaft off **both** silhouettes |

Three of the four features the brief listed as blocked outright are now present (bedroll,
straps, pick arc). The fourth — stepped forms that only work on a diagonal — is the pick head's
staircase, which is the same mechanism.

## 5. The pickaxe, measured five ways

The brief's two hard constraints on this tool **conflict with its stance ruling**, and the
conflict is real rather than a matter of taste: a tool **1.04 H long**, **held**, by an arm that
is **not posed** and therefore hangs with the hand at 0.33 H. There is not enough room under the
hand for the tool, so it must lean, and a leaning shaft is a sloping silhouette edge — and a
sloping edge steps on almost every row, which check 4 counts.

Five placements were built and measured, not guessed:

| placement | front | side-left | side-right | back |
|---|---|---|---|---|
| x+29 y+22, tilt 12° about X | **57.9 / 0.84** | **55.0 / 0.93** | 53.6 / 0.88 | 65.7 / 1.00 |
| x+29 y+14, tilt 34/20° | 44.3 / 0.63 *(shaft through the skirt)* | — | — | — |
| x+41 y+2, tilt 12° | 44.3 / 0.63 | 50.7 / 0.92 | 52.1 / 0.87 | 43.6 / 0.65 |
| x+29 y+22, **vertical** | 55.7 / 0.73 | **34.3 / 0.45** | 36.4 / 0.38 | 62.9 / 0.76 |
| **shipped** (x+29 y+22, 12° about X, asymmetric head) | 52.9 / 0.76 | 53.6 / 0.97 | 51.4 / 0.95 | 62.1 / 0.93 |

The two instructive failures: **vertical** collapses the profile from 55 to 34 because the
shaft becomes the entire front edge and hides the nose, beard, belt and skirt; **outboard**
collapses the front from 58 to 44 for the same reason on the other axis. Twelve degrees is
what keeps the shaft crossing the body's own envelope rather than bounding it.

Final geometry: **shaft 146 px = 1.043 H**, gripped 46 px above the butt, **butt clears the
sole by 10.1 rows (86 mm)** — held, not planted. Round 5 planted it because it followed the
label's 0.83 H, at which length it cannot hang from a hand at all.

**The head span is a departure and here is the full evidence**, because the inputs disagree
three ways:

- the gear breakdown's **label** says head span ÷ length = 0.89;
- **the brief** says the art measures 0.54 and the label is wrong;
- **`gear.png`, measured here** at source resolution, gives the tool 64 rows tall with a head
  spanning 56 px — **ratio 0.875, which agrees with the LABEL, not with 0.54**;
- **`front.png`**, where the tool is actually held, draws the head about 40 px across against a
  tool reading ~104 px — **ratio 0.38**, and foreshortening can only have made that smaller
  than the truth.

Following the reference **drawing** over both numbers, as the brief directs: **58 px of span
against 146 px of length, ratio 0.40**, which lands on the front view's own read. At 0.54 the
head is a crescent wider than his shoulders and owns every view; it was built that way first
and is in `progress/13-pickaxe-held.png` if that call wants revisiting.

The head is also **asymmetric** — a spike one side, a short flat adze the other. Two mirrored
arcs make a crescent, which is a mattock, and at 60 px that is exactly what it read as.

## 6. Checks 5, 3 and 2

**Check 5 — the bare neck in profile: PASS, 0.211 H against a 0.08 H gate**, in both side views
(139 rows of 659), measured by nearest palette cell exactly as specified.

**But the brief's neck geometry cannot be built as written, and this is the round's second
document defect** — see §7.2. What is built: the head's side mass is taken up to row 41 and the
hair is sent **down the back** rather than round the sides, which leaves a genuinely bare band
between the jaw and the shoulder line, joined to the visible ear and nose above it.

**Check 3 — visible skin ≥ 10 %: PASS on all three views measured** — front 10.5 %, side-left
15.0 %, three-quarter 11.1 %, by nearest palette cell. It is not comfortable on the front: the
pickaxe is brown and steel and dilutes the share, and this sat at 9.5 % until the forehead and
cheeks were widened to the sheet's own full head width.

**Check 2 — the tunic owns four of ten height bands: FAIL, 0 of 10.** Not in §9's judged list,
reported because it is in the sheet. The tunic's own bands are 70-80 % (25.8 %) and 80-90 %
(17.3 %), and it loses every band to the brown family. The cause is not a wide beard — the
beard is at the head's width, which is the check that governs it — but that **the chest is
covered by beard, pack straps and the pickaxe shaft simultaneously**, and the figure carries
brown boots, brown hair, a brown beard, a brown pack and a brown tool. Worth a ruling before
round 7: either this check is about beard width and is already satisfied by check 1, or the
figure genuinely needs more tunic showing, which is a gear-placement change.

**Zoom strip / speckle:** delta 22.1 at 10 px, 10.9 at 30 px, 3.8 at 100 px. Nothing turns to
speckle — the 10 px figure is still a dark head over a green body with an orange lantern.

## 7. Conflicts and defects found

**7.1 Check 4's published numbers are not reproducible from round 5's own renders.** L14/R75
measured against L13/R27 published. §3 has the mechanism and the replacement. **This is the
biggest one: the brief's headline target of "≥ 10 steps per 100 rows" is, on any instrument
that survives scrutiny, about a fifth of what the reference actually scores.** Round 5's figure
would have passed a literal 10.0 on the reference's own normalisation while carrying a third of
its detail.

**7.2 Check 5's bare neck cannot exist in an orthographic side view of the sheet's own
figure.** The sheet puts the head+hair mass ending at **0.707 H** and the shoulder line at
**0.700 H** — *one row apart*. A side view takes the largest |X| at each (Y,Z), so a neck
narrower than the shoulders is hidden by the sleeve cap from the shoulder row down; the only
band where a neck can be seen is between the bottom of the head mass and the shoulder line, and
those two numbers leave one row of it. Followed literally, the check is unbuildable.

**7.3 Check 5's neck rows actually locate the EAR.** Check 5 and `dwarf-ortho/README.md` both
give the bare skin column as **z/H 0.793 → 0.679, 17 source px tall, up to 9 px wide**.
Measuring `side-left.png` and `side-right.png` for bright warm pixels finds exactly one such
column — **17 px tall, 5 px wide, at rows 29-45** — and the model sheet's own table puts **ear
top at 0.850 (row 28) and ear bottom at 0.729 (row 45)**. The two agree to the row. It is the
ear. The actual bare skin below the jaw is a wider, shorter band at rows 44-51 which does land
"on the collar just below the shoulder line (0.700)" exactly as check 5 says. So check 5 has
the *landing* right and the *column* wrong by about seven rows. Both are built: an ear at rows
28-45 at ear-to-ear width, and a neck below it.

**7.4 The reference sheet is shaded, so our palette cannot be used to classify it.** The brief
warns that a classifier mislabels the sheet, and it is worse than "mislabels": the sheet's skin
reads **(204,144,120)** at the nose in front and **(127-146, 99-108, 84-99)** in profile,
against `#E9D2BB` = (233,210,187). Nearest-palette-cell classification labels the entire head
as hair and wood. Everything here was read as brightness and warmth maps instead. This is also
independent confirmation that "the reference looks warmer than our palette" is a property of
the *sheet*, not only of the one dim video frame — worth carrying into the Epic 11 palette
revisit rather than treating the question as closed by the video alone.

**7.5 MY OWN, and it is the one that cost this session most: a depth-frame bug.** `K.Y(col)`
converts a side-left **column** to metres by subtracting the depth centre 58. I authored every
body part with its depth as an **offset** from that centre — the torso as `-20..+20`, the beard
front as `+28` — and fed those through `Y()` as though they were columns. The whole figure was
therefore built **58 px (0.497 m) behind the origin**. It was self-consistent, so every render
looked correct and every relative depth was right; it only surfaced when the pickaxe, authored
in true columns, was placed beside it and appeared to float half a metre in front of him.

The first fix shifted the existing meshes and left `Y()` alone, which meant every part rebuilt
*after* it landed in the old frame again — that is what dropped front skin to 4.8 %. The real
fix was to change `B()` so both x and y are offsets, one convention, and rebuild the four parts
that were on the wrong side of it. Every part's world depth is now checked against the sheet's
own table: head −22..+26 (sheet −22..+25), nose +31 (+31), beard front +29.5 (+28), torso
−22..+21.5 (−20..+20), pack −48.5..−19.5 (−46..−20), boot −13.5..+19 (−11..+18).

**7.6 A socket's parent inverse was stale**, hanging the beard 5 px behind its own mesh, which
is why the beard barely read in profile. Sockets are mount points and must cancel exactly;
recomputed for all five.

**7.7 `session_tokens.py` builds its project slug the Linux way** and cannot find a transcript
on this Windows clone: it produced `C:\Users\suihk/.claude/projects/D:\Workspace\frostvein`
instead of `...\projects\D--Workspace-frostvein`. Worked around with `--transcript`; not fixed,
not mine. Same family as the known colon-path defect on this clone.

## 8. The checker, verbatim

```
FAIL src-assets\export\SM_VoxelDwarf_Miner01.glb: grid clause: POSITION values must use the 0.0125 m project grid
```

Exit code 1. **Expected, per the brief — a box model fails the grid clause by design.** The
model was not quantised and the checker was not edited.

One note the brief did not anticipate: **this is the checker's entire output.** The grid clause
raises inside `figures()` *before* the figures line is printed, so the palette clause and the
naming clause — the two the brief says are known-broken — never report at all. There is nothing
further to quote from them.

## 9. §9's judged list, item by item

| clause | result |
|---|---|
| ≥ 10 silhouette steps / 100 rows, front and side | **PASS** — 52.9 front, 53.6 / 51.4 side |
| neither edge below 70 % of the other | **PASS** — worst 0.76 (front) |
| bare neck ≥ 0.08 H in both side views | **PASS** — 0.211 H |
| crown stepped, not a single box | **PASS** — 11 slabs |
| brow, nose and ear break the profile | **PASS** — all three modelled and visible in profile |
| shoulder caps exist, torso wider at shoulders than waist | **PASS** — 0.528 H over the caps vs 0.439 H at the waist |
| side profile depths match `side-left.png` proportionally | **PASS** — table in §7.5 |
| beard no wider than the head, as long as the sheet draws it | **PASS** — ±24 px vs a ±23.5 head; tip at 0.464 H |
| visible skin ≥ 10 % by nearest palette cell | **PASS** — 10.5 / 15.0 / 11.1 % |
| pickaxe at drawn length (~1.04 H) and held | **PASS** — 1.043 H, butt 86 mm clear |
| export flat-and-planar 0/0/0/0 | **PASS** |
| nothing turns to speckle at 10 px | **PASS** — delta 22.1 |
| every datablock carries `r6` | **PASS** — collection, object, mesh, material, palette image |
| built in Wolf's open Blender, followable | **PASS** — §0, 37 saved steps, 37 progress renders |
| nothing outside `src-assets/`, no git command run | **PASS** — 7 changed paths, all under `src-assets/` |

## 10. What I would change next, in order

1. **Stance, via the rig** — this is the biggest remaining likeness gap and it is correctly
   not mine. The side-by-side makes it obvious: the reference holds the pick across the body in
   both hands, and every difference in the upper body follows from that one thing.
2. **Rule on check 2** (§6). The tunic owning zero bands is either a non-issue that check 1
   already covers, or a real note that the gear is burying the garment.
3. **The beard's mass.** It is the right width and the right length and it is still a stepped
   block where the reference's is a rounded bush. That wants more, smaller boxes on its own
   silhouette rather than any change of dimension — the same medicine as check 4, applied one
   level down.
4. **Fix check 4 in the model sheet** to the 140-row normalisation, or the next round inherits
   a target five times too small.

## 11. Cost

```
Session token cost  (1b7649a3-3453-455d-b35f-b404dad08f1e.jsonl, tool=claude)  (266 turns, claude-opus-5)
  input (fresh)            532
  cache creation       719,430
  cache read        58,314,683
  output               558,171
  total processed   59,592,816
  wall-clock            52 min  (elapsed, includes idle gaps)
  est. cost             $47.61  (benchmark - verify rates in PRICES)
```

Model from the banner: **Opus 5 (1M context)**, `claude-opus-5[1m]`. Row: `dev-art`. Printed
with `--transcript` because of §7.7; no ledger row was written, and no cursor advanced.

## 12. Deliverables

| # | what | path |
|---|---|---|
| 0 | live-scene report | §0 above |
| 1 | the source, saved after every part | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the render script | `src-assets/blender/render_r6.py` |
| 3 | the exported GLB | `src-assets/export/SM_VoxelDwarf_Miner01.glb` |
| 4 | progress renders, one per part | `src-assets/renders/r6/progress/01..37-*.png` |
| 5 | the five final views, flat and lit | `src-assets/renders/r6/dwarf-{flat,lit}-*.png` |
| 6 | zoom strip with the speckle metric | `src-assets/renders/r6/zoom-strip.png` |
| 7 | side-by-side against side and front | `src-assets/renders/r6/vs-ortho-side.png`, `vs-ortho-front.png` |
| 8 | silhouette step density, both views, both edges | §3 above |
| 9 | this report | `src-assets/prompts/dwarf-miner-round-6-report.md` |
| 10 | the checker's output, verbatim | §8 above |
| 11 | cost | §11 above |

Also written, and inside `src-assets/` as required: `src-assets/blender/r6_kit.py` (the
authoring primitives — measurement frame, palette ids, box and rotated-box maths, progress
renderer; **no part specs**, deliberately, so the session could not become a generator),
`src-assets/blender/measure_r6.py` (checks 2-5 as one reproducible instrument) and
`src-assets/blender/sheet_r6.py` (zoom strip and side-by-sides).

No git command was run that changes anything; `git status` was read once, to verify the claim
in §9 that nothing was written outside `src-assets/`.
