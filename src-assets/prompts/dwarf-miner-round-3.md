# Round 3 — the `dwarf.mp4` look, and a scheme for hundreds of dwarves

**The target has moved and so has the constraint.** Wolf, 2026-09-11, verbatim:

> "Now when we are not limited to procedural strict voxel dwarfs I would like them to look more
> like in reference sheets. Now we have vertical voxel bands and detail is not enough. I think the
> best render is actually in dwarf.mp4. And I think we need that level of detail anyway for
> rendered shots. Then after that is done we can optimize dwarves for the game if needed. One
> thing there to consider how we can make hundreds of variations easily without creating all
> models from the scratch."

So: **`src-assets/references/dwarf.mp4` is the authority for this round**, not
`dwarf-contact-sheet.jpg`.

**And the judge is the GAME, at every zoom level.** Wolf, immediately after the above:

> "sure but plan is to be able to zoom in zoom out freely.. and my ambitious target is to actually
> take shots from game and use game as a rendering engine also for 'marketing' material"

That settles something this brief originally got wrong. There is **no hero-versus-game split and no
second asset family**: one asset has to hold from a handful of pixels to filling the frame, because
the same renderer produces both the gameplay view and the marketing shot. "Optimise for the game
later" means tune cost later — it does not mean a throwaway model now.

## Verified here on the shipped GLB, independently — do not redo any of this

`scripts/bench/check_asset.py assets/gltf/SM_VoxelDwarf_Miner01.glb` exits 0 and reports:

    size_m=1.2x1.2x0.8  min_y_m=0.000000  centre_x_m=0.000000  centre_z_m=0.000000
    tris=14398  verts=28796
    palette=#E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50,#6B5B49,#34271C,#F0A63C

Round 2's five features all landed — the beard is stepped, the tunic has a collar, the belt has a
buckle plate, the lantern has a cage with the flame visible between bars, the boots have cuffs. The
`backpack` group is there too, 5,900-odd voxels with a bedroll, flap, cinch strap and buckle; it
simply does not appear in a front render. **The asset did what round 2 asked. Round 2 asked for the
wrong thing.**

**One number, and read it as the FLOOR of a range rather than as the target.** At the shipped boot
framing a dwarf draws **8.74 px tall** — measured through the client's own projection oracle
(`CameraRig::project_world_point_with_depth`): 11.66 px per terrain cell at the camp, and a 1.20 m
dwarf draws 0.75 cells (`crates/gui/src/project.rs:270`). A 4.3 m pine gets 31 px. Today that is
the only framing there is; with free zoom it becomes the wide end of the range and the close end is
the whole screen.

**This is why the brief below is a constraint and not a compromise.** A detail has to work at both
ends. Grooves at 96 voxels are sub-pixel at 8.74 px — they do not merely vanish, they **alias**,
and an aliasing surface shimmers as the camera moves. Big value steps and silhouette notches, which
is what the reference uses, survive downsampling into readable shape. **The same fix serves the
close-up and the wide shot.** Do not trade look for triangles here, and do not trade the wide shot
for the close-up either.

## What the video actually shows, measured rather than admired

From the frame at **t=10 s** (`ffmpeg -i dwarf.mp4 -vf fps=1`), at 8x nearest-neighbour:

- **The reference is COARSER than what we built.** Voxel pitch reads ~8 px and the figure spans
  ~510 px, so the reference dwarf is roughly **60–65 voxels tall against our 96**. Its belt buckle
  is about **5 voxels** across. Read off a perspective render, so treat it as ±10 %, but the
  direction is not in doubt: **we are already finer than the target and still read flatter.**
  **Resolution is not the lever.** Do not raise the voxel count in this round.
- **Value steps inside one material.** The tunic carries at least three greens — a lighter band at
  the hem, a mid field, a darker fold at the side. Ours has exactly **one** tunic cell, `#5F7A6A`,
  and leans on scene lighting for all its form. Same story for the leather and the beard.
- **Detail lives in the silhouette.** Beard locks notch the outline; the tunic hem steps; the boot
  cuffs step; the buckle is a shape, not a pattern. None of it is surface grooving.
- **He is POSED.** Across t=1/4/7/10 s the arms and legs sit at different angles, the torso bends
  over the swing. That needs separate limbs, which is what "not limited to procedural strict voxel
  dwarfs" unlocks.
- **The lantern emits.** It is the brightest thing in frame and throws a warm pool on the ground.

**What I could NOT read from the video, so do not guess at it either:** whether each material
carries a fine surface texture. The file is 3.4 MB for 10 s at 1280x720, and at 8x the belt's
mottling is indistinguishable from h264 blocking. If you want a textured look, propose it as a
decision with a render, do not infer it from this source.

## The diagnosis, and it is the opposite of round 2's

Round 2 said "more detail, and specifically NOT more triangles", and the session delivered detail
as **grooves**: `groove()` in `dwarf_miner.py` cuts a 1-voxel channel every `step=3` down z across
the beard and tunic. At 96 voxels tall that is corduroy — **the "vertical voxel bands" Wolf is
objecting to.** We bought 2.4x the reference's resolution and spent it on a repeating surface
pattern, while the reference spends its lower resolution on form, value and pose.

## The ask

1. **Remove the groove passes.** Falsifiable on delivery: no 1-voxel channel repeating at a fixed
   `z` step anywhere on the asset.
2. **Give every major material 2–3 palette cells and paint form with them** — hem and fold on the
   tunic, at least two browns in the beard, two leathers, a metal plus its highlight. The palette
   atlas is a 64x64 image of cells; adding cells is nearly free. Expect the cell count to roughly
   double, and report the new hex list.
3. **Spend detail on silhouette, not surface.** Beard locks that notch the outline; a stepped hem;
   stepped cuffs; shapes that survive being seen from ten metres.
4. **Hold the voxel height at 96 or come down toward the reference's ~60.** Your call with a
   render to justify it — but state which you did and why, and do not go up.
5. **Pose and limbs** — see decision A below. If A says V2, deliver him in the mp4's swing pose
   rather than a neutral stance, because that is what the reference is judged on.
6. **The lantern** — see decision B.
7. **Prove it at BOTH ends of the zoom, and make that a deliverable.** Alongside the five full-size
   views, commit a strip of the same asset rendered at **10 px, 30 px, 100 px and full height**
   (nearest-neighbour downscale of the full render is fine — it is the same filter the game's
   rasteriser approximates). Two questions it has to answer: does the silhouette still read as a
   bearded dwarf with a lantern at 10 px, and does any surface detail turn to noise there? **A
   detail that dissolves into speckle at 10 px is a defect, not a lost luxury** — with free zoom
   the camera will pass through every one of those sizes, and speckle that changes frame to frame
   shimmers.

## Hundreds of variations without modelling each one

This is the part with real design in it, and it wants deciding before any geometry is cut. Three
axes, cheapest first:

- **Palette swap — nearly free, and the pipeline already supports it.** The asset is one material
  plus one 64x64 atlas of palette cells. A recoloured dwarf is therefore **a new image of a few
  kilobytes with the geometry shared and untouched**. Tunic, hair, beard, skin and leather each
  become a variation axis at zero geometry cost. Build the atlas so a cell's MEANING is fixed by
  its coordinate — cell 3 is always "tunic mid" — and a variant is a table of hexes, not an edit.
- **A parts library with FIXED SOCKETS.** Author head, hair, beard, torso, arms, legs, boots,
  belt, pack and tool as separately assembled voxel groups that each attach at a **named anchor
  voxel coordinate declared in the generator**. Any head then fits any torso by construction. The
  generator already groups voxels by part (`groups=` in its FIGURES line), so this is a matter of
  making the anchors explicit and stable, not of restructuring.
- **Two or three scalars:** overall height, girth, beard length index.

Six heads x eight beards x four tunic palettes x three packs is 576 dwarves from about twenty
authored parts. **What makes it work is the sockets being declared and stable**; what breaks it is
a part that only fits the torso it was drawn against.

**One honest limit, and it is now roadmap rather than someday.** The client loads a single GLB for
`EntityKind::Dwarf` (`DWARF_SCENE_PATH`, `crates/gui/src/project.rs:266`) and hands every dwarf the
same material, so hundreds of *live* palette variants need per-instance materials in the client.
That is engine work and not this round's — but since the game is also the marketing renderer, it is
work that will happen, so **do not design the variation scheme around an offline assembly step.**
A variant must be expressible as data the client could pick at spawn: a part list plus a table of
hexes.

## Three decisions for Wolf — they change the CONTRACT, not just the model

The v1 contract in `check_asset.py` is explicit that it is v1: *"one-mesh/material/image clause
(V1 voxel assets only)"*. Round 3 as described does not fit it, and the checker should gain a v2
profile rather than have the clause quietly loosened.

- **A. One mesh, or separate limbs and a rig?** V1 requires exactly one mesh, one material, one
  image, one primitive, one node, no glTF extensions. The mp4's poses need separate limbs. Because
  the game is the renderer, **a V2 profile is not a render-only escape hatch -- it is the contract
  the game itself will load**, and the client's dwarf path is written against a single-node scene
  (the spawn transform, `apply_entity_blending` rewriting the translation every frame, the yaw-only
  facing rule). Ruling needed, and it is the expensive one: V2 with separate limbs now, or hold the
  single mesh this round and take posing when the animation work happens.
- **B. Emissive, or a real light?** V1 forbids emissive at any colour, and that rule came out of
  the game's lighting work. In the reference the lantern is the brightest thing in frame and throws
  a warm pool on the ground -- which a marketing shot taken from the game needs. The two are not
  the same fix: an emissive material makes the lantern GLOW, a **point light parented to the
  lantern** makes it LIGHT the scene, and only the first is asset work. Ruling needed on the asset
  half: does the flame cell get emissive or not.
- **C. It replaces the shipped asset, so the NAME has to be handled rather than inherited.** There
  is no second family any more -- the zoom answer settles that -- so this round overwrites
  `assets/gltf/SM_VoxelDwarf_Miner01.glb`. Keep the file name and **move the internal mesh name
  with the content**: this project has already shipped two different meshes both called
  `SM_VoxelPine_Tree02` and spent a story working out which binary was which. If the round produces
  a candidate rather than a replacement it stays in `src-assets/` under a name that says candidate,
  and never lands in `assets/gltf/`.

## Everything else in the standing brief holds unchanged

`src-assets/` only — never write outside it. `min Y = 0`, even width in X and Z, grid-aligned
positions, the greedy-meshed unwelded quad soup, the `FIGURES` line printed by the generator, and
the **byte-identical cold-run proof** as the finishing condition: the committed generator must
regenerate the committed GLB exactly, on a cold Blender, or the deliverable is the transcript
rather than the asset.

Render the five views with `render_dwarf.py` as usual, and confirm the flat pass still prints
`FLAT-CHECK all 10 palette colours reach the PNG exactly` — with your new cell count in place of
the 10.

## And report your cost

`session_tokens.py` in print mode against your own transcript, pasted verbatim, the model read from
your session banner, and the row labelled **`dev-art`**.
