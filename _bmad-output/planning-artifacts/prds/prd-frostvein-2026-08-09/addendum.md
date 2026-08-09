# PRD Addendum — frostvein Milestone 2

Depth that belongs downstream (architecture, UX, tech art, story work), not in
the PRD narrative.

## Z-level navigation in a 3D diorama (open design question, story-level)

Wolf's call: the game keeps discrete z-levels even in 3D, DF-style — dwarves
start at ground level and dig down; the player must be able to slice into the
mountain to see and work the underground. His candidate control is the
mousewheel, and he flagged it himself as needing practical testing, especially
for how slicing behaves *above* ground level.

Known collision to resolve in that story, not here: the mousewheel is also the
conventional zoom control for an orbit camera, and the zoom continuum
(diorama ↔ vista) is already decided. One wheel cannot drive both. Candidate
resolutions to test — modifier+wheel for slicing, dedicated keys (`<`/`>`
parity with the TUI), or slice-follows-selection. The PRD states the outcome
(underground legible and workable); the mechanism is chosen by testing.

## World-edge treatment at vista zoom (open design question, story-level)

The reference images are borderless; the 128×128 world shows its cut edges
when the camera pulls out. The PRD's bar: the world reads as a miniature
whose edges dissolve into the night — a raw grid edge is never visible at
any zoom. Candidates to test in the camera/atmosphere story: fog skirt,
darkness falloff at the rim, sky wrapping below the horizon line, vignette.
Chosen by testing, like the z-slice mechanism.

## Worldgen guidance from the references (non-binding)

- The narrative's camp sits on a raised, blocky plateau — guidance for
  starting-area terrain tuning inside FR27's story orbit, not an FR.
- Open question, to be decided on the record at worldgen tuning: should
  in-grid terrain give the vista skyline a mountain silhouette (peaks
  backlit by the aurora) within 128×128×32? M1's FR2 assumption ("modest
  rolling hills") was made for pathfinding, not for the vista register —
  it may need conscious revisiting, not silent stretching.

## Valheim — reference-game lesson (guidance, not target)

Wolf named Valheim as a distant reference. The transferable lesson for a
procedural-first voxel game: Valheim's beauty comes almost entirely from
atmosphere laid over cheap stylized geometry — fog, dramatic skies, lighting,
weather — on fully procedural seeded worlds with no designed levels. Richness
from light and air, not from authored content. That is exactly the cost
profile M2 wants: geometry stays procedural and simple; the budget goes to
light, sky, and atmosphere.

## Asset pipeline — recorded for the story that eventually needs it

Procedural-first is decided; no asset pipeline in the base build. When a
concrete case forces authored assets (dwarves are the expected first), the
short path is MagicaVoxel `.vox` via `bevy_vox_scene` (greedy meshing through
`block-mesh-rs`; approximates MagicaVoxel's rendered look). Bevy's hot reload
(`file_watcher`) is the art-iteration win; its asset *processing* system
stays off at this scale. Caveat on those dependencies: Bevy breaks its API
~3×/year and community voxel crates lag each release — every one is a
maintenance edge on a solo project.
