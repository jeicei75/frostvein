# Story 12.6: Wolf gives them their trades. `set_profession` changes a dwarf's trade and releases a
# job of his old trade; the gui shows the roster and sends the next trade on `T`. Run alone, after
# commit, with a fresh debug `simd` built (row 11's real-binary test runs it, and `cargo test -p gui`
# does not rebuild it):
#   cargo build -p simd && RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-6.sh
# Rows 6a/6b are one sabotage (the story's row 6) killed by each of its two named tests.

mutation "set_profession changes nothing" sim-core a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            return self.set_profession(dwarf, profession);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let _ = (dwarf, profession);\n            return None;\n'))
PY

mutation "a reassignment keeps the held job" sim-core a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if held.is_some_and(|job| trade(job.kind) != profession) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && held.is_some_and(|job| trade(job.kind) != profession) {\n'))
PY

mutation "the released job is removed from Jobs" sim-core a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        if held.is_some_and(|job| trade(job.kind) != profession) {
            release_claim(&mut self.ecs, entity);
        }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        if let Some(job) = held.filter(|job| trade(job.kind) != profession) {
            release_claim(&mut self.ecs, entity);
            self.ecs.resource_mut::<Jobs>().remove(job.id);
        }
'''))
PY

mutation "retry_claim instead of release_claim" sim-core a_reassigned_miner_lets_go_and_the_other_miner_takes_the_same_job <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '''        if held.is_some_and(|job| trade(job.kind) != profession) {
            release_claim(&mut self.ecs, entity);
        }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        if let Some(job) = held.filter(|job| trade(job.kind) != profession) {
            retry_claim(&mut self.ecs, entity, job.id);
        }
'''))
PY

mutation "a reassigned hauler keeps carrying" sim-core a_reassigned_hauler_puts_the_stone_down_and_another_hauler_delivers_it <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if held.is_some_and(|job| trade(job.kind) != profession) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if held.is_some_and(|job| {\n            trade(job.kind) != profession && !matches!(job.kind, JobKind::Haul { .. })\n        }) {\n'))
PY

mutation "6a an unknown dwarf is not refused (sim)" sim-core an_unknown_dwarf_is_refused_and_an_emptied_trade_is_allowed <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            return Some(Refusal::SetProfession { dwarf });\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            return None;\n'))
PY

mutation "6b an unknown dwarf is not refused (wire)" simd a_reassigned_miner_lets_go_on_the_wire_and_an_unknown_dwarf_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            return Some(Refusal::SetProfession { dwarf });\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            return None;\n'))
PY

mutation "the bridge reads hauler as miner" simd a_reassigned_miner_lets_go_on_the_wire_and_an_unknown_dwarf_is_refused <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '        protocol::Profession::Hauler => sim_core::Profession::Hauler,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        protocol::Profession::Hauler => sim_core::Profession::Miner,\n'))
PY

mutation "T sends the current trade, not the next" gui t_sends_one_set_profession_with_the_next_trade_and_the_readout_waits_for_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '        profession: next_trade(current),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        profession: current,\n'))
PY

mutation "the roster drops the trade word" gui the_roster_lists_every_dwarf_in_id_order_and_follows_a_trade_change <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            format!("{}{newline}", trade(entity).unwrap_or_default()),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            newline.to_string(),\n'))
PY

mutation "the readout shows the requested trade, not the mirror's" gui t_sends_one_set_profession_with_the_next_trade_and_the_readout_waits_for_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        match trade(entity) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        match entity\n            .profession\n            .map(|p| client_core::profession_text(crate::command::next_trade(p)))\n        {\n'))
PY

mutation "--trade pushes nothing" gui a_trade_set_from_the_gui_comes_back_on_the_wire ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let (Some(trade), Some(dwarf)) = (trade, selected.0) else {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let (Some(trade), Some(dwarf), true) = (trade, selected.0, false) else {\n'))
PY
