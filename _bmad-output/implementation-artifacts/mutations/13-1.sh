# Story 13.1: Scripted runs and the tick bench. Run alone, after commit:
#   env -u NO_COLOR RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/13-1.sh
# One row per decision (Task 6), each killed by the test named beside it.

mutation "1 the bench skips command application (AC 2)" simd bench_replays_the_scenario_identically_and_reaches_the_oracle_state <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '            for command in schedule.due(world.tick()) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            for command in schedule.due(world.tick()).into_iter().filter(|_| false) {\n'))
PY

mutation "2 the bench applies a command one tick late (AC 2)" simd bench_replays_the_scenario_identically_and_reaches_the_oracle_state <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '            for command in schedule.due(world.tick()) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            for command in schedule.due(world.tick().saturating_sub(1)) {\n'))
PY

mutation "3 a served scenario applies a command one tick late (AC 3)" simd a_served_scenario_holds_for_its_client_and_ends_on_the_benchs_hash <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '        self.schedule.due(tick)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.schedule.due(tick.saturating_sub(1))\n'))
PY

mutation "4 the state hash ignores the world (AC 1)" simd a_scenario_without_its_command_reaches_a_different_world <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    bytes.iter().take(0).fold(0xcbf2_9ce4_8422_2325, |hash, byte| {\n'))
PY

mutation "5 p99 is taken as the max (AC 1)" simd stats_take_p99_by_nearest_rank_and_count_slow_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '        p99: sorted[(n * 99).div_ceil(100) - 1],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        p99: sorted[n - 1],\n'))
PY

mutation "6 a served scenario does not hold for its client (AC 3)" simd a_served_scenario_holds_for_its_client_and_ends_on_the_benchs_hash <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            protocol::Speed::Paused\n        }\n        None => protocol::Speed::Normal,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            protocol::Speed::Fast4x\n        }\n        None => protocol::Speed::Normal,\n'))
PY

mutation "7 the hold never ends (AC 3)" simd a_served_scenario_holds_for_its_client_and_ends_on_the_benchs_hash <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '        if !self.holding {\n            return None;\n        }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if self.holding || !self.holding {\n            return None;\n        }\n'))
PY

mutation "8 a served scenario does not stop on its end tick (AC 3)" simd a_served_scenario_holds_for_its_client_and_ends_on_the_benchs_hash <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            if live.finish(&world) {\n                speed = protocol::Speed::Paused;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if live.finish(&world) {\n                let _ = protocol::Speed::Paused;\n'))
PY

mutation "9 a refusal does not fail the bench (AC 2)" simd every_unusable_scenario_fails_loudly_naming_the_cause <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '                if let Some(refusal) = crate::apply_sim_command(&mut world, command.clone()) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                if let Some(refusal) = crate::apply_sim_command(&mut world, command.clone()).filter(|_| false) {\n'))
PY

mutation "10 a daemon command is let into a scenario (AC 6)" simd every_unusable_scenario_fails_loudly_naming_the_cause <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '                bail!("a scenario drives the sim, not the daemon: {command:?} at {at}");\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let _ = ();\n'))
PY

mutation "11 a scenario's save loads without the live checks (AC 6)" simd every_unusable_scenario_fails_loudly_naming_the_cause <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/scenario.rs'); s = p.read_text()
old = '                crate::try_load_world_from(path).with_context(|| {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                Ok::<_, anyhow::Error>(sim_core::World::generate(1, sim_core::Dims::DEFAULT)).with_context(|| {\n'))
PY
