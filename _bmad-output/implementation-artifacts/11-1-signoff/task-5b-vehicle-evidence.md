# 11.1b Task 5 — gingerspice vehicle evidence, 2026-09-20

Run by Wolf on the delivery vehicle against build **`5cd523c`** (`main`, 11.1b's work merged).
Four runs, **one fresh `simd.exe` each**. Wolf confirmed at the sitting that all four printed the
same `--static-world` pause tick and all four exited 0.

Recipe, via `scripts/launch-gui.ps1 -GuiArgs @(...)` — the launcher forwards the port and
`--assets` itself and refuses a binary that does not match the checkout, so the stamp check is not
a manual step. The `--` form in its own docstring cannot work (issue #81); `-GuiArgs` is the way in.

```powershell
simd.exe          # restarted before EACH run -- see the card's §1 table for why
.\scripts\launch-gui.ps1 -GuiArgs @('--headless','--static-world','--lights-steady','--frames','160','--perf-log','effects-on.csv','--capture','effects-on.png')
.\scripts\launch-gui.ps1 -GuiArgs @('--headless','--static-world','--lights-steady','--frames','160','--fx-off','fxaa','--perf-log','fxaa-off.csv','--capture','fxaa-off.png')
.\scripts\launch-gui.ps1 -GuiArgs @('--headless','--static-world','--lights-steady','--frames','160','--fx-off','ao','--perf-log','ao-off.csv','--capture','ao-off.png')
.\scripts\launch-gui.ps1 -GuiArgs @('--headless','--static-world','--lights-steady','--frames','160','--fx-off','bloom','--perf-log','bloom-off.csv','--capture','bloom-off.png')
```

**`--headless` is load-bearing and the card originally omitted it.** `ingest.rs:605` hardcodes
`vsync: !args.headless` with no other way to disable vsync, so a windowed run is
`PresentMode::Fifo`: all four p50s would have clamped to the refresh interval in 16.67 ms steps and
the comparison would have carried no information about any effect. This is the same correction
11.1a's review made to its own card; 11.1b's was written without it and was corrected before the
sitting. Preamble on all four confirms it took:

```text
build=5cd523c assets=disk:D:\Workspace\frostvein\assets subdiv=4 vsync=off
terrain=1888 trees=259 dwarves=5
```

The stamp is clean — no `-dirty`, and it names the merged tip, so the binary is the code.

## AC8 — MET. No effect costs the 60 fps bar.

Steady-state frame time, `scripts/bench/perf_summary.py`, n=166 per run:

| percentile | effects-on | `--fx-off fxaa` | `--fx-off ao` | `--fx-off bloom` |
| --- | ---: | ---: | ---: | ---: |
| **p50** | **3.33 ms** | **3.55** | **3.42** | **3.36** |
| p95 | 4.33 | 4.67 | 5.03 | 4.73 |
| p99 | 27.58 | 35.93 | 33.65 | 34.81 |
| p99.9 / worst | 54.58 | 62.45 | 54.56 | 64.87 |

**NFR6's bar is 16.67 ms and the worst p50 of the four is 3.55 ms — 4.7x of headroom.** Nothing is
owed to a ruling: no effect costs the bar, so this story tunes nothing.

**Every effect-off run is SLOWER than effects-on, and that is what carries the reading.** A
post-process pass cannot make rendering quicker, so none of these three deltas is a cost — each is
run-to-run variance, and FXAA's, AO's and bloom's true costs are all below this setup's measurement
floor. It is the SIGN that carries it, not the size, which matters because **one run per condition
is not a noise floor** (this project's own rule, issue #98). **No cost figure is published from
these deltas**, and none should be quoted from this table.

**Say which cost the AO column measures.** `--fx-off ao` removes `DepthPrepass` and `NormalPrepass`
along with the SSAO component, so the AO-off run is genuinely without AO's cost. Before the
2026-09-19 review it removed only the SSAO component and left both full-scene prepasses running,
which would have under-reported AO's true cost by exactly its expensive half.

**The hitch profile, recorded and NOT attributed to any effect.** p99 sits at 27.58–35.93 ms and
each run carries one hitch >= 50 ms, in a 0.7 s window. It is present with every effect OFF as
readily as ON, so it is not effect-attributable and this story does not own it. Filed here so it is
measured evidence rather than something rediscovered later; a 0.7 s window is too short to
characterise it, and nothing here should be read as having done so.

## AC9 — SIGNED OFF by Wolf at the sitting, 2026-09-20

The judged pair is `effects-on.png` against `bloom-off.png` at boot framing, filed beside
`fxaa-off.png`, `ao-off.png` and all four CSVs.

Bloom reads. The camp pit floor and the lantern cluster blow to white with bloom on and resolve
into readable geometry with it off; the halo is concentrated at the camp rather than spread across
the frame, which the near-white measurement below independently confirms.

Two things known before judging, both measured, neither a reason to change anything:

- **AO does not read as creases darkening**, as issue #106 predicted and as Wolf ruled on
  2026-09-18. AO is accepted here as WIRED AND PROVED; its look judgement is deferred to 11.3,
  where the day/night cycle raises ambient and gives it something to attenuate.
- **The near-white ceiling did not trip** — see below. AC5 deferred that judgement to this sitting.

## AC5's deferred ceiling judgement — the ceiling holds on the vehicle, with room

`near_white <= NEAR_WHITE_AREA_CEILING` is asserted inside the capture range check
(`capture.rs:1519`), which panics on trip, so all four runs exiting 0 already establishes the pass.
Measured directly off the four delivered PNGs rather than inferred from an exit code, mirroring
`capture.rs`'s `near_white_area_fraction` — Rec.709 luma coefficients, threshold 200, fraction of
pixels at or above it:

| frame | near-white area | ceiling | headroom |
| --- | ---: | ---: | ---: |
| `effects-on.png` | 0.5012% | 0.9461% | **+0.4449 pp** |
| `fxaa-off.png` | 0.4824% | 0.9461% | +0.4637 pp |
| `ao-off.png` | 0.4856% | 0.9461% | +0.4605 pp |
| `bloom-off.png` | 0.4891% | 0.9461% | +0.4569 pp |

**The trip the card expected did not happen, and the headroom is about 2.5x what was predicted.**
The prediction was ~0.18 pp headless headroom against a recorded 0.4–0.6 pp delivery-GPU penalty
near-white — a penalty larger than the whole headroom, hence the expectation of a trip. At boot
framing on this build the vehicle shows +0.4449 pp of headroom with every effect on.

**What this does NOT establish.** No same-build headless capture was taken for comparison, so this
says nothing about the size or direction of the venue delta on `5cd523c`; it measures the vehicle
alone. The recorded 0.4–0.6 pp penalty was measured in the bright tail, and a night boot framing is
not the bright tail, so this is not evidence against it either. The venue delta has been
mispredicted twice before and is not settled by this run.

**Bloom's own contribution to near-white is 0.0121 pp** (0.5012% on, 0.4891% off). That is
consistent with a halo concentrated at the camp and is not a frame-wide brightening.

## The gap this sitting exposed, for the next comparison set

The `--static-world` pause tick is printed to the console and **is not written into the perf-log
preamble**. The preamble's content counters (`terrain=1888 trees=259 dwarves=5`) are identical
across all four runs, but those count SPAWNED ENTITIES (issue #83) and read the same whether the
daemon was restarted per run or not — so the artifacts alone cannot show that a comparison set is
comparable. Here that was closed by Wolf's word at the sitting, which is sound but does not survive
into the record. **The pause tick belongs in the `# run:` preamble line**; then a comparison set
proves its own validity from the files and any later reader can check it.

Recorded by the devpod seat from Wolf's delivered artifacts. **No vehicle observation or FPS figure
was invented here**; every number above is computed from the four CSVs and four PNGs in this folder.
