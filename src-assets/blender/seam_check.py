"""Range-of-motion seam check for the voxel dwarf.

Answers one question: when a joint rotates, does any pair of parts that TOUCH at bind pull
apart into a visible slit?

Run it in the live session (or headless against the .blend):

    exec(open(r"src-assets/blender/seam_check.py").read()); report()

Why surface-to-surface and not vertex-to-vertex
-----------------------------------------------
The obvious metric -- nearest vertex of A to nearest vertex of B -- finds ONE contacting
pair on this figure instead of 24, and it is not a tuning problem. Every part here is a
stack of boxes, and a small box seated inside a big one (the shin inside the boot cuff, a
lock inside the hair mass) has its vertices at the CORNERS, metres of box-width away from
the large box's own corners, while the two surfaces are flush. Vertex distance measures the
corners; the eye sees the surfaces. So each query is a point against the other mesh's
BVH -- `find_nearest` returns the closest point ON the surface, not the closest vertex --
and it is run in BOTH directions, taking the min, because A's vertices can be far from B's
surface while B's are right up against A's.

The numbers are worst-case over the joint range, which is the number that matters: a seam
that is shut at bind and open at -20 degrees shimmers for most of a walk cycle.
"""

import math

import bpy
from mathutils.bvhtree import BVHTree

# Contact is generous on purpose: parts meant to meet are built to overlap by OVERLAP
# (4 mm), so a pair sitting at 1.6 mm apart is already the defect, not a near miss.
CONTACT = 0.003
PASS_MM = 10.0

# Joint ranges, degrees. Axis is X unless named -- the head swivels about Z and the wrist
# about Y, which is how they are actually animated.
#
# r18 widened the leg entries to the angles the `Walk` action actually reaches (walk_r18.py,
# joint_range()), because a generic ROM is not evidence about the poses the cycle uses. Two
# of the three were on the wrong side of zero to begin with:
#
#   knee  the old (0, 70) was HYPERextension. hip/knee point down at bind with local X =
#         world X, so a POSITIVE turn swings the shin forward. The walk runs -82 .. -40 and
#         never once enters the old window.
#   foot  the walk needs +45 of local dorsiflexion, not because the ankle bends that far in
#         the world -- absolute foot pitch only runs -18 .. +12 -- but because the foot has
#         to cancel a knee that is flexed 40-82 to keep the sole flat on the ground.
#
# The old bounds are kept where they are the wider of the two, so this is a union and not a
# replacement.
RANGES = {
    "hip":      ("X", (-45.0, 62.0)),
    "knee":     ("X", (-83.0, 70.0)),
    "foot":     ("X", (-25.0, 45.0)),
    "shoulder": ("X", (-55.0, 40.0)),
    "elbow":    ("X", (-70.0, 0.0)),
    "hand":     ("Y", (-35.0, 35.0)),
    "spine":    ("X", (-15.0, 15.0)),
    "chest":    ("X", (-12.0, 12.0)),
    "neck":     ("X", (-20.0, 20.0)),
    "head":     ("Z", (-35.0, 35.0)),
    "hips":     ("X", (-12.0, 12.0)),
    "beard":    ("X", (-4.0, 25.0)),
}
STEPS = 6                       # samples across each range, endpoints included

AXIS_I = {"X": 0, "Y": 1, "Z": 2}


def armature():
    arms = [o for o in bpy.data.objects if o.type == 'ARMATURE']
    if len(arms) != 1:
        raise SystemExit("seam_check: expected one armature, found %d" % len(arms))
    return arms[0]


def meshes():
    return sorted((o for o in bpy.data.objects if o.type == 'MESH'), key=lambda o: o.name)


def bind():
    arm = armature()
    for pb in arm.pose.bones:
        pb.rotation_mode = 'XYZ'
        pb.rotation_euler = (0.0, 0.0, 0.0)
        pb.location = (0.0, 0.0, 0.0)
        pb.scale = (1.0, 1.0, 1.0)
    bpy.context.view_layer.update()


def world_geom(ob, deps):
    """Evaluated mesh of `ob`, in world space, as (verts, polys) for BVHTree.FromPolygons."""
    ev = ob.evaluated_get(deps)
    me = ev.to_mesh()
    mw = ob.matrix_world
    verts = [mw @ v.co for v in me.vertices]
    polys = [tuple(p.vertices) for p in me.polygons]
    ev.to_mesh_clear()
    return verts, polys


def snapshot(names=None):
    """{name: (verts, tree)} for the CURRENT pose."""
    deps = bpy.context.evaluated_depsgraph_get()
    out = {}
    for ob in meshes():
        if names is not None and ob.name not in names:
            continue
        verts, polys = world_geom(ob, deps)
        out[ob.name] = (verts, BVHTree.FromPolygons(verts, polys))
    return out


def gap(a, b):
    """Worst-case-honest surface-to-surface distance between two snapshot entries."""
    (va, ta), (vb, tb) = a, b
    best = float("inf")
    for v in va:
        loc, _, _, d = tb.find_nearest(v)
        if loc is not None and d < best:
            best = d
    for v in vb:
        loc, _, _, d = ta.find_nearest(v)
        if loc is not None and d < best:
            best = d
    return best


def bone_groups():
    """{object name: set of bone names it is weighted to}."""
    out = {}
    for ob in meshes():
        out[ob.name] = {g.name for g in ob.vertex_groups}
    return out


def descendants(arm, name):
    bone = arm.data.bones.get(name)
    if bone is None:
        return set()
    return {name} | {b.name for b in bone.children_recursive}


def report():
    arm = armature()
    bind()
    base = snapshot()
    groups = bone_groups()
    names = sorted(base)

    pairs = []
    for i, a in enumerate(names):
        for b in names[i + 1:]:
            d = gap(base[a], base[b])
            if d <= CONTACT:
                pairs.append((a, b, d))
    print("contacting pairs at bind (<= %.0f mm): %d" % (CONTACT * 1000, len(pairs)))

    worst = {(a, b): (d, "bind", 0.0) for a, b, d in pairs}

    for pb in arm.pose.bones:
        stem = pb.name.split(".")[0]
        if stem not in RANGES:
            continue
        axis, (lo, hi) = RANGES[stem]
        moved = descendants(arm, pb.name)
        touched = {n for n in names if groups[n] & moved}
        if not touched:
            continue
        live = [(a, b) for a, b, _ in pairs if a in touched or b in touched]
        if not live:
            continue
        pb.rotation_mode = 'XYZ'
        for s in range(STEPS):
            deg = lo + (hi - lo) * s / (STEPS - 1)
            if abs(deg) < 1e-9:
                continue
            pb.rotation_euler[AXIS_I[axis]] = math.radians(deg)
            bpy.context.view_layer.update()
            shot = snapshot(touched)
            for a, b in live:
                ga = shot.get(a, base[a])
                gb = shot.get(b, base[b])
                d = gap(ga, gb)
                if d > worst[(a, b)][0]:
                    worst[(a, b)] = (d, "%s %s" % (pb.name, axis), deg)
        pb.rotation_euler = (0.0, 0.0, 0.0)
        bpy.context.view_layer.update()

    print()
    print("%-26s %-26s %9s %9s  %s" % ("part", "part", "bind mm", "worst mm", "at"))
    rows = sorted(worst.items(), key=lambda kv: -kv[1][0])
    fails = 0
    for (a, b), (d, who, deg) in rows:
        at_bind = next(x for p, q, x in pairs if p == a and q == b)
        flag = ""
        if d * 1000 > PASS_MM:
            flag = "   <-- FAIL"
            fails += 1
        print("%-26s %-26s %9.1f %9.1f  %s %+.0f%s"
              % (a, b, at_bind * 1000, d * 1000, who, deg, flag))
    print()
    print("PASS" if not fails else "FAIL", "-- %d of %d pairs exceed %.0f mm"
          % (fails, len(rows), PASS_MM))
    bind()
    return rows


def sweep(bone, axis, degs):
    """Control: one pair's gap against the angle it is blamed on."""
    arm = armature()
    bind()
    base = snapshot()
    groups = bone_groups()
    moved = descendants(arm, bone)
    touched = {n for n in base if groups[n] & moved}
    pb = arm.pose.bones[bone]
    pb.rotation_mode = 'XYZ'
    names = sorted(base)
    pairs = []
    for i, a in enumerate(names):
        for b in names[i + 1:]:
            if (a in touched or b in touched) and gap(base[a], base[b]) <= CONTACT:
                pairs.append((a, b))
    for deg in degs:
        pb.rotation_euler = (0.0, 0.0, 0.0)
        pb.rotation_euler[AXIS_I[axis]] = math.radians(deg)
        bpy.context.view_layer.update()
        shot = snapshot(touched)
        out = []
        for a, b in pairs:
            d = gap(shot.get(a, base[a]), shot.get(b, base[b]))
            out.append("%s/%s %.1f" % (a.replace("r17_", ""), b.replace("r17_", ""), d * 1000))
        print("%s %s %+6.1f deg:  %s" % (bone, axis, deg, "   ".join(out)))
    bind()
