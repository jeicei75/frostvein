# Tech-art record

Why the rules in [`tech-art-guidelines.md`](tech-art-guidelines.md) are what they are: the
measurements that set each value, the candidates that were walked to the end and rejected, and the
readings that caught a real failure. Headings match the guidelines section by section.

This is a **record**, not a runbook — per `docs/dev-workflow.md`, it answers "why is it this way?"
and is read once, when someone asks. Nothing here is a rule. If a line here contradicts the
guidelines, the guidelines win and this file is stale.

## Value and materials

These choices live in `gui`'s appearance tables, not beside draw calls.

Foliage was `(55, 73, 84)` `#374954` **(superseded by 9.4)**, only 9.9 from stone on the same measure the
marks are held to at 40, so the base cubes were near-camouflage against the ground.

Foliage receives no terrain-style snow slab: its broken silhouette leaves the valley's snow
landform visible instead of turning every ground-level tree skirt into a bright tile.

The settled cap was trimmed in the same round-7 commit, `(158, 170, 196)` → `(146, 158, 184)`,
~8%: at the boot pitch the caps dominate the visible area, so the field's measured brightness
tracks THIS albedo more than the light table — boot4 proved the light lever weak, a 2.6x ambient
cut moving the field only 7%. The guidelines carried the pre-trim value until 2026-09-02 because
that commit's doc follow-up corrected only the crown.

The exposed crown's colour started as the approved artifact's `SPRUCE_SNOW` `(172, 186, 210)` `#ACBAD2`
**(superseded at round 7)** and was trimmed at round 7: the artifact shows that colour on thin
sprite tops while our cubes show whole faces of it, and every tree glowing at near-cap brightness
is what made the boot4 foreground read as clutter. This is a material swap and deliberately not a
terrain cap: capping foliage puts a bright slab on every ground-level skirt tile and buries the
landform, which is what the round-3 capture showed. Bare cube foliage without it reads as a dark
clump in a lit field.

Round-7 commit `10c06e1` moved the foliage taper together with the snow cap and the crown. Its
documentation follow-up recorded only the crown, leaving the guidelines' taper stale until story
10.4 corrected both occurrences.

## Sky and lights

The aurora was first built on a ring of **radius 220** **(superseded by
[Keeping the sky outside the camera](#keeping-the-sky-outside-the-camera): 600)**, and the star
shell at **radius 250** **(superseded: 650)**. The camera orbits, so any flat quad turns edge-on.
Its top stays below the boot camera's eye height. Three opaque cuboids failed this by mechanism,
not by placement — `unlit` plus `AlphaMode::Blend` on a `Cuboid` can only produce a flat rectangle with
hard edges.

The star band is narrow because this camera looks down: the visible sky is a thin wedge above the
ridge line, and a full dome would put most stars outside every frame. Sky materials sit far outside
the fog volume and would otherwise be erased with the far terrain.

**The light budget is set from measurement, not estimate — and it took two rounds of measurement to
converge.** The round-4 capture's valley floor read a median sRGB luminance of 22.5 against the
artifact's 112.6 (~18x short in linear light); scaling up by that factor then measured 156 on the
boot3 vehicle capture — 26% over — with shadows flooded (p05 87 vs the artifact's 28) and a
saturated blue-green cast. The overshoot taught the real rule: the budget divides, it doesn't just
scale. The boot3 cast came from lighting blue snow with saturated blue-green lights, not from the
material table. The white-clip radius scales as sqrt(intensity), and **72M lm** **(rejected — the
campfire ships at 25M)** blew a ~9-tile pool to flat white where AC9 reserves white for emissive
faces alone.

Measured on the approved artifact, the camp is only ~1.3x the field in luminance (135.9 vs 104.3)
while its R/B goes from 0.72 to 0.97. A bare **3x floor with no ceiling** **(superseded by the
1.2×–6.0× band)** was satisfied both by the 1/1000-scale table that shipped black and by a camp
blown to white. The table is static until story 6.1 adds flicker.

## Boot framing

The offset is necessary because a camera that directly looks at the camp would always centre it and
could not produce the approved foreground composition.

A push expressed in world axes (the original `(0, 6.7, -37.42)` **(superseded by the view-plane
offset)**) carries a 28.6-unit component along `right` at this yaw and slid the camp to 23% of the
frame width while every vertical assertion stayed green.

## Edge treatment

The ramp was originally described as **five quantised steps over the outermost 10 tiles**
**(superseded: 13 levels over `RIM_WIDTH`)**; the linear 5-step/10-tile ramp read as a hard band on
the boot4 vehicle capture.

Fog alone *cannot* dissolve this world's edge, and that is now measured rather than assumed. At the
boot framing the entire visible skyline **is** the map boundary — the silhouette against the sky
runs from depth 86 to depth 145 while the camp sits at 71. Fog tight enough to hide the nearest
silhouette point would also erase the valley the frame exists to show. This is the "fog skirt"
candidate walked to the end and found wanting, by arithmetic on the real framing.

Story 9.4 moved the draw-set oracle twice in one story: **53,365 of 315,068** **(superseded by
9.4)** before it, **45,261** **(superseded by 9.4's second half)** after the tree-density cut,
44,984 after the ground-level foliage ring was removed. It moves whenever world content moves; what
must not change is that the rim dissolves by colour alone and removes no tiles.

A lighter "horizon haze" would be more true to a real aurora-lit night — and the approved artifact
has one — but with a uniform sky it would make far terrain *lighter* than the sky behind it and hand
the world its edge straight back. That is a consequence of the sky being a flat `ClearColor`.

## Keeping the sky outside the camera

At the 500-unit clamp the camera orbits 426 units from the world centre, so the **220-unit ring**
**(superseded: 600)** the curtain was first built on would have put the camera outside it and swung
the aurora across the front of the valley at full vista — the exact register AC10 says must carry
sky and aurora.

## The value floor

The approved artifact reads 123 in that window; the round-4 capture read 21 (black field) and the
boot3 capture read 156 (washed toward white) — each end of the band has now caught a real failure.
No headless test can see either, so the instrument carries AC9's value discipline in both
directions.

Deriving two axes from the same constant correlates them perfectly — the first star shell used the
golden ratio for azimuth and height and all 300 stars landed on one helix, which the vehicle showed
as dotted lines across the sky. A shared speed or a shared respawn height re-synchronizes the flake
field into marching rows.

## Motion, flicker, and work evidence

The campfire's shadows mean its light no longer passes through solid terrain; each additional
shadow-casting emitter would cost six cube-map faces against NFR6. The dig chips make digging read
as work without inventing simulation state.

## Resolution contract

This resolves the three apparently divergent values: the reference sheet's dwarf is 12 voxels =
1.20 m = 0.75 cells; its trees are measured in 1.6 m cells; and `gui`'s current `scale: 0.65`
**(superseded — owed to 10.5 as 0.75)** is simply stale.

The noisy measurement stand-in reaches the 928,884 ceiling; coherent detail reaches 80,120, so the
11.5× bracket is not a look budget. Detail contributes 96.8% of that ceiling and story 10.4's
authored terrain decides where a real look lands.

The terrain/asset split is deliberate: terrain appears in tens of thousands of cells, while tree and
five-dwarf assets have instance counts roughly four orders of magnitude lower. The reference sheet's
16 voxels/cell remains the authored-asset target, not a request to render terrain at that density.
`k=8` is frame-rate servable (100–140 fps) but is not adopted because every dig hitches 38–78 ms
versus k=4's 5–13 ms. `k=16` is geometry-only, without a valid fps reading.
