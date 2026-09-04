"""Enclosed-sky pixels: sky-coloured pixels NOT reachable from the frame border through sky.

A hole is a TOPOLOGICAL fact -- sky with terrain drawn all the way around it -- so resolve it with
a flood fill from the border. Everything the fill reaches is open sky, however deep in the frame it
looks; everything it cannot reach is a hole.

WHY THIS EXISTS, AND WHY `holes.py` IS NOT ENOUGH. `holes.py` counts, per column, the sky pixels
below that column's topmost non-sky pixel. That silhouette rule **never engages on a real capture**:
the night sky is a GRADIENT, so the top of every column is some other shade and the "silhouette" is
found at y<=19 in all 1,280 columns. What it then counts is simply open sky. Measured on
`head-sd1-a`: 18,889 exactly-SKY pixels in the frame, 7,715 above the silhouettes, 11,174 below --
and that 11,174 is the whole of its reading, against 1,650 pixels genuinely enclosed. So its number
is ~87% open sky. Its DELTA still tracks holes (437 px against this file's 425 on the same pair),
which is why it read as working; but a delta can only ever say "some closed", never "none left", and
AC12 asks for gone. 10.7 read a delta as a level and shipped 54 holes under a green guard.

RED-first, on `head-sd1-a.png` (baseline 1,650 px / 15 blobs):
  punch a 20x20 SKY square into solid terrain -> 2,050 px / 16 blobs   (+400 exactly, +1 blob)
  add a 4x4 "star" to the open sky           -> 1,650 px / 15 blobs   (unmoved, as it must be)
Same-build noise floor: 0 px. Two captures of one binary gave 2,177 / 2,177 at --subdiv 2 and
1,650 / 1,650 at --subdiv 1 -- exact agreement, against holes.py's 45 px spread over eight readings.

Usage: python3 enclosed.py a.png=label [b.png=label ...]   (add --regions for a per-blob list)
"""

import sys
from collections import deque

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from lumstats import load  # noqa: E402  -- same loader, same colour-type guard

SKY = (5, 12, 28)


def analyse(path):
    """Total enclosed-sky pixels, and every blob as (px, x0, y0, x1, y1), largest first."""
    width, height, pixels = load(path)
    sky = bytearray(width * height)
    for i in range(width * height):
        j = i * 3
        if (pixels[j], pixels[j + 1], pixels[j + 2]) == SKY:
            sky[i] = 1

    seen = bytearray(width * height)
    queue = deque()
    for x in range(width):
        for y in (0, height - 1):
            i = y * width + x
            if sky[i] and not seen[i]:
                seen[i] = 1
                queue.append(i)
    for y in range(height):
        for x in (0, width - 1):
            i = y * width + x
            if sky[i] and not seen[i]:
                seen[i] = 1
                queue.append(i)
    while queue:
        i = queue.popleft()
        x, y = i % width, i // width
        for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
            if 0 <= nx < width and 0 <= ny < height:
                n = ny * width + nx
                if sky[n] and not seen[n]:
                    seen[n] = 1
                    queue.append(n)

    regions = []
    for i in range(width * height):
        if sky[i] and not seen[i]:
            seen[i] = 1
            blob, pending = [i], deque([i])
            while pending:
                k = pending.popleft()
                x, y = k % width, k // width
                for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                    if 0 <= nx < width and 0 <= ny < height:
                        n = ny * width + nx
                        if sky[n] and not seen[n]:
                            seen[n] = 1
                            blob.append(n)
                            pending.append(n)
            xs = [k % width for k in blob]
            ys = [k // width for k in blob]
            regions.append((len(blob), min(xs), min(ys), max(xs), max(ys)))
    regions.sort(reverse=True)
    return sum(r[0] for r in regions), regions


if __name__ == "__main__":
    show = "--regions" in sys.argv
    for argument in sys.argv[1:]:
        if argument.startswith("--"):
            continue
        path, _, label = argument.partition("=")
        total, regions = analyse(path)
        print(f"{label or path:<28} enclosed-sky px = {total:>6,}   blobs = {len(regions):>4}")
        if show:
            for n, x0, y0, x1, y1 in regions[:20]:
                print(f"    {n:>5} px  x {x0:>4}-{x1:<4} y {y0:>4}-{y1:<4}")
