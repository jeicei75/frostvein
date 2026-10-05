# Story 12.5: dwarves at work -- dig and haul. The wire carries a dwarf's job and carried stone;
# the gui picks Walk/Dig/Carry from it. Run alone after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-5.sh

mutation "the bridge sends no job" simd deltas_label_a_miners_dig_a_haulers_haul_and_the_stone_he_carries <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '                .map(|job| dwarf_job(*job)),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .map(|job| dwarf_job(*job))\n                .filter(|_| false),\n'))
PY

mutation "the bridge sends no carried stone" simd deltas_label_a_miners_dig_a_haulers_haul_and_the_stone_he_carries <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '                .and_then(|(_, stone)| *stone),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .and_then(|(_, stone)| *stone)\n                .filter(|_| false),\n'))
PY

# Re-pointed 2026-10-05 (12.7): `dwarf_clip`'s match gained the `Cut` arm (a woodcutter in Work swings), so the quoted `matches!` is the three-arm form; the sabotage still moves the carry test ahead of it.
mutation "Carry is preferred over Dig" gui digging_outranks_carrying_in_the_clip_choice <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '''    if entity.state == protocol::JobState::Work
        && matches!(
            entity.job,
            Some(
                protocol::DwarfJob::Dig { .. }
                    | protocol::DwarfJob::Channel { .. }
                    | protocol::DwarfJob::Cut { .. }
            )
        )
    {
        DwarfClip::Dig
    } else if entity.carrying.is_some() {
        DwarfClip::Carry
    } else {
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''    if entity.carrying.is_some() {
        DwarfClip::Carry
    } else if entity.state == protocol::JobState::Work
        && matches!(
            entity.job,
            Some(
                protocol::DwarfJob::Dig { .. }
                    | protocol::DwarfJob::Channel { .. }
                    | protocol::DwarfJob::Cut { .. }
            )
        )
    {
        DwarfClip::Dig
    } else {
'''))
PY

mutation "the dig phase runs on wall time" gui the_dig_phase_runs_on_delivered_ticks_and_holds_when_the_ticks_repeat <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    clock: Res<TickClock>,\n    mut commands: Commands,\n'
assert s.count(old) == 1
s = s.replace(old, '    clock: Res<TickClock>,\n    time: Res<bevy::time::Time>,\n    mut commands: Commands,\n')
old = '''            dig.phase = ((tick.saturating_sub(dig.entered) as f32 + clock.factor())
                / WORK_SWING_TICKS as f32)
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '            let _ = &clock;\n            dig.phase = (time.elapsed_secs() * 2.0)\n'))
PY

mutation "a digging dwarf no longer faces his target" gui a_digging_dwarf_faces_his_target_and_a_channel_keeps_his_heading <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
# Re-pointed 2026-10-05 (12.7): `dig_yaw` also faces a `Cut` target, so the `let Some(..)` pattern is `Dig | Cut` and rustfmt wraps it; the sabotage still blinds it.
old = '    let Some(protocol::DwarfJob::Dig { target } | protocol::DwarfJob::Cut { target }) = entity.job\n    else {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let Some(protocol::DwarfJob::Dig { target } | protocol::DwarfJob::Cut { target }) = entity.job.filter(|_| false)\n    else {\n'))
PY

mutation "the blend writer moves a carried stone" gui a_carried_stone_is_the_dwarfs_child_at_the_carry_offset_until_he_lets_go <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
# Re-pointed 2026-10-05 (12.7): the blend's item map now holds the whole item (kind and stack), so the binding is `item`, not `position`.
old = '        } else if let Some(item) = items.get(&marker.0).filter(|_| parent.is_none()) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        } else if let Some(item) = items.get(&marker.0).filter(|_| parent.is_none() || true) {\n'))
PY

mutation "clips are bound by index again" gui each_clip_binds_the_label_of_its_own_name_in_export_order <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let index = names.iter().position(|name| name == wanted)?;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let index = names.iter().position(|name| name == wanted).map(|_| 0)?;\n'))
PY

# Wolf at the seat, 2026-10-03: a dig is ten swings, DIG_WORK_TICKS 50; hauls keep WORK_TICKS 5.
mutation "a dig takes a haul's WORK_TICKS again" sim-core execute_jobs_walks_then_digs_for_exactly_dig_work_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '            JobKind::Dig | JobKind::Channel => DIG_WORK_TICKS,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            JobKind::Dig | JobKind::Channel => WORK_TICKS,\n'))
PY

mutation "a save taken mid-dig is refused again" simd a_save_taken_mid_dig_loads <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '                if dwarf.work_progress > sim_core::DIG_WORK_TICKS {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                if dwarf.work_progress > sim_core::WORK_TICKS {\n'))
PY

# Wolf at the seat, 2026-10-03: "should probably stop first and then start digging" -- the dig clip
# and the dig facing both wait until his drawn body reaches the cell.
mutation "the dig clip starts while he is still walking in" gui a_dwarf_still_walking_in_does_not_swing_or_turn_until_he_is_drawn_at_his_cell <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            DwarfClip::Dig if !arrived => {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            DwarfClip::Dig if false && !arrived => {\n'))
PY

mutation "the dig facing turns him while he is still walking in" gui a_dwarf_still_walking_in_does_not_swing_or_turn_until_he_is_drawn_at_his_cell <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                .filter(|_| drawn_at_cell(entity, transform.translation))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                .filter(|_| true || drawn_at_cell(entity, transform.translation))\n'))
PY

# Wolf at the seat, 2026-10-03: "hauler when dropping cargo walks into it ..so also it should stop
# before dropping" -- a delivered stone stays in his hands until his drawn body reaches the cell.
mutation "the stone is put down while he is still walking in" gui a_hauler_keeps_his_stone_until_he_is_drawn_at_the_cell_he_drops_it_on <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            (None, Some(parent)) if !walking_in.contains(&parent.parent()) => {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            (None, Some(parent)) if true || !walking_in.contains(&parent.parent()) => {\n'))
PY

mutation "he drops the carry pose while still holding the stone" gui a_hauler_keeps_his_stone_until_he_is_drawn_at_the_cell_he_drops_it_on <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            DwarfClip::Walk if still_holding => DwarfClip::Carry,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# 12.5 review, 2026-10-04. The live tests designate only channels, so the bridge's `Dig` arm is
# judged by one unit test alone.
mutation "the bridge sends a dig as a channel" simd a_dig_job_goes_on_the_wire_as_dig_with_its_target <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '        sim_core::JobKind::Dig => protocol::DwarfJob::Dig { target },\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        sim_core::JobKind::Dig => protocol::DwarfJob::Channel { target },\n'))
PY

# 12.5 review: AC8's `clip` lines come from the clip CHOICE, above the asset, so they stay green
# when a clip never binds. Only the test's STALLED assertion can kill this.
mutation "the Dig clip is never bound" gui a_miner_logs_dig_and_a_hauler_logs_carry_from_a_real_daemon ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                let (walk, dig, carry) = (load("Walk"), load("Dig"), load("Carry"));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let (walk, dig, carry) = (load("Walk"), load("Dug"), load("Carry"));\n'))
PY

# NOTE: AC8's deliberate RED (the bridge sends no job, judged by the real-binary AC8 test) cannot be
# a row here. `cargo test -p gui` does not rebuild `simd`, so the test would drive the STALE daemon
# and report SURVIVED. It is run by hand in a scratch worktree with its own target dir; see the
# story's Debug Log.
