# Story 12.4: every dwarf has a trade, claiming filters by it, and each dwarf spends his own A*
# budget (#159 fix B'). Run alone after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-4.sh

# Retired 2026-11-02 (13.7): rows 9-15 ("exhaustion bails out of the tick again", "one shared budget
# again", "stamp-and-stop on a budget per dwarf", "the sat-out rule dropped", "exhaustion leaves the
# dwarf loop", "budget check before the trade filter", "sat out leaves the dwarf loop") sabotaged
# the per-dwarf budget, its `(None, true, _)` arms and the sat-out rule. 13.7 Q2 (a) removed all
# three: every claim search gets a fresh budget of the world's cell count, which cannot run out,
# so the arms are `unreachable!`. The claim side is pinned by 13-7.sh.
# Retired 2026-11-02 (13.7): row 16 "budget raised past the joined plates" (MAX_ASTAR_NODES 50,000 ->
# 60,000) SURVIVED when re-run on its renamed target: the claim no longer reads the constant, and
# the test lost the exhaustion precondition the bump was meant to fail. The constant's other
# readers are pinned by 3-2-the-dig.sh "MAX_ASTAR_NODES is widened" (astar_stops_at_the_node_cap).

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
