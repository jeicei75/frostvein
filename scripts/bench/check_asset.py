#!/usr/bin/env python3
"""Check the v1 mechanically-checkable glTF asset contract."""

# NOTE: This duplicates the small GLB reader in 10-2's frozen voxel_pine.py because that
# deliverable imports bpy at module scope and cannot be imported outside Blender.

import json
import pathlib
import struct
import sys
import zlib


MAX_GLB_BYTES = 16 * 1024 * 1024
MAX_DECODED_PNG_BYTES = 16 * 1024 * 1024
PROJECT_GRID_METRES = 0.1
GRID_TOLERANCE = 0.000_01
JSON_CHUNK = 0x4E4F534A
BIN_CHUNK = 0x004E4942
COMPONENT_SIZE = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
TYPE_COMPONENTS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}
PALETTE_HEX = [
    "#4A3B2E", "#6B5B49", "#2A3E34", "#364D3F", "#52715B", "#FFFFFF", "#D8E4EC",
]
ATLAS = 64


class AssetError(ValueError):
    """A concrete asset-contract clause is violated."""


def load_glb(path):
    """Return the JSON and binary chunks of one bounded, structurally valid GLB."""
    try:
        with path.open("rb") as handle:
            data = handle.read(MAX_GLB_BYTES + 1)
    except OSError as error:
        raise AssetError(f"file clause: cannot read {error}") from error
    if len(data) > MAX_GLB_BYTES:
        raise AssetError(f"file clause: exceeds {MAX_GLB_BYTES} byte limit")
    if len(data) < 12:
        raise AssetError("file clause: GLB header is truncated")
    magic, version, length = struct.unpack_from("<III", data)
    if magic != 0x46546C67 or version != 2 or length != len(data):
        raise AssetError("file clause: expected a complete GLB v2")

    document = binary = None
    offset = 12
    while offset < len(data):
        if offset + 8 > len(data):
            raise AssetError("file clause: GLB chunk header is truncated")
        chunk_length, chunk_type = struct.unpack_from("<II", data, offset)
        offset += 8
        end = offset + chunk_length
        if end > len(data):
            raise AssetError("file clause: GLB chunk exceeds file length")
        chunk = data[offset:end]
        if chunk_type == JSON_CHUNK:
            try:
                document = json.loads(chunk.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError) as error:
                raise AssetError("file clause: invalid JSON chunk") from error
        elif chunk_type == BIN_CHUNK:
            binary = chunk
        offset = end
    if document is None or binary is None:
        raise AssetError("file clause: requires JSON and binary chunks")
    return document, binary


def decode_png_rgb(png):
    """Decode the 8-bit RGB/RGBA PNG the GLB actually embeds."""
    position, idat, width, height, depth, colour = 8, b"", 0, 0, 0, 0
    while position < len(png):
        if position + 12 > len(png):
            raise AssetError("palette/material clause: truncated PNG chunk")
        length = struct.unpack_from(">I", png, position)[0]
        end = position + 12 + length
        if end > len(png):
            raise AssetError("palette/material clause: PNG chunk exceeds image")
        tag = png[position + 4:position + 8]
        payload = png[position + 8:position + 8 + length]
        if tag == b"IHDR":
            width, height, depth, colour = struct.unpack_from(">IIBB", payload, 0)
        elif tag == b"IDAT":
            idat += payload
        position = end
    if depth != 8 or colour not in (2, 6):
        raise AssetError("palette/material clause: expected 8-bit RGB/RGBA PNG")
    channels = 3 if colour == 2 else 4
    stride = width * channels
    raw_length = height * (stride + 1)
    if not width or not height or raw_length > MAX_DECODED_PNG_BYTES:
        raise AssetError("palette/material clause: PNG decoded size exceeds limit")
    try:
        decoder = zlib.decompressobj()
        raw = decoder.decompress(idat, raw_length + 1)
        raw += decoder.flush(raw_length + 1 - len(raw))
    except zlib.error as error:
        raise AssetError("palette/material clause: invalid PNG compression") from error
    if len(raw) != raw_length or decoder.unconsumed_tail:
        raise AssetError("palette/material clause: PNG scanline length is invalid")
    output, previous, offset = bytearray(height * stride), bytearray(stride), 0
    for row in range(height):
        filter_type = raw[offset]
        offset += 1
        line = bytearray(raw[offset:offset + stride])
        offset += stride
        if filter_type == 1:
            for index in range(channels, stride):
                line[index] = (line[index] + line[index - channels]) & 0xFF
        elif filter_type == 2:
            for index in range(stride):
                line[index] = (line[index] + previous[index]) & 0xFF
        elif filter_type == 3:
            for index in range(stride):
                left = line[index - channels] if index >= channels else 0
                line[index] = (line[index] + ((left + previous[index]) >> 1)) & 0xFF
        elif filter_type == 4:
            for index in range(stride):
                left = line[index - channels] if index >= channels else 0
                above, corner = previous[index], previous[index - channels] if index >= channels else 0
                pa, pb, pc = abs(above - corner), abs(left - corner), abs(left + above - 2 * corner)
                predictor = left if pa <= pb and pa <= pc else (above if pb <= pc else corner)
                line[index] = (line[index] + predictor) & 0xFF
        elif filter_type != 0:
            raise AssetError("palette/material clause: unsupported PNG filter")
        output[row * stride:(row + 1) * stride] = line
        previous = line
    return width, height, channels, bytes(output)


def palette_from_glb(document, binary):
    """Read the v1 pine atlas cells from the artifact, not the generator."""
    try:
        view = document["bufferViews"][document["images"][0]["bufferView"]]
        start = view.get("byteOffset", 0)
        png = binary[start:start + view["byteLength"]]
    except (IndexError, KeyError, TypeError) as error:
        raise AssetError("palette/material clause: embedded image is malformed") from error
    width, height, channels, pixels = decode_png_rgb(png)
    if width != ATLAS or height != ATLAS:
        raise AssetError("palette/material clause: expected a 64x64 atlas")
    values = []
    for index in range(len(PALETTE_HEX)):
        column, row = index % 4, index // 4
        x, y = column * 16 + 8, height - 1 - (row * 16 + 8)
        offset = (y * width + x) * channels
        values.append("#%02X%02X%02X" % tuple(pixels[offset:offset + 3]))
    return values


def accessor(document, binary, index):
    """Return a checked accessor description and its byte location."""
    try:
        item = document["accessors"][index]
        view = document["bufferViews"][item["bufferView"]]
        component_size = COMPONENT_SIZE[item["componentType"]]
        components = TYPE_COMPONENTS[item["type"]]
        count = item["count"]
    except (IndexError, KeyError, TypeError) as error:
        raise AssetError("geometry clause: malformed accessor") from error
    stride = view.get("byteStride", component_size * components)
    if not isinstance(count, int) or count < 0 or stride < component_size * components:
        raise AssetError("geometry clause: invalid accessor layout")
    start = view.get("byteOffset", 0) + item.get("byteOffset", 0)
    end = start if count == 0 else start + (count - 1) * stride + component_size * components
    if start < 0 or end > len(binary):
        raise AssetError("geometry clause: accessor exceeds binary chunk")
    return item, start, stride, component_size, components


def positions(document, binary, index):
    item, start, stride, component_size, components = accessor(document, binary, index)
    if item["componentType"] != 5126 or item["type"] != "VEC3":
        raise AssetError("geometry clause: POSITION must be float VEC3")
    return [struct.unpack_from("<fff", binary, start + row * stride) for row in range(item["count"])]


def has_applied_transform(node):
    return (
        node.get("translation", [0, 0, 0]) == [0, 0, 0]
        and node.get("rotation", [0, 0, 0, 1]) == [0, 0, 0, 1]
        and node.get("scale", [1, 1, 1]) == [1, 1, 1]
        and node.get("matrix") in (None, [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1])
    )


def contract_data(document, binary):
    """Read the contract figures after validating the common v1 GLB shape."""
    meshes = document.get("meshes", [])
    materials = document.get("materials", [])
    images = document.get("images", [])
    if len(meshes) != 1 or len(materials) != 1 or len(images) != 1:
        raise AssetError("one-mesh/material/image clause: expected exactly one of each")
    primitives = meshes[0].get("primitives", [])
    if len(primitives) != 1:
        raise AssetError("one-primitive clause: expected exactly one primitive")
    if document.get("extensionsUsed"):
        raise AssetError("no-extensions clause: extensionsUsed must be absent")
    if materials[0].get("doubleSided", False) is not False:
        raise AssetError("single-sided clause: doubleSided must be false")

    primitive = primitives[0]
    if primitive.get("material") != 0:
        raise AssetError("palette/material clause: primitive must use the sole material")
    texture = materials[0].get("pbrMetallicRoughness", {}).get("baseColorTexture", {}).get("index")
    try:
        texture_data = document["textures"][texture]
        sampler = document["samplers"][texture_data["sampler"]]
        if texture_data["source"] != 0 or sampler.get("magFilter") != 9728:
            raise AssetError("palette/material clause: embedded atlas must use NEAREST filtering")
        # The standing contract verifies the atlas exists and is nearest-filtered. Its declared
        # colour-to-role map remains a signoff comparison, so a different valid asset family is
        # not rejected by the pine palette literals above.
        palette_from_glb(document, binary)
        mesh_name = meshes[0]["name"]
        node = next(node for node in document["nodes"] if node.get("mesh") == 0)
        node_name = node["name"]
    except (IndexError, KeyError, StopIteration, TypeError) as error:
        raise AssetError("palette/material clause: malformed material mapping") from error
    if mesh_name != node_name:
        raise AssetError("naming clause: mesh and node names must agree")
    if not has_applied_transform(node):
        raise AssetError("transform clause: mesh node must have an applied identity transform")

    try:
        position_index = primitive["attributes"]["POSITION"]
        index_accessor = primitive["indices"]
    except (KeyError, TypeError) as error:
        raise AssetError("geometry clause: primitive needs POSITION and indices") from error
    point_data = positions(document, binary, position_index)
    index_data, *_ = accessor(document, binary, index_accessor)
    if not point_data or index_data["count"] % 3:
        raise AssetError("geometry clause: requires non-empty triangle indices")
    if any(
        abs(value / PROJECT_GRID_METRES - round(value / PROJECT_GRID_METRES)) > GRID_TOLERANCE
        for point in point_data for value in point
    ):
        raise AssetError("grid clause: POSITION values must use the 0.1 m project grid")
    minimum = tuple(min(point[axis] for point in point_data) for axis in range(3))
    maximum = tuple(max(point[axis] for point in point_data) for axis in range(3))
    tris = index_data["count"] // 3
    verts = len(point_data)
    if verts != tris * 2:
        raise AssetError("quad-soup clause: verts must equal tris/2 × 4")
    return minimum, maximum, tris, verts


def figures(path):
    document, binary = load_glb(path)
    minimum, maximum, tris, verts = contract_data(document, binary)
    size = tuple(maximum[axis] - minimum[axis] for axis in range(3))
    centre_x = (minimum[0] + maximum[0]) / 2
    centre_z = (minimum[2] + maximum[2]) / 2
    line = (
        f"FIGURES {path} size={size[0]:.1f}x{size[1]:.1f}x{size[2]:.1f} "
        f"min_y={minimum[1]:.6f} centre_x={centre_x:.6f} centre_z={centre_z:.6f} "
        f"tris={tris} verts={verts}"
    )
    if abs(minimum[1]) > 0.000_001:
        failure = f"origin-centring clause: min Y is {minimum[1]:.6f}, expected 0.000000"
    elif abs(centre_x) > 0.000_001 or abs(centre_z) > 0.000_001:
        failure = (
            "origin-centring clause: centre X/Z are "
            f"{centre_x:.6f}/{centre_z:.6f}, expected 0.000000/0.000000"
        )
    else:
        failure = None
    return line, failure


def main(argv):
    if not argv:
        print("usage: check_asset.py <asset.glb> [asset.glb ...]", file=sys.stderr)
        return 2
    for value in argv:
        path = pathlib.Path(value)
        if path.suffix.lower() != ".glb":
            print(f"FAIL {path}: file clause: expected a .glb path", file=sys.stderr)
            return 1
        try:
            line, failure = figures(path)
            print(line, flush=True)
            if failure:
                raise AssetError(failure)
        except AssetError as error:
            print(f"FAIL {path}: {error}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
