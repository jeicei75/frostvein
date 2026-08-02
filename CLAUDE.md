# CLAUDE.md — frostvein

Operating rules for AI agents in this repo. **What we're building and why**:
`docs/project-brief.md`. **Stack, ADRs, anti-overengineering policy, story rules**:
`docs/technical-preferences.md` — read both before any planning or code work; when
they conflict with your instinct toward thoroughness, **they win**.

## The gate (every story, before "done")

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Ground rules

1. **YAGNI is policy, not advice.** No abstraction with a single implementation; no
   config/plugin/event systems before a third concrete use; hardcoded constants are
   fine. When torn between simple and general, pick simple + a `// NOTE:` naming
   the limitation.
2. **Layout is decided:** one Cargo workspace, four crates — `sim-core` (pure lib,
   zero I/O), `protocol` (wire types only, the single home of message shapes),
   `simd` (daemon: tick loop + TCP), `tui` (client, depends on `protocol` only).
   Clients contain zero game logic.
3. **Determinism is load-bearing.** All sim randomness flows from the world seed;
   scenario tests (build world → inject commands → tick N → assert) depend on it.
4. **Small commits, imperative messages**, author `Völundr <jeicei75@gmail.com>`.
   One story = one branch = one PR; push/PR only after review, on Wolf's explicit yes.
5. **Search with `rg`, find files with `fd`** — never `find .`/`grep -R`. Don't
   broad-search `target/`.
6. **This repo is hosted in the Nidavellir forge but owns its process** — its own
   BMad install, sprint tracking, and memory. Never write to the forge's
   `_bmad-output/`; never assume Asgard context exists here.
