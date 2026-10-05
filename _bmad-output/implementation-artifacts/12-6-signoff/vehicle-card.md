# Story 12.6 seat card — Wolf gives them their trades (AC11)

**Before this card:** pull the branch on the Windows checkout. `launch-gui.ps1` rebuilds and refuses
a `gui.exe` whose stamp is not HEAD. Judge at **Normal** speed; do not press `+`.

**Where to look.** The roster is **top-right, under the clock**. Refusals are **bottom-left**, where a
refused stockpile shows. The trade only changes when the daemon's next delta says so. What you read is
the world, not your key press.

## Start

WSL, first terminal (fresh daemon):

```bash
simd 7451
```

WSL, second terminal, for the parity check (the camp is at z 9 on the shipped seed):

```bash
tui 7451 --z 9
```

PowerShell, from the Windows checkout:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

## (a) The roster, nothing selected

Top-right, under the clock, one line per dwarf in id order. Each name is in his tunic colour and his
trade is in grey:

```
Nain   woodcutter
Ori    hauler
Bifur  miner
Frar   hauler
Dori   miner
```

The tui's roster row (third from the bottom) says the same trades.

**Question 1:** does the roster read, and does it sit well under the clock?

## (b) Give the digger a new trade

1. Press `2`, then drag a block of channel marks on the camp floor east of the fire, about 4×7.
2. Press `3`, then drag a 3×3 stockpile west of the fire.
3. Press `Esc` to leave stockpile mode; a click in that mode starts a drag, not a selection. Then click
   **Bifur (red)** while he is digging. The slot now reads his name, then
   `miner   T: change trade` in grey.
4. Press `T`. His line changes to `hauler   T: change trade`, and the tui roster says `Bifur hauler`.
5. Bifur stops digging. The mark he was on goes back to the queue, and **Dori (blue)** takes it later,
   not at once: she reaches it in FIFO order, behind her earlier marks (measured: 272 ticks, about 27 s at
   Normal). It is the same job, claimed by the other miner. Its dig starts from zero.
6. Press `T` again for `woodcutter` (he wanders: there is no cut job until 12.7), and again for `miner`.
7. Press `Esc`. The roster is back, with Bifur's current trade, in the gui and in the tui.

**Question 2:** does `T` feel right, and does the line change soon enough after the key?

**Allowed by your ruling (Task 0.2):** making every hauler a miner is accepted. Their haul jobs then
wait silently, and the roster is the only signal. You can see it by pressing `T` on Ori and Frar until
both read `miner`: dug stones stay where they fell.

Seat result (2026-10-05): AC11 PASSED, "works" (branch at `a5eb634`).
