"""Round 19 -- the work clips `Dig` and `Carry` for the r17 voxel dwarf.

The third link of the chain dwarf_r17.py -> walk_r18.py -> work_r19.py. Run it in the live
session against SM_VoxelDwarf_Miner01.blend, after `Walk` exists:

    exec(open(r"src-assets/blender/work_r19.py").read())
    build_dig(); build_carry(); checks(); detach()
    renders("dig"); renders("carry")          # Cycles strips to renders/r19/

Both clips follow walk_r18's rules: frames 0..24 at 24 fps, frame 24 == frame 0, every bone
keyed on every frame, LINEAR; in place (`root` bobs vertically and nothing else); saved
unassigned on a fake user. `Walk` itself is never touched -- fcurve_hash("Walk") before and
after.

This round changes the RIG, on Wolf's call: add_pick_bone() adds a 20th joint `pick` under
`hand.R` and moves the pickaxe's weights onto it, so Carry can sling the pickaxe on his back
while Dig keeps it in his hand. No vertex moves; export_dwarf.JOINTS carries `pick`.
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
import walk_r18 as W                    # noqa: E402
importlib.reload(W)

FPS = W.FPS
CYCLE = W.CYCLE
X, Y, Z = W.X, W.Y, W.Z
# Round 19 adds a 20th joint, `pick`: see add_pick_bone(). Dig and Carry key it; Walk does
# not and is not touched -- the glTF exporter samples every joint for every clip, so Walk
# ships `pick` at rest without a curve of its own.
PICK = "pick"
BONES = W.BONES + [PICK]
ACTIONS = ("Walk", "Dig", "Carry")
STAGES = os.path.join(os.path.dirname(HERE), "renders", "r19", "stages")


# ------------------------------------------------------------------ plumbing
def arm_ob():
    return W.arm_ob()


def add_pick_bone():
    """The 20th joint: `pick`, child of `hand.R`, and the pickaxe weighted to it.

    Wolf's call, round 19: during Carry both hands hold the stone and the pickaxe rides on
    his back, while Dig needs it in his right hand. A rigid one-joint weight cannot be both,
    so the pickaxe gets a joint of its own. At rest `pick` sits exactly where the pickaxe
    already was, so with `pick` unposed nothing moves: Walk and Dig look as they did.

    The bone's head is the grip (the shaft's centre line at the fist), its tail runs up the
    shaft, so its own Y axis IS the shaft. Idempotent: the chain re-runs it on a cold build.
    NOTE: export_dwarf.JOINTS carries `pick` from this round; the forge side owns the GLB.
    """
    arm = arm_ob()
    if PICK not in arm.data.bones:
        bpy.context.view_layer.objects.active = arm
        prev = arm.mode
        bpy.ops.object.mode_set(mode='EDIT')
        eb = arm.data.edit_bones.new(PICK)
        eb.head = (0.506, 0.0, 0.547)
        eb.tail = (0.506, 0.0, 0.647)
        eb.roll = 0.0
        eb.parent = arm.data.edit_bones["hand.R"]
        eb.use_connect = False
        eb.use_deform = True
        bpy.ops.object.mode_set(mode=prev if prev != 'EDIT' else 'OBJECT')
    vg = bpy.data.objects["r17_pickaxe"].vertex_groups
    if "hand.R" in vg and PICK not in vg:
        vg["hand.R"].name = PICK
    pb = arm.pose.bones[PICK]
    pb.rotation_mode = 'QUATERNION'
    print("joint %r: parent %s, %d joints; r17_pickaxe groups %s"
          % (PICK, arm.data.bones[PICK].parent.name, len(arm.data.bones),
             [g.name for g in vg]))
    key_walk_pick()


def key_walk_pick():
    """`pick` keyed at rest on every frame of Walk.

    Without it, Walk has no curve for `pick`, and in Blender an un-keyed bone keeps
    whatever pose it last had: play Carry, switch to Walk, and the pickaxe stays slung on
    his back while his hand walks empty (Wolf caught it). The GLB was never affected -- the
    exporter samples `pick` at rest -- but the .blend should not lie either. Walk's own 79
    curves are untouched: fcurve_hash("Walk", W.BONES) is round 18's hash."""
    act = bpy.data.actions.get("Walk")
    if act is None or any('"%s"' % PICK in fc.data_path for fc in W.fcurves(act)):
        return
    arm = arm_ob()
    keep = arm.animation_data.action if arm.animation_data else None
    attach("Walk")
    rest_pick()
    pb = arm.pose.bones[PICK]
    for f in range(0, CYCLE + 1):
        pb.keyframe_insert("rotation_quaternion", frame=f, group=PICK)
        pb.keyframe_insert("location", frame=f, group=PICK)
    for fc in W.fcurves(act):
        if '"%s"' % PICK in fc.data_path:
            for kp in fc.keyframe_points:
                kp.interpolation = 'LINEAR'
            fc.update()
    arm.animation_data.action = keep
    if keep is not None:
        arm.animation_data.action_slot = keep.slots[0]
    print("Walk: `pick` keyed at rest, frames 0..%d; round-18 curves hash %s"
          % (CYCLE, fcurve_hash("Walk", W.BONES)))


def attach(name):
    """Bind action `name` AND its slot. With three actions in the file a missing slot is
    the likely way to measure the wrong clip, so this is the only way checks bind one."""
    arm = arm_ob()
    if arm.animation_data is None:
        arm.animation_data_create()
    act = bpy.data.actions[name]
    arm.animation_data.action = act
    arm.animation_data.action_slot = act.slots[0]
    return act


def detach():
    """Leave the figure at bind, every action kept as data on a fake user."""
    arm = arm_ob()
    if arm.animation_data is not None:
        arm.animation_data.action = None
    W.bind_pose()
    print("detached: pose is bind; actions %s"
          % ", ".join("%s(fake user %s)" % (a.name, a.use_fake_user)
                      for a in sorted(bpy.data.actions, key=lambda a: a.name)))


def fcurve_hash(name, bones=None):
    """sha256 over every key of every F-curve of action `name`, in a fixed order.

    `bones` restricts it to those bones' curves: fcurve_hash("Walk", W.BONES) is round 18's
    Walk exactly -- the hash it had before round 19 added the `pick` rest keys."""
    h = hashlib.sha256()
    curves = sorted((fc for fc in W.fcurves(bpy.data.actions[name])
                     if bones is None or fc.data_path.split('"')[1] in bones),
                    key=lambda fc: (fc.data_path, fc.array_index))
    for fc in curves:
        h.update(("%s[%d]" % (fc.data_path, fc.array_index)).encode())
        for kp in fc.keyframe_points:
            h.update(("%r %r %s|" % (kp.co[0], kp.co[1], kp.interpolation)).encode())
    return h.hexdigest()


def stage_shot(name):
    """Save the 3D viewport as Wolf sees it to renders/r19/stages/<name>.png."""
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
# A pose is a dict of channels, angles in DEGREES about armature-space axes (walk_r18's
# convention: these bones point UP, so negative X leans the spine forward; the arms hang
# DOWN, so positive X swings an arm forward). Every arm channel is about world X only, so
# the pickaxe and lantern stay in their own sagittal planes at |x| ~ 0.5 and can never
# swing inward at the skull (round 18 sec.5). `drop` is root z in metres.
REST = dict(drop=0.0, hips=0.0, spine=0.0, chest=0.0, chest_tw=0.0, neck=0.0, head=0.0,
            beard=0.0, sR=0.0, eR=0.0, wR=0.0, sL=0.0, eL=0.0, wL=0.0)


def ease(kind, t):
    t = min(1.0, max(0.0, t))
    if kind == "in":                    # accelerate into the key: the strike
        return t * t
    if kind == "out":                   # decelerate into the key
        return 1.0 - (1.0 - t) * (1.0 - t)
    return t * t * (3.0 - 2.0 * t)


def sample(keys, f):
    """Channels at frame f from [(frame, ease_into_this_key, {channel: value})]."""
    for (f0, _, p0), (f1, kind, p1) in zip(keys, keys[1:]):
        if f0 <= f <= f1:
            t = ease(kind, (f - f0) / float(f1 - f0))
            a, b = dict(REST, **p0), dict(REST, **p1)
            return {k: a[k] + (b[k] - a[k]) * t for k in REST}
    raise ValueError("frame %r outside the keys" % f)


def rad(d):
    return math.radians(d)


def apply_upper(c, arm):
    """Root, spine chain and arms from channel dict c. Legs are not touched."""
    pbs = arm.pose.bones
    for name in BONES:
        pbs[name].rotation_mode = 'QUATERNION'
    inv = pbs["root"].bone.matrix_local.to_3x3().inverted()
    pbs["root"].location = inv @ Vector((0.0, 0.0, c["drop"]))
    pbs["root"].rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
    q = W.axis_quat
    pbs["hips"].rotation_quaternion = q(pbs["hips"], [(X, rad(c["hips"]))])
    pbs["spine"].rotation_quaternion = q(pbs["spine"], [(X, rad(c["spine"]))])
    pbs["chest"].rotation_quaternion = q(pbs["chest"], [(X, rad(c["chest"])),
                                                        (Z, rad(c["chest_tw"]))])
    pbs["neck"].rotation_quaternion = q(pbs["neck"], [(X, rad(c["neck"]))])
    pbs["head"].rotation_quaternion = q(pbs["head"], [(X, rad(c["head"]))])
    pbs["beard"].rotation_quaternion = q(pbs["beard"], [(X, rad(c["beard"]))])
    for side in ("R", "L"):
        for bone, ch in (("shoulder.", "s"), ("elbow.", "e"), ("hand.", "w")):
            pb = pbs[bone + side]
            pb.rotation_quaternion = q(pb, [(X, rad(c[ch + side]))])


def plant_legs(c, arm):
    """Both soles flat on their bind footprints, whatever the root and pelvis do.

    walk_r18's two-link solve with the ankle held at its bind position and zero foot
    pitch: the planted foot cannot slide because it is never asked to move.
    """
    if W.LEG is None:
        W.prepare()
    pbs = arm.pose.bones
    alpha = rad(c["hips"])
    for side in ("L", "R"):
        L = W.LEG[side]
        ank = arm.data.bones["foot." + side].head_local
        hy, hz = W.hip_pivot(L, alpha, c["drop"])
        t1, t2, t3 = W.leg_angles(ank.y, ank.z, 0.0, hy, hz, alpha, L)
        pbs["hip." + side].rotation_quaternion = Quaternion(X, t1)
        pbs["knee." + side].rotation_quaternion = Quaternion(X, t2)
        pbs["foot." + side].rotation_quaternion = Quaternion(X, t3)


def pose_channels(c, update=True):
    """Pose the rig from channels. update=False while keying: with an action bound, a
    depsgraph update flushes the action's old values back over the pose before it is keyed."""
    arm = arm_ob()
    apply_upper(c, arm)
    plant_legs(c, arm)
    if update:
        bpy.context.view_layer.update()


# ---------------------------------------------------------------- prop points
def box_ids(obname, box):
    ob = bpy.data.objects[obname]
    starts = list(ob["box_starts"]) + [len(ob.data.vertices)]
    return list(range(starts[box], starts[box + 1]))


def pick_head():
    """(+y blade tip, head centre) of the pickaxe as posed now, world metres.

    The blade's +y tip (box 9) is the end that points DOWN once the shaft is swung
    forward past horizontal, so it is the end that strikes; box 2 is the eye of the head.
    """
    vs = W.evald("r17_pickaxe")
    tip_box = [vs[i] for i in box_ids("r17_pickaxe", 9)]
    eye = [vs[i] for i in box_ids("r17_pickaxe", 2)]
    tip = min(tip_box, key=lambda v: v.z)
    centre = sum(eye, Vector()) / len(eye)
    return tip, centre


# ------------------------------------------------------------------- the keyer
def key_clip(name, pose_at, bones=None, located=("root", PICK)):
    """Replace action `name` with frames 0..CYCLE posed by pose_at(f), LINEAR, fake user.

    pose_at(f) sets every pose bone for frame f % CYCLE; frame CYCLE re-poses frame 0, so
    the closing key is the opening key by construction, not by copy.

    Two passes. The poses are taken with NO action bound, so pose_at may update the
    depsgraph to measure what it built (Carry pins the pickaxe to the pack off the evaluated
    hand); with an action bound that update would flush the action's values over the pose.
    Then the action is bound and the recorded values keyed, with no update in between.
    `bones` and `located` default to this round's rig: round 20 passes its 21st joint `axe`,
    which moves through space and so needs location keys as `root` and `pick` do.
    """
    bones = BONES if bones is None else bones
    arm = arm_ob()
    bpy.context.view_layer.objects.active = arm
    if arm.animation_data is None:
        arm.animation_data_create()
    arm.animation_data.action = None
    poses = []
    for f in range(0, CYCLE + 1):
        pose_at(f % CYCLE)
        pose = {b: (arm.pose.bones[b].rotation_quaternion.copy(),
                    arm.pose.bones[b].location.copy()) for b in bones}
        # q and -q are one rotation but LINEAR keys between them spin the long way round.
        # A decomposed matrix (the pickaxe sling) can come back with either sign.
        if poses:
            for b, (qq, loc) in pose.items():
                if qq.dot(poses[-1][b][0]) < 0.0:
                    pose[b] = (-qq, loc)
        poses.append(pose)

    old = bpy.data.actions.get(name)
    if old is not None:
        old.use_fake_user = False
        bpy.data.actions.remove(old)
    act = bpy.data.actions.new(name)
    arm.animation_data.action = act
    sc = bpy.context.scene
    sc.render.fps = FPS
    sc.frame_start, sc.frame_end = 0, CYCLE
    for f, pose in enumerate(poses):
        for b in bones:
            pb = arm.pose.bones[b]
            pb.rotation_quaternion, pb.location = pose[b]
            pb.keyframe_insert("rotation_quaternion", frame=f, group=b)
            if b in located:
                pb.keyframe_insert("location", frame=f, group=b)
    curves = W.fcurves(act)
    for fc in curves:
        for kp in fc.keyframe_points:
            kp.interpolation = 'LINEAR'
        fc.update()
    act.use_fake_user = True
    arm.animation_data.action_slot = act.slots[0]
    bpy.context.view_layer.update()
    print("ACTION %s -- frames 0..%d at %d fps, %d fcurves, %d keys"
          % (name, CYCLE, FPS, len(curves), sum(len(fc.keyframe_points) for fc in curves)))
    return act


# ------------------------------------------------------------------------ Dig
#
# One pick swing per cycle, authored for HALF A SECOND of game time (5 ticks at Normal):
# the client maps one 0..24 cycle onto one 5-tick work run. So there is no idle hold --
# frame 0 is already the pick coming up out of the last strike.
#
#   0  lift      pick rising out of the rock, weight coming up
#  10  cocked    pick up and back over the right shoulder, body leaning back
#  12  top       a two-frame settle: the anticipation before the drop
#  18  STRIKE    blade tip on the floor ~0.65 m ahead (phase 0.75), accelerating in
#  20  bite      body follows through 2 frames, the pick stays put in the rock
#  24  = 0
#
# The right arm swings about world X only, so the pick stays in the plane x ~ 0.5 and
# cannot reach the skull (|x| <= 0.23). The wrist does the work of turning the shaft
# from "up and back" to "forward and down": a cocked wrist, not a shoulder roll.
# NOTE: one-handed. The lantern hand braces forward on the strike but never takes the
# shaft: the shaft is 0.5 m to his right and the left hand would have to cross his body.
STRIKE = 18
DIG_KEYS = [
    (0, None, dict(drop=-0.035, hips=-2, spine=-3, chest=-2, chest_tw=-2, neck=3, head=2,
                   beard=2, sR=75, eR=45, wR=-150, sL=12, eL=25)),
    (10, "out", dict(drop=-0.020, hips=2, spine=6, chest=6, chest_tw=-8, neck=-4, head=-4,
                     beard=-3, sR=150, eR=40, wR=-170, sL=-5, eL=15)),
    (12, "smooth", dict(drop=-0.022, hips=2.5, spine=7, chest=7, chest_tw=-9, neck=-5,
                        head=-4, beard=-4, sR=154, eR=44, wR=-170, sL=-6, eL=15)),
    (18, "in", dict(drop=-0.040, hips=-6, spine=-10, chest=-8, chest_tw=6, neck=8, head=6,
                    beard=3, sR=20, eR=10, wR=-120, sL=25, eL=30)),
    # the bite: everything that carries the pick holds still -- it is in the rock, and
    # any follow-through on root/pelvis/spine/arm drives the blade under the floor
    (20, "out", dict(drop=-0.040, hips=-6, spine=-10, chest=-8, chest_tw=6, neck=9, head=8,
                     beard=8, sR=20, eR=10, wR=-120, sL=28, eL=33)),
]
DIG_KEYS.append((CYCLE, "smooth", DIG_KEYS[0][2]))


def pose_dig(f):
    pose_channels(sample(DIG_KEYS, f), update=False)
    rest_pick()


def rest_pick():
    """`pick` keyed at rest, so a Dig after a Carry never inherits a pickaxe on the back."""
    pb = arm_ob().pose.bones[PICK]
    pb.rotation_mode = 'QUATERNION'
    pb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
    pb.location = (0.0, 0.0, 0.0)


def build_dig():
    W.prepare()
    add_pick_bone()
    act = key_clip("Dig", pose_dig)
    attach("Dig")
    bpy.context.scene.frame_set(STRIKE)
    tip, eye = pick_head()
    print("STRIKE frame %d (phase %.3f): blade tip (%.3f, %.3f, %.3f), head eye (%.3f, %.3f, %.3f)"
          % ((STRIKE, STRIKE / float(CYCLE)) + tuple(tip) + tuple(eye)))
    return act


# ------------------------------------------------------------- watching it
def strip(name, frames, action=None, crop=(0.30, 0.70), step=3):
    """Viewport filmstrip: one screenshot per frame, the middle of each kept, stitched,
    every `step`-th pixel. What Wolf's viewport shows, frame by frame, in one file."""
    import numpy as np
    if action is not None:
        attach(action)
    sc = bpy.context.scene
    win = bpy.context.window_manager.windows[0]
    area = next(a for a in win.screen.areas if a.type == 'VIEW_3D')
    cols = []
    for f in frames:
        sc.frame_set(f)
        bpy.ops.wm.redraw_timer(type='DRAW_WIN_SWAP', iterations=1)
        tmp = os.path.join(bpy.app.tempdir, "r19_vs.png")
        with bpy.context.temp_override(window=win, area=area):
            bpy.ops.screen.screenshot_area(filepath=tmp)
        im = bpy.data.images.load(tmp, check_existing=False)
        a = np.array(im.pixels[:], dtype=np.float32).reshape(im.size[1], im.size[0], 4)
        bpy.data.images.remove(im)
        w = a.shape[1]
        a = a[::step, int(w * crop[0]):int(w * crop[1]):step].copy()
        a[:, 0, :3] = 0.1
        cols.append(a)
    s = np.ascontiguousarray(np.concatenate(cols, axis=1))
    os.makedirs(STAGES, exist_ok=True)
    path = os.path.join(STAGES, name + ".png")
    out = bpy.data.images.new(name, width=s.shape[1], height=s.shape[0], alpha=True)
    out.pixels = s.reshape(-1).tolist()
    out.filepath_raw = path
    out.file_format = 'PNG'
    out.save()
    bpy.data.images.remove(out)
    print("strip %s  %d x %d  frames %s" % (path, s.shape[1], s.shape[0], list(frames)))
    return path


# ------------------------------------------------------------- the hold solver
#
# The Carry arms are solved, not keyed by eye: the input is where each PALM must be and
# which way it must face, and shoulder (X, Y, Z), elbow (X) and wrist (X, Y, Z) fall out
# of a coordinate descent over a hand-written FK of the chest -> shoulder -> elbow -> hand
# chain. The FK takes the chest's posed matrix, so the solve holds when the spine moves.
#
# The palm is the inner face of glove box 0 (the fist), at its centre: x 0.427, z 0.547
# at bind on the right, mirrored on the left, facing the body's midline.
PALM_REST = {"R": Vector((0.427, 0.0, 0.547)), "L": Vector((-0.427, 0.0, 0.547))}
INWARD = {"R": Vector((-1.0, 0.0, 0.0)), "L": Vector((1.0, 0.0, 0.0))}


def arm_quats(side, x):
    """[(bone, quaternion)] for shoulder/elbow/hand from x = (sx, sy, sz, e, wx, wy, wz)."""
    sx, sy, sz, e, wx, wy, wz = x
    pbs = arm_ob().pose.bones
    q = W.axis_quat
    return [("shoulder." + side, q(pbs["shoulder." + side], [(X, sx), (Y, sy), (Z, sz)])),
            ("elbow." + side, q(pbs["elbow." + side], [(X, e)])),
            ("hand." + side, q(pbs["hand." + side], [(X, wx), (Y, wy), (Z, wz)]))]


def arm_fk(side, x, chest):
    """(palm point, palm normal, hand deform matrix) for arm angles x under a chest whose
    POSE matrix (armature space) is `chest`."""
    bones = arm_ob().data.bones
    m, prev = chest, "chest"
    for bone, q in arm_quats(side, x):
        m = m @ (bones[prev].matrix_local.inverted() @ bones[bone].matrix_local) \
            @ q.to_matrix().to_4x4()
        prev = bone
    mh = m @ bones["hand." + side].matrix_local.inverted()
    return mh @ PALM_REST[side], (mh.to_3x3() @ INWARD[side]).normalized(), mh


def solve_arm(side, target, normal, chest, start=None, extra=None):
    """Arm angles (radians) putting the palm at `target` facing `normal`.

    `extra(x, palm, n, hand_matrix)` adds a cost term (used to steer the prop). Returns
    (x, miss in metres, normal dot)."""
    x = list(start) if start else [0.8, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0]

    def cost(x):
        p, n, mh = arm_fk(side, x, chest)
        c = 100.0 * (p - target).length_squared + 0.2 * (1.0 - n.dot(normal))
        c += 0.0005 * sum(v * v for v in x[4:])
        if extra:
            c += extra(x, p, n, mh)
        return c

    c, step = cost(x), 0.3
    while step > 1e-4:
        better = False
        for i in range(7):
            for d in (step, -step):
                y = list(x)
                y[i] += d
                if i == 3 and not 0.0 <= y[3] <= 2.6:      # the elbow does not hyperextend
                    continue
                cy = cost(y)
                if cy < c:
                    x, c, better = y, cy, True
        if not better:
            step *= 0.5
    p, n, _ = arm_fk(side, x, chest)
    return x, (p - target).length, n.dot(normal)


def temp_cube(size, centre, name="TMP_r19_cube"):
    """A see-through stand-in for the client's carried stone. NEVER saved: remove_temp()
    deletes it, and checks() refuses to pass while it exists."""
    import bmesh
    ob = bpy.data.objects.get(name)
    if ob is None:
        me = bpy.data.meshes.new(name)
        bm = bmesh.new()
        bmesh.ops.create_cube(bm, size=1.0)
        bm.to_mesh(me)
        bm.free()
        mat = bpy.data.materials.new(name)
        mat.diffuse_color = (0.55, 0.55, 0.6, 0.5)
        me.materials.append(mat)
        ob = bpy.data.objects.new(name, me)
        bpy.context.scene.collection.objects.link(ob)
    ob.scale = (size, size, size)
    ob.location = centre
    return ob


def remove_temp():
    for ob in [o for o in bpy.data.objects if o.name.startswith("TMP_r19")]:
        me = ob.data
        bpy.data.objects.remove(ob)
        if me is not None and me.users == 0:
            for m in list(me.materials):
                if m is not None and m.users <= 1:
                    bpy.data.materials.remove(m)
            bpy.data.meshes.remove(me)


# ---------------------------------------------------------------------- Carry
#
# Legs, root and pelvis are Walk's, produced by the very function that built Walk
# (walk_r18.pose_frame), so they key to identical values: same stride, same phase, the
# client drives both clips off the same ground-covered phase. checks() proves it key by key.
#
# THE STONE. Palm reach is 0.302 m from the shoulder and the body's front is at y 0.20-0.26,
# so a palm only ever reaches the REAR edge of a cube's side face, at any size
# (stages/carry-01). A 0.25 m stone was tried first and its fists, pulled in to x = +-0.125,
# needed ~55 degrees of shoulder roll -- it wrecked the elbows and sleeves. Wolf's call: the
# brief's 0.64 m cube. Its side faces sit near shoulder width, so the arms reach almost
# straight forward (pitch ~73, roll ~22, elbow ~straight) and hold its rear edges.
#
# THE PICKAXE rides on his back, slung diagonally across the pack, head up behind the right
# shoulder -- also Wolf's call, so both hands are free for the stone. It is pinned to the
# CHEST every frame through the 20th joint, so it moves with his torso, not with his hand.
STONE = 0.64                          # metres; the client draws this cube (0.4 cells)
STONE_NEAR = 0.262                    # y of the stone's near face: 6 mm off the beard
STONE_Z = 0.720                       # stone centre z, with the walk's crouch added per frame
GRIP_DEPTH = 0.020                    # palms 20 mm in from the near face: what the reach allows
HANG = 0.30                           # weight keeping the fists (and the lantern) hanging down
POSTURE = 0.05                        # weight against shoulder/wrist roll and twist
CARRY_SPINE = 1.0                     # degrees, + = lean back against the weight
CARRY_CHEST = 1.5
CARRY_TWIST = 2.0                     # chest twist amplitude; Walk's is 7
SLING_TILT = 35.0                     # shaft off vertical, head toward +x (his right)
SLING_CENTRE = Vector((0.0, -0.425, 0.720))   # shaft centre, on the pack's back face
SHAFT_MID = 0.214                     # grip -> shaft centre along the shaft, metres

CARRY_SOL = {}                        # frame -> {side: arm angles}, so frame 24 == frame 0
CARRY_MISS = {}                       # (frame, side) -> (palm miss m, palm facing dot)


def stone_centre(f):
    return Vector((0.0, STONE_NEAR + STONE / 2.0, STONE_Z + W.bob_at(f)))


def palm_targets(f):
    c = stone_centre(f)
    return {s: Vector((sg * STONE / 2.0, STONE_NEAR + GRIP_DEPTH, c.z))
            for s, sg in (("R", 1.0), ("L", -1.0))}


def posture_cost(side):
    """The arm-shape terms of the hold solve.

    Shoulder roll/twist (Y, Z) and wrist roll/twist (Y, Z) are what wreck the sleeve and
    elbow on this rig, so they pay POSTURE per radian squared: the solve reaches forward
    with shoulder pitch first. Both fists pay HANG for tipping off vertical -- the left
    keeps the lantern hanging, and the right pays it too so the two solves are the SAME
    problem mirrored: with the term on one side only, one elbow bent 38 degrees and the
    other stayed straight, and the hold read lopsided. `side` is kept for that reason."""
    def cost(x, p, n, mh):
        d = (mh.to_3x3() @ Vector((0.0, 0.0, -1.0))).normalized()
        return HANG * (1.0 + d.z) + POSTURE * (x[1] ** 2 + x[2] ** 2 + x[5] ** 2 + x[6] ** 2)
    return cost


def mirrored(x):
    """Right-arm angles as the left arm's: X about the same axis, Y and Z negated."""
    return [x[0], -x[1], -x[2], x[3], x[4], -x[5], -x[6]]


def sling_matrix():
    """Deform matrix (rest -> slung) of the pickaxe with the chest at bind."""
    s, c = math.sin(math.radians(SLING_TILT)), math.cos(math.radians(SLING_TILT))
    u = Vector((s, 0.0, c))               # where the shaft (rest +Z) goes
    b = Vector((c, 0.0, -s))              # where the blade (rest +Y) goes, flat on the pack
    rot = Matrix((b.cross(u), b, u)).transposed()     # columns: images of X, Y, Z
    grip_rest = arm_ob().data.bones[PICK].head_local
    grip = SLING_CENTRE - u * SHAFT_MID
    return Matrix.Translation(grip) @ rot.to_4x4() @ Matrix.Translation(-grip_rest)


def pose_carry(f):
    arm = arm_ob()
    pbs = arm.pose.bones
    W.pose_frame(f, arm)                  # Walk, all 19 joints: legs, root, hips verbatim
    rest_pick()
    q = W.axis_quat
    wv = W.wave(f)
    pbs["spine"].rotation_quaternion = q(pbs["spine"], [(X, rad(CARRY_SPINE))])
    pbs["chest"].rotation_quaternion = q(pbs["chest"], [(X, rad(CARRY_CHEST)),
                                                        (Z, rad(CARRY_TWIST) * wv)])
    pbs["neck"].rotation_quaternion = q(pbs["neck"], [(X, -rad(CARRY_SPINE + CARRY_CHEST))])
    pbs["head"].rotation_quaternion = q(pbs["head"], [
        (X, W.HEAD_NOD * math.cos(2.0 * math.pi * (f - W.BOB_LOW) / float(W.HALF))),
        (Z, -rad(CARRY_TWIST) * wv)])
    pbs["beard"].rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
    bpy.context.view_layer.update()
    chest = pbs["chest"].matrix.copy()

    if f not in CARRY_SOL:
        prev = CARRY_SOL.get(f - 1)
        sol = {}
        for side, sg in (("R", 1.0), ("L", -1.0)):
            # The left arm starts from the mirrored right solution on frame 0: solved
            # independently the two arms fell into different minima (one elbow at 70
            # degrees, the other straight) and the hold read lopsided.
            if prev:
                start = prev[side]
            else:
                start = mirrored(sol["R"]) if side == "L" else None
            sol[side], miss, facing = solve_arm(side, palm_targets(f)[side],
                                                Vector((-sg, 0.0, 0.0)), chest,
                                                start=start, extra=posture_cost(side))
            CARRY_MISS[(f, side)] = (miss, facing)
        CARRY_SOL[f] = sol
    for side in ("R", "L"):
        for b, qq in arm_quats(side, CARRY_SOL[f][side]):
            pbs[b].rotation_quaternion = qq
    bpy.context.view_layer.update()

    # the pickaxe: rigid to the chest, whatever the right hand is doing
    bones = arm.data.bones
    sling = sling_matrix()
    chest_def = chest @ bones["chest"].matrix_local.inverted()
    want = chest_def @ sling @ bones[PICK].matrix_local
    parent = pbs["hand.R"].matrix @ (bones["hand.R"].matrix_local.inverted()
                                     @ bones[PICK].matrix_local)
    basis = parent.inverted() @ want
    loc, rot, _ = basis.decompose()
    pbs[PICK].location = loc
    pbs[PICK].rotation_quaternion = rot


def build_carry():
    W.prepare()
    add_pick_bone()
    CARRY_SOL.clear()
    CARRY_MISS.clear()
    arm_ob().animation_data.action = None
    for f in range(0, CYCLE):             # solve in order, warm-starting each from the last
        pose_carry(f)
    act = key_clip("Carry", pose_carry)
    worst = max(CARRY_MISS.values())
    print("PALMS  worst miss %.1f mm off the stone face, worst facing dot %.3f (1 = square)"
          % (worst[0] * 1000.0, min(v[1] for v in CARRY_MISS.values())))
    return act


# ---------------------------------------------------------------- the proofs
#
# Read off the EVALUATED meshes with the clip bound (action AND slot, via attach()), never
# off the maths that authored them. Same metrics and table shapes as walk_r18.
HEAD_PARTS = W.HEAD_PARTS
PROPS = W.PROPS
LEG_BONES = ("root", "hips", "hip.L", "knee.L", "foot.L", "hip.R", "knee.R", "foot.R")


def palm_points():
    """World palm points (R, L) as posed now."""
    arm = arm_ob()
    out = []
    for s in ("R", "L"):
        pb = arm.pose.bones["hand." + s]
        mh = pb.matrix @ arm.data.bones["hand." + s].matrix_local.inverted()
        out.append(arm.matrix_world @ (mh @ PALM_REST[s]))
    return out


def carry_offset():
    """CARRY_OFFSET: the palm midpoint averaged over the cycle, and how far it wanders."""
    attach("Carry")
    sc = bpy.context.scene
    mids = []
    for f in range(0, CYCLE):
        sc.frame_set(f)
        r, l = palm_points()
        mids.append((r + l) / 2.0)
    mean = sum(mids, Vector()) / len(mids)
    span = [max(m[i] for m in mids) - min(m[i] for m in mids) for i in range(3)]
    far = max((m - mean).length for m in mids)
    stone = mean + Vector((0.0, STONE / 2.0 - GRIP_DEPTH, 0.0))
    print("CARRY_OFFSET (palm midpoint, rig metres, mean of 24 frames) = (%.3f, %.3f, %.3f)"
          % tuple(mean))
    print("  wander: x %.1f  y %.1f  z %.1f mm span; farthest %.1f mm from the mean"
          % (span[0] * 1e3, span[1] * 1e3, span[2] * 1e3, far * 1e3))
    print("  stone CENTRE that puts the palms on its side faces = (%.3f, %.3f, %.3f) -- the"
          " palm midpoint + %.3f m in y (the palms hold its rear edge)"
          % (tuple(stone) + (STONE / 2.0 - GRIP_DEPTH,)))
    return mean, stone, far


def box_gap(pts, centre, size):
    """Signed distance of the nearest point to an axis-aligned box: < 0 means inside."""
    h = size / 2.0
    best = float("inf")
    for p in pts:
        d = [abs(p[i] - centre[i]) - h for i in range(3)]
        out = Vector([max(v, 0.0) for v in d]).length
        best = min(best, out if out > 0.0 else max(d))
    return best


def surface_gap(av, at, bv, bt):
    """walk_r18's metric: points of each against the other's surface, both ways."""
    best = float("inf")
    for q in av:
        loc, _, _, d = bt.find_nearest(q)
        if loc is not None:
            best = min(best, d)
    for q in bv:
        loc, _, _, d = at.find_nearest(q)
        if loc is not None:
            best = min(best, d)
    return best


def prop_clearance(name, stone=None):
    """Per frame: each prop's lowest z and nearest approach to the head assembly; with a
    `stone` centre (the client's FIXED offset), each prop's, the beard's and the fists'
    signed distance to the stone's volume."""
    attach(name)
    sc = bpy.context.scene
    print("prop clearance, %s -- lowest world z, nearest approach to the head assembly%s"
          % (name, ", and to the held stone's volume (< 0 = inside)" if stone else ""))
    hdr = "  %5s %10s %11s %10s %12s %9s %9s" % ("frame", "axe min z", "axe-head mm",
                                                 "lant z", "lant-head mm", "boot.L z", "boot.R z")
    if stone:
        hdr += " %10s %11s %10s %10s" % ("axe-stone", "lant-stone", "beard-stn", "fists-stn")
    print(hdr)
    bad = []
    rows = []
    for f in range(0, CYCLE):
        sc.frame_set(f)
        heads = [W.evald_tree(n) for n in HEAD_PARTS]
        row = [f]
        for p in PROPS:
            pv, pt = W.evald_tree(p)
            row.append(min(v.z for v in pv))
            row.append(min(surface_gap(pv, pt, hv, ht) for hv, ht in heads) * 1000.0)
        for s in ("L", "R"):
            row.append(min(v.z for v in W.evald("r17_boot." + s)))
        if stone:
            c = stone                     # the client's stone does NOT bob: fixed offset
            for p in PROPS + ("r17_beard",):
                row.append(box_gap(W.evald(p), c, STONE) * 1000.0)
            row.append(min(box_gap(W.evald("r17_glove." + s), c, STONE)
                           for s in ("L", "R")) * 1000.0)
        rows.append(row)
        line = "  %5d %10.5f %11.2f %10.5f %12.2f %9.5f %9.5f" % tuple(row[:7])
        if stone:
            line += " %10.1f %11.1f %10.1f %10.1f" % tuple(row[7:])
        print(line)
        if row[1] < 0.0 or row[3] < 0.0 or row[2] < 1.0 or row[4] < 1.0:
            bad.append(f)
        if row[5] < -0.0005 or row[6] < -0.0005:
            bad.append(f)
        if stone and (row[7] < 1.0 or row[8] < 1.0 or row[9] < 1.0):
            bad.append(f)
    cols = list(zip(*rows))
    tail = ""
    if stone:
        tail = (", axe-stone %.1f, lantern-stone %.1f, beard-stone %.1f mm, fists %.1f..%.1f mm"
                % (min(cols[7]), min(cols[8]), min(cols[9]), min(cols[10]), max(cols[10])))
    print("  worst: axe z %.4f, axe-head %.1f mm, lantern z %.4f, lantern-head %.1f mm%s"
          % (min(cols[1]), min(cols[2]), min(cols[3]), min(cols[4]), tail))
    print("  %s" % ("PASS" if not bad else "FAIL at frames %s" % sorted(set(bad))))
    return bad


def foot_slide_carry():
    """Walk's own test, on Carry: sole position with the client's travel added back."""
    attach("Carry")
    W.prepare()
    sc = bpy.context.scene
    st = W.stride()
    worst = 0.0
    print("foot slide, Carry -- stride %.4f m, sole y* = world y + stride * k / %d"
          % (st, CYCLE))
    for side in ("L", "R"):
        ids = W.sole_ids(side)
        shift = 0 if side == "L" else W.HALF
        pinned = {}
        print("  %s boot   %5s %6s %7s %10s %9s %10s %9s %9s"
              % (side, "frame", "pivot", "p", "heel y*", "heel z", "toe y*", "toe z", "slide mm"))
        for k in range(0, CYCLE):
            f = (shift + k) % CYCLE
            kind, frac = W.leg_phase(f, shift)
            sc.frame_set(f)
            allv = W.evald("r17_boot." + side)
            vs = [allv[i] for i in ids]
            glide = st * k / float(CYCLE)
            ymin, ymax = min(v.y for v in vs), max(v.y for v in vs)
            hz = min(v.z for v in vs if v.y <= ymin + 1e-4)
            tz = min(v.z for v in vs if v.y >= ymax - 1e-4)
            hy, ty = ymin + glide, ymax + glide
            if kind != "stance":
                print("           %5d %6s %7.3f %10s %9.5f %10s %9.5f %9s"
                      % (f, "swing", frac, "-", hz, "-", tz, "-"))
                continue
            which = W.stance_state(frac, W.LEG[side])[1]
            py, pz = (hy, hz) if which == "heel" else (ty, tz)
            pinned.setdefault(which, (py, pz))
            d = math.hypot(py - pinned[which][0], pz - pinned[which][1]) * 1000.0
            worst = max(worst, d)
            print("           %5d %6s %7.3f %10.5f %9.5f %10.5f %9.5f %9.3f"
                  % (f, which, frac, hy, hz, ty, tz, d))
    print("  WORST SLIDE %.3f mm" % worst)
    return worst


def foot_slide_dig():
    """Dig is planted: every sole vertex must hold its frame-0 world position, all frames."""
    attach("Dig")
    sc = bpy.context.scene
    ids = {s: W.sole_ids(s) for s in ("L", "R")}
    ref, worst = {}, 0.0
    print("foot slide, Dig -- planted: sole vertices against their frame-0 world positions")
    print("  %5s %12s %12s %10s %10s" % ("frame", "L drift mm", "R drift mm",
                                         "L min z", "R min z"))
    for f in range(0, CYCLE + 1):
        sc.frame_set(f)
        row = [f]
        zs = []
        for s in ("L", "R"):
            allv = W.evald("r17_boot." + s)
            vs = [allv[i] for i in ids[s]]
            if s not in ref:
                ref[s] = vs
            d = max((a - b).length for a, b in zip(vs, ref[s])) * 1000.0
            worst = max(worst, d)
            row.append(d)
            zs.append(min(v.z for v in allv))
        print("  %5d %12.4f %12.4f %10.5f %10.5f" % tuple(row + zs))
    print("  WORST SLIDE %.4f mm" % worst)
    return worst


def in_place(name):
    attach(name)
    sc = bpy.context.scene
    arm = arm_ob()
    xs, ys, zs = [], [], []
    for f in range(0, CYCLE + 1):
        sc.frame_set(f)
        t = arm.matrix_world @ arm.pose.bones["root"].matrix.translation
        xs.append(t.x)
        ys.append(t.y)
        zs.append(t.z)
    print("in place, %s -- root x span %.3e m, y span %.3e m; z %.4f .. %.4f"
          % (name, max(xs) - min(xs), max(ys) - min(ys), min(zs), max(zs)))
    return max(max(xs) - min(xs), max(ys) - min(ys))


def loop_seam(name):
    attach(name)
    sc = bpy.context.scene
    arm = arm_ob()
    sc.frame_set(0)
    a = {pb.name: pb.matrix.copy() for pb in arm.pose.bones}
    sc.frame_set(CYCLE)
    b = {pb.name: pb.matrix.copy() for pb in arm.pose.bones}
    worst = max(abs(a[n][i][j] - b[n][i][j]) for n in a for i in range(4) for j in range(4))
    print("loop seam, %s -- frame 0 vs %d, %d joints: worst element delta %.3e  %s"
          % (name, CYCLE, len(a), worst, "IDENTICAL" if worst == 0.0 else "DIFFERENT"))
    return worst


def carry_legs_verbatim():
    """Carry's root/hips/leg keys against Walk's, value for value."""
    def keys(act):
        out = {}
        for fc in W.fcurves(act):
            bone = fc.data_path.split('"')[1]
            if bone in LEG_BONES:
                out[(fc.data_path, fc.array_index)] = [tuple(k.co) for k in fc.keyframe_points]
        return out
    w, c = keys(bpy.data.actions["Walk"]), keys(bpy.data.actions["Carry"])
    diff = [k for k in w if w[k] != c.get(k)]
    same = not diff and set(w) == set(c)
    print("Carry legs/root/hips vs Walk: %d curves, %d keys compared, %d curves differ  %s"
          % (len(w), sum(len(v) for v in w.values()), len(diff),
             "VERBATIM" if same else "DIFFERENT %s" % diff[:4]))
    return same


def seam_clip(name, limit_mm=10.0, skip=None):
    """walk_r18.seam_walk's method at the real posed frames of `name`."""
    import seam_check
    importlib.reload(seam_check)
    skip = skip or {}
    detach()
    base = seam_check.snapshot()
    names = sorted(base)
    pairs = [(a, b) for i, a in enumerate(names) for b in names[i + 1:]
             if seam_check.gap(base[a], base[b]) <= seam_check.CONTACT]
    attach(name)
    sc = bpy.context.scene
    worst = {p: (0.0, 0) for p in pairs}
    for f in range(0, CYCLE):
        sc.frame_set(f)
        shot = seam_check.snapshot()
        for p in pairs:
            d = seam_check.gap(shot[p[0]], shot[p[1]])
            if d > worst[p][0]:
                worst[p] = (d, f)
    fails = 0
    print("seams, %s -- %d pairs that touch at bind, measured every frame" % (name, len(pairs)))
    for (a, b), (d, f) in sorted(worst.items(), key=lambda kv: -kv[1][0]):
        flag = ""
        why = skip.get((a, b)) or skip.get((b, a))
        if why:
            flag = "   (by design: %s)" % why
        elif d * 1000.0 > limit_mm:
            flag = "   <-- FAIL"
            fails += 1
        print("  %-16s %-16s %8.1f mm  frame %2d%s" % (a, b, d * 1000.0, f, flag))
    print("  %s -- %d of %d pairs exceed %.0f mm" % ("PASS" if not fails else "FAIL", fails,
                                                    len(pairs), limit_mm))
    return fails


def strike_report():
    attach("Dig")
    bpy.context.scene.frame_set(STRIKE)
    tip, eye = pick_head()
    print("STRIKE frame %d of %d (phase %.3f): blade tip (%.3f, %.3f, %.3f), head eye "
          "(%.3f, %.3f, %.3f), rig metres"
          % ((STRIKE, CYCLE, STRIKE / float(CYCLE)) + tuple(tip) + tuple(eye)))
    return tip, eye


def checks():
    bar = "=" * 78
    print(bar)
    print("actions: %s" % sorted(a.name for a in bpy.data.actions))
    print("Walk  round-18 curves %s  (must be 628961a8...)" % fcurve_hash("Walk", W.BONES))
    for name in ("Walk", "Dig", "Carry"):
        print("%-5s fcurve hash %s" % (name, fcurve_hash(name)))
    for name in ("Dig", "Carry"):
        act = bpy.data.actions[name]
        curves = W.fcurves(act)
        print("%s: frames %d..%d at %d fps, slots %d, fake user %s, %d fcurves, all LINEAR %s"
              % (name, act.frame_range[0], act.frame_range[1], bpy.context.scene.render.fps,
                 len(act.slots), act.use_fake_user, len(curves),
                 all(k.interpolation == 'LINEAR' for fc in curves for k in fc.keyframe_points)))
    print(bar)
    strike_report()
    print(bar)
    _, stone, _ = carry_offset()
    print(bar)
    sd = foot_slide_dig()
    print(bar)
    sc_ = foot_slide_carry()
    print(bar)
    verb = carry_legs_verbatim()
    print(bar)
    bd = prop_clearance("Dig")
    print(bar)
    bc = prop_clearance("Carry", stone=stone)
    print(bar)
    ip = max(in_place("Dig"), in_place("Carry"))
    print(bar)
    ls = max(loop_seam("Dig"), loop_seam("Carry"))
    print(bar)
    sling = {("r17_glove.R", "r17_pickaxe"): "the pickaxe rides on the back in Carry"}
    fd = seam_clip("Dig")
    fc = seam_clip("Carry", skip=sling)
    print(bar)
    temp = [o.name for o in bpy.data.objects if o.name.startswith("TMP_r19")]
    print("SUMMARY  slide Dig %.4f / Carry %.3f mm | legs verbatim %s | props Dig %s Carry %s"
          " | in-place %.1e m | loop %.1e | seams Dig %d Carry %d fails | temp objects %s"
          % (sd, sc_, verb, "ok" if not bd else "FAIL", "ok" if not bc else "FAIL", ip, ls,
             fd, fc, temp or "none"))
    detach()


# ------------------------------------------------------------------- renders
def renders(which):
    """Cycles key strips to renders/r19/ via render_r18.sheet, wider for the swing.

    Render settings, the scene camera and frame range are put back afterwards, and the
    strip camera and the stand-in stone are deleted: none of it is saved into the .blend.
    """
    import render_r18 as R
    importlib.reload(R)
    R.OUT = os.path.join(os.path.dirname(HERE), "renders", "r19")
    R.TARGET = Vector((0.0, 0.12, 0.80))
    R.SPAN = 2.0
    R.COLW, R.COLH = 380, 440
    sc = bpy.context.scene
    keep = (sc.render.engine, sc.render.resolution_x, sc.render.resolution_y,
            sc.render.filepath, sc.camera, sc.render.film_transparent,
            sc.render.image_settings.color_mode, sc.frame_start, sc.frame_end,
            sc.frame_current, sc.render.fps)
    keep_cy = (sc.cycles.device, sc.cycles.samples, sc.cycles.use_denoising)
    try:
        if which == "dig":
            attach("Dig")
            keys = [0, 10, 12, 16, 17, 18, 20, 22]
            R.sheet("dig-keys-side", keys, 90)
            R.sheet("dig-keys-front", keys, 180)
            R.sheet("dig-keys-threequarter", keys, 135, 10)
            R.sheet("dig-cycle-side", list(range(0, CYCLE, 2)), 90)
        else:
            attach("Carry")
            temp_cube(STONE, carry_offset()[1])
            keys = [0, 3, 6, 9, 12, 15, 18, 21]
            R.sheet("carry-keys-side", keys, 90)
            R.sheet("carry-keys-front", keys, 180)
            R.sheet("carry-keys-threequarter", keys, 135, 10)
            R.sheet("carry-keys-back", [0, 6, 12, 18], 0)
    finally:
        (sc.render.engine, sc.render.resolution_x, sc.render.resolution_y,
         sc.render.filepath, sc.camera, sc.render.film_transparent,
         sc.render.image_settings.color_mode, sc.frame_start, sc.frame_end,
         sc.frame_current, sc.render.fps) = keep
        sc.cycles.device, sc.cycles.samples, sc.cycles.use_denoising = keep_cy
        cam =bpy.data.objects.get("cam_r18")
        if cam is not None:
            data = cam.data
            bpy.data.objects.remove(cam)
            bpy.data.cameras.remove(data)
        remove_temp()
