# Round 16 report. The wedge works, and it is not where the deficit was.

**All three stages are done.** Blockout with the envelope wired in from the first
build, the direction metric, the lit A/B, paint, the 19-joint rig, and a clean export
that `check_asset.py` passes.

## 0. The brief's premise was wrong, and Wolf confirmed it

r16 opens *"Round 15 is the safe path and is pushed."* It is not: there is no
`dwarf_r15.py`, no `renders/r15/`, no round-15 report, nothing named r15 in the tree.
**Round 15 was specified and never built.** Wolf confirmed it and said to start over.

That removes the thing r16 wanted to A/B against, so this was built as **one generator
with a `FACET` switch**: off emits plain boxes and *is* the r15 baseline, on cuts the
corners. Same PARAMS, same rows, same camera and light. It is a tighter A/B than the
brief asked for — the two figures cannot differ by anything but the wedge — and it fills
the r15 gap for the same work.

## 1. Reference files opened, and when (r15 §1)

All on 2026-09-15, before any geometry. `front.png` (the row-by-row outline, **the
geometry authority**) · `side-left.png` (every depth: pack taper, nose ramp, boot toe) ·
`gear.png` (pick arms **plainly angled** — r16 §3's claim holds) · `f100` (brow pair with
the bridge recessed, hair lobes) · `f088` (pack depth, boot toe cap) · `f140` (neck,
shoulder caps, a clear **third plane** on the lit tunic corner) · both READMEs and
`dwarf-model-sheet.md` as cross-checks.

`back.png`, `side-right.png` and `f084/f092/f104/f120/f164` were **not** opened. Saying so
rather than citing them is the point of r15 §1.

## 2. The extraction, which is the first deliverable (r15 §2)

`sheet_r16.py` reads both pinned crops at **the sheet's own resolution**; full dump in
`src-assets/blender/r16-profile.txt`. It found the figure at **rows 7–147, centre column
77.5** — the sheet's own 140 px = 1.00 H and its column 77, independently recovered.

Both traps in r15 §2 fired, and the first is subtle enough to record:

- **the furniture filter has to be in SOURCE pixels.** The first cut dropped ink runs
  thinner than 8 *image* px, but the crops are 5× blow-ups, so a 2-source-px dimension
  arrow is 10 px wide and sailed through. It put the ink bbox at column 0 and threw every
  column in the table off by 16.
- **the figure splits at the legs**, so the run through the centre column does not exist
  below row 119 — it read `(none)` for 28 rows until the profile came off the connected
  component instead.

Prop exclusion (r15 §3.2) falls out of the same operation that removes the furniture:
eroding by 2 px cuts the lantern's hook and the arrows, and **symmetrising the front by
the narrower half drops the pick head, the haft and the lantern**, because each widens
exactly one side of any row. No second rule, no hand-drawn mask.

The extraction agrees with the model sheet at every cross-checkable landmark — crown
steps 0.164/0.229/0.286/0.336 at rows 7/10/12/15, head-with-hair 0.336, ear-to-ear 0.383,
hem 0.207 at row 118, stance 0.398, boot sole depth 0.214. Where the table and the art
disagree, the art was taken, as instructed.

## 3. The metric (r16 §4) — reported, not padded to

```
rev | cage faces | distinct directions (canonical) | % off the six axes | overshoot F / S
r15 |        772 |            31  ( 8 of the 26)   |       10.5 %       |  +1.0 / +3.0 px
r16 |       1164 |            70  (17 of the 26)   |       24.8 %       |  +1.0 / +3.0 px
```

**A raw distinct-normal count is not comparable to r16 §4's "roughly 26", so both are
printed.** The 26 is the cube's 6 axes plus its 12 edge and 8 corner diagonals; every ramp
the *art* draws at its own angle — the nose across rows 31–43, the pack's back, the boot
toe — adds a one-off normal that inflates the raw figure. 53 of r16's 70 are those. r16
clears the expectation (≥ 24 directions, ≥ 15 % off-axis); the baseline does not.

**But the baseline row is the finding.** Round 14 measured 14 directions with 94.6 % of
faces on the six axes. The r15 baseline here — *plain axis-aligned boxes, not one wedge* —
already reads **31 directions and 10.5 % off-axis**, because r15's own §4.2 says build the
nose as a ramp, the pack as a taper, the crown as six steps rather than as blocks. **A
large part of the deficit r16 diagnoses was round 14 building the art's own ramps as
blocks, and r15's method fixes that without any stylistic departure at all.** The wedge
then roughly doubles what is left.

## 4. The envelope (r15 §3), and what the wedge cost: nothing

Body, both pinned views, worst overshoot per z-decile, tolerance 3 source px:

```
r15 front +1.0 px PASS    r15 side-left +3.0 px PASS
r16 front +1.0 px PASS    r16 side-left +3.0 px PASS
incl. arms, both revs: front +19.0 px FLAG, side +3.0 px   (r15 §6: expected, A-pose)
```

**The two revisions are identical to the tenth of a pixel**, the round's cleanest result,
and predicted rather than discovered: a **vertical** 45° corner chamfer cannot move either
orthographic silhouette, because the face at `x=x1` still exists between `y0+c` and
`y1-c`, so the extreme x is still reached — likewise for y. r15 §4.1 uses that trick for
its shape; r16 gets its normals free from the same fact.

That is also r16 §6's answer: the widest rows are untouched. **Horizontal** wedges do move
the outline, so they went only where the art already draws the ramp — the crown (front
rows 7–15 widen 11.5 → 23.5 px), the pack's back corner (side rows 35–37 and 86–94, one
column per row) and the boot toe (side rows 133–138). Those are fidelity, not the
departure r16 §3 owns up to. Measured cost: **2 px undershoot on front rows 7–8**, from
the crown's top wedge.

The envelope did its real drift-detection job once: the pack's back edge read **+4 px**
until the art was re-read and found to *hold* column 16 from row 38 to row 66 and then
**step** to 12 at row 67, rather than tapering across the span. No landmark check sees that.

**Unmeasurable, not passing (r15 §3):** the torso and the skirt. The posed arms cover them
in all views — our extraction confirms it, reading +0.271…+0.293 H at chest rows on
`side-left.png`, which is the forearm, not the chest. Those two alone come from the model
sheet's table, and they are the only numbers here that do.

## 5. The A/B — the artefact the round is judged on

`src-assets/renders/r16/ab-r15-vs-r16-lit.png`, grey, same camera, pose and light. Grey is
deliberate at Stage A: it isolates what the wedge does to the *shading* from what paint
would later do.

**My read, Wolf's eye decides:** r16 is better, on the large masses and only there. Boots,
skirt hem, sleeve caps, pack and crown each gain a mid-value plane between the lit front
and the shadowed side — the third plane `f140` shows on the reference's tunic corner,
which is the honest reason for the round. The beard's chamfer softens a stepped mass that
was reading well; that is the one place I would take the wedge back off.

**The first A/B failed, and that is the more useful half.** Faceting everything above
r16 §2's 30 mm floor wrecked the head: brow, nose, moustache, ears and hair lobes became
the slivers round 12 was criticised for, and — because a chamfer *shrinks* a mass at its
corners — parts that used to abut opened visible slits at the neck and around the face.
**The rule that works is not a size threshold.** Anything seating inside or against
another mass stays square whatever its size; free-standing masses get the wedge. That
dropped the raw direction count and made the figure better — §4 working as intended.

### 5.1 What the gate structurally could not see: the props

Opening the figure in Blender, rather than reading numbers off it, showed both props
**hanging in mid-air** 0.10 H outboard of the hands, with a pick head a third of its drawn
size. The gate was not at fault: r15 §3.2 *excludes* props from our silhouette, so no
number in this round was ever looking at them. They are the one part with no instrument on
it. Three fixes, all from `gear.png` as drawn: the hand is now **derived** from the
shoulder and the 40° arm rather than guessed; the pick is **1.04 H, longer than the dwarf
is tall** — the sheet gets away with that by carrying it diagonally, and hung vertically
the first placement put 0.43 H of haft through the floor, so it now stands butt-down with
the head above the crown; and the head span went to 0.54 of the length. Body envelope
after all three: unchanged, as it must be.

## 6. Traps wired in from the start (r15 §7)

§7.2 no box named by index · §7.3 no axis-aligned width probe touches the 40° arms — the
gate is a rendered silhouette, not an AABB · §7.4 block-centre sampling, so geometry
touching a pixel boundary cannot light the row above · §7.5 degenerate-box assert —
**it fired**, catching mirrored parts whose `x0`/`x1` arrived swapped · §7.6 normals
against each chunk's own centre, **0 inside-out** · §7.7 hair rearmost above the
shoulders, −0.157 against the skull's −0.129 · §7.8 all counting at the sheet's own
resolution. §7.1 and §7.9 belong to Stage B.

## 6. Stage B — paint

One material, one packed 512² atlas, `Closest`, backface culling, Specular IOR Level
0.5. **Ten families × four value steps = 40 cells**, the approved ten plus r3's
recorded value steps — no palette was re-derived (r16 §8), and per r15 §4.4 each base
is the LIGHT cell with the approved cell as a shadow step: hair on `#513C2B`, beard on
`#826145`, so the two masses do not both range down to near-black and merge.

Every face maps to the **centre** of one cell, so "crisp value steps, no gradients"
holds by construction. **The step comes from the polygon normal**, which is what makes
r16 worth anything: a chamfer face lands on a real intermediate value instead of
borrowing its neighbour's. A wedge that painted the same as the front would be invisible.

The face is an **island projected in world coordinates**, so the eyes sit on the
0.807 H eye line because the projection puts them there. Three bugs, all caught by
looking rather than by measuring:

- `sym_rect` never mirrored — for the −x side it recomputed the same +x rect, so only
  half of every feature was painted and the face came out asymmetric;
- the island **swallowed the beard.** The beard's front is proud of the face plane so it
  qualified, and everything below the island's z range clamped to its bottom row: a slab
  of face paint down the chest. The island is bounded in z now, and the beard takes an
  ordinary cell — same colour, none of the clamping;
- **the shipped texture was a full sRGB decode too dark.** The atlas was authored in
  linearised floats and Blender wrote that buffer straight out as 8-bit, so `#F3E2D2`
  shipped as `#E5C2A4` — *every* cell wrong. It passed `check_asset` (which bounds and
  counts a palette, it cannot know what the cells were meant to be) and it passed by eye
  in Blender, because the same wrong buffer was what Blender was rendering. The atlas is
  authored in bytes now and the image is **reloaded from the saved PNG** before render or
  pack, so what Blender shows and what ships are the same pixels. The GLB's palette is
  now exactly the 40 authored cells.

A canary colour fills unpainted atlas and **0 faces sample it**; it is then buried under
a palette cell so nothing outside the approved set ships. r15 §5's question is closed
from the other side too: the key had been tuned against the dark buffer, and once the
albedo was right the same light blew the mid-tones to pastel. At a comparable exposure
the approved palette reads as the reference's olive tunic and tan boots.

## 7. Stage C — rig and export

19 joints, **61 boxes bound, rigid, one joint per vertex at 1.0**. Weights are assigned
**by BOX, not by object** (r15 §8): the neck box lives inside the head part but belongs
to `neck`, the torso's waist box to `spine`, the sleeve's forearm box to `elbow`. The
generator splits every part that straddles a joint — leg into thigh and shin, torso into
chest and waist (new this stage; the two abut, so the silhouette is unchanged), sleeve
into cap, upper arm and forearm.

Which box gets which joint is read from **each box's own extents**, never its index
(r15 §7.2). That immediately repeated §7.3 in a new place: a plain z threshold put the
upper arm on `elbow`, because the 40° rotation swings its lowest corner below the
forearm's top. Measured *along the arm axis* it is unambiguous.

```
boxes per joint: beard=2 chest=5 elbow.L/R=1 foot.L/R=2 hand.L=8 hand.R=11 head=17
                 hip.L/R=1 hips=1 knee.L/R=1 neck=1 shoulder.L/R=2 spine=2
```

`root` carries no geometry, which is legal and still exported. Exporter verbatim:

```
triangles     1648  of 100000 budget
topology      n-gons 0  non-manifold edges 0  loose verts 0  loose edges 0
              degenerate faces 0  flipped winding 0  missing UV layer 0
slivers       24 of 1188 faces narrower than one source pixel (2.0%)
holes         0 boundary edges, 0 EXPOSED to the outside
inside-out    none
rig           joints 19  missing 0  unexpected 0  unweighted verts 0  soft-weighted verts 0
check_asset.py  EXIT 0   profile=painted-map
```

The **2.0 % sliver figure is the one number to watch**, since slivers are what round 12
was criticised for — it is what the chamfer costs, and it is small.

## 8. "We are lacking all the detail we achieved with 14" — measured

Wolf's read, checked against r14 itself rather than argued about. The committed r14
(`a391d28`) was pulled out of git and rendered under this round's identical camera and
light: `renders/r16/ab-r14-vs-r16-lit.png`.

```
        objects  cage faces  triangles  directions  palette cells
r14          21        1032       2064          14   (4 per family)
r16          17        1236       1712          70    51 shipped
```

**r16 is not carrying less geometry — it carries ~20 % more cage faces.** It was reading
flatter, and comparing the two side by side found the reason, which was a real defect in
the paint and not a shortage of boxes:

- **front-facing and side-facing polygons were landing on the same palette cell.** With
  four value steps there is no set of breaks that separates top / lit corner / front /
  lit side, so the figure painted flat whatever the geometry did — and worse, **the
  chamfers painted identical to the front, which made r16's entire third plane
  invisible.** Five steps separate them, and the chamfer now gets a cell of its own
  (`#6E8E7B` on the tunic, between the top's `#7DA18C` and the front's approved
  `#5F7A6A`). This is the single most important fix in the round: without it the wedges
  were costing faces and buying nothing on screen.
- **value came only from the normal, so every box of a part facing one way was one
  colour**, while the art varies by FEATURE. Added, each drawn in `front.png`: the boot
  cuff as a lighter band than the shaft, the boots dark rather than sandy (no trouser
  shows between hem and cuff in the art, so the legs paint as boot), the lighter band
  along the tunic hem, and the collar at the neckline.

**What r14 still has that r16 does not**, listed honestly: a stepped skirt flange with
fold lines, a hanging tab below the belt buckle, and three-band boots. Per r15 §9 those
are **exactly the ~600 triangles of "skirt folds, belt stitching and finger blocks" that
round 14 added because a triangle count was low, wrote citations to fit, and that were
found on checking the frames not to exist.** I have looked at `front.png` again and I
cannot cite them. So they are not added here: adding them would require writing the same
false citations both briefs exist to prevent. If they are wanted as a deliberate
stylistic extension they are cheap to add — but they would be labelled as invention, not
as fidelity, and that is Wolf's call rather than mine.

## 9. What moved on disk, and one thing that could not be run

`REV` in `export_dwarf.py` is bumped to `r16`, which makes its documented one-command
export read `SM_VoxelDwarf_Miner01.blend`. That file still held **r14, with uncommitted
changes**, so it was **backed up to the session scratchpad before being overwritten** and
r16 promoted into it — otherwise the repo is in a state where the documented command
fails. Say the word and it goes back. r16 also lives in `SM_VoxelDwarf_Miner01_r16.blend`,
and the two A/B figures in `r16_base.blend` / `r16_facet.blend`.

**`scripts/gate.sh` was NOT run green — it cannot run on this machine.** There is no Rust
toolchain here (`cargo` is absent, as is a system `python`), so it reports RED on every
step for want of tools rather than for anything in the tree. This round changed no Rust;
the work is Blender and Python under `src-assets/`. Flagging that rather than claiming a
gate I did not run.

Nothing is committed. Per CLAUDE.md §4 that waits on Wolf's explicit yes.

| cost row | value |
|---|---|
| `dev-art` | 1 round, all three stages, ~115 tool calls against r16 §7's budget of 100 |

**The budget was overrun by ~15 %**, and it is worth saying where it went rather than
rounding it off: round 15 not existing meant building its whole method from scratch
inside r16's budget, and the three paint bugs plus the props each cost a build. The
wedge itself — the round's actual subject — cost almost nothing.
