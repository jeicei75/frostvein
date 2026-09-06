"""Render the dwarf's orthographic turnaround, for comparison against the reference.

    blender --background --python render_dwarf.py -- <out_dir>

Builds the model by importing dwarf_miner -- NOT by loading a .blend -- so the
renders are of the same geometry the generator ships and cannot drift from it.

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


def build():
    pine.wipe_scene()
    vox, _parts = dwarf.build_voxels()
    vox, _parts, _shift = dwarf.centre_voxels(vox, _parts)
    verts, faces, uvs = dwarf.build_mesh(vox, dwarf.DEFAULT_VOXEL)
    return dwarf.build_object(verts, faces, uvs)


def add_camera(ob):
    cam_data = bpy.data.cameras.new("TurnaroundCam")
    cam_data.type = 'ORTHO'
    cam_data.ortho_scale = max(ob.dimensions) * 1.06   # square frame: fits every view
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


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    positional = [a for a in argv if not a.startswith("--")]
    out_dir = os.path.abspath(positional[0]) if positional else "renders"
    engine = "workbench"
    if "--engine" in argv:
        engine = argv[argv.index("--engine") + 1]
    if engine not in ("workbench", "cycles"):
        raise SystemExit("error: --engine must be workbench or cycles (got %r)" % engine)
    os.makedirs(out_dir, exist_ok=True)

    ob = build()
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

    target = mathutils.Vector((0.0, 0.0, ob.dimensions.z / 2.0))
    cam = add_camera(ob)

    default_transform = scene.view_settings.view_transform   # before any engine setup moves it

    if engine == "cycles":
        setup_cycles(scene)
        add_studio_lights(target)
        lit_material = ob.data.materials[0]
        flat_material = make_flat_material(ob)

    # The flat pass is only "unlit albedo" if the VIEW TRANSFORM is Standard. Blender's default
    # (AgX) rolls highlights off and quietly rewrites every colour: measured on the Workbench flat
    # renders committed on 2026-09-06, ZERO of the ten palette hexes survived to the PNG -- skin
    # #E9D2BB read back as #BDB3AA -- so a palette read off that frame is wrong in silence. Set per
    # mode rather than globally so the lit pass keeps whatever look it was judged under.
    written = []
    for mode, light in (("flat", 'FLAT'), ("lit", 'STUDIO')):
        shading.light = light
        scene.view_settings.view_transform = 'Standard' if mode == "flat" else default_transform
        if engine == "cycles":
            ob.data.materials[0] = flat_material if mode == "flat" else lit_material
            # 1 sample is exact for pure emission and there is nothing to converge; the lit pass
            # has real light transport and needs more.
            scene.cycles.samples = 1 if mode == "flat" else 24
        for name, azimuth, elevation in VIEWS:
            aim(cam, target, azimuth, elevation)
            path = os.path.join(out_dir, "dwarf-%s-%s.png" % (mode, name))
            scene.render.filepath = path
            bpy.ops.render.render(write_still=True)
            written.append(path)

    assert_flat_is_flat(out_dir)

    sheet = contact_comparison(out_dir)
    for path in written + [sheet]:
        print("RENDER %s" % path)
    print("OK %d renders -> %s" % (len(written) + 1, out_dir))


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


if __name__ == "__main__":
    main()
