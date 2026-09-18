# Story 11.1a sabotage table. Run with scripts/mutate.sh <this file>.

mutation "the live camera keeps multisample antialiasing enabled" gui configured_camera_disables_msaa_on_the_live_rig <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Msaa::Off,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the live camera discards its chosen exposure" gui configured_camera_carries_the_chosen_ev100_on_the_live_rig <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            Exposure { ev100: 9.7 },\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
