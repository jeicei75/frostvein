# Story 7.2 — Task 6 vehicle runbook

The Dev Agent Record promised this file and it did not exist. It carries the corrected commands
(the story's own Verification block was left unedited by the dev pass, per its rules) and everything
found since, so nothing has to be reconstructed from three story files at the machine.

**Vehicle:** gingerspice (native Windows / NVIDIA). `simd` stays in WSL; `gui.exe` runs Windows-side
against `localhost:<port>`. No devpod here can open a window.

**What is being closed:** AC8, AC17, and the rendered half of AC9. Task 7 is Wolf's sign-off and no
agent can check it.

---

## 0. Before you start

```bash
cd /workspace/projects/frostvein
git checkout main && git pull --ff-only     # 7.2 + the campfire work are both on main now
scripts/gate.sh                             # must be GREEN before anything below means anything
```

---

## 1. Build and launch

```bash
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
cargo build -p simd -p tui

./target/debug/simd 7451                    # leave running in WSL
```

---

## 2. Put all three mark kinds on the ground

**How to send a raw wire command.** The daemon reads newline-delimited JSON from any TCP client
(`simd/src/main.rs:652-675`). This box has **no `nc` and no `socat`**, so the bare JSON lines the
story's Verification block prints are not runnable as written. Two things that do work here, both
executed and verified 2026-08-22:

```bash
# zsh, no dependencies -- ztcp is built in
send() {
  zmodload zsh/net/tcp
  ztcp localhost 7451 || return 1
  print -r -u $REPLY -- "$1"
  sleep 1
  ztcp -c $REPLY
}
send '{"type":"designate","kind":"dig","rect":{"min":[50,58,9],"max":[57,69,9]}}'
```

```bash
# or python, if you would rather not think about zsh modules
uv run python -c 'import socket,sys,time
s=socket.create_connection(("localhost",7451)); s.sendall(sys.argv[1].encode()+b"\n"); time.sleep(1); s.close()' \
  '{"type":"designate","kind":"dig","rect":{"min":[50,58,9],"max":[57,69,9]}}'
```

> **`uv run` resolves to THIS repo**, not the forge. `pyproject.toml` and `.python-version` were
> added 2026-08-22 for exactly that reason — before them, `uv run` walked up to `/workspace` and
> silently borrowed the forge's interpreter, which is the coupling frostvein's own-process rule
> exists to prevent. The pin matters: `requires-python = ">=3.13"` alone let uv pick 3.14.5 while
> the gate ran 3.13.13, so `.python-version` holds both at 3.13.
>
> Nothing here needs installing — the lines below are stdlib `socket`/`json`, and `dependencies` is
> deliberately empty. If uv has not made the venv yet it will do so on first `uv run`, which costs
> a second and no network.
>
> **`scripts/gate.sh` stays on bare `python3` and should NOT be moved to uv.** Its own comment gives
> the reason: it runs from the pre-commit hook, and stdlib-only means the hook cannot break on a
> missing dev dependency or an unbuilt venv. The split is: gate-path scripts use `python3`,
> anything interactive uses `uv run python`.

**ORDER MATTERS, and the reason is measured.** The dwarves CONSUME designations, and digs and
channels decay differently:

- **Digs plateau.** The 8x12 rect below yields 79 marks and decays to a **stable floor of ~50** from
  t+120 onward, because the remainder becomes unreachable. Designate it first and it will still be
  there.
- **Channels do NOT plateau — they go to ZERO.** A channel only ever targets STANDABLE ground, so
  every one of them is reachable by definition and there is no unreachable remainder. Measured
  2026-08-22: an 8x8 channel rect gave 39 marks, 14 by +52 ticks, and **0 by +114**. A
  `--frames 1500` capture is ~110 ticks, so a small channel rect designated early photographs
  nothing. **Designate channels LAST, immediately before launching the capture.**

**Do all three from one script.** A blind TUI key path for the stockpile DOES NOT WORK reliably —
`p,h,h,h,h,h,h,h,h,enter,l,j,enter` walks a fixed number of steps from wherever the cursor opens, and
whether it lands on standable ground depends on the world state at that moment. Observed
2026-08-22: it produced 2 zone tiles on a fresh world and **0 on a world that already had the dig
and channel rects**, silently, with `PlaceStockpile` accepting the command and keeping nothing.
Compute the ground instead of guessing at it.

```bash
uv run python - <<'PY'
import socket, json, time
PORT = 7451
def snap():
    s = socket.create_connection(("localhost", PORT), timeout=10)
    m = json.loads(s.makefile("rb").readline()); s.close(); return m
def send(cmd):
    s = socket.create_connection(("localhost", PORT), timeout=10)
    s.sendall((json.dumps(cmd) + "\n").encode()); time.sleep(0.6); s.close()

# (a) THE DIG RECT, first. 79 marks at t=0, plateauing at ~50 as the rest become unreachable.
send({"type": "designate", "kind": "dig",
      "rect": {"min": [50, 58, 9], "max": [57, 69, 9]}})

# (b) THE STOCKPILE, on ground proven standable from the snapshot -- solid at z-1, empty at z.
# `PlaceStockpile` keeps ONLY is_standable positions and drops the rest without a word, so a rect
# picked by eye or by keystroke count is a coin flip. This picks the 2x2 nearest the world centre
# whose four corners all qualify, and lands all four.
m = snap(); dims, tiles = m["dims"], m["tiles"]
i = lambda x, y, z: x + y * dims["x"] + z * dims["x"] * dims["y"]
solid = lambda t: isinstance(t, dict) and ("solid" in t or "ramp" in t)
Z = 10
ground = {(x, y) for y in range(dims["y"]) for x in range(dims["x"])
          if solid(tiles[i(x, y, Z - 1)]) and not solid(tiles[i(x, y, Z)])}
cx, cy = dims["x"] // 2, dims["y"] // 2
_, x0, y0 = min((abs(x - cx) + abs(y - cy), x, y) for (x, y) in ground
                if all((x + dx, y + dy) in ground for dx in (0, 1) for dy in (0, 1)))
send({"type": "place_stockpile",
      "rect": {"min": [x0, y0, Z], "max": [x0 + 1, y0 + 1, Z]}})

# (c) THE CHANNELS, LAST. Start the capture within a few seconds of this line.
send({"type": "designate", "kind": "channel",
      "rect": {"min": [48, 54, 10], "max": [80, 80, 10]}})

time.sleep(1)
m = snap()
k = {}
for d in m["designations"]:
    k[d["kind"]] = k.get(d["kind"], 0) + 1
print("designations:", k, " zones:", len(m["zones"]))
PY
```

Expect roughly `designations: {'dig': 79, 'channel': 94}  zones: 4`. **All three kinds must be
non-zero before you spend a capture.** Dig and channel counts will already be falling as you read
them; zones do not decay.

This channel rect deliberately overlaps the stockpile, so the capture also exercises the
channel-over-zone raise-and-inset added at the 2026-08-21 review. That is a thing to look at, not a
mistake.

**Confirm the sim took all three before spending a capture on it** — the cheap byte-assertable
cross-check on the expensive renderer:

```bash
uv run python -c 'import socket,json
m=json.loads(socket.create_connection(("localhost",7451)).makefile("rb").readline())
k={}
for d in m["designations"]: k[d["kind"]]=k.get(d["kind"],0)+1
print("designations:",k," zones:",len(m["zones"]))'
```

Expect something like `designations: {'dig': 50, 'channel': 94}  zones: 2`. **A missing kind here
means the capture cannot judge AC4 or AC8 — fix it before capturing, not after.**

## 3. The two captures

```bash
# Working zoom. --distance 30 exists because at BOOT_DISTANCE = 90 the 6.1 dig site occupied
# 0.30 % of a 1280x720 frame and Wolf's reaction was "did not see the difference".
gui.exe 7451 --capture 7-2-marks-working.png --frames 1500 --z 10 --distance 30

# Vista AT FULL DEPTH - no --z. This is Wolf's Task 0 Ruling 2. `range_band_applies` returns early
# and SKIPS both the warm-pixel and ground-median assertions whenever the cut is below the world
# top, so the --z 10 vista the story originally prescribed printed its numbers and asserted
# NOTHING. That is exactly how AC9's recipe proved nothing last time.
gui.exe 7451 --capture 7-2-marks-vista.png   --frames 1500
```

Name them `7-2-marks-*.png` so they cannot overwrite the approved Task 0 pair.

---

## 4. What the output must say

Match by **prefix**, never whole line — 7.1 changed the draw-set oracle's shape and older recipes
quoting the full line stopped matching.

| Line | Requirement |
|---|---|
| `marks: z 10 designations=N of E zones=M of F` | **N ≥ 20**, **M ≥ 2**, and **N == E, M == F** |
| `motion:` | ticks ≥ 100, position changes > 0, mid-blend frames > 0 |
| `capture range check:` | warm-lit pixels ≥ 3,000, ground-median luminance in `[70, 180]`, **with marks on screen** — that is AC9 |

The `of E` / `of F` are the mirror's own counts, added at the 2026-08-21 review. The instrument now
asserts projected == mirror, so a projection that silently drops half its marks fails where the old
`> 0` check passed. A mismatch is a real defect, not a recipe problem.

**Exit 0 is not a result.** Read the numbers.

---

## 5. What only your eye can settle

Everything below was computed and never rendered. Take them in this order — the first is the one
most likely to be wrong.

1. **Do the marks read as orders on a mountainside, or as a blue sheet laid over it?** This is the
   buried-dig change and it is the largest visual change here. The slice draws every solid tile at
   the cut as a full cube, which used to seal dig slabs inside opaque rock — measured live at
   **0 of 50 surviving marks visible** at the plateau while the instrument correctly printed 50.
   Buried digs are now drawn on the top face of the rock covering them. A dig with open sky above
   it is unchanged, on its own tile.
2. **Dig, channel and zone — three things or two?** Retuned twice, never seen. Dig `(56,132,250)`,
   channel `(150,96,230)`, zone `(40,120,150)`.
3. **The camp: still too blown out, or now too dim?** Ruling (d) took the base 32M → 25M keeping
   6.1's ±40% breathing, landing the peak on 5.4's approved 35.52M. This is the first look at it,
   and it is the one change that could fail in the *opposite* direction from the complaint.
4. **The vista's `capture range check:` must ASSERT, not skip.** Confirm it does not print the
   skip line. **Do not re-tune the campfire to make this pass** — that reading is a known
   carried-open item and tuning it here would be tuning to the instrument.
5. **A stockpile sitting over a dig** — does the raised, inset zone slab read, or does it fight?
6. **The gutter between neighbouring slabs** is ~2 px at `--distance 30` and ~0.65 px at the vista.
   Separate tiles, or anti-aliasing noise?
7. **The slice readout** should now read `Slice: z 10/31 - underground` with a hyphen. The em-dash
   had no glyph on the vehicle and drew as a box in `7-1-slice.png` and every capture since.
8. **AC8's full bar:** dwarves, terrain, designations, items and stockpile zones each tellable at a
   glance, and the encampment still takes the eye first.

---

## 6. Recording it (Task 6) and signing off (Task 7)

- Paste the `marks:`, `motion:` and `capture range check:` lines into the Dev Agent Record.
- State in the record, in words, the answers to §5 — especially 1, 2 and 3.
- Tick Task 6's boxes only for what was actually observed.
- **Task 7 is Wolf's alone.** View the built result against `what-you-will-see.md`, including its
  2026-08-22 amendment, which lists what changed after approval and why.

Then 7.2 moves to `done` and Epic 7 is closeable.

---

## What NOT to do

- Do not re-tune any look constant to make a capture pass. If a reading is wrong, record it open —
  6.2's campfire was carried open for exactly this reason and it was the right call.
- Do not take the vista with `--z`. It silently asserts nothing.
- Do not trust a green exit. Three separate defects on this story produced exit 0 while showing
  nothing they claimed to show.
