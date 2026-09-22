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

Live toggles at the seat: **F1 dof**, **F2 haze**, beside F10 fxaa / F11 ao / F12 bloom.
With the prepass fix, F11 no longer disturbs F1 or F2, and the three can be cycled in any order. The
readout names each one's state. (F1/F2 rather than F13/F14: a standard keyboard stops at F12 --
see issue #118, which carries the keymap rethink.)

**Before judging either effect, know this (fixed 2026-09-22).** Depth of field and haze both read
the camera's depth prepass but declare it nowhere; only ambient occlusion did. Turning **F11 (ao)**
off therefore removed the depth buffer out from under both, and only F11 put it back — cycling F1
never did. At the seat that read as a black screen and a green/red/blue overlay, and the readout
still said `F1 dof on` throughout. It is fixed: prepass ownership is now computed from the whole
effect set. **Judge dof and haze on a build carrying that fix**, and note that any earlier on/off
comparison made after an F11 press was measuring a broken state.

Questions for Wolf:

1. Does f/0.05 make the far valley soften while the camp remains the focus plane? If not, choose
   a new value only inside the measured 0.02–0.10 bracket and record it with the pair.
2. **THE OPEN ONE.** Does the density-0.015 vertical ramp dissolve naturally above the skyline,
   with no hard band, while the haze reads as air rather than a dimmer? Wolf's first reading was
   *"haze could be stronger.. cannot see the difference between on/off"* — but that may have been
   taken in the broken-prepass state above, so **re-compare F2 on/off on the fixed build first**.
   The instrument agrees it is faint either way: the haze moves the one window it reaches by 2
   levels out of 255. If it still reads flat, record the ground median and the chosen replacement
   before changing `FOG_DENSITY_FACTOR` — and expect the LL control to be re-baselined with it.
3. Record p50 frame times for all effects, DoF off, and haze off; re-read NFR6's 60 fps bar.
