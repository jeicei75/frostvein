#!/usr/bin/env bash
# Story 10.4's capture and mesh-tree witnesses. Each literal has an occurrence guard so a source
# refactor that strands this table is a failed mutation, not a claimed kill.

mutation "copied executable asset root wins over the build workspace" gui ingest::tests::asset_root_prefers_a_copied_executables_assets_then_the_stamped_workspace <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if beside_executable.is_dir() {\n        beside_executable\n    } else {'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false {\n        beside_executable\n    } else {'))
PY

mutation "zero spawned meshes cannot pass a treed capture" gui capture::tests::tree_capture_requires_loaded_scenes_and_every_rederived_mesh <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        spawned,\n        expected,\n        "capture spawned {spawned} tree meshes'
assert s.count(old) == 1
p.write_text(s.replace(old, '        expected,\n        expected,\n        "capture spawned {spawned} tree meshes'))
PY

mutation "cut oracle includes whole tree meshes above their source tiles" gui capture::tests::cut_oracle_counts_a_whole_tree_mesh_above_its_last_tile <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '    count + expected_tree_mesh_count(mirror, level)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    count\n'))
PY
