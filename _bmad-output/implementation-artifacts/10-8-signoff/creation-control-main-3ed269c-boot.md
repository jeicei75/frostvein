# Creation frames — `main` 3ed269c, 2026-09-08

The shipped look, captured at story creation so the story starts from a reading and not a quote.
This is the CONTROL, not a candidate and not a verdict; the filename carries the commit, nothing else.

- Build: `target/debug/gui`, stamp `gui build 3ed269c` (matches `main`); daemon `simd 0`, fresh world.
- Command: `gui <port> --headless --frames 160 --capture <png> --subdiv 1` (the pixel guard's own recipe).
- Exit: **101** on the near-white clause.

```
capture range check: warm-lit pixels=21037 ground-median-luminance=135 near-white-area=2.1686% blown-pool=1.1077% p99-luminance=229.4
near-white area is 2.1686%, above the 1.5630% ceiling calibrated on boot7.png
```

Single run; near-white swings ~0.1 pp between runs from dwarf motion (10.7). The blown-pool figure is
printed for diagnosis and is NOT asserted (`capture.rs`: "AREA IS THE ASSERTION, not the pool").

## The second control run and two probes (same daemon recipe, same build)

| file | `--lights-off` | range check | exit |
|---|---|---|---|
| `creation-control-main-3ed269c-boot-b.png` | — | `warm-lit pixels=21988 ground-median-luminance=136 near-white-area=2.2088% blown-pool=1.1196% p99-luminance=230.3` | 101 |
| `creation-probe-torches-off-3ed269c.png` | `torches` | `warm-lit pixels=9702 ground-median-luminance=122 near-white-area=1.3342% blown-pool=0.2936% p99-luminance=208.7` | **0** |
| `creation-probe-sun-off-3ed269c.png` | `sun` | `warm-lit pixels=21219 ground-median-luminance=117 near-white-area=1.7378% blown-pool=1.0318% p99-luminance=228.6` | 101 |

lumstats means: a 101.114 · b 101.218 · torches-off 99.768 · sun-off 87.800. Probes are for the
record; they are not candidates and carry no verdict.
