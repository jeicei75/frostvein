# Story 12.4: every dwarf has a trade, claiming filters by it, and each dwarf spends his own A*
# budget (#159 fix B'). Run alone after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-4.sh

mutation "trade check removed" sim-core hauling_starts_while_the_dig_backlog_is_still_queued <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            if trade(job.kind) != **profession {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && trade(job.kind) != **profession {\n'))
PY

mutation "trade check moved after attempted" sim-core a_job_with_no_free_dwarf_of_its_trade_gets_no_retry_stamp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''            if trade(job.kind) != **profession {
                // Before `attempted`: a dwarf of another trade must not stamp a cooldown on this job.
                continue;
            }
'''
assert s.count(old) == 1
s = s.replace(old, '')
old = '                attempted = true;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, old + '                if trade(job.kind) != **profession {\n                    continue;\n                }\n'))
PY

mutation "trade maps Haul to Miner" sim-core each_trade_holds_only_its_own_jobs_and_the_woodcutter_wanders <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        JobKind::Haul { .. } => Profession::Hauler,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        JobKind::Haul { .. } => Profession::Miner,\n'))
PY

mutation "professions drawn from the spawn stream" sim-core spawn_positions_for_seed_42_are_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        professions.shuffle(&mut profession_rng);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        professions.shuffle(&mut spawn_rng);\n'))
PY

mutation "the pool has no hauler" sim-core every_world_has_two_miners_two_haulers_and_one_woodcutter <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            Profession::Hauler,\n            Profession::Hauler,\n            Profession::Woodcutter,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Profession::Miner,\n            Profession::Miner,\n            Profession::Woodcutter,\n'))
PY

mutation "from_save ignores the saved profession" sim-core save_load_then_tick_matches_never_saved <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    dwarf.identity,\n                    dwarf.profession,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    dwarf.identity,\n                    Profession::Miner,\n'))
PY

mutation "the bridge sends no profession" simd save_then_load_rewinds_every_client <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '                .map(|(_, profession)| profession_out(*profession)),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .map(|(_, profession)| profession_out(*profession))\n                .filter(|_| false),\n'))
PY

mutation "the roster drops the trade word" tui each_trade_word_sits_after_its_own_dwarfs_name_in_grey_and_follows_a_profession_change <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '            let trade = profession\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let trade = profession\n                .filter(|_| false)\n'))
PY

# Row 9: the story named AC11's test, but under a budget per dwarf no dwarf in AC11's 11k-cell
# plates ever exhausts, so this arm is unreachable there. AC12's 55k area is where it fires.
# `return` is `main`'s `break 'jobs`: nothing follows the job loop, and the stamp is skipped.
mutation "exhaustion bails out of the tick again" sim-core a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '(None, true, _) => continue,\n'
assert s.count(old) == 2
p.write_text(s.replace(old, '(None, true, _) => return,\n'))
PY

mutation "one shared budget again" sim-core a_reachable_job_behind_unreachable_ones_is_claimed_when_areas_sum_past_the_budget <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'budgets[slot]'
assert s.count(old) == 3
p.write_text(s.replace(old, 'budgets[0]'))
PY

# Row 11: the issue's candidate, stamp-and-stop, on a budget per dwarf (mutA, 12.4 review 2). The
# exhausting dwarf stamps the job and ends the tick, so a later dwarf of his trade never tries it.
mutation "stamp-and-stop on a budget per dwarf (the issue's candidate)" sim-core a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '(None, true, _) => continue,\n'
assert s.count(old) == 2
p.write_text(s.replace(old, '(None, true, _) => {\n jobs.get_mut(job.id).expect("iterated job still exists").retry_after = tick.0.saturating_add(RETRY_COOLDOWN);\n return;\n }\n'))
PY

mutation "the sat-out rule dropped" sim-core a_dwarf_over_his_budget_sits_out_and_the_crew_goes_on <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if attempted && !assigned && !sat_out {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if attempted && !assigned {\n'))
PY

# Row 13 (mutB, 12.4 review 2): exhaustion leaves the dwarf loop, so the next dwarf of the trade
# never tries the job and it is stamped unclaimed.
mutation "exhaustion leaves the dwarf loop" sim-core a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '(None, true, _) => continue,\n'
assert s.count(old) == 2
p.write_text(s.replace(old, '(None, true, _) => break,\n'))
PY

# Row 14 (mutC, 12.4 review 3): the budget check moves above the trade filter, so an exhausted
# miner marks a haul sat out and the unreachable haul is never stamped.
mutation "budget check before the trade filter" sim-core a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
budget = """                if budgets[slot] == 0 {
                    sat_out = true;
                    continue;
                }
"""
assert s.count(budget) == 1
s = s.replace(budget, '')
filt = '            if trade(job.kind) != **profession {\n'
assert s.count(filt) == 1
p.write_text(s.replace(filt, budget + filt))
PY

# Row 15 (mutD, 12.4 review run 2): a dwarf with no budget left leaves the dwarf loop, so a later
# miner never tries a miner job the exhausted lowest-id miner sits out.
mutation "sat out leaves the dwarf loop" sim-core a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = """                if budgets[slot] == 0 {
                    sat_out = true;
                    continue;
                }
"""
assert s.count(old) == 1
p.write_text(s.replace(old, old.replace('continue;', 'break;')))
PY

# Row 16 (12.4 review run 2): a budget bump past the joined plates' 55,003 cells means no miner
# exhausts. The precondition must fail rather than the test passing vacuously.
mutation "budget raised past the joined plates" sim-core a_job_one_dwarf_exhausted_on_goes_to_the_next_of_his_trade_and_others_still_stamp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'const MAX_ASTAR_NODES: usize = 50_000;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const MAX_ASTAR_NODES: usize = 60_000;\n'))
PY
