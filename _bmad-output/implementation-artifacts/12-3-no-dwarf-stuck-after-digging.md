---
baseline_commit: bbf1518
model: claude-opus-5-5  # session default
---

# Story 12.3: No Dwarf Stuck After Digging

Status: done

## Story

As the boss,
I want a dwarf that cannot finish its job to let go of it and move on,
so that my crew never freezes in place with work still waiting.

## Not stacked — branch off `main`

`main` is `bbf1518` (PR #158 merged), clean. Branch: `story-12-3-no-dwarf-stuck-after-digging`.
Closes **#132**. Epic 12's nine standing ACs (`epics.md`, "Standing acceptance criteria") bind this
story and are not restated. **Wire diff: none.** **New saved state: none** (see guardrails).

## Found at creation — #132 reproduced (suspect 1, in a sharper form)

Throwaway probes on `DEFAULT_SEED` were run against `bbf1518` and then deleted.

**The defect is head-of-line starvation of the whole job market.** It is the retro's suspect 1,
"an unreachable designation retried forever", but no dwarf holds the bad job. It blocks every job
behind it:

- The dwarves' walkable component is **16,063 cells** (18,336 standable in the world).
  `claim_jobs` shares one `MAX_ASTAR_NODES = 50_000` budget per tick (`lib.rs:43`, `:449`).
- A job whose work positions are **non-empty but unreachable** makes each idle dwarf's A* explore
  the whole component. Three dwarves spend about 48k nodes, and the fourth exhausts the budget. That
  hits `(None, true) => break 'jobs` (`lib.rs:485`), which skips the `retry_after` stamp
  (`lib.rs:499-503`). Next tick the same job is first in FIFO again, and this repeats **forever**.
  Every later job is never attempted.
- **Measured.** Control: a reachable dig at `(45,62,12)` designated at tick 101 is dug at tick 339.
  With one dig on a pine crown `(65,56,13)` designated first, the same reachable dig is never
  claimed in 700 ticks. All five dwarves are idle, the crown job's `retry_after` is frozen at `28`,
  and the reachable job's stays `0` (never attempted). The debug daemon also drops from 10 to ~6
  ticks/s, because it burns 50k A* nodes every tick.
- **Why "after digging".** With ≤3 idle dwarves the bad job's attempt fits the budget and gets
  stamped. The trap closes when the 4th and 5th dwarves finish their digs and go idle. From then
  on nothing new is claimed.
- **Natural trigger.** The fresh world has 292 such dig targets, **all tree foliage**. Their work
  positions are tree-top cells (standable, unreachable), and a gui dig drag anchored on a crown
  makes one (`gui/src/designate.rs:180`, single-z rect at the anchor's level). No stone/soil target
  qualifies on a fresh world.

**Suspect 2 does not strand a dwarf.** A channel worker standing on `T` when `T`'s support is
removed falls one level at +1 tick. At +2 ticks it is released by `retry_claim`
(`execute_jobs`, `lib.rs:954-956`). The job stays queued with `retry_after` stamped, and the
designation stays. The orphaned job then has an empty goal set, which `astar_with_budget`
early-returns on for free (`lib.rs:666`). Wolf's 2026-08-06 ruling (`deferred-work.md:317-331`,
"leave as specified") still stands.

**A second, by-design class looks the same at the seat, and is NOT fixed here** (Wolf question 1).
`scripts/task6-designate.py`'s dig rect `(50..57, 58..69, z9)` drains 79 → **50 and stops**. All 50
leftovers have an **empty** goal set: hill tiles at z9 sit behind a strip of `Ramp` tiles at z9. A
ramp is neither standable nor diggable, and `Dig` work positions are same-z only (`lib.rs:729-737`).
Dwarves idle and the marks stay: FR8's "retried forever", costing nothing. This matches
`deferred-work.md:424-428` ("two dig designations never completed across ~38k ticks").

**Live RED on `bbf1518`** (the Verification recipe, run verbatim):
`marks: z 12 designations=2 of 2`, snapshot at tick 490 still holds `[45,62,12]`.
Control (same recipe minus step 1, fresh daemon): `(45,62,12)` dug by tick 482. Snapshot
`[[52,64,12]]`, and that one is a ramp-shielded leftover of the by-design class.

## Acceptance Criteria

1. A scenario test on `DEFAULT_SEED`, **red on the unfixed code and recorded as red**, puts at least
   200 dig jobs with non-empty but unreachable work positions ahead of a reachable dig in FIFO
   order, all five dwarves idle. It asserts the reachable dig is **claimed within 40 ticks** of
   designation and **dug within 600**.
2. In the same test, every unreachable dig is still designated and queued at the end. Each one's
   `retry_after` has been re-stamped after the reachable dig was designated (FR8: retried, never
   parked, never dropped).
3. A dwarf holding a channel job whose support is removed under it holds no claim within **2 ticks**.
   The job and its designation stay queued. A second reachable job is then claimed and dug
   (FR51 "goes on with other work").
4. A dwarf holding a dig job whose every work position becomes unreachable mid-walk holds no claim
   within **1 tick** of the terrain change. The job stays queued with `retry_after` stamped.
5. Existing guards stay green unchanged: the walking skeleton
   (`scenario.rs:826`), `unreachable_job_stays_queued_and_retries_after_twenty_ticks`
   (`scenario.rs:672`) and `an_unreachable_lower_id_does_not_starve_a_reachable_dwarf`
   (`lib.rs:3085`). `claim_jobs_bounds_aggregate_astar_expansions_per_tick` (`lib.rs:3156`) is
   amended per Task 2 and still proves a per-tick bound on claim-time A* work.
6. #132 closes with the AC1 test's red-then-green record.

## Tasks / Subtasks

- [x] **Task 0 — RED first (AC1, AC2).** Add the scenario test to
  `crates/sim-core/tests/scenario.rs` (skeleton below) and run it on the unfixed code. Paste the
  failing assertion and the frozen `retry_after` values into the Debug Log.
- [x] **Task 1 — the fix in `claim_jobs` (AC1, AC2).** An unreachable job must cost a bounded amount
  per tick, however many there are, and must still be stamped `retry_after = tick + RETRY_COOLDOWN`
  every time it is attempted. Recommended shape (see Key decisions):
  - A failed search that ran to completion has explored the whole walkable component of its start.
    Return that explored set from the failure (e.g. `astar_with_budget` hands back its `costs` keys
    on `(None, false)`), and keep a per-tick list of known components inside `claim_jobs`.
  - Before searching from dwarf `d` for job `j`: if `d`'s position is in a known component `C` and
    `goals ∩ C` is empty, skip the search. That still counts as `attempted` (so the stamp fires) and
    spends no budget.
  - Components live only for one `claim_jobs` call: no resource, no cross-tick cache (AD-5), no
    `SaveState` change (AD-11).
- [x] **Task 2 — amend the budget test (AC5).** `claim_jobs_bounds_aggregate_astar_expansions_per_tick`
  asserts that jobs 5..9 are never attempted. That is the starvation shape this story removes, so it
  goes red under the fix. Re-fixture it so it still proves that one tick's claim-time A* expansions
  stay ≤ `MAX_ASTAR_NODES`, e.g. with reachable-but-long searches, or dwarves in separate large
  components. Record the amendment and its reason in Completion Notes.
- [x] **Task 3 — release guards (AC3, AC4).** These are expected green on the current code; they pin
  behaviour the fix must not break. If either goes red, it is a second defect and is fixed here.
  - Scenario test (AC3): use the channel-on-`T` fixture from "Found at creation". Wait for a dwarf in
    `Work` on `T`, `set_tile(T - z1, Empty)` (the cell below `T - z1` is soil on `DEFAULT_SEED` when
    `T` is 2 east of dwarf 0), and designate a reachable dig. Assert release ≤ 2 ticks, the channel
    job and designation remain, and the reachable dig is dug.
  - Unit test in `lib.rs` (AC4): build a fixture like `claimed_dwarf_settles_before_moving_from_newly_unsupported_ground`
    (`lib.rs:3813`). Give a holder a path, wall off its every work position with `set_tile`, then call
    `clear_paths` the way `execute_jobs` does after a sim dig (`lib.rs:1065`). Assert the claim is
    `None` after one `step`, the job remains, and its `retry_after = tick + 20`.
- [x] **Task 4 — instrument (existing, cited; no new code).** The human-visible instrument is
  `tui --frame --z 12`. It prints `marks: z 12 designations=N of M` to stderr (`tui/src/main.rs:207`;
  tally pinned by `view.rs:711`, key names by `main.rs:568`). The snapshot's `designations` list
  backs it up. Run the Verification recipe after the fix (GREEN), then with mutation row 1 applied
  (deliberate RED: `2 of 2` again), then restored. Paste all three outputs into the Debug Log.
- [x] **Task 5 — mutations.** `_bmad-output/implementation-artifacts/mutations/12-3.sh` (NEW), run
  alone after commit with `RUST_TEST_THREADS=1 scripts/mutate.sh`:
  1. remove the component skip (always search) → AC1 test fails on the claim bound;
  2. skip without counting as `attempted` → AC2 assertion fails (`retry_after` never re-stamped);
  3. treat the first failed component as covering every dwarf → `an_unreachable_lower_id_does_not_starve_a_reachable_dwarf` fails.
  Re-point any older row whose anchor the fix moves; `scripts/audit-mutations.py` and the pre-commit
  hook report rot.
- [x] **Task 6 — record.** Mark `deferred-work.md:424-428` with what was found: the two never-cleared
  digs were most likely the ramp-shielded empty-goal class, and that class is now Wolf question 1.
  Update `lib.rs:760`'s `#132` NOTE to the ruling on Wolf question 2. Full gate:
  `RUST_TEST_THREADS=1 scripts/gate.sh`, green before review.

- [x] **Task 7 — Wolf question 2, PULLED IN (Wolf, 2026-10-01).** The sealed-off stockpile cell
  pick-up/drop loop (`lib.rs:760` NOTE; 12.1 handover `12-1-stones-stay-out-of-the-fire.md:320-322`).
  - [x] Reproduce first (M2-27): a scenario test, red on the unfixed code and recorded as red, where the
    only free zone cell is standable but sealed off from every dwarf, and a stone is reachable.
    Record the pick-up/drop cycle it shows (counts, ticks) in the Debug Log.
  - [x] Fix so the cycle stops while FR8 holds (the haul job stays queued and is retried, never
    dropped). The fix design is confirmed with the orchestrator before it lands.
  - [x] Mutation row(s) for the fix in `12-3.sh`. The `:760` NOTE is replaced by what the fix does.
  - Wolf question 1 (ramp-shielded empty-goal digs) RULED **leave as FR8** (Wolf, 2026-10-01).

### Review Findings

Review run 1 (2026-10-01) on `19fce82` vs `bbf1518`. There were four layers, and every one ran cargo
with its own `CARGO_TARGET_DIR`. None timed out. Blind Hunter (Sonnet) took `crates/sim-core/src`.
Edge Case Hunter (Sonnet) took `crates/sim-core/tests` and `mutations/`. The Acceptance Auditor and
Feature Auditor (Opus) took the whole diff. Severity runs HIGH (feature) / MED (edge+blind) / LOW
(auditor), set at triage. There were 0 HIGH findings. 3 were dismissed: the budget test's
`all(retry_after == 0)` is covered by its control run; an unwalkable-start component only ever
misses a skip; and a forest-wide drag queues the reachable dig behind about 500 reachable crown
jobs, which is FIFO by design (12.7).

| # | Layer | Sev | Route | Finding |
| --- | --- | --- | --- | --- |
| 1 | accept | MED | decision -> #159 + patch | Multi-component budget residual |
| 2 | edge+accept | MED | patch | AC2 assertion weaker than AC2 |
| 3 | accept+orchestrator | LOW | patch | Task 7 record stale (unticked subtask, Completion Note, File List) |
| 4 | feature | LOW | patch | `deferred-work.md` says the crown class is "fixed" |
| 5 | blind | LOW | defer | Haul pays two searches per successful claim |
| 6 | blind | LOW | patch (in-function) | Dead `!explored.is_empty()` guards |
| 7 | accept | LOW | defer | Q2 mid-walk residual recorded only in a code NOTE |
| 8 | edge+accept | LOW | defer | AC3 test soft spots |
| 9 | edge | LOW | defer | Fixture prerequisites assumed, not asserted |

- [x] [Review][Decision] Multi-component budget residual — **RESOLVED (Wolf, option 2): filed #159 (`bug` + `route:story`); NOTE extended + defer, below. Reason: unreachable on today's worlds.** When the dwarves stand in several
  components whose sizes sum to more than `MAX_ASTAR_NODES`, or in one component larger than it,
  the last flood exhausts the budget on EVERY tick. Job 0 is then never stamped, and nothing behind
  it is attempted. That is #132's freeze in a form this fix does not close. The amended
  `claim_jobs_bounds_aggregate_astar_expansions_per_tick` builds exactly this case (five 11,000-cell
  plates), and the next tick repeats it identically. It cannot happen on today's worlds: the
  dwarves' component is about 16k cells, and a walled-in dwarf adds only a small one. It needs
  digging to grow the walkable area past about 50k. The `// NOTE:` at `lib.rs:452` names only the
  single-component case, and the Key decision's "≤18,336, measured" holds for a fresh world only.
- [x] [Review][Patch] Extend the budget NOTE to the summed bound and drop the dead guards (from decision 1) [crates/sim-core/src/lib.rs:452,511,531] —
  The NOTE must say the bound is the SUM of the idle dwarves' distinct components (#159), not one component.
  The two `!explored.is_empty()` guards in the same function go too (finding 6).
- [x] [Review][Patch] AC2 assertion weaker than AC2 [crates/sim-core/tests/scenario.rs:826] —
  `retry_after > designated_at` also passes for a job last stamped up to 19 ticks BEFORE
  designation. "Re-stamped after designation" is `retry_after > designated_at + 20`
  (`RETRY_COOLDOWN` is private to `lib.rs`). Re-mutate row 2 afterwards and confirm it still dies
  on this assertion.
- [x] [Review][Patch] Task 7 record stale [12-3-no-dwarf-stuck-after-digging.md:145,302,308-317] —
  The "Mutation row(s)" subtask is unticked, but rows 4 and 5 exist and are KILLED (the auditor
  re-killed them by hand). The `:760` NOTE is replaced (`lib.rs:812-813`). The Completion Note still
  says "left for the later mutation step". The File List lists `lib.rs`/`scenario.rs` twice and
  omits `metrics/12-3-…md` and `metrics/.session-cursors.json`.
- [x] [Review][Patch] `deferred-work.md` overclaims [_bmad-output/implementation-artifacts/deferred-work.md:434] —
  It says the crown class "is fixed in 12.3". What is fixed is the job-market STARVATION. The crown
  marks themselves stay designated and retried forever under FR8 until 12.7 stops dig taking tree
  tiles.
**Patch pass closure table** (in-session, Wolf option 1, 2026-10-01; code in `15fd2b8`):

| Item | Fix written for | Then tested | Named fixture / sabotage |
| --- | --- | --- | --- |
| Summed-bound NOTE + dead guards | the reader of `claim_jobs` (#159); the guards were unreachable | the old anchor: `3-2-the-dig.sh` "unreachable lower id starves a reachable dwarf", re-pointed at the guard-free block | that row re-run on `15fd2b8`: KILLED at `lib.rs:3185` (`an_unreachable_lower_id_does_not_starve_a_reachable_dwarf`); sim-core lib 63/63 |
| AC2 `> designated_at + 20` | a stamp made 1-19 ticks BEFORE designation no longer passes | the old failure: `12-3.sh` row 2 "component skip not counted as attempted", re-run alone on `15fd2b8` | KILLED, and it dies on the STRENGTHENED assertion itself (`scenario.rs:828`, `retry_after=0`), not an earlier one. NOT constructed: a mutant whose stamps stop exactly at designation (the only case the old bound let through) |
| Task 7 record | the subtask checkbox, Completion Note and File List | read against `12-3.sh` rows 4-5 and `git diff --stat main...HEAD` | rows 4/5 KILLED (Debug Log; re-killed by hand by the Acceptance Auditor) |
| `deferred-work.md` crown wording | "fixed" → starvation fixed; marks stay until 12.7 | the Feature Auditor's live GREEN, `[[52,64,12],[65,56,13]]` | `(65,56,13)` is still designated after the fix |

After the patches: **FULL GATE GREEN** `RUST_TEST_THREADS=1 scripts/gate.sh` on `cc3a9af`, `GATE GREEN  4458s`.
Layer caches reaped (`scripts/reap-build-caches.sh --tmp-only --force`): 8 directories, 8.6 GB.
Review $9.38 (202 turns, recorded).

- [x] [Review][Defer] Haul pays two searches per successful claim [crates/sim-core/src/lib.rs:501-540] — deferred, a cost this change introduced but not a starvation risk
- [x] [Review][Defer] Q2 mid-walk residual only in a code NOTE [crates/sim-core/src/lib.rs:812-813] — deferred, recorded in deferred-work.md
- [x] [Review][Defer] AC3 test soft spots [crates/sim-core/tests/scenario.rs:263-271] — deferred, test hardening
- [x] [Review][Defer] Fixture prerequisites assumed, not asserted [crates/sim-core/tests/scenario.rs:105-117, ~1844] — deferred, test hardening

### Scenario test skeleton (Task 0)

```rust
#[test]
fn unreachable_digs_never_starve_a_reachable_one() {
    let mut world = World::generate(sim_core::DEFAULT_SEED, Dims::DEFAULT);
    // A sky plate far above the terrain, 20 x 20 (assert each fixture cell was Empty first):
    // Solid(Stone) at z=24 under the whole plate; at z=25, rows alternate between Empty (standable
    // floor) and Solid(Stone) targets -> 10 target rows x 20 = 200 dig jobs. Each target's work
    // positions are the floor cells beside it: standable, unreachable from the valley.
    // ... set_tile the plate; designate Dig over the plate at z=25 (one rect keeps only the targets) ...
    for _ in 0..100 { world.step(); }                  // all 5 dwarves past reaction delay, idle
    let reachable = Pos { x: 45, y: 62, z: 12 };       // assert Solid(Stone) with a reachable neighbour
    world.apply_command(SimCommand::Designate { kind: DesignationKind::Dig, rect: rect(reachable, reachable) });
    let designated_at = world.tick();
    // step; record the tick a claim holds the reachable job; assert <= designated_at + 40
    // step on; assert world.tile(reachable) == Some(Tile::Empty) by designated_at + 600
    // LAST (so no earlier assert absorbs mutation 2): every sky target still in designations()
    // and its job's retry_after > designated_at
}
```

## Dev Notes

### Scope guardrails (do NOT)

- Do not change `Dig`/`Channel` work positions, make ramps diggable, or refuse "unworkable" marks.
  The ramp-shielded class is Wolf question 1 and stays FR8's retried-forever until he rules.
- Do not fix the sealed-off stockpile cell pick-up/drop loop (`lib.rs:760` NOTE, handed over by
  12.1's guardrails). It is Wolf question 2. Reproduce it before any fix if he pulls it in (M2-27).
- Do not add occupancy, a pathfinder, a hierarchy or a cross-tick reachability cache (AD-5; #133 is
  12.9). Plain A* stays the only route-finder. The component reuse is A*'s own explored set.
- Do not add persistent per-job state (failure counters, an "abandoned" flag). If a design needs
  one, it goes into `SaveState` with a save → load → tick N test, and the reason goes in Completion
  Notes.
- Do not drop or cancel a job the sim cannot reach. FR8 holds, and AC2 pins it.
- No wire, protocol, client or gui change. No refusal: nothing here is a command the sim refuses.

### What already exists (build on it)

- `claim_jobs` (`lib.rs:422-505`): FIFO by `JobId`, dwarves by ascending `Id`, reaction delay,
  `retry_after`, one shared budget. The fix changes only how an unreachable search is spent (AD-12:
  claiming logic stays single).
- `astar_with_budget` (`lib.rs:659-706`) already returns `(None, false)` for a completed search and
  `(None, true)` for budget exhaustion. Its `costs` map holds the explored set.
- `retry_claim` → `release_claim` (`lib.rs:885`, `:805`) is the one release funnel: drop, idle,
  `wander.home` reset, `Path`/`WorkProgress` removed. Holder release (AC3, AC4) already goes through it.
- `settle` (`lib.rs:1076`) drops a dwarf one level per tick. `execute_jobs` waits on unstandable
  ground (`lib.rs:929-933`) and then recomputes the path.

### Key decisions & traps

- **Stamping on budget exhaustion alone is not enough.** With ≥20 unreachable jobs, one of them
  comes off cooldown every tick and eats that tick's budget, so the reachable job is still never
  reached. AC1's 200 jobs exist to fail that patch.
- **Skipping same-component dwarves within ONE job is not enough either.** One failed search costs
  about 16k nodes, so about 3 unreachable jobs fit per tick, or about 60 per cooldown window. With
  200 jobs the starvation comes back. The component must be reused **across jobs** within the tick.
- **A component belongs to its start, not to every dwarf.** A walled-in dwarf's tiny component says
  nothing about the others, and `an_unreachable_lower_id_does_not_starve_a_reachable_dwarf` is the
  guard (mutation 3).
- **A skipped search is still an attempt.** `attempted = true` is set before the search today, so
  keep it that way, or the job is never stamped and AC2 fails (mutation 2).
- **The budget still applies to the searches that do run.** A component flood that hits the budget
  returns `(None, true)` with no component. Next tick's fresh 50k completes it: every component on
  `Dims::DEFAULT` is ≤18,336 cells (measured). `// NOTE:` that the bound assumes a component fits
  in `MAX_ASTAR_NODES`.
- **The AC1 test is slow while red** (~0.16 s/tick debug at full budget burn). Keep its tick loops
  bounded by the AC numbers. Do not loop "until claimed".

### Verification (recipe; the RED half was run at creation on `bbf1518`)

```bash
cargo build -q -p simd -p tui
./target/debug/simd 7493 >/dev/null 2>&1 &              # fresh daemon, DEFAULT_SEED, camp cursor (64,64)
H=$(printf 'h,%.0s' $(seq 19))
# 1. a dig on a pine crown (65,56,13): non-empty, unreachable work positions (tree tops)
./target/debug/tui 7493 --frames 20 --z 13 --key d,enter,l,k,k,k,k,k,k,k,k,enter >/dev/null
sleep 10                                                 # all five dwarves start burning the budget
# 2. a dig rect (45..64, 62..64, z12): keeps (45,62,12) reachable and (52,64,12) ramp-shielded
./target/debug/tui 7493 --frames 20 --z 12 --key d,enter,${H}k,k,enter >/dev/null
sleep 60
./target/debug/tui 7493 --frame --z 12 2>&1 >/dev/null | rg 'marks: z 12'
bash -c 'head -1 < /dev/tcp/127.0.0.1/7493' | python3 -c "import json,sys; s=json.loads(sys.stdin.readline()); print('tick', s['tick'], 'designations', [d['pos'] for d in s['designations']])"
pkill -x simd
#   RED (observed, bbf1518): marks: z 12 designations=2 of 2 zones=0 of 0
#                            tick 490 designations [[45, 62, 12], [52, 64, 12], [65, 56, 13]]
#   CONTROL (observed, step 1 omitted): (45,62,12) dug by tick 482; designations [[52, 64, 12]]
#   GREEN (required): designations=1 of 1, and the list is [[52, 64, 12], [65, 56, 13]]
```

Deliberate RED after the fix: apply mutation row 1, rebuild, rerun → `designations=2 of 2`. Restore
it, and the recipe shows `1 of 1` again. Exit 0 is not a result; the `marks:` line is.
`(52,64,12)` stays in every run (by-design class), so `1 of 1` is GREEN and `0 of 0` would mean the
recipe aimed wrong. The crown trigger dies when 12.7 stops dig taking tree tiles; this recipe is
12.3's only.

### Project Structure Notes

- `crates/sim-core/src/lib.rs`: UPDATE (`claim_jobs`, `astar_with_budget` failure return, AC4 unit
  test, Task 2 test amendment, `:760` NOTE)
- `crates/sim-core/tests/scenario.rs`: UPDATE (AC1/AC2 test, AC3 test)
- `_bmad-output/implementation-artifacts/mutations/12-3.sh`: NEW
- `_bmad-output/implementation-artifacts/deferred-work.md`: UPDATE (`:424-428`)

### Previous story intelligence (12.1, 12.2)

- Put the vacuity-sensitive assertion LAST. 12.1's "pile filled" assert, placed early, absorbed a
  mutation meant for a later one (`12-2` story :550-551). The AC2 assert closes the AC1 test for
  this reason.
- 12.1 handed the "only free stockpile tile is unreachable" loop to 12.3
  (`12-1-stones-stay-out-of-the-fire.md:320-322`). It is surfaced as Wolf question 2, not silently
  taken or dropped.
- The full gate goes green only at `RUST_TEST_THREADS=1` (~45-70 min). At 2 threads the pixel
  guards fail on load, not on pixels.

### References

- `epics.md` Epic 12 intro (order: #132 before #133; standing ACs) and Story 12.3. PRD
  `prd-frostvein-2026-09-28` FR51 (`:118-121`), NFR9. PRD `prd-frostvein-2026-08-01` FR8 (`:77-79`).
- Parent spine `architecture-frostvein-2026-08-01/ARCHITECTURE-SPINE.md`: AD-5 (`:87-89`), AD-7
  (`:115-120`), AD-11 (`:183-192`), AD-12 (`:200-205`)
- Issue #132. `deferred-work.md:317-331` (channel orphan ruling), `:424-428` (never-cleared digs)
- `scripts/task6-designate.py` docstring ("Digs decay to a floor"): the by-design class, observed
  in 7.2

## Dev Agent Record

### Agent Model Used

Sonnet 5.5 subagents x2 (Tasks 0-3; Task 7 reproduce, then fix), orchestrated and verified by Opus 5.5, which also did Tasks 4-6, the Task 7 mutation rows and the added haul-starvation test.

### Debug Log References

- **Task 0 RED** (unfixed `702446e` code; `cargo test -p sim-core --test scenario unreachable_digs_never_starve_a_reachable_one`, 22 s): `the reachable dig was not claimed within 40 ticks of designation`. Probe at tick 141 (designated at 100): the 200 unreachable jobs' distinct `retry_after` values were frozen at `[0, 26..33, 36, 37]` (none re-stamped since the first ticks); the reachable job `JobId(200)` target `(45,62,12)` had `retry_after: 0`, never attempted.
- **Task 1 GREEN**: same test, 12.6 s: reachable dig claimed at designation +6, dug at +204.
- **Task 7 RED** (unfixed `19823ac` code; `cargo test -q -p sim-core --test scenario a_sealed_off`): fixture seed 42, stone 10 at `(66,61,25)`, one-cell pile at `stone + (0,6)` walled on all four sides (cells asserted standable first). 7 pick-ups in 400 ticks at 140 (dwarf 1), 271, 299, 327, 355, 383, 411 (dwarf 0), a steady 28-tick period; 7 drops one tick after each, all at the stone's own tile `(66,61,25)`; `retry_after` stamps `{0,161,292,320,348,376,404,432}` (job kept and re-stamped, FR8 holds). Control `an_opened_pile_cell_receives_the_stone` (one wall opened) passes: stone ends on the pile.
- **Task 7 GREEN**: same test asserts 0 pick-ups and passes; control still passes; `cargo test -q -p sim-core` all green.
- **Task 4 instrument** (the Verification recipe, fresh daemon each run, debug build):
  - GREEN on `4947aac`: `marks: z 12 designations=1 of 1 zones=0 of 0` / `tick 753 designations [[52, 64, 12], [65, 56, 13]]`
  - Deliberate RED (mutation row 1 applied, simd rebuilt): `marks: z 12 designations=2 of 2 zones=0 of 0` / `tick 474 designations [[45, 62, 12], [52, 64, 12], [65, 56, 13]]`
  - Restored (`b571790`), rebuilt: `marks: z 12 designations=1 of 1 zones=0 of 0` / `tick 753 designations [[52, 64, 12], [65, 56, 13]]`
- **Task 5 mutations** (`RUST_TEST_THREADS=1 scripts/mutate.sh .../mutations/12-3.sh`, run alone after commit):
  - First run on `0582159`: rows 1-4 KILLED, row 5 ("component skip ignores the delivery leg") SURVIVED against the sealed-pile cycle test. The clause only saves floods, so that test could not see it.
  - Added `a_sealed_off_pile_cell_does_not_starve_a_reachable_dig` (#132's haul form: five idle dwarves each flooding the valley for the sealed pile exhaust the budget, and a reachable dig behind the haul is never attempted), re-pointed row 5 at it (`b571790`). Re-run: **5/5 KILLED**.
  - Per-assertion check (trap 1), each row applied by hand on committed code: row 1 dies on `the reachable dig was not claimed within 40 ticks of designation` (scenario.rs:801); row 2 dies on the LAST assertion, `job JobId(20) at Pos { x: 42, y: 40, z: 25 } was never retried: retry_after=0` (scenario.rs:826), so no earlier assert absorbs it.
  - `scripts/audit-mutations.py`: clean.
- **FULL GATE GREEN** on `e03787d`: `RUST_TEST_THREADS=1 scripts/gate.sh`, `GATE GREEN  4221s` (pixel guards 3970 s).

### Completion Notes List

- **Tasks 4-6.** No new instrument code: the existing `tui --frame --z 12` `marks:` line showed GREEN, RED under mutation row 1, and GREEN again (Debug Log). `12-3.sh` has five rows, all KILLED. `deferred-work.md`'s never-cleared-digs entry now names the ramp-shielded class (Wolf Q1: leave as FR8) and #132's crown class (fixed here). Task 6's `:760` NOTE item was overtaken by Wolf pulling Q2 in: Task 7 replaced the NOTE with what the fix does.
- **Added test beyond the skeleton:** `a_sealed_off_pile_cell_does_not_starve_a_reachable_dig`, written because mutation row 5 survived. It pins that the delivery-disjoint clause is load-bearing for the budget, not just tidiness.
- **AC6 (#132 closes):** closes via the PR's `Closes #132`. The red-then-green record is the Debug Log above. No comment posted on the issue.
- **Task 7 (Wolf Q2).** `claim_jobs` now, for a `Haul` job, also computes the delivery goals (`work_positions(.., Some(item))`, the `free` set). The component skip fires when a known component holding the dwarf misses the pick-up goals OR the delivery goals; otherwise it searches dwarf to `free` first on the shared budget (completed failure records the component and skips; exhausted breaks as before), discards that path, then runs the existing dwarf to stone search. Valid because `astar_neighbours` is symmetric. `attempted` stays true on every skip/failure, so `retry_after` is still stamped (FR8). Residual, named in the replaced `:778` NOTE: a pile sealed off mid-walk can still cost one pick-up/drop per retry. `release_claim`/#153 untouched. Mutation rows 4 (delivery pre-search) and 5 (delivery clause of the component skip) are in `12-3.sh`, both KILLED (Debug Log, Task 5).

- **Task 3: both release guards were GREEN on first run, no second defect.** AC3 scenario `a_channel_worker_whose_support_is_removed_lets_go_and_the_crew_goes_on`: holder (Id 2) in `Work` on `T`, `T - z1` set Empty, claim gone within 2 steps, channel job and designation kept, a second reachable dig designated at tick 26 was dug at tick 260. AC4 unit `a_holder_whose_every_work_position_is_walled_off_mid_walk_lets_go_in_one_step`: claim `None` after one `step`, job kept, `retry_after = tick + 20`, designation kept. (An idle dwarf's `JobState` after release is not asserted: `wander` can set `Walk` in the same step.)
- **Task 1.** `astar_with_budget` now returns `(Option<path>, exhausted, explored)`; `explored` is the start's whole walkable component on a COMPLETED failure, empty otherwise. `claim_jobs` keeps a per-call `Vec<BTreeSet<Pos>>` of those components and, before searching from a dwarf, skips (still `attempted`, no budget) when the dwarf's position is in a known component that the job's goals miss. No resource, no cross-tick state, no `SaveState` change. `// NOTE:` records the assumption that a component fits in `MAX_ASTAR_NODES`.
- **Task 2 amendment.** `claim_jobs_bounds_aggregate_astar_expansions_per_tick` asserted jobs 5..9 were never attempted (the starvation shape); under the fix those jobs are stamped for free, so it went red as predicted. Re-fixtured: each of the five dwarves stands on its own 11,000-cell plate (separate components), so five floods cost 55,000 > `MAX_ASTAR_NODES`. Run one: four floods fit, the fifth hits the budget, so no job is stamped. Control run (fifth plate shrunk): all ten jobs stamped, proving the system ran. Checked that raising `MAX_ASTAR_NODES` to 500,000 turns it red.

### File List

- `crates/sim-core/src/lib.rs` (`claim_jobs` component skip + Task 7 delivery check, `astar_with_budget`, `work_positions` NOTE, Task 2 test, AC4 test)
- `crates/sim-core/tests/scenario.rs` (AC1/AC2 test, AC3 test, Task 7 sealed-pile test + control, `a_sealed_off_pile_cell_does_not_starve_a_reachable_dig`)
- `_bmad-output/implementation-artifacts/mutations/12-3.sh` (NEW, 5 rows)
- `_bmad-output/implementation-artifacts/mutations/3-2-the-dig.sh` (two rows re-pointed at the new `astar_with_budget` return shape)
- `_bmad-output/implementation-artifacts/deferred-work.md` (never-cleared-digs entry; review deferrals)
- `_bmad-output/implementation-artifacts/metrics/12-3-no-dwarf-stuck-after-digging.md` (NEW), `metrics/.session-cursors.json`
- `_bmad-output/implementation-artifacts/12-3-no-dwarf-stuck-after-digging.md`, `sprint-status.yaml`

## Change Log

| Date | Change |
| --- | --- |
| 2026-10-01 | Story created on `bbf1518`. #132 reproduced at creation (sim probe and live daemon): A* budget starvation behind an unreachable job; suspect 2 does not reproduce |
| 2026-10-01 | Dev started on `story-12-3-no-dwarf-stuck-after-digging`. Wolf ruled Q1 leave as FR8, Q2 pulled in as Task 7; dev delegated to Sonnet 5.5 subagents |
| 2026-10-01 | Dev done: #132 fixed (per-call component reuse in `claim_jobs`), Wolf Q2 fixed (haul delivery reachability at claim time), AC3/AC4 guards, budget test re-fixtured, 5/5 mutations KILLED, live RED/GREEN recorded; full gate green on `e03787d`. Status -> review |
| 2026-10-01 | Review run 1: 0 HIGH. 4 patches applied (`15fd2b8`, `cc3a9af`). The multi-component budget residual was filed as #159 (Wolf). Full gate green on `cc3a9af`. Status -> done |
