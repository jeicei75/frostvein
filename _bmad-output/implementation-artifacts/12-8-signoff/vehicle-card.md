# Story 12.8 seat card — the cut (AC9)

**Before this card:** pull the branch on the Windows checkout. `launch-gui.ps1` rebuilds, and it
refuses a `gui.exe` whose stamp is not HEAD.

**Two passes.** Pass 1 runs today: #164's timing, #174's cut mode and #173's line. Pass 2 runs
once round 20's `Cut` clip is promoted into the GLB (Task 2c).

**Until then the launcher console says `clip Cut ABSENT`, and Nain plays the Dig clip** at the
cut's 10-tick period: a slow pick swing. That is the fallback, not the chop. Judge the chop only
in pass 2.

What you read is the world, not your key press. A mark appears only when the daemon's next delta
says so.

## Start

WSL, first terminal (fresh daemon):

```bash
simd 7451
```

WSL, second terminal, for parity:

```bash
tui 7451 --z 12
```

PowerShell, from the Windows checkout:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

Keep the launcher console visible. The gui prints `gui trees: materials=` and
`gui dwarf N clip X` there.

## (a) #173's line, on a fresh world

Right after the trees load, the console reads `gui trees: materials=4 [M_VoxelPine:259]`. That is
four handles, one per pine variant, with the count matching `gui trees: meshes=`.

**If you ever see a green pine you did not mark,** copy the latest `gui trees: materials=` line.
Note what you did just before: save/load, `--assets`, a speed change. That goes into #173.

## (b) Cut mode: pointer, box and tint (#174, against `12-8-signoff/draft.md`)

1. Press `5`. Sweep the cursor slowly across the crown of a pine near the fire (x 73, y 56-59).
   The hover slab stays at **that pine's foot**, and it no longer jumps to the ground behind the tree.
2. Press `1` and sweep the same crown. Dig mode still sees the ground through the crown, as before.
   Press `Esc`, then `5` again.
3. Drag a box over **open ground** first, without releasing. Every cell of the box shows a thin hover
   slab, so you can see the box's size. Press `Esc` to abort.
4. Drag across the two pines nearest the fire. **While you drag**, the pines the box catches turn
   **flat green**. Release. Only the pines stay marked: each keeps the green and a slab at its foot.
5. Press `4` and drag over one marked pine's foot. Its green and its slab both go, and it is snowy
   again. The console's `cut-tint:N` drops by one.

**Question 1:** does the cut mode read as in the draft: the pointer at the tree, the whole box, the
flat-green pine? Is the green the right strength next to the snowy pines?

## (c) Dig, haul and cut at once, at Normal (AC9)

Set trades if needed: select a dwarf and press `T`.

1. Press `3` and drag a 3×3 pile on the camp floor west of the fire.
2. Press `1` and drag a small dig.
3. Press `5` and drag across the pines nearest the fire. Then watch, at **Normal**:
   - a miner swings at the dig;
   - a hauler carries, and a log or stone **leaves the ground only when his body reaches it**, and
     is set down only when his body reaches the pile cell (#164);
   - Nain walks out and works the pine: the Dig fallback in pass 1, **the chop in pass 2**, five chops
     a tree;
   - all at the same time.

**Question 2 (pass 2):** does the chop read as an axe at a trunk, distinct from the dig swing? Do
dig, haul and cut read together?

## (d) The same at Fast (#164 at Fast)

Press `+` once (Fast). Select the hauler (`--select 1` is Ori) and watch one pick-up, then mark
another pine and watch the chop.

- The walkers now move five times faster, like a fast-forwarded film. They arrive in time to show
  the work.
- Nothing jumps into a hand before the dwarf is there.

**Question 3:** does the timing hold at Normal and at Fast?
