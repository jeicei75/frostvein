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

mutation "abort no longer wins over a concurrent release" gui right_button_during_a_drag_abandons_it_and_sends_nothing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '    if abort {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false && abort {\n'))
PY

# The row above disables BOTH abort sources at once, so it could never tell a broken Esc from a
# broken right-click -- and the only abort test pressed the right button, which is why Esc could
# be deleted outright with the suite green. These two attack each source ALONE.
mutation "Esc no longer abandons a live drag" gui escape_during_a_drag_abandons_it_and_sends_nothing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = 'keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right)'
assert s.count(old) == 1
p.write_text(s.replace(old, 'mouse.just_pressed(MouseButton::Right)'))
PY

mutation "Esc no longer leaves the mode when no drag is live" gui escape_with_no_drag_leaves_the_mode <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = """        } else if keys.just_pressed(KeyCode::Escape) {
            *mode = DesignateMode::None;
        }"""
assert s.count(old) == 1
p.write_text(s.replace(old, '        }'))
PY

# THE MARCH FACE. Inverting either assignment buries the hover slab in the neighbouring cube --
# 8.1's deferred defect, which this story exists to fix -- and left all 149 tests green.
mutation "the marched east/west face is inverted" gui a_marched_hit_face_always_opposes_the_ray_that_found_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = 'face = if step.x >= 0 { Face::West } else { Face::East };'
assert s.count(old) == 1
p.write_text(s.replace(old, 'face = if step.x >= 0 { Face::East } else { Face::West };'))
PY

mutation "the marched top/bottom face is inverted" gui a_marched_hit_face_always_opposes_the_ray_that_found_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = 'face = if step.y >= 0 { Face::Bottom } else { Face::Top };'
assert s.count(old) == 1
p.write_text(s.replace(old, 'face = if step.y >= 0 { Face::Top } else { Face::Bottom };'))
PY

mutation "a camera inside the rock reports the top face again" gui a_ray_starting_inside_solid_rock_reports_the_face_it_looks_at_not_the_top <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = '        return facing_face(direction);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        return Face::Top;\n'))
PY

# The offset alone is not the feature: unrotated, the slab beside a wall is an edge-on wafer.
mutation "the hover slab is no longer turned onto its face normal" gui a_vertical_hit_face_places_the_hover_slab_outside_the_cell_side <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '\n                    .with_rotation(bevy::prelude::Quat::from_rotation_arc(Vec3::Y, normal))'
assert s.count(old) == 2
p.write_text(s.replace(old, ''))
PY

# THREE OF THE FOUR MODES IN THE STORY TITLE. Only Digit1 was ever pressed by any test, so each
# of these left the whole suite green.
mutation "every mode key selects dig" gui each_mode_key_sends_its_own_distinct_wire_command <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = """    } else if keys.just_pressed(KeyCode::Digit2) {
        *mode = DesignateMode::Channel;
    } else if keys.just_pressed(KeyCode::Digit3) {
        *mode = DesignateMode::Stockpile;
    } else if keys.just_pressed(KeyCode::Digit4) {
        *mode = DesignateMode::Clear;
    }"""
assert s.count(old) == 1
new = """    } else if keys.just_pressed(KeyCode::Digit2)
        || keys.just_pressed(KeyCode::Digit3)
        || keys.just_pressed(KeyCode::Digit4)
    {
        *mode = DesignateMode::Dig;
    }"""
p.write_text(s.replace(old, new))
PY

mutation "channel designates a dig" gui each_mode_key_sends_its_own_distinct_wire_command <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = """        DesignateMode::Channel => vec![Command::Designate {
            kind: DesignationKind::Channel,
            rect,
        }],"""
assert s.count(old) == 1
p.write_text(s.replace(old, old.replace('DesignationKind::Channel', 'DesignationKind::Dig')))
PY

mutation "stockpile placement issues nothing" gui each_mode_key_sends_its_own_distinct_wire_command <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '        DesignateMode::Stockpile => vec![Command::PlaceStockpile { rect }],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Stockpile => Vec::new(),\n'))
PY

mutation "a mode key pressed mid-drag changes what the release commits" gui a_drag_commits_in_the_mode_it_began_in <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = 'for command in commands_for(drag_mode.0.unwrap_or(*mode), rect) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'for command in commands_for(*mode, rect) {'))
PY

# The hint bar was inert: neutering the update left it reading its no-mode string forever.
mutation "the hint bar never updates after startup" gui the_hint_bar_names_the_mode_that_will_commit <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '    if !mode.is_changed() && !anchor.is_changed() && !drag_mode.is_changed() {\n        return;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    return;\n'))
PY

mutation "the drag preview is never cleaned up" gui the_drag_preview_appears_while_dragging_and_disappears_on_release <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = """        if preview_rect.0.take().is_some() {
            for entity in &previews {
                commands.entity(entity).despawn();
            }
        }
        return;"""
assert s.count(old) == 1
p.write_text(s.replace(old, '        preview_rect.0.take();\n        return;'))
PY

# THE INSTRUMENTS. These are the rows that would have caught this round's HIGH findings: an
# instrument that reports success when it captured nothing is worse than no instrument.
mutation "the runner's exit status is discarded again" gui run_consumes_the_runners_exit_status_rather_than_discarding_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = """    if let AppExit::Error(code) = app.run() {
        std::process::exit(code.get().into());
    }"""
assert s.count(old) == 1
p.write_text(s.replace(old, '    app.run();'))
PY

mutation "a scripted drag that designates nothing still passes" gui mark_counts_are_checked_against_the_mirror_not_merely_against_zero <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = """            DesignateMode::Dig | DesignateMode::Channel => assert!(
                self.expected_designations > 0,"""
assert s.count(old) == 1
p.write_text(s.replace(old, old.replace('> 0,', '>= 0,')))
PY

mutation "the scripted drag presses before any pick resolves" gui a_scripted_drag_waits_for_a_live_pick_instead_of_pressing_into_the_dark <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        let pick_is_live = picked.tile().is_some();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let pick_is_live = true;\n'))
PY

mutation "cursor and drag are accepted together again" gui a_scripted_cursor_and_a_scripted_drag_are_mutually_exclusive <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if drag.is_some() && cursor.is_some() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false {\n'))
PY

# A lost designation must leave a COUNTED trace; stderr is not an observable.
mutation "commands lost to a dead peer are not counted" gui a_dead_peer_drains_the_queue_and_counts_every_lost_command <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '            pending.queue.clear();\n            pending.dropped += lost;\n            return;'
assert s.count(old) == 1
p.write_text(s.replace(old, '            pending.queue.clear();\n            return;'))
PY

mutation "the pending queue grows without bound" gui the_queue_bound_drops_and_counts_rather_than_growing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '        if self.queue.len() == MAX_PENDING_COMMANDS {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false {\n'))
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
p.write_text(s.replace(old, '            if u64::from(capture.elapsed) >= ticks_after_start {\n'))
PY

mutation "hover slab returns to the unconditional top-face offset" gui a_vertical_hit_face_places_the_hover_slab_outside_the_cell_side <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'normal * 0.55'
assert s.count(old) == 2
p.write_text(s.replace(old, 'Vec3::Y * 0.55'))
PY
