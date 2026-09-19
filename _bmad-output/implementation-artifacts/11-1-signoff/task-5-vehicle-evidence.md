# 11.1a Task 5 — gingerspice vehicle evidence, 2026-09-19

Run by Wolf on the delivery vehicle against build **`20b9005`** (`main`, both 11.1 stories merged).
Both runs exited 0 and the captured frames were judged correct, so the capture range checks —
ground median, warm pixels, black-frame, uniform-frame — all passed on the measured frames.

Recipe, as corrected at the code review (`task-5-vehicle-card.md` §1). `--headless` is load-bearing:
`ingest.rs` hardcodes `vsync: !args.headless` with no other way to disable vsync, so a windowed run
is `PresentMode::Fifo` and both p50s would clamp to the refresh interval, quantised in 16.67 ms
steps — the comparison would have carried no information about FXAA. The preamble confirms it took:
`vsync=off`.

```text
gui.exe 7466 --headless --static-world --frames 160 --perf-log fxaa-on.csv  --capture fxaa-on.png
gui.exe 7466 --headless --static-world --frames 160 --fx-off fxaa --perf-log fxaa-off.csv --capture fxaa-off.png
```

Preamble on both: `build=20b9005 assets=disk subdiv=4 vsync=off terrain=1888 trees=259 dwarves=5`.
The build stamp is clean — no `-dirty`, and it names the merged tip, so the binary is the code.

## AC10 — MET. FXAA does not cost the 60 fps bar.

Steady-state frame time, `scripts/bench/perf_summary.py`:

| percentile | FXAA on | `--fx-off fxaa` | delta |
| --- | ---: | ---: | ---: |
| **p50** | **3.47 ms** | **3.55 ms** | **−0.08** |
| p95 | 4.80 | 4.87 | −0.07 |
| p99 | 36.05 | 37.19 | −1.14 |
| p99.9 | 71.77 | 84.80 | −13.03 |
| worst | 71.77 | 84.80 | −13.03 |

**NFR6's bar is 16.67 ms and the worse of the two p50s is 3.55 ms — 4.7x of headroom.** Nothing is
owed to a ruling: FXAA does not cost the bar, so the story tunes nothing.

**Why "FXAA is free" is a sound reading, and it is the SIGN that carries it, not the size.** FXAA is
faster on *every* percentile. A post-process pass cannot make rendering quicker, so the measured
difference is not FXAA's cost at all — it is run-to-run variance, and FXAA's true cost is below this
setup's measurement floor. That argument does not need a noise floor to stand, which matters because
one run per condition IS NOT a floor (this project's own rule, issue #98). **No cost figure is
quoted here, only "below the noise", and the two runs do not license a stronger claim.**

## What these runs do NOT establish, recorded so nobody reads more into them

- **The tail is one or two frames.** 167 measured frames over 0.7 s makes p99 about two frames and
  p99.9 a single one. The 71.77 / 84.80 ms figures and the one `>=50ms` hitch in each run are almost
  certainly start-up: the run ends before the scene has settled in wall-clock terms. They are not
  evidence of a stutter problem, and they are not evidence against one either.
- **Hitch counts are equal** (7 at `>=2x median` in both runs), which is consistent with the hitches
  being start-up rather than anything FXAA does.
- **`edit frames: none recorded`** — nothing was dug during these runs, so this says nothing about
  the re-mesh cost that 10.6 measured. That remains the expensive frame in this game.
- **Content counters are spawned-entity counts** and are blind to the camera (issue #83), so they
  cannot by themselves prove the terrain was rasterised. What rules that out here is that both
  captures exited 0 — a frame that failed the ground-median or warm-pixel floors would have exited
  101 — and Wolf judged the frames correct. The recorded scar this guards against is a ~140 fps
  reading that survived a 39% triangle cut because the terrain was never drawn.

## AC11 — CLOSED, both clauses

- **Edges** — closed 2026-09-18, build `d3ecdff`: *"edges are fine"*.
- **Exposure** — **closed 2026-09-19, build `20b9005`**: Wolf judged `fxaa-on.png` and confirmed
  **10.5 EV100** is the exposure he wants 11.1b's effects judged under.

That closes the clause the code review had to leave open. 10.5 was RULED from ground-median numbers
measured on lavapipe — a venue that is sited and has mispredicted the delivery GPU twice — so until
this sitting the value was chosen but unseen. It has now been seen, on the delivery GPU, at the
shipped build.

## Still open

**AC4's on-screen clause.** It needs a WINDOWED run: neither readout renders headless, and a scan of
a real 1280x720 headless capture for the readout's colour `(219, 232, 255)` found zero pixels. The
string is still proven only as a `Text` component value inside a `MinimalPlugins` test app, and
nobody has pressed F10 on a machine. F10 is conventionally a menu-activation key on Windows, which
is an unretired risk on this platform.

## Found while filing this: the exposure ruling has eaten a third guard's margin (#111)

The full gate flaked twice in four runs on `bloom_lifts_the_camp_halo_without_brightening_open_snow`
while passing alone and 8/8 under the gate's own `--ignored` settings. It is not load or timing.

Its near-white clause requires `off_near_white - on_near_white >= 0.10` pp. Calibrated at 9.7 EV100
the fall was **0.32 pp against a same-build floor of 0.019**. At the ruled 10.5 EV100 near-white
roughly halved (5.17 % -> 3.53 %) and the bloom-vs-no-bloom difference halved with it: measured
**0.1137 pp and 0.1679 pp** on two passing runs, so the worse one clears the bar by **0.0137 pp**
against a noise floor documented as 0.019. The margin is smaller than the noise, which is exactly
why it trips by luck.

**This is the THIRD guard the exposure ruling has taken headroom from**, and the first where the
margin went under the noise. The other two still pass and are recorded: `ALL_OFF_DROP_FLOOR`
headroom 21.707 -> 8.707, and `GROUND_LUMINANCE_FLOOR`'s binding path (`--fx-off fxaa`) at 58
against 55. None of the three was lowered to fit; the frame moved toward them.

Filed as **#111** with the measurements and the two options worth measuring — re-derive the bar from
a floor measured AT 10.5, or switch to a ratio, since an absolute pp bar will keep re-breaking now
that exposure is a ruled variable that moves again in 11.2 and 11.3. It is deliberately NOT patched
here: lowering the bar until runs pass is the move 10.8's rule forbids, and the noise floor at 10.5
is unmeasured.
