"""Windowed pixel diff, for story 10.5's AC2.

    python3 window_diff.py <a.png> <b.png> <label> <x0> <y0> <x1> <y1>

WHY A WINDOW AND NOT THE WHOLE FRAME. 10.5's authoring run measured the same-build, whole-frame
noise on an UNCHANGED binary at raw=64,851 / >=4=24,243 / >=16=8,982 -- the snow is animated, so
tens of thousands of pixels differ frame to frame with no code change at all. A whole-frame 10x
bar is therefore unreachable by five dwarf-sized silhouettes, and it fails in the direction that
looks like the feature is broken. This is 10.7's AC11 lesson arriving one story later.

The PNG decoder is imported from `10-7-signoff/pixel_diff.py` rather than copied: it is the same
reader, already exercised, and it raises on a colour type it cannot handle instead of returning
plausible numbers.
"""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "10-7-signoff"))
from pixel_diff import load  # noqa: E402


def main():
    if len(sys.argv) != 8:
        raise SystemExit(__doc__.strip().splitlines()[2].strip())
    a_path, b_path, label = sys.argv[1:4]
    x0, y0, x1, y1 = (int(v) for v in sys.argv[4:8])
    w, h, a = load(a_path)
    w2, h2, b = load(b_path)
    if (w, h) != (w2, h2):
        raise SystemExit(f"frames differ in size: {w}x{h} vs {w2}x{h2}")
    if not (0 <= x0 < x1 <= w and 0 <= y0 < y1 <= h):
        raise SystemExit(f"window {x0},{y0}..{x1},{y1} is outside a {w}x{h} frame")

    raw = d4 = d16 = 0
    for y in range(y0, y1):
        row = y * w * 3
        for x in range(x0, x1):
            i = row + x * 3
            m = max(abs(a[i] - b[i]), abs(a[i + 1] - b[i + 1]), abs(a[i + 2] - b[i + 2]))
            if m:
                raw += 1
            if m >= 4:
                d4 += 1
            if m >= 16:
                d16 += 1
    area = (x1 - x0) * (y1 - y0)
    print(
        f"{label:<26} window={x0},{y0}..{x1},{y1} ({area:,} px)  "
        f"raw={raw:>6,}  >=4={d4:>6,}  >=16={d16:>6,}"
    )


if __name__ == "__main__":
    main()
