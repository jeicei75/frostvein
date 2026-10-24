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
- [x] **Task 1: the census instrument (AC 3, 4).** NEW `13-2-signoff/census.sh PATCH...`:
  - It makes a scratch worktree at `4f57027` and applies the patches in order. It builds a release `simd` there; `cd` into the worktree, because `CARGO_TARGET_DIR` builds the cwd's sources.
  - It prints every reproducer's `end` line (the 8 `scenarios/*.json` and the two issue saves at 10,000 ticks) and the cage-sweep lists.
  - `census.patch` turns the cage sweep's three `assert_eq!`/`assert!` on `landed`/`frozen`/`unfinished` (`tests/scenario.rs:3109-3125`) and the staircase `worked` asserts into `eprintln!`. Today the first failing assert hides the rest, and a prototype is expected to change the pins.
  - Run the sweeps with `cargo test --release -p sim-core --test scenario <name> -- --nocapture`.
  - Add (Q3) `scenarios/stuck-tunnel.json` (#206) and `scenarios/dig-loop.json` (#203), each starting from its save with 10,000 ticks and no commands, and say in `about` which issue the file reproduces.
  - **RED, then control.** `census.sh` with no prototype must reproduce the creation table above. `census.sh ghost.patch` (commit the probe as `prototypes/ghost.patch`) must read seat-loop `marks=0` and plan `marks=4`. A census that cannot tell these two apart is broken.
- [x] **Task 2: the survey (AC 1).** Web sources only, with each claim linked. One table plus at most a paragraph per lead. Check DFHack's walkable-group id in its source or docs, not from memory.
- [x] **Task 3: the profile (AC 2).**
  - NEW `prototypes/counters.patch` adds plain counters (a resource or statics, read at the bench's end line or printed by a test). It counts A\* calls and nodes per caller (`claim_jobs` `:633`/`:646`, `execute_jobs` replan `:2113`, `resolve_blocked_step` `:1751`, `clearing_set_down` `:1927`), `route_to_nearest` calls and cells, heuristic goal-checks, and cells visited by `stranded` (`:1015`), `caging_stone` (`:1832`), `nearest_free_cell` (`:1436`, which has no cap) and the `pile_targets` floods (`:851`).
  - Counters only, with no behaviour change: every reproducer's state hash must equal base.
  - RED: on top of counters, make `astar_heuristic` iterate its goals twice (13.1's sabotage). Plan's goal-check count must double, and nodes must not move.
  - Use tick-bench for the ms column (`RUNS=5`, never beside the gate or `mutate.sh`). Judge between builds by counts. A ms delta under about 6% between two SHAs is weak evidence.
- [x] **Task 4: scale (AC 6).** Crews of 5/20/50/100/200 on 128×128×32 and 256×256×32 (Q1). NEW `prototypes/scale.patch` makes the crew size and the dims scenario-settable (throwaway). **Trap: a bigger map costs only if the work uses it.** Put the 256² scenarios' marks and piles far enough apart that paths and floods span the map, or the extra area is idle ground and reads as free. **On 256², #159 may become live:** DEFAULT_SEED's walkable area is about 16k cells at 128², so 4× the area can pass the 50k `MAX_ASTAR_NODES` cap. Measure that component size, and fill #159's cell from it. **Trap: spawn runs out of cells.** `spawn_dwarves` (`lib.rs:3447`) draws from the standable cells within `CAMP_RADIUS = 3` of the camp (`worldgen.rs:7`), which is at most 49 cells, and `random_range(0..0)` panics once they are used up. So 50+ dwarves need a wider spawn ring. The identity and profession arrays are fixed at 5 too. Scenario files go in `13-2-signoff/scale/`, not `scenarios/`. Each scale point needs real work for the crew (marks plus a pile), or it measures idle dwarves. Run base plus each option patch on top of it.
- [x] **Task 5: connectivity map (AC 3, 4).** `prototypes/connectivity.patch`. It keeps reachability between ticks and answers "can he get there" before any A\* or flood, so #159's starvation and the per-question floods go.
  - **Trap: items move on every haul.** If the map includes items, it is invalidated constantly. If it covers terrain only, A\* still decides around items. Measure the invalidation rate and say which way it went and why.
  - Invalidate on the same inputs as `pile_ground` (`set_tile` at `lib.rs:742`, a zones change at `:868`).
  - Also measure whether the map lets the four wall-in guards (`walls_in_a_dwarf` `:959`, `walls_in_a_dwarf_from` `:1002`, `closes_last_level_exit` `:974`, `caging_stone`) become one question ("does every dwarf still reach work?"). A guard that answers a different question once it is cheaper is a different rule: compare outcomes as well as ms.
- [x] **Task 6: squeeze-past (AC 3, 4, 5).** `prototypes/squeeze.patch`. A timed swap of two dwarves in a one-wide way, where the gui would sell it with an animation (out of scope here).
  - `resolve_blocked_step` never swaps today (`lib.rs:1685`). `step_and_watch` (`tests/scenario.rs:4746`) panics on a swap, and changing that check IS the rule change: list it under Rulings needed.
  - List every writer of a dwarf's `Pos` (`execute_jobs`, `settle`, `wander`; `dwarf_tiles` doc, `lib.rs:1195`) and keep each one's `occupied` current. Then check that no tile holds two dwarves at a tick boundary. `step_and_watch` covers only the one-wide staircase, so `counters.patch` also counts tick boundaries where two dwarves share a tile, on every reproducer. It must read 0 on base and on every prototype (AC 5's emitter).
  - The ghost probe is not this prototype's ceiling; see Found at creation.
- [x] **Task 7: clamber-over (AC 3, 4, 5).** `prototypes/clamber.patch`. A costed step over a blocking stone.
  - A\* is unit-cost today, so this needs a weighted step, and the heuristic must stay admissible. Emitters share `blocked` with stones (`blocked_cells` `:795`, `world_blocked` `:800`) and must stay blocking.
  - The item-placing writers (haul drop `:2258`, dig spawn `:2337`, channel landing `:2395`, `settle` `:2493`) must still never place a stone on a dwarf. The sweep's `landed` list must stay empty.
- [x] **Task 8: the proposal (AC 1–7).** NEW `13-2-signoff/proposal.md` with § Survey, Profile, Scale, Prototypes, Issues, Rulings needed and Recommendation. Every number names its patch stack and command. State ADR 5 ("plain A\* for M1", `docs/technical-preferences.md`) and what the recommendation does to it.
- [x] **Task 9: Wolf rules, then close.**
  - Record the ruling in the Change Log and in the proposal's last section. Note any issue it re-scopes on that issue, with no closing keyword; nothing here closes an issue.
  - Then the ONE full gate (`RUST_TEST_THREADS=1 scripts/gate.sh`) and paste the block. Then the cost row (`session_tokens.py --phase dev --story 13-2-the-way-through-pathfinding-and-reachability-spike`) and `phase_state.py clear`.

### Review Findings

Code review run 1 (2026-10-24, a fresh session). Four layers ran on `4f57027..ccc1da07`. Each ran `cargo 1.97.1` in its own `CARGO_TARGET_DIR` on trees extracted with `git archive`. No layer timed out or failed. Nothing under `/workspace` was modified, and `git worktree list` is clean afterwards. Every recorded patch hash matches the committed patch.

**What was re-run and matched:**
- All 18 patch stacks apply to `4f57027`.
- `census.sh` (base) matches `census-base.txt` except the header line.
- `census.sh counters squeeze` is byte-identical to `census-squeeze.txt`.
- **pile-memo at 128² crew 200:** 338.0 → 20.94 ms run back to back (feature), and 303.7 → 24.25 ms under shared load (acceptance). Counts are equal and `state=e0b2df85` matches base.
- **Timer shares:** plan-compact 90% (committed 91%), plan-far 72% (73%).
- **#187 at 16k ticks:** squeeze `[(0,4,1),(6,6,1),(9,6,1),(13,6,1)]`, C0 `[]`. Reproduced by two layers.
- The pile-memo RED rebuilt from prose: plan 12/10/1 → 12/5/1, big-pile 14 → 15.
- clamber cost 4 and cost 8, the counters RED (2x goal-checks, nodes unchanged) and both new scenarios (marks 3 and 11).
- **Totals:** all 200 § Scale cells and every § Profile total.
- **Fixtures:** all 10 regenerate byte-identical.

**AC 7 holds:** the `crates/` diff is empty, and the ruling commit precedes Status review.

**Totals:** no HIGH findings; 2 decision (both resolved into patches), 8 patch (left as action items, Wolf 2026-10-24), 13 defer, 1 dismissed. Review cost $23.83 / 405 turns (4 subagents = 82.9%). `reap-build-caches.sh --tmp-only --force` reaped 23 dirs (35.5 GB; 30.8 GB reclaimed). Convergence (layers that found the same thing independently) appears in 7 findings.

- [x] [Review][Decision] **MED, acceptance (AC 5).** The idle-blocker swap is missing from § Rulings needed, so Ruling 1 never saw it. `proto-squeeze.md` § Rule departures item 5 says an idle blocker with no way out also swaps after the wait, and its `home` moves to the holder's old tile. The proposal's Rulings needed item 1 names only the timed swap and the head-on swap, and Wolf ruled "the swap is allowed (variant B)" on that text. Options: (a) Ruling 1 covers it; record that in § Ruling. (b) 13.4 asks it as its own Task 0 question. **Wolf (2026-10-24): (b).** It becomes the patch item below.
- [x] [Review][Decision] **LOW-MED, feature.** 13.3, 13.4 and 13.5 exist only in prose. `epics.md` Epic 13 lists only 13.1 and 13.2, and `sprint-status.yaml` has no `13-3`/`13-4`/`13-5` keys, so `bmad-create-story` has no "next" to pick ([[board-order-contradicts-ruling]]). Options: (a) add the three epic entries and backlog keys on this branch, in the ruled order. (b) Leave it to `bmad-create-story 13.3`. **Wolf (2026-10-24): (a).** It becomes the patch item below.
- [x] [Review][Patch] **From decision 1.** Record that Ruling 1 did not cover the idle-blocker swap (its `home` moves) [`13-2-signoff/proposal.md` § Ruling], and carry it as a Task 0 question in 13.4's epic entry.
- [x] [Review][Patch] **From decision 2.** Add 13.3 pile-memo, 13.4 squeeze-past and 13.5 clamber-over to `planning-artifacts/epics.md` Epic 13, and add backlog keys to `sprint-status.yaml`, in the ruled order [`planning-artifacts/epics.md`, `sprint-status.yaml`].
- [x] [Review][Patch] **MED, acceptance + feature.** The pile-memo expected-cost line Wolf ruled on miscounts [`13-2-signoff/proposal.md:348-350`, story Debug Log]. `tickbench.txt` §6 has **six** scenarios at −55…−91% and **four** others (deep-pit −10.6, dig-loop +4.3, full-pile-crowd +3.2, stuck-tunnel −1.9), not seven and three. dig-loop and full-pile-crowd are BEYOND FLOOR (controls +1.4/−0.1), so "inside the noise" should read "weak evidence by the 6% rule, beyond the bench floor". A 13.3 target of "−55% on seven" cannot be met.
- [x] [Review][Patch] **LOW-MED, feature (same paragraph).** "A 200-dwarf crew at a fifth of the 100 ms budget" quotes the mean only [`proposal.md:350-351`]. pile-memo's p99 is 97.1 ms and its max is 134 ms (feature re-run: 95.6 / 129.6), and 1,030 of 2,000 ticks run over 10 ms. Fix: add the tail beside the mean.
- [x] [Review][Patch] **MED, feature + acceptance.** The #187 16k-tick reading that gates 13.4 has no committed producer [`proposal.md:273,307,323,356`; `proto-squeeze.md:153`]. It came from a scratch `TICKS = 16_000` edit, and `census.sh` runs 4,000 ticks. The number is right (reproduced by two layers). Fix: commit `prototypes/census-16k.patch` and name it wherever the reading is quoted.
- [x] [Review][Patch] **MED, feature + acceptance + blind.** The pile-memo RED is not committed, and it does not test half the key [`proposal.md:242-243`]. The "key ignores `blocked`" RED exists only as prose. Blind built a `near` = zone-cells-only RED, which stays hash-equal on all 8 scenarios, so no census guards the side-neighbour ring the key depends on. Fix: commit both REDs as patches with their readings, and say in Recommendation item 1 that 13.3's test must put the blocked cell on a non-zone side neighbour and kill the `near` RED.
- [x] [Review][Patch] **LOW (mislabelled instrument), feature + acceptance.** The timer tables' caption gives the wrong denominator, and "every 128² scale point" overclaims [`proposal.md:93,100,337`]. Shares reproduce only with systems + `tick_boundary` as the denominator. At 256² crew 200 after the memo, A\* is 40% published against 54% of the system time, which understates "A\* is next at 256²". Also, timer covers only 128² crews 50/100/200, not 5 or 20.
- [x] [Review][Patch] **LOW (latent silent-failure trap), edge + feature.** The instruments can print a blind reading that looks real. In `census.sh` [`13-2-signoff/census.sh:47-55`], an empty counters line skips the IDENTICAL/DIFFER check and prints nothing. In `scale.sh` [`13-2-signoff/scale.sh:42-47`], an absent key sums to a 0 that reads as clean, and the run's `state=` hash is stripped, so "hash equal to base at scale" is printed by no committed script. Fix: `counters NAME MISSING` in census; in scale, print run 1's `state=` and flag `boundaries=0`.
**Patch pass 1 (2026-10-24, fresh session): 8/8 landed** (`739adfac` instruments, `a3379cbb` pile-memo REDs, `7374218e` 16k producer, `9abe8b6c` board, `b3a4bf74` proposal text). Closure table. No row is REWORK, because this is the first patch pass. No cargo test changed (nothing under `crates/`). The instrument checks' mutations are the two committed sabotage patches, and `instrument-red.txt` records them.

| item | fix written for | then tested | pre-existing-state fixture |
|---|---|---|---|
| 1 idle-blocker swap | the record: § Ruling says Ruling 1 did not cover it, and 13.4's epic entry asks it as Task 0 | the old text: the quoted rule matches its source | `proto-squeeze.md` § Rule departures item 5 against proposal § Rulings needed item 1, both as of `28162c85` |
| 2 epic entries + keys | `bmad-create-story` finding a next story | YAML parses, and the key order is `13-2` → `13-3` → `13-4` → `13-5` → retro (`uv run --with pyyaml`) | `sprint-status.yaml` and `epics.md` at `28162c85`, which have no 13.3–13.5 |
| 3 recount | the expected-cost line | each number re-read from the raw rows: 6 at −55.5…−91.0%, 4 others, controls +1.4% / −0.1% | `tickbench.txt` § "C0 vs pile-memo" (committed at dev) |
| 4 tail | the mean-only claim | p99 and max read from the committed run, and `over10` read from the new `scale.sh` | `scale-r1-pile-memo.txt` (p99 97.1, max 134); `scale-hash-pile-memo.txt` (over10 1,081 under load; the review re-run read 1,030) |
| 5 16k producer | a committed producer | both stacks re-run through it, and they reproduce the scratch reading exactly (squeeze `[(0,4,1),(6,6,1),(9,6,1),(13,6,1)]`, C0 `[]`) | the scratch-run row in `proto-squeeze.md:72` (written at dev, before the patch existed) |
| 6 pile-memo REDs | committed REDs, plus 13.3's test requirement | a full census of each RED. The key RED reproduces the dev prose (plan 12/10/1 → 12/5/1, big-pile 14 → 15) and moves 8 of 10 hashes, not "all 4 tried". **The near RED corrects the finding:** 10/10 scenario hashes stay equal, but the cage sweep's `frozen` moves, so one sweep line does see it | the dev Debug Log's prose RED; `census-pile-memo.txt` (committed at dev) as the equal side |
| 7 timer denominator | the caption and the overclaim | every published share recomputed from the raw ns under both denominators: systems + `tick_boundary` reproduces them all; systems only gives A\* 27/70/54% after the memo | `timer.txt`, `timer-pile-memo.txt` (committed at dev) |
| 8 blind instruments | the blind side (counters absent, or a key absent) | **both:** (a) the sabotaged stacks, where the old scripts print nothing or `share_ticks=0` and the new ones print `counters NAME MISSING` ×10 and `COUNTERS MISSING: boundaries=0`; (b) the healthy stacks, where the new `census.sh` on base gives `census-base.txt` again (the only diff is the known `--runs 1` header line), the pile-memo RED censuses with counters print `IDENTICAL` ×10 and no MISSING, and the new `scale.sh` on base and pile-memo prints no flag, with totals equal to `scale-r1-*.txt` | `prototypes/red-counters-silent.patch`, `prototypes/red-counters-no-emitter.patch` (old scripts = `28162c85`); `census-base.txt`, `scale-r1-base.txt`, `scale-r1-pile-memo.txt` |

- [x] [Review][Defer] The 313 → 20.8 ms headline compares a base run in a batch of four with a memo run beside one census [`proposal.md:208`] — deferred: review re-runs read 12.5x and 16x, the counts are equal, and "a fifth" survives
- [x] [Review][Defer] The 256² cells never reach the pile in 2,000 ticks, so they are lower bounds [`proposal.md:210-212`, `scale-table.md`] — deferred: disclosed in § Scale reading
- [x] [Review][Defer] § Issues cells use off-vocabulary words, and #159/squeeze has no number [`proposal.md:305-306`] — deferred, AC 4 wording
- [x] [Review][Defer] The sweeps table has no pile-memo column, and squeeze variant A has no patch or raw output [`proposal.md` § Prototypes] — deferred: pile-memo's sweep lines were verified equal to C0
- [x] [Review][Defer] Raw files miss stack or env headers, and the tick-bench SHAs are dangling objects [`census-clamber-cost8.txt`, `census-connectivity-i.txt`, `timer*.txt`, `tickbench.txt`, `census-base.txt` `--runs 1`] — deferred: every one reproduces from the committed patches
- [x] [Review][Defer] The squeeze shares=64, clamber emitter 407 and connectivity ramps REDs are not committed [`proto-*.md`] — deferred: 13.4/13.5 rebuild them in real code
- [x] [Review][Defer] "28 of 28 `walls_in_a_dwarf` calls in dig-loop" should read 28 of the 28 measured (42 calls) [`proposal.md:294`] — deferred, wording
- [x] [Review][Defer] Profile labels and bookkeeping: the ticks column is the end tick; option deltas are against C0, not base; the control is one range; the issue-comment branch links are dead until push [`proposal.md` § Profile/§ Prototypes] — deferred, labels
- [x] [Review][Defer] Crews above about 48 spawn across several z-levels, which `scale/README.md` does not say [`prototypes/scale.patch` spawn hunk] — deferred: all 200 spawn, distinct
- [x] [Review][Defer] A SIGKILLed `census.sh`/`scale.sh` leaves a stale `.git/worktrees` entry [`census.sh:11`, `scale.sh:11`] — deferred: `git worktree prune`
- [x] [Review][Defer] Timer figures are inclusive, and counter bookkeeping inside the timed `pile_targets` inflates its share by a few percent [`prototypes/timer.patch`] — deferred: does not change a conclusion
- [x] [Review][Defer] `can_clamber` keys on the counters' sticky caller label [`prototypes/clamber.patch`] — deferred: a 13.5 note, do not copy the pattern
- [x] [Review][Defer] Squeeze's wait timer is unsaved and unhashed, so a real 13.4 likely changes the save format and rots `scenarios/saves/*` [`prototypes/squeeze.patch`] — deferred: a 13.4 design note (use `serde(default)`)

**Code review run 2 (2026-10-24, a fresh session), on `28162c85..2a074218`, which is patch pass 1.**

Three layers ran: Edge Case Hunter (Sonnet), Acceptance Auditor (Opus) and Feature Auditor (Opus).
- Blind Hunter was **not launched**: its territory (`sim-core`, `client-core`, gui pure files) has zero lines in this diff.
- Each layer ran `cargo 1.97.1` in its own `/tmp` target dir, on `git archive` trees or a `git clone --shared`.
- No layer timed out. Nothing under `/workspace` was modified.
- AC 7 still holds: `git diff 4f57027 2a074218 -- crates/` is empty.

**Delta against run 1:**

| | run 1 | run 2 |
|---|---|---|
| new findings | 0 HIGH (2 decision, 8 patch, 13 defer, 1 dismissed) | **0 HIGH**, 2 MED, 12 LOW (9 patch, 5 defer, 0 dismissed) |
| rework | — | **2 of 8** closure rows (row 4 tail, row 6 near RED) |

**STOPPING RULE FIRED:** every new finding is below HIGH, so there is no round 3. This is a spike with nothing under `crates/`, so the next spend goes to the one full gate at PR, not to another static round.

**Re-run and matched:**
- **Row 5:** the 16k producer reproduces both committed files byte-identical: squeeze `[(0,4,1),(6,6,1),(9,6,1),(13,6,1)]`, C0 `[]`.
- **Row 6:**
  - The near RED census is byte-identical to the committed file.
  - The key RED moves 8 of 10 hashes.
- **Row 8:**
  - `census.sh` + `red-counters-silent` prints `counters NAME MISSING` ×10.
  - `scale.sh` flags both REDs and stays silent on the healthy stack.
- **Rows 3 and 7:** recomputed from the raw files.
- **Row 2:** `bmad-create-story`'s first-`backlog` scan lands on `13-3`.
- **Live 128² crew 200 pile-memo point:** hash `e0b2df85` = base; mean 19.2, p99 87.1, max 120.7 ms.

**Closure table audit:** rows 1, 2, 3, 5, 7 and 8 are CLOSED; rows 4 and 6 are REWORK (patches 1 and 2 below).

**Wolf (2026-10-24): leave the 9 patches as action items.** Review cost $14.40 / 254 turns (3 subagents = 81.9%). `reap-build-caches.sh --tmp-only --force` reaped 17 dirs (17.8 GB; 14.6 GB reclaimed).

- [x] [Review][Patch] **MED, feature + acceptance. REWORK of row 4: three mean-only "fits the budget" claims survive the tail fix** [`13-2-signoff/proposal.md:122`, `:397`; `planning-artifacts/epics.md:2879`].
  - `:122` "far under budget at all three (5.5–18.9 ms)" and `:397` (the ADR 5 paragraph Wolf ruled on) "the scale curve fits the budget once pile-memo lands" are both means.
  - 13.3's So-that line (written in this pass) says "a 200-dwarf crew fits the tick budget on today's map".
  - The same committed files show maxima over 100 ms at 128² crew 200: 134 (`scale-r1`), 189.6 (`scale-hash`), 116.7 (`timer-pile-memo`), and 120.7 live. At 256² crew 100 the max is 233 ms.
  - Fix: say "mean" in all three places and put the tail beside it. The ADR 5 premise that decides the ruling (A\* 0–3%) is unchanged.
- [x] [Review][Patch] **LOW, feature + acceptance. REWORK of row 6: "the only census reading that moves is the cage sweep's `frozen`" is false, and the gate already catches the near RED** [`proposal.md:255-257`, `epics.md:2889`].
  - The two-wide sweep's counts move: boundaries 51,058 → 51,610, `claim_delivery` goal-checks 3.6M → 17.5M, `clearing_landing` cells 436k → 565k. Every cage-sweep counter moves too. Only the outcome lines stay equal.
  - `pinned=false` is the exact comparison `main`'s `assert_eq!(frozen, ONE_LEVEL_ISLANDS)` makes, so the full gate fails under the near RED. That catch is incidental.
  - Fix: no scenario **hash** sees it; the counts and `frozen` do; the gate's catch is incidental. 13.3's test is still required.
- [x] [Review][Patch] **LOW, acceptance. "Read them as a small real slowdown, not as noise" goes past the 6% rule in the same sentence** [`proposal.md:368-369`]. The two readings are dig-loop +4.3% and full-pile-crowd +3.2%. Fix: keep "weak evidence, beyond the bench floor" and drop the "real slowdown" clause.
- [x] [Review][Patch] **LOW, acceptance + feature. The 1,030 over-10 ms figure is credited to an uncommitted "review re-run"** [`proposal.md:371-372`]. The only committed 1,030 is in `timer-pile-memo.txt`, which is a different stack (p99 85.4, max 116.7). The tail line mixes three runs. Fix: name the committed source for each figure.
- [x] [Review][Patch] **LOW, acceptance. The File List was not updated by patch pass 1** [story § File List]. Missing:
  - the five new patches;
  - `census-16k.txt`, `census-squeeze-16k.txt`, `census-pile-memo-red-{key,near}.txt`, `instrument-red.txt`, `scale-hash-{base,pile-memo}.txt`;
  - the UPDATE of `planning-artifacts/epics.md`.
- [x] [Review][Patch] **LOW, acceptance. 13.3 says "the six pile scenarios"** [`epics.md:2892`]. seat-slow is not a pile scenario. Use the proposal's wording, "the five pile layouts and seat-slow".
- [x] [Review][Patch] **LOW, feature. The epics name basis patches as if they apply to `main`** [`epics.md:2885`, `:2900`, `:2921`].
  - `squeeze.patch` and `clamber.patch` fail `git apply --check` on bare `4f57027`.
  - `pile-memo.patch` applies, but calls `counters::add`, which does not exist on `main`.
  - Both REDs carry that line as context.
  - Fix: one line saying these are C0-stack patches (census + counters), to be ported and their REDs re-expressed.
- [x] [Review][Patch] **LOW, feature. 13.4's #187 gate does not say how to run it on the 13.4 branch** [`epics.md:2907`; `census.sh:9` and `scale.sh:12` hardcode `BASE=4f57027`].
  - `census-16k.patch` alone fails `main`'s pinned cage asserts at 16k.
  - Fix: name the stack to hand-apply on the branch (`census` + `counters` + `census-16k`), and the C0 `unfinished=[]` it is compared against.
- [x] [Review][Patch] **LOW (latent silent-failure trap; frostvein exception), edge + acceptance + feature. Row 8's guards miss "block present, key absent", and `scale.sh`'s flag overclaims** [`census.sh:45-48`, `scale.sh:49-51`].
  - `census.sh` on `counters` + `red-counters-no-emitter` prints the block with no `emitter:ticks` line, then `IDENTICAL`, and nothing else. AC 5's `shares=0` reading is blind there, and the script says nothing.
  - `scale.sh` prints "every total above is blind" on the no-emitter RED, whose A\* and flood totals are real.
  - Fix:
    - `census.sh` prints `counters NAME NO EMITTER` when a counted block has no `emitter:ticks boundaries=`.
    - `scale.sh`'s message names only the emitter totals as blind (`share_ticks`), or says "counters absent" when every total is 0.
- [x] [Review][Defer] MISSING and COUNTERS MISSING print but never change either script's exit code [`census.sh:80`, `scale.sh` awk `END`] — deferred: item 8 asked for a printed flag, and no caller gates on these scripts' status
- [x] [Review][Defer] `census.sh` arms MISSING only when a patch's basename is exactly `counters.patch` [`census.sh:18`] — deferred: every committed stack uses the canonical name
- [x] [Review][Defer] The `scale.sh` digest sed matches single-digit run counts only, so `SCALE_RUNS>=10` prints raw `bench … run` lines [`scale.sh:41`] — deferred: the raw line still carries `state=` and `over10=`; nothing is hidden
- [x] [Review][Defer] `13-5-…: backlog` is a plain board key, so a bare `create-story` would pick it after 13.4 [`sprint-status.yaml:2765`] — deferred: Wolf ruled (a), and the epic heading carries "*(only if ruled)*", which create-story reads ([[board-order-contradicts-ruling]] shape)
- [x] [Review][Defer] Patch pass 1's "full gate green at `b3a4bf74` (3902 s)" has no pasted gate block, unlike Task 9 [story Change Log] — deferred: M3-4's one full gate at PR is the binding one, and it will be pasted

**Patch pass 2 (2026-10-24, fresh session): 9/9 landed** (`7cf68ec4` instruments, `0e7df2e4` proposal, `6cef978c` epics, `0e5c0308` readings + File List). Closure table below. Rows 1, 2 and 9 are REWORK: rows 1 and 2 re-close pass 1's rows 4 and 6, and row 9 closes a gap in pass 1's row 8 (the run-2 audit had marked row 8 CLOSED). Nothing under `crates/` changed. Row 9's mutations are the two committed instrument REDs, recorded in `instrument-red.txt`.

| item | fix written for | then tested | pre-existing-state fixture |
|---|---|---|---|
| 1 mean-only claims (REWORK row 4) | the three mean-only sentences (`proposal.md` :122 and the ADR 5 paragraph; 13.3's So-that line) | each tail figure re-read from its raw line, and a sweep of the signoff dir, `epics.md` and `technical-preferences.md` for surviving "fits the budget" / "far under budget" copies (none outside the review records) | `timer-pile-memo.txt` rows `scale-128-crew200`, `scale-256-crew100` and `scale-256-crew200` (p99 85.4/64.6/62.2, max 116.7/233.3/103.2); `scale-r1-pile-memo.txt` crew 200 (p99 97.1, max 134.1) |
| 2 near RED reach (REWORK row 6) | the "only `frozen` moves" claim | the full diff of the two censuses, with every moved line accounted for. It also corrects the finding: the `pile_targets` counters of big-pile, deep-pit and seat-slow move too. `pinned` was traced to `frozen_keys == ONE_LEVEL_ISLANDS` in `census.patch`, which is `main`'s assert at `tests/scenario.rs:3113-3120` | `census-pile-memo.txt` against `census-pile-memo-red-near.txt` (pass 1) |
| 3 "real slowdown" clause | the over-reach | the kept sentence re-read against the raw row | `tickbench.txt` § "C0 vs pile-memo": dig-loop +4.3% (control +1.4%), full-pile-crowd +3.2% (control −0.1%) |
| 4 tail sources | the uncommitted "review re-run" credit | each figure traced to one committed line. `scale-r1` has no over-10 count (it predates the field), so 1,030 is credited to `timer-pile-memo.txt` | `scale-r1-pile-memo.txt:30`, `timer-pile-memo.txt:13`, `scale-hash-pile-memo.txt:30` |
| 5 File List | the missing pass 1 files | every A/M path in `git diff --name-status 28162c85 486bb03f` under `13-2-signoff/` and `planning-artifacts/` is now named | the File List as of `486bb03f` |
| 6 "six pile scenarios" | 13.3's wording | the six −55…−91% rows of the raw section are big-pile, plan, plan-compact, plan-far, plan-spread and seat-slow | `tickbench.txt` § "C0 vs pile-memo" |
| 7 C0-stack basis | the "applies to `main`" reading | `git apply` on a bare `4f57027` worktree (bash): `squeeze` and `clamber` fail bare and apply on `census`+`counters`; `pile-memo` applies bare, but `main` has no `counters` module (`rg 'pub mod counters'` finds nothing) | `4f57027` (`main`'s code) |
| 8 #187 gate on the branch | 13.4's run instruction | the `census`+`counters`+`census-16k` stack applies to `4f57027`, and it is exactly the stack in `census-16k.txt`'s header, which reads `unfinished=[]`. Not run on a 13.4 branch, because none exists | `census-16k.txt`, `census-squeeze-16k.txt` (pass 1) |
| 9 block present, key absent (REWORK row 8) | the blind side (an emitter key absent from a printed block) | **both.** (a) Blind: `census.sh counters red-counters-no-emitter` prints `counters NAME NO EMITTER` ×10. `scale.sh` prints `NO EMITTER: … share_ticks is blind` on the no-emitter RED, where the A\* and flood totals are real, and `COUNTERS MISSING: no counters block` on the silent RED. (b) Healthy: `census.sh counters` is byte-identical to `census-counters.txt` (0 NO EMITTER, 10 IDENTICAL), and `scale.sh` on base prints no flag | `prototypes/red-counters-no-emitter.patch`, `prototypes/red-counters-silent.patch` (pass 1); `census-counters.txt` (dev) |

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

- **Task 1, census (Sonnet agent; `8a90b8d`, `63590df`, `c09ca31`).** Opus re-ran both from a cold target and diffed them: `census.sh` is byte-identical to `census-base.txt`, and `census.sh prototypes/ghost.patch` is byte-identical to `census-ghost.txt` (80 s cold for the base run).
  - **Base reproduces the creation table.** stuck-tunnel `tick=28082 marks=3 items=57 goals=38/4/0`, dig-loop `tick=17640 marks=11 items=44`, plan `marks=0 goals=12/10/1`, full-pile-crowd `marks=0 items=46`, deep-pit `marks=0`, seat-slow `marks=1 goals=38/12/1`.
  - **Base sweeps.** frozen `[(10,6,0),(12,4,1),(12,4,4),(12,6,3)]`, unfinished `[(11,6,1)]`, landed `[]`, two-wide `24/24`, one-wide `failed=[]`. Shares and swaps are 0.
  - **Ghost discriminates.** stuck-tunnel marks 3→0, plan 0→4 (goals 12/8/1), full-pile-crowd 0→37, dig-loop 11→20. The share counter fires: head-on 319, one-wide 37,231.
  - **Census additions.** `census.patch` adds one census-only test, `census_207_four_bars_round_a_hole`, because no test filled #207's four-bar shape. It reads `stored=14/24`, the issue's number. Other readings: #189 `stored=23/24` with 80 pick-ups after t2500, and pocket207 `24/24`.
  - **The saves.** `12-12-seat-loop.save` is byte-identical to the kept copy (`cmp`). The patched #203 save adds `"clearing": null` per dwarf and top-level `"goals_completed": 0`. Both scenarios give the same hash over `--runs 2`.
- **Task 3, counters (Sonnet agent; `e467791`, `a1072d5`).** Opus re-ran the counters census, which matches the committed `census-counters.txt` exactly. Every state hash and sweep line equals base, and 10/10 counter blocks read `IDENTICAL across 2 runs`.
  - **RED.** Heuristic-twice doubles every goal-check count (plan `claim_goals` 176,776 → 353,552) and leaves nodes (83,060) and every hash unchanged.
  - **Share-counter RED.** Under ghost the emitter reads shares > 0 on 9 of 10 scenarios. Base reads 0 on all 14 blocks.
  - **Top cost.** `caging_stone`'s flood is the top cost in 9 of 10 scenarios (59–74% of nodes+cells). In stuck-tunnel it is `execute_replan` instead: 69 no-path searches of about 16k nodes each, 1.1M nodes in all.
- **Task 2, survey (Sonnet agent, web).** The draft is in the session scratchpad and goes into the proposal at Task 8. Opus re-fetched the load-bearing quote, DFHack `Maps.cpp` L1076-1087 `canWalkBetween` = `tile1 && tile1 == tile2` and `df.block.xml` L288-289, and it matches.
- **Task 4, scale (Sonnet agent; `caa0a71`, `9f9069d`; Opus `9294b89`).** `scale.patch` is inert on the committed scenarios (hashes equal C0). Base reading (3 runs, `scale-base.txt`): 128² crews 5…200 → 0.41/5.25/30.9/95.7/309 ms. The camp component is 16,063 cells on 128² and 64,300 on 256², which is over the 50k budget. Exhausted A\* calls on 256² are 10/10/131/218/138; there are none on 128². The scale agent's first run was killed mid-way by another agent's `pkill -f` (stitched from 4 invocations; header says so). Opus then re-ran all ten points for base and every option at once with `SCALE_RUNS=1` (`scale-r1-*.txt`). Base counts match `scale-base.txt` exactly.
- **Task 5, connectivity (Sonnet agent; `c8c9327`).** Opus re-ran the T and I censuses: both are byte-identical to the committed files, and every hash equals base. The quiet tick-bench (`tickbench.txt` §2–3) shows every scenario slower: T +1.8% … +118.7%, I +8.6% … +421.5%. The guards-as-one-question variant was not built. The level guard reads false at 3,018 of 3,018 calls in seat-slow.
- **Task 6, squeeze (Sonnet agent; `443b872`).** Opus re-ran its census byte-identical (`shares=0`). It fixes #206 (3 → 0) and #189 (24/24), and moves #207 from 14 to 18. It makes #187 worse: 4 cage runs are stuck at 16k ticks (review-patch: producer committed as `prototypes/census-16k.patch`, raw `census-16k.txt` and `census-squeeze-16k.txt`). Tick-bench §4: stuck-tunnel −71% (the livelock is gone); the other states differ.
- **Task 7, clamber (Sonnet agent; `31d7342`).** Opus re-ran the cost-4 census byte-identical (`landed=[]`, `shares=0`). It fixes #189 and #207 (24/24), and makes #203 worse (11 → 14). The guards variant breaks outcomes (two-wide staircase 12/24). Tick-bench §5: +6% … +35% in the changed worlds.
- **Task 3 addendum: counts ranked the wrong thing (Opus).** In the quiet tick-bench, plan-compact runs 1.39 ms and plan-far 0.22 ms on nearly equal counts. A throwaway `timer.patch` (lap systems plus drop-timers; hashes equal base) put `pile_targets` at 69–94% of the tick in every scenario over 0.1 ms and at the three 128² scale points timed (crews 50/100/200), with A\* at 0–3%. A cache hit still rebuilds the depth map, about 57 µs at 203 zone cells.
- **pile-memo (Opus; AC 3's "cost fix the profile justifies"; `9294b89`).** `pile-memo.patch` keeps the last `pile_targets` answer, keyed on the blocked cells among the zones and their side neighbours. Census: 10/10 hashes and end lines equal C0, and every sweep line is equal. RED: a key that ignores `blocked` moves 8 of 10 hashes (review-patch: committed as `pile-memo-red-key-ignores-blocked.patch`; the prose first said all 4 tried), and a key that reads the zone cells only (`pile-memo-red-near-zone-only.patch`) moves no hash and only the cage sweep's `frozen` list. Quiet tick-bench (§6): −55% … −91% on six scenarios (the five pile layouts and seat-slow); the other four move −10.6% … +4.3% (review-patch: this line first said seven and three). Scale: 128² crew 200 drops from 313 to 20.8 ms with identical counts. A first hash check was vacuous: the regex matched 0 lines on both sides, so "equal" proved nothing. The re-check matched 10 lines.
- **Task 8 (Opus).** Wrote `13-2-signoff/proposal.md` (§ Survey, Profile, Scale, Prototypes, Issues, Rulings needed, Recommendation, ADR 5). `git diff 4f57027 -- crates/` is empty.

- **Task 9, the one full gate (Opus; `RUST_TEST_THREADS=1 scripts/gate.sh` at `18c56634`).** Green, 6413 s. That is longer than the 45–65 min estimate because the pixel guards alone took 5568 s:
  ```
  frostvein gate
    cargo fmt --check           ok  1s
    cargo clippy -D warnings    ok  2s
    cargo test                  ok  737s
    cargo test (pixel guards)   ok  5568s
    tui has no sim-core edge                ok
    client-core has no sim-core edge        ok
    gui has no sim-core edge                ok
    metrics ledger tests        ok  0s
    bench tests                 ok  51s
    mutation tables still apply ok  25s
  GATE GREEN  6413s
  ```

### Completion Notes List

- Spike done. No sim change lands: `git diff 4f57027 -- crates/` is empty (AC 7). Every prototype is a throwaway patch under `13-2-signoff/prototypes/`.
- Wolf ruled the proposal on 2026-10-23. The next stories are 13.3 pile-memo, then 13.4 squeeze-past, then 13.5 clamber-over only if ruled. ADR 5's scope is amended.

### File List

All under `_bmad-output/implementation-artifacts/` unless noted; nothing under `crates/`.

- NEW `scenarios/stuck-tunnel.json`, `scenarios/dig-loop.json`, `scenarios/saves/12-12-seat-loop.save`, `scenarios/saves/12-11-dig-loop-patched.save` (repo root)
- NEW `13-2-signoff/census.sh`, `13-2-signoff/scale.sh`
- NEW `13-2-signoff/prototypes/{census,ghost,counters,counters-red-heuristic-twice,scale,connectivity,squeeze,clamber,clamber-guards,pile-memo,timer}.patch`
- NEW `13-2-signoff/scale/*.json` (10) and `13-2-signoff/scale/README.md`
- NEW `13-2-signoff/census-{base,ghost,counters,counters-ghost,connectivity,connectivity-i,squeeze,clamber,clamber-cost8,clamber-guards,pile-memo}.txt`, `13-2-signoff/profile-red.txt`
- NEW `13-2-signoff/scale-{base,connectivity,connectivity-sub,squeeze,clamber}.txt`, `13-2-signoff/scale-r1-{base,connectivity-i,squeeze,clamber,pile-memo}.txt`
- NEW `13-2-signoff/tickbench.txt`, `13-2-signoff/timer.txt`, `13-2-signoff/timer-pile-memo.txt`
- NEW `13-2-signoff/profile-table.md`, `13-2-signoff/scale-table.md`, `13-2-signoff/proto-{connectivity,squeeze,clamber}.md`, `13-2-signoff/proposal.md`
- UPDATE `docs/technical-preferences.md` (ADR 5 scope line, per Wolf's ruling)
- UPDATE this story, `sprint-status.yaml`
- Review patch passes 1 and 2:
  - NEW `13-2-signoff/prototypes/{census-16k,pile-memo-red-key-ignores-blocked,pile-memo-red-near-zone-only,red-counters-silent,red-counters-no-emitter}.patch`
  - NEW `13-2-signoff/census-16k.txt`, `13-2-signoff/census-squeeze-16k.txt`, `13-2-signoff/census-pile-memo-red-{key,near}.txt`, `13-2-signoff/instrument-red.txt`, `13-2-signoff/scale-hash-{base,pile-memo}.txt`
  - UPDATE `13-2-signoff/census.sh`, `13-2-signoff/scale.sh`, `13-2-signoff/proposal.md`, `13-2-signoff/proto-squeeze.md`, `deferred-work.md`
  - UPDATE `planning-artifacts/epics.md` (Epic 13: stories 13.3, 13.4, 13.5)

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-23 | Story created on `db8fde3` (Opus 5.5). Creation probe: both issue saves freeze on HEAD (#206 marks=3 and #203 marks=11, each over 10k+ ticks; #203's save needed `clearing` and `goals_completed` added). The ghost-dwarves RED moved #206 from 3 to 0 and made 3 of 6 reproducers worse. There is no M4 PRD draft and no profiler on the devpod. Task 0 Q1–Q3 await Wolf. |
| 2026-10-23 | Task 0 ruled by Wolf: Q1 crews of 5/20/50/100/200 on today's map, and (follow-up) on 256×256×32 too; Q2 one story (split at Part B on overrun); Q3 commit both issue saves. |
| 2026-10-23 | Tasks 1–8 done (Sonnet 5.5 agents implement, Opus 5.5 verifies; every agent output re-run by Opus). The proposal recommends: pile-memo first (no ruling; 128² crew 200 drops from 313 to 21 ms with identical outcomes), then squeeze if the swap is ruled in, then clamber only if standing on a stone is ruled in. No connectivity map; #159 waits for the M4 map size. Four issues are not movement. ADR 5 keeps plain A\*, and its scope becomes "while the walkable component is under the node budget". Awaiting Wolf's ruling. |
| 2026-10-23 | **Wolf ruled the proposal** (four questions, each answered with the recommended option). Order as recommended: 13.3 pile-memo, 13.4 squeeze-past, 13.5 clamber-over only if ruled. Ruling 1: the swap is allowed, and 13.4 must clear the #187 sweep regression first. Ruling 2: clamber is decided after squeeze, once #207 is re-measured on 13.4. ADR 5 is amended as proposed (scope line in `docs/technical-preferences.md`). #159 waits for the M4 map size; #186/#187/#203/#204 get their own stories. Recorded in proposal § Ruling. |
| 2026-10-23 | Full gate green at `18c56634` (6413 s). Status → review. |
| 2026-10-24 | Review patch pass 1 (fresh session, Opus 5.5): 8/8 `[Review][Patch]` items landed in 5 commits (`739adfac`…`b3a4bf74`), each both-sides verified (closure table under Review Findings). Item 6 corrected its finding: the near RED is seen by the cage sweep's `frozen`, not by any scenario hash. Full gate green at `b3a4bf74` (`RUST_TEST_THREADS=1`, 3902 s). `git diff 4f57027 -- crates/` still empty. Status → review for round 2. Cost $11.78 / 192 turns. |
| 2026-10-24 | Code review run 2 (fresh session, Opus 5.5) on `28162c85..2a074218`: 0 HIGH; closure rows 4 and 6 REWORK; 9 patch left as action items (Wolf), 5 deferred. The stopping rule fired: no round 3. Status → in-progress. |
| 2026-10-24 | Review patch pass 2 (fresh session, Opus 5.5): 9/9 `[Review][Patch]` items landed in 4 commits (`7cf68ec4`…`0e5c0308`), each both-sides verified (pass 2 closure table under Review Findings; rows 1, 2 and 9 are REWORK). Each commit's fast gate was green. **The full gate was NOT completed:** Wolf stopped it at 16 min (`fmt`, `clippy` and `cargo test` were ok in 715 s; the pixel guards and `serve.rs` were not run), to be resumed in the next session. `git diff 4f57027 -- crates/` is still empty. Status stays in-progress. |
