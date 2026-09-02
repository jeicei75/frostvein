#!/usr/bin/env bash
# Story 10.4's capture and mesh-tree witnesses. Each literal has an occurrence guard so a source
# refactor that strands this table is a failed mutation, not a claimed kill.

mutation "every pine is embedded in the binary" gui ingest::tests::every_tree_variant_is_embedded_in_the_binary_as_a_real_glb <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'include_bytes!("../../../assets/trees/SM_VoxelPine_Tree04R.glb"),'
assert s.count(old) == 1
# An empty blob is exactly what the filesystem loader used to produce silently: a handle that
# resolves to nothing. The row must redden on CONTENT, not merely on the count of entries.
p.write_text(s.replace(old, 'b"",'))
PY

mutation "embedded table and loader agree which pine is which" gui ingest::tests::tree_asset_paths_match_the_loader <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    "trees/SM_VoxelPine_Tree01.glb",\n    "trees/SM_VoxelPine_Tree02.glb",'
assert s.count(old) == 1
# Swap two variants in the loader only. Every tree still draws, as the wrong species, and
# nothing else in the suite can see it.
p.write_text(s.replace(old, '    "trees/SM_VoxelPine_Tree02.glb",\n    "trees/SM_VoxelPine_Tree01.glb",'))
PY

mutation "zero spawned meshes cannot pass a treed capture" gui capture::tests::tree_capture_requires_loaded_scenes_and_every_rederived_mesh <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        spawned, expected,\n        "capture spawned {spawned} tree meshes'
assert s.count(old) == 1
# Compare expected against itself: the count check becomes a tautology while still reading like
# a real assertion, which is how a zero-mesh capture would have passed.
p.write_text(s.replace(old, '        expected, expected,\n        "capture spawned {spawned} tree meshes'))
PY

mutation "cut oracle includes whole tree meshes above their source tiles" gui capture::tests::cut_oracle_counts_a_whole_tree_mesh_above_its_last_tile <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '    count + expected_tree_mesh_count(mirror, level)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    count\n'))
PY
