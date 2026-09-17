"""Render every r10 deliverable from the committed .blend, headless.

ONE COMMAND, from the repo root:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/render_r10.py

Writes src-assets/renders/r10/. Authoring happens in Wolf's running Blender through the MCP
addon (round 10 sec.8); only this and export_dwarf.py are allowed to run headless, and this
script NEVER edits geometry -- it poses, renders, restores, and composites.

IT IS ALSO THE SESSION'S RENDER LIBRARY. The authoring session imports `camera`, `render` and
`progress` from here through the MCP addon so the progress renders and the stage sets come out
of ONE definition of "front", "flat" and "key-lit" rather than two that drift. Everything below
`main()` is the headless deliverable pass; everything above it is safe to import.

WHAT "FLAT" AND "KEY-LIT" MEAN HERE. Both are Workbench. Flat is `light='FLAT'`, which shows the
atlas albedo with no shading at all -- the honest way to check paint. Key-lit is `light='STUDIO'`,
Blender's fixed three-point studio, deterministic across machines in a way a hand-placed sun in
EEVEE is not. Game lighting is Epic 11's and is not simulated here.

THE COMPARISON SHEETS are composited from the rendered PNGs and the reference images with
Blender's own image API, so the round needs no image library beyond Blender.
"""

import math
import os

import bpy

REV = "r10"
COLL = f"SM_VoxelDwarf_Miner01_{REV}"
REFCOLL = f"{REV}_reference"
HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.dirname(HERE)
OUT = os.path.join(ASSETS, "renders", REV)
REF_FRAMES = os.path.join(ASSETS, "references", "dwarf-frames")
REF_ORTHO = os.path.join(ASSETS, "references", "dwarf-ortho")
H = 1.200                       # the figure's height, and the unit every scale below is in

# Camera location and XYZ euler in degrees. The dwarf faces +Y, so "front" looks from +Y back
# along -Y; "side" looks from +X, which is the sheet's side-left.png orientation (nose to the
# right of frame) and the only one vs-ortho-side-left.png may be built from.
VIEWS = {
    "front":     ((0.0, 3.0, 0.60), (90, 0, 180)),
    "side":      ((3.0, 0.0, 0.60), (90, 0, 90)),
    "back":      ((0.0, -3.0, 0.60), (90, 0, 0)),
    "quarter":   ((2.1, 2.1, 0.60), (90, 0, 135)),
    "quarter.L": ((-2.1, 2.1, 0.60), (90, 0, 225)),
    "head":      ((0.0, 3.0, 1.01), (90, 0, 180)),
    "head.q":    ((2.1, 2.1, 1.01), (90, 0, 135)),
}
SCALE = {"head": 0.46, "head.q": 0.46}   # ortho width in m; everything else frames the figure
FRAME = 1.75          # the A-pose spans 1.103 m; 1.45 cropped the hands and the props


def camera(view, ortho=None):
    name = f"{REV}cam." + view.replace(".", "_")
    ob = bpy.data.objects.get(name)
    if ob is None:
        ob = bpy.data.objects.new(name, bpy.data.cameras.new(name))
        bpy.context.scene.collection.objects.link(ob)
    loc, rot = VIEWS[view]
    ob.data.type = 'ORTHO'
    ob.data.ortho_scale = ortho or SCALE.get(view, FRAME)
    ob.location = loc
    ob.rotation_euler = [math.radians(a) for a in rot]
    return ob


def show_reference(view, on):
    """The two ortho image planes.

    Shown for a silhouette comparison, hidden for a render set -- and only ever the ONE plane that
    faces the camera. Both at once puts the side plane edge-on in the front view, which reads as a
    stray black rule down the frame and has already been mistaken for geometry once.
    """
    coll = bpy.data.collections.get(REFCOLL)
    if coll is None:
        return
    want = {"front": "front", "back": "front", "side": "side"}.get(view)
    for ob in coll.objects:
        ob.hide_render = not (on and ob.name.endswith(want or "\x00"))


def render(path, view="front", res=(640, 940), flat=False, ortho=None, reference=False,
           transparent=False, colour=None):
    scene = bpy.context.scene
    show_reference(view, reference)
    cam = camera(view, ortho)
    scene.camera = cam
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.render.resolution_x, scene.render.resolution_y = res
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = transparent
    shading = scene.display.shading
    shading.light = 'FLAT' if flat else 'STUDIO'
    shading.color_type = colour or 'TEXTURE'
    shading.show_cavity = False
    shading.show_backface_culling = True
    full = path if os.path.isabs(path) else os.path.join(OUT, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    scene.render.filepath = full
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA' if transparent else 'RGB'
    bpy.ops.render.render(write_still=True)
    return full


def progress(n, operation, view="quarter", reference=False, res=(400, 580), colour=None):
    """Deliverable 5: one numbered render per authoring step, written as the work happens."""
    name = f"progress/{n:02d}-{operation}.png"
    return render(name, view=view, res=res, reference=reference, colour=colour)


def stage_set(n, label, views=("front", "side", "quarter", "back"), reference=True,
              colour=None):
    """Deliverable 4: the four-view set that closes a stage, against the reference planes."""
    return [render(f"stage-{n}-{label}-{v}.png", view=v, colour=colour,
                   reference=reference and v in ("front", "side"))
            for v in views]


# ------------------------------------------------------- exact-scale sheet comparison
# The ortho crops are 5x nearest-neighbour blow-ups of the sheet, so ONE source pixel is
# 1.200/140 m = 8.571 mm. These two framings put one rendered pixel on one source pixel, which
# is what makes a silhouette comparison a measurement rather than an impression.
PX = 1.200 / 140.0
SHEET = {                     # (res_x, res_y), camera centre, ortho_scale (Blender = larger dim)
    "front": ((174, 154), (-0.5 * (174 - 2 * 77.5) * PX, 3.0, (147.5 + (147 - 153.5)) / 2 * PX),
              174 * PX),
    "side":  ((120, 154), (3.0, ((119.5 - 58) + (-58.5)) / 2 * PX, (147.5 + (147 - 153.5)) / 2 * PX),
              154 * PX),
}


def sheet(view, path, flat=True):
    """Render the figure alone, alpha-cut, at exactly one rendered pixel per source pixel.

    The framing goes through VIEWS rather than onto the camera object, because render() rebuilds
    the camera from VIEWS on every call and would otherwise put the location straight back.
    """
    res, loc, ortho = SHEET[view]
    keep = VIEWS[view]
    VIEWS[view] = (loc, keep[1])
    try:
        return render(path, view=view, res=res, flat=flat, ortho=ortho, transparent=True)
    finally:
        VIEWS[view] = keep


# ------------------------------------------------------------------ compositing
# Numpy through Blender's own image API, so the round needs no image library beyond Blender.
# Everything is handled top-down; Blender's buffers are bottom-up, hence the [::-1].
import numpy as np


def _load(path):
    img = bpy.data.images.load(path, check_existing=False)
    w, h = img.size
    a = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(a)
    bpy.data.images.remove(img)
    return a.reshape(h, w, 4)[::-1].copy()


def _save(arr, name):
    h, w = arr.shape[:2]
    img = bpy.data.images.new(name, w, h, alpha=True)
    img.pixels.foreach_set(np.ascontiguousarray(arr[::-1]).reshape(-1).astype(np.float32))
    path = os.path.join(OUT, name)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img.filepath_raw = path
    img.file_format = 'PNG'
    img.save()
    bpy.data.images.remove(img)
    return path


def _nearest(arr, w, h):
    ys = (np.arange(h) * arr.shape[0] / h).astype(int).clip(0, arr.shape[0] - 1)
    xs = (np.arange(w) * arr.shape[1] / w).astype(int).clip(0, arr.shape[1] - 1)
    return arr[ys][:, xs]


def _over(dst, src, x, y):
    h, w = src.shape[:2]
    y0, x0 = max(0, y), max(0, x)
    y1, x1 = min(dst.shape[0], y + h), min(dst.shape[1], x + w)
    if y1 <= y0 or x1 <= x0:
        return
    s = src[y0 - y:y1 - y, x0 - x:x1 - x]
    a = s[:, :, 3:4]
    dst[y0:y1, x0:x1, :3] = s[:, :, :3] * a + dst[y0:y1, x0:x1, :3] * (1 - a)
    dst[y0:y1, x0:x1, 3:4] = np.maximum(dst[y0:y1, x0:x1, 3:4], a)


def _canvas(w, h, rgb=(0.13, 0.13, 0.14)):
    c = np.zeros((h, w, 4), dtype=np.float32)
    c[:, :, :3] = rgb
    c[:, :, 3] = 1.0
    return c


# ------------------------------------------------------------------ deliverables
FINAL = ("front", "side", "back", "quarter", "quarter.L")


def final_views():
    """Deliverable 6: five views, flat (albedo only) and key-lit (Workbench STUDIO)."""
    out = []
    for v in FINAL:
        out.append(render(f"final-flat-{v.replace('.', '_')}.png", view=v, res=(640, 940), flat=True))
        out.append(render(f"final-lit-{v.replace('.', '_')}.png", view=v, res=(640, 940)))
    return out


def vs_frames():
    """Deliverable 7: beside f088 and f104 at matched scale.

    The frames are 380 x 622 and their dwarf stands ~500 px from sole to crown, so the render is
    framed to put 1.200 m on 500 px. f088 is a crouch, so the MATCH IS OF SCALE, not of pose --
    the point is whether the masses read the same size against the same background.
    """
    made = []
    for frame, view in (("f088", "quarter.L"), ("f104", "quarter")):
        ref = _load(os.path.join(REF_FRAMES, frame + ".png"))
        h, w = ref.shape[:2]
        ortho = 1.200 * h / 500.0
        path = render(f"_tmp-vs-{frame}.png", view=view, res=(w, h), ortho=ortho, transparent=True)
        shot = _load(path)
        os.remove(path)
        c = _canvas(w * 2 + 12, h)
        _over(c, ref, 0, 0)
        _over(c, shot, w + 12, 0)
        made.append(_save(c, f"vs-frames-{frame}.png"))
    return made


def vs_ortho_side():
    """Deliverable 8: the side profile beside side-left.png, one rendered px on one source px."""
    ref = _load(os.path.join(REF_ORTHO, "side-left.png"))
    h, w = ref.shape[:2]
    path = sheet("side", os.path.join(OUT, "_tmp-side.png"))
    shot = _nearest(_load(path), w, h)
    os.remove(path)
    c = _canvas(w * 2 + 20, h)
    _over(c, ref, 0, 0)
    _over(c, shot, w + 20, 0)
    return _save(c, "vs-ortho-side-left.png")


def readability():
    """Deliverable 9: the figure at 100 px and 60 px tall, then blown up 4x nearest to be judged."""
    tiles = []
    for px in (100, 60):
        wide = max(1, int(round(px * 1.103 / 1.200)))
        path = render(f"_tmp-read-{px}.png", view="front", res=(wide, px), ortho=1.200,
                      transparent=True)
        tiles.append((px, _load(path)))
        os.remove(path)
        path = render(f"_tmp-readq-{px}.png", view="quarter", res=(wide, px), ortho=1.200,
                      transparent=True)
        tiles.append((px, _load(path)))
        os.remove(path)
    W = sum(t.shape[1] * 4 + 16 for _, t in tiles) + 16
    H = max(t.shape[0] * 4 for _, t in tiles) + 32
    c = _canvas(W, H)
    x = 16
    for px, t in tiles:
        big = _nearest(t, t.shape[1] * 4, t.shape[0] * 4)
        _over(c, big, x, 16)
        x += big.shape[1] + 16
    return _save(c, "readability.png")


# ------------------------------------------------------------------ pose and deflection
import math as _math
from mathutils import Euler

ARMATURE = f"SK_VoxelDwarf_Miner01_{REV}"

# Deliverable 10. One deflection per JOINT GROUP -- paired joints move together, so twelve
# renders cover the nineteen names (root carries no geometry, so it has nothing to deflect).
DEFLECT = [
    ("hips",     {"hips": (0, 0, 22)}),
    ("spine",    {"spine": (14, 0, 0)}),
    ("chest",    {"chest": (-16, 0, 0)}),
    ("neck",     {"neck": (12, 0, 0)}),
    ("head",     {"head": (0, 0, 30)}),
    ("beard",    {"beard": (25, 0, 0)}),
    ("shoulder", {"shoulder.R": (0, 45, 0), "shoulder.L": (0, -45, 0)}),
    ("elbow",    {"elbow.R": (0, 55, 0), "elbow.L": (0, -55, 0)}),
    ("hand",     {"hand.R": (0, 35, 0), "hand.L": (0, -35, 0)}),
    ("hip",      {"hip.R": (30, 0, 0), "hip.L": (-18, 0, 0)}),
    ("knee",     {"knee.R": (-35, 0, 0), "knee.L": (0, 0, 0)}),
    ("foot",     {"foot.R": (22, 0, 0), "foot.L": (-14, 0, 0)}),
]

# The two-handed carry: a POSE, delivered as a render, which is where sec.4 says the arms are
# finally compared against the sheet. The pickaxe is weighted to hand.R, so it comes along.
CARRY = {
    "shoulder.R": (-8, -34, -26), "elbow.R": (0, -38, 0), "hand.R": (0, -10, 0),
    "shoulder.L": (10, 30, 30),   "elbow.L": (0, 46, 0),  "hand.L": (0, 12, 0),
    "chest": (-6, 0, 0), "head": (0, 0, -8),
}


def _pose(rots):
    arm = bpy.data.objects[ARMATURE]
    for pb in arm.pose.bones:
        pb.rotation_mode = 'XYZ'
        pb.rotation_euler = Euler((0, 0, 0))
    for name, (rx, ry, rz) in rots.items():
        pb = arm.pose.bones.get(name)
        if pb:
            pb.rotation_euler = Euler([_math.radians(a) for a in (rx, ry, rz)])
    bpy.context.view_layer.update()


def poses():
    made = []
    _pose(CARRY)
    made.append(render("pose-carry-front.png", view="front", res=(640, 940)))
    made.append(render("pose-carry-quarter.png", view="quarter", res=(640, 940)))
    for label, rots in DEFLECT:
        _pose(rots)
        made.append(render(f"joint-{label}.png", view="quarter", res=(460, 660)))
    _pose({})
    return made


def main():
    show_reference("front", False)
    show_reference("side", False)
    out = []
    # Stage 3 is shown in CLAY. Its set is rendered here rather than at the time, because the
    # paint pass changed no geometry -- what stage 3 owes is the carved surface, and SINGLE
    # shading is the honest way to show it once a texture exists.
    out += stage_set(3, "carved", reference=False, colour="SINGLE")
    out += stage_set(4, "textured", reference=False)
    out += final_views()
    out += vs_frames()
    out.append(vs_ortho_side())
    out.append(readability())
    out += poses()
    print("\nRENDERED %d files into %s" % (len(out), OUT))
    for p in out:
        print("  " + os.path.relpath(p, ASSETS))


if __name__ == "__main__":
    main()
