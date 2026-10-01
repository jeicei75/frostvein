import bpy, math, os, sys

# Usage: blender -b --python draft.py -- /abs/out/draft-crew.png  (Cycles CPU, ~1 min on the devpod)

GLB = "/workspace/projects/frostvein/assets/gltf/SM_VoxelDwarf_Miner01.glb"
OUT = sys.argv[sys.argv.index("--") + 1]
# Tunic cells (row, col) of 32-px cells, from a UV->joint census: chest/hips/shoulders only.
TUNIC = [(9, 0), (9, 1), (9, 2), (9, 3), (10, 15)]
CREW = [  # candidate crew colours, sRGB
    ("Durin", (178, 58, 52)),     # red
    ("Nori", (214, 164, 44)),     # gold
    ("Bifur", (62, 146, 76)),     # green
    ("Dvalin", (60, 98, 186)),    # blue
    ("Frosti", (128, 76, 168)),   # purple
]

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.device = "CPU"
scene.cycles.samples = 32
scene.cycles.use_denoising = False
scene.render.resolution_x, scene.render.resolution_y = 1600, 700
scene.view_settings.view_transform = "Standard"

world = bpy.data.worlds.new("w")
scene.world = world
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = (0.55, 0.62, 0.72, 1)
world.node_tree.nodes["Background"].inputs[1].default_value = 0.8

def lum(r, g, b):
    return 0.2126 * r + 0.7152 * g + 0.0722 * b

for i, (name, rgb) in enumerate(CREW):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=GLB)
    new = [o for o in bpy.data.objects if o not in before]
    for o in new:
        if o.parent is None:
            o.location.x += (i - 2) * 1.1
    mesh = [o for o in new if o.type == "MESH" and len(o.data.materials)][0]
    mat = mesh.material_slots[0].material.copy()
    mesh.material_slots[0].material = mat
    tex = [n for n in mat.node_tree.nodes if n.type == "TEX_IMAGE"][0]
    src = tex.image
    w, h = src.size
    px = list(src.pixels)  # bottom-up rows, stored (sRGB) values for a byte image
    cells = []
    for (r, c) in TUNIC:
        for y in range(r * 32, r * 32 + 32):
            for x in range(c * 32, c * 32 + 32):
                cells.append(((h - 1 - y) * w + x) * 4)
    ref = max(lum(*px[k:k + 3]) for k in cells)
    t = [v / 255 for v in rgb]
    for k in cells:
        s = lum(*px[k:k + 3]) / ref
        px[k:k + 3] = [min(1.0, v * s) for v in t]
    img = bpy.data.images.new(f"atlas_{name}", w, h, alpha=True)
    img.pixels = px
    path = os.path.join(os.path.dirname(OUT), f"atlas_{name}.png")
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    tex.image = bpy.data.images.load(path)

bpy.ops.mesh.primitive_plane_add(size=20, location=(0, 0, 0))
snow = bpy.data.materials.new("snow")
snow.use_nodes = True
snow.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.8, 0.84, 0.9, 1)
bpy.context.object.data.materials.append(snow)

bpy.ops.object.light_add(type="SUN", location=(0, 0, 10))
sun = bpy.context.object
sun.data.energy = 3.0
sun.rotation_euler = (math.radians(50), 0, math.radians(30))

bpy.ops.object.camera_add(location=(0, 7.5, 2.4))
cam = bpy.context.object
cam.rotation_euler = (math.radians(78), 0, math.radians(180))
cam.data.lens = 50
scene.camera = cam
scene.render.filepath = OUT
bpy.ops.render.render(write_still=True)
