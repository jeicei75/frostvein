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
SIDE_DELTAS = ((-1, 0), (1, 0), (0, -1), (0, 1))
# Guard the process before Python object overhead can exhaust the devpod.  This is a benchmark
# limit, not a renderer limit -- `gui --subdiv N` is the authority on what can actually be drawn.
# Sized from a measurement, not a guess: k=16 on the real world holds 23,014,708 faces in 4,298
# MiB, so one face costs ~196 bytes and 48M faces is ~9.0 GiB. The devpod has ~17 GiB free.
# It was 4,000,000, which reported a WALL AT k=8 that was an artefact of the old implementation:
# k=8 completes in 7.6s and k=16 in 30.4s.
MAX_FINE_FACES = 48_000_000
BUILD_TIME_BUDGET_SECONDS = 120.0
CHUNK_EDGE_CELLS = 16
FOLIAGE_MATERIAL = "tree_foliage"
MAX_SNAPSHOT_BYTES = 64 * 1024 * 1024


def detail_offset(seed, x, y, z):
    """Return a deterministic exposed-surface displacement in fine voxels.

    // NOTE: This is a measurement stand-in for 10.4's authored terrain look, not a visual
    decision.  The small value-noise displacement deliberately breaks flat greedy runs.

    Every step is masked to 32 bits because the client writes the same rule in u32
    `wrapping_mul`.  Python integers are unbounded, so leaving the multiplies unmasked made
    the two sides a DIFFERENT rule that agreed only at chance for k > 1 -- invisible at k=1,
    where the depth clamp forces both to zero.  `scripts/tests/test_resolution_bench.py` and
    `crates/gui/src/project.rs` pin the same vector so the claim is tested, not commented.
    """
    mask = 0xFFFFFFFF
    value = seed
    value ^= (x & mask) * 0x9E3779B1 & mask
    value ^= (y & mask) * 0x85EBCA77 & mask
    value ^= (z & mask) * 0xC2B2AE3D & mask
    value ^= value >> 16
    value = value * 0x7FEB352D & mask
    value ^= value >> 15
    return value % 5 - 2


def detail_depth(seed, x, y, z, k):
    """Return the depth of one closed top-surface pit, bounded by its fine cell height."""
    return min(abs(detail_offset(seed, x, y, z)), k - 1)


def _dims(snapshot):
    dims = snapshot["dims"]
    return dims["x"], dims["y"], dims["z"]


def _material(tile):
    if isinstance(tile, dict):
        return tile.get("solid", tile.get("ramp"))
    return None


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


def _grid(snapshot):
    """Flatten the wire tiles once into (dims, per-cell material).

    The geometry pass touches every cell six to twelve times; re-decoding the wire tile dict
    each time is what made the coarse-only version the whole runtime.
    """
    dx, dy, dz = _dims(snapshot)
    return dx, dy, dz, [_material(tile) for tile in snapshot["tiles"]]


def _coarse_faces(snapshot):
    """Count exposed COARSE cell faces -- the k=1 surface, and the pre-allocation estimate."""
    dx, dy, dz = _dims(snapshot)
    _, _, _, materials = _grid(snapshot)
    total = 0
    for z in range(dz):
        for y in range(dy):
            for x in range(dx):
                if materials[x + y * dx + z * dx * dy] is None:
                    continue
                for axis, sign in NEIGHBOURS:
                    neighbour = [x, y, z]
                    neighbour[axis] += sign
                    nx, ny, nz = neighbour
                    if not (0 <= nx < dx and 0 <= ny < dy and 0 <= nz < dz):
                        total += 1
                    elif materials[nx + ny * dx + nz * dx * dy] is None:
                        total += 1
    return total


def _cell_heights(x, y, z, k, carved, lattice=1):
    """Fine column heights inside one solid coarse cell, in fine voxels.

    A cell whose top is exposed carries the detail pits and is a heightfield; every other
    solid cell is a full k-cube.  `None` means "uniform k" and is what keeps the flat control
    from allocating k² identical columns per cell.
    """
    if not carved:
        return None
    plane = (z + 1) * k
    # `lattice` samples the SAME rule on a coarser grid, so blocks of `lattice` fine columns share
    # a depth. It is a measurement knob, not a second rule: lattice=1 is the shipped stand-in and
    # every committed figure uses it. It exists because the budget turns out to be almost entirely
    # a function of this one property -- see "how much of the budget is the placeholder" in
    # 10-6-signoff/axis-a-geometry.md.
    def at(u, v):
        return k - detail_depth(
            WORLD_SEED, plane, u // lattice * lattice, v // lattice * lattice, k
        )

    return [[at(x * k + i, y * k + j) for j in range(k)] for i in range(k)]


def geometry_summary(snapshot, k=1, detail=True, foliage_as_cubes=False, detail_lattice=1):
    """Measure exposed fine faces and greedy quads for a snapshot.

    Every reported face is EMITTED, never derived.  The earlier version multiplied the coarse
    face count by k², which is wrong the moment a pit exists: carving the top of a cell also
    removes the top fine voxels of that cell's SIDE faces, and exposes new faces on any solid
    neighbour the carve uncovered.  A cell is modelled as a k×k heightfield of fine columns,
    and a face is emitted wherever a solid fine voxel meets a non-solid one -- across the cell
    boundary as readily as inside it.  At k=1 every column is 1 tall and the model collapses
    to the shipped per-cube surface, which is why the k=1 control could not see the defect.
    """
    if not isinstance(k, int) or k < 1:
        raise ValueError("k must be a positive integer")
    if not isinstance(detail_lattice, int) or detail_lattice < 1:
        raise ValueError("detail_lattice must be a positive integer")
    assert_workload_limit(_coarse_faces(snapshot) * k * k, k, detail)
    # `gui --subdiv N` keeps tree foliage on the shipped one-cube-per-cell path, because a
    # greedy cuboid does not preserve a sparse crown silhouette. Foliage still OCCLUDES either
    # way; it just contributes no chunk geometry. Without this the offline and live triangle
    # counts cannot be compared at all, which is how a 53% gap went unexplained.
    dx, dy, dz, materials = _grid(snapshot)
    detailed = detail and k > 1

    def material_at(x, y, z):
        if not (0 <= x < dx and 0 <= y < dy and 0 <= z < dz):
            return None
        return materials[x + y * dx + z * dx * dy]

    def carved_at(x, y, z):
        # Foliage on the client's cube path is drawn whole, so it carries no pit and uncovers
        # no neighbour. Modelling it as carved here made the two sides disagree by 9 enclosed
        # tree trunks and 5 chunks.
        return (
            detailed
            and material_at(x, y, z) is not None
            and material_at(x, y, z + 1) is None
            and not (foliage_as_cubes and material_at(x, y, z) == FOLIAGE_MATERIAL)
        )

    heights_cache = {}

    def heights_at(x, y, z):
        key = (x, y, z)
        if key not in heights_cache:
            heights_cache[key] = _cell_heights(x, y, z, k, carved_at(x, y, z), detail_lattice)
        return heights_cache[key]

    masks = collections.defaultdict(dict)
    chunks = set()
    meshed_cells = 0
    for z in range(dz):
        for y in range(dy):
            for x in range(dx):
                material = material_at(x, y, z)
                if material is None:
                    continue
                if foliage_as_cubes and material == FOLIAGE_MATERIAL:
                    continue
                above = material_at(x, y, z + 1) is None
                below = material_at(x, y, z - 1) is None
                sides = [material_at(x + sx, y + sy, z) is None for sx, sy in SIDE_DELTAS]
                if not (above or below or any(sides)):
                    # A cell buried at the COARSE scale still gains faces if a neighbour's pit
                    # uncovered it. Everything else is interior rock and emits nothing.
                    if not detailed or not any(
                        carved_at(x + sx, y + sy, z) for sx, sy in SIDE_DELTAS
                    ):
                        continue
                own = heights_at(x, y, z)
                wrote = False

                if above:
                    for i in range(k):
                        for j in range(k):
                            plane = z * k + (k if own is None else own[i][j])
                            masks[(2, 1, plane)][(x * k + i, y * k + j)] = material
                    wrote = True
                if below:
                    plane = z * k
                    for i in range(k):
                        for j in range(k):
                            masks[(2, -1, plane)][(x * k + i, y * k + j)] = material
                    wrote = True

                for (sx, sy), open_side in zip(SIDE_DELTAS, sides):
                    axis = 0 if sx else 1
                    sign = sx or sy
                    other = None if open_side else heights_at(x + sx, y + sy, z)
                    near = k - 1 if sign > 0 else 0
                    far = 0 if sign > 0 else k - 1
                    plane = ((x if axis == 0 else y) + (1 if sign > 0 else 0)) * k
                    mask = masks[(axis, sign, plane)]
                    for step in range(k):
                        i, j = (near, step) if axis == 0 else (step, near)
                        top = k if own is None else own[i][j]
                        if open_side:
                            floor = 0
                        else:
                            oi, oj = (far, step) if axis == 0 else (step, far)
                            floor = k if other is None else other[oi][oj]
                        if floor >= top:
                            continue
                        for level in range(floor, top):
                            if axis == 0:
                                mask[(y * k + step, z * k + level)] = material
                            else:
                                mask[(z * k + level, x * k + step)] = material
                        wrote = True

                if own is not None:
                    for i in range(k):
                        for j in range(k):
                            for di, dj in ((1, 0), (0, 1)):
                                ni, nj = i + di, j + dj
                                if ni >= k or nj >= k:
                                    continue
                                top, neighbour = own[i][j], own[ni][nj]
                                if top == neighbour:
                                    continue
                                axis = 0 if di else 1
                                sign = 1 if top > neighbour else -1
                                lower, upper = sorted((top, neighbour))
                                plane = (x * k + i + 1) if di else (y * k + j + 1)
                                mask = masks[(axis, sign, plane)]
                                for level in range(lower, upper):
                                    if axis == 0:
                                        mask[(y * k + j, z * k + level)] = material
                                    else:
                                        mask[(z * k + level, x * k + i)] = material
                                wrote = True

                if wrote:
                    meshed_cells += 1
                    chunks.add(
                        (
                            x // CHUNK_EDGE_CELLS,
                            y // CHUNK_EDGE_CELLS,
                            z // CHUNK_EDGE_CELLS,
                        )
                    )
    exposed_faces = sum(len(mask) for mask in masks.values())
    quads = sum(_greedy_quads(mask) for mask in masks.values())
    return {
        "exposed_faces": exposed_faces,
        "greedy_quads": quads,
        "triangles": quads * 2,
        "chunks": len(chunks),
        "cells": meshed_cells,
    }


def per_class_census(snapshot):
    """Split the EXPOSED draw set by class: terrain, trees, and the trunk-column tree count.

    AC3a asks the bench to report what each class can afford separately, and it used to report
    nothing per class at all -- the sign-off table was hand arithmetic that subtracted a
    WHOLE-WORLD tree population (every buried canopy and trunk cell) from the EXPOSED cell set.
    Two different populations, so trees came out 534 cells too large. Everything here is counted
    over the same exposure rule `_coarse_faces` uses, and the totals are asserted against the
    AC4 control by `ResolutionRealWorldControlTests`, so the split cannot drift from the surface
    it is a split OF.
    """
    dx, dy, dz = _dims(snapshot)
    _, _, _, materials = _grid(snapshot)
    census = {
        "tree_cells": 0, "tree_faces": 0,
        "terrain_cells": 0, "terrain_faces": 0,
        "trees": 0,
    }
    trunk_columns = set()
    for z in range(dz):
        for y in range(dy):
            for x in range(dx):
                material = materials[x + y * dx + z * dx * dy]
                if material is None:
                    continue
                if material == "tree_trunk":
                    trunk_columns.add((x, y))
                exposed = 0
                for axis, sign in NEIGHBOURS:
                    neighbour = [x, y, z]
                    neighbour[axis] += sign
                    nx, ny, nz = neighbour
                    if not (0 <= nx < dx and 0 <= ny < dy and 0 <= nz < dz):
                        exposed += 1
                    elif materials[nx + ny * dx + nz * dx * dy] is None:
                        exposed += 1
                if not exposed:
                    continue
                kind = "tree" if str(material).startswith("tree_") else "terrain"
                census[f"{kind}_cells"] += 1
                census[f"{kind}_faces"] += exposed
    census["trees"] = len(trunk_columns)
    return census


def assert_control(summary):
    """Reject a sweep unless its k=1 geometry is the independently measured real-world oracle."""
    if summary.get("exposed_faces") != CONTROL_FACES or summary.get("greedy_quads") != CONTROL_QUADS:
        raise ValueError(
            f"k=1 control mismatch: exposed_faces={summary.get('exposed_faces')} "
            f"(expected {CONTROL_FACES}), greedy_quads={summary.get('greedy_quads')} "
            f"(expected {CONTROL_QUADS})"
        )


def assert_workload_limit(fine_faces, k, detail):
    # Pits add connector faces on top of the sampled surface and remove carved side faces.
    # Measured on the real world the detailed total is 1.44-1.47x the flat one at every k that
    # runs; reserve 2x before allocating any masks. The flat control is now genuinely meshed
    # rather than short-circuited, so it needs the same guard.
    estimated_faces = fine_faces * 2 if detail else fine_faces
    if estimated_faces > MAX_FINE_FACES:
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


def measure(snapshot, k, detail, foliage_as_cubes=False, detail_lattice=1):
    """Return one geometry row, including real elapsed time and process peak RSS."""
    started = time.perf_counter()
    summary = geometry_summary(
        snapshot,
        k=k,
        detail=detail,
        foliage_as_cubes=foliage_as_cubes,
        detail_lattice=detail_lattice,
    )
    elapsed = time.perf_counter() - started
    # Linux reports KiB, macOS bytes. The devpod is Linux; retain a usable POSIX fallback.
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    if sys.platform != "darwin":
        peak *= 1024
    return {
        "k": k,
        **summary,
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
        "k={k} cells={cells} exposed_faces={exposed_faces} greedy_quads={greedy_quads} "
        "triangles={triangles} chunks={chunks} mesh_build_seconds={mesh_build_seconds:.3f} "
        "peak_memory_bytes={peak_memory_bytes}".format(
            **row
        )
    )


def _measure_in_child(snapshot_path, k, args):
    """Measure one row in a FRESH process, so its peak RSS is that row's own.

    `getrusage` reports the high-water mark for the LIFE of a process. Measuring every k in one
    process made each row carry every earlier k's peak, and made k=1's number the export/parse
    baseline rather than anything about k=1. AC3 asks for peak memory at each step.

    The measurement flags are FORWARDED. They used not to be, so `--sweep --no-detail`,
    `--sweep --client-parity` and `--sweep --detail-lattice N` each silently produced the plain
    default sweep -- the project's own "accepted and discarded" defect class, landing on the
    story's own numbers. `_sweep_forwarded_flags` is the single list, so a new measurement flag
    cannot be added to `measure()` and forgotten here.
    """
    argv = [sys.executable, str(Path(__file__).resolve()), "--k", str(k), "--json",
            "--snapshot", str(snapshot_path)]
    argv += _sweep_forwarded_flags(args)
    result = subprocess.run(argv, capture_output=True, text=True, check=True)
    return json.loads(result.stdout)


def _sweep_forwarded_flags(args):
    """Every flag that changes what `measure()` measures, as child argv."""
    flags = []
    if args.no_detail:
        flags.append("--no-detail")
    if args.client_parity:
        flags.append("--client-parity")
    if args.detail_lattice != 1:
        flags += ["--detail-lattice", str(args.detail_lattice)]
    return flags


def _sweep(snapshot, snapshot_path, args):
    emit = (lambda row: print(json.dumps(row))) if args.json else _print_row
    control = _measure_in_child(snapshot_path, 1, args)
    # The control values describe the DEFAULT measurement. A sweep asked for a different one
    # (flat control, client parity, a coarser lattice) is not a k=1 control run and must not be
    # graded against 61,142/19,264 -- but it must still say so rather than quietly skip.
    if not (args.no_detail or args.client_parity or args.detail_lattice != 1):
        assert_control(control)
    else:
        print(
            "note: sweep run with "
            + " ".join(_sweep_forwarded_flags(args))
            + " -- k=1 control assertion not applicable to this configuration"
        )
    emit(control)
    last = control
    k = 2
    while True:
        estimated_faces = control["exposed_faces"] * k * k * 2
        if estimated_faces > MAX_FINE_FACES:
            print(
                f"wall: hard face limit at k={k}: up to {estimated_faces} detailed faces exceeds "
                f"{MAX_FINE_FACES}; last_completed_k={last['k']}"
            )
            return
        row = _measure_in_child(snapshot_path, k, args)
        emit(row)
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
    parser.add_argument(
        "--detail-lattice",
        type=int,
        default=1,
        help="sample the detail rule every N fine columns (1 = the shipped stand-in)",
    )
    parser.add_argument(
        "--client-parity",
        action="store_true",
        help="leave tree foliage to the client's cube path, so the row compares to gui --subdiv N",
    )
    parser.add_argument("--sweep", action="store_true", help="double k until the guarded wall")
    parser.add_argument("--json", action="store_true", help="emit one row as JSON (used by --sweep)")
    parser.add_argument("--sim-costs", action="store_true", help="print derived sim-grid wire costs")
    parser.add_argument("--per-class", action="store_true", help="split the exposed surface by class (AC3a)")
    parser.add_argument("--snapshot", type=Path, help="existing exported snapshot (otherwise export one)")
    args = parser.parse_args()
    temporary = None
    try:
        if args.snapshot:
            snapshot_path = args.snapshot
            snapshot = _load_snapshot(snapshot_path)
        else:
            temporary, snapshot_path, snapshot = _export_snapshot()
        # NOT elif: `--sweep --sim-costs` used to run --sim-costs ONLY and drop the sweep with
        # no error. Both are outputs, so both run.
        if args.sim_costs:
            raw = snapshot_path.read_text(encoding="utf-8")
            for sim_k in (1, 2, 4):
                print("sim " + " ".join(f"{key}={value}" for key, value in sim_axis_cost(raw, sim_k).items()))
        if args.per_class:
            census = per_class_census(snapshot)
            print("per-class " + " ".join(f"{key}={value}" for key, value in census.items()))
        if args.sweep:
            _sweep(snapshot, snapshot_path, args)
        elif not (args.sim_costs or args.per_class):
            row = measure(
                snapshot,
                args.k,
                detail=not args.no_detail,
                foliage_as_cubes=args.client_parity,
                detail_lattice=args.detail_lattice,
            )
            if (
                args.k == 1
                and not args.no_detail
                and not args.client_parity
                and args.detail_lattice == 1
            ):
                assert_control(row)
            if args.json:
                print(json.dumps(row))
            else:
                _print_row(row)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"resolution bench failed: {type(error).__name__}: {error}") from error
    finally:
        if temporary is not None:
            temporary.cleanup()


if __name__ == "__main__":
    main()
