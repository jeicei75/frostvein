"""Render one Task 2 sun-elevation candidate through the committed valley bench.

Run with Blender: ``blender --background --python sun_elevation_candidate.py -- SNAPSHOT OUT ELEVATION``.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "scripts" / "bench"))

import valley_bench


def main():
    args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    if len(args) != 3:
        raise SystemExit("usage: sun_elevation_candidate.py -- <snapshot.json> <out.png> <degrees>")
    snapshot, output, elevation = args
    valley_bench.SUN_ELEVATION_DEGREES = float(elevation)
    sys.argv = [sys.argv[0], "--", snapshot, output]
    valley_bench.main()


if __name__ == "__main__":
    main()
