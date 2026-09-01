"""Black-box checks for the asset-contract instrument."""

import pathlib
import subprocess
import sys
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
CHECKER = ROOT / "scripts/bench/check_asset.py"
SIGNOFF = ROOT / "_bmad-output/implementation-artifacts/10-2-signoff"


def check(*paths):
    return subprocess.run(
        [sys.executable, str(CHECKER), *(str(path) for path in paths)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


class CheckAssetTests(unittest.TestCase):
    def test_the_four_published_pines_report_their_literal_figures(self):
        paths = [SIGNOFF / "export" / f"SM_VoxelPine_Tree0{number}.glb" for number in range(1, 5)]
        result = check(*paths)

        self.assertEqual(result.returncode, 0, result.stderr)
        figures = [line for line in result.stdout.splitlines() if line.startswith("FIGURES ")]
        self.assertEqual(len(figures), 4, result.stdout)
        self.assertIn(
            "size=5.0x6.4x5.0 min_y=0.000000 centre_x=0.000000 "
            "centre_z=0.000000 tris=4366 verts=8732",
            figures[0],
        )
        self.assertIn(
            "size=5.0x8.0x5.4 min_y=0.000000 centre_x=0.000000 "
            "centre_z=0.000000 tris=5894 verts=11788",
            figures[1],
        )
        self.assertIn(
            "size=3.8x8.0x3.4 min_y=0.000000 centre_x=0.000000 "
            "centre_z=0.000000 tris=3474 verts=6948",
            figures[2],
        )
        self.assertIn(
            "size=4.6x10.6x4.6 min_y=0.000000 centre_x=0.000000 "
            "centre_z=0.000000 tris=5280 verts=10560",
            figures[3],
        )

    def test_off_centre_stale_asset_names_the_origin_clause(self):
        result = check(SIGNOFF / "tree.glb")

        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("origin-centring", result.stderr)
        self.assertIn("-0.100000", result.stderr)


if __name__ == "__main__":
    unittest.main()
