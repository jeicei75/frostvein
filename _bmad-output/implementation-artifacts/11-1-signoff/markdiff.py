"""Crop a window from frame A and paint the pixels that differ from frame B in red.

Diagnostic. `counts are winding-blind` — a blob census says how many and how big, not WHAT.
This draws the changed pixels onto the image so the feature can be named by looking at it.

Usage: markdiff.py a.png b.png out.png x0 y0 x1 y1 zoom
"""

import os, struct, sys, zlib

sys.path.insert(0, "/workspace/projects/frostvein/_bmad-output/implementation-artifacts/10-7-signoff")
from lumstats import load  # noqa: E402

a_path, b_path, dst = sys.argv[1], sys.argv[2], sys.argv[3]
x0, y0, x1, y1, zoom = [int(v) for v in sys.argv[4:9]]

w, h, pa = load(a_path)
_, _, pb = load(b_path)

cw, ch = (x1 - x0) * zoom, (y1 - y0) * zoom
rows = []
changed_total = 0
for y in range(ch):
    sy = y0 + y // zoom
    row = bytearray([0])
    for x in range(cw):
        sx = x0 + x // zoom
        i = (sy * w + sx) * 3
        if pa[i] != pb[i] or pa[i + 1] != pb[i + 1] or pa[i + 2] != pb[i + 2]:
            row += bytes((255, 0, 0))
            changed_total += 1
        else:
            row += bytes(pa[i:i + 3])
    rows.append(bytes(row))

raw = zlib.compress(b"".join(rows), 9)


def chunk(tag, data):
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))


png = (b"\x89PNG\r\n\x1a\n"
       + chunk(b"IHDR", struct.pack(">IIBBBBB", cw, ch, 8, 2, 0, 0, 0))
       + chunk(b"IDAT", raw) + chunk(b"IEND", b""))
open(dst, "wb").write(png)
print(f"{dst}  {cw}x{ch}  window {x0},{y0}..{x1},{y1} at {zoom}x  "
      f"changed(sub-pixel-sampled)={changed_total}")
