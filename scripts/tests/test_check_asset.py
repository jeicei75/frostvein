"""Black-box checks for the asset-contract instrument."""

import json
import pathlib
import subprocess
import struct
import sys
import tempfile
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


def write_tree02_mutant(target, change):
    """Write a tiny, real GLB mutant without changing its source artifact."""
    data = (SIGNOFF / "export/SM_VoxelPine_Tree02.glb").read_bytes()
    offset, chunks = 12, []
    while offset < len(data):
        length, kind = struct.unpack_from("<II", data, offset)
        chunks.append((kind, data[offset + 8:offset + 8 + length]))
        offset += 8 + length
    document = json.loads(next(chunk for kind, chunk in chunks if kind == 0x4E4F534A))
    binary = bytearray(next(chunk for kind, chunk in chunks if kind == 0x004E4942))
    position = document["accessors"][document["meshes"][0]["primitives"][0]["attributes"]["POSITION"]]
    view = document["bufferViews"][position["bufferView"]]
    change(document, binary, view.get("byteOffset", 0) + position.get("byteOffset", 0))
    encoded = json.dumps(document, separators=(",", ":")).encode()
    encoded += b" " * (-len(encoded) % 4)
    binary += b"\0" * (-len(binary) % 4)
    target.write_bytes(
        struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(encoded) + 8 + len(binary))
        + struct.pack("<II", len(encoded), 0x4E4F534A) + encoded
        + struct.pack("<II", len(binary), 0x004E4942) + binary
    )


class CheckAssetTests(unittest.TestCase):
    def test_the_four_published_pines_report_their_literal_figures(self):
        paths = [SIGNOFF / "export" / f"SM_VoxelPine_Tree0{number}.glb" for number in range(1, 5)]
        result = check(*paths)

        self.assertEqual(result.returncode, 0, result.stderr)
        figures = [line for line in result.stdout.splitlines() if line.startswith("FIGURES ")]
        self.assertEqual(len(figures), 4, result.stdout)
        self.assertIn(
            "size_m=5.0x6.4x5.0 min_y_m=0.000000 centre_x_m=0.000000 "
            "centre_z_m=0.000000 palette=#4A3B2E,#6B5B49,#2A3E34,#364D3F,#52715B,#FFFFFF,#D8E4EC tris=4366 verts=8732",
            figures[0],
        )
        self.assertIn(
            "size_m=5.0x8.0x5.4 min_y_m=0.000000 centre_x_m=0.000000 "
            "centre_z_m=0.000000 palette=#4A3B2E,#6B5B49,#2A3E34,#364D3F,#52715B,#FFFFFF,#D8E4EC tris=5894 verts=11788",
            figures[1],
        )
        self.assertIn(
            "size_m=3.8x8.0x3.4 min_y_m=0.000000 centre_x_m=0.000000 "
            "centre_z_m=0.000000 palette=#4A3B2E,#6B5B49,#2A3E34,#364D3F,#52715B,#FFFFFF,#D8E4EC tris=3474 verts=6948",
            figures[2],
        )
        self.assertIn(
            "size_m=4.6x10.6x4.6 min_y_m=0.000000 centre_x_m=0.000000 "
            "centre_z_m=0.000000 palette=#4A3B2E,#6B5B49,#2A3E34,#364D3F,#52715B,#FFFFFF,#D8E4EC tris=5280 verts=10560",
            figures[3],
        )

    def test_the_authored_dwarf_reports_all_ten_cells_not_the_pines_seven(self):
        """The fixture is deliberately the DWARF, whose cell count DIFFERS from the pines'.

        A seven-colour fixture cannot discriminate here: the bound used to be the pines' own
        seven-entry list, so a pine reports the same figure whether the reader asks the constant
        or the artifact. Only an asset with a different number of cells can tell those apart.

        The three cells at stake are Wood Trunk, Hair and the Lantern flame. The flame is the one
        that matters: `dwarf_miner.py` says it "is a COLOUR and never an emitter. A pixel guard
        asserts that", and until this test the guard's subject was read by nothing on the artifact
        side. The generator checking its own output is not independent verification of it.
        """
        result = check(ROOT / "assets/gltf/SM_VoxelDwarf_Miner01.glb")

        self.assertEqual(result.returncode, 0, result.stderr)
        figures = [line for line in result.stdout.splitlines() if line.startswith("FIGURES ")]
        self.assertEqual(len(figures), 1, result.stdout)
        self.assertIn(
            "size_m=1.2x1.2x0.8 min_y_m=0.000000 centre_x_m=0.000000 centre_z_m=0.000000 "
            "palette=#E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50,"
            "#6B5B49,#34271C,#F0A63C tris=14398 verts=28796",
            figures[0],
        )
        # Named separately from the literal above, because the literal would still "pass" if the
        # reader were re-hardcoded to a ten-entry dwarf list -- which is the same defect wearing
        # the other family's clothes.
        palette = figures[0].split("palette=")[1].split(" ")[0].split(",")
        self.assertEqual(len(palette), 10, "the dwarf carries ten painted cells")
        self.assertIn("#F0A63C", palette, "the lantern flame must be read from the artifact")

    def test_a_palette_is_bounded_by_its_own_painted_cells_not_a_family_constant(self):
        """Same reader, two families, two different counts -- from ONE code path.

        This is the assertion that would fail if anyone re-introduced a per-family list.
        """
        dwarf = check(ROOT / "assets/gltf/SM_VoxelDwarf_Miner01.glb")
        pine = check(SIGNOFF / "export/SM_VoxelPine_Tree01.glb")
        self.assertEqual(dwarf.returncode, 0, dwarf.stderr)
        self.assertEqual(pine.returncode, 0, pine.stderr)

        def cells(result):
            figure = next(l for l in result.stdout.splitlines() if l.startswith("FIGURES "))
            return figure.split("palette=")[1].split(" ")[0].split(",")

        self.assertEqual(len(cells(dwarf)), 10)
        self.assertEqual(len(cells(pine)), 7)
        self.assertNotEqual(
            len(cells(dwarf)),
            len(cells(pine)),
            "one reader must report each family's OWN cell count; a shared constant cannot",
        )

    def test_off_centre_stale_asset_names_the_origin_clause(self):
        result = check(SIGNOFF / "tree.glb")

        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("FIGURES ", result.stdout)
        self.assertIn("origin-centring", result.stderr)
        self.assertIn("-0.100000", result.stderr)
        # AC5's identity clause, made visible: same published NAME, different published figures.
        self.assertIn("tris=5130", result.stdout)
        self.assertNotIn("palette=#4A3B2E", result.stdout)

    def test_off_grid_positions_and_unapplied_transforms_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            off_grid = pathlib.Path(directory) / "off-grid.glb"
            # -2.007 is off the project grid: 2.007 / 0.0125 = 160.56, not an integer.
            # It was -2.05 while the grid was 0.1 m, and -2.05 is EXACTLY 164 grid steps at
            # 0.0125 -- so the 2026-09-06 grid move would have made this fixture legal and
            # left the test passing vacuously against an accepted file. If the project grid
            # moves again, re-check this literal against it; a fixture that lands on the new
            # grid cannot fail the clause it exists to prove.
            write_tree02_mutant(
                off_grid,
                lambda document, binary, start: struct.pack_into("<f", binary, start, -2.007),
            )
            result = check(off_grid)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("grid clause", result.stderr)

            transformed = pathlib.Path(directory) / "translated.glb"
            write_tree02_mutant(
                transformed,
                lambda document, binary, start: document["nodes"][0].update(translation=[0, 1, 0]),
            )
            result = check(transformed)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("transform clause", result.stderr)

    def test_a_parent_node_cannot_hide_an_unapplied_transform(self):
        """A wrapper empty is the shape a Blender or MCP export arrives in."""
        with tempfile.TemporaryDirectory() as directory:
            wrapped = pathlib.Path(directory) / "SM_VoxelPine_Tree02.glb"

            def wrap(document, binary, start):
                mesh_node = next(
                    index for index, node in enumerate(document["nodes"]) if node.get("mesh") == 0
                )
                document["nodes"].append(
                    {"name": "Wrapper", "translation": [3.0, 5.0, -2.0], "children": [mesh_node]}
                )
                for scene in document.get("scenes", []):
                    scene["nodes"] = [len(document["nodes"]) - 1]

            write_tree02_mutant(wrapped, wrap)
            result = check(wrapped)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("transform clause", result.stderr)
            self.assertIn("Wrapper", result.stderr)

    def test_a_mismatched_file_basename_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            renamed = pathlib.Path(directory) / "not-the-published-name.glb"
            renamed.write_bytes((SIGNOFF / "export/SM_VoxelPine_Tree02.glb").read_bytes())
            result = check(renamed)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("naming clause", result.stderr)
            self.assertIn("SM_VoxelPine_Tree02", result.stderr)

    def test_non_finite_positions_name_a_clause_instead_of_crashing(self):
        with tempfile.TemporaryDirectory() as directory:
            broken = pathlib.Path(directory) / "SM_VoxelPine_Tree02.glb"
            write_tree02_mutant(
                broken,
                lambda document, binary, start: struct.pack_into(
                    "<f", binary, start, float("nan")
                ),
            )
            result = check(broken)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("geometry clause", result.stderr)
            self.assertIn("finite", result.stderr)
            self.assertNotIn("Traceback", result.stderr)

    def test_a_required_extension_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            extended = pathlib.Path(directory) / "SM_VoxelPine_Tree02.glb"
            write_tree02_mutant(
                extended,
                lambda document, binary, start: document.update(
                    extensionsRequired=["KHR_materials_unlit"]
                ),
            )
            result = check(extended)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("no-extensions clause", result.stderr)

    def test_a_uv_outside_the_atlas_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            stretched = pathlib.Path(directory) / "SM_VoxelPine_Tree02.glb"

            def push_uv(document, binary, start):
                primitive = document["meshes"][0]["primitives"][0]
                item = document["accessors"][primitive["attributes"]["TEXCOORD_0"]]
                view = document["bufferViews"][item["bufferView"]]
                offset = view.get("byteOffset", 0) + item.get("byteOffset", 0)
                struct.pack_into("<f", binary, offset, 1.5)

            write_tree02_mutant(stretched, push_uv)
            result = check(stretched)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("outside the 0-1 atlas", result.stderr)


if __name__ == "__main__":
    unittest.main()
