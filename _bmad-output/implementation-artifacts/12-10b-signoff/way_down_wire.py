#!/usr/bin/env python3
"""Story 12.10b's wire instrument: can the crew channel a way down and walk it, judged by the daemon?

    python3 _bmad-output/implementation-artifacts/12-10b-signoff/way_down_wire.py PORT [TICKS] [SHAPE]

Run it against a FRESH `simd PORT` (DEFAULT_SEED). From the snapshot it finds a SITE: the nearest
7x3 block of standable cells at the camp's level with 6 solid non-tree cells under each, no emitter,
more than 3 from the camp (DEFAULT_SEED: [61,62,9]). It places one big stockpile on the far side of
the camp, then channels a TWO-WIDE staircase 4 levels deep, one level at a time:
    level L (0..3): channel x = site.x+L .. site.x+4, y = site.y .. site.y+1, z = camp.z - L,
each sent once the previous level's marks are all worked (every target's floor is `empty`). The pile
is big because 4 levels dig 28 blocks and one item fits per pile cell (12.9). SHAPE (12.10b review)
varies the staircase: `two` (the default, above); `onewide`, only the row y = site.y (#194); `same`,
every level the whole x = site.x .. site.x+4 rect, never shrinking (#193). Idle dwarves wander on
their own level only (#186), so the miners stay at the bottom of a finished stair, and on a one-wide
stair the blocks often land too high for a hauler to go low. So once the 4th level is worked with
nobody back up yet, the script marks one channel up top (site + [3, -1, 0]) to call a dwarf from the
bottom (as AC 3 does; `two` too since the review run 2 patch, where 1 of 16 runs was RED without it).
`same` can never be OK once
a channel that would cage the pit is refused: the last link to the rim stays as the step, so its level
is never fully worked. Read its lines, not its verdict.

It reads the daemon's own deltas at fast4x until the 4th level is worked or TICKS (default 12000).
One line per level, then one verdict line:
  `WAY DOWN OK` (exit 0): all 4 levels worked, a dwarf stood at camp.z-4 or lower (THE WAY DOWN), and a
      dwarf seen at camp.z-3 or lower was later seen back at camp.z (THE WAY UP, usually a hauler
      carrying a block to the pile);
  `WAY DOWN RED` (exit 1): a level was refused or never worked, or nobody went down or came back up;
  `WAY DOWN BROKEN: ...` (exit 2): no daemon, no snapshot, no site, no pile, an unreadable line, or the
      daemon stalled or closed before TICKS.
Exit 0 alone is not a result: read the lines.

RED at creation, on main 015548f: a channel turns the floor into a ramp and opens nothing, so level 0 is
never worked by this script's rule (`level 0: NEVER WORKED ... the floor under [61, 62, 9] is {"ramp": ...}`).
"""
import json
import socket
import sys

port = int(sys.argv[1])
limit = int(sys.argv[2]) if len(sys.argv) > 2 else 12000
shape = sys.argv[3] if len(sys.argv) > 3 else "two"
LEVELS = 4


def broken(why):
    print(f"WAY DOWN BROKEN: {why}")
    sys.exit(2)


if shape not in ("two", "onewide", "same"):
    broken(f"unknown SHAPE {shape!r} (two, onewide, same)")


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


def standable(p):
    x, y, z = p
    return tiles.get(p) == "empty" and isinstance(tiles.get((x, y, z - 1)), dict)


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
print(f"site: {list(site)} (camp {list(camp)}), shape {shape}", flush=True)


def send(obj):
    sock.sendall((json.dumps(obj) + "\n").encode())


# One big pile on the camp's far side from the site; the daemon keeps only its valid cells.
py = cy + 2 if sy < cy else cy - 9
send({"type": "place_stockpile", "rects": [{"min": [cx - 8, py, cz], "max": [cx + 8, py + 7, cz]}]})


def level_targets(level):
    if shape == "onewide":
        return [(x, sy, cz - level) for x in range(sx + level, sx + 5)]
    first = sx if shape == "same" else sx + level
    return [(x, y, cz - level) for x in range(first, sx + 5) for y in (sy, sy + 1)]


def designate(level):
    a, b = level_targets(level)[0], level_targets(level)[-1]
    send({"type": "designate", "kind": "channel", "rect": {"min": list(a), "max": list(b)}})


send({"type": "set_speed", "speed": "fast4x"})
designate(0)
level, sent_at, done = 0, None, {}
pos = {e["id"]: tuple(e["pos"]) for e in snap["entities"] if e["kind"] == "dwarf"}
deepest, went_low, came_up, refused = cz, set(), None, None
zones, last = set(), -1
st, item_pos = {}, {i["id"]: tuple(i["pos"]) for i in snap["items"]}
try:
    for line in f:
        msg = parse(line)
        if msg.get("type") != "delta":
            continue
        for r in msg.get("refusals") or []:
            print(f"refusal at tick {msg['tick']}: {json.dumps(r)}", flush=True)
            if "designate" in json.dumps(r):
                refused = refused or f"level {level}: REFUSED"
        if refused:
            break
        if msg["tick"] == last:
            continue
        tick = last = msg["tick"]
        sent_at = sent_at if sent_at is not None else tick
        for change in msg["tiles"]:
            tiles[tuple(change["pos"])] = change["tile"]
        for z in msg.get("zones") or []:
            zones.add(tuple(z["pos"]))
        for it in msg.get("items") or []:
            item_pos[it["id"]] = tuple(it["pos"])
        for e in msg["entities"]:
            if e["kind"] != "dwarf":
                continue
            st[e["id"]] = e["state"]
            p = pos[e["id"]] = tuple(e["pos"])
            deepest = min(deepest, p[2])
            if p[2] <= cz - 3:
                went_low.add(e["id"])
            elif p[2] == cz and e["id"] in went_low and came_up is None:
                came_up = (e["id"], tick)
        if all(tiles.get((x, y, z - 1)) == "empty" for x, y, z in level_targets(level)):
            done[level] = tick
            print(f"level {level}: worked by tick {tick} ({len(level_targets(level))} channels)", flush=True)
            level += 1
            if level == LEVELS:
                # Keep reading until somebody who went low comes back up, or TICKS.
                if came_up:
                    break
                top = [sx + 3, sy - 1, cz]
                send({"type": "designate", "kind": "channel", "rect": {"min": top, "max": top}})
                print(f"nobody back up yet: a channel marked up top at {top}", flush=True)
            else:
                designate(level)
        if level == LEVELS and came_up:
            break
        if tick - sent_at >= limit:
            break
    else:
        # The stream ended before any judgement: a dead daemon is not a RED.
        broken(f"the daemon closed the connection at tick {last}")
except socket.timeout:
    broken(f"the daemon stalled for 30 s at tick {last}")
except OSError as e:
    broken(f"the connection failed at tick {last}: {e}")

if not zones:
    broken("the daemon reported no stockpile cells")
if refused:
    print(refused)
elif level < LEVELS:
    left = [t for t in level_targets(level) if tiles.get((t[0], t[1], t[2] - 1)) != "empty"]
    print(f"level {level}: NEVER WORKED by tick {last} ({len(left)} of {len(level_targets(level))} "
          f"channels left; the floor under {list(left[0])} is {json.dumps(tiles.get((left[0][0], left[0][1], left[0][2] - 1)))})")
if level < LEVELS:
    print(f"dwarves below the camp floor: {sorted((p, st.get(i)) for i, p in pos.items() if p[2] < cz)}")
    print(f"items on the stair: {sorted(p for p in item_pos.values() if p[2] < cz and sx <= p[0] <= sx + 5)}")
print(f"deepest dwarf z {deepest} (camp floor {cz}; the way down needs {cz - LEVELS} or lower)")
print(f"back up: {'dwarf %d at tick %d' % came_up if came_up else 'NEVER'}")
ok = not refused and level == LEVELS and deepest <= cz - LEVELS and came_up is not None
print("WAY DOWN OK" if ok else "WAY DOWN RED")
sys.exit(0 if ok else 1)
