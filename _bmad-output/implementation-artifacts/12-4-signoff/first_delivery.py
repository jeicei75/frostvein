#!/usr/bin/env python3
"""Story 12.4's instrument: does the first stone reach the pile while the dig backlog is still there?

    python3 _bmad-output/implementation-artifacts/12-4-signoff/first_delivery.py PORT

Run it against a FRESH `simd PORT` (DEFAULT_SEED). It designates a 4x7 CHANNEL block on the camp
bowl's floor and a 3x3 stockpile west of the fire, sets fast4x, and reads the daemon's own deltas.
It prints the tick of the first delivery and how many marks are still standing when it happens.

Channels, not digs: every channel target is standable and therefore reachable, so no mark sits on
retry cooldown. A dig block on the bowl wall does NOT reproduce the symptom, because its inner tiles
are unreachable at first. Their retry stamps hand the free dwarves to the hauls (measured at
creation: first delivery at tick 212 with 39 of 49 dig marks left, on main).

Observed at creation (2026-10-02):
  RED   main f4d9ba5:            backlog 25, BACKLOG EXHAUSTED tick 220, FIRST DELIVERY tick 299 marks_left 0 of 25
  GREEN prototype (2M/2H/1W):    backlog 25, FIRST DELIVERY tick 153 marks_left 19 of 25
GREEN means marks_left > 5 (more marks than the crew could be holding). Exit 0 is not a result.
"""
import json
import socket
import sys

port = int(sys.argv[1])
sock = socket.create_connection(("127.0.0.1", port))
f = sock.makefile()
snap = json.loads(f.readline())
dx, dy, dz = snap["dims"]["x"], snap["dims"]["y"], snap["dims"]["z"]
tiles = snap["tiles"]


def tile(x, y, z):
    if not (0 <= x < dx and 0 <= y < dy and 0 <= z < dz):
        return None
    return tiles[x + y * dx + z * dx * dy]


def standable(x, y, z):
    return tile(x, y, z) == "empty" and isinstance(tile(x, y, z - 1), dict)


camp = next(e["pos"] for e in snap["entities"] if e["kind"] == "campfire")
assert camp == [64, 64, 9], f"recipe is pinned to DEFAULT_SEED's camp, got {camp}"
cx, cy, cz = camp
# The bowl floor east of the fire. Designate keeps only its standable cells (25 of 28).
channel = {"min": [65, 61, cz], "max": [68, 67, cz]}
# The first fully standable 3x3 a few cells from the fire: (59..61, 64..66) on DEFAULT_SEED.
pile = None
for r in range(3, 12):
    for px, py in ((cx + r, cy), (cx - r - 2, cy), (cx, cy + r), (cx, cy - r - 2)):
        if all(standable(x, y, cz) for x in range(px, px + 3) for y in range(py, py + 3)):
            pile = {"min": [px, py, cz], "max": [px + 2, py + 2, cz]}
            break
    if pile:
        break
assert pile, "no 3x3 standable pile near the camp"


def send(obj):
    sock.sendall((json.dumps(obj) + "\n").encode())


send({"type": "designate", "kind": "channel", "rect": channel})
send({"type": "place_stockpile", "rects": [pile]})
send({"type": "set_speed", "speed": "fast4x"})
print(f"camp {camp} channel {channel} pile {pile}", flush=True)
start = None
exhausted = None
for line in f:
    msg = json.loads(line)
    if msg.get("type") != "delta":
        continue
    marks = len(msg["designations"])
    zones = {tuple(z["pos"]) for z in msg["zones"]}
    if start is None and marks:
        start = marks
        print(f"backlog: tick {msg['tick']} marks {marks} pile cells {len(zones)}", flush=True)
    if start and not marks and exhausted is None:
        exhausted = msg["tick"]
        print(f"BACKLOG EXHAUSTED tick {exhausted}, no stone on the pile yet", flush=True)
    if start and any(tuple(i["pos"]) in zones for i in msg["items"]):
        print(f"FIRST DELIVERY tick {msg['tick']} marks_left {marks} of {start}", flush=True)
        break
    if msg["tick"] > 20000:
        print("NO DELIVERY by tick 20000")
        sys.exit(1)
