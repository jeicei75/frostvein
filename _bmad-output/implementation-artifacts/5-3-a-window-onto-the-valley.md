---
baseline_commit: 5b2e834
model: claude-opus-5[1m]  # default Opus; 1M-context variant, as at 5.1 and 5.2
---

# Story 5.3: A Window Onto the Valley

Status: in-progress

## Story

As the boss,
I want a Bevy client that opens a window and shows the real world as voxels I can orbit,
so that the render path is proven to work before anything beautiful is built on it.

**This story is allowed to be ugly.** Unlit grey boxes that orbit at speed are a pass; the
visual bars belong to 5.4. The sign-off gate (UX-DR22) binds 5.4, explicitly not this story.

## Environment: read this before anything else

**The premise the spine recorded for this story is false as written, and the correction was
made on the record at story-creation (2026-08-11).** The spine says *"Runtime topology: the
WSL2 devpod with `gui` displaying via WSLg (D3D12 passthrough, RTX 4080 Laptop, Mesa
25.3.5)"*. That describes **WSL2**, not the devpod. Measured inside the Nidavellir devpod at
story-creation:

| Probe | Result |
| --- | --- |
| kernel | `6.18.33.2-microsoft-standard-WSL2` — the host is WSL2 |
| `/dev` | `core fd full mqueue null ptmx pts random shm std* tty urandom zero` — minimal container `/dev` |
| `/dev/dri`, `/dev/dxg` | absent |
| `/mnt/wslg`, `/usr/lib/wsl` | absent |
| `DISPLAY`, `WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR` | unset |
| `/workspace` | `/dev/sdd`, an ext4 docker volume — not a host bind mount |
| graphics userspace | `libvulkan.so.1`, `libGL.so.1`, `libEGL.so.1` all absent; no Vulkan ICDs; no Mesa DRI drivers (Debian 13 trixie) |

Wolf confirmed the frostvein devpod is the same image and setup. **No devpod as currently
configured can open a window**, and the GPU is one layer up rather than absent.

**Wolf's decision, 2026-08-11: prove the envelope on `rebelspice` now** — a 2016 MacBook Pro
running Fedora 44 with an AMD (Polaris) GPU, native Linux, where container GPU/display
passthrough is `--device /dev/dri` plus a display socket rather than WSL2's D3D12 gymnastics.
Mesa's RADV is a mature conformant Vulkan driver, so the spine's recorded risk (*"wgpu prefers
Vulkan via WSLg's Dozen driver, younger and less conformant"*) does not apply on that path.

**The consequence for NFR6, which must not be blurred:** NFR6's bar is defined against *the
WSLg devpod with the RTX 4080*. A figure measured on rebelspice proves the **envelope**, not
the bar. Record it labelled with its machine and mark the WSLg figure as still owed — see AC16.

## Acceptance Criteria

### The crate and the graph

1. `crates/gui` exists as a workspace-member binary whose **normal** dependencies are exactly
   `protocol`, `client-core`, `anyhow` and `bevy`. It carries `#![forbid(unsafe_code)]` and has
   no `sim-core` edge.
2. The workspace dependency graph is exactly the M2 closed set, read off the six `Cargo.toml`
   files: `simd → sim-core`, `simd → protocol`, `client-core → protocol`, `tui → protocol`,
   `tui → client-core`, `gui → protocol`, `gui → client-core`. No other edge exists.
3. `scripts/gate.sh` probes that `gui` has no `sim-core` edge. This is the **third** such probe,
   which is the point 5.2 named for collapsing them: the three inverted `cargo tree` blocks
   become one loop over `tui client-core gui`. A match is still the failure.
4. `bevy` is pinned at `0.19.0` and `Cargo.lock` contains exactly **one** `bevy_ecs`, at
   `0.19.0` — the spine's never-two-Bevy-versions convention, asserted rather than assumed.

### The build must not need system libraries the devpods lack

5. `bevy` is declared `default-features = false` with an explicit feature list. Dropping
   `audio`, `bevy_gilrs` and `wayland` is required, not preference: with default features the
   build fails in this devpod on missing `wayland-client`, and `alsa-sys` and `libudev-sys` are
   fetched to fail behind it. The story records the feature list and the one-sentence
   justification the closed-stack rule requires.
6. `cargo build -p gui` succeeds in a devpod with **no** graphics system libraries installed —
   the build is not allowed to depend on machine-local `apt install` state, because the fix must
   survive a fresh clone and both devpods.

### The envelope (the live half)

7. On a machine with a reachable display, `gui` opens a window and renders continuously.
8. The story records, from the running binary rather than from inference: **which machine**
   (rebelspice or the WSLg devpod), **which wgpu backend** initialised, and the adapter name,
   device type and driver string.
9. If the envelope does not hold, that is this story's finding: it is reported with the actual
   error, and **never worked around in production code**. The ladder, in order — force
   `WGPU_BACKEND=gl`, then `vulkan`; then report. Neither a software-rasteriser fallback nor a
   sandbox workaround may enter `crates/gui`.

### Lifecycle: one mirror, two clients, live

10. `gui` connects to `simd`, builds a `client_core::Mirror` from the snapshot and applies each
    delta to it. Wire messages mutate **only** the mirror; no ingestion code touches the Bevy
    world.
11. A `tui` client attached to the same daemon at the same time shows the same world — the same
    tick, the same dwarf count, the same terrain figures.

### Projection (AD-14)

12. Every render entity is exactly one of two classes: **world-projected** (terrain, dwarves,
    items, emitters) or **client-local** (camera rig, overlay, lights). The class is structural —
    a marker component, not a naming convention — so a query can assert the partition.
13. Terrain renders as one cube per **exposed** solid tile: a solid tile with at least one of its
    six neighbours empty or out of bounds. On the shipped seed this is **53,365 cubes of 315,068
    solid tiles**; drawing all solid tiles is not a viable starting point and drawing only
    top faces (20,788) is not correct. Dwarves, items and emitters render as cubes too — this
    story is allowed to be ugly.
14. Reconciliation systems keyed by sim `Id` are the **only** place world-projected entities are
    created or despawned.
15. Despawning every world-projected entity and re-projecting reproduces the same scene,
    asserted headlessly under `MinimalPlugins` in `cargo test` with no GPU.

### The seam 5.2 left inert (AD-18, and the trap it recorded)

16. A delta carrying a dirty tile updates **only** that tile's cube — driven by
    `Mirror::changes()`, so the seam's output is consumed rather than recomputed from scratch.
17. **The branch-changing negative.** A snapshot arriving over a populated world rebuilds the
    terrain in full **even though `changes()` reports empty after `apply_snapshot`**. A test
    asserts a non-zero cube count after the reset snapshot, and it must **fail** if the
    reconciler is driven naively by `changes()` alone. This is the exact defect 5.2's review
    recorded and deferred to this story: an empty Bevy window beside a correct TUI, on every
    reconnect and every `Load`.
18. The mirror's out-of-bounds dirty tiles are skipped *and* omitted from `changes().tiles`
    (`client-core/src/lib.rs:68-73`). The terrain path must not diverge from a full repaint
    because of it; a test pins the two agreeing.

### The coordinate transform (spine convention)

19. Exactly one transform pair, `world_to_render` / `render_to_world`, exists in `gui` for sim
    z-up `[x,y,z]` ↔ Bevy Y-up. Projection, camera and capture all call it; no system does its
    own axis math.
20. The pair is pinned by **two** assertions, because a round-trip test alone cannot catch a
    mirrored world — `(x,y,z) → (x,z,y)` and `(x,y,z) → (x,z,-y)` both round-trip perfectly and
    only one preserves handedness. So: a round-trip property test, **and** a literal-oracle
    assertion on one asymmetric point whose expected value is written out by hand.

### The camera

21. The camera looks down into the world isometrically from outside. Orbit and zoom are
    continuous, every angle is reachable, and the fortress cannot be lost — asserted headlessly
    on the camera logic, not by eye.

### The NFR6 instrument

22. A frame-time overlay is readable on screen. The story states whether it uses
    `bevy::dev_tools::fps_overlay::FpsOverlayPlugin` (needs the non-default `bevy_dev_tools`
    feature) or a hand-rolled overlay.
23. The overlay is toggleable and **off by default in `--capture` output** — a burnt-in counter
    both spoils 5.4's sign-off artifact and gives the capture's "changes when the world changes"
    self-test a false positive for the wrong reason.
24. The measured figure is recorded **with the machine it was measured on**. A rebelspice figure
    is labelled as the envelope baseline and explicitly **not** NFR6's bar; the WSLg figure stays
    owed, and 5.4 inherits that debt rather than discovering it.

### The capture instrument (AD-17 rung 3)

25. `gui --capture <path> --frames N` writes an image file and exits. Its own tests assert the
    file exists, is **not black**, **changes when the world changes**, and range-check what they
    came to see — a non-zero count of non-background pixels. **Exit 0 is not a result.**
26. The capture self-tests need a real render surface and are therefore excluded from
    `scripts/gate.sh` and default `cargo test`, and are separately invoked. The gate stays
    headless and stays green on a machine with no GPU.

### Evidence

27. A sabotage table exists at
    `_bmad-output/implementation-artifacts/mutations/5-3-a-window-onto-the-valley.sh` and every
    mutation in it is KILLED, with the RED output pasted verbatim into the Dev Agent Record.

## Tasks / Subtasks

- [ ] **Task 0 — Reach a display (Wolf's, not the dev agent's)** (AC: 7, 8)
  - [ ] Bring up a frostvein devpod on `rebelspice` with GPU and display passthrough. Starting
        recipe, **unverified — this is the story's first real unknown**:
        `--device /dev/dri`, `-v /tmp/.X11-unix:/tmp/.X11-unix`, `-e DISPLAY=$DISPLAY`, and the
        user added to the `video`/`render` groups. Under a Wayland session, XWayland satisfies
        the `x11` winit backend, which is why AC5 can drop the `wayland` feature.
  - [ ] **Passing the device through is not enough — the image has no graphics userspace at
        all.** Measured in the Nidavellir devpod (Debian 13 trixie) at story-creation:
        `libvulkan.so.1`, `libGL.so.1` and `libEGL.so.1` are all absent, `/usr/share/vulkan/icd.d/`
        is empty and there are no Mesa DRI drivers. With `/dev/dri` present and this userspace
        missing, wgpu enumerates **zero adapters** and the failure looks like a broken renderer
        rather than a missing package. Install inside the container:
        ```bash
        sudo apt-get update && sudo apt-get install -y \
          libvulkan1 mesa-vulkan-drivers vulkan-tools \
          libgl1 libglx-mesa0 mesa-utils \
          libxkbcommon0 libx11-6 libxcursor1 libxrandr2 libxi6
        ```
        `mesa-vulkan-drivers` carries RADV, which is the driver rebelspice's Polaris GPU needs.
        `vulkan-tools`/`mesa-utils` are there to *diagnose* (`vulkaninfo --summary`, `glxinfo -B`)
        — run them before `gui`, so a zero-adapter environment is identified as such.
  - [ ] **These are runtime libraries and they do not change AC6.** The build needs no system
        libraries and must keep needing none; if someone "fixes" a build error by installing a
        `-dev` package, the feature trim has been undone and the fix will not survive a fresh
        clone.
  - [ ] Confirm before starting the live half: `ls /dev/dri` is non-empty, `echo $DISPLAY`
        is set, and `vulkaninfo --summary` lists at least one device. If they are not, Tasks 1–7
        still complete in any devpod; Task 8 is what waits.
  - [ ] If the bring-up fails, that is a finding about the environment and not about `gui`.
        Record it and stop at the headless boundary — do not fake the live evidence.

- [ ] **Task 1 — The crate, the feature trim, the gate loop** (AC: 1, 2, 3, 4, 5, 6)
  - [ ] `crates/gui/Cargo.toml`: `protocol`, `client-core`, `anyhow.workspace = true`, and
        `bevy` with `default-features = false`. Add `bevy = "0.19.0"` to root
        `[workspace.dependencies]`.
  - [ ] The feature list, **verified to compile in this devpod at story-creation** (449 crates,
        1m02s, exit 0):
        ```toml
        bevy = { version = "0.19.0", default-features = false, features = [
          "std", "default_app", "3d_bevy_render", "ui_api", "ui_bevy_render",
          "default_font", "bevy_winit", "multi_threaded", "x11", "bevy_dev_tools",
        ] }
        ```
        **Justification for the closed stack:** the spine names `bevy` 0.19.0 already and permits
        trimming "on a measured problem" — this is one. Default features fail to build here (see
        Key decisions); `bevy_dev_tools` is the non-default feature the spine's own NFR6 row
        names for the ready-made overlay.
  - [ ] Collapse the three inverted `cargo tree` probes in `scripts/gate.sh:73-105` into one loop
        over `tui client-core gui`. 5.2's Task 1 named this story as the point where the loop
        earns itself. Keep the inversion: a match is the failure. Widen the label column while
        you are there — the existing `printf '  %-28s'` is too narrow for the 32-character
        `client-core has no sim-core edge`, which currently prints as `...edgeok`, and `gui`'s
        label will collide the same way.
  - [ ] Assert `rg -c '^name = "bevy_ecs"$' Cargo.lock` is 1 and its version is `0.19.0`.

- [ ] **Task 2 — Connect and ingest** (AC: 10, 11)
  - [ ] Port the connection shape from `tui/src/main.rs:157-165` and its reader thread
        (`:220-224`): `TcpStream`, `SNAPSHOT_READ_TIMEOUT`, `MAX_SNAPSHOT_BYTES`,
        `read_message`, an `mpsc::sync_channel`. **Networking lives in `gui`, exactly as it lives
        in `tui`** — `client-core` has zero I/O and gaining one is an AD-13 breach.
  - [ ] The mirror lives in a Bevy `Resource`. A system drains the channel and calls
        `apply_snapshot` / `apply_delta`. **That system is the only code in `gui` that touches
        `protocol` message types.**
  - [ ] Test: with the mirror resource driven by recorded wire literals, no ECS mutation happens
        outside the reconciliation systems — the AD-14 negative.

- [ ] **Task 3 — The one transform pair** (AC: 19, 20)
  - [ ] `world_to_render(pos: [i32; 3]) -> Vec3` and `render_to_world(v: Vec3) -> [i32; 3]`.
  - [ ] Round-trip property test over a spread of coordinates, **plus** the literal oracle:
        pick one asymmetric point, write its expected `Vec3` out by hand with a `// NOTE:` saying
        it is a literal precisely so a handedness flip cannot survive it.
  - [ ] `rg 'Vec3::new' crates/gui/src/` must show axis construction only inside this module.

- [ ] **Task 4 — Projection and reconciliation** (AC: 12, 13, 14, 15, 16, 17, 18)
  - [ ] Two marker components, `WorldProjected(Id)` and `ClientLocal`, so AC12's partition is a
        query and not a convention.
  - [ ] Terrain: the exposed-tile predicate, then one cube per exposed solid tile. Share one
        `Mesh3d` handle and one `MeshMaterial3d` per material so Bevy batches the draw; 8
        materials appear on the shipped seed.
  - [ ] Entities: reconcile dwarves/items/emitters from `mirror.entities()` / `items()` against
        the projected set keyed by sim `Id` — spawn missing, despawn absent, update transforms.
        At ~15 entities a full pass per frame is correct and cheap; do not optimise it.
  - [ ] Terrain updates take the other route: `changes().tiles` on a delta (AC16), **full rebuild
        on a snapshot** (AC17). Write the snapshot test first and watch it fail before the
        rebuild path exists — that RED is the evidence AC17 asks for.
  - [ ] The despawn-all-and-re-project test (AC15) under `MinimalPlugins`, comparing the
        resulting entity set, not a screenshot.

- [ ] **Task 5 — The camera** (AC: 21)
  - [ ] Isometric look-down orbit rig: yaw/pitch orbit around a focus point plus a zoom
        distance, pitch clamped away from the degenerate poles.
  - [ ] Headless tests on the rig's maths: every yaw is reachable, the pitch clamp holds at both
        ends, and the focus point stays inside the world bounds at every zoom — "never lose the
        fortress" as an assertion rather than a hope.

- [ ] **Task 6 — The frame-time overlay** (AC: 22, 23, 24)
  - [ ] `FpsOverlayPlugin` with `FpsOverlayConfig { enabled: false, ..default() }`, plus
        `FrameTimeDiagnosticsPlugin::default()`. Both compile-verified at story-creation.
  - [ ] A key toggles `config.enabled`; `--capture` forces it `false`. Test the forcing, not the
        key.
  - [ ] Record the figure with its machine (AC24). State plainly in the Dev Agent Record whether
        the WSLg number is still owed.

- [ ] **Task 7 — The capture instrument** (AC: 25, 26)
  - [ ] `gui --capture <path> --frames N`, mirroring `tui`'s scripted-flag discipline
        (`tui/src/main.rs:76-155` is the arg-parsing shape to copy).
  - [ ] Capture via `commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path))`;
        exit after N frames with `MessageWriter<AppExit>` and `AppExit::Success`.
        **`MessageWriter`, not `EventWriter`** — verified against 0.19 at story-creation.
  - [ ] Self-tests behind `#[ignore]` (or a separate `--test` target excluded from the gate), so
        `scripts/gate.sh` stays headless and green on a GPU-less devpod. Name the exact invocation
        in the Dev Agent Record.
  - [ ] The "changes when the world changes" test drives the same daemon at two different ticks
        and asserts the images differ. Range-check first: assert a non-zero count of
        non-background pixels **before** drawing any conclusion.

- [ ] **Task 8 — The live run** (AC: 7, 8, 9, 11, 24)
  - [ ] Run the recipe in Verification. Paste the backend/adapter line verbatim.
  - [ ] Run `tui` against the same daemon concurrently and record both outputs (AC11).
  - [ ] If the window does not open: record the error, walk the AC9 ladder, and report. A failed
        envelope is a legitimate outcome of this story.

- [ ] **Task 9 — Mutations, deferrals, gate** (AC: 27)
  - [ ] Write the sabotage table. Minimum set: the reconciler is driven by `changes()` alone (the
        snapshot rebuild dies); the exposed-tile predicate returns all solid tiles; the transform
        pair's handedness flips; reconciliation is keyed by something other than sim `Id`; the
        camera pitch clamp is removed.
  - [ ] Append a `## Deferred from: story 5.3` note recording that **`Mirror::previous_entity()`
        is still without a live caller** — AD-15 interpolation is deliberately deferred to 6.1
        (see Key decisions), so the seam stays inert by design for one more story rather than
        silently dropped.
  - [ ] **Record the gate's wall-clock before and after adding `bevy`.** Measured green at
        story-creation: **61 s warm**. Adding a 449-crate dependency to the workspace makes
        `cargo clippy --all-targets` and `cargo test` compile Bevy, and the gate runs on every
        commit via the pre-commit hook. The spine defers "trimming bevy features / build-time
        work" with the trigger *"a measured gate-time problem"* — this measurement is what arms
        that trigger, so record the number even though this story does not act on it.
  - [ ] `scripts/gate.sh` green. Branch `5-3-a-window-onto-the-valley`, small commits, imperative
        messages, author `Völundr <jeicei75@gmail.com>`. Push/PR only on Wolf's explicit yes.

## Dev Notes

### Scope guardrails — do NOT build these here

- **No lighting, no palette, no atmosphere.** Grey boxes pass. Sky, aurora, snowfall, warm/cold
  read, the `LightKind` → appearance table — all 5.4.
- **No interpolation.** See Key decisions: deliberately moved to 6.1, against 5.2's expectation.
- **No input beyond camera.** Picking is 8.1, designation is 8.2, z-slicing is 7.1, speed control
  is 8.3. `gui` issues **zero commands** in this story.
- **No designation or zone rendering.** AD-14 lists them as world-projected, but 5.3's AC names
  only terrain, dwarves, items and emitters; rendering designations and zones is **7.2**, and
  that placement is deliberate so it survives if Epic 8's input work is cut. The mirror already
  exposes `designations()` and `zones()` — leave them unread.
- **No change to `protocol`, `sim-core` or `simd`.** AD-16's sanctioned wire diff was spent in
  full at 5.1. This story is a pure consumer.
- **No change to `client-core`'s API.** The `changes()`-after-snapshot contract was ruled on by
  Wolf at 5.2's review; work with it, do not reopen it.
- **No TUI changes.** The parity rule's backward half does not fire — nothing sim-side changed.
- **No greedy meshing, no chunking, no LOD, no culling beyond the exposed-tile predicate.**
  Nothing has been profiled. AC13's predicate is the one concession, and it is there because
  315,068 is not a viable starting point, not because it was measured against an alternative.
- **No sign-off gate.** UX-DR22 binds 5.4.
- **No workaround for a failed envelope in production code** (AC9).

### What already exists (build on it, do not re-derive)

- **`client-core` is complete and needs nothing from this story.** `Mirror` exposes `dims()`,
  `tick()`, `speed()`, `tile(pos)`, `entities()`, `items()`, `designations()`, `zones()`,
  `previous_entity(id)`, `changes()`. Entities and items iterate in ascending id order.
- **`tui/src/main.rs` is the working reference for every non-Bevy concern** — arg parsing
  (`:76-155`), connect (`:157-165`), `read_message`, reader thread and channel (`:220-224`),
  the frame-loop shape (`:226-268`).
- **The wire's content on the shipped seed**, measured live at story-creation (`simd 7431`,
  snapshot at tick 30): dims 128×128×32, 524,288 tiles, 8 distinct materials, 5 dwarves
  (ids 0–4) and 5 emitters (ids 5–9). The camp is at **z 9**.
- **`scripts/gate.sh:73-105`** holds the two inverted probes to collapse into a loop.

### Key decisions & traps

- **Default bevy features cannot build here, and the first failure is not the one you would
  guess.** Verified at story-creation: `cargo check` on stock `bevy = "0.19.0"` fails on
  **`wayland-sys`** (`Package wayland-client was not found`), with `alsa-sys` and `libudev-sys`
  downloaded and queued to fail behind it. Bevy 0.19's `default` is `["2d","3d","ui","audio"]`,
  and `2d`/`3d`/`ui` each pull `default_platform`, which is what drags in `wayland`, `bevy_gilrs`
  and `bevy_clipboard`. The trimmed list in Task 1 pulls **none** of `alsa-sys`, `libudev-sys` or
  `wayland-sys`; `x11-dl` is present and dlopens, so it needs no build-time library.
- **The single most likely way to ship this story broken is a `changes()`-driven reconciler.**
  After `apply_snapshot`, `changes()` is empty — no spawned, no despawned, no tiles — even for
  entities the snapshot carries. Wolf ruled at 5.2's review that this stays; the contract is in
  `deferred-work.md`. A naive reconciler renders **nothing** on connect. AC17 exists solely to
  make that failure impossible to ship.
- **A round-trip test cannot catch a mirrored world.** `(x,y,z)→(x,z,y)` and `(x,y,z)→(x,z,-y)`
  both round-trip; one flips handedness. Sim is z-up, Bevy is Y-up, and the whole valley coming
  out mirrored is invisible in grey boxes and obvious the moment 5.4 puts the camp in it. Pin one
  asymmetric point with a hand-written literal — this project has hit the self-referential-oracle
  trap in 1.1, 1.2, 1.3 and again at 5.2's review.
- **Interpolation is deliberately deferred to 6.1, overriding 5.2's expectation.** 5.2's deferral
  named this story as the wiring point for *"gui reconciliation and AD-15 interpolation"*. The
  reconciliation half lands here; the blending half does not. Reasons: this story is allowed to
  be ugly, smooth motion is 6.1's headline outcome ("The World Moves"), and 5.3 is already the
  milestone's largest story. **The decision changes, the reason is recorded, and
  `previous_entity()` stays inert for one more story** — Task 9 writes that deferral so review
  can tell "inert by design" from "silently dropped".
- **Verified Bevy 0.19 API surface** (each compile-checked at story-creation, so do not
  re-derive from memory or from older Bevy docs):
  - `bevy::render::view::screenshot::{Screenshot, save_to_disk}`;
    `Screenshot::primary_window()`; `.observe(save_to_disk(path))`.
  - `bevy::dev_tools::fps_overlay::{FpsOverlayPlugin, FpsOverlayConfig}`; `config.enabled: bool`
    is the toggle; pair with `bevy::diagnostic::FrameTimeDiagnosticsPlugin::default()`.
  - `bevy::render::renderer::RenderAdapterInfo` — fields `backend`, `name`, `device_type`,
    `driver`, `driver_info`. This is AC8's source; read it, do not infer the backend.
  - `RenderCreation::Automatic` takes **`Box<WgpuSettings>`**, not `WgpuSettings`.
  - Buffered events are `MessageWriter<AppExit>` with `.write(AppExit::Success)` — the
    `EventWriter`/`.send()` spelling is from an older Bevy and will not compile.
  - Rendering components are `Mesh3d`, `MeshMaterial3d`, `Camera3d`; `MinimalPlugins` +
    `app.update()` drives the headless tests.
  - Pinned transitively by the trim: `wgpu` 29.0.4, `winit` 0.30.13, `naga` 29.0.4, `ash` 0.38.
- **`--frames N` must drive the real loop.** 1.3 shipped `tui --frame`, which returned before the
  reader thread was spawned and structurally could not show a climbing tick; 2.2's `--frames N`
  re-centred the camera every frame and rendered motion as stillness. Both were found at review.
  The capture must exercise the same path the live window does.
- **`simd` has no seed flag** — the seed is the constant `SEED` (`simd/src/main.rs:20`) and the
  port is positional (`simd 7431`).
- **The mutation runner rewrites source in place and is not concurrency-safe** — run
  `scripts/mutate.sh` alone, never beside a gate or a review.
- When torn between simple and general, pick simple and leave a `// NOTE:` naming the limitation.

### Project Structure (files to touch)

```
Cargo.toml                          UPDATE  members += gui; workspace dep bevy 0.19.0
crates/gui/Cargo.toml               NEW     protocol + client-core + anyhow + bevy (trimmed)
crates/gui/src/main.rs              NEW     CLI, app wiring, connect + reader thread
crates/gui/src/mirror.rs            NEW     Mirror resource + the one ingestion system
crates/gui/src/transform.rs         NEW     world_to_render / render_to_world + its two tests
crates/gui/src/project.rs           NEW     markers, exposed-tile predicate, reconciliation
crates/gui/src/camera.rs            NEW     orbit/zoom rig + headless rig tests
crates/gui/src/capture.rs           NEW     --capture, screenshot, frame-count exit
crates/gui/tests/headless.rs        NEW     MinimalPlugins: re-project, seam, partition
crates/gui/tests/capture.rs         NEW     surface-requiring self-tests, excluded from the gate
scripts/gate.sh                     UPDATE  three probes collapse into one loop over the crates
_bmad-output/implementation-artifacts/deferred-work.md                       UPDATE  5.3 deferral
_bmad-output/implementation-artifacts/mutations/5-3-a-window-onto-the-valley.sh   NEW
```

### Previous story intelligence

- **5.2 left two seams inert and named this story as the wiring point.** `changes()` is wired
  here (AC16/17); `previous_entity()` is not, and Task 9 records why.
- **5.2's review corrected an instrument caveat in the direction of *more* determinism**: a
  fixed daemon-start-to-connect delay plus a fixed frame count makes the TUI capture byte-stable
  including entity glyphs. Hold both fixed in AC11's concurrent-TUI check and it is a real
  comparison rather than a range check.
- **Pin `--z 9` on any TUI cross-check.** `opening_z` picks z 19, which shows no camp, no dwarves
  and no lights while the status line still reads `dwarves 5`.

### Verification

**Gate (must be green before done, and stays headless):**

```bash
scripts/gate.sh
```

**Envelope + backend (AC7, AC8). Run on a display-capable machine:**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --bins
./target/debug/simd 7431 &
sleep 3
./target/debug/gui 7431            # a window must open and keep rendering
```

The binary must print its adapter line on startup. **The required observation** is a line naming
a real backend and adapter, e.g. `backend=Vulkan adapter="AMD Radeon Pro 460 (RADV POLARIS11)"`.
`backend=Gl` is equally a pass — AC8 asks which, not which one specifically. Paste it verbatim.

**Concurrent TUI cross-check (AC11), same daemon, in a second shell:**

```bash
out=$(./target/debug/tui 7431 --frames 6 --z 9 2>/dev/null | sed -e $'s/\x1b\\[[0-9;]*m//g')
for g in '│' '♠' '†' '♨' '☺'; do
  printf '%s = %s\n' "$g" "$(printf '%s' "$out" | grep -o "$g" | wc -l)"
done
```

Terrain must read `│ = 6` and `♠ = 48` — byte-stable because terrain is seeded and static. Count
with `grep -o`, never `tr -cd`: `tr` works on bytes and the box glyphs share leading UTF-8 bytes.

**Capture (AC25). This recipe cannot run until the instrument exists**, so it is stated here as
the obligation the dev agent inherits:

```bash
./target/debug/gui 7431 --capture /tmp/frostvein-5-3.png --frames 60
```

**The required non-zero observation:** the file exists, is larger than a trivially-black PNG, and
a pixel histogram reports a non-zero count of non-background pixels. Then run it a second time at
a later tick and assert the two images differ. **Exit 0 is not a result.**

**Terrain range-check, measured at story-creation against the shipped seed** — the figures the
projection must reproduce:

| Quantity | Value |
| --- | --- |
| tiles in the world | 524,288 |
| solid tiles | 315,068 |
| **exposed solid tiles (the draw set)** | **53,365** |
| exposed faces | 81,648 |
| top-face-only tiles | 20,788 |
| distinct materials on exposed tiles | 8 |

A projection reporting ~315k or ~20.8k cubes has the predicate wrong in a specific, diagnosable
direction. This table is the instrument's oracle.

**Sabotage:**

```bash
scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/5-3-a-window-onto-the-valley.sh
```

### If this overruns one session

Epic 5's pre-named split line is **envelope + lifecycle | projection + instruments**. That line
was drawn before the environment was measured and it now cuts the wrong way: the first half is
the half that needs a display, so taking it front-loads the blocked work. **The split this story
actually affords is Tasks 1–7 (everything headless-provable, in any devpod) | Task 8 (the live
run).** That is a horizontal cut and it breaks the vertical-slice rule, so it is a last resort,
not a plan.

**Either split breaches CM2** — wow beat 1 moves from story 4 of 11 (36%) to story 5 of 12 (42%),
against the first-third mandate. Epic 5 records that a split of 5.2 or 5.3 is the trigger to
re-check CM2 **on the record**, never a free move. If you take it, say so and re-check.

### References

- Story 5.3 ACs and Epic 5's split/CM2 rules — `_bmad-output/planning-artifacts/epics.md:699-745`,
  `:613-619`
- AD-13…AD-18, M2 conventions, stack, structural seed —
  `_bmad-output/planning-artifacts/architecture/architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md:62-242`
- FR31, FR37, NFR6, NFR8, UX-DR1, UX-DR20 — `epics.md:59-100`, `:149-160`
- The `changes()`-after-snapshot contract, in full —
  `_bmad-output/implementation-artifacts/deferred-work.md` § *Deferred from: story 5.2*
- `client-core`'s API and the out-of-bounds dirty-tile skip — `crates/client-core/src/lib.rs:44-193`
- The connect/reader/arg-parse shapes to copy — `crates/tui/src/main.rs:76-268`
- The gate probes to collapse — `scripts/gate.sh:73-105`
- Story rules, instrument rule, "exit 0 is not a result" — `docs/technical-preferences.md:64-101`
- 5.2's instrument determinism correction and the `--z 9` trap —
  `_bmad-output/implementation-artifacts/5-2-one-mirror-two-clients.md:451-465`, `:376-378`

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
| --- | --- |
| 2026-08-11 | Story created. Environment premise corrected on the record: no devpod can open a window, and the image carries no graphics userspace at all (both measured), so the live half moves to `rebelspice` by Wolf's decision and Task 0 owns the runtime-library install. Bevy feature trim verified to compile (449 crates, exit 0) and default features verified to fail on `wayland-sys`. Bevy 0.19 API surface compile-checked. Terrain draw-set measured off a live snapshot: 53,365 exposed of 315,068 solid. Gate baseline recorded green at 61 s warm. |
