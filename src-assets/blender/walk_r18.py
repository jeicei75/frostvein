"""Round 18 -- the looping `Walk` action for the r17 voxel dwarf.

Run it in the live session against SM_VoxelDwarf_Miner01.blend:

    exec(open(r"src-assets/blender/walk_r18.py").read()); build(); checks()

WHAT THIS FILE DECIDES, AND WHY
-------------------------------
The cycle is IN PLACE. `blended_translation()` on the Rust side already lerps the dwarf
between cells, so no bone may carry net horizontal travel; `root` gets a vertical bob and
nothing else. The one number the client needs back is the STRIDE -- how far one full cycle
WOULD advance him -- and it is not a number you pick, it is a number the leg geometry
hands you. See stride().

The legs are solved, not eyeballed. A hand-keyed leg on this rig slides: the boot's ground
face is 0.249 m long under a 0.200 m leg, so a couple of degrees at the hip is centimetres
at the sole. Instead the CONTACT POINT is the input -- heel, then whole sole, then toe,
each pinned to the ground and travelling backward through body space at exactly the ground
speed -- and hip/knee/foot fall out of a closed-form two-link solve. Foot slide is then
zero by construction rather than by tuning, and checks() is a proof instead of a hope.

NOTE: `hips` rotates about X only -- no pelvic twist, no pelvic roll. Either would swing
the leg chain out of the sagittal plane, and since hip/knee/foot are single-axis X hinges
the solve could not put the sole back on its line: a 5 deg pelvis twist is ~16 mm of
lateral foot slide. The counter-rotation read lives on `spine`/`chest` instead, which have
no leg under them. A future round that wants a twisting pelvis has to pay for it with a Z
channel on hip.L/R.

NOTE: every frame is keyed, LINEAR. The glTF exporter samples per frame anyway, so linear
keys make what Blender plays and what Bevy plays the same curve. Bezier handles at 1-frame
spacing overshoot between the samples, and an overshoot on the stance leg is foot slide
that no integer-frame check would ever see.

The four key poses the brief asks for land on frames 0 contact, 3 down, 6 passing, 9 up,
and their mirrors at 12, 15, 18, 21. They are where the trajectory puts them; they are not
separately authored, because a key pose that disagreed with the trajectory would be a pose
the cycle never actually reaches.
"""

import math

import bpy
from mathutils import Vector, Quaternion

# ------------------------------------------------------------------ the dials
FPS = 24
CYCLE = 24                  # frames per full cycle (two steps) -> 1.000 s
STANCE = 13                 # frames of stance per foot; the rest is swing
HALF = CYCLE // 2           # the other foot is this one, shifted by HALF

FWD = 0.074                 # ankle y at heel strike, armature space
BACK = -0.116                # ankle y at toe off
PHI_HS = math.radians(12.0)         # foot pitch at heel strike, toe UP
PHI_TO = math.radians(-18.0)        # foot pitch at toe off, toe DOWN
P_HEEL_END = 0.14           # stance fraction spent rolling off the heel
P_TOE_START = 0.45           # stance fraction at which the heel leaves

# The bind pose IS this rig's tallest standing pose: hip head 0.330, sole 0.000, and
# hip -> ankle 0.200 with the knee dead straight. So every frame of a walk has to sit
# BELOW bind -- there is no headroom to bob up into. ROOT_BASE buys the room the planted
# leg needs while the sole is flat; without it the stance foot simply leaves the ground
# mid-stance, which is what the first build did.
ROOT_BASE = -0.015          # constant drop off the bind pose, metres
BOB = 0.005                 # rise/fall about ROOT_BASE
BOB_LOW = 2.0               # frame of the lowest point (and +HALF)

SWING_LIFT = 0.004          # extra ankle rise at mid swing
SWING_DORSI = math.radians(5.0)     # toe-up through mid swing, for clearance

HIPS_X = math.radians(2.0)          # pelvis nods twice a cycle
SPINE_LEAN = math.radians(-2.5)     # negative X = forward: these bones point UP
SPINE_TWIST = math.radians(-2.5)
CHEST_LEAN = math.radians(-1.5)
CHEST_TWIST = math.radians(7.0)
NECK_LEVEL = math.radians(3.0)      # undo the lean so the head stays level
HEAD_TWIST = math.radians(-4.0)     # the face keeps pointing down +Y
HEAD_NOD = math.radians(1.5)
BEARD_SWING = math.radians(4.0)
BEARD_LAG = 2.0                     # frames the beard trails the chest

ARM_SWING = math.radians(11.0)
ARM_LAG = 1.0                       # frames the arm trails its opposite leg
ELBOW_BASE = math.radians(4.0)
ELBOW_SWING = math.radians(4.0)

ACTION = "Walk"
ARM_NAME = "SK_VoxelDwarf_Miner01_r17"

BONES = ["root", "hips", "spine", "chest", "neck", "head", "beard",
         "shoulder.R", "elbow.R", "hand.R", "shoulder.L", "elbow.L", "hand.L",
         "hip.R", "knee.R", "foot.R", "hip.L", "knee.L", "foot.L"]

X = Vector((1.0, 0.0, 0.0))
Y = Vector((0.0, 1.0, 0.0))
Z = Vector((0.0, 0.0, 1.0))

LEG = None          # {"R": {...}, "L": {...}}, filled by prepare()
REACH = []          # (needed, available) per leg solve, so a clamp can never hide


# ------------------------------------------------------------------ rig facts
def arm_ob():
    return bpy.data.objects[ARM_NAME]


def sole(side):
    """(heel y, toe y, z) of the boot's ground face, measured off the mesh.

    Bounding boxes lie on this figure, so this reads the bottom face of the sole box
    itself and not the object's extents: box 0 of the boot is the sole, and the vertices
    at its lowest z are the ones that touch the floor.
    """
    ob = bpy.data.objects["r17_boot." + side]
    starts = list(ob["box_starts"])
    end = starts[1] if len(starts) > 1 else len(ob.data.vertices)
    vs = [ob.data.vertices[i].co for i in range(starts[0], end)]
    zlo = min(v.z for v in vs)
    face = [v for v in vs if v.z <= zlo + 1e-6]
    return min(v.y for v in face), max(v.y for v in face), zlo


def prepare():
    """Per side: hip head, segment lengths, ankle->contact offsets, ground speed."""
    global LEG
    bones = arm_ob().data.bones
    hips_head = Vector(bones["hips"].head_local)
    out = {}
    for side in ("R", "L"):
        hip, knee, foot = bones["hip." + side], bones["knee." + side], bones["foot." + side]
        hy, ty, sz = sole(side)
        ank = Vector(foot.head_local)
        L = dict(
            hips_head=hips_head,
            H=Vector(hip.head_local),
            L1=hip.length, L2=knee.length,
            heel=(hy - ank.y, sz - ank.z),      # rest offset ankle -> heel, (y, z)
            toe=(ty - ank.y, sz - ank.z),
            soleN=ty - hy,
        )
        # heel y at strike, chosen so the ankle lands at FWD
        L["yh0"] = FWD + rot2(L["heel"][0], L["heel"][1], PHI_HS)[0]
        # toe y at toe off, chosen so the ankle lands at BACK -- this fixes the ground speed
        toe_end = BACK + rot2(L["toe"][0], L["toe"][1], PHI_TO)[0]
        L["D"] = L["yh0"] + L["soleN"] - toe_end
        out[side] = L
    LEG = out
    return out


# ------------------------------------------------------------------ the maths
def smooth(t):
    t = min(1.0, max(0.0, t))
    return t * t * (3.0 - 2.0 * t)


def rot2(oy, oz, phi):
    """(oy, oz) rotated by phi about +X, in the (y, z) plane."""
    c, s = math.cos(phi), math.sin(phi)
    return oy * c - oz * s, oy * s + oz * c


def stance_state(p, L):
    """Foot pitch and the ground-pinned contact point at stance fraction p in [0, 1]."""
    if p <= P_HEEL_END:
        phi = PHI_HS * (1.0 - smooth(p / P_HEEL_END))
    elif p <= P_TOE_START:
        phi = 0.0
    else:
        phi = PHI_TO * smooth((p - P_TOE_START) / (1.0 - P_TOE_START))
    # The heel is pinned from strike until P_TOE_START, the toe from then on. Through the
    # flat phase both are on the ground, and the two expressions agree there.
    if p <= P_TOE_START:
        return phi, "heel", L["yh0"] - L["D"] * p
    return phi, "toe", L["yh0"] + L["soleN"] - L["D"] * p


def ankle_from_contact(phi, which, py, L):
    """Ankle (y, z) that puts contact point `which` on the ground at y = py."""
    dy, dz = rot2(L[which][0], L[which][1], phi)
    return py - dy, -dz          # the contact point sits at z = 0


def stance_ankle(p, L):
    phi, which, py = stance_state(p, L)
    ay, az = ankle_from_contact(phi, which, py, L)
    return ay, az, phi


def swing_ankle(u, L):
    a0y, a0z, _ = stance_ankle(1.0, L)
    a1y, a1z, _ = stance_ankle(0.0, L)
    t = smooth(u)
    ay = a0y + (a1y - a0y) * t
    az = a0z + (a1z - a0z) * t + SWING_LIFT * math.sin(math.pi * u ** 0.9)
    phi = PHI_TO + (PHI_HS - PHI_TO) * t + SWING_DORSI * math.sin(math.pi * u)
    return ay, az, phi


def leg_angles(ay, az, phi, hip_y, hip_z, alpha, L):
    """Closed-form hip/knee/foot X rotations putting the ankle at (ay, az).

    Everything here is in the sagittal plane: `hips` only rotates about X, so the chain's
    absolute pitches add. A bone at absolute pitch b points along (sin b, -cos b) in
    (y, z) -- the leg bones point straight down at bind.
    """
    L1, L2 = L["L1"], L["L2"]
    dy, dz = ay - hip_y, az - hip_z
    r = math.hypot(dy, dz)
    # A clamp here is not a rounding detail: it means the leg could not reach the ground
    # and the foot floats. The first build did exactly that for six frames of mid-stance
    # and the slide table was the only thing that said so, so record it instead.
    REACH.append((r, L1 + L2))
    r = min(r, L1 + L2 - 1e-5)
    r = max(r, abs(L2 - L1) + 1e-5)
    base = math.atan2(dy, -dz)
    # the knee sits forward of the hip->ankle line: that is the way this rig's knee bends
    d = math.acos(max(-1.0, min(1.0, (r * r + L1 * L1 - L2 * L2) / (2.0 * r * L1))))
    b1 = base + d
    ky, kz = hip_y + L1 * math.sin(b1), hip_z - L1 * math.cos(b1)
    b2 = math.atan2(ay - ky, -(az - kz))
    return b1 - alpha, b2 - b1, phi - b2


def hip_pivot(L, alpha, bz):
    """World (y, z) of this leg's hip head once `hips` has tilted by alpha and root by bz."""
    o = L["H"] - L["hips_head"]
    dy, dz = rot2(o.y, o.z, alpha)
    return L["hips_head"].y + dy, L["hips_head"].z + dz + bz


def stride():
    """Metres one full cycle would advance him. Derived, not chosen.

    Through stance the grounded point of the foot travels backward through body space at
    the ground speed; over the whole stance that is D metres. Stance is STANCE/CYCLE of
    the cycle, so the cycle is worth D * CYCLE / STANCE.
    """
    return LEG["L"]["D"] * float(CYCLE) / float(STANCE)


# ------------------------------------------------------------- pose assembly
def bob_at(f):
    return ROOT_BASE - BOB * math.cos(2.0 * math.pi * (f - BOB_LOW) / float(HALF))


def wave(f, lag=0.0):
    """+1 when the LEFT leg is fully forward, -1 half a cycle later."""
    return math.cos(2.0 * math.pi * (f - lag) / float(CYCLE))


def axis_quat(pb, pairs):
    """Rotation about ARMATURE-space axes, expressed in the bone's own rest frame."""
    inv = pb.bone.matrix_local.to_3x3().inverted()
    q = Quaternion((1.0, 0.0, 0.0, 0.0))
    for axis, ang in pairs:
        if abs(ang) > 1e-12:
            q = q @ Quaternion(inv @ axis, ang)
    return q


def leg_phase(f, shift):
    """("stance"|"swing", fraction) for one leg at frame f."""
    n = (f - shift) % CYCLE
    if n <= STANCE:
        return "stance", n / float(STANCE)
    return "swing", (n - STANCE) / float(CYCLE - STANCE)


def pose_frame(f, arm):
    """Set every pose bone for frame f. Returns {side: (kind, frac, ay, az, phi, t1,t2,t3)}."""
    pbs = arm.pose.bones
    for name in BONES:
        pbs[name].rotation_mode = 'QUATERNION'

    # ---- root: vertical only. `root` points up, so world +Z is its own local +Y.
    bz = bob_at(f)
    inv = pbs["root"].bone.matrix_local.to_3x3().inverted()
    pbs["root"].location = inv @ Vector((0.0, 0.0, bz))
    pbs["root"].rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))

    # ---- spine chain
    alpha = HIPS_X * math.cos(2.0 * math.pi * (f - BOB_LOW) / float(HALF))
    pbs["hips"].rotation_quaternion = axis_quat(pbs["hips"], [(X, alpha)])
    pbs["spine"].rotation_quaternion = axis_quat(
        pbs["spine"], [(X, SPINE_LEAN), (Z, SPINE_TWIST * wave(f))])
    pbs["chest"].rotation_quaternion = axis_quat(
        pbs["chest"], [(X, CHEST_LEAN), (Z, CHEST_TWIST * wave(f))])
    pbs["neck"].rotation_quaternion = axis_quat(pbs["neck"], [(X, NECK_LEVEL)])
    pbs["head"].rotation_quaternion = axis_quat(
        pbs["head"], [(X, HEAD_NOD * math.cos(2.0 * math.pi * (f - BOB_LOW) / float(HALF))),
                      (Z, HEAD_TWIST * wave(f))])
    pbs["beard"].rotation_quaternion = axis_quat(
        pbs["beard"], [(X, BEARD_SWING * wave(f, BEARD_LAG))])

    # ---- arms: each swings against the leg on its own side
    for side, sign in (("L", 1.0), ("R", -1.0)):
        s = sign * wave(f, ARM_LAG)          # +1 when this arm is BACK
        pbs["shoulder." + side].rotation_quaternion = axis_quat(
            pbs["shoulder." + side], [(X, -ARM_SWING * s)])
        # the elbow flexes -- hand forward and up -- which on a down-hanging arm is a
        # POSITIVE turn about world X, and it flexes most while the arm is forward
        pbs["elbow." + side].rotation_quaternion = axis_quat(
            pbs["elbow." + side], [(X, ELBOW_BASE - ELBOW_SWING * s)])
        pbs["hand." + side].rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))

    # ---- legs, solved from the contact point
    out = {}
    for side, shift in (("L", 0), ("R", HALF)):
        L = LEG[side]
        kind, frac = leg_phase(f, shift)
        ay, az, phi = (stance_ankle if kind == "stance" else swing_ankle)(frac, L)
        hy, hz = hip_pivot(L, alpha, bz)
        t1, t2, t3 = leg_angles(ay, az, phi, hy, hz, alpha, L)
        pbs["hip." + side].rotation_quaternion = Quaternion(X, t1)
        pbs["knee." + side].rotation_quaternion = Quaternion(X, t2)
        pbs["foot." + side].rotation_quaternion = Quaternion(X, t3)
        out[side] = (kind, frac, ay, az, phi,
                     math.degrees(t1), math.degrees(t2), math.degrees(t3))
    return out


# ------------------------------------------------------------------- the build
def fcurves(act):
    """Every F-curve of a slotted action. Blender 5 dropped Action.fcurves: the curves
    now hang off layer -> strip -> channelbag(slot), one bag per slot."""
    out = []
    for layer in act.layers:
        for strip in layer.strips:
            for bag in strip.channelbags:
                out.extend(bag.fcurves)
    return out


def bind_pose():
    arm = arm_ob()
    for pb in arm.pose.bones:
        pb.rotation_mode = 'QUATERNION'
        pb.rotation_quaternion = Quaternion((1.0, 0.0, 0.0, 0.0))
        pb.location = (0.0, 0.0, 0.0)
        pb.scale = (1.0, 1.0, 1.0)
    bpy.context.view_layer.update()


def build():
    prepare()
    del REACH[:]
    arm = arm_ob()
    bpy.context.view_layer.objects.active = arm

    old = bpy.data.actions.get(ACTION)
    if old is not None:
        old.use_fake_user = False
        bpy.data.actions.remove(old)
    if arm.animation_data is None:
        arm.animation_data_create()
    act = bpy.data.actions.new(ACTION)
    arm.animation_data.action = act

    sc = bpy.context.scene
    sc.render.fps = FPS
    sc.frame_start = 0
    sc.frame_end = CYCLE              # frame 24 repeats frame 0: the closing key

    for f in range(0, CYCLE + 1):
        pose_frame(f % CYCLE, arm)
        for name in BONES:
            pb = arm.pose.bones[name]
            pb.keyframe_insert("rotation_quaternion", frame=f, group=name)
            if name == "root":
                pb.keyframe_insert("location", frame=f, group=name)

    curves = fcurves(act)
    for fc in curves:
        for kp in fc.keyframe_points:
            kp.interpolation = 'LINEAR'
        fc.update()
    act.use_fake_user = True

    bpy.context.view_layer.update()
    print("ACTION %s -- frames 0..%d at %d fps (%.3f s), %d fcurves, %d keys"
          % (ACTION, CYCLE, FPS, CYCLE / float(FPS), len(curves),
             sum(len(fc.keyframe_points) for fc in curves)))
    print("STRIDE %.3f m per cycle (%.3f m per step); stance %d/%d frames = %.1f%%"
          % (stride(), stride() / 2.0, STANCE, CYCLE, 100.0 * STANCE / CYCLE))
    print("SPEED  at v m/s the clip plays at v / %.3f" % stride())
    need = max(r for r, _ in REACH)
    have = REACH[0][1]
    print("REACH  worst %.5f m of %.5f m available (%.1f%%); %d solves clamped %s"
          % (need, have, 100.0 * need / have,
             sum(1 for r, a in REACH if r > a - 1e-5),
             "<-- THE FOOT FLOATS" if need > have - 1e-5 else "-- none"))
    return act


def detach():
    """Leave the figure at bind, with the action kept as data on the file."""
    arm = arm_ob()
    if arm.animation_data is not None:
        arm.animation_data.action = None
    bind_pose()
    print("detached: pose is bind, action %r kept (fake user %s)"
          % (ACTION, bpy.data.actions[ACTION].use_fake_user))


def attach():
    arm = arm_ob()
    if arm.animation_data is None:
        arm.animation_data_create()
    arm.animation_data.action = bpy.data.actions[ACTION]


# ---------------------------------------------------------------- the proofs
#
# Every number below is read off the EVALUATED mesh with the action driving the rig, not
# off the maths that authored it. A solve can be right and the rig still wrong -- weights,
# bone roll, a quaternion that never took -- and only the evaluated mesh knows.

HEAD_PARTS = ("r17_head", "r17_hair", "r17_beard", "r17_moustache")
PROPS = ("r17_pickaxe", "r17_lantern")


def sole_ids(side):
    """Vertex indices of the boot's ground face at bind."""
    ob = bpy.data.objects["r17_boot." + side]
    starts = list(ob["box_starts"])
    end = starts[1] if len(starts) > 1 else len(ob.data.vertices)
    vs = ob.data.vertices
    zlo = min(vs[i].co.z for i in range(starts[0], end))
    return [i for i in range(starts[0], end) if vs[i].co.z <= zlo + 1e-6]


def evald(name):
    """World-space vertices of `name` as posed right now."""
    deps = bpy.context.evaluated_depsgraph_get()
    ob = bpy.data.objects[name]
    ev = ob.evaluated_get(deps)
    me = ev.to_mesh()
    mw = ob.matrix_world
    out = [mw @ v.co for v in me.vertices]
    ev.to_mesh_clear()
    return out


def evald_tree(name):
    from mathutils.bvhtree import BVHTree
    deps = bpy.context.evaluated_depsgraph_get()
    ob = bpy.data.objects[name]
    ev = ob.evaluated_get(deps)
    me = ev.to_mesh()
    mw = ob.matrix_world
    vs = [mw @ v.co for v in me.vertices]
    ps = [tuple(p.vertices) for p in me.polygons]
    ev.to_mesh_clear()
    return vs, BVHTree.FromPolygons(vs, ps)


def foot_slide():
    """The check that matters most.

    The clip is in place, so a grounded sole travels backward through body space on
    purpose: the client's forward motion is what cancels it. The test is therefore not
    "does the sole hold still" but "does the sole hold still once the client's travel is
    put back" -- add stride * (frame - 1) / CYCLE to y, and the grounded point must not
    move. Residuals are against each contact's own pinned value.
    """
    prepare()
    attach()
    sc = bpy.context.scene
    ids = {s: sole_ids(s) for s in ("R", "L")}
    st = stride()
    print("foot slide -- sole world position with the client travel added back")
    print("  stride %.4f m/cycle, so +%.5f m of travel per frame" % (st, st / CYCLE))
    worst = 0.0
    for side in ("L", "R"):
        shift = 0 if side == "L" else HALF
        pinned = {}
        print("  %s boot" % side)
        print("    %5s %6s %7s %10s %9s %10s %9s %9s"
              % ("frame", "pivot", "p", "heel y*", "heel z", "toe y*", "toe z", "slide mm"))
        # walked in the leg's OWN phase order, starting at its heel strike. Walking it in
        # frame order instead restarts the travel baseline in the middle of a stance and
        # reports half a stride of "slide" that is not there.
        for k in range(0, CYCLE):
            f = (shift + k) % CYCLE
            kind, frac = leg_phase(f, shift)
            sc.frame_set(f)
            allv = evald("r17_boot." + side)
            vs = [allv[i] for i in ids[side]]
            glide = st * k / float(CYCLE)
            ymin = min(v.y for v in vs)
            ymax = max(v.y for v in vs)
            hz = min(v.z for v in vs if v.y <= ymin + 1e-4)
            tz = min(v.z for v in vs if v.y >= ymax - 1e-4)
            hy, ty = ymin + glide, ymax + glide
            if kind != "stance":
                print("    %5d %6s %7.3f %10s %9.5f %10s %9.5f %9s"
                      % (f, "swing", frac, "-", hz, "-", tz, "-"))
                continue
            which = stance_state(frac, LEG[side])[1]
            py, pz = (hy, hz) if which == "heel" else (ty, tz)
            if which not in pinned:
                pinned[which] = (py, pz)
            d = math.hypot(py - pinned[which][0], pz - pinned[which][1]) * 1000.0
            worst = max(worst, d)
            print("    %5d %6s %7.3f %10.5f %9.5f %10.5f %9.5f %9.3f"
                  % (f, which, frac, hy, hz, ty, tz, d))
    print("  WORST SLIDE %.3f mm" % worst)
    return worst


def prop_clearance():
    prepare()
    attach()
    sc = bpy.context.scene
    print("prop clearance -- lowest world z, and nearest approach to the head assembly")
    print("  %5s %11s %12s %11s %13s %10s %10s"
          % ("frame", "axe min z", "axe-head mm", "lantern z", "lantern-head",
             "boot.L z", "boot.R z"))
    bad = []
    for f in range(0, CYCLE):
        sc.frame_set(f)
        headv, headt = [], []
        for n in HEAD_PARTS:
            v, t = evald_tree(n)
            headv.append(v)
            headt.append(t)
        row = [f]
        for p in PROPS:
            pv, pt = evald_tree(p)
            row.append(min(v.z for v in pv))
            best = float("inf")
            for v, t in zip(headv, headt):
                for q in pv:
                    loc, _, _, d = t.find_nearest(q)
                    if loc is not None and d < best:
                        best = d
                for q in v:
                    loc, _, _, d = pt.find_nearest(q)
                    if loc is not None and d < best:
                        best = d
            row.append(best * 1000.0)
        for s in ("L", "R"):
            row.append(min(v.z for v in evald("r17_boot." + s)))
        print("  %5d %11.5f %12.2f %11.5f %13.2f %10.5f %10.5f" % tuple(row))
        if row[1] < 0.0 or row[3] < 0.0 or row[2] < 1.0 or row[4] < 1.0:
            bad.append(f)
        if row[5] < -0.0005 or row[6] < -0.0005:
            bad.append(f)
    print("  %s" % ("PASS -- nothing under the floor, nothing touching the skull"
                    if not bad else "FAIL at frames %s" % sorted(set(bad))))
    return bad


def in_place():
    prepare()
    attach()
    sc = bpy.context.scene
    print("in place -- root world x and y must not move")
    arm = arm_ob()
    xs, ys = [], []
    for f in range(0, CYCLE + 1):
        sc.frame_set(f)
        t = arm.matrix_world @ arm.pose.bones["root"].matrix.translation
        xs.append(t.x)
        ys.append(t.y)
        print("  frame %3d   root x %+.9f   y %+.9f   z %+.6f" % (f, t.x, t.y, t.z))
    print("  root x span %.3e m   y span %.3e m" % (max(xs) - min(xs), max(ys) - min(ys)))
    return max(max(xs) - min(xs), max(ys) - min(ys))


def loop_seam():
    prepare()
    attach()
    sc = bpy.context.scene
    arm = arm_ob()
    sc.frame_set(0)
    a = {pb.name: pb.matrix.copy() for pb in arm.pose.bones}
    sc.frame_set(CYCLE)
    b = {pb.name: pb.matrix.copy() for pb in arm.pose.bones}
    worst = 0.0
    for n in sorted(a):
        d = max(abs(a[n][i][j] - b[n][i][j]) for i in range(4) for j in range(4))
        worst = max(worst, d)
        if d > 1e-9:
            print("  %-12s differs by %.3e" % (n, d))
    print("loop seam -- frame 0 vs frame %d: worst element delta %.3e  %s"
          % (CYCLE, worst, "IDENTICAL" if worst <= 1e-9 else "DIFFERENT"))
    return worst


def joint_range():
    """The angles this cycle actually uses, to widen seam_check with."""
    prepare()
    lo, hi = {}, {}
    arm = arm_ob()
    for f in range(0, CYCLE):
        st = pose_frame(f, arm)
        for side in ("L", "R"):
            for name, v in zip(("hip", "knee", "foot"), st[side][5:8]):
                lo[name] = min(lo.get(name, 1e9), v)
                hi[name] = max(hi.get(name, -1e9), v)
    print("leg joint range this cycle actually uses, degrees about X")
    for n in ("hip", "knee", "foot"):
        print("  %-6s %+8.2f .. %+8.2f" % (n, lo[n], hi[n]))
    bind_pose()
    return lo, hi


def checks():
    print("=" * 78)
    w = foot_slide()
    print("=" * 78)
    b = prop_clearance()
    print("=" * 78)
    d = in_place()
    print("=" * 78)
    s = loop_seam()
    print("=" * 78)
    joint_range()
    print("=" * 78)
    print("SUMMARY  slide %.3f mm | props %s | in-place %.1e m | seam %.1e"
          % (w, "ok" if not b else "FAIL", d, s))


def seam_walk(limit_mm=10.0):
    """The seams at the poses this cycle ACTUALLY reaches.

    seam_check.report() sweeps one joint at a time, which is the right shape for a range
    of motion but the wrong shape for a clip: this walk moves hip, knee and foot together,
    and a pair that survives every single-joint sweep can still open on the combination.
    So this re-measures the same contacting pairs frame by frame with the action driving
    the rig -- the same surface-to-surface BVH metric, just at the real poses.
    """
    import os
    import sys
    here = os.path.dirname(bpy.data.filepath)
    if here not in sys.path:
        sys.path.insert(0, here)
    import importlib
    import seam_check
    importlib.reload(seam_check)

    prepare()
    detach()
    base = seam_check.snapshot()
    names = sorted(base)
    pairs = []
    for i, a in enumerate(names):
        for b in names[i + 1:]:
            d = seam_check.gap(base[a], base[b])
            if d <= seam_check.CONTACT:
                pairs.append((a, b))
    print("seams through the walk -- %d pairs that touch at bind, measured every frame"
          % len(pairs))

    attach()
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
    print("  %-22s %-22s %9s %8s" % ("part", "part", "worst mm", "at frame"))
    for (a, b), (d, f) in sorted(worst.items(), key=lambda kv: -kv[1][0]):
        flag = ""
        if d * 1000.0 > limit_mm:
            flag = "   <-- FAIL"
            fails += 1
        print("  %-22s %-22s %9.1f %8d%s" % (a, b, d * 1000.0, f, flag))
    print("  %s -- %d of %d pairs exceed %.0f mm during the cycle"
          % ("PASS" if not fails else "FAIL", fails, len(pairs), limit_mm))
    detach()
    return fails
