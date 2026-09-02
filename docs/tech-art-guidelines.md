# Tech-art guidelines

**This document is the current state only.** Every section states what MUST and MUST NOT hold, the
current values, and where the rule is checked. It carries no history and no rationale — those live
in [`tech-art-record.md`](tech-art-record.md), section by section under the same headings, and the
diffs live in git.

A number that is no longer the rule but is still named here carries **(superseded …)**. A number
that contradicts the code it describes carries **(stale …)** and is listed under
[Found during restructure](#found-during-restructure) rather than corrected.

**The approved artifact** this document measures against is two images, already in the repo:
[the diorama](17d7215b-6c05-4286-b3bb-56592ca617ec.jpg) and
[the vista](a9d4e72b-b4c3-43f2-8a1c-e25c539fd6c1.jpg). Every "the approved artifact reads …"
below is a measurement off one of them.

## Critical values

These tables are a **view** of the sections below, not a second source. The section is
authoritative; a table row exists so a value can be found quickly and links back to the section
that rules it. Where a section does not state a value, the cell is `—` and the enforcing symbol is
named instead — do not fill it in from the code without ruling the value into the section first.

### Materials

Ruled in [Value and materials](#value-and-materials), except the rim row. Every row is pinned by
`appearance_tables_pin_the_cold_boot_palette` except the last two: the taper by
`test_foliage_scale_matches_the_client_crown_rule` and
`test_foliage_scale_does_not_change_which_faces_are_exposed`, the rim by
`the_rim_dissolve_runs_from_the_untouched_material_to_the_bare_sky`.

| Identifier | RGB | Hex | Notes |
| --- | --- | --- | --- |
| `Material::Stone` | `(60, 70, 92)` | `#3C465C` | night palette |
| `Material::Soil` | `(56, 52, 62)` | `#38343E` | night palette |
| `Material::Ice` | `(104, 128, 170)` | `#6880AA` | keeps its blue surface; never capped |
| `Material::Snow` | `(136, 150, 178)` | `#8896B2` | terrain snow; night palette |
| `snow_cap_color()` | `(158, 170, 196)` **(stale)** | `#9EAAC4` | settled cap — its own material, not a brighter snow |
| `Material::TreeFoliage` | `(44, 100, 58)` | `#2C643A` | green since 9.4 |
| ↳ before 9.4 | `(55, 73, 84)` **(superseded by 9.4)** | `#374954` | 9.9 from stone — near-camouflage |
| `foliage_snow_color()` | `(156, 170, 196)` | `#9CAAC4` | exposed spruce crown; a material swap, never a terrain cap |
| ↳ before round 7 | `(172, 186, 210)` **(superseded at round 7)** | `#ACBAD2` | the artifact's `SPRUCE_SNOW` |
| `Material::TreeTrunk` | — | — | stated in the code table only |
| foliage taper | 0.72 / 0.86 / full | — | skirts and tips / upper crown / mid-crown |
| `rim_dissolved_color()` | exactly the sky colour | — | see [Edge treatment](#edge-treatment) |

**(stale)** — the code ships `(146, 158, 184)` `#929EB8`; see
[Found during restructure](#found-during-restructure).

### Lights

Ruled in [Sky and lights](#sky-and-lights), except the last row. Pinned by
`appearance_tables_pin_the_cold_boot_palette`; the flicker bands by
`flicker_is_bounded_distinct_and_deterministic`.

| Identifier | Colour | Hex | Intensity | Range / shadow | Flicker |
| --- | --- | --- | --- | --- | --- |
| `night_lighting().ambient` | `(120, 140, 165)` | `#788CA5` | 4,500 | cold fill | static |
| `night_lighting().directional` | `(150, 190, 180)` | `#96BEB4` | 22,000 | key light | static |
| `LightKind::Torch` | — | — | 14M lm | — | bounded, deterministic |
| `LightKind::Campfire` | — | — | 25M lm | only shadow caster | 1.40 peak → 35M |
| ↳ rejected | — | — | 72M lm **(rejected)** | blew a ~9-tile pool white | — |
| moving light | table-driven by `LightKind` | — | table-driven | table-driven | eye-only |

The sections state no colour, range or amplitude for torch and campfire; those cells stay `—`
rather than importing values the prose has not ruled. The moving-light row is ruled in
[Moving lights](#moving-lights).

### Sky and geometry

Ruled in [Keeping the sky outside the camera](#keeping-the-sky-outside-the-camera) (the radii and
the horizon angle), [Sky and lights](#sky-and-lights) (curtain top and edge alpha),
[Boot framing](#boot-framing) (camera and composition) and
[Edge treatment](#edge-treatment) (fog and rim).

| Value | Current | Enforced by |
| --- | --- | --- |
| Aurora ring radius | 600 (`AURORA_RADIUS`) | `the_curtain_mesh_is_a_closed_ring_at_the_aurora_radius` |
| Aurora ring, first build | 220 **(superseded)** | camera fell outside it at the 500 clamp |
| Star shell radius | 650 (`STAR_RADIUS`) | `the_star_shell_fills_the_visible_sky_wedge` |
| Star shell, first build | 250 **(superseded)** | superseded with the curtain |
| Curtain top | 45 (`AURORA_TOP`) | `the_aurora_curtain_hugs_the_horizon_beyond_the_world` |
| Horizon angle rule | an angle from the boot eye, ceiling from `BOOT_VERTICAL_FOV` | `the_aurora_curtain_hugs_the_horizon_beyond_the_world` |
| Curtain edge alpha | zero top and bottom, by `sin(pi v)` | `the_aurora_gradient_fades_to_nothing_at_both_edges` |
| Boot camera | yaw 0.70, pitch 0.45 rad, distance 90 | the boot composition tests below |
| Composition targets | camp 48% W / 78% H, skyline 24% H, ±3% | the boot composition tests below |
| Composition offset | view plane only, zero along camera-right | `boot_composition_never_pushes_along_the_camera_right_vector` |
| Fog range at boot | opens 75, saturates 155 | eye-only |
| Rim dissolve | 13 levels (`RIM_LEVELS`) over `RIM_WIDTH` | the rim dissolve test below |
| Rim dissolve, before | linear 5 steps over 10 tiles **(superseded)** | read as a hard band |
| Draw-set oracle | 44,984 of 301,048 — a measurement | the cube oracle |

The boot composition tests are
`boot_composition_places_the_camp_low_and_the_skyline_at_the_top_third` and
`boot_composition_never_pushes_along_the_camera_right_vector`; the rim dissolve test is
`the_rim_dissolve_runs_from_the_untouched_material_to_the_bare_sky`.

### Value ladder

Ruled in [The value floor](#the-value-floor), except the contrast band, the chromatic term and the
artifact camp/field readings, which are ruled in [Sky and lights](#sky-and-lights).

| Value | Current | Enforced by |
| --- | --- | --- |
| Capture window | x 0.25–0.75, y 0.50–0.90, frame centre | `the_ground_median_reads_the_valley_floor_and_ignores_the_sky` |
| Floor | 70 median sRGB | `GROUND_LUMINANCE_FLOOR` |
| Ceiling | 180 median sRGB | `GROUND_LUMINANCE_CEILING` |
| Statistic | median, never mean | `the_ground_median_reads_the_valley_floor_and_ignores_the_sky` |
| Artifact: capture window | 123 | eye-only |
| Artifact: valley floor | 112.6 | eye-only |
| Artifact: camp vs field | 135.9 vs 104.3 (~1.3x) | eye-only |
| Artifact: camp R/B | 0.72 → 0.97 | eye-only |
| Artifact: shadows p05 | 28 | eye-only |
| Caught — round-4 capture | 21, a black field | eye-only |
| Caught — boot3 capture | 156, p05 87, washed white | eye-only |
| Contrast band | 1.2x–6.0x campfire-to-cold-fill | `appearance_tables_pin_the_cold_boot_palette` |
| Chromatic term | every light's R/B ≥ 2x the ambient's | `appearance_tables_pin_the_cold_boot_palette` |
| Contrast, before | a bare 3x floor, no ceiling **(superseded)** | passed by both a black frame and a blown camp |
| Bench floor | `MIN_TERRAIN_LUMA = 20.0`, one-sided | `pixel_figures` in `valley_bench.py` |

The bench floor is a **different instrument**: a mean over non-sky pixels, which can see neither
the 70 floor, the 180 ceiling, nor the median the rows above require.

### Resolution and budgets

Ruled in [Resolution contract](#resolution-contract). `ResolutionDetailRuleTests`
(`scripts/tests/test_resolution_bench.py`) pins the k>1 detail rule; the adopted `k`, the bracket
and the per-class budgets are recorded measured decisions, not mechanical checks.

| Value | Current | Enforced by |
| --- | --- | --- |
| Simulation cell | 1.6 m | measured decision (10.6) |
| Project/authored voxel | 0.1 m → 16 per cell | measured decision |
| Terrain visual subdivision | adopted `k = 4` (0.4 m) | measured decision |
| Terrain, shipped default | `k = 1` — needs `--subdiv 4` | `deferred-work.md`, no owner |
| Trees | 0.2 m (2x the project voxel) | `check_asset.py` bounds |
| Dwarves | 0.1 m — 12 voxels = 1.20 m = 0.75 cells | eye-only until 10.5 |
| Terrain triangle bracket | 80,120–928,884 at k=4 | measured decision |
| Bracket excludes | ~54k tris, 4,501 foliage cubes | counts chunk meshes only |
| Dwarf render scale | `0.65` shipped → `0.75` owed by 10.5 | eye-only |
| Not adopted — `k = 8` | 100–140 fps, digs hitch 38–78 ms | measured decision |
| Not adopted — `k = 16` | geometry-only, no valid fps reading | measured decision |

## Value and materials

The boot frame is a night scene.

- Terrain and foliage colours MUST come from `gui`'s appearance tables, not from beside a draw call.
- Stone, soil, ice and terrain snow take the approved dark-but-readable night palette. The values
  are in [Materials](#materials); they are not repeated here.
- A settled snow cap is its own brighter material, not a brighter snow, so caps read over snow
  terrain without becoming white.
- Near-white MUST remain reserved for stars and emitter faces.
- Snow is a client-side settled cap: exposed snow, stone and soil tops receive the distinct cap
  material; ice keeps its blue surface; covered terrain retains its bare flank.
- Foliage MUST NOT receive a terrain-style snow slab.
- Foliage is green since story 9.4. Trees separate on GREEN, the axis the cool directional does
  not compress.
- Every terrain material MUST keep blue at or above red.
- Foliage cubes taper by their contiguous foliage above: ground skirts and crown tips scale to
  0.72, the upper crown to 0.86, and the mid-crown stays full scale. The taper MUST NOT change the
  six-neighbour exposed-face set.
- Ramps follow the same material rules, because the renderer presents them as full cubes.
- The **exposed crown** of a spruce — foliage with nothing solid above it — takes its own
  snow-laden material. This is a material swap and MUST NOT be a terrain cap.

Check: `appearance_tables_pin_the_cold_boot_palette` (`crates/gui/src/appearance.rs`) pins the
palette, `snow_cap_color`, `foliage_snow_color` and the blue-at-or-above-red ordering;
`test_foliage_scale_matches_the_client_crown_rule` and
`test_foliage_scale_does_not_change_which_faces_are_exposed`
(`scripts/tests/test_valley_bench.py`, `ValleyGeometryTests`) pin the crown rule.

## Sky and lights

- The sky is an illuminant. Cold ambient fill and a green-blue directional light let the aurora
  catch snow and ice.
- The aurora MUST be a **curtain on a ring** around the world, not a set of billboards.
- Its shape comes entirely from a procedurally generated RGBA gradient — the table's aurora colour
  throughout, with alpha forced to exactly zero at the top and bottom edges of the strip by a
  `sin(pi v)` term, so the curtain has no silhouette.
- Low-frequency folds across the ring stop it reading as a ribbon.
- The curtain top is 45 (`AURORA_TOP`) and MUST stay below the boot camera's eye height — that is
  what "hugs the horizon rather than hanging overhead" means in geometry. The eye height is derived
  from the boot camera and is deliberately not restated here. The ring radius is ruled under
  [Keeping the sky outside the camera](#keeping-the-sky-outside-the-camera), not here.
- Stars sit on a shell, scattered by golden-angle azimuth over a deliberately narrow height band.
  Sizes vary so the shell does not read as a lattice.
- Sky materials MUST set `fog_enabled: false`.
- Torch, campfire, and future lantern properties are one data table containing colour, lumen
  intensity, and range.
- The light budget **divides, it does not just scale**: a small desaturated ambient lets shadow
  faces go genuinely dark; a desaturated cool directional carries the lit faces. Both tints sit
  near neutral because light colour MULTIPLIES onto already-blue materials. The values are in
  [Lights](#lights).
- Torches are 14M lm and the campfire 25M lm (35M at its 1.40 flicker peak).
- **Warm against cold is carried by hue, not by a large luminance ratio.** The contrast oracle is a
  *band* — campfire-to-cold-fill between 1.2× and 6.0× — plus a chromatic term: every light's R/B
  at least twice the ambient's.
- The ladder is dark sky and flanks, midtone snow and ice, then warm pools and near-white emitter
  faces.

Check: `appearance_tables_pin_the_cold_boot_palette` and `flicker_is_bounded_distinct_and_deterministic`
(`crates/gui/src/appearance.rs`) pin the light table, the warm-emitter ordering and the flicker
band; `the_aurora_curtain_hugs_the_horizon_beyond_the_world`,
`the_curtain_mesh_is_a_closed_ring_at_the_aurora_radius` and
`the_aurora_gradient_fades_to_nothing_at_both_edges` (`crates/gui/src/atmosphere.rs`) pin the
curtain; `the_star_shell_fills_the_visible_sky_wedge` and
`star_sizes_vary_so_the_shell_never_reads_as_a_lattice` pin the shell. Aurora colour and fold
character are **eye-only**.

## Boot framing

- The boot camera uses yaw 0.70, pitch 0.45 radians, and distance 90.
- It looks through a composition offset toward the far valley: the camp projects to 48% of the
  frame width and 78% of its height, and the far skyline to 24%, each within a 3% tolerance.
- The offset scales down below boot distance so the 4-unit close zoom keeps the camp in front of
  the camera; it is fully applied from the boot distance through the 500-unit vista.
- **The offset MUST lie in the camera's view plane** — along the view direction and straight up,
  never along the camera's right vector.

Check: `boot_composition_places_the_camp_low_and_the_skyline_at_the_top_third` and
`boot_composition_never_pushes_along_the_camera_right_vector` (`crates/gui/src/camera.rs`), against
`BOOT_YAW`, `BOOT_PITCH` and `BOOT_DISTANCE`.

## Edge treatment

Two mechanisms are in the build, and they do different jobs.

- **Distance fog is aerial perspective, not the edge treatment.** Its range follows the camera
  distance so the far valley survives the whole zoom clamp: at the boot framing it opens at 75
  (just past the camp at depth 71) and saturates at 155 (just past the deepest in-frame terrain at
  148).
- **The world edge is dissolved in world space instead.** `rim_level` blends terrain toward the sky
  colour in quantised steps, so the boundary fades out at every zoom and camera angle rather than at
  one tuned distance. The current ramp is **13 levels** (`RIM_LEVELS`) over `RIM_WIDTH` tiles,
  quadratically eased.
- Shared steps per surface keep the dissolve to a handful of material handles; per-tile blending
  would mean one material per cube.
- The rim tiles MUST still be drawn. The draw set is watched by the cube oracle and MUST never
  shrink to hide an edge. **The oracle is a measurement of the shipped world, not a constant** — it
  reads **44,984** exposed cubes of 301,048 solid today, and moves whenever world content moves.
- **The fog colour and the rim's target colour MUST both be exactly the sky colour.** A haze colour
  only becomes available once the sky itself carries a vertical gradient; until then these three
  colours move together or not at all.
- Per AC11's amendment the final choice is still Wolf's, at the vehicle. What is settled here is
  that fog-alone has been eliminated on evidence.

Check: `the_rim_dissolve_runs_from_the_untouched_material_to_the_bare_sky`
(`crates/gui/src/appearance.rs`) pins level 0 as untouched, the last level as pure sky, and
monotonicity; `rim_level` (`crates/gui/src/project.rs`) computes the level. Fog range is
**eye-only**.

## Keeping the sky outside the camera

- Sky geometry is hung on rings around the world, so its radius is not a free choice: the camera
  MUST stay **inside** it at every zoom.
- The curtain sits at 600 (`AURORA_RADIUS`) and the star shell at 650 (`STAR_RADIUS`), and a test
  pins the camera's furthest excursion below both.
- Sizes and height bands are scaled to that radius.
- The "hugs the horizon" rule MUST be asserted as an ANGLE from the boot eye, against a ceiling
  derived from `BOOT_VERTICAL_FOV`, because a raw height threshold stops meaning anything the
  moment the radius changes. **The ceiling is not written down here**: it is computed in the test,
  and a copy of it in prose is a number that can rot while the test stays green.

Check: `the_aurora_curtain_hugs_the_horizon_beyond_the_world` pins the angle and that the ring
encloses the world footprint; `the_curtain_mesh_is_a_closed_ring_at_the_aurora_radius` and
`the_star_shell_fills_the_visible_sky_wedge` pin the radii and that the camera's furthest excursion
stays below both (`crates/gui/src/atmosphere.rs`).

## The value floor

- `--capture` range-checks the **median** sRGB luminance of the valley floor (frame centre, x 0.25
  to 0.75 and y 0.50 to 0.90) against a floor of **70** AND a ceiling of **180**.
- Median rather than mean, so a handful of blown-out emitter faces cannot carry a black field over
  the floor.
- Sky scatter MUST use low-discrepancy pairs with INDEPENDENT irrationals per axis (the R2/R3
  sequences).
- The same rule covers snowfall: scattered disc positions, per-flake fall speeds and
  phase-preserving respawn.

Check: `GROUND_LUMINANCE_FLOOR` and `GROUND_LUMINANCE_CEILING` (`crates/gui/src/capture.rs`),
tested by `the_ground_median_reads_the_valley_floor_and_ignores_the_sky`,
`a_black_field_fails_the_value_floor_that_a_lit_one_passes` and
`a_blown_out_field_fails_the_value_ceiling_that_a_midtone_one_passes`;
`stars_scatter_instead_of_lying_on_a_helix` and
`snowfall_scatters_through_the_camp_read_without_marching_in_rows`
(`crates/gui/src/atmosphere.rs`).

## Motion, flicker, and work evidence

- Motion is presentation only: dynamic entities blend between the previous and current positions
  the wire delivered, clamped at the current position, with **no prediction or extrapolation**.
- Torch and campfire point lights breathe inside their table-defined bands from a deterministic
  function of simulation id and client elapsed time; the emitter material remains static.
- The campfire — and **only** the campfire — casts point-light shadows.
- When an `Empty` tile arrives, four deterministic, client-local stone chips sit at the position
  until a snapshot rebuild clears them. They carry no simulation state.

Check: `flicker_is_bounded_distinct_and_deterministic` (`crates/gui/src/appearance.rs`) pins the
flicker bands and determinism; the no-extrapolation rule is ruled by `ARCHITECTURE-SPINE.md`
§ **AD-15 — Interpolation is presentation**. The perceived readability of interpolation is
**eye-only**.

## Moving lights

- A moving light MUST use the same `LightKind` appearance-table lookup as a static one.
- A dwarf is **not** special-cased warm: when its wire entity carries a light, reconciliation
  attaches that table-driven point light to the dwarf's blended projection, so the pool moves with
  the delivered entity.

Check: `light_properties` / `LightKind` (`crates/gui/src/appearance.rs`), pinned by
`appearance_tables_pin_the_cold_boot_palette`.

## Mountain slices

- Slicing is a client-local view filter over the existing full-depth exposure rule: below the
  selected level, exposed terrain remains visible, while solid terrain at the selected level
  supplies the cut face.
- The cut face MUST use the normal terrain material — no hatching, shading variant, or
  simulation/wire state is introduced.
- The visible z-level MUST always be shown as surface or underground in the client UI.

Check: `SliceLevel` (`crates/gui/src/slice.rs`) is pinned by
`the_slice_starts_at_the_top_and_clamps_at_both_world_bounds` and
`the_readout_names_the_current_level_and_whether_it_is_surface_or_underground`. The exposure filter
itself is `is_visible_at_slice` (`crates/gui/src/project.rs`), which has **no dedicated test**; the
solid cut face and the absence of hatch or simulation state are **eye-only** until one exists.

## Resolution contract

- The simulation cell is **1.6 m**. The project/authored voxel is **0.1 m**, therefore there are
  **16 project voxels per cell**.
- A declared asset MAY use an integer multiple of the project voxel: the 10.2 pines use 0.2 m
  (= 2 project voxels) while retaining metre dimensions on the same grid.
- Terrain has a separate served resolution and budget from authored assets. The **adopted decision**
  is **0.4 m terrain visual voxels**: visual subdivision **`k = 4`** of one 1.6 m simulation cell,
  while the simulation remains `k = 1`.
- **Adopted is not shipped.** The default build serves terrain at `k = 1`; reaching the adopted
  resolution today requires `--subdiv 4`. Putting the adopted `k` behind one constant is **owed work
  with no owner yet** — see `deferred-work.md`, "The adopted terrain `k = 4` has no constant and no
  owner".
- The terrain budget is **80,120–928,884 chunk-mesh triangles at k=4**. The ceiling excludes about
  54k triangles from the 4,501 tree-foliage cube entities, because it counts chunk meshes only.
- Terrain is served at 0.4 m; trees remain authored at their declared 0.2 m (a 2× project-voxel
  multiple); dwarves target 0.1 m (12 voxels = 1.20 m = 0.75 cells).
- `gui`'s `scale: 0.65` is stale. **Story 10.5 owes the one-line correction to `scale: 0.75`**,
  because 1.20 m / 1.6 m per cell = 0.75. It is not a change made by this contract.

Check: `ResolutionDetailRuleTests` (`scripts/tests/test_resolution_bench.py`) pins the k>1 detail
rule; the choice of `k=4`, the bracket, and the per-class budgets are recorded measured decisions.

## Procedural-content contract

Procedural presentation is a client-side interpretation of `Material`, `EntityKind`, exposure and
the delivered world mirror; it must never add presentation fields to the wire. The following are
the standing rules implied by this guide. A cited test or instrument is a mechanical check; an
**eye-only** item is deliberate human vehicle review rather than an untestable claim of automation.

- Night terrain colours, brighter settled snow, green foliage and the cold-snow channel ordering
  come only from the appearance tables. **Mechanically-checkable:**
  `appearance_tables_pin_the_cold_boot_palette` (`crates/gui/src/appearance.rs`).
- Foliage is smaller than its tile according to the crown rule, without changing the six-neighbour
  exposed-face set. **Mechanically-checkable:** `ValleyGeometryTests` —
  `test_foliage_scale_matches_the_client_crown_rule`, `test_foliage_is_drawn_smaller_than_its_cell`
  and `test_foliage_scale_does_not_change_which_faces_are_exposed`
  (`scripts/tests/test_valley_bench.py`).
- Light is table-driven by `LightKind`; warm emitters remain chromatically distinct from cold fill,
  and flicker is deterministic, bounded presentation. **Mechanically-checkable:**
  `flicker_is_bounded_distinct_and_deterministic` and the light-table block of
  `appearance_tables_pin_the_cold_boot_palette` (`crates/gui/src/appearance.rs`).
- The camp and valley retain the measured value ladder: capture's terrain window must be 70–180
  median sRGB, not merely non-black. **Mechanically-checkable:**
  `GROUND_LUMINANCE_FLOOR` and `GROUND_LUMINANCE_CEILING` (`crates/gui/src/capture.rs`), tested by
  `a_black_field_fails_the_value_floor_that_a_lit_one_passes` and
  `a_blown_out_field_fails_the_value_ceiling_that_a_midtone_one_passes`. This is the
  `--capture` instrument described under "The value floor" above, NOT a headless bench check:
  `pixel_figures` in `scripts/bench/valley_bench.py` computes `terrain_luma` as a **mean** over
  non-sky pixels against a one-sided `MIN_TERRAIN_LUMA = 20.0` and can see neither the 70 floor,
  the 180 ceiling, nor the median this clause requires.
- The boot camera keeps its approved camp and skyline placement, and its composition offset stays
  in the view plane rather than sliding along camera-right. **Mechanically-checkable:**
  `boot_composition_places_the_camp_low_and_the_skyline_at_the_top_third` and
  `boot_composition_never_pushes_along_the_camera_right_vector` (`crates/gui/src/camera.rs`).
- Aurora is a camera-enclosing curtain ring with transparent top and bottom edges; stars sit on a
  larger shell, and both lie outside terrain fog. **Mechanically-checkable:**
  `the_aurora_curtain_hugs_the_horizon_beyond_the_world`,
  `the_curtain_mesh_is_a_closed_ring_at_the_aurora_radius` and
  `the_aurora_gradient_fades_to_nothing_at_both_edges` (`crates/gui/src/atmosphere.rs`). Its
  particular colour and fold character are **eye-only**.
- Snowfall uses independent low-discrepancy placement, varied speeds and phase-preserving respawn;
  it must not form marching rows. **Mechanically-checkable:**
  `snowfall_scatters_through_the_camp_read_without_marching_in_rows`
  (`crates/gui/src/atmosphere.rs`).
- Fog supplies aerial perspective only. The world edge dissolves by a **13-level** rim ramp
  (`RIM_LEVELS` in `crates/gui/src/appearance.rs`; the original linear 5-step ramp
  **(superseded)** was replaced after it read as a hard band — see `rim_level` in
  `crates/gui/src/project.rs`) toward the exact sky colour and never by omitting draw-set tiles.
  **Mechanically-checkable:** `the_rim_dissolve_runs_from_the_untouched_material_to_the_bare_sky`
  (`crates/gui/src/appearance.rs`); the draw-set number itself is a moving measurement,
  checked by the cube oracle rather than a fixed contract constant.
- Delivered entity motion may interpolate between the two received states but must never predict;
  local sky, flakes, flicker and dig chips carry no simulation meaning. **Mechanically-checkable:**
  `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`
  § **AD-15 — Interpolation is presentation**. The perceived readability of interpolation is
  **eye-only**.
- Mountain slicing is a client-local filter: expose below the selected level, retain the selected
  solid cut face, and add neither hatch nor simulation state. **Mechanically-checkable:**
  the selected level is clamped and read out by `SliceLevel` (`crates/gui/src/slice.rs`), pinned by
  `the_slice_starts_at_the_top_and_clamps_at_both_world_bounds` and
  `the_readout_names_the_current_level_and_whether_it_is_surface_or_underground`. The exposure
  filter itself is `is_visible_at_slice` (`crates/gui/src/project.rs`), which has **no dedicated
  test**; that the cut face stays solid and that slicing adds neither hatch nor simulation state
  are **eye-only** until one exists.

## Asset contract

This section discharges the PRD's asset-contract obligation
([`prd.md`](_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/prd.md) §
"Scope shape (agreed, pre-FR)", the tech-art-guidelines deliverable bullet).
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
  `_bmad-output/implementation-artifacts/10-2-signoff/voxel_pine.py`, in its
  `if __name__ == "__main__":` guard, still cites `"the asset contract's clause 6"` and that
  reference does not resolve here.)
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

## Found during restructure

Content that is wrong by inspection. **Nothing below was corrected** — each is listed for a ruling,
because fixing it would change a value this pass is not authorised to change.

1. **Settled snow cap: this document says `(158, 170, 196)` `#9EAAC4`; the code ships `(146, 158, 184)` `#929EB8`.**
   `snow_cap_color` (`crates/gui/src/appearance.rs`) returns `(146, 158, 184)` `#929EB8` and
   `appearance_tables_pin_the_cold_boot_palette` asserts exactly that. The doc's triple is ~8%
   brighter on every channel, which matches the code's own note that the cap was "trimmed ~8% at
   round 7" — so the document appears to carry the **pre-round-7** value. Marked `(stale …)` in
   "Value and materials" and in the Materials table.
2. **The horizon angle rule said "≤ 10 degrees"; the test rejects 10 degrees. RESOLVED BY
   REMOVAL.** `the_aurora_curtain_hugs_the_horizon_beyond_the_world` computes its ceiling as
   `(BOOT_VERTICAL_FOV * 0.5).to_degrees() / 4.0`, and its comment records that "a first attempt
   used 10 deg, which at a 600-unit radius let a 140-unit top through at 8 deg — the sabotage
   caught it". The doc stated the value the sabotage killed. **The number is now gone from this
   document** rather than corrected: it is derived, so any copy of it here can rot while the test
   stays green. The rule names the mechanism and the constant it derives from; the ceiling lives in
   the test.
3. **Boot camera eye height said 55; the shipped constants give ~47.6. RESOLVED BY REMOVAL here,
   STILL OPEN in the code.** `CameraRig::new([64, 64, 9]).transform().translation.y` is
   `world_to_render([64,64,9]).y + BOOT_COMPOSITION_LIFT + BOOT_DISTANCE * sin(BOOT_PITCH)` =
   `9 + (−0.5) + 90 × sin(0.45)` = **47.647**. The rule ("curtain top stays below the eye") holds
   either way, and the eye height added nothing to it, so **the number is gone from this document**.
   It is NOT gone from the code: `the_aurora_curtain_hugs_the_horizon_beyond_the_world`'s comment
   still says "Production reads −0.95 deg", which back-solves to an eye height of 55, where the
   constants give −0.253°. Doc and code comment agreed with each other and both disagreed with the
   constants — which is why a doc-vs-code text guard would not have caught this one.
4. **`RIM_LEVELS`'s own doc comment contradicts its value.** The comment above
   `pub const RIM_LEVELS: usize = 13` still reads "five shared steps read as a gradient and cost
   five handles per slot". Code-side only; not touched by this pass.
5. **Dangling reference, already recorded.** `voxel_pine.py`'s `__main__` guard cites "the asset
   contract's clause 6", which was never ported into this document. Already carried in
   `deferred-work.md`; the citation here now names the guard rather than a line number, but the
   reference still does not resolve.
