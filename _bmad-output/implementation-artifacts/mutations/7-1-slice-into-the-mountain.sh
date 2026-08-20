# Story 7.1 sabotage table. Run alone with scripts/mutate.sh.

mutation "cut face no longer fills buried terrain" gui keyboard_slice_rebuilds_the_cut_face_and_hides_surface_entities <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    position[2] <= level\n        && (is_exposed(mirror, position)\n            || (position[2] == level\n                && matches!(mirror.tile(position), Some(Tile::Solid(_) | Tile::Ramp(_)))))'
assert s.count(old) == 1
p.write_text(s.replace(old, '    position[2] <= level && is_exposed(mirror, position)'))
PY

mutation "slice no longer hides surface entities" gui keyboard_slice_rebuilds_the_cut_face_and_hides_surface_entities <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '.filter(|entity| entity.pos[2] <= slice.level())'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "slice input stops requesting the established rebuild path" gui keyboard_slice_rebuilds_the_cut_face_and_hides_surface_entities <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if changed {\n        work.snapshot = true;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "slice can rise above the world top" gui slice::tests::the_slice_starts_at_the_top_and_clamps_at_both_world_bounds <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/slice.rs'); s = p.read_text()
old = 'requested.clamp(0, self.top)'
assert s.count(old) == 1
p.write_text(s.replace(old, 'requested.max(0)'))
PY

mutation "slice readout loses its underground state" gui slice::tests::the_readout_names_the_current_level_and_whether_it_is_surface_or_underground <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/slice.rs'); s = p.read_text()
old = 'self.label(covered)'
assert s.count(old) == 1
p.write_text(s.replace(old, '""'))
PY

mutation "capture accepts an empty requested slice" gui capture::tests::draw_count_instrument_rejects_an_empty_level_and_accepts_terrain <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'self.terrain_tiles > 0'
assert s.count(old) == 1
p.write_text(s.replace(old, 'self.terrain_tiles >= 0'))
PY

mutation "the --z flag parses but never reaches the slice resource" gui ingest::tests::the_z_flag_reaches_the_slice_resource_rather_than_merely_parsing <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        |level| SliceLevel::pinned(dims, level),'
assert s.count(old) == 1
p.write_text(s.replace(old, '        |_level| SliceLevel::at_world_top(dims),'))
PY

# --- Added by the code review of 2026-08-19. Every mutation below was EXECUTED against a copy of
# --- the tree by a review layer and left the WHOLE SUITE GREEN. They are the sabotage the original
# --- table could not kill: the table's 7 mutations were all genuinely killed, but "every mutation
# --- is killed" was true of the table and not of the story's new code.

mutation "the level readout is never drawn" gui the_level_readout_is_drawn_on_the_live_path_and_follows_the_cut <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '        .add_systems(Startup, setup_slice_readout)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
mutation "the level readout never follows the cut" gui the_level_readout_is_drawn_on_the_live_path_and_follows_the_cut <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '.add_systems(Update, update_slice_readout.after(ProjectionSet))'
assert s.count(old) == 1
p.write_text(s.replace(old, '.add_systems(Update, || {})'))
PY
mutation "the readout calls any cut below the top underground" gui slice::tests::the_label_follows_cover_rather_than_position <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/slice.rs'); s = p.read_text()
old = 'if covered { "underground" } else { "surface" }'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if self.level == self.top { "surface" } else { "underground" }'))
PY
mutation "items float above the cut" gui items_above_the_cut_are_hidden_with_the_entities <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        .filter(|item| item.pos[2] <= slice.level())\n'
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
mutation "the capture accepts a hollow cut with no floor" gui capture::tests::draw_count_instrument_rejects_a_hollow_cut_that_kept_its_total_up <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'self.cut_face_tiles, self.expected_cut_face,'
assert s.count(old) == 1
p.write_text(s.replace(old, 'self.cut_face_tiles, self.cut_face_tiles,'))
PY
mutation "lantern assertions key off any dwarf, not one below the cut" gui capture::tests::lantern_assertions_follow_the_mirror_not_the_observation <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '.any(|entity| entity.kind == EntityKind::Dwarf && entity.pos[2] <= level)'
assert s.count(old) == 1
p.write_text(s.replace(old, '.any(|entity| entity.kind == EntityKind::Dwarf)'))
PY

# --- Added 2026-08-20 from the live vehicle session. The z 9 capture panicked on the inherited
# --- ground-luminance floor and wrote NO PNG: `save_to_disk` and the range checks were two
# --- observers on one event and Bevy ran the checks first. The failing run destroyed its own
# --- evidence, which is the exact inverse of this instrument's "exit 0 is not a result" rule.

mutation "a failing range check destroys the frame that would explain it" gui capture::tests::the_capture_is_written_before_it_is_judged <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'fn save_before_validate(save: impl FnOnce(), validate: impl FnOnce()) {\n    save();\n    validate();\n}'
assert s.count(old) == 1
p.write_text(s.replace(old, 'fn save_before_validate(save: impl FnOnce(), validate: impl FnOnce()) {\n    validate();\n    save();\n}'))
PY

mutation "the calibrated band is skipped at full depth too" gui capture::tests::the_calibrated_band_judges_the_boot_framing_and_stands_aside_at_a_cut <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '    slice.level() >= slice.top()\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    false\n'))
PY

mutation "a cut is still judged against the boot-framing band" gui capture::tests::the_calibrated_band_judges_the_boot_framing_and_stands_aside_at_a_cut <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '    if !band_applies {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false {\n'))
PY
