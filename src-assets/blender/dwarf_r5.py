"""Round 5 of SM_VoxelDwarf_Miner01: box-modelled from the ORTHOGRAPHIC sheet.

The .blend is the source of truth. This module exists so a part can be re-derived
after a correction without hand-editing 200 vertices through a socket, exactly as
round 4's in-blend text datablocks did -- it is an authoring helper, not a
generator, and nothing downstream reads it.

    blender --background --python src-assets/blender/dwarf_r5.py -- --step N

Step 0 seeds the file from round 4's: the palette atlas, the material and the
render settings are INHERITED verbatim and only the revision token changes.
Steps 1..12 each build one part, then the caller saves and renders.

THE MEASUREMENT FRAME. Everything is authored in the source pixels of
src-assets/references/dwarf-ortho/, so the numbers in this file can be checked
against the sheet by counting 5x5 blocks:

  * the figure spans rows 7 (crown) .. 147 (sole) in front.png and side-left.png,
    so 140 source pixels == 1.20 m and one source pixel == 8.571 mm;
  * front.png column 77 is the body's centre line  -> X;
  * side-left.png column 58 is the body's depth centre, +Y forward (he faces +Y);
  * back.png is 123 px tall for the same figure, so back-view pixels are scaled
    by 140/123 before they are used here.
"""

import os
import sys

import bpy
from mathutils import Vector

REV = "r5"
ASSET = "SM_VoxelDwarf_Miner01"
HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.dirname(HERE)
BLEND = os.path.join(HERE, "%s.blend" % ASSET)
COLLECTION = "%s_%s" % (ASSET, REV)
MATERIAL = "M_VoxelDwarf_%s" % REV
PALETTE = "T_VoxelDwarf_Palette_%s" % REV

# --- the measurement frame ---------------------------------------------------
HEIGHT = 1.20
PXH = 140.0
U = HEIGHT / PXH                 # 8.5714 mm per source pixel
ROW_SOLE = 147.0
COL_X0 = 77.0
COL_Y0 = 58.0


def Z(row):
    return (ROW_SOLE - row) * U


def Y(col):
    return (col - COL_Y0) * U


def P(px):
    return px * U


# --- the inherited palette ---------------------------------------------------
# Round 4's atlas, copied verbatim. Cell 15 was its one unspent slot and round 5
# leaves it unspent too: every part here is carried by a value step that already
# exists, and the only thing that wanted a new cell was a warm buckle, which is a
# HUE change and still out of scope. Do not re-derive any of these.
SKIN, BEARD, SNOW, TUNIC, PANTS, METAL, WOOD, TRUNK, HAIR, FLAME = range(10)
SKIN_DK = 10       # #BAA896 skin shadow plane
BEARD_LT = 11      # #826145 beard highlight
TUNIC_LT = 12      # #7DA18C tunic lit plane
TUNIC_DK = 13      # #44584C tunic turned-away plane
PANTS_LT = 14      # #63695B trouser lit plane
SPARE = 15         # round 5: see PALETTE_NOTE

ATLAS, CELL = 64, 16
FACE_DIRS = ("xlo", "xhi", "ylo", "yhi", "zlo", "zhi")


def cell_uv(cid):
    col, row = cid % 4, cid // 4
    return ((col * CELL + CELL / 2) / ATLAS, (row * CELL + CELL / 2) / ATLAS)


def shade(base, lit=None, dark=None, front=None, back=None):
    """One box's six cells under the round's single value convention.

    The step is TOP-vs-UNDERSIDE, not front-vs-back: +Z takes the lit value, -Z the
    turned-away one, and every vertical face keeps the base. Painting the -Y faces
    dark as well -- the first thing tried here -- reads fine from the front and
    turns the whole back view into mud, which is precisely the view back.png was
    added to fix. A flat-albedo asset should encode FORM, not a light direction.
    """
    lit = base if lit is None else lit
    dark = base if dark is None else dark
    front = base if front is None else front
    back = base if back is None else back
    return {"xlo": base, "xhi": base, "ylo": back, "yhi": front,
            "zlo": dark, "zhi": lit}


# --- box modelling -----------------------------------------------------------
def box_faces(lo, hi, cells):
    """(verts, faces, uvs) for one axis-aligned box; a None cell drops that face.

    Every face carries its own four vertices, so no normal is ever averaged
    across a hard edge and no quad can span a joint.
    """
    if isinstance(cells, int):
        cells = {d: cells for d in FACE_DIRS}
    x0, y0, z0 = lo
    x1, y1, z1 = hi
    corner = {
        "xlo": [(x0, y0, z0), (x0, y1, z0), (x0, y1, z1), (x0, y0, z1)],
        "xhi": [(x1, y1, z0), (x1, y0, z0), (x1, y0, z1), (x1, y1, z1)],
        "ylo": [(x1, y0, z0), (x0, y0, z0), (x0, y0, z1), (x1, y0, z1)],
        "yhi": [(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)],
        "zlo": [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0)],
        "zhi": [(x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1)],
    }
    verts, faces, uvs = [], [], []
    for d in FACE_DIRS:
        cid = cells.get(d)
        if cid is None:
            continue
        base = len(verts)
        verts.extend(corner[d])
        faces.append((base, base + 1, base + 2, base + 3))
        uvs.extend([cell_uv(cid)] * 4)
    return verts, faces, uvs


def B(r0, r1, x0, x1, y0, y1, cells):
    """One box in SOURCE-PIXEL coordinates.

    r0/r1 are front-view rows (r0 above r1), x0/x1 are front-view columns offset
    from the centre line, y0/y1 are side-left columns. Returns the (lo, hi, cells)
    triple the mesh builder wants, in metres.
    """
    lo = (P(min(x0, x1)), Y(min(y0, y1)), Z(max(r0, r1)))
    hi = (P(max(x0, x1)), Y(max(y0, y1)), Z(min(r0, r1)))
    return (lo, hi, cells)


def panel_y(y_col, rows):
    """Tile a +Y-facing plane with flat colour regions that EXACTLY tile it.

    `rows` is [(r0, r1, [(x0, x1, cell), ...]), ...] in source pixels. The regions
    neither overlap nor leave gaps, so one surface can carry several palette cells
    without a second surface hovering over it and z-fighting. This is colour per
    face, not a texture and not a groove: no geometry is cut.
    """
    verts, faces, uvs = [], [], []
    yy = Y(y_col)
    for r0, r1, segs in rows:
        z0, z1 = Z(max(r0, r1)), Z(min(r0, r1))
        for x0, x1, cid in segs:
            if cid is None:
                continue
            a, b = P(min(x0, x1)), P(max(x0, x1))
            base = len(verts)
            verts.extend([(a, yy, z0), (b, yy, z0), (b, yy, z1), (a, yy, z1)])
            faces.append((base, base + 1, base + 2, base + 3))
            uvs.extend([cell_uv(cid)] * 4)
    return verts, faces, uvs


def drop_part(name):
    ob = bpy.data.objects.get(name)
    if ob:
        me = ob.data
        bpy.data.objects.remove(ob, do_unlink=True)
        if me.users == 0:
            bpy.data.meshes.remove(me)


def make_part(name, boxes=(), panels=()):
    verts, faces, uvs = [], [], []

    def absorb(v, f, u):
        off = len(verts)
        verts.extend(v)
        faces.extend(tuple(i + off for i in q) for q in f)
        uvs.extend(u)

    for lo, hi, cells in boxes:
        absorb(*box_faces(lo, hi, cells))
    for chunk in panels:
        absorb(*chunk)

    drop_part(name)
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    me.validate()
    layer = me.uv_layers.new(name="UVMap")
    for i, uv in enumerate(uvs):
        layer.data[i].uv = uv
    for poly in me.polygons:
        poly.use_smooth = False
    me.materials.append(bpy.data.materials[MATERIAL])
    ob = bpy.data.objects.new(name, me)
    bpy.data.collections[COLLECTION].objects.link(ob)
    return ob


def socket(name, location, child):
    """A declared mount point. The part is authored in world space and the empty
    is slid under it, so a variant can swap the child without moving anything."""
    emp = bpy.data.objects.get(name)
    if emp is None:
        emp = bpy.data.objects.new(name, None)
        emp.empty_display_type = 'PLAIN_AXES'
        emp.empty_display_size = 0.03
        bpy.data.collections[COLLECTION].objects.link(emp)
    emp.location = location
    bpy.context.view_layer.update()
    child.parent = emp
    child.matrix_parent_inverse = emp.matrix_world.inverted()
    return emp


# =============================================================================
# The parts. Every number is a source-pixel reading off dwarf-ortho/.
# =============================================================================

# vertical landmarks, in front/side-view rows
R_CROWN, R_DOME_B, R_DOME_C, R_SKULL = 7, 10, 12, 15
R_BROW, R_EYE, R_CHEEK, R_NOSE_B = 24, 30, 35, 44
R_EAR_T, R_EAR_B = 28, 45
R_JAW = 45                  # head bottom, raised 3 px off the sheet so a neck reads
R_COLLAR = 51
R_SHOULDER = 46
R_CUFF = 68                 # sleeve ends, bare forearm begins
R_HAND_T, R_HAND_B = 88, 100
R_BELT_T, R_BELT_B = 88, 99
R_BEARD_TIP = 82
R_SKIRT = 99
R_HEM = 118
R_CUFF_BOOT_T, R_BOOT_T = 124, 132
R_TOE = 138

# horizontal landmarks, in front-view pixels from the centre line
HX = 23.5                   # head half-width WITH hair  (47 px = 0.336 H)
SKULL_X = 21.5              # skull under the 2 px hair shell
EAR_X = 27.0
NECK_X = 9.0
CHEST_X = 22.0
WAIST_X = 23.0
SKIRT_X = 30.0
SHOULDER_X = 37.0           # 74 px = 0.528 H, measured on back.png
BEARD_X = 22.0

# depth landmarks, in side-left columns (58 is the body centre)
Y_HEAD_B, Y_HEAD_F = 36.0, 83.0
Y_SKULL_B, Y_SKULL_F = 38.0, 81.0
Y_NOSE = 89.0
Y_BEARD_F = 88.0
Y_TORSO_B, Y_TORSO_F = 38.0, 78.0
Y_SKIRT_B, Y_SKIRT_F = 37.0, 79.0
Y_PACK_B = 12.0
Y_SHIN_B, Y_SHIN_F = 47.0, 70.0
Y_TOE = 76.0


def build_torso():
    """The tunic. Chest 44 px wide against a 61 px skirt: the sheet's tunic is a
    skirted one that flares below the belt, and the shoulder width it is judged on
    comes from the sleeve caps in build_arm(), not from the chest."""
    t = shade(TUNIC, TUNIC_LT, TUNIC_DK)
    boxes = [
        # collar, a band standing proud of the chest around the neck
        B(R_COLLAR - 3, R_COLLAR, -15, 15, 44, 72, shade(TUNIC_LT, TUNIC_LT, TUNIC)),
        # chest
        B(R_COLLAR - 1, 84, -CHEST_X, CHEST_X, Y_TORSO_B, Y_TORSO_F, t),
        # waist, where the belt sits
        B(84, R_SKIRT, -WAIST_X, WAIST_X, Y_TORSO_B, Y_TORSO_F, t),
        # skirt, flared
        B(R_SKIRT, 114, -SKIRT_X + 1, SKIRT_X - 1, Y_SKIRT_B + 1, Y_SKIRT_F - 1, t),
        # hem lip
        B(114, R_HEM, -SKIRT_X, SKIRT_X, Y_SKIRT_B, Y_SKIRT_F,
          shade(TUNIC_DK, TUNIC, TUNIC_DK)),
    ]
    ob = make_part("Torso_%s" % REV, boxes)
    ob.data.name = ob.name
    return ob


def build_neck():
    """The sheet hides the neck behind hair and beard in all four views, so this
    is the one piece of the figure that is INFERRED rather than measured -- see the
    report. The head is raised 3 px (2 % of figure height) off the sheet's reading
    so that a band of it shows between the hair and the collar."""
    boxes = [
        B(R_JAW - 4, R_COLLAR + 1, -NECK_X, NECK_X, 48, 68,
          shade(SKIN, SKIN, SKIN_DK, SKIN)),
        # trapezius wedge, so the neck does not read as a peg in a hole
        B(R_COLLAR - 3, R_COLLAR + 1, -14, 14, 46, 70,
          shade(SKIN_DK, SKIN, SKIN_DK)),
    ]
    ob = make_part("Neck_%s" % REV, boxes)
    ob.data.name = ob.name
    return ob


def build_head():
    """Skull, stepped dome, face and ears -- complete under the hair and beard.

    The crown steps are the round's headline correction. Both side views step the
    crown in at the front AND the back over three source-pixel bands, and front.png
    steps it in at the sides over the same bands: 23 px wide at the top, then 32,
    then 40, then the full 47. Round 4 read that as one box.
    """
    sk = shade(SKIN_DK, SKIN, SKIN_DK)
    boxes = [
        # --- the stepped dome, measured off all three views
        B(R_CROWN, R_DOME_B, -10, 10, 47, 75, sk),
        B(R_DOME_B, R_DOME_C, -14, 14, 44, 77, sk),
        B(R_DOME_C, R_SKULL, -18, 18, 41, 79, sk),
        # --- the skull, split at y=70 into a back half the hair covers and a front
        # half it does not. That split is what puts a FACE in the side view: the
        # sheet's hair stops short of the cheek, leaving 11 px of lit skin in
        # profile, and round 4's hair shell ran the full depth so the profile was
        # a slab of hair with an ear on it.
        B(R_SKULL, R_JAW, -SKULL_X, SKULL_X, Y_SKULL_B, 70,
          {"xlo": SKIN_DK, "xhi": SKIN_DK, "ylo": SKIN_DK,
           "yhi": None, "zlo": SKIN_DK, "zhi": SKIN}),
        B(R_SKULL, R_JAW, -SKULL_X, SKULL_X, 70, Y_SKULL_F,
          {"xlo": SKIN, "xhi": SKIN, "ylo": None,
           "yhi": None, "zlo": SKIN_DK, "zhi": SKIN}),
        # --- nose, the sheet's most forward point at 6 px past the brow
        B(R_EYE, R_NOSE_B, -4, 4, Y_SKULL_F, Y_NOSE,
          {"xlo": SKIN_DK, "xhi": SKIN_DK, "ylo": None, "yhi": SKIN,
           "zlo": SKIN_DK, "zhi": SKIN}),
    ]
    # brow ridges, heavy and angled inwards like the sheet's
    for s in (1, -1):
        boxes.append(B(R_BROW, R_BROW + 5, s * 3, s * 17, Y_SKULL_F, Y_SKULL_F + 2, HAIR))
    # ears, reaching past the hair shell as they do on the sheet
    for s in (1, -1):
        boxes.append(B(R_EAR_T, R_EAR_B, s * 21, s * EAR_X, 55, 63,
                       {"xlo": SKIN, "xhi": SKIN, "ylo": SKIN_DK, "yhi": SKIN,
                        "zlo": SKIN_DK, "zhi": SKIN_DK}))

    # the face, tiled onto the skull's front plane
    def sides(inner):
        return [(-SKULL_X, -18.0, SKIN_DK)] + inner + [(18.0, SKULL_X, SKIN_DK)]

    rows = [
        (R_SKULL, R_BROW, sides([(-18, 18, SKIN)])),                      # forehead
        (R_BROW, R_BROW + 5, sides([(-18, -3, HAIR), (-3, 3, SKIN_DK), (3, 18, HAIR)])),
        (R_BROW + 5, R_EYE, sides([(-18, 18, SKIN_DK)])),
        (R_EYE, R_EYE + 3, sides([
            (-18, -14, SKIN_DK), (-14, -12, SNOW), (-12, -10, HAIR),
            (-10, -4, SKIN_DK), (-4, 4, SKIN_DK), (4, 10, SKIN_DK),
            (10, 12, HAIR), (12, 14, SNOW), (14, 18, SKIN_DK)])),
        (R_EYE + 3, R_CHEEK, sides([(-18, 18, SKIN_DK)])),
        (R_CHEEK, R_NOSE_B, sides([(-18, -8, SKIN), (-8, 8, SKIN_DK), (8, 18, SKIN)])),
        (R_NOSE_B, R_JAW, sides([(-18, 18, SKIN_DK)])),
    ]
    ob = make_part("Head_%s" % REV, boxes, panels=[panel_y(Y_SKULL_F, rows)])
    ob.data.name = ob.name
    return ob


def build_beard():
    """Moustache, jaw mass and vertical locks with a ragged bottom.

    Measured: the mass is 44 px across against a 47 px head, so it is NARROWER
    than the head, and its tip is at row 82 -- z 0.464 H, level with the top of
    the belt. Round 4 put the tip at 0.428 H off the video.
    """
    dark = HAIR
    boxes = [
        # moustache, sitting on the nose's underside
        B(R_NOSE_B - 4, R_NOSE_B + 1, -14, 14, Y_SKULL_F, Y_NOSE - 2,
          {"xlo": dark, "xhi": dark, "ylo": None, "yhi": BEARD_LT,
           "zlo": dark, "zhi": BEARD_LT}),
        # cheek-to-jaw mass
        B(R_NOSE_B, 58, -BEARD_X, BEARD_X, 72, Y_BEARD_F,
          {"xlo": BEARD, "xhi": BEARD, "ylo": None, "yhi": BEARD,
           "zlo": None, "zhi": dark}),
    ]
    # cheek wings: the sheet carries the beard UP the sides of the face to just
    # below the eye line, leaving only a mask of skin round the eyes and nose.
    # Round 4 left the whole cheek bare, which is where its skin share came from.
    for s in (1, -1):
        boxes.append(B(R_CHEEK + 2, R_NOSE_B, s * 13, s * BEARD_X,
                       Y_SKULL_F - 1, Y_BEARD_F - 1,
                       {"xlo": BEARD, "xhi": BEARD, "ylo": None, "yhi": BEARD,
                        "zlo": dark, "zhi": dark}))
    # locks: staggered bottoms so the tip is ragged, not a shelf. Lengths are
    # deliberately NOT mirrored -- a mirror-symmetric beard reads as a machine part.
    LOCKS = [
        (-22.0, -17.0, 84.0, 70, BEARD),
        (-17.0, -11.0, 85.0, 76, BEARD_LT),
        (-11.0, -4.0, 86.0, 80, BEARD),
        (-4.0, 4.0, 86.0, R_BEARD_TIP, BEARD_LT),
        (4.0, 11.0, 86.0, 79, BEARD),
        (11.0, 17.0, 85.0, 74, BEARD_LT),
        (17.0, 22.0, 84.0, 68, BEARD),
    ]
    for x0, x1, yf, rb, front in LOCKS:
        boxes.append(B(58, rb, x0, x1, 72, yf,
                       {"xlo": dark, "xhi": dark, "ylo": None, "yhi": front,
                        "zlo": dark, "zhi": None}))
    ob = make_part("Beard_%s" % REV, boxes)
    ob.data.name = ob.name
    return ob


def build_hair():
    """A shell over the skull: the stepped dome, a back slab, two side locks and a
    fringe, leaving the face open. Two values, neither costing a palette slot --
    HAIR #34271C is the mass and BEARD #5E4632 the plane that catches light."""
    m, lt = HAIR, BEARD
    h = shade(m, lt, m)
    boxes = [
        B(R_CROWN, R_DOME_B, -11.5, 11.5, 45, 77, h),
        B(R_DOME_B, R_DOME_C, -16, 16, 42, 79, h),
        B(R_DOME_C, R_SKULL, -20, 20, 39, 81, h),
        B(R_SKULL, R_SKULL + 4, -HX, HX, Y_HEAD_B, Y_HEAD_F, h),     # top cap
        # back slab, carried down to the shoulder line as the sheet's back view has it
        B(R_SKULL + 4, R_COLLAR - 3, -HX, HX, Y_HEAD_B, Y_SKULL_B + 6, h),
    ]
    # Side locks stop ABOVE the collar, tucked behind the ears, so a band of neck
    # shows between them and the tunic. The sheet's hair reaches the shoulders on
    # every view and hides the neck completely; this is the deliberate departure
    # that makes the neck readable. Deliberately unequal, so the silhouette is not
    # mirror-symmetric.
    boxes.append(B(R_SKULL + 4, R_JAW - 2, -HX, -19.5, Y_HEAD_B, 70, h))
    boxes.append(B(R_SKULL + 4, R_JAW - 4, 19.5, HX, Y_HEAD_B, 70, h))
    # fringe over the brow, sitting proud of the face plane
    boxes.append(B(R_SKULL + 4, R_BROW, -19.5, 19.5, 70, Y_HEAD_F, h))
    ob = make_part("Hair_%s" % REV, boxes)
    ob.data.name = ob.name
    return ob


def build_arm(side):
    """Sleeve cap, sleeve, bare forearm and hand.

    The shoulder cap is the round's second correction: back.png puts the outer
    edge of the sleeve at 0.264 H from the centre against a 0.168 H head, so there
    is 0.096 H of shoulder outboard of the skull -- room for the over-shoulder
    strap the sheet has and round 4 had nowhere to route.
    """
    s = 1 if side == "R" else -1
    t = shade(TUNIC, TUNIC_LT, TUNIC_DK)
    sk = shade(SKIN, SKIN, SKIN_DK)
    boxes = [
        # shoulder cap: sits ON the shoulder and is the widest point of the figure
        B(R_SHOULDER, 58, s * 20, s * SHOULDER_X, 40, 74, t),
        # sleeve
        B(58, R_CUFF, s * 22, s * (SHOULDER_X - 1), 42, 74, t),
        # cuff
        B(R_CUFF, R_CUFF + 3, s * 21.5, s * (SHOULDER_X - 0.5), 41.5, 74.5,
          shade(TUNIC_DK, TUNIC, TUNIC_DK)),
        # bare forearm, THICKER than the sleeve -- dwarf proportion, and where
        # round 4 found most of its visible-skin share
        B(R_CUFF + 3, 80, s * 23, s * (SHOULDER_X - 1), 46, 70, sk),
        # wrist, pinched, so the arm is not one featureless slab in profile
        B(80, R_HAND_T, s * 24.5, s * (SHOULDER_X - 3), 48, 68,
          shade(SKIN_DK, SKIN, SKIN_DK)),
        # hand, wider again: a closed fist
        B(R_HAND_T, R_HAND_B, s * 22, s * SHOULDER_X, 46, 71, sk),
    ]
    ob = make_part("Arm.%s_%s" % (side, REV), boxes)
    ob.data.name = ob.name
    return ob


def build_leg(side):
    s = 1 if side == "R" else -1
    p = shade(PANTS, PANTS_LT, HAIR)
    boxes = [B(100, R_CUFF_BOOT_T + 2, s * 5.5, s * 27.5, Y_SHIN_B, Y_SHIN_F, p)]
    ob = make_part("Leg.%s_%s" % (side, REV), boxes)
    ob.data.name = ob.name
    return ob


def build_boot(side):
    """Long forward: the sheet's profile puts the toe 8 px past the front of the
    shin, so the foot is 30 px (0.214 H) deep against a 23 px wide leg."""
    s = 1 if side == "R" else -1
    lea = shade(HAIR, BEARD, HAIR)
    boxes = [
        # cuff, the widest part of the boot
        B(R_CUFF_BOOT_T, R_BOOT_T, s * 3, s * 30, Y_SHIN_B - 1, Y_SHIN_F + 1,
          shade(BEARD, BEARD_LT, HAIR)),
        # shaft
        B(R_BOOT_T, R_TOE, s * 5.5, s * 27.5, Y_SHIN_B, Y_SHIN_F, lea),
        # foot, projecting forward
        B(R_TOE, ROW_SOLE, s * 5.5, s * 27.5, Y_SHIN_B, Y_TOE, lea),
        # sole
        B(ROW_SOLE - 2, ROW_SOLE, s * 5, s * 28, Y_SHIN_B - 0.5, Y_TOE + 0.5,
          shade(TRUNK, TRUNK, TRUNK)),
    ]
    ob = make_part("Boot.%s_%s" % (side, REV), boxes)
    ob.data.name = ob.name
    return ob


def build_belt():
    lea = shade(TRUNK, WOOD, HAIR)
    boxes = [
        B(R_BELT_T, R_BELT_B, -WAIST_X - 1.5, WAIST_X + 1.5,
          Y_TORSO_B - 1.5, Y_TORSO_F + 1.5, lea),
        # buckle
        B(R_BELT_T + 2, R_BELT_B - 1, -8, 8, Y_TORSO_F + 1.5, Y_TORSO_F + 3,
          shade(METAL, METAL, HAIR)),
        B(R_BELT_T + 4, R_BELT_B - 3, -4.5, 4.5, Y_TORSO_F + 1.5, Y_TORSO_F + 3.4,
          shade(HAIR, HAIR, HAIR)),
    ]
    ob = make_part("Belt_%s" % REV, boxes)
    ob.data.name = ob.name
    return ob


def build_pack():
    """Pack, flap, bedroll and the two over-shoulder straps.

    back.png is authoritative here and was previously INVENTED outright: the pack
    is 0.374 H across with a big flap and a single centre buckle, and the straps
    run vertically over the trapezius, not diagonally.
    """
    lea = shade(TRUNK, WOOD, HAIR)
    flap = shade(WOOD, WOOD, TRUNK)
    grey = shade(PANTS_LT, METAL, PANTS)
    PX = 26.0
    boxes = [
        B(57, 96, -PX, PX, Y_PACK_B + 2, Y_TORSO_B, lea),
        B(57, 80, -PX + 3, PX - 3, Y_PACK_B, Y_PACK_B + 2, flap),
        B(76, 82, -5, 5, Y_PACK_B - 1, Y_PACK_B + 2, shade(METAL, METAL, HAIR)),
        # bedroll across the top of the pack, stepped so it reads as a coil.
        # side-left.png puts it at rows 32..51 -- level with the lower head, which
        # is why round 4 never saw it: the video frame has the head in the way.
        B(45, 57, -21, 21, Y_PACK_B + 2, Y_TORSO_B - 2, grey),
        B(42, 45, -18, 18, Y_PACK_B + 5, Y_TORSO_B - 5, grey),
        B(57, 60, -18, 18, Y_PACK_B + 5, Y_TORSO_B - 5, grey),
    ]
    # Straps: over the shoulder and down the chest, plus the two bands the back
    # view carries on the pack's own outer face. They sit at x +/-13..19 rather
    # than the sheet's +/-2..11 so that they clear the beard and actually read --
    # which is only possible at all because the shoulder caps gave them somewhere
    # to go. Round 4 had 7 mm of shoulder and terminated its straps at the seam.
    for s in (1, -1):
        boxes.append(B(R_SHOULDER, 62, s * 13, s * 19, Y_TORSO_B - 2, Y_TORSO_B, lea))
        boxes.append(B(R_SHOULDER - 2, R_SHOULDER, s * 13, s * 19,
                       Y_TORSO_B - 2, Y_TORSO_F + 1, shade(WOOD, WOOD, TRUNK)))
        boxes.append(B(R_SHOULDER, R_BELT_T, s * 13, s * 19,
                       Y_TORSO_F, Y_TORSO_F + 1.5, lea))
        # on the pack itself, over the flap
        boxes.append(B(57, 80, s * 6, s * 12, Y_PACK_B - 1, Y_PACK_B, lea))
    ob = make_part("Pack_%s" % REV, boxes)
    ob.data.name = ob.name
    return ob


def build_lantern():
    """0.33x dwarf height, from the sheet's own label on the gear breakdown --
    which is 26 % larger than the lantern drawn in his hand. See the report."""
    XC, YC = -44.0, 58.0     # hangs outboard of the left hand, clear of the skirt
    W, D = 11.0, 11.0
    dark = shade(HAIR, TRUNK, HAIR)
    boxes = [
        # hoop, passing through the hand
        B(94, 99, XC - 2, XC + 2, YC - 2, YC + 2, shade(METAL, METAL, HAIR)),
        B(99, 103, XC - W + 2, XC + W - 2, YC - D + 2, YC + D - 2, dark),
        # cap
        B(103, 107, XC - W, XC + W, YC - D, YC + D, dark),
        # body: glass on all four sides
        B(107, 132, XC - W + 1, XC + W - 1, YC - D + 1, YC + D - 1,
          {"xlo": FLAME, "xhi": FLAME, "ylo": FLAME, "yhi": FLAME,
           "zlo": HAIR, "zhi": HAIR}),
        # the hot core, a snow cell inside the glass on the two broad faces
        B(113, 127, XC - 4, XC + 4, YC - D + 0.5, YC + D - 0.5,
          {"xlo": None, "xhi": None, "ylo": SNOW, "yhi": SNOW,
           "zlo": None, "zhi": None}),
        # corner posts
        B(107, 132, XC - W + 1, XC - W + 2.5, YC - D + 1, YC + D - 1, dark),
        B(107, 132, XC + W - 2.5, XC + W - 1, YC - D + 1, YC + D - 1, dark),
        # base
        B(132, 137, XC - W, XC + W, YC - D, YC + D, dark),
    ]
    ob = make_part("Lantern_%s" % REV, boxes)
    ob.data.name = ob.name
    return ob


def build_pickaxe():
    """0.83x dwarf height, from the sheet's label: 116 px, so it cannot HANG from
    a hand without going through the floor. It stands butt-on-ground beside his
    right hand, handle vertical and head at the top -- which is precisely the
    attitude the gear breakdown's own front view draws it in."""
    XC = 44.0
    YB, YF = 84.0, 91.0        # planted just in front of his toes, clear of the beard
    R_TOP, R_BUTT = 31.0, ROW_SOLE
    wood = shade(WOOD, WOOD, TRUNK)
    steel = shade(METAL, METAL, HAIR)
    boxes = [
        B(R_TOP, R_BUTT, XC - 3.5, XC + 3.5, YB + 1.5, YF - 1.5, wood),   # handle
        B(R_BUTT - 7, R_BUTT, XC - 4.5, XC + 4.5, YB + 0.5, YF - 0.5,
          shade(TRUNK, TRUNK, HAIR)),                                     # ferrule
        B(R_TOP + 3, R_TOP + 15, XC - 5, XC + 5, YB, YF, steel),          # eye
    ]
    # The head: a staircase of boxes each side, because the sheet's arc is a smooth
    # curve and a box model cannot have one. It is BROADSIDE, as the sheet draws it,
    # which is only possible because the tool is planted forward of the figure --
    # at 0.83 H (116 px) it has to stand butt-on-ground, and a broadside head on a
    # shaft held at his side passes straight through his ear and his shoulder cap.
    # The forward plant is the price of keeping the sheet's own attitude.
    for s in (1, -1):
        for dx0, dx1, dr in ((5, 11, 5), (11, 16, 8), (16, 20, 11)):
            boxes.append(B(R_TOP + dr, R_TOP + dr + 10,
                           XC + s * dx0, XC + s * dx1, YB, YF, steel))
    ob = make_part("Pickaxe_%s" % REV, boxes)
    ob.data.name = ob.name
    return ob


# =============================================================================
# Steps
# =============================================================================
STEPS = [
    ("torso", lambda: [build_torso()]),
    ("neck", lambda: [build_neck()]),
    ("head", lambda: [build_head()]),
    ("beard", lambda: [build_beard()]),
    ("hair", lambda: [build_hair()]),
    ("arms", lambda: [build_arm("L"), build_arm("R")]),
    ("legs", lambda: [build_leg("L"), build_leg("R")]),
    ("boots", lambda: [build_boot("L"), build_boot("R")]),
    ("belt", lambda: [build_belt()]),
    ("pack", lambda: [build_pack()]),
    ("lantern", lambda: [build_lantern()]),
    ("pickaxe", lambda: [build_pickaxe()]),
]


def wire_sockets():
    """Declared mount points, so a variant is a part list and nothing else."""
    pairs = [
        ("socket.hair", (0.0, Y(COL_Y0), Z(R_SKULL)), "Hair_%s" % REV),
        ("socket.beard", (0.0, Y(Y_SKULL_F), Z(R_NOSE_B)), "Beard_%s" % REV),
        ("socket.pack", (0.0, Y(Y_TORSO_B), Z(70)), "Pack_%s" % REV),
        ("socket.hand.L", (P(-29.5), Y(58), Z(R_HAND_B)), "Lantern_%s" % REV),
        ("socket.hand.R", (P(29.5), Y(58), Z(R_HAND_B)), "Pickaxe_%s" % REV),
    ]
    for name, loc, child in pairs:
        ob = bpy.data.objects.get(child)
        if ob is not None:
            socket(name, loc, ob)


def seed_from_r4(r4_path):
    """Inherit round 4's palette image, material and render settings verbatim;
    rename the revision token and drop every scrap of r4 geometry."""
    bpy.ops.wm.open_mainfile(filepath=r4_path)

    for ob in list(bpy.data.objects):
        bpy.data.objects.remove(ob, do_unlink=True)
    for me in list(bpy.data.meshes):
        bpy.data.meshes.remove(me)
    for txt in list(bpy.data.texts):
        bpy.data.texts.remove(txt)
    for coll in list(bpy.data.collections):
        bpy.data.collections.remove(coll)

    img = bpy.data.images.get("T_VoxelDwarf_Palette_r4")
    if img is None:
        raise SystemExit("seed: round 4's palette image is not in %s" % r4_path)
    img.name = PALETTE
    img.use_fake_user = True          # nothing references it until step 1 saves

    mat = bpy.data.materials.get("M_VoxelDwarf_r4")
    if mat is None:
        raise SystemExit("seed: round 4's material is not in %s" % r4_path)
    mat.name = MATERIAL
    mat.use_fake_user = True
    if not mat.use_backface_culling:
        raise SystemExit("seed: inherited material lost its backface culling")

    coll = bpy.data.collections.new(COLLECTION)
    bpy.context.scene.collection.children.link(coll)
    return coll


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if argv and argv[0] == "seed":
        seed_from_r4(argv[1])
        bpy.ops.wm.save_as_mainfile(filepath=BLEND)
        print("SEEDED %s  material=%s palette=%s" % (BLEND, MATERIAL, PALETTE))
        return

    step = int(argv[0])
    name, fn = STEPS[step - 1]
    built = fn()
    wire_sockets()
    print("STEP %02d %-8s ->  %s" % (step, name, ", ".join(o.name for o in built)))
    for ob in built:
        tris = len(ob.data.polygons) * 2
        bad = sum(1 for p in ob.data.polygons
                  if not (sorted(abs(c) for c in p.normal)[2] > 0.999999
                          and sorted(abs(c) for c in p.normal)[1] < 1e-6))
        smooth = sum(1 for p in ob.data.polygons if p.use_smooth)
        print("         %-16s tris %4d  non-axis-aligned %d  smooth %d"
              % (ob.name, tris, bad, smooth))
    bpy.ops.wm.save_mainfile(filepath=BLEND)
    print("SAVED %s" % BLEND)


if __name__ == "__main__":
    main()
