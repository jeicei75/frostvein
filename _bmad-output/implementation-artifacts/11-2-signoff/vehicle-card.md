# Story 11.2 vehicle card — The Miniature

Launch the way the seat always does -- `simd` in WSL, the client through `launch-gui.ps1` from the
Windows checkout in PowerShell (see the README's *At the vehicle* section). The devpod has lavapipe
only, so it cannot establish NFR6's 60 fps bar; these runs are vehicle-only.

**The seat itself** (live toggles, the eye check for AC14):

```powershell
# WSL
simd 7451
# PowerShell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

**AC13 -- frame times.** Boot framing, one run per configuration, each with a FRESH `simd 7451
--pause-at 120` (restart it before every run), output under `.bin\` so the checkout stays clean:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--headless','--static-world','--lights-steady','--frames','160','--capture','D:\Workspace\frostvein\.bin\vehicle-all.png','--perf-log','D:\Workspace\frostvein\.bin\vehicle-all.csv')
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--headless','--static-world','--lights-steady','--frames','160','--fx-off','dof','--capture','D:\Workspace\frostvein\.bin\vehicle-dof-off.png','--perf-log','D:\Workspace\frostvein\.bin\vehicle-dof-off.csv')
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--headless','--static-world','--lights-steady','--frames','160','--fx-off','haze','--capture','D:\Workspace\frostvein\.bin\vehicle-haze-off.png','--perf-log','D:\Workspace\frostvein\.bin\vehicle-haze-off.csv')
```

Read each CSV with `python3 scripts/bench/perf_summary.py <csv>` and open the all-effects frame
beside both controls. **A headless frame time is not the seat's**: to judge the ~60 fps Wolf saw at
the seat, also take a WINDOWED `--perf-log` run (the seat launch plus `'--perf-log','...csv'`, pressing F4
mid-run for a haze-off stretch and F3 to mark it). Compare like with like: the windowed run carries
the always-on fps overlay and its frame-time graph, which are not free, while the headless runs
above do not (`--capture` forces the overlay off). Do not score the overlay's cost as 11.2's.

`--pause-at 120` is what lets a hand-started client still freeze on tick 120; without it the
capture is refused (`asked to freeze at tick 120 but the daemon stopped at ...`). `--frames` does
not need scaling on the 4080 -- the capture waits for its ticks.

Live toggles at the seat, in key order: **F4 haze, F5 dof, F6 bloom, F7 ao**, then
**F8 sun, F9 ambient, F10 campfire, F11 torches, F12 lanterns**. The readout prints them in that
same order. See the table below for why the row was rebuilt, and issue #118 for the wider rethink.

**THE KEYS MOVED (2026-09-22). `F1` and `F2` were never ours.** `DefaultPlugins` pulls in
`bevy_dev_tools::render_debug::RenderDebugOverlayPlugin`, which hardcodes **F1** to cycle a
depth/normal debug overlay and **F2** to cycle that overlay's opacity. 11.2 had put dof and haze on
exactly those keys, so every press drove BOTH: the readout said `F1 dof on` while Bevy blacked the
frame out (depth overlay), painted it green/pink/blue (normal overlay), or left it at half opacity.
Nothing in the test suite could see it -- the tests build on `MinimalPlugins`, which has no such
plugin. The seat recordings are what caught it.

Now:

| key | control |
| --- | --- |
| F1 / F2 | **Bevy's** debug overlay: cycle depth/normal, cycle opacity — reserved, not ours |
| F3 | mark a frame in the perf log |
| **F4 F5 F6 F7** | **effects**: haze, depth of field, bloom, ambient occlusion — widest-acting first |
| **F8 F9 F10 F11 F12** | **lights**: sun, ambient, campfire, torches, lanterns — biggest reach first |
| *(none)* | fps overlay — always on; `--capture` forces it off |
| *(none)* | fxaa — `--fx-off fxaa` only |

A separate prepass defect was also fixed: dof and haze read the depth prepass but declare it
nowhere, and only AO did, so turning AO off (then on F11, now **F7**) used to pull the depth
buffer out from under both. It is now safe to cycle in any order.

Questions for Wolf:

1. Does f/0.05 make the far valley soften while the camp remains the focus plane? If not, choose
   a new value only inside the measured 0.02–0.10 bracket and record it with the pair.
2. **Haze -- RESOLVED 2026-09-23, confirmed at the seat.** It was never the GPU: **F4 could not
   turn the haze off live on any machine.** Bevy 0.19 only ever inserts `VolumetricFog` on the
   render-world camera and clears it only when no light carries `VolumetricLight`, so removing it
   from the camera left the fog drawing. (Code review correction: that cleanup skips our camera,
   because its `VolumetricFog` is already gone. What turns the haze off is the same branch
   stripping every `FogVolume`. Full account in `sync_haze_light`'s doc comment.) Both halves of the earlier seat recording had haze ON, which
   is why they measured -0.07 levels apart; a headless `--fx-off haze` pair on the 4080 showed the
   real difference. Fixed in `cf5e008` (F4 now moves the sun's `VolumetricLight` too). Remaining
   judgement for AC14 is the strength -- `FOG_DENSITY_FACTOR` and the ramp -- against the art.
   Wolf's seat note: haze + the sun's volumetric light is the heaviest effect; read that in AC13.

3. **AC13 -- ANSWERED 2026-09-23 at the seat, fullscreen:** haze on ~60 fps, haze off ~140 fps.
   Wolf: accepted, no optimisation now. The headless runs above could not rank the effects (frame
   time drifted ~35% within each run and the runs differed in length); keep them for the frames,
   not for fps.
