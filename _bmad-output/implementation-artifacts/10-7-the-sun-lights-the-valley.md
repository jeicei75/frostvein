---
baseline_commit: 47139fa
model: claude-opus-5[1m]  # Opus default; the 1M-context variant, recorded so the ledger row is readable
---

# Story 10.7: The Sun Lights The Valley

Status: in-progress

**RUNS BEFORE 10.5.** See "Why this is before the dwarves". The board's key is placed above
10.5 deliberately, because this board's next-story rule reads top to bottom and a prose ruling
alone has silently lost to numeric order twice on this project.

Created 2026-09-03 out of 10.4's vehicle sitting on Wolf's instruction, and **added to
`epics.md` § Epic 10 the same day** — the epic's execution-order ruling now reads
**10.6 → 10.3 → 10.4 → 10.7 → 10.5**.

## Story

As the boss,
I want the sun to actually light the valley,
so that every look judgement I make from here on is made under the lighting the game ships with,
instead of the ambient-only scene every judgement so far was made under.

## Premises re-verified at creation — 2026-09-03, on `47139fa`

The 2026-09-03 draft of this file was written from the deferred-work entry and had **not** had a
context pass. Five premises were checked against source: **four were wrong, incomplete, or
entirely absent**, and the fifth is confirmed still true. Read this section before the
Acceptance Criteria.

1. **IT IS A DIRECTION BUG, NOT A POSITION BUG.** Bevy's `DirectionalLight` ignores its transform's
   translation entirely — *"The light shines along the forward direction of the entity's
   transform"* (`bevy_light-0.19.0/src/directional_light.rs:25`, the crate this workspace pins at
   0.19.0). The sun is not "67.5 units under the world" in any sense the renderer reads. What is
   wrong is the **aim**: `looking_at(CAMP_FOCUS)` from a point below the camp yields
   `forward().y = +0.1118`, i.e. the light travels **upward**, and the sun sits at
   **−6.42° elevation — just below the horizon.** The old draft's "cheap shape" for the guard, an
   assertion on the light's world-space height, **would certify a broken build**: a light at
   Y = +500 rotated upward passes it. Elevation, not height, is the axis this story works in.

   | build | `aurora_core()` y | `forward().y` | sun elevation |
   |---|---:|---:|---:|
   | shipped | −58.5 | +0.1118 | **−6.42°** |
   | probe Y=100 | 100 | −0.1499 | +8.62° |
   | **probe Y=200** (the one that was measured) | 200 | −0.3033 | **+17.66°** |
   | probe Y=300 | 300 | −0.4363 | +25.87° |
   | probe Y=400 | 400 | −0.5459 | +33.09° |

2. **THE BENCH CARRIES THE SAME WRONG SUN, AND IS PINNED TO THE CLIENT.**
   `scripts/bench/valley_bench.py:149-159` reimplements `aurora_core()` and `sun_direction()`, and
   `crates/gui/tests/bench_contract.rs:192-193` greps both call sites and requires each to match
   **exactly once**. Both venues are wrong *identically*, which is the only reason they agree
   today. Changing one without the other re-opens exactly the client/bench divergence 10.4 existed
   to close.

3. **THE EXISTING DIRECTION TEST IS SELF-REFERENTIAL AND CANNOT BE THE GUARD.**
   `crates/gui/src/atmosphere.rs:361-368` asserts `aurora_light_transform().forward()` matches
   `(CAMP_FOCUS - aurora_core()).normalize()` — the expected value is derived from the same
   constant the defect lives in, so it passes happily with the sun below the horizon. It is green
   right now. This is the antipattern this project hit in 1.1, 1.2 and 1.3; the guard AC5 asks for
   must have an oracle **independent** of the sun constant.

4. **`NEAR_WHITE_AREA_CEILING` IS ALREADY BREACHED, ON BOTH VENUES, BEFORE THIS STORY — AND
   `gui --headless --capture` ALREADY EXITS 101 ON `main`.** Measured here, twice, on `47139fa`
   with no source changes:

   ```
   capture range check: warm-lit pixels=19732 ground-median-luminance=117 near-white-area=1.8757% blown-pool=1.1684% p99-luminance=233.5
   thread 'main' panicked at crates/gui/src/capture.rs:1348:5:
   near-white area is 1.8757%, above the 1.5630% ceiling calibrated on boot7.png
   EXIT=101
   ```

   Second run: `near-white-area=1.7755%`, same panic, same exit. The vehicle reads 2.2071 % on the
   shipped build. **A red capture range check is therefore NOT evidence that the sun broke
   anything** — it is the state of `main`. The PNG is still written: `save_before_validate`
   (`capture.rs:1250-1253`) saves first and validates second, so every capture in this story
   remains usable despite the non-zero exit. Do not "fix" this, and above all do not raise the
   constant to make your own run go green.

5. **The falsified `CascadeShadowConfig` candidate is confirmed still falsified** and its evidence
   frame is committed (`probe-cascade-max-500-FALSIFIED.png`). Leave it alone.

## Wolf's rulings — 2026-09-03

The three questions the draft left open are answered. They are the story's frame, not suggestions.

1. **Where the sun goes: benched, not pre-picked.** No elevation is written into this story. Task 2
   produces candidates and Wolf chooses, exactly as 10.4 chose candidate D. Y = 200 / +17.66° is
   the *probe that proved the mechanism* and is a legitimate candidate, not the answer.
2. **The sun and the aurora are DECOUPLED.** The sun gets its own transform and its own elevation
   constant; `aurora_core()` goes back to meaning only the curtain's bright point. A decorative
   constant silently steering the key light is the whole defect, and separating two concerns that
   already exist is not a new abstraction under the YAGNI policy.
3. **Nothing else is re-tuned.** 9.1's blow-out work, 9.4's tree colours, 10.3's rules of the look
   and 10.4's tree judgement are **not** re-opened here, and neither is the near-white ceiling —
   it is measured and recorded, not moved. Its pre-existing breach (premise 4) is filed as its own
   defect for its own story.

## Why this is before the dwarves

10.5 puts the first authored dwarves in front of Wolf for look judgement. **Judging them under a
scene with no sun repeats exactly the failure 10.4 exposed**: candidate D was approved against a
bench frame that differed from what the client drew, and the fix was to make the two agree before
asking for judgement. Same shape here, one level up.

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier, not `--fast`) is green, and the diff is confined to this
   story's own commit range from `baseline_commit` — not `main..HEAD`.

### The decision this story exists to produce

2. Bench artifacts exist for **at least two sun elevations**: the shipped −6.42° as the control,
   and at least one candidate above the horizon. Each carries the `range-check:` line
   `valley_bench.py` already emits; an artifact without that line is not evidence. **A candidate's
   figures must differ from the control's** — identical figures mean one treatment photographed
   twice.
3. The chosen elevation is recorded in this story with the artifact filename and the figures it
   rests on, naming who decided and when.

### The fix

4. **The sun measurably lights the valley.** Using `10-7-signoff/lumstats.py` on
   `gui --headless --subdiv 1 --frames 160` captures at boot framing, the mean-luminance change
   between the shipped build and the chosen candidate is **at least 10x the same-build noise
   floor**, and that noise floor is **re-measured on this story's own build** — two runs of one
   binary, the **worst** of the two taken — not quoted from this file. A figure published without
   its noise floor beside it is not evidence (10.4's AC5 published a delta inside its own noise).
5. **A guard fails when the sun returns below the horizon, and it asserts the light's DIRECTION.**
   Its oracle must be independent of the sun's own elevation constant (premise 3): a hand-written
   floor on `forward().y`, in the same style as `APPROVED_PEAK` in
   `campfire_keeps_local_contrast_over_the_midtone_cold_fill` (`appearance.rs:581`, inside the test at `:561`), which is
   deliberately not derived from the table it guards. **It must be shown to fail** by re-applying
   the shipped `aurora_core()` aim — that is one of the mutation rows, not a described intention.
6. **The client and the bench move in lockstep.** `bench_contract.rs`'s sun anchors are updated on
   both sides in the same commit and each still matches exactly once. A green `bench_contract.rs`
   with a bench that no longer aims where the client does is the 10.4 defect returning.
7. **`NEAR_WHITE_AREA_CEILING` is not moved.** The story records the near-white figure for the
   control and the chosen candidate, and states in one line that the ceiling was already breached
   before this story (premise 4) and is therefore not this story's to re-calibrate. **Do not raise
   it to clear a panic.** A capture exiting 101 on this check is expected on `main` today.
8. `_bmad-output/implementation-artifacts/mutations/10-7-the-sun-lights-the-valley.sh` carries at
   least **three rows that the mutation run kills**, one of them AC5's restore-the-shipped-aim row.
   `audit-mutations.py` on the gate checks rows still apply; it does not notice a missing file.

### Sign-off

9. UX-DR22 **opening** half: Wolf approved a bench artifact before the client change was
   implemented. UX-DR22 **closing** half: Wolf has viewed the built result live on the vehicle and
   compared it against the approved artifact.

## Tasks / Subtasks

- [x] **Task 1 — Decouple the sun from the aurora** (AC: 6; Wolf's ruling 2)
  - [x] Add a sun transform with its own elevation constant in `crates/gui/src/atmosphere.rs`.
        `aurora_light_transform()` is used at exactly one site, `crates/gui/src/ingest.rs:859` —
        `rg -n 'aurora_light_transform' --type rust` before and after to confirm you moved the
        only caller.
  - [x] `aurora_core()` keeps its other two uses (`atmosphere.rs:342` frustum check, and the
        curtain's bright point). Do **not** change the curtain's geometry: `AURORA_BOTTOM`,
        `AURORA_TOP` and `AURORA_RADIUS` are the curtain's, and they stay.
  - [x] Replace the self-referential assertion at `atmosphere.rs:361-368` (premise 3). It must not
        derive its expected value from the sun constant.

- [x] **Task 2 — Bench the control and at least one candidate elevation** (AC: 2, 3)
  - [x] Control first: `python3 scripts/bench/export_world.py <snapshot.json>` then
        `blender --background --python scripts/bench/valley_bench.py -- <snapshot.json> <out.png>`.
        Paste each `range-check:` line into the Dev Agent Record.
  - [x] **Candidate bench edits stay uncommitted**, or live in a scratch copy under
        `10-7-signoff/`. `bench_contract.rs` forbids a committed bench aim the client does not
        carry, so a committed candidate turns AC1 red and forces the client-first change Task 3
        exists to avoid. The lockstep edit happens once, in Task 3.
  - [ ] Artifacts land in `_bmad-output/implementation-artifacts/10-7-signoff/` beside the six
        frames already there.

- [x] **Task 3 — Wolf judges, and the decision is recorded** (AC: 3, 9 opening half)
  - [x] Present control and candidates side by side, each with its `range-check:` line and its
        elevation in degrees. Record the decision, the date, and the artifact it rests on.
  - [x] **Stop here until Wolf has ruled.** No client change before the opening half is signed.

- [ ] **Task 4 — Land the chosen elevation in client and bench together** (AC: 6)
  - [ ] Rust and Python in ONE commit. `bench_contract.rs:192-193` greps the client for
        `Transform::from_translation(aurora_core()).looking_at(CAMP_FOCUS, Vec3::Y)` and the bench
        for `vector_normalize(vector_subtract(CAMP_FOCUS, aurora_core()))`; both anchors move.
  - [ ] Both render paths must agree: at `--subdiv 1` every cell is a `Cuboid`, at `--subdiv > 1`
        trunks go through the chunk mesher. Lighting is per-scene, not per-path, so confirm rather
        than assume — capture at both and say so.
  - [ ] Do **not** touch `directional_illuminance` (22,000) or `ambient_brightness` (4,500).
        Ambient's balance genuinely cannot be judged until the sun is above the horizon, and that
        is the next story's question, not this one's.

- [ ] **Task 5 — The guard, its RED, and the mutation rows** (AC: 5, 8)
  - [ ] Write the direction guard per AC5. Independent oracle; hand-written floor.
  - [ ] Author `mutations/10-7-the-sun-lights-the-valley.sh`, ≥3 rows, format per
        `mutations/9-4-trees-fewer-and-distinct-from-the-ground.sh`. Suggested rows: (a) restore
        the shipped `aurora_core()` aim — AC5's required row; (b) flip the sign of the new
        elevation constant; (c) diverge the bench's `sun_direction()` from the client's, which
        `bench_contract.rs` must catch.
  - [ ] Run `scripts/mutate.sh` and record KILLED **per row, naming the mutation**. `mutate.sh`
        rewrites source in place and is **not** concurrency-safe — never run it alongside anything
        else. **Commit the fix before mutating**: undoing a mutation with `git checkout --` on an
        uncommitted fix destroys the fix.
  - [ ] Re-mutate after any strengthening. "KILLED" names the TEST, not your new assertion — an
        earlier assert can absorb the mutation while the line you just added has never run.

- [ ] **Task 6 — Measure, with the noise floor beside it** (AC: 4, 7)
  - [ ] Instrument: `_bmad-output/implementation-artifacts/10-7-signoff/lumstats.py`. It already
        exists and is already tested (see Verification). Cite it; do not write a third one, and do
        **not** use a pixel diff — its noise floor here is 38,989 pixels, larger than the signal.
  - [ ] Two runs of the shipped build for the noise floor, two of the candidate. Publish mean,
        dark(<40) and shade-band(40-89) for all four, and the ratio of signal to the **worst**
        noise reading.
  - [ ] Record near-white area for control and candidate from the `capture range check:` line.
        Record it. Do not act on it. (AC7)

- [ ] **Task 7 — Verification and the closing half** (AC: 1, 9)
  - [ ] Execute the recipe below, RED first, and paste both outputs into the Dev Agent Record.
  - [ ] Full `scripts/gate.sh` green, pasted.
  - [ ] Hand Wolf a vehicle card in the shape of
        `10-4-signoff/task-6-vehicle-runbook.md` for the closing half.

## THE RULING — Task 3, AC3, UX-DR22 opening half

**Wolf chose `+17.66°` on 2026-09-03.** The sun's elevation constant becomes `17.66` in the client
and the bench together.

| | elevation | artifact | terrain_luma | whole-frame mean | distinct colours | px from control (d>=4) |
|---|---:|---|---:|---:|---:|---:|
| control | −6.42° | `control-shipped-minus6.42.png` | 105.853 | 75.435 | 59,190 | — |
| candidate | +8.62° | `candidate-plus8.62.png` | 119.546 | 84.503 | 85,727 | 199,830 |
| **CHOSEN** | **+17.66°** | **`candidate-plus17.66.png`** | **132.927** | **93.374** | **90,237** | **232,431** |
| candidate | +25.87° | `candidate-plus25.87.png` | 143.913 | 100.564 | 89,906 | 253,437 |

Every row carries its `range-check:` line in the Dev Agent Record. The control's was reproduced
verbatim by an independent re-render whose PNG is bit-identical to the committed one (0 of 518,400
pixels differ), so the separation column above is signal over a zero-pixel floor.

**What the ruling rested on.** The three candidates were presented side by side with a
hold-to-compare against the shipped control, at
<https://claude.ai/code/artifact/89dbd61a-cd0f-476f-ae21-fe1c6bbe100a>, together with the figures
above and the three judgement calls the numbers cannot settle: the campfire's standing as the
valley's own light source, shadow length across the dig terraces, and the fact that these are
Cycles renders pinned to the client's constants rather than frames the client drew.
`+17.66°` was chosen as the elevation where directional modelling reads clearly — tree shadows on
the slope, depth in the terraces — while the campfire still owns its pool of warm light, which
`+25.87°` begins to take from it. It is also the elevation of the original Y=200 probe that proved
the mechanism.

**This is the OPENING half of UX-DR22 only.** The closing half needs Wolf's eye on the built result
on the vehicle, against this approved artifact (Task 7).

## Dev Notes

### Scope guardrails — do NOT

- **Do not re-tune any other look constant.** Wolf's ruling 3. 9.1's blow-out work, 9.4's tree
  colours, 10.3's rules of the look, 10.4's tree judgement: all were tuned with the sun off, all
  stay as they are. Raising the sun moves the ground they stand on; re-judging them is a later
  story and Wolf's call to schedule.
- **Do not move `NEAR_WHITE_AREA_CEILING`, `BLOWN_POOL_FRACTION_CEILING`, `WARM_PIXEL_FLOOR`,
  `GROUND_LUMINANCE_FLOOR/CEILING`.** The near-white breach is pre-existing and is not yours.
- **Do not touch `CascadeShadowConfig`.** Measured and falsified; changing it moved the frame less
  than run-to-run noise. It stays untested-and-unfixed until something needs it.
- **Do not change the aurora curtain's geometry or colour.** The curtain is decoration; this story
  separates the sun from it and otherwise leaves it exactly as it is.
- **Do not weaken or delete a test to make a capture exit 0.** Exit 101 on the near-white check is
  the state of `main` (premise 4).

### What already exists

- **The light:** one `DirectionalLight`, `setup_night_lighting` at
  `crates/gui/src/ingest.rs:851-862`, `shadow_maps_enabled: true` already set at `:856`, aimed by
  `aurora_light_transform()` (`atmosphere.rs:209-211`).
- **The constants:** `CAMP_SURFACE_Y = 9.0` (`atmosphere.rs:27`), `CAMP_FOCUS` (`:28`),
  `SKY_CENTRE` (`:33`), `AURORA_BOTTOM = -162.0` (`:42`), `AURORA_TOP = 45.0` (`:43`),
  `aurora_core()` (`:67-71`). Scale is one render unit per simulation cell
  (`crates/gui/src/transform.rs:4`).
- **The light table:** `night_lighting()` at `crates/gui/src/appearance.rs:40-50` —
  `ambient_brightness 4_500`, `directional_illuminance 22_000`, directional tint
  `srgb_u8(150, 190, 180)`. Pinned at `appearance.rs:407`.
- **The evidence, committed:** `10-7-signoff/` holds the six frames behind the table above, plus
  `lumstats.py` (the instrument that found this) and `pixel_diff.py` (the one that could not).
- **The capture CLI:** `gui <port> --headless --subdiv N --capture <png> --frames N`, parsed at
  `crates/gui/src/ingest.rs:535-600`. `--capture` bails without `--frames` or `--at-tick`.

### Key decisions and traps

- **Elevation is the axis, not Y.** Premise 1. Express the ruling and the guard in degrees above
  the horizon; a Y coordinate is 600 units of horizontal distance away from being meaningful.
- **`--at-tick` is unusable on both venues** — its floor counts observed ticks, and software
  rendering observes about a third of them while a fast GPU queues them into one frame. Use
  `--frames`.
- **Pausing the world to kill dwarf-motion noise does not work** — the capture's motion health
  floor panics *before* the screenshot when ticks stop, so a paused run writes no PNG at all.
  Measured 2026-09-03. Live with the noise and measure it instead.
- **Near-white area is itself noisy.** Two runs of `main` read 1.8757 % and 1.7755 % — a 0.10 pp
  swing from dwarf motion alone. A single near-white reading is weak evidence; say which run.
- **`bench_contract.rs` is a source-text grep, not a behaviour test.** Renaming a function breaks
  the suite even when nothing renders differently — and, the other way round, it cannot see that
  your two sides compute different angles. It pins text; you must check the maths.
- **The campfire ratio test's premise changes but its arithmetic does not.**
  `campfire_keeps_local_contrast_over_the_midtone_cold_fill` (`appearance.rs:560+`) sums
  `ambient_brightness + directional_illuminance` as the "cold fill". Neither constant moves in this
  story, so the test stays green — but it only becomes *true* once the sun is above the horizon.
  Expect a reviewer to ask; the answer is that this story does not change either number.

### Project structure

| Path | NEW/UPDATE | Note |
|---|---|---|
| `crates/gui/src/atmosphere.rs` | UPDATE | sun transform + elevation constant; replace the self-referential test |
| `crates/gui/src/ingest.rs` | UPDATE | `:859`, the one call site, points at the new transform |
| `scripts/bench/valley_bench.py` | UPDATE | `sun_direction()` — in lockstep with the client, per `bench_contract.rs` |
| `crates/gui/tests/bench_contract.rs` | UPDATE | the two sun anchors, both sides |
| `_bmad-output/implementation-artifacts/10-7-signoff/` | UPDATE | bench artifacts + the approved one, named |
| `_bmad-output/implementation-artifacts/mutations/10-7-the-sun-lights-the-valley.sh` | NEW | ≥3 rows |
| `_bmad-output/implementation-artifacts/deferred-work.md` | UPDATE | file the pre-existing near-white breach as its own defect |

## Verification

**Executed at story creation, 2026-09-03, on `47139fa`. Full gate GREEN, all 9 checks `ok`, no
skips.** The instrument was run RED first: a green from an instrument never seen to fail is a
habit, not evidence.

The instrument is `lumstats.py`. Two REDs were observed, because it can fail in two directions —
by dying, and by lying:

```bash
cd _bmad-output/implementation-artifacts/10-7-signoff
head -c 40000 control-shipped-a-e930d07.png > /tmp/truncated.png
python3 lumstats.py /tmp/truncated.png=truncated     # RED 1: a broken frame
python3 lumstats.py /tmp/black.png=all-black         # RED 2: a 64x64 all-black frame
```

Observed, verbatim:

```
RED 1  zlib.error: Error -5 while decompressing data: incomplete or truncated stream
RED 2  all-black    mean=  0.000  dark(<40)=  4,096 (100.00%)  shade-band(40-89)=      0 ( 0.00%)
```

RED 1 proves it dies loudly on a bad frame rather than printing plausible numbers. RED 2 proves a
blank capture reads as an unmistakable `mean=0.000 / dark=100.00%` and cannot masquerade as a lit
valley. Restore: nothing — both REDs are separate scratch files.

GREEN, on the committed evidence, reproducing the table this story rests on:

```
control-a              mean= 87.894  dark(<40)=161,492  shade-band(40-89)=223,502
control-b-NOISE        mean= 87.973  dark(<40)=161,495  shade-band(40-89)=223,412
sun-deleted            mean= 87.815  dark(<40)=161,489  shade-band(40-89)=223,560
sun-lifted-Y200        mean=101.188  dark(<40)=160,432  shade-band(40-89)=198,034
```

**The noise floor on THIS story's baseline**, two live captures of `47139fa` (the dev must
re-measure on their own build per AC4, but this is the figure to beat):

```bash
./target/debug/simd 7463 &
./target/debug/gui 7463 --headless --subdiv 1 --capture /tmp/a.png --frames 160   # exits 101, writes the PNG
./target/debug/gui 7463 --headless --subdiv 1 --capture /tmp/b.png --frames 160
python3 _bmad-output/implementation-artifacts/10-7-signoff/lumstats.py /tmp/a.png=run-a /tmp/b.png=run-b
```

```
main-run-a             mean= 87.892  dark(<40)=161,491  shade-band(40-89)=223,504
main-run-b             mean= 87.785  dark(<40)=161,489  shade-band(40-89)=223,603
```

**Same-build noise = 0.107 mean.** The Y=200 probe's 13.3 is **124x** that, so AC4's 10x bar is
comfortably reachable — but it must be re-measured, not inherited.

**The obligation this recipe cannot yet discharge:** the AC5 guard does not exist at authoring
time. The dev must produce its RED — re-apply the shipped `aurora_core()` aim, observe the named
test go red, restore, observe green — as a mutation row, and paste both outputs.

## Branch and commits

Branch `10-7-the-sun-lights-the-valley`, cut from **`main` at `47139fa`** — this story is **not
stacked**: 10.4 and its closure are merged and the tree is clean, so AC1's "this story's own commit
range" is simply `47139fa..HEAD` on the branch. Author every commit
`Völundr <jeicei75@gmail.com>`. Commit at minimum once per completed task — never one squashed
commit; a stalled run restarts from the last green. Review-gated: **no push, no PR** until Wolf
says so.

## References

- `_bmad-output/implementation-artifacts/10-7-signoff/README.md` — the six frames, the instruments,
  and the falsified candidate kept deliberately
- `_bmad-output/implementation-artifacts/deferred-work.md` § "Found while closing 10.4: THE SUN IS
  UNDER THE MAP (2026-09-03)"
- `_bmad-output/implementation-artifacts/10-4-the-trees-look-right-the-pilot.md` — the bench-then-
  judge-then-land shape this story reuses; `10-4-signoff/task-6-vehicle-runbook.md` for the card
- `bevy_light-0.19.0/src/directional_light.rs:25` — the forward-direction semantics
- `crates/gui/src/atmosphere.rs:27-43, 67-71, 209-211, 361-368`;
  `crates/gui/src/ingest.rs:851-862`; `crates/gui/src/capture.rs:586, 1250-1253, 1348`
- `_bmad-output/planning-artifacts/epics.md` § Epic 10, § Story 10.7 (execution order
  amended 2026-09-03), § UX-DR22
- `CLAUDE.md`, `docs/technical-preferences.md`

## Change Log

| Date | Change |
|---|---|
| 2026-09-03 | Story created out of 10.4's vehicle sitting, on Wolf's instruction ("write sun story so we don't forget it"). Evidence complete and measured; rulings and context pass outstanding. |
| 2026-09-03 | Context pass. Five premises re-verified on `47139fa`: the defect is a DIRECTION not a position (Bevy ignores a directional light's translation), the bench carries the same wrong sun and is pinned to it, the existing direction test is self-referential, and `NEAR_WHITE_AREA_CEILING` is already breached with `gui --headless --capture` already exiting 101 on `main`. Wolf's three rulings recorded: bench the elevation, decouple sun from aurora, re-tune nothing else. ACs firmed 6 → 9; tasks, dev notes and an executed verification recipe added. `baseline_commit` corrected `e930d07` → `47139fa` (it was 5 commits stale, and AC1 grades the diff from it). Status → ready-for-dev. |
| 2026-09-03 | Task 1 complete: separated the directional-light travel vector from the aurora, preserved the shipped direction, locked client and bench sun constants together, and recorded three KILLED mutations. |
| 2026-09-03 | Task 2 complete: captured the shipped control and +8.62°, +17.66°, and +25.87° bench candidates through an import-only elevation driver; each candidate's figures differ from the control. Pending Wolf's Task 3 ruling. |
| 2026-09-03 | Run A (Tasks 1-2) delegated to Codex and KILLED mid-self-gate; commit cadence preserved all four commits. Orchestrator verified independently: gate GREEN 9/9, control re-rendered pixel-identical (0/518,400), candidates separated by 199,830-253,437 px at d>=4. Found that `lumstats.py` and `pixel_diff.py` silently misread RGBA PNGs (hardcoded `bpp=3`), which is why AC4's instrument must gain a colour-type assertion in Task 6. |
| 2026-09-03 | **Task 3: Wolf ruled `+17.66°`** against the shipped `−6.42°` control and the `+8.62°` / `+25.87°` alternatives, on the side-by-side comparison of the four bench frames. UX-DR22 opening half signed; the closing half still needs his eye on the vehicle. |

## Dev Agent Record

### Agent Model Used

GPT-5.6 (Codex)

### Debug Log References

- Task 1 RED: `cargo test --offline -p gui the_aurora_curtain_hugs_the_horizon_beyond_the_world`
  initially failed with `unresolved import super::sun_direction`.
- Direction equivalence: old `aurora_light_transform().forward()` = `(0.760800, 0.111784,
  0.639287)`; new `sun_direction()` = `(0.760799, 0.111783, 0.639288)`; maximum component delta
  `0.00000074`.
- Mutations: KILLED — `sun direction returns to the aurora-to-camp aim` by
  `the_aurora_curtain_hugs_the_horizon_beyond_the_world`; KILLED — `bench sun elevation diverges
  from the client` by `bench_literals_match_the_client_palette_lights_and_boot_camera`; KILLED —
  `bench sun travels downward at the shipped elevation` by
  `ValleyFramingTests.test_sun_is_aimed_the_way_the_client_aims_it`.
- Full `scripts/gate.sh`: GREEN (fmt, clippy, full cargo test, dependency probes, metrics, bench
  tests, and mutation-table audit all passed).
- Task 2 range checks (all Blender 5.2.1, `exposed_cells=44984`):
  - `control-shipped-minus6.42.png`: `non_sky_fraction=0.686736 distinct_colors=59190 terrain_luma=105.853`.
  - `candidate-plus8.62.png`: `non_sky_fraction=0.687060 distinct_colors=85727 terrain_luma=119.546`.
  - `candidate-plus17.66.png`: `non_sky_fraction=0.687143 distinct_colors=90237 terrain_luma=132.927`.
  - `candidate-plus25.87.png`: `non_sky_fraction=0.687182 distinct_colors=89906 terrain_luma=143.913`.

### Completion Notes List

- Task 1 complete: decoupled the directional-light transform from the aurora, preserving the
  shipped direction until Wolf selects an elevation. The bench reads the matching module-scope
  constants at call time. The bench shipped-elevation assertion intentionally remains upward and
  must move with the approved elevation in Task 4.
- Re-anchored 10.1's hand-picked-sun mutation row to the new bench formula after the global
  mutation audit correctly detected its removed seam.
- Task 2 complete: exported a tick-21 snapshot and rendered the shipped control plus +8.62°,
  +17.66°, and +25.87° candidates through an import-only driver. Each candidate's range-check
  figures differs from the control's; Wolf's Task 3 decision remains outstanding.

**Orchestrator (Claude) independent verification of Run A, 2026-09-03.** Codex's exit was not
trusted; every claim below was re-run here.

- **The delegated run was KILLED by the devpod execution layer mid-self-gate** (empty last-message
  file, log truncated inside a `codex review` diff). The commit-cadence floor paid for itself for
  the third time on this project: four commits had already landed, so nothing was lost. The two
  `401` hits in the log are false positives — the handoff text echoed back, and a line number.
- **Full `scripts/gate.sh` re-run by the orchestrator: GREEN, 9/9, no skips.**
- **The control artifact was re-rendered independently and is PIXEL-IDENTICAL** to the committed
  one: `0` of 518,400 pixels differ, and the `range-check:` line reproduces verbatim —
  `range-check: blender=5.2.1 exposed_cells=44984 non_sky_fraction=0.686736 distinct_colors=59190
  terrain_luma=105.853 floors(non_sky_fraction=0.020000, distinct_colors=32, terrain_luma=20.000)`.
  AC2's "one treatment photographed twice" risk is closed by measurement, not by inspection.
- **Candidate separation against a zero-pixel same-build floor** (control vs candidate, `d>=4`):
  +8.62° = 199,830 px, +17.66° = 232,431 px, +25.87° = 253,437 px. Monotonic, and enormous
  against the 0-pixel reproduction floor above.
- **Both bench figures are monotonic in elevation**, recomputed here from the saved PNGs rather
  than taken from the run: `terrain_luma` 105.853 -> 119.546 -> 132.927 -> 143.913, and whole-frame
  Rec.601 mean 75.435 -> 84.503 -> 93.374 -> 100.564.

**INSTRUMENT DEFECT FOUND, and it lies rather than dying — `lumstats.py` (AC4's instrument) and
`pixel_diff.py` both hardcode `bpp=3` and never read the PNG's colour type.** The bench writes
**RGBA** (colour type 6); client captures are **RGB** (colour type 2). Run on the bench frames,
`lumstats.py` misparses every scanline and returns plausible, wrong, NON-MONOTONIC figures
(mean 68.266 -> 69.065 -> 67.882 -> 66.981) for frames whose true ladder is cleanly monotonic.
It caught the orchestrator first: the two instruments appeared to disagree about the direction of
the effect, and the disagreement was entirely this bug. The story's creation-time REDs proved
`lumstats.py` dies on a truncated frame and reports an all-black frame honestly; a **third**
direction — a frame whose colour type it does not support — was never tested, and there it is
believed. Its own job (AC4, client captures) is RGB and is unaffected today, but nothing in the
file enforces that. **Task 6 must make it assert its colour type before it is used as evidence.**

### File List

- `crates/gui/src/atmosphere.rs` — sun direction/transform and independent shipped-vector test.
- `crates/gui/src/ingest.rs` — use the decoupled sun transform.
- `crates/gui/tests/bench_contract.rs` — lockstep sun-value anchors.
- `scripts/bench/valley_bench.py` — matching module-scope sun constants and formula.
- `scripts/tests/test_valley_bench.py` — decoupled shipped-elevation oracle.
- `_bmad-output/implementation-artifacts/mutations/10-7-the-sun-lights-the-valley.sh` — Task 1
  mutation evidence.
- `_bmad-output/implementation-artifacts/mutations/10-1-the-headless-bench.sh` — re-anchored
  stale sun-formula row.
- `_bmad-output/implementation-artifacts/10-7-signoff/world-snapshot.json` — Task 2 exported
  tick-21 bench input.
- `_bmad-output/implementation-artifacts/10-7-signoff/sun_elevation_candidate.py` — import-only
  candidate elevation driver.
- `_bmad-output/implementation-artifacts/10-7-signoff/control-shipped-minus6.42.png` — control.
- `_bmad-output/implementation-artifacts/10-7-signoff/candidate-plus8.62.png` — candidate.
- `_bmad-output/implementation-artifacts/10-7-signoff/candidate-plus17.66.png` — candidate.
- `_bmad-output/implementation-artifacts/10-7-signoff/candidate-plus25.87.png` — candidate.
- `_bmad-output/implementation-artifacts/10-7-the-sun-lights-the-valley.md` — Task 1–2 record.
