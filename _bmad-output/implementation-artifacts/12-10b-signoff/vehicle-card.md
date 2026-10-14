# Story 12.10b seat card: a way down (AC 12, about 10 minutes)

**Before this card:** pull the branch on the Windows checkout. `launch-gui.ps1` rebuilds, and it
refuses a `gui.exe` whose stamp is not HEAD.

What you read is the world, not your key press. The sim decides what a channel opens and where the
block lands; the gui and the tui only draw it.

**Dig or channel:**

- A **dig** (`1` in the gui, `d` in the tui) takes out the block at the miner's own level, so it goes
  sideways into a hillside.
- A **channel** (`2` in the gui, `c` in the tui) goes a step down. It digs out the block under the
  marked cell and turns the block under that into a ramp, so dwarves walk down into the hole and
  back up.

## Start

WSL, first terminal (fresh daemon, DEFAULT_SEED):

```bash
simd 7451
```

WSL, second terminal (tui parity; z 8 is the first level down):

```bash
tui 7451 --z 8
```

PowerShell, from the Windows checkout:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

## Where to look

The **campfire is at `[64, 64, 9]`**. The staircase site is just **north of the fire**: cells
`x 61..65`, `y 62..63`, at the fire's level, z 9. Six solid levels lie under it, and it has no torch.

- **The gui is a mirror image of the tui (#169, open).** With north up in both, **east is on the
  LEFT in the gui** and on the right in the tui. Every "east" below means world +x.
- **The gui boots with north DOWN-LEFT, not up.** Read the compass. To read the site north-up, orbit
  with `A`/`D` until the compass reads N up. The site is then just above the fire, and its rows run
  toward the **left** (east, +x).
- **tui:** `<` / `>` walk down / up one z-level. Start at `--z 8`, and press `>` once to see z 9.

## (a) A big pile first

Four levels two wide dig **28 blocks**, and a pile holds one block per cell. With no room, the crew
stops (they never drop a block anywhere but a pile).

1. Press `3` and drag a big pile **south of the fire** (away from the site): about `x 56..72`,
   `y 66..73`, at z 9. Then `Esc`.

## (b) Channel the staircase, one level at a time

Press `2` (channel). Drag each row, wait until its marks are all gone, then drag the next. Each row
is **one cell shorter**: it starts one cell further east and is dragged on the pit floor the previous
row opened. Its west end, left unmarked, is the step back up.

| Row | Marks at | Cells | What opens |
| --- | --- | --- | --- |
| 0 | z 9 (the fire's level) | `x 61..65`, `y 62..63` | z 8 opens; a ramp at z 7 |
| 1 | z 8 | `x 62..65`, `y 62..63` | z 7 opens; a ramp at z 6 |
| 2 | z 7 | `x 63..65`, `y 62..63` | z 6 opens; a ramp at z 5 |
| 3 | z 6 | `x 64..65`, `y 62..63` | z 5 opens; a ramp at z 4 |

The preview lights only cells a channel will take, so a lit cell is on the right level. If the rim
hides the pit floor, step the slice down with `,` until it shows. In the tui, walk to the row's
level with `<` and mark it there with `c`, then `Enter` on each corner.

1. Row 0: miners stand at the rim and channel. Each dug block comes **up top, beside the miner**, never
   into the hole, and a hauler carries it to the pile.
2. From row 1 on, miners **walk down the ramps** into the pit to work the next row, and haulers walk
   down to lift blocks and **carry them back up** to the pile.
3. In the tui, at `--z 8` and lower, the pit shows as open cells with ramps, and dwarves inside it.
   Press `>` to watch a hauler arrive at z 9.

**Question 1:** do miners walk down and haulers carry blocks back up, in the gui and the tui?

**Question 2:** does the staircase reach row 3 (z 5 open) without the crew jamming at the dig face?

## Known, by design (not defects)

- **Blocks can float.** A block that landed on a cell a later row opened under stays in the air until
  12.11 (#135). About 6 on this two-wide staircase (the review measured 6 on 2 runs); a one-wide one
  measured 4.
- **Miners stay at the bottom.** Idle dwarves wander on their own level only (#186), so once the last
  row is worked the miners idle in the pit. Haulers come up on their own, carrying blocks. To bring a
  miner up, mark something up top (a channel or a dig at the rim).
- **A one-wide staircase works too** (#194, #196): when a dwarf stands in the way at the dead end,
  idle or busy, the dwarves coming down back out of the cut to let it past, one at a time. It is
  slower than two wide. Nobody swaps places.
- A full pile stops the work. Drag a second pile if it fills.
- A channel mark is refused on the campfire or a torch, and below z 2.
