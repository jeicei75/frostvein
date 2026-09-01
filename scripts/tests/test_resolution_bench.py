import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts" / "bench"))
import resolution_bench


def snapshot(dims, tiles):
    return {"dims": {"x": dims[0], "y": dims[1], "z": dims[2]}, "tiles": tiles}


def staircase():
    """A 4x4x4 world of stepped columns: cliffs, buried cells, and coplanar tops."""
    tiles = []
    for z in range(4):
        for _ in range(4):
            for x in range(4):
                tiles.append({"solid": "stone"} if z <= x else "empty")
    return snapshot((4, 4, 4), tiles)


def brute_force_faces(world, k, detail):
    """Independent oracle: materialise every fine voxel and count faces one at a time.

    This shares no code with `geometry_summary`, which reasons about columns and cell
    boundaries analytically.  It is deliberately the slowest possible implementation --
    correctness here is the only thing being bought.  The defect this exists to catch shipped
    because the k>1 assertion was `greater than 6`, which no count can fail in either
    direction.
    """
    dims = world["dims"]
    dx, dy, dz = dims["x"], dims["y"], dims["z"]
    materials = [resolution_bench._material(tile) for tile in world["tiles"]]

    def at(x, y, z):
        if not (0 <= x < dx and 0 <= y < dy and 0 <= z < dz):
            return None
        return materials[x + y * dx + z * dx * dy]

    solid = set()
    for z in range(dz):
        for y in range(dy):
            for x in range(dx):
                if at(x, y, z) is None:
                    continue
                carved = detail and k > 1 and at(x, y, z + 1) is None
                for i in range(k):
                    for j in range(k):
                        height = k
                        if carved:
                            height -= resolution_bench.detail_depth(
                                resolution_bench.WORLD_SEED,
                                (z + 1) * k,
                                x * k + i,
                                y * k + j,
                                k,
                            )
                        for level in range(height):
                            solid.add((x * k + i, y * k + j, z * k + level))
    faces = 0
    for fx, fy, fz in solid:
        for axis, sign in resolution_bench.NEIGHBOURS:
            neighbour = [fx, fy, fz]
            neighbour[axis] += sign
            if tuple(neighbour) not in solid:
                faces += 1
    return faces


class ResolutionDetailRuleTests(unittest.TestCase):
    # The SAME vector is pinned in `crates/gui/src/project.rs`. Two literal tables in two
    # languages are the oracle for "both sides run one rule"; the comment that used to claim it
    # was tested by nothing, and the two rules had in fact diverged.
    VECTOR = ((0, 0, 0), (1, 2, 3), (8, 5, 1), (9, 9, 9), (64, 17, 5))

    def test_detail_rule_is_seeded_and_has_a_hand_written_two_voxel_range(self):
        self.assertEqual(
            [resolution_bench.detail_offset(resolution_bench.WORLD_SEED, *point) for point in self.VECTOR],
            [-1, 0, -1, -1, -2],
        )
        offsets = [resolution_bench.detail_offset(resolution_bench.WORLD_SEED, x, 3, 7) for x in range(32)]
        self.assertGreater(len(set(offsets)), 1)
        self.assertTrue(all(-2 <= offset <= 2 for offset in offsets))

    def test_detail_rule_stays_inside_32_bits(self):
        # The divergence that shipped: Python integers are unbounded, so an unmasked multiply
        # left the client's u32 rule and this one agreeing only at chance for k > 1.
        for point in ((0x7FFF_FFFF, 0x7FFF_FFFF, 0x7FFF_FFFF), (123_456_789, 987_654_321, 5)):
            self.assertIn(resolution_bench.detail_offset(resolution_bench.WORLD_SEED, *point), range(-2, 3))

    def test_depth_is_clamped_by_the_fine_cell_height(self):
        self.assertEqual(
            [resolution_bench.detail_depth(resolution_bench.WORLD_SEED, *point, 4) for point in self.VECTOR],
            [1, 0, 1, 1, 2],
        )
        # k=1 has no room for a pit, which is exactly why the k=1 control is blind to the rule.
        for point in self.VECTOR:
            self.assertEqual(resolution_bench.detail_depth(resolution_bench.WORLD_SEED, *point, 1), 0)


class ResolutionGeometryTests(unittest.TestCase):
    def test_greedy_mesher_merges_a_two_cell_prism_with_hand_written_counts(self):
        world = snapshot((2, 1, 1), [{"solid": "stone"}, {"solid": "stone"}])
        self.assertEqual(
            resolution_bench.geometry_summary(world, k=1, detail=True),
            {"exposed_faces": 10, "greedy_quads": 6, "triangles": 12, "chunks": 1},
        )

    def test_subdivided_flat_prism_keeps_the_same_six_quads_without_detail(self):
        world = snapshot((2, 1, 1), [{"solid": "stone"}, {"solid": "stone"}])
        self.assertEqual(
            resolution_bench.geometry_summary(world, k=16, detail=False),
            {"exposed_faces": 2560, "greedy_quads": 6, "triangles": 12, "chunks": 1},
        )

    def test_detail_rule_changes_subdivided_counts_exactly_and_leaves_k_one_alone(self):
        world = snapshot((2, 1, 1), [{"solid": "stone"}, {"solid": "stone"}])
        # Exact counts in BOTH directions. Detail removes carved side faces as well as adding
        # connectors, so k=2 detailed is FEWER faces than k=2 flat (32 against 40) -- the
        # reduction the analytic `coarse_faces * k * k` baseline could never express.
        self.assertEqual(
            resolution_bench.geometry_summary(world, k=2, detail=True),
            {"exposed_faces": 32, "greedy_quads": 12, "triangles": 24, "chunks": 1},
        )
        self.assertEqual(
            resolution_bench.geometry_summary(world, k=4, detail=True),
            {"exposed_faces": 176, "greedy_quads": 72, "triangles": 144, "chunks": 1},
        )
        self.assertEqual(
            resolution_bench.geometry_summary(world, k=1, detail=True)["greedy_quads"], 6
        )

    def test_a_stepped_world_matches_hand_written_counts(self):
        # Cliffs, coplanar tops across cells, and cells buried at the coarse scale.
        self.assertEqual(
            resolution_bench.geometry_summary(staircase(), k=1, detail=True),
            {"exposed_faces": 84, "greedy_quads": 18, "triangles": 36, "chunks": 1},
        )
        self.assertEqual(
            resolution_bench.geometry_summary(staircase(), k=2, detail=True),
            {"exposed_faces": 334, "greedy_quads": 77, "triangles": 154, "chunks": 1},
        )
        self.assertEqual(
            resolution_bench.geometry_summary(staircase(), k=4, detail=True),
            {"exposed_faces": 1608, "greedy_quads": 463, "triangles": 926, "chunks": 1},
        )

    def test_face_count_matches_a_brute_force_fine_voxel_oracle(self):
        worlds = {
            "single": snapshot((1, 1, 1), [{"solid": "stone"}]),
            "prism": snapshot((2, 1, 1), [{"solid": "stone"}, {"solid": "stone"}]),
            "staircase": staircase(),
            "two materials": snapshot(
                (2, 2, 1),
                [{"solid": "stone"}, {"solid": "dirt"}, "empty", {"ramp": "stone"}],
            ),
        }
        for name, world in worlds.items():
            for k in (1, 2, 3, 4):
                for detail in (True, False):
                    with self.subTest(world=name, k=k, detail=detail):
                        self.assertEqual(
                            resolution_bench.geometry_summary(world, k=k, detail=detail)["exposed_faces"],
                            brute_force_faces(world, k, detail),
                        )

    def test_chunks_are_counted_in_three_dimensions_from_emitted_geometry(self):
        # The old column was ceil(dx/16) * ceil(dy/16): 2-D, z-blind, empty-chunk-blind, and it
        # never touched the mesher. A 1x1x40 column spans three 16-cell chunks in z.
        world = snapshot((1, 1, 40), [{"solid": "stone"}] * 40)
        self.assertEqual(resolution_bench.geometry_summary(world, k=1)["chunks"], 3)
        hollow = snapshot((40, 1, 1), [{"solid": "stone"}] + ["empty"] * 39)
        self.assertEqual(resolution_bench.geometry_summary(hollow, k=1)["chunks"], 1)

    def test_control_check_requires_the_real_world_literals(self):
        resolution_bench.assert_control({"exposed_faces": 61142, "greedy_quads": 19264})
        with self.assertRaisesRegex(ValueError, "61142"):
            resolution_bench.assert_control({"exposed_faces": 61141, "greedy_quads": 19264})


class ResolutionRealWorldControlTests(unittest.TestCase):
    """AC4's oracle, with a caller.

    `assert_control` was reachable only from `main()`, so every gate-run test meshed a synthetic
    two-cell world and none meshed the real one. 61,142 / 19,264 are MEASUREMENTS of world
    content -- exactly the shape that went stale unnoticed in 9.4 -- and nothing went red when
    worldgen or the exposure rule moved. This is the caller.
    """

    def test_the_exported_world_still_meshes_to_the_recorded_control(self):
        simd = REPO_ROOT / "target" / "debug" / "simd"
        self.assertTrue(
            simd.exists(),
            f"{simd} is missing: build the workspace before the bench tests (scripts/gate.sh does)",
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "world.json"
            subprocess.run(
                [sys.executable, str(REPO_ROOT / "scripts" / "bench" / "export_world.py"), str(path)],
                cwd=REPO_ROOT,
                check=True,
                capture_output=True,
            )
            world = resolution_bench._load_snapshot(path)
        summary = resolution_bench.geometry_summary(world, k=1, detail=True)
        resolution_bench.assert_control(summary)
        self.assertEqual(summary["triangles"], 38_528)


class ResolutionSimCostTests(unittest.TestCase):
    def test_wire_snapshot_scaling_repeats_the_real_tile_encoding_not_an_average(self):
        raw = '{"type":"snapshot","tiles":["empty",{"solid":"stone"}],"tick":0}'
        self.assertEqual(
            resolution_bench.sim_axis_cost(raw, 2),
            {"sim_k": 2, "cells": 16, "tile_bytes": 217, "snapshot_bytes": 246},
        )


class ResolutionSafetyTests(unittest.TestCase):
    def test_workload_guard_rejects_a_hand_written_over_limit_count(self):
        resolution_bench.assert_workload_limit(1_000_000, 1, True)
        with self.assertRaisesRegex(ValueError, "48,000,002"):
            resolution_bench.assert_workload_limit(24_000_001, 1, True)
        # The flat control is genuinely meshed now rather than short-circuited to k=1's quads,
        # so it needs the same guard: it allocates one fine face per sample.
        resolution_bench.assert_workload_limit(48_000_000, 1, False)
        with self.assertRaisesRegex(ValueError, "48,000,001"):
            resolution_bench.assert_workload_limit(48_000_001, 1, False)

    def test_snapshot_size_guard_uses_the_explicit_wire_limit(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "snapshot.json"
            path.write_text('{"dims":{"x":1,"y":1,"z":1},"tiles":["empty"]}')
            self.assertEqual(resolution_bench._load_snapshot(path)["tiles"], ["empty"])
        with self.assertRaisesRegex(ValueError, "too large"):
            resolution_bench.assert_snapshot_size(resolution_bench.MAX_SNAPSHOT_BYTES + 1)


if __name__ == "__main__":
    unittest.main()
