---
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
baseline_commit: ce69409a2d33f5734d2d35c946bfa2bb30407e56
---

# Story 13.6: Two Hundred on 256²

Status: in-progress

## Story

As the boss,
I want the crew size and the map size to be world settings, a fresh game on a 256² valley, and a committed crew-200 scenario that does real work,
so that every M4 story is measured at the target scale from the first one on, not checked at the end.

## Branch: off local `main`, which is ahead of `origin/main`

Branch `story-13-6-two-hundred-on-256` was cut from local `main` `e8f4a1f9`. It carries three unpushed record commits (`3319e68c` 13.4 done-mark, `ba0b91a0` M4 PRD, `e8f4a1f9` M4 epics) and the uncommitted sprint-planning board (13-6..13-21 keys). These ride into this PR, as `380fd789` did for 13.4. Epic 13's standing ACs bind: M3's ACs 1–8 with **one full gate per story, at PR**, plus **M4 standing ACs 10–18** (`epics.md` Epic 13).

**Wire diff: none.** `Snapshot.dims` already carries the map size (`protocol/src/lib.rs:280`), and positions already travel in the deltas. **Save format: no new field.** `SaveState` already holds `dims` and every dwarf, so the crew size is the saved dwarf list. Keep it that way: the state hash covers the save encoding, so a new saved field moves every committed scenario's hash and breaks AC 4.

## Task 0 — Wolf rules before dev

Each question has a recommendation. The ACs and Tasks are written for it, and a different answer amends the ACs it names.

- **Q1. #187 still reproduces on `main`, so it becomes its own story (ruled 2026-10-28). Where does it go?** Readings are in *Found at creation* below. 13.4 traced its loops as #218's shapes A1–A3 (full pile, refused cut).
  - (a) **Recommended:** one new story covering #187 and #218, placed after 13.18 (the other movement-debt stories). That makes 17 new stories against the 12–16 cap, so the stretch is on the record.
  - (b) A #187-only story, with #218 left open.
  - (c) Fold #187 into 13.18 (stones seal two miners), which holds at 16. **← RULED 2026-10-30.** Q2–Q5 ruled (a). See the Change Log.

  The evidence comment for #187 is drafted in `13-6-signoff/issue-187-comment.md`. Posting it needs Wolf's OK, because the tool blocked the GitHub write at creation.
- **Q2. What does a scenario `start` with no `dims`/`crew` mean?**
  - (a) **Recommended: 128×128×32 and crew 5,** the world every committed file was recorded on. The 11 committed files stay byte-for-byte unchanged, `main`'s `simd` can still read them, and `tick-bench.sh main HEAD` measures AC 4 on every old row.
  - (b) The fresh-game default (256², 5), with explicit `"dims":[128,128,32],"crew":5` written into every committed file. `main`'s `simd` then refuses all 11 files (`deny_unknown_fields`), so 13.6 has no bench delta at all.
- **Q3. Keeping the daemon tests on the 128² world.** 55 `serve.rs` tests (`Daemon::spawn`, `serve.rs:49-70`) and the gui pixel guards (`pixel_guard.rs:155-170`) all start a bare `simd 0`, which is a fresh game. Several are pinned to DEFAULT_SEED's 128² camp (`serve.rs:584`, `:712`, `:2778`), and the pixel guards' ceilings were calibrated on the 128² boot frame. A fresh 256² game breaks those pins, and every connect then encodes a snapshot of about 28 MB in debug.
  - (a) **Recommended:** the fresh game takes `simd PORT --dims X,Y,Z [--crew N]` (default 256,256,32 and 5), refused loudly when out of range. `Daemon::spawn` and the pixel guard pass `--dims 128,128,32`, so their worlds, pins and wall-clock stay as they are.
  - (b) Move the tests to 256: re-pin the camp recipes, re-calibrate the pixel guards, and accept an unmeasured slowdown of the slow suite.
  - (c) Do not flip the fresh game here. AC 2 moves to 13.8.
- **Q4. The real-work test cannot run in the debug gate.** A debug build steps 256² crew 200 at 29 ms/tick while the crew is still walking, 22 times release's 1.3 ms, and the first delivery comes at about t1,750 (*Found at creation*).
  - (a) **Recommended:** an `#[ignore]`d test in `crates/simd/tests/bench.rs` that drives the release binary through `simd --bench`, run by a new full-tier line in `scripts/gate.sh`: `cargo test --release -p simd --test bench -- --ignored two_hundred`. That costs about one release build plus a ~100 s run per full gate, and nothing on the fast tier.
  - (b) No gate test. The scenario file carries an `expect` block that `--bench` enforces, so `tick-bench.sh` FAILs the row on every later story.
- **Q5. The gui camera at 256².** The focus clamp is the 128² world (`camera.rs:9-12`, whose own NOTE says a larger world must follow `Dims`), and the boot focus is a literal `(64,64,9)` (`ingest.rs:1615`). On 256² the camp is `(127,128,7)`, so the camera boots off camp and cannot reach three quarters of the map. Sky, fog, aurora and snow are centred on the 128² world (`atmosphere.rs:52`, `:66`, `ingest.rs:1705`).
  - (a) **Recommended:** the clamp follows the snapshot's `Dims`, and the boot focus is the campfire once the snapshot lands, unless `--camera` was given. On 128² DEFAULT_SEED the campfire is at `(64,64,9)`, so the boot frame there is unchanged. The atmosphere stays as it is; the seat judges it (Task 8).
  - (b) No gui code. Measure with `--camera` (the clamp still reaches the camp at `(127,127,7)`) and file the camera and atmosphere as issues.

## Found at creation (2026-10-29, `main` code `0a8b4462`)

Throwaway scratch worktrees, release builds. The patches and raw lines are in `13-6-signoff/` (`prototypes/scale-near.patch`, `prototypes/probe187-refusals.patch`, `creation-probes.txt`). Timings are **not** timing-grade: up to four runs shared the devpod.

- **#187 reproduces.** A counter at the channel cut's refusal (`lib.rs:2537-2542`) ran over the four `UNFINISHED_LATE` cage runs (`scenario.rs:3054`) to 16,000 ticks. Runs (0,4) and (6,6) refuse the same cut 56 and 55 times in t12k–16k, about one refusal every 72 ticks to the end of the run. Runs (9,6) and (13,6) never refuse (never claimed: #218 shape B).
- **#159 does not stop real work, so 13.6 and 13.7 do not swap** (epic Task 0). 13.2's own 256² layout (mine at (64,64), pile at (192,192), legs across the map) stores **0 by t3,000**, with 501 items lying and the tick at 174 ms. The cause is distance, not the search budget: the same layout at crew 5 finishes all 45 marks by t8,000 and stores 2. A layout near the camp (`scale-near.patch`, `SCALE_OFF=16`: mine targets (111,112) and pile (143,144) from camp (127,128,7)) does real work:

  | t | 750 | 1,000 | 1,750 | 2,000 | 2,500 | 3,000 |
  |---|---:|---:|---:|---:|---:|---:|
  | stored | 0 | 0 | **5** | 17 | **50** | 74 |
  | dig / cut marks done (of 1,600 / 200) | 7 / 7 | 39 / 15 | 146 / 58 | 167 / 65 | 227 / 86 | 274 / 114 |
  | mean ms over the last 250 ticks | 3.6 | 15.0 | 55.5 | 68.1 | 88.7 | 108.5 |

  `SCALE_OFF=32` stores 13 by t3,000. 1,800 marks are under `MAX_DESIGNATIONS` 4096. The camp's walkable component is 64,300 cells, over `MAX_ASTAR_NODES` 50,000 (#159 is live, but not blocking).
- **The first NFR12 reading will sit far over the bar.** The tick cost climbs with items lying (737 at t3,000), to about 4× NFR12's 25 ms mean. Recording it is this story's job; NFR12 gates the milestone close, and optimization waits for a profile (TP).
- **Sizes at 256².** A 128² save is 7.27 MB (`scenarios/saves/`) and the 128² snapshot about 6.9 MB (`tui/src/main.rs:40`). 4× both is about 29 MB and 28 MB, under the 64 MB caps (`simd/src/main.rs:33`, `tui/src/main.rs:41`, `gui/src/ingest.rs:85`). Task 2 measures them for real.

## Acceptance Criteria

1. **Settings from a scenario (FR53).** A scenario `start` of `{"seed":S,"dims":[X,Y,Z],"crew":N}` loads a world with those dims and exactly N dwarves. With `dims`/`crew` absent it loads the world Q2 names. The loader refuses, naming the cause: crew 0, dims outside the supported range (x and y in 128..=256, z = 32), and a crew with more dwarves than walkable ground near the camp.
2. **Saved and deterministic (FR53, AD-11).** A 256² crew-200 world saved at tick T, loaded and stepped N ticks equals the never-saved world (dims, every dwarf, jobs, `to_save` bytes). Two worlds generated from the same seed, dims and crew are identical.
3. **A fresh game is 256² (FR53).** `simd` with no scenario and no load serves a snapshot whose `dims` are 256×256×32 (emitter `bridge.rs:13-17`) with 5 dwarf entities. Under Q3 (a), `--dims`/`--crew` override that. The 256² DEFAULT_SEED world has the camp and campfire, trees, lake ice, and a walkable component from the camp.
4. **Two hundred spawn (FR53).** On 256² and on 128², crew 200 spawns with no panic: 200 distinct standable tiles, none a campfire or torch cell, each in the camp's walkable component. A crew that fits the camp ring draws exactly as today: `spawn_positions_for_seed_42_are_pinned` and every other worldgen pin pass unchanged.
5. **No committed outcome moves (FR53).** `RUNS=3 scripts/tick-bench.sh main HEAD` over the 11 committed scenarios (`scenarios/*.json` before this story) reads `same` on every row.
6. **The 256² crew-200 scenario does real work (FR54).** `scenarios/two-hundred-on-256.json` runs `--bench` with an end line that gains `done=dig:N,cut:M` (marks the sim completed during the run), `stored=S` (items lying on zone cells at the end) and `cells=C` (tiles that differ from the start world). A test asserts `N ≥ 1`, `M ≥ 1`, `S ≥ 1` and `C ≥ 1` on the real file (Q4). The file holds ≤ 4096 marks.
7. **The instrument moves with the world.** On `tests/fixtures/one-dig.json`: the no-command variant reads `done=dig:0 … stored=0 cells=0`; a variant long enough for the dig to finish reads `done=dig:1` and `cells≥1`.
8. **NFR12's first reading.** The record holds `simd --bench scenarios/two-hundred-on-256.json --runs 5` (release, nothing else running): each run's mean and p99, the median p99, and the spread as the noise floor. `main` cannot read this file (unknown `dims`), so this reading is HEAD-only. Later stories take their delta against `main` here. The bar does not gate this story.
9. **The scale traps are each checked** (Task 6): fixed, or carrying a `// NOTE:` that names the story that fixes it. Names repeat past 16 and colours past 5 until 13.8. Professions for crew N are the 2:2:1 miner:hauler:woodcutter mix in one seeded shuffle, not `i % 5`; crew 200 holds 80/80/40, and crew 5 matches today's draw.
10. **The seat (NFR13's first reading).** Wolf runs the scenario at the seat with `--perf-log`, and the gui frame rate with 200 dwarves on 256² is recorded at working zoom and at full vista against NFR6's bars. A miss is measured and ruled, not fixed here. **Split line:** this AC becomes its own story if 13.6 overruns a session.

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings (Q1–Q5).** Record the answers in the Change Log and amend the ACs they name. Post `13-6-signoff/issue-187-comment.md` on #187 if Wolf says yes. Then ASK which implementer to use before dev (memory: `dev-implementer-is-sonnet-not-codex`).
- [x] **Task 1: crew and dims in `sim-core` (AC 2, 4, 9).** UPDATE `crates/sim-core/src/lib.rs`. Port `13-2-signoff/prototypes/scale.patch`'s two `lib.rs` hunks by hand (its `scale_gen` module is NOT shipped).
  - [x] `pub fn generate_with_crew(seed: u64, dims: Dims, crew: usize) -> World`. `generate(seed, dims)` calls it with 5. Doc the panic: no walkable ground for `crew`.
  - [x] `spawn_dwarves(camp, rng, identities: Vec<Identity>, professions: Vec<Profession>)` (`:3598`). The ring candidates are drawn exactly as today. Only when the ring holds fewer than `crew`, the candidates become the first `crew` cells of a breadth-first walk from the camp over `astar_neighbours` (`:1130`), with the camp emitters blocked (scale.patch's shape). Then the same `rng.random_range` draw.
  - [x] Identities: today's shuffled `names` and `colours`, with dwarf i taking `names[i % 16]` and `colours[i % 5]`, and a `// NOTE:` that 13.8 makes names unique. Professions: `[Miner, Miner, Hauler, Hauler, Woodcutter]` cycled to length `crew`, then ONE `shuffle(&mut profession_rng)` over the whole list. For crew 5 this is today's array and today's shuffle.
  - [x] Wander phase NOTE (`:3635`): rewrite it to say what 200 dwarves give, about 20 per phase. Any assignment into a 10-tick period gives that, so the NOTE says the seat judges lockstep (Task 8) and nothing changes here.
  - [x] `Dims::DEFAULT` stays 128×128×32 (280+ test sites pin it). Add `Dims::FRESH = 256×256×32`, and give `DEFAULT` a doc line: the 128² world every pinned test and committed scenario runs on.
- [x] **Task 2: tests for Task 1 (AC 2, 3, 4, 9).**
  - [x] UPDATE `crates/sim-core/tests/worldgen.rs`. `a_crew_of_200_spawns_near_the_camp_one_per_tile` at 128² and 256² (AC 4's four properties, plus a positive count of 200). `a_256_valley_is_playable` (campfire at the camp, trunk columns > 0 via `tree_trunk_columns`, lake ice cells > 0, the camp component > 50,000 cells: creation read 64,300). `crew_200_holds_the_two_two_one_mix` (80/80/40, and the first five dwarves' professions equal crew 5's).
  - [x] UPDATE `crates/sim-core/tests/save_load.rs`. `a_256_crew_200_world_saves_loads_and_steps_identically`: save at tick 10, load, step 20, compare `to_save` bytes with the never-saved world. Keep the ticks small: debug runs 29 ms/tick here. Print `to_save` byte length (AC 2's size reading; the snapshot is measured in Task 4).
- [x] **Task 3: scenario settings and the fresh game in `simd` (AC 1, 3; Q2, Q3).** UPDATE `crates/simd/src/scenario.rs` (`Start` `:36`, `world()` `:120`) and `crates/simd/src/main.rs` (`parse_args` `:92`, fresh world `:174`).
  - [x] `start` takes optional `dims` and `crew` beside `seed` (not with `save`). Absent means Q2's answer. Refuse at load: crew 0, dims out of 128..=256 / z ≠ 32, `dims`/`crew` with `save`. Each refusal names its field (NFR11). A crew too big for the ground is caught before `generate_with_crew` panics: refuse when the camp's component is smaller than `crew`, or let the panic message name it, but name it either way.
  - [x] Fresh game: `World::generate_with_crew(DEFAULT_SEED, Dims::FRESH, 5)`. Under Q3 (a), `--dims X,Y,Z` / `--crew N` for the fresh game only (refused with `--scenario`/`--bench`, same range checks), and `Daemon::spawn` (`serve.rs:67`) and the pixel guard (`pixel_guard.rs:165`) pass `--dims 128,128,32`.
  - [x] **Print no new startup line.** `bench.rs:254` expects `holding` as the second stdout line, `serve.rs:111` `next_log` treats every stderr line as a reply to input, and `Daemon::spawn` drops its stdout receiver after the listening line (a later `println!` can hit a closed pipe). AC 3 reads the snapshot instead.
  - [x] Tests: extend `every_unusable_scenario_fails_loudly_naming_the_cause` (`bench.rs:149`) with crew 0, an out-of-range dims, dims with save, and a crew too big. Add a `bench.rs` test where a `dims:[256,256,32],"crew":7` variant's end line and the oracle agree (rebuild the expected world in the test with `generate_with_crew`, as `expected()` does). Add `a_fresh_game_serves_a_256_valley_with_five_dwarves` to `serve.rs`: a bare spawn (no `--dims`) serves a snapshot with `dims` 256×256×32 and 5 dwarf entities (a debug 256² snapshot is about 28 MB, so read it once). Under Q3 (a), add `--dims 160,160,32 --crew 7` → those dims and 7 dwarves, plus refusals for an out-of-range `--dims` and `--crew 0`.
- [x] **Task 4: the instrument, RED first (AC 6, 7).** UPDATE `crates/simd/src/scenario.rs` `bench` (`:281-345`).
  - [x] Keep a copy of the start tiles. Each tick, outside the timed window, record which Dig/Cut marks existed before `world.step()` and are gone after it: that is `done`. Commands land before the step, so a cancel never counts. At the end, append `done=dig:N,cut:M stored=S cells=C` to the end line, after the existing fields; `field()` (`bench.rs:91`) parses by key.
  - [x] Tests (AC 7) in `crates/simd/tests/bench.rs`: the fixture's no-command variant reads `done=dig:0,cut:0`, `stored=0`, `cells=0`. A `ticks: 400` variant of `one-dig.json` reads `done=dig:1` and `cells≥1`. Creation read the dig still marked at t100 and finished by t200 (`marks=0 items=1`), so 400 leaves margin.
  - [x] Print the snapshot byte length once for 256² (a throwaway, or a `bridge.rs` test that prints `serde_json::to_vec(&snapshot(..)).len()`) and record it with Task 2's save size. Both must be under 64 MB.
- [ ] **Task 5: the scenario and its real-work test (AC 6, 8; Q4).**
  - [x] NEW `scenarios/two-hundred-on-256.json`: copy `13-6-signoff/near16-256-crew200.json` (generated at creation by `scale-near.patch` with `SCALE_OFF=16`), set `"name":"two-hundred-on-256"`, `"ticks":2500` and `"speed":"normal"` (the bench ignores speed; at the seat, fast4x's 5 ms period is far below a 40–100 ms tick), and rewrite `about` to say what it is and how it was made (patch + command). Creation reading at t2,500: stored 50, dig 227, cut 86. Do not hand-edit the commands.
  - [x] RED: run the release binary on a copy with `"commands":[]` and paste the end line (`done=dig:0,cut:0 stored=0 cells=0`). Then GREEN on the real file.
  - [x] Under Q4 (a): `#[ignore = "release: steps 256² crew 200 for 2500 ticks"]` test `the_two_hundred_on_256_scenario_does_real_work` in `bench.rs`, running `env!("CARGO_BIN_EXE_simd") --bench scenarios/two-hundred-on-256.json --runs 1` and asserting AC 6 on the end line, with positive values pasted from the run in a comment. UPDATE `scripts/gate.sh`: the full tier (not `--fast`; check the `--range` scope block, `gate.sh:133-145`) runs `cargo test --release -p simd --test bench -- --ignored two_hundred`.
  - [ ] AC 8: `target/release/simd --bench scenarios/two-hundred-on-256.json --runs 5` on an idle devpod. Paste every run line, the median p99 and the spread.
- [x] **Task 6: the scale traps (AC 9).** One line each in the Debug Log, naming fixed or NOTE and where:
  - [x] `Dims::DEFAULT` → Task 1. `[Identity; 5]`/`[Profession; 5]` → Task 1. `CAMP_RADIUS` spawn panic → Task 1.
  - [x] `MAX_ASTAR_NODES` (`lib.rs:50`): `// NOTE:` that the 256² camp component is 64,300 cells and 13.7 (#159) sizes the budget.
  - [x] `MAX_DESIGNATIONS` (`:49`): the scenario's 1,800 marks are under it. `// NOTE:` that #170 drops cells silently at the cap and M4's forage/fishing marks add to the count.
  - [x] Wander phases → Task 1. The gui camera → Task 7 (Q5).
- [x] **Task 7: the gui camera (Q5 a; AC 10's precondition).** UPDATE `crates/gui/src/camera.rs` and `crates/gui/src/ingest.rs`.
  - [x] `FOCUS_MAX` becomes the snapshot's `Dims` minus one on each axis, for both `place` (`:72`) and `pan` (`:105`). Replace the NOTE at `:9-11`. Tests: the existing clamp tests (`camera.rs:392`, `ingest.rs:7297`) keep their 128² values when given 128² dims, and a 256² rig pans to `(200,200,7)`.
  - [x] Without `--camera`, the first snapshot moves the focus to the campfire. The rig is built before the snapshot (`ingest.rs:1615`), so this is a one-shot on ingest. Test: on 128² DEFAULT_SEED the focus stays `(64,64,9)` (the boot pin at `ingest.rs:7285` stands), and on 256² it lands at the campfire.
  - [x] Atmosphere centring (`atmosphere.rs:52`, `:66`; fog volume `ingest.rs:1705`) is not touched. File an issue only if the seat shows a defect (Task 8). Look work waits for a concrete defect (memory: `art-gates-visual-judgement`).
- [ ] **Task 8: the seat card (AC 10).** NEW `_bmad-output/implementation-artifacts/13-6-signoff/vehicle-card.md`. Push first (`scripts/push.sh --fast`, standing authorisation: memory `push-branch-for-vehicle-testing`). In WSL, run a release `simd 7451 --scenario scenarios/two-hundred-on-256.json`. In PowerShell from the Windows checkout, run `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-6-perf-working.log','--camera','0.7,0.45,45,127,128,7')` for working zoom. Restart the daemon, then do the same with `13-6-perf-vista.log` and `'0.7,0.45,500,127,128,7'` for full vista. Check both `--camera` lines headless first (lavapipe; memory: `headless-gui-instruments`). Then a bare `simd 7451` (the fresh game) with no `--camera`. Questions for Wolf: (1) fps at each framing against NFR6, read from the perf log; (2) do the camp and the work read on 256² (camera reach, atmosphere)? (3) does the fresh game boot onto the camp of a 256² valley? Record the answers. A defect gets an issue.
- [ ] **Task 9: mutation table.** NEW `_bmad-output/implementation-artifacts/mutations/13-6.sh` in `13-4.sh`'s shape. Commit first, then run it alone (`env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …`). Every row must be KILLED at its own assert:
  1. the spill never runs (ring only) → the crew-200 spawn test (panic);
  2. the spill takes any standable cell in a box instead of the camp component → the component assert;
  3. the spill always runs, even for crew 5 → `spawn_positions_for_seed_42_are_pinned`;
  4. professions cycle `i % 5` → `crew_200_holds_the_two_two_one_mix` (it reads 80/80/40 under the cycle too, so the test also needs an order assert that the cycle fails; prove the row KILLS);
  5. the scenario loader ignores `dims` (generates `Dims::DEFAULT`) → the 256 crew-7 bench test;
  6. absent `dims` means 256 → `bench_replays_the_scenario_identically_and_reaches_the_oracle_state`;
  7. the fresh game uses `Dims::DEFAULT` → the bare-spawn `serve.rs` snapshot test;
  8. `done` never counts → the 400-tick fixture test;
  9. `cells` compares the end world with itself → the 400-tick fixture test;
  10. `create_haul_jobs` returns early → `the_two_hundred_on_256_scenario_does_real_work` (release; a heavy row, about 3 min);
  11. (Q5 a) `FOCUS_MAX` stays 127 → the 256² pan test.

  `python3 scripts/audit-mutations.py` must read every row applying.
- [ ] **Task 10: gate and close.** Fast tier on every commit (pre-commit hook). ONE full gate at PR: `RUST_TEST_THREADS=1 scripts/gate.sh` (~70 min, memory: `gate-ooms-at-default-parallelism`). Paste the block. Paste AC 5's `tick-bench` table. Cost row (`python3 _bmad/scripts/session_tokens.py --phase dev --story 13-6-two-hundred-on-256`), then `phase_state.py clear`, then Status → review on the story and the board.

## Verification

```bash
# AC 2, 4, 9: sim-core
cargo test -p sim-core --test worldgen crew_200 a_256_valley spawn_positions_for_seed_42
cargo test -p sim-core --test save_load a_256_crew_200
# AC 1, 3, 7: loader, fresh game, instrument (debug)
cargo test -p simd --test bench
cargo test -p simd --test serve fresh_game
# AC 6: RED (commands removed -> done=dig:0,cut:0 stored=0 cells=0), then GREEN, then the gate's release test
cargo build --release -p simd
target/release/simd --bench /tmp/<copy with "commands":[]>.json --runs 1
target/release/simd --bench scenarios/two-hundred-on-256.json --runs 1     # stored>=1, dig>=1, cut>=1, cells>=1
cargo test --release -p simd --test bench -- --ignored two_hundred
# AC 8: NFR12's first reading (idle devpod, nothing else running)
target/release/simd --bench scenarios/two-hundred-on-256.json --runs 5
# AC 5: no committed outcome moves (the 11 old files; main cannot read the new one)
RUNS=3 scripts/tick-bench.sh main HEAD $(git ls-files 'scenarios/*.json' | grep -v two-hundred)
# AC 9: sabotage table
env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-6.sh
python3 scripts/audit-mutations.py
# AC 10: Task 8's seat card (Windows)
```

## Dev Notes

### Scope guardrails (do NOT)

- No arrivals, unique names or crew list (13.8); no budget or heuristic change to A* (13.7, #159); no #187/#218 fix (Q1).
- No new saved field, no `protocol` change, no new dependency.
- No look tuning: atmosphere, fog and snow centring stay as they are (Task 7). No gui work beyond Q5's answer.
- No optimization for NFR12: record the reading (AC 8). A profile comes first, in a later story.
- Do not ship `scale_gen` or any generator. The committed JSON is the artifact, and its `about` names the patch that made it.

### What already exists (build on it)

- `13-2-signoff/prototypes/scale.patch` is the sketch for Task 1 (the spill) and Task 3 (the `StartFile` match). `13-6-signoff/prototypes/scale-near.patch` is scale.patch plus the camp-relative layout (`SCALE_OFF`) and the throwaway `TL`/`PROBE` lines.
- `World::generate` (`lib.rs:2866-2923`): per-purpose RNG streams (`:28-34`). Identity and profession streams never shift a spawn. Keep that.
- `SaveState` already has `dims` (`lib.rs:2988-3003`); `from_save` rebuilds dwarves from the saved list.
- Bench end line and `state_hash`: `scenario.rs:333-345`, `:248`. Test helpers: `field`, `end_line`, `variant`, `expected` (`bench.rs:30-103`).
- tui camera clamps to the wire `dims` already (`tui/src/view.rs:171-178`), so the tui needs no change.

### Key decisions & traps

- **`tick-bench.sh` reads each scenario file from THIS checkout for both builds.** `main`'s `simd` refuses `dims`/`crew` (`deny_unknown_fields`), so pass the 11 old files explicitly (Verification). The new file is HEAD-only in 13.6.
- **The state hash covers the save encoding.** No new saved field means AC 5 can read `same`. If any row reads `differ`, find the outcome change. Do not re-pin.
- **Crew 5 must draw exactly as today.** Ring first, BFS only when the ring runs short. Shuffle the same 5-element mix with the same RNG. `SliceRandom::shuffle` on a `Vec` and on an array is the same algorithm over a slice; the worldgen pins prove it.
- **Debug is 22× release at this scale** (29 ms/tick idle). No debug test may step a 256² crew-200 world past a few dozen ticks. The real-work test is release (Q4).
- **The bench's new bookkeeping must stay outside `started.elapsed()`.** `designations()` builds a 1,800-entry Vec, so diff before/after the step outside the timed window.
- **The tick climbs with items lying** (creation: 108 ms mean over t2,750–3,000). Expect AC 8 well above 25 ms. Record it; do not optimize.
- **`wander` homes all 200 on the camp.** Crowding at the camp is game feel for 13.11, not a defect here.
- **Probes go in scratch worktrees, built from inside them, `-j 8`** (memory: `cargo-target-dir-builds-cwd-sources`, `build-parallelism-ooms-the-devpod`). Reap daemons by PID, never `pkill -f` (memory: `pkill-pattern-self-match`, exit 144 at creation).

### Previous story intelligence

- 13.4: the state hash moved everywhere when one saved field was added (memory: `state-hash-covers-save-encoding`). 13.6 adds none, which is why AC 5 can demand `same`.
- 13.2/13.3: the scale prototypes are throwaway C0-stack patches. Port by hand: no `counters`, no `census`, and `scale.sh` hardcodes `BASE=4f57027`.
- 13.4 ran Sonnet 5.5 subagents with Opus orchestrating; its full gate was 4,140–4,278 s with `RUST_TEST_THREADS=1`.

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs`: `generate_with_crew`, `spawn_dwarves`, `Dims::FRESH`, NOTEs
- UPDATE `crates/sim-core/tests/worldgen.rs`, `crates/sim-core/tests/save_load.rs`
- UPDATE `crates/simd/src/scenario.rs`: `Start`, refusals, end line
- UPDATE `crates/simd/src/main.rs`: fresh game, `--dims`/`--crew` (Q3 a), world line
- UPDATE `crates/simd/tests/bench.rs`, `crates/simd/tests/serve.rs`, `crates/gui/tests/pixel_guard.rs` (`--dims`, Q3 a)
- UPDATE `crates/gui/src/camera.rs`, `crates/gui/src/ingest.rs` (Q5 a)
- UPDATE `scripts/gate.sh` (Q4 a)
- NEW `scenarios/two-hundred-on-256.json`; NEW `mutations/13-6.sh`; NEW `13-6-signoff/vehicle-card.md`
- `13-6-signoff/`: `creation-probes.txt`, `near16-256-crew200.json`, `issue-187-comment.md`, `prototypes/scale-near.patch`, `prototypes/probe187-refusals.patch` (creation)

### References

- `_bmad-output/planning-artifacts/epics.md` Story 13.6 (`:3072-3109`), Epic 13 M4 standing ACs 10–18, Milestone 4 requirements (`:299-330`)
- `_bmad-output/planning-artifacts/prds/prd-frostvein-2026-10-28/prd.md` FR53, FR54, NFR12, NFR13; `addendum.md` § Scale data, § What the code fixes at five, § Movement and sim debt; `reconcile-inputs.md` 6.1, 6.2
- `_bmad-output/implementation-artifacts/13-2-signoff/scale-table.md`, `scale/README.md`, `prototypes/scale.patch`
- `crates/sim-core/src/lib.rs:28-50`, `:134-147`, `:1130`, `:2537-2542`, `:2866-2923`, `:3598-3645`
- `crates/simd/src/scenario.rs:22-41`, `:118-131`, `:248`, `:281-345`; `crates/simd/src/main.rs:33`, `:92-145`, `:164-177`
- `crates/simd/tests/bench.rs:30-160`; `crates/simd/tests/serve.rs:49-95`; `crates/gui/tests/pixel_guard.rs:150-185`
- `crates/gui/src/camera.rs:9-12`, `:68-105`; `crates/gui/src/ingest.rs:1615`, `:1705`, `:7285-7297`; `crates/gui/src/atmosphere.rs:52`, `:66`
- `scripts/tick-bench.sh`, `scripts/gate.sh`; GitHub #159, #170, #179, #187, #214, #218

## Dev Agent Record

### Agent Model Used

Implementer: Sonnet 5.5 subagents (`claude-sonnet-5-5`), Opus 5.5 orchestrating (Wolf, 2026-10-30).

### Debug Log References

- **Tasks 1–2 (`6256b996`, Sonnet 5.5):** `generate_with_crew`, the BFS spill (only when the ring holds fewer than `crew`), cycled identities/professions with one shuffle, `Dims::FRESH`. Readings: the 256² DEFAULT_SEED camp component is **64,301 cells**; `to_save` JSON at 256² crew 200, tick 10, is **28,351,241 bytes** (under the 64 MB cap). Opus re-ran `worldgen -- crew_200 a_256_valley a_crew_of_200 spawn_positions_for_seed_42`: 4 passed. The commit's fast gate was green (fast set 981 s).
- **Task 2 amendment:** "the first five dwarves' professions equal crew 5's" was impossible as written. One shuffle over 200 entries is not crew 5's shuffle over 5 (crew 200 drew `[M,M,W,M,H]` against crew 5's `[W,H,M,H,M]`), so that assert was dropped. AC 9's claim, that crew 5 draws exactly as today, is still held by `spawn_positions_for_seed_42_are_pinned` and the existing profession tests, which pass unchanged. The order assert that kills an `i % 5` cycle reads "periodic with period 5" (hand sabotage: RED).
- **Dev-dependency (TP: one sentence each):** `sim-core` gains `serde_json` under `[dev-dependencies]` (already a workspace crate, nothing new in the lock tree beyond the edge). The 256² save test round-trips `to_save` through the JSON bytes simd actually writes, and reads their length.
- **Component oracle:** the worldgen test's BFS mirrors `astar_neighbours` independently. A spill sabotage that takes the first N box cells in iteration order SURVIVES (the first box cells are all reachable), and a random draw from a ±6 z box is KILLED (`Pos { x: 123, y: 122, z: 13 } is standable but the camp cannot walk to it`). Task 9 row 2 is written as the random draw. The test also runs seeds 0..16 at 128² so that one lucky seed cannot carry it.
- **Task 3 (`4b476c51`, Sonnet 5.5):** `start` is a `StartFile` (optional seed/save/dims/crew). Absent dims/crew are 128²/5 (Q2 a). Refusals name their field: `start crew`, `start dims X,Y,Z`, dims/crew with save, seed+save or neither. Fresh game `Dims::FRESH`/5 with `--dims`/`--crew` (Q3 a), refused with `--scenario`/`--bench`. **Crew too big:** `generate_world()` (`main.rs`) catches the sim's panic and returns `crew N does not fit the camp's ground at XxYxZ`. Chosen because sim-core has no public count of the camp's ground; the panic hook still prints the sim's own line first. Q3 (a) spawns moved to `--dims 128,128,32`: `Daemon::spawn` (now `spawn_with(&[..])`), the pixel guard, and **`scripts/bench/export_world.py`, a trap the story missed**: its gate test pins the 128² mesh counts. `scripts/tests` 83 passed. No other bare-`simd` spawn exists. Tests: 6 new refusal cases; `a_seed_start_with_dims_and_crew_builds_that_world` (256²/7, no command, oracle via `expected_world`, and ≠ the 128²/5, 128²/7 and 256²/5 oracles); `parse_args` unit tests; serve.rs `a_fresh_game_serves_a_256_valley_with_five_dwarves` (+ `--dims 160,160,32 --crew 7`) and `a_bad_dims_or_crew_flag_is_refused_naming_the_flag`.
- **Task 4 (`f1c83b7a`, Sonnet 5.5):** the end line gains `done=dig:N,cut:M stored=S cells=C`, with mark bookkeeping outside the timed window. RED: `no done= in "bench one-dig end tick=400 marks=0 items=1 zones=0 goals=0/0/0"`. GREEN: `bench` 6 passed, `--bins` 37 passed. **256² snapshot: 28,053,919 bytes**, pinned under the 64 MiB cap by `bridge.rs::the_fresh_256_snapshot_fits_the_client_cap`. Save at 256²/200 is 28,351,241 bytes (Task 2). Both are under 64 MB.
- **Task 5 (Opus):** `scenarios/two-hundred-on-256.json` is `13-6-signoff/near16-256-crew200.json` with the name, `ticks` 2500, `speed` normal and a new `about` (the patch plus the exact generator command). The 1,801 commands are byte-identical (1,800 marks, ≤ 4096). Release, scratch worktree at `f1c83b7a`. **RED** (`"commands":[]`): `bench two-hundred-on-256-nocmd end tick=2500 marks=0 items=0 zones=0 goals=0/0/0 done=dig:0,cut:0 stored=0 cells=0`. **GREEN** (the real file, 91 s): `run 1/1: ticks=2500 commands=1801/1801 mean=36.1294 p99=169.1693 max=265.037 over10=1563 state=3fc1461d491ee5d9` / `end tick=2500 marks=1478 items=602 zones=1800 goals=28/0/0 done=dig:231,cut:91 stored=59 cells=2165`. Creation read 227/86/50. The difference is the shipped spill against the prototype's, which is expected. `the_two_hundred_on_256_scenario_does_real_work` (`#[ignore]`, panics in a debug build) passed in release twice (95.9 s, 92.5 s). `gate.sh`: the full tier's code arm runs `cargo test --release -p simd --test bench -- --ignored two_hundred`, and the fast and scoped banners name it as SKIPPED.
- **Task 6, scale traps (AC 9):** `Dims::DEFAULT`: kept 128² with a doc line, `Dims::FRESH` added (Task 1). `[Identity; 5]`/`[Profession; 5]`: FIXED, Vecs sized to the crew (Task 1). `CAMP_RADIUS` spawn panic: FIXED, BFS spill (Task 1). Names repeat past 16 and colours past 5: `// NOTE:` naming 13.8 (`generate_with_crew`). Professions: one seeded shuffle of the cycled 2:2:1 mix (80/80/40 at 200). Wander phases: NOTE rewritten (about 20 per phase at 200; the seat judges). `MAX_ASTAR_NODES` (`lib.rs:55`): `// NOTE:` 64,301-cell camp component, #159, 13.7 sizes it. `MAX_DESIGNATIONS` (`lib.rs:52`): `// NOTE:` 1,800 marks, #170 silent drop, M4 forage/fishing add. gui camera: Task 7.
- **Task 7 (Sonnet 5.5, committed by Opus):** `CameraRig.focus_max` (boot 127,127,31), `fit_world(dims)` on every snapshot. `follow_snapshot(.., first, start)` on the BOOT snapshot (read by `connect_to_daemon` before the app exists, so it is applied in `setup_camera`) moves the focus to the campfire, or re-applies `--camera` against the real ceiling. **That second half was a trap the story missed:** `--camera ...,127,128,7` was placed before any snapshot and clamped y to 127. Tests: `a_256_world_lifts_the_focus_ceiling_past_the_old_128_clamp` (200,200,7 holds; pans to 255,255,7 and 0,0,7), `the_boot_snapshot_sets_the_focus_ceiling_and_moves_the_focus_to_the_campfire` (128: 64,64,9 unchanged; 256: 127,128,7; no campfire stays; `--camera` wins; 128 still clamps). RED by hand: `fit_world` leaving the ceiling unset failed both. **(64,64,9) verified live:** the first snapshot of `simd 7777 --dims 128,128,32` carries the Campfire at [64,64,9]. The agent kept the old pan-clamp line with an `#[allow(non_snake_case)] let FOCUS_MAX` shim so 10.10's mutation row applied. Opus removed the shim and re-pointed that row's literal to `self.focus_max` instead (audit: 962 rows match). The existing `shift_middle_drag_pans...` fixture now sets `focus_max` (2x1x1 world), with its expected values unchanged.

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-10-29 | Story created (Opus 5.5). Creation probes: #187 reproduces on `main`; #159 does not block real work near the camp; layout, readings and patches in `13-6-signoff/`. Task 0 Q1–Q5 await Wolf. |
| 2026-10-30 | Task 0 ruled (Wolf): **Q1 (c)** fold #187 into 13.18 (epic stays at 16; 13.18's AC amended in `epics.md`); **Q2 (a)** a `start` with no `dims`/`crew` is 128×128×32, crew 5; **Q3 (a)** `--dims`/`--crew` on the fresh game, tests pass `--dims 128,128,32`; **Q4 (a)** release `#[ignore]` test on the full gate tier; **Q5 (a)** clamp follows `Dims`, boot focus is the campfire. Only Q1 differs from the recommendations, and no 13.6 AC names it (Scope: no #187/#218 fix). #187 evidence comment posted, amended to name 13.18. Board, story and `13-6-signoff/` committed on the 13.6 branch. |
