import json
import shutil
import subprocess
import sys
import tempfile
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


class ValleyGeometryTests(unittest.TestCase):
    def test_exposed_faces_use_six_orthogonal_neighbours_and_world_edges(self):
        adjacent = snapshot((2, 1, 1), [{"solid": "stone"}, {"ramp": "snow"}])
        self.assertEqual(valley_bench.geometry_summary(adjacent), {"exposed_cells": 2, "faces": 10})

        enclosed = snapshot((3, 3, 3), [{"solid": "stone"}] * 27)
        self.assertEqual(valley_bench.geometry_summary(enclosed), {"exposed_cells": 26, "faces": 54})

    def test_geometry_summary_changes_when_world_content_changes(self):
        empty = snapshot((2, 1, 1), ["empty", "empty"])
        one_cell = snapshot((2, 1, 1), [{"solid": "stone"}, "empty"])
        self.assertNotEqual(valley_bench.geometry_summary(empty), valley_bench.geometry_summary(one_cell))


class ValleyRangeTests(unittest.TestCase):
    def test_floor_functions_reject_empty_geometry_and_accept_visible_frame(self):
        with self.assertRaisesRegex(AssertionError, "no exposed cells"):
            valley_bench.assert_range(
                valley_bench.range_check({"exposed_cells": 0}, {"non_sky_fraction": 1.0, "distinct_colors": 7})
            )
        valley_bench.assert_range(
            valley_bench.range_check({"exposed_cells": 1}, {"non_sky_fraction": 0.5, "distinct_colors": 40})
        )

    def test_pixel_figures_change_between_empty_and_one_cell_frames(self):
        sky = list(valley_bench.srgb_to_linear(valley_bench.SKY_RGB)) + [1.0]
        empty = valley_bench.pixel_figures(sky * 2)
        one_cell = valley_bench.pixel_figures(sky + [1.0, 1.0, 1.0, 1.0])
        self.assertNotEqual(empty, one_cell)
        self.assertEqual(empty["non_sky_fraction"], 0.0)
        self.assertGreater(one_cell["non_sky_fraction"], 0.0)


@unittest.skipUnless(shutil.which("blender"), "Blender is required for bench subprocess tests")
class ValleyBlenderTests(unittest.TestCase):
    def run_bench(self, export):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            source = directory / "snapshot.json"
            output = directory / "frame.png"
            source.write_text(json.dumps(export), encoding="utf-8")
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
        self.assertIn("range-check: exposed_cells=0", result.stdout)


if __name__ == "__main__":
    unittest.main()
