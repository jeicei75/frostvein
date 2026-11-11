---
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
baseline_commit: 483b95b1
---

# Story 13.7: The Long Way

Status: done

## Story

As the boss,
I want a dwarf on a 256² valley to reach a reachable job however large the walkable ground grows,
so that work far from the camp is never silently starved by the path-search budget (#159).

## Branch: off local `main`, which is ahead of `origin/main`

Branch `story-13-7-the-long-way` was cut from local `main` `483b95b1`. That commit carries the unpushed 13.6 done-mark, which rides into this PR, as `3319e68c` did for 13.4. Epic 13's standing ACs bind: M3's ACs 1–8, with **one full gate per story, at PR**, plus M4 standing ACs 10–12 and 18. ACs 13–17 do not apply: there are no needs, stock, trades or rendering here. **No wire change and no saved field.** The budget is derived from the terrain, so no state hash moves through the save encoding (memory: `state-hash-covers-save-encoding`). **No seat card:** this is sim work, judged by scenario tests (M2-26).

## Task 0 — Wolf rules before dev

Each question has a recommendation. The ACs and Tasks are written for it, and a different answer amends the ACs it names.

- **Q1. Which searches get the new budget?** Every 256² exhaustion is an unreachable goal (*Found at creation* B), and the other sites' `None` is the same answer either way (C).
  - (a) **Recommended: `claim_jobs` only. ← RULED 2026-11-02.** It is the only site whose outcome moves, and it is cost-neutral: mean −1%, p99 flat, max +18% (D). `astar()`, `route_to_nearest` and `stranded` keep `MAX_ASTAR_NODES` = 50,000, with a `// NOTE:` naming what 13.7 measured. **AMENDED 2026-11-02 (review run 1, decision 1):** the `execute_jobs` re-plan also gets the world's cell count; see AC 4 as amended.
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
  - [x] `MAX_ASTAR_NODES` NOTE (`:52-53`): it now bounds `astar()`, `route_to_nearest` and `stranded` only. 13.7 found every one of their 256² exhaustions to be an unreachable goal, so capping gives the same `None`, cheaper. A reachable reroute or re-plan past it would give up (`resolve_blocked_step` `:1887`, `execute_jobs` `:2263` → `retry_claim`), which no probe has seen. Do not change the value (AC 4). **AMENDED 2026-11-02 (review run 1, decision 1):** the `execute_jobs` re-plan no longer gives up. Under terrain churn a capped re-plan looped claim/release, so that site now gets the world's cell count (AC 4 as amended). Only `resolve_blocked_step`'s reroute is still capped there.
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
- [x] [Review][Patch] **Decision 1 (a): the `execute_jobs` re-plan gets the world budget** (feature, RAN; MED, latent silent failure) [crates/sim-core/src/lib.rs:2245-2246; NOTE :52-55; story AC 4] — at that one call site, use `astar_with_budget` with `terrain.tiles.len()` in place of `astar()`. `astar()`, `resolve_blocked_step`, `route_to_nearest` and `stranded` stay at `MAX_ASTAR_NODES`. Amend AC 4 and the NOTE (AMENDED 2026-11-02, naming this ruling). Add a test with the serpentine fixture plus terrain churn: the claim is not released while the job stays reachable. Add a mutation row that puts `astar()` back at the site. Re-read the tick-bench (10 rows `same`?) and the 256² cost. Both sides: the fixture is one HEAD loops on (499 claims and 499 releases), so the RED must reproduce the loop before the fix. — **LANDED `faad260a`** (row 4 `f5b106b4`): RED `claims=8 releases=8 by tick 201`, GREEN `claims=1 releases=0`; fixture is a 256² plate + corridor, not the reviewer's unsaved serpentine (see patch pass 1).
- [x] [Review][Patch] **`audit-mutations.py` cannot see `13-7.sh` rows 1 and 2, so the gate's "mutation tables still apply" step cannot catch them rotting** (acceptance, RAN; MED, latent silent failure) [_bmad-output/implementation-artifacts/mutations/13-7.sh:7-57; scripts/audit-mutations.py `LITERAL`/`INLINE_ANCHOR`] — both rows keep their anchors in an `edits = [(old, new, n), …]` tuple list. `LITERAL` sees only a name bound to a literal, and `INLINE_ANCHOR` sees only `s.replace("…", …)`. So the rows are skipped entirely and are not even counted as unguarded. This is a fifth rot shape beside the four the script names. On a copy, Acceptance changed row 1's `let mut assigned = false;` to `let mut assigned: bool = false;` and the audit still printed `973 rows, every literal still matches`. The row's runtime `assert s.count(old) == n` fires only when someone re-runs `13-7.sh`. Fix: the audit reports any row from which it extracts no anchor (closing this shape for good), and rows 1 and 2 are rewritten so every literal is visible to it. Both sides: Acceptance's broken-anchor copy must turn the audit RED, and HEAD must stay clean. — **LANDED `b74f1242`**: fails the gate (21 older unreadable rows listed by name, Wolf's pick at the patch checkpoint).
- [x] [Review][Patch] **AC 1's test does not assert that the 25 pocket digs are queued, or that the claim is the reachable dig** (acceptance, read; LOW, latent silent failure: memory `silent-sim-filter-trap`) [crates/sim-core/src/lib.rs:5505-5535] — the claim check is `claims().iter().any(|(_, job)| job.is_some())`. If a future designate filter drops the pocket digs, the test passes on `main`'s budget too. Fix: assert `jobs().len() == 25` after the 25 designations, and assert that the claimed job is the late dig's id. — **LANDED `f5b106b4`** (the 25 are asserted after the 100 steps: jobs are created in `step()`); rows 5, 6 KILLED.
- [x] [Review][Patch] **AC 3's test does not assert that `joined_plates` is over `MAX_ASTAR_NODES`** (edge, RAN; LOW, latent: the test would pass on `main` for the wrong reason) [crates/sim-core/src/lib.rs:5626-5658] — AC 1 asserts `camp.len() > MAX_ASTAR_NODES`; AC 3 asserts nothing about its 55,003-cell area. The retired 12-4 row 16 was the only guard. Fix: a flood-size precondition assert. Both sides: on a copy, shrink the plate below 50,000 and the assert must fire. — **LANDED `f5b106b4`**; row 7 KILLED.
- [x] [Review][Patch] **`a_job_miner_zero_cannot_reach_…` has no mutation row** (edge, RAN; LOW, in `13-7.sh`, which patch 1 edits) [_bmad-output/implementation-artifacts/mutations/13-7.sh] — retiring 12-4 rows 11 and 13-15 removed every row on the "stamp, and go on to the next job" seam. Edge ran two mutants on a copy, and both died at `miner 1 claims the dig miner 0 cannot reach` (`left: (Id(1), None)`): `continue` → `break` in the goals-search `(None, false, explored)` arm, and dropping `component.contains(*pos)` from the cache skip. Add both, with count guards. — **LANDED `a57ddeff`**; rows 8, 9 KILLED.
- [x] [Review][Patch] **Record: the "sequential −5.8%" is a mismatched ratio, the p99 delta has no floor, and the File List is short** (acceptance, read; LOW, record, in this file) [story § Debug Log AC 5; § File List] — 33.18 / 35.22 is HEAD's run 1 against `main`'s run 3. The three-run means give −4.8% (33.31 vs 35.01). State that the p99 −3.3% comes from separate `--runs 3` runs with no floor. Add `epics.md` (Task 0) and the `metrics/` files to the File List. — **LANDED** in the patch pass 1 record commit (-4.85% recomputed from the six means).

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

#### Code review run 2 (2026-11-03, fresh session, Opus 5.5) on `35c6d6f4..b9f7209b`

**Delta against run 1:** 4 NEW findings (0 HIGH / 1 MED / 3 LOW) and 1 REWORK (P2 is half-closed: row 6). 3 deferred, 4 dismissed. **Stopping rule: no NEW finding is HIGH, so this round ends the audit.** No round 3. 13.7 has no seat card (sim work, M2-26), so the next spend after the patches is the PR.

Code diff: 3 files, +363/−56 (`lib.rs`, `audit-mutations.py`, `mutations/13-7.sh`). Four layers ran `cargo 1.97.1`, each in its own `/tmp/review-<layer>` target. None timed out or failed. Territories:
- **Blind Hunter (Sonnet):** the sim-core diff, blind.
- **Edge Case Hunter (Sonnet):** the audit script, `13-7.sh` and the test fixtures.
- **Acceptance and Feature Auditors (Opus):** the whole diff.

Nothing in the repo was modified (`git status` is clean). Sabotage and probes ran on private `git archive` copies. Cost: review row $15.03 (289 turns, 4 subagents = 66.8% of tokens). `reap-build-caches.sh --tmp-only --force` reaped 12 directories (13.5 GB; 8.1 GB of free space reclaimed).

What the layers re-ran:
- **The new test, at HEAD:** `a_far_claim_…` reads `claims=1 releases=0 by tick 201` (all four layers).
- **Row 4 by hand** (`astar()` back at the re-plan): the test dies at the churn assert, `left: (8, 8)` (edge, acceptance, blind).
- **The far dig is actually dug** (feature, RAN; release probe on a copy, run to completion): `dug_at=Some(11140)`, claimed once, through about 1,390 path wipes.
- **The AC 1 test:** reads `claimed at tick 115, dug at tick 208` (feature).
- **Release bench at HEAD:** `state=6ba8edcc3d6c7b9c`, `done=dig:234,cut:90 stored=27 cells=2146`, which equals the record. The timing was taken under the load of three sibling layers, so it is no evidence on cost (feature).
- **`audit-mutations.py`:**
  - 979 rows, exit 0 at HEAD.
  - Exit 1 on `13-7.sh` as of `35c6d6f4`, and exit 1 on the broken-anchor copy.
  - With the allow-list emptied, exactly its 21 rows fail (edge, acceptance).
  - Six single-line rot probes on a copy were each caught (edge).
- **Rows 5-9 by hand:**
  - Each dies at its recorded assert.
  - With the assert each row is credited to removed, row 5 then passes, so its new assert is the only killer.
  - Row 6 still dies, at the older `must be dug within 300 ticks` assert (acceptance; see the REWORK patch below).
- **Hop trace (feature):** Designate (`:3264`) → `create_jobs` (`:410`) → `claim_jobs` (`:567`, world budget) → `execute_jobs` (`:2090`) → `clear_paths` (`:2552`, `:1348`, `:3167`) → re-plan on the world budget (`:2250`) → work → `set_tile` → `drain_dirty` → `bridge.rs:56`. Every hop is WIRED. No other `retry_claim` site (`:2234`, `:2341`, `:2414`, `:2483`, `:2547`) depends on the path budget.

**Closure-table audit** (acceptance, RAN):

| Row | Verdict |
|---|---|
| P0 | CLOSED |
| P1 | CLOSED as written. "For good" is overstated: see patches 1 and 2. |
| P2 | HALF-CLOSED (REWORK): the queue half is closed, the id half is unpinned |
| P3 | CLOSED |
| P4 | CLOSED |
| P5 | CLOSED |

Decisions:
- [x] [Review][Decision] **RESOLVED 2026-11-03, Wolf "a": now patch 0 below.** **The allow-listed rows are never anchor-checked, so their rot stays invisible** (edge + acceptance, RAN; latent silent failure) [scripts/audit-mutations.py:72-103 `UNREADABLE_ALLOWED`, :220] — the gate skips all 21 entirely. Two of them already cannot apply (patch 1). The comment's "listed so they stay visible" means visible only in the script's source. Of the 21, 9 keep their anchor in a parenthesised implicit concatenation `old = ('…' '…')`, which `ast.literal_eval` reads exactly: all of `12-10.sh`'s and `12-10b.sh`'s rows and both of `12-12.sh`'s, including both dead rows. The other 12 are `for old in` loops (4), a regex (1), and 7 the parser cannot delimit.
  - (a) Teach the audit the concatenated form, and drop those 9 rows from the allow-list once they read clean. 12 stay grandfathered.
  - (b) Fix only patch 1's two rows, and file an issue for the remaining 19.

Patches:
- [x] [Review][Patch] **Decision 1 (a): the audit reads a parenthesised implicit concatenation as a literal** (edge + acceptance, RAN; latent silent failure) [scripts/audit-mutations.py `LITERAL` :47-50, `UNREADABLE_ALLOWED` :72-103] — add an alternative for `name = ('…' '…' …)` to `LITERAL`; `ast.literal_eval` already joins it. Drop the 9 concatenated rows from `UNREADABLE_ALLOWED` once each reads clean, and amend the NOTE (12 stay: 4 loops, 1 regex, 7 undelimitable). Land it together with the next patch, since it turns the gate RED on those two rows. Both sides: before the re-point, the new audit must name exactly `12-10b` row 1 and `12-12` row 16 as CANNOT APPLY; after it, it exits 0, and a broken anchor in one of the other 7 on a copy is caught. — **LANDED `02eb29a7`**: 8 rows leave the list, not 9 (12-12 row 18 is a loop row; see patch pass 2).
- [x] [Review][Patch] **Two allow-listed rows cannot apply: their anchors match 0 times** (acceptance, RAN; MED, latent silent failure, rot predating 13.7 that patch pass 1 grandfathered unchecked) [mutations/12-10b.sh:6-17 row 1; mutations/12-12.sh:147-154 row 16] — row 1's anchor is now `let goals: BTreeSet<_> = astar_neighbours(…)` (`lib.rs:1394`). Row 16's `self.pile_ground…open.clear();` is now `let kept = self.pile_ground…; kept.open.clear(); kept.memo = None;` (`lib.rs:756-761`). Both already counted 0 at `origin/main` and at `483b95b1`. Re-point each to today's source and run it alone. If it KILLS, keep it; if it SURVIVES, retire it with a dated comment (Task 4's rule). Either way, drop it from `UNREADABLE_ALLOWED`. Both sides: the audit must go RED on each old anchor before the re-point. — **LANDED `02eb29a7`**: both re-pointed and KILLED.
- [x] [Review][Patch] **A row with one readable anchor still hides rot in a tuple list or loop beside it** (edge + acceptance, RAN; LOW, latent silent failure: the fifth shape, half-closed in mixed rows) [scripts/audit-mutations.py:153-221] — `checked` goes True on any one literal, so the row counts as read. A probe row with a guarded `old = '…'` plus `edits = [('THIS ANCHOR IS NOT IN THE SOURCE', 'x', 1)]` drew no report. Fix: also flag a row whose block holds an anchor loop or tuple list (`for … in (`/`[`, `edits = [`) unless it is allow-listed. Both sides: the probe row must turn the audit RED, and HEAD must stay clean (or name every existing mixed row). — **LANDED `f21c799b`**: HEAD clean (the 5 mixed rows are all allow-listed).
- [x] [Review][Patch] **REWORK of P2: row 6 does not pin the claim-id assert** (acceptance, RAN; LOW, latent silent failure: memory `strengthened-test-needs-remutation`) [mutations/13-7.sh row 6; crates/sim-core/src/lib.rs:5563] — with the id assert replaced by `let _ = (job, late);`, row 6 still dies, at the older `must be dug within 300 ticks` assert. All five miners hold pocket claims forever, so the late dig is never dug. Fix: rewrite row 6 so that only the id assert can catch it: a wrong claim held by one dwarf while another still claims and digs the late dig. Re-run it with and without the id assert. If no such sabotage can be built, say in the closure table that no row pins the id assert. — **LANDED `56229c37`**: KILLED at the id assert; passes with it stubbed.
- [x] [Review][Patch] **The record still states the pre-amendment rule, and the AC 5 and AC 6 figures predate the re-plan change** (acceptance, read; LOW, record: memory `partial-doc-update-immunises`) [this file :25, :72, § Debug Log AC 5 and AC 6, § File List; deferred-work.md 13-7 run 1 `route_to_nearest` entry] — fixes:
  - Line 25 (Q1 (a): "`claim_jobs` only") and line 72 (Task 1's NOTE text, "a reachable reroute or re-plan past it would give up … `execute_jobs` → `retry_claim`") get AMENDED pointers to AC 4.
  - AC 5's "cost-neutral / max +16%" is from `641dbc7b`. Say that HEAD against `main` was not re-read after the patch, and that by the record's own numbers the max is roughly +30-45% over `main`.
  - AC 6's "A* 47.1%" predates the re-plan change. Say so.
  - Add `deferred-work.md` to the File List.
  - The `route_to_nearest` deferred entry now names two world-sized sites, not one.
  — **LANDED `af3c19bf`**.

Deferred (LOW, recorded in `deferred-work.md` under run 2):
- [x] [Review][Defer] An unreachable `execute_jobs` re-plan floods the whole component, with no per-tick cache. Measured +2.6-3.9% mean / +21% max today; it grows with the dug area (blind + feature, read) [crates/sim-core/src/lib.rs:2250-2258] — deferred, LOW cap
- [x] [Review][Defer] `resolve_blocked_step`'s capped reroute holds a far walker on open ground over 50,000 cells behind a parked job-holder; the claim is kept (blind + feature, read, no repro) [crates/sim-core/src/lib.rs:1871, :2283-2297] — deferred, ruled by AC 4 as amended
- [x] [Review][Defer] The churn test proves the claim is kept for 200 ticks, not that the dig completes. Feature's release run of the same fixture dug at t11140 (blind, read; feature, RAN) [crates/sim-core/src/lib.rs:5773-5880] — deferred, LOW cap

Dismissed (4):
- `&mut terrain.tiles.len()` as an anonymous temporary (blind): idiomatic; a fresh budget per call is the intent.
- The plate's "50,800 cells" comment and the 8-tick churn period are not asserted (blind): the precondition asserts the property that matters (capped `astar()` gives `None`).
- The test cost of the `walkable_area` helper (blind): blind itself raised no issue.
- "Re-plan cost unmeasured" (blind, reviewed blind): patch pass 1 measured it. It is folded into the first deferral.

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
  - **Delta against the floor:** mean -2.9% interleaved (tick-bench), inside base's 7.7% floor; the sequential runs' three-run means read -4.8% (33.31 vs 35.01 ms; **CORRECTED 2026-11-02**, review run 1: the -5.8% first written here was HEAD's run 1 over `main`'s run 3). p99 median -3.3%, from those separate `--runs 3` runs, which have no floor of their own; max +16% (creation: +18%). So it is cost-neutral, as creation D found. The bench test's `≥ 1` asserts stand, and its doc comment now carries this reading (standing AC 11).
  - **STALE 2026-11-03 (review run 2): these figures are `641dbc7b` against `main`, from BEFORE patch pass 1's re-plan change.** HEAD against `main` was not re-read after it. Patch pass 1 measured that change alone at +2.6% mean (sequential) / +3.9% (tick-bench, BEYOND FLOOR) and max 299-304 -> 362-366 ms. Against `main`'s max 249-279 above, HEAD's max is roughly +30-47%, by the record's own numbers from separate sessions with no shared floor. So "cost-neutral" holds for the mean and p99 of `641dbc7b`, not for HEAD's max.
- **AC 6 profile** (HEAD `641dbc7b`, two-hundred-on-256, release; throwaway patches committed only under `13-7-signoff/prototypes/`):
  - `astar-timer.patch` (`--runs 1`): `tick total 83862.2 ms, astar_with_budget 39539.0 ms (47.1%), route_to_nearest 583.3 ms (0.7%)`; mean 33.54 ms, state `6ba8edcc3d6c7b9c` (= HEAD, so the timer costs nothing measurable). **A\* is 47.1% of the tick.** `astar_with_budget` also counts `astar()`'s re-plans and the 896 unreachable reroutes in `resolve_blocked_step` (creation B, not fixed here: a finding, per the scope guardrails). The systems are `.chain()`ed, so the summed time is a share of wall time, not of thread time. **STALE 2026-11-03 (review run 2):** measured at `641dbc7b`, before patch pass 1 gave the `execute_jobs` re-plan the world budget. Each unreachable re-plan now floods its whole component, so A\*'s share at HEAD is likely higher; it was not re-measured.
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

- **Review patch pass 1 (2026-11-02, fresh session, Opus 5.5).** Commits: `b74f1242` audit, `faad260a` re-plan fix + test + AC 4/NOTE, `f5b106b4` AC 1/AC 3 asserts + rows 4-7, `a57ddeff` rows 8-9, then the record commit (row 6 two-site rewrite + this record).
  - **Fixture deviation (patch 0).** The reviewer's 256² serpentine was not saved. `joined_plates` (128²) was tried first and its own precondition refused it: a capped `astar()` from (0,0,1) to plate 4 SUCCEEDS, so a test on it would pass on the old code. The test builds a 256² world: a 254 x 200 plate at z = 1 (50,800 cells), a ramp up at its far corner, and a sealed corridor at z = 3 (north, west, south) to a dig at (1, 0, 3) above the start. The heuristic points straight at the goal, so every search from the plate floods it first. Preconditions asserted: capped `astar()` gives `None`, the world-budget search finds the path. 3.4-4.8 s debug.
  - **Mutation run** (`a57ddeff`, alone, `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …/13-7.sh`): rows 1-5 and 7-9 KILLED, each at its own assert (line numbers in the sabotaged file): 1 the AC 1 claim panic (132 s); 2 and 3 the first stamps assert; 3b the tick-120 assert; 4 `miner 0 must claim the far dig once and keep it through the churn` (`left: (8, 8)`); 5 `the 25 pocket digs are queued`; 7 `the joined plates (… cells) must exceed the node budget`; 8 and 9 `miner 1 claims the dig miner 0 cannot reach`. **Row 6 SURVIVED as first written** (an unreachable dig claimed with an empty path): the same tick's re-plan fails and `retry_claim` releases the wrong claim before `claims()` can see it. Rewritten to also keep the claim on a failed re-plan; re-run alone from a scratch table: **KILLED** at `the claim must be the late, reachable dig` (122 s). 10/10 KILLED. `audit-mutations.py`: 979 rows, exit 0.
  - **Bench re-read** (`RUNS=5 scripts/tick-bench.sh 35c6d6f4 a57ddeff` over all 11 files, so the delta is the patch pass alone; load 3.67 before, 1.48 after): `same` on all 11 rows, two-hundred-on-256 included. The re-plans that used to give up were all unreachable (creation B), so the world budget returns the same `None` and the outcome does not move. **two-hundred-on-256 +3.9% mean, BEYOND FLOOR** (floors 0.6 / 1.9 / 2.4%, control +1.8%: about 2x the control, weak evidence between two builds by tick-bench's own NOTE). The other ten are within floor (-1.8% to +0.8%). Separate `--runs 3` per build, sequential: mean 33.72 / 33.94 / 34.07 -> 34.84 / 34.78 / 34.82 ms (+2.6%), p99 median 156.8 -> 161.8 (+3.2%), max 299-304 -> 362-366 (+21%); end line identical (`done=dig:234,cut:90 stored=27 cells=2146`), state `6ba8edcc3d6c7b9c` both. The review inferred under 1%; measured it is 2.6-3.9% mean and a +21% max spike. That is the cost of each unreachable re-plan flooding its whole component (64,300 cells) instead of stopping at 50,000.
  - **Closure table** (both-sides; no row re-closes a prior round's fix, so none is REWORK):

    | Item | Written for | Tested from | Pre-existing-state fixture |
    |---|---|---|---|
    | P0 re-plan world budget | a far reachable claim whose re-plan floods past 50,000 under churn | the OLD code: the new test on `b74f1242` (pre-fix) reads `claims=8 releases=8 by tick 201`; then row 4 (`astar()` back) KILLED at the churn assert | `a_far_claim_is_kept_through_terrain_churn_past_the_node_cap`'s 256² plate + corridor, both preconditions asserted |
    | P1 audit unreadable rows | a row whose anchors sit in a tuple list | the OLD table: the new audit over `13-7.sh` as committed in `35c6d6f4` exits 1 listing rows 1-2; the reviewer's broken anchor (`let mut assigned: bool = false;`) on a copy exits 1 (`old_assigned asserts 1 match(es) … source has 0`); HEAD exits 0 | `13-7.sh` at `35c6d6f4` (rows 1-2 in `edits = [...]` form) |
    | P2 AC 1 queue + claim id | a designate filter that drops the pocket digs; a wrong claim | row 5 (`create_jobs` drops z = 1 targets) KILLED at the 25-queued assert; row 6 (unreachable claimed and kept) KILLED at the id assert | AC 1's 256² DEFAULT_SEED valley with 25 sealed pockets |
    | P3 AC 3 area precondition | `joined_plates` shrinking under the budget | row 7 (`0..110` -> `0..90` in `joined_plates`) KILLED at the area assert | `joined_plates` |
    | P4 rows 8-9 | the stamp-and-go-on seam left unpinned by the 12-4 retirements | the unchanged miner-zero test, against Edge's two mutants | `a_job_miner_zero_cannot_reach_…`'s joined plates + pockets |
    | P5 record | the mismatched -5.8% ratio | recomputed from the six means: -4.85%; mismatched ratio reproduces -5.79% | the AC 5 Debug Log means |

  - **Full gate GREEN on `da3a502c`** (`RUST_TEST_THREADS=1 scripts/gate.sh`, exit 0), covering every patch-pass commit:

    ```
    frostvein gate
      cargo fmt --check           ok  0s
      cargo clippy -D warnings    ok  3s
      cargo test                  ok  1186s
      cargo test (release real-work scenario)ok  99s
      cargo test (pixel guards)   ok  3069s
      tui has no sim-core edge                ok
      client-core has no sim-core edge        ok
      gui has no sim-core edge                ok
      metrics ledger tests        ok  0s
      bench tests                 ok  19s
      mutation tables still apply ok  11s
    GATE GREEN  4395s
    ```

- **Review patch pass 2 (2026-11-03, fresh session, Opus 5.5).** Commits: `02eb29a7` audit reads concatenated anchors + rows re-pointed (P0+P1 together: the audit alone turns the gate RED), `f21c799b` mixed-row check (P2), `56229c37` row 6 rewrite (P3), then the record commit (P4).
  - **Correction to the finding (P0):** it counted 9 concatenated rows, but `12-12` row 18's concatenation (`own_box = (...)`) is replacement text, and its anchors sit in a `for old, new in [...]` loop. It cannot read clean, so by the ruling ("drop … once each reads clean") it stays allow-listed. **8 rows leave the list and 13 stay (5 loops, 1 regex, 7 undelimitable).**
  - **P0 RED (allow-list untouched):** the new audit names exactly `12-10b` row 1 and `12-12` row 16, `old asserts 1 match(es) … source has 0`, exit 1. After the re-point it exits 0 (979 rows). On a copy, one anchor broken in each of the 8 rows that left the list: all 8 named, exit 1.
  - **P1 re-points:** row 1 → `let goals: BTreeSet<_> = astar_neighbours(…)…collect();` (the Channel arm, `lib.rs:1394`), with the same `side_neighbours` + `is_walkable` sabotage. Row 16 → `kept.open.clear();\n kept.memo = None;\n` with `kept.memo = None;\n` kept, so only the floods' `open` answers survive a tile change, as the row's name says.
  - **P2:** `STRINGS` blanks every string literal, so Rust inside an anchor (`3-1`'s `for (x, glyph) in (` in its `old`) cannot match. Then `ANCHOR_LOOP` (`for … in (`/`[`, `name = [(`) flags a row that is already read. At HEAD only 5 rows have that shape, all allow-listed, so the audit exits 0. The reviewer's probe row (a guarded `old` + `edits = [('THIS ANCHOR IS NOT IN THE SOURCE', 'x', 1)]` and its loop) on a copy: the pre-P2 audit prints `980 rows, every literal still matches` (exit 0); the new audit names it (exit 1).
  - **Mutation run** (`56229c37`, a scratch table holding just these three rows, `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …`): **3/3 KILLED**, each at its own assert. `12-10b` row 1: `staircase worked for 0 of 24 designation delays` (`tests/scenario.rs:4850`, 86 s). `12-12` row 16: `the dug spur opens the hole` (`lib.rs:8116` in the sabotaged file). `13-7` row 6: `the claim must be the late, reachable dig` (`lib.rs:5567`, 30 s).
  - **Row 6 without the id assert** (scratch worktree off `56229c37`, the assert replaced by `let _ = (job, late);`, row 6 applied): **passes**, `claimed at tick 101, dug at tick 208`. Miner 0's wrong claim is held before the designation, and miners 1-4 dig the late dig on time. So the id assert is the only assert that kills row 6.
  - **Closure table** (both sides):

    | Item | Written for | Tested from | Pre-existing-state fixture |
    |---|---|---|---|
    | P0 audit reads `('…' '…')` | rot in the 8 concatenated rows the allow-list hid | the OLD rows: the new audit over the unchanged tables (`448c7c3a`) names exactly the 2 dead rows; each of the 8 broken on a copy is caught; HEAD exits 0 | `12-10b.sh` row 1 and `12-12.sh` row 16 as committed at `448c7c3a` (dead since before `483b95b1`) |
    | P1 re-point 2 dead rows | rows whose anchors matched 0 times | the audit RED on each old anchor (above), then each re-pointed row run alone: both KILLED at their own test | the same two rows at `448c7c3a` |
    | P2 mixed rows | a readable row hiding a broken tuple-list anchor | the OLD audit (`02eb29a7`) on the reviewer's probe row: blind (exit 0); new audit: RED; HEAD: clean | the run 2 reviewer's probe row (guarded `old` + `edits = [('THIS ANCHOR IS NOT IN THE SOURCE', 'x', 1)]`) |
    | P3 row 6 (**REWORK** of run 1 P2) | a wrong claim the id assert alone must catch | row 6 KILLED at the id assert; with the id assert stubbed, the same sabotage PASSES (the late dig is dug) | AC 1's 256² DEFAULT_SEED valley with 25 sealed pockets |
    | P4 record | the pre-amendment text and the stale AC 5/AC 6 figures | read against AC 4 as amended and patch pass 1's bench re-read: max +30-47% recomputed from 249-279 → 362-366 | this file's lines 25 and 72, Debug Log AC 5/AC 6, File List; `deferred-work.md`'s run 1 entry |
  - **Full gate GREEN on `af3c19bf`** (`RUST_TEST_THREADS=1 scripts/gate.sh`, exit 0), covering every patch pass 2 commit:

    ```
    frostvein gate
      cargo fmt --check           ok  0s
      cargo clippy -D warnings    ok  2s
      cargo test                  ok  1173s
      cargo test (release real-work scenario)ok  100s
      cargo test (pixel guards)   ok  3293s
      tui has no sim-core edge                ok
      client-core has no sim-core edge        ok
      gui has no sim-core edge                ok
      metrics ledger tests        ok  0s
      bench tests                 ok  20s
      mutation tables still apply ok  11s
    GATE GREEN  4607s
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
- `_bmad-output/planning-artifacts/epics.md` (UPDATE: Story 13.7 ACs 1-2 AMENDED, Task 0 Q3 a)
- `_bmad-output/implementation-artifacts/metrics/13-7-the-long-way.md`, `metrics/.session-cursors.json` (cost rows)
- `_bmad-output/implementation-artifacts/13-7-signoff/creation-probes.txt`, `issue-159-comment.md`, `prototypes/{probe159-counters,probe-the-long-way,budget-all-sites}.patch` (NEW at creation)
- `_bmad-output/implementation-artifacts/deferred-work.md` (13.7 review runs 1 and 2 deferrals)
- Review patch pass 1: `scripts/audit-mutations.py` (UPDATE: fails a row with no readable anchor; 21 older rows listed), `crates/sim-core/src/lib.rs` (the `execute_jobs` re-plan's budget, NOTE, one new test, asserts in AC 1 and AC 3's tests), `mutations/13-7.sh` (rows 1-2 rewritten, rows 4-9 NEW)
- Review patch pass 2: `scripts/audit-mutations.py` (UPDATE: reads `old = ('...' '...')`; 8 rows leave the allow-list, 13 stay; flags a readable row whose anchors also sit in a loop or tuple list), `mutations/12-10b.sh` row 1 and `mutations/12-12.sh` row 16 (re-pointed), `mutations/13-7.sh` row 6 (rewritten)

## Change Log

| Date | Change |
|---|---|
| 2026-11-02 | Story created (Opus 5.5). Creation probes: every 256² budget exhaustion is an unreachable goal (13.2's reading corrected); the red reproduces on the real valley; claim-only resize is cost-neutral. Evidence in `13-7-signoff/`. Task 0 Q1–Q3 await Wolf. |
| 2026-11-02 | Task 0 RULED (Wolf, "ok for all"): Q1 (a) claim-only resize; Q2 (a) per-search world-sized budget, 12.4 sat-out removed; Q3 (a) epic 13.7 ACs 1-2 amended, `issue-159-comment.md` posted on #159. ACs and Tasks stand as written. Next: ask which implementer. |
| 2026-11-02 | Dev started: Wolf picked Sonnet 5.5 subagents (Opus orchestrates). Status in-progress. |
| 2026-11-02 | Dev complete (Sonnet 5.5 agent for Tasks 1-2, Opus for Tasks 3-5): `641dbc7b` claim budget sized to the world + tests + mutation retirements, `92b5e326` record. AC 1 RED/GREEN, 13-7.sh 4/4 KILLED, tick-bench 10/10 same (256² differ, explained), A* 47.1% of the 256² tick, flat storage slower. Full gate GREEN on `92b5e326` (4,424 s). Status review. |
| 2026-11-02 | Code review run 1 (fresh session, 4 layers, none failed): 0 HIGH. Decision 1, Wolf "1": the `execute_jobs` re-plan gets the world budget, which amends Q1 (a) and AC 4 at that one site. 6 patches left as action items (Wolf "2"), 4 deferred, 4 dismissed. Status in-progress. |
| 2026-11-02 | Review patch pass 1 (fresh session, Opus 5.5): 6/6 patches landed (`b74f1242`, `faad260a`, `f5b106b4`, `a57ddeff`, record). AC 4 AMENDED (re-plan world budget, decision 1). 13-7.sh 10/10 KILLED (row 6 rewritten after it SURVIVED). tick-bench 11/11 same; two-hundred-on-256 +3.9% mean BEYOND FLOOR, +21% max. Full gate GREEN on `da3a502c` (4,395 s). Cost row $14.12. Status review; next: round-2 code review. |
| 2026-11-03 | Code review run 2 (fresh session, Opus 5.5) on `35c6d6f4..b9f7209b`: 4 layers OK; NEW 4 (0 HIGH/1 MED/3 LOW), REWORK 1 (row 6). Stopping rule: no round 3. Decision 1 (Wolf "a"): audit reads parenthesised concatenations. 5 patches left as action items (Wolf "2"); 3 deferred, 4 dismissed. Cost $15.03. Status in-progress. |
| 2026-11-03 | Review patch pass 2 (fresh session, Opus 5.5): 5/5 patches landed (`02eb29a7`, `f21c799b`, `56229c37`, `af3c19bf`). The audit reads concatenated anchors (8 rows leave the allow-list, 13 stay) and flags mixed rows. `12-10b` row 1 and `12-12` row 16 re-pointed, both KILLED. 13-7 row 6 rewritten, KILLED at the id assert and passing with it stubbed. Full gate GREEN on `af3c19bf` (4,607 s). Cost row $8.12. Status review; run 2 ruled no round 3, so next: PR on Wolf's yes. |
| 2026-11-03 | Merged as PR #229 (`f9e33e34`, head `95bbc922`), closing #159 only (`closingIssuesReferences` [159]). Status → done. |
