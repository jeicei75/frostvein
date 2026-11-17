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

- [x] **Task 1 — baseline and RED (AC 1, 7).** Record `main`'s `state=` and end line for the 12 committed scenarios (the AC 1 loop) before changing code. Rebase `split-probe.patch` onto the branch head in a scratch worktree with its own `CARGO_TARGET_DIR` (build from inside the worktree). Run seat-like twice on an idle machine (`uptime` load under 1 before each run) and confirm the creation figures. This is the instrument's RED: the three costs are present.
- [x] **Task 2 — kept failed components (AC 2, 3, 4).**
  - [x] Add the `#[cfg(test)]` flood counter in `astar_with_budget`'s failure path (`lib.rs:1317`), then write AC 2's test and see it RED (more than one flood).
  - [x] Keep the failed components between calls. They live with the terrain, as `pile_ground` does (`lib.rs:808`), and `set_tile` drops them (`lib.rs:848-853`). They are also dropped when any cell that was in `blocked` at the last call is not in it now. A cell entering `blocked` keeps them: closing a cell cannot make anything reachable.
  - [x] Never keep an empty set (`astar_with_budget` returns one for empty goals, `lib.rs:1278`).
  - [x] AC 3's two tests. Run each RED first with the drop removed.
- [x] **Task 3 — one stone map per `execute_jobs` call (AC 5).** Build `stones` and `blocked` once before the loop (`lib.rs:2186-2219`) and keep both current at every item change inside it: the miner's lift (`:2457`) and set-down (`:2461-2462`), the haul pick-up and delivery arms, a dig's or cut's spawned items (`:1446`, `:2659`), and every `release_claim` / `retry_claim` that drops a carried item (`:1613`). Add the debug assert at the top of each dwarf's turn, and the counter test.
- [x] **Task 4 — pile cells once per claim call (AC 6).** `terrain`, `blocked` and `zones` are fixed for the whole of one `claim_jobs` call, so compute `pile_targets` at most once per call and hand it to both haul sets (`lib.rs:688-704`, `work_positions` `lib.rs:1454`). `execute_jobs` keeps its per-dwarf call: items move inside its loop.
- [x] **Task 5 — measure (AC 1, 4, 7, 8).** `state=` on the 12 files and seat-like against `main`. Seat-like with the probe, two idle runs per build, table pasted beside the creation table. `tick-bench.sh`, table pasted. Save the outputs under `13-8b-signoff/`.
- [x] **Task 6 — mutation rows (AC 10).** One per decision: kept components survive `set_tile`; kept components survive a cell leaving `blocked`; the kept stone map misses a lift; misses a set-down; misses a spawned stone; `claim_jobs` reuses pile cells from the previous call. Run `scripts/audit-mutations.py` before the first code commit and after.
- [x] **Task 7 — the instrument (observability rule).** The human instrument is `simd --bench <file> --runs 1`: its run line carries mean, p99, max, `over10` and `state=`. It is already tested against the real binary (`crates/simd/tests/bench.rs:110`, `:137`). Nothing new is built; the probe patch stays a prototype and is not merged.
- [ ] **Task 8 — the seat (AC 11).** Write `13-8b-signoff/vehicle-card.md`; Wolf runs it.
- [ ] **Task 9 — record.** File the Q1 follow-up issue (and Q3's, if ruled) with the figures. One full gate at PR.

## Amendment 2026-11-15: the stops (Wolf's rulings)

Seat run 2 (card, Sitting Run 2): "it's better but after 21 elapsed dwarwes started to stop briefly (not all at once) ... it does not look smooth enough still". Asked close or widen: **"I think we will continue until we are happy"**. On the head-on finding: **"I think we should include 236 also to this fix?"** So the story now changes what dwarves do, and #236 is in scope. **AC 1 is AMENDED:** it holds for Tasks 1-7 (`89be48c7`: `state=` identical); the tasks below change `state=` on purpose.

### Found (headless, seat-like, probes in `13-8b-signoff/prototypes/`)

- **Who makes a job-walking dwarf wait** (`why-probe.patch`, `why-probe-30k.txt`; probe is neutral, `state=5511268fb5faf97f`). Per 1,000 ticks: two job-holders head-on 415 at t12,000 and 1,394-2,339 after the dig; a job-holding blocker not head-on with no way round 5 and 571-2,911; an idle blocker under 40 throughout. So #227's camp crowd is NOT the cause. The head-on rule (`resolve_blocked_step`, `if !idle { try_squeeze(..) }`) never tries anything but the `SQUEEZE_WAIT_TICKS` wait.
- **Prototype: head-on job-holders swap on the first look** (`why-probe-swap-now.patch`, `why-probe-swap-now-30k.txt`). Stalls per 1,000 ticks 266 → 8 at t12,000; 1,127-1,641 → 190-515 after the dig. End: `done=dig:1430` (was 1370), `stored=552` (was 497), mean 10.2 ms (was 11.6), `state=fca7c3e18fa333c1`.
- **The slowest ticks are the re-plan burst** (`swap-now-slow-ticks-30k.txt`): of 61 ticks over 150 ms, 56 are dominated by the `execute_jobs` re-plan after `clear_paths` (74% of their time; 80-138 searches in the tick), 4 by `resolve_blocked_step` (10%). #236's guess holds.
- **Prototype: keep a path across a dig unless it passes within 2 cells of the target** (`swap-keep-paths-probe.patch`, `KEEP_PATHS=1`, `swap-keep-paths-30k.txt`). Re-plans fall 80-95%, but from t23,000 "no way round" waits climb to 9,000 per 1,000 ticks and the dig stalls: `done=dig:1027`, mean 19.4 ms. **It jams; the cause is not found.** Not a fix as built.
- **One miner waits for good** from about t24,000: he carries a cleared stone and `clearing_set_down` finds nowhere to put it (`carry_wait_ticks` about 1,000 per 1,000 ticks). Its own bug.

### Acceptance Criteria (added)

12. **Two job-holders who meet head-on trade tiles on the first look.** A holder whose next tile holds a job-holder whose path starts on the holder's tile swaps with him in that call; neither waits `SQUEEZE_WAIT_TICKS`. An idle blocker's wait is unchanged. Test first, RED on `19bab45a` (the swap comes one step period later).
13. **On seat-like the stalls fall** (`stall-probe.patch` re-applied; a stall is a job-holder in `Walk` on one tile for more than 12 ticks). Per 1,000 ticks: at most 30 in every window from t7,000 to t13,000 (now 105-267), at most 600 in every window from t19,000 (now 1,127-1,641).
14. **The re-plan burst is cut without losing work.** On seat-like, idle machine, two runs: no tick over 250 ms (now 369-446), and `done=dig` at t30,000 is not below AC 12's build alone (1,430). The rule is the dev's to find; the keep-paths prototype's jam is diagnosed first (Task 11) and the diagnosis recorded.
15. **Blocking, measured after 12 and 14.** `resolve_blocked_step` ms per tick and the "no way round" waits per window, pasted beside today's (3-11 ms; 189-1,285). Whether it needs its own cut is Wolf's ruling on those figures.
16. **Outcomes are re-read, not assumed** (standing AC 11). Every committed scenario's end line is re-run on the new build and listed beside the old; none does less work (`done=`, `stored=`). Re-pinned tests and 13.4's mutation rows that name the head-on wait are re-pointed or retired in the same commit as the code, each re-run (KILLED, or retired with the reason). Saves still load (`Squeeze` keeps its fields).
17. **Mutation rows** for each new decision, and **seat run 3**: Wolf runs seat-like with the tap and rules on what he sees.

### Tasks (added)

- [ ] **Task 10 — head-on swap at once (AC 12, 13, 16).** Test first. Then the audit (`scripts/audit-mutations.py`) before the first code commit, because rows against 13.4's wait go RED with it.
- [x] **Task 11 — the re-plan burst (AC 14).** Diagnose the keep-paths jam on the prototype (which dwarf first waits for good, on what, and why a re-plan would have freed him). Then the rule, test first.
- [x] **Task 12 — measure (AC 13, 14, 15, 16).** Stall probe, slow-tick split, end lines of every committed scenario, `tick-bench.sh`.
- [x] **Task 13 — mutation rows (AC 17).**
- [ ] **Task 14 — seat run 3 (AC 17).** Card section, pushed build, Wolf's words recorded.
- [x] **Task 15 — record.** File the stuck stone-carrying miner as its own issue; comment on #236 with what this story closed of it; #219 (swap animation) gets a note that swaps are now more frequent.

## Amendment 2026-11-16: the seat falls behind (Wolf's rulings)

Seat run 3's tap: after the big dig the seat's daemon falls behind real time (115-128 s per 1,000 ticks, up to 327 ticks per 1,000 over 150 ms late, longest gap 581 ms). The same build on the devpod keeps real time, but with almost no headroom (100.1-101.1 s). The daemon logs no tick times, so it is not known whether the seat's time goes to the step, to encoding the ~150 KB delta, or to the machine (gui, WSL, CPU). Two lines were put to Wolf. He ruled **"Both, timer first"**: the daemon gets its own per-tick timer, and then the tick after the dig gets cheaper. Implementer: **Opus 5.5 agents**, claude-fable-5-2 orchestrating.

### Acceptance Criteria (added)

18. **The daemon times its own ticks.** Every 1,000 stepped ticks, `simd` writes one line to stderr. The line gives the tick range, the wall seconds those ticks took, mean and max ms of `world.step()`, mean and max ms of delta encoding plus broadcast, and how many iterations overran their period. Paused iterations are not counted. A serve test reads the line from the real binary, pins its fields, and checks that `step` is non-zero.
19. **Baseline before the cut.** The AC 18 line is recorded on seat-like at `8c864b93`'s sim on an idle devpod: release live daemon at Fast4x, the tap as its only client, to t30,000. Two runs, saved under `13-8b-signoff/`. This is the "before" for AC 20, and it splits step from encoding.
20. **The tick after the dig is at least twice as cheap.** Same run as AC 19, two runs on the new build. In every 1,000-tick window from t19,000 to t30,000, the mean of step plus encoding is at most half of AC 19's for that window. The max tick is no worse. If AC 19 shows encoding is the larger part, the cut goes there first (a wire change: stop and ask Wolf). Outcomes come under AC 16: `done=dig` at t30,000 is not below 1,486, and every committed scenario's end line is re-read.
21. **Seat run 4.** Wolf runs seat-like with the gui, the tap and the daemon's stderr saved to a file. If he has time, he does a second run with the tap only (no gui), to learn the gui's share. He rules on what he sees.

### Tasks (added)

- [x] **Task 16 — the daemon's tick timer (AC 18, 19).** Test first. Write AC 19's two baseline runs before any Task 17 code.
- [x] **Task 17 — cut the post-dig tick (AC 20).** The rule is the dev's to find. Profile before choosing. The leads are the wait-cause probe (`prototypes/wait-cause-30k.txt`: a walker with a free tile ahead that still stands, 15-20%; rings that do not step, 16%; clearers with no path, 17%), the blocking cost (0.5-15 ms a tick, `blocking-a28a5523.txt`) and the re-plans. Neutral cuts come first (`state=` unchanged). A behaviour change goes through AC 16.
- [x] **Task 18 — mutation rows** for each decision of Tasks 16-17.
- [ ] **Task 19 — seat run 4 (AC 21).** Card section, pushed build, Wolf's words recorded.

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

Amendment tasks 10-13 (asked 2026-11-15): Wolf picked **Opus 5.5** subagents. claude-fable-5-2 orchestrates and verifies as before, and ran the probes behind the amendment.

### Debug Log References

All outputs are under `13-8b-signoff/`. Code head for every figure: `89be48c7`.

- **Task 1, baseline.** `state-main-5f83e82e.txt`: `state=` and end line of the 12 committed scenarios and seat-like on `main`'s code (seat-like `5511268fb5faf97f`, end line as in the Verification block). The probe applied cleanly on that code and reproduced the creation counts exactly (`cells_goal_fail` 24,036 / 35,467 / 45,147 / 30,414 / 28,850). That first run was on a loaded machine (load 2.7 to 5.4), so its ms are not quoted; the idle ms for `main` are the two Task 5 runs.
- **REDs (Sonnet agent, before each fix).** AC 2: `one flood of the plate, however many cooldowns  left: 1200  right: 400` (three floods in 61 ticks). AC 6: `left: 6  right: 1` (2K for K = 3). AC 5: `left: 3  right: 1` (one build per job-holding dwarf). AC 3 (i) with `set_tile`'s drop removed: `the opened route must be claimed within 50 ticks`. AC 3 (ii) with the subset test always true: `the dig must be claimed within 50 ticks of the stone leaving`.
- **AC 1.** `state-head-89be48c7.txt` against the baseline: `state=` and end line IDENTICAL on all 13 files.
- **AC 4, AC 7.** Probe rebased by hand onto the head (4 rejected hunks; `prototypes/split-probe-on-89be48c7.patch`). Two runs per build, alternating, load average under 1 before each (0.83 to 0.91; the runs themselves raise it). Files `split-30k-{main,head}-run{1,2}.txt`. Both runs of a build agree.

  | | `main` | head |
  |---|---|---|
  | mean ms | 20.23 / 20.35 | 11.78 / 11.88 |
  | p99 ms | 130.7 / 132.2 | 105.5 / 106.9 |
  | max ms | 403 / 397 | 404 / 406 |
  | `over10` | 11,456 / 11,375 | 5,970 / 6,018 |
  | `state=` | `5511268fb5faf97f` | `5511268fb5faf97f` |

  Run 1 of each build, mean per tick over each 500-tick window:

  | tick | step ms `main` → head | `cells_goal_fail` | `claim_jobs` ms | `ej_stones_blocked` ms | `cj_goals` ms | `execute_jobs` ms |
  |---|---|---|---|---|---|---|
  | 19,000 | 43.4 → 39.1 | 2,046 → 2,046 | 1.5 → 1.5 | 3.25 → 0.04 | 0.02 → 0.01 | 41.5 → 37.2 |
  | 22,000 | 44.7 → 27.9 | 24,036 → 8,010 | 15.6 → 5.3 | 5.22 → 0.07 | 0.23 → 0.19 | 28.5 → 22.0 |
  | 24,000 | 54.2 → 29.5 | 35,467 → 10,005 | 23.0 → 6.6 | 6.85 → 0.09 | 0.28 → 0.20 | 30.5 → 22.1 |
  | 26,000 | 53.6 → 19.6 | 45,147 → 8,195 | 29.4 → 5.6 | 8.20 → 0.10 | 0.28 → 0.18 | 23.3 → 13.1 |
  | 28,000 | 54.4 → 24.9 | 30,414 → 8,287 | 23.9 → 6.0 | 9.97 → 0.12 | 4.22 → 0.54 | 29.5 → 17.8 |
  | 30,000 | 54.5 → 22.5 | 28,850 → 6,135 | 24.0 → 4.7 | 10.76 → 0.12 | 5.28 → 0.54 | 29.2 → 16.5 |

  AC 4: the largest head window is 10,005, under 12,000; the head counts equal the prototype's. AC 7: `ej_stones_blocked` at most 0.13 and `cj_goals` at most 0.55 in every window. The stone map is built about 1.2 to 1.3 times a tick (`n_stone_map_build`) for 123 to 155 job holders. What is left is `resolve_blocked_step` (4.5 to 30 ms) and the re-plan (2.6 to 7 ms): #236.
- **The instrument can still fail.** With the keep disabled in the probed scratch build (`if false && was_blocked.is_subset(..)`), `cells_goal_fail` reads 24,036 / 35,467 / 45,147 / 30,414 / 28,850 again, `state=` unchanged (`split-30k-head-keep-disabled.txt`; loaded machine, counts only).
- **AC 8.** `RUNS=5 scripts/tick-bench.sh main HEAD` (`tick-bench-main-89be48c7.txt`; load 4.8 at the start, runs interleaved, control within ±1.8%). `state` is `same` on all 12. No regression: big-pile -35.7%, seat-slow -15.7%, two-hundred-on-256 -11.9% (23.46 → 20.68 ms), plan-compact-piles -11.8%, plan -9.6%, deep-pit -3.8%, all BEYOND FLOOR; the other six -0.8% to -8.0%, within floor.
- **AC 9.** `cargo test -p sim-core` green with `save_load` unchanged (24 tests); `SaveState` is untouched.
- **AC 10.** `mutations/13-8b.sh`, 12 rows, 12/12 KILLED, each panic line read. Rows 1-3 die at the AC 2 / AC 3 asserts (`lib.rs:6013`, `:6040`, `:6087`), rows 10-11 at the counter asserts (`:6164`, `:6136`), rows 4-9 and 12 at the debug assert `the kept stone map is current` (`lib.rs:2318`). Before writing the table each stale-mark removal was run against the whole of `cargo test -p sim-core` in a scratch worktree: the lift, the set-down and the cut pass all 117 unit tests and are caught only by `tests/scenario.rs` (8, 8 and 3 tests). `audit-mutations.py`: 1006 rows match.

### Completion Notes List

- **Task 2** (`f3fa3f8b`): `Terrain::kept_failures: Mutex<(BTreeSet<Pos>, Vec<BTreeSet<Pos>>)>` holds the last call's `blocked` and its failed components. `set_tile` clears the components; `claim_jobs` uses them only when the stored `blocked` is a subset of its own, and stores back with empty sets removed. Tests: `an_unreachable_dig_is_flooded_once_not_once_per_cooldown`, `a_tile_that_opens_the_route_drops_the_kept_failure`, `a_stone_hauled_off_the_only_route_drops_the_kept_failure` (the stone leaves by the sim's own haul job).
- **Task 4** (`d253f491`): `work_positions` is a wrapper over `work_positions_with(.., pile: &mut Option<..>)`; `claim_jobs` holds one `pile` per call. Test: `claim_jobs_finds_the_pile_cells_once_per_call`.
- **Task 3** (`89be48c7`): `execute_jobs` keeps one stone map and a `stale` flag. Every item change sets the flag and the next dwarf's turn rebuilds. Test: `execute_jobs_builds_the_stone_map_once_when_no_stone_moves`. Debug assertions are on in the test profile (shown: removing one stale mark fails `cargo test`).
- **Deviations, for the review.**
  1. Task 3 says "keep both current at every item change"; the code rebuilds after a change instead of editing the map in place. Measured cost: about 1.25 builds a tick, 0.04 to 0.13 ms. AC 5 as written is met.
  2. The AC 2 counter sits at the end of `claim_jobs`, not in `astar_with_budget`: other systems' failed searches cannot reach it, and two older mutation anchors stay intact.
  3. `#[allow(clippy::needless_borrow)]` on `execute_jobs`, with a NOTE: `stones` and `blocked` are borrows now, and the old `&blocked` / `&stones` spellings are anchors of older mutation rows (12-9, 13-7 and others). The alternative is to re-point those rows.
  4. Five test fixtures that build a `Terrain` literal gained the line `kept_failures: Default::default()`. No test logic changed.
  5. The story's mutation row "`claim_jobs` reuses pile cells from the previous call" cannot be written: `pile` is a local of one call. Row 11 pins the other side (the delivery set finding them again). Rows 7-9 and 12 are extra: every stale site has a row.
  6. Task 1's probe run was on a loaded machine (the agent was building); counts only. The idle `main` ms come from Task 5's two runs.
- **Task 7:** nothing built, as written. The probe stays a prototype.
- **Task 8:** `13-8b-signoff/vehicle-card.md` is written. **Sat 2026-11-13 on `6b6a041f` (Wolf): "we still have issue"; against 13.8's run 3, "less but still visible"; and "I don't thik we are done with the story".** Task 8 and AC 11 stay open. The seat perf log shows no gui freeze (no frame over 250 ms after startup) and the same dig cadence as run 3 (53 against 56 pauses over 3 s between digs landing, longest 8.0 s in both). A devpod wire tap on 2026-11-14 (`scripts/bench/wire_tap.py`, `13-8b-signoff/wire-tap-devpod-6b6a041f.csv`) shows the daemon itself does not stop: longest silence between two ticks 0.38 s to tick 24,500, 10 s lost to real time after the dig; the per-tick message grows from about 81 KB to 133 to 144 KB at the dig. So the multi-second stops are not in the devpod's sim; the card's Run 2 puts the same tap on the seat to tell the gui from the connection and the seat's sim. Figures: the card's Sitting. #237 (drawn dwarves snapping) waits behind this by Wolf's word: "let's focus on fixing perfomnace first".
- **Task 8, seat run 2 (2026-11-15, `19bab45a`, Wolf):** "it's better but after 21 elapsed dwarwes started to stop briefly (not all at once).. so I just wonder is it actually perf issue or more like decision what next takes longer when more dwarves around.. it does not look smooth enough still". The seat tap (`13-8b-signoff/13-8b-wire-tap-seat.csv`) shows an even daemon at minute 21 (no gap over 227 ms) and no multi-second silence in the run; after the big dig single ticks reach 0.97 s and the world runs at 83 to 98% of real time. A headless stall probe (`prototypes/stall-probe-30k.txt`) shows the sim itself holds job-walking dwarves on their tile: about 270 stalls per 100 s at crew 150, 1,000 to 1,600 after the big dig (120 to 150 of 200 dwarves, the longest 138 s), most one squeeze wait long. So Wolf's reading holds: mostly a movement-rule matter, plus a smaller tick-spike part after the dig. **Wolf ruled the story continues: "I think we will continue until we are happy".** The scope widens past AC 1 ("nothing a dwarf does changes"); the new ACs and tasks are written after the attribution probe. Figures: the card's Sitting, Run 2.
- **Task 10 (in progress), commit `55cc430b` (Opus 5.5 agent; fast gate green, pushed):** two job-holders head-on trade tiles on the first look (`try_squeeze` waits only for an idle blocker). RED of `two_job_holders_head_on_trade_tiles_on_the_first_look` on `92dd760c`: `lib.rs:8325 left: Pos { x: 13, y: 20, z: 5 } right: Pos { x: 14, y: 20, z: 5 }` (the holder had not moved on his first look). The new timing turned three scenario tests red: (1) `a_stockpile_around_the_campfire_never_zones_or_receives_the_fire`, 101 pick-ups after t2500 (#189's loop): an idle dwarf stepped aside onto the last free pile cell and called it home, so every delivery to it was refused; fixed in the same commit (case 1's aside and exit never end on a free pile cell), unit test `an_idle_dwarf_moved_out_of_the_way_never_lands_on_a_free_pile_cell` RED `lib.rs:8503 left (9,19,5) right (9,21,5)`. (2) `a_carried_stone_is_always_set_down_beside_its_carrier`, a hauler's refused delivery dropped a stone 4 cells away: NOT fixed, filed **#238** (`route:undecided`), NOTE at the test; it passes again by timing. (3) the cage sweep re-pinned, each entry traced: islands 5 → 3 (#186), unfinished at t4,000 5 → 4, at t16,000 4 → 2 (#182 mode ii, #218). `a_held_dwarf_swaps_only_after_the_wait` now pins the idle wait only; the stand-off save test saves an idle stand-off.
- **Task 10, measured on `55cc430b`** (`prototypes/task10-55cc430b-probes-30k.txt`; ms taken while a mutation run was live, so only the counts are evidence): stalls per 1,000 ticks t7,000-13,000 at most 36 (AC 13 asks 30; was 105-267); after t19,000 227-725 (AC 13 asks 600), dwarf-ticks lost standing 265,172 (before the change 245,324), longest stall 2,496 ticks. `state=96283c7ab8118370`, `done=dig:1457 stored=522`. The pile-cell fix leaves less room to step aside, and rings of job-holders gridlock. With a ring-step prototype on top (`task10-plus-ring-probe.patch`, `task10-plus-ring-30k.txt`): 103-557 per window, lost 99,984, longest 516, `done=dig:1486`. So the ring rule goes into Task 11. **AC 13 is NOT met on `55cc430b` alone.**
- **Task 10, mutation re-run on `55cc430b`** (`mutate.sh`, RUST_TEST_THREADS=1): `13-4.sh` 11/11 KILLED; `13-8b.sh` 12/12 KILLED; `12-9.sh` 30 KILLED, **row 4 "the per-step flip rule" SURVIVED** (its test is a head-on job-holder pair that no longer reaches the yield); `12-10b.sh` 20 KILLED, **row 5c NO-COMPILE** (a gui sabotage, `crates/gui/src/project.rs`, code this story does not touch). Both are Task 13's.
- **Task 11, found before dev** (`prototypes/ring-keep-paths-30k.txt`, `keep-paths-jam-stuck-30k.txt`): the keep-paths jam is a ring (four job-holders round a 2x2 block, each waiting on the next, 25 queued behind). The slowest ticks are FAILED re-plans: 34 of 36 ticks over 200 ms hold 5-11 failures, each flooding the about 63,000-cell component (successes cost hundreds of nodes). So Task 11 is: a failed re-plan's component is flooded once (neutral), then the ring rule. Keep-paths is dropped unless the figures still ask for it. The miner stuck holding a cleared stone is filed as **#239**.
- **Task 11 DONE, commits `5969d7fc` and `a28a5523`** (Opus 5.5 agent in its own worktree, cherry-picked; fast gate green on each, pushed). (A) `execute_jobs` keeps the components its failed re-plans flooded in a per-call list, dropped with the stone map on `stale`; a later failed re-plan starting inside one, with no goal in it, searches nothing. RED of `failed_re_plans_in_one_component_flood_it_once` on `55cc430b`: `left: 800 right: 400`. `a_stone_lifted_mid_call_lets_a_later_re_plan_through` RED with `failed.clear()` removed by hand: `lib.rs:6276 miner 2 still holds his dig left: None right: Some(JobId(900))`. Neutral: `state=` and end lines identical on the 12 committed scenarios and seat-like (`96283c7ab8118370`). (B) `step_the_ring`: where case 2's re-route fails, the chain blocker → the dwarf on his next tile → ... is followed (job-holders in `Walk` with a legal next step only); a chain that closes on the holder steps every member at once. Smallest ring is four. RED of `a_ring_of_waiting_job_holders_steps_at_once` on `5969d7fc`: `lib.rs:8544 the holder stepped onto b left: (5,5,1) right: (6,5,1)`. `a_chain_ending_on_a_dwarf_working_in_place_moves_nobody` RED with the `Walk` check removed by hand: `lib.rs:8654`. No pinned test moved. Rows re-anchored, NOT yet re-run: `13-7.sh` row 4, `13-8b.sh` row 10.
- **Task 12, measured on `a28a5523`** (idle devpod, load 1.2-3.8): seat-like two plain runs `mean=10.09 p99=92.8 max=188.2` and `mean=10.10 p99=92.5 max=188.5`, `state=250b5bbeee340ff9`, `done=dig:1486 stored=538` (**AC 14 met**: no tick over 250 ms; was 369-446; digs above 1,430). Stall probe (`13-8b-signoff/stall-probe-a28a5523.txt`): t7,000-13,000 at most 36 per 1,000 ticks (**AC 13 asks 30: missed by 6 in one window**; was 105-267); from t19,000 103-557 (AC 13 asks 600: met; was 1,127-1,641); dwarf-ticks lost standing after the dig 99,984 (was 245,324), longest stall 516 ticks (was 1,384). **AC 16, end lines old (`89be48c7`) → new (`state-head-a28a5523.txt`):** nine crew-5 files unchanged in `done=` and `stored=`; two-hundred-on-256 dig 234 → 321, cut 90 → 96, stored 27 → 36; **big-pile dig 34 → 32, stored 16 → 27, marks 12 → 14: two fewer digs, so AC 16's "none does less work" is NOT met for big-pile** (the #218 freeze scenario; not traced yet). Not yet done: AC 15's blocking figures, `tick-bench.sh`.
- **AC 15, blocking measured on `a28a5523`** (`13-8b-signoff/blocking-a28a5523.txt`, probe `prototypes/blocking-probe-on-a28a5523.patch`, neutral, load 1.9-2.6; a loaded first run gave the same counts). Per 1,000-tick window after the dig (t19,000-30,000): `resolve_blocked_step` 0.5-15.1 ms a tick (before: 3-11), "no way round" waits 76-1,945 (before: 189-1,285); the two worst windows are t19,000 (15.1 ms, 1,945) and t24,000 (12.7 ms, 1,059); re-plan 0.5-5.6 ms a tick. Before the dig: at most 0.17 ms and 11 waits. So blocking is NOT cheaper after this story: it is still the largest part after the big dig, and the first window is worse. The count includes looks where the ring then stepped. Whether it gets its own cut is Wolf's ruling.
- **AC 16, big-pile traced** (`13-8b-signoff/big-pile-over-time.txt`; the scenario's `ticks` raised, nothing else): the old build (`89be48c7`) stops for good at dig 35 of 46, stored 16 (the same line at t6,000, 8,000, 12,000 and 20,000: #218's freeze). The new build is two digs behind at the file's own end (t5,006: 32 against 34) but does not freeze: 36 at t6,000, 44 at t7,000, all 46 dug at t8,000 and all 46 stored at t12,000. So the two fewer digs are a snapshot of a slower start, not lost work. The letter of AC 16 ("none does less work" at the end line) is still not met for big-pile at t5,006; Wolf's ruling.
- **Task 12 closed: `RUNS=5 scripts/tick-bench.sh main HEAD`** (`13-8b-signoff/tick-bench-main-4048f3f1.txt`, `main` `5f83e82e` against `4048f3f1`, load 1.3-4.1): no scenario is slower. two-hundred-on-256 23.03 → 17.48 ms (-24.1%, BEYOND FLOOR, control -0.0%), big-pile -62.5%, the plan files -10% to -14.5%, seat-slow -11.2%, they-keep-coming -1.2% (same state, within floor); deep-pit, plan-spread-piles and stuck-tunnel faster but within floor. The measuring is done; two figures wait on Wolf: AC 13's pre-dig window (36 against 30) and AC 16's big-pile snapshot, so Task 10 stays open.
- **Task 15 DONE:** #239 filed (the miner stuck holding a stone), comment on #236 with what the story closed and what is left (blocking not cheaper, the burst still there), note on #219 that swaps are now frequent. The seat's answer goes on #236 after run 3.
- **Seat run 3 (2026-11-15), Wolf:** "still the same issue after the big designation target is assigned.." So AC 17 is NOT met after the big dig, and the blocking that AC 15 measured as not cheaper is what is left. Under Wolf's standing ruling ("I think we will continue until we are happy") the story goes on: diagnose what the dwarves wait on after the dig, then cut it. Questions 1 and 3 of the card are unanswered; the two files have not arrived.
- **Seat run 3, the file (2026-11-15), Wolf:** "between 16-17 it started to struggle but then it recovered later on in the end when I came back to watch.. csv is in signoff folder". Only the tap arrived (`13-8b-signoff/13-8b-wire-tap-seat-run3.csv`). It shows the seat's daemon falls behind real time after the big dig: 1,000 ticks take 115 s at t19,000 and 117-128 s from t22,000 to t27,000 (100 s is real time), with 250-327 of each 1,000 ticks arriving over 150 ms late; back to 100.8 s at t30,000. Run 2 was behind in fewer windows (t22,000-23,000 and t28,000-30,000) but had the longer single gap (967 ms against 581 ms). **So what Wolf sees after the dig is also lag at the seat, not only dwarves waiting on each other, and the devpod bench (20-30 ms a tick there) does not show it.** Not known yet: whether the seat's daemon is slower in the step or in what the bench does not time (encoding about 150 KB a tick for each client). A live devpod run with a tap is the next measurement. The wait-cause probe (`prototypes/wait-cause-30k.txt`): of the standing time after the dig, 16% is queueing behind a digging dwarf; the rest ends on a dwarf with a free tile ahead (20%), no route (17%), a ring that does not step (16%), a free tile ahead of the dwarf himself (15%), an idle dwarf (7%), a head-on job-holder (5%). Not traced yet.
- **Live devpod run with the tap on `a28a5523`** (`13-8b-signoff/wire-tap-devpod-a28a5523.csv`; release daemon at Normal, the tap its only client, 3,009 s to t30,000): the devpod's daemon keeps real time through the whole run. After the big dig 1,000 ticks take 100.1-101.1 s (the seat: 101-128 s), 29-89 ticks per 1,000 arrive over 150 ms late (the seat: 97-327), none over 300 ms (the seat: up to 43 per 1,000), longest gap 288 ms (the seat: 581 ms). **So the live daemon is not slower than the bench says; the seat machine runs the same heavy ticks several times slower than the devpod.** Not known: why (the daemon sharing the seat with the gui, WSL, or the CPU). What follows for the story: the tick after the dig has to get cheaper by a large factor for the seat, or the seat's share has to be measured there (a daemon-side timer in its log).
- **Task 13 DONE, commits `899136e5`, `37de6c7b`, `dc6fc77d`** (Opus 5.5 agent in its own worktree, cherry-picked; fast gate green on each in the worktree). `13-8b.sh` rows 13-24, one per decision of Tasks 10 and 11: head-on wait back (13), aside onto a free pile cell (14), exit onto one (15), failed component flooded again (16), `failed.clear()` removed (17), skip ignores the goals (18), ring never stepped (19), ring takes a link not in `Walk` (20), an idle link (21), a link whose step is not legal (22), links step but not the holder (23), holder steps but not the links (24). Rows 18, 21 and 22 survived the whole sim-core suite first, so each got its test: `a_failed_re_plan_leaves_a_reachable_dig_in_its_component_to_the_next_miner`, `a_chain_through_an_idle_dwarf_with_an_exit_moves_nobody`, `a_chain_with_a_step_that_is_not_a_legal_move_moves_nobody`. Repairs: `12-9.sh` row 4 re-pointed at `a_higher_id_miner_yields_to_an_idle_dwarf_at_the_dead_end_when_only_it_has_an_escape` (KILLED `lib.rs:8373`; the blocker half alone kills it, the holder half alone is caught only by scenario tests, noted in the row); `12-10b.sh` row 5c compiles again (KILLED `project.rs:4232`); **`13-7.sh` row 6 SURVIVED** (not known before: `claim_jobs`' kept failed components, AC 2, skip miner 0 before the sabotaged arm; which commit made it inert is not established), repaired with a third edit, KILLED `lib.rs:6566`. Re-anchored rows re-run: `13-7.sh` row 4 KILLED `lib.rs:6872`, `13-8b.sh` row 10 KILLED `lib.rs:6277`. Whole tables, each alone, RUST_TEST_THREADS=1: `13-8b.sh` 24/24, `13-4.sh` 11/11, `12-9.sh` 32/32, `12-10b.sh` 22/22, `13-7.sh` 10/10 KILLED. Audit: 1018 rows, every literal matches.
- **Task 16 DONE, commits `cde6e93f` (timer, serve test, row 25, 2.3 row re-pointed) and `1d8018ec` (baseline)** (Opus 5.5 agent in its own worktree, cherry-picked; fast gate green on each in the worktree, `cargo test -p simd` green including serve). **AC 18:** `TickTimer` in `crates/simd/src/main.rs` writes, every 1,000 stepped ticks, `tick-timer ticks=19000-20000 wall_s=22.703 step_ms_mean=22.2705 step_ms_max=173.7179 send_ms_mean=0.4210 send_ms_max=1.2014 over=951` (`send` = delta encode + broadcast; the socket write is on each client's own thread and is not in it). Test `the_daemon_logs_its_tick_times_every_thousand_stepped_ticks` (real binary, Fast4x): RED `no tick-timer line within 30 s`, GREEN in 5.6 s. Row 25 (step timer measures nothing) KILLED at the "the step took no time" assert (`serve.rs:3444`). The 2.3 row "pause no longer skips world.step" anchored on the old block; re-pointed, still KILLED by `paused_daemon_freezes_tick_and_entities_then_resumes`. Audit: 1019 rows, every literal matches. **AC 19, baseline** (`13-8b-signoff/tick-timer-baseline-run{1,2}.txt`; release live daemon, seat-like copied to Fast4x, the tap the only client, load 0.86 and 0.94 before; both `state=250b5bbeee340ff9`): after the dig the step is about 98% of the tick's work. Step mean 20.6-29.8 ms from t19,000 to t27,000, then 18.4, 16.5, 10.8; max 131-196 ms. Send mean 0.42-0.63 ms, max under 5 ms. The two runs agree within 2%. The timer matches the bench (whole-run step mean 10.46 ms against the bench's 10.1; `wall_s` = step + send within 0.1%). **So the cut goes in the sim, not the wire**: AC 20's encoding branch does not apply. `over` reads 1000 at Fast4x after the dig (every 5 ms period overruns), so it only tells something at Normal, on the seat. The turn is at t18,000-19,000 (25.8 ms mean, max 201-205 ms, the run's highest), just before AC 20's window.
- **Task 17 DONE, commits `ccac9e21` (pile cells once per stone map), `bc3c3d3b` (goal pocket), `9efc8185` (four older rows re-pointed), `b351ece5` (AC 20 runs)** (Opus 5.5 agent in its own worktree, cherry-picked; fast gate green on each in the worktree). Profile first (`prototypes/task17-split-646fa2c0.txt`, `task17-pocket-sizes-646fa2c0.txt`, probe `task17-split-probe-on-646fa2c0.patch`): after the dig most of the step was searches that fail because their goals are shut in a small pocket (blocked re-routes 5-12 ms a tick, claim searches 2-7 ms), each flooding the start's whole side to find that out, and `pile_targets` re-run on every haul turn (about 3 ms). **Two neutral cuts, no behaviour change:** (1) `execute_jobs` keeps the pile's free cells with the stone map and drops them with it, a debug assert checks them against a fresh `pile_targets` every turn; (2) `astar_with_budget` first floods the goals' side up to `POCKET_CELLS` (64) and answers `None` when the start cannot step into it. That is sound because every move can be walked back; a debug assert runs the full search on every such answer. **Test change:** six tests that pin kept failures (AC 2, 3, Task 11) dug into a one-cell pocket, which the new answer spares, so each now opens a room of 81 cells behind the pocket (`open_a_room_behind_the_pocket`) and the flood they pin still happens. `state=` and end lines identical on the 12 committed scenarios and seat-like (`13-8b-signoff/state-head-67be6aef.txt` against `state-head-a28a5523.txt`; seat-like `state=250b5bbeee340ff9`, `done=dig:1486`, `stored=538`), so AC 16 holds with nothing to rule. **AC 20 met** (`tick-timer-after-run{1,2}.txt`, same runner as AC 19, load 1.0-1.1): step plus send mean per window t19,000-30,000 is 25-42% of the baseline's (4.5-11.2 ms against 11.0-30.3); worst max 83 ms against 196.
- **Task 18 DONE.** `13-8b.sh` rows 26-30: kept pile cells survive a stone change (26), every haul turn finds them again (27), the pocket never asked (28), the pocket test asks where the start stands instead of where he can step (29), a start on its own non-walkable goal skipped (30). All KILLED: 26 in the agent's run; 27-30 re-run by the orchestrator on `b2c64a3c` (= `b351ece5`'s tree), RUST_TEST_THREADS=1. 27 and 28 die at the tests' asserts (`lib.rs:6479`, `:6409`); 29 and 30 at the pocket's debug assert (`lib.rs:1385-1386`), the oracle, like rows 4-9. **The pocket made four older rows inert:** `12-3.sh` "component skip not counted as attempted" and "first failed component covers every dwarf", `13-7.sh` rows 1 and 2 put their unreachable digs in one-cell pockets and SURVIVED on `bc3c3d3b` (each KILLED on `ccac9e21`); each row now also turns the pocket off, 4/4 KILLED. **Three SURVIVED rows the record did not know about**, dead on `main` `5f83e82e` as well as on `646fa2c0`, so older than this story: `12-3` "component skip removed: every dwarf searches", `12-11` row 14 "a kept landing he can no longer reach is kept anyway (seat)", `3-2-the-dig` "terrain mutation leaves other dwarves' cached paths stale". (`12-1` "retry drop stacks a full stockpile" survives too and is already deferred.) Not fixed here: for Wolf.
- **Task 9:** Q1 follow-up filed as **#236** (`bug`, `route:story`); Q3 is #235. The full gate is owed at PR and has not been run.
- **Seat run 4 (2026-11-17), Wolf:** "files are in folder but was busy with other things so forgot to watch". No eyes on it, no no-gui run. The tap (`13-8b-signoff/13-8b-wire-tap-seat-run4.csv`): the seat's daemon keeps real time through the whole run, 100.1-100.2 s per 1,000 ticks after the big dig (Run 3: up to 128.1 s), 0-32 late ticks per 1,000 (Run 3: 250-327), longest gap 292 ms (Run 3: 581). The daemon's timer (`13-8b-simd-stderr-seat-run4.txt`): after the dig step mean 3.2-16.6 ms, worst 144 ms, 0-4 ticks per 1,000 over the 100 ms tick. Gui frame times unchanged from Run 2. AC 21 is half met: the files are in, Wolf's ruling on what he sees is still owed. Card: Sitting, Run 4.

### File List

- `crates/sim-core/src/lib.rs` (modified)
- `_bmad-output/implementation-artifacts/mutations/13-8b.sh` (new)
- `_bmad-output/implementation-artifacts/13-8b-signoff/` (new): `vehicle-card.md`, `state-main-5f83e82e.txt`, `state-head-89be48c7.txt`, `split-30k-main-run1.txt`, `split-30k-main-run2.txt`, `split-30k-head-run1.txt`, `split-30k-head-run2.txt`, `split-30k-head-keep-disabled.txt`, `tick-bench-main-89be48c7.txt`, `prototypes/split-probe-on-89be48c7.patch`
- `scripts/bench/wire_tap.py` (new, 2026-11-14)
- `_bmad-output/implementation-artifacts/13-8b-signoff/`: `13-8b-seat-like-first-hour.csv`, `wire-tap-devpod-6b6a041f.csv` (new, 2026-11-14)
- `_bmad-output/implementation-artifacts/13-8b-signoff/`: `13-8b-seat-like-run2.csv`, `13-8b-wire-tap-seat.csv`, `prototypes/stall-probe.patch`, `prototypes/stall-probe-30k.txt` (new, 2026-11-15)
- `_bmad-output/implementation-artifacts/13-8b-signoff/prototypes/`: `why-probe.patch`, `why-probe-30k.txt`, `why-probe-swap-now.patch`, `why-probe-swap-now-30k.txt`, `swap-keep-paths-probe.patch`, `swap-keep-paths-30k.txt`, `swap-now-slow-ticks-30k.txt` (new, 2026-11-15)
- `crates/sim-core/tests/save_load.rs`, `crates/sim-core/tests/scenario.rs`, `_bmad-output/implementation-artifacts/mutations/13-4.sh` (modified, `55cc430b`)
- `13-8b-signoff/prototypes/`: `ring-keep-paths-probe.patch`, `ring-keep-paths-30k.txt`, `ring-only-30k.txt`, `keep-paths-jam-stuck-30k.txt`, `task10-55cc430b-probes-30k.txt`, `task10-plus-ring-probe.patch`, `task10-plus-ring-30k.txt` (new, 2026-11-15)
- `_bmad-output/implementation-artifacts/mutations/13-7.sh`, `13-8b.sh` (re-anchored, `5969d7fc`)
- `13-8b-signoff/`: `stall-probe-a28a5523.txt`, `state-head-a28a5523.txt` (new, 2026-11-15)
- `_bmad-output/implementation-artifacts/mutations/13-8b.sh`, `12-9.sh`, `12-10b.sh`, `13-7.sh` (modified, Task 13); `crates/sim-core/src/lib.rs` (three tests, `899136e5`)
- `13-8b-signoff/`: `blocking-a28a5523.txt`, `big-pile-over-time.txt`, `prototypes/blocking-probe-on-a28a5523.patch` (new, 2026-11-15)
- `13-8b-signoff/tick-bench-main-4048f3f1.txt` (new, 2026-11-15)
- `13-8b-signoff/`: `13-8b-wire-tap-seat-run3.csv`, `prototypes/wait-cause-probe.patch`, `prototypes/wait-cause-30k.txt` (new, 2026-11-15)
- `13-8b-signoff/wire-tap-devpod-a28a5523.csv` (new, 2026-11-15)
- `crates/simd/src/main.rs`, `crates/simd/tests/serve.rs`, `mutations/13-8b.sh` row 25, `mutations/2-3-master-of-time.sh` (re-pointed) (`cde6e93f`); `13-8b-signoff/tick-timer-baseline-run1.txt`, `-run2.txt` (new, 2026-11-16)
- `crates/sim-core/src/lib.rs`, `mutations/13-8b.sh` rows 26-30, `mutations/12-3.sh`, `mutations/13-7.sh` (re-pointed) (Task 17); `13-8b-signoff/`: `tick-timer-after-run1.txt`, `-run2.txt`, `state-head-67be6aef.txt`, `prototypes/task17-split-646fa2c0.txt`, `task17-pocket-sizes-646fa2c0.txt`, `task17-split-probe-on-646fa2c0.patch`; `vehicle-card.md` Run 4 (new, 2026-11-16)
- `_bmad-output/implementation-artifacts/13-8b-after-the-big-dig.md`, `sprint-status.yaml` (record)
- `13-8b-signoff/`: `13-8b-simd-stderr-seat-run4.txt`, `13-8b-seat-like-run4.csv`, `13-8b-wire-tap-seat-run4.csv` (new, 2026-11-17)

## Change Log

| Date | Change |
|---|---|
| 2026-11-12 | Created (claude-fable-5-2) on `story-13-8b-after-the-big-dig` off `main` `5f83e82e`. Four creation probes on seat-like; Task 0 Q1–Q3 await Wolf. |
| 2026-11-13 | Task 0 ruled (Wolf): Q1 (a) three neutral cuts, blocked-step and re-plan burst to a follow-up issue at Task 9; Q2 slot 13.8b accepted, epic count 17; Q3 haul failures filed as #235. Implementer: Sonnet 5.5 subagents, claude-fable-5-2 orchestrates. |
| 2026-11-13 | Dev, Tasks 1-7 (Sonnet 5.5 agent wrote Tasks 2-4, `f3fa3f8b` `d253f491` `89be48c7`; claude-fable-5-2 measured and ran the mutation table). `state=` identical on all 13 files; seat-like mean 20.3 → 11.8 ms; 12/12 mutation rows KILLED; tick-bench no regression. Q1 follow-up filed as #236. Open: the seat (Task 8, AC 11) and the full gate at PR. |
| 2026-11-14 | Seat run 1 recorded (Wolf: "less but still visible"; story not done). Seat log read against 13.8's run 3; devpod wire tap added (`scripts/bench/wire_tap.py`): the daemon does not stop and start. Card gains Run 2 (the tap on the seat). |
| 2026-11-15 | Seat run 2 recorded (Wolf: "it's better but ... dwarwes started to stop briefly (not all at once) ... it does not look smooth enough still"). Seat tap: daemon even at minute 21, ticks up to 0.97 s after the big dig. Headless stall probe: job-walking dwarves stand still inside the sim, most for one squeeze wait. **Wolf ruled the story continues past its perf scope: "I think we will continue until we are happy".** New ACs follow the attribution probe. |
| 2026-11-15 | **Amended on Wolf's rulings** ("continue until we are happy"; "include 236 also to this fix"): ACs 12-17 and Tasks 10-15 added, AC 1 limited to Tasks 1-7. Probes: head-on job-holders are the stops Wolf saw (not #227's crowd); a first-look swap cuts stalls 266 → 8 per 1,000 ticks at t12,000; the slowest ticks are the re-plan burst; a keep-paths prototype jams. Implementer for Tasks 10-13: Opus 5.5 subagents (Wolf, 2026-11-15), claude-fable-5-2 orchestrating. |
| 2026-11-15 | Task 10 code `55cc430b` (Opus 5.5 agent): head-on swap on the first look, pile-cell step-aside fix, cage sweep re-pinned, #238 filed. Measured: stops before the dig fixed (at most 36 per 1,000 ticks), after the dig NOT better in standing time until a ring rule is added (prototype). #239 filed (miner stuck holding a stone). Task 11 re-scoped on the probes: flood a failed re-plan once, then the ring rule. |
| 2026-11-15 | Task 11 code `5969d7fc` (a failed re-plan's component flooded once, neutral) and `a28a5523` (a ring of waiting job-holders steps at once). Measured: worst tick 188 ms (was 369-446), stops after the dig 103-557 per 1,000 ticks (was 1,127-1,641), digs 1,486. Open: AC 13's pre-dig window (36 against 30), big-pile does two fewer digs (AC 16), mutation rows (Task 13), seat run 3 (card written). |
| 2026-11-15 | Task 13 (`899136e5`, `37de6c7b`, `dc6fc77d`): 12 mutation rows for the swap, the failed re-plan list and the ring step, three new tests for sabotages nothing killed; `12-9.sh` row 4, `12-10b.sh` row 5c and `13-7.sh` row 6 (found SURVIVED in this run) repaired. Five tables all KILLED. |
| 2026-11-15 | AC 15 measured: blocking is not cheaper (0.5-15.1 ms a tick, 76-1,945 no-way-round waits per window after the dig). AC 16: big-pile's two fewer digs traced: the old build freezes at 35 of 46, the new one finishes all 46 by t8,000. |
| 2026-11-15 | Bench against `main`: nothing slower, two-hundred-on-256 -24.1%. Tasks 12 and 15 ticked; #236 and #219 commented. Open: Wolf's rulings on AC 13 and AC 16, seat run 3 (Task 14), the full gate. |
| 2026-11-15 | Seat run 3: Wolf sees the same stops after the big designation. The story continues on the blocking after the dig. |
| 2026-11-15 | Seat run 3's tap: the seat's daemon runs up to 28% behind real time after the big dig and recovers by t30,000. Wait-cause probe added. Open: where the seat's time goes, and the rule gaps the probe names. |
| 2026-11-15 | Live devpod tap: the devpod keeps real time after the dig (at most 101.1 s per 1,000 ticks), the seat does not (up to 128 s). The gap is the seat machine, cause not known. |
| 2026-11-16 | **Amended on Wolf's rulings** ("Both, timer first"; implementer Opus 5.5 agents): ACs 18-21 and Tasks 16-19 cover a per-tick timer in the daemon's log, a baseline, a 2x cheaper tick after the dig, and seat run 4. |
| 2026-11-16 | Task 16 (`cde6e93f`, `1d8018ec`): the daemon logs its tick times. Baseline: after the dig the step is about 98% of the work (20-30 ms mean), encoding 0.4-0.6 ms, so the cut is in the sim. |
| 2026-11-16 | Task 17 (`ccac9e21`, `bc3c3d3b`, `9efc8185`, `b351ece5`): two neutral cuts (pile cells kept with the stone map; a search for goals in a small pocket answered without the flood). `state=` unchanged everywhere; the tick after the dig at 25-42% of the baseline (AC 20). Task 18: rows 26-30 KILLED, four older rows re-pointed, three pre-existing survivors found. Run 4 card written. |
| 2026-11-17 | Seat run 4's files (Wolf did not watch): the seat's daemon now keeps real time after the big dig (100.2 s per 1,000 ticks, was up to 128 s). Wolf's ruling on the look is still owed. Full gate green at `e6eb06d5`. |
