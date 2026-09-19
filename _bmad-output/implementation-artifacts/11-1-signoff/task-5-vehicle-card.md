# 11.1a Task 5 — gingerspice vehicle card

This task is vehicle-bound. Do not run it in the devpod and do not infer FPS from headless
llvmpipe. At the gingerspice sitting, start the daemon, then take these two boot-framing runs
with the same window resolution and capture settings:

```text
gui.exe <daemon-port> --static-world --frames 160 --perf-log fxaa-on.csv \
  --capture fxaa-on.png
gui.exe <daemon-port> --static-world --frames 160 --fx-off fxaa --perf-log fxaa-off.csv \
  --capture fxaa-off.png
```

Wolf reads the p50 frame time from each CSV, re-reads NFR6's 60 fps bar (16.67 ms), and records
both numbers. If FXAA costs the bar, record the pair and ask Wolf to rule; do not tune effects
in this story.

Wolf judges `fxaa-on.png` against `fxaa-off.png` at the same boot framing: edges must read
cleaner than the MSAA-4x control, and exposure must be the frame he wants 11.1b's effects judged
under. File that frame pair with the two CSVs. **No vehicle observation has been made here.**
