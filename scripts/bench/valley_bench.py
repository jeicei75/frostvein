"""Cycles bench for a protocol-exported frostvein valley.

Run with Blender: ``blender --background --python valley_bench.py -- SNAPSHOT OUT``.
"""

import json
import math
import traceback
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

# The boot camera, SINGLE-SOURCED. `project_boot_point` and the rendered camera must read the
# same constants or the framing test guards a projection the renderer does not use: measured on
# the previous split version, widening only the render FOV to pi/3 changed 1,050,234 of 2,073,600
# pixel values while the framing test still read camp=(0.500, 0.779) skyline_y=0.240, inside
# tolerance, and the range check still exited 0.
BOOT_YAW, BOOT_PITCH, BOOT_DISTANCE = 0.7, 0.45, 90.0
BOOT_COMPOSITION_FORWARD = 33.0
BOOT_COMPOSITION_LIFT = -0.5
BOOT_VERTICAL_FOV = math.pi / 4
BOOT_ASPECT_RATIO = 16.0 / 9.0
BOOT_FOCUS = (64.0, 64.0, 9.0)
RENDER_HEIGHT = 540
RENDER_WIDTH = round(RENDER_HEIGHT * BOOT_ASPECT_RATIO)

# The aurora itself is out of scope and is NOT drawn. Its geometry remains here for the bench's
# own bright-point calculations, but it no longer steers the directional light.
AURORA_RADIUS = 600.0
AURORA_BOTTOM = -162.0
AURORA_TOP = 45.0
SKY_CENTRE = (63.5, 0.0, -63.5)
CAMP_FOCUS = (64.0, 9.0, -64.0)

# [atmosphere.rs] The client and bench must share the sunlight's travel bearing and elevation.
SUN_AZIMUTH_DEGREES = 40.0398
SUN_ELEVATION_DEGREES = -6.4181

# Cycles has no ambient-light object: the world background IS the ambient term. The client adds
# `AmbientLight { color: night_lighting().ambient, brightness: 4_500.0 }` [ingest.rs:714-718] on
# top of its flat sky, and drives its sun at 22,000 lux [appearance.rs:48].
#
# NEITHER number converts: Bevy's brightness/illuminance and Cycles' background strength and sun
# energy share no units, and the bench deliberately omits the aurora curtain, which is a real
# light source in the client. So these two scalars are CALIBRATED, not converted, against one
# objective target: mean Rec.709 luma over the bottom 65% of the frame (terrain-dominated at the
# boot framing, and free of the aurora that contaminates a whole-frame average).
#   client `gui-capture.png`  105.7   mean RGB (87, 108, 138)
#   bench at these values     103.6   mean RGB (81, 106, 147)
# Re-calibrate against a fresh client capture if the client's lighting moves; do not tune these
# by eye. Before ambient was wired at all the same band read 65.0 and the frame was visibly flat.
AMBIENT_STRENGTH = 3.3
SUN_ENERGY = 21.0

# Floors, each well below its delivered measurement so reframing does not trip them.
# MEASURED on the delivered bench, shipped seed, 960x540: non_sky_fraction 0.674020 and
# distinct_colors 45,642. An all-sky frame reads 0.000000 / 4, which is what these floors exist to
# reject. What moves them: reframing and world content move the fraction; material and light
# tuning move the quantised colour count. Story 10.4 will move both, so they are floors and not
# pins.
MIN_NON_SKY_FRACTION = 0.02
MIN_DISTINCT_COLORS = 32
# The figure that would have caught an unlit scene. AMBIENT_RGB was defined, pinned by the drift
# guard, and read by nothing -- the frame rendered ~24% darker than the client and every existing
# floor stayed green, because a dark frame is neither empty nor monochrome.
MIN_TERRAIN_LUMA = 20.0


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


def boot_horizontal_forward():
    """[camera.rs:21-23] -- also the direction the aurora core sits along."""
    return (-math.cos(BOOT_YAW), 0.0, -math.sin(BOOT_YAW))


def aurora_core():
    """[atmosphere.rs:76-80]. The compass point the curtain is brightest at."""
    return vector_add(
        vector_add(SKY_CENTRE, vector_scale(boot_horizontal_forward(), AURORA_RADIUS)),
        (0.0, (AURORA_BOTTOM + AURORA_TOP) * 0.5, 0.0),
    )


def sun_direction():
    """[atmosphere.rs] The client's directional-light travel vector."""
    azimuth = math.radians(SUN_AZIMUTH_DEGREES)
    elevation = math.radians(SUN_ELEVATION_DEGREES)
    horizontal = math.cos(elevation)
    return (
        math.cos(azimuth) * horizontal,
        -math.sin(elevation),
        math.sin(azimuth) * horizontal,
    )


def boot_camera_frame():
    """Return the boot camera's location and local axes without requiring Blender."""
    yaw, pitch, distance = BOOT_YAW, BOOT_PITCH, BOOT_DISTANCE
    forward = boot_horizontal_forward()
    target = vector_add(
        vector_add(
            world_to_render(BOOT_FOCUS),
            vector_scale(forward, BOOT_COMPOSITION_FORWARD),
        ),
        (0.0, BOOT_COMPOSITION_LIFT, 0.0),
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
    half_vertical = math.tan(BOOT_VERTICAL_FOV * 0.5)
    return (
        0.5 + vector_dot(offset, right) / (2.0 * depth * half_vertical * BOOT_ASPECT_RATIO),
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


def foliage_scale(snapshot, x, y, z):
    """The client's cube-foliage shrink [project.rs:874-892]: 0.62 / 0.78 / 0.95 by crown depth.

    Counts consecutive TreeFoliage SOLID cells directly above, at most two, exactly as the
    client's `take_while` does -- a ramp does not count and the walk stops at the first gap.
    Scale changes only where a face is DRAWN, never which faces are exposed, so the cell and
    face counts are untouched by it.
    """
    if terrain_material(tile_at(snapshot, x, y, z)) != "tree_foliage":
        return 1.0
    above = 0
    for offset in (1, 2):
        tile = tile_at(snapshot, x, y, z + offset)
        if isinstance(tile, dict) and tile.get("solid") == "tree_foliage":
            above += 1
        else:
            break
    return (0.62, 0.78, 0.95)[above]


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
        # Foliage is drawn shrunk about its cell centre so a crown reads as sparse branches
        # rather than a solid canopy; a full-size cube tree is the 5.4 "the artifact does not
        # predict the build" failure in miniature, and 10.4 is judged on this picture.
        scale = foliage_scale(snapshot, x, y, z)
        for corner in FACE_CORNERS[face_index]:
            cx, cy, cz = corner
            vertices.append(
                world_to_render(
                    (
                        x + (cx - 0.5) * scale,
                        y + (cy - 0.5) * scale,
                        z + (cz - 0.5) * scale,
                    )
                )
            )
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
    luma_sum = 0.0
    for red, green, blue, _ in zip(*[iter(pixels)] * 4):
        total += 1
        if max(abs(red - sky[0]), abs(green - sky[1]), abs(blue - sky[2])) > 0.02:
            non_sky += 1
            # Rec.709 luma over the LIT pixels only. Averaging the whole frame would let a
            # bigger sky hide a darker scene, which is the confound this figure exists to avoid.
            luma_sum += 255.0 * (0.2126 * red + 0.7152 * green + 0.0722 * blue)
        colors.add((round(red * 255), round(green * 255), round(blue * 255)))
    return {
        "non_sky_fraction": non_sky / total if total else 0.0,
        "distinct_colors": len(colors),
        "terrain_luma": luma_sum / non_sky if non_sky else 0.0,
    }


def range_check(summary, figures):
    return {
        "exposed_cells": summary["exposed_cells"],
        "non_sky_fraction": figures["non_sky_fraction"],
        "distinct_colors": figures["distinct_colors"],
        "terrain_luma": figures["terrain_luma"],
        "minimum_non_sky_fraction": MIN_NON_SKY_FRACTION,
        "minimum_distinct_colors": MIN_DISTINCT_COLORS,
        "minimum_terrain_luma": MIN_TERRAIN_LUMA,
    }


def assert_range(check):
    assert check["exposed_cells"] > 0, "no exposed cells"
    assert check["non_sky_fraction"] >= check["minimum_non_sky_fraction"], "frame is too close to sky"
    assert check["distinct_colors"] >= check["minimum_distinct_colors"], "frame has too few colours"
    assert check["terrain_luma"] >= check["minimum_terrain_luma"], "lit surfaces are too dark"


def build_world(world):
    """Flat SKY_RGB to the camera, AMBIENT_RGB fill to every other ray.

    The client paints a flat `ClearColor` sky AND adds a separate `AmbientLight`
    [ingest.rs:198, 714-718]. Cycles has no ambient-light object, so the world background has to
    be both: mixing on `Is Camera Ray` keeps the visible backdrop exactly SKY_RGB while diffuse
    rays pick up AMBIENT_RGB. Before this the only fill was the near-black sky, AMBIENT_RGB was
    a dead constant, and the frame rendered ~24% darker than the build it must predict.
    """
    world.use_nodes = True
    tree = world.node_tree
    tree.nodes.clear()
    output = tree.nodes.new("ShaderNodeOutputWorld")
    mix = tree.nodes.new("ShaderNodeMixShader")
    light_path = tree.nodes.new("ShaderNodeLightPath")
    sky = tree.nodes.new("ShaderNodeBackground")
    ambient = tree.nodes.new("ShaderNodeBackground")
    sky.inputs["Color"].default_value = (*srgb_to_linear(SKY_RGB), 1.0)
    sky.inputs["Strength"].default_value = 1.0
    ambient.inputs["Color"].default_value = (*srgb_to_linear(AMBIENT_RGB), 1.0)
    ambient.inputs["Strength"].default_value = AMBIENT_STRENGTH
    # Fac 0 takes input 1, Fac 1 takes input 2, and Is Camera Ray is 1 only for the backdrop.
    tree.links.new(light_path.outputs["Is Camera Ray"], mix.inputs["Fac"])
    tree.links.new(ambient.outputs["Background"], mix.inputs[1])
    tree.links.new(sky.outputs["Background"], mix.inputs[2])
    tree.links.new(mix.outputs["Shader"], output.inputs["Surface"])


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
    scene.render.resolution_x = RENDER_WIDTH
    scene.render.resolution_y = RENDER_HEIGHT
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.look = "None"
    scene.view_settings.view_transform = "Standard"
    scene.world.color = srgb_to_linear(SKY_RGB)
    build_world(scene.world)

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
    sun_data.energy = SUN_ENERGY
    sun = bpy.data.objects.new("directional", sun_data)
    # A SUN emits along its local -Z. Roll about that axis is meaningless for a directional
    # light, so track_quat is safe here in a way it is NOT for the camera basis below.
    sun.rotation_euler = Vector(sun_direction()).to_track_quat("-Z", "Y").to_euler()
    bpy.context.collection.objects.link(sun)

    camera_data = bpy.data.cameras.new("boot camera")
    camera_data.sensor_fit = "VERTICAL"
    camera_data.angle = BOOT_VERTICAL_FOV
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


def render(args):
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
        # THE VENUE IS PART OF THE EVIDENCE. Two Blenders live on this machine since 2026-08-31
        # (apt 4.3.2 at /usr/bin, the vehicle-matching 5.2.1 at /opt/blender-5.2, bare `blender`
        # resolving to 5.2.1), and the figures below are NOT comparable across them: measured on
        # the same snapshot, non_sky_fraction moved 0.686815 -> 0.686736, distinct_colors
        # 58993 -> 59191, terrain_luma 106.260 -> 105.853, with 64.72% of pixels differing by at
        # most 2/255. Each version is still bit-deterministic with itself (0 of 518,400 pixels
        # across two runs). Without this field a recorded line cannot say which venue produced
        # it, and a PATH change would re-baseline the bench silently.
        f" blender={'.'.join(str(part) for part in bpy.app.version)}"
        f" exposed_cells={check['exposed_cells']}"
        f" non_sky_fraction={check['non_sky_fraction']:.6f}"
        f" distinct_colors={check['distinct_colors']}"
        f" terrain_luma={check['terrain_luma']:.3f}"
        f" floors(non_sky_fraction={check['minimum_non_sky_fraction']:.6f},"
        f" distinct_colors={check['minimum_distinct_colors']},"
        f" terrain_luma={check['minimum_terrain_luma']:.3f})"
    )
    try:
        assert_range(check)
    except AssertionError as error:
        # Blender logs an uncaught Python AssertionError yet exits 0; propagate failure to shell.
        raise SystemExit(f"range check failed: {error}") from error


def main():
    if bpy is None:
        raise SystemExit("run this script with blender")
    args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    if len(args) != 2:
        raise SystemExit("usage: valley_bench.py -- <snapshot.json> <out.png>")
    try:
        render(args)
    except Exception as error:
        # Blender's --background runner prints a traceback for ANY uncaught exception and still
        # exits 0. Guarding only the range assert left every other failure -- malformed JSON, an
        # entity kind the palette does not know, a dims/tiles length mismatch -- printing a
        # traceback and reporting success on a frame that was never rendered. Exit 0 is not a
        # result. SystemExit is deliberately not caught: assert_range already raises it.
        traceback.print_exc()
        raise SystemExit(f"bench failed: {type(error).__name__}: {error}") from error


if __name__ == "__main__":
    main()
