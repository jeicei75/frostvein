"""Render the valley with 10.2's authored voxel pines in place of the cube trees.

SCRATCH EVIDENCE SCRIPT for story 10.4 Task 2 — not part of the shipped bench, not read by
`bench_contract.rs`. It exists to answer one question the isolated grey-background asset
renders cannot: *does an authored pine read as a tree IN SITU*, at boot framing, in our
valley, against the same terrain and lights as the control.

    blender --background --python authored_bench.py -- <snapshot.json> <out.png>

METHOD
    1. Every `tree_trunk` / `tree_foliage` cell is stripped from the snapshot, exactly as the
       story's RED recipe does. The terrain, camera, lights and materials then come from
       `valley_bench.setup_scene` UNCHANGED — imported, not copied, so this cannot drift from
       the control the way a forked file would.
    2. One authored pine is instanced per trunk column, sharing mesh data between instances.

SCALE — the one number that decides whether this is a fair test
    `docs/tech-art-guidelines.md` § Resolution contract fixes 1.6 m per simulation cell. The
    bench renders in CELL units (`world_to_render` maps sim coords straight to Blender units),
    while the GLBs are in METRES. So every pine is scaled by 1/1.6 = 0.625 and NOT stretched to
    fit: stretching would silently re-cut the 0.2 m voxel the asset was authored on, and the
    whole point of this comparison is to judge the asset at its contracted size.

VARIANT SELECTION is deterministic from the trunk column, never from `HashMap` order or the
clock. Variants are chosen by matching AUTHORED HEIGHT to the sim tree's height in cells:
    sim height 4 -> Tree01  (6.4 m = 4.00 cells)  exact
    sim height 5 -> Tree02 / Tree03 (8.0 m = 5.00 cells)  exact, hash picks between them
    sim height 6 -> Tree04R (9.6 m = 6.00 cells)  exact
Every placement is now height-exact. The shipped `SM_VoxelPine_Tree04` is 10.6 m = 6.625
cells and overshot on 103 of 265 placements; `pine_6cell.py` regenerates it at 6 cells and
records the reference-sheet contradiction that produced the wrong height. The overshoot
counter below stays in place so a regression cannot pass silently -- it must print 0.
"""

import json
import math
import os
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, "/workspace/projects/frostvein/scripts/bench")
import valley_bench as vb  # noqa: E402

SIGNOFF = "/workspace/projects/frostvein/_bmad-output/implementation-artifacts/10-4-signoff"
EXPORT = "/workspace/projects/frostvein/_bmad-output/implementation-artifacts/10-2-signoff/export"
METRES_PER_CELL = 1.6

# Authored height in metres, parsed from check_asset.py FIGURES at the time of writing and
# RE-ASSERTED below against the imported mesh, so a swapped asset cannot pass silently.
#
# Tree04R replaces the shipped Tree04 (10.6 m = 6.625 cells, which fits no tree `place_trees`
# can generate and overshot on 103 of 265 placements). It is Tree04 scaled uniformly to
# exactly 6 cells; see pine_6cell.py for the reference-sheet contradiction behind it.
VARIANTS = {
    "SM_VoxelPine_Tree01": 6.4,
    "SM_VoxelPine_Tree02": 8.0,
    "SM_VoxelPine_Tree03": 8.0,
    "SM_VoxelPine_Tree04R": 9.6,
}

# Tree04R lives beside this script; the other three are 10.2's shipped deliverables.
VARIANT_DIR = {name: (SIGNOFF if name.endswith("R") else EXPORT) for name in VARIANTS}


def stable_hash(*parts):
    """Deterministic across runs and processes -- Python's hash() is salted, so it is unusable."""
    value = 2166136261
    for part in parts:
        for byte in str(part).encode():
            value = ((value ^ byte) * 16777619) & 0xFFFFFFFF
    return value


def trunk_columns(snapshot):
    """Yield (x, y, base_z, height_cells) for every tree, derived from the trunk cells.

    worldgen writes trunk over `surface+1 .. crown_top-1` and the tip at `crown_top`, so a
    column's height in cells is `max_trunk_z - min_trunk_z + 2`.
    """
    dims = vb.dims_of(snapshot)
    dx, dy, dz = dims
    for y in range(dy):
        for x in range(dx):
            zs = [z for z in range(dz)
                  if vb.terrain_material(vb.tile_at(snapshot, x, y, z)) == "tree_trunk"]
            if not zs:
                continue
            yield x, y, min(zs), max(zs) - min(zs) + 2


def strip_trees(snapshot):
    stripped = dict(snapshot)
    stripped["tiles"] = [
        {} if isinstance(tile, dict) and tile.get("solid") in ("tree_foliage", "tree_trunk")
        else tile
        for tile in snapshot["tiles"]
    ]
    return stripped


def import_variant(name):
    """Import one GLB, re-orient it into the bench's Y-up render space, and verify its height."""
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=os.path.join(VARIANT_DIR[name], f"{name}.glb"))
    imported = [o for o in set(bpy.data.objects) - before if o.type == "MESH"]
    if len(imported) != 1:
        raise SystemExit(f"{name}: expected exactly one mesh, got {len(imported)}")
    obj = imported[0]

    # The glTF importer converts the asset's Y-up into Blender's Z-up. The bench's render space
    # is Y-up INSIDE a Z-up Blender, so that conversion has to be undone rather than trusted.
    # `rotation_mode` MUST be set first: the importer leaves it on QUATERNION, and assigning
    # `rotation_euler` while it is QUATERNION is silently ignored -- the tree renders unrotated
    # and nothing errors.
    obj.rotation_mode = "XYZ"
    obj.rotation_euler = (-math.pi / 2.0, 0.0, 0.0)
    bpy.context.view_layer.update()

    # Verify rather than assume: after re-orientation the authored height must lie on Y.
    # `obj.dimensions` is the LOCAL bounding box and ignores rotation entirely -- reading it here
    # measures the asset as imported, not as placed, and would pass a mis-oriented tree. The world
    # bounding box is the only thing that answers "which way is this pine actually pointing".
    height_m = VARIANTS[name]
    corners = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
    extent = {
        "x": max(c.x for c in corners) - min(c.x for c in corners),
        "y": max(c.y for c in corners) - min(c.y for c in corners),
        "z": max(c.z for c in corners) - min(c.z for c in corners),
    }
    if abs(extent["y"] - height_m) > 0.05:
        raise SystemExit(
            f"{name}: height should sit on Y at {height_m} m after re-orientation, "
            f"measured x={extent['x']:.2f} y={extent['y']:.2f} z={extent['z']:.2f}"
        )
    # The base must sit at Y=0 so `location` places the trunk foot, not the pine's centre.
    base_y = min(c.y for c in corners)
    if abs(base_y) > 0.02:
        raise SystemExit(f"{name}: base should sit at Y=0, measured {base_y:.3f}")

    tris = sum(len(polygon.vertices) - 2 for polygon in obj.data.polygons)
    print(f"  imported {name}: {height_m} m ({height_m / METRES_PER_CELL:.3f} cells), {tris} tris")
    return obj


def place_pines(snapshot):
    sources = {name: import_variant(name) for name in VARIANTS}
    for obj in sources.values():
        obj.hide_render = True

    by_height = {
        4: ["SM_VoxelPine_Tree01"],
        5: ["SM_VoxelPine_Tree02", "SM_VoxelPine_Tree03"],
        6: ["SM_VoxelPine_Tree04R"],
    }
    scale = 1.0 / METRES_PER_CELL
    placed = 0
    heights = {}
    overshoot = 0
    for x, y, base_z, height in sorted(trunk_columns(snapshot)):
        heights[height] = heights.get(height, 0) + 1
        choices = by_height.get(height) or by_height[min(by_height, key=lambda h: abs(h - height))]
        name = choices[stable_hash(x, y, "variant") % len(choices)]
        if VARIANTS[name] / METRES_PER_CELL > height + 1e-6:
            overshoot += 1

        source = sources[name]
        inst = bpy.data.objects.new(f"pine_{x}_{y}", source.data)
        bpy.context.collection.objects.link(inst)
        # A cell centred on integer (x, y, z) spans +-0.5, so the ground under the first trunk
        # cell is at sim z = base_z - 0.5. world_to_render maps (x, y, z) -> (x, z, -y).
        inst.location = Vector((float(x), float(base_z) - 0.5, -float(y)))
        inst.scale = (scale, scale, scale)
        inst.rotation_mode = "XYZ"
        # Undo the importer's Z-up conversion, then spin about the (now vertical) Y axis in
        # quarter turns so 265 copies of four meshes do not all face the camera identically.
        inst.rotation_euler = (
            -math.pi / 2.0,
            (stable_hash(x, y, "yaw") % 4) * (math.pi / 2.0),
            0.0,
        )
        placed += 1

    print(f"  placed {placed} pines; sim heights {dict(sorted(heights.items()))}")
    print(f"  height-overshooting placements (authored taller than the sim tree): {overshoot}")
    return placed


def main():
    args = sys.argv[sys.argv.index("--") + 1:]
    if len(args) != 2:
        raise SystemExit("usage: authored_bench.py -- <snapshot.json> <out.png>")

    with open(args[0], encoding="utf-8") as handle:
        snapshot = json.load(handle)

    stripped = strip_trees(snapshot)
    summary = vb.geometry_summary(stripped)
    print("terrain-only exposed cells:", summary["exposed_cells"], "faces:", summary["faces"])
    print("  (cube trees are GONE -- exposed_cells describes terrain only and is expected to")
    print("   read the treeless 40,148, not the control's 44,984. Judge the pixel figures.)")

    vb.setup_scene(stripped)
    place_pines(snapshot)

    bpy.context.scene.render.filepath = args[1]
    bpy.ops.render.render(write_still=True)
    image = bpy.data.images.load(args[1], check_existing=False)
    figures = vb.pixel_figures(image.pixels[:])
    bpy.data.images.remove(image)

    check = vb.range_check(summary, figures)
    print(
        "range-check:"
        f" blender={'.'.join(str(part) for part in bpy.app.version)}"
        f" exposed_cells={check['exposed_cells']}"
        f" non_sky_fraction={check['non_sky_fraction']:.6f}"
        f" distinct_colors={check['distinct_colors']}"
        f" terrain_luma={check['terrain_luma']:.3f}"
    )


if __name__ == "__main__":
    main()
