---
baseline_commit: b8b7a5c
model: claude-opus-5-5  # session default; Wolf picked Opus 5.5 to IMPLEMENT too (2026-10-21), no Sonnet subagents
---

# Story 13.1: Scripted Runs and the Tick Bench

Status: review

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
- **Resolution, honestly:** about 3.5% is the edge on this devpod. Seat-slow's RED +3.4% cleared that run's 2.0% floor, but would not have cleared the control run's 4.9%. The two cheap scenarios (~0.04 ms/tick) swing up to 18% between runs, so only large changes read there.
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
  - `plan-compact-piles` is a CLUSTER of flat 5×5 squares west of the camp. The camp is a hollow whose only flat ground the plan's own piles already hold, so no five 5×5 piles fit near the fire.
  - `big-pile` is one drag of seven single-level rects, because a rect must be single-level (`rect_is_valid`).

  `deep-pit` marks 12.10b's staircase levels on the ticks the level above opened (0/420/1180/1644), measured by a throwaway probe that was not committed. Wolf's save is committed under `scenarios/saves/` (Q1).
- **Deviation: `serde` added to `simd`.** This is the workspace's own `serde` with `derive` (already used by sim-core and protocol); `Cargo.lock` gains one edge, not a crate. It is the story's "no new dependency" in spirit, but is recorded here because the story said none.
- **Deviation: the run-shape test lives in `bench.rs`, not `serve.rs`.** It needs no shared harness, and in the fast tier it runs on every commit (~2 s).
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
| 2026-10-21 | Dev done (Opus 5.5, directly): `simd --bench` / `--scenario` through one `apply_sim_command`, eight scenarios plus the seat save, `scripts/tick-bench.sh`. Every scenario is identical across 5 runs; plan state `2479b66a35ac887e` = the probe = the live run. RED +3.5% BEYOND FLOOR, control all within floor; resolution about 3.5%. 13-1.sh 11/11 KILLED, 5 re-pointed rows KILLED. Status review. The full gate is owed at PR (M3-4), and the seat (AC 7) is Wolf's. |
| 2026-10-21 | Task 0 ruled by Wolf: Q1 (a) commit the seat save under `scenarios/saves/`; Q2 hold the start tick until the first client connects. |
| 2026-10-21 | Story created on `b8b7a5c` (Opus 5.5). Creation probe: determinism holds (identical hashes ×5); the plan's mean 0.1744–0.1758 ms; the seat save loads (0.41 ms); RED +4% clears the floor in the mean, not in the max. Task 0 Q1 (commit the save) and Q2 (hold until connect) await Wolf. |
