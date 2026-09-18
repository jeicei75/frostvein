# 11.1b Task 5 — gingerspice vehicle card

This task is vehicle-bound. Do not run it in the devpod and do not infer FPS from headless
llvmpipe. At the gingerspice sitting, start the daemon and keep the same boot framing, window
resolution, and capture settings for every run:

```text
simd.exe 7466
gui.exe 7466 --static-world --frames 160 --perf-log effects-on.csv \
  --capture effects-on.png
gui.exe 7466 --static-world --frames 160 --fx-off fxaa --perf-log fxaa-off.csv \
  --capture fxaa-off.png
gui.exe 7466 --static-world --frames 160 --fx-off ao --perf-log ao-off.csv \
  --capture ao-off.png
gui.exe 7466 --static-world --frames 160 --fx-off bloom --perf-log bloom-off.csv \
  --capture bloom-off.png
```

Wolf reads and records the p50 frame time from all four CSVs, then re-reads NFR6's 60 fps bar
(16.67 ms). If any individual effect costs that bar, record the all-on/off pair and ask Wolf to
rule; this story does not tune an effect.

Wolf judges `effects-on.png` against `bloom-off.png` at the same boot framing, with the other
three captures and CSVs filed beside them. The pair isolates the new emitter halo under the chosen
exposure; the complete set also lets Wolf judge the FXAA and AO seats. **No vehicle observation or
FPS figure has been made here.**
