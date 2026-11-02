---
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
baseline_commit: 483b95b1
---

# Story 13.7: The Long Way

Status: ready-for-dev

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
4. **The other searches keep their budget (Q1 a).** `astar()`, `route_to_nearest` and `stranded` still use `MAX_ASTAR_NODES` = 50,000, and every pin on it passes unchanged (`astar_stops_at_the_node_cap`, `astar_finds_a_path_well_inside_the_node_cap`).
5. **Bench delta (standing AC 10/11).** `RUNS=5 scripts/tick-bench.sh main HEAD` over all 11 scenarios reads `same` state on the 10 128² rows. two-hundred-on-256 reads `differ`, and the record states why: the claim searches complete instead of exhausting. The record also gives its mean and p99 delta against the floor; creation measured flat (D).
6. **The A\* storage profile is recorded (epic AC 4).** At HEAD on two-hundred-on-256, the record gives A\*'s share of the tick, and the mean and p99 of a throwaway flat-`Vec` storage A/B with its `state=`. No storage change is committed.
7. **The mutation table kills** (Task 4), and `python3 scripts/audit-mutations.py` reports no BROKEN row.

## Tasks / Subtasks

- [ ] **Task 0: Wolf's rulings (Q1–Q3).** Record the answers in the Change Log and amend the ACs they name. Under Q3 (a), amend `epics.md` Story 13.7 ACs 1–2 with an `AMENDED 2026-11-02` line, and post `13-7-signoff/issue-159-comment.md` on #159 if Wolf says yes. Then ASK which implementer to use (memory: `dev-implementer-is-sonnet-not-codex`).
- [ ] **Task 1: the claim budget (AC 2, 4).** UPDATE `crates/sim-core/src/lib.rs` `claim_jobs` (`:559-703`).
  - [ ] Delete `budgets` (`:591-592`), `sat_out` (`:623-624`, `:644-647`, `:697`) and the `enumerate`/`slot` that only indexed the budgets. Before each `astar_with_budget` call (`:658`, `:671`), set `let mut budget = terrain.tiles.len();` and pass `&mut budget`. The stamp condition becomes `attempted && !assigned`.
  - [ ] Both `(None, true, _)` arms (`:665`, `:683`) become one `(_, true, _) => unreachable!("a search sized to the world cannot run out")` per match. The `(Some(_), true, _)` arms fold into it.
  - [ ] Rewrite the comments that are now false: `:559-561` ("spends one node budget per dwarf"), and the NOTE at `:595-598`. The new NOTE says: a search expands each cell at most once (consistent heuristic), so the world's cell count cannot run out; an unreachable job costs one flood of its component per call, cached in `components`.
  - [ ] `MAX_ASTAR_NODES` NOTE (`:52-53`): it now bounds `astar()`, `route_to_nearest` and `stranded` only. 13.7 found every one of their 256² exhaustions to be an unreachable goal, so capping gives the same `None`, cheaper. A reachable reroute or re-plan past it would give up (`resolve_blocked_step` `:1887`, `execute_jobs` `:2263` → `retry_claim`), which no probe has seen. Do not change the value (AC 4).
- [ ] **Task 2: the tests (AC 1, 3, 4).** In `crates/sim-core/src/lib.rs` `mod tests`. Port the setup from `13-7-signoff/prototypes/probe-the-long-way.patch` (`probe_the_long_way_stepped`), dropping its `println!`s.
  - [ ] NEW `a_reachable_dig_behind_unreachable_ones_is_dug_on_the_256_valley` (AC 1). The *Found at creation* E/F setup: `World::generate_with_crew(DEFAULT_SEED, Dims::FRESH, 5)`, `stand_miners_at` (`:5412`) on the dwarves' own spots, the 25 digs designated through `SimCommand::Designate`, `world.step()` × 100, then the reachable dig designated. Preconditions, each asserted before the steps:
    - every cell around each pocket is `Solid`;
    - the camp component (a BFS over `astar_neighbours` from dwarf 0) holds more than `MAX_ASTAR_NODES` cells and holds no pocket.

    Without these, the test passes on `main` for the wrong reason. Assert a claim within 60 ticks of the late designation, then the tile reads `Tile::Empty` within 300 ticks. Debug cost at creation: 21 s, which fits the fast tier.
  - [ ] REWRITE `a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on` (`:5527-5597`) as AC 3. Keep the `joined_plates` fixture: its 55,003 cells are over 50,000, so `main` exhausts there. Name it for what it pins. At tick 100: stamps `[120 ×10, 0]` and miner 0 claims dig 10. Then run the schedule at ticks 101–120: at tick 120 the 10 read `140` and `world.jobs().len()` is unchanged.
  - [ ] `a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp` (`:5599-5680`): delete its exhaustion precondition (`:5641-5652`) and rename it. Rewrite its comment to say what it pins now: miner 0's flood is cached, the trade filter comes before `attempted`, and the next miner claims. Keep its claim and stamp asserts, which must still pass. `a_reachable_job_behind_unreachable_ones_is_claimed_when_areas_sum_past_the_budget` (`:5438-5499`) passes unchanged. Fix only comment text that is now false (it mentions `MAX_ASTAR_NODES`).
  - [ ] The two-hundred bench test's doc comment (`crates/simd/tests/bench.rs:266-269`): add 13.7's reading, with its reason, from Task 3 (standing AC 11). Its `≥ 1` asserts stand.
- [ ] **Task 3: the instrument, RED first (AC 1, 5).** The instrument is AC 1's test run with `--nocapture`. It prints one line, `claimed at tick T, dug at tick U` (or `never claimed by tick T`), which is what a human reads. The bench side is `simd --bench`'s end line and `state=` (`crates/simd/src/scenario.rs:410-420`), driven by `scripts/tick-bench.sh`. Both are tested by `crates/simd/tests/bench.rs` and `mutations/13-1.sh`.
  - [ ] RED: apply row 1 by hand on a copy (scratch worktree, memory: `mutation-run-poisons-concurrent-builds`), run the test with `--nocapture`, and paste the failing line: no claim within 60 ticks. Then GREEN on HEAD: paste the line. Creation read t115 and t208 for designation at t100.
  - [ ] AC 5: on an idle devpod, run `RUNS=5 scripts/tick-bench.sh main HEAD` over all 11 `scenarios/*.json` (`main`, at 13.6, reads the 256² file). Paste the table. Name the reason for the two-hundred `differ`, its end line on both sides, and the mean and p99 delta against the floor.
  - [ ] AC 6, the profile, using throwaway patches committed only under `13-7-signoff/prototypes/`:
    - `astar-timer.patch`: an `Instant` accumulator inside `astar_with_budget` and `route_to_nearest`, plus the tick total, printed at the bench's end. This gives A\*'s share of the tick.
    - `astar-flat-storage.patch`: `costs` and `came_from` as `Vec`s indexed by `worldgen::index` over the terrain. This gives the mean and p99 of `--runs 3` against HEAD's, and its `state=`, which must equal HEAD's.

    Record both readings in the Debug Log. Changing the storage is a separate story, on Wolf's ruling.
- [ ] **Task 4: mutation table (AC 7).** NEW `_bmad-output/implementation-artifacts/mutations/13-7.sh`, in `13-6.sh`'s shape. Commit first, then run it alone (`env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh …`). Each row must be KILLED at its own assert. Read the panic line (memory: `mutation-test-name-prefix-trap`).
  1. the claim budget is `MAX_ASTAR_NODES` again → AC 1's test, at the claim assert (a heavy row: `main`'s budget costs 5 × 50,000 nodes a tick, so allow about 4 min in debug);
  2. the same sabotage → AC 3's rewrite, at the first stamps assert;
  3. the cooldown stamp becomes `u64::MAX` (dropped, never retried) → AC 3's rewrite, at the tick-120 assert.

  Retire the rows that sabotage removed code, each with a dated `# Retired` comment at the top of its file (`5-1-the-world-grows-things-that-glow.sh:4` is the shape): `12-4.sh` rows 9–16 (`:76-162`), and `3-2-the-dig.sh`'s "each claim search gets a fresh node budget" (`:954-971`, whose sabotage is now the code). Row 16 ("budget raised past the joined plates", `:156`) still applies, because its anchor is the constant, but it targets the deleted precondition's test. Run it once: if it SURVIVES, retire it. Re-point a row instead of retiring it only if its anchor survives and it still kills. `audit-mutations.py` must report no BROKEN row.
- [ ] **Task 5: gate and close.** Fast tier on every commit (pre-commit hook). ONE full gate at PR: `RUST_TEST_THREADS=1 scripts/gate.sh` (~70 min, memory: `gate-ooms-at-default-parallelism`). Paste the block. Cost row: `python3 _bmad/scripts/session_tokens.py --phase dev --story 13-7-the-long-way`, then `phase_state.py clear`, then Status → review on the story and the board. The PR body may say `Closes #159` and nothing else closing; check `closingIssuesReferences` (memory: `closing-keyword-closes-whole-issue`).

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

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-11-02 | Story created (Opus 5.5). Creation probes: every 256² budget exhaustion is an unreachable goal (13.2's reading corrected); the red reproduces on the real valley; claim-only resize is cost-neutral. Evidence in `13-7-signoff/`. Task 0 Q1–Q3 await Wolf. |
| 2026-11-02 | Task 0 RULED (Wolf, "ok for all"): Q1 (a) claim-only resize; Q2 (a) per-search world-sized budget, 12.4 sat-out removed; Q3 (a) epic 13.7 ACs 1-2 amended, `issue-159-comment.md` posted on #159. ACs and Tasks stand as written. Next: ask which implementer. |
