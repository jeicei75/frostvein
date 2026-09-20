# 11.1b Task 5 — gingerspice vehicle card

This task is vehicle-bound. Do not run it in the devpod and do not infer FPS from headless
llvmpipe.

**Filed at `task-5b-…` deliberately.** An earlier commit in this story wrote 11.1b's card over
`task-5-vehicle-card.md`, which is **11.1a's** card for 11.1a's own still-unchecked Task 5 and is
referenced four times by that story. The spec chose a separate `-5b` name precisely to avoid that;
11.1a's card has been restored.

## One daemon per run. This is not a style note.

`--static-world` freezes the world at whatever tick it had reached when the client connected, so a
second client against a daemon that has been running through the first freezes a **later** world —
different dwarf positions, read as if they were the effect under test. Measured on `6140ca3`:

| recipe | camp median | p90 | mean | near-white |
| --- | ---: | ---: | ---: | ---: |
| fresh `simd` per capture (n=4) | 0 | 0 | 0.020 | **0.0070 pp** |
| one shared `simd` (n=5) | 1 | 2 | 0.867 | **0.3619 pp** |

The shared-daemon spread is 52x worse and fails the bar AC1 is measured against. The four runs
below are a comparison set, so each gets its own daemon, and **every run passes `--lights-steady`**
— emitter flicker runs on the client's wall clock and `--static-world` does not touch it, which
matters most for the one pair Wolf judges, since that pair is a bloom HALO comparison.

**`--headless` is load-bearing, and the first draft of this card omitted it.** `ingest.rs:605`
hardcodes `vsync: !args.headless` with no other way to disable vsync, so a windowed run is
`PresentMode::Fifo` and all four p50s clamp to the refresh interval in 16.67 ms steps — the
comparison would carry no information about any effect. Headless still writes the PNGs judged for
AC9. 11.1a's review made this same correction to its own card; this one repeated the mistake.

Run through `scripts/launch-gui.ps1`, which forwards the port and `--assets` itself and refuses a
binary that does not match the checkout. Pass flags as `-GuiArgs @(...)`: the `--` form in the
script's own docstring cannot work, because the first positional parameter swallows it (issue #81).
The launcher defaults to `-Port 7451` and a bare `simd` lands on the same port, so neither needs a
port argument — do not mix that with the `7466` the first draft of this card used.

```powershell
# repeat for each of the four runs, restarting simd.exe each time
simd.exe
.\scripts\launch-gui.ps1 -GuiArgs @('--headless','--static-world','--lights-steady','--frames','160','--perf-log','effects-on.csv','--capture','effects-on.png')
# then stop simd.exe, start it again, and take the next one:
.\scripts\launch-gui.ps1 -GuiArgs @('--headless','--static-world','--lights-steady','--frames','160','--fx-off','fxaa','--perf-log','fxaa-off.csv','--capture','fxaa-off.png')
.\scripts\launch-gui.ps1 -GuiArgs @('--headless','--static-world','--lights-steady','--frames','160','--fx-off','ao','--perf-log','ao-off.csv','--capture','ao-off.png')
.\scripts\launch-gui.ps1 -GuiArgs @('--headless','--static-world','--lights-steady','--frames','160','--fx-off','bloom','--perf-log','bloom-off.csv','--capture','bloom-off.png')
```

Each run prints `sim PAUSED (--static-world) at tick N`. **Check that N is the same for all four.**
If it is not, the frames are not comparable and the daemon was not restarted.

## AC8 — the perf read

Wolf reads and records the p50 frame time from all four CSVs, then re-reads NFR6's 60 fps bar
(16.67 ms). If any individual effect costs that bar, record the pair and ask Wolf to rule; this
story does not tune an effect.

**Say which cost the AO column measures.** `--fx-off ao` now removes `DepthPrepass` and
`NormalPrepass` along with the SSAO component, so the AO-off run is genuinely without AO's cost.
Before the 2026-09-19 review it removed only the SSAO component and left both full-scene prepasses
running, which would have under-reported AO's true cost by exactly its expensive half.

## AC9 — the frame pair

Wolf judges `effects-on.png` against `bloom-off.png` at the same boot framing, with the other two
captures and all four CSVs filed beside them.

**Two things to know before judging, both measured, neither a reason to change anything:**

- **AO will not read as creases darkening.** Issue #106, ruled 2026-09-18: SSAO attenuates only the
  ambient term, and this scene tunes ambient deliberately small (1,500 against a 7,000 directional
  and 7,000,000 lm point lights), so AO's strongest effect anywhere in frame is about 2% of base
  luminance and is broadly uniform rather than concentrated at contacts. AO is accepted here as
  WIRED AND PROVED; its look judgement was deferred to 11.3, where the day/night cycle raises
  ambient and gives it something to attenuate.
- **The near-white ceiling may trip.** Headless headroom is ~0.18 pp near-white and ~0.03 pp
  blown-pool, and the recorded delivery-GPU penalty is 0.4–0.6 pp WORSE near-white than lavapipe —
  larger than the whole headroom. AC5 defers the ceiling judgement to this sitting by Wolf's
  2026-09-18 ruling; a trip here is expected information, not a regression to fix on the spot.

**RUN 2026-09-20 on build `5cd523c`.** The sitting's figures, and what they do and do not
establish, are in `task-5b-vehicle-evidence.md`. AC8 met, AC9 signed off, AC5's deferred ceiling
judgement closed — the ceiling held with +0.4449 pp of headroom, so the trip this card predicted
did not happen.

**No vehicle observation or FPS figure was made in this card.**
