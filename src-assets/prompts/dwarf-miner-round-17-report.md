# Round 17 report. Fork r14, keep what r16 proved, and shut two seams the metric could not see.

Round 17 has no brief. It is a **fork**, opened after Wolf's verdict on round 16, and it ran
in three phases that are worth keeping separate because two of them are already committed
and the third is not:

| phase | what | state |
|---|---|---|
| A — the fork | `dwarf_r17.py` = `dwarf_r14.py` + four generator changes | committed `f270f66` |
| B — the colour defect | #94: drop `srgb_to_linear`, rename to r17, promote | committed `96fc338` |
| C — the seams | range-of-motion pass, two interior boxes | **in the working tree, uncommitted** |

This report covers all three, because phases A and B were never written up.

---

## 0. What round 16's brief could not give us

Round 16 was instructed to build from nothing and not to carry geometry across. It obeyed,
it passed its own gates, and **Wolf's verdict was that it had lost fourteen rounds of
accumulated tweaking.** He was right, and that is the finding that produced round 17.

Two things are worth naming precisely, because they are the reason this round exists:

- **The "start from nothing" target has a large hidden cost that no gate can see.** r16's
  own numbers were not bad — 1236 cage faces against r14's 1032, so it carried *more*
  geometry, not less. What it did not carry was the settled judgement: the skirt flange,
  the belt tab, the three-band boots, and a hundred smaller numbers that had each been
  argued to a conclusion in an earlier round. A rebuild re-litigates all of them at once
  and wins none of them back.
- **r16's brief opened on a false premise.** Its first line was "Round 15 is the safe path
  and is pushed". Round 15 never existed — its brief was written, but no `dwarf_r15.py`,
  no renders and no report were ever produced. Anything in r16 reasoning from that premise
  was reasoning from nothing.

**What r16 did prove, and r17 keeps:** the wedge is real and costs almost nothing; four
value steps cannot separate top / lit corner / front / lit side, so the chamfer painted
identical to the front and the third plane was invisible; and value has to vary by FEATURE,
not only by normal. Those are r16's findings and they are carried forward. `sheet_r16.py`'s
extraction and `render_r16.py`'s envelope gate are worth keeping on their own merits.

So round 17 is not "r16 was wrong". It is: **take r14's settled figure, and apply what r16
proved to it.**

---

## 1. Phase A — the fork: r17 = r14 plus four changes

Every tuned number in `dwarf_r14.py` is untouched. Four things changed:

- **`prism8()`** — a real octagonal cross-section, so a mass has surfaces that actually
  *face* its corners. r14 had none; its corners were the meeting of two axis-aligned planes.
- **`chamfered()`** — two *stacked prisms* in place of two axis-aligned boxes. It must keep
  returning **two** entries: `PAINT` and `WEIGHTS` key per-box overrides **by index**, so
  collapsing it to one silently repaints and mis-weights everything downstream.
- **`wedge()`** — a one-entry drop-in for `box()`, applied to hair, beard and pack. Masses
  under `SMALL_WEDGE = 0.045` fall back to square, because below ~45 mm a chamfer is a
  sliver. **Never applied to the head**: tried and reverted, because the face island is
  projected onto those boxes' +Y faces and chamfering them turned the brows, eyes and nose
  soft.
- **`orient()` + `STEPS`** — a fifth class, `corner`, at **0.92**, between front 1.00 and
  side 0.85. It was first set to 1.09, *brighter* than the front, which made every wedge
  read as a highlight strip rather than as a third plane. This is r16's five-step finding
  applied to r14's figure, and it is what makes the wedge visible at all.

Result: **14 → 66 distinct face directions, 1032 → 2166 cage faces, and 0 px of silhouette
difference from r14 in both pinned views.** The last is not luck: a vertical corner chamfer
cannot move an orthographic silhouette, because the extreme x and y faces still exist.

Two facts recorded so they are not re-derived:

- `dwarf_r14.py` regenerates the committed r14 blend exactly — 21 parts, 1376 verts, 1032
  faces. The generator **is** r14's source of truth; there are no hand edits to lose.
- The rig's weight loop assumed **8 verts per box** and stepped the mesh in blocks of eight.
  That was true while every part was a box. Prisms have 18. Fixed to carry the real
  `box_starts` offsets, which the builder now writes onto each object.

---

## 2. Phase B — the defect the fork carried back (#94)

Forking from r14 rather than from r16 reintroduced a bug r16 had already fixed.

`hex_rgb` put every approved cell through `srgb_to_linear` before writing it into an 8-bit
image, **whose `pixels` are already display-encoded**. So the packed PNG shipped linear
bytes, and the game — reading `baseColorTexture` as sRGB, per the glTF spec — drew the whole
figure too dark. The packed image was also still named for r14.

Fixed in `96fc338`: drop the transform, rename `TEX`/`MAT` to r17, rebuild both blends from
the patched script, promote the export. Geometry untouched — 21 parts, 3324 triangles,
identical part by part, topology all zeros, rig 19 joints with no unweighted or
soft-weighted verts. The artifact now reads `#E9D2BB` and `#F0A63C` where it read `#D0A47F`
and `#DE610C`, so the flame assertion that was widened on the r14 promotion is **narrowed
back to the approved value alone** in `scripts/tests/test_check_asset.py`.

### 2.1 A postscript found during phase C, which matters to anyone judging this figure

**The live Blender session was serving a stale, pre-fix atlas.** On the first rebuild of
phase C the figure visibly lightened, which was checked rather than waved through:
`T_VoxelDwarf_r14` still in the blend holds `#DE610C` and `#D0A47F` — the linear-encoded
values, the exact signature of #94 — while the atlas rebuilt from the committed script holds
`#F0A63C` and `#E9D2BB`, matching what `96fc338` says the artifact should read.

So the dwarf on screen before that rebuild was the **dark, pre-fix** figure. Any judgement
of colour, value or the wedge's readability made in the session before that point was made
against the wrong image. **Rebuild from the script before judging paint.**

---

## 3. Phase C — the two seams

A range-of-motion pass over the promoted r17 rig found that **3 of 24 part-pairs that touch
at bind open a visible slit when a joint rotates.** The rig itself is sound: the influence
map is correct on all 19 joints, perfectly mirrored, and the props follow their holding
hand. The defect is geometric — the masses barely overlap, so rotation parts them.

### 3.1 The tool: `src-assets/blender/seam_check.py`

New this round. For every pair of parts in contact at bind (≤ 3 mm) it measures the worst
surface-to-surface gap across each joint's range and prints a table; pass is no pair over
10 mm.

**It measures point-to-SURFACE, via `BVHTree.FromPolygons` + `find_nearest`, in both
directions.** The obvious metric — nearest vertex of A to nearest vertex of B — finds *one*
contacting pair on this figure instead of 24, and that is not a tuning problem: every part
here is a stack of boxes, and a small box seated inside a big one (the shin inside the boot
cuff, a lock inside the hair mass) has its vertices at the corners, far from the large box's
own corners, while the two surfaces are flush. Vertex distance measures the corners; the eye
sees the surfaces. Both directions, min of the two, because A's vertices can be far from B's
surface while B's are hard against A's.

Run in the live session:

```
exec(open(r"src-assets/blender/seam_check.py").read()); report()
```

`sweep(bone, axis, degs)` prints one joint's profile, which is the control described in §3.5.

### 3.2 The table

| pair | bind | before | after | worst at |
|---|---|---|---|---|
| `r17_hair` ↔ `r17_torso` | 0.4 mm | **34.1 mm** | **3.6 mm** | neck −20 → −4 |
| `r17_boot.R` ↔ `r17_leg.R` | 1.6 → 0.4 mm | **17.6 mm** | **2.5 mm** | foot.R −15 → +25 |
| `r17_boot.L` ↔ `r17_leg.L` | 1.6 → 0.4 mm | **17.6 mm** | **2.5 mm** | foot.L −15 → +25 |

PASS — 0 of 24 pairs over 10 mm, and **the other 21 pairs are unmoved to the decimal.**

### 3.3 The ankle, and the obvious fix that was wrong

`derive()` sets `p["boot_cuff_t"] = h(0.157)` = 0.1884 m, and `part_leg()`'s shin box starts
at z = 0.190. The only thing bridging them is the **cuff lip** box, which reaches 0.1944 and
overlaps the shin by 4.4 mm. That is not enough: the pair opens 12.5 mm at only −10°, and a
walk cycle rotates the ankle every step.

The obvious edit — drop box 0's bottom from 0.190 to 0.150 — **would have moved the
silhouette.** The shin's back face is `y_shin_b` (−0.094) and the cuff's is
`y_shin_b + 0.009` (−0.085), so a lowered shin stands 9 mm proud of the cuff's back between
z 0.150 and 0.1884, and that stretch is a notch in the SIDE outline. Instead, an **appended**
interior box, inset in y and x:

```python
box(xo - p["w_limb"] + 0.004, xo - 0.004,
    p["y_shin_b"] + 0.013, 0.096,
    0.150, 0.190 + o),
```

38 mm of leg column buried inside the cuff, against a 17.6 mm worst opening. Appending is
safe by inspection: `WEIGHTS["r17_leg.R"] = {"*": "knee.R", 1: "hip.R"}` gives a box 2 the
`knee` default, which is the joint it must follow, and `PAINT["r17_leg.R"] = ("pants", {})`
has no per-index overrides. Box 0 stays the shin for the 0.164 depth check.

### 3.4 THE FINDING OF THE ROUND: the nape seam is not a hole

The hair/torso pair measures 34.1 mm at neck −20, which reads as "a slit opens across the
nape". **It is not a slit, and the metric is not measuring what the eye sees.**

Head, neck column and hair are **one rigid assembly** — `WEIGHTS` gives head box 9 to
`neck` and everything above it to `head`, and `head` is `neck`'s child — so a nod cannot
move any of them relative to each other, and nothing opens to daylight. What moves is the
**sightline**. The hair's back band hangs at y −0.176..−0.130 and the neck column's back
face is at −0.128: two millimetres of cover. Tilt the assembly 20° and a horizontal ray
passes *under* the band's bottom edge and lands on the neck column, which paints skin.

**What Wolf sees is a pale strip appearing across the nape**, bright against the hair —
not a gap. Ray-cast from a rear camera at neck −20, z 0.852 through 0.876 all return
`r17_head`; at bind every one of them returns `r17_hair`. That is the defect, stated
exactly.

Two consequences:

- **The fix is an occluder, not a spacer.** A first attempt placed a box to satisfy the
  distance metric and drove hair↔torso to 0.2 mm — and the render was unchanged, the pale
  strip still there. A green number and an unfixed defect. What works is a tongue of hair
  hanging down the nape immediately behind the neck column, low enough that the tilted
  sightline still meets hair:

  ```python
  box(-0.145, 0.145, -0.148, -0.130, 0.770, 0.847),
  ```

  Every nape sightline at neck −20 now returns `r17_hair` again.
- **Hide the pack to see it.** The pack covers the whole nape from directly behind, so the
  defect is invisible in a plain rear render while being plainly visible from the side. The
  diagnostic renders in `src-assets/renders/r17/` are taken with `hide_render` on the pack:
  `nape-bind-control.png`, `nape-neck-20-before.png`, `nape-neck-20-after.png`.

### 3.5 The silhouette constraint, and how it was proved

**0 px changed, front and side.** Method: ortho renders with `film_transparent` on,
denoising **off**, fixed seed and fixed samples, diffing the **alpha** channel — alpha *is*
the outline, and a denoised before against a denoised after can differ by noise alone.

The control that makes the seam numbers trustworthy: the gap must scale with the angle it is
blamed on. Before the fix, neck ran **0.4 / 9.1 / 17.6 / 26.0 / 34.1 mm** across
0/−5/−10/−15/−20, then *collapsed* to 23.7 at −25 and 12.7 at −30 — the metric changing
subject as a different surface becomes nearest, not the slit closing. After the fix the
profile is flat at 0.2–3.6 mm across the whole range, which is the real signature of a
closed seam.

Two things this caught that reasoning had missed:

- **The figure's bounding boxes lie.** `r17_torso`'s bounds say its back is −0.1794; its
  back *face* above z 0.78 is **−0.1614**, and the pack's front face is **−0.1714** — a
  10 mm open channel runs between them. A box placed in the hair's own rear plane, which is
  the natural place for it, sat in that channel and changed **244 px** of the side outline.
  The torso also steps to x ±0.150 / back −0.1494 at z 0.846, and the head to x ±0.130 /
  back −0.1280 above 0.848. The extents now in the source are **ray-cast face by face** at
  the specific heights, and the comment records the table.
- **Check the noise floor before trusting an RGB diff.** Before/after differed in 1,607 px;
  two renders of *identical* geometry at different seeds differed in **201,680**. The change
  is far below the noise floor, so there is no shading change — but an RGB diff read on its
  own would have looked alarming.

### 3.6 One thing deliberately not taken

Running the nape tongue up to `head_ends` closed a **real 0.4 mm see-through crack** that
exists at bind, between the torso's top (0.8480) and the hair's bottom (0.8484), where the
pack's front face (−0.1714) and the head's back (−0.1280) leave nothing standing behind
y −0.148..−0.130. It reads as 19 partial-alpha pixels in the side outline.

Closing it is arguably an improvement. **The fix does not need it**, and closing a crack
that has been in the outline since r14 is not this round's call, so the tongue stops at
0.847 and those 19 pixels are exactly as they were. **Open for Wolf.**

---

## 4. Three errors found in the inputs, named

- **The phase-C brief's mechanism was slightly wrong.** "The leg and the cuff never overlap
  at all — they sit 1.6 mm apart" is true of the cuff box, but the **cuff lip** at
  `dwarf_r17.py:756` reaches 0.1944 and overlaps the shin by 4.4 mm. It does not change the
  fix, but it sends you looking for a bug that is not there.
- **`arm.pose.bones["foot.R"].rotation_euler.x = radians(-20)` does nothing as written.**
  Pose bones default to `rotation_mode = 'QUATERNION'`, so the euler is **silently ignored**
  — the figure stays at bind while every number looks right. Set the mode first.
- **A Cycles RENDERED viewport screenshot over the MCP returns a black frame**, because the
  grab happens before the viewport converges. Render to a file and read that; use MATERIAL
  shading for live looking.

---

## 5. What moved on disk

```
 M src-assets/blender/SM_VoxelDwarf_Miner01.blend        143890 -> 171614 bytes
 M src-assets/blender/SM_VoxelDwarf_Miner01_r17.blend    143890 -> 171806 bytes
 M src-assets/blender/dwarf_r17.py                       +67 lines
 ?? src-assets/blender/seam_check.py
 ?? src-assets/renders/r17/
```

Both blends saved **from bind**, from the same in-memory build, compressed (a first pass at
`compress=False` ballooned them to 1.08 MB). The scene was restored to the same 26 objects
it was found with, back on EEVEE with `cam_lit`: the six measuring cameras and the
inspection light added during the round were removed before saving.

**The runtime slot was NOT re-promoted.** `assets/gltf/SM_VoxelDwarf_Miner01.glb` still
carries the phase-B export. Promotion is a commit-level decision and waits on Wolf.

Figure, after phase C: **21 parts, 66 distinct face directions, 2184 cage faces, 3360
triangles**, atlas 75 cells / 77 distinct colours.

---

## 6. Verification

```
EXPORT  3360 triangles of 100000 budget
  topology     n-gons 0  non-manifold 0  loose verts 0  loose edges 0  degenerate 0  flipped 0
  holes        0 boundary edges, 0 EXPOSED to the outside
  rig          19 joints, 0 missing, 0 unexpected, 0 unweighted verts, 0 soft-weighted
  material     M_VoxelDwarf_r17 / T_VoxelDwarf_r17
```

`check_asset` → **EXIT 0**, `profile=painted-map`, palette carrying `#E9D2BB` and `#F0A63C`.
All **17** `test_check_asset.py` tests pass, including the promoted-runtime-dwarf flame test.

**`scripts/gate.sh` was NOT run green — it cannot run on this machine.** `cargo` is absent,
so all six Rust steps report `command not found`, and the `python` on PATH is the Windows
Store stub, which fails the three Python steps. **Not one failure is about the tree**, and
this round changed no Rust. Flagging that rather than claiming a gate that was not obtained.

---

## 7. Still open

- **The pickaxe overshoots the figure** — `z 0.230..1.322` against a 1.2 m figure, so it
  stands 12 cm proud of the hair and reads as a leaning pole rather than a carried tool.
  This is r14's own geometry, inherited untouched. Both props *do* intersect their gloves,
  so it is a reach problem, not r16's mid-air defect.
- **The 0.4 mm bind crack at the nape** (§3.6) — leave or close.
- **Which r16 files to keep.** `sheet_r16.py`'s extraction and `render_r16.py`'s envelope
  gate earn their place; `dwarf_r16.py`, `paint_r16.py`, `rig_r16.py` and the two A/B blends
  are now dead weight.
- **Promotion and commit** of phase C.

The r14-detail question from r15 §9 — skirt flange, belt tab, three-band boots — is **moot**:
r17 is r14's lineage, so it has them.

---

## 8. Cost

| row | value |
|---|---|
| `dev-art` | phase C only: 1 range-of-motion pass, 2 boxes, ~45 tool calls |

Phase C's honest shape: the two geometry edits were trivial and the measuring was the whole
round. Three of the four iterations on the nape box were spent discovering that bounding
boxes do not describe this figure's cover, and one was spent discovering that the defect was
occlusion rather than a gap — after a version that had already driven the metric green.
**The number going green was the least reliable signal in the round.**
