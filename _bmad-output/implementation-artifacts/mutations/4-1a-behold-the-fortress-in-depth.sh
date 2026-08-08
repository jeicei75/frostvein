# Mutation set for story 4.1a. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/4-1a-behold-the-fortress-in-depth.sh
#
# ONE MUTATION THE STORY NAMED IS NOT HERE, and the reason matters more than the entry
# would. "The dwarf index ignores `EntityKind`" cannot be killed, because
# `protocol::EntityKind` has exactly ONE variant (`Dwarf`,
# crates/protocol/src/lib.rs:36-38). Deleting `if entity.kind == EntityKind::Dwarf` is a
# semantic no-op: every entity on the wire is a dwarf, so no snapshot can be built that
# the two versions disagree about. This is NOT 3.3's rejected "no scenario can tell them
# apart" argument -- that one fell to a unit test one level down. Here the type system
# has a single inhabitant, so there is no level at which a test could observe it. The
# filter stays in the code as the seam 4.1b widens, carrying the same `// NOTE:` the flat
# view's contention rules carry. Re-add this mutation the moment a second variant exists.

mutation "v does not toggle the view" tui v_toggles_the_depth_view_from_normal_and_back <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''            state.view = match state.view {
                View::Flat => View::Depth,
                View::Depth => View::Flat,
            };
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "v toggles out of a designation mode" tui v_is_ignored_in_every_designation_mode <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "        KeyCode::Char('v') if state.mode == Mode::Normal => {"
assert old in s
p.write_text(s.replace(old, "        KeyCode::Char('v') => {"))
PY

mutation "turning goes the wrong way round" tui turning_walks_the_eight_headings_and_wraps_both_ways <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''            if state.view == View::Depth {
                state.heading = (state.heading + 7) % 8;'''
assert old in s
new = '''            if state.view == View::Depth {
                state.heading = (state.heading + 1) % 8;'''
p.write_text(s.replace(old, new))
PY

mutation "turning does not wrap" tui turning_walks_the_eight_headings_and_wraps_both_ways <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "                state.heading = (state.heading + 1) % 8;"
assert old in s
p.write_text(s.replace(old, "                state.heading += 1;"))
PY

mutation "forward reads the next heading_step entry" tui forward_then_back_returns_to_the_starting_tile <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "    let (dx, dy) = crate::raycast::heading_step(state.heading);"
assert old in s
new = "    let (dx, dy) = crate::raycast::heading_step(state.heading + 1);"
p.write_text(s.replace(old, new))
PY

mutation "forward is not clamped to the world" tui forward_clamps_at_the_world_edge <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''    state.camera.0 = (state.camera.0 + dx * sign).clamp(0, i64::from(dims.x.saturating_sub(1)));
    state.camera.1 = (state.camera.1 + dy * sign).clamp(0, i64::from(dims.y.saturating_sub(1)));'''
assert old in s
new = '''    state.camera.0 += dx * sign;
    state.camera.1 += dy * sign;'''
p.write_text(s.replace(old, new))
PY

mutation "d enters dig mode from the depth view" tui designation_keys_do_nothing_in_the_depth_view <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "            if state.mode == Mode::Normal && state.view == View::Flat =>"
assert old in s
p.write_text(s.replace(old, "            if state.mode == Mode::Normal =>"))
PY

mutation "render always dispatches the flat view" tui the_two_views_draw_different_pictures_of_the_same_world <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = "        View::Depth => crate::raycast::draw(snapshot, state, w, map_h, &mut framebuffer.cells),"
assert old in s
new = "        View::Depth => draw_flat(snapshot, state, w, map_h, &mut framebuffer.cells),"
p.write_text(s.replace(old, new))
PY

mutation "the status line omits the view and heading" tui the_depth_status_line_reports_the_view_and_the_heading <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '''            View::Depth => format!(
                "tick {}  {}  3d {}  z {}/{}  dwarves {}",
                snapshot.tick,
                speed,
                crate::raycast::heading_name(state.heading),'''
assert old in s
new = '''            View::Depth => format!(
                "tick {}  {}  z {}/{}  dwarves {}",
                snapshot.tick,
                speed,'''
p.write_text(s.replace(old, new))
PY

mutation "the depth hint advertises d c p x" tui hint_bar_names_every_modes_keys_and_fits_eighty_columns <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
old = '        return "depth: hl turn  kj walk  <> z  v flat view  q quit client";'
assert old in s
new = '        return "depth: d dig  c channel  p stockpile  x clear  v flat view  q quit";'
p.write_text(s.replace(old, new))
PY

mutation "the DDA drops its z component" tui cast_reports_the_face_it_crossed_on_each_axis <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = "    let direction = [direction.0, direction.1, direction.2];"
assert old in s
p.write_text(s.replace(old, "    let direction = [direction.0, direction.1, 0.0];"))
PY

mutation "the DDA indexes tiles without the bounds check" tui a_ray_that_leaves_the_world_draws_blank <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = '''        if voxel[0] < 0
            || voxel[1] < 0
            || voxel[2] < 0
            || voxel[0] >= i64::from(dims.x)
            || voxel[1] >= i64::from(dims.y)
            || voxel[2] >= i64::from(dims.z)
        {
            return Cast { hit: None, steps };
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "the step cap is removed" tui a_ray_into_open_air_stops_at_the_step_cap <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = '''        if steps >= MAX_RAY_STEPS {
            return Cast { hit: None, steps };
        }
'''
assert old in s
p.write_text(s.replace(old, ''))
PY

mutation "the band ramp collapses to one glyph" tui a_wall_lands_in_a_band_and_moves_to_a_nearer_one <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = "const BAND_GLYPHS: [char; 4] = ['█', '▓', '▒', '░'];"
assert old in s
p.write_text(s.replace(old, "const BAND_GLYPHS: [char; 4] = ['█', '█', '█', '█'];"))
PY

mutation "face shading is removed" tui a_downward_ray_is_darkened_by_the_face_it_crosses <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = "const FACE_SHADE: [u16; 3] = [100, 78, 60];"
assert old in s
p.write_text(s.replace(old, "const FACE_SHADE: [u16; 3] = [100, 100, 100];"))
PY

mutation "the hit colour comes from a second hardcoded table" tui the_hit_colour_is_the_palette_entry_shaded <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = "                        What::Terrain(tile) => tile_cell(tile).fg,"
assert old in s
new = '''                        What::Terrain(tile) => match tile {
                            Tile::Empty => (0, 0, 0),
                            Tile::Solid(_) => (120, 120, 120),
                            Tile::Ramp(_) => (140, 140, 140),
                        },'''
p.write_text(s.replace(old, new))
PY

mutation "the dwarf index is built and its lookup discarded" tui a_dwarf_on_the_ray_is_drawn_instead_of_the_terrain <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = '''        if let Some(job) = dwarves.get(&position) {
            return Cast {
                hit: Some(Hit {
                    what: What::Dwarf(*job),
                    distance,
                    face,
                }),
                steps,
            };
        }
'''
assert old in s
# The seam stays PRESENT and INERT -- the us-09 dead-call shape this must catch.
new = '''        let _ = dwarves.get(&position);
'''
p.write_text(s.replace(old, new))
PY

mutation "a shared tile goes to the highest id" tui the_dwarf_index_gives_a_shared_tile_to_the_lowest_id <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = "            index.entry(entity.pos).or_insert(entity.state);"
assert old in s
p.write_text(s.replace(old, "            index.insert(entity.pos, entity.state);"))
PY

mutation "a miss draws a band glyph instead of BLANK" tui a_ray_that_leaves_the_world_draws_blank <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = "                None => BLANK,"
assert old in s
new = '''                None => Cell {
                    glyph: BAND_GLYPHS[3],
                    fg: BLANK.fg,
                },'''
p.write_text(s.replace(old, new))
PY

mutation "the ray angle ignores the heading" tui every_heading_sees_the_wall_placed_in_its_own_direction <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/raycast.rs'); s = p.read_text()
old = "    let (step_x, step_y) = heading_step(state.heading);"
assert old in s
p.write_text(s.replace(old, "    let (step_x, step_y) = heading_step(0);"))
PY

mutation "shade returns the colour unchanged" tui dim_darkens_monotonically <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/palette.rs'); s = p.read_text()
old = '''pub fn shade(fg: Rgb, percent: u16) -> Rgb {
    (
        (u16::from(fg.0) * percent / 100) as u8,
        (u16::from(fg.1) * percent / 100) as u8,
        (u16::from(fg.2) * percent / 100) as u8,
    )
}'''
assert old in s
new = '''pub fn shade(fg: Rgb, _percent: u16) -> Rgb {
    fg
}'''
p.write_text(s.replace(old, new))
PY

mutation "named_key forgets v" tui every_instrument_key_name_is_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
old = '''        "v" => Some(KeyCode::Char('v')),
'''
assert old in s
p.write_text(s.replace(old, '', 1))
PY
