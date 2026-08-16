# Tech-art guidelines

## Value and materials

The boot frame is a night scene. Stone is `(60, 70, 92)`, soil `(56, 52, 62)`, ice
`(104, 128, 170)`, and terrain snow `(136, 150, 178)`: the approved dark-but-readable
night palette. A settled snow cap is its own brighter `(158, 170, 196)` material, so caps
read over snow terrain without becoming white; near-white remains reserved for stars and
emitter faces. These choices live in `gui`'s appearance tables, not beside draw calls.

Snow is a client-side settled cap: exposed snow, stone, and soil tops receive the distinct
cap material, while
ice keeps its blue surface and covered terrain retains its bare flank. Foliage receives
no terrain-style snow slab: its dark, broken silhouette leaves the valley's snow landform
visible instead of turning every ground-level tree skirt into a bright tile. Foliage cubes
taper by their contiguous foliage above: ground skirts and crown tips scale to 0.72, the
upper crown to 0.86, and the mid-crown stays full scale. Ramps follow the same material
rules because the renderer presents them as full cubes.

The **exposed crown** of a spruce — foliage with nothing solid above it — takes its own
`(172, 186, 210)` snow-laden material, matching the approved artifact's `SPRUCE_SNOW`. This
is a material swap and deliberately not a terrain cap: capping foliage puts a bright slab on
every ground-level skirt tile and buries the landform, which is what the round-3 capture
showed. Bare cube foliage without it reads as a dark clump in a lit field.

## Sky and lights

The sky is an illuminant. Cold ambient fill and a green-blue directional light let the
aurora catch snow and ice.

The aurora is a **curtain on a ring** around the world (radius 220), not a set of billboards:
the camera orbits, so any flat quad turns edge-on. Its shape comes entirely from a procedurally
generated RGBA gradient — the table's aurora colour throughout, with alpha forced to exactly
zero at the top and bottom edges of the strip by a `sin(pi v)` term, so the curtain has no
silhouette. Low-frequency folds across the ring stop it reading as a ribbon. Its top (45) stays
below the boot camera's eye height (55): that is what "hugs the horizon rather than hanging
overhead" means in geometry. Three opaque cuboids failed this by mechanism, not by placement —
`unlit` plus `AlphaMode::Blend` on a `Cuboid` can only produce a flat rectangle with hard edges.

Stars sit on a shell at radius 250, scattered by golden-angle azimuth over a deliberately narrow
height band. The band is narrow because this camera looks down: the visible sky is a thin wedge
above the ridge line, and a full dome would put most stars outside every frame. Sizes vary so
the shell does not read as a lattice. Sky materials set `fog_enabled: false` — they sit far
outside the fog volume and would otherwise be erased with the far terrain.

Torch, campfire, and future lantern properties are one data table containing colour, lumen
intensity, and range. **The light budget is set from measurement, not estimate.** The round-4
capture's valley floor read a median sRGB luminance of 22.5 where the approved artifact reads
112.6 — a factor of ~18 in linear light. Ambient is 30,000 and directional 60,000, with the
directional share raised from 11% to ~24% of the total so the field keeps its modelling instead
of flattening under pure ambient; torches are 30M lm and the campfire 72M lm.

**Warm against cold is carried by hue, not by a large luminance ratio.** Measured on the
approved artifact, the camp is only ~1.3x the field in luminance (135.9 vs 104.3) while its
R/B goes from 0.72 to 0.97. The contrast oracle is therefore a *band* (campfire-to-cold-fill
between 1.2x and 6.0x) plus a chromatic term (every light's R/B at least twice the ambient's).
A bare 3x floor with no ceiling was satisfied both by the 1/1000-scale table that shipped black
and by a camp blown to white. The ladder is dark sky and flanks, midtone snow and ice, then warm
pools and near-white emitter faces. The table is static until story 6.1 adds flicker.

## Boot framing

The boot camera uses yaw 0.70, pitch 0.45 radians, and distance 90. It looks through a
composition offset toward the far valley: the camp projects to 48% of the frame width and 78%
of its height, and the far skyline to 30%, each within a 3% tolerance. The offset scales down
below boot distance so the 4-unit close zoom keeps the camp in front of the camera; it is fully
applied from the boot distance through the 500-unit vista. The offset is necessary because a
camera that directly looks at the camp would always centre it and could not produce the approved
foreground composition.

**The offset must lie in the camera's view plane** — along the view direction and straight up,
never along the camera's right vector. A push expressed in world axes (the original
`(0, 6.7, -37.42)`) carries a 28.6-unit component along `right` at this yaw and slid the camp to
23% of the frame width while every vertical assertion stayed green.

## Edge treatment

Two mechanisms are in the build, and they do different jobs.

**Distance fog is aerial perspective, not the edge treatment.** Its range follows the camera
distance so the far valley survives the whole zoom clamp: at the boot framing it opens at 75
(just past the camp at depth 71) and saturates at 155 (just past the deepest in-frame terrain
at 148).

Fog alone *cannot* dissolve this world's edge, and that is now measured rather than assumed.
At the boot framing the entire visible skyline **is** the map boundary — the silhouette against
the sky runs from depth 86 to depth 145 while the camp sits at 71. Fog tight enough to hide the
nearest silhouette point would also erase the valley the frame exists to show. This is the
"fog skirt" candidate walked to the end and found wanting, by arithmetic on the real framing.

**The world edge is dissolved in world space instead.** `rim_level` blends terrain toward the
sky colour over the outermost 10 tiles in five quantised steps, so the boundary fades out at
every zoom and camera angle rather than at one tuned distance. Five shared steps per surface
keep it to a handful of material handles; per-tile blending would mean one material per cube.
The tiles are still drawn — the draw set is pinned by the 53,365-cube oracle and must never
change to hide an edge.

Per AC11's amendment the final choice is still Wolf's, at the vehicle. What is settled here is
that fog-alone has been eliminated on evidence.

## The value floor

`--capture` range-checks the median sRGB luminance of the valley floor (frame centre, x 0.25 to
0.75 and y 0.50 to 0.90) against a floor of 70. The approved artifact reads 123 in that window;
the round-4 capture read 21. No headless test can see a black frame, so the instrument carries
AC9's value discipline. Median rather than mean, so a handful of blown-out emitter faces cannot
carry a black field over the floor.
