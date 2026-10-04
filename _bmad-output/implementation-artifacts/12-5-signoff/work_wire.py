#!/usr/bin/env python3
"""Story 12.5's wire instrument: does the daemon say WHICH work a dwarf is doing, and what he carries?

    python3 _bmad-output/implementation-artifacts/12-5-signoff/work_wire.py PORT [SPEED] [TICKS]

Run it against a FRESH `simd PORT` (DEFAULT_SEED). Like 12.4's first_delivery.py it designates a
4x7 CHANNEL block on the camp bowl's floor east of the fire and a 3x3 stockpile west of it, sets
SPEED (default fast4x), and reads the daemon's own deltas until TICKS (default 900).

For every dwarf on every distinct tick it checks the two fields 12.5 adds to `Entity`:
  - a dwarf in `state: work` holding a dig or channel job carries `job: {dig|channel: {target}}`;
  - a dwarf carrying a stone carries `carrying: <id>`, and that id is in the same delta's `items`
    at the dwarf's own cell.
It prints run lengths (ticks) per trade and state, then one verdict line.

Observed at creation (2026-10-03, main f3b7cb3, fast4x, 900 ticks):
  RED   dwarf wire keys: ['id', 'identity', 'kind', 'light', 'pos', 'profession', 'state']
        miner work runs=25 all 5 ticks; hauler work runs=19 all 5 ticks; hauler walk runs up to 87 ticks
        -> NO JOB FIELD ON THE WIRE (exit 1)
GREEN means: `job` seen on every miner work tick (dig_ticks > 0), `carrying` seen on > 0 hauler
ticks, every carried id resolves to an item on the carrier's cell, and no mismatch. Exit 0 alone is
not a result: read the counts.
"""
import json
import socket
import sys
from collections import Counter, defaultdict

port = int(sys.argv[1])
speed = sys.argv[2] if len(sys.argv) > 2 else "fast4x"
limit = int(sys.argv[3]) if len(sys.argv) > 3 else 900
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
send({"type": "set_speed", "speed": speed})
print(f"camp {camp} channel {channel} pile {pile} speed {speed}", flush=True)

keys = set()
runs = defaultdict(list)
cur = {}
dig_ticks = carry_ticks = 0
mismatches = []
last_tick = None
for line in f:
    msg = json.loads(line)
    if msg.get("type") != "delta":
        continue
    t = msg["tick"]
    if t == last_tick:
        continue
    last_tick = t
    items = {i["id"]: tuple(i["pos"]) for i in msg["items"]}
    for e in msg["entities"]:
        if e["kind"] != "dwarf":
            continue
        keys |= set(e)
        trade, st = e.get("profession"), e["state"]
        job = e.get("job")
        if trade == "miner" and st == "work":
            if isinstance(job, dict) and ("dig" in job or "channel" in job):
                dig_ticks += 1
            else:
                mismatches.append(f"tick {t} miner {e['id']} works with job={job!r}")
        carried = e.get("carrying")
        if carried is not None:
            carry_ticks += 1
            if items.get(carried) != tuple(e["pos"]):
                mismatches.append(f"tick {t} dwarf {e['id']} carries {carried} at {items.get(carried)}, not {e['pos']}")
        prev = cur.get(e["id"])
        if prev is None or prev[0] != st:
            if prev is not None:
                runs[(trade, prev[0])].append(t - prev[1])
            cur[e["id"]] = (st, t)
    if t >= limit:
        break

print("dwarf wire keys:", sorted(keys))
for (trade, st), r in sorted(runs.items()):
    print(f"{trade:10} {st:5} runs={len(r):3} lengths(ticks)={dict(sorted(Counter(r).items())[:8])}")
print(f"dig_ticks {dig_ticks} carry_ticks {carry_ticks} mismatches {len(mismatches)}")
for m in mismatches[:5]:
    print("  ", m)
if "job" not in keys:
    print("NO JOB FIELD ON THE WIRE")
    sys.exit(1)
if "carrying" not in keys:
    print("NO CARRYING FIELD ON THE WIRE")
    sys.exit(1)
if mismatches or dig_ticks == 0 or carry_ticks == 0:
    print("WORK WIRE WRONG")
    sys.exit(1)
print("WORK WIRE OK")
