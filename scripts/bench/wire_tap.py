"""Time every message a running `simd` sends, as one more client on its socket.

    python3 wire_tap.py <out.csv> [--port 7451] [--host 127.0.0.1] [--stop-tick N]

Writes `t_s,tick,bytes` per message: seconds since the first message, the message's tick, and its
length. A world that stops and starts shows here as gaps in `t_s` between consecutive ticks; a
world that only LOOKS stopped (the client is behind) does not. Stop it with Ctrl+C, or with
`--stop-tick`.

NOTE: it is a second client, so the daemon encodes one more connect snapshot when it joins. Start
it after the gui: a `--scenario` daemon starts its clock at the first client.

Stdlib only, like the other bench scripts.
"""

import argparse
import re
import socket
import time

TICK = re.compile(rb'"tick":(\d+)')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("out")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=7451)
    parser.add_argument("--stop-tick", type=int)
    args = parser.parse_args()

    stream = socket.create_connection((args.host, args.port)).makefile("rb")
    start = None
    # Line-buffered, so Ctrl+C loses nothing.
    with open(args.out, "w", buffering=1) as out:
        out.write("t_s,tick,bytes\n")
        try:
            for line in stream:
                now = time.monotonic()
                start = now if start is None else start
                found = TICK.search(line)
                tick = int(found.group(1)) if found else -1
                out.write(f"{now - start:.4f},{tick},{len(line)}\n")
                if args.stop_tick is not None and tick >= args.stop_tick:
                    break
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
