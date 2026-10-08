"""Round 20 -- the work clip `Cut` for the r17 voxel dwarf, and the axe it swings.

The fourth link of the chain dwarf_r17.py -> walk_r18.py -> work_r19.py -> cut_r20.py. Run
it in the live session against SM_VoxelDwarf_Miner01.blend, after Walk, Dig and Carry exist:

    exec(open(r"src-assets/blender/cut_r20.py").read())
    build_cut(); checks(); detach()

`Cut` follows the round-18/19 rules: frames 0..24 at 24 fps, frame 24 == frame 0, every bone
keyed on every frame, LINEAR; in place; saved unassigned on a fake user. Walk, Dig and Carry
are not re-authored: their 20-joint curves hash exactly as round 19 left them.

Option (B), on Wolf's yes ("option B with proper axe"): add_axe() adds a 22nd part `r17_axe`
weighted to a 21st joint `axe`, child of `chest`. At rest the axe is strapped to the left
side of the pack, so the three older clips need only constant rest keys for `axe`. In Cut it
is pinned into the right fist every frame, and the pickaxe rides on the back in Carry's sling.
No existing vertex moves; export_dwarf.JOINTS carries `axe`.
"""

import hashlib
import importlib
import math
import os
import sys

import bpy
from mathutils import Matrix, Quaternion, Vector

try:
    HERE = os.path.dirname(os.path.abspath(__file__))
except NameError:                       # exec(open(...).read()) in the live session
    HERE = os.path.dirname(bpy.data.filepath)
if HERE not in sys.path:
    sys.path.insert(0, HERE)
import dwarf_r17 as D                   # noqa: E402
import work_r19 as V                    # noqa: E402
importlib.reload(V)
W = V.W

FPS, CYCLE = V.FPS, V.CYCLE
X, Y, Z = W.X, W.Y, W.Z
AXE = "axe"
AXE_PART = "r17_axe"
BONES = V.BONES + [AXE]
OLD = ("Walk", "Dig", "Carry")
ACTIONS = ("Carry", "Cut", "Dig", "Walk")
STAGES = os.path.join(os.path.dirname(HERE), "renders", "r20", "stages")
ROUND19 = {
    "Walk": "b1079d210fb881bdcc87ee2a2d4992b4c3b0a6572707339ff86ac2907950d3ea",
    "Dig": "844687a11997b6826d097eee23bb3332c3838be56e26b5666038da56c472b962",
    "Carry": "c50f8269451166101c9dc64f18fc93e9a4be8584a3f73d2d6f39b1439a175bde",
}

arm_ob = V.arm_ob
attach = V.attach
fcurve_hash = V.fcurve_hash


# ------------------------------------------------------------------- the axe
#
# Built in its own frame: the GRIP (the fist's centre) at the origin, the haft up +Z, the
# edge out along +X, the blade thin in Y. 0.87 m overall, a bearded single-bit felling axe.
# It was first built at the pickaxe's 1.05 m to reach the pine's real bark at 1.30 m; at
# that range the arm locks straight and the chop reads as a thrust (stages/cut-03), so Wolf
# moved the bite to 0.95 m with a client nudge (BARK_Y) and the haft came down to suit it.
# Boxes only, so PAINT-style per-box colours work by index.
AXE_PAINT = ("wood", {1: "trunk", 2: "metal", 3: "metal", 4: "metal", 5: "metal",
                      6: "metal", 7: "metal"})


def axe_boxes():
    box = D.box
    return [
        box(-0.018, 0.018, -0.018, 0.018, -0.022, 0.720),     # 0 haft, ending inside the fist
        box(-0.023, 0.023, -0.023, 0.023, -0.022, 0.105),     # 1 grip wrap, in the fist
        box(-0.025, 0.025, -0.025, 0.025, 0.105, 0.125),      # 2 ferrule over the wrap
        box(-0.032, 0.040, -0.026, 0.026, 0.600, 0.725),      # 3 eye, round the haft
        box(-0.075, -0.030, -0.021, 0.021, 0.620, 0.705),     # 4 poll, the hammer back
        box(0.038, 0.110, -0.016, 0.016, 0.590, 0.720),       # 5 cheek
        box(0.108, 0.170, -0.012, 0.012, 0.545, 0.735),       # 6 blade, bearded below
        box(0.168, 0.196, -0.008, 0.008, 0.525, 0.745),       # 7 bit: the cutting edge
    ]


# THE STOW: haft flat against the pack's left face (ray-cast at x -0.185..-0.190 over
# z 0.45..0.90, 4 mm overlap), running diagonally up and BACK along it, head up behind the
# left shoulder with the edge pointing back, away from the skull. Child of `chest`, so it
# rides with the pack. It first stood vertical at mid-depth (y -0.30), and Dig's forward lean
# tipped its head into the hair (1.4 mm, faces intersecting at frames 18-22); leaning it back
# 14.5 degrees moved the head ~10 cm off the skull. It rides high enough that Dig's lean-back
# (chest + spine +13 deg) cannot swing its foot into the skirt, as it did from z 0.40.
STOW_GRIP = Vector((-0.204, -0.235, 0.500))
STOW_HAFT = Vector((0.0, -0.25, 0.97)).normalized()
EDGE_STOWED = Vector((0.0, -1.0, 0.0))


def frame_matrix(grip, haft, edge):
    """Axe frame -> armature: origin at `grip`, +Z along `haft`, +X along `edge`."""
    u = haft.normalized()
    e = (edge - u * edge.dot(u)).normalized()
    m = Matrix((e, u.cross(e), u)).transposed().to_4x4()
    m.translation = grip
    return m


def stow_matrix():
    return frame_matrix(STOW_GRIP, STOW_HAFT, EDGE_STOWED)


def add_axe():
    """The 22nd part and the 21st joint. Idempotent: the cold chain re-runs it."""
    arm = arm_ob()
    coll = bpy.data.collections[D.COLL]
    if AXE not in arm.data.bones:
        bpy.context.view_layer.objects.active = arm
        prev = arm.mode
        bpy.ops.object.mode_set(mode='EDIT')
        eb = arm.data.edit_bones.new(AXE)
        eb.head = STOW_GRIP
        eb.tail = STOW_GRIP + STOW_HAFT * 0.100
        eb.roll = 0.0
        eb.parent = arm.data.edit_bones["chest"]
        eb.use_connect = False
        eb.use_deform = True
        bpy.ops.object.mode_set(mode=prev if prev != 'EDIT' else 'OBJECT')
    arm.pose.bones[AXE].rotation_mode = 'QUATERNION'

    ob = bpy.data.objects.get(AXE_PART)
    if ob is None:
        boxes = axe_boxes()
        m = stow_matrix()
        boxes = [([tuple(m @ Vector(p)) for p in v], f) for v, f in boxes]
        me = D.mesh_from(AXE_PART, boxes)
        ob = bpy.data.objects.new(AXE_PART, me)
        starts, acc = [], 0
        for bv, _ in boxes:
            starts.append(acc)
            acc += len(bv)
        ob["box_starts"] = starts
        ob["box_count"] = len(starts)
        coll.objects.link(ob)
        paint_axe(ob)
        ob.vertex_groups.new(name=AXE).add(list(range(len(me.vertices))), 1.0, 'REPLACE')
        ob.parent = arm
        ob.matrix_parent_inverse = arm.matrix_world.inverted()
        ob.modifiers.new("Armature", 'ARMATURE').object = arm
    print("joint %r: parent %s, %d joints; %s %d verts %d tris; parts %d"
          % (AXE, arm.data.bones[AXE].parent.name, len(arm.data.bones), AXE_PART,
             len(ob.data.vertices), sum(len(p.vertices) - 2 for p in ob.data.polygons),
             sum(1 for o in coll.objects if o.type == 'MESH')))
    key_rest_axe()
    return ob


def paint_axe(ob):
    """dwarf_r17's own atlas and cell rule: one texel per flat face, value step by normal."""
    mat = bpy.data.materials[D.MAT]
    ob.data.materials.append(mat)
    key, over = AXE_PAINT
    uv = ob.data.uv_layers.new(name="UVMap")
    for poly in ob.data.polygons:
        col, row = D.CELLS["%s.%s" % (over.get(poly.index // 6, key), D.orient(poly.normal))]
        for li in poly.loop_indices:
            uv.data[li].uv = ((col * D.CELL + D.CELL / 2) / D.TEX_SIZE,
                              (row * D.CELL + D.CELL / 2) / D.TEX_SIZE)


def rest_axe():
    pb = arm_ob().pose.bones[AXE]
    pb.rotation_mode = 'QUATERNION'
    pb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
    pb.location = (0.0, 0.0, 0.0)


def key_rest_axe():
    """`axe` keyed at rest on every frame of Walk, Dig and Carry: round 19's `pick` fix
    again. An un-keyed bone keeps its last pose, so without these a Walk after a Cut would
    walk with the axe still in his fist. Their 20-joint curves are not touched."""
    arm = arm_ob()
    keep = arm.animation_data.action if arm.animation_data else None
    for name in OLD:
        act = bpy.data.actions.get(name)
        if act is None or any('"%s"' % AXE in fc.data_path for fc in W.fcurves(act)):
            continue
        attach(name)
        rest_axe()
        pb = arm.pose.bones[AXE]
        for f in range(0, CYCLE + 1):
            pb.keyframe_insert("rotation_quaternion", frame=f, group=AXE)
            pb.keyframe_insert("location", frame=f, group=AXE)
        for fc in W.fcurves(act):
            if '"%s"' % AXE in fc.data_path:
                for kp in fc.keyframe_points:
                    kp.interpolation = 'LINEAR'
                fc.update()
        same = fcurve_hash(name, V.BONES) == ROUND19[name]
        print("%s: `axe` keyed at rest, frames 0..%d; round-19 curves %s"
              % (name, CYCLE, "UNCHANGED" if same else "CHANGED <-- FAIL"))
    if arm.animation_data is not None:
        arm.animation_data.action = keep
        if keep is not None:
            arm.animation_data.action_slot = keep.slots[0]


def view(yaw, pitch=75.0, dist=3.2, target=(0.0, 0.3, 0.65)):
    """Point Wolf's viewport: MATERIAL shading, yaw 0 looks at his back (he faces +Y),
    180 at his face, 90 at his right side."""
    win = bpy.context.window_manager.windows[0]
    area = next(a for a in win.screen.areas if a.type == 'VIEW_3D')
    sp = area.spaces[0]
    sp.shading.type = 'MATERIAL'
    r3 = sp.region_3d
    r3.view_perspective = 'PERSP'
    r3.view_rotation = Quaternion(Z, math.radians(yaw)) @ Quaternion(X, math.radians(pitch))
    r3.view_location = Vector(target)
    r3.view_distance = dist
    bpy.ops.wm.redraw_timer(type='DRAW_WIN_SWAP', iterations=1)


def stage_shot(name):
    """The 3D viewport as Wolf sees it, to renders/r20/stages/<name>.png."""
    os.makedirs(STAGES, exist_ok=True)
    path = os.path.join(STAGES, name + ".png")
    win = bpy.context.window_manager.windows[0]
    area = next(a for a in win.screen.areas if a.type == 'VIEW_3D')
    with bpy.context.temp_override(window=win, area=area):
        bpy.ops.screen.screenshot_area(filepath=path)
    print("stage shot %s" % path)
    return path


# ------------------------------------------------------------- pose assembly
#
# Round 19's channel convention (degrees about armature axes; these spine bones point UP so
# negative X leans forward), widened for a swing ACROSS the body: the spine and chest twist
# about Z (positive turns him to his LEFT), and each arm is the full 7-angle vector of
# work_r19.arm_quats -- shoulder X/Y/Z, elbow X, wrist X/Y/Z -- because a chop is not a
# sagittal motion and Dig's X-only arm cannot make one.
ARM7 = ("sx", "sy", "sz", "e", "wx", "wy", "wz")
REST = dict(drop=0.0, hips=0.0, spine=0.0, spine_tw=0.0, chest=0.0, chest_tw=0.0,
            neck=0.0, neck_tw=0.0, head=0.0, beard=0.0, grip=0.0,
            **{s + k: 0.0 for s in "RL" for k in ARM7})


def sample(keys, f):
    """Channels at frame f from [(frame, ease_into_this_key, {channel: value})]."""
    for (f0, _, p0), (f1, kind, p1) in zip(keys, keys[1:]):
        if f0 <= f <= f1:
            t = V.ease(kind, (f - f0) / float(f1 - f0))
            a, b = dict(REST, **p0), dict(REST, **p1)
            return {k: a[k] + (b[k] - a[k]) * t for k in REST}
    raise ValueError("frame %r outside the keys" % f)


def arm_x(c, side):
    return [math.radians(c[side + k]) for k in ARM7]


def pose_body(c):
    """Root, spine, head and both arms from channels; legs planted on their bind footprints."""
    c = dict(REST, **c)
    arm = arm_ob()
    pbs = arm.pose.bones
    V.apply_upper(dict(V.REST, **{k: c[k] for k in V.REST if k in c}), arm)
    q, r = W.axis_quat, math.radians
    pbs["spine"].rotation_quaternion = q(pbs["spine"], [(X, r(c["spine"])), (Z, r(c["spine_tw"]))])
    pbs["neck"].rotation_quaternion = q(pbs["neck"], [(X, r(c["neck"])), (Z, r(c["neck_tw"]))])
    for side in ("R", "L"):
        for b, qq in V.arm_quats(side, arm_x(c, side)):
            pbs[b].rotation_quaternion = qq
    V.plant_legs(c, arm)


# ---------------------------------------------------------------- the tools
#
# In Cut the axe is rigid in the right fist and the pickaxe rides on the back in Carry's
# sling. Both are expressed through their own joints (`axe` under chest, `pick` under
# hand.R), solved from the posed parents every frame -- round 19's Carry pinning, twice.
GRIP_R = Vector((0.517, 0.0, 0.534))     # 17 mm below the fist's centre, toward the fingers:
                                         # the cuff band needs the room (see WRIST_BEND)
HAFT_IN_HAND = Vector((0.0, 1.0, 0.0))  # ACROSS the fist: forward when the arm hangs
EDGE_MID = Vector((0.196, 0.0, 0.635))   # bit box 7, outer face, mid height, axe frame
EDGE_ENDS = (Vector((0.196, 0.0, 0.525)), Vector((0.196, 0.0, 0.745)))


def held_matrix(phi=0.0):
    """Axe frame -> the right hand's REST frame. The haft crosses the fist at right angles
    to the forearm. Held like the pickaxe (along the fist's Z, nearly the forearm's own
    axis) it ran up inside the sleeve -- 16..29 intersecting faces every frame -- and arm and
    haft read as one straight spear. `phi` (radians) angles the haft in the fist toward the
    fingers, as a hand at full reach does -- the solve picks it per key.

    The edge lies in the plane of haft and forearm, pointing past the fingers: down, for an axe
    carried at his side. It first faced the palm side (his midline), a quarter turn off in the
    fist, and every raised pose showed the edge to his left (Wolf)."""
    d = forearm_axis()
    return frame_matrix(GRIP_R, math.cos(phi) * HAFT_IN_HAND + math.sin(phi) * d,
                        math.cos(phi) * d - math.sin(phi) * HAFT_IN_HAND)


def axe_frame(hand_def, phi=0.0):
    """Axe frame -> armature, for a right-hand deform matrix."""
    return hand_def @ held_matrix(phi)


def pin(bone, want_def, parent):
    """Basis for `bone` so its deform matrix is want_def under a posed parent."""
    arm = arm_ob()
    bones, pbs = arm.data.bones, arm.pose.bones
    want = want_def @ bones[bone].matrix_local
    par = pbs[parent].matrix @ (bones[parent].matrix_local.inverted()
                                @ bones[bone].matrix_local)
    loc, rot, _ = (par.inverted() @ want).decompose()
    pbs[bone].location = loc
    pbs[bone].rotation_quaternion = rot


def pose_tools(grip=0.0):
    """Axe into the right fist at `grip` degrees, pickaxe onto the back."""
    arm = arm_ob()
    bones, pbs = arm.data.bones, arm.pose.bones
    bpy.context.view_layer.update()
    hand = pbs["hand.R"].matrix @ bones["hand.R"].matrix_local.inverted()
    chest = pbs["chest"].matrix @ bones["chest"].matrix_local.inverted()
    pin(AXE, axe_frame(hand, math.radians(grip)) @ stow_matrix().inverted(), "chest")
    pin(V.PICK, chest @ V.sling_matrix(), "hand.R")


def grip_for_edge(edge_point, haft, edge):
    """The grip that puts the edge midpoint on `edge_point` for this haft/edge direction."""
    r = frame_matrix(Vector(), haft, edge).to_3x3()
    return edge_point - r @ EDGE_MID


# The cuff band (sleeve box 3) ends 34 mm above the fist's centre and is nearly as wide as
# the fist, so a wrist folded more than ~20 degrees drives it into the haft -- the first
# solves used 75. The solve pays heavily past WRIST_BEND: the haft is turned by the shoulder
# and elbow and by forearm twist (free), not by folding the wrist.
WRIST_BEND = math.radians(20.0)
GRIP_MAX = math.radians(80.0)          # the haft angles at most this far toward the fingers:
                                       # at full reach the haft runs nearly in line with the arm
# Round 19's price on shoulder/wrist roll and twist, tripled: at 0.05 the downswing rolled
# the shoulder far enough to open the sleeve.R/strap.R seam 13.5 mm at frame 16.
CUT_POSTURE = 3.0 * V.POSTURE
# The shoulder's roll and twist pay double that again: they lift the sleeve cap off the strap
# and torso, and at 3x the rising recover still opened sleeve.R/strap.R 12 mm and
# sleeve.R/torso 16 mm on single frames.
SHOULDER_POSTURE = 2.0 * CUT_POSTURE


def forearm_axis():
    b = arm_ob().data.bones["hand.R"]
    return (b.tail_local - b.head_local).normalized()


def wrist_bend(x, chest):
    """Angle (radians) between the fist's axis and the forearm's, for arm angles x."""
    d = forearm_axis()
    _, _, mh = V.arm_fk("R", x, chest)
    _, _, m0 = V.arm_fk("R", list(x[:4]) + [0.0, 0.0, 0.0], chest)
    c = (mh.to_3x3() @ d).normalized().dot((m0.to_3x3() @ d).normalized())
    return math.acos(max(-1.0, min(1.0, c)))


def solve_axe(c, grip, haft, edge, start=None):
    """Right-arm angles (degrees) putting the axe at frame_matrix(grip, haft, edge) under the
    torso of channels c: coordinate descent over work_r19.arm_fk, with round 19's posture
    price on shoulder/wrist roll and twist and the WRIST_BEND price. Returns (angles, grip
    miss m, worst axis dot, wrist bend degrees)."""
    arm = arm_ob()
    pose_body(c)
    bpy.context.view_layer.update()
    chest = arm.pose.bones["chest"].matrix.copy()
    want = frame_matrix(grip, haft, edge).to_3x3()
    edge_at = frame_matrix(grip, haft, edge) @ EDGE_MID
    x = list(start) if start else arm_x(dict(REST, **c), "R") + [0.5]

    def cost(x):
        _, _, mh = V.arm_fk("R", x[:7], chest)
        m = axe_frame(mh, x[7])
        r = m.to_3x3()
        return (100.0 * (m.translation - grip).length_squared
                + 100.0 * (m @ EDGE_MID - edge_at).length_squared
                + 0.5 * (2.0 - (r @ Z).dot(want @ Z) - (r @ X).dot(want @ X))
                + SHOULDER_POSTURE * (x[1] ** 2 + x[2] ** 2) + CUT_POSTURE * (x[5] ** 2 + x[6] ** 2)
                + 50.0 * max(0.0, wrist_bend(x[:7], chest) - WRIST_BEND) ** 2)

    cbest, step = cost(x), 0.2
    while step > 1e-4:
        better = False
        for i in range(8):
            for d in (step, -step):
                y = list(x)
                y[i] += d
                if i == 3 and not 0.0 <= y[3] <= 2.6:      # the elbow does not hyperextend
                    continue
                if i == 7 and not 0.0 <= y[7] <= GRIP_MAX:
                    continue
                cy = cost(y)
                if cy < cbest:
                    x, cbest, better = y, cy, True
        if not better:
            step *= 0.5
    _, _, mh = V.arm_fk("R", x[:7], chest)
    m = axe_frame(mh, x[7])
    r = m.to_3x3()
    return ([math.degrees(v) for v in x], (m.translation - grip).length,
            min((r @ Z).dot(want @ Z), (r @ X).dot(want @ X)), math.degrees(wrist_bend(x[:7], chest)))


# ------------------------------------------------------------------------ Cut
#
# One chop per cycle, authored for ONE SECOND of game time (10 ticks at Normal, Wolf's 12.8
# Task 0.2): the first clip whose authored speed is its game speed. Five cycles fell a tree.
#
#   0  recover   axe wrenched out of the bark, rising to his right
#   8  cocked    fist up in front of the right shoulder, haft standing, edge forward;
#                chest turned 22 deg right
#  11  top       three frames' settle: the fuller wind-up an axe's weight earns
#  18  BITE      edge in the bark, waist-to-chest high, a diagonal blow from upper right
#                (phase 0.75, as Dig's strike)
#  21  drive     body commits through it, the blade sinks 15 mm deeper
#  24  = 0
#
# Not Dig: Dig's pick never leaves the plane x ~ 0.5 and lands on the floor ahead. Here the
# axe cocks out past his right shoulder, the chest turns 39 degrees through the swing, and
# the head comes over and across onto the trunk's RIGHT SIDE FACE at z 0.62, edge flat on
# the bark. Not the front face: an axe's edge runs parallel to its haft, his fist reaches
# only y ~0.45, so a haft long enough to touch the front face points straight at it and the
# edge goes in end-first (measured: 118 mm deep at one end, 92 mm short at the other).
#
# KEYS FIX THE AXE, THE ARM IS SOLVED. Each key says where the axe is; between keys its grip
# is eased along a line and its orientation slerped (axe_at), and the right arm is solved on
# every frame, warm-started from the last. Interpolating solved joint angles instead made
# the axe flail through the floor between keys (stages/cut-16).
#
# THE TRUNK. A pine's visible trunk is 0.60 m square (half-width 0.30 at z 0.4..0.9 on all
# four meshes, measured off assets/trees/*.glb), so its bark stands 1.30 m from his origin
# when he works from the next cell's centre. Wolf's call: author the bite at BARK_Y and have
# the client draw him CUT_OFFSET toward the trunk while he cuts.
# NOTE: one-handed. The left palm reaches 0.302 m from its shoulder and the haft is on his
# right; a two-handed grip would need the left fist 0.5 m across his body.
N = lambda *a: Vector(a).normalized()                    # noqa: E731
BARK_REAL = 1.30                       # m: trunk cell centre 1.60 - visible half-width 0.30
BARK_Y = 0.95                          # m: the bark plane this clip is authored against
CUT_OFFSET = BARK_REAL - BARK_Y        # m: how far the client draws him toward the trunk
BITE = 18
BITE_DEPTH = 0.015                     # m: how far the edge sinks into the bark
TRUNK_HALF = 0.30                      # m: the visible trunk's half-width, all four pines
EDGE_HALF = 0.5 * (EDGE_ENDS[1].z - EDGE_ENDS[0].z)   # the edge starts at the trunk's corner
BITE_AT = Vector((TRUNK_HALF - BITE_DEPTH, BARK_Y + EDGE_HALF, 0.62))
# The bite's axe, as the arm can hold it with the haft across the fist: haft forward along
# the trunk's side and a little up, edge into the side face leading left and down. Asking for
# the haft dead forward left no arm solution -- the edge came out facing RIGHT.
BITE_HAFT = N(0.08, 0.97, 0.22)
BITE_EDGE = N(-0.75, 0.21, -0.63)
DRIVE_AT = BITE_AT + Vector((-0.015, 0.0, -0.012))

RECOVER = dict(drop=-0.035, hips=-2, spine=-2, spine_tw=-4, chest=-1, chest_tw=-6, neck=4,
               neck_tw=6, head=1, beard=2, Lsx=10, Le=25)
COCKED = dict(drop=-0.020, hips=2, spine=4, spine_tw=-12, chest=4, chest_tw=-22, neck=-6,
              neck_tw=18, head=-2, beard=-3, Lsx=35, Le=30)
TOP = dict(COCKED, drop=-0.022, spine=5, spine_tw=-13, chest=5, chest_tw=-24, neck=-7,
           neck_tw=20, beard=-4, Lsx=38)
STRIKE = dict(drop=-0.045, hips=-8, spine=-11, spine_tw=8, chest=-6, chest_tw=14, neck=14,
              neck_tw=-14, head=3, beard=3, Lsx=-20, Le=20)
DRIVE = dict(STRIKE, drop=-0.050, hips=-9, spine=-12, spine_tw=9, chest=-7, chest_tw=15,
             neck=15, head=5, beard=9, Lsx=-25, Le=24)

# (frame, ease into this key, torso + left arm channels, where the axe is)
# Every key's edge faces forward or up -- toward the trunk or the coming blow. The first
# build gave `recover` an edge facing BACK, and Wolf saw the head reversed at the start and
# end of the loop (frames 22..4 slerp through it). Wolf again: while the axe is high the edge
# faces straight forward or a little down, never left -- it rolls left on the way down, into
# the bite. The cocked haft stands nearer upright for it: an edge square to a haft leaning
# back cannot face forward without swinging right. So the fist is raised in front of his
# right shoulder, the haft stands up behind it, and the edge looks at the trunk. The recover
# grip sits out at x 0.47: at 0.44 the rising shoulder opened sleeve.R/strap.R 12.2 mm (frame 1).
# The axe is ("grip", grip, haft, edge) or ("edge", edge point, haft, edge).
CUT_KEYS = [
    (0, None, RECOVER, ("grip", Vector((0.47, 0.24, 0.82)), N(0.30, 0.55, 0.78),
                        N(-0.6, 0.6, -0.3))),   # edge still toward the trunk, fresh out of it
    (8, "out", COCKED, ("grip", Vector((0.38, 0.08, 0.98)), N(0.30, -0.25, 0.92),
                        N(0.0, 1.0, -0.5))),     # edge forward, a little down
    (11, "smooth", TOP, ("grip", Vector((0.40, 0.04, 1.00)), N(0.32, -0.35, 0.88),
                         N(0.0, 1.0, -0.5))),
    (BITE, "in", STRIKE, ("edge", BITE_AT, BITE_HAFT, BITE_EDGE)),
    (21, "out", DRIVE, ("edge", DRIVE_AT, BITE_HAFT, BITE_EDGE)),
]
STARTS = ([20, 0, 0, 20, -90, 0, -30], [40, 0, 0, 40, -90, 0, 0], [150, -20, 0, 40, -160, 0, 0],
          [100, 0, 0, 60, -120, 0, 0], [0, 0, 0, 100, -160, 20, -30], [30, 0, 0, 30, 0, 0, 0],
          [60, 0, 0, 30, 40, 0, 0], [140, 0, 0, 30, 60, 0, 0], [140, 0, -30, 20, 90, 0, 0])
CUT_POSES = []                         # [(frame, ease, torso channels)]
AXE_PATH = []                          # [(frame, ease, axe frame matrix)]
CUT_SOL = {}                           # frame -> solved right arm (7 angles + grip, radians)
CUT_MISS = {}                          # frame -> (grip miss m, worst axis dot, wrist bend deg)


def key_frames():
    """Resolve CUT_KEYS into torso channel keys and axe-frame keys. Frame 24 repeats 0."""
    del CUT_POSES[:]
    del AXE_PATH[:]
    for f, kind, torso, spec in CUT_KEYS:
        if spec[0] == "edge":
            _, point, haft, edge = spec
            grip = grip_for_edge(point, haft, edge)
        else:
            _, grip, haft, edge = spec
        CUT_POSES.append((f, kind, dict(REST, **torso)))
        AXE_PATH.append((f, kind, frame_matrix(grip, haft, edge)))
    CUT_POSES.append((CYCLE, "smooth", CUT_POSES[0][2]))
    AXE_PATH.append((CYCLE, "smooth", AXE_PATH[0][2]))


def axe_at(f):
    """The axe frame at frame f: grip eased along a line, orientation slerped -- the axe's own
    path, so the head travels where the keys say rather than wherever an interpolation of
    joint angles happens to carry it (the first build's flailing, stages/cut-16)."""
    for (f0, _, m0), (f1, kind, m1) in zip(AXE_PATH, AXE_PATH[1:]):
        if f0 <= f <= f1:
            t = V.ease(kind, (f - f0) / float(f1 - f0))
            q0, q1 = m0.to_quaternion(), m1.to_quaternion()
            if q0.dot(q1) < 0.0:
                q1 = -q1
            m = q0.slerp(q1, t).to_matrix().to_4x4()
            m.translation = m0.translation.lerp(m1.translation, t)
            return m
    raise ValueError("frame %r outside the keys" % f)


def solve_frame(f):
    """The right arm at frame f, warm-started from frame f-1 (frame 0 from the best of STARTS),
    cached, so frame 24 re-uses frame 0's solution and the loop closes exactly."""
    if f in CUT_SOL:
        return CUT_SOL[f]
    torso = sample(CUT_POSES, f)
    m = axe_at(f)
    grip, haft, edge = m.translation, m.to_3x3() @ Z, m.to_3x3() @ X
    fresh = [[math.radians(v) for v in s] + [0.9] for s in STARTS]
    starts = [CUT_SOL[f - 1]] if f - 1 in CUT_SOL else fresh
    best = None
    for attempt in (starts, fresh):
        for s in attempt:
            x, miss, dot, bend = solve_axe(torso, grip, haft, edge, start=s)
            score = 100.0 * miss + (1.0 - dot)
            if best is None or score < best[3]:
                best = (x, miss, dot, score, bend)
        # a warm start that faces the axe wrong (the target turned away from the last frame's
        # branch) is retried from every fresh start: a pop beats a wrong-facing axe
        if best[1] < 0.030 and best[2] > 0.90:
            break
    CUT_SOL[f] = [math.radians(v) for v in best[0]]
    CUT_MISS[f] = (best[1], best[2], best[4])
    return CUT_SOL[f]


def solve_keys():
    """Every frame's right arm, in order, so each warm-starts from the last."""
    W.prepare()
    arm_ob().animation_data.action = None
    key_frames()
    CUT_SOL.clear()
    CUT_MISS.clear()
    for f in range(0, CYCLE):
        solve_frame(f)
    worst = max(CUT_MISS.items(), key=lambda kv: kv[1][0])
    print("  arm solved on %d frames: worst grip miss %.1f mm (frame %d), worst axis dot %.3f,"
          " worst wrist bend %.0f deg"
          % (CYCLE, worst[1][0] * 1000.0, worst[0], min(v[1] for v in CUT_MISS.values()),
             max(v[2] for v in CUT_MISS.values())))


def pose_cut(f):
    c = dict(sample(CUT_POSES, f))
    x = solve_frame(f)
    c.update({"R" + k: math.degrees(v) for k, v in zip(ARM7, x[:7])})
    pose_body(c)
    pose_tools(math.degrees(x[7]))


def build_cut():
    W.prepare()
    add_axe()
    solve_keys()
    act = V.key_clip("Cut", pose_cut, bones=BONES, located=("root", V.PICK, AXE))
    attach("Cut")
    bpy.context.scene.frame_set(BITE)
    print("BITE frame %d (phase %.3f): edge %s" % (BITE, BITE / float(CYCLE), edge_now()))
    return act


def edge_now():
    """The blade edge as posed now, off the evaluated mesh: (low end, mid, high end)."""
    vs = W.evald(AXE_PART)
    rest = bpy.data.objects[AXE_PART].data.vertices
    m = stow_matrix().inverted()            # bind mesh -> axe frame
    face = [vs[i] for i in V.box_ids(AXE_PART, 7)
            if abs((m @ rest[i].co).x - EDGE_MID.x) < 1e-4]
    lo, hi = min(face, key=lambda v: v.z), max(face, key=lambda v: v.z)
    return lo, sum(face, Vector()) / len(face), hi


# ------------------------------------------------------------- watching it
def temp_trunk(name="TMP_r20_trunk"):
    """A stand-in for the pine's visible trunk where this clip is authored to meet it: 0.60 m
    square, near face at BARK_Y. NEVER saved: remove_temp() deletes it and checks() refuses
    to pass while it exists."""
    import bmesh
    ob = bpy.data.objects.get(name)
    if ob is None:
        me = bpy.data.meshes.new(name)
        bm = bmesh.new()
        bmesh.ops.create_cube(bm, size=1.0)
        bm.to_mesh(me)
        bm.free()
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (
            0.42, 0.36, 0.29, 1.0)
        me.materials.append(mat)
        ob = bpy.data.objects.new(name, me)
        bpy.context.scene.collection.objects.link(ob)
    ob.scale = (2 * TRUNK_HALF, 2 * TRUNK_HALF, 2.4)
    ob.location = (0.0, BARK_Y + TRUNK_HALF, 1.2)
    return ob


def remove_temp():
    for ob in [o for o in bpy.data.objects if o.name.startswith("TMP_r20")]:
        me = ob.data
        bpy.data.objects.remove(ob)
        if me is not None and me.users == 0:
            for m in list(me.materials):
                if m is not None and m.users <= 1:
                    bpy.data.materials.remove(m)
            bpy.data.meshes.remove(me)


# ---------------------------------------------------------------- the proofs
#
# Off the EVALUATED meshes with the clip bound (action AND slot, via attach()).
PICKAXE, LANTERN = "r17_pickaxe", "r17_lantern"


def gap(a, b):
    """(surface gap mm, intersecting face pairs) between two posed parts."""
    av, at = W.evald_tree(a)
    bv, bt = W.evald_tree(b)
    return V.surface_gap(av, at, bv, bt) * 1000.0, len(at.overlap(bt))


def body_parts():
    coll = bpy.data.collections[D.COLL]
    return sorted(o.name for o in coll.objects if o.type == 'MESH')


def clearance(name):
    """Per frame: the axe's lowest z, its nearest approach to the head assembly, to the
    pickaxe and to the rest of the body (the part it is strapped to or held by excluded,
    and named), with intersecting-face counts; the pickaxe's and lantern's approach to the
    head. The head columns are what round 18 sec.5 warned about: shoulder roll and wrist
    twist drive a shaft into the skull."""
    attach(name)
    sc = bpy.context.scene
    held = name == "Cut"
    skip = {AXE_PART, PICKAXE} | set(W.HEAD_PARTS) | ({"r17_glove.R"} if held else {"r17_pack"})
    others = [p for p in body_parts() if p not in skip]
    print("clearance, %s -- axe %s; mm are surface gaps, x N = intersecting face pairs"
          % (name, "in the right fist" if held else "strapped to the pack"))
    print("  %5s %9s %10s %9s %22s %9s %10s %10s %8s" % (
        "frame", "axe min z", "axe-head", "axe-pick", "axe-body (nearest)", "axe-pack",
        "pick-head", "lant-head", "pick z"))
    bad, rows = [], []
    for f in range(0, CYCLE):
        sc.frame_set(f)
        axe_z = min(v.z for v in W.evald(AXE_PART))
        pick_z = min(v.z for v in W.evald(PICKAXE))
        ah = min((gap(AXE_PART, h) for h in W.HEAD_PARTS), key=lambda g: g[0])
        ap = gap(AXE_PART, PICKAXE)
        body = min(((gap(AXE_PART, p), p) for p in others), key=lambda g: g[0][0])
        apk = gap(AXE_PART, "r17_pack")
        ph = min(gap(PICKAXE, h)[0] for h in W.HEAD_PARTS)
        lh = min(gap(LANTERN, h)[0] for h in W.HEAD_PARTS)
        rows.append((axe_z, ah[0], ap[0], body[0][0], ph, lh, pick_z))
        print("  %5d %9.4f %7.1f x%d %6.1f x%d %9.1f x%d %-9s %7.1f %10.1f %10.1f %8.4f" % (
            f, axe_z, ah[0], ah[1], ap[0], ap[1], body[0][0], body[0][1], body[1][4:],
            apk[0], ph, lh, pick_z))
        if axe_z < 0.0 or pick_z < 0.0 or ah[1] or ah[0] < 1.0 or ap[1] or body[0][1] \
                or ph < 1.0 or lh < 1.0:
            bad.append(f)
    cols = list(zip(*rows))
    print("  worst: axe z %.4f, axe-head %.1f, axe-pick %.1f, axe-body %.1f, pick-head %.1f,"
          " lantern-head %.1f mm, pick z %.4f"
          % (min(cols[0]), min(cols[1]), min(cols[2]), min(cols[3]), min(cols[4]),
             min(cols[5]), min(cols[6])))
    print("  %s" % ("PASS" if not bad else "FAIL at frames %s" % bad))
    return bad


def foot_slide_planted(name):
    """Planted clip: every sole vertex against its frame-0 world position, frames 0..24."""
    attach(name)
    sc = bpy.context.scene
    ids = {s: W.sole_ids(s) for s in ("L", "R")}
    ref, worst = {}, 0.0
    print("foot slide, %s -- planted: sole vertices against their frame-0 world positions"
          % name)
    print("  %5s %12s %12s %10s %10s" % ("frame", "L drift mm", "R drift mm", "L min z",
                                         "R min z"))
    for f in range(0, CYCLE + 1):
        sc.frame_set(f)
        row, zs = [f], []
        for s in ("L", "R"):
            allv = W.evald("r17_boot." + s)
            vs = [allv[i] for i in ids[s]]
            ref.setdefault(s, vs)
            d = max((a - b).length for a, b in zip(vs, ref[s])) * 1000.0
            worst = max(worst, d)
            row.append(d)
            zs.append(min(v.z for v in allv))
        print("  %5d %12.4f %12.4f %10.5f %10.5f" % tuple(row + zs))
    print("  WORST SLIDE %.4f mm" % worst)
    return worst


def bite_report():
    attach("Cut")
    bpy.context.scene.frame_set(BITE)
    lo, mid, hi = edge_now()
    print("BITE frame %d of %d (phase %.3f), rig metres:" % (BITE, CYCLE, BITE / float(CYCLE)))
    for tag, v in (("edge low end", lo), ("edge middle", mid), ("edge high end", hi)):
        print("  %-14s (%.3f, %.3f, %.3f)" % ((tag,) + tuple(v)))
    print("  bark: the trunk's right side face x = %.3f for y %.3f .. %.3f (authored, near"
          " face y = BARK_Y = %.2f); the real bark is %.2f m out -> CUT_OFFSET %.2f m"
          % (TRUNK_HALF, BARK_Y, BARK_Y + 2 * TRUNK_HALF, BARK_Y, BARK_REAL, CUT_OFFSET))
    print("  edge depth into the bark: %.1f .. %.1f mm"
          % tuple(sorted(((TRUNK_HALF - v.x) * 1000.0) for v in (lo, hi))))
    return lo, mid, hi


def geometry_hash(names=None):
    """sha256 per mesh part over its bind vertices and faces, and over all of them."""
    out, whole = {}, hashlib.sha256()
    for n in names or body_parts():
        h = hashlib.sha256()
        me = bpy.data.objects[n].data
        for v in me.vertices:
            h.update(("%r %r %r|" % tuple(v.co)).encode())
        for p in me.polygons:
            h.update(("%s|" % list(p.vertices)).encode())
        out[n] = h.hexdigest()
        whole.update(out[n].encode())
    return out, whole.hexdigest()


def checks():
    bar = "=" * 78
    print(bar)
    acts = sorted(a.name for a in bpy.data.actions)
    print("actions: %s  %s" % (acts, "OK" if acts == list(ACTIONS) else "<-- FAIL"))
    print("joints: %d; parts: %d" % (len(arm_ob().data.bones), len(body_parts())))
    for name in OLD:
        h20 = fcurve_hash(name, V.BONES)
        print("%-5s round-19 curves %s %s | all curves %s"
              % (name, h20[:16], "UNCHANGED" if h20 == ROUND19[name] else "CHANGED <-- FAIL",
                 fcurve_hash(name)))
    print("Cut   fcurve hash %s" % fcurve_hash("Cut"))
    for name in ACTIONS:
        act = bpy.data.actions[name]
        curves = W.fcurves(act)
        print("%-5s frames %d..%d at %d fps, slots %d, fake user %s, %d fcurves, all LINEAR %s"
              % (name, act.frame_range[0], act.frame_range[1], bpy.context.scene.render.fps,
                 len(act.slots), act.use_fake_user, len(curves),
                 all(k.interpolation == 'LINEAR' for fc in curves for k in fc.keyframe_points)))
    print(bar)
    bite_report()
    print(bar)
    slide = foot_slide_planted("Cut")
    print(bar)
    clear = {n: clearance(n) for n in ACTIONS}
    print(bar)
    ip = V.in_place("Cut")
    print(bar)
    ls = V.loop_seam("Cut")
    print(bar)
    by_design = {(AXE_PART, "r17_pack"): "the axe leaves the pack for the fist",
                 ("r17_glove.R", PICKAXE): "the pickaxe rides on the back"}
    fails = V.seam_clip("Cut", skip=by_design)
    print(bar)
    temp = [o.name for o in bpy.data.objects if o.name.startswith("TMP_")]
    print("SUMMARY  slide %.4f mm | clearance %s | in-place %.1e m | loop %.1e | seams %d"
          " fails | temp objects %s"
          % (slide, " ".join("%s %s" % (n, "ok" if not b else "FAIL") for n, b in clear.items()),
             ip, ls, fails, temp or "none"))
    detach()


def detach():
    V.detach()


# ------------------------------------------------------------------- renders
CUT_FRAMES = [0, 5, 8, 11, 14, 16, 17, 18, 21, 23]


def renders():
    """Cycles key strips to renders/r20/ via render_r18.sheet, as round 19 did, with the
    stand-in trunk where the clip meets it (hidden from the front, where it would cover
    him). `Dig` and `Cut` are rendered from the same cameras so the two can be compared.
    Render settings, scene camera and frame range are put back; the strip camera and the
    trunk are deleted: none of it is saved into the .blend."""
    import render_r18 as R
    importlib.reload(R)
    R.OUT = os.path.join(os.path.dirname(HERE), "renders", "r20")
    R.TARGET = Vector((0.0, 0.40, 0.75))
    R.SPAN = 2.3
    R.COLW, R.COLH = 380, 440
    sc = bpy.context.scene
    keep = (sc.render.engine, sc.render.resolution_x, sc.render.resolution_y,
            sc.render.filepath, sc.camera, sc.render.film_transparent,
            sc.render.image_settings.color_mode, sc.frame_start, sc.frame_end,
            sc.frame_current, sc.render.fps)
    keep_cy = (sc.cycles.device, sc.cycles.samples, sc.cycles.use_denoising)
    try:
        trunk = temp_trunk()
        attach("Cut")
        R.sheet("cut-keys-side", CUT_FRAMES, 90)
        R.sheet("cut-keys-threequarter", CUT_FRAMES, 135, 10)
        R.sheet("cut-keys-high", CUT_FRAMES, 60, 40)
        trunk.hide_render = True
        R.sheet("cut-keys-front", CUT_FRAMES, 180)
        R.sheet("cut-cycle-back", list(range(0, CYCLE, 2)), 0, 15)
        trunk.hide_render = False
        attach("Dig")
        R.sheet("dig-keys-high", [0, 5, 8, 10, 12, 14, 16, 17, 18, 21], 60, 40)
        attach("Walk")
        R.sheet("walk-axe-stowed", [0, 6, 12, 18], 30, 10)
    finally:
        (sc.render.engine, sc.render.resolution_x, sc.render.resolution_y,
         sc.render.filepath, sc.camera, sc.render.film_transparent,
         sc.render.image_settings.color_mode, sc.frame_start, sc.frame_end,
         sc.frame_current, sc.render.fps) = keep
        sc.cycles.device, sc.cycles.samples, sc.cycles.use_denoising = keep_cy
        cam = bpy.data.objects.get("cam_r18")
        if cam is not None:
            data = cam.data
            bpy.data.objects.remove(cam)
            bpy.data.cameras.remove(data)
        remove_temp()
