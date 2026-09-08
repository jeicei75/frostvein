# AC3 re-taken — per-emitter marginals at the APPROVED treatment

**Supersedes `AC3-marginals.md`**, whose every figure was measured against the OLD camp with the
four torches at ±2 cells and the shipped lighting table. Both changed, so that table is history,
not evidence. It is kept in place for the record; **read this one.**

**Executed 2026-09-08.** Build stamped `gui build 3548a95`, equal to HEAD, no `-dirty`. One daemon,
boot framing, `--frames 160`, no `--subdiv` (k=4 is the shipped default), no `--z`. Approved
treatment: the moonlight table at the ±8-cell torch ring.

## THE HEADLINE: the approved treatment passes its own guard

```
all-on a  warm-lit=35379 ground-median=83 near-white=0.7959% blown-pool=0.4487% p99=189.5   EXIT 0
all-on b  warm-lit=33539 ground-median=82 near-white=0.7670% blown-pool=0.4601% p99=187.9   EXIT 0
```

Against the ceilings re-derived from this very treatment (`NEAR_WHITE_AREA_CEILING = 0.946072 %`,
`BLOWN_POOL_FRACTION_CEILING = 0.6238 %`) and the untouched `GROUND_LUMINANCE_FLOOR = 70`.
**AC5 is satisfied: exit 0 on two consecutive runs.** This is the first time in this story's
history that a clean boot capture has exited 0 — the shipped `main` has been red on the near-white
ceiling since before 10.7.

## The table

Noise floor, worst of the all-on pair: warm-lit `1,840 px` · ground median `1` · near-white
`0.0289 pp` · blown pool `0.0114 pp` · mean `0.144`. All-on reference is the pair's midpoint.

| source off | warm-lit | ground median | near-white | blown pool | p99 | mean | Δmean | ×noise | exit |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| — (a) | 35,379 | 83 | 0.7959 % | 0.4487 % | 189.5 | 65.560 | — | — | **0** |
| — (b) | 33,539 | 82 | 0.7670 % | 0.4601 % | 187.9 | 65.416 | — | — | **0** |
| `sun` | 35,206 | 68 | 0.7571 % | 0.4605 % | 187.1 | 58.617 | −6.871 | 47.7× | 101 |
| `ambient` | 73,151 | 52 | 0.6707 % | 0.4372 % | 181.6 | 38.143 | −27.345 | 189.9× | 101 |
| `campfire` | 29,653 | 81 | 0.6155 % | 0.3959 % | 177.1 | 64.911 | −0.577 | 4.0× | 0 |
| `torches` | 12,510 | 67 | 0.6904 % | 0.4260 % | 177.5 | 62.950 | −2.538 | 17.6× | 101 |
| `lanterns` | 30,302 | 79 | **0.3481 %** | **0.0620 %** | 167.9 | 64.467 | −1.021 | 7.1× | 0 |
| `campfire,torches` | 9,738 | 66 | 0.6089 % | 0.4498 % | 172.2 | 62.583 | −2.905 | 20.2× | 101 |

**Every 101 in this table is the FLOOR, not a ceiling.** With the table dimmed to a night key,
removing any major source drops the valley floor under `GROUND_LUMINANCE_FLOOR = 70` — sun 68,
torches 67, campfire+torches 66, ambient 52. That is the guard working as designed (*"a black
field, not a lit night"*) on a frame that deliberately has a main light switched off. It is NOT a
regression, and a diagnostic `--lights-off` capture is no longer expected to exit 0. Note this
before reading a future red as a defect.

## THE FINDING: the lanterns are now the white-maker

Wolf asked to see the lanterns separately. At the approved treatment they are **the single largest
source of near-white and almost the entire blown pool**:

| source | share of the near-white budget | share of the blown pool |
|---|---:|---:|
| **lanterns** | **55.5 %** | **86.4 %** |
| campfire | 21.2 % | 12.9 % |
| ambient | 14.2 % | 3.8 % |
| torches | 11.7 % | 6.3 % |
| sun | 3.1 % | 1.3 % |

Switching the lanterns off alone takes near-white from 0.7814 % to **0.3481 %** and collapses the
blown pool from 0.4544 % to **0.0620 %**.

**This is a complete inversion of the old camp**, where the torch ring outweighed the campfire
4.5:1 and the lanterns read as a rounding error (`AC3-marginals.md`, and 10.7's finding before it).
The cause is geometric, not photometric: **the torches moved out to ±8 cells and the lanterns did
not.** Whatever still sits at the centre of the camp now owns the bright core, and that is the
lanterns. Their intensity fell 5.0M → 3.0M in the approved table, proportionally less than the
torches' 14M → 7M, and they kept their position.

**What it means for the next decision:** if any further reduction of the white core is wanted, the
lanterns are the lever, not the campfire and not the torches. It also means the lanterns are now
worth their own look decision, exactly as Wolf suspected when he asked to see them alone.

## Torches against campfire, each measured with the other OFF

| marginal | warm-lit | near-white |
|---|---:|---:|
| campfire, torches already off | −2,772 | −0.0815 pp |
| torches, campfire already off | −19,915 | −0.0066 pp |

**The torch ring still dominates the WARM SIGNATURE, 7.2:1 on warm-lit** (it was 6.4:1 at the old
camp — the widening slightly increased it, because spread torches light more distinct surface).
**But on NEAR-WHITE the relationship reverses**: the campfire's marginal is now 12× the torches'.
Moving the torches away from the centre took them out of the blown core while leaving their warm
spread intact, which is precisely the outcome the widening was chosen for.

Read together with the warm-lit caveat carried over from the first table: `warm_lit_pixels`
(`capture.rs:548`) counts `red − blue > 30`, a purely relative test, so switching off a COOL source
INFLATES it — ambient off reads 73,151 here against all-on's 34,459. Judge a cool source on ground
median and frame mean.
