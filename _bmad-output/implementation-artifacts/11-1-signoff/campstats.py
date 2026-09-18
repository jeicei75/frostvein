"""Camp-window Rec.601 statistics, matching what 11.1b's Task 0/Task 1 record.

Same integer Rec.601 luma as creases.py and pixel_guard.rs. Window is 11.1b's camp rect.

Usage: campstats.py a.png b.png ...
"""

import os
import sys

sys.path.insert(0, "/workspace/projects/frostvein/_bmad-output/implementation-artifacts/10-7-signoff")
from lumstats import load  # noqa: E402

CAMP = (500, 400, 760, 620)


def luma(px, i):
    return (px[i] * 299 + px[i + 1] * 587 + px[i + 2] * 114) // 1000


def stats(path):
    w, h, px = load(path)
    x0, y0, x1, y1 = CAMP
    vals = [luma(px, (y * w + x) * 3) for y in range(y0, y1) for x in range(x0, x1)]
    vals.sort()
    n = len(vals)
    near_white = sum(1 for v in vals if v >= 230)
    return {
        "median": vals[n // 2],
        "p90": vals[int(n * 0.90)],
        "p99": vals[int(n * 0.99)],
        "mean": sum(vals) / n,
        "nw": 100.0 * near_white / n,
    }


rows = [(os.path.basename(p), stats(p)) for p in sys.argv[1:]]
print(f"camp window {CAMP}, Rec.601 integer luma")
print(f"{'frame':<32} {'median':>7} {'p90':>6} {'p99':>6} {'mean':>10} {'near-white %':>13}")
for name, s in rows:
    print(f"{name:<32} {s['median']:>7} {s['p90']:>6} {s['p99']:>6} {s['mean']:>10.3f} {s['nw']:>13.4f}")

for key, label, fmt in (("median", "median", "{:.0f}"), ("p90", "p90", "{:.0f}"),
                        ("p99", "p99", "{:.0f}"), ("mean", "mean", "{:.3f}"),
                        ("nw", "near-white pp", "{:.4f}")):
    vals = [s[key] for _, s in rows]
    print(f"spread {label:<14} {fmt.format(max(vals) - min(vals))}")
