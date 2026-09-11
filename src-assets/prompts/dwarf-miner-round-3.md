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
5. **Rig him to a skeleton with rigid weights, keeping ONE mesh** — ruled, see A below, including
   the one topology rule that makes it work (no quad may cross a joint). Deliver him in the mp4's
   swing pose as well as a neutral stance, because the swing is what the reference is judged on.
6. **The lantern's flame cell may be emissive** — ruled, see B. Author the lantern and the pickaxe
   as parts on hand sockets so they can become separate assets later without a remodel.
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
  **BEARD AND HAIR ARE THE FIRST SWAP AXES — Wolf named them, so design for them from the start.**
  *"just remember that for variants we need to have different beards, hair etc in the future"*.
  Concretely, for this round: the beard and the hair must each be **one part, mounted on one
  declared socket on the head, weighted to a single joint, and shaped so that removing it leaves a
  complete head underneath** — no skin voxels borrowed from the beard's volume, no hairline that
  only works with this hair. Round 2 called the beard the largest single read on the character, so
  it is also the highest-value axis: a dwarf with a different beard reads as a different dwarf at
  a fraction of the work. If you deliver only one beard and one hair this round, deliver them as
  if the second and third already existed.
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

## RULED by Wolf, 2026-09-11 — these are decisions, not questions

### A. A SKELETON with rigid weights. One mesh stays. No separate limb meshes, no soft skinning.

Wolf: *"A probably skeleton .. I think what matters is topology of polygons? I would not probably
create separate limbs? Unless we don't want soft body animation at all?"*

**Yes, topology is exactly what matters, and it is one specific property — not vertex density.**
Measured on the shipped GLB: 14,398 triangles over 28,796 vertices, which is 7,199 quads x 4
vertices each. **Nothing is welded between quads**; greedy meshing merges coplanar voxel faces into
big quads, so a single quad can span the whole outer arm.

That gives one hard rule and one honest trade:

- **THE RULE: no quad may cross a joint.** Greedy-merge *within* a part and never across parts, so
  every quad's four vertices belong to exactly one joint. A quad spanning shoulder to wrist cannot
  bend — it can only shear into a parallelogram, which is what a "bending" voxel arm looks like
  when it goes wrong. This is mechanically checkable and should be checked, not trusted.
- **Rigid weights: every vertex 1.0 to one joint, no blending.** That is not a downgrade from soft
  skinning, it is what the reference does — look at t=1/4/7/10 s and the limbs rotate as solid
  blocks. Soft weights on cubes smear the voxel read, which is the look we are buying.
- **You do NOT need separate limb meshes for this.** A skeleton deforms one mesh through vertex
  groups, so **one mesh, one material, one palette image all survive** — the GLB gains a `skin`
  with named joints (and later animation clips), and nothing else about the contract moves.
- **The trade, stated so it is a decision and not a surprise:** greedy meshing removes the interior
  vertices that soft deformation needs. Choosing greedy quads and rigid weights is choosing rigid
  animation. If a part ever wants genuine soft motion — a cloak, a beard sway — that part needs
  welded, denser topology and will stop reading as voxels. Nothing here forecloses it; it just will
  not be free later.

#### Welding the mesh: it is a topology question, and the topology says no

Wolf asked whether a welded mesh is a good idea regardless. It is a question about topology, not
about file size, so here is the topology answer first.

**A weld is only legal where two vertices agree on position, normal AND UV.** On a voxel surface
they almost never do: adjacent quads meet at a hard 90-degree edge, so their normals differ, and
the palette atlas gives each material its own cell, so a corner shared by a skin face and a tunic
face cannot carry one UV. **Measured on the shipped GLB rather than assumed: of 28,796 vertices,
the number sharing position AND normal AND UV with another is ZERO.** Greedy meshing already
performed every legal weld — merging coplanar same-material faces is exactly what it does — so
there is no welded version of this topology that keeps the look.

**Welding anyway, by position alone, costs three things and buys none of them back:**

- **It averages normals across hard voxel edges.** Crisp cube faces become smooth shading and the
  voxel read is gone. This is the whole look, not a detail.
- **It breaks the per-cell UVs**, so a welded corner can only belong to one material.
- **It makes rigid joints impossible.** At a joint seam the two sides currently hold separate
  vertices, so each follows its own bone and the seam opens and closes cleanly. One welded vertex
  can follow only one bone, and it drags the other part with it.

So the unwelded quad soup is not a legacy compromise to be tidied up — **it is the property that
makes rigid voxel joints work at all.** Weld only if the look becomes smooth-shaded, which is a
different game, and then weld per-part rather than wholesale.

*(The size angle, for completeness and not as the argument: welding by position would collapse
28,796 vertices to 9,108, roughly 615 KiB of a 986 KiB file. The asset is essentially all vertex
data — the palette PNG is 188 bytes, which is separately why palette-swap variants are nearly
free.)*

#### `check_asset.py` WILL REJECT a rigged deliverable, and that is expected

The v1 contract requires one node and no glTF extensions; a skin adds joint nodes. So the moment
you add a skeleton, `scripts/bench/check_asset.py` fails with the one-mesh/node clause. **That is
the checker being right about a contract that has not caught up yet, not a defect in your asset.**

What to do: run it anyway, **paste the exact failure text into your report**, and carry on. Do NOT
loosen the checker — it lives outside `src-assets/` and this round may not write there — and do not
strip the skin to make it pass. The V2 profile is owed work on this side and it is tracked; a
workaround smuggled in from the art seat is how a contract quietly stops meaning anything.

Everything the checker *can* still tell you remains worth having, so report its figures if it gets
far enough to print them: size, `min Y`, centring, the palette list, triangle and vertex counts.

**Name the joints now and keep them stable**, because a skeleton's real payoff is that one
animation clip drives every variant: `root, hips, spine, chest, neck, head, shoulder.L/R,
elbow.L/R, hand.L/R, hip.L/R, knee.L/R, foot.L/R`, plus `beard` if you want it to move. Those names
are the contract between this asset and every future one.

### B. Emissive on the flame cell: YES. Items become separate assets later.

Wolf: *"B yes .. and lanter and other items should be separate in the future at least"*

So the flame cell may be emissive in this round. And because the lantern and the pickaxe are going
to be split into their own assets later, **author them now as their own parts on declared hand
sockets** — `hand.L` holds the lantern, `hand.R` the pickaxe — so splitting them out is a file
move rather than a remodel. Do not fuse a tool into the torso geometry.

**Lighting is explicitly not your problem this round** (Wolf: *"no need to worry about lighting etc
.. just model first"*). An emissive flame makes the lantern glow; making it *light the scene* is a
point light in the client, and that is engine work for another day.

### C. The naming scheme — proposed here, correct it if you disagree

The problem being solved: this project has shipped two different meshes both called
`SM_VoxelPine_Tree02`, and a stale binary carrying the right internal name authenticates itself.
Today the dwarf's GLB holds node `SM_VoxelDwarf_Miner01`, mesh `SM_VoxelDwarf_Miner01`, material
`M_VoxelDwarf`, image `T_VoxelDwarf_Palette` — **nothing in it says which round produced it.**

1. **The file path is a SLOT and does not churn.** `assets/gltf/SM_VoxelDwarf_Miner01.glb` stays;
   the client loads it by path (`DWARF_SCENE_PATH`). `01` denotes the ROLE, not a version — miner
   as opposed to a future smith. Never bump it to `Miner02` to mean "second attempt".
2. **Internal names carry the revision, because they are what a stale binary shows:** node and mesh
   `SM_VoxelDwarf_Miner01_r3`, material `M_VoxelDwarf_r3`, image `T_VoxelDwarf_Palette_r3`. Bump
   `rN` on every authored round that changes content.
3. **The generator declares the same token** (`REVISION = "r3"`) and prints it in its `FIGURES`
   line, so the source and the binary can be compared without opening Blender.
4. **`check_asset.py` asserts they agree** — every internal name ends with the same `rN`, and it
   matches the generator's constant. A stale GLB then FAILS the checker instead of passing it.
5. **Candidates never enter `assets/gltf/`.** They live at
   `src-assets/candidates/SM_VoxelDwarf_Miner01_r3.glb`; promotion is a copy plus the revision bump.
6. **Variants get no file names at all.** A palette variant is a table of hexes; a part-swapped
   variant is a manifest entry. Only the skeleton and the parts library are files — otherwise
   "hundreds of variations" becomes hundreds of binaries nobody can tell apart, which is the
   original defect at scale.
## Everything else in the standing brief holds unchanged

`src-assets/` only — never write outside it. `min Y = 0`, even width in X and Z, grid-aligned
positions, one mesh / one material / one palette image, the greedy-meshed unwelded quad soup —
**now merged within a part only, never across a joint** — the `FIGURES` line printed by the
generator, and the **byte-identical cold-run proof** as the finishing condition: the committed generator must
regenerate the committed GLB exactly, on a cold Blender, or the deliverable is the transcript
rather than the asset.

Render the five views with `render_dwarf.py` as usual, and confirm the flat pass still prints
`FLAT-CHECK all 10 palette colours reach the PNG exactly` — with your new cell count in place of
the 10.

## And report your cost

`session_tokens.py` in print mode against your own transcript, pasted verbatim, the model read from
your session banner, and the row labelled **`dev-art`**.
