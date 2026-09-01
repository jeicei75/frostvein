---
baseline_commit: 0b8b6735f04b282e2d75b82e426346be49590082
---

# Story 10.3: The Rules of the Look

Status: in-progress

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

1. **"Grid scale is unset" — HOLDS, but the reference sheet already answers half of it.**
   `epics.md:1505` names grid scale as blocking 10.4/10.5 and predates the 10.2 spike. What the
   sheet fixes and what it cannot is set out in Dev Notes; **the servable half comes from story
   10.6, which runs before this one.**
2. **"The client's cell is a unit cube" — HOLDS.** `Cuboid::default()` at
   [crates/gui/src/project.rs:194] is 1×1×1; the dwarf is `Vec3::splat(0.65)` on it
   [crates/gui/src/project.rs:615], `scale: 0.65` at [crates/gui/src/appearance.rs:278].
3. **"Worldgen grows trees 4–6 cells" — HOLDS.** `rng.random_range(4..=6)` at
   [crates/sim-core/src/worldgen.rs:196].
4. **The 1.20 m dwarf anchor is NOT ratified anywhere.** It appears only in
   [10-2-signoff/ASSET_NOTES.md:57] as that story's working assumption. `rg` over `docs/` and the
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

- [ ] **Task 3 — Write the asset contract** (AC: 4, 5)
  - [ ] Generalise the standing rules out of [10-2-signoff/ASSET_NOTES.md] — that file is one
        asset's brief; the contract is the family of rules it is an instance of.
  - [ ] Decide and state file locations: where a generator lives, where a runtime glTF lives,
        where signoff artifacts live. Naming a path is enough; **do not create `assets/`**.
  - [ ] Write the identity clause (AC5) with `tree.glb`'s measured figures as the case.

- [ ] **Task 4 — Build the checker** (AC: 6, 7)
  - [ ] `scripts/bench/check_asset.py`, stdlib only. Lift the GLB reader from
        [10-2-signoff/voxel_pine.py:437-530] (`load_glb`, `read_accessor`, `decode_png_rgb`,
        `palette_from_glb`) rather than re-deriving it, and carry a `# NOTE:` saying why it is
        duplicated: that file imports `bpy` at module scope, so it is not importable outside
        Blender, and it is 10.2's frozen deliverable.
  - [ ] Check only the mechanically-checkable clauses. Every clause the checker enforces must be
        one the contract states; every contract clause it cannot check stays marked eye-only.

- [ ] **Task 5 — Test the instrument and the sabotage rows** (AC: 8)
  - [ ] `scripts/tests/test_check_asset.py` — picked up automatically by
        [scripts/gate.sh:117] (`unittest discover -s scripts/tests`); no gate edit needed.
  - [ ] The test must assert the checker **fails** on `tree.glb` and **names the clause**, not
        merely that it exits non-zero. An exit code alone does not discriminate a working checker
        from one that rejects everything.
  - [ ] Mutation table rows, format per [mutations/10-1-the-headless-bench.sh]. At least: remove
        the origin-centring assertion; make the failure path exit 0; break a figure the `FIGURES`
        line reports. Run `scripts/mutate.sh` and record KILLED per row.

- [ ] **Task 6 — Verification** (AC: 7)
  - [ ] Execute the recipe below, RED first, and paste both outputs into the Dev Agent Record.

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

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
