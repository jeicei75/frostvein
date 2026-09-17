# Round 6 — build the detail the boxes were never allowed to carry

**Build the model from scratch, in Wolf's open Blender, where he can watch it happen.**

Round 5 was a good round. It measured the orthographic sheet instead of the video, got every depth
right, built a complete figure at 1,116 triangles, and reported four defects in its own input
documents. Wolf's verdict was *"closer but not happy"*, and when asked what was still wrong he named
two things and dismissed a third:

> *"I think the problem is not stance but stance should come with the rig not with model anyway.
> Problem is resolution and our previous decision to limit rotating cubes. Now we don't need to have
> rotating cubes limitation anymore like we talked yesterday. We should start new brief from the
> scratch and make sure that I can follow how it progress in blender."* — Wolf, 2026-09-12

So this round has exactly three differences from round 5, and everything else is inherited:

1. **Rotated boxes are allowed.** The clause that forbade them is gone.
2. **Resolution — more, smaller boxes**, with a number behind it for the first time (check 4).
3. **It must be watchable.** The figure appears in Wolf's viewport, part by part, as you work.

**Stance is explicitly NOT this round's problem.** Round 5's own report recommended a pose as the
biggest remaining likeness gap; Wolf has overruled that and the reason is sound — a pose belongs to
the rig, where it is one transform and reversible, not to the mesh, where it is baked in. **Build a
neutral standing base. Do not pose the arms, do not chase the sheet's arms-forward position, and do
not tilt the figure for "life".** If the side view looks emptier than the sheet because the arms
hang, that is correct and expected; the rig round fixes it.

---

## 1. Rotated boxes are allowed — and here is what replaces the rule

**The ruling, 2026-09-11:** *"maybe we get rid of no rotated boxes limit … that came from voxels look
I think."* He is right about the provenance. The axis-aligned-normal clause was invented as the
replacement for the voxel lattice when that was dropped, so it descends from the same requirement,
and it was costing real things: rounds 4 and 5 both reported the same four features it blocked
outright — the slung bedroll, the diagonal chest straps, the pick head's arc, and the stepped forms
that only work on a diagonal.

**Why rotation was never the thing that guarded the style:** a rotated box is still a box. Hard
edges and flat faces are what read as blocky, and a box tilted 30° has both.

**What is still forbidden, and this is the clause that actually does the work:**

- **No smooth normals and no curved surfaces.** No subdivision, no bevels or chamfers, no cylinders,
  spheres or cones, no shade-smooth anywhere, no normals averaged across an edge.
- **Every face flat and planar.** A quad whose four corners do not lie in one plane is not a box
  face.
- **No modifiers at all.** A box model needs none, and a modifier is how a curve sneaks in.

**This is enforced, not requested.** `src-assets/blender/export_dwarf.py` has already been changed on
the orchestrator's side: its axis-aligned test is replaced by a flat-and-planar test that **fails the
build** on any smooth-shaded face, any non-planar face, any custom split normals, and any modifier.
It has been run against round 5's `.blend` (passes, 0/0/0/0) and against three deliberate mutations —
a shaded-smooth face, a face bent 1 mm out of plane, and an added Subdivision modifier — and it
fails the build on each. Its report line now reads:

    smooth-shaded     0   non-planar 0   custom normals 0   modifiers 0

**Axis-aligned is still the DEFAULT.** Rotate for a stated reason — a strap that follows the body, a
slung bedroll, the pick head's arc, a boot that toes out — and **list in your report every part you
rotated and why**. A model where everything is slightly rotated has lost the discipline as surely as
one that could not rotate at all.

## 2. Resolution — this is the round's headline, and it has a target

Wolf, on round 5: *"resolution could be higher with more details."* Measured, that is real and it is
about 40 % short. See **check 4** in `src-assets/references/dwarf-model-sheet.md` for the method.

| view | the reference | round 5 |
|---|---|---|
| side | **10.3** silhouette steps / 100 rows | 7.1 |
| front | **11.2** silhouette steps / 100 rows | 6.3 |

**And the sharper finding the averages hide: round 5's silhouette is asymmetric.** Side L33/R12,
front L13/R27, against the reference's near-symmetric L33/R36 and L37/R38. **One side of that model
is a straight slab** — which is precisely what "flat blocky stick" has been describing since round 4.

**The targets, both views:**

1. **>= 10 silhouette steps per 100 rows** of figure height.
2. **Neither edge below 70 % of the other** on the same view.

**What this asks for, stated plainly because the question has looped three times: MORE AND SMALLER
BOXES where the sheet shows a step we do not have. It is NOT an argument for a voxel lattice, a
finer grid, or a uniform cell size.** The sheet's own steps are the list: the stepped crown, the
brow, the nose, the ear, the pack standing off the back, boot layers, strap plates, the belt's
buckle, the tunic hem, the beard's stepped mass.

**There is no triangle budget.** Wolf's standing ruling is *"let's keep stretching limits first"*.
Round 5 spent 1,116 against an eventual LOD0 ceiling of 4,000, so the headroom for this exists three
times over. **Spend it on steps, not on smoothness**, and do not optimise anything.

## 3. It must be watchable — Wolf is sitting in front of the viewport

This is a hard requirement of this round, and each of the last three rounds failed it in a different
way: round 3 ran entirely headless and Wolf watched nothing happen for a whole session; round 4's own
report (§9.9) found the same gravity pulling it back toward a generator; round 5's first attempt
wrote a `.blend` from a subprocess that Wolf's open Blender could not see.

- **Author INSIDE Wolf's running Blender, through the MCP addon. Never in a `blender --background`
  subprocess.** Only the final export may run headless.
- **Prove it before your first edit.** Query the live scene and **report what was already in it** —
  the objects present, the current filepath. A subprocess starts from an empty or default scene; the
  instance Wolf is sitting in front of does not. **If you cannot reach the live instance, stop and
  say so.** Do not fall back to writing a `.blend` from a subprocess: the file appears on disk, the
  work looks done, **and Blender never reloads a file that changed underneath it**, so Wolf watches
  an unchanged viewport and the session looks broken when it is merely invisible.
- **Get geometry on screen within minutes.** Measure the front view only, **block the whole figure
  out immediately** from those numbers, and measure each part's depth off `side-left.png` as you come
  to it. If you are thirty minutes in with no boxes, you are doing this wrong — that is a direct
  quote from round 5's first attempt: *"30mins and it builds with python... nothing on screen"*.
- **One part per tool call, so parts appear one at a time.** Do not build the figure in a single
  call that fills the viewport in one jump.
- **After the blockout, edit the OBJECTS IN THE SCENE.** Round 5 kept its part specs in a file and
  re-ran a builder that wiped and rebuilt the whole figure in forty seconds. It was fast, and it is
  exactly what makes a session unwatchable and the `.blend` a disposable artifact. A full
  wipe-and-rebuild is allowed, but it is now the exception: **announce it in the report, with the
  reason, each time you do it.**
- **Name every object for the outliner** — `r6_head`, `r6_crown_step_1`, `r6_beard_lower` — so Wolf
  can find, hide and select parts while you work.
- **Leave the viewport usable.** Do not park the camera inside the model or leave parts hidden.

**Do not stop and wait for approval.** Wolf's rule from round 5 stands: *"let's let it work now
without asking my comments"*. Work through to a complete figure and deliver. Watchable and
unattended are not in conflict — he wants to see it happen, not to be asked about it.

**Still report every conflict, in writing, in the report.** When an input contradicts the reference,
**follow the reference** and write down what you did and why. Rounds 4 and 5 each found real defects
in their input documents that way; two more are corrected below.

## 4. The neck is bare in profile — round 5 got this wrong and so did the brief

Wolf, 2026-09-12: *"neck is visible on side view of reference images."* He is right, and it has been
measured since: both side views agree to the row that a bare skin column runs **z/H 0.793 down to
0.679** — **0.121 H of visible neck**, up to **0.064 H wide**, landing on the collar just below the
shoulder line.

Round 5's report §9.2 states that the sheet shows no neck in any view and that it built one which is
covered from every angle. **That reading is wrong for the side views.** `dwarf-ortho/README.md` and
**check 5** in the model sheet now carry the correction and the numbers.

**The lever is where the HAIR falls, not how wide the beard is.** The reference keeps a full beard
*and* a bare neck by parting the hair around the neck rather than over it. **Do not narrow the beard
to chase this** — the beard was the single biggest improvement round 5 made over round 4.

## 5. Measure the gear AS DRAWN — the sheet's labels are wrong

Round 5's pickaxe is the weakest part of that figure, and the cause was this brief's ancestor telling
it to trust the gear breakdown's printed labels. Measured off the art instead:

| | the label says | the art draws |
|---|---|---|
| pickaxe, overall length | 0.83 H | **~1.04 H** |
| pickaxe, head span ÷ length | 0.89 | **~0.54** |
| lantern, height | 0.33 H | **0.26 H** |

At 0.83 H the pick cannot hang from a hand without going through the floor, which is why round 5
planted it butt-on-ground and forward of the figure. **At the drawn length it can be held.** The
generalisation, and it applies to every annotation on that sheet: **the drawings are orthographic and
trustworthy; the numbers printed beside them are not.**

## 6. Inherit ALL of the scaffolding — only the geometry is new

Round 5's mid-session correction, and it stands: *"from scratch" means the SHAPE, never the
scaffolding.* Round 5's files are in the working tree. Take them and go.

- **The palette is SOLVED. Copy round 5's atlas and its cell coordinates verbatim**, rename the image
  datablock to `r6`, change nothing else. In atlas order, 15 of 16 cells: `#E9D2BB, #5E4632, #FFFFFF,
  #5F7A6A, #474B41, #A9B2AC, #8B6B50, #6B5B49, #34271C, #F0A63C, #BAA896, #826145, #7DA18C, #44584C,
  #63695B`, **one slot spare**. Cells 10–14 are, in order: skin shadow plane, beard highlight, tunic
  lit plane, tunic turned-away plane, trouser lit plane. Boots, belt, pack, lantern and pickaxe cost
  zero slots by reusing approved cells cross-purpose. **Do not re-derive any of this and do not go
  looking for hexes.** The palette revisit is ruled to wait for Epic 11's lighting.
- **The material is solved.** Reuse round 5's `M_VoxelDwarf` setup unchanged, including the fix that
  matters: **Specular IOR Level must stay at its 0.5 default** — a 0 there emits
  `KHR_materials_specular` and the no-extensions clause rejects the export. Backface culling on.
- **The exporter is solved and already updated for this round.** One command, from the repo root:

      blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
              --python src-assets/blender/export_dwarf.py

  Its `REV` is already bumped to `r6`, so **your collection must be named
  `SM_VoxelDwarf_Miner01_r6`** or the export will tell you it cannot find it.
- **The render settings are solved.** Round 5's flat and lit passes, on `#6F7073`, same paths and
  names. `render_r5.py` is the working script and reads the `.blend`; copy it to `render_r6.py`.
  (`render_dwarf.py` still imports the retired `dwarf_miner` module and would render the OLD dwarf —
  reported by rounds 4 and 5, still true, still not yours to fix.)
- **Round 5's geometry is NOT inherited** — `dwarf_r5.py` and the r5 collection stay on disk as a
  reference you may open beside your own work, but the figure is rebuilt. Rotation and the step
  density change the part decomposition enough that editing r5's boxes would cost more than it saves.

If any inherited piece genuinely blocks you, say so in the report and work around it — but the
default is reuse, and time spent rebuilding solved scaffolding is time not spent on form.

## 7. Everything below is unchanged from round 5 and is not in question

- **Box modelling, no lattice.** Rectangular boxes at whatever size the form needs.
- **Measure from `src-assets/references/dwarf-ortho/`** — 5x nearest-neighbour crops, so every source
  pixel is a countable 5x5 block. **Read its README first.** Work at source resolution and read the
  views as an ASCII map rather than running a colour classifier over them: the crops are blown up
  from a JPEG, every warm material on that sheet shares a chromaticity, and a classifier mislabels
  whole regions. Trust the interior blocks more than the silhouette edge, where the ringing is worst.
- **The model sheet `src-assets/references/dwarf-model-sheet.md` is PROVISIONAL where it was measured
  off the video, and authoritative where it was measured off the orthographic views.** Correct it
  rather than obeying it, and say so in the report.
- **Save the `.blend` after every part**, and write `src-assets/renders/progress/NN-<part>.png`
  showing the WHOLE figure full-size *and* at ~60 px. Four of six delegated runs in this project have
  been killed by the harness; the saves are the crash insurance and the renders are the audit trail.
- **Put the revision in every path you write.** Round 5's renders overwrote round 4's and it then
  analysed round 4's renders as its own. Renders go under `src-assets/renders/r6/`.
- **Blender resolves a relative `render.filepath` against the `.blend`, not the CWD.** Round 5's
  renders escaped to `C:\src-assets\renders\` where git could not see them. Use absolute paths.
- **Parts as separate objects** in the `.blend`, joined into one mesh by the exporter. The beard and
  hair must lift off and leave a complete head underneath.
- **The naming scheme:** source and export share the stem —
  `src-assets/blender/SM_VoxelDwarf_Miner01.blend` exported to
  `src-assets/export/SM_VoxelDwarf_Miner01.glb`, revision `r6` in the object, mesh, material and
  image datablock names. Never write to `assets/gltf/`.
- **Never touch git.** Write files at the paths named here; the operator commits.
- **Never write outside `src-assets/`.**
- **`check_asset.py`'s grid clause will fail a box model.** Expected. Report it verbatim, do not
  quantise the model and do not edit the checker. **Two more of its clauses are known-broken on the
  orchestrator's side and are not yours to chase either:** its palette clause reads only 4 cells of
  any atlas (which is the reason the ceiling is 16, and it under-reports rather than failing), and
  its naming clause has not yet been decided for a revisioned mesh. Report what it says; do not
  change the model to satisfy any of the three.

## 8. Deliverables

| # | what | path |
|---|---|---|
| 0 | the live-scene report that proves you are in Wolf's Blender | in your report, before any geometry |
| 1 | the source, saved after every part | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the render script | `src-assets/blender/render_r6.py` |
| 3 | the exported GLB (gitignored scratch) | `src-assets/export/SM_VoxelDwarf_Miner01.glb` |
| 4 | progress renders, one per part | `src-assets/renders/r6/progress/NN-<part>.png` |
| 5 | the five final views | `src-assets/renders/r6/` |
| 6 | the zoom strip with round 4's speckle metric (`delta = mean abs(nearest - mip)` at 10/30/100 px) | `src-assets/renders/r6/zoom-strip.png` |
| 7 | side-by-side against `dwarf-ortho/side-left.png` **and** `front.png` | `src-assets/renders/r6/vs-ortho-side.png`, `vs-ortho-front.png` |
| 8 | **the silhouette step density of your own figure**, both views, both edges | in your report |
| 9 | your report | `src-assets/prompts/dwarf-miner-round-6-report.md` |
| 10 | the checker's output, verbatim, pass or fail | in the report |
| 11 | your cost — `session_tokens.py` print mode, verbatim, model from your banner, row `dev-art` | in the report |

## 9. How this round is judged

By Wolf's eye against the sheet, plus these, which are mechanical:

- **>= 10 silhouette steps per 100 rows on both the front and the side view**, and **neither edge
  below 70 % of the other** — check 4, and the headline of this round
- **a bare neck at least 0.08 H tall reads in both side views** — check 5
- the crown is stepped, not a single box; the brow, nose and ear break the profile
- shoulder caps exist and the torso is wider at the shoulders than at the waist
- the side profile's depths match `side-left.png` proportionally
- the beard is no wider than the head, and as long as the sheet draws it
- visible skin >= 10 % of figure pixels, **measured by nearest palette cell**
- the pickaxe is at its drawn length (~1.04 H) and is held, not planted
- the export passes flat-and-planar: `smooth-shaded 0  non-planar 0  custom normals 0  modifiers 0`
- nothing turns to speckle at 10 px on the zoom strip
- every datablock name carries `r6`
- **the figure was built in Wolf's open Blender and he could follow it**
- nothing written outside `src-assets/`, and no git command run at all

## 10. What comes after this round, so you do not do it now

**A rig and a pose, in that order** — and stance is theirs, not yours. Joint names are fixed:
`root, hips, spine, chest, neck, head, shoulder.L/R, elbow.L/R, hand.L/R, hip.L/R, knee.L/R,
foot.L/R`, plus `beard`. Rigid weights, one mesh, no quad crossing a joint. **Do not design geometry
that forecloses it:** a part mounts on a declared socket, and no box should span a joint it will need
to bend at.

Lighting is Epic 11's and is not yours. The palette revisit is gated on it.

**The endpoint this is building toward: hand-author the PARTS, generate the COMBINATIONS.** Wolf
wants a unique dwarf per living person — roughly twelve discrete axes plus four continuous dials,
with a dwarf stored as a seed. A variant must be expressible as a part list plus a table of hexes.
