# 11.3 Task 9d — moon candidates

Rendered 2026-09-25 in an ISOLATED `git worktree` of `71711bb` (the shared tree was never edited).
The probe reads two env overrides on `night_lighting()`: `directional_illuminance` (the moon)
and `ambient_brightness`. **Control:** the probe with no override set is `cmp`-identical to the
unpatched build at `--clock 22`. Boot framing, all effects on, one fresh `simd --pause-at 120`
per capture:

```
gui <port> --headless --static-world --lights-steady --subdiv 4 --frames 160 --clock <h> --capture <png>
```

| candidate | moon lux | night ambient | 22:00 ground median | 19:30 ground median | capture exit |
| --- | ---: | ---: | ---: | ---: | --- |
| m7000 (today) | 7,000 | 1,500 | 69 | 62 | 0 / 0 |
| m3000 | 3,000 | 1,500 | 62 | 57 | 0 / 0 |
| m1500 | 1,500 | 1,500 | 58 | 54 | 0 / **101** |
| m750 | 750 | 1,500 | 54 | 52 | **101** / **101** |
| m750a1000 | 750 | 1,000 | 48 | 44 | **101** / **101** |

A **101** is the capture band's ground-median floor (55), which was calibrated on the 7,000-lux
night. That floor must not choose the look. After the pick, it re-baselines with AC2's control and
the boot-calibrated rendered guards. Every PNG was still written.

Side by side: `moon-contact-sheet-71711bb.png`. Each row is a candidate; the left column is 22:00
and the right is 19:30.

For reference, the noon sun is 12,000 lux. Today's moon is 58% of that; m3000 is 25%, m1500 13%
and m750 6%.
