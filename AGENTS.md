# AGENTS.md — frostvein

Rules for any coding agent working in this repo, **including Codex running headless via
`scripts/codex-handoff.sh`**. Codex resolves `AGENTS.md` from its working root upward; this
file is the reason it reads frostvein's rules instead of the forge's generic ones.

Orchestrator-side guidance lives in `CLAUDE.md`. Where both speak, they agree.

**What we're building and why:** `docs/project-brief.md`.
**Stack, ADRs, anti-overengineering policy, story rules:** `docs/technical-preferences.md`.
Read both before planning or code work; when they conflict with your instinct toward
thoroughness, **they win**.

## The gate (every story, before "done")

```bash
scripts/gate.sh
```

Runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and
the `cargo tree -p tui | rg sim-core` dependency-edge probe. It exits non-zero on any
failure and is wired to a pre-commit hook via `core.hooksPath=.githooks`. **Do not report a
green gate you have not run.**

## Ground rules

1. **YAGNI is policy, not advice.** No abstraction with a single implementation; no
   config/plugin/event systems before a third concrete use; hardcoded constants are fine.
   When torn between simple and general, pick simple + a `// NOTE:` naming the limitation.
   *Exception, learned in Epic 1:* resource bounds at an I/O boundary (timeouts, line
   caps, connection caps) are **not** speculative features. Bound every read, write and
   accept loop you add.
2. **Layout is decided:** one Cargo workspace, four crates — `sim-core` (pure lib, zero
   I/O), `protocol` (wire types only, the single home of message shapes), `simd` (daemon:
   tick loop + TCP), `tui` (client, depends on `protocol` only). Clients contain zero game
   logic.
3. **Determinism is load-bearing.** All sim randomness flows from the world seed; scenario
   tests (build world → inject commands → tick N → assert) depend on it. No wall clock, no
   unseeded randomness, no `HashMap`/`HashSet` iteration feeding a sim outcome.
4. **`#![forbid(unsafe_code)]` in all four crates.** `thiserror` in `sim-core`/`protocol`,
   `anyhow` in `simd`/`tui`.
5. **The dependency stack is closed.** A new dependency needs a one-sentence justification
   in its story. Do not add one to make something convenient.
6. **Small commits, imperative messages**, author `Völundr <jeicei75@gmail.com>`. Commit on
   every green step — a TDD log, never one squash at the end. One story = one branch = one
   PR; **never push and never open a PR** — that is Wolf's call, made after review.
7. **Search with `rg`, find files with `fd`** — never `find .` / `grep -R`. Don't
   broad-search `target/`.

## Rules for a delegated dev agent

These exist because Epic 1's code review found the same classes three stories running.
Each one is cheap here and expensive later.

1. **A green suite is not evidence. Sabotage is.** For every test that pins a mapping, a
   look-up table or a boundary constant: break the production code so that test *should*
   fail, run it, confirm it goes red, restore. **Paste the actual failure output into the
   Dev Agent Record.** A checked-off coverage box is a claim; the red output is the proof.
   - Never assert by running both sides through the function under test — that proves
     ordering and never mapping. State the expected truth independently, as a hand-written
     literal.
   - **Sabotage the constants too**, not just the mappings. Epic 1 shipped a suite where
     widening a `PEEK_DEPTH` constant 3→6 left every test green because the fixture lacked
     the range to express the negative case.
2. **Report honestly, and say "manual" when a step was manual.** Do not describe a manual
   `cargo run` as an automated assertion. If you did not observe something, say you did not
   observe it.
3. **If a blocker is environmental, say so and stop working around it.** Never change
   production code, weaken or delete a test, or hand-roll a replacement for a std primitive
   to make a sandbox limitation go away. Report it in your final message and leave the
   production code correct.
4. **On hitting a blocker, finish everything that does not depend on it** before stopping,
   then report. Epic 1 lost a whole run to stopping at the first blocker while unblocked
   test work sat undone.
5. **If you are continuing a story a previous session started, re-derive and restate the
   RED evidence** for any task you complete. A new session does not inherit the previous
   run's prompt, and "RED not observed because the implementation already existed" is not
   an acceptable record.
6. **Respect the story's scope guardrails literally.** If you believe a wire shape or a
   capability is missing, stop and say so rather than adding it.

## Command hygiene

- Scoped and quiet: target paths/globs, pipe through `tail` / `--name-only` / `--oneline`,
  quiet installs. Quiet **stdout**, never **stderr** — no blanket `2>/dev/null`.
- Absolute paths, not `cd`. Each shell invocation is fresh; state does not persist.
- Never broad-search vendored/build trees (`target/`, `.venv`, `node_modules`).
