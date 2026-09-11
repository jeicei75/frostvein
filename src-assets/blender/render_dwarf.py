"""Render the dwarf's orthographic turnaround, for comparison against the reference.

    blender --background --python render_dwarf.py -- <out_dir> [--engine E]
                                                   [--reference REF.png]

Builds the model by importing dwarf_miner -- NOT by loading a .blend -- so the
renders are of the same geometry the generator ships and cannot drift from it.
That includes the SKELETON: the swing views are the rigged mesh actually bent by
its bones, not a second model posed by hand, which is the only way a render can
say anything about whether the rig works.

Three sets of output, and the third is a deliverable of round 3 rather than a
convenience:

  * five views x (flat, lit) in the neutral stance;
  * the same five lit, in the mp4's swing pose;
  * THE ZOOM STRIP, `dwarf-zoom-strip.png`: the lit front render downscaled
    nearest-neighbour to 10, 30 and 100 px tall and blown back up so the pixels
    are inspectable, beside the full-height frame. With free zoom the camera
    passes through every one of those sizes, so a detail that dissolves into
    speckle at 10 px is a defect and not a lost luxury -- and this is the frame
    that shows it.

Workbench, because it is deterministic and has no sampler noise. Two passes per
view: `flat` is unlit albedo, which is the only honest way to read the palette,
and `lit` is studio-lit, which is the only way to see that the forms are stepped
at all. The contact sheet is a torch-lit scene, so `lit` is the one to hold
beside it; neither is a claim about how the client will light him.

No camera, light or empty from this file ever reaches the GLB: the exporter runs
in dwarf_miner.py, in its own process, before any of this exists.
"""

import math
import os
import struct
import sys
import zlib

import bpy
import mathutils
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dwarf_miner as dwarf                                 # noqa: E402
import voxel_pine as pine                                   # noqa: E402

RES = (700, 700)
# (name, azimuth about +Z from +Y, elevation). He faces +Y, so his front is seen
# from +Y and his right (which carries the pickaxe) is +X.
VIEWS = [
    ("front", 0.0, 0.0),
    ("side-right", 90.0, 0.0),
    ("back", 180.0, 0.0),
    ("side-left", 270.0, 0.0),
    ("three-quarter", 35.0, 20.0),
]


def build(pose):
    """The generator's own build, rig and pose -- imported, never reimplemented."""
    pine.wipe_scene()
    ob, _arm = dwarf.build(dwarf.DEFAULT_VOXEL, pose)[:2]
    return ob


def deformed_extent(ob):
    """The posed mesh's half-extent, which is NOT ob.dimensions.

    ob.dimensions measures the rest mesh; the armature modifier is a deformation
    the depsgraph applies afterwards, so a swing that throws the pickaxe forward
    is invisible to it and the camera crops the tool off. Evaluate and measure
    what will actually be drawn.
    """
    graph = bpy.context.evaluated_depsgraph_get()
    evaluated = ob.evaluated_get(graph)
    mesh = evaluated.to_mesh()
    try:
        points = [ob.matrix_world @ v.co for v in mesh.vertices]
    finally:
        evaluated.to_mesh_clear()
    lo = mathutils.Vector((min(p[i] for p in points) for i in range(3)))
    hi = mathutils.Vector((max(p[i] for p in points) for i in range(3)))
    return lo, hi


def add_camera(extent):
    lo, hi = extent
    cam_data = bpy.data.cameras.new("TurnaroundCam")
    cam_data.type = 'ORTHO'
    cam_data.ortho_scale = max(hi[i] - lo[i] for i in range(3)) * 1.06   # fits every view
    cam = bpy.data.objects.new("TurnaroundCam", cam_data)
    bpy.context.collection.objects.link(cam)
    bpy.context.scene.camera = cam
    return cam


def aim(cam, target, azimuth_deg, elevation_deg, radius=6.0):
    a, e = math.radians(azimuth_deg), math.radians(elevation_deg)
    offset = mathutils.Vector((math.sin(a) * math.cos(e),
                               math.cos(a) * math.cos(e),
                               math.sin(e))) * radius
    cam.location = target + offset
    # Look at the target: -Z of the camera points along the view direction.
    cam.rotation_euler = (-offset).to_track_quat('-Z', 'Y').to_euler()


# Workbench is the default and the better instrument: deterministic, no sampler noise, and
# `light=FLAT` is exactly unlit albedo. It renders through EGL, and `libEGL.so.1` is ABSENT on the
# forge devpod -- Blender aborts there with exit 134 before writing anything. That left the asset
# reproducible from the forge but its RENDERS reproducible only on the art seat, so the committed
# PNGs were the only evidence of the look: the failure the "script is the durable record" clause
# exists to prevent, one level up from the generator.
#
# So: `-- <out_dir> --engine cycles` renders the same views on CPU, which needs no EGL. It is
# slower and carries sampler noise, and the two engines' `lit` passes are NOT pixel-comparable --
# use one engine for any comparison. Denoising is off because the venue has no denoiser.
#
# There is no auto-fallback on purpose. Workbench does not raise when EGL is missing, it ABORTS the
# process, so there is nothing to catch; a caller that cannot use it has to say so.
def atlas_image(ob):
    """The palette texture out of the object's own material, not a re-load from disk."""
    tree = ob.data.materials[0].node_tree
    return next(n.image for n in tree.nodes if n.type == 'TEX_IMAGE')


def make_flat_material(ob):
    """Unlit albedo under Cycles: emission at strength 1 with a Standard view transform means the
    rendered pixel IS the palette hex. Any other view transform (AgX is the default) rolls the
    highlights off and quietly rewrites the colours you are trying to read."""
    material = bpy.data.materials.new("DwarfFlat")
    material.use_nodes = True
    tree = material.node_tree
    tree.nodes.clear()
    out = tree.nodes.new('ShaderNodeOutputMaterial')
    emission = tree.nodes.new('ShaderNodeEmission')
    tex = tree.nodes.new('ShaderNodeTexImage')
    tex.image, tex.interpolation, tex.extension = atlas_image(ob), 'Closest', 'EXTEND'
    tree.links.new(tex.outputs['Color'], emission.inputs['Color'])
    tree.links.new(emission.outputs['Emission'], out.inputs['Surface'])
    return material


def add_studio_lights(target):
    for name, offset, energy in (("key", (-3.0, -4.0, 4.0), 900.0),
                                 ("fill", (4.0, -3.0, 1.5), 250.0),
                                 ("rim", (2.0, 4.0, 3.0), 400.0)):
        data = bpy.data.lights.new(name, 'AREA')
        data.energy, data.size = energy, 4.0
        lamp = bpy.data.objects.new(name, data)
        bpy.context.collection.objects.link(lamp)
        lamp.location = target + mathutils.Vector(offset)
        direction = target - lamp.location
        lamp.rotation_euler = direction.to_track_quat('-Z', 'Y').to_euler()


def setup_cycles(scene):
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.use_denoising = False
    # The view transform is deliberately NOT set here -- it is per-mode, below. Setting it in this
    # function captured 'Standard' into `default_transform` and so forced the LIT pass onto it too,
    # and left the per-mode line unable to fail. Caught by sabotaging that line and watching the
    # flat-pass check survive when it should have died.
    scene.world.use_nodes = True
    scene.world.node_tree.nodes["Background"].inputs[0].default_value = (0.16, 0.16, 0.17, 1.0)


# The pose the swing views are rendered in, and the sizes the zoom strip asks about.
# 10 px is the wide end measured through the client's own projection oracle -- a 1.20 m
# dwarf draws 8.74 px at the shipped boot framing -- and full height is the marketing
# shot. With free zoom the camera passes through everything between, so both ends and
# two points in the middle are what a detail has to survive.
ZOOM_HEIGHTS = (10, 30, 100, RES[1])
# Each panel is blown back up to the full render's own height, so the last panel is
# native and the others are INTEGER magnifications of it -- 70x, 23x, 7x. A panel
# smaller than the render would have forced the full-height frame through a
# fractional downscale, which is the one filter this strip exists to avoid.
ZOOM_PANEL = RES[1]
ZOOM_GUTTER = 8


def parse_argv():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    positional, flags, rest = [], {}, list(argv)
    while rest:
        item = rest.pop(0)
        if item.startswith("--"):
            flags[item] = rest.pop(0) if rest and not rest[0].startswith("--") else True
        else:
            positional.append(item)
    return positional, flags


def main():
    positional, flags = parse_argv()
    out_dir = os.path.abspath(positional[0]) if positional else "renders"
    engine = flags.get("--engine", "workbench")
    reference = flags.get("--reference")
    if engine not in ("workbench", "cycles"):
        raise SystemExit("error: --engine must be workbench or cycles (got %r)" % engine)
    os.makedirs(out_dir, exist_ok=True)

    written = []
    for pose in ("neutral", "swing"):
        written += render_pose(pose, engine, out_dir)

    assert_flat_is_flat(out_dir)
    extras = [zoom_strip(out_dir), contact_comparison(out_dir)]
    if reference and reference is not True:
        extras.append(reference_comparison(out_dir, os.path.abspath(reference)))
    for path in written + extras:
        print("RENDER %s" % path)
    print("OK %d renders -> %s" % (len(written) + len(extras), out_dir))


def render_pose(pose, engine, out_dir):
    """One full turnaround of one pose. The scene is rebuilt per pose rather than
    re-posed, because the camera has to be framed on the DEFORMED extent and the
    flat/lit material swap is per-object state."""
    ob = build(pose)
    scene = bpy.context.scene
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.render.resolution_x, scene.render.resolution_y = RES
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = 'PNG'
    scene.render.film_transparent = False
    scene.world = bpy.data.worlds.new("Flat") if scene.world is None else scene.world
    scene.world.color = (0.16, 0.16, 0.17)

    shading = scene.display.shading
    shading.color_type = 'TEXTURE'
    shading.show_object_outline = False
    shading.show_specular_highlight = False

    # Aim at the DEFORMED bounding box's centre on all three axes. The neutral
    # pose centres on X and Y by contract, so r2 could hardcode those to zero; a
    # swing throws the pickaxe forward in Y and the frame slid off him.
    lo, hi = deformed_extent(ob)
    target = mathutils.Vector(tuple((lo[i] + hi[i]) / 2.0 for i in range(3)))
    cam = add_camera((lo, hi))

    default_transform = scene.view_settings.view_transform   # before any engine setup moves it

    flat_material = lit_material = None
    if engine == "cycles":
        setup_cycles(scene)
        add_studio_lights(target)
        lit_material = ob.data.materials[0]
        flat_material = make_flat_material(ob)

    # The neutral pose carries both passes; the swing carries the lit one only. The
    # flat pass exists to read the PALETTE, and the palette does not change when a
    # bone turns -- a second set of flat views would be five more files saying the
    # same thing, and assert_flat_is_flat already reads the neutral ones.
    modes = (("flat", 'FLAT'), ("lit", 'STUDIO')) if pose == "neutral" else (("lit", 'STUDIO'),)

    # The flat pass is only "unlit albedo" if the VIEW TRANSFORM is Standard. Blender's default
    # (AgX) rolls highlights off and quietly rewrites every colour: measured on the Workbench flat
    # renders committed on 2026-09-06, ZERO of the ten palette hexes survived to the PNG -- skin
    # #E9D2BB read back as #BDB3AA -- so a palette read off that frame is wrong in silence. Set per
    # mode rather than globally so the lit pass keeps whatever look it was judged under.
    written = []
    for mode, light in modes:
        shading.light = light
        scene.view_settings.view_transform = 'Standard' if mode == "flat" else default_transform
        if engine == "cycles":
            ob.data.materials[0] = flat_material if mode == "flat" else lit_material
            # 1 sample is exact for pure emission and there is nothing to converge; the lit pass
            # has real light transport and needs more.
            scene.cycles.samples = 1 if mode == "flat" else 24
        for name, azimuth, elevation in VIEWS:
            aim(cam, target, azimuth, elevation)
            stem = "dwarf-%s-%s" % (mode, name) if pose == "neutral"                 else "dwarf-swing-%s-%s" % (mode, name)
            path = os.path.join(out_dir, "%s.png" % stem)
            scene.render.filepath = path
            bpy.ops.render.render(write_still=True)
            written.append(path)
    return written


def assert_flat_is_flat(out_dir):
    """The flat pass claims to be readable albedo. Make it prove that rather than assert it.

    Every palette colour must survive to the PNG as its exact hex somewhere across the flat views.
    A view transform, a colour-managed image node or a stray light shifts all of them at once, so
    this fails loudly on the whole class rather than on one colour. Edge pixels are antialiased and
    land between palette entries, which is why the check asks whether each colour APPEARS, not
    whether every pixel is one.
    """
    seen = set()
    for name, _a, _e in VIEWS:
        frame = read_png(os.path.join(out_dir, "dwarf-flat-%s.png" % name))
        seen |= {tuple(int(c) for c in px) for px in frame.reshape(-1, 3)}
    wanted = {tuple(int(h.lstrip("#")[i:i + 2], 16) for i in (0, 2, 4)) for h in dwarf.PALETTE_HEX}
    missing = sorted("#%02X%02X%02X" % c for c in wanted - seen)
    if missing:
        raise SystemExit(
            "flat pass is not flat: %d of %d palette colours never reach the PNG (%s). The view "
            "transform is the usual cause -- it must be Standard for this pass."
            % (len(missing), len(wanted), ", ".join(missing))
        )
    print("FLAT-CHECK all %d palette colours reach the PNG exactly" % len(wanted))


# --- the comparison the brief asks for -------------------------------------
# The renders have to be judged against the contact sheet, so put them in one
# image with a frame of it rather than leaving that to whoever opens the folder.
CONTACT = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "..", "references", "dwarf-contact-sheet.jpg")
CONTACT_FRAME = (1600, 540, 320, 180)      # frame r3c5: the clearest near-front idle


def read_png(path):
    image = bpy.data.images.load(path)
    width, height = image.size
    buffer = np.empty(width * height * 4, dtype=np.float32)
    image.pixels.foreach_get(buffer)
    bpy.data.images.remove(image)
    return np.clip(buffer.reshape(height, width, 4)[::-1, :, :3] * 255.0 + 0.5,
                   0, 255).astype(np.uint8)


def nearest_scale(image, height):
    factor = height / image.shape[0]
    width = int(image.shape[1] * factor)
    rows = (np.arange(height) / factor).astype(int).clip(0, image.shape[0] - 1)
    cols = (np.arange(width) / factor).astype(int).clip(0, image.shape[1] - 1)
    return image[rows][:, cols]


def write_png(path, pixels):
    # No escape sequences here on purpose: this file is edited by scripts and a
    # literal NUL or CR in the source is a corruption that only shows up as a
    # SyntaxError three edits later.
    magic = bytes((137, 80, 78, 71, 13, 10, 26, 10))
    height, width, _ = pixels.shape
    raw = b"".join(bytes(1) + pixels[row].tobytes() for row in range(height))

    def chunk(tag, payload):
        return (struct.pack(">I", len(payload)) + tag + payload
                + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF))

    with open(path, "wb") as handle:
        handle.write(magic
                     + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
                     + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def contact_comparison(out_dir):
    tiles = [read_png(os.path.join(out_dir, "dwarf-lit-%s.png" % name))
             for name, _a, _e in VIEWS]
    x, y, w, h = CONTACT_FRAME
    tiles.append(read_png(os.path.abspath(CONTACT))[y:y + h, x:x + w])
    height = max(tile.shape[0] for tile in tiles)
    strip = [nearest_scale(tile, height) for tile in tiles]
    canvas = np.concatenate(strip, axis=1)
    path = os.path.join(out_dir, "dwarf-vs-contact-sheet.png")
    write_png(path, canvas)
    return path


def zoom_strip(out_dir):
    """The deliverable the round asks for: one render at four sizes, side by side.

    Each panel is the lit front view downscaled NEAREST-NEIGHBOUR to its height --
    the same filter the game's rasteriser approximates -- and then blown back up
    by an integer factor so the pixels can be counted. The true-size image is
    inset at the bottom left of its panel, because a 10 px dwarf blown up to 420
    stops looking like 10 px and the point of the strip is what 10 px looks like.

    Two questions it has to answer, and they are the two the brief asks: does the
    silhouette still read as a bearded dwarf with a lantern at 10 px, and does any
    surface detail turn to speckle there. A detail that dissolves at 10 px is a
    defect and not a lost luxury -- with free zoom the camera passes through every
    one of these sizes, and speckle that changes frame to frame shimmers.
    """
    full = read_png(os.path.join(out_dir, "dwarf-lit-front.png"))
    panels = []
    for height in ZOOM_HEIGHTS:
        small = nearest_scale(full, height)
        factor = max(1, ZOOM_PANEL // height)
        blown = np.repeat(np.repeat(small, factor, axis=0), factor, axis=1)
        panel = np.full((ZOOM_PANEL, ZOOM_PANEL, 3), 24, dtype=np.uint8)
        oy, ox = (ZOOM_PANEL - blown.shape[0]) // 2, (ZOOM_PANEL - blown.shape[1]) // 2
        panel[oy:oy + blown.shape[0], ox:ox + blown.shape[1]] = blown
        if height < ZOOM_PANEL:
            # the same frame at its TRUE size, inset bottom-left with a one-pixel
            # rule. A 10 px dwarf blown up to 700 stops looking like 10 px, and
            # what 10 px looks like is the whole question.
            th, tw = small.shape[:2]
            panel[ZOOM_PANEL - th - 9:ZOOM_PANEL - 9, 8:8 + tw] = small
            panel[ZOOM_PANEL - th - 10, 7:9 + tw] = 200
        panels.append(panel)
        gutter = np.full((ZOOM_PANEL, ZOOM_GUTTER, 3), 90, dtype=np.uint8)
        panels.append(gutter)
    canvas = np.concatenate(panels[:-1], axis=1)
    path = os.path.join(out_dir, "dwarf-zoom-strip.png")
    write_png(path, canvas)
    return path


def reference_comparison(out_dir, reference):
    """The five lit views beside a frame of the round's actual authority.

    `references/dwarf.mp4` is a video and nothing in this script can decode one,
    so the frame is passed in rather than extracted -- see ASSET_NOTES for the
    one ffmpeg line that produces it. Passing it keeps the comparison
    reproducible from the script plus a stated command, which the alternative
    (a hand-made PNG committed with no recipe) does not.
    """
    tiles = [read_png(os.path.join(out_dir, "dwarf-lit-%s.png" % name))
             for name, _a, _e in VIEWS]
    tiles.append(read_png(reference))
    height = max(tile.shape[0] for tile in tiles)
    canvas = np.concatenate([nearest_scale(tile, height) for tile in tiles], axis=1)
    path = os.path.join(out_dir, "dwarf-vs-dwarf-mp4.png")
    write_png(path, canvas)
    return path


if __name__ == "__main__":
    main()
