# Story 12.8: Dwarves at work, the cut. #164: a hauler's item is lifted only once his drawn body
# reaches its cell, and released only once he is drawn at the cell the wire drops it on; the drawn
# walker keeps pace with the sim speed. Run alone, after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-8.sh
# Row names follow the story's Task 6 list.

mutation "pick-up ungated" gui a_hauler_does_not_pick_up_his_stone_until_he_is_drawn_at_its_cell <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                    && !walking_in.contains_key(&dwarf) =>\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    && (true || !walking_in.contains_key(&dwarf)) =>\n'))
PY

mutation "Fast walker unscaled" gui a_dwarf_walking_in_at_fast_reaches_his_work_clip_within_the_run <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        protocol::Speed::Fast => 5.0,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        protocol::Speed::Fast => 1.0,\n'))
PY

mutation "Cut mapped back to Dig" gui a_woodcutter_on_a_cut_job_in_work_gets_the_cut_clip <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        DwarfClip::Cut\n    } else if entity.carrying.is_some() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DwarfClip::Dig\n    } else if entity.carrying.is_some() {\n'))
PY

mutation "Cut mapped back to Dig, seen by the real binary" gui a_miner_logs_dig_and_a_hauler_logs_carry_from_a_real_daemon ignored <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        DwarfClip::Cut\n    } else if entity.carrying.is_some() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DwarfClip::Dig\n    } else if entity.carrying.is_some() {\n'))
PY

mutation "a cut swings at a dig's period" gui a_cut_swing_advances_at_half_the_rate_of_a_dig_swing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            DwarfClip::Cut => 10,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            DwarfClip::Cut => 5,\n'))
PY

mutation "cut-mode crown fall-through restored" gui in_cut_mode_a_ray_through_a_crown_resolves_to_that_tree <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = '            if cut && (is_tree_tile(mirror, world) || is_tree_foliage(mirror, world)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false && cut && (is_tree_tile(mirror, world) || is_tree_foliage(mirror, world)) {\n'))
PY

# Review run 1 patch: in cut mode a trunk hit reached the pick as itself with a side face. This
# puts back exactly the pre-patch foliage-only guard.
mutation "cut-mode trunk hit keeps its side face" gui in_cut_mode_a_ray_through_a_crown_resolves_to_that_tree <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = '            if cut && (is_tree_tile(mirror, world) || is_tree_foliage(mirror, world)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if cut && is_tree_foliage(mirror, world) {\n'))
PY

# Review run 2 patch: the trunk half's guard must fire when no ray meets the bare trunk first.
# Drops the only trunk-first target; the trace then counts 0.
mutation "cut-mode trunk target dropped" gui in_cut_mode_a_ray_through_a_crown_resolves_to_that_tree <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = '                [60, 60, 2],\n            ] {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            ] {\n'))
PY

mutation "the box filtered back to trees" gui the_cut_preview_covers_every_rect_cell_and_only_tree_cells_are_in_the_cut_style <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        DesignateMode::Cut | DesignateMode::Clear | DesignateMode::None => true,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Cut => crate::designate::is_tree_tile(mirror, tile),\n        DesignateMode::Clear | DesignateMode::None => true,\n'))
PY

mutation "the box's open cells drawn in the cut style" gui the_cut_preview_covers_every_rect_cell_and_only_tree_cells_are_in_the_cut_style <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                assets.hover_highlight.clone()\n            },\n        ),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                assets.cut_mark.clone()\n            },\n        ),\n'))
PY

mutation "the tint never cleared" gui a_cut_marked_pine_wears_the_shared_tint_and_keeps_it_across_a_respawn <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            if let Some(own) = own {\n                material.0 = own.0.clone();\n            }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let _ = own;\n'))
PY

mutation "a respawned pine is not re-tinted" gui a_cut_marked_pine_wears_the_shared_tint_and_keeps_it_across_a_respawn <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    let mut targets: Vec<(BevyEntity, bool)> = Vec::new();\n    for mesh in meshes.p0().iter() {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let mut targets: Vec<(BevyEntity, bool)> = Vec::new();\n    for mesh in meshes.p0().iter().filter(|_| false) {\n'))
PY

mutation "a live cut drag tints nothing" gui a_live_cut_drag_tints_the_pines_it_catches_and_no_other_mode_does <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    if drag_mode.is_some_and(|mode| mode.0 == Some(DesignateMode::Cut)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false && drag_mode.is_some_and(|mode| mode.0 == Some(DesignateMode::Cut)) {\n'))
PY

mutation "a drag in any mode tints" gui a_live_cut_drag_tints_the_pines_it_catches_and_no_other_mode_does <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    if drag_mode.is_some_and(|mode| mode.0 == Some(DesignateMode::Cut)) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if drag_mode.is_some_and(|mode| mode.0.is_some()) {\n'))
PY

mutation "the materials line printing a constant" gui the_trees_materials_line_counts_pines_per_material_and_follows_the_marks <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '''    let line = format!(
        "gui trees: materials={} [{}]",
        handles.len(),
        labels.join(", ")
    );
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = (&handles, &labels);\n    let line = "gui trees: materials=4 [M_VoxelPine:259]".to_string();\n'))
PY

mutation "the tint is not told apart in the materials line" gui the_trees_materials_line_counts_pines_per_material_and_follows_the_marks <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        let label = if tint.as_ref().is_some_and(|tint| tint.0 == material.0) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let label = if false && tint.as_ref().is_some_and(|tint| tint.0 == material.0) {\n'))
PY

mutation "the materials line never updates" gui the_trees_materials_line_counts_pines_per_material_and_follows_the_marks <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    if report.0 != line {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if report.0.is_empty() {\n'))
PY

mutation "CUT_OFFSET never applied" gui a_cutting_dwarfs_armature_is_nudged_toward_the_trunk_and_his_entity_is_not <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        transform.translation = if chosen == DwarfClip::Cut {\n            rest + Vec3::new(0.0, 0.0, -CUT_OFFSET)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        transform.translation = if chosen == DwarfClip::Cut {\n            rest\n'))
PY

mutation "the cut nudge never cleared" gui a_cutting_dwarfs_armature_is_nudged_toward_the_trunk_and_his_entity_is_not <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        } else {\n            rest\n        };\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        } else {\n            transform.translation\n        };\n'))
PY
