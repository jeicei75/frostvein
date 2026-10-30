**Re-checked at 13.6's creation (2026-10-29), as ruled 2026-10-28: it still reproduces on `main` (`0a8b4462`, 13.4 merged), so it stays open. Wolf ruled 2026-10-30 (13.6 Q1 (c)) that it folds into Story 13.18 (stones seal two miners, #204) rather than a story of its own.**

Probe (`13-6-signoff/prototypes/probe187-refusals.patch`): a throwaway counter at the channel cut's refusal site (`crates/sim-core/src/lib.rs:2537-2542`, the `walls_in_a_dwarf_from` → `retry_claim` branch), then the four `UNFINISHED_LATE` cage runs (`scenario.rs:3054`) to 16,000 ticks with `13-4-signoff/probe187.rs`'s setup (4x7 / 6x10 channel beside the fire, 3x3 pile), release build.

| run (seed, width) | marks left at 16k | refusals, whole run | refusals t12k–16k |
|---|---:|---:|---:|
| (0, 4) | 1 | 205 | **56** |
| (6, 6) | 1 | 161 | **55** |
| (9, 6) | 1 | 0 | 0 |
| (13, 6) | 1 | 0 | 0 |

Runs (0,4) and (6,6) are this issue's loop: the same cut is refused about every 72 ticks to the end of the run. Runs (9,6) and (13,6) never reach a refusal (never claimed, #218 shape B). The committed pin `no_dwarf_is_caged_by_stones_over_the_probe_seeds` still holds all four on `main`, so "done when that pin empties" is still the test.
