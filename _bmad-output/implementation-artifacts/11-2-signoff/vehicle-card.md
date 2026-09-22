# Story 11.2 vehicle card — The Miniature

Run each command at boot framing with a fresh daemon, then collect the `--perf-log` CSV p50 on the
vehicle. The devpod has lavapipe only, so it cannot establish NFR6's 60 fps bar.

```bash
./target/release/simd 7501 &
./target/release/gui 7501 --headless --static-world --lights-steady --subdiv 4 --frames 160 \
  --capture _bmad-output/implementation-artifacts/11-2-signoff/vehicle-all.png \
  --perf-log _bmad-output/implementation-artifacts/11-2-signoff/vehicle-all.csv
```

Repeat with `--fx-off dof` and `--fx-off haze`, using a new daemon and `vehicle-dof-off` /
`vehicle-haze-off` filenames. Open the all-effects frame beside both controls.

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
| F3 | fps overlay |
| **F4 F5 F6 F7** | **effects**: haze, depth of field, bloom, ambient occlusion — widest-acting first |
| **F8 F9 F10 F11 F12** | **lights**: sun, ambient, campfire, torches, lanterns — biggest reach first |
| M | mark a frame in the perf log (off the F row: it is full) |
| *(none)* | fxaa — `--fx-off fxaa` only |

A separate prepass defect was also fixed: dof and haze read the depth prepass but declare it
nowhere, and only AO did, so turning **F11** off used to pull the depth buffer out from under both.
F11 is now safe to cycle in any order.

Questions for Wolf:

1. Does f/0.05 make the far valley soften while the camp remains the focus plane? If not, choose
   a new value only inside the measured 0.02–0.10 bracket and record it with the pair.
2. **THE OPEN ONE -- and it is now a DEFECT, not a tuning question.** Measured inside Wolf's own
   seat recording (static camera, 9 frames averaged per state, overlay disabled at the time), the
   haze toggles from on to off and the frame moves **-0.07 levels** -- smaller than the
   frame-to-frame noise in either state, and flat across every row of the frame. On the devpod
   (lavapipe) the same toggle moves the whole frame **+3.96 levels** and lifts the mid-distance by
   about **+10**. **The volumetric fog does nothing at all on the vehicle's GPU.** Do not tune
   `FOG_DENSITY_FACTOR` against this -- a stronger value multiplied by zero is still zero. What is
   needed first is why the fog renders here and not there.

3. Record p50 frame times for all effects, DoF off, and haze off; re-read NFR6's 60 fps bar.
