# Story 12.12 seat card: the first goals (AC 10 and AC 11, about 20 minutes)

**Before this card:** pull the branch on the Windows checkout. `launch-gui.ps1` rebuilds, and it
refuses a `gui.exe` whose stamp is not HEAD.

What you read is the daemon's goal state, not a client's count. The sim decides what is STORED (an
item on a pile cell, not carried, the first item on its cell) and when a goal completes; the gui and
the tui only print the wire's `goals`.

**Start from a fresh world.** Do not load your kept save: a save written before 12.12 is refused by
design (`missing field 'goals_completed'`), and simd says so in its log.

Two sittings, each on a **fresh daemon**: (A) the goal chain, start to finish; (B) a pile ringing the
fire (the pile-ring fix). A pile from (A) would serve (B), so restart between them.

## Start (both sittings)

WSL, first terminal (fresh daemon, DEFAULT_SEED; `Ctrl+C` and start it again for sitting B):

```bash
simd 7451
```

WSL, second terminal (tui parity; restart it with the daemon):

```bash
tui 7451 --z 9
```

PowerShell, from the Windows checkout (restart it with the daemon):

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

## Where to look

The **campfire is at `[64, 64, 9]`**.

- **The goal list (gui) is bottom-right:** a `goals` title over three lines, `[ ] wood  0 / 10` first.
  A done goal is green with `[x]`, the current one white, later ones grey. `H` hides it with the rest
  of the HUD.
- **The goal row (tui) is the row above the roster:** `goal 1 of 3: wood 0/10`, grey, with the done
  goals after it in pile green.
- **The gui is a mirror image of the tui (#169, open).** With north up in both, **east is on the
  LEFT in the gui** and on the right in the tui. Every "east" below means world +x, "west" world -x.
- **The gui boots with north DOWN-LEFT, not up.** Read the compass, and orbit with `A`/`D` until it
  reads N up.
- **Keys (gui):** `1` dig, `3` pile, `4` clear (marks AND piles), `5` cut, `Esc` leaves the mode; `,` /
  `.` step the slice down / up; `=` speeds up (three presses from normal reach `fast4x`), `Space`
  pauses. **tui:** `<` / `>` walk down / up one z-level.

## Sitting A: the chain (about 5 minutes at fast4x after the marks)

All marks can go in at once; the daemon works them in its own order. A line `[x0..x1, y, z]` is one
drag along that row at that level.

1. **Piles first** (`3`), three drags near the fire, none round it:

   | Pile | Drag | Level |
   | --- | --- | --- |
   | West of the fire | `x 57..61`, `y 61..66` | z 9 |
   | East of the fire | `x 66..68`, `y 60..67` | z 9 |
   | South, one step up | `x 58..66`, `y 68..70` | z 10 |

   The pile figures on this card (67 cells here, 20 in sitting B) were measured with flat one-level
   rectangles. The gui's pile drag follows the ground, so where the ground steps your pile can take a
   few cells more or fewer. Read every cell count on this card as approximate.

2. **Cut** (`5`) the three trees, at their trunk base: `[65, 56, 10]`, `[61, 74, 13]`, `[65, 73, 12]`.
3. **Dig** (`1`) the tunnels to the ore. Step the slice to each level first.

   | Run | Drags | Level | Reaches |
   | --- | --- | --- | --- |
   | East | `x 73..78, y 64` and the one cell `[77, 65]` | z 11 | the tetrahedrite vein at `[77, 64, 11]` |
   | South-east | `x 69, y 73..79` | z 12 | tetrahedrite at `[69, 78..79, 12]` |
   | South-east, high | `x 68, y 76..79` and `[69, 79]` | z 14 | tetrahedrite at `[69, 79, 14]` |
   | South-west | `x 51, y 72..77`; `x 48..50, y 77`; `x 48, y 78..80`; `x 46..47, y 80`; `x 46, y 81..83` | z 12 | tetrahedrite at `[51,77]`, `[50,77]`, `[48,80]`, `[46,83]` |
   | Bloodstone | `x 55, y 74..83` | z 13 | the bloodstone at `[55, 83, 13]` |

   These runs hold exactly 10 tetrahedrite, the target, so one missed cell or one stone left lying
   stops the chain at 9 / 10 for good. **The spare:** `[45, 83]` at z 12, the cell just west of the
   South-west run's last cell `[46, 83]`. It is the only tetrahedrite within four cells of any run.
   Dig it at the start with the rest (the measured run then digs 11), or dig it if the count sits
   at 9 once every mark is gone.

4. Speed up to `fast4x` and watch the goal list and the tui row.

Measured on the branch (a scenario-path probe of exactly these drags, from tick 30): wood completes at
tick ~1,960 (about 50 s at fast4x), tetrahedrite and bloodstone at ~10,290 (about 4 minutes at fast4x).
**Bloodstone is stored long before tetrahedrite is done** (around tick 6,500): its grey line reads
`1 / 1` while it waits, and it ticks the moment tetrahedrite does.

**Question 1:** does each goal tick (green `[x]` in the gui; moving into `done:` in the tui) when its
count reaches the target, in order, and never before the one ahead of it?

**Question 2:** when the last one completes, does the gui title turn gold, `all goals done - play
on`, and the tui row read `all goals done - play on: wood .../10  tetrahedrite 10/10  bloodstone
1/1`? Does play go on (the tick keeps climbing, the dwarves keep working)?

5. **Stickiness:** press `4` (clear) and drag over the WEST pile to remove it. The counts drop (its
   items are now loose); the `[x]`s stay.

**Question 3:** do the counts follow the stock down while every goal stays done?

## Sitting B: a pile round the fire (about 2 minutes at fast4x)

Restart the daemon and both clients.

1. Press `3` and drag **`x 62..66`, `y 62..66` at z 9**: a 5 by 5 square centred on the fire (the fire's
   own cell is never zoned).
2. Press `4` (clear) and click the fire's four side neighbours one by one: `[64, 63]`, `[63, 64]`,
   `[65, 64]`, `[64, 65]` (z 9). The pile is now a 20-cell ring with the fire and its four neighbours
   inside it, not zoned. This is the shape that sealed itself before 12.12.
3. Give it something to haul: dig (`1`) the East, South-east and South-east-high runs of sitting A,
   and cut (`5`) the three trees.
4. Speed up to `fast4x`.

Measured on the branch (same probe): 20 of 20 cells filled by tick ~3,030 (about 75 s at fast4x).
Before the fix, the creation's 58-cell ring stopped at 50 of 58 with 8 stones lying loose for good.

**Question 4:** does the ring fill to the last cell, the four cells next to the fire included, with no
stone left lying beside a free pile cell?

## Known, by design (not defects)

- **Some items stay loose after the chain** (in the measured runs 3-4 tetrahedrite, 4 logs, a
  bloodstone): the piles fill with stones dug on the way. A goal counts only what is stored.
- **A pile placed over items while the game is paused** shows the new count at once but completes
  nothing until the next tick (D1).
- **A miner clearing his own way may lift an item off a pile** (12.11); a goal already done stays done.
- **A big pile (several hundred cells) slows the daemon** (about 3.5 ms a tick, before and after 12.12).
  Keep the piles to the sizes above.
- Goals cannot be skipped, reset or reordered, and there is no reward: they only count.
