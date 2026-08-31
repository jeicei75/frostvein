"""Standalone generator for the Frostvein voxel pine trees.

Reproduces the four pine variants from Section B of the modelling reference
sheet as game-ready glTF assets. No MCP, no live session, no manual steps.

    blender --background --python voxel_pine.py -- <type> <out.glb> [options]

        <type>      1 | 2 | 3 | 4   (sheet: 4, 5, 5 and 6 bough cells)
        <out.glb>   destination path; parent directories are created

        --seed N    seed for bough jitter and snow clumping (default 1337)
        --voxel M   metres per voxel (default 0.2)

Determinism
    Identical arguments produce a byte-identical GLB. All variation comes from
    an integer hash of (seed, x, y, z, tag); nothing consults the clock, the
    filesystem or Python's random module. Sets of integer tuples are iterated
    in sorted order so results never depend on hash-table layout.

Self-verification
    Prints one line of figures parsed back out of the written GLB, then checks
    them and exits non-zero if any check fails.

Mesh properties (see also ASSET_NOTES.md)
    The mesh is a greedy-meshed voxel hull: every quad carries its own four
    vertices and NO vertices are shared between quads. It is intentionally
    left unwelded. Consequences are documented in check_mesh_properties().

Requires only the Python standard library and bpy.
"""

import json
import math
import os
import struct
import sys
import traceback
import zlib

import bpy

# ---------------------------------------------------------------------------
# Palette. Hex values as printed in the reference sheet's swatch column; the
# three shaded/lit variants extend the sheet's greens and snow for voxel
# fleck, and are named here so the whole palette lives in one place.
# ---------------------------------------------------------------------------
HEX_TRUNK_BROWN  = "#4A3B2E"   # sheet: Trunk Brown
HEX_WOOD_TRUNK   = "#6B5B49"   # sheet: Wood Trunk
HEX_NEEDLE_DARK  = "#2A3E34"   # sheet: Needle Green, shaded
HEX_NEEDLE_GREEN = "#364D3F"   # sheet: Needle Green
HEX_NEEDLE_LIGHT = "#52715B"   # sheet: Needle Green, lit
HEX_SNOW         = "#FFFFFF"   # sheet: Snow
HEX_SNOW_SHADE   = "#D8E4EC"   # sheet: Snow, shaded

BARK_D, BARK, NDL_D, NDL_M, NDL_L, SNOW, SNOW_S = range(7)
PALETTE_HEX = [
    HEX_TRUNK_BROWN, HEX_WOOD_TRUNK, HEX_NEEDLE_DARK,
    HEX_NEEDLE_GREEN, HEX_NEEDLE_LIGHT, HEX_SNOW, HEX_SNOW_SHADE,
]
NEEDLES = (NDL_D, NDL_M, NDL_L)

DEFAULT_SEED = 1337
DEFAULT_VOXEL = 0.2            # metres per voxel
DWARF_HEIGHT_M = 1.20          # anchor for the sheet's "N x dwarf height"

ATLAS, CELL, INSET = 64, 16, 4  # palette atlas: 4x4 grid of 16px cells
TIER_LAYERS = 3                 # voxel layers per bough cell
TIER_STEP = 2.5                 # radius lost per layer as a tier rises
LOBE_AMP = 0.8                  # 4-lobe angular term -> boughy silhouette
JITTER_AMP = 1.9                # radial jitter on the tier edge
SNOW_TIP_VOXELS = 5             # topmost spire voxels always capped

# ---------------------------------------------------------------------------
# Tree variants, Section B of the sheet.
#
# height is the total height in voxels, chosen so that height * 0.2 m matches
# the sheet's "N x dwarf height" label with the dwarf anchored at 1.20 m.
# tiers is [(base_z, radius)] per bough cell, bottom first.
# spire is (base_z, radius) for the tapering snow-capped top.
# trunk_r is (base, tip): the trunk column's radius at the bottom and where it
#   meets the top tier. Deliberately much smaller than flare -- the wide root
#   plate against a slender column is the proportion the sheet reads with.
# flare is (z0_radius, z1_radius) for the root plate, sized independently.
# ---------------------------------------------------------------------------
TREE_TYPES = {
    1: dict(label="Tree01", cells=4, height=32, dwarf_mult=5.32,
            trunk_r=(1.5, 1.0), flare=(3, 2), spire=(23, 3.0),
            tiers=[(5, 11.0), (10, 9.0), (15, 7.0), (20, 5.0)]),
    2: dict(label="Tree02", cells=5, height=40, dwarf_mult=6.67,
            trunk_r=(1.6, 1.0), flare=(3, 2), spire=(30, 3.0),
            tiers=[(7, 12.0), (12, 10.0), (17, 8.0), (22, 6.0), (27, 4.2)]),
    3: dict(label="Tree03", cells=5, height=40, dwarf_mult=6.67,
            trunk_r=(2.0, 1.2), flare=(4, 3), spire=(35, 2.6),
            tiers=[(14, 8.0), (19, 7.0), (24, 6.0), (28, 5.0), (32, 3.6)]),
    4: dict(label="Tree04", cells=6, height=53, dwarf_mult=8.80,
            trunk_r=(2.1, 1.2), flare=(4, 3), spire=(45, 3.0),
            tiers=[(14, 10.0), (20, 9.0), (26, 8.0), (32, 6.5), (37, 5.0),
                   (42, 3.6)]),
}


# ---------------------------------------------------------------------------
# Deterministic noise
# ---------------------------------------------------------------------------
def noise(*args):
    """FNV-1a derived hash of integer arguments -> float in [0, 1].

    Deterministic across runs, machines and Python builds: it touches only
    integer arithmetic, never hash() (which is randomised for str and bytes).
    """
    x = 2166136261
    for a in args:
        x ^= (int(a) + 1000) & 0xFFFFFFFF
        x = (x * 16777619) & 0xFFFFFFFF
        x ^= x >> 13
    return ((x >> 8) & 0xFFFF) / 65535.0


def mirrored_noise(seed, x, y, z, tag):
    """Noise mirrored across both horizontal axes, via abs() on x and y.

    Anything deciding whether a voxel EXISTS must use this. Signed-coordinate
    noise makes the silhouette a voxel wider on one side, which pushes the
    bounding-box centre off the trunk, and every instance placed from the
    asset then leans the same way. Colour choice and snow placement
    deliberately use raw noise() instead: neither can widen a column, so
    neither can skew the box, and keeping them asymmetric stops the tree
    reading as mirrored.
    """
    return noise(seed, abs(x), abs(y), z, tag)


# ---------------------------------------------------------------------------
# Voxel construction
# ---------------------------------------------------------------------------
def build_voxels(spec, seed):
    """Return {(x, y, z): palette_index}. Occupancy is symmetric in x and y."""
    height = spec["height"]
    tiers = spec["tiers"]
    flare_lo, flare_hi = spec["flare"]
    spire_z, spire_r = spec["spire"]
    core_top = tiers[-1][0] + TIER_LAYERS
    tip = height - 1
    vox = {}

    # Flared root base: stacked diamonds, symmetric by construction.
    for z, rd in ((0, flare_lo), (1, flare_hi)):
        for x in range(-rd, rd + 1):
            for y in range(-rd, rd + 1):
                if abs(x) + abs(y) <= rd:
                    vox[(x, y, z)] = BARK_D

    # Trunk, tapering from trunk_base to trunk_tip as it rises. The radial
    # cross-section lets thickness be tuned in sub-voxel steps rather than
    # jumping 3x3 -> 5x5: hypot <= 1.0 is a 5-cell plus, <= 1.5 a full 3x3,
    # <= 2.0 a 13-cell cross, <= 2.3 a 21-cell round, <= 2.9 a full 5x5.
    trunk_base, trunk_tip = spec["trunk_r"]
    for z in range(2, core_top):
        t = (z - 2) / max(1, core_top - 3)
        r = trunk_base + (trunk_tip - trunk_base) * t
        ext = int(r) + 1
        for x in range(-ext, ext + 1):
            for y in range(-ext, ext + 1):
                if math.hypot(x, y) <= r:
                    vox[(x, y, z)] = BARK_D if noise(seed, x, y, z, 7) < 0.35 else BARK

    # Bare trunk continuing up through the spire, so it shows between boughs.
    for z in range(core_top, min(spire_z + 3, tip)):
        vox[(0, 0, z)] = BARK

    # Bough cells: a drooping skirt per tier, narrowing over TIER_LAYERS.
    for ti, (z0, radius) in enumerate(tiers):
        for layer in range(TIER_LAYERS):
            z = z0 + layer
            r = radius - TIER_STEP * layer
            if r < 1.0:
                continue
            ext = int(r) + 2
            for x in range(-ext, ext + 1):
                for y in range(-ext, ext + 1):
                    lobe = LOBE_AMP * math.cos(4.0 * math.atan2(y, x))
                    jitter = JITTER_AMP * (mirrored_noise(seed, x, y, z, ti) - 0.5)
                    if math.hypot(x, y) <= r + lobe + jitter:
                        n = noise(seed, x, y, z, 3)
                        vox[(x, y, z)] = (NDL_D if n < 0.34
                                          else (NDL_M if n < 0.80 else NDL_L))

    # Spire: linear taper to a single-voxel tip.
    for z in range(spire_z, tip):
        t = (z - spire_z) / max(1, tip - 1 - spire_z)
        r = spire_r - (spire_r - 1.0) * t
        ext = int(r) + 1
        for x in range(-ext, ext + 1):
            for y in range(-ext, ext + 1):
                if math.hypot(x, y) <= r + 0.9 * (mirrored_noise(seed, x, y, z, 11) - 0.5):
                    vox.setdefault((x, y, z),
                                   NDL_M if noise(seed, x, y, z, 5) < 0.7 else NDL_D)
    vox[(0, 0, tip)] = NDL_M

    # Snow: coarse clumps on sky-facing needles, dilated once so drifts stay
    # contiguous rather than speckled. Snow never extends a column sideways,
    # and never above tip, so the bounding box stays fixed.
    tops = [p for p, c in vox.items()
            if c in NEEDLES and (p[0], p[1], p[2] + 1) not in vox]
    caps = set()
    for (x, y, z) in tops:
        if z >= tip - SNOW_TIP_VOXELS:
            caps.add((x, y, z))
            continue
        score = (0.78 * noise(seed, x // 4, y // 4, z // 4, 61)
                 + 0.22 * noise(seed, x, y, z, 62))
        if math.hypot(x, y) > 4.0:
            score += 0.10          # snow settles on the bough tips
        if score > 0.74:
            caps.add((x, y, z))
    grown = set(caps)
    for (x, y, z) in tops:
        if (x, y, z) in caps:
            continue
        if sum(1 for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
               if (x + dx, y + dy, z) in caps) >= 2:
            grown.add((x, y, z))
    for (x, y, z) in sorted(grown):
        vox[(x, y, z)] = SNOW_S if noise(seed, x, y, z, 31) < 0.18 else SNOW
        if z + 1 < height and noise(seed, x, y, z, 41) < 0.34:
            vox[(x, y, z + 1)] = SNOW

    return vox


# ---------------------------------------------------------------------------
# Greedy meshing with hidden-face culling
# ---------------------------------------------------------------------------
def cell_uv(cid):
    col, row = cid % 4, cid // 4
    return ((col * CELL + INSET) / ATLAS, (col * CELL + CELL - INSET) / ATLAS,
            (row * CELL + INSET) / ATLAS, (row * CELL + CELL - INSET) / ATLAS)


def greedy_mesh(vox, voxel_m):
    """Emit only faces exposed to air, merging co-planar same-colour runs.

    Vertices are NOT shared between quads; see check_mesh_properties().
    """
    verts, faces, uvs = [], [], []
    for d in range(3):
        u, v = (d + 1) % 3, (d + 2) % 3
        for s in (1, -1):
            slices = {}
            for p, c in vox.items():
                n = list(p)
                n[d] += s
                if tuple(n) in vox:
                    continue
                slices.setdefault(p[d], {})[(p[u], p[v])] = c
            for sl in sorted(slices):
                mask = slices[sl]
                used = set()
                for (a, b) in sorted(mask):
                    if (a, b) in used:
                        continue
                    c = mask[(a, b)]
                    w = 1
                    while mask.get((a + w, b)) == c and (a + w, b) not in used:
                        w += 1
                    hgt = 1
                    while all(mask.get((a + i, b + hgt)) == c
                              and (a + i, b + hgt) not in used
                              for i in range(w)):
                        hgt += 1
                    for i in range(w):
                        for j in range(hgt):
                            used.add((a + i, b + j))
                    plane = sl + 1 if s == 1 else sl
                    quad = [(a, b), (a + w, b), (a + w, b + hgt), (a, b + hgt)]
                    u0, u1, v0, v1 = cell_uv(c)
                    quv = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
                    if s == -1:
                        quad, quv = quad[::-1], quv[::-1]
                    idx = len(verts)
                    for (uu, vv) in quad:
                        co = [0, 0, 0]
                        co[d] = plane
                        co[u] = uu
                        co[v] = vv
                        # -0.5 on x/y centres an odd-width trunk on the origin;
                        # z is left unshifted so the base sits on the ground.
                        verts.append(((co[0] - 0.5) * voxel_m,
                                      (co[1] - 0.5) * voxel_m,
                                      co[2] * voxel_m))
                    faces.append((idx, idx + 1, idx + 2, idx + 3))
                    uvs.extend(quv)
    return verts, faces, uvs


# ---------------------------------------------------------------------------
# Blender scene assembly
# ---------------------------------------------------------------------------
def hex_to_bytes(hx):
    return tuple(int(hx[i:i + 2], 16) for i in (1, 3, 5))


def wipe_scene():
    """Start from a known-empty scene regardless of the user's startup file."""
    for ob in list(bpy.data.objects):
        bpy.data.objects.remove(ob, do_unlink=True)
    for coll in (bpy.data.meshes, bpy.data.materials, bpy.data.images):
        for item in list(coll):
            try:
                coll.remove(item)
            except (RuntimeError, ReferenceError):
                pass
    bpy.context.scene.unit_settings.system = 'METRIC'
    bpy.context.scene.unit_settings.scale_length = 1.0


def palette_cell_origin(index):
    """Bottom-left pixel of a palette cell, in Blender (bottom-up) pixel space."""
    return (index % 4) * CELL, (index // 4) * CELL


def encode_palette_png():
    """Encode the palette atlas as PNG bytes straight from the sheet's hex.

    Written by hand rather than via Image.pack() with no data: pack() on a
    GENERATED image re-encodes the generated source and throws away whatever
    was assigned to .pixels, and the glTF exporter then copies those (black)
    packed bytes verbatim into the GLB. Encoding here also keeps the shipped
    texels byte-exact to HEX_* above, with no float round-trip, and keeps the
    output deterministic since we control the compressor.
    """
    rgb = bytearray(ATLAS * ATLAS * 3)          # top-down rows, as PNG wants
    for i, hx in enumerate(PALETTE_HEX):
        r, g, b = hex_to_bytes(hx)
        cx, cy = palette_cell_origin(i)
        for y in range(cy, cy + CELL):
            row = ATLAS - 1 - y                 # flip: Blender is bottom-up
            for x in range(cx, cx + CELL):
                o = (row * ATLAS + x) * 3
                rgb[o:o + 3] = bytes((r, g, b))
    raw = b"".join(b"\x00" + bytes(rgb[y * ATLAS * 3:(y + 1) * ATLAS * 3])
                   for y in range(ATLAS))       # filter type 0 on every row

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
    # a second sRGB decode into the texels and ships a visibly too-dark tree.
    px = [0.0] * (ATLAS * ATLAS * 4)
    for i, hx in enumerate(PALETTE_HEX):
        cx, cy = palette_cell_origin(i)
        r, g, b = (c / 255.0 for c in hex_to_bytes(hx))
        for y in range(cy, cy + CELL):
            for x in range(cx, cx + CELL):
                o = (y * ATLAS + x) * 4
                px[o:o + 4] = [r, g, b, 1.0]
    img.pixels = px                 # kept consistent with the packed PNG
    img.colorspace_settings.name = 'sRGB'
    png = encode_palette_png()
    img.pack(data=png, data_len=len(png))   # exact bytes for the exporter
    img.source = 'FILE'
    img.filepath_raw = "//%s.png" % name    # names the embedded file only
    return img


def make_material(name, img):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    mat.use_backface_culling = True     # -> glTF omits doubleSided (false)
    mat.blend_method = 'OPAQUE'
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Metallic"].default_value = 0.0
    bsdf.inputs["Roughness"].default_value = 0.92
    if "Specular IOR Level" in bsdf.inputs:
        bsdf.inputs["Specular IOR Level"].default_value = 0.5   # no extensions
    tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
    tex.image = img
    tex.interpolation = 'Closest'       # palette cells must not blend
    tex.extension = 'CLIP'
    tex.location = (-400, 250)
    mat.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    return mat


def build_object(spec, verts, faces, uvs):
    name = "SM_VoxelPine_%s" % spec["label"]
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    me.update()
    uvl = me.uv_layers.new(name="UVMap")
    for i, uv in enumerate(uvs):
        uvl.data[i].uv = uv
    for poly in me.polygons:
        poly.use_smooth = False         # voxels are flat-shaded
    me.validate(verbose=False)
    img = make_palette_image("T_VoxelPine_Palette")
    me.materials.append(make_material("M_VoxelPine", img))
    ob = bpy.data.objects.new(name, me)
    bpy.context.collection.objects.link(ob)
    ob.location = (0.0, 0.0, 0.0)
    ob.rotation_euler = (0.0, 0.0, 0.0)
    ob.scale = (1.0, 1.0, 1.0)
    return ob


def export_glb(ob, path):
    for o in bpy.data.objects:
        o.select_set(False)
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.export_scene.gltf(
        filepath=path, export_format='GLB', use_selection=True,
        export_apply=True, export_yup=True, export_image_format='AUTO',
        export_materials='EXPORT', export_normals=True, export_texcoords=True,
        export_cameras=False, export_lights=False, export_animations=False,
    )


# ---------------------------------------------------------------------------
# Read the written GLB back and verify the artifact itself
# ---------------------------------------------------------------------------
_COMPONENT = {5120: ("b", 1), 5121: ("B", 1), 5122: ("h", 2),
              5123: ("H", 2), 5125: ("I", 4), 5126: ("f", 4)}
_COUNT = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}


def load_glb(path):
    with open(path, "rb") as fh:
        data = fh.read()
    gj, binary, off = None, None, 12
    while off < len(data):
        clen, ctype = struct.unpack_from("<II", data, off)
        chunk = data[off + 8: off + 8 + clen]
        if ctype == 0x4E4F534A:        # 'JSON'
            gj = json.loads(chunk.decode("utf-8"))
        elif ctype == 0x004E4942:      # 'BIN\0'
            binary = chunk
        off += 8 + clen
    return gj, binary, len(data)


def read_accessor(gj, binary, index):
    acc = gj["accessors"][index]
    fmt, size = _COMPONENT[acc["componentType"]]
    ncomp = _COUNT[acc["type"]]
    bv = gj["bufferViews"][acc["bufferView"]]
    base = bv.get("byteOffset", 0) + acc.get("byteOffset", 0)
    stride = bv.get("byteStride") or size * ncomp
    unpack = struct.Struct("<" + fmt * ncomp).unpack_from
    return [unpack(binary, base + i * stride) for i in range(acc["count"])]


def decode_png_rgb(png):
    """Minimal PNG reader for 8-bit truecolour, handling all five filters.

    Only used to verify the palette that actually shipped inside the GLB, so
    it must not assume our own encoder wrote it -- the exporter is free to
    re-encode with any filter it likes.
    """
    pos, idat, width, height, depth, colour = 8, b"", 0, 0, 0, 0
    while pos < len(png):
        length = struct.unpack_from(">I", png, pos)[0]
        tag = png[pos + 4:pos + 8]
        payload = png[pos + 8:pos + 8 + length]
        if tag == b"IHDR":
            width, height, depth, colour = struct.unpack_from(">IIBB", payload, 0)
        elif tag == b"IDAT":
            idat += payload
        pos += 12 + length
    if depth != 8 or colour not in (2, 6):
        raise ValueError("unsupported PNG: depth=%d colour=%d" % (depth, colour))
    nch = 3 if colour == 2 else 4
    stride = width * nch
    raw = zlib.decompress(idat)
    out = bytearray(height * stride)
    prev = bytearray(stride)
    p = 0
    for y in range(height):
        ftype = raw[p]
        p += 1
        line = bytearray(raw[p:p + stride])
        p += stride
        if ftype == 1:
            for i in range(nch, stride):
                line[i] = (line[i] + line[i - nch]) & 0xFF
        elif ftype == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 0xFF
        elif ftype == 3:
            for i in range(stride):
                a = line[i - nch] if i >= nch else 0
                line[i] = (line[i] + ((a + prev[i]) >> 1)) & 0xFF
        elif ftype == 4:
            for i in range(stride):
                a = line[i - nch] if i >= nch else 0
                b = prev[i]
                c = prev[i - nch] if i >= nch else 0
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pred = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pred) & 0xFF
        out[y * stride:(y + 1) * stride] = line
        prev = line
    return width, height, nch, bytes(out)


def palette_from_glb(gj, binary):
    """Return the shipped hex of every palette cell, read out of the GLB."""
    bv = gj["bufferViews"][gj["images"][0]["bufferView"]]
    start = bv.get("byteOffset", 0)
    png = binary[start:start + bv["byteLength"]]
    width, height, nch, pix = decode_png_rgb(png)
    shipped = []
    for i in range(len(PALETTE_HEX)):
        cx, cy = palette_cell_origin(i)
        x = cx + CELL // 2
        y = height - 1 - (cy + CELL // 2)       # flip back to top-down rows
        o = (y * width + x) * nch
        shipped.append("#%02X%02X%02X" % (pix[o], pix[o + 1], pix[o + 2]))
    return shipped


def signed_volume(positions, indices):
    """Divergence-theorem volume, the correct closure oracle for this mesh.

    Euler characteristic and edge-manifold tests are meaningless on an
    unwelded quad soup, but the signed volume still pins the hull down
    exactly: it equals voxel_count * voxel_size**3 only if every exposed face
    is present, none is duplicated, and all normals point outward.
    """
    total = 0.0
    for i in range(0, len(indices), 3):
        a = positions[indices[i][0]]
        b = positions[indices[i + 1][0]]
        c = positions[indices[i + 2][0]]
        total += (a[0] * (b[1] * c[2] - b[2] * c[1])
                  - a[1] * (b[0] * c[2] - b[2] * c[0])
                  + a[2] * (b[0] * c[1] - b[1] * c[0])) / 6.0
    return total


def check_mesh_properties(quads, nverts):
    """The mesh is a disconnected quad soup by design: nverts == quads * 4,
    with no vertex shared between quads. That is correct output for a greedy
    voxel mesher and it is deliberately left unwelded, but it means the asset
    does NOT support:

      * smooth / averaged vertex normals (no adjacency to average across)
      * subdivision surfaces
      * adjacency-based auto-LOD or decimation needing a connected manifold
      * collision generation that walks shared edges (convex decomposition and
        box/voxel colliders are fine; prefer a separate primitive collider)

    Greedy merging also leaves T-junctions where a large quad abuts smaller
    ones. Harmless here, since the volume check proves closure, but it is the
    other reason not to run adjacency algorithms over this mesh.
    """
    return nverts == quads * 4


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------
USAGE = ("usage: blender --background --python voxel_pine.py -- "
         "<type 1-4> <out.glb> [--seed N] [--voxel M]")


def parse_args(argv):
    argv = argv[argv.index("--") + 1:] if "--" in argv else []
    if len(argv) < 2:
        raise SystemExit(USAGE)
    try:
        ttype = int(argv[0])
    except ValueError:
        raise SystemExit("error: type must be an integer 1-4\n" + USAGE)
    if ttype not in TREE_TYPES:
        raise SystemExit("error: type must be one of %s\n%s"
                         % (sorted(TREE_TYPES), USAGE))
    out = argv[1]
    if not out.lower().endswith(".glb"):
        raise SystemExit("error: output must be a .glb path\n" + USAGE)
    seed, voxel, rest = DEFAULT_SEED, DEFAULT_VOXEL, list(argv[2:])
    while rest:
        flag = rest.pop(0)
        if flag == "--seed":
            seed = int(rest.pop(0))
        elif flag == "--voxel":
            voxel = float(rest.pop(0))
        else:
            raise SystemExit("error: unknown option %r\n%s" % (flag, USAGE))
    # Voxel size must be positive. Zero is the dangerous one: it collapses the mesh to a point
    # AND scales expected_h/expected_volume by the same zero, so every closure check passes
    # vacuously and the script reports OK on nothing. The oracle has to be independent of the
    # input it is checking. (Negative values already fail the checks, on sign.)
    if not voxel > 0.0:
        raise SystemExit("error: --voxel must be greater than 0 (got %r)\n%s" % (voxel, USAGE))
    return ttype, os.path.abspath(out), seed, voxel


def main():
    ttype, out, seed, voxel = parse_args(list(sys.argv))
    spec = TREE_TYPES[ttype]
    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)

    wipe_scene()
    vox = build_voxels(spec, seed)
    verts, faces, uvs = greedy_mesh(vox, voxel)
    ob = build_object(spec, verts, faces, uvs)
    export_glb(ob, out)

    # --- figures, read back out of the GLB that was just written -----------
    gj, binary, nbytes = load_glb(out)
    prim = gj["meshes"][0]["primitives"][0]
    pos_acc = gj["accessors"][prim["attributes"]["POSITION"]]
    positions = read_accessor(gj, binary, prim["attributes"]["POSITION"])
    indices = read_accessor(gj, binary, prim["indices"])
    uv_vals = read_accessor(gj, binary, prim["attributes"]["TEXCOORD_0"])

    lo, hi = pos_acc["min"], pos_acc["max"]         # glTF space: Y is up
    size = [hi[i] - lo[i] for i in range(3)]
    centre = [(hi[i] + lo[i]) / 2.0 for i in range(3)]
    vol = signed_volume(positions, indices)
    expected_vol = len(vox) * voxel ** 3
    expected_h = spec["height"] * voxel
    nverts, ntris, nquads = len(positions), len(indices) // 3, len(faces)
    nmats = len(gj.get("materials", []))
    nprims = len(gj["meshes"][0]["primitives"])
    shipped_palette = palette_from_glb(gj, binary)

    print(
        "FIGURES type=%d cells=%d seed=%d voxel=%.3f voxels=%d quads=%d "
        "verts=%d tris=%d bbox=%.3fx%.3fx%.3f centre_x=%+.6f centre_z=%+.6f "
        "min_y=%+.6f volume=%.6f expected_volume=%.6f materials=%d "
        "primitives=%d palette=%s glb_bytes=%d"
        % (ttype, spec["cells"], seed, voxel, len(vox), nquads, nverts, ntris,
           size[0], size[1], size[2], centre[0], centre[2], lo[1],
           vol, expected_vol, nmats, nprims, ",".join(shipped_palette), nbytes)
    )

    # --- checks -------------------------------------------------------------
    fails = []

    def check(ok, msg):
        if not ok:
            fails.append(msg)

    check(abs(centre[0]) < 1e-6,
          "bbox centre X is %+.6f, expected 0.000000 (asset leans in X)"
          % centre[0])
    check(abs(centre[2]) < 1e-6,
          "bbox centre Z is %+.6f, expected 0.000000 (asset leans in Z)"
          % centre[2])
    check(abs(lo[1]) < 1e-6,
          "bbox min Y is %+.6f, expected 0.000000 (base not on the ground)"
          % lo[1])
    check(abs(size[1] - expected_h) < 1e-4,
          "height is %.4f m, expected %.4f m" % (size[1], expected_h))
    check(abs(vol - expected_vol) <= max(1e-6, expected_vol * 1e-4),
          "signed volume %.6f != voxel volume %.6f (hull not closed, faces "
          "duplicated, or normals inverted)" % (vol, expected_vol))
    check(ntris == nquads * 2, "tris %d != quads*2 %d" % (ntris, nquads * 2))
    check(check_mesh_properties(nquads, nverts),
          "verts %d != quads*4 %d (not the expected unwelded quad soup)"
          % (nverts, nquads * 4))
    check(nmats == 1, "expected exactly 1 material, got %d" % nmats)
    check(nprims == 1,
          "expected exactly 1 primitive (1 draw call), got %d" % nprims)
    check(not gj.get("extensionsUsed"),
          "expected no glTF extensions, got %s" % gj.get("extensionsUsed"))
    check(gj["materials"][0].get("doubleSided", False) is False,
          "material should be single-sided")
    check(all(0.0 <= u <= 1.0 and 0.0 <= v <= 1.0 for u, v in uv_vals),
          "some UVs fall outside 0-1")
    check(len(gj.get("images", [])) == 1,
          "expected exactly 1 embedded palette image, got %d"
          % len(gj.get("images", [])))
    # Guards a real regression: Image.pack() with no data silently ships a
    # black atlas, which every check above still passes.
    check(shipped_palette == PALETTE_HEX,
          "shipped palette %s != sheet palette %s"
          % (",".join(shipped_palette), ",".join(PALETTE_HEX)))
    check(gj["samplers"][0].get("magFilter") == 9728,
          "palette magFilter is %s, expected 9728 (NEAREST)"
          % gj["samplers"][0].get("magFilter"))

    if fails:
        for msg in fails:
            sys.stderr.write("FAIL: %s\n" % msg)
        sys.stderr.write("%d check(s) failed\n" % len(fails))
        sys.exit(1)
    print("OK %s type %d -> %s" % (spec["label"], ttype, out))
    sys.exit(0)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        # main() and parse_args already exit with a meaningful code; assert-style failures use 1.
        raise
    except Exception as error:
        # Blender's --background runner prints a traceback for ANY uncaught exception and still
        # exits 0, so a bad --seed/--voxel value, an unwritable output path, or an export failure
        # would report success having written nothing. This is the same guard valley_bench.py and
        # spike_pine_render.py carry, and the asset contract's clause 6 ("Exit 0 with no output is
        # not a result") requires it of the generator too.
        traceback.print_exc()
        raise SystemExit("generator failed: %s: %s" % (type(error).__name__, error)) from error
