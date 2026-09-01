# Resolution decision for 10.3

> **ADOPTED, RULED BY WOLF 2026-09-01: k = 4.** This is the number 10.3 copies.
>
> **The ruling now has a better reason than the one it was made for.** It was taken when no valid
> fps reading existed. One does now (gingerspice, build `caa8689-dirty`): **k=4 is >130 fps and
> k=8 is 100–140 fps, both clearing NFR6's 60 and 30 bars comfortably.** So frame rate does NOT
> decide between them. What does is the cost of a dig, measured live on the same run: **5–13 ms at
> k=4 against 38–78 ms at k=8** — one frame versus a 3–5 frame hitch, on every dig, in a game that
> digs constantly. k=4 is adopted for dig smoothness, not for triangles.
>
> The earlier reopening in favour of k=16 stays WITHDRAWN: the 60–90 fps that motivated it was
> read on a build predating `bace455` and `c8675fc` and is void. k=16 remains proven only as
> GEOMETRY. See [vehicle-fps.md].

Adopt **1.6 metres per simulation cell**, **0.4 metres per terrain visual voxel**, and **visual
subdivision k=4**. Keep the simulation grid at k=1.

**The k=4 number 10.3 builds against is 928,884 triangles**, which is what `gui --subdiv 4`
submits on the real exported world. The offline bench predicts 920,502 for the same scene; the
+0.91% is the client partitioning masks by chunk and rim before merging, and is fully accounted
for in [axis-a-geometry.md].

| At k=4 | Value | Label |
|---|---:|---|
| Triangles submitted by the renderer | 928,884 | measured, live — CHUNK MESHES ONLY, see note |
| Triangles predicted offline | 920,502 | measured, offline, client-parity |
| Greedy quads | 460,251 | measured, offline |
| Meshed coarse cells / chunks | 45,584 / 121 | measured, both agree exactly |
| Render entities | 6,826 | measured, live (was 14,527 before `bace455` painted the snow) |
| Mesh build (debug, lavapipe) | 2,477 ms | measured — a build time, never a frame time |

**What the triangle figure does and does not include.** 928,884 counts the CHUNK MESHES. It
excludes the 4,501 tree-foliage cube entities the client still draws per cell (~54k triangles).
The exclusion is conservative — the real scene is slightly heavier than the number — but a
contract that reads "928,884 triangles is the cost of k=4 terrain" is reading terrain only, which
is the right thing for an asset contract and the wrong thing for a frame budget.

## What changed since the 2026-08-31 version, and why

That version adopted k=4 at **997,428 triangles**, and the renderer submitted **1,527,754** —
53% more than the budget 10.4 and 10.5 would have authored against. Three independent defects,
each moving the number, all invisible to a k=1 control:

- The client meshed against the drawn set rather than solidity, so 44% of its submitted faces
  were sealed inside rock.
- The bench derived exposed faces as `coarse_faces × k²`, which no pit obeys.
- The `--no-detail` control that validated the whole k>1 dataset could not fail.

Both sides are now measured, and they agree on the cell set and the chunk count to the unit.

## The budget is a RANGE, and the placeholder sets its ceiling

**k=4 terrain is 80,120 to 928,884 triangles.** The 928,884 figure is what k=4 costs when the
sub-cell surface is uncorrelated noise, which is exactly what this story's measurement stand-in
is. Sampled coherently over a whole cell the same rule costs **80,120** — an 11.5× spread on one
property of a placeholder that was never meant to be a look. Detail accounts for 96.8% of the
committed figure. See "How much of the k=4 budget is the placeholder?" in [axis-a-geometry.md].

**The two ends are not two points on one curve, and both labels matter.** 928,884 is LIVE
(the renderer); 80,120 is OFFLINE (the bench, client-parity), and its live twin would be about
+0.9% by the same partitioning gap as everywhere else. They also mesh slightly different surfaces
— 45,584 cells at the noisy end against 44,828 at the coherent one, because coherent detail
carves fewer neighbours open. Read the range as a BRACKET on the same rule sampled two ways, not
as a measured curve.

**10.3 should carry the range and the reason, not the single number.** Where inside it the real
budget lands is decided by 10.4's authored terrain, and an asset contract that fixes 928,884 as
"the cost of k=4" would be pricing a placeholder. If a single number is needed to start work, use
928,884 as a ceiling and expect to reclaim most of it.

**The same caveat is WIDER at k=16, and it is why k=16 cannot be adopted on geometry alone.**
k=16 spans **80,754 to 13,849,802 triangles — a 172× spread**, against k=4's 11.5×, and the only
k=16 figure anyone has quoted (13,873,064) is the noisy end. The placeholder's contribution grows
with k, so the finer the subdivision the less a placeholder-derived budget means.

## The cost that should decide this is not fps

Every number above describes a **static** scene. As first built, `--subdiv N > 1` re-meshed the
ENTIRE world on any terrain change, so one dug tile cost 540 ms at k=2, ~2,500 ms at k=4,
10,386 ms at k=8 and 44,740 ms at k=16 on the devpod. `--subdiv 1` never did this. A fortress digs
constantly, so this and not the frame rate was likely to decide the adopted k, and no measurement
in this story existed for it until Wolf reported the client stopping dead on a dig.

**Fixed 2026-09-01 on Wolf's second report.** `reconcile` rebuilds only the chunks a changed cell
can reach. Measured live at k=4 on a real dig: 1 chunk instead of 121, 13,554 triangles instead of
928,884, **55 ms instead of ~2,500**. *(Corrected in the round-2 review: the reach is TWO cells,
not one — a dug cell changes whether its neighbours are drawn, and a newly drawn neighbour emits
faces into ITS neighbours' chunks. The one-step version left stale chunks standing near a seam.
A dig away from a chunk boundary still rebuilds 1 chunk; one near a seam rebuilds a handful,
which does not change the order of magnitude of the saving.)* The saving grows with k (10.1× at k=2, 20.5× at k=4, 26.6× at
k=8 offline), because the cost is one chunk either way — which also means the per-dig cost is no
longer a reason to prefer a coarser k. Detail and the two safety proofs in [axis-a-geometry.md].

## k=8 is now a live candidate, not an excluded one

The previous exclusion rested on the bench's own guard failing at k=8, on a shared host, with the
stress test deferred. That guard was mis-sized: k=8 completes offline in 7.9 s, and
`gui --subdiv 8` builds **3,539,074 triangles** on this devpod. So the reason given for excluding
k=8 was never evidence, and it is withdrawn rather than carried forward.

**A valid reading now exists — but three earlier ones did not, and that is the durable lesson.**
The 2026-09-01 run on build `caa8689-dirty` reads >130 fps at k=4 and 100–140 at k=8, on a client
that draws its terrain and carries painted snow and the incremental rebuild. Treat the margin, not
the absolute number, as the result: k=8 submits 3.8x the geometry and its range overlaps k=4's, so
both are near the panel rather than near the GPU's limit.

**The three earlier readings were void, each for a different reason.** The first pair of
readings measured a scene whose terrain was entirely back-face culled — the chunk mesher wound
every quad against its own normal — so ~140 fps described snow caps and tree cubes over a void
(fixed 2026-09-01). The k=16 reading that replaced it predates `bace455` and `c8675fc` and is
void in turn. A vehicle reading must from now on record the commit it was taken at; see the
command card in [vehicle-fps.md].

**k=8 is servable on frame rate and is not adopted anyway.** It clears both NFR6 bars at a 100 fps
floor, so nothing about the renderer excludes it; what excludes it is the 38–78 ms per-dig hitch.
If a later story makes the rebuild cheaper — per-chunk k, a smaller chunk edge, or meshing off the
main thread — k=8 becomes a live option again on this evidence. k=16 stays a geometry result only,
with no valid reading. 10.3's contract should be written so that revisiting the adopted k is a
change of one constant.

This deliberately does **not** settle the reference sheet's 16 voxels/cell target. It gives 10.3 a
served terrain number now, while trees and five dwarves stay much finer on their separate asset
budgets.
