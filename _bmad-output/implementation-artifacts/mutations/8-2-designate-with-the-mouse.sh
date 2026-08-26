# Mutation set for story 8.2. Run alone: scripts/mutate.sh <this file>

mutation "command writer is dropped from the live input schedule" gui configured_app_sends_a_real_mouse_drags_command_to_the_daemon_socket <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            send_commands.after(designation_input),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "built designation command is discarded instead of queued" gui configured_app_sends_a_real_mouse_drags_command_to_the_daemon_socket <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '                pending.push(command);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let _ = command;\n'))
PY

mutation "shared rect helper is replaced by local min max normalization" gui designation_input_uses_the_shared_rect_helper_not_local_normalization <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '''            let rect = client_core::rect_on_level(
                (anchor_tile[0], anchor_tile[1]),
                (release_tile[0], release_tile[1]),
                anchor_tile[2],
            );
'''
assert s.count(old) == 1
new = '''            let rect = Rect {
                min: [
                    anchor_tile[0].min(release_tile[0]),
                    anchor_tile[1].min(release_tile[1]),
                    anchor_tile[2],
                ],
                max: [
                    anchor_tile[0].max(release_tile[0]),
                    anchor_tile[1].max(release_tile[1]),
                    anchor_tile[2],
                ],
            };
'''
p.write_text(s.replace(old, new))
PY

mutation "release height replaces the anchor level" gui mouse_drag_uses_the_anchor_level_and_clears_its_anchor_on_release <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '                anchor_tile[2],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                release_tile[2],\n'))
PY

mutation "abort no longer wins over a concurrent release" gui abort_wins_over_a_same_frame_release_and_sends_nothing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '    if abort {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false && abort {\n'))
PY

mutation "clear mode omits stockpile removal" gui clear_issues_both_existing_commands_in_tui_order <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '            Command::RemoveStockpile { rect },\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "parsed drag never reaches the scripted mouse state" gui parsed_capture_drags_send_their_own_rectangles_to_the_daemon_socket <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '''        app.insert_resource(ScriptedDrag {
            spec,
            stage: ScriptedDragStage::Press,
        });
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '        let _ = spec;\n'))
PY

mutation "at-tick capture fires on frame count" gui at_tick_capture_waits_for_the_mirror_tick_and_reports_an_exhausted_budget <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '            if mirror.0.tick() >= target_tick {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if capture.elapsed >= ticks_after_start {\n'))
PY

mutation "hover slab returns to the unconditional top-face offset" gui a_vertical_hit_face_places_the_hover_slab_outside_the_cell_side <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'normal * 0.55'
assert s.count(old) == 2
p.write_text(s.replace(old, 'Vec3::Y * 0.55'))
PY

