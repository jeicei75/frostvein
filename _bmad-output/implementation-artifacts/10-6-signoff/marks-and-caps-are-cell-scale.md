# The big tiles on top of everything: they are the snow caps

**Wolf's guess was right.** The large pale-blue slabs lying on top of everything, which cannot be
dug, are `SnowCap` entities.

| | |
|---|---|
| Colour | `snow_cap_color()` = srgb_u8(146, 158, 184) [appearance.rs:238] |
| Geometry | `Cuboid::new(1.02, 0.08, 1.02)`, placed at the cell top + `Y * 0.54` |
| Count | **8,145** on this world — one per exposed non-ice, non-soil, non-foliage top |
| Component | `SnowCap` + **`ClientLocal`** [project.rs:1448] |

**Why they cannot be dug: they are not tiles.** They are presentation-only client-local entities —
the wire terrain keeps its original material and knows nothing about them. Picking raycasts the
**mirror** [pick.rs:55], which contains no cap, so a cap is invisible to the cursor and you always
designate the tile underneath it. There is nothing there to dig.

Proved by rendering the same frame with `has_snow_cap` forced false: the plates vanish and the
sub-cell detail is visible across the whole surface. Entities fall 14,527 → 6,382, exactly the
8,145 caps. See `caps-a-subdiv4-with-snow-caps.png` and `caps-b-subdiv4-without-snow-caps.png`,
same camera, same world.

## Why they only started looking wrong at k > 1

Nothing about the caps changed. What changed is everything around them.

A cap is **cell-scale decoration on what is now a sub-cell surface**. At `--subdiv 1` a cap and a
terrain cube are the same size, so a cap reads as snow lying on a tile. At k=4 the terrain carries
sixteen sub-cell columns per cell while the cap is still one flat slab covering 102% of the cell,
so it reads as a plate dropped on top of the landscape — and it **hides the detail we are paying
for**.

That is a measurable cost, not just a look complaint: **8,145 of the 45,584 meshed cells — 17.9%
of the fine terrain surface — sit under an opaque cell-scale slab.** Roughly a fifth of the k=4
triangle budget buys geometry no one can see, and the fraction is the same at every k.

## The other coloured slabs, while we are here

Two more cell-scale slabs appear in the same frames, both `Cuboid::new(1.02, 0.08, 1.02)` scaled
by `MARK_FOOTPRINT_SCALE = 0.94`. Sampled from `Screenshot 2026-09-01 103715.png`:

| In the frame | Sampled | Palette base | What it is |
|---|---|---|---|
| saturated blue | (56, 105, 198) | (56, 132, 250) | `DesignationKind::Dig` mark |
| violet | (95, 83, 182) | (150, 96, 230) | `DesignationKind::Channel` mark |

These are the ones you place yourself, so they are not the "on top of everything" ones — but they
have the same shape of problem.

## And one real "cannot be dug" that is NOT a cap

`Tile::Ramp` is silently rejected by dig designation: the filter accepts `Tile::Solid(_)` only
[sim-core/src/lib.rs:1344]. Ramps are drawn exactly like solids — same cube, same material, no
distinguishing mark anywhere in the client — so a ramp is a tile that looks ordinary and refuses
to be designated with no feedback. This world has **2,553 ice ramps and 2,534 snow ramps**. If the
tiles that will not dig are sometimes ordinary-looking terrain rather than the pale plates, that
is why. Not this story's to fix; recorded because it is invisible from the client by construction.

## The general point for 10.4

Subdividing terrain silently orphans **every** cell-scale thing drawn on it: snow caps,
designation marks, zone overlays, the hover slab, dig chips. Each keeps a resolution the terrain no
longer has. This is a real cost of adopting k > 1 that no triangle count shows, and it lands on
whoever owns the look.
