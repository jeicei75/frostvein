# Round 18 brief — the walk cycle

**For:** a Claude Code session with the Blender MCP server attached, Blender **5.2.1**, GPU
Cycles, driving the **running** Blender instance. Wolf watches the viewport.
**Produces:** one looping `Walk` action on the r17 armature, plus the stride figure the client
needs to play it without skating.
**You write only inside `src-assets/`.**

Start from the committed figure: `src-assets/blender/SM_VoxelDwarf_Miner01.blend` — r17 after
round 17's seam fix. 21 parts, 3360 triangles, 19 joints, rigid one-joint-per-vertex weights
assigned by BOX. It regenerates exactly from `dwarf_r17.py`; keep that true.

---

## 0. What the pipeline already supports, so you are not guessing

| stage | state |
|---|---|
| skinning | **shipping** — `export_skins=True`, 19 joints, 0 unweighted, 0 soft-weighted verts |
| `export_dwarf.py` | `export_animations=False` at line 661. Flipping it exports **every** action in the file |
| `check_asset.py` | **no animation or joint clauses at all** — a missing, empty or mis-named clip is invisible to every gate we have |
| engine | **supported** — Bevy 0.19 with `bevy_animation` in the graph and `bevy_gltf` built with its `bevy_animation` feature |
| client wiring | **absent** — no `AnimationPlayer`/`AnimationGraph` in `crates/gui`; the dwarf loads as `#Scene0` and stands still |
| what moves dwarves | `blend.rs::blended_translation(previous, current, factor)` lerps him between integer cell positions each tick |

The engine half needs nothing. The client wiring, the exporter change and the new
`check_asset.py` clauses are **the forge side's job, not yours** — you author and export.

## 1. The one rule that breaks everything if you miss it

**The cycle is IN PLACE. No net horizontal displacement on any bone, including `root`.**

The client positions the dwarf itself: `blended_translation()` lerps him between cells every
tick and `entity_draw_offset` drops him to the cell floor. A clip that also walks him forward
makes him travel at double speed and skate. This codebase has already lost a session to
translation having two writers — do not add a third.

A **vertical bob on `root` is wanted** and does not conflict; it rides under the client's
transform. Horizontal is what is forbidden.

## 2. The number the code half needs from you

**Stride length: how many metres one full cycle would advance him, if it advanced him.**

This is the handoff. The Rust side divides the dwarf's actual speed by it to set the playback
rate; wrong or missing, the feet slide. Derive it from the planted foot — during stance the
contact foot must stay **world-fixed**, so measure how far the hips travel across one stance
phase, double it for two steps, and report it in metres to three decimals.

One world cell is 1.0 m and the dwarf is 1.2 m tall, so a stride near 0.8–1.0 m per two steps
is the plausible band. **Report what you measured, not what you aimed at.**

## 3. What to author

One action, named exactly **`Walk`**, on the armature. 24 fps, a whole number of frames, first
and last frame identical so it loops seamlessly — author the loop, then check the exported
channel for a doubled final key.

Four standard key poses, mirrored half a cycle apart: **contact, down, passing, up.**

The rig is rigid boxes. There is no squash, no stretch, no deformation — every bit of life has
to come out of rotation, so the poses have to be read, not eyeballed.

Joints available: `root`, `hips`, `spine`, `chest`, `neck`, `head`, `beard`, `shoulder.L/R`,
`elbow.L/R`, `hand.L/R`, `hip.L/R`, `knee.L/R`, `foot.L/R`.

- **Legs carry it** — hip swing, knee flex on the passing leg, ankle roll through contact.
- **Arms counter-swing** — but the right hand holds a **1.04 H pickaxe** and the left a
  lantern, both parented to their hand. A swing that reads fine on a bare arm will drive the
  pickaxe through the floor or the head. **Check the props at every pose.**
  Known and accepted: the pickaxe already overshoots the figure (z 0.230..1.322 against a
  1.2 m dwarf) and is being fixed separately. Author around it; do not fix it here, and do not
  let it silently clip.
- **Hips and chest counter-rotate** slightly; `spine`/`neck` keep the head level.
- **Beard** may lag a frame or two for life. Small.

A dwarf should read heavy: short stride, low bob, weight settling onto the contact foot.

## 4. Constraints

- **Geometry is frozen.** No box moves, no part is added. If the cycle exposes a geometry
  problem, **report it, do not fix it** — the model is being revised separately.
- Do not touch the colour path. `hex_rgb` must stay free of `srgb_to_linear` (#94, and it has
  already been regressed once by a round forking from the wrong parent).
- Leave the armature at **bind** and save from bind. The action is data on the armature; the
  saved rest state must still be the bind pose.
- If you flip `export_animations` to test, make sure `Walk` is the **only** action in the file
  or the GLB gains clips nobody asked for. Say in your report which you did — **the exporter
  change itself is the forge side's to land, not yours.**

## 5. How to prove it

You cannot run the game or the gate from your machine, so prove it where you are.

1. **Foot-slide check, numerically.** For each frame of stance, print the world position of the
   contact foot's sole. It must not move more than a millimetre or two. **This is the check
   that matters most** — a cycle that looks right and slides here will skate in game.
2. **Prop clearance.** Per frame: the pickaxe's and lantern's lowest world z, and their minimum
   distance to the head. Nothing below 0, nothing intersecting the skull.
3. **In-place proof.** `root`'s world x and y identical on every frame. Print them.
4. **Loop seam.** Frame 1 and the wrap frame produce identical bone rotations.
5. **Watch it.** Play the cycle at speed in the viewport from front, side and three-quarter,
   and render a strip of the four key poses to `src-assets/renders/r18/`. Screenshot as you go
   — Wolf judges this by eye, and a walk that measures perfectly can still read wrong.
6. **Re-run `seam_check.py`** with the walk's own extremes added to its ranges. The seams closed
   in round 17 must stay closed through the poses this cycle actually uses, not just the
   generic ROM ranges.

## 6. Venue notes

- Cycles on the GPU for everything. A Cycles **rendered-viewport** screenshot over MCP comes
  back black (grabbed before convergence) — render to a file and read that; use MATERIAL
  shading for live looking.
- Pose bones default to `rotation_mode = 'QUATERNION'`, so setting `rotation_euler` is
  **silently ignored**: the figure stays at bind while every number looks right. Set the mode
  first. This cost round 17 a build.
- The dwarf faces **+Y**. A camera at −Y renders the backpack and reads as a broken figure.
- **Bounding boxes lie on this figure.** `r17_torso`'s bounds say its back is −0.1794 while its
  back face above z 0.78 is −0.1614. Take extents by ray-casting faces at the height you care
  about.
- An RGB diff needs its noise floor: two renders of identical geometry at different seeds
  differed by 201,680 px. Diff the **alpha** for silhouette work.

## 7. Reporting

`src-assets/prompts/dwarf-miner-round-18-report.md`. State the **stride in metres**, the frame
count and fps, the foot-slide table, the prop-clearance table, and anything you could not
verify. **Do not report a gate you did not run.**

If the rig turns out not to support a readable walk without geometry changes, **stop and say
so** — that is Wolf's call, not yours.
