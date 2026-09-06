"""Render the dwarf's orthographic turnaround, for comparison against the reference.

    blender --background --python render_dwarf.py -- <out_dir>

Builds the model by importing dwarf_miner -- NOT by loading a .blend -- so the
renders are of the same geometry the generator ships and cannot drift from it.

Workbench, because it is deterministic and has no sampler noise. Two passes per
view: `flat` is unlit albedo, which is the only honest way to read the palette,
and `lit` is studio-lit, which is the only way to see that the forms are stepped
at all. The contact sheet is a torch-lit scene, so `lit` is the one to hold
beside it; neither is a claim about how the client will light him.

No camera, light or empty from this file ever reaches the GLB: the exporter runs
in dwarf_miner.py, in its own process, before any of this exists.
"""

import math
import os
import struct
import sys
import zlib

import bpy
import mathutils
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dwarf_miner as dwarf                                 # noqa: E402
import voxel_pine as pine                                   # noqa: E402

RES = (700, 700)
# (name, azimuth about +Z from +Y, elevation). He faces +Y, so his front is seen
# from +Y and his right (which carries the pickaxe) is +X.
VIEWS = [
    ("front", 0.0, 0.0),
    ("side-right", 90.0, 0.0),
    ("back", 180.0, 0.0),
    ("side-left", 270.0, 0.0),
    ("three-quarter", 35.0, 20.0),
]


def build():
    pine.wipe_scene()
    vox, _parts = dwarf.build_voxels()
    vox, _parts, _shift = dwarf.centre_voxels(vox, _parts)
    verts, faces, uvs = dwarf.build_mesh(vox, dwarf.DEFAULT_VOXEL)
    return dwarf.build_object(verts, faces, uvs)


def add_camera(ob):
    cam_data = bpy.data.cameras.new("TurnaroundCam")
    cam_data.type = 'ORTHO'
    cam_data.ortho_scale = max(ob.dimensions) * 1.06   # square frame: fits every view
    cam = bpy.data.objects.new("TurnaroundCam", cam_data)
    bpy.context.collection.objects.link(cam)
    bpy.context.scene.camera = cam
    return cam


def aim(cam, target, azimuth_deg, elevation_deg, radius=6.0):
    a, e = math.radians(azimuth_deg), math.radians(elevation_deg)
    offset = mathutils.Vector((math.sin(a) * math.cos(e),
                               math.cos(a) * math.cos(e),
                               math.sin(e))) * radius
    cam.location = target + offset
    # Look at the target: -Z of the camera points along the view direction.
    cam.rotation_euler = (-offset).to_track_quat('-Z', 'Y').to_euler()


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    out_dir = os.path.abspath(argv[0]) if argv else "renders"
    os.makedirs(out_dir, exist_ok=True)

    ob = build()
    scene = bpy.context.scene
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.render.resolution_x, scene.render.resolution_y = RES
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = 'PNG'
    scene.render.film_transparent = False
    scene.world = bpy.data.worlds.new("Flat") if scene.world is None else scene.world
    scene.world.color = (0.16, 0.16, 0.17)

    shading = scene.display.shading
    shading.color_type = 'TEXTURE'
    shading.show_object_outline = False
    shading.show_specular_highlight = False

    target = mathutils.Vector((0.0, 0.0, ob.dimensions.z / 2.0))
    cam = add_camera(ob)

    written = []
    for mode, light in (("flat", 'FLAT'), ("lit", 'STUDIO')):
        shading.light = light
        for name, azimuth, elevation in VIEWS:
            aim(cam, target, azimuth, elevation)
            path = os.path.join(out_dir, "dwarf-%s-%s.png" % (mode, name))
            scene.render.filepath = path
            bpy.ops.render.render(write_still=True)
            written.append(path)

    sheet = contact_comparison(out_dir)
    for path in written + [sheet]:
        print("RENDER %s" % path)
    print("OK %d renders -> %s" % (len(written) + 1, out_dir))


# --- the comparison the brief asks for -------------------------------------
# The renders have to be judged against the contact sheet, so put them in one
# image with a frame of it rather than leaving that to whoever opens the folder.
CONTACT = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "..", "references", "dwarf-contact-sheet.jpg")
CONTACT_FRAME = (1600, 540, 320, 180)      # frame r3c5: the clearest near-front idle


def read_png(path):
    image = bpy.data.images.load(path)
    width, height = image.size
    buffer = np.empty(width * height * 4, dtype=np.float32)
    image.pixels.foreach_get(buffer)
    bpy.data.images.remove(image)
    return np.clip(buffer.reshape(height, width, 4)[::-1, :, :3] * 255.0 + 0.5,
                   0, 255).astype(np.uint8)


def nearest_scale(image, height):
    factor = height / image.shape[0]
    width = int(image.shape[1] * factor)
    rows = (np.arange(height) / factor).astype(int).clip(0, image.shape[0] - 1)
    cols = (np.arange(width) / factor).astype(int).clip(0, image.shape[1] - 1)
    return image[rows][:, cols]


def write_png(path, pixels):
    # No escape sequences here on purpose: this file is edited by scripts and a
    # literal NUL or CR in the source is a corruption that only shows up as a
    # SyntaxError three edits later.
    magic = bytes((137, 80, 78, 71, 13, 10, 26, 10))
    height, width, _ = pixels.shape
    raw = b"".join(bytes(1) + pixels[row].tobytes() for row in range(height))

    def chunk(tag, payload):
        return (struct.pack(">I", len(payload)) + tag + payload
                + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF))

    with open(path, "wb") as handle:
        handle.write(magic
                     + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
                     + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def contact_comparison(out_dir):
    tiles = [read_png(os.path.join(out_dir, "dwarf-lit-%s.png" % name))
             for name, _a, _e in VIEWS]
    x, y, w, h = CONTACT_FRAME
    tiles.append(read_png(os.path.abspath(CONTACT))[y:y + h, x:x + w])
    height = max(tile.shape[0] for tile in tiles)
    strip = [nearest_scale(tile, height) for tile in tiles]
    canvas = np.concatenate(strip, axis=1)
    path = os.path.join(out_dir, "dwarf-vs-contact-sheet.png")
    write_png(path, canvas)
    return path


if __name__ == "__main__":
    main()
