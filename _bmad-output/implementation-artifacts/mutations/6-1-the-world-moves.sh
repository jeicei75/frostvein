# Story 6.1 sabotage table. Run alone with scripts/mutate.sh.

mutation "blend extrapolates beyond delivered state" gui blend::tests::midpoint_and_snap_are_literal_wire_positions <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/blend.rs'); s = p.read_text()
old = 'world_to_render(current), factor.clamp(0.0, 1.0))'
assert s.count(old) == 1
p.write_text(s.replace(old, 'world_to_render(current), factor)'))
PY

mutation "torch flicker band widens" gui appearance::flicker_tests::flicker_is_bounded_distinct_and_deterministic <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '            flicker_amplitude: 0.07,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            flicker_amplitude: 0.70,\n'))
PY

mutation "dig chips lose client-local ownership" gui empty_tile_delta_leaves_deterministic_client_local_chips_and_snapshot_clears_them <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                        DigChip(*position),\n                        ClientLocal,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        DigChip(*position),\n'))
PY

mutation "motion capture requires too many ticks" gui capture::tests::motion_instrument_rejects_stillness_and_accepts_the_required_observation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'self.ticks.len() >= 100'
assert s.count(old) == 1
p.write_text(s.replace(old, 'self.ticks.len() >= 101'))
PY

mutation "snapshot rewind no longer snaps" gui snapshot_rewind_snaps_at_a_mid_blend_clock <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/blend.rs'); s = p.read_text()
old = '        None => world_to_render(current),'
assert s.count(old) == 1
p.write_text(s.replace(old, '        None => Vec3::ZERO,'))
PY

mutation "reconciliation overwrites blended translation" gui later_production_reconciliation_does_not_clobber_a_blended_translation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            if let Some(light) = mirror_entity.and_then(|entity| entity.light) {'
assert s.count(old) == 1
p.write_text(s.replace(old, '            commands.entity(bevy_entity).insert(Transform::from_translation(world_to_render(position)));\n            if let Some(light) = mirror_entity.and_then(|entity| entity.light) {'))
PY

mutation "live projection omits the blend" gui projection_pipeline_blends_at_a_midpoint <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            blend_projection,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "reconciliation resets flickered light" gui flickered_light_survives_a_later_production_reconciliation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                if projected_light.is_none_or(|existing| existing.0 != light) {'
assert s.count(old) == 1
p.write_text(s.replace(old, '                if true {'))
PY
