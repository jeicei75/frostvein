---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: 32e693317f08f3319f52596637fba30c4488f26d
---

# Story 8.1: Point at the World

Status: ready-for-dev

## Story

As the boss,
I want to point at a block in the 3D view and see exactly which one I am pointing at,
so that I can trust where my orders will land before I give any.

## No sign-off gate on this story — read before looking for a Task 0

UX-DR22 applies to **8.3 and not to 8.1–8.2**, decided in the epic rather than left to
inference [epics.md:1014]. The hover highlight is **legibility** work on a look 5.4 and 7.2
already settled, governed by UX-DR17 and UX-DR18. So there is **no Task 0 artifact, no
`8-1-signoff/` directory, and no closing-half AC**. Task 6 is still vehicle-bound, because
NFR6 must be re-measured with picking live — but Wolf measures, he does not sign off a look.

**Do not re-tune any look constant to make a capture pass.** M2-2 is open and carries the
gfx pass's inherited targets; a look change needs a concrete defect, not a preference.

## The live vehicle — unchanged, do not re-derive

**gingerspice**: cross-compiled `gui.exe` on native Windows, NVIDIA Vulkan, `simd` in WSL
over localhost. **No devpod can open a window** — measured at 5.3, both fallbacks walked to
the end. Build recipe is in Verification.

**REBUILD `gui.exe` BEFORE THE SESSION, AND SAY SO IN THE RECORD.** The stale-binary trap
fired three times in 5.4 alone; once it cost a whole vehicle session, because the `.exe` was
built at 13:24 and the earliest patch commit landed at 13:58, so the "live check" showed no
change at all. **M2-7 is still open** — there is no build script and no SHA stamp in `gui`
(verified 2026-08-25: `scripts/` holds only `audit-mutations.py`, `codex-handoff.sh`,
`gate.sh`, `mutate.sh`, `task6-designate.py`; `rg 'GIT_SHA|git_sha|build_sha|vergen'` over
`crates/gui/src/` returns nothing). Nothing in the delegated dev flow triggers the rebuild.

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier, not `--fast`) is green on a cold rebuild, and the
   diff is confined to this story's own commit range from `baseline_commit`.

### The ray

2. Exactly one screen-ray-to-tile path exists in `gui`. It takes its ray from the rendering
   camera via `Camera::viewport_to_world`, and the only sim↔render axis conversion it
   performs is a call to `transform::render_to_world`. *Mechanism is load-bearing here: the
   spine's "no system does its own axis math" convention is the requirement, and a second
   projection that drifts from the camera that drew the frame puts the highlight on the wrong
   tile* [ARCHITECTURE-SPINE.md:194].

### The tile it picks

3. At any orbit yaw, any pitch, any distance in `4.0..=500.0`, and any slice level, pointing
   at a visible block picks the block a player would say they are pointing at — including on
   sliced underground levels.
4. The tile picked is always one the current slice admits, and never one hidden behind a
   nearer visible tile. A tile the slice hides is never picked.
5. A hover highlight marks the picked tile in the rendered frame before any command is
   issued, and it reads as distinct from 7.2's designation marks at working zoom — asserted
   as a colour separation against hand-written literals, and staying clear of the near-white
   reserved for stars and emitter faces.

### Nothing picked is nothing picked

6. With the cursor over empty sky, over a tile the current slice hides, or outside the
   window, nothing is picked and no highlight is drawn. The pick yields no tile rather than
   falling back to a default such as the origin.

### Headless (AD-17 rung 2)

7. Under `MinimalPlugins` in `cargo test`, a known camera pose plus a known cursor coordinate
   resolves to the expected tile, asserted across orbit angles, zoom distances and slice
   levels.
8. The transform round-trip pin is extended to cover the picking path: a tile's own screen
   position, computed independently by `CameraRig::project_world_point`, is picked back to
   that same tile — projection and pick proven mutually inverse.

### Client-local, never wire

9. Picking and the highlight are entirely client-local. No command is issued, nothing about
   the cursor or the pick reaches the wire, and the highlight entity carries `ClientLocal`
   rather than `WorldProjected`.

### The instrument

10. `gui --capture <path> --frames N --z N --cursor <x>,<y>` places the cursor at a scripted
    viewport coordinate and prints the tile it picked **and** the tile it expected, then
    asserts they match. It reports the mismatch rather than exiting 0.
11. The instrument has its own test: the reported pick **changes** when the scripted cursor
    moves, and the instrument says so explicitly when nothing is picked instead of emitting a
    well-formed line that proves nothing.

### Measured on the vehicle

12. On the live vehicle (gingerspice / native-Windows `gui.exe` / NVIDIA Vulkan, `simd` in
    WSL over localhost), with picking active on the full 128×128×32 world, all dwarves and
    all lights, NFR6 still holds: sustained **60 fps at working zoom** and **≥30 fps at full
    vista**, read from the frame-time overlay.

### Evidence

13. A sabotage table at `_bmad-output/implementation-artifacts/mutations/8-1-point-at-the-world.sh`
    covers every seam AC above; every mutation is KILLED and the RED output is pasted into
    the Dev Agent Record with the assertion that went red per row.

## Tasks / Subtasks

- [ ] **Task 1 — The pick path (AC: 2, 3, 4, 6)**
  - [ ] New `crates/gui/src/pick.rs`. One public entry point; no other module gains screen or axis math.
  - [ ] Query `(&Camera, &GlobalTransform)` and the primary `Window`; take `cursor_position()`.
  - [ ] `camera.viewport_to_world(global_transform, cursor)` → `Ray3d` in render space. On `Err`, pick nothing.
  - [ ] DDA-march the ray through **integer render-space cells**, testing each against the mirror via the slice-visibility rule. Stop at the first visible hit.
  - [ ] Convert the hit **cell centre** — a voxel-aligned `Vec3` — through `transform::render_to_world`. Never the raw hit point (see D2).
  - [ ] Bound the march so a ray into empty sky terminates: cap at the world's diagonal extent, not at an arbitrary step count.
  - [ ] Store the result in a client-local resource holding `Option<[i32; 3]>`.
- [ ] **Task 2 — The highlight (AC: 5, 9)**
  - [ ] Spawn/despawn a single highlight entity following the picked tile; despawn when nothing is picked.
  - [ ] Tag it `ClientLocal` **at spawn** — `classify_client_local` runs at `PostStartup` (`ingest.rs:183`) and will not see an entity spawned later in `Update`.
  - [ ] Colour it from `appearance.rs`, beside the mark colours — never a literal at the draw site.
  - [ ] Test the colour separation against hand-written literals, following `mark_colours_are_distinct_cold_literals`.
- [ ] **Task 3 — Headless tests (AC: 7, 8)**
  - [ ] Extend `crates/gui/tests/headless.rs` with a camera-bearing harness (skeleton in D3).
  - [ ] Assert known pose + known cursor → expected tile across at least: three orbit yaws, three distances spanning the 4.0..=500.0 clamp, and three slice levels including one underground.
  - [ ] Assert the three nothing-picked cases from AC6 separately — sky, slice-hidden, cursor outside the viewport.
  - [ ] Add the mutual-inverse test of AC8. Mind the units: `project_world_point` returns **normalized** coords (0..1, y down, `camera.rs:76`) while `viewport_to_world` takes **viewport pixels** — multiply by the physical size you pinned in D3. **If it fails, suspect `BOOT_ASPECT_RATIO` before suspecting the pick** (D6) — and report it, do not paper over it.
- [ ] **Task 4 — The instrument (AC: 10, 11)**
  - [ ] Add `--cursor <x>,<y>` to `parse_args_from` (`ingest.rs:248`). Validate it requires `--capture`, matching the existing `--distance` shape (`ingest.rs:301-303`).
  - [ ] **A typo'd flag is silently swallowed as the TCP port** (`ingest.rs:288-290`) and fails as an invalid port. Reject an unparseable `--cursor` value explicitly.
  - [ ] Print `pick: cursor=(x,y) picked=[x,y,z] expected=[x,y,z]` and assert equality; the expected tile comes from the independent forward projection, not from the pick.
  - [ ] Test the instrument itself: two different scripted cursors produce two different picks, and the no-pick case prints its own distinct line.
- [ ] **Task 5 — Sabotage table (AC: 13)**
  - [ ] Write `_bmad-output/implementation-artifacts/mutations/8-1-point-at-the-world.sh` in the house format — `assert s.count(old) == 1` guard on every edit.
  - [ ] Minimum rows: the pick system deleted from `client_systems`' tuple; the slice-visibility filter removed from the march; `render_to_world` replaced by raw truncation of the hit point; the nothing-picked branch replaced by a fallback to `[0,0,0]`; the highlight's despawn-on-no-pick removed; `--cursor` parsed but never reaching the pick.
  - [ ] **Commit before running** (M2-9). Run `scripts/mutate.sh` **alone** — it is not concurrency-safe. Capture the exit code before any pipe.
  - [ ] **Dry anchor-check first** (M2-8): grep every `old =` string against the live tree before the run.
- [ ] **Task 6 — VEHICLE-BOUND: NFR6 with picking live (AC: 12)**
  - [ ] **Rebuild and re-copy `gui.exe` first**, and record the build time and the commit it was built from.
  - [ ] Read sustained fps at working zoom and at full vista from the F3 overlay, with the cursor moving over the world.
  - [ ] Paste both figures labelled `gingerspice / native Windows / NVIDIA`. A failed reading is the finding and gets reported, not worked around.
- [ ] **Task 7 — The gate (AC: 1)**
  - [ ] `cargo clean -p gui`, then `scripts/gate.sh` full tier. Paste the tail. A `GATE GREEN (FAST)` line is a coverage hole, not a pass.

## Dev Notes

### The epic's premises, verified against source 2026-08-25

Five of five M2 epic premises checked before this story have been wrong, so all four of 8.1's
were re-verified against the tree rather than inherited. **All four hold** — with one
correction and one trap.

- **`render_to_world` exists** — `crates/gui/src/transform.rs:9`, `pub fn render_to_world(value: Vec3) -> [i32; 3]`.
  **Correction to the epic's word "existing": it has zero production callers today.** Its
  only three call sites are test oracles in `crates/gui/tests/headless.rs:194,223,235` plus
  its own round-trip pin. This story makes it production code for the first time.
- **The round-trip test exists** — `coordinate_transform_round_trips_a_spread`,
  `crates/gui/src/transform.rs:20`. It lives in `src/`, not `tests/`, so extending it means
  editing `src/transform.rs`. A sibling handedness pin sits at `transform.rs:27` and must
  keep passing.
- **`--capture`, `--frames`, `--z` all exist** — `ingest.rs:257`, `:260`, `:270`. Parsing is
  in **`ingest.rs:248`, not `main.rs`** (`main.rs` is five lines).
- **NFR6's machine is already corrected** in the epic text (M2-4, 2026-08-23). No stale WSLg
  wording survives in 8.1's ACs.
- **Reported separately, not fixed here:** `docs/architecture.md:32` and `:127-129` still
  describe `gui` as running "via WSLg" and state the NFR6 bar against "the WSLg devpod".
  M2-4 corrected `epics.md` and the spine but missed the companion doc. Outside this story's
  diff.

### Key decisions & traps

**D1 — The ray comes from the rendering camera, not from `CameraRig`.** Ruled by Wolf,
2026-08-25. `Camera::viewport_to_world` (`bevy_camera-0.19.0/src/camera.rs:647`) is usable
because the camera really is a `Camera3d` with `Projection::Perspective` (`ingest.rs:321-325`).
The alternative — inverting the hand-rolled `project_render_point` — was rejected because it
would be the **third** copy of the frustum math (`camera.rs:76`, `atmosphere.rs:213`, and the
new one) and any drift from the real camera lands the highlight on the wrong tile.

**D2 — `render_to_world` TRUNCATES; it does not floor, and it must only ever see
voxel-aligned points.** This is the trap most likely to ship a half-wrong feature.
`Cuboid::default()` (`project.rs:177`) is centred on its translation, and translation is
exactly `world_to_render(position)` (`project.rs:416`), so voxel *p* occupies the render-space
box *p* ± 0.5. `render_to_world` is `[value.x as i32, -value.z as i32, value.y as i32]`
(`transform.rs:10`) — `as i32` truncates toward zero. Feed it a raw ray-hit point and **half
of every voxel resolves to its neighbour**: a hit at render z −4.8 sits inside voxel world
y = 5 but yields 4. Its own doc comment says it takes "a voxel-aligned Bevy position"
(`transform.rs:8`). **March integer cells and convert the cell centre.**

**D3 — `MinimalPlugins` gives you no camera and no transforms; build them by hand.**
`MinimalPlugins` is `TaskPoolPlugin`, `FrameCountPlugin`, `TimePlugin`, `ScheduleRunnerPlugin`
— no `TransformPlugin` (so `GlobalTransform` is never propagated) and no `camera_system` (so
`Camera.computed.clip_from_view` is never populated, and `viewport_to_world` silently reads a
zeroed matrix). `Camera.computed` is `pub` (`bevy_camera-0.19.0/src/camera.rs:393`), as are
`clip_from_view` and `target_info` (`:219-220`). Bevy's own unit test shows the construction
(`bevy_camera-0.19.0/src/camera.rs:1076-1094`):

```rust
// in tests: what camera_system + TransformPlugin would have done in production
let mut camera = Camera::default();
camera.computed.target_info = Some(RenderTargetInfo { physical_size: UVec2::new(1920, 1080), scale_factor: 1.0 });
let mut projection = PerspectiveProjection { fov: BOOT_VERTICAL_FOV, ..default() };
projection.update(1920.0, 1080.0);
camera.computed.clip_from_view = projection.get_clip_from_view();
let global = GlobalTransform::from(rig.transform());   // written by hand, not propagated
```

**D4 — Register the pick system in `client_systems` (`ingest.rs:170`), nowhere else.** That
and `projection_systems` (`ingest.rs:132`) are the shared registration points the live app and
the headless harness both drive. A system added anywhere else is invisible to the suite —
6.1's inert-seam defect, which then recurred as the top-severity finding in four consecutive
stories. **M2-1 closed this class at the root specifically so 8.1 could ride it**: its success
criterion was "verified by a mutation row before 8.1 is dev'd", and the retro names 8.1's
picking path as "exactly the kind of single-call-site system that has gone inert five times."

**D5 — In production the pick must run after transform propagation.** Bevy's own docs for
`viewport_to_world` warn that the camera's global transform must be up to date. Schedule it in
`PostUpdate` after `TransformSystems::Propagate`, and note that this ordering does not exist
under `MinimalPlugins` — which is exactly why D3's harness writes the transform by hand.

**D6 — If AC8's mutual-inverse test fails, suspect the hand-rolled projection first.**
`camera.rs:30` hardcodes `BOOT_ASPECT_RATIO = 16.0/9.0`, while the real camera derives aspect
from the actual viewport. Pin the test viewport to 16:9 so the two agree. **If they still
disagree, that is a finding about `project_render_point` being wrong on non-16:9 windows
today — report it in the Dev Agent Record; do not adjust the pick to match a suspect oracle.**

**D7 — Assert observable effects, never registration.** Seven tests landed under M2-1 doing
exactly this. A test that checks "the system is registered" is the vacuity M2-11 names. Drive
the pick by writing a real cursor position and running `app.update()`, never by inserting the
picked-tile resource directly — 6.1's four seam tests all passed whether or not production
drove them, and three one-line deletions killed the feature with the suite green.

**D8 — Expected tiles are hand-written literals or come from the independent forward
projection.** Never from the pick itself. The self-referential-test antipattern has landed at
1.1, 1.2, 1.3 and 6.1.

**D9 — `--frames` is not ticks, and the conversion is fps.** `ticks = frames ÷ fps × 10`
(`capture.rs:523-536` counts `Update` runs, not ticks). Measured at 7.2 on this vehicle: the
same `--frames 1500` gave 58 ticks on a light scene and 237 on a heavy one. **Do not copy
1500 from 7.2's block.** This story's capture asserts a pick, not motion, so it does not need
the ≥100-tick floor — but state the frame count you used and why. Building `--capture-at-tick`
is M2-15's work and rides on **8.2**, not here.

### Scope guardrails — do NOT build these here

- **No commands, upstream, of any kind.** `gui` is receive-only and structurally cannot send:
  the `TcpStream` is consumed by `BufReader` at `ingest.rs:86` and moved into the reader
  thread at `ingest.rs:92`; no write handle survives. **Do not restructure `run()`'s stream
  ownership** — that is 8.2's work.
- **No drag, no rectangles, no modes, no hint bar.** All 8.2.
- **No mouse buttons and no wheel.** Hover only. The wheel is still unclaimed in code and
  **still unruled by Wolf from 7.1**; claiming it here costs a migration when UX-DR2 brings
  wheel zoom. Leave the decision where 7.1 left it.
- **Do not enable the `bevy_picking` / `mesh_picking` Cargo features.** The crate is in the
  lockfile via `bevy_dev_tools` (`Cargo.lock:1091`) but `bevy::picking` is not reachable
  through the facade, and enabling it was considered and rejected.
- **Do not touch `client-core`.** `rect_on_level` (`client-core/src/lib.rs:188`) already
  exists and is shared; 8.2 uses it. This story adds nothing there.
- **No look tuning** (M2-2 open).

### What already exists (build on it, do not re-derive)

- **Transform pair** — `world_to_render` (`transform.rs:4`), `render_to_world`
  (`transform.rs:9`), plus round-trip (`:20`) and handedness (`:27`) pins.
- **Camera** — `CameraRig { focus, yaw, pitch, distance }` (`camera.rs:33`) as a `Component` on
  the camera entity; `orbit` (`:50`), `zoom` (`:55`, clamps `4.0..=500.0`), `transform` (`:59`);
  forward-only projections `project_render_point` (`:76`) and `project_world_point` (`:92`).
- **Slice visibility** — `SliceLevel` (`slice.rs:6`, client-local, `level()`/`top()`/`set()`/`step()`);
  the single hide predicate `is_visible_at_slice` (`project.rs:836`, **private** — the pick
  needs it, so widen its visibility rather than writing a second copy) and its public wrapper
  `terrain_positions_at` (`project.rs:806`).
- **Registration + CLI** — `client_systems` (`ingest.rs:170`), `projection_systems` (`:132`),
  `capture_systems` (`:199`), `parse_args_from` (`:248`) with its validation block (`:292-303`).
- **Headless harness** — `live_app` (`headless.rs:1996`) is the pattern to copy; it drives
  `client_systems` and supplies by hand exactly what `DefaultPlugins` provides in production.
  Also `apply_delta` (`:83`), `apply_snapshot` (`:98`), `snapshot_with_dims` (`:118`).
- **Mirror** — `tile(pos) -> Option<Tile>` (`client-core/src/lib.rs:119`), `dims()` (`:107`).

### Project Structure (files to touch)

| File | NEW/UPDATE | What |
| --- | --- | --- |
| `crates/gui/src/pick.rs` | NEW | The single screen-ray-to-tile path and the picked-tile resource |
| `crates/gui/src/lib.rs` | UPDATE | `mod pick;` |
| `crates/gui/src/ingest.rs` | UPDATE | Register the pick + highlight systems in `client_systems`; `--cursor` in `parse_args_from` and its validation |
| `crates/gui/src/project.rs` | UPDATE | Widen `is_visible_at_slice` for the pick; spawn/despawn the highlight |
| `crates/gui/src/appearance.rs` | UPDATE | Highlight colour, beside the mark colours |
| `crates/gui/src/transform.rs` | UPDATE | Extend the round-trip pin to the picking path (AC8) |
| `crates/gui/src/capture.rs` | UPDATE | Instrument line + the pick assertion |
| `crates/gui/tests/headless.rs` | UPDATE | Camera-bearing harness and the AC7/AC8 tests |
| `_bmad-output/implementation-artifacts/mutations/8-1-point-at-the-world.sh` | NEW | Sabotage table |
| `_bmad-output/implementation-artifacts/metrics/8-1-point-at-the-world.md` | NEW | Ledger rows (written by the workflow, not by hand) |

### Previous story intelligence (deltas that change THIS story)

- **Branch from `main`.** 7.2 merged (PR #31/#32) and forge-process 1.2.0 merged (PR #33,
  `32e6933`); the working tree is on `main`, clean. The stacked-branch rule still applies to
  AC1's diff scope: prove it against **this story's own commit range**, never against `main`
  or a branch tip.
- **7.2's instrument photographed an empty site and exited 0** — all 50 designations were
  genuinely projected, so a counter could not catch it. AC10 therefore asserts
  `picked == expected`, not `picked.is_some()`. This is M2-11: non-zero evidence **of the
  story's own headline outcome**.
- **`cargo clean -p gui` after a mutation round** was mandated at 7.1 and 7.2 because
  `mutate.sh` poisoned the build cache. M2-16 fixed the root cause on 2026-08-23 (`tar -xmf`),
  so the clean may now be redundant — keep it this once and say in the record whether it was
  still needed.

### Verification

**Executed at story creation, 2026-08-25** — the full gate on `32e6933`, clean tree, cold
cache:

```
frostvein gate
  cargo fmt --check           ok
  cargo clippy -D warnings    ok
  cargo test                  ok
  tui has no sim-core edge                ok
  client-core has no sim-core edge        ok
  gui has no sim-core edge                ok
  metrics ledger tests        ok
  mutation tables still apply ok
GATE GREEN
```

**Not executable at story creation — the feature does not exist yet.** The obligation is
inherited: the dev agent must run each of these and paste the non-zero observation named
beside it.

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# 1. Headless (AC 7, 8) — must name the poses and levels it covered, not just pass
cargo test -p gui pick

# 2. Sabotage table (AC 13) — commit first; run alone; exit code before any pipe
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/8-1-point-at-the-world.sh
cargo clean -p gui

# 3. The gate (AC 1) — full tier
scripts/gate.sh
```

Vehicle side (Task 6), after the mandatory rebuild:

```bash
# WSL
cargo build -p simd -p tui
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
./target/debug/simd 7451          # port is positional; seed is fixed in the binary

# Windows, after copying target/x86_64-pc-windows-gnu/release/gui.exe across
gui.exe 7451 --capture 8-1-pick-working.png --frames <justified> --z 10 --cursor 960,540
```

**Required observation, not exit 0.** The capture must print a `pick:` line whose `picked`
and `expected` tiles are equal and are **not** `[0,0,0]`, and the PNG must show the highlight
on that tile. Match the line by **prefix** — 7.1 changed the draw-set oracle's shape and older
recipes quoting whole lines stopped matching. Then press **F3** and read sustained fps at
working zoom and at full vista.

### Branch and commits

Branch `8-1-point-at-the-world`, cut from `main`. Author every commit
`Völundr <jeicei75@gmail.com>`. **Commit at minimum once per completed task, ideally on each
green** — never one squashed commit; the pre-commit hook runs `scripts/gate.sh --fast`, so
each commit is individually gate-green, and the pre-push hook runs the full gate.
Review-gated: **no push, no PR** until Wolf says so.

### If this overruns one session

Split at the instrument. Tasks 1–3 (the pick path, the highlight, the headless tests) are a
complete vertical slice with observable behaviour; Tasks 4–7 (the scripted-cursor flag, the
sabotage table, the vehicle measurement, the gate) become the continuation. **Restate the RED
evidence in the continuation handoff** — 1.2 lost it across a session boundary.

**Self-gate findings land in the Dev Agent Record, fixed or not** (M2-10). A finding that
exists only in a handback message is lost at the session boundary.

### References

- Story text and Epic 8 framing — `_bmad-output/planning-artifacts/epics.md:1004-1055`
- FR35–FR37 — `epics.md:79-84`; NFR5–NFR8 — `epics.md:95-119`; UX-DR17/18/21/22 — `epics.md:204-215`
- AD-13…AD-18 — `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:78-185`;
  AD-17's three rungs at `:151-166`; the one-transform convention at `:194`; `gui` CLI discipline at `:192`
- M2 retrospective (M2-1, M2-4, M2-7, M2-8, M2-9, M2-10, M2-11, M2-15, M2-16) —
  `_bmad-output/implementation-artifacts/epic-5-retro-2026-08-23.md`
- Vehicle procedure — `_bmad-output/implementation-artifacts/vehicle-session-runbook.md`;
  worked example `7-2-signoff/task-6-vehicle-runbook.md`
- Story rules and anti-overengineering policy — `docs/technical-preferences.md`

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-25 | Story created. Picking approach ruled by Wolf: ray from the rendering camera via `Camera::viewport_to_world`. All four epic premises re-verified against source; `render_to_world` confirmed present but test-only today, and its truncation semantics recorded as D2. |
