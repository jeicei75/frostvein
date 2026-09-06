"""Standalone generator for the Frostvein voxel dwarf miner.

Reproduces the dwarf of the modelling reference sheet's Section A, at the
resolution ruled for version 2 of the brief, as a game-ready glTF asset.
No MCP, no live session, no manual steps.

    blender --background --python dwarf_miner.py -- <out.glb> [options]

        <out.glb>    destination path; parent directories are created

        --voxel M    metres per voxel (default 0.0125)
        --blend P    also save the editable .blend to P

Determinism
    There is none to manage: unlike the pines this generator draws no random
    numbers at all. Every voxel is placed by an explicit rule below, so
    identical arguments give a byte-identical GLB by construction and there is
    no --seed to pass.

Reused from voxel_pine.py
    The greedy mesher, the material, the exporter, the GLB reader, the PNG
    decoder and the volume oracle are IMPORTED from the sibling pine
    generator, which is left untouched. Only the genuinely dwarf-shaped things
    live here: the voxel body, the palette and its atlas.

Requires only the Python standard library and bpy.
"""

import os
import struct
import sys
import traceback
import zlib

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import voxel_pine as pine                                   # noqa: E402

ATLAS, CELL = pine.ATLAS, pine.CELL          # 64 px atlas, 4x4 grid of 16 px cells

# ---------------------------------------------------------------------------
# Palette -- Section A's "GEAR & PROP BREAKDOWN" swatch column, plus the two
# cells version 2 of the brief added. The atlas is a 4x4 grid of sixteen; ten
# are used.
#
# Cells 0-7 are the sheet's. Two of their labels are ambiguous on the sheet and
# CANNOT be settled from it -- see PALETTE_PROVENANCE.
# ---------------------------------------------------------------------------
HEX_SKIN       = "#E9D2BB"
HEX_BEARD      = "#5E4632"
HEX_SNOW       = "#FFFFFF"
HEX_TUNIC      = "#5F7A6A"
HEX_PANTS      = "#474B41"
HEX_METAL      = "#A9B2AC"
HEX_WOOD       = "#8B6B50"
HEX_WOOD_TRUNK = "#6B5B49"
HEX_HAIR       = "#34271C"
HEX_FLAME      = "#F0A63C"

SKIN, BEARD, SNOW, TUNIC, PANTS, METAL, WOOD, WOOD_TRUNK, HAIR, FLAME = range(10)
PALETTE_HEX = [
    HEX_SKIN, HEX_BEARD, HEX_SNOW, HEX_TUNIC, HEX_PANTS,
    HEX_METAL, HEX_WOOD, HEX_WOOD_TRUNK, HEX_HAIR, HEX_FLAME,
]
ROLE_NAMES = [
    "Skin", "Beard", "Snow/eye white", "Tunic", "Pants",
    "Metal", "Wood", "Wood Trunk", "Hair/dark iron", "Lantern flame",
]

PALETTE_PROVENANCE = (
    "Skin/Beard/Tunic/Metal  label legible on the sheet, swatch agrees\n"
    "Snow/Wood Trunk         confirmed: byte-identical in the shipped pine atlas\n"
    "Pants/Wood              UNRESOLVED. The sheet's 8 and B are the same glyph at\n"
    "                        1024 px, which is the sheet's full resolution and not a\n"
    "                        downscale, and the swatches are JPEG-compressed and\n"
    "                        internally noisy. Carried from the brief, NOT confirmed.\n"
    "Hair  #34271C           DERIVED, not sampled clean. See HAIR_DERIVATION.\n"
    "Flame #F0A63C           DERIVED, not sampled clean. See FLAME_DERIVATION."
)

# Cell 8. The brief says: darker than Beard, sampled from the contact-sheet head.
# Sampling it is not clean, and the honest number is a range rather than a value:
#
#   reference sheet, front ortho   hair Y 38.3   beard Y 43.9   ratio 0.87
#   contact sheet, frame r3c5      hair Y 20.3   beard Y 48.1   ratio 0.42
#
# Both sources are compromised in opposite directions. The sheet's figure is
# 139 px tall and JPEG-compressed, so a 5/255 luminance gap sits inside its
# noise; the contact sheet is torch-lit from above and behind, so the hair mass
# is frequently the shadow side while the beard catches the key. Neither ratio
# can be trusted alone, so this cell is the MIDPOINT of the two, 0.56, applied
# to the Beard albedo #5E4632 and keeping the beard's hue exactly:
#
#   (94, 70, 50) * 0.56 -> (53, 39, 28) = #34271C     Y 41.0 against beard's 73.7
#
# It is therefore a DERIVED value and not a measured one, which matters because
# check_asset.py cannot see this cell at all (see the note on cells 8 and 9
# below). What is certain, and is the point the brief makes, is the DIRECTION:
# v1 painted hair and beard the same #5E4632 and the whole head read as one
# brown mass. Any clear value separation fixes that; this one is 0.56.
HAIR_DERIVATION = "midpoint of the sheet (0.87) and contact-sheet (0.42) hair:beard ratios"

# Cell 9. The brief says: a warm amber, sampled from the contact-sheet panes.
# The pane CORES cannot be used -- they are blown out by the light the client
# puts inside the lantern, and measure #F7DFA6 (contact) / #FDECAB (sheet),
# which is a pale cream a hair off Skin #E9D2BB. Shipping that would repeat
# v1's failure exactly, where Metal + Snow read as a pale brick.
#
# The pane EDGE, where the glow falls off and the glass's own colour survives,
# measures #533517 -- R:G:B of 1 : 0.64 : 0.28. That hue, raised to a value a
# lit pane wants (Y ~= 174, between the sheet gear-callout's muted Y 158 and a
# fully saturated amber), gives (240, 166, 60) = #F0A63C.
FLAME_DERIVATION = "hue of the contact-sheet pane edge #533517, raised to a lit-glass value"

# The flame is a COLOUR and never an emitter. A pixel guard asserts that with
# all five light sources off, zero pixels satisfy R-B > 30; that guard passes
# on raw albedo this warm because with nothing lighting the scene every surface
# renders near-black. An emissive material is a light nobody can switch off.
# See make_material(), which sets no emission, and the check in main().

PUBLISHED_NAME = "SM_VoxelDwarf_Miner01"     # ruled by Wolf; also the mesh and node name
HEIGHT_VOXELS = 96                           # ruled by Wolf, brief v2
DEFAULT_VOXEL = 1.20 / HEIGHT_VOXELS         # 0.0125 m; he is 1.20 m tall and that has not changed


# ---------------------------------------------------------------------------
# The body
#
# Axes are Blender's: +X is the dwarf's RIGHT, +Y is the direction he faces,
# +Z is up. (For a character facing +Y with up +Z, right = forward x up = +X.)
# The exporter's export_yup then lands him facing -Z in glTF, which is the
# usual forward for a character. He therefore carries the pickaxe at +X and the
# lantern at -X, which is the contact sheet's hands round the right way.
#
# WIDTHS ARE EVEN ON PURPOSE. voxel_pine.greedy_mesh() shifts the lattice half
# a voxel in X and Y so an ODD-width trunk centres on the origin; that shift is
# half of 0.0125 m, which is off the project grid check_asset.py enforces. So
# the dwarf is even-width in X and Y and the half-shift is undone after meshing
# (see build_mesh). Every solid() column is even by construction -- see the
# note there on why its centre has to be an integer.
#
# HEIGHTS ARE MEASURED, not invented. The Z bands below come from resampling
# the reference sheet's front orthographic to 96 rows and reading the
# transitions off it; the sheet is 139 px tall there, so it fixes proportion
# and cannot fix detail. Detail comes from the contact sheet, magnified.
#
#   z  0.. 8   boot foot          z 24..62   tunic torso
#   z  9..10   boot cuff          z 30..74   beard
#   z 11..16   boot shaft         z 63..95   head
#   z 17..20   trouser gap        z 72..90   face
#   z 21..23   belt               z 82..85   brow ledge
#                                 z 76..81   eyes
#
# THE FORMS ARE ROUNDED, NOT BOXED, and that is the whole difference between
# this and v1. A voxel figure made of rectangular slabs greedy-meshes down to
# almost nothing -- the first cut of this body was 149,528 voxels and 451
# quads, because a flat slab face merges into one enormous quad no matter how
# many voxels are behind it. Triangle count here is not a cost to pay, it is
# the SYMPTOM of whether the surface has any shape: the brief asks for
# 14,000-30,000 triangles because that is what a rounded, stepped, ten-colour
# figure of this size produces. solid() below builds every organic mass as a
# superelliptic column whose profile varies with z, so the silhouette steps the
# way the reference's does and the quads follow.
#
# THE POSE IS THE CONTACT SHEET'S, NOT THE FRONT ORTHOGRAPHIC'S, and that is a
# decision worth naming. The sheet's front view is a display pose: the pickaxe
# lies diagonally across the body with the head and the lantern BOTH out on his
# left, which puts a pick head's worth of gear on one side and a haft butt on
# the other. The bounding box would still centre -- the contract only measures
# the box -- but it would centre on a point far off his feet, and every
# instance placed from the asset would stand beside its own origin. The contact
# sheet, which the brief makes the primary source, carries the pickaxe in the
# right hand and the lantern in the left, which is both the in-situ pose and
# the one that balances. Gear extents are then what centre him, so moving one
# without the other decentres the asset (see centre_voxels).
# ---------------------------------------------------------------------------

# Profiles are [(z, half_width_x, y_back, y_front)], piecewise-linear in z.
# half_width_x is a HALF width: the column spans 2*hx voxels, which is why
# every one of these is even without having to say so.
TORSO = [(19, 22, -12, 8), (22, 24, -13, 9), (26, 24, -14, 10), (34, 25, -15, 10),
         (44, 26, -16, 11), (52, 27, -16, 11), (58, 27, -16, 11), (62, 24, -15, 10)]
HEAD = [(63, 17, -15, 13), (66, 19, -17, 15), (72, 20, -18, 16), (84, 20, -18, 16),
        (89, 19, -17, 15), (93, 16, -14, 12), (95, 12, -11, 9)]
BEARD_P = [(30, 9, 6, 13), (33, 11, 5, 15), (38, 14, 4, 17), (44, 16, 3, 18),
           (50, 17, 2, 19), (56, 18, 1, 19), (62, 18, -2, 19), (70, 19, -8, 19),
           (74, 18, -12, 17)]
NAPE = [(55, 17, -18, -4), (59, 19, -18, -2), (63, 20, -18, 0)]
BOOT = [(0, 9, -10, 15), (3, 9, -10, 15), (6, 9, -9, 13), (8, 8, -9, 12)]
CUFF = [(9, 10, -11, 14), (10, 10, -11, 14)]
SHAFT = [(11, 9, -9, 10), (16, 8, -8, 9)]
THIGH = [(17, 8, -8, 9), (20, 9, -9, 9)]
LEG_X = 12                       # leg centre; mirrored to -12 for the other side

UPPER_ARM = [(46, 5, -9, 5), (52, 6, -10, 6), (58, 7, -11, 7), (62, 7, -11, 7)]
FOREARM = [(35, 5, -8, 7), (41, 5, -9, 6), (46, 5, -9, 5)]
FIST = [(29, 5, -8, 9), (33, 6, -9, 10), (37, 5, -8, 9)]
ARM_X = 31                       # arm centre, just clear of the torso's 27

PACK = [(33, 13, -35, -16), (37, 16, -37, -15), (48, 17, -38, -15),
        (57, 16, -37, -15), (62, 13, -35, -16)]
BEDROLL = [(63, 16, -34, -20), (66, 17, -35, -20), (69, 15, -33, -21)]


def sample(table, z):
    """Piecewise-linear (hx, y_back, y_front) from a profile table at height z."""
    if z <= table[0][0]:
        return table[0][1:]
    if z >= table[-1][0]:
        return table[-1][1:]
    for lo, hi in zip(table, table[1:]):
        if lo[0] <= z <= hi[0]:
            t = (z - lo[0]) / (hi[0] - lo[0])
            return tuple(a + (b - a) * t for a, b in zip(lo[1:], hi[1:]))
    return table[-1][1:]


def solid(put, table, z0, z1, colour, part, power=3.0, xc=0):
    """Fill a superelliptic column whose cross-section follows `table`.

    `xc` is the column's centre and MUST be an integer: the x cells run
    [xc - hx, xc + hx - 1], which is 2*hx wide -- even -- only because the
    centre sits on a lattice boundary rather than on a voxel. Putting it on a
    voxel centre would make every limb odd-width and put the whole model half a
    voxel off the project grid, which is the bug the module docstring's
    even-width note exists to prevent.

    `power` shapes the corner: 2 is an ellipse, 3 is a rounded box, and higher
    approaches the slab that greedy-meshes into nothing.
    """
    for z in range(z0, z1 + 1):
        hx, yb, yf = sample(table, z)
        hx = int(round(hx))
        yb, yf = int(round(yb)), int(round(yf))
        if hx < 1 or yf < yb:
            continue
        cy = (yb + yf) / 2.0
        hy = (yf - yb + 1) / 2.0
        for x in range(xc - hx, xc + hx):
            u = abs((x + 0.5 - xc) / hx)
            for y in range(yb, yf + 1):
                v = abs((y - cy) / hy)
                if u ** power + v ** power <= 1.0:
                    put(x, y, z, colour, part)


def build_voxels():
    """Return ({(x, y, z): palette index}, {(x, y, z): part name})."""
    vox, part_of = {}, {}

    def put(x, y, z, colour, part):
        vox[(x, y, z)] = colour
        part_of[(x, y, z)] = part

    def fill(x0, x1, y0, y1, z0, z1, colour, part="body"):
        for x in range(x0, x1 + 1):
            for y in range(y0, y1 + 1):
                for z in range(z0, z1 + 1):
                    put(x, y, z, colour, part)

    def both(table, z0, z1, colour, part, power=3.0, xc=0):
        """A limb and its reflection. x -> -1-x is exact on this lattice."""
        solid(put, table, z0, z1, colour, part, power, xc)
        solid(put, table, z0, z1, colour, part, power, -xc)

    # --- legs, z 0..20 -----------------------------------------------------
    # Short and thick: the front ortho gives the legs the bottom fifth of the
    # figure, which is what makes the head and the beard read as big.
    both(BOOT, 0, 8, WOOD_TRUNK, "body", 3.0, LEG_X)          # boot foot, leather
    both(CUFF, 9, 10, HAIR, "body", 3.0, LEG_X)               # dark turn-down cuff
    both(SHAFT, 11, 16, WOOD_TRUNK, "body", 3.0, LEG_X)       # boot shaft
    both(THIGH, 17, 20, PANTS, "body", 3.0, LEG_X)            # trouser, briefly bare

    # --- torso, z 19..62 ---------------------------------------------------
    # The tunic is short and belted low: hem at 19, belt at 21..23.
    solid(put, TORSO, 19, 62, TUNIC, "body")
    solid(put, TORSO, 55, 62, PANTS, "body")                  # shoulders, the darker green
    belt = [(z, hx + 1, yb - 1, yf + 1) for (z, hx, yb, yf) in TORSO]
    solid(put, belt, 20, 24, WOOD, "belt")                    # belt, a full band
    fill(-6, 5, 9, 11, 20, 24, METAL, "belt")                 # buckle, front face only

    # --- arms, z 29..62 ----------------------------------------------------
    # Sleeve to the elbow, bare forearm below it, fist at the end -- the
    # reference has no gloves. Both arms hang forward of the hips so each hand
    # meets its gear without the tool having to bisect him in profile.
    both(UPPER_ARM, 46, 62, PANTS, "body", 2.5, ARM_X)        # sleeve, matching the shoulders
    both(FOREARM, 35, 45, SKIN, "body", 2.5, ARM_X)           # bare forearm
    both(FIST, 29, 37, SKIN, "body", 2.5, ARM_X)              # fist

    # --- backpack, z 33..69 ------------------------------------------------
    # Hung off the back of the torso, with a rolled bedroll strapped across its
    # top. The side ortho gives it a third of the figure's depth.
    solid(put, PACK, 33, 62, WOOD, "backpack", 5.0)
    solid(put, BEDROLL, 63, 69, WOOD_TRUNK, "backpack", 2.0)   # a rolled bedroll IS round
    fill(-12, 11, -39, -38, 42, 56, WOOD_TRUNK, "backpack")   # flap
    fill(-16, 15, -39, -38, 36, 39, WOOD_TRUNK, "backpack")   # cinch strap round it
    for sx in (-1, 1):                                        # two straps down the flap
        fill(min(3 * sx, 7 * sx), max(3 * sx, 7 * sx), -40, -39, 42, 56, WOOD_TRUNK,
             "backpack")
    fill(-4, 3, -40, -40, 45, 51, METAL, "backpack")          # buckle
    for sx in (-1, 1):
        fill(min(6 * sx, 13 * sx), max(6 * sx, 13 * sx), -16, -15, 55, 62,
             WOOD_TRUNK, "backpack")                          # shoulder straps

    # --- head and beard, z 30..95 ------------------------------------------
    solid(put, HEAD, 63, 95, HAIR, "body", 3.0)
    solid(put, NAPE, 55, 63, HAIR, "body")
    solid(put, BEARD_P, 30, 74, BEARD, "body", 2.5)
    build_face(vox, put, fill)

    # Tunic hem trim and boot straps: small, and both are on the reference.
    hem = [(z, hx + 1, yb - 1, yf + 1) for (z, hx, yb, yf) in TORSO]
    solid(put, hem, 19, 19, WOOD_TRUNK, "body")
    both(CUFF, 13, 13, HAIR, "body", 3.0, LEG_X)              # a strap round each boot
    fill(-30, -21, 6, 12, 18, 26, WOOD_TRUNK, "belt")         # belt pouch, on his left

    build_pickaxe(fill)
    build_lantern(fill)

    # STRANDS. The contact sheet's beard and hair are visibly stranded, not
    # smooth, and cutting a groove every third column is what puts that in. It is
    # also most of the difference between a figure that greedy-meshes to nothing
    # and one that does not: a groove breaks the co-planar run a flat mass merges
    # into, so the quads follow the shape instead of hiding it. Purely a function
    # of x, so it stays deterministic.
    groove(vox, part_of, BEARD, 30, 74)
    groove(vox, part_of, HAIR, 63, 95)
    return vox, part_of


def groove(vox, part_of, colour, z0, z1, step=3):
    """Cut a one-voxel channel down every `step`-th column of a coloured mass.

    Only the frontmost voxel of each column goes, so the mass stays closed --
    the volume oracle in main() checks exactly that and would catch a hole.
    """
    columns = {}
    for (x, y, z), c in vox.items():
        if c != colour or not z0 <= z <= z1 or x % step:
            continue
        if columns.get((x, z), -10 ** 6) < y:
            columns[(x, z)] = y
    for (x, z), y in columns.items():
        del vox[(x, y, z)]
        del part_of[(x, y, z)]


# --- the face --------------------------------------------------------------
# Painted onto the head's FRONT SURFACE rather than placed at absolute
# coordinates, because the head is a rounded column and its front plane moves
# with z and x. front_surface() finds it; everything below then works in
# "voxels proud of the face" and survives any reshaping of HEAD.
#
# This is what a 12-voxel budget could not buy and 96 can: a brow ledge stepped
# two voxels proud with the eyes in its shadow, whites with pupils set apart by
# a nose bridge, and a nose that projects past the brow. v1 correctly reported
# all four as unavailable.
FACE_HALF = 15                  # the face window's half width; the hair frames it beyond
HAIRLINE = {84: 13, 85: 11}     # it narrows as it rises, so the hairline rounds
FACE_Z = (72, 85)               # chin-line to hairline. NOT past 88: the hair closes
                                # over the top of it, and running the skin to 90 left a
                                # bare cream band ringing the head above the brow.
EYE_Z = (75, 80)
BROW_Z = (81, 83)               # forehead is then 84..88, bare skin
BROW_HALF = 11                  # narrower than the face: skin passes it at the temples
SIDEBURN_TOP = 79               # the beard rises this high at the sides of the face
NOSE_Z = (70, 80)


def front_surface(vox):
    """Frontmost occupied y for each (x, z). The face is painted onto this."""
    front = {}
    for (x, y, z) in vox:
        key = (x, z)
        if front.get(key, -10 ** 6) < y:
            front[key] = y
    return front


def build_face(vox, put, fill):
    front = front_surface(vox)

    def face_cells(z0, z1, half=FACE_HALF):
        for z in range(z0, z1 + 1):
            span = min(half, HAIRLINE.get(z, half))
            for x in range(-span, span):
                y = front.get((x, z))
                if y is not None:
                    yield x, y, z

    # Skin, three voxels deep so the brow and nose can be cut back into it
    # without exposing hair through the cheek.
    for x, y, z in face_cells(*FACE_Z):
        for d in range(3):
            put(x, y - d, z, SKIN, "body")

    # Brow ledge: full face width, two voxels proud, with the forehead left as
    # bare skin above it. Dark, so the eyes below read as sunk in its shadow.
    for x, y, z in face_cells(*BROW_Z, half=BROW_HALF):
        put(x, y + 1, z, HAIR, "body")
        put(x, y + 2, z, HAIR, "body")

    # Eyes. Whites four across and five high, a two-voxel pupil sunk one voxel
    # behind the white, and a four-voxel nose bridge holding them apart. The
    # bridge is the reason a pair reads as eyes rather than as a band.
    def eye(x_range, z_range, colour):
        for x in x_range:
            for z in z_range:
                for cell in (x, -1 - x):                # both eyes; the reflection is exact
                    y = front.get((cell, z))
                    if y is not None:
                        put(cell, y + 1, z, colour, "body")

    eye(range(2, 8), range(EYE_Z[0], EYE_Z[1] + 1), SNOW)
    eye(range(3, 7), range(EYE_Z[0] + 1, EYE_Z[1]), HAIR)

    # Nose: a bridge between the eyes, then a tip that projects past the brow.
    # It is the only part of the face proud of the brow ledge.
    for z in range(NOSE_Z[0], NOSE_Z[1] + 1):
        reach = 3 if z >= 76 else 4        # bridge, then the wider tip
        half = 2 if z >= 76 else 3
        for x in range(-half, half):
            y = front.get((x, z))
            if y is None:
                continue
            for d in range(1, reach + 1):
                put(x, y + d, z, SKIN, "body")

    # Moustache, closing the gap between the nose and the beard so the face
    # does not float on a brown field.
    for x, y, z in face_cells(68, 71, 11):
        put(x, y + 1, z, BEARD, "body")
        put(x, y + 2, z, BEARD, "body")

    # Sideburns, rising to eye level and framing the face down to the beard.
    for z in range(63, SIDEBURN_TOP + 1):
        for x in list(range(9, FACE_HALF)) + list(range(-FACE_HALF, -9)):
            y = front.get((x, z))
            if y is not None:
                put(x, y, z, BEARD, "body")
                put(x, y + 1, z, BEARD, "body")


# --- pickaxe, in the right hand (+X) ---------------------------------------
# 0.83x dwarf height, which is the sheet's label: 80 voxels from butt to the
# top of the head. Carried upright, head level with the top of his hair -- NOT
# above it. The bounding box is his height, so a tool that outtops him would
# make a 1.20 m dwarf 1.10 m of dwarf under a raised pick.
#
# The head is the sheet's: a central boss the haft passes through, and two
# wings sweeping out and DOWN to a point. Both wings matter. A bar with a cap
# reads as a hammer head-on, which is what v1's did.
HAFT_X = (36, 41)
HAFT_Y = (6, 11)


def build_pickaxe(fill):
    x0, x1 = HAFT_X
    y0, y1 = HAFT_Y
    fill(x0, x1, y0, y1, 15, 92, WOOD, "pickaxe")                 # haft, butt at z 15,
    #                                                            # ending INSIDE the boss
    fill(x0 - 2, x1 + 2, y0 - 1, y1 + 1, 86, 92, METAL, "pickaxe")   # boss, around the haft
    # Wings, stepping out and down from the boss and mirrored about the haft's
    # own centre plane (x -> 73 - x), not the body's.
    for span, ztop, zbot in ((3, 92, 89), (6, 91, 87), (9, 89, 85),
                             (11, 87, 84), (13, 85, 83)):
        for u0 in (x1 + span - 2, x0 - span):
            fill(u0, u0 + 2, y0, y1, zbot, ztop, METAL, "pickaxe")


# --- lantern, in the left hand (-X) ----------------------------------------
# 0.33x dwarf height, the sheet's label: 32 voxels from base to the top of the
# ring. Hung from the fist, so its ring is at the hand and the body swings just
# below -- slung lower it reads as a box on the floor.
#
# It reads as a lantern because it has a CAGE: dark corner posts and a centre
# mullion standing in front of two warm panes, which is what the contact sheet
# shows and what v1's Metal-and-Snow block could not do. The frame is the Pants
# cell, the darkest neutral in the palette and the closest thing to iron.
LANT_X = (-45, -28)
LANT_Y = (0, 17)


def build_lantern(fill):
    x0, x1 = LANT_X
    y0, y1 = LANT_Y
    fill(x0 + 6, x1 - 6, y0 + 6, y1 - 6, 33, 37, PANTS, "lantern")   # ring, in the fist
    fill(x0 + 2, x1 - 2, y0 + 2, y1 - 2, 29, 32, PANTS, "lantern")   # peaked cap
    fill(x0, x1, y0, y1, 26, 28, PANTS, "lantern")                   # cap brim
    fill(x0 + 1, x1 - 1, y0 + 1, y1 - 1, 11, 25, FLAME, "lantern")   # glass, all four faces
    fill(x0 + 4, x1 - 4, y0 + 4, y1 - 4, 11, 25, PANTS, "lantern")   # dark body behind it
    for u0, v0 in ((x0 + 1, y0 + 1), (x1 - 3, y0 + 1), (x0 + 1, y1 - 3), (x1 - 3, y1 - 3)):
        fill(u0, u0 + 2, v0, v0 + 2, 11, 25, PANTS, "lantern")       # corner posts
    fill(x0 + 8, x1 - 8, y0 + 1, y1 - 1, 11, 25, PANTS, "lantern")   # mullion, X faces
    fill(x0 + 1, x1 - 1, y0 + 8, y1 - 8, 11, 25, PANTS, "lantern")   # mullion, Y faces
    fill(x0, x1, y0, y1, 7, 10, PANTS, "lantern")                    # base
    fill(x0 + 1, x1 - 1, y0 + 1, y1 - 1, 5, 6, METAL, "lantern")     # foot rim


# ---------------------------------------------------------------------------
# Centring
# ---------------------------------------------------------------------------
def centre_voxels(vox, part_of):
    """Translate the lattice so the bounding box centres on X and Y.

    A box spanning [min, max] in voxels occupies [min, max+1] in lattice units,
    so it centres on the origin exactly when max == -min - 1 -- which requires
    an EVEN span, and no translation can fix an odd one. That is asserted here
    rather than left to the GLB check, because the fix is authoring a voxel in
    or out of a gear extent and the assertion is what says which axis to look at.

    The translation is reported on the FIGURES line. It is small by design: the
    gear extents above are chosen to balance, so a large offset here means the
    pickaxe and the lantern have drifted apart and the body no longer stands on
    its own origin. Z is never translated -- his feet are on the ground at z 0.
    """
    shifts = []
    for axis in (0, 1):
        lo = min(p[axis] for p in vox)
        hi = max(p[axis] for p in vox)
        span = hi - lo + 1
        if span % 2:
            raise SystemExit(
                "error: %s span is %d voxels, which is odd and cannot centre on the "
                "origin. Add or remove one voxel column of gear on that axis."
                % ("XY"[axis], span))
        shifts.append(-(lo + hi + 1) // 2)
    dx, dy = shifts
    if not dx and not dy:
        return vox, part_of, (0, 0)
    moved = {(x + dx, y + dy, z): c for (x, y, z), c in vox.items()}
    moved_parts = {(x + dx, y + dy, z): p for (x, y, z), p in part_of.items()}
    return moved, moved_parts, (dx, dy)


# ---------------------------------------------------------------------------
# Atlas
# ---------------------------------------------------------------------------
def encode_palette_png():
    """Encode the 64x64 palette atlas as PNG bytes straight from the hex above.

    Written by hand rather than via Image.pack() with no data: pack() on a
    GENERATED image re-encodes the generated source and throws away whatever
    was assigned to .pixels, and the glTF exporter then copies those (black)
    packed bytes verbatim into the GLB. This is voxel_pine's encoder with this
    module's palette; it is not imported because that one closes over the
    pine's seven-colour PALETTE_HEX.
    """
    rgb = bytearray(ATLAS * ATLAS * 3)               # top-down rows, as PNG wants
    for index, hx in enumerate(PALETTE_HEX):
        r, g, b = pine.hex_to_bytes(hx)
        cx, cy = pine.palette_cell_origin(index)
        for y in range(cy, cy + CELL):
            row = ATLAS - 1 - y                      # flip: Blender is bottom-up
            for x in range(cx, cx + CELL):
                o = (row * ATLAS + x) * 3
                rgb[o:o + 3] = bytes((r, g, b))
    raw = b"".join(b"\x00" + bytes(rgb[y * ATLAS * 3:(y + 1) * ATLAS * 3])
                   for y in range(ATLAS))            # filter type 0 on every row

    def chunk(tag, payload):
        return (struct.pack(">I", len(payload)) + tag + payload
                + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF))

    return (b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", ATLAS, ATLAS, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9))
            + chunk(b"IEND", b""))


def make_palette_image(name):
    img = bpy.data.images.new(name, ATLAS, ATLAS, alpha=False)
    # images.new() is a BYTE image, so .pixels is display-referred: write the
    # sheet hex straight in. Linearising here (the obvious-looking thing) bakes
    # a second sRGB decode into the texels and ships a visibly too-dark asset.
    px = [0.0] * (ATLAS * ATLAS * 4)
    for index, hx in enumerate(PALETTE_HEX):
        cx, cy = pine.palette_cell_origin(index)
        r, g, b = (c / 255.0 for c in pine.hex_to_bytes(hx))
        for y in range(cy, cy + CELL):
            for x in range(cx, cx + CELL):
                o = (y * ATLAS + x) * 4
                px[o:o + 4] = [r, g, b, 1.0]
    img.pixels = px                                  # kept consistent with the packed PNG
    img.colorspace_settings.name = 'sRGB'
    png = encode_palette_png()
    img.pack(data=png, data_len=len(png))            # exact bytes for the exporter
    img.source = 'FILE'
    img.filepath_raw = "//%s.png" % name
    return img


def palette_from_glb(gj, binary):
    """Return the shipped hex of every palette cell, read out of the GLB."""
    bv = gj["bufferViews"][gj["images"][0]["bufferView"]]
    start = bv.get("byteOffset", 0)
    png = binary[start:start + bv["byteLength"]]
    width, height, nch, pix = pine.decode_png_rgb(png)
    shipped = []
    for index in range(len(PALETTE_HEX)):
        cx, cy = pine.palette_cell_origin(index)
        x = cx + CELL // 2
        y = height - 1 - (cy + CELL // 2)            # flip back to top-down rows
        o = (y * width + x) * nch
        shipped.append("#%02X%02X%02X" % (pix[o], pix[o + 1], pix[o + 2]))
    return shipped


# ---------------------------------------------------------------------------
# Blender scene assembly
# ---------------------------------------------------------------------------
def build_mesh(vox, voxel):
    """Greedy-mesh with the pine's mesher, then undo its half-voxel lattice shift.

    voxel_pine.greedy_mesh subtracts half a voxel from X and Y so an odd-width
    trunk centres on the origin. That is a rigid translation of the whole quad
    soup, so adding it back here is exact and leaves the mesher itself unedited
    -- see the layout note above for why the dwarf needs the unshifted lattice.
    """
    verts, faces, uvs = pine.greedy_mesh(vox, voxel)
    half = 0.5 * voxel
    return [(x + half, y + half, z) for (x, y, z) in verts], faces, uvs


def build_object(verts, faces, uvs):
    me = bpy.data.meshes.new(PUBLISHED_NAME)
    me.from_pydata(verts, [], faces)
    me.update()
    uvl = me.uv_layers.new(name="UVMap")
    for i, uv in enumerate(uvs):
        uvl.data[i].uv = uv
    for poly in me.polygons:
        poly.use_smooth = False                      # voxels are flat-shaded
    me.validate(verbose=False)
    img = make_palette_image("T_VoxelDwarf_Palette")
    me.materials.append(pine.make_material("M_VoxelDwarf", img))
    ob = bpy.data.objects.new(PUBLISHED_NAME, me)
    bpy.context.collection.objects.link(ob)
    ob.location = (0.0, 0.0, 0.0)
    ob.rotation_euler = (0.0, 0.0, 0.0)
    ob.scale = (1.0, 1.0, 1.0)
    return ob


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------
USAGE = ("usage: blender --background --python dwarf_miner.py -- "
         "<out.glb> [--voxel M] [--blend P]")


def parse_args(argv):
    argv = argv[argv.index("--") + 1:] if "--" in argv else []
    if not argv:
        raise SystemExit(USAGE)
    out = argv[0]
    if not out.lower().endswith(".glb"):
        raise SystemExit("error: output must be a .glb path\n" + USAGE)
    if os.path.splitext(os.path.basename(out))[0] != PUBLISHED_NAME:
        # The contract requires basename == mesh name == node name, and the mesh
        # name is compiled into the client. Refuse rather than write a GLB that
        # check_asset.py will reject on the naming clause. This fires BEFORE the
        # --voxel guard below, so the two are exercised independently.
        raise SystemExit("error: basename must be %r, got %r\n%s"
                         % (PUBLISHED_NAME, os.path.basename(out), USAGE))
    voxel, blend, rest = DEFAULT_VOXEL, None, list(argv[1:])
    while rest:
        flag = rest.pop(0)
        if flag == "--voxel":
            voxel = float(rest.pop(0))
        elif flag == "--blend":
            blend = os.path.abspath(rest.pop(0))
        else:
            raise SystemExit("error: unknown option %r\n%s" % (flag, USAGE))
    # Voxel size must be positive. Zero is the dangerous one: it collapses the mesh to a point
    # AND scales expected_h/expected_volume by the same zero, so every closure check passes
    # vacuously and the script reports OK on nothing. The oracle has to be independent of the
    # input it is checking. (Negative values already fail the checks, on sign.)
    if not voxel > 0.0:
        raise SystemExit("error: --voxel must be greater than 0 (got %r)\n%s" % (voxel, USAGE))
    return os.path.abspath(out), voxel, blend


def main():
    out, voxel, blend = parse_args(list(sys.argv))
    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)

    pine.wipe_scene()
    vox, part_of = build_voxels()
    vox, part_of, (dx, dy) = centre_voxels(vox, part_of)
    verts, faces, uvs = build_mesh(vox, voxel)
    ob = build_object(verts, faces, uvs)
    pine.export_glb(ob, out)
    if blend:
        os.makedirs(os.path.dirname(blend) or ".", exist_ok=True)
        bpy.ops.wm.save_as_mainfile(filepath=blend, check_existing=False)

    # --- figures, read back out of the GLB that was just written -----------
    gj, binary, nbytes = pine.load_glb(out)
    prim = gj["meshes"][0]["primitives"][0]
    pos_acc = gj["accessors"][prim["attributes"]["POSITION"]]
    positions = pine.read_accessor(gj, binary, prim["attributes"]["POSITION"])
    indices = pine.read_accessor(gj, binary, prim["indices"])
    uv_vals = pine.read_accessor(gj, binary, prim["attributes"]["TEXCOORD_0"])

    lo, hi = pos_acc["min"], pos_acc["max"]          # glTF space: Y is up
    size = [hi[i] - lo[i] for i in range(3)]
    centre = [(hi[i] + lo[i]) / 2.0 for i in range(3)]
    vol = pine.signed_volume(positions, indices)
    expected_vol = len(vox) * voxel ** 3
    expected_h = HEIGHT_VOXELS * voxel
    nverts, ntris, nquads = len(positions), len(indices) // 3, len(faces)
    nmats = len(gj.get("materials", []))
    nprims = len(gj["meshes"][0]["primitives"])
    shipped_palette = palette_from_glb(gj, binary)
    groups = {}
    for part in part_of.values():
        groups[part] = groups.get(part, 0) + 1
    mesh_name = gj["meshes"][0]["name"]
    node_name = next(n["name"] for n in gj["nodes"] if n.get("mesh") == 0)
    basename = os.path.splitext(os.path.basename(out))[0]

    print(
        "FIGURES name=%s voxel=%.4f height_voxels=%d voxels=%d groups=%s centring=%+d,%+d "
        "quads=%d verts=%d tris=%d bbox=%.4fx%.4fx%.4f centre_x=%+.6f centre_z=%+.6f "
        "min_y=%+.6f volume=%.6f expected_volume=%.6f materials=%d primitives=%d images=%d "
        "palette=%s glb_bytes=%d"
        % (PUBLISHED_NAME, voxel, HEIGHT_VOXELS, len(vox),
           ",".join("%s:%d" % kv for kv in sorted(groups.items())), dx, dy,
           nquads, nverts, ntris, size[0], size[1], size[2],
           centre[0], centre[2], lo[1], vol, expected_vol, nmats, nprims,
           len(gj.get("images", [])), ",".join(shipped_palette), nbytes)
    )

    # --- checks -------------------------------------------------------------
    fails = []

    def check(ok, msg):
        if not ok:
            fails.append(msg)

    check(abs(centre[0]) < 1e-6,
          "bbox centre X is %+.6f, expected 0.000000 (asset leans in X)" % centre[0])
    check(abs(centre[2]) < 1e-6,
          "bbox centre Z is %+.6f, expected 0.000000 (asset leans in Z)" % centre[2])
    check(abs(lo[1]) < 1e-6,
          "bbox min Y is %+.6f, expected 0.000000 (feet not on the ground)" % lo[1])
    check(abs(size[1] - expected_h) < 1e-4,
          "height is %.4f m, expected %.4f m" % (size[1], expected_h))
    # He is 96 voxels tall, which is the ruled resolution and NOT a consequence of
    # --voxel: read the count back out of the written height, so a body authored to
    # some other count fails here rather than silently shipping.
    check(abs(size[1] / voxel - HEIGHT_VOXELS) < 1e-3,
          "model is %.3f voxels tall, expected %d" % (size[1] / voxel, HEIGHT_VOXELS))
    # The project grid, which check_asset.py enforces at 0.0125 m. The pine never
    # checks this: its half-voxel lattice shift happens to land on the grid at a
    # 0.2 m voxel. Ours does not, and this is the check that catches it.
    grid = DEFAULT_VOXEL
    check(all(abs(v - round(v / grid) * grid) <= 1e-5 for p in positions for v in p),
          "some POSITION values are off the %g m project grid" % grid)
    check(abs(vol - expected_vol) <= max(1e-6, expected_vol * 1e-4),
          "signed volume %.6f != voxel volume %.6f (hull not closed, faces "
          "duplicated, or normals inverted)" % (vol, expected_vol))
    check(ntris == nquads * 2, "tris %d != quads*2 %d" % (ntris, nquads * 2))
    check(pine.check_mesh_properties(nquads, nverts),
          "verts %d != quads*4 %d (not the expected unwelded quad soup)"
          % (nverts, nquads * 4))
    check(nmats == 1, "expected exactly 1 material, got %d" % nmats)
    check(nprims == 1, "expected exactly 1 primitive (1 draw call), got %d" % nprims)
    check(len(gj.get("meshes", [])) == 1,
          "expected exactly 1 mesh, got %d" % len(gj.get("meshes", [])))
    check(not gj.get("extensionsUsed"),
          "expected no glTF extensions, got %s" % gj.get("extensionsUsed"))
    check(not gj.get("extensionsRequired"),
          "expected no required glTF extensions, got %s" % gj.get("extensionsRequired"))
    check(gj["materials"][0].get("doubleSided", False) is False,
          "material should be single-sided")
    check(all(0.0 <= u <= 1.0 and 0.0 <= v <= 1.0 for u, v in uv_vals),
          "some UVs fall outside 0-1")
    check(len(gj.get("images", [])) == 1,
          "expected exactly 1 embedded palette image, got %d" % len(gj.get("images", [])))
    # Guards a real regression: Image.pack() with no data silently ships a black
    # atlas, which every check above still passes. ALL TEN cells are read back
    # here, including 8 and 9, which check_asset.py cannot see: its PALETTE_HEX is
    # hardcoded to the pine's seven and it iterates range(len(PALETTE_HEX)). This
    # is the only mechanical check those two cells get anywhere.
    check(shipped_palette == PALETTE_HEX,
          "shipped palette %s != this module's palette %s"
          % (",".join(shipped_palette), ",".join(PALETTE_HEX)))
    check(gj["samplers"][0].get("magFilter") == 9728,
          "palette magFilter is %s, expected 9728 (NEAREST)"
          % gj["samplers"][0].get("magFilter"))
    check(gj["samplers"][0].get("wrapS") == 33071 and gj["samplers"][0].get("wrapT") == 33071,
          "palette sampler wrap is %s/%s, expected 33071/33071 (CLAMP_TO_EDGE)"
          % (gj["samplers"][0].get("wrapS"), gj["samplers"][0].get("wrapT")))
    # The naming clause becomes a compiled-in string in the client; check the
    # file that was written, not the request.
    check(mesh_name == node_name == PUBLISHED_NAME == basename,
          "naming clause: basename/mesh/node are %r/%r/%r, expected %r"
          % (basename, mesh_name, node_name, PUBLISHED_NAME))
    # No lights, cameras, animations or armatures may reach the GLB: the client
    # owns lighting, and nothing here is rigged.
    for clause in ("animations", "skins", "cameras", "extensions"):
        check(not gj.get(clause), "GLB must contain no %s, got %s" % (clause, gj.get(clause)))
    # The lantern is a colour, never an emitter: an emissive material is a light
    # nobody can switch off, and the darkness guard has found one from the seat.
    check(not any("emissive" in key.lower() for key in gj["materials"][0]),
          "material must not be emissive: %s" % sorted(gj["materials"][0]))

    if fails:
        for msg in fails:
            sys.stderr.write("FAIL: %s\n" % msg)
        sys.stderr.write("%d check(s) failed\n" % len(fails))
        sys.exit(1)
    print("OK %s -> %s" % (PUBLISHED_NAME, out))
    sys.exit(0)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        # main() and parse_args already exit with a meaningful code; assert-style failures use 1.
        raise
    except Exception as error:
        # Blender's --background runner prints a traceback for ANY uncaught exception and still
        # exits 0, so a bad --voxel value, an unwritable output path or an export failure would
        # report success having written nothing. Same guard voxel_pine.py carries, for the same
        # reason: exit 0 with no output is not a result.
        traceback.print_exc()
        raise SystemExit("generator failed: %s: %s" % (type(error).__name__, error)) from error
