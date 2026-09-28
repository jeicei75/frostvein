# Story 12.1 seat card — stones stay out of the fire

Launch the daemon in WSL with `simd 7451`. In PowerShell from the Windows checkout, launch the client with `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`. Attach a second client in WSL with `tui 7451`.

(a) Press `3`, then drag a 5×5 stockpile centred on the campfire.

(b) Press `1`, then dig a few blocks nearby. Press `+` to speed up. Watch the stockpile ring fill with nothing on or in the fire.

(c) Press `3`, then drag one cell on the fire. Confirm `stockpile refused: no valid cells` appears in both the gui and the attached tui.

(d) Once the stockpile is full, check the attached tui. Every stockpile cell around the `♨` shows a **green** `*` (a stored stone), and loose stones elsewhere stay grey (Task 8). The fire's own tile is never green. In the gui, no stockpile cell shows a heap of stones (#153, Task 7).

Note: Windows Terminal draws `♨` and the dwarf glyphs as wide emoji that overlap the next tile to the right (#152, not fixed in this story). A `*` that seems to share the fire's tile is the stone on the tile east of it.

Wolf: is this the case you saw?

Seat result / AC1 answer: pending Wolf's sitting.
