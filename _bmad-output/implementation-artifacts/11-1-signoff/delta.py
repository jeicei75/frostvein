"""Magnitude, not just extent: how far do the changed pixels actually move?

A whole-pool +/-1 level shift and a pool that MOVED look identical to a blob census and
completely different here.

Usage: delta.py a.png b.png [x0 y0 x1 y1]
"""

import os, sys
from collections import Counter

sys.path.insert(0, "/workspace/projects/frostvein/_bmad-output/implementation-artifacts/10-7-signoff")
from lumstats import load  # noqa: E402


def luma(px, i):
    return (px[i] * 299 + px[i + 1] * 587 + px[i + 2] * 114) // 1000


a_path, b_path = sys.argv[1], sys.argv[2]
rect = [int(v) for v in sys.argv[3:7]] if len(sys.argv) > 6 else None

w, h, pa = load(a_path)
_, _, pb = load(b_path)
x0, y0, x1, y1 = rect if rect else (0, 0, w, h)

hist = Counter()
signed = Counter()
changed = 0
total = 0
la_sum = lb_sum = 0
for y in range(y0, y1):
    for x in range(x0, x1):
        i = (y * w + x) * 3
        la, lb = luma(pa, i), luma(pb, i)
        la_sum += la
        lb_sum += lb
        total += 1
        d = lb - la
        if pa[i] != pb[i] or pa[i + 1] != pb[i + 1] or pa[i + 2] != pb[i + 2]:
            changed += 1
            hist[abs(d)] += 1
            signed[max(-3, min(3, d))] += 1

print(f"{os.path.basename(a_path)} vs {os.path.basename(b_path)}  window {x0},{y0}..{x1},{y1}")
print(f"  pixels {total}, changed {changed} ({100.0*changed/total:.2f}%)")
print(f"  mean luma {la_sum/total:.4f} -> {lb_sum/total:.4f}  (delta {(lb_sum-la_sum)/total:+.4f})")
print("  |delta luma| over CHANGED pixels:")
for k in sorted(hist)[:10]:
    print(f"      {k:3d} : {hist[k]:7d}  ({100.0*hist[k]/changed:5.2f}%)")
big = sum(v for k, v in hist.items() if k > 9)
print(f"      >9  : {big:7d}  ({100.0*big/changed:5.2f}%)")
print("  signed delta (clamped +/-3):")
for k in sorted(signed):
    print(f"      {k:+d} : {signed[k]:7d}")
