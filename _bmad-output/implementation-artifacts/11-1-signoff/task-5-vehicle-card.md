# 11.1a Task 5 — gingerspice vehicle card

This task is vehicle-bound. Do not run it in the devpod and do not infer FPS from headless
llvmpipe. **Updated at the 2026-09-19 code review:** the exposure ruling and two findings added
items here, and the AC10 recipe below was corrected — the version before it could not measure what
AC10 asks.

11.1b has its own card at `task-5b-vehicle-card.md`. Its **one daemon per run** rule applies to
every recipe here too: `--static-world` freezes the world at the tick reached AT CONNECT, so a
second client against a still-running daemon freezes a later world and reads the difference as the
effect under test.

## 1. AC10 — the FXAA frame-time cost

```text
gui.exe <daemon-port> --headless --static-world --frames 160 --perf-log fxaa-on.csv \
  --capture fxaa-on.png
gui.exe <daemon-port> --headless --static-world --frames 160 --fx-off fxaa --perf-log fxaa-off.csv \
  --capture fxaa-off.png
```

**`--headless` is load-bearing and is the correction.** `ingest.rs` hardcodes
`vsync: !args.headless` and there is no flag to disable vsync otherwise, so a windowed run on the
vehicle is `PresentMode::Fifo`: both p50s clamp to the refresh interval, quantise in 16.67 ms steps,
and the FXAA comparison carries no information about FXAA. `perf.rs:59`'s own doc comment names this
hazard. Headless still renders on the real GPU — it removes the swapchain cap, not the rendering.
Raised by the Feature Auditor.

Wolf reads the p50 frame time from each CSV, re-reads NFR6's 60 fps bar (16.67 ms), and records both
numbers. If FXAA costs the bar, record the pair and ask Wolf to rule; do not tune effects here.

## 2. AC11 exposure clause — the ruling needs a frame

The exposure is **10.5 EV100**, ruled at the code review. It was ruled from ground-median numbers
measured on **lavapipe**, which is venue-sited and has mispredicted the delivery GPU twice. Wolf
judges the frame and says whether 10.5 is the exposure he wants 11.1b's effects judged under.

Context he may want: 10.5 is the darkest value the frame survives. `capture.rs`'s ground-median
floor was ruled 70 -> 55 in the same sitting to reach it. On 11.1a alone the binding path is
`--fx-off fxaa` at 58 against that 55, and one step darker (10.75) fails. With 11.1b's stack on top
the frame sits **higher**, not lower — Bloom's halo raises the median to 69 all-on — so the margin
is more comfortable in the combined build than in 11.1a alone. `EV100_OVERCAST` (12.0) is
unreachable at any floor: that frame reads ~32, darker than a broken `--lights-off ambient` one.

AC11's first clause (edges) is **closed** — Wolf, 2026-09-18, build `d3ecdff`, "edges are fine". Its
original wording asked for a comparison against an MSAA-4x control; that control cannot be built
from this branch (`Msaa::Off` is unconditional, no flag restores it), so the AC was amended to say
what was actually judged rather than leave an unfalsifiable clause open.

## 3. AC4 on-screen clause — NOT named vehicle-bound by the story, but it is

Press **F10** at the seat and confirm the readout line changes beside the F5–F9 lights.

No rendered frame anywhere contains the readout — measured at the review by scanning a real
1280x720 headless capture for the readout's colour `Color::srgb(0.86, 0.91, 1.0)` ≈
`(219, 232, 255)`: **zero pixels**, because neither readout renders headless. The string is proven
only as a `Text` component value inside a `MinimalPlugins` test app. Note F10 is conventionally a
menu-activation key on Windows — a small, unretired risk on the delivery platform. Raised by the
Feature Auditor.

---

File the frame pair and the two CSVs against this card when the sitting happens.
