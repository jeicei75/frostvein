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

and the world stops. **Press F3 in the gui as soon as the `done` line prints.** F3 marks the perf log
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
