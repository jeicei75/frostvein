---
baseline_commit: 110cab4
model: claude-opus-5-5  # session default, same as 12.1-12.7's creation
---

# Story 12.8: Dwarves at Work — The Cut

Status: done

## Story

As the boss,
I want to see a woodcutter swing at the tree, and see cut mode, work and wood behave where my eye is,
so that felling reads as work like digging does, and I can watch dig, haul and cut together.

## Not stacked: branch off `main`

`main` is `110cab4` (PR #171, 12.7, merged). Branch `story-12-8-dwarves-at-work-the-cut` carries:
- `53f6097`: 12.7 marked `done` (its `4` clear was seen at the seat after the merge);
- `486095d`: the fold of #164, #173 and #174 into this story (`epics.md`, Story 12.8, "Folded in");
- this file, `12-8-signoff/` and the board edit.

Epic 12's nine standing ACs (`epics.md:2422-2434`) bind this story and are not restated. **No wire change**
(standing AC 4): cutting is already on the wire as `state: work` + `job: {"cut":{"target":[x,y,z]}}`
(`protocol/src/lib.rs:110`, `simd/src/bridge.rs:195`). Standing AC 7 applies to #174 only: Wolf approves
`12-8-signoff/draft.md` at Task 0 before any of it is built. The cut swing skips the draft and goes straight
to the seat.

## Found at creation (2026-10-06, on `486095d`)

- **The woodcutter plays the Dig clip.** `dwarf_clip` (`gui/src/project.rs:429-445`) returns `DwarfClip::Dig`
  for `Work` on `Dig | Channel | Cut`, with `// NOTE: placeholder until 12.8's Cut clip` (`:427`). Observed
  live at creation (see Verification): Nain logs `gui dwarf 0 clip dig` while the wire says `job cut`.
- **`CUT_WORK_TICKS = 50`** (`sim-core/src/lib.rs:52`, `// NOTE: tuned with 12.8's clip`), equal to
  `DIG_WORK_TICKS`. A cut lasts 5 s at Normal, 1.0 s at Fast and 0.25 s at Fast4x (`simd/src/main.rs:20-23`).
  The swing period is `WORK_SWING_TICKS = 5` (`project.rs:454`), so it must divide the work ticks.
- **#164 is live in today's code** (measured in the 12.5 review, seen again at the 12.7 seat with logs).
  - The drawn walker moves at `DWARF_WALK_CELLS_PER_SECOND = 0.9` cells per second of WALL time
    (`project.rs:639`, step at `:2649`). It does not scale with sim speed, so it trails the wire.
  - Pick-up is ungated: `carried.insert` runs on `entity.carrying` alone (`project.rs:499-500`), so a log or
    stone jumps into the hauler's hands before his drawn body arrives.
  - Release is gated on `walking_in` for his CURRENT wire cell, not the drop cell (`:559-571`).
  - At Fast, one cell of walk (~1.1 s) outlasts a whole cut or dig (1.0 s). 25 wire dig runs gave 5
    `clip dig` lines (issue body).
- **#173 is NOT reproduced, and its first premise was wrong** (issue comment, 2026-10-06).
  - The ghost in Wolf's seat video is the **pine mesh drawn in flat green** with no snow, not the cube
    fallback. It is the marked `Tree04R` at base (69,75,14).
  - Clean every time:
    - wire streams replayed through `projection_systems` at subdiv 4 and 1 (fell A, fell the touching pair
      A+B, mark+cancel): live `TreeMesh` set == fresh `tree_meshes`, 0 cube-drawn tree cells;
    - a live lavapipe session felling five pines;
    - a per-frame audit of every `TreeMesh` hierarchy: always exactly 4 material handles, one per GLB.
  - The only per-entity material writer today is `apply_dwarf_tunics` (`project.rs:3511-3606`), which reaches
    dwarf hierarchies only.
  - Untested: a render-world stale binding on the 4080, hot reload under `--assets`, a save/load in that
    session.
- **#174's mechanisms** (from the code and the video):
  - **Flicker.** The pick skips foliage (`pick.rs:337-339`, `is_tree_foliage` `project.rs:2814`), so a ray
    through a crown lands on ground 8-12 cells behind the tree at the boot pitch (0.45 rad, `camera.rs:14`).
    Sweeping across one pine alternates between the trunk's side face and that far ground, and the cut
    preview rect jumps with it (`preview_cells` takes x,y from the release, `project.rs:948-951`).
  - **No box over open ground.** The cut preview keeps tree tiles only (`sim_will_keep`, `project.rs:976`),
    so a drag over open ground draws nothing.
  - **The release side is already right.** `commands_for(Cut)` sends one `Designate{Cut, rect}`
    (`designate.rs:303-306`), and the sim keeps one mark per distinct tree base and drops non-tree cells
    (`sim-core/src/lib.rs:1768-1772`).
  - **A marked pine looks unmarked** apart from a `mark_mesh` slab at its base (`project.rs:2202-2211`,
    colour (30,190,150) `appearance.rs:208`). The mark's cell is exactly `TreeMesh(base)`'s key.
  - **Each pine GLB** (`assets/trees/*.glb`) is 1 mesh, 1 primitive, one material `M_VoxelPine` with an
    atlas texture.

## Acceptance Criteria

1. In `work` on a cut job, once drawn at his cell, a woodcutter plays a `Cut` clip bound by NAME from the
   embedded GLB, in swings timed in delivered ticks, following pause and speed as Dig does (FR45). Dig and
   channel keep `Dig`.
2. The embedded GLB carries `Walk`, `Dig`, `Carry` and `Cut`. The startup line reads
   `clips Walk, Dig, Carry, Cut`, and a missing clip reads `clip Cut ABSENT`.
3. A hauler's log or stone stays on the ground until his drawn body reaches its cell, and stays in his hands
   until he is drawn at the cell where the wire drops it (#164, effects 1-2).
4. At Fast, the drawn dwarf is at his work cell for the work: a cut or dig run at Fast produces a work clip
   line for it (#164, effect 3; the shape per Task 0.3).
5. #174, per the approved draft:
   - in cut mode, a ray through a crown resolves to that tree, not the ground behind it;
   - while dragging, the cut preview shows the whole box;
   - a marked pine shows the approved tint.

   Dig, channel, stockpile and clear keep today's pick.
6. The gui reports every pine's material: a `gui trees: materials=` line names the distinct material handles
   in use, and how many pines carry each (#173's instrument). A cut-tinted pine is counted as such.
7. The instrument is the real `gui` binary. A real-binary test against a real `simd` sees the woodcutter log
   `clip cut` and later `clip walk`, while miners still log `clip dig` and haulers `clip carry`. Its
   deliberate RED is `dwarf_clip` mapping Cut back to Dig.
8. TUI: no change and no regression (NFR10).
9. At the seat, in one sitting, Wolf watches dig, haul and cut happen at the same time and judges each
   animation. Each art round gets a row in "Art ledger". The story is done when Wolf judges them read, or
   rules ship-plain or park after two rounds his eye has not judged converging (FR45, M2-24). He also judges
   #174's look against the draft, and the #164 timing at Normal and at Fast.

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings, at creation.** Ruled 2026-10-07, all four as recommended:
  0.1 draft §1 and §2 approved, tint (a) repaint, foot slab kept; 0.2 swing period 10 ticks (5 chops per
  50-tick cut, `WORK_SWING_TICKS` per-clip); 0.3 (a) the walker's speed scales with the sim speed; 0.4 #173
  stays instrument-only. The seat-session question (save/load, `--assets`, clock) was not answered.
  The original asks:
  1. **The draft** `12-8-signoff/draft.md` (#174):
     - §1 pointer, §2 whole box: approve or amend;
     - §3 tint: option (a) repaint the pine green (recommended), (b) a glow, or (c) cubes over its cells;
     - keep the foot slab (recommended) or drop it.
  2. **The chop:** keep 50 ticks of work per tree. The swing period is either 5 ticks (10 chops, like dig),
     or 10 ticks (5 slower axe chops, recommended: an axe reads heavier than a pick). `WORK_SWING_TICKS`
     becomes per-clip if 10.
  3. **#164 at Fast:**
     - (a, recommended) the drawn walker's speed scales with the sim speed (legs stay phase-locked to the
       ground, so they speed up like a fast-forwarded film);
     - (b) snap him to his work cell when work starts while he trails;
     - (c) accept, and judge the swings at Normal only.

     Effects 1-2 (pick-up/drop gating) are fixed in every case.
  4. **#173:** keep the instrument-only scope (AC6: a fix only from a reproduction), or take #173 out of 12.8.
     Also: did that seat session save/load, use `--assets`, or change the clock?
- [x] **Task 1: #164, RED first (AC3, AC4).** Headless (`crates/gui/tests/headless.rs`, MinimalPlugins,
  `ManualDuration(100ms)`), following `a_hauler_keeps_his_stone_until_he_is_drawn_at_the_cell_he_drops_it_on`
  (`:5338`).
  - [x] RED first:
    - pick-up: one delta steps the hauler into the item's cell AND sets `carrying`; after one `app.update()`
      the item is NOT parented to him. Record the failure;
    - Fast: feed one 50-tick work run at Fast's tick rate and assert the dwarf reaches his work clip within
      the run. Record the failure.
  - [x] Gate the pick-up on `drawn_at_cell` (`project.rs:499-500`), as the drop already is, and gate the drop
    on the cell the wire drops at, not his current cell (`:559-571`).
  - [x] Implement Task 0.3's ruling for Fast. If (a), the step at `:2649` multiplies by the tick-rate factor
    the clock already knows (`TickClock::factor`, `:459`); keep `DWARF_WALK_SNAP_CELLS` (`:644`) as is.
  - [x] Both tests GREEN; the old hauler test stays GREEN.
- [x] **Task 2: the Cut clip, from a live BlenderMCP seat (AC1, AC2).** Tasks 1, 3, 4 and 5 do not need it;
  run them while the seat is pending.
  - [x] **2a — write the brief** `src-assets/prompts/dwarf-miner-round-20.md`, in round 19's shape
    (`dwarf-miner-round-19.md`): the "For:" header, the in-place rule, LINEAR keys at 24 fps on frames 0..24
    with frame 24 == frame 0, the action saved unassigned with slotted `fcurves()`, one tool call per pose
    stage with a screenshot each, and the generator script written as it goes. Content:
    - **one `Cut` action**: a two-handed axe chop at a trunk standing one cell ahead, with the period Task 0.2
      rules;
    - the axe is a new prop on the existing rig, following round 19's 20th `pick` joint, OR the pick
      re-used: the seat decides, and the brief says so;
    - Walk, Dig and Carry are untouched;
    - the report goes to `dwarf-miner-round-20-report.md`.
  - [x] **2b — Wolf runs the seat**, watching live. Art hard stop: after two rounds his eye has not judged
    converging, stop and ask him (ship plain or park). One "Art ledger" row per round: model id, his `/cost`,
    his words, verdict.
  - [x] **2c — export, check, promote.**
    - `blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend --python src-assets/blender/export_dwarf.py`
      (it writes to the gitignored `src-assets/export/`).
    - `check_asset.py` must print four clips, for example
      `anims=Carry:60ch@1.00s,Cut:…,Dig:60ch@1.00s,Walk:60ch@1.00s`. Its loop check (`check_asset.py:463`)
      assumes every clip loops, so Cut loops. No stray action.
    - Promote to `assets/gltf/SM_VoxelDwarf_Miner01.glb` in its OWN commit, and re-run `check_asset.py` on
      the promoted file. Rebuild the gui (`include_bytes!` at `ingest.rs:421-423`).
- [x] **Task 3: the gui binds and plays Cut (AC1, AC2).** Add Cut beside Dig at every site:
  - [x] `DwarfClip` and `label()` (`project.rs:404-418`); `DwarfClips` field, `node()`, `nodes()`
    (`:335-375`); the graph build `load("Cut")`, `present`, `take` order (`:765-786`);
  - [x] `dwarf_clip` returns `Cut` for `Work` + `Cut{..}` and drops the placeholder NOTE (`:427-445`);
  - [x] the arrival gate and the swing-phase arm treat Cut like Dig (`:510-532`, `:2481-2484`), with Task
    0.2's period; the STALLED loop gains Cut (`:2450`); `dig_yaw` already faces a cut target (`:2532-2543`);
  - [x] `DWARF_CLIP_NAMES` (`ingest.rs:490`) and its report and absent tests (`:5992`, `:6031-6035`), the
    `README.md:313` literal and the `clip <walk|dig|carry>` README text;
  - [x] rename `a_woodcutter_on_a_cut_job_in_work_gets_the_dig_clip` (`headless.rs:5859`) to `…_the_cut_clip`;
    `the_embedded_dwarf_carries_walk_dig_and_carry_by_name` (`ingest.rs:5960`) covers all four;
  - [x] if Cut is missing from the GLB, Cut falls back to Dig. The startup line says `clip Cut ABSENT`, the
    same shape as the others.
- [x] **Task 4: #174 cut mode, per the approved draft (AC5).** RED first for the pick and the box; the tint is
  judged at the seat.
  - [x] **Pick:** in cut mode only, a ray entering a foliage cell resolves to that tree, the same answer
    `cut_target` gives for its trunk (`designate.rs:272`). Pure test with `pillars()` (`pick.rs:490`) and
    `ray_at` (`:699`): sweep rays across one crown at pitch 0.45. In cut mode, every hit stays within that
    tree's 3×3 column; in dig mode, today's fall-through is unchanged. RED first.
  - [x] **Box:** the Cut arm of `preview_cells` returns the whole rect at the cut level. `preview_appearance`
    draws non-tree cells in the hover material and the caught trees per the draft. Extend
    `the_cut_preview_lights_only_tree_tiles_at_the_cut_level` (`project.rs:3771`): its new name and asserts
    say "every rect cell is previewed; only tree cells in the cut style". The release still sends one
    `Designate{Cut, rect}` (unchanged).
  - [x] **Tint**, per Task 0.1. For (a), follow `apply_dwarf_tunics` (`project.rs:3511-3606`):
    - a marker on a `TreeMesh` whose base carries a cut designation, set and cleared from
      `mirror.designations()` (`client-core/src/lib.rs:181`);
    - on `Added<MeshMaterial3d>` and on `Changed<marker>`, swap in ONE shared tinted clone of `M_VoxelPine`,
      and swap the original back when the mark goes;
    - insert or replace handles only, never remove a component (`remove_with_requires` is banned,
      `ingest.rs:2091`);
    - re-apply on respawn, because the incremental tree path respawns `TreeMesh` (`project.rs:3044`).

    Headless test: mark a tree, and its mesh carries the tinted handle; clear the mark, and it carries the
    original. The incremental path is covered by felling the marked tree's neighbour.
- [x] **Task 5: #173's instrument (AC6).** Once the scene reports trees loaded, the gui prints
  `gui trees: materials=<n> [<label>:<count>, …]`, and again whenever the count changes. `<label>` is the GLB
  material name or `cut-tint`. A probe of this shape ran clean at creation (4 handles, per variant).
  - [x] Test: a headless app with two marked pines reports `cut-tint:2`, and the line changes when one mark
    is cleared.
  - [x] (Not triggered: no ghost was reported at either seat pass.) If Wolf reproduces the ghost at the seat (Task 7), file the line in #173. Do NOT fix #173 in this
    story without a red reproduction (M2-27). If a fix is found, it gets its own task with Wolf's yes.
- [x] **Task 6: the record.**
  - [x] `mutations/12-8.sh`, at least:
    - Cut mapped back to Dig in `dwarf_clip` (killed by AC7's real-binary test and the headless test);
    - pick-up ungated;
    - Fast walker unscaled (if 0.3(a));
    - cut-mode crown fall-through restored;
    - the box filtered back to trees;
    - the tint never cleared;
    - the materials line printing a constant.

    Commit the GLB BEFORE any row runs: `mutate.sh` does not back up `assets/` (`mutate.sh:71`).
  - [x] Re-point old rows the change breaks (12.5 rows on `dwarf_clip`, the `clips Walk, Dig, Carry` literal).
    Run them, and every row must be KILLED.
  - [x] README `gui` section: the Cut clip and the clip list, cut-mode pick/box/tint, the `materials=` line.
  - [x] Seat card `12-8-signoff/vehicle-card.md` in the seat's launch form (see Verification).
- [x] **Task 7: the live recipe, the seat (AC9), then the full gate.** `RUST_TEST_THREADS=1 scripts/gate.sh`
  on the final HEAD.

### Art ledger

| Round | Model | Wolf's `/cost` | Wolf's words | Verdict |
| --- | --- | --- | --- | --- |
| 20 (`Cut`, axe) | `claude-opus-5-5` (Claude Code + BlenderMCP, Wolf's live seat) | **$21.59** (Opus 5.5 $21.59 + Haiku 4.5 $0.001; 203 requests, API 47m 43s, wall 2h 31m) | "option B with proper axe"; "axe head is rotated to wrong way in the start (backwards) and in the end" (fixed); "when axe is high the head edge should point either straight to front or a little bit down not to left" (fixed); two hands out of reach on this rig: "finish it .. we can use this now.. have to rethink the model and rig anyway at some point" | **Accepted at the Blender seat**, one-handed: bite at 0.95 m plus a 0.35 m client nudge (`CUT_OFFSET`); 21 joints. Judged in game at the seat (Task 7). |

### Review Findings

Code review run 1, 2026-10-07, on `8779499` (diff `origin/main...HEAD`, 110cab4..8779499; 7 crate files plus
README, mutation files and export scripts, 2142 lines). Four layers ran and none timed out. Blind Hunter
(`appearance.rs`, `project.rs`) and Edge Case Hunter (`pick.rs`, `ingest.rs`, `build.rs`, `tests/`,
mutations) ran on Sonnet with the R1 territories. The Acceptance and Feature Auditors ran on Opus over the
whole diff. Every layer's cargo ran (1.97.1), and each built in its own target dir. Live runs:
- The Feature Auditor ran a release `simd` + `gui --headless` for 410 s with a channel, a pile and cuts on
  A, then B at Fast. It saw:
  - `clips Walk, Dig, Carry, Cut`;
  - `materials=4 [M_VoxelPine:259]`, then `materials=5 [M_VoxelPine:258, cut-tint:1]` on the mark, and back
    to 4 on the felling (A and B);
  - at Fast, `gui dwarf 0 clip cut` then `clip walk`, and the miners `clip dig` then `walk`.

  There was no STALLED line. It also parsed the embedded GLB: 4 clips at 63 channels and 21 joints with
  `axe`. None of the 252 channels targets the nudged armature node.
- The Acceptance Auditor saw the same startup, materials and clip lines on its own daemon. The AC7
  real-binary test PASSED (116.7 s, `woodcutters cut [(0,1)] walk [(0,1)]`). Six lib tests and eleven
  headless tests passed. `check_asset.py` on the promoted GLB: 4 anims, 21 joints, exit 0.
- The Edge layer checked that every mutation row in the directory still applies (no APPLY-FAILED). It
  did not re-run kills.
- The diff under `sim-core`, `simd`, `protocol`, `tui` and `client-core` is empty (AC8, standing AC 4).

Not proven by this review: the look halves of AC1/3/4/5 and AC9. The record's only seat words are
"1 ok 2 ok" (decision below). The full gate is last green on `f7a1649`; HEAD differs from it by two record
commits only. Review cost $19.23 over 307 turns (Opus $16.53, Sonnet $2.70; subagents 75.1% of tokens).
`reap-build-caches.sh --tmp-only --force` reclaimed 26.3 GB. No HIGH defect. Tally: 1 decision-needed, 1 patch, 7 defer, 12 dismissed. Patch pass 2026-10-08: the one patch LANDED `ef5dfb0`, full gate GREEN. Layer and severity
are in brackets.

**Process note.** The Edge layer `exec`'d every mutation payload with `p.write_text(` stubbed out, against
the preamble's ban and [[mutation-payload-exec-writes-tree]]. `git status` was clean afterwards, so no harm
was done. A payload that writes any other way would have mutated the tree.

- [x] [Review][Decision] **AC9's #164 judgement ("timing at Normal and at Fast") has no unambiguous verdict
  on record** (acceptance + feature, MED; record). `12-8-signoff/vehicle-card.md` asks three questions: Q1
  cut mode, Q2 the chop with dig, haul and cut together, Q3 "does the timing hold at Normal and at Fast?".
  The only words recorded are "1 ok 2 ok" (`:524`), read as passes 1 and 2, so Q3 has no explicit
  answer. The card was also never revised for pass 2: it still says Nain plays the Dig clip and does not
  mention `CUT_OFFSET`. Options: (a) Wolf confirms Q3 was seen at both speeds, and the record says so;
  (b) Q3 stays OPEN until a short seat look; (c) accept as covered by pass 2.
  — RESOLVED (Wolf, 2026-10-07, option a): "yes timing is fine with all speeds". Q3 was judged at the seat
  and AC9's #164 half is closed. The stale card text is left as the pass-1 record it was.
- [x] [Review][Patch] **In cut mode a TRUNK hit does not move the highlight to the pine's foot**
  (feature + acceptance, MED; the edge layer's weak-test finding is merged in) [crates/gui/src/pick.rs:341].
  - Draft §1 and the README both promise "point at any part of a pine, crown or trunk, and the highlight
    sits at that pine's foot".
  - Only foliage goes through `tree_foot`. A trunk cell returns at `:342` as itself with a side face, so a
    sweep across a pine still hops between the foot and the trunk's side, which is #174's flicker at a
    smaller scale. The release is already right (`cut_target` of a trunk is the base).
  - Fix: in cut mode, route any tree tile (trunk or foliage) through `tree_foot`.
  - Strengthen `in_cut_mode_a_ray_through_a_crown_resolves_to_that_tree`:
    - every cut hit that lands on the tree must equal the foot cell exactly, instead of "within the 3×3
      column, `feet > 0`". Today a foliage fallback passes, and so does a trunk face;
    - add rays aimed at the trunk.
  - RED first, then re-mutate the "cut-mode crown fall-through restored" row
    ([[strengthened-test-needs-remutation]]).
  - **LANDED `ef5dfb0`** (patch pass, 2026-10-08). In cut mode any tree tile (`is_tree_tile`, or
    `is_tree_foliage` so Ramp foliage keeps its old route) goes through `tree_foot` BEFORE the foliage
    fall-through; other modes are unchanged. The test now asserts, for every ray, `([60,60,1], Top)`
    and `cut_target == [60,60,2]`. It adds rays at the bare trunk `[60,60,2]` and `[60,60,3]`, and a
    positive count proving that dig-mode rays reach a `TreeTrunk` tile (`dig_trunk > 0`).
    **CORRECTED by review run 2:** only `[60,60,2]` reaches the trunk first (37 of 40 rays).
    `[60,60,3]` is covered by the crown ring at z 4 on all 40. `dig_trunk` could not fire, because
    dig sees through the crown. Both are fixed in the run-2 patch pass below (`73c7d45`).
    - RED before the fix: `left: ([60, 60, 2], South) right: ([60, 60, 1], Top)`, yaw -1, target
      `[60, 60, 2]`.
    - Mutations (`RUST_TEST_THREADS=1 scripts/mutate.sh` on these two rows of `12-8.sh`): **2/2 KILLED**.
      "cut-mode crown fall-through restored" is re-anchored to the new guard. The new row "cut-mode
      trunk hit keeps its side face" puts back exactly the pre-patch foliage-only guard. Exclusivity
      is not checked.
    - Full gate GREEN on `ef5dfb0` (`RUST_TEST_THREADS=1 scripts/gate.sh`, 3344 s, pixel guards included).

  | Item | Fix written for | Then tested | Pre-existing-state fixture |
  | --- | --- | --- | --- |
  | Trunk hit → foot | a ray meeting the bare TRUNK first | crown rays (the side already handled) must still give the exact foot cell, and dig mode must still fall through | `pine()`: the round-1 fixture, unchanged (trunk z 2..=5 at (60,60), 3×3 crown z 4..=6, stone floor), the same world the round-1 RED ran on |
- [x] [Review][Defer] **AC7's real-binary test proves `clip cut` only with the sim held 30 s; at Normal on
  lavapipe the cut is not drawn** [crates/gui/tests/pixel_guard.rs:1084] (feature + acceptance + edge, LOW;
  RAN). The Feature Auditor's un-held release run, on a box loaded by three sibling builds, logged no
  `clip cut` for cut A at Normal and did log it at Fast. This is #175 exactly; the issue is the state.
- [x] [Review][Defer] **The pick-up and drop gates are frame-sampled** [crates/gui/src/project.rs:212,231]
  (blind + feature + acceptance, LOW; read).
  - Pick-up keys on his CURRENT wire cell, not the item's.
  - Drop needs a frame inside 0.1 cell of the drop cell. At Fast4x (0.3 cell per 60 fps frame), below
    ~4 fps, or on a pursuit path that cuts the corner, the stone rides on to his wire cell and snaps back.
  - Not reachable at Normal or Fast at 60 fps: a hauler stands about 17 ticks on the cell, and the walker
    needs about 11. Visible, never stuck.
- [x] [Review][Defer] **On release, the drag-tinted pines go snowy for one delta before the wire's Cut mark
  re-tints them** [crates/gui/src/project.rs:587] (feature + acceptance, LOW; read; draft §2 says "stay
  tinted"). `DragMode` clears on release, and the mark arrives at most 100 ms plus a frame later.
- [x] [Review][Defer] **Look departures from draft §2/§3 that Wolf passed at the seat ("1 ok")**
  [crates/gui/src/project.rs:666,1050] (acceptance + blind, LOW; record).
  - The tint drops the atlas texture, so the whole pine, trunk included, is flat foliage green. §3(a)
    said "keeps … voxel texture" and also "flat … green".
  - The box slabs sit at the cut level, not "on its ground" (Task 4's wording won over the draft's).
- [x] [Review][Defer] **#173's `materials=` line counts pines per LABEL, not per handle**
  [crates/gui/src/project.rs:506] (acceptance, LOW; RAN, and disclosed in Task 5). All four GLBs name their
  material `M_VoxelPine`, so a same-named cross-variant handle swap is invisible. A ghost-style flat
  material still shows as a 5th handle or a new label.
- [x] [Review][Defer] **Fast2x/Fast4x walker ratios have no test or mutation row; the Fast test passes any
  ratio ≥ ~1.12, and no test walks a woodcutter in and asserts no `Cut` before arrival**
  [crates/gui/src/project.rs:250] (edge + acceptance, LOW; read).
- [x] [Review][Defer] **Two new per-frame systems are unmeasured (standing AC 9 / NFR6)**
  [crates/gui/src/project.rs:477,573] (acceptance, LOW; read). `report_pine_materials` walks every
  `MeshMaterial3d` parent chain and allocates a label per pine mesh every frame, and `sync_cut_tint_marks`
  scans about 265 `TreeMesh`. The seat raised no fps complaint.

Dismissed (12):
- `tree_base_at` picks the wrong neighbour (blind), Ramp foliage (blind), and the tint/mark divergence on
  ring or split trunks (acceptance). Unreachable: worldgen keeps trunks ≥ 3 apart (`worldgen.rs:313`) and
  rings at `top-1..top` (`:342`), the same invariant as sim `tree_of`.
- `ArmatureRest` overwritten by animation (blind): no channel targets the node, checked live in the GLB.
- The 8-level ancestor cap (blind) and the acknowledged pop (blind).
- The materials report is not reset on empty (blind).
- `sim_will_keep` callers (blind): `preview_cells` is the only one.
- Missing Cut visible only at startup (edge): AC2's design.
- Trunkless foliage (edge): unreachable.
- Mode switch mid-drag (feature): a contrived input.
- Debug-vs-release simd for the AC7 test (acceptance): a preamble error, not code.

#### Code review run 2, 2026-10-08, on `679e55a` (diff `712cd48..679e55a`, the run-1 patch only)

**Delta vs run 1:** 5 kept findings, all NEW to this diff. 1 is REWORK: the run-1 patch's test strengthening
is half-closed. Severity: 0 HIGH, 0 MED, 5 LOW. **Stopping rule: no HIGH, so the static audit ENDS.** The next
spend goes to the seat (trunk hover at the foot), not to a run 3.

Four layers ran, none timed out, and cargo 1.97.1 ran in each layer's own target dir. `pick.rs` sits outside
Blind's R1 territory, so Blind got the same code hunk without the spec rather than an empty share. Edge got
`pick.rs` and the mutation rows, and the Opus auditors got the whole diff. Live runs:
- The Feature Auditor drove the real `configure_client_app` in-process in a `/tmp` copy, with a window,
  a camera at pitch 0.45, a TCP socket and `ButtonInput`:
  - aim at the bare trunk `[5,5,2]`: the dig pick is `[5,5,3] East`; after `5` the pick is `[5,5,1] Top`
    and the hover slab sits at `(5.0, 1.55, -5.0)`;
  - a click sends `designate cut` at `[5,5,2]`;
  - a drag from the trunk anchors at the foot and tints 1 pine.

  Every hop from the cursor to the wire is WIRED. The sim hop was read only.
- Edge and Acceptance re-applied both mutation rows by hand in `/tmp` copies: both were KILLED, and the
  RED string reproduced exactly. Nobody ran `mutate.sh` or a payload.

Not proven: the on-screen hover at the foot (the headless `--cursor` has no `PrimaryWindow`), and the
#174 feel at trunk scale. Those are seat items.

Wolf chose option 2: the two patches are LEFT AS ACTION ITEMS for a fresh-session patch pass. Story and board
are `in-progress`. Review cost $12.73 over 244 turns (Opus $10.75, Sonnet $1.98; subagents 67.5% of tokens).
`reap-build-caches.sh --tmp-only --force` reclaimed 52.0 GB. Tally: 0 decision-needed, 2 patch, 3 defer,
7 dismissed. Patch pass 2, 2026-10-08: both patches LANDED (`73c7d45`, `3d7f117`, mutation row `de6ff85`).
Full gate GREEN on `de6ff85` (`RUST_TEST_THREADS=1 scripts/gate.sh`, 3431 s, pixel guards included). Status stays
`in-progress`: the seat items above (trunk hover at the foot, the #174 feel at trunk scale) are unrun.
**Seat, 2026-10-08, on `4f97181` (pushed `--fast`): both items PASSED.** Wolf: "1 yes it's ok 2 no does not flicker
any more". He also saw a NEW defect: while dragging, the cut box flickers and the terrain shows through it. Wolf ruled
that it is filed, not fixed, because 12.8's review is done: **#177** (bug, route:undecided). Status `done`.

- [x] [Review][Patch] **The `dig_trunk > 0` guard cannot fire, so the trunk half of the strengthened test
  is unguarded** (acceptance, LOW, REWORK; RAN. A latent silent-failure trap, routed to patch under the frostvein
  exception) [crates/gui/src/pick.rs:866].
  - It counts DIG-mode hits on a trunk tile. Dig sees through the crown, so the 4 crown targets alone
    produce 45 of them.
  - With the trunk mutant applied AND both trunk targets removed, the test PASSES, and its message "or the
    trunk half of this test is vacuous" never fires.
  - Merged: `[60,60,3]` meets the trunk before any foliage on 0 of 40 rays, because the crown ring at z 4
    comes first. Only `[60,60,2]` (37 of 40) exercises the fix, so the record's "adds rays at the bare
    trunk `[60,60,2]` and `[60,60,3]`" (`:283`) overstates.
  - Fix:
    - count rays whose first TREE tile (foliage opaque) is a trunk, from an independent trace, and assert
      that count > 0;
    - drop or re-aim `[60,60,3]`;
    - correct the record.
    - Show the guard firing: the trunk mutant with the trunk targets removed must FAIL.
  - **LANDED `73c7d45`** (patch pass 2, 2026-10-08). `dig_trunk` is replaced by `trunk_first`. An
    independent trace lists every tree cell once, by a scan of the world rather than the march. For
    each ray it takes the nearest one by `cell_entry_distance`, with foliage opaque, and counts the ray
    when that cell is a `TreeTrunk`. The assertion is `trunk_first > 0`. `[60,60,3]` is dropped,
    because the sweep is pinned to the boot pitch, so re-aiming it is not an option. The record above
    (`:283`) is corrected.
    - Measured with a temporary `eprintln!`, then reverted from the commit: `trunk_first=37` on the
      real targets. With `[60,60,2]` dropped it reads `0` and fails with "…or the trunk half of this
      test is vacuous". That holds without the mutant AND with the trunk mutant
      (`if cut && is_tree_foliage`) applied. This is the case run 2 RAN as a pass.
    - Mutations: new row "cut-mode trunk target dropped" (`de6ff85`). `RUST_TEST_THREADS=1
      scripts/mutate.sh` on the three cut-mode rows of `12-8.sh`: **3/3 KILLED**. Two of them are the
      re-mutation of the run-1 rows ([[strengthened-test-needs-remutation]]). Exclusivity is not
      checked.
- [x] [Review][Patch] **`tree_foot`'s doc and parameter still say foliage only, and it now takes trunk
  tiles** (acceptance, LOW; read; the stale half of a doc the patch updated for one case only)
  [crates/gui/src/pick.rs:375-377].
  - **LANDED `3d7f117`**. The doc now says "this tree tile (trunk or crown)", matching `tree_base_at`'s
    own doc, and the parameter `foliage` is now `tile`. No behaviour change, so there is no test or
    mutation.

  | Item | Fix written for | Then tested | Pre-existing-state fixture |
  | --- | --- | --- | --- |
  | Trunk guard can fire (REWORK of run 1's test strengthening) | the vacuous world: no ray meets the trunk first, plus the run-1 trunk mutant | the real world: all targets present and the fix in place, so the guard must NOT fire (`trunk_first=37`, pass) | `pine()`, unchanged since round 1, under run 2's RAN state (trunk mutant + trunk targets removed), which the old guard passed |
  | `tree_foot` doc | a trunk tile passed in (the run-1 call site) | n/a, doc and name only | n/a |
- [x] [Review][Defer] **The Ramp-foliage clause of the cut guard is untested** [crates/gui/src/pick.rs:344]
  (acceptance + blind + edge, LOW; RAN) — deferred.
  - `pine()` has no Ramp tile, so a mutant that drops `|| is_tree_foliage` survives.
  - `cut_target` treats `Ramp(TreeFoliage)` as not-a-tree, so an orphan ramp crown gets z+1. That is
    pre-existing.
  - `Ramp(TreeFoliage)` only arises from the dig-ramp rule at `sim-core/src/lib.rs:1219`, and that path
    has not been shown to be reachable.
- [x] [Review][Defer] **`a_cut_drag_over_a_trunk_writes_one_designate_cut_at_the_trunks_level` no longer
  drags from a trunk** [crates/gui/tests/headless.rs:5693] (feature, LOW; RAN) — deferred.
  - Its precondition is checked in mode None. After `5`, `update_pick` re-picks the foot `[1,1,1]`, so the
    anchor is the foot.
  - The wire is the same, `[1,1,2]`, because the base is the trunk's own level in that fixture. The name
    and doc describe a path the patch removed.
- [x] [Review][Defer] **A trunk-anchored drag now slabs at the base z, not the hit z** [crates/gui/src/designate.rs:201]
  (feature, LOW; read) — deferred, a design-consistent change.
  - It now matches crown-anchored and foot-anchored drags, and the draft's "box at the cut level".
  - On a slope, the single-level box misses uphill pines. Preview and wire agree. This is a seat question
    only.

Dismissed (7):
- `tree_base_at` attributes a trunk to a neighbour (blind, ×2 with its fixture-gap twin), and stacked trunks
  merge (edge). Unreachable: trunks are ≥ 3 apart Chebyshev (`sim-core/src/lib.rs:815`), so a trunk's 3×3
  holds only its own column. Edge and Feature confirmed this.
- The silent `tree_foot` fallback (blind): intentional and pre-existing. `cut_target` of a trunk tile is
  itself.
- A foot at `base-1` at z=0 (blind + edge): trees stand on ground.
- The trunk return comes before the foliage skip (blind): blind's own verdict was "fine".
- A one-frame pick lag on the mode key (feature): pre-existing, and invisible at 60 fps.
- The story header said `in-progress` while the board said `review` (acceptance): corrected by this
  review's status sync.

## Dev Notes

### Scope guardrails (do NOT)

- No wire change, no sim change except `CUT_WORK_TICKS` if Task 0.2 moves it (it stays 50 unless Wolf rules).
- No new tui input (NFR10), no tui display change.
- No #173 fix without a red reproduction. The instrument is the scope.
- No change to dig/channel/stockpile/clear picking. The crown resolve is cut mode only.
- No new clip besides Cut. No retiming of Walk, Dig or Carry.
- #172 (tui tree view), #169 (mirror) and #170 (cap overlap) are out of scope.

### What already exists (build on it)

- 12.5's clip machinery: name binding (`glb_clip_names`, `dwarf_clip_label`, `ingest.rs:454-487`), the
  graph (`project.rs:754-788`), the delivered-tick swing phase (`:521-532`), and `gui dwarf N clip X` stderr
  (`:538`).
- The arrival gate `drawn_at_cell` (`project.rs:2548-2550`) and the drop gate (`:559-571`).
- `apply_dwarf_tunics` is the per-entity material-swap pattern (`project.rs:3511-3606`, scheduled at
  `ingest.rs:801`).
- The real-binary clip test `a_miner_logs_dig_and_a_hauler_logs_carry_from_a_real_daemon`
  (`gui/tests/pixel_guard.rs:1020`, `#[ignore]`, full tier, 60-80 s on lavapipe). Extend it with a cut
  designate on tree A (73,59,12), or add a sibling test.
- `12-7-signoff/timber_wire.py` designates cuts on A and B and reports felling ticks.

### Key decisions & traps

- **The GLB exports actions in NAME order: Carry, Cut, Dig, Walk.** Binding by name absorbs it; never
  bind by index (`#Animation0` trap, 12.5).
- **`mutate.sh` does not back up `assets/`.** Commit the promoted GLB before running any row.
- **`cargo test -p gui` does not rebuild `simd`.** Build release `simd` before the real-binary test.
- **The swing is anchored to the work-entry tick**, so with stop-first gating the first drawn swing can
  start mid-cycle. Do not "fix" it by un-anchoring: it is what keeps the swings timed to delivered ticks.
- **Axis conversion:** Blender (x,y,z) → Bevy (x,z,-y). It applies to any axe prop offset.
- **The tint must survive respawn:** the incremental tree path despawns and respawns `TreeMesh`, and a swap
  applied once at mark time is lost. Key it on `Added<MeshMaterial3d>` + the marker, as tunics do.
- **Do not fix #173 by the tint.** A green pine produced by §3's tint is deliberate. The ghost was a marked
  pine drawn green before any tint existed, and AC6's line is what distinguishes the two.

### Project Structure Notes

- UPDATE `crates/gui/src/project.rs` (clips, arrival/drop gates, walker step, preview, tint, materials line)
- UPDATE `crates/gui/src/pick.rs` (cut-mode crown resolve), `crates/gui/src/designate.rs` (if the mode must
  reach the pick)
- UPDATE `crates/gui/src/ingest.rs` (`DWARF_CLIP_NAMES`, tests, system registration)
- UPDATE `crates/gui/tests/headless.rs`, `crates/gui/tests/pixel_guard.rs`
- UPDATE `assets/gltf/SM_VoxelDwarf_Miner01.glb` (promoted, own commit)
- NEW `src-assets/prompts/dwarf-miner-round-20.md` (+ its report, written by the seat)
- NEW `_bmad-output/implementation-artifacts/mutations/12-8.sh`, `12-8-signoff/vehicle-card.md`
- UPDATE `README.md`

### Previous story intelligence

- 12.7: wood rides the kind-blind `carrying` path, so Task 1's pick-up gate fixes logs and stones together.
  Its seat found #168 only by watching, so felled-tree visuals need a seat look again.
- 12.5: the AC8 real-binary test was the only thing that proved the clips play. Keep its shape, and keep
  its deliberate RED (a bridge `job: None` fails it by name).
- 12.5: `cargo test -p gui` doesn't rebuild simd, and a GLB sabotage row is not restored by `mutate.sh`.

### References

- `epics.md:2617-2650` (Story 12.8 + "Folded in"), `:111` (FR45), `:2422-2434` (standing ACs), `:302`
  (draft rule)
- Issues #164, #173 (and its 2026-10-06 research comment), #174
- `12-5-dwarves-at-work-dig-and-haul.md` (Tasks 2a-2c, the AC8 test, the art ledger)
- `12-7-timber.md` (cut on the wire, the Dig placeholder, the deferred "lagging walker misses the cut")
- `12-8-signoff/draft.md` (Task 0.1)

## Verification

**Cut clip, RED observed at creation** (fresh release `simd`, DEFAULT_SEED, release `gui` headless on
`486095d`):

```bash
target/release/simd 7791 &                                    # kill it by PID afterwards
timeout 600 target/release/gui 7791 --headless --frames 1500 2> "$SCRATCH/gui.err" &
sleep 8   # cut tree A at Normal; Nain (0, the woodcutter) walks out and works it
python3 -c 'import socket; s=socket.create_connection(("127.0.0.1",7791)); f=s.makefile(); f.readline(); s.sendall(b"{\"type\":\"designate\",\"kind\":\"cut\",\"rect\":{\"min\":[73,59,12],\"max\":[73,59,12]}}\n"); f.readline()'
wait; grep -E 'gui dwarf 0 clip|clips ' "$SCRATCH/gui.err"
```

- **Observed at creation:** the startup line reads `gui dwarf asset: embedded in this binary, 388320 bytes,
  clips Walk, Dig, Carry`. Nain's only clip lines are `gui dwarf 0 clip dig`, then `gui dwarf 0 clip walk`:
  he chops with the Dig clip. No other dwarf has work on a fresh world. `gui trees: meshes=259`. The gui
  exited 124: lavapipe did not reach 1500 frames in 600 s, and the clip lines came before that. The exit
  code is not the result; the lines are.
- **Required of dev:**
  - the startup line reads `clips Walk, Dig, Carry, Cut`;
  - Nain (0) logs `clip cut` during his cut, then `clip walk`;
  - zero `clip cut` lines is a failure, whatever the exit code.
- **Deliberate RED, required of dev:** mutation row "Cut mapped back to Dig" logs `clip dig` and fails the
  real-binary test by name.

**#164 (headless tests, Task 1):**
- RED, recorded before the fix: the pick-up test shows the item parented on the first update, and the Fast
  test shows no work clip within the run.
- GREEN after the fix.
- At the seat: `--select 1` (Ori, hauler) at Normal, then Fast. The log leaves the ground only when he is
  drawn at it.

**#174 (pure tests, Task 4):** RED first, as listed in Task 4; the look is judged at the seat against the
approved draft.

**#173 (Task 5):** `grep 'gui trees: materials=' "$SCRATCH/gui.err"` shows four GLB materials on a fresh
world, and `cut-tint:N` once N pines are marked. Zero lines is a failure.

**Seat (AC9), in the seat's launch form:**
- WSL: `simd 7451`, and in a second WSL terminal `tui 7451 --z 12` for parity.
- PowerShell: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')`.
- Set trades if needed (`T`). Press `1` and drag a small dig, `3` and drag a 3×3 pile west of the fire,
  `5` and drag across the pines nearest the fire (A and B at x 73, y 56-59). Watch, at Normal:
  - a miner swings at the dig;
  - a hauler carries;
  - Nain chops;
  - all at once.
- Then press `+` once (Fast) and watch a pick-up and a chop.
- Judge #174 against the draft: sweep the cursor across a crown in cut mode, then drag over open ground and
  over pines.
- Note the `gui trees: materials=` lines in the launcher's console if a pine looks wrong.

## Dev Agent Record

### Agent Model Used

- Orchestrator + verifier: Claude Opus 5.5 (`claude-opus-5-5[1m]`). Wolf picked Sonnet 5.5 subagents
  for the implementation (2026-10-07), as in 12.2-12.7. They run one after the other, on one tree.

### Debug Log References

- **Full gate on `4eb9e42` RED (2026-10-07, 3372 s):** one failure, the AC7 real-binary test. Nain logged
  no clip line (`woodcutters cut [(0, 0)]`) while miners and haulers passed. Alone, debug, it passes.
  - **Timeline, measured idle:**
    - the wire puts Nain in `work` on the cut at 22.4 s;
    - the gui logs `clip cut` at 26.5 s;
    - wire work ends at 27.4 s, and the gui logs `clip walk` at 28.1 s.
  - **Cause:** the drawn walker steps by Bevy's `Time<Virtual>` delta, capped at 0.25 s, against
    ~0.55 s lavapipe frames. He trails until the 2.5-cell snap, so only 1.6 s of the 5 s cut was ever
    drawn, and under the gate's load none of it. This happens below ~4 fps only; the seat never
    sees it. Filed as **#175** (product behaviour unchanged in 12.8).
  - **Test fix `4408542`:** the test's own client pauses the sim for 30 s once the wire shows the
    woodcutter in `work` on a cut. The swing holds and the walker catches up; then it resumes.
  - **Shown under load** (28 of 32 cores busy with `yes`):
    - hold 30 s PASSES (400 s run);
    - hold 0 s FAILS with `cut [(0, 0)]` at the woodcutter assert, the gate's failure exactly.
- **Stamp fix `1f69475` (Wolf, "fix it now"):** `build.rs` stamps `-dirty` only for uncommitted changes
  under `crates/`, `assets/` and the manifests. A story note or a seat video had made `launch-gui.ps1`
  refuse a build. Verified on `gui --version`:
  - clean tree: `4408542`;
  - an untracked doc: `4408542`;
  - a `crates/` edit: `4408542-dirty`.

  Wolf's untracked 12.7 seat video was moved to `.bin/12-7-signoff/`.
- **Full gate on `4408542` GREEN** (`RUST_TEST_THREADS=1 scripts/gate.sh`, 4016 s), every pixel guard included.

### Completion Notes List

- **Task 1 (#164)**, Sonnet subagent, `cd9d9ca`. Pick-up is gated on the drawn body reaching his wire
  cell. The drop is gated on his drawn body being within `DROP_REACH_CELLS` (0.1 cell) of the cell the
  wire drops the item on. The walker step is scaled by `walk_speed_ratio(mirror.speed())`: Normal and
  Paused 1, Fast 5, Fast2x 10, Fast4x 20 (`simd` tick periods).
  - **Story premise corrected:** `TickClock::factor()` is the 0..1 blend fraction between ticks, not a
    tick rate. The speed comes from the wire (`Mirror::speed()`).
  - **RED, recorded before the fix:**
    - `a_hauler_does_not_pick_up_his_stone_until_he_is_drawn_at_its_cell` panicked with "still walking
      onto the stone's cell: it is not in his hands yet";
    - `a_dwarf_walking_in_at_fast_reaches_his_work_clip_within_the_run` panicked with "never reached the
      Dig clip within the 50-tick run".
  - GREEN after; `cargo test -p gui` lib 231 and headless 114 pass.
  - `a_carried_stone_is_the_dwarfs_child_at_the_carry_offset_until_he_lets_go` waited 4 frames after
    teleporting the hauler onto the stone, which is exactly the ungated pick-up. It now waits 20 frames;
    its asserts are unchanged.
  - The 12.5 row "the stone is put down while he is still walking in" was re-pointed to
    `DROP_REACH_CELLS = 1000.0`.
  - **Mutations** (orchestrator-run, `RUST_TEST_THREADS=1 scripts/mutate.sh`), all KILLED by the
    assertion named:
    - `12-8.sh` "pick-up ungated" kills the pick-up test at its first assert;
    - "Fast walker unscaled" (Fast 5.0 → 1.0) kills the Fast test;
    - the re-pointed 12.5 row kills `a_hauler_keeps_his_stone_until_he_is_drawn_at_the_cell_he_drops_it_on`.
  - Untested: Fast2x and Fast4x ratios (only Fast has a test). At Fast4x a walker can step over the
    0.1-cell drop window in one frame; he then lets go on arriving at his wire cell, late but never stuck.
- **Task 3 (Cut binding)**, Sonnet subagent, `f3473fb`. `DwarfClip::Cut` ("cut") is bound by name with
  `load("Cut")`. `DwarfClips::node(Cut)` falls back to the Dig node while the GLB has no Cut.
  `WORK_SWING_TICKS` became `DwarfClip::swing_ticks()` (Cut 10, else 5) and `swings()`. The arrival gate,
  the phase arm, `DwarfHeadings` and the STALLED loop all treat Cut like Dig. `DWARF_CLIP_NAMES` has four
  names, and the README is updated.
  - Startup line today (Cut not yet in the GLB): `clip Cut ABSENT -- this dwarf cannot play it; the
    runtime GLB is behind the .blend`.
  - RED before the fix:
    - `a_woodcutter_on_a_cut_job_in_work_gets_the_cut_clip` (renamed): `left: Dig right: Cut`;
    - new `a_cut_swing_advances_at_half_the_rate_of_a_dig_swing`: "a cut is half way at 5 ticks: 0".
  - **AC7 real-binary test:** extended `a_miner_logs_dig_and_a_hauler_logs_carry_from_a_real_daemon`
    with a cut on tree A. GREEN (~123 s, lavapipe):
    - woodcutter cut [(0,1)] then walk;
    - miners dig and walk;
    - haulers carry and walk.

    Its deliberate RED (Cut→Dig in `dwarf_clip`) panics at `pixel_guard.rs:1164` "no woodcutter [0]
    logged clip cut followed by clip walk".
  - **TEMPORARILY LOOSENED until the GLB is promoted, and must be re-tightened at Task 2c:**
    - the AC7 test accepts `clip Cut ABSENT` in place of `clips Walk, Dig, Carry, Cut`, and its STALLED
      filter ignores "the Cut clip never loaded";
    - `the_embedded_dwarf_carries_walk_dig_and_carry_by_name` asserts a 3-name literal.

    All three carry a NOTE.
  - **Re-pointed rows:**
    - 12.5: "Carry is preferred over Dig", "the dig phase runs on wall time", "the dig clip starts while
      he is still walking in", "the Dig clip is never bound";
    - 12.7: row 16, now on `..._gets_the_cut_clip`.
  - **Mutations** (orchestrator-run, `RUST_TEST_THREADS=1 scripts/mutate.sh`), 8/8 KILLED at the
    targeted assertion:
    - `12-8.sh` "Cut mapped back to Dig" (headless `:5994`), "Cut mapped back to Dig, seen by the real
      binary" (`pixel_guard.rs:1164`), "a cut swings at a dig's period" (`:5237`);
    - the four re-pointed 12.5 rows ("the Dig clip is never bound" at the STALLED assert,
      `pixel_guard.rs:1185`);
    - 12.7 row 16.
  - Untested by any automated test: the Cut→Dig node fallback itself (it needs real assets). The real
    binary exercised it with no STALLED.
- **Task 4 (#174 cut mode)**, Sonnet subagent: `ac389ab` (pick), `e526fe7` (box), `dbb93da` (tint).
  - **Pick:** `update_pick` reads `DesignateMode` and passes `cut: bool` to `first_visible_hit`. In cut mode
    a foliage hit returns `tree_foot`, the ground cell under the trunk, whose `cut_target` is the base.
    Every other mode passes `false` and is unchanged. The new helper `tree_base_at` serves the pick and
    the tint.
    - RED for `in_cut_mode_a_ray_through_a_crown_resolves_to_that_tree` (pick.rs, new `pine()` fixture):
      "cut mode left the tree's column at yaw -1 ... [57, 52, 1]".
  - **Box:** `sim_will_keep` keeps every cell for Cut. `preview_appearance` draws `cut_mark` on tree tiles
    and `hover_highlight` elsewhere.
    - RED for `the_cut_preview_covers_every_rect_cell_and_only_tree_cells_are_in_the_cut_style` (renamed):
      "the whole box shows over open ground, not only the trunk".
  - **Tint (a):**
    - `CutTinted(bool)` on `TreeMesh` is set from Cut designations and from a live Cut drag's
      `DragPreviewCells`.
    - `apply_cut_tint` (the `apply_dwarf_tunics` pattern: `Added<MeshMaterial3d>` + `Changed<CutTinted>`)
      swaps in ONE shared clone of the pine material with its texture dropped and `base_color` set to
      foliage (44,100,58). It restores `PineOwnMaterial` when the mark goes. Handles only; no component
      is removed. The foot slab is kept.
    - RED for headless `a_cut_marked_pine_wears_the_shared_tint_and_keeps_it_across_a_respawn`: "a
      cut-marked pine must wear the tint".
    - Unit test `a_live_cut_drag_tints_the_pines_it_catches_and_no_other_mode_does` was written green
      only, and its two mutation rows below are its RED.
  - Re-pointed: 8.1 row "slice visibility is removed from the march".
  - **Mutations** (orchestrator-run), 8/8 KILLED at the named assertion:
    - "cut-mode crown fall-through restored" (`pick.rs:841`);
    - "the box filtered back to trees" ("the whole box shows over open ground");
    - "the box's open cells drawn in the cut style" ("open air in the box is the hover slab");
    - "the tint never cleared" ("clearing the mark puts the pine's own material back");
    - "a respawned pine is not re-tinted" ("a respawned marked pine must come back tinted");
    - "a live cut drag tints nothing" ("a crown cell in the box tints its pine");
    - "a drag in any mode tints" ("a dig drag over a crown tints nothing");
    - 8.1's re-pointed row.
  - Not judged headless, so judged at the seat: the flat-green pine look, hover slabs over a large box,
    and the foot hover. `sync_cut_tint_marks` runs every frame over ~265 `TreeMesh` (unmeasured; it
    inserts only on change).
- **Task 5 (#173 instrument)**, Sonnet subagent, `4da9568`. `report_pine_materials`
  (`project.rs`, registered after `apply_cut_tint`) runs once `TreeReportState.reported` is set. It prints
  `gui trees: materials=<distinct handles> [<label>:<pines>, ...]` whenever the line changes. The label is
  `cut-tint` for the shared `CutTint` handle, otherwise Bevy's `GltfMaterialName`.
  - RED for `the_trees_materials_line_counts_pines_per_material_and_follows_the_marks`: `left: ""`.
  - GREEN asserts, in order: `materials=1 [PineBark:3]`, then `materials=2 [PineBark:1, cut-tint:2]`, then
    `materials=2 [PineBark:2, cut-tint:1]`.
  - **Real binary, fresh world** (release `simd 7795`, `gui --headless`):
    - `gui trees: meshes=259 scenes_loaded=true source=embedded frames=2`
    - `gui trees: materials=4 [M_VoxelPine:259]`
  - **Limitation:** all four pine GLBs name their material `M_VoxelPine`, so the label cannot tell the
    variants apart; only the handle count (4) does. A non-tint handle swap shows as a 5th handle or a
    new label. Kept minimal; per-variant labels only if a #173 reproduction needs them.
  - Mutations: see the Task 5 rows of `12-8.sh` (below).
- **Task 6 so far:** `12-8.sh` has 15 rows, and every one ran KILLED in the per-task runs above. The
  rows the change broke were re-pointed and re-run KILLED: four in 12.5, one in 12.7, one in 8.1.
  - README: the Cut clip, the clip list, cut-mode pick/box/tint and the `materials=` line.
  - Seat card `12-8-signoff/vehicle-card.md`, in two passes (pass 2 after the GLB is promoted).
  - Open: the whole-file run after the GLB promotion, since `mutate.sh` does not back up `assets/`.
- **Pushed** `4eb9e42` with `push.sh --fast` (VERIFIED) for seat pass 1. Full gate running detached.
- **Task 2 (round 20).** Wolf ran the seat and committed it himself (`ebe6b41`, his own authorship).
  - Option (B): `r17_axe` on a 21st joint `axe` (child of `chest`), stowed on the pack in
    Walk/Dig/Carry; one-handed.
  - The bite was authored at 0.95 m, so the client draws him 0.35 m closer (`CUT_OFFSET`).
  - Export (`blender --background ... export_dwarf.py`): `rig joints 21, missing 0, unexpected 0`,
    `clips Carry, Cut, Dig, Walk (63 channels)`.
  - `check_asset.py` on the promoted file: `tris=3456 joints=21
    anims=Carry:63ch@1.00s,Cut:63ch@1.00s,Dig:63ch@1.00s,Walk:63ch@1.00s`, exit 0.
  - Promoted alone in `b5a9e63`; the fast gate was green, and no test pinned the old asset.
  - **Joint count:** nothing under `crates/` pins it. The exporter's rig gate (missing/unexpected
    joints) is the pin, and the clips bind by name. No count assertion was added.
- **After promotion** (Sonnet subagent):
  - `0105b94` re-tightened what Task 3 loosened. `the_embedded_dwarf_carries_walk_dig_carry_and_cut_by_name`
    checks `DWARF_CLIP_NAMES`; the AC7 test accepts only `clips Walk, Dig, Carry, Cut` and no STALLED
    line at all.
  - `c27e416` added **`CUT_OFFSET` = 0.35 m** (Wolf's round-20 ruling, `appearance.rs`).
    `nudge_dwarf_for_cut` moves the GLB armature node (`SK_VoxelDwarf_Miner01_r17`, which carries the
    `AnimationPlayer`, confirmed in the real binary) by `(0, 0, -0.35)` from its captured
    `ArmatureRest` while the clip is Cut. The dwarf entity is never moved, so `drawn_at_cell` and the
    walker are untouched. No clip channel targets that node. The nudge pops with no ease (NOTE).
  - RED for `a_cutting_dwarfs_armature_is_nudged_toward_the_trunk_and_his_entity_is_not`:
    `left: Vec3(0, 0, -0.17516592) right: Vec3(0, 0, -0.5251659)`.
  - AC7 real binary after it (debug): `woodcutters cut [(0, 1)] walk [(0, 1)]`; the startup line is
    asserted as `clips Walk, Dig, Carry, Cut`.
- **Final mutation run** (GLB committed first, `RUST_TEST_THREADS=1 scripts/mutate.sh`): **24/24 KILLED**.
  That is all 17 rows of `12-8.sh` plus the seven re-pointed rows (12.5 x5, 12.7 row 16, 8.1).
  - The two new rows: "CUT_OFFSET never applied" dies at `headless.rs:6086`, and "the cut nudge never
    cleared" at `:6104` ("the nudge must clear exactly").
- **Full gate on `f7a1649` GREEN** (`RUST_TEST_THREADS=1 scripts/gate.sh`, 3223 s), pixel guards included, with the round-20 GLB.
- **Seat, AC9 (Wolf, 2026-10-07): "1 ok 2 ok".**
  - Pass 1: cut-mode pointer, box and tint against the approved draft, and the `materials=` line.
  - Pass 2: the axe chop in game with `CUT_OFFSET`, dig + haul + cut together at Normal, and Fast.
  - His aside, "woodcutter could walk also with axe in the hand instead of switching it when
    cutting", is parked as **#176** (enhancement, route:story). It is not in 12.8.
  - No ghost pine was reported, so #173 stays instrument-only.
- Issues filed: **#175** (walker below its speed when frames exceed 0.25 s), **#176** (axe in hand
  while walking).
- **Task 2a**: brief `src-assets/prompts/dwarf-miner-round-20.md` (`6425a40`). Tool A (re-use the
  pickaxe) or B (an axe, a 21st joint, Wolf's explicit yes) is decided at the seat.

### File List

- `crates/gui/src/project.rs`, `crates/gui/src/pick.rs`, `crates/gui/src/ingest.rs`, `crates/gui/src/appearance.rs`,
  `crates/gui/build.rs`
- `crates/gui/tests/headless.rs`, `crates/gui/tests/pixel_guard.rs`
- `assets/gltf/SM_VoxelDwarf_Miner01.glb` (round 20, promoted)
- `README.md`
- `src-assets/prompts/dwarf-miner-round-20.md`, and from Wolf's seat commit `ebe6b41`:
  - `src-assets/prompts/dwarf-miner-round-20-report.md`;
  - `src-assets/blender/SM_VoxelDwarf_Miner01.blend`, `cut_r20.py`, `coldrun_r20.py`, `work_r19.py`,
    `export_dwarf.py`;
  - `src-assets/renders/r20/` (36 files).
- `_bmad-output/implementation-artifacts/mutations/12-8.sh` (new), and the re-pointed `12-5.sh`, `12-7.sh`,
  `8-1-point-at-the-world.sh`
- `_bmad-output/implementation-artifacts/12-8-signoff/draft.md`, `12-8-signoff/vehicle-card.md`
- `_bmad-output/implementation-artifacts/12-8-dwarves-at-work-the-cut.md`, `sprint-status.yaml`, and the
  `metrics/` ledger. Plus creation-time edits: `epics.md`, `12-7-timber.md`.

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-06 | Created on `486095d`. Cut clip + #164 + #173 (instrument only, not reproduced) + #174 (draft-first). Task 0 open. |
| 2026-10-07 | Task 0 ruled by Wolf: draft approved with tint (a) + slab, 10-tick swing, Fast walker scales (a), #173 instrument-only. |
| 2026-10-07 | Task 0 ruled. Dev (Sonnet subagents, Opus orchestrating): #164 gating + Fast walker; Cut bound by name at 10 ticks; #174 pick/box/tint; #173 `materials=` line. Round 20 (Wolf's seat, axe, $21.59) promoted; `CUT_OFFSET` 0.35 m. Stamp narrowed to build inputs. AC7 test held 30 s (#175). 24/24 mutation rows KILLED; full gate GREEN on `f7a1649`. Seat: "1 ok 2 ok". Status review. |
| 2026-10-08 | Review run 1 patch pass: in cut mode a trunk hit now resolves to the pine's foot (`ef5dfb0`). The crown test asserts the exact foot cell and adds trunk rays. RED first; 2/2 mutation rows KILLED; full gate GREEN on `ef5dfb0` (3344 s). |
| 2026-10-08 | Review run 2 on the patch (`712cd48..679e55a`): 0 HIGH/MED; 2 LOW patches left as action items (the `dig_trunk` guard cannot fire, REWORK; `tree_foot` doc); 3 deferred. No run 3. Status in-progress. |
| 2026-10-08 | Review run 2 patch pass: trunk guard replaced by an independent first-tree-cell trace (`73c7d45`, 37/40 rays; `[60,60,3]` dropped, record corrected); `tree_foot` doc (`3d7f117`); new mutation row (`de6ff85`), 3/3 cut rows KILLED; full gate GREEN on `de6ff85` (3431 s). Seat items open; status in-progress. |
| 2026-10-08 | Seat on `4f97181`: trunk hover sits at the foot, and the hover no longer flickers (Wolf). The drag box flickers with terrain showing through it: filed as #177 at Wolf's ruling, since review is done. Status done. |
