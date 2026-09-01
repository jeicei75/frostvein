# Resolution decision for 10.3

Adopt **1.6 metres per simulation cell**, **0.4 metres per terrain visual voxel**, and **visual
subdivision k=4**. Keep the simulation grid at k=1.

**The k=4 number 10.3 builds against is 928,884 triangles**, which is what `gui --subdiv 4`
submits on the real exported world. The offline bench predicts 920,502 for the same scene; the
+0.91% is the client partitioning masks by chunk and rim before merging, and is fully accounted
for in [axis-a-geometry.md].

| At k=4 | Value | Label |
|---|---:|---|
| Triangles submitted by the renderer | 928,884 | measured, live |
| Triangles predicted offline | 920,502 | measured, offline, client-parity |
| Greedy quads | 460,251 | measured, offline |
| Meshed coarse cells / chunks | 45,584 / 121 | measured, both agree exactly |
| Render entities | 14,527 | measured, live |
| Mesh build (debug, lavapipe) | 2,477 ms | measured — a build time, never a frame time |

## What changed since the 2026-08-31 version, and why

That version adopted k=4 at **997,428 triangles**, and the renderer submitted **1,527,754** —
53% more than the budget 10.4 and 10.5 would have authored against. Three independent defects,
each moving the number, all invisible to a k=1 control:

- The client meshed against the drawn set rather than solidity, so 44% of its submitted faces
  were sealed inside rock.
- The bench derived exposed faces as `coarse_faces × k²`, which no pit obeys.
- The `--no-detail` control that validated the whole k>1 dataset could not fail.

Both sides are now measured, and they agree on the cell set and the chunk count to the unit.

## The cost that should decide this is not fps

Every number above describes a **static** scene. At `--subdiv N > 1` any terrain change re-meshes
the entire world, so one dug tile costs 540 ms at k=2, ~2,500 ms at k=4, 10,386 ms at k=8 and
44,740 ms at k=16 on the devpod — the "Mesh build" column is the per-dig price, not a boot cost.
`--subdiv 1` does not do this. A fortress digs constantly, so this and not the frame rate is
likely to be what decides the adopted k, and no measurement in this story existed for it until
Wolf reported the client stopping dead on a dig.

**Fixed 2026-09-01 on Wolf's second report.** `reconcile` rebuilds only the chunks a changed cell
can reach. Measured live at k=4 on a real dig: 1 chunk instead of 121, 13,554 triangles instead of
928,884, **55 ms instead of ~2,500**. The saving grows with k (10.1× at k=2, 20.5× at k=4, 26.6× at
k=8 offline), because the cost is one chunk either way — which also means the per-dig cost is no
longer a reason to prefer a coarser k. Detail and the two safety proofs in [axis-a-geometry.md].

## k=8 is now a live candidate, not an excluded one

The previous exclusion rested on the bench's own guard failing at k=8, on a shared host, with the
stress test deferred. That guard was mis-sized: k=8 completes offline in 7.9 s, and
`gui --subdiv 8` builds **3,539,074 triangles** on this devpod. So the reason given for excluding
k=8 was never evidence, and it is withdrawn rather than carried forward.

**No fps reading exists yet for either k.** Both runs so far measured a scene whose terrain was
entirely back-face culled — the chunk mesher wound every quad against its own normal — so ~140 fps
described snow caps and tree cubes over a void. Fixed 2026-09-01; see [vehicle-fps.md] and the
three captures beside it.

What is still owed is the only thing that can settle it: a vehicle fps reading. There is no measured headroom to reason from, so k=8 is
neither excluded nor assumed servable: it is a row on the card.

**k=4 is the number to build against today**, on geometry grounds. If the vehicle reading clears k=8 at the NFR6 bars,
the adopted k is Wolf's to revisit, and 10.3's contract should be written so that revisiting it is
a change of one constant.

This deliberately does **not** settle the reference sheet's 16 voxels/cell target. It gives 10.3 a
served terrain number now, while trees and five dwarves stay much finer on their separate asset
budgets.
