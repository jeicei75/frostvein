"""What moves between two same-build captures that already have the flicker pinned?

Diagnostic only. Loads via the committed stdlib PNG reader, diffs two frames, clusters the
changed pixels into 4-connected blobs, and reports where they are and how bright they are.
If the residual is falling snow, the blobs are many, small, scattered frame-wide, and bright.
If it were residual emitter flicker, they would be few, large, and sitting on the light pools.
"""

import os
import sys
from collections import deque

SIGNOFF = "/workspace/projects/frostvein/_bmad-output/implementation-artifacts/11-1-signoff"
sys.path.insert(0, os.path.join(SIGNOFF, "..", "10-7-signoff"))
from lumstats import load  # noqa: E402

CAMP = (500, 400, 760, 620)


def luma(px, i):
    return (px[i] * 299 + px[i + 1] * 587 + px[i + 2] * 114) // 1000


def diff_mask(a, b, width, height):
    """Indices of pixels differing in any channel."""
    changed = set()
    for y in range(height):
        row = y * width
        for x in range(width):
            i = (row + x) * 3
            if a[i] != b[i] or a[i + 1] != b[i + 1] or a[i + 2] != b[i + 2]:
                changed.add((x, y))
    return changed


def blobs(changed):
    """4-connected clusters."""
    seen = set()
    out = []
    for seed in changed:
        if seed in seen:
            continue
        q = deque([seed])
        seen.add(seed)
        cluster = []
        while q:
            x, y = q.popleft()
            cluster.append((x, y))
            for n in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                if n in changed and n not in seen:
                    seen.add(n)
                    q.append(n)
        out.append(cluster)
    return out


def report(pa, pb, width, height, label):
    changed = diff_mask(pa, pb, width, height)
    cl = blobs(changed)
    cl.sort(key=len, reverse=True)
    in_camp = [c for c in cl if any(CAMP[0] <= x < CAMP[2] and CAMP[1] <= y < CAMP[3] for x, y in c)]
    # brightness of changed pixels, whichever frame is brighter there
    brights = []
    for c in cl:
        for x, y in c:
            i = (y * width + x) * 3
            brights.append(max(luma(pa, i), luma(pb, i)))
    brights.sort()
    near_white = sum(1 for v in brights if v >= 230)
    print(f"--- {label} ---")
    print(f"  changed pixels     : {len(changed)}")
    print(f"  blobs              : {len(cl)}")
    print(f"  blobs touching camp: {len(in_camp)}")
    if cl:
        sizes = [len(c) for c in cl]
        print(f"  blob size max/med/min: {sizes[0]} / {sizes[len(sizes)//2]} / {sizes[-1]}")
        ys = [y for c in cl for _, y in c]
        xs = [x for c in cl for x, _ in c]
        print(f"  spread x {min(xs)}..{max(xs)}  y {min(ys)}..{max(ys)}  (frame {width}x{height})")
        print(f"  changed pixels >=230 luma: {near_white} ({100.0*near_white/len(brights):.2f}% of changed)")
        print(f"  changed-pixel luma p50/p90: {brights[len(brights)//2]} / {brights[int(len(brights)*0.9)]}")
        print("  10 largest blobs (size @ centroid):")
        for c in cl[:10]:
            cx = sum(x for x, _ in c) / len(c)
            cy = sum(y for _, y in c) / len(c)
            print(f"      {len(c):5d} @ ({cx:6.1f},{cy:6.1f})")


def main():
    frames = {}
    for tag in sys.argv[1:]:
        path, _, label = tag.partition("=")
        w, h, px = load(path)
        frames[label or path] = (w, h, px)
    labels = list(frames)
    for i in range(len(labels) - 1):
        (w, h, a) = frames[labels[i]]
        (_, _, b) = frames[labels[i + 1]]
        report(a, b, w, h, f"{labels[i]} vs {labels[i+1]}")


main()
