---
baseline_commit: c5164d93
model: claude-opus-5-5  # session default (story authored on Opus 5.5)
---

# Story 13.4: Squeeze Past — Two Dwarves Trade Tiles

Status: in-progress

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

1. **The swap rule (Ruling 1, variant B; the mechanism is the ruling).** When a holder's next tile is held by a dwarf the ladder cannot clear, and the same dwarf has held him for `SQUEEZE_WAIT_TICKS` (10, one step period), the two trade tiles in one write. The swap applies to two head-on job-holders, which swap instead of #196's yield, and (Q1 a) to an idle blocker that has not moved off for the wait, exit or not, whose `home` becomes the holder's old tile and whose exit `Path` is dropped. A job-holder who is not coming the other way is never swapped. Both rest `STEP_REST_TICKS` after it. **AMENDED 2026-10-26 (code review run 1, decision 1 (a), Wolf "1"):** the idle blocker with an exit that runs ahead along the holder's path is swapped too (variant B's own site, `resolve_blocked_step`'s `!head_on && idle` branch), pinned by `an_idle_blocker_with_an_exit_ahead_swaps_after_the_wait`. *Was:* "an idle blocker with no tile aside and no exit".
2. **One dwarf per tile at every tick boundary.** `watch` (`lib.rs:6911`) and `step_and_watch` (`scenario.rs:4743`) still fail on a shared tile and no longer fail on a swap. A sabotage that writes only one of the two positions fails them.
3. **The wait is real.** A lib test drives `resolve_blocked_step` on a head-on pair: no swap on the first look, then a swap once the wait has run. A swap on the first look fails it.
4. **The timer is saved.** A `save_load` test saves at every tick of a stand-off that ends in a swap, loads each save, and steps it identically to the never-saved world (dwarves, claims, paths). Dropping the timer on load fails it. Old saves load under Q3's answer.
5. **AMENDED 2026-10-26 (Wolf, at Task 3's split point: "land squeeze, pin 16k"):** the committed cage sweep runs each run to 16,000 ticks and PINS the marks left there (`UNFINISHED_LATE` = the four wait-10 runs, each with its traced shape), with no stone landed on a dwarf. Waits 20 and 30 are recorded, not asserted. The shapes are filed as #218, and #187 has a comment. *Was:* **#187 is cleared (Q2 a).** A committed 16,000-tick cage sweep reads no run with marks left and no stone landed on a dwarf. The same sweep, with throwaway edits of `SQUEEZE_WAIT_TICKS` to 20 and to 30, also reads no unfinished run.
6. **The reproducers.** `simd --bench scenarios/stuck-tunnel.json` ends `marks=0` (`main`: 3). The campfire test is restored to 0 pick-ups after t2500 and every zone cell filled, the two figures its #189 comment names (`scenario.rs:114-131`). #207's ring is re-measured on top of the swap and recorded for 13.5's ruling.
7. **Every moved pin is explained, none is loosened.** The two head-on lib tests assert that a swap resolved the meeting, both jobs finished, and nobody was released. Each new or removed `ONE_LEVEL_ISLANDS` entry and each 4,000-tick `UNFINISHED` entry carries a read-by-dump comment in the house style (`scenario.rs:2950-3006`). An `UNFINISHED` entry must be shown to finish by 16,000 ticks, **or**, if it does not, be one of the 16,000-tick `UNFINISHED_LATE` pins with its traced shape. **AMENDED 2026-10-26 (code review run 1), under Wolf's "land squeeze, pin 16k" (AC 5):** 4 of the 5 `UNFINISHED` entries are `UNFINISHED_LATE` and never finish; only (12,6,6) does. *Was:* "An `UNFINISHED` entry must be shown to finish by 16,000 ticks."
8. **Same run twice, same state.** `scripts/tick-bench.sh main HEAD` reads two identical runs per build, and `differ` against `main` on the scenarios the swap changes. The story records the table, stuck-tunnel's cost included (13.2: −71%, the livelock gone).

## Tasks / Subtasks

- [x] **Task 0: Wolf's rulings (Q1–Q4).** Record the answers in the Change Log and amend the ACs they name. Then ASK which implementer before dev (memory: `dev-implementer-is-sonnet-not-codex`).
- [x] **Task 1: port the swap (AC 1).** UPDATE `crates/sim-core/src/lib.rs`. Port `13-2-signoff/prototypes/squeeze.patch` by hand: it is a C0-stack patch, and its back-out hunk carries `counters::at`, which `main` lacks. Hunks 1–5, 7 and 8 apply on HEAD with offsets; hunk 6 is ported by hand (creation did exactly this).
  - [x] **Done means B's readings reproduce** (the timer as a component must not change an outcome): `simd --bench` ends stuck-tunnel `tick=28082 marks=0 items=60 zones=89 goals=38/4/1`, big-pile `marks=12 items=34 … goals=0/1/0`, dig-loop `marks=10 items=44`, seat-slow `marks=1 items=129 … goals=38/11/1`; the 4,000-tick cage `frozen` reads `[(2,6,1),(7,6,3),(8,6,4),(12,4,4),(12,6,3)]`. Compare end lines, not `state=`: the hash covers the save encoding, which gains the new field.
  - [x] `SQUEEZE_WAIT_TICKS` beside `STEP_REST_TICKS` (`:44`). Drop `SQUEEZE_HEAD_ON_MOVERS`: variant B is ruled, so no switch (YAGNI).
  - [x] Replace the `Squeeze` resource with a per-holder component, `Squeeze { blocker: Id, since: u64, last: u64 }`, beside `Clearing` (`:303-308`). It is set by the first blocked look at a blocker and kept while the same blocker holds him within `2 * STEP_REST_TICKS` of the last look; it is removed by the swap. It is saved per dwarf (Task 2), so save and load do not need a resource. Keep the prototype's `(who, first, last)` semantics.
  - [x] `try_squeeze` and its four call sites in `resolve_blocked_step` (`:1720-1859`). No `counters::add`. The back-out (`:1818-1829`) is deleted; with Q1 (b) it stays for an idle blocker.
  - [x] Rewrite `resolve_blocked_step`'s doc comment (`:1697-1719`): item 3 gains the swap, the #194/#196 back-out paragraph goes, and the closing `NOTE: no swap anywhere (Wolf, Task 0 Q1b)` becomes the rule: the swap, its wait, and who never swaps. Fix `make_room_down_the_idle_chain`'s "No swap" line (`:1659`) and `an_idle_dwarf_on_the_dig_face…`'s "NO swap" comment (`:7100-7102`) only if they become false.
- [x] **Task 2: save the timer (AC 4).** UPDATE `crates/sim-core/src/save.rs`: `SavedDwarf.squeeze: Option<SavedSqueeze>` with `SavedSqueeze { blocker: u32, since: u64, last: u64 }`, under Q3's answer. UPDATE `to_save` (`lib.rs:2851`) and `from_save` (`:2966`) as `clearing` does. The state hash (`simd/src/scenario.rs:248`, FNV over the save encoding) then covers it.
  - [x] NEW test in `crates/sim-core/tests/save_load.rs`, copying `a_world_saved_at_every_tick_of_a_clear_loads_and_steps_identically` (`:855`). Use the head-on tunnel fixture from `save_load_keeps_the_exit_path_of_an_idle_dwarf_leaving_a_dead_end` (`:652`) or the lib `head_on`, and assert a swap happened in the window (vacuity).
- [x] **Task 3: diagnose the stuck runs, then stop if needed (AC 5; Q2 a).** On the branch after Task 1, with the cage sweep run to 16,000 ticks (Task 4's test), trace every unfinished run at waits 10, 20 and 30: why the cut is refused (which guard: `walls_in_a_dwarf`, `closes_last_level_exit`, the #195 dwarf-on-target refusal), or why no claim finds a path. `13-4-signoff/probe187.rs` is a ready starting point. Record the shape per run in the Debug Log. **Split point:** if the shapes are more than the two in Q2, or the fix reaches into claiming beyond them, stop and ask Wolf before writing the fix.
- [x] **Task 4: fix the shapes, and keep the 16k sweep (AC 5).** **AMENDED (Wolf 2026-10-26): no fix. The 16k sweep is kept and pins the four wait-10 runs; the shapes are on #218.** *Original text:* UPDATE `crates/sim-core/src/lib.rs` (the fix) and `crates/sim-core/tests/scenario.rs`. Run the cage sweep (`:2947-3133`) to 16,000 ticks **once**, reading the 4,000-tick pins (`frozen`, `UNFINISHED`) on the way and asserting at 16,000 that `unfinished` and `landed` are empty. Name it so the Verification filter `the_cage_sweep_leaves_no_mark_after_16000_ticks` (or `cage_sweep`) selects it. Cost: the 16k sweep took **333 s in a debug build** at creation (C0, one test, light load), against about a quarter of that for today's 4,000-tick sweep, so one run that reads both costs the full gate about 4 minutes more; two separate sweeps would cost about 5. Then run it with `SQUEEZE_WAIT_TICKS` edited to 20 and to 30 (throwaway; paste both readings). Before the fix it must be RED at wait 10 with the 4 runs above; paste that.
- [x] **Task 5: rewrite the no-swap pins (AC 2, 7).** UPDATE `lib.rs` tests and `tests/scenario.rs`.
  - [x] `watch` (`lib.rs:6911-6946`) and `step_and_watch` (`scenario.rs:4743-4773`): delete the swap assert and keep the shared-tile assert. Update both doc comments ("or swap tiles", "no swap, 12.9 Task 0 Q1b").
  - [x] `head_on_in_a_tunnel_resolves_when_the_lower_id_is_nearer_the_{open,dead}_end` (`:6973-7006`): replace the "backed out" assert with "a swap of dwarves 0 and 1 between two consecutive trace rows" (both jobs done and nobody released, as now). Rewrite the 12.9 AC3 comments to the new rule.
  - [x] NEW lib test, beside `a_head_on_with_no_escape_drops_the_idle_blockers_stale_path` (`:7012`): `a_held_dwarf_swaps_only_after_the_wait` (AC 3), on a head-on job-holder pair: first `resolve_blocked_step` call → `false`, positions unchanged; tick advanced by `SQUEEZE_WAIT_TICKS`, second call → positions traded, `occupied` unchanged, the blocker's `Path` advanced. With Q1 (a), a second case: an idle blocker with no way out swaps and his `home` is the holder's old tile.
  - [x] Campfire test (`scenario.rs:114-131`): restore `pickups_after_full == 0` and `filled.len() == zones.len()`, and replace the #189 jam comment with the 13.4 fix.
  - [x] Cage sweep pins: re-read `ONE_LEVEL_ISLANDS` and `UNFINISHED` after Tasks 1 and 4 and re-pin per AC 7. Every entry gets its reading, as the existing comments do.
- [x] **Task 6: mutation table (AC 2, 3, 4, 5).** NEW `_bmad-output/implementation-artifacts/mutations/13-4.sh` in `13-3.sh`'s shape. Run it alone, after commit: `env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-4.sh`. Every row must be KILLED at its own assert:
  1. the wait is 0 (swap on the first look) → `a_held_dwarf_swaps_only_after_the_wait`;
  2. `try_squeeze` writes the blocker's `Pos` but not the holder's (13.2's shares=64 sabotage) → a `watch` or `step_and_watch` shared-tile assert;
  3. head-on job-holders never swap (the call removed) → a rewritten head-on test;
  4. (Q1 a) the idle blocker's `home` is not moved, or it never swaps → the idle case of `a_held_dwarf_swaps_only_after_the_wait`;
  5. `from_save` drops the timer → the new `save_load` test;
  6. ~~one row per shape fixed in Task 4~~ none: no shape is fixed (AC 5 amended).

  UPDATE `mutations/12-10b.sh`: delete rows 12 and 15 (their seam, the back-out, is gone) and row 18 (an instant swap no longer trips a test; row 1 above pins the wait). Leave a comment naming 13.4 at each. `python3 scripts/audit-mutations.py` must read every row applying.
- [x] **Task 7: the instrument, RED first (AC 6, 8), and the seat (Q4 a).** The instrument is `simd --bench` (its end line, `crates/simd/src/scenario.rs:334`, and `state=` at `:321`), driven by 13.1's `scripts/tick-bench.sh`, which is tested by `crates/simd/tests/bench.rs` and `mutations/13-1.sh`. Run nothing else at the same time.
  - [x] **RED:** `main`'s binary on `scenarios/stuck-tunnel.json` ends `marks=3` (creation: C0 `marks=3 … goals=38/4/0`). **Green:** HEAD ends `marks=0`. Then `RUNS=3 scripts/tick-bench.sh main HEAD`: paste the table, count the `differ` cells, and check that each build's runs agree.
  - [x] Re-measure #207: hand-apply `13-2-signoff/prototypes/census.patch` on a scratch worktree of HEAD if it applies (else port only `census_207_four_bars_round_a_hole` from it). Paste the `census ring` line (creation, B: `stored=18/24 free=6 … last_rise_tick=5989`) for 13.5's ruling.
  - [x] Seat card, NEW `_bmad-output/implementation-artifacts/13-4-signoff/vehicle-card.md`: push first (`scripts/push.sh --fast`), then a release `simd 7451 --scenario scenarios/stuck-tunnel.json` in WSL (13.1's card: restart both if the gui exits) and `.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')` at the seat. The stand-off is in the tunnel x 77, y 64–71, z 10, from about t19,082. Put the camera flag there only after checking it headless (lavapipe, memory: `headless-gui-instruments`). Questions for Wolf: (1) did the two pass and the marks finish? (2) does the pass read acceptably without an animation? Record the answers in the card.
- [x] **Task 8: gate and close.** Fast tier on every commit (pre-commit hook). ONE full gate at PR: `RUST_TEST_THREADS=1 scripts/gate.sh` (~45–65 min, memory: `gate-ooms-at-default-parallelism`). Paste the block under Debug Log References. Cost row (`python3 _bmad/scripts/session_tokens.py --phase dev --story 13-4-squeeze-past-two-dwarves-trade-tiles`), `phase_state.py clear`, Status → review on the story and the board.

### Review Findings

Code review run 1 (2026-10-26, a fresh session), on `c5164d93..8ac4deb8`. Four layers, each ran `cargo 1.97.1` in its own `CARGO_TARGET_DIR` on trees extracted with `git archive`; none timed out or failed. Territories: Blind Hunter (Sonnet) `crates/sim-core/src`; Edge Case Hunter (Sonnet) `crates/sim-core/tests` + `mutations/`; Acceptance and Feature Auditors (Opus) the whole diff. Nothing under `/workspace` was modified.

What the layers re-ran: all 8 `13-4.sh` rows, 12-10b row 6 and 12-9 row 5, applied by hand on copies, each KILLED at its recorded assert (acceptance; edge re-ran rows 1b, 2, 12-10b 6, 12-9 5). `audit-mutations.py`: 959 rows apply (edge + acceptance). stuck-tunnel `--bench --runs 2`: HEAD `marks=0 … goals=38/4/1` `state=a5d257cac0182a1f`, `main` `marks=3 … goals=38/4/0` `state=8365ab4d3aafed06` (feature + acceptance, independently); the seat copy reads the same state, so the run Wolf sat is the run HEAD makes. e9712e5f: identical `state=` on all 10 scenarios parent vs HEAD (acceptance). Live daemon (feature, raw TCP reader, `simd --scenario stuck-tunnel.json`): ticks 18138-19300 all delivered, the 7 swaps each trade two positions inside one delta, no tick shows a shared tile; a `save` sent at t18138 with both dwarves mid-stand-off reloads and steps to t19300 byte-identical (`cmp`), and the same save with `squeeze` nulled loads but diverges (first swap t18143 → t18154). Swaps per call site (feature, instrumented): the head-on job-holder site does 6-14 per scenario, the idle no-exit site 1 each on stuck-tunnel and dig-loop, the other two sites never swap in play. `cargo test --release -p sim-core` 227 passed; lib 101 passed.

- [x] [Review][Decision] An idle blocker that HAS an exit can be swapped (feature + acceptance + edge; severity MED) — `lib.rs:1856-1866`: when the idle blocker's new exit `Path` does not start on the holder's tile (it runs ahead along the holder's path), the `!head_on && idle` branch calls `try_squeeze(idle=true)`. AC 1 and `resolve_blocked_step`'s NOTE say the idle swap is only for a blocker "with no tile aside and no exit". It is `squeeze.patch`'s own site (variant B is the ruling), and it swapped 0 times in the 10 scenarios plus the cage, campfire, staircase and head-on tests (313 swaps measured). Its `idle ||` Path removal (`lib.rs:1719`) is load-bearing only here (a kept `route[1..]` would not start next to the blocker's new tile) and is unpinned: dropping `idle ||` SURVIVED lib, save_load and the full scenario suite (edge, release). Options: (a) keep the site as ported; amend AC 1 and the NOTE to say "an idle blocker that has not moved off for the wait, exit or not"; add a lib case with an idle blocker on a multi-tile exit `Path` that is gone after the swap, and a `13-4.sh` row dropping `idle ||`; (b) remove the call, so an idle blocker with an exit is left to walk it (the holder waits); re-read every `--bench` end line and the cage pins, since it departs from B.
  **Resolved 2026-10-26: (a)** (Wolf: "1"). Becomes the patch below.
- [x] [Review][Patch] Pin the idle-with-an-exit swap and amend its wording (decision 1 (a); MED) [crates/sim-core/src/lib.rs:1719 `idle ||`, :1856-1866, `resolve_blocked_step` NOTE; story § AC 1; mutations/13-4.sh] — AC 1 and the NOTE say "an idle blocker that has not moved off for the wait, exit or not"; a lib case gives an idle blocker a multi-tile exit `Path` that runs ahead along the holder's path, waits him out, and asserts the swap and that the blocker's `Path` is gone; a `13-4.sh` row dropping `idle ||` is KILLED at that assert.
- [x] [Review][Patch] The stand-off save test cannot see a wrong saved blocker id (edge; MED, latent silent failure) [crates/sim-core/tests/save_load.rs:931-1062; lib.rs to_save `blocker: squeeze.blocker.0`] — sabotage `to_save` writes `blocker: 0` SURVIVED lib + save_load (edge, RAN). Both dwarves time each other, so one side's corrupted timer restarts while the other side's still swaps on time; and the loop compares dwarves, claims, jobs and paths, never `squeeze`. Fix: in the window, assert every saved `squeeze` names the other dwarf's id (an independent oracle, not the save's own output); add a `13-4.sh` row `to_save writes blocker 0`, KILLED at that assert.
- [x] [Review][Patch] The record says the swap does not reach 5 scenarios; it reaches all 10 (acceptance + feature; MED, a wrong instrument reading) [story § Debug Log, Task 7 bullet "SAME on full-pile-crowd and all four plan scenarios"; `13-4-signoff/tick-bench-main-HEAD.txt` reading] — measured: with `squeeze` skipped when `None` (so the encoding equals `main`'s), `state=` still differs on all 10; on plan, full-pile-crowd and plan-{compact,far,spread} every one of 5 dwarves ends on a different tile, 1-34 items differ, swaps 9/2/14/9/9. End lines match only because they are coarse. So the hash difference is behaviour as well as encoding, and "plan +6.3% BEYOND" is partly a behaviour change. Correct the bullet and the reading; state end lines as a summary, not as reach.
- [x] [Review][Patch] AC 7's last clause and epics.md's 13.4 AC were not amended with AC 5 (acceptance; MED) [story § AC 7; `_bmad-output/planning-artifacts/epics.md` Story 13.4] — AC 7 still says "An `UNFINISHED` entry must be shown to finish by 16,000 ticks"; 4 of the 5 `UNFINISHED` entries are `UNFINISHED_LATE` and never finish (only (12,6,6) does). epics.md still says "the #187 regression is cleared before the story lands". Wolf's "land squeeze, pin 16k" covers both in substance; amend the text with an AMENDED line naming the ruling, as AC 5 does.
- [x] [Review][Patch] Record text stale after e9712e5f and the gate (acceptance + feature; LOW, same file and lines this record touches) [story § Completion Notes "its #194 `or_else` branch may now be dead (kept; Wolf's call)"; § Debug Log 12-10b row 11 line "The `or_else` #194 branch … is kept"; `sprint-status.yaml` 13.4 DEV DONE comment (names only the `1aeb4bf8` gate); `mutations/12-10b.sh` row 6 payload comment "tried inside the exit's `or_else`"; § Verification filters `the_cage_sweep_leaves_no_mark_after_16000_ticks` / `cage_sweep`, which select no test (kept name `no_dwarf_is_caged_by_stones_over_the_probe_seeds`); Task 7 checkbox `[ ]` with every subtask `[x]`; Dev Notes "a holder who looked away for one period keeps his count" (a blocked dwarf looks every 11 ticks, all 63 measured swaps read `waited=11`, so a missed look is a 22-tick gap and exceeds the 20-tick window; behaviour is B's, only the sentence is wrong); e9712e5f's "dead": the branch was reachable, 6 hits (4 staircase, 2 stuck-tunnel), each landing in `(None, None)` exactly as HEAD does, so it was redundant, and neutrality is measured, not general].
- [x] [Review][Defer] A `Squeeze` whose `since`/`last` is ahead of the tick panics in debug [crates/sim-core/src/lib.rs:1694, :1700] — deferred, unreachable in play (blind, RAN): `now - squeeze.last` overflows ("attempt to subtract with overflow"); release wraps and restarts the wait. Only a hand-edited save reaches it; `from_save` copies the fields unchecked. Fix if ever needed: `saturating_sub`.
- [x] [Review][Defer] A leftover `Squeeze` is reused if the same pair meets again within 20 ticks [crates/sim-core/src/lib.rs:1689-1697] — deferred, never observed (blind read + feature RAN): the component is removed only by the holder's swap, so the partner keeps his and a stand-off that ends unswapped keeps the holder's; a re-meeting within `2 * STEP_REST_TICKS` of `last` inherits `since` and swaps early. Every one of 63 swaps in the 10 scenarios waited 11 ticks. The record's "the `last` window ignores it" is true only past 20 ticks.
- [x] [Review][Defer] The #185 chain shift and the squeeze race on long idle chains [crates/sim-core/src/lib.rs:1836-1850] — deferred, B's semantics (blind, read): each look gives the chain its exit, and the swap at the second look drops the idle blocker's `Path` even if `make_room` had just sent him. Not introduced by e9712e5f (before it, a `make_room` true also fell to `try_squeeze`). Pinned only by `tile_trades <= 2`.
- [x] [Review][Defer] `try_squeeze` finds the holder by an O(n) `iter_entities()` scan with no `Dwarf` filter [crates/sim-core/src/lib.rs:1679-1683] — deferred (blind, read): correct while ids share one allocator; the caller has the entity and could pass it.
- [x] [Review][Defer] The saved `since` and `last` are not told apart by any test [crates/sim-core/tests/save_load.rs:931] — deferred, observationally equivalent today (edge, RAN: four field-swap sabotages SURVIVED): a stand-off has two looks, so `since == last` until the swap.
- [x] [Review][Defer] The wait length is pinned only by a test that reads the constant [crates/sim-core/src/lib.rs:7233, :7290] — deferred (edge, RAN): `SQUEEZE_WAIT_TICKS = STEP_REST_TICKS + 1` SURVIVED lib and scenario; `<`→`<=` and `+1` in the comparison are KILLED. Ruling 1 fixes the value, so a change is a visible edit.
- [x] [Review][Defer] The stale-timer window and the blocker-id check are killed only by the cage sweep [crates/sim-core/tests/scenario.rs:3171, :3180] — deferred (edge, RAN): `100 *` window and a dropped `blocker == blocker_id` survive lib + save_load and die at the cage pins; `13-4.sh` has no row for either.
- [x] [Review][Defer] No mutation row pins AC 2's one-write sabotage against `step_and_watch` [mutations/13-4.sh row 2] — deferred, behaviour met (acceptance, RAN): row 2's payload fails `step_and_watch` at `scenario.rs:4813` ("two dwarves share a tile at tick 541/761"), but the committed row selects only the lib `watch` test.
- [x] [Review][Defer] The `UNFINISHED_LATE` comment cites traces that hold only 2 of its 4 runs [crates/sim-core/tests/scenario.rs `UNFINISHED_LATE` comment; `13-4-signoff/diag-traces-waits-10-20-30.txt`] — deferred (acceptance, RAN): the file has seeds 0/6 (w10), 0/12 (w20), 4 (w30); seeds 9 and 13 (shape B) are absent. Re-run `probe187-extended.rs CASES=9:6,13:6` on HEAD confirms shape B (all 5 idle, `claim=None`, 0 claim changes).

Cost: review row $20.25 (336 turns; 4 subagents 75.5% of tokens). Reap after triage: 16 /tmp directories, 18.3 GB (13.1 GB free space reclaimed). Wolf chose "leave as action items": next is `bmad-review-patch` in a fresh session; patches 1-2 are test code, so a round-2 review of that diff may follow.

**Review-patch run 1 (2026-10-26, a fresh session, Opus 5.5): 5/5 landed.** `78bf1d41` (patch 1), `c906be17` (patch 2), `b0c82211` (patch 3), `e972ccba` (patch 4), `a0ae8a23` (patch 5). Fast gate green on each commit; FULL GATE GREEN on `a0ae8a23` (4140 s, Debug Log). New mutation rows, run alone after commit on `c906be17`: **4c** KILLED at `lib.rs:7350` ("the swapped idle blocker kept his exit path"); **5c** KILLED at `save_load.rs:1041` ("dwarf 0 saved a stand-off timer naming the wrong blocker at tick 112"). Exclusivity: the focused rows only; the full gate ran the unmutated suite green.

Closure table (none REWORK: every row is round 1's first closure):

| Item | Fix written for | Side then tested | Pre-existing-state fixture |
|---|---|---|---|
| 1 Pin the idle-with-an-exit swap | an idle blocker whose exit runs AHEAD along the holder's path (`!head_on && idle`, `try_squeeze(idle=true)` with a 2-tile `Path`) | the same site positively (first look: exit `[x15, x16]` derived and kept, nobody moves; after the wait: traded, `home` = holder's old tile, `Path` gone) and the no-exit side still green (the idle case of `a_held_dwarf_swaps_only_after_the_wait`, a 1-tile `Path`) | `room_and_tunnel` with dwarves at x 13/14/12/11/10 and dwarf 1 an idle woodcutter with NO `Path` and no `Squeeze` (the world before any stand-off; the code derives the exit), in `an_idle_blocker_with_an_exit_ahead_swaps_after_the_wait`; row 4c |
| 2 Saved blocker id | `to_save`'s `blocker` | the save's output against the fixture's ids (an independent oracle: the other dwarf is `1 - id`), at every tick of the window; the load side stays on rows 5/5b | `fixture()` in `a_world_saved_at_every_tick_of_a_stand_off_loads_and_steps_identically`: the hand-built two-miner tunnel save, both `squeeze: None` (pre-stand-off); row 5c killed on dwarf 0, so both sides do time each other |
| 3 Swap reaches all 10 | the record's claim | the corrected bullet against the review's measurement (squeeze skipped when `None`, all 10 differ; 9/2/14/9/9 swaps), cited as the review's, not re-run here; plan's +6.3% now read as partly behaviour | `13-4-signoff/tick-bench-main-HEAD.txt` (the pre-existing table, `main` `c5164d93` vs `9d6810d9`), note appended |
| 4 AC 7 + epics.md | the AC text | against the pins: `UNFINISHED` (`scenario.rs:3033`) has 5 entries, `UNFINISHED_LATE` (`:3054`) 4, only (12,6,6) finishes | AC 5's AMENDED line and the committed cage sweep |
| 5 Stale record text | each named line | each against source/history: the cage test name exists; 12-10b row 6's anchor is unchanged (comment only; the gate's audit applies it); e9712e5f's 6 hits are the review's count, cited | the record as it stood at `7a1cd12e` |

Dismissed (6): head-on job-holders now wait and swap even where an escape exists (blind; AC 1 rules it); `try_squeeze`'s return value is behaviourally redundant (blind; its doc names the "wait" contract, and the `(None, None)` arm's skipped Path removal is a no-op); `SavedSqueeze.blocker` unchecked on load (blind; a stale id never matches, harmless); a 100+ char doc line (edge; cosmetic); the gui pass reads as walking through (feature; #219, Q4 ruled); #189 shown by the scenario test only (feature; AC 6 names that test).

**Code review run 2 (2026-10-27, a fresh session), on `7a1cd12e..2f050a07` (review-patch run 1).** Four layers, each ran `cargo 1.97.1` in its own `CARGO_TARGET_DIR` on a `git archive` copy; none timed out or failed. Territories as run 1: Blind Hunter (Sonnet) `lib.rs`; Edge Case Hunter (Sonnet) `save_load.rs` + `mutations/`; Acceptance and Feature Auditors (Opus) the whole diff. Nothing under `/workspace` was modified by a layer.

Delta vs run 1: **NEW 10** (0 HIGH, 2 MED, 8 LOW), **REWORK 1** (closure-table row 2), dismissed 4. No NEW finding is HIGH, so by the round-over-round rule **the static audit ends here**: the next spend is the live gate (Q4's seat card), not a round 3.

What the layers re-ran: rows 4c and 5c KILLED at `lib.rs:7350` and `save_load.rs:1041` as recorded (edge + acceptance); option (b) (the `!head_on && idle` `try_squeeze` call removed) KILLED by the new test at its swap assert, and only by it (acceptance); `idle=true`→`false` at that site KILLED at the `home` assert (blind); `audit-mutations.py` 961 rows match; 12-10b row 6's anchor occurs once and its new comment is true (`lib.rs:1845`); swaps per scenario 9/2/14/9/9 on plan, full-pile-crowd, plan-{compact,far,spread} and 8/6/2/3/7 on the other five (acceptance, probe build); `7a1cd12e` vs HEAD `state=` identical on all 10 and stuck-tunnel `marks=0 … goals=38/4/1 state=a5d257cac0182a1f` (feature + acceptance). The `!head_on && idle` site (`lib.rs:1861`): reached 4 times in the 10 scenarios and 26 in the scenario + save_load suites, swapped 0 times, every look a first look, the blocker walking his exit within 10 ticks (feature, instrumented).

Closure-table audit: rows 1, 3, 4 CLOSED; row 5 CLOSED but for two lines (patches 6-7); **row 2 HALF-CLOSED (REWORK, patch 2)**.

- [x] [Review][Decision] The only live way into the idle-exit-ahead swap is a timer carried over from a head-on stand-off (feature; MED) — `lib.rs:1775-1777` (case 1), `:1791-1796` (NOTE), story AC 1. They say the two swap after `SQUEEZE_WAIT_TICKS` "if the blocker has not moved off", between "two dwarves that have met the same way". Live (RAN, probe in a copy driven by `World::step` under `watch`): holder dwarf 0 at x13 digging, job-holder dwarf 1 at x14 heading west, three idle woodcutters behind the holder; `CancelDesignation` on dwarf 1's target at ticks 102-111 makes site `:1861` swap at tick 112 on its FIRST look, because the `Squeeze` key is only `blocker` plus the 20-tick `last` window, so the 11 ticks counted while they met head-on (site `:1882`) carry over. The blocker had cooldown 1 and a free exit `[x15, x16]`: he was one tick from walking off. No shared tile, the dig finishes; a player sees "cancel a dwarf's job mid-stand-off and the two trade at once". This is run 1's deferred "leftover `Squeeze` is reused" item, now shown live-reachable. Options: (a) keep B's semantics; amend case 1, the NOTE and AC 1 to say the timer is kept per blocker across a change of case (a cancel turns a head-on job-holder idle), amend Debug Log line 240 (patch 7), and file the carried timer as an issue (`bug` + `route:story`) so it is a measured state rather than a deferred line; (b) restart the timer when the case changes (key `Squeeze` on `idle` too): a behaviour change from variant B, so re-read every `--bench` end line and the cage pins. Recommended: (a).
  **Resolved 2026-10-27: (a)** (Wolf: "1"). The carried timer is filed as #220 (`bug`, `route:story`). Becomes patch 1 below.
- [ ] [Review][Patch] Amend case 1, the NOTE and AC 1: the timer is kept per blocker across a change of case (decision 1 (a); MED) [crates/sim-core/src/lib.rs:1775-1777, :1791-1796; this file § AC 1] — say the wait is counted per blocker within the 20-tick `last` window whatever way the two meet, so a cancel that turns a head-on job-holder idle can swap on the first look (#220). Carries patches 4 and 7 (same comment, same residue line). Doc and record only: no behaviour change, so `state=` must stay identical.
- [ ] [Review][Patch] Row 5c kills only on a leftover timer; the "each side times the other" claim is false (acceptance + edge; MED, REWORK of run-1 patch 2, latent silent failure) [crates/sim-core/tests/save_load.rs:1032-1034 comment; mutations/13-4.sh row 5c; this file's closure-table row 2] — RAN (acceptance, a probe printing each saved `squeeze`): ticks 101-111 only dwarf 1 has a timer `(blocker 0, since 101, last 101)`; at 112, the swap tick, dwarf 0 has `(1, 112, 112)` from his look earlier that tick, dwarf 1 `None`. So run 1's `blocker: 0` survived because 0 is the CORRECT value for the one real timer (the fixture-matches-constant shape), and 5c's kill at "dwarf 0 … tick 112" rides on dwarf 0's leftover: with the partner's `Squeeze` removed at the swap (the obvious defer-2 cleanup), 5c SURVIVES. Load side: `from_save` `Id(0)` survives lib + save_load (edge + acceptance) for the same reason (correct on the real timer, inert on the leftover), so it is not the row to add; `from_save` `Id(1)` is KILLED by the divergence assert (`save_load.rs:~1067`). Fix: re-point 5c to `blocker: 1` (KILLED at `:1041`, dwarf 1, tick 101, RAN by both layers); add row 5d `from_save` writes `Id(1)`; correct the test comment, the 5c comment and closure-table row 2 (mark it REWORK). Both-sides check for the closure row: 5c re-run with the defer-2 cleanup applied on a copy must still KILL.
- [ ] [Review][Patch] Row 4c's comment says the idle-with-an-exit site is the only one with a blocker `Path` longer than 1 (acceptance; LOW, same file as patch 2) [mutations/13-4.sh row 4c comment] — RAN: the scenario suite records one idle head-on `(None, None)` swap with a 2-tile `Path`, which `idle ||` also drops (harmless there; unpinned). Say "the only one a test pins".
- [ ] [Review][Patch] The NOTE's reason for dropping the exit `Path` holds only at the ahead site (acceptance; LOW, same comment as decision 1) [crates/sim-core/src/lib.rs:1793-1794] — "since it no longer starts next to him" is false at the idle head-on site, where `blocker_path[1..]` does start next to his new tile. Reword with decision 1.
- [ ] [Review][Patch] Closure-table row 1 says the no-exit idle case runs on "a 1-tile `Path`" (acceptance; LOW) [this file, closure-table row 1] — RAN: #183 drops that 1-tile head-on `Path` on the first look, and the swap then fires at the no-exit site with no `Path`, so `a_held_dwarf_swaps_only_after_the_wait`'s idle case cannot see `idle ||`; the new test is its only pin (row 4c).
- [ ] [Review][Patch] The story header still says the story "must clear the #187 cage-sweep regression before it lands" (acceptance; LOW) [this file, § Not stacked] — patch 4 amended AC 7 and epics.md, not this sentence. Add the AMENDED pointer as AC 5 does.
- [ ] [Review][Patch] Debug Log "the `last` window ignores it" (acceptance + feature; LOW) [this file, § Debug Log, "Known residue" bullet] — true only past 20 ticks (run 1 defer 2), and decision 1 shows a reuse within 20 ticks is reachable live. Amend with decision 1.
- [x] [Review][Defer] The new test jumps the clock by `SQUEEZE_WAIT_TICKS` in one hop [crates/sim-core/src/lib.rs:7344] — deferred (blind, read): it passes only while the wait is at most the 20-tick `last` window; raising the wait would fail it loudly, not silently.
- [x] [Review][Defer] The new test does not assert the holder's `Squeeze` is gone after the swap [crates/sim-core/src/lib.rs:7344-7351] — deferred (blind, read); `occupied` is also never read back.
- [x] [Review][Defer] The new test's fixture (an idle blocker sitting on a free exit for the whole wait) is not a live shape [crates/sim-core/src/lib.rs:7309-7354] — deferred (feature, RAN: live, the blocker walks it within 10 ticks, 30 of 30 looks); the branch is reachable only by decision 1's carried timer, and the test still pins the live outcome: with `idle ||` removed, the cancel probe shows dwarf 1 jump (13,20)→(16,20) at tick 122, `wander` walking the stale `route[1..]` through the holder. A tick-driven cancel case would pin the real route; it waits on decision 1.

Dismissed (4): no "not yet" look in the new test (blind; the comparison off-by-ones are KILLED by `a_held_dwarf_swaps_only_after_the_wait`, run 1); "the blocker walks off, no swap" unpinned by a lib test (blind; outside the diff, measured live 30 looks / 0 swaps); case 3's doc omits the idle `(None, None)` swap (blind; the NOTE covers it); `1 - d.id` hardcodes the two-dwarf fixture (edge; the comment names it).

Not proven by running in this round: Q4's seat card (Wolf at the seat); a live TCP capture of this diff (`state=` identical instead); the "third dwarf or item on the exit's first tile" route into `:1861` (reasoned, not built); the full gate (run 1 ran it on `a0ae8a23`; this diff since is record text only).

Cost: review row $15.61 (246 turns; 4 subagents 70.6% of tokens). Reap after triage: 14 /tmp directories, 18.4 GB (13.2 GB free space reclaimed). Wolf chose "leave as action items": next is `bmad-review-patch` in a fresh session for the 7 patches. Status → in-progress.

## Verification

```bash
# AC 2-4, 7: lib + scenario + save_load tests (fast tier runs the lib)
cargo test -p sim-core --lib swap
cargo test --release -p sim-core --test save_load
cargo test --release -p sim-core --test scenario -- no_dwarf_is_caged a_stockpile_around_the_campfire head_on staircase
# AC 5 (amended): the cage sweep runs to 16,000 ticks and pins UNFINISHED_LATE; waits 20 and 30 are throwaway edits, recorded
cargo test --release -p sim-core --test scenario no_dwarf_is_caged_by_stones_over_the_probe_seeds -- --nocapture
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
- **The timer's `last` window matters.** The prototype keeps a spell alive while `now - last <= 2 * STEP_REST_TICKS`. A blocked holder looks every 11 ticks (all 63 measured swaps read `waited=11`), so the window covers the gap between two looks; a missed look is a 22-tick gap and restarts the count. Keep it: the measured numbers rest on it. (Corrected at code review run 1; was "a holder who looked away for one period keeps his count".)
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

- **Tasks 1-2 (Sonnet agent A, commits `2aa6afdd`, `064c0605`; verified by Opus on HEAD `064c0605`).** Release `simd --bench --runs 2`, end lines = variant B's exactly, `state` identical across runs: stuck-tunnel `tick=28082 marks=0 items=60 zones=89 goals=38/4/1`; big-pile `tick=5006 marks=12 items=34 zones=518 goals=0/1/0`; dig-loop `tick=17640 marks=10 items=44 zones=21 goals=0/0/0`; seat-slow `tick=56759 marks=1 items=129 zones=222 goals=38/11/1`. The three save-backed scenarios load pre-13.4 saves (no `squeeze` field), so Q3 (a) holds. Cage 4,000-tick `frozen` = `[(2,6,1),(7,6,3),(8,6,4),(12,4,4),(12,6,3)]`, as at creation. `cargo test --release -p sim-core --no-fail-fast` green (agent's run); the fast gate passed on each commit.
- RED for the two new tests was shown by sabotage after the code existed, not before it: `a_held_dwarf_swaps_only_after_the_wait` with the wait multiplied by 0 fails at `lib.rs:7196` (holder's path advanced on the first look); the stand-off save test with `from_save` dropping the timer fails "saves taken mid-stand-off diverged … [(101,112) … (111,112)]". Both are re-run formally as rows 1 and 5 of `mutations/13-4.sh` (Task 6).
- Known residue, kept on purpose (prototype semantics): a `Squeeze` component outlives a stand-off that ends without a swap; the `last` window ignores it, but it is saved and hashed, so it moves `state=` only.
- **Task 3 diagnosis (Sonnet agent B, scratch worktree on `064c0605`; spot-checked by Opus against the traces).** Evidence in `13-4-signoff/`: `sweep16k.patch` (the 16k sweep test, RED), `diag-instrumentation.patch` (throwaway prints), `probe187-extended.rs`, `diag-traces-waits-10-20-30.txt`.
  - **RED at wait 10** (`the_cage_sweep_leaves_no_mark_after_16000_ticks`, release, 22.4 s): `runs with marks left after 16000 ticks … left: [(0, 4, 1), (6, 6, 1), (9, 6, 1), (13, 6, 1)]`. The 4,000-tick pins hold and `landed` is empty over all 16k ticks. **Wait 20:** `[(0,4,1),(12,6,1)]`. **Wait 30:** `[(4,6,1)]`. These are creation's readings; `landed=[]` at every wait.
  - **The only refusal is the channel cut's wall-in check** (`walls_in_a_dwarf_from` at completion -> `retry_claim`, `lib.rs:2529/:2544`): `cut_wall_in=true` on all 70 traced refusals; `closes_last_level_exit` is not on the channel path. The 72-tick cycle is claim -> refused 52 ticks later -> 20-tick cooldown. In every run the pile has no delivery cell (`delivery_cells={}`: full, or its floor cut away by the channel where the pile search overlaps it, seeds 13 w10 and 12 w20), so the stones walling the pit are never hauled.
  - **Four shapes, not Q2's two:**
    - **A1** seed 0 w10: the worker alone is sealed in his 5-cell pit piece (stones 26/28/31/32/33). A* picks the inside goal (68,65,11) over the rim goal (69,64,13).
    - **A2** seed 6 w10, seed 12 w20: the worker plus idle dwarves are sealed in one stone-walled pit (17-20 stones). **Correction to Q2:** on seed 6 the refusal is the wall-in, not #195. Dwarves 0, 1 and 4 each suffice, so dwarf 2 standing on the mark is incidental (counterfactual not run).
    - **A3** seed 0 w20, seed 4 w30: the worker is outside, and an idle dwarf elsewhere sits on a stone-walled island whose only link is the mark's rim (#186 + stones).
    - **B** seeds 9 and 13 w10: nobody ever claims. Every goal is in a sealed piece with the target, every dwarf is outside, and the goals are not empty, so `work_positions`' empty-goals fallback never fires.
  - **Fix sketches, none written:** A1 is ~10-15 lines (drop work cells the cut would seal). A2/A3 need new behaviour: evict an idle dwarf from a sealed piece, or clear multi-stone bands. B needs claim-goal changes in three sites (~25-40 lines). A2, A3 and B share a root, the ruled #180/FR8 never-drop with the pile full. **SPLIT POINT FIRED** (more than two shapes, and the fix reaches claiming): stopped before Task 4 and asked Wolf.
- **Task 4 (amended, Opus).** `sweep16k.patch` applied; the 16k assert pins `UNFINISHED_LATE = [(0,4,1),(6,6,1),(9,6,1),(13,6,1)]` with the traced shapes; the 4,000-tick `UNFINISHED` comment's partly-inferred lines are replaced by a pointer to them. `cargo test --release -p sim-core --test scenario cage_sweep`-equivalent (`no_dwarf_is_caged…`): ok, 22.45 s. Issue #218 filed (`bug`, `route:story`); #187 comment 6412027097.
- **Task 5 (agent A in `2aa6afdd`; cage re-pins finished in Task 4's `3751d336`).** `watch`/`step_and_watch` keep only the shared-tile assert; both head-on tunnel tests assert a swap of dwarves 0 and 1 between consecutive trace rows (`traded_places`), both jobs done, nobody released; `a_held_dwarf_swaps_only_after_the_wait` covers the head-on pair and the idle case (`home` = holder's old tile); the campfire test is back to 0 pick-ups after t2500 and every zone cell filled; `ONE_LEVEL_ISLANDS` and `UNFINISHED` are re-pinned with dump readings (in the test comments). `an_idle_dwarf_on_the_dig_face…`'s "NO swap" comment stays: it is still true (the miner backs out via the head-on escape yield; no trade in its trace).
- **Task 6, `mutations/13-4.sh` on `3751d336`:** 8/8 KILLED, each at its own assert: 1 (wait 0) `lib.rs:7196` the first-look path assert; 1b (stand-off forgotten between looks) `:7208` the post-wait swap; 2 (only the blocker's `Pos` written, `World::step` debug_assert compiled out) `watch`'s "two dwarves share a tile at tick 113"; 3 (head-on job-holders never swap) "dwarves 0 and 1 never swapped" `:7103`; 4b (idle never swaps) `:7259`; 4 (idle `home` not moved) `:7262`; 5 (`from_save` drops the timer) save_load "saves taken mid-stand-off diverged"; 5b (`to_save` drops it) save_load "no stand-off timer ever ran". No row 6: no shape was fixed (AC 5 amended).
- **Older rows the squeeze made inert (found by re-running every row anchored in the blocked-step code; the audit is static and could not see this).** 12-10b rows 6, 11 and 12-9 row 5 SURVIVED on `2e1e9b88`, while 12-10b row 19 and 12-9 rows 6, 10 and 22 still KILLED. Sonnet agent C, `85146b49`: **12-10b row 6** is re-pinned by a trade count (`tile_trades <= 2`: 2 with the chain shift, 3 without, a measured bound); KILLED at `lib.rs:7333` "3 tile trades: the idle chain did not shift". **12-9 row 5** is re-pointed to the new `a_higher_id_miner_yields_to_an_idle_dwarf_at_the_dead_end_when_only_it_has_an_escape` (the dig-face fixture with ids flipped; head-on job-holders now swap before the yield match, so only an idle head-on blocker reaches it); KILLED at `watch`'s "not done by tick" `:7046`, a livelock, not the new `tile_trades == 0` assert. Both were re-run by Opus: KILLED at the same lines. **12-10b row 11 DELETED** (Opus): its exit is still found, 4 times in its test, but with no holder escape the head-on match lands in `(None, None)`, which drops that `Path` and squeezes; with the sabotage applied, all 101 swaps matched HEAD tick for tick. The `or_else` #194 branch in `resolve_blocked_step` was kept here (production code, out of Task 6's scope) and raised with Wolf; it was removed afterwards in `e9712e5f` (entry below).
- **Task 7, the instrument (Opus).** **RED:** `main` `c5164d93`'s release binary on `scenarios/stuck-tunnel.json`, `--runs 2`, ends `tick=28082 marks=3 items=57 zones=89 goals=38/4/0` (state identical across runs). **Green:** HEAD ends `marks=0 … goals=38/4/1`. Every mark is done within 1,000 ticks of the start (end-line bisect: 500 ticks -> `marks=1`, 1000 -> `marks=0`).
  - `RUNS=3 scripts/tick-bench.sh main HEAD` (HEAD `9d6810d9`, exit 0, so each build's runs agree; table in `13-4-signoff/tick-bench-main-HEAD.txt`, load 7.3 before and 3-7 after): stuck-tunnel **-70.6% BEYOND FLOOR** (0.1028 -> 0.0302 ms; 13.2 read -71%); dig-loop -29.6% BEYOND; big-pile **+120.5% BEYOND** (0.0995 -> 0.2193); deep-pit +10.7% BEYOND (control +7.2%); plan +6.3% BEYOND (control +1.5%); the other five within floor. **`state` reads `differ` on all 10 rows, and the swap reaches all 10.** The hash covers the save encoding, which now carries `squeeze`, but that is not the whole difference. **CORRECTED 2026-10-26 (code review run 1):** with `squeeze` skipped when `None` (so the encoding equals `main`'s), `state=` still differs on all 10; on plan, full-pile-crowd and plan-{compact,far,spread} every one of the 5 dwarves ends on a different tile, 1-34 items differ, and the swap fires 9/2/14/9/9 times (review measurement, feature + acceptance layers). So the hash difference is behaviour as well as encoding, and plan's +6.3% BEYOND is partly a behaviour change, not swap overhead alone. The end lines (`main` vs HEAD, release `--runs 1`) are a coarse summary, not a reading of reach: they differ on big-pile, deep-pit (items 28 vs 27), dig-loop (marks 11 vs 10), seat-slow (goals 38/12/1 vs 38/11/1) and stuck-tunnel, and match on full-pile-crowd and the four plan scenarios only because they count marks, items, zones and goals. *Was:* "So the swap's reach was read from END LINES instead … **SAME** on full-pile-crowd and all four plan scenarios."
  - **big-pile +120% is a freeze, not swap cost, and the freeze is pre-existing (#218).** HEAD's big-pile freezes at 11 marks from about t5,500 (10,000 ticks: `main` marks=0, HEAD marks=11). From t4,073 the run has an 8 ms tick every 20 ticks (`RETRY_COOLDOWN`): two idle dwarves fail claims on 11 digs, and each failure floods the 16k-cell component. The squeeze swaps only 8 times in the run, with no pair more than twice, so there is no ping-pong. Sonnet agent D traced it: stones 15 and 21 plug two one-wide dead-end corridors. They are never hauled because a clearing set-down at t1717 (stone 25 on (52,71,12)) orphaned the pile's interior (free delivery cells 156 -> 1-2), and the lone cell's refusal makes each release set the stone back on its plug. That is #218 shape B, reached through the full-pile never-drop rule. **Reproduced by Opus on a clean `main` build:** big-pile with its tick-6 commands moved to tick 10 ends 16,000 ticks frozen at `marks=13` (HEAD at the same shift: `marks=2`). Agent D's sweep of shifts 6-45: marks left at 12,000 ticks for 24/40 shifts on `main` and 26/40 on HEAD. The squeeze changes which timing draw big-pile lands on. Posted on #218 (comment 6413682356). Other scenarios at doubled ticks: only big-pile is worse on HEAD, stuck-tunnel is better, and dig-loop freezes on both.
  - **#207 re-measured** on HEAD `9d6810d9` (scratch worktree, `census.patch`'s `assert_ring_fills` census hunk + `census_207_four_bars_round_a_hole` ported by hand, as `census.patch` no longer applies whole): `census ring census_207_four_bars_round_a_hole stored=18/24 free=6 hauls_retrying=8 last_rise_tick=5989 retry_ticks=2631`, identical to creation's B reading. For 13.5's ruling: 18/24 on top of the squeeze (`main` 14/24).
  - **Seat card** `13-4-signoff/vehicle-card.md`: it uses `13-4-signoff/stuck-tunnel-seat.json`, a Normal-speed copy of `stuck-tunnel.json` with the same `--bench` state `a5d257cac0182a1f`, because at the scenario's fast4x the first swap lands 0.3 s after connect. The swap times are taken from a probe (all seven swaps in the run, t18,143..t19,205; the story's "from about t19,082" was `main`'s freeze tick). The framing is `--z 10 --select 4`, checked headless (`seat-framing-select4-z10.png`). Two flags failed on the way: `--camera` focus points that missed the tunnel, and `--at-tick`, which counts ticks after start and tripped the capture's motion guard after 1 observed tick.
- **Seat sitting (Q4 a), 2026-10-26, Wolf on `1aeb4bf8`:** "1 yes 2 not really ..they are just going through". Q1 PASS: the dwarves got past each other and the marks finished. Q2: the pass does not read acceptably without an animation (the two figures blend through each other). Filed as **#219**, as Q4 ruled. Recorded in the card's § Sitting.
- **After dev, Wolf "clean code also" (2026-10-26): the #194 exit back up the line is removed** (`e9712e5f`). The `or_else` route in `resolve_blocked_step`'s idle no-exit path is gone. The #185 chain shift still runs, then the squeeze. `make_room_down_the_idle_chain` returns `()` (its `bool` lost its only reader). The removal is behaviour-neutral by measurement: the release `--bench` `state=` hash was identical before and after on all 10 scenarios plus big-pile at 10,000 ticks and at the tick-10 shift (same save encoding on both sides). **CORRECTED 2026-10-26 (code review run 1):** the branch was not dead, as the commit subject says: it was reachable, 6 hits (4 staircase, 2 stuck-tunnel), each landing in `(None, None)` exactly as HEAD does. It was redundant, and its neutrality is measured on these runs, not shown in general. `cargo test --release -p sim-core` passes in full, 227 tests, no pin moved. 12-10b row 6 is re-anchored on the bare `make_room_down_the_idle_chain(...)` call: KILLED on its `tile_trades <= 2` assert. The fast gate passed on the commit. **The full gate (GREEN on `1aeb4bf8`) predates this commit; it is owed again at PR.**
- **Task 8, FULL GATE GREEN on `1aeb4bf8`** (`RUST_TEST_THREADS=1 scripts/gate.sh`; the only later changes are this record and the seat card):
  ```
  frostvein gate
    cargo fmt --check           ok  0s
    cargo clippy -D warnings    ok  2s
    cargo test                  ok  1144s
    cargo test (pixel guards)   ok  3094s
    tui has no sim-core edge                ok
    client-core has no sim-core edge        ok
    gui has no sim-core edge                ok
    metrics ledger tests        ok  0s
    bench tests                 ok  19s
    mutation tables still apply ok  11s
  reaped 1082 stale build artifacts from target (2.9 GB)
  GATE GREEN  4278s
  ```
- **Review-patch run 1, FULL GATE GREEN on `a0ae8a23`** (`RUST_TEST_THREADS=1 scripts/gate.sh`, exit 0; covers `e9712e5f` and all five patches):
  ```
  frostvein gate
    cargo fmt --check           ok  0s
    cargo clippy -D warnings    ok  2s
    cargo test                  ok  1134s
    cargo test (pixel guards)   ok  2968s
    tui has no sim-core edge                ok
    client-core has no sim-core edge        ok
    gui has no sim-core edge                ok
    metrics ledger tests        ok  0s
    bench tests                 ok  19s
    mutation tables still apply ok  10s
  GATE GREEN  4140s
  ```

### Completion Notes List

- The squeeze is ported from 13.2's variant B with no change of outcome: the bench end lines equal B's on all four reference scenarios. Two head-on job-holders, or a holder and an idle blocker that has not moved off, exit or not (whose `home` moves; review decision 1 (a)), trade tiles in one write once the same stand-off has lasted `SQUEEZE_WAIT_TICKS` (10). The #194/#196 back-out is gone. The timer is a saved per-dwarf `Squeeze` component, a plain `Option` in `SavedDwarf`, so old saves load.
- #206 and #189 are fixed: stuck-tunnel ends `marks=0` (`main` 3, -70.6% tick), and the campfire pile reaches 0 pick-ups after t2500 with every cell filled. #207's ring reads 18/24 (13.5's input).
- **AC 5 was amended by Wolf at Task 3's split point.** The 16k cage sweep pins its four stuck runs instead of clearing them; the shapes (all from the full pile with the cut wall-in) are filed as #218.
- Evidence beyond the plan: three older mutation rows went inert under the squeeze. Two are re-pinned and kill again; 12-10b row 11 is deleted, and its #194 `or_else` branch was later removed on Wolf's "clean code also" (`e9712e5f`): reachable but redundant, every hit landing where HEAD lands (Debug Log). big-pile's +120% is a freeze `main` shares at other command timings (#218 comment).
- Seat: Q1 pass; Q2 the pass reads as walking through each other -> #219 (gui animation, not 13.4's work).
- Not done here, open for Wolf: reopening #189/#206 (closed by #216's prose) and the PR's closing keywords; whether #187 closes (its loop still exists, pinned as #218's shapes).

### File List

- `crates/sim-core/src/lib.rs` (modified)
- `crates/sim-core/src/save.rs` (modified)
- `crates/sim-core/tests/save_load.rs` (modified)
- `crates/sim-core/tests/scenario.rs` (modified)
- `_bmad-output/implementation-artifacts/mutations/12-10b.sh` (modified)
- `_bmad-output/implementation-artifacts/mutations/12-9.sh` (modified)
- `_bmad-output/implementation-artifacts/mutations/13-4.sh` (new)
- `_bmad-output/implementation-artifacts/13-4-signoff/{vehicle-card.md,stuck-tunnel-seat.json,tick-bench-main-HEAD.txt,seat-framing-select4-z10.png}` (new, Task 7)
- `_bmad-output/implementation-artifacts/metrics/13-4-squeeze-past-two-dwarves-trade-tiles.md` (cost rows)
- `_bmad-output/implementation-artifacts/metrics/.session-cursors.json` (modified: metric cursor)
- `_bmad-output/implementation-artifacts/13-4-squeeze-past-two-dwarves-trade-tiles.md` (modified)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)
- `_bmad-output/implementation-artifacts/13-4-signoff/{sweep16k.patch,diag-instrumentation.patch,probe187-extended.rs,diag-traces-waits-10-20-30.txt}` (new, Task 3 evidence)

## Change Log

| Date | Change |
|---|---|
| 2026-10-26 | Review-patch run 1 (fresh session): 5/5 patches landed (`78bf1d41` `c906be17` `b0c82211` `e972ccba` `a0ae8a23`); rows 4c and 5c KILLED at their own asserts; FULL GATE GREEN on `a0ae8a23` (4140 s), which also covers `e9712e5f`. Status review, for a round-2 review (patches 1-2 are test code). |
| 2026-10-26 | Code review run 1 (fresh session) on `8ac4deb8`: 4 layers, none timed out, 0 HIGH. Decision 1 (idle blocker with an exit is swapped) -> Wolf (a) keep + pin. 5 patches left as action items, 9 deferred, 6 dismissed. Status in-progress. Full gate still owed at PR (`e9712e5f` postdates the GREEN on `1aeb4bf8`). |
| 2026-10-26 | Wolf: "clean code also": #194's exit back up the line removed (`e9712e5f`), behaviour-neutral (12 `--bench` state hashes identical); 12-10b row 6 re-anchored and KILLED. Full gate owed again at PR. |
| 2026-10-26 | Dev complete: full gate GREEN on `1aeb4bf8` (4278 s). Status review. |
| 2026-10-26 | Seat sitting (Wolf, `1aeb4bf8`): Q1 pass (the dwarves pass and the marks finish); Q2 the pass reads as walking through each other with no animation -> #219. |
| 2026-10-26 | Task 3 split point fired: four shapes (A1/A2/A3/B), not two, and A2/A3/B need new behaviour on the #180/FR8 never-drop rule. Wolf ruled "land squeeze, pin 16k": AC 5 and Task 4 amended, the cage sweep runs to 16,000 ticks and pins `UNFINISHED_LATE`, the shapes are filed as #218, #187 has a comment. The test keeps its name `no_dwarf_is_caged_by_stones_over_the_probe_seeds`, because 10 mutation rows in 12-9/12-10b/12-11 select it. |
| 2026-10-26 | Implementer: Wolf picked Sonnet 5.5 subagents, Opus orchestrating. |
| 2026-10-26 | Task 0 ruled by Wolf: Q1 (a) allow the idle-blocker swap; Q2 (a) fold the #187 fix in (16k sweep clean at waits 10/20/30, split point after Task 3); Q3 (a) plain `Option` timer field, old saves load; Q4 (a) short seat sitting. No AC amended. |
| 2026-10-26 | Story created (Opus 5.5) from Epic 13 Story 13.4 and the 13.2 proposal. Creation probe on HEAD `380fd789`: 13.2's squeeze numbers reproduce exactly; the idle-blocker swap (Q1) moves only #207; the 4 stuck cage runs are #187's refused cut and a stone-walled mark with the pile full, landed by timing (waits 10/20/30 → 4/2/1 different runs) (Q2). Task 0 Q1–Q4 await Wolf. Status ready-for-dev. |
