"""Pinned-window EDGE-CONTRAST instrument for story 11.2 (depth of field).

Usage:
    python3 sharpness.py capture-a.png[=label] [capture-b.png[=label] ...]

Depth of field removes HIGH-FREQUENCY DETAIL; it barely moves a window's luminance
level.  So none of 11.1's statistics (p10/median/p90/mean/near-white area) can read it:
they are all level statistics and a blur preserves the mean almost exactly.  This
instrument measures the LOCAL CONTRAST instead.

Statistic: the 4-neighbour Laplacian of integer Rec.601 luma,
    lap(x, y) = |4*L(x,y) - L(x-1,y) - L(x+1,y) - L(x,y-1) - L(x,y+1)|
reported as `lap_mean` and `lap_p90` over the window's interior.  `grad_mean` is the
mean of |dL/dx| + |dL/dy| (a cheaper, lower-order sibling kept as a cross-check; a
real change must move BOTH).

Rec.601 integer luma, `(r*299 + g*587 + b*114)//1000` -- the same statistic as
`creases.py`, `campstats.py` and `crates/gui/tests/pixel_guard.rs`, and deliberately
NOT `capture.rs`'s Rec.709.
"""

import os
import sys

sys.path.insert(
    0,
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "10-7-signoff"),
)
from lumstats import load  # noqa: E402 -- path is set above


# Derived from the boot control on 7442174 at 1280x720 by eye, then range-checked:
# every window must carry a non-trivial baseline `lap_mean` or it cannot LOSE any.
WINDOWS = {
    # The far world edge: rock silhouette and snow-capped pines against the aurora.
    # This is the window depth of field must SOFTEN.
    "far-ridge": (450, 120, 900, 250),
    # The camp terraces at the focal plane. This is the window it must LEAVE ALONE.
    "camp-focus": (500, 400, 760, 620),
    # The near foreground below the camp, in front of the focal plane. Recorded, not
    # asserted: a physical lens softens this too, and the figure says by how much.
    "near-foreground": (140, 620, 640, 716),
    # Stars against the night sky, well above the skyline. Depth of field must NOT smear these:
    # `max_depth` is the only thing that stops it, and the default is `f32::INFINITY`.
    "sky-stars": (60, 10, 460, 110),
}


def luma_plane(path):
    width, height, pixels = load(path)
    plane = [
        (pixels[i] * 299 + pixels[i + 1] * 587 + pixels[i + 2] * 114) // 1000
        for i in range(0, width * height * 3, 3)
    ]
    return width, height, plane


def statistics(width, height, plane, rect):
    x0, y0, x1, y1 = rect
    if not (0 <= x0 < x1 <= width and 0 <= y0 < y1 <= height):
        raise ValueError(f"window {rect} is outside {width}x{height}")
    laps, grads = [], []
    for y in range(max(y0, 1), min(y1, height - 1)):
        row = y * width
        for x in range(max(x0, 1), min(x1, width - 1)):
            c = plane[row + x]
            left, right = plane[row + x - 1], plane[row + x + 1]
            up, down = plane[row - width + x], plane[row + width + x]
            laps.append(abs(4 * c - left - right - up - down))
            grads.append(abs(right - left) + abs(down - up))
    laps.sort()
    n = len(laps)
    return {
        "lap_mean": sum(laps) / n,
        "lap_p90": laps[int(n * 0.90)],
        "grad_mean": sum(grads) / n,
        "pixels": n,
    }


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    rows = []
    for spec in argv:
        path, _, label = spec.partition("=")
        width, height, plane = luma_plane(path)
        rows.append(
            (
                label or os.path.basename(path),
                {name: statistics(width, height, plane, rect) for name, rect in WINDOWS.items()},
            )
        )
    for name, rect in WINDOWS.items():
        print(f"\n{name} {rect}, Rec.601 integer luma")
        print(f"{'frame':<34} {'lap_mean':>10} {'lap_p90':>8} {'grad_mean':>10}")
        for label, per_window in rows:
            s = per_window[name]
            print(f"{label:<34} {s['lap_mean']:>10.4f} {s['lap_p90']:>8} {s['grad_mean']:>10.4f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
