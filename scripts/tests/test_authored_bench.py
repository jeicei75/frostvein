"""Gate tests for the shipped mesh bench.

Blender is not available to the gate, so these exercise the PURE halves — the column derivation,
the tree strip, and the variant mapping. That mapping is the half that can silently diverge from
the client: `authored_bench` and `crates/gui/src/project.rs` must agree on which pine a column of
a given height gets, or the bench measures a valley the client does not draw.
"""

import json
import subprocess
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts" / "bench"))
import authored_bench  # noqa: E402


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


def trunk_world(columns, dims):
    """A snapshot holding one trunk cell at every (x, y, z) in `columns`."""
    dx, dy, dz = dims
    tiles = [{} for _ in range(dx * dy * dz)]
    for x, y, z in columns:
        tiles[x + y * dx + z * dx * dy] = {"solid": "tree_trunk"}
    return snapshot(dims, tiles)


class AuthoredBenchTests(unittest.TestCase):
    def test_every_shipped_pine_is_a_whole_number_of_cells(self):
        """The scale claim the whole comparison rests on. A pine that is not an exact cell
        count either overshoots its column or floats above it."""
        for name, metres in authored_bench.VARIANTS.items():
            cells = metres / authored_bench.METRES_PER_CELL
            self.assertAlmostEqual(
                cells, round(cells), places=6, msg=f"{name} is {cells} cells, not a whole number"
            )

    def test_the_bench_reads_the_same_glbs_the_client_embeds(self):
        """A bench pointing at its own copy of an asset is how a bench and a client drift."""
        for name in authored_bench.VARIANTS:
            path = Path(authored_bench.VARIANT_DIR[name]) / f"{name}.glb"
            self.assertTrue(path.is_file(), f"{path} must exist")
            self.assertEqual(
                path.parent,
                REPO_ROOT / "assets" / "trees",
                "the bench must read the shipped assets, not a signoff copy",
            )
            with open(path, "rb") as handle:
                self.assertEqual(handle.read(4), b"glTF", f"{name} must be binary glTF")

    def test_trunk_columns_derive_base_and_height_from_the_trunk_cells(self):
        # One column of three trunk cells at z 2,3,4: base 2, height = max - min + 2 = 4.
        world = trunk_world([(1, 1, 2), (1, 1, 3), (1, 1, 4)], (3, 3, 8))
        self.assertEqual(sorted(authored_bench.trunk_columns(world)), [(1, 1, 2, 4)])

    def test_strip_trees_removes_every_tree_cell(self):
        world = trunk_world([(0, 0, 0), (0, 0, 1)], (2, 1, 3))
        world["tiles"][1] = {"solid": "stone"}
        stripped = authored_bench.strip_trees(world)
        self.assertEqual(
            [tile for tile in stripped["tiles"] if tile.get("solid") == "tree_trunk"],
            [],
            "no trunk cell may survive the strip",
        )
        self.assertIn({"solid": "stone"}, stripped["tiles"], "terrain must survive the strip")

    def test_the_height_to_variant_mapping_matches_the_client(self):
        """The client's rule, read from `project.rs` rather than restated here — the two are one
        mapping in two languages, and a copy in this test would agree with itself forever."""
        source = (REPO_ROOT / "crates" / "gui" / "src" / "project.rs").read_text()
        for height, variant in ((4, "Tree01"), (6, "Tree04R")):
            self.assertIn(
                f"{height} => TreeVariant::{variant}",
                source,
                f"the client must map height {height} to {variant}",
            )
        self.assertIn("5 => TreeVariant::Tree03", source)
        bench_names = {
            4: ["SM_VoxelPine_Tree01"],
            5: ["SM_VoxelPine_Tree02", "SM_VoxelPine_Tree03"],
            6: ["SM_VoxelPine_Tree04R"],
        }
        for height, names in bench_names.items():
            for name in names:
                self.assertIn(
                    name.replace("SM_VoxelPine_", ""),
                    source,
                    f"the client must know {name} for height {height}",
                )

    def test_stable_hash_is_stable_across_processes(self):
        """`hash()` is salted per process, so a bench using it would place different trees on
        every run and no two artifacts would be comparable."""
        script = (
            "import sys, json; sys.path.insert(0, sys.argv[1]); import authored_bench; "
            'print(json.dumps(authored_bench.stable_hash(3, 7, "variant")))'
        )
        out = subprocess.run(
            [sys.executable, "-c", script, str(REPO_ROOT / "scripts" / "bench")],
            capture_output=True,
            text=True,
            check=True,
        )
        self.assertEqual(
            json.loads(out.stdout), authored_bench.stable_hash(3, 7, "variant")
        )

if __name__ == "__main__":
    unittest.main()
