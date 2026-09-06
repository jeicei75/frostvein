# Story 10.5 Part A sabotage table. Run alone: scripts/mutate.sh <this file>
#
# AC12 asks for at least three rows the run KILLS, one of them AC3's floor offset. Every row here
# targets a claim the story makes, not a line that happens to be easy to break.

# AC3, and the arm that actually shipped the bug. `apply_entity_blending` rewrites the translation
# of every projected entity on every frame after the spawn, so dropping the offset HERE leaves the
# spawn correct and lifts the dwarf half a cell from the next frame on -- which is exactly what
# Wolf saw from the seat while every test was green.
mutation "the blend arm forgets the dwarf's floor offset" gui the_dwarf_stands_on_the_cell_floor_and_stays_there_after_a_blend <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '            ) + entity_draw_offset(entity.kind);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            );\n'))
PY

# AC3's other half. The spawn is the easy half to get right and the easy half to assume is enough;
# this row proves the test is not passing on the blend arm alone.
mutation "the spawn arm forgets the dwarf's floor offset" gui the_dwarf_stands_on_the_cell_floor_and_stays_there_after_a_blend <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                                world_to_render(position) + entity_draw_offset(mirror_entity.kind),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                                world_to_render(position),\n'))
PY

# The offset must be the DWARF's alone. A fix of the shape "drop everything" passes the dwarf test
# and sinks every cube kind half a cell into the ground.
mutation "every kind gets the floor offset, not just the dwarf" gui the_floor_drop_does_not_move_the_cube_kinds <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '''    match kind {
        EntityKind::Dwarf => -Vec3::Y * 0.5,
        _ => Vec3::ZERO,
    }'''
assert s.count(old) == 1
p.write_text(s.replace(old, '    -Vec3::Y * 0.5'))
PY

# AC4. The scale is pinned on BOTH sides of the bench contract by a source-text grep, so a client
# that drifts from the bench must be caught rather than quietly rendering a differently sized dwarf
# than the bench draws.
mutation "the client's dwarf scale drifts from the bench" gui bench_literals_match_the_client_palette_lights_and_boot_camera <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '''            color: Color::srgb_u8(151, 116, 96),
            scale: 0.75,'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''            color: Color::srgb_u8(151, 116, 96),
            scale: 0.65,'''))
PY

# AC10. The instrument must report what was DRAWN. A hardcoded count is the failure shape a
# well-formedness assertion cannot see, which is why the test varies the slice and reads the number.
mutation "the dwarf startup line reports a constant" gui the_dwarf_startup_line_reports_what_was_actually_drawn <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    let dwarves = scene_entities.iter().count();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let dwarves = 5;\n'))
PY
