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
# NOTE: retargeted 2026-08-20. Commit 04e6de5 raised the torch amplitude 0.07 -> 0.30 and left
# this row aiming at the old literal, so it APPLY-FAILED and pinned nothing from that commit
# until now. A stale sabotage is a silent one.
old = '            flicker_amplitude: 0.30,\n'
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

mutation "live ingest stops re-basing the blend clock" gui ingest::tests::ingesting_a_delta_rebases_the_blend_clock_from_the_wire <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '                clock.observe_tick(mirror.0.tick());\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the blend clock is never advanced by frame time" gui production_drives_the_blend_clock_from_frame_time <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'blend_entities(&mirror.0, &mut clock, time.delta_secs(), &mut projected);'
assert s.count(old) == 1
p.write_text(s.replace(old, 'blend_entities(&mirror.0, &mut clock, 0.0, &mut projected);'))
PY

mutation "the flicker is never advanced by elapsed time" gui production_drives_the_flicker_from_elapsed_time <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'flicker_lights(time.elapsed_secs(), &mut lights);'
assert s.count(old) == 1
p.write_text(s.replace(old, 'flicker_lights(0.0, &mut lights);'))
PY

mutation "a same-frame tick burst collapses the measured cadence" gui blend::tests::a_burst_of_ticks_in_one_frame_keeps_the_measured_cadence <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/blend.rs'); s = p.read_text()
old = '            if self.elapsed >= MIN_TICK_INTERVAL {\n                self.interval = self.elapsed.clamp(MIN_TICK_INTERVAL, MAX_TICK_INTERVAL);\n            }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            self.interval = self.elapsed.clamp(MIN_TICK_INTERVAL, MAX_TICK_INTERVAL);\n'))
PY

mutation "the flicker band widens past its named literals" gui appearance::flicker_tests::flicker_is_bounded_distinct_and_deterministic <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '    1.0 + properties.flicker_amplitude * (primary + secondary) / 1.3'
assert s.count(old) == 1
p.write_text(s.replace(old, '    1.0 + properties.flicker_amplitude * (primary + secondary) / 0.5'))
PY

mutation "the item count stops being a running maximum" gui capture::tests::motion_instrument_rejects_stillness_and_accepts_the_required_observation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'self.item_count = self.item_count.max(item_count);'
assert s.count(old) == 1
p.write_text(s.replace(old, 'self.item_count = item_count;'))
PY

# --- Added 2026-08-20 from the live vehicle session. Wolf saw a dug tile that still read as rock
# --- and rubble he could not find; the cause was an item branch that never set a scale, so the
# --- stone item inherited 1.0 and stood as a terrain-sized block in the tile it came out of. Both
# --- mutations below restore the shipped defect exactly. NOTE the whole suite was GREEN with it.

mutation "a stone item is drawn at terrain-cube scale" gui a_projected_item_is_rubble_resting_on_the_tile_floor <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = 'pub const STONE_ITEM_SCALE: f32 = 0.4;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const STONE_ITEM_SCALE: f32 = 1.0;'))
PY

mutation "the blend lifts every item back off the tile floor" gui a_projected_item_is_rubble_resting_on_the_tile_floor <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            transform.translation = item_translation(*position);'
assert s.count(old) == 1
p.write_text(s.replace(old, '            transform.translation = world_to_render(*position);'))
PY

mutation "an item swallows the debris chips that share its tile" gui project::tests::a_stone_item_never_encloses_its_chips <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = 'pub const STONE_ITEM_SCALE: f32 = 0.4;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const STONE_ITEM_SCALE: f32 = 0.9;'))
PY
