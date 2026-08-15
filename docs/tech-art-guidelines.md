# Tech-art guidelines

## Value and materials

The boot frame is a night scene. Snow is deliberately midtone blue-grey rather than
white; ice keeps its blue top, while stone, soil, trunks, and foliage remain dark and
desaturated. Near-white is reserved for stars and emitter faces. These choices live in
`gui`'s material table, not beside draw calls.

Snow is a client-side settled cap: exposed snow, stone, soil, and foliage tops receive
it, while ice keeps its blue surface and covered terrain retains its bare flank. Ramps
follow those same material rules because the renderer presents them as full cubes.

## Sky and lights

The sky is an illuminant. Low cold ambient fill and a green-blue directional light let
the aurora catch snow and ice; the translucent horizon bands sit beyond the far terrain
edge and above the skyline, never inside terrain or overhead. Torch, campfire, and future
lantern properties are one data table containing colour, lumen intensity, and range.
The camp lights use hundreds of thousands of lumens against an 8 cd/m² + 8 lux cold fill,
so their pools dominate without lifting the night field. Every entry is warm (red exceeds
blue); the table is static until story 6.1 adds flicker.

## Edge treatment

Distance fog to the night-sky colour is the current candidate: its range follows the
camera distance so the far valley survives the whole zoom clamp while preserving depth.
It is pending the Task 6 vehicle comparison against per-rim material darkening; no final
edge choice has been made in the devpod.
