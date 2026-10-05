# Story 12.7 seat card — Timber (AC12)

**Before this card:** pull the branch on the Windows checkout. `launch-gui.ps1` rebuilds and refuses
a `gui.exe` whose stamp is not HEAD. Judge at **Normal** speed and do not press `+`.

**Where to look.**
- The **hint bar is bottom-left**. Refusals show just above it, where a refused stockpile shows.
- **Cut marks sit at the pines' feet:** a flat cyan-green slab around the trunk, at ground level.
- **The logs lie at the foot of the felled pine**, stacked one on another: brown boxes, longer than a
  stone.

What you read is the world, not your key press. A mark only appears when the daemon's next delta
says so.

## Start

WSL, first terminal (fresh daemon):

```bash
simd 7451
```

WSL, second terminal, for the parity check. The two pines nearest the fire stand on z 12-13, and the
camp is at z 9:

```bash
tui 7451 --z 12
```

PowerShell, from the Windows checkout:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

## (a) A pile, then mark the pines

1. Press `3` and drag a 3×3 stockpile on the camp floor west of the fire, then press `Esc`.
2. Press `5`. The hint bar reads `cut: drag over the foot of the trees  Esc leave`, and while you
   drag it reads `cut: release to mark  Esc abort`.
3. Drag across the foot of the two pines nearest the fire. They are about 11 tiles out, at x 73,
   y 56-59; the cursor readout gives the cell. While you drag, the preview lights the tree cells
   the drag will catch.
4. Release. Each pine you caught gets **one** cyan-green slab at the foot of its trunk.

**Question 1:** do the marks read at the pines' feet, and does the colour sit apart from the dig
blue, the channel violet and the stockpile teal?

## (b) Nain fells one whole pine

1. **Nain (purple)**, the woodcutter, walks out to one of the marked pines. On the wire he picks the
   job up about 1 s after the mark, and the walk takes about 20 s.
2. At its foot he plays the **dig swing, facing the trunk**: the placeholder until 12.8.
3. After about 5 s of work, **the whole pine is gone on one tick**, trunk and crown together. The
   other pine stays standing, even though its crown touched the first one's.
4. **One log per trunk cell** lies stacked at the felled pine's foot: 4 for the pine at y 59, 3 for
   the one at y 56.
5. A hauler carries the logs to your pile, one at a time, with the Carry clip, just as he carries
   stones. On the wire the first log reached a pile about 34 s after the felling.

In the tui (`tui 7451 --z 12`), a mark is a `/` in orange-red, and the logs show as `=`. One
pine's foot is on z 12 and the other's on z 13 (`>` steps up one level). Switch the tui to the
pile's level (`<` steps down one level, or relaunch with `--z 9`): a log on the pile is a green `=`.

**Question 2:** do the felling and the stacked logs read, and does the dig swing work as a
placeholder?

## (c) Clear a mark with `4`

1. Press `4` and drag over the foot of the pine that is still marked (mark a fresh one with `5` if
   both have fallen). Its slab goes, and the tui `/` goes with it.
2. If Nain was already walking to that pine, he stops and lets go.

## (d) A cut over nothing

Press `5` and drag on the bare camp floor, where there are no trees. The slot above the hint bar
reads `cut refused: no tree`, and the tui status row reads the same.

**Allowed by your ruling (Task 0.3):** a channel drag over the edge of a forest now raises
`channel refused: nothing to channel` for the treetop rows, while the ground rows still apply.
