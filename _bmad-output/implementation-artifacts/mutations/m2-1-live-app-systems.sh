# M2-1 sabotage table — the live `App` that `run()` builds. Run alone with scripts/mutate.sh.
#
# WHY THIS TABLE EXISTS. Before M2-1 the registration tuples lived inline in `run()`, unreachable
# from any test, so deleting a system left the whole suite green. That was filed MED and deferred
# at 5.4's review and then produced the top-severity finding in the next FOUR consecutive stories
# (6.1 twice, 6.2, 7.1, 7.2). Every row below deletes one system from the tuples `client_systems`
# registers — the same function `run()` calls. A row that SURVIVES means the class is back.

mutation "the startup scene loses its camera" gui the_live_startup_scene_spawns_its_camera_lighting_and_atmosphere <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            setup_camera,\n            setup_night_lighting,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            setup_night_lighting,'))
PY

mutation "the startup scene loses its directional fill" gui the_live_startup_scene_spawns_its_camera_lighting_and_atmosphere <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            setup_night_lighting,\n            setup_fog_volume,\n            setup_projection_assets,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            setup_fog_volume,\n            setup_projection_assets,'))
PY

mutation "the startup scene loses its sky and snowfall" gui the_live_startup_scene_spawns_its_camera_lighting_and_atmosphere <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            setup_atmosphere,\n            setup_designate_hint,\n            log_adapter,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            setup_designate_hint,\n            log_adapter,'))
PY

mutation "the client-local classification pass never runs" gui the_classification_pass_leaves_no_entity_outside_the_partition <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    .add_systems(bevy::app::PostStartup, classify_client_local)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "camera controls drop out of the update tuple" gui camera_controls_drive_the_rig <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
# RE-ANCHORED 2026-09-03 (story 10.7): light_controls now sits between these two. RE-ANCHORED
# AGAIN 2026-09-17 (story 10.10): camera_readout followed camera_controls in the tuple.
# RE-ANCHORED 2026-09-18 (10.10 review patch): camera_readout left the tuple for its own
# registration, where it carries the ordering edges that keep it reading a rig this frame's
# systems have already written. The row still drops camera_controls out of the update tuple and
# nothing else.
# RE-ANCHORED 2026-09-26 (story 8.3): toggle_hud now follows camera_controls in the tuple.
old = '            camera_controls,\n            toggle_hud,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            toggle_hud,'))
PY

mutation "fog stops following the camera" gui fog_follows_the_camera_rig_every_frame <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            update_fog_from_camera,\n            crate::perf::mark_perf_frame_on_key,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            crate::perf::mark_perf_frame_on_key,'))
PY

# RE-POINTED 2026-09-22. The seam is gone: the overlay no longer HAS a toggle. It is simply on,
# because the F row ran out of keys and F3 became the perf mark. The row's question -- is the
# diagnostic overlay actually wired up, or only believed to be? -- survives the change, so it now
# sabotages the default it was given instead of the registration it used to have.
mutation "the diagnostic overlay is not actually enabled" gui the_interactive_overlay_is_on_and_has_no_key_to_restore_it <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    let mut config = FpsOverlayConfig {\n        enabled: true,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let mut config = FpsOverlayConfig {\n        enabled: false,\n'))
PY

mutation "snow stops falling" gui snow_falls_every_frame <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
# Anchored on the one symbol, for the reason given on the row above.
old = '            fall_snow,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

# The exact seam story 7.2's review found inert: the flag parsed, validated, and never reached
# the rig, while its only test was NAMED for reaching the camera setup.
mutation "--distance never reaches the camera rig" gui the_capture_distance_resource_reaches_the_camera_rig <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if let Some(distance) = distance {\n        rig.distance = distance.0.clamp(4.0, 500.0);'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if let Some(_distance) = distance {\n        rig.distance = rig.distance;'))
PY
