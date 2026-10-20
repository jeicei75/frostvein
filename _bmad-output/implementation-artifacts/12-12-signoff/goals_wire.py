#!/usr/bin/env python3
"""Story 12.12's wire instrument: does the daemon run the goal chain, and do its goals say what the
stockpiles hold?

    python3 _bmad-output/implementation-artifacts/12-12-signoff/goals_wire.py PORT [TICKS]
        [--wood N] [--mineral N] [--gem N] [--plan-out PLAN.json]

Run it against a FRESH `simd PORT` (DEFAULT_SEED), started from a directory with no
`frostvein.save`, and connect at once: the plan starts from where the connect snapshot finds the
dwarves (12.10's #191). The targets default to 10 / 10 / 1 (Task 0 Q1, recommended).

It plans a scripted chain from the snapshot alone:
  - wood: cut the cheapest trees (one log per trunk cell, as `fell_tree` yields) until their logs
    reach the wood target;
  - minerals, then gems: the cheapest tunnels (steps + 15 per dig, digging at the digger's own
    level, as 12.10's `ore_wire.py` plans) to that many tetrahedrite, then bloodstone, cells, each
    later tunnel free to reuse the earlier ones;
  - piles: one cell per item the plan yields, plus 20 (a dig under a trunk fells it, 12.11 Q2), in
    compact blobs of at most 30 cells near the crew, kept apart and refused if one ENCLOSES
    walkable ground (a pile ringing the campfire seals itself off from the haulers; story 12.12,
    Found at creation).
It pauses the daemon on connect, plans, sends every mark and the piles, then runs at fast4x and
reads the daemon's own deltas (drained on a thread, so a slow plan never backs the daemon up).
`--plan-out` writes the plan as JSON (the scenario test's literals come from it).

THE ORACLE, independent of the sim: on every delta it counts the STORED items of each kind from the
wire alone, by 3.3's rule (`create_haul_jobs`): an item is stored iff it is not carried, lies on a
stockpile cell, and is the lowest-id uncarried item on that cell.

THE JUDGE, if the delta carries `goals` (12.12's wire diff:
`[{"kind":"wood","target":N,"progress":P,"completed":B}, ...]`, wood then tetrahedrite then
bloodstone): on every delta, every goal's `progress` equals the oracle's count of its kind and its
`target` the constant; no `completed` ever goes back to false; a goal is completed only once every
goal before it is, and it is completed on the first delta where every earlier goal is completed and
the oracle reaches its target (the daemon never pauses here, so the sim's per-tick completion is
never behind the live count).

PHASE 2: once the chain is done (on a daemon with no `goals`: once the oracle has met every
target), it removes the pile holding the most counted items and reads 200 more ticks, so the stock
DROPS. The goals must stay completed, `progress` must follow the oracle down, and the tick must keep
climbing (play continues). A run in which no count dropped is RED: stickiness went unexercised.

One line per finding, then one verdict: `GOALS OK` (exit 0), `GOALS RED` (exit 1), or
`GOALS BROKEN: ...` (exit 2: no snapshot, no plan, the daemon closed or stalled). On a daemon that
sends no `goals` field it still reports how far the ORACLE got (the plan's reachability), then
`GOALS RED`. Exit 0 alone is not a result: read the lines.
"""
import heapq
import json
import queue
import socket
import sys
import threading

argv = sys.argv[1:]


def flag(name, default):
    if name in argv:
        i = argv.index(name)
        value = int(argv[i + 1])
        del argv[i:i + 2]
        return value
    return default


plan_out = None
if "--plan-out" in argv:
    i = argv.index("--plan-out")
    plan_out = argv[i + 1]
    del argv[i:i + 2]
WOOD = flag("--wood", 10)
MINERAL = flag("--mineral", 10)
GEM = flag("--gem", 1)
port = int(argv[0])
limit = int(argv[1]) if len(argv) > 1 else 30000
CHAIN = [("wood", WOOD), ("tetrahedrite", MINERAL), ("bloodstone", GEM)]
AFTER = 200


def broken(why):
    print(f"GOALS BROKEN: {why}")
    sys.exit(2)


sock = socket.create_connection(("127.0.0.1", port))
f = sock.makefile()
first = f.readline()
# Freeze the world while the plan is made: planning takes tens of seconds, the daemon would tick on
# under it (and drop this reader with `client delta queue full` if nothing drained its deltas).
# Every command then lands on the paused tick, so the run starts from the state that was planned.
sock.sendall(b'{"type":"set_speed","speed":"paused"}\n')
lines = queue.Queue()


def drain():
    for raw in f:
        lines.put(raw)
    lines.put(None)


threading.Thread(target=drain, daemon=True).start()
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


camp = next(tuple(e["pos"]) for e in snap["entities"] if e["kind"] == "campfire")
emitters = {tuple(e["pos"]) for e in snap["entities"] if e["kind"] in ("campfire", "torch")}
lying = {tuple(i["pos"]) for i in snap["items"]}
start = [tuple(e["pos"]) for e in snap["entities"] if e["kind"] == "dwarf"]
DIRS = ((-1, 0), (1, 0), (0, -1), (0, 1))
DIG_COST = 15


def plan_costs(dug):
    """Dijkstra from the dwarves: a step costs 1, a dig DIG_COST (12.10's `ore_wire.py` planner).
    Cells in `dug` are already planned digs, so they walk like empty ground."""
    def empty(p):
        return tiles.get(p) == "empty" or p in dug

    def standable(p):
        x, y, z = p
        return empty(p) and isinstance(tiles.get((x, y, z - 1)), dict) and (x, y, z - 1) not in dug

    def diggable(p):
        kind = solid(p) or ramp(p)
        return (kind is not None and kind not in ("tree_trunk", "tree_foliage") and p not in dug
                and p not in emitters and p not in lying)

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
                if c in emitters or c in lying:
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
    return cost, parent, standable


# Ore tunnels, greedily: each target is the cheapest cell of its kind given every dig planned so far.
dug = set()
digs = []
targets = {}
for kind, wanted in CHAIN[1:]:
    targets[kind] = []
    for _ in range(wanted):
        cost, parent, _ = plan_costs(dug)
        reach = [(cost[c], c) for c in cost if solid(c) == kind and c not in dug]
        if not reach:
            broken(f"no more {kind} within digging reach after {len(targets[kind])}")
        _, target = min(reach)
        path = []
        c = target
        while c in parent:
            if tiles[c] != "empty" and c not in dug:
                path.append(c)
            c = parent[c]
        for c in reversed(path):
            dug.add(c)
            digs.append(c)
        targets[kind].append(target)
    depths = []
    for t in targets[kind]:
        top = max(z for z in range(dz) if isinstance(tiles[(t[0], t[1], z)], dict)
                  and solid((t[0], t[1], z)) not in ("tree_trunk", "tree_foliage"))
        depths.append(top - t[2])
    print(f"plan: {wanted} {kind} at {[list(t) for t in targets[kind]]} (depths {depths})", flush=True)
print(f"plan: {len(digs)} digs in all", flush=True)

# Trees: base = a trunk cell over a non-trunk; one log per trunk cell of its column.
cost, parent, standable = plan_costs(dug)
trees = []
for (x, y, z), t in tiles.items():
    if solid((x, y, z)) == "tree_trunk" and solid((x, y, z - 1)) != "tree_trunk":
        logs = 0
        while solid((x, y, z + logs)) == "tree_trunk":
            logs += 1
        near = [cost[(x + a, y + b, z)] for a, b in DIRS if (x + a, y + b, z) in cost
                and standable((x + a, y + b, z))]
        crown = {(x + a, y + b, zz) for a in (-1, 0, 1) for b in (-1, 0, 1)
                 for zz in range(z, z + logs + 2)}
        if near and not crown & dug:
            trees.append((min(near), (x, y, z), logs, crown))
trees.sort()
cuts, logs, tree_cells = [], 0, set()
for _, base, n, crown in trees:
    if logs >= WOOD:
        break
    cuts.append(base)
    logs += n
    tree_cells |= crown
if logs < WOOD:
    broken(f"only {logs} logs in reachable trees, want {WOOD}")
print(f"plan: cut {len(cuts)} trees at {[list(b) for b in cuts]} for {logs} logs", flush=True)

# +20: a dig under a trunk fells that tree too (12.11 Q2), so the crew yields more logs than planned.
need = len(digs) + logs + 20
# Piles: compact blobs of up to PILE_MAX cells, nearest the crew first, each grown from a centre at
# least 3 from every emitter, kept apart from one another, and refused if it ENCLOSES walkable
# ground. The first version of this planner ringed the fire, sealed its inner cells off from every
# hauler (`pile_targets` counts enclosed ground as a way in), and the chain stalled for want of a
# reachable pile cell, not for want of goals.
cz = camp[2]


def open_cell(c):
    return (c in cost and standable(c) and c not in dug and c not in tree_cells and c not in lying
            and all(abs(c[0] - e[0]) + abs(c[1] - e[1]) > 2 for e in emitters))


def encloses(blob):
    xs, ys, bz = [c[0] for c in blob], [c[1] for c in blob], next(iter(blob))[2]
    box = {(x, y, bz) for x in range(min(xs) - 3, max(xs) + 4) for y in range(min(ys) - 3, max(ys) + 4)}
    walk = {c for c in box if c not in blob and c not in emitters and standable(c)}
    edge = {c for c in walk if c[0] in (min(xs) - 3, max(xs) + 3) or c[1] in (min(ys) - 3, max(ys) + 3)}
    seen, todo = set(edge), list(edge)
    while todo:
        x, y, z = todo.pop()
        for a, b in DIRS:
            n = (x + a, y + b, z)
            if n in walk and n not in seen:
                seen.add(n)
                todo.append(n)
    return any((c[0] + a, c[1] + b, bz) in walk - seen for c in blob for a, b in DIRS)


pile_cells, piles, used = [], [], set()
PILE_MAX = 30
centres = sorted((cost[c] + 3 * abs(c[2] - cz), c) for c in cost
                 if open_cell(c) and all(open_cell((c[0] + a, c[1] + b, c[2])) for a, b in DIRS))
for _, centre in centres:
    if len(pile_cells) >= need:
        break
    if any((centre[0] + a, centre[1] + b, centre[2]) in used for a in (-1, 0, 1) for b in (-1, 0, 1)):
        continue
    want = min(PILE_MAX, need - len(pile_cells))
    blob, frontier, member = [centre], [centre], {centre}
    while frontier and len(blob) < want:
        frontier.sort(key=lambda c: abs(c[0] - centre[0]) + abs(c[1] - centre[1]))
        x, y, z = frontier.pop(0)
        for a, b in DIRS:
            n = (x + a, y + b, z)
            near_used = any((n[0] + i, n[1] + j, z) in used for i in (-1, 0, 1) for j in (-1, 0, 1))
            if n not in member and open_cell(n) and not near_used and len(blob) < want:
                member.add(n)
                blob.append(n)
                frontier.append(n)
    if len(blob) >= min(9, want) and not encloses(member):
        pile_cells += blob
        piles.append(blob)
        used |= member
if len(pile_cells) < need:
    broken(f"only {len(pile_cells)} pile cells in non-enclosing piles, want {need}")
print(f"plan: {need} pile cells in {len(piles)} piles round {[list(b[0]) for b in piles]}", flush=True)


def send(obj):
    sock.sendall((json.dumps(obj) + "\n").encode())


def cell(p):
    return {"min": list(p), "max": list(p)}


if plan_out:
    with open(plan_out, "w") as out:
        json.dump({"targets": dict(CHAIN), "snapshot_tick": snap["tick"], "digs": digs,
                   "cuts": cuts, "piles": piles}, out)
        out.write("\n")
    print(f"plan written to {plan_out} (snapshot tick {snap['tick']})", flush=True)
for c in digs:
    send({"type": "designate", "kind": "dig", "rect": cell(c)})
for b in cuts:
    send({"type": "designate", "kind": "cut", "rect": cell(b)})
send({"type": "place_stockpile", "rects": [cell(c) for c in pile_cells]})
send({"type": "set_speed", "speed": "fast4x"})

reached = {k: None for k, _ in CHAIN}
completed_at = {k: None for k, _ in CHAIN}
faults = []
goals_seen = 0
chain_done = None
removed_at = None
before, low = {}, {}
last = -1


def fault(text):
    if len(faults) < 10:
        print(f"FAULT {text}", flush=True)
    faults.append(text)


try:
    while True:
        line = lines.get(timeout=30)
        if line is None:
            broken(f"the daemon closed the connection at tick {last} (if its log says `delta queue "
                   "full`, it dropped this reader: run fewer at once)")
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            broken(f"a truncated message after tick {last}: the daemon dropped this reader (check its "
                   "log for `delta queue full`)")
        if msg.get("type") != "delta":
            continue
        for r in msg.get("refusals") or []:
            print(f"refusal at tick {msg['tick']}: {json.dumps(r)}", flush=True)
        if msg["tick"] == last:
            continue
        tick = last = msg["tick"]
        for change in msg["tiles"]:
            tiles[tuple(change["pos"])] = change["tile"]
        zones = {tuple(z["pos"]) for z in msg["zones"]}
        carried = {e["carrying"] for e in msg["entities"] if e.get("carrying") is not None}
        taken = set()
        count = {k: 0 for k, _ in CHAIN}
        loose = {}
        for it in sorted(msg["items"], key=lambda it: it["id"]):
            p = tuple(it["pos"])
            if it["id"] in carried:
                continue
            if p not in zones or p in taken:
                loose.setdefault(it["kind"], []).append(p)
                continue
            taken.add(p)
            if it["kind"] in count:
                count[it["kind"]] += 1
        for k, n in CHAIN:
            if reached[k] is None and count[k] >= n:
                reached[k] = tick
                print(f"oracle: {n} {k} stored by tick {tick}", flush=True)
        if tick % 2000 == 0:
            marks = len(msg["designations"])
            print(f"tick {tick}: stored {count}; marks left {marks}; items {len(msg['items'])}; "
                  f"free pile cells {len(zones - taken)}; loose "
                  f"{ {k: len(v) for k, v in loose.items()} }", flush=True)
        goals = msg.get("goals")
        if goals is not None:
            goals_seen += 1
            if [g.get("kind") for g in goals] != [k for k, _ in CHAIN]:
                fault(f"tick {tick}: goal kinds {[g.get('kind') for g in goals]}")
            else:
                earlier_done = True
                for g, (k, n) in zip(goals, CHAIN):
                    if g["target"] != n:
                        fault(f"tick {tick}: {k} target {g['target']}, want {n}")
                    if g["progress"] != count[k]:
                        fault(f"tick {tick}: {k} progress {g['progress']}, oracle {count[k]}")
                    if completed_at[k] is not None and not g["completed"]:
                        fault(f"tick {tick}: {k} un-completed (completed at {completed_at[k]})")
                    if g["completed"] and completed_at[k] is None:
                        if not earlier_done:
                            fault(f"tick {tick}: {k} completed before the goal ahead of it")
                        completed_at[k] = tick
                        print(f"goal: {k} completed at tick {tick} (progress {g['progress']}/{n})",
                              flush=True)
                    if earlier_done and count[k] >= n and not g["completed"]:
                        fault(f"tick {tick}: {k} has {count[k]}/{n} stored after the goals ahead "
                              "of it, and is not completed")
                    earlier_done = earlier_done and g["completed"]
                if earlier_done and chain_done is None:
                    chain_done = tick
                    print(f"goal: chain completed at tick {tick}", flush=True)
        # Phase 2: once the chain is done (or, on a daemon with no goals, once the oracle has met
        # every target), remove the pile holding the most counted items, so the stock DROPS: a
        # completed goal must stay completed and `progress` must follow the oracle down.
        done = chain_done if goals is not None else (max(reached.values()) if all(reached.values())
                                                     else None)
        if done is not None and removed_at is None:
            held = [sum(1 for it in msg["items"] if tuple(it["pos"]) in set(b) and it["kind"] in count)
                    for b in piles]
            pile = piles[held.index(max(held))]
            for c in pile:
                send({"type": "remove_stockpile", "rect": cell(c)})
            removed_at, before, low = tick, dict(count), dict(count)
            print(f"phase 2 at tick {tick}: stored {count}; removing the pile round {list(pile[0])} "
                  f"({max(held)} counted items on it)", flush=True)
        elif removed_at is not None:
            low = {k: min(low[k], count[k]) for k in low}
            if tick >= removed_at + AFTER:
                print(f"phase 2 end at tick {tick}: stored {count}; lowest since removal {low}",
                      flush=True)
                break
        if tick >= limit:
            break
except queue.Empty:
    broken(f"the daemon stalled for 30 s at tick {last}")

for e in msg["entities"]:
    if e["kind"] == "dwarf":
        print(f"dwarf {e['id']} {e.get('profession')} at {e['pos']} state {e['state']} "
              f"job {e.get('job')} carrying {e.get('carrying')}")
for k, ps in sorted(loose.items()):
    if k != "stone":
        print(f"loose at the end: {len(ps)} {k} at {[list(p) for p in ps[:8]]}")
for k, n in CHAIN:
    print(f"{k}: oracle reached {n} at {'tick ' + str(reached[k]) if reached[k] else 'NEVER'}; "
          f"goal completed at {'tick ' + str(completed_at[k]) if completed_at[k] else 'NEVER'}")
if goals_seen == 0:
    print(f"goals: the daemon sent NO `goals` field (last tick {last})")
else:
    print(f"goals: {goals_seen} deltas carried goals; {len(faults)} faults")
    if chain_done is not None:
        print(f"play continued to tick {last} ({last - chain_done} ticks after the chain)")
dropped = [k for k in low if low[k] < before[k]]
if removed_at is not None:
    print(f"stock dropped after the removal for: {dropped or 'NOTHING (stickiness unexercised)'}")
ok = (goals_seen > 0 and not faults and chain_done is not None and removed_at is not None
      and last >= removed_at + AFTER and bool(dropped))
print("GOALS OK" if ok else "GOALS RED")
sys.exit(0 if ok else 1)
