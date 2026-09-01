#!/usr/bin/env bash
# Sabotages for the asset-contract evidence channel.

mutation "the stale off-centre asset is accepted" py scripts.tests.test_check_asset.CheckAssetTests.test_off_centre_stale_asset_names_the_origin_clause <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '    elif abs(centre_x) > 0.000_001 or abs(centre_z) > 0.000_001:\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    elif False:\n'))
PY

mutation "a failed contract returns success" py scripts.tests.test_check_asset.CheckAssetTests.test_off_centre_stale_asset_names_the_origin_clause <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '        except AssetError as error:\n            print(f"FAIL {path}: {error}", file=sys.stderr)\n            return 1\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        except AssetError as error:\n            print(f"FAIL {path}: {error}", file=sys.stderr)\n            return 0\n'))
PY

mutation "reported triangle figures lie" py scripts.tests.test_check_asset.CheckAssetTests.test_the_four_published_pines_report_their_literal_figures <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '        f"tris={tris} verts={verts}"\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        f"tris=0 verts={verts}"\n'))
PY

mutation "a failed asset omits its figures" py scripts.tests.test_check_asset.CheckAssetTests.test_off_centre_stale_asset_names_the_origin_clause <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '            print(line, flush=True)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            pass\n'))
PY

mutation "off-grid positions are accepted" py scripts.tests.test_check_asset.CheckAssetTests.test_off_grid_positions_and_unapplied_transforms_are_rejected <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '        raise AssetError("grid clause: POSITION values must use the 0.1 m project grid")\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        pass\n'))
PY

mutation "unapplied transforms are accepted" py scripts.tests.test_check_asset.CheckAssetTests.test_off_grid_positions_and_unapplied_transforms_are_rejected <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '        raise AssetError("transform clause: mesh node must have an applied identity transform")\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        pass\n'))
PY

# --- Added by the 2026-09-01 code review: sabotage for the four clauses it repaired. ---

mutation "a parent node hides an unapplied transform" py scripts.tests.test_check_asset.CheckAssetTests.test_a_parent_node_cannot_hide_an_unapplied_transform <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '    for ancestor in ancestor_nodes(document, node_index):\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    for ancestor in []:\n'))
PY

mutation "a mismatched file basename is accepted" py scripts.tests.test_check_asset.CheckAssetTests.test_a_mismatched_file_basename_is_rejected <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '    elif path.stem != mesh_name:\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    elif False:\n'))
PY

mutation "non-finite positions crash instead of naming a clause" py scripts.tests.test_check_asset.CheckAssetTests.test_non_finite_positions_name_a_clause_instead_of_crashing <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '        raise AssetError("geometry clause: POSITION values must be finite")\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        pass\n'))
PY

mutation "the published palette is not read from the artifact" py scripts.tests.test_check_asset.CheckAssetTests.test_off_centre_stale_asset_names_the_origin_clause <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = "        f\"palette={','.join(palette)} \"\n"
assert s.count(old) == 1
p.write_text(s.replace(old, "        f\"palette={','.join(PALETTE_HEX)} \"\n"))
PY
