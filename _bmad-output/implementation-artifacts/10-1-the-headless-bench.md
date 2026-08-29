---
model: claude-opus-5[1m]  # policy default (Opus); recorded per the model policy so the ledger row is readable
baseline_commit: 212fbcdc3caa0bf2daba821fe1598df2c1fdbf38
---

# Story 10.1: The Headless Bench

Status: review

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
   between a one-cell world and an empty one. A test that only asserts the bench ran satisfies
   neither. *Both halves of the instrument must visibly differ when what they report on differs.*
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

## Change Log

| date | change |
| --- | --- |
| 2026-08-29 | Story created. Epic 10 opens. **AC1's workload re-measurement was executed at creation rather than deferred**: 44,984 exposed cells, Cycles 2.01 s / process 3.67 s, 0 of 2,073,600 pixel values differing across two full-scale runs — so the epic's gingerspice fallback does not fire and the devpod stays the venue. Three epic premises checked (one false: `bevy_gltf` IS enabled, which corrects 10.5). The export file was found to exist already and is promoted, not rebuilt. Baseline `212fbcd`, full gate GREEN at creation (run, not claimed); stacks on the 9.4 tip because `main` still carries the superseded foliage colour. Revised after an adversarial checklist review that found five criticals — the emptiness metric was inert (a non-black floor cannot detect an empty frame once the sky is `(5,12,28)`), AC8's exit-code check was unbuildable against the bpy-free test rule, the new `py` mutation tier reopened the false-KILL class, the drift guard's placement was justified circularly, and Task 2 pinned a moving content measurement inside the epic that will move it. |
| 2026-08-29 | Implemented the headless bench, tests, mutation coverage, and signoff artifacts. |
| 2026-08-29 | Dev run 1 (Codex `gpt-5.6-terra`, 9 commits): export, geometry, look, range check, tests, mutation table, signoff pair. Handed back three honest caveats rather than claiming success. |
| 2026-08-29 | Orchestrator verification found two defects a green gate could not see, both fixed with tests and sabotage rows: the bench camera was **rolled 110 degrees** so the artifact did not predict the build (AC15's failure mode; AC9's text-scrape guard stayed green because the constants were right and the maths was wrong), and the range check's pixel half was **inert on real renders** because it compared a display-referred readback against a linear reference, scoring a 100%-sky frame at `non_sky_fraction=1.000000`. Mutation table 5 rows -> **7, all KILLED**, each verified to kill on the intended assertion. Full gate GREEN, 7 bench tests, 0 skipped. Re-measured: Cycles 1.77 s / 1.66 s, process 4.68 s / 4.54 s, 0 of 2,073,600 pixels differ. Self-gate NOT run — a named coverage hole. AC15 remains open for Wolf. |
