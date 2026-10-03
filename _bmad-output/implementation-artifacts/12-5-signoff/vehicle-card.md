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
3. Watch Bifur walk to a mark and swing. A dig is now **ten swings, 5 s** (your ruling at the first
   sitting), and the tile changes as the tenth strike lands. The pick is one-handed and swings on his RIGHT side, so the blade
   lands about 0.7 m ahead and to his right. A channel is dug under his own feet, so he keeps the
   heading he arrived with rather than turning to a rock face.
4. The console prints `gui dwarf 2 clip dig` at the start of each dig and `gui dwarf 2 clip walk` after it.

**Question 1:** does the swing read as a dwarf digging?

## (b) The hauler carries

WSL: restart `simd 7451`.

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--select','1')
```

This follows **Ori, the hauler in green**.

1. Make the same designation as in (a): `2` for channel marks east of the fire, `3` for a 3×3
   stockpile west of it.
2. Wait for the dug stones. Ori walks to one, picks it up, and carries it to the pile. He holds it
   out in front by its back edges, arms straight, with the pickaxe slung on his pack. The stone
   moves with him, not cell by cell. It is the 0.64 m stone you picked at the seat, so from the
   front it hides him except his hair and boots. Look from the side or three-quarter.
3. On delivery the stone is put down on the pile cell, and he goes back to walking empty-handed.
4. The console prints `gui dwarf 1 clip carry` at pick-up and `gui dwarf 1 clip walk` at the drop.

**Question 2:** does the carry read as a dwarf carrying a stone, and do his feet hold the ground
without skating?

## Art ledger

Each round gets a row in the story's "Art ledger", with Wolf's words. **Hard stop:** after two
rounds that your eye has not judged converging, the story asks whether the clips ship plain or are
parked.

Seat result: _(pending)_
