"""Black-box checks for the asset-contract instrument."""

import json
import pathlib
import re
import subprocess
import struct
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
CHECKER = ROOT / "scripts/bench/check_asset.py"
sys.path.insert(0, str(ROOT / "scripts/bench"))

import check_asset  # noqa: E402
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


def atlas_parts(colours, side=check_asset.ATLAS):
    """A minimal in-memory `(document, binary)` carrying a `side`x`side` atlas of `colours`.

    Built here rather than by mutating a shipped `.glb` because the cases below need atlases no
    published family has: an entirely unpainted one, one with a HOLE between painted cells, and
    one LARGER than the pines' 64x64 -- which is legal from 2026-09-13, when the reader stopped
    pinning the atlas to that side length.
    """
    return image_parts(png_bytes(cell_pixels(colours, side), side))


def cell_pixels(colours, side):
    """The pixel rows of an atlas: one flat colour per CELL-sized cell, unpainted cells black."""
    cells_per_row = side // check_asset.CELL
    pixels = bytearray(b"\x00" * (side * side * 3))
    for index, colour in enumerate(colours):
        if colour is None:
            continue
        rgb = bytes.fromhex(colour)
        column, row = index % cells_per_row, index // cells_per_row
        for dy in range(check_asset.CELL):
            for dx in range(check_asset.CELL):
                x = column * check_asset.CELL + dx
                # Row 0 of the image is the TOP; the reader indexes cells from the bottom.
                y = side - 1 - (row * check_asset.CELL + dy)
                pixels[(y * side + x) * 3:(y * side + x) * 3 + 3] = rgb
    return pixels


def painted_pixels(side, first, second, third):
    """A map that is NOT cell-quantised: a 1 px checker, with `third` painted on one pixel.

    One pixel of a third colour is what makes the census ORDER assertable -- the checker halves
    are the same size, so without it two colours tie and either order is correct.
    """
    pixels = bytearray(side * side * 3)
    for y in range(side):
        for x in range(side):
            rgb = bytes.fromhex(first if (x + y) % 2 else second)
            pixels[(y * side + x) * 3:(y * side + x) * 3 + 3] = rgb
    pixels[0:3] = bytes.fromhex(third)
    return pixels


def png_bytes(pixels, side):
    """Encode `pixels` as the 8-bit RGB PNG the GLB would embed."""
    import zlib

    raw = b"".join(
        b"\x00" + bytes(pixels[y * side * 3:(y + 1) * side * 3]) for y in range(side)
    )

    def chunk(kind, payload):
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", side, side, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw))
        + chunk(b"IEND", b"")
    )
    return png


def image_parts(png):
    """Wrap one encoded PNG as the `(document, binary)` pair the image readers take."""
    document = {
        "images": [{"bufferView": 0}],
        "bufferViews": [{"byteOffset": 0, "byteLength": len(png)}],
    }
    return document, png


# The ten cells of the r3 voxel dwarf, kept as a literal because what this fixture exists to do is
# report a cell count that is NOT the pines' seven. The family it came from no longer ships -- see
# `test_an_atlas_reports_its_own_cell_count_not_a_family_constant`.
TEN_CELLS = ["E9D2BB", "5E4632", "FFFFFF", "5F7A6A", "474B41",
             "A9B2AC", "8B6B50", "6B5B49", "34271C", "F0A63C"]


def retexture_tree02(document, binary, png):
    """Point tree02's sole image at `png`, appended to its binary chunk, in place."""
    binary += b"\0" * (-len(binary) % 4)
    document["bufferViews"].append(
        {"buffer": 0, "byteOffset": len(binary), "byteLength": len(png)}
    )
    binary += png
    document["images"][0] = {
        "bufferView": len(document["bufferViews"]) - 1,
        "mimeType": "image/png",
        "name": "T_Test",
    }
    document["buffers"][0]["byteLength"] = len(binary)


def repaint_tree02(document, binary):
    """Swap tree02's 64x64 atlas for a map of the same size that is NOT cell-quantised.

    The two artifacts then differ in ONE property -- whether the embedded image is an atlas --
    which is what the profile is derived from, so a clause that changes verdict between them is
    changing verdict on the profile and nothing else.
    """
    retexture_tree02(document, binary, png_bytes(
        painted_pixels(check_asset.ATLAS, "0A141E", "28323C", "F0A63C"), check_asset.ATLAS))


def ten_cell_atlas_tree02(document, binary):
    """Give tree02 a TEN-cell atlas -- an atlas whose count differs from the pines' seven."""
    retexture_tree02(document, binary, png_bytes(
        cell_pixels(TEN_CELLS, check_asset.ATLAS), check_asset.ATLAS))


def break_the_voxel_clauses(document, binary, start):
    """Put tree02 off the project grid AND out of quad soup -- one vertex, one index count.

    -2.007 is the same off-grid literal the grid-clause test uses (2.007 / 0.0125 = 160.56), and
    dropping three indices leaves 5893 triangles against 11788 vertices, which is no longer
    tris x 2.
    """
    struct.pack_into("<f", binary, start, -2.007)
    document["accessors"][document["meshes"][0]["primitives"][0]["indices"]]["count"] -= 3


def read_palette(document, binary):
    """Decode, classify, and read -- the three steps `contract_data` takes, in one call."""
    width, height, channels, pixels = check_asset.embedded_image(document, binary)
    reader = (
        check_asset.palette_from_atlas
        if check_asset.is_palette_atlas(width, height, channels, pixels)
        else check_asset.palette_from_painted_map
    )
    return reader(width, height, channels, pixels)


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

    def test_an_atlas_reports_its_own_cell_count_not_a_family_constant(self):
        """One reader, two atlases, two different counts -- and NEITHER fixture is the runtime slot.

        The bound used to be the PINES' seven-entry hex list, so a pine reports the same figure
        whether the reader asks the constant or the artifact: only an asset with a different cell
        count can discriminate. Until 2026-09-13 that asset was `assets/gltf/`'s promoted dwarf,
        which was the r3 voxel figure with a ten-cell atlas.

        **That fixture choice was wrong, and promoting round 8 is what exposed it.** The runtime
        slot is promoted by the operator whenever a new asset is signed off -- it is a MOVING
        TARGET by design -- so a test pinned to its literal figures fails on every promotion, and
        the pressure that creates is to avoid promoting in order to keep the gate green. A test
        must never make shipping the thing it checks more expensive.

        So the discriminating atlas is synthesised here from a tracked artifact instead: tree02
        carrying a ten-cell atlas. The count it must report, ten, is a property of the fixture in
        this file, and nothing anyone promotes can change it.
        """
        with tempfile.TemporaryDirectory() as directory:
            ten = pathlib.Path(directory) / "SM_VoxelPine_Tree02.glb"
            write_tree02_mutant(
                ten, lambda document, binary, start: ten_cell_atlas_tree02(document, binary)
            )
            atlas_ten = check(ten)
            pine = check(SIGNOFF / "export/SM_VoxelPine_Tree01.glb")
            self.assertEqual(atlas_ten.returncode, 0, atlas_ten.stderr)
            self.assertEqual(pine.returncode, 0, pine.stderr)

            def cells(result):
                figure = next(
                    line for line in result.stdout.splitlines() if line.startswith("FIGURES ")
                )
                self.assertIn("profile=voxel-atlas", figure)
                return figure.split("palette=")[1].split(" ")[0].split(",")

            self.assertEqual(len(cells(atlas_ten)), 10)
            self.assertEqual(len(cells(pine)), 7)
            self.assertNotEqual(
                len(cells(atlas_ten)),
                len(cells(pine)),
                "one reader must report each atlas's OWN cell count; a shared constant cannot",
            )
            self.assertEqual(cells(atlas_ten), ["#" + value for value in TEN_CELLS])

    def test_the_promoted_runtime_dwarf_passes_and_its_lantern_flame_is_read_from_the_artifact(
        self,
    ):
        """What is true of ANY promoted dwarf, so promotion rarely has to edit this test.

        "Never" was the claim until the r8 -> r14 promotion, and it was too strong: the flame
        assertion below is a content literal of exactly the kind this docstring disclaims, and
        r14 broke it. See the comment on that assertion.

        The property worth keeping from the old version: `dwarf_miner.py` says the lantern flame
        "is a COLOUR and never an emitter. A pixel guard asserts that" -- and until that test
        existed, the guard's subject was read by nothing on the artifact side. A generator
        checking its own output is not independent verification of it.

        Deliberately NOT asserted: the triangle count, the size, the colour count, the profile, or
        the revision in the mesh name. Every one of those is a property of whichever asset is
        currently promoted, and the r3 -> r8 promotion changed all five (14,398 tris of
        voxel-atlas became 3,955 of painted-map). The clauses still run on it, so a broken
        promotion fails here -- it just fails on the contract rather than on a stale literal.
        """
        result = check(ROOT / "assets/gltf/SM_VoxelDwarf_Miner01.glb")

        self.assertEqual(result.returncode, 0, result.stderr)
        figures = [line for line in result.stdout.splitlines() if line.startswith("FIGURES ")]
        self.assertEqual(len(figures), 1, result.stdout)
        palette = figures[0].split("palette=")[1].split(" ")[0].split(",")
        # The approved flame cell, narrowed back from the pair this carried while #94 was open.
        # `#DE610C` was `#F0A63C` put through `srgb_to_linear` by r14's `hex_rgb`, which wrote it
        # into an 8-bit image whose `pixels` are already display-encoded -- so the packed PNG
        # shipped linear bytes and the game, reading baseColorTexture as sRGB per the glTF spec,
        # drew the whole figure too dark. r17 dropped the transform and the promoted artifact now
        # reads `#F0A63C` directly, so the widening is gone rather than left in place. See #94.
        self.assertIn(
            "#F0A63C",
            palette,
            "the lantern flame must be read from the artifact; palette was %r" % (palette,),
        )
        mesh = figures[0].split("mesh=")[1].split(" ")[0]
        self.assertEqual(
            re.sub(r"_r\d+$", "", mesh),
            "SM_VoxelDwarf_Miner01",
            "the promoted mesh must be this asset, bar its revision",
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

    def test_the_voxel_only_clauses_apply_to_an_atlas_and_not_to_a_painted_map(self):
        """The 2026-09-13 change, on two artifacts that differ ONLY in their embedded image.

        The grid clause and the quad-soup clause describe the generated-voxel pipeline: every
        vertex on the authored lattice, every face an independent quad sampling one cell. A
        hand-modelled painted figure satisfies neither and is not wrong for it -- the round-7
        dwarf is off-lattice by construction and shares vertices, and before this change the
        checker never reached ANY clause for it, so it certified nothing at all.

        Both fixtures carry the same broken geometry. If the skip were keyed on something other
        than the profile, or if it leaked into the atlas family, one of these two assertions
        fails -- which is the whole risk of a derived profile, made a test.
        """
        with tempfile.TemporaryDirectory() as directory:
            atlas = pathlib.Path(directory) / "atlas" / "SM_VoxelPine_Tree02.glb"
            painted = pathlib.Path(directory) / "painted" / "SM_VoxelPine_Tree02.glb"
            atlas.parent.mkdir()
            painted.parent.mkdir()
            write_tree02_mutant(atlas, break_the_voxel_clauses)
            write_tree02_mutant(painted, lambda document, binary, start: (
                break_the_voxel_clauses(document, binary, start),
                repaint_tree02(document, binary),
            ))

            rejected = check(atlas)
            self.assertEqual(rejected.returncode, 1, rejected.stdout + rejected.stderr)
            # The grid clause firing IS the classification: no other path runs it. The FIGURES
            # line never prints for a rejected asset, so there is no `profile=` to read here.
            self.assertIn("grid clause", rejected.stderr)

            accepted = check(painted)
            self.assertEqual(accepted.returncode, 0, accepted.stdout + accepted.stderr)
            self.assertIn("profile=painted-map", accepted.stdout)
            # The census, not cells: the checker map is a 1 px checker of two colours plus one
            # pixel of a third, so the painted reader must report exactly those three.
            self.assertIn("palette=#0A141E,#28323C,#F0A63C", accepted.stdout)

    def test_a_revision_suffix_is_allowed_on_the_mesh_name_and_nothing_else_is(self):
        """Round 4 put the revision in the datablock names and in no filename.

        `SM_VoxelDwarf_Miner01.glb` therefore publishes a mesh called
        `SM_VoxelDwarf_Miner01_r7`, which the naming clause rejected outright. It may not become
        a licence for any other disagreement, and the revision has to stay VISIBLE -- a suffix
        nobody prints cannot announce a stale export -- so `mesh=` is asserted too.
        """
        with tempfile.TemporaryDirectory() as directory:
            for suffix, expected in (("_r9", 0), ("_v9", 1), ("_r9_old", 1)):
                target = pathlib.Path(directory) / suffix / "SM_VoxelPine_Tree02.glb"
                target.parent.mkdir()

                def rename(document, binary, start, suffix=suffix):
                    name = f"SM_VoxelPine_Tree02{suffix}"
                    document["meshes"][0]["name"] = name
                    for node in document["nodes"]:
                        if node.get("mesh") == 0:
                            node["name"] = name

                write_tree02_mutant(target, rename)
                result = check(target)
                self.assertEqual(
                    result.returncode, expected, f"{suffix}: {result.stdout}{result.stderr}"
                )
                if expected == 0:
                    self.assertIn(f"mesh=SM_VoxelPine_Tree02{suffix}", result.stdout)
                else:
                    self.assertIn("naming clause", result.stderr)




class PaletteReadTests(unittest.TestCase):
    """The palette reader's own boundaries, which no shipped asset exercises."""

    def test_an_entirely_unpainted_atlas_is_refused_rather_than_reported_as_empty(self):
        """It returned `[]` and the caller printed `palette=` and exited 0.

        An empty palette is not a description of an asset, it is the absence of one -- and it read
        exactly like a healthy asset nobody had looked closely at.
        """
        document, binary = atlas_parts([])
        with self.assertRaises(check_asset.AssetError) as caught:
            read_palette(document, binary)
        self.assertIn("no palette at all", str(caught.exception))

    def test_a_hole_between_painted_cells_is_named_not_reported_as_black(self):
        """The trailing-black trim cannot see an interior gap, so it became a phantom colour.

        `#000000` in the middle of the list reads to the eye doing the signoff as a deliberate
        black. No shipped family paints black, so this is a gap in the paint.
        """
        document, binary = atlas_parts(["0A141E", None, "28323C"])
        with self.assertRaises(check_asset.AssetError) as caught:
            read_palette(document, binary)
        self.assertIn("[1]", str(caught.exception))
        self.assertIn("not a colour", str(caught.exception))

    def test_a_painted_run_reads_back_exactly_and_the_unpainted_tail_is_trimmed(self):
        """The control: the two refusals above must not be firing on healthy input."""
        document, binary = atlas_parts(["0A141E", "28323C", "F0A63C"])
        self.assertEqual(
            read_palette(document, binary),
            ["#0A141E", "#28323C", "#F0A63C"],
        )

    def test_an_atlas_is_recognised_by_its_cells_and_not_by_its_side_length(self):
        """64 was the PINES' cell count showing through, and it was the whole discriminator.

        A voxel family needing more than sixteen colours would ship a 128x128 atlas of the same
        16 px cells; under the old size test it was read as a painted map and silently lost the
        grid and quad-soup clauses. Seventeen cells cannot fit in 64x64, so this fixture can only
        be read by a reader that takes its cell count from the image.
        """
        colours = ["%02X0A14" % (index + 1) for index in range(17)]
        document, binary = atlas_parts(colours, side=128)
        self.assertTrue(check_asset.is_palette_atlas(*check_asset.embedded_image(document, binary)))
        self.assertEqual(read_palette(document, binary), ["#" + value for value in colours])

    def test_a_map_that_is_not_cell_quantised_is_read_as_a_census_most_painted_first(self):
        """The painted family's `palette=`: every colour present, in coverage order.

        Order matters because the line is read by eye against the approved sheet -- the figure
        should open with the colours that carry the asset. The fixture is a 1 px checker, which
        no cell-uniform test can mistake for an atlas.
        """
        document, binary = image_parts(
            png_bytes(painted_pixels(64, "0A141E", "28323C", "F0A63C"), 64)
        )
        self.assertFalse(check_asset.is_palette_atlas(*check_asset.embedded_image(document, binary)))
        self.assertEqual(read_palette(document, binary), ["#0A141E", "#28323C", "#F0A63C"])



def animation_glb(values_by_channel, times=(0.0, 0.5, 1.0), name="Walk"):
    """A minimal document+binary carrying one clip, for exercising the animation clauses.

    Built by hand rather than exported, so each test can put exactly one thing wrong. The
    accessors are laid out back to back with no stride games -- what is under test is the
    clause, not the reader, which `geometry clause` cases already cover.
    """
    binary = b""
    accessors, views = [], []

    def add(rows, components):
        nonlocal binary
        offset = len(binary)
        for row in rows:
            binary += struct.pack("<" + "f" * components, *row)
        views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(binary) - offset})
        accessors.append({
            "bufferView": len(views) - 1, "componentType": 5126, "count": len(rows),
            "type": {1: "SCALAR", 3: "VEC3", 4: "VEC4"}[components],
        })
        return len(accessors) - 1

    channels, samplers = [], []
    for node_index, path, rows in values_by_channel:
        components = len(rows[0])
        sampler = {"input": add([(value,) for value in times[:len(rows)]], 1),
                   "output": add(rows, components), "interpolation": "LINEAR"}
        samplers.append(sampler)
        channels.append({"sampler": len(samplers) - 1,
                         "target": {"node": node_index, "path": path}})
    document = {
        "nodes": [{"name": "root"}, {"name": "hip.L"}],
        "bufferViews": views,
        "accessors": accessors,
        "animations": [{"name": name, "channels": channels, "samplers": samplers}],
    }
    return document, binary


class AnimationClauseTests(unittest.TestCase):
    """The clauses that did not exist until a walk cycle was on the way.

    Before these, `check_asset.py` had no animation clause of any kind: a clip that was
    missing, empty, single-keyframed or drifting passed every gate in the repo. That is the
    same silent-acceptance shape the sim's own filter trap had -- the artifact is accepted
    and does nothing, and every test stays green.
    """

    def test_an_asset_with_no_clips_reports_so_rather_than_failing(self):
        self.assertEqual(check_asset.animation_facts({"nodes": []}, b""), "anims=-")

    def test_a_closed_loop_passes_and_is_reported(self):
        document, binary = animation_glb([
            (0, "translation", [(0.0, 0.0, 0.0), (0.0, 0.02, 0.0), (0.0, 0.0, 0.0)]),
            (1, "rotation", [(0.0, 0.0, 0.0, 1.0), (0.1, 0.0, 0.0, 0.995), (0.0, 0.0, 0.0, 1.0)]),
        ])
        self.assertEqual(check_asset.animation_facts(document, binary), "anims=Walk:2ch@1.00s")

    def test_a_vertical_bob_closes_but_a_drift_does_not(self):
        """The in-place rule, which is the whole reason this clause exists.

        The client owns the dwarf's world position -- `blended_translation` lerps him between
        cells every tick -- so a clip that displaces him too makes him skate. Both cases below
        move the SAME channel by the same amount; only one returns to where it started.
        """
        bob = animation_glb([(0, "translation", [(0.0, 0.0, 0.0), (0.0, 0.9, 0.0), (0.0, 0.0, 0.0)])])
        self.assertEqual(check_asset.animation_facts(*bob), "anims=Walk:1ch@1.00s")

        drift = animation_glb([(0, "translation", [(0.0, 0.0, 0.0), (0.0, 0.45, 0.0), (0.0, 0.9, 0.0)])])
        with self.assertRaises(check_asset.AssetError) as raised:
            check_asset.animation_facts(*drift)
        self.assertIn("does not close its loop", str(raised.exception))
        self.assertIn("root", str(raised.exception))

    def test_a_quaternion_that_loops_with_a_flipped_sign_is_closed(self):
        """q and -q are the same rotation, so a baked loop may end on either."""
        document, binary = animation_glb([
            (1, "rotation", [(0.0, 0.1, 0.0, 0.995), (0.0, 0.5, 0.0, 0.866), (0.0, -0.1, 0.0, -0.995)]),
        ])
        self.assertEqual(check_asset.animation_facts(document, binary), "anims=Walk:1ch@1.00s")

    def test_a_single_keyframe_channel_cannot_animate_and_is_rejected(self):
        document, binary = animation_glb([(1, "rotation", [(0.0, 0.0, 0.0, 1.0)])])
        with self.assertRaises(check_asset.AssetError) as raised:
            check_asset.animation_facts(document, binary)
        self.assertIn("cannot animate anything", str(raised.exception))

    def test_a_clip_with_no_channels_is_rejected_as_inert(self):
        document, binary = animation_glb([])
        with self.assertRaises(check_asset.AssetError) as raised:
            check_asset.animation_facts(document, binary)
        self.assertIn("animates nothing", str(raised.exception))

    def test_a_channel_targeting_a_missing_node_is_rejected(self):
        document, binary = animation_glb([(9, "rotation", [(0.0, 0.0, 0.0, 1.0), (0.0, 0.0, 0.0, 1.0)])])
        with self.assertRaises(check_asset.AssetError) as raised:
            check_asset.animation_facts(document, binary)
        self.assertIn("node that does not exist", str(raised.exception))

if __name__ == "__main__":
    unittest.main()
