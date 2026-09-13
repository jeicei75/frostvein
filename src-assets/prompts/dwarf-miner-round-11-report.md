# Round 11 — report

Built from nothing, in Wolf's running Blender through the MCP addon, to
`src-assets/blender/SM_VoxelDwarf_Miner01.blend`. Every gate is green and
`check_asset.py` exits 0.

| | |
|---|---|
| triangles | **25,248** of 30,000 |
| holes | **0 boundary edges, 0 exposed** |
| inside-out masses | **none** |
| resolution | every mass **16.3–23.5 mm**, figure median **21.5 mm**, face **12.0 mm** |
| UV island coverage | **61.6 %** of the 512² map |
| rig | 19 joints, 0 missing, 0 unexpected, **0 unweighted, 0 soft-weighted** |
| cage | **269 faces** — coarse and editable |
| cost | **$106.72**, 533 turns, 89 min wall-clock |

---

## 0. The live-scene report — and a stop, before any geometry

**§8's proof first.** The MCP connection reached a real GUI instance:
`bpy.app.background False`, Blender 5.2.1 LTS, PID 41680. Not a `--background`
subprocess.

**But the instance had no file open** (`bpy.data.filepath` was `''`) — an unsaved
default startup scene, `Cube` / `Light` / `Camera`. That is exactly what §8 warns a
subprocess looks like, so it was worth separating: a GUI instance that has not loaded
the round's file is not the same failure as a headless one, and the fix is different.

Opening `src-assets/blender/SM_VoxelDwarf_Miner01.blend` then hit **§7's stop
condition**:

```
objects    : r10_beard, r10_belt, r10_body, r10_boots, r10_hair, r10_lantern,
             r10_moustache, r10_pack, r10_pickaxe, r10_ref_front, r10_ref_side,
             r10_straps, r10cam.* (11 cameras), SK_VoxelDwarf_Miner01_r10
collections: Collection, r10_reference, SM_VoxelDwarf_Miner01_r10
meshes     : r10_body (3775 verts, 4018 faces), + 11 others
materials  : M_VoxelDwarf_r10, ...      images: r10 (512x512), front.png, side-left.png
armatures  : SK_VoxelDwarf_Miner01_r10
```

**The file was not empty. The empty save named in §7 was missed, and the whole of
round 10 was still in it.** I stopped and said so rather than building on it, as §7
requires. Wolf's call was to wipe the scene and start; round 10 is recoverable because
`.blend` files are tracked (only `.blend[0-9]` backups are ignored) and the working
tree was clean at `db26fc2 brief 10 result .. broken`. The wipe removed every object,
collection, mesh, armature, camera, material and non-render image, and the file was
saved empty before the first primitive.

Nothing was imported, appended or linked from any other file, and no revision was
recovered from git. **No git command was run at any point.**

---

## 1. What the round produced

A 1.200 m dwarf miner in an A-pose, one continuous body mesh plus nine swappable
masses, one material, one packed 512² map, and a 19-joint rigid-weighted rig.

The figure is **exactly 1.200 m** tall: the hair's crown cap lands on z = 1.200 and the
boot soles on z = 0. Depth is 0.663 m against the measured 0.660 m (0.550 H pack-to-nose).

### Where the measurements came from

The coordinate system was re-derived from the ortho crops before any geometry and
agreed with §2's table exactly:

* figure rows **7–147** in both crops (row 147 is the sheet's own sole rule; rows 0–4
  are its top dimension rules) → 140 px = 1.00 H, 1 px = 8.571 mm;
* `front.png` centre column **77**, confirmed at 77.0–77.5 on every symmetric row;
* crown steps measured **0.164 / 0.229 / 0.286 / 0.336 H** — the table's four numbers;
* head with hair **0.336 H**; stance boot-to-boot **0.400 H**;
* side profile: head back **y = −0.189**, forehead **+0.206**, nose tip **+0.257**,
  beard front **+0.239**, pack back **−0.394** → pack-to-nose **0.550 H**, which is
  §2's whole-figure depth to three decimals.

---

## 2. Resolution — the heart of the round (deliverable 11)

**One SIMPLE subdivision per mass, at the level that lands that mass on the floor given
its own cage.** No loop was hand-cut to reach the floor.

| part | cage faces | faces | tris | size m (X,Y,Z) | **median edge mm** | faces/m² | subdiv |
|---|---|---|---|---|---|---|---|
| `r11_beard` | 26 | 832 | 1,664 | 0.412 × 0.285 × 0.383 | **22.4** | 1,402 | SIMPLE 2 |
| `r11_belt` | 11 | 352 | 704 | 0.484 × 0.388 × 0.093 | **23.5** | 707 | SIMPLE 2 |
| `r11_body` | 126 | 4,016 | 8,032 | 1.114 × 0.437 × 1.060 | **22.5** | 1,564 | SIMPLE 2 |
| `r11_boots` | 18 | 2,304 | 4,608 | 0.502 × 0.249 × 0.197 | **18.7** | 4,330 | SIMPLE 3 |
| `r11_hair` | 17 | 2,176 | 4,352 | 0.403 × 0.395 × 0.352 | **21.4** | 2,907 | SIMPLE 3 |
| `r11_lantern` | 26 | 416 | 832 | 0.150 × 0.150 × 0.350 | **22.5** | 2,177 | SIMPLE 2 |
| `r11_moustache` | 5 | 160 | 320 | 0.224 × 0.053 × 0.065 | **16.3** | 3,206 | SIMPLE 2 |
| `r11_pack` | 14 | 1,792 | 3,584 | 0.310 × 0.236 × 0.485 | **18.7** | 2,775 | SIMPLE 3 |
| `r11_pickaxe` | 16 | 256 | 512 | 0.089 × 0.456 × 1.092 | **21.6** | 571 | SIMPLE 2 |
| `r11_straps` | 10 | 320 | 640 | 0.350 × 0.420 × 0.337 | **18.8** | 1,277 | SIMPLE 2 |
| **total** | **269** | **12,624** | **25,248** | | **median 21.5** | | |

**This is one number, not a spread: 16.3 – 23.5 mm, a 1.4× range.** Round 9's was
6.4 – 38 mm, a 6× range. **The body is not the coarsest mass** (22.5 mm against the
belt's 23.5 mm), which was round 9's specific failure.

**The face region** (body faces with z > 0.90, y > 0.13) measures **12.0 mm** — 1.9×
finer than the body, as §3 wants, but **short of the ~8.6 mm the spec asks for**. Being
straight about it: 8.6 mm would need either a denser face cage than the carving
produced, or a level-3 subdivision on the body — and level 3 takes the body alone from
8,032 to 32,128 triangles, which breaks the 30,000 budget on its own. Given the choice
the spec itself offers ("spend on the body, the head and the hands"), I took a face at
12.0 mm inside budget over 8.6 mm outside it. It is the one resolution number that
misses.

**Departure from the letter of §3, stated plainly.** §3 says *level 1*. Level 1 assumes
round 9's cage density (38 mm). This cage is coarser — 269 faces, 65–172 mm — because
its loops were placed only where the form changes and where features were carved, so
level 1 would have left the body at 45 mm and the hair at 86 mm. The levels above are
what put each mass on the floor with the *same* tool and the *same* discipline the
section is actually arguing for: density from a live modifier, never from hand-cut
loops and never from a script. The cage stayed coarse and editable, which is the other
half of what §3 asks.

---

## 3. The `holes` line at every stage boundary (deliverable 11b)

| boundary | triangles | `holes` line |
|---|---|---|
| stage 1 — block-out | 224 | `0 boundary edges, 0 EXPOSED to the outside` |
| stage 2 — form pass | — | **not taken separately; see below** |
| stage 3 — face carved (mid-stage) | 464 | `0 boundary edges, 0 EXPOSED to the outside` |
| stage 3 — all ten masses built | 884 | `0 boundary edges, 0 EXPOSED to the outside` |
| stage 4/5 — subdivided, textured | 25,248 | `0 boundary edges, 0 EXPOSED to the outside` |
| **before rigging** | 25,248 | `0 boundary edges, 0 EXPOSED to the outside` |
| **after rigging** | 25,248 | `0 boundary edges, 0 EXPOSED to the outside` |

**The figure never had an exposed hole, at any point in the round.**

**The stage-2 miss is mine.** I ran the export at the stage-1 boundary and again early
in stage 3, but rolled from the form pass straight into carving without stopping to
export between them. The two exports bracket that window and both read 0/0, so nothing
is hiding in it — but §5 asks for the line at *every* boundary and one is missing.

**§6's before/after-rigging comparison: identical.** Same triangle count, same 0
boundary edges, same 0 exposed, same `inside-out masses none`. Rigging did not touch
the geometry — which is the check Wolf's round-10 read ("model got messed at least when
it was playing with the rig") asked for.

The one gate that ever failed was **`degenerate faces 2`**, caught by the mid-stage-3
export: tapering the brow ridge at the temple collapsed two side faces to zero area.
Welding the coincident vertices fixed it in one operator. It was found by the export,
which is the whole argument for running it at boundaries.

---

## 4. UV coverage (deliverable 12) and the map

**Islands cover 61.6 % of the 512² map** — above the ~60 % bar. Round 9's map was
256² with 87 % of its texels a single flat fill.

All ten masses were unwrapped and packed **together** in one multi-object edit, so the
islands share one 0–1 space and stay disjoint after the exporter joins the parts. Every
mass's UVs lie inside 0–1. The only overlaps are the deliberate mirrored pairs — the
Mirror modifier copies UVs, so the two halves of every symmetric mass share their
islands, which §10 explicitly allows.

A 4-pass dilation grows each island outward into the gutter (61.6 % → 83.8 % of the map
written) so NEAREST filtering never samples across a seam. The unwritten remainder is
filled with `#5F7A6A`, a palette colour, so the census carries nothing stray.

**Texel density.** The pack is uniform rather than face-weighted, at roughly
**0.27 px/mm** (≈3.7 mm per texel) across the figure — the eye island is 6.3 × 13.6 px
for a 63 × 30 mm eye. The face therefore does *not* have a denser island than the rest,
which is the second half of the 12.0 mm miss above and the honest reason the pupil is a
2 × 4 texel block.

**Painted, not flat-filled:** brow ridge, temple and cheek shading as value steps on
skin; the eye white with its pupil; the lit nose front; the beard's proud centre lock
lighter than the side locks and its tip in shadow; the shoulder yoke darker than the
tunic; the waist panel lighter with a crisp square motif; boot sole, welt and cuff each
a different step; pack flap darker than the body; bedroll in the trouser colour; the
lantern's flame cell a colour behind a set-back pane, never an emitter.

`check_asset.py` reports 34 colours: the approved ten (`#E9D2BB #5E4632 #FFFFFF
#5F7A6A #474B41 #A9B2AC #8B6B50 #6B5B49 #34271C #F0A63C`, all present and exact) plus
24 value steps derived from them. No stray hues.

---

## 5. Per-part census (deliverable 13)

The table in §2 above carries it — name, cage faces, evaluated faces, triangles and
size per part.

**Stage 4 separation.** The body is **one continuous mesh**: torso, arms, hands, legs,
neck, head and every carved head feature. Separated only for what the combination layer
swaps: `r11_hair`, `r11_beard`, `r11_moustache`, `r11_belt`, `r11_pack`, `r11_straps`,
`r11_lantern`, `r11_pickaxe`, `r11_boots`.

**17 feature tags**, all surviving to the export:
`feature_nose`, `feature_brow.R`, `feature_eye.R`, `feature_ear.R`, `feature_jaw`,
`feature_hem`, `feature_shoulder.R`, `feature_buckle`, `feature_beard_lock`,
`feature_boot_cuff`, `feature_pack_flap`, `feature_bedroll`, `feature_crown_steps`,
`feature_lantern_glass`, `feature_pick_head`, `feature_moustache`, `feature_strap_run`.

**Swap boundary loops I intend as the stitch lines**, if the combination layer wants to
freeze them:

* **hair** — the ring at z = 1.100 where the fringe band meets the side lobes, and the
  lobes' bottom rim at z = 0.848;
* **beard** — the top ring at z = 0.940 (tucked under the cheeks) and the z = 0.860
  widest ring;
* **moustache** — its whole rim; it is a single block under the nose;
* **belt** — the z = 0.412 and z = 0.505 rings, which are also body form loops;
* **boots** — the cuff's top ring at z = 0.197, which is the tunic-hem-to-boot handover;
* **pack / straps** — the strap's chest rim at z = 0.520 and the pack's z = 0.500 base;
* **lantern / pickaxe** — the grip rings, at the hand.

---

## 6. The rig

19 joints, names exactly as the contract, `root` carrying no geometry. Every joint sits
on an edge ring the figure actually has:

| joint | on the ring at | joint | on the ring at |
|---|---|---|---|
| `root` | z = 0.000 | `shoulder.L/R` | arm axis t = 0.000 |
| `hips` | z = 0.248 (hem) | `elbow.L/R` | t = 0.205 (elbow ring) |
| `spine` | z = 0.505 (belt top) | `hand.L/R` | t = 0.345 (wrist ring) |
| `chest` | z = 0.628 | `hip.L/R` | z = 0.248 |
| `neck` | z = 0.840 (shoulder line) | `knee.L/R` | z = 0.190 |
| `head` | z = 0.870 (jaw ring) | `foot.L/R` | z = 0.120 (ankle) |
| `beard` | z = 0.875 | | |

**Weights by selection, region by region.** Each region is a named part of the figure —
head above the jaw, the neck column, the arm island, chest / spine / hips bands between
form loops, the leg column below the hem — and **every assignment was printed and read
back before it was trusted**. Props are weighted to what carries them: pickaxe →
`hand.R`, lantern → `hand.L`, pack and straps → `chest`, boots → `foot`, belt → `hips`,
hair and moustache → `head`, beard → `beard`.

**The round-10 failure mode showed up here and was caught by that verification.** My
first arm rule was "x is large", and it swept the leg's outer vertices (x = 0.215) and
the belt's (x = 0.235) into the arm bones — the same shape of defect as round 10
weighting the skirt to the arm. The census printed `hip.R: 2` where the ring has 4
vertices, which is what exposed it. The rule became *inside the arm's own cylinder*
(perpendicular distance < 0.13 m from the arm axis, t ≥ −0.02), and the fixed census
was re-read vertex by vertex: every arm vertex now lands cleanly on a ring at t = 0.000
/ 0.075 / 0.205 / 0.345 / 0.435, and every leg ring has its full 4.

The thresholds sit **between** rings, so each joint breaks on a real edge ring rather
than splitting one.

**912 interpolated vertices were snapped** by the exporter to their dominant joint —
mechanical, expected from a subdivision, and exactly what §6 describes. **0 vertices
were soft-weighted**, so nothing needed hiding.

The deflection renders show no tearing, no hole and no geometry following the wrong
bone. The GLB ships in the bind pose with no animation.

**No face was culled, anywhere.** The legs' bottom caps sit sealed inside the boots and
stay there.

---

## 7. The build, and what it cost in mistakes

The cage was built one Blender operator per tool call, on a selection — `primitive_cube_add`,
`extrude_region_move`, `inset`, `subdivide`, `bevel`, `resize`, `translate`, `rotate`,
`remove_doubles`, `delete`, `edge_face_add`. **No setters, no per-op helpers, no
bisect-by-plane, nothing in `driver_namespace`, no script that moves vertices in bulk by
rule.** The three declarative passes that are not shaping — adding the ten subdivision
modifiers, assigning vertex groups, and rasterising the texture — are grouped, and none
of them touches a vertex coordinate.

Five mistakes, all caught and repaired in-session rather than shipped:

1. **Selecting faces by area** caught two shoulder strips along with the neck face and
   extruded all three. Repaired by translating the region back and welding the
   duplicated vertices. Every later selection is by explicit bounds with an assert on
   the count.
2. **`bm.select_flush(True)` on an edge selection over-selects** — it re-selects every
   edge whose endpoints are selected, so a 4-edge ring became the whole box and
   `subdivide` left n-gons. The cage was rebuilt (it was nine operations old) and every
   later edge selection uses `bm.select_flush_mode()`, verified at 4 edges before
   operating.
3. **A wedge sleeve.** The arm was first extruded from the torso's full-depth side face,
   giving a sleeve 0.342 m deep at the shoulder. Deleted, the side face re-closed, and
   the arm rebuilt off a properly inset 0.110 × 0.170 attach cell.
4. **The eye funnel, then the ears.** Resizing an inset's inner face hard turns the ring
   into a wide sloping funnel — the first eye read as goggles. Flattened and re-cut as a
   tight socket. Then the *paint* rule for the eye matched the **ear's** front face as
   well (both sit under y = 0.200 on the same z band), so the ears were painted white.
   Fixed by constraining the eye in x too.
5. **A stale packed image.** `Image.pack()` on an already-packed image does not replace
   the packed bytes, so three corrected textures rendered identically while the PNG on
   disk was right. The image datablock is now rebuilt from the file and re-packed.

---

## 8. What is weaker than it should be

* **The face is 12.0 mm, not 8.6 mm**, and its island is not denser than the rest of the
  map. Both are stated above with the budget arithmetic behind the choice.
* **The stage-2 boundary export was not taken.**
* **The visible neck column in profile is ~30 mm, not the measured 0.121 H (145 mm).**
  The head's jaw ring sits at z = 0.870 and the shoulder line at 0.840, so only that
  band is bare between the hair and the beard. The README's reading is right and the
  geometry under-delivers it.
* **The pickaxe is planted vertically** — butt on the floor beside the right boot, head
  up, gripped by the neutral right hand. It is in the hand as §4 requires and it keeps
  the butt on z = 0 so the seating gate reads the boots, but at 0.91 H of shaft it
  crosses the figure in the side view and reads more "miner at rest" than the sheet's
  diagonal carry. The two-handed carry is in `pose-carry-*.png`, where §4 says it belongs.
* **Bindings at the pickaxe head and hardware on the straps are paint, not geometry.**
* **The waist panel's motif is paint**; the panel itself is a value step, not an inset.

---

## 9. Deliverables

| # | what | where |
|---|---|---|
| 0 | live-scene report | §0 above |
| 1 | the source, saved after every call, image packed | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture | `src-assets/blender/textures/T_VoxelDwarf_r11.png` |
| 3 | the render script | `src-assets/blender/render_r11.py` |
| 4 | a render set per stage | `src-assets/renders/r11/stage-*.png` |
| 5 | progress renders | `src-assets/renders/r11/progress/NN-*.png` |
| 6 | five final views, flat and key-lit | `src-assets/renders/r11/final-*.png` |
| 7 | beside `f088` and `f104` | `src-assets/renders/r11/vs-frames-*.png` |
| 8 | side profile beside the sheet | `src-assets/renders/r11/vs-ortho-side-left.png` |
| 9 | readability at 100 px and 60 px | `src-assets/renders/r11/readability.png` |
| 10 | posed carry, one deflection per joint group | `src-assets/renders/r11/pose-*.png`, `joint-*.png` |
| 11 | resolution table | §2 |
| 11b | `holes` per stage boundary | §3 |
| 12 | UV coverage | §4 |
| 13 | per-part census | §2 / §5 |
| 14 | this report | `src-assets/prompts/dwarf-miner-round-11-report.md` |
| 15 | exporter and checker output | §10 |
| 16 | cost | §11 |

Nothing was written outside `src-assets/`.

---

## 10. Exporter and checker output, verbatim (deliverable 15)

```
EXPORT D:\Workspace\frostvein\src-assets\export\SM_VoxelDwarf_Miner01.glb
  parts joined      10 -> 1 mesh 'SM_VoxelDwarf_Miner01_r11'
  object / mesh     SM_VoxelDwarf_Miner01_r11 / SM_VoxelDwarf_Miner01_r11
  materials         M_VoxelDwarf_r11
  texture image     r11   in the GLB: T_VoxelDwarf_r11
  triangles         25248  of 30000 budget
  size m (X,Y,Z)    1.140 x 0.663 x 1.200
  blender min Z     0.000000   (glTF min Y)
  blender centre XY 0.000000, 0.000000   (glTF centre X, Z)
  topology          n-gons 0   non-manifold edges 0   loose verts 0   loose edges 0   degenerate faces 0   flipped winding 0   missing UV layer 0
  holes             0 boundary edges, 0 EXPOSED to the outside
  inside-out masses none   (0 open shells not judged: -)
  feature tags      17 vertex groups, 0 face attributes   feature_beard_lock, feature_bedroll, feature_boot_cuff, feature_brow.R, feature_buckle, feature_crown_steps, feature_ear.R, feature_eye.R, feature_hem, feature_jaw, feature_lantern_glass, feature_moustache, feature_nose, feature_pack_flap, feature_pick_head, feature_shoulder.R, feature_strap_run
  live modifiers    r11_beard:MIRROR, r11_beard:SUBSURF, r11_belt:MIRROR, r11_belt:SUBSURF, r11_body:MIRROR, r11_body:SUBSURF, r11_boots:MIRROR, r11_boots:SUBSURF, r11_hair:MIRROR, r11_hair:SUBSURF, r11_lantern:SUBSURF, r11_moustache:MIRROR, r11_moustache:SUBSURF, r11_pack:MIRROR, r11_pack:SUBSURF, r11_pickaxe:SUBSURF, r11_straps:MIRROR, r11_straps:SUBSURF
  rig               joints 19   missing joints 0   unexpected joints 0   unweighted verts 0   soft-weighted verts 0   joint names in the GLB: beard, chest, elbow.L, elbow.R, foot.L, foot.R, hand.L, hand.R, head, hip.L, hip.R, hips, knee.L, knee.R, neck, root, shoulder.L, shoulder.R, spine
  weights snapped   912 interpolated vertices quantised to their dominant joint
  GLB min/max       [-0.5698201060295105, 0, -0.33149969577789307] / [0.5698201060295105, 1.2000001668930054, 0.33149969577789307]   (glTF axes: X, Y up, Z)
  bytes             1144480
```

```
$ python3 scripts/bench/check_asset.py src-assets/export/SM_VoxelDwarf_Miner01.glb
FIGURES src-assets\export\SM_VoxelDwarf_Miner01.glb size_m=1.1x1.2x0.7 min_y_m=0.000000 centre_x_m=0.000000 centre_z_m=0.000000 palette=#5F7A6A,#8B6B50,#6B5B49,#34271C,#E9D2BB,#4F3B2A,#5B4D3E,#5E4632,#D2BDA8,#A9B2AC,#5C4E3F,#3E4239,#474B41,#848B86,#6F533B,#4B3828,#C4B09D,#7A5E46,#D6C1AC,#463526,#291E16,#2C2F28,#443224,#C8B5A1,#769783,#546B5D,#70604D,#73553D,#F0A63C,#DBC5B0,#F7DFC6,#6C513A,#FFFFFF,#4A5F53 tris=25248 verts=18685 mesh=SM_VoxelDwarf_Miner01_r11 profile=painted-map
exit code: 0
```

---

## 11. Cost (deliverable 16)

```
$ python _bmad/scripts/session_tokens.py --tool claude \
      --transcript ~/.claude/projects/D--Workspace-frostvein/10916cd1-...jsonl \
      --phase dev-art
Session token cost  (10916cd1-3c6c-4dd7-9d97-cb471155b14c.jsonl, tool=claude)  (533 turns, claude-opus-5)
  input (fresh)          1,066
  cache creation     1,061,064
  cache read       153,019,025
  output               942,887
  total processed  155,024,042
  wall-clock            89 min  (elapsed, includes idle gaps)
  est. cost            $106.72  (benchmark — verify rates in PRICES)
```

Row `dev-art`. The script's default transcript path is broken on Windows, so the path
was passed explicitly. No ledger row was written — the number is reported here only.

Against round 10's $131.67 / 585 turns: **$106.72 / 533 turns**, and the turns went into
form and gates rather than into tooling and repair. There is no toolkit to show for this
round, by design — the density is one modifier per mass and the gates are the exporter's.
