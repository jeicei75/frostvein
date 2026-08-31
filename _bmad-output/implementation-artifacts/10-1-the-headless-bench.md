---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: 212fbcdc3caa0bf2daba821fe1598df2c1fdbf38
---

# Story 10.1: The Headless Bench

Status: done

## Story

As the boss,
I want a committed script to render a reviewable image of proposed look work from our actual
world data,
so that I can judge a look before anyone builds it, and the artifact UX-DR22 demands stops being
hand-made every time.

## The venue AND the workload are both measured — 2026-08-29, at story creation

AC1 says the planning probe "proved the venue, not the workload". That re-measurement was
**executed at creation**, not inherited: a throwaway bench was built against a real export, run
twice, and the two PNGs compared with a stdlib decoder.

| quantity | planning probe (test cube) | **real valley, measured here** |
| --- | --- | --- |
| exposed cells | — | **44,984** |
| mesh handed to Cycles | 1 cube | 244,568 verts / 61,142 quads (exposed faces only) |
| Cycles internal render | 0.54 s | **2.01 s / 2.06 s** |
| whole `blender --background` process | 1.39 s | **3.67 s / 3.76 s** |
| pixel determinism @ 960x540 | 0 of 2,073,600 differ | **0 of 2,073,600 differ** |

**The devpod handles the real workload, so the epic's gingerspice fallback (its AC4) does not
fire.** The frame was viewed: real terrain, 9.4's green spruce, ramps.

44,984 is the client's own cube oracle for the shipped seed, reached here through an independent
**data** path (wire JSON -> Python -> Blender). Note the limit of that evidence: the data is
independent, the *predicate* is a reimplementation of `is_exposed` — an independent oracle for
the geometry, not for the rule. **It is a content measurement, not an invariant** — 9.4 moved it
twice in one story (53,365 -> 45,261 -> 44,984), and **10.4 in this same epic exists to move it
again.** Nothing in the gate may pin it.

Three of the epic's premises were checked at creation; one is false (`bevy_gltf` IS enabled,
which corrects 10.5's AC, not this story), one has the wrong reason (Eevee is blocked by a
missing `libegl1`, not llvmpipe — keep Cycles), and the "1.0 s" figure conflates Cycles-internal
with whole-process time. **None changes this story's work.** Full detail is in the
`10-1-the-headless-bench` block of `sprint-status.yaml`; do not re-derive it.

## The export file already exists — do NOT invent a second one

`5-4-signoff/capture_snapshot.py` spawns the real `simd`, reads the snapshot-on-connect line off
the socket, and writes wire-true JSON. That IS the epic's "explicit export file", and it crosses
the protocol rather than the repo. Verified at creation: 7.1 MB, `dims 128x128x32`, `tiles` a
flat list of 524,288 externally-tagged values (`"empty"`, `{"solid":"stone"}`,
`{"ramp":"snow"}`), plus `entities`, `designations`, `zones`, `items`.

**Promote it into `scripts/bench/`. Do not write a new exporter, and do not add an export flag to
`simd`.** (`Command::Save` also writes JSON, but it is a save format with RNG state in it.)

**What must NOT be reused is its neighbour.** `5-4-signoff/artifact_render.py` is the hand-made
mock this story retires, and the recorded 5.4 failure: it drew trees as "snow-laden spruce
sprites" — geometry nobody was tasked to build — so the built-vs-artifact comparison failed on
trees by construction. Do not port its tree code, and do not port its printed `53,365` oracle.

## Acceptance Criteria

### The gate

1. `scripts/gate.sh` passes on the full tier with the story's work in place.

### The bench renders our valley

2. `blender --background --python scripts/bench/valley_bench.py -- <snapshot.json> <out.png>`
   writes a PNG of the default-seed valley, reading world data from the named export file and
   from nothing else. The script imports no repo Rust source, runs no cargo command, and opens no
   socket. *Mechanism is load-bearing: this is the epic's portability constraint — the bench is
   expected to move to its own venue, and an export file is the only thing that travels with it.*
3. Every solid cell the bench draws comes from the export file's `tiles`, and its drawn set is
   the exposed-face set of those tiles. No sprite, billboard, or substituted shape appears for
   any material. *Mechanism is load-bearing: this is the 5.4 defect — an artifact that draws
   geometry nobody is tasked to build cannot be compared against the built result.*
4. The bench prints its exposed-cell count, and the count changes when world content changes.
   **No test may pin its value**; the shipped-seed figure is an expected observation a human
   reads in the Verification recipe.
5. `scripts/bench/export_world.py` writes the snapshot to a named file from a real `simd`, runs
   from either devpod mount path, and is the only way world data enters the bench.

### The bench cannot report success on nothing

6. The bench prints a range-check line, **then** asserts on it, carrying at least: exposed cells,
   the fraction of pixels differing from the sky colour, and distinct colour count. It exits
   non-zero when any figure is below its named floor. *Mechanism is load-bearing: `--at-tick`
   wrote `AppExit::error()` into a discarded return and a run that captured nothing exited 0;
   3.3's recipe captured zero of every glyph and exited 0. Exit 0 is not a result.*
7. A test drives the bench over two export files whose world content differs and asserts the
   reported geometry summary **changes**; a second test does the same for the **pixel** figures
   between a **populated** world and an empty one, **through a real render**. A test that only
   asserts the bench ran satisfies neither. *Both halves of the instrument must visibly differ
   when what they report on differs.*
   *(**Reworded 2026-08-29 by Wolf, at code review.** The AC originally said "between a one-cell
   world and an empty one". Measured: at the fixed boot camera those two render **pixel-identical**
   — 0 of 2,073,600 values differ — because the single cell falls outside the frame, so the
   comparison could never have proved anything. The delivered test had quietly substituted two
   hand-built pixel lists and rendered nothing. A populated world is the smallest input that
   exercises the pixel half end to end.)*
8. Running the bench on an export file with no solid tiles exits **non-zero** and claims no
   success, proved by a test that spawns Blender against a minimal synthetic export.
9. The bench's terrain palette, light table and boot-camera constants are literal-equal to the
   client's, proved by a test that reads `crates/gui/src/appearance.rs` and
   `crates/gui/src/camera.rs` and compares. *This is the mechanical guard against the 5.4 drift
   class.*

### Evidence

10. Wall-time and pixel-determinism are re-measured **on the delivered bench** and recorded here,
    with Cycles-internal and whole-process figures distinguished. Creation-time figures were
    measured on a throwaway bench with no sky and do not carry over.
11. `scene.cycles.use_denoising` is set False explicitly, with a comment naming the hard failure.
12. A sabotage table exists at
    `_bmad-output/implementation-artifacts/mutations/10-1-the-headless-bench.sh`, every row
    KILLED, zero APPLY-FAILED, and `python3 scripts/audit-mutations.py` runs clean.
13. The bench's own tests run inside `scripts/gate.sh`, so a broken bench fails the gate.
14. `docs/tech-art-guidelines.md`'s "Boot framing" skyline figure reads 24 %, matching the code
    and its test.

### Wolf's eye — the closing half, which no agent can check

15. One bench artifact and one `gui --capture` frame of the same seed and framing are committed
    side by side in `_bmad-output/implementation-artifacts/10-1-signoff/`, with the known
    differences written down, and Wolf judges whether the bench artifact **predicts** what the
    client shows. A bench whose picture does not predict the build is the 5.4 failure with better
    tooling, so this is the story's real bar.
    *No UX-DR22 opening artifact is proposed for 10.1. UX-DR22 binds "visually subjective"
    stories, and this one reproduces the client's existing look in a second renderer rather than
    proposing a look — there is nothing to approve in advance, and its bar is fidelity, not
    taste. Flagged as an interpretation of a process obligation, not settled by an agent.*

## Tasks / Subtasks

- [x] **Task 1 — The export file (AC: 5)**
  - [x] Promote `5-4-signoff/capture_snapshot.py` to `scripts/bench/export_world.py`: spawn
        `simd 0`, parse the `listening on 127.0.0.1:<port>` banner, read one snapshot line, write
        it out, print tick/entities/dims. **Keep the banner parse** — `simd`'s only argument is
        the port and `0` means OS-assigned, which is exactly why the parse exists.
  - [x] **Replace the hardcoded `/workspace/projects/frostvein/target/debug/simd`** with a path
        derived from the script's own location. `gate.sh:58-63` records that this repo is mounted
        at two different absolute paths; the literal fails on one of them and contradicts AC2's
        portability constraint.
  - [x] The exported tick is a **sample, not a property** (the original sleeps 2.1 s to land near
        tick 20). Entity positions differ between exports; terrain does not. Say so in a comment.
  - [x] Leave the original in `5-4-signoff/` untouched — it is provenance for an approved artifact.

- [x] **Task 2 — The bench script, geometry half (AC: 2, 3, 4)**
  - [x] `scripts/bench/valley_bench.py`, run as
        `blender --background --python scripts/bench/valley_bench.py -- <snapshot.json> <out.png>`.
  - [x] **Guard the `bpy` import** (`try: import bpy / except ImportError: bpy = None`) so the
        pure functions are importable by a plain `python3` test without Blender.
  - [x] **stdlib only.** No numpy, no PIL. Measured at creation: Debian's Blender does not bundle
        a Python — it resolves to the uv CPython at
        `/home/vscode/.local/share/uv/python/cpython-3.13-linux-x86_64-gnu`, so Debian's
        `python3-numpy` is not on its `sys.path`, and appending `/usr/lib/python3/dist-packages`
        fails on numpy's own source-tree guard. `mathutils` ships with Blender and is fine.
  - [x] **The exposed predicate, exactly** [project.rs:429-442, NEIGHBOURS at project.rs:24-31]:
        **six orthogonal neighbours only**, `Solid(_)` and `Ramp(_)` both occlude, `Empty` does
        not, and **out-of-bounds counts as not-solid** so every world-edge cell is exposed. All
        three are needed to reproduce the client's count; each is a natural coin-flip in a
        reimplementation. Index is `x + y*dx + z*dx*dy`.
  - [x] Emit **exposed faces only**, one mesh, via `from_pydata` + `foreach_set("material_index")`.
        Measured: 244,568 verts / 61,142 quads, built in 0.9 s. **Do not create one Blender object
        per cube** — that is the difference between 2 s and minutes.
  - [x] Print the exposed-cell count. **Do not assert its value anywhere that runs in the gate**
        (AC4) — 10.4 will move it, and a pinned count turns a correct change into a red gate.
  - [x] `// NOTE:` the one known divergence: the client's boot draw set is
        `is_exposed(..) || (z == level && solid)` with `level = dims.z-1` [project.rs:1041-1073],
        so the client draws a thin extra top layer the bench does not. Small; named so it is not
        chased.

- [x] **Task 3 — The bench script, look half (AC: 2, 9)**
  - [x] Terrain materials from `appearance.rs::material_color`, sRGB u8 converted to linear for
        the Principled BSDF: Stone `(60,70,92)`, Soil `(56,52,62)`, Ice `(104,128,170)`,
        Snow `(136,150,178)`, TreeTrunk `(43,47,58)`, TreeFoliage `(44,100,58)`.
  - [x] **Include the two presentation swaps, or the picture will not predict the build.**
        `snow_cap_color()` `(146,158,184)` [appearance.rs:233-239] — its own comment says at the
        boot pitch the caps dominate the visible area — and `foliage_snow_color()` `(156,170,196)`
        for the exposed spruce crown [appearance.rs:241-247, project.rs:934-959]. The crown rule
        is client-derived, not in the wire: exposed above AND not resting directly on ground.
  - [x] Sky as a flat background at `night_lighting().sky` `(5,12,28)` — the client's sky is a
        flat `ClearColor` [ingest.rs:198].
  - [x] Ambient `(120,140,165)` and directional `(150,190,180)` from the same table. Point lights
        from `light_properties()`: Torch `(255,140,62)`, Campfire `(255,173,92)`, Lantern
        `(255,195,110)`. Note lanterns are **not** an `EntityKind` — `EntityKind` is
        `Dwarf|Torch|Campfire` and a lantern arrives as `Entity.light: Some(Lantern)` on a dwarf
        [protocol/src/lib.rs:37-49, 100-107].
  - [x] Entities as cubes at `entity_appearance` colour and scale (Dwarf `(151,116,96)` @ 0.65).
  - [x] **The camera** — see Dev Notes for the exact block. **Blender FOV trap:** Bevy's
        `PerspectiveProjection.fov` is the *vertical* FOV [ingest.rs:707-710], while Blender's
        `camera.data.angle` follows `sensor_fit`, which defaults to AUTO and therefore means
        *horizontal* on a 16:9 render. Set `sensor_fit = 'VERTICAL'` (or set `angle_y`), or the
        frame comes out visibly tighter than the client's.

- [x] **Task 4 — The range check that can actually fail (AC: 6, 10, 11)**
  - [x] After rendering, read the frame back — Blender hands you the pixels directly
        (`bpy.data.images.load(out).pixels`), so no PNG decoder is needed inside the bench.
  - [x] **The emptiness metric is distance from the SKY colour, not from black.** An empty frame
        renders 100 % non-black once Task 3 sets the sky to `(5,12,28)`, so a non-black floor
        cannot detect one. Count pixels differing from the sky colour beyond a small tolerance.
  - [x] Print one range-check line naming every figure, **then** assert. A failing run must still
        show its numbers.
  - [x] Exit non-zero on any floor breach, and confirm the non-zero reaches the shell —
        `bpy.ops.render.render()` returning is not the bench succeeding.
  - [x] **Measure the floors on the delivered bench, with sky and lights in place, and record
        them.** The creation-time 79.3 % / 22,422 figures came from a throwaway bench with a
        black background and do not transfer. Give each floor a comment naming the figure it came
        from and what moves it; set them well below measurement so reframing does not trip them.
  - [x] Cycles CPU, 32 samples, 960x540. `scene.cycles.use_denoising = False` with a comment
        naming the hard failure: `RuntimeError: Error: Failed to denoise, build has no
        OpenImageDenoise support`, exit 1, no PNG written.

- [x] **Task 5 — Tests, and getting them into the gate (AC: 7, 8, 9, 13)**
  - [x] `scripts/tests/test_valley_bench.py` (unittest): exposed-face extraction over small
        hand-built worlds (including an out-of-bounds edge case), the floor functions, and AC7's
        two assertions — geometry summary changes across two worlds, **and** the pixel figures
        change between a one-cell world and an empty one.
  - [x] **`scripts/bench` is not on `sys.path`** for `unittest discover -s scripts/tests`. Insert
        it explicitly or use `importlib`; the `_bmad/scripts/tests` precedent does not cover this
        because its subject is a sibling.
  - [x] AC8's test **spawns Blender** against a minimal synthetic export (a handful of tiles, and
        an empty one) and asserts the process exit code. Keep the synthetic worlds tiny — this
        must not render the full valley, because it runs on every commit via the pre-commit hook.
        Guard with `skipUnless(shutil.which("blender"))` so a Blender-less machine skips rather
        than fails, and **report which tests actually ran** — a skipped AC8 has judged nothing.
  - [x] Add one line to `scripts/gate.sh` using its existing `run` helper exactly as the
        neighbouring line does [gate.sh:116]:
        `run "bench tests" python3 -m unittest discover -s scripts/tests`.
  - [x] **The AC9 drift guard goes in Rust**, at `crates/gui/tests/bench_contract.rs`. The reason
        is not the mutation harness (Task 6 gives that a Python tier anyway): it guards the
        *client's* constants against a downstream consumer, so it belongs with the client's suite
        and must run even where Blender is absent.
  - [x] Two things that will otherwise cost the dev an hour: an integration test's cwd is the
        **package** root, so reach the repo root the way the existing test does —
        `Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")` [crates/gui/tests/capture.rs:159].
        And the boot-camera constants are **private** `const`, not `pub` (only
        `BOOT_VERTICAL_FOV`/`BOOT_ASPECT_RATIO` are public) — so the comparison must scrape the
        Rust source text on both sides. **Do not add `pub` to make it importable**; that is a
        crate change this story forbids.

- [x] **Task 6 — The sabotage table, and the harness gaps it exposes (AC: 12)**
  - [x] **Commit first, then mutate.** Never `git checkout --` over an uncommitted fix.
  - [x] **Gap 1, a live trap worth closing regardless of this story:** `backup_all` snapshots only
        `$(git ls-files 'crates/*')` [mutate.sh:46], so a sabotage patching anything under
        `scripts/` is **never restored** — it silently survives the run. Extend the backup set.
  - [x] **Gap 2:** `mutation()` runs `cargo test --offline -p "$pkg" "$test"` [mutate.sh:69]. Add
        a `py` tier that runs the unittest instead.
  - [x] **Gap 3, and this one is the whole point of the script:** KILLED is decided as
        "not `could not compile`, then `rc != 0`" [mutate.sh:82-94], and that compile guard is
        Rust-specific. A Python sabotage that lands a `SyntaxError`/`ImportError` exits non-zero
        and would print **KILLED while proving nothing** — verbatim the false-kill class the
        script's own comment records from story 5.3 [mutate.sh:71-79]. The `py` tier must treat
        collection/import/syntax errors as **SURVIVOR**, and require a genuine assertion failure
        to call a row KILLED.
  - [x] Rows, at minimum: (a) a palette literal in `appearance.rs` changed -> the AC9 drift test
        goes RED; (b) a boot-camera constant in `camera.rs` changed -> the same; (c) the bench's
        range-check assertion deleted -> AC6's test goes RED; (d) the exposed-face neighbour test
        inverted so every face is emitted -> AC7's geometry assertion goes RED; (e) the non-zero
        exit replaced with `exit(0)` -> AC8's test goes RED.
  - [x] **Check WHICH assertion kills each row, not just that it says KILLED.** An earlier pin
        absorbing the mutation is this project's most repeated review finding — the strengthened
        test then looks identical to the weak one from outside.
  - [x] Confirm `python3 scripts/audit-mutations.py` still runs clean over every table.

- [x] **Task 7 — The instrument, named and demonstrated (AC: 4, 6, 10)**
  - [x] **The instrument for this story is the bench itself**; its exact command is in
        Verification. It is not a substitute for the tests above, and Task 5 is its own test.
  - [x] Run it end to end on the shipped seed. Record the exposed-cell count, the Cycles-internal
        time, the whole-process time, and the full range-check line in the Dev Agent Record.
  - [x] Run it **twice** to different outputs and diff the pixels (RGBA values, `w*h*4`). Record
        how many differ. **Do not tick AC10 until the numbers are pasted in** — a checkbox is
        worth only what its verification is worth.

- [x] **Task 8 — The comparison pair for Wolf (AC: 15)**
  - [x] Create `_bmad-output/implementation-artifacts/10-1-signoff/`.
  - [x] Commit one bench artifact and one `gui --capture` frame at the boot framing. Both are
        producible headlessly here (9.1 established the path; the 9.4 signoff PNGs were made this
        way). `--capture` requires `--frames N` or `--at-tick N`, needs `--headless` on a devpod,
        and needs a live `simd` [ingest.rs:394-396, 497, 1464-1478]. Put the exact command you
        used in the Dev Agent Record.
  - [x] Write a short `what-you-will-see.md` naming what to compare and, up front, **every known
        difference** so Wolf is not asked to rediscover them: a path tracer against a rasterizer;
        `gui --headless` renders 1280x720 against the bench's 960x540 (same aspect); no aurora,
        stars, fog or `rim_level` in the bench; and the two frames sit at different ticks, so
        dwarf positions differ while terrain does not.

### Review Findings

Code review 2026-08-29, fresh context, four layers (Blind Hunter / Edge Case Hunter on Sonnet,
Acceptance Auditor / Feature Auditor on Opus), per-layer `CARGO_TARGET_DIR`. **Zero coverage
holes** — every layer ran `cargo --version` clean, every layer executed the binaries, none timed
out, none was killed. Convergence: 3 findings raised independently by two layers (marked below).
Severity set by the orchestrator after reading each site, not inherited from the layers.

**What the review CONFIRMED as honest:** the full gate is GREEN (run, not claimed); `strace` shows
**0 `connect()` / 0 `socket()`** and the only repo file the bench opens is itself (AC2 proven
mechanically); 61,142 drawn quads == 61,142 exposed faces (AC3); no test pins 44,984 anywhere
(AC4); `bench-valley.png` reproduces from current code **bit-for-bit**, 0 of 2,073,600 values
differing; all 7 mutation rows replay RED on their named assertion; `audit-mutations.py` clean over
415 rows. The recorded figures (44984 / 61142 / 0.674020 / 45642) reproduced exactly on three
independent runs.

- [x] [Review][Patch] **AC7's pixel half is unachievable as worded** — the AC asks for pixel
      figures that change "between a one-cell world and an empty one", but a real one-cell export
      and a real empty export render **pixel-identical (0 of 2,073,600 values differ)** at the
      fixed boot camera: the single cell is off-frame, both report
      `non_sky_fraction=0.000000 distinct_colors=4`. The committed test
      [scripts/tests/test_valley_bench.py:53-63] calls `pixel_figures()` on two hand-built pixel
      lists and renders nothing, so the AC is ticked against a substitution. Residual risk is
      bounded (AC6's floor fires at runtime; mutation row (g) drives a real render), but **no
      committed test asserts a pixel figure from a populated real render.** Options: (a) reword
      AC7 to "a populated world vs an empty one" and add that test; (b) place the one cell in
      frame; (c) accept the substitution and record why. `auditor+feature`
      **Wolf's call, 2026-08-29 — option (a):** reword AC7 to a populated world vs an empty one,
      and add a test that asserts the pixel figures from a real populated render.
- [x] [Review][Patch] **The bench ignores `foliage_scale`, so tree silhouettes do not predict
      the build** — the client draws every foliage cell at **0.62–0.95** of its cell
      [crates/gui/src/project.rs:868-892, "Keeps cube foliage readable as sparse spruce branches
      instead of a solid square canopy"]; the bench emits full unit faces for every material
      [scripts/bench/valley_bench.py:205-213]. In matched crops the client's trees are a slim trunk
      carrying separated, shrunken crown cubes with sky between them; the bench's are solid green
      slabs with a white lid. AC3 is still met (a scale, not a substituted shape). **This is the
      geometry 10.4 will be judged on, using this bench.** Options: (a) apply `foliage_scale` now;
      (b) document it as a known difference and let 10.4 fix the bench first. Note the "no tree
      redesign" guardrail forbids *redesigning* trees, not reproducing the client's current draw.
      `feature`
      **Wolf's call, 2026-08-29 — option (a):** apply `foliage_scale` to the bench now, so 10.4 is
      judged against a bench that draws today's trees correctly.
- [x] [Review][Patch] **The bench's key-light direction is invented, 122° from the client's** —
      [scripts/bench/valley_bench.py:354] `rotation_euler = (-35°, 20°, 30°)`; the client aims its
      directional light by `aurora_light_transform()` [crates/gui/src/atmosphere.rs:209-211].
      Measured: client light direction `(0.761, 0.112, 0.639)`, a near-horizontal rake from 6.4°
      *below* the horizon; bench sun `(0.044, -0.637, -0.770)`, from 39.6° elevation on the
      opposite side — **122.0° apart**, so different faces are lit and shadowed. The spec fixed the
      light *colours*, never the direction. Options: (a) aim the sun with the client's transform
      (aiming is not aurora geometry, so the guardrail arguably permits it); (b) document as a
      known difference. `feature`
      **Wolf's call, 2026-08-29 — option (a):** aim the sun with the client's
      `aurora_light_transform()`; aiming a light is not building aurora geometry.
- [x] [Review][Patch] **HIGH — Exit 0 is not a result: only `assert_range` is guarded** — the sole
      exception translated into a non-zero exit is `assert_range`'s `AssertionError`
      [scripts/bench/valley_bench.py:402-405], whose own comment explains why the translation is
      needed. Every other exception in `main()` is unguarded and Blender's `--background --python`
      runner prints the traceback and **still exits 0**. Reproduced three ways: malformed JSON
      (`JSONDecodeError`, exit 0), an unknown entity kind (`KeyError` at :336, exit 0), and `dims`
      claiming more cells than `tiles` holds (`IndexError` at :148, exit 0). Any malformed export,
      any protocol drift, or any `export_world.py` bug crashes the renderer mid-script and reports
      success. Fix: wrap the body of `main()` in `except Exception as error: raise SystemExit(...)
      from error`. [scripts/bench/valley_bench.py:376-405] `blind`
- [x] [Review][Patch] **HIGH — `AMBIENT_RGB` is a dead constant; the bench applies no ambient
      light** — [scripts/bench/valley_bench.py:45] defines it and the only other occurrence in the
      repo is the assertion that pins it [crates/gui/tests/bench_contract.rs:50-52]. `setup_scene`
      builds a sun, point lights and a sky-coloured world background, and never an ambient fill;
      the client applies `AmbientLight { color: night_lighting().ambient, brightness: 4_500.0 }`
      [crates/gui/src/ingest.rs:714-718]. Measured over non-sky pixels of the committed pair by two
      layers independently: bench mean luma **68.7** vs client **90.9** (medians 74.9 / 99.0), mean
      terrain RGB `(56,81,92)` vs `(70,96,120)` — the artifact is systematically ~24 % darker and
      less blue than the build it must predict. **Task 3 ticks this sub-requirement `[x]` and AC9
      pins the literal as though it proved something.** Same class as this story's own two fixed
      defects: the constant is right and nothing consumes it. `auditor+feature` (convergence)
- [x] [Review][Patch] **HIGH — `what-you-will-see.md` omits differences Wolf is asked to judge
      around** — AC15 requires **every** known difference written down so Wolf is not asked to
      rediscover them. The file [_bmad-output/implementation-artifacts/10-1-signoff/what-you-will-see.md:14-27]
      lists renderer, resolution, aurora/stars/fog/rim_level, tick and the top slice. It omits: the
      missing ambient light (the largest tonal difference after the aurora); the invented key-light
      direction; the ignored `foliage_scale`; the client's snow cap being a **separate raised slab**
      `Cuboid::new(1.02, 0.08, 1.02)` at `+Y*0.54` [project.rs:195, 894-906] where the bench
      recolours a top face [valley_bench.py:219-220]; visible Cycles sampling grain at 32 samples
      with denoising correctly off; and ramp tiles drawn as full cubes. Content depends on the
      three decisions above. `auditor+feature` (convergence)
- [x] [Review][Patch] **MED — FOV and aspect are a parallel copy, so the framing test is blind to
      the renderer's projection** — `project_boot_point` re-declares `math.pi / 4` and `16.0 / 9.0`
      as its own literals [scripts/bench/valley_bench.py:126-128] while the renderer reads
      `camera_data.angle` [:359] and `resolution_x/y` [:315-316]. Demonstrated on a `/tmp` copy:
      widening `camera_data.angle` to `math.pi / 3` changes **1,050,234 of 2,073,600 pixel values**
      — a visibly different frame — while the framing test still reads
      `camp=(0.500, 0.779) skyline_y=0.240`, inside tolerance, and the range check still exits 0.
      Only the text scrape catches it, which is the guard that stayed green through the 110° roll.
      The comment at [:363-366] claims "a change here cannot leave that test green against a frame
      it no longer matches" — true for the basis, **false for the projection**. `feature`
- [x] [Review][Patch] **MED — AC9's dwarf row anchors a match-arm header, not the values** — the
      client literal is `"EntityKind::Dwarf => EntityAppearance {"`
      [crates/gui/tests/bench_contract.rs:69-72]; every other row anchors the value line itself.
      Mutating the dwarf colour to `(9,9,9)` and scale to `0.01` in a fixture left the anchor
      matching. The bench's pinned `"dwarf": ((151, 116, 96), 0.65)` can go stale with the guard
      green, and the sabotage table has no dwarf row. [crates/gui/src/appearance.rs:277-278]
      `edge+auditor` (convergence)
- [x] [Review][Patch] **MED — AC9's light rows match a literal that occurs 4x file-wide** —
      [crates/gui/tests/bench_contract.rs:57-68] matches `"color: Color::srgb_u8(255, 140, 62)"`
      against the whole of `appearance.rs`, not against the `LightKind::Torch` arm. Counted in
      current source: `255, 140, 62` **4 occurrences**, `255, 173, 92` **4**, `255, 195, 110` **3**.
      Swap Torch and Campfire colours in the client and every literal is still present — the test
      stays green while the bench's lights no longer match. `auditor`
- [x] [Review][Patch] **MED — the `py` tier reports SURVIVED, with no diagnostic, when Blender is
      merely absent** — 3 of the 7 rows target `ValleyBlenderTests`, which is
      `skipUnless(shutil.which("blender"))` [scripts/tests/test_valley_bench.py:79]. With Blender
      off `PATH` the row exits 0, misses the NO-COLLECT regex, and falls to the SURVIVED branch
      [scripts/mutate.sh:69-73, 99-103]; its diagnostic greps for cargo's `test result` string,
      which never matches unittest output, so the operator sees an empty row and reads "the test is
      not pinning what it claims" when the test never ran. This is the false-KILL class the story
      closed, reappearing as its mirror. `edge`
- [x] [Review][Patch] **MED — a mistyped tier is reported as KILLED** — `[ "$tier" = "py" ]` is the
      only gate [scripts/mutate.sh:69-73]; anything else (`"Py"`, a stray space, empty) goes to
      `cargo test --offline -p "$tier"`, which exits **101** with `did not match any packages` /
      `package name cannot be empty`. Neither matches the `could not compile` guard, so `rc != 0`
      → **KILLED** [scripts/mutate.sh:82-94], a clean kill proving nothing about any test. `edge`
- [x] [Review][Patch] **MED — the gate prints `ok` while bench tests skip** — [scripts/gate.sh:117]
      with the `run` helper [:73-83] discarding stdout on success. With Blender off `PATH`,
      `python3 -m unittest discover -s scripts/tests` exits 0 printing `OK (skipped=2)`, and the
      gate line reads `bench tests  ok` with no skip announcement. Task 5 explicitly required
      "report which tests actually ran — a skipped AC8 has judged nothing." `edge`
- [x] [Review][Patch] **MED — the orchestrator's dev-phase spend is unrecorded** — the ledger has
      **zero `dev | claude` rows** for 10.1, only two `dev | codex` rows ($2.12 + $0.29), and
      session `2870b2e6` (2.4 MB, running to 18:37:49 UTC) is **absent from `.session-cursors.json`
      entirely** — neither rowed nor `--mark`ed. That window produced `74f3a23`..`40acd37`: the
      verification, both defect fixes, the two new tests, the two extra sabotage rows, the
      re-rendered pair, the gate and the record. So 10.1's dev reads as $2.41/132 turns while the
      work that found and fixed both HIGH defects is billed to nobody, which will corrupt Epic 10's
      dev-vs-review comparison. Per the METRIC RULE that window was owed a **row**. Still
      recoverable: the transcript is on disk and the cursor never advanced past it.
      [_bmad-output/implementation-artifacts/metrics/10-1-the-headless-bench.md] `orchestrator`
- [x] [Review][Patch] **LOW (latent silent-failure — patched under the standing exception) —
      `audit-mutations.py` prints Rust-only wording for py-tier orphans** — [scripts/audit-mutations.py:186-188]
      unconditionally prints ``no `fn {test}` anywhere under crates/`` even though the tier-aware
      search added in this diff [:99-107] looks for `def` under `scripts/` when `tier == "py"`.
      Reproduced against a scratch fixture. A developer chasing a real orphaned py row is sent to
      look for a Rust `fn` under `crates/`. `edge`
- [x] [Review][Patch] **LOW (latent silent-failure — patched under the standing exception) — the
      widened backup set still misses `_bmad/scripts/*.py`** — [scripts/mutate.sh:46]
      `git ls-files 'crates/*' 'scripts/*'` now covers this story's Gap 1, but leaves
      `_bmad/scripts/session_tokens.py` and its tests outside the restore set, while `gate.sh:116`
      runs them. The `py` tier is generic, so a future row sabotaging that file would be **silently
      left mutated on disk**, contradicting the file's own header ("Every tracked file is restored
      from a backup after each mutation"). Same shape as the gap this story closed. `edge`
- [x] [Review][Defer] **No `timeout=` anywhere in the new Blender-spawning chain**
      [scripts/tests/test_valley_bench.py:81-101; scripts/mutate.sh:69-70; scripts/gate.sh:117] —
      deferred, first subprocess in this repo that can hang the gate indefinitely; no observed hang.
- [x] [Review][Defer] **`export_world.py` ignores `CARGO_TARGET_DIR`** [scripts/bench/export_world.py:18]
      — deferred, AC5's actual requirement (both devpod mounts) is met; a target-dir override yields
      a stale or missing binary, the stale-binary trap's seventh shape.
- [x] [Review][Defer] **AC9 guards light colours but not intensities**
      [crates/gui/src/appearance.rs:45,48 vs scripts/bench/valley_bench.py:344,352] — deferred,
      Blender and Bevy units genuinely differ so literal equality is impossible; an interpretation
      question for 10.3's contract work, not a defect.
- [x] [Review][Defer] **Ramp tiles render as full cubes** [scripts/bench/valley_bench.py:215-218,
      FACE_CORNERS] — deferred, `Ramp(_)` correctly occludes for the exposed predicate, but no
      sloped geometry exists; a known simplification that was never written down.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No aurora, no star shell, no distance fog, no `rim_level` edge treatment.** Each is bespoke
  client geometry and none is needed to judge terrain, palette and framing. Leave a `// NOTE:`
  rather than building them.
- **No changes to any crate's behaviour.** The only Rust this story adds is a test. Editing
  `appearance.rs` or `camera.rs` — including adding `pub` — means you have taken 10.4's work.
- **No `simd` export flag, no protocol change, no new wire type.**
- **No BlenderMCP, no glTF, no authored assets, no `bevy_gltf`/`file_watcher` enablement** —
  10.2 and 10.5.
- **No tree redesign.** 10.4 owns that, on this bench's evidence. Render the wire's trees.
- **Do not install `libegl1` or chase Eevee.** Cycles CPU is measured sufficient.
- **Do not extend `docs/tech-art-guidelines.md` with contracts** — that is 10.3. AC14's one-figure
  correction is the only edit it takes here.

### What already exists (build on it, do not re-derive)

- The export path: `5-4-signoff/capture_snapshot.py` — proven, wire-true, protocol-only.
- Palette, light table, entity table: `crates/gui/src/appearance.rs`.
- Boot framing and projection: `crates/gui/src/camera.rs`; the axis swap: `transform.rs:4-6`.
- The exposed predicate and the crown rule: `crates/gui/src/project.rs`.
- Repo-root resolution from an integration test: `crates/gui/tests/capture.rs:159`.
- The gate already runs a Python unittest discover — copy that line, do not invent a runner.

### The camera, exactly

Verified against source at creation:

```
world_to_render([x,y,z]) = (x, z, -y)          # Bevy is Y-up  [transform.rs:4-6]
BOOT_YAW      = 0.7      rad                   # [camera.rs:7]   (private const)
BOOT_PITCH    = 0.45     rad                   # [camera.rs:8]   (private const)
BOOT_DISTANCE = 90.0                           # [camera.rs:9]   (private const)
BOOT_COMPOSITION_FORWARD = 33.0                # [camera.rs:16]  (private const)
BOOT_COMPOSITION_LIFT    = -0.5                # [camera.rs:17]  (private const)
BOOT_VERTICAL_FOV = pi/4 ;  ASPECT = 16/9      # [camera.rs:29-30]  (pub)
focus = [64, 64, 9]                            # boot rig  [ingest.rs:700]

horizontal_forward = (-cos(yaw), 0, -sin(yaw))
composition_offset = horizontal_forward * 33.0 + Y * -0.5
target = world_to_render(focus) + composition_offset * min(distance/90, 1)
h   = distance * cos(pitch)
eye = target + ( X*h*cos(yaw) + Y*distance*sin(pitch) + Z*h*sin(yaw) )
look_at(target, up = +Y)
```

The offset **must** lie in the camera's view plane. A push expressed in world axes carries a
28.6-unit component along `right` at this yaw and slides the camp to 23 % of frame width while
every vertical assertion stays green — a recorded, already-paid-for bug [camera.rs:251-263].

Pinned composition [camera.rs:223-249]: camp at **0.48** width / **0.78** height, skyline at
**0.24** height, tolerance **0.03**. The skyline reference point is the render-space literal
`Vec3::new(64.0, 26.0, -128.0)` [camera.rs:231] — you need it to reproduce the check.

### Key decisions & traps

- **stdlib + `bpy` + `mathutils` only.** Measured, not assumed. This is also what keeps the bench
  portable to the venue the epic names.
- **`use_denoising = False` is mandatory**, not a preference — Cycles hard-fails, exit 1, no PNG.
- **Exit 0 is not a result.** Print the numbers, then assert. Shipped three times here.
- **Cycles is a third renderer**, distinct from Bevy/wgpu and from the llvmpipe headless path.
  9.1 measured llvmpipe under-reading near-white area by ~16 % against the vehicle and had to
  re-derive its metric when two renderers disagreed. Treat any bench-vs-client pixel comparison
  as **directional**, never as a calibrated bar.
- **Cycles is deterministic here**, but PNG *bytes* still differ — five `tEXt` chunks carry
  timestamps and render duration. Compare pixels, not files.
- **The stale-binary trap has fired six times.** `export_world.py` runs `target/debug/simd`;
  build it from the branch you are on before you trust an export.

### Previous story intelligence (deltas that change THIS story)

- **This story stacks on `212fbcd`, not `main`.** 9.1 and 9.4 are done but unmerged — `main` is
  49 commits behind and still carries the superseded foliage colour `(55,73,84)`
  [main:appearance.rs:215] against this branch's `(44,100,58)` [appearance.rs:229]. A bench cut
  from `main` would bake in a dead palette. Prove any "no crate behaviour changed" claim against
  **this story's own commit range**, never `main..HEAD`.
- **9.4 moved world content**, so every frozen figure is suspect: trees 704 -> 265, exposed cubes
  53,365 -> 45,261 -> 44,984. `artifact_render.py` still prints the oldest as an "oracle".
- **9.1's headless capture path works** and is how Task 8's `gui --capture` half is produced
  without a vehicle.

### Project Structure (files to touch)

| file | state | why |
| --- | --- | --- |
| `scripts/bench/export_world.py` | NEW | the explicit export file, promoted from 5-4-signoff |
| `scripts/bench/valley_bench.py` | NEW | the bench: stdlib + bpy, geometry + look + range check |
| `scripts/tests/test_valley_bench.py` | NEW | geometry, floors, AC7/AC8 instrument tests |
| `crates/gui/tests/bench_contract.rs` | NEW | AC9 drift guard (test-only; no crate source change) |
| `scripts/gate.sh` | UPDATE | one `run "bench tests" ... discover -s scripts/tests` line |
| `scripts/mutate.sh` | UPDATE | back up `scripts/*`; `py` tier; py-side false-KILL guard |
| `mutations/10-1-the-headless-bench.sh` | NEW | AC12 |
| `_bmad-output/implementation-artifacts/10-1-signoff/` | NEW | AC15 pair + what-you-will-see.md |
| `docs/tech-art-guidelines.md` | UPDATE | AC14: skyline 30 % -> 24 % |
| this story file, `sprint-status.yaml` | UPDATE | status and record |

### Verification

**Executed at story creation, 2026-08-29** — the full gate on `212fbcd`, clean tree, run rather
than claimed:

```
frostvein gate
  cargo fmt --check           ok
  cargo clippy -D warnings    ok
  cargo test                  ok
  tui / client-core / gui have no sim-core edge   ok
  metrics ledger tests        ok
  mutation tables still apply ok
GATE GREEN
```

**Also executed at creation** — AC1's workload re-measurement, on a throwaway bench against a
real export. Blender 4.3.2 (`4.3.2+dfsg-2`), 32 cores, no `/dev/dri`, empty `$DISPLAY`. Figures
in the table above. **Those figures do not carry over to the delivered bench** (no sky, no
lights, no caps) — AC10 requires re-measuring them.

**Run these, and report the named observation beside each — not exit 0:**

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# 0. Build the daemon the export comes from, ON THIS BRANCH
cargo build --offline -p simd

# 1. The export file (AC 5) — report tick, entities, dims
python3 scripts/bench/export_world.py /tmp/snapshot.json

# 2. The bench (AC 2, 4, 6) — report the exposed-cell count and the WHOLE range-check line
blender --background --python scripts/bench/valley_bench.py -- /tmp/snapshot.json /tmp/valley.png
echo "exit=$?"
#    EXPECT ~44,984 exposed cells on the shipped seed. A different number is not automatically
#    wrong — say what moved (world content) or find out why. Nothing asserts this value.

# 3. Determinism at scale (AC 10) — render twice, diff PIXELS not bytes, report how many differ
blender --background --python scripts/bench/valley_bench.py -- /tmp/snapshot.json /tmp/valley2.png

# 4. Emptiness is caught (AC 6, 8) — this MUST exit non-zero; report the exit code and the line
blender --background --python scripts/bench/valley_bench.py -- <an empty export> /tmp/empty.png
echo "exit=$?"

# 5. The bench's own tests (AC 7, 8, 13) — name which tests ran AND which SKIPPED
python3 -m unittest discover -s scripts/tests -v

# 6. The drift guard (AC 9)
cargo test --offline -p gui --test bench_contract

# 7. Sabotage (AC 12) — commit first; run alone; read the exit code before any pipe
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/10-1-the-headless-bench.sh
python3 scripts/audit-mutations.py

# 8. The gate (AC 1) — full tier
scripts/gate.sh
```

**The bench renders in ~2 s. If a run takes minutes, you are creating one Blender object per
cube.**

### Branch and commits

Branch `10-1-the-headless-bench` off **`212fbcd`** (the 9.4 branch tip), not `main`. Author every
commit `Völundr <jeicei75@gmail.com>`. Small, imperative, one commit per completed task minimum;
no squash. Push and PR only on Wolf's explicit yes.

### If this overruns one session

Split after **Task 2**, and split the mutation table with it:

- **Part 1 — the honest instrument:** Tasks 1, 2, 4, 5 (Python half only), 6, 7. ACs 1-8, 10-13.
  Mutation rows (c), (d), (e). This ships the export, the geometry, the range check and its
  sabotage proof — a complete and useful story on its own.
- **Part 2 — the look:** Task 3, Task 5's Rust drift guard, Task 8. ACs 9, 14, 15. Mutation rows
  (a), (b).

Do not split by dropping the range check: a bench without it is exactly the evidence channel this
project has been burned by.

### References

- Epic 10 header and story 10.1 ACs — `_bmad-output/planning-artifacts/epics.md:1348-1470`
- UX-DR22 both halves — `epics.md:218`; UX-DR17 — `epics.md:207`
- PRD asset-pipeline trigger — `prds/prd-frostvein-2026-08-09/prd.md:145-151`
- Pipeline supersession (Blender -> glTF) — `prds/prd-frostvein-2026-08-09/addendum.md:61-68`.
  Note `docs/architecture.md:178-179` and
  `planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:305-310`
  still name the superseded MagicaVoxel path — stale, out of scope here, flagged for 10.3/10.5.
- The 5.4 spruce-sprite failure — `5-4-the-cold-boot.md:405-421`;
  `epic-5-retro-2026-08-23.md:269-273`; `deferred-work.md:635-642`
- Palette, lights, entities — `crates/gui/src/appearance.rs:40-89, 222-289`
- Exposed predicate, crown rule, boot draw set — `crates/gui/src/project.rs:24-31, 429-442,
  934-959, 1041-1073`
- Boot framing and projection — `crates/gui/src/camera.rs:7-30, 223-263`; `transform.rs:4-6`
- Look rules and the cube oracle — `docs/tech-art-guidelines.md`
- Export precedent — `5-4-signoff/capture_snapshot.py`, `5-4-signoff/README.md`
- Mutation harness contract and its false-KILL history — `scripts/mutate.sh:9-19, 46, 69, 71-94`
- Inherited Epic 10 eye-checks folded in from 9.2/9.3 — `epics.md:1406-1434`. Not 10.1's to
  close; they ride on the epic's art pass.

## Dev Agent Record

### Agent Model Used

gpt-5.6-terra

### Debug Log References

**Dev run 1 (Codex, `gpt-5.6-terra`, 9 commits).**
- RED evidence: empty bench first logged `AssertionError: no exposed cells` yet Blender exited 0;
  after explicit `SystemExit`, the same export printed its range line and exited 1. Geometry
  sabotage RED: `{'exposed_cells': 2, 'faces': 12} != {'exposed_cells': 2, 'faces': 10}`. Palette
  sabotage RED: `client literal drifted: Material::Stone => Color::srgb_u8(60, 70, 92)`.
- Handed back three honest caveats: its own full gate never finished, the `gui --capture` command
  exited 101, and it did not run the `codex review` self-gate. All three were correct.

**Orchestrator verification, and the two defects a green gate could not see.**

Both were found by *looking at the rendered frame*, not by reading exit codes. Both are the same
class the story warns about: a guard sitting one level above the thing it claims to guard.

1. **The bench camera was rolled 110 degrees, so the artifact did not predict the build** — AC15's
   stated failure mode. `to_track_quat` levels against Blender's global +Z, but the scene is built
   in Bevy's Y-up render space. Measured against the delivered camera block: scene-up landed at
   `(0.845, -0.310, 0.435)` in camera space (correct is `~(0, 1, 0)`), while Blender's own +Z
   landed at `(0.000, 0.815, 0.580)` — it had levelled to the wrong axis. AC9's drift guard stayed
   green throughout because it compares literals as **text**: the constants were right and the
   maths consuming them was wrong.
   *Fix:* the basis is built explicitly as Bevy's `look_at` does, and `setup_scene` now consumes
   the same `boot_camera_frame()` the framing test checks — one camera, so the test cannot stay
   green against a frame it no longer matches.
   *New test:* asserts the client's own pinned composition (camp 0.48 width / 0.78 height, skyline
   0.24 height, tol 0.03). Non-tautological: against the old camera it reads camp `(0.352, 0.404)`
   and skyline y `1.207` — off-frame entirely.

2. **The pixel half of the range check was inert on real renders.** `pixel_figures` compared the
   readback against `srgb_to_linear(SKY_RGB)`, but a rendered frame reads back display-referred.
   MEASURED: a real all-sky frame reads `(0.01961, 0.04706, 0.1098)`, exactly `SKY_RGB/255`, while
   the code compared against `(0.00152, 0.00368, 0.01161)` — 0.098 away in blue against a 0.02
   tolerance. So **every sky pixel counted as non-sky, and a 100%-sky frame scored
   `non_sky_fraction=1.000000`**, far above the 0.02 floor. The floor could never fire. Only
   `exposed_cells > 0` was catching the empty case; the pixel half reported success on nothing,
   which is precisely what AC6 exists to prevent.
   The unit test could not see it because it built its input from `srgb_to_linear(SKY_RGB)` — the
   same conversion the code used, so it agreed with the code in whichever colour space the code
   picked. The self-referential-test antipattern, third instance on this project.
   *Fix:* compare in the render's own colour space. *New tests:* the sky reference is now the
   measured display-referred literal (an independent oracle), and a new test drives the **real
   renderer** to assert an all-sky frame reads `non_sky_fraction=0.000000`.

**Instrument, re-measured on the delivered bench after both fixes (AC 4, 6, 10):**
- `exposed cells: 44984 faces: 61142` — matches the client's cube oracle for the shipped seed.
  Nothing pins this value; 10.4 will move it.
- `range-check: exposed_cells=44984 non_sky_fraction=0.674020 distinct_colors=45642 floors(non_sky_fraction=0.020000, distinct_colors=32)`
- An all-sky frame, by contrast: `exposed_cells=0 non_sky_fraction=0.000000 distinct_colors=4`,
  exit non-zero. The two ends of the instrument now genuinely differ.
- Cycles internal: **1.77 s / 1.66 s**. Whole `blender --background` process: **4.68 s / 4.54 s**.
- Pixel determinism: **0 of 2,073,600 RGBA values differ** across two full-scale runs.
- Export: `tick: 21 entities: 10 dims: {'x': 128, 'y': 128, 'z': 32}`.

**Tests (AC 7, 8, 13) — 7 ran, 0 SKIPPED** (`python3 -m unittest discover -s scripts/tests -v`):
`test_exposed_faces_use_six_orthogonal_neighbours_and_world_edges`,
`test_geometry_summary_changes_when_world_content_changes`,
`test_floor_functions_reject_empty_geometry_and_accept_visible_frame`,
`test_pixel_figures_change_between_empty_and_one_cell_frames`,
`test_boot_projection_matches_the_client_composition`,
`test_empty_export_exits_nonzero`,
`test_all_sky_frame_reads_as_sky_in_a_real_render`.
The last two spawn Blender; both ran here rather than skipping.

**Mutations (AC 12) — 7 rows, ALL KILLED, zero APPLY-FAILED**, re-run independently by the
orchestrator. Each row was checked for *which* assertion kills it, not merely that it says KILLED:
| row | killing assertion |
| --- | --- |
| palette drift | `bench_contract.rs:74` — the client-literal assert |
| boot camera drift | `bench_contract.rs:111` — the camera-literal assert |
| missing range assertion | exit-code assert, `0 == 0` |
| inverted neighbour predicate | `{'faces': 12} != {'faces': 10}` — the geometry quantity |
| zero exit | exit-code assert, `0 == 0` |
| z-up camera basis | `0.352 != 0.48 within 0.03 delta` — the framing assert |
| linear sky reference | real render prints `non_sky_fraction=1.000000` — reproduces the defect |

**Gate (AC 1):** FULL tier run by the orchestrator, GATE GREEN, including the new `bench tests`
line. `python3 scripts/audit-mutations.py` clean.

**Known limitation, reported not worked around (AC 15's client half):** the
`gui --headless --capture` command exits **101** on a devpod. `capture.rs`'s delivered-tick floor
is unreachable under llvmpipe, where tick delivery is erratic — measured 26 ticks at `--frames`
1,500 and 9,000, then 2 ticks at `--at-tick 20 --frames 200000`. The PNG is written *before*
validation, so the committed frame is genuine and complete; only the integrity assertion fails.
Story 9.1 already recorded that every `--capture` AC is vehicle-bound on a devpod. The assertion
was deliberately left alone: weakening a client-side integrity rule to make a bench story's
artifact green would ship a workaround for an environment limitation.

**Self-gate: NOT RUN — this is a coverage hole the review should know about.** Dev run 1 skipped
it; the continuation run that would have carried it was killed by the harness at its commit step,
and its authored work was recovered by hand rather than relaunched (a restart bills full quota and
buys nothing). No `codex review` conclusion exists for this story.

### Completion Notes List

- Promoted the wire snapshot exporter and added a Cycles CPU bench: exposed-only mesh geometry
  from the export file alone, the client's palette/lights/camera, a range check that prints before
  it asserts, and gate-integrated tests.
- Fixed two defects found in verification: a 110-degree camera roll that made the artifact fail to
  predict the build, and a colour-space error that made the pixel half of the range check inert on
  real renders. Both now carry a test and a sabotage row.
- Collapsed the two camera implementations into one so the framing test guards the code that
  actually renders, rather than a parallel copy of it.
- AC15 (Wolf's judgement of whether the bench predicts the build) remains **open** — no agent can
  close it. The pair is committed with every known difference written down.

### File List

- scripts/bench/export_world.py
- scripts/bench/valley_bench.py
- scripts/tests/test_valley_bench.py
- crates/gui/tests/bench_contract.rs
- scripts/gate.sh
- scripts/mutate.sh
- scripts/audit-mutations.py
- _bmad-output/implementation-artifacts/mutations/10-1-the-headless-bench.sh
- _bmad-output/implementation-artifacts/10-1-signoff/bench-valley.png
- _bmad-output/implementation-artifacts/10-1-signoff/gui-capture.png
- _bmad-output/implementation-artifacts/10-1-signoff/what-you-will-see.md
- docs/tech-art-guidelines.md
- _bmad-output/implementation-artifacts/10-1-the-headless-bench.md
- _bmad-output/implementation-artifacts/metrics/.session-cursors.json
- _bmad-output/implementation-artifacts/metrics/10-1-the-headless-bench.md
- _bmad-output/implementation-artifacts/sprint-status.yaml

### Review Patch Record — 2026-08-29

**All 15 patch findings applied; all 3 decision-needed resolved by Wolf as option (a).** Full gate
GREEN (run, not claimed). Mutation table 7 -> 14 rows, **14/14 KILLED, zero APPLY-FAILED, zero
NOT-RUN, zero BAD-TIER**. Bench tests **7 -> 17, none skipped**. `audit-mutations.py` clean (422
rows).

**The three defects the review found were one shape, and it is this story's own shape.** 10.1
already shipped two fixes for "the constant is right and the code consuming it is wrong or
absent". The review found three more of exactly that:

1. **`AMBIENT_RGB` was a dead constant.** Defined at `valley_bench.py:45`, pinned by the AC9 drift
   guard, and read by nothing. `setup_scene` built a sun, point lights and a sky-coloured world
   background but never an ambient fill, while the client applies
   `AmbientLight { .., brightness: 4_500.0 }` [ingest.rs:714-718]. Measured over non-sky pixels of
   the committed pair, independently by two review layers and again by the orchestrator: bench
   mean luma **68.2** against the client's **90.3**. Task 3 ticked the sub-requirement `[x]`.
   *Fix:* Cycles has no ambient-light object, so the world background now mixes on `Is Camera Ray`
   — flat `SKY_RGB` to the camera, `AMBIENT_RGB` to every other ray.
2. **The exit-0 guard was applied to one call site, not to `main()`.** Only `assert_range`'s
   `AssertionError` reached the shell. Malformed JSON, an unknown entity kind and a `dims`/`tiles`
   mismatch each printed a traceback under `blender --background` and **exited 0**. Reproduced
   three ways by the Blind Hunter, and all three are now a test.
3. **FOV and aspect were a parallel copy.** `project_boot_point` re-declared `math.pi / 4` and
   `16.0 / 9.0` while the renderer set `camera_data.angle` and `resolution_x/y` independently.
   Demonstrated: widening only the render FOV to `pi/3` moved **1,050,234 of 2,073,600 pixels**
   while the framing test still read `camp=(0.500, 0.779)`, inside tolerance, and the range check
   still exited 0. The camera comment claimed a change "cannot leave that test green against a
   frame it no longer matches" — true for the basis it had just fixed, false for the projection.

**Wolf's three calls, all option (a):**
- **AC7 reworded** and given a real test. The AC asked for pixel figures differing between a
  one-cell world and an empty one; measured, those two render **pixel-identical** at the fixed
  boot camera because the cell is off-frame. The delivered test had substituted two hand-built
  pixel lists and rendered nothing. Now a populated world vs an empty one, through Blender.
- **`foliage_scale` applied** [project.rs:868-892]: foliage draws at 0.62/0.78/0.95 of its cell,
  so crowns read as sparse branches rather than solid slabs. 10.4 is judged on this bench, and it
  was mis-drawing the geometry 10.4 changes. Scale moves only where a face is DRAWN, never which
  faces are exposed — cell and face counts are unchanged.
- **The sun is aimed the way the client aims it.** `aurora_core()` is ported [atmosphere.rs:67-71];
  `sun_direction()` computes **(0.761, 0.112, 0.639)**, matching the Feature Auditor's independent
  derivation exactly. The replaced hand-picked euler pointed **122 degrees** away.

**Exposure is CALIBRATED, not converted, and not tuned by eye.** Bevy's `brightness: 4_500` /
22,000 lux and Cycles' background strength / sun energy share no units, and the bench omits the
aurora, which is a real light source in the client. Both scalars were fitted to one objective
target — mean Rec.709 luma over the bottom 65% of frame, terrain-dominated at the boot framing and
free of the aurora that contaminates a whole-frame average:

| | client `gui-capture.png` | bench, before | bench, after |
| --- | --- | --- | --- |
| mean luma, bottom 65% | **105.7** | 65.0 | **103.6** |
| mean RGB | (87, 108, 138) | (49, 68, 78) | (81, 106, 147) |

**A new figure in the range check: `terrain_luma`** — the one that would have caught the dead
ambient, which no existing floor could, because a dark frame is neither empty nor monochrome. On
the populated test world it reads **124.3 wired against 0.456 unwired**.

**Instrument, re-measured on the patched bench (AC 4, 6, 10):**
- `exposed cells: 44984 faces: 61142` — unchanged, and still pinned by nothing.
- `range-check: exposed_cells=44984 non_sky_fraction=0.686815 distinct_colors=58993 terrain_luma=106.260 floors(non_sky_fraction=0.020000, distinct_colors=32, terrain_luma=20.000)`
- Cycles internal **1.42 s / 1.41 s**; whole `blender --background` process **4.34 s / 4.35 s**.
- Pixel determinism: **0 of 2,073,600 RGBA values differ**. PNG bytes still differ (tEXt).

**VENUE RE-BASELINED 2026-08-31 — every figure above is the Blender 4.3.2 era.** The devpod now
carries Blender **5.2.1 LTS** at `/opt/blender-5.2` to match the vehicle, and bare `blender`
resolves to it; the apt 4.3.2 stays reachable at `/usr/bin/blender`. Measured on the same
snapshot, both versions, same session:

| | 4.3.2 | 5.2.1 |
| --- | --- | --- |
| `exposed_cells` / `faces` | 44984 / 61142 | 44984 / 61142 — identical, geometry is not rendered |
| `non_sky_fraction` | 0.686815 | 0.686736 |
| `distinct_colors` | 58993 | 59191 |
| `terrain_luma` | 106.260 | 105.853 |
| whole process | 4.42 s | 4.55 s |

**Determinism survived the move, which is the property that mattered:** 0 of 518,400 pixels differ
between two runs of 4.3.2, and 0 between two runs of 5.2.1. Across versions 335,501 pixels differ
(64.72%) with a worst channel delta of **2/255** and a mean of 0.80 — a sub-1% numerical nudge, not
a visible change. Nothing pinned the old numbers: the bench asserts FLOORS by design, and 5.2.1
clears all three. **The current baseline is therefore
`blender=5.2.1 exposed_cells=44984 non_sky_fraction=0.686736 distinct_colors=59191 terrain_luma=105.853`**,
and the line now names its own venue (see below) so a recorded figure can never again be read
against the wrong Blender.

**Harness: three false verdicts, all reproduced before being fixed.**
- A py row whose test SKIPPED (Blender absent) exited 0, missed every guard and landed in
  **SURVIVED** — "your test is not pinning what it claims", when the test never ran. Now `NOT-RUN`.
- A **mistyped tier** fell through to `cargo test -p <typo>`, which exits 101 with "did not match
  any packages" — not "could not compile" — and printed **KILLED** having run nothing. Tiers are
  now validated against the workspace package list; `Py`, `""` and `" gui"` all return `BAD-TIER`.
- `gate.sh` printed `bench tests  ok` with 2 of 7 tests skipped. It now prints
  `ok — WITH SKIPS (coverage hole)`, verified with Blender genuinely off `PATH`.
- Backup set widened again to `_bmad/scripts/*`; `audit-mutations.py` no longer prints Rust
  wording for py-tier orphans.

**Mutation rows — 14/14 KILLED, and each checked for WHICH assertion kills it:**

| row | killing assertion |
| --- | --- |
| palette drift | `bench_contract.rs:24` — anchor matched 0 |
| boot camera drift | `bench_contract.rs:24` — anchor matched 0 |
| missing range assertion | exit-code assert, `0 == 0` |
| inverted neighbour predicate | `{'faces': 12} != {'faces': 10}` |
| zero exit | exit-code assert, `0 == 0` |
| z-up camera basis | `0.352 != 0.48 within 0.03 delta` |
| linear sky reference | real render prints `non_sky_fraction=1.000000` |
| **unwired ambient** | real render EXITS NON-ZERO — the `terrain_luma` floor rejects it |
| **dwarf colour drift** | `bench_contract.rs:24` — arm anchor matched 0 |
| **torch/campfire swapped** | `bench_contract.rs:24` — arm anchor matched 0 |
| **full-size foliage** | `1.0 != 0.62 within 6 places` |
| **re-copied FOV literal** | `projection ignores BOOT_VERTICAL_FOV` |
| **hand-picked sun aim** | `key light points downward: (0.044, -0.637, -0.77)` |
| **swallowed exception** | exit-code assert `0 == 0`, on all three broken exports |

**One row was re-mutated after being strengthened, and it mattered.** The sun-aim row first died on
`1.000605 != 1.0` — the *normalisation* assert, because a hand-written vector is never exactly unit
length. It reported KILLED while pinning nothing about direction, and looked identical from outside
to a row that worked. The aim assertions were moved ahead of the normalisation and the row re-run:
it now dies on `key light points downward`. This is the third time this project has hit "KILLED
names the test, not the assertion".

**Metrics defect closed.** The ledger had zero `dev | claude` rows and session `2870b2e6` was
absent from `.session-cursors.json` entirely — neither rowed nor marked — while that window
produced the verification and both original defect fixes. Recorded: **226 turns, $22.36**. 10.1's
dev cost is **$24.77, not $2.41**.

**STILL OPEN — AC15, and its "known differences" clause is now met but its judgement is not.**
The signoff pair was **re-rendered**: the previous `bench-valley.png` was made unlit, with the sun
122 degrees off and full-size foliage, so it is not the picture to judge. `what-you-will-see.md`
was rewritten and now names every difference the review found — the aurora and stars, the camp
pool's much larger blown core (the client drives its campfire at 25M lm against the bench's Cycles
1,500; only light *colours* are pinned, intensities are a deferred calibration question), Cycles
grain at 32 samples, the snow-cap slab geometry, ramps drawn as cubes, and the exposure
calibration — ordered by how much of the frame each moves. **Whether the bench artifact predicts
the build remains Wolf's call and no agent can close it.**

**Review cost, and one honest caveat about how it is booked.** 513 turns, $50.94, 96.1% of every
token processed a cache read, 4 subagent transcripts = 38.8% of the session. **This single `review`
row covers the patch pass too**, because both ran in one session and the delta cursor was never
marked at the boundary — so the split between reviewing and repairing is not recoverable from the
ledger. Next time, run `session_tokens.py --phase review` at the moment triage is written, before
the first patch, so `review` and `review-patch` land as two rows. Build caches reaped after triage:
**38.1 GB across 14 directories** under /tmp.

**AC15 CLOSED BY WOLF — 2026-08-30, on the re-rendered pair.** His judgement, verbatim: *"bench
valley looks decent, better lighting than in gui-capture but that's ok. And it's a bit dithered but
that's not big deal either. Geometrically it's correct and overall looks the same."* The bench
artifact **predicts the build** — which is this story's real bar and the one thing no agent could
close. Both differences he named were already on the difference list and are accounted for: the
brightness is the un-pinned point-light intensity (client campfire 25M lm against the bench's Cycles
1,500 — terrain luma itself is calibrated to 103.6 vs 105.7), and the dither is Cycles sampling
noise at 32 samples with denoising necessarily off. Neither is a defect; both stay documented.

**THE BENCH IMMEDIATELY EARNED ITS KEEP, and its first finding is about the CLIENT.** Wolf,
2026-08-30, on the same pair: *"actually the bench looks more like what we are targeting"* (noise
aside). Not the terrain — that is calibrated to match. The camp pool: client campfire 25M lm blows
to flat white, bench Cycles 1,500 keeps detail. This is 6.2's carried-open "camp is too blown out",
and the 2026-08-22 ruling that closed it treated the **peak** and states in its own comment that
"this still frame never moved" [appearance.rs:66-76] — so the still-frame case was ruled out of
scope, not ruled acceptable. Logged to `deferred-work.md` as an input for **10.4**; no light
constant was touched here on the strength of one observation.

**Coverage holes carried into this record, not resolved by it:** `codex review --base main` still
never ran on this story. The `gui --headless --capture` exit-101 limitation is unchanged and the
client half of the pair was not re-rendered, correctly — the review changed no crate behaviour,
only a test, proved against this story's own commit range.

## Change Log

| date | change |
| --- | --- |
| 2026-08-29 | Story created. Epic 10 opens. **AC1's workload re-measurement was executed at creation rather than deferred**: 44,984 exposed cells, Cycles 2.01 s / process 3.67 s, 0 of 2,073,600 pixel values differing across two full-scale runs — so the epic's gingerspice fallback does not fire and the devpod stays the venue. Three epic premises checked (one false: `bevy_gltf` IS enabled, which corrects 10.5). The export file was found to exist already and is promoted, not rebuilt. Baseline `212fbcd`, full gate GREEN at creation (run, not claimed); stacks on the 9.4 tip because `main` still carries the superseded foliage colour. Revised after an adversarial checklist review that found five criticals — the emptiness metric was inert (a non-black floor cannot detect an empty frame once the sky is `(5,12,28)`), AC8's exit-code check was unbuildable against the bpy-free test rule, the new `py` mutation tier reopened the false-KILL class, the drift guard's placement was justified circularly, and Task 2 pinned a moving content measurement inside the epic that will move it. |
| 2026-08-29 | Implemented the headless bench, tests, mutation coverage, and signoff artifacts. |
| 2026-08-29 | Dev run 1 (Codex `gpt-5.6-terra`, 9 commits): export, geometry, look, range check, tests, mutation table, signoff pair. Handed back three honest caveats rather than claiming success. |
| 2026-08-29 | Orchestrator verification found two defects a green gate could not see, both fixed with tests and sabotage rows: the bench camera was **rolled 110 degrees** so the artifact did not predict the build (AC15's failure mode; AC9's text-scrape guard stayed green because the constants were right and the maths was wrong), and the range check's pixel half was **inert on real renders** because it compared a display-referred readback against a linear reference, scoring a 100%-sky frame at `non_sky_fraction=1.000000`. Mutation table 5 rows -> **7, all KILLED**, each verified to kill on the intended assertion. Full gate GREEN, 7 bench tests, 0 skipped. Re-measured: Cycles 1.77 s / 1.66 s, process 4.68 s / 4.54 s, 0 of 2,073,600 pixels differ. Self-gate NOT run — a named coverage hole. AC15 remains open for Wolf. |
| 2026-08-29 | **Code review (4 layers, zero coverage holes) + patch pass.** 15 patch findings and 3 decisions, all applied. Three defects of the story's own signature shape: `AMBIENT_RGB` was a dead constant (frame ~24% dark), the exit-0 guard covered one call site so malformed exports reported success, and FOV/aspect were a parallel copy (a pi/3 FOV moved 1,050,234 pixels with the framing test green). Wolf took option (a) on all three decisions: AC7 reworded and given a real populated-vs-empty render, `foliage_scale` applied, and the sun aimed by the client's `aurora_light_transform()`. Exposure calibrated against the client capture: 103.6 vs 105.7 mean luma. New `terrain_luma` figure in the range check. Harness stopped reporting three false verdicts (skip-as-SURVIVED, typo-as-KILLED, gate `ok` over skips). Mutations 7 -> **14, all KILLED**, one re-mutated after an earlier assert absorbed it. Tests 7 -> **17, none skipped**. Full gate GREEN. Orchestrator's unbilled dev window recorded: dev cost $2.41 -> **$24.77**. Signoff pair re-rendered; **AC15 still open for Wolf**. |
| 2026-08-30 | **AC15 CLOSED by Wolf on the re-rendered pair** — "geometrically it's correct and overall looks the same". The bench predicts the build. Story DONE. Render re-measured: Cycles **1.42 s / 1.41 s**, whole process **4.34 s / 4.35 s** — faster than before the patches, the shrunken foliage lets more rays escape to sky. Pushed and PR opened. |
