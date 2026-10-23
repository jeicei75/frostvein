---
baseline_commit: 4f57027
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
---

# Story 13.2: The Way Through — Pathfinding and Reachability Spike

Status: in-progress

## Story

As the boss,
I want the movement layer checked against how colony sims solve reachability and crowding,
so that M4 builds on a movement layer that is fixed as a whole, not one cage shape at a time.

## Not stacked: branch off `main`

`main` is `4f57027` (PR #215, 13.1 merged). Branch `story-13-2-the-way-through-pathfinding-and-reachability-spike` carries `db8fde3` (13.1 done-mark) and this file. This is action item M3-1 (#209), owner Winston. It measures with 13.1's bench. Epic 13's standing ACs bind it: **one full gate per story, at PR** (M3-4, #212).

**This is a spike. No sim change lands.** Every prototype is a throwaway patch committed under `13-2-signoff/prototypes/`, applied in a scratch worktree, and never merged. **Wire diff: none.**

## Found at creation (2026-10-23, on `db8fde3`, which has `main`'s code)

- **No M4 PRD draft exists.** `planning-artifacts/prds/` holds only the 08-01, 08-09 and 09-28 PRDs. The epic's "the scale target is the M4 PRD draft's numbers" has nothing to read yet (Task 0 Q1). Today's scale is hardcoded: `Dims::DEFAULT` 128×128×32 (`lib.rs:137`) and exactly 5 dwarves (`spawn_dwarves` takes `[Identity; 5]`, `lib.rs:3451`). A scenario's `start` takes only a `seed` or a `save`, not dims or a crew size.
- **The retro's code facts hold.** `astar_heuristic` (`lib.rs:1124`) takes the min over every goal. It runs at the start, on each push (`:1175`) and on each pop's staleness check (`:1151`). `came_from` and `costs` are `BTreeMap`s (`:1146-1147`), and every step costs 1. `MAX_ASTAR_NODES = 50_000` (`:46`). Running out of budget reads as "no path" through `astar` (`:1185`). Nothing is cached across ticks (`:569`) except `Terrain::pile_ground` (`:707`).
- **No profiler on this devpod.** `perf`, `samply` and `cargo-flamegraph` are absent. Deterministic counters (A* calls, nodes expanded, heuristic goal-checks, flood cells) are the profile's currency. They are identical across runs and across builds, which matters because cross-build timing noise is about 6% here (13.1 review-patch run 2).
- **Reproducer census, baseline on this HEAD.** These are bench `end` lines (`crates/simd/src/scenario.rs:334`), release, 1 run each, and they are deterministic:

  | issue | reproducer | HEAD reading |
  |---|---|---|
  | #159 | `lib.rs:5322` `a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on`, `:5234` `a_reachable_job_behind_…_sum_past_the_budget`. The open half (more than 20 unreachable jobs ahead) has no test | today's worlds can't trigger it (DEFAULT_SEED's area is about 16k, under the 50k cap; per the issue) |
  | #186 | cage sweep `no_dwarf_is_caged_by_stones_over_the_probe_seeds`, `ONE_LEVEL_ISLANDS` (`tests/scenario.rs:2988`) | 4 frozen runs pinned |
  | #187 | the same sweep, `UNFINISHED` (`:3006`) | `[(11, 6, 1)]` |
  | #189 | `a_stockpile_around_the_campfire_never_zones_or_receives_the_fire` (`tests/scenario.rs:35`) | pile stops at 23/24 (per the issue; not re-run) |
  | #203 | `~/frostvein-saves/12-11-dig-loop.save`, **refused as-is** (`missing field clearing`, then `goals_completed`). A scratch copy with `"clearing": null` on each dwarf and top-level `"goals_completed": 0` loads | **marks=11, frozen** from t ≤ 17640 to t27640 (1k/10k/20k-tick runs read 14/11/11) |
  | #204 | `scenarios/seat-slow.json`, `scenarios/full-pile-crowd.json` (and the #203 save's later freeze) | seat-slow `marks=1 goals=38/12/1`; full-pile-crowd `marks=0 items=46` |
  | #206 | `~/frostvein-saves/12-12-seat-loop.save` (t18082; the issue names no file, so this match rests on its name, date and tick) loads as-is | **marks=3, frozen** from t19082 to t38082 |
  | #207 | the `assert_ring_fills` fixture (`tests/scenario.rs:5198`), pinned by the lib test `a_hole_outside_each_piles_own_box_is_still_a_pocket` | 14/24 stored (per the issue) |

- **Ghost-dwarves probe (this was the RED for the census, and it carries a trap).** A throwaway patch made `dwarf_tiles` (`lib.rs:1200`) return an empty set, so no dwarf ever blocks another. Results, HEAD → ghost:

  | reproducer | marks | mean ms |
  |---|---|---|
  | seat-loop (#206), 10k ticks | **3 → 0** | 0.097 → 0.080 |
  | dig-loop (#203), 10k | 11 → **20** | 0.035 → **0.904** |
  | plan | 0 → **4** (goals 12/10/1 → 12/8/1) | 0.190 → 0.426 |
  | full-pile-crowd | 0 → **37** | 0.031 → 1.055 |
  | deep-pit | 0 → 0 | 0.038 → 0.022 |
  | seat-slow | 1 → 1 (goals 12 → 11 stone) | 0.475 → 0.835 |

  It does two things. It shows the census discriminates, and it is evidence that #206 is a dwarf-blocks-dwarf freeze, which is the squeeze-past family. **And deleting a rule is not an upper bound.** Occupancy feeds the wall-in guards and the job logic as well as movement, so ghosting made 3 of 6 reproducers worse. A prototype has to change movement only.

## Task 0: RULED (Wolf, 2026-10-23: "1 5, 20, 50, 100, 200 2 a 3 yes")

1. **Scale: crews of 5, 20, 50, 100 and 200 dwarves, on today's 128×128×32 map AND on a bigger 256×256×32 map** (Wolf, follow-up: "we should test also bigger map"). That is ten points. The spike measures the curve, the proposal says where each option breaks, and the PRD picks a point later. Worldgen takes any `Dims`; its only 128-literals are in tests (`worldgen.rs:457-519`).
2. **One story (a).** Prototypes are crude by design, and a prototype that can't be made to work is recorded with what blocked it, which is a result. If dev overruns, split at Part B (Tasks 6–7 plus their rows) into 13.3, and 13.2 ships a partial proposal.
3. **Commit both issue saves.** `12-12-seat-loop.save` as it is, and the #203 save as `12-11-dig-loop-patched.save`, with its two added fields (`"clearing": null` on each dwarf, top-level `"goals_completed": 0`) named in its scenario's `about`.

## Acceptance Criteria

1. `13-2-signoff/proposal.md` § Survey marks each retro §4.1 lead as VERIFIED, CORRECTED or STRUCK, with a source link and one line on what it means here. The leads are: DF whole-map A\*, DF's per-tile walkable-group id, Factorio FFF #317's heuristic replacement, Going Medieval's items slowing travel to 70%, hierarchical A\*, and cooperative pathfinding.
2. § Profile gives, for every committed scenario and both issue saves, A\* calls, nodes expanded and heuristic goal-checks per caller, and cells visited per flood, beside the tick mean. It names each scenario's top cost. Two runs of one build give identical counts.
3. Every prototype has a row reproducible from a committed patch that applies to `4f57027`. The row covers tick-bench against base (with the control column), every reproducer's `end` line, and the cage-sweep lists (frozen, unfinished, landed, and the two staircase sweeps' worked counts). The prototypes are a connectivity map, squeeze-past and clamber-over, plus any A\* cost fix the profile justifies.
4. § Issues gives each of #159 #186 #187 #189 #203 #204 #206 #207 one cell per option (fixed, unchanged, worse, or not movement) with the number behind it. A "not movement" cell says what kind of issue it is instead.
5. In every prototype, no two dwarves stand on one tile at any tick boundary. Any departure from today's rules (a swap, a step over an item, a changed wall-in guard) is listed under § Rulings needed with what it buys.
6. § Scale gives the tick mean and nodes expanded for base and each option at crews of 5, 20, 50, 100 and 200 dwarves on both 128×128×32 and 256×256×32 (Task 0 Q1).
7. § Recommendation names the options, their order and the stories they would become, with expected cost taken from the proposal's own numbers. Wolf's ruling is recorded in this story before Status moves to review, and `git diff 4f57027 -- crates/` is empty.

## Tasks / Subtasks

- [x] **Task 0: Wolf rules Q1–Q3.** Ruled 2026-10-23: Q1 crews 5/20/50/100/200 on 128² and 256², Q2 one story, Q3 commit both saves.
- [ ] **Task 1: the census instrument (AC 3, 4).** NEW `13-2-signoff/census.sh PATCH...`:
  - It makes a scratch worktree at `4f57027` and applies the patches in order. It builds a release `simd` there; `cd` into the worktree, because `CARGO_TARGET_DIR` builds the cwd's sources.
  - It prints every reproducer's `end` line (the 8 `scenarios/*.json` and the two issue saves at 10,000 ticks) and the cage-sweep lists.
  - `census.patch` turns the cage sweep's three `assert_eq!`/`assert!` on `landed`/`frozen`/`unfinished` (`tests/scenario.rs:3109-3125`) and the staircase `worked` asserts into `eprintln!`. Today the first failing assert hides the rest, and a prototype is expected to change the pins.
  - Run the sweeps with `cargo test --release -p sim-core --test scenario <name> -- --nocapture`.
  - Add (Q3) `scenarios/stuck-tunnel.json` (#206) and `scenarios/dig-loop.json` (#203), each starting from its save with 10,000 ticks and no commands, and say in `about` which issue the file reproduces.
  - **RED, then control.** `census.sh` with no prototype must reproduce the creation table above. `census.sh ghost.patch` (commit the probe as `prototypes/ghost.patch`) must read seat-loop `marks=0` and plan `marks=4`. A census that cannot tell these two apart is broken.
- [ ] **Task 2: the survey (AC 1).** Web sources only, with each claim linked. One table plus at most a paragraph per lead. Check DFHack's walkable-group id in its source or docs, not from memory.
- [ ] **Task 3: the profile (AC 2).**
  - NEW `prototypes/counters.patch` adds plain counters (a resource or statics, read at the bench's end line or printed by a test). It counts A\* calls and nodes per caller (`claim_jobs` `:633`/`:646`, `execute_jobs` replan `:2113`, `resolve_blocked_step` `:1751`, `clearing_set_down` `:1927`), `route_to_nearest` calls and cells, heuristic goal-checks, and cells visited by `stranded` (`:1015`), `caging_stone` (`:1832`), `nearest_free_cell` (`:1436`, which has no cap) and the `pile_targets` floods (`:851`).
  - Counters only, with no behaviour change: every reproducer's state hash must equal base.
  - RED: on top of counters, make `astar_heuristic` iterate its goals twice (13.1's sabotage). Plan's goal-check count must double, and nodes must not move.
  - Use tick-bench for the ms column (`RUNS=5`, never beside the gate or `mutate.sh`). Judge between builds by counts. A ms delta under about 6% between two SHAs is weak evidence.
- [ ] **Task 4: scale (AC 6).** Crews of 5/20/50/100/200 on 128×128×32 and 256×256×32 (Q1). NEW `prototypes/scale.patch` makes the crew size and the dims scenario-settable (throwaway). **Trap: a bigger map costs only if the work uses it.** Put the 256² scenarios' marks and piles far enough apart that paths and floods span the map, or the extra area is idle ground and reads as free. **On 256², #159 may become live:** DEFAULT_SEED's walkable area is about 16k cells at 128², so 4× the area can pass the 50k `MAX_ASTAR_NODES` cap. Measure that component size, and fill #159's cell from it. **Trap: spawn runs out of cells.** `spawn_dwarves` (`lib.rs:3447`) draws from the standable cells within `CAMP_RADIUS = 3` of the camp (`worldgen.rs:7`), which is at most 49 cells, and `random_range(0..0)` panics once they are used up. So 50+ dwarves need a wider spawn ring. The identity and profession arrays are fixed at 5 too. Scenario files go in `13-2-signoff/scale/`, not `scenarios/`. Each scale point needs real work for the crew (marks plus a pile), or it measures idle dwarves. Run base plus each option patch on top of it.
- [ ] **Task 5: connectivity map (AC 3, 4).** `prototypes/connectivity.patch`. It keeps reachability between ticks and answers "can he get there" before any A\* or flood, so #159's starvation and the per-question floods go.
  - **Trap: items move on every haul.** If the map includes items, it is invalidated constantly. If it covers terrain only, A\* still decides around items. Measure the invalidation rate and say which way it went and why.
  - Invalidate on the same inputs as `pile_ground` (`set_tile` at `lib.rs:742`, a zones change at `:868`).
  - Also measure whether the map lets the four wall-in guards (`walls_in_a_dwarf` `:959`, `walls_in_a_dwarf_from` `:1002`, `closes_last_level_exit` `:974`, `caging_stone`) become one question ("does every dwarf still reach work?"). A guard that answers a different question once it is cheaper is a different rule: compare outcomes as well as ms.
- [ ] **Task 6: squeeze-past (AC 3, 4, 5).** `prototypes/squeeze.patch`. A timed swap of two dwarves in a one-wide way, where the gui would sell it with an animation (out of scope here).
  - `resolve_blocked_step` never swaps today (`lib.rs:1685`). `step_and_watch` (`tests/scenario.rs:4746`) panics on a swap, and changing that check IS the rule change: list it under Rulings needed.
  - List every writer of a dwarf's `Pos` (`execute_jobs`, `settle`, `wander`; `dwarf_tiles` doc, `lib.rs:1195`) and keep each one's `occupied` current. Then check that no tile holds two dwarves at a tick boundary. `step_and_watch` covers only the one-wide staircase, so `counters.patch` also counts tick boundaries where two dwarves share a tile, on every reproducer. It must read 0 on base and on every prototype (AC 5's emitter).
  - The ghost probe is not this prototype's ceiling; see Found at creation.
- [ ] **Task 7: clamber-over (AC 3, 4, 5).** `prototypes/clamber.patch`. A costed step over a blocking stone.
  - A\* is unit-cost today, so this needs a weighted step, and the heuristic must stay admissible. Emitters share `blocked` with stones (`blocked_cells` `:795`, `world_blocked` `:800`) and must stay blocking.
  - The item-placing writers (haul drop `:2258`, dig spawn `:2337`, channel landing `:2395`, `settle` `:2493`) must still never place a stone on a dwarf. The sweep's `landed` list must stay empty.
- [ ] **Task 8: the proposal (AC 1–7).** NEW `13-2-signoff/proposal.md` with § Survey, Profile, Scale, Prototypes, Issues, Rulings needed and Recommendation. Every number names its patch stack and command. State ADR 5 ("plain A\* for M1", `docs/technical-preferences.md`) and what the recommendation does to it.
- [ ] **Task 9: Wolf rules, then close.**
  - Record the ruling in the Change Log and in the proposal's last section. Note any issue it re-scopes on that issue, with no closing keyword; nothing here closes an issue.
  - Then the ONE full gate (`RUST_TEST_THREADS=1 scripts/gate.sh`) and paste the block. Then the cost row (`session_tokens.py --phase dev --story 13-2-the-way-through-pathfinding-and-reachability-spike`) and `phase_state.py clear`.

## Dev Notes

### Scope guardrails (do NOT)

- **Do not** land any change under `crates/`. Prototypes live as patches; AC 7 checks this.
- **Do not** change the strict rules in any shipped code, or assume a ruling. Wolf ruled 2026-10-21 that the rules stay. Present each option; don't pick one silently.
- **Do not** write movement stories or edit the M4 PRD. Wolf's ruling on the proposal comes first.
- **Do not** add a dependency or install a profiler. Counters are the profile.
- **Do not** fix #214 (off-map rect) or #199 (what stops a digger). Neither is movement.

### What already exists (build on it)

- 13.1's `simd --bench FILE --runs N` (end line `scenario.rs:334`: tick, marks, items, zones, goals) and `scripts/tick-bench.sh BASE HEAD [FILES]` (with its shuffled-layout control column). The README's Tick cost section documents both.
- The cage sweeps in `crates/sim-core/tests/scenario.rs`: the stones sweep (`:2947`), the two-wide staircase (`:4673`), the one-wide staircase with `step_and_watch` (`:4779`), and head-on in a one-wide staircase (`:4849`).
- The movement code, all in `lib.rs` (about lines 538–2648). Systems are chained at `:2701-2713` (`claim_jobs`, `execute_jobs`, `settle`, `wander`, `carry_items`). `resolve_blocked_step`'s escape ladder is at `:1687`.
- The `pile_ground` cache (`lib.rs:707`, cleared at `:742`/`:868`). It is the one invalidate-on-change cache already in the sim, so model the connectivity map on it.

### Key decisions & traps

- **Every cost A/B also compares a behaviour outcome** (marks, items, goals, the sweep lists). A cheaper rule is a different rule (12.12's per-pile boxes resealed the fire ring).
- **Read #186 before blaming movement.** The issue records `wander` stepping on one level only (stale NOTE), and a probe drawing wander's moves from `astar_neighbours` cut that sweep from 5/32 to 1/32. A cell can say "not movement: wander" or "fixed by wander alone".
- **#187 fixes (a)/(b)/(c) were built and dropped** as no better than V2 (see the issue thread). Fix (c) moved p99 from 56 µs to about 1.3 ms. Don't re-run them as new options.
- **#207 and part of #189 may not be movement** (where a hauler is sent to stand). A "not movement" cell is a valid result.
- **Build hygiene:** `CARGO_BUILD_JOBS=8`, at most about 4 release builds at once (the devpod OOMs). Each prototype gets its own scratch worktree and target dir. Commits in a scratch worktree need `--no-verify`. Never `git checkout --` a probe over uncommitted work.

### Project Structure Notes

- NEW `_bmad-output/implementation-artifacts/13-2-signoff/`: `census.sh`, `proposal.md`, `prototypes/{census,ghost,counters,scale,connectivity,squeeze,clamber}.patch`, `scale/*.json`.
- NEW (Q3) `scenarios/stuck-tunnel.json`, `scenarios/dig-loop.json`, `scenarios/saves/12-12-seat-loop.save`, `scenarios/saves/12-11-dig-loop-patched.save`.
- UPDATE this story and `sprint-status.yaml`. Nothing under `crates/`.

### Previous story intelligence

- 13.1: the bench's same-binary floor is about 1–3%, but cross-build it is about 6% (`--shuffle-sections` control). That is why counts, not ms, decide between prototypes.
- 13.1: a committed save rots at the next save-format change. The #203 save already has, and needed two fields added by hand.
- 12.9/12.11: every cage fix closed one shape and the next appeared at the seat. The sweep lists are the only cross-shape view; read them whole.

### References

- `epic-12-retro-2026-10-21.md` §4.1, §8; `action-items.md` § M3-1; issue #209 and the eight cage issues.
- `epics.md` Epic 13, Story 13.2; `docs/technical-preferences.md` ADR 5.
- `13-1-scripted-runs-and-the-tick-bench.md` (Debug Log reference figures; Verification).

## Verification

Run from the repo root; release builds.

```bash
# 0. creation baseline (ran 2026-10-23): issue saves via scratch scenarios {"start":{"save":...},"ticks":10000,"commands":[]}
target/release/simd --bench <seat-loop>.json --runs 1   # observe: end tick=28082 marks=3 items=57 goals=38/4/0
target/release/simd --bench <dig-loop-patched>.json --runs 1   # observe: end tick=17640 marks=11 items=44

# 1. census RED, then control (Task 1)
_bmad-output/implementation-artifacts/13-2-signoff/census.sh                       # observe: the creation table above
_bmad-output/implementation-artifacts/13-2-signoff/census.sh prototypes/ghost.patch  # observe: seat-loop marks=0, plan marks=4, full-pile-crowd marks=37
# restore: none needed; every census run is a scratch worktree, removed at exit

# 2. counters RED (Task 3): counters.patch + heuristic-twice sabotage
#    observe: plan heuristic goal-checks exactly 2x the counters-only run; nodes expanded identical; state hash = base

# 3. per prototype: census.sh counters.patch <proto>.patch ; RUNS=5 scripts/tick-bench.sh 4f57027 <proto-sha>
#    observe: a full row in proposal § Prototypes / § Issues

# 4. AC 7
git diff 4f57027 -- crates/   # observe: empty
```

Exit 0 is not a result: read the lines.

## Dev Agent Record

### Agent Model Used

Wolf picked Sonnet 5.5 (`claude-sonnet-5-5`) subagents for the implementation (2026-10-23). Opus 5.5 orchestrates and verifies.

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-23 | Story created on `db8fde3` (Opus 5.5). Creation probe: both issue saves freeze on HEAD (#206 marks=3 and #203 marks=11, each over 10k+ ticks; #203's save needed `clearing` and `goals_completed` added). The ghost-dwarves RED moved #206 from 3 to 0 and made 3 of 6 reproducers worse. There is no M4 PRD draft and no profiler on the devpod. Task 0 Q1–Q3 await Wolf. |
| 2026-10-23 | Task 0 ruled by Wolf: Q1 crews of 5/20/50/100/200 on today's map, and (follow-up) on 256×256×32 too; Q2 one story (split at Part B on overrun); Q3 commit both issue saves. |
