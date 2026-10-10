# Story 12.9 seat card: one dwarf per tile, stones block (Task 9, about 5 minutes)

**Before this card:** pull the branch on the Windows checkout. `launch-gui.ps1` rebuilds, and it
refuses a `gui.exe` whose stamp is not HEAD.

What you read is the world, not your key press. The sim decides who steps, who waits and where a
stone lands, and the gui draws it.

## Start

WSL, first terminal (fresh daemon):

```bash
simd 7451
```

PowerShell, from the Windows checkout:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

## (a) The busy camp, at Normal

1. Press `2` and drag a channel block on the camp floor **east** of the fire, about 4×7 cells.
2. Press `3` and drag a 3×3 pile **west** of the fire.
3. Watch for a minute at **Normal**:
   - **No two dwarves ever stand inside each other.** Before this story, two would often merge
     into one body.
   - A dwarf whose way is taken **waits a step, goes round, or steps aside**. A dwarf caught
     facing another in a narrow spot backs off.
   - **Nobody walks through a stone.** Dwarves go round loose stones and stones on the pile.
   - A hauler **picks a stone up from the cell beside it**, and **puts it down on the pile cell
     beside him**. He no longer stands inside the stone.
   - **The pile fills from the centre outward.** The middle cell gets a stone first, then the ring.

**Question 1:** do they read as bodies in a real place? Does the waiting or backing off look
natural, or does anyone look stuck?

## (b) The same at Fast

Press `+` once (Fast) and watch for another minute: the same points as (a).

**Question 2:** does it hold at Fast? Note anything that looks like a dwarf freezing, jittering
back and forth, or a stone landing somewhere odd.

## (c) Pass 2: the seat pass 1 fixes

1. Drag a small channel. **The miner stands beside the cell he channels** and faces it, not on top of it.
   The stone lands on the channelled cell.
2. Watch a hauler pick up and drop, at Normal and then at Fast:
   - the stone **stays on its cell until his hands reach it**, then **rises into his hands** in about a
     third of a second. It no longer jumps in from a cell away;
   - logs are carried, not pushed ahead of him;
   - at the pile, the stone is **set down onto the cell beside him**, not thrown.

**Question 3:** do the channel and the carry now read right?

## Known, by design (not defects)

- In a **one-wide tunnel**, digging pauses at each new stone until a hauler clears it. With no
  pile, it stops.
- Two dwarves sealed in one pocket with no way past each other both wait.
