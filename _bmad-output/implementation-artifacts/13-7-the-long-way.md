---
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
baseline_commit: 483b95b1
---

# Story 13.7: The Long Way

Status: in-progress

## Story

As the boss,
I want a dwarf on a 256² valley to reach a reachable job however large the walkable ground grows,
so that work far from the camp is never silently starved by the path-search budget (#159).

## Branch: off local `main`, which is ahead of `origin/main`

Branch `story-13-7-the-long-way` was cut from local `main` `483b95b1`. That commit carries the unpushed 13.6 done-mark, which rides into this PR, as `3319e68c` did for 13.4. Epic 13's standing ACs bind: M3's ACs 1–8, with **one full gate per story, at PR**, plus M4 standing ACs 10–12 and 18. ACs 13–17 do not apply: there are no needs, stock, trades or rendering here. **No wire change and no saved field.** The budget is derived from the terrain, so no state hash moves through the save encoding (memory: `state-hash-covers-save-encoding`). **No seat card:** this is sim work, judged by scenario tests (M2-26).

## Task 0 — Wolf rules before dev

Each question has a recommendation. The ACs and Tasks are written for it, and a different answer amends the ACs it names.

- **Q1. Which searches get the new budget?** Every 256² exhaustion is an unreachable goal (*Found at creation* B), and the other sites' `None` is the same answer either way (C).
  - (a) **Recommended: `claim_jobs` only. ← RULED 2026-11-02.** It is the only site whose outcome moves, and it is cost-neutral: mean −1%, p99 flat, max +18% (D). `astar()`, `route_to_nearest` and `stranded` keep `MAX_ASTAR_NODES` = 50,000, with a `// NOTE:` naming what 13.7 measured.
  - (b) Every site. The outcome is the same as (a), and it costs +11% mean and +20% p99 (D).
- **Q2. What shape does the claim budget take?** Searches are bounded per search, never per tick: the heuristic is consistent, so a search expands each cell at most once (A: an unreachable flood expanded exactly 64,300 = its component). A budget equal to the world's cell count therefore cannot run out, and digging cannot outgrow it.
  - (a) **Recommended (← RULED 2026-11-02):** every claim-time search gets a fresh budget of the world's cell count. 12.4's per-dwarf budget and its sat-out rule (fix B′) existed only for a budget that runs out, so they go. The 12.3 component cache stays: it is what makes an unreachable job cheap, one flood per component per call. The `(None, true, _)` arms become `unreachable!`. 12.4's rows that sabotage the removed code are retired (Task 4).
  - (b) Keep the per-dwarf, per-tick budget and sat-out, with the budget sized to the world. That is a smaller diff, but sat-out can then no longer happen in any test world, so its code is dead and untested and its rows cannot be killed.
- **Q3. The epic AC's premise is wrong. Amend it?** Epic 13.7 AC 1 says "a reachable job **beyond the budget**", and AC 2 offers "or improves the heuristic". Both rest on 13.2's reading that most 256² exhaustions were reachable goals farther than the budget. Measured, none is (B), and no reachable search from the camp needs more than 14,604 nodes (A). The live shape is #159's title: a reachable job queued behind more than `RETRY_COOLDOWN` unreachable ones. A heuristic cannot shorten a search that has no goal to find.
  - (a) **Recommended (← RULED 2026-11-02; epic amended, comment posted):** amend epic AC 1 to "a reachable job behind more than `RETRY_COOLDOWN` unreachable ones" and strike the heuristic option from AC 2. Then post `13-7-signoff/issue-159-comment.md` on #159.
  - (b) Keep the epic text and add a reachable-beyond-the-budget test anyway. Nothing on the valley produces that today, so it would need a hand-built maze.

## Found at creation (2026-11-02, `main` code `f148cfbe`)

Throwaway scratch worktree, release builds. Raw lines and patches are in `13-7-signoff/`: `creation-probes.txt` sections A–F, `prototypes/probe159-counters.patch`, `prototypes/probe-the-long-way.patch` and `prototypes/budget-all-sites.patch`.

- **A. The fresh 256² valley.** The camp component is 64,300 cells. The 1,954 other components are all tree-crown tops of at most 16 cells, and no stone or soil cell has every dig face off the camp. An unreachable flood expands exactly 64,300 nodes in 34–37 ms. Reachable single-goal searches from dwarf 0 (every 97th camp cell, 663 searches) peak at 14,604 nodes.
- **B. Every exhaustion is unreachable.** On `two-hundred-on-256.json` over 2,500 ticks: `claim_jobs` 238/238, `astar()` 915/915 (896 reroutes in `resolve_blocked_step`, 19 re-plans in `execute_jobs`), and `route_to_nearest` 27 (not classified).
- **C. Only the claim budget moves an outcome.** Raising the constant at every site leaves all 10 128² scenarios' `state=` equal to `main`'s. two-hundred-on-256 moves `3fc1461d…` → `6ba8edcc…`, and raising the claim budget alone gives the same state.
- **D. Timing** (two-hundred-on-256, `--runs 3` each, sequential, idle devpod):

  | variant | mean ms | p99 (median) | max | end line |
  |---|---|---|---|---|
  | `main` | 34.0 / 34.5 / 34.3 | 155.9 | 255–260 | `done=dig:231,cut:91 stored=33 cells=2165` |
  | claim budget = world | 33.4 / 33.8 / 34.0 | 155.5 | 302–312 | `done=dig:234,cut:90 stored=27 cells=2146` |
  | every site = world | 37.6 / 38.0 / 38.6 | 187.9 | 372–381 | same as claim |

- **E/F. The red reproduces on the real valley.** Setup: 256² DEFAULT_SEED crew 5, all miners; 25 one-cell sealed pockets at z = 1 (x = 20 + 4k, y = 20), each with a stone dig target at x + 1; one stone placed at dwarf 0 + (3, 0, 0). The 25 digs are designated, the world steps 100 ticks, and then the reachable dig is designated.
  - On `main` it is never claimed by t500. Jobs 20–24 are never re-attempted, and the tick averages 102 ms.
  - With the fix it is claimed at t115 and dug at t208, in 10.8 s of debug.
  - **Control:** if all 26 are designated at t0, `main` also claims at t15, because the reaction delays let the reachable job slip in before the unreachable ones cycle. The late designation is load-bearing.
  - Claim-only form at t100: `main` stamps one job per tick at about 133 ms a tick; the fix stamps all 25 in one 38 ms tick.

## Acceptance Criteria

1. **A reachable job behind unreachable ones is claimed and done on 256² (FR55, red first).** On the *Found at creation* E/F setup, the reachable dig designated after 100 ticks is claimed within 60 ticks of its designation and dug within 300. Under Task 4 row 1, which restores `main`'s budget, the claim assert fails.
2. **The claim budget is sized to the world (ADR 5 as amended; mechanism AC).** Every claim-time search runs with a budget of the world's cell count, so it completes in any world and as digging grows the component. There is no hierarchy and no heuristic change. This is mechanism on purpose: the ADR 5 scope line is the requirement.
3. **An unreachable job is answered in one flood and retried (M1 FR8).** A dwarf in one area over 50,000 cells, with 10 unreachable digs ahead of a reachable one: on the first claim tick all 10 are stamped `tick + RETRY_COOLDOWN` and the reachable dig is claimed. Twenty ticks later the 10 are attempted again and re-stamped, and they are still queued.
4. **The other searches keep their budget (Q1 a).** `astar()`, `route_to_nearest` and `stranded` still use `MAX_ASTAR_NODES` = 50,000, and every pin on it passes unchanged (`astar_stops_at_the_node_cap`, `astar_finds_a_path_well_inside_the_node_cap`). **AMENDED 2026-11-02 (review run 1, decision 1, Wolf "1" = option (a)):** the `execute_jobs` re-plan also gets the world's cell count, because a capped re-plan of a far claim looped claim/release forever under terrain churn. `astar()` itself, `resolve_blocked_step`, `route_to_nearest` and `stranded` keep `MAX_ASTAR_NODES`. Pinned by `a_far_claim_is_kept_through_terrain_churn_past_the_node_cap`.
5. **Bench delta (standing AC 10/11).** `RUNS=5 scripts/tick-bench.sh main HEAD` over all 11 scenarios reads `same` state on the 10 128² rows. two-hundred-on-256 reads `differ`, and the record states why: the claim searches complete instead of exhausting. The record also gives its mean and p99 delta against the floor; creation measured flat (D).
6. **The A\* storage profile is recorded (epic AC 4).** At HEAD on two-hundred-on-256, the record gives A\*'s share of the tick, and the mean and p99 of a throwaway flat-`Vec` storage A/B with its `state=`. No storage change is committed.
7. **The mutation table kills** (Task 4), and `python3 scripts/audit-mutations.py` reports no BROKEN row.

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings (Q1–Q3).** Record the answers in the Change Log and amend the ACs they name. Under Q3 (a), amend `epics.md` Story 13.7 ACs 1–2 with an `AMENDED 2026-11-02` line, and post `13-7-signoff/issue-159-comment.md` on #159 if Wolf says yes. Then ASK which implementer to use (memory: `dev-implementer-is-sonnet-not-codex`).
- [x] **Task 1: the claim budget (AC 2, 4).** UPDATE `crates/sim-core/src/lib.rs` `claim_jobs` (`:559-703`).
  - [x] Delete `budgets` (`:591-592`), `sat_out` (`:623-624`, `:644-647`, `:697`) and the `enumerate`/`slot` that only indexed the budgets. Before each `astar_with_budget` call (`:658`, `:671`), set `let mut budget = terrain.tiles.len();` and pass `&mut budget`. The stamp condition becomes `attempted && !assigned`.
  - [x] Both `(None, true, _)` arms (`:665`, `:683`) become one `(_, true, _) => unreachable!("a search sized to the world cannot run out")` per match. The `(Some(_), true, _)` arms fold into it.
  - [x] Rewrite the comments that are now false: `:559-561` ("spends one node budget per dwarf"), and the NOTE at `:595-598`. The new NOTE says: a search expands each cell at most once (consistent heuristic), so the world's cell count cannot run out; an unreachable job costs one flood of its component per call, cached in `components`.
  - [x] `MAX_ASTAR_NODES` NOTE (`:52-53`): it now bounds `astar()`, `route_to_nearest` and `stranded` only. 13.7 found every one of their 256² exhaustions to be an unreachable goal, so capping gives the same `None`, cheaper. A reachable reroute or re-plan past it would give up (`resolve_blocked_step` `:1887`, `execute_jobs` `:2263` → `retry_claim`), which no probe has seen. Do not change the value (AC 4).
- [x] **Task 2: the tests (AC 1, 3, 4).** In `crates/sim-core/src/lib.rs` `mod tests`. Port the setup from `13-7-signoff/prototypes/probe-the-long-way.patch` (`probe_the_long_way_stepped`), dropping its `println!`s.
  - [x] NEW `a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley` (AC 1). The *Found at creation* E/F setup: `World::generate_with_crew(DEFAULT_SEED, Dims::FRESH, 5)`, `stand_miners_at` (`:5412`) on the dwarves' own spots, the 25 digs designated through `SimCommand::Designate`, `world.step()` × 100, then the reachable dig designated. Preconditions, each asserted before the steps:
    - every cell around each pocket is `Solid`;
    - the camp component (a BFS over `astar_neighbours` from dwarf 0) holds more than `MAX_ASTAR_NODES` cells and holds no pocket.

    Without these, the test passes on `main` for the wrong reason. Assert a claim within 60 ticks of the late designation, then the tile reads `Tile::Empty` within 300 ticks. Debug cost at creation: 21 s, which fits the fast tier.
  - [x] REWRITE `a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on` (`:5527-5597`) as AC 3. Keep the `joined_plates` fixture: its 55,003 cells are over 50,000, so `main` exhausts there. Name it for what it pins. At tick 100: stamps `[120 ×10, 0]` and miner 0 claims dig 10. Then run the schedule at ticks 101–120: at tick 120 the 10 read `140` and `world.jobs().len()` is unchanged.
  - [x] `a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp` (`:5599-5680`): delete its exhaustion precondition (`:5641-5652`) and rename it. Rewrite its comment to say what it pins now: miner 0's flood is cached, the trade filter comes before `attempted`, and the next miner claims. Keep its claim and stamp asserts, which must still pass. `a_reachable_job_behind_unreachable_ones_is_claimed_when_areas_sum_past_the_budget` (`:5438-5499`) passes unchanged. Fix only comment text that is now false (it mentions `MAX_ASTAR_NODES`).
  - [x] The two-hundred bench test's doc comment (`crates/simd/tests/bench.rs:266-269`): add 13.7's reading, with its reason, from Task 3 (standing AC 11). Its `≥ 1` asserts stand.
- [x] **Task 3: the instrument, RED first (AC 1, 5).** The instrument is AC 1's test run with `--nocapture`. It prints one line, `claimed at tick T, dug at tick U` (or `never claimed by tick T`), which is what a human reads. The bench side is `simd --bench`'s end line and `state=` (`crates/simd/src/scenario.rs:410-420`), driven by `scripts/tick-bench.sh`. Both are tested by `crates/simd/tests/bench.rs` and `mutations/13-1.sh`.
  - [x] RED: apply row 1 by hand on a copy (scratch worktree, memory: `mutation-run-poisons-concurrent-builds`), run the test with `--nocapture`, and paste the failing line: no claim within 60 ticks. Then GREEN on HEAD: paste the line. Creation read t115 and t208 for designation at t100.
  - [x] AC 5: on an idle devpod, run `RUNS=5 scripts/tick-bench.sh main HEAD` over all 11 `scenarios/*.json` (`main`, at 13.6, reads the 256² file). Paste the table. Name the reason for the two-hundred `differ`, its end line on both sides, and the mean and p99 delta against the floor.
  - [x] AC 6, the profile, using throwaway patches committed only under `13-7-signoff/prototypes/`:
    - `astar-timer.patch`: an `Instant` accumulator inside `astar_with_budget` and `route_to_nearest`, plus the tick total, printed at the bench's end. This gives A\*'s share of the tick.
    - `astar-flat-storage.patch`: `costs` and `came_from` as `Vec`s indexed by `worldgen::index` over the terrain. This gives the mean and p99 of `--runs 3` against HEAD's, and its `state=`, which must equal HEAD's.

    Record both readings in the Debug Log. Changing the storage is a separate story, on Wolf's ruling.
- [x] **Task 4: mutation table (AC 7).** NEW `_bmad-output/implementation-artifacts/mutations/13-7.sh`, in `13-6.sh`'s shape. Commit first, then run it alone (`env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …`). Each row must be KILLED at its own assert. Read the panic line (memory: `mutation-test-name-prefix-trap`).
  1. the claim budget is `MAX_ASTAR_NODES` again → AC 1's test, at the claim assert (a heavy row: `main`'s budget costs 5 × 50,000 nodes a tick, so allow about 4 min in debug);
  2. the same sabotage → AC 3's rewrite, at the first stamps assert;
  3. the cooldown stamp becomes `u64::MAX` (dropped, never retried) → AC 3's rewrite, at the tick-120 assert.

  Retire the rows that sabotage removed code, each with a dated `# Retired` comment at the top of its file (`5-1-the-world-grows-things-that-glow.sh:4` is the shape): `12-4.sh` rows 9–16 (`:76-162`), and `3-2-the-dig.sh`'s "each claim search gets a fresh node budget" (`:954-971`, whose sabotage is now the code). Row 16 ("budget raised past the joined plates", `:156`) still applies, because its anchor is the constant, but it targets the deleted precondition's test. Run it once: if it SURVIVES, retire it. Re-point a row instead of retiring it only if its anchor survives and it still kills. `audit-mutations.py` must report no BROKEN row.
- [x] **Task 5: gate and close.** Fast tier on every commit (pre-commit hook). ONE full gate at PR: `RUST_TEST_THREADS=1 scripts/gate.sh` (~70 min, memory: `gate-ooms-at-default-parallelism`). Paste the block. Cost row: `python3 _bmad/scripts/session_tokens.py --phase dev --story 13-7-the-long-way`, then `phase_state.py clear`, then Status → review on the story and the board. The PR body may say `Closes #159` and nothing else closing; check `closingIssuesReferences` (memory: `closing-keyword-closes-whole-issue`).

### Review Findings

Code review run 1 (2026-11-02, a fresh session), on `main...88bb1f2d` (code diff 2 files, +205/−82; 17 files in all). Four layers, each running `cargo 1.97.1` in its own `CARGO_TARGET_DIR` under `/tmp/review-<layer>`. None timed out or failed. Territories: Blind Hunter (Sonnet) took `crates/sim-core`. Edge Case Hunter (Sonnet) took `crates/simd/tests/bench.rs`, the mutation tables, and the sim-core test fixtures, because the shell territory was thin. The Acceptance and Feature Auditors (Opus) took the whole diff. Nothing in the repo was modified. Sabotage ran on private `git archive HEAD` copies.

What the layers re-ran:
- **Tests:** `cargo test -p sim-core --lib` 103 passed, 1 ignored (blind, acceptance). The AC 1 test prints `claimed at tick 115, dug at tick 208` (acceptance, edge).
- **Mutation rows:** on private copies, rows 1, 2, 3 and 3b each die at their own assert (edge). Row 1 reads `never claimed by tick 160`, panic `:5543`. `audit-mutations.py` reads 973 rows (edge, acceptance).
- **`unreachable!` soundness:** a search expands each cell at most once (blind, RAN: 135 random searches, budget used = `explored.len()` on every failure; feature and acceptance, read). The arms cannot fire under today's rules.
- **Release bench:** `--bench scenarios/two-hundred-on-256.json --runs 1` at HEAD reads `state=6ba8edcc3d6c7b9c … done=dig:234,cut:90 stored=27 cells=2146`, and at `main` reads `3fc1461d491ee5d9 … dig:231,cut:91 stored=33 cells=2165`. Both match the record (feature, acceptance). A 5,000-tick HEAD run reads `stored=106` (feature), so the 33 → 27 is timing, not less hauling.
- **Cost and cleanup:** review row $15.92 (285 turns). `reap-build-caches.sh --tmp-only --force` reaped 13 directories (13.5 GB, 9.7 GB of free space reclaimed).
- **Hop trace (feature):** Designate → `apply_command` → `create_jobs` → `claim_jobs` (world budget) → `execute_jobs` → `set_tile` → delta. Every hop is WIRED. The re-plan hop is capped at 50,000 (decision 1).

Decisions:
- [x] [Review][Decision] **RESOLVED 2026-11-02, Wolf "1": option (a), now patch 0 below.** **A far reachable job that needs a re-plan over 50,000 nodes gets stuck in a silent claim/release loop** (feature, RAN; MED, latent silent failure) [crates/sim-core/src/lib.rs:2245-2246, `MAX_ASTAR_NODES` :56] — every dig or cut calls `clear_paths`, which wipes every dwarf's path. A walker then re-plans with `astar()` (capped at 50,000), and on `None` calls `retry_claim`. The job is released with a 20-tick cooldown, re-claimed with the world budget, and the loop repeats.
  - **Fixture:** a 256² world with a 51,200-cell plate and a 3-row serpentine mine. The dig is 772 steps by path, and the claim search takes 51,844 expansions.
  - **Results:**

    | Code | Terrain churn | Outcome |
    |---|---|---|
    | HEAD | none | dug at t8566 |
    | HEAD | one `set_tile` on a far cell every 8 ticks (the bench's real rate: 324 digs and cuts in 2,500 ticks) | 499 claims and 499 releases in 12,000 ticks, never dug |
    | HEAD with `astar()` on the world budget | same | dug at t8566 |
    | `main` | either | never claimed |

    So this is not a regression, but the story's promise ("however large the walkable ground grows") stops at the re-plan.
  - **Why the probes missed it:** Q1 (a) and the `MAX_ASTAR_NODES` NOTE accept that "a reachable re-plan past it would give up". The creation probes saw no such case: probe B's marks are all near camp, and probe A sampled only the undug valley. Neither measured that the consequence is a repeating loop, not a single give-up.
  - **Not reproduced on the fresh valley.**
  - **Options:**
    - (a) The `execute_jobs` re-plan only gets the world budget (`astar_with_budget` with `terrain.tiles.len()` at that one call site). `astar()`, `resolve_blocked_step`, `route_to_nearest` and `stranded` stay capped. `resolve_blocked_step` keeps the claim and waits, with no release. AC 4 and the NOTE are amended. Add a test with the serpentine fixture plus churn: no release while the path is reachable. Add a mutation row. Cost: creation B saw 19 re-plan exhaustions in 2,500 ticks at about 35 ms a flood. That is inferred at under 1% of the tick, so measure it.
    - (b) Keep Q1 (a). File an issue (`bug` + `route:story`) with this repro, and correct the NOTE to say "loops", not "gives up".

Patches:
- [ ] [Review][Patch] **Decision 1 (a): the `execute_jobs` re-plan gets the world budget** (feature, RAN; MED, latent silent failure) [crates/sim-core/src/lib.rs:2245-2246; NOTE :52-55; story AC 4] — at that one call site, use `astar_with_budget` with `terrain.tiles.len()` in place of `astar()`. `astar()`, `resolve_blocked_step`, `route_to_nearest` and `stranded` stay at `MAX_ASTAR_NODES`. Amend AC 4 and the NOTE (AMENDED 2026-11-02, naming this ruling). Add a test with the serpentine fixture plus terrain churn: the claim is not released while the job stays reachable. Add a mutation row that puts `astar()` back at the site. Re-read the tick-bench (10 rows `same`?) and the 256² cost. Both sides: the fixture is one HEAD loops on (499 claims and 499 releases), so the RED must reproduce the loop before the fix.
- [ ] [Review][Patch] **`audit-mutations.py` cannot see `13-7.sh` rows 1 and 2, so the gate's "mutation tables still apply" step cannot catch them rotting** (acceptance, RAN; MED, latent silent failure) [_bmad-output/implementation-artifacts/mutations/13-7.sh:7-57; scripts/audit-mutations.py `LITERAL`/`INLINE_ANCHOR`] — both rows keep their anchors in an `edits = [(old, new, n), …]` tuple list. `LITERAL` sees only a name bound to a literal, and `INLINE_ANCHOR` sees only `s.replace("…", …)`. So the rows are skipped entirely and are not even counted as unguarded. This is a fifth rot shape beside the four the script names. On a copy, Acceptance changed row 1's `let mut assigned = false;` to `let mut assigned: bool = false;` and the audit still printed `973 rows, every literal still matches`. The row's runtime `assert s.count(old) == n` fires only when someone re-runs `13-7.sh`. Fix: the audit reports any row from which it extracts no anchor (closing this shape for good), and rows 1 and 2 are rewritten so every literal is visible to it. Both sides: Acceptance's broken-anchor copy must turn the audit RED, and HEAD must stay clean.
- [ ] [Review][Patch] **AC 1's test does not assert that the 25 pocket digs are queued, or that the claim is the reachable dig** (acceptance, read; LOW, latent silent failure: memory `silent-sim-filter-trap`) [crates/sim-core/src/lib.rs:5505-5535] — the claim check is `claims().iter().any(|(_, job)| job.is_some())`. If a future designate filter drops the pocket digs, the test passes on `main`'s budget too. Fix: assert `jobs().len() == 25` after the 25 designations, and assert that the claimed job is the late dig's id.
- [ ] [Review][Patch] **AC 3's test does not assert that `joined_plates` is over `MAX_ASTAR_NODES`** (edge, RAN; LOW, latent: the test would pass on `main` for the wrong reason) [crates/sim-core/src/lib.rs:5626-5658] — AC 1 asserts `camp.len() > MAX_ASTAR_NODES`; AC 3 asserts nothing about its 55,003-cell area. The retired 12-4 row 16 was the only guard. Fix: a flood-size precondition assert. Both sides: on a copy, shrink the plate below 50,000 and the assert must fire.
- [ ] [Review][Patch] **`a_job_miner_zero_cannot_reach_…` has no mutation row** (edge, RAN; LOW, in `13-7.sh`, which patch 1 edits) [_bmad-output/implementation-artifacts/mutations/13-7.sh] — retiring 12-4 rows 11 and 13-15 removed every row on the "stamp, and go on to the next job" seam. Edge ran two mutants on a copy, and both died at `miner 1 claims the dig miner 0 cannot reach` (`left: (Id(1), None)`): `continue` → `break` in the goals-search `(None, false, explored)` arm, and dropping `component.contains(*pos)` from the cache skip. Add both, with count guards.
- [ ] [Review][Patch] **Record: the "sequential −5.8%" is a mismatched ratio, the p99 delta has no floor, and the File List is short** (acceptance, read; LOW, record, in this file) [story § Debug Log AC 5; § File List] — 33.18 / 35.22 is HEAD's run 1 against `main`'s run 3. The three-run means give −4.8% (33.31 vs 35.01). State that the p99 −3.3% comes from separate `--runs 3` runs with no floor. Add `epics.md` (Task 0) and the `metrics/` files to the File List.

Deferred (LOW, recorded in `deferred-work.md`):
- [x] [Review][Defer] `route_to_nearest`'s doc comment still says "Bounded by `MAX_ASTAR_NODES` expansions, like every other search", which has been false since 13.7 (acceptance, read) [crates/sim-core/src/lib.rs:1246] — deferred, LOW cap
- [x] [Review][Defer] In AC 3's test, the retry at tick 120 is done by the four one-cell-pocket miners, not by the dwarf in the area over 50,000 cells (miner 0 is busy with dig 10) (acceptance, read) [crates/sim-core/src/lib.rs:5658-5729] — deferred, LOW cap
- [x] [Review][Defer] The `unreachable!` arms rest on a consistent heuristic and unit step cost, and nothing pins that. A future weighted move would panic the daemon (blind, RAN; read by feature and acceptance) [crates/sim-core/src/lib.rs:656-668] — deferred, LOW cap
- [x] [Review][Defer] No scenario file carries the #159 shape. A far-only scenario (6 far dig columns) reads identical on `main` and HEAD (feature, RAN) [scenarios/] — deferred, LOW cap

Dismissed (4):
- `bench.rs` says 64,300 cells, but Edge's flood counts 64,299: the start cell counted or not.
- Successful searches per idle dwarf are no longer capped: this is the Q2 (a) design, and the bench measured it cost-neutral.
- The haul delivery A\* is repeated before the goals flood is cached: it is bounded by the cache, so only the first occurrence per component is paid.
- Stored 33 → 27 is timing, not less work: HEAD reaches `stored=106` by t5000.

## Verification

```bash
# AC 1 (RED via row 1 on a scratch copy first, then GREEN), AC 3, AC 4
cargo test -p sim-core --lib a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley -- --nocapture
cargo test -p sim-core --lib            # AC 3's rewrite, the 12.4 tests, the node-cap pins
# AC 5: bench delta, idle devpod, nothing else running
RUNS=5 scripts/tick-bench.sh main HEAD $(git ls-files 'scenarios/*.json')
# AC 6: profile (throwaway patches on a scratch worktree, release)
target/release/simd --bench scenarios/two-hundred-on-256.json --runs 3
# AC 7
env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-7.sh
python3 scripts/audit-mutations.py
# gate (once, at PR)
RUST_TEST_THREADS=1 scripts/gate.sh
```

## Dev Notes

### Scope guardrails (do NOT)

- No change to `MAX_ASTAR_NODES`, `astar()`, `route_to_nearest` or `stranded` (Q1 a). No heuristic change, no hierarchy, no connectivity map (ADR 5).
- No A\* storage change in shipped code (AC 6 records only). No optimization of the tick for NFR12.
- No new saved field, no `protocol` change, no new dependency, no gui or tui change.
- No fix for the reroute cost: 896 unreachable reroutes in `resolve_blocked_step` is a finding. Mention it in the record; do not fix it here.

### What already exists (build on it)

- `astar_with_budget` (`lib.rs:1193-1241`) returns `(path, exhausted, explored)`, and a completed failure hands back the whole component. The 12.3 cache (`components`, `:599`, `:650-656`) reuses it for the rest of the call.
- Test helpers: `stand_miners_at` (`:5412`), `insert_dig` (`:5428`), `joined_plates` (`:5503`), `set_profession`, `super::worldgen::index`. Also `World::generate_with_crew`, `Dims::FRESH`, `world.tiles()`, `world.claims()`, `world.jobs()`, and `apply_command(SimCommand::Designate { .. })`.
- `crates/sim-core/tests/worldgen.rs:1048` already pins the 256² camp component at more than 50,000 cells.
- The probe patches in `13-7-signoff/prototypes/` are throwaway. Port the test setup by hand; never ship `PROBE` statics.

### Key decisions & traps

- **The late designation is the red.** If all 26 digs are designated at once, `main` claims at t15 too, because reaction delays stagger the miners. Designate the reachable dig after the unreachable ones have cycled for 100 ticks.
- **`stranded` reads `MAX_ASTAR_NODES` as a piece-size cap** (`:1012`, `:1092`, `:1099`). It is a different rule and stays as it is.
- **The removed sat-out rule is what 12.4's three `#159` tests pinned.** Their fixtures still exhaust `main`'s budget, which is why AC 3 keeps `joined_plates`: it is a cheap 128² red beside AC 1's 256² one.
- **two-hundred-on-256's outcome moves** (stored 33 → 27 at creation). That is the fix, not a regression. The bench test asserts `≥ 1` and stands; the record says why (standing AC 11).
- **Probes and RED runs go in scratch worktrees, built from inside them, with jobs capped** (memory: `cargo-target-dir-builds-cwd-sources`, `build-parallelism-ooms-the-devpod`). Never run a `git checkout --` mutation over an uncommitted fix (memory: `sabotage-restore-trap`).

### Previous story intelligence

- 13.6 made `main`'s `simd` able to read `dims`/`crew`, so this story's tick-bench takes all 11 files. Its debug cost rule stands: no debug test steps 256² crew 200 for more than a few dozen ticks. AC 1 is crew 5.
- 13.6 Q4: the two-hundred real-work test runs only in the full tier (`gate.sh:191`). A rewritten fixture test belongs on the fast tier.
- 13.4: once a backstop lands, old rows can go inert. Re-run any row that is re-pointed, not just audit it (memory: `backstop-makes-old-rows-inert`).

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs`: `claim_jobs`, NOTEs at `:52-53` and `:595-598`, and tests `:5438-5680` plus one new test
- UPDATE `crates/simd/tests/bench.rs`: doc comment `:266-269` only
- UPDATE `_bmad-output/implementation-artifacts/mutations/12-4.sh`, `3-2-the-dig.sh` (retired rows); NEW `mutations/13-7.sh`
- UPDATE `_bmad-output/planning-artifacts/epics.md` Story 13.7 (Q3 a)
- `13-7-signoff/`: `creation-probes.txt`, `issue-159-comment.md`, `prototypes/{probe159-counters,probe-the-long-way,budget-all-sites}.patch` (creation); `prototypes/{astar-timer,astar-flat-storage}.patch` (Task 3)

### References

- `_bmad-output/planning-artifacts/epics.md` Story 13.7 (`:3111-3129`), Epic 13 standing ACs (`:2950-2962`)
- `docs/technical-preferences.md` ADR 5 (scope line, Wolf 2026-10-23)
- `_bmad-output/implementation-artifacts/13-2-signoff/proposal.md` § Scale (`:178-223`), `:121-124` (the `BTreeMap` question), § Ruling item 4
- `crates/sim-core/src/lib.rs:52-63`, `:559-703`, `:1012-1139`, `:1141-1297`, `:5412-5680`, `:5916-5958`
- `crates/simd/src/scenario.rs:322-440`; `crates/simd/tests/bench.rs:260-300`; `scripts/tick-bench.sh`, `scripts/gate.sh:191`, `scripts/audit-mutations.py`
- `_bmad-output/implementation-artifacts/mutations/12-4.sh`, `3-2-the-dig.sh:400-425`, `:954-971`
- GitHub #159 (and #132, the 12.3 cache)

## Dev Agent Record

### Agent Model Used

Wolf picked Sonnet 5.5 subagents for the implementation (2026-11-02); Opus 5.5 orchestrates and verifies, and keeps the mutation table, every mutation run, the bench and the full gate.

### Debug Log References

- **AC 1 RED (Task 3, row 1 applied by hand on a scratch worktree off `17d407e3` + the 13.7 tree; row 1 restores `main`'s per-dwarf budget and sat-out whole):** `never claimed by tick 160`, then `panicked at crates/sim-core/src/lib.rs:5543:13: the reachable dig must be claimed within 60 ticks of its designation (tick 100)` (131.8 s debug). The Sonnet agent saw the same line on the untouched pre-Task-1 code, test written first (134 s).
- **AC 1 GREEN (the 13.7 tree):** `claimed at tick 115, dug at tick 208` (21.1 s debug), the creation reading exactly.
- **AC 7 mutation run** (`641dbc7b`, alone, `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …/13-7.sh`, 2 min 17 s): **4/4 KILLED**, each at its own assert. Row 1: the AC 1 test at the claim assert (`lib.rs:5543`, `never claimed by tick 160`). Row 2: the AC 3 rewrite at the first stamps assert (`:5709`, shifted by the sabotage's added lines). Row 3: the same assert (`:5703`). Row 3b: the tick-120 assert (`:5719`, `tick 120: the 10 are re-stamped only when the cooldown ends at 120`).
- **Re-pointed rows, re-run** (scratch table holding just these two rows, same command): `3-2-the-dig.sh` "unreachable lower id starves a reachable dwarf" **KILLED** at its own test (`an_unreachable_lower_id_does_not_starve_a_reachable_dwarf`, `lib.rs:5243`). `12-4.sh` row 16 "budget raised past the joined plates" **SURVIVED** on its renamed target, as the story expected: the claim no longer reads the constant. Retired with a dated comment. `audit-mutations.py`: 973 rows, every literal matches, no BROKEN row.
- **AC 5 bench** (`RUNS=5 scripts/tick-bench.sh main HEAD` over all 11 `scenarios/*.json`; base `483b95b1` = `main`, head `641dbc7b`; load 2.71 before, 1.64 after):

  | scenario | base_ms | head_ms | delta | control | base_floor | head_floor | ctl_floor | state | verdict |
  |---|---|---|---|---|---|---|---|---|---|
  | big-pile | 0.1977 | 0.1966 | -0.6% | +0.7% | 2.7% | 3.2% | 5.0% | same | within floor |
  | deep-pit | 0.0382 | 0.0373 | -2.5% | -1.6% | 3.4% | 3.8% | 6.6% | same | within floor |
  | dig-loop | 0.0278 | 0.0276 | -0.6% | -0.8% | 1.8% | 5.4% | 3.6% | same | within floor |
  | full-pile-crowd | 0.0317 | 0.0317 | -0.1% | +0.4% | 3.8% | 2.5% | 4.4% | same | within floor |
  | plan-compact-piles | 0.1561 | 0.1517 | -2.8% | +1.4% | 0.6% | 2.8% | 1.6% | same | within floor |
  | plan-far-piles | 0.0906 | 0.0873 | -3.6% | +0.5% | 2.0% | 2.9% | 4.4% | same | within floor |
  | plan-spread-piles | 0.1790 | 0.1723 | -3.7% | +0.3% | 0.6% | 3.5% | 1.9% | same | BEYOND FLOOR |
  | plan | 0.0863 | 0.0845 | -2.2% | +0.2% | 0.7% | 2.4% | 1.3% | same | within floor |
  | seat-slow | 0.1733 | 0.1663 | -4.0% | -0.3% | 5.8% | 6.3% | 3.3% | same | within floor |
  | stuck-tunnel | 0.0284 | 0.0274 | -3.4% | -2.6% | 2.5% | 6.2% | 6.5% | same | within floor |
  | two-hundred-on-256 | 34.5276 | 33.5176 | -2.9% | +0.2% | 7.7% | 1.5% | 2.8% | differ | within floor |

  `same` on all ten 128² rows. plan-spread-piles' -3.7% BEYOND FLOOR is a speed-up, about 2x its 1.9% control floor: weak evidence between two builds (tick-bench's own NOTE), and plausibly real, since 13.7 removed the per-dwarf budget vector and the sat-out branch. **two-hundred-on-256 `differ`, and why:** its claim searches now complete instead of exhausting (creation B: 238/238 claim exhaustions on `main`), so jobs are claimed at other ticks. State `3fc1461d491ee5d9` → `6ba8edcc3d6c7b9c`, the creation prototype's state exactly. End lines (separate `--runs 3` per binary, sequential): Both are given below.
  - `main` (`483b95b1`) end: `marks=1478 items=602 zones=1800 goals=28/0/0 done=dig:231,cut:91 stored=33 cells=2165`; mean 34.54 / 35.26 / 35.22 ms, p99 160.5 / 159.6 / 158.7 (median 159.6), max 249-279.
  - HEAD (`641dbc7b`) end: `marks=1476 items=600 zones=1800 goals=22/0/0 done=dig:234,cut:90 stored=27 cells=2146`; mean 33.18 / 33.14 / 33.61 ms, p99 152.7 / 154.3 / 154.7 (median 154.3), max 302-308.
  - **Delta against the floor:** mean -2.9% interleaved (tick-bench), inside base's 7.7% floor; the sequential runs read -5.8%. p99 median -3.3%, max +16% (creation: +18%). So it is cost-neutral, as creation D found. The bench test's `≥ 1` asserts stand, and its doc comment now carries this reading (standing AC 11).
- **AC 6 profile** (HEAD `641dbc7b`, two-hundred-on-256, release; throwaway patches committed only under `13-7-signoff/prototypes/`):
  - `astar-timer.patch` (`--runs 1`): `tick total 83862.2 ms, astar_with_budget 39539.0 ms (47.1%), route_to_nearest 583.3 ms (0.7%)`; mean 33.54 ms, state `6ba8edcc3d6c7b9c` (= HEAD, so the timer costs nothing measurable). **A\* is 47.1% of the tick.** `astar_with_budget` also counts `astar()`'s re-plans and the 896 unreachable reroutes in `resolve_blocked_step` (creation B, not fixed here: a finding, per the scope guardrails). The systems are `.chain()`ed, so the summed time is a share of wall time, not of thread time.
  - `astar-flat-storage.patch` (`costs`/`came_from` as `Vec`s over the terrain, indexed by `worldgen::index`; `--runs 3`): mean 77.37 / 74.26 / 71.68 ms, p99 456.5 / 422.8 / 403.7 (median 422.8), max 487-559; state `6ba8edcc3d6c7b9c` = HEAD's. **Mean +124%, p99 +174% against HEAD: flat storage as specified is SLOWER.** Inferred, not measured: each search allocates and fills two 2,097,152-entry vectors (8 MB of `u32` costs, 25 MB of `Pos`) whatever its size, and most searches are small. A flat layout would need storage reused across searches to win, which is a signature change and a separate story on Wolf's ruling. No storage change is committed.

- **Full gate GREEN on `92b5e326`** (`RUST_TEST_THREADS=1 scripts/gate.sh`, exit 0):

  ```
  frostvein gate
    cargo fmt --check           ok  1s
    cargo clippy -D warnings    ok  2s
    cargo test                  ok  1155s
    cargo test (release real-work scenario)ok  94s
    cargo test (pixel guards)   ok  3134s
    tui has no sim-core edge                ok
    client-core has no sim-core edge        ok
    gui has no sim-core edge                ok
    metrics ledger tests        ok  0s
    bench tests                 ok  20s
    mutation tables still apply ok  10s
  GATE GREEN  4424s
  ```

### Completion Notes List

- **Tasks 1-2 (Sonnet 5.5 agent; Opus reviewed the diff).** `claim_jobs`: `budgets`, `sat_out` and the `enumerate`/`slot` removed; each `astar_with_budget` call gets `let mut budget = terrain.tiles.len();`; one `(_, true, _) => unreachable!("a search sized to the world cannot run out")` arm per match; stamp is `attempted && !assigned`; the header comment, the `components` NOTE and the `MAX_ASTAR_NODES` NOTE rewritten. `MAX_ASTAR_NODES`, `astar()`, `route_to_nearest` and `stranded` are untouched.
- Tests: NEW `a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley` (AC 1, preconditions asserted: pockets sealed, camp area > `MAX_ASTAR_NODES` and holds no pocket); `a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on` → `unreachable_digs_are_stamped_in_one_flood_and_retried_after_the_cooldown` (AC 3: `[120 ×10, 0]`, miner 0 claims dig 10 at tick 100, ticks 101-120 checked each tick, `140` at 120, `jobs().len()` unchanged); `a_job_one_dwarf_exhausted_on_…` → `a_job_miner_zero_cannot_reach_goes_to_the_next_of_his_trade_and_others_still_stamp` (precondition deleted, claim and stamp asserts unchanged and passing); comment-only fixes in `a_reachable_job_behind_unreachable_ones_is_claimed_when_areas_sum_past_the_budget` and `joined_plates`. `cargo test -p sim-core --lib` 103 passed / 1 ignored; fmt and clippy clean.
- **Deviation: Tasks 1, 2 and the Task 4 retirements landed in ONE commit.** The story asked for a commit per task, with retirements after. Neither order can pass the pre-commit hook: the agent's Task-1-only commit went GATE RED on `audit-mutations.py` (9 rows CANNOT APPLY against removed code), and a Task-1-only tree also fails the old 12.4 tests, which pinned the sat-out rule it removes. So code, tests and table edits go together.
- **Not on the story's list:** `3-2-the-dig.sh` "unreachable lower id starves a reachable dwarf" was anchored on the old `budgets[slot]` goals match. Its seam (the claim skips the goals search) survives, so it is re-pointed to the new literal rather than retired, and must be re-run (memory: `backstop-makes-old-rows-inert`).
- **Row 1's shape:** it restores `main`'s budget whole (per dwarf, per tick, sat-out), not a 50,000 cap per search. A per-search cap would hit the new `unreachable!` arm and KILL for the wrong reason. Row 3 (`u64::MAX` stamp) as the story wrote it dies at the FIRST stamps assert, not at tick 120, so row 3b (a stamped job is never retried) was added for the tick-120 assert.

### File List

- `crates/sim-core/src/lib.rs` (UPDATE: `claim_jobs`, NOTEs, tests)
- `_bmad-output/implementation-artifacts/mutations/13-7.sh` (NEW)
- `crates/simd/tests/bench.rs` (UPDATE: the two-hundred test's doc comment only)
- `_bmad-output/implementation-artifacts/mutations/12-4.sh` (rows 9-16 retired: 9-15 against removed code, 16 SURVIVED on re-run)
- `_bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh` (one row retired, one re-pointed)
- `_bmad-output/implementation-artifacts/13-7-signoff/prototypes/astar-timer.patch`, `astar-flat-storage.patch` (NEW, throwaway, Task 3)
- `_bmad-output/implementation-artifacts/13-7-the-long-way.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

| Date | Change |
|---|---|
| 2026-11-02 | Story created (Opus 5.5). Creation probes: every 256² budget exhaustion is an unreachable goal (13.2's reading corrected); the red reproduces on the real valley; claim-only resize is cost-neutral. Evidence in `13-7-signoff/`. Task 0 Q1–Q3 await Wolf. |
| 2026-11-02 | Task 0 RULED (Wolf, "ok for all"): Q1 (a) claim-only resize; Q2 (a) per-search world-sized budget, 12.4 sat-out removed; Q3 (a) epic 13.7 ACs 1-2 amended, `issue-159-comment.md` posted on #159. ACs and Tasks stand as written. Next: ask which implementer. |
| 2026-11-02 | Dev started: Wolf picked Sonnet 5.5 subagents (Opus orchestrates). Status in-progress. |
| 2026-11-02 | Dev complete (Sonnet 5.5 agent for Tasks 1-2, Opus for Tasks 3-5): `641dbc7b` claim budget sized to the world + tests + mutation retirements, `92b5e326` record. AC 1 RED/GREEN, 13-7.sh 4/4 KILLED, tick-bench 10/10 same (256² differ, explained), A* 47.1% of the 256² tick, flat storage slower. Full gate GREEN on `92b5e326` (4,424 s). Status review. |
| 2026-11-02 | Code review run 1 (fresh session, 4 layers, none failed): 0 HIGH. Decision 1, Wolf "1": the `execute_jobs` re-plan gets the world budget, which amends Q1 (a) and AC 4 at that one site. 6 patches left as action items (Wolf "2"), 4 deferred, 4 dismissed. Status in-progress. |
