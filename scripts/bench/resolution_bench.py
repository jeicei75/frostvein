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

    At k=1 the exported world is meshed exactly as shipped.  At larger k each exposed face is
    expanded to k² fine samples; detail moves samples between nearby slabs, preventing flat runs
    from merging.  No-detail is the control that must collapse back to k=1's quads.
    """
    if not isinstance(k, int) or k < 1:
        raise ValueError("k must be a positive integer")
    groups = _face_groups(snapshot)
    coarse_faces = sum(len(group) for group in groups.values())
    exposed_faces = coarse_faces * k * k
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
    for (axis, _sign, slab), faces in groups.items():
        masks = collections.defaultdict(dict)
        for base_u, base_v, material in faces:
            for du in range(k):
                for dv in range(k):
                    u = base_u * k + du
                    v = base_v * k + dv
                    displacement = 0
                    if detail and k > 1:
                        displacement = detail_offset(WORLD_SEED, axis * 1_000_003 + slab * k, u, v)
                    masks[slab * k + displacement][(u, v)] = material
        quads += sum(_greedy_quads(mask) for mask in masks.values())
    return {
        "exposed_faces": exposed_faces,
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
    return temporary, _load_snapshot(path)


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
        if estimated_faces > MAX_FINE_FACES:
            print(
                f"wall: hard face limit at k={k}: {estimated_faces} fine faces exceeds "
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
    parser.add_argument("--snapshot", type=Path, help="existing exported snapshot (otherwise export one)")
    args = parser.parse_args()
    temporary = None
    try:
        if args.snapshot:
            snapshot = _load_snapshot(args.snapshot)
        else:
            temporary, snapshot = _export_snapshot()
        if args.sweep:
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
