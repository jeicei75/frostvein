#!/usr/bin/env python3
"""Story 12.9's wire instrument: does the daemon ever put two dwarves on one tile?

    python3 _bmad-output/implementation-artifacts/12-9-signoff/occupancy_wire.py PORT [TICKS]

Run it against a FRESH `simd PORT` (DEFAULT_SEED). Like 12.5's work_wire.py and 12.6's
trade_wire.py it designates a 4x7 CHANNEL block east of the fire and a 3x3 stockpile west of it,
sets fast4x, and reads the daemon's own deltas until TICKS (default 1500), so miners, haulers and
idle wanderers all move through the camp bowl at once.

On every distinct tick it groups the dwarf entities by `pos` and counts:
  - shared ticks: ticks on which two or more dwarves stand on one tile (FR49 says zero);
  - dwarf moves: position changes summed over all dwarves (a crew frozen in place also shares
    nothing, so a fix that stops movement must not read as green);
  - channel marks cleared and stones on the pile (the crew still works, not just walks);
  - stone entries (#162, AC10, added at Task 0): a dwarf whose tile changed ONTO a tile that held an
    uncarried item (one no dwarf's `carrying` names) in the previous delta. Every item blocks.
  - items landed on a dwarf (12.9 review, #182): an uncarried item that is new, moved, or was
    carried in the previous delta, and now lies on a tile a dwarf stands on. A stone ENTRY is a
    dwarf moving onto an item; this is the other direction, an item placed onto a dwarf.
Every delta must carry exactly five dwarves, every tick after the first must be read (a gap can
hide a shared tick), and every delta must parse, or the run proves nothing and says so (exit 2).

It prints the counts, the first shared tile, the first stone entry and the first landing, then one
verdict line per rule: OCCUPANCY OK/RED and STONES OK/RED. Exit 1 if either is RED.

Observed at creation (2026-10-08, main 26185a0, release simd, fast4x): see the story's
Verification section. Exit 0 alone is not a result: read the counts.
"""
import json
import socket
import sys
from collections import defaultdict

port = int(sys.argv[1])
limit = int(sys.argv[2]) if len(sys.argv) > 2 else 1500
sock = socket.create_connection(("127.0.0.1", port))
f = sock.makefile()
snap = json.loads(f.readline())
dx, dy = snap["dims"]["x"], snap["dims"]["y"]
tiles = snap["tiles"]


def tile(x, y, z):
    return tiles[x + y * dx + z * dx * dy]


def standable(x, y, z):
    return tile(x, y, z) == "empty" and isinstance(tile(x, y, z - 1), dict)


camp = next(e["pos"] for e in snap["entities"] if e["kind"] == "campfire")
assert camp == [64, 64, 9], f"recipe is pinned to DEFAULT_SEED's camp, got {camp}"
cx, cy, cz = camp
channel = {"min": [65, 61, cz], "max": [68, 67, cz]}
pile = None
for r in range(3, 12):
    for px, py in ((cx - r - 2, cy), (cx, cy + r), (cx, cy - r - 2)):
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


def send(obj):
    sock.sendall((json.dumps(obj) + "\n").encode())


send({"type": "designate", "kind": "channel", "rect": channel})
send({"type": "place_stockpile", "rects": [pile]})
send({"type": "set_speed", "speed": "fast4x"})
print(f"camp {camp} channel {channel['min']}..{channel['max']} pile {pile['min']}..{pile['max']}", flush=True)

seen_ticks = set()
shared_ticks = 0
first_shared = None
max_per_tile = 0
moves = 0
bad_counts = 0
last = None
peak_marks = 0
marks = 0
on_pile = 0
stone_entries = 0
first_entry = None
last_items = set()
landings = 0
first_landing = None
last_item_pos = {}  # uncarried item id -> pos, previous delta
gaps = 0
for line in f:
    try:
        msg = json.loads(line)
        if msg.get("type") != "delta":
            continue
        tick = msg["tick"]
        dwarves = {e["id"]: tuple(e["pos"]) for e in msg["entities"] if e["kind"] == "dwarf"}
        carried = {e.get("carrying") for e in msg["entities"] if e["kind"] == "dwarf"} - {None}
        items = {i["id"]: tuple(i["pos"]) for i in msg["items"] if i["id"] not in carried}
        marks = sum(1 for d in msg["designations"] if d["kind"] == "channel")
    except (ValueError, KeyError, TypeError) as err:
        print(f"RUN PROVES NOTHING -- malformed delta ({type(err).__name__}: {err})")
        sys.exit(2)
    if tick in seen_ticks:
        continue  # paused or repeated iteration: same tick, same state
    if seen_ticks and tick != max(seen_ticks) + 1:
        gaps += max(tick - max(seen_ticks) - 1, 1)
    seen_ticks.add(tick)
    if len(dwarves) != 5:
        bad_counts += 1
    by_tile = defaultdict(list)
    for dwarf, pos in dwarves.items():
        by_tile[pos].append(dwarf)
    crowded = {pos: ids for pos, ids in by_tile.items() if len(ids) > 1}
    max_per_tile = max(max_per_tile, max((len(ids) for ids in by_tile.values()), default=0))
    if crowded:
        shared_ticks += 1
        first_shared = first_shared or (tick, crowded)
    if last is not None:
        moved = {dwarf: pos for dwarf, pos in dwarves.items() if last.get(dwarf) != pos}
        moves += len(moved)
        entered = {dwarf: pos for dwarf, pos in moved.items() if pos in last_items}
        if entered:
            stone_entries += len(entered)
            first_entry = first_entry or (tick, entered)
    if last is not None:
        dwarf_tiles = set(dwarves.values())
        landed = {item: pos for item, pos in items.items()
                  if last_item_pos.get(item) != pos and pos in dwarf_tiles}
        if landed:
            landings += len(landed)
            first_landing = first_landing or (tick, landed)
    last = dwarves
    last_item_pos = items
    last_items = set(items.values())
    peak_marks = max(peak_marks, marks)
    on_pile = sum(1 for i in msg["items"] if tuple(i["pos"]) in pile_cells)
    if tick >= limit:
        break

print(f"ticks read {len(seen_ticks)}  deltas without exactly 5 dwarves {bad_counts}")
print(f"shared ticks {shared_ticks}  max dwarves on one tile {max_per_tile}  first shared {first_shared}")
print(f"dwarf moves {moves}  channel marks {peak_marks} -> {marks}  items on the pile {on_pile}")
print(f"stone entries {stone_entries}  first entry {first_entry}")
print(f"items landed on a dwarf {landings}  first landing {first_landing}")
print(f"tick gaps after the first delta {gaps}")
if len(seen_ticks) < limit // 2 or bad_counts or gaps:
    print("RUN PROVES NOTHING -- too few ticks read, a tick gap, or a delta without five dwarves")
    sys.exit(2)
if moves < 200 or marks >= peak_marks or on_pile == 0:
    print("CREW DID NOT WORK -- a frozen crew shares no tile; this is not a green")
    sys.exit(2)
print("OCCUPANCY OK" if shared_ticks == 0 else "OCCUPANCY RED")
print("STONES OK" if stone_entries == 0 and landings == 0 else "STONES RED")
sys.exit(0 if shared_ticks == 0 and stone_entries == 0 and landings == 0 else 1)
