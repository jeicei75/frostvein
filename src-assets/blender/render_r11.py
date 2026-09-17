"""Render every r11 deliverable from the committed .blend, headless.

ONE COMMAND, from the repo root:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/render_r11.py -- <mode> [args]

    <mode>  stage N          the stage-N render set, against the reference planes
            progress NN tag  one numbered progress render
            final            every final deliverable: five views, vs-frames, vs-ortho,
                             readability, the carry pose and the joint deflections

Writes src-assets/renders/r11/. Authoring happens in Wolf's running Blender through the MCP
addon (round 11 sec.8); only this and export_dwarf.py are allowed to run headless, and this
script NEVER edits geometry -- it poses, renders, restores, and composites.

Adapted from render_r9.py, which is the script that produced the round Wolf accepted on form.
Two things are new. STAGE RENDERS KEEP THE REFERENCE PLANES VISIBLE, because round 11 judges
its cage against the ortho sheet at every stage boundary and a silhouette you cannot see the
target behind is not a comparison. And `progress` exists because sec.8 wants a numbered render
trail through the authoring, not only at the boundaries.

WHAT "FLAT" AND "KEY-LIT" MEAN HERE. Both are Workbench. Flat is `light='FLAT'`, which shows
the atlas albedo with no shading at all -- the honest way to check paint. Key-lit is
`light='STUDIO'`, Blender's fixed three-point studio, which is deterministic across machines
in a way a hand-placed sun in EEVEE is not. Game lighting is Epic 11's and is not simulated.

THE COMPARISON SHEETS are composited from the rendered PNGs and the reference images with
Blender's own image API, so the round needs no image library beyond Blender.
"""

import math
import os
import sys

import bpy
from mathutils import Euler, Matrix

REV = "r11"
COLL = f"SM_VoxelDwarf_Miner01_{REV}"
REFCOLL = f"{REV}_reference"
HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.dirname(HERE)
OUT = os.path.join(ASSETS, "renders", REV)
REF_FRAMES = os.path.join(ASSETS, "references", "dwarf-frames")
REF_ORTHO = os.path.join(ASSETS, "references", "dwarf-ortho")
H = 1.200                       # the figure's height, and the unit every scale below is in

# camera location and XYZ euler in degrees. The dwarf faces +Y, so "front" looks from +Y.
# "side" looks from +X, which is the view side-left.png is drawn in: its image-right is +Y.
VIEWS = {
    "front":     ((0.0, 3.0, 0.60), (90, 0, 180)),
    "side":      ((3.0, 0.0, 0.60), (90, 0, 90)),
    "back":      ((0.0, -3.0, 0.60), (90, 0, 0)),
    "quarter":   ((2.1, 2.1, 0.60), (90, 0, 135)),
    "quarter.L": ((-2.1, 2.1, 0.60), (90, 0, 225)),
    "head":      ((0.0, 3.0, 1.01), (90, 0, 180)),
    "head.q":    ((1.5, 1.5, 1.01), (90, 0, 135)),
}
# ortho width in m; everything else frames the whole figure. head.q is wider than head
# because a 0.40 x 0.36 m head seen corner-on spans ~0.54 m, and a 0.42 frame crops it.
SCALE = {"head": 0.44, "head.q": 0.62}


def camera(view):
    name = "r11cam." + view.replace(".", "_")
    ob = bpy.data.objects.get(name)
    if ob is None:
        ob = bpy.data.objects.new(name, bpy.data.cameras.new(name))
        bpy.context.scene.collection.objects.link(ob)
    loc, rot = VIEWS[view]
    ob.data.type = 'ORTHO'
    ob.data.ortho_scale = SCALE.get(view, 1.45)
    ob.location = loc
    ob.rotation_euler = [math.radians(a) for a in rot]
    return ob


def show_refs(on):
    """The reference planes are scaffolding: visible while judging a cage, never in a final."""
    for name in ("r11_ref_front", "r11_ref_side"):
        ob = bpy.data.objects.get(name)
        if ob is not None:
            ob.hide_render = not on


def render(path, view="front", res=(640, 940), flat=False, ortho=None, refs=False):
    scene = bpy.context.scene
    cam = camera(view)
    if ortho:
        cam.data.ortho_scale = ortho
    scene.camera = cam
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.render.resolution_x, scene.render.resolution_y = res
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = False
    shading = scene.display.shading
    shading.light = 'FLAT' if flat else 'STUDIO'
    shading.color_type = 'TEXTURE'
    shading.show_cavity = False
    shading.show_backface_culling = True
    show_refs(refs)
    full = os.path.join(OUT, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    scene.render.filepath = full
    scene.render.image_settings.file_format = 'PNG'
    bpy.ops.render.render(write_still=True)
    return full


# ---------------------------------------------------------------- compositing
class Canvas:
    """A row-major RGBA buffer with Blender's bottom-left origin, so paste() can nearest-scale."""

    def __init__(self, w, h, rgb=(0.16, 0.16, 0.17)):
        self.w, self.h = w, h
        self.buf = [rgb[0], rgb[1], rgb[2], 1.0] * (w * h)

    def paste(self, src, sw, sh, x, y, scale=1):
        for row in range(sh * scale):
            dy = y + row
            if not 0 <= dy < self.h:
                continue
            sy = row // scale
            for col in range(sw * scale):
                dx = x + col
                if not 0 <= dx < self.w:
                    continue
                si = (sy * sw + col // scale) * 4
                di = (dy * self.w + dx) * 4
                self.buf[di:di + 4] = src[si:si + 4]

    def save(self, name):
        image = bpy.data.images.get(name)
        if image:
            bpy.data.images.remove(image)
        image = bpy.data.images.new(name, self.w, self.h, alpha=False)
        image.pixels = self.buf
        image.filepath_raw = os.path.join(OUT, name)
        image.file_format = 'PNG'
        image.save()
        return image.filepath_raw


def read(path):
    image = bpy.data.images.load(path, check_existing=False)
    w, h = image.size
    pixels = list(image.pixels)
    bpy.data.images.remove(image)
    return pixels, w, h


def beside(out_name, left_png, right_png, gap=16):
    """Two images side by side, each scaled to the taller one's height by a whole factor."""
    a, aw, ah = read(left_png)
    b, bw, bh = read(right_png)
    sa = max(1, round(max(ah, bh) / ah))
    sb = max(1, round(max(ah, bh) / bh))
    height = max(ah * sa, bh * sb)
    canvas = Canvas(aw * sa + gap + bw * sb, height)
    canvas.paste(a, aw, ah, 0, (height - ah * sa) // 2, sa)
    canvas.paste(b, bw, bh, aw * sa + gap, (height - bh * sb) // 2, sb)
    return canvas.save(out_name)


# ---------------------------------------------------------------- posing
def bones():
    coll = bpy.data.collections.get(COLL)
    arm = next((o for o in coll.objects if o.type == 'ARMATURE'), None) if coll else None
    return arm, (arm.pose.bones if arm else None)


def pose(pairs):
    """Set pose-bone rotations from (bone, axis, radians) triples; returns a reset callable."""
    arm, pbs = bones()
    for name, axis, angle in pairs:
        pb = pbs[name]
        pb.rotation_mode = 'XYZ'
        rot = [0.0, 0.0, 0.0]
        rot["xyz".index(axis)] = angle
        pb.rotation_euler = Euler(rot, 'XYZ')
    bpy.context.view_layer.update()

    def reset():
        for pb in pbs:
            pb.rotation_mode = 'XYZ'
            pb.rotation_euler = Euler((0, 0, 0), 'XYZ')
            pb.location = (0, 0, 0)
            pb.scale = (1, 1, 1)
        bpy.context.view_layer.update()
    return reset


# ---------------------------------------------------------------- modes
def stage_set(n):
    """The four views a stage boundary is judged on, with the sheet visible behind them."""
    written = []
    for view in ("front", "side", "quarter", "back"):
        written.append(render(f"stage-{n}-{view}.png", view, refs=True))
    written.append(render(f"stage-{n}-front-clean.png", "front", refs=False))
    written.append(render(f"stage-{n}-side-clean.png", "side", refs=False))
    return written


def progress(number, tag, view="quarter", flat="0"):
    res = (560, 560) if view.startswith("head") else (480, 700)
    return [render(os.path.join("progress", f"{number}-{tag}.png"), view,
                   res=res, refs=False, flat=(flat == "flat"))]


def final():
    written = []
    for view in ("front", "side", "back", "quarter", "quarter.L"):
        tag = view.replace(".", "-")
        written.append(render(f"final-{tag}-key.png", view))
        written.append(render(f"final-{tag}-flat.png", view, flat=True))
    written.append(render("face-key.png", "head", (560, 560)))
    written.append(render("face-flat.png", "head", (560, 560), flat=True))
    written.append(render("face-quarter-key.png", "head.q", (560, 560)))

    # beside the frames, matched scale: f088 is the depth authority, f104 the front one
    written.append(beside("vs-frames-f088.png",
                          render("scratch/vs-side.png", "side", (380, 620)),
                          os.path.join(REF_FRAMES, "f088.png")))
    written.append(beside("vs-frames-f104.png",
                          render("scratch/vs-front.png", "front", (380, 620)),
                          os.path.join(REF_FRAMES, "f104.png")))

    # the side profile beside the orthographic sheet it was measured from. The crop spans
    # 154 source px for a figure of 140, so the render is framed to the same 1.320 m.
    written.append(beside("vs-ortho-side-left.png",
                          render("scratch/vs-ortho.png", "side", (600, 770), ortho=H * 154.0 / 140.0),
                          os.path.join(REF_ORTHO, "side-left.png")))
    written.append(beside("vs-ortho-front.png",
                          render("scratch/vs-ortho-f.png", "front", (870, 770), ortho=H * 174.0 / 140.0),
                          os.path.join(REF_ORTHO, "front.png")))

    # readability: the figure at exactly 100 px and 60 px tall, blown up nearest-neighbour
    strips = []
    for px in (100, 60):
        for view in ("front", "quarter"):
            # ortho_scale is the frame's LARGER dimension in metres, so framing exactly H makes
            # the figure exactly `px` pixels tall -- which is what the readability check means.
            strips.append((render(f"scratch/read-{view}-{px}.png", view,
                                  (int(px * 0.75), px), ortho=H), px))
    rows = []
    for path, px in strips:
        pixels, w, h = read(path)
        rows.append((pixels, w, h, px))
    scale = {100: 3, 60: 5}
    width = sum(w * scale[px] + 24 for _, w, _, px in rows) + 24
    height = max(h * scale[px] for _, _, h, px in rows) + 48
    canvas = Canvas(width, height)
    x = 24
    for pixels, w, h, px in rows:
        canvas.paste(pixels, w, h, x, (height - h * scale[px]) // 2, scale[px])
        x += w * scale[px] + 24
    written.append(canvas.save("readability.png"))

    arm, _ = bones()
    if arm is None:
        print("RENDER r11 -- unrigged figure, skipping the pose and deflection sheets")
        return written

    # the carry pose, and one deflection render per joint group
    reset = pose([("shoulder.R", "x", math.radians(-38)), ("elbow.R", "x", math.radians(-52)),
                  ("shoulder.L", "x", math.radians(-14)), ("elbow.L", "x", math.radians(-22)),
                  ("spine", "z", math.radians(6)), ("chest", "z", math.radians(5)),
                  ("neck", "z", math.radians(-6)), ("head", "z", math.radians(-8)),
                  ("hip.R", "x", math.radians(14)), ("knee.R", "x", math.radians(-22)),
                  ("hip.L", "x", math.radians(-10)), ("knee.L", "x", math.radians(16))])
    for view in ("front", "quarter", "side"):
        written.append(render(f"pose-carry-{view.replace('.', '-')}.png", view))
    reset()

    groups = {
        "neck-head":  [("neck", "z", math.radians(28)), ("head", "z", math.radians(40)),
                       ("head", "x", math.radians(-14))],
        "shoulder-elbow": [("shoulder.R", "x", math.radians(-70)), ("elbow.R", "x", math.radians(-75)),
                           ("shoulder.L", "x", math.radians(55)), ("elbow.L", "x", math.radians(-40))],
        "spine-chest": [("spine", "x", math.radians(-26)), ("chest", "x", math.radians(-22)),
                        ("chest", "z", math.radians(22))],
        "hip-knee-foot": [("hip.R", "x", math.radians(55)), ("knee.R", "x", math.radians(-70)),
                          ("foot.R", "x", math.radians(28)), ("hip.L", "x", math.radians(-24))],
        "beard": [("beard", "x", math.radians(-34))],
        "hand": [("hand.R", "x", math.radians(-45)), ("hand.L", "x", math.radians(40))],
    }
    for name, pairs in groups.items():
        reset = pose(pairs)
        view = "side" if name in ("spine-chest", "hip-knee-foot", "beard") else "quarter"
        written.append(render(f"joint-{name}.png", view))
        reset()

    # the GLB must ship in the neutral stance, so prove the pose was put back
    _, pbs = bones()
    moved = [pb.name for pb in pbs if pb.matrix_basis != Matrix.Identity(4)]
    print("  actions in file  %s" % ([a.name for a in bpy.data.actions] or "none"))
    print("  pose bones off rest after restore: %s" % (moved or "none"))
    return written


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else ["final"]
    mode = argv[0] if argv else "final"
    if mode == "stage":
        written = stage_set(argv[1])
    elif mode == "progress":
        written = progress(argv[1], argv[2], *argv[3:5])
    else:
        written = final()
    print("")
    print("RENDER r11 %s -- %d files" % (mode, len(written)))
    for path in written:
        print("   ", os.path.relpath(path, ASSETS).replace("\\", "/"))


if __name__ == "__main__":
    main()
