# 13.1 seat card: one scripted run, recorded with no keys pressed (AC 7)

Push first (`scripts/push.sh --fast`) so the seat builds this branch's HEAD. One sitting.

## Run it

```bash
# WSL, from the repo (a FRESH daemon; it holds tick 0 until the gui connects)
simd 7451 --scenario scenarios/plan.json
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-1-plan.csv')
```

Then do nothing. Move the camera if you like: the camera doesn't change the sim, but don't press `+`/`-` and
don't designate. After about 50 s at fast4x, the WSL terminal prints:

```
scenario plan: done at tick 10006 state=2479b66a35ac887e commands=50/50
```

and the world stops. Quit the gui, then (WSL):

```bash
python3 scripts/bench/perf_summary.py /mnt/d/Workspace/frostvein/.bin/13-1-plan.csv
```

## Questions

1. Did the run go from the hold to `done` without a key pressed?
2. Is the `done` line's state `2479b66a35ac887e`? That is what `simd --bench scenarios/plan.json` prints on the devpod.
   A different hash means a key or a drag changed the sim, and the line then also says `(clients sent N commands)`.
3. Does `perf_summary.py` read the whole run (frames from the connect to the stop, about 50 s of them)?
