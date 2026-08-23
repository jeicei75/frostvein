#!/usr/bin/env -S uv run python
"""Put all three 7.2 mark kinds on the ground, in the order that survives a capture.

THROWAWAY. Written for story 7.2's Task 6 vehicle session; delete it once 7.2 is signed off.

    scripts/task6-designate.py [port]        # default 7451

Then start the working-zoom capture within a few seconds — the counts are already falling.

WHY THIS EXISTS RATHER THAN A LIST OF COMMANDS IN THE RUNBOOK. Three separate things about this
sim make a hand-run sequence unreliable, and each one fails SILENTLY with a zero exit:

  * `PlaceStockpile` keeps only `is_standable` positions and drops the rest without a word. A rect
    chosen by eye, or by counting TUI keystrokes from wherever the cursor opened, is a coin flip:
    measured 2026-08-22, the same key sequence gave 2 zone tiles on a fresh world and 0 on a world
    that already had the dig and channel rects. This computes the ground from the snapshot instead.
  * Channels are CONSUMED and, unlike digs, never plateau — every channel target is standable and
    therefore reachable, so there is no unreachable remainder. An 8x8 channel rect measured 39
    marks, 14 by +52 ticks, 0 by +114. A `--frames 1500` capture is ~110 ticks. Hence the large
    rect, and hence channels going LAST.
  * Digs decay to a floor. The rect below yields 79 and settles at ~50, which is why it is safe to
    designate first.
"""

import json
import socket
import sys
import time

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 7451
Z_DIG = 9
Z_GROUND = 10

# 8x12 near the camp: 79 of 96 cells are solid and become marks, settling at a floor of ~50.
DIG_RECT = {"min": [50, 58, Z_DIG], "max": [57, 69, Z_DIG]}
# The camp's standable band. Yields ~94 marks, decaying ~0.4/tick — roughly 50 still standing at
# the capture trigger. A smaller rect does not survive.
CHANNEL_RECT = {"min": [48, 54, Z_GROUND], "max": [80, 80, Z_GROUND]}


def connect():
    try:
        return socket.create_connection(("localhost", PORT), timeout=10)
    except OSError as error:
        sys.exit(f"no daemon on localhost:{PORT} ({error}) — is `simd {PORT}` running?")


def snapshot():
    sock = connect()
    try:
        line = sock.makefile("rb").readline()
    finally:
        sock.close()
    if not line:
        sys.exit("daemon accepted the connection but sent no snapshot")
    return json.loads(line)


def send(command):
    sock = connect()
    try:
        sock.sendall((json.dumps(command) + "\n").encode())
        time.sleep(0.6)  # the daemon reads on its own thread; give it a tick to drain
    finally:
        sock.close()


def standable(world):
    """Solid at z-1, empty at z — the sim's own rule (`Terrain::is_standable`)."""
    dims, tiles = world["dims"], world["tiles"]

    def at(x, y, z):
        return tiles[x + y * dims["x"] + z * dims["x"] * dims["y"]]

    def solid(tile):
        return isinstance(tile, dict) and ("solid" in tile or "ramp" in tile)

    return {
        (x, y)
        for y in range(dims["y"])
        for x in range(dims["x"])
        if solid(at(x, y, Z_GROUND - 1)) and not solid(at(x, y, Z_GROUND))
    }


def stockpile_rect(world):
    """The 2x2 nearest the world centre whose four corners are ALL standable, so all four land."""
    ground = standable(world)
    dims = world["dims"]
    centre_x, centre_y = dims["x"] // 2, dims["y"] // 2
    corners = [
        (abs(x - centre_x) + abs(y - centre_y), x, y)
        for (x, y) in ground
        if all((x + dx, y + dy) in ground for dx in (0, 1) for dy in (0, 1))
    ]
    if not corners:
        sys.exit(f"no 2x2 of standable ground at z={Z_GROUND}; the world seed may have changed")
    _, x, y = min(corners)
    return {"min": [x, y, Z_GROUND], "max": [x + 1, y + 1, Z_GROUND]}


def counts(world):
    kinds = {}
    for designation in world["designations"]:
        kinds[designation["kind"]] = kinds.get(designation["kind"], 0) + 1
    return kinds, len(world["zones"])


def main():
    send({"type": "designate", "kind": "dig", "rect": DIG_RECT})

    rect = stockpile_rect(snapshot())
    send({"type": "place_stockpile", "rect": rect})
    print(f"stockpile at {rect['min'][:2]}–{rect['max'][:2]} (z {Z_GROUND}), chosen from the snapshot")

    send({"type": "designate", "kind": "channel", "rect": CHANNEL_RECT})

    time.sleep(1)
    kinds, zones = counts(snapshot())
    print(f"designations: {kinds}  zones: {zones}")

    missing = [k for k in ("dig", "channel") if not kinds.get(k)] + ([] if zones else ["zone"])
    if missing:
        sys.exit(
            f"MISSING: {', '.join(missing)}. The capture cannot judge AC4 or AC8 without all three "
            f"— fix this before spending a capture, not after."
        )

    print("\nall three kinds are on the ground. Start the capture NOW — the counts are falling:")
    print(f"  gui.exe {PORT} --capture 7-2-marks-working.png --frames 1500 --z 10 --distance 30")
    print(f"  gui.exe {PORT} --capture 7-2-marks-vista.png   --frames 1500")
    print("\n(the vista takes NO --z: below the world top the range band skips its assertions)")


if __name__ == "__main__":
    main()
