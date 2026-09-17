"""Compose one progress sheet per accepted step, and the zoom strip.

    uv run --extra signoff python src-assets/blender/sheet_r5.py sheet  <shots> <out> <label>
    uv run --extra signoff python src-assets/blender/sheet_r5.py zoom   <flat-front> <out>
    uv run --extra signoff python src-assets/blender/sheet_r5.py vsortho <flat-side> <out>

Blender's bundled interpreter has numpy but no PIL, so composition lives out here
rather than inside the render step.
"""

import os
import sys

import numpy as np
from PIL import Image, ImageDraw

BG = (0x6F, 0x70, 0x73)
INK = (0xE8, 0xE8, 0xEA)
PAD = 18


def _label(img, text, xy, fill=INK):
    ImageDraw.Draw(img).text(xy, text, fill=fill)


def sheet(shot_dir, out, label):
    """The WHOLE figure, full-size and small, every step -- the audit trail."""
    names = ["flat-front.png", "lit-three-quarter.png", "lit-side-left.png",
             "lit-back.png"]
    ims = [Image.open(os.path.join(shot_dir, n)).convert("RGB") for n in names]
    w, h = ims[0].size
    small = 128
    sw = small * len(ims) + PAD * (len(ims) - 1)
    W = w * len(ims) + PAD * (len(ims) + 1)
    H = PAD + h + PAD + small + PAD + 14
    out_im = Image.new("RGB", (W, H), BG)
    for i, im in enumerate(ims):
        out_im.paste(im, (PAD + i * (w + PAD), PAD))
    x0 = (W - sw) // 2
    for i, im in enumerate(ims):
        out_im.paste(im.resize((small, small), Image.LANCZOS),
                     (x0 + i * (small + PAD), PAD + h + PAD))
    _label(out_im, label, (PAD, H - 16))
    _label(out_im, "  ".join(n[:-4] for n in names), (PAD, PAD - 14))
    os.makedirs(os.path.dirname(out), exist_ok=True)
    out_im.save(out)
    print("SHEET %s  (%dx%d)" % (out, W, H))


def _trim(im):
    a = np.asarray(im.convert("RGB")).astype(int)
    fg = (np.abs(a - np.array(BG)).sum(2) > 18)
    ys, xs = np.where(fg)
    return im.crop((xs.min(), ys.min(), xs.max() + 1, ys.max() + 1))


def zoom(flat_front, out):
    """The zoom strip plus round 4's speckle metric, kept so that 'does it read
    small' stays a number: delta = mean |nearest-neighbour - area-averaged|."""
    src = _trim(Image.open(flat_front).convert("RGB"))
    sizes = [10, 30, 100]
    tiles, deltas = [], []
    for s in sizes:
        # s is the FIGURE HEIGHT in pixels; the aspect is kept, because a squashed
        # dwarf is not the thing anyone will see in game
        wh = (max(1, round(src.width * s / src.height)), s)
        near = src.resize(wh, Image.NEAREST)
        mip = src.resize(wh, Image.BOX)
        d = float(np.abs(np.asarray(near).astype(float)
                         - np.asarray(mip).astype(float)).mean())
        deltas.append(d)
        k = 200 // s
        tiles.append(near.resize((wh[0] * k, wh[1] * k), Image.NEAREST))
    cw = max(t.width for t in tiles) + PAD
    ch = max(t.height for t in tiles)
    W = PAD + len(tiles) * cw
    H = PAD + ch + 34
    im = Image.new("RGB", (W, H), BG)
    for i, t in enumerate(tiles):
        x = PAD + i * cw
        im.paste(t, (x, PAD + ch - t.height))
        _label(im, "%d px tall   delta %.1f" % (sizes[i], deltas[i]), (x, PAD + ch + 4))
    _label(im, "delta = mean |nearest - mip|, on the flat albedo pass",
           (PAD, PAD + 218))
    os.makedirs(os.path.dirname(out), exist_ok=True)
    im.save(out)
    print("ZOOM %s  deltas %s" % (out, " ".join("%.1f" % d for d in deltas)))


def vsortho(flat_side, out):
    """Deliverable 7: the render beside the orthographic view it was measured from,
    scaled to the same figure height. The side view is what was wrong, so the side
    view is what gets compared."""
    ref = Image.open("src-assets/references/dwarf-ortho/side-left.png").convert("RGB")
    # the sheet's figure is rows 7..147 of the 5x crop, and he faces +x there
    ref = ref.crop((0, 7 * 5, ref.width, 148 * 5))
    ours = _trim(Image.open(flat_side).convert("RGB"))
    # NOTE: the sheet's "Side View (Left Orthographic)" draws him facing image-right
    # with the pack behind on the image-left, which is anatomically a view of his
    # RIGHT side. Our dwarf-*-side-right.png is the camera on his right, and it
    # lands in the same orientation, so the two can be laid side by side untouched.
    Hp = 700
    ref = ref.resize((max(1, int(ref.width * Hp / ref.height)), Hp), Image.NEAREST)
    ours = ours.resize((max(1, int(ours.width * Hp / ours.height)), Hp), Image.NEAREST)
    W = PAD * 3 + ref.width + ours.width
    im = Image.new("RGB", (W, Hp + PAD * 2 + 16), BG)
    im.paste(ref, (PAD, PAD))
    im.paste(ours, (PAD * 2 + ref.width, PAD))
    _label(im, "dwarf-ortho/side-left.png (rows 7..147)", (PAD, Hp + PAD + 2))
    _label(im, "r5 flat side-left, scaled to the same figure height",
           (PAD * 2 + ref.width, Hp + PAD + 2))
    os.makedirs(os.path.dirname(out), exist_ok=True)
    im.save(out)
    print("VS-ORTHO %s" % out)


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "sheet":
        sheet(sys.argv[2], sys.argv[3], sys.argv[4])
    elif mode == "zoom":
        zoom(sys.argv[2], sys.argv[3])
    else:
        vsortho(sys.argv[2], sys.argv[3])
