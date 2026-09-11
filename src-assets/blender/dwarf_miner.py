"""Standalone generator for the Frostvein voxel dwarf miner, revision r3.

Builds the miner of `references/dwarf.mp4` -- the authority for this round -- as a
game-ready, SKINNED glTF asset. No MCP, no live session, no manual steps.

    blender --background --python dwarf_miner.py -- <out.glb> [options]

        <out.glb>     destination path; parent directories are created
        --voxel M     metres per voxel (default 0.0125)
        --pose NAME   neutral (default) or swing
        --beard NAME  which beard part to mount     (default miner-full)
        --hair NAME   which hair part to mount      (default miner-shag)
        --blend P     also save the editable .blend to P

Determinism
    There is none to manage: this generator draws no random numbers at all.
    Every voxel is placed by an explicit rule below, so identical arguments give
    a byte-identical GLB by construction and there is no --seed to pass.

What changed in r3, and why
    Round 2 spent 2.4x the reference's resolution on a repeating SURFACE pattern
    -- groove() cut a one-voxel channel every third column -- and the figure read
    flatter than a reference that is COARSER than it. The grooves are gone. In
    their place:

      * every major material carries 2-3 palette cells and the form is painted
        with them (hem, fold, lock, cuff, highlight), 23 cells against 10;
      * detail is spent on the SILHOUETTE -- beard locks, a stepped tunic hem,
        stepped boot cuffs, a buckle that is a shape rather than a painted face;
      * the figure is rigged to a named skeleton with RIGID weights, one mesh,
        and no quad may cross a joint;
      * the lantern's flame cell is emissive, through a second UV set, so the
        one-mesh / one-material / one-image shape survives;
      * beard, hair, pickaxe and lantern are parts on DECLARED SOCKETS, so the
        first variation axes are swaps rather than remodels.

Reused from voxel_pine.py
    The material scaffolding, the GLB reader, the PNG decoder and the volume
    oracle are IMPORTED from the sibling pine generator, which is left
    untouched. The MESHER and the EXPORTER are this module's own now, for two
    reasons neither of which is taste: the mesher has to break greedy runs at a
    joint boundary (see greedy_mesh_parts), and the exporter has to carry a skin
    and a second UV set, which the pine's does not.

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

# ---------------------------------------------------------------------------
# Identity
#
# The FILE PATH is a slot and does not churn: assets/gltf/SM_VoxelDwarf_Miner01.glb
# is what the client loads, and `01` denotes the ROLE (miner, as opposed to a
# future smith) and never a version. The INTERNAL names carry the revision,
# because they are what a stale binary shows about itself. A candidate that has
# not been promoted carries the revision in its basename too, so that basename,
# mesh and node still agree.
# ---------------------------------------------------------------------------
PUBLISHED_NAME = "SM_VoxelDwarf_Miner01"     # the slot; the client's DWARF_SCENE_PATH
REVISION = "r3"                              # bump on every authored round that changes content
ASSET_NAME = "%s_%s" % (PUBLISHED_NAME, REVISION)
MATERIAL_NAME = "M_VoxelDwarf_%s" % REVISION
IMAGE_NAME = "T_VoxelDwarf_Palette_%s" % REVISION


def asset_name(pose):
    """The published name of one POSE of this revision.

    A posed GLB and a neutral one are different assets and must not share a name.
    That is the defect the whole scheme exists to prevent, one level down from two
    different meshes both called SM_VoxelPine_Tree02: a stale binary that names
    itself correctly authenticates itself.
    """
    return ASSET_NAME if pose == "neutral" else "%s_%s" % (ASSET_NAME, pose)


def carries_revision(name):
    """The revision has to appear as a TOKEN, not as a suffix.

    `SM_VoxelDwarf_Miner01_r3_swing` carries r3 and does not end with it, and a
    suffix test would reject it; `..._r33` ends with neither and a substring test
    would accept it. Underscore-delimited is the rule that gets both right.
    """
    return "_%s_" % REVISION in "%s_" % name

HEIGHT_VOXELS = 96                           # held at 96; see HEIGHT_RULING
DEFAULT_VOXEL = 1.20 / HEIGHT_VOXELS         # 0.0125 m; he is 1.20 m tall and that has not moved

# Why 96 and not the reference's ~60-65, which the brief leaves as this seat's call.
#
# The brief's measurement is not in doubt -- the mp4 figure spans ~510 px at a ~8 px
# voxel pitch, so it is roughly 60-65 voxels tall against our 96, and we are already
# FINER than the target while reading flatter. Resolution is not the lever and the
# count does not go up. But it cannot come down to 60 either, and the reason is
# mechanical rather than aesthetic:
#
#   check_asset.py's grid clause requires every POSITION to be a multiple of
#   PROJECT_GRID_METRES = 0.0125 m. The voxel is 1.20 / N, so N must divide 96.
#   The only candidates near the reference are 96 and 48 -- 60, 64 and 65 all put
#   every vertex off the project grid, and 48 drops the eyes, the brow ledge and
#   the nose, which is exactly the loss v1 reported at 12 voxels.
#
# So the choice is 96 or a contract change, and a contract change is not this
# round's to make. 96 it is, and the round's lever is value and silhouette.
HEIGHT_RULING = "held at 96: N must divide 96 for the 0.0125 m grid clause; 48 loses the face"


# ---------------------------------------------------------------------------
# Palette -- 23 cells in a 64x64 atlas, 8x8 grid of 8 px cells
#
# THE TEN RULED HEXES ARE UNCHANGED. Every new cell is a VALUE STEP computed
# from one of them by a stated factor, not a new measurement: the mp4 is a
# 3.4 MB h264 encode of a torch-lit scene and nothing sampled off it would
# survive being called a colour. What the video DOES settle, and what these
# cells exist for, is that each material carries several values -- the tunic has
# a lighter hem, a mid field and a darker fold; the beard has light locks over a
# mid mass with dark outline locks; the belt steps between two leathers.
#
# step() scales a hex, which keeps its hue EXACTLY -- the same method the r2
# HAIR cell was derived by. It refuses to clamp, so a factor that would blow a
# channel past 255 is an error rather than a silent desaturation. tint() mixes
# toward white and is used only where a scale would clamp; it is named
# separately because it does NOT keep hue, and the two are not interchangeable.
#
# THE ATLAS GEOMETRY MOVED: 8x8 cells of 8 px, not 4x4 of 16 px. The image is
# still the contracted 64x64, but a 4x4 grid holds sixteen cells and this
# palette needs twenty-three. The cost is real and is reported: check_asset.py
# samples the atlas on the 4x4 grid, so it reads this palette wrongly. See the
# report -- that clause is owed the same V2 work as the one-node clause.
# ---------------------------------------------------------------------------
ATLAS = pine.ATLAS                           # 64 px, the contracted V1 atlas size
CELL = 8                                     # 8x8 grid of 8 px cells -> 64 cells available
INSET = 2                                    # UV inset, in texels, from each cell's edge
CELLS_PER_ROW = ATLAS // CELL


def step(hx, factor):
    """Scale a ruled hex by `factor`, keeping its hue exactly.

    Refuses to clamp: a factor that would push any channel past 255 raises,
    because a clamped scale silently desaturates and stops being the derivation
    it claims to be.
    """
    raw = [c * factor for c in pine.hex_to_bytes(hx)]
    if max(raw) > 255.0:
        raise ValueError("step(%s, %.2f) clamps at %.1f; use tint() and say so" % (hx, factor, max(raw)))
    return "#%02X%02X%02X" % tuple(int(round(c)) for c in raw)


def tint(hx, t):
    """Mix a hex toward white by `t`. Does NOT keep hue -- only for cells a
    scale would clamp, and every use below says why it is the right one."""
    return "#%02X%02X%02X" % tuple(
        int(round(c + (255 - c) * t)) for c in pine.hex_to_bytes(hx))


# The ten ruled cells, carried verbatim from r2. Provenance is unchanged and is
# recorded in PALETTE_PROVENANCE below.
HEX_SKIN    = "#E9D2BB"
HEX_BEARD   = "#5E4632"
HEX_SNOW    = "#FFFFFF"
HEX_TUNIC   = "#5F7A6A"
HEX_PANTS   = "#474B41"
HEX_METAL   = "#A9B2AC"
HEX_WOOD    = "#8B6B50"
HEX_LEATHER = "#6B5B49"          # r2's "Wood Trunk"; renamed for what it is on this asset
HEX_HAIR    = "#34271C"
HEX_FLAME   = "#F0A63C"

(SKIN, SKIN_SHADE,
 BEARD, BEARD_LIT, BEARD_DARK,
 SNOW,
 TUNIC, TUNIC_LIT, TUNIC_DARK,
 PANTS, PANTS_LIT,
 METAL, METAL_LIT, METAL_DARK,
 WOOD, WOOD_LIT,
 LEATHER, LEATHER_LIT, LEATHER_DARK,
 HAIR, HAIR_LIT,
 FLAME, FLAME_CORE) = range(23)

# (index, hex, role). Ordered so a material's cells sit together: a variant is a
# TABLE OF HEXES against these indices, and a reader swapping a tunic wants its
# three cells adjacent.
PALETTE = [
    (SKIN,         HEX_SKIN,                    "Skin"),
    (SKIN_SHADE,   step(HEX_SKIN, 0.80),        "Skin, in shadow"),
    (BEARD,        HEX_BEARD,                   "Beard, mid"),
    (BEARD_LIT,    step(HEX_BEARD, 1.38),       "Beard, lit lock"),
    (BEARD_DARK,   step(HEX_BEARD, 0.70),       "Beard, outline lock"),
    (SNOW,         HEX_SNOW,                    "Snow / eye white"),
    (TUNIC,        HEX_TUNIC,                   "Tunic, mid field"),
    (TUNIC_LIT,    step(HEX_TUNIC, 1.32),       "Tunic, hem and lit band"),
    (TUNIC_DARK,   step(HEX_TUNIC, 0.72),       "Tunic, fold and sleeve"),
    (PANTS,        HEX_PANTS,                   "Trouser / lantern iron"),
    (PANTS_LIT,    step(HEX_PANTS, 1.40),       "Trouser, lit"),
    (METAL,        HEX_METAL,                   "Metal"),
    (METAL_LIT,    step(HEX_METAL, 1.20),       "Metal, highlight"),
    (METAL_DARK,   step(HEX_METAL, 0.66),       "Metal, underside"),
    (WOOD,         HEX_WOOD,                    "Wood"),
    (WOOD_LIT,     step(HEX_WOOD, 1.26),        "Wood, lit face"),
    (LEATHER,      HEX_LEATHER,                 "Leather, mid"),
    (LEATHER_LIT,  step(HEX_LEATHER, 1.34),     "Leather, cuff and top edge"),
    (LEATHER_DARK, step(HEX_LEATHER, 0.68),     "Leather, sole and shadow"),
    (HAIR,         HEX_HAIR,                    "Hair / dark iron"),
    (HAIR_LIT,     step(HEX_HAIR, 1.55),        "Hair, crown"),
    (FLAME,        HEX_FLAME,                   "Lantern flame"),
    # A scale cannot reach a hot core from #F0A63C without clamping red, and a
    # flame core IS less saturated than its edge, so the mix toward white is the
    # right operation here rather than a workaround for the clamp.
    (FLAME_CORE,   tint(HEX_FLAME, 0.45),       "Lantern flame, hot core"),
]
PALETTE_HEX = [hx for _index, hx, _role in PALETTE]
ROLE_NAMES = [role for _index, _hx, role in PALETTE]
assert [index for index, _hx, _role in PALETTE] == list(range(len(PALETTE)))

# The cell the emissive UV set points at for every quad that does NOT emit. It
# has to be a real, black cell inside the atlas, because emission is sampled
# from the same image the base colour is -- see make_dwarf_material().
EMIT_OFF_CELL = CELLS_PER_ROW * CELLS_PER_ROW - 1        # the last cell, left black
EMISSIVE_COLOURS = {FLAME, FLAME_CORE}

PALETTE_PROVENANCE = (
    "Skin/Beard/Tunic/Metal  label legible on the sheet, swatch agrees\n"
    "Snow/Leather            confirmed: byte-identical in the shipped pine atlas\n"
    "Pants/Wood              UNRESOLVED. The sheet's 8 and B are the same glyph at\n"
    "                        1024 px, which is the sheet's full resolution and not a\n"
    "                        downscale, and the swatches are JPEG-compressed and\n"
    "                        internally noisy. Carried from the brief, NOT confirmed.\n"
    "Hair  #34271C           DERIVED in r2, not sampled clean.\n"
    "Flame #F0A63C           DERIVED in r2, not sampled clean.\n"
    "the other thirteen      VALUE STEPS of the above, by the stated factor. Not\n"
    "                        measurements, and not claimed to be: nothing survives\n"
    "                        being sampled off a 3.4 MB h264 encode of a torch-lit\n"
    "                        scene. What the mp4 settles is that each material has\n"
    "                        SEVERAL values, which is what these cells are for."
)


# ---------------------------------------------------------------------------
# The skeleton
#
# Ruled by Wolf: a skeleton with RIGID weights, one mesh, no separate limb
# meshes, no soft skinning. These names are the contract between this asset and
# every future one -- one animation clip has to drive every variant -- so they
# are declared here and do not move.
#
# Head and tail are in VOXEL coordinates on the pre-centring lattice; they are
# translated and scaled with the body in build_armature(), so a bone cannot
# drift from the geometry it drives.
#
# Axes are Blender's: +X is the dwarf's RIGHT, +Y is the direction he faces,
# +Z is up. `.R` is therefore the +X side.
# ---------------------------------------------------------------------------
BONES = [
    # name          parent        head (x, y, z)     tail (x, y, z)
    ("root",        None,         (0, 0, 0),         (0, 0, 18)),
    ("hips",        "root",       (0, 0, 18),        (0, 0, 42)),
    ("spine",       "hips",       (0, 0, 42),        (0, 0, 50)),
    ("chest",       "spine",      (0, 0, 50),        (0, 0, 58)),
    ("neck",        "chest",      (0, 0, 58),        (0, 0, 64)),
    ("head",        "neck",       (0, 0, 64),        (0, 0, 92)),
    ("beard",       "head",       (0, 4, 74),        (0, 10, 38)),
    ("shoulder.R",  "chest",      (31, 0, 61),       (31, 0, 46)),
    ("elbow.R",     "shoulder.R", (31, 0, 46),       (31, 0, 37)),
    ("hand.R",      "elbow.R",    (31, 0, 37),       (31, 0, 28)),
    ("shoulder.L",  "chest",      (-31, 0, 61),      (-31, 0, 46)),
    ("elbow.L",     "shoulder.L", (-31, 0, 46),      (-31, 0, 37)),
    ("hand.L",      "elbow.L",    (-31, 0, 37),      (-31, 0, 28)),
    # The hip pivots at the TOP OF THE THIGH, not at the hip line. The trouser
    # runs from z 17 up to 31 under the tunic skirt, so a pivot at 21 leaves ten
    # voxels of thigh ABOVE it -- and rigid weights then swing that ten out
    # through the back of the tunic the moment the leg moves. Visible immediately
    # in the first swing render as a detached slab of trouser.
    ("hip.R",       "hips",       (12, 0, 31),       (12, 0, 17)),
    ("knee.R",      "hip.R",      (12, 0, 17),       (12, 0, 11)),
    ("foot.R",      "knee.R",     (12, 0, 11),       (12, 0, 0)),
    ("hip.L",       "hips",       (-12, 0, 31),      (-12, 0, 17)),
    ("knee.L",      "hip.L",      (-12, 0, 17),      (-12, 0, 11)),
    ("foot.L",      "knee.L",     (-12, 0, 11),      (-12, 0, 0)),
]
JOINT_NAMES = [name for name, _p, _h, _t in BONES]

# Where the spine chain hands over, declared ONCE so the geometry and the bones
# cannot disagree about it. Every torso and head column asks this, and the
# mesher breaks its greedy runs on the answer -- which is what makes the
# no-quad-crosses-a-joint rule hold by construction rather than by care.
SPINE_HANDOVER = ((42, "hips"), (50, "spine"), (58, "chest"), (64, "neck"))


def spine_joint(z):
    for top, name in SPINE_HANDOVER:
        if z < top:
            return name
    return "head"


def sided(joint, xc):
    """`.R` for the +X side, `.L` for -X: he faces +Y, so +X is his right."""
    return "%s.%s" % (joint, "R" if xc >= 0 else "L")


# ---------------------------------------------------------------------------
# Sockets
#
# A named anchor voxel coordinate that a swappable part is built against. This
# is what makes "hundreds of variations from twenty authored parts" mechanical
# rather than hopeful: any beard built against `beard` fits any head that
# declares it, BY CONSTRUCTION, and a part that only fits the torso it was drawn
# against cannot happen by accident.
#
# THESE FOUR ARE REAL THIS ROUND -- the parts below are authored relative to
# them and nothing else. The remaining groups (torso, legs, belt, pack) are
# still absolute; they are not swap axes yet and giving them sockets before
# there is a second part to mount would be a plugin system with one plug.
# ---------------------------------------------------------------------------
SOCKETS = {
    "beard":  (0, 4, 74),        # the jaw line; also the `beard` bone's head
    "hair":   (0, 0, 63),        # the skull base, on the head's own axis
    "hand.R": (31, 2, 33),       # the right fist's centre -- the pickaxe hangs here
    "hand.L": (-31, 2, 33),      # the left fist's centre -- the lantern hangs here
}


# ---------------------------------------------------------------------------
# The body
#
# WIDTHS ARE EVEN ON PURPOSE. An odd span cannot centre on the origin and no
# translation can fix it; centre_voxels() asserts that rather than leaving it to
# the GLB check, because the fix is authoring a voxel in or out of a gear extent.
#
# HEIGHTS ARE MEASURED, not invented -- the Z bands come from resampling the
# reference sheet's front orthographic to 96 rows. r3 does not move them; it
# moves what is PAINTED on them and what the outline does at their edges.
#
#   z  0.. 8   boot foot          z 26..62   tunic, hem edge notched
#   z  9..12   boot cuff          z 34..40   belt
#   z 13..16   boot shaft         z 46..74   beard, bottom edge notched
#   z 17..31   trouser            z 63..95   head: skull + hair shell
#   z 72..85   face               z 81..83   brow ledge
#
# THE MID-BODY MOVED IN r3, and it is the round's only band change. Measured off
# the mp4 at t=10 s against its own 10.3 px voxel pitch: the reference belt
# centres at z 40, its tunic hem lands at z 29, and the trouser shows from there
# down to a boot cuff at z 16. r2 put the belt at 20..26 and the hem at 19, so the
# belt sat ON the hem -- there was no skirt, no trouser showed at all, and the
# tunic's hem band had nowhere to be. The boots already agreed with the reference
# and did not move. Stated here because it is a ratified band being changed on the
# authority of the round's own reference, not a quiet retune.
#
# THE FORMS ARE ROUNDED, NOT BOXED. A voxel figure made of rectangular slabs
# greedy-meshes down to almost nothing, because a flat slab face merges into one
# enormous quad however many voxels are behind it. solid() builds every organic
# mass as a superelliptic column whose profile varies with z, so the silhouette
# steps the way the reference's does and the quads follow.
# ---------------------------------------------------------------------------

# Profiles are [(z, half_width_x, y_back, y_front)], piecewise-linear in z.
# half_width_x is a HALF width: the column spans 2*hx voxels, which is why every
# one of these is even without having to say so.
TORSO = [(19, 22, -12, 8), (22, 24, -13, 9), (26, 24, -14, 10), (34, 25, -15, 10),
         (44, 26, -16, 11), (52, 27, -16, 11), (58, 27, -16, 11), (62, 24, -15, 10)]
HEAD = [(63, 17, -15, 13), (66, 19, -17, 15), (72, 20, -18, 16), (84, 20, -18, 16),
        (89, 19, -17, 15), (93, 16, -14, 12), (95, 12, -11, 9)]
# THE BEARD IS SHORTER THAN r2's, AND THAT IS THE ROUND'S BIGGEST SINGLE CHANGE.
# r2's ran from the cheeks to z 30 -- 46 of 96 rows -- so it covered the belt, the
# whole tunic front and most of the face, and the reference's does not: measured
# off the mp4 at t=10 s, the beard tip sits at about 47% of the figure's height,
# which is z 51 here, and the mass is narrower than the torso so the tunic shows
# on both sides of it. Ours reached z 28 and was as wide as the chest.
#
# This is not a taste edit. Round 3's second ask is to paint form with two and
# three palette cells per material, and the tunic's three greens are INVISIBLE
# behind a beard that covers the tunic -- the first render of this round showed
# exactly that. Shortening it is what makes the rest of the round legible.
#
# The top three rows are unchanged, deliberately: they set the front extent, and
# the Y span's parity is balanced against them.
BEARD_P = [(46, 8, 4, 14), (50, 11, 4, 16), (54, 14, 3, 18), (58, 16, 2, 19),
           (62, 18, -2, 19), (70, 19, -8, 19), (74, 18, -12, 17)]
NAPE = [(55, 17, -18, -4), (59, 19, -18, -2), (63, 20, -18, 0)]
BOOT = [(0, 9, -10, 15), (3, 9, -10, 15), (6, 9, -9, 13), (8, 8, -9, 12)]
SOLE = [(0, 9, -10, 15), (1, 9, -10, 15)]
TOECAP = [(2, 8, 6, 14), (5, 8, 6, 13)]
CUFF = [(9, 10, -11, 14), (10, 10, -11, 14)]
SHAFT = [(11, 9, -9, 10), (16, 8, -8, 9)]
THIGH = [(17, 8, -8, 9), (22, 9, -9, 9), (31, 11, -10, 10)]
LEG_X = 12                       # leg centre; mirrored to -12 for the other side

UPPER_ARM = [(46, 5, -9, 5), (52, 6, -10, 6), (58, 7, -11, 7), (62, 7, -11, 7)]
FOREARM = [(38, 5, -8, 7), (41, 5, -9, 6), (46, 5, -9, 5)]
FIST = [(28, 5, -8, 9), (32, 6, -9, 10), (37, 5, -8, 9)]
ARM_X = 31                       # arm centre, just clear of the torso's 27

PACK = [(33, 13, -35, -16), (37, 16, -37, -15), (48, 17, -38, -15),
        (57, 16, -37, -15), (62, 13, -35, -16)]
BEDROLL = [(63, 16, -34, -20), (66, 17, -35, -20), (69, 15, -33, -21)]


# --- the silhouette tables -------------------------------------------------
# THIS IS WHERE R3'S DETAIL BUDGET GOES, and the numbers are coarse ON PURPOSE.
#
# A 3-voxel step in an outline survives a nearest-neighbour downscale to 10 px
# as a readable notch. A 1-voxel surface channel does not survive it at all --
# it ALIASES, and an aliasing surface shimmers as the camera moves. With free
# zoom the camera passes through every size between 8.7 px and full frame, so
# the same fix has to serve both ends: big value steps and silhouette notches,
# which is exactly what the reference uses.
#
# Each table is (|x| band start, value); band() reads the last entry whose start
# is <= |x|. Purely a function of x, so it stays deterministic.
BEARD_LOCKS = ((0, 46), (4, 52), (8, 48), (12, 55), (16, 50))
HAIR_CROWN = ((0, 95), (4, 93), (8, 94), (12, 92), (16, 90))
HAIR_LINE = ((0, 63), (15, 68), (17, 65), (19, 71))
TUNIC_HEM = ((0, 25), (5, 28), (10, 23), (15, 27), (20, 24), (24, 29))
BOOT_CUFF_TOP = ((0, 11), (4, 10), (7, 12))


def band(table, value):
    """The last entry in a |x| band table whose start is <= abs(value)."""
    out = table[0][1]
    for start, result in table:
        if abs(value) >= start:
            out = result
    return out


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


def solid(put, table, z0, z1, colour, part, joint, power=3.0, xc=0, floor=None, ceiling=None):
    """Fill a superelliptic column whose cross-section follows `table`.

    `xc` is the column's centre and MUST be an integer: the x cells run
    [xc - hx, xc + hx - 1], which is 2*hx wide -- even -- only because the centre
    sits on a lattice boundary rather than on a voxel. Putting it on a voxel
    centre would make every limb odd-width and put the whole model half a voxel
    off the project grid.

    `power` shapes the corner: 2 is an ellipse, 3 a rounded box, higher
    approaches the slab that greedy-meshes into nothing.

    `colour` and `joint` may each be a callable. `colour(sx, y, z, hx, yb, yf)`
    receives sx = x - xc + 0.5, the column's own HALF-INTEGER offset, and
    `floor`/`ceiling` are callables of the same sx returning the lowest/highest z
    the column keeps -- that is how a lock, a hem or a cuff notches the OUTLINE
    rather than the surface. `joint(z)` receives the absolute height.

    THE HALF IS NOT COSMETIC. The column spans [xc - hx, xc + hx - 1], so its
    true centre is the lattice boundary xc, not a voxel; the signed distance to
    it is x - xc + 0.5 and only that is exact under the mirror x -> -1 - x. Using
    the integer x - xc instead puts a mirrored limb's bands one voxel out on one
    side, which shows as a figure whose two boots notch differently.
    """
    for z in range(z0, z1 + 1):
        hx, yb, yf = sample(table, z)
        hx = int(round(hx))
        yb, yf = int(round(yb)), int(round(yf))
        if hx < 1 or yf < yb:
            continue
        cy = (yb + yf) / 2.0
        hy = (yf - yb + 1) / 2.0
        jz = joint(z) if callable(joint) else joint
        for x in range(xc - hx, xc + hx):
            sx = x - xc + 0.5
            if floor is not None and z < floor(sx):
                continue
            if ceiling is not None and z > ceiling(sx):
                continue
            u = abs(sx / hx)
            for y in range(yb, yf + 1):
                v = abs((y - cy) / hy)
                if u ** power + v ** power <= 1.0:
                    c = colour(sx, y, z, hx, yb, yf) if callable(colour) else colour
                    put(x, y, z, c, part, jz)


# ---------------------------------------------------------------------------
# The parts
# ---------------------------------------------------------------------------
def build_voxels(beard_name="miner-full", hair_name="miner-shag"):
    """Return ({(x,y,z): colour}, {(x,y,z): part}, {(x,y,z): joint}).

    Either swappable part may be None, which mounts nothing on that socket. That
    is not a feature for its own sake -- it is how the round's removability claim
    is CHECKED: main() builds the body without the beard, and again without the
    hair, and requires every remaining voxel to be identical to the full build's.
    A beard that carved into the head, or a hairline that only works with this
    hair, fails there rather than in a render nobody compares.
    """
    vox, part_of, joint_of = {}, {}, {}

    def put(x, y, z, colour, part, joint):
        vox[(x, y, z)] = colour
        part_of[(x, y, z)] = part
        joint_of[(x, y, z)] = joint

    def put_empty(x, y, z, colour, part, joint):
        """Place only where nothing is: how a shell wraps a core it must not eat."""
        if (x, y, z) not in vox:
            put(x, y, z, colour, part, joint)

    def fill(x0, x1, y0, y1, z0, z1, colour, part, joint):
        for x in range(x0, x1 + 1):
            for y in range(y0, y1 + 1):
                for z in range(z0, z1 + 1):
                    put(x, y, z, colour, part, joint)

    def both(table, z0, z1, colour, part, joint, power=3.0, xc=0, floor=None, ceiling=None):
        """A limb and its reflection. x -> -1-x is exact on this lattice, and the
        joint takes the side with it."""
        for sign in (1, -1):
            solid(put, table, z0, z1, colour, part, sided(joint, sign), power, sign * xc,
                  floor, ceiling)

    def mfill(x0, x1, y0, y1, z0, z1, colour, part, joint, sided_joint=False):
        """A box and its reflection, for fittings that come in pairs."""
        fill(x0, x1, y0, y1, z0, z1, colour, part,
             sided(joint, 1) if sided_joint else joint)
        fill(-1 - x1, -1 - x0, y0, y1, z0, z1, colour, part,
             sided(joint, -1) if sided_joint else joint)

    def front_of(xs, zs):
        """Frontmost occupied y for each (x, z) in the given ranges."""
        xs, zs = set(xs), set(zs)
        cols = {}
        for (x, y, z) in vox:
            if x in xs and z in zs and cols.get((x, z), -10 ** 6) < y:
                cols[(x, z)] = y
        return cols

    def paint_front(xs, zs, colour, part, joint, mirror=False):
        """Recolour the frontmost voxel of each (x, z) column.

        The bodies here are rounded, so their front plane moves with x and z and
        a box fill would either sink inside them or float off them. This lands on
        the surface whatever the profile does -- which is also why a seam placed
        this way can never change the silhouette, the extents or the centring.
        """
        for (x, z), y in front_of(xs, zs).items():
            put(x, y, z, colour, part, joint(z) if callable(joint) else joint)
        if mirror:
            paint_front([-1 - x for x in xs], zs, colour, part, joint)

    build_legs(both, mfill)
    build_torso(put, both, fill, paint_front)
    build_arms(both)
    build_pack(put, mfill, fill)
    build_head(put, put_empty, front_of, HAIRS.get(hair_name))
    if beard_name is not None:
        BEARDS[beard_name](put, front_of, SOCKETS["beard"])
    build_pickaxe(fill, SOCKETS["hand.R"])
    build_lantern(fill, SOCKETS["hand.L"])
    return vox, part_of, joint_of


# --- legs ------------------------------------------------------------------
# Short and thick: the front ortho gives the legs the bottom fifth of the
# figure, which is what makes the head and the beard read as big.
#
# The boot is TWO leathers and the split is the silhouette's, not a pattern's:
# a dark shaft under a light cuff whose top edge STEPS across the boot, which is
# the read the mp4's boots have at any size. r2 put three one-voxel lacing bands
# there instead and they dissolve into speckle below about 40 px.
def build_legs(both, mfill):
    def shaft_colour(sx, y, z, hx, yb, yf):
        return LEATHER_DARK if z < 3 or abs(sx) >= hx - 1 else LEATHER

    def cuff_colour(sx, y, z, hx, yb, yf):
        return LEATHER_LIT if z >= band(BOOT_CUFF_TOP, sx) - 1 else LEATHER

    both(BOOT, 0, 8, shaft_colour, "leg", "foot", 3.0, LEG_X)
    both(SOLE, 0, 1, LEATHER_DARK, "leg", "foot", 3.0, LEG_X)
    both(TOECAP, 2, 5, WOOD_LIT, "leg", "foot", 3.0, LEG_X)
    # The cuff: a light leather turn-down whose TOP EDGE is the notch. It is two
    # to three voxels deep so the step survives a downscale.
    both(CUFF, 9, 12, cuff_colour, "leg", "foot", 3.0, LEG_X,
         ceiling=lambda sx: band(BOOT_CUFF_TOP, sx))
    both(SHAFT, 11, 16, LEATHER, "leg", "knee", 3.0, LEG_X,
         floor=lambda sx: band(BOOT_CUFF_TOP, sx) + 1)
    both(THIGH, 17, 31, lambda sx, y, z, hx, yb, yf: PANTS_LIT if y >= yf - 1 else PANTS,
         "leg", "hip", 3.0, LEG_X)
    mfill(17, 20, -3, 2, 9, 10, METAL, "leg", "foot", sided_joint=True)   # outboard buckle


# --- torso -----------------------------------------------------------------
# The tunic is short and belted low. Three greens do the work r2 asked one green
# and a groove pass to do: a LIGHT hem band that steps across the bottom edge, a
# MID field, and a DARK fold down each side and across the shoulders. That is the
# mp4's read -- lighter body, darker sleeves, a pale band at the hem -- and every
# one of the three survives being seen from ten metres, which a fold groove does
# not.
def build_torso(put, both, fill, paint_front):
    def tunic_colour(sx, y, z, hx, yb, yf):
        """Three greens, assigned by FORM and in this order.

        The order matters and the first cut had it wrong: paint the hem band last
        and the side fold eats it at both ends of the hem, so the step that the
        hem table cuts into the outline stops being visible as a band. The read
        the mp4 has is a lighter body, darker sleeves and shoulders, and a pale
        band at the hem -- all three of which survive a downscale to 30 px, which
        a fold groove does not.
        """
        if z <= band(TUNIC_HEM, sx) + 2:
            return TUNIC_LIT                       # the hem band, following its own step
        if z >= 56 or abs(sx) >= hx - 2:
            return TUNIC_DARK                      # shoulders, and the fold down each side
        if y >= yf - 1 and abs(sx) <= hx - 6:
            return TUNIC_LIT                       # the front plane, square to the light
        return TUNIC

    solid(put, TORSO, min(v for _s, v in TUNIC_HEM), 62, tunic_colour, "torso", spine_joint,
          2.5, floor=lambda sx: band(TUNIC_HEM, sx))

    # Collar: a dark band closing the neck hole, two voxels so it reads at size.
    collar = [(z, hx - 2, yb + 2, yf - 2) for (z, hx, yb, yf) in TORSO]
    solid(put, collar, 61, 62, LEATHER_DARK, "torso", spine_joint, 2.5)

    # Belt: a full leather band, two leathers deep -- lit along its top edge,
    # dark below it. A buckle PLATE stands two voxels proud of the band, which is
    # what makes it a shape rather than a painted face: it notches the profile.
    belt = [(z, hx + 1, yb - 1, yf + 1) for (z, hx, yb, yf) in TORSO]
    solid(put, belt, 34, 40, lambda sx, y, z, hx, yb, yf: LEATHER_LIT if z >= 39 else LEATHER,
          "belt", "hips")
    fill(-3, 2, 10, 13, 34, 40, METAL_LIT, "belt", "hips")       # buckle plate, proud
    fill(-1, 0, 12, 14, 36, 38, METAL_DARK, "belt", "hips")      # its tongue slot
    fill(-2, 1, 11, 13, 29, 33, LEATHER_DARK, "belt", "hips")    # strap tail below it

    # A belt pouch on his left and a sheathed knife on his right: silhouette
    # again, both proud of the belt line and both readable as lumps at 30 px.
    fill(-30, -21, 6, 13, 32, 40, LEATHER, "belt", "hips")
    fill(-28, -23, 13, 14, 38, 39, METAL, "belt", "hips")
    fill(21, 24, 7, 13, 22, 35, LEATHER_DARK, "belt", "hips")
    fill(21, 24, 7, 13, 36, 38, METAL, "belt", "hips")
    fill(20, 25, 6, 14, 33, 34, LEATHER, "belt", "hips")

    # Tunic seams framing the front panel. Painted on the surface, so they cost
    # nothing in the outline -- and they are TWO voxels wide, because a one-voxel
    # seam is the r2 groove by another name.
    paint_front(range(13, 15), range(42, 56), TUNIC_DARK, "torso", spine_joint, mirror=True)


# --- arms ------------------------------------------------------------------
# Sleeve to the elbow, a light leather bracer cuff where it ends, bare forearm
# below, fist at the end. The reference has no gloves and it does have bracers.
def build_arms(both):
    both(UPPER_ARM, 46, 62, TUNIC_DARK, "arm", "shoulder", 2.5, ARM_X)
    both(UPPER_ARM, 44, 45, LEATHER_LIT, "arm", "elbow", 2.5, ARM_X)     # bracer cuff
    both(FOREARM, 38, 43, lambda sx, y, z, hx, yb, yf: SKIN_SHADE if abs(sx) >= hx - 1 else SKIN,
         "arm", "elbow", 2.5, ARM_X)
    both(FIST, 28, 37, lambda sx, y, z, hx, yb, yf: SKIN_SHADE if z <= 30 else SKIN,
         "arm", "hand", 2.5, ARM_X)


# --- backpack --------------------------------------------------------------
# Hung off the back of the torso with a rolled bedroll strapped across its top.
# Never appears in a front render and is not meant to; it is what the mp4's swing
# frames show, and the swing is what the reference is judged on.
def build_pack(put, mfill, fill):
    solid(put, PACK, 33, 62,
          lambda sx, y, z, hx, yb, yf: LEATHER_DARK if abs(sx) >= hx - 2 else LEATHER,
          "pack", "chest", 3.6)
    solid(put, BEDROLL, 63, 69, LEATHER_LIT, "pack", "chest", 2.0)
    fill(-12, 11, -39, -38, 42, 56, LEATHER_DARK, "pack", "chest")       # flap
    fill(-16, 15, -39, -38, 36, 39, LEATHER_DARK, "pack", "chest")       # cinch strap
    for sx in (-1, 1):
        fill(min(3 * sx, 7 * sx), max(3 * sx, 7 * sx), -40, -39, 42, 56,
             LEATHER_DARK, "pack", "chest")
    # The pack buckle is TWO voxels proud, not one. It is also what makes the Y
    # span even: the beard reaches +22 and the flap -40, which is 63 columns and
    # cannot centre on the origin. centre_voxels() says exactly that and says to
    # author a voxel of gear in or out on the axis, so this is that voxel -- and
    # a buckle that stands off its strap is the right place to spend it.
    fill(-4, 3, -41, -40, 45, 51, METAL_LIT, "pack", "chest")            # buckle
    for sx in (-1, 1):
        fill(min(6 * sx, 13 * sx), max(6 * sx, 13 * sx), -16, -15, 55, 62,
             LEATHER_DARK, "pack", "chest")                              # shoulder straps
    mfill(17, 19, -34, -24, 38, 48, LEATHER, "pack", "chest")            # side pockets
    mfill(5, 8, -36, -19, 63, 69, HAIR, "pack", "chest")                 # bedroll binding
    for stud_z in (45, 50, 55):
        mfill(9, 10, -40, -39, stud_z, stud_z + 1, METAL, "pack", "chest")
    for coil_z in (50, 54, 58):
        mfill(16, 19, -33, -23, coil_z, coil_z + 1, LEATHER_LIT, "pack", "chest")


# --- head ------------------------------------------------------------------
# THE HEAD IS A COMPLETE SKIN SKULL AND THE HAIR IS A SHELL OVER IT. That is the
# round's ask, stated as a falsifiable property: removing every voxel of the
# `hair` part has to leave a complete head underneath -- no skin borrowed from
# the hair's volume, no hairline that only works with this hair. Same for the
# beard. build_voxels asserts it in main() rather than claiming it here.
#
# The skull is HEAD shrunk by one voxel; the hair is HEAD placed only where the
# skull is not, which makes a one-voxel shell over the cranium and a two-voxel
# cap at the crown. The face window is simply where the shell is not placed.
FACE_HALF = 15                  # the face window's half width; the hair frames it beyond
HAIRLINE = {84: 13, 85: 11}     # it narrows as it rises, so the hairline rounds
FACE_Z = (72, 85)               # where SKIN is painted: cheek-line to hairline
# Where NO HAIR may stand in front of the head. It reaches further down than
# FACE_Z because the nose and the mouth are built onto whatever is frontmost, and
# a hair voxel sitting at z 70 in front of the skull moved the nose forward by
# one -- so the nose's position depended on the hair being mounted, which is the
# exact coupling the swap axis is not allowed to have. Caught by the check, not
# by looking.
FACE_WINDOW_Z = (63, 85)
EYE_Z = (75, 80)
BROW_Z = (81, 83)
BROW_HALF = 11                  # narrower than the face: skin passes it at the temples
NOSE_Z = (70, 80)
# The skull and the nape are the HEAD and NAPE profiles shrunk by one voxel, and
# they stop at z 88 rather than at the crown. That is what leaves room for the
# hair: a one-voxel shell around the sides and a solid cap above 88, placed only
# into air. Stopping the skull at the crown instead would leave a ring of bare
# scalp wherever the crown's step cuts the hair back.
SKULL = [(z, hx - 1, yb + 1, yf - 1) for (z, hx, yb, yf) in HEAD]
SKULL_TOP = 88
NAPE_CORE = [(z, hx - 1, yb + 1, yf - 1) for (z, hx, yb, yf) in NAPE]


def face_half(z):
    return min(FACE_HALF, HAIRLINE.get(z, FACE_HALF))


def in_face_window(x, z):
    half = face_half(z)
    return FACE_WINDOW_Z[0] <= z <= FACE_WINDOW_Z[1] and -half <= x < half


def build_head(put, put_empty, front_of, hair_builder):
    solid(put, SKULL, 63, SKULL_TOP, SKIN, "head", spine_joint, 2.6)
    solid(put, NAPE_CORE, 55, 63, SKIN_SHADE, "head", spine_joint)
    if hair_builder is not None:
        hair_builder(put_empty, SOCKETS["hair"])
    build_face(put, front_of)


def hair_miner_shag(put_empty, socket):
    """The mp4's shag: a one-voxel shell over the cranium with a two-voxel cap,
    a CROWN cut into steps and a HAIRLINE that steps across the temples.

    Both notches are in the outline, both are 2-3 voxels deep, and both are
    against the background at every camera angle -- which is the whole point.
    The crown is the one silhouette feature that is visible at 10 px from any
    direction, because nothing of his is ever in front of the top of his head.

    Mounted on the `hair` socket and weighted entirely to `head`, and shaped so
    that removing it leaves a complete skull: it is placed only into air, never
    over the skin. A second and a third hair are a new function with this
    signature and a new key in HAIRS, and nothing else -- which is the point.
    """
    _ox, _oy, oz = socket                        # this hair sits on the head's own axis
    top = oz + 32                                # z 95: the figure's ceiling

    def hair_colour(sx, y, z, hx, yb, yf):
        return HAIR_LIT if z >= top - 5 or y <= yb + 1 else HAIR

    def put_hair(x, y, z, colour, part, joint):
        # Two cut-outs, and nothing else removes hair. The face window is where
        # the hair is simply not placed -- that is what leaves a complete skull
        # underneath instead of a hairline that only works with this hair.
        if in_face_window(x, z) and y > 0:
            return
        if y > -2 and z < band(HAIR_LINE, x + 0.5):
            return
        put_empty(x, y, z, colour, part, joint)

    solid(put_hair, HEAD, oz, top, hair_colour, "hair", "head", 2.6,
          ceiling=lambda sx: band(HAIR_CROWN, sx))
    # The nape's shell, so the hair reads as a mass from behind as well.
    solid(put_hair, NAPE, oz - 8, oz, HAIR, "hair", "head")


def beard_miner_full(put, front_of, socket):
    """The mp4's full beard: a mass whose BOTTOM EDGE is cut into locks of five
    different lengths, dark at the outline and lit on the upper front lobes,
    plus a moustache and sideburns painted proud of the skull.

    One part, one socket, one joint (`beard`). Removing every voxel of it leaves
    a complete skin head -- the moustache and the sideburns are PROUD of the
    skull's front surface and do not replace it.
    """
    ox, oy, oz = socket
    del ox, oy

    def beard_colour(sx, y, z, hx, yb, yf):
        if abs(sx) >= hx - 1 or z <= band(BEARD_LOCKS, sx) + 2:
            return BEARD_DARK                    # the outline, and the tip of each lock
        if z >= 56 and y >= yf - 2:
            return BEARD_LIT                     # the upper front lobes catch the light
        return BEARD

    solid(put, BEARD_P, min(v for _s, v in BEARD_LOCKS), oz, beard_colour, "beard", "beard",
          2.2, floor=lambda sx: band(BEARD_LOCKS, sx))

    # Moustache, closing the gap between the nose and the beard so the face does
    # not float on a brown field. Two voxels proud, three through its centre, lit
    # on top. The third voxel is also the one that makes the Y span even -- the
    # flap reaches -41 and this reaches +22 -- and a moustache that stands off
    # the beard is the right place to spend it: it is the nearest thing on the
    # face to the camera, so it is the one that still steps at 30 px.
    for z in range(68, 72):
        for (x, _z), y in front_of(range(-11, 11), [z]).items():
            put(x, y + 1, z, BEARD, "beard", "beard")
            put(x, y + 2, z, BEARD_LIT if z >= 70 else BEARD, "beard", "beard")
            if abs(x + 0.5) < 7 and 69 <= z <= 70:
                put(x, y + 3, z, BEARD_LIT, "beard", "beard")

    # The mouth line, cut into the beard UNDER the moustache. It belongs to this
    # part and not to the face: the beard mass stands four voxels proud of the
    # skull at the jaw, so the head's own mouth (which build_face paints, and
    # which a shorter beard would expose) is behind it and invisible while this
    # beard is mounted. Two beards, two mouths, one head that is complete either
    # way -- which is what the socket contract is for.
    for (x, z), y in front_of(range(-6, 6), range(66, 68)).items():
        put(x, y + 1, z, BEARD_DARK, "beard", "beard")

    # Sideburns, rising to eye level and framing the face down to the beard.
    for z in range(64, 80):
        xs = list(range(9, FACE_HALF)) + list(range(-FACE_HALF, -9))
        for (x, _z), y in front_of(xs, [z]).items():
            put(x, y + 1, z, BEARD_DARK if abs(x) >= 13 else BEARD, "beard", "beard")


def build_face(put, front_of):
    """Painted onto the skull's FRONT SURFACE rather than at absolute coordinates,
    because the skull is a rounded column and its front plane moves with z and x.

    This is what a 12-voxel budget could not buy and 96 can: a brow ledge stepped
    two voxels proud with the eyes in its shadow, whites with pupils set apart by
    a nose bridge, and a nose that projects past the brow.
    """
    # INCLUSIVE of FACE_Z[1]. range(*FACE_Z) stops one short, which left z 85
    # falling back to the full +-15 and painting cheek skin out over the
    # hairline -- caught by the swap-axis check, because that skin then differed
    # depending on whether the hair was mounted.
    window = {z: range(-face_half(z), face_half(z))
              for z in range(FACE_Z[0], FACE_Z[1] + 1)}

    def surface(xs, zs):
        return front_of(xs, zs)

    # The cheeks fall away from the nose: two skins, not one flat cream field.
    for z in range(FACE_Z[0], FACE_Z[1] + 1):
        for (x, _z), y in surface(window.get(z, range(-FACE_HALF, FACE_HALF)), [z]).items():
            if abs(x) >= 10:
                put(x, y, z, SKIN_SHADE, "head", "head")

    # Brow ledge: two voxels proud, dark, with bare skin above it. The eyes below
    # then read as sunk in its shadow -- the single biggest read on this face at
    # any size above about 30 px.
    for z in range(BROW_Z[0], BROW_Z[1] + 1):
        for (x, _z), y in surface(range(-BROW_HALF, BROW_HALF), [z]).items():
            put(x, y + 1, z, HAIR, "head", "head")
            put(x, y + 2, z, HAIR, "head", "head")

    def eye(x_range, z_range, colour):
        for x in x_range:
            for z in z_range:
                for cell in (x, -1 - x):                # the reflection is exact
                    found = surface([cell], [z])
                    if (cell, z) in found:
                        put(cell, found[(cell, z)] + 1, z, colour, "head", "head")

    eye(range(2, 8), range(EYE_Z[0], EYE_Z[1] + 1), SNOW)
    eye(range(3, 7), range(EYE_Z[0] + 1, EYE_Z[1]), HAIR)

    # Nose: a bridge between the eyes, then a tip that projects past the brow. It
    # is the only part of the face proud of the brow ledge.
    for z in range(NOSE_Z[0], NOSE_Z[1] + 1):
        reach = 3 if z >= 76 else 4
        half = 2 if z >= 76 else 3
        for (x, _z), y in surface(range(-half, half), [z]).items():
            for d in range(1, reach + 1):
                put(x, y + d, z, SKIN if d < reach else SKIN_SHADE, "head", "head")

    # A mouth: two voxels of shadow, which is all a mouth needs at this scale.
    # It is on the SKULL and belongs to the head, so a head wearing a shorter
    # beard still has one; the mounted beard cuts its own over the top.
    for (x, z), y in surface(range(-6, 6), range(66, 68)).items():
        put(x, y + 1, z, HAIR, "head", "head")


# --- pickaxe, on the hand.R socket -----------------------------------------
# 0.83x dwarf height, which is the sheet's label: 80 voxels from butt to the top
# of the head. Carried upright, head level with the top of his hair -- NOT above
# it. The bounding box is his height, so a tool that outtops him would make a
# 1.20 m dwarf 1.10 m of dwarf under a raised pick.
#
# Authored against the socket and weighted entirely to `hand.R`, so splitting it
# into its own asset later is a file move rather than a remodel.
HAFT_HALF = 3                    # the haft is 6 voxels across
HAFT_DEPTH = 3
HAFT_OUT = 8                     # and stands this far outboard of the socket, clear of the fist


def build_pickaxe(fill, socket):
    ox, oy, _oz = socket
    x0, x1 = ox + HAFT_OUT - HAFT_HALF, ox + HAFT_OUT + HAFT_HALF - 1
    y0, y1 = oy + 5 - HAFT_DEPTH, oy + 5 + HAFT_DEPTH - 1
    fill(x0, x1, y0, y1, 15, 92,
         WOOD, "pickaxe", "hand.R")                              # haft, butt at z 15
    fill(x0, x1, y1, y1, 15, 92, WOOD_LIT, "pickaxe", "hand.R")  # its lit face
    fill(x0, x1, y0, y1, 15, 17, METAL_DARK, "pickaxe", "hand.R")   # butt cap
    fill(x0, x1, y0, y1, 28, 31, LEATHER_DARK, "pickaxe", "hand.R")  # grip wrap
    fill(x0, x1, y0, y1, 34, 37, LEATHER_DARK, "pickaxe", "hand.R")
    fill(x0 - 1, x1 + 1, y0 - 1, y1 + 1, 82, 84, METAL, "pickaxe", "hand.R")   # ferrule
    fill(x0 - 2, x1 + 2, y0 - 1, y1 + 1, 86, 92, METAL, "pickaxe", "hand.R")   # boss
    # Wings, stepping out and down from the boss and mirrored about the HAFT's own
    # centre plane, not the body's. Both wings matter: a bar with a cap reads as a
    # hammer head-on, which is what v1's did.
    for span, ztop, zbot in ((3, 92, 89), (6, 91, 87), (9, 89, 85),
                             (11, 87, 84), (13, 85, 83)):
        for u0 in (x1 + span - 2, x0 - span):
            fill(u0, u0 + 2, y0, y1, zbot, ztop,
                 METAL_LIT if ztop >= 89 else METAL, "pickaxe", "hand.R")


# --- lantern, on the hand.L socket -----------------------------------------
# 0.33x dwarf height, the sheet's label: 32 voxels from base to the top of the
# ring. Hung from the fist, so its ring is at the hand and the body swings just
# below -- slung lower it reads as a box on the floor.
#
# It reads as a lantern because it has a CAGE: dark corner posts and a centre
# mullion standing in front of warm panes. The panes are the asset's only
# EMISSIVE cells (Wolf, B: yes), so the lantern is the brightest thing in frame
# the way it is in the mp4. Making it LIGHT THE SCENE is a point light in the
# client and is not this round's work.
LANT_HALF = 9                    # 18 voxels across, centred on the socket
LANT_DEPTH = 9


def build_lantern(fill, socket):
    ox, oy, _oz = socket
    x0, x1 = ox - 5 - LANT_HALF, ox - 5 + LANT_HALF - 1
    y0, y1 = oy + 7 - LANT_DEPTH, oy + 7 + LANT_DEPTH - 1
    j = "hand.L"
    fill(x0 + 6, x1 - 6, y0 + 6, y1 - 6, 33, 37, METAL, "lantern", j)     # ring, in the fist
    fill(x0 + 2, x1 - 2, y0 + 2, y1 - 2, 29, 32, PANTS, "lantern", j)     # peaked cap
    fill(x0, x1, y0, y1, 26, 28, PANTS_LIT, "lantern", j)                 # cap brim, lit
    fill(x0 + 1, x1 - 1, y0 + 1, y1 - 1, 11, 25, FLAME, "lantern", j)     # glass, four faces
    # The hot core goes on the OUTER SKIN of each pane, not inside the glass
    # volume: a colour buried under another voxel has no face, ships no
    # triangle and reaches no render. The first cut of this put it at the
    # lantern's centre and it was invisible in every one of the five views.
    for u0, u1, v0, v1 in ((x0 + 1, x0 + 1, y0 + 1, y1 - 1), (x1 - 1, x1 - 1, y0 + 1, y1 - 1),
                           (x0 + 1, x1 - 1, y0 + 1, y0 + 1), (x0 + 1, x1 - 1, y1 - 1, y1 - 1)):
        fill(u0, u1, v0, v1, 15, 21, FLAME_CORE, "lantern", j)
    fill(x0 + 4, x1 - 4, y0 + 4, y1 - 4, 11, 13, PANTS, "lantern", j)     # dark body under it
    for u0, v0 in ((x0 + 1, y0 + 1), (x1 - 3, y0 + 1), (x0 + 1, y1 - 3), (x1 - 3, y1 - 3)):
        fill(u0, u0 + 2, v0, v0 + 2, 11, 25, PANTS, "lantern", j)         # corner posts
    fill(x0 + 8, x1 - 8, y0 + 1, y1 - 1, 11, 25, PANTS, "lantern", j)     # mullion, X faces
    fill(x0 + 1, x1 - 1, y0 + 8, y1 - 8, 11, 25, PANTS, "lantern", j)     # mullion, Y faces
    fill(x0, x1, y0, y1, 7, 10, PANTS, "lantern", j)                      # base
    fill(x0 + 1, x1 - 1, y0 + 1, y1 - 1, 5, 6, METAL, "lantern", j)       # foot rim


# The swap registries. One entry each this round -- and the signature is the
# contract, so the second and the third are a function and a dict key.
BEARDS = {"miner-full": beard_miner_full}
HAIRS = {"miner-shag": hair_miner_shag}


# ---------------------------------------------------------------------------
# Poses
#
# Rigid weights mean the limbs rotate as solid blocks, which is what the mp4
# does -- look at t=1/4/7/10 s. Angles are degrees about the bone's LOCAL axes,
# and the table below says what that means in world terms for each chain; the
# axes themselves come out of `pose_bone.x_axis / y_axis / z_axis`, which is
# worth printing once before writing any new pose rather than reasoning about
# Blender's bone roll.
#
# `swing` is the mp4's t=4 s frame: the torso folded over the swing, the arms
# carried through and down, the front leg braced. It is delivered because the
# swing is what the reference is judged on and because a rig nobody has bent is
# a rig nobody has tested.
# ---------------------------------------------------------------------------
POSES = {
    "neutral": {},
    # The mp4's t=4 s frame: torso folded over the swing, both arms carried
    # through and down onto the work, front leg braced, head up enough to still
    # be looking at it.
    #
    # ANGLES ARE ABOUT EACH BONE'S LOCAL X, AND EVERY BONE HERE HAS LOCAL X =
    # WORLD +X, which is the one fact that makes this table readable. Verified
    # rather than assumed -- the first cut of this pose had the sign inverted and
    # threw both arms out behind him. The signs then follow from the rest
    # direction, and they are OPPOSITE for the two chains:
    #
    #   spine chain (bones point UP):   negative bends FORWARD  (+Y)
    #   limb chains (bones point DOWN): positive swings FORWARD (+Y)
    #
    # Rotations compose down the chain, so the comment on each line is the
    # ABSOLUTE world pitch it lands at -- that is what a reader wants and what
    # the local number does not say.
    "swing": {
        "hips":       (-12.0, 0.0, 0.0),      # -12   pelvis tips into the swing
        "spine":      (-20.0, 0.0, 0.0),      # -32
        "chest":      (-20.0, 0.0, 0.0),      # -52   the fold, and it is the chest's
        "neck":       (18.0, 0.0, 0.0),       # -34
        "head":       (18.0, 0.0, 0.0),       # -16   still looking at the work
        "beard":      (20.0, 0.0, 0.0),       #       hangs, instead of sticking out
        "shoulder.R": (100.0, 0.0, -12.0),    # +48   the pickaxe arm, carried through
        "elbow.R":    (-22.0, 0.0, 0.0),      # +26
        "hand.R":     (-8.0, 0.0, 0.0),       # +18
        "shoulder.L": (82.0, 0.0, 12.0),      # +30   the lantern arm, trailing
        "elbow.L":    (-16.0, 0.0, 0.0),      # +14
        "hip.R":      (24.0, 0.0, 0.0),       # +12   front leg, braced
        "knee.R":     (-16.0, 0.0, 0.0),      # -4
        "foot.R":     (-8.0, 0.0, 0.0),       # -12
        "hip.L":      (-18.0, 0.0, 0.0),      # -30   back leg, driving
        "knee.L":     (12.0, 0.0, 0.0),       # -18
        "foot.L":     (10.0, 0.0, 0.0),       # -8
    },
}


# ---------------------------------------------------------------------------
# Centring
# ---------------------------------------------------------------------------
def centre_voxels(vox, part_of, joint_of):
    """Translate the lattice so the bounding box centres on X and Y.

    A box spanning [min, max] in voxels occupies [min, max+1] in lattice units,
    so it centres on the origin exactly when max == -min - 1 -- which requires an
    EVEN span, and no translation can fix an odd one. That is asserted here
    rather than left to the GLB check, because the fix is authoring a voxel in or
    out of a gear extent and the assertion is what says which axis to look at.

    Z is never translated -- his feet are on the ground at z 0.
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
    moved = {(x + dx, y + dy, z): c for (x, y, z), c in vox.items()}
    moved_parts = {(x + dx, y + dy, z): p for (x, y, z), p in part_of.items()}
    moved_joints = {(x + dx, y + dy, z): j for (x, y, z), j in joint_of.items()}
    return moved, moved_parts, moved_joints, (dx, dy)


# ---------------------------------------------------------------------------
# Falsifiable properties of the voxel body
# ---------------------------------------------------------------------------
# A groove column runs the height of the mass it is cut into, so it is notched in
# many z rows. An honest notch -- the crease where an arm meets the torso, the gap
# between the boots -- can be notched in many rows too, which is why the count
# alone settles nothing and the SPACING has to. Measured on this body: the clean
# figure has four columns notched in six rows or more, at x -26, 20, 25 and 35,
# and no stride relates them; r2's pass on the beard adds nine at a stride of 3.
CHANNEL_MIN_ROWS = 6            # z rows a column must be notched in to be a channel
CHANNEL_MIN_RUN = 4             # channels in an unbroken run at one stride before it counts
CHANNEL_PURITY = 0.8            # share of the channels the run spans that must be the run's


def find_repeating_channel(vox):
    """Look for r2's groove pass: a one-voxel channel cut down every Nth column.

    The round's first ask is falsifiable on delivery -- "no 1-voxel channel
    repeating at a fixed step anywhere on the asset" -- so it is checked rather
    than asserted in prose. Returns the first `(side, stride)` that looks like a
    groove pass, or None.

    THE TEST IS REGULAR SPACING. Getting there took three versions, and the two
    that were wrong both called a deliberately grooved body clean:

      * grouping columns by (colour, z) cannot see r3's OWN beard grooved. The
        beard carries three cells now, so a channel cut through its lit front
        leaves no lit voxel in that column rather than a lowered one, and a
        per-colour row simply has a gap where the notch should be;
      * scoring "what share of ALL notched columns sit at this stride" cannot see
        it either. The figure has honest notches everywhere, so a groove over one
        mass is a minority of the body's notches however regular it is.

    What a groove is, and what nothing else on this figure is, is an UNBROKEN RUN
    of columns evenly spaced in x, each notched down most of a mass's height. So:
    take the columns notched in at least CHANNEL_MIN_ROWS rows, and look for four
    or more of them in a row at one stride, owning the span they cover.

    The run has to be unbroken, and a share over the whole spread is not enough:
    grooving the pack put eleven channels at a stride of 3, but two of them were
    isolated and two honest notches fell inside their spread, which drags the
    share to 0.79 against a 0.8 bar. It missed a groove by two hundredths. The
    dense middle of that same set is eight consecutive multiples of 3 owning
    every channel between them, which is not a ratio anything honest produces.

    Verified by sabotage in both directions: reinstating r2's pass on the beard
    (front, strides 3 and 5) and on the pack (back, stride 3) is caught, and the
    shipped body is clean on both sides.
    """
    for side in (1, -1):
        surface = {}
        for (x, y, z) in vox:
            row = surface.setdefault(z, {})
            if row.get(x, -10 ** 9) < y * side:
                row[x] = y * side
        rows_per_x = {}
        for row in surface.values():
            for x, depth in row.items():
                left, right = row.get(x - 1), row.get(x + 1)
                if left is not None and right is not None and left > depth and right > depth:
                    rows_per_x[x] = rows_per_x.get(x, 0) + 1
        channels = sorted(x for x, rows in rows_per_x.items() if rows >= CHANNEL_MIN_ROWS)
        if len(channels) < CHANNEL_MIN_RUN:
            continue
        for stride in range(2, 9):                  # ascending: the true stride wins
            for run in unbroken_runs(channels, stride):
                span = [x for x in channels if run[0] <= x <= run[-1]]
                if len(run) >= CHANNEL_PURITY * len(span):
                    return side, stride
    return None


def unbroken_runs(values, stride):
    """Every maximal run in `values` whose consecutive gaps are exactly `stride`,
    at least CHANNEL_MIN_RUN long. Longest first, so the densest evidence is
    judged before a shorter run that happens to sit inside a busier span."""
    runs, run = [], [values[0]]
    for value in values[1:]:
        if value - run[-1] == stride:
            run.append(value)
        else:
            runs.append(run)
            run = [value]
    runs.append(run)
    return sorted((r for r in runs if len(r) >= CHANNEL_MIN_RUN), key=len, reverse=True)


def unmounted_difference(vox, part_of, part, beard, hair):
    """Build the body again with `part` unmounted; return where the REST differs.

    This is the property that makes the beard and the hair real swap axes rather
    than a claim -- "shaped so that removing it leaves a complete head
    underneath: no skin voxels borrowed from the beard's volume, no hairline that
    only works with this hair." Stated that way it is exactly this: every voxel
    that is NOT the part's has to be identical whether the part is mounted or not.

    The obvious cheaper test -- "does some voxel remain in every column the part
    occupied" -- is the wrong one, and was tried first: a one-voxel shell is
    legitimately wider than the core it wraps, so the hair failed a check the
    asset passes. Rebuilding costs a second and a half and asks the real question.
    """
    mounts = {"beard": beard, "hair": hair}
    mounts[part] = None
    bare, _parts, _joints = build_voxels(mounts["beard"], mounts["hair"])
    rest = {p: c for p, c in vox.items() if part_of[p] != part}
    # A HOLE is the defect: a voxel the rest of the body has only because the
    # part was mounted. The reverse is not -- the bare build legitimately has
    # MORE voxels, because a beard standing proud of the chest overwrites the
    # tunic it covers and that tunic comes back when the beard goes.
    holes = set(rest) - set(bare)
    # And a recolour is the same defect wearing different clothes: a body voxel
    # whose colour depended on the part being there.
    recoloured = {p for p in set(rest) & set(bare) if rest[p] != bare[p]}
    return sorted(holes | recoloured)[:3], len(holes) + len(recoloured)


# ---------------------------------------------------------------------------
# Meshing
#
# THE RULE: NO QUAD MAY CROSS A JOINT. Greedy-merge within a part and never
# across parts, so every quad's four vertices belong to exactly one joint. A quad
# spanning shoulder to wrist cannot bend -- it can only shear into a
# parallelogram, which is what a "bending" voxel arm looks like when it goes
# wrong. It holds here BY CONSTRUCTION, because the mask a run merges over is
# keyed by (colour, joint) rather than by colour; main() checks it anyway,
# against the written GLB, because a rule that is only true by construction is
# one refactor from being false.
#
# This is the pine's mesher with that one change and one removal: the pine
# subtracts half a voxel in X and Y so an ODD-width trunk centres on the origin,
# and r2 added it straight back after meshing. The dwarf is even-width, so the
# shift and its undo are both simply absent here.
# ---------------------------------------------------------------------------
def cell_uv(cid):
    col, row = cid % CELLS_PER_ROW, cid // CELLS_PER_ROW
    return ((col * CELL + INSET) / ATLAS, (col * CELL + CELL - INSET) / ATLAS,
            (row * CELL + INSET) / ATLAS, (row * CELL + CELL - INSET) / ATLAS)


def greedy_mesh_parts(vox, joint_of, voxel_m):
    """Emit only faces exposed to air, merging co-planar same-colour SAME-JOINT runs.

    Vertices are NOT shared between quads, which is correct output for a voxel
    mesher and is the property that makes rigid joints work at all: at a joint
    seam the two sides hold separate vertices, so each follows its own bone and
    the seam opens and closes cleanly.
    """
    verts, faces, uvs, uvs_emit, quad_joint = [], [], [], [], []
    for d in range(3):
        u, v = (d + 1) % 3, (d + 2) % 3
        for s in (1, -1):
            slices = {}
            for p, c in vox.items():
                n = list(p)
                n[d] += s
                if tuple(n) in vox:
                    continue
                slices.setdefault(p[d], {})[(p[u], p[v])] = (c, joint_of[p])
            for sl in sorted(slices):
                mask = slices[sl]
                used = set()
                for (a, b) in sorted(mask):
                    if (a, b) in used:
                        continue
                    key = mask[(a, b)]
                    w = 1
                    while mask.get((a + w, b)) == key and (a + w, b) not in used:
                        w += 1
                    hgt = 1
                    while all(mask.get((a + i, b + hgt)) == key
                              and (a + i, b + hgt) not in used
                              for i in range(w)):
                        hgt += 1
                    for i in range(w):
                        for j in range(hgt):
                            used.add((a + i, b + j))
                    colour, joint = key
                    plane = sl + 1 if s == 1 else sl
                    quad = [(a, b), (a + w, b), (a + w, b + hgt), (a, b + hgt)]
                    u0, u1, v0, v1 = cell_uv(colour)
                    quv = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
                    e0, e1, f0, f1 = cell_uv(colour if colour in EMISSIVE_COLOURS
                                             else EMIT_OFF_CELL)
                    equv = [(e0, f0), (e1, f0), (e1, f1), (e0, f1)]
                    if s == -1:
                        quad, quv, equv = quad[::-1], quv[::-1], equv[::-1]
                    idx = len(verts)
                    for (uu, vv) in quad:
                        co = [0, 0, 0]
                        co[d] = plane
                        co[u] = uu
                        co[v] = vv
                        verts.append((co[0] * voxel_m, co[1] * voxel_m, co[2] * voxel_m))
                    faces.append((idx, idx + 1, idx + 2, idx + 3))
                    uvs.extend(quv)
                    uvs_emit.extend(equv)
                    quad_joint.append(joint)
    return verts, faces, uvs, uvs_emit, quad_joint


# ---------------------------------------------------------------------------
# Atlas
# ---------------------------------------------------------------------------
def palette_cell_origin(index):
    """Bottom-left pixel of a palette cell, in Blender (bottom-up) pixel space."""
    return (index % CELLS_PER_ROW) * CELL, (index // CELLS_PER_ROW) * CELL


def encode_palette_png():
    """Encode the 64x64 palette atlas as PNG bytes straight from the hex above.

    Written by hand rather than via Image.pack() with no data: pack() on a
    GENERATED image re-encodes the generated source and throws away whatever was
    assigned to .pixels, and the glTF exporter then copies those (black) packed
    bytes verbatim into the GLB.
    """
    rgb = bytearray(ATLAS * ATLAS * 3)               # top-down rows, as PNG wants
    for index, hx in enumerate(PALETTE_HEX):
        r, g, b = pine.hex_to_bytes(hx)
        cx, cy = palette_cell_origin(index)
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
    # images.new() is a BYTE image, so .pixels is display-referred: write the hex
    # straight in. Linearising here (the obvious-looking thing) bakes a second
    # sRGB decode into the texels and ships a visibly too-dark asset.
    px = [0.0] * (ATLAS * ATLAS * 4)
    for index, hx in enumerate(PALETTE_HEX):
        cx, cy = palette_cell_origin(index)
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
        cx, cy = palette_cell_origin(index)
        x = cx + CELL // 2
        y = height - 1 - (cy + CELL // 2)            # flip back to top-down rows
        o = (y * width + x) * nch
        shipped.append("#%02X%02X%02X" % (pix[o], pix[o + 1], pix[o + 2]))
    return shipped


# ---------------------------------------------------------------------------
# Blender scene assembly
# ---------------------------------------------------------------------------
UV_BASE = "UVMap"
UV_EMIT = "EmissiveMask"


def make_dwarf_material(name, img):
    """One material. Base colour reads the atlas through UV set 0; emission reads
    THE SAME ATLAS through UV set 1, where every non-emitting quad points at a
    black cell.

    This is the only way glTF expresses per-texel emission without a second
    image: emissiveTexture is emissiveFactor times a sampled texel, so a mask has
    to be black where nothing glows. A second image would have cost the
    one-image clause; a second UV set costs eight bytes a vertex and keeps the
    one mesh / one material / one palette image shape the ruling promised.

    Emission strength stays EXACTLY 1.0, because anything else makes the exporter
    write KHR_materials_emissive_strength and the asset would carry an extension.
    """
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    mat.use_backface_culling = True     # -> glTF omits doubleSided (false)
    mat.blend_method = 'OPAQUE'
    tree = mat.node_tree
    bsdf = tree.nodes["Principled BSDF"]
    bsdf.inputs["Metallic"].default_value = 0.0
    bsdf.inputs["Roughness"].default_value = 0.92
    if "Specular IOR Level" in bsdf.inputs:
        bsdf.inputs["Specular IOR Level"].default_value = 0.5   # no extensions
    bsdf.inputs["Emission Strength"].default_value = 1.0

    base = tree.nodes.new("ShaderNodeTexImage")
    base.image, base.interpolation, base.extension = img, 'Closest', 'CLIP'
    base.location = (-400, 250)
    base_uv = tree.nodes.new("ShaderNodeUVMap")
    base_uv.uv_map, base_uv.location = UV_BASE, (-620, 250)
    tree.links.new(base_uv.outputs["UV"], base.inputs["Vector"])
    tree.links.new(base.outputs["Color"], bsdf.inputs["Base Color"])

    emit = tree.nodes.new("ShaderNodeTexImage")
    emit.image, emit.interpolation, emit.extension = img, 'Closest', 'CLIP'
    emit.location = (-400, -100)
    emit_uv = tree.nodes.new("ShaderNodeUVMap")
    emit_uv.uv_map, emit_uv.location = UV_EMIT, (-620, -100)
    tree.links.new(emit_uv.outputs["UV"], emit.inputs["Vector"])
    tree.links.new(emit.outputs["Color"], bsdf.inputs["Emission Color"])
    return mat


def build_mesh_object(verts, faces, uvs, uvs_emit, name):
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    me.update()
    for layer_name, data in ((UV_BASE, uvs), (UV_EMIT, uvs_emit)):
        layer = me.uv_layers.new(name=layer_name)
        for i, uv in enumerate(data):
            layer.data[i].uv = uv
    for poly in me.polygons:
        poly.use_smooth = False                      # voxels are flat-shaded
    me.validate(verbose=False)
    img = make_palette_image(IMAGE_NAME)
    me.materials.append(make_dwarf_material(MATERIAL_NAME, img))
    ob = bpy.data.objects.new(name, me)
    bpy.context.collection.objects.link(ob)
    ob.location = (0.0, 0.0, 0.0)
    ob.rotation_euler = (0.0, 0.0, 0.0)
    ob.scale = (1.0, 1.0, 1.0)
    return ob


def build_armature(shift, voxel, name):
    """The skeleton, in metres, translated with the body.

    Bones are declared in the same voxel lattice the geometry is, then moved by
    the SAME centring shift, so a bone cannot drift from what it drives.
    """
    dx, dy = shift
    arm_data = bpy.data.armatures.new("%s_Skeleton" % name)
    arm = bpy.data.objects.new("%s_Skeleton" % name, arm_data)
    bpy.context.collection.objects.link(arm)
    arm.location = (0.0, 0.0, 0.0)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode='EDIT')
    for name, parent, head, tail in BONES:
        bone = arm_data.edit_bones.new(name)
        bone.head = ((head[0] + dx) * voxel, (head[1] + dy) * voxel, head[2] * voxel)
        bone.tail = ((tail[0] + dx) * voxel, (tail[1] + dy) * voxel, tail[2] * voxel)
        bone.roll = 0.0
        bone.use_connect = False
        if parent is not None:
            bone.parent = arm_data.edit_bones[parent]
    bpy.ops.object.mode_set(mode='OBJECT')
    return arm


def bind(ob, arm, faces, quad_joint):
    """Rigid weights: every vertex 1.0 to one joint, no blending.

    That is not a downgrade from soft skinning, it is what the reference does --
    the limbs rotate as solid blocks. Soft weights on cubes smear the voxel read,
    which is the look we are buying.
    """
    groups = {name: ob.vertex_groups.new(name=name) for name in JOINT_NAMES}
    members = {name: [] for name in JOINT_NAMES}
    for face, joint in zip(faces, quad_joint):
        members[joint].extend(face)
    for name, indices in members.items():
        if indices:
            groups[name].add(indices, 1.0, 'REPLACE')
    ob.parent = arm
    modifier = ob.modifiers.new("Armature", 'ARMATURE')
    modifier.object = arm
    modifier.use_vertex_groups = True


def apply_pose(arm, pose):
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode='POSE')
    for bone in arm.pose.bones:
        bone.rotation_mode = 'XYZ'
        angles = pose.get(bone.name, (0.0, 0.0, 0.0))
        bone.rotation_euler = tuple(a * 3.141592653589793 / 180.0 for a in angles)
    bpy.ops.object.mode_set(mode='OBJECT')
    bpy.context.view_layer.update()


def export_glb(ob, arm, path, rest_position):
    for o in bpy.data.objects:
        o.select_set(False)
    ob.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.export_scene.gltf(
        filepath=path, export_format='GLB', use_selection=True,
        export_apply=False, export_yup=True, export_image_format='AUTO',
        export_materials='EXPORT', export_normals=True, export_texcoords=True,
        export_cameras=False, export_lights=False, export_animations=False,
        export_skins=True, export_def_bones=False, export_leaf_bone=False,
        export_rest_position_armature=rest_position,
    )


def build(voxel=DEFAULT_VOXEL, pose="neutral", beard="miner-full", hair="miner-shag"):
    """Everything between a wiped scene and an exportable pair of objects."""
    vox, part_of, joint_of = build_voxels(beard, hair)
    vox, part_of, joint_of, shift = centre_voxels(vox, part_of, joint_of)
    verts, faces, uvs, uvs_emit, quad_joint = greedy_mesh_parts(vox, joint_of, voxel)
    name = asset_name(pose)
    ob = build_mesh_object(verts, faces, uvs, uvs_emit, name)
    arm = build_armature(shift, voxel, name)
    bind(ob, arm, faces, quad_joint)
    apply_pose(arm, POSES[pose])
    return ob, arm, vox, part_of, joint_of, shift, faces


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------
USAGE = ("usage: blender --background --python dwarf_miner.py -- <out.glb> "
         "[--voxel M] [--pose neutral|swing] [--beard NAME] [--hair NAME] [--blend P]")
ACCEPTED_BASENAMES = {asset_name(pose): pose for pose in POSES}


def parse_args(argv):
    argv = argv[argv.index("--") + 1:] if "--" in argv else []
    if not argv:
        raise SystemExit(USAGE)
    out = argv[0]
    if not out.lower().endswith(".glb"):
        raise SystemExit("error: output must be a .glb path\n" + USAGE)
    basename = os.path.splitext(os.path.basename(out))[0]
    if basename not in ACCEPTED_BASENAMES:
        # The contract requires basename == mesh name == node name. The published
        # SLOT is SM_VoxelDwarf_Miner01.glb and does not churn, but a CANDIDATE
        # carries the revision in its basename so that all three still agree --
        # promotion is a copy plus the revision bump, and a stale binary that
        # names itself r2 is then visibly stale. Refuse rather than write a GLB
        # whose internal name disagrees with its path.
        raise SystemExit("error: basename must be one of %s, got %r\n%s"
                         % (", ".join(sorted(ACCEPTED_BASENAMES)), basename, USAGE))
    voxel, blend, rest = DEFAULT_VOXEL, None, list(argv[1:])
    pose, beard, hair = ACCEPTED_BASENAMES[basename], "miner-full", "miner-shag"
    while rest:
        flag = rest.pop(0)
        if flag == "--voxel":
            voxel = float(rest.pop(0))
        elif flag == "--pose":
            pose = rest.pop(0)
        elif flag == "--beard":
            beard = rest.pop(0)
        elif flag == "--hair":
            hair = rest.pop(0)
        elif flag == "--blend":
            blend = os.path.abspath(rest.pop(0))
        else:
            raise SystemExit("error: unknown option %r\n%s" % (flag, USAGE))
    if pose not in POSES:
        raise SystemExit("error: --pose must be one of %s (got %r)"
                         % (", ".join(sorted(POSES)), pose))
    if pose != ACCEPTED_BASENAMES[basename]:
        # A posed GLB and a neutral one are different assets and must not share a
        # name: that is the exact defect the naming scheme exists to prevent, one
        # level down from two meshes called SM_VoxelPine_Tree02.
        raise SystemExit("error: basename %r names the %r pose, but --pose is %r"
                         % (basename, ACCEPTED_BASENAMES[basename], pose))
    if beard not in BEARDS:
        raise SystemExit("error: --beard must be one of %s (got %r)"
                         % (", ".join(sorted(BEARDS)), beard))
    if hair not in HAIRS:
        raise SystemExit("error: --hair must be one of %s (got %r)"
                         % (", ".join(sorted(HAIRS)), hair))
    # Voxel size must be positive. Zero is the dangerous one: it collapses the mesh
    # to a point AND scales expected_h/expected_volume by the same zero, so every
    # closure check passes vacuously and the script reports OK on nothing. The
    # oracle has to be independent of the input it is checking.
    if not voxel > 0.0:
        raise SystemExit("error: --voxel must be greater than 0 (got %r)\n%s" % (voxel, USAGE))
    return os.path.abspath(out), voxel, pose, beard, hair, blend


def main():
    out, voxel, pose, beard, hair, blend = parse_args(list(sys.argv))
    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)

    pine.wipe_scene()
    ob, arm, vox, part_of, joint_of, (dx, dy), faces = build(voxel, pose, beard, hair)
    export_glb(ob, arm, out, rest_position=(pose == "neutral"))
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
    uv_emit = pine.read_accessor(gj, binary, prim["attributes"].get("TEXCOORD_1", -1)) \
        if "TEXCOORD_1" in prim["attributes"] else []
    joints = pine.read_accessor(gj, binary, prim["attributes"]["JOINTS_0"]) \
        if "JOINTS_0" in prim["attributes"] else []
    weights = pine.read_accessor(gj, binary, prim["attributes"]["WEIGHTS_0"]) \
        if "WEIGHTS_0" in prim["attributes"] else []

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
    skins = gj.get("skins", [])
    joint_nodes = [gj["nodes"][i]["name"] for i in skins[0]["joints"]] if skins else []
    mesh_name = gj["meshes"][0]["name"]
    node_name = next(n["name"] for n in gj["nodes"] if n.get("mesh") == 0)
    basename = os.path.splitext(os.path.basename(out))[0]
    material = gj["materials"][0]

    print(
        "FIGURES name=%s revision=%s pose=%s beard=%s hair=%s voxel=%.4f height_voxels=%d "
        "voxels=%d groups=%s joints=%d centring=%+d,%+d quads=%d verts=%d tris=%d "
        "bbox=%.4fx%.4fx%.4f centre_x=%+.6f centre_z=%+.6f min_y=%+.6f volume=%.6f "
        "expected_volume=%.6f materials=%d primitives=%d images=%d cells=%d palette=%s "
        "glb_bytes=%d"
        % (asset_name(pose), REVISION, pose, beard, hair, voxel, HEIGHT_VOXELS, len(vox),
           ",".join("%s:%d" % kv for kv in sorted(groups.items())), len(joint_nodes),
           dx, dy, nquads, nverts, ntris, size[0], size[1], size[2],
           centre[0], centre[2], lo[1], vol, expected_vol, nmats, nprims,
           len(gj.get("images", [])), len(PALETTE_HEX), ",".join(shipped_palette), nbytes)
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
    # He is 96 voxels tall, which is ruled and NOT a consequence of --voxel: read
    # the count back out of the written height, so a body authored to some other
    # count fails here rather than silently shipping.
    check(abs(size[1] / voxel - HEIGHT_VOXELS) < 1e-3,
          "model is %.3f voxels tall, expected %d" % (size[1] / voxel, HEIGHT_VOXELS))
    grid = DEFAULT_VOXEL
    check(all(abs(v - round(v / grid) * grid) <= 1e-5 for p in positions for v in p),
          "some POSITION values are off the %g m project grid" % grid)
    check(abs(vol - expected_vol) <= max(1e-6, expected_vol * 1e-4),
          "signed volume %.6f != voxel volume %.6f (hull not closed, faces duplicated, "
          "or normals inverted)" % (vol, expected_vol))
    check(ntris == nquads * 2, "tris %d != quads*2 %d" % (ntris, nquads * 2))
    check(pine.check_mesh_properties(nquads, nverts),
          "verts %d != quads*4 %d (not the expected unwelded quad soup)"
          % (nverts, nquads * 4))
    check(nmats == 1, "expected exactly 1 material, got %d" % nmats)
    check(nprims == 1, "expected exactly 1 primitive (1 draw call), got %d" % nprims)
    check(len(gj.get("meshes", [])) == 1,
          "expected exactly 1 mesh, got %d" % len(gj.get("meshes", [])))
    check(len(gj.get("images", [])) == 1,
          "expected exactly 1 embedded palette image, got %d" % len(gj.get("images", [])))
    check(not gj.get("extensionsUsed"),
          "expected no glTF extensions, got %s" % gj.get("extensionsUsed"))
    check(not gj.get("extensionsRequired"),
          "expected no required glTF extensions, got %s" % gj.get("extensionsRequired"))
    check(gj["materials"][0].get("doubleSided", False) is False,
          "material should be single-sided")
    check(all(0.0 <= u <= 1.0 and 0.0 <= v <= 1.0 for u, v in uv_vals),
          "some UVs fall outside 0-1")
    check(bool(uv_emit) and all(0.0 <= u <= 1.0 and 0.0 <= v <= 1.0 for u, v in uv_emit),
          "the emissive UV set is missing or falls outside 0-1")
    # Guards a real regression: Image.pack() with no data silently ships a black
    # atlas, which every check above still passes. ALL cells are read back here.
    check(shipped_palette == PALETTE_HEX,
          "shipped palette %s != this module's palette %s"
          % (",".join(shipped_palette), ",".join(PALETTE_HEX)))
    check(gj["samplers"][0].get("magFilter") == 9728,
          "palette magFilter is %s, expected 9728 (NEAREST)"
          % gj["samplers"][0].get("magFilter"))
    check(gj["samplers"][0].get("wrapS") == 33071 and gj["samplers"][0].get("wrapT") == 33071,
          "palette sampler wrap is %s/%s, expected 33071/33071 (CLAMP_TO_EDGE)"
          % (gj["samplers"][0].get("wrapS"), gj["samplers"][0].get("wrapT")))
    check(mesh_name == node_name == asset_name(pose) == basename,
          "naming clause: basename/mesh/node are %r/%r/%r, expected %r"
          % (basename, mesh_name, node_name, asset_name(pose)))
    check(all(carries_revision(str(name))
              for name in (mesh_name, node_name, material.get("name", ""),
                           gj["images"][0].get("name", ""))),
          "every internal name must carry the revision %r: mesh/node/material/image are "
          "%r/%r/%r/%r" % (REVISION, mesh_name, node_name, material.get("name"),
                           gj["images"][0].get("name")))
    for clause in ("animations", "cameras", "extensions"):
        check(not gj.get(clause), "GLB must contain no %s, got %s" % (clause, gj.get(clause)))

    # --- the rig ------------------------------------------------------------
    check(len(skins) == 1, "expected exactly 1 skin, got %d" % len(skins))
    # The contract is the NAMES, not their order: the exporter emits joints in its
    # own hierarchy walk, and a clip binds by name. Checked as a set, with
    # duplicates caught separately because a repeated name would pass a set test
    # and break every clip that used it.
    check(sorted(joint_nodes) == sorted(JOINT_NAMES) and len(set(joint_nodes)) == len(joint_nodes),
          "joint names are %s, expected exactly %s" % (joint_nodes, JOINT_NAMES))
    # RIGID WEIGHTS: every vertex 1.0 to one joint, no blending.
    bad_weight = next((w for w in weights
                       if abs(w[0] - 1.0) > 1e-6 or any(abs(x) > 1e-6 for x in w[1:])), None)
    check(bool(weights) and bad_weight is None,
          "weights must be rigid -- 1.0 to a single joint; found %s" % (bad_weight,))
    # THE RULE: no quad may cross a joint. Checked against the WRITTEN GLB, not
    # against the mesher's intent, because a rule that is only true by
    # construction is one refactor away from being false.
    crossing = next((i for i in range(0, len(joints), 4)
                     if len({j[0] for j in joints[i:i + 4]}) != 1), None)
    check(bool(joints) and crossing is None,
          "a quad crosses a joint: vertices %d..%d carry joints %s"
          % (crossing or 0, (crossing or 0) + 3,
             sorted({j[0] for j in joints[(crossing or 0):(crossing or 0) + 4]})))

    # --- emissive -----------------------------------------------------------
    # The flame cells emit (Wolf, B: yes) and NOTHING ELSE DOES. That is the
    # property worth checking: emission is sampled from the shared atlas through
    # UV set 1, so a mis-wired mask would make the whole dwarf glow.
    emissive = material.get("emissiveTexture")
    check(emissive is not None and emissive.get("texCoord") == 1,
          "expected an emissiveTexture on UV set 1, got %s" % emissive)
    check(material.get("emissiveFactor", [0, 0, 0]) == [1, 1, 1],
          "emissiveFactor is %s, expected [1, 1, 1]" % material.get("emissiveFactor"))
    # glTF's UV origin is the TOP left and Blender's is the bottom left, so the
    # exporter writes v_gltf = 1 - v_blender. The mask cell's box has to be
    # flipped with it; without that this check reads the wrong corner of the
    # atlas, finds nothing there and reports that the whole dwarf glows.
    off_u0, off_u1, off_v0, off_v1 = cell_uv(EMIT_OFF_CELL)
    off_v0, off_v1 = 1.0 - off_v1, 1.0 - off_v0
    lit_verts = sum(1 for (u, v) in uv_emit
                    if not (off_u0 - 1e-6 <= u <= off_u1 + 1e-6
                            and off_v0 - 1e-6 <= v <= off_v1 + 1e-6))
    check(0 < lit_verts < nverts * 0.02,
          "%d of %d vertices sample a lit emissive cell; the mask should cover the "
          "lantern panes and nothing else" % (lit_verts, nverts))

    # --- the body's own falsifiable properties ------------------------------
    channel = find_repeating_channel(vox)
    check(channel is None,
          "a one-voxel channel repeats at a fixed stride (side %s, stride %s) -- that is the "
          "r2 groove pass and this round removed it" % (channel or ("", "")))
    # The swap axes, checked on the UNCENTRED body: unmounting a part changes the
    # extents and so changes the centring shift, which would make every voxel
    # differ for a reason that has nothing to do with the claim.
    raw_vox, raw_parts, _raw_joints = build_voxels(beard, hair)
    for part in ("beard", "hair"):
        sample_diff, ndiff = unmounted_difference(raw_vox, raw_parts, part, beard, hair)
        check(not ndiff,
              "unmounting the %r part changes %d voxel(s) that are not its own, first %s: a swap "
              "axis has to leave a complete body underneath" % (part, ndiff, sample_diff))

    if fails:
        for msg in fails:
            sys.stderr.write("FAIL: %s\n" % msg)
        sys.stderr.write("%d check(s) failed\n" % len(fails))
        sys.exit(1)
    print("OK %s (%s, %s) -> %s" % (asset_name(pose), REVISION, pose, out))
    sys.exit(0)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        # main() and parse_args already exit with a meaningful code; assert-style
        # failures use 1.
        raise
    except Exception as error:
        # Blender's --background runner prints a traceback for ANY uncaught
        # exception and still exits 0, so a bad --voxel value, an unwritable
        # output path or an export failure would report success having written
        # nothing. Same guard voxel_pine.py carries, for the same reason: exit 0
        # with no output is not a result.
        traceback.print_exc()
        raise SystemExit("generator failed: %s: %s" % (type(error).__name__, error)) from error
