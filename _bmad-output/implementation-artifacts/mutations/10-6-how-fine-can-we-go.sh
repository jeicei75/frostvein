mutation "greedy merge removal fails prism geometry" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_greedy_mesher_merges_a_two_cell_prism_with_hand_written_counts <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '    quads = sum(_greedy_quads(mask) for mask in masks.values())'
assert s.count(old) == 1
p.write_text(s.replace(old, '    quads = sum(len(mask) for mask in masks.values())'))
PY

mutation "detail rule removal fails subdivided geometry" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_detail_rule_changes_subdivided_counts_exactly_and_leaves_k_one_alone <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '''        return (
            detailed
            and material_at(x, y, z) is not None'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''        return (
            False
            and material_at(x, y, z) is not None'''))
PY

mutation "side-face carve removal re-inflates every k>1 row" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_face_count_matches_a_brute_force_fine_voxel_oracle <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '                        top = k if own is None else own[i][j]'
assert s.count(old) == 1
p.write_text(s.replace(old, '                        top = k'))
PY

mutation "cross-cell connector removal opens the fine surface" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_a_stepped_world_matches_hand_written_counts <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '                    other = None if open_side else heights_at(x + sx, y + sy, z)'
assert s.count(old) == 1
p.write_text(s.replace(old, '''                    if not open_side:
                        continue
                    other = None'''))
PY

mutation "unmasked multiply diverges from the client u32 rule" py scripts.tests.test_resolution_bench.ResolutionDetailRuleTests.test_detail_rule_is_seeded_and_has_a_hand_written_two_voxel_range <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '    value ^= (y & mask) * 0x85EBCA77 & mask'
assert s.count(old) == 1
p.write_text(s.replace(old, '    value ^= (y & mask) * 0x85EBCA77'))
PY

mutation "chunk count collapses back to two dimensions" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_chunks_are_counted_in_three_dimensions_from_emitted_geometry <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '                            z // CHUNK_EDGE_CELLS,'
assert s.count(old) == 1
p.write_text(s.replace(old, '                            0,'))
PY

mutation "k one control drift fails control assertion" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_control_check_requires_the_real_world_literals <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = 'CONTROL_QUADS = 19_264'
assert s.count(old) == 1
p.write_text(s.replace(old, 'CONTROL_QUADS = 19_263'))
PY

mutation "subdiv flag reaches chunk mesh instead of parsing inertly" gui ingest::tests::subdiv_flag_reaches_the_rendered_terrain_and_one_keeps_the_shipped_scene <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        if subdiv > 1 {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if false {\n'))
PY

mutation "drawn-set culling redraws faces buried in rock" gui project::tests::buried_rock_contributes_no_faces_from_either_side <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    position[2] <= level && matches!(mirror.tile(position), Some(Tile::Solid(_) | Tile::Ramp(_)))'
assert s.count(old) == 1
p.write_text(s.replace(old, '    position[2] <= level && false'))
PY

mutation "side faces ignore the pit that carved them away" gui project::tests::the_fine_mesher_reproduces_the_benchs_face_and_triangle_counts <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '                let top = column_height(own.as_ref(), du, dv, subdiv);'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let top = subdiv;'))
PY

mutation "cross-cell connectors are dropped and the fine surface cracks" gui project::tests::the_fine_mesher_reproduces_the_benchs_face_and_triangle_counts <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = """            let other = solid
                .then(|| column_heights(mirror, neighbour, subdiv, level))
                .flatten();"""
assert s.count(old) == 1
p.write_text(s.replace(old, '            let other = None;'))
PY

mutation "greedy tie-break drifts away from the bench's row order" gui project::tests::the_fine_mesher_reproduces_the_benchs_face_and_triangle_counts <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    ordered.sort_by_key(|&(u, v)| (v, u));'
assert s.count(old) == 1
p.write_text(s.replace(old, '    ordered.sort_by_key(|&(u, v)| (u, v));'))
PY

mutation "the client detail rule drifts off the bench's pinned vector" gui project::tests::the_detail_rule_matches_the_benchs_pinned_vector <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        ^ (u as u32).wrapping_mul(0x85EB_CA77)'
assert s.count(old) == 1
p.write_text(s.replace(old, '        ^ (u as u32).wrapping_mul(0x85EB_CA78)'))
PY

mutation "chunk cells go unrecorded and the capture oracle blinds again" gui project::tests::every_drawn_cell_is_recorded_on_exactly_one_chunk <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    mesh.cells.insert(cell);'
assert s.count(old) == 1
p.write_text(s.replace(old, '    let _ = cell;'))
PY
