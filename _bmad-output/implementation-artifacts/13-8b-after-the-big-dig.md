---
model: claude-fable-5-2  # Wolf switched the session with /model on 2026-11-12; not the Opus default
baseline_commit: 5f83e82e
---

# Story 13.8b: After the Big Dig

Status: in-progress

## Story

As the boss,
I want the fortress to keep ticking smoothly after I order a big dig with two hundred dwarves,
so that the game does not stop and start while they work.

Closes the three measured parts of #231 that can be cut without changing what any dwarf does. The other two parts are named under Task 0 Q1.

## Branch

`story-13-8b-after-the-big-dig`, cut from `main` `5f83e82e` (PR #234 merged; local `main` equals `origin/main`). Commits authored `Völundr <jeicei75@gmail.com>`, no attribution trailer.

**Standing ACs that bind:** Epic 13's M3 ACs 1–8 (one full gate, at PR), and M4 standing ACs 10 and 12. AC 11 binds as "nothing moves": this story changes no outcome, so no scenario is re-pinned. ACs 13–18 do not apply (no need, stock, trade, look, rendering or new constant).

**Wire diff:** none. **Saved fields:** none. Anything this story keeps between ticks is derived data and is never written to a save.

## Task 0 — Wolf rules before dev

Each question has a recommendation; the ACs and Tasks are written for it. **RULED 2026-11-13 (Wolf): Q1 (a), Q2 accept, Q3 file the issue (filed as #235).**

- **Q1. Which of the five measured costs does this story cut?** (Figures under "Found at creation".)
  - (a) **Recommended (← RULED 2026-11-13):** the three that are provably the same rule done less often: failed claim searches repeated on unchanged ground (AC 2–4), the stone map rebuilt for every dwarf (AC 5), and the pile cells recomputed for every haul job (AC 6). Dwarves blocking each other (`resolve_blocked_step`, 4–29 ms a tick) and the re-plan burst after every dig (about 130 searches in one tick) go to one follow-up issue with these figures. Both are movement rules: a cheaper version is a different rule and needs its own A/B.
  - (b) All five in this story. It is then two sessions, and the last two cannot promise an unchanged `state=`.
- **Q2. Slot.** This story is `13.8b`, before 13.9, as 12.10b was. The epic's count goes from 16 to 17. Recommended (← RULED 2026-11-13): accept; the alternative is to renumber or to drop a later story under the cut order.
- **Q3. A finding outside this story.** From about tick 28,000 on seat-like, about 710 haul jobs fail together every 20 ticks while only 497 of 1,530 stones are stored. Why they fail is not investigated. Recommended (← RULED 2026-11-13): I file it as its own issue; this story does not touch it. Filed as **#235** (`bug`, `route:undecided`).

## Found at creation (2026-11-12, `main` `5f83e82e`)

Instrument: `13-8b-signoff/prototypes/split-probe.patch` (timers and counters, applied in a scratch worktree), release `simd --bench 13-8-signoff/prototypes/seat-like.json --runs 1`. Two idle runs agree: mean 19.42 ms, p99 125.4 / 126.0, max 381, `over10` 11,177 / 11,230, `state=5511268fb5faf97f` (the unprobed hash, so the probe is neutral). Output: `13-8b-signoff/prototypes/split-30k-main.txt`.

Mean per tick over each 500-tick window, crew 200:

| tick | step ms | `claim_jobs` ms | failed goal searches | cells they flood | `cj_goals` ms | `execute_jobs` ms | stone-map rebuild ms | blocked step ms | re-plan ms | haul work positions ms |
|---|---|---|---|---|---|---|---|---|---|---|
| 19,000 | 42.0 | 1.4 | 0.03 | 2,046 | 0.02 | 40.2 | 3.2 | 29.0 | 2.8 | 3.5 |
| 22,000 | 43.0 | 14.9 | 4.5 | 24,036 | 0.2 | 27.6 | 5.1 | 9.8 | 6.6 | 4.1 |
| 24,000 | 51.8 | 21.9 | 9.6 | 35,467 | 0.3 | 29.3 | 6.6 | 9.2 | 6.7 | 4.4 |
| 26,000 | 51.0 | 28.0 | 15.6 | 45,147 | 0.3 | 22.2 | 7.9 | 4.3 | 3.3 | 4.7 |
| 28,000 | 52.5 | 22.9 | 15.2 | 30,414 | 4.2 | 28.6 | 9.7 | 6.8 | 3.5 | 4.9 |
| 30,000 | 53.1 | 23.4 | 25.4 | 28,850 | 5.0 | 28.4 | 10.6 | 7.5 | 2.5 | 5.3 |

- **#231's attribution was half right.** The stone-map rebuild is 3–11 ms and grows with loose stones; it is not the 22–41 ms the issue gave `execute_jobs`. The largest part just after the dig is `resolve_blocked_step`.
- **Claim searches: the goal leg fails, the delivery leg does not** (`n_deliv_fail` ≤ 1 a tick). The per-call `components` cache already skips about 95% of candidates (361–523 of 393–543) and costs under 0.05 ms. What is left is a flood of the main walkable component, about 63,000 cells, in most ticks: one job's unreachable goals, searched again every time a job comes off its 20-tick cooldown.
- **Single slow ticks** (`spikes-30k-main-loaded.txt`; that run was on a loaded machine, so read its counts, not its ms): one 63,000-cell flood; 129–151 re-plans in one tick after a dig (`clear_paths`); and about 712 haul jobs building goal sets in one tick.
- **Prototype of AC 2–4** (`kept-components-prototype.patch`, `kept-components-30k.txt`): failed components kept across calls while the terrain is unchanged and no cell has left `blocked`. `state=5511268fb5faf97f` and the end line are identical to `main`. The kept set is valid on 80–88% of calls (terrain resets 7–11%, a cell leaving `blocked` 5–9%). Cells flooded per tick: 24,036 → 8,010 (t22,000), 35,467 → 10,005, 45,147 → 8,195, 30,414 → 8,287, 28,850 → 6,135. That run was loaded too; its ms are not evidence.

## Acceptance Criteria

1. **Nothing a dwarf does changes.** Release `simd --bench` prints the same `state=` and end line as `main` `5f83e82e` on all 12 committed scenarios and on seat-like (`5511268fb5faf97f`, `marks=230 items=1530 … stored=497 … crew=200 arriving=0`). Emitter: `crates/simd/src/scenario.rs:410`.
2. **An unreachable job is not searched again on unchanged ground.** In a world with one dig no miner can reach and nothing else changing, the cells expanded by failed claim searches over 61 ticks total at most one flood of the miner's component. (On `main`: one flood per 20-tick cooldown.) The count is a `#[cfg(test)]` counter beside `CLAIM_SET_BUILDS` (`lib.rs:932`); adding it is Task 2.
3. **A kept failure is dropped when the ground opens** (seam exercised). After (i) a tile change that opens a route to that dig, and separately (ii) the removal of an item that alone blocked the route, the dig is claimed within `RETRY_COOLDOWN` plus the longest reaction delay (20 + 30 ticks). Each has its own test, and each fails if kept results are never dropped.
4. **On seat-like the floods fall.** With the probe re-applied on the story's head, `cells_goal_fail` is at most 12,000 a tick in every window from t22,000 to t30,000 (`main`: 24,036–45,147; prototype: 6,135–10,005).
5. **The stone map is built once per `execute_jobs` call.** In a tick where several dwarves hold jobs and no item is lifted, set down, dropped or spawned, it is built once (a `#[cfg(test)]` counter; on `main` it is once per job-holding dwarf, `lib.rs:2218-2219`). Mechanism AC, on purpose: in debug builds each dwarf's turn asserts the kept map and blocked set equal a fresh rebuild, so every scenario test checks the kept copy.
6. **Pile cells are computed once per `claim_jobs` call.** With K unclaimed hauls and one free hauler, one call computes the pile's free cells once, not 2K times (a `#[cfg(test)]` counter in `pile_targets`, `lib.rs:969`; RED on `main` reads 2K).
7. **Seat-like, measured.** On an idle machine, two runs each of `main` and the story's head: mean, p99, max, `over10`, and the per-window table above with the probe. `ej_stones_blocked` and `cj_goals` are each under 1 ms a tick in every window (`main`: up to 10.6 and 5.0).
8. **Bench delta (standing AC 10).** `RUNS=5 scripts/tick-bench.sh main HEAD` on the committed scenarios; the table is pasted. A regression BEYOND FLOOR needs Wolf's ruling.
9. **Saves are unaffected (standing AC 12).** `cargo test -p sim-core --test save_load` passes unchanged; no kept data is in `SaveState`.
10. **Mutation table.** `mutations/13-8b.sh` has one row per decision (Task 6), each KILLED by the named test at its intended assert.
11. **The seat.** Wolf runs seat-like at Normal past tick 17,000 with a perf log and rules on the stops. The answer is recorded; what remains goes to the Q1 follow-up issue.

## Tasks / Subtasks

- [ ] **Task 1 — baseline and RED (AC 1, 7).** Record `main`'s `state=` and end line for the 12 committed scenarios (the AC 1 loop) before changing code. Rebase `split-probe.patch` onto the branch head in a scratch worktree with its own `CARGO_TARGET_DIR` (build from inside the worktree). Run seat-like twice on an idle machine (`uptime` load under 1 before each run) and confirm the creation figures. This is the instrument's RED: the three costs are present.
- [ ] **Task 2 — kept failed components (AC 2, 3, 4).**
  - [ ] Add the `#[cfg(test)]` flood counter in `astar_with_budget`'s failure path (`lib.rs:1317`), then write AC 2's test and see it RED (more than one flood).
  - [ ] Keep the failed components between calls. They live with the terrain, as `pile_ground` does (`lib.rs:808`), and `set_tile` drops them (`lib.rs:848-853`). They are also dropped when any cell that was in `blocked` at the last call is not in it now. A cell entering `blocked` keeps them: closing a cell cannot make anything reachable.
  - [ ] Never keep an empty set (`astar_with_budget` returns one for empty goals, `lib.rs:1278`).
  - [ ] AC 3's two tests. Run each RED first with the drop removed.
- [ ] **Task 3 — one stone map per `execute_jobs` call (AC 5).** Build `stones` and `blocked` once before the loop (`lib.rs:2186-2219`) and keep both current at every item change inside it: the miner's lift (`:2457`) and set-down (`:2461-2462`), the haul pick-up and delivery arms, a dig's or cut's spawned items (`:1446`, `:2659`), and every `release_claim` / `retry_claim` that drops a carried item (`:1613`). Add the debug assert at the top of each dwarf's turn, and the counter test.
- [ ] **Task 4 — pile cells once per claim call (AC 6).** `terrain`, `blocked` and `zones` are fixed for the whole of one `claim_jobs` call, so compute `pile_targets` at most once per call and hand it to both haul sets (`lib.rs:688-704`, `work_positions` `lib.rs:1454`). `execute_jobs` keeps its per-dwarf call: items move inside its loop.
- [ ] **Task 5 — measure (AC 1, 4, 7, 8).** `state=` on the 12 files and seat-like against `main`. Seat-like with the probe, two idle runs per build, table pasted beside the creation table. `tick-bench.sh`, table pasted. Save the outputs under `13-8b-signoff/`.
- [ ] **Task 6 — mutation rows (AC 10).** One per decision: kept components survive `set_tile`; kept components survive a cell leaving `blocked`; the kept stone map misses a lift; misses a set-down; misses a spawned stone; `claim_jobs` reuses pile cells from the previous call. Run `scripts/audit-mutations.py` before the first code commit and after.
- [ ] **Task 7 — the instrument (observability rule).** The human instrument is `simd --bench <file> --runs 1`: its run line carries mean, p99, max, `over10` and `state=`. It is already tested against the real binary (`crates/simd/tests/bench.rs:110`, `:137`). Nothing new is built; the probe patch stays a prototype and is not merged.
- [ ] **Task 8 — the seat (AC 11).** Write `13-8b-signoff/vehicle-card.md`; Wolf runs it.
- [ ] **Task 9 — record.** File the Q1 follow-up issue (and Q3's, if ruled) with the figures. One full gate at PR.

## Verification

```bash
# AC 2, 3, 5, 6, 9 (fast tier)
cargo test -p sim-core
# AC 1: state= against main, 12 committed files + seat-like (release)
cargo build --release -p simd
for f in $(git ls-files 'scenarios/*.json') _bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json; do
  target/release/simd --bench "$f" --runs 1 | rg -o 'bench \S+ run.*state=\S+|bench \S+ end.*'
done
#   seat-like must print: state=5511268fb5faf97f
#   and: end tick=30000 marks=230 items=1530 zones=1800 goals=136/0/0 done=dig:1370,cut:30 stored=497 cells=2176 crew=200 arriving=0
# AC 4, 7: the probe, in a scratch worktree (never in this tree), idle machine
git worktree add --detach "$SCRATCH/wt" HEAD && cd "$SCRATCH/wt"
git apply --reject <repo>/_bmad-output/implementation-artifacts/13-8b-signoff/prototypes/split-probe.patch   # fix rejects by hand
CARGO_TARGET_DIR="$SCRATCH/tgt" cargo build --release -p simd
uptime   # load average under 1, or the ms are not evidence
"$SCRATCH/tgt/release/simd" --bench <repo>/_bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json --runs 1 2> split.txt
rg -N 'PROBE tick=(22000|24000|26000|28000|30000) ' -A1 split.txt \
  | rg -o 'tick=\d+|step_mean=[\d.]+|(cells_goal_fail|ej_stones_blocked|cj_goals|claim_jobs|execute_jobs)=[\d.]+'
# AC 8
RUNS=5 scripts/tick-bench.sh main HEAD
# AC 10
env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-8b.sh
python3 scripts/audit-mutations.py
# gate (once, at PR)
RUST_TEST_THREADS=1 scripts/gate.sh
```

Run at creation on `main` `5f83e82e`: the seat-like run and end lines printed exactly as shown (from the probe build, whose hash equals the unprobed one), and the probe printed the table under "Found at creation". The loop over the 12 committed files was not run at creation: Task 1 records `main`'s 12 hashes before any code changes. That table is the RED for AC 4 and AC 7 (`cells_goal_fail` 24,036–45,147; `ej_stones_blocked` 3.2–10.6; `cj_goals` up to 5.0). The dev agent must show each of those three drop on the story's head, and must show the instrument can still fail: with Task 2's keep disabled in the scratch worktree, `cells_goal_fail` returns to the `main` range.

Seat (AC 11), written as the seat runs it:

```
# WSL
simd 7451 --scenario _bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-8b-seat-like.csv')
```

## Dev Notes

### Scope guardrails (do NOT)

- Do not change `RETRY_COOLDOWN`, the search budget, `clear_paths`, `resolve_blocked_step` or any movement rule. A changed `state=` on any file is a defect in this story, not a re-pin.
- Do not replace the A* or its data structures. A faster search is a different story.
- Do not put kept data in a `static` or `thread_local`: tests build many worlds in one process, and the prototype's static is prototype-only.
- Do not save any kept data, and do not merge the probe.
- Do not touch the haul failures of Q3.

### What already exists (build on it)

- `Terrain.pile_ground: Mutex<PileGround>` (`lib.rs:808-823`): a cache inside the terrain resource, dropped by `set_tile`. Task 2's kept components follow it.
- `claim_jobs`' per-call `components` and its skip test (`lib.rs:663`, `:720-726`): Task 2 seeds this vector, nothing else changes.
- `#[cfg(test)]` counters `FLOODED`, `MEMO_HITS`, `CLAIM_SET_BUILDS` (`lib.rs:926-932`) and their tests (`lib.rs:5788-5800`): the pattern for AC 2, 5 and 6. Run a counted system alone in a `Schedule` with `SingleThreadedExecutor`, or the thread-local reads 0.
- `unreachable_digs_never_starve_a_reachable_one` (`tests/scenario.rs:783`) and `a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley` (`lib.rs:5857`): they must stay green untouched.

### Key decisions & traps

- **Why a cell entering `blocked` keeps the kept failures.** A stone set down or spawned only removes walkable cells. Every position in an old failed component is still unable to reach those goals. The prototype's 80–88% hit rate depends on this; resetting on any change of `blocked` would lose most of it.
- **The kept components are a superset after a stone lands.** A dwarf may now stand in a smaller pocket of an old component. The skip stays correct (he still cannot reach the goals) and no new flood is needed.
- **The debug assert in Task 3 keeps the old cost in debug builds only.** That is intended: tests pay it, release does not.
- **`release_claim` is called from inside the `execute_jobs` loop for other dwarves** (`lib.rs:2441-2456`, a hauler losing custody at a miner's lift). It can drop a carried stone, so the kept map must see it.
- **Timings need an idle machine.** Two of the four creation runs read 60% high under a load average near 3. Counts and `state=` are deterministic; quote ms only from runs with `uptime` under 1.
- **`sim-core` is now built at opt-level 3 in dev and test builds** (PR #234). Test timings from before 2026-11-12 do not compare.

### Previous story intelligence

- 13.8's AC 14 is the model: a skip proved by a test counter, a RED, an unchanged `state=` on every file, and the probe re-run (`13-8-they-keep-coming.md`, Task 11).
- A `KILLED` row names the test, not the assert: read the panic line for each row.
- Deleting or moving a line that an older mutation row anchors on turns the hook's audit RED: run `audit-mutations.py` before the first code commit.

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs` (`Terrain`, `set_tile`, `claim_jobs`, `execute_jobs`, `work_positions` / `pile_targets` call sites, tests).
- NEW `_bmad-output/implementation-artifacts/mutations/13-8b.sh`.
- NEW under `_bmad-output/implementation-artifacts/13-8b-signoff/`: the Task 5 outputs, `vehicle-card.md`.
- No change to `protocol`, `simd`, `tui`, `client-core` or `gui`.

### References

- Issue #231; `13-8-they-keep-coming.md` (AC 14, Task 11, seat runs 2 and 3).
- `13-8b-signoff/prototypes/`: `split-probe.patch`, `split-30k-main.txt`, `spikes-30k-main-loaded.txt`, `kept-components-prototype.patch`, `kept-components-30k.txt`.
- `13-8-signoff/prototypes/seat-like.json` (the scenario; not a tick-bench row).
- `epics.md`, Epic 13 standing ACs and NFR12; `docs/technical-preferences.md` (optimize only after a profile).

## Dev Agent Record

### Agent Model Used

Wolf picked Sonnet 5.5 subagents for the implementation (asked 2026-11-13). claude-fable-5-2 orchestrates and verifies, and keeps the REDs, the mutation run, the bench and the full gate.

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-11-12 | Created (claude-fable-5-2) on `story-13-8b-after-the-big-dig` off `main` `5f83e82e`. Four creation probes on seat-like; Task 0 Q1–Q3 await Wolf. |
| 2026-11-13 | Task 0 ruled (Wolf): Q1 (a) three neutral cuts, blocked-step and re-plan burst to a follow-up issue at Task 9; Q2 slot 13.8b accepted, epic count 17; Q3 haul failures filed as #235. Implementer: Sonnet 5.5 subagents, claude-fable-5-2 orchestrates. |
