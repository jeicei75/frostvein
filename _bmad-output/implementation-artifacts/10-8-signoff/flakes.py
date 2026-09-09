"""How many flakes a frame actually SHOWS, counted as bright blobs against the night sky.

Ruling 3 defect (c) is "maybe flakes could be smaller", with Wolf's own bound on it: "if we make
too small flakes then those will disappear completely". That bound is the whole reason this
script exists -- a scale ladder without a visibility floor is the lantern mistake again, where a
sixth turned out to be a speck that was largely the emissive face.

WHY NOT A DIFF AGAINST A NO-FLAKE BASELINE. That was tried first and it does not discriminate.
Under `--frames N` the capture fires on a FRAME count, so every run photographs a different sim
tick: the DWARVES have walked and their lantern pools have moved. Measured, two no-flake frames
differ by 19,212 px in 766 blobs, one of them 6,575 px -- noise the same size as the signal, with
the two smallest candidates reading BELOW it. `--at-tick` would freeze the scene but its runs end
on the frame budget long before the tick arrives (2 of 20 delivered).

SO: count the flakes directly instead. A flake is a bright blob on a very dark sky (5,12,28), and
the dwarves and their lanterns are all BELOW the terrain line, so a sky band excludes them without
needing two runs to agree on anything. Stars and the aurora are in the band too -- they are
identical across builds, so the no-flake baseline measures them and every candidate is read as its
excess over that.

LIMITATION, stated rather than hidden: the band is a fixed fraction of the frame, not the true
terrain silhouette, so this counts the flakes in the upper sky and not every flake in the picture.
It is a consistent proxy across candidates, which is what choosing between them needs; it is not a
count of all snowfall.

Usage: python3 flakes.py baseline.png candidate.png=label [candidate.png=label ...]
Reads RGB PNGs only.
"""
import pathlib
import sys

# The PNG loader lives with 10.7's instruments and is shared deliberately: a second decoder is a
# second place for a colour-type bug to hide, and its guard is what refuses a non-RGB capture.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent / "10-7-signoff"))
from lumstats import load  # noqa: E402

# A flake is lit snow on a sky whose luma is 11. Anything this bright in the sky band is a flake,
# a star, or the aurora's peak -- the last two are constant across builds and cancel in the excess.
BRIGHT = 70
# Fraction of frame height treated as sky. The boot framing puts the terrain line well below this.
SKY_BAND = 0.40


def mask(pixels, width, height):
    """Bright pixels inside the sky band; everything below the band is zeroed, not tested."""
    rows = int(height * SKY_BAND)
    out = bytearray(width * height)
    for index in range(width * rows):
        offset = index * 3
        # Rec. 601 luma, the same weighting `lumstats` reports the frame mean with.
        luma = (
            pixels[offset] * 299 + pixels[offset + 1] * 587 + pixels[offset + 2] * 114
        ) // 1000
        out[index] = 1 if luma > BRIGHT else 0
    return out


def blobs(bits, width, height):
    """4-connected runs of changed pixels, as a list of sizes. Iterative: a flake field is
    small, but a recursive flood fill on a bad threshold would blow the stack on the terrain."""
    seen = bytearray(len(bits))
    sizes = []
    for start in range(len(bits)):
        if not bits[start] or seen[start]:
            continue
        stack = [start]
        seen[start] = 1
        size = 0
        while stack:
            index = stack.pop()
            size += 1
            x, y = index % width, index // width
            for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                if 0 <= nx < width and 0 <= ny < height:
                    neighbour = ny * width + nx
                    if bits[neighbour] and not seen[neighbour]:
                        seen[neighbour] = 1
                        stack.append(neighbour)
        sizes.append(size)
    return sizes


def main(argv):
    if len(argv) < 3:
        raise SystemExit(__doc__)
    width, height, pixels = load(argv[1])
    base_bits = mask(pixels, width, height)
    base_sizes = blobs(bytearray(base_bits), width, height)
    base_blobs, base_px = len(base_sizes), sum(base_sizes)
    print(
        f"baseline (no flakes) {argv[1].rsplit('/', 1)[-1]}  {width}x{height}  "
        f"sky band={SKY_BAND:.0%}  bright>{BRIGHT}"
    )
    print(f"  stars and aurora in the band: {base_blobs} blobs, {base_px:,} px\n")
    print(f"{'label':<26}{'FLAKES':>7}{'px':>9}{'big':>7}{'median':>8}{'1px':>6}")
    for spec in argv[2:]:
        path, _, label = spec.partition("=")
        w, h, candidate = load(path)
        if (w, h) != (width, height):
            raise SystemExit(f"{path}: {w}x{h} cannot be compared with a {width}x{height} baseline")
        # Flakes only: bright here and NOT bright in the no-flake frame. Stars and the aurora sit
        # at identical pixels in every build -- the noise pair differs by 0 blobs, which is what
        # makes this subtraction legitimate rather than a second source of error.
        bits = mask(candidate, w, h)
        for index, bit in enumerate(base_bits):
            if bit:
                bits[index] = 0
        sizes = sorted(blobs(bits, w, h), reverse=True)
        median = sizes[len(sizes) // 2] if sizes else 0
        singles = sum(1 for size in sizes if size == 1)
        print(
            f"{label or path:<26}{len(sizes):>7}{sum(sizes):>9,}"
            f"{sizes[0] if sizes else 0:>7}{median:>8}{singles:>6}"
        )


if __name__ == "__main__":
    main(sys.argv)
