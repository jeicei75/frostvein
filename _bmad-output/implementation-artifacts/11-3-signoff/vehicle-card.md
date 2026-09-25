# Story 11.3 vehicle card — Night Falls, Day Breaks

Run `simd` in WSL. Run every client command below in PowerShell from
`D:\Workspace\frostvein`. The launcher checks that its built client matches HEAD.

## Sitting 2 — after the 2026-09-25 rulings (run this one)

Sitting 1's answers are recorded in the story. This build changes the four things it ruled on:
- **The moon is 750 lux** (was 7,000). The new night is the approved night; there is no older frame
  to match it against.
- **Nothing in the sky casts a shadow.** The fast "orbiting" shadow was a star.
- **Speeds:** `+` now walks Normal → Fast → Fast2x → Fast4x. A day takes 40 / 8 / 4 / ~2 min.
- **Shadow crawl is accepted** and filed as #126, so no check is needed.

Run the live cycle below at **Fast4x** (press `+` three times), then the three pinned captures
further down. On this build the noon capture's 101 warning still stands, and 22:00 must exit 0.

Questions for Wolf:
1. Does the 750-lux night read as night, with moonlight rather than a sun?
2. Do dusk and dawn now read as a transition rather than a dip? (Warm sunset light is still not
   modelled.)
3. Is Fast4x usable? Does the world keep moving smoothly, with no freeze?

The rest of this card is sitting 1, kept for its record.

## Live cycle

Start a fresh daemon at tick 0 in WSL:

```text
simd 7451
```

In PowerShell:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

The unpinned client follows the daemon tick, which is 22:00 only at tick 0. Every minute the
daemon runs before the first frame moves the clock 0.6 h and the moon 9° of azimuth, so **this
run is for the cycle, not for judging the night** — judge the night from the pinned 22:00
capture below. Press `+` once from Normal to Fast; a full day then takes about eight minutes.
`-` steps Fast → Normal → Paused, and Space pauses or resumes. Keep the client open through
dawn and dusk.

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

Judge the night from `11-3-night.png` against `10-8-signoff/approved-moonlit-camp-3479a43-a.png`,
and the noon from `11-3-noon.png` against `11-3-signoff/approved-day-bb893b2.png`.

**The noon capture may exit 101 on this GPU.** The devpod's noon near-white is 0.8519% against a
0.9461% ceiling, which is only 0.094 pp of headroom. This GPU has read 0.4–0.6 pp higher on
near-white in the bright tail than the devpod. A 101 at noon with the range-check line blaming
near-white is that venue offset, not a regression. The PNG may still be written; if it is not,
judge noon from the live run below. A 101 at 22:00 **is** a regression — report it.

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
   **The sun's ramp changed after your pick.** You picked A on a 10° horizon ramp; `7ad8b32`
   widened the sun's to 25° for AC4, and you ruled to keep it. The sun is now below full
   strength before about 08:35 and after about 15:25 (with 10°: 06:58 and 17:02). At 17:30 the
   key is about 1,350 lux instead of about 6,400 (−79%), so `candidate-A-provisional-bb893b2-h17.5.png`
   is brighter than what this build renders. Judge dawn and dusk live, in the cycle run.
4. The haze brightens by day with ambient intensity, while torches, lanterns and campfire are
   unchanged. Keep that treatment?
5. At noon fullscreen, is AO with F7 worth keeping visually (#106)? What frame rate and frame
   times does NFR6 show with the effect on and off?

Record Wolf's answers and any follow-up decision in the story. AC13 itself is his vehicle sitting.
