# AC4 — Candidate night-key tables, for Wolf to choose from

**Executed 2026-09-08** on branch commit `d04e59f`, at the shipped `k = 4` default (no `--subdiv`
flag), boot framing, `--frames 160`, no `--z`. One daemon per candidate.

**Every candidate edit was UNCOMMITTED and REVERTED.** `crates/gui/src/appearance.rs` was restored
with `git checkout --` after each candidate's two captures, and the tree was verified clean between
candidates. Nothing has landed. The build stamp for a candidate capture therefore reads
`gui build d04e59f-dirty`, which is the honest signal that the frame came from an uncommitted edit —
the exact values are listed below so each is reproducible.

**Ruling 1 is what these obey:** a NIGHT MOONLIGHT key, turned by the INTENSITIES. No `Exposure`
component. Sky, aurora and the star shell are untouched in all three — they carry no named defect.

## What each candidate changes, one variable at a time

The ladder is deliberate: each step adds exactly one knob family to the one before, so the record
can say which knob moved the picture rather than attributing a combined result to a whole table.

| | ambient | ambient brightness | directional | directional illuminance | torch | campfire | lantern |
|---|---|---:|---|---:|---:|---:|---:|
| **shipped control** | `(120,140,165)` | 4,500 | `(150,190,180)` | 22,000 | 14.0M | 25.0M | 5.0M |
| **A** cold fill dimmed | unchanged | **2,400** | unchanged | **7,000** | unchanged | unchanged | unchanged |
| **B** A + moon-coloured key | **`(108,128,170)`** | 2,400 | **`(178,200,240)`** | 7,000 | unchanged | unchanged | unchanged |
| **C** B + emitters trimmed | `(108,128,170)` | 2,400 | `(178,200,240)` | 7,000 | **9.0M** | **18.0M** | **3.5M** |

C's campfire peak is `18.0M × 1.40 = 25.2M`, under the `APPROVED_PEAK` of 35.52M that Wolf ruled at
6.2, so C does not reopen that ruling. All three only ever LOWER an emitter, never raise one.

## Figures — two runs each, same build

| candidate | run | warm-lit | ground median | near-white | blown pool | p99 | mean | exit |
|---|---|---:|---:|---:|---:|---:|---:|---|
| control | a | 27,664 | 126 | 2.4398 % | 1.2082 % | 232.4 | 94.472 | 101 |
| control | b | 27,500 | 126 | 2.4872 % | 1.2735 % | 233.7 | 94.588 | 101 |
| **A** | a | 35,725 | 100 | 1.9213 % | 1.1867 % | 232.0 | 77.089 | 101 |
| **A** | b | 37,219 | 101 | 1.9846 % | 1.2554 % | 232.5 | 77.276 | 101 |
| **B** | a | 33,800 | 97 | 1.9102 % | 1.1832 % | 232.0 | 74.883 | 101 |
| **B** | b | 34,422 | 97 | 1.9379 % | 1.2345 % | 231.7 | 74.918 | 101 |
| **C** | a | 29,948 | 94 | 1.5429 % | 0.9743 % | 221.8 | 74.054 | **0** |
| **C** | b | 31,230 | 94 | 1.6102 % | 1.0781 % | 224.9 | 74.241 | 101 |

Frames: `candidate-<name>-d04e59f-{a,b}.png`. Control pair: `AC3-all-on-{a,b}-c7bfb00.png` and
`control-k4-25f217b-{a,b}.png`.

### Read the exit column carefully — it is informational, not a verdict

Those exit codes are measured against `NEAR_WHITE_AREA_CEILING = 1.5630 %`, **which is calibrated
on `boot7.png`, a frame rendered with the sun under the map.** Premise 3 records that the
calibration frame no longer represents the game, and **AC6 re-derives every capture constant from
the APPROVED treatment's own two runs.** So a candidate exiting 101 against the old ceiling is not
a failing candidate; it is the old ceiling failing, which is what this story exists to fix.

What AC6 *does* require is that the shipped control still sits ABOVE the re-derived ceiling, so the
guard can still tell the two apart. All three candidates satisfy that:

| candidate | new ceiling = worst run + that pair's swing | control at 2.4635 % |
|---|---:|---|
| A | 1.9846 + 0.0633 = **2.0479 %** | above ✓ |
| B | 1.9379 + 0.0277 = **1.9656 %** | above ✓ |
| C | 1.6102 + 0.0673 = **1.6775 %** | above ✓ |

C is the only one that would also clear the OLD boot7 ceiling, and it straddles it — one run under,
one over, a 0.0673 pp swing across a bar it sits on. That is exactly the run-to-run behaviour AC6
was written to accommodate, and it is why AC5's "exit 0 on two consecutive runs" only becomes
meaningful after the ceiling is re-derived.

## What each knob actually did

Noise floor from the control pair: near-white `0.0474 pp`, mean `0.116`.

| step | near-white | ×noise | frame mean | ×noise |
|---|---:|---:|---:|---:|
| control → A, cold fill dimmed | −0.5105 pp | 10.8× | −17.347 | 149.5× |
| A → B, key recoloured | −0.0289 pp | **0.6×** | −2.282 | 19.7× |
| B → C, emitters trimmed | −0.3475 pp | 7.3× | −0.753 | 6.5× |

Three things fall out of that, and none of them was obvious before the captures:

1. **Dimming the cold fill is the whole picture.** It carries 149× the noise on frame mean and
   two-thirds of the near-white reduction. This is the ambient finding from AC3 showing up again:
   the valley's light is the ambient, not the key.
2. **Recolouring the key is invisible to the highlight structure.** A → B moves near-white by
   `0.0289 pp`, which is **BELOW the noise floor** — the instrument cannot distinguish it. It does
   move the frame mean and the ground median (100 → 97), so it is a value change, not a highlight
   change. If Wolf prefers B's cooler cast it costs nothing, but it must then move in lockstep
   through `valley_bench.py` and both `bench_contract.rs` anchors (AC8), which is real work for a
   difference no capture metric can see. **B's case has to be made by eye or not at all.**
3. **Only trimming the emitters moves near-white meaningfully once the fill is down.** B → C is
   7.3× noise on near-white against 6.5× on mean — it reshapes the highlights without darkening the
   scene further, which is precisely what UX-DR10 asks for (snow midtone, only emissive near white).

## Read by eye

All three read as night rather than the control's daylit snowfield. The control's long hard tree
shadows are gone in every candidate, because the directional that cast them is down from 22,000 to
7,000. The snow sits blue-grey and the camp is the brightest thing in frame in all three, which is
UX-DR10's rule.

- **A** — the valley reads as night, the camp pool still spreads wide and its core is still flat
  white. Closest to the control in colour.
- **B** — all but indistinguishable from A at a glance; the cast is slightly cooler and the ground
  a little deeper. The numbers agree: the only column that moved beyond noise is the mean.
- **C** — the camp pool is visibly tighter and its white core smaller, the ring of lit snow reads
  as firelight falling off rather than as a blown plate. The valley is otherwise B.

**One thing to judge that no metric here covers:** with the directional at 7,000 the moon casts
almost no visible modelling on the snow. If Wolf wants the moon to *read* as a key — a direction to
the light, faint shadows off the trees — that is a fourth candidate between 7,000 and 22,000, and
it is a choice about look, not about the guard.

## Not decided here

The brightness itself is Wolf's, from these frames. Once chosen: Task 4 lands the table with the
bench in the same commit (AC8, if a colour moved), Task 5 re-derives the capture constants from the
approved pair's own two runs (AC6), and AC7's two `appearance.rs` tests are corrected to the new
values with the cause stated rather than widened.
