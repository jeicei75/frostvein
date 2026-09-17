# Round 18 report — the walk cycle

**Delivered:** one looping `Walk` action on the r17 armature in
`src-assets/blender/SM_VoxelDwarf_Miner01.blend`, regenerable from
`src-assets/blender/walk_r18.py`.

| | |
|---|---|
| **Stride** | **0.493 m per cycle** (0.246 m per step) |
| frames | 0 … 24, with frame 24 repeating frame 0 |
| fps | 24 — one cycle is **1.000 s** exactly |
| keys | 1975, every bone on every frame, **LINEAR** |
| stance / swing | 13 / 11 frames per foot (54.2% / 45.8%), 4 frames of double support |
| worst foot slide | **0.000 mm** |
| in place | root world x and y span **0.000e+00 m** |
| loop seam | frame 0 vs frame 24 — **bit-identical** pose matrices |
| seams | worst 3.2 mm over the whole cycle (limit 10 mm) — PASS |
| geometry | **unchanged** — 21 parts, 3360 tris, 19 joints; hash matches HEAD |

The playback rate the client wants is `speed / 0.493`.

---

## 1. Stride: 0.493 m, and why it is not in the brief's band

The brief put the plausible band at 0.8–1.0 m and asked for what I measured, so: **0.493 m**,
about half of it. That is not a tuning choice, it is the leg.

The band assumes human-ish proportions. This dwarf does not have them. His hip pivot is at
**z 0.330** on a **1.200 m** figure — legs are **27.5% of height**, where a human is near 48%.
Hip to ankle is **0.200 m** total. Scaled off leg length rather than height, 0.493 m is a
normal walking stride; it is only short against his height, which is the whole point of a
dwarf.

How the number is derived (`walk_r18.stride()`): through stance, whichever point of the sole
is on the ground is momentarily stationary in the world, so in an in-place clip it travels
backward through body space at exactly the ground speed. Over the 13 frames of stance the
contact point moves back **D = 0.2669 m**. Stance is 13/24 of the cycle, so the cycle is
worth `D × 24 / 13 = 0.493 m`. It is measured off the planted foot, as asked, not off the hip.

**0.8 m is not reachable on this rig at all.** I measured the ceiling rather than estimating
it, by pushing the forward and backward ankle reach until the solve could no longer put the
sole on the floor. The binding constraint is the 0.200 m of leg; the only way to buy more is
to crouch deeper, which trades stride against how the figure stands:

| root drop below bind | longest stride the leg can reach | reach used |
|---|---|---|
| 10 mm | 0.552 m | 98.9% |
| **15 mm (shipped)** | **0.566 m** | 98.3% |
| 20 mm | 0.589 m | 98.8% |
| 30 mm | 0.618 m | 98.5% |
| 50 mm | 0.670 m | 98.4% |

Even at a 50 mm crouch — which on a 1.200 m figure reads as a deliberate sneak, not a walk —
the cycle tops out at **0.67 m**. The brief's band is off the end of the table.

Those numbers are also all at 98%+ of reach, meaning a leg dead straight at both ends of
stance: an IK singularity and a stiff-legged read. The shipped 0.493 m sits at **94.7%**,
which is the margin that keeps the knee doing something at contact and toe-off.

One consequence worth knowing on the code side: **the boot's ground face is 0.2486 m long and
the step is 0.2463 m**, so the feet never fully separate — at contact the rear boot's toe and
the front boot's heel are about 2 mm apart in y. That is a fact about the model's
boot size, not a defect in the cycle, and it is why the walk reads as a heavy trudge rather
than a stride.

## 2. How the legs are made, and the one thing that would have broken

The legs are **solved, not keyed**. The input is the contact point — heel, then whole sole,
then toe — each pinned to the ground and gliding backward at the ground speed; hip, knee and
foot come out of a closed-form two-link solve against the real hip position for that frame.
Foot slide is therefore zero *by construction*, and §4 is a measurement rather than a hope.
Hand-keying this rig would not have survived: with a 0.2486 m boot on a 0.200 m leg, two
degrees at the hip is centimetres at the sole.

**`hips` rotates about X only — no pelvic twist, no pelvic roll.** Both would swing the leg
chain out of the sagittal plane, and since hip/knee/foot are single-axis X hinges the solve
then cannot put the sole back on its line: a 5° pelvis twist is roughly **16 mm of lateral
foot slide**. The counter-rotation read lives on `spine`/`chest`, which have no leg under
them, and it reads fine from the front and three-quarter. A future round that wants a
twisting pelvis has to pay for it with a Z channel on `hip.L/R`.

**Every frame is keyed, LINEAR.** The glTF exporter samples per frame anyway, so linear keys
make Blender's playback and Bevy's the same curve. Bezier handles at one-frame spacing
overshoot *between* the samples, and an overshoot on the stance leg is foot slide that no
integer-frame check would ever catch.

## 3. The geometry problem this exposed — reported, not fixed

**The bind pose is this rig's tallest standing pose.** Hip head 0.330, sole 0.000, hip-to-ankle
0.200 with the knee dead straight. There is no headroom to bob up into: any upward motion of
the pelvis while the sole is flat lifts the foot off the floor.

The first build did exactly that — the stance foot floated up to 15 mm for six frames of
mid-stance, and the slide table was the only thing that said so. The fix is `ROOT_BASE =
-0.015`: the whole cycle sits 15 mm below bind, and the ±5 mm bob rides under that. So the
walking dwarf is never as tall as the standing one.

Two consequences, both of which I worked around rather than fixed, since geometry is frozen:

- **The knee is never straight.** It runs −82° to −40° through the cycle and never enters the
  ±5° a real walk would show at contact. Almost all of it is invisible: the skirt hangs to
  z 0.2484 and the boot cuff reaches 0.1944, so only ~54 mm of shin is ever on screen.
- **Mid-swing needs 82° of knee flex to clear the ground by 30 mm.** The thigh segment is
  0.068 m against a 0.132 m shin — a split chosen for box ownership, not anatomy — so bending
  the knee is an inefficient way to shorten the leg. 82° buys only 35 mm.

Neither is a defect I could fix inside this brief. If the model revision that is already in
flight wants to help the next animator, the single highest-value change is **binding with the
knee slightly flexed** (hip head 0.335–0.340, or the ankle a few mm higher), which would hand
back the headroom for a proper bob and a straighter contact pose.

## 4. Foot slide — the check that matters most

The clip is in place, so a grounded sole travels backward through body space **on purpose**;
the client's forward motion is what cancels it. So the test is not "does the sole hold still",
it is "does the sole hold still once the client's travel is put back". Columns marked `y*` are
`world y + stride × frame / 24`. A pinned point must not move.

Left boot (the right is the same table shifted 12 frames; both were measured):

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
       14  swing   0.091          -   0.07295          -   0.00488         -
       15  swing   0.182          -   0.06555          -   0.01098         -
       16  swing   0.273          -   0.05599          -   0.01820         -
       17  swing   0.364          -   0.04543          -   0.02620         -
       18  swing   0.455          -   0.03486          -   0.03446         -
       19  swing   0.545          -   0.02500          -   0.04229         -
       20  swing   0.636          -   0.01637          -   0.04896         -
       21  swing   0.727          -   0.00930          -   0.05375         -
       22  swing   0.818          -   0.00402          -   0.05608         -
       23  swing   0.909          -   0.00080          -   0.05549         -
  WORST SLIDE 0.000 mm
```

Read it as three phases. Frames 0–2 the **heel** is pinned at `y* = 0.00880` while the toe
drops from 51.7 mm to the floor — that is the ankle roll through contact. Frames 2–6 the sole
is flat, both ends pinned. Frames 6–13 the **toe** is pinned at `y* = 0.25737` while the heel
lifts to 76.8 mm — the push-off. The pinned column never moves in the fifth decimal, which is
sub-micron; it is exact because the solve inverts this table rather than approximating it.

The pivot switches from heel to toe at frame 6, where both are on the ground, so the switch is
not a discontinuity. Swing clearance: lowest point of the swinging boot is **0.8 mm** at frame
23 — one frame before touchdown, where it should be — and **~25–35 mm** through mid-swing.

## 5. Prop clearance

Lowest world z of each prop, and the nearest surface-to-surface approach to the head assembly
(`r17_head`, `r17_hair`, `r17_beard`, `r17_moustache`), per frame, off the evaluated meshes:

```
  frame   axe min z  axe-head mm   lantern z  lantern-head   boot.L z   boot.R z
      0     0.22296       256.61     0.20558        350.57   -0.00000   -0.00000
      1     0.22457       257.15     0.20285        352.71    0.00000   -0.00000
      2     0.22375       257.13     0.20061        351.81   -0.00000    0.00488
      3     0.22081       257.10     0.19945        347.91   -0.00000    0.01098
      4     0.21733       257.11     0.19924        342.02   -0.00000    0.01820
      5     0.21539       257.17     0.19948        335.80   -0.00000    0.02620
      6     0.21649       257.32     0.19982        330.92   -0.00000    0.03446
      7     0.21955       257.53     0.20028        327.96   -0.00000    0.02500
      8     0.22285       257.80     0.19916        323.97    0.00000    0.01637
      9     0.22642       258.09     0.19714        319.24   -0.00000    0.00930
     10     0.22854       258.34     0.19727        314.90    0.00000    0.00402
     11     0.22841       258.51     0.19919        311.83   -0.00000    0.00080
     12     0.22638       258.57     0.20159        310.51   -0.00000   -0.00000
     13     0.22356       258.51     0.20289        310.97   -0.00000    0.00000
     14     0.22108       258.34     0.20214        313.00    0.00488   -0.00000
     15     0.21954       258.09     0.19970        316.22    0.01098   -0.00000
     16     0.21881       257.80     0.19714        320.18    0.01820   -0.00000
     17     0.21836       257.53     0.19651        324.38    0.02620   -0.00000
     18     0.21778       257.32     0.19922        328.27    0.03446   -0.00000
     19     0.21720       257.17     0.20263        331.23    0.02500   -0.00000
     20     0.21618       257.11     0.20444        332.93    0.01637    0.00000
     21     0.21555       256.54     0.20689        336.09    0.00930   -0.00000
     22     0.21694       256.23     0.20827        341.01    0.00402    0.00000
     23     0.21988       256.27     0.20774        346.22    0.00080   -0.00000
  PASS -- nothing under the floor, nothing touching the skull
```

Worst case: pickaxe **215 mm** above the floor, lantern **197 mm**, pickaxe **256 mm** from the
skull, lantern **311 mm**. Nothing is close to either limit, and no boot goes below zero.

The head clearance is comfortable for a structural reason, not a lucky one: the props sit at
|x| ≈ 0.46–0.55 while the head reaches only |x| ≤ 0.23, and every arm rotation in this cycle is
about world X, which does not change x at all. Swinging the arm can only move a prop
fore/aft and up/down, never inward. **This margin evaporates the moment anyone adds shoulder
roll (Y) or a wrist twist to a future clip** — that is the axis that would drive the pickaxe
into the skull, and it is the one to re-check, not the swing.

The pickaxe's known overshoot (z 0.230 … 1.322 on a 1.200 m dwarf) is untouched. I authored
around it: the arm swing is ±11° and the elbow 0–8°, deliberately small, because a swing that
would read fine on a bare arm rotates a 1.09 m shaft through a quarter of a metre at the
head. The axe stays near vertical with a slight backward lean against the shoulder, which is
also how it reads best.

## 6. In place, and the loop seam

`root` carries **vertical bob only**. Its world x and y are literally constant — not small,
zero — across all 25 frames:

```
  root x span 0.000e+00 m   y span 0.000e+00 m
  root z: -0.0175 at frame 0, low -0.0200 at frames 2 and 14, high -0.0100 at frames 8 and 20
```

So the bob is **±5 mm about a 15 mm crouch**, twice per cycle, lowest just after each contact.
No bone carries net horizontal travel; `blended_translation()` stays the only writer.

Loop seam: frame 0 and frame 24 produce **bit-identical** pose matrices for all 19 bones —
worst element delta `0.000e+00`, not "small".

**The doubled final key is correct and deliberate.** In the exported channel the time accessor
has **25 entries from 0.000 s to 1.000 s**, with the first and last output values identical.
That is what a looping sampler needs: the clip's duration is exactly 1.000 s and t = 1.0 is
the wrap point, so there is no held frame.

This is also a thing I changed after measuring. The first build keyed frames **1…25**, and the
exporter writes glTF time as `frame / fps` — so the clip came out spanning **0.0417 s to
1.0417 s**. Bevy takes clip duration from the maximum keyframe time, so that clip is 1.0417 s
long with nothing before 0.0417 s: **a one-frame hitch on every loop**. Shifting the action to
frames 0…24 fixes it at the source. Worth knowing for any future clip authored here.

## 7. Seams

`seam_check.py`'s leg ranges were widened to what this cycle actually reaches (measured by
`walk_r18.joint_range()`), keeping the old bound wherever it was wider — a union, not a
replacement:

| joint | was | walk uses | now |
|---|---|---|---|
| `hip` | −45 … +35 | −8.5 … +61.4 | −45 … +62 |
| `knee` | 0 … +70 | −82.5 … −39.6 | −83 … +70 |
| `foot` | −25 … +25 | −4.9 … +44.8 | −25 … +45 |
| `beard` | 0 … +25 | ±4 | −4 … +25 |

**Two of the old three were on the wrong side of zero.** `hip` and `knee` point down at bind
with local X = world X, so a *positive* turn swings the segment forward: the old `knee`
(0, +70) was hyperextension, and the walk runs −82 … −40 without ever entering it. The old
`foot` (−25, +25) understated the need — the walk wants +45 of *local* dorsiflexion, not
because the ankle bends that far in the world (absolute foot pitch only runs −18 … +12) but
because the foot has to cancel a knee flexed 40–82° to keep the sole flat.

`seam_check.report()` with the new ranges: **PASS, 0 of 24 pairs exceed 10 mm**, worst 5.1 mm
(belt/torso at `spine X −15`).

That sweep moves one joint at a time, which is the right shape for a range of motion and the
wrong shape for a clip — this walk moves hip, knee and foot together. So I added
`walk_r18.seam_walk()`, which re-measures the same 24 contacting pairs with the same
surface-to-surface BVH metric **at the real posed frames**:

```
  r17_boot.L   r17_leg.L        3.2 mm   frame 20
  r17_boot.R   r17_leg.R        3.2 mm   frame  8
  r17_beard    r17_moustache    2.3 mm   frame 23
  r17_pack     r17_strap.R      1.6 mm   frame  8
  ... 20 more, all under 1.6 mm
  PASS -- 0 of 24 pairs exceed 10 mm during the cycle
```

Round 17's seams stay closed through the poses this cycle actually uses.

## 8. Looked at, not just measured

Rendered to `src-assets/renders/r18/`, Cycles on the GPU, with a temporary checkered floor
that is **not** saved into the .blend:

- `keys-side.png` — the four key poses (contact 0, down 3, passing 6, up 9), his right side
- `keys-front.png`, `keys-threequarter.png` — all eight key poses including the mirrors
- `cycle-side.png` — all 24 frames, two rows of twelve
- `travel-feet.png` — **the skate test by eye**: frames 0–12 with the client's travel applied
  to the figure and the camera left alone, so a planted boot has to hold the same floor tile

I also played it at speed in the viewport in MATERIAL shading from the front, the side and
three-quarter and watched it loop. It reads as a heavy, short-strided trudge: low bob, weight
settling onto the contact foot, shoulders counter-rotating against a pelvis that stays put,
head level, beard trailing a couple of frames. The pickaxe and lantern swing gently and stay
upright. `cycle-side.png` shows no pops between any adjacent pair of frames.

Where the eye caught what the numbers did not: the first pass had an arm swing of ±17° with a
14° elbow, which measured perfectly and looked absurd — the pickaxe raked ~40° across the
figure. That is the brief's warning about props, and it is only visible by looking.

## 9. What I did not do, and what I could not verify

- **The exporter change is not mine and I did not land it.** `export_dwarf.py` still has
  `export_animations=False` at line 661, untouched. I tested the clip by calling
  `bpy.ops.export_scene.gltf(..., export_animations=True)` directly against a scratch GLB and
  reading the result back. `Walk` is the **only** action in the .blend and the only animation
  in the probe GLB: 57 channels (19 joints × translation/rotation/scale), LINEAR on the
  animated tracks, 19 joints in the skin. Flipping the flag as it stands will not gain clips
  nobody asked for.
- **I could not run the gate or the game.** No `scripts/gate.sh`, no Bevy, no `check_asset.py`
  run — I have no Rust toolchain here and `check_asset.py` has no animation clauses to run
  anyway. Nothing in this report is a gate result.
- **The client wiring is untested.** There is no `AnimationPlayer`/`AnimationGraph` in
  `crates/gui`, so "does it skate in game" is unanswered by construction. What I can say is
  that the clip is in place to machine precision and the stride is measured off the planted
  foot, so if it skates the error will be in the playback rate, not in the clip.
- **Sub-frame behaviour is argued, not measured.** Every check samples integer frames. Linear
  keys at one-frame spacing make interpolation exact between samples by construction, which is
  why I chose them, but I did not sample at 0.5-frame offsets to prove it.
- **The action is saved unassigned.** `Walk` is kept on a fake user and
  `animation_data.action` is `None`, so the file's rest state is bind, as asked. If the
  exporter is ever changed to read the *assigned* action rather than scanning
  `bpy.data.actions`, it will find nothing — worth a glance when that change lands.

## 10. Files

| file | state |
|---|---|
| `src-assets/blender/SM_VoxelDwarf_Miner01.blend` | **modified** — `Walk` added; geometry, rig and weights hash identical to HEAD; saved at bind |
| `src-assets/blender/walk_r18.py` | **new** — authors the action and runs every check in this report |
| `src-assets/blender/render_r18.py` | **new** — the pose strips |
| `src-assets/blender/seam_check.py` | **modified** — leg and beard ranges widened to the walk's own extremes |
| `src-assets/renders/r18/*.png` | **new** — five strips |
| `src-assets/blender/dwarf_r17.py` | **untouched** — `hex_rgb` still has no `srgb_to_linear` |
| `src-assets/blender/export_dwarf.py` | **untouched** — `export_animations` still `False` |

Rebuild from scratch in the live session:

```python
exec(open(r"src-assets/blender/walk_r18.py").read())
build(); checks(); seam_walk(); detach()
```
