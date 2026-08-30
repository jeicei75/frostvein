"""Export one wire snapshot from the local daemon for the headless bench."""

import json
import re
import socket
import subprocess
import sys
import time
from pathlib import Path


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: export_world.py <snapshot.json>")

    out_path = Path(sys.argv[1])
    repo_root = Path(__file__).resolve().parents[2]
    simd = repo_root / "target" / "debug" / "simd"
    proc = subprocess.Popen([simd, "0"], stdout=subprocess.PIPE, text=True)
    try:
        assert proc.stdout is not None
        line = proc.stdout.readline()
        match = re.search(r"listening on 127\.0\.0\.1:(\d+)", line)
        if not match:
            raise SystemExit(f"unexpected banner: {line!r}")
        port = int(match.group(1))

        # This tick is a sample, not a property: moving entities differ between exports,
        # while terrain is deterministic. The pause lands near tick 20 at 10 Hz.
        time.sleep(2.1)
        with socket.create_connection(("127.0.0.1", port), timeout=5) as stream:
            data = b""
            while b"\n" not in data:
                chunk = stream.recv(65536)
                if not chunk:
                    break
                data += chunk
        snapshot_line = data.split(b"\n", 1)[0]
        snapshot = json.loads(snapshot_line)
        out_path.write_bytes(snapshot_line + b"\n")
        print(
            "tick:", snapshot.get("tick"),
            "entities:", len(snapshot.get("entities", [])),
            "dims:", snapshot.get("dims"),
        )
    finally:
        proc.kill()
        proc.wait()


if __name__ == "__main__":
    main()
