# 11.3 Task 4a — candidate day tables (AC12)

Rendered 2026-09-24 by the orchestrator in an ISOLATED `git worktree` of `bb893b2` (the shared tree
was never edited). Each candidate is a temporary edit of `day_lighting()`'s five literals only, with
no other code changes. Boot framing, all effects on, one fresh `simd --pause-at 120` per capture:

```
gui <port> --headless --static-world --lights-steady --subdiv 4 --frames 160 --clock <h> --capture <png>
```

Side by side: `candidates-contact-sheet-bb893b2.png` (top: the approved night at 22:00 for reference).

## Tables

| cand | sky sRGB | ambient sRGB / brightness | key sRGB / lux |
| --- | --- | --- | --- |
| **A** provisional (= creation probe p3) | (110,155,205) | (190,210,235) / 4,000 | (255,244,228) / 12,000 |
| **B** soft warm | (130,170,215) | (180,200,230) / 3,000 | (255,236,210) / 11,000 |
| **C** crisp blue | (95,145,210) | (170,195,235) / 3,500 | (255,250,240) / 14,000 |
| **D** pale overcast | (175,190,210) | (200,210,225) / 5,000 | (235,238,245) / 8,000 |

`star` and `aurora` stay the night table's (they fade by weight, not by table).

## Range-check lines

| frame | warm-lit | ground median | near-white | blown-pool | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| night 22:00 (control) | 23,433 | 69 | 0.3906 % | 0.3637 % | 153.5 |
| A noon | 10,218 | 161 | 0.8519 % | 0.5068 % | 197.4 |
| A dusk 17.5 | 13,602 | 109 | 0.4297 % | 0.3896 % | 180.2 |
| B noon | 13,787 | 149 | 0.5590 % | 0.4556 % | 188.2 |
| B dusk 17.5 | 16,518 | 95 | 0.4117 % | 0.3774 % | 171.1 |
| C noon | 10,042 | 157 | 0.8163 % | 0.5136 % | 196.2 |
| C dusk 17.5 | 14,096 | 99 | 0.4166 % | 0.3818 % | 176.9 |
| D noon | 9,762 | 156 | 0.7526 % | 0.4768 % | 196.3 |
| D dusk 17.5 | 13,093 | 118 | 0.4527 % | 0.3992 % | 182.3 |

Every frame passed the existing night band unchanged (AC8): no skip needed for any candidate.
A's noon and dusk reproduce the main tree's `bb893b2` captures exactly; A's noon is `cmp`-identical
to the creation probe `probe-p3-day-flat.png`.

## AC9's sky bar, per candidate (`sharpness.py`, `sky-stars` lap_mean)

| frame | lap_mean | fall vs night 1.8812 |
| --- | ---: | ---: |
| A noon | 0.5228 | −72 % |
| B noon | 0.2550 | −86 % |
| C noon | 0.8599 | −54 % |
| D noon | 1.7583 | **−7 %** — would FAIL AC9's ≥ 50 % bar as written |

## Things to know before picking

1. **Dusk has no warm light.** The key keeps the DAY colour down to the horizon and only dims
   (horizon ramp over the last 10° when these frames were rendered — see "Changed after the pick"
   below); the sky blends to night between 17:00 and 19:00. A warm
   sunset tint is not in the model — adding one is a new ruling, not a table pick.
2. **The haze brightens by day.** AC11 derives the haze ambient intensity from the ambient
   brightness (`0.1 × brightness / 80`): night 1.875 → A 5.0, B 3.75, C 4.375, D 6.25. That is
   the milky wash over the far ridge in every noon frame, strongest in D.
3. **Torches, lanterns and the campfire are unchanged by day** (guardrail) — visible as the warm
   pool at camp in every noon frame.

## Changed after the pick — the sun's horizon ramp is now 25°

Wolf picked A live on `probe-11-3-day-toggle`, which carried the 10° sun ramp that rendered the
frames above. `7ad8b32` then widened the **sun's** ramp to 25° to meet AC4's literal 100-lux step
bar; the moon keeps 10°. Wolf ruled 2026-09-24 (code review): **keep 25°, disclose it, and judge
dawn and dusk live at AC13.**

- At 17:30 (sun 5.2° up) the key falls from 6,396 to 1,351 lux (−79%).
- The sun is now below full strength before about 08:35 and after about 15:25. With the 10° ramp
  that was before about 06:58 and after about 17:02.
- The A-dusk frame above is therefore **no longer what the build renders**. On the reviewed
  build (`6cc4981`/`d917314`, same code) the 17:30 ground median reads 108 (was 109) but p99
  reads 169.0 (was 180.2), so the ground median alone understates the change. Noon (12:00, sun
  40° up) is unaffected and stays `cmp`-identical to `approved-day-bb893b2.png`.
