# Mutation set for story 7.2. Run alone: scripts/mutate.sh <this file>

mutation "designation projection is deleted" gui snapshot_marks_project_through_the_live_ingest_schedule <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
# Replace the whole derivation, not just its head: swapping the source for an empty slice
# orphans the .filter/.map/.collect chain and the mutation fails to COMPILE, which proves
# nothing. An empty map of the same type is the honest "this projection was deleted".
old = '''    let wanted_designations = mirror
        .designations()
        .iter()
        .filter(|designation| designation.pos[2] <= slice.level())
        .map(|designation| (designation.pos, designation.kind))
        .collect::<std::collections::BTreeMap<_, _>>();
'''
assert s.count(old) == 1
new = '    let wanted_designations = std::collections::BTreeMap::<[i32; 3], DesignationKind>::new();\n'
p.write_text(s.replace(old, new))
PY

mutation "zone projection is deleted" gui snapshot_marks_project_through_the_live_ingest_schedule <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
# Same correction as the designation row above: delete the whole derivation, not its head.
old = '''    let wanted_zones = mirror
        .zones()
        .iter()
        .filter(|zone| zone.pos[2] <= slice.level())
        .map(|zone| zone.pos)
        .collect::<BTreeSet<_>>();
'''
assert s.count(old) == 1
new = '    let wanted_zones = BTreeSet::<[i32; 3]>::new();\n'
p.write_text(s.replace(old, new))
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
old = '        if !wanted_designations.contains_key(&position) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && !wanted_designations.contains_key(&position) {\n'))
PY

mutation "kind changes do not restyle" gui a_designation_kind_change_restyles_the_existing_position_mark <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            if existing_kind != kind {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && existing_kind != kind {\n'))
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

mutation "zone absence no longer despawns" gui draw_count_instrument_follows_projected_marks_from_live_ingest <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
# The position-indexing refactor split despawn-on-absence into two loops, one per mark kind.
# The designation loop has its own row above; without this one the zone half is unpinned.
old = '        if !wanted_zones.contains(&position) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false && !wanted_zones.contains(&position) {\n'))
PY

mutation "a mark colour drifts into the terrain palette" gui mark_colours_are_distinct_cold_literals <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
# Move channel onto Material::Snow AND move the expected literal with it, so the exact-literal
# assertion still holds and the SEPARATION floor is the only thing left to catch it. Editing the
# production colour alone would kill on the literal check and prove nothing about the floor.
old = '        DesignationKind::Channel => Color::srgb_u8(86, 120, 214),\n'
assert s.count(old) == 1
s = s.replace(old, '        DesignationKind::Channel => Color::srgb_u8(136, 150, 178),\n')
old_expected = '                [86, 120, 214],\n'
assert s.count(old_expected) == 1
p.write_text(s.replace(old_expected, '                [136, 150, 178],\n'))
PY
