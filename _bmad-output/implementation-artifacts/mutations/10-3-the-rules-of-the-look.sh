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
old = '        f"tris={tris} verts={verts} mesh={mesh_name} profile={profile} "\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        f"tris=0 verts={verts} mesh={mesh_name} profile={profile} "\n'))
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
old = '''            raise AssetError(
                f"grid clause: POSITION values must use the {PROJECT_GRID_METRES} m project grid"
            )
'''
assert s.count(old) == 1
p.write_text(s.replace(old, '            pass\n'))
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
old = '    elif path.stem != REVISION_SUFFIX.sub("", mesh_name):\n'
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

# --- Added for round 18's walk cycle: the animation clauses. Until these existed the
# --- checker had NO animation clause at all, so a clip that was inert or that drifted
# --- passed every gate in the repo.

mutation "a clip that does not close its loop is accepted" py scripts.tests.test_check_asset.AnimationClauseTests.test_a_vertical_bob_closes_but_a_drift_does_not <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '            closed = all(abs(a - b) <= 1e-5 for a, b in zip(first, last))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            closed = True\n'))
PY

mutation "a single-keyframe channel is accepted as animation" py scripts.tests.test_check_asset.AnimationClauseTests.test_a_single_keyframe_channel_cannot_animate_and_is_rejected <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '            if len(times) < 2:\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if False:\n'))
PY

mutation "a clip with no channels is accepted as inert" py scripts.tests.test_check_asset.AnimationClauseTests.test_a_clip_with_no_channels_is_rejected_as_inert <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/check_asset.py'); s = p.read_text()
old = '        if not channels:\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if False:\n'))
PY
