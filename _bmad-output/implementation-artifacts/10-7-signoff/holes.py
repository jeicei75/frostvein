"""Interior-sky pixels: sky-coloured pixels INSIDE the terrain silhouette.

A hole is sky where terrain is drawn around it. Horizon sky is not a hole, which is why the
silhouette is resolved per COLUMN — the topmost non-sky pixel of that column — rather than by a
y-threshold. A y-threshold was tried first and it counted horizon sky whose edge shifts between
builds, which is how a 884-pixel regression first read as an improvement.

Usage: python3 holes.py a.png=label [b.png=label ...]
Reads RGB PNGs only (client captures); it refuses anything else rather than misparsing it.
"""

import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from lumstats import load  # noqa: E402  -- same loader, same colour-type guard

SKY = (5, 12, 28)


def interior_sky(path):
    width, height, pixels = load(path)
    count = 0
    for x in range(width):
        silhouette = None
        for y in range(height):
            i = (y * width + x) * 3
            is_sky = (pixels[i], pixels[i + 1], pixels[i + 2]) == SKY
            if silhouette is None:
                if not is_sky:
                    silhouette = y
            elif is_sky:
                count += 1
    return count


for argument in sys.argv[1:]:
    path, _, label = argument.partition("=")
    print(f"{label or path:<24} interior-sky px = {interior_sky(path):>7,}")
