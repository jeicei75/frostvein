#!/usr/bin/env python3
"""Story 12.7's wire instrument: does a cut fell ONE whole tree into wood, judged by the daemon?

    python3 _bmad-output/implementation-artifacts/12-7-signoff/timber_wire.py PORT [TICKS]

Run it against a FRESH `simd PORT` (DEFAULT_SEED). It is pinned to DEFAULT_SEED's trees:
  - tree A, trunk column (73, 59), and tree B, trunk column (73, 56). Their crowns TOUCH, so a
    flood fill over tree tiles would take both. Only A is marked.
  - tree D, trunk column (65, 56): a DIG mark is sent over one trunk tile of it.
It sends, at fast4x:

    {"type":"designate","kind":"cut","rect":<one trunk cell of A, at A's base level>}
    {"type":"designate","kind":"cut","rect":<2x2 of camp air at z 9, no tree>}
    {"type":"designate","kind":"dig","rect":<one trunk cell of D>}
    {"type":"place_stockpile","rects":[<3x3 standable pile near the camp>]}

and reads the daemon's own deltas until TICKS (default 1500). It checks:
  - a `cut` mark appears at A's trunk base;
  - a woodcutter holds `{"cut":{"target":<A's base>}}`;
  - every tile of A (its trunk column, and the foliage in the 3x3 column around it from the base
    to one above the top trunk tile) becomes `empty`, while every tile of B stays as it was;
  - an item with `"kind":"wood"` appears at A's base, and later sits on a pile cell;
  - a refusal `{"command":"designate","kind":"cut",...}` arrives for the no-tree rect;
  - the dig over D's trunk never becomes a mark, and D's tiles never change.
One line per finding, then one verdict line. Exit 0 alone is not a result: read the lines.

Observed at creation (2026-10-05, main fd9ca98, release simd): see the story's Verification.
"""
import json
import socket
import sys

port = int(sys.argv[1])
limit = int(sys.argv[2]) if len(sys.argv) > 2 else 1500
sock = socket.create_connection(("127.0.0.1", port))
f = sock.makefile()
snap = json.loads(f.readline())
dx, dy, dz = snap["dims"]["x"], snap["dims"]["y"], snap["dims"]["z"]
tiles = {}
for i, t in enumerate(snap["tiles"]):
    tiles[(i % dx, (i // dx) % dy, i // (dx * dy))] = t


def material(p):
    t = tiles[p]
    return t.get("solid") if isinstance(t, dict) else None


def standable(x, y, z):
    return tiles[(x, y, z)] == "empty" and isinstance(tiles[(x, y, z - 1)], dict)


def tree(column):
    """A tree's tiles: its trunk column, plus the foliage in the 3x3 column around it, from the
    base up to one above the top trunk tile. Trunks are >= 3 apart (worldgen.rs place_trees), so
    no other tree's tile is in that box."""
    x, y = column
    zs = [z for z in range(dz) if material((x, y, z)) == "tree_trunk"]
    assert zs, f"no trunk at {column}: the recipe is pinned to DEFAULT_SEED"
    lo, hi = min(zs), max(zs)
    cells = {(x, y, z) for z in zs}
    for z in range(lo, hi + 2):
        for fx in range(x - 1, x + 2):
            for fy in range(y - 1, y + 2):
                if material((fx, fy, z)) == "tree_foliage":
                    cells.add((fx, fy, z))
    return (x, y, lo), cells


camp = next(tuple(e["pos"]) for e in snap["entities"] if e["kind"] == "campfire")
assert camp == (64, 64, 9), f"recipe is pinned to DEFAULT_SEED's camp, got {camp}"
cx, cy, cz = camp
a_base, a_cells = tree((73, 59))
b_base, b_cells = tree((73, 56))
d_base, d_cells = tree((65, 56))
assert not a_cells & b_cells
b_before = {p: tiles[p] for p in b_cells}
d_before = {p: tiles[p] for p in d_cells}
crowns_touch = any(
    sum(abs(p[i] - q[i]) for i in range(3)) == 1 for p in a_cells for q in b_cells
)
pile = None
for r in range(3, 12):
    for px, py in ((cx - r - 2, cy), (cx, cy + r), (cx, cy - r - 2), (cx + r, cy)):
        if all(standable(x, y, cz) for x in range(px, px + 3) for y in range(py, py + 3)):
            pile = {"min": [px, py, cz], "max": [px + 2, py + 2, cz]}
            break
    if pile:
        break
assert pile, "no 3x3 standable pile near the camp"
pile_cells = {
    (x, y, cz)
    for x in range(pile["min"][0], pile["max"][0] + 1)
    for y in range(pile["min"][1], pile["max"][1] + 1)
}
no_tree = {"min": [cx - 2, cy - 2, cz], "max": [cx - 1, cy - 1, cz]}
assert all(material((x, y, cz)) not in ("tree_trunk", "tree_foliage")
           for x in range(cx - 2, cx) for y in range(cy - 2, cy))


def cell(p):
    return {"min": list(p), "max": list(p)}


def send(obj):
    sock.sendall((json.dumps(obj) + "\n").encode())


send({"type": "designate", "kind": "cut", "rect": cell(a_base)})
send({"type": "designate", "kind": "cut", "rect": no_tree})
send({"type": "designate", "kind": "dig", "rect": cell(d_base)})
send({"type": "place_stockpile", "rects": [pile]})
send({"type": "set_speed", "speed": "fast4x"})
print(f"tree A base {list(a_base)} ({len(a_cells)} tiles), tree B base {list(b_base)} "
      f"({len(b_cells)} tiles), crowns touch: {crowns_touch}; dig over D's trunk {list(d_base)}",
      flush=True)

cut_mark = cut_held = a_felled = wood_at_base = wood_on_pile = cut_refused = None
dig_mark_on_tree = b_changed = d_changed = None
last = -1
for line in f:
    msg = json.loads(line)
    if msg.get("type") != "delta" or msg["tick"] == last:
        continue
    tick = last = msg["tick"]
    for change in msg["tiles"]:
        tiles[tuple(change["pos"])] = change["tile"]
    for d in msg["designations"]:
        if d["kind"] == "cut" and tuple(d["pos"]) == a_base:
            cut_mark = cut_mark or tick
        if d["kind"] == "dig" and tuple(d["pos"]) in d_cells:
            dig_mark_on_tree = dig_mark_on_tree or tick
    for e in msg["entities"]:
        job = e.get("job")
        if (e["kind"] == "dwarf" and e.get("profession") == "woodcutter"
                and isinstance(job, dict) and tuple(job.get("cut", {}).get("target", ())) == a_base):
            cut_held = cut_held or (e["id"], tick)
    for item in msg["items"]:
        if item.get("kind") == "wood":
            if tuple(item["pos"]) == a_base:
                wood_at_base = wood_at_base or tick
            if tuple(item["pos"]) in pile_cells:
                wood_on_pile = wood_on_pile or tick
    for refusal in msg.get("refusals", []):
        if (refusal.get("command") == "designate" and refusal.get("kind") == "cut"
                and refusal.get("rect") == no_tree):
            cut_refused = cut_refused or tick
    if a_felled is None and all(tiles[p] == "empty" for p in a_cells):
        a_felled = tick
    if b_changed is None and any(tiles[p] != b_before[p] for p in b_cells):
        b_changed = tick
    if d_changed is None and any(tiles[p] != d_before[p] for p in d_cells):
        d_changed = tick
    if tick >= limit:
        break


def when(v):
    return f"tick {v}" if isinstance(v, int) else (str(v) if v else "NEVER")


print(f"cut mark at A's base: {when(cut_mark)}")
print(f"a woodcutter holds the cut (dwarf, tick): {when(cut_held)}")
print(f"every tile of A empty: {when(a_felled)}")
print(f"wood item at A's base: {when(wood_at_base)}")
print(f"wood item on a pile cell: {when(wood_on_pile)}")
print(f"tree B (touching crown) changed: {when(b_changed) if b_changed else 'no'}")
print(f"cut over no tree refused: {when(cut_refused)}")
print(f"dig mark placed on tree D: {when(dig_mark_on_tree) if dig_mark_on_tree else 'no'}")
print(f"tree D changed: {when(d_changed) if d_changed else 'no'}")
ok = (cut_mark and cut_held and a_felled and wood_at_base and wood_on_pile and cut_refused
      and not b_changed and not dig_mark_on_tree and not d_changed)
print("TIMBER WIRE OK" if ok else "TIMBER WIRE RED")
sys.exit(0 if ok else 1)
