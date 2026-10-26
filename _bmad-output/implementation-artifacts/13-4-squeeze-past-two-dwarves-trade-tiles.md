---
baseline_commit: c5164d93
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
---

# Story 13.4: Squeeze Past — Two Dwarves Trade Tiles

Status: ready-for-dev

## Story

As the boss,
I want a dwarf held up by another in a one-wide way to trade tiles with him after a short wait,
so that the seat freeze (#206) and the boxed-in hauler (#189) stop happening, without giving up one dwarf per tile.

## Not stacked: branch off `main`

`main` is `c5164d93` (PR #217, 13.3). Branch `story-13-4-squeeze-past-two-dwarves-trade-tiles` carries `380fd789` (13.3's done-mark) and this file. Epic 13's standing ACs bind: M3's ACs 1–8, with **one full gate per story, at PR** (M3-4, #212). Wolf ruled the 13.2 proposal on 2026-10-23 (Ruling 1): the swap is allowed, variant B of `13-2-signoff/prototypes/squeeze.patch` is the basis, and this story must clear the #187 cage-sweep regression before it lands.

**Wire diff (standing AC 4): none.** A swap is two `Pos` writes in one tick, and positions already travel in the deltas. No client assumes one dwarf per tile (`rg` over `gui`, `client-core`, `tui`). **Save format: one new per-dwarf field**, the stand-off timer (Task 0 Q3).

## Task 0 — Wolf rules before dev

**RULED 2026-10-26 (Wolf: "1 ok 2 a 3 a 4 yes"):** every recommended option. Q1 (a) the idle-blocker swap is allowed, `home` moves. Q2 (a) the #187 fix is folded in, judged at waits 10/20/30, with the split point after Task 3. Q3 (a) the timer is a plain `Option`, so a pre-13.4 save loads with no stand-off. Q4 (a) a short seat sitting (Task 7). The ACs and Tasks stand as written.

Each question has a recommendation. The ACs and Tasks below are written for the recommended answers; a different answer amends the ACs named.

- **Q1. The idle-blocker swap** (13.2 review decision 1, epics.md Story 13.4). `squeeze.patch` also swaps an idle blocker that has no tile aside and no exit, and moves his `home` to the holder's old tile. Ruling 1 did not cover it. Measured at creation (`variant-n.patch`: only head-on job-holders swap; idle blockers keep `main`'s ladder, #194 back-out included):

  | reading | `main` (C0) | B, as ruled | N, no idle swap |
  |---|---|---|---|
  | #206 stuck-tunnel marks | 3 | **0** | **0** |
  | #189 campfire pick-ups after t2500 / stored | 80 / 23 of 24 | **0 / 24 of 24** | **0 / 24 of 24** |
  | #207 ring (`census_207_four_bars_round_a_hole`) | 14/24 | **18/24** | 13/24 |
  | cage `unfinished` at 16k ticks | `[]` | 4 runs | the same 4 runs |

  (a) **Allow it, as in the patch (recommended).** It is the only part that moves #207 (13 → 18 of 24, which 13.5's ruling reads), and it costs nothing on #187. The `home` move is one tile. (b) Swap head-on job-holders only. #206 and #189 are still fixed, #207 reads one worse than `main`, and 12.10b's back-out stays for idle blockers (row 12 keeps its seam; row 15 goes inert either way, since only idle blockers reach it). Answer (b) amends AC 1, Task 1 and mutation row 4.

- **Q2. What "clear the #187 regression" means.** The 4 runs are not a swap shape. Each ends with the pile full and one channel mark that nobody completes:
  - **seeds 0 and 6** are #187's refused-cut loop. A miner claims the cut, works it, is refused and retries, with 28 claim changes in the last 1,000 ticks. On seed 6 the refusal is an idle dwarf on a one-level island who stands **on the mark** (68,60,8) and never leaves (#186 + #195's refusal).
  - **seeds 9 and 13** have every dwarf idle. The mark's work cells are walled by unhauled stones, so the claim finds no path and is re-stamped (the #182 mode ii shape, with the pile full).

  **Which runs land there is timing.** The same patch with `SQUEEZE_WAIT_TICKS` at 20 leaves `[(0,4,1),(12,6,1)]`, at 30 it leaves `[(4,6,1)]`, and at 5 it leaves the same 4 as at 10 (control: 5 and 10 both swap on the holder's second look). So a clean sweep is a fix to those shapes, not a tuning of the swap.
  - (a) **Fold the fix in (recommended).** This story lands squeeze **and** fixes the stuck shapes. It is judged by a 16,000-tick sweep reading no unfinished run at the shipped wait, and also at waits 20 and 30, so the clean reading is not one timing draw. Split point: after Task 3's diagnosis, if the fix needs more than the two shapes above, or touches the claim system beyond them, stop and ask (squeeze stays on the branch; the fix becomes 13.4b).
  - (b) A separate #187 story first, using this stack at waits 10/20/30 as its RED (7 stuck runs); 13.4 lands after it.
  - (c) Re-rule the gate: land squeeze with the 16k reading pinned and the 4 runs posted on #187 as reproducers.

  Answer (b) or (c) amends AC 5, removes Tasks 3–4, and re-scopes the PR.

- **Q3. Saving the stand-off timer** (standing AC 3: new sim state is saved). The prototype's timer is unsaved, so a save mid-wait loads with the wait restarted and diverges. Every per-dwarf field so far is **required** (a pre-X save is refused; `save.rs:40-57`).
  - (a) **A plain `Option`, so a missing field reads as `None` (recommended).** A pre-13.4 save cannot hold a stand-off timer, so `None` is exact. The three committed scenario saves stay byte for byte, and `stuck-tunnel` is this story's own #206 reproducer. A `// NOTE:` says why it differs from `clearing`.
  - (b) Required (`deserialize_with`, as `clearing`). Then rewrite the three `scenarios/saves/*.save` with `"squeeze":null` per dwarf, and show that `main`'s binary reads the same end `state=` from old and rewritten files (`main` ignores the unknown field).

- **Q4. A seat sitting.** Sim work is judged by scenario tests (M2-26), but this is the first time two dwarves visibly pass through each other, and the gui has no swap animation (13.2: out of scope). (a) **Short sitting (recommended):** `simd 7451 --scenario scenarios/stuck-tunnel.json`, watch the tunnel at x 77, y 64–71, z 10. Wolf judges whether the pass reads acceptably plain; if not, it becomes an issue, not this story's work. (b) No sitting. Answer (b) removes Task 7.

## Found at creation (2026-10-26, on HEAD `380fd789` = `main` + the done-mark)

Throwaway scratch worktrees, never committed. Stacks are 13.2's: **C0** = `census` + `counters` + `census-16k` (16,000-tick cage sweep, asserts turned into `census …` lines); **B** = C0 + `squeeze.patch`; **N** = B + `13-4-signoff/variant-n.patch`. All three applied cleanly to HEAD (13.3's memo did not conflict). `probe.sh`-style run: `simd --bench` on 4 scenarios, then the cage, campfire, ring, staircase and head-on tests in release.

- **The 13.2 numbers reproduce exactly on today's HEAD.** C0: stuck-tunnel `marks=3` (`state=8365ab4d3aafed06`), campfire 80 / 23 of 24, ring 14/24, cage 16k `unfinished=[]`. B: stuck-tunnel `marks=0 … goals=38/4/1` with 7 swaps, campfire 0 / 24 of 24, ring 18/24, cage 16k `unfinished=[(0,4,1),(6,6,1),(9,6,1),(13,6,1)]`, `landed=[]`, `shares=0` everywhere, one-wide staircase 96/96 with 101 swaps, head-on 4/4 with 1 swap.
- **N**: see Q1. Same 4 unfinished runs as B, so the idle-blocker swap is not their cause.
- **Wait sweep** (B with only the constant edited): see Q2. #206 and #189 stay fixed at every wait (7 swaps, 0 / 24 of 24).
- **The 4 runs, read at tick 16,000** by `13-4-signoff/probe187.rs` (a throwaway test: `CASES=0:4,6:6 cargo test --release -p sim-core --test probe187 -- --nocapture` in a stacked worktree). It prints the marks, non-haul jobs, each dwarf's tile, state, claim, cells visited and claim changes over the last 1,000 ticks, and a 7×7 map on three levels around the mark. Readings in Q2.
- **Clean port on HEAD** (`squeeze.patch` with its back-out hunk ported by hand and the `counters::add` line removed; the swap asserts removed from `watch` and `step_and_watch`). Full `cargo test --release --no-fail-fast -p sim-core`, exactly 4 failures:
  - `head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_{open,dead}_end`, on their positive "backed out" asserts (`lib.rs:6984-6987`, `:7002-7005`). The swap now replaces the yield.
  - `a_stockpile_around_the_campfire_never_zones_or_receives_the_fire`, at `scenario.rs:120`: "pick-ups after t=2500: 0 (pinned 80, the #189 jam)". That is the fix showing.
  - `no_dwarf_is_caged_by_stones_over_the_probe_seeds`, at the `frozen` pin (`scenario.rs:3113`): `[(2,6,1),(7,6,3),(8,6,4),(12,4,4),(12,6,3)]` against `ONE_LEVEL_ISLANDS`.
  - `a_dead_end_miner_behind_idle_woodcutters_finishes_its_dig` and `a_3x3_pile_fills_from_the_inside_out` pass once the swap assert is gone. Every `save_load` test passes, but none exercises a stand-off.
- **Mutation audit** (`python3 scripts/audit-mutations.py` on B): `12-10b.sh` rows 12 and 15 cannot apply, because the back-out they sabotage is deleted. Row 18 ("a head-on yield swaps the two dwarves") still applies, but it goes inert: an instant swap is legal to `step_and_watch` once the swap assert is removed. On `main`, all 955 rows apply.

## Acceptance Criteria

1. **The swap rule (Ruling 1, variant B; the mechanism is the ruling).** When a holder's next tile is held by a dwarf the ladder cannot clear, and the same dwarf has held him for `SQUEEZE_WAIT_TICKS` (10, one step period), the two trade tiles in one write. The swap applies to two head-on job-holders, which swap instead of #196's yield, and (Q1 a) to an idle blocker with no tile aside and no exit, whose `home` becomes the holder's old tile. A job-holder who is not coming the other way is never swapped. Both rest `STEP_REST_TICKS` after it.
2. **One dwarf per tile at every tick boundary.** `watch` (`lib.rs:6911`) and `step_and_watch` (`scenario.rs:4743`) still fail on a shared tile and no longer fail on a swap. A sabotage that writes only one of the two positions fails them.
3. **The wait is real.** A lib test drives `resolve_blocked_step` on a head-on pair: no swap on the first look, then a swap once the wait has run. A swap on the first look fails it.
4. **The timer is saved.** A `save_load` test saves at every tick of a stand-off that ends in a swap, loads each save, and steps it identically to the never-saved world (dwarves, claims, paths). Dropping the timer on load fails it. Old saves load under Q3's answer.
5. **#187 is cleared (Q2 a).** A committed 16,000-tick cage sweep reads no run with marks left and no stone landed on a dwarf. The same sweep, with throwaway edits of `SQUEEZE_WAIT_TICKS` to 20 and to 30, also reads no unfinished run.
6. **The reproducers.** `simd --bench scenarios/stuck-tunnel.json` ends `marks=0` (`main`: 3). The campfire test is restored to 0 pick-ups after t2500 and every zone cell filled, the two figures its #189 comment names (`scenario.rs:114-131`). #207's ring is re-measured on top of the swap and recorded for 13.5's ruling.
7. **Every moved pin is explained, none is loosened.** The two head-on lib tests assert that a swap resolved the meeting, both jobs finished, and nobody was released. Each new or removed `ONE_LEVEL_ISLANDS` entry and each 4,000-tick `UNFINISHED` entry carries a read-by-dump comment in the house style (`scenario.rs:2950-3006`). An `UNFINISHED` entry must be shown to finish by 16,000 ticks.
8. **Same run twice, same state.** `scripts/tick-bench.sh main HEAD` reads two identical runs per build, and `differ` against `main` on the scenarios the swap changes. The story records the table, stuck-tunnel's cost included (13.2: −71%, the livelock gone).

## Tasks / Subtasks

- [ ] **Task 0: Wolf's rulings (Q1–Q4).** Record the answers in the Change Log and amend the ACs they name. Then ASK which implementer before dev (memory: `dev-implementer-is-sonnet-not-codex`).
- [ ] **Task 1: port the swap (AC 1).** UPDATE `crates/sim-core/src/lib.rs`. Port `13-2-signoff/prototypes/squeeze.patch` by hand: it is a C0-stack patch, and its back-out hunk carries `counters::at`, which `main` lacks. Hunks 1–5, 7 and 8 apply on HEAD with offsets; hunk 6 is ported by hand (creation did exactly this).
  - [ ] **Done means B's readings reproduce** (the timer as a component must not change an outcome): `simd --bench` ends stuck-tunnel `tick=28082 marks=0 items=60 zones=89 goals=38/4/1`, big-pile `marks=12 items=34 … goals=0/1/0`, dig-loop `marks=10 items=44`, seat-slow `marks=1 items=129 … goals=38/11/1`; the 4,000-tick cage `frozen` reads `[(2,6,1),(7,6,3),(8,6,4),(12,4,4),(12,6,3)]`. Compare end lines, not `state=`: the hash covers the save encoding, which gains the new field.
  - [ ] `SQUEEZE_WAIT_TICKS` beside `STEP_REST_TICKS` (`:44`). Drop `SQUEEZE_HEAD_ON_MOVERS`: variant B is ruled, so no switch (YAGNI).
  - [ ] Replace the `Squeeze` resource with a per-holder component, `Squeeze { blocker: Id, since: u64, last: u64 }`, beside `Clearing` (`:303-308`). It is set by the first blocked look at a blocker and kept while the same blocker holds him within `2 * STEP_REST_TICKS` of the last look; it is removed by the swap. It is saved per dwarf (Task 2), so save and load do not need a resource. Keep the prototype's `(who, first, last)` semantics.
  - [ ] `try_squeeze` and its four call sites in `resolve_blocked_step` (`:1720-1859`). No `counters::add`. The back-out (`:1818-1829`) is deleted; with Q1 (b) it stays for an idle blocker.
  - [ ] Rewrite `resolve_blocked_step`'s doc comment (`:1697-1719`): item 3 gains the swap, the #194/#196 back-out paragraph goes, and the closing `NOTE: no swap anywhere (Wolf, Task 0 Q1b)` becomes the rule: the swap, its wait, and who never swaps. Fix `make_room_down_the_idle_chain`'s "No swap" line (`:1659`) and `an_idle_dwarf_on_the_dig_face…`'s "NO swap" comment (`:7100-7102`) only if they become false.
- [ ] **Task 2: save the timer (AC 4).** UPDATE `crates/sim-core/src/save.rs`: `SavedDwarf.squeeze: Option<SavedSqueeze>` with `SavedSqueeze { blocker: u32, since: u64, last: u64 }`, under Q3's answer. UPDATE `to_save` (`lib.rs:2851`) and `from_save` (`:2966`) as `clearing` does. The state hash (`simd/src/scenario.rs:248`, FNV over the save encoding) then covers it.
  - [ ] NEW test in `crates/sim-core/tests/save_load.rs`, copying `a_world_saved_at_every_tick_of_a_clear_loads_and_steps_identically` (`:855`). Use the head-on tunnel fixture from `save_load_keeps_the_exit_path_of_an_idle_dwarf_leaving_a_dead_end` (`:652`) or the lib `head_on`, and assert a swap happened in the window (vacuity).
- [ ] **Task 3: diagnose the stuck runs, then stop if needed (AC 5; Q2 a).** On the branch after Task 1, with the cage sweep run to 16,000 ticks (Task 4's test), trace every unfinished run at waits 10, 20 and 30: why the cut is refused (which guard: `walls_in_a_dwarf`, `closes_last_level_exit`, the #195 dwarf-on-target refusal), or why no claim finds a path. `13-4-signoff/probe187.rs` is a ready starting point. Record the shape per run in the Debug Log. **Split point:** if the shapes are more than the two in Q2, or the fix reaches into claiming beyond them, stop and ask Wolf before writing the fix.
- [ ] **Task 4: fix the shapes, and keep the 16k sweep (AC 5).** UPDATE `crates/sim-core/src/lib.rs` (the fix) and `crates/sim-core/tests/scenario.rs`. Run the cage sweep (`:2947-3133`) to 16,000 ticks **once**, reading the 4,000-tick pins (`frozen`, `UNFINISHED`) on the way and asserting at 16,000 that `unfinished` and `landed` are empty. Name it so the Verification filter `the_cage_sweep_leaves_no_mark_after_16000_ticks` (or `cage_sweep`) selects it. Cost: the 16k sweep took **333 s in a debug build** at creation (C0, one test, light load), against about a quarter of that for today's 4,000-tick sweep, so one run that reads both costs the full gate about 4 minutes more; two separate sweeps would cost about 5. Then run it with `SQUEEZE_WAIT_TICKS` edited to 20 and to 30 (throwaway; paste both readings). Before the fix it must be RED at wait 10 with the 4 runs above; paste that.
- [ ] **Task 5: rewrite the no-swap pins (AC 2, 7).** UPDATE `lib.rs` tests and `tests/scenario.rs`.
  - [ ] `watch` (`lib.rs:6911-6946`) and `step_and_watch` (`scenario.rs:4743-4773`): delete the swap assert and keep the shared-tile assert. Update both doc comments ("or swap tiles", "no swap, 12.9 Task 0 Q1b").
  - [ ] `head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_{open,dead}_end` (`:6973-7006`): replace the "backed out" assert with "a swap of dwarves 0 and 1 between two consecutive trace rows" (both jobs done and nobody released, as now). Rewrite the 12.9 AC3 comments to the new rule.
  - [ ] NEW lib test, beside `a_head_on_with_no_escape_drops_the_idle_blockers_stale_path` (`:7012`): `a_held_dwarf_swaps_only_after_the_wait` (AC 3), on a head-on job-holder pair: first `resolve_blocked_step` call → `false`, positions unchanged; tick advanced by `SQUEEZE_WAIT_TICKS`, second call → positions traded, `occupied` unchanged, the blocker's `Path` advanced. With Q1 (a), a second case: an idle blocker with no way out swaps and his `home` is the holder's old tile.
  - [ ] Campfire test (`scenario.rs:114-131`): restore `pickups_after_full == 0` and `filled.len() == zones.len()`, and replace the #189 jam comment with the 13.4 fix.
  - [ ] Cage sweep pins: re-read `ONE_LEVEL_ISLANDS` and `UNFINISHED` after Tasks 1 and 4 and re-pin per AC 7. Every entry gets its reading, as the existing comments do.
- [ ] **Task 6: mutation table (AC 2, 3, 4, 5).** NEW `_bmad-output/implementation-artifacts/mutations/13-4.sh` in `13-3.sh`'s shape. Run it alone, after commit: `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-4.sh`. Every row must be KILLED at its own assert:
  1. the wait is 0 (swap on the first look) → `a_held_dwarf_swaps_only_after_the_wait`;
  2. `try_squeeze` writes the blocker's `Pos` but not the holder's (13.2's shares=64 sabotage) → a `watch` or `step_and_watch` shared-tile assert;
  3. head-on job-holders never swap (the call removed) → a rewritten head-on test;
  4. (Q1 a) the idle blocker's `home` is not moved, or it never swaps → the idle case of `a_held_dwarf_swaps_only_after_the_wait`;
  5. `from_save` drops the timer → the new `save_load` test;
  6. one row per shape fixed in Task 4 → `the_cage_sweep_leaves_no_mark_after_16000_ticks`.

  UPDATE `mutations/12-10b.sh`: delete rows 12 and 15 (their seam, the back-out, is gone) and row 18 (an instant swap no longer trips a test; row 1 above pins the wait). Leave a comment naming 13.4 at each. `python3 scripts/audit-mutations.py` must read every row applying.
- [ ] **Task 7: the instrument, RED first (AC 6, 8), and the seat (Q4 a).** The instrument is `simd --bench` (its end line, `crates/simd/src/scenario.rs:334`, and `state=` at `:321`), driven by 13.1's `scripts/tick-bench.sh`, which is tested by `crates/simd/tests/bench.rs` and `mutations/13-1.sh`. Run nothing else at the same time.
  - [ ] **RED:** `main`'s binary on `scenarios/stuck-tunnel.json` ends `marks=3` (creation: C0 `marks=3 … goals=38/4/0`). **Green:** HEAD ends `marks=0`. Then `RUNS=3 scripts/tick-bench.sh main HEAD`: paste the table, count the `differ` cells, and check that each build's runs agree.
  - [ ] Re-measure #207: hand-apply `13-2-signoff/prototypes/census.patch` on a scratch worktree of HEAD if it applies (else port only `census_207_four_bars_round_a_hole` from it). Paste the `census ring` line (creation, B: `stored=18/24 free=6 … last_rise_tick=5989`) for 13.5's ruling.
  - [ ] Seat card, NEW `_bmad-output/implementation-artifacts/13-4-signoff/vehicle-card.md`: push first (`scripts/push.sh --fast`), then a release `simd 7451 --scenario scenarios/stuck-tunnel.json` in WSL (13.1's card: restart both if the gui exits) and `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')` at the seat. The stand-off is in the tunnel x 77, y 64–71, z 10, from about t19,082. Put the camera flag there only after checking it headless (lavapipe, memory: `headless-gui-instruments`). Questions for Wolf: (1) did the two pass and the marks finish? (2) does the pass read acceptably without an animation? Record the answers in the card.
- [ ] **Task 8: gate and close.** Fast tier on every commit (pre-commit hook). ONE full gate at PR: `RUST_TEST_THREADS=1 scripts/gate.sh` (~45–65 min, memory: `gate-ooms-at-default-parallelism`). Paste the block under Debug Log References. Cost row (`python3 _bmad/scripts/session_tokens.py --phase dev --story 13-4-squeeze-past-two-dwarves-trade-tiles`), `phase_state.py clear`, Status → review on the story and the board.

## Verification

```bash
# AC 2-4, 7: lib + scenario + save_load tests (fast tier runs the lib)
cargo test -p sim-core --lib swap
cargo test --release -p sim-core --test save_load
cargo test --release -p sim-core --test scenario -- cage_sweep no_dwarf_is_caged a_stockpile_around_the_campfire head_on staircase
# AC 5: RED at wait 10 before the fix (4 runs), green after; then waits 20 and 30 (throwaway edits)
cargo test --release -p sim-core --test scenario the_cage_sweep_leaves_no_mark_after_16000_ticks -- --nocapture
# AC 6: the instrument. main → marks=3 (RED), HEAD → marks=0
target/release/simd --bench scenarios/stuck-tunnel.json --runs 2
RUNS=3 scripts/tick-bench.sh main HEAD
# AC 2-5: the sabotage table; every row KILLED at its own assert
env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-4.sh
python3 scripts/audit-mutations.py
# Q4: Task 7's seat card (Windows)
```

## Dev Notes

### Scope guardrails (do NOT)

- Do not port `census.patch`, `counters.patch`, `timer.patch` or any `counters` module. A throwaway census stack in a scratch worktree is fine for readings.
- Do not change the wall-in guards, claiming or job logic **except** the Task 4 fix that Q2 (a) admits, and only within Task 3's split point.
- Do not loosen a pin. A pin moves only with a reading that explains it (AC 7). Do not tune `SQUEEZE_WAIT_TICKS` to make a sweep pass; it stays one step period (Ruling 1).
- No gui swap animation, no `protocol` change, no clamber (13.5), no #186 `wander` change.

### What already exists (build on it)

- The blocked-step ladder: `resolve_blocked_step` (`lib.rs:1720`), its one caller in `execute_jobs` (`:2168-2187`; the "wait" branch stores the holder's `Path` and sets the rest), `step_aside` (`:1626`), `make_room_down_the_idle_chain` (`:1660`). The prototype's swap returns "wait", so the caller's existing branch stores the advanced path; `occupied` needs no update (the same two tiles stay taken).
- Every `Pos` writer and its `occupied` handling is tabled in `13-2-signoff/proto-squeeze.md` § "Every writer of a dwarf's `Pos`". `settle` and `wander` build their own sets, and the swap touches neither.
- `Clearing` / `SavedClearing` (`lib.rs:303`, `save.rs:52-67`, `lib.rs:2851`, `:2966`) is the per-dwarf saved-component pattern to copy, and `a_world_saved_at_every_tick_of_a_clear_loads_and_steps_identically` (`save_load.rs:855`) is the test to copy.
- Fixtures: `solid_world`, `carve`, `crew_at`, `room_and_tunnel`, `watch`, `head_on` (`lib.rs:6859-6971`); `stand_miners_at`, `insert_dig` (`:5235`, `:5251`); `dwarf_entity` (`:3827`).
- 13.2 evidence: `proto-squeeze.md`, `census-squeeze.txt`, `census-16k.txt`, `census-squeeze-16k.txt`; 13.4's: `13-4-signoff/variant-n.patch`, `13-4-signoff/probe187.rs`.

### Key decisions & traps

- **The swap returns "wait" (`false`).** `try_squeeze` has already advanced the holder's `path`, and the caller stores it with the rest period. Returning `true` would step him a second tile.
- **`World::step` has a debug-only shared-tile assert** that shadows `step_and_watch`'s in debug runs (`12-10b.sh` row 19 comment). Row 2 of the new table must die at its own assert: check the KILLED line, and compile the debug_assert out in the row as row 19 does if needed.
- **The 16k sweep's clean reading can be a timing draw.** At creation, waits 10/20/30 left 4/2/1 *different* runs stuck. AC 5 therefore reads three waits. A fix that clears wait 10 only has moved the lottery, not fixed the shape.
- **The cage `frozen` list reshuffles under any timing change.** B adds `(2,6,1)`, `(7,6,3)` and `(8,6,4)` and drops `(10,6,0)` and `(12,4,1)` (13.2: "2 kept, 2 gone, 3 new"). Each entry gets the #186 reading (`scenario.rs:2965-2987` style). It is not this story's to fix (#186 is `wander`).
- **The timer's `last` window matters.** The prototype keeps a spell alive while `now - last <= 2 * STEP_REST_TICKS`, so a holder who looked away for one period keeps his count. Keep it: the measured numbers rest on it.
- **No-swap text lives in comments too:** `resolve_blocked_step`'s NOTE, `watch`'s doc, `step_and_watch`'s doc, the head-on tests' AC3 comments, and `12-10b.sh`'s rows 18–19 comment. Update each one that the swap makes false (memory: `partial-doc-update-immunises`).
- **Build probes in scratch worktrees, built from inside them** (memory: `cargo-target-dir-builds-cwd-sources`, `mutation-run-poisons-concurrent-builds`), with `CARGO_BUILD_JOBS` ≤ 10 (`build-parallelism-ooms-the-devpod`). A throwaway commit there needs `--no-verify` (`worktree-commit-runs-the-gate`).

### Previous story intelligence

- 13.3: 13.2's patches are C0-stack artefacts. Port them, and re-express their REDs as mutation rows. `census.sh` and `scale.sh` hardcode `BASE=4f57027`, so do not use them on this branch.
- 13.3: a mutation anchor in an older table can run into new code (`12-9.sh` row 20). Here that is `12-10b.sh` rows 12, 15 and 18 (Task 6). Run `audit-mutations.py` before the first commit.
- **#189 and #206 are CLOSED** by PR #216's prose. Whether to reopen them before this PR is still Wolf's open call (since 13.3). The PR body uses `Closes`/`Fixes` only where Wolf says (memory: `closing-keyword-closes-whole-issue`), and #187 closes only if Task 4 fixes its shape and Wolf agrees.

### Project Structure Notes

- UPDATE `crates/sim-core/src/lib.rs` (constants, `Squeeze` component, `try_squeeze`, `resolve_blocked_step`, `to_save`/`from_save`, the Task 4 fix, the test module)
- UPDATE `crates/sim-core/src/save.rs` (`SavedDwarf.squeeze`, `SavedSqueeze`)
- UPDATE `crates/sim-core/tests/scenario.rs` (`step_and_watch`, the campfire pin, the cage sweep helper + 16k test + re-pins)
- UPDATE `crates/sim-core/tests/save_load.rs` (the stand-off test)
- NEW `_bmad-output/implementation-artifacts/mutations/13-4.sh`; UPDATE `mutations/12-10b.sh`
- NEW `_bmad-output/implementation-artifacts/13-4-signoff/vehicle-card.md` (Q4 a); `13-4-signoff/variant-n.patch` and `probe187.rs` exist (creation)
- `scenarios/saves/*.save` unchanged under Q3 (a)
- No new dependency.

### References

- `_bmad-output/planning-artifacts/epics.md` Story 13.4; Epic 13 and Epic 12 standing ACs (`:2434-2446`, `:2825`)
- `_bmad-output/implementation-artifacts/13-2-signoff/proposal.md` § Prototypes, § Issues, § Rulings needed item 1, § Recommendation item 2, § Ruling
- `_bmad-output/implementation-artifacts/13-2-signoff/proto-squeeze.md`; `prototypes/squeeze.patch`, `census.patch`, `counters.patch`, `census-16k.patch`
- `_bmad-output/implementation-artifacts/13-2-the-way-through-pathfinding-and-reachability-spike.md` Review Findings (decision 1; the unsaved-timer defer)
- `crates/sim-core/src/lib.rs:44`, `:303-311`, `:1624-1859`, `:2168-2187`, `:2851`, `:2966`, `:6911-7067`, `:7100`, `:7334`
- `crates/sim-core/src/save.rs:34-67`; `crates/sim-core/tests/save_load.rs:652`, `:855`
- `crates/sim-core/tests/scenario.rs:34-133`, `:2940-3133`, `:4743-4895`
- `crates/simd/src/scenario.rs:248`, `:321`, `:334`; `scripts/tick-bench.sh`; `scripts/audit-mutations.py`
- `_bmad-output/implementation-artifacts/mutations/12-10b.sh` rows 12, 15, 18, 19
- GitHub #187 (open), #189 and #206 (closed by #216's prose), #207 (open), #186 (open)

## Dev Agent Record

### Agent Model Used

Implementer: Sonnet 5.5 subagents (`claude-sonnet-5-5`), Opus 5.5 orchestrating (Wolf, 2026-10-26).

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-10-26 | Implementer: Wolf picked Sonnet 5.5 subagents, Opus orchestrating. |
| 2026-10-26 | Task 0 ruled by Wolf: Q1 (a) allow the idle-blocker swap; Q2 (a) fold the #187 fix in (16k sweep clean at waits 10/20/30, split point after Task 3); Q3 (a) plain `Option` timer field, old saves load; Q4 (a) short seat sitting. No AC amended. |
| 2026-10-26 | Story created (Opus 5.5) from Epic 13 Story 13.4 and the 13.2 proposal. Creation probe on HEAD `380fd789`: 13.2's squeeze numbers reproduce exactly; the idle-blocker swap (Q1) moves only #207; the 4 stuck cage runs are #187's refused cut and a stone-walled mark with the pile full, landed by timing (waits 10/20/30 → 4/2/1 different runs) (Q2). Task 0 Q1–Q4 await Wolf. Status ready-for-dev. |
