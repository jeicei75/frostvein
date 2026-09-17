"""Round 16, Stage B -- paint. One material, one packed 512 atlas, Closest.

    blender --background --factory-startup --python src-assets/blender/paint_r16.py

r15 s4.4 is the whole specification and it is unusual in one way worth restating:
**every flat face maps to the CENTRE of one palette cell**, so a face is exactly one
colour and "crisp value steps, no gradients" holds BY CONSTRUCTION rather than by
being checked afterwards. Nothing here can produce a gradient.

VALUE STEPS COME FROM THE POLYGON NORMAL, not from which part a face belongs to.
That is what makes the rotated 40 deg arms and r16's chamfers step like axis-aligned
boxes: the step is a function of the normal alone, so a wedge face lands on a real
intermediate value instead of borrowing its neighbour's. It is also the reason the
wedges are worth anything at all -- a third plane that painted the same as the front
would be invisible.

THE FACE IS AN ISLAND, PROJECTED IN WORLD COORDINATES (r15 s4.4). The eyes therefore
sit on the 0.807 H eye line because the projection puts them there, and cannot drift
when geometry moves. No smart_project anywhere.

r15 s4.4's last clause, which cost round 14 its brows: **every box whose front stands
proud of the face must be ON the island.** A proud box left on a flat palette cell
covers the paint behind it. Here that is decided by geometry -- any +Y face at or in
front of the face plane, on a head part -- not by a hand-kept list that can go stale.
"""

import os
import sys

import bpy
import numpy as np
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import dwarf_r16 as D              # noqa: E402

ATLAS = 512
CELL = 32                          # px; a face samples the middle of one of these
COLS = ATLAS // CELL

# Island: world-projected, in the clear area below the cell grid.
ISL_X0, ISL_Y0, ISL_W, ISL_H = 0, 256, 256, 192
ISL_XH = 0.21                      # island covers x in +-0.21 H ...
ISL_Z0, ISL_Z1 = 0.70, 0.97        # ... and z in 0.70..0.97 H

# The approved ten plus r3's 23 value steps. NOT a new palette -- r16 s8 forbids
# re-deriving it, and the question is closed (r15 s5). Ordered light -> dark, and
# per r15 s4.4 the BASE IS THE LIGHT CELL with the approved cell as a shadow step:
# round 14 built hair on #34271C and beard on #5E4632 as bases, which through the
# orientation multipliers ranged down to #221A12 -- black in all but name, and the
# two masses merged. Hair is based on #513C2B and beard on #826145 instead.
RAMPS = {
    "skin":  ["F3E2D2", "E9D2BB", "BAA896", "8F7A62"],
    "hair":  ["513C2B", "423123", "34271C", "221A12"],
    "beard": ["826145", "6B5039", "5E4632", "493E32"],
    "tunic": ["7DA18C", "5F7A6A", "44584C", "33423A"],
    "pants": ["63695B", "474B41", "383B33", "2B2E28"],
    "metal": ["CBD6CE", "A9B2AC", "707572", "4F5350"],
    "wood":  ["AF8765", "8B6B50", "6E543F", "493E32"],
    "trunk": ["8F7A62", "6B5B49", "4A3B2E", "332920"],
    "flame": ["F7CE94", "F0A63C", "C8842A", "8A5A1C"],
    "white": ["FFFFFF", "D8E4EC", "AFBAC2", "8A939A"],
    # Added after comparing against r14 side by side: r16 carries MORE cage faces
    # than r14 (1188 vs 1032) and still read flatter, because value here comes from
    # the normal alone -- so every box of a part facing the same way painted one
    # colour, while the art varies by FEATURE. These three are what the art varies.
    "boot":  ["6B5B49", "4A3B2E", "332920", "221A12"],   # boots are dark in front.png
    "cuff":  ["AF8765", "8B6B50", "6E543F", "493E32"],   # the lighter band at the top
    "hem":   ["98AB9D", "7DA18C", "5F7A6A", "44584C"],   # tunic hem and collar

}
FAMS = list(RAMPS)

# A key from the front, above and to the left, the same direction the lit diagnostic
# uses. Quantised to four steps -- the ramp depth, not a free parameter.
LIGHT = Vector((0.30, 0.45, 0.84)).normalized()

# FIVE steps, not four, and the reason is structural rather than a taste call.
# The key gives each orientation a value: top +0.834, lit corner +0.527, front
# +0.447, lit side +0.298, then everything facing away negative. With only four
# steps there is no set of breaks that separates all of top / corner / front / side:
# the first cut put FRONT AND SIDE ON THE SAME CELL, so the figure painted flat no
# matter what the geometry did, and the chamfer painted identical to the front --
# which makes r16's whole third plane invisible. Five steps separate them:
#   0 top   1 lit corner   2 front   3 lit side   4 turned away
STEPS = 5
BREAKS = (0.65, 0.49, 0.37, -0.20)

FACE_PLANE = D.fy(82) * D.H         # side-left col 82: where the face front sits
LAST_BUILD = None
HEAD_PARTS = {"head", "moustache"}


def rgb(hexcode):
    """The cell's bytes, NOT linearised.

    The first cut linearised here, and Blender wrote that float buffer straight out
    as 8-bit: #F3E2D2 shipped as #E5C2A4, every cell a full sRGB decode too dark,
    on EVERY colour in the atlas. It passed check_asset (which only counts and
    bounds the palette, it does not know what the cells were meant to be) and it
    passed by eye in Blender, because the same buffer was being rendered. The file
    on disk is the artifact, so the atlas is authored in bytes and the image is
    RELOADED from the saved PNG before it is rendered or packed -- after which what
    Blender shows and what ships are the same pixels.
    """
    return [int(hexcode[i:i + 2], 16) / 255.0 for i in (0, 2, 4)]


def step_of(n):
    k = n.dot(LIGHT)
    for i, b in enumerate(BREAKS):
        if k > b:
            return i
    return STEPS - 1


def blend(a, b, t):
    va, vb = rgb(a), rgb(b)
    return "".join(f"{int(round((x * (1 - t) + y * t) * 255)):02X}"
                   for x, y in zip(va, vb))


def ramp5(fam):
    """The four recorded anchors, with ONE interpolated step between the lightest
    and the approved cell so the lit corner has somewhere of its own to sit. Every
    anchor is a recorded palette cell; only the inserted step is derived, and it is
    derived BETWEEN two of them rather than invented (r16 s8)."""
    a = RAMPS[fam]
    return [a[0], blend(a[0], a[1], 0.5), a[1], a[2], a[3]]


def cell_rc(fam, step):
    idx = FAMS.index(fam) * STEPS + step
    return idx // COLS, idx % COLS


def cell_uv(fam, step):
    r, c = cell_rc(fam, step)
    return ((c + 0.5) * CELL / ATLAS, 1.0 - (r + 0.5) * CELL / ATLAS)


def on_island(part, me, poly):
    """A +Y face on a head part, at or proud of the face plane (r15 s4.4)."""
    if part not in HEAD_PARTS or poly.normal.y < 0.5:
        return False
    if min(me.vertices[v].co.y for v in poly.vertices) < FACE_PLANE - 1e-4:
        return False
    z = poly.center.z / D.H
    return ISL_Z0 - 1e-4 <= z <= ISL_Z1 + 1e-4


def island_uv(co):
    u = (co.x / D.H + ISL_XH) / (2 * ISL_XH)
    v = (co.z / D.H - ISL_Z0) / (ISL_Z1 - ISL_Z0)
    u = ISL_X0 + min(max(u, 0.0), 1.0) * ISL_W
    v = ISL_Y0 + min(max(v, 0.0), 1.0) * ISL_H
    return (u / ATLAS, 1.0 - v / ATLAS)


def uv_fn(part, fam, me, poly):
    if on_island(part, me, poly):
        return [island_uv(me.vertices[v].co) for v in poly.vertices]
    return [cell_uv(fam, step_of(poly.normal))] * len(poly.vertices)


# --------------------------------------------------------------------- the atlas

def rect(img, x0, y0, x1, y1, hexcode):
    img[y0:y1, x0:x1] = rgb(hexcode)


def sym_rect(img, xa, xb, z0, z1, hexcode):
    """Paint a band mirrored about the centre line, in H units, into the island."""
    for s in (-1, 1):
        lo, hi = sorted((s * xa, s * xb))
        px0 = ISL_X0 + int(round((lo + ISL_XH) / (2 * ISL_XH) * ISL_W))
        px1 = ISL_X0 + int(round((hi + ISL_XH) / (2 * ISL_XH) * ISL_W))
        py0 = ISL_Y0 + int(round((z0 - ISL_Z0) / (ISL_Z1 - ISL_Z0) * ISL_H))
        py1 = ISL_Y0 + int(round((z1 - ISL_Z0) / (ISL_Z1 - ISL_Z0) * ISL_H))
        rect(img, px0, py0, px1, py1, hexcode)


def build_atlas():
    img = np.zeros((ATLAS, ATLAS, 3), np.float32)
    img[:] = rgb("FF00FF")                       # anything unmapped screams
    for fam in FAMS:
        for step, hexcode in enumerate(ramp5(fam)):
            r, c = cell_rc(fam, step)
            rect(img, c * CELL, r * CELL, (c + 1) * CELL, (r + 1) * CELL, hexcode)

    # ---- the face, painted in world coordinates. Rows below are SOURCE rows of
    # front.png converted to z/H, so every feature sits where the art draws it.
    rect(img, ISL_X0, ISL_Y0, ISL_X0 + ISL_W, ISL_Y0 + ISL_H, "E9D2BB")
    sym_rect(img, 0.000, ISL_XH, 0.893, 0.970, "F3E2D2")     # forehead catches light
    sym_rect(img, 0.121, ISL_XH, ISL_Z0, 0.970, "8F7A62")    # past the cheek: hair
    sym_rect(img, 0.100, 0.121, ISL_Z0, 0.970, "BAA896")     # cheeks turn away
    # brows: front rows 23-28, two masses with the bridge recessed between them (f100)
    sym_rect(img, 0.032, 0.118, 0.843, 0.893, "34271C")
    # eyes on the 0.807 H eye line (front rows 30-34)
    sym_rect(img, 0.039, 0.107, 0.793, 0.839, "D8E4EC")
    sym_rect(img, 0.054, 0.089, 0.800, 0.829, "332920")
    # nose: down the centre, rows 31-43, lit on top and shaded on its left side
    sym_rect(img, 0.000, 0.029, 0.729, 0.843, "F3E2D2")
    sym_rect(img, 0.018, 0.029, 0.729, 0.807, "BAA896")
    # moustache, over the beard's dip at rows 43-49
    sym_rect(img, 0.000, 0.093, ISL_Z0, 0.779, "6B5039")
    sym_rect(img, 0.000, 0.021, 0.750, 0.779, "826145")      # gap under the nose
    return img


def canary_faces(col, px):
    """How many faces sample atlas the atlas never painted (r15 s4.4's real gate)."""
    n = 0
    for ob in col.objects:
        if ob.type != "MESH" or not ob.data.uv_layers:
            continue
        uv = ob.data.uv_layers[0]
        for poly in ob.data.polygons:
            u, v = uv.data[poly.loop_indices[0]].uv
            y = min(max(int((1.0 - v) * (ATLAS - 1)), 0), ATLAS - 1)
            x = min(max(int(u * (ATLAS - 1)), 0), ATLAS - 1)
            if np.allclose(px[y, x], rgb("FF00FF")):
                n += 1
    return n


def make_material(img_dat):
    mat = bpy.data.materials.new("M_VoxelDwarf_r16")
    mat.use_nodes = True
    mat.use_backface_culling = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = 0.75
    bsdf.inputs["Specular IOR Level"].default_value = 0.5
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = img_dat
    tex.interpolation = "Closest"                # r15 s4.4 -- no filtering, ever
    tex.location = (-380, 220)
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    return mat


def paint(facet=True):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    D.FACET = facet
    b = D.build()
    col = b.to_objects(uv_fn=uv_fn)

    px = build_atlas()
    img_dat = bpy.data.images.new("T_VoxelDwarf_r16", ATLAS, ATLAS, alpha=False)
    rgba = np.ones((ATLAS, ATLAS, 4), np.float32)
    rgba[..., :3] = px
    img_dat.pixels.foreach_set(rgba[::-1].ravel())
    tex_path = os.path.join(HERE, "textures", "T_VoxelDwarf_r16.png")
    img_dat.filepath_raw = tex_path
    img_dat.file_format = "PNG"
    img_dat.save()
    img_dat.pack()

    stray = canary_faces(col, px)
    if stray:
        raise SystemExit(f"paint: {stray} faces sample unpainted atlas -- UVs drifted")
    # No face lands on it, so bury the canary rather than ship a magenta the
    # palette does not contain; it has already done its job by this line.
    keep = px.copy()
    px[np.all(np.isclose(px, rgb("FF00FF")), axis=-1)] = rgb("332920")
    rgba[..., :3] = px
    img_dat.pixels.foreach_set(rgba[::-1].ravel())
    img_dat.save()
    img_dat.reload()                    # render what actually shipped, not the buffer

    mat = make_material(img_dat)
    for ob in col.objects:
        ob.data.materials.clear()
        ob.data.materials.append(mat)

    # r15 s7.9: report the CELL LIST with any skin figure, because the visible-skin
    # check classifies to the nearest cell and new mid-browns pull antialiased
    # boundary pixels out of the skin family with no model change.
    n_island = sum(1 for ob in col.objects
                   for poly in ob.data.polygons
                   if on_island(ob.name.split("_", 1)[1], ob.data, poly))
    print(f"\n  atlas {ATLAS}px, {len(FAMS)} families x 4 steps = {len(FAMS)*4} cells")
    for fam in FAMS:
        print(f"    {fam:6s} {'  '.join('#' + c for c in ramp5(fam))}")
    print(f"  face island {ISL_W}x{ISL_H} px, world-projected; {n_island} polygons on it")
    print(f"  texture -> {tex_path}")
    global LAST_BUILD
    LAST_BUILD = b
    return col, img_dat


if __name__ == "__main__":
    col, _ = paint(facet=True)
    out = os.environ.get("R16_BLEND")
    if out:
        bpy.ops.wm.save_as_mainfile(filepath=out)
        print("saved", out)
