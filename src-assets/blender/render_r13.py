"""Render every r13 deliverable from the committed .blend, headless.

ONE COMMAND, from the repo root:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/render_r13.py -- <mode> [args]

    <mode>  stage X          the stage-X render set, against the reference planes
            final            every final deliverable: five views, the face and the hands,
                             vs-frames, vs-ortho, vs-r9, readability, the carry pose and
                             the joint deflections
            timelapse        assemble src-assets/renders/r13/live/*.png into timelapse.mp4

Writes src-assets/renders/r13/. Authoring happens in Wolf's running Blender through the MCP
addon (round 13 sec.8); only this and export_dwarf.py are allowed to run headless, and this
script NEVER edits geometry -- it poses, renders, restores, and composites.

Adapted from render_r12.py. Three things are new for round 13:
  * the reference planes are the r13 empties `ref.front` / `ref.side` in collection `ref_r13`;
  * a HAND close-up, because sec.2 defect 4 says the close-ups land on the hands and round 13
    is the round that gave them a wrist, a palm, a thumb and a knuckle block;
  * `vs-r9`, the before/after the brief asks for as deliverable 10 -- round 13 is seeded from
    round 9, so the only honest way to show what the round bought is the same camera and the
    same lighting on both figures. Round 9's finals are already committed under renders/r9/,
    rendered by render_r9.py with these same camera definitions, so they composite directly.

WHAT "FLAT" AND "KEY-LIT" MEAN HERE. Both are Workbench. Flat is `light='FLAT'`, which shows
the atlas albedo with no shading at all -- the honest way to check paint. Key-lit is
`light='STUDIO'`, Blender's fixed three-point studio, deterministic across machines in a way
a hand-placed sun in EEVEE is not. Game lighting is Epic 11's and is not simulated.
"""

import math
import os
import sys

import bpy
from mathutils import Euler, Matrix

REV = "r13"
COLL = f"SM_VoxelDwarf_Miner01_{REV}"
HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.dirname(HERE)
OUT = os.path.join(ASSETS, "renders", REV)
PREV = os.path.join(ASSETS, "renders", "r9")
REF_FRAMES = os.path.join(ASSETS, "references", "dwarf-frames")
REF_ORTHO = os.path.join(ASSETS, "references", "dwarf-ortho")
H = 1.200                       # the figure's height, and the unit every scale below is in

# camera location and XYZ euler in degrees. The dwarf faces +Y, so "front" looks from +Y.
# "side" looks from +X, which is the view side-left.png is drawn in: its image-right is +Y.
# hand.R is aimed down the quarter axis THROUGH the right hand at (0.28, 0.02, 0.39) rather
# than through the figure's centre line, or the hand sits out of frame.
VIEWS = {
    "front":     ((0.0, 3.0, 0.60), (90, 0, 180)),
    "side":      ((3.0, 0.0, 0.60), (90, 0, 90)),
    "back":      ((0.0, -3.0, 0.60), (90, 0, 0)),
    "quarter":   ((2.1, 2.1, 0.60), (90, 0, 135)),
    "quarter.L": ((-2.1, 2.1, 0.60), (90, 0, 225)),
    "head":      ((0.0, 3.0, 1.01), (90, 0, 180)),
    "head.q":    ((1.5, 1.5, 1.01), (90, 0, 135)),
    "hand.R":    ((2.05, 1.79, 0.39), (90, 0, 135)),
    "boot":      ((1.5, 1.5, 0.10), (90, 0, 135)),
}
# ortho width in m; everything else frames the whole figure. head.q is wider than head
# because a 0.40 x 0.36 m head seen corner-on spans ~0.54 m, and a 0.42 frame crops it.
SCALE = {"head": 0.44, "head.q": 0.62, "hand.R": 0.34, "boot": 0.46}


def camera(view):
    # a distinct prefix from the r13cam.* cameras Wolf keeps in the file, so rendering
    # never silently reframes a camera he is navigating with.
    name = "r13rend." + view.replace(".", "_")
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
    for name in ("ref.front", "ref.side"):
        ob = bpy.data.objects.get(name)
        if ob is not None:
            ob.hide_render = not on
            for child in ob.children:
                child.hide_render = not on


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


def final():
    written = []
    for view in ("front", "side", "back", "quarter", "quarter.L"):
        tag = view.replace(".", "-")
        written.append(render(f"final-{tag}-key.png", view))
        written.append(render(f"final-{tag}-flat.png", view, flat=True))

    # the close-ups the round is judged on: sec.2 defects 1-4 all live here
    written.append(render("face-key.png", "head", (560, 560)))
    written.append(render("face-flat.png", "head", (560, 560), flat=True))
    written.append(render("face-quarter-key.png", "head.q", (560, 560)))
    written.append(render("hand-key.png", "hand.R", (560, 560)))
    written.append(render("hand-flat.png", "hand.R", (560, 560), flat=True))
    written.append(render("boot-key.png", "boot", (560, 560)))
    written.append(render("boot-flat.png", "boot", (560, 560), flat=True))

    # beside the frames, matched scale: f088 is the form authority, f104 the front one
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

    # deliverable 10: before/after against round 9, SAME camera, SAME lighting.
    # r13 is seeded from r9, so this is the only picture that shows what the round bought.
    for view in ("front", "side", "quarter"):
        tag = view.replace(".", "-")
        prev = os.path.join(PREV, f"final-{tag}-key.png")
        if os.path.exists(prev):
            written.append(beside(f"vs-r9-{tag}.png", prev,
                                  os.path.join(OUT, f"final-{tag}-key.png")))
        else:
            print(f"RENDER r13 -- no round-9 render at {prev}, skipping vs-r9-{tag}")
    prev_face = os.path.join(PREV, "face-key.png")
    if os.path.exists(prev_face):
        written.append(beside("vs-r9-face.png", prev_face, os.path.join(OUT, "face-key.png")))

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
        print("RENDER r13 -- unrigged figure, skipping the pose and deflection sheets")
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


def timelapse():
    """Assemble the per-operation viewport screenshots into the round's record of itself.

    live/ is gitignored; timelapse.mp4 is the deliverable. Blender's own FFmpeg writer is
    used so the round needs no video tool beyond Blender.
    """
    live = os.path.join(OUT, "live")
    frames = sorted(f for f in os.listdir(live) if f.endswith(".png"))
    if not frames:
        print("RENDER r13 -- no frames in live/, nothing to assemble")
        return []
    scene = bpy.context.scene
    first, w, h = read(os.path.join(live, frames[0]))
    scene.sequence_editor_clear()
    se = scene.sequence_editor_create()
    strip = se.strips.new_image("r13live", os.path.join(live, frames[0]), 1, 1) \
        if hasattr(se, "strips") else se.sequences.new_image("r13live", os.path.join(live, frames[0]), 1, 1)
    for f in frames[1:]:
        strip.elements.append(f)
    scene.frame_start = 1
    scene.frame_end = len(frames)
    # H264 refuses odd dimensions, and a viewport screenshot is whatever size the area is
    # (round 13's was 1559x1332), so round both down to even rather than rescale the frames.
    scene.render.resolution_x, scene.render.resolution_y = w - (w % 2), h - (h % 2)
    scene.render.resolution_percentage = 100
    scene.render.fps = 8
    # Blender 5.x gates FFMPEG behind image_settings.media_type; setting file_format first
    # raises "enum FFMPEG not found", which is what round 13 hit.
    scene.render.image_settings.media_type = 'VIDEO'
    scene.render.image_settings.file_format = 'FFMPEG'
    scene.render.ffmpeg.format = 'MPEG4'
    scene.render.ffmpeg.codec = 'H264'
    scene.render.ffmpeg.constant_rate_factor = 'HIGH'
    scene.render.filepath = os.path.join(OUT, "timelapse.mp4")
    bpy.ops.render.render(animation=True)
    out = os.path.join(OUT, "timelapse.mp4")
    print(f"RENDER r13 timelapse -- {len(frames)} frames at {w}x{h}, 8 fps -> {out}")
    return [out]


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else ["final"]
    mode = argv[0] if argv else "final"
    if mode == "stage":
        written = stage_set(argv[1])
    elif mode == "timelapse":
        written = timelapse()
    else:
        written = final()
    print("")
    print("RENDER r13 %s -- %d files" % (mode, len(written)))
    for path in written:
        print("   ", os.path.relpath(path, ASSETS).replace("\\", "/"))


if __name__ == "__main__":
    main()
