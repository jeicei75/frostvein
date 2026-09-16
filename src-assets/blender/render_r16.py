"""Round 16 renders and the envelope gate.

    blender --background --factory-startup --python src-assets/blender/render_r16.py

THE CAMERA IS PINNED TO THE CROP, not to the figure. Each ortho view is set up so
the rendered image is the SAME SIZE as the sheet crop it is judged against --
870x770 for front.png, 600x770 for side-left.png -- with the same 5x block grid. So
a rendered pixel and a source pixel are the same thing, overlays need no resampling,
and r15 s7.8's warning (silhouette density is not resolution independent) is moot
because nothing is ever counted at another scale.

Our silhouette is sampled at the CENTRE of each 5x5 block, exactly as sheet_r16.py
samples the art. That is also r15 s7.4's fix: geometry that merely touches a pixel
boundary lights the row above it, and every band then reads one row early as ~4 px
of phantom overshoot on a single row. Sampling block centres cannot see it.
"""

import os
import sys

import bpy
import numpy as np
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import dwarf_r16 as D              # noqa: E402
import sheet_r16 as S              # noqa: E402

OUT = os.path.join(os.path.dirname(HERE), "renders", "r16")
SCALE = 5
PROPS = ("pickaxe", "lantern")     # r15 s3.2 -- never in OUR silhouette
ARMS = ("sleeve", "glove")         # r15 s3.3 -- reported separately, exempt

VIEWS = {
    # name: (res_x, res_y, centre, ortho_scale, location, rotation)
    "front": (174 * SCALE, 154 * SCALE, None, None, None, None),
    "side-left": (120 * SCALE, 154 * SCALE, None, None, None, None),
}


def px(n):
    return n * D.PX * D.H


def view_setup(name):
    """Return (res_x, res_y, cam_loc, cam_rot, ortho_scale) pinned to the crop."""
    zc = px((-6 + 148) / 2.0)                      # crop rows 0..153 about z
    zspan = px(154)
    # HANDEDNESS. +Y is forward (the nose), so a TRUE front view looks along -Y
    # from +Y, and a camera with up=+Z then has its image-right along -X. The
    # extraction's fx() puts image-left at -X, so the front render is flipped back
    # after the fact. The first cut of this had the camera at -Y looking +Y, which
    # silently rendered the BACK: symmetric in X, so every silhouette number came
    # out right and only the overlay would have shown it.
    if name == "front":
        rx, ry = 174 * SCALE, 154 * SCALE
        loc = (px(174 / 2.0 - 77.5), 4.0, zc)
        rot = (np.pi / 2, 0.0, np.pi)
        oscale = px(174)                           # res_x is the larger axis
    else:
        # side-left.png draws +Y (the nose) at image RIGHT, which needs the camera
        # on +X. The body is symmetric in X, so this is the sheet's own view.
        rx, ry = 120 * SCALE, 154 * SCALE
        loc = (4.0, px(120 / 2.0 - 58.0), zc)
        rot = (np.pi / 2, 0.0, np.pi / 2)
        oscale = zspan                             # res_y is the larger axis
    return rx, ry, loc, rot, oscale


def make_camera(name):
    cam = bpy.data.cameras.new(f"cam_{name}")
    cam.type = "ORTHO"
    rx, ry, loc, rot, oscale = view_setup(name)
    cam.ortho_scale = oscale
    ob = bpy.data.objects.new(f"cam_{name}", cam)
    ob.location = loc
    ob.rotation_euler = rot
    bpy.context.scene.collection.objects.link(ob)
    sc = bpy.context.scene
    sc.camera = ob
    sc.render.resolution_x, sc.render.resolution_y = rx, ry
    sc.render.resolution_percentage = 100
    return ob


def workbench():
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_WORKBENCH"
    sh = sc.display.shading
    sh.light = "FLAT"
    sh.color_type = "SINGLE"
    sh.single_color = (1, 1, 1)
    sh.show_object_outline = False
    sh.show_specular_highlight = False
    sc.render.film_transparent = True
    sc.view_settings.view_transform = "Standard"
    sc.render.image_settings.file_format = "PNG"
    sc.render.image_settings.color_mode = "RGBA"


def render_to_array(path):
    bpy.context.scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(path)
    img.colorspace_settings.name = "Non-Color"
    w, h = img.size
    buf = np.empty(w * h * 4, np.float32)
    img.pixels.foreach_get(buf)
    a = buf.reshape(h, w, 4)[::-1]
    bpy.data.images.remove(img)
    return a


def silhouette(view, include=None, exclude=()):
    """Our mask at SOURCE resolution, block centres, alpha-thresholded."""
    for ob in bpy.context.scene.objects:
        if ob.type != "MESH":
            continue
        part = ob.name.split("_", 1)[1]
        vis = (include is None or part in include) and part not in exclude
        ob.hide_render = not vis
    a = render_to_array(os.path.join(OUT, "scratch", f"sil-{view}.png"))
    if view == "front":
        a = a[:, ::-1]           # camera right is -X; fx() puts image-left at -X
    alpha = a[..., 3]
    return alpha[SCALE // 2::SCALE, SCALE // 2::SCALE] > 0.5


# ------------------------------------------------------------------- the gate
# r15 s3: the OUTLINE, row by row, from the first build. Landmark checks are NOT a
# gate -- round 14 had thirty of them reading +0.00 px while the outline was 4-5 px
# off, because a box added BETWEEN two landmarks moves the silhouette without
# moving either. Tolerance: <= 3 source px on the body passes (r15 s3, as amended).
#
# The envelope's real job is catching DRIFT, not proving fidelity. One source pixel
# is 0.06 of a screen pixel at the distance dwarves are actually seen.

TOL = 3.0
DECILES = 10


def sheet_edges(view):
    """(left, right) per source row of the ART's body, prop-free, symmetrised for
    the front (r15 s3.1: the art is hand-drawn asymmetric; comparing raw charges us
    for its own wobble, and cost round 14 a full source pixel of phantom error)."""
    name = "front.png" if view == "front" else "side-left.png"
    e = S.extract(name)
    n = e["img"].shape[0]
    L = np.full(n, np.nan)
    R = np.full(n, np.nan)
    for r in range(n):
        rr = [x for x in S.row_runs(e["fig"][r]) if x[1] - x[0] >= 4]
        if not rr:
            continue
        if view == "front":
            c = e["centre"]
            half = min(c - rr[0][0], rr[-1][1] - c)
            L[r], R[r] = c - half, c + half
        else:
            L[r], R[r] = rr[0][0], rr[-1][1]
    return L, R


def ours_edges(mask):
    n = mask.shape[0]
    L = np.full(n, np.nan)
    R = np.full(n, np.nan)
    for r in range(n):
        idx = np.flatnonzero(mask[r])
        if idx.size:
            L[r], R[r] = idx[0], idx[-1] + 1
    return L, R


def envelope(view, mask, label):
    """Worst OVERSHOOT per z-decile, in source pixels, plus undershoot.

    r16 s6 is why undershoot is reported at all: a wedge cannot push the silhouette
    out, but it CAN pull it in, and the three places that matters -- the widest row
    of head/beard/skirt, the bare-neck run, the face island -- are all invisible to
    an overshoot-only gate.
    """
    sL, sR = sheet_edges(view)
    oL, oR = ours_edges(mask)
    rows = [r for r in range(len(sL))
            if not np.isnan(sL[r]) and not np.isnan(oL[r])]
    if not rows:
        return None
    top, bot = min(rows), max(rows)
    out = []
    for d in range(DECILES):
        a = top + (bot - top + 1) * d // DECILES
        bnd = top + (bot - top + 1) * (d + 1) // DECILES
        seg = [r for r in rows if a <= r < bnd]
        if not seg:
            continue
        over = max(max(sL[r] - oL[r], oR[r] - sR[r]) for r in seg)
        under = max(max(oL[r] - sL[r], sR[r] - oR[r]) for r in seg)
        zt = (147 - a) / 140.0
        zb_ = (147 - bnd) / 140.0
        worst = max(seg, key=lambda r: max(sL[r] - oL[r], oR[r] - sR[r]))
        out.append((d, zt, zb_, over, under, worst))
    print(f"\n  {label} / {view}: worst overshoot per z-decile (source px, "
          f"tolerance {TOL:.0f})")
    print("    decile   z/H          over    under   worst row")
    for (d, zt, zb_, over, under, worst) in out:
        flag = "  FLAG" if over > TOL else ""
        print(f"      {d}    {zt:.3f}..{zb_:.3f}  {over:+6.1f}  {under:+6.1f}"
              f"     {worst}{flag}")
    worst_over = max(o[3] for o in out)
    print(f"    worst overshoot on the body: {worst_over:+.1f} px "
          f"({'PASS' if worst_over <= TOL else 'FLAG'})")
    return worst_over


def overlay(view, mask, path):
    """Our silhouette over the sheet crop, at the crop's own size -- no resampling."""
    name = "front.png" if view == "front" else "side-left.png"
    art = S.source_image(name) / 255.0
    h, w = mask.shape
    out = art.copy()
    e = S.extract(name)
    fig = e["fig"]
    # art-only in red, ours-only in blue, agreement left as the art
    only_art = fig & ~mask
    only_ours = mask & ~fig
    out[only_art] = out[only_art] * 0.35 + np.array([0.90, 0.10, 0.10]) * 0.65
    out[only_ours] = out[only_ours] * 0.35 + np.array([0.10, 0.35, 0.95]) * 0.65
    save_rgb(out, path)


def save_rgb(arr, path):
    h, w = arr.shape[:2]
    img = bpy.data.images.new(os.path.basename(path), w, h, alpha=False)
    rgba = np.ones((h, w, 4), np.float32)
    rgba[..., :3] = np.clip(arr, 0, 1)
    img.pixels.foreach_set(rgba[::-1].ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    bpy.data.images.remove(img)


# ------------------------------------------------------- the lit A/B, r16 s7
# r15 s5: Workbench flat is correct for the GATES and actively misleading for "does
# it look like the reference". Round 14 spent most of its length reasoning about art
# from neutral studio renders. The figure is judged by eye on the lit one -- and at
# Stage A, judged on GREY, which isolates what the wedges do to the shading from
# what paint would do.

def grey_material():
    m = bpy.data.materials.new("r16_grey")
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (0.45, 0.44, 0.42, 1)
    bsdf.inputs["Roughness"].default_value = 0.72
    bsdf.inputs["Specular IOR Level"].default_value = 0.5
    for ob in bpy.context.scene.objects:
        if ob.type == "MESH":
            ob.data.materials.append(m)


def lit_scene():
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_EEVEE"
    sc.view_settings.view_transform = "AgX"
    sc.render.film_transparent = False
    try:
        sc.eevee.use_gtao = True
        sc.eevee.use_raytracing = True
    except AttributeError:
        pass
    world = bpy.data.worlds.new("r16_dark")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.012, 0.014, 0.020, 1)
    world.node_tree.nodes["Background"].inputs[1].default_value = 1.0
    sc.world = world

    key = bpy.data.lights.new("key", "AREA")
    # 260 W was tuned while the atlas was a full sRGB decode too dark (see
    # paint_r16.rgb); once the texture carried its real albedo the same key blew the
    # mid-tones out to pastel. This is r15 s5's point from the other side: the
    # palette is not too desaturated, but it is only judgeable under light that is
    # actually comparable to the frame's.
    key.energy, key.size = 95.0, 1.4
    key.color = (1.0, 0.80, 0.58)                  # the mine's warm torchlight
    ko = bpy.data.objects.new("key", key)
    ko.location = (1.9, 1.7, 2.1)
    ko.rotation_euler = (Vector((0.0, 0.0, 0.75)) - Vector(ko.location)
                         ).to_track_quat("-Z", "Y").to_euler()
    sc.collection.objects.link(ko)

    fill = bpy.data.lights.new("fill", "AREA")
    fill.energy, fill.size = 9.0, 2.0
    fill.color = (0.55, 0.65, 0.95)
    fo = bpy.data.objects.new("fill", fill)
    fo.location = (-1.9, 1.1, 1.2)
    fo.rotation_euler = (Vector((0.0, 0.0, 0.7)) - Vector(fo.location)
                         ).to_track_quat("-Z", "Y").to_euler()
    sc.collection.objects.link(fo)
    rim_light()


def rim_light():
    """A cool back rim. Without it the back of the figure renders essentially black
    and cannot be judged at all -- which is a lighting failure masquerading as a
    modelling one, and exactly the sort of thing r15 s5 warns about reasoning from."""
    sc = bpy.context.scene
    rim = bpy.data.lights.new("rim", "AREA")
    rim.energy, rim.size = 55.0, 1.6
    rim.color = (0.62, 0.72, 1.0)
    ro = bpy.data.objects.new("rim", rim)
    ro.location = (-1.4, -2.2, 1.7)
    ro.rotation_euler = (Vector((0.0, 0.0, 0.75)) - Vector(ro.location)
                         ).to_track_quat("-Z", "Y").to_euler()
    sc.collection.objects.link(ro)
    return ro


def lit_camera():
    cam = bpy.data.cameras.new("cam_lit")
    cam.lens = 70.0
    ob = bpy.data.objects.new("cam_lit", cam)
    # +Y is forward, so a three-quarter FRONT view sits at +Y, not -Y.
    ob.location = (1.35, 2.25, 1.05)
    ob.rotation_euler = (Vector((0.0, 0.0, 0.62)) - Vector(ob.location)
                         ).to_track_quat("-Z", "Y").to_euler()
    bpy.context.scene.collection.objects.link(ob)
    sc = bpy.context.scene
    sc.camera = ob
    sc.render.resolution_x, sc.render.resolution_y = 560, 900
    return ob


def variant(facet, tag):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    D.FACET = facet
    b = D.build()
    b.to_objects()
    faces, ndir, off, canon = D.metric(b)
    print(f"\n{'='*66}\n  {tag}  FACET={facet}")
    print(f"  cage faces {faces} | distinct directions {ndir} "
          f"({canon} canonical + {ndir - canon} ramp angles) | "
          f"off the six axes {off} ({100.0*off/faces:.1f} %)")
    print("=" * 66)

    workbench()
    res = {}
    for view in ("front", "side-left"):
        make_camera(view)
        body = silhouette(view, exclude=PROPS + ARMS)
        res[view] = envelope(view, body, tag)
        overlay(view, body, os.path.join(OUT, f"overlay-A-{view}-{tag}.png"))
        arms = silhouette(view, include=ARMS)
        envelope(view, arms | body, f"{tag} incl. arms")
        for ob in list(bpy.context.scene.objects):
            if ob.type == "CAMERA":
                bpy.data.objects.remove(ob, do_unlink=True)
    for ob in bpy.context.scene.objects:
        if ob.type == "MESH":
            ob.hide_render = False

    grey_material()
    lit_scene()
    lit_camera()
    a = render_to_array(os.path.join(OUT, "scratch", f"lit-{tag}.png"))
    return res, faces, ndir, off, canon, a[..., :3]


def main():
    os.makedirs(os.path.join(OUT, "scratch"), exist_ok=True)
    stats = {}
    lit = {}
    for facet, tag in ((False, "r15"), (True, "r16")):
        res, faces, ndir, off, canon, img = variant(facet, tag)
        stats[tag] = (faces, ndir, off, res, canon)
        lit[tag] = img

    h = min(lit["r15"].shape[0], lit["r16"].shape[0])
    gap = 12
    w = lit["r15"].shape[1] + lit["r16"].shape[1] + gap
    ab = np.zeros((h, w, 3), np.float32)
    ab[:, :lit["r15"].shape[1]] = lit["r15"][:h]
    ab[:, lit["r15"].shape[1] + gap:] = lit["r16"][:h]
    save_rgb(ab, os.path.join(OUT, "ab-r15-vs-r16-lit.png"))

    print("\n" + "=" * 66)
    print("  A/B -- same camera, same pose, same lighting, grey (r16 s7)")
    print("  rev | cage faces | directions (canon) | off-axis | overshoot F / S")
    for tag in ("r15", "r16"):
        f, n, o, res, canon = stats[tag]
        print(f"  {tag} | {f:10d} | {n:6d} ({canon:2d})      | {100.0*o/f:7.1f}% | "
              f"{res['front']:+.1f} / {res['side-left']:+.1f} px")
    print("=" * 66)


if __name__ == "__main__":
    main()


# ------------------------------------------------------------- Stage B renders
# r15 s5: the figure is judged by eye on the LIT one. Workbench flat is correct for
# the gates and actively misleading for "does it look like the reference" -- round 14
# spent most of its length reasoning about art from neutral studio renders and
# concluded the approved palette was too desaturated. It is not.

def load_rgb(path):
    img = bpy.data.images.load(path)
    img.colorspace_settings.name = "Non-Color"
    w, h = img.size
    b = np.empty(w * h * 4, np.float32)
    img.pixels.foreach_get(b)
    a = b.reshape(h, w, 4)[::-1, :, :3].copy()
    bpy.data.images.remove(img)
    return a


def beside(ours, frame_path, out_path):
    """Our lit render next to the reference frame, matched in height."""
    ref = load_rgb(frame_path)
    h = max(ours.shape[0], ref.shape[0])
    gap = 14
    w = ours.shape[1] + ref.shape[1] + gap
    strip = np.full((h, w, 3), 0.02, np.float32)
    strip[:ours.shape[0], :ours.shape[1]] = ours
    strip[:ref.shape[0], ours.shape[1] + gap:] = ref
    save_rgb(strip, out_path)


def stage_b():
    import paint_r16 as P
    col, _ = P.paint(facet=True)

    lit_scene()
    lit_camera()
    sc = bpy.context.scene
    ours = render_to_array(os.path.join(OUT, "scratch", "lit-painted.png"))[..., :3]
    frames = os.path.join(os.path.dirname(HERE), "references", "dwarf-frames")
    beside(ours, os.path.join(frames, "f104.png"), os.path.join(OUT, "lit-vs-f104.png"))
    beside(ours, os.path.join(frames, "f088.png"), os.path.join(OUT, "lit-vs-f088.png"))

    # face close-up, lit and flat -- the flat one is the gate on where the paint sits
    cam = sc.camera
    cam.location = (0.22, 1.05, 0.98)
    cam.rotation_euler = (Vector((0.0, 0.0, 0.94)) - Vector(cam.location)
                          ).to_track_quat("-Z", "Y").to_euler()
    cam.data.lens = 95.0
    sc.render.resolution_x, sc.render.resolution_y = 520, 520
    a = render_to_array(os.path.join(OUT, "scratch", "face-key.png"))[..., :3]
    save_rgb(a, os.path.join(OUT, "face-key.png"))
    workbench()
    sc.display.shading.color_type = "TEXTURE"
    sc.render.film_transparent = False
    a = render_to_array(os.path.join(OUT, "scratch", "face-flat.png"))[..., :3]
    save_rgb(a, os.path.join(OUT, "face-flat.png"))

    # readability at the sizes the figure is actually seen (r15 deliverable 7)
    lit_scene()
    sc.render.engine = "BLENDER_EEVEE"
    cam.data.lens = 70.0
    cam.location = (1.35, 2.25, 1.05)
    cam.rotation_euler = (Vector((0.0, 0.0, 0.62)) - Vector(cam.location)
                          ).to_track_quat("-Z", "Y").to_euler()
    strips = []
    for px_h in (200, 100, 60):
        sc.render.resolution_y = px_h
        sc.render.resolution_x = int(px_h * 0.62)
        strips.append(render_to_array(
            os.path.join(OUT, "scratch", f"read-{px_h}.png"))[..., :3])
    H_ = max(s.shape[0] for s in strips)
    W_ = sum(s.shape[1] for s in strips) + 20 * (len(strips) - 1)
    out = np.full((H_, W_, 3), 0.02, np.float32)
    x = 0
    for s in strips:
        out[H_ - s.shape[0]:, x:x + s.shape[1]] = s
        x += s.shape[1] + 20
    save_rgb(out, os.path.join(OUT, "readability.png"))
    print("  stage B renders written to", OUT)


# ------------------------------------------------- Stage C: deflection and pose
# r15 deliverable 8. The point of rigid, by-BOX weights is that a joint moves whole
# masses and nothing shears; these renders are what shows that, since no dimensional
# check can (round 14 shipped eight builds with every arm face reversed while thirty
# landmark checks read +0.00 px).

DEFLECT = [
    ("neck-head", [("neck", "X", -22), ("head", "Z", 28)]),
    ("shoulder-elbow", [("shoulder.R", "X", -45), ("elbow.R", "X", -55)]),
    ("hip-knee-foot", [("hip.L", "X", -40), ("knee.L", "X", 55), ("foot.L", "X", -18)]),
    ("spine-chest", [("spine", "X", -18), ("chest", "X", -14)]),
    ("beard", [("beard", "X", 25)]),
    ("hand", [("hand.R", "Y", 35), ("hand.L", "Y", -35)]),
]

CARRY = [("shoulder.R", "X", -62), ("elbow.R", "X", -48), ("shoulder.L", "X", -20),
         ("elbow.L", "X", -35), ("spine", "X", -8), ("head", "X", 6)]


def apply_pose(arm, spec):
    for pb in arm.pose.bones:
        pb.rotation_mode = "XYZ"
        pb.rotation_euler = (0, 0, 0)
    for name, axis, deg in spec:
        pb = arm.pose.bones[name]
        i = "XYZ".index(axis)
        e = list(pb.rotation_euler)
        e[i] = np.radians(deg)
        pb.rotation_euler = e
    bpy.context.view_layer.update()


def stage_c():
    import paint_r16 as P
    import rig_r16 as R
    col, _ = P.paint(facet=True)
    b = P.LAST_BUILD
    arm = R.build_rig(b, col)
    R.skin(b, col, arm)

    lit_scene()
    cam = lit_camera()
    sc = bpy.context.scene
    sc.render.resolution_x, sc.render.resolution_y = 460, 760

    for tag, spec in DEFLECT:
        apply_pose(arm, spec)
        a = render_to_array(os.path.join(OUT, "scratch", f"j-{tag}.png"))[..., :3]
        save_rgb(a, os.path.join(OUT, f"joint-{tag}.png"))

    apply_pose(arm, CARRY)
    a = render_to_array(os.path.join(OUT, "scratch", "carry-q.png"))[..., :3]
    save_rgb(a, os.path.join(OUT, "pose-carry-quarter.png"))
    cam.location = (0.0, 2.6, 0.72)
    cam.rotation_euler = (Vector((0.0, 0.0, 0.62)) - Vector(cam.location)
                          ).to_track_quat("-Z", "Y").to_euler()
    a = render_to_array(os.path.join(OUT, "scratch", "carry-f.png"))[..., :3]
    save_rgb(a, os.path.join(OUT, "pose-carry-front.png"))
    print(f"  stage C: {len(DEFLECT)} deflection renders + 2 carry renders")
