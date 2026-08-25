# Mutation set for story 8.1. Run alone: scripts/mutate.sh <this file>

mutation "pick system leaves the shared client schedule" gui a_cursor_at_a_visible_tiles_independent_projection_picks_that_tile <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '            update_pick.after(apply_scripted_cursor),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "slice visibility is removed from the march" gui picking_nothing_leaves_no_hover_for_sky_hidden_tiles_and_outside_the_window <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = '        if mirror.tile(world).is_some() && is_visible_at_slice(mirror, world, level) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if mirror.tile(world).is_some() {\n'))
PY

mutation "render-to-world is replaced by raw render axes" gui a_cursor_at_a_visible_tiles_independent_projection_picks_that_tile <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = '        let world = render_to_world(centre);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let world = [centre.x as i32, centre.y as i32, centre.z as i32];\n'))
PY

mutation "no-pick falls back to the origin" gui picking_nothing_leaves_no_hover_for_sky_hidden_tiles_and_outside_the_window <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/pick.rs'); s = p.read_text()
old = '    picked.0 = window.cursor_position().and_then(|cursor| {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    picked.0 = window.cursor_position().and_then(|cursor| {\n'))
old = '    });\n}\n\nfn first_visible_hit('
assert s.count(old) == 1
p.write_text(s.replace(old, '    }).or(Some([0, 0, 0]));\n}\n\nfn first_visible_hit('))
PY

mutation "hover survives when no tile is picked" gui the_live_pick_spawns_a_client_local_highlight_and_despawns_it_without_a_pick <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '''    } else {
        for (entity, _) in highlights.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn terrain_standard_material'''
assert s.count(old) == 1
new = '''    } else {
    }
}

fn terrain_standard_material'''
p.write_text(s.replace(old, new))
PY

mutation "cursor parses but never reaches the pick" gui the_cursor_flag_reaches_a_live_resource_rather_than_merely_parsing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        app.insert_resource(ScriptedCursor(cursor));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let _ = cursor;\n'))
PY
