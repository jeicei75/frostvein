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
old = '            setup_night_lighting,\n            setup_projection_assets,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            setup_projection_assets,'))
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
# RE-ANCHORED 2026-09-03 (story 10.7): light_controls now sits between these two. The row still
# drops camera_controls out of the update tuple and nothing else.
old = '            camera_controls,\n            light_controls,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            light_controls,'))
PY

mutation "fog stops following the camera" gui fog_follows_the_camera_rig_every_frame <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            update_fog_from_camera,\n            toggle_overlay,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            toggle_overlay,'))
PY

mutation "the F3 overlay toggle is never registered" gui f3_toggles_the_diagnostic_overlay <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            toggle_overlay,\n            fall_snow,'
assert s.count(old) == 1
p.write_text(s.replace(old, '            fall_snow,'))
PY

mutation "snow stops falling" gui snow_falls_every_frame <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            toggle_overlay,\n            fall_snow,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            toggle_overlay,\n'))
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
