# AC2 — `--assets` reads different bytes, measured and looked at

**Taken 2026-09-07 on the devpod, commit `8cdab98`, against one paused daemon on port 7493.**

The control is this same binary with **no flag**; the variant is the same binary with `--assets`
pointed at a directory whose `gltf/SM_VoxelDwarf_Miner01.glb` is **a pine `.glb` copied under the
dwarf's name**. Same commit, same daemon, same paused world, one flag apart — so the measurement
isolates *where the bytes came from* and nothing else.

```
window 612,570..642,610 (1,200 px), Part A's window, tight on ONE wire dwarf
  floor   embedded-a vs embedded-b   raw=     0   >=4=     0   >=16=     0
  floor   disk-a     vs disk-b       raw=     0   >=4=     0   >=16=     0
  change  embedded-a vs disk-a       raw= 1,026   >=4= 1,026   >=16=   938
  change  embedded-b vs disk-b       raw= 1,026   >=4= 1,026   >=16=   938
```

**The same-build floor is EXACTLY ZERO on both builds**, so the change clears any multiple rather
than merely the 10x the AC asks for. 1,026 of 1,200 px is **85.5% of the window**. Both change rows
are identical, which is the measurement reporting its own determinism.

## And nothing else changed — the half a delta cannot answer on its own

A change figure says something moved. It cannot say *only* the intended thing moved. Two more
windows, each with **its own floor measured in that same window**:

```
  upper-left  100,100..300,300 (40,000 px)   floor= 74/72/72    change= 72/71/71
  lower-right 1000,500..1200,700 (40,000 px) floor=  0/ 0/ 0    change=  0/ 0/ 0
```

**The upper-left "change" is BELOW its own floor.** That is not a small change, it is no change —
the animated snow, which `--static-world` does not stop because it is not simulation state. Had I
published the 72 without measuring the floor beside it, it would have read as the feature leaking
across the frame. The disk tree's four pines are byte-identical copies of the embedded ones, so
the trees are exactly the control this needs: they did not move, and the dwarves did.

## The whole frame is still the wrong instrument, re-confirmed

```
  whole frame 0,0..1280,720 (921,600 px)   floor= 7,809/5,538/3,372   change= 14,238/11,824/9,242
```

Change over floor is **1.8x**. A whole-frame 10x bar would fail this AC while the feature works
perfectly — Part A's finding, re-measured on a different change rather than inherited.

## Looked at, not only counted

`ac2-crop-embedded.png` and `ac2-crop-disk.png` are the same 140x100 region at 6x
(`crop.py`, `580,520..720,620`). **Where five dwarves stand in one, five pines stand in the other.**

This is the check the counts cannot make. `gui dwarves: meshes=5` counts `WorldAssetRoot` entities,
and it would have printed `meshes=5` just as happily if the disk scene had loaded and drawn
*nothing* — five entities rendering empty. The startup line's `scenes_loaded=true` argues against
that; the crop settles it.

## Reproducing

```bash
./target/debug/simd 7493 &
python3 pause.py 7493                       # {"type":"set_speed","speed":"paused"}
mkdir -p /abs/tree/gltf /abs/tree/trees
cp assets/trees/*.glb /abs/tree/trees/
cp assets/trees/SM_VoxelPine_Tree01.glb /abs/tree/gltf/SM_VoxelDwarf_Miner01.glb

COMMON=(--headless --subdiv 1 --frames 160 --static-world --lights-off campfire,torches,lanterns)
./target/debug/gui 7493 "${COMMON[@]}" --capture ac2-embedded-a.png
./target/debug/gui 7493 "${COMMON[@]}" --assets /abs/tree --capture ac2-disk-a.png
python3 ../10-5-signoff/window_diff.py ac2-embedded-a.png ac2-disk-a.png "change" 612 570 642 610
python3 crop.py ac2-disk-a.png ac2-crop-disk.png 580 520 720 620 6
```

`--static-world` and `--lights-off campfire,torches,lanterns` are **not optional**: Part A measured
the change SMALLER than the noise without them. All four captures exit **101** on the near-white
ceiling breached on `main` since before 10.7 (issue #72's neighbour); the PNG is still written, and
a red capture exit is not evidence about this story.
