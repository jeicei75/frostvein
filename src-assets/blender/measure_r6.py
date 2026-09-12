"""Round 6's measuring instrument: check 4 (silhouette step density), check 5 (the
bare neck in profile) and check 3 (visible skin), on the REFERENCE and on our own
renders with the same code path.

    uv run python src-assets/blender/measure_r6.py all

WHY IT EXISTS, AND WHY IT DOES NOT REPRODUCE CHECK 4'S PUBLISHED NUMBERS. Check 4
counts silhouette steps and divides by the figure's height IN THAT IMAGE'S OWN
PIXELS, so the figure is 700 rows tall in the 5x reference crop and 635 in round
5's render -- a metric that shrinks as the image grows. Worse, it was run on an
ANTIALIASED render, where a one-pixel edge wobble counts as a step: rerunning it on
round 5's own delivered PNGs gives L14/R75 on the front view at a tolerant
threshold, not the L13/R27 it published, so the published numbers cannot be
reproduced from the published renders at all.

What this does instead, and it is the only change: resample every silhouette to
EXACTLY 140 rows -- the sheet's own source-pixel height, where one row is 8.571 mm
-- before counting. That is scale-free, immune to antialiasing, and puts the
reference and our render on one axis. Read on this instrument the reference scores
60.0 steps/100 rows on the front view and 47.1 on the side, and round 5 scores 20.0
and 17.9, so round 5 carries a THIRD of the reference's detail rather than the 60 %
check 4 reports. The targets used this round are therefore the reference's own
numbers, not the absolute 10.0 in the brief, which this instrument clears trivially.
"""

import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.dirname(HERE)
REF_DIR = os.path.join(SRC, "references", "dwarf-ortho")
R6 = os.path.join(SRC, "renders", "r6")

REF_BG = np.array([227, 232, 234])
OUR_BG = np.array([0x6F, 0x70, 0x73])
PALETTE = ["#E9D2BB", "#5E4632", "#FFFFFF", "#5F7A6A", "#474B41", "#A9B2AC",
           "#8B6B50", "#6B5B49", "#34271C", "#F0A63C", "#BAA896", "#826145",
           "#7DA18C", "#44584C", "#63695B"]
SKIN_CELLS = [0, 10]                 # #E9D2BB base and #BAA896 shadow plane
CELLS = np.array([[int(h[i:i + 2], 16) for i in (1, 3, 5)] for h in PALETTE])


def ref_mask(name):
    """The crop is a 5x nearest-neighbour blow-up, so reduce to SOURCE pixels first.
    Rows 8..146 and columns from 10 drop the sheet's own 1-px dimension rules, which
    touch the figure at the crown row, the sole row and the left margin."""
    a = np.asarray(Image.open(os.path.join(REF_DIR, name + ".png")).convert("RGB")).astype(int)
    h, w = a.shape[0] // 5, a.shape[1] // 5
    src = np.median(a[:h * 5, :w * 5].reshape(h, 5, w, 5, 3), axis=(1, 3))
    m = np.abs(src - REF_BG).sum(2) > 90
    return m[8:147, (10 if name == "front" else 0):]


def our(path):
    a = np.asarray(Image.open(path).convert("RGB")).astype(int)
    m = np.abs(a - OUR_BG).sum(2) > 60
    ys = np.where(m.any(1))[0]
    return m[ys.min():ys.max() + 1], a, ys.min(), ys.max()


def norm140(mask):
    im = Image.fromarray((mask * 255).astype(np.uint8))
    w = max(1, round(im.width * 140 / im.height))
    return np.asarray(im.resize((w, 140), Image.BOX)) >= 128


def step_density(mask, label, target=None):
    m = norm140(mask)
    edges = []
    for row in m:
        xs = np.where(row)[0]
        edges.append((int(xs.min()), int(xs.max())) if xs.size else None)
    rows = [i for i, v in enumerate(edges) if v is not None]
    r0, r1 = rows[0], rows[-1]
    H = r1 - r0 + 1
    L = sum(1 for i in range(r0 + 1, r1 + 1)
            if edges[i] and edges[i - 1] and edges[i][0] != edges[i - 1][0])
    R = sum(1 for i in range(r0 + 1, r1 + 1)
            if edges[i] and edges[i - 1] and edges[i][1] != edges[i - 1][1])
    dens = 100.0 * (L + R) / H
    ratio = min(L, R) / max(L, R) if max(L, R) else 0.0
    verdict = ""
    if target is not None:
        verdict = "  %s" % ("PASS" if dens >= target and ratio >= 0.70 else "FAIL")
    print("%-26s H%4d  L%3d R%3d   steps/100 rows %5.1f   edge balance %.2f%s"
          % (label, H, L, R, dens, ratio, verdict))
    return dens, ratio


def skin_run(path):
    """Check 5: longest CONTIGUOUS run of rows carrying a skin pixel, by nearest cell."""
    mask, a, y0, y1 = our(path)
    H = mask.shape[0]
    best = run = 0
    at = None
    for r in range(H):
        xs = np.where(mask[r])[0]
        hit = False
        if xs.size:
            px = a[y0 + r, xs]
            d = np.abs(px[:, None, :] - CELLS[None, :, :]).sum(2)
            hit = bool(np.isin(d.argmin(1), SKIN_CELLS).any())
        run = run + 1 if hit else 0
        if run > best:
            best, at = run, r - run + 1
    print("%-26s H%4d  longest skin run %3d rows = %.3f H at rows %s..%s   %s"
          % (os.path.basename(path), H, best, best / H, at,
             None if at is None else at + best - 1,
             "PASS" if best / H >= 0.08 else "FAIL"))
    return best / H


def skin_share(path):
    """Check 3: visible skin as a share of figure pixels, BY NEAREST PALETTE CELL --
    the flat pass is antialiased and an exact match counts only each region's core."""
    mask, a, y0, y1 = our(path)
    px = a[y0:y1 + 1][mask]
    d = np.abs(px[:, None, :] - CELLS[None, :, :]).sum(2)
    share = float(np.isin(d.argmin(1), SKIN_CELLS).mean())
    print("%-26s figure px %6d   skin %5.1f %%   %s"
          % (os.path.basename(path), px.shape[0], 100 * share,
             "PASS" if share >= 0.10 else "FAIL"))
    return share


def band_shares(path):
    """The tunic's chest coverage, ten bands of height, by nearest palette cell."""
    mask, a, y0, y1 = our(path)
    H = mask.shape[0]
    tunic = {3, 12, 13}
    brown = {1, 6, 7, 8, 11}
    wins = 0
    print("  band      tunic   brown")
    for b in range(10):
        s, e = y0 + b * H // 10, y0 + (b + 1) * H // 10
        sub = mask[b * H // 10:(b + 1) * H // 10]
        px = a[s:e][sub]
        if not px.size:
            continue
        idx = np.abs(px[:, None, :] - CELLS[None, :, :]).sum(2).argmin(1)
        t = float(np.isin(idx, list(tunic)).mean())
        br = float(np.isin(idx, list(brown)).mean())
        wins += t > br
        print("  %3d-%3d %%  %5.1f   %5.1f" % (b * 10, (b + 1) * 10, 100 * t, 100 * br))
    print("  tunic wins %d of 10 bands   %s" % (wins, "PASS" if wins >= 4 else "FAIL"))
    return wins


def main():
    print("== check 4: silhouette step density, every silhouette resampled to 140 rows ==")
    ref = {}
    for n in ("front", "side-left", "side-right", "back"):
        ref[n] = step_density(ref_mask(n), "REFERENCE " + n)
    print()
    for n in ("front", "side-left", "side-right", "back"):
        step_density(our(os.path.join(R6, "dwarf-flat-%s.png" % n))[0],
                     "r6 " + n, target=ref[n][0])
    print()
    print("== check 5: a bare neck reads in profile, both side views ==")
    for n in ("side-left", "side-right"):
        skin_run(os.path.join(R6, "dwarf-flat-%s.png" % n))
    print()
    print("== check 3: visible skin >= 10 %% of figure pixels, by nearest cell ==")
    for n in ("front", "side-left", "three-quarter"):
        skin_share(os.path.join(R6, "dwarf-flat-%s.png" % n))
    print()
    print("== check 2: the tunic owns at least four of the ten height bands ==")
    band_shares(os.path.join(R6, "dwarf-flat-front.png"))


if __name__ == "__main__":
    main()
