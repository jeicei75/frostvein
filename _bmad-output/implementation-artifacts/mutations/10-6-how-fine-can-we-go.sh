mutation "greedy merge removal fails prism geometry" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_greedy_mesher_merges_a_two_cell_prism_with_hand_written_counts <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '        quads += sum(_greedy_quads(mask) for mask in masks.values())'
assert s.count(old) == 1
p.write_text(s.replace(old, '        quads += sum(len(mask) for mask in masks.values())'))
PY

mutation "detail rule removal fails subdivided geometry" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_detail_rule_increases_subdivided_quads_but_not_the_k_one_control <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '''                    if detail and k > 1 and axis == 2 and sign > 0:
                        # A pit removes top fine voxels; its bottom remains solid. This is a'''
assert s.count(old) == 1
p.write_text(s.replace(old, '''                    if False:
                        # A pit removes top fine voxels; its bottom remains solid. This is a'''))
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
