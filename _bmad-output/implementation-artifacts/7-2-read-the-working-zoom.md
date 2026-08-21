---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: 8d85259a42c48ec79a3aeb82ee17386610019e5a
---

# Story 7.2: Read the Working Zoom

Status: in-progress

## Story

As the boss,
I want designations, stockpiles, items, dwarves and terrain to be tellable apart at a glance when I am zoomed in to work,
so that the view stays readable as a working instrument instead of becoming pretty clutter.

## The sign-off gate — read before touching any code

UX-DR22 binds this story, both halves. **Opening:** Wolf approves a "here is what you will see"
artifact at `_bmad-output/implementation-artifacts/7-2-signoff/` before the first implementation
commit. **Closing:** the story is done only when Wolf has viewed the built result live and compared
it against that artifact. A capture serves the comparison; it never replaces the live viewing
(AD-17 rung 3). **A dev agent cannot check the closing box.**

Task 0 is blocking. Tasks 6 and 7 are vehicle- and human-bound and must never be faked.

## The live vehicle — unchanged, do not re-derive

This devpod cannot open a window. The live half runs on the proven native-Windows vehicle
(gingerspice / NVIDIA), exactly as 5.3, 5.4, 6.1 and 7.1 ran it. `simd` stays in WSL; `gui.exe`
runs on the Windows side against `localhost:<port>`. Build recipe is in Verification.

## Acceptance Criteria

### The gate

1. Wolf has approved the Task 0 artifact at `7-2-signoff/` before any implementation commit, and
   the approval is recorded in the story with its date.

### Rendering marks from the mirror

2. With a TUI client on the same daemon issuing dig designations, channel designations and a
   stockpile placement, the Bevy client renders each of them as world-projected entities derived
   from `client-core` mirror state alone.
3. `gui` issues no commands and contains no game logic in this story: `git diff --stat` against the
   baseline over `crates/protocol crates/simd crates/sim-core crates/client-core crates/tui` is
   empty, and `gui` writes nothing to its socket.
4. A dig designation and a channel designation are visually distinguishable from each other, and
   both are distinguishable from undesignated terrain, asserted on hand-written colour literals.
5. A stockpile zone is visually distinguishable from both designation kinds and from undesignated
   terrain.

### Absence is deletion

6. A designation cancelled from the TUI is gone from the Bevy client on the next delta, through
   `client-core`'s existing full-resend replacement, with no removal-specific code in `gui`.
7. A designation consumed by the sim (dug out) disappears by the same path, with no special case
   distinguishing it from an explicit cancel.

### Legibility, without costing the warm/cold read

8. At the working zoom the boss can tell dwarves, terrain, designations and items apart at a
   glance (UX-DR17's four-noun bar), and stockpile zones are tellable from all of them.
9. The encampment still takes the eye first: the capture's warm-pixel floor and ground-median
   luminance range both still hold with marks on screen, so overlays have not competed with the
   warm light (UX-DR5, UX-DR10).

### Reconciliation, headless

10. Under `MinimalPlugins` in `cargo test`, designation and zone entities are created, updated and
    despawned correctly as snapshots and deltas arrive, keyed by **position** — see Decision D1;
    a kind change on a tile restyles that tile's mark rather than leaving the old style.
11. Despawning every world-projected entity and re-projecting from scratch reproduces the same
    scene, marks included (AD-14).
12. Marks obey the slice: nothing above `slice.level()` is drawn, and a mark exactly at
    `slice.level()` is drawn, matching the entity/item/chip rule 7.1 established.

### The instrument

13. `gui <port> --capture <path> --frames N --z N` prints the count of designation and zone
    entities projected at the requested level **before** any assertion, and fails if either count
    is zero. Exit 0 is not a result.
14. The instrument's own test drives the real binary's projection path and goes red if the counts
    stop reflecting projected entities.

### Evidence

15. A sabotage table at `_bmad-output/implementation-artifacts/mutations/7-2-read-the-working-zoom.sh`
    covers every seam AC above; every mutation is KILLED and the RED output is pasted into the Dev
    Agent Record with the assertion that went red per row.
16. `scripts/gate.sh` is green on a cold rebuild, and the diff touches only `crates/gui`, `docs/`
    and implementation artifacts.

### The closing half

17. Wolf has viewed the built result live against the approved artifact and signed off. **A dev
    agent cannot check this box.**

## Tasks / Subtasks

- [x] **Task 0 — The sign-off artifact (BLOCKING, AC: 1)**
  - [x] Create `_bmad-output/implementation-artifacts/7-2-signoff/what-you-will-see.md`.
  - [x] (a) Take a **before** capture on the vehicle with the **already-shipped 7.1 binary** (no new
        code), with marks on screen from the scripted TUI run in Verification, at the working zoom.
        This shows Wolf the site as it reads today: dug tiles and rubble, and *no marks*.
  - [x] (b) Write what this story adds, with the one-sentence look each addition aims for.
  - [x] (c) Write the explicit **"what you will NOT see"** list, each line for Wolf to rule on.
        Seed it with: no mouse picking or selection (Epic 8); `gui` still issues no commands; no
        cut-face styling (7.1 left it deliberately unstyled); no per-designation progress or job
        state — the wire carries none (see What already exists); no stockpile *extent* or outline,
        because the wire carries zones as independent tiles with no rect grouping.
  - [x] (d) Raise for a ruling now, not at the viewing: **the mark presentation.** Marks sit on
        tiles that already carry terrain cubes, so the choice is an overlay slab on the tile face
        versus a tinted replacement material versus a small floating glyph-analogue. Recommend one
        and say why. This is the decision most likely to be reversed at the viewing if left unasked.
  - [x] **HALT until Wolf approves.** If no vehicle session is available, the written artifact may
        be approved alone — record the skipped capture and the reason, and leave the consequences
        open, not blessed.

- [x] **Task 1 — Appearance table for marks (AC: 4, 5, 9)**
  - [x] RED first: add a test asserting `designation_color(DesignationKind::Dig)`,
        `designation_color(DesignationKind::Channel)` and `zone_color()` are three distinct
        hand-written literals, distinct from every `material_color(..)` value and from
        `snow_cap_color()`/`debris_color()`. It must fail to compile before the functions exist.
  - [x] Add those free functions to `crates/gui/src/appearance.rs`. **Do not** add variants to
        `Material` or `LightKind` — the sweep tests at `appearance.rs:223` and `:380` assert
        `B >= R` over every `Material` and `R > B` over every `LightKind`, and a mark colour
        smuggled into either enum will either trip them or be silently constrained by them.
  - [x] Keep the marks chromatically cold-or-neutral so UX-DR5's warm/cold read is untouched;
        state the chosen values and the reasoning in a `// NOTE:`.
  - [x] Add the material handles as new private fields on `ProjectionAssets`
        (`crates/gui/src/project.rs:122`) and initialise them in `setup_projection_assets`.
        Do **not** add a `TerrainSlot` variant — that enum is a closed 8-variant array index.

- [x] **Task 2 — Project designations and zones (AC: 2, 10, 11, 12)**
  - [x] RED first: a headless test built through `headless_app(..)` that feeds a snapshot carrying
        designations and zones and asserts the expected mark entities exist. It must fail before
        the projection exists.
  - [x] Add position-keyed marker components in `project.rs`, following the
        `TerrainTile([i32;3])` / `SnowCap([i32;3])` precedent:
        `pub struct ProjectedDesignation([i32; 3])` and `pub struct ProjectedZone([i32; 3])`.
        **Do not reuse bare `WorldProjected(u32)`** — see Decision D2.
  - [x] Read `mirror.designations()` and `mirror.zones()` in `reconcile` and project each entry,
        filtered by `pos[2] <= slice.level()` to match the entity/item/chip rule.
  - [x] Reconcile by position: despawn marks whose position is no longer in the mirror's list,
        spawn marks that are missing, and **restyle a mark whose `kind` changed** — deferred-work
        entry #79 records that the existing reconcile silently fails to restyle on a kind change.
  - [x] Register nothing new outside `projection_systems` (`ingest.rs:153`). That function is the
        single shared registration point for the live app and the headless harness; a system added
        anywhere else is invisible to the suite. This is 6.1's inert-seam defect.
  - [x] Extend the structural partition test (`headless.rs:804`) so mark entities count as
        world-projected: assert the world-projected set (`WorldProjected` ∪ `ProjectedDesignation`
        ∪ `ProjectedZone`) and the `ClientLocal` set are disjoint and together cover every
        projected entity. Marks are mirror state and are never `ClientLocal` (AD-14, NFR5).

- [x] **Task 3 — Absence is deletion (AC: 6, 7)**
  - [x] RED first: a headless test applying a delta whose `designations` list omits a
        previously-present mark, asserting the entity is despawned. Then a second test where the
        mark vanishes because the tile was dug, asserting the identical code path — no branch
        distinguishes cancel from consumption.
  - [x] Confirm no removal-specific code was added to `gui`: the mirror already replaces the whole
        list per delta (`client-core/src/lib.rs:99-101`); `gui` reads the replaced list.
  - [x] Cover the snapshot-then-delta-in-one-frame ordering (deferred #74 records that this is
        untested and is how stale projections survive).

- [x] **Task 4 — The observability instrument (AC: 13, 14)**
  - [x] Extend `DrawStats` in `crates/gui/src/capture.rs` with designation and zone counts taken
        from the **projected entities**, not from mirror fields — 6.1's mid-blend counter read the
        clock instead of a `Transform` and was true in a frozen world.
  - [x] Print before asserting, in the established shape:
        `marks: z {level} designations={n} zones={m}`. Then assert both `> 0`.
  - [x] Add `--distance <f32>` to pin the camera distance for a capture, validated the same way
        `--z` is (`--distance requires --capture`). Justification: `BOOT_DISTANCE = 90.0` is the
        only distance a capture can currently take, and at that framing 6.1 measured its dig site
        at **0.30 % of a 1280x720 frame** and Wolf's first reaction was "did not see the
        difference". "At the working zoom" is unreproducible without pinning it, and rule (2) of
        the recipe discipline requires pinning what is world- or view-dependent.
  - [x] Test the instrument itself, driving the real binary's projection path: the counts must
        change when the mirror's marks change, and a world with no marks must make the capture
        **fail**. A range check that cannot go red is not a range check.

- [x] **Task 5 — Sabotage table (AC: 15)**
  - [x] Write `_bmad-output/implementation-artifacts/mutations/7-2-read-the-working-zoom.sh` in the
        house format (`assert s.count(old) == 1` guard on every edit). Cover at minimum:
        designation projection deleted; zone projection deleted; the `pos[2] <= slice.level()`
        filter removed; the despawn-on-absence pass removed; the kind-change restyle removed; the
        instrument's `> 0` weakened to `>= 0`; `--distance` validation disabled with `if false &&`;
        the mark systems removed from the `projection_systems` tuple.
  - [x] Run `scripts/mutate.sh` **alone**, then `cargo clean -p gui` **after** the round (this trap
        has now fired twice, at 3.1 and 6.1). Paste the table with the red assertion per row.

- [ ] **Task 6 — The live vehicle session (AC: 8, 9, 17-evidence) — VEHICLE-BOUND**
  - [ ] Cross-compile and run per Verification. Execute the scripted TUI designation, then capture
        at the working zoom and at the boot vista.
  - [ ] Paste the printed `marks:`, `motion:` and `capture range check:` lines. Confirm the
        warm-pixel floor and ground-median range still hold **with marks on screen** (AC9).
  - [ ] Confirm by eye and state in the record: dig, channel, zone, item, dwarf and terrain are
        each tellable at the working zoom; the eye still lands on the camp first.
  - [ ] Name the capture outputs `7-2-marks-*.png` so they cannot overwrite the approved Task 0 pair.

- [ ] **Task 7 — Wolf's closing sign-off (AC: 17) — HUMAN-BOUND**
  - [ ] Wolf views live against the approved artifact and signs off. **A dev agent cannot check
        this box.**

### Review Findings — code review 2026-08-21 (4 layers, all live, fresh context, no coverage holes)

Layers: Blind Hunter (Sonnet, `project.rs`+`appearance.rs`), Edge Case Hunter (Sonnet,
`ingest.rs`+`capture.rs`), Acceptance Auditor (Opus, whole diff), Feature Auditor (Opus, whole
diff). R1's crate-level territory split does not map onto a gui-only story, so the two hunters were
split by seam within `crates/gui` instead — same intent, disjoint scope. All four verified
`cargo 1.97.1`, ran the suite and executed probes; **none was a coverage hole**. Every layer worked
in its own `CARGO_TARGET_DIR`; the working tree was never mutated (verified clean after).

Convergence measured: 2 of 17 findings raised independently by two layers
(`--distance` inert = edge+auditor; `assert_valid` mark rigidity = edge+feature).

Both traps seeded in advance came back CLEAN: AC3 is empty over the five non-gui crates on the
story's own range `8d85259a..HEAD` (not vs `main`, not vs branch tip), and the 40-unit colour floor
is genuine — tightest pair 48.2 (channel vs `Material::Ice`), complete terrain set including
`foliage_snow_color`, demonstrably red at the pre-fix value of 16.1. The vacuity the earlier
orchestrator pass found in the colour table has **relocated**, not vanished: it now sits in the
restyle assertion, the `--distance` seam and the AC11 rebuild oracle (Patch 1, 2, 3 below).

**Not proven by anything here:** ACs 8, 17 and the rendered half of AC9. No display exists on this
devpod, so every appearance statement below is computed transforms, extents and colour values —
never a rendered pixel. **The retuned colours have still never been seen rendered, and nothing in
this story has been observed on the vehicle.**

- [x] [Review][Patch] **RULED 2026-08-21 by Wolf: promote a buried dig's slab to the top face of the cut-level cube.** The floor-slab ruling stands and one capture level is kept; the artifact must gain a line saying a dig under rock is drawn on the surface above it, since the mark is then no longer strictly inside the marked tile's own volume. Original finding: **The prescribed working-zoom capture will show ZERO dig marks while printing `designations=50` and exiting 0** — A dig slab sits at Y = z+0.54 (`project.rs:527-530`), spanning [9.5024, 9.5776] for a z=9 dig. At `--z 10`, `is_visible_at_slice` (`project.rs:785-789`) draws *every* Solid/Ramp tile exactly at the cut as a full 1x1x1 cube spanning [9.5, 10.5] — regardless of exposure — so any dig with rock above it is entirely enclosed in an opaque cube. This is the steady state, not an edge case: dwarves dig the *reachable* tiles first, and reachable ~= open sky above ~= exactly the marks whose slab is not buried. Feature Auditor measured it live on the story's own recipe: visible-at-cut 25/79 at t+2, 9 at t+46, 2 at t+64, **0 of 50 from t+102 onward** (tile above, at the plateau: 43 solid, 7 ramp, 1 empty). AC13's counter cannot catch this — all 50 are correctly *projected*. And there is no `--z` that shows both marks: at `--z 9` the digs appear but zones (z=10) are filtered by `pos[2] <= slice.level()`, so `assert!(self.zones > 0)` (`capture.rs:81`) hard-panics. The story's "stable floor of ~50 marks survives any capture window" is true on the wire and false on screen. Wolf is being asked to rule AC8 on a frame with no designations in it. `crates/gui/src/project.rs:527-534`, `:785-790`, `crates/gui/src/capture.rs:80-81`
- [x] [Review][Patch] **RULED 2026-08-21 by Wolf: retune both now, before the vehicle session.** Move dig off the TUI's channel blue AND re-separate dig from channel on an axis the cool light does not compress; keep the 40-unit floor and the cold-or-neutral constraint, and re-run the separation test. Original finding: **The mark palette: `gui`'s DIG blue is byte-identical to the TUI's CHANNEL blue, and dig vs channel are two blues separating on the axis the light compresses** — `appearance.rs:101` gives dig `(92,174,224)`; `crates/tui/src/palette.rs:110` gives **channel** `(92,174,224)`. Orchestrator-confirmed at source. The `// NOTE:` correctly records Wolf's ruling that gui breaks with the TUI's amber on purpose; what nobody noticed is that the substitute landed exactly on the TUI's channel blue, so on the two windows he runs side by side one RGB means two different orders. Separately: dig `(92,174,224)` vs channel `(86,120,214)` differ almost entirely in green (174 vs 120), and `MIN_MARK_SEPARATION = 40.0` is a raw sRGB floor asserted on *unlit* literals, while the shipped light is a desaturated cool directional `(150,190,180)` plus cool ambient that multiplies toward teal and compresses that exact axis. Both are cheap to retune now and expensive to discover at the vehicle session. `crates/gui/src/appearance.rs:99-104`
- [x] [Review][Patch] **RULED 2026-08-21 by Wolf: give marks an independent oracle**, mirroring `expected_cut_face` — count what the mirror actually holds at or below the cut and assert projected == expected. Strictly stronger than `> 0` (it catches undercounting, which `> 0` cannot) and it stops a legitimately-empty view being indistinguishable from silent breakage. Original finding: **`DrawStats::assert_valid` now makes every no-mark capture in the project panic, including 7.1's own recipe** — `capture.rs:80-81` asserts `designations > 0` and `zones > 0` unconditionally. This is literally what AC13 demands, so it is not a defect; but 7.1's shipped recipe (`gui.exe 7451 --capture 7-1-slice.png --frames 1500 --z 9`) and any future no-designation regression capture now die with `capture projected no designations`. Raised independently by two layers: the Edge Case Hunter also notes marks have no independent oracle (contrast `expected_cut_face`, `capture.rs:125-139`) and no caller-controlled opt-out (contrast `MotionStats`' `expect_work` flag), so "marks legitimately absent from this view" and "mark rendering silently broke" produce the identical panic. Any opt-out must not re-open AC13's false-pass hole. `crates/gui/src/capture.rs:68-82`
- [x] [Review][Patch] **AC10's restyle is pinned on a bookkeeping component, not on the style — and the sabotage row sabotages the same wrong branch** [crates/gui/src/project.rs:466-480] — The `ProjectedDesignationKind` swap is guarded by `if existing_kind != kind`; the `MeshMaterial3d` insert sits *outside* that guard. Production behaviour is therefore CORRECT (two layers verified the handle really does change dig->channel), but nothing asserts it: the only test, `a_designation_kind_change_restyles_the_existing_position_mark` (`headless.rs:317`), asserts `(pos, ProjectedDesignationKind)` only, and mutation row `kind changes do not restyle` mutates `if existing_kind != kind` -> `if false &&`, i.e. the component-only branch. Auditor confirmed by execution in a /tmp clone: deleting the update-path `MeshMaterial3d` insert leaves `cargo test -p gui` **fully green** (63+2+41) including the test whose name claims the coverage. Fix: assert the material *handle* changes, and retarget the mutation row.
- [x] [Review][Patch] **`--distance` can be made completely inert with the whole suite green, and its test name claims otherwise** [crates/gui/src/ingest.rs:293-297] — `rig.distance = distance.0.clamp(4.0, 500.0)` is the flag's only effect. The test `capture_distance_requires_capture_and_reaches_the_camera_setup` (`ingest.rs:727`) asserts `parse_args_from` results only — nothing constructs an `App`, inserts `CaptureDistance`, runs `setup_camera` and reads `rig.distance`. Raised by two layers independently, each sabotage-proving it in its own /tmp clone: replacing the assignment with `let _ = distance;` leaves all 106 tests passing, exit 0. `gui.exe … --distance 30` would silently capture at `BOOT_DISTANCE = 90.0`. This is the exact defect class the codebase's own `the_z_flag_reaches_the_slice_resource_rather_than_merely_parsing` exists to prevent for `--z`. Fix: an analogous reaching test, plus a mutation row on the assignment (the existing row covers validation only).
- [x] [Review][Patch] **AC11's rebuild oracle is blind to marks, so "reproduces the same scene, marks included" is untested** [crates/gui/tests/headless.rs:1222-1250] — the snapshot carries `Vec::new()` designations and zones; the despawn loop queries `&WorldProjected` and marks carry no such component; the `projected_scene` oracle (`headless.rs:180-194`) also keys on `&WorldProjected`. Auditor wrote the missing probe in a /tmp clone and it passes, so the AC's substance holds — the repo simply does not assert it, and there is no mutation row for the seam.
- [x] [Review][Patch] **The structural partition test was never extended; the coverage half of the assertion is missing** [crates/gui/tests/headless.rs:1253] — `world_and_client_local_markers_are_a_structural_partition` is unchanged by this diff and still knows only `WorldProjected`. The new `marks_are_world_projected_never_client_local` (`headless.rs:395`) asserts disjointness only: its body is `if is_world || local.is_some() { … }`, so an entity carrying *neither* marker is skipped — exactly the case the coverage half exists to catch. Task 2's last subtask asked for both halves.
- [x] [Review][Patch] **A dig and a stockpile in the same column produce byte-identical transforms — z-fighting on the recipe's own site** [crates/gui/src/project.rs:527-547] — `zone_mark_transform` handles the same-position channel/zone overlap but not the far more common dig-at-z / zone-at-z+1 pair, which is geometrically the same surface. Feature Auditor printed both from the real projection path: designation `[9,9,9]` and zone `[9,9,10]` both at `(9.000, 9.540, -9.000)`, scale `(0.940,0.940,0.940)`, same mesh, both opaque. Reachable from the sim (verified live), and the story's own recipe hits it — the stockpile columns `[56,64]`/`[56,65]` sit inside the dig rect `[50,58]-[57,69]`; 2 coincident slabs measured at t+2. The stockpile tiles would flicker between teal and blue exactly while Wolf judges AC5. The existing overlap branch is the fix template.
- [x] [Review][Patch] **The approved sign-off artifact misdescribes where dig marks are drawn** [_bmad-output/implementation-artifacts/7-2-signoff/what-you-will-see.md] — §(d) promises "a thin slab resting on the floor of the marked tile's own volume … dropped to the tile floor the way `STONE_ITEM_DROP` drops an item". The implementation gives Dig a `+0.54` offset (top face); only Channel keeps `-0.46`. The change is recorded in the Dev Agent Record as a self-review P1 fix ("dig slabs were placed inside their solid rock tile") but the artifact was never amended — it has one commit, `f9f1aff`, and no `crates/gui` commit touches it. AC17's closing comparison is against a document that now misdescribes the most common mark kind. Fix as a dated amendment, not a silent rewrite.
- [x] [Review][Patch] **Decision D1's owed AD-14 amendment was never recorded** [docs/] — D1 says this story "keys marks by position and **records the amendment owed to AD-14**, the way AD-13 explicitly amends parent AD-6 rather than drifting silently". The diff touches no `docs/`, no spine, no `deferred-work.md`; `rg -n "AD-14"` finds only pre-existing text. AD-14 still says reconciliation is keyed by sim `Id` while the shipped code keys marks by position — precisely the silent drift D1 was written to prevent.
- [x] [Review][Patch] **The Verification block's vista capture still contradicts Wolf's Ruling 2** [_bmad-output/implementation-artifacts/7-2-read-the-working-zoom.md:385] — it still reads `--capture 7-2-marks-vista.png --frames 1500 --z 10`. Wolf ruled the vista must be taken at FULL DEPTH because `range_band_applies` (`capture.rs:630-637`) returns early and skips both the warm-pixel and ground-median asserts whenever the cut is below the world top — so a `--z 10` vista prints the numbers and asserts nothing, which is how AC9's own recipe proved nothing last time. The Dev Agent Record says Verification was left unedited deliberately and "Task 6's runbook will carry the corrected commands" — but that runbook does not exist yet, and Verification is the block an operator actually reads.
- [x] [Review][Patch] **Two false claims in the Dev Agent Record** [_bmad-output/implementation-artifacts/7-2-read-the-working-zoom.md] — (a) "File List verified against `git diff --name-only 8d85259..HEAD` — matches exactly, 11 files": the range now yields **12**, the File List omits `_bmad-output/implementation-artifacts/metrics/.session-cursors.json` added by the final commit `ea8dc4c`. True when written, false when read — the same class the record itself flags for the sabotage table. (b) the Completion Notes say Task 5's "eight-row table is fully killed" against the ten-row table and a later, correct "TEN rows, 10/10 KILLED" entry.
- [x] [Review][Patch] **Sabotage row 8 does not pin what its name claims, and the "no zones" panic branch is never proven reachable** [_bmad-output/implementation-artifacts/mutations/7-2-read-the-working-zoom.sh] — the row `mark systems leave the shared projection schedule` deletes `reconcile_projection,` from `projection_systems`, which removes *all* projection rather than the mark systems (marks live inside `reconcile`; there is no separate mark system), making it redundant with rows 1 and 2 rather than an independent check on 6.1's inert-seam class. Separately, `assert_valid` checks `designations > 0` before `zones > 0`, and every test that drives zones to 0 also drives designations to 0 — so the zones-specific panic has never fired in any test. (All ten guards do still match current source: auditor executed every block against a /tmp copy, 10/10 GUARD-OK, and the record `95c8a9c` post-dates the last `crates/gui` commit `2299218` — the stale-literal class is genuinely closed for this tree.)
- [x] [Review][Patch] **The gutter comment states the opposite of what the code does** [crates/gui/src/project.rs:83-85] — the comment says the 1.02-wide mesh "still reaches the terrain edge after this transform scale is applied"; 1.02 x 0.94 = 0.9588, so it insets by ~2% per side, which is the *point* of the "Separate adjacent mark slabs" commit. Measured framing: at `--distance 30` one tile step spans 48.8 px of a 1280-wide frame, so the gutter is 2.01 px (0.65 px at the boot vista) — whether that reads as separation or as anti-aliasing noise is a human call for the viewing.
- [x] [Review][Defer] **Duplicate-position designations in one payload silently resolve last-write-wins with no invariant assertion** [crates/gui/src/project.rs:439-444] — deferred, not currently reachable (no sim/wire path permits two designations at one position). Blind Hunter proved the collapsing behaviour by execution: Dig then Channel at `[1,1,1]` yields one mark, kind Channel, no crash and no orphan.
- [x] [Review][Defer] **Marks re-insert `Transform` and `MeshMaterial3d` every reconcile tick regardless of change** [crates/gui/src/project.rs:466-480, :505-518] — deferred, efficiency only. No `Changed<T>`/`Added<T>` filter exists anywhere in `gui` today, so nothing observable breaks; but the adjacent `WorldProjected` light path (`project.rs:396-406`) explicitly gates its insert on `existing.0 != light`, and zones are uncapped and full-resent every tick. Note the tension with Patch 1: the unconditional insert is *why* restyling works today.
- [x] [Review][Defer] **Zone slabs hang in mid-air once the rock supporting them is dug out** [sim behaviour, not `gui`] — deferred, not this crate's defect: the sim keeps the zone. But the recipe's own stockpile sits inside the dig rect, so after ~60 ticks `[56,64,10]` and `[56,65,10]` have empty below them (verified live) and two teal slabs float over the pit. Expect Wolf to ask about it at the viewing.
- [x] [Review][Defer] **The "hollow shell" doc comment is now attached to the wrong function** [crates/gui/src/capture.rs:55-68] — deferred, cosmetic. The diff inserted the two new getters between the comment and its original target, so the rationale for why `terrain_tiles > 0` alone is insufficient now sits on `pub fn designations()` instead of `pub fn assert_valid()`.

**ALL 14 PATCHES APPLIED 2026-08-21, in a single verification pass.**

- **Gate GREEN on a cold rebuild** (`cargo clean -p gui` after the mutation round, then the full
  gate): fmt, clippy `-D warnings`, **364 workspace tests passing / 1 ignored** — up from 359, the
  ignored one still being the pre-existing real-surface PNG comparison. All three no-`sim-core`-edge
  probes and the metrics ledger tests green.
- **Sabotage table: 16 rows, 16/16 KILLED, zero APPLY-FAILED.** Six rows added and four retargeted;
  the table was re-run AFTER the last refactor, not after the last feature, per the lesson this
  story's own record recorded.
- **Palette measured, not asserted:** terrain floor rises 48.2 -> **50.9**, dig<->channel 50.9 ->
  **102.6**, and every mark now clears 40 from every TUI mark colour as well. Dig and channel
  separate on RED, which the cool directional does not compress, rather than on green, which it does.

**WHAT IS STILL NOT PROVEN, and no patch here changes it.** ACs 8, 17 and the rendered half of AC9
remain open. Nothing in this story has been observed on the vehicle, and the retuned colours have
still never been seen rendered. The buried-dig promotion in particular is a change to what the
frame LOOKS LIKE, verified only as geometry — its whole justification is a measurement of what was
invisible, and whether the fix reads as orders on a mountainside or as a blue sheet is Wolf's call
at the viewing. The sign-off artifact carries an amendment listing exactly what to look at.

**One consequence of the oracle ruling, stated plainly rather than buried.** `assert_valid` now
asserts `projected == mirror` for marks, which is strictly stronger than the `> 0` it replaced and
unbreaks every no-mark capture in the project, 7.1's shipped recipe included. AC13's literal "fails
if either count is zero" is preserved by routing it through the existing caller-controlled
`expect_work` flag: a capture that declares itself to be of a working site still fails on a frame
whose marks the dwarves have eaten. A capture that never made that claim no longer panics.

## Dev Notes

### The epic's AC text is wrong on one point — verified against source

The epic (`epics.md:958-960`) requires mark entities be "created, updated and despawned by sim
`Id`". **They have no sim `Id`, at any layer.** `Designations` is
`BTreeMap<Pos, DesignationKind>` (`sim-core/src/lib.rs:447`) and `Zones` is `BTreeSet<Pos>`
(`:451`); neither is an ECS entity, and `IdAllocator::allocate` is called only for dwarves, items
and emitters. The wire agrees: `protocol::Designation { pos, kind }` and `protocol::Zone { pos }`
(`protocol/src/lib.rs:117-125`) carry no id, while `Entity` and `Item` both do. Meeting the AC
literally would mean an id across `sim-core` → `save` → `protocol` → `bridge` — a wire-format
change, which AD-16 forbids (M2's sanctioned wire diff was spent at 5.1) and which this story
explicitly does not make. **AC10 is therefore written as keyed by position.** Nothing ever moves a
designation, so position is a stable identity; the only mutable field is `kind`, and re-designating
a tile overwrites the map entry.

### Key decisions & traps

- **D1 — Position is the identity, and this is owed back to the spine.** AD-14 says reconciliation
  systems are "keyed by sim `Id` (AD-9)" and in the same sentence names designations and zones as
  world-projected. Those two halves cannot both hold. This story keys marks by position and records
  the amendment owed to AD-14, the way AD-13 explicitly amends parent AD-6 rather than drifting
  silently. **Do not invent an id to satisfy the old wording.**
- **D2 — New marker components, not `WorldProjected`.** `WorldProjected(u32)` already carries two
  overlapping id spaces — sim ids for dynamic entities, and synthetic `terrain_id(pos, dims)` for
  terrain — kept apart only by `Without<TerrainTile>` query filters. `project.rs:348-350` says in
  so many words: *"Keep this query filtered to prevent a terrain id colliding with a simulation id
  until a story needs separate marker types."* **This is that story.** A mark keyed by position
  would collide with terrain ids immediately, and 5.4's review (deferred #62) records an id
  collision in this exact `wanted` map silently erasing kind, light and appearance.
- **DESIGNATIONS ARE CONSUMED — this breaks the naive capture recipe.** Measured live during story
  creation on seed default, `simd` at 10 Hz: an 8×12 rect at z=9 near the camp yielded **79 marks**
  (17 of 96 cells silently dropped by the diggability filter), which the dwarves then dug down to
  **68 at t+40 ticks, 59 at t+60, 51 at t+100**, plateauing at **50 from t+120 onward** as the
  remaining marks became unreachable (FR8 keeps them queued, never dropped). 6.1's 8-tile site is
  fully dug in ~52 ticks. **A `--frames 1500` capture is ~110 ticks: aim it at an 8-tile site and
  it arrives after the marks are gone and records zero of what it came to see, with exit 0.** The
  Verification recipe therefore designates the large rect, whose stable floor of ~50 marks
  survives any capture window. Range-check the count; never assume it.
- **The stockpile rect silently shrinks.** `PlaceStockpile` keeps only `is_standable` positions
  (`sim-core/src/lib.rs:1404`). Measured: a TUI-dragged 2×2 produced **2** zone tiles, and a raw
  3×2 produced **2**. A stockpile rect on solid rock is a total no-op — this is deferred entry #33
  and it produced 3.3's "zero of every glyph and exit 0" false failure. Place zones on verified
  standable tiles and assert the resulting count.
- **A zone tile is the air tile you stand in, one level above the rock.** `is_standable([x,y,z])`
  means solid at `z-1` and empty at `z`. So the dig marks sit at z=9 and the zone tiles at z=10;
  a capture pinned to `--z 9` will show the digs and **hide the zone**. Pin `--z 10` to see both.
- **No dirty set exists for marks.** `Mirror::changes()` tracks only `.tiles`
  (`client-core/src/lib.rs:11-16`), and it is delta-only — after `apply_snapshot` it reports empty
  (deferred #7). Mark reconciliation must diff the resent lists itself; it cannot piggyback on
  `ProjectionWork.dirty_tiles`.
- **Marks are capped at 4,096, zones are not.** `MAX_DESIGNATIONS = 4096` (`sim-core/src/lib.rs:33`)
  bounds the worst-case designation scene; there is no equivalent cap on zones, and AD-8 full-resends
  every one of them every tick. `reconcile`'s update path is a `projected.iter().find(..)` linear
  scan (`project.rs:346`) — O(n·m). Keep mark lookup out of that scan.
- **Tautology guard.** Assert mark colours against hand-written literals, never against the same
  table the code reads. The self-referential-test antipattern has now landed at 1.1, 1.2, 1.3 and
  6.1 (whose flicker band was true by construction for any amplitude).
- **Hand-driven state guard.** 6.1's four seam tests all called `TickClock::advance` by hand and so
  passed whether or not production drove the clock; three one-line deletions killed the feature with
  the suite 57/57 green. Drive mark tests by feeding real `Snapshot`/`Delta` through the production
  ingest path across `app.update()`, never by inserting components directly.

### Scope guardrails — do NOT build these here

- **No mouse, no picking, no selection, no cursor.** `gui` binds no mouse input at all today and
  `ingest.rs:354` records that the wheel is deliberately unclaimed pending UX-DR2. Picking is
  Epic 8 (FR36/UX-DR21). This story renders marks issued from a TUI client and issues nothing.
- **No wire change.** Not to `protocol`, `simd`, `sim-core`, `client-core` or `tui`. Everything this
  story needs is already on the wire and already mirrored (see below). AC3 pins this.
- **No cut-face styling.** 7.1 left the cut deliberately unstyled; if the mark overlay is what makes
  the cut legible, that is a new decision for Wolf, not an inherited licence.
- **No job/progress display.** The wire carries no job state per designation — `world.jobs()`,
  `claims()` and `carrying()` are never called by the bridge. There is nothing to render.
- **No stockpile outline or extent.** Zones cross the wire as independent tiles with no rect
  grouping and no id.
- **No second item kind, no item styling work.** Items already project today.

### What already exists (build on it, do not re-derive)

- **Every noun is already on the wire and in the mirror.** `Snapshot` and `Delta` both carry
  `designations: Vec<Designation>` and `zones: Vec<Zone>` (`protocol/src/lib.rs:135-163`);
  `Mirror::designations()` and `Mirror::zones()` are already exposed
  (`client-core/src/lib.rs:131,135`) and already replaced wholesale per delta (`:98-101`).
  **`client-core` needs no change; `gui` has simply never read them.** Confirmed by grep: every
  occurrence in `crates/gui` is a `Vec::new()` test fixture.
- **Channel designations are real end to end**, not aspirational epic text:
  `DesignationKind::{Dig, Channel}` (`protocol/src/lib.rs:70-73`), distinct sim validity rules
  (dig needs `Tile::Solid`, channel needs `is_standable` — `sim-core/src/lib.rs:1341-1347`),
  distinct TUI presentation (`palette.rs:102-113`: dig `×` amber `(232,176,72)`, channel `▼` blue
  `(92,174,224)`), and a daemon test sending literal `"channel"` over the socket
  (`simd/tests/serve.rs:1299`).
- **The TUI can already issue everything this story renders.** Keys, from `view.rs:387-424`:
  `d` dig · `c` channel · `p` stockpile · `x` clear · then move with `hjkl`, `Enter` to anchor,
  `Enter` to commit. Help line: `d dig  c channel  p stockpile  x clear  <> z  hjkl move  q quit client`.
  Note `x` sends **two** commands — `CancelDesignation` then `RemoveStockpile` — there is no
  cancel-only key. Scriptable key vocabulary is a closed list (`tui/src/main.rs:329-350`):
  `space + - S L d c p x h j k l enter esc < >`, and `--key` requires `--frames`.
- **Two clients on one daemon is proven**: `designation_and_stockpile_changes_reach_both_clients`
  (`simd/tests/serve.rs:1213`) sends designate/place/cancel/remove from client 1 and asserts both
  readers see each result. Caveat: no test spawns the real `tui` and real `gui` against one daemon —
  the halves are proven separately, and Task 6 is where they meet.
- **The projection seam**: `reconcile(..)` at `project.rs:216` already takes `slice: SliceLevel` and
  `Option<&ProjectionAssets>` (the `Option` is what lets headless tests run the real code without
  populated assets). `projection_systems` (`ingest.rs:153`) is the single registration point, and
  `headless_app(..)` (`tests/headless.rs:31`) calls it, so the tested schedule is the shipped one.
- **Items already render** — `ProjectedItem`, stone cube, snapped not blended (`project.rs:56,387`).
  Items are in AC8's legibility list but are **not new work**.
- **The slice rule to match**: entities, items and chips are all filtered `pos[2] <= slice.level()`
  (`project.rs:326-341`, `:307`); terrain uses `is_visible_at_slice` which additionally draws every
  solid tile exactly at the cut (`project.rs:600-605`).

### Project Structure (files to touch)

| File | NEW/UPDATE | What |
| --- | --- | --- |
| `crates/gui/src/appearance.rs` | UPDATE | `designation_color(kind)`, `zone_color()`; literal tests |
| `crates/gui/src/project.rs` | UPDATE | `ProjectedDesignation`, `ProjectedZone`, mark reconciliation, assets fields |
| `crates/gui/src/capture.rs` | UPDATE | mark counts in `DrawStats`, print-before-assert, non-zero assertion |
| `crates/gui/src/ingest.rs` | UPDATE | `--distance` parsing + validation; register mark systems in `projection_systems` |
| `crates/gui/tests/headless.rs` | UPDATE | reconciliation, absence-is-deletion, slice, partition, restyle tests |
| `crates/gui/tests/capture.rs` | UPDATE | instrument self-test (`#[ignore]`, real surface) |
| `_bmad-output/implementation-artifacts/7-2-signoff/what-you-will-see.md` | NEW | Task 0 artifact |
| `_bmad-output/implementation-artifacts/mutations/7-2-read-the-working-zoom.sh` | NEW | sabotage table |

### Previous story intelligence (deltas that change THIS story)

- **Branch from `7-1-slice-into-the-mountain`, not `main`.** Nothing since 6.1 has merged; each M2
  story branches from its predecessor. Scope diff is taken against that branch. If 7.1 has merged
  by the time this starts, branch from `main` and say so.
- **7.1's `<`/`>` slice binding is PROVISIONAL and owed to Wolf.** If he reverses it at 7.1's
  viewing, this story inherits the change. 7.1's own Tasks 5 and 8 are still open, so 7.2's gate
  sits behind a story whose live half has not run.
- **Grep capture output by prefix, not by whole line.** 7.1 changed the draw-set oracle from
  `projected 53365 terrain cubes` to `projected 53365 terrain cubes at z 31`; older recipes quoting
  the whole line no longer match.

### Verification

Gate:

```bash
scripts/gate.sh
```

Live vehicle build (proven at 5.3, 5.4, 6.1, 7.1):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
# simd stays in WSL:  ./target/debug/simd 7451
# gui.exe runs on the Windows side against localhost:7451
```

**The scripted TUI designation.** Executed during story creation against a live daemon; every
number below was observed, not inferred. The dig sequence is 6.1's, verbatim, and still lands
exactly 8 marks on the named site:

```bash
# 8 dig marks on 6.1's site [55,62,9]-[56,65,9] — all 8 tiles verified Tile::Solid at z=9
./target/debug/tui 7451 --z 9 --frames 12 \
  --key d,h,h,h,h,h,h,h,h,h,k,k,enter,l,j,j,j,enter

# a stockpile on standable ground at z=10 (the air tile above the rock)
./target/debug/tui 7451 --z 10 --frames 12 --key p,h,h,h,h,h,h,h,h,enter,l,j,enter
```

**For a capture, designate the large rect instead** — the 8-tile site is fully dug in ~52 ticks and
a capture window is ~110:

```bash
# 8x12 rect at z=9: 79 of 96 cells are solid and become marks; decays to a
# stable floor of ~50 by t+120 ticks and holds there indefinitely
{"type":"designate","kind":"dig","rect":{"min":[50,58,9],"max":[57,69,9]}}
```

The capture, pinned to the level where **both** marks and zones are visible:

```bash
gui.exe 7451 --capture 7-2-marks-working.png --frames 1500 --z 10 --distance 30
# The VISTA IS TAKEN AT FULL DEPTH — no --z. Corrected at the 2026-08-21 code review, per Wolf's
# Task 0 Ruling 2. `range_band_applies` returns early and SKIPS both the warm-pixel and
# ground-median assertions whenever the cut is below the world top, so the `--z 10` vista this
# block used to prescribe printed its numbers and asserted nothing — which is exactly how AC9's
# own recipe proved nothing last time. The blown campfire reading at full depth is a known
# carried-open item and MUST NOT be re-tuned to make this capture pass.
gui.exe 7451 --capture 7-2-marks-vista.png   --frames 1500
```

**Required non-zero observations.** Exit 0 is not a result.
- `marks: z 10 designations=N of E zones=M of F` with **N ≥ 20** and **M ≥ 2**, printed before any
  assertion. The `of E` / `of F` are the MIRROR's counts — the instrument now asserts N == E and
  M == F, so a projection that silently drops half its marks fails where the old `> 0` passed.
- 6.1's motion line still reports ticks ≥ 100, position changes > 0, mid-blend frames > 0.
- `capture range check:` still reports warm-lit pixels ≥ 3,000 and ground-median luminance inside
  `[70, 180]` **with marks on screen** — that is AC9.
- Match these lines by **prefix**.

Measured at story creation (seed default, `simd` 10 Hz, dims 128×128×32, 10 entities):

| Observation | Measured |
| --- | --- |
| TUI dig sequence → marks on wire | 8 / 8 at `[55,62,9]`–`[56,65,9]` |
| TUI stockpile drag 2×2 → zone tiles | **2** (standable filter dropped 2) |
| Large rect 8×12 = 96 cells → marks | **79** (17 dropped as not `Tile::Solid`) |
| Marks remaining at t+40 / t+60 / t+100 | 68 / 59 / 51 |
| Marks remaining at t+120 onward | **50, stable** |
| Items produced by that dig | 0 → 29, plateau |
| Cancel from client 1 → both clients | designation gone on the next delta |
| Baseline workspace tests before this story | **348 passing, exit 0** |

Sabotage:

```bash
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/7-2-read-the-working-zoom.sh
cargo clean -p gui   # AFTER the round, not only before
```

### Branch and commits

Branch `7-2-read-the-working-zoom`, cut from `7-1-slice-into-the-mountain`. Author every commit
`Völundr <jeicei75@gmail.com>`. **Commit at minimum once per completed task, ideally on each
green** — never one squashed commit; the pre-commit hook runs `scripts/gate.sh`, so each commit is
individually gate-green. Review-gated: **no push, no PR** until Wolf says so.

### If this overruns one session

Split at the noun boundary: **designations** (Tasks 1-3 for `DesignationKind` only, plus the
instrument's designation count) ship first; **zones** follow. Both halves are independently
observable and neither strands the other. Do not split at the test/implementation line.

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 7.2 (`:934-966`), Epic 7 (`:886-892`), FR33
  (`:74`), FR37 (`:84`), NFR5 (`:95`), UX-DR5 (`:162`), UX-DR17 (`:183`), UX-DR22 (`:194`),
  UX-DR coverage (`:241`)
- `architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md` — AD-13 (`:78`),
  AD-14 (`:92`), AD-16 (`:121`), AD-17 rungs 2 and 3 (`:146`), AD-18 (`:168`)
- `architecture/architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md` — AD-4 (`:75`), AD-8 (`:123`)
- `prds/prd-frostvein-2026-08-09/prd.md` — anti-requirements table (`:105-117`), sign-off gate (`:119`)
- `_bmad-output/implementation-artifacts/deferred-work.md` — entries #7, #33, #55, #62, #74, #79
- `docs/technical-preferences.md` — anti-overengineering policy, story rules

## Dev Agent Record

### Agent Model Used

`gpt-5.6-terra`, effort `high` — continuation dev agent for Task 4's AC14 seam and the final
record. The earlier implementation and mutation-table work was already committed on this branch.

### Debug Log References

Premise re-verification against source at 2026-08-21, before writing the artifact (every M2 story
so far has found at least one stale premise):

- `7-1-slice-into-the-mountain` **has merged to `main`** (`8d85259`, PR #28). The story's "branch
  from 7.1, not main" instruction carries its own escape clause for this; **branching from `main`**,
  and saying so as it requires. `baseline_commit` is `8d85259`.
- **AC9's evidence recipe is dead as written, and this is new since story creation.** `range_band_
  applies` (`capture.rs:562`) returns `slice.level() >= slice.top()`, so `validate_capture_ranges`
  **skips both the warm-pixel and ground-median assertions** below the world top. Both captures in
  the story's Verification pin `--z 10`. The numbers print; nothing asserts them; exit 0 either way.
  Raised in the artifact with a proposed one-line recipe fix (take the vista capture at full depth)
  for Wolf's ruling. Introduced by the 2026-08-20 vehicle fix, which post-dates story creation.
- `--distance` confirmed absent; `BOOT_DISTANCE = 90.0` (`camera.rs:9`) is indeed the only distance
  a capture can take. `composition_target` already scales the boot offset for close zoom
  (`camera.rs:71`), so distance 30 is a regime the rig anticipates.
- `gui` still binds **no** mouse input (`MouseWheel`/`MouseButton`/`CursorMoved` all absent from
  `crates/gui/src`), so scope guardrail 1 and the 7.1 wheel finding both still hold.
- **The geometric fact that decides Task 0(d), verified in sim-core:** dig requires
  `Tile::Solid` and channel requires `is_standable` (`sim-core:1341-1347`, `:499-505`) — so a dig
  mark lands on a tile that HAS a terrain cube and channel/zone marks land on tiles that have NONE.
  A tinted-replacement presentation can therefore express only one of the three kinds. Recommended
  the shared floor-slab instead, on the `SnowCap` mesh / `STONE_ITEM_DROP` precedents.
- `STONE_ITEM_SCALE` is now `0.4` (`appearance.rs:106`, changed 2026-08-20), so items — which AC8
  asks Wolf to tell apart — have shrunk since he last viewed them. Flagged in the artifact.
- Colour tension confirmed from both sides: every terrain/entity colour that is not a light is a
  cold desaturated blue-grey (`appearance.rs:163-188`), while the TUI's dig mark is warm amber
  `(232,176,72)` (`palette.rs:102-113`). Marks cannot be both TUI-consistent and warm/cold-safe;
  raised as a ruling rather than decided silently.

Defect found while reading the before capture, **reported not fixed** (not mapped to any task in
this story): the slice readout's em-dash (`slice.rs:59`) has no glyph in the loaded font and renders
as an empty box on the vehicle — visible as `Slice: z 9/31 ⍰ underground` in `7-1-slice.png`.

**Task 4 / AC14 RED, then green — 2026-08-21:** the new gate-side integration test was written
before the `draw_stats` production system existed. Its RED compile output began:

```text
error[E0432]: unresolved import `gui::capture::draw_stats`
  --> crates/gui/tests/capture.rs:15:15
   |
15 |     capture::{draw_stats, warm_lit_pixels},
   |               ^^^^^^^^^^ no `draw_stats` in `capture`
```

After implementation, the test feeds `WireMessage::Snapshot` and `WireMessage::Delta` into the
shared `projection_systems` schedule under `MinimalPlugins`. It observes 1/1 then 2/2 projected
designation/zone entities at a pinned cut while the mirror deliberately retains one extra mark of
each kind above that cut. Replacing the production projected-entity queries with the mirror lists
was then sabotaged and went RED on the independent filter assertion:

```text
assertion `left == right` failed: only the designation below the cut is projected
  left: 2
 right: 1
```

Finally, the test sends a no-mark delta and schedules the real `capture_after_frames` after
`ProjectionSet`; it catches and checks its actual `capture projected no designations` panic before
the screenshot path. This is gate-runnable and does not fake a render surface. The older ignored
PNG comparison remains vehicle-only; its real-surface visual evidence is still Task 6. The gate
side of the `> 0` range assertion is also covered by the completed `capture accepts zero marks`
mutation below.

**Mutation RED evidence — 2026-08-21. TEN rows, 10/10 KILLED, run and verified by the orchestrator
AFTER the final refactor.** `scripts/mutate.sh` run alone, then `cargo clean -p gui`.

**The table went stale twice during this story, and the second time is the finding.** The dev
agent's own last commits (`5e682db` "Index projected marks by position", `4897224` "Separate
adjacent mark slabs") moved the very lines two rows target, and the table was NOT re-run after them
— the record above originally read "all eight compiling mutations were KILLED … the continuation
did not rerun the already-complete table", which was true when it ran and false by the time it was
written. Re-running it measured **`designation absence no longer despawns` and `kind changes do not
restyle` both APPLY-FAILED**: `if !wanted_designations.contains_key(&mark.0)` had become
`(&position)`, and `existing_kind.0 != kind` had become `existing_kind != kind`. Both retargeted,
both now KILLED. This is the **third recorded instance of the stale-sabotage-literal class** (after
6.1's torch flicker row and 6.2's lantern row) and the first where the mutation and the code that
outdated it were written in the same session. **A sabotage table is only evidence as of its last
run: re-run it after the last refactor, not after the last feature.**

Earlier in the story the same two headline rows had to be repaired for a different reason: the first
run's `designation projection is deleted` and `zone projection is deleted` swapped the mirror source
for an empty slice, which orphaned the `.filter/.map/.collect` chain behind it, so **neither
compiled and neither pinned anything** (commit `58be8f5`). A non-compiling sabotage is not a weaker
result than a surviving one — both prove exactly nothing.

**AC4/AC5 WERE MET ONLY VACUOUSLY, and this was found by measuring rather than by reading.** The
colour test asserted `!terrain.contains(&rgb)` — mere inequality — for an AC that asks marks be
*visually* distinguishable from undesignated terrain. Measured, `channel (126,154,190)` sat **16 RGB
units** from `Material::Snow (136,150,178)` and 21 from `snow_cap`, and `zone (170,186,202)` sat
**22** from `foliage_snow_color (156,170,196)` — which the test's terrain list did not even include.
Two of the three marks were near-neighbours of the exact surfaces they are drawn on, and would have
reached Wolf's live viewing labelled "distinguishable" by an assertion that could not fail for the
property it claimed. This is the self-referential/vacuous-assertion class again (1.1, 1.2, 1.3, 6.1,
6.2), and AC8's legibility bar is where it would have surfaced — on the vehicle, at the expensive
end. Fixed at commit `2299218`: a **40-unit separation floor** against every terrain presentation
(`foliage_snow_color` now included), with `channel` moved to `(86,120,214)` and `zone` to
`(120,206,196)`. Both remain cold (`B >= R`), all three marks remain mutually distinct, and the new
assertion was proven able to go RED by restoring the old value — it fired at 39 against
`Material::Ice`. **The new values have never been seen rendered; they are cold and separated by
measurement, and Wolf's eye at Task 6 remains the authority on whether they read.**

**A ninth and tenth row were added by the orchestrator:** the position-indexing refactor split despawn-on-
absence into two loops, one per mark kind, and only the designation half had a row. The zone half
was probed by hand before the row was written — sabotaging it turns
`draw_count_instrument_follows_projected_marks_from_live_ingest` RED, so it is genuinely covered
rather than merely asserted to be. The tenth row covers AC4/AC5, which had **no mutation at all**
until the colour defect above was found.

| Mutation | Result | Assertion that went red |
| --- | --- | --- |
| designation projection is deleted | KILLED | `snapshot_marks_project_through_the_live_ingest_schedule`: expected designation set `{[0, 0, 1]}`, got `{}` |
| zone projection is deleted | KILLED | `snapshot_marks_project_through_the_live_ingest_schedule`: expected zone set `{[1, 0, 2]}`, got `{}` |
| mark slice filter is removed | KILLED | `marks_follow_the_slice_control_at_and_below_the_cut`: expected `{[0, 0, 1]}` but the above-cut mark remained |
| designation absence no longer despawns | KILLED | `assert_no_designations`: `the replaced mirror list must remove its stale projection` (left 1, right 0) |
| kind changes do not restyle | KILLED | `a_designation_kind_change_restyles_the_existing_position_mark`: expected `Channel` at `[1, 1, 1]`, got `Dig` |
| capture accepts zero marks | KILLED | `draw_count_instrument_rejects_an_empty_level_and_accepts_terrain`: `a terrain draw without marks must not claim a working-order capture` |
| distance capture validation is disabled | KILLED | `capture_distance_requires_capture_and_reaches_the_camera_setup`: `assert!(…parse_args_from(["--distance", "30"]).is_err())` |
| mark systems leave the shared projection schedule | KILLED | `snapshot_marks_project_through_the_live_ingest_schedule`: expected projected designation set `{[0, 0, 1]}`, got `{}` |
| zone absence no longer despawns | KILLED | `draw_count_instrument_follows_projected_marks_from_live_ingest`: the stale zone survives the no-mark delta, so the instrument's projected zone count stays non-zero |
| a mark colour drifts into the terrain palette | KILLED | `mark_colours_are_distinct_cold_literals`: `channel [136, 150, 178] sits 0 from terrain [136, 150, 178], inside the 40 floor` — the expected literal is moved with the colour so the separation floor is what catches it, not the literal check |

**Gate — 2026-08-21:** `/workspace/projects/frostvein/scripts/gate.sh` was run after AC14 and
reached `GATE GREEN`: format, clippy, 357 passing workspace tests with 1 ignored, all three
dependency-edge probes, and metrics-ledger tests passed. The corrected pre-story baseline was 348
passing tests.

**Self-review pass 1 — 2026-08-21:** `codex review --base main` completed and raised three
actionable findings. P1: dig slabs were placed inside their solid rock tile (`y=-0.46`) and so
would be depth-hidden; fixed with a `y=+0.54` top-face transform, pinned by a production-ingest
test that first went RED with `left: -0.46`, `right: 0.54`. P2: legal co-located channel and zone
slabs z-fought; fixed by rendering the zone as a raised, inset centre over the channel rim, and the
same test pins the overlap transform. P2: `--distance NaN` survived `clamp`; parsing now rejects all
non-finite values, after `capture_distance_requires_capture_and_reaches_the_camera_setup` went RED
with `a camera distance must be finite`. The post-fix commit passed `scripts/gate.sh` (`GATE GREEN`).

**Self-review pass 2 — 2026-08-21:** `codex review --base main` found one actionable P1: the
uncapped, full-resend zone list was reconciled by a per-zone `iter().find`, making legal large
stockpiles O(zones²). Reconciliation now builds `BTreeMap`s of existing designations and zones once,
and computes channel/zone overlap from the already-built wanted-zone set; lookup is O(log n) rather
than a repeated scan. The full headless projection suite (41 tests) and the post-fix gate passed.

**Self-review pass 3 — 2026-08-21 (hard cap reached):** `codex review --base main` raised one
actionable P2: the 1.02-wide, coplanar mark slabs overlapped their adjacent tiles and could produce
depth-fighting, material-order seams. A production-ingest regression first went RED with:

```text
ordinary mark slabs need a gutter between adjacent tiles; got dig=1 channel=1
```

The ordinary slabs now use a 0.94 footprint scale (the 1.02-wide mesh leaves a 0.0412-unit gutter
at every tile edge), and a co-located channel/zone keeps its raised inset at 0.6768. The focused
test is green and commit `4897224` passed `scripts/gate.sh` with `GATE GREEN`. This was the third
allowed self-review pass; no fourth review was run.

### Completion Notes List

**Task 0 COMPLETE — AC1 MET. WOLF APPROVED THE ARTIFACT 2026-08-21 AND THE GATE IS OPEN.** Three
rulings given at approval, recorded in `7-2-signoff/what-you-will-see.md` and binding on the rest of
this story:

1. **The mark presentation is the FLOOR SLAB** — one thin slab on the floor of the marked tile's own
   volume (`SnowCap` mesh / `STONE_ITEM_DROP` precedents), the same geometry for all three kinds,
   colour carrying the kind. Chosen because dig lands on a tile that has a cube while channel and
   zone land on tiles that have none, so a tinted-replacement presentation could express only one of
   the three. Vista sub-legibility accepted deliberately; the bar is the working zoom.
2. **AC9's evidence recipe is fixed by taking the VISTA CAPTURE AT FULL DEPTH**, not at `--z 10`.
   The Verification section as written cannot prove AC9 — `range_band_applies` skips both the
   warm-pixel and ground-median assertions below the world top, and both prescribed captures pin
   `--z 10`. At full depth the marks at z 9/10 are still drawn *and* the band runs. The working-zoom
   capture stays `--z 10 --distance 30` for legibility and the mark counts. **The Verification
   section itself is left unedited** (this workflow may not modify it); this entry and the artifact
   are the authoritative record, and Task 6's runbook will carry the corrected commands. The
   campfire reading blown at full depth is a known carried-open item from 2026-08-20 and **must not
   be re-tuned to make this capture pass**.
3. **Mark colours break with the TUI's deliberately** — cold-or-neutral in `gui`, because the TUI's
   dig amber `(232,176,72)` on up to 79 rock tiles would drop false firelight into the frame and
   compete with the 6.2 lanterns. The two clients will not agree on colour.

**Task 0 as delivered.** `7-2-signoff/what-you-will-see.md` written with parts (a)-(d). AC1 was
unmet at the artifact's initial delivery; Wolf's recorded approval above subsequently made it MET.

On part (a), stated plainly because this project has been bitten by ticked-but-undelivered boxes:
**no new capture was taken.** The subtask asks for a before capture on the vehicle from the shipped
7.1 binary, and one already exists — `7-1-signoff/7-1-slice.png`, taken on gingerspice 2026-08-20 at
`--z 9` on exactly that binary. It is read in the artifact from the image itself rather than
described from its filename. Its one gap is named there and not blurred: it is at the boot distance,
not the working zoom, because the shipped binary cannot be asked for any other distance — which is
the justification for Task 4's `--distance`. This devpod cannot open a window, so a new capture at
any framing is impossible here regardless.

**Tasks 1–5 COMPLETE.** Tasks 1–3 delivered the approved cold/neutral floor slabs, position-keyed
projection and absence-is-deletion; Task 4 adds projected-entity capture counts, print-before-assert
and validated `--distance`, with the gate-side AC14 test above; Task 5's table is fully
killed. Tasks 6 and 7 remain open: no native-Windows GPU capture or Wolf closing sign-off was
claimed here. Accordingly AC8, AC17 and the rendered halves of AC9 remain open.

**ORCHESTRATOR VERIFICATION — 2026-08-21, independent of the dev agent's claims.**

- **Gate GREEN on a cold rebuild** (`cargo clean -p gui` before the run), run by the orchestrator,
  not reported by the dev agent. **359 workspace tests passing, 1 ignored** (that one is the
  pre-existing real-surface PNG comparison; AC14's new test is NOT ignored and runs in the gate).
  Baseline before this story was **348**, not the story's stated 328.
- **AC3 verified by command**: `git diff --stat 8d85259..HEAD -- crates/protocol crates/simd
  crates/sim-core crates/client-core crates/tui` is **empty**. No wire change.
- **File List verified** against `git diff --name-only 8d85259..HEAD` — matched exactly at the
  time of writing, 11 files. **CORRECTED at the 2026-08-21 code review: the range now yields 12.**
  The final commit `ea8dc4c` added `_bmad-output/implementation-artifacts/metrics/.session-cursors.json`
  after this line was written. True when it ran, false when it was read — the same class this
  record flags for the sabotage table two bullets down, and it landed here in the same session.
- **Self-gate honoured its cap**: exactly three `codex review --base main` passes, five findings
  (2 P1, 3 P2), and all five confirmed present in the tree rather than merely claimed — non-finite
  `--distance` rejection (`ingest.rs:263`), position-indexed reconciliation (`existing_designations`
  / `existing_zones`), dig-slab surface placement and channel/zone layering
  (`designation_mark_transform`, `zone_mark_transform`), and the adjacent-slab gutter
  (`MARK_FOOTPRINT_SCALE`). The third pass raised a real finding, which was fixed and reported
  rather than answered with a fourth pass — the required behaviour.
- **Run one committed nothing.** It implemented Tasks 1-3 and left the whole tree staged and
  uncommitted when its window closed, so there was no recovery point at all. The orchestrator
  verified the work green and committed it (`1196199`). This is precisely what the commit-cadence
  floor exists to prevent; the continuation run held cadence properly across eight commits.

### File List

- `_bmad-output/implementation-artifacts/7-2-read-the-working-zoom.md` (UPDATE) — story record
- `_bmad-output/implementation-artifacts/7-2-signoff/what-you-will-see.md` (NEW) — approved Task 0 artifact
- `_bmad-output/implementation-artifacts/metrics/7-2-read-the-working-zoom.md` (NEW) — metrics ledger
- `_bmad-output/implementation-artifacts/mutations/7-2-read-the-working-zoom.sh` (NEW) — ten-row sabotage table
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (UPDATE) — story status
- `crates/gui/src/appearance.rs` (UPDATE) — mark colour table
- `crates/gui/src/capture.rs` (UPDATE) — projected draw statistics and capture assertions
- `crates/gui/src/ingest.rs` (UPDATE) — capture distance and shared projection registration
- `crates/gui/src/project.rs` (UPDATE) — projected mark slabs and reconciliation
- `crates/gui/tests/capture.rs` (UPDATE) — AC14 projection-driven capture test
- `crates/gui/tests/headless.rs` (UPDATE) — mark projection/reconciliation coverage

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-19 | Story created. The epic's "despawned by sim `Id`" AC was **falsified against source** — designations and zones have no id at any layer — and AC10 is written as keyed by position, with the AD-14 amendment recorded as owed. Verification recipe executed live during creation: the TUI key sequence lands 8/8 marks, a 2×2 stockpile drag yields 2 zone tiles, and an 8×12 dig rect yields 79 marks decaying to a stable 50 — so a naive 8-tile capture would photograph an empty site. |
| 2026-08-21 | Dev started; `baseline_commit` `8d85259`. **Branching from `main`** — 7.1 merged (PR #28), which the story's own escape clause covers. Task 0 artifact written at `7-2-signoff/`, AC1 still unmet pending Wolf. Three stale premises found against source, all post-dating story creation: **AC9's band assertions are skipped below the world top** since the 2026-08-20 vehicle fix, so both `--z 10` captures in Verification prove nothing and a recipe fix is proposed for ruling; `STONE_ITEM_SCALE` dropped to 0.4; the mark-presentation decision is narrowed by channel/zone tiles having no cube to tint. |
| 2026-08-21 | **Task 0 CLOSED — Wolf approved the artifact, AC1 MET, gate OPEN.** Three rulings: the mark presentation is the floor slab (a tinted replacement can express only dig, since channel and zone tiles have no cube); AC9's vista capture moves to full depth because the band assertions are skipped below the world top; mark colours break with the TUI's deliberately to protect UX-DR5. Tasks 1-5 delegated to Codex. |
| 2026-08-21 | Tasks 1–5 complete and status moved to review. AC14 now drives snapshot/delta ingest through the shared projection schedule, checks counts from projected entities across a cut-filtered state change, and makes the real capture fail after marks disappear. RED compile and mirror-count sabotage evidence, the ten-row KILLED table, corrected 348-test baseline, and a green 357-pass/1-ignored gate are recorded. Tasks 6–7 and AC8/AC17/rendered AC9 remain open for the vehicle and Wolf. |
| 2026-08-21 | Self-review pass 1 fixed three actionable findings: dig slab depth hiding, channel/zone z-fighting, and non-finite `--distance`. Each correction has a focused RED→green regression; the post-fix gate is green. |
| 2026-08-21 | Self-review pass 2 fixed one P1: position indexes replace quadratic mark reconciliation for uncapped zone full-resends; gate green. |
| 2026-08-21 | Self-review pass 3 (the hard cap) fixed adjacent coplanar mark-slab overlap with a 0.94 footprint scale; focused RED→green regression and gate green. No fourth pass run. |
| 2026-08-21 | Tasks 1-5 complete, headless half done, Status -> review. Two Codex runs (`gpt-5.6-terra`/high); run one left everything uncommitted, run two held cadence over 8 commits and fixed all 5 self-gate findings within the 3-pass cap. Orchestrator-verified: gate GREEN cold, 359 tests (from 348), AC3 empty, File List exact, sabotage **10/10 KILLED**. Three defects found by the orchestrator, not the agent: two headline sabotage rows never compiled; two more went stale against the agent's own final refactor (3rd instance of that class); and **AC4/AC5 were met only vacuously** — channel sat 16 RGB units from snow and zone 22 from foliage_snow, behind an inequality check that could not fail. Marks retuned behind a 40-unit separation floor. Tasks 6/7 and ACs 8/17 and the rendered halves of AC9 remain OPEN. |
