import json, re, socket, subprocess, sys, time

out_path = sys.argv[1]
proc = subprocess.Popen(
    ["/workspace/projects/frostvein/target/debug/simd", "0"],
    stdout=subprocess.PIPE, text=True,
)
line = proc.stdout.readline()
m = re.search(r"listening on 127\.0\.0\.1:(\d+)", line)
if not m:
    proc.kill()
    sys.exit(f"unexpected banner: {line!r}")
port = int(m.group(1))
time.sleep(2.1)  # 10 Hz -> ~tick 20 by connect time
s = socket.create_connection(("127.0.0.1", port), timeout=5)
buf = b""
while not buf.endswith(b"\n"):
    chunk = s.recv(65536)
    if not chunk:
        break
    buf += chunk
    if b"\n" in buf:
        buf = buf.split(b"\n", 1)[0] + b"\n"
        break
s.close()
proc.kill()
snap = json.loads(buf)
with open(out_path, "w") as f:
    f.write(buf.decode())
print("tick:", snap.get("tick"), "entities:", len(snap.get("entities", [])),
      "dims:", snap.get("dims"))
