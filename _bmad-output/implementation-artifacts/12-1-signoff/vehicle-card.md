# Story 12.1 seat card — stones stay out of the fire

Launch the daemon in WSL with `simd 7451`. In PowerShell from the Windows checkout, launch the client with `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`. Attach a second client in WSL with `tui 7451`.

(a) Press `3`, then drag a 5×5 stockpile centred on the campfire.

(b) Press `1`, then dig a few blocks nearby. Press `+` to speed up. Watch the stockpile ring fill with nothing on or in the fire.

(c) Press `3`, then drag one cell on the fire. Confirm `stockpile refused: no valid cells` appears in both the gui and the attached tui.

Wolf: is this the case you saw?

Seat result / AC1 answer: pending Wolf's sitting.
