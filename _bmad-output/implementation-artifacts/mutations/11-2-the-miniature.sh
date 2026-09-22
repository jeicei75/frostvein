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
old = '        VolumetricLight,\n'
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

mutation "re-inserted DoF keeps boot framing for a frame" gui toggling_dof_back_on_focuses_the_live_camera_not_boot_framing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            update_dof_from_camera.after(effect_controls),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            update_dof_from_camera,\n'))
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
old = '            Self::Dof => Some(KeyCode::F10),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            Self::Dof => Some(KeyCode::F1),\n'))
PY
