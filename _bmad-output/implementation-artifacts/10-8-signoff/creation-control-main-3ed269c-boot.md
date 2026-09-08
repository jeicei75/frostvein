# Creation control frame — `main` 3ed269c, 2026-09-08

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
