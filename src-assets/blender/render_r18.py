"""Round 18 renders -- pose strips for the `Walk` cycle.

Run it in the LIVE session, after walk_r18.build():

    exec(open(r"src-assets/blender/render_r18.py").read())
    sheet("keys-side", [1, 4, 7, 10], 90)

One strip per view, one column per frame, stitched here rather than in a paint program
so the columns are guaranteed the same camera. Cycles on the GPU, because a Cycles
rendered-viewport grab over MCP comes back black -- it is taken before convergence -- so
everything judged by eye has to land in a file first.

The dwarf faces +Y, so AZIM 180 is the camera in front of his nose and AZIM 90 is the
camera off his right shoulder, the side the pickaxe is on. A camera at -Y renders the
backpack and reads as a broken figure.
"""

import os

import bpy
import numpy as np
from mathutils import Vector, Euler

OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(
    bpy.data.filepath or "x"))), "renders", "r18")
TARGET = Vector((0.0, 0.0, 0.67))
SPAN = 1.60                 # metres of world height in frame
COLW, COLH = 300, 420


def cam(azim_deg, elev_deg=0.0, dist=4.0):
    """Ortho camera at `azim` degrees around Z. 180 = facing his nose, 90 = his right."""
    ob = bpy.data.objects.get("cam_r18")
    if ob is None:
        data = bpy.data.cameras.new("cam_r18")
        ob = bpy.data.objects.new("cam_r18", data)
        bpy.context.scene.collection.objects.link(ob)
    ob.data.type = 'ORTHO'
    ob.data.ortho_scale = SPAN
    rot = Euler((np.radians(90.0 - elev_deg), 0.0, np.radians(azim_deg)), 'XYZ')
    ob.rotation_euler = rot
    back = rot.to_quaternion() @ Vector((0.0, 0.0, 1.0))
    ob.location = TARGET + back * dist
    bpy.context.scene.camera = ob
    return ob


def cycles(samples=48):
    sc = bpy.context.scene
    sc.render.engine = 'CYCLES'
    sc.cycles.device = 'GPU'
    sc.cycles.samples = samples
    sc.cycles.use_denoising = True
    sc.render.film_transparent = False
    sc.render.resolution_x, sc.render.resolution_y = COLW, COLH
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = 'PNG'
    sc.render.image_settings.color_mode = 'RGBA'


def frame_px(f):
    """Render frame f and return its pixels as a (h, w, 4) float array."""
    sc = bpy.context.scene
    sc.frame_set(f)
    tmp = os.path.join(bpy.app.tempdir, "r18_%03d.png" % f)
    sc.render.filepath = tmp
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(tmp, check_existing=False)
    a = np.array(img.pixels[:], dtype=np.float32).reshape(img.size[1], img.size[0], 4)
    bpy.data.images.remove(img)
    return a


def sheet(name, frames, azim, elev=0.0, samples=48):
    cam(azim, elev)
    cycles(samples)
    cols = [frame_px(f) for f in frames]
    strip = np.concatenate(cols, axis=1)
    # a one-pixel rule between columns, so a pose that barely differs still reads as two
    for i in range(1, len(cols)):
        strip[:, i * COLW, :3] = 0.10
    os.makedirs(OUT, exist_ok=True)
    path = os.path.join(OUT, name + ".png")
    out = bpy.data.images.new(name, width=strip.shape[1], height=strip.shape[0], alpha=True)
    out.pixels = strip.reshape(-1).tolist()
    out.filepath_raw = path
    out.file_format = 'PNG'
    out.save()
    bpy.data.images.remove(out)
    print("wrote %s   %d x %d   frames %s" % (path, strip.shape[1], strip.shape[0], frames))
    return path
