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
no terrain-style snow slab: its broken silhouette leaves the valley's snow landform
visible instead of turning every ground-level tree skirt into a bright tile. **Foliage is green
`(44,100,58)` since story 9.4** — it was `(55,73,84)`, only 9.9 from stone on the same measure the
marks are held to at 40, so the base cubes were near-camouflage against the ground. Trees separate
on GREEN, the axis the cool directional does not compress; every terrain material still keeps blue
at or above red. Foliage cubes
taper by their contiguous foliage above: ground skirts and crown tips scale to 0.72, the
upper crown to 0.86, and the mid-crown stays full scale. Ramps follow the same material
rules because the renderer presents them as full cubes.

The **exposed crown** of a spruce — foliage with nothing solid above it — takes its own
`(156, 170, 196)` snow-laden material. That started as the approved artifact's `SPRUCE_SNOW`
`(172, 186, 210)` and was trimmed at round 7: the artifact shows that colour on thin sprite tops
while our cubes show whole faces of it, and every tree glowing at near-cap brightness is what made
the boot4 foreground read as clutter. This
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
intensity, and range. **The light budget is set from measurement, not estimate — and it took
two rounds of measurement to converge.** The round-4 capture's valley floor read a median sRGB
luminance of 22.5 against the artifact's 112.6 (~18x short in linear light); scaling up by that
factor then measured 156 on the boot3 vehicle capture — 26% over — with shadows flooded (p05 87
vs the artifact's 28) and a saturated blue-green cast. The overshoot taught the real rule:
**the budget divides, it doesn't just scale.** A small desaturated ambient (`(120,140,165)` at
4,500) lets shadow faces go genuinely dark; a desaturated cool directional (`(150,190,180)` at
22,000) carries the lit faces. Both tints sit near neutral because light colour MULTIPLIES onto
already-blue materials — the boot3 cast came from lighting blue snow with saturated blue-green
lights, not from the material table. Torches are 14M lm and the campfire 25M lm (35M at its
1.40 flicker peak): the white-clip radius scales as sqrt(intensity), and 72M blew a ~9-tile pool
to flat white where AC9 reserves white for emissive faces alone.

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
of its height, and the far skyline to 24%, each within a 3% tolerance. The offset scales down
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
The tiles are still drawn — the draw set is watched by the cube oracle and must never shrink to
hide an edge. **The oracle is a measurement of the shipped world, not a constant.** It reads
**44,984** exposed cubes of 301,048 solid today. Story 9.4 moved it twice in one story: 53,365 of
315,068 before it, 45,261 after the tree-density cut, 44,984 after the ground-level foliage ring was
removed. It moves whenever world content moves; what must not change is that the rim dissolves by
colour alone and removes no tiles.

Per AC11's amendment the final choice is still Wolf's, at the vehicle. What is settled here is
that fog-alone has been eliminated on evidence.

**The fog colour and the rim's target colour must both be exactly the sky colour, and that is a
consequence of the sky being a flat `ClearColor`.** A lighter "horizon haze" would be more true
to a real aurora-lit night — and the approved artifact has one — but with a uniform sky it would
make far terrain *lighter* than the sky behind it and hand the world its edge straight back. A
haze colour only becomes available once the sky itself carries a vertical gradient; until then
these three colours move together or not at all.

## Keeping the sky outside the camera

Sky geometry is hung on rings around the world, so its radius is not a free choice: the camera
must stay **inside** it at every zoom. At the 500-unit clamp the camera orbits 426 units from the
world centre, so the 220-unit ring the curtain was first built on would have put the camera
outside it and swung the aurora across the front of the valley at full vista — the exact register
AC10 says must carry sky and aurora. The curtain sits at 600 and the star shell at 650, and a
test pins the camera's furthest excursion below both. Sizes and height bands are scaled to that
radius, and the "hugs the horizon" rule is asserted as an ANGLE from the boot eye (<= 10 degrees),
because a raw height threshold stops meaning anything the moment the radius changes.

## The value floor

`--capture` range-checks the median sRGB luminance of the valley floor (frame centre, x 0.25 to
0.75 and y 0.50 to 0.90) against a floor of 70 AND a ceiling of 180. The approved artifact reads
123 in that window; the round-4 capture read 21 (black field) and the boot3 capture read 156
(washed toward white) — each end of the band has now caught a real failure. No headless test can
see either, so the instrument carries AC9's value discipline in both directions. Median rather
than mean, so a handful of blown-out emitter faces cannot carry a black field over the floor.

Sky scatter uses low-discrepancy pairs with INDEPENDENT irrationals per axis (the R2/R3
sequences). Deriving two axes from the same constant correlates them perfectly — the first star
shell used the golden ratio for azimuth and height and all 300 stars landed on one helix, which
the vehicle showed as dotted lines across the sky. The same rule covers snowfall: scattered
disc positions, per-flake fall speeds and phase-preserving respawn, because a shared speed or a
shared respawn height re-synchronizes the field into marching rows.

## Motion, flicker, and work evidence

Motion is presentation only: dynamic entities blend between the previous and current positions
the wire delivered, clamped at the current position with no prediction or extrapolation. Torch
and campfire point lights breathe inside their table-defined bands from a deterministic function
of simulation id and client elapsed time; the emitter material remains static. The campfire — and
only the campfire — casts point-light shadows, so its light no longer passes through solid
terrain; each additional shadow-casting emitter would cost six cube-map faces against NFR6. When an `Empty`
tile arrives, four deterministic, client-local stone chips sit at the position until a snapshot
rebuild clears them. They make digging read as work without inventing simulation state.

## Moving lights

A moving light uses the same `LightKind` appearance-table lookup as a static one. A dwarf is not
special-cased warm: when its wire entity carries a light, reconciliation attaches that table-driven
point light to the dwarf's blended projection, so the pool moves with the delivered entity.

## Mountain slices

Slicing is a client-local view filter over the existing full-depth exposure rule: below the selected
level, exposed terrain remains visible, while solid terrain at the selected level supplies the cut
face. The cut face uses the normal terrain material—no hatching, shading variant, or simulation/wire
state is introduced. The visible z-level is always shown as surface or underground in the client UI.

## Resolution contract

The simulation cell is **1.6 m**. The project/authored voxel is **0.1 m**, therefore there are
**16 project voxels per cell**. A declared asset may use an integer multiple of the project voxel:
for example the 10.2 pines use 0.2 m (= 2 project voxels) while retaining metre dimensions on the
same grid. This resolves the three apparently divergent values: the reference sheet's dwarf is
12 voxels = 1.20 m = 0.75 cells; its trees are measured in 1.6 m cells; and `gui`'s current
`scale: 0.65` is simply stale. Story 10.5 owes the one-line correction to `scale: 0.75`, because
1.20 m / 1.6 m per cell = 0.75. It is not a change made by this contract.

Terrain has a separate served resolution and budget from authored assets. The **adopted decision**
is **0.4 m terrain visual voxels**: visual subdivision **`k = 4`** of one 1.6 m simulation cell,
while the simulation remains `k = 1`.

**This is a decision, not yet a property of the shipped client.** The default build serves terrain
at `k = 1` (1.6 m): `TerrainSubdivision` is inserted only when `--subdiv` is passed
[`crates/gui/src/ingest.rs:203`], and every consumer falls back through `subdivision.map_or(1, ..)`
[`crates/gui/src/project.rs:1108`, `:1184`, `:1196`]. Reaching the adopted resolution today
requires `--subdiv 4`. Making `k = 4` the default — putting the adopted `k` in one constant so a
future evidence-led revision changes one constant rather than a grid convention — is **owed work
with no owner yet**; it is recorded in `deferred-work.md` and belongs to whichever story next
takes terrain rendering. Do not read this paragraph as a description of what the client does. The terrain budget is **80,120–928,884
chunk-mesh triangles at k=4**. The noisy measurement stand-in reaches the 928,884 ceiling; coherent
detail reaches 80,120, so the 11.5× bracket is not a look budget. Detail contributes 96.8% of that
ceiling and story 10.4's authored terrain decides where a real look lands. The ceiling excludes
about 54k triangles from the 4,501 tree-foliage cube entities, because it counts chunk meshes only.

This split is deliberate: terrain appears in tens of thousands of cells, while tree and five-dwarf
assets have instance counts roughly four orders of magnitude lower. Terrain is served at 0.4 m;
trees remain authored at their declared 0.2 m (a 2× project-voxel multiple); dwarves target 0.1 m
(12 voxels = 1.20 m = 0.75 cells). The reference sheet's 16 voxels/cell remains the authored-asset
target, not a request to render terrain at that density. k=8 is frame-rate servable (100–140 fps)
but is not adopted because every dig hitches 38–78 ms versus k=4's 5–13 ms. k=16 is geometry-only,
without a valid fps reading. **Mechanically-checkable:**
`scripts/tests/test_resolution_bench.py:ResolutionDetailRuleTests` pins the k>1 detail rule;
the choice of `k=4`, the bracket, and the per-class budgets are recorded measured decisions.

## Procedural-content contract

Procedural presentation is a client-side interpretation of `Material`, `EntityKind`, exposure and
the delivered world mirror; it must never add presentation fields to the wire. The following are
the standing rules implied by this guide. A cited test or instrument is a mechanical check; an
**eye-only** item is deliberate human vehicle review rather than an untestable claim of automation.

- Night terrain colours, brighter settled snow, green foliage and the cold-snow channel ordering
  come only from the appearance tables. **Mechanically-checkable:**
  `crates/gui/src/appearance.rs:303-390` (`appearance_tables_pin_the_cold_boot_palette`).
- Foliage is smaller than its tile according to the crown rule, without changing the six-neighbour
  exposed-face set. **Mechanically-checkable:** `scripts/tests/test_valley_bench.py:62-104`.
- Light is table-driven by `LightKind`; warm emitters remain chromatically distinct from cold fill,
  and flicker is deterministic, bounded presentation. **Mechanically-checkable:**
  `crates/gui/src/appearance.rs:158-204` and `:323-344`.
- The camp and valley retain the measured value ladder: capture's terrain window must be 70–180
  median sRGB, not merely non-black. **Mechanically-checkable:**
  `crates/gui/src/capture.rs:459` (`GROUND_LUMINANCE_FLOOR`) and `:464`
  (`GROUND_LUMINANCE_CEILING`), tested at `crates/gui/src/capture.rs:1329-1343`. This is the
  `--capture` instrument described under "The value floor" above, NOT a headless bench check:
  `scripts/bench/valley_bench.py:terrain_luma` computes a **mean** over non-sky pixels against a
  one-sided `MIN_TERRAIN_LUMA = 20.0` [`scripts/bench/valley_bench.py:110`, `:370`] and can see
  neither the 70 floor, the 180 ceiling, nor the median this clause requires.
- The boot camera keeps its approved camp and skyline placement, and its composition offset stays
  in the view plane rather than sliding along camera-right. **Mechanically-checkable:**
  `crates/gui/src/camera.rs:220-263`.
- Aurora is a camera-enclosing curtain ring with transparent top and bottom edges; stars sit on a
  larger shell, and both lie outside terrain fog. **Mechanically-checkable:**
  `crates/gui/src/atmosphere.rs:320-425`. Its particular colour and fold character are **eye-only**.
- Snowfall uses independent low-discrepancy placement, varied speeds and phase-preserving respawn;
  it must not form marching rows. **Mechanically-checkable:** `crates/gui/src/atmosphere.rs:511-581`.
- Fog supplies aerial perspective only. The world edge dissolves by a **13-level** rim ramp
  (`RIM_LEVELS` [`crates/gui/src/appearance.rs:251`]; the original linear 5-step ramp was replaced
  after it read as a hard band — see [`crates/gui/src/project.rs:1669`]) toward the
  exact sky colour and never by omitting draw-set tiles. **Mechanically-checkable:**
  `crates/gui/src/appearance.rs:596-617`; the draw-set number itself is a moving measurement,
  checked by the cube oracle rather than a fixed contract constant.
- Delivered entity motion may interpolate between the two received states but must never predict;
  local sky, flakes, flicker and dig chips carry no simulation meaning. **Mechanically-checkable:**
  `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:107-119`. The perceived readability of interpolation is **eye-only**.
- Mountain slicing is a client-local filter: expose below the selected level, retain the selected
  solid cut face, and add neither hatch nor simulation state. **Mechanically-checkable:**
  the selected level is clamped and read out by `SliceLevel`
  [`crates/gui/src/slice.rs:90-153`]. The exposure filter itself is `is_visible_at_slice`
  [`crates/gui/src/project.rs:1851-1856`], which has **no dedicated test**; that the cut face
  stays solid and that slicing adds neither hatch nor simulation state are **eye-only** until
  one exists.

## Asset contract

This section discharges the PRD's asset-contract obligation
[`prd.md:150`](_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/prd.md#L150).
It applies to output from a generator, hand work, or an MCP session. An asset class must declare
whether it replaces **tile** presentation (trees: `Material`/draw-set rules) or is an **entity**
(dwarves: reconciliation rules). Presentation is keyed by kind or material; RGB, radius, geometry
and other appearance values never become wire state (AD-16).

- **Grid and orientation.** Geometry uses metres, +Y up and applied transforms. It is authored on
  the 0.1 m project grid, or on an explicitly declared integer multiple (the existing pines are
  0.2 m = 2×). **Mechanically-checkable:** `scripts/bench/check_asset.py` verifies the supplied
  glTF's position bounds; the declared multiple and visual read are **eye-only**.
- **Origin.** The trunk/foot base sits at `min Y = 0`; its placement origin is centred in X and Z
  (within the published six-decimal tolerance). **Mechanically-checkable:**
  `scripts/bench/check_asset.py` reports and enforces these bounds.
- **Palette and material mapping.** The asset publishes its palette-cell-to-role map with its
  signoff figures. V1 trees use one embedded atlas image of exactly **64×64** texels, one material
  and one primitive, with `magFilter = NEAREST`, `wrapS`/`wrapT` = `CLAMP_TO_EDGE`, all
  `TEXCOORD_0` values inside 0–1, flat, single-sided (`doubleSided` absent or false)
  metallic-roughness material and neither `extensionsUsed` nor `extensionsRequired`.
  **Mechanically-checkable:** `scripts/bench/check_asset.py` enforces the count, atlas dimension,
  filter, wrap, UV-range, single-sided and extension clauses, and **publishes the seven sampled
  palette cells on the `FIGURES` line** so the role comparison below has something to read
  against. The intended role read and future multi-material mapping are **eye-only** until a
  concrete second format exists.

  **Scope, and it is load-bearing:** every clause in this bullet is **V1-voxel-asset only**, and
  `check_asset.py` applies them to any `.glb` it is handed. An asset class with two materials, no
  texture, or a different atlas size — an authored dwarf, for instance — will be REJECTED with
  `one-mesh/material/image clause (V1 voxel assets only)`. That is a **scope mismatch, not a
  contract violation**: story 10.5 introduces the second asset family and owns generalising these
  clauses. Do not "fix" a conforming asset to satisfy a pine rule.
- **Topology.** V1 voxel assets are greedy-meshed, unwelded quad soup: triangles are pairs of
  quads and `verts == tris/2 × 4`. They are flat-shaded and do not accept adjacency-based
  smoothing, decimation or auto-LOD. **Mechanically-checkable:**
  `scripts/bench/check_asset.py` checks the vertex/triangle relation; downstream-tool suitability
  is **eye-only** at import review.
- **Naming and locations.** A generator belongs under its story signoff directory until a runtime
  consumer is introduced; its generated runtime glTF belongs at `assets/gltf/<published-name>.glb`
  (created by story 10.5, not here); generated evidence, source notes and figures remain under
  `_bmad-output/implementation-artifacts/<story>-signoff/`. The file basename, mesh name and node
  name use the same published name. **Mechanically-checkable:** `scripts/bench/check_asset.py`
  checks that the mesh name, the node name AND the file basename all agree; path placement and
  source ownership are **eye-only** repository review.
- **Deliverables, and what counts as the record.** An asset ships three things: the editable
  source (`.blend` or equivalent), the exported per-variant glTF, and a **standalone headless
  generator script** that reproduces it. **The script is the durable record; the session is not** —
  a live MCP or interactive session that produced an asset without leaving a runnable script has
  not delivered one. Ported from story 10.2's standing contract (clause 7), which is the clause
  that makes the MCP path in this document's own opening sentence reproducible.
  **Mechanically-checkable:** nothing enforces this today; it is **eye-only** at signoff.
  (10.2's clauses 6 — self-verification order, "Exit 0 with no output is not a result" — and 8 —
  declaring known deviations — are recorded as deferred, see `deferred-work.md`. Note
  `_bmad-output/implementation-artifacts/10-2-signoff/voxel_pine.py:714` still cites "the asset
  contract's clause 6" and that reference does not resolve here.)
- **Identity and published figures.** An asset's identity is its published name **and** its
  published figures (size XYZ, min Y, centre X/Z, triangles and vertices), never its internal
  glTF name alone. The measured counterexample is
  `10-2-signoff/tree.glb`: its mesh and node name are `SM_VoxelPine_Tree02`, yet it is 5,130 tris,
  5.2 × 7.6 × 5.4 m and centre X −0.100000; the deliverable of that name is 5,894 tris,
  5.0 × 8.0 × 5.4 m and centre X +0.000000. **Mechanically-checkable:**
  `scripts/bench/check_asset.py` prints the figures and rejects the off-centre file by the named
  origin-centring clause, and a mismatched file basename by the naming clause. The published
  palette makes the collision visible too: the four deliverables publish
  `palette=#4A3B2E,...` while `tree.glb` publishes `palette=#110B07,...`. Comparing a new asset's
  published figures to its signoff record is **eye-only** until a second asset family establishes
  a manifest format.
