# 11.1a crease instrument proof — re-measured at EV100 10.5, 2026-09-19

Re-taken at the code review after the exposure ruling. Build `77d5056` plus the review patches; all
frames `--headless --static-world --subdiv 4 --frames 160`, fresh daemon per capture. Luma is
integer **Rec.601** — the same statistic as `lumstats.py:38` and `pixel_guard.rs:35-41`, and
deliberately NOT `capture.rs:604`'s Rec.709.

## GREEN — same build, FXAA-on a vs b

| window | p10 a → b | median a → b | mean a → b |
| --- | --- | --- | --- |
| terrace-creases | 23 → 23 (**0**) | 50 → 50 (**0**) | 53.363 → 53.363 |
| open-snow-LL | 51 → 51 (**0**) | 94 → 94 (**0**) | 83.016 → 83.089 |
| open-snow-LR | 53 → 53 (**0**) | 94 → 94 (**0**) | 86.229 → 86.143 |

Every pinned window has zero `p10` and zero `median` delta. None is on the camp, which AC6 forbids
measuring in — the camp window's mean moves 1.19 between same-build runs from light flicker.

## RED — FXAA-on control vs `--lights-off ambient`

The ambient-off capture intentionally exited 101 after writing its PNG: its ground median is below
the value floor, which is the point of the control.

| window | p10 | median | mean |
| --- | --- | --- | --- |
| **terrace-creases** | **23 → 0** (−23, a **100%** fall) | 50 → 2 | −32.011 |
| open-snow-LL | 51 → 0 | 94 → 82 | −19.331 |
| open-snow-LR | 53 → 5 | 94 → 82 | −17.783 |

**AC7 is met on its amended, proportional form: the crease window's `p10` falls by 100%, against a
bar of 75%.**

The bar used to read "at least 25 LEVELS", and that absolute form became arithmetically unreachable
when the exposure was ruled to 10.5: the crease window's own `p10` is then 23, so even a total
collapse to zero is a 23-level drop. The instrument had not weakened — it still drives `p10` to 0
and the median 50 → 2. The unit was wrong, because a level threshold is exposure-dependent and
exposure is now a ruled art variable that moves again in 11.1b. No bar was lowered to fit a run:
100% clears 75% outright, and the pre-ruling measurement (31 → 0 at 9.7 EV100) also passes the new
form. Wolf ruled the restatement at the 2026-09-19 code review; see AC7.
