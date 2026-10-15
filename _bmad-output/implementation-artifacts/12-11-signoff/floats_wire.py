#!/usr/bin/env python3
"""Story 12.11's wire instrument: does anything stay floating over a dug-out cell, judged by the daemon?

    python3 _bmad-output/implementation-artifacts/12-11-signoff/floats_wire.py PORT [TICKS]

Run it against a FRESH `simd PORT` (DEFAULT_SEED), started from a directory with no `frostvein.save`.
It drives two routes to #135 at once, reading the daemon's own deltas at fast4x:

  STAIR: 12.10b's two-wide staircase (the site and levels of `12-10b-signoff/way_down_wire.py`: the
      nearest 7x3 block of standable cells at the camp's level with 6 solid non-tree cells under each,
      DEFAULT_SEED [61,62,9]), channelled 4 levels deep one level at a time, with a 17x8 pile on the
      camp's far side (big enough for all 28 blocks once none of them float). A channel's block lands
      up top beside its worker (12.10b), which is often a cell a later level channels under.
  TREE: one dig under a tree. From the snapshot it picks the trunk base nearest the camp whose ground
      cell (the solid cell under the base) has a standable side neighbour on its own level, more than
      12 from the site, and marks a dig on that ground cell.

An item FLOATS when it lies uncarried on an `empty` cell over an `empty` cell. A tree FLOATS when its
trunk base stands over an `empty` cell. A float shorter than GRACE (30) ticks is not counted, so a
fall in progress is not one. One line per level and per event, then the counts and one verdict:
  `FLOATS OK` (exit 0): all 4 levels worked, the tree's ground cell was dug, no item floated for GRACE
      ticks or longer, no trunk base was left over air, and at least one block reached the pile;
  `FLOATS RED` (exit 1): something floated, or a level or the tree dig was never worked;
  `FLOATS BROKEN: ...` (exit 2): no daemon, no snapshot, no site, no tree, an unreadable line, or the
      daemon stalled or closed before TICKS.
Exit 0 alone is not a result: read the counts.

RED at creation, on main 56bfbe5: blocks float on the stair (`items floated >= 30 ticks: 8`, on the
run recorded in the story) and the tree is left standing on air (`tree ... base [x,y,z] over air`).
GREEN at creation on a throwaway prototype (items fall; a channel lifts what lies on its target up
top; a dig under a trunk fells the tree): `items floated >= 30 ticks: 0`, tree felled.
"""
import json
import socket
import sys

port = int(sys.argv[1])
limit = int(sys.argv[2]) if len(sys.argv) > 2 else 12000
LEVELS = 4
GRACE = 30


def broken(why):
    print(f"FLOATS BROKEN: {why}")
    sys.exit(2)


def parse(line):
    try:
        return json.loads(line)
    except ValueError:
        broken(f"an unreadable line from the daemon: {line[:80]!r}")


try:
    sock = socket.create_connection(("127.0.0.1", port))
except OSError as e:
    broken(f"no daemon on port {port}: {e}")
sock.settimeout(30)
f = sock.makefile()
try:
    first = f.readline()
except (socket.timeout, OSError) as e:
    broken(f"no snapshot from the daemon: {e}")
if not first:
    broken("the daemon closed before sending a snapshot")
snap = parse(first)
if snap.get("type") != "snapshot":
    broken(f"first message is {snap.get('type')!r}, not a snapshot")
dx, dy = snap["dims"]["x"], snap["dims"]["y"]
tiles = {}
for i, t in enumerate(snap["tiles"]):
    tiles[(i % dx, (i // dx) % dy, i // (dx * dy))] = t


def solid(p):
    t = tiles.get(p)
    return isinstance(t, dict) and t.get("solid") not in (None, "tree_trunk", "tree_foliage")


def trunk(p):
    t = tiles.get(p)
    return isinstance(t, dict) and t.get("solid") == "tree_trunk"


def standable(p):
    x, y, z = p
    return tiles.get(p) == "empty" and isinstance(tiles.get((x, y, z - 1)), dict)


def over_air(p):
    x, y, z = p
    return tiles.get((x, y, z - 1)) == "empty"


camp = next(tuple(e["pos"]) for e in snap["entities"] if e["kind"] == "campfire")
emitters = {tuple(e["pos"]) for e in snap["entities"] if e["kind"] in ("campfire", "torch")}
cx, cy, cz = camp
best = None
for by in range(cy - 20, cy + 20):
    for bx in range(cx - 20, cx + 20):
        cells = [(bx + i, by + j, cz) for i in range(-1, 6) for j in range(-1, 2)]
        if all(standable(p) and p not in emitters and all(solid((p[0], p[1], cz - k)) for k in range(1, 7))
               for p in cells):
            d = abs(bx - cx) + abs(by - cy)
            if d > 3 and (best is None or d < best[0]):
                best = (d, (bx, by, cz))
if best is None:
    broken("no 7x3 site with 6 solid levels under it near the camp")
sx, sy, _ = site = best[1]

tree = None
for (x, y, z), t in tiles.items():
    if not trunk((x, y, z)) or trunk((x, y, z - 1)) or not solid((x, y, z - 1)):
        continue
    ground = (x, y, z - 1)
    if not any(standable((x + ax, y + ay, z - 1)) for ax, ay in ((1, 0), (-1, 0), (0, 1), (0, -1))):
        continue
    if abs(x - sx) + abs(y - sy) <= 12:
        continue
    d = abs(x - cx) + abs(y - cy)
    if tree is None or (d, (x, y, z)) < tree[0]:
        tree = ((d, (x, y, z)), (x, y, z), ground)
if tree is None:
    broken("no tree whose ground cell can be dug from its own level")
_, base, ground = tree
print(f"site: {list(site)} (camp {list(camp)}); tree base {list(base)}, ground {list(ground)}", flush=True)


def send(obj):
    sock.sendall((json.dumps(obj) + "\n").encode())


py = cy + 2 if sy < cy else cy - 9
send({"type": "place_stockpile", "rects": [{"min": [cx - 8, py, cz], "max": [cx + 8, py + 7, cz]}]})


def level_targets(level):
    return [(x, y, cz - level) for x in range(sx + level, sx + 5) for y in (sy, sy + 1)]


def designate(kind, a, b):
    send({"type": "designate", "kind": kind, "rect": {"min": list(a), "max": list(b)}})


send({"type": "set_speed", "speed": "fast4x"})
designate("channel", level_targets(0)[0], level_targets(0)[-1])
designate("dig", ground, ground)
level, sent_at, last = 0, None, -1
zones, items, carried = set(), {}, set()
since, floated, tree_dug, tree_float = {}, {}, None, None
try:
    for line in f:
        msg = parse(line)
        if msg.get("type") != "delta":
            continue
        for r in msg.get("refusals") or []:
            print(f"refusal at tick {msg['tick']}: {json.dumps(r)}", flush=True)
        if msg["tick"] == last:
            continue
        tick = last = msg["tick"]
        sent_at = sent_at if sent_at is not None else tick
        for change in msg["tiles"]:
            tiles[tuple(change["pos"])] = change["tile"]
        for z in msg.get("zones") or []:
            zones.add(tuple(z["pos"]))
        # Absence is deletion (protocol): the item list is a full resend.
        items = {it["id"]: tuple(it["pos"]) for it in msg.get("items") or []}
        carried = {e["carrying"] for e in msg["entities"] if e["kind"] == "dwarf" and e.get("carrying") is not None}
        for item, p in items.items():
            if item not in carried and tiles.get(p) == "empty" and over_air(p):
                since.setdefault(item, tick)
                if tick - since[item] >= GRACE and item not in floated:
                    floated[item] = p
                    print(f"item {item} floating at {list(p)} since tick {since[item]}", flush=True)
            else:
                since.pop(item, None)
        if tree_dug is None and tiles.get(ground) == "empty":
            tree_dug = tick
            print(f"tree ground {list(ground)} dug by tick {tick}", flush=True)
        if tree_dug is not None and tree_float is None and tick - tree_dug >= GRACE:
            x, y, _ = base
            hanging = [z for z in range(base[2], base[2] + 8) if trunk((x, y, z)) and over_air((x, y, z))]
            tree_float = f"base {[x, y, hanging[0]]} over air" if hanging else "felled or supported"
            print(f"tree at tick {tick}: {tree_float}", flush=True)
        if level < LEVELS and all(tiles.get((x, y, z - 1)) == "empty" for x, y, z in level_targets(level)):
            print(f"level {level}: worked by tick {tick}", flush=True)
            level += 1
            if level < LEVELS:
                designate("channel", level_targets(level)[0], level_targets(level)[-1])
            else:
                stair_done = tick
        if level == LEVELS and tree_float is not None and tick - stair_done >= 600:
            break
        if tick - sent_at >= limit:
            break
    else:
        broken(f"the daemon closed the connection at tick {last}")
except socket.timeout:
    broken(f"the daemon stalled for 30 s at tick {last}")
except OSError as e:
    broken(f"the connection failed at tick {last}: {e}")

if not zones:
    broken("the daemon reported no stockpile cells")
on_pile = sum(1 for i, p in items.items() if p in zones and i not in carried)
still = sorted(list(p) for i, p in items.items() if i not in carried and tiles.get(p) == "empty" and over_air(p))
print(f"levels worked: {level} of {LEVELS}")
print(f"items floated >= {GRACE} ticks: {len(floated)}; floating at the end: {len(still)} {still[:10]}")
print(f"tree: {tree_float or 'ground NEVER DUG'}")
print(f"items on the pile: {on_pile} of {len(items)}")
ok = level == LEVELS and not floated and tree_float == "felled or supported" and on_pile > 0
print("FLOATS OK" if ok else "FLOATS RED")
sys.exit(0 if ok else 1)
