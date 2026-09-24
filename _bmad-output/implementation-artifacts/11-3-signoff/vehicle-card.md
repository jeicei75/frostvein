# Story 11.3 vehicle card — Night Falls, Day Breaks

Run `simd` in WSL. Run every client command below in PowerShell from
`D:\Workspace\frostvein`. The launcher checks that its built client matches HEAD.

## Live cycle

Start a fresh daemon at tick 0 in WSL:

```text
simd 7451
```

In PowerShell:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

The unpinned client begins at 22:00 and follows the daemon tick. Press `+` once from Normal to
Fast; a full day then takes about eight minutes. `-` steps Fast → Normal → Paused, and Space
pauses or resumes. Check the moonlit night against
`10-8-signoff/approved-moonlit-camp-3479a43-a.png`, and the noon day against
`11-3-signoff/approved-day-bb893b2.png`. Keep the client open through dawn and dusk.

## Pinned looks

For each command here, **stop and restart** the WSL daemon as `simd 7451 --pause-at 120` before
launching. Every `--static-world` capture needs its own fresh daemon. The clock stays pinned even
while the daemon pauses at tick 120.

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--headless','--static-world','--lights-steady','--frames','160','--clock','22','--capture','D:\Workspace\frostvein\.bin\11-3-night.png')
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--headless','--static-world','--lights-steady','--frames','160','--clock','12','--capture','D:\Workspace\frostvein\.bin\11-3-noon.png')
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--headless','--static-world','--lights-steady','--frames','160','--clock','17.5','--capture','D:\Workspace\frostvein\.bin\11-3-dusk.png')
```

The night and noon captures should print `clock=22.00 (--clock)` and `clock=12.00 (--clock)`.
The measured devpod ground medians are 69 and 161. The capture band applies to both.

For the live noon look and AO judgement, start `simd 7451` afresh and launch:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--clock','12','--perf-log','D:\Workspace\frostvein\.bin\11-3-noon-window.csv')
```

Make the window fullscreen, press F7 to compare AO on and off, and read NFR6's frame time at
noon. Use F3 to mark the perf log, then read it with
`python3 scripts/bench/perf_summary.py D:\Workspace\frostvein\.bin\11-3-noon-window.csv`.
The devpod's software renderer cannot establish the seat's NFR6 result.

## Wolf's rulings to record

1. Does the 22:00 night still match the approved moonlit camp?
2. Does noon match candidate A's approved day frame? Is the day look accepted over the full cycle?
3. How does dusk at 17:30 read? It has **no warm sunset light**: the key keeps its day colour
   until it darkens at the horizon. Is that acceptable for now?
4. The haze brightens by day with ambient intensity, while torches, lanterns and campfire are
   unchanged. Keep that treatment?
5. At noon fullscreen, is AO with F7 worth keeping visually (#106)? What frame rate and frame
   times does NFR6 show with the effect on and off?

Record Wolf's answers and any follow-up decision in the story. AC13 itself is his vehicle sitting.
