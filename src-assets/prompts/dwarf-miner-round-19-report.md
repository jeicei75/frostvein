# Round 19 report — the work clips: `Dig` and `Carry`

**Delivered:** two looping actions on the r17 armature in
`src-assets/blender/SM_VoxelDwarf_Miner01.blend`, regenerable from
`dwarf_r17.py` → `walk_r18.py` → **`work_r19.py`** (proven cold, §9).

| | `Dig` | `Carry` |
|---|---|---|
| frames / fps | 0 … 24 at 24 fps, frame 24 ≡ frame 0 | 0 … 24 at 24 fps, frame 24 ≡ frame 0 |
| game duration | **0.5 s** per cycle (one 5-tick work run) | phase-locked to ground covered, `distance / 0.4926 m` |
| keys | 86 fcurves, 2150 keys, all LINEAR | 86 fcurves, 2150 keys, all LINEAR |
| strike / hold | **strike frame 18** (phase 0.750) | **`CARRY_OFFSET = (0.000, 0.280, 0.705)`**, 0.64 m stone |
| foot slide | 0.0002 mm (planted; float noise) | **0.000 mm**, legs/root/hips **verbatim** from `Walk` |
| in place | root x/y span **0.000e+00** | root x/y span **0.000e+00** |
| loop seam | **bit-identical** (20 joints) | **bit-identical** (20 joints) |
| seams | PASS, worst 9.4 mm | PASS, worst 7.4 mm |

**Departures from the brief, all Wolf's calls during the session (§1):**

- The pickaxe rides **on his back** during `Carry`. That needed a **20th joint, `pick`**, and
  `export_dwarf.JOINTS` now includes it.
- `Walk` gained **rest keys for `pick` only**. Its 79 round-18 curves are byte-identical.
- The stone stays at the brief's **0.64 m**. A 0.25 m stone was built first and rejected:
  pulling the fists in that far wrecked the shoulders and elbows.

---

## 1. What changed from the brief, and why

### The stone: the hands only ever reach its rear edge, and 0.64 m is what reads best

I measured the reach before posing anything. **Palm reach is 0.302 m from the shoulder head**
(upper arm 0.121, forearm 0.190, palm offset inside the glove). The body's front surface sits
at **y 0.20** (belly, belt) to **y 0.26** (beard, z 0.6–1.0). Below z ≈ 0.51 a palm cannot
reach the front at all.

I solved both arms (shoulder X/Y/Z, elbow X, wrist X/Y/Z, coordinate descent over a
hand-written FK) for palms on a cube's side faces at chest height, with the cube's near face
just off the beard:

| cube side | palm at the rear edge | palm 25 % deep | palm at mid-depth |
|---|---|---|---|
| 0.20 m | reaches | 8 mm short | 46 mm short |
| 0.30 m | reaches | 11 mm short | 74 mm short |
| 0.40 m | reaches | 19 mm short | 110 mm short |
| **0.64 m** | reaches | **68 mm short** | **223 mm short** |

At **any** size the hands reach only the cube's **rear edge**
(`renders/r19/stages/carry-01-cube-0.64-unreachable.png`). This is the arm length, not the
pose. It is a geometry fact I did not change.

**How the size was settled, in two passes:**

1. **0.25 m first.** I offered it as the smallest stone that reads in front of the chest
   (`stages/carry-02` … `carry-07`, all 0.25 m), and that was the first call. Watching it, Wolf
   found the arms wrecked. To bring the fists in to x = ±0.125 (the shoulders are at ±0.277),
   the solve rolled each shoulder **52–56°** and twisted it **36–38°**, and the wrists twisted
   **62–76°**. On this rig that twists the sleeves and elbows into knots.
2. **0.64 m, Wolf's call after comparing it with 0.45 m live.** A bigger box puts its side
   faces near shoulder width, so the arms can reach **almost straight forward**: shoulder pitch
   ~73°, roll **±21–25°**, elbows 0–3°. The palms sit on its side faces 20 mm in from the near
   face, which is the deepest this reach allows.

The cost, said plainly: from the front, a 0.64 m box hides him except the hair and boots. From
the three-quarter view his eyes just clear its top. The client constant stays at **0.4 cells**.

Two solver fixes came with the rebuild, both visible in the angles:

- **Posture cost.** Shoulder and wrist roll/twist (Y, Z) pay 0.05 per rad² (`POSTURE`), so
  the solve reaches with shoulder pitch first.
- **Symmetry.** The left arm is seeded from the mirrored right solution, and **both** fists pay
  the lantern-hang term. With it on the left only, the right elbow bent 38° while the left
  stayed straight, and the hold read lopsided.

### The pickaxe goes on his back during `Carry`: a 20th joint

Also Wolf's call: both hands hold the stone, and the pickaxe is slung on the pack. A rigid
one-joint weight cannot do this. `Dig` needs the pickaxe on `hand.R` and `Carry` needs it on
the chest, and both are the same vertices. So:

- **`work_r19.add_pick_bone()`** adds joint **`pick`**, child of `hand.R`, head at the grip
  (0.506, 0, 0.547), tail up the shaft. It renames `r17_pickaxe`'s only vertex group from
  `hand.R` to `pick`. It is idempotent, and it is part of the regeneration chain.
- At rest `pick` moves nothing: the pickaxe at bind matches the 19-joint figure to
  **1.2e-7 m** (float32), and the `Dig` strike tip came out identical to the millimetre before
  and after.
- `Dig` keys `pick` at rest on every frame, so a `Dig` that follows a `Carry` can never inherit
  a pickaxe on the back. `Carry` solves `pick` every frame so the pickaxe is **rigid to the
  chest**: shaft 35° off vertical, head up behind the right shoulder, flat on the pack's back
  face (`stages/carry-04-back-pick-slung.png`, `carry-keys-back.png`).
- **`Walk` gets `pick` rest keys, and nothing else changes.** The first build left `Walk`
  without a `pick` curve. In Blender an un-keyed bone keeps its last pose, so playing `Carry`
  and then switching to `Walk` left the pickaxe slung on his back while his hand walked empty.
  Wolf caught it in the viewport. `work_r19.key_walk_pick()` (called from `add_pick_bone()`)
  now keys `pick` at rest on frames 0–24, LINEAR. **`Walk`'s 79 round-18 curves are
  byte-identical**, and `fcurve_hash("Walk", W.BONES)` is still `628961a8…` (§8). I re-ran
  the case Wolf hit: `Carry` at frame 5, then `Walk` at frame 7, and `pick` reads exactly rest.
- The GLB was never affected. I exported a probe GLB (scratch path, not the real export) and
  read it back: **20 joints in the skin, 60 channels in every clip**. That probe was taken
  before the `Walk` rest keys existed, and even then the exporter sampled `pick` at exactly
  its rest transform in `Walk`, value-identical to `Dig`'s keyed rest.
- **`export_dwarf.py`**: `"pick"` added to `JOINTS` (Wolf approved this), with the docstring
  updated from "19 joint names" to 20. Without it the exporter's rig gate raises on an
  "unexpected joint".

**For the forge side:** the GLB now has **20 joints**. Nothing under `crates/` asserts a joint
count. `check_asset.py` only prints `joints=`, and the client binds clips by name. But anything
that hard-codes 19 will need the bump. I did not run `export_dwarf.py` against the real
output path; that is the forge side's job.

## 2. `Dig` — one pick swing in half a second

```
   0  lift      pick rising out of the last strike, weight coming up
  10  cocked    pick up and back over the right shoulder, body leaning back   (ease out)
  12  top       two-frame settle: the anticipation
  18  STRIKE    blade tip on the floor ahead, accelerating in                 (ease in, t^2)
  20  bite      body follows through; root, pelvis, spine and right arm hold still
  24  = 0
```

**Strike, frame 18 (phase 0.750), rig metres:**

| | x | y | z |
|---|---|---|---|
| blade tip (the +y blade end, `r17_pickaxe` box 9, lowest vertex) | 0.434 | **0.692** | **0.007** |
| pick head eye (box 2 centre) | 0.417 | 0.808 | 0.205 |

So the blade lands **0.69 m ahead, 7 mm above the floor**. That is inside the brief's
0.5–0.8 m floor-to-knee band. It is short of the 0.8 m rock face of a Dig cell and well
ahead of his feet for a Channel. The strike is 0.43–0.52 m to his right because the pick swings
on his right side. The client's yaw to face the target puts the target's near face square in
front of him, so the visual hit is right-of-centre on that face.

**How it is built.** Five keyed poses of channels (degrees about armature axes), eased per
segment, sampled at every integer frame and keyed LINEAR. **The right arm moves about world X
only.** That keeps the whole pickaxe in the sagittal plane x ≈ 0.42–0.53, while the head
assembly reaches only |x| ≤ 0.23, so the 1.09 m shaft physically cannot sweep into the skull.
The brief warned that shoulder roll or a wrist twist would do that. This swing uses
**neither**. The **wrist** does the turning: about −120° to −170° about X, a cocked wrist
rather than a rolled shoulder. A chest twist of −9° … +6° about Z adds body; the clearance
table carries it (minimum 257 mm).

**Legs are planted and solved, not keyed:** `walk_r18.leg_angles` with each ankle held at its
bind position and zero foot pitch, whatever root (−20 … −40 mm crouch) and pelvis (−6° … +2.5°)
do. A foot that is never asked to move cannot slide.

**One-handed.** The lantern hand braces forward on the strike, but it never takes the shaft.
The shaft is 0.5 m to his right, and the left hand would have to cross his body to reach it.

Two things I changed after measuring. The first build carried the chest twist and a
root/pelvis follow-through into the bite, which drove the blade **34 mm under the floor**. Now
everything that carries the pick holds still through 18–20 (it is in the rock), and the strike
crouch was trimmed from 50 to 40 mm, which put the tip at +7 mm.

## 3. `Carry` — walking with the stone

**Legs, root and pelvis are `Walk`'s**, produced by the same `walk_r18.pose_frame` that built
`Walk`. They are proven **verbatim** key for key: **35 curves, 875 keys compared, 0 differ**.
Same stride, same phase; foot slide reads **0.000 mm** exactly as `Walk` does (table in §5).

**Upper body:** spine leans back 1° and chest 1.5° against the weight. Chest twist is cut from
`Walk`'s 7° to 2°, the neck levels the head, the head counter-twists, and the beard is held
still. **Arms are solved every frame**, warm-started from the previous frame. On frame 0 the
left starts from the mirrored right, and frame 24 reuses frame 0's solution, so the seam is
exact. The palms sit on the 0.64 m stone's side faces, facing inward, riding the walk's ±5 mm
bob. The arms reach almost straight forward: shoulder pitch 71–74°, roll ±21–25°, elbows
0–3°, and the angles mirror to within a few degrees across the cycle. A cost keeps each fist's
"down" pointing down, which keeps the lantern hanging instead of swinging up into the stone.
Worst palm miss is **9.4 mm** off its target point on the face, and the worst palm facing is a
0.970 dot (≈14°).

### `CARRY_OFFSET`

| | x | y | z |
|---|---|---|---|
| **`CARRY_OFFSET`**, the palm midpoint, mean of 24 frames | **0.000** | **0.280** | **0.705** |
| wander through the cycle (span) | 1.5 mm | 8.8 mm | 8.1 mm |
| farthest from the mean | | | **6.0 mm** |

**Read this before parenting the stone at `CARRY_OFFSET`.** The brief defines the offset as
the palm midpoint, and that is the number above. But the palms hold the stone's **rear
edge**, so a 0.64 m stone **centred** on the palm midpoint would sit 300 mm back, through his
chest and beard. The stone **centre** that puts the palms on its side faces is:

> **stone centre = (0.000, 0.580, 0.705)**, the palm midpoint + 0.300 m in y

That is the point every clearance number in §6 is measured against: a fixed 0.64 m cube there,
not bobbing, exactly as the client would draw it.

## 4. Watched

Played both clips in the live viewport in MATERIAL shading from the side, front and
three-quarter. **`Dig` at game speed, 0.5 s per cycle**: scene at 48 fps for playback, 24
restored afterwards. `Carry` at 24 fps, with a stand-in stone at the fixed centre above. I
watched it first at 0.25 m. Wolf watched that and rejected it, then compared 0.45 m and 0.64 m
live (§1).

`Dig` reads as a short, chopping pick swing. The pick rises in front, rolls back over the
right shoulder, settles for two frames, then drops hard. Frame 17 has the pick horizontal at
head height and frame 18 has it in the floor, a one-frame smear at 21 ms of game time. The
bite holds and the beard swings through it. `Carry` reads as a dwarf trudging behind a big
block he holds out in front of him by its back edges, arms straight and level. The legs and
bob are `Walk`'s, and the pick stays flat on the pack. From the front, the block hides him
except the hair and boots.

Cycles strips, GPU, in `src-assets/renders/r19/`:

- `dig-keys-side.png`, `dig-keys-front.png`, `dig-keys-threequarter.png`: frames 0, 10, 12, 16, 17, 18, 20, 22
- `dig-cycle-side.png`: every second frame of the cycle
- `carry-keys-side.png`, `carry-keys-front.png`, `carry-keys-threequarter.png`: frames 0, 3, … 21, with the 0.64 m stone
- `carry-keys-back.png`: the slung pickaxe

Per-stage viewport screenshots, in the order they were taken, are in `renders/r19/stages/`:
strike, wind-up, the first `Dig` build, the 0.64 m cube against the reach (`carry-01`), and the
**rejected 0.25 m** pass (`carry-02` … `carry-07`). They are kept as the record of why it was
rejected. The shipped 0.64 m hold is in the Cycles strips above.

## 5. Foot slide

**`Carry`**, using `Walk`'s own test (sole `y* = world y + stride × k / 24`, walked in each
leg's own phase order). Left boot shown; the right is the same table shifted 12 frames, and
both were measured:

```
  frame  pivot       p    heel y*    heel z     toe y*     toe z  slide mm
      0   heel   0.000    0.00880  -0.00000    0.25194   0.05168     0.000
      1   heel   0.077    0.00880   0.00000    0.25639   0.02215     0.000
      2   heel   0.154    0.00880   0.00000    0.25737  -0.00000     0.000
      3   heel   0.231    0.00880  -0.00000    0.25737   0.00000     0.000
      4   heel   0.308    0.00880  -0.00000    0.25737  -0.00000     0.000
      5   heel   0.385    0.00880   0.00000    0.25737  -0.00000     0.000
      6    toe   0.462    0.00880   0.00010    0.25737  -0.00000     0.000
      7    toe   0.538    0.00886   0.00541    0.25737  -0.00000     0.000
      8    toe   0.615    0.00938   0.01692    0.25737   0.00000     0.000
      9    toe   0.692    0.01087   0.03203    0.25737  -0.00000     0.000
     10    toe   0.769    0.01350   0.04808    0.25737   0.00000     0.000
     11    toe   0.846    0.01679   0.06250    0.25737  -0.00000     0.000
     12    toe   0.923    0.01972   0.07285    0.25737  -0.00000     0.000
     13    toe   1.000    0.02097   0.07681    0.25737  -0.00000     0.000
     14-23  swing (identical to Walk's table, round 18 §4)
  WORST SLIDE 0.000 mm
```

This is `Walk`'s table from round 18 §4 digit for digit, as it has to be with verbatim keys.

**`Dig`**: planted, so every sole vertex against its own frame-0 world position, every frame
0–24:

```
  frame   L drift mm   R drift mm    L min z    R min z
      0       0.0000       0.0000   -0.00000   -0.00000
    1-23  0.0000-0.0002 (both)       all -0.00000 .. 0.00000
     24       0.0000       0.0000   -0.00000   -0.00000
  WORST SLIDE 0.0002 mm
```

0.2 µm is float32 noise through a different root/pelvis per frame; the feet do not move.

## 6. Prop clearance

Lowest world z of each prop, and the nearest surface-to-surface approach to the head assembly
(`r17_head`, `r17_hair`, `r17_beard`, `r17_moustache`), every frame, off the evaluated meshes.

**`Dig`:**

```
  frame  axe min z axe-head mm     lant z lant-head mm  boot.L z  boot.R z
      0    0.51283      281.32    0.22275       306.81  -0.00000  -0.00000
      1    0.54532      279.65    0.22335       308.65  -0.00000  -0.00000
      2    0.58097      271.66    0.22457       310.21   0.00000   0.00000
      3    0.61477      266.22    0.22588       311.52  -0.00000  -0.00000
      4    0.63860      260.43    0.22734       312.59  -0.00000  -0.00000
      5    0.65749      257.57    0.22881       313.45  -0.00000  -0.00000
      6    0.67194      257.32    0.23019       314.12   0.00000   0.00000
      7    0.68235      257.32    0.23136       314.62  -0.00000  -0.00000
      8    0.68929      257.32    0.23225       314.97  -0.00000  -0.00000
      9    0.69323      257.48    0.23281       315.17   0.00000   0.00000
     10    0.69451      257.57    0.23300       315.24  -0.00000  -0.00000
     11    0.70240      257.32    0.23596       314.88  -0.00000   0.00000
     12    0.71098      257.32    0.23911       314.51  -0.00000  -0.00000
     13    0.70349      257.34    0.23762       314.20   0.00000   0.00000
     14    0.67797      257.32    0.23357       313.23   0.00000   0.00000
     15    0.62855      265.04    0.22819       311.44  -0.00000  -0.00000
     16    0.54560      281.32    0.22348       308.65  -0.00000  -0.00000
     17    0.49904      281.74    0.22272       304.65  -0.00000  -0.00000
     18    0.00671      321.15    0.23012       299.43  -0.00000  -0.00000
     19    0.00671      323.35    0.24935       298.56  -0.00000  -0.00000
     20    0.00671      324.31    0.25639       298.17  -0.00000  -0.00000
     21    0.15070      315.84    0.24905       299.58  -0.00000  -0.00000
     22    0.50417      287.07    0.23575       302.65  -0.00000  -0.00000
     23    0.49882      281.32    0.22605       305.56  -0.00000  -0.00000
  worst: axe z 0.0067, axe-head 257.3 mm, lantern z 0.2227, lantern-head 298.2 mm
  PASS
```

The pickaxe is never below the floor. Its lowest point is **6.7 mm** at the strike, held
through the bite. It never comes closer than **257 mm** to the skull, even passing directly
overhead at frames 4–14, because it never leaves the plane x ≈ 0.5 (§2).

**`Carry`**: also the signed distance to the held stone's **volume** (a fixed 0.64 m cube at
(0, 0.580, 0.705); negative = inside). Columns: pickaxe, lantern, beard, and the nearer fist:

```
  frame  axe min z axe-head mm     lant z lant-head mm  boot.L z  boot.R z  axe-stone  lant-stone  beard-stn  fists-stn
      0    0.22958      214.56    0.36866       173.06  -0.00000  -0.00000      671.8        31.2       11.0      -14.0
      1    0.22224      214.07    0.36727       174.41   0.00000  -0.00000      671.0        31.4       10.2      -11.6
      2    0.21963      214.41    0.36669       175.22  -0.00000   0.00488      669.7        31.5       10.0      -10.7
      3    0.22242      215.63    0.36702       175.20  -0.00000   0.01098      668.2        31.2       10.2      -10.6
      4    0.22988      217.60    0.36836       174.54  -0.00000   0.01820      666.5        31.6       11.0      -11.9
      5    0.24000      217.84    0.37002       173.93  -0.00000   0.02620      664.6        27.5       12.4      -13.9
      6    0.25010      217.93    0.37134       173.89  -0.00000   0.03446      662.4        18.6        6.8      -15.8
      7    0.25748      218.40    0.36983       174.34  -0.00000   0.02500      660.1        12.0        2.8      -20.2
      8    0.26018      219.40    0.36843       174.96   0.00000   0.01637      657.6         8.9        1.4      -22.5
      9    0.25754      220.93    0.36764       175.64  -0.00000   0.00930      655.1         8.8        2.8      -22.6
     10    0.25034      222.81    0.36757       176.52   0.00000   0.00402      652.8        11.9        6.8      -20.7
     11    0.24055      224.20    0.36810       177.67  -0.00000   0.00080      651.0        17.0       12.4      -17.2
     12    0.23081      225.49    0.36696       178.95  -0.00000  -0.00000      649.9        22.9       11.0      -14.1
     13    0.22368      226.19    0.36580       179.98  -0.00000   0.00000      649.7        27.8       10.2      -11.7
     14    0.22100      225.31    0.36549       180.17   0.00488  -0.00000      650.7        30.4       10.0      -10.7
     15    0.22348      224.57    0.36615       179.25   0.01098  -0.00000      652.6        30.2       10.2      -10.6
     16    0.23049      223.46    0.36758       177.54   0.01820  -0.00000      655.5        27.4       11.0      -11.9
     17    0.24023      220.71    0.36943       175.55   0.02620  -0.00000      658.9        22.8       12.4      -13.9
     18    0.25010      217.93    0.37132       173.86   0.03446  -0.00000      662.4        18.6        6.8      -15.8
     19    0.25741      215.54    0.37237       172.70   0.02500  -0.00000      657.6        16.1        2.8      -20.1
     20    0.26010      213.89    0.37328       171.85   0.01637   0.00000      652.0        16.6        1.4      -22.6
     21    0.25735      213.13    0.37320       171.23   0.00930  -0.00000      652.5        19.8        2.8      -22.6
     22    0.24988      213.23    0.37228       171.13   0.00402   0.00000      658.7        26.2        6.8      -20.6
     23    0.23970      214.03    0.37066       171.78   0.00080  -0.00000      668.4        31.3       12.4      -17.2
  worst: axe z 0.2196, axe-head 213.1 mm, lantern z 0.3655, lantern-head 171.1 mm, axe-stone 649.7, lantern-stone 8.8, beard-stone 1.4 mm, fists -22.6..-10.6 mm
  PASS
```

Worst case: pickaxe 220 mm above the floor and 213 mm from the skull; lantern 366 mm up and
171 mm from the head. Two numbers are **tight**, and I am naming them rather than hiding them
in the table:

- **Beard to stone: 1.4 mm** at frames 8 and 20, where the walk's bob lifts the beard toward
  the fixed stone. It's clear, but only just. Pushing the stone forward to buy margin would
  cost grip depth the arms don't have.
- **Lantern to stone: 8.8 mm** at frame 9. The lantern hangs from the left fist just under
  the stone's near edge.

**The fists are inside the stone by 10.6–22.6 mm.** That is the grip, and it is intended:
the fist boxes sink into the stone's sides. It varies with the bob, because the client's stone
is fixed and the hands are not.

## 7. In place, loop seam, seams

**In place:** `root` world x and y spans are **0.000e+00 m** on both clips, every frame
0–24. Vertical only: `Dig` −0.040 … −0.020, `Carry` −0.020 … −0.010 (`Walk`'s).

**Loop seam:** frame 0 vs frame 24, all **20** joints' pose matrices: worst element delta
**0.000e+00** on both. Bit-identical, not "small". `Carry` gets this because frame 24 re-uses
frame 0's arm solution rather than re-solving it.

**Seams:** `seam_walk()`'s method (the same 24 pairs that touch at bind, the same
surface-to-surface BVH metric) at every real posed frame:

| clip | result | worst pairs |
|---|---|---|
| `Dig` | **PASS**, 0 of 24 over 10 mm | sleeve.R/strap.R **9.4 mm** f1, sleeve.R/torso 6.0 f2, head/torso 5.8 f22, hair/torso 4.9, belt/torso 4.7, boots/legs 3.3 |
| `Carry` | **PASS**, 0 of 24 over 10 mm | sleeve.R/torso **7.4 mm** f21, sleeve.L/torso 7.3 f9, hair/torso 4.7, boots/legs 3.2, sleeve/strap 2.5 |

`Dig`'s 9.4 mm is the right shoulder at full lift (frame 1). It passes, but it is the closest
any seam came to the limit. If round 17's shoulder ever opens up, look there first.

## 8. Hashes

`fcurve_hash()`: sha256 over every key of every F-curve, sorted by `(data_path, index)`.
`fcurve_hash("Walk", W.BONES)` restricts it to the 19 round-18 bones.

| action | hash |
|---|---|
| `Walk` **before** this round (79 curves) | `628961a89cccd44cfcad1fc2eff84907f0124c275db083453d28e0a6727af3c7` |
| `Walk` **after**, round-18 curves only (saved == cold) | `628961a89cccd44cfcad1fc2eff84907f0124c275db083453d28e0a6727af3c7`, **unchanged** |
| `Walk` **after**, all curves incl. the 7 `pick` rest curves (saved == cold) | `b1079d210fb881bdcc87ee2a2d4992b4c3b0a6572707339ff86ac2907950d3ea` |
| `Dig` (saved == cold) | `844687a11997b6826d097eee23bb3332c3838be56e26b5666038da56c472b962` |
| `Carry` (saved == cold) | `c50f8269451166101c9dc64f18fc93e9a4be8584a3f73d2d6f39b1439a175bde` |

So the brief's "`Walk` untouched" holds for every curve `Walk` had. The whole-action hash
moved only because Wolf asked for the `pick` fix (§1).

**Actions in the saved file: exactly `Carry`, `Dig`, `Walk`**, each on a fake user with one
slot. The armature is saved at bind with `animation_data.action = None`.

## 9. Cold-run regeneration

A background run (`blender -b --factory-startup`). There were two this round: the first
proved the 0.25 m build, and this one, after Wolf's 0.64 m and `Walk`-fix changes, proves
what ships. Neither authored anything:

1. opened **HEAD's** committed .blend and fingerprinted every part's vertices and faces;
2. opened the **saved** .blend and fingerprinted it, plus all three action hashes;
3. `read_factory_settings(use_empty=True)`, then `dwarf_r17.build("C")` → `walk_r18.build()` →
   `work_r19.build_dig(); build_carry()`, then fingerprinted and hashed again.

```
HEAD geometry    e38dd0e1…fe82a6, 3360 tris     HEAD joints   19
saved geometry   e38dd0e1…fe82a6, 3360 tris     saved joints  20
cold geometry    e38dd0e1…fe82a6, 3360 tris     cold joints   20
geometry frozen (HEAD == saved == cold)      YES
Walk  saved == cold                          YES
Walk round-18 curves == 628961a8 (saved, cold) YES
Dig   saved == cold                          YES
Carry saved == cold                          YES
actions exactly Carry, Dig, Walk             YES
COLDRUN PASS
```

**Geometry is frozen:** no vertex and no face changed from HEAD, and 21 parts / 3360 triangles
are as before. What changed is the **rig** (19 → 20 joints) and **one vertex group's name** on
`r17_pickaxe` (`hand.R` → `pick`). Both are covered in §1.

## 10. Session

| | |
|---|---|
| model | **`claude-opus-5-5`** (Claude Opus 5.5), Claude Code CLI |
| cost | **not visible to me.** `/cost` in this session has it |
| turns | **not counted reliably by me.** The session transcript has the exact number. Wolf's decisions: 0.25 m stone + pickaxe on the back; the 20th joint; after watching, a bigger box (0.64 m) and the `Walk` pickaxe fix |

## 11. What I did not do, and what I could not verify

- **No gate run.** I did not run `scripts/gate.sh`. This round touched only `src-assets/`
  (no Rust), and nothing in this report is a gate result.
- **No real export, no `check_asset.py`, no game.** I did not run `export_dwarf.py` against
  `OUT_PATH`, and I did not run `check_asset.py` or the client. The 20-joint / 60-channel /
  `Walk`-holds-`pick`-at-rest facts come from a **probe GLB** written to a scratch path with
  `export_animations=True` and read back. That is the same evidence shape round 18 used, not
  the shipping exporter.
- **The client's stone offset is unverified in game.** `CARRY_OFFSET` and the stone centre are
  measured in rig space. Whether the client's parenting matches (yaw, the 0.4-cell constant,
  stone centre vs palm midpoint, §3) is the forge side's to check.
- **The GLB probe predates the `Walk` rest keys and the 0.64 m `Carry`.** It proved the
  20-joint skin and `pick` sampled at rest. I did not re-export after the final build.
- **Sub-frame behaviour is argued, not measured.** All checks sample integer frames, and LINEAR
  keys at one-frame spacing make the in-between exact by construction. The `Dig` strike moves
  ~0.7 m between frames 17 and 18 by design; I did not sample half-frames.
- **The `pick` quaternion sign** is kept continuous frame to frame by the keyer (q and −q
  are one rotation). With 0.000 loop-seam delta and no visible spin in the strips, it held.
  I did not add a separate check for it.
- **Not committed.** Nothing is staged or committed; that waits on Wolf.

## 12. Files

| file | state |
|---|---|
| `src-assets/blender/SM_VoxelDwarf_Miner01.blend` | **modified**: `Dig`, `Carry` added; `pick` rest keys on `Walk`; joint `pick` added; pickaxe group renamed; geometry hash = HEAD; saved at bind |
| `src-assets/blender/work_r19.py` | **new**: `add_pick_bone`, `build_dig`, `build_carry`, the hold solver, `checks`, `renders`, `attach(name)`/`detach()` |
| `src-assets/blender/export_dwarf.py` | **modified**: `"pick"` in `JOINTS`; docstring 19 → 20 joints |
| `src-assets/renders/r19/*.png` | **new**: eight Cycles strips |
| `src-assets/renders/r19/stages/*.png` | **new**: ten viewport stage shots |
| `src-assets/blender/walk_r18.py` | **untouched**. `Walk`'s 79 round-18 curves hash unchanged; `work_r19` adds only the `pick` rest keys |
| `src-assets/blender/dwarf_r17.py` | **untouched**: `hex_rgb` still has no `srgb_to_linear` (#94) |

Rebuild in the live session (after `Walk` exists):

```python
exec(open(r"src-assets/blender/work_r19.py").read())
build_dig(); build_carry(); checks(); detach()
```
