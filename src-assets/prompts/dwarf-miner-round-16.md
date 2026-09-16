# Round 16 — faceted voxel: spend the budget on DIRECTIONS, not faces

**This round is an experiment, and it may fail.** Round 15 is the safe path and is pushed; if this
one does not read better than r15 by Wolf's eye, we keep r15 and this brief is a dead end that cost
one round. Say so plainly in the report rather than defending it.

**Round 15's method is inherited whole and is not repeated here.** Extraction before geometry, the
per-row envelope, the nine measurement traps, the part list, the paint scheme, the rig contract, the
lit diagnostic. Read `dwarf-miner-round-15.md` first; this brief only states what changes.

---

## 1. Why — the measurement that motivates the round

Round 14's finished figure, measured on its own mesh:

```
cage faces          : 1032
distinct directions : 14
faces on the 6 axis-aligned directions: 976 of 1032 (94.6 %)
```

And across that round: **444 → 600 → 852 → 960 → 1032 cage faces, with the plane count at 14 every
single time.** The geometry more than doubled and added **zero** new surface directions. The only
non-axis directions in the whole figure are the arms, because they are rotated 40°.

That is the whole problem in one table. **An axis-aligned box has six possible normals however many
you stack.** More boxes buy silhouette steps and colour regions, and we are already at the sheet's
own resolution on the first. It is also exactly what the model sheet already records from r11: a
level-3 subdivision at 32,128 triangles produced **0.00 %** silhouette change, because subdivision
cannot add a direction.

So the LOD0 ceiling is not the constraint — *faces* were never the scarce thing. **Directions are.**

## 2. What changes: rotated corner wedges

The axis-aligned clause was lifted on 2026-09-12 — *"those boxes may also be rotated … the
axis-aligned clause is gone, flat shading is what stays."* Round 14 used that only on the arms. This
round uses it on the form.

Replace the hard vertical corner of each large mass with a **45° wedge**, rotated about Z. Where a
mass reads strongly from above or below — crown, boot toe, pack top — add a horizontal wedge too.
Flat shading throughout; no smooth normals, no bevel modifier, no subdivision.

A vertical wedge adds normals at (±0.707, ±0.707, 0); horizontal ones add (0, ±0.707, ±0.707) and
(±0.707, 0, ±0.707). **14 directions should become roughly 26.**

**Facet these:** torso, skirt, pack, head, boots, sleeve caps, beard.
**Leave square:** belt, buckle, straps, small interior detail, and anything under ~30 mm. A wedge on
a 20 mm box is a sliver, and slivers are what round 12 was criticised for.

## 3. The honest framing, which belongs in the report

**The reference is mostly axis-aligned.** Its body masses are blocks. Some of its gear is not — the
pickaxe head in `gear.png` is plainly angled, and the boot toe and pack corners show angled planes in
`f140`. So faceting the *gear* is fidelity; faceting the *body* is a **stylistic extension beyond the
reference**, chosen because the reference's own renderer has lighting we do not yet have and its
corners read softer than ours as a result.

Do not describe this as matching the reference more closely. It is a deliberate departure, and the
round stands or falls on whether it looks better, not on whether it measures closer.

## 4. The metric — reported, with an expectation, never a target to pad to

Print on every build:

```
cage faces, distinct directions, % of faces off the six axes
```

Expectation: **≥ 24 directions, ≥ 15 % of faces off the axes.** That is what the facets above produce
if they are applied; it is a *consequence*, not a goal.

**§9 of round 15 still has teeth and applies here:** no operation added because a metric moved, and
every added box cites a file and a row or frame. If the direction count is low, the answer is "which
mass is still a plain box", never "add wedges until the number rises". Round 14 added ~600 triangles
of skirt folds and finger blocks because a triangle count was low, then wrote reference citations to
fit; checking the frames afterwards, none of it existed.

## 5. The envelope gate

**Unchanged from round 15 §3, which carries the tolerance and the reasoning**: ≤ 3 source px on the
body passes, the per-decile map is a drift detector rather than a fidelity target, the front sheet is
symmetrised before comparing, the props come out of our silhouette, the arms are reported separately,
and the parts the art never exposes are listed rather than counted as passes.

One thing to re-read there before starting: a source pixel is **0.06 of a screen pixel** at the
distance dwarves are actually seen. Nothing in this round is worth spending builds on at that scale.

## 6. The wedge must not break what round 14 fixed

Faceting cuts the corner off a mass, so it makes things **narrower**, never wider. That means it
cannot push the silhouette out — but it can pull it in, and three checks are sensitive:

- a wedge at the widest row of the head, beard or skirt will reduce the measured width below the
  sheet. Facet **between** the sampled rows, not across them, or carry the full width on the
  unfaceted core.
- the bare-neck run is set by occluders; a wedge on the hair lobes changes where they sit relative to
  the neck's y window. Re-measure it, do not assume.
- the face island must still cover every box whose front stands proud of the face plane, wedges
  included, or the paint behind them is covered — that is how round 14 lost the brows entirely.

## 7. Stages, budget, deliverables

Stages and deliverables are round 15's, with these additions:

- the direction count printed per build, and in the report
- **an A/B against round 15**: same camera, same pose, same lighting, r15 and r16 side by side, lit.
  This is the artefact the round is judged on
- `dwarf_r16.py`, `render_r16.py`, `REV = "r16"`

**Budget ≤ 100 tool calls.** The method is inherited; only the wedges are new. If the direction count
is up and it does not look better, stop early and say so — that is a complete and useful result.

## 8. Not in this round

Animation, LODs, the plane-of-form metric, re-deriving the palette from the video, smooth shading,
bevel modifiers, subdivision, and carrying any geometry across from r14 or r15.
