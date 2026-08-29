# Epic 9 shared sitting — the short card (2026-08-29)

Everything measurable for 9.1 and 9.4 has been measured headlessly. **This sitting owes four
readings and nothing else.** The long per-story cards (`9-1-signoff/task-6-vehicle-runbook.md`,
`9-4-signoff/task-7-vehicle-runbook.md`) carry the history; this is the checklist.

**Vehicle:** gingerspice only (native Windows / NVIDIA Vulkan). Fill the blanks; infer nothing.

## 0. Build identity — before any observation

```
git checkout 9-4-trees-fewer-and-distinct-from-the-ground
git pull
```
Run the client once and read its first line: `gui build <sha>`. It must match
`git rev-parse --short HEAD` and must **not** say `-dirty`. If it does, stop — you are holding a
stale binary, which is the one failure this project has hit repeatedly.

`gui build`: ______________   `HEAD`: ______________

## 1. ONE capture — closes 9.1's AC13 ceiling half

```
simd.exe 7451
gui.exe 7451 --capture 9-1-vista.png --at-tick 20
echo "exit=$?"
```

Read the range-check line. **It gained a field on 2026-08-29** and now reads:
`capture range check: warm-lit pixels=N ground-median-luminance=N near-white-area=X% blown-pool=Y% p99-luminance=Z`

  near-white-area: ______ %   ← THE ONLY BAR THAT MATTERS: `<= 1.5630426 %`
  blown-pool:      ______ %   ← DIAGNOSTIC ONLY. Do NOT judge by it, do NOT compare to 0.6651 %.
  ground-median:   ______     (band 70-180)
  p99:             ______     exit: ______

**EXPECT exit 101.** Headless predicts ~1.74 % area on this branch against a ceiling calibrated on
a GPU frame. That is the measurement, not a broken build — the PNG is written before validation, so
the evidence survives. At or under the bar CONFIRMS the ceiling; over it IS the correction. Either
answer closes AC13.

Also expected at startup: `projected 44984 terrain cubes at z 31`. It read 53365 before 9.4.
**Not a regression** — tree density changed, then the ground-level foliage ring was removed.

## 2. Your eye on the trees — closes 9.4's AC10

Look at the live valley. The tree work changed shape as well as colour, so this needs a fresh look
even though you saw an earlier build:

- [ ] Fewer trees — 704 -> 265 on the shipped seed.
- [ ] Foliage reads **green**, not the old blue-grey that sat 9.9 from stone.
- [ ] **Every tree has a visible trunk.** 86 of 265 previously drew none at all.
- [ ] **No green cubes lying on the ground** beside trees, and no bright snow ring at their bases.

AC10 verdict: ____________________________________________

## 3. Two re-checks for 9.1 — both already answered, confirm or contradict

- **AC14** (already NO): does the fire read as light on snow rather than glare, against
  `5-4-signoff/candidate-artifact-2026-08-15.png`? Nothing about the fire changed since your
  reading, so this is a confirmation.  ____________________
- **AC15** (causation already answered YES): hover the cursor on terrain near the campfire — is the
  hover slab visible? Measured cause: the slab is luminance 189.5 and the pool exceeds 200 and
  saturates to 255, so it inverts. The rendered fix is 9.2's.  ____________________

## 4. Report back

Four things: the `near-white-area` figure and exit code, your AC10 verdict, and the two re-checks.
Both stories then close on those.
