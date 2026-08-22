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

mutation "a kind change does not update the mark's kind component" gui a_designation_kind_change_restyles_the_existing_position_mark <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            if existing_kind != kind {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && existing_kind != kind {\n'))
PY

mutation "the mark oracle is made to compare the draw set with itself" gui mark_counts_are_checked_against_the_mirror_not_merely_against_zero <<'PYX'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
# The `> 0` this replaced could not see a projection that drops SOME of its marks. Turning the
# oracle into a self-comparison restores exactly that blindness while still "asserting" something.
old = '            self.designations, self.expected_designations,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            self.designations, self.designations,\n'))
PYX

mutation "a working-site capture accepts a frame whose marks were all consumed" gui draw_count_instrument_follows_projected_marks_from_live_ingest <<'PYX'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
# AC13's "exit 0 is not a result", against this story's measured trap: the dwarves consume the
# designations, so a capture aimed at a small site arrives after they are gone and photographs
# nothing it came to see.
old = '                self.expected_designations > 0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                self.expected_designations >= 0,\n'))
PYX

mutation "distance capture validation is disabled" gui capture_distance_requires_capture_and_is_retained_for_pinning <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if distance.is_some() && capture.is_none() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false && distance.is_some() && capture.is_none() {\n'))
PY

mutation "all projection leaves the shared registration point" gui snapshot_marks_project_through_the_live_ingest_schedule <<'PY'
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
old = '        DesignationKind::Channel => Color::srgb_u8(150, 96, 230),\n'
assert s.count(old) == 1
s = s.replace(old, '        DesignationKind::Channel => Color::srgb_u8(136, 150, 178),\n')
old_expected = '                [150, 96, 230],\n'
assert s.count(old_expected) == 1
p.write_text(s.replace(old_expected, '                [136, 150, 178],\n'))
PY

# ---------------------------------------------------------------------------------------------
# Added at the 2026-08-21 code review. Every row below pins a seam an AC names that the original
# table could not reach -- three of them were found by sabotaging the SHIPPED code in a scratch
# clone and watching the whole suite stay green.

mutation "a kind change does not restyle the mark's material" gui a_designation_kind_change_restyles_the_existing_position_mark <<'PYX'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
# AC10's named property is the STYLE, and the original table sabotaged the kind-COMPONENT guard
# instead. Deleting this insert left the whole suite green: a dig retuned to a channel kept dig
# blue forever, with the table reporting the seam covered.
old = """            if let Some(assets) = assets {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(assets.designation_material(kind)));
            }
"""
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PYX

mutation "the distance flag never reaches the camera rig" gui the_distance_flag_reaches_the_camera_rig_rather_than_merely_parsing <<'PYX'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
# The flag parsed, validated, and was then ignored, with all 106 tests green -- while its only
# test was NAMED for reaching the camera setup and asserted `parse_args_from` alone.
old = """    if let Some(distance) = distance {
        rig.distance = distance.0.clamp(4.0, 500.0);
    }
"""
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = distance;\n'))
PYX

mutation "buried digs are sealed back inside the rock" gui a_dig_buried_under_the_cut_is_drawn_on_the_rock_that_covers_it <<'PYX'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
# Restores the defect that made the story's own recipe photograph an empty site and exit 0:
# 0 of 50 surviving marks visible, while the instrument correctly printed designations=50.
old = '    while top < level && is_visible_at_slice(mirror, [x, y, top + 1], level) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    while false && is_visible_at_slice(mirror, [x, y, top + 1], level) {\n'))
PYX

mutation "a stockpile over a dig shares the dig's surface" gui a_stockpile_over_a_dig_does_not_share_the_digs_surface <<'PYX'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
# Both slabs then project to byte-identical translations and scales with the same opaque mesh.
old = """                || wanted_designations.get(&[position[0], position[1], position[2] - 1])
                    == Some(&DesignationKind::Dig)
"""
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PYX

mutation "a mark colour drifts onto the TUI's colour for a different order" gui mark_colours_are_distinct_cold_literals <<'PYX'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
# Dig SHIPPED byte-identical to the TUI's CHANNEL blue. The terrain floor cannot see this: the
# collision is with the OTHER client, which Wolf reads side by side with this one. As with the
# terrain row, the expected literal moves too, so only the cross-client floor is left to catch it.
old = '        DesignationKind::Dig => Color::srgb_u8(56, 132, 250),\n'
assert s.count(old) == 1
s = s.replace(old, '        DesignationKind::Dig => Color::srgb_u8(92, 174, 224),\n')
old_expected = '                [56, 132, 250],\n'
assert s.count(old_expected) == 1
p.write_text(s.replace(old_expected, '                [92, 174, 224],\n'))
PYX
