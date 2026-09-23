"""RED proof for sharpness.py: a known blur must collapse the statistic.

Loads a capture, box-blurs the LUMA PLANE by radius r, and runs sharpness.py's own
`statistics()` on the result -- the shipped function, not a copy.  A blur is exactly
what depth of field does, so if `lap_mean` does not fall here the instrument cannot
read depth of field at all and no green from it means anything.

Usage: blur_proof.py capture.png
"""

import sys

from sharpness import WINDOWS, luma_plane, statistics


def box_blur(width, height, plane, radius):
    horizontal = [0] * (width * height)
    for y in range(height):
        row = y * width
        for x in range(width):
            lo, hi = max(0, x - radius), min(width - 1, x + radius)
            horizontal[row + x] = sum(plane[row + i] for i in range(lo, hi + 1)) // (hi - lo + 1)
    out = [0] * (width * height)
    for y in range(height):
        lo, hi = max(0, y - radius), min(height - 1, y + radius)
        for x in range(width):
            out[y * width + x] = sum(horizontal[i * width + x] for i in range(lo, hi + 1)) // (
                hi - lo + 1
            )
    return out


def main(path):
    width, height, plane = luma_plane(path)
    planes = [("sharp (r=0)", plane)]
    for radius in (1, 2, 4):
        planes.append((f"box blur r={radius}", box_blur(width, height, plane, radius)))
    for name, rect in WINDOWS.items():
        print(f"\n{name} {rect}")
        print(f"{'variant':<20} {'lap_mean':>10} {'lap_p90':>8} {'grad_mean':>10}")
        for label, p in planes:
            s = statistics(width, height, p, rect)
            print(f"{label:<20} {s['lap_mean']:>10.4f} {s['lap_p90']:>8} {s['grad_mean']:>10.4f}")


if __name__ == "__main__":
    main(sys.argv[1])
