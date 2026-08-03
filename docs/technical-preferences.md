# Technical Preferences

These preferences apply to every agent (analyst, PM, architect, SM, dev, QA) on
this project. When these conflict with an agent's instinct toward thoroughness,
these win.

## Stack (decided — record, don't evaluate)
- **Language**: Rust, stable toolchain, 2021+ edition.
- **Workspace**: single Cargo workspace, four crates:
  - `sim-core` — pure library: world, ECS, jobs, pathfinding. No I/O of any kind.
  - `protocol` — pure library: serde wire types only (no logic, no I/O). The single
    source of truth for message shapes (architecture AD-6).
  - `simd` (daemon) — binary: tick loop, TCP server, protocol encoding. Depends on
    sim-core and protocol.
  - `tui` — binary: terminal client. Depends on protocol only (talks TCP).
- **ECS**: `bevy_ecs` crate used headless. Not the full Bevy engine.
- **Async/networking**: prefer std::net + threads for v0. Introduce tokio only if
  a story concretely needs it. Do not build an async abstraction layer.
- **Serialization**: `serde` + `serde_json`, newline-delimited over TCP.
- **TUI**: `crossterm` for terminal control; hand-rolled cell framebuffer flushed
  once per frame. No TUI framework unless a story shows crossterm alone hurts.
  Use 24-bit truecolor from the start; treat color as data (material/profession
  to RGB mapping), not a fixed palette — the future raycast view reuses this.
- **Errors**: `thiserror` in libraries, `anyhow` in binaries.
- **Randomness**: seeded `rand` (StdRng or ChaCha). All sim randomness flows from
  the world seed so runs are reproducible.
- **Testing**: built-in cargo test. Scenario tests live in sim-core as integration
  tests that build a world, inject commands, tick N times, and assert state.

## Pre-made ADRs (write these up as-is, one paragraph of rationale each)
1. Sim core is a pure library; daemon and clients are shells around it.
2. Fixed-timestep tick loop (10/sec default), decoupled from clients; pause and
   fast-forward = tick-rate changes (refined by architecture AD-2: the daemon loop
   never stops — pause freezes sim time, not command intake); determinism from
   seed + command sequence (a persistent command log is deferred, AD-10).
3. Protocol v0 is newline-delimited JSON over localhost TCP: snapshot on connect,
   deltas per tick, commands upstream. Optimization deferred until the message
   shapes have stabilized in practice.
4. Clients contain zero game logic. Any rule a client needs is exposed by the sim.
5. Plain A* pathfinding on the voxel grid for milestone 1. Hierarchical
   pathfinding is acknowledged future work, not scaffolded now.

Architect: record decisions and module layout. Do not produce comparative
evaluations of alternatives. If you disagree with a decision, flag it in one
sentence and proceed.

## Anti-overengineering rules (policy, not suggestions)
- **YAGNI is policy.** Build for the current story's needs, not the imagined
  fortress of 500 dwarves.
- No traits, generics, or abstraction layers with a single implementation.
  Introduce the abstraction when the second concrete case exists, not before.
- No config files, plugin systems, event buses, or data-driven content systems
  until a third concrete use case exists in shipped code.
- Hardcoded constants are fine. Promote to a constants module when reused;
  promote to config only when a story requires runtime change.
- Protocol chattiness is acceptable. Do not optimize serialization, batch
  messages, or add compression in milestone 1.
- No premature performance work. Optimize only after a profile shows a problem
  in a realistic scenario.
- Dependencies: prefer std and the crates named above. Each new dependency needs
  one sentence of justification in the story.
- Unsafe Rust: not in milestone 1.

## Story rules (for the SM)
- Stories are vertical slices: a thin path through sim → protocol → TUI (or a
  headless sim test), never a horizontal layer ("build the complete job system").
- Every story ends with something observable: a passing scenario test or a
  visible TUI behavior. No pure-infrastructure or pure-refactoring stories in
  milestone 1.
- A story fits one dev-agent session. If it doesn't, split it vertically.
- Milestone 1 is 8–12 stories. More than that means scope creep — cut, don't plan.

## Dev workflow
- Every story: `scripts/gate.sh` green before done — `cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test`, plus a probe that `tui` has no `sim-core`
  edge. It exits non-zero, so a green gate is a fact rather than a claim.
- Dev agent writes the tests for its own story; there is no separate QA gate.
  The walking-skeleton scenario test is the milestone gate.
- Small commits per story with imperative messages. No long-lived branches.
- When uncertain between a simple and a general solution, choose simple and
  leave a `// NOTE:` comment naming the known limitation.

## Documentation restraint
- PRD: a few pages, derived from the project brief. No personas, no market
  analysis, no competitive research.
- Architecture doc: the ADRs above + protocol message list + crate layout. Aim
  for something readable in ten minutes.
- Default answer to "should I also document X?" is no.
