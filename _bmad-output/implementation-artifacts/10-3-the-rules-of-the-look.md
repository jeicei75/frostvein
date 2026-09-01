---
baseline_commit: 0b8b6735f04b282e2d75b82e426346be49590082
---

# Story 10.3: The Rules of the Look

Status: done

**GATED ON STORY 10.6.** The grid-scale clause is the reason this story exists, and 10.6 measures
what is actually servable. Do not start this story until 10.6's decision is recorded. Ruled by
Wolf, 2026-08-31, after the first draft had the scale picked from three unmeasured options.

## Story

As the boss,
I want the tech-art guidelines to grow two contracts — one for procedural content, one for
authored assets,
so that output from any tool, hand, script or MCP session can be checked against the same bar.

## Premises re-verified at creation — 2026-08-31

The epic orders its planning-time premises re-verified at story creation. All four checked
against source; **two of them are wrong as written in `epics.md` and are corrected here.**
(Corrected by the 2026-09-01 code review: this block lists **five** items, not four, and four of
its `file:line` citations had gone stale — they were written 39 commits before the branch base.
Every premise still HOLDS in substance; only the anchors moved.)

1. **"Grid scale is unset" — HOLDS, but the reference sheet already answers half of it.**
   `epics.md:1510` names grid scale as blocking 10.4/10.5 and predates the 10.2 spike. What the
   sheet fixes and what it cannot is set out in Dev Notes; **the servable half comes from story
   10.6, which runs before this one.**
2. **"The client's cell is a unit cube" — HOLDS.** `Cuboid::default()` at
   [crates/gui/src/project.rs:235] is 1×1×1; the dwarf is scaled on it by
   `Vec3::splat(appearance.scale)` [crates/gui/src/project.rs:1361], with `scale: 0.65` at
   [crates/gui/src/appearance.rs:278]. (The literal `Vec3::splat(0.65)` this premise originally
   cited exists nowhere in the repo; the value reaches the cube through the appearance table.)
3. **"Worldgen grows trees 4–6 cells" — HOLDS.** `rng.random_range(4..=6)` at
   [crates/sim-core/src/worldgen.rs:193].
4. **The 1.20 m dwarf anchor is NOT ratified anywhere.** It appears only in
   [10-2-signoff/ASSET_NOTES.md:36] as that story's working assumption. `rg` over `docs/` and the
   planning artifacts finds no other record. This story ratifies it or replaces it; it must not
   inherit it silently.
5. **The stale-artifact RED was executed at creation, not assumed** — see Verification.

## Acceptance Criteria

1. `scripts/gate.sh` passes with the story's work in place.
2. `docs/tech-art-guidelines.md` gains a **Procedural-content contract** section stating, as
   checkable clauses, the rules its existing sections imply. Each clause either cites the test or
   instrument that already enforces it (`file:line`) or is marked **eye-only**.
3. The **grid-scale decision is taken and recorded** in that document: one metres-per-cell, one
   project voxel size, and the rule for per-asset voxel multiples — with the arithmetic shown and
   the three divergent implied values named as what it resolves.
4. `docs/tech-art-guidelines.md` gains an **Asset contract** covering grid scale, orientation,
   origin, palette/material mapping, naming and file locations, each clause concrete enough to
   check against one file, and each marked mechanically-checkable or eye-only. The section records
   that it discharges the PRD's asset-contract obligation, citing [prd.md:150].
5. The contract states that **an asset's identity is its name AND its published figures**, citing
   the measured case: `10-2-signoff/tree.glb` carries the mesh and node name
   `SM_VoxelPine_Tree02` — the deliverable's name — with different geometry.
6. `scripts/bench/check_asset.py` takes one or more `.glb` paths, prints one `FIGURES` line per
   file, and exits non-zero naming the first violated clause. Stdlib only; no Blender, no numpy.
7. The checker **passes all four** `10-2-signoff/export/SM_VoxelPine_Tree0{1,2,3,4}.glb` and
   **fails** `10-2-signoff/tree.glb`, naming the origin-centring clause.
8. `scripts/tests/test_check_asset.py` covers the checker, including a case that fails when the
   origin-centring assertion is removed, and `_bmad-output/implementation-artifacts/mutations/10-3-the-rules-of-the-look.sh`
   carries at least three rows that the mutation run kills.

## Tasks / Subtasks

- [x] **Task 1 — Settle the grid scale** (AC: 3)
  - [x] **Read story 10.6's recorded decision first** — it runs before this one and measures what
        is servable. If 10.6 has not landed, stop: this task cannot be done from the sheet alone.
  - [x] Confirm the sheet's derivation in Dev Notes against the cited `file:line`, then write down
        metres-per-cell, project voxel size, and the per-asset integer-multiple rule.
  - [x] Record the per-class resolutions 10.6 reports (terrain / trees / dwarves) rather than
        collapsing them to one project number.
  - [x] Record `appearance.rs`'s `scale: 0.65 → 0.75` in the contract as **owed by 10.5**, with
        the arithmetic. **Do not make the change here.**

- [x] **Task 2 — Write the procedural-content contract** (AC: 2)
  - [x] Extract the implied rules from the existing sections of `docs/tech-art-guidelines.md`.
        The candidates, with their enforcement, are listed in Dev Notes; verify each citation
        before writing it down.
  - [x] Mark each clause with its enforcement or with **eye-only**. A clause with neither is not
        a contract clause — cut it or make enforcing it an owed item.

- [x] **Task 3 — Write the asset contract** (AC: 4, 5)
  - [x] Generalise the standing rules out of [10-2-signoff/ASSET_NOTES.md] — that file is one
        asset's brief; the contract is the family of rules it is an instance of.
  - [x] Decide and state file locations: where a generator lives, where a runtime glTF lives,
        where signoff artifacts live. Naming a path is enough; **do not create `assets/`**.
  - [x] Write the identity clause (AC5) with `tree.glb`'s measured figures as the case.

- [x] **Task 4 — Build the checker** (AC: 6, 7)
  - [x] `scripts/bench/check_asset.py`, stdlib only. Lift the GLB reader from
        [10-2-signoff/voxel_pine.py:437-530] (`load_glb`, `read_accessor`, `decode_png_rgb`,
        `palette_from_glb`) rather than re-deriving it, and carry a `# NOTE:` saying why it is
        duplicated: that file imports `bpy` at module scope, so it is not importable outside
        Blender, and it is 10.2's frozen deliverable.
  - [x] Check only the mechanically-checkable clauses. Every clause the checker enforces must be
        one the contract states; every contract clause it cannot check stays marked eye-only.

- [x] **Task 5 — Test the instrument and the sabotage rows** (AC: 8)
  - [x] `scripts/tests/test_check_asset.py` — picked up automatically by
        [scripts/gate.sh:117] (`unittest discover -s scripts/tests`); no gate edit needed.
  - [x] The test must assert the checker **fails** on `tree.glb` and **names the clause**, not
        merely that it exits non-zero. An exit code alone does not discriminate a working checker
        from one that rejects everything.
  - [x] Mutation table rows, format per [mutations/10-1-the-headless-bench.sh]. At least: remove
        the origin-centring assertion; make the failure path exit 0; break a figure the `FIGURES`
        line reports. Run `scripts/mutate.sh` and record KILLED per row.

- [x] **Task 6 — Verification** (AC: 7)
  - [x] Execute the recipe below, RED first, and paste both outputs into the Dev Agent Record.

## Dev Notes

### Scope guardrails — do NOT

- **Do not change `appearance.rs`, `light_properties()`, or any client colour or intensity.** The
  bench's camp-blow-out finding is explicitly 10.4's call to make, not this story's
  [deferred-work.md:1269-1280].
- **Do not modify `10-2-signoff/voxel_pine.py`, the four `export/*.glb`, or `tree.glb`, and do
  not delete any of them.** Deleting committed assets is Wolf's call [deferred-work.md:1330].
  `tree.glb` is this story's RED specimen — it must stay exactly as it is.
- **Do not enable `bevy_gltf` or `file_watcher`.** That is 10.5, with its justification line
  against the feature trim.
- **Do not author, re-export or re-scale any asset.** 10.3 defines the target; 10.4 and 10.5 hit
  it. If the chosen scale invalidates an existing asset, say so in the contract — do not fix it.
- **Do not create `assets/`.** Name the path in the contract; 10.5 creates it.
- **Do not harden `spike_pine_render.py`** (issue #59) and **do not write the MCP handover
  runbook** (issue #58). Both are adjacent and both are somebody else's vehicle. Review
  specifically found that 10.3 "will not pick this up on its own" [deferred-work.md:1379] — that
  is correct and intended.

### The grid-scale decision (AC3) — what is already settled, and what 10.6 supplies

**The reference sheet fixes the target grid, and it is self-consistent.** Section A of
[10-2-signoff/reference-sheet.jpg] labels the dwarf **"12 Voxels"**; Section B labels the trees in
cells (type 1 = 4 cells at 5.32× dwarf height). At the 1.20 m dwarf that is:

**1 voxel = 0.1 m · 1 cell = 1.6 m · 16 voxels per cell · dwarf = 12 voxels = 0.75 cells**

Every figure on the sheet and in [10-2-signoff/ASSET_NOTES.md] agrees with this. **One constant
disagrees**: the client's `scale: 0.65` [crates/gui/src/appearance.rs:278], which should be 0.75.
So this was never a three-way fight over metres-per-cell — it is one stale constant, and the
correction is a one-line `appearance.rs` change **owed to 10.5, not taken here** (see the do-NOT
list). Pine type 4 is the only loose end: the sheet labels it 6 cells and 8.8× dwarf height, which
disagree with each other by ~10%; record it as the sheet's rounding, not as a contradiction.

**What the sheet cannot tell us is what we can render**, and that is exactly the number the
contract must not guess. **Story 10.6 runs before this one and supplies it**: the servable
terrain subdivision, reported per class (terrain / trees / dwarves have instance counts four
orders of magnitude apart, so they do not share one budget), swept to the wall rather than
sampled. Copy 10.6's recorded decision into the contract; do not re-derive it, and do not fall
back to the sheet's 16 if 10.6 measured less.

**Voxel size is a separate axis from cell size.** At the 1.20 m dwarf, a 0.2 m voxel makes the
dwarf 6 voxels tall, which cannot carry the beard, belt, tunic panel and lantern the sheet draws
[deferred-work.md:1376]; 0.1 m gives 12, and `dwarf.mp4` renders finer still at ~20–24. Shape to
write down: **one project voxel size, with an asset free to use an integer multiple of it,
declared per asset** — this keeps 10.2's 0.2 m pine legal as a 2× asset instead of invalidating
it.

### What already exists — build on it, don't restate it

- [docs/tech-art-guidelines.md] — 175 lines, entirely about the *client's procedural look*. There
  is no asset content in it at all. Both new sections are additions; do not rewrite what is there.
- [10-2-signoff/ASSET_NOTES.md] — the per-asset brief the standing contract generalises from:
  generation command, variants table, per-asset properties, palette table, mesh-topology
  warnings, and the `FIGURES` self-check list. Its figures are accurate (re-measured at creation).
- [10-2-signoff/voxel_pine.py] — stdlib GLB reader at lines 437–530, and `check_mesh_properties`
  at 551. Blender-bound (`import bpy`, line 38).
- [scripts/tests/test_valley_bench.py] and [mutations/10-1-the-headless-bench.sh] — the shapes to
  copy for a bench test and a sabotage row.

### Key decisions & traps

- **Presentation never becomes wire state (AD-16).** An asset is client-side presentation keyed
  off `Material` / `EntityKind`; the wire carries kind identifiers only, never RGB, radius or
  geometry. The contract must say this — it is the boundary an authored-asset pipeline is most
  likely to erode.
- **Trees are tiles, dwarves are entities (AD-16).** A dwarf asset lands on the existing
  reconciliation seam; a tree asset would have to replace per-tile cubes and is therefore
  governed by the draw-set oracle. The asset contract must state which of the two an asset class
  is, because the two have different rules.
- **The draw-set oracle is a measurement, not a constant.** Already stated in the guidelines
  ("44,984 exposed cubes … it moves whenever world content moves"). The procedural contract
  should keep that framing rather than pinning a number.
- **A glTF internal name is not an identity.** Measured: `tree.glb` and
  `export/SM_VoxelPine_Tree02.glb` both carry mesh and node name `SM_VoxelPine_Tree02`, with
  5,130 tris / 5.2 × 7.6 × 5.4 m / centreX −0.100 against 5,894 / 5.0 × 8.0 × 5.4 / centreX
  +0.000. A checker that matched on name alone would authenticate the stale file.
- **A checker that rejects everything passes an exit-code test.** AC7 and Task 5 require the
  clause name, not the exit code. This is the same defect class as 10.1's literal-scraping guard,
  which stayed green while the bench camera rolled 110°.
- **Do not let the checker duplicate the generator's job.** `voxel_pine.py` asserts its own build
  invariants at build time. `check_asset.py` asserts the *standing contract* against any GLB,
  including one that came out of a live MCP session and never ran the generator — which is the
  gap 10.2's handoff decision opened.

### Project Structure

| Path | State | Note |
|---|---|---|
| `docs/tech-art-guidelines.md` | UPDATE | Two new sections appended; existing sections untouched |
| `scripts/bench/check_asset.py` | NEW | Stdlib only |
| `scripts/tests/test_check_asset.py` | NEW | Auto-discovered by the gate |
| `_bmad-output/implementation-artifacts/mutations/10-3-the-rules-of-the-look.sh` | NEW | ≥3 rows |

Branch `10-3-the-rules-of-the-look`, off `main`. Commits authored `Völundr <jeicei75@gmail.com>`,
imperative messages. **Put `Closes #57` in the PR body** — this story is issue #57's (M2-20)
vehicle, and `Closes` does close an issue from a PR in this repo. Push and PR only on Wolf's
explicit yes.

### References

- [_bmad-output/planning-artifacts/epics.md:1496-1520] — story 10.3 and its blocking claim
- [_bmad-output/planning-artifacts/epics.md:1406-1435] — the inherited eye-checks from 9.2/9.3
- [_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:121] — AD-16
- [_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/prd.md:150] — the PRD obligation this discharges
- [_bmad-output/planning-artifacts/prds/prd-frostvein-2026-08-09/addendum.md:68] — Blender → glTF supersession
- [_bmad-output/implementation-artifacts/deferred-work.md:1301-1387] — 10.2's review findings and its three owed items

## Verification

**The instrument is `scripts/bench/check_asset.py`** — the thing a human runs to see whether a
file meets the contract. It does not exist yet; the recipe below states the exact commands and
the exact observations the dev agent must produce.

**RED first — run this before accepting any green from the checker:**

```
python3 scripts/bench/check_asset.py _bmad-output/implementation-artifacts/10-2-signoff/tree.glb
```

Required observation: **exit 1**, with the origin-centring clause named and the measured value
`-0.100000` printed. An exit code with no clause name does not satisfy this.

**Then green:**

```
python3 scripts/bench/check_asset.py \
  _bmad-output/implementation-artifacts/10-2-signoff/export/SM_VoxelPine_Tree0{1,2,3,4}.glb
```

Required observation: **exit 0**, four `FIGURES` lines, and the figures matching the table below.

**Second RED, on the instrument itself** (the untested-evidence-channel rule): delete the
origin-centring assertion from `check_asset.py`, re-run the first command, and confirm it now
exits 0 — proving the assertion is what fails it. Restore, then re-run. This is also mutation row
1; commit the fix before mutating [see the sabotage-restore trap].

**The gate** (AC1). Rust is not on the agent's PATH — prefix it, or every cargo step fails for a
reason that has nothing to do with the story:

```
export PATH="$HOME/.local/share/mise/installs/rust/1.97.1/bin:$PATH" && scripts/gate.sh
```

`scripts/tests/test_check_asset.py` is picked up by the `bench tests` line without any gate edit.

**Measured at creation — 2026-08-31, clean tree at `0b8b673`, gate GREEN.** A stdlib probe read
all five committed GLBs. These are the figures the checker must reproduce, and the RED specimen
is confirmed genuinely red rather than assumed:

| File | Size XYZ (m) | min Y | centre X | centre Z | tris | verts |
|---|---|---|---|---|---|---|
| `SM_VoxelPine_Tree01.glb` | 5.0 × 6.4 × 5.0 | 0.000000 | **0.000000** | 0.000000 | 4,366 | 8,732 |
| `SM_VoxelPine_Tree02.glb` | 5.0 × 8.0 × 5.4 | 0.000000 | **0.000000** | 0.000000 | 5,894 | 11,788 |
| `SM_VoxelPine_Tree03.glb` | 3.8 × 8.0 × 3.4 | 0.000000 | **0.000000** | 0.000000 | 3,474 | 6,948 |
| `SM_VoxelPine_Tree04.glb` | 4.6 × 10.6 × 4.6 | 0.000000 | **0.000000** | 0.000000 | 5,280 | 10,560 |
| `tree.glb` **(RED)** | 5.2 × 7.6 × 5.4 | 0.000000 | **−0.100000** | 0.000000 | 5,130 | 10,260 |

All five: `materials=1 primitives=1 images=1 magFilter=9728 (NEAREST) doubleSided=absent
extensionsUsed=none`, and `verts == tris/2 × 4` exactly. So those clauses do **not** discriminate
the stale file — **only the origin centring does**, which is why AC7 names that clause
specifically. `tree.glb`'s mesh and node names are `SM_VoxelPine_Tree02`, identical to the
deliverable's.

## Change Log

| Date | Change |
|---|---|
| 2026-08-31 | Story created. Baseline `0b8b673`, gate green at creation. Five GLBs measured; RED specimen confirmed. |
| 2026-08-31 | Gated on new story 10.6. Grid-scale section rewritten: the reference sheet resolves metres-per-cell (16 voxels/cell, 0.1 m voxel, 1.6 m cell) and the three-way divergence was one stale constant, `scale: 0.65 → 0.75`, owed to 10.5. The servable half now comes from 10.6's measurements instead of being chosen from unmeasured options. |
| 2026-09-01 | Added the procedural and authored-asset contracts, stdlib GLB contract checker, literal-output tests, and six killed mutation rows; full gate green. |

## Dev Agent Record

### Agent Model Used

`gpt-5.6-terra` (Codex CLI 0.146.0, reasoning effort `high`), delegated dev.
Orchestration and verification: `claude-opus-5`.

### Debug Log References

- RED: `python3 -m unittest scripts.tests.test_check_asset -v` failed before the checker existed
  (`can't open file .../scripts/bench/check_asset.py`), then failed on the deliberately incorrect
  quad-soup relation before the minimal correction. The same two tests passed after the correction.
- `scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/10-3-the-rules-of-the-look.sh`
  ran the six recorded sabotages sequentially; `python3 scripts/audit-mutations.py` then reported
  `452 rows, every literal still matches its target`.
- Full gate: `export PATH="$HOME/.local/share/mise/installs/rust/1.97.1/bin:$PATH"; scripts/gate.sh`
  completed with `GATE GREEN`.
- Self-review pass 1 (`codex review --base main`) found and this pass fixed: bounded PNG expansion,
  `FIGURES` for an origin failure, 0.1 m position-grid validation, applied node transforms, and
  explicit `doubleSided: false` acceptance.

### Completion Notes List

- Recorded the 10.6 ruling without collapsing the two resolution axes: terrain is served at 0.4 m
  (`k=4`), while authored assets retain the 0.1 m project voxel and declared integer multiples.
  The 80,120–928,884 terrain range and its placeholder reason are explicit; 928,884 is the
  chunk-mesh ceiling only, excluding about 54k tree-foliage triangles. `appearance.rs` 0.65 → 0.75
  remains owed by story 10.5.
- Added contracts for procedural presentation and authored assets, including AD-16's tile/entity
  distinction and the measured `tree.glb` identity counterexample. No asset, client appearance,
  renderer feature, or simulation code was changed.
- The checker is stdlib-only, bounds GLB reads to 16 MiB, prints literal `FIGURES` for every
  accepted path, and names the first violated contract clause.
- Mutation `the stale off-centre asset is accepted`: **KILLED** — removing the origin-centre
  condition made the stale file return 0 and the named-clause test failed.
- Mutation `a failed contract returns success`: **KILLED** — changing the error return to 0 made
  the named-clause test fail.
- Mutation `reported triangle figures lie`: **KILLED** — changing the reported triangle value to
  zero made the independent literal-figure test fail.
- Mutation `a failed asset omits its figures`: **KILLED** — removing the `FIGURES` print made the
  stale-asset test fail.
- Mutation `off-grid positions are accepted`: **KILLED** — disabling the 0.1 m grid rejection made
  the off-grid GLB fixture return 0.
- Mutation `unapplied transforms are accepted`: **KILLED** — disabling transform rejection made
  the translated GLB fixture return 0.

Verification RED (observed before accepting the recipe GREEN):

```text
FIGURES _bmad-output/implementation-artifacts/10-2-signoff/tree.glb size=5.2x7.6x5.4 min_y=0.000000 centre_x=-0.100000 centre_z=0.000000 tris=5130 verts=10260
FAIL _bmad-output/implementation-artifacts/10-2-signoff/tree.glb: origin-centring clause: centre X/Z are -0.100000/0.000000, expected 0.000000/0.000000
exit 1
```

Verification GREEN:

```text
FIGURES _bmad-output/implementation-artifacts/10-2-signoff/export/SM_VoxelPine_Tree01.glb size=5.0x6.4x5.0 min_y=0.000000 centre_x=0.000000 centre_z=0.000000 tris=4366 verts=8732
FIGURES _bmad-output/implementation-artifacts/10-2-signoff/export/SM_VoxelPine_Tree02.glb size=5.0x8.0x5.4 min_y=0.000000 centre_x=0.000000 centre_z=0.000000 tris=5894 verts=11788
FIGURES _bmad-output/implementation-artifacts/10-2-signoff/export/SM_VoxelPine_Tree03.glb size=3.8x8.0x3.4 min_y=0.000000 centre_x=0.000000 centre_z=0.000000 tris=3474 verts=6948
FIGURES _bmad-output/implementation-artifacts/10-2-signoff/export/SM_VoxelPine_Tree04.glb size=4.6x10.6x4.6 min_y=0.000000 centre_x=0.000000 centre_z=0.000000 tris=5280 verts=10560
exit 0
```

Second RED (the first mutation removes the origin-centre assertion; the checker then accepts the
same stale file, proven by the targeted test's observed `0 != 1`):

```text
AssertionError: 0 != 1 : FIGURES /workspace/projects/frostvein/_bmad-output/implementation-artifacts/10-2-signoff/tree.glb size=5.2x7.6x5.4 min_y=0.000000 centre_x=-0.100000 centre_z=0.000000 tris=5130 verts=10260
```

### Orchestrator verification — 2026-09-01

The delegated dev run was **killed by the harness during its own final `scripts/gate.sh`**, so its
green-gate claim was not self-evidenced. Everything below was re-run by the orchestrator on the
branch tip and observed directly, not read off the dev transcript.

- **Full `scripts/gate.sh`: GREEN**, exit 0 — the full `cargo test`, not the fast set.
- **Mutation table re-run independently: 6/6 KILLED**, working tree restored clean afterwards.
  Stale `__pycache__` cleared after the run and the RED/GREEN pair re-observed (exit 1 / exit 0),
  guarding the `.pyc` shadow trap.
- **Verification recipe re-run:** RED exits 1 naming the origin-centring clause and printing
  `-0.100000`; GREEN exits 0 with four `FIGURES` lines. All figures match an independent stdlib
  probe of the five GLBs taken before handoff.
- **Scope verified structurally, not asserted:** `git diff a5d8acf..HEAD` touches **zero files
  under `crates/`**; all five committed GLBs are byte-identical to main by sha256; no `assets/`
  directory was created; `check_asset.py` imports only `json`, `pathlib`, `struct`, `sys`, `zlib`.
- **Every contract citation resolved** against the current tree. `valley_bench.py:terrain_luma` is
  a figure key rather than a `def`, enforced by `test_terrain_luma_averages_lit_pixels_only`
  at the cited lines — substantively correct.
- **8 commits, all authored `Völundr <jeicei75@gmail.com>`**, one per task, no squash.
- Codex ran **1 of its 3 permitted `codex review --base main` self-gate passes**.

**Recorded discrepancy, not corrected:** `baseline_commit` is `0b8b673` (the creation baseline),
while the branch is cut from `a5d8acf`, 39 commits later — main moved when 10.6 merged. The
workflow rule preserves an existing `baseline_commit`, so it was left alone; **review should diff
against `a5d8acf`, not `0b8b673`.**

**Two story citations drifted** and were corrected in the handoff rather than in the story text:
the gate's python discovery is `scripts/gate.sh:124` (story says :117) and `import bpy` is
`voxel_pine.py:40` (story says :38). Substance unaffected.

### File List

- MODIFIED: `docs/tech-art-guidelines.md`
- NEW: `scripts/bench/check_asset.py`
- NEW: `scripts/tests/test_check_asset.py`
- NEW: `_bmad-output/implementation-artifacts/mutations/10-3-the-rules-of-the-look.sh`
- MODIFIED: `_bmad-output/implementation-artifacts/10-3-the-rules-of-the-look.md`
- MODIFIED: `_bmad-output/implementation-artifacts/sprint-status.yaml`
- MODIFIED: `_bmad-output/implementation-artifacts/metrics/.session-cursors.json`
- MODIFIED: `_bmad-output/implementation-artifacts/metrics/10-3-the-rules-of-the-look.md`
- MODIFIED: `_bmad-output/implementation-artifacts/metrics/10-6-how-fine-can-we-go.md`
- NEW: `_bmad-output/implementation-artifacts/metrics/10-4-the-trees-look-right-the-pilot.md`
- MODIFIED (code review 2026-09-01): `_bmad-output/implementation-artifacts/deferred-work.md`

### Review Findings — code review 2026-09-01

Four layers, all live, NO coverage holes: Blind Hunter (`check_asset.py`), Edge Case Hunter
(tests/mutations/docs), Acceptance Auditor and Feature Auditor (whole diff). Every layer ran
`cargo --version` clean and executed the instruments. R1 territories were REASSIGNED: this diff
touches no Rust, so both hunters' approved territories (`sim-core`; the shells) were empty —
recorded so this story is not counted as evidence for or against the R1 split.

Two four-way convergences (D2/P2 and P5) and one three-way (P3) — a much stronger convergence
signal than Epic 3's 1-in-8.

**Decision needed**

- [x] [Review][Decision] Contract states k=4 / 0.4 m terrain voxels in the present tense, but the
      shipped default is k=1 — `docs/tech-art-guidelines.md:190`. `TerrainSubdivision` is inserted
      only under `--subdiv` (`crates/gui/src/ingest.rs:203-205`); every consumer falls back via
      `subdivision.map_or(1, ...)` (`crates/gui/src/project.rs:1108`, `:1184`, `:1196`). The doc's
      own next sentence asks for "the adopted `k` in one constant" — no such constant exists and no
      story owns it, unlike `scale: 0.65 -> 0.75` which is explicitly owed by 10.5. Either reword as
      an adopted decision with an owner, or make k=4 the shipped default. [accept+feature] HIGH
- [x] [Review][Decision] The checker is a v1-pine validator applied to every `.glb`
      — `scripts/bench/check_asset.py:194`, `:198`, `:142`. It unconditionally demands exactly one
      mesh AND one material AND one image, one primitive, and a 64x64 atlas before any grid/origin
      clause runs. The contract scopes those to "V1 trees"/"V1 voxel assets". A two-material dwarf —
      the asset class 10.5 is about to build, and which `epics.md:1560` says will be checked against
      this contract — fails with `one-mesh/material/image clause`, reading as a contract violation
      when it is a scope mismatch. [feature] MED
- [x] [Review][Decision] Three of 10.2's eight standing-contract clauses did not survive into the
      durable document — clause 6 (self-verification order; "Exit 0 with no output is not a
      result"), clause 7 (three deliverables; "the script is the durable record; the session is
      not"), clause 8 (declare deviations). Clause 7 is what makes the MCP-session path in this
      story's headline sentence reproducible. `voxel_pine.py:714` still cites "the asset contract's
      clause 6", which now resolves to nothing in `docs/`. Port them or record why not. [feature] MED

**Patch**

- [x] [Review][Patch] A parent node defeats BOTH the origin and applied-transform clauses
      [scripts/bench/check_asset.py:218] — HIGH. `has_applied_transform` is called only on the leaf
      mesh node, and `positions()` reads local-space coordinates, so an ancestor's translation is
      never seen. VERIFIED: Tree02 wrapped in a parent translated [3.0, 5.0, -2.0] reports
      `min_y=0.000000 centre_x=0.000000 centre_z=0.000000`, exit 0 — an asset rendering 5 m in the
      air and 3 m off-centre certified as perfectly centred. This is the most likely shape for a
      Blender/hand/MCP export, i.e. exactly the producer path the story exists to police.
      [feature+orchestrator]
- [x] [Review][Patch] The 70-180 clause cites an instrument that enforces neither its bound nor its
      statistic [docs/tech-art-guidelines.md:221] — HIGH, FOUR-WAY CONVERGENCE. Cited as
      mechanically-checkable via `valley_bench.py:terrain_luma` + `test_valley_bench.py:125-159`.
      Actually: `terrain_luma` is `luma_sum / non_sky`, a MEAN (`valley_bench.py:370`); the only
      bound is `MIN_TERRAIN_LUMA = 20.0` (`:110`) with NO ceiling. The real enforcer is
      `crates/gui/src/capture.rs:459`/`:464` (`GROUND_LUMINANCE_FLOOR/CEILING`), tested at
      `capture.rs:1329-1343`. The same document CONTRADICTS ITSELF at `:141`: "No headless test can
      see either... Median rather than mean." Re-cite to capture.rs. This also falsifies the story's
      own claim that every contract citation resolved against the current tree.
      [accept+edge+feature+orchestrator]
- [x] [Review][Patch] The naming clause's file-basename half is unenforced
      [docs/tech-art-guidelines.md:275 / scripts/bench/check_asset.py:222] — MED, three-way.
      The clause is marked Mechanically-checkable and lists only "path placement and source
      ownership" as eye-only, but the checker compares mesh name to node name and never looks at
      `path.stem`. VERIFIED: a copy named `WrongName.glb` passes at exit 0. Note the sting — the
      story's own motivating file `tree.glb` IS a basename/internal-name mismatch, and is caught
      only by accident because it also happens to be 0.1 m off-centre. A properly centred stale file
      passes everything. One `path.stem` comparison closes it. [edge+feature+orchestrator]
- [x] [Review][Patch] NaN/Inf in a POSITION crashes with a raw traceback instead of naming a clause
      [scripts/bench/check_asset.py:237] — MED. `round()` on NaN/Inf raises
      `ValueError`/`OverflowError`, uncaught. No `FIGURES` line and no `FAIL <path>: <clause>` line
      are produced, breaking AC6's "exits non-zero naming the first violated clause". VERIFIED on a
      repacked GLB with one NaN float. Guard with `math.isfinite` and raise an `AssetError`.
      [blind+orchestrator]
- [x] [Review][Patch] `PALETTE_HEX` is a constant read by nothing, and its eye-only clause has
      nothing to look at [scripts/bench/check_asset.py:22] — MED, FOUR-WAY CONVERGENCE. The seven
      hex literals are consumed only as `len(PALETTE_HEX)` at `:145`; `palette_from_glb`'s return
      value is DISCARDED at `:216`. 10.2's own `voxel_pine.py:688` DID assert
      `shipped_palette == PALETTE_HEX` — the port kept the decode and dropped the assertion. The
      contract is honest (role read marked eye-only), but `FIGURES` never prints the palette, so the
      eye-only signoff comparison it promises has no data to compare. Print the sampled hex in the
      FIGURES line. [blind+accept+feature+orchestrator]
- [x] [Review][Patch] "five-step rim colour" is a stale value; `RIM_LEVELS = 13`
      [docs/tech-art-guidelines.md:232] — MED. `crates/gui/src/appearance.rs:251` is 13, and
      `project.rs:1669-1671` records that the 5-step ramp was explicitly REPLACED after Wolf called
      the falloff too sharp. The cited test is level-count-generic, so it can never catch this
      drift. [edge+orchestrator]
- [x] [Review][Patch] The slice clause cites a test module, not its mechanism
      [docs/tech-art-guidelines.md:240] — MED. `crates/gui/src/slice.rs:90-153` is `SliceLevel`'s
      test module (clamping, readout string, surface/underground label). The actual "expose below
      level, retain the solid cut face" mechanism is `is_visible_at_slice` at
      `crates/gui/src/project.rs:1851-1856`, with no dedicated test; the "neither hatch nor
      simulation state" half is untested by the cited range. Re-cite, and mark the untested half
      eye-only. [edge]
- [x] [Review][Patch] The checker enforces a 64x64 atlas rule the contract never states
      [scripts/bench/check_asset.py:142] — MED. Task 4 requires every clause the checker enforces to
      be one the contract states. `ATLAS = 64` rejects a conforming 32x32 or 128x128 atlas by a rule
      no human can read in the contract. [accept]
- [x] [Review][Patch] Two cheap mechanical checks were dropped and marked in NEITHER column
      [docs/tech-art-guidelines.md:259] — MED. `ASSET_NOTES.md:143` requires all UVs inside 0-1 and
      10.2's clause 3 requires wrap `CLAMP_TO_EDGE`; neither appears in the new contract nor in the
      checker. The shipped assets already declare `wrapS/wrapT = 33071` and carry `TEXCOORD_0`, so
      both are two JSON lookups and one bounds loop over an accessor the checker already reads.
      AC2/AC4 require every clause to be marked one way or the other; these vanished. [feature]
- [x] [Review][Patch] `extensionsRequired` is unchecked; only `extensionsUsed` is
      [scripts/bench/check_asset.py:199] — LOW, bundled (same function). VERIFIED: a GLB declaring
      `extensionsRequired: [KHR_materials_unlit]` passes at exit 0 against a "no glTF extensions"
      clause. [blind+orchestrator]
- [x] [Review][Patch] `FIGURES` names no units [scripts/bench/check_asset.py:257] — LOW, bundled.
      `size=5.0x8.0x5.4` could be metres, cells or voxels, in a story whose whole subject is that
      three implied scales diverged. [feature]
- [x] [Review][Patch] Four of five premise citations no longer resolve; one names a literal that
      does not exist [story:25-38] — LOW, bookkeeping. Substance of all five HOLDS; only the
      citations are stale (written 39 commits before the branch base). `Cuboid::default()` is
      `project.rs:235` not `:194`; `rng.random_range(4..=6)` is `worldgen.rs:193` not `:196`; the
      1.20 m assumption is `ASSET_NOTES.md:36` not `:57`; `epics.md`'s grid-scale language is
      `:1510`/`:1518` not `:1505`. `Vec3::splat(0.65)` exists NOWHERE in the repo — the real path is
      `Vec3::splat(appearance.scale)` at `project.rs:1361` with `scale: 0.65` at
      `appearance.rs:278`. Also the header says "All four checked" then lists five. [accept]
- [x] [Review][Patch] Citation sweep: two more anchors [docs/tech-art-guidelines.md:213] — LOW,
      bundled with the citation fixes above. `appearance.rs:300` is the closing `};` of a `use`
      statement; the enforcing test starts at `:303`. And `ARCHITECTURE-SPINE.md:107-119` carries no
      path prefix while two candidate files exist (content matches the `-08-09` copy). [accept+edge]
- [x] [Review][Patch] File List is incomplete [story:387] — LOW, bookkeeping. Lists 5 files; the
      diff touches 10. Omitted: `metrics/.session-cursors.json`, `metrics/10-3-...md`,
      `metrics/10-4-the-trees-look-right-the-pilot.md` (a NEW file for a backlog story),
      `metrics/10-6-...md`, `sprint-status.yaml`. [accept]

**Deferred**

- [x] [Review][Defer] Nothing auto-checks any asset that does not already exist
      [scripts/tests/test_check_asset.py:52] — deferred to 10.5. The gate DOES re-check the five
      committed `.glb` files on every run (better than expected), but the paths are hardcoded: no
      glob, no discovery, no CI (`.github/` does not exist), and `assets/gltf/` — the runtime home
      the contract names — is created by 10.5, not here.
- [x] [Review][Defer] Nothing hands the asset producer the contract
      [_bmad-output/implementation-artifacts/10-2-signoff/voxel_pine.py:714] — deferred, out of
      scope. Hops 1-2 of the feature path are unwired: no generator, hand workflow or MCP session is
      given the rules at production time. The MCP runbook is explicitly issue #58.
- [x] [Review][Defer] Batch runs abort at the first bad file
      [scripts/bench/check_asset.py:289] — deferred, spec-consistent. AC6 says "naming the first
      violated clause", but a user with ten assets fixes them one round-trip at a time, and a
      structurally invalid file emits no `FIGURES` line at all.
- [x] [Review][Defer] "Three divergent implied values" is two consistent halves plus one stale
      constant [docs/tech-art-guidelines.md:181] — deferred, AC letter met. The dwarf-voxel and
      tree-cell readings are the two halves of one self-consistent derivation, not two rival
      values; the Dev Notes say so honestly ("it is one stale constant") but the shipped contract
      text keeps the three-way framing.

#### Review verification — 2026-09-01

All 3 decisions resolved by Wolf and all 17 patches applied in one batch, then ONE verification
pass (the batching rule: re-gate turns are the highest cost-per-turn work in a review).

- **`scripts/gate.sh` FULL tier: GATE GREEN**, all 9 checks `ok`, no "WITH SKIPS (coverage hole)"
  line. Bench suite 42 tests (was 37), OK, no skips.
- **Mutation table re-run after the strengthening patches: 10/10 KILLED** (was 6/6). Re-mutation
  was mandatory, not optional — "KILLED" names the TEST, not the assertion just added.
  `audit-mutations.py`: 456 rows, every literal still matches.
- **One mutation row was retargeted during the patch pass, and this is the interesting bit.**
  "the published palette is not read from the artifact" first targeted the four-pines test — but
  those four assets' palettes EQUAL `PALETTE_HEX`, so substituting the constant for the
  artifact-read value produces byte-identical output and the row would have SURVIVED while reading
  as evidence. Only `tree.glb`, whose palette differs (`#110B07,...`), can tell the two apart. Row
  retargeted to the stale-asset test; it now kills. This is the self-referential-oracle antipattern
  caught inside the review's own new evidence.
- **AC7 re-observed after the patches and is UNCHANGED**: the four exports pass at exit 0;
  `tree.glb` still fails naming the **origin-centring** clause, not the new naming clause, because
  the basename check is deliberately ordered after the origin branch.
- **Both-sides closure.** The parent-node fix was written for the ancestor direction and tested
  from the leaf direction too (the pre-existing `translated.glb` leaf-transform case still kills);
  the fixture is a byte-mutated REAL Tree02, not a shape built from what the new code emits.
- Build caches reaped after triage: `reap-build-caches.sh --tmp-only --force` removed 7
  directories / 51.6 GB under /tmp, reclaiming 25.8 GB of free space. Repository `target/` untouched.

**Not proven by this review, and stated rather than buried:** every clause the two contracts mark
**eye-only** remains unverified by construction — aurora colour and fold character, the declared
voxel multiple and visual read, the intended palette-role read, downstream-tool suitability, path
placement, the solid cut face, and figures-vs-signoff comparison. They are correctly *marked*,
which is what AC2 and AC4 ask for; they are not *satisfied*, and 10.4/10.5 inherit them.
