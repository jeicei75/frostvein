"""Round 14 overlay renders -- the figure on the ortho sheet, at matched scale.

Run it in the live Blender right after a build:

    exec(open(PATH_TO_THIS_FILE).read()); overlays("A")

WHAT "MATCHED SCALE" MEANS HERE, EXACTLY. The dwarf-ortho crops are 5x nearest-neighbour
blow-ups of reference-sheet.jpg, so one source pixel is a 5x5 block and one source pixel is
8.571 mm. Rendering at 5 * (1/8.571 mm) = 583.33 px/m and framing the camera on the sheet's
own centre puts our figure on the sheet's figure pixel for pixel -- no eyeballing, no scaling
of either image.

The sheet pins the mapping for two of the four views and the model sheet states the pins:
rows 7 (crown) .. 147 (sole) in front.png and side-left.png, front column 77 is the centre
line, side-left column 58 is the depth centre. back.png is drawn SMALLER -- the model sheet
says its figure is 123 px, not 140 -- and side-right.png is unpinned, so for those two the
mapping is derived by scanning the sheet for the figure and is PRINTED, not assumed.

HANDEDNESS, because getting it wrong silently mirrors the overlay. A camera looking down -Y
with +Z up has its right at -X, so world +X lands on image LEFT -- which is what front.png
does (the figure's left hand, holding the lantern, is drawn on the image right). Viewed from
behind, the figure's right is on the image right. Both signs are in VIEWS below.

This script never edits geometry. It adds cameras named r14cam.* and writes PNGs.
"""

import os
import math

import bpy

REV = "r14"
COLL = "SM_VoxelDwarf_Miner01_" + REV
HERE = os.path.dirname(os.path.abspath(bpy.data.filepath)) if bpy.data.filepath else ""
ASSETS = r"D:\Workspace\frostvein\src-assets"
REF = os.path.join(ASSETS, "references", "dwarf-ortho")
FRAMES = os.path.join(ASSETS, "references", "dwarf-frames")
OUT = os.path.join(ASSETS, "renders", REV)

H = 1.200
PX = H / 140.0                 # one source pixel, metres

# Mirrors of dwarf_r14's paint tables, so the classifier is self-contained and this script can
# run against a .blend without the generator loaded. They must stay in step with it.
PALETTE_SRC = {
    "skin": "#E9D2BB", "beard": "#5E4632", "snow": "#FFFFFF", "tunic": "#5F7A6A",
    "pants": "#474B41", "metal": "#A9B2AC", "wood": "#8B6B50", "trunk": "#6B5B49",
    "hair": "#34271C", "flame": "#F0A63C",
}
EXTRA_SRC = {"dirt": "#493E32", "skinhi": "#F2DFCB"}
STEPS_SRC = {"top": 1.18, "front": 1.00, "side": 0.85, "bottom": 0.66}
SRC_SCALE = 5                  # the crops are 5x blow-ups
PPM = SRC_SCALE / PX           # 583.333 image px per metre, for the pinned views

# loc gives the two fixed camera axes; `sign` maps image-x to a world axis.
VIEWS = {
    "front":      dict(png="front.png",      dist=(0.0, 3.0),  rot=(90, 0, 180),
                       axis="x", sign=-1, centre_col=77, pinned=True),
    "side-left":  dict(png="side-left.png",  dist=(3.0, 0.0),  rot=(90, 0, 90),
                       axis="y", sign=+1, centre_col=58, pinned=True),
    "side-right": dict(png="side-right.png", dist=(-3.0, 0.0), rot=(90, 0, 270),
                       axis="y", sign=-1, centre_col=None, pinned=False),
    "back":       dict(png="back.png",       dist=(0.0, -3.0), rot=(90, 0, 0),
                       axis="x", sign=+1, centre_col=None, pinned=False),
}


# ------------------------------------------------------------------ image helpers
def load(path):
    img = bpy.data.images.load(path, check_existing=False)
    w, h = img.size
    px = list(img.pixels)
    bpy.data.images.remove(img)
    return px, w, h


def save(name, px, w, h):
    old = bpy.data.images.get(name)
    if old:
        bpy.data.images.remove(old)
    img = bpy.data.images.new(name, w, h, alpha=False)
    img.pixels = px
    os.makedirs(OUT, exist_ok=True)
    img.filepath_raw = os.path.join(OUT, name)
    img.file_format = 'PNG'
    img.save()
    bpy.data.images.remove(img)
    return os.path.join(OUT, name)


def is_ink(r, g, b):
    """Graph paper is light and near-neutral; the drawing is darker or coloured."""
    return (max(r, g, b) - min(r, g, b)) > 0.10 or max(r, g, b) < 0.62


def scan(px, w, h):
    """Find the FIGURE on a sheet crop, not the sheet's furniture.

    The crops carry dimension arrows along the top and, on back.png, a printed caption at
    the bottom. Both are ink, so a plain bounding box of all ink is wrong -- the first pass
    of this round measured side-right's figure as 147.8 source pixels when it is 140. Arrows
    and captions are THIN; the figure is a tall run of WIDE rows. So: keep only rows whose
    ink spans at least 12 % of the width, then take the longest contiguous run of them.
    """
    rows = {}
    for y in range(h):
        iy = h - 1 - y                      # blender rows are bottom-up
        run = [x for x in range(w) if is_ink(px[(y * w + x) * 4],
                                             px[(y * w + x) * 4 + 1],
                                             px[(y * w + x) * 4 + 2])]
        if run and (run[-1] - run[0]) >= w * 0.12:
            rows[iy] = (run[0], run[-1])

    best = cur = []
    for iy in sorted(rows):
        if cur and iy == cur[-1] + 1:
            cur.append(iy)
        else:
            cur = [iy]
        if len(cur) > len(best):
            best = list(cur)

    y0, y1 = best[0], best[-1]
    head = [rows[iy] for iy in best[:max(1, int(len(best) * 0.22))]]
    cx = (min(a for a, _ in head) + max(b for _, b in head)) / 2.0
    xs = [rows[iy] for iy in best]
    return dict(x0=min(a for a, _ in xs), x1=max(b for _, b in xs),
                y0=y0, y1=y1, head_cx=cx, rows=rows)


# ------------------------------------------------------------------ camera framing
def frame(view):
    """Return (ortho_scale, cam_h, cam_z, res) putting our figure on the sheet's figure."""
    cfg = VIEWS[view]
    px, w, h = load(os.path.join(REF, cfg["png"]))
    if cfg["pinned"]:
        ppm = PPM
        src_w, src_h = w / SRC_SCALE, h / SRC_SCALE
        cc = (src_w - 1) / 2.0
        rc = (src_h - 1) / 2.0
        cam_h = cfg["sign"] * (cc - cfg["centre_col"]) * PX
        cam_z = (147.0 - rc) * PX
        note = "pinned rows 7/147, centre col %d" % cfg["centre_col"]
    else:
        s = scan(px, w, h)
        ppm = (s["y1"] - s["y0"]) / H
        cam_h = cfg["sign"] * ((w - 1) / 2.0 - s["head_cx"]) / ppm
        cam_z = (s["y1"] - (h - 1) / 2.0) / ppm
        note = ("scanned: figure rows %d..%d = %.1f px = %.1f px/m (%.1f source px), "
                "head centre col %.1f" % (s["y0"], s["y1"], s["y1"] - s["y0"], ppm,
                                          (s["y1"] - s["y0"]) / SRC_SCALE, s["head_cx"]))
    ortho = max(w, h) / ppm
    return ortho, cam_h, cam_z, (w, h), ppm, note


def camera(view, ortho, cam_h, cam_z):
    cfg = VIEWS[view]
    name = "r14cam." + view
    ob = bpy.data.objects.get(name)
    if ob is None:
        ob = bpy.data.objects.new(name, bpy.data.cameras.new(name))
        bpy.context.scene.collection.objects.link(ob)
    ob.data.type = 'ORTHO'
    ob.data.ortho_scale = ortho
    dx, dy = cfg["dist"]
    ob.location = (cam_h if cfg["axis"] == "x" else dx,
                   cam_h if cfg["axis"] == "y" else dy,
                   cam_z)
    ob.rotation_euler = [math.radians(a) for a in cfg["rot"]]
    return ob


def render(view, path, res, transparent=True, flat=True, textured=False):
    sc = bpy.context.scene
    sc.render.engine = 'BLENDER_WORKBENCH'
    sc.render.resolution_x, sc.render.resolution_y = res
    sc.render.resolution_percentage = 100
    sc.render.film_transparent = transparent
    sh = sc.display.shading
    sh.light = 'FLAT' if flat else 'STUDIO'
    sh.color_type = 'TEXTURE' if textured else 'SINGLE'
    sh.single_color = (0.62, 0.62, 0.64)
    sh.show_cavity = False
    sh.show_backface_culling = textured
    sc.render.image_settings.file_format = 'PNG'
    sc.render.image_settings.color_mode = 'RGBA'
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)
    return path


# ------------------------------------------------------------------ the overlay
def composite(name, sheet, ours, w, h):
    """Sheet underneath; our silhouette as a 50 % magenta wash with a solid edge."""
    out = [0.0] * (w * h * 4)
    alpha = [ours[i * 4 + 3] > 0.35 for i in range(w * h)]
    for i in range(w * h):
        j = i * 4
        r, g, b = sheet[j], sheet[j + 1], sheet[j + 2]
        if alpha[i]:
            x, y = i % w, i // w
            edge = False
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if not (0 <= nx < w and 0 <= ny < h) or not alpha[ny * w + nx]:
                    edge = True
                    break
            if edge:
                r, g, b = 1.0, 0.0, 0.62
            else:
                r = r * 0.5 + 0.90 * 0.5
                g = g * 0.5 + 0.10 * 0.5
                b = b * 0.5 + 0.70 * 0.5
        out[j], out[j + 1], out[j + 2], out[j + 3] = r, g, b, 1.0
    return save(name, out, w, h)


def overlays(stage="A"):
    os.makedirs(OUT, exist_ok=True)
    scratch = os.path.join(OUT, "scratch")
    os.makedirs(scratch, exist_ok=True)
    written = []
    print("RENDER r14 overlays, stage %s" % stage)
    for view, cfg in VIEWS.items():
        ortho, cam_h, cam_z, res, ppm, note = frame(view)
        cam = camera(view, ortho, cam_h, cam_z)
        bpy.context.scene.camera = cam
        raw = os.path.join(scratch, "r14-%s.png" % view)
        render(view, raw, res, textured=(stage == "B"))
        ours, w, h = load(raw)
        sheet, sw, sh_ = load(os.path.join(REF, cfg["png"]))
        assert (w, h) == (sw, sh_), "%s: render %dx%d vs sheet %dx%d" % (view, w, h, sw, sh_)
        path = composite("overlay-%s-%s.png" % (stage, view), sheet, ours, w, h)
        written.append(path)
        print("  %-11s %dx%d  ortho %.4f m  cam h %+.4f z %+.4f  %.2f px/m" %
              (view, w, h, ortho, cam_h, cam_z, ppm))
        print("              %s" % note)
        print("              -> %s" % os.path.relpath(path, ASSETS).replace("\\", "/"))
    return written


def boxes_of(ob):
    """Every part is built from 8-vertex boxes, added in order, so they regroup exactly."""
    vs = [ob.matrix_world @ v.co for v in ob.data.vertices]
    out = []
    for i in range(0, len(vs), 8):
        g = vs[i:i + 8]
        out.append([(min(v[a] for v in g), max(v[a] for v in g)) for a in range(3)])
    return out


def span(coll, picks, axis):
    """Extent along `axis` of the named (object, box index) pairs. Exact, no band filter."""
    a = {"x": 0, "y": 1, "z": 2}[axis]
    lo, hi = 1e9, -1e9
    for name, idx in picks:
        bs = boxes_of(coll.objects[name])
        for i in ([idx] if isinstance(idx, int) else idx):
            lo = min(lo, bs[i][a][0])
            hi = max(hi, bs[i][a][1])
    return hi - lo


def beside(name, left_png, right_png, gap=14):
    """Two images side by side, each whole-number scaled to the taller one's height."""
    a, aw, ah = load(left_png)
    b, bw, bh = load(right_png)
    sa = max(1, round(max(ah, bh) / ah))
    sb = max(1, round(max(ah, bh) / bh))
    hh = max(ah * sa, bh * sb)
    ww = aw * sa + gap + bw * sb
    out = [0.10, 0.10, 0.11, 1.0] * (ww * hh)

    def paste(src, sw, sh_, ox, oy, s):
        for r in range(sh_ * s):
            dy = oy + r
            if not 0 <= dy < hh:
                continue
            for c in range(sw * s):
                dx = ox + c
                if not 0 <= dx < ww:
                    continue
                si = ((r // s) * sw + c // s) * 4
                di = (dy * ww + dx) * 4
                al = src[si + 3]
                for k in range(3):
                    out[di + k] = src[si + k] * al + out[di + k] * (1 - al)
                out[di + 3] = 1.0

    paste(a, aw, ah, 0, (hh - ah * sa) // 2, sa)
    paste(b, bw, bh, aw * sa + gap, (hh - bh * sb) // 2, sb)
    return save(name, out, ww, hh)


def vs_frames(stage="A"):
    """Our figure beside the two turntable frames the brief judges form and face against.

    The frames are lit, perspective and posed, so "matched scale" can only mean matched
    FIGURE HEIGHT: the dwarf stands about 500 px tall in a 620 px frame, so our ortho frame
    is 620 px for 1.488 m and the figure lands at the same 500 px.
    """
    scratch = os.path.join(OUT, "scratch")
    os.makedirs(scratch, exist_ok=True)
    # wide enough for the A-pose: the arms span 1.164 m, which is 485 px at this scale, so
    # a 380 px frame (the width of the reference frames) crops the hands off.
    res = (560, 620)
    ortho = H * 620.0 / 500.0
    written = []
    # THREE-QUARTER, not dead-on. An orthographic front view of an axis-aligned box model is
    # flat by construction: every forward-facing box shares one normal, so studio light gives
    # one grey and no form at all to judge. f104 and f088 are three-quarter views; these match
    # them. f104 is front-right of the figure, f088 behind-left.
    quarters = {
        "q.front": ((2.1, 2.1, 0.60), (90, 0, 135)),
        "q.back": ((-2.1, -2.1, 0.60), (90, 0, -45)),
    }
    for view, frame_png, tag in (("q.front", "f104.png", "f104"), ("q.back", "f088.png", "f088")):
        loc, rot = quarters[view]
        name = "r14cam." + view
        cam = bpy.data.objects.get(name)
        if cam is None:
            cam = bpy.data.objects.new(name, bpy.data.cameras.new(name))
            bpy.context.scene.collection.objects.link(cam)
        cam.data.type = 'ORTHO'
        cam.data.ortho_scale = ortho
        cam.location = loc
        cam.rotation_euler = [math.radians(a) for a in rot]
        bpy.context.scene.camera = cam
        ours = os.path.join(scratch, "vs-%s.png" % tag)
        render(view, ours, res, flat=False, textured=True)   # lit AND painted, or neither
                                                             # the form nor the paint compares
        written.append(beside("vs-frames-%s.png" % tag, ours, os.path.join(FRAMES, frame_png)))
        print("  vs %s -> %s" % (tag, os.path.relpath(written[-1], ASSETS).replace("\\", "/")))
    return written


def skin_share():
    """Visible skin as a share of figure pixels, classified to the NEAREST palette cell.

    Nearest-cell, not exact match, because the flat pass is antialiased: every cell smears
    across dozens of near-duplicate values and an exact match counts only each region's pure
    core. The model sheet records this exact correction -- round 4 reported 7.4 % and failed
    this check when the true figure was 13.9 %; the model was never short of skin, the
    instrument was.
    """
    cells = {}
    for name, hexv in list(PALETTE_SRC.items()) + list(EXTRA_SRC.items()):
        h = hexv.lstrip("#")
        rgb = [int(h[i:i + 2], 16) / 255.0 for i in (0, 2, 4)]
        lin = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in rgb]
        for step, mul in STEPS_SRC.items():
            cells["%s.%s" % (name, step)] = [min(1.0, v * mul) for v in lin]

    scratch = os.path.join(OUT, "scratch")
    os.makedirs(scratch, exist_ok=True)
    out = {}
    for view in ("front", "side-left"):
        ortho, cam_h, cam_z, size, _, _ = frame(view)
        cam = camera(view, ortho, cam_h, cam_z)
        bpy.context.scene.camera = cam
        p = os.path.join(scratch, "skin-%s.png" % view)
        render(view, p, size, transparent=True, flat=True, textured=True)
        px, w, h = load(p)
        tally, total = {}, 0
        for i in range(w * h):
            j = i * 4
            if px[j + 3] <= 0.35:
                continue
            total += 1
            best, who = 1e9, None
            for key, c in cells.items():
                d = ((px[j] - c[0]) ** 2 + (px[j + 1] - c[1]) ** 2 + (px[j + 2] - c[2]) ** 2)
                if d < best:
                    best, who = d, key
            base = who.rsplit(".", 1)[0]
            tally[base] = tally.get(base, 0) + 1
        skin = tally.get("skin", 0) + tally.get("skinhi", 0)
        share = skin / total if total else 0.0
        out[view] = share
        top = sorted(tally.items(), key=lambda kv: -kv[1])[:5]
        print("  visible skin %-10s %5.1f %% of %d figure px   target >= 10 %%   %s" %
              (view, share * 100, total, "OK" if share >= 0.10 else "OFF"))
        print("              families: %s" %
              ", ".join("%s %.1f%%" % (k, v / total * 100) for k, v in top))
    return out


def bones():
    coll = bpy.data.collections.get(COLL)
    arm = next((o for o in coll.objects if o.type == 'ARMATURE'), None) if coll else None
    return arm, (arm.pose.bones if arm else None)


def pose(pairs):
    """Set pose-bone rotations from (bone, axis, degrees); returns a reset callable."""
    from mathutils import Euler
    arm, pbs = bones()
    for name, axis, deg in pairs:
        pb = pbs[name]
        pb.rotation_mode = 'XYZ'
        rot = [0.0, 0.0, 0.0]
        rot["xyz".index(axis)] = math.radians(deg)
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


def quarter_cam(ortho=1.45, z=0.60):
    name = "r14cam.q.front"
    cam = bpy.data.objects.get(name)
    if cam is None:
        cam = bpy.data.objects.new(name, bpy.data.cameras.new(name))
        bpy.context.scene.collection.objects.link(cam)
    cam.data.type = 'ORTHO'
    cam.data.ortho_scale = ortho
    cam.location = (2.1, 2.1, z)
    cam.rotation_euler = [math.radians(a) for a in (90, 0, 135)]
    return cam


def rig_shots():
    """One deflection render per joint group, plus the two-handed carry, then prove the reset."""
    from mathutils import Matrix
    arm, pbs = bones()
    if arm is None:
        print("RIG SHOTS -- no armature, skipping")
        return []
    written = []
    res = (560, 700)

    carry = [("shoulder.R", "x", -38), ("elbow.R", "x", -52),
             ("shoulder.L", "x", -14), ("elbow.L", "x", -22),
             ("spine", "z", 6), ("chest", "z", 5), ("neck", "z", -6), ("head", "z", -8),
             ("hip.R", "x", 14), ("knee.R", "x", -22), ("hip.L", "x", -10), ("knee.L", "x", 16)]
    reset = pose(carry)
    for view, cam in (("front", camera("front", 1.45, 0.0, 0.60)), ("quarter", quarter_cam())):
        bpy.context.scene.camera = cam
        p = os.path.join(OUT, "pose-carry-%s.png" % view)
        render("front", p, res, transparent=False, flat=False, textured=True)
        written.append(p)
    reset()

    groups = {
        "neck-head": [("neck", "z", 28), ("head", "z", 40), ("head", "x", -14)],
        "shoulder-elbow": [("shoulder.R", "x", -70), ("elbow.R", "x", -75),
                           ("shoulder.L", "x", 55), ("elbow.L", "x", -40)],
        "spine-chest": [("spine", "x", -26), ("chest", "x", -22), ("chest", "z", 22)],
        "hip-knee-foot": [("hip.R", "x", 55), ("knee.R", "x", -70), ("foot.R", "x", 28),
                          ("hip.L", "x", -24)],
        "beard": [("beard", "x", -34)],
        "hand": [("hand.R", "x", -45), ("hand.L", "x", 40)],
    }
    for name, pairs in groups.items():
        reset = pose(pairs)
        cam = (camera("side-left", 1.45, 0.0, 0.60)
               if name in ("spine-chest", "hip-knee-foot", "beard") else quarter_cam())
        bpy.context.scene.camera = cam
        p = os.path.join(OUT, "joint-%s.png" % name)
        render("side-left", p, res, transparent=False, flat=False, textured=True)
        written.append(p)
        reset()

    moved = [pb.name for pb in pbs if pb.matrix_basis != Matrix.Identity(4)]
    print("  actions in file: %s" % ([a.name for a in bpy.data.actions] or "none"))
    print("  pose bones off rest after restore: %s" % (moved or "none"))
    for p in written:
        print("  %s" % os.path.relpath(p, ASSETS).replace("\\", "/"))
    return written


def face_shots():
    """The face close up, and the readability strip the round is judged on at 60 px.

    `readability.png` puts the figure at EXACTLY 100 px and 60 px tall and then blows those
    up nearest-neighbour. Framing the camera to H makes the figure exactly `px` tall, so the
    number on the strip is the number being judged, not an approximation of it.
    """
    scratch = os.path.join(OUT, "scratch")
    os.makedirs(scratch, exist_ok=True)
    written = []
    head_z = 1.200 - 0.176
    for tag, ortho, flat in (("flat", 0.46, True), ("key", 0.46, False)):
        cam = camera("front", ortho, 0.0, head_z)
        cam.data.ortho_scale = ortho
        bpy.context.scene.camera = cam
        p = os.path.join(OUT, "face-%s.png" % tag)
        render("front", p, (560, 560), transparent=False, flat=flat, textured=True)
        written.append(p)
    cam = camera("front", 0.62, 0.0, head_z)
    cam.data.ortho_scale = 0.62
    bpy.context.scene.camera = cam

    # the strip: the figure at 100 px and 60 px tall, nearest-neighbour blown up
    strips = []
    for px_tall in (100, 60):
        cam = camera("front", H, 0.0, H / 2.0)
        cam.data.ortho_scale = H
        bpy.context.scene.camera = cam
        p = os.path.join(scratch, "read-%d.png" % px_tall)
        # FLAT, so the strip judges albedo. The model sheet's readability and palette checks
        # are all specified on the flat pass; a studio-lit strip grades the lighting instead.
        render("front", p, (int(px_tall * 0.95), px_tall), transparent=False,
               flat=True, textured=True)
        strips.append((load(p), px_tall))
    scale = {100: 4, 60: 6}
    ww = sum(w * scale[t] + 24 for (_, w, _), t in strips) + 24
    hh = max(h * scale[t] for (_, _, h), t in strips) + 48
    out = [0.10, 0.10, 0.11, 1.0] * (ww * hh)
    x = 24
    for (pxs, w, h), t in strips:
        s = scale[t]
        oy = (hh - h * s) // 2
        for r in range(h * s):
            for c in range(w * s):
                si = ((r // s) * w + c // s) * 4
                di = ((oy + r) * ww + x + c) * 4
                out[di:di + 4] = pxs[si:si + 3] + [1.0]
        x += w * s + 24
    written.append(save("readability.png", out, ww, hh))
    for p in written:
        print("  %s" % os.path.relpath(p, ASSETS).replace("\\", "/"))
    return written


def measure():
    """Our figure's own landmarks, in sheet units, so they compare with the table directly."""
    coll = bpy.data.collections[COLL]
    print("MEASURE r14 -- our figure against the sheet's table")
    lo = [1e9] * 3
    hi = [-1e9] * 3
    for ob in coll.objects:
        if ob.type != 'MESH' or ob.name in ("r14_pickaxe", "r14_lantern"):
            continue
        for v in ob.data.vertices:
            w = ob.matrix_world @ v.co
            for i in range(3):
                lo[i] = min(lo[i], w[i])
                hi[i] = max(hi[i], w[i])
    print("  body only (no props): height %.4f m = %.3f H, width %.4f = %.3f H, "
          "depth %.4f = %.3f H" % (hi[2] - lo[2], (hi[2] - lo[2]) / H,
                                   hi[0] - lo[0], (hi[0] - lo[0]) / H,
                                   hi[1] - lo[1], (hi[1] - lo[1]) / H))
    # Each sheet number was read off ONE feature, so each check names the box that feature is.
    # Box order follows the build: head 0 skull 1 jaw 2 brow 3-4 nose 5-6 ears 7 neck;
    # hair 0 cap 1-4 crown steps 5 back 6-7 lobes; torso 0 collar 1 chest 2 waist;
    # skirt 0 top 1 mid 2 hem; sleeve 0 cap 1 upper 2 fore; boot 0 sole 1 body 2 toe 3 cuff.
    checks = [
        ("head with hair, width", [("r14_hair", 0)], "x", 0.336),
        ("ear to ear, width", [("r14_head", 5), ("r14_head", 6)], "x", 0.383),
        ("crown step 1, width", [("r14_hair", 2)], "x", 0.286),
        ("crown step 2, width", [("r14_hair", 3)], "x", 0.229),
        ("crown top, width", [("r14_hair", 4)], "x", 0.164),
        ("beard widest, width", [("r14_beard", 1), ("r14_beard", 2)], "x", 0.343),
        ("chest, tunic only", [("r14_torso", 1)], "x", 0.317),
        ("shoulders over the caps", [("r14_sleeve.R", 0), ("r14_sleeve.L", 0)], "x", 0.528),
        ("skirt at the hem", [("r14_skirt", 2)], "x", 0.439),
        ("stance, boot to boot", [("r14_boot.R", 1), ("r14_boot.L", 1)], "x", 0.398),
        ("one boot, width", [("r14_boot.R", 1)], "x", 0.164),
        ("boot cuff, width", [("r14_boot.R", 3)], "x", 0.200),
        # The sheet's 0.064 H neck is NOT checked here. It is the depth of the exposed skin
        # column, which is a property of what the jaw and hair leave uncovered, not of the
        # neck box -- measuring the box against it made a correct full-depth throat report
        # +12.97 px. neck_bare() measures the exposed column itself and reports it.
        # 0.336 runs from the back of the HAIR to the front of the face, so it spans both
        # objects; the skull's own back plane sits 8 mm forward of it to avoid z-fighting
        ("head depth, back to face", [("r14_hair", 0), ("r14_head", 0)], "y", 0.336),
        ("torso depth", [("r14_torso", 1)], "y", 0.286),
        ("skirt depth", [("r14_skirt", 2)], "y", 0.300),
        ("pack behind the torso", [("r14_pack", 0), ("r14_pack", 1)], "y", 0.186),
        ("shin depth", [("r14_leg.R", 0)], "y", 0.164),
        ("boot sole length", [("r14_boot.R", 0)], "y", 0.214),
        ("pack to nose, depth", None, "y", 0.550),
    ]
    heights = [
        ("crown", "r14_hair", 4, 1, 1.000),
        ("main skull top", "r14_head", 0, 1, 0.943),
        ("head + hair mass ends", "r14_hair", 5, 0, 0.707),
        ("shoulder line", "r14_sleeve.R", 0, 1, 0.700),
        ("beard tip", "r14_beard", 4, 0, 0.464),
        ("belt top", "r14_belt", 0, 1, 0.421),
        ("belt bottom", "r14_belt", 0, 0, 0.343),
        ("tunic hem", "r14_skirt", 2, 0, 0.207),
        ("boot cuff top", "r14_boot.R", 3, 1, 0.164),
        ("boot cuff bottom", "r14_boot.R", 3, 0, 0.107),
        ("sole", "r14_boot.R", 0, 0, 0.000),
    ]
    worst = 0.0
    for label, picks, axis, target in checks:
        if picks is None:
            picks = [(o.name, list(range(len(boxes_of(o))))) for o in coll.objects
                     if o.type == 'MESH' and o.name not in ("r14_pickaxe", "r14_lantern")]
        got = span(coll, picks, axis) / H
        d_px = (got - target) * H / PX
        worst = max(worst, abs(d_px))
        print("  %-26s %.3f H   sheet %.3f H   %+5.2f px  %s" %
              (label, got, target, d_px, "OK" if abs(d_px) <= 1.0 else "OFF"))
    print("  -- heights, z/H from the sole --")
    for label, name, idx, end, target in heights:
        got = boxes_of(coll.objects[name])[idx][2][end] / H
        d_px = (got - target) * H / PX
        worst = max(worst, abs(d_px))
        print("  %-26s %.3f H   sheet %.3f H   %+5.2f px  %s" %
              (label, got, target, d_px, "OK" if abs(d_px) <= 1.0 else "OFF"))
    print("  worst deviation %.2f source px (gate: 1.00 = %.1f mm)" % (worst, PX * 1000))
    normals()


def normals():
    """Every face must point away from its own box. Catches inside-out mass (sec.8).

    Worth its own check: the arm and glove boxes were built on a left-handed cross-section
    frame for eight builds. The geometry measured correct in every dimension and every face
    was wound inside-out, which no silhouette or landmark test can see.
    """
    coll = bpy.data.collections[COLL]
    bad = []
    for ob in coll.objects:
        if ob.type != 'MESH':
            continue
        me = ob.data
        for poly in me.polygons:
            bi = poly.index // 6
            vs = [ob.matrix_world @ me.vertices[i].co for i in range(bi * 8, bi * 8 + 8)]
            c = [sum(v[a] for v in vs) / 8.0 for a in range(3)]
            n = poly.normal
            pc = ob.matrix_world @ poly.center
            if (n.x * (pc.x - c[0]) + n.y * (pc.y - c[1]) + n.z * (pc.z - c[2])) <= 0:
                bad.append("%s box %d face %d" % (ob.name, bi, poly.index % 6))
    print("  inside-out faces: %s" % (("%d -- %s" % (len(bad), ", ".join(bad[:8])))
                                      if bad else "none"))
    back_skin()
    return bad


def back_skin(step=0.004):
    """Head skin must never be the rearmost surface above the shoulders.

    The head is behind the hair by construction, so any (x, z) where a head box is rearmost
    means skin is showing through the back of the hair. That is how an 8 mm skull margin --
    1 mm proud of the shallowest hair band -- put a bar of skin across the back of the head
    while every dimensional check still read 0.00 px.
    """
    coll = bpy.data.collections[COLL]
    allb = []
    for ob in coll.objects:
        if ob.type != 'MESH':
            continue
        for i, b in enumerate(boxes_of(ob)):
            allb.append((ob.name, i, b))
    hair = [b for n, _, b in allb if n == "r14_hair"]
    xlo = min(b[0][0] for b in hair)
    xhi = max(b[0][1] for b in hair)
    zlo = min(b[2][0] for b in hair)
    zhi = max(b[2][1] for b in hair)

    hits = []
    z = zlo
    while z <= zhi:
        x = xlo
        while x <= xhi:
            besty, who = 1e9, None
            for n, i, b in allb:
                if b[0][0] <= x <= b[0][1] and b[2][0] <= z <= b[2][1] and b[1][0] < besty:
                    besty, who = b[1][0], (n, i)
            if who and who[0] == "r14_head":
                hits.append((who[1], round(x, 3), round(z, 3)))
            x += step
        z += step
    boxes = sorted({h[0] for h in hits})
    print("  skin showing behind the hair: %s" %
          (("%d samples, head boxes %s" % (len(hits), boxes)) if hits else "none"))
    return hits


# ------------------------------------------------------- the two sec.2 reported checks
def neck_bare(step=0.0015):
    """Longest run of rows where the NECK is the outermost surface, in each side view.

    Computed on the geometry, not on a render, because at stage A every part is the same
    grey and a render cannot tell neck from jaw. An orthographic side view shows, at each
    (y, z), the surface with the greatest |x|; the neck is bare exactly where that surface
    is the neck box. The neck is box 7 of r14_head.
    """
    coll = bpy.data.collections[COLL]
    allb = []
    for ob in coll.objects:
        if ob.type != 'MESH':
            continue
        for i, b in enumerate(boxes_of(ob)):
            allb.append((ob.name, i, b))
    neck = [b for n, i, b in allb if n == "r14_head" and i == 7][0]

    out = {}
    for label, sign in (("side-left (+X)", 1), ("side-right (-X)", -1)):
        rows = []
        z = neck[2][0] - 0.05
        while z <= neck[2][1] + 0.05:
            bare = False
            y = neck[1][0]
            while y <= neck[1][1]:
                best, who = -1e9, None
                for n, i, b in allb:
                    if b[1][0] <= y <= b[1][1] and b[2][0] <= z <= b[2][1]:
                        v = sign * b[0][1] if sign > 0 else -b[0][0]
                        if v > best:
                            best, who = v, (n, i)
                if who == ("r14_head", 7):
                    bare = True
                    break
                y += step * 4
            rows.append((z, bare))
            z += step
        best = cur = 0
        for _, bare in rows:
            cur = cur + 1 if bare else 0
            best = max(best, cur)
        out[label] = best * step

        # and how DEEP the exposed column is, which is what the sheet's "up to 0.064 H wide"
        # actually measures on side-left.png. Sampled at the middle of the bare run.
        zmid = None
        run = 0
        for z, bare in rows:
            run = run + 1 if bare else 0
            if run * step >= best * step / 2 and zmid is None and bare:
                zmid = z
        depth = 0.0
        if zmid is not None:
            y = neck[1][0]
            while y <= neck[1][1]:
                bestx, who = -1e9, None
                for n, i, b in allb:
                    if b[1][0] <= y <= b[1][1] and b[2][0] <= zmid <= b[2][1]:
                        v = sign * b[0][1] if sign > 0 else -b[0][0]
                        if v > bestx:
                            bestx, who = v, (n, i)
                if who == ("r14_head", 7):
                    depth += step
                y += step
        print("  neck bare, %-16s %.4f m = %.3f H tall   column %.3f H deep "
              "(sheet 0.064)   %s" %
              (label, best * step, best * step / H, depth / H,
               "OK" if best * step / H >= 0.080 else "OFF"))
    return out


def step_density():
    """Silhouette steps per 100 rows on front and side, and the left/right balance.

    COUNTED AT THE SHEET'S OWN RESOLUTION, which is the only basis on which our number and
    the reference's mean the same thing. "Steps per 100 rows" is not resolution-independent:
    a diagonal edge yields one step per row at any scale, so a 5x render of the same shape
    reports a fifth the density. The model sheet's published 10.3 (side) and 11.2 (front)
    were counted on its own ~335-row render, not on the 140-row source -- its raw counts are
    side L33/R36 and front L37/R38, which on the 140-row source are 24.6 and 26.8. Those are
    the targets used here, and our render is sampled down to source pixels to match.
    """
    scratch = os.path.join(OUT, "scratch")
    os.makedirs(scratch, exist_ok=True)
    res = {}
    for view, raw_lr in (("front", (37, 38)), ("side-left", (33, 36))):
        ortho, cam_h, cam_z, size, ppm, _ = frame(view)
        cam = camera(view, ortho, cam_h, cam_z)
        bpy.context.scene.camera = cam
        raw = os.path.join(scratch, "steps-%s.png" % view)
        render(view, raw, size)
        px, w, h = load(raw)
        sw, sh_ = w // SRC_SCALE, h // SRC_SCALE
        edges = []
        for sy in range(sh_):
            y = sy * SRC_SCALE + SRC_SCALE // 2
            run = [sx for sx in range(sw)
                   if px[(y * w + sx * SRC_SCALE + SRC_SCALE // 2) * 4 + 3] > 0.35]
            if run:
                edges.append((sh_ - 1 - sy, run[0], run[-1]))
        if not edges:
            continue
        edges.sort()
        tall = edges[-1][0] - edges[0][0]
        left = sum(1 for a, b in zip(edges, edges[1:]) if a[1] != b[1])
        right = sum(1 for a, b in zip(edges, edges[1:]) if a[2] != b[2])
        per100 = (left + right) / 2.0 / tall * 100.0
        ref100 = sum(raw_lr) / 2.0 / 140.0 * 100.0
        bal = min(left, right) / max(left, right) if max(left, right) else 0.0
        res[view] = (per100, left, right, bal)
        print("  steps %-10s %5.1f /100 src rows (reference %.1f, its L%d/R%d)   "
              "ours L %d / R %d over %d rows   balance %.2f   %s" %
              (view, per100, ref100, raw_lr[0], raw_lr[1], left, right, tall, bal,
               "OK" if per100 >= 10.0 and bal >= 0.70 else "OFF"))
    return res
