# CLAUDE.md — frostvein

Operating rules for AI agents in this repo. **What we're building and why**:
`docs/project-brief.md`. **Stack, ADRs, anti-overengineering policy, story rules**:
`docs/technical-preferences.md` — read both before any planning or code work; when
they conflict with your instinct toward thoroughness, **they win**.

## The gate (every story, before "done")

```bash
scripts/gate.sh
```

Runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and a
probe that `tui` has not grown a `sim-core` edge. It exits non-zero and is wired to a
pre-commit hook (`git config core.hooksPath .githooks`, once per clone). **Run the script,
not the three commands by hand** — it also repairs the build cache when work moves between
the two devpods, which otherwise makes every binary-spawning test fail while unit tests stay
green. **Do not report a green gate you have not run.**

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
   **One carve-out, ruled 2026-09-11: a post-merge BOARD RECORD goes straight to `main`.**
   That means a single commit which records a merge that has ALREADY happened, touches only
   `_bmad-output/implementation-artifacts/sprint-status.yaml` and at most a story file's
   `Status:` line, and changes nothing under `crates/`, `scripts/` or `assets/`. It is a
   record, not a change, and a PR for it buys no review. Two conditions, both load-bearing:
   the pre-push hook still runs the fast gate (it does, on every branch — nothing is
   bypassed), and **get off `main` the moment it is pushed.** Wolf's merge has already left
   the working copy sitting on `main`, and that is exactly how the NEXT commit lands there by
   inertia. Anything wider than the carve-out — a rule change, a record that proposes work,
   a story file edited beyond its status — is a branch and a PR like everything else.
5. **Search with `rg`, find files with `fd`** — never `find .`/`grep -R`. Don't
   broad-search `target/`.
6. **This repo is hosted in the Nidavellir forge but owns its process** — its own
   BMad install, sprint tracking, and memory. Never write to the forge's
   `_bmad-output/`; never assume Asgard context exists here.
