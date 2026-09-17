"""Round 6's composed sheets: the zoom strip and the two side-by-sides.

    uv run python src-assets/blender/sheet_r6.py

Blender's bundled interpreter has numpy but no PIL, so composition lives out here
rather than inside the render step. Round 5's sheet_r5.py is the ancestor; the two
changes are that the side-by-side is produced for the FRONT view as well as the
side (round 6 asks for both), and that everything is written under renders/r6/ --
round 5's outputs landed in renders/ and overwrote round 4's, after which it
analysed round 4's renders as its own.
"""

import os

import numpy as np
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.dirname(HERE)
R6 = os.path.join(SRC, "renders", "r6")
ORTHO = os.path.join(SRC, "references", "dwarf-ortho")
BG = (0x6F, 0x70, 0x73)
INK = (0xE8, 0xE8, 0xEA)
PAD = 18


def _label(img, text, xy, fill=INK):
    ImageDraw.Draw(img).text(xy, text, fill=fill)


def _trim(im):
    a = np.asarray(im.convert("RGB")).astype(int)
    fg = np.abs(a - np.array(BG)).sum(2) > 18
    ys, xs = np.where(fg)
    return im.crop((xs.min(), ys.min(), xs.max() + 1, ys.max() + 1))


def zoom(out):
    """Round 4's speckle metric, kept so 'does it read small' stays a number:
    delta = mean |nearest-neighbour - area-averaged| at 10, 30 and 100 px tall."""
    src = _trim(Image.open(os.path.join(R6, "dwarf-flat-front.png")).convert("RGB"))
    sizes = [10, 30, 100]
    tiles, deltas = [], []
    for s in sizes:
        wh = (max(1, round(src.width * s / src.height)), s)
        near = src.resize(wh, Image.NEAREST)
        mip = src.resize(wh, Image.BOX)
        deltas.append(float(np.abs(np.asarray(near).astype(float)
                                   - np.asarray(mip).astype(float)).mean()))
        k = 200 // s
        tiles.append(near.resize((wh[0] * k, wh[1] * k), Image.NEAREST))
    cw = max(t.width for t in tiles) + PAD
    ch = max(t.height for t in tiles)
    im = Image.new("RGB", (PAD + len(tiles) * cw, PAD + ch + 34), BG)
    for i, t in enumerate(tiles):
        x = PAD + i * cw
        im.paste(t, (x, PAD + ch - t.height))
        _label(im, "%d px tall   delta %.1f" % (sizes[i], deltas[i]), (x, PAD + ch + 4))
    _label(im, "delta = mean |nearest - mip|, on the flat albedo pass", (PAD, PAD + ch + 18))
    im.save(out)
    print("ZOOM %s  deltas %s" % (out, "  ".join("%.1f" % d for d in deltas)))
    return deltas


def vsortho(view, ref_name, out):
    """The render beside the orthographic view it was measured from, both scaled to
    the same figure height."""
    ref = Image.open(os.path.join(ORTHO, ref_name)).convert("RGB")
    ref = ref.crop((0, 7 * 5, ref.width, 148 * 5))          # the sheet's rows 7..147
    ours = _trim(Image.open(os.path.join(R6, "dwarf-flat-%s.png" % view)).convert("RGB"))
    Hp = 700
    ref = ref.resize((max(1, int(ref.width * Hp / ref.height)), Hp), Image.NEAREST)
    ours = ours.resize((max(1, int(ours.width * Hp / ours.height)), Hp), Image.NEAREST)
    im = Image.new("RGB", (PAD * 3 + ref.width + ours.width, Hp + PAD * 2 + 16), BG)
    im.paste(ref, (PAD, PAD))
    im.paste(ours, (PAD * 2 + ref.width, PAD))
    _label(im, "dwarf-ortho/%s (rows 7..147)" % ref_name, (PAD, Hp + PAD + 2))
    _label(im, "r6 flat %s, scaled to the same figure height" % view,
           (PAD * 2 + ref.width, Hp + PAD + 2))
    im.save(out)
    print("VS-ORTHO %s" % out)


if __name__ == "__main__":
    os.makedirs(R6, exist_ok=True)
    zoom(os.path.join(R6, "zoom-strip.png"))
    vsortho("side-left", "side-left.png", os.path.join(R6, "vs-ortho-side.png"))
    vsortho("front", "front.png", os.path.join(R6, "vs-ortho-front.png"))
