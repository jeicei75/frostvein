# 11.1a creation control — build `41b3f02`, 2026-09-18

Taken at story creation, before any code change, on a clean tree. These are the figures story
11.1a's Dev Notes quote. **They are a control to compare against, not a floor to inherit** — the
floor is build-specific and has moved 3.3x and 20x across single stories before. Task 0 re-takes it.

## Provenance

`./target/debug/gui --version` printed `gui build 5f8e259`, which is the second parent of the merge
at `HEAD` `41b3f02`. Not stale: `git rev-parse 41b3f02^{tree} 5f8e259^{tree}` are the same tree
(`8068d41e…`), so the binary's content is HEAD's. The stamp names the merged commit, not the merge.

```bash
./target/debug/simd 7466 &
./target/debug/gui 7466 --headless --static-world --subdiv 4 --frames 160 \
  --capture boot-41b3f02-<a|b|c|d>.png
```

All four exited 0. `a` and `d` are committed here; `b` and `c` were measured and discarded.

## The four same-build runs

| run | warm-lit | ground-median | near-white % | blown-pool % | p99 | mean (Rec.601) |
| --- | --- | --- | --- | --- | --- | --- |
| a | 27,958 | 81 | 0.7656 | 0.6084 | 188.3 | 71.337 |
| b | 28,562 | 81 | 0.5789 | 0.4516 | 178.3 | 71.144 |
| c | 28,746 | 81 | 0.7601 | 0.5312 | 188.4 | 71.376 |
| d | 28,760 | 81 | 0.8177 | 0.5768 | 191.3 | 71.396 |
| **spread** | **802** | **0** | **0.2388 pp** | **0.1568 pp** | **13.0** | **0.252** |

Mean luminance is Rec.601 via `../10-7-signoff/lumstats.py`, not the plain RGB channel average that
issue #98 found mislabelled in 10.10's record.

## Finding 1 — both area ceilings have less headroom than their own noise

`NEAR_WHITE_AREA_CEILING` is 0.946072 % and `BLOWN_POOL_FRACTION_CEILING` is 0.623806 %
(`crates/gui/src/capture.rs:602, :578`).

| statistic | worst run | ceiling | headroom | same-build spread |
| --- | --- | --- | --- | --- |
| near-white area | 0.8177 % | 0.946072 % | **0.1284 pp** | 0.2388 pp |
| blown pool | 0.6084 % | 0.623806 % | **0.0154 pp** | 0.1568 pp |

The blown-pool margin is **a tenth of the noise**. A capture can trip a ceiling by luck alone, with
nothing changed. This is issue #90 ("near-white-area's same-build swing is 2.8x the jitter its
ceiling was calibrated for") confirmed at four samples rather than two, and extended to blown-pool.

It is compounded by a venue rule already on the record: `deferred-work.md:1237-1246` — llvmpipe
under-reads near-white area by ~16 %, so **a headless area figure must never be judged against a
ceiling calibrated on a GPU frame**; read it only as a delta between two headless runs.

**Consequence, and why Epic 11 split:** the epic's bloom AC ("`NEAR_WHITE_AREA_CEILING` and the
blown-pool figure stay inside 10.8's re-calibrated ceilings") could not be honoured headless. Wolf
ruled it a headless on/off DELTA plus a vehicle-judged ceiling. No ceiling was raised.

`ground-median-luminance` is the one whole-frame band that does not move at all — 81 on all four.

## Finding 2 — the crease instrument works, and has a zero noise floor

`creases-prototype.py` reports Rec.601 `p10`/`median`/`p90`/`mean` over pinned windows.

**GREEN (same build, a vs d):** `p10` and `median` are **identical on all four windows**. The means
move ≤ 0.027 on the three non-camp windows, against 0.252 for whole-frame mean over four runs.

**RED (control vs `--lights-off ambient`, an occlusion proxy):** `terrace-creases` `p10` 30 → 0 and
median 65 → 1, while `open-snow-LL` median moves only 117 → 104. The window that is full of creases
moves far more than the flat one — the discrimination Epic 11's AO criterion asks for.

**camp-terraces is a diagnostic only.** Its mean moves **1.19** between the two same-build runs —
light flicker, not the sim, since `--static-world` pauses the world. Nothing may be asserted in it.

The ambient-off run exits **101** on `GROUND_LUMINANCE_FLOOR` (ground median 81 → 45). That is
expected and does not spoil the measurement: `save_then_validate` (`capture.rs:1276`) writes the PNG
before the range check asserts.

## Finding 3 — Epic 11's lavapipe unknown is answered

Epic 11 states that whether any post-stack mechanism renders under lavapipe is unknown and makes the
probe each story's first task. For SSAO it is now answered and stays answered: the plugin's only GPU
gate is `max_storage_textures_per_shader_stage >= 5`
(`bevy_pbr-0.19.0/src/ssao/mod.rs:61-70`, a `warn!` then `return`). Probed through wgpu 29.0.4 on
this devpod:

```
adapter: llvmpipe (LLVM 19.1.7, 256 bits) | backend=Vulkan | device_type=Cpu | driver=Mesa 25.0.7
  max_storage_textures_per_shader_stage = 48      (SSAO needs >= 5 -> SUPPORTED)
```

This proves the plugin loads, not that the effect looks right.

## Finding 4 — the silent skip that split the story

`extract_ssao_settings` (`bevy_pbr-0.19.0/src/ssao/mod.rs:479-494`) logs an `error!` and then
**`return`s out of the loop** — not `continue` — when `Msaa != Off`, so every camera is skipped. The
default is `Sample4` (`bevy_render-0.19.0/src/view/mod.rs:243-244`) and this client sets `Msaa`
nowhere. AO added without `Msaa::Off` renders a clean frame and exits 0 with no occlusion in it.

By contrast the other prerequisites are automatic: `Bloom` carries `#[require(Hdr)]` and
`ScreenSpaceAmbientOcclusion` carries `#[require(DepthPrepass, NormalPrepass)]`. `Msaa::Off` is the
only one a human must remember, and it is the only one that fails quietly.

## Task 0 re-take — `41b3f02-dirty`, 2026-09-18

The rebuilt GUI printed `gui build 41b3f02-dirty`. The dirty suffix is from the story's
uncommitted implementation-artifact files, not stale executable content: the clean merge tree is
`8068d41e1a6e30adf3d8ceecaaa04c8b3da4da1e`; the old `5f8e259` stamp names its second parent and
that parent has the same tree. Per the task, `crates/gui/build.rs` was touched and `cargo build -p
gui --offline` completed before this retake.

All four captures used `--headless --static-world --subdiv 4 --frames 160` and exited 0.

| run | warm-lit | ground-median | near-white % | blown-pool % | p99 | mean (Rec.601) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| a | 28,772 | 81 | 0.8584 | 0.5872 | 192.5 | 71.428 |
| b | 27,399 | 80 | 0.5431 | 0.4428 | 175.3 | 71.050 |
| c | 27,865 | 80 | 0.6891 | 0.4949 | 186.8 | 71.265 |
| d | 27,714 | 81 | 0.6763 | 0.5158 | 183.8 | 71.217 |
| **spread** | **1,373** | **1** | **0.3153 pp** | **0.1444 pp** | **17.2** | **0.378** |

Means use the existing Rec.601 integer instrument in `10-7-signoff/lumstats.py`. These are the
new build-specific control floors; no creation figure is reused as a threshold.
