# Story 12.5 seat card — dwarves at work: dig and haul

**Before this card:** the round-19 BlenderMCP seat (`src-assets/prompts/dwarf-miner-round-19.md`)
has produced `Dig` and `Carry`, and they have been exported, gated and promoted (Task 2c). Pull the
branch on the Windows checkout. `launch-gui.ps1` rebuilds and checks that the binary's stamp matches
HEAD, and the clips are embedded with `include_bytes!`, so a stale binary would still be on Walk only.

The launcher's console shows the startup line, `gui dwarf asset: … clips Walk, Dig, Carry`. If it
says `clip Dig ABSENT` or `clip Carry ABSENT`, stop: the promoted GLB is not in this build.

Judge at **Normal** speed. Do not press `+`. At fast-forward a dig run is 25 ms and the swing is
invisible; that is expected, not a defect.

## (a) The miner swings

WSL: `simd 7451` (fresh: restart it before this part).

PowerShell, from the Windows checkout:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--select','2')
```

This follows **Bifur, the miner in red**. He stays centred in the frame, and his name shows top
right, under the clock.

1. Press `2`, then drag a block of channel marks on the camp floor east of the fire, about 4×7.
2. Press `3`, then drag a 3×3 stockpile west of the fire.
3. Watch Bifur walk to a mark and swing. One swing is one dig (0.5 s), and the strike lands just
   before the tile changes. He digs a channel under his own feet, so he keeps the heading he
   arrived with.
4. The console prints `gui dwarf 2 clip dig` at each swing and `gui dwarf 2 clip walk` after it.

**Question 1:** does the swing read as a dwarf digging?

## (b) The hauler carries

WSL: restart `simd 7451`.

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--select','1')
```

This follows **Ori, the hauler in green**.

1. Make the same designation as in (a): `2` for channel marks east of the fire, `3` for a 3×3
   stockpile west of it.
2. Wait for the dug stones. Ori walks to one, picks it up, and carries it to the pile, holding it
   in both hands in front of his chest. The stone moves with him, not cell by cell.
3. On delivery the stone is put down on the pile cell, and he goes back to walking empty-handed.
4. The console prints `gui dwarf 1 clip carry` at pick-up and `gui dwarf 1 clip walk` at the drop.

**Question 2:** does the carry read as a dwarf carrying a stone, and do his feet hold the ground
without skating?

## Art ledger

Each round gets a row in the story's "Art ledger", with Wolf's words. **Hard stop:** after two
rounds that your eye has not judged converging, the story asks whether the clips ship plain or are
parked.

Seat result: _(pending)_
