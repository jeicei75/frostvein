# Review — Version & Reality-Check Lens

- **Artifact:** `architecture-frostvein-2026-08-09/ARCHITECTURE-SPINE.md`
- **Mandate:** every committed technology decision web-researched or reality-checked, not asserted from training data
- **Reviewed:** 2026-08-09
- **Verdict:** PASS — all committed stack decisions check out against crates.io/docs.rs and the repo; one minor imprecision in the diagnostics-overlay phrasing, two informational notes.

## 1. bevy 0.19.0 — current stable, same train as bevy_ecs — CONFIRMED

- crates.io API: `bevy` max_stable_version = **0.19.0**, published **2026-06-19**
  (rc train May–June 2026; previous stable 0.18.1, 2026-03-04). The spine's
  "verified current on crates.io / bevy.org, 2026-08-09" holds — 0.19.0 was
  ~7 weeks old at spine date, no newer release.
- crates.io API: `bevy_ecs` max_stable_version = **0.19.0**, published the
  **same day** (2026-06-19). Same release train; the "move together, same 0.x
  line" convention is coherent and currently satisfied.
- MSRV note: bevy 0.19.0 requires Rust ≥ 1.95.0. Repo toolchain is 1.97.1
  (mise) — compatible. (Informational; spine doesn't record MSRV and doesn't
  need to.)

## 2. Frame-time diagnostics + screenshot API in 0.19 — CONFIRMED, one phrasing nit

- **Frame-time diagnostics:** `bevy::diagnostic::FrameTimeDiagnosticsPlugin`
  exists in 0.19 (frame time, fps, frame count). Core, no feature flag.
- **Overlay:** the ready-made overlay is `bevy::dev_tools::fps_overlay::FpsOverlayPlugin`,
  present in 0.19 — **but it is behind the `bevy_dev_tools` cargo feature,
  which is NOT among the 74 default features** (verified against
  docs.rs/crate/bevy/0.19.0/features). The spine's stack row says "default
  features; ... Frame diagnostics and screenshots are built-ins — no extra
  deps". "No extra deps" is true; "default features" is not quite, for the
  overlay specifically. Either enable `features = ["bevy_dev_tools"]` on the
  `gui` crate or hand-roll a text overlay from `FrameTimeDiagnosticsPlugin`
  data. **MINOR** — one Cargo.toml line, but the spine should not imply the
  overlay ships in the default feature set.
- **Screenshot API:** `bevy::render::view::window::screenshot::{Screenshot, save_to_disk}`
  exists in 0.19 (spawn `Screenshot::primary_window()`, observe with
  `save_to_disk(path)`; official `window/screenshot` example current).
  `bevy_render` IS a default feature, so this one genuinely is default-features
  built-in. AD-17 rung 3's `--capture` instrument is real.

## 3. Headless Bevy for logic tests — CONFIRMED, current practice

- Official upstream guidance exists: `bevy` repo `tests/how_to_test_apps.rs`
  builds apps with `MinimalPlugins` and drives `app.update()` under
  `cargo test`. Community docs (taintedcoders headless-mode guide) match.
- The spine's "no GPU in CI" constraint is not just possible but *required*:
  `RenderPlugin` panics without a GPU and `WinitPlugin` is main-thread-only,
  so excluding them is the documented pattern. AD-17 rung 2 is sound.
- Note: upstream has an open discussion (bevy#15203) about a dedicated
  `HeadlessPlugins` group; `MinimalPlugins` remains the current practice the
  spine can rely on.

## 4. wgpu-prefers-Vulkan / WSLg Dozen — SANITY-CHECKED, holds

- Dozen (`dzn`) is real: Mesa's Vulkan-over-D3D12 driver, built by
  Collabora/Microsoft specifically for cases like WSLg where no native Linux
  Vulkan ICD exists (microsoft/wslg#1254, Phoronix coverage).
- "Younger than its GL path" — correct: the GL-on-D3D12 Gallium driver
  predates dzn (dzn merged in Mesa 22.1, 2022, after the GL path shipped with
  WSLg's launch stack), and maturity concerns are documented
  (microsoft/wslg#1340: conformance strong on Vulkan 1.0, incomplete ≥1.2/1.3;
  apps "fail to run or experience degraded performance").
- wgpu's default backend priority on Linux does prefer Vulkan over GL.
- The spine's own hedge — "unproven until run", first `gui` story proves the
  envelope — is exactly the right control for this residual risk. The local
  claims (glxinfo GL proof, Mesa 25.3.5, RTX 4080, D3D12 passthrough) are
  stated as dev-machine reality-checks, which is the correct evidence class
  for them; not web-verifiable, accepted as declared.

## 5. Repo pins vs inherited stack — CONFIRMED

- `/workspace/projects/frostvein/Cargo.toml` line 16: `bevy_ecs = "0.19.0"`
  (workspace dep; `crates/sim-core/Cargo.toml` uses `bevy_ecs.workspace = true`).
- `/workspace/projects/frostvein/Cargo.lock`: `bevy_ecs 0.19.0` from
  crates.io, with the 0.19.0 satellite crates (bevy_reflect, bevy_tasks,
  bevy_utils, ...). No full `bevy` in the lockfile yet — correct, `gui`
  doesn't exist.
- Parent spine (2026-08-01) stack table: "bevy_ecs (headless, not full Bevy) |
  0.19.0". The M2 spine's "aligns with `sim-core`'s bevy_ecs 0.19.0" matches
  what is actually pinned. No drift.

## 6. Remaining claims sweep — nothing else out of date

- Bevy 0.19's headline change (new BSN scene system) does not touch any spine
  claim; the spine commits to procedural meshes, no scenes/assets. No conflict.
- **INFO:** the Deferred item names `bevy_vox_scene` for a future asset
  pipeline. Third-party Bevy crates lag engine releases; its 0.19
  compatibility was not verified here and must be re-verified at trigger
  time. Correctly deferred — flagged so the trigger story checks it.
- NFR2 ~200 ms, AD references, and parent-staleness notes (FR24/Unreal) are
  project-internal, consistent with the sprint-change proposal; no external
  claims to verify.

## Findings summary

| # | Severity | Finding |
| --- | --- | --- |
| 1 | MINOR | `FpsOverlayPlugin` is behind the non-default `bevy_dev_tools` feature; the stack row's "default features ... built-ins" overstates the overlay. Fix: name the feature flag in the stack note, or plan a hand-rolled overlay from `FrameTimeDiagnosticsPlugin`. |
| 2 | INFO | Dozen/WSLg Vulkan maturity risk is real and documented upstream; spine's first-story envelope proof is the right mitigation — keep it non-negotiable. |
| 3 | INFO | `bevy_vox_scene` (deferred) is unverified against 0.19; re-verify at trigger. |
| 4 | INFO | bevy 0.19 MSRV is Rust 1.95.0; toolchain 1.97.1 satisfies it. |

Sources: [crates.io bevy API](https://crates.io/api/v1/crates/bevy), [crates.io bevy_ecs API](https://crates.io/api/v1/crates/bevy_ecs), [Bevy 0.19 announcement](https://bevy.org/news/bevy-0-19/), [docs.rs bevy 0.19 diagnostic](https://docs.rs/bevy/0.19.0/bevy/diagnostic/index.html), [docs.rs bevy 0.19 features](https://docs.rs/crate/bevy/0.19.0/features), [docs.rs Screenshot](https://docs.rs/bevy/latest/bevy/render/view/window/screenshot/struct.Screenshot.html), [Bevy screenshot example](https://bevy.org/examples/window/screenshot/), [how_to_test_apps.rs](https://github.com/bevyengine/bevy/blob/main/tests/how_to_test_apps.rs), [HeadlessPlugins issue](https://github.com/bevyengine/bevy/issues/15203), [wslg#1340 Dozen Vulkan compliance](https://github.com/microsoft/wslg/issues/1340), [wslg#1254 Vulkan in WSLg](https://github.com/microsoft/wslg/issues/1254), [Phoronix on Mesa Dozen](https://www.phoronix.com/news/Mesa-Dozen-VLK-D3D12)
