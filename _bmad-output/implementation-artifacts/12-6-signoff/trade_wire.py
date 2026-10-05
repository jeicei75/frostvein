#!/usr/bin/env python3
"""Story 12.6's wire instrument: can a client change a dwarf's trade, and does the daemon say so?

    python3 _bmad-output/implementation-artifacts/12-6-signoff/trade_wire.py PORT [TICKS]

Run it against a FRESH `simd PORT` (DEFAULT_SEED). Like 12.5's work_wire.py it designates a 4x7
CHANNEL block east of the fire and a 3x3 stockpile west of it, at fast4x. When the lowest-id miner
holds a channel job, it sends

    {"type":"set_profession","dwarf":<that miner>,"profession":"hauler"}
    {"type":"set_profession","dwarf":999,"profession":"miner"}        # no such dwarf

and reads the daemon's own deltas until TICKS (default 900). It checks:
  - the reassigned dwarf's `profession` reads `hauler` on the wire, and he no longer holds the
    channel job (no `job`, or `"job":"haul"`);
  - the channel target he let go of is later held by a dwarf whose profession is miner;
  - a delta carries a refusal naming dwarf 999 (`"command":"set_profession"`).
It prints one line per finding, then one verdict line.

Observed at creation (2026-10-05, main 98149e1, release simd, fast4x, 900 ticks): see the story's
Verification section. Exit 0 alone is not a result: read the lines.
"""
import json
import socket
import sys

port = int(sys.argv[1])
limit = int(sys.argv[2]) if len(sys.argv) > 2 else 900
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
miners = sorted(e["id"] for e in snap["entities"] if e.get("profession") == "miner")
assert miners, "no miner in the snapshot"
miner = miners[0]


def send(obj):
    sock.sendall((json.dumps(obj) + "\n").encode())


send({"type": "designate", "kind": "channel", "rect": channel})
send({"type": "place_stockpile", "rects": [pile]})
send({"type": "set_speed", "speed": "fast4x"})
print(f"camp {camp} miners {miners} reassigning dwarf {miner} miner -> hauler", flush=True)

sent_at = None
released = None
reassigned_seen = still_holding = reclaimed_by = refusal_seen = None
for line in f:
    msg = json.loads(line)
    if msg.get("type") != "delta":
        continue
    tick = msg["tick"]
    dwarves = {e["id"]: e for e in msg["entities"] if e["kind"] == "dwarf"}
    for refusal in msg.get("refusals", []):
        if refusal.get("command") == "set_profession" and refusal.get("dwarf") == 999:
            refusal_seen = refusal_seen or tick
    if sent_at is None:
        job = dwarves[miner].get("job")
        if isinstance(job, dict) and "channel" in job:
            released = job["channel"]["target"]
            send({"type": "set_profession", "dwarf": miner, "profession": "hauler"})
            send({"type": "set_profession", "dwarf": 999, "profession": "miner"})
            sent_at = tick
            print(f"tick {tick}: dwarf {miner} holds channel {released}; sent both commands", flush=True)
    else:
        me = dwarves[miner]
        if me.get("profession") == "hauler" and reassigned_seen is None:
            reassigned_seen = tick
        job = me.get("job")
        if tick > sent_at + 2 and isinstance(job, dict) and "channel" in job:
            still_holding = still_holding or tick
        for other in dwarves.values():
            job = other.get("job")
            if (
                other["id"] != miner
                and other.get("profession") == "miner"
                and isinstance(job, dict)
                and job.get("channel", {}).get("target") == released
            ):
                reclaimed_by = reclaimed_by or (other["id"], tick)
    if tick >= limit:
        break

print(f"profession reads hauler on the wire: {'tick ' + str(reassigned_seen) if reassigned_seen else 'NEVER'}")
print(f"old channel job still held after the command: {'tick ' + str(still_holding) if still_holding else 'no'}")
print(f"released target {released} reclaimed by a miner: {reclaimed_by or 'NEVER'}")
print(f"refusal for dwarf 999: {'tick ' + str(refusal_seen) if refusal_seen else 'NEVER'}")
if sent_at is None:
    print(f"NO MINER TOOK A CHANNEL JOB BY TICK {limit} -- the recipe never fired")
    sys.exit(2)
if reassigned_seen and not still_holding and reclaimed_by and refusal_seen:
    print("TRADES WIRE OK")
    sys.exit(0)
print("TRADES WIRE RED")
sys.exit(1)
