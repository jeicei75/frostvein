# Story 12.11 seat card: nothing floats (AC 12, about 15 minutes)

**Before this card:** pull the branch on the Windows checkout. `launch-gui.ps1` rebuilds, and it
refuses a `gui.exe` whose stamp is not HEAD.

What you read is the world, not your key press. The sim decides where a stone falls and where a
miner sets one aside; the gui and the tui only draw it.

Two sittings, each on a **fresh daemon**: (A) the staircase and the tree, with a pile; (B) the
tunnel, with **no pile**. A pile you placed in (A) would serve (B), so restart between them.

## Start (both sittings)

WSL, first terminal (fresh daemon, DEFAULT_SEED; `Ctrl+C` and start it again for sitting B):

```bash
simd 7451
```

WSL, second terminal (tui parity; restart it with the daemon):

```bash
tui 7451 --z 8
```

PowerShell, from the Windows checkout (restart it with the daemon):

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

## Where to look

The **campfire is at `[64, 64, 9]`**.

- **The gui is a mirror image of the tui (#169, open).** With north up in both, **east is on the
  LEFT in the gui** and on the right in the tui. Every "east" below means world +x, "west" world -x.
- **The gui boots with north DOWN-LEFT, not up.** Read the compass, and orbit with `A`/`D` until it
  reads N up.
- **tui:** `<` / `>` walk down / up one z-level.

## Sitting A: the staircase and the tree (with a pile)

1. Press `3` and drag a big pile **south of the fire**: about `x 56..72`, `y 66..73`, at z 9. `Esc`.
2. Press `2` (channel) and work 12.10b's staircase north of the fire, one row at a time, each row
   after the previous row's marks are gone. Step the gui slice down with `,` when the rim hides the
   pit floor.

   | Row | Marks at | Cells |
   | --- | --- | --- |
   | 0 | z 9 (the fire's level) | `x 61..65`, `y 62..63` |
   | 1 | z 8 | `x 62..65`, `y 62..63` |
   | 2 | z 7 | `x 63..65`, `y 62..63` |
   | 3 | z 6 | `x 64..65`, `y 62..63` |

   A block that lies on a cell a later row opens under now **drops onto the ground below** in the
   same tick. A block lying on a cell a miner is about to channel is **carried off it by that miner**:
   he lifts it, walks it a few cells, sets it down, and goes back to his channel.
3. Press `1` (dig) and mark **`[67, 53, 11]`**: the ground under the hillside tree whose trunk base is
   `[67, 53, 12]`, north-east of the fire, workable from its own level. When the miner digs it, the
   **whole tree comes down** as a cut does, and its logs land on the ground beside the hole.

**Question 1:** after the staircase reaches row 3 and the tree is down, is anything hanging over
empty air (a block, a log, a trunk, or dig rubble), in the gui and in the tui?

## Sitting B: the tunnel (NO pile)

Restart the daemon and both clients. **Place no pile.**

1. The site is a snowbank **west of the camp at z 13**, about 23 steps' walk from the fire. In the tui
   press `>` five times from `--z 8` to see z 13. The dwarves walk up to it on their own.
2. Press `1` (dig) and mark ONE cell at a time, the next only when the previous one is gone, going
   west:

   | Mark | Cell |
   | --- | --- |
   | 1 | `[40, 63, 13]` |
   | 2 | `[39, 63, 13]` |
   | 3 | `[38, 63, 13]` |
   | 4 | `[37, 63, 13]` |

   The tunnel is one wide (rock and snow on both sides), so each dug stone lies on the only cell the
   next mark can be worked from. Before 12.11 the miner stopped there for good with no pile.

**Question 2:** does the miner carry each stone out of his own way and finish all four marks, with no
hauler and no pile?

**Question 3:** while he carries a stone aside, does he look like a hauler (lift, carry, set down)
and NOT swing his pick, in the gui? (The tui draws no job, so in the tui check only that no stone
hangs and the tunnel finishes.)

Measured on the build before review (a fresh daemon, these cells, no pile, `fast4x`): all four marks
dug by tick 757, the miner reported as `haul` while carrying.

## Known, by design (not defects)

- **Terrain never falls.** A block of rock over an empty cell stays (no cave-ins).
- **The miner never takes a stone to the pile.** He sets it aside; with a pile, a hauler collects it
  later.
- **Miners stay at the bottom of the staircase** (#186) once the last row is worked.
- A dig mark is refused on the cell under the campfire or a torch, and the gui preview does not light
  it. A channel mark is refused on the campfire or a torch, and below z 2.
- A full pile stops the work. Drag a second pile if it fills.
