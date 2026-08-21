# Mutation set for story 7.2. Run alone: scripts/mutate.sh <this file>

mutation "designation projection is deleted" gui snapshot_marks_project_through_the_live_ingest_schedule <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    let wanted_designations = mirror\n        .designations()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let wanted_designations: &[protocol::Designation] = &[];\n'))
PY

mutation "zone projection is deleted" gui snapshot_marks_project_through_the_live_ingest_schedule <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    let wanted_zones = mirror\n        .zones()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let wanted_zones: &[protocol::Zone] = &[];\n'))
PY

mutation "mark slice filter is removed" gui marks_follow_the_slice_control_at_and_below_the_cut <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        .filter(|designation| designation.pos[2] <= slice.level())\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        .filter(|_designation| true)\n'))
PY

mutation "designation absence no longer despawns" gui cancellation_delta_despawns_a_missing_designation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        if !wanted_designations.contains_key(&mark.0) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && !wanted_designations.contains_key(&mark.0) {\n'))
PY

mutation "kind changes do not restyle" gui a_designation_kind_change_restyles_the_existing_position_mark <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            if existing_kind.0 != kind {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && existing_kind.0 != kind {\n'))
PY

mutation "capture accepts zero marks" gui draw_count_instrument_rejects_an_empty_level_and_accepts_terrain <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '''        assert!(self.designations > 0, "capture projected no designations");
        assert!(self.zones > 0, "capture projected no zones");
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        assert!(self.designations >= 0, "capture projected no designations");
        assert!(self.zones >= 0, "capture projected no zones");
'''))
PY

mutation "distance capture validation is disabled" gui capture_distance_requires_capture_and_reaches_the_camera_setup <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if distance.is_some() && capture.is_none() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false && distance.is_some() && capture.is_none() {\n'))
PY

mutation "mark systems leave the shared projection schedule" gui snapshot_marks_project_through_the_live_ingest_schedule <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '                reconcile_projection,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
