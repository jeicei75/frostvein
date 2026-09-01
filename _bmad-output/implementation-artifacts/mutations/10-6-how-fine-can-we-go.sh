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
old = '''    def carved_at(x, y, z):
        return (
            detailed'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''    def carved_at(x, y, z):
        return (
            False'''))
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
