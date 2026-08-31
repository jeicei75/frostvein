#!/usr/bin/env python3
"""Offline geometry and simulation-resolution measurements for story 10.6.

The benchmark intentionally uses only the Python standard library.  Its detail rule is a
measurement stand-in, not game art: it gives an exposed sub-cell surface a small seeded height
variation so the greedy mesher has real fine geometry to account for.
"""

import argparse
import collections
import json
import resource
import subprocess
import sys
import tempfile
import time
from pathlib import Path


WORLD_SEED = 0xF005_7E1A
CONTROL_FACES = 61_142
CONTROL_QUADS = 19_264
NEIGHBOURS = ((0, -1), (0, 1), (1, -1), (1, 1), (2, -1), (2, 1))
# Guard the process before Python object overhead can exhaust the devpod.  This is a deliberately
# conservative benchmark hard limit, not a renderer limit.
MAX_FINE_FACES = 4_000_000
BUILD_TIME_BUDGET_SECONDS = 120.0
CHUNK_EDGE_CELLS = 16
MAX_SNAPSHOT_BYTES = 64 * 1024 * 1024


def detail_offset(seed, x, y, z):
    """Return a deterministic exposed-surface displacement in fine voxels.

    // NOTE: This is a measurement stand-in for 10.4's authored terrain look, not a visual
    decision.  The small value-noise displacement deliberately breaks flat greedy runs.
    """
    value = seed ^ (x * 0x9E3779B1) ^ (y * 0x85EBCA77) ^ (z * 0xC2B2AE3D)
    value ^= value >> 16
    value = (value * 0x7FEB352D) & 0xFFFFFFFF
    value ^= value >> 15
    return value % 5 - 2


def detail_depth(seed, x, y, z, k):
    """Return the depth of one closed top-surface pit, bounded by its fine cell height."""
    return min(abs(detail_offset(seed, x, y, z)), k - 1)


def _dims(snapshot):
    dims = snapshot["dims"]
    return dims["x"], dims["y"], dims["z"]


def _tile(snapshot, x, y, z):
    dx, dy, dz = _dims(snapshot)
    if not (0 <= x < dx and 0 <= y < dy and 0 <= z < dz):
        return "empty"
    return snapshot["tiles"][x + y * dx + z * dx * dy]


def _material(tile):
    if isinstance(tile, dict):
        return tile.get("solid", tile.get("ramp"))
    return None


def _solid(snapshot, x, y, z):
    return _material(_tile(snapshot, x, y, z)) is not None


def _face_groups(snapshot):
    """Collect actual exposed cell faces keyed by direction and physical coarse slab."""
    groups = collections.defaultdict(list)
    dx, dy, dz = _dims(snapshot)
    for z in range(dz):
        for y in range(dy):
            for x in range(dx):
                material = _material(_tile(snapshot, x, y, z))
                if material is None:
                    continue
                coordinate = (x, y, z)
                for axis, sign in NEIGHBOURS:
                    neighbour = list(coordinate)
                    neighbour[axis] += sign
                    if _solid(snapshot, *neighbour):
                        continue
                    slab = coordinate[axis] + (1 if sign > 0 else 0)
                    # Match the repository's reference greedy mesher: cyclic axes, rather than
                    # merely "the other two". Rectangle choice is traversal-order dependent.
                    u, v = (axis + 1) % 3, (axis + 2) % 3
                    groups[(axis, sign, slab)].append((coordinate[u], coordinate[v], material))
    return groups


def _greedy_quads(mask):
    """Count maximal same-material rectangles in one co-planar face mask."""
    used = set()
    quads = 0
    # Scan rows first.  This is the reference mesher's rectangle tie-break and is load-bearing:
    # scanning columns first creates 19,353 quads on the real world, not the independent 19,264
    # control result.
    for u, v in sorted(mask, key=lambda point: (point[1], point[0])):
        if (u, v) in used:
            continue
        material = mask[(u, v)]
        width = 1
        while mask.get((u + width, v)) == material and (u + width, v) not in used:
            width += 1
        height = 1
        while all(
            mask.get((u + offset, v + height)) == material
            and (u + offset, v + height) not in used
            for offset in range(width)
        ):
            height += 1
        for du in range(width):
            for dv in range(height):
                used.add((u + du, v + dv))
        quads += 1
    return quads


def geometry_summary(snapshot, k=1, detail=True):
    """Measure exposed fine faces and greedy quads for a snapshot.

    At k=1 the exported world is meshed exactly as shipped. At larger k every exposed face is
    expanded to k² fine samples. Detail carves deterministic pits into top faces and emits the
    connecting vertical faces between different pit depths, so the measured surface is closed.
    No-detail is the control that must collapse back to k=1's quads.
    """
    if not isinstance(k, int) or k < 1:
        raise ValueError("k must be a positive integer")
    groups = _face_groups(snapshot)
    coarse_faces = sum(len(group) for group in groups.values())
    exposed_faces = coarse_faces * k * k
    assert_workload_limit(exposed_faces, k, detail)
    if not detail and k > 1:
        # A flat fine surface is exactly the k=1 greedy control scaled in tessellation only.
        # Returning this invariant avoids allocating k² identical Python cells for the RED run.
        coarse = geometry_summary(snapshot, k=1, detail=True)
        return {
            "exposed_faces": exposed_faces,
            "greedy_quads": coarse["greedy_quads"],
            "triangles": coarse["triangles"],
        }
    quads = 0
    connector_faces = 0
    for (axis, sign, slab), faces in groups.items():
        masks = collections.defaultdict(dict)
        top_planes = {}
        for base_u, base_v, material in faces:
            for du in range(k):
                for dv in range(k):
                    u = base_u * k + du
                    v = base_v * k + dv
                    plane = slab * k
                    if detail and k > 1 and axis == 2 and sign > 0:
                        # A pit removes top fine voxels; its bottom remains solid. This is a
                        # voxel-valid downward displacement rather than a floating face patch.
                        plane -= detail_depth(WORLD_SEED, slab * k, u, v, k)
                    masks[plane][(u, v)] = material
                    if detail and k > 1 and axis == 2 and sign > 0:
                        top_planes[(u, v)] = (plane, material)
        quads += sum(_greedy_quads(mask) for mask in masks.values())
        # Adjacent top samples at different heights expose vertical voxel faces. Compare only
        # neighbours inside this same coarse top slab: cliffs between slabs are already supplied
        # by their ordinary exposed side faces above.
        connectors = collections.defaultdict(dict)
        for (u, v), (plane, material) in top_planes.items():
            for du, dv in ((1, 0), (0, 1)):
                neighbour = top_planes.get((u + du, v + dv))
                if neighbour is None or neighbour[0] == plane:
                    continue
                other_plane, other_material = neighbour
                lower, higher = sorted((plane, other_plane))
                if du:
                    connector_axis = 0
                    connector_sign = 1 if plane > other_plane else -1
                    connector_plane = u + 1
                    coordinates = ((v, z) for z in range(lower, higher))
                else:
                    connector_axis = 1
                    connector_sign = 1 if plane > other_plane else -1
                    connector_plane = v + 1
                    coordinates = ((z, u) for z in range(lower, higher))
                connector_material = material if plane > other_plane else other_material
                mask = connectors[(connector_axis, connector_sign, connector_plane)]
                for coordinate in coordinates:
                    mask[coordinate] = connector_material
        connector_faces += sum(len(mask) for mask in connectors.values())
        quads += sum(_greedy_quads(mask) for mask in connectors.values())
    return {
        "exposed_faces": exposed_faces + connector_faces,
        "greedy_quads": quads,
        "triangles": quads * 2,
    }


def assert_control(summary):
    """Reject a sweep unless its k=1 geometry is the independently measured real-world oracle."""
    if summary.get("exposed_faces") != CONTROL_FACES or summary.get("greedy_quads") != CONTROL_QUADS:
        raise ValueError(
            f"k=1 control mismatch: exposed_faces={summary.get('exposed_faces')} "
            f"(expected {CONTROL_FACES}), greedy_quads={summary.get('greedy_quads')} "
            f"(expected {CONTROL_QUADS})"
        )


def assert_workload_limit(fine_faces, k, detail):
    # A top pit can expose up to two runs of vertical connector faces in addition to its sampled
    # top. Reserve a conservative 3× budget before allocating any masks.
    estimated_faces = fine_faces * 3 if detail else fine_faces
    if detail and estimated_faces > MAX_FINE_FACES:
        raise ValueError(
            f"k={k} can need {estimated_faces:,} detailed faces, exceeding the "
            f"{MAX_FINE_FACES:,} hard limit"
        )


def assert_snapshot_size(size):
    if size > MAX_SNAPSHOT_BYTES:
        raise ValueError(
            f"snapshot is too large: {size:,} bytes exceeds {MAX_SNAPSHOT_BYTES:,}-byte limit"
        )


def sim_axis_cost(raw_snapshot, sim_k):
    """Scale the actual tile JSON payload for an unbuilt finer simulation grid.

    This deliberately repeats every encoded real tile value, including its exact punctuation,
    instead of multiplying an average byte count. Snapshot framing and dynamic entities remain
    one copy; only the terrain tile array changes.
    """
    if sim_k not in (1, 2, 4):
        raise ValueError("sim_k must be one of 1, 2, or 4")
    marker = '"tiles":'
    start = raw_snapshot.index(marker) + len(marker)
    _, end = json.JSONDecoder().raw_decode(raw_snapshot[start:])
    encoded_tiles = raw_snapshot[start : start + end]
    tiles = json.loads(encoded_tiles)
    cells = len(tiles) * sim_k**3
    # `encoded_tiles` is `[` + values separated by commas + `]`. Values are copied verbatim;
    # commas are deterministic one-byte separators in the daemon's compact wire JSON.
    value_bytes = len(encoded_tiles) - 2 - max(0, len(tiles) - 1)
    tile_bytes = len(marker) + 2 + value_bytes * sim_k**3 + max(0, cells - 1)
    # All non-tile fields are re-serialized after mapping their real coordinates to the finer
    # grid. Dims and each wire position can gain digits, so retaining the old envelope would
    # understate the real protocol size.
    scaled = json.loads(raw_snapshot)
    if "dims" in scaled:
        scaled["dims"] = {axis: value * sim_k for axis, value in scaled["dims"].items()}
    for collection in ("entities", "designations", "zones", "items"):
        for entry in scaled.get(collection, []):
            if "pos" in entry:
                entry["pos"] = [coordinate * sim_k for coordinate in entry["pos"]]
    scaled["tiles"] = []
    encoded_scaled = json.dumps(scaled, separators=(",", ":"))
    empty_tiles = '"tiles":[]'
    if empty_tiles not in encoded_scaled:
        raise ValueError("snapshot encoding did not retain an empty tiles field")
    snapshot_bytes = len(encoded_scaled) - len(empty_tiles) + tile_bytes
    if raw_snapshot.endswith("\n"):
        snapshot_bytes += 1
    return {
        "sim_k": sim_k,
        "cells": cells,
        "tile_bytes": tile_bytes,
        "snapshot_bytes": snapshot_bytes,
    }


def _chunks(snapshot):
    dx, dy, _ = _dims(snapshot)
    return ((dx + CHUNK_EDGE_CELLS - 1) // CHUNK_EDGE_CELLS) * (
        (dy + CHUNK_EDGE_CELLS - 1) // CHUNK_EDGE_CELLS
    )


def measure(snapshot, k, detail):
    """Return one geometry row, including real elapsed time and process peak RSS."""
    started = time.perf_counter()
    summary = geometry_summary(snapshot, k=k, detail=detail)
    elapsed = time.perf_counter() - started
    # Linux reports KiB, macOS bytes. The devpod is Linux; retain a usable POSIX fallback.
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    if sys.platform != "darwin":
        peak *= 1024
    return {
        "k": k,
        **summary,
        "chunks": _chunks(snapshot),
        "mesh_build_seconds": elapsed,
        "peak_memory_bytes": peak,
    }


def _load_snapshot(path):
    assert_snapshot_size(path.stat().st_size)
    with path.open(encoding="utf-8") as source:
        return json.load(source)


def _export_snapshot():
    repo = Path(__file__).resolve().parents[2]
    temporary = tempfile.TemporaryDirectory(prefix="frostvein-resolution-")
    path = Path(temporary.name) / "world.json"
    subprocess.run(
        [sys.executable, str(repo / "scripts" / "bench" / "export_world.py"), str(path)],
        cwd=repo,
        check=True,
    )
    return temporary, path, _load_snapshot(path)


def _print_row(row):
    print(
        "k={k} exposed_faces={exposed_faces} greedy_quads={greedy_quads} triangles={triangles} "
        "chunks={chunks} mesh_build_seconds={mesh_build_seconds:.3f} peak_memory_bytes={peak_memory_bytes}".format(
            **row
        )
    )


def _sweep(snapshot):
    control = measure(snapshot, 1, detail=True)
    assert_control(control)
    _print_row(control)
    last = control
    k = 2
    while True:
        estimated_faces = control["exposed_faces"] * k * k
        if estimated_faces * 3 > MAX_FINE_FACES:
            print(
                f"wall: hard face limit at k={k}: up to {estimated_faces * 3} detailed faces exceeds "
                f"{MAX_FINE_FACES}; last_completed_k={last['k']}"
            )
            return
        row = measure(snapshot, k, detail=True)
        _print_row(row)
        if row["mesh_build_seconds"] > BUILD_TIME_BUDGET_SECONDS:
            print(
                f"wall: build-time budget at k={k}: {row['mesh_build_seconds']:.3f}s exceeds "
                f"{BUILD_TIME_BUDGET_SECONDS:.1f}s; last_completed_k={last['k']}"
            )
            return
        last = row
        k *= 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--k", type=int, default=1, help="visual subdivision (default: 1)")
    parser.add_argument("--no-detail", action="store_true", help="flat-surface control")
    parser.add_argument("--sweep", action="store_true", help="double k until the guarded wall")
    parser.add_argument("--sim-costs", action="store_true", help="print derived sim-grid wire costs")
    parser.add_argument("--snapshot", type=Path, help="existing exported snapshot (otherwise export one)")
    args = parser.parse_args()
    temporary = None
    try:
        if args.snapshot:
            snapshot_path = args.snapshot
            snapshot = _load_snapshot(snapshot_path)
        else:
            temporary, snapshot_path, snapshot = _export_snapshot()
        if args.sim_costs:
            raw = snapshot_path.read_text(encoding="utf-8")
            for sim_k in (1, 2, 4):
                print("sim " + " ".join(f"{key}={value}" for key, value in sim_axis_cost(raw, sim_k).items()))
        elif args.sweep:
            _sweep(snapshot)
        else:
            row = measure(snapshot, args.k, detail=not args.no_detail)
            if args.k == 1:
                assert_control(row)
            _print_row(row)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"resolution bench failed: {type(error).__name__}: {error}") from error
    finally:
        if temporary is not None:
            temporary.cleanup()


if __name__ == "__main__":
    main()
