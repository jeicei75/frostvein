# Round 15 — rebuild from the art's own profile, with the gate wired in first

**This round changes the METHOD, not the target.** Round 14 reached a figure whose outline sits
1.2–2.4 source pixels off the sheet, with correct form, a working rig and a clean export — but it
got there by ~40 builds of tweak-measure-tweak, and most of that was avoidable. Nearly every
correction traced to one of three things: **building from sampled numbers instead of the drawing**,
**measuring with an instrument that could not see the error**, or **adding geometry because a number
was low**. This brief removes all three up front.

Target: **start from nothing and land where round 14 landed, without the tuning.**

Read §1 and §2 before anything else.

---

## 1. Inputs, and the one that is NOT the authority

| input | authoritative for |
|---|---|
| `references/dwarf-ortho/front.png`, `side-left.png` | **the silhouette, row by row.** These two are pinned and are the geometry authority |
| `references/dwarf-ortho/back.png`, `side-right.png` | cross-checks; unpinned, drawn at different scale, derive and PRINT the mapping |
| `references/dwarf-ortho/gear.png` | **the pickaxe and lantern.** Open it. Round 14 wrote "gear.png draws…" in a comment before opening the file and got the pick's shape wrong |
| `references/dwarf-frames/` f084 f088 f092 f100 f104 f120 f140 f164 | **form, depth and interior detail.** f100 is the best head in the set. Open all eight |
| `references/dwarf-contact-sheet.jpg`, `reference-sheet.jpg` | palette and gear, upstream source |
| `references/dwarf-model-sheet.md` | **the numbers — but see §2. It is a set of samples, not the drawing.** |

**Report which reference files you actually opened, and when.** Round 14 wrote confident citations
for three files it never opened.

## 2. THE TABLE IS SAMPLES. THE ART IS THE AUTHORITY.

The model sheet's tables are **twenty sampled landmarks**. The drawing they were read from carries
far more steps, and the two **disagree by 1 to 5 source pixels** at specific places. §8's gate is the
outline, so where they differ the art wins. Round 14 verified these; the model sheet now records
them:

| feature | table | the art |
|---|---|---|
| crown at row 15 | 0.336 H | **0.320** — 0.336 is the art's row *sixteen* |
| ear top | 0.850 H | **0.843** — the edge steps out at row 29 |
| boot cuff top | 0.164 H | **0.157** — steps out at row 125 |
| pack back | flat at column 12 | **tapers** −0.285 H at row 45, −0.331 at rows 68–75, −0.275 by row 94 |
| head front | column 83 | **+0.170 H** rows 15–23, **+0.181** at the brow, row 24 |
| nose | (a block) | **a ramp**: 0.189 / 0.196 / 0.210 / 0.217 H across rows 31–38, back to 0.196 by row 43 |
| crown depth | (absent) | front edge +0.131 H at row 7 out to +0.167 by row 10 — the crown is DEEP |

**So the first deliverable is an extraction, not geometry.** Write `sheet_profile()` before anything
else: for each source row 7–147 of `front.png` and `side-left.png`, print the figure's left and right
edges in H. Build `PARAMS` from that table. The sampled numbers are a cross-check, not the source.

Two things the extraction must handle, because both bit round 14:

- **The crops carry furniture.** `front.png` has a dimension arrow down its left margin and
  `back.png` a printed caption. Drop ink runs thinner than 8 px; do NOT take only the longest run,
  because the figure legitimately splits into two at the legs.
- **The art does not expose the torso or the skirt.** The posed arms cover them in front, side AND
  back — back.png at row 49 reads ±0.262 H, which is the sleeve caps, not the chest. Those two come
  from the table, and only those two.

## 3. The gate: the OUTLINE, row by row, from the first build

**Landmark checks are not a gate.** Round 14 had thirty of them reading **+0.00 px** while the
outline was 4–5 px off: a box added between two landmarks moves the silhouette without moving any
of them. Build `envelope()` before you build the figure.

`envelope()` renders front and side-left at the pinned scale, walks every row, and reports the worst
overshoot per z-decile in source pixels. Rules it must follow:

1. **Symmetrise the front sheet** about column 77 before comparing. The figure is symmetric by
   design (Wolf, round 14) and the art is hand-drawn asymmetric — row 12 is −0.124 H left against
   +0.139 right. Comparing raw charges you for the sheet's own wobble, and cost round 14 a full
   source pixel of phantom error. The side view has no such symmetry; compare it raw.
2. **Exclude the props from OUR silhouette.** §8 gates head, hair, beard, torso, skirt, boots and
   pack. Ours hangs the pickaxe vertically in a neutral hand where the sheet carries it across the
   body; left in, it reads as +51 px of overshoot that is not geometry.
3. **Report the arms separately.** They are exempt (§6, pose).

### The tolerance, and what a source pixel is actually worth

**≤ 3 source px on the body passes. Anything beyond that flags and is explained.**

One source pixel is 8.571 mm, 1/140 of figure height. Against the distances the figure is actually
seen at:

| context | figure height | 1 source px on screen |
|---|---|---|
| gameplay — this sheet's own boot-camera measure | **8.74 px** | **0.06 px** |
| readability strip / LOD2 | 60 px | 0.43 px |
| LOD1 inspection | 200 px | 1.4 px |
| LOD0 hero render | 500 px | 3.6 px |

Round 14 finished at 1.2 px front and 2.4 px side. At the distance dwarves are actually seen that
worst case is **0.15 of a screen pixel** — invisible, and the builds it spent closing the gap from
5 px were largely wasted. Do not chase this number below 3.

**The envelope's real job is catching drift, not proving fidelity.** It caught four separate
additions wandering off the sheet inside a single build, each of which every landmark check passed.
That is what it is for.

**And list the parts the art never exposes.** The torso and the skirt sit behind the posed arms in
all four views, so their envelope figure is *unmeasurable*, not passing. One number hides that.

## 4. Method

One generator, `src-assets/blender/dwarf_r15.py`: a `PARAMS` dict derived in code from the extracted
profile, and a `build()` that deletes and rebuilds collection `SM_VoxelDwarf_Miner01_r15`. Parts are
separate objects, each a stack of boxes. Start from an empty scene.

```
r15_head  r15_hair  r15_beard  r15_moustache  r15_torso  r15_skirt  r15_belt  r15_buckle
r15_sleeve.L/R  r15_glove.L/R  r15_leg.L/R  r15_boot.L/R  r15_pack  r15_strap.L/R
r15_lantern  r15_pickaxe
```

### 4.1 Chamfer the big masses — it is free

The reference's roundness is not curvature: its masses **step their corners**, so a three-quarter
view shows a narrow third plane between front and side. Plain boxes read as slabs however correct
their dimensions are. Build each big mass as **two boxes** — one full-width and recessed in depth,
one full-depth and narrower in width. Their union chamfers all four vertical corners, and **neither
orthographic silhouette changes at all**, because one box carries front/back and the other carries
both sides. Apply to torso, skirt, sleeve caps, boots, pack, and the head's front and crown.

### 4.2 Shapes the art dictates

- **crown**: six or seven steps between rows 7 and 16, from the art's own edge, not four bands
- **nose**: a ramp across rows 31–43, not a block
- **brow**: two masses with the nose bridge recessed between them (f100), not one bar
- **hair**: lobes coming forward past the ears and down the cheeks in two depth layers (f100)
- **beard**: centre mass proud of the sides, stepped bottom, cheek masses at the sideburns. The
  art's front DIPS at rows 43–49, between the moustache and the beard's main mass — that dip is
  real form
- **pack**: tapered, deepest at rows 68–75
- **pickaxe**: symmetric double pick, both arms sweeping out and down to points, collar at the haft
  and a banded grip (`gear.png`)
- **lantern**: a frame — corner posts, vented cap, footed base — not a solid block

### 4.3 The neck, which two rounds got wrong

"Up to 0.064 H wide" is measured on `side-left.png`, so it is the **DEPTH of the visible skin
column, not the neck's thickness**. The column is not the neck: it is what the jaw and the hair
leave uncovered. Build the throat at full anatomical size (~0.150 H across, full depth) and set the
**occluders** — hair lobes behind at y ≤ −0.130, jaw in front from −0.050 — so the gap between them
is the sheet's 0.064 H. Anything shoulder-borne that rides above 0.700 H will eat into the run.

### 4.4 Paint

One material, one packed 512² atlas, `Closest`, backface culling, Specular IOR Level 0.5. Every flat
face maps to the **centre of one palette cell**, so a face is exactly one colour and "crisp value
steps, no gradients" holds by construction. Value steps come from the polygon **normal**, so rotated
arm boxes step like axis-aligned ones. The face front gets a fixed rectangular island with a planar
projection painted in **world coordinates** — the eyes then sit on the 0.807 H eye line and cannot
drift. No `smart_project`.

**Value bases are the LIGHT cell, with the approved cell as the shadow step.** Round 14 built hair on
`#34271C` and beard on `#5E4632` as bases; through the orientation multipliers those range down to
`#221A12` — black in everything but name, and the two masses merged. Use `#513C2B` and `#826145`
(both from r3's recorded 23) as bases instead.

**Every box whose front stands proud of the face must be on the face island.** A proud box left on a
flat palette cell covers the paint behind it — that is how round 14's brows disappeared entirely.

## 5. Judge the look under LIGHT, not under Workbench

Workbench flat is correct for the gates — silhouette, albedo classification, readability — and it is
**actively misleading for "does it look like the reference".** Round 14 spent most of its length
reasoning about art from neutral studio renders and concluded the approved palette was too
desaturated. It is not. Under a warm key with AgX the same texture reads as the reference's olive
tunic and tan boots.

So this round delivers **both**: the Workbench set for the gates, and a lit diagnostic — warm key,
dark environment, AgX, AO — **posed to match the frame**, beside `f104` and `f088`. The figure is
judged by eye on the lit one.

The palette question is closed and does not reopen: the approved ten stand, and "the reference looks
warmer than our palette" has an answer — it does not, under comparable light.

## 6. Pose

A-pose for the bind, arms out ~40°, ≥ 20 mm armpit clearance. Silhouette will not match at the arms;
expected. **But the sheet's hand-bottom at 0.330 H is a POSED measurement** — take arm LENGTH from
it (0.370 H shoulder-to-hand), not arm height. For the lit diagnostic, pose the rig to the frame.

## 7. The measurement traps — all of these cost round 14 a build or more

Wire each of these into the harness from the start:

1. **Filter boxes by OVERLAP, never vertices by band.** A box straddling a z-band with both corners
   outside it vanishes from a vertex filter. This alone produced six false failures in one run.
2. **Never name a box by index in a check.** Every restructure shifts them, silently, and the check
   then measures a different mass while still printing a number. This happened **four times**. Use
   heights or object extents.
3. **`width_at` on axis-aligned bounds over-reports rotated boxes.** The 40° upper arm's AABB reaches
   x 0.404 and spans z 0.674–0.838, so any height sample inside the shoulder cap swept it in and
   reported the shoulders at 0.674 H against 0.528.
4. **Pull band tops down half a render pixel.** Geometry that merely touches a pixel boundary lights
   the row above it, so every step renders one row early and reads as ~4 px of overshoot on that
   single row while its neighbours are 0.0.
5. **Check for degenerate boxes.** One with z running backwards held the crown a step too wide; a
   normals check cannot see it, because a box with no extent has no outward direction.
6. **Check normals against each box's own centre, every build.** Inside-out faces are invisible to
   every dimensional test — round 14 shipped eight builds with every arm and hand face reversed while
   all thirty landmark checks read +0.00 px.
7. **Check that no head box is the rearmost surface above the shoulders.** Skin showing through the
   back of the hair is a 1 mm margin error that no dimension can see.
8. **Silhouette step density is NOT resolution independent.** A diagonal steps once per row at any
   scale, so a 5× render reports a fifth the density. Count at the sheet's own resolution. The model
   sheet's published 10.3 and 11.2 were counted on a ~335-row render; on the 140-row source its raw
   counts are **24.6 (side) and 26.8 (front)**. Those are the targets.
9. **The visible-skin check moves when you add palette cells.** It classifies to the nearest cell, so
   new mid-browns pull antialiased boundary pixels out of the skin family with no model change —
   round 14 saw 14.7 % → 10.1 % from a colour reassignment. Report it with the cell list.

## 8. Stages and checkpoints

- **Stage A — extraction and blockout.** `sheet_profile()` output, then all parts, grey, with
  `envelope()` reporting from the first build. **Stop and show Wolf.**
- **Stage B — paint**, including the lit diagnostic. **Stop and show Wolf.**
- **Stage C — rig and export.** 19-joint contract, rigid weights **assigned by BOX not by object**
  (the neck box lives inside the head but belongs to `neck`; the torso's waist to `spine`; the
  sleeve's forearm to `elbow`). Split the leg into shin and thigh so hip and knee each own a whole
  box — a single leg box has to be divided vertex-by-vertex and shears.

## 9. Budget, and the rule with teeth

- **≤ 120 tool calls.** Round 14 took far more, and §7 is most of the difference.
- **No operation added because a metric moved.** Round 14 broke this: told "still only 1,356
  triangles", it added ~600 triangles of skirt folds, belt stitching and finger blocks, then wrote
  reference citations to fit. Checking the frames afterwards, none of it was there and it was
  removed. **Every added box must cite a file and a row or frame in its comment, and the file must
  have been opened.**
- **Fewer triangles is never a failure.** Round 14 finished at 2,064 on 1,032 cage faces.

## 10. Deliverables

| # | what | path |
|---|---|---|
| 0 | live-scene report, and the list of reference files opened | in the report |
| 1 | the extracted sheet profile, both pinned views | in the report |
| 2 | generator and render scripts | `src-assets/blender/dwarf_r15.py`, `render_r15.py` |
| 3 | the `.blend`, saved per run, image packed | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 4 | overlay renders, four views, each checkpoint | `renders/r15/overlay-<A\|B>-*.png` |
| 5 | envelope report, worst per decile, both views | in the report |
| 6 | **lit diagnostic, posed, beside f104 and f088** | `renders/r15/lit-vs-f104.png`, `lit-vs-f088.png` |
| 7 | face close-up and readability at 100 px and 60 px | `renders/r15/face-*.png`, `readability.png` |
| 8 | deflection and carry renders | `renders/r15/joint-*.png`, `pose-*.png` |
| 9 | texture | `src-assets/blender/textures/T_VoxelDwarf_r15.png` |
| 10 | report under 150 lines, with exporter and checker output verbatim, cost row `dev-art` | `src-assets/prompts/dwarf-miner-round-15-report.md` |

## 11. Tooling

`export_dwarf.py`: bump `REV` to `r15` — the exporter's own header calls that the one line each
round changes. `FORM_PLANE_FLOOR` stays 0; the plane metric is retired and the count is printed but
not acted on. `check_asset.py` must exit 0; note that it reports `profile=painted-map` for a
hand-modelled figure, which makes its grid and quad-soup clauses **inapplicable, not skipped** —
consistent with §3's "there is no lattice".

## 12. Not in this round

Animation, LODs, the plane-of-form metric, re-deriving the palette from the video, and any attempt
to carry round 14's geometry across.
