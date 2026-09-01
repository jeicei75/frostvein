# Why the big blue tiles look wrong at k=4

**They are dig and channel designation marks, and they are not new.**

Sampled from `Screenshot 2026-09-01 103715.png` against the palette in
[crates/gui/src/appearance.rs:122]:

| In the frame | Sampled | Palette base | What it is |
|---|---|---|---|
| saturated blue slab | (56, 105, 198) | (56, 132, 250) | `DesignationKind::Dig` mark |
| violet slab | (95, 83, 182) | (150, 96, 230) | `DesignationKind::Channel` mark |

The difference is night lighting; the hues are unmistakable. Both are drawn as
`Cuboid::new(1.02, 0.08, 1.02)` scaled by `MARK_FOOTPRINT_SCALE = 0.94`, so each is a solid opaque
slab covering 95.9% of its tile, laid flat on the surface.

## Why they suddenly read as wrong

Nothing about the marks changed. What changed is everything around them.

A mark is **cell-scale UI on what is now a sub-cell surface**. At `--subdiv 1` a mark slab and a
terrain cube are the same size, so a mark reads as a marked tile. At k=4 the terrain carries
sixteen sub-cell columns per cell and the mark is still one flat slab over the whole cell, so it
reads as a foreign plate dropped on top of the landscape. The same is true of the 8,145 snow caps,
which are the pale plates in the earlier screenshots.

This is worth naming because it is a **general consequence of subdividing terrain and nothing
else**: every cell-scale decoration the client draws — designation marks, snow caps, zone
overlays, the hover slab, dig chips — keeps a resolution the terrain no longer has. It is not a
defect in this story's mesher and it is not fixed here. It belongs to whoever owns the look once a
subdivision is adopted (10.4), and it is a real cost of adopting k > 1 that no triangle count
shows.

## One thing checked and cleared

A cap or mark pinned to the coarse cell top could hover over a pitted surface. Measured on the
real world: at k=4 only **216 of 8,145** capped cells (2.7%) have every one of their sixteen fine
columns pitted, which is what it takes for the cap to lose contact; at k=8 and k=16 it is zero. At
k=2 it is 3,410 of 8,145 (41.9%) with a half-cell gap, so **k=2 would need this looked at** and
k=4 and finer would not.
