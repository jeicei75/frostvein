# Review — reality check (versions & fitness)

Lens: every committed decision verified against the web, not asserted from
training data. Run inline (sequential fallback — subagent quota exhausted
mid-gate); verification calls made from the parent session via WebFetch
against the crates.io API, 2026-08-01.

## Verified

| Claim | Method | Result |
| --- | --- | --- |
| bevy_ecs 0.19.0 current stable | crates.io API `max_stable_version` | ✅ confirmed |
| crossterm 0.29.0 | crates.io API | ✅ confirmed |
| serde 1.0.229 / serde_json 1.0.151 | crates.io API | ✅ confirmed |
| rand 0.10.2 / rand_chacha 0.10.0 | crates.io API | ✅ confirmed (same 0.10 line, rand_core-compatible) |
| thiserror 2.0.19 / anyhow 1.0.104 | crates.io API | ✅ confirmed |
| rand_chacha can serialize RNG state (AD-11) | crates.io API features for rand_chacha 0.10.0 | ✅ `serde` cargo feature exists (`["dep:serde"]`) — **must be enabled**; noted as a finding below |
| Rust edition 2024 | stable since Rust 1.85 (Feb 2025); tech prefs allow "2021+" | ✅ |

## Accepted on strong prior (not re-fetched)

- bevy_ecs standalone/headless use without full Bevy: bevy_ecs is published
  and documented as a usable-standalone ECS crate; the project brief already
  committed to it.
- Explicit single-threaded ordering: `Schedule::run(&mut World)` with
  `.chain()`ed systems is core stable bevy_ecs API across 0.12→0.19.
- crossterm 24-bit truecolor: `Color::Rgb { r, g, b }` has been in crossterm
  for many major versions.

## Findings

- **[medium]** AD-11 depends on rand_chacha's optional `serde` feature; a dev
  agent adding the default features only will hit a wall. Fix: note the
  feature flag in the Stack table. → applied.
- **[low]** rand 0.10's API differs from the widely-trained-on 0.8/0.9
  (`rand::rng()`, `random_range`, trait renames). No spine change — versions
  are pinned, dev agents will read docs.rs — but story authors should not
  trust memory for rand idioms. Recorded here for the record.

Verdict: stack is real, current, and fit; one feature-flag note applied to
the spine.
