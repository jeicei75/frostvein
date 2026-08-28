---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: 815cd6cd913f9dcc7fa1948c06a4d9ac008c68c4
---

# Story 9.4: Trees — Fewer, and Distinct from the Ground

Status: in-progress

## Story

As the boss,
I want fewer trees, coloured apart from the terrain,
so that the valley reads as a landscape with trees in it rather than a confusion of
same-coloured blocks.

## The epic's blast-radius warning is OVERSTATED — read this before planning the work

Epic 9 says the density change means *"every seeded world changes"*. **It does not.** Verified
against source at story creation:

```
crates/sim-core/src/lib.rs:1088   let mut rng      = ChaCha8Rng::seed_from_u64(seed ^ STREAM_WORLDGEN);
crates/sim-core/src/lib.rs:1093   let mut tree_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_TREES);
crates/sim-core/src/lib.rs:1094   worldgen::place_trees(dims, &heights, &mut tiles, camp_origin, &mut tree_rng);
```

Trees draw from a **dedicated stream**. Terrain heights, the camp origin and every spawn position
come from `STREAM_WORLDGEN` and **do not move** when the tree knob changes. `spawn_positions_for_seed_42_are_pinned`
is therefore not at risk, and neither is the height-span test.

**No mutation row anchors on the density literal either.** The only worldgen-anchored rows are in
`mutations/5-1-the-world-grows-things-that-glow.sh`: one targets the camp-clearing block (`:12`),
one the tree stream seed in `lib.rs` (`:25`). Both survive a `0..12` change. `audit-mutations.py`
was run clean at creation.

**What DOES change is tile contents only** — fewer `TreeTrunk` / `TreeFoliage` cells. That is a
real blast radius and AC7 walks it, but it is a much smaller one than the epic implies. Do not
budget for a full re-pin of the seeded world.

## The numbers, measured at story creation 2026-08-28

Counted on the real generated world (`World::generate`), trunk columns as trees:

| roll | trees | trunk cells | foliage cells | density of eligible columns |
| --- | ---: | ---: | ---: | ---: |
| **`0..12` — today** | **704** | 2,816 | 16,786 | 4.434 % |
| `0..20` | 531 | — | 12,671 | — |
| `0..30` | 400 | — | 9,558 | — |
| **`0..48` — the target** | **265** | — | 6,329 | — |

Other seeds at today's knob: 696 (seed 42), 709 (seed 7451) — so ~700 is the world, not the seed.

**The knob is damped by the spacing exclusion, and that is the trap.** Cutting the roll 2.5×
(12 → 30) removes only 43 % of trees, not 60 %: the 2-cell Chebyshev exclusion
[worldgen.rs:185-187] is already rejecting roughly half the successful rolls, so it absorbs part
of every reduction. **Do not assume proportionality — measure.** The four rows above are real
measurements taken by editing the literal, generating, counting and reverting.

### The hue defect, measured

`channel_distance` is Euclidean over RGB, the same helper the mark floor uses [appearance.rs:497].

| pair | distance |
| --- | ---: |
| foliage `(55,73,84)` ↔ **stone** `(60,70,92)` | **9.9** |
| foliage `(55,73,84)` ↔ soil `(56,52,62)` | 30.4 |
| (for scale) the shipped mark floor `MIN_MARK_SEPARATION` | 40.0 |

Foliage sits **9.9** from stone. The marks are held to 40. That is the whole case for the hue AC.

## Wolf's rulings, taken at story creation 2026-08-28

| # | Question | Ruling |
| --- | --- | --- |
| W1 | The epic wants a target band; what is it? | **265 trees, roll `0..48`, band 230–300.** The aggressive end of the measured curve — the strongest legibility gain. |
| W2 | The 2026-08-28 ruling says "brown/green", but `rgb[2] >= rgb[0]` ("night terrain stays blueward of red", [appearance.rs:319-322]) forbids brown | **Green, and keep the invariant.** Foliage separates on GREEN; the cold-boot value discipline 5.4 converged is not reopened. Brown is not taken, and is not to be reached for quietly. |

**W2 is a scope boundary, not a preference.** A brown foliage requires red above blue, which
breaks the invariant, its assertion inside `appearance_tables_pin_the_cold_boot_palette`, and the
palette pin 5.4 measured. If green cannot reach the separation this story asks for, **stop and
report the numbers** — do not relax the invariant to get there. Epic 10's tree pilot owns the
warmer authored look.

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier, not `--fast`) is green on a cold rebuild, and the diff is
   confined to this story's own commit range from `baseline_commit`. *(Not `main..HEAD` — this
   story is stacked on 9.1 and that range is wrong by default.)*

### Density

2. The default world's tree count — distinct trunk columns in `World::generate(DEFAULT_SEED,
   Dims::DEFAULT)` — lands in **230–300**, against today's measured 704.
3. The count is **deterministic**: the same seed yields the same count across runs, and a
   headless test asserts the count for a **named seed** rather than only the default, so a future
   worldgen edit that perturbs the tree stream is caught.

### Hue

4. `material_color(Material::TreeFoliage)` sits at least **40.0** (`MIN_MARK_SEPARATION`, the
   shipped floor) from **both** stone `(60,70,92)` and soil `(56,52,62)`, measured with the
   existing `channel_distance` helper. *Mechanism named deliberately: the 40.0 floor is an
   existing shipped constant and reusing it is the point — a second, weaker floor invented for
   foliage would be the thing this AC exists to prevent.*
5. The foliage colour keeps `rgb[2] >= rgb[0]` (W2), and `foliage_snow_color()` `(156,170,196)` is
   **unchanged** and still passes its "a snow-laden crown must be visibly brighter than bare
   foliage" guard [appearance.rs:328-334] — the exposed-crown tests pass untouched.
6. `Material::TreeTrunk` `(43,47,58)`, stone, soil, ice, snow and `snow_cap_color()` are
   **unchanged**. Only foliage moves.

### The interaction — measured, not assumed

7. **9.1 and 9.4 push valley-floor luminance in opposite directions and the story must measure the
   result.** Fewer dark tree skirts raise the floor while 9.1's shadows lower it. At the boot vista
   the existing `--capture` band (70–180, [capture.rs:430,435]) must still hold **and** 9.1's new
   blown-pool ceiling (`BLOWN_POOL_FRACTION_CEILING`, 0.6651 % at threshold 200,
   [capture.rs:442]) must still be judged rather than skipped. *A capture that reports
   `assertions skipped` has judged nothing.*
8. **Blast radius walked, not discovered.** `python3 scripts/audit-mutations.py` runs clean over
   every table, and any test or capture recipe that depends on tree tiles is re-checked and named
   in the Dev Agent Record — found by search, not by waiting for a red gate.

### Evidence

9. A sabotage table exists at `mutations/9-4-trees-fewer-and-distinct-from-the-ground.sh`, every
   row KILLED, zero APPLY-FAILED, re-run **after the last refactor**. Rows at minimum: the density
   literal reverted to `0..12` → the count test goes RED; the foliage colour reverted to
   `(55,73,84)` → the separation test goes RED; the separation floor lowered so the old colour
   would pass → the test goes RED.

### Wolf's eye — the closing half, which no agent can check

10. On the vehicle (UX-DR22 closing, UX-DR17/18; merged into Epic 9's shared sitting), Wolf judges
    that the valley reads as **a landscape with trees in it** — trees tellable from the ground at
    a glance.

## Tasks / Subtasks

- [x] **Task 1 — Measure before you change (AC: 2)**
  - [x] Reproduce the creation figure first: count distinct trunk columns in
        `World::generate(DEFAULT_SEED, Dims::DEFAULT)` and confirm **704**. If it is not 704, stop
        — something moved and every number in this story needs re-taking.
  - [x] Only then change the roll. The target is `0..48` → **265**, band 230–300.

- [x] **Task 2 — The density knob (AC: 2, 3)**
  - [x] `worldgen.rs:184` `rng.random_range(0..12)` → `0..48`. **Nothing else in `place_trees`
        changes** — not the 2-cell exclusion, not the camp clearing, not the trunk height range,
        not the crown shape.
  - [x] A test in **`crates/sim-core/tests/worldgen.rs`** (beside the existing
        `pines_use_both_tree_materials_and_leave_the_camp_clear`, which is the only test that
        filter matches today) asserts the tree count for a **named seed** is inside the band, and a
        second run of the same seed gives the identical count. Count **trunk columns**, not trunk
        cells: trunk height is `4..=6`, so cells move with height and are the wrong oracle.
  - [x] **Hardcoded constant is fine** (technical-preferences). No density config, no builder.

- [ ] **Task 3 — The foliage hue (AC: 4, 5, 6)**
  - [ ] `appearance.rs:215` `Material::TreeFoliage => Color::srgb_u8(55, 73, 84)` → a green that
        clears stone **and** soil by ≥ 40.0 while keeping `rgb[2] >= rgb[0]`. `(44,100,58)` was
        checked at creation and clears stone by **48.1** and soil by **49.6** — use it or beat it,
        but state the measured distances you actually got.
  - [ ] Update the pin at `appearance.rs:311` in the same commit — it asserts the exact literal and
        will go red otherwise. That is the pin working, not a defect.
  - [ ] Extend `appearance_tables_pin_the_cold_boot_palette` (or add a sibling test) to assert the
        foliage↔stone and foliage↔soil distances against `MIN_MARK_SEPARATION`, so a future palette
        edit cannot silently re-merge them. **Reuse `channel_distance` and `MIN_MARK_SEPARATION`;
        do not invent a second helper or a second floor.**
  - [ ] Leave the doc comment on `material_color` naming the measured before/after distance, in the
        style of the existing table comments.

- [ ] **Task 4 — Walk the blast radius (AC: 7, 8)**
  - [ ] Search for anything depending on tree tiles or foliage colour: `rg -n "TreeFoliage|TreeTrunk"
        crates/` and the capture recipes. **`crates/tui/src/palette.rs` also carries tree colours** —
        decide and state whether the TUI follows. The gui↔tui cross-check at
        [appearance.rs:441-456] covers **marks only**, not terrain, so the TUI is not forced by a
        test; say what you chose and why in one sentence.
  - [ ] Run `python3 scripts/audit-mutations.py`. Expect clean — no row anchors on the density
        literal (verified at creation) — and if a row DOES fail to apply, that is a finding, not
        noise.
  - [ ] Name in the Dev Agent Record every test you found that touches trees, and its outcome.

- [ ] **Task 5 — The interaction with 9.1 (AC: 7)**
  - [ ] This story is **stacked on 9.1**, so 9.1's blown-pool ceiling is live in your tree. Fewer
        dark trees near the campfire can only make the near-white pool **larger**. If a boot-vista
        capture now exits **101** on `BLOWN_POOL_FRACTION_CEILING`, that is this story's most
        important finding — **report it with the number, do not raise the ceiling.** The ceiling is
        9.1's calibrated bar and Wolf ruled on 2026-08-28 that it stays hard.
  - [ ] Likewise the 70–180 band: fewer dark skirts push the ground median **up** from today's
        123.4. Record the new median. A breach of 180 is a finding, not something to tune away.
  - [ ] Both numbers are headless-measurable from the committed-PNG path only if a frame exists;
        the live capture is vehicle-bound. State plainly which you measured and which you did not.

- [ ] **Task 6 — The sabotage table (AC: 9)**
  - [ ] Commit first, then mutate — never `git checkout --` over an uncommitted fix.
  - [ ] Rows: (a) density literal back to `0..12` → count test RED; (b) foliage back to
        `(55,73,84)` → separation test RED; (c) the separation floor lowered (e.g. to 5.0) so the
        old colour would pass → the test RED.
  - [ ] **Check WHICH assertion kills each row**, not merely that the row says KILLED. A row killed
        by an earlier assertion than the one you strengthened is a row that proves nothing — this
        exact defect was found in 9.1's review patch pass, where an equality pin sat ahead of the
        behavioural clause and hid it.
  - [ ] `scripts/mutate.sh` is **not concurrency-safe** — run it alone, read the exit code before
        any pipe.

- [ ] **Task 7 — VEHICLE-BOUND: Wolf's eye (AC: 10)**
  - [ ] Write the session card for Epic 9's shared sitting **before** the session, ordered so
        nothing erases its own evidence. Use 9.1's corrected card
        (`9-1-signoff/task-6-vehicle-runbook.md`) as the worked example — in particular its §0,
        which states expected-failure traps up front.
  - [ ] Wolf's AC10 judgement. **A dev agent cannot check this box.**

- [ ] **Task 8 — The gate and the record (AC: 1, 8)**
  - [ ] `scripts/gate.sh` full tier on a cold rebuild.
  - [ ] Verify AC6's untouched-colours claim by diff over **this story's own commit range**.
  - [ ] Update `docs/tech-art-guidelines.md` if it names the foliage colour or tree density — it
        carries the light/material table prose and went stale twice already (corrected 2026-08-28
        for both the campfire lumens and the ambient/directional pair).

## Dev Notes

### Scope guardrails — do NOT build these here

- **No brown foliage** (W2). Green, keeping `rgb[2] >= rgb[0]`. If green cannot reach 40.0, stop
  and report.
- **No change to the 2-cell spacing exclusion, camp clearing, trunk height range or crown shape.**
  The density knob is the roll, and only the roll.
- **No change to trunk, stone, soil, ice, snow, snow-cap or crown colours** (AC6).
- **No new mark or hover colour work** — 9.2/9.3's headless halves already shipped in 8.2.
- **Do not raise 9.1's blown-pool ceiling or widen the 70–180 band** to accommodate more light.
- **No new dependency.**

### What already exists (build on it, do not re-derive)

- `place_trees` [worldgen.rs:167-232] — the roll, the exclusion, the camp clearing, the crown.
- `STREAM_TREES` [lib.rs:30, :1093] — trees have their own RNG stream; terrain does not move.
- `channel_distance` and `MIN_MARK_SEPARATION = 40.0` [appearance.rs:497, :511] — the shipped
  separation helper and floor. Reuse both.
- `appearance_tables_pin_the_cold_boot_palette` [appearance.rs:285] — the terrain pins and the
  blueward-of-red invariant live here.
- 9.1's blown-pool instrument [capture.rs:442, :1128] — live in this tree because 9.4 stacks on 9.1.

### Key decisions & traps

- **The count oracle is trunk COLUMNS, not trunk cells.** Trunk height is `rng.random_range(4..=6)`,
  so cells vary with height even at a fixed tree count.
- **The spacing exclusion damps the knob.** 12→30 is a 2.5× cut in the roll but only a 43 % cut in
  trees. Measure, do not extrapolate.
- **`Dims::DEFAULT` is 128×128×32**, 15,876 eligible columns after the 1-tile border.
- **A checkbox is worth only what its verification is worth.** 6.1 had four subtasks ticked without
  being delivered.

### Previous story intelligence (deltas that change THIS story)

- **This story is STACKED on 9.1**, whose branch is unmerged. Cut from
  `9-1-the-frame-stops-blowing-out`, and answer every "unchanged" AC with the story's **own**
  commit range from `baseline_commit` — `main..HEAD` is wrong by default and has shipped as a
  defect ten times.
- **9.1's review found that a strengthened assertion can be hidden by an earlier one** and that
  the mutation row still reports KILLED. Task 6 carries the check that catches it.
- **9.1's shadows did NOT close the blow-out** (Wolf, on the vehicle, 2026-08-28). The campfire
  pool is still over its ceiling, so a boot-vista capture in this tree may already exit 101 before
  9.4 changes anything. **Establish that baseline before attributing any capture failure to trees.**

### Project Structure (files to touch)

| file | NEW/UPDATE | what |
| --- | --- | --- |
| `crates/sim-core/src/worldgen.rs` | UPDATE | the density roll, line 184 only |
| `crates/sim-core/tests/worldgen.rs` | UPDATE | the seeded tree-count test, beside the existing `pines_use_both_tree_materials_and_leave_the_camp_clear` |
| `crates/gui/src/appearance.rs` | UPDATE | foliage colour, its pin, the separation assertions |
| `mutations/9-4-trees-...sh` | NEW | the sabotage table |
| `crates/tui/src/palette.rs` | DECIDE | state whether the TUI follows; not forced by any test |
| `docs/tech-art-guidelines.md` | CHECK | correct it if it names foliage or density |

### Verification

**Executed at story creation, 2026-08-28** — the full gate on `815cd6c`, clean tree, run not
claimed:

```
frostvein gate
  cargo fmt --check           ok
  cargo clippy -D warnings    ok
  cargo test                  ok
  tui / client-core / gui have no sim-core edge   ok
  metrics ledger tests        ok
  mutation tables still apply ok
GATE GREEN
```

**Also executed at creation** — the measurements this story rests on. Tree counts were taken by
editing the roll, generating, counting trunk columns and reverting; the working tree was verified
clean afterwards and the literal confirmed back at `0..12`:

```
roll 0..12 -> 704 trees, 16,786 foliage cells      (seed 42: 696, seed 7451: 709)
roll 0..20 -> 531 trees, 12,671 foliage cells
roll 0..30 -> 400 trees,  9,558 foliage cells
roll 0..48 -> 265 trees,  6,329 foliage cells      <- the target
```

Colour distances were computed with the same Euclidean form as `channel_distance`:
foliage↔stone **9.9**, foliage↔soil **30.4**, and the candidate green `(44,100,58)` ↔ stone
**48.1**, ↔ soil **49.6**.

**Run these, and report the named observation beside each — not exit 0:**

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# 1. The count and the colour (AC 2, 3, 4, 5, 6)
# Verified at creation: this filter matches exactly ONE test today
# (pines_use_both_tree_materials_and_leave_the_camp_clear). Yours must appear beside it.
cargo test --offline -p sim-core tree
cargo test --offline -p gui appearance

# 2. The interaction (AC 7) — name which assertions ran, and whether any were SKIPPED
cargo test --offline -p gui capture

# 3. Sabotage (AC 9) — commit first; run alone; read the exit code before any pipe
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/9-4-trees-fewer-and-distinct-from-the-ground.sh
python3 scripts/audit-mutations.py

# 4. The gate (AC 1) — full tier
scripts/gate.sh
```

**Not executable in a devpod — no devpod can open a window.** AC10 is inherited by Epic 9's shared
vehicle sitting; the live `--capture` numbers for AC7 belong there too.

### Branch and commits

Branch `9-4-trees-fewer-and-distinct-from-the-ground`, cut from
`9-1-the-frame-stops-blowing-out` (this story is stacked). Author every commit
`Völundr <jeicei75@gmail.com>`. Commit at minimum once per completed task. Review-gated: **no
push, no PR** until Wolf says so.

### References

- Epic + ACs: `_bmad-output/planning-artifacts/epics.md:1271-1318` (Epic 9, Story 9.4)
- Density: `crates/sim-core/src/worldgen.rs:167-232`; stream isolation `crates/sim-core/src/lib.rs:27-30, :1088-1094`
- Colour: `crates/gui/src/appearance.rs:205-233` (table), `:285` (pins), `:497,:511` (helper + floor)
- The interaction: `crates/gui/src/capture.rs:430,435` (band), `:442,:1128` (9.1's ceiling)
- Standing art rule: memory `art-gates-visual-judgement`

## Dev Agent Record

### Agent Model Used

gpt-5.6-terra, effort high

### Debug Log References

- Task 1 green: `cargo test --offline -p sim-core default_world_tree_column_count_is_measured_before_density_change -- --exact` ran `default_world_tree_column_count_is_measured_before_density_change`; its independent trunk-column oracle measured exactly `704` for `DEFAULT_SEED`.
- Task 2 RED: before changing the roll, `cargo test --offline -p sim-core tree_density_for_seed_42_is_deterministic_and_in_target_band -- --exact` failed its target-band assertion: `seed 42 generated 696 distinct trunk columns, outside the 230..=300 target band`.
- Task 2 green: after only changing `rng.random_range(0..12)` to `0..48`, `cargo test --offline -p sim-core tree -- --nocapture` ran and passed `tree_density_for_seed_42_is_deterministic_and_in_target_band` (including its second-generation equality assertion) and `pines_use_both_tree_materials_and_leave_the_camp_clear`; the other tree-named regression assertions also passed. A temporary upper-bound probe measured 265 default-seed trunk columns (`seed 4026891802 generated 265 distinct trunk columns, outside the 230..=264 target band`), then was restored before this commit.

### Completion Notes List

- Task 1: added the pre-change trunk-column measurement checkpoint. It is intentionally temporary evidence for the existing `0..12` density and will be replaced by Task 2's 230–300 deterministic named-seed guard before changing the roll.
- Task 2: changed only the tree placement roll to `0..48`; default seed now measures 265 trunk columns and named seed 42 is deterministically inside 230–300.

### File List

- crates/sim-core/tests/worldgen.rs
- crates/sim-core/src/worldgen.rs
- _bmad-output/implementation-artifacts/9-4-trees-fewer-and-distinct-from-the-ground.md

## Change Log

| date | change |
| --- | --- |
| 2026-08-28 | Task 2 complete: changed only the tree roll from `0..12` to `0..48`; default-world density measures 265 distinct trunk columns, and a seed-42 two-run test pins the 230–300 band and determinism. |
| 2026-08-28 | Task 1 complete: independently counted distinct `TreeTrunk` columns in the default world and reproduced the required pre-change measurement of 704. |
| 2026-08-28 | Story created. Baseline `815cd6c`, full gate green at creation (run, not claimed). **The epic's blast-radius paragraph was falsified against source**: trees draw from a dedicated `STREAM_TREES`, so terrain heights, camp origin and spawn positions do NOT move, and no mutation row anchors on the density literal — the radius is tile contents only. **The density curve was measured rather than estimated** (704 / 531 / 400 / 265 at rolls 12/20/30/48) and revealed that the 2-cell spacing exclusion damps the knob, so a 2.5× cut in the roll removes only 43 % of trees. Two rulings taken from Wolf: W1 the target band 230–300 at roll `0..48`; W2 **green, not brown** — the epic's "brown/green" collides with the shipped `rgb[2] >= rgb[0]` invariant at `appearance.rs:319-322`, which brown cannot satisfy, so the invariant stands and green carries the separation. The hue defect was measured: foliage sits **9.9** from stone against a shipped mark floor of 40.0. |
