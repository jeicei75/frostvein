# Task 0 sign-off artifact: render OUR actual world (live simd snapshot) as an
# isometric night mock with the proposed 5.4 look. Mock only — sells the look;
# the real renderer is Bevy. Geometry, camp, trees, emitters are wire-true.
#
# v2 (2026-08-15, Wolf's iteration direction on the parked 08-14 candidate):
# draft-2 framing — camp in the foreground, low angle, distance fog dissolving
# the far valley into the sky, sky/aurora given the top third; trees drawn as
# snow-laden spruce sprites instead of per-tile boxes. Optional --thin F keeps
# fraction F of trees — a PROPOSAL image only (true density is the wire truth;
# thinning would be a sim-core worldgen change, not 5.4 client work).
import json, math, random, sys
import numpy as np
from PIL import Image, ImageDraw

args = [a for a in sys.argv[1:] if not a.startswith("--thin")]
SNAP, OUT = args[0], args[1]
THIN = 1.0
for a in sys.argv[1:]:
    if a.startswith("--thin"):
        THIN = float(a.split("=", 1)[1]) if "=" in a else float(sys.argv[sys.argv.index(a) + 1])

W, H = 1920, 1080
TW2, TH2, ZH = 16.0, 8.0, 15.0  # iso half-width, half-height, z lift

d = json.load(open(SNAP))
dx, dy, dz = d["dims"]["x"], d["dims"]["y"], d["dims"]["z"]
tiles = d["tiles"]

def tile_at(x, y, z):
    if not (0 <= x < dx and 0 <= y < dy and 0 <= z < dz):
        return None
    return tiles[x + y * dx + z * dx * dy]

def solidish(t):  # ramps occlude like solids (the 5.3 predicate)
    return isinstance(t, dict)

def mat_of(t):
    if not isinstance(t, dict):
        return None
    return list(t.values())[0]

def is_ramp(t):
    return isinstance(t, dict) and "ramp" in t

def is_tree(t):
    return mat_of(t) in ("tree_trunk", "tree_foliage")

# terrain predicate: tree tiles are replaced by spruce sprites, so they neither
# draw as terrain nor occlude the ground beneath them
def terrainish(t):
    return solidish(t) and not is_tree(t)

# ---- palette (value discipline: night snow midtone blue-grey; only emissive nears white)
MAT = {
    "stone": (60, 70, 92),
    "soil": (56, 52, 62),
    # the surface is a per-tile snow/ice mix; keep ice near snow's value so it
    # reads as mottling, not checkerboard (the Bevy client blends materials)
    "ice": (104, 128, 170),
    "snow": (136, 150, 178),
}
SNOWCAP = (158, 170, 196)
SPRUCE_SIDE = np.array((42, 60, 62), float)
SPRUCE_SNOW = np.array((172, 186, 210), float)
SKY_TOP = np.array((4, 8, 20), float)
SKY_HOR = np.array((18, 30, 56), float)
FOGC = np.array((20, 32, 58), float)

EMITTERS = []   # (pos, color, strength, radius)
DWARVES = []
CAMP = None
for e in d["entities"]:
    if e["light"] == "torch":
        EMITTERS.append((e["pos"], np.array((255, 150, 60), float), 0.8, 8.0))
    elif e["light"] == "campfire":
        EMITTERS.append((e["pos"], np.array((255, 172, 84), float), 1.0, 11.0))
        CAMP = e["pos"]
    elif e["kind"] == "dwarf":
        DWARVES.append(e["pos"])

def warm(x, y, z):
    w = np.zeros(3)
    for (ep, col, s, r) in EMITTERS:
        dd = math.dist((x, y, z), ep)
        if dd < r:
            w += col * (s * (1.0 - dd / r) ** 1.8)
    return w

def aurora_tint(z):
    t = max(0.0, min(1.0, (z - 8) / 18.0))
    return np.array((10, 34, 26), float) * t

# ---- wire-truth oracle: the FULL exposed set must still match 5.3's AC13 draw set
oracle = 0
for z in range(dz):
    zoff = z * dx * dy
    for y in range(dy):
        yoff = y * dx
        for x in range(dx):
            t = tiles[x + yoff + zoff]
            if not solidish(t):
                continue
            if (not solidish(tile_at(x, y, z + 1)) or not solidish(tile_at(x + 1, y, z))
                    or not solidish(tile_at(x, y + 1, z)) or not solidish(tile_at(x - 1, y, z))
                    or not solidish(tile_at(x, y - 1, z)) or not solidish(tile_at(x, y, z - 1))):
                oracle += 1
print("oracle exposed (5.3 AC13 predicate):", oracle)

# ---- collect exposed TERRAIN tiles (trees excluded, drawn as sprites instead)
exposed = []
for z in range(dz):
    zoff = z * dx * dy
    for y in range(dy):
        yoff = y * dx
        for x in range(dx):
            t = tiles[x + yoff + zoff]
            if not terrainish(t):
                continue
            top = not terrainish(tile_at(x, y, z + 1))
            fx = not terrainish(tile_at(x + 1, y, z))
            fy = not terrainish(tile_at(x, y + 1, z))
            if top or fx or fy or not terrainish(tile_at(x - 1, y, z)) or not terrainish(tile_at(x, y - 1, z)) or not terrainish(tile_at(x, y, z - 1)):
                exposed.append((x, y, z, t, top, fx, fy))

# ---- collect trees: one sprite per trunk column
trees = {}
for z in range(dz):
    zoff = z * dx * dy
    for y in range(dy):
        for x in range(dx):
            t = tiles[x + y * dx + zoff]
            if mat_of(t) == "tree_trunk":
                base, top = trees.get((x, y), (z, z))
                trees[(x, y)] = (min(base, z), max(top, z))
kept = []
for (x, y), (base, top) in sorted(trees.items()):
    hkey = (x * 73856093 ^ y * 19349663) % 1000
    if hkey < THIN * 1000:
        kept.append((x, y, base, top - base + 3.0))
print(f"trees: {len(trees)} in world, {len(kept)} rendered (thin={THIN})")

def proj(x, y, z):
    return ((x - y) * TW2, (x + y) * TH2 - z * ZH)

# framing: campfire anchored foreground, draft-2 style
cfx, cfy = proj(CAMP[0] + 0.5, CAMP[1] + 0.5, CAMP[2])
OX, OY = W * 0.48 - cfx, H * 0.78 - cfy
CAMP_ROW = CAMP[0] + CAMP[1]
HORIZON = H * 0.30

CAMP_LAT = CAMP[0] - CAMP[1]

def fogf(x, y):
    # elliptical atmosphere: terrain behind the camp dissolves into the sky
    # quickly with depth, slowly with lateral offset — a horizon, not a dome
    dr = CAMP_ROW - (x + y)          # depth behind the camp
    if dr <= 0:
        return 0.0
    dl = (x - y) - CAMP_LAT          # lateral offset
    dd = math.sqrt(dr * dr + (0.40 * dl) ** 2)
    return min(1.0, max(0.0, (dd - 30) / 34.0) ** 1.25)

def darkf(x, y):
    # foreground recedes into darkness instead
    dr = (x + y) - CAMP_ROW
    if dr <= 0:
        return 0.0
    dl = (x - y) - CAMP_LAT
    dd = math.sqrt(dr * dr + (0.35 * dl) ** 2)
    return min(0.55, max(0.0, (dd - 24) / 40.0) * 0.55)

img = Image.new("RGB", (W, H))
draw = ImageDraw.Draw(img, "RGBA")

# ---- sky: full-frame gradient; terrain paints over the lower part
sky = np.zeros((H, W, 3), float)
for row in range(H):
    t = min(1.0, row / (H * 0.52))
    sky[row, :] = SKY_TOP + (SKY_HOR - SKY_TOP) * (t ** 1.4)
img.paste(Image.fromarray(sky.astype(np.uint8)), (0, 0))

# stars (seeded)
rng = random.Random(42)
for _ in range(430):
    sx, sy = rng.uniform(0, W), rng.uniform(0, HORIZON * 1.1)
    b = rng.uniform(50, 230) * max(0.25, 1 - sy / (HORIZON * 1.2))
    r = 1 if rng.random() < 0.85 else 2
    draw.ellipse([sx - r / 2, sy - r / 2, sx + r / 2, sy + r / 2],
                 fill=(int(b * 0.9), int(b * 0.95), int(b), 255))

def skycol(sy):
    t = min(1.0, max(0.0, sy) / (H * 0.52))
    return SKY_TOP + (SKY_HOR - SKY_TOP) * (t ** 1.4)

def shade(base, mult, x, y, z, top_face):
    f = fogf(x, y)
    c = np.array(base, float) * mult
    if top_face:
        c += aurora_tint(z)
    c = c + warm(x, y, z) * 0.5 * (1 - f)
    # dissolve into the sky at this tile's own screen row, not a fixed fog color
    sy = (x + y) * TH2 - z * ZH + OY
    c = c * (1 - f) + skycol(sy) * f
    c = c * (1 - darkf(x, y))
    return tuple(int(min(255, max(0, v))) for v in c)

def quad(pts, col):
    draw.polygon([(px + OX, py + OY) for (px, py) in pts], fill=col)

# ---- painter's order over terrain + tree sprites together
draw_list = [("tile", e[0] + e[1], e[2], e) for e in exposed if fogf(e[0], e[1]) < 0.93]
draw_list += [("tree", x + y, base, (x, y, base, h)) for (x, y, base, h) in kept
              if fogf(x, y) < 0.93]
draw_list.sort(key=lambda it: (it[1], it[2]))

def boxc(x0, y0, z0, w, h_, side, top):
    A = proj(x0, y0, z0 + h_); B = proj(x0 + w, y0, z0 + h_)
    C = proj(x0 + w, y0 + w, z0 + h_); D = proj(x0, y0 + w, z0 + h_)
    quad([B, C, proj(x0 + w, y0 + w, z0), proj(x0 + w, y0, z0)],
         tuple(int(v * 0.62) for v in side))
    quad([C, D, proj(x0, y0 + w, z0), proj(x0 + w, y0 + w, z0)],
         tuple(int(v * 0.40) for v in side))
    quad([A, B, C, D], tuple(int(v) for v in top))

def draw_spruce(x, y, base, h):
    f = fogf(x, y)
    dk = 1 - darkf(x, y)
    wrm = warm(x + 0.5, y + 0.5, base + 2) * (1 - f)
    sc = skycol((x + y) * TH2 - base * ZH + OY)
    def mixc(c, wscale):
        v = (np.array(c, float) * (1 - f) + sc * f + wrm * wscale) * dk
        return np.clip(v, 0, 255)
    cx0, cy0 = x + 0.5, y + 0.5
    # visible trunk: two stacked small blocks below the foliage (Wolf, 2026-08-15)
    boxc(cx0 - 0.17, cy0 - 0.17, base, 0.34, 0.6,
         mixc((58, 48, 50), 0.5), mixc((58, 48, 50), 0.5))
    boxc(cx0 - 0.14, cy0 - 0.14, base + 0.6, 0.28, 0.6,
         mixc((52, 44, 46), 0.5), mixc((52, 44, 46), 0.5))
    # snow-laden skirts: wide flat layers, bright tops, spruce-dark undersides
    layers = [(0.82, base + 1.2, 1.0), (0.60, base + 2.3, 0.9), (0.38, base + 3.2, 0.8)]
    for r, zc, lh in layers:
        boxc(cx0 - r, cy0 - r, zc, r * 2, lh,
             mixc(SPRUCE_SIDE * 1.5, 0.35), mixc(SPRUCE_SNOW, 0.25))
    boxc(cx0 - 0.14, cy0 - 0.14, base + 4.0, 0.28, 0.5,
         mixc(SPRUCE_SNOW * 0.85, 0.2), mixc(SPRUCE_SNOW, 0.2))

for (knd, _, _, e) in draw_list:
    if knd == "tree":
        draw_spruce(*e)
        continue
    x, y, z, t, top, fx, fy = e
    m = mat_of(t)
    ztop = z + (0.5 if is_ramp(t) else 1.0)
    base = MAT.get(m, (70, 70, 80))
    if top:
        topc = SNOWCAP if m in ("stone", "soil", "snow") else base
    A = proj(x, y, ztop); B = proj(x + 1, y, ztop)
    C = proj(x + 1, y + 1, ztop); D = proj(x, y + 1, ztop)
    if fx:
        quad([B, C, proj(x + 1, y + 1, z), proj(x + 1, y, z)], shade(base, 0.82, x, y, z, False))
    if fy:
        quad([C, D, proj(x, y + 1, z), proj(x + 1, y + 1, z)], shade(base, 0.62, x, y, z, False))
    if top:
        quad([A, B, C, D], shade(topc, 1.0, x, y, z, True))

# ---- dwarves + emitter props (drawn after terrain; camp is foreground-safe)
for (px, py, pz) in DWARVES:
    # dark silhouettes against the fire, draft-style
    c = np.clip(np.array((46, 38, 36), float) + warm(px, py, pz) * 0.2, 0, 255)
    boxc(px + 0.22, py + 0.22, pz, 0.55, 1.3, c, np.clip(c * 1.05, 0, 255))
for (ep, col, s, r) in EMITTERS:
    px, py, pz = ep
    if s > 0.9:  # campfire: low bright mound
        boxc(px + 0.15, py + 0.15, pz, 0.7, 0.35, (255, 196, 112), (255, 196, 112))
    else:  # torch: thin bright post
        boxc(px + 0.36, py + 0.36, pz, 0.28, 1.05, (255, 214, 128), (255, 214, 128))

# ---- additive layers on numpy
arr = np.asarray(img).astype(float)

def add_radial(cx_, cy_, radius, col, peak):
    x0, x1 = int(max(0, cx_ - radius)), int(min(W, cx_ + radius))
    y0, y1 = int(max(0, cy_ - radius)), int(min(H, cy_ + radius))
    if x0 >= x1 or y0 >= y1:
        return
    ys, xs = np.mgrid[y0:y1, x0:x1]
    dd = np.sqrt((xs - cx_) ** 2 + (ys - cy_) ** 2) / radius
    a = np.clip(1 - dd, 0, 1) ** 2 * peak
    arr[y0:y1, x0:x1] += a[..., None] * col

# emitter glows (radii scaled with the closer camera)
for (ep, col, s, r) in EMITTERS:
    gx, gy = proj(ep[0] + 0.5, ep[1] + 0.5, ep[2] + 0.6)
    add_radial(gx + OX, gy + OY, 120 if s > 0.9 else 55, col, 0.22 if s > 0.9 else 0.20)

# aurora: bands hugging the fog horizon
ys = np.arange(H, dtype=float)[:, None]
xs = np.arange(W, dtype=float)[None, :]
for (amp, ybase, sig, col, peak, freq, ph) in [
    (30, HORIZON - 60, 24, np.array((44, 215, 150), float), 0.58, 1.9, 0.0),
    (70, HORIZON - 78, 80, np.array((30, 120, 90), float), 0.16, 0.9, 0.4),
    (40, HORIZON - 96, 30, np.array((60, 190, 190), float), 0.44, 1.3, 1.7),
    (20, HORIZON - 34, 16, np.array((120, 170, 225), float), 0.28, 2.6, 3.5),
]:
    center = ybase + amp * np.sin(xs / W * freq * 2 * math.pi + ph)
    prof = np.exp(-((ys - center) ** 2) / (2 * sig ** 2))
    fade = np.clip((HORIZON * 1.25 - ys) / (HORIZON * 1.25), 0, 1)
    arr += (prof * fade * peak)[..., None] * col

# snowfall (sparse, must not obscure the camp)
for _ in range(430):
    sx, sy = rng.uniform(0, W), rng.uniform(0, H)
    b = rng.uniform(100, 185)
    arr[int(sy), int(sx)] += (b, b, b)
    if rng.random() < 0.4 and sy + 1 < H:
        arr[int(sy) + 1, int(sx)] += (b * 0.5, b * 0.5, b * 0.5)

# vignette
vx = (np.arange(W) - W / 2) / (W / 2)
vy = (np.arange(H) - H / 2) / (H / 2)
vr = np.sqrt(vx[None, :] ** 2 + vy[:, None] ** 2)
arr *= (1 - 0.22 * np.clip(vr - 0.45, 0, 1) ** 1.6)[..., None]

Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8)).save(OUT)
print("wrote", OUT)
