# Round 20 report — the work clip `Cut`, and a proper axe

**Delivered:** a looping action **`Cut`** on the r17 armature, and option **(B)**: a new
part, `r17_axe`, on a 21st joint, `axe`. Both are in `src-assets/blender/SM_VoxelDwarf_Miner01.blend`,
which regenerates from `dwarf_r17.py` → `walk_r18.py` → `work_r19.py` → **`cut_r20.py`**. The
cold run proves it (§9).

| | `Cut` |
|---|---|
| frames / fps | 0 … 24 at 24 fps, frame 24 ≡ frame 0. **1.0 s per chop at Normal** (10 ticks); five cycles fell a tree |
| keys | 93 fcurves, 2325 keys, all LINEAR, one slot, fake user |
| **bite** | **frame 18** (phase 0.750). The edge lies flat in the trunk's **right side face**, an even 12–15 mm deep, at z 0.59–0.65 |
| edge while high | faces **forward** (frames 3–14), rolls **left** on the downswing (15–17), as Wolf asked |
| tool | **(B) an axe**: Wolf's words, "option B with proper axe". **One-handed**: two hands are out of this rig's reach (§1) |
| foot slide | **0.0001 mm** (planted; float noise) |
| in place | root x/y span **0.000e+00 m** |
| loop seam | **bit-identical**, all 21 joints |
| seams | **one pair over 10 mm**: sleeve.R/torso **12.3 mm** at frame 2, not visible in close-ups. A construction limit of the r17 shoulder (§7) |
| clearance | **PASS** in all four clips. Nothing below the floor, nothing touching the skull, no intersecting faces |
| `Walk` / `Dig` / `Carry` | their **20-joint curves hash exactly as round 19's §8**, before and after |

**Read these three first. Each one needs action from the forge side.**

1. **`CUT_OFFSET = 0.35 m`.** Wolf moved the bite (§2): `Cut` is authored against bark
   **0.95 m** in front of him, but the client stands him where the bark is **1.30 m** away.
   The client should draw him **0.35 m (0.219 cells) toward the trunk while he cuts**, along
   the yaw it already applies. Without the nudge, the blade bites air 0.35 m short of the bark.
2. **The GLB will have 21 joints and 22 parts** (3456 triangles, up from 3360). `"axe"` is in
   `export_dwarf.JOINTS`. Nothing under `crates/` or the tests asserts a joint or part count;
   I searched.
3. **`work_r19.key_clip` gained two optional parameters** (`bones`, `located`). The defaults
   are exactly the old behaviour, and the cold run regenerates `Dig` and `Carry` byte-identical.

---

## 1. The tool: (B), and where everything rides

**Wolf ruled (B) at the start of the session: "option B with proper axe".**

**The axe.** It is a bearded, single-bit felling axe, 0.87 m overall: a 0.74 m haft, a wrapped
grip, a metal ferrule, and a head with an eye, a hammer poll, a cheek, a bearded blade and a
bright bit. That is 8 boxes, 64 verts and 96 tris, painted with `dwarf_r17`'s own atlas and
cell rule (wood haft, trunk-brown wrap, metal head). It was first built at the pickaxe's
1.05 m to reach the real bark at 1.30 m. Once Wolf moved the bite to 0.95 m (§2), I cut it
down, because at the old length it forced the fist out sideways.

**The joint.** `axe` is a **child of `chest`**, and the axe's single vertex group is `axe`
(rigid, 1.0).

- **When he is not cutting, the axe is strapped to the left side of the pack.** The haft lies
  flat on the pack's left face (ray-cast at x −0.185…−0.190; 4 mm overlap) and runs diagonally
  up and back along it. The head stands up behind his left shoulder, with the edge pointing
  back, away from the skull. Because the joint is the chest's child, its rest pose rides with
  the pack, so `Walk`, `Dig` and `Carry` need only **constant rest keys** on `axe`. This is
  round 19's `pick` precedent: rest keys on the new joint only, and every old curve
  byte-identical.
- **When he cuts, the axe is in the right fist** and the **pickaxe rides on his back**, in
  exactly `Carry`'s sling (`work_r19.sling_matrix`). Both are pinned through their own joints
  every frame.

Two stow positions were rejected by the clearance table, not by eye:

- **Vertical, mid-depth** (`stages/cut-01`, `cut-02`). `Dig`'s forward lean tipped the axe's
  head into the hair: 1.4 mm, with faces intersecting at frames 18–22. Leaning it back 14.5°
  moved the head about 10 cm off the skull.
- **Foot at z 0.40.** `Dig`'s lean-back (chest + spine +13°) swung the foot into the skirt,
  with faces intersecting at frames 4–15. It now rides 10 cm higher.

**One-handed, and the lantern hand stays on the lantern; nothing reparented.** After watching
it, Wolf said "ofc it would be more convincing if it would use both hands", and then "but maybe
the model is not allowing that now". I measured it, and it does not:

- the left arm reaches **0.35 m** from the shoulder head to the fist centre;
- the nearest any graspable part of the haft comes to the left shoulder is **0.62 m**, at the
  top of the wind-up (frame 8), and **0.76 m** at the bite;
- **no other swing fixes it.** Two fists on one haft must meet close together in front of
  him. The shoulders are 0.55 m apart and the chest and beard stand out to y 0.20–0.26, so
  with 0.35 m arms the only place they can meet is inside his chest. This is round 19's
  `Carry` limit again: his hands only reach things held about shoulder-width apart;
- standing him closer to the trunk does not help: the limit is where the hands can meet, not
  how far away the bark is.

Wolf's call: **ship one-handed now** ("we can use this now.. have to rethink the model and
rig anyway at some point"). A two-handed chop needs longer arms. That is a geometry change
beyond (B), which re-poses every clip. The lantern arm swings for balance: forward on the
wind-up, back on the bite.

## 2. The bite, and where the bark is

**The trunk, measured.** I imported all four `assets/trees/*.glb` temporarily, measured them,
deleted them (orphan textures purged) and did not save them. The visible trunk is **0.60 m
square**: half-width **0.30 m** at z 0.4–0.9 on all four meshes. Below z ≈ 0.4 the root flare
widens to ±0.5–0.7. The lowest branches are at **z 1.0** (Tree01), **1.4** (Tree02), **2.8**
(Tree03) and **2.6** (Tree04R). The GLBs are in metres: the client scales trees and dwarf by
the same 0.625.

So from the next cell's centre, **the bark at waist-to-chest height is 1.30 m in front of
him**, not 0.8 m (0.8 is the trunk cell's face; the mesh is narrower than its cell).

**Wolf's call: bite at 0.95 m plus a client nudge.** I put three options to him:

- 1.30 m on the real bark;
- 0.95 m plus a client nudge;
- a low chop at the root flare.

At 1.30 m, even a 1.05 m axe with a 20° lean locks the arm dead straight, and the chop reads
as a spear thrust (`stages/cut-03`). At 0.95 m the arm can bend (`cut-04`). The flare is 1.10 m
on two variants and 0.90 m on the other two. He picked **0.95 m + `CUT_OFFSET`**.

**Why the side face.** An axe's edge runs **parallel to its haft**. His fist reaches only
y ≈ 0.45, so a haft long enough to touch the front face points straight at it, and the edge
goes in end-first. I measured exactly that on the first build: one end 118 mm deep, the other
92 mm short. So the blow comes over from his right and lands on the **trunk's right side
face**, haft running forward alongside the trunk, edge flat on the bark and leading down.
That is how a right-hander chops a trunk standing in front of him.

**Bite, frame 18 (phase 0.750), rig metres, off the evaluated mesh** (`bite_report()`):

| | x | y | z |
|---|---|---|---|
| edge, low end | 0.285 | 0.959 | 0.587 |
| **edge, middle** | **0.287** | **1.065** | **0.619** |
| edge, high end | 0.288 | 1.171 | 0.650 |
| **bark it meets**: the trunk's right side face | **x = 0.300** | y 0.95 … 1.55 | z 0.4 … 0.9, measured ±0.30 |

The edge sits **12.0 to 14.9 mm** into the bark along its whole length, parallel to the face.
It starts at the trunk's front corner and leads left and down. The haft runs forward along the
trunk and slightly up (direction (0.08, 0.97, 0.22)). Through the drive (frames 18–21) the
blade sinks a further 15 mm.
In the client's world, after the 0.35 m nudge, the same face is the real trunk's right face.

## 3. How `Cut` is built: one chop in a second

```
   0  recover   axe wrenched out of the bark, edge still toward the trunk, rising
   8  cocked    fist up in front of the right shoulder, haft standing, EDGE FORWARD;
                chest turned 22 deg right
  11  top       three frames' settle: the fuller wind-up an axe's weight earns
  15-17         the edge rolls LEFT on the way down
  18  BITE      edge in the bark, a diagonal blow from upper right          (ease in, t^2)
  21  drive     body commits through it, the blade sinks 15 mm deeper
  24  = 0
```

**The keys fix the axe, and the arm is solved.** Each key says where the axe is (grip, haft
direction, edge direction). The bite and drive keys are placed by the edge point itself.
Between keys, the axe's grip is eased along a line and its orientation is slerped. The right
arm (shoulder X/Y/Z, elbow, wrist X/Y/Z, and the haft's angle in the fist) is then solved
**on every frame**, warm-started from the previous one. Torso, head and left arm are
interpolated channels, as in `Dig`. Legs are planted by `work_r19.plant_legs`.

I got there through three failures, all of which are visible in `stages/`:

1. **Interpolating solved joint angles between keys.** The axe flailed through the floor
   between keys (`cut-16`), because neighbouring solutions were unrelated.
2. **Holding the haft the way the pickaxe is held**, along the fist's Z axis, which is nearly
   the forearm's own axis. The haft ran up inside the sleeve, with 16–29 intersecting faces
   on every frame. It also made arm and haft read as one straight spear. Now the haft
   **crosses the fist**: square to the forearm, angled up to 80° toward the fingers when the
   arm reaches (the solve picks the angle per frame).
3. **A 75° wrist fold.** It drove sleeve box 3, the cuff band (which ends 34 mm above the
   fist's centre), into the grip. The solve now pays heavily past a 20° wrist bend, and the
   haft sits 17 mm below the fist's centre. Shoulder roll and twist cost 6× round 19's price,
   and wrist roll and twist 3×. At lower prices the downswing and the rising recover opened
   the shoulder seams 12–16 mm.

**Two fixes after Wolf watched it:**

- **"axe head is rotated to wrong way in the start (backwards) and in the end".** The
  `recover` key gave the edge a backward direction (forward component −0.66 at frame 0), and
  frames 22 → 4 slerp through it. `recover` now keeps the edge toward the trunk, fresh out of
  the bark (`stages/cut-22`, `cut-23`).
- **"when axe is high the head edge should point either straight to front or a little bit
  down not to left.. so it should rotate to left during the swing".** The root cause was a
  **quarter turn in the fist**: the edge faced the palm side (his midline), so every raised
  pose showed it to his left. It now lies in the plane of haft and forearm, pointing past the
  fingers, which is how an axe is gripped. The wind-up now raises the fist in front of his
  right shoulder with the haft standing. Measured edge direction on the evaluated rig:

  | frames | edge (x, y, z) | reads as |
  |---|---|---|
  | 0–2 | (−0.60…−0.17, +0.74…+0.97, −0.29…−0.16) | forward, a little down and left: just out of the bark |
  | 3–14 | (−0.06…+0.28, **+0.93…+1.00**, −0.05…+0.22) | **forward** |
  | 15–17 | (−0.10…−0.51, +0.98…+0.63, −0.15…−0.58) | **rolling left and down** |
  | 18–21 (bite, drive) | (−0.69…−0.75, +0.20, −0.69…−0.63) | into the side face, leading left-down |

  At the top it tilts slightly **up** (z up to +0.22) and a touch right rather than level or
  slightly down. The arm runs out of range there (grip misses 15–30 mm on frames 7–12). I
  told Wolf before he accepted it. The bite pose had to be re-targeted for the corrected grip:
  asking for the haft dead forward left no arm solution (the edge came out facing **right**),
  so the haft now runs forward and slightly up, which the arm can hold. `stages/cut-24`,
  `cut-25` show the result.

The solver does not hit the authored path exactly. The worst grip miss is 31 mm at the top of
the wind-up. It does hit at the bite, where the edge point is priced directly. Frames whose
warm start leaves the axe facing wrong (axis dot < 0.9, or a miss over 30 mm) are re-solved
from nine fresh starts.

**Not `Dig`.** Compare `renders/r20/cut-keys-high.png` with `dig-keys-high.png` (same camera,
40° up, behind his right shoulder):

- `Dig`'s pick stays in the plane x ≈ 0.5 and comes straight down in front of him onto the
  floor.
- `Cut` raises the axe **at his right shoulder, edge facing the trunk** (frames 3–14). It then
  comes over and **across, rolling left, onto a vertical face** at waist height. The chest turns 39° through the swing (−24° → +15°), and the spine adds 22°.

I played them one after the other in Wolf's viewport, MATERIAL shading, from the same high
camera: `Dig` at 48 fps (0.5 s per swing, its game speed), then `Cut` at 24 fps (1.0 s).

## 4. Watched

I played `Cut` in the live viewport at game speed (1.0 s per cycle, frames 0–23 looping), in
MATERIAL shading, with a temporary 0.60 m trunk at the authored position (never saved). Views:
three-quarter, side, front (trunk hidden so it does not cover him) and the high camera.

It reads as a woodsman's chop: a slow wind-up at the right shoulder with the edge looking at
the trunk, a three-frame hang at the top, then the axe comes over fast, rolls left and stops
dead in the side of the trunk. The body drives
through for three frames and the beard swings. Then he wrenches the axe out and rises into
the next chop. Five loops in a row read as steady chopping.

Cycles strips (GPU) in `src-assets/renders/r20/`:

- `cut-keys-side.png`, `cut-keys-threequarter.png`, `cut-keys-high.png`, `cut-keys-front.png`: frames 0, 5, 8, 11, 14, 16, 17, 18, 21, 23
- `cut-cycle-back.png`: every second frame, from behind (the slung pickaxe)
- `dig-keys-high.png`: `Dig` from `cut-keys-high`'s camera, for the comparison
- `walk-axe-stowed.png`: `Walk` from behind-left, with the axe strapped to the pack

Viewport stage shots, in the order taken: `renders/r20/stages/cut-01` … `cut-27`. That
includes the rejected passes (the 1.30 m spear, the front-face poke, the flailing
interpolation, the in-sleeve grip, the backward edge). They are kept as the record of why.
`cut-26` and `cut-27` are the right-shoulder close-ups for the §7 seam.

## 5. Foot slide

`Cut` is planted. Every sole vertex is measured against its own frame-0 world position, on
every frame 0–24:

```
  frame   L drift mm   R drift mm    L min z    R min z
      0       0.0000       0.0000   -0.00000   -0.00000
   1-23  0.0000-0.0001 (both)       all -0.00000 .. 0.00000
     24       0.0000       0.0000   -0.00000   -0.00000
  WORST SLIDE 0.0001 mm
```

No step: the legs are solved to their bind footprints under every root drop (−20 … −50 mm)
and pelvis pitch (−9° … +2°).

## 6. Clearance

Per frame, off the evaluated meshes, for every clip:

- the axe's lowest z;
- surface gaps from the axe to the head assembly (head, hair, beard, moustache), to the
  pickaxe, and to the rest of the body (the part it is held by or strapped to excluded);
- intersecting-face counts;
- the pickaxe's and the lantern's gap to the head.

Full tables are in `renders/r20/checks-r20.txt`.

| clip | axe min z | axe ↔ skull | axe ↔ pickaxe | axe ↔ body (nearest) | pickaxe ↔ skull | lantern ↔ skull | faces intersecting |
|---|---|---|---|---|---|---|---|
| **`Cut`** (axe in hand) | 0.550 | **117.0 mm** | 549.8 mm | 13.2 mm (sleeve.R) | 109.7 mm | 247.5 mm | **0** |
| `Walk` (stowed) | 0.461 | 96.3 | 666.6 | 40.1 (belt) | 256.2 | 310.5 | 0 |
| `Dig` (stowed) | 0.396 | 73.8 | 666.1 | **2.4 (skirt, frame 12)** | 257.3 | 298.2 | 0 |
| `Carry` (stowed) | 0.433 | 110.3 | **133.1** (slung pick) | 32.8 (belt) | 213.1 | 171.1 | 0 |

All four pass. Two numbers I'm naming rather than leaving in the tables:

- **Stowed axe ↔ skirt in `Dig`: 2.4 mm** at the top of the wind-up (frame 12), where
  `Dig`'s lean-back swings the pack's foot toward the skirt. It is clear, but only just.
- **In `Carry`**, the stowed axe and the slung pickaxe share the pack's back, and are
  **133 mm apart** at their nearest.

In `Cut`, the pickaxe (slung) never goes below z 0.213, and the axe never goes below z 0.550.
The cross-body chop needed shoulder roll and forearm twist, the combination round 18 §5
warned about. With the axe raised at his shoulder, the closest it comes to the skull is
117 mm. The axe ↔ pack
gap in the stowed clips is a constant 0.4 mm on every frame: both are rigid to `chest`.

## 7. In place, loop seam, seams

**In place:** `root` world x and y spans are **0.000e+00 m** on every frame 0–24. Vertical
only: −0.050 … −0.020.

**Loop seam:** frame 0 vs frame 24, all **21** joints' pose matrices. Worst element delta is
**0.000e+00**, bit-identical: frame 24 re-uses frame 0's cached arm solution.

**Seams:** `seam_walk()`'s method (`work_r19.seam_clip`) at every real posed frame of `Cut`.
**One of 25 touching pairs is over the 10 mm limit**:

- **sleeve.R/torso 12.3 mm at frame 2: over the limit**
- sleeve.R/strap.R 10.0 mm at frame 1: at the limit
- skirt/torso 8.4 mm (frame 9)
- sleeve.L/torso 6.0 mm
- head/torso 5.4 mm

**Why it stays, said plainly.** Both over-limit frames are the right arm rising out of the
bark (shoulder pitch 41° → 62°, roll 8° → 23°, twist about 20°). There is no solver branch
jump; the arm moves smoothly. The sleeve cap separates from the torso and strap in a band of
shoulder angles that any rise from about 40° to 100° with some roll passes through. I tried
four fixes, and each one only moved the spike:

| change | worst over the rise |
|---|---|
| shoulder roll/twist priced 2× | 12.3 mm |
| shoulder roll/twist priced 4× | 11.2 mm |
| shoulder roll/twist priced 8× | 15.3 mm |
| smooth ease on frames 0–8 | 19.1 mm |
| ease-in on frames 0–8 | 15.4 mm |
| recover grip / chest-twist variants (7 tried) | 9.0–12.2 mm on frames 0–4; the best, 9.7 mm, missed its grip by 13 mm and was not tested past frame 4 |

So it is the **r17 shoulder's construction**, exposed by this clip. Per the brief I am
reporting it, not fixing it, and Wolf plans to rethink the model and rig. **It is not visible.**
Viewport close-ups of the right shoulder at frames 0–4 from front-right and back-right
(`stages/cut-26`, `cut-27`) show the cap covered on every frame. The metric is a surface
distance, not daylight, the distinction round 17's seam work recorded.

Two pairs are excluded by design:

- **axe/pack:** the 25th touching pair at bind. In `Cut` the axe leaves the pack for the fist.
- **glove.R/pickaxe:** the pickaxe is slung on his back, as in `Carry`.

## 8. Hashes, joints, actions

`fcurve_hash()` is round 19's: sha256 over every key of every F-curve, sorted by
`(data_path, index)`. "20-joint curves" means `fcurve_hash(name, work_r19.BONES)`, i.e. every
curve the action had in round 19.

| action | before (HEAD, = round 19 §8) | after: 20-joint curves (saved == cold) | after: all curves, incl. `axe` rest keys (saved == cold) |
|---|---|---|---|
| `Walk` | `b1079d210fb881bdcc87ee2a2d4992b4c3b0a6572707339ff86ac2907950d3ea` | `b1079d21…` **unchanged** | `69a3d226c2a31f3dfc7ed3443b35cd26341293a3c8414f2cd559dc75775fd894` |
| `Dig` | `844687a11997b6826d097eee23bb3332c3838be56e26b5666038da56c472b962` | `844687a1…` **unchanged** | `0da15832629121145613045d8c6dd6fa30ec3c5ac88e995546caf55e3ea964de` |
| `Carry` | `c50f8269451166101c9dc64f18fc93e9a4be8584a3f73d2d6f39b1439a175bde` | `c50f8269…` **unchanged** | `d5637d199ac20895c25dfb1b63bdeebd1b72a494d3f2ce7b49027defd2af1516` |
| **`Cut`** | n/a | n/a | **`1a40193d842fc1068a52380f3605b530ae172360f3999664efc5e191ff5f040e`** |

The old whole-action hashes moved **only** because of the 7 `axe` rest curves each gained,
which is the brief's (B) precedent. Every key they had is byte-identical.

- **Joints: 21** (round 19's 20 + `axe`, child of `chest`).
- **Parts: 22** (+ `r17_axe`), 3456 triangles.
- **Actions in the saved file: exactly `Carry`, `Cut`, `Dig`, `Walk`**, each on a fake user
  with one slot. The armature is saved at bind with `animation_data.action = None`.
- No temporary objects or materials are in the file.

## 9. Cold-run regeneration

Three background runs (`blender -b --factory-startup --python coldrun_r20.py`), none of which
authored anything: one per version Wolf saw (the first build, then each of his two edge fixes).
Log: `renders/r20/coldrun-r20.txt`.

1. HEAD's committed .blend: per-part geometry fingerprints and action hashes.
2. The saved .blend: the same.
3. `read_factory_settings(use_empty=True)`, then `dwarf_r17.build("C")` → `walk_r18.build()` →
   `work_r19.build_dig(); build_carry()` → `cut_r20.build_cut()`, then the same again.

```
HEAD   21 parts, 3360 tris, 20 joints, geometry 66b513ee84730b0c
saved  22 parts, 3456 tris, 21 joints, geometry 058050c8002a9a80
cold   22 parts, 3456 tris, 21 joints, geometry 058050c8002a9a80
old 21 parts: HEAD == saved == cold                        YES
r17_axe: saved == cold                                     YES
joints: HEAD 20, saved 21, cold 21                         YES
actions exactly Carry, Cut, Dig, Walk (saved, cold)        YES
every action: saved == cold                                YES
Walk/Dig/Carry 20-joint curves == round 19 (saved, cold)   YES
Walk/Dig/Carry HEAD hashes == round 19                     YES
COLDRUN PASS
```

**The geometry of every existing part is frozen**: all 21 match HEAD vertex for vertex and
face for face. The only additions are the axe part, its joint, and the rest keys.

## 10. Session

| | |
|---|---|
| model | **`claude-opus-5-5`** (Claude Opus 5.5), Claude Code CLI |
| cost | **not visible to me.** `/cost` in this session has it |
| turns | **not counted reliably by me.** The session transcript has the exact number. Wolf's decisions: option (B) with a proper axe; the bite at 0.95 m with a client nudge, rather than 1.30 m on the real bark or a low chop at the flare; after watching, "axe head is rotated to wrong way in the start (backwards) and in the end" (fixed, §3); "when axe is high the head edge should point either straight to front or a little bit down not to left" (fixed, §3); two hands (not reachable on this rig, §1), then "finish it .. we can use this now.. have to rethink the model and rig anyway at some point" |

## 11. What I did not do, and what I could not verify

- **No gate run.** I did not run `scripts/gate.sh`. This round touched only `src-assets/` (no
  Rust), and nothing in this report is a gate result.
- **No real export, no `check_asset.py`, no game.** I did not run `export_dwarf.py` against
  `OUT_PATH`, and I did not probe a GLB this round. The 21-joint skin, `axe` being sampled at
  rest in the old clips, and `Cut` loop closure in `check_asset.py` are all **unverified in a
  GLB**. They follow round 19's evidence for `pick`, but I did not re-prove them.
- **`CUT_OFFSET` is unverified in game.** The client nudge (0.35 m toward the trunk, along
  its yaw) is the forge side's to add and check. Without it, the bite lands 0.35 m short of
  the bark.
- **Tree yaw.** The client turns trees in quarter turns. The trunk is square, so the side
  face is always at ±0.30 m. I did not check every tree variant at every yaw in the client.
- **The seat camera.** I judged "not `Dig`" from a high three-quarter camera in Blender, not
  through the game's seat camera.
- **Sub-frame behaviour is argued, not measured.** All checks sample integer frames. Between
  frames 17 and 18 the head travels about 0.4 m: a one-frame smear by design, as in `Dig`.
- **Wolf's Blender crashed once mid-session**, during my back-to-back 90–120 s solver calls
  over the MCP. Nothing was lost: the last good save was on disk. After that I kept every live
  call under about 40 s, with solved frames cached between calls. The proofs above all ran on
  the file saved afterwards.
- **The edge tilts slightly up at the top of the wind-up** (§3), not level or down as Wolf
  first asked. He saw it before accepting the build.
- **Not committed.** Nothing is staged or committed; that waits on Wolf.

## 12. Files

| file | state |
|---|---|
| `src-assets/blender/SM_VoxelDwarf_Miner01.blend` | **modified**: `Cut` added; joint `axe` + part `r17_axe` added; `axe` rest keys on `Walk`/`Dig`/`Carry`; old geometry = HEAD; saved at bind, unassigned |
| `src-assets/blender/cut_r20.py` | **new**: `add_axe`, `build_cut`, the axe-frame solver, `checks`, `renders`, `attach`/`detach` |
| `src-assets/blender/coldrun_r20.py` | **new**: the §9 proof |
| `src-assets/blender/work_r19.py` | **modified**: `key_clip(name, pose_at, bones=None, located=("root", PICK))`, with defaults exactly as before |
| `src-assets/blender/export_dwarf.py` | **modified**: `"axe"` in `JOINTS`; docstring 20 → 21 joints |
| `src-assets/renders/r20/*.png`, `stages/*.png` | **new**: 7 Cycles strips, 23 stage shots |
| `src-assets/renders/r20/checks-r20.txt`, `coldrun-r20.txt` | **new**: full proof output |
| `dwarf_r17.py`, `walk_r18.py` | **untouched**. `hex_rgb` still has no `srgb_to_linear` (#94) |

Rebuild in the live session (after `Walk`, `Dig` and `Carry` exist):

```python
exec(open(r"src-assets/blender/cut_r20.py").read())
build_cut(); checks(); detach()
renders()                      # Cycles strips to renders/r20/
```
