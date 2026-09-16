"""Round 16, Stage C -- the 19-joint rig, with weights assigned BY BOX.

    blender --background --factory-startup --python src-assets/blender/rig_r16.py

r15 s8 Stage C is specific about the one thing that is easy to get wrong: rigid
weights are assigned **by BOX, not by object**. The neck box lives inside the head
part but belongs to `neck`; the torso's waist box to `spine`; the sleeve's forearm
box to `elbow`. Assigning per OBJECT would put the whole sleeve on `shoulder` and
the forearm would not bend.

The generator already splits every part that straddles a joint -- leg into thigh and
shin, torso into chest and waist, sleeve into cap, upper arm and forearm -- so that
each joint owns whole boxes. r15 s8: a single leg box has to be divided
vertex-by-vertex and shears.

WHICH box gets which joint is decided from each chunk's own EXTENTS, never from its
index in a list (r15 s7.2 -- every restructure shifts indices silently, and the check
then measures a different mass while still printing a number; that happened four
times in round 14).

The exporter owns the joint SET and will refuse anything else, so JOINTS is imported
from it rather than retyped here.
"""

import math
import os
import sys

import bpy
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import dwarf_r16 as D              # noqa: E402
import paint_r16 as P              # noqa: E402
import render_r16 as R             # noqa: E402

H = D.H
SH_X, SH_Z = D.hw(29.6), D.zb(48)
ARM_DIR = Vector((math.sin(math.radians(40.0)), 0.0, -math.cos(math.radians(40.0))))

# +X is the character's RIGHT: facing +Y with up +Z, right = forward x up = Y x Z = X.
SIDE = {-1: "L", 1: "R"}


def bone_table():
    """(name, head, tail, parent) in H units. Joints sit where the geometry bends."""
    out = [
        ("root",  (0, 0, 0.00), (0, 0, 0.06), None),
        ("hips",  (0, 0, 0.21), (0, 0, 0.34), "root"),
        ("spine", (0, 0, 0.34), (0, 0, 0.52), "hips"),
        ("chest", (0, 0, 0.52), (0, 0, 0.69), "spine"),
        ("neck",  (0, 0, 0.69), (0, 0, 0.74), "chest"),
        ("head",  (0, 0, 0.74), (0, 0, 1.00), "neck"),
        # the beard is its own joint (it is in the 19) and hangs off the head
        ("beard", (0, D.fy(70), 0.72), (0, D.fy(86), 0.48), "head"),
    ]
    for s in (-1, 1):
        n = SIDE[s]
        sh = Vector((s * SH_X, 0.0, SH_Z))
        el = sh + ARM_DIR * 0.20 * Vector((s, 1, 1))
        hd = sh + ARM_DIR * 0.31 * Vector((s, 1, 1))
        tip = sh + ARM_DIR * 0.37 * Vector((s, 1, 1))
        out += [
            (f"shoulder.{n}", tuple(sh), tuple(el), "chest"),
            (f"elbow.{n}",    tuple(el), tuple(hd), f"shoulder.{n}"),
            (f"hand.{n}",     tuple(hd), tuple(tip), f"elbow.{n}"),
        ]
        lx = s * 0.118
        out += [
            (f"hip.{n}",  (lx, 0, 0.207), (lx, 0, 0.150), "hips"),
            (f"knee.{n}", (lx, 0, 0.150), (lx, 0, 0.100), f"hip.{n}"),
            (f"foot.{n}", (lx, 0, 0.100), (lx, D.fy(76), 0.02), f"knee.{n}"),
        ]
    return out


def joint_for(part, verts):
    """The joint a BOX belongs to, from the box's own extents (r15 s7.2)."""
    zs = [v[2] for v in verts]
    xs = [v[0] for v in verts]
    z0, z1 = min(zs), max(zs)
    n = SIDE[1] if (sum(xs) / len(xs)) > 0 else SIDE[-1]

    if part in ("hair", "head", "moustache"):
        return "head"
    if part == "neck":
        return "neck"
    if part == "beard":
        return "beard"
    if part == "torso":
        # the waist box goes to spine, the chest box to chest (r15 s8)
        return "chest" if z1 > D.zb(76) + 1e-6 else "spine"
    if part in ("pack", "strap"):
        return "chest"
    if part in ("belt", "buckle"):
        return "spine"
    if part == "skirt":
        return "hips"
    if part == "hem":
        # two boxes: the collar at the neckline rides the chest, the hem band
        # at the tunic's bottom rides the hips. Told apart by height, not order.
        return "chest" if z0 > 0.5 else "hips"
    if part == "sleeve":
        # cap and upper arm ride the shoulder; the FOREARM box is the elbow's.
        # Measured ALONG THE ARM, not in z: the arm is rotated 40 deg, so a plain
        # z threshold mixed them up -- the upper arm's lowest CORNER swings below
        # the forearm's top and it was binding to elbow. This is r15 s7.3 in a new
        # place: an axis-aligned reading of a rotated box lies about where it is.
        s_ = 1 if (sum(xs) / len(xs)) > 0 else -1
        ctr = Vector((sum(xs) / len(xs),
                      sum(v[1] for v in verts) / len(verts),
                      sum(zs) / len(zs)))
        axis = Vector((s_ * ARM_DIR.x, ARM_DIR.y, ARM_DIR.z))
        t = (ctr - Vector((s_ * SH_X, 0.0, SH_Z))).dot(axis)
        return f"shoulder.{n}" if t < 0.20 else f"elbow.{n}"
    if part == "glove":
        return f"hand.{n}"
    if part == "leg":
        return f"hip.{n}" if z0 >= D.zb(126) - 1e-6 else f"knee.{n}"
    if part == "boot":
        return f"foot.{n}"
    if part == "pickaxe":
        return "hand.R"          # held on the character's right, as the sheet draws
    if part == "lantern":
        return "hand.L"
    raise SystemExit(f"rig: no joint rule for part {part!r}")


def build_rig(b, col):
    arm_dat = bpy.data.armatures.new("A_VoxelDwarf_r16")
    arm = bpy.data.objects.new("A_VoxelDwarf_r16", arm_dat)
    col.objects.link(arm)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="EDIT")
    made = {}
    for name, head, tail, parent in bone_table():
        eb = arm_dat.edit_bones.new(name)
        eb.head = Vector(head) * H
        eb.tail = Vector(tail) * H
        eb.use_deform = True
        made[name] = eb
    for name, _, _, parent in bone_table():
        if parent:
            made[name].parent = made[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    return arm


def skin(b, col, arm):
    """One vertex group per joint; every vertex of a box at 1.0 on its own joint."""
    counts = {}
    for ob in col.objects:
        if ob.type != "MESH":
            continue
        part = ob.name.split("_", 1)[1]
        groups = {}
        off = 0
        for (v, f) in b.parts[part]:
            j = joint_for(part, v)
            if j not in groups:
                groups[j] = ob.vertex_groups.new(name=j)
            groups[j].add(list(range(off, off + len(v))), 1.0, "REPLACE")
            counts[j] = counts.get(j, 0) + 1
            off += len(v)
        assert off == len(ob.data.vertices), f"{part}: chunk vertex ranges drifted"
        ob.parent = arm
        mod = ob.modifiers.new("Armature", "ARMATURE")
        mod.object = arm
        mod.use_vertex_groups = True
    return counts


def main():
    col, _ = P.paint(facet=True)
    b = P.LAST_BUILD
    arm = build_rig(b, col)
    counts = skin(b, col, arm)

    from export_dwarf import JOINTS
    used = set(counts)
    print(f"\n  rig: {len(bpy.data.armatures['A_VoxelDwarf_r16'].bones)} bones, "
          f"{sum(counts.values())} boxes bound")
    print(f"  joints with geometry : {len(used)} of {len(JOINTS)}")
    missing = sorted(JOINTS - used)
    extra = sorted(used - JOINTS)
    print(f"  boxes per joint      : "
          + ", ".join(f"{k}={v}" for k, v in sorted(counts.items())))
    if missing:
        print(f"  joints carrying NO geometry (legal, still exported): {missing}")
    if extra:
        raise SystemExit(f"rig: joints outside the 19-joint contract: {extra}")

    # Light the saved file. paint() starts from an EMPTY factory scene, so without
    # this the .blend ships with no world and no lamps and Rendered shading is solid
    # black when you open it -- the figure is fine, there is simply nothing lighting
    # it. The exporter drops lights and cameras, so this cannot reach the GLB.
    R.lit_scene()
    R.lit_camera()

    out = os.environ.get("R16_BLEND",
                         os.path.join(HERE, "SM_VoxelDwarf_Miner01_r16.blend"))
    bpy.ops.wm.save_as_mainfile(filepath=out)
    print("  saved", out)


if __name__ == "__main__":
    main()
