# AC2 — the seam changed the frame, measured

**Taken 2026-09-06 on the devpod, commit `bbb6e51`, against one paused daemon.**

```
window 612,570..642,610 (1,200 px), tight on ONE wire dwarf
  noise   cube-a vs cube-b     raw=   0   >=4=   0   >=16=   0
  noise   mesh-a vs mesh-b     raw=   0   >=4=   0   >=16=   0
  change  cube-a vs mesh-a     raw= 411   >=4= 313   >=16= 192
  change  cube-b vs mesh-b     raw= 411   >=4= 313   >=16= 192
```

**The same-build noise floor is EXACTLY ZERO** — one binary, two runs, byte-identical pixels in
this window — so the change clears any multiple, not merely 10x. Both change rows are identical,
which is the measurement reporting its own determinism.

## The recipe changed, and the old controls could not have supported this AC

The committed controls were originally captured **unpaused**, with `--frames 160`. The dwarves
wander, so the same-build noise floor contained dwarf-sized differences and the signal could never
separate from it. Measured before the change, in a 12,000 px window at the camp:

```
  noise   mesh-a vs mesh-b     raw= 8,034   >=4= 5,806   >=16= 1,081
  change  control-a vs mesh-a  raw= 7,973   >=4= 6,105   >=16= 2,940
```

**The change was SMALLER than the noise on the raw channel.** The story had already corrected this
AC once, from a whole-frame bar to a windowed one; that fixed the wrong axis. The unfixed one was
the moving subject.

## Three confounders, removed one at a time

Each was measured, not assumed:

1. **The dwarves wander.** Fixed by pausing the simulation (`set_speed: paused`, now bound to
   space) and capturing with `--static-world`. `dwarf position changes=0`.
2. **The campfire, torches and lanterns flicker on wall-clock time, which pausing the SIM does not
   stop.** This was the entire remaining floor, and it hits a detailed mesh far harder than a flat
   cube because it has vastly more lit silhouette:

   ```
   mesh noise, flicker on    raw= 7,709   >=4= 5,063   >=16= 84
   mesh noise, flicker off   raw=    28   >=4=    22   >=16= 12
   ```

3. **A 12,000 px window dilutes the signal and not the floor.** Tightening onto one dwarf took the
   floor to zero. The window was not chosen by eye: it is the bounding box of the largest blob in
   the change itself, 192 px at x 619..634, y 576..603 — which is what a dwarf looks like.

## The control is this build with the dwarf forced down the cube arm

Not the pre-seam binary. A one-line edit (`if mirror_entity.kind == EntityKind::Dwarf` ->
`if false`) so both sides share every other commit, and the measurement isolates the seam rather
than everything that landed alongside it. The pre-seam binary also predates `--static-world` and
so cannot capture a paused world at all.

## Reproducing

```
./target/debug/simd 0                       # note the port
python3 pause.py <port>                     # {"type":"set_speed","speed":"paused"}
./target/debug/gui <port> --headless --subdiv 1 --frames 160 --static-world \
    --lights-off campfire,torches,lanterns --capture mesh-dwarves-a.png
python3 window_diff.py control-cube-dwarves-a.png mesh-dwarves-a.png "change" 612 570 642 610
```
