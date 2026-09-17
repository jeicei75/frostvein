"""Round 14 -- the voxel dwarf miner, generated from the model sheet as separate parts.

RUN IT IN THE LIVE BLENDER, one tool call per run:

    exec(open(PATH_TO_THIS_FILE).read()); build()

Iteration is: change a number in PARAMS, run, render the overlay, look. Nothing in this file
is measured by hand -- every dimension is derived in `derive()` from the orthographic table in
`references/dwarf-model-sheet.md` (round 14 brief, sec.3), which is settled and not re-measured.

THE TWO FRAMES THE SHEET IS READ IN
  * heights  z = (147 - row) * PX      front.png / side-left.png, row 7 crown .. row 147 sole
  * widths   x = (col - 77) * PX       front.png, column 77 is the centre line
  * depths   y = (col - 58) * PX       side-left.png, column 58 is the depth centre, +Y forward
  PX = 1.200 / 140 = 8.571 mm, one source pixel.

WHY DEPTHS ARE TAKEN FROM COLUMNS AND NOT FROM THE /H FRACTIONS. The sheet gives both, and the
fractions alone are ambiguous about WHERE each span sits. The columns pin them absolutely and
they agree: head 36..83 = 47 px = 0.336 H, torso 38..78 = 0.286 H, pack 12..38 = 0.186 H,
figure 12..89 = 0.550 H. They also settle an apparent contradiction: the brief's "hair's front
edge in profile is y = +0.111" is column 71, while the head's depth runs to column 83. Those are
two different edges -- 83 is the FRONT OF THE FACE, 71 is where the hair's fringe stops and skin
begins. Both are honoured below.

THE ONE PLACE THE TABLE FORCED A SHAPE. The sheet says the hair mass ends at 0.707 H and also
that bare neck shows from 0.793 down to 0.679 H in profile. In an orthographic side view the
surface with the largest |x| wins, so a full-depth head reaching down to 0.707 would bury the
neck and leave 0.028 H, not the sheet's 0.121 H. They reconcile if the SKULL is deep and the JAW
is shallow: below 0.793 H the head keeps only its front (the face and chin), the hair lobes hang
BEHIND the neck rather than beside it, and the neck column is the outermost surface in between.
That is what the head/hair/neck boxes below do, and it is the only inference in the file.
Everything else is a number off the sheet.
"""

import bpy
import math

REV = "r17"
COLL = "SM_VoxelDwarf_Miner01_" + REV
H = 1.200                 # figure height, metres
PX = H / 140.0            # one source pixel, 8.571 mm
SRC = 5                   # the ortho crops are 5x nearest-neighbour blow-ups
UNIT = H / 64.0           # the sheet's feature unit, ~18.75 mm
OVERLAP = 0.004           # parts within one object overlap, never meet exactly (brief sec.4)


def z_of(row):
    return (147.0 - row) * PX          # front / side rows -> world z


def x_of(col):
    return (col - 77.0) * PX           # front columns     -> world x


def y_of(col):
    return (col - 58.0) * PX           # side-left columns -> world y


def h(frac):
    return frac * H                    # a z/H or width/H fraction -> metres


def row_top(r):
    # z of the TOP edge of source row r, pulled down by half a RENDER pixel.
    #
    # A band whose top sits exactly on the boundary still covers the pixel row above it --
    # rasterisation lights a pixel that geometry merely touches -- so every crown step
    # rendered one image row early and the overlay read 3.8 source px of overshoot on that
    # single row, while every settled row around it was 0.0. This is a rasterisation offset,
    # not a width error, and half a render pixel is the whole of it.
    return (147.5 - r) * PX - (PX / SRC / 2.0)


def row_bot(r):
    return (146.5 - r) * PX            # z of the BOTTOM edge of source row r


def derive():
    """Every dimension round 14 uses, named, in metres, straight off the sheet."""
    p = {}

    # ---- heights, z/H from the sole (sheet sec.3 height table) -----------------
    p["crown"] = h(1.000)
    p["crown_s2"] = h(0.979)
    p["crown_s1"] = h(0.964)
    p["skull_top"] = h(0.943)
    p["brow"] = h(0.879)
    # 0.850 is the table's ear top; front.png's own left edge only steps out to the ear at
    # row 29, which is 0.843. One row, but it put the ears 3.4 px outside the sheet at 0.850.
    p["ear_top"] = h(0.843)
    p["eye_line"] = h(0.807)
    p["neck_top"] = h(0.793)
    p["nose_low"] = h(0.750)
    p["ear_bot"] = h(0.729)
    p["head_ends"] = h(0.707)
    p["shoulder"] = h(0.700)
    p["neck_bot"] = h(0.679)
    p["cuff"] = h(0.566)
    p["beard_tip"] = h(0.464)
    p["belt_top"] = h(0.421)
    p["belt_bot"] = h(0.343)
    p["pack_bot"] = h(0.350)
    p["hand_bot"] = h(0.330)
    p["hem"] = h(0.207)
    # likewise 0.164 is the table's cuff top; the art steps out to the cuff at row 125, 0.157
    p["boot_cuff_t"] = h(0.157)
    p["boot_cuff_b"] = h(0.107)
    p["sole"] = 0.0

    # ---- widths, /H, as half-widths about the centre line ---------------------
    p["hw_head"] = h(0.336) / 2        # head with hair
    p["hw_ear"] = h(0.383) / 2         # ear to ear
    p["hw_crown"] = [h(w) / 2 for w in (0.336, 0.286, 0.229, 0.164)]
    p["hw_beard"] = h(0.343) / 2       # essentially the head's own width
    p["hw_chest"] = h(0.317) / 2       # tunic only, between the sleeve seams
    p["hw_shoulder"] = h(0.528) / 2    # carried by the sleeve caps, not the torso
    p["hw_waist"] = h(0.439) / 2       # skirt at the hem
    p["hw_stance"] = h(0.398) / 2      # boot outer to boot outer
    p["w_limb"] = h(0.164)             # one leg / one boot
    p["w_bootcuff"] = h(0.200)
    p["hw_neck"] = h(0.064) / 2

    # ---- depths, from side-left columns, +Y forward ---------------------------
    p["y_head_back"] = y_of(36)        # back of the hair          -0.1886
    p["y_hair_front"] = y_of(71)       # fringe stops, skin begins +0.1114
    p["y_face"] = y_of(83)             # front of the face         +0.2143
    # side-left.png's own front edge across the head, which the table does not carry:
    # +0.174 H at rows 15-17, dipping to +0.167 at 18-22, back to +0.174 at 23 and out to
    # +0.181 at the brow, row 24. Column 83 is 0.6-1.6 px proud of all of it, and a brow
    # ridge built 15 mm proud of column 83 stood 3.4 px outside the sheet.
    p["y_face_art"] = h(0.170)         # skull front, rows 15-23
    p["y_brow_art"] = h(0.181)         # brow and jaw front, rows 24-48
    p["y_fringe_art"] = h(0.174)       # hair fringe front, rows 7-15
    p["y_beard_f"] = y_of(86)          # beard front               +0.2400
    p["y_nose"] = y_of(89)             # nose tip                  +0.2657
    p["y_torso_b"] = y_of(38)          # torso back                -0.1714
    p["y_torso_f"] = y_of(78)          # torso front               +0.1714
    p["y_skirt_b"] = y_of(37)
    p["y_skirt_f"] = y_of(79)
    p["y_pack_b"] = y_of(12)           # pack back                 -0.3943
    p["y_pack_f"] = y_of(38)
    p["y_shin_b"] = y_of(47)
    p["y_shin_f"] = y_of(70)
    p["y_sole_f"] = y_of(76)           # toe, 0.057 H past the shin front

    # ---- pose: A-pose, arms out 40 degrees from straight down -----------------
    p["arm_deg"] = 40.0
    # the sheet is POSED with the arms hanging, so shoulder-to-hand is a LENGTH,
    # not a height: 0.700 H shoulder to 0.330 H hand bottom = 0.370 H of arm.
    p["arm_len"] = h(0.700 - 0.330)
    p["arm_cuff_t"] = h(0.700 - 0.566)   # along-arm distance to the sleeve cuff
    p["hand_len"] = h(0.078)
    p["arm_x"] = h(0.528) / 2 - h(0.055)  # arm axis, inboard of the cap edge
    p["cap_top"] = h(0.700)               # sleeve cap tops out ON the shoulder line

    # ---- gear (sheet sec.3 feature scale; pickaxe off front.png) --------------
    p["buckle_w"] = 5 * UNIT
    p["pick_shaft"] = h(0.910)
    p["pick_blade"] = h(0.380)
    p["lantern_h"] = h(0.260)
    return p


P = derive()

PALETTE = {                       # the approved ten; stage A paints none of them
    "skin": "#E9D2BB", "beard": "#5E4632", "snow": "#FFFFFF", "tunic": "#5F7A6A",
    "pants": "#474B41", "metal": "#A9B2AC", "wood": "#8B6B50", "trunk": "#6B5B49",
    "hair": "#34271C", "flame": "#F0A63C",
}


# --------------------------------------------------------------------- geometry
def box(x0, x1, y0, y1, z0, z1):
    """Eight verts and six outward-wound quads. Axis-aligned."""
    v = [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0),
         (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]
    f = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (3, 7, 6, 2), (0, 4, 7, 3), (1, 2, 6, 5)]
    return v, f


def prism8(x0, x1, y0, y1, z0, z1, cx, cy):
    """One mass with its four vertical corners cut at 45 degrees -- a REAL wedge.

    This is the round-16 finding applied to round 14's own geometry, and it is a
    two-line change because r14 already decided WHERE the chamfers belong: it calls
    chamfered() on the torso, skirt, sleeve caps and boots exactly as r15 s4.1 asks.
    What it could not do was buy a NORMAL with them -- its chamfer is two
    axis-aligned boxes, one full-width and one full-depth, whose union steps the
    corner without ever producing a surface that faces the corner. That is why r14
    measures 14 distinct directions across the whole figure and why 94.6 % of its
    faces sit on the six axes: an axis-aligned box has six possible normals however
    many you stack.

    The octagonal ring below produces the same union -- the face at x=x1 still exists
    between y0+cy and y1-cy, so the extreme x is still reached, and likewise for y --
    so BOTH ORTHOGRAPHIC SILHOUETTES ARE UNCHANGED and every one of r14's tuned
    dimensions still lands exactly where it landed. It simply adds the four diagonal
    planes that were missing.
    """
    ring = [(x0 + cx, y0), (x1 - cx, y0), (x1, y0 + cy), (x1, y1 - cy),
            (x1 - cx, y1), (x0 + cx, y1), (x0, y1 - cy), (x0, y0 + cy)]
    n = len(ring)
    v = [(px, py, z0) for (px, py) in ring] + [(px, py, z1) for (px, py) in ring]
    f = []
    for i in range(n):
        j = (i + 1) % n
        f.append((i, j, j + n, i + n))
    cl = len(v)
    v.append((sum(p[0] for p in ring) / n, sum(p[1] for p in ring) / n, z0))
    cu = len(v)
    v.append((sum(p[0] for p in ring) / n, sum(p[1] for p in ring) / n, z1))
    for i in range(n):
        j = (i + 1) % n
        f.append((cl, j, i))
        f.append((cu, i + n, j + n))
    return v, f


SMALL_WEDGE = 0.045      # ~45 mm. Below this a chamfer is a sliver, which is what
                         # round 12 was criticised for; such boxes stay square.


def wedge(x0, x1, y0, y1, z0, z1, cx=0.016, cy=0.016):
    """A drop-in for box() that cuts the four vertical corners.

    Returns ONE entry, exactly as box() does, so it can be swapped in anywhere
    without shifting a single index in PAINT -- which matters because that table
    keys per-box overrides by number (the head's nose steps are boxes 3-6, the
    beard's bands 1-4, the pack's bedroll 2-4) and r15 s7.2 is explicit that every
    restructure shifts them silently while the checks keep printing numbers.

    Small masses fall back to a plain box on their own, so this can be applied
    across a whole part without hand-auditing every call: interior detail keeps its
    square corners and only the masses big enough to carry a third plane get one.
    """
    if min(x1 - x0, y1 - y0) < SMALL_WEDGE:
        return box(x0, x1, y0, y1, z0, z1)
    return prism8(x0, x1, y0, y1, z0, z1,
                  min(cx, 0.30 * (x1 - x0)), min(cy, 0.30 * (y1 - y0)))


def chamfered(x0, x1, y0, y1, z0, z1, cx=0.018, cy=0.018):
    """One mass as TWO chamfered prisms, stacked.

    It returns TWO entries and not one on purpose. r14's PAINT table keys per-box
    overrides BY INDEX -- the boot's sole is box 0 and its heel box 7, and its own
    comment records that "the chamfer split the body and shaft into 1-4". Returning
    a single prism here would shift every index after each call site and silently
    repaint the wrong boxes, which is r15 s7.2 exactly ("every restructure shifts
    them, silently, and the check then measures a different mass while still
    printing a number"). Splitting the mass in Z keeps the count at two, so every
    tuned index in PAINT still names the mass it was tuned against.
    """
    zm = 0.5 * (z0 + z1)
    return [prism8(x0, x1, y0, y1, z0, zm, cx, cy),
            prism8(x0, x1, y0, y1, zm, z1, cx, cy)]


def arm_box(t0, t1, hw, hd, pivot, deg, side=1, off=(0.0, 0.0)):
    """A box along an arm axis `deg` out from straight down, through `pivot`.

    side=+1 is the figure's right (+X). The cross-section is hw across the arm in the
    XZ plane and hd in Y, so the sleeve stays square to the body however far it swings.

    `off` shifts the box within that cross-section: (along e1, along Y). It exists for the
    thumb. Offsetting the thumb in Y alone puts it on the front of the palm, in the middle
    of the hand; a thumb sits on the INBOARD edge, which is +e1 (e1 points toward the body
    on both sides), with only a little forward lead.
    """
    a = math.radians(deg)
    u = (side * math.sin(a), 0.0, -math.cos(a))          # down the arm
    # e1 MUST be y_hat x u. box() winds its faces for a right-handed (e1, +Y, u) frame, and
    # the obvious (cos a, 0, sin a) gives e1 x y_hat = -u -- a left-handed frame, which winds
    # every arm and glove face inside-out. The geometry looked correct and the normals were
    # all reversed.
    e1 = (-math.cos(a), 0.0, -side * math.sin(a))        # across it, in XZ
    px, py, pz = pivot[0] * side, pivot[1], pivot[2]

    def at(t, s1, s2):
        d1 = s1 * hw + off[0]
        return (px + u[0] * t + e1[0] * d1,
                py + s2 * hd + off[1],
                pz + u[2] * t + e1[2] * d1)

    v = [at(t0, -1, -1), at(t0, 1, -1), at(t0, 1, 1), at(t0, -1, 1),
         at(t1, -1, -1), at(t1, 1, -1), at(t1, 1, 1), at(t1, -1, 1)]
    f = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (3, 7, 6, 2), (0, 4, 7, 3), (1, 2, 6, 5)]
    if side < 0:                                    # mirroring flips the winding back
        f = [tuple(reversed(q)) for q in f]
    return v, f


def mirror_x(parts):
    out = []
    for v, f in parts:
        out.append(([(-a, b, c) for a, b, c in v], [tuple(reversed(q)) for q in f]))
    return out


def mesh_from(name, parts):
    verts, faces = [], []
    for v, f in parts:
        n = len(verts)
        verts.extend(v)
        faces.extend([tuple(i + n for i in q) for q in f])
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    me.validate()
    me.update()
    for poly in me.polygons:
        poly.use_smooth = False
    return me


# --------------------------------------------------------------------- the parts
def part_head():
    # THE HEAD STAYS SQUARE. Wedging it was tried and reverted: the face island is
    # projected onto these boxes' +Y faces, and cutting their corners narrows that
    # face and turns the corners into 45-degree planes that take the projection at a
    # slant -- the brows, eyes and nose ridge all went soft. That is r16 s6's third
    # warning (the island must still cover every box proud of the face plane, wedges
    # included) and the same lesson r16 learned twice: free-standing masses gain a
    # third plane, INTERLOCKING detail loses its joins. Hair, beard and pack are
    # wedged; the face is not.
    p = P
    o = OVERLAP
    # The SKULL is 8 mm narrower each side than "head with hair", so the hair cap wraps the
    # sides rather than butting against them. Built flush, the skull's own side faces showed
    # as bare skin above the ears in every three-quarter view, where f084 and f104 show hair.
    # The 0.336 width is carried by the hair, which is what the sheet measures.
    hw = p["hw_head"] - 0.008
    # The skull's BACK stops 20 mm forward of column 36. The sheet's 0.336 depth is measured
    # from the back of the HAIR to the front of the face, so the skull must end inside the
    # hair, not at the same plane. Two bugs came from getting this margin wrong: at 0 mm it
    # was coplanar with the hair's back bands and z-fought; at 8 mm it still poked 1 mm out
    # behind the SHALLOWEST band (band 7 sits at -0.1796), which rendered as a thin bar of
    # skin across the back of the hair. 20 mm clears the shallowest band by 7 mm.
    y_skull_back = p["y_head_back"] + 0.020
    return [
        # deep skull, down to the top of the bare neck run
        # Skull, recessed 10 mm from the face plane and stopping below the top chamfer. The
        # face plate and the top band are appended boxes; together they step the head in at
        # the crown and at its front corners, which is what "rounded" means on a box figure
        # and what f100 and f092 both read.
        box(-hw, hw, y_skull_back, p["y_face_art"] - 0.010,
            p["neck_top"] - o, 1.0800),
        # shallow jaw: front of the head only, so the neck stays the outermost
        # surface behind it in profile (see the module docstring). Its back plane is what
        # sets the FRONT of the visible neck column.
        box(-hw + 0.012, hw - 0.012, -0.050, p["y_brow_art"],
            p["head_ends"], p["neck_top"] + o),
        # Brow band at 0.879 -- the art's step out to +0.181 H at row 24, spanning rows
        # 24-30. Built as a fixed 15 mm proud of column 83 it reached 0.191 H where the art
        # still has 0.167, the largest single violation left in the side view.
        # The brow BAND is recessed 10 mm; the two brow blocks appended below stand at the
        # art's 0.181 H. In f100 the brows are two masses with the nose bridge sunk between
        # them, not one bar across the face.
        box(-hw, hw, p["y_face_art"] - o, p["y_brow_art"] - 0.010,
            row_bot(30), row_top(24)),
        # NOSE, ramped from side-left.png's own rows. The art does not carry a block: its
        # front edge steps 0.189 / 0.196 / 0.210 / 0.217 H across rows 31-38 and is back to
        # 0.196 by row 43. A two-box nose reaching 0.207 H from row 24 upward stood 2.5
        # source px proud of the sheet at z/H 0.833, where the art still has only brow.
        box(-0.034, 0.034, p["y_brow_art"] - o, h(0.196), row_bot(32), row_top(31)),
        box(-0.032, 0.032, p["y_brow_art"] - o, h(0.210), row_bot(34), row_top(33) + o),
        box(-0.028, 0.028, p["y_brow_art"] - o, h(0.217), row_bot(38), row_top(35) + o),
        box(-0.024, 0.024, p["y_brow_art"] - o, h(0.210), row_bot(42), row_top(39) + o),
        # ears, tabs out to the sheet's 0.383 ear-to-ear, kept FORWARD of the neck
        # EARS. Two things were wrong. They sat at y 0.030..0.150 -- forward of the skull's
        # own centre (0.017) and so reading as too far front -- and at 120 mm deep they were
        # a slab, not a tab. An ear protrudes to 0.383 H while the hair can only reach the
        # sheet's 0.336 H head width, so by construction NOTHING can sit behind the part that
        # sticks out: its back face is exposed, and at 120 mm of bare skin in dark hair that
        # is the "hole" in the back of the hair. Centring them on the skull and cutting them
        # to a 70 mm tab fixes both, and clears the neck window (-0.128..-0.050) completely
        # so the ears can no longer eat into the bare-neck run.
        box(hw - o, p["hw_ear"], -0.020, 0.050, p["ear_bot"], p["ear_top"]),
        box(-p["hw_ear"], -hw + o, -0.020, 0.050, p["ear_bot"], p["ear_top"]),
        # NECK, a full column -- not a slab.
        #
        # The sheet's "up to 0.064 H wide" is measured on side-left.png, so it is the DEPTH of
        # the VISIBLE skin column, not the neck's thickness. Two mistakes came out of that.
        # First it was built 0.064 H THICK, giving a 77 mm neck under a 403 mm head, which
        # reads as a hole. Then it was widened but its Y span was left at the 0.064 H window,
        # so only the back of the neck existed and the front half was missing.
        #
        # The window is not the neck: it is what the jaw and the hair leave uncovered. So the
        # column runs the full depth of the throat and the OCCLUDERS set what shows -- the
        # hair lobes stop at -0.130 behind it, the jaw starts at -0.050 in front, and the
        # 0.078 m between them is the sheet's column. Everything forward of -0.050 is inside
        # the jaw and the beard, exactly as it should be.
        # It runs out to 0.130, which is the hair lobes' inner edge. At 0.090 it stopped short
        # of them and left an open slot at x 0.09..0.20, y -0.13..-0.05, under the back corner
        # of the jaw -- a notch you could see into from below and behind. It still shows in
        # profile because the lobes sit BEHIND it (y <= -0.130), not beside it.
        box(-0.130, 0.130, -0.128, 0.060, p["neck_bot"], p["neck_top"] + 0.020),
        # ---- appended, so boxes 0-9 keep the indices the checks and weights name ----
        # 10, 11: the two brow masses, at the art's 0.181 H front
        box(-0.150, -0.034, p["y_brow_art"] - 0.016, p["y_brow_art"],
            row_bot(29), row_top(25)),
        box(0.034, 0.150, p["y_brow_art"] - 0.016, p["y_brow_art"],
            row_bot(29), row_top(25)),
        # 12: the face plate, proud of the recessed skull and narrower than it, so the head's
        # front corners step back instead of meeting at a hard edge
        box(-hw + 0.014, hw - 0.014, p["y_face_art"] - 0.014, p["y_face_art"],
            p["neck_top"] - o, 1.0800),
        # 13: the crown chamfer -- the skull steps in before the hair takes over
        box(-hw + 0.012, hw - 0.012, y_skull_back + 0.008, p["y_face_art"] - 0.018,
            1.0760, p["skull_top"]),
    ]


def part_hair():
    p = P
    o = OVERLAP
    hw = p["hw_crown"]
    cs2, cs1, top, crown = p["crown_s2"], p["crown_s1"], p["skull_top"], p["crown"]
    yb, yf = p["y_head_back"], p["y_hair_front"]
    return [
        # THE CAP, over the top and the back; its front edge is the fringe at column 71.
        #
        # It stops at the BOTTOM of row 15, where the crown's first step begins. Run up to
        # skull_top it covered row 15 at the full 0.336 width where the art is 0.320, and the
        # box that was meant to bridge the gap had its z running BACKWARDS -- 1.1276 down to
        # 1.1271, a box of negative height. normals() cannot see that one: a degenerate box
        # has no outward direction to test against.
        wedge(-p["hw_head"], p["hw_head"], yb, yf, p["ear_top"], row_bot(15) + o),
    ] + [
        # Half-widths read off front.png's own RIGHT edge, row by row -- the left edge is
        # under the pickaxe from row 8 down, and the art is asymmetric by 1-2 px anyway, so
        # one clean edge is the honest source. The table's "0.336 at row 15" is the art's row
        # SIXTEEN value; built on the table this crown stood 4.4 source px outside the sheet.
        #
        # Each band spans its rows' own EDGES, not their centres. Boundaries computed by hand
        # grazed the sheet's pixel rows: a band top landing 0.03 mm inside the row above put
        # the wider step into that row and read as 3.8 px of overshoot.
        # DEPTH comes from side-left.png's own rows too, not from an inset off the fringe.
        # The art's crown is deep -- its front edge runs +0.131 H at row 7 out to +0.167 by
        # row 10 -- and an inset off y_hair_front left the top of the head up to 13 source px
        # shallower than the sheet, which is a good part of why the profile read as a slab.
        wedge(-half * H, half * H, back * H, front * H,
            row_bot(r_lo) - OVERLAP, min(row_top(r_hi), crown))
        # MEAN of the art's two edges where both are clean, not the right edge alone. The art
        # is hand-drawn and asymmetric -- row 12 is -0.124 left against +0.139 right -- so a
        # symmetric model built to one edge is 2.1 px outside the other. The mean splits that
        # in half, which is the best a symmetric figure can do. Rows 8-11 keep the right edge
        # because the pickaxe covers their left.
        # rows, half-width, then the side view's back and front edges -- all in H
        for r_hi, r_lo, half, back, front in (
            (15, 15, 0.1600, -0.160, 0.174),
            (13, 14, 0.1425, -0.139, 0.167),
            (12, 12, 0.1315, -0.139, 0.167),
            # rows 10 and 11 differ in DEPTH (-0.103 against -0.139) though not in width, so
            # they are separate bands: averaged into one they sat 2.5 px behind row 10.
            (10, 10, 0.1170, -0.103, 0.167),
            (11, 11, 0.1170, -0.139, 0.167),
            (8, 9, 0.0890, -0.103, 0.135),
            (7, 7, 0.0775, -0.096, 0.131),
        )
    ] + [
        # Back mass, full width, hanging to 0.707 -- but only BEHIND the neck's back face.
        # Three bands, not one slab: f084 and f088 show the back of the hair stepping in
        # layers. The middle band alone reaches column 36, which is the sheet's hair back,
        # so the stepping never pushes the head past its 0.336 depth. The bottom band is
        # first because the 0.707 height check reads this box.
        wedge(-p["hw_head"], p["hw_head"], yb + 0.013, -0.130, p["head_ends"], 0.904),
        wedge(-p["hw_head"], p["hw_head"], yb, -0.130, 0.900, 0.984),
        wedge(-p["hw_head"], p["hw_head"], yb + 0.009, -0.130, 0.980, p["ear_top"] + o),
        # Side lobes come forward to the temples ONLY above the neck. Below the top of the
        # bare run they stay behind y = -0.130, or they win the side view on |x| and bury
        # the neck -- which is exactly what they did on the first build (0.000 H of neck).
        wedge(0.130, p["hw_head"], yb, 0.020, p["neck_top"] - o, p["ear_top"] + o),
        wedge(-p["hw_head"], -0.130, yb, 0.020, p["neck_top"] - o, p["ear_top"] + o),
        wedge(0.130, p["hw_head"], yb, -0.130, p["head_ends"], p["neck_top"] + o),
        wedge(-p["hw_head"], -0.130, yb, -0.130, p["head_ends"], p["neck_top"] + o),
        # THE FRINGE, and it is appended rather than inserted so every earlier box index --
        # which measure() names to check the crown steps -- keeps its meaning.
        #
        # The cap above stops at column 71, so from the FRONT the skull's own face showed
        # through for 77 mm between the brow ridge and the crown: a bare forehead f104 does
        # not have. This band brings the hair forward to the face plane (column 83, which is
        # what the depth table itself calls "front of the hair") from just above the brow
        # ridge upward. Column 71 stays the side lobes' front edge in profile, so both
        # readings of sec.3 hold.
        # 4 mm proud of the face plane, not flush with it: a fringe ending exactly on
        # y_face is coplanar with the skull's front face and z-fights, which rendered as a
        # pale band across the forehead.
        wedge(-p["hw_head"], p["hw_head"], yf - o, p["y_fringe_art"], 1.072,
            p["skull_top"] + o),
    ] + [
        # Locks down the back of the hair. f084 and f088 read it as layered strands, and
        # this is the largest unbroken area left on the figure. They sit INSIDE the hair's
        # back plane so the 0.336 depth is untouched.
        wedge(lx - 0.022, lx + 0.022, yb + 0.004, yb + 0.026, p["head_ends"], 1.020)
        for lx in (-0.150, -0.080, 0.000, 0.080, 0.150)
    ] + [
        # and two temple locks either side, in front of the back mass
        wedge(-p["hw_head"] + 0.004, -p["hw_head"] + 0.030, yb + 0.020, -0.136,
            p["head_ends"], p["neck_top"]),
        wedge(p["hw_head"] - 0.030, p["hw_head"] - 0.004, yb + 0.020, -0.136,
            p["head_ends"], p["neck_top"]),
        # FACE-FRAMING LOBES. f100 is unambiguous: the hair comes forward past the ears and
        # down the cheeks, framing the face, in two depth layers. They sit ABOVE the bare
        # neck run so they cannot bury it, and at y <= +0.118 they stay well inside the art's
        # +0.167 H front edge at these rows.
        wedge(0.150, p["hw_head"], -0.060, 0.118, p["neck_top"], 1.0400),
        wedge(-p["hw_head"], -0.150, -0.060, 0.118, p["neck_top"], 1.0400),
        # the second, shallower layer behind them
        wedge(0.128, 0.166, -0.100, 0.070, p["neck_top"] - 0.030, 1.0200),
        wedge(-0.166, -0.128, -0.100, 0.070, p["neck_top"] - 0.030, 1.0200),
        # THE NAPE TONGUE, appended last. The hair mass stops dead at head_ends (0.848) and
        # the torso starts at 0.848, so the two only touch -- 0.4 mm at bind -- and a 20 deg
        # nod measures 34.1 mm between them.
        #
        # What that distance is actually reporting took a rear render with the pack hidden
        # to pin down, and it is NOT a hole through the figure. Nothing opens to daylight:
        # the head, the neck column and the hair are one rigid assembly (WEIGHTS gives head
        # box 9 to "neck" and everything above it to "head", and head is neck's child), so
        # a nod cannot move any of them relative to each other. What moves is the SIGHTLINE.
        # The hair's back band hangs at y -0.176..-0.130 and the neck column's back face is
        # at -0.128, just 2 mm in front of it, so the band covers the nape only for rays
        # arriving near horizontal. Tilt the assembly 20 deg and a horizontal ray passes
        # under the band's bottom edge and lands on the neck column instead -- which paints
        # skin, so what Wolf sees is a pale strip appearing across the nape, bright against
        # the hair. Ray-cast from the rear camera at neck -20, z 0.852 through 0.876 all
        # return r17_head; at bind every one of them returns r17_hair. That strip, not a
        # slit, is the defect.
        #
        # So this box is not a spacer, it is an OCCLUDER: a tongue of hair hanging down the
        # nape at y -0.148..-0.130, immediately behind the neck column, low enough that the
        # tilted sightline still meets hair. It is what "extend the lobes' lower edge down"
        # means once the mechanism is known.
        #
        # Extents come from ray-cast FACES, not bounding boxes, and that distinction cost a
        # build: the torso's bounds say its back is -0.1794 while its back face above z 0.78
        # is -0.1614, and the pack's front face is -0.1714, so a 10 mm channel runs open
        # between them. A first attempt sat in it and read as 244 changed pixels in the SIDE
        # silhouette. Measured, the cover is:
        #
        #     z 0.770..0.844   torso   x +-0.190..0.198   back face -0.1614
        #     z 0.844..0.848   torso   x +-0.150          back face -0.1494
        #     y < -0.1714      pack    x +-0.1776         (its own shadow, all these z)
        #
        # so y stops at -0.148 (1.4 mm inside the tightest torso face) and x at +-0.145
        # (5 mm inside the collar step).
        #
        # The TOP stops at 0.847, below the torso's own top at 0.8480, and that 1 mm is
        # deliberate. The committed figure has a real 0.4 mm see-through crack at bind
        # between the torso's top (0.8480) and the hair's bottom (0.8484): at the nape the
        # pack's front face is -0.1714 and the head's back is -0.1280, so nothing stands
        # behind y -0.148..-0.130 and the slit reads as 19 partial-alpha pixels in the SIDE
        # outline. Running this box up to head_ends filled them -- an improvement, but a
        # silhouette change, and closing a crack that has been in the outline since r14 is
        # not this round's call to make. Stopping at 0.847 leaves those 19 pixels exactly as
        # they were. The occlusion this box exists for happens between 0.770 and 0.848 and
        # does not need the last millimetre. Checked by rendering the outlines rather than
        # reasoning about them: 0 px changed, front and side.
        box(-0.145, 0.145, -0.148, -0.130, 0.770, 0.847),
    ]


def part_beard():
    """0.343 wide at the top, tapering, tip at 0.464. Front at column 86.

    Five steps. It starts at the cheeks and hangs to just above the belt; the tunic and
    the belt show either side of it because 0.343 sits well inside the 0.528 shoulders.
    """
    p = P
    o = OVERLAP
    hwb, yf = p["hw_beard"], p["y_beard_f"]
    # Boxes 1 and 2 carry the sheet's 0.343 width; box 1 alone reaches the 0.357 front
    # (column 86). Every band takes a different front plane so the beard steps in profile
    # instead of presenting one flat face.
    bands = [
        (0.900, 0.960, hwb - 0.036, 0.030, yf - 0.034),
        (0.830, 0.905, hwb, 0.020, yf),
        (0.760, 0.835, hwb, 0.020, yf - 0.011),
        (0.670, 0.765, hwb - 0.031, 0.020, yf - 0.031),
        (p["beard_tip"], 0.675, hwb - 0.071, 0.020, yf - 0.058),
    ]
    out = [wedge(-hw, hw, y0, y1, z0 - (o if z0 > p["beard_tip"] else 0.0), z1)
           for z0, z1, hw, y0, y1 in bands]
    # Two locks standing proud of the beard's front. The reference beard is not one smooth
    # face -- f104 and f088 both read it as hanging strands -- and these are steps on the
    # front edge of the side silhouette, which is where our density is thinnest. They stay
    # ABOVE the 0.464 H tip, so the beard's length is unchanged.
    out += [
        wedge(-0.072, -0.012, yf - 0.006, yf + 0.016, 0.600, 0.836),
        wedge(0.016, 0.070, yf - 0.020, yf + 0.006, 0.646, 0.848),
        # THE CENTRE MASS, proud of the sides. In f100 and f104 the beard is not a slab: its
        # middle carries forward and the sides fall away, which is what gives it volume at
        # 60 px. It stays inside column 86, the sheet's beard front.
        # TWO bands, because the art's front DIPS here: +0.210 H at row 41, back to +0.196
        # at rows 43-49, out again to +0.210 by row 51. That dip between the moustache and
        # the beard's main mass is real form, and a single centre mass at one depth stood
        # 2.8 px proud through it.
        wedge(-0.104, 0.104, 0.030, h(0.196), 0.826, 0.902),
        wedge(-0.104, 0.104, 0.030, h(0.208), 0.640, 0.830),
        wedge(-0.086, 0.086, 0.030, h(0.204), 0.586, 0.660),
        # and a stepped bottom rather than a flat cut, above the 0.464 H tip
        wedge(-0.062, 0.062, 0.030, yf - 0.042, p["beard_tip"], 0.600),
        # cheek masses, where the beard meets the sideburns
        wedge(-p["hw_beard"], -0.120, 0.030, yf - 0.048, 0.856, 0.946),
        wedge(0.120, p["hw_beard"], 0.030, yf - 0.048, 0.856, 0.946),
    ]
    return out


def part_moustache():
    p = P
    # A centre block with two wings dropping away either side, which is how f100 and f104
    # read it -- proud of the beard, wider than the mouth, and stepped rather than one bar.
    yf = p["y_beard_f"]
    return [
        # It sits on rows 39-42, where the art's front is +0.217 to +0.210 H. Dropped to rows
        # 43-44 at that depth it ran 2.3 px past the sheet, which only allows +0.196 there.
        box(-0.072, 0.072, p["y_face_art"] - 0.006, h(0.208), 0.894, 0.932),
        box(-0.126, -0.066, p["y_face_art"] - 0.006, h(0.200), 0.898, 0.928),
        box(0.066, 0.126, p["y_face_art"] - 0.006, h(0.200), 0.898, 0.928),
    ]


def part_torso():
    p = P
    o = OVERLAP
    hw, yb, yf = p["hw_chest"], p["y_torso_b"], p["y_torso_f"]
    # Box 1 is the chest and carries the sheet's 0.286 depth. The collar above and the waist
    # below step BACK, so the profile reads as three masses rather than one slab -- the side
    # silhouette scored 3.4 steps/100 rows against the reference's 10.3 when all three sat
    # at the same y.
    return (
        chamfered(-hw, hw, yb + 0.010, yf - 0.026, 0.740 - o, p["cap_top"], 0.020, 0.016)
        + chamfered(-hw, hw, yb, yf, 0.560 - o, 0.745, 0.022, 0.018)
        + chamfered(-hw + 0.010, hw - 0.010, yb + 0.014, yf - 0.014,
                    p["belt_bot"], 0.565, 0.018, 0.014)
    ) + [
        # collar band at the neckline and a yoke step across the shoulders -- both are plain
        # in f104 and f140, and both are steps the side silhouette does not currently have
        box(-0.150, 0.150, yb + 0.022, yf - 0.012, 0.818, 0.848),
        box(-hw - 0.008, hw + 0.008, yb - 0.008, yf - 0.040, 0.782, 0.812),
        # INTERIOR DETAIL from the lit frames. The ortho sheet is 140 px tall and cannot
        # carry any of this; sec.1 names f088/f104/f140 as the form authority and they show
        # it plainly. None of it touches the silhouette, which is already within 1-2 px.
        # f104: a collar either side of the neck, and a placket down the chest.
        box(-0.104, -0.030, yf - 0.030, yf + 0.004, 0.806, 0.844),
        box(0.030, 0.104, yf - 0.030, yf + 0.004, 0.806, 0.844),
        box(-0.026, 0.026, yf - 0.004, yf + 0.005, 0.560, 0.822),
        # no side seams: they are not resolvable in any frame either
        # the hem lip of the tunic body, under the belt
        box(-hw - 0.006, hw + 0.006, yb + 0.010, yf - 0.010, 0.412, 0.436),
    ]


def part_skirt():
    p = P
    o = OVERLAP
    yb, yf = p["y_skirt_b"], p["y_skirt_f"]
    # Box 2 is the hem and carries the sheet's 0.439 width and 0.300 depth; the two above it
    # step back in both axes so the skirt flares in profile as well as in front.
    # The skirt's BACK is the one part of it the art exposes -- from row 95 down, where the
    # pack ends -- and it runs -0.124 H at row 100 to -0.153 by row 118. A constant back off
    # column 37 sat 2.7 px behind the sheet at rows 99-102. The front stays on the table:
    # the arm and the lantern cover it in every view.
    return (
        chamfered(-0.210, 0.210, h(-0.125), yf - 0.016, 0.390 - o, 0.460, 0.022, 0.016)
        + chamfered(-0.238, 0.238, h(-0.142), yf - 0.008, 0.300 - o, 0.395, 0.024, 0.018)
        + chamfered(-p["hw_waist"], p["hw_waist"], h(-0.153), yf, p["hem"], 0.305,
                    0.026, 0.020)
    ) + [
        # hem lip, standing proud all round the bottom edge, and the front split f104 shows
        # down the centre of the skirt
        # the hem lip is proud in Y only -- flared 9 mm in X too it pushed the widest part
        # of the skirt to 0.454 H against the sheet's 0.439
        box(-p["hw_waist"], p["hw_waist"], yb, yf + 0.012,
            p["hem"], p["hem"] + 0.026),
        box(-0.036, 0.036, yf - 0.006, yf + 0.014, p["hem"] + 0.020, 0.372),
    ]
    # NO FOLD RIDGES. Ten of them were added here on the claim that "f088 and f104 read the
    # tunic skirt as panelled cloth". Checked against the frames: they do not. The skirt is a
    # flat green mass in both, and its variation is painted value, not geometry. The ridges
    # were added because the triangle count was low and the reference reading was written to
    # fit -- which is the thing sec.6 forbids.


def part_belt():
    p = P
    return [
        # The belt is exposed behind only below z/H 0.372, where the pack ends, and the art
        # reads -0.124 to -0.139 H there. Carried back to the torso's own column 38 it was
        # 2.4 px behind the sheet at rows 99-102.
        box(-0.196, 0.196, h(-0.134), p["y_torso_f"] + 0.008,
            p["belt_bot"], p["belt_top"]),
        # the strap end hanging past the buckle, which f104 and f164 both show
        box(-0.104, -0.052, p["y_torso_f"] + 0.004, p["y_torso_f"] + 0.018, 0.352, 0.428),
        # f140 carries a second strap and buckle at the figure's left hip, below the belt.
        # The "row of stitch blocks either side of the buckle" that sat here is gone: the
        # belt in both frames is a band and a buckle, nothing else.
        box(0.118, 0.186, p["y_torso_f"] - 0.006, p["y_torso_f"] + 0.026, 0.330, 0.418),
        box(0.126, 0.178, p["y_torso_f"] + 0.020, p["y_torso_f"] + 0.032, 0.392, 0.412),
    ]


def part_buckle():
    p = P
    hw = p["buckle_w"] / 2
    mid = (p["belt_bot"] + p["belt_top"]) / 2
    return [box(-hw, hw, p["y_torso_f"] + 0.004, p["y_torso_f"] + 0.020,
                mid - hw, mid + hw)]


def part_sleeve(side):
    """Shoulder cap + upper arm + bare forearm, down to the wrist."""
    p = P
    o = OVERLAP
    pivot = (p["arm_x"], 0.0, p["cap_top"])
    parts = chamfered(p["hw_chest"] - 0.006, p["hw_shoulder"], -0.160, 0.160,
                      0.730, p["cap_top"], 0.016, 0.020) + [
        # (the cap above is what carries the 0.528 shoulder width, not the torso)
        arm_box(0.055, p["arm_cuff_t"] + o, 0.062, 0.068, pivot, p["arm_deg"], 1),
        arm_box(p["arm_cuff_t"], p["arm_len"] - p["hand_len"] + o, 0.052, 0.058,
                pivot, p["arm_deg"], 1),
        # the sleeve's cuff band at 0.566 H, standing proud of the bare forearm below it
        arm_box(p["arm_cuff_t"] - 0.024, p["arm_cuff_t"] + 0.014, 0.068, 0.074,
                pivot, p["arm_deg"], 1),
    ]
    return parts if side > 0 else mirror_x(parts)


def part_glove(side):
    p = P
    pivot = (p["arm_x"], 0.0, p["cap_top"])
    t0 = p["arm_len"] - p["hand_len"] - OVERLAP
    parts = [
        # palm, then a knuckle step across its outboard end -- the reference hand is not one
        # box, and f104's close range lands on the hands
        arm_box(t0, p["arm_len"] - 0.026, 0.060, 0.064, pivot, p["arm_deg"], 1),
        arm_box(p["arm_len"] - 0.030, p["arm_len"], 0.052, 0.058, pivot, p["arm_deg"], 1),
        # thumb: on the INBOARD edge (+e1) with a little forward lead, not on the palm face
        arm_box(t0 + 0.012, t0 + 0.062, 0.022, 0.026, pivot, p["arm_deg"], 1,
                off=(0.062, 0.022)),
        # NO FINGER BLOCKS. Three per hand were added on the claim that f104's close range
        # shows them; it does not -- the hands read as a block with a thumb in every frame,
        # which is what is built. The wrist cuff stays: that one is visible.
        arm_box(t0 - 0.006, t0 + 0.022, 0.066, 0.070, pivot, p["arm_deg"], 1),
    ]
    return parts if side > 0 else mirror_x(parts)


def part_leg(side):
    """Shin then thigh, as two boxes.

    Not a step for its own sake: rigid weighting gives a whole box to one joint, and a single
    leg box would have to be split vertex-by-vertex between hip and knee, which shears the box
    instead of rotating it. Two boxes let hip and knee each own one outright. Box 0 stays the
    shin so the 0.164 depth check keeps pointing at it.
    """
    p = P
    o = OVERLAP
    xo = p["hw_stance"]
    parts = [
        box(xo - p["w_limb"], xo, p["y_shin_b"], p["y_shin_f"], 0.190, 0.262),
        box(xo - p["w_limb"] + 0.004, xo - 0.004, p["y_shin_b"] + 0.004, p["y_shin_f"] - 0.004,
            0.262 - o, 0.330),
        # THE ANKLE PLUG, appended. The shin's bottom sits at 0.190 and the cuff's top at
        # 0.1884, so leg and boot barely meet -- 1.6 mm apart at bind, and the only thing
        # bridging them is the cuff lip, which reaches 0.1944 and overlaps the shin by 4.4 mm.
        # That is not enough: at foot -15 deg the pair opens 17.6 mm, and it is already
        # 12.5 mm at -10 deg. A walk cycle rotates the ankle every step, so the slit would
        # shimmer along the boot top through the whole animation. This buries 38 mm of leg
        # column inside the cuff, so the rotation runs out of angle before it runs out of
        # overlap.
        #
        # A SEPARATE box, appended, rather than box 0's bottom dropped to 0.150 -- which is
        # the obvious edit and is wrong. The shin's back face is y_shin_b (-0.094) and the
        # cuff's is y_shin_b + 0.009, so a lowered shin would stand 9 mm proud of the cuff's
        # back between z 0.150 and 0.1884. That stretch is a notch in the SIDE silhouette,
        # and filling it with bare shin would move the outline. Inset in y and x the plug is
        # inside the cuff from every direction, and appending keeps box 0 the shin for the
        # 0.164 depth check and box 1 the thigh for the hip weight.
        box(xo - p["w_limb"] + 0.004, xo - 0.004,
            p["y_shin_b"] + 0.013, 0.096,
            0.150, 0.190 + o),
    ]
    return parts if side > 0 else mirror_x(parts)


def part_boot(side):
    p = P
    o = OVERLAP
    xo = p["hw_stance"]
    xc = xo - p["w_limb"] / 2
    hwc = p["w_bootcuff"] / 2
    parts = [
        # The sole is no wider than the boot: front.png reads +-0.196 H at rows 135-146, and
        # a 0.110 half-width flared it to 0.209 H, 1.8 px outside the sheet at the very
        # bottom of the figure -- the last front violation left after the crown was fixed.
        box(xc - 0.0947, xc + 0.0947, p["y_shin_b"], p["y_sole_f"], 0.000, 0.032),   # sole
        # The boot's front STEPS: the art reads +0.074 H at rows 133-136 and only reaches
        # +0.131 at row 139, down at the toe. One box front at 0.110 m stood 2.5 px forward
        # of the sheet at z/H 0.099.
    ] + chamfered(xo - p["w_limb"], xo, p["y_shin_b"] + 0.004, 0.110,
                  0.028 - o, 0.104, 0.016, 0.014) + chamfered(
                  xo - p["w_limb"], xo, p["y_shin_b"] + 0.004, 0.088,
                  0.100, 0.135, 0.016, 0.014) + [
        box(xc - 0.091, xc + 0.091, 0.100, p["y_sole_f"], 0.028 - o, 0.080),         # toe
        box(xc - hwc, xc + hwc, p["y_shin_b"] + 0.009, 0.105,
            p["boot_cuff_b"], p["boot_cuff_t"]),                                     # cuff
        # heel block under the rear of the sole -- f088 shows a distinct heel, and it is
        # appended so the sole/body/cuff checks keep reading boxes 0, 1 and 3
        box(xc - 0.095, xc + 0.095, p["y_shin_b"], -0.030, 0.000, 0.052),            # heel
        # welt between sole and upper, and a toe cap -- both read clearly in f088, and both
        # are steps in the SIDE silhouette, which is the view carrying half the reference's
        # step density
        box(xc - 0.101, xc + 0.101, p["y_shin_b"] + 0.002, 0.148, 0.030, 0.046),     # welt
        # the toe cap tops out at 0.080: the sheet's boot steps back above z/H 0.077, and at
        # 0.096 this box stood 7 source px proud of the sheet's outline in the side view
        box(xc - 0.086, xc + 0.086, 0.086, p["y_sole_f"] - 0.004, 0.044, 0.080),     # toe cap
        # f088's boots carry a strap with a buckle across the instep, a cuff lip, and a
        # stitched welt line up the back. Interior detail, inside the silhouette.
        box(xc - 0.090, xc + 0.090, 0.016, 0.050, 0.082, 0.126),
        box(xc - 0.028, xc + 0.028, 0.010, 0.056, 0.092, 0.118),
        box(xc - hwc + 0.004, xc + hwc - 0.004, p["y_shin_b"] + 0.004, 0.100,
            p["boot_cuff_t"] - 0.018, p["boot_cuff_t"] + 0.006),
        # no back welt: it sat inside the boot and was never visible from anywhere
    ]
    return parts if side > 0 else mirror_x(parts)


def part_pack():
    p = P
    # the flap is the REARMOST thing on the figure, so the body stops short of column 12
    # and the flap reaches it -- otherwise pack-to-nose overshoots the sheet's 0.550 H.
    # Boxes 0 and 1 carry the sheet's 0.186 depth; the bedroll and pocket are added after
    # them so the measured checks keep pointing at the right masses.
    yb = p["y_pack_b"]
    yf_pack = p["y_pack_f"]
    # THE PACK TAPERS. side-left.png's back edge runs -0.285 H at row 45 down to -0.331 at
    # rows 68-75 and back to -0.275 by row 94: deepest in the middle, shallower top and
    # bottom. Built as one constant-depth slab at the table's column 12 it stood 5.5 source
    # px behind the sheet at z/H 0.726 -- the single largest envelope violation in the side
    # view -- while still being correct at its deepest row, which is the row the table quotes.
    # Boxes 0 and 1 stay the ones the 0.186 H depth check reads.
    bands = [
        (0.700, 0.752, -0.285), (0.650, 0.702, -0.295), (0.590, 0.652, -0.305),
        (0.510, 0.592, -0.331), (0.430, 0.512, -0.324), (0.400, 0.432, -0.300),
        (0.372, 0.402, -0.275),
    ]
    return [
        wedge(-0.185, 0.185, h(-0.331), yf_pack, h(0.510), h(0.592)),
        wedge(-0.155, 0.155, h(-0.331), h(-0.311), h(0.520), h(0.582)),
    ] + [
        wedge(-0.185, 0.185, h(back), yf_pack, h(lo), h(hi)) for lo, hi, back in bands
    ] + [
        # Bedroll across the TOP of the pack -- f088 shows it standing above and behind,
        # with its rolled end proud at the sides. It sits at rows 36-38's own back edge
        # (-0.289 H); left at the old flat -0.394 it became the deepest thing at that height
        # and simply inherited the violation the tapering was meant to remove.
        # two bands, because the art ramps here too: -0.267 at row 35 to -0.289 by row 38.
        # One band at -0.289 stood 3.0 px behind the sheet at its top.
        wedge(-0.170, 0.170, h(-0.272), h(-0.230), h(0.780), h(0.800)),
        wedge(-0.170, 0.170, h(-0.289), h(-0.230), h(0.752), h(0.784)),
        wedge(0.170, 0.192, h(-0.283), h(-0.236), h(0.757), h(0.795)),
        wedge(-0.192, -0.170, h(-0.283), h(-0.236), h(0.757), h(0.795)),
        # lower pocket, a touch proud of the band it sits on (side-left.png)
        wedge(-0.115, 0.115, h(-0.328), h(-0.300), h(0.435), h(0.500)),
        # f088's pack detail: two vertical straps down the flap, each with a buckle plate,
        # and a pocket on each side wall. All interior -- none of it moves the outline.
        # straps and buckles stay INSIDE -0.331 H, the pack's own deepest band -- past it
        # they became the rearmost thing on the figure and pushed pack-to-nose 1.3 px over
        wedge(-0.092, -0.052, h(-0.329), h(-0.310), h(0.430), h(0.700)),
        wedge(0.052, 0.092, h(-0.329), h(-0.310), h(0.430), h(0.700)),
        wedge(-0.100, -0.044, h(-0.330), h(-0.318), h(0.505), h(0.545)),
        wedge(0.044, 0.100, h(-0.330), h(-0.318), h(0.505), h(0.545)),
        wedge(-0.190, -0.172, h(-0.300), h(-0.200), h(0.470), h(0.610)),
        wedge(0.172, 0.190, h(-0.300), h(-0.200), h(0.470), h(0.610)),
    ]


def part_strap(side):
    """Thin, on the sleeve cap, inboard -- never outboard of the neck in profile."""
    # It sits ON the cap and stops AT the shoulder line. A strap that rides above 0.700 H
    # crosses the bare-neck run and wins the side view on |x|: at 0.830..0.852 it cut the
    # run to 0.079 H, a millimetre under the sec.2 target, for a 22 mm strap.
    parts = [box(0.150, 0.250, -0.200, 0.140, 0.820, 0.844)]
    return parts if side > 0 else mirror_x(parts)


def _hand_point(side):
    p = P
    a = math.radians(p["arm_deg"])
    t = p["arm_len"] - p["hand_len"] / 2
    return (side * (p["arm_x"] + math.sin(a) * t), 0.0,
            p["cap_top"] - math.cos(a) * t)


def part_pickaxe():
    """Shaft 0.91 H, blade span 0.38 H, head span / length 0.42. In the right hand."""
    p = P
    hx, _, hz = _hand_point(1)
    z0 = hz - p["pick_shaft"] * 0.28
    top = z0 + p["pick_shaft"]
    b = p["pick_blade"] / 2
    # gear.png, now actually opened. It draws a SYMMETRIC double pick: both arms sweep out
    # and DOWN from the haft to a point, with a collar where the head meets the shaft and a
    # banded grip below it. An earlier version of this comment claimed the same drawing
    # showed a point one side and a flat adze the other -- it does not, and the file had not
    # been opened when that was written.
    arms = []
    for s in (-1, 1):
        arms += [
            box(hx - 0.022, hx + 0.022, min(s * 0.050, s * b * 0.62),
                max(s * 0.050, s * b * 0.62), top - 0.066, top - 0.008),
            box(hx - 0.018, hx + 0.018, min(s * b * 0.58, s * b * 0.86),
                max(s * b * 0.58, s * b * 0.86), top - 0.082, top - 0.030),
            box(hx - 0.014, hx + 0.014, min(s * b * 0.82, s * b),
                max(s * b * 0.82, s * b), top - 0.104, top - 0.058),
        ]
    return [
        box(hx - 0.019, hx + 0.019, -0.019, 0.019, z0, top - 0.030),           # haft
        box(hx - 0.024, hx + 0.024, -0.026, 0.026, top - 0.082, top - 0.040),  # collar
        box(hx - 0.026, hx + 0.026, -0.058, 0.058, top - 0.062, top),          # head centre
        # the banded grip gear.png draws down the haft below the collar
        box(hx - 0.023, hx + 0.023, -0.023, 0.023, top - 0.196, top - 0.092),
    ] + arms


def part_lantern():
    """0.26 H tall, hanging from the left hand, in contact with it."""
    p = P
    hx, _, hz = _hand_point(-1)
    top = hz - 0.010
    bot = top - p["lantern_h"]
    # A lantern is a FRAME around glass, not a solid block: the contact sheet and f104 both
    # show corner posts, a vented cap and a footed base. The posts are what make the glass
    # read as glass when it is lit.
    g, pz0, pz1 = 0.036, bot + 0.030, top - 0.046
    posts = [box(hx + sx * g - 0.008, hx + sx * g + 0.008, sy * g - 0.008, sy * g + 0.008,
                 pz0 - 0.004, pz1 + 0.004)
             for sx in (-1, 1) for sy in (-1, 1)]
    return [
        box(hx - 0.009, hx + 0.009, -0.008, 0.008, top - 0.034, top),            # bail
        box(hx - 0.046, hx + 0.046, -0.046, 0.046, top - 0.054, top - 0.026),    # cap
        box(hx - 0.034, hx + 0.034, -0.034, 0.034, top - 0.070, top - 0.048),    # vent
        box(hx - 0.034, hx + 0.034, -0.034, 0.034, pz0, pz1),                    # glass
        box(hx - 0.048, hx + 0.048, -0.048, 0.048, bot + 0.010, bot + 0.040),    # base
        box(hx - 0.040, hx + 0.040, -0.040, 0.040, bot, bot + 0.016),            # foot
    ] + posts


PART_BUILDERS = [
    ("r17_head", part_head),
    ("r17_hair", part_hair),
    ("r17_beard", part_beard),
    ("r17_moustache", part_moustache),
    ("r17_torso", part_torso),
    ("r17_skirt", part_skirt),
    ("r17_belt", part_belt),
    ("r17_buckle", part_buckle),
    ("r17_sleeve.R", lambda: part_sleeve(1)),
    ("r17_sleeve.L", lambda: part_sleeve(-1)),
    ("r17_glove.R", lambda: part_glove(1)),
    ("r17_glove.L", lambda: part_glove(-1)),
    ("r17_leg.R", lambda: part_leg(1)),
    ("r17_leg.L", lambda: part_leg(-1)),
    ("r17_boot.R", lambda: part_boot(1)),
    ("r17_boot.L", lambda: part_boot(-1)),
    ("r17_pack", part_pack),
    ("r17_strap.R", lambda: part_strap(1)),
    ("r17_strap.L", lambda: part_strap(-1)),
    ("r17_pickaxe", part_pickaxe),
    ("r17_lantern", part_lantern),
]


# ===================================================================== stage B: paint
#
# The atlas is a PALETTE, not a skin. Every flat face maps to the CENTRE of a 32 px colour
# cell, so all four of its UVs sit on one texel: the face is exactly one colour, no bleeding,
# no filtering artefacts, and the "crisp value steps, no gradients" clause holds by
# construction rather than by care. The only exception is the face front, which gets a fixed
# rectangular island with a real planar projection so the eyes land on a known world position
# and cannot drift between runs (sec.5).
#
# Value steps come from face ORIENTATION, read off the polygon normal rather than the face
# index, so the rotated arm boxes step the same way the axis-aligned ones do.

TEX = "T_VoxelDwarf_r17"
MAT = "M_VoxelDwarf_r17"
TEX_SIZE = 512
CELL = 32                        # 16 x 16 cells
FACE_ISLAND = (8, 8, 176, 128)   # x, y, w, h in pixels, bottom-left origin
FACE_BOXES = (0, 1, 2, 10, 11, 12)   # r14_head boxes whose +Y face carries the island
FACE_RECT = (-0.2016, 0.2016, 0.8484, 1.1316)     # world x0,x1,z0,z1 the island covers

# A chamfer plane faces halfway between the front and the side, and this ramp is a
# non-directional one -- top down to bottom -- so the corner belongs BETWEEN front
# and side, not above front. It was first set to 1.09, brighter than the front face,
# which made every wedge read as a highlight strip instead of a transition and was
# why the third plane still did not show after the geometry was there.
STEPS = {"top": 1.18, "front": 1.00, "corner": 0.92, "side": 0.85,
         "bottom": 0.66}

# which palette cell each part is, with per-box overrides where a part is two materials
PAINT = {
    # the brow ridge and the nose stand 15-50 mm proud, so they take a brighter skin step --
    # without it a dead-on front view paints them the same value as the face plane and the
    # nose disappears entirely
    # Boxes 3-6 are the nose steps and take the brighter skin. The BROW RIDGE (box 2) does
    # not: it runs the full head width, so a value step on it reads as a headband across the
    # face. Its form comes from the painted brows just under it.
    "r17_head": ("skin", {3: "skinhi", 4: "skinhi", 5: "skinhi", 6: "skinhi"}),
    # hairhi is the base, with the approved #34271C doing the shadow work through the
    # orientation steps rather than being the base and crushing everything to black
    "r17_hair": ("hairhi", {}),
    # Same inversion as the hair: #826145 is the base and #5E4632 the shadow, not the other
    # way round. f104's beard is a warm mid-brown that reads clearly LIGHTER than the hair;
    # built on #5E4632 it went black in shadow and the two masses merged into one.
    # The main hanging bands keep #5E4632; beardhi lights the cheeks, the proud centre mass
    # and the locks. Beardhi on everything read a shade too light and orange against f104,
    # and it pulled antialiased beard/skin boundary pixels into the brown families.
    "r17_beard": ("beardhi", {1: "beard", 2: "beard", 3: "beard", 4: "beardlo"}),
    "r17_moustache": ("beardhi", {}),
    "r17_torso": ("tunic", {}),
    "r17_skirt": ("tunic", {}),
    "r17_belt": ("trunk", {}),
    "r17_buckle": ("metal", {}),
    "r17_sleeve.R": ("tunic", {2: "skin"}),        # box 2 is the bare forearm
    "r17_sleeve.L": ("tunic", {2: "skin"}),
    "r17_glove.R": ("wood", {}),
    "r17_glove.L": ("wood", {}),
    "r17_leg.R": ("pants", {}),
    "r17_leg.L": ("pants", {}),
    # f104's boots are LIGHT tan against dark trousers. Built on the darker leather they sank
    # into the background at 60 px and the whole lower body went illegible, so the boot body
    # is the lighter leather and only the sole and heel take the dirt step.
    # box 0 is the sole and 7 the heel; the chamfer split the body and shaft into 1-4
    "r17_boot.R": ("wood", {0: "dirt", 7: "dirt"}),
    "r17_boot.L": ("wood", {0: "dirt", 7: "dirt"}),
    "r17_pack": ("wood", {2: "metal", 3: "metal", 4: "metal"}),   # the bedroll
    "r17_strap.R": ("trunk", {}),
    "r17_strap.L": ("trunk", {}),
    # pickaxe: box 0 is the haft and 3 its grip band; 1, 2 and the six arm boxes are the head
    "r17_pickaxe": ("wood", {1: "metal", 2: "metal", 3: "trunk",
                             4: "metal", 5: "metal", 6: "metal",
                             7: "metal", 8: "metal", 9: "metal"}),
    # lantern: box 3 is the lit glass, 4-5 the base, 6-9 the corner posts
    "r17_lantern": ("metal", {3: "flame", 4: "trunk", 5: "trunk"}),
}

# r3's approved value steps, reused rather than invented: dirt for soles, skinhi for the
# proud brow ridge and nose block
# skinhi is ONE value step off the skin, not a hue change. r3's #F7CE94 is a warm gold and at
# full face width it painted an orange T across the brow ridge and nose.
EXTRA = {
    "dirt": "#493E32", "skinhi": "#F2DFCB",
    # The beard was one cell and read as a black mass; f104's is two or three browns with the
    # lit top clearly lighter than the hanging bottom. Both are r3's recorded value steps.
    "beardhi": "#826145", "beardlo": "#423123",
    # And so was the hair. #34271C is the approved hair cell, but used as the BASE it ranges
    # #3D2E21 down to #221A12 through the orientation steps -- black in everything but name,
    # which is what a warm-key render exposed. Used as the shadow step under #513C2B (also
    # r3's) it reads as the reference's dark warm brown and still separates from the beard.
    # The model sheet sanctions exactly this: "build on the 10 approved cells, and propose
    # which of the 13 value steps earn the spare slots as the parts that need them arrive."
    "hairhi": "#513C2B",
}


def hex_rgb(h, mul=1.0):
    """Hex -> the 0..1 floats that write these exact BYTES into the 8-bit atlas.

    There is deliberately no sRGB->linear transform here. `make_image` creates the image
    WITHOUT `float_buffer=True`, so its `pixels` are already display-encoded and `pack()`
    writes those bytes straight into the PNG; glTF then reads baseColorTexture as sRGB.
    Putting the approved hexes through `srgb_to_linear` first double-encodes them and the
    game draws the whole figure dark and oversaturated -- issue #94. Round 16 fixed that in
    `paint_r16.py`; this script was forked from r14 and had inherited the defect.
    """
    h = h.lstrip("#")
    out = []
    for i in (0, 2, 4):
        v = int(h[i:i + 2], 16) / 255.0 * mul
        out.append(min(1.0, v))
    return out


def cell_table():
    """key -> (cell column, cell row). Rows 0-4 are reserved for the face island."""
    keys = []
    for name in list(PALETTE) + list(EXTRA):
        for step in STEPS:
            keys.append("%s.%s" % (name, step))
    table, col, row = {}, 0, 5
    for k in keys:
        table[k] = (col, row)
        col += 1
        if col >= TEX_SIZE // CELL:
            col, row = 0, row + 1
    return table


CELLS = cell_table()


def orient(normal):
    """Which value step a face takes, from its normal alone.

    A fifth class, "corner", is added for the diagonal planes prism8 now produces.
    Without it every wedge face falls through to "side" and paints identically to
    the side it was cut from -- the whole third plane would be invisible and the
    change would cost faces and buy nothing on screen.
    """
    if normal.z > 0.7:
        return "top"
    if normal.z < -0.7:
        return "bottom"
    if abs(normal.x) > 0.42 and abs(normal.y) > 0.42:
        return "corner"
    if abs(normal.y) > 0.7:
        return "front"
    return "side"


def make_image():
    img = bpy.data.images.get(TEX)
    if img:
        bpy.data.images.remove(img)
    img = bpy.data.images.new(TEX, TEX_SIZE, TEX_SIZE, alpha=False)
    px = [0.0, 0.0, 0.0, 1.0] * (TEX_SIZE * TEX_SIZE)

    def put(x, y, rgb):
        if 0 <= x < TEX_SIZE and 0 <= y < TEX_SIZE:
            i = (y * TEX_SIZE + x) * 4
            px[i], px[i + 1], px[i + 2], px[i + 3] = rgb[0], rgb[1], rgb[2], 1.0

    def rect(x0, y0, w, hgt, rgb):
        for y in range(int(y0), int(y0 + hgt)):
            for x in range(int(x0), int(x0 + w)):
                put(x, y, rgb)

    # the palette cells
    base = dict(PALETTE)
    base.update(EXTRA)
    for key, (col, row) in CELLS.items():
        name, step = key.rsplit(".", 1)
        rect(col * CELL, row * CELL, CELL, CELL, hex_rgb(base[name], STEPS[step]))

    # ---- the face island, painted in WORLD coordinates so the eyes cannot drift ----
    ix, iy, iw, ih = FACE_ISLAND
    wx0, wx1, wz0, wz1 = FACE_RECT
    rect(ix, iy, iw, ih, hex_rgb(PALETTE["skin"]))

    def wpx(x, z):
        return (ix + (x - wx0) / (wx1 - wx0) * iw, iy + (z - wz0) / (wz1 - wz0) * ih)

    def wrect(x0, x1, z0, z1, rgb):
        a, b = wpx(x0, z0)
        c, d = wpx(x1, z1)
        rect(round(a), round(b), max(1, round(c - a)), max(1, round(d - b)), rgb)

    dark = hex_rgb(PALETTE["hair"])
    white = hex_rgb(PALETTE["snow"])
    cheek = hex_rgb("#DDC0A4")
    # heavy brows, sitting just above the eyes -- the 15 mm brow RIDGE is geometry and sits
    # higher; these are the painted brows f104 reads at 60 px
    wrect(-0.136, -0.040, 0.996, 1.026, dark)
    wrect(0.040, 0.136, 0.996, 1.026, dark)
    # eyes on the sheet's eye line (0.807 H = 0.9684 m), big and dark, one white highlight
    wrect(-0.128, -0.046, 0.952, 0.988, dark)
    wrect(0.046, 0.128, 0.952, 0.988, dark)
    wrect(-0.116, -0.092, 0.972, 0.986, white)
    wrect(0.058, 0.082, 0.972, 0.986, white)
    # Cheek warmth, wide and low against the jaw. The first pass put two small high-contrast
    # patches at the temples and they read as stickers rather than as warmth, so this is a
    # single skin value step, one cell apart from the base.
    wrect(-0.196, -0.104, 0.888, 0.948, cheek)
    wrect(0.104, 0.196, 0.888, 0.948, cheek)
    # mouth line, below the moustache
    wrect(-0.058, 0.058, 0.862, 0.876, hex_rgb(PALETTE["beard"], 0.70))
    # Shading the flat face has to do here what geometry is forbidden to (sec.4: no eye
    # sockets, no cheekbones): a shadow under the brow across the eye band, and a soft one
    # down each side of the nose. f104 reads both clearly at 60 px.
    shade = hex_rgb("#D8BCA0")
    wrect(-0.150, -0.128, 0.952, 0.996, shade)
    wrect(0.128, 0.150, 0.952, 0.996, shade)
    wrect(-0.040, -0.026, 0.900, 0.996, shade)
    wrect(0.026, 0.040, 0.900, 0.996, shade)
    # and the jaw's own shadow where the beard mass begins
    wrect(-0.170, 0.170, 0.848, 0.862, hex_rgb("#C9AA8D"))

    img.pixels = px
    img.pack()
    return img


def paint_material(img):
    mat = bpy.data.materials.get(MAT)
    if mat:
        bpy.data.materials.remove(mat)
    mat = bpy.data.materials.new(MAT)
    mat.use_nodes = True
    mat.use_backface_culling = True
    nt = mat.node_tree
    bsdf = nt.nodes.get("Principled BSDF")
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = img
    tex.interpolation = 'Closest'
    tex.location = (-320, 0)
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    for slot in ("Specular IOR Level", "Specular"):
        if slot in bsdf.inputs:
            bsdf.inputs[slot].default_value = 0.5
            break
    if "Roughness" in bsdf.inputs:
        bsdf.inputs["Roughness"].default_value = 0.85
    return mat


def assign_uvs(coll):
    """One texel per flat face; a planar island for the face front. No smart_project."""
    ix, iy, iw, ih = FACE_ISLAND
    wx0, wx1, wz0, wz1 = FACE_RECT
    for ob in coll.objects:
        if ob.type != 'MESH':
            continue
        key, over = PAINT.get(ob.name, ("metal", {}))
        me = ob.data
        uv = me.uv_layers.get("UVMap") or me.uv_layers.new(name="UVMap")
        for poly in me.polygons:
            bi = poly.index // 6
            mat_key = over.get(bi, key)
            # Everything that forms the front of the face takes the island: skull, jaw, brow
            # band, the two brow masses and the face plate. A proud box left on a flat
            # palette cell covers the paint behind it -- that is how the brows went missing.
            face_island = (ob.name == "r17_head" and bi in FACE_BOXES
                           and poly.normal.y > 0.7)
            for li in poly.loop_indices:
                if face_island:
                    co = ob.matrix_world @ me.vertices[me.loops[li].vertex_index].co
                    u = (ix + (co.x - wx0) / (wx1 - wx0) * iw) / TEX_SIZE
                    v = (iy + (co.z - wz0) / (wz1 - wz0) * ih) / TEX_SIZE
                else:
                    col, row = CELLS["%s.%s" % (mat_key, orient(poly.normal))]
                    u = (col * CELL + CELL / 2) / TEX_SIZE
                    v = (row * CELL + CELL / 2) / TEX_SIZE
                uv.data[li].uv = (u, v)


def paint(coll):
    img = make_image()
    mat = paint_material(img)
    for ob in coll.objects:
        if ob.type != 'MESH':
            continue
        ob.data.materials.clear()
        ob.data.materials.append(mat)
    assign_uvs(coll)
    print("PAINT %s -- %s %dx%d packed, %d cells, face island %dx%d px" %
          (REV, TEX, TEX_SIZE, TEX_SIZE, len(CELLS), FACE_ISLAND[2], FACE_ISLAND[3]))
    return mat


def blockout_material():
    name = "M_VoxelDwarf_r14_blockout"
    mat = bpy.data.materials.get(name)
    if mat is None:
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        if bsdf:
            bsdf.inputs["Base Color"].default_value = (0.55, 0.55, 0.56, 1.0)
            if "Roughness" in bsdf.inputs:
                bsdf.inputs["Roughness"].default_value = 0.85
    return mat


# ================================================================== stage C: rig
#
# The 19-joint contract is fixed and is not this file's to negotiate:
#   root hips spine chest neck head shoulder/elbow/hand.L/R hip/knee/foot.L/R beard
#
# Weights are RIGID -- one joint per vertex at 1.0 -- and assigned BY BOX, not by object.
# Per-object would be coarser than the figure needs in three places: the neck box lives
# inside r14_head but belongs to `neck`, the torso's waist belongs to `spine` while its chest
# belongs to `chest`, and the sleeve's forearm belongs to `elbow` while its cap and upper arm
# belong to `shoulder`. Every box still goes wholly to one joint, so no box ever shears.

ARM = "arm"          # marker: resolved against the live arm axis in rig_joints()

RIG = [
    # name, parent, head, tail
    ("root", None, (0.0, 0.0, 0.0), (0.0, 0.0, 0.060)),
    ("hips", "root", (0.0, 0.0, 0.420), (0.0, 0.0, 0.560)),
    ("spine", "hips", (0.0, 0.0, 0.560), (0.0, 0.0, 0.700)),
    ("chest", "spine", (0.0, 0.0, 0.700), (0.0, 0.0, 0.840)),
    ("neck", "chest", (0.0, -0.030, 0.840), (0.0, -0.030, 0.950)),
    ("head", "neck", (0.0, -0.030, 0.950), (0.0, -0.030, 1.200)),
    ("beard", "head", (0.0, 0.090, 0.940), (0.0, 0.140, 0.600)),
]

WEIGHTS = {
    # object: {box index: joint}, with "*" as the object's default
    "r17_head": {"*": "head", 9: "neck"},      # box 9 is the neck column
    "r17_hair": {"*": "head"},
    "r17_moustache": {"*": "head"},
    "r17_beard": {"*": "beard"},
    "r17_torso": {"*": "chest", 2: "spine"},
    "r17_skirt": {"*": "hips"},
    "r17_belt": {"*": "hips"},
    "r17_buckle": {"*": "hips"},
    "r17_pack": {"*": "chest"},
    "r17_strap.R": {"*": "chest"},
    "r17_strap.L": {"*": "chest"},
    # box 2 is the forearm and box 3 the cuff band that sits over the elbow junction
    "r17_sleeve.R": {"*": "shoulder.R", 2: "elbow.R", 3: "elbow.R"},
    "r17_sleeve.L": {"*": "shoulder.L", 2: "elbow.L", 3: "elbow.L"},
    "r17_glove.R": {"*": "hand.R"},
    "r17_glove.L": {"*": "hand.L"},
    "r17_leg.R": {"*": "knee.R", 1: "hip.R"},
    "r17_leg.L": {"*": "knee.L", 1: "hip.L"},
    "r17_boot.R": {"*": "foot.R"},
    "r17_boot.L": {"*": "foot.L"},
    "r17_pickaxe": {"*": "hand.R"},
    "r17_lantern": {"*": "hand.L"},
}


def rig_joints():
    """The full 19, with the arm chain placed on the live A-pose axis."""
    p = P
    a = math.radians(p["arm_deg"])
    joints = list(RIG)
    for side, s in (("R", 1), ("L", -1)):
        def at(t):
            return (s * (p["arm_x"] + math.sin(a) * t), 0.0, p["cap_top"] - math.cos(a) * t)
        joints += [
            ("shoulder." + side, "chest", at(0.040), at(p["arm_cuff_t"])),
            ("elbow." + side, "shoulder." + side, at(p["arm_cuff_t"]),
             at(p["arm_len"] - p["hand_len"])),
            ("hand." + side, "elbow." + side, at(p["arm_len"] - p["hand_len"]), at(p["arm_len"])),
            ("hip." + side, "hips", (s * 0.1404, 0.0, 0.330), (s * 0.1404, 0.0, 0.262)),
            ("knee." + side, "hip." + side, (s * 0.1404, 0.0, 0.262), (s * 0.1404, 0.0, 0.130)),
            ("foot." + side, "knee." + side, (s * 0.1404, 0.0, 0.130), (s * 0.1404, 0.110, 0.060)),
        ]
    return joints


def build_rig(coll):
    name = "SK_VoxelDwarf_Miner01_" + REV
    old = bpy.data.objects.get(name)
    if old:
        arm_data = old.data
        bpy.data.objects.remove(old, do_unlink=True)
        if arm_data.users == 0:
            bpy.data.armatures.remove(arm_data)

    arm = bpy.data.armatures.new(name)
    ob = bpy.data.objects.new(name, arm)
    coll.objects.link(ob)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.object.mode_set(mode='EDIT')
    joints = rig_joints()
    for jname, parent, head, tail in joints:
        b = arm.edit_bones.new(jname)
        b.head, b.tail = head, tail
        b.use_connect = False
    for jname, parent, _, _ in joints:
        if parent:
            arm.edit_bones[jname].parent = arm.edit_bones[parent]
    bpy.ops.object.mode_set(mode='OBJECT')

    # rigid weights, one joint per vertex at 1.0, a whole box at a time
    for mesh_ob in coll.objects:
        if mesh_ob.type != 'MESH':
            continue
        spec = WEIGHTS.get(mesh_ob.name)
        if spec is None:
            raise SystemExit("rig: no weight rule for %s" % mesh_ob.name)
        mesh_ob.vertex_groups.clear()
        groups = {}
        nverts = len(mesh_ob.data.vertices)
        starts = list(mesh_ob.get("box_starts", []))
        if not starts:
            raise SystemExit("rig: %s has no box_starts" % mesh_ob.name)
        ends = starts[1:] + [nverts]
        for bi, (a, b) in enumerate(zip(starts, ends)):
            jname = spec.get(bi, spec["*"])
            if jname not in groups:
                groups[jname] = mesh_ob.vertex_groups.new(name=jname)
            groups[jname].add(list(range(a, b)), 1.0, 'REPLACE')
        if mesh_ob.parent is None:
            mesh_ob.parent = ob
            mesh_ob.matrix_parent_inverse = ob.matrix_world.inverted()
        if not any(m.type == 'ARMATURE' for m in mesh_ob.modifiers):
            mod = mesh_ob.modifiers.new("Armature", 'ARMATURE')
            mod.object = ob

    print("RIG %s -- %s, %d joints, rigid weights by box" % (REV, name, len(joints)))
    return ob


def build(stage="A"):
    """Delete collection SM_VoxelDwarf_Miner01_r14 if present and rebuild it from scratch.

    stage "A" gives every part the grey blockout material; stage "B" paints the atlas.
    """
    global P
    P = derive()

    old = bpy.data.collections.get(COLL)
    if old is not None:
        for ob in list(old.objects):
            data, kind = ob.data, ob.type
            bpy.data.objects.remove(ob, do_unlink=True)
            if data and data.users == 0:
                # once the rig exists the collection is not all meshes, and the datablock has
                # to go back to the collection it came from
                (bpy.data.armatures if kind == 'ARMATURE' else bpy.data.meshes).remove(data)
        bpy.data.collections.remove(old)

    coll = bpy.data.collections.new(COLL)
    bpy.context.scene.collection.children.link(coll)
    mat = blockout_material()

    tris = 0
    for name, fn in PART_BUILDERS:
        boxes = fn()
        me = mesh_from(name, boxes)
        ob = bpy.data.objects.new(name, me)
        # Record where each BOX starts. The weight loop used to assume eight verts
        # per box and step through the mesh in blocks of eight, which was true while
        # every mass was a plain cuboid. A chamfered prism carries eighteen, so that
        # assumption silently mis-weights the figure -- box 9 of the head is the neck
        # column, box 2 of the torso the waist, boxes 2-3 of the sleeve the forearm,
        # and every one of them would land on arbitrary vertices. It would only show
        # when posed. Carrying the real offsets makes the box indices mean what
        # WEIGHTS says they mean, whatever shape the box is.
        starts, acc = [], 0
        for (bv, _bf) in boxes:
            starts.append(acc)
            acc += len(bv)
        ob["box_starts"] = starts
        ob["box_count"] = len(starts)
        ob.data.materials.append(mat)
        coll.objects.link(ob)
        tris += sum(len(poly.vertices) - 2 for poly in me.polygons)

    if stage in ("B", "C"):
        paint(coll)
    if stage == "C":
        build_rig(coll)
    report(tris)
    return coll


def report(tris):
    coll = bpy.data.collections[COLL]
    print("BUILD %s -- %d parts, %d triangles" % (REV, len(coll.objects), tris))
    meshes = [o for o in coll.objects if o.type == 'MESH']
    lo = [1e9] * 3
    hi = [-1e9] * 3
    for ob in meshes:
        for v in ob.data.vertices:
            w = ob.matrix_world @ v.co
            for i in range(3):
                lo[i] = min(lo[i], w[i])
                hi[i] = max(hi[i], w[i])
    print("  figure bounds  x %.4f..%.4f  y %.4f..%.4f  z %.4f..%.4f" %
          (lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]))
    print("  height %.4f m (target %.4f)   width %.4f   depth %.4f" %
          (hi[2] - lo[2], H, hi[0] - lo[0], hi[1] - lo[1]))
    for ob in sorted(meshes, key=lambda o: o.name):
        vs = [ob.matrix_world @ v.co for v in ob.data.vertices]
        print("    %-14s %3d verts %3d faces x %7.4f..%7.4f y %7.4f..%7.4f z %7.4f..%7.4f" %
              (ob.name, len(vs), len(ob.data.polygons),
               min(v[0] for v in vs), max(v[0] for v in vs),
               min(v[1] for v in vs), max(v[1] for v in vs),
               min(v[2] for v in vs), max(v[2] for v in vs)))
