import collections
import json
import shutil
import subprocess
import sys
import tempfile
import math
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts" / "bench"))
import valley_bench


def snapshot(dims, tiles):
    return {
        "type": "snapshot",
        "dims": {"x": dims[0], "y": dims[1], "z": dims[2]},
        "tiles": tiles,
        "entities": [],
        "designations": [],
        "zones": [],
        "items": [],
        "speed": "normal",
        "tick": 0,
    }


# MEASURED on the populated test world: 124.3 with the ambient wired, 0.456 with
# AMBIENT_STRENGTH zeroed -- the raking key light alone barely touches a flat-topped block. A
# floor here separates those two by a wide margin without pinning either.
LIT_TERRAIN_LUMA = 40.0


def populated_world():
    """A world with terrain around the boot camera's focus, small enough for the pre-commit hook.

    The boot camera is fixed on (64, 64, 9), so a "populated" export has to put its cells THERE.
    A single cell at the origin is off-frame: measured, a one-cell export and an empty one render
    pixel-identical (0 of 2,073,600 values differ), which is why AC7's pixel half now compares a
    populated world against an empty one instead.
    """
    dims = (80, 80, 12)
    tiles = ["empty"] * (dims[0] * dims[1] * dims[2])
    for z in range(10):
        for y in range(54, 74):
            for x in range(54, 74):
                tiles[x + y * dims[0] + z * dims[0] * dims[1]] = {"solid": "stone"}
    return snapshot(dims, tiles)


class ValleyGeometryTests(unittest.TestCase):
    def test_exposed_faces_use_six_orthogonal_neighbours_and_world_edges(self):
        adjacent = snapshot((2, 1, 1), [{"solid": "stone"}, {"ramp": "snow"}])
        self.assertEqual(valley_bench.geometry_summary(adjacent), {"exposed_cells": 2, "faces": 10})

        enclosed = snapshot((3, 3, 3), [{"solid": "stone"}] * 27)
        self.assertEqual(valley_bench.geometry_summary(enclosed), {"exposed_cells": 26, "faces": 54})

    def test_foliage_scale_matches_the_client_crown_rule(self):
        # Independent oracle: the four values and the "solid foliage only, at most two, stop at
        # the first gap" walk are read off project.rs:874-892, not off the bench.
        column = snapshot(
            (1, 1, 5),
            [
                {"solid": "tree_trunk"},
                {"solid": "tree_foliage"},
                {"solid": "tree_foliage"},
                {"solid": "tree_foliage"},
                "empty",
            ],
        )
        self.assertEqual(valley_bench.foliage_scale(column, 0, 0, 0), 1.0)
        self.assertEqual(valley_bench.foliage_scale(column, 0, 0, 1), 0.95)
        self.assertEqual(valley_bench.foliage_scale(column, 0, 0, 2), 0.78)
        self.assertEqual(valley_bench.foliage_scale(column, 0, 0, 3), 0.62)

        ramp_above = snapshot((1, 1, 2), [{"solid": "tree_foliage"}, {"ramp": "tree_foliage"}])
        self.assertEqual(valley_bench.foliage_scale(ramp_above, 0, 0, 0), 0.62)

    def test_foliage_is_drawn_smaller_than_its_cell(self):
        # The client shrinks foliage about its cell centre; a full-size cube canopy is the
        # difference an eye reads as "the trees changed" between bench and build.
        indexes = collections.defaultdict(int)
        foliage = snapshot((1, 1, 1), [{"solid": "tree_foliage"}])
        stone = snapshot((1, 1, 1), [{"solid": "stone"}])
        spans = []
        for world in (foliage, stone):
            vertices, _, _ = valley_bench.mesh_geometry(world, indexes)
            xs = [vertex[0] for vertex in vertices]
            spans.append(max(xs) - min(xs))
        self.assertAlmostEqual(spans[0], 0.62, places=6)
        self.assertAlmostEqual(spans[1], 1.0, places=6)

    def test_foliage_scale_does_not_change_which_faces_are_exposed(self):
        world = snapshot((1, 1, 2), [{"solid": "tree_trunk"}, {"solid": "tree_foliage"}])
        self.assertEqual(valley_bench.geometry_summary(world), {"exposed_cells": 2, "faces": 10})

    def test_geometry_summary_changes_when_world_content_changes(self):
        empty = snapshot((2, 1, 1), ["empty", "empty"])
        one_cell = snapshot((2, 1, 1), [{"solid": "stone"}, "empty"])
        self.assertNotEqual(valley_bench.geometry_summary(empty), valley_bench.geometry_summary(one_cell))


class ValleyRangeTests(unittest.TestCase):
    def test_floor_functions_reject_empty_geometry_and_accept_visible_frame(self):
        lit = {"non_sky_fraction": 0.5, "distinct_colors": 40, "terrain_luma": 80.0}
        with self.assertRaisesRegex(AssertionError, "no exposed cells"):
            valley_bench.assert_range(
                valley_bench.range_check(
                    {"exposed_cells": 0},
                    {"non_sky_fraction": 1.0, "distinct_colors": 7, "terrain_luma": 80.0},
                )
            )
        with self.assertRaisesRegex(AssertionError, "lit surfaces are too dark"):
            valley_bench.assert_range(
                valley_bench.range_check({"exposed_cells": 1}, {**lit, "terrain_luma": 1.0})
            )
        valley_bench.assert_range(valley_bench.range_check({"exposed_cells": 1}, lit))

    def test_terrain_luma_averages_lit_pixels_only(self):
        # A whole-frame average would let a bigger sky mask a darker scene, which is exactly the
        # confound this figure exists to remove.
        sky = [0.01961, 0.04706, 0.1098, 1.0]
        white = [1.0, 1.0, 1.0, 1.0]
        self.assertAlmostEqual(valley_bench.pixel_figures(sky + white)["terrain_luma"], 255.0, places=3)
        self.assertAlmostEqual(
            valley_bench.pixel_figures(sky * 3 + white)["terrain_luma"], 255.0, places=3
        )
        self.assertEqual(valley_bench.pixel_figures(sky * 2)["terrain_luma"], 0.0)

    def test_pixel_figures_change_between_empty_and_one_cell_frames(self):
        # The sky reference is the MEASURED display-referred value a real render reads back, not
        # a value derived through the same conversion pixel_figures uses. Deriving it made this
        # test agree with the code in whichever colour space the code happened to pick, and it
        # stayed green while a 100%-sky frame scored non_sky_fraction=1.0.
        sky = [0.01961, 0.04706, 0.1098, 1.0]
        empty = valley_bench.pixel_figures(sky * 2)
        one_cell = valley_bench.pixel_figures(sky + [1.0, 1.0, 1.0, 1.0])
        self.assertNotEqual(empty, one_cell)
        self.assertEqual(empty["non_sky_fraction"], 0.0)
        self.assertGreater(one_cell["non_sky_fraction"], 0.0)


class ValleyFramingTests(unittest.TestCase):
    def test_boot_projection_matches_the_client_composition(self):
        tolerance = 0.03
        camp = valley_bench.project_boot_point((64.0, 9.0, -64.0))
        skyline = valley_bench.project_boot_point((64.0, 26.0, -128.0))

        self.assertIsNotNone(camp)
        self.assertIsNotNone(skyline)
        self.assertAlmostEqual(camp[0], 0.48, delta=tolerance)
        self.assertAlmostEqual(camp[1], 0.78, delta=tolerance)
        self.assertAlmostEqual(skyline[1], 0.24, delta=tolerance)

    def test_projection_reads_the_shared_fov_and_aspect_constants(self):
        # The renderer and the framing test must not hold separate copies of the projection.
        # Measured on the split version: widening ONLY the render FOV to pi/3 moved 1,050,234 of
        # 2,073,600 pixels while this test still read camp=(0.500, 0.779), inside tolerance.
        camp = valley_bench.project_boot_point((64.0, 9.0, -64.0))
        for name, value in (("BOOT_VERTICAL_FOV", math.pi / 3), ("BOOT_ASPECT_RATIO", 4.0 / 3.0)):
            original = getattr(valley_bench, name)
            try:
                setattr(valley_bench, name, value)
                self.assertNotEqual(
                    valley_bench.project_boot_point((64.0, 9.0, -64.0)),
                    camp,
                    f"projection ignores {name}",
                )
            finally:
                setattr(valley_bench, name, original)

    def test_render_resolution_is_derived_from_the_boot_aspect_ratio(self):
        self.assertEqual(valley_bench.RENDER_WIDTH, 960)
        self.assertEqual(valley_bench.RENDER_HEIGHT, 540)
        self.assertAlmostEqual(
            valley_bench.RENDER_WIDTH / valley_bench.RENDER_HEIGHT,
            valley_bench.BOOT_ASPECT_RATIO,
            places=6,
        )

    def test_sun_is_aimed_the_way_the_client_aims_it(self):
        # Independent oracle: computed from atmosphere.rs's own constants, not from the bench.
        # The previous hand-picked euler pointed 122 degrees away from this.
        core = valley_bench.aurora_core()
        self.assertAlmostEqual(core[1], (-162.0 + 45.0) * 0.5, places=6)
        direction = valley_bench.sun_direction()
        # AIM IS ASSERTED FIRST, DELIBERATELY. The aurora core hangs BELOW the horizon
        # ((-162 + 45) / 2 = -58.5) while the camp sits at y = 9, so the client's key light rakes
        # gently UPWARD across the valley rather than shining down on it; the replaced hand-picked
        # euler came down from 39.6 degrees above. When the normalisation check ran first it
        # ABSORBED the sabotage -- a hand-written vector is never exactly unit length, so the row
        # died on "1.000605 != 1.0" and the aim assertion below never executed. The test looked
        # identical from outside while pinning nothing about direction.
        self.assertGreater(direction[1], 0.0, f"key light points downward: {direction}")
        self.assertLess(direction[1], 0.2, f"key light is too steep: {direction}")
        self.assertGreater(direction[0], 0.5, f"key light comes from the wrong compass point: {direction}")
        self.assertAlmostEqual(sum(component * component for component in direction), 1.0, places=6)


@unittest.skipUnless(shutil.which("blender"), "Blender is required for bench subprocess tests")
class ValleyBlenderTests(unittest.TestCase):
    def run_bench(self, export, raw=None):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            source = directory / "snapshot.json"
            output = directory / "frame.png"
            source.write_text(raw if raw is not None else json.dumps(export), encoding="utf-8")
            return subprocess.run(
                [
                    "blender",
                    "--background",
                    "--python",
                    str(REPO_ROOT / "scripts" / "bench" / "valley_bench.py"),
                    "--",
                    str(source),
                    str(output),
                ],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                check=False,
            )

    def test_empty_export_exits_nonzero(self):
        result = self.run_bench(snapshot((1, 1, 1), ["empty"]))
        self.assertNotEqual(result.returncode, 0, result.stdout)
        line = next(l for l in result.stdout.splitlines() if l.startswith("range-check:"))
        reported = line[len("range-check:") :].split("floors(")[0]
        self.assertEqual(dict(f.split("=") for f in reported.split())["exposed_cells"], "0")

    def test_pixel_figures_differ_between_a_populated_and_an_empty_real_render(self):
        # AC7's pixel half, on the REAL renderer. The AC originally asked for a one-cell world
        # against an empty one; measured, those two render pixel-identical because the single
        # cell falls outside the fixed boot frame, so the comparison proved nothing. A populated
        # world is the smallest thing that actually exercises the pixel half end to end.
        populated = self.run_bench(populated_world())
        self.assertEqual(populated.returncode, 0, populated.stdout)
        empty = self.run_bench(snapshot((1, 1, 1), ["empty"]))
        self.assertNotEqual(empty.returncode, 0, empty.stdout)

        figures = {}
        for name, result in (("populated", populated), ("empty", empty)):
            line = next(l for l in result.stdout.splitlines() if l.startswith("range-check:"))
            reported = line[len("range-check:") :].split("floors(")[0]
            figures[name] = dict(field.split("=") for field in reported.split())
        self.assertNotEqual(figures["populated"], figures["empty"])
        self.assertGreater(float(figures["populated"]["non_sky_fraction"]), 0.0)
        self.assertEqual(float(figures["empty"]["non_sky_fraction"]), 0.0)
        self.assertGreater(float(figures["populated"]["terrain_luma"]), float(figures["empty"]["terrain_luma"]))

    def test_the_range_check_names_the_blender_that_produced_it(self):
        # Two Blenders live on this machine since 2026-08-31 and their figures are NOT
        # comparable: the same snapshot renders non_sky_fraction 0.686815 under 4.3.2 and
        # 0.686736 under 5.2.1. Each is bit-deterministic with itself, so a drifting figure is
        # supposed to mean a real change -- which is only true while the line says which venue
        # produced it. A PATH change would otherwise re-baseline the bench in silence.
        # The oracle is INDEPENDENT: `blender --version` is a different code path in the same
        # binary, so this cannot pass by the bench agreeing with itself.
        banner = subprocess.run(
            ["blender", "--version"], text=True, stdout=subprocess.PIPE, check=True
        ).stdout.split()
        expected = banner[banner.index("Blender") + 1]

        result = self.run_bench(populated_world())
        self.assertEqual(result.returncode, 0, result.stdout)
        line = next(l for l in result.stdout.splitlines() if l.startswith("range-check:"))
        reported = line[len("range-check:") :].split("floors(")[0]
        fields = dict(field.split("=") for field in reported.split())
        self.assertEqual(fields["blender"], expected, line)

    def test_a_broken_export_exits_nonzero_instead_of_reporting_success(self):
        # Blender's --background runner prints a traceback for an uncaught exception and STILL
        # exits 0. Guarding only the range assert left malformed JSON, an unknown entity kind and
        # a dims/tiles mismatch each printing a traceback and reporting success on a frame that
        # was never rendered. Each of these three reproduced exit 0 before the fix.
        broken = {
            "malformed json": dict(export=None, raw="{not json at all"),
            "unknown entity kind": dict(
                export={
                    **snapshot((1, 1, 1), [{"solid": "stone"}]),
                    "entities": [{"id": 1, "kind": "elf_wizard", "pos": [0, 0, 0]}],
                }
            ),
            "dims larger than tiles": dict(export=snapshot((2, 2, 2), [{"solid": "stone"}])),
        }
        for label, kwargs in broken.items():
            with self.subTest(label):
                result = self.run_bench(**kwargs)
                self.assertNotEqual(result.returncode, 0, f"{label}: {result.stdout}")
                self.assertIn("bench failed:", result.stdout)

    def test_a_populated_render_is_lit_not_merely_non_black(self):
        # AMBIENT_RGB was defined, pinned by the drift guard, and read by NOTHING: the frame came
        # out ~24% darker than the client while every existing floor stayed green, because a dark
        # frame is neither empty nor monochrome. This drives the real renderer and reads the
        # figure that would have caught it.
        result = self.run_bench(populated_world())
        self.assertEqual(result.returncode, 0, result.stdout)
        line = next(l for l in result.stdout.splitlines() if l.startswith("range-check:"))
        reported = line[len("range-check:") :].split("floors(")[0]
        luma = float(dict(f.split("=") for f in reported.split())["terrain_luma"])
        self.assertGreater(luma, LIT_TERRAIN_LUMA, f"lit terrain reads {luma}: {line}")

    def test_all_sky_frame_reads_as_sky_in_a_real_render(self):
        # Drives the REAL renderer, because the pixel half of the range check is only worth what
        # it reports on actual render output. An all-sky frame must score 0.0; when the sky
        # reference sat in the wrong colour space this printed 1.000000 and the floor was inert.
        result = self.run_bench(snapshot((1, 1, 1), ["empty"]))
        self.assertIn("non_sky_fraction=0.000000", result.stdout)


if __name__ == "__main__":
    unittest.main()
