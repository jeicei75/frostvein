"""Round 6's authoring kit: the PRIMITIVES only, no part specs.

Round 5 kept its part specs in a module and re-ran a builder that wiped and
rebuilt the whole figure in forty seconds, which is what made that session
unwatchable. So this file deliberately holds no dwarf: it holds the measurement
frame, the palette cell ids, the box and ROTATED-box mesh maths, and the
progress renderer. Every part's numbers are authored in the live Blender session
that creates it, one part per call, and the .blend is the artifact.

THE MEASUREMENT FRAME, unchanged from round 5 and re-verified against the crops:
the figure spans rows 7 (crown) .. 147 (sole) in front.png and side-left.png, so
140 source px == 1.20 m and one source px == 8.571 mm; front.png column 77 is the
centre line -> X; side-left.png column 58 is the depth centre, +Y forward.
"""

import importlib
import math
import os
import sys

import bpy
from mathutils import Euler, Vector

REV = "r6"
ASSET = "SM_VoxelDwarf_Miner01"
COLLECTION = "%s_%s" % (ASSET, REV)
MATERIAL = "M_VoxelDwarf_%s" % REV
HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.dirname(HERE)
BLEND = os.path.join(HERE, "%s.blend" % ASSET)
RENDERS = os.path.join(SRC, "renders", REV)

HEIGHT = 1.20
PXH = 140.0
U = HEIGHT / PXH
ROW_SOLE, COL_X0, COL_Y0 = 147.0, 77.0, 58.0


def Z(row):
    return (ROW_SOLE - row) * U


def Y(col):
    return (col - COL_Y0) * U


def P(px):
    return px * U


# --- the inherited palette, cell ids into the r5 atlas ------------------------
SKIN, BEARD, SNOW, TUNIC, PANTS, METAL, WOOD, TRUNK, HAIR, FLAME = range(10)
SKIN_DK, BEARD_LT, TUNIC_LT, TUNIC_DK, PANTS_LT = 10, 11, 12, 13, 14
SPARE = 15
ATLAS, CELL = 64, 16
FACE_DIRS = ("xlo", "xhi", "ylo", "yhi", "zlo", "zhi")


def cell_uv(cid):
    col, row = cid % 4, cid // 4
    return ((col * CELL + CELL / 2) / ATLAS, (row * CELL + CELL / 2) / ATLAS)


def shade(base, lit=None, dark=None, front=None, back=None):
    """Round 5's value convention, inherited: the step is TOP vs UNDERSIDE, never
    front vs back -- painting -Y dark turns the whole back view to mud."""
    return {"xlo": base, "xhi": base,
            "ylo": base if back is None else back,
            "yhi": base if front is None else front,
            "zlo": base if dark is None else dark,
            "zhi": base if lit is None else lit}


# --- solids -------------------------------------------------------------------
def _corners(lo, hi):
    x0, y0, z0 = lo
    x1, y1, z1 = hi
    return {
        "xlo": [(x0, y0, z0), (x0, y1, z0), (x0, y1, z1), (x0, y0, z1)],
        "xhi": [(x1, y1, z0), (x1, y0, z0), (x1, y0, z1), (x1, y1, z1)],
        "ylo": [(x1, y0, z0), (x0, y0, z0), (x0, y0, z1), (x1, y0, z1)],
        "yhi": [(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)],
        "zlo": [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0)],
        "zhi": [(x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1)],
    }


def B(r0, r1, x0, x1, y0, y1, cells, rot=None, pivot=None):
    """One box in SOURCE-PIXEL coordinates -> a solid (verts, faces, uvs).

    r0/r1 are front-view rows; x0/x1 and y0/y1 are both OFFSETS IN SOURCE PIXELS
    from the body's centre line and from its depth centre -- x0/x1 off front.png
    column 77, y0/y1 off side-left.png column 58. `rot` is (axis, degrees) or a
    list of them, applied about `pivot` (x-offset, y-offset, row; default the box
    centre). A rigid rotation preserves planarity EXACTLY, which is why rotated
    boxes are legal since 2026-09-11 and smoothing still is not.

    NOTE: round 5 took y0/y1 as absolute side-left COLUMNS and this took them as
    offsets for fifteen parts before anyone noticed, which built the whole figure
    0.497 m behind the origin -- self-consistent, so it looked right, until a part
    authored the other way was put beside it. Offsets win because that is what the
    figure is already in; use Y() only to convert a column you have just counted.
    """
    lo = (P(min(x0, x1)), P(min(y0, y1)), Z(max(r0, r1)))
    hi = (P(max(x0, x1)), P(max(y0, y1)), Z(min(r0, r1)))
    if isinstance(cells, int):
        cells = {d: cells for d in FACE_DIRS}
    corner = _corners(lo, hi)

    xf = None
    if rot:
        rots = [rot] if isinstance(rot[0], str) else list(rot)
        if pivot is None:
            piv = Vector(((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2))
        else:
            piv = Vector((P(pivot[0]), P(pivot[1]), Z(pivot[2])))
        m = Euler((0.0, 0.0, 0.0), 'XYZ').to_matrix()
        for axis, deg in rots:
            e = Euler((0.0, 0.0, 0.0), 'XYZ')
            setattr(e, axis.lower(), math.radians(deg))
            m = e.to_matrix() @ m
        xf = lambda p: tuple(m @ (Vector(p) - piv) + piv)

    verts, faces, uvs = [], [], []
    for d in FACE_DIRS:
        cid = cells.get(d)
        if cid is None:
            continue
        pts = corner[d] if xf is None else [xf(p) for p in corner[d]]
        base = len(verts)
        verts.extend(pts)
        faces.append((base, base + 1, base + 2, base + 3))
        uvs.extend([cell_uv(cid)] * 4)
    return verts, faces, uvs


def panel(y_col, rows):
    """Tile a +Y-facing plane with flat regions that EXACTLY tile it -- colour per
    face, no hovering decal and no z-fighting. rows = [(r0, r1, [(x0,x1,cell)..])]"""
    verts, faces, uvs = [], [], []
    yy = P(y_col)
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


def _weld(solids):
    verts, faces, uvs = [], [], []
    for v, f, u in solids:
        off = len(verts)
        verts.extend(v)
        faces.extend(tuple(i + off for i in q) for q in f)
        uvs.extend(u)
    return verts, faces, uvs


def _mesh(name, verts, faces, uvs):
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    me.validate()
    layer = me.uv_layers.new(name="UVMap")
    for i, uv in enumerate(uvs):
        layer.data[i].uv = uv
    for poly in me.polygons:
        poly.use_smooth = False
    me.materials.append(bpy.data.materials[MATERIAL])
    return me


def part(name, solids):
    """Create or REPLACE one named part. Replacing ONE part is not the
    wipe-and-rebuild the brief forbids -- every other part is untouched."""
    verts, faces, uvs = _weld(solids)
    ob = bpy.data.objects.get(name)
    me = _mesh(name, verts, faces, uvs)
    if ob is not None:
        old = ob.data
        ob.data = me
        if old.users == 0:
            bpy.data.meshes.remove(old)
        me.name = name
        return ob
    ob = bpy.data.objects.new(name, me)
    bpy.data.collections[COLLECTION].objects.link(ob)
    return ob


def add_to(name, solids):
    """APPEND solids to a part already in the scene -- editing the object, which
    is what the brief asks for once the blockout is standing."""
    ob = bpy.data.objects[name]
    me = ob.data
    uvl = me.uv_layers[0].data
    verts = [v.co[:] for v in me.vertices]
    faces = [tuple(p.vertices) for p in me.polygons]
    uvs = [uvl[l].uv[:] for p in me.polygons for l in p.loop_indices]
    off = len(verts)
    nv, nf, nu = _weld(solids)
    verts += nv
    faces += [tuple(i + off for i in q) for q in nf]
    uvs += nu
    new = _mesh(name, verts, faces, uvs)
    ob.data = new
    if me.users == 0:
        bpy.data.meshes.remove(me)
    new.name = name
    return ob


def drop(*names):
    for name in names:
        ob = bpy.data.objects.get(name)
        if ob is None:
            continue
        me = ob.data
        bpy.data.objects.remove(ob, do_unlink=True)
        if getattr(me, "users", 1) == 0:
            bpy.data.meshes.remove(me)


def socket(name, px_loc, child):
    """A declared mount point, so a variant is a part list and nothing else."""
    emp = bpy.data.objects.get(name)
    if emp is None:
        emp = bpy.data.objects.new(name, None)
        emp.empty_display_type = 'PLAIN_AXES'
        emp.empty_display_size = 0.03
        bpy.data.collections[COLLECTION].objects.link(emp)
    emp.location = (P(px_loc[0]), P(px_loc[1]), Z(px_loc[2]))
    bpy.context.view_layer.update()
    child.parent = emp
    child.matrix_parent_inverse = emp.matrix_world.inverted()
    return emp


# --- the session's eyes -------------------------------------------------------
def tris():
    return sum(len(o.data.polygons) * 2
               for o in bpy.data.collections[COLLECTION].objects if o.type == 'MESH')


def report():
    coll = bpy.data.collections[COLLECTION]
    out = []
    for ob in sorted(coll.objects, key=lambda o: o.name):
        if ob.type != 'MESH':
            continue
        smooth = sum(1 for p in ob.data.polygons if p.use_smooth)
        out.append("  %-22s tris %4d  smooth %d  mods %d"
                   % (ob.name, len(ob.data.polygons) * 2, smooth, len(ob.modifiers)))
    n = sum(1 for o in coll.objects if o.type == 'MESH')
    return "\n".join(out) + "\n  TOTAL tris %d over %d parts" % (tris(), n)


def frame_viewport():
    """Leave the viewport usable and looking at the figure, front-on and ortho."""
    for w in bpy.context.window_manager.windows:
        for a in w.screen.areas:
            if a.type != 'VIEW_3D':
                continue
            for s in a.spaces:
                if s.type != 'VIEW_3D':
                    continue
                s.shading.type = 'SOLID'
                s.shading.color_type = 'TEXTURE'
                s.shading.light = 'STUDIO'
                s.region_3d.view_perspective = 'ORTHO'
                s.region_3d.view_rotation = Euler(
                    (math.radians(90), 0.0, math.radians(180)), 'XYZ').to_quaternion()
                s.region_3d.view_location = (0.0, 0.0, 0.62)
                s.region_3d.view_distance = 1.6


def _srgb_to_linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


BG = tuple(_srgb_to_linear(c / 255.0) for c in (0x6F, 0x70, 0x73))
SPAN, CENTRE = 1.42, Vector((0.0, 0.0, 0.60))     # a FIXED progress frame


def _setup(res, lit):
    scn = bpy.context.scene
    scn.render.engine = 'BLENDER_WORKBENCH'
    scn.render.resolution_x = scn.render.resolution_y = res
    scn.render.resolution_percentage = 100
    scn.render.film_transparent = False
    scn.render.image_settings.file_format = 'PNG'
    sh = scn.display.shading
    sh.light = 'STUDIO' if lit else 'FLAT'
    sh.color_type = 'TEXTURE'
    sh.show_object_outline = False
    sh.show_specular_highlight = False
    scn.world.use_nodes = False
    scn.world.color = BG
    scn.view_settings.view_transform = 'Standard'
    return scn


def _cam(azimuth, elevation, span=None, centre=None):
    span = SPAN if span is None else span
    centre = CENTRE if centre is None else centre
    cam = bpy.data.objects.get("R6Cam")
    if cam is None:
        cam = bpy.data.objects.new("R6Cam", bpy.data.cameras.new("R6Cam"))
        bpy.context.scene.collection.objects.link(cam)
    cam.data.type = 'ORTHO'
    cam.data.ortho_scale = span * 1.06
    a, e = math.radians(azimuth), math.radians(elevation)
    cam.location = centre + Vector((math.sin(a) * math.cos(e),
                                    math.cos(a) * math.cos(e),
                                    math.sin(e))) * span * 3.0
    cam.rotation_euler = (centre - cam.location).to_track_quat('-Z', 'Y').to_euler()
    bpy.context.scene.camera = cam
    return cam


def _render(path, azimuth, elevation, res, lit, span=None, centre=None):
    _setup(res, lit)
    _cam(azimuth, elevation, span, centre)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    # ABSOLUTE: Blender resolves a relative render.filepath against the .blend,
    # which is how round 5's renders escaped to C:\src-assets\renders\.
    bpy.context.scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    return path


def _read(path):
    import numpy as np
    img = bpy.data.images.load(path, check_existing=False)
    a = np.array(img.pixels[:], dtype="f4").reshape(img.size[1], img.size[0], 4)
    bpy.data.images.remove(img)
    return a[::-1, :, :3]                          # Blender's rows run bottom-up


PROGRESS_VIEWS = [("flat-front", 0.0, 0.0, False),
                  ("lit-three-quarter", 35.0, 12.0, True),
                  ("lit-side-left", -90.0, 0.0, True)]


def snap(nn, label, res=420, small=60):
    """Deliverable 4: the WHOLE figure, full size AND at ~60 px, one file per part.

    Four of six delegated runs in this project have been killed by the harness, so
    the progress renders are the audit trail and the save beside them is the
    crash insurance.
    """
    import numpy as np
    tmp = os.path.join(RENDERS, "progress", "_tmp")
    big = [_read(_render("%s-%s.png" % (tmp, n), az, el, res, lit))
           for n, az, el, lit in PROGRESS_VIEWS]
    tiny = [_read(_render("%s-%s-s.png" % (tmp, n), az, el, small, lit))
            for n, az, el, lit in PROGRESS_VIEWS]
    pad, k = 8, 3
    W = pad + len(big) * (res + pad)
    H = pad + res + pad + small * k + pad
    canvas = np.empty((H, W, 3), dtype="f4")
    canvas[:, :] = np.array(BG, dtype="f4")
    for i, im in enumerate(big):
        x = pad + i * (res + pad)
        canvas[pad:pad + res, x:x + res] = im
        up = np.repeat(np.repeat(tiny[i], k, 0), k, 1)
        canvas[pad + res + pad:pad + res + pad + small * k, x:x + small * k] = up
    out = os.path.join(RENDERS, "progress", "%02d-%s.png" % (nn, label))
    img = bpy.data.images.new("snap", W, H, alpha=False)
    img.pixels = np.concatenate(
        [canvas[::-1], np.ones((H, W, 1), dtype="f4")], axis=2).ravel()
    img.file_format = 'PNG'
    img.filepath_raw = out
    img.save()
    bpy.data.images.remove(img)
    for n, _, _, _ in PROGRESS_VIEWS:
        for suffix in ("", "-s"):
            p = "%s-%s%s.png" % (tmp, n, suffix)
            if os.path.exists(p):
                os.remove(p)
    return out


def save():
    bpy.ops.wm.save_mainfile(filepath=BLEND)
    return BLEND


def step(nn, label):
    """Save, snap and say where things stand -- the end of every part."""
    frame_viewport()
    p = snap(nn, label)
    save()
    print("STEP %02d %s" % (nn, label))
    print(report())
    print("SNAP  %s" % p)
    print("SAVED %s" % BLEND)


def reload():
    importlib.reload(sys.modules[__name__])
