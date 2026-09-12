"""Renders and comparison sheets for the round-7 dwarf.

    blender --background <blend> --python src-assets/blender/render_r7.py -- <mode> [res]

    mode `final`  the five delivered views, flat AND key-lit, into src-assets/renders/r7/
    mode `sheet`  the readability strip, the two ortho side-by-sides, the face vs f104

WHAT CHANGED FROM render_r5.py, and why. r5's "lit" pass was Workbench STUDIO, which
produces IDENTICAL RGB across about a tenth of the figure -- it adds no form, and every
look decision through round 6 was made under it. So:

  * the flat pass stays Workbench FLAT + TEXTURE. It is the only honest read of the
    albedo, it has no sampler noise, and every measurement in the report is taken off it.
  * the lit pass is EEVEE with ONE strong sun and real shadows, aimed roughly along
    f088's key -- high, from the figure's front left. That frame is where the reference's
    sense of form comes from, and a light with no shadow cannot show a stepped crown,
    a brow ridge or a nose.
  * every path is ABSOLUTE. Blender resolves a relative render.filepath against the
    .blend, which is how round 5's renders escaped to C:\\src-assets\\renders\\.
  * the revision is in every path: src-assets/renders/r7/.
"""

import math
import os
import sys

import bpy
from mathutils import Vector

REV = "r7"
COLLECTION = "SM_VoxelDwarf_Miner01_%s" % REV
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)                      # src-assets/
OUT = os.path.join(ROOT, "renders", REV)
REFS = os.path.join(ROOT, "references")

VIEWS = [("front", 0.0, 0.0), ("side-left", -90.0, 0.0), ("side-right", 90.0, 0.0),
         ("back", 180.0, 0.0), ("three-quarter", 35.0, 12.0)]


def _srgb_to_linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


BG = tuple(_srgb_to_linear(c / 255.0) for c in (0x6F, 0x70, 0x73))   # #6F7073


def figure_bounds():
    lo = Vector((1e9, 1e9, 1e9))
    hi = Vector((-1e9, -1e9, -1e9))
    for ob in bpy.data.collections[COLLECTION].objects:
        if ob.type != 'MESH':
            continue
        for v in ob.data.vertices:
            w = ob.matrix_world @ v.co
            lo = Vector((min(lo[i], w[i]) for i in range(3)))
            hi = Vector((max(hi[i], w[i]) for i in range(3)))
    return lo, hi


def purge_lights():
    for ob in list(bpy.data.objects):
        if ob.type in {'CAMERA', 'LIGHT'}:
            bpy.data.objects.remove(ob, do_unlink=True)


def setup_flat(res):
    scn = bpy.context.scene
    scn.render.engine = 'BLENDER_WORKBENCH'
    sh = scn.display.shading
    sh.light = 'FLAT'
    sh.color_type = 'TEXTURE'
    sh.show_object_outline = False
    sh.show_specular_highlight = False
    _common(scn, res)


def setup_lit(res):
    """One sun, sharp shadow, aimed along f088's key: high and from the front left."""
    scn = bpy.context.scene
    scn.render.engine = 'BLENDER_EEVEE'
    # Raytracing OFF: with one sun and flat albedo it contributes nothing but sampler
    # grain, and grain on a flat-shaded asset reads as texture that is not there.
    for attr, val in (("taa_render_samples", 64), ("use_shadows", True),
                      ("use_raytracing", False)):
        if hasattr(scn.eevee, attr):
            setattr(scn.eevee, attr, val)
    key = bpy.data.objects.get("R7Key")
    if key is None:
        key = bpy.data.objects.new("R7Key", bpy.data.lights.new("R7Key", 'SUN'))
        scn.collection.objects.link(key)
    key.data.energy = 4.2
    key.data.angle = math.radians(3.0)          # nearly parallel -> a crisp shadow
    key.data.use_shadow = True
    _aim(key, -42.0, 34.0)
    # A weak, shadowless fill from behind the opposite shoulder. Without it the two side
    # views are read against a single key and half the figure is unjudgeable; with it the
    # key still carries every form, which is the point f088 makes.
    fill = bpy.data.objects.get("R7Fill")
    if fill is None:
        fill = bpy.data.objects.new("R7Fill", bpy.data.lights.new("R7Fill", 'SUN'))
        scn.collection.objects.link(fill)
    fill.data.energy = 1.1
    fill.data.angle = math.radians(40.0)
    fill.data.use_shadow = False
    _aim(fill, 138.0, 12.0)
    _common(scn, res)


def _aim(lamp, az, el):
    a, e = math.radians(az), math.radians(el)
    lamp.location = Vector((math.sin(a) * math.cos(e),
                            math.cos(a) * math.cos(e), math.sin(e))) * 6.0
    lamp.rotation_euler = (Vector((0, 0, 0.6)) - lamp.location).to_track_quat('-Z', 'Y').to_euler()


def _common(scn, res):
    scn.render.resolution_x = scn.render.resolution_y = res
    scn.render.resolution_percentage = 100
    scn.render.film_transparent = False
    scn.render.image_settings.file_format = 'PNG'
    # World.use_nodes is on its way out in 5.x and setting world.color no longer reaches
    # EEVEE, so drive the background node directly: it is both the backdrop the flat pass
    # is measured against AND the only ambient fill under the single key.
    w = scn.world
    w.use_nodes = True
    bgn = next((n for n in w.node_tree.nodes if n.type == 'BACKGROUND'), None)
    if bgn is None:
        bgn = w.node_tree.nodes.new('ShaderNodeBackground')
        out = next(n for n in w.node_tree.nodes if n.type == 'OUTPUT_WORLD')
        w.node_tree.links.new(bgn.outputs['Background'], out.inputs['Surface'])
    bgn.inputs['Color'].default_value = BG + (1.0,)
    bgn.inputs['Strength'].default_value = 1.0
    w.color = BG
    scn.view_settings.view_transform = 'Standard'
    scn.view_settings.look = 'None'


def place_camera(az, el, span, centre):
    cam = bpy.data.objects.get("R7Cam")
    if cam is None:
        cam = bpy.data.objects.new("R7Cam", bpy.data.cameras.new("R7Cam"))
        bpy.context.scene.collection.objects.link(cam)
    cam.data.type = 'ORTHO'
    cam.data.ortho_scale = span
    c = Vector(centre)
    a, e = math.radians(az), math.radians(el)
    # He faces +Y, so azimuth 0 -- his FRONT -- is seen from +Y. His right is +X.
    cam.location = c + Vector((math.sin(a) * math.cos(e),
                               math.cos(a) * math.cos(e), math.sin(e))) * 6.0
    cam.rotation_euler = (c - cam.location).to_track_quat('-Z', 'Y').to_euler()
    bpy.context.scene.camera = cam
    return cam


def shot(path, az, el, res, lit, span, centre):
    (setup_lit if lit else setup_flat)(res)
    place_camera(az, el, span, centre)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.context.scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    return path


# ---------------------------------------------------------------- comparison sheets

def _read(path):
    import numpy as np
    img = bpy.data.images.load(path, check_existing=False)
    w, h = img.size
    a = np.array(img.pixels[:], dtype=np.float32).reshape(h, w, 4)[::-1, :, :3]
    if img.colorspace_settings.name != 'sRGB':      # our renders come back linear
        a = np.where(a <= 0.0031308, a * 12.92,
                     1.055 * np.clip(a, 1e-8, None) ** (1 / 2.4) - 0.055)
    bpy.data.images.remove(img)
    return np.clip(a, 0, 1)


def _write(path, arr):
    import numpy as np
    h, w = arr.shape[:2]
    img = bpy.data.images.new(os.path.basename(path), w, h, alpha=False)
    img.colorspace_settings.name = 'sRGB'
    img.pixels = np.concatenate([arr[::-1], np.ones((h, w, 1), np.float32)], axis=2).ravel().tolist()
    img.filepath_raw = path
    img.file_format = 'PNG'
    img.save()
    bpy.data.images.remove(img)
    return path


def _nn(arr, w, h):
    import numpy as np
    ys = (np.arange(h) * arr.shape[0] / h).astype(int)
    xs = (np.arange(w) * arr.shape[1] / w).astype(int)
    return arr[ys][:, xs]


def _trim(arr, bg=(0x6F / 255., 0x70 / 255., 0x73 / 255.), tol=0.02):
    import numpy as np
    d = np.abs(arr - np.array(bg, np.float32)).max(axis=2) > tol
    ys, xs = np.nonzero(d)
    return arr[ys.min():ys.max() + 1, xs.min():xs.max() + 1]


def sheets(res):
    """The readability strip, the ortho side-by-sides and the face-vs-f104 plate."""
    import numpy as np
    lo, hi = figure_bounds()
    span = max(hi.x - lo.x, hi.y - lo.y, hi.z - lo.z) * 1.06
    centre = (lo + hi) / 2.0
    tmp = os.path.join(OUT, "_tmp")

    # --- readability: the figure at 100 px and 60 px of FIGURE height, key-lit
    strip_src = shot(os.path.join(tmp, "read-src.png"), 0.0, 0.0, 900, True, span, centre)
    tq_src = shot(os.path.join(tmp, "read-tq.png"), 35.0, 12.0, 900, True, span, centre)
    tiles = []
    for src in (strip_src, tq_src):
        fig = _trim(_read(src))
        for px in (100, 60):
            w = max(1, int(round(fig.shape[1] * px / fig.shape[0])))
            small = _nn(fig, w, px)
            tiles.append((small, px))
    pad = 12
    H = max(t.shape[0] for t, _ in tiles) + 2 * pad
    W = sum(t.shape[1] + pad for t, _ in tiles) + pad
    canvas = np.full((H, W, 3), 0.42, np.float32)
    x = pad
    for t, px in tiles:
        canvas[H - pad - t.shape[0]:H - pad, x:x + t.shape[1]] = t
        x += t.shape[1] + pad
    _write(os.path.join(OUT, "readability.png"), _nn(canvas, W * 3, H * 3))

    # --- side by side against the orthographic sheet, matched on figure height
    for name, ref in (("front", "front.png"), ("side-left", "side-left.png")):
        az = 0.0 if name == "front" else -90.0
        ours = _trim(_read(shot(os.path.join(tmp, "vs-%s.png" % name), az, 0.0, 900,
                                False, span, centre)))
        them = _trim(_read(os.path.join(REFS, "dwarf-ortho", ref)),
                     bg=(0.90, 0.92, 0.93), tol=0.06)
        h = 700
        a = _nn(ours, max(1, int(ours.shape[1] * h / ours.shape[0])), h)
        b = _nn(them, max(1, int(them.shape[1] * h / them.shape[0])), h)
        W = a.shape[1] + b.shape[1] + 3 * pad
        c = np.full((h + 2 * pad, W, 3), 0.42, np.float32)
        c[pad:pad + h, pad:pad + a.shape[1]] = a
        c[pad:pad + h, 2 * pad + a.shape[1]:2 * pad + a.shape[1] + b.shape[1]] = b
        _write(os.path.join(OUT, "vs-ortho-%s.png" % name), c)

    # --- the face, beside f104, matched on head height
    ours = _read(shot(os.path.join(tmp, "vs-face.png"), 0.0, 0.0, 700, True, 0.50, (0, 0, 1.00)))
    them = _read(os.path.join(REFS, "dwarf-frames", "f104.png"))[30:250, 120:330]
    h = 640
    a = _nn(ours, int(ours.shape[1] * h / ours.shape[0]), h)
    b = _nn(them, int(them.shape[1] * h / them.shape[0]), h)
    W = a.shape[1] + b.shape[1] + 3 * pad
    c = np.full((h + 2 * pad, W, 3), 0.42, np.float32)
    c[pad:pad + h, pad:pad + a.shape[1]] = a
    c[pad:pad + h, 2 * pad + a.shape[1]:2 * pad + a.shape[1] + b.shape[1]] = b
    _write(os.path.join(OUT, "vs-face.png"), c)
    print("SHEETS -> %s" % OUT)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else ["final"]
    mode = argv[0]
    res = int(argv[1]) if len(argv) > 1 else 700
    purge_lights()
    lo, hi = figure_bounds()
    span = max(hi.x - lo.x, hi.y - lo.y, hi.z - lo.z) * 1.06
    centre = (lo + hi) / 2.0
    if mode == "final":
        for name, az, el in VIEWS:
            shot(os.path.join(OUT, "dwarf-flat-%s.png" % name), az, el, res, False, span, centre)
            shot(os.path.join(OUT, "dwarf-lit-%s.png" % name), az, el, res, True, span, centre)
    elif mode == "sheets":
        sheets(res)
    else:
        raise SystemExit("render_r7: unknown mode %r" % mode)
    print("RENDERED mode=%s span=%.4f -> %s" % (mode, span, OUT))


if __name__ == "__main__":
    main()
