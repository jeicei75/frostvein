"""Render a voxel-pine GLB headless, for story 10.2's handoff evidence.

Run with Blender: ``blender --background --python spike_pine_render.py -- ASSET.glb OUT.png``.

NOTE: SPIKE OUTPUT, NOT MACHINERY. Story 10.2 states this exception explicitly: this script is
not wired into `scripts/gate.sh` and carries no mutation table, because it exists to produce one
comparison image for a decision and may be superseded the moment that decision is recorded. Its
evidence is the executed two-run recipe in the story record, with its figures, standing in for a
standing test. If the decision keeps it, hardening it (test + sabotage row) is the follow-up's
first task. It deliberately does NOT touch `valley_bench.py`, which is calibrated machinery.

It reads nothing from any live session: the GLB path and the output path are the only inputs.
"""

import math
import sys
import traceback

try:
    import bpy
except ImportError:
    bpy = None


# A neutral studio backdrop, NOT the client's sky: this image answers "did the handoff keep the
# look of the ASSET", so the frame must not smuggle in scene lighting the asset does not own.
#
# HELD AS DISPLAY-REFERRED sRGB 0-255, and converted to linear only when it reaches the world
# shader. Reason, hit here before it was fixed: `image.pixels` reads back DISPLAY-referred, so
# comparing it against a LINEAR backdrop value made every pixel of a completely empty frame count
# as subject -- an all-backdrop render scored subject_fraction=1.000000. The same trap is
# documented in valley_bench.pixel_figures; it was read and walked into anyway.
BACKDROP_RGB = (100, 104, 112)
KEY_ENERGY = 6.0
FILL_ENERGY = 1.6
RENDER_WIDTH = 960
RENDER_HEIGHT = 960

# Floors, not equalities -- the same reasoning as the valley bench: a later look change should
# move these, and a floor separates "rendered something" from "rendered nothing" without pinning
# a number no one intends to hold.
# MEASURED on the DELIVERABLE `export/SM_VoxelPine_Tree02.glb` (2026-08-31, Blender 5.2.1):
# fraction 0.127873, colours 11,288, luma 112.625. Named with its asset on purpose -- the figures
# this comment carried before review (0.207 / 6,432 / 96.4) matched no committed asset: they were
# nearest the SUPERSEDED hand export `tree.glb` (0.135638 / 7,002 / 99.687), left unupdated when
# the generator replaced it. A calibration figure is worthless without the artifact it was taken on.
MIN_SUBJECT_FRACTION = 0.02
MAX_SUBJECT_FRACTION = 0.90
MIN_DISTINCT_COLORS = 32
MIN_SUBJECT_LUMA = 20.0


def subject_figures(pixels):
    """Figures over the pixels that are NOT the backdrop.

    Same shape as the valley bench's: the readback is display-referred sRGB, so the backdrop is
    compared in that space. Luma is averaged over subject pixels only -- averaging the whole
    frame would let a bigger backdrop hide a darker asset.
    """
    backdrop = tuple(component / 255.0 for component in BACKDROP_RGB)
    subject = 0
    colors = set()
    total = 0
    luma_sum = 0.0
    for red, green, blue, _ in zip(*[iter(pixels)] * 4):
        total += 1
        if max(abs(red - backdrop[0]), abs(green - backdrop[1]), abs(blue - backdrop[2])) > 0.02:
            subject += 1
            luma_sum += 255.0 * (0.2126 * red + 0.7152 * green + 0.0722 * blue)
        colors.add((round(red * 255), round(green * 255), round(blue * 255)))
    return {
        "subject_fraction": subject / total if total else 0.0,
        "distinct_colors": len(colors),
        "subject_luma": luma_sum / subject if subject else 0.0,
    }


def assert_range(check):
    assert check["tris"] > 0, "no geometry imported"
    assert check["subject_fraction"] >= MIN_SUBJECT_FRACTION, "frame is almost entirely backdrop"
    # TWO-SIDED ON PURPOSE. The floor alone cannot tell "the asset fills the frame" from "the
    # backdrop comparison is broken": with the backdrop held in the wrong colour space, an EMPTY
    # frame scored 1.000000 and only the colour floor caught it. A framed asset always leaves
    # backdrop visible, so an all-subject frame means the instrument is lying, not that the render
    # is full.
    assert check["subject_fraction"] <= MAX_SUBJECT_FRACTION, (
        "frame has no backdrop at all -- the backdrop comparison is broken, not the render")
    assert check["distinct_colors"] >= MIN_DISTINCT_COLORS, "frame has too few colours"
    assert check["subject_luma"] >= MIN_SUBJECT_LUMA, "the asset renders too dark"


def setup_scene(asset):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=asset)
    meshes = [o for o in bpy.data.objects if o.type == "MESH"]
    if not meshes:
        raise SystemExit("bench failed: the GLB imported no mesh")

    tris = 0
    for obj in meshes:
        obj.data.calc_loop_triangles()
        tris += len(obj.data.loop_triangles)

    # Frame the asset from its own bounds, so a taller variant is not cropped and a shorter one
    # is not a speck. Nothing here is hardcoded to one tree.
    lo = [min(min((obj.matrix_world @ v.co)[i] for v in obj.data.vertices) for obj in meshes)
          for i in range(3)]
    hi = [max(max((obj.matrix_world @ v.co)[i] for v in obj.data.vertices) for obj in meshes)
          for i in range(3)]
    centre = [(lo[i] + hi[i]) / 2.0 for i in range(3)]
    span = max(hi[i] - lo[i] for i in range(3))

    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 48
    # RuntimeError: Error: Failed to denoise, build has no OpenImageDenoise support.
    scene.cycles.use_denoising = False
    scene.render.resolution_x = RENDER_WIDTH
    scene.render.resolution_y = RENDER_HEIGHT
    scene.render.image_settings.file_format = "PNG"

    world = bpy.data.worlds.new("W")
    world.use_nodes = True
    linear = tuple(
        c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
        for c in (component / 255.0 for component in BACKDROP_RGB)
    )
    world.node_tree.nodes["Background"].inputs[0].default_value = linear + (1.0,)
    scene.world = world

    cam_data = bpy.data.cameras.new("Cam")
    cam = bpy.data.objects.new("Cam", cam_data)
    scene.collection.objects.link(cam)
    # Fixed three-quarter view; distance scales with the asset so every variant frames the same.
    angle = math.radians(35.0)
    dist = span * 2.1
    cam.location = (centre[0] + dist * math.cos(angle), centre[1] - dist * math.sin(angle),
                    centre[2] + span * 0.55)
    # Aim with a TRACK_TO constraint rather than hand-rolled euler angles: the hand-rolled
    # version pointed the camera away from the asset and rendered a frame of pure backdrop.
    target = bpy.data.objects.new("Target", None)
    target.location = centre
    scene.collection.objects.link(target)
    track = cam.constraints.new(type="TRACK_TO")
    track.target = target
    track.track_axis = "TRACK_NEGATIVE_Z"
    track.up_axis = "UP_Y"
    scene.camera = cam

    for name, energy, offset in (("Key", KEY_ENERGY, (1.0, -1.0, 1.4)),
                                 ("Fill", FILL_ENERGY, (-1.2, -0.6, 0.5))):
        light = bpy.data.lights.new(name, type="SUN")
        light.energy = energy
        obj = bpy.data.objects.new(name, light)
        obj.rotation_euler = (math.radians(55.0), math.radians(offset[1] * 12.0),
                              math.radians(offset[0] * 35.0))
        scene.collection.objects.link(obj)
    return tris


def render(asset, out):
    tris = setup_scene(asset)
    bpy.context.scene.render.filepath = out
    bpy.ops.render.render(write_still=True)
    image = bpy.data.images.load(out, check_existing=False)
    figures = subject_figures(image.pixels[:])
    bpy.data.images.remove(image)
    check = dict(tris=tris, **figures)
    print(
        "range-check:"
        f" blender={'.'.join(str(part) for part in bpy.app.version)}"
        f" tris={check['tris']}"
        f" subject_fraction={check['subject_fraction']:.6f}"
        f" distinct_colors={check['distinct_colors']}"
        f" subject_luma={check['subject_luma']:.3f}"
        f" floors(subject_fraction={MIN_SUBJECT_FRACTION:.6f},"
        f" distinct_colors={MIN_DISTINCT_COLORS},"
        f" subject_luma={MIN_SUBJECT_LUMA:.3f})"
    )
    try:
        assert_range(check)
    except AssertionError as error:
        # Blender logs an uncaught Python AssertionError yet exits 0; propagate failure to shell.
        raise SystemExit(f"range check failed: {error}") from error


def main():
    if bpy is None:
        raise SystemExit("run under Blender: blender --background --python spike_pine_render.py -- ASSET OUT")
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if len(args) != 2:
        raise SystemExit("usage: blender --background --python spike_pine_render.py -- ASSET.glb OUT.png")
    try:
        render(args[0], args[1])
    except SystemExit:
        raise
    except Exception:
        # Same guard as the valley bench: Blender exits 0 on an uncaught exception, so every
        # failure -- a missing file, a bad GLB, an import error -- must be turned into non-zero.
        traceback.print_exc()
        raise SystemExit("bench failed: see traceback above")


main()
