#!/usr/bin/env bash
# Sabotages for the asset-contract evidence channel.

mutation "the stale off-centre asset is accepted" py scripts.tests.test_check_asset.CheckAssetTests.test_off_centre_stale_asset_names_the_origin_clause <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '    if abs(centre_x) > 0.000_001 or abs(centre_z) > 0.000_001:\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if False:\n'))
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
