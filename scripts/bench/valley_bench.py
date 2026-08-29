"""Cycles bench for a protocol-exported frostvein valley.

Run with Blender: ``blender --background --python valley_bench.py -- SNAPSHOT OUT``.
"""

import json
import math
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

TERRAIN_RGB = {
    "stone": (60, 70, 92),
    "soil": (56, 52, 62),
    "ice": (104, 128, 170),
    "snow": (136, 150, 178),
    "tree_trunk": (43, 47, 58),
    "tree_foliage": (44, 100, 58),
}
SNOW_CAP_RGB = (146, 158, 184)
FOLIAGE_SNOW_RGB = (156, 170, 196)
SKY_RGB = (5, 12, 28)
AMBIENT_RGB = (120, 140, 165)
DIRECTIONAL_RGB = (150, 190, 180)
LIGHT_RGB = {
    "torch": (255, 140, 62),
    "campfire": (255, 173, 92),
    "lantern": (255, 195, 110),
}
ENTITY_APPEARANCE = {
    "dwarf": ((151, 116, 96), 0.65),
    "torch": ((255, 140, 62), 0.28),
    "campfire": ((255, 173, 92), 0.55),
}


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


def has_snow_cap(snapshot, x, y, z):
    material = terrain_material(tile_at(snapshot, x, y, z))
    return (
        material not in (None, "ice", "soil", "tree_foliage")
        and not is_solid(tile_at(snapshot, x, y, z + 1))
    )


def has_snow_laden_crown(snapshot, x, y, z):
    if terrain_material(tile_at(snapshot, x, y, z)) != "tree_foliage":
        return False
    if is_solid(tile_at(snapshot, x, y, z + 1)):
        return False
    return terrain_material(tile_at(snapshot, x, y, z - 1)) not in (
        "stone", "soil", "ice", "snow"
    )


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
        # world_to_render changes handedness, so face winding follows it to preserve normals.
        faces.append((first, first + 3, first + 2, first + 1))
        material_name = material
        if has_snow_laden_crown(snapshot, x, y, z):
            material_name = "foliage_snow"
        elif face_index == 5 and has_snow_cap(snapshot, x, y, z):
            material_name = "snow_cap"
        face_materials.append(material_indexes[material_name])
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


def srgb_to_linear(rgb):
    def channel(value):
        value /= 255.0
        return value / 12.92 if value <= 0.04045 else ((value + 0.055) / 1.055) ** 2.4

    return tuple(channel(value) for value in rgb)


def make_material(name, rgb):
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    material.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (
        *srgb_to_linear(rgb),
        1.0,
    )
    material.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 1.0
    return material


def add_cube(name, location, scale, material):
    bpy.ops.mesh.primitive_cube_add(size=1, location=location)
    cube = bpy.context.object
    cube.name = name
    cube.scale = (scale, scale, scale)
    cube.data.materials.append(material)
    return cube


def setup_scene(snapshot):
    from mathutils import Vector

    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 32
    # RuntimeError: Error: Failed to denoise, build has no OpenImageDenoise support.
    scene.cycles.use_denoising = False
    scene.render.resolution_x = 960
    scene.render.resolution_y = 540
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.look = "None"
    scene.view_settings.view_transform = "Standard"
    scene.world.color = srgb_to_linear(SKY_RGB)
    scene.world.use_nodes = True
    scene.world.node_tree.nodes["Background"].inputs["Color"].default_value = (
        *srgb_to_linear(SKY_RGB),
        1.0,
    )
    scene.world.node_tree.nodes["Background"].inputs["Strength"].default_value = 1.5

    materials = {name: make_material(name, rgb) for name, rgb in TERRAIN_RGB.items()}
    materials["snow_cap"] = make_material("snow_cap", SNOW_CAP_RGB)
    materials["foliage_snow"] = make_material("foliage_snow", FOLIAGE_SNOW_RGB)
    create_terrain(snapshot, materials)

    for entity in snapshot.get("entities", []):
        kind = entity["kind"]
        rgb, scale = ENTITY_APPEARANCE[kind]
        position = Vector(world_to_render(entity["pos"]))
        entity_material = make_material("entity_" + str(entity["id"]), rgb)
        add_cube(kind, position, scale, entity_material)
        light_kind = entity.get("light")
        if light_kind:
            light_data = bpy.data.lights.new(light_kind, "POINT")
            light_data.color = srgb_to_linear(LIGHT_RGB[light_kind])
            light_data.energy = {"torch": 750.0, "campfire": 1_500.0, "lantern": 300.0}[light_kind]
            light_data.shadow_soft_size = 1.0
            light = bpy.data.objects.new(light_kind, light_data)
            light.location = position + Vector((0.0, 0.5, 0.0))
            bpy.context.collection.objects.link(light)

    sun_data = bpy.data.lights.new("directional", "SUN")
    sun_data.color = srgb_to_linear(DIRECTIONAL_RGB)
    sun_data.energy = 3.0
    sun = bpy.data.objects.new("directional", sun_data)
    sun.rotation_euler = (math.radians(-35), math.radians(20), math.radians(30))
    bpy.context.collection.objects.link(sun)

    camera_data = bpy.data.cameras.new("boot camera")
    camera_data.sensor_fit = "VERTICAL"
    camera_data.angle = math.pi / 4
    camera = bpy.data.objects.new("boot camera", camera_data)
    bpy.context.collection.objects.link(camera)
    yaw, pitch, distance = 0.7, 0.45, 90.0
    forward = Vector((-math.cos(yaw), 0.0, -math.sin(yaw)))
    target = Vector(world_to_render((64, 64, 9))) + forward * 33.0 + Vector((0.0, -0.5, 0.0))
    horizontal = distance * math.cos(pitch)
    camera.location = target + Vector(
        (horizontal * math.cos(yaw), distance * math.sin(pitch), horizontal * math.sin(yaw))
    )
    camera.rotation_euler = (target - camera.location).to_track_quat("-Z", "Y").to_euler()
    scene.camera = camera


# NOTE: The client also draws solid cells at its selected top slice. The bench intentionally
# renders only the exposed set, so this small boot-draw divergence remains visible and named.


def main():
    if bpy is None:
        raise SystemExit("run this script with blender")
    args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    if len(args) != 2:
        raise SystemExit("usage: valley_bench.py -- <snapshot.json> <out.png>")
    with open(args[0], encoding="utf-8") as snapshot_file:
        snapshot = json.load(snapshot_file)
    summary = geometry_summary(snapshot)
    print("exposed cells:", summary["exposed_cells"], "faces:", summary["faces"])
    setup_scene(snapshot)
    bpy.context.scene.render.filepath = args[1]
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    main()
