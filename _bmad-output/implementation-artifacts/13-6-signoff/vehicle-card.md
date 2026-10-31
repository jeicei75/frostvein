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

2026-10-31, Wolf, in progress.

- **Fullscreen hides the `done` line.** Changing focus to WSL would disturb the fps being read, so
  the run's end is read from the HUD clock line instead: `speed paused` is the daemon's end-of-run
  pause. A HUD readout of the scenario state is #223.
- **Run 1, Wolf: "it started pretty ok with 30 FPS and FPS is not an issue but longer the sim run
  the dwarfs started to act - stop - act - stop -- so in the end all is slow but it's not GUI
  performance issue."** The daemon was a **debug** build (`./target/debug/simd …`, which printed
  `DEBUG BUILD: figures are not a measurement`). The run took 47 min (gui 06:16:07–07:03:12), about
  1.1 s per tick, and ended `done at tick 2500 state=3fc1461d491ee5d9 commands=1801/1801`, the
  bench's hash. So the act–stop is the debug daemon missing its 100 ms period, **not a reading of
  the sim at scale**. A release daemon plus a socket-draining client on the devpod ran the same
  scenario in 255 s, same hash. The gui was release (`b224da85`), so its fps reading stands.
- **Run 1 fps** (`perf_summary.py 13-6-perf-working.csv`, vsync on, RTX 4080 Laptop, 104,526
  frames over 2,824 s): steady state p50 25.79 ms (≈ 39 fps), p95 35.69, p99 47.98, p99.9 67.01,
  worst 313.03 ms. Edit frames (n=305) p50 43.41, p99 75.72 ms. Flat across the run (5-minute
  buckets p50 23.8–29.7 ms). **Misses NFR6's 60 fps sustained at working zoom.** Recorded, not
  fixed here.
- **Run 1 again, release daemon** (`13-6-perf-release.csv`, 36,414 frames over 884.5 s, F3 at
  856.3 s). The work ran 0–321.5 s, the last edit before a 542 s gap. From 863 s the world ran
  again (trees 927 → 921 by 870 s, after the F3), so the pooled `perf_summary.py` line is not the
  reading. **Running window only**, steady n=12,263: p50 25.23 ms (≈ 40 fps), p95 31.15, p99
  37.70 (≈ 27 fps), worst 300.46 ms. Edit frames (n=305) p50 38.85, p99 65.92 ms. That is the
  debug-daemon run's figure to within 0.6 ms at p50, so the debug daemon did not cost the gui its
  frames. **Working zoom misses NFR6's 60 fps sustained**, and costs about 2.2× full vista's
  frame (25.2 vs 11.3 ms p50).
- **Wolf: "faster now but still the same issue that dwarves act in pulses in the end."** With a
  release daemon this is the real thing: 2,500 ticks took about 321 s on the vehicle, ≈ 128 ms per
  tick against the 100 ms period. The devpod daemon-only run averaged 102 ms per tick. A tick past
  its period delivers its delta late, and the dwarves stall on their last cell until it arrives.
  AC 8's bench on this scenario: mean 33.6 ms, median p99 155.9 ms, max 267 ms, with the mean
  climbing late in the run. This is **NFR12's miss, seen at the seat for the first time**. NFR12
  gates the M4 close, not 13.6.
- **Run 2 fps, full vista** (release daemon, `13-6-perf-vista.csv`, 59,939 frames over 669.8 s,
  F3 at 665.4 s): the work ran 0–266 s, the last edit frame. The rest is the world paused at
  `done`, so the pooled `perf_summary.py` line (steady p50 10.96, p99 15.51, worst 297.89 ms) is
  over half paused. **Running window only**, steady frames n=22,938: p50 11.29 ms (≈ 89 fps),
  p95 12.78, p99 13.69 (≈ 73 fps), worst 297.89 ms. Edit frames (n=305) p50 20.12, p99 34.24 ms.
  **Meets NFR6's ≥ 30 fps at full vista.**
- **Ruled (Wolf): record both and file them.** Working-zoom fps miss: **#224**. Late-run pulse
  (NFR12 at the seat): **#225**.
- **Run 3, fresh game** (release daemon, no `--camera`): **boots onto the campfire of the 256²
  valley, and all four edges are reachable** (Wolf: "yes"). `13-6-fresh-game-fog-wall.png`
  (3840×1529, 5-dwarf crew, `elapsed 0d 00:26`, the overlay reads 63.84 fps): the fog volume's
  hard translucent wall cuts the right of the frame. That is **#222** on the vehicle GPU, and the
  one defect Wolf named at the sitting. No other sky, aurora or snow defect was reported.
