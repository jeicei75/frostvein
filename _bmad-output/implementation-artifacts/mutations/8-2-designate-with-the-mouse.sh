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
old = '''            let picked_rect = client_core::rect_on_level(
                (anchor_cell.tile[0], anchor_cell.tile[1]),
                (release_cell.tile[0], release_cell.tile[1]),
                anchor_cell.tile[2],
            );
'''
assert s.count(old) == 1
new = '''            let picked_rect = Rect {
                min: [
                    anchor_cell.tile[0].min(release_cell.tile[0]),
                    anchor_cell.tile[1].min(release_cell.tile[1]),
                    anchor_cell.tile[2],
                ],
                max: [
                    anchor_cell.tile[0].max(release_cell.tile[0]),
                    anchor_cell.tile[1].max(release_cell.tile[1]),
                    anchor_cell.tile[2],
                ],
            };
'''
p.write_text(s.replace(old, new))
PY

mutation "release height replaces the anchor level" gui mouse_drag_uses_the_anchor_level_and_clears_its_anchor_on_release <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '                anchor_cell.tile[2],\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                release_cell.tile[2],\n'))
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
mutation "every mode key selects dig" gui each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts <<'PY'
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

mutation "channel designates a dig" gui each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
# `DesignationKind::Channel` also appears in the Clear arm's neighbourhood, so the anchor takes
# the Channel arm's map closure whole.
old = """            .map(|rect| Command::Designate {
                kind: DesignationKind::Channel,
                rect: *rect,
            })"""
assert s.count(old) == 1
p.write_text(s.replace(old, old.replace('DesignationKind::Channel', 'DesignationKind::Dig')))
PY

mutation "stockpile placement issues nothing" gui each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '''        DesignateMode::Stockpile => surface
            .iter()
            .map(|rect| Command::PlaceStockpile { rect: *rect })
            .collect(),'''
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Stockpile => Vec::new(),'))
PY

mutation "a mode key pressed mid-drag changes what the release commits" gui a_drag_commits_in_the_mode_it_began_in <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = 'let mode = drag_mode.0.unwrap_or(*mode);'
assert s.count(old) == 1
p.write_text(s.replace(old, 'let mode = *mode;'))
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
old = """        if preview_cells_cache.0.take().is_some() {
            for entity in &previews {
                commands.entity(entity).despawn();
            }
        }
        return;"""
assert s.count(old) == 1
p.write_text(s.replace(old, '        preview_cells_cache.0.take();\n        return;'))
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

mutation "clear mode omits stockpile removal" gui clear_reaches_both_the_picked_cell_and_the_standable_one <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = '                    Command::RemoveStockpile { rect: *rect },\n'
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

# ---------------------------------------------------------------------------
# ROUND 2, 2026-08-27. Found by Wolf's hands on the vehicle, not by any layer:
# channel and stockpile were COMPLETELY INERT. Picking only ever resolves a
# solid cell; `sim-core` filters both commands on standability and discards the
# rest in silence. Every test above asserts what the CLIENT queues, so all of
# them stayed green while the daemon kept nothing.
# ---------------------------------------------------------------------------

mutation "channel and stockpile designate the solid cell the ray hit" gui each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = 'DesignateMode::Channel | DesignateMode::Stockpile | DesignateMode::Clear => {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'DesignateMode::Channel | DesignateMode::Stockpile | DesignateMode::Clear if false => {'))
PY

mutation "clear stops reaching the cell the ray hit" gui each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = 'std::iter::once(Command::CancelDesignation { rect: picked_rect })'
assert s.count(old) == 1
p.write_text(s.replace(old, 'std::iter::empty::<Command>()'))
PY

mutation "the daemon is assumed to keep a designation anywhere" simd the_daemon_keeps_channels_and_stockpiles_only_at_standable_cells <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = """    matches!(mirror.tile(pos), Some(Tile::Empty))
        && matches!(
            mirror.tile([pos[0], pos[1], pos[2] - 1]),
            Some(Tile::Solid(_) | Tile::Ramp(_))
        )"""
assert s.count(old) == 1
p.write_text(s.replace(old, '    matches!(mirror.tile(pos), Some(Tile::Empty))'))
PY

mutation "a frame whose ray misses terrain erases the live preview" gui the_drag_preview_survives_a_frame_whose_ray_misses_terrain <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = """    let Some(release) = picked.0 else {
        return;
    };"""
assert s.count(old) == 1
new = """    let Some(release) = picked.0 else {
        if preview_rect.0.take().is_some() {
            for entity in &previews {
                commands.entity(entity).despawn();
            }
        }
        return;
    };"""
p.write_text(s.replace(old, new))
PY

mutation "the preview promises marks the sim will discard" gui the_preview_covers_only_the_cells_the_sim_will_keep <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                .filter(|tile| sim_will_keep(mirror, *tile, mode))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "a buried channel mark stays sealed inside the rock above it" gui a_buried_channel_mark_climbs_onto_the_rock_covering_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = """    if top == z {
        slab_transform(position, -0.46)
    } else {
        slab_transform([x, y, top], 0.54)
    }"""
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = (top, x, y, z);\n    slab_transform(position, -0.46)'))
PY

# ---------------------------------------------------------------------------
# ROUND 3, 2026-08-27. Wolf drove the fixed build and it was STILL wrong, in
# two ways that only measurement settled: the face rule landed 8.5-11.8% of
# side-face hits, and AC4's single-z rect kept a median 19.4% of a 6x6
# stockpile footprint on natural ground.
# ---------------------------------------------------------------------------

mutation "a side-face hit stops falling back to the cell above" gui a_side_face_hit_on_flat_ground_falls_back_to_the_cell_above <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = """            let above = [cell.tile[0], cell.tile[1], cell.tile[2] + 1];
            if client_core::is_standable(mirror, above) {
                return above;
            }"""
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the fallback wins over a standable ledge" gui a_side_face_hit_on_flat_ground_falls_back_to_the_cell_above <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = """            if client_core::is_standable(mirror, neighbour) {
                return neighbour;
            }"""
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the standable modes flatten back to the anchor level" gui a_channel_drag_across_a_step_follows_the_ground_while_dig_stays_on_one_level <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '            if let Some(cell) = standable_in_column(mirror, x, y, a[2], level) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if let Some(cell) = Some([x, y, a[2]]) {\n'))
PY

mutation "dig follows the surface instead of cutting one level" gui a_channel_drag_across_a_step_follows_the_ground_while_dig_stays_on_one_level <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/designate.rs'); s = p.read_text()
old = """        DesignateMode::Dig => vec![Command::Designate {
            kind: DesignationKind::Dig,
            rect: picked_rect,
        }],"""
assert s.count(old) == 1
p.write_text(s.replace(old, """        DesignateMode::Dig => surface
            .iter()
            .map(|rect| Command::Designate {
                kind: DesignationKind::Dig,
                rect: *rect,
            })
            .collect(),"""))
PY

mutation "the column scan stops one short of the cut surface" gui each_mode_key_sends_its_own_command_at_the_cell_the_sim_accepts <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = '    (0..=level + 1)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    (0..=level)\n'))
PY

mutation "the followed surface is sent as one bounding box" simd a_surface_following_drag_lands_its_whole_footprint_and_nothing_else <<'PY'
import pathlib
p = pathlib.Path('crates/client-core/src/lib.rs'); s = p.read_text()
old = """            Some(last)
                if last.max[1] == cell[1]
                    && last.max[2] == cell[2]
                    && last.max[0] + 1 == cell[0] =>
            {
                last.max[0] = cell[0];
            }"""
assert s.count(old) == 1
p.write_text(s.replace(old, """            Some(last) => {
                last.min = [last.min[0].min(cell[0]), last.min[1].min(cell[1]), last.min[2].min(cell[2])];
                last.max = [last.max[0].max(cell[0]), last.max[1].max(cell[1]), last.max[2].max(cell[2])];
            }"""))
PY

mutation "the preview stops following the ground with the send path" gui a_channel_drag_across_a_step_follows_the_ground_while_dig_stays_on_one_level <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        DesignateMode::Channel | DesignateMode::Stockpile => client_core::surface_targets(\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Stockpile => client_core::surface_targets(\n'))
PY

# AC18's readout. Found 2026-08-27 while testing the recipe rather than shipping it: `tui --frame`
# drew only what fit its viewport and reported nothing about the rest, so a 9-tile stockpile read
# as 0 glyphs from one terminal and 7 from another, silently. An instrument that reports success
# when it captured nothing is worse than no instrument -- the standing exception to out-of-scope.

mutation "the tui frame reports a mark count it cannot back up" tui the_mark_tally_reports_what_the_frame_could_not_show <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = """        self.drawn_designations == self.mirror_designations && self.drawn_zones == self.mirror_zones"""
assert s.count(old) == 1
p.write_text(s.replace(old, '        true'))
PY

mutation "the tally counts the mirror twice instead of the frame" tui the_mark_tally_reports_what_the_frame_could_not_show <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = """        drawn_zones: framebuffer
            .cells
            .iter()
            .filter(|cell| cell.glyph == zone_glyph)
            .count(),"""
assert s.count(old) == 1
p.write_text(s.replace(old, """        drawn_zones: mirror
            .zones()
            .iter()
            .filter(|zone| zone.pos[2] == state.z)
            .count(),"""))
PY
