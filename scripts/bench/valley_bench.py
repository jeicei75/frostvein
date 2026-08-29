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

# Floors, each well below its delivered measurement so reframing does not trip them.
# MEASURED on the delivered bench, shipped seed, 960x540: non_sky_fraction 0.674020 and
# distinct_colors 45,642. An all-sky frame reads 0.000000 / 4, which is what these floors exist to
# reject. What moves them: reframing and world content move the fraction; material and light
# tuning move the quantised colour count. Story 10.4 will move both, so they are floors and not
# pins.
MIN_NON_SKY_FRACTION = 0.02
MIN_DISTINCT_COLORS = 32


def vector_add(left, right):
    return tuple(a + b for a, b in zip(left, right))


def vector_subtract(left, right):
    return tuple(a - b for a, b in zip(left, right))


def vector_scale(vector, scalar):
    return tuple(component * scalar for component in vector)


def vector_dot(left, right):
    return sum(a * b for a, b in zip(left, right))


def vector_cross(left, right):
    return (
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    )


def vector_normalize(vector):
    length = math.sqrt(vector_dot(vector, vector))
    if length == 0.0:
        raise ValueError("cannot normalize a zero-length vector")
    return vector_scale(vector, 1.0 / length)


def boot_camera_frame():
    """Return the boot camera's location and local axes without requiring Blender."""
    yaw, pitch, distance = 0.7, 0.45, 90.0
    forward = (-math.cos(yaw), 0.0, -math.sin(yaw))
    target = vector_add(
        vector_add(world_to_render((64.0, 64.0, 9.0)), vector_scale(forward, 33.0)),
        (0.0, -0.5, 0.0),
    )
    horizontal = distance * math.cos(pitch)
    location = vector_add(
        target,
        (horizontal * math.cos(yaw), distance * math.sin(pitch), horizontal * math.sin(yaw)),
    )
    back = vector_normalize(vector_subtract(location, target))
    # Match Bevy's `looking_at(target, Vec3::Y)`: Frostvein render space is Y-up.
    right = vector_normalize(vector_cross((0.0, 1.0, 0.0), back))
    up = vector_cross(back, right)
    return location, right, up, back


def project_boot_point(point):
    """Project a render-space point to normalized (left, top) boot-frame coordinates."""
    location, right, up, back = boot_camera_frame()
    offset = vector_subtract(point, location)
    depth = -vector_dot(offset, back)
    if depth <= 0.0:
        return None
    half_vertical = math.tan((math.pi / 4) * 0.5)
    return (
        0.5 + vector_dot(offset, right) / (2.0 * depth * half_vertical * (16.0 / 9.0)),
        0.5 - vector_dot(offset, up) / (2.0 * depth * half_vertical),
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


def pixel_figures(pixels):
    # MEASURED, not assumed: a rendered frame read back through `bpy.data.images.load(..).pixels`
    # under the "Standard" view transform returns DISPLAY-referred sRGB, so an all-sky frame reads
    # (0.01961, 0.04706, 0.1098) == SKY_RGB/255. Comparing against srgb_to_linear(SKY_RGB) instead
    # put every sky pixel 0.098 away in blue against a 0.02 tolerance, so a 100%-sky frame scored
    # non_sky_fraction=1.0 and the floor below could never fire. Materials still take the linear
    # conversion; only this readback is display-referred.
    sky = tuple(component / 255.0 for component in SKY_RGB)
    non_sky = 0
    colors = set()
    total = 0
    for red, green, blue, _ in zip(*[iter(pixels)] * 4):
        total += 1
        if max(abs(red - sky[0]), abs(green - sky[1]), abs(blue - sky[2])) > 0.02:
            non_sky += 1
        colors.add((round(red * 255), round(green * 255), round(blue * 255)))
    return {"non_sky_fraction": non_sky / total if total else 0.0, "distinct_colors": len(colors)}


def range_check(summary, figures):
    return {
        "exposed_cells": summary["exposed_cells"],
        "non_sky_fraction": figures["non_sky_fraction"],
        "distinct_colors": figures["distinct_colors"],
        "minimum_non_sky_fraction": MIN_NON_SKY_FRACTION,
        "minimum_distinct_colors": MIN_DISTINCT_COLORS,
    }


def assert_range(check):
    assert check["exposed_cells"] > 0, "no exposed cells"
    assert check["non_sky_fraction"] >= check["minimum_non_sky_fraction"], "frame is too close to sky"
    assert check["distinct_colors"] >= check["minimum_distinct_colors"], "frame has too few colours"


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
    from mathutils import Matrix, Vector

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
    scene.world.node_tree.nodes["Background"].inputs["Strength"].default_value = 1.0

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
    # The rendered camera IS the projected camera: boot_camera_frame() is what the framing test
    # checks, so a change here cannot leave that test green against a frame it no longer matches.
    # Blender's track quaternion would level against its Z-up scene, but Frostvein's render space
    # is Y-up, so the basis is built explicitly, exactly as Bevy's look_at does.
    location, right, up, back = boot_camera_frame()
    camera.location = Vector(location)
    camera.rotation_euler = Matrix((Vector(right), Vector(up), Vector(back))).transposed().to_euler()
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
    image = bpy.data.images.load(args[1], check_existing=False)
    figures = pixel_figures(image.pixels[:])
    bpy.data.images.remove(image)
    check = range_check(summary, figures)
    print(
        "range-check:"
        f" exposed_cells={check['exposed_cells']}"
        f" non_sky_fraction={check['non_sky_fraction']:.6f}"
        f" distinct_colors={check['distinct_colors']}"
        f" floors(non_sky_fraction={check['minimum_non_sky_fraction']:.6f},"
        f" distinct_colors={check['minimum_distinct_colors']})"
    )
    try:
        assert_range(check)
    except AssertionError as error:
        # Blender logs an uncaught Python AssertionError yet exits 0; propagate failure to shell.
        raise SystemExit(f"range check failed: {error}") from error


if __name__ == "__main__":
    main()
