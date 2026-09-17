"""Round 16 -- the faceted voxel dwarf. Spend the budget on DIRECTIONS, not faces.

    blender --background --factory-startup --python src-assets/blender/dwarf_r16.py

r16's whole thesis (its s1): across round 14 the cage went 444 -> 1032 faces with the
distinct-normal count stuck at 14 every single time, because an axis-aligned box has
six possible normals however many you stack. So this round buys DIRECTIONS.

ONE generator, TWO figures. FACET=False emits plain axis-aligned boxes and is the
round-15 baseline; FACET=True cuts the corners off the large masses at 45 degrees.
Same PARAMS, same rows, same everything else -- which is what makes the A/B in s7 an
honest comparison rather than two different sculpts.

WHY A PRISM BUILDER AND NOT A BEVEL MODIFIER. r16 s8 forbids bevel, subdivision and
smooth shading. Every mass here is a stack of horizontal RINGS bridged by quads, so
a chamfer is just a ring with eight corners instead of four, and a horizontal wedge
is a ring inset in Z. Flat shading throughout, by construction.

THE SILHOUETTE COST OF A WEDGE, which r16 s6 is anxious about:
  * a VERTICAL corner chamfer costs NOTHING in either pinned view. The side face at
    x=x1 still exists between y0+c and y1-c, so the extreme x is still reached, and
    likewise for y. Both orthographic silhouettes are bit-identical to the box's.
    This is r15 s4.1's chamfer trick, used for its normals rather than its shape.
  * a HORIZONTAL wedge does cut the top or bottom row of a mass, so it is used ONLY
    where the art already draws a ramp. It turns out the art draws three of them:
    the crown ramps 11.5 -> 23.5 px over rows 7-15 (front.png), the pack's top-back
    corner steps one column per row over rows 35-37 and its bottom-back over rows
    86-94 (side-left.png), and the boot's toe steps out between rows 133 and 138.
    Those are FIDELITY, not the stylistic extension r16 s3 owns up to.

Geometry authority is the extraction in sheet_r16.py, never the model sheet's twenty
samples (r15 s2). Every box below cites the rows it came from.
"""

import math
import os
import sys

import bpy
import numpy as np
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

REV = "r16"
FACET = True          # the round's experiment. False == the r15 baseline, same code
H = 1.20              # metres; the figure is 140 source rows tall, 1 px = 8.571 mm
COLLECTION = f"SM_VoxelDwarf_Miner01_{REV}"

PX = 1.0 / 140.0      # one source pixel, in H
FRONT_C = 77.5        # front.png centre column, from the extraction (sheet says 77)
SIDE_C = 58.0         # side-left.png depth centre, +Y forward (model sheet line 77)

# Which palette family each part paints from. Per CHUNK where a part is two
# materials: the pickaxe is a wood haft with a metal head, the lantern a metal
# frame around a lit pane.
PART_FAMILY = {
    "head": "skin", "neck": "skin", "glove": "skin",
    "hair": "hair", "beard": "beard", "moustache": "beard",
    "torso": "tunic", "sleeve": "tunic", "skirt": "tunic",
    "leg": "boot",
    "belt": "trunk", "boot": "boot", "pack": "trunk", "strap": "trunk",
    "buckle": "metal", "pickaxe": "metal", "lantern": "metal",
    "hem": "hem",
}

CHAMFER_MIN = 2.5 * PX     # below this a wedge is a sliver -- round 12's criticism
CHAMFER_MAX = 6.0 * PX
CHAMFER_FRAC = 0.20
# Raised from 3.5 px (~30 mm) to 5.5 px (~47 mm) after looking at the first lit A/B.
# r16 s2's own limit was the right idea set too low: at 30 mm the brow, nose, ears,
# moustache and hair lobes all qualified, and chamfering them did two bad things --
# it made the slivers round 12 was criticised for, and because a chamfer SHRINKS a
# mass at its corners, parts that used to abut opened visible slits at the neck and
# around the face. Free-standing masses gain a third plane; INTERLOCKING ones lose
# their joins. So the rule that actually works is not a size threshold alone:
# anything that seats inside or against another mass stays square (facet=False).
SMALL = 5.5 * PX


# ---------------------------------------------------------------- sheet mapping

def zb(row):
    """z of the BOTTOM edge of a source row. Row 147 is the sole, row 7 the crown."""
    return (147.0 - row) * PX


def band(a, b):
    """(z0, z1) for a mass covering source rows a..b inclusive."""
    return zb(b), zb(a - 1)


def fx(col):
    """front.png column -> x, about the centre line."""
    return (col - FRONT_C) * PX


def fy(col):
    """side-left.png column -> y, +Y forward."""
    return (col - SIDE_C) * PX


def chamfer(w, d, allow=True):
    if not (FACET and allow):
        return 0.0
    m = min(w, d)
    if m < SMALL:
        return 0.0
    return max(CHAMFER_MIN, min(CHAMFER_MAX, CHAMFER_FRAC * m))


# ---------------------------------------------------------------- mesh building

def ring(x0, x1, y0, y1, c):
    """One horizontal cross-section, CCW seen from +Z. 4 corners, or 8 if chamfered."""
    if c <= 1e-9:
        return [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
    return [(x0 + c, y0), (x1 - c, y0), (x1, y0 + c), (x1, y1 - c),
            (x1 - c, y1), (x0 + c, y1), (x0, y1 - c), (x0, y0 + c)]


class Build:
    def __init__(self):
        self.parts = {}
        self.fams = {}          # per CHUNK, aligned with parts[...]: the palette family
        self.cites = []

    def prism(self, part, sections, cite="", fam=None):
        """Stack horizontal rings and bridge them. sections: [(z, x0,x1,y0,y1, c)].

        r15 s7.5 -- check for degenerate boxes. One with z running backwards held the
        crown a step too wide in round 14, and a normals check cannot see it because
        a box with no extent has no outward direction.
        """
        assert cite, f"{part}: r15 s9 -- every box cites a file and a row"
        # Mirrored parts are written with the same expression for both sides, so
        # x0/x1 arrive swapped on one of them. Normalise, but keep the assert: a
        # ZERO-extent box is the r15 s7.5 defect and must still stop the build.
        sections = [(z, min(x0, x1), max(x0, x1), min(y0, y1), max(y0, y1), c)
                    for (z, x0, x1, y0, y1, c) in sections]
        for (z, x0, x1, y0, y1, c) in sections:
            assert x1 - x0 > 1e-6 and y1 - y0 > 1e-6, f"{part}: degenerate in XY {cite}"
        for a, b in zip(sections, sections[1:]):
            assert b[0] - a[0] > 1e-6, f"{part}: z runs backwards {cite}"
        n = len(ring(*sections[0][1:]))
        assert all(len(ring(*s[1:])) == n for s in sections), f"{part}: ring mismatch"

        v, f = [], []
        rings = []
        for (z, x0, x1, y0, y1, c) in sections:
            idx = []
            for (px_, py_) in ring(x0, x1, y0, y1, c):
                idx.append(len(v))
                v.append((px_, py_, z))
            rings.append(idx)
        for lo, hi in zip(rings, rings[1:]):
            for i in range(n):
                j = (i + 1) % n
                f.append([lo[i], lo[j], hi[j], hi[i]])
        # caps, fanned from a centre vertex: the exporter forbids n-gons
        for idx, up in ((rings[0], False), (rings[-1], True)):
            cx = sum(v[i][0] for i in idx) / n
            cy = sum(v[i][1] for i in idx) / n
            cz = v[idx[0]][2]
            ci = len(v)
            v.append((cx, cy, cz))
            for i in range(n):
                j = (i + 1) % n
                f.append([ci, idx[i], idx[j]] if up else [ci, idx[j], idx[i]])
        self.parts.setdefault(part, []).append((v, f))
        self.fams.setdefault(part, []).append(fam or PART_FAMILY.get(part, "skin"))
        self.cites.append((part, cite))

    def box(self, part, a, b, x0, x1, y0, y1, cite="", facet=True, fam=None):
        """A mass spanning source rows a..b. Vertical corner wedge, silhouette-free."""
        z0, z1 = band(a, b)
        c = chamfer(x1 - x0, y1 - y0, facet)
        self.prism(part, [(z0, x0, x1, y0, y1, c), (z1, x0, x1, y0, y1, c)], cite, fam)

    def box_z(self, part, z0, z1, x0, x1, y0, y1, cite="", facet=True, fam=None):
        """Like box(), but positioned by z in H rather than by source row -- the
        props hang off the HAND, whose height the A-pose determines, not off a row
        of the sheet (where the sheet draws them carried across the body instead)."""
        c = chamfer(abs(x1 - x0), abs(y1 - y0), facet)
        self.prism(part, [(z0, x0, x1, y0, y1, c), (z1, x0, x1, y0, y1, c)], cite, fam)

    def ramp(self, part, sections, cite="", facet=True, fam=None):
        """A mass whose footprint changes with height -- the art's own ramps."""
        cs = [chamfer(x1 - x0, y1 - y0, facet) for (_, x0, x1, y0, y1) in sections]
        c = max(cs) if FACET and facet else 0.0
        secs = []
        for (row, x0, x1, y0, y1) in sections:
            c_ = min(c, 0.49 * min(x1 - x0, y1 - y0))
            secs.append((zb(row), x0, x1, y0, y1, c_))
        secs.sort(key=lambda t: t[0])          # written top-down, built bottom-up
        self.prism(part, secs, cite, fam)

    def wedge(self, part, a, b, x0, x1, y0, y1, tc=0.0, bc=0.0, cite="", facet=True, fam=None):
        """A mass with an optional HORIZONTAL wedge on its top and/or bottom ring.

        Only used where the art already draws the ramp (see the module docstring),
        because unlike the vertical chamfer this one does move the silhouette.
        """
        z0, z1 = band(a, b)
        c = chamfer(x1 - x0, y1 - y0, facet)
        if not FACET:
            tc = bc = 0.0
        secs = []
        if bc > 0:
            secs.append((z0, x0 + bc, x1 - bc, y0 + bc, y1 - bc, c))
            secs.append((z0 + bc, x0, x1, y0, y1, c))
        else:
            secs.append((z0, x0, x1, y0, y1, c))
        if tc > 0:
            secs.append((z1 - tc, x0, x1, y0, y1, c))
            secs.append((z1, x0 + tc, x1 - tc, y0 + tc, y1 - tc, c))
        else:
            secs.append((z1, x0, x1, y0, y1, c))
        self.prism(part, secs, cite, fam)

    def to_objects(self, uv_fn=None):
        col = bpy.data.collections.new(COLLECTION)
        bpy.context.scene.collection.children.link(col)
        for part, chunks in self.parts.items():
            V, F, FAM = [], [], []
            for (v, f), fam in zip(chunks, self.fams[part]):
                o = len(V)
                V += [(p[0] * H, p[1] * H, p[2] * H) for p in v]
                F += [[i + o for i in face] for face in f]
                FAM += [fam] * len(f)
            me = bpy.data.meshes.new(f"{REV}_{part}")
            me.from_pydata(V, [], F)
            me.validate()
            for poly in me.polygons:
                poly.use_smooth = False          # r16 s8: flat shading, always
            if uv_fn is not None:
                uv = me.uv_layers.new(name="UVMap")
                for poly, fam in zip(me.polygons, FAM):
                    co = uv_fn(part, fam, me, poly)
                    for k, li in enumerate(poly.loop_indices):
                        uv.data[li].uv = co[k]
            ob = bpy.data.objects.new(f"{REV}_{part}", me)
            col.objects.link(ob)
        return col


# --------------------------------------------------------------------- PARAMS
# Every row cited below is a SOURCE row of the pinned crops, read off the
# extraction in sheet_r16.py (dumped to r16-profile.txt), never off the model
# sheet's twenty samples. hw = half-width in H, about the centre line.

def hw(px_half):
    return px_half * PX


def build():
    for c in list(bpy.data.collections):
        if c.name.startswith("SM_VoxelDwarf_Miner01"):
            for o in list(c.objects):
                bpy.data.objects.remove(o, do_unlink=True)
            bpy.data.collections.remove(c)

    b = Build()

    # ---- crown, six steps. front.png rows 7-14 widen 11.5 -> 20.5 px; side-left
    # rows 7-14 deepen cols 45-77 -> 36-82. r15 s4.2: six or seven steps, from the
    # art's own edge, not four bands. The top step takes a horizontal wedge: the
    # crown is the one mass read from directly above, and rows 7-9 give it room.
    CROWN = [  # (a, b, half_px, side_col0, side_col1)
        (7,  9,  11.5, 45, 77),
        (10, 10, 15.5, 44, 82),
        (11, 11, 16.5, 39, 82),
        (12, 12, 19.5, 39, 82),
        (13, 14, 20.5, 36, 82),
    ]
    for i, (a, bb, h, c0, c1) in enumerate(CROWN):
        b.wedge("hair", a, bb, -hw(h), hw(h), fy(c0), fy(c1),
                tc=(2.0 * PX if i == 0 else 0.0),
                cite=f"front.png rows {a}-{bb} half {h}px; side-left cols {c0}-{c1}")

    # ---- hair shell, rows 15-48. front half 23.5 px flat all the way (0.336 H, the
    # model sheet's head-with-hair, which the extraction confirms to the pixel).
    # The shell is the back and top; the face is left open and the lobes come
    # forward past the ears and down the cheeks in two depth layers (f100).
    b.box("hair", 15, 48, -hw(23.5), hw(23.5), fy(36), fy(66),
          cite="front.png rows 15-48 half 23.5px; side-left rows 14-48 cols 36-82")
    for s in (-1, 1):
        b.box("hair", 15, 47, s * hw(23.5), s * hw(16.0), fy(36), fy(80), facet=False,
              cite="f100 hair lobes forward past the ears; front rows 15-47")
        b.box("hair", 15, 44, s * hw(21.0), s * hw(14.0), fy(66), fy(82), facet=False,
              cite="f100 second, shallower lobe layer down the cheek")

    # ---- skull and face. The face plane sits at side-left col 82 (+0.171 H); the
    # brow steps out to col 84 and the nose ramps to col 89 (rows 31-43).
    b.box("head", 15, 48, -hw(16.5), hw(16.5), fy(40), fy(82), facet=False,
          cite="front rows 15-48 inside the hair; side-left cols 40-82 face plane")
    # brow: TWO masses with the bridge recessed between them (f100), not one bar
    for s in (-1, 1):
        b.box("head", 23, 27, s * hw(4.0), s * hw(15.0), fy(82), fy(84), facet=False,
              cite="f100 brow pair, bridge recessed; side-left rows 23-27 col 84")
    # nose, a RAMP across rows 31-43 (r15 s2: 0.189/0.196/0.210/0.217 H), not a block
    NOSE = [(31, 86), (33, 87), (34, 88), (38, 89), (41, 89), (43, 86)]
    b.ramp("head", [(r, -hw(3.5), hw(3.5), fy(80), fy(c)) for (r, c) in NOSE],
           facet=False, cite="side-left rows 31-43 cols 86-89, the nose ramp")
    # ears, front rows 29-45 step out from 23.5 to 26.5 px (ear top 0.850, bottom 0.729)
    for s in (-1, 1):
        b.box("head", 29, 45, s * hw(23.0), s * hw(26.5), fy(50), fy(66), facet=False,
              cite="front.png rows 29-45, half 23.5 -> 26.5 px: the ears")

    # ---- neck, built at full anatomical size; the sheet's 0.064 H bare column is
    # what the JAW and the HAIR LOBES leave uncovered, not the neck's thickness
    # (r15 s4.3, and the ortho README's note that the neck is bare in both sides).
    b.box("neck", 44, 52, -hw(10.5), hw(10.5), fy(48), fy(74), facet=False,
          cite="r15 s4.3: throat ~0.150 H across, full depth; occluders set the run")

    # ---- beard. Widest 0.343 H at front rows 44-56 -- essentially the head's own
    # width -- tapering to the tip at row 82 (z 0.464). Front edge side-left col 86.
    # The art's front DIPS at rows 43-49, between moustache and the main mass.
    BEARD = [(44, 24.0, 86), (49, 24.0, 86), (57, 23.0, 84), (70, 18.0, 80), (82, 13.0, 76)]
    b.ramp("beard", [(r, -hw(h), hw(h), fy(56), fy(c)) for (r, h, c) in BEARD],
           cite="front rows 44-82 (tip 0.464 H); side-left col 86 beard front")
    b.box("beard", 44, 78, -hw(9.0), hw(9.0), fy(56), fy(88), facet=False,
          cite="f100: centre mass proud of the sides")
    b.box("moustache", 38, 45, -hw(12.0), hw(12.0), fy(82), fy(88), facet=False,
          cite="f100/front rows 38-45, above the beard's dip at rows 43-49")

    # ---- torso and skirt: THE ART NEVER EXPOSES THESE (r15 s2). The posed arms
    # cover them in front, side and back, so these two alone come from the model
    # sheet's table: chest 0.317 H wide, torso depth 0.286 (cols 38-78); skirt
    # 0.440 wide, depth 0.300 (cols 37-79). Their envelope figure is unmeasurable.
    # Split at the waist so CHEST and SPINE each own a whole box. r15 s8 wants rigid
    # weights assigned by BOX, and a single torso box would have to be divided
    # vertex-by-vertex at the waist, which is the shear it warns about for the legs.
    # The two abut, so the silhouette is identical to the one box it replaces.
    b.box("torso", 49, 75, -hw(22.2), hw(22.2), fy(38), fy(78),
          cite="model sheet: chest 0.317 H, torso depth 0.286 -- art does not expose")
    b.box("torso", 76, 101, -hw(22.2), hw(22.2), fy(38), fy(78),
          cite="model sheet: the waist below it, same section; belongs to spine")
    b.box("skirt", 100, 118, -hw(30.8), hw(30.8), fy(37), fy(79),
          cite="front rows 113-118 half 30.5px (0.436 H); hem row 118 = 0.207 H")
    b.box("hem", 112, 118, -hw(31.2), hw(31.2), fy(36), fy(80), fam="hem",
          cite="front.png rows 112-118: the lighter band along the tunic's hem")
    b.box("hem", 49, 55, -hw(19.0), hw(19.0), fy(36), fy(80), fam="hem",
          cite="front.png rows 49-55: the lighter collar around the neckline")
    b.box("belt", 88, 99, -hw(23.0), hw(23.0), fy(36), fy(80), facet=False,
          cite="front rows 88-99 (belt top 0.421, bottom 0.343) -- r16 s2 leaves belts square")
    b.box("buckle", 90, 97, -hw(5.0), hw(5.0), fy(80), fy(82), facet=False,
          cite="front.png buckle, centred on the belt -- square per r16 s2")

    # ---- arms. A-pose for the bind, out ~40 deg (r15 s6). The sheet's hand-bottom
    # at 0.330 H is a POSED measurement, so arm LENGTH comes off it (0.370 H
    # shoulder-to-hand) and arm HEIGHT does not. Shoulders 0.528 H over the caps.
    SH_Z, SH_X, ARM = zb(48), hw(29.6), 0.370
    for s in (-1, 1):
        b.box("sleeve", 49, 56, s * hw(22.2), s * hw(37.0), fy(42), fy(74),
              cite="model sheet shoulders 0.528 H over the caps; front row 49 = 0.521")
        for (part, z0, z1, half, d0, d1, cit) in (
            ("sleeve", SH_Z - 0.20, SH_Z, 8.5, 44, 72, "upper arm, sleeve to cuff 0.566 H"),
            ("sleeve", SH_Z - 0.31, SH_Z - 0.20, 7.5, 46, 70, "forearm, f140 sleeve on shoulder"),
            ("glove", SH_Z - ARM, SH_Z - 0.31, 9.0, 44, 72, "hand bottom 0.330 H, posed"),
        ):
            ax0, ax1 = s * SH_X - hw(half), s * SH_X + hw(half)
            c = chamfer(ax1 - ax0, fy(d1) - fy(d0))
            b.prism(part, [(z0, ax0, ax1, fy(d0), fy(d1), c),
                           (z1, ax0, ax1, fy(d0), fy(d1), c)], cite=cit)
            rotate_chunk(b, part, (s * SH_X, 0.0, SH_Z), "Y", -s * 40.0)

    # ---- legs, split at the knee so hip and knee each own a WHOLE box (r15 s8):
    # a single leg box has to be divided vertex-by-vertex and shears. front.png
    # rows 119-132 give cols 50-73 and 83-106; side-left rows 119-123 give cols 47-70.
    for s, (c0, c1) in ((-1, (50, 73)), (1, (83, 106))):
        b.box("leg", 119, 125, fx(c0), fx(c1), fy(47), fy(70),
              cite=f"front rows 119-125 cols {c0}-{c1} (thigh); side-left cols 47-70")
        b.box("leg", 126, 132, fx(c0), fx(c1), fy(47), fy(70),
              cite=f"front rows 126-132 cols {c0}-{c1} (shin); side-left cols 47-70")

    # ---- boots. Cuff rows 125-132 steps out to 0.207 H (front cols 47-76 / 81-108).
    # The TOE is the art's own horizontal step: side-left rows 133-136 stop at col
    # 70, rows 138-147 reach col 77, so the ramp between them is fidelity (r16 s3).
    for s, (c0, c1, k0, k1) in ((-1, (50, 73, 47, 76)), (1, (83, 106, 81, 108))):
        b.box("boot", 124, 132, fx(k0), fx(k1), fy(44), fy(73), fam="cuff",
              cite=f"front rows 125-132 cols {k0}-{k1}: the boot cuff, a LIGHTER "
                   f"band than the shaft below it in front.png")
        b.ramp("boot", [(133, fx(c0), fx(c1), fy(47), fy(70)),
                        (137, fx(c0), fx(c1), fy(47), fy(72)),
                        (147, fx(c0), fx(c1), fy(47), fy(77))],
               cite="side-left rows 133-147 cols 47-70 -> 47-77: the toe steps out")

    # ---- pack. side-left rows 35-94; the back edge TAPERS, and the taper is drawn
    # one column per row at rows 35-37 and again at 86-94, which is a 45 deg ramp in
    # the source. Deepest col 12 at rows 67-85 (-0.331 H), matching r15 s2's table.
    # The back edge is NOT one long taper. The art holds col 16-17 from row 38 to
    # row 66 and then STEPS to col 12 at row 67; ramping straight from 38 to 67
    # undercut rows 38-66 and was the whole of the side view's +4 px flag.
    b.ramp("pack", [(35, -hw(21.0), hw(21.0), fy(21), fy(38)),
                    (38, -hw(21.0), hw(21.0), fy(17), fy(38)),
                    (66, -hw(21.0), hw(21.0), fy(16), fy(38)),
                    (68, -hw(21.0), hw(21.0), fy(12), fy(38)),
                    (85, -hw(21.0), hw(21.0), fy(12), fy(38)),
                    (94, -hw(21.0), hw(21.0), fy(21), fy(38))],
           cite="side-left rows 35-94: back col 21->17 (35-38), 16 (to 66), "
                "12 (67-85), back to 21 by 94; f088 pack behind the torso")
    for s in (-1, 1):
        b.box("strap", 49, 88, s * hw(8.0), s * hw(14.0), fy(34), fy(80), facet=False,
              cite="back.png over-shoulder straps; r16 s2 leaves straps square")

    # ---- props. THE GATE CANNOT SEE THESE: r15 s3.2 keeps them out of our
    # silhouette, so nothing in the envelope was checking them, and the first build
    # had both hanging in mid-air a full 0.10 H outboard of the hands with a pick
    # head a third of its drawn size. Caught by eye in the viewport, not by a number.
    # The hand is where the 40 deg A-pose PUTS it, so derive it rather than guess:
    # the glove's mid-height is ARM-0.03 below the shoulder, swung out by 40 deg.
    hand_d = ARM - 0.03
    HAND_X = SH_X + hand_d * math.sin(math.radians(40.0))
    HAND_Z = SH_Z - hand_d * math.cos(math.radians(40.0))

    # pickaxe, measured off gear.png AS DRAWN -- the ortho README records the
    # breakdown's own labels as wrong. Length ~1.04 H, head span ~0.54 of that.
    # Hung vertically in a neutral hand (r15 s3.2).
    PX_X, PX_Y = HAND_X, fy(78)
    # Measured as drawn the pick is 1.04 H -- LONGER THAN THE DWARF IS TALL. The
    # sheet gets away with it by carrying it diagonally across the body; hung
    # vertically it has to stick out somewhere, and the first placement put 0.43 H
    # of haft through the floor. Butt on the ground, head just above the crown, the
    # hand gripping the middle -- which is how you actually hold a pick upright.
    haft_top = 1.04
    b.prism("pickaxe", [(haft_top - 1.04, PX_X - 0.013, PX_X + 0.013,
                         PX_Y - 0.013, PX_Y + 0.013, 0.0),
                        (haft_top, PX_X - 0.013, PX_X + 0.013,
                         PX_Y - 0.013, PX_Y + 0.013, 0.0)],
            cite="gear.png: the haft, ~1.04 H long measured as drawn, not labelled",
            fam="wood")
    HEAD_SPAN = 0.54 * 1.04 / 2.0          # gear.png: head span 0.54 of the length
    b.box_z("pickaxe", haft_top - 0.10, haft_top - 0.03, PX_X - 0.030, PX_X + 0.030,
            PX_Y - 0.026, PX_Y + 0.026,
            cite="gear.png: the collar where the head meets the haft")
    for s2 in (-1, 1):
        # both arms sweep OUT and DOWN to points -- four steps, each shorter and
        # lower than the last, which is how the art draws the taper
        for i in range(4):
            o0 = 0.030 + i * (HEAD_SPAN - 0.030) / 4.0
            o1 = 0.030 + (i + 1) * (HEAD_SPAN - 0.030) / 4.0
            t = haft_top - 0.015 - i * 0.030
            hgt = 0.055 - i * 0.011
            b.box_z("pickaxe", t - hgt, t,
                    PX_X + s2 * o0, PX_X + s2 * o1,
                    PX_Y - 0.020 + i * 0.004, PX_Y + 0.020 - i * 0.004,
                    facet=False,
                    cite="gear.png: the pick arm sweeping out and DOWN to a point")

    # lantern: a FRAME -- corner posts, vented cap, footed base -- not a solid block.
    # gear.png draws it at 0.26 H as measured (its own label says 0.33).
    LX, LY = -HAND_X, fy(78)
    top = HAND_Z - 0.02
    b.box_z("lantern", top - 0.030, top, LX - 0.042, LX + 0.042, LY - 0.042, LY + 0.042,
            cite="gear.png: the vented cap")
    for sx in (-1, 1):
        for sy in (-1, 1):
            b.box_z("lantern", top - 0.200, top - 0.030,
                    LX + sx * 0.028, LX + sx * 0.042,
                    LY + sy * 0.028, LY + sy * 0.042, facet=False,
                    cite="gear.png: corner posts of the frame")
    b.box_z("lantern", top - 0.245, top - 0.200, LX - 0.042, LX + 0.042,
            LY - 0.042, LY + 0.042, cite="gear.png: the footed base")
    b.box_z("lantern", top - 0.195, top - 0.035, LX - 0.028, LX + 0.028,
            LY - 0.028, LY + 0.028, cite="gear.png: the glass, inside the frame",
            fam="flame")
    return b


def rotate_chunk(b, part, pivot, axis, deg):
    """Rotate the chunk just added to `part`. r15 s7.3: a rotated box's AABB lies,
    so nothing downstream may measure these with an axis-aligned width probe."""
    v, f = b.parts[part][-1]
    R = Matrix.Rotation(math.radians(deg), 3, axis)
    p = Vector(pivot)
    b.parts[part][-1] = ([tuple(R @ (Vector(q) - p) + p) for q in v], f)


# ------------------------------------------------------------------- the metric
# r16 s4: printed on EVERY build, with an expectation, never a target to pad to.
# If the count is low the answer is "which mass is still a plain box", never "add
# wedges until the number rises" (r15 s9, which r16 s4 re-arms).

AXES = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]


def face_normal(v, f):
    a, b_, c = (Vector(v[i]) for i in f[:3])
    n = (b_ - a).cross(c - a)
    return n.normalized() if n.length > 1e-9 else Vector((0, 0, 0))


def canonical(n):
    """Is this normal one of the 26 the brief has in mind -- the 6 axes plus the 12
    edge and 8 corner diagonals of a cube? r16 s4 expects "14 directions should
    become roughly 26", so a raw distinct-normal count is NOT comparable to it: every
    ramp drawn at an angle the art chose (the nose, the pack's back, the boot toe)
    contributes its own one-off normal. Reporting both keeps the number honest."""
    v = sorted(abs(round(x, 3)) for x in n)
    for cand in ((0, 0, 1), (0, 0.707, 0.707), (0.577, 0.577, 0.577)):
        if all(abs(a - c) < 0.02 for a, c in zip(v, cand)):
            return True
    return False


def metric(b):
    dirs, faces, off = {}, 0, 0
    for part, chunks in b.parts.items():
        for (v, f) in chunks:
            for face in f:
                n = face_normal(v, face)
                if n.length < 0.5:
                    continue
                faces += 1
                key = tuple(round(x, 3) + 0.0 for x in n)
                dirs[key] = dirs.get(key, 0) + 1
                if not any(abs(n[0] - a[0]) < 1e-3 and abs(n[1] - a[1]) < 1e-3
                           and abs(n[2] - a[2]) < 1e-3 for a in AXES):
                    off += 1
    canon = sum(1 for d in dirs if canonical(d))
    return faces, len(dirs), off, canon


def check_normals(b):
    """r15 s7.6 -- against each box's OWN centre, every build. Round 14 shipped
    eight builds with every arm and hand face reversed while all thirty landmark
    checks read +0.00 px. Each chunk here is one convex prism, so the test is exact."""
    bad = 0
    for part, chunks in b.parts.items():
        for (v, f) in chunks:
            ctr = Vector((sum(p[0] for p in v) / len(v),
                          sum(p[1] for p in v) / len(v),
                          sum(p[2] for p in v) / len(v)))
            for face in f:
                n = face_normal(v, face)
                fc = sum((Vector(v[i]) for i in face), Vector()) / len(face)
                if n.length > 0.5 and n.dot(fc - ctr) <= 0:
                    bad += 1
    return bad


def check_hair_covers_skull(b):
    """r15 s7.7 -- no head box may be the rearmost surface above the shoulders.
    Skin showing through the back of the hair is a 1 mm margin error that no
    dimension test can see."""
    def back(part, zmin):
        ys = [p[1] for (v, _) in b.parts.get(part, []) for p in v if p[2] >= zmin]
        return min(ys) if ys else None
    hb, kb = back("hair", zb(48)), back("head", zb(48))
    return hb, kb, (hb is not None and kb is not None and hb < kb)


def main():
    global FACET
    env = os.environ.get("R16_FACET")
    if env is not None:
        FACET = env not in ("0", "false", "False")
    bpy.ops.wm.read_factory_settings(use_empty=True)

    b = build()
    col = b.to_objects()
    faces, ndir, off, canon = metric(b)
    boxes = sum(len(c) for c in b.parts.values())
    tri = 0
    for chunks in b.parts.values():
        for (_, f) in chunks:
            for face in f:
                tri += len(face) - 2

    print("\n" + "=" * 66)
    print(f"  {REV}  FACET={FACET}   parts {len(b.parts)}   masses {boxes}")
    print(f"  cage faces          : {faces}")
    print(f"  distinct directions : {ndir}  "
          f"({canon} of the cube's 26, {ndir - canon} one-off ramp angles)")
    print(f"  faces off the 6 axes: {off} of {faces} ({100.0*off/max(faces,1):.1f} %)")
    print(f"  triangles           : {tri}")
    bad = check_normals(b)
    hb, kb, ok = check_hair_covers_skull(b)
    print(f"  inside-out faces    : {bad}   (r15 s7.6)")
    print(f"  hair back {hb:+.4f} vs skull back {kb:+.4f} -> "
          f"{'hair is rearmost, ok' if ok else 'SKIN SHOWS THROUGH'}   (r15 s7.7)")
    print("=" * 66 + "\n")

    out = os.environ.get("R16_BLEND")
    if out:
        bpy.ops.wm.save_as_mainfile(filepath=out)
        print("saved", out)
    return col


if __name__ == "__main__":
    main()
