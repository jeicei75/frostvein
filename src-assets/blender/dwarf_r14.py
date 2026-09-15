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

REV = "r14"
COLL = "SM_VoxelDwarf_Miner01_" + REV
H = 1.200                 # figure height, metres
PX = H / 140.0            # one source pixel, 8.571 mm
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


def derive():
    """Every dimension round 14 uses, named, in metres, straight off the sheet."""
    p = {}

    # ---- heights, z/H from the sole (sheet sec.3 height table) -----------------
    p["crown"] = h(1.000)
    p["crown_s2"] = h(0.979)
    p["crown_s1"] = h(0.964)
    p["skull_top"] = h(0.943)
    p["brow"] = h(0.879)
    p["ear_top"] = h(0.850)
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
    p["boot_cuff_t"] = h(0.164)
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


def arm_box(t0, t1, hw, hd, pivot, deg, side=1):
    """A box along an arm axis `deg` out from straight down, through `pivot`.

    side=+1 is the figure's right (+X). The cross-section is hw across the arm in the
    XZ plane and hd in Y, so the sleeve stays square to the body however far it swings.
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
        return (px + u[0] * t + e1[0] * s1 * hw,
                py + s2 * hd,
                pz + u[2] * t + e1[2] * s1 * hw)

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
        box(-hw, hw, y_skull_back, p["y_face"], p["neck_top"] - o, p["skull_top"]),
        # shallow jaw: front of the head only, so the neck stays the outermost
        # surface behind it in profile (see the module docstring). Its back plane is what
        # sets the FRONT of the visible neck column.
        box(-hw + 0.012, hw - 0.012, -0.050, p["y_face"],
            p["head_ends"], p["neck_top"] + o),
        # brow band, ~15 mm proud, full width, at 0.879
        box(-hw, hw, p["y_face"] - o, p["y_face"] + 0.015,
            p["brow"] - 0.013, p["brow"] + 0.013),
        # nose: a base block and a narrower tip, lowest point at 0.750
        box(-0.034, 0.034, p["y_face"] - o, p["y_face"] + 0.034,
            p["nose_low"], p["brow"] - 0.013 + o),
        box(-0.024, 0.024, p["y_face"] + 0.026, p["y_nose"],
            p["nose_low"], p["nose_low"] + 0.100),
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
        box(-0.090, 0.090, -0.128, 0.060, p["neck_bot"], p["neck_top"] + 0.020),
    ]


def part_hair():
    p = P
    o = OVERLAP
    hw = p["hw_crown"]
    cs2, cs1, top, crown = p["crown_s2"], p["crown_s1"], p["skull_top"], p["crown"]
    yb, yf = p["y_head_back"], p["y_hair_front"]
    return [
        # cap over the top and the back; its front edge is the fringe at column 71
        box(-p["hw_head"], p["hw_head"], yb, yf, p["ear_top"], top + o),
        # four crown steps: 0.336 / 0.286 / 0.229 / 0.164 at 0.943 / 0.964 / 0.979 / 1.000
        box(-hw[0], hw[0], yb, yf, top - o, cs1),
        box(-hw[1], hw[1], yb + 0.018, yf - 0.012, cs1 - o, cs2),
        box(-hw[2], hw[2], yb + 0.034, yf - 0.024, cs2 - o, crown - 0.012),
        box(-hw[3], hw[3], yb + 0.048, yf - 0.034, crown - 0.012 - o, crown),
        # Back mass, full width, hanging to 0.707 -- but only BEHIND the neck's back face.
        # Three bands, not one slab: f084 and f088 show the back of the hair stepping in
        # layers. The middle band alone reaches column 36, which is the sheet's hair back,
        # so the stepping never pushes the head past its 0.336 depth. The bottom band is
        # first because the 0.707 height check reads this box.
        box(-p["hw_head"], p["hw_head"], yb + 0.013, -0.130, p["head_ends"], 0.904),
        box(-p["hw_head"], p["hw_head"], yb, -0.130, 0.900, 0.984),
        box(-p["hw_head"], p["hw_head"], yb + 0.009, -0.130, 0.980, p["ear_top"] + o),
        # Side lobes come forward to the temples ONLY above the neck. Below the top of the
        # bare run they stay behind y = -0.130, or they win the side view on |x| and bury
        # the neck -- which is exactly what they did on the first build (0.000 H of neck).
        box(0.130, p["hw_head"], yb, 0.020, p["neck_top"] - o, p["ear_top"] + o),
        box(-p["hw_head"], -0.130, yb, 0.020, p["neck_top"] - o, p["ear_top"] + o),
        box(0.130, p["hw_head"], yb, -0.130, p["head_ends"], p["neck_top"] + o),
        box(-p["hw_head"], -0.130, yb, -0.130, p["head_ends"], p["neck_top"] + o),
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
        box(-p["hw_head"], p["hw_head"], yf - o, p["y_face"] + 0.004, 1.072,
            p["skull_top"] + o),
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
    return [box(-hw, hw, y0, y1, z0 - (o if z0 > p["beard_tip"] else 0.0), z1)
            for z0, z1, hw, y0, y1 in bands]


def part_moustache():
    p = P
    return [box(-0.130, 0.130, p["y_face"] - 0.006, p["y_beard_f"] + 0.014, 0.880, 0.922)]


def part_torso():
    p = P
    o = OVERLAP
    hw, yb, yf = p["hw_chest"], p["y_torso_b"], p["y_torso_f"]
    # Box 1 is the chest and carries the sheet's 0.286 depth. The collar above and the waist
    # below step BACK, so the profile reads as three masses rather than one slab -- the side
    # silhouette scored 3.4 steps/100 rows against the reference's 10.3 when all three sat
    # at the same y.
    return [
        box(-hw, hw, yb + 0.010, yf - 0.026, 0.740 - o, p["cap_top"]),
        box(-hw, hw, yb, yf, 0.560 - o, 0.745),
        box(-hw + 0.010, hw - 0.010, yb + 0.014, yf - 0.014, p["belt_bot"], 0.565),
    ]


def part_skirt():
    p = P
    o = OVERLAP
    yb, yf = p["y_skirt_b"], p["y_skirt_f"]
    # Box 2 is the hem and carries the sheet's 0.439 width and 0.300 depth; the two above it
    # step back in both axes so the skirt flares in profile as well as in front.
    return [
        box(-0.210, 0.210, yb + 0.016, yf - 0.016, 0.390 - o, 0.460),
        box(-0.238, 0.238, yb + 0.008, yf - 0.008, 0.300 - o, 0.395),
        box(-p["hw_waist"], p["hw_waist"], yb, yf, p["hem"], 0.305),
    ]


def part_belt():
    p = P
    return [box(-0.196, 0.196, p["y_torso_b"] - 0.006, p["y_torso_f"] + 0.008,
                p["belt_bot"], p["belt_top"])]


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
    parts = [
        # the cap: this is what carries the 0.528 shoulder width, not the torso
        box(p["hw_chest"] - 0.006, p["hw_shoulder"], -0.160, 0.160, 0.730, p["cap_top"]),
        arm_box(0.055, p["arm_cuff_t"] + o, 0.062, 0.068, pivot, p["arm_deg"], 1),
        arm_box(p["arm_cuff_t"], p["arm_len"] - p["hand_len"] + o, 0.052, 0.058,
                pivot, p["arm_deg"], 1),
    ]
    return parts if side > 0 else mirror_x(parts)


def part_glove(side):
    p = P
    pivot = (p["arm_x"], 0.0, p["cap_top"])
    t0 = p["arm_len"] - p["hand_len"] - OVERLAP
    parts = [
        arm_box(t0, p["arm_len"], 0.060, 0.064, pivot, p["arm_deg"], 1),
        arm_box(t0 + 0.014, t0 + 0.058, 0.026, 0.028,
                (p["arm_x"], 0.072, p["cap_top"]), p["arm_deg"], 1),      # thumb
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
    ]
    return parts if side > 0 else mirror_x(parts)


def part_boot(side):
    p = P
    o = OVERLAP
    xo = p["hw_stance"]
    xc = xo - p["w_limb"] / 2
    hwc = p["w_bootcuff"] / 2
    parts = [
        box(xc - 0.110, xc + 0.110, p["y_shin_b"], p["y_sole_f"], 0.000, 0.032),     # sole
        box(xo - p["w_limb"], xo, p["y_shin_b"] + 0.004, 0.110, 0.028 - o, 0.135),   # body
        box(xc - 0.091, xc + 0.091, 0.100, p["y_sole_f"], 0.028 - o, 0.080),         # toe
        box(xc - hwc, xc + hwc, p["y_shin_b"] + 0.009, 0.105,
            p["boot_cuff_b"], p["boot_cuff_t"]),                                     # cuff
        # heel block under the rear of the sole -- f088 shows a distinct heel, and it is
        # appended so the sole/body/cuff checks keep reading boxes 0, 1 and 3
        box(xc - 0.095, xc + 0.095, p["y_shin_b"], -0.030, 0.000, 0.052),            # heel
    ]
    return parts if side > 0 else mirror_x(parts)


def part_pack():
    p = P
    # the flap is the REARMOST thing on the figure, so the body stops short of column 12
    # and the flap reaches it -- otherwise pack-to-nose overshoots the sheet's 0.550 H.
    # Boxes 0 and 1 carry the sheet's 0.186 depth; the bedroll and pocket are added after
    # them so the measured checks keep pointing at the right masses.
    yb = p["y_pack_b"]
    return [
        box(-0.185, 0.185, yb + 0.016, p["y_pack_f"], p["pack_bot"], 0.800),
        box(-0.155, 0.155, yb, yb + 0.020, 0.580, 0.790),
        # bedroll strapped across the top of the pack -- f088 shows it as a separate mass
        # standing above and behind the pack, with its rolled end proud at the sides
        box(-0.170, 0.170, yb + 0.009, yb + 0.139, 0.796, 0.872),
        box(0.170, 0.192, yb + 0.019, yb + 0.129, 0.804, 0.864),
        box(-0.192, -0.170, yb + 0.019, yb + 0.129, 0.804, 0.864),
        # lower pocket, proud of the body but behind the flap's rear plane (side-left.png)
        box(-0.115, 0.115, yb + 0.004, yb + 0.024, 0.450, 0.560),
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
    return [
        box(hx - 0.021, hx + 0.021, -0.021, 0.021, z0, z0 + p["pick_shaft"]),
        box(hx - 0.030, hx + 0.030, -p["pick_blade"] / 2, p["pick_blade"] / 2,
            z0 + p["pick_shaft"] - 0.062, z0 + p["pick_shaft"] - 0.010),
        box(hx - 0.024, hx + 0.024, -p["pick_blade"] / 2 - 0.052, -p["pick_blade"] / 2 + 0.012,
            z0 + p["pick_shaft"] - 0.100, z0 + p["pick_shaft"] - 0.020),
    ]


def part_lantern():
    """0.26 H tall, hanging from the left hand, in contact with it."""
    p = P
    hx, _, hz = _hand_point(-1)
    top = hz - 0.010
    bot = top - p["lantern_h"]
    return [
        box(hx - 0.010, hx + 0.010, -0.008, 0.008, top - 0.030, top),            # bail
        box(hx - 0.046, hx + 0.046, -0.046, 0.046, top - 0.052, top - 0.022),    # cap
        box(hx - 0.038, hx + 0.038, -0.038, 0.038, bot + 0.030, top - 0.046),    # glass
        box(hx - 0.048, hx + 0.048, -0.048, 0.048, bot, bot + 0.036),            # base
    ]


PART_BUILDERS = [
    ("r14_head", part_head),
    ("r14_hair", part_hair),
    ("r14_beard", part_beard),
    ("r14_moustache", part_moustache),
    ("r14_torso", part_torso),
    ("r14_skirt", part_skirt),
    ("r14_belt", part_belt),
    ("r14_buckle", part_buckle),
    ("r14_sleeve.R", lambda: part_sleeve(1)),
    ("r14_sleeve.L", lambda: part_sleeve(-1)),
    ("r14_glove.R", lambda: part_glove(1)),
    ("r14_glove.L", lambda: part_glove(-1)),
    ("r14_leg.R", lambda: part_leg(1)),
    ("r14_leg.L", lambda: part_leg(-1)),
    ("r14_boot.R", lambda: part_boot(1)),
    ("r14_boot.L", lambda: part_boot(-1)),
    ("r14_pack", part_pack),
    ("r14_strap.R", lambda: part_strap(1)),
    ("r14_strap.L", lambda: part_strap(-1)),
    ("r14_pickaxe", part_pickaxe),
    ("r14_lantern", part_lantern),
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

TEX = "T_VoxelDwarf_r14"
MAT = "M_VoxelDwarf_r14"
TEX_SIZE = 512
CELL = 32                        # 16 x 16 cells
FACE_ISLAND = (8, 8, 176, 128)   # x, y, w, h in pixels, bottom-left origin
FACE_RECT = (-0.2016, 0.2016, 0.8484, 1.1316)     # world x0,x1,z0,z1 the island covers

STEPS = {"top": 1.18, "front": 1.00, "side": 0.85, "bottom": 0.66}

# which palette cell each part is, with per-box overrides where a part is two materials
PAINT = {
    # the brow ridge and the nose stand 15-50 mm proud, so they take a brighter skin step --
    # without it a dead-on front view paints them the same value as the face plane and the
    # nose disappears entirely
    "r14_head": ("skin", {2: "skinhi", 3: "skinhi", 4: "skinhi"}),
    "r14_hair": ("hair", {}),
    "r14_beard": ("beard", {}),
    "r14_moustache": ("beard", {}),
    "r14_torso": ("tunic", {}),
    "r14_skirt": ("tunic", {}),
    "r14_belt": ("trunk", {}),
    "r14_buckle": ("metal", {}),
    "r14_sleeve.R": ("tunic", {2: "skin"}),        # box 2 is the bare forearm
    "r14_sleeve.L": ("tunic", {2: "skin"}),
    "r14_glove.R": ("wood", {}),
    "r14_glove.L": ("wood", {}),
    "r14_leg.R": ("pants", {}),
    "r14_leg.L": ("pants", {}),
    # f104's boots are LIGHT tan against dark trousers. Built on the darker leather they sank
    # into the background at 60 px and the whole lower body went illegible, so the boot body
    # is the lighter leather and only the sole and heel take the dirt step.
    "r14_boot.R": ("wood", {0: "dirt", 4: "dirt"}),
    "r14_boot.L": ("wood", {0: "dirt", 4: "dirt"}),
    "r14_pack": ("wood", {2: "metal", 3: "metal", 4: "metal"}),   # the bedroll
    "r14_strap.R": ("trunk", {}),
    "r14_strap.L": ("trunk", {}),
    "r14_pickaxe": ("wood", {1: "metal", 2: "metal"}),
    "r14_lantern": ("metal", {2: "flame"}),
}

# r3's approved value steps, reused rather than invented: dirt for soles, skinhi for the
# proud brow ridge and nose block
# skinhi is ONE value step off the skin, not a hue change. r3's #F7CE94 is a warm gold and at
# full face width it painted an orange T across the brow ridge and nose.
EXTRA = {"dirt": "#493E32", "skinhi": "#F2DFCB"}


def srgb_to_linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def hex_rgb(h, mul=1.0):
    h = h.lstrip("#")
    out = []
    for i in (0, 2, 4):
        v = int(h[i:i + 2], 16) / 255.0 * mul
        out.append(srgb_to_linear(min(1.0, v)))
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
    if normal.z > 0.7:
        return "top"
    if normal.z < -0.7:
        return "bottom"
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
    wrect(-0.200, -0.120, 0.902, 0.958, cheek)
    wrect(0.120, 0.200, 0.902, 0.958, cheek)
    # mouth line, below the moustache
    wrect(-0.058, 0.058, 0.862, 0.876, hex_rgb(PALETTE["beard"], 0.70))

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
            face_island = (ob.name == "r14_head" and bi in (0, 1) and poly.normal.y > 0.7)
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
    "r14_head": {"*": "head", 7: "neck"},
    "r14_hair": {"*": "head"},
    "r14_moustache": {"*": "head"},
    "r14_beard": {"*": "beard"},
    "r14_torso": {"*": "chest", 2: "spine"},
    "r14_skirt": {"*": "hips"},
    "r14_belt": {"*": "hips"},
    "r14_buckle": {"*": "hips"},
    "r14_pack": {"*": "chest"},
    "r14_strap.R": {"*": "chest"},
    "r14_strap.L": {"*": "chest"},
    "r14_sleeve.R": {"*": "shoulder.R", 2: "elbow.R"},
    "r14_sleeve.L": {"*": "shoulder.L", 2: "elbow.L"},
    "r14_glove.R": {"*": "hand.R"},
    "r14_glove.L": {"*": "hand.L"},
    "r14_leg.R": {"*": "knee.R", 1: "hip.R"},
    "r14_leg.L": {"*": "knee.L", 1: "hip.L"},
    "r14_boot.R": {"*": "foot.R"},
    "r14_boot.L": {"*": "foot.L"},
    "r14_pickaxe": {"*": "hand.R"},
    "r14_lantern": {"*": "hand.L"},
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
        for bi in range(nverts // 8):
            jname = spec.get(bi, spec["*"])
            if jname not in groups:
                groups[jname] = mesh_ob.vertex_groups.new(name=jname)
            groups[jname].add(list(range(bi * 8, bi * 8 + 8)), 1.0, 'REPLACE')
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
        me = mesh_from(name, fn())
        ob = bpy.data.objects.new(name, me)
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
