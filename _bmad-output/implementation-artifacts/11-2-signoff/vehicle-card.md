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

Questions for Wolf:

1. Does f/0.05 make the far valley soften while the camp remains the focus plane? If not, choose
   a new value only inside the measured 0.02–0.10 bracket and record it with the pair.
2. Does the density-0.015 vertical ramp dissolve naturally above the skyline, with no hard band,
   while the haze reads as air rather than a dimmer? If not, record ground median and chosen
   replacement before changing it.
3. Record p50 frame times for all effects, DoF off, and haze off; re-read NFR6's 60 fps bar.
