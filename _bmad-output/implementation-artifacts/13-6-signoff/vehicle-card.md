# 13.6 seat card: 200 dwarves on 256² (AC 10, NFR13's first reading)

The sim's real work is proven headless (`the_two_hundred_on_256_scenario_does_real_work`, M2-26). This
sitting reads the **gui frame rate** with 200 dwarves on the 256² valley, against NFR6's bars: **60 fps
sustained at working zoom, ≥ 30 fps at full vista**. A miss is measured and ruled, not fixed in 13.6
(#179, the 20–180 fps swing, is the known defect it meets first). It also checks that the fresh game
boots onto the camp of a 256² valley (Q5 a).

Pull `story-13-6-two-hundred-on-256` and rebuild `gui.exe` at its HEAD before you start. The launcher
refuses a stale one.

## Run 1: working zoom

```bash
# WSL, from the repo. Release: a debug daemon misses the tick period at this scale.
cargo build --release -p simd
target/release/simd 7451 --scenario scenarios/two-hundred-on-256.json
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-6-perf-working.csv','--camera','0.7,0.45,45,127,128,7')
```

The run holds tick 0 until the gui connects, then runs 2,500 ticks at Normal speed. That is about
4 min if every tick fit in its 100 ms period, but the sim's mean is ~36 ms, rising past 100 ms
late in the run, so expect longer. Press **F3** when WSL prints the `done` line, then close the gui.

## Run 2: full vista

Restart the daemon (the hold happens only once), then:

```bash
target/release/simd 7451 --scenario scenarios/two-hundred-on-256.json
```

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-6-perf-vista.csv','--camera','0.7,0.45,500,127,128,7')
```

F3 at `done`, then close.

## Run 3: the fresh game

```bash
target/release/simd 7451
```

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

No `--camera`: the gui should open on the campfire of the 256² valley. Pan to the edges.

## Questions

1. **fps at each framing:** paste both logs' `python3 scripts/bench/perf_summary.py <csv>` lines (or
   send the CSVs). Do they meet 60 sustained at working zoom and ≥ 30 at full vista?
2. **Do the camp and the work read on 256²?** Can you see the dig and the pile from the working zoom?
   At full vista, do the sky, fog, aurora and snow look wrong anywhere? They are still centred on
   the old 128² world (`atmosphere.rs`). The fog volume's wall is already #222. Anything else gets
   an issue, not a fix in 13.6.
3. **Does the fresh game boot onto the camp, and can you pan to all four edges of the 256² valley?**

## Checked headless first (devpod, lavapipe, `55202de2`)

`target/release/gui <port> --headless --frames 160 --capture … --subdiv 4 [--camera …]`, one fresh
release daemon per capture:

- **Working zoom** (`0.7,0.45,45,127,128,7`): the camp, the dig face, the green pile grid and the
  crew are all in frame. The rig reports `focus=127,128,7`, so y=128 survives the clamp (13.6 Task 7).
  The capture's range check trips on near-white 2.83% against the 0.95% boot ceiling. That band is
  calibrated for the boot framing only and is expected to trip on a `--camera` view (`capture.rs`).
- **Full vista** (`0.7,0.45,500,127,128,7`): the whole 256² valley, with the camp lit near its centre.
  The warm-pixel floor trips, as expected for a vista. **A hard-edged translucent box sits over
  one quadrant**: the fog volume is still the 128² box. Filed as **#222**.
- **Fresh game** (no `--camera`): boots onto the campfire of the 256² valley. The same fog wall
  cuts the right third of the frame (#222).

These are lavapipe frames, so they say nothing about fps. The seat does.

## Sitting

(filled in at the sitting)
