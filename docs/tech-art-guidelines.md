# Tech-art guidelines

## Value and materials

The boot frame is a night scene. Snow is deliberately midtone blue-grey rather than
white; ice is visibly bluer, and stone, soil, trunks, and foliage remain dark and
desaturated. Near-white is reserved for emissive sources and stars. These choices live
in `gui`'s material table, not beside draw calls.

Snow is a client-side settled cap: an exposed solid top receives the snow material,
while covered solids retain their bare flank material. Foliage is treated as a solid
top, so its branches read as loaded rather than uniformly coated terrain.

## Sky and lights

The sky is an illuminant. Cold ambient fill and a low green-blue directional light let
the aurora catch snow and ice; the horizon bands are emissive geometry behind the
skyline, never an overhead backdrop. Torch, campfire, and future lantern properties are
one data table containing colour, lumen intensity, and range. Every entry is warm
(red exceeds blue); the table is static until story 6.1 adds flicker.

## Edge treatment

The chosen edge treatment is distance fog to the night-sky colour, starting beyond the
camp read and becoming opaque at the far valley. It was selected over per-rim material
darkening because fog preserves the material table's value discipline and also supplies
the required depth separation. The vehicle review must confirm this against the approved
artifact; it cannot be judged in the devpod.
