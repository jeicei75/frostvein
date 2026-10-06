---
baseline_commit: 110cab4
model: claude-opus-5-5  # session default, same as 12.1-12.7's creation
---

# Story 12.8: Dwarves at Work — The Cut

Status: ready-for-dev

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

- [ ] **Task 0: Wolf's rulings, at creation.** Open; asked at the end of creation.
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
- [ ] **Task 1: #164, RED first (AC3, AC4).** Headless (`crates/gui/tests/headless.rs`, MinimalPlugins,
  `ManualDuration(100ms)`), following `a_hauler_keeps_his_stone_until_he_is_drawn_at_the_cell_he_drops_it_on`
  (`:5338`).
  - [ ] RED first:
    - pick-up: one delta steps the hauler into the item's cell AND sets `carrying`; after one `app.update()`
      the item is NOT parented to him. Record the failure;
    - Fast: feed one 50-tick work run at Fast's tick rate and assert the dwarf reaches his work clip within
      the run. Record the failure.
  - [ ] Gate the pick-up on `drawn_at_cell` (`project.rs:499-500`), as the drop already is, and gate the drop
    on the cell the wire drops at, not his current cell (`:559-571`).
  - [ ] Implement Task 0.3's ruling for Fast. If (a), the step at `:2649` multiplies by the tick-rate factor
    the clock already knows (`TickClock::factor`, `:459`); keep `DWARF_WALK_SNAP_CELLS` (`:644`) as is.
  - [ ] Both tests GREEN; the old hauler test stays GREEN.
- [ ] **Task 2: the Cut clip, from a live BlenderMCP seat (AC1, AC2).** Tasks 1, 3, 4 and 5 do not need it;
  run them while the seat is pending.
  - [ ] **2a — write the brief** `src-assets/prompts/dwarf-miner-round-20.md`, in round 19's shape
    (`dwarf-miner-round-19.md`): the "For:" header, the in-place rule, LINEAR keys at 24 fps on frames 0..24
    with frame 24 == frame 0, the action saved unassigned with slotted `fcurves()`, one tool call per pose
    stage with a screenshot each, and the generator script written as it goes. Content:
    - **one `Cut` action**: a two-handed axe chop at a trunk standing one cell ahead, with the period Task 0.2
      rules;
    - the axe is a new prop on the existing rig, following round 19's 20th `pick` joint, OR the pick
      re-used: the seat decides, and the brief says so;
    - Walk, Dig and Carry are untouched;
    - the report goes to `dwarf-miner-round-20-report.md`.
  - [ ] **2b — Wolf runs the seat**, watching live. Art hard stop: after two rounds his eye has not judged
    converging, stop and ask him (ship plain or park). One "Art ledger" row per round: model id, his `/cost`,
    his words, verdict.
  - [ ] **2c — export, check, promote.**
    - `blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend --python src-assets/blender/export_dwarf.py`
      (it writes to the gitignored `src-assets/export/`).
    - `check_asset.py` must print four clips, for example
      `anims=Carry:60ch@1.00s,Cut:…,Dig:60ch@1.00s,Walk:60ch@1.00s`. Its loop check (`check_asset.py:463`)
      assumes every clip loops, so Cut loops. No stray action.
    - Promote to `assets/gltf/SM_VoxelDwarf_Miner01.glb` in its OWN commit, and re-run `check_asset.py` on
      the promoted file. Rebuild the gui (`include_bytes!` at `ingest.rs:421-423`).
- [ ] **Task 3: the gui binds and plays Cut (AC1, AC2).** Add Cut beside Dig at every site:
  - [ ] `DwarfClip` and `label()` (`project.rs:404-418`); `DwarfClips` field, `node()`, `nodes()`
    (`:335-375`); the graph build `load("Cut")`, `present`, `take` order (`:765-786`);
  - [ ] `dwarf_clip` returns `Cut` for `Work` + `Cut{..}` and drops the placeholder NOTE (`:427-445`);
  - [ ] the arrival gate and the swing-phase arm treat Cut like Dig (`:510-532`, `:2481-2484`), with Task
    0.2's period; the STALLED loop gains Cut (`:2450`); `dig_yaw` already faces a cut target (`:2532-2543`);
  - [ ] `DWARF_CLIP_NAMES` (`ingest.rs:490`) and its report and absent tests (`:5992`, `:6031-6035`), the
    `README.md:313` literal and the `clip <walk|dig|carry>` README text;
  - [ ] rename `a_woodcutter_on_a_cut_job_in_work_gets_the_dig_clip` (`headless.rs:5859`) to `…_the_cut_clip`;
    `the_embedded_dwarf_carries_walk_dig_and_carry_by_name` (`ingest.rs:5960`) covers all four;
  - [ ] if Cut is missing from the GLB, Cut falls back to Dig. The startup line says `clip Cut ABSENT`, the
    same shape as the others.
- [ ] **Task 4: #174 cut mode, per the approved draft (AC5).** RED first for the pick and the box; the tint is
  judged at the seat.
  - [ ] **Pick:** in cut mode only, a ray entering a foliage cell resolves to that tree, the same answer
    `cut_target` gives for its trunk (`designate.rs:272`). Pure test with `pillars()` (`pick.rs:490`) and
    `ray_at` (`:699`): sweep rays across one crown at pitch 0.45. In cut mode, every hit stays within that
    tree's 3×3 column; in dig mode, today's fall-through is unchanged. RED first.
  - [ ] **Box:** the Cut arm of `preview_cells` returns the whole rect at the cut level. `preview_appearance`
    draws non-tree cells in the hover material and the caught trees per the draft. Extend
    `the_cut_preview_lights_only_tree_tiles_at_the_cut_level` (`project.rs:3771`): its new name and asserts
    say "every rect cell is previewed; only tree cells in the cut style". The release still sends one
    `Designate{Cut, rect}` (unchanged).
  - [ ] **Tint**, per Task 0.1. For (a), follow `apply_dwarf_tunics` (`project.rs:3511-3606`):
    - a marker on a `TreeMesh` whose base carries a cut designation, set and cleared from
      `mirror.designations()` (`client-core/src/lib.rs:181`);
    - on `Added<MeshMaterial3d>` and on `Changed<marker>`, swap in ONE shared tinted clone of `M_VoxelPine`,
      and swap the original back when the mark goes;
    - insert or replace handles only, never remove a component (`remove_with_requires` is banned,
      `ingest.rs:2091`);
    - re-apply on respawn, because the incremental tree path respawns `TreeMesh` (`project.rs:3044`).

    Headless test: mark a tree, and its mesh carries the tinted handle; clear the mark, and it carries the
    original. The incremental path is covered by felling the marked tree's neighbour.
- [ ] **Task 5: #173's instrument (AC6).** Once the scene reports trees loaded, the gui prints
  `gui trees: materials=<n> [<label>:<count>, …]`, and again whenever the count changes. `<label>` is the GLB
  material name or `cut-tint`. A probe of this shape ran clean at creation (4 handles, per variant).
  - [ ] Test: a headless app with two marked pines reports `cut-tint:2`, and the line changes when one mark
    is cleared.
  - [ ] If Wolf reproduces the ghost at the seat (Task 7), file the line in #173. Do NOT fix #173 in this
    story without a red reproduction (M2-27). If a fix is found, it gets its own task with Wolf's yes.
- [ ] **Task 6: the record.**
  - [ ] `mutations/12-8.sh`, at least:
    - Cut mapped back to Dig in `dwarf_clip` (killed by AC7's real-binary test and the headless test);
    - pick-up ungated;
    - Fast walker unscaled (if 0.3(a));
    - cut-mode crown fall-through restored;
    - the box filtered back to trees;
    - the tint never cleared;
    - the materials line printing a constant.

    Commit the GLB BEFORE any row runs: `mutate.sh` does not back up `assets/` (`mutate.sh:71`).
  - [ ] Re-point old rows the change breaks (12.5 rows on `dwarf_clip`, the `clips Walk, Dig, Carry` literal).
    Run them, and every row must be KILLED.
  - [ ] README `gui` section: the Cut clip and the clip list, cut-mode pick/box/tint, the `materials=` line.
  - [ ] Seat card `12-8-signoff/vehicle-card.md` in the seat's launch form (see Verification).
- [ ] **Task 7: the live recipe, the seat (AC9), then the full gate.** `RUST_TEST_THREADS=1 scripts/gate.sh`
  on the final HEAD.

### Art ledger

| Round | Model | Wolf's `/cost` | Wolf's words | Verdict |
| --- | --- | --- | --- | --- |

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

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-06 | Created on `486095d`. Cut clip + #164 + #173 (instrument only, not reproduced) + #174 (draft-first). Task 0 open. |
