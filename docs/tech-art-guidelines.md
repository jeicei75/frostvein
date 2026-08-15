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

## Sky and lights

The sky is an illuminant. Cold ambient fill and a green-blue directional light let
the aurora catch snow and ice; three translucent 56×6 horizon bands sit beyond the far
terrain edge and immediately above the skyline. The stars scatter across at least 65% of
the boot sky's width and 25% of its height, rather than forming one horizon row. Their centres
are pinned inside the boot camera frustum, so the visible sky carries the aurora rather than
placing it overhead or out of frame. Torch, campfire, and future lantern properties are one
data table containing colour, lumen intensity, and range.
At the default camera exposure, the 2,000 cd/m² ambient fill and 1,500 lux directional
light lift blue-grey snow to a midtone; the 2.5M lm torches and 6M lm campfire make warm
local pools read above that field. The resulting ladder is dark sky and flanks, midtone
snow and ice, then warm pools and near-white emitter faces. Every entry is warm (red
exceeds blue); the table is static until story 6.1 adds flicker.

## Boot framing

The boot camera uses yaw 0.70, pitch 0.45 radians, and distance 90. It looks through a
composition offset toward the far valley: the camp projects to 78% of the frame height and
the far skyline to 30%, each within a 2% tolerance. The offset scales down below boot distance
so the 4-unit close zoom keeps the camp in front of the camera; it is fully applied from the
boot distance through the 500-unit vista. The offset is necessary because a camera that directly
looks at the camp would always centre it and could not produce the approved foreground composition.

## Edge treatment

Distance fog to the night-sky colour is the current candidate: its range follows the
camera distance so the far valley survives the whole zoom clamp while preserving depth.
It is pending the Task 6 vehicle comparison against per-rim material darkening; no final
edge choice has been made in the devpod.
