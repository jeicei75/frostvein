# Story 12.4 seat card — every dwarf has a trade

Launch the daemon in WSL with `simd 7451`. In PowerShell from the Windows checkout, launch the client with `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`. Attach a second client in WSL with `tui 7451`.

Old `frostvein.save` files will not load: a save from before trades existed is refused with a `could not decode frostvein.save` log line. Do not press Ctrl+L on an old save expecting it to work.

(a) In the attached tui, the roster row (third line from the bottom) names each dwarf in his tunic colour, then his trade in grey. On a fresh `simd 7451` (DEFAULT_SEED) it reads `Nain woodcutter  Ori hauler  Bifur miner  Frar hauler  Dori miner`: two miners, two haulers, one woodcutter.

(b) In the gui, press `2`, then drag a block of channel marks on the camp floor east of the fire (about 4×7). Press `3`, then drag a 3×3 stockpile west of the fire. Press `+` to speed up.

(c) Watch a dwarf carry a stone to the stockpile while channel marks are still standing. On the devpod the first stone lands with 17 of 25 marks left (`first_delivery.py`). Before this story the marks all went first, and the first stone landed after the last one.

The woodcutter has no work yet (that comes in 12.7). He just wanders.

Seat result (Wolf, 2026-10-02): "1 ok". He also asked whether the gui should show the full roster and trades, which would need a small design pass. It was not built in this story.
