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
old = 'format!("Slice: z {}/{} — {}", self.level, self.top, self.label())'
assert s.count(old) == 1
p.write_text(s.replace(old, 'format!("Slice: z {}/{}", self.level, self.top)'))
PY

mutation "capture accepts an empty requested slice" gui capture::tests::draw_count_instrument_rejects_an_empty_level_and_accepts_terrain <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = 'self.terrain_tiles > 0'
assert s.count(old) == 1
p.write_text(s.replace(old, 'self.terrain_tiles >= 0'))
PY

mutation "capture z works without capture mode" gui ingest::tests::capture_slice_level_requires_capture_and_is_retained_for_pinning <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'if slice_level.is_some() && capture.is_none() {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if false && slice_level.is_some() && capture.is_none() {'))
PY
