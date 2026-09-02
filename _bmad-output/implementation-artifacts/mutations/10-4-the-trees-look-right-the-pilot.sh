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

# --- added at the 2026-09-02 code review, one row per fix that closed a HIGH finding ---

mutation "a gapped trunk column cannot be meshed" gui a_gapped_trunk_column_is_rejected_and_falls_back_to_cubes <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    if top_z - base_z + 1 != cells {\n        return None;\n    }\n'
assert s.count(old) == 1
# Drop the contiguity check and min/max alone decide again, which is what let a dwarf dig a hole
# through a trunk while the client redrew an unbroken pine over it.
p.write_text(s.replace(old, ''))
PY

mutation "a tree no mesh draws falls back to cubes" gui a_gapped_trunk_column_is_rejected_and_falls_back_to_cubes <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    is_tree(mirror, position) && cover.covers(position)\n'
assert s.count(old) == 1
# Exclude EVERY tree cell from the terrain again, cover or no cover. A column the mesh rule
# rejects is then drawn by the mesh path and the cube path neither, and vanishes silently.
p.write_text(s.replace(old, '    is_tree(mirror, position)\n'))
PY

mutation "the lantern sweep reads both draw paths" gui the_lantern_sweep_sees_terrain_that_exists_only_as_chunk_meshes <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = """            .chain(
                chunk_cells
                    .iter()
                    .flat_map(|cells| cells.0.iter().copied())
                    .map(|cell| (cell, world_to_render(cell))),
            )
"""
assert s.count(old) == 1
# Back to TerrainTile only. At subdiv>1 there are none, so lit_tiles is empty and every
# subdiv-2 capture panics before writing its PNG -- 10.4 shipped exactly this.
p.write_text(s.replace(old, ''))
PY

mutation "a windowed capture carries its tree accounting" gui a_capture_carries_its_tree_accounting_whether_or_not_it_is_headless <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = '    if args.capture.is_some() {\n        app.insert_resource(TreeCaptureVerification::default());'
assert s.count(old) == 1
# Re-gate on --headless. The expected side still counts the trees, the actual side no longer
# gains them, and the vehicle sitting card's own command asserts 0 == 265.
p.write_text(s.replace(old, '    if args.headless && args.capture.is_some() {\n        app.insert_resource(TreeCaptureVerification::default());'))
PY

mutation "the startup asset line counts bytes, not entries" gui the_startup_asset_line_reads_the_blobs_rather_than_the_array_length <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/ingest.rs'); s = p.read_text()
old = 'include_bytes!("../../../assets/trees/SM_VoxelPine_Tree01.glb"),'
assert s.count(old) == 1
# Empty a DIFFERENT pine than the row above, so this row pins the STARTUP LINE's count rather
# than the asset test's. Reading TREE_ASSETS.len() reports 4 of 4 with this blob gone.
p.write_text(s.replace(old, 'b"",'))
PY

mutation "the independent oracle can fail" gui the_independent_oracle_fails_a_tree_that_neither_path_draws <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/capture.rs'); s = p.read_text()
old = '        .filter(|cell| !cover.covers(*cell) && !drawn.contains(cell))\n'
assert s.count(old) == 1
# Make the undrawn set unreachable. The oracle then reports success on a valley whose trees are
# drawn by nothing at all -- which is the state it was written to detect.
p.write_text(s.replace(old, '        .filter(|_| false)\n'))
PY
