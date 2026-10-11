# Story 12.10 seat card: ore in the rock (AC 10, about 5 minutes)

**Before this card:** pull the branch on the Windows checkout. `launch-gui.ps1` rebuilds, and it
refuses a `gui.exe` whose stamp is not HEAD.

What you read is the world, not your key press. The sim places the ore and decides what a dig
yields; the gui and the tui only draw it.

## Start

WSL, first terminal (fresh daemon, DEFAULT_SEED):

```bash
simd 7451
```

WSL, second terminal (tui parity; Z is the tetrahedrite target's z, see "Where to look" below):

```bash
tui 7451 --z Z
```

PowerShell, from the Windows checkout:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

## Where to look

From `ore_wire.py` on a fresh DEFAULT_SEED daemon (dev, 2026-10-11):

- the nearest **tetrahedrite** is at `[77, 64, 11]`, 3 below its column's surface, in the hill
  **east** of the camp (the fire is near `[62, 66, 9]`). Use `--z 11` in the tui;
- the nearest **bloodstone** is at `[55, 83, 13]`, 6 below its surface, in the hill **south-west**
  of the camp. Use `--z 13` to see it.

## (a) Ore in the rock

1. On a fresh world **no ore shows anywhere**. Ore is only inside the rock.
2. Press `,` to step the slice down into the hill east of the camp. The cut face shows
   **tetrahedrite veins in deep blue**, as short winding runs, on the grey-blue stone.
3. Keep going down. Five or more levels into the rock, a lone **bloodstone in pale teal** shows.

**Question 1:** do the veins and the bloodstone read as ore in the rock, and are they easy to tell
from stone, soil and ice?

## (b) Dig to a vein

1. Press `3` and drag a 3×3 stockpile beside the fire, then `Esc`.
2. Press `1` and drag a tunnel line from the hillside to a vein. **The slope cells light in the
   preview and take marks.** Before this story a hillside refused a dig.
3. Watch a miner dig in. When he reaches the vein, **a blue cube** comes out where the cell was,
   not a grey stone, and a hauler carries it to the pile.
4. In the tui, the ore cell is `%` in blue; the dug item is a blue `•`, and it turns **green** on the
   pile.

**Question 2:** does the ore come out and reach the pile, in both clients?

## Known, by design (not defects)

- A dug slope cell can leave a dwarf standing on air for a moment, as digging any rock already can
  (12.11).
- In a one-wide tunnel, digging pauses at each new item until a hauler clears it (12.9). A pile
  holds one item per cell, so **a full pile stops the tunnel**: drag a second pile if it fills.
- No channel goes down a level yet (12.10b, #163).
