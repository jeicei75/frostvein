---
baseline_commit: b8b7a5c
model: claude-opus-5-5  # session default; Wolf picked Opus 5.5 to IMPLEMENT too (2026-10-21), no Sonnet subagents
---

# Story 13.1: Scripted Runs and the Tick Bench

Status: in-progress

## Story

As the boss,
I want one scripted scenario file that replays the same run headless for a tick benchmark and at my seat for frame rate,
so that performance is measured on reproducible runs instead of by playing, and a cost regression is caught before it ships.

## Not stacked: branch off `main`

`main` is `b8b7a5c` (PR #213, the M3-close retro). Branch `story-13-1-scripted-runs-and-the-tick-bench` carries `07b4153` (Epic 13 opened) and this file. Epic 13's standing ACs (`epics.md`, Epic 13 header) bind this story: M3's ACs 1–8, with **one full gate per story, at PR** (M3-4, #212); patch passes and commits run the fast tier. This story is action item M3-2 (#210), and 13.2 (#209, the movement spike) measures with it.

**Wire diff (standing AC 4): none.** Scenario files reuse `protocol::Command`'s JSON (`#[serde(tag = "type")]`, `protocol/src/lib.rs:167-212`), so there is no new message shape. The scenario wrapper struct is daemon-local, not wire.

## Found at creation (2026-10-21, on `main` `b8b7a5c`)

A throwaway probe (`simd/src/bin/probe.rs` in a scratch worktree, never committed): release, `World::step` timed per tick, FNV-1a over `serde_json::to_vec(&world.to_save())`, runs one after another, load average about 1.3.

| Run | Ticks | mean ms | p99 ms | max ms | > 10 ms | state |
|---|---|---|---|---|---|---|
| `plan-10-10-1.json` from tick 6, ×5 | 10,000 | 0.1744–0.1758 | 0.421–0.435 | 9.12–9.37 | 0 | `2479b66a35ac887e` ×5 |
| `~/frostvein-saves/12-12-seat-slow.save` (tick 53,759), ×3 | 3,000 | 0.4134–0.4178 | 0.735–0.849 | 9.43–9.76 | 0 | `17b53b99e9161018` ×3 |
| **RED:** `astar_heuristic` iterates its goals twice, plan ×3 | 10,000 | 0.1815–0.1826 (**+4%**) | 0.428–0.446 | 8.81–9.05 | 0 | `2479b66a35ac887e` (same behaviour) |

What the probe established:
- **Determinism holds.** Every run of one start gave the same state hash, so a replay is reproducible.
- **The mean is the discriminating figure, not the max.** The mean spread was under 1% across same-build runs, and a 4% sabotage cleared it 5 times over. The max and p99 moved 3–15% between identical runs, and the max did not move under the RED. Every run's max is a single ~9 ms tick, probably #179's lone A* call.
- **The seat save loads on `main`**, at 7.2 MB. It reads 0.41 ms mean. The 0.82 ms in memory was a live figure, not `step`.
- **The 12.12 tick tables were scratch probes, and none was committed.** spread5's cells were never recorded (`12-12-the-first-goals.md:511`, deferred), and the 518-cell pile is recorded only as "camp ±20, z ±3". This story builds the reproducible versions, and its own figures supersede the old ones.
- **`main` has no `--bench`.** A compare against `main` cannot run until this story merges (Key decisions).

## Task 0: RULED (Wolf, 2026-10-21: "1 ok 2 yes")

- **Q1 (a):** commit `12-12-seat-slow.save` under `scenarios/saves/`.
- **Q2 yes:** `--scenario` holds the start tick until the first client connects.

The questions as asked:


1. **Commit the seat save?** `12-12-seat-slow.save` is 7.2 MB of JSON.
   - **(a) Commit it** under `scenarios/saves/` *(recommended)*. Anyone can then run it, and the bench reproduces it byte for byte. It rots at the next save-format change (12.11 refused older saves), and when it does, the bench fails loudly naming the file, so it gets re-captured rather than silently skipped.
   - (b) Reference `~/frostvein-saves/` and skip it with a loud `SKIPPED` line when the file is absent. That keeps git light, but the scenario exists only on this devpod.
   - (c) Drop it, and rely on scripted scenarios only.
2. **Hold until you connect?** *(recommended: yes)* `simd 7451 --scenario FILE` holds the world on its start tick until the first client connects, then runs at the scenario's speed, and pauses at the end. Your `--perf-log` then covers the whole run from its first tick, however long the launch takes. The alternative is to start at once like `simd 7451`, and then the start of the run is missed.

## Acceptance Criteria

1. `simd --bench FILE [--runs N]` replays a scenario headless in release, with no TCP and no tick sleep. It prints one line per run (ticks, `step` mean/p99/max/over-10 ms, and the state hash) and one summary line. The summary gives the spread of the means across runs, and says `state identical across N runs` or `STATE DIVERGED`; divergence exits 1.
2. A scenario's commands apply on their tick, through the same code that applies a client's command in the live loop. The bench reports `commands applied K/K`. A command the sim refuses prints the refusal and exits 1.
3. `simd PORT --scenario FILE` holds the start tick until the first client connects, then runs at the scenario's `speed`. It applies the same commands on the same ticks, and pauses at `start + ticks`, printing the end line with the state hash. On an uninterrupted run, that hash equals the bench's hash for the same file.
4. `scripts/tick-bench.sh BASE HEAD` builds both commits in scratch worktrees and runs every scenario, interleaving base and head. It prints one row per scenario: base mean, head mean, Δ%, each build's own floor %, and `BEYOND FLOOR` or `within floor`. The verdict is informational, not a threshold. It exits non-zero only when something cannot be measured: a build fails, a scenario fails to load, state diverges, or the base binary has no `--bench`.
5. `scenarios/` holds the starting set, each file named after its cost case. Every scenario is accepted with zero refusals, and its end line shows the case happened (Task 4).
6. A scenario that cannot load (a missing file, malformed JSON, an unknown command type, a daemon-only command, or a save the sim refuses) exits non-zero, naming the file and the cause. A debug build prints `DEBUG BUILD: figures are not a measurement` on every bench run.
7. Wolf runs one scenario at the seat with `--perf-log`, with no manual input. The gui's CSV covers the run, and simd's end line prints the bench's hash for that file (Task 8).

## Tasks / Subtasks

- [x] **Task 0: Wolf rules Q1 and Q2.** Ruled 2026-10-21: Q1 (a) commit the save, Q2 hold until connect.
- [x] **Task 1: the scenario file and one apply path (AC 2, 6).**
  - [x] NEW `crates/simd/src/scenario.rs`. Shape (field names fixed; a `serde(deny_unknown_fields)` struct):
    ```json
    {
      "name": "plan",
      "about": "12.12's goal-chain plan: 46 digs, 3 cuts, 78 pile cells, from tick 6 on DEFAULT_SEED",
      "start": { "seed": 4026891802 },
      "ticks": 10000,
      "speed": "fast4x",
      "commands": [ { "at": 6, "command": { "type": "designate", "kind": "dig", "rect": { "min": [73,64,11], "max": [73,64,11] } } } ]
    }
    ```
    - `start` is either `{"seed": u64}` (generated with `Dims::DEFAULT`) or `{"save": "<path relative to the scenario file>"}`.
    - `at` counts ticks after the start tick. A save starts at its own tick, so absolute ticks would be unreadable there.
    - `ticks` is the run length after the start. `speed` is a `protocol::Speed` used by the live mode; `paused` is refused.
    - Commands must be in non-decreasing `at` order, each `at < ticks`. `set_speed`, `save`, `load` and `quit` are refused at load: a scenario drives the sim, not the daemon.
    - `4026891802` is `sim_core::DEFAULT_SEED` (`0xF005_7E1A`, `lib.rs:27`). JSON has no hex, so write the decimal.
  - [x] Load a save through the SAME validation as the live `Load` (`load_world_from`, `main.rs:344`). Generalise it to return the error rather than print it, so a refused save fails the bench (AC 6) with the same cause text.
  - [x] Extract the five sim arms of the live loop's `match` (`main.rs:224-257`) into one `fn apply_sim_command(world: &mut sim_core::World, command: protocol::Command) -> Option<protocol::Refusal>`. The loop and the scenario both call it. That one function IS AC 2's "same code"; do not write a second mapping.
- [x] **Task 2: `--bench` (AC 1, 2, 6).**
  - [x] `parse_args` gains `--bench FILE`, `--runs N` (default 5; `1` allowed but its summary says `1 run: no floor`) and `--scenario FILE`. `--bench` excludes a port, `--pause-at` and `--scenario`, and conflicting flags are an error. Return a small struct rather than a growing tuple.
  - [x] Per run: build the world from `start`. On each tick, apply the commands whose `at` is due (before the step, as the live loop does), time `world.step()` with `Instant`, then call `bridge::delta(&mut world, speed, refusals)` and encode it as the loop does (`main.rs:306-309`), outside the timed window. The bench therefore leaves the world exactly as the daemon would. Report `step` only. NOTE: delta encoding is untimed; say so in the doc comment.
  - [x] Stats: mean; p99 by nearest rank (`sorted[ceil(0.99·n)-1]`); max; count over 10 ms. Put the stats in a pure function with a unit test on synthetic durations, including n = 1 and an all-equal set.
  - [x] State hash: FNV-1a 64 over `serde_json::to_vec(&world.to_save())`, printed as 16 hex digits. Hand-roll it in a few lines rather than adding a dependency (`std`'s `DefaultHasher` is not stable across releases).
  - [x] End summary line, so the case is visible: `end tick=T marks=M items=I zones=Z goals=w/t/b`. These are `world.designations().len()`, `world.items().len()`, `world.zones().len()`, and each goal's `progress`.
  - [x] Output contract (Task 5 parses it; keep it exact):
    ```
    bench plan run 1/5: ticks=10000 commands=50/50 mean=0.1751 p99=0.4258 max=9.230 over10=0 state=2479b66a35ac887e
    ...
    bench plan end tick=10006 marks=0 items=… zones=78 goals=12/10/1
    bench plan: 5 runs, mean 0.1744..0.1758 ms (spread 0.8%), state identical across 5 runs
    ```
  - [x] `cfg!(debug_assertions)` prints `DEBUG BUILD: figures are not a measurement` on stderr before each run.
- [x] **Task 3: `--scenario` live mode (AC 3).**
  - [x] With `--scenario`, the loop starts `Paused` on the start tick (print `scenario NAME: holding tick T until a client connects`). On the first admitted client (`connect_client` returning `Some`), it sets `speed = scenario.speed`. Q2 may change this; do not build the hold if Wolf rules "start at once".
  - [x] It applies due commands through `apply_sim_command` at the top of the iteration, before the step, using the same "`at` is due" test as the bench. Refusals go into the iteration's `refusals`, so clients see them as usual.
  - [x] At `start + ticks`, it sets `Paused` and prints `scenario NAME: done at tick T state=<hash> commands=K/K` once. The daemon keeps serving afterwards; clients may unpause and play on. A client command during the run is allowed, but then the hash no longer matches the bench. The end line says `(clients sent N commands)` when N > 0, so a mismatch explains itself.
  - [x] `Load` during a scenario run abandons the scenario with a loud line. It does not keep applying commands to a different world.
- [x] **Task 4: the starting set in `scenarios/` (AC 5).** NEW top-level `scenarios/`. Each file says in `about` what it exercises and where its cells came from. Each must run with 0 refusals, and its bench `end` line must show the case.
  - [x] `plan.json`: `12-12-signoff/plan-10-10-1.json` from `at` 6. Its 46 digs and 3 cuts are one-cell `designate` commands, and its 78 pile cells are ONE `place_stockpile` with 78 one-cell rects, exactly as the chain test does (`scenario.rs:5068-5100`). 10,000 ticks. The end shows `goals=12/10/1` or what the run actually reaches; record it.
  - [x] `plan-far-piles.json`: the plan plus three 2×2 piles with min corners `[104,104,13]`, `[24,104,19]` and `[94,64,25]` (`12-12-the-first-goals.md:420`). If a corner is refused or not standable, move it to the nearest accepted 2×2 and record the move.
  - [x] `plan-spread-piles.json` and `plan-compact-piles.json`: the plan plus five 5×5 piles, spread across the map or packed near the camp. **Choose the cells, record them in `about`**, and confirm each pile is accepted. These replace the unrecorded spread5/compact5.
  - [x] `big-pile.json`: one `place_stockpile` rect spanning camp ±20 x/y and z ±3 (`world.camp_origin()`), clipped to the map, plus the plan's digs. 5,000 ticks. Record the zone cell count; 12.12 said 518.
  - [x] `deep-pit.json`: repeated `channel` marks making a way down several levels at one spot. Reuse a 12.10b scenario-test fixture's cells if one fits DEFAULT_SEED; otherwise pick a spot and record it. The end line must show the marks consumed (`marks` well below the count designated).
  - [x] `full-pile-crowd.json`: a pile of at most 4 cells and many digs, so stones pile up around the pit (#204's shape). The end line must show `items` well above the pile's capacity.
  - [x] `seat-slow.json`: per Q1, `{"save": "saves/12-12-seat-slow.save"}`, 3,000 ticks, no commands.
- [x] **Task 5: `scripts/tick-bench.sh BASE HEAD` (AC 4).**
  - [x] Resolve both refs to SHAs, and print them. Build each in its own scratch worktree under `mktemp -d`. **`cd` into each worktree to run `cargo build --release -p simd -j 8`, with that worktree's own `CARGO_TARGET_DIR`.** A target dir pointed at a worktree from another cwd builds the CWD's sources (memory: CARGO_TARGET_DIR builds cwd sources). Remove both worktrees on exit (`trap`).
  - [x] Probe each binary with `--bench` on a tiny file first. If it is missing, fail with `BASE <sha> has no --bench (pre-13.1); pick a base at or after 13.1's merge`.
  - [x] Per scenario: `--runs 1` alternately base, head, base, head… up to `RUNS` (default 5), so load drift hits both builds alike. A build's floor is the spread of its own means, max−min over mean, in %. The verdict is `BEYOND FLOOR` when |Δ| exceeds the larger of the two floors. Print `uptime` before and after.
  - [x] It exits 0 even on a regression; the table is the result.
- [x] **Task 6: tests and the mutation table.** NEW `crates/simd/tests/bench.rs`, driving the real binary (`env!("CARGO_BIN_EXE_simd")`) on small fixtures under `crates/simd/tests/fixtures/`. Keep it fast-tier cheap: a few hundred ticks each.
  - [x] **Determinism:** `--runs 3` gives one hash, and `state identical`.
  - [x] **Seam exercised:** the same fixture with its commands removed gives a different hash and `commands=0/0`. A dig scheduled at `at` K is applied on that tick. Build a fixture where the dig's tile changes, and assert it through the end line or a second fixture with `ticks` = K and K+1.
  - [x] **Loud failures (AC 6):** a missing save, an unknown `type`, a `quit` command, `at >= ticks`, out-of-order `at`, and a refused command each exit non-zero with the cause in stderr.
  - [x] **One run shape (AC 3):** spawn `simd 0 --scenario F`, wait before connecting, and assert the first snapshot's tick is the start tick (the hold). Read deltas until the end line, and assert its hash equals `--bench F --runs 1`'s hash. Put this one in `serve.rs` if it needs the daemon harness; that file is full-tier only, which is fine.
  - [x] **Stats unit test** in `scenario.rs`, on synthetic durations.
  - [x] `mutations/13-1.sh`, one row per decision, each killed at its own assert (memory: strengthened test needs re-mutation):
    - the bench skips command application;
    - an off-by-one tick, live versus bench;
    - the hash ignores the world (a constant);
    - p99 taken as the max;
    - the hold removed;
    - the end pause removed;
    - a refusal not failing the bench.
- [x] **Task 7: the instrument, run and RED (AC 1, 4, 5).**
  - [x] Run `--bench` on every scenario, 5 runs, release. Paste the summary lines and the end lines into the Dev Agent Record. These are the reference figures; compare `plan` to the creation table above.
  - [x] RED: make a detached throwaway commit in a scratch worktree off this branch's HEAD, with the creation's sabotage (`astar_heuristic` iterating `goals.iter().chain(goals.iter())`). Run `scripts/tick-bench.sh HEAD <that-sha>`. Expected: `plan` reads `BEYOND FLOOR` at about +4% with an identical state. Paste the table. Then show a control, `HEAD HEAD`, where every row reads `within floor`.
  - [x] README: under "At the vehicle", add the scenario seat form (Task 8's commands). Under "Test", add one line for `--bench` and one for `scripts/tick-bench.sh`.
- [x] **Task 8: the seat card (AC 7).** NEW `13-1-signoff/vehicle-card.md`, in the vehicle launch form:
  ```
  # WSL, from the repo
  simd 7451 --scenario scenarios/plan.json
  # PowerShell, from the Windows checkout
  .\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-1-plan.csv')
  ```
  Questions for Wolf:
  1. Did the run go from start to `done` with no key pressed?
  2. Does simd's `done` line hash equal the bench's hash for `plan.json`?
  3. Does `python3 scripts/bench/perf_summary.py <csv>` read the whole run?

  Push first (`scripts/push.sh --fast`) so the seat builds HEAD.
- [ ] **Task 9: gate and close.** Fast tier on every commit. ONE full `scripts/gate.sh` (run with `RUST_TEST_THREADS=1`) at PR. Paste the block. Then the cost row (`session_tokens.py --phase dev --story 13-1-scripted-runs-and-the-tick-bench`), and `phase_state.py clear`.
  - [x] Fast tier green on every commit (pre-commit hook). Every `simd` test, `serve.rs` included, green on `de3e290`'s code (34 + 4 + 80).
  - [x] Dev cost row and `phase_state.py clear` (dev close).
  - [ ] **The ONE full gate, at PR (M3-4, #212).** Deliberately not run at dev close: the rule is one full gate per story, at PR, after review and any patch pass.

### Review Findings

Code review run 1 (2026-10-21, a fresh session). Four layers ran, and every one ran `cargo 1.97.1` against its own `CARGO_TARGET_DIR`. No layer timed out or failed. The Blind Hunter ran release `--bench` on about 15 adversarial files but not `cargo test`; the other three ran `cargo test -p simd` green (34 + 4 + 80). The bench ran end to end on all 8 scenarios, and every end line and hash matched this record. Served runs matched the bench hash, and the real gui ran headless on lavapipe. **AC 7 (the seat) is not observed and stays open.** AC 4's RED and control were not re-run; `HEAD HEAD` and the pre-13.1 probe were RAN in a clone. No HIGH findings; 7 patch, 11 defer (one filed as #214), 4 dismissed.

- [x] [Review][Patch] **MED, acceptance.** `bench.rs` is not in the fast tier, and the record says it is [`scripts/gate.sh:182`]. Fast runs `cargo test -p simd --bins`, which skips the integration tests, so AC 1/2/3/6's four tests run only at the full gate. The SKIPPED banner names only `serve.rs`. Fix: add `cargo test -p simd --test bench` to the fast arm, and correct the Completion Notes claim.
- [x] [Review][Patch] **MED, acceptance + feature.** tick-bench aborts the whole table at the first scenario it cannot measure [`scripts/tick-bench.sh:78-96`]. `deep-pit`'s channel marks have about 50 ticks of slack: shifted 100 ticks early, the bench refuses at tick 1544 (RAN). `seat-slow` fails on whichever build disagrees with the save format. Either one ends the compare before `plan`, the piles or `seat-slow` are measured, which is exactly where 13.2 will hit it. Fix: print a failed row with its cause, continue, and exit 1 at the end.
- [x] [Review][Patch] **MED, edge + acceptance.** tick-bench accepts `RUNS` below 3 [`scripts/tick-bench.sh:34`]. `RUNS=1` gives 0.0% floors, so a binary against itself reads `+12.9% BEYOND FLOOR` (RAN). `RUNS=0` dies on `base_states[0]: unbound variable` after both builds (RAN). Fix: refuse a non-integer or `< 3` before building ("two samples are not a floor").
- [x] [Review][Patch] **MED, edge.** `--pause-at` with `--scenario` is silently defeated [`crates/simd/src/main.rs:133`]. On a save start, the daemon logs `already past, applying Paused now`, and then the first connect's `release()` runs the world anyway (RAN: `last tick 54718 fast4x`). Fix: refuse the combination in `parse_args`; a scenario's end pause is its freeze point.
- [x] [Review][Patch] **MED, feature.** The seat card cannot mark where the run ends, so the CSV pools the paused tail [`_bmad-output/implementation-artifacts/13-1-signoff/vehicle-card.md`]. A probe CSV read `over 87.1s` against a done line at about 53.6 s, and Q3 cannot be answered from the summary. Fix: press **F3** (the perf mark, which leaves the sim alone) when the done line prints. While in the card, two LOWs from the same layer: if the gui exits (evicted at connect), restart both simd and the gui, since the hold happens only once; and run a release `simd`.
- [x] [Review][Patch] **LOW (a silent-failure trap), feature.** `--scenario` from a debug build gives no warning [`crates/simd/src/main.rs:237-245`]. A debug daemon runs 5.34 ms per tick against fast4x's 5 ms period, with 801 ticks over 10 ms (RAN), so a seat run stretches while the hash still matches. Fix: print the bench's `DEBUG BUILD: figures are not a measurement` at the hold.
- [x] [Review][Patch] **LOW, acceptance (in a file being edited).** `big-pile` runs 5,006 ticks, and the record lists only the plan files' 10,006 [this file, Completion Notes]. Fix: one line in the deviations.
- [x] [Review][Defer] A huge `ticks` panics (`capacity overflow`) rather than failing cleanly [`crates/simd/src/scenario.rs:293`] — deferred, unreachable in practice
- [x] [Review][Defer] A zero mean makes tick-bench's awk produce NaN and read `within floor` [`scripts/tick-bench.sh:103`] — deferred, needs a sub-50 ns tick
- [x] [Review][Defer] A client `Load` after `done` still prints `abandoned` [`crates/simd/src/main.rs:297`] — deferred, cosmetic; the hash was already printed
- [x] [Review][Defer] Served scenario refusals are not logged by the daemon [`crates/simd/src/main.rs:331`] — deferred, only reachable with client commands, which the done line already flags
- [x] [Review][Defer] `--bench --runs 3` gives a port error; repeated flags are last-wins [`crates/simd/src/main.rs`] — deferred, still a loud exit 1
- [x] [Review][Defer] No test for a save that decodes but fails the live checks [`crates/simd/tests/bench.rs`] — deferred, the behaviour was RAN correct
- [x] [Review][Defer] The done line does not prove the gui watched the run [`crates/simd/src/scenario.rs:234`] — deferred, card Q3 checks the CSV on its own
- [x] [Review][Defer] Speed keys leave no trace in the done line [`crates/simd/src/main.rs:255`] — deferred, the card says not to press them
- [x] [Review][Defer] After `done`, the gui's first Space press is a no-op [`crates/gui/src/command.rs:298`] — deferred, pre-existing
- [x] [Review][Defer] The served run-shape test panics, rather than failing, on an early end, and three served behaviours are untested [`crates/simd/tests/bench.rs`] — deferred, all three RAN live
- [x] [Review][Defer] An off-map designate/cancel/remove rect is accepted silently [`crates/sim-core/src/lib.rs:3061`] — deferred, pre-existing; filed as #214

Review cost: $15.63 over 300 turns (subagents 74.1% of tokens). The patches were left as action items (Wolf), for a fresh `bmad-review-patch` session. Reaped after triage: `reap-build-caches.sh --tmp-only --force`, 9 directories, 11.7 GB (6.6 GB of free space reclaimed).

**Patch pass (2026-10-22, a fresh session, Opus 5.5): all 7 landed.** Commits: `6a7f3d6` (#4, #6), `60bc6c7` (#1), `4aabd3d` (#2, #3), `c3d6168` (mutation rows 12-13), `5bfdfdb` (#5 and the README's copy of the seat form), plus this record (#7, #1's claim). Verification, per the Epic 13 standing AC (M3-4, Wolf "yes"): the fast tier was green on every commit, and `cargo test -p simd` exited 0 (34 + 4 + 80, `serve.rs` included). The ONE full gate is still owed at PR. Rows 12-13 are KILLED, each at its own new assert (`main.rs:1085`, `bench.rs:297`). Rows 1-11 were not re-run; the audit shows they all still apply.

| item | written for | then tested | pre-existing-state fixture |
|---|---|---|---|
| #1 bench.rs in fast | the fast arm runs `--test bench` | the OLD gate under a bench.rs-only sabotage: GREEN (the hole). The NEW gate under it: RED at `cargo test simd bench.rs`, 2 of 4 failed (`no marks=`) | committed tree `6a7f3d6` with `scenario.rs:334` `marks=` → `mark=` (no mutation row targets that line) |
| #2 continue past a failure | a base-side refusal | RED first: the unfixed script stopped before `plan`. Fixed: base refusal, base missing save, a **head-only** save refusal and a **head-only** divergence each print a FAILED row with the deepest cause; `plan` is still measured; exit 1 | `a-deep-pit-early.json` (committed deep-pit marks shifted 100 ticks early, the review's repro, refused at 1544); committed `seat-slow.json` + its save, against throwaway `45ddd5d` (refuses every save, i.e. format rot on one side); throwaway `d54f89c` (hash takes the clock) |
| #3 RUNS ≥ 3 | RUNS=1 and RUNS=0 | 0, 1, 2, `abc`, `3x` each exit 2 before any build; RUNS=3 runs | the review's RAN RED (RUNS=1 +12.9%, RUNS=0 unbound), not re-run here |
| #4 `--pause-at` + `--scenario` | a save start already past the tick | both flag orders refused; a seed start is refused too (the ruling, though it used to work); plain `--pause-at` still serves; mutation row 12 | committed `scenarios/seat-slow.json` (save at tick 53,759), the review's repro |
| #5 seat card | the paused tail pooled into the CSV | doc only. `perf_summary.py:204` prints a `MARKED` line for an F3 frame. **Unproven live: Wolf's seat answers it** | none (a card) |
| #6 debug warning at the hold | a debug daemon | a release daemon prints nothing on stderr, a debug one prints the line; the served test asserts presence == `cfg!(debug_assertions)`; mutation row 13. The test's release branch was not run (`cargo test --release` was not run) | committed `scenarios/plan.json` |
| #7 big-pile 5,006 | the record | n/a (doc; checked against `big-pile.json`'s `ticks` 5006) | n/a |

No row is REWORK (this was the first patch round).

Patch-pass cost: $6.51 over 140 turns. One avoidable cost: a throwaway sabotage commit in a scratch worktree ran the pre-commit gate (the hooks path is shared), which took 9 minutes and refused the commit. Throwaway commits need `--no-verify`.

**Found during the patch pass (not patched; for Wolf):** two **different** builds read `plan` at **−6.1%** and then **−5.4% BEYOND FLOOR** (floors 0.2–1.4%). Their only change was in the save loader, which `plan` never runs (HEAD vs throwaway `45ddd5d`, RUNS=3, twice). `tick-bench` builds once per SHA, so the `HEAD HEAD` control timed ONE binary for both columns, and it never measured how far two different builds differ when their code is the same. The ~3.5% "resolution" in this record is therefore a same-binary figure. A layout or codegen shift between builds can read as a regression, or hide one.

Dismissed (4): `STATE DIVERGED` shows only as the stderr error (it is printed, and it exits 1, so AC 1 holds); the probe message says `<sha>`, not `BASE <sha>` (head can be pre-13.1 too); tick-bench reads the working tree's scenarios (documented in its header, by design); `DEBUG BUILD` prints once per run (Task 2 asks for that).

**Code review run 2 (2026-10-22, a fresh session), diff `9e86180..aba37d9` (the patch pass).** Delta against run 1: **NEW 9** (0 HIGH, 1 MED, 8 LOW), **REWORK 0**, and closure rows **7/7 CLOSED** (acceptance audited every row; rows 12-13 were re-killed by hand at `main.rs:1085` and `bench.rs:297` by acceptance and edge, independently; `audit-mutations.py` 950/950 literals match). **The stopping rule FIRES: no round 3.** The next spend goes to the live gate (AC 7, Wolf's seat) and the one full gate at PR. Three layers ran, each with `cargo 1.97.1` and its own `CARGO_TARGET_DIR`, and none timed out. The **Blind Hunter was not launched**: its territory (sim-core, client-core, gui's pure predicates) has no line in this diff, so there was nothing to give it. That was a zero-scope decision, not a failure or timeout. What was RAN: tick-bench FAILED rows on each side with `plan` still measured and exit 1 (feature `d07cbc0`, acceptance `45ddd5d`); `scripts/gate.sh --fast` GREEN in a clone, with `cargo test simd bench.rs ok 3s`; and a release scenario daemon plus the real headless gui, `done at tick 10006 state=2479b66a35ac887e commands=50/50`. F3 reaching the CSV was proven by a scratch test in the feature layer's clone (`marked frame 2`, `mark=1`), not by a committed one. **AC 7 is not observed and stays open.** `serve.rs` was not run.

- [x] [Review][Patch] **MED, acceptance. RULED (b) FIX NOW (Wolf, 2026-10-22: "2"). Cross-build noise leaves the RED's evidence unvalidated, and the record still states a ~3.5% resolution.** The RED (`de3e290` vs `8da0db9`, Debug Log) compared two builds. The patch pass measured two builds with identical plan source at −6.1% and −5.4% BEYOND FLOOR. Against that, the RED's +3.5% does not show the instrument sees the sabotage, though its agreement with the creation probe's +4% suggests it does. Three places still present the same-build floor as the threshold for a two-SHA compare: the Debug Log's "Resolution, honestly" line ("about 3.5% is the edge"), the tick-bench header (`scripts/tick-bench.sh:21-22`) and README Tick cost. No AC is violated, since AC 4 says the verdict is informational. It is a latent false-verdict trap: 13.2 will measure with this instrument. Acceptance's repro was inconclusive (load 11-23, floors 30%+). Options: (a) qualify the three claims, mark the RED "unvalidated across builds", and file an issue for a cross-build floor; (b) build a cross-build control into tick-bench now. **Candidate fix (UNTESTED):** tick-bench also builds base a second time with a codegen-only perturbation (e.g. `RUSTFLAGS="-C metadata=tick-bench-control"`, which leaves the source alone and changes symbol hashes and layout). It runs that build interleaved as a third column, and the verdict is BEYOND FLOOR only when |Δ| exceeds max(each floor, \|control Δ\|). First check that the perturbation changes the binary and reproduces a few-% shift (the −6.1%/−5.4% pair HEAD vs `45ddd5d` is the reference). If it does not, pick another perturbation and record the choice. Still qualify the Debug Log resolution line, the header (`scripts/tick-bench.sh:21-22`) and README Tick cost. Re-run the RED and `HEAD HEAD` under the new verdict and record the result honestly, even if the +3.5% RED then reads `within floor`. Cost: a third build and 50% more bench time.
- [x] [Review][Patch] **LOW, acceptance (a line this patch rewrote).** The gate.sh comment still says `--bins, 18 tests in 0.41s`. RAN: 34 tests, 3.83 s at 2 threads [`scripts/gate.sh:180`]
- [x] [Review][Patch] **LOW (silent-failure trap: a misreported cause), feature.** A panicking scenario's FAILED cause is the backtrace hint. RAN: a `ticks` 2^62 file (exit 101) reads `note: run with RUST_BACKTRACE=1 …`. Prefer the `panicked at` line [`scripts/tick-bench.sh:90-93`]
- [x] [Review][Patch] **LOW (silent-failure trap: a misreported side), acceptance + edge.** A FAILED row says `on base` for a file that every build refuses, because `break 2` never tries head. RAN: `a-deep-pit-early`. The same applies to divergence, where only base is named when both diverge. The label should not claim a side it did not compare [`scripts/tick-bench.sh:93`, `:105-110`]
- [x] [Review][Patch] **LOW, acceptance + feature (README, which this patch edited).** README Tick cost says nothing about the RUNS ≥ 3 refusal or the FAILED-row-then-exit-1 behaviour. Its seat form runs `target/release/simd` with no `cargo build --release -p simd` step, which the card has [`README.md:192`, `:386-395`]
- [x] [Review][Defer] An over-long `RUNS` digit string passes validation: `[ -lt ]` errors loudly and the bench runs on [`scripts/tick-bench.sh:40`] — deferred, it prints an integer-expression error, so it is not silent
- [x] [Review][Defer] perf_summary's percentiles and `over Ns` still pool the paused tail after `done`, and no tool cuts at the F3 mark [`scripts/bench/perf_summary.py:121-147`] — deferred, pre-existing tool; the card discloses it
- [x] [Review][Defer] The F3 → CSV `mark=1` hop has no committed test, though card Q3 now depends on it [`crates/gui/src/perf.rs:265`, `ingest.rs:902`] — deferred, pre-existing hop, proven by a scratch test this review

Review run 2 cost: $12.72 over 242 turns (subagents 72.5% of tokens). Patches left as action items (Wolf "2"), for a fresh `bmad-review-patch` session. Reaped after triage: `reap-build-caches.sh --tmp-only --force`, 10 directories, 54.8 GB (30.0 GB of free space reclaimed).

**Patch pass 2 (2026-10-22, a fresh session, Opus 5.5): all 5 landed.** Commits: `8e14f7d` (the control, plus the two FAILED-row LOWs, which share its loop), `9ca72d2` (gate.sh comment), `8c44d89` (README), and this record. Verification follows Wolf's ruling at the checkpoint: the fast tier was green on all three commits, and the ONE full gate is still owed at PR. No Rust source changed in this pass.

*Choosing the perturbation (the ruling: "first check that it reproduces the shift; if not, pick another and record the choice").* Same-source rebuilds of `60bc6c7`, interleaved with the reference pair `60bc6c7` vs `45ddd5d` on `plan` (RUNS=7; the readable passes had floors ≤ 4%):

| build of `60bc6c7` | Δ vs base, pass 1 / pass 2 |
|---|---|
| reference: `45ddd5d` (identical plan code; 200 KB smaller, because the save decoder goes dead) | −5.3% / −5.7%, −4.8% |
| `-C metadata=tick-bench-control` (the candidate) | −1.6% / −1.6%, −1.8% |
| `-C metadata=xyzzy` | −2.1% |
| `-C llvm-args=-align-all-functions=6` | −2.3% |
| `-C codegen-units=8` | −1.6% |
| `-C link-arg=-Wl,--shuffle-sections=*=1` (**chosen**) | −3.0%, −2.8% |
| `--shuffle-sections=*=2` | −2.8%, −3.4% |

Every perturbation changes the binary and moves the mean beyond the same-binary floor, but none reproduces the reference's ~5%. `--shuffle-sections` (rust-lld is the linker on 1.97.1) came closest, and it randomises function order, which is the mechanism rather than a proxy. **The control is therefore a LOWER bound on cross-build noise**, and the header and README say so. A guard refuses a control that is byte-identical to base.

*Re-run under the new verdict* (RUNS=5, `BEYOND FLOOR` needs |Δ| > max(three floors, |control|)):

```
REFERENCE 60bc6c7 45ddd5d (identical plan code)
plan                       0.1892     0.1788    -5.5%    -3.4%       1.6%       1.7%       1.0%  same   BEYOND FLOOR
RED de3e290 8da0db9 (load 5.5 falling to 1.6)
scenario                  base_ms    head_ms    delta  control base_floor head_floor  ctl_floor  state  verdict
big-pile                   1.0528     1.0536    +0.1%    -0.4%       1.7%       1.9%       1.8%  same   within floor
deep-pit                   0.0376     0.0385    +2.4%    +0.6%       3.2%       5.2%       4.8%  same   within floor
full-pile-crowd            0.0306     0.0308    +0.7%    +1.2%       1.6%       0.6%       2.9%  same   within floor
plan-compact-piles         1.3876     1.4176    +2.2%    +1.1%       1.6%       0.8%       1.6%  same   BEYOND FLOOR
plan-far-piles             0.2175     0.2222    +2.2%    +4.0%       1.7%       0.9%       1.3%  same   within floor
plan-spread-piles          1.6830     1.6788    -0.3%    -0.2%       1.9%       1.7%       1.5%  same   within floor
plan                       0.1850     0.1912    +3.3%    +3.0%       0.6%       3.0%       1.8%  same   BEYOND FLOOR
seat-slow                  0.4395     0.4562    +3.8%    +1.4%       3.7%       3.2%       2.7%  same   BEYOND FLOOR
CONTROL HEAD HEAD (8e14f7d)
big-pile                   1.0462     1.0416    -0.4%    -2.7%       1.9%       2.1%       1.7%  same   within floor
deep-pit                   0.0366     0.0367    +0.2%    +1.5%       5.5%       6.8%       7.3%  same   within floor
full-pile-crowd            0.0298     0.0297    -0.3%    +0.9%       4.0%       3.4%       5.7%  same   within floor
plan-compact-piles         1.3907     1.3790    -0.8%    -2.6%       2.7%       1.4%       1.8%  same   within floor
plan-far-piles             0.2206     0.2214    +0.4%    -3.6%       4.5%       2.0%       1.6%  same   within floor
plan-spread-piles          1.6781     1.6780    -0.0%    -2.6%       2.4%       1.6%       1.3%  same   within floor
plan                       0.1878     0.1872    -0.3%    -3.8%       1.6%       1.4%       1.1%  same   within floor
seat-slow                  0.4734     0.4744    +0.2%    -5.7%       1.8%       0.9%       0.9%  same   within floor
```

**What this says, honestly:**
- **The reference false positive survives.** Two builds of identical plan code still read −5.5% BEYOND FLOOR, because the control read −3.4%. A single control cannot bound a spread it samples once.
- **The RED is not validated across builds.** `plan` +3.3% and `seat-slow` +3.8% read BEYOND FLOOR by 0.3 and 0.1 points, and the `HEAD HEAD` control column alone shows layout moving `plan` by −3.8% and `seat-slow` by −5.7%. Its agreement with the creation probe's +4% is the only support.
- **The cross-build resolution on this devpod is about 6%, not 3.5%.** A change under that, between two builds, is not distinguishable from layout.
- **The control decides one row.** Removing the control term from the verdict (recomputed from the table above) flips exactly the RED's `plan-far-piles`, +2.2% against floors ≤ 1.7%, from BEYOND FLOOR to within floor (control +4.0%). That row is the sabotage, so the term cost a likely-true positive there. It is live in both directions.
- **For Wolf (not patched, beyond the ruling):** the tool that would close the gap is layout randomisation on BOTH sides, with several shuffle seeds per build so that the mean is taken over layouts (Stabilizer-style). That trades the third column for K builds per side. Worth an issue if 13.2's spike needs changes under ~6%.

| item | written for | then tested | pre-existing-state fixture |
|---|---|---|---|
| #1 cross-build control (MED) | two builds of identical code reading BEYOND FLOOR | the reference pair under the NEW verdict: STILL BEYOND (−5.5% vs control −3.4%), recorded above. The RED and `HEAD HEAD` were re-run. The control term flips the RED's far-piles row. The guard was mutated (control flags emptied → `byte-identical to base`, exit 1) | throwaway `45ddd5d` vs its parent `60bc6c7` (the patch pass's −6.1%/−5.4% pair); the throwaway RED `8da0db9` vs `de3e290` |
| #2 gate.sh comment (LOW) | the stale "18 tests in 0.41s" | n/a (comment; 34 = the `--bins` count in this record) | n/a |
| #3 panic cause (LOW) | a panicking scenario | RED on the unfixed script: `note: run with RUST_BACKTRACE=1 …`. Fixed: `panicked at …/raw_vec/mod.rs:28:5: capacity overflow` | `huge.json`, committed `deep-pit.json` with `ticks` 2^62 (the review's repro) |
| #4 the side a FAILED row claims (LOW) | a file every build refuses, and divergence on more than one side | RED on the unfixed script: `on base` for `a-deep-pit-early`. Fixed: `on every build`. Also: head-only refusal (`60bc6c7 45ddd5d` seat-slow → `on head`); base-side refusal (`45ddd5d 60bc6c7` → `on base, control`); head-only divergence (`60bc6c7 d54f89c` → `on head: STATE DIVERGED`); all sides diverging with different hashes (`d54f89c d54f89c` → three per-side causes); plus a good row measured beside each | `a-deep-pit-early.json` (committed deep-pit marks 100 ticks early, refused at 1544); committed `seat-slow.json` + save; throwaways `45ddd5d` and `d54f89c` |
| #5 README (LOW) | the missing RUNS/FAILED/build-step lines | doc only; checked against the script's own messages | n/a |

No row is REWORK; each closes a new round-2 finding. One defect was caught in-pass: a `[ ] && echo` as the last command in a `$(for …)` made `set -e` exit with no output. It was fixed with `if` before the commit.

## Dev Notes

### Scope guardrails (do NOT)

- **Do not** time or optimise anything in `sim-core`. This story builds the instrument; 13.2 uses it. No sim behaviour changes, and every scenario test keeps its pins.
- **Do not** add a pass/fail cost threshold to the gate. The bench and compare are run on demand.
- **Do not** add a sim-tick column to the gui perf log or any gui change. The seat run reuses `--perf-log` as it is.
- **Do not** build a scenario recorder, a generator crate or a scenario DSL. The files are hand-authored JSON, and a one-off script that converts `plan-10-10-1.json` stays uncommitted.
- **No new dependency.** `serde_json` and `anyhow` are already in `simd`, and FNV is about 5 lines.

### What already exists (build on it)

- The live loop's command arms and scheduled-speed pattern (`main.rs:168-271`) and save validation (`load_world_from`, `main.rs:344`).
- The plan, as literals (`scenario.rs:4904-5000`) and as JSON (`12-12-signoff/plan-10-10-1.json`), with snapshot tick 6.
- `World` getters for the end line: `designations` `items` `zones` `goals` `camp_origin` (`lib.rs:2961-3429`).
- The seat side: `--perf-log` (`gui/src/perf.rs`, one CSV row per frame) and `scripts/bench/perf_summary.py`.
- The spawn harness for daemon tests (`simd/tests/serve.rs:49-67`).

### Key decisions & traps

- **Judge by the mean.** Same-build max/p99 swing more than a real 4% change (creation table), so `BEYOND FLOOR` uses the means. Print p99 and max anyway, since they find spikes.
- **Two samples are not a floor.** The default is 5 runs, and a `--runs 1` summary must say `no floor` (memory: delta needs a noise floor).
- **Compare interleaved, never back to back.** Load drifts between batches, and the gate or a mutation run on the same machine moves the floor. Print `uptime`. Never bench while `scripts/mutate.sh` runs (memory: mutation run poisons concurrent builds).
- **`main` has no `--bench` until this story merges.** The story's own RED compares HEAD with a sabotage commit, not with `main`. The probe failure message names the reason.
- **One apply function, or AC 3's hash check is meaningless.** Live and bench agree only if both apply the same command at the same tick before the same step. An off-by-one shows up only as a hash mismatch, and the mutation row exists for that.
- **A committed save rots** at the next save-format change (12.11 refused all older saves). The bench names the file and the decode error, so the scenario is re-captured rather than skipped.
- **Seat speed is not sim behaviour.** Speed changes only the period (`main.rs:273`), so a fast4x seat run and an unthrottled bench reach the same state. That holds unless the hold or end logic touches the world; they must only change `speed`.

### Project Structure Notes

- UPDATE `crates/simd/src/main.rs`: `parse_args` (an args struct); `apply_sim_command` extracted; scenario hold, apply and end in `tick`; `--bench` entry; `load_world_from` returns its error.
- NEW `crates/simd/src/scenario.rs`: the file struct, load and validate, the due-command iterator, stats, FNV, and the bench run.
- NEW `crates/simd/tests/bench.rs` and `crates/simd/tests/fixtures/*.json`; possibly UPDATE `crates/simd/tests/serve.rs` (the AC 3 run-shape test).
- NEW `scenarios/*.json`, and (per Q1) `scenarios/saves/12-12-seat-slow.save`.
- NEW `scripts/tick-bench.sh`.
- NEW `_bmad-output/implementation-artifacts/mutations/13-1.sh` and `13-1-signoff/vehicle-card.md`.
- UPDATE `README.md`, two short additions.
- Daemons write `frostvein.save` into their cwd, so run test and probe daemons from scratch directories (memory: orphan daemon answers the next run; reap them by PID).

### Previous story intelligence

- 12.12's tick tables came from uncommitted scratch probes, and spread5 could not be reproduced. Every figure this story publishes comes from a committed scenario and a committed command.
- Build probes in scratch worktrees, at most about 4 release builds at a time; the 32-core / 23 GB devpod OOMs above that.
- The full gate takes about 3,800 s at `RUST_TEST_THREADS=1`, and it runs once, at PR (M3-4).

### References

- `epic-12-retro-2026-10-21.md` §4.2 and §8; issues #210, #209, #50 (scenario-driven gui sessions; this advances it) and #179.
- `epics.md`: Epic 13, Story 13.1.
- `12-12-the-first-goals.md:679-690` and `:820-836` (the scratch-probe tick tables); `:511` (spread5 not reproducible).
- `crates/simd/src/main.rs:63-127` (args and main), `:155-314` (the tick loop), `:344` (save validation).
- `crates/protocol/src/lib.rs:124-212` (`Speed`, `Command`, `Rect`).

## Verification

All commands run from the repo root, in release, from a scratch cwd for any daemon.

```bash
# 1. the bench, every scenario
cargo build --release -p simd
for f in scenarios/*.json; do target/release/simd --bench "$f" --runs 5; done
# observe: per file, 5 run lines, one end line, one summary with "state identical across 5 runs";
#          plan.json mean within ~10% of the creation's 0.175 ms on this devpod; commands=K/K, 0 refusals

# 2. RED, then control
#    (throwaway sabotage commit in a scratch worktree off HEAD, astar_heuristic iterating its goals twice)
scripts/tick-bench.sh HEAD <sabotage-sha>   # observe: plan BEYOND FLOOR, about +4%, both builds' states identical
scripts/tick-bench.sh HEAD HEAD             # observe: every row "within floor"

# 3. loud failure
target/release/simd --bench crates/simd/tests/fixtures/<missing-save>.json; echo "exit $?"   # observe: non-zero, names the file

# 4. one run shape (also the serve test): from a scratch dir
target/release/simd 7461 --scenario <repo>/scenarios/plan.json &   # observe: "holding tick 0 until a client connects"
target/release/tui 7461                                             # observe: tick climbs; simd prints "done at tick 10006 state=<hash>"
# the <hash> equals bench 1's plan hash. Reap the daemon by PID.
```

Exit 0 is not a result: read the lines. The seat half of AC 7 is Wolf's (Task 8).

## Dev Agent Record

### Agent Model Used

claude-opus-5-5 (Opus 5.5), building directly in the creation session, with no subagents (Wolf's pick, 2026-10-21).

### Debug Log References

- **Reference figures** (release, `de3e290`, 5 runs each, load average 3.1–3.8 falling to 1.7):

  | scenario | mean ms (5 runs) | spread | end line |
  |---|---|---|---|
  | plan | 0.1858–0.1895 | 2.0% | tick=10006 marks=0 items=58 zones=78 goals=12/10/1 |
  | plan-far-piles | 0.2214–0.2377 | 7.2% | zones=90 goals=12/10/1 |
  | plan-spread-piles | 1.7213–1.8077 | 4.9% | zones=203 goals=12/2/0 |
  | plan-compact-piles | 1.4681–1.5264 | 3.9% | zones=203 goals=12/6/1 |
  | big-pile | 1.0997–1.1527 | 4.7% | tick=5006 marks=14 items=32 **zones=518** goals=0/2/0 |
  | deep-pit | 0.0376–0.0448 | 18.1% | tick=3000 **marks=0** items=28 zones=22 |
  | full-pile-crowd | 0.0306–0.0360 | 16.4% | **items=46 zones=4** |
  | seat-slow | 0.4488–0.4519 | 0.7% | tick=56759 marks=1 items=129 zones=222 goals=38/12/1 |

  Every scenario reported `state identical across 5 runs`. `plan`'s state `2479b66a35ac887e` is the creation probe's hash, computed by independent code. big-pile zones exactly 12.12's 518 cells.
- **RED** (`scripts/tick-bench.sh de3e290 8da0db9`, RUNS=5; `8da0db9` is a throwaway detached commit in which `astar_heuristic` iterates `goals.iter().chain(goals.iter())`):
  ```
  scenario                  base_ms    head_ms    delta base_floor head_floor  state  verdict
  plan                       0.1834     0.1898    +3.5%       1.6%       0.7%  same   BEYOND FLOOR
  big-pile                   1.0600     1.0633    +0.3%       1.6%       1.5%  same   within floor
  seat-slow                  0.4429     0.4579    +3.4%       2.1%       2.0%  same   BEYOND FLOOR
  ```
  big-pile does not move. Its cost is not the heuristic, which matches 12.12's "the floods are not its cost".
- **Control** (`scripts/tick-bench.sh HEAD HEAD`, RUNS=5, every scenario): every row reads `within floor` with `same` state. Deltas were −0.4% to +0.3%, and floors 0.5%–6.3% (deep-pit 6.3/3.7, seat-slow 3.7/4.9).
- **Resolution, honestly:** about 3.5% is the edge on this devpod **for ONE binary against itself**. Seat-slow's RED +3.4% cleared that run's 2.0% floor, but would not have cleared the control run's 4.9%. The two cheap scenarios (~0.04 ms/tick) swing up to 18% between runs, so only large changes read there. **QUALIFIED at review-patch run 2:** that figure is a same-binary floor. Between two builds, layout alone moves `plan` by 3–6% (cross-build control below), so this RED is **unvalidated across builds**: its +3.5% sits inside that band.
- **Live run** (release, `simd 7461 --scenario scenarios/plan.json`, a Python client that connected after 3 s): `holding tick 0`; snapshot tick 0; `running at Fast4x`; `done at tick 10006 state=2479b66a35ac887e commands=50/50` after 51.2 s. The same hash as the bench.
- **Pre-13.1 base:** `scripts/tick-bench.sh main HEAD` → `tick-bench: b8b7a5c… has no --bench (pre-13.1)`, exit 1. The first version of this probe MISSED: under `pipefail` the binary's own exit 1 hid grep's match, and the script went on to fail later with a raw port error. Fixed by capturing the output before grepping.
- **Mutations:** `mutations/13-1.sh` 11/11 KILLED, each at its own assert. Rows 1/2 at the oracle state (`bench.rs:112`); 3 at the done-hash; 4 at the no-command state; 5 in the stats unit test; 6 at the held snapshot tick; 7 at the missing `running` line; 8 at the end-tick pause; 9 and 11 at exit 0; 10 at the stderr cause. The five re-pointed 3.1/12.1 rows also ran, 5/5 KILLED.

### Completion Notes List

- **One apply path.** The live loop's five sim arms became `apply_sim_command` (`main.rs`). The loop and both scenario modes call it, so a scenario run IS the daemon's code path, and the three hashes agree (probe, bench, served run).
- **`--bench`** times `World::step` only. It builds and encodes the delta each tick outside the timed window, so the dirty set drains as in the daemon. Its output follows the story's contract. A refused command exits 1 and names the command and refusal; a debug build warns on every run. A one-run summary says `1 run: no floor` and makes no identity claim.
- **`--scenario`** holds the start tick until the first client connects (Q2), runs at the file's speed, pauses on the end tick and prints the hash once. Client sim commands are counted into the done line. A client `Load` abandons the scenario loudly.
- **Loading.** The live `Load`'s validation became `try_load_world_from`, which returns the cause; `load_world_from` keeps its old print-and-`None` contract for the live path and the existing tests. A scenario refuses `set_speed`/`save`/`load`/`quit`, out-of-order or late `at`, unknown fields, and invalid rects (the client path's own `rect_is_valid`).
- **Scenarios** (Task 4, all with 0 refusals). Three deviations from the task text, each recorded in the file's `about`:
  - The plan-based files run 10,006 ticks, so the end tick matches 12.12's 10,006; the task said 10,000.
  - `big-pile` runs 5,006 ticks for the same reason (5,000 after the plan's snapshot tick 6); the task said 5,000.
  - `plan-compact-piles` is a CLUSTER of flat 5×5 squares west of the camp. The camp is a hollow whose only flat ground the plan's own piles already hold, so no five 5×5 piles fit near the fire.
  - `big-pile` is one drag of seven single-level rects, because a rect must be single-level (`rect_is_valid`).

  `deep-pit` marks 12.10b's staircase levels on the ticks the level above opened (0/420/1180/1644), measured by a throwaway probe that was not committed. Wolf's save is committed under `scenarios/saves/` (Q1).
- **Deviation: `serde` added to `simd`.** This is the workspace's own `serde` with `derive` (already used by sim-core and protocol); `Cargo.lock` gains one edge, not a crate. It is the story's "no new dependency" in spirit, but is recorded here because the story said none.
- **Deviation: the run-shape test lives in `bench.rs`, not `serve.rs`.** It needs no shared harness. CORRECTED at the patch pass: this said it ran in the fast tier on every commit, which was false, because the fast arm ran `cargo test -p simd --bins`, and that skips integration tests. Since `60bc6c7` the fast arm also runs `--test bench` (~2 s).
- **Re-pointed:** five mutation rows (3.1 ×4, 12.1 ×1) followed the sim arms into `apply_sim_command`, and ran KILLED.
- **Not done here:** the full gate (Task 9, at PR per M3-4) and the seat (AC 7, Wolf's; card `13-1-signoff/vehicle-card.md`).

### File List

- `crates/simd/src/main.rs` (UPDATE): `Args` and `parse_args`; `apply_sim_command`; scenario hold, apply and end in `tick`; `try_load_world_from`; arg tests.
- `crates/simd/src/scenario.rs` (NEW)
- `crates/simd/Cargo.toml`, `Cargo.lock` (UPDATE): `serde.workspace`
- `crates/simd/tests/bench.rs` (NEW), `crates/simd/tests/fixtures/one-dig.json` (NEW)
- `scenarios/{plan,plan-far-piles,plan-spread-piles,plan-compact-piles,big-pile,deep-pit,full-pile-crowd,seat-slow}.json` (NEW), `scenarios/saves/12-12-seat-slow.save` (NEW)
- `scripts/tick-bench.sh` (NEW)
- `README.md` (UPDATE): the scripted seat run, and Tick cost
- `_bmad-output/implementation-artifacts/mutations/13-1.sh` (NEW); `mutations/3-1-give-the-order.sh` and `mutations/12-1.sh` (UPDATE, re-pointed rows)
- `_bmad-output/implementation-artifacts/13-1-signoff/vehicle-card.md` (NEW)
- `_bmad-output/implementation-artifacts/13-1-scripted-runs-and-the-tick-bench.md`, `sprint-status.yaml` (UPDATE)

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-22 | Review-patch run 2 (Opus 5.5): all 5 landed (`8e14f7d`, `9ca72d2`, `8c44d89`). tick-bench gains a cross-build control: base rebuilt with `--shuffle-sections`, chosen over `-C metadata`, which read about −1.7%. Under the new verdict the identical-code reference pair still reads −5.5% BEYOND FLOOR (control −3.4%). The RED reads BEYOND by at most 0.3 points and is unvalidated across builds, and the cross-build resolution is about 6%. FAILED rows name every failed build; a panic shows its message. Fast tier green on each commit; the full gate is owed at PR. Status stays in-progress (AC 7). |
| 2026-10-22 | Code review run 2 (fresh session), diff `9e86180..aba37d9`: closure 7/7 CLOSED, 0 REWORK; 0 HIGH, 5 patch (MED cross-build control, ruled fix now; 4 LOW), 3 defer. Stopping rule fired: no round 3. Patches left as action items. Status stays in-progress (AC 7 and the PR full gate are open). |
| 2026-10-22 | Patch pass (fresh session, Opus 5.5): all 7 review patches landed (`6a7f3d6` `60bc6c7` `4aabd3d` `c3d6168` `5bfdfdb`), each with its both-sides line; rows 12-13 KILLED; fast tier + `cargo test -p simd` green. Found: two builds with the same `plan` code read −5.4% to −6.1% apart, so the same-binary control understates the noise. Full gate still owed at PR; AC 7 still Wolf's. |
| 2026-10-21 | Dev done (Opus 5.5, directly): `simd --bench` / `--scenario` through one `apply_sim_command`, eight scenarios plus the seat save, `scripts/tick-bench.sh`. Every scenario is identical across 5 runs; plan state `2479b66a35ac887e` = the probe = the live run. RED +3.5% BEYOND FLOOR, control all within floor; resolution about 3.5%. 13-1.sh 11/11 KILLED, 5 re-pointed rows KILLED. Status review. The full gate is owed at PR (M3-4), and the seat (AC 7) is Wolf's. |
| 2026-10-21 | Task 0 ruled by Wolf: Q1 (a) commit the seat save under `scenarios/saves/`; Q2 hold the start tick until the first client connects. |
| 2026-10-21 | Story created on `b8b7a5c` (Opus 5.5). Creation probe: determinism holds (identical hashes ×5); the plan's mean 0.1744–0.1758 ms; the seat save loads (0.41 ms); RED +4% clears the floor in the mean, not in the max. Task 0 Q1 (commit the save) and Q2 (hold until connect) await Wolf. |
