"""Generate a grid-aligned 6-cell pine (Tree04-R) for story 10.4's Task 2 evidence.

    blender --background --python pine_6cell.py -- <out.glb> [--seed N]

WHY THIS EXISTS
    `place_trees` generates trees 4-6 cells tall and the resolution contract fixes 1.6 m per
    cell, so a 6-cell tree is exactly 9.6 m. The shipped `SM_VoxelPine_Tree04` is 10.6 m =
    6.625 cells, which fits NO tree the sim can generate. In candidate D that mismatch hit
    103 of 265 trees (39%), every one of them 1.0 m too tall.

THE UPSTREAM DEFECT, checked against the approved reference sheet rather than assumed
    `reference-sheet.jpg` Section B labels each variant TWICE, and for Type 4 the two labels
    contradict each other:

        Type 1   "4 CELLS"   "5.32x dwarf height"  -> 5.32 * 1.20 = 6.38 m = 3.99 cells  agree
        Type 2   "5 CELLS"   "6.67x dwarf height"  -> 6.67 * 1.20 = 8.00 m = 5.00 cells  agree
        Type 3   "5 CELLS"   "6.67x dwarf height"  -> 8.00 m = 5.00 cells               agree
        Type 4   "6 CELLS"   "8.80x dwarf height"  -> 8.80 * 1.20 = 10.56 m = 6.60 cells  CLASH

    Only Type 4 is internally inconsistent. The CELL label is the half the simulation can
    honour -- `place_trees` has a hard ceiling of 6 -- so the cell count wins and the dwarf
    multiple is the stale half. In voxels at the sheet's 0.2 m: 6 cells * 1.6 m / 0.2 m = 48,
    which also restores the 8-voxels-per-cell alignment the other three variants already have
    (32 = 4x8, 40 = 5x8, 48 = 6x8; the shipped 53 is 6.625x8).

METHOD
    The design is scaled UNIFORMLY by 48/53. Rescaling height alone would squat the silhouette
    into a different tree, and this comparison is meant to test the ASSET, not a new design.
    `voxel_pine.py` is imported, never edited -- it is story 10.2's signed-off deliverable.
    Radii stay floats (the generator compares them with `<=`); z positions and `flare` must be
    ints because the generator feeds them to `range()`.
"""

import os
import sys

sys.path.insert(
    0, "/workspace/projects/frostvein/_bmad-output/implementation-artifacts/10-2-signoff"
)
import voxel_pine as vp  # noqa: E402

TARGET_CELLS = 6
METRES_PER_CELL = 1.6
VOXEL_M = vp.DEFAULT_VOXEL                       # 0.2 m, unchanged from the sheet
TARGET_HEIGHT_VOXELS = round(TARGET_CELLS * METRES_PER_CELL / VOXEL_M)   # 48


def rescaled_tree04():
    source = vp.TREE_TYPES[4]
    factor = TARGET_HEIGHT_VOXELS / source["height"]
    spec = dict(
        source,
        label="Tree04R",
        cells=TARGET_CELLS,
        height=TARGET_HEIGHT_VOXELS,
        dwarf_mult=TARGET_CELLS * METRES_PER_CELL / vp.DWARF_HEIGHT_M,
        trunk_r=tuple(r * factor for r in source["trunk_r"]),
        flare=tuple(max(1, round(r * factor)) for r in source["flare"]),
        spire=(round(source["spire"][0] * factor), source["spire"][1] * factor),
        tiers=[(round(z * factor), r * factor) for z, r in source["tiers"]],
    )
    return spec, factor


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if not args or not args[0].lower().endswith(".glb"):
        raise SystemExit("usage: pine_6cell.py -- <out.glb> [--seed N]")
    out = os.path.abspath(args[0])
    seed = vp.DEFAULT_SEED
    rest = list(args[1:])
    while rest:
        flag = rest.pop(0)
        if flag == "--seed":
            seed = int(rest.pop(0))
        else:
            raise SystemExit(f"unknown option {flag!r}")

    spec, factor = rescaled_tree04()
    print(f"Tree04 rescale factor {factor:.5f}  height {vp.TREE_TYPES[4]['height']} -> "
          f"{spec['height']} voxels")
    print(f"  tiers {[(z, round(r, 2)) for z, r in spec['tiers']]}")
    print(f"  spire {(spec['spire'][0], round(spec['spire'][1], 2))}  "
          f"flare {spec['flare']}  trunk_r {tuple(round(r, 2) for r in spec['trunk_r'])}")

    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
    vp.wipe_scene()
    vox = vp.build_voxels(spec, seed)
    verts, faces, uvs = vp.greedy_mesh(vox, VOXEL_M)
    obj = vp.build_object(spec, verts, faces, uvs)
    vp.export_glb(obj, out)

    # Read the height back out of the GLB that was actually written, never off the spec that
    # was asked for -- the asset contract's rule, and the only check that can catch a generator
    # that silently ignored a field.
    gj, binary, _ = vp.load_glb(out)
    accessor = gj["accessors"][gj["meshes"][0]["primitives"][0]["attributes"]["POSITION"]]
    lo, hi = accessor["min"], accessor["max"]
    height_m = hi[1] - lo[1]
    cells = height_m / METRES_PER_CELL
    print(f"FIGURES {os.path.basename(out)} height_m={height_m:.4f} cells={cells:.4f} "
          f"size_m={hi[0]-lo[0]:.2f}x{height_m:.2f}x{hi[2]-lo[2]:.2f} "
          f"tris={len(vp.read_accessor(gj, binary, gj['meshes'][0]['primitives'][0]['indices']))//3}")
    if abs(height_m - TARGET_CELLS * METRES_PER_CELL) > 0.01:
        raise SystemExit(
            f"height {height_m:.4f} m is not the {TARGET_CELLS * METRES_PER_CELL} m "
            f"a {TARGET_CELLS}-cell tree must be"
        )
    print(f"OK {spec['label']} -> {out}")


if __name__ == "__main__":
    main()
