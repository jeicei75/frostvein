import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts" / "bench"))
import resolution_bench


class ResolutionDetailRuleTests(unittest.TestCase):
    def test_detail_rule_is_seeded_and_has_a_hand_written_two_voxel_range(self):
        # Independent expected values for the story's fixed seed.  These are deliberately
        # literals, not a second call through the detail-rule implementation.
        self.assertEqual(
            [resolution_bench.detail_offset(0xF05_7EED, *point) for point in ((0, 0, 0), (1, 2, 3), (8, 5, 1), (9, 9, 9))],
            [2, 2, -2, 2],
        )
        offsets = [resolution_bench.detail_offset(0xF05_7EED, x, 3, 7) for x in range(32)]
        self.assertGreater(len(set(offsets)), 1)
        self.assertTrue(all(-2 <= offset <= 2 for offset in offsets))


if __name__ == "__main__":
    unittest.main()
