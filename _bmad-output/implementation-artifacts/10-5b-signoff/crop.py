"""Crop a window and upscale it, so a human can SEE what a count only asserts."""
import os, sys, zlib, struct
sys.path.insert(0, "_bmad-output/implementation-artifacts/10-7-signoff")
from pixel_diff import load

src, dst, x0, y0, x1, y1, zoom = sys.argv[1], sys.argv[2], *[int(v) for v in sys.argv[3:8]]
w, h, px = load(src)
cw, ch = (x1 - x0) * zoom, (y1 - y0) * zoom
rows = []
for y in range(ch):
    sy = y0 + y // zoom
    row = bytearray([0])
    for x in range(cw):
        sx = x0 + x // zoom
        i = (sy * w + sx) * 3
        row += bytes(px[i:i + 3])
    rows.append(bytes(row))
raw = zlib.compress(b"".join(rows), 9)
def chunk(tag, data):
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))
png = (b"\x89PNG\r\n\x1a\n"
       + chunk(b"IHDR", struct.pack(">IIBBBBB", cw, ch, 8, 2, 0, 0, 0))
       + chunk(b"IDAT", raw) + chunk(b"IEND", b""))
open(dst, "wb").write(png)
print(f"{dst}  {cw}x{ch}  from {x0},{y0}..{x1},{y1} at {zoom}x")
