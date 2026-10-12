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

WSL, second terminal (tui parity; z 11 is the tetrahedrite's level, see "Where to look" below):

```bash
tui 7451 --z 11
```

PowerShell, from the Windows checkout:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

## Where to look

The **campfire is at `[64, 64, 9]`**. Read on a fresh DEFAULT_SEED daemon (review patch, 2026-10-11):

- **The gui is a mirror image of the tui (#169, open).** With north up in both, **east is on the
  LEFT in the gui** and on the right in the tui. Every "east" below means world +x.
- **The gui boots with north DOWN-LEFT, not up.** Read the compass. The positions below are given at
  boot; to read them north-up instead, orbit with `A`/`D` until the compass reads N up.
- **Tetrahedrite:** the nearest is at `[77, 64, 11]`, 3 below its column's surface, in the hill
  east of the fire. The tui at `--z 11` shows it as a blue `%`, with more veins beside it.
- **Bloodstone:** use **`[80, 72, 10]`**, 6 below its surface, in the hill south-east of the fire.
  In the gui, slice down to z 10: at boot it is **right of the fire**. z 10 holds other bloodstones,
  so go by the position, not by looking for a single one. In the tui, press `<` once from z 11: it
  is a pale-teal `$` low on the right. (The `ore_wire.py` target `[55, 83, 13]` is off the tui's
  80×24 frame at `--z 13`, which shows no bloodstone at all.)

## (a) Ore in the rock

1. On a fresh world **no ore shows anywhere**. Ore is only inside the rock.
2. Press `,` to step the slice down into the hill east of the camp. The cut face shows
   **tetrahedrite veins in deep blue**, as short winding runs, on the grey-blue stone.
3. Keep going down to z 10. In the hill south-east of the fire (right of the fire in the gui at
   boot), a **bloodstone in pale teal** shows at `[80, 72, 10]`.

**Question 1:** do the veins and the bloodstone read as ore in the rock, and are they easy to tell
from stone, soil and ice?

## (b) Dig to a vein

1. Press `3` and drag a 3×3 stockpile beside the fire, then `Esc`.
2. Press `1` and drag a tunnel line from the hillside to a vein. **The slope cells light in the
   preview and take marks.** Before this story a hillside refused a dig.
3. Watch a miner dig in. When he reaches the vein, **a blue cube** comes out where the cell was,
   not a grey stone, and a hauler carries it to the pile.
4. In the tui at z 11, the ore cell is `%` in blue and the dug item is a blue `•`. The pile is at
   the fire's level, z 9: **press `<` twice** to see the item turn **green** on it. At z 11 the
   pile shows nothing.

**Question 2:** does the ore come out and reach the pile, in both clients?

## Known, by design (not defects)

- A dug slope cell can leave a dwarf standing on air for a moment, as digging any rock already can
  (12.11).
- In a one-wide tunnel, digging pauses at each new item until a hauler clears it (12.9). A pile
  holds one item per cell, so **a full pile stops the tunnel**: drag a second pile if it fills.
- No channel goes down a level yet (12.10b, #163).
