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
old = '            && is_visible_at_slice(mirror, world, level)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
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

# Added at review-patch round 1 (2026-08-26). AC13 asks the table to cover every seam AC; the six
# rows above match Task 5's stated minimum and stop there, leaving AC9's only structural clause,
# AC5's separation floor, and the wiring CALL SITE the review found inert unpinned.

mutation "the hover highlight is spawned without its client-local tag" gui the_live_pick_spawns_a_client_local_highlight_and_despawns_it_without_a_pick <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                MeshMaterial3d(assets.hover_highlight.clone()),\n                ClientLocal,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                MeshMaterial3d(assets.hover_highlight.clone()),\n'))
PY

mutation "the hover colour drifts to a near-neighbour of the dig mark" gui hover_highlight_colour_is_a_distinct_cold_literal <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '    Color::srgb_u8(80, 220, 210)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    Color::srgb_u8(56, 140, 240)\n'))
PY

mutation "the capture flags are wired by a call run() never makes" gui the_production_wiring_runs_every_call_run_makes_after_its_plugins <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    insert_capture_resources(app, &args);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
