"""Renders and comparison sheets for the round-8 dwarf.

    blender --background <blend> --python src-assets/blender/render_r8.py -- <mode> [res]

    final    the five delivered views, flat AND key-lit, into src-assets/renders/r8/
    sheets   readability strip, the two ortho side-by-sides, the face plates, the body plate
    gear     the gear close-up beside the sheet's own gear breakdown
    pose     the two-handed carry, front and three-quarter (SS6: a POSE, never baked to vertices)
    joints   one deflection render per joint group

WHAT CARRIES OVER FROM render_r7.py, because it earned it:
  * the flat pass is Workbench FLAT + TEXTURE. It is the only honest read of the albedo, it has
    no sampler noise, and every measurement in the report is taken off it.
  * the lit pass is EEVEE with ONE sun and real shadows aimed along f088's key -- high, from the
    figure's front left -- plus a weak shadowless fill. A light with no shadow cannot show a
    brow ridge, and round 8 is a round about form.
  * every path is ABSOLUTE and carries the revision. Blender resolves a relative
    render.filepath against the .blend, which is how round 5's renders escaped to C:/.
  * the world background is driven through the BACKGROUND NODE. World.use_nodes is on its way
    out in 5.x and setting world.color no longer reaches EEVEE.

WHAT IS NEW IN r8:
  * the face plate is rendered in THREE-QUARTER as well as front, because the whole point of
    modelled form is that it survives a camera move (SS9);
  * a gear plate, because SS3.5 is judged on the pickaxe, lantern, pack and straps and none of
    them reads in a full-figure shot;
  * the reference crops are cut to the figure's own ROWS (7..147 of the source sheet, which is
    z 1.200..0.000) before trimming. r7 trimmed on colour alone, which lets the sheet's grid
    lines and dimension arrows into the crop and quietly breaks the height match.
  * pose and joint modes, which need the armature and say so if it is not there.
"""

import math
import os
import sys

import bpy
from mathutils import Vector

REV = "r8"
COLLECTION = "SM_VoxelDwarf_Miner01_%s" % REV
ARMATURE = "SK_VoxelDwarf_Miner01_%s" % REV
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)                      # src-assets/
OUT = os.path.join(ROOT, "renders", REV)
REFS = os.path.join(ROOT, "references")

VIEWS = [("front", 0.0, 0.0), ("side-left", -90.0, 0.0), ("side-right", 90.0, 0.0),
         ("back", 180.0, 0.0), ("three-quarter", 35.0, 12.0)]

# The sheet draws the figure over source rows 7..147: row 7 is the crown at z = 1.200 and row
# 147 is the sole at z = 0.000, 140 rows for 1.200 m. Cropping to these rows before trimming is
# what makes "matched scale" true rather than asserted.
FIG_ROWS = (7, 148)
SRC_SCALE = 5                                     # dwarf-ortho/ is a 5x nearest blow-up

# SS6: the mesh ships in the NEUTRAL stance and the GLB carries the rest pose. This is applied
# for renders only and reverted afterwards, so nothing here can reach the exported vertices.
#
# The carry is NOT a table of Euler angles, and the reason is worth stating. The pickaxe is
# rigidly bound to hand.R, so its final orientation is the composition of three bone rotations
# and cannot be dialled in by hand -- every attempt to steepen the shaft also swung it behind
# the beard. Instead the shoulder and elbow PLACE the hand and hand.R's WORLD matrix is set so
# that its -Y axis (the shaft's direction in the hand's rest frame) lies along SHAFT_DIR.
#
# SHAFT_DIR is measured, not chosen: on front.png the shaft crosses the whole drawing at about
# 17 degrees off HORIZONTAL. Every earlier guess had the pick far too upright, which is what put
# the blade across the face.
#
# hand.L gets the same treatment for the opposite reason: the lantern hangs from it, and a
# hanging lantern stays vertical, so hand.L's world rotation is pinned back to its rest value
# after the arm moves.
SHAFT_DIR = (-0.90, 0.22, 0.36)
CARRY_ARMS = {"shoulder.R": (0.75, 0.00, 0.35), "elbow.R": (-0.70, 0.00, 0.00),
              "shoulder.L": (0.85, 0.00, 0.35), "elbow.L": (-0.75, 0.00, 0.00)}
# One render per joint group, each with the camera that actually SHOWS its bend -- a deflection
# aimed away from the lens proves nothing. Bones swing forward (local +X) so the deformation
# faces the viewer rather than hiding behind the torso.
DEFLECT = [
    ("shoulders-elbows", {"shoulder.L": (0.95, 0, 0), "shoulder.R": (0.95, 0, 0),
                          "elbow.L": (-1.10, 0, 0), "elbow.R": (-1.10, 0, 0)}, 34.0, 8.0),
    ("hips-knees",       {"hip.L": (0.85, 0, 0), "knee.L": (0.90, 0, 0),
                          "hip.R": (-0.30, 0, 0)}, -78.0, 2.0),
    ("neck-head",        {"neck": (0.35, 0, 0), "head": (0.25, 0, 0.70)}, 18.0, 6.0),
    ("beard",            {"beard": (0.70, 0, 0)}, -68.0, 2.0),
    ("spine-hips",       {"hips": (0.22, 0, 0), "spine": (0.42, 0, 0.20)}, -80.0, 4.0),
]


def _srgb_to_linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


BG = tuple(_srgb_to_linear(c / 255.0) for c in (0x6F, 0x70, 0x73))   # #6F7073


def figure_bounds(objs=None):
    lo = Vector((1e9, 1e9, 1e9))
    hi = Vector((-1e9, -1e9, -1e9))
    src = objs if objs is not None else bpy.data.collections[COLLECTION].objects
    for ob in src:
        if ob.type != 'MESH':
            continue
        for v in ob.data.vertices:
            w = ob.matrix_world @ v.co
            lo = Vector((min(lo[i], w[i]) for i in range(3)))
            hi = Vector((max(hi[i], w[i]) for i in range(3)))
    return lo, hi


def purge_cams():
    for ob in list(bpy.data.objects):
        if ob.type == 'CAMERA':
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
    # Raytracing OFF: with one sun and flat albedo it contributes nothing but sampler grain,
    # and grain on a flat-shaded asset reads as texture that is not there.
    for attr, val in (("taa_render_samples", 64), ("use_shadows", True),
                      ("use_raytracing", False)):
        if hasattr(scn.eevee, attr):
            setattr(scn.eevee, attr, val)
    key = bpy.data.objects.get("R8Key")
    if key is None:
        key = bpy.data.objects.new("R8Key", bpy.data.lights.new("R8Key", 'SUN'))
        scn.collection.objects.link(key)
    key.data.energy = 4.2
    key.data.angle = math.radians(3.0)          # nearly parallel -> a crisp shadow
    key.data.use_shadow = True
    _aim(key, -42.0, 34.0)
    fill = bpy.data.objects.get("R8Fill")
    if fill is None:
        fill = bpy.data.objects.new("R8Fill", bpy.data.lights.new("R8Fill", 'SUN'))
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
    cam = bpy.data.objects.get("R8Cam")
    if cam is None:
        cam = bpy.data.objects.new("R8Cam", bpy.data.cameras.new("R8Cam"))
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


# ---------------------------------------------------------------- image helpers

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
    os.makedirs(os.path.dirname(path), exist_ok=True)
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


def _ref_figure(name):
    """An ortho crop cut to the FIGURE'S OWN ROWS first, then trimmed in x only.

    The sheet carries a grid and dimension arrows that a colour trim happily includes, and any
    pixel of those above the crown or below the sole silently rescales the comparison. The row
    bounds are a measurement: source row 7 is z = 1.200 and row 147 is z = 0.000.
    """
    import numpy as np
    a = _read(os.path.join(REFS, "dwarf-ortho", name))
    a = a[FIG_ROWS[0] * SRC_SCALE:FIG_ROWS[1] * SRC_SCALE]
    d = np.abs(a - np.array((0.90, 0.92, 0.93), np.float32)).max(axis=2) > 0.10
    cols = np.nonzero(d.any(axis=0))[0]
    return a[:, cols.min():cols.max() + 1]


def _plate(path, tiles, h, pad=12):
    """Several images laid side by side, each scaled to the SAME height."""
    import numpy as np
    scaled = [_nn(t, max(1, int(round(t.shape[1] * h / t.shape[0]))), h) for t in tiles]
    W = sum(t.shape[1] for t in scaled) + pad * (len(scaled) + 1)
    c = np.full((h + 2 * pad, W, 3), 0.42, np.float32)
    x = pad
    for t in scaled:
        c[pad:pad + h, x:x + t.shape[1]] = t
        x += t.shape[1] + pad
    return _write(path, c)


# ---------------------------------------------------------------- the rig, for pose/joints

def armature():
    arm = bpy.data.objects.get(ARMATURE)
    if arm is None:
        raise SystemExit("render_r8: no armature %r in this .blend -- half B has not run" % ARMATURE)
    return arm


def apply_pose(arm, table):
    bpy.context.view_layer.objects.active = arm
    for bone in arm.pose.bones:
        bone.rotation_mode = 'XYZ'
        bone.rotation_euler = (0.0, 0.0, 0.0)
    for name, rot in table.items():
        pb = arm.pose.bones.get(name)
        if pb is not None:
            pb.rotation_euler = rot
    bpy.context.view_layer.update()


def clear_pose(arm):
    apply_pose(arm, {})


def apply_carry(arm):
    """The two-handed carry: place the hands with the arms, then set each hand's WORLD rotation.

    See the note on SHAFT_DIR. Euler tables cannot hit a prop's final orientation through a
    three-bone chain; setting the last bone's world matrix can, and it is the same one line for
    the pick (aim it) and the lantern (leave it hanging).
    """
    from mathutils import Matrix, Vector
    clear_pose(arm)
    rest_hand_l = arm.pose.bones["hand.L"].matrix.to_quaternion().copy()
    for name, rot in CARRY_ARMS.items():
        arm.pose.bones[name].rotation_euler = rot
    bpy.context.view_layer.update()

    right = arm.pose.bones["hand.R"]
    aim = (-Vector(SHAFT_DIR).normalized()).to_track_quat('Y', 'Z')
    right.matrix = Matrix.Translation(right.matrix.translation) @ aim.to_matrix().to_4x4()
    bpy.context.view_layer.update()

    left = arm.pose.bones["hand.L"]
    left.matrix = Matrix.Translation(left.matrix.translation) @ rest_hand_l.to_matrix().to_4x4()
    bpy.context.view_layer.update()


# ---------------------------------------------------------------- modes

def _frame(pad=1.06, objs=None):
    lo, hi = figure_bounds(objs)
    return max(hi.x - lo.x, hi.y - lo.y, hi.z - lo.z) * pad, (lo + hi) / 2.0


def final(res):
    span, centre = _frame()
    for name, az, el in VIEWS:
        shot(os.path.join(OUT, "dwarf-flat-%s.png" % name), az, el, res, False, span, centre)
        shot(os.path.join(OUT, "dwarf-lit-%s.png" % name), az, el, res, True, span, centre)


def sheets(res):
    import numpy as np
    span, centre = _frame()
    tmp = os.path.join(OUT, "_tmp")

    # --- readability: the figure at 100 px and 60 px of FIGURE height, key-lit
    tiles = []
    for az, el in ((0.0, 0.0), (35.0, 12.0)):
        src = shot(os.path.join(tmp, "read-%.0f.png" % az), az, el, 900, True, span, centre)
        fig = _trim(_read(src))
        for px in (100, 60):
            tiles.append(_nn(fig, max(1, int(round(fig.shape[1] * px / fig.shape[0]))), px))
    pad = 12
    H = max(t.shape[0] for t in tiles) + 2 * pad
    W = sum(t.shape[1] + pad for t in tiles) + pad
    canvas = np.full((H, W, 3), 0.42, np.float32)
    x = pad
    for t in tiles:
        canvas[H - pad - t.shape[0]:H - pad, x:x + t.shape[1]] = t
        x += t.shape[1] + pad
    _write(os.path.join(OUT, "readability.png"), _nn(canvas, W * 3, H * 3))

    # --- side by side against the orthographic sheet, matched on FIGURE height
    for name, ref, az in (("front", "front.png", 0.0), ("side-left", "side-left.png", -90.0)):
        ours = _trim(_read(shot(os.path.join(tmp, "vs-%s.png" % name), az, 0.0, 900,
                                False, span, centre)))
        _plate(os.path.join(OUT, "vs-ortho-%s.png" % name), [ours, _ref_figure(ref)], 700)

    # --- the face, front AND three-quarter, beside f104 at matched head height
    f104 = _read(os.path.join(REFS, "dwarf-frames", "f104.png"))[30:250, 120:330]
    for tag, az, el in (("front", 0.0, 0.0), ("tq", 34.0, 6.0)):
        ours = _read(shot(os.path.join(tmp, "face-%s.png" % tag), az, el, 760, True,
                          0.46, (0.0, 0.05, 1.00)))
        _plate(os.path.join(OUT, "vs-face-%s.png" % tag), [ours, f104], 640)

    # --- the body beside the sheet, matched scale
    body = _trim(_read(shot(os.path.join(tmp, "body.png"), 0.0, 0.0, 900, True, span, centre)))
    _plate(os.path.join(OUT, "vs-body.png"), [body, _ref_figure("front.png")], 760)
    print("SHEETS -> %s" % OUT)


def gear(res):
    """SS3.5 is judged on the pickaxe, lantern, pack and straps, and none of them reads in a
    full-figure shot. Each gets a framed close-up, beside the sheet's own gear breakdown."""
    groups = [("pickaxe", ["r8_pickaxe_head", "r8_pickaxe_binding", "r8_pickaxe_shaft"], 30.0, 6.0),
              ("lantern", ["r8_lantern_frame", "r8_lantern_glass", "r8_lantern_flame",
                           "r8_lantern_cap", "r8_lantern_bail"], 28.0, 8.0),
              ("pack", ["r8_pack", "r8_pack_flap", "r8_pack_buckle", "r8_bedroll",
                        "r8_strap.L", "r8_strap.R"], 205.0, 14.0)]
    tiles = []
    for tag, names, az, el in groups:
        objs = [bpy.data.objects[n] for n in names if n in bpy.data.objects]
        span, centre = _frame(1.12, objs)
        tiles.append(_read(shot(os.path.join(OUT, "_tmp", "gear-%s.png" % tag),
                                az, el, 700, True, span, centre)))
    tiles.append(_read(os.path.join(REFS, "dwarf-ortho", "gear.png")))
    _plate(os.path.join(OUT, "vs-gear.png"), tiles, 560)
    print("GEAR -> %s" % os.path.join(OUT, "vs-gear.png"))


def pose(res):
    arm = armature()
    apply_carry(arm)
    span, centre = _frame(1.10)
    for tag, az, el in (("front", 0.0, 0.0), ("tq", 34.0, 10.0)):
        shot(os.path.join(OUT, "pose-carry-%s.png" % tag), az, el, res, True, span, centre)
    clear_pose(arm)
    print("POSE -> %s" % OUT)


def joints(res):
    arm = armature()
    for tag, table, az, el in DEFLECT:
        apply_pose(arm, table)
        span, centre = _frame(1.16)
        shot(os.path.join(OUT, "joint-%s.png" % tag), az, el, res, True, span, centre)
    clear_pose(arm)
    print("JOINTS -> %s" % OUT)


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else ["final"]
    mode = argv[0]
    res = int(argv[1]) if len(argv) > 1 else 700
    purge_cams()
    fn = {"final": final, "sheets": sheets, "gear": gear, "pose": pose, "joints": joints}.get(mode)
    if fn is None:
        raise SystemExit("render_r8: unknown mode %r" % mode)
    fn(res)
    print("RENDERED mode=%s -> %s" % (mode, OUT))


if __name__ == "__main__":
    main()
