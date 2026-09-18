# 11.1a crease instrument proof — `c3735d1`, 2026-09-18

All frames use `--headless --static-world --subdiv 4 --frames 160`; luma is integer Rec.601.

## GREEN — same build, FXAA-on a vs b

| window | p10 a → b | median a → b | mean a → b |
| --- | --- | --- | --- |
| terrace-creases | 31 → 31 (0) | 65 → 65 (0) | 68.197 → 68.182 |
| open-snow-LL | 68 → 68 (0) | 117 → 117 (0) | 104.235 → 104.235 |
| open-snow-LR | 69 → 69 (0) | 117 → 117 (0) | 107.858 → 107.858 |

All pinned windows have zero p10 and median delta. None is on the camp.

## RED — FXAA-on control vs `--lights-off ambient`

The ambient-off capture intentionally exited 101 after writing its PNG: ground median was 43,
below the capture floor of 70. The crease window moves `p10` **31 → 0**, a 31-level drop, which
exceeds the required 25-level discriminating delta (median 65 → 2). The other two non-camp windows
also darken, but are diagnostics rather than the AC7 threshold.
