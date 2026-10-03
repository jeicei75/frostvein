---
baseline_commit: f3b7cb3
model: claude-opus-5-5  # session default, same as 12.1-12.4's creation
---

# Story 12.5: Dwarves at Work — Dig and Haul

Status: in-progress

## Story

As the boss,
I want to see a miner swing at the rock and a hauler carry the stone,
so that the valley shows work being done, not dwarves standing still.

## Not stacked: branch off `main`

`main` is `f3b7cb3` (PR #161 merged), clean. Branch `story-12-5-dwarves-at-work-dig-and-haul` carries
this file, `12-5-signoff/` and the board edit. The board edit also flips `12-4-every-dwarf-has-a-trade`
to `done`: 12.4 was merged while the board still said `review`. Epic 12's nine standing ACs
(`epics.md`, "Standing acceptance criteria") bind this story and are not restated. "Wire diff" below
satisfies standing AC 4. This is a work-animation story, so standing AC 7 has no draft: it goes straight
to the seat under the hard stop.

## Found at creation (2026-10-03, on `f3b7cb3`)

- **Dig is on the wire, but it is ambiguous.** `state: work` lasts exactly `WORK_TICKS` = 5 ticks at the
  face (`sim-core/src/lib.rs:45`, `:1065-1070`). A hauler's pick-up and drop are also 5-tick `work` runs
  (`:1079-1094`). So `state` alone cannot tell "swing a pick" from "bend for a stone".
- **Carrying is NOT on the wire.** `Carrying(Option<u32>)` (`lib.rs:246`) is exposed only as
  `World::carrying()` (`:1775`), and nothing bridges it. The stone is an ordinary `Item` that
  `carry_items` (`:1178`) keeps on its carrier's cell. The gui draws it snapping from cell to cell
  (`gui/src/project.rs:1955`, blend `:2264-2269`) while the dwarf walks smoothly.
- **Facing is client-derived** from position deltas (`entity_draw_rotation`, `project.rs:559`;
  `DwarfHeadings::record`, `:2181`). A miner arriving at a dig face from the side faces along his last
  step, not at the rock. A Dig's work position is any of the target's four orthogonal neighbours
  (`work_positions`, `lib.rs:800-810`). A Channel's work position is the target tile itself, and the
  dwarf digs the floor under his own feet.
- **The GLB carries one clip.** `check_asset.py assets/gltf/SM_VoxelDwarf_Miner01.glb` gives
  `anims=Walk:57ch@1.00s`, 19 joints. The client binds it by index (`#Animation0`,
  `project.rs:513-525`). `dwarf_clip_summary()` (`ingest.rs:452`) only checks that `"animations":`
  appears in the JSON, so it cannot see which clips are present.
- **Live RED and measured timing** (`12-5-signoff/work_wire.py`, fresh `simd`, DEFAULT_SEED, fast4x,
  900 ticks):
  - dwarf keys are `id identity kind light pos profession state`;
  - 25 miner work runs and 19 hauler work runs, every one exactly 5 ticks;
  - hauler walk runs (most of them carrying) last 56-87 ticks;
  - miner time splits idle 0.41 / walk 0.52 / **work 0.07**;
  - result: `NO JOB FIELD ON THE WIRE`, exit 1, with 125 miner work ticks carrying no job.
- **At Normal a dig run is 0.5 s**, so one tile gets one swing. At Fast4x the run is 25 ms, about 1.5
  frames at 60 fps, so the swing cannot be seen. Fast-forward speeds the clips up as the AC asks, but at
  Fast4x the dig is effectively invisible. That is expected, not a defect.
- **DEFAULT_SEED's crew:** 0 Nain woodcutter, 1 Ori hauler, **2 Bifur miner**, 3 Frar hauler,
  **4 Dori miner**. `--select ID` follows a dwarf at `SELECT_DISTANCE` 20 (`pick.rs:83`), where he is
  about 49 px tall. At the boot camera he is about 11 px, and no clip can be judged at that size.

## Wire diff (standing AC 4)

- NEW `protocol::DwarfJob`: `Dig { target: [i32; 3] } | Channel { target: [i32; 3] } | Haul`, with
  `#[serde(rename_all = "snake_case")]`, externally tagged.
  - A dwarf's line becomes `"job":{"dig":{"target":[x,y,z]}}`, `"job":{"channel":{"target":[…]}}` or
    `"job":"haul"`.
  - Haul has no target: `Job.target` for a Haul is the stone's position at creation and goes stale at
    pick-up (`lib.rs:221-224`).
- `protocol::Entity` gains two last fields, after `profession`, each with
  `#[serde(default, skip_serializing_if = "Option::is_none")]`:
  - `job: Option<DwarfJob>`, set while the dwarf holds a job, in `walk` and `work` alike;
  - `carrying: Option<u32>`, the id of the stone he carries, which also appears in the same message's
    `items`.
- Emitter lines are byte-identical, and so is an idle dwarf's line. `Entity` stays `Copy`.
- **No sim-core change and no save change.** `CurrentJob` and `Carrying` are already saved. `simd`
  composes the fields from `World::claims()`, `jobs()` and `carrying()`.

## Acceptance Criteria

1. The real daemon is the judge for the wire.
   - In a delta, every miner in `work` carries `job` as `dig` or `channel`, with the job's target.
   - A hauler holding a haul carries `"job":"haul"`.
   - Every `carrying` id is an item in that delta's `items`, at the dwarf's own cell.
   - A dwarf with no job and no stone serialises byte-identical to today.
2. The embedded dwarf GLB carries three clips named `Walk`, `Dig` and `Carry`, and `check_asset.py`
   passes on it (loop closure on all three).
   - A test reads the clip **names** from the embedded bytes and fails, naming any missing clip.
   - The startup line names the clips present.
3. The gui picks a dwarf's clip from his wire state alone:
   - `Dig` while `state == work` and `job` is `dig` or `channel`;
   - otherwise `Carry` while `carrying` is `Some`;
   - otherwise `Walk`.
   When the state that chose `Dig` or `Carry` ends, the dwarf is back on `Walk` within the frame that
   ingests that delta. No other input chooses a clip.
4. The dig swing is timed in sim ticks. A dwarf's `Dig` phase is
   `(mirror tick − the tick he entered work + TickClock::factor()) / WORK_SWING_TICKS`, wrapped to
   `[0, 1)`, with `WORK_SWING_TICKS = 5`. So one work run is one swing.
   - Mechanism, load-bearing: the phase comes from delivered ticks plus AD-15's blend factor. It is
     never wall time and never a predicted tick.
   - Deltas that repeat one tick (pause) leave the phase constant across frames. Faster ticks advance
     it faster.
   - A snapshot restarts every phase.
5. `Carry` is phase-locked to the ground the dwarf is drawn covering, exactly like `Walk` (shared
   `WalkPhase`, the same stride `DWARF_WALK_STRIDE_METRES`). Its legs keep the walk's stride, so a
   carrying dwarf does not skate. A dwarf who covers no ground holds his pose.
6. While working a `dig` job, the dwarf is drawn facing his target: yaw toward `target − pos` on the
   horizontal axis. A `channel` keeps his heading. On his next step, position-derived facing resumes.
7. While a dwarf's `carrying` is `Some(id)`, item `id` is drawn held:
   - it is a child of the dwarf at `CARRY_OFFSET`, so it moves with his blended position;
   - it is not drawn at its own cell.
   On the delta that clears `carrying`, it is drawn at its cell again.
8. The instrument is the real `gui` binary.
   - Every clip switch prints `gui dwarf {id} clip {walk|dig|carry}` to stderr.
   - A real-binary test against a real `simd` has a test client designate a channel block and a pile.
     It sees a miner log `clip dig`, a hauler log `clip carry`, and each later log `clip walk`.
   - Its deliberate RED is the bridge sending `job: None`, which makes the test fail by name.
9. TUI: no display change and no regression (NFR10). The tui decodes the new fields, and its captures
   are unchanged.
10. At the seat, Wolf watches a selected miner swing at a dig face and a selected hauler carry a stone to
    the pile.
    - Each art round gets a row in "Art ledger" below.
    - The story is done when Wolf judges both clips read, or when, after two rounds his eye has not
      judged converging, he rules that they ship plain or are parked (FR45, M2-24).

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings, 2026-10-03, at creation.**
  1. **Wire shape: `job` + `carrying`**, as in "Wire diff". He chose it over `carrying` only, which would
     have inferred Dig from `profession == miner && state == work` (a trade rule in the client, AD-4) and
     left no target to face.
  2. **Dig timing: keep `WORK_TICKS` = 5**, one swing per dig (0.5 s at Normal), judged at the seat.
     Lengthening it is a one-constant sim change. Its fallout is known: the `Walk, Work×5` pins, the
     `first_delivery.py` figures and the 12.4 backlog numbers all move. It is his call at the seat, not
     the dev's.
  3. **Clips: a live BlenderMCP seat from round 1**, in round 18's form. He chose this over plain clips
     scripted in the devpod's headless Blender. A Claude Code session drives his **running** Blender
     through BlenderMCP while he watches ([[live-modelling-must-be-watchable]]). He then judges the
     promoted clips in the game.
- [x] **Task 1: protocol + simd (AC1; the wire diff).**
  - [x] `protocol/src/lib.rs`:
    - add `DwarfJob` beside `Profession` (`:97`);
    - add the two fields after `profession` (`:200`);
    - fix every `Entity { .. }` literal with `job: None, carrying: None` (let the compiler list them).
  - [x] Pin tests in `protocol`:
    - the existing entity and delta literals stay byte-identical;
    - one literal per `DwarfJob` variant, plus `carrying`, round-trips;
    - `every_material_and_tile_variant_has_a_pinned_wire_name` (`:524`) gains a `DwarfJob` block.
  - [x] `simd/src/bridge.rs`:
    - `fn dwarf_job(job: sim_core::Job) -> protocol::DwarfJob`: an exhaustive `match` on `job.kind`, no
      wildcard, so 12.7's `Cut` fails to compile until it is given a wire arm;
    - `dwarf_entities` (`:134`) sets `job` from `world.claims()` joined to `world.jobs()` by `JobId`, and
      sets `carrying` from `world.carrying()`, looked up by id the way `identities` is.
  - [x] `simd/tests/serve.rs`: a serve test drives a real daemon. It designates a channel block and a
    pile and steps at fast4x. On deltas it asserts:
    - a miner in `work` with `job` dig or channel;
    - a hauler with `"job":"haul"`;
    - a `carrying` id present in `items` at the carrier's cell.
    This also closes 12.4's deferral that "deltas carry a profession" had no automated test: assert
    `profession` on those deltas too.
- [ ] **Task 2: the clips, from a live BlenderMCP seat (AC2; Task 0 ruling 3).** Tasks 1 and 3's headless
  parts do not need the clips, so run them while the seat is pending. Task 4 and the seat need the
  promoted GLB.
  - [x] **2a — write the brief** `src-assets/prompts/dwarf-miner-round-19.md`. Copy round 18's shape
    (`dwarf-miner-round-18.md`: the "For:" header, the pipeline table, the in-place rule).
    - **For:** a Claude Code session with the Blender MCP server attached, Blender 5.2.1, driving Wolf's
      **running** Blender instance. It writes only inside `src-assets/`.
      - It must not spawn `blender --background` for authoring. A background run is allowed only as the
        final cold-run regeneration proof.
      - One tool call per pose stage, with a committed screenshot each.
      - The generator `src-assets/blender/work_r19.py` is written as the clip is built.
    - **Produces** two actions on the r17 armature in `SM_VoxelDwarf_Miner01.blend`, named exactly `Dig`
      and `Carry`. `Walk` is left untouched.
      - Both follow `walk_r18.py`: LINEAR keys at 24 fps, frames **0..24** with frame 24 == frame 0
        (Trap 2), in place with no horizontal `root` motion, each action saved unassigned
        (`detach()`/`attach()`), slotted fcurves through `fcurves()`, and `checks()` measuring the
        bound, evaluated mesh (Trap 1).
    - **`Dig`**: one pick swing per cycle. The client plays one cycle per 5-tick work run (0.5 s at
      Normal), so the brief states that duration.
      - The pick (`r17_pickaxe`, on `hand.R`) strikes at about frame 18 (phase ≈ 0.75), just before the
        sim's tile change.
      - The swing goes along the direction `Walk` walks (the client yaws him toward the target), and
        the strike lands
        about one cell (1.6 m) in front of him at foot-to-knee height. A Channel uses the same clip.
      - Clearance-check the pick at every frame: it is 1.04 H and overshoots (round-18 brief, lines
        68-72).
    - **`Carry`**: `Walk`'s leg and root keys copied verbatim, so the stride stays 0.4926 m and the
      client's phase lock holds. The arms hold a stone in front of the chest. The seat measures where
      the hands meet and reports it as `CARRY_OFFSET` (metres, rig space). The lantern stays on
      `hand.L`.
    - **Report** `dwarf-miner-round-19-report.md`: frames, strike frame, the hand-meeting point, foot
      slide in mm on `Carry` (must match Walk's 0.000), the pick clearance, and the session's cost and
      turns for the ledger.
  - [ ] **2b — the seat (Wolf).** Wolf runs the round-19 session. Add a ledger row for the round, with its
    exact model id and cost. **Hard stop:** after two rounds his eye has not judged converging, stop and
    ask him whether it ships plain or is parked.
  - [ ] **2c — export, gate, promote.** Export with
    `blender --background … --python src-assets/blender/export_dwarf.py`
    (`export_animations=True` already exports every action, so check nothing stray is in the file).
    - Run `check_asset.py` on the export and confirm `anims=` lists Carry, Dig and Walk, each passing
      loop closure.
    - Promote to `assets/gltf/SM_VoxelDwarf_Miner01.glb` in its own commit, as previous rounds did, and
      re-run `check_asset.py` on the PROMOTED file.
    - Set `CARRY_OFFSET` from the report. `include_bytes!` means rebuild before any gui run.
  - [ ] `ingest.rs`:
    - `dwarf_clip_summary()` returns the clip NAMES parsed from the GLB's JSON chunk;
    - the startup line says `clips Walk, Dig, Carry`, or `clip <Name> ABSENT -- …` for each one missing;
    - `the_embedded_dwarf_carries_its_walk_clip` becomes a test that the embedded dwarf carries all
      three by name.
- [x] **Task 3: gui clip choice and drive (AC3-AC7).**
  - [x] Bind the clips **by name**: through `Assets<Gltf>` `named_animations`
    (`bevy_gltf-0.19.0/src/assets.rs:46`), or through `GltfAssetLabel::Animation(i)` with `i` resolved
    from the names Task 2 parses. `AnimationGraph::from_clips` gives one node per clip.
    `DwarfWalk` (`project.rs:305`) becomes the three nodes plus clips.
  - [x] A pure fn `fn dwarf_clip(entity: &protocol::Entity) -> DwarfClip` (`Walk | Dig | Carry`) holds
    AC3's rule. Store it in a component per dwarf, set in `reconcile_projection` from the mirror.
    Print the AC8 line when it changes.
  - [x] Generalise `start_dwarf_walk`/`drive_dwarf_walk` (`:2057`, `:2095`):
    - play the chosen node paused and `seek_to(phase × duration)`;
    - `Walk`/`Carry` read `WalkPhase`, and `Dig` reads a new `DigPhase { entered: u64 }` per AC4;
    - stop the other nodes;
    - keep every existing STALLED line, and add a `Dig` and `Carry` "clip never loaded" line.
  - [x] AC6 facing: when the mirror has a dwarf in `work` with `job: dig`, his heading is the yaw toward
    the target. Compute it where `DwarfHeadings::record` (`:2181`) decides headings, so there is one
    writer ([[spawn-is-not-the-only-writer]]).
  - [x] AC7 carried item: `pub const CARRY_OFFSET: Vec3` in `appearance.rs` beside `STONE_ITEM_SCALE`
    (`:233`).
    - While carried, the item's entity is parented to the dwarf at that offset.
    - Both item translation writers must skip a carried item: `reconcile_projection` (`:1955`) and the
      blend (`:2264-2269`).
    - On release, unparent the item and snap it to its cell.
  - [x] Headless tests in `gui/tests/headless.rs` (`headless_app`, no AnimationPlugin), each feeding
    deltas through the mirror:
    - AC3: a dwarf goes walk → dig → walk and walk → carry → walk; the component follows each delta.
    - AC4: deltas at ticks n..n+5 move `Dig`'s phase 0 → 1. Six frames that repeat one tick hold it
      constant. A snapshot resets it.
    - AC6: a dig to his east turns him east, and a channel leaves his heading as it was.
    - AC7: the carried item's parent is the dwarf and its local translation is `CARRY_OFFSET`. After
      release it is unparented, at `item_translation(pos)`.
- [x] **Task 4: the instrument (AC8).**
  - [x] `gui/tests/pixel_guard.rs` (it already spawns `simd`, `:150-190`): NEW
    `a_miner_logs_dig_and_a_hauler_logs_carry_from_a_real_daemon`.
    - The test opens its own TCP client and sends `first_delivery.py`'s three commands.
    - It runs `gui <port> --headless --frames N --capture <tmp>` and scrapes stderr.
    - It asserts, by id, a miner's `clip dig`, a hauler's `clip carry`, and a later `clip walk` for each.
    - Set N from a measured run, not a guess ([[frame-counts-are-venue-calibrated]]). The capture's own
      range checks may fire; the test reads stderr either way.
  - [x] Show its RED: bridge `job: None` makes it fail by name. Record that in the Debug Log.
- [x] **Task 5: tui (AC9).** The tui decodes the new fields and renders as today. Give one dwarf in the
  existing `capture_roster` stub (`tui/tests/client.rs:1563`) a `job` and a `carrying`, and assert its
  frame is unchanged.
- [ ] **Task 6: the record.**
  - [x] `_bmad-output/implementation-artifacts/mutations/12-5.sh`. Every row is KILLED by the test named:
    1. bridge `job: None` → the Task 1 serve test;
    2. bridge `carrying: None` → the Task 1 serve test;
    3. `dwarf_clip` prefers `Carry` over `Dig` (order swapped) → the AC3 headless test;
    4. Dig phase from wall time (`time.elapsed`) instead of ticks → the AC4 "repeat one tick" assert;
    5. the dig facing dropped → the AC6 test;
    6. the item blend writer not skipping a carried item → the AC7 test;
    7. clips bound by index (`#Animation0`) again → the AC2 names test, or the AC8 real-binary test
       (record which one killed it).
  - [x] README gui section: the work clips, and the `gui dwarf N clip …` line.
- [ ] **Task 7: the live recipe, the seat (AC10), then the full gate.**
  - [ ] Run the Verification recipe on the branch: GREEN, then the deliberate RED. Record both outputs.
  - [x] Write `12-5-signoff/vehicle-card.md` in the seat's launch form (see Verification).
    - Restart `simd 7451` before each part.
    - Say where on screen to look: the selected dwarf is centred, and his name is top-right under the
      clock.
  - [ ] Add a row to "Art ledger" for every seat round, with Wolf's words.
  - [ ] Full gate: `RUST_TEST_THREADS=1 scripts/gate.sh` (~50-75 min, [[gate-ooms-at-default-parallelism]]).

## Art ledger (FR45, M2-24)

| Round | Date | Clips | Made by (exact model / venue) | Cost | Wolf's verdict | Converging? |
| --- | --- | --- | --- | --- | --- | --- |

## Dev Notes

### Scope guardrails (do NOT)

- No sim-core change. No `WORK_TICKS` change unless Wolf rules it in Task 0.2.
- No new tui input or display.
- No cut clip: that is 12.8. No roster or profession UI in the gui: that is 12.6.
- No Bevy animation blending, masks or transitions. One node plays at a time.
- No pick-up or drop clip for a hauler's 5-tick `work` runs: he holds `Walk`, or `Carry` at a drop.
  Park ideas as issues ([[story-scope-move-forward]]).
- No model change: the rig, mesh, atlas and props stay r17. The seat adds actions only.

### What already exists (build on it)

- **The walk chain.** `WalkPhase` is accumulated from the drawn transform in `blend_entities`
  (`project.rs:2201-2272`). The player is held paused with `seek_to` every frame (`:2146`). Bevy applies
  the pose while paused (`bevy_animation-0.19.0/src/lib.rs:1207`).
- `TickClock::factor()` (`blend.rs`) is clamped to `[0, 1]`, so it saturates and holds while ticks
  repeat.
- `client_core::Mirror::tick()`/`speed()` (`client-core/src/lib.rs:148-154`). Every delta resends every
  entity, and `apply_delta` reports a dwarf as changed only when he differs (`:112-133`).
- `World::claims() / jobs() / carrying()` (`sim-core/src/lib.rs:1755-1785`).
- The `simd`-spawning harness and `dwarf_report` stderr scraping in `gui/tests/pixel_guard.rs:150, :901`.

### Key decisions & traps

- **Index binding breaks silently.** The glTF exporter writes actions in name order, so `Carry, Dig,
  Walk` makes `#Animation0` the **Carry** clip. Every dwarf would then "walk" with a carry pose and no
  test would notice. Bind by name.
- **`drive_dwarf_walk` seeks EVERY active animation** to the walk phase (`:2146-2148`). With three nodes
  it must seek only the chosen one.
- **Promotion is a separate act** ([[walk-cycle-pipeline-facts]] Trap 0). After the export, re-run
  `check_asset.py` on the PROMOTED file and rebuild the gui.
- **Measure the clip bound, not the blend.** The actions are saved unassigned. Bind the action AND its
  slot before any `checks()`, or you are measuring a static figure (Trap 1).
- **Two item writers.** The item's projected `Transform` is written at spawn/reconcile AND by the blend.
  A carried item must be skipped in both, or it flickers between the hand and the cell
  ([[spawn-is-not-the-only-writer]]).
- **The gui pixel guards have no work in them.** Boot frames have no designations, so no clip other
  than `Walk` plays. The guards have no reason to move with the new GLB unless the rest pose changes.
  Run the guard tier after promotion and treat any move as a finding.
- **The clip-switch line is chatty by design**: 5 dwarves, one line per switch. That is acceptable.
  Leave a `// NOTE:`.
- **Speed.** Fast4x makes a dig run 25 ms, which is invisible. The seat judges at Normal. Fast-forward
  only has to visibly speed the clips up.

### Project Structure Notes

- UPDATE `crates/protocol/src/lib.rs`, `crates/simd/src/bridge.rs`, `crates/simd/tests/serve.rs`
- UPDATE `crates/gui/src/{project.rs, ingest.rs, appearance.rs}`, `crates/gui/tests/{headless.rs, pixel_guard.rs}`
- UPDATE `crates/tui/tests/client.rs` (the test only), plus every `Entity` literal the compiler names
- NEW `src-assets/prompts/dwarf-miner-round-19.md` (dev), and from the seat `src-assets/blender/work_r19.py`,
  `dwarf-miner-round-19-report.md` and screenshots; UPDATE `src-assets/blender/SM_VoxelDwarf_Miner01.blend`,
  `assets/gltf/SM_VoxelDwarf_Miner01.glb`
- NEW `_bmad-output/implementation-artifacts/mutations/12-5.sh`, `12-5-signoff/vehicle-card.md`
- UPDATE `README.md`

### Previous story intelligence

- 12.4's protocol pattern carries over directly: an `Option` field with `skip_serializing_if`, an
  exhaustive bridge `match`, and pins in `protocol`. About 63 `Entity` literals needed fixing last time.
- Branch pushes use `push.sh --fast`, and the FULL gate runs only before the PR
  ([[push-branch-for-vehicle-testing]]). Kill test daemons by PID, never `pkill -x simd`: that took down
  a sibling layer's daemon in 12.4's review.
- An untracked draft mutation table blocks every commit. Draft rows outside `mutations/` until they are
  ready.

### References

- `_bmad-output/planning-artifacts/epics.md:2540-2558` (Story 12.5); PRD
  `prds/prd-frostvein-2026-09-28/prd.md:89-93` (FR45)
- `ARCHITECTURE-SPINE.md` (2026-08-09): AD-15 (`:107`), AD-17 rung 2 (`:146`)
- `src-assets/prompts/dwarf-miner-round-18.md` (brief), `src-assets/blender/walk_r18.py`,
  `export_dwarf.py`, `scripts/bench/check_asset.py:444-509` (animation clauses)
- [[walk-cycle-pipeline-facts]], [[walk-cycle-open-question]], [[delegated-blind-modelling-is-the-wrong-tool]],
  [[live-modelling-must-be-watchable]]

## Verification

**Wire (runs today, RED observed at creation):**

```bash
target/release/simd 7493 &   # fresh daemon, DEFAULT_SEED; kill it by PID afterwards
python3 _bmad-output/implementation-artifacts/12-5-signoff/work_wire.py 7493
```

- **RED** at creation, on `main` `f3b7cb3`: the keys are `id identity kind light pos profession state`,
  then `dig_ticks 0 carry_ticks 0 mismatches 125`, `NO JOB FIELD ON THE WIRE`, exit 1.
- **GREEN, required of dev:** `job` and `carrying` appear in the keys, with `dig_ticks` > 0 (expect
  125), `carry_ticks` > 0 and `mismatches 0`, then `WORK WIRE OK`.
- **Deliberate RED, required of dev:** mutation row 1 (bridge `job: None`) gives `NO JOB FIELD ON THE
  WIRE` again. Restore it before going on.

**Clips (the feature does not exist yet; the obligation is inherited):**

```bash
target/release/simd 7494 &
python3 _bmad-output/implementation-artifacts/12-5-signoff/work_wire.py 7494 normal 400 &   # designates; holds the socket
target/release/gui 7494 --headless --subdiv 4 --frames 1500 --capture "$SCRATCH/clips.png" 2>&1 \
  | rg 'gui dwarf asset|gui dwarf [0-9] clip'
```

- **Required of dev:** the startup line names `Walk, Dig, Carry`. Miners 2 and 4 log `clip dig` and then
  `clip walk`. Haulers 1 and 3 log `clip carry` and then `clip walk`. Count each and record the counts;
  zero of any of them is a failure, whatever the exit code.
- **Deliberate RED:** row 1 again gives no `clip dig` line at all.
- **At creation, today's binary prints `walk clip present` and no `clip` line.**
- If 1500 frames does not reach a delivery at Normal on lavapipe, raise it and record the figure.

**Seat (AC10), in the seat's launch form:**

- WSL: `simd 7451`.
- PowerShell: `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--select','2')`, which follows Bifur
  (miner, red). Press `2` and drag channel marks on the camp floor east of the fire. Press `3` and drag a
  3×3 stockpile west of it. Watch Bifur swing at each mark at Normal speed.
- Restart `simd 7451`, then launch with `--select','1'` to follow Ori (hauler, green). Watch him walk a
  stone to the pile, holding it.

## Dev Agent Record

### Agent Model Used

- Orchestrator + verifier: Claude Opus 5.5 (`claude-opus-5-5[1m]`). Wolf picked Sonnet 5.5 subagents
  for the implementation (2026-10-03), as in 12.2-12.4.
- Agent A (Sonnet 5.5): Tasks 1 and 5. Agent B (Sonnet 5.5): Task 3 and the `ingest.rs` part of 2c.
- The orchestrator wrote the Task 2a brief, ran every mutation and ran the live recipe.

### Debug Log References

- **Task 1 RED:** with the bridge still sending `job: None`, the serve test
  `deltas_label_a_miners_dig_a_haulers_haul_and_the_stone_he_carries` failed `tick 61: miner 4 works
  with job None` (`serve.rs:635`). It went green with the bridge.
- **Task 5:** passes first run, as expected. The tui already decoded `Entity` and draws neither
  field, so there is no RED to show. The test pins that it stays so.
- **Mutation rows 1-2** (`mutations/12-5.sh`, run alone through `scripts/mutate.sh`, 2026-10-03):
  - row 1 "the bridge sends no job" KILLED at the named assert, `serve.rs:635` (`miner … works with
    job None`);
  - row 2 "the bridge sends no carried stone" KILLED at `serve.rs:677`, the closing
    `need all three` panic (no `carrying` was ever seen in 1500 deltas).
- **Live wire recipe GREEN** on `d448069` (fresh release `simd 7493`, `work_wire.py`): keys
  `carrying id identity job kind light pos profession state`; miner work runs 25 × 5 ticks, hauler
  work runs 19 × 5 ticks; `dig_ticks 125 carry_ticks 791 mismatches 0`, `WORK WIRE OK`, exit 0.
- **Live wire recipe deliberate RED** (row 1, built in a throwaway scratch worktree with its own
  target dir so the sabotage could not reach the shared tree, then removed): keys lack `job`;
  `dig_ticks 0 carry_ticks 791 mismatches 125`, `NO JOB FIELD ON THE WIRE`, exit 1.

- **Task 3 REDs** (Agent B): AC3 and AC4 compile-RED first (`DwarfClip` and `DigPhase` did not
  exist), then green. AC6 behavioural RED at the east dig: `drew Quat(0,0.707,0,0.707), wanted
  Quat(0,-0.707,0,0.707)`. AC7: `a carried stone is his child left: None right: Some(35v0)`.
- **Mutation rows 3-7** (run alone through `mutate.sh`, 2026-10-03, on `70fe281`). Each was killed at the
  assertion that targets it:
  - row 3 "Carry is preferred over Dig": KILLED, `digging_outranks_carrying_in_the_clip_choice`,
    `headless.rs:5019`;
  - row 4 "the dig phase runs on wall time": KILLED in
    `the_dig_phase_runs_on_delivered_ticks_and_holds_when_the_ticks_repeat`, first at the tick-formula
    assert (`headless.rs:5087`), not at the "repeat one tick" assert the story names. **Trap 1**,
    so I re-ran it with that test's five earlier asserts disarmed. The hold assert then kills it
    alone at `headless.rs:5116`, "a repeated tick must not move the swing". The test advances virtual
    time 100 ms per frame (`TimeUpdateStrategy::ManualDuration`), so the hold is not vacuous;
  - row 5 "a digging dwarf no longer faces his target": KILLED, the AC6 rotation assert in
    `a_digging_dwarf_faces_his_target_and_a_channel_keeps_his_heading` (`headless.rs:5166`);
  - row 6 "the blend writer moves a carried stone": KILLED at `held.translation == CARRY_OFFSET`
    (`headless.rs:5259`), asserted after four frames of `blend_entities`;
  - row 7 "clips are bound by index again": KILLED by the NEW unit test
    `ingest::tests::each_clip_binds_the_label_of_its_own_name_in_export_order` (`ingest.rs:5557`).
- **Row 7 deviation:** the story offered the AC2 names test or the AC8 real-binary test as the
  killer, and **neither can kill it**. The names test checks which names the GLB carries, not
  which index is loaded. AC8's `clip …` lines come from the wire choice, not the bound clip. Agent
  B's version also had no killer even after promotion. So I extracted the lookup into
  `ingest::dwarf_clip_label(names, wanted)` and pinned it on the exporter's real order
  `[Carry, Dig, Walk]`, where Walk is `Animation2`.

- **Task 4 (AC8), `81f5482`, Agent C:** `a_miner_logs_dig_and_a_hauler_logs_carry_from_a_real_daemon`
  (`#[ignore]`d with the other pixel guards, so the full gate's tier runs it).
  - **GREEN**, re-run independently by the orchestrator with identical counts. The crew was derived
    from the wire: 0 woodcutter, 1 and 3 haulers, 2 and 4 miners. `clip dig` miner 2 ×12 and
    miner 4 ×7, each followed by `clip walk` (12, 7). `clip carry` hauler 1 ×4 and hauler 3 ×3,
    followed by `clip walk` (3, 3). gui exit 0, test 60.68 s.
  - **Frame count:** `--frames 100`. Every required line was already present at `--frames 1`. The
    run length (~60 s) is set by the capture's 100-delivered-tick floor, which runs on wall-clock
    ticks, so a loaded machine is the flake risk. The failure message prints every clip line seen.
  - **Deliberate RED** (bridge `job: None`, applied after the commit, restored, `git diff` empty):
    `panicked at crates/gui/tests/pixel_guard.rs:1144:5: no miner [2, 4] logged \`clip dig\`
    followed by \`clip walk\``, with miners dig `[(2, 0), (4, 0)]`. The haulers still logged carry,
    because `carrying` was untouched.

- **Clips recipe** (Verification, pre-promotion), release `simd 7494` + `work_wire.py 7494 normal 400`
  + `gui 7494 --headless --subdiv 4 --frames 1500 --capture …`, on `b1d0966`. gui exit 0 in 951 s.
  - The wire said `dig_ticks 90 carry_ticks 310 mismatches 0`, `WORK WIRE OK`.
  - Clip lines: `gui dwarf 2 clip dig` ×11 and `clip walk` ×11; dwarf 4 the same, 11/11. `gui dwarf 1
    clip carry` ×5 and `clip walk` ×5; dwarf 3 the same, 5/5. No miner logged carry, and no hauler
    logged dig.
  - The startup line said `clip Dig ABSENT …; clip Carry ABSENT …`, which is correct on the Walk-only
    GLB. The recipe's "startup line names `Walk, Dig, Carry`" waits on 2c's promotion.

### Completion Notes List

- **Task 1:** `protocol::DwarfJob` (externally tagged, snake_case) and `Entity.job` / `Entity.carrying`
  as the last two fields, both skipped when `None`. Emitter and idle-dwarf lines stay
  byte-identical; the existing pins are unchanged. Bridge: `dwarf_job` is an exhaustive match with no
  wildcard; `dwarf_entities` joins `claims()` to `jobs()` by `JobId` and reads `carrying()`. No
  sim-core or save change. The serve test also asserts `profession` on every dwarf delta, which
  closes 12.4's deferral. On every sample, a channel's target equalled the miner's own cell, and no
  claim named a job missing from `jobs()`.
- **Task 5:** `capture_roster` became `capture_roster_with(no_color, at_work)`. With `at_work`,
  dwarf 1 gets `job: Haul` and `carrying: Some(77)` in every message, and stdout is byte-identical
  to the control.
- **Task 2a:** brief `src-assets/prompts/dwarf-miner-round-19.md`. It corrects one creation figure:
  the strike does not land "one cell (1.6 m) in front of him". A Dig's rock face is the near face
  of the next cell, **0.8 m** ahead of his origin, and a Channel's floor is under his own feet, so
  the brief asks for a floor-to-knee strike 0.5–0.8 m ahead. It also tells the seat that the drawn
  stone is a **0.64 m cube** (`STONE_ITEM_SCALE` 0.4 × 1.6 m cells), more than half the dwarf's
  1.2 m. If that cannot read as held, scaling the carried stone is Wolf's call.

- **Task 3** (Agent B, five commits `6bc748a`..`44b88cb`, then my `70fe281`):
  - **Clip names:** read from the embedded GLB's JSON chunk in array order (`glb_clip_names`).
    `dwarf_clip_summary()` returns them, and the startup line says `clips Walk, Dig, Carry` or
    `clip <Name> ABSENT -- …`. Each clip loads `#Animation{i}` for its own name's index through
    `dwarf_clip_label`. NOTE: the names come from the EMBEDDED bytes, so `--assets <dir>` with a
    differently ordered GLB would bind wrongly.
  - **Missing clip:** a chosen clip that has not loaded plays as `Walk`, and Dig/Carry each get a
    "clip never loaded" STALLED line at frame 180. Agent B's real-binary smoke run on today's
    Walk-only GLB printed both ABSENT clauses and both STALLED lines; Walk played, with no panic.
  - **`sync_dwarf_work`:** a new system that runs after the blend, because it needs the clock
    factor. It sets `DwarfClip` and `DigPhase`, prints `gui dwarf {id} clip {walk|dig|carry}` on
    each change (none for the initial Walk), and parents or unparents carried stones.
  - **Dig phase:** `entered` lives in `DwarfHeadings.1`, beside the headings; `.0` stays the
    headings because the 10.5 mutation row quotes `headings.0.get`. A snapshot resets `entered` to
    the snapshot tick. `DigPhase.phase` is stored only so the headless test can read the computed
    value.
  - **AC6 facing:** in `DwarfHeadings::record`, after the position-derived facing. It reuses
    `entity_draw_rotation` from his cell to the target.
  - **AC7:** the carried stone is a `ChildOf` the dwarf at `CARRY_OFFSET`, with local scale
    `STONE_ITEM_SCALE / METRES_TO_CELLS`, because the dwarf entity is scaled 0.625 and the drawn
    size must stay 0.4. The blend writer skips any item with a parent; the spawn writer runs only
    once. Release resets it to `item_translation(pos)`.
  - **Despawn:** Bevy despawns children recursively, so a stone would go with its dwarf. That is
    unreachable today: the sim never despawns dwarves, and a slice hides dwarf and stone together.
    Reconcile would respawn the stone a frame later anyway, so it is left unhandled.
  - **`CARRY_OFFSET` is PROVISIONAL**, `(0, 0.7, -0.3)` in the dwarf's glTF/Bevy local metres. I
    corrected its doc: the seat reports BLENDER rig space, and glTF maps Blender `(x, y, z)` to
    `(x, z, -y)`. So a reported `(bx, by, bz)` goes in as `Vec3::new(bx, bz, -by)`. Agent B's doc
    said "no conversion", which would have put the stone at the wrong height and depth.
  - **2c split:** `the_embedded_dwarf_carries_its_walk_clip` now asserts `Walk` BY NAME. The
    all-three upgrade lands with the promoted GLB in 2c; before then it would turn the gate red.

- **Task 6 README:** a "Work clips" paragraph in the gui section, covering the clip rule, the
  by-name binding and startup clause, the `gui dwarf <id> clip …` line, and judging at Normal.
- **Task 7 card:** `12-5-signoff/vehicle-card.md` is written in the seat's launch form. It starts
  with the startup-line check that the promoted GLB is in the build, and has two parts, Bifur
  `--select 2` and Ori `--select 1`, each on a restarted `simd 7451`. Each part says where to look
  and asks one question.

### File List

- `crates/protocol/src/lib.rs`
- `crates/simd/src/bridge.rs`
- `crates/simd/tests/serve.rs`
- `crates/client-core/src/lib.rs` (literal fixes only)
- `crates/gui/src/capture.rs`, `crates/gui/tests/capture.rs` (literal fixes only)
- `crates/gui/src/ingest.rs`, `crates/gui/src/project.rs`, `crates/gui/src/appearance.rs`
- `crates/gui/tests/headless.rs`
- `crates/tui/src/view.rs` (literal fixes only)
- `crates/tui/tests/client.rs`
- `src-assets/prompts/dwarf-miner-round-19.md` (new)
- `crates/gui/tests/pixel_guard.rs`
- `README.md`
- `_bmad-output/implementation-artifacts/12-5-signoff/vehicle-card.md` (new)
- `_bmad-output/implementation-artifacts/mutations/12-5.sh` (new)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-03 | Story created on `f3b7cb3`. RED observed on the live wire (`work_wire.py`: no `job` or `carrying`, 125 miner work ticks unlabelled). The GLB carries `Walk` only. Task 0 ruled by Wolf the same day: `job` + `carrying`, keep 5 work ticks, live BlenderMCP seat from round 1. |
