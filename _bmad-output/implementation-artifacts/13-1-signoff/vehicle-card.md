# 13.1 seat card: one scripted run, recorded with no keys pressed (AC 7)

Push first (`scripts/push.sh --fast`) so the seat builds this branch's HEAD. One sitting.

## Run it

```bash
# WSL, from the repo (a FRESH release daemon; it holds tick 0 until the gui connects)
cargo build --release -p simd
target/release/simd 7451 --scenario scenarios/plan.json
```

A release build: a debug `simd` misses fast4x's 5 ms period, so the run stretches while the hash still
matches (it prints `DEBUG BUILD: figures are not a measurement` at the hold if you got the wrong one).

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-1-plan.csv')
```

Then do nothing. Move the camera if you like: the camera doesn't change the sim, but don't press `+`/`-` and
don't designate. After about 50 s at fast4x, the WSL terminal prints:

```
scenario plan: done at tick 10006 state=2479b66a35ac887e commands=50/50
```

and the world stops: the gui's readout holds at `elapsed 0d 10:00` (1,000 ticks per hour). That stop IS the
end of the run, not a fault. **Press F3 in the gui as soon as the `done` line prints.** F3 marks the perf log
and leaves the sim alone. The world stays paused after `done`, and the CSV keeps recording those paused
frames until you quit, so the mark is the only way to tell where the run ended. Then quit the gui, and
run this in WSL:

```bash
python3 scripts/bench/perf_summary.py /mnt/d/Workspace/frostvein/.bin/13-1-plan.csv
```

If the gui exits before `done` (for example, evicted at connect), **restart both**: Ctrl-C simd and
start it fresh, then relaunch the gui. The hold happens only once, for the first client. A second gui
joins a run that is already under way, and its CSV misses the start.

## Questions

1. Did the run go from the hold to `done` without a key pressed (F3 at `done` aside)?
2. Is the `done` line's state `2479b66a35ac887e`? That is what `simd --bench scenarios/plan.json` prints on the devpod.
   A different hash means a key or a drag changed the sim, and the line then also says `(clients sent N commands)`.
3. Does `perf_summary.py` read the whole run? Its `MARKED` line is the F3 frame. Is that frame about
   50 s after the first frame, and does everything before it look like the run you watched? The overall
   figures also include the paused tail after the mark.

---

## The sitting (Wolf, 2026-10-23, build `2fb1f17`): AC 7 MET

simd ran in WSL from the forge checkout on the host; the gui ran from the Windows checkout. Only the CSV
crosses the two, and it is read once after the run, so it doesn't matter that `/mnt/d` access is slow.
The evidence is `13-1-plan.csv` in this folder.

1. **No manual input: yes.** `done at tick 10006 state=2479b66a35ac887e commands=50/50`, with no
   `(clients sent N commands)` suffix, so the gui sent the sim nothing.
2. **Hash: yes.** `2479b66a35ac887e` is the devpod bench's hash for `plan.json`.
3. **The CSV covers the run: yes.** It has 17,018 frames over 261.9 s from frame 0 (`build=2fb1f17`).
   The first edit frame is at 0.3 s and the last at 43.7 s (48 edit frames; terrain 1888->1896, trees
   259->256), which fits a ~50 s fast4x run. **The F3 mark is LATE**, at 164.6 s, plus two repeat presses
   at 166.4 s and 170.0 s. Wolf asked whether the stop at elapsed 10:00 was a fault before pressing it,
   so the mark is not the end of the run; the last edit frame bounds the run instead. The card now says
   that the stop is the end.

The frame times are NOT a performance statement: `vsync=on`, so steady-state p50 16.16 ms is the 60 Hz
display interval. AC 7 asks for an unattended run that the CSV covers, not a frame budget.
