# Round 20 brief — the work clip: Cut

**For:** a Claude Code session with the Blender MCP server attached, Blender **5.2.1**, GPU
Cycles, driving Wolf's **running** Blender instance. Wolf watches the viewport.
**Produces:** one looping action on the r17 armature, named exactly **`Cut`**: a woodcutter
chopping at a tree trunk.
**You write only inside `src-assets/`.**

Start from the committed figure: `src-assets/blender/SM_VoxelDwarf_Miner01.blend`, which is r17
with round 18's `Walk` and round 19's `Dig` and `Carry`. It has 21 parts, 3360 triangles and
**20 joints** (round 19 added `pick`, a child of `hand.R`). Weights are rigid, one joint per
vertex. The file regenerates from `dwarf_r17.py` → `walk_r18.py` → `work_r19.py`. Keep that
true, and make **`cut_r20.py`** the fourth link in the chain.

---

## How this round is run (live, watchable)

- **Author in the running Blender, over MCP.** Do not spawn `blender --background` to author
  anything. One background run is allowed, at the very end: the cold-run regeneration proof
  (§6, step 7).
- **One tool call per pose stage, with a screenshot of each,** committed under
  `src-assets/renders/r20/stages/`. Wolf is watching. A single call that builds the whole clip
  shows him nothing until it is over.
- **Write the generator as you build.** `src-assets/blender/cut_r20.py` grows stage by stage,
  so the final file rebuilds exactly what Wolf watched. Follow `work_r19.py`'s shape:
  `build_cut()`, `checks()`, `detach()`, `attach(name)`. Reuse `walk_r18.fcurves(act)` and
  round 19's keyer and hashing; do not copy them.

---

## 0. What the pipeline already supports, so you are not guessing

| stage | state |
|---|---|
| `Walk`, `Dig`, `Carry` | **shipping**, rounds 18-19. Frames 0..24 at 24 fps, LINEAR, loop-closed |
| `export_dwarf.py` | `export_animations=True`: it exports **every** action in the file. Nothing stray may be in it |
| `check_asset.py` | prints `anims=Name:Nch@Ts` per clip and checks loop closure on **every** clip, so `Cut` must loop |
| client wiring | binds clips **by name**. **`Cut` is the contract name.** If it is absent, the client plays `Dig` for a cut and logs `clip Cut ABSENT` |
| what drives `Cut` | the phase is the number of sim ticks since he started work, over **10 ticks** (Wolf, 12.8 Task 0.2): one cycle per chop, **5 chops per tree** |
| what moves him | the client, between cells. Yaw too: while he cuts he is turned to face the trunk, as a miner faces rock |

Export, promotion and client wiring are **the forge side's job, not yours**. You author.

## 1. The rules that break everything if you miss them

Round 19 §1 still holds, word for word:

- **In place.** No net horizontal displacement on any bone, including `root`. A vertical bob on
  `root` is fine.
- **Frames 0..24, never 1..25.** Frame 24 == frame 0, with LINEAR keys on every bone on every
  frame, **including `pick`**. In Blender an un-keyed bone keeps its last pose. Round 19 hit
  exactly this when `Walk` had no `pick` curve.
- **Saved unassigned.** Leave the armature at bind with `animation_data.action = None`, and put
  every action on a fake user.
- **Measure the bound clip, not a static figure.** Before every check, bind the action **and its
  slot** (`ad.action = act; ad.action_slot = act.slots[0]`). With four actions in the file, a
  missing slot assignment is the likely way to measure the wrong one.
- **Blender 5.2 actions are slotted**: go through `walk_r18.fcurves(act)`. Pose bones default to
  QUATERNION, so set the rotation mode before writing `rotation_euler`.

**`Walk`, `Dig` and `Carry` are untouched.** Their fcurve hashes must match round 19's §8
before and after this round:

| action | round 19 hash |
|---|---|
| `Walk` (all curves, incl. `pick` rest) | `b1079d210fb881bdcc87ee2a2d4992b4c3b0a6572707339ff86ac2907950d3ea` |
| `Dig` | `844687a11997b6826d097eee23bb3332c3838be56e26b5666038da56c472b962` |
| `Carry` | `c50f8269451166101c9dc64f18fc93e9a4be8584a3f73d2d6f39b1439a175bde` |

## 2. `Cut` — one chop per cycle

- **Duration in game: 1.0 s at Normal speed** (10 ticks of 100 ms), and faster under
  fast-forward. Authored 0..24 at 24 fps, this is the **first clip whose authored speed is its
  game speed at Normal**. It is twice as long as `Dig`'s half-second. Wolf's reason for 10 ticks:
  **an axe reads heavier than a pick.** Spend the extra time on weight: a fuller wind-up and a
  committed follow-through. Do not add an idle hold.
- **The blade bites at about frame 18** (phase ≈ 0.75), as `Dig` does.
- **It must not read as `Dig`.** `Dig` is an overhead strike down at the floor ahead. A chop
  swings **across**, into a **vertical trunk** at about waist-to-chest height: a diagonal or
  near-horizontal blow from over one shoulder. Seen from the seat camera, the difference is the
  whole point of the clip.
- **Direction:** along +Y, the direction `Walk` walks. The client turns him to face the trunk.
- **Where the trunk is, measured from the code.** A world cell is **1.6 m**
  (`METRES_TO_CELLS = 0.625`), and he stands at the centre of a cell next to the tree's trunk
  cell. The trunk cell's near face is **0.8 m in front of his origin**. The pine **mesh's**
  visible trunk is narrower than its cell. Measure it: import one `assets/trees/*.glb`
  temporarily, read the trunk's radius and the height of its lowest branches, then delete it.
  **Do not save it.** Aim the bite at the visible bark in front of him, and report where the
  blade edge is at frame 18 (rig-space metres) and where that bark is.
- **The tool: you and Wolf decide at the seat, and the report says which and why.**
  - **(A) Re-use the pickaxe.** No geometry change, and `pick` already moves the pickaxe
    independently of `hand.R`. It is cheaper, but it may read as "digging the tree".
  - **(B) An axe.** This is a new part and probably a 21st joint (`axe`). It is visible in
    **every** clip, so `Walk`, `Dig` and `Carry` would need rest keys for it. Round 19 set the
    precedent with `pick`: rest keys on a new joint only, and every old curve byte-identical.
    You would also need to say where the axe rides when he is not cutting, and where the pickaxe
    goes while he chops. **This changes the figure and the GLB's joint count, so it needs Wolf's
    explicit yes before you build it.** If he says yes, add the joint to `export_dwarf.JOINTS`
    as round 19 did, and keep geometry hashes for every other part unchanged.
- **Two-handed.** The lantern is on `hand.L`. If the swing wants both hands on the haft, the
  lantern hand comes to the shaft: report it, and do not reparent anything. Round 19 found palm
  reach of **0.302 m** from the shoulder head and documented its two-arm solver; use them.
- **Clearance at every frame:** the tool's lowest z (nothing below 0) and its nearest approach
  to the head assembly (nothing touching the skull). Shoulder roll and wrist twist drive a
  shaft into the skull (round 18 §5), and a cross-body chop needs both, so watch that column.
- **Legs planted.** A step into the swing must put the feet back by frame 24, and no planted
  foot may slide. Print the sole positions as rounds 18 and 19 did.

## 3. Constraints

- **Geometry is frozen** except option (B)'s axe, and only with Wolf's yes. No box moves, and
  no existing part is added to or re-parented. If the clip exposes a geometry problem, report
  it; do not fix it.
- Do not touch the colour path: `hex_rgb` stays free of `srgb_to_linear` (#94).
- **Exactly four actions in the file at the end: `Carry`, `Cut`, `Dig`, `Walk`.** A stray
  action becomes a stray clip in the GLB.

## 4. Not in this round

- No felling animation, no falling tree, no log pick-up. The tree vanishes on the sim's tick,
  and a hauler carries the logs with `Carry`.
- No retiming of `Walk`, `Dig` or `Carry`.
- No blending or transitions. The client plays one clip at a time.

## 5. What the client does with it (so you can judge it as the game will)

The client plays `Cut` only once he is **drawn** at his work cell, facing the trunk. The swing
is anchored to the tick he started work, so one tree is exactly five cycles, frame 0 → 24 five
times, and then he walks. Judge the loop as five chops in a row, not one.

## 6. How to prove it

1. **Foot slide:** planted feet do not move. Use round 19's table shape.
2. **Tool clearance per frame:** lowest z and nearest approach to the head. For (B), also check
   the stowed pickaxe or axe against the body.
3. **In place:** `root` world x and y are constant on every frame.
4. **Loop seam:** frame 0 and frame 24 are bit-identical on all joints.
5. **Seams:** `seam_walk()`'s method at the real posed frames (≤ 10 mm).
6. **Watch it.** Play it at game speed (**1.0 s per cycle**) in MATERIAL shading, five loops,
   from the side, the front and three-quarter, with a temporary trunk-sized cylinder 0.8 m ahead
   (**do not save it**). Render key strips to `src-assets/renders/r20/`. The dwarf faces **+Y**.
   Show Wolf `Dig` and `Cut` one after the other, so he can judge that they differ.
7. **Cold-run regeneration:** one background run of `dwarf_r17.py` → `walk_r18.py` →
   `work_r19.py` → `cut_r20.py` reproduces the saved `Cut` fcurves exactly, and leaves the three
   older hashes unchanged. Report the hashes.

## 7. Venue notes

Round 18 §6 and round 19 still hold: a screenshot of the Cycles rendered viewport comes back
black, so render to a file; bounding boxes lie on this figure, so use ray-cast extents; and
diff alpha, not RGB.

## 8. Reporting

Write `src-assets/prompts/dwarf-miner-round-20-report.md`, stating:

- frames and fps; the **bite frame**; where the blade edge is at it, and where the bark it
  meets is;
- the tool decision (A or B), with Wolf's words if he ruled;
- the foot-slide, clearance, in-place, loop seam and seam results;
- the hashes of `Walk`, `Dig` and `Carry` before and after, and of `Cut`;
- the joint count, and the action list in the saved file (exactly `Carry`, `Cut`, `Dig`, `Walk`);
- **the session's exact model id, cost and turns**, which go in the story's art ledger;
- anything you could not verify. **Do not report a gate you did not run.**

If the rig cannot make a readable chop without geometry changes beyond option (B), **stop and
say so.** That is Wolf's call, not yours.
