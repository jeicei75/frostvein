"""Pinned-window Rec.601 luminance instrument for story 11.1a.

Usage:
    python3 creases.py capture-a.png=control-a [capture-b.png=control-b ...]

The output uses integer Rec.601 luma, `(r*299 + g*587 + b*114)//1000`, matching
`10-7-signoff/lumstats.py` and `crates/gui/tests/pixel_guard.rs`.  It deliberately does
not use `capture.rs`'s Rec.709 statistic.
"""

import os
import sys

sys.path.insert(
    0,
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "10-7-signoff"),
)
from lumstats import load  # noqa: E402 -- path is set above


# Re-derived against the 11.1a boot control at 1280x720.  Each straddles terrain creases but
# excludes the camp: its flickering light moved the prototype's mean by 1.19 on a same-build pair.
WINDOWS = {
    "terrace-creases": (860, 190, 1060, 290),
    "open-snow-LL": (180, 620, 380, 700),
    "open-snow-LR": (950, 590, 1150, 670),
}


def luma(pixels, index):
    return (pixels[index] * 299 + pixels[index + 1] * 587 + pixels[index + 2] * 114) // 1000


def statistics(path, rect):
    width, height, pixels = load(path)
    x0, y0, x1, y1 = rect
    if not (0 <= x0 < x1 <= width and 0 <= y0 < y1 <= height):
        raise ValueError(f"window {rect} is outside {width}x{height}: {path}")
    values = [
        luma(pixels, (y * width + x) * 3)
        for y in range(y0, y1)
        for x in range(x0, x1)
    ]
    values.sort()
    count = len(values)
    return {
        "n": count,
        "p10": values[count // 10],
        "median": values[count // 2],
        "p90": values[9 * count // 10],
        "mean": sum(values) / count,
    }


def parse_capture(argument):
    path, separator, label = argument.partition("=")
    if not path:
        raise ValueError("a capture path is required")
    return path, label if separator and label else path


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__.strip())
    captures = [parse_capture(argument) for argument in sys.argv[1:]]
    results = {}
    for path, label in captures:
        results[label] = {}
        for name, rect in WINDOWS.items():
            result = statistics(path, rect)
            results[label][name] = result
            print(
                f"{label:<14} {name:<17} rect={','.join(map(str, rect))} "
                f"n={result['n']:>5} p10={result['p10']:>3} median={result['median']:>3} "
                f"p90={result['p90']:>3} mean={result['mean']:7.3f} Rec.601"
            )
    if len(captures) == 2:
        left, right = (label for _, label in captures)
        print("deltas (right - left):")
        for name in WINDOWS:
            a, b = results[left][name], results[right][name]
            print(
                f"  {name:<17} p10={b['p10'] - a['p10']:+d} "
                f"median={b['median'] - a['median']:+d} mean={b['mean'] - a['mean']:+.3f}"
            )


if __name__ == "__main__":
    main()
