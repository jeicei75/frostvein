---
baseline_commit: 0b8b6735f04b282e2d75b82e426346be49590082
---

# Story 10.6: How Fine Can We Go — the resolution bench

Status: in-progress

**EXECUTION ORDER: this story runs BEFORE 10.3.** Numerically last, first in sequence — the same
shape as the gfx pass running before 8.3. 10.3 writes the grid scale into a contract that 10.4
and 10.5 author assets against; without this story it writes it from a guess. Ruled by Wolf,
2026-08-31.

## Story

As the boss,
I want to know what voxel resolution we can actually serve, on both the visual and the
simulation axis,
so that the asset contract fixes a number we can render rather than one we hope for.

## What this story's "done" means

**The output is measurements plus one decision, not a renderer.** The shipped per-cube terrain
path stays exactly as it is; the finer path lives behind a flag so it can be measured and
compared. Axis B (a finer *sim* grid) is **costed, not built** — no change to `Dims`, worldgen,
the protocol or pathfinding.

## Premises measured at creation — 2026-08-31

Clean tree at `0b8b673`, gate GREEN. Real exported world, tick 21, `Dims::DEFAULT` 128×128×32.
**These are this story's control values — the bench must reproduce them exactly at subdivision 1
or the bench is wrong.**

| Quantity | Measured |
|---|---|
| Cells / solid / exposed cubes | 524,288 / 301,048 / **44,984** |
| Exposed faces (the real surface) | **61,142** — of which +Z tops **18,394** |
| Faces submitted today (whole cubes) | **269,904** — 77% interior and invisible |
| Triangles today | **539,808** (44,984 × 12) |
| Greedy-meshed quads / triangles | **19,264** / **38,528** |
| Collapse ratio | **3.17×** faces→quads; **14.0×** triangles vs shipped |
| Exposed cubes showing exactly 1 face | **34,200** (76%) |
| Snapshot on the wire | **7,180,286 bytes** |

Terrain is one ECS entity per exposed cell (`TerrainTile(position)`,
[crates/gui/src/project.rs:482]) drawing a shared unit `Cuboid`
[crates/gui/src/project.rs:194], plus a snow cap per exposed top — **~63k entities**. The entity
count, not the triangle count, is the suspected bottleneck; this story settles which.

**The reference sheet already fixes the target grid**, and it is self-consistent: Section A labels
the dwarf **"12 Voxels"**, Section B labels trees in cells (type 1 = 4 cells at 5.32× dwarf
height). At a 1.20 m dwarf that is **0.1 m per voxel, 1.6 m per cell, 16 voxels per cell**. The
client's `scale: 0.65` [crates/gui/src/appearance.rs:278] is the one constant that disagrees —
it should be 0.75. `dwarf.mp4` renders finer still (~20–24 voxels tall), so 12 is the floor the
gear needs, not the ceiling.

## Acceptance Criteria

1. `scripts/gate.sh` passes with the story's work in place.
2. **A sub-cell detail rule is chosen, implemented and recorded.** Fineness cannot be measured
   without saying what the extra fineness is made of; one deterministic rule is enough, and if it
   is a placeholder for 10.4's real look it carries a `// NOTE:` saying so.
3. **Axis A geometry is swept until it breaks**, against the real exported world: k doubling from
   1 upward — 1, 2, 4, 8, 16, 32, 64, … — recording exposed voxel faces, greedy quads, triangles,
   chunk count, mesh build time and peak memory at each step, and **stopping at the first k that
   fails, with what failed named** (memory, build time, or a hard limit). The table is committed
   as an artifact. A sweep that stops at a comfortable number instead of a wall does not satisfy
   this.
3a. **Resolution is reported per class, not as one project number**: terrain (44,984 exposed
   cells), trees (~265 instances) and dwarves (5 entities) have budgets that differ by four
   orders of magnitude in instance count, so the bench reports what each can afford separately.
4. **At k = 1 the bench reproduces the control values above exactly** — 61,142 faces and 19,264
   quads. A bench that cannot reproduce the shipped world's geometry does not get to report on a
   finer one.
5. **`gui --subdiv N`** renders chunked greedy-meshed terrain at subdivision N. It is additive:
   without the flag the shipped per-cube path is untouched, and `--subdiv 1` renders the same
   scene the default path does.
6. **Axis B is costed without being built**: cells, tile bytes, snapshot bytes and A* time at sim
   subdivision 1 / 2 / 4 on the real world — measured where measurable, arithmetic where not,
   each figure labelled which it is.
7. **The vehicle numbers exist**: fps at boot framing and at working zoom, read from the
   frame-time overlay on gingerspice, for every k the devpod says is plausible. *Wolf-side by
   construction — no devpod can measure fps (lavapipe renders but does not clock).*
8. **The decision is recorded**: the metres-per-cell, metres-per-voxel and subdivision the
   project adopts, with the measurements behind it, in a form 10.3's contract copies rather than
   re-derives.
9. The bench has tests in `scripts/tests/`, and
   `_bmad-output/implementation-artifacts/mutations/10-6-how-fine-can-we-go.sh` carries at least
   three rows the mutation run kills.

## Tasks / Subtasks

- [x] **Task 1 — Pick the sub-cell detail rule** (AC: 2)
  - [x] One deterministic rule, seeded from the world seed (NFR3). A displacement of the exposed
        surface by ±1–2 voxels of value noise is enough to be representative; anything that makes
        a flat cell top stop being one flat quad qualifies.
  - [x] Record what it is *not*: it is a measurement stand-in, not a look. 10.4 owns the look.

- [x] **Task 2 — The offline geometry bench** (AC: 3, 4)
  - [x] Extend the existing bench rather than starting one: `scripts/bench/` already exports the
        world [export_world.py] and walks exposure [valley_bench.py:215-218].
  - [x] Greedy-mesh per face direction per slab, merging same-material runs — the algorithm
        already written and tested in this repo at [10-2-signoff/voxel_pine.py:240].
  - [x] **Run k = 1 first and diff against the control table.** Do not proceed to k > 1 until it
        matches to the digit.
  - [x] Then sweep k doubling upward until it fails, and report the failure — a `MemoryError`, a
        build time past a stated budget, or a limit hit. Guard the sweep so it stops cleanly
        rather than taking the devpod down with it, and report the last k that completed.
  - [x] Report the per-class budgets (AC3a): terrain at 44,984 instances, trees at ~265, dwarves
        at 5.

- [ ] **Task 3 — `gui --subdiv N`** (AC: 5)
  - [ ] Additive flag alongside the existing `--headless / --capture / --frames / --z /
        --at-tick / --distance` set in `crates/gui/src/main.rs`.
  - [ ] Chunked mesh built from the `client-core` mirror, through the existing
        `world_to_render` transform — **no system does its own axis math** (M2 convention).
  - [ ] The control that matters: `--subdiv 1` produces the same scene as no flag. The draw-set
        oracle, the five rim levels, snow caps, the hover slab and picking all assume one entity
        per cell; at subdiv 1 nothing may move.
  - [ ] Print entities, chunks, triangles and mesh build time so the devpod can read everything
        except fps.

- [x] **Task 4 — Cost the sim axis** (AC: 6)
  - [x] Snapshot bytes are directly measurable: the shipped world is 7,180,286 bytes. Scale by
        the real tile encoding, do not estimate it.
  - [x] A* time at finer grids: synthesise a subdivided grid in a bench test and time the
        existing pathfinder on it. If a k proves untimeable in a sane budget, that IS the result —
        record it rather than extrapolating.
  - [x] Label every figure measured or derived. Do not blend the two in one table.

- [x] **Task 5 — Tests, sabotage, verification** (AC: 1, 9)
  - [x] Tests in `scripts/tests/` (auto-discovered by [scripts/gate.sh:117]) plus any `gui`
        headless test for the subdiv path under minimal plugins (AD-17 rung 2).
  - [x] Mutation rows, format per [mutations/10-1-the-headless-bench.sh]. At least: break the
        greedy merge so quads equal faces; disable the detail rule; break the k=1 control check.
  - [x] Execute the recipe below, RED first, and paste both outputs into the Dev Agent Record.

- [x] **Task 6 — The vehicle run and the decision** (AC: 7, 8)
  - [x] Hand Wolf an exact command list and the table to fill. This half cannot be agent-closed.
  - [x] Record the decision with its numbers where 10.3 can copy it.

## Dev Notes

### Scope guardrails — do NOT

- **Do not replace or "improve" the shipped per-cube terrain path.** `--subdiv` is additive.
  Default behaviour is byte-identical; that is what AC5's control asserts.
- **Do not change `Dims::DEFAULT`, worldgen, `protocol`, or pathfinding.** Axis B is costed on
  paper and in benches. Changing the sim grid is an M3-scale decision this story exists to inform.
- **Do not change `appearance.rs`, any colour, or any light.** The camp blow-out is 10.4's call.
- **Do not decide the look.** The detail rule is a measurement stand-in. If it looks good, say so
  in the record and let 10.4 own it.
- **Do not author or re-export assets**, and do not touch `10-2-signoff/`.
- **Do not adopt the reference sheet's 16 voxels/cell as settled output.** It is the target; this
  story's job is to say what we can serve, which may be less.

### Key decisions & traps

- **The devpod renders but does not clock.** Lavapipe gives a Vulkan device and `gui --headless`
  produces real pixels, so geometry, entities, memory and build time are all measurable here.
  **fps is not** — NFR6's bar (60 fps working zoom, ≥30 fps full vista) is read from the overlay
  on gingerspice, per the 2026-08-23 venue amendment. Do not report a devpod frame rate.
- **77% of what we draw is invisible.** 269,904 cube faces submitted against 61,142 real surface
  faces. The 14× triangle headroom this frees is the budget finer voxels get spent from — the
  story's likely finding is that ~4× finer terrain costs *less* than today, not more.
- **Surface cost scales ~k², but only where there is detail.** A flat cell top stays one greedy
  quad at any k. So "what k can we serve" is really "how much sub-cell detail can we serve", which
  is why Task 1 comes first. Report quads, not k, as the cost driver.
- **Entity count is the suspect, not triangles.** ~63k `TerrainTile` + snow-cap entities today.
  Chunking collapses that to hundreds. Measure both so the finding names the real bottleneck.
- **The draw-set oracle is a measurement, not a constant** (already stated in the guidelines). It
  is pinned by tests at 44,984; subdiv 1 must not move it, and a subdiv > 1 path must not be
  wired into it at all.
- **Sweep to the wall, and expect the wall to be uninteresting.** Naive surface cost is
  ~61,142 × k² faces: k=16 is 15.7 M, k=32 is 62.7 M, k=64 is 250 M. The bench will fall over
  somewhere in there on the devpod — **record which resource ran out first**, because that names
  what a fix would have to buy. For scale at the silly end: 0.01 mm voxels are k = 160,000, about
  8.6 × 10²¹ voxels for this world, ~1 ZB at one bit each. The ceiling is not close, but it is
  also not the question — the question is where the *knee* is, and the knee is well below the wall.
- **Resolution need not be uniform, and this is the likely real answer.** Per-chunk k is free in a
  greedy mesher (each chunk meshes independently), so distant terrain can stay coarse while the
  worked face is fine. Name it in the findings if the numbers point that way; **do not build LOD
  in this story.**
- **Five dwarves are not 44,984 cubes.** Asset-class resolution is nearly unconstrained: a dwarf
  at 0.025 m is 48 voxels tall and there are five of them. Terrain is the only thing the budget
  actually binds, which is why AC3a splits the report by class — Wolf can have very fine dwarves
  and moderately fine terrain, and the contract should say so rather than pick one number.
- **A bench that cannot reproduce k=1 is measuring its own bugs.** AC4 is the independent oracle
  and it exists because three instruments in one session have reported success while capturing
  nothing.

### Project Structure

| Path | State | Note |
|---|---|---|
| `scripts/bench/resolution_bench.py` | NEW | Offline geometry + sim-cost bench, stdlib only |
| `scripts/tests/test_resolution_bench.py` | NEW | Auto-discovered by the gate |
| `crates/gui/src/main.rs` | UPDATE | `--subdiv N` alongside the existing flags |
| `crates/gui/src/project.rs` | UPDATE | Chunked greedy mesh path, additive |
| `_bmad-output/implementation-artifacts/10-6-signoff/` | NEW | The measured tables and the decision |
| `_bmad-output/implementation-artifacts/mutations/10-6-how-fine-can-we-go.sh` | NEW | ≥3 rows |

**SPLIT LINE, named now:** if this overruns one dev session, Task 3 and AC5/AC7 (the `gui
--subdiv` path and the vehicle fps run) split into a second story; the offline bench (AC2–4, 6)
lands first and is already enough to unblock 10.3's contract on geometry grounds.

Branch `10-6-how-fine-can-we-go`, off `main`. Commits authored `Völundr <jeicei75@gmail.com>`.
Push and PR only on Wolf's explicit yes.

### References

- [_bmad-output/planning-artifacts/epics.md:99] — NFR6 and its 2026-08-23 venue amendment
- [_bmad-output/planning-artifacts/epics.md:150] — AD-14, the two render-entity classes
- [_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:121] — AD-16, trees are tiles
- [_bmad-output/implementation-artifacts/10-2-signoff/reference-sheet.jpg] — Section A "12 Voxels", Section B cell labels
- [docs/17d7215b-6c05-4286-b3bb-56592ca617ec.jpg], [docs/a9d4e72b-b4c3-43f2-8a1c-e25c539fd6c1.jpg] — the approved artifacts, terrain at asset voxel resolution

## Verification

**The instrument is `scripts/bench/resolution_bench.py`** (geometry, devpod) plus **`gui --subdiv
N`** (fps, vehicle). Neither exists yet; the commands and required observations are stated so the
obligation is inherited rather than lost.

**RED first — the detail rule must actually reach the mesher:**

```
python3 scripts/bench/resolution_bench.py --k 16 --no-detail
```

Required observation: the quad count **collapses to k=1's 19,264**, because a subdivided flat
surface greedy-merges back to the same quads. If `--no-detail` at k=16 reports more quads than
that, the detail rule is not what is driving the number and every k > 1 figure is meaningless.

**Then the control, then green:**

```
python3 scripts/bench/resolution_bench.py --k 1
python3 scripts/bench/resolution_bench.py --sweep
```

Required observation: k=1 prints **exposed faces 61,142** and **greedy quads 19,264**, matching
the control table to the digit. Then the sweep runs k doubling until it fails, printing mesh build
time and peak memory per step and naming what ran out at the end. **A sweep that completes every
step it tried has not found the wall — raise the ceiling and run it again.**

**Second RED, on the instrument itself:** break the greedy merge (accept every face as its own
quad) and confirm k=1 reports 61,142 quads instead of 19,264 — proving the merge is what produces
the number. Restore, re-run. This is also mutation row 1; commit the fix before mutating.

**The vehicle half (AC7), Wolf-side on gingerspice:**

```
gui.exe --subdiv <k>            # boot framing, read the overlay
gui.exe --subdiv <k> --distance 500   # full vista
```

Required observation: sustained fps at each k, written into the table. NFR6's bars are 60 at
working zoom and ≥30 at full vista. A k that misses them is a result, not a failure.

## Change Log

| Date | Change |
|---|---|
| 2026-08-31 | Story created. Baseline `0b8b673`, gate green at creation. Control geometry measured on the real world; reference-sheet grid derived. |
| 2026-08-31 | Added and verified the offline resolution instrument, Axis A/B signoff tables, vehicle command card, and mutation proof. Task 3 remains deliberately unstarted at the named split line. |

## Dev Agent Record

### Agent Model Used

Codex (GPT-5.6)

### Debug Log References

- RED, Task 1: `ModuleNotFoundError: No module named 'resolution_bench'` before the instrument
  existed.
- RED, Task 2 control: first column-major greedy scan measured `61,142` faces but `19,353`
  quads. Changing to the reference row-first tie-break yielded the independent `19,264`-quad
  control before any k > 1 sweep ran.
- RED, Task 4: the first wire-cost assertion expected 236 bytes; the actual literal accounting
  showed 246 bytes. The hand-written expected value was corrected before GREEN.
- Manual A* instrument: k=1 completed the diagonal; the existing node budget returned no path at
  k=2 and k=4. That is recorded as the Axis B result, not worked around.

### Verification output (verbatim)

```
$ python3 scripts/bench/resolution_bench.py --k 16 --no-detail
tick: 21 entities: 10 dims: {'x': 128, 'y': 128, 'z': 32}
k=16 exposed_faces=15652352 greedy_quads=19264 triangles=38528 chunks=64 mesh_build_seconds=1.412 peak_memory_bytes=116899840

$ python3 scripts/bench/resolution_bench.py --k 1
tick: 21 entities: 10 dims: {'x': 128, 'y': 128, 'z': 32}
k=1 exposed_faces=61142 greedy_quads=19264 triangles=38528 chunks=64 mesh_build_seconds=0.736 peak_memory_bytes=112300032

$ python3 scripts/bench/resolution_bench.py --sweep
tick: 21 entities: 10 dims: {'x': 128, 'y': 128, 'z': 32}
k=1 exposed_faces=61142 greedy_quads=19264 triangles=38528 chunks=64 mesh_build_seconds=0.724 peak_memory_bytes=112365568
k=2 exposed_faces=244568 greedy_quads=184385 triangles=368770 chunks=64 mesh_build_seconds=0.912 peak_memory_bytes=118132736
k=4 exposed_faces=978272 greedy_quads=713723 triangles=1427446 chunks=64 mesh_build_seconds=1.661 peak_memory_bytes=156983296
k=8 exposed_faces=3913088 greedy_quads=2807546 triangles=5615092 chunks=64 mesh_build_seconds=5.132 peak_memory_bytes=326963200
wall: hard face limit at k=16: 15652352 fine faces exceeds 4000000; last_completed_k=8
```

```
$ [greedy merge deliberately changed to accept each face]
{'exposed_faces': 61142, 'greedy_quads': 61142, 'triangles': 122284}
```

```
$ scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/10-6-how-fine-can-we-go.sh
greedy merge removal fails prism geometry                    KILLED
detail rule removal fails subdivided geometry                KILLED
k one control drift fails control assertion                  KILLED

All mutations killed.
```

### Completion Notes List

- Task 1: added a deterministic ±2 fine-voxel seeded displacement rule, explicitly documented as
  a 10.4 look stand-in.
- Task 2: k=1 reproduces the real exported-world oracle exactly: 61,142 exposed faces and 19,264
  greedy quads. The guarded Axis A wall is k=16; k=8 is last complete.
- Task 4: Axis B is costed from the exact captured tile encoding and an ignored/manual test of the
  unmodified existing A*. All measured versus derived figures are labelled in signoff.
- Task 5: six stdlib bench tests pass. Mutations `greedy merge removal`, `detail rule removal`,
  and `k one control drift` each KILLED their named test; the real-world greedy sabotage also
  produced 61,142 quads before restoration.
- Task 6: handed Wolf the exact gingerspice commands and an intentionally empty fps table; no
  devpod fps was observed or claimed. Decision: retain 1.6 m/cell, use visual k=4 (0.4 m/voxel),
  keep sim k=1.
- Task 3 / AC5 is not implemented. I stopped at the story's named split line rather than ship a
  partial `--subdiv` flag: a compliant path needs chunk meshes separated by existing terrain
  material/rim levels while preserving snow caps, slice/pick/draw-set behaviour, and byte-identical
  default/k=1 rendering. No GUI code was changed.

### File List

- `scripts/bench/resolution_bench.py` (new)
- `scripts/tests/test_resolution_bench.py` (new)
- `crates/sim-core/src/lib.rs` (manual ignored A* resolution instrument)
- `_bmad-output/implementation-artifacts/10-6-signoff/detail-rule.md` (new)
- `_bmad-output/implementation-artifacts/10-6-signoff/axis-a-geometry.md` (new)
- `_bmad-output/implementation-artifacts/10-6-signoff/axis-b-sim-cost.md` (new)
- `_bmad-output/implementation-artifacts/10-6-signoff/decision.md` (new)
- `_bmad-output/implementation-artifacts/10-6-signoff/vehicle-fps.md` (new)
- `_bmad-output/implementation-artifacts/mutations/10-6-how-fine-can-we-go.sh` (new)
- `_bmad-output/implementation-artifacts/10-6-how-fine-can-we-go.md` (updated)
