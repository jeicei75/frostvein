"""Cycles bench for a protocol-exported frostvein valley.

Run with Blender: ``blender --background --python valley_bench.py -- SNAPSHOT OUT``.
"""

import json
import sys

try:
    import bpy
except ImportError:
    bpy = None


NEIGHBOURS = (
    (-1, 0, 0),
    (1, 0, 0),
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
)

FACE_CORNERS = (
    ((0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 0)),
    ((1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)),
    ((0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)),
    ((0, 1, 0), (0, 1, 1), (1, 1, 1), (1, 1, 0)),
    ((0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 0, 0)),
    ((0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)),
)


def dims_of(snapshot):
    dims = snapshot["dims"]
    return dims["x"], dims["y"], dims["z"]


def index_of(x, y, z, dims):
    dx, dy, _ = dims
    return x + y * dx + z * dx * dy


def tile_at(snapshot, x, y, z):
    dims = dims_of(snapshot)
    dx, dy, dz = dims
    if not (0 <= x < dx and 0 <= y < dy and 0 <= z < dz):
        return "empty"
    return snapshot["tiles"][index_of(x, y, z, dims)]


def terrain_material(tile):
    if isinstance(tile, dict):
        return tile.get("solid", tile.get("ramp"))
    return None


def is_solid(tile):
    return terrain_material(tile) is not None


def exposed_faces(snapshot):
    """Yield (position, material, face-direction) for every externally visible face."""
    dims = dims_of(snapshot)
    dx, dy, dz = dims
    for z in range(dz):
        for y in range(dy):
            for x in range(dx):
                tile = tile_at(snapshot, x, y, z)
                material = terrain_material(tile)
                if material is None:
                    continue
                for face_index, (nx, ny, nz) in enumerate(NEIGHBOURS):
                    if not is_solid(tile_at(snapshot, x + nx, y + ny, z + nz)):
                        yield (x, y, z), material, face_index


def geometry_summary(snapshot):
    faces = list(exposed_faces(snapshot))
    return {"exposed_cells": len({position for position, _, _ in faces}), "faces": len(faces)}


def world_to_render(point):
    x, y, z = point
    return x, z, -y


def mesh_geometry(snapshot, material_indexes):
    vertices = []
    faces = []
    face_materials = []
    for (x, y, z), material, face_index in exposed_faces(snapshot):
        first = len(vertices)
        for corner in FACE_CORNERS[face_index]:
            cx, cy, cz = corner
            vertices.append(world_to_render((x + cx - 0.5, y + cy - 0.5, z + cz - 0.5)))
        faces.append((first, first + 1, first + 2, first + 3))
        face_materials.append(material_indexes[material])
    return vertices, faces, face_materials


def create_terrain(snapshot, materials):
    if bpy is None:
        raise RuntimeError("valley_bench.py must run under Blender")
    material_indexes = {name: index for index, name in enumerate(materials)}
    vertices, faces, face_materials = mesh_geometry(snapshot, material_indexes)
    mesh = bpy.data.meshes.new("valley terrain")
    mesh.from_pydata(vertices, [], faces)
    mesh.polygons.foreach_set("material_index", face_materials)
    mesh.update()
    terrain = bpy.data.objects.new("valley terrain", mesh)
    bpy.context.collection.objects.link(terrain)
    for material in materials.values():
        mesh.materials.append(material)
    return terrain


# NOTE: The client also draws solid cells at its selected top slice. The bench intentionally
# renders only the exposed set, so this small boot-draw divergence remains visible and named.


def main():
    if bpy is None:
        raise SystemExit("run this script with blender")
    args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    if len(args) != 2:
        raise SystemExit("usage: valley_bench.py -- <snapshot.json> <out.png>")
    snapshot = json.loads(open(args[0], encoding="utf-8").read())
    summary = geometry_summary(snapshot)
    print("exposed cells:", summary["exposed_cells"], "faces:", summary["faces"])
    raise SystemExit("look setup follows in Task 3")


if __name__ == "__main__":
    main()
