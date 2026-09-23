# Story 11.2 sabotage table. Run alone: scripts/mutate.sh <this file>

mutation "a physically plausible aperture silently disables miniature blur" gui dof_softens_the_far_ridge_while_retaining_camp_focus_and_stars ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'const DOF_APERTURE_F_STOPS: f32 = 0.05;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const DOF_APERTURE_F_STOPS: f32 = 1.0;\n'))
PY

mutation "infinite DoF depth erases the star cores" gui dof_softens_the_far_ridge_while_retaining_camp_focus_and_stars ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'const DOF_MAX_DEPTH: f32 = 120.0;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const DOF_MAX_DEPTH: f32 = f32::INFINITY;\n'))
PY

mutation "focus returns to the rig orbit radius instead of the aim point" gui dof_focus_is_derived_from_the_camera_transform_and_tracks_framing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    camera_translation.distance(world_to_render_f32(rig.focus))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    rig.distance\n'))
PY

mutation "haze returns to Bevy daylight density" gui haze_lifts_and_softens_the_far_valley_without_swallowing_the_sky ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'const FOG_DENSITY_FACTOR: f32 = 0.015;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const FOG_DENSITY_FACTOR: f32 = 0.06;\n'))
PY

mutation "sun stops participating in the volumetric pass" gui haze_lifts_and_softens_the_far_valley_without_swallowing_the_sky ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
# Two writers since 2026-09-23: the spawn, and `sync_haze_light`, which re-inserts the marker every
# frame the haze is on. Removing only the spawn's copy SURVIVES -- the sync puts it back -- so the
# sabotage takes both.
old = '        VolumetricLight,\n'
assert s.count(old) == 1
s = s.replace(old, '')
old = '            commands.entity(entity).insert(VolumetricLight);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "fog ambient stops following the night ambient budget" gui volumetric_fog_ambient_intensity_tracks_the_scenes_ambient_budget <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        ambient_intensity: 0.1 * night_lighting().ambient_brightness / 80.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        ambient_intensity: 0.1,\n'))
PY

mutation "a selected dwarf stops being the focal subject" gui depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let id = selected.0?;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let id = None::<u32>?;\n'))
PY

# RE-POINTED 2026-09-23 (11.2 code review): the focus system moved to `PostUpdate`, after
# transform propagation, where it cannot run before `effect_controls`. The sabotage puts it back in
# `Update` with no ordering at all, which is the defect this row was written for.
mutation "re-inserted DoF keeps boot framing for a frame" gui toggling_dof_back_on_focuses_the_live_camera_not_boot_framing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            update_dof_from_camera.after(TransformSystems::Propagate),\n'
assert s.count(old) == 1
s = s.replace(old, '')
anchor = '            crate::perf::mark_perf_frame_on_key,\n'
assert s.count(anchor) == 1
p.write_text(s.replace(anchor, '            update_dof_from_camera,\n' + anchor))
PY

mutation "ao off takes the depth prepass dof and haze sample" gui turning_ambient_occlusion_off_keeps_the_depth_prepass_dof_and_haze_sample <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let depth = ambient_occlusion || on(CameraEffect::Dof) || on(CameraEffect::Haze);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let depth = ambient_occlusion;\n'))
PY

mutation "an effect sits on a key bevy_dev_tools already binds" gui the_client_keymap_avoids_keys_other_plugins_have_claimed <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::Dof => Some(KeyCode::F5),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::Dof => Some(KeyCode::F1),\n'))
PY

mutation "two controls of ours land on one key" gui the_client_keymap_avoids_keys_other_plugins_have_claimed <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Self::Haze => Some(KeyCode::F4),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::Haze => Some(KeyCode::F3),\n'))
PY

# 2026-09-23. F4 removed VolumetricFog from the camera only; Bevy never syncs that removal, so the
# fog kept drawing. The haze key must also take the sun's marker, which is Bevy's one cleanup path.
mutation "F4 leaves the sun volumetric, so the fog never leaves the render world" gui the_haze_key_moves_the_suns_volumetric_marker <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            commands.entity(entity).remove::<VolumetricLight>();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# 2026-09-23, the three timing leaks that made two captures of one frozen world disagree.
mutation "a frozen world lets its snow fall on the wall clock" gui snow_holds_still_in_a_static_world <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/atmosphere.rs'); s = p.read_text()
old = '    if static_world.0 {\n        return;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = static_world;\n'))
PY

mutation "a frozen world leaves each stride where frame timing put it" gui a_landed_static_world_holds_every_stride_at_one_phase <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        walk.distance = STATIC_WORLD_WALK_PHASE * DWARF_WALK_STRIDE_METRES;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let _ = &mut walk;\n'))
PY

mutation "facing is read once per frame, so a batched step never turns the dwarf" gui the_dwarf_faces_where_he_is_walking_and_holds_it_when_he_stops <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    headings.record(mirror);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = &headings;\n'))
PY

# 2026-09-23, the frame-counted waits the vehicle's RTX 4080 broke.
mutation "the static-world pause timeout counts frames again" gui static_world_pause_timeout_counts_time_not_frames <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/command.rs'); s = p.read_text()
old = '        state.waited += time.delta();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        state.waited += Duration::from_millis(100);\n        let _ = &time;\n'))
PY

mutation "a plain capture fires on its frames with its ticks still missing" gui a_plain_capture_waits_for_its_ticks_and_not_only_its_frames <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        if !floor_applies || self.static_world || self.motion.ticks.len() >= MIN_DELIVERED_TICKS {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if true {\n'))
PY

mutation "simd drops --pause-at on the floor" simd pause_at_is_parsed_beside_the_port_and_rejects_garbage <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '            pause_at = Some(\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let _ = Some::<u64>(\n'))
PY

# ADDED by the 2026-09-23 code review. Both focus tests read the camera through `GlobalTransform`
# on a harness with no `TransformPlugin`, so it sat at the origin and their oracle shared the same
# frozen position. These two mutants read a camera that never moves; before the fix both survived.
mutation "dof focuses a selected dwarf from a camera frozen at the origin" gui depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Some(point) => transform.translation().distance(point),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Some(point) => Vec3::ZERO.distance(point),\n'))
PY

mutation "dof focuses the aim point from a camera frozen at the origin" gui toggling_dof_back_on_focuses_the_live_camera_not_boot_framing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            None => dof_focal_distance(transform.translation(), rig),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            None => dof_focal_distance(Vec3::ZERO, rig),\n'))
PY

# The one-frame lag: focus computed in `Update`, before transform propagation, reads last frame's
# camera, so the frame a dwarf is selected on is drawn focused where the camera USED to be.
mutation "dof focus runs before transform propagation again" gui depth_of_field_focuses_the_selected_dwarf_not_the_rigs_aim_point <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            update_dof_from_camera.after(TransformSystems::Propagate),\n'
assert s.count(old) == 1
s = s.replace(old, '')
anchor = '            crate::perf::mark_perf_frame_on_key,\n'
assert s.count(anchor) == 1
p.write_text(s.replace(anchor, '            update_dof_from_camera.after(effect_controls),\n' + anchor))
PY
