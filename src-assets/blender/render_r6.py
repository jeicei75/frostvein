"""Renders for the round-6 dwarf. Round 4's settings, inherited unchanged.

Workbench, because it is deterministic and has no sampler noise. `flat` is unlit
albedo -- the only honest read of the palette -- and `lit` is studio-lit, the only
way to see that the forms are stepped at all. Neither is a claim about how the
client will light him; that is Epic 11's.

    blender --background <blend> --python src-assets/blender/render_r5.py -- \
            <out-dir> <mode> [res]

    mode `progress`  front/side/three-quarter, flat and lit, into <out-dir>
    mode `final`     the five delivered views, flat and lit, into src-assets/renders
"""

import math
import os
import sys

import bpy
from mathutils import Vector


def _srgb_to_linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


# #6F7073, the flat ground round 4 used, so background subtraction stays trivial.
# The world colour is scene-referred, so it is stored LINEAR and comes back out as
# #6F7073 once the Standard view transform re-encodes it.
BG = tuple(_srgb_to_linear(c / 255.0) for c in (0x6F, 0x70, 0x73))


COLLECTION = "SM_VoxelDwarf_Miner01_r6"


def figure_bounds():
    """Only the r6 collection. The .blend still carries r5 as a reference, parked
    out of the view layer -- it does not render, but it WOULD have been measured
    here, and a span taken over both figures frames neither."""
    lo = Vector((1e9, 1e9, 1e9))
    hi = Vector((-1e9, -1e9, -1e9))
    for ob in bpy.data.collections[COLLECTION].objects:
        if ob.type != 'MESH' or ob.hide_render:
            continue
        for v in ob.data.vertices:
            w = ob.matrix_world @ v.co
            lo = Vector((min(lo[i], w[i]) for i in range(3)))
            hi = Vector((max(hi[i], w[i]) for i in range(3)))
    return lo, hi


def setup(res, lit):
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


def place_camera(azimuth, elevation, margin=1.06, span=None, centre=None):
    lo, hi = figure_bounds()
    if centre is None:
        centre = (lo + hi) / 2.0
    if span is None:
        span = max(hi.x - lo.x, hi.y - lo.y, hi.z - lo.z)
    cam = bpy.data.objects.get("R6Cam")
    if cam is None:
        cam = bpy.data.objects.new("R6Cam", bpy.data.cameras.new("R6Cam"))
        bpy.context.scene.collection.objects.link(cam)
    cam.data.type = 'ORTHO'
    cam.data.ortho_scale = span * margin
    a, e = math.radians(azimuth), math.radians(elevation)
    d = span * 3.0
    # He faces +Y, so azimuth 0 -- his FRONT -- is seen from +Y, the convention
    # render_dwarf.py already used. His right, which carries the pickaxe, is +X.
    cam.location = centre + Vector((math.sin(a) * math.cos(e),
                                    math.cos(a) * math.cos(e),
                                    math.sin(e))) * d
    cam.rotation_euler = (centre - cam.location).to_track_quat('-Z', 'Y').to_euler()
    bpy.context.scene.camera = cam
    return cam


def shot(path, azimuth=0.0, elevation=0.0, res=700, lit=True, span=None):
    setup(res, lit)
    place_camera(azimuth, elevation, span=span)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.context.scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    return path


VIEWS = [("front", 0.0), ("side-left", -90.0), ("side-right", 90.0),
         ("back", 180.0), ("three-quarter", 35.0)]


def main():
    argv = sys.argv[sys.argv.index("--") + 1:]
    # Blender resolves a RELATIVE render.filepath against the .blend, not the shell's
    # working directory, so a relative out-dir silently lands on the drive root.
    out, mode = os.path.abspath(argv[0]), argv[1]
    res = int(argv[2]) if len(argv) > 2 else 700
    for ob in list(bpy.data.objects):
        if ob.type in {'CAMERA', 'LIGHT'}:
            bpy.data.objects.remove(ob, do_unlink=True)

    lo, hi = figure_bounds()
    span = max(hi.x - lo.x, hi.y - lo.y, hi.z - lo.z, 1.25)

    if mode == "progress":
        # a FIXED frame for every progress sheet, so a part cannot appear to
        # change size between steps and the figure is always shown whole
        span, centre = 1.42, Vector((0.0, 0.0, 0.60))
        for path, az, el in (("flat-front.png", 0.0, 0.0),
                             ("lit-three-quarter.png", 35.0, 12.0),
                             ("lit-side-left.png", -90.0, 0.0),
                             ("lit-back.png", 180.0, 0.0)):
            setup(res, path.startswith("lit"))
            place_camera(az, el, span=span, centre=centre)
            os.makedirs(out, exist_ok=True)
            bpy.context.scene.render.filepath = os.path.join(out, path)
            bpy.ops.render.render(write_still=True)
    else:
        base = os.path.join(out)
        for name, az in VIEWS:
            el = 12.0 if name == "three-quarter" else 0.0
            shot(os.path.join(base, "dwarf-flat-%s.png" % name), az, el, res, False, span)
            shot(os.path.join(base, "dwarf-lit-%s.png" % name), az, el, res, True, span)
    print("RENDERED mode=%s span=%.4f -> %s" % (mode, span, out))


if __name__ == "__main__":
    main()
