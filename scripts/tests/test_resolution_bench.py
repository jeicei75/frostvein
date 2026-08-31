import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts" / "bench"))
import resolution_bench


def snapshot(dims, tiles):
    return {"dims": {"x": dims[0], "y": dims[1], "z": dims[2]}, "tiles": tiles}


class ResolutionDetailRuleTests(unittest.TestCase):
    def test_detail_rule_is_seeded_and_has_a_hand_written_two_voxel_range(self):
        # Independent expected values for the story's fixed seed.  These are deliberately
        # literals, not a second call through the detail-rule implementation.
        self.assertEqual(
            [resolution_bench.detail_offset(resolution_bench.WORLD_SEED, *point) for point in ((0, 0, 0), (1, 2, 3), (8, 5, 1), (9, 9, 9))],
            [-1, -1, 1, -1],
        )
        offsets = [resolution_bench.detail_offset(resolution_bench.WORLD_SEED, x, 3, 7) for x in range(32)]
        self.assertGreater(len(set(offsets)), 1)
        self.assertTrue(all(-2 <= offset <= 2 for offset in offsets))


class ResolutionGeometryTests(unittest.TestCase):
    def test_greedy_mesher_merges_a_two_cell_prism_with_hand_written_counts(self):
        world = snapshot((2, 1, 1), [{"solid": "stone"}, {"solid": "stone"}])
        self.assertEqual(
            resolution_bench.geometry_summary(world, k=1, detail=True),
            {"exposed_faces": 10, "greedy_quads": 6, "triangles": 12},
        )

    def test_subdivided_flat_prism_keeps_the_same_six_quads_without_detail(self):
        world = snapshot((2, 1, 1), [{"solid": "stone"}, {"solid": "stone"}])
        self.assertEqual(
            resolution_bench.geometry_summary(world, k=16, detail=False),
            {"exposed_faces": 2560, "greedy_quads": 6, "triangles": 12},
        )

    def test_detail_rule_increases_subdivided_quads_but_not_the_k_one_control(self):
        world = snapshot((2, 1, 1), [{"solid": "stone"}, {"solid": "stone"}])
        self.assertEqual(resolution_bench.geometry_summary(world, k=1, detail=True)["greedy_quads"], 6)
        self.assertGreater(resolution_bench.geometry_summary(world, k=4, detail=True)["greedy_quads"], 6)

    def test_control_check_requires_the_real_world_literals(self):
        resolution_bench.assert_control({"exposed_faces": 61142, "greedy_quads": 19264})
        with self.assertRaisesRegex(ValueError, "61142"):
            resolution_bench.assert_control({"exposed_faces": 61141, "greedy_quads": 19264})


if __name__ == "__main__":
    unittest.main()
