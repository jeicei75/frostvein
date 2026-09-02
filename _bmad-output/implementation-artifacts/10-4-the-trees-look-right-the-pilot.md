---
baseline_commit: 2ef194d
model: claude-opus-5[1m]  # Opus default; the 1M-context variant, recorded so the ledger row is readable
---

# Story 10.4: The Trees Look Right (the pilot)

Status: ready-for-dev

**Runs after 10.6 and 10.3** (epic execution order 10.6 → 10.3 → 10.4 → 10.5, ruled by Wolf
2026-08-31). Both have landed: the resolution contract fixes 1.6 m/cell and the asset contract is
written and enforced by `check_asset.py`.

## Story

As the boss,
I want the trees redesigned through the bench until the valley's trees look like trees I chose,
so that the bench proves itself on the asset I am least attached to before touching the one I care
about most.

## Premises re-verified at creation — 2026-09-02

The epic orders its planning-time premises re-verified here. **Three of the six are wrong as
written**, and a seventh defect was found in `docs/tech-art-guidelines.md` while checking them.

1. **"The trim carries neither `bevy_gltf` nor `file_watcher`" — FALSE, and it was already
   corrected once.** `bevy_gltf` **is enabled**, transitively: the workspace turns on
   `3d_bevy_render`, and bevy 0.19.0's own `Cargo.toml` defines `3d_bevy_render = [… "bevy_gltf",
   "bevy_pbr", … "gltf_animation"]`. Only `file_watcher` is off. **Story 10.6 already found and
   recorded this** — `sprint-status.yaml`, the 10.6 block: "`bevy_gltf` IS enabled (transitively
   via 3d_bevy_render, Cargo.lock:754) -- only file_watcher is off". This story's first draft
   re-derived the epic's stale claim instead of reading that correction, which is the epic-premise
   trap firing inside the section written to catch it. **Consequence:** if the decision goes
   authored, the cost is *not* a feature-trim change with the never-two-Bevy-versions rule
   attached — it is asset plumbing (`AssetServer`, a shipped `assets/` location, deterministic
   variant selection), which is still out of scope here.
2. **"No devpod can open a window" — HALF FALSE, and the false half is load-bearing here.** No
   window, correct. But `/usr/share/vulkan/icd.d/lvp_icd.json` is present and story 10.6 ran
   `gui --headless --subdiv N --capture <png> --frames N` on real lavapipe, publishing live
   geometry figures. **Pixel evidence from the real client is producible in this devpod**; frame
   rate is not (lavapipe renders but does not clock). Do not scope this story as "no client
   evidence until the vehicle".
3. **"Blender 4.3.2" — STALE.** Two Blenders have lived here since 2026-08-31 and bare `blender`
   resolves to **5.2.1 LTS** (verified: `blender --version`). `valley_bench.py` stamps
   `blender=<version>` into every range-check line precisely because **figures are not comparable
   across the two** — measured on the same snapshot, `terrain_luma` moved 106.260 → 105.853 and
   `distinct_colors` 58,993 → 59,191. Every artifact this story produces must carry that stamp.
4. **`use_denoising = False` is mandatory — HOLDS.** Present in both `valley_bench.py` and
   `spike_pine_render.py`, each with the OpenImageDenoise failure recorded above it.
5. **"Eevee is ruled out; llvmpipe is the recorded dead end" — DECISION HOLDS, REASON IS WRONG.**
   Eevee is unavailable because **`libegl1` is missing**, not because of llvmpipe. Keep the
   decision (Cycles CPU only); do not repeat the reason.
6. **The epic's AC says "the `SPRUCE_SNOW` exposed-crown rule" — the NAME points at a superseded
   value.** `SPRUCE_SNOW` `(172, 186, 210)` was trimmed to `(156, 170, 196)` at round 7 (commit
   `10c06e1`). The rule that survives is `foliage_snow_color()`; 9.4's AC5 already uses that name.
   Read the epic's AC as "the exposed-crown rule", not as a claim about that constant.
7. **NEW DEFECT — `docs/tech-art-guidelines.md` states a foliage taper that does not exist.** The
   Materials table and the "Value and materials" section say `0.72 / 0.86 / full scale`. The
   client ships **`0.62 / 0.78 / 0.95`** (`foliage_scale`, asserted by
   `foliage_tapers_from_wide_mid_crown_to_narrow_tip_and_skirt`) and the bench matches it. The
   same round-7 commit `10c06e1` moved the taper, the snow cap and the crown together and updated
   no doc; a later commit documented only the crown. **Task 0 corrects it** — see Key decisions.

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier, not `--fast`) is green, and the diff is confined to this
   story's own commit range from `baseline_commit` — not `main..HEAD`.

### The decision this story exists to produce

2. Bench artifacts exist for **at least two tree treatments**: today's shipped trees as the
   control, and at least one tuned-procedural candidate. An authored-asset treatment is
   **optional**, not required.
3. Every published artifact carries its venue and its figures on one line — the
   `range-check: blender=<version> exposed_cells=… non_sky_fraction=… distinct_colors=…
   terrain_luma=…` line `valley_bench.py` already emits. An artifact without that line is not
   evidence. **A candidate's figures must differ from the control's**, `distinct_colors` among
   them; identical figures mean the two artifacts are one treatment photographed twice.
4. **The procedural-vs-authored decision is recorded in the story with the artifact and the figure
   it rests on**, naming who decided and when. Wolf's standing instinct is procedural (2026-08-28);
   the story records confirmation or overturn, not a restatement of the instinct.

### The winning treatment in the client

5. The winning treatment renders in the real client. Produce a `gui --headless --capture <png>
   --frames N` PNG at boot framing from `baseline_commit` **and** one from HEAD, commit both, and
   **state the pixel-difference count between them — it must be non-zero.** A PNG on its own
   proves nothing; 10.6 measured this capture as deterministic (0 of 2,073,600 values differ
   across two runs of one build), so a pixel diff is a discriminator already known to read zero
   when nothing changed. Note `--capture` **bails** without `--frames N` or `--at-tick N`.
6. The exposed-crown rule still holds: `foliage_snow_color()` remains the material of a spruce
   crown with nothing solid above it, and the existing crown tests pass untouched.
7. The landform-not-buried result still holds, in two halves that need two different guards:
   foliage receives no terrain-style snow slab — the client rule in `has_snow_cap`, guarded by
   `snow_caps_follow_material_and_exposure_in_a_seed_shaped_toy_world` — and the taper does not
   change which faces are exposed, guarded by the bench's
   `test_foliage_scale_does_not_change_which_faces_are_exposed`. The bench test knows nothing
   about snow slabs; do not cite it for that half.
8. **9.4's colour values are superseded or confirmed explicitly.** If `Material::TreeFoliage`
   moves, it still clears the shipped `MIN_MARK_SEPARATION` floor of 40.0 from stone, soil **and
   tree trunk**, and keeps `rgb[2] >= rgb[0]`; if it does not move, the story says so in one line.
   Trunk is in that list because the shipped guard
   `appearance_tables_pin_the_cold_boot_palette` iterates all three — trunk and foliage are the
   two materials of one drawn object, and the old foliage sat 38.7 from trunk. Meeting a
   stone-and-soil-only reading of this AC can still turn the suite red.
9. Any tree figure this story changes is updated **in every place that pins it**, and the places
   are enumerated because this project's recorded defect is a value living in four documents:
   the census in `test_the_exported_world_still_meshes_to_the_recorded_control`; `CONTROL_FACES`
   / `CONTROL_QUADS` in **`scripts/bench/resolution_bench.py`** (where 61,142 actually lives —
   the test reaches it through `assert_control`); and the draw-set oracle 44,984, which appears in
   **seven** live places: `crates/gui/src/project.rs`, `crates/gui/src/appearance.rs`,
   `crates/gui/src/ingest.rs`, `docs/tech-art-guidelines.md` (twice), `docs/tech-art-record.md`
   and `_bmad-output/planning-artifacts/epics.md`. Proof: `rg -n '44,984|44984' --glob '!target'`
   returns only current values.
10. **The taper the guidelines get wrong is corrected**, independently of whether any tree figure
    moves: `docs/tech-art-guidelines.md` states the shipped `0.62 / 0.78 / 0.95` in both the
    Materials table and the Value-and-materials bullet, and `docs/tech-art-record.md` records why
    it was wrong. Without this AC, skipping Task 0 leaves every AC and the gate green.
11. `_bmad-output/implementation-artifacts/mutations/10-4-the-trees-look-right-the-pilot.sh`
    carries at least **three rows that the mutation run kills**. `audit-mutations.py` on the gate
    checks that rows still apply; it does not notice a missing file.

### Sign-off

12. UX-DR22 **opening** half: Wolf approved a bench artifact before the client change was
    implemented. UX-DR22 **closing** half: Wolf has viewed the built result live on the vehicle
    and compared it against the approved artifact.

## Tasks / Subtasks

- [ ] **Task 0 — Correct the taper the guidelines get wrong** (AC: 10)
  - [ ] `docs/tech-art-guidelines.md`: Materials table row and the "Value and materials" bullet
        both read `0.62 / 0.78 / 0.95`, mapped **skirts and tips / upper crown / mid-crown** —
        0.62 covers the crown tip as well as the skirt. Note in
        `docs/tech-art-record.md` that round-7 commit `10c06e1` moved it with the snow cap and the
        crown, and that only the crown was documented at the time.
  - [ ] Do **not** touch `foliage_scale` itself. This task changes documentation only.

- [ ] **Task 1 — Bench the control and at least one candidate** (AC: 2, 3)
  - [ ] Control first: `python3 scripts/bench/export_world.py <snapshot.json>` then
        `blender --background --python scripts/bench/valley_bench.py -- <snapshot.json> <out.png>`.
        Paste the `range-check:` line into the Dev Agent Record.
  - [ ] Candidate treatments are produced by editing the **snapshot** or the bench's tree
        drawing, not by editing the client first. The bench exists so the client change is made
        once, after the decision.
  - [ ] **Candidate bench edits stay uncommitted**, or live in a scratch copy of `valley_bench.py`
        under `10-4-signoff/`. `bench_contract.rs` forbids a committed bench taper the client does
        not carry, so a committed candidate turns AC1 red and pushes the dev into exactly the
        client-first change the line above forbids. The lockstep edit happens once, in Task 3.
  - [ ] Artifacts land in `_bmad-output/implementation-artifacts/10-4-signoff/`.

- [ ] **Task 2 — Wolf judges, and the decision is recorded** (AC: 4, 12 opening half)
  - [ ] Present control and candidates side by side. Record the decision, the date, and the
        artifact filename it rests on.
  - [ ] **Stop here if the decision is authored.** An authored tree is a different story shape —
        see Scope guardrails — and needs its scope agreed before any client work.

- [ ] **Task 3 — Land the winning treatment in the client** (AC: 5, 6, 7, 8)
  - [ ] Both render paths must agree. At `--subdiv 1` every cell is one `Cuboid` spawned in
        `reconcile`; at `--subdiv > 1` **foliage stays per-cube while trunks go through the chunk
        mesher** (`build_chunk_meshes` opens with `if is_tree_foliage(..) { continue; }`). A change
        applied to one path only ships two different trees.
  - [ ] If `foliage_scale`'s literal changes, `crates/gui/tests/bench_contract.rs` requires a
        **coordinated Rust + Python edit**: it greps for the exact source text
        `match foliage_above { 0 => 0.62, 1 => 0.78, _ => 0.95, }` in `project.rs` and
        `return (0.62, 0.78, 0.95)[above]` in `valley_bench.py`, each matching **exactly once**.
  - [ ] Re-check `foliage_is_never_picked_and_never_hides_the_trunk_beneath_it` — picking assumes
        foliage is drawn sub-cell. A treatment that fills or overhangs the cell changes what the
        mouse can select.

- [ ] **Task 4 — Update every pinned tree figure** (AC: 9)
  - [ ] `CONTROL_FACES` / `CONTROL_QUADS` in `scripts/bench/resolution_bench.py` — 61,142 lives
        there, NOT in the test; editing the test alone produces a red from `assert_control`.
  - [ ] `test_the_exported_world_still_meshes_to_the_recorded_control` pins
        `{"tree_cells": 5048, "tree_faces": 13704, "terrain_cells": 39936, "terrain_faces": 47438,
        "trees": 265}` plus `exposed_faces = 61142`. The invariant `tree + terrain == total` must
        still hold, so this is a re-derivation, not a renumber.
  - [ ] The draw-set oracle comment in `reconcile` (44,984 of 301,048) and the same number in
        `docs/tech-art-guidelines.md`.

- [ ] **Task 5 — The instrument, a test of it, and the mutation rows** (AC: 3, 5, 11)
  - [ ] The instrument is `valley_bench.py`'s `range-check:` line for the artifact half and
        `gui --headless --capture` for the client half. Both already exist; cite them rather than
        inventing a third.
  - [ ] **The instrument must be shown to move.** The story-creation RED below proves
        `valley_bench.py` sees trees. Re-run an equivalent RED against the winning treatment: if
        the figures do not move between control and candidate, the artifact is not evidence of a
        tree change.
  - [ ] Author `mutations/10-4-the-trees-look-right-the-pilot.sh`, ≥3 rows, format per
        `mutations/10-1-the-headless-bench.sh`; run `scripts/mutate.sh` and record KILLED per row.

- [ ] **Task 6 — Verification** (AC: 1, 11, 12)
  - [ ] Execute the recipe below, RED first, and paste both outputs into the Dev Agent Record.

## Dev Notes

### Scope guardrails — do NOT

- **Do not build asset plumbing** unless Task 2's decision is *authored*, and then only after the
  scope is agreed. `bevy_gltf` is already compiled in (premise 1), so the missing piece is not the
  feature — it is that the client loads **zero** external meshes today: no `AssetServer`, no
  `SceneRoot`, no `assets/` directory anywhere in the repo. Authored trees are a new subsystem —
  a shipped asset location, a loader path, and a deterministic variant-selection rule — not a
  tweak.
- **Do not invent tree identity on the wire.** `protocol` carries trees as `Material::TreeTrunk` /
  `Material::TreeFoliage` tiles only — no tree entity, no origin cell, no variant index. A client
  that wants one mesh per tree must re-derive trunk columns itself. Presentation values never
  become wire state (AD-16).
- **Do not re-tune the light table or the snow cap** to make trees read better. Those were settled
  on measured captures; a tree story that moves them is two stories.
- **Do not touch `place_trees`' density.** 9.4 set 230–300 trunk columns on evidence and
  `tree_density_for_seed_42_is_deterministic_and_in_target_band` pins it. This story is about how a
  tree *looks*, not how many there are. **Tree height is the tempting exception and is not free:**
  `rng.random_range(4..=6)` draws from the same `STREAM_TREES` sequence as the 1-in-48 column
  roll, so widening it shifts every later column — expect the density band and the whole census to
  move together, and re-derive both rather than renumbering.

### What already exists

- **Sim:** `place_trees(dims, heights, tiles, camp, rng)` in `crates/sim-core/src/worldgen.rs` —
  height `rng.random_range(4..=6)`, ~1-in-48 columns with a 3-cell Chebyshev rejection, camp
  excluded, own RNG stream `STREAM_TREES`. Foliage is the tip cell plus 3×3-minus-centre rings on
  the two levels below; **there is no ground-level foliage ring** (removed by 9.4).
- **Client:** `foliage_scale` (0.62 / 0.78 / 0.95), `has_snow_laden_crown`, `rests_on_the_ground`,
  `is_tree_foliage`, `TerrainSlot::{TreeTrunk, TreeFoliage, FoliageCrown}` — all
  `crates/gui/src/project.rs`; colours in `crates/gui/src/appearance.rs`.
- **Bench:** `scripts/bench/export_world.py` spawns the daemon itself and writes a snapshot;
  `scripts/bench/valley_bench.py` renders it and prints the range-check line. Its own
  `foliage_scale` mirrors the client's and is pinned against it by `bench_contract.rs`.
- **Assets, if they are ever needed:** four deliverable pines under
  `_bmad-output/implementation-artifacts/10-2-signoff/export/`, footprints 3.4–5.4 m and heights
  6.4–10.6 m, all passing `scripts/bench/check_asset.py`. `tree.glb` in the parent directory is the
  superseded hand export and is a deliberate RED specimen — do not ship it.

### Key decisions and traps

- **The scale question is already answered — do not re-open it.** `docs/tech-art-guidelines.md`
  § Resolution contract fixes **1.6 m per simulation cell**, 0.1 m project voxel, 16 voxels/cell.
  A sim tree is 4–6 cells = **6.4–9.6 m**. The authored pines measure 6.4, 8.0, 8.0 and 10.6 m,
  so the ranges **overlap but are not the same**: three of four sit inside the band, and
  `SM_VoxelPine_Tree04` at 10.6 m is 6.6 cells — above `place_trees`' hard ceiling of 6, so it
  does not fit any tree the sim can generate. Scale is not a blocker; Tree04 needs a ruling
  rather than an assumption.
- **The bench's exit code does not discriminate — its figures do.** Measured at story creation:
  removing every tree cell moved `exposed_cells` 44,984 → 40,148, `distinct_colors` 59,191 →
  27,999 and `terrain_luma` 105.853 → 125.883, and the run still **exited 0 and passed its range
  check**, because the floors sit at 0.02 / 32 / 20.0. Judge the figures. Exit 0 is not a result.
- **Two tree counts appear in this story and both are right.** 5,582 is every tree tile in the
  snapshot; the census's 5,048 is the *exposed* subset; the exposed-cell delta is 4,836
  (44,984 → 40,148) because removing a tree also exposes the terrain that was under it.
- **`distinct_colors` is the most tree-sensitive figure by far** — it halved when the trees went.
  It is the figure a tree treatment should be expected to move.
- **`bench_contract.rs` is a source-text grep, not a behaviour test.** Renaming or deleting
  `foliage_scale` breaks the suite even if nothing renders differently.
- **Known neighbouring bug, out of scope but likely to surface at review:** `opening_z` in
  `crates/tui/src/view.rs` counts tree foliage as standable ground. Recorded in
  `deferred-work.md`; do not fix it here, and do not be surprised by it.

### Project structure

| Path | NEW/UPDATE | Note |
|---|---|---|
| `_bmad-output/implementation-artifacts/10-4-signoff/` | NEW | artifacts + the approved one, named |
| `docs/tech-art-guidelines.md` | UPDATE | Task 0 taper correction; tree figures if they move |
| `docs/tech-art-record.md` | UPDATE | why the taper was wrong |
| `crates/gui/src/project.rs` | UPDATE | only if the winning treatment is a client change |
| `crates/gui/src/appearance.rs` | UPDATE | only if 9.4's colours are superseded |
| `scripts/bench/valley_bench.py` | UPDATE | only in lockstep with `project.rs`, per `bench_contract.rs` |
| `scripts/tests/test_resolution_bench.py` | UPDATE | census, if tree geometry moves |
| `scripts/bench/resolution_bench.py` | UPDATE | `CONTROL_FACES` / `CONTROL_QUADS` live here, not in the test |
| `_bmad-output/implementation-artifacts/mutations/10-4-the-trees-look-right-the-pilot.sh` | NEW | ≥3 rows |

## Verification

**Executed at story creation, 2026-09-02, on `2ef194d` — both halves observed.**

RED first. The instrument must be shown to see trees before any green from it is believed.

```bash
export PATH="$HOME/.local/share/mise/installs/rust/1.97.1/bin:$PATH"
python3 scripts/bench/export_world.py /tmp/world.json
# RED: strip every tree cell from the snapshot, then render the same world
python3 - <<'PY'
import json
d=json.load(open("/tmp/world.json"))
d["tiles"]=[({} if isinstance(t,dict) and t.get("solid") in ("tree_foliage","tree_trunk") else t)
            for t in d["tiles"]]
json.dump(d, open("/tmp/world-notrees.json","w"))
PY
blender --background --python scripts/bench/valley_bench.py -- /tmp/world-notrees.json /tmp/red.png
blender --background --python scripts/bench/valley_bench.py -- /tmp/world.json       /tmp/green.png
```

Observed, verbatim:

```
RED    range-check: blender=5.2.1 exposed_cells=40148 non_sky_fraction=0.662805 distinct_colors=27999 terrain_luma=125.883
GREEN  range-check: blender=5.2.1 exposed_cells=44984 non_sky_fraction=0.686736 distinct_colors=59191 terrain_luma=105.853
```

5,582 tree cells removed; every figure moved. **Note what the RED also proves: it exited 0 and
passed its range check.** The floors (`non_sky_fraction 0.02`, `distinct_colors 32`,
`terrain_luma 20.0`) are far too loose to fail on a world with no trees at all. The discrimination
lives in the printed figures, and any AC resting on this instrument must read them.

Restore: nothing to restore — the RED is a separate snapshot file, the tree is untouched.

Full gate at creation: **GREEN**, all 9 checks `ok`.

## References

- `_bmad-output/planning-artifacts/epics.md` § Epic 10, § Story 10.4, § UX-DR22
- `docs/tech-art-guidelines.md` § Resolution contract, § Value and materials, § Critical values
- `_bmad-output/implementation-artifacts/9-4-trees-fewer-and-distinct-from-the-ground.md` (ACs 2–6)
- `_bmad-output/implementation-artifacts/deferred-work.md` (`opening_z`; the gfx-era revisit)
- `CLAUDE.md`, `docs/technical-preferences.md`

## Change Log

| Date | Change |
|---|---|
| 2026-09-02 | Story created. Premises re-verified: 3 of 6 wrong, plus a stale taper found in the guidelines. Verification recipe executed RED-then-GREEN on `2ef194d`. |
| 2026-09-02 | Fresh-context validation, 13 findings, all applied. Two were critical: premise 1 was FALSE (`bevy_gltf` is enabled via `3d_bevy_render`, already corrected at 10.6 and re-derived here anyway), and Task 1's method would have turned AC1 red through `bench_contract.rs`. AC5 and AC3 gained discriminators, AC7's two halves were split onto the two guards that can actually see them, AC9 now enumerates all seven homes of the draw-set oracle, and the taper correction and the mutation rows gained ACs of their own. |

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
