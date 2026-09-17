"""Extraction: the figure's outline, row by row, out of the pinned ortho crops.

r15 s2 -- THE TABLE IS SAMPLES, THE ART IS THE AUTHORITY. Nothing in r16 is built
from the model sheet's twenty landmarks; PARAMS is derived from the table this
module prints. Run standalone:

    blender --background --factory-startup --python src-assets/blender/sheet_r16.py

r15 s2 names the two traps this must handle:

  * the crops carry FURNITURE -- a dimension arrow down front.png's left margin,
    arrows across the top of both, a baseline rule under side-left.png. The first
    cut of this module dropped runs thinner than 8 IMAGE px, but the crops are 5x
    blow-ups, so a 2-source-px arrow is 10 px wide and sailed through; it put the
    ink bbox at x=0 and threw every column off. Everything here therefore works at
    the sheet's OWN resolution (r15 s7.8), and the figure is the largest connected
    component rather than anything per-row;
  * the figure legitimately SPLITS INTO TWO at the legs, so "longest run" is wrong.
    Rows are read off the component, which spans both legs through the hips.

Props: r15 s3.2 keeps the pickaxe and lantern out of OUR silhouette but leaves them
in the sheet's, which only makes the gate more permissive, since the gate is
one-sided (where do WE stick out past the ART). r16 s6 also wants undershoot, and
for that the sheet's own body edge is needed -- so each row additionally reports the
run containing the body's centre column, which drops the pick head and the lantern.
Where the haft crosses the body it merges and the row is flagged.
"""

import os
from collections import deque

import bpy
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
REF = os.path.join(os.path.dirname(HERE), "references", "dwarf-ortho")

SCALE = 5            # crops are 5x nearest-neighbour blow-ups of the sheet's own pixels
H_ROWS = 140.0       # figure height in SOURCE rows; 1 source px = 1/140 H = 8.571 mm


def source_image(name):
    """The crop at the sheet's own resolution, raw (no sRGB decode)."""
    img = bpy.data.images.load(os.path.join(REF, name))
    img.colorspace_settings.name = "Non-Color"
    w, h = img.size
    buf = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(buf)
    a = buf.reshape(h, w, 4)[::-1, :, :3] * 255.0    # Blender hands back bottom-up
    bpy.data.images.remove(img)
    assert w % SCALE == 0 and h % SCALE == 0, f"{name} is not a clean {SCALE}x blow-up"
    return a[SCALE // 2::SCALE, SCALE // 2::SCALE]   # centre of each 5x5 block


def ink_mask(a):
    """Figure ink vs graph paper.

    The paper is light and COOL (blue >= red); every palette cell in the figure is
    either dark or warm. Skin #E9D2BB is the brightest thing on the sheet and clears
    the warmth test by r-b = 46; Metal #A9B2AC is cool but dark enough for luminance.
    """
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    lum = 0.299 * r + 0.587 * g + 0.114 * b
    return (lum < 200.0) | ((r - b) > 12.0)


def _shift_or(m, op):
    p = np.pad(m, 1, constant_values=False)
    out = p[1:-1, 1:-1].copy()
    for dy in (0, 1, 2):
        for dx in (0, 1, 2):
            out = op(out, p[dy:dy + m.shape[0], dx:dx + m.shape[1]])
    return out


def erode(m, n=1):
    for _ in range(n):
        m = _shift_or(m, np.logical_and)
    return m


def dilate_within(m, limit, n=1):
    for _ in range(n):
        m = _shift_or(m, np.logical_or) & limit
    return m


def components(mask):
    """4-connected labels; returns (labels, sizes) with label 0 = background."""
    h, w = mask.shape
    lab = np.zeros((h, w), np.int32)
    sizes = [0]
    nxt = 0
    for sy in range(h):
        for sx in range(w):
            if not mask[sy, sx] or lab[sy, sx]:
                continue
            nxt += 1
            n = 0
            q = deque([(sy, sx)])
            lab[sy, sx] = nxt
            while q:
                y, x = q.popleft()
                n += 1
                for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    ny, nx = y + dy, x + dx
                    if 0 <= ny < h and 0 <= nx < w and mask[ny, nx] and not lab[ny, nx]:
                        lab[ny, nx] = nxt
                        q.append((ny, nx))
            sizes.append(n)
    return lab, sizes


ERODE = 2        # source px; furniture lines and the haft are thinner than this


def figure_mask(ink):
    """The BODY, with the sheet's furniture and its props both gone.

    Erode by ERODE, keep the largest surviving blob, then dilate back within the ink
    by the same amount. The dimension arrows are 2-3 px wide and the pickaxe haft
    and the lantern hook about 5 px on the diagonal, so all three are cut by the
    erosion and cannot rejoin -- which is r15 s3.2's prop exclusion falling out of
    the same operation that removes the furniture, rather than a second rule. Full
    reconstruction is deliberately NOT used: it would flood back down the haft and
    drag the pick head in with it.
    """
    core = erode(ink, ERODE)
    lab, sizes = components(core)
    keep = int(np.argmax(sizes))
    return dilate_within(lab == keep, ink, ERODE)


def row_runs(row):
    idx = np.flatnonzero(row)
    if idx.size == 0:
        return []
    cuts = np.flatnonzero(np.diff(idx) > 1)
    starts = np.r_[idx[0], idx[cuts + 1]]
    stops = np.r_[idx[cuts], idx[-1]] + 1
    return list(zip(starts, stops))


def extract(name):
    """-> dict with the figure mask, its bbox, the centre column and the profiles."""
    a = source_image(name)
    ink = ink_mask(a)
    fig = figure_mask(ink)
    ys = np.flatnonzero(fig.any(axis=1))
    xs = np.flatnonzero(fig.any(axis=0))
    top, bot = int(ys[0]), int(ys[-1]) + 1
    # bootstrap the centre off the head, which is alone above the pick head
    head = [r for y in range(top, top + 18) for r in row_runs(fig[y])]
    wide = max(head, key=lambda r: r[1] - r[0])
    centre = (wide[0] + wide[1]) / 2.0

    n = bot - top
    full = np.full((n, 2), np.nan)     # whole sheet silhouette, props included
    body = np.full((n, 2), np.nan)     # the run through the centre column
    merged = np.zeros(n, bool)         # a prop run fused with the body on this row
    for i in range(n):
        rr = [r for r in row_runs(fig[top + i]) if r[1] - r[0] >= 4]
        if not rr:
            continue
        full[i] = [rr[0][0], rr[-1][1]]
        hit = [r for r in rr if r[0] <= centre < r[1]]
        if hit:
            body[i] = [hit[0][0], hit[0][1]]
            merged[i] = len(rr) > 1 and (hit[0][1] - hit[0][0]) > 0.75 * (rr[-1][1] - rr[0][0])
    return dict(name=name, img=a, fig=fig, top=top, bot=bot, n=n,
                centre=centre, x0=int(xs[0]), x1=int(xs[-1]) + 1,
                full=full, body=body, merged=merged)


def main():
    for name in ("front.png", "side-left.png"):
        e = extract(name)
        print(f"\n=== {name}: source {e['img'].shape[1]}x{e['img'].shape[0]} px, "
              f"figure rows {e['top']}..{e['bot']-1} ({e['n']} tall), "
              f"cols {e['x0']}..{e['x1']-1}, centre col {e['centre']:.1f}")
        print("    row |  full L    R  |  body L    R   w  | H from centre (body)")
        for i in range(e["n"]):
            r = e["top"] + i
            f, b = e["full"][i], e["body"][i]
            if np.isnan(b[0]):
                continue
            flag = " merged" if e["merged"][i] else ""
            print(f"    {r:4d} | {f[0]:5.0f} {f[1]:5.0f} | {b[0]:5.0f} {b[1]:5.0f} "
                  f"{b[1]-b[0]:4.0f} | {(b[0]-e['centre'])/H_ROWS:+.3f} "
                  f"{(b[1]-e['centre'])/H_ROWS:+.3f}{flag}")




def steps(name):
    """Collapse the per-row profile into the drawing's own STEP LIST.

    This is the form PARAMS is built from: a stacked-box figure wants the rows at
    which an edge moves, not 141 samples of it.
    """
    e = extract(name)
    out, cur = [], None
    for i in range(e["n"]):
        b = e["body"][i]
        key = (None, None) if np.isnan(b[0]) else (int(b[0]), int(b[1]))
        if cur is None or key != cur[2]:
            if cur is not None:
                out.append(cur)
            cur = [e["top"] + i, e["top"] + i, key]
        else:
            cur[1] = e["top"] + i
    out.append(cur)
    return e, out


def dump():
    for name in ("front.png", "side-left.png"):
        e, st = steps(name)
        c, top = e["centre"], e["top"]
        print(f"\n=== {name}  centre col {c:.1f}  rows {top}..{e['bot']-1}  "
              f"(1 src px = 1/140 H = 8.571 mm)")
        print("     rows      z/H top..bot     cols      L/H     R/H    w/H")
        for a, b, (l, r) in st:
            zt = (147 - a) / H_ROWS
            zb = (147 - b - 1) / H_ROWS
            if l is None:
                print(f"    {a:3d}-{b:3d}  {zt:.3f}..{zb:.3f}   (none)")
                continue
            print(f"    {a:3d}-{b:3d}  {zt:.3f}..{zb:.3f}  {l:3d}-{r:3d}  "
                  f"{(l-c)/H_ROWS:+.3f}  {(r-c)/H_ROWS:+.3f}  {(r-l)/H_ROWS:.3f}")




def symmetric(name="front.png"):
    """Front body half-width per row, symmetrised about the centre (r15 s3.1).

    Symmetrising is in the brief because the art is hand-drawn asymmetric and
    comparing raw charges us for its wobble. Taking the NARROWER half does a second
    job for free: the pick head, the haft and the lantern all widen exactly one side
    at any given row, so min() drops all three and leaves the body. It also fixes
    the legs, where the run containing the centre column does not exist at all.
    """
    e = extract(name)
    c = e["centre"]
    half = np.full(e["n"], np.nan)
    for i in range(e["n"]):
        rr = [r for r in row_runs(e["fig"][e["top"] + i]) if r[1] - r[0] >= 4]
        if not rr:
            continue
        half[i] = min(c - rr[0][0], rr[-1][1] - c)
    return e, half


def legs():
    e = extract("front.png")
    print("\n=== front leg rows: every run, to place the gap between the boots")
    for i in range(e["n"]):
        r = e["top"] + i
        if r < 116:
            continue
        rr = [x for x in row_runs(e["fig"][i + e["top"]]) if x[1] - x[0] >= 4]
        z = (147 - r) / H_ROWS
        cols = "  ".join(f"{a:3d}-{b:3d}" for a, b in rr)
        print(f"    row {r:3d}  z {z:.3f}  {cols}")


def sym_dump():
    e, half = symmetric()
    print("\n=== front.png SYMMETRISED body half-width (centre col "
          f"{e['centre']:.1f})")
    print("     rows      z/H top..bot    half_px   half/H   width/H")
    cur = None
    out = []
    for i in range(e["n"]):
        k = None if np.isnan(half[i]) else round(float(half[i]), 1)
        if cur is None or k != cur[2]:
            if cur is not None:
                out.append(cur)
            cur = [e["top"] + i, e["top"] + i, k]
        else:
            cur[1] = e["top"] + i
    out.append(cur)
    for a, b, k in out:
        zt, zb = (147 - a) / H_ROWS, (146 - b) / H_ROWS
        if k is None:
            print(f"    {a:3d}-{b:3d}  {zt:.3f}..{zb:.3f}   (none)")
        else:
            print(f"    {a:3d}-{b:3d}  {zt:.3f}..{zb:.3f}  {k:7.1f}  "
                  f"{k/H_ROWS:+.3f}   {2*k/H_ROWS:.3f}")


if __name__ == "__main__":
    dump()
    sym_dump()
    legs()
