# Round 19 brief — the work clips: Dig and Carry

**For:** a Claude Code session with the Blender MCP server attached, Blender **5.2.1**, GPU
Cycles, driving Wolf's **running** Blender instance. Wolf watches the viewport.
**Produces:** two looping actions on the r17 armature, named exactly **`Dig`** and **`Carry`**,
plus the one number the client needs to put a stone in the dwarf's hands.
**You write only inside `src-assets/`.**

Start from the committed figure: `src-assets/blender/SM_VoxelDwarf_Miner01.blend` — r17 with
round 18's `Walk`. 21 parts, 3360 triangles, 19 joints, rigid one-joint-per-vertex weights. It
regenerates from `dwarf_r17.py` + `walk_r18.py`; keep that true, and make `work_r19.py` the third
link in that chain.

---

## How this round is run (live, watchable)

- **Author in the running Blender, over MCP.** Do not spawn `blender --background` to author
  anything. One background run is allowed at the very end: the cold-run regeneration proof
  (§6, step 7).
- **One tool call per pose stage, with a screenshot each,** committed under
  `src-assets/renders/r19/stages/`. Wolf is watching; a single call that builds the whole clip
  shows him nothing until it is over.
- **Write the generator as you build.** `src-assets/blender/work_r19.py` grows stage by stage,
  so the final file rebuilds exactly what Wolf watched. Keep the `walk_r18.py` shape: `build_dig()`,
  `build_carry()`, `checks()`, `detach()`, `attach(name)`.

---

## 0. What the pipeline already supports, so you are not guessing

| stage | state |
|---|---|
| `Walk` | **shipping** — `walk_r18.py`, frames 0..24 at 24 fps, stride **0.4926 m**, foot slide 0.000 mm |
| `export_dwarf.py` | `export_animations=True` — it exports **every** action in the file. Nothing stray may be in it |
| `check_asset.py` | animation clauses exist: it prints `anims=Name:Nch@Ts` per clip and checks loop closure on each |
| client wiring | today it binds clip **index 0** and assumes it is `Walk`. The forge side is moving it to bind **by name** this story; **the names `Walk`, `Dig`, `Carry` are the contract** |
| what drives `Walk`/`Carry` | phase locked to ground covered: `distance / 0.4926 m`, player held paused, `seek_to` each frame |
| what drives `Dig` | phase = sim ticks since he started working, over **5 ticks**: one cycle per dig |
| what moves him | the client, between cells. Yaw too: while digging he is turned to face the target |

The export, promotion and client wiring are **the forge side's job, not yours** — you author.

## 1. The rules that break everything if you miss them

**In place.** No net horizontal displacement on any bone, including `root`. The client owns his
world position; a clip that moves him gives translation a second writer. A vertical bob on
`root` is fine.

**Frames 0..24, never 1..25.** The exporter writes time as `frame / fps`; a 1..25 action exports
as 0.0417..1.0417 s and hitches once per loop. Frame 24 == frame 0, LINEAR keys on every bone on
every frame (round 18 §2 and §6 explain why).

**Saved unassigned.** Leave the armature at bind with `animation_data.action = None`, every
action on a fake user. `Walk` is left **untouched** — its fcurves must hash identical before and
after this round.

**Measure the bound clip, not a static figure.** Anything that evaluates the depsgraph with no
action assigned measures bind. Before every check bind the action **and its slot**:
`ad.action = act; ad.action_slot = act.slots[0]`. With three actions in the file now, a missing
slot assignment is the likely way to measure the wrong one.

**Blender 5.2 actions are slotted.** `action.fcurves` does not exist; go through
`walk_r18.fcurves(act)` (layers → strips → channelbags). Pose bones default to QUATERNION, so a
`rotation_euler` write is silently ignored — set the mode first.

## 2. `Dig` — one pick swing per cycle

- **Duration in game: 0.5 s at Normal speed** (5 ticks of 100 ms), faster under fast-forward.
  Author 0..24 at 24 fps as usual; the client scales one cycle onto one 5-tick work run. So
  author for a **half-second** swing: wind-up, strike, recover, no idle hold.
- **The pick strikes at about frame 18** (phase ≈ 0.75), just before the sim changes the tile.
- **Direction:** along +Y, the direction `Walk` walks — the client turns him to face the target.
- **Where the strike lands — the geometry, measured from the code:** a world cell is **1.6 m**
  (`METRES_TO_CELLS = 0.625`) and he stands at his cell's centre. For a **Dig**, the rock face he
  works is the near face of the next cell: **0.8 m in front of his origin**, from floor height up.
  For a **Channel** he digs the floor of the cell he stands in, so the strike lands at his feet.
  One clip serves both: strike the floor-to-knee band roughly **0.5–0.8 m ahead**. Report where
  the pick head actually is at the strike frame (rig-space metres).
- **The pickaxe (`r17_pickaxe` on `hand.R`) is 1.04 H and overshoots the figure** (z 0.230..1.322
  on a 1.2 m dwarf; round 18 §3). A two-handed overhead swing rotates that shaft through his head
  and the floor. **Clearance-check the pick at every frame:** lowest z (nothing below 0) and
  nearest approach to the head assembly (nothing touching the skull). Round 18 §5 shows the
  table shape and warns that **shoulder roll (Y) or a wrist twist** is the axis that drives the
  pick into the skull — a swing will need some of both, so watch that column.
- The lantern stays on `hand.L`. If the swing wants two hands, the lantern hand may come up to
  the shaft — report it; do not reparent anything.
- Legs: planted. A step into the swing must return the feet to the same place by frame 24, and
  no foot may slide while planted (print the sole positions as round 18 did).

## 3. `Carry` — walking with a stone in both hands

- **Legs and root: `Walk`'s keys copied verbatim.** Same frames, same values, same stride. The
  client drives `Carry` off the same ground-covered phase as `Walk`, so a carry with a different
  stride skates. **Foot slide must read 0.000 mm, exactly as Walk does.**
- **Arms:** both hands hold the stone in front of the chest, the walk's arm swing replaced by a
  braced hold (a little bounce with the bob is life; a swing is not).
- **The stone he holds is drawn by the client as a cube 0.4 cells on a side = 0.64 m** — over
  half his 1.2 m height. Pose the hands around a 0.64 m cube, check in a viewport with a
  temporary 0.64 m cube parented at the hand-meeting point (do **not** save it), and if a cube
  that size cannot read as held on this rig, **say so** with a screenshot — scaling the carried
  stone is a one-constant client change and Wolf's call.
- The pickaxe and lantern stay on their hands. Clearance-check both props against the head and
  the floor at every frame, and against the held cube's volume.
- **Report `CARRY_OFFSET`:** the point midway between the two palms, averaged over the cycle, in
  rig-space metres (x right, y forward, z up, origin at the feet), to three decimals. The client
  parents the stone there. Also report how far the meeting point moves through the cycle (mm),
  since the client holds the stone at a fixed offset.

## 4. Constraints

- **Geometry is frozen.** No box moves, no part is added, no prop is re-parented. If a clip
  exposes a geometry problem, report it — do not fix it.
- Do not touch the colour path: `hex_rgb` stays free of `srgb_to_linear` (#94).
- `Walk` untouched (hash its fcurves before and after; report both).
- Exactly three actions in the file at the end: `Walk`, `Dig`, `Carry`. A stray action becomes
  a stray clip in the GLB.

## 5. Not in this round

- No pick-up or drop clip — a hauler's 5-tick pick-up and drop show `Walk`/`Carry`.
- No cut (woodcutter) clip — that is a later story.
- No blending or transitions — the client plays one clip at a time.

## 6. How to prove it

1. **Foot slide** on both clips, round 18's table shape. `Carry`: 0.000 mm. `Dig`: planted feet
   do not move.
2. **Prop clearance** per frame on both clips: pickaxe and lantern lowest z, nearest approach to
   the head; on `Carry` also to the held cube.
3. **In place:** `root` world x and y constant on every frame of both clips.
4. **Loop seam:** frame 0 vs frame 24 bit-identical on both.
5. **Seams:** `seam_walk()`'s method at the real posed frames of both clips (≤ 10 mm).
6. **Watch it.** Play each at game speed in MATERIAL shading — `Dig` at **0.5 s per cycle**, not
   1 s — from the side, front and three-quarter. Render key strips to `src-assets/renders/r19/`.
   The dwarf faces **+Y**; a camera at −Y shows the backpack.
7. **Cold-run regeneration:** one background run of `dwarf_r17.py` → `walk_r18.py` →
   `work_r19.py` reproduces the saved `Dig` and `Carry` fcurves exactly. Report the hashes.

## 7. Venue notes

Everything in round 18 §6 still holds: a Cycles rendered-viewport screenshot comes back black
(render to a file); bounding boxes lie on this figure (ray-cast extents); diff alpha, not RGB.

## 8. Reporting

`src-assets/prompts/dwarf-miner-round-19-report.md`, stating:

- frames and fps of each clip; the **strike frame** and where the pick head is at it;
- **`CARRY_OFFSET`** (rig-space metres, 3 decimals) and how far the hand-meeting point moves;
- the foot-slide tables (Carry must match Walk's 0.000 mm), the prop-clearance tables, in-place,
  loop seam and seam results;
- `Walk`'s fcurve hash before and after;
- the action list in the saved file (exactly `Carry`, `Dig`, `Walk`);
- **the session's exact model id, cost and turns** — they go in the story's art ledger;
- anything you could not verify. **Do not report a gate you did not run.**

If the rig cannot make a readable swing or hold without geometry changes, **stop and say so** —
that is Wolf's call, not yours.
