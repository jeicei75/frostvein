---
baseline_commit: cb45817
model: claude-opus-5[1m]  # Opus default; the 1M-context variant, recorded so the ledger row is readable
---

# Story 10.5b: Dwarves Worth Looking At — Part B, the art-iteration loop

Status: dev-done-except-AC8 (UX-DR22 needs Wolf at the vehicle)

**Part A took the epic's named split line and shipped the seam. This is the other half.**
Part A (`10-5-dwarves-worth-looking-at`, 14 commits, `cb45817`) is **implemented and pushed but
NOT merged and NOT reviewed** — this story's baseline is that branch tip, not `main`. Every
`file:line` below is read on `cb45817`; on `main` most of them do not exist yet.

**Part A over-delivered, and that changes what B is.** Its own Task 2 specified a *pine* as the
stand-in "obviously not a dwarf". The dev session instead shipped **the real authored dwarf**
(`assets/gltf/SM_VoxelDwarf_Miner01.glb`, 14,398 tris, byte-reproducible from
`src-assets/blender/dwarf_miner.py`). So B is **not** "the authored dwarf itself" as the epic and
Part A both describe it. That work is done. What is left is everything around it: the loop that
lets Wolf *change* the dwarf, the checker that can *read* him, and the sign-off.

## Story

As the boss,
I want to change my dwarf and see the change in the running client without a rebuild,
so that authoring a creature is a loop I can turn, not a cross-compile I have to wait for.

## Premises verified against source — 2026-09-07, on `cb45817`

**Executed, not read.** Three of the seven corrected what the record said.

1. **THE AUTHORED DWARF IS ALREADY SHIPPED AND EMBEDDED.** `DWARF_ASSET` (`ingest.rs:266-269`)
   `include_bytes!`s `assets/gltf/SM_VoxelDwarf_Miner01.glb` and `register_tree_assets`
   (`ingest.rs:275-281`) registers it alongside the four pines. Part A's premise *"Part B cannot
   start until Wolf has a model"* and the epic's *"the authored dwarf itself"* are both **SPENT**.

2. **`check_asset.py` ACCEPTS the dwarf — and SILENTLY UNDER-READS HIS PALETTE BY THREE COLOURS.**
   This is the concrete generalisation B owes, and it is not the rejection everyone expected.
   Run on `cb45817`:

   ```
   FIGURES assets/gltf/SM_VoxelDwarf_Miner01.glb size_m=1.2x1.2x0.8 min_y_m=0.000000
   centre_x_m=0.000000 centre_z_m=0.000000
   palette=#E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50 tris=14398 verts=28796
   ```

   Seven colours. **The dwarf has ten** (`dwarf_miner.py:50-59`). `palette_from_glb` loops
   `for index in range(len(PALETTE_HEX))` (`check_asset.py:164`) and `PALETTE_HEX`
   (`check_asset.py:41-43`) is **the pines' seven-entry list**. Decoding all 16 cells of the
   dwarf's own atlas directly:

   | cell | 0 | 1 | 2 | 3 | 4 | 5 | 6 | **7** | **8** | **9** | 10-15 |
   |---|---|---|---|---|---|---|---|---|---|---|---|
   | colour | E9D2BB | 5E4632 | FFFFFF | 5F7A6A | 474B41 | A9B2AC | 8B6B50 | **6B5B49** | **34271C** | **F0A63C** | 000000 |
   | read? | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **✗** | **✗** | **✗** | ✗ |

   The 4×4 grid holds 16; only the loop bound is wrong. The three unread cells are
   `HEX_WOOD_TRUNK`, `HEX_HAIR`, and **`HEX_FLAME` `#F0A63C`** — the one colour
   `dwarf_miner.py:116` says "is a COLOUR and never an emitter. A pixel guard asserts that."
   The artifact-side check of that colour does not happen.

   **This is the [[delta-cannot-say-none-left]] shape.** The `palette=` figure is a *signoff
   comparison* by design (`check_asset.py:271-273`), so a human compares it by eye — and it
   reports seven with nothing to say three more are there. It cannot answer "and nothing else".

3. **`notify` CROSS-COMPILES AND LINKS TO WINDOWS — settled here, with a binary.** Part A left
   this as its one unclaimed risk.

   **UPGRADED at dev time, and the upgrade matters:** the first probe was a *scratch crate*, which
   proves the crate builds but not that THIS workspace does with `file_watcher` actually on.
   `cargo build -p gui --target x86_64-pc-windows-gnu` on `d3bd25b` → **exit 0, a real
   `gui.exe`**, and `notify` / `ReadDirectoryChangesW` symbols are present in it, so the watcher is
   linked rather than feature-gated away. **NOTE: the debug artifact is 1.8 GB** — a vehicle copy
   wants `--release`.

   The original scratch-crate probe, kept because it is what settled the risk before any code:
   `notify-debouncer-full 0.7.0` (pulling `notify 8.2.0`, `notify-types 2.1.0`),
   `cargo build --target x86_64-pc-windows-gnu` → **exit 0, a 13,127,158-byte `.exe`**. The
   constructor is called in `main`, so it is linked rather than compiled away. Target and
   `x86_64-w64-mingw32-gcc` are both installed on this devpod. **No C-toolchain work in this story.**

4. **`bevy/file_watcher` reaches `bevy_asset` cleanly.** `bevy/file_watcher` →
   `bevy_internal/file_watcher` → `bevy_asset?/file_watcher` → `["notify-debouncer-full", "watch",
   "multi_threaded"]` (`bevy_asset-0.19.0/Cargo.toml:40-44`). `bevy_asset` is already in the graph
   via `3d_bevy_render`, so the optional `?` is satisfied. One word in `Cargo.toml:19-30`.

5. **`--assets <dir>` NEEDS NOTHING STAMPED INTO THE BINARY, and an ABSOLUTE path is the whole
   mechanism.** `FileAssetReader::new` does `root_path = get_base_path().join(path)`
   (`bevy_asset-0.19.0/src/io/file/mod.rs:44`), and `Path::join` with an absolute argument
   **replaces** the base. So `AssetPlugin { file_path: "<absolute dir>" }` resolves to exactly that
   dir regardless of what `get_base_path()` returns (`BEVY_ASSET_ROOT` → `CARGO_MANIFEST_DIR` →
   exe dir, `:19-29`). **This is why option A is not the deleted `resolve_asset_root`**: the path
   is an argument at runtime, never a compile-time stamp (`crates/gui/build.rs:20-25`).

6. **`source=embedded` IS A HARDCODED WORD IN BOTH INSTRUMENT LINES.** `project.rs:2372` and
   `:2386` print the literal string `source=embedded`. It will keep printing `embedded` with
   `--assets` pointed anywhere. **This is exactly [[literal-guard-misses-the-maths]]** — text that
   does not move when the mechanism does. Task 1's condition 2 has a home, and it is a correction,
   not an addition.

7. **`embedded://` IS HARDCODED AT BOTH LOAD SITES.** `project.rs:316` (trees) and `:324` (dwarf)
   `format!("embedded://{path}#Scene0")`. These two `format!`s are the seam `--assets` switches.
   **The `map_or_else` → `Handle::default()` fallback around them is load-bearing** (`:313-326`,
   Part A's AC9): every `MinimalPlugins` test runs with no `AssetServer` and must not panic.

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` (the **full** tier) is green, and the diff is confined to this story's own
   commit range from `baseline_commit` — **not `main..HEAD`**, which spans Part A's 14 commits too.

### The loop

2. **`--assets <dir>` makes the client read the dwarf from disk instead of from the binary, and
   the difference is OBSERVED, not reported.** Point `--assets` at a directory whose
   `gltf/SM_VoxelDwarf_Miner01.glb` is a **different asset** (a pine `.glb`, copied under the
   dwarf's name). With the real daemon and `--static-world`, a `--headless --capture` frame differs
   from the same run without `--assets`, inside a window containing at least one wire dwarf, by at
   least **10x the same-build noise floor measured in that same window**. Reuse
   `10-5-signoff/window_diff.py` and publish the window's coordinates and both figures.
   **A printed path is not evidence** — AC6 covers the path; this AC covers the bytes.
3. **With `--assets` absent the binary behaves exactly as it does today** (Task 1 condition 1): the
   embedded blobs are used, and a test asserts the resolved source is `embedded` on the no-flag path.
   The flag is never a silent default.
4. **`file_watcher` is enabled with a justification line against the trim's recorded reasons**
   (`Cargo.toml:19-20` shows the existing comment style), and the workspace still resolves exactly
   one Bevy version.
5. **Enabling `file_watcher` does not start a watcher when there is nothing to watch.** With
   `--assets` absent, no filesystem watch is armed. `AssetPlugin.watch_for_changes_override`
   (`bevy_asset-0.19.0/src/lib.rs:238-244`) defaults watching **ON** once the feature is compiled
   in, so this is an explicit `Some(false)`, not an assumption — and a test pins it, because the
   cost otherwise lands on every headless test in the gate.

### The instrument

6. **Both startup lines report the RESOLVED asset source, and the test asserts the line CHANGES
   when the source does.** `project.rs:2372` and `:2386` currently print the literal
   `source=embedded`; they must print the source actually resolved, beside the existing
   `gui build <sha>` stamp (`ingest.rs:289`). The test drives the real binary and compares the
   line with and without `--assets` — a test that only checks the line is well-formed does not
   satisfy this.

### The checker

7. **`check_asset.py` reads every palette cell the asset actually carries, and the dwarf's
   `FIGURES` line reports ten colours including `#F0A63C`.** The four pines still report their
   existing seven-colour literals unchanged (`test_check_asset.py:51-77`), and a test covers an
   asset whose cell count differs from the pines' — the fixture must NOT be a seven-colour asset,
   or the row cannot discriminate ([[sabotage-blind-when-fixture-matches-constant]]).

### The sign-off

8. **UX-DR22, both halves, on the dwarf.** *Opening:* Wolf has approved a "here is what you will
   see" artifact of our actual world at this framing before implementation — name the artifact and
   its approval date in this file. *Closing:* Wolf has viewed the built result **live on the
   vehicle** and compared it against that artifact. Per AD-17, `gui --capture` output serves the
   closing half and never replaces the opening half.

### The measurement — RULED 2026-09-07

**What it is, in Wolf's words: performance written to a log so it can be read AFTER a live run,
instead of squinting at an on-screen counter during one.** Scope ruled the same day: frametime plus
content counters, **no CPU/GPU split** (that needs `RenderDiagnosticsPlugin` and GPU timestamp
queries, which `llvmpipe` here cannot exercise — vehicle-only and untestable in the devpod, so it
is not bought until something asks for it).

9. **`--perf-log <path>` writes one CSV row per frame, and every row carries the CONTENT beside the
    TIME.** Columns as shipped: `frame, t_ms, frametime_ms, terrain, trees, dwarves, dirty_tiles,
    mark`. **AMENDED from the authored `draws, tris, remesh_chunks`, and each change is a
    correction rather than a substitution:** `draws`/`tris` need `RenderDiagnosticsPlugin` and GPU
    timestamps that `llvmpipe` cannot exercise here, so they would be vehicle-only and untestable —
    the entity counts move when the drawn world moves, which is the property the column exists for.
    `remesh_chunks` became `dirty_tiles` because `reconcile` returns `()` and giving it a chunk
    count would change a signature with many callers for a refinement nothing has asked for; tiles
    answer the question the split needs. `mark` is new — see AC11. A timing column with
    no content column beside it cannot be told apart from a broken instrument — **~140 fps once
    survived a 39% triangle cut here because the terrain was never rasterised**
    ([[fps-that-does-not-move]]). The header names the run: build stamp, asset source, `--subdiv`,
    dwarf/tree counts, and **whether vsync was on**, because a capped run measures the monitor.
10. **The summary splits STEADY-STATE frames from EDIT frames, and the summariser is tested here
    against a synthetic CSV with hand-computed percentiles.** Report `p50/p95/p99/p99.9`
    frametime in ms, the hitch count (frames over 2× median, and an absolute >50 ms bucket), and
    the two populations **separately** — `remesh_chunks == 0` against `remesh_chunks > 0`.
    Averaged together the edit cost disappears, which is exactly how 10.6 measured a still scene
    and missed that one dug tile re-meshes the world ([[static-measurement-misses-edit-cost]]).
    **If "1% low" appears anywhere it names its definition in the header** — the two conventions in
    the wild (worst-1%-averaged vs the 99th-percentile frame) are not the same number.
    The summariser is a pure function over the CSV, so it gets a RED **on this devpod** before any
    real data is trusted to it; the numbers themselves come from the vehicle.
11. **A key at the seat marks the moment, and the summary reports it** (Wolf's ruling — the flag
    serves a scripted run, the key catches "that felt bad just now", which a scripted run never
    reproduces). **AMENDED from "dumps the last N frames": with every row written as it happens
    there is nothing left to dump** — the rows are already on disk. What a ten-minute log actually
    lacks is a LANDMARK, so `F4` flags the next frame and the summary prints every marked frame with
    its time. Off unless asked, like `--capture` and `--static-world`.

### Scope

12. `_bmad-output/implementation-artifacts/mutations/10-5b-the-art-iteration-loop.sh` carries at
    least **three rows the mutation run kills**, one of them AC7's palette-loop-bound row.

## Tasks / Subtasks

- [x] **Task 1 — `--assets <dir>` (AC2, AC3, AC5).**
  - [ ] Parse `--assets <dir>` in `parse_args_from` (`ingest.rs:676`), beside `--static-world`.
        Absolute path; reject a relative one with a message rather than resolving it against an
        unstated base.
  - [ ] `DefaultPlugins.set(AssetPlugin { file_path: <dir>, watch_for_changes_override: Some(true),
        ..default() })` on the flag path; `watch_for_changes_override: Some(false)` otherwise (AC5).
        Both arms of `ingest.rs:309-327`, headless and windowed.
  - [ ] Replace the two hardcoded `embedded://` prefixes (`project.rs:316`, `:324`) with the
        resolved prefix. **Keep the `map_or_else` → `Handle::default()` fallback exactly**
        (premise 7) or every `MinimalPlugins` test panics.
  - [ ] Skip `register_tree_assets` (`ingest.rs:275`) on the disk path, or leave it and let the
        prefix decide — either is fine; say which in a `// NOTE:` and why.
- [x] **Task 2 — the resolved source on both instrument lines (AC6).** `project.rs:2372`, `:2386`.
      Test drives the real binary with and without the flag and asserts the two lines DIFFER.
- [x] **Task 3 — `file_watcher` (AC4).** One feature word in `Cargo.toml:19-30` plus its
      justification comment. `notify` cross-compiles — premise 3, already settled.
- [x] **Task 4 — the pwsh launcher (closes issue #46, M2-7).** Fetch the checkout, copy the fresh
      `gui.exe`, start it with `--assets`. **It must verify the checkout's SHA against the
      `gui build <sha>` stamp and refuse to launch on a mismatch, and refuse on a `-dirty` stamp**
      where no exact comparison exists. A launcher that only copies is a convenience; one that
      checks is the guard. It cannot run here — state its RED and the observation Wolf must produce.
- [x] **Task 5 — generalise the palette reader (AC7).** `check_asset.py:164`'s loop bound.
      Decide what "every cell the asset carries" means from the ARTIFACT (a trailing run of
      `#000000` is the terminator the dwarf's atlas already uses) rather than from a per-family
      constant — a second hardcoded list is the abstraction this project's YAGNI rule forbids.
- [ ] **Task 6 — UX-DR22 (AC8) + issue #74 — OPEN, WOLF'S SEAT.** Wolf's time on the vehicle.
      `authored_bench.py` renders authored assets in situ and is the opening artifact's machinery —
      extend it, do not start over.
  - [ ] **Issue #74 joins this sitting (Wolf, 2026-09-07):** dwarves path THROUGH the campfire, and
        the authored mesh's shadow swings wildly. Same dwarf, same seat, so one judgement covers
        both rather than two vehicle sessions.
  - [ ] **Walk `scripts/launch-gui.ps1`'s RED before trusting its check** — it has never been
        executed. The refusals to observe: a binary built at a different commit than the checkout
        (expect `MISMATCH` and exit 1), and a `-dirty` stamp (expect the dirty refusal and exit 1).
        A check that has never been seen to refuse is a habit, not a guard.
- [x] **Task 7 — the performance log (AC9, AC10, AC11).** RULED 2026-09-07; scope in the ACs.
  - [ ] `--perf-log <path>` in `parse_args_from` (`ingest.rs:676`), and the on-demand key beside
        `toggle_overlay` (`ingest.rs:1299`) / `toggle_pause`.
  - [ ] **`FrameTimeDiagnosticsPlugin` is currently added on the WINDOWED path only**
        (`ingest.rs:323`) — the headless arm (`:309-320`) does not have it. Wire whichever arm the
        flag is used from, and say in a `// NOTE:` that headless timings on this devpod are
        `llvmpipe` and are **not** a performance statement.
  - [ ] `remesh_chunks` is the discriminating column and it is the one that does not exist yet —
        source it from the same place `dirty_tiles` drives the rebuild (`ingest.rs:1328-1390`).
  - [ ] The summariser is a **separate stdlib-only script** under `scripts/bench/` (Blender uses
        the uv python, so numpy is invisible — [[gfx-bench-venue]]), tested from
        `scripts/tests/` like `check_asset.py` is.
- [x] **Task 8 — mutations (AC12), then the full gate (AC1).**

## Dev Notes

### Scope guardrails — do NOT

- **Do not re-author the dwarf.** He is shipped, byte-reproducible, and `check_asset.py` accepts
  him. A modification gets a clean session that starts FROM `src-assets/blender/dwarf_miner.py`,
  never from the reference — and the round-2 brief's `## Start from scratch` opening is **wrong for
  anything after v1** and would throw the asset away.
- **Do not make `--assets` a default, and do not ship an `assets/` copy beside `gui.exe`.** The
  directory is a **Windows checkout of this repo** (Wolf, 2026-09-05). Two candidate asset trees is
  the stale-artifact shape this project keeps being bitten by.
- **Do not add a second hardcoded palette list** for the dwarf family (Task 5).
- **Do not re-tune any look constant.** `gui --headless --capture` already exits 101 on the
  near-white ceiling on `main` (`pixel_guard.rs:173-177`), so a red capture exit is not evidence
  about this story. That guard is also **flaky** — issue #72.
- **Do not fix the partition hole.** Issue #73, and a fix aimed only at scene children closes a
  third of it at `k=1`, looks complete, and leaves two thirds open at the adopted `--subdiv 4`.

### What already exists — build on it, do not rebuild it

- `10-5-signoff/window_diff.py` — windowed pixel diff, **noise floor exactly zero** against a
  paused world with the light flicker off. `--static-world` (`ingest.rs`, `capture.rs`) is what
  makes that floor zero; AC2 is unmeasurable without it.
- `10-5-signoff/AC2-measurement.md` — the recipe that works, and the three confounders (wandering
  dwarves, wall-clock light flicker that pausing the sim does NOT stop, an over-large window).
- `scripts/bench/authored_bench.py` — renders authored assets in situ in our valley, cell-to-metre
  scaling reasoned out. AC8's opening artifact.
- `scripts/mutate.sh` — now has the cargo NOT-RUN guard and an `ignored` row argument (Part A).

### Key decisions and traps

- **A printed path is not proof of the bytes read.** AC6 and AC2 are deliberately two ACs. The
  `source=embedded` literal (premise 6) is this project's own worst instrument shape and it is in
  the very line B has to change.
- **`Path::join` replaces on an absolute argument** — premise 5. A *relative* `--assets` silently
  resolves against `get_base_path()`, which on the Windows vehicle is the exe's directory. Reject
  relative paths rather than resolving them.
- **Enabling a watcher feature turns watching on globally** (AC5) — the inert-mechanism shape in
  reverse: not a capability nothing exercises, but a cost everything pays.
- **Part A is unmerged, so `main..HEAD` is the wrong range for AC1** — [[stacked-branch-ac-defect]].
  Only this story's own commit range answers it.

### Project structure — files to touch

| Path | NEW/UPDATE | Why |
|---|---|---|
| `Cargo.toml` | UPDATE | `file_watcher` feature + justification (`:19-30`) |
| `crates/gui/src/ingest.rs` | UPDATE | `--assets` parse (`:676`), `AssetPlugin` config (`:309-327`), `register_tree_assets` (`:275`) |
| `crates/gui/src/project.rs` | UPDATE | prefix at `:316`/`:324`, `source=` at `:2372`/`:2386` |
| `scripts/bench/check_asset.py` | UPDATE | palette loop bound (`:164`) |
| `scripts/tests/test_check_asset.py` | UPDATE | ten-colour fixture; pines' seven unchanged |
| `crates/gui/tests/headless.rs` | UPDATE | `--assets` seam, watcher-off, instrument-line tests |
| `scripts/launch-gui.ps1` | NEW | Task 4, closes issue #46 |
| `scripts/bench/perf_summary.py` | NEW | Task 7's summariser — stdlib only, percentiles + the steady/edit split |
| `scripts/tests/test_perf_summary.py` | NEW | AC10's RED: synthetic CSV, hand-computed percentiles |
| `_bmad-output/implementation-artifacts/mutations/10-5b-the-art-iteration-loop.sh` | NEW | AC9 |

### References

- `epics.md:1556-1593` story 10.5 · `:218` UX-DR22 · `_bmad-output/implementation-artifacts/10-5-dwarves-worth-looking-at.md:120-160` Task 1's ruling and its three conditions
- `bevy_asset-0.19.0/src/lib.rs:233-260` `AssetPlugin` · `src/io/file/mod.rs:19-29,44` base path and the join · `Cargo.toml:40-44` the feature chain
- `docs/tech-art-guidelines.md:452-458` the V1-only scope trap · `src-assets/blender/dwarf_miner.py:50-59,116` the ten colours and the flame rule
- Issues **#46** (the launcher), **#72** (flaky guard), **#73** (partition), **#74** (dwarf paths through the campfire), **#75** (lighting)

## Verification

**Executed at story creation, 2026-09-07, on `cb45817`.** Two of this story's claims were proved
here rather than inherited; the third recipe cannot run until the feature exists and states its
obligation instead.

### Executed — the palette under-read (AC7)

`check_asset.py` on the shipped dwarf reports **7 colours for a 10-colour asset**; all 16 atlas
cells decoded directly show cells 7/8/9 carrying `#6B5B49`, `#34271C`, `#F0A63C` and never read.
Full table in premise 2. **This is the story's own falsification evidence** — the checker does not
reject the dwarf, it under-reports him.

**The deliberate RED for AC7 — OBSERVED, not described.** `check_asset.py:164`'s bound set to
`range(3)`:

```
dwarf   palette=#E9D2BB,#5E4632,#FFFFFF                       (3 of 7 reported)
tests   FAILED (failures=1)  Ran 48 tests
        test_the_four_published_pines_report_their_literal_figures
        ... palette=#4A3B2E,#6B5B49,#2A3E34 ...  not found in the expected seven-colour literal
```

**Restored** with `git checkout -- scripts/bench/check_asset.py` plus a `__pycache__` sweep
([[stale-pyc-survives-mutation-restore]]); control returns seven colours and 48 tests pass. The
file was committed and clean before the mutation ([[sabotage-restore-trap]]).

**What the RED proves, and it is the half that matters:** the pines' literals **do** discriminate a
bound change, so AC7 cannot be closed by quietly widening the loop — but note only **one** of the
48 tests moved, and it is a pine test. **Nothing in the suite fails if the dwarf's three unread
cells stay unread**, which is why AC7 requires a fixture whose cell count differs from the pines'.

### Executed — the Windows cross-compile (premise 3)

```bash
# off-repo scratch crate; notify-debouncer-full 0.7.0, constructor called in main so it links
cargo build --target x86_64-pc-windows-gnu
# -> exit 0, target/x86_64-pc-windows-gnu/debug/notify-probe.exe, 13,127,158 bytes
```

Part A's one unclaimed risk is closed. **Restore:** none, off-repo scratch.

### Inherited obligation — AC2, which cannot run yet

`--assets` does not exist at authoring time. The dev agent must produce, on their own build:

```bash
./target/debug/simd 7491 &
# control: no flag
./target/debug/gui 7491 --headless --static-world --subdiv 1 --frames 160 \
  --capture <control>.png
# and again to <control-b>.png for the same-build noise floor in the SAME window
# then: a dir whose gltf/SM_VoxelDwarf_Miner01.glb is a PINE .glb under the dwarf's name
./target/debug/gui 7491 --headless --static-world --subdiv 1 --frames 160 \
  --assets /abs/path/to/tree --capture <disk>.png
python3 _bmad-output/implementation-artifacts/10-5-signoff/window_diff.py ...
```

**The required non-zero observation:** the windowed floor (worst of two same-build runs) and the
change, with the change at least 10x the floor, both published with the window's coordinates.
`--static-world` is not optional — Part A measured the change SMALLER than the noise without it.

## Branch and commits

Branch `10-5b-the-art-iteration-loop`, off **`10-5-dwarves-worth-looking-at` at `cb45817`**, not
off `main`. Author `Völundr <jeicei75@gmail.com>`. Small commits, imperative messages. Push and PR
only on Wolf's explicit yes. **Part A is unreviewed** — Wolf's ruling 2026-09-07 is that one review
covers Part A and Part B together, after B.

## Change Log

| Date | Change |
|---|---|
| 2026-09-07 | Story created. **Part A over-delivered and that redefined B**: the authored dwarf is already shipped and embedded, so "Part B cannot start until Wolf has a model" is spent and B is the loop, the checker and the sign-off. Seven premises verified on `cb45817`, three of them correcting the record: `check_asset.py` does not reject the dwarf, it **silently under-reads his palette by three cells** including the flame `#F0A63C` (all 16 cells decoded, table in premise 2); `notify-debouncer-full` **cross-compiles AND LINKS** to `x86_64-pc-windows-gnu`, closing Part A's one unclaimed risk with a 13 MB `.exe`; and `source=embedded` is a **hardcoded literal in both instrument lines**, so the line B must change is itself the project's worst instrument shape. Also settled from vendored source: an absolute `AssetPlugin.file_path` replaces the base path via `Path::join`, so option A needs nothing stamped into the binary, and `file_watcher` turns watching on globally unless overridden — AC5. Task 7's scope is UNRULED and carries an open question. |
| 2026-09-07 | **Task 7 RULED, and it is not a "probe".** Wolf: *"one of my requests was to get performance measured to log or something so it's easier to check it after a live run instead of trying to see it from the screen."* So it is a **performance LOG**, read after the fact, not a bench and not an overlay. Scope ruled the same sitting: **frametime + content counters, flag-gated `--perf-log <path>` PLUS an on-demand key at the seat**; the **CPU/GPU split was considered and NOT bought** (it needs `RenderDiagnosticsPlugin` and GPU timestamp queries that `llvmpipe` cannot exercise here — vehicle-only and untestable in the devpod). New AC9/AC10/AC11; mutations moved 9 → 12. The industry framing that decided the columns: average FPS is a mean of reciprocals and hides stutter, so the summary is **percentile frametime** (p50/p95/p99/p99.9) plus a hitch count — and, because of this project's own history, content counters sit on the same row as the time and the summary splits steady-state from edit frames. Also found while scoping it: **`FrameTimeDiagnosticsPlugin` is on the WINDOWED path only** (`ingest.rs:323`); the headless arm never had it. |
| 2026-09-07 | **Wolf ruled the story runs WHOLE, Tasks 1-7**, against the recommendation to split 6-7 off. Open question 2 is closed by that ruling; question 1 is closed by the Task 7 entry above. Question 3 (issue #74) remains OPEN. |
| 2026-09-07 | **Issue #74 JOINS Task 6's vehicle sitting** (Wolf) — dwarves pathing through the campfire and the authored mesh's swinging shadow are about the dwarf this story finishes, so one judgement covers both rather than two vehicle sessions. All three questions raised at story creation are now answered. |

## Dev Agent Record

### Agent Model Used

`claude-opus-5[1m]`, orchestrator and dev in one session, 2026-09-07.

### Completion Notes List

**FULL GATE GREEN, 475s**, pixel guards included; the flaky all-off guard (#72) did not fire.
**Mutation table: 12 rows, all KILLED** — after one round that found a real defect, below.

**Eleven of twelve ACs met. AC8 (UX-DR22) is the exception and is not mine to close** — it needs
Wolf at the Windows vehicle. Task 4's launcher is written but **UNRUN**, for the same reason.

Six findings beyond the ACs, each measured rather than argued:

1. **My own watcher test pinned nothing, and only the mutation table said so.**
   `the_file_watcher_is_armed_only_when_there_is_a_disk_tree_to_watch` asserted
   `args.assets.is_some()` — the flag going *in*, not the decision coming *out* — so forcing
   `watch_for_changes_override` to `Some(true)` left it green. The `AssetPlugin` construction is
   now extracted into `asset_plugin_for` so the decision can be read, and the test asserts both
   directions against hand-written literals. Fourth time this project has hit the self-referential
   shape (1.1, 1.2, 1.3, here).

2. **Batching lost 60 of 300 frames on the perf log's first real run.** The capture path exits by
   **panic** (101 on `main`), and a panic runs no destructors and never returns from `app.run()`,
   so the buffered tail was simply gone — and a short file reads as a short RUN, not a truncated
   one. Every row is now written as it happens. Re-measured: **153 rows from a run that still exits
   101**, where batching wrote 120.

3. **A column named `chunks` read 40,148.** That is `TerrainTile`, one per exposed cell — a
   plausible-looking number under a name that made it mean something else. Renamed `terrain`, and
   it now sums `TerrainTile` **and** `TerrainChunk`, because `--subdiv 1` draws the first and
   `--subdiv N>1` the second: a column that silently counts nothing in one of two modes is worse
   than no column.

4. **This story broke two of story M2-1's mutation rows, and the gate refused the commit.**
   Registering a system between `toggle_overlay,` and `fall_snow,` invalidated two rows that
   matched that **adjacent pair**. Re-anchored on one exactly-once symbol each, and both re-verified
   KILLED by running them. A row that cannot apply pins nothing however green its record reads.

5. **AC2's second window nearly became a false finding.** A window with no dwarf in it showed 72
   changed pixels — which reads as the feature leaking across the frame. Its floor *in that same
   window* is **74**. The change was below its own floor: animated snow, which `--static-world` does
   not stop because it is not simulation state.

6. **`meshes=5` could not have caught a scene that loaded and drew nothing.** So the frames were
   **looked at**, not only counted: the 6x crops show five pines standing where five dwarves stand.

Two things checked and found **not** to be defects, recorded so nobody re-opens them: `mutate.sh`'s
cargo NOT-RUN guard works correctly (a non-existent test name reports NOT-RUN, not SURVIVED — an
earlier SURVIVED of mine came from a truncated extraction, not the guard); and the perf log's
`t_ms` failure was my own test racing `File::create`, not a product bug — `t_ms` is now anchored to
the first recorded frame.

### File List

- `crates/gui/src/project.rs` — `SceneSource`, the two load prefixes, the two `source=` lines
- `crates/gui/src/ingest.rs` — `--assets`, `--perf-log`, `--version`, `asset_plugin_for`,
  `record_perf_frame`
- `crates/gui/src/perf.rs` — NEW, the per-frame log
- `crates/gui/tests/pixel_guard.rs` — AC6's real-binary test
- `Cargo.toml` — `file_watcher` and its justification against the trim
- `scripts/bench/check_asset.py` — the palette reader; `PALETTE_HEX` deleted
- `scripts/bench/perf_summary.py` — NEW, percentiles + the steady/edit split
- `scripts/tests/test_check_asset.py`, `scripts/tests/test_perf_summary.py` — NEW tests
- `scripts/launch-gui.ps1` — NEW, Task 4, **unrun**
- `src-assets/prompts/dwarf-miner-blender-mcp.md` — the corrected under-read note
- `_bmad-output/implementation-artifacts/mutations/10-5b-the-art-iteration-loop.sh` — NEW, 12 rows
- `_bmad-output/implementation-artifacts/mutations/m2-1-live-app-systems.sh` — two rows re-anchored
- `_bmad-output/implementation-artifacts/10-5b-signoff/` — AC2's frames, crops and measurement

### Review Findings

## Open questions for Wolf

### Closed 2026-09-07

1. ~~**The performance probe is recorded nowhere. What is it measuring?**~~ **ANSWERED.** It is a
   performance **log**, written to a file so a live run can be read afterwards rather than watched
   on screen. Full ruling in the Change Log; scope in AC9-AC11 and Task 7. My guess at story
   creation — the 10.6 edit-cost question — was *part* of it but not the ask: the ask was about
   **when you get to read the numbers**, not which numbers. Recorded because that distinction is
   the reason the task was unrecoverable from the repo: nothing in it was about performance
   *measurement*, it was about performance *reporting*.
2. ~~**Split Tasks 6-7 into their own story?**~~ **NO — the story runs whole, Tasks 1-7** (Wolf,
   2026-09-07), against my recommendation. Noted so a later retro can see the call was made
   deliberately rather than by drift.

### Still open

3. ~~**Issue #74 — does it join Task 6's vehicle session?**~~ **YES — Wolf, 2026-09-07.** It is
   about the dwarf this story finishes, so the same sitting judges it: dwarves pathing through the
   campfire, and the authored mesh's shadow swinging wildly. Added to Task 6.

### Nothing open

All three questions raised at story creation are answered.
