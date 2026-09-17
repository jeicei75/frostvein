# Story 10.10 sabotage table. Run alone: scripts/mutate.sh <this file>
#
# NOT concurrency-safe: it rewrites source IN PLACE. Never run it while anything else compiles.
# The fix must be COMMITTED before mutating -- an interrupted row restores from its own backup,
# but nothing here protects uncommitted work.

# --- Task 2: the seat controls -------------------------------------------------------------

# The key rates are per-SECOND. Without delta time they are per-FRAME, so the same one second of
# input moves the camera twice as far at 60 fps as at 30.
mutation "delta-time scaling leaves the key rates per-frame" gui camera_controls_are_scaled_by_elapsed_time <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let key_scale = time.delta_secs() * multiplier;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let key_scale = multiplier;\n'))
PY

# The MIRROR of the row above, and the reason both exist: a MouseMotion delta is already the
# movement that happened this frame, so scaling it by dt makes one physical sweep depend on the
# frame rate -- the opposite of what "rates are per-second" is for. Scaling everything by dt
# passes the key test above while breaking the mouse.
mutation "mouse deltas are scaled by delta time as well" gui mouse_drag_maps_the_same_motion_at_every_frame_rate <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = "                    yaw - motion.x * MOUSE_ORBIT_RATE * multiplier,\n                    pitch - motion.y * MOUSE_ORBIT_RATE * multiplier,\n"
assert s.count(old) == 1
new = "                    yaw - motion.x * MOUSE_ORBIT_RATE * key_scale,\n                    pitch - motion.y * MOUSE_ORBIT_RATE * key_scale,\n"
p.write_text(s.replace(old, new))
PY

# --- Task 1: the movable focus -------------------------------------------------------------

mutation "the pan clamp lets the focus leave the world" gui pan_moves_focus_on_the_camera_ground_plane_and_stays_inside_the_world <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '        self.focus = (self.focus + movement).clamp(Vec3::ZERO, FOCUS_MAX);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.focus += movement;\n'))
PY

# The boot-rig guard is the WHOLE of AC1 after its luminance clause was struck (issue #98), so it
# is the only thing standing between a moved BOOT_* constant and a silently different opening
# frame. Its literals are hand-written, never read back from the constants, which is exactly what
# makes this row able to kill.
mutation "the boot yaw moves out from under the pinned framing" gui boot_rig_and_transform_are_pinned_by_literals <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = 'const BOOT_YAW: f32 = 0.7;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const BOOT_YAW: f32 = 0.75;\n'))
PY

mutation "the fractional inverse transform mirrors an axis" gui the_fractional_transform_pair_round_trips_and_pins_its_handedness <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/transform.rs'); s = p.read_text()
old = '    Vec3::new(value.x, -value.z, value.y)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    Vec3::new(value.x, value.z, value.y)\n'))
PY

# --- Task 3: the instrument ----------------------------------------------------------------

# THE ROW THIS STORY WAS WRITTEN AROUND. `--distance`'s own docstring records that replacing its
# assignment with `let _ = distance;` left all 106 tests green at 7.2 -- parsed, validated, and
# silently dropped. Measured again here on 2026-09-17: this sabotage leaves 159 other tests green.
mutation "the --camera value is discarded after parsing" gui the_camera_flag_reaches_the_camera_rig_rather_than_merely_parsing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = "    if let Some(start) = start {\n        rig.place(start.yaw, start.pitch, start.distance, start.focus);\n    }\n"
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = start;\n'))
PY

# The readout line IS the save format, so an approximate line is a broken instrument: paste it
# back and you get a DIFFERENT framing while the record claims you reproduced one.
mutation "the readout rounds the framing it prints" gui the_camera_readout_round_trips_through_the_flag_exactly <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '        "{},{},{},{},{},{}",\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        "{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",\n'))
PY

mutation "the readout prints the same line for every rig" gui the_camera_readout_differs_whenever_the_rig_differs <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '        "camera: yaw={} pitch={} distance={} focus={},{},{} --camera {argument}",\n        rig.yaw, rig.pitch, rig.distance, rig.focus.x, rig.focus.y, rig.focus.z\n    )\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        "camera: a framing{}",\n        ""\n    )\n'))
PY

# AC11. The ceiling is calibrated for the BOOT framing only; once --camera makes other framings
# routine, a trip that does not say WHICH view produced it cannot be told from a regression.
mutation "the near-white ceiling stops naming its framing" gui blown_pool_range_failure_is_a_real_panic_not_a_successful_capture <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        "near-white area is {:.4}%, above the {:.4}% ceiling calibrated on boot7.png, at {}",\n        near_white * 100.0,\n        NEAR_WHITE_AREA_CEILING * 100.0,\n        framing\n    );\n'
assert s.count(old) == 1
new = '        "near-white area is {:.4}%, above the {:.4}% ceiling calibrated on boot7.png",\n        near_white * 100.0,\n        NEAR_WHITE_AREA_CEILING * 100.0,\n    );\n    let _ = framing;\n'
p.write_text(s.replace(old, new))
PY

# --- Task 4: select and frame --------------------------------------------------------------

# The trap the story named explicitly: pointing the focus AT the dwarf leaves him off-centre by
# the composition push. Measured under this sabotage -- he projects at y 0.7794 instead of 0.5.
mutation "the framing skips the composition push and aims straight at the dwarf" gui a_left_click_selects_the_nearest_dwarf_and_frames_him_at_screen_centre <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '        self.focus = render_to_world_f32(target - self.composition_push());\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.focus = render_to_world_f32(target);\n'))
PY

mutation "the pick radius is removed so any dwarf answers a click" gui escape_releases_the_selection_and_an_empty_click_leaves_the_rig_untouched <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = '            (distance <= DWARF_PICK_RADIUS).then_some((distance, depth, entity.id))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Some((distance, depth, entity.id))\n'))
PY

mutation "escape stops releasing the selection" gui escape_releases_the_selection_and_an_empty_click_leaves_the_rig_untouched <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = "    if keys.just_pressed(KeyCode::Escape) {\n        selected.0 = None;\n        return;\n    }\n"
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# A framing solved ONCE at the click passes the centring row above and still lets a walking dwarf
# drift out of frame, which is why tracking has its own row.
mutation "the follow solves the framing once instead of tracking him" gui the_focus_tracks_the_selected_dwarf_as_he_walks <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
imports = 'use protocol::EntityKind;'
assert s.count(imports) == 1
s = s.replace(imports, 'use bevy::ecs::change_detection::DetectChanges;\nuse protocol::EntityKind;')
old = "    let Some(id) = selected.0 else {\n        return;\n    };\n"
assert s.count(old) == 1
new = "    if !selected.is_changed() {\n        return;\n    }\n    let Some(id) = selected.0 else {\n        return;\n    };\n"
p.write_text(s.replace(old, new))
PY

# --- Added 2026-09-17 after Wolf's verdict from the seat -------------------------------------

# AC3's wheel half was pinned by NOTHING until the step was raised: the wheel term could have been
# deleted and all 88 tests stayed green. This row is the guard that gap needed.
mutation "the wheel contributes nothing to the zoom" gui the_wheel_zooms_the_rig_and_shift_multiplies_the_step <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        rig.zoom(zoom + wheel * WHEEL_ZOOM_STEP * multiplier);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        rig.zoom(zoom);\n        let _ = wheel;\n'))
PY

# The magnitude, not just the mechanism. The test's expected distances are hand-written from the
# boot 90.0 rather than computed from this constant, which is what lets this row kill.
# NOTE: this row names a TUNED constant, so a future retune will APPLY-FAIL it rather than kill it.
# That is the audit's job to surface (`scripts/audit-mutations.py` fails the gate on a dead row) --
# re-point it to the new value, do not delete it.
mutation "the wheel step falls back to its pre-seat value" gui the_wheel_zooms_the_rig_and_shift_multiplies_the_step <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    const WHEEL_ZOOM_STEP: f32 = 6.0;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    const WHEEL_ZOOM_STEP: f32 = 1.0;\n'))
PY

# Wolf tried a reversed MMB drag from the seat on 2026-09-17 and rejected it, so the shipped
# direction is a DECISION and not an accident. This row is what stops it flipping silently.
mutation "the mouse orbit drag reverses both axes" gui mouse_drag_maps_the_same_motion_at_every_frame_rate <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = "                    yaw - motion.x * MOUSE_ORBIT_RATE * multiplier,\n                    pitch - motion.y * MOUSE_ORBIT_RATE * multiplier,\n"
assert s.count(old) == 1
new = "                    yaw + motion.x * MOUSE_ORBIT_RATE * multiplier,\n                    pitch + motion.y * MOUSE_ORBIT_RATE * multiplier,\n"
p.write_text(s.replace(old, new))
PY
