"""Standalone generator for the Frostvein voxel dwarf miner.

Reproduces Section A of the modelling reference sheet as a game-ready glTF
asset. No MCP, no live session, no manual steps.

    blender --background --python dwarf_miner.py -- <out.glb> [options]

        <out.glb>    destination path; parent directories are created

        --voxel M    metres per voxel (default 0.1)
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
# Palette -- Section A's "GEAR & PROP BREAKDOWN" swatch column.
#
# Two of the eight labels are ambiguous on the sheet and CANNOT be settled from
# it. See PALETTE_PROVENANCE below; those two values are the brief's, carried
# forward because the artifact cannot decide between them.
# ---------------------------------------------------------------------------
HEX_SKIN       = "#E9D2BB"
HEX_BEARD      = "#5E4632"
HEX_SNOW       = "#FFFFFF"
HEX_TUNIC      = "#5F7A6A"
HEX_PANTS      = "#474B41"
HEX_METAL      = "#A9B2AC"
HEX_WOOD       = "#8B6B50"
HEX_WOOD_TRUNK = "#6B5B49"

SKIN, BEARD, SNOW, TUNIC, PANTS, METAL, WOOD, WOOD_TRUNK = range(8)
PALETTE_HEX = [
    HEX_SKIN, HEX_BEARD, HEX_SNOW, HEX_TUNIC,
    HEX_PANTS, HEX_METAL, HEX_WOOD, HEX_WOOD_TRUNK,
]
ROLE_NAMES = ["Skin", "Beard", "Snow", "Tunic", "Pants", "Metal", "Wood", "Wood Trunk"]

# NOTE: "Snow" is the sheet's white, and on this asset it is the lantern's glass
# pane -- the one place a dwarf underground wants a pure white. The pane is
# plain albedo and NOT emissive: the client owns the lantern's light.

PALETTE_PROVENANCE = (
    "Skin/Beard/Tunic/Metal  label legible on the sheet, swatch agrees\n"
    "Snow/Wood Trunk         confirmed: byte-identical in the shipped pine atlas\n"
    "Pants/Wood              UNRESOLVED. The sheet's 8 and B are the same glyph at\n"
    "                        1024 px, which is the sheet's full resolution and not a\n"
    "                        downscale, and the swatches are JPEG-compressed and\n"
    "                        internally noisy: calibrating channel differences against\n"
    "                        the three known-truth swatches leaves a residual error up\n"
    "                        to 8.8/255, well over the 3/255 that separates the two\n"
    "                        candidates. Carried from the brief, NOT confirmed."
)

PUBLISHED_NAME = "SM_VoxelDwarf_Miner01"     # ruled by Wolf; also the mesh and node name
HEIGHT_VOXELS = 12                           # 12 x 0.1 m = 1.20 m, the ratified anchor
DEFAULT_VOXEL = 0.1                          # metres per voxel; the project grid, 1x


# ---------------------------------------------------------------------------
# The body
#
# Axes are Blender's: +X is the dwarf's left, +Y is the direction he faces, +Z
# is up. The exporter's export_yup then lands him facing -Z in glTF, which is
# the usual forward for a character.
#
# WIDTHS ARE EVEN ON PURPOSE, and this is the one place the dwarf cannot copy
# the pine. voxel_pine.greedy_mesh() shifts the lattice half a voxel in X and Y
# so an ODD-width trunk centres on the origin; at the pine's 0.2 m voxel that
# half-shift is 0.1 m and still lands on the project grid. At the dwarf's 0.1 m
# voxel it is 0.05 m, and check_asset.py's grid clause rejects every vertex. So
# the dwarf is even-width in X and Y and the half-shift is undone after meshing
# (see build_mesh): the unshifted lattice both centres the bounding box and
# stays on the 0.1 m grid.
#
#   X spans -5..4  (10 voxels, 1.00 m)   pickaxe head .. lantern
#   Y spans -3..2  ( 6 voxels, 0.60 m)   backpack ..... beard and nose
#   Z spans  0..11 (12 voxels, 1.20 m)   boot sole .... hair and pickaxe head
#
# A bounding box is centred only when max == -min - 1 on an axis, so the gear
# is what balances him: the pickaxe reaches -5 and the lantern +4, the backpack
# -3 and the beard +2. Moving one without the other decentres the asset, and
# every instance placed from it then leans the same way.
# ---------------------------------------------------------------------------
LEG_COLUMNS = ((-3, -2), (1, 2))             # a 2-voxel gap between the boots


def build_voxels():
    """Return ({(x, y, z): palette index}, {(x, y, z): part name})."""
    vox, part_of = {}, {}

    def fill(x0, x1, y0, y1, z0, z1, colour, part="body"):
        for x in range(x0, x1 + 1):
            for y in range(y0, y1 + 1):
                for z in range(z0, z1 + 1):
                    vox[(x, y, z)] = colour
                    part_of[(x, y, z)] = part

    def put(cells, colour, part):
        for (x, y, z) in cells:
            vox[(x, y, z)] = colour
            part_of[(x, y, z)] = part

    # --- boots and trousers, z 0-3 -----------------------------------------
    for x0, x1 in LEG_COLUMNS:
        fill(x0, x1, -1, 1, 0, 1, WOOD_TRUNK)        # leather boots
        fill(x0, x1, -1, 1, 2, 3, PANTS)             # trousers

    # --- torso, z 4-7 -------------------------------------------------------
    fill(-3, 2, -2, 1, 4, 7, TUNIC)
    fill(-3, 2, -2, 1, 4, 4, WOOD_TRUNK)             # belt, a full band
    fill(-1, 0, 1, 1, 4, 4, METAL)                   # buckle, on the front face only
    # Hands reach the full depth of the torso so each one touches its gear: the
    # pickaxe is carried at y=1 (in front of him) and the lantern at y=0.
    fill(-3, -3, -1, 1, 5, 5, SKIN)                  # right hand, on the pickaxe
    fill(2, 2, -1, 1, 5, 5, SKIN)                    # left hand, on the lantern

    # --- backpack, hung off the back of the torso ---------------------------
    fill(-2, 1, -3, -3, 5, 7, WOOD_TRUNK)

    # --- head, z 8-11 -------------------------------------------------------
    # A solid hair/beard block, with the face cut back into its front plane.
    fill(-2, 1, -1, 1, 8, 11, BEARD)
    fill(-2, 1, 1, 1, 10, 10, SKIN)                  # brow, the full width of the head
    fill(-1, 0, 1, 1, 9, 9, SKIN)                    # face, framed by hair on both sides
    fill(-1, 0, 2, 2, 9, 9, SKIN)                    # nose, proud of the beard

    # NOTE: no eyes. Once the hair frames it the face is 2 voxels wide, so a
    # separated pair cannot be drawn and an adjacent pair reads as a brow band
    # rather than as eyes. Reported as a 12-voxel budget casualty.

    # --- beard, hanging forward over the chest ------------------------------
    fill(-2, 1, 2, 2, 7, 8, BEARD)                   # full width under the chin
    fill(-1, 0, 2, 2, 6, 6, BEARD)                   # tapering to a point
    fill(-1, 0, 1, 1, 6, 7, BEARD)                   # lying on the tunic

    # --- pickaxe, in the right hand, head raised over the shoulder ----------
    # Carried at y=1, a voxel proud of the torso, so it does not bisect him in
    # profile. The haft runs straight up through the head, whose two ends turn
    # down -- a bar alone reads as a hook, which is what the first cut did.
    fill(-4, -4, 1, 1, 5, 10, WOOD, "pickaxe")       # haft
    fill(-5, -3, 1, 1, 11, 11, METAL, "pickaxe")     # head, crossing the haft
    put([(-5, 1, 10), (-3, 1, 10)], METAL, "pickaxe")   # the two picks, turned down

    # --- lantern, carried in the left hand ----------------------------------
    # Its cap sits level with the fist and the body hangs just under, which is
    # how the contact sheet carries it. Slung lower -- the first two cuts put
    # the body at z 1-3 and z 2-4, on a bail -- it read as a box on the floor.
    fill(3, 4, -1, 0, 5, 5, METAL, "lantern")        # cap, gripped by the hand
    fill(3, 4, -1, 0, 4, 4, SNOW, "lantern")         # glass, lit by the CLIENT
    fill(3, 4, -1, 0, 3, 3, METAL, "lantern")        # base

    return vox, part_of


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
        # check_asset.py will reject on the naming clause.
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
        "FIGURES name=%s voxel=%.3f voxels=%d groups=%s quads=%d verts=%d tris=%d "
        "bbox=%.3fx%.3fx%.3f centre_x=%+.6f centre_z=%+.6f min_y=%+.6f "
        "volume=%.6f expected_volume=%.6f materials=%d primitives=%d images=%d "
        "palette=%s glb_bytes=%d"
        % (PUBLISHED_NAME, voxel, len(vox),
           ",".join("%s:%d" % kv for kv in sorted(groups.items())),
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
    # The 0.1 m project grid. The pine never checks this: its half-voxel lattice
    # shift happens to land on the grid at a 0.2 m voxel. At 0.1 m it does not,
    # and this is the check that catches it.
    check(all(abs(v / 0.1 - round(v / 0.1)) <= 1e-5 for p in positions for v in p),
          "some POSITION values are off the 0.1 m project grid")
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
    # atlas, which every check above still passes.
    check(shipped_palette == PALETTE_HEX,
          "shipped palette %s != sheet palette %s"
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
