"""Round 20 cold-run proof: the chain regenerates the saved file.

    git show HEAD:src-assets/blender/SM_VoxelDwarf_Miner01.blend > <scratch>/head.blend
    blender -b --factory-startup --python src-assets/blender/coldrun_r20.py -- <scratch>/head.blend

1. HEAD's committed .blend: per-part geometry fingerprints (the 21 parts round 19 shipped);
2. the saved .blend: the same, plus r17_axe, plus every action's fcurve hash;
3. an empty scene, then dwarf_r17.build("C") -> walk_r18.build() -> work_r19.build_dig(),
   build_carry() -> cut_r20.build_cut(): the same fingerprints and hashes again.
Nothing is saved.
"""

import os
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
SAVED = os.path.join(HERE, "SM_VoxelDwarf_Miner01.blend")
HEAD = sys.argv[sys.argv.index("--") + 1]


def fingerprint():
    import cut_r20 as C
    parts, whole = C.geometry_hash()
    arm = C.arm_ob()
    acts = {a.name: C.fcurve_hash(a.name) for a in bpy.data.actions}
    old20 = {n: C.fcurve_hash(n, C.V.BONES) for n in C.OLD if n in acts}
    tris = sum(len(p.vertices) - 2 for n in parts for p in bpy.data.objects[n].data.polygons)
    return dict(parts=parts, whole=whole, joints=len(arm.data.bones), acts=acts, old20=old20,
                tris=tris)


def show(tag, fp):
    print("%-6s %d parts, %d tris, %d joints, geometry %s" % (tag, len(fp["parts"]), fp["tris"],
                                                            fp["joints"], fp["whole"][:16]))
    for n, h in sorted(fp["acts"].items()):
        print("       %-5s %s" % (n, h))


bpy.ops.wm.open_mainfile(filepath=HEAD)
head = fingerprint()
show("HEAD", head)

bpy.ops.wm.open_mainfile(filepath=SAVED)
saved = fingerprint()
show("saved", saved)

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.context.preferences.filepaths.save_version = 0
import dwarf_r17                                 # noqa: E402
import walk_r18                                  # noqa: E402
import work_r19                                  # noqa: E402
import cut_r20                                   # noqa: E402
dwarf_r17.build("C")
walk_r18.build()
work_r19.build_dig()
work_r19.build_carry()
cut_r20.build_cut()
cut_r20.detach()
cold = fingerprint()
show("cold", cold)

old = sorted(head["parts"])
checks = [
    ("old 21 parts: HEAD == saved == cold",
     all(head["parts"][n] == saved["parts"][n] == cold["parts"][n] for n in old)),
    ("r17_axe: saved == cold", saved["parts"].get("r17_axe") == cold["parts"].get("r17_axe")),
    ("joints: HEAD 20, saved 21, cold 21",
     (head["joints"], saved["joints"], cold["joints"]) == (20, 21, 21)),
    ("actions exactly Carry, Cut, Dig, Walk (saved, cold)",
     sorted(saved["acts"]) == sorted(cold["acts"]) == ["Carry", "Cut", "Dig", "Walk"]),
    ("every action: saved == cold", saved["acts"] == cold["acts"]),
    ("Walk/Dig/Carry 20-joint curves == round 19 (saved, cold)",
     all(saved["old20"][n] == cold["old20"][n] == cut_r20.ROUND19[n] for n in cut_r20.OLD)),
    ("Walk/Dig/Carry HEAD hashes == round 19",
     all(head["acts"][n] == cut_r20.ROUND19[n] for n in cut_r20.OLD)),
]
for label, ok in checks:
    print("%-58s %s" % (label, "YES" if ok else "NO"))
print("COLDRUN %s" % ("PASS" if all(ok for _, ok in checks) else "FAIL"))
