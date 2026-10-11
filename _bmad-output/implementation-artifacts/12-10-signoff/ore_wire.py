#!/usr/bin/env python3
"""Story 12.10's wire instrument: is there ore in the rock, and does digging it yield ore, judged by
the daemon?

    python3 _bmad-output/implementation-artifacts/12-10-signoff/ore_wire.py PORT [TICKS] [--control]

Run it against a FRESH `simd PORT` (DEFAULT_SEED). It reads the snapshot and prints a CENSUS: every
solid material other than the six pre-12.10 ones is an ore kind, with its cell count, its depth
range below its column's surface, and how many of its cells touch air. Then, for each ore kind, it
plans the CHEAPEST tunnel (steps + 15 per dig) from the dwarves to a cell of that kind (walking by
`astar_neighbours`' rules, digging at the digger's own level, ramps included), marks every cell
of it, places a 3x3 pile near the camp, runs
at fast4x and reads the daemon's own deltas until TICKS (default 4000). Per kind it checks:
  - the target ore cell becomes `empty`;
  - an item whose `"kind"` is the ORE'S OWN wire name appears on the target cell;
  - that item later sits on a pile cell.
`--control` swaps the targets for plain interior stone (z <= surface - 3) and expects `"stone"`
items: it proves the planner, the digs, the item reader and the pile reader.

One line per finding, then one verdict line: `ORE WIRE OK` (exit 0), `ORE WIRE RED` (exit 1), or
`ORE WIRE BROKEN: ...` (exit 2: no snapshot, the daemon closed or stalled before TICKS, or no tunnel
could be planned). Exit 0 alone is not a result: read the lines.

Observed at creation (2026-10-11, main 32565fd, release simd): see the story's Verification.
"""
import collections
import heapq
import json
import socket
import sys

args = [a for a in sys.argv[1:] if not a.startswith("--")]
control = "--control" in sys.argv
port = int(args[0])
limit = int(args[1]) if len(args) > 1 else 4000
BASE = {"stone", "soil", "ice", "snow", "tree_trunk", "tree_foliage"}


def broken(why):
    print(f"ORE WIRE BROKEN: {why}")
    sys.exit(2)


sock = socket.create_connection(("127.0.0.1", port))
sock.settimeout(30)
f = sock.makefile()
first = f.readline()
if not first:
    broken("the daemon closed before sending a snapshot")
snap = json.loads(first)
if snap.get("type") != "snapshot":
    broken(f"first message is {snap.get('type')!r}, not a snapshot")
dx, dy, dz = snap["dims"]["x"], snap["dims"]["y"], snap["dims"]["z"]
tiles = {}
for i, t in enumerate(snap["tiles"]):
    tiles[(i % dx, (i // dx) % dy, i // (dx * dy))] = t


def solid(p):
    t = tiles.get(p)
    return t.get("solid") if isinstance(t, dict) else None


def ramp(p):
    t = tiles.get(p)
    return t.get("ramp") if isinstance(t, dict) else None


def standable(p):
    x, y, z = p
    return tiles.get(p) == "empty" and isinstance(tiles.get((x, y, z - 1)), dict)


# Column surface: the highest non-tree solid or ramp. "Interior" = at least 3 below it.
surface = {}
for (x, y, z), t in tiles.items():
    if isinstance(t, dict) and solid((x, y, z)) not in ("tree_trunk", "tree_foliage"):
        surface[(x, y)] = max(surface.get((x, y), -1), z)

census = collections.defaultdict(lambda: {"cells": 0, "depths": [], "exposed": 0, "ramps": 0})
for p, t in tiles.items():
    name = solid(p) or ramp(p)
    if not name or name in BASE:
        continue
    entry = census[name]
    entry["cells"] += 1
    entry["ramps"] += ramp(p) is not None
    entry["depths"].append(surface[(p[0], p[1])] - p[2])
    x, y, z = p
    if any(tiles.get(n) == "empty" for n in
           ((x + 1, y, z), (x - 1, y, z), (x, y + 1, z), (x, y - 1, z), (x, y, z + 1), (x, y, z - 1))):
        entry["exposed"] += 1
if census:
    for name, e in sorted(census.items()):
        print(f"census: {name} cells={e['cells']} depth={min(e['depths'])}..{max(e['depths'])} "
              f"exposed={e['exposed']} ramps={e['ramps']}", flush=True)
else:
    print("census: no ore kinds (every solid tile is one of the six pre-12.10 materials)", flush=True)

# The tunnel planner: cheapest path from the dwarves, a step costing 1 and a dig DIG_COST (its 50
# work ticks plus hauling its stone out, in step-equivalents: a step is ~10 ticks), walking by `astar_neighbours`' rules
# (lib.rs), digging only at the digger's own level (`work_positions`: side neighbours). It digs
# RAMPS as well as solid non-tree tiles: that is 12.10's dig rule (Task 0.2). On a tree without
# it, every hill is ringed by undiggable ramps and nothing below the surface is reachable.
camp = next(tuple(e["pos"]) for e in snap["entities"] if e["kind"] == "campfire")
blocked = {tuple(e["pos"]) for e in snap["entities"] if e["kind"] in ("campfire", "torch")}
blocked |= {tuple(i["pos"]) for i in snap["items"]}
start = [tuple(e["pos"]) for e in snap["entities"] if e["kind"] == "dwarf"]
DIRS = ((-1, 0), (1, 0), (0, -1), (0, 1))
DIG_COST = 15


def diggable(p):
    kind = solid(p) or ramp(p)
    return kind is not None and kind not in ("tree_trunk", "tree_foliage") and p not in blocked


cost = {s: 0 for s in start}
parent = {}
heap = [(0, s) for s in start]
while heap:
    d, here = heapq.heappop(heap)
    if d > cost[here]:
        continue
    x, y, z = here
    for ddx, ddy in DIRS:
        for ddz in (0, -1, 1):
            c = (x + ddx, y + ddy, z + ddz)
            if c in blocked:
                continue
            if standable(c):
                lower = c if ddz < 0 else here
                if ddz and ramp((lower[0], lower[1], lower[2] - 1)) is None:
                    continue
                step = 1
            elif ddz == 0 and diggable(c) and isinstance(tiles.get((x + ddx, y + ddy, z - 1)), dict):
                step = DIG_COST
            else:
                continue
            if d + step < cost.get(c, 1 << 30):
                cost[c] = d + step
                parent[c] = here
                heapq.heappush(heap, (d + step, c))
print(f"crew can walk to {sum(1 for c in cost if tiles[c] == 'empty')} cells from {len(start)} "
      f"dwarves; {sum(1 for c in cost if tiles[c] != 'empty')} cells are within digging reach",
      flush=True)


def interior(p):
    return solid(p) == "stone" and p[2] <= surface[(p[0], p[1])] - 3


targets = {"stone": interior} if control else {name: (lambda n: lambda p: solid(p) == n)(name)
                                               for name in census}
plans = {}
for name, wanted in targets.items():
    reach = [(cost[c], c) for c in cost if wanted(c)]
    if not reach:
        broken(f"no {name} cell is within digging reach of the crew")
    _, target = min(reach)
    cells = []
    c = target
    while c in parent:
        if tiles[c] != "empty":
            cells.append(c)
        c = parent[c]
    plans[name] = cells[::-1]
    print(f"plan: {name} target {list(target)} (depth {surface[target[:2]] - target[2]}) via "
          f"{len(cells)} digs from {list(c)}", flush=True)
taken = {p for cells in plans.values() for p in cells}

cx, cy, cz = camp
pile = None
for r in range(3, 12):
    for px, py in ((cx - r - 2, cy), (cx, cy + r), (cx, cy - r - 2), (cx + r, cy)):
        cells = {(x, y, cz) for x in range(px, px + 3) for y in range(py, py + 3)}
        if all(standable(c) and c not in blocked for c in cells) and not cells & taken:
            pile = {"min": [px, py, cz], "max": [px + 2, py + 2, cz]}
            break
    if pile:
        break
if not pile:
    broken("no 3x3 standable pile near the camp")
pile_cells = {(x, y, cz) for x in range(pile["min"][0], pile["max"][0] + 1)
              for y in range(pile["min"][1], pile["max"][1] + 1)}


def send(obj):
    sock.sendall((json.dumps(obj) + "\n").encode())


for cells in plans.values():
    for c in cells:
        send({"type": "designate", "kind": "dig", "rect": {"min": list(c), "max": list(c)}})
send({"type": "place_stockpile", "rects": [pile]})
send({"type": "set_speed", "speed": "fast4x"})

found = {name: {"dug": None, "item": None, "piled": None, "wrong": None} for name in plans}
target_of = {cells[-1]: name for name, cells in plans.items()}
item_of = {}
last = -1
try:
    for line in f:
        msg = json.loads(line)
        if msg.get("type") != "delta" or msg["tick"] == last:
            continue
        tick = last = msg["tick"]
        for change in msg["tiles"]:
            p = tuple(change["pos"])
            tiles[p] = change["tile"]
            if p in target_of and change["tile"] == "empty":
                found[target_of[p]]["dug"] = found[target_of[p]]["dug"] or tick
        for item in msg["items"]:
            p = tuple(item["pos"])
            if p in target_of and item["id"] not in item_of:
                name = target_of[p]
                if found[name]["dug"] and item["kind"] == name:
                    item_of[item["id"]] = name
                    found[name]["item"] = found[name]["item"] or tick
                elif found[name]["dug"] and not found[name]["wrong"]:
                    found[name]["wrong"] = f"tick {tick} kind={item['kind']!r}"
            if item["id"] in item_of and p in pile_cells:
                name = item_of[item["id"]]
                found[name]["piled"] = found[name]["piled"] or tick
        if tick >= limit:
            break
except socket.timeout:
    broken(f"the daemon stalled for 30 s at tick {last}")
if last < limit:
    broken(f"the daemon closed at tick {last}, before TICKS={limit}")


def when(v):
    return f"tick {v}" if v else "NEVER"


ok = bool(plans)
for name, r in sorted(found.items()):
    print(f"{name}: target dug {when(r['dug'])}; {name} item on it {when(r['item'])}; "
          f"on a pile cell {when(r['piled'])}"
          + (f"; FIRST ITEM THERE WAS THE WRONG KIND ({r['wrong']})" if r["wrong"] else ""))
    ok = ok and r["dug"] and r["item"] and r["piled"]
if not control:
    kinds = len(census)
    print(f"ore kinds in the rock: {kinds} (12.10 wants 2 or 3: one or two minerals and one gem)")
    ok = ok and 2 <= kinds <= 3 and all(e["exposed"] == 0 for e in census.values())
print(("ORE WIRE OK" if ok else "ORE WIRE RED") + (" (CONTROL: plain stone)" if control else ""))
sys.exit(0 if ok else 1)
