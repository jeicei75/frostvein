# Story 8.2 — Task 7 vehicle runbook

Written 2026-08-27, from the worked example at `7-2-signoff/task-6-vehicle-runbook.md`. Every
command below was read off the source at `aca07be`, not copied forward: `--drag` and `--at-tick`
did not exist when 7.2's runbook was written, and three of the flag rules here were only added by
the 2026-08-26 review.

**Vehicle:** gingerspice (native Windows / NVIDIA Vulkan). `simd` stays in WSL; `gui.exe` runs
Windows-side against `localhost:<port>`. **No devpod here can open a window** — re-measured twice
at 8.1 (`bevy_winit`: neither `WAYLAND_DISPLAY` nor `WAYLAND_SOCKET` nor `DISPLAY`). Everything in
this file is vehicle-only and none of it has ever executed anywhere.

**What is being closed:** **AC19** (Task 7, the hand drags and the fps readings), **AC13's rendered
half**, **AC15 and AC16 end to end**, and **AC18**. Nothing else in 8.2 is open — the headless half
is done, 403 tests, 27/27 mutations killed, full gate green.

**Two things this session is the only defence against.** The review found the DDA march's hit face
entirely unpinned (inverting the X and Y face assignments left 149/149 green) and the slab's
rotation onto the face normal untested (deleting `.with_rotation(...)` from both call sites left
149/149 green). Both are patched and now have tests. Neither has ever been *seen*. A wrong face or
a missing rotation ships as an edge-on wafer or a highlight buried in the neighbouring cube, and
§4 is the only step that can catch it.

---

## 0. Before you start

**This story is on a branch, unlike 7.2.** Baseline `cca118a` is also `main`'s tip — 8.2 is NOT
stacked — but the work is not on `main` and there is no PR.

```bash
cd /workspace/projects/frostvein
git checkout 8-2-designate-with-the-mouse
git log --oneline -1                        # expect aca07be or later
export PATH="$HOME/.cargo/bin:$PATH"
scripts/gate.sh                             # FULL tier, no --fast. Must be GREEN before
                                            # anything below means anything.
```

A `GATE GREEN (FAST)` line is a coverage hole, not a pass — it skips `simd/tests/serve.rs`, the
61 daemon integration tests, which are exactly the ones that matter when a client starts writing
commands to that daemon.

---

## 1. Build, and STAMP IT

**M2-7 is missing for the fifth time**, so the stamp is manual. `rg 'GIT_SHA|git_sha|vergen'
crates/gui/src/` returns nothing and `scripts/` holds no build-stamp automation; there is nothing
to read the build identity out of the binary at runtime. 8.1 recorded the commit and not the
wall-clock. **Record both**, and paste them into the story:

```bash
git rev-parse --short HEAD                  # the source commit
date -u +%Y-%m-%dT%H:%M:%SZ                 # build start, UTC

time CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu

cargo build -p simd -p tui                  # WSL side, debug is fine
```

**Then re-copy `gui.exe` Windows-side and confirm the copy is the new one.** The stale-binary trap
has fired five times on this project — three in 5.4, a 216-minute-old binary at 8.1's vehicle
session, and the 08-25 stamp that predated every 8.1 patch. The mtime of the copied file, checked
against the build you just ran, is the whole guard:

```bash
ls -l --time-style=full-iso target/x86_64-pc-windows-gnu/release/gui.exe
```

Launch the daemon and leave it running:

```bash
./target/debug/simd 7451
```

---

## 2. Seed the world — but LESS than 7.2 did

7.2 needed marks already on the ground because it was photographing them. **8.2 creates its own
marks with the mouse**, so a pre-seeded world mostly gets in the way: a clear drag over ground you
did not designate yourself removes nothing, and you cannot tell that from a dead clear path.

Seed exactly one thing — **a dig rect for the clear drag in §3 step 4 to remove** — and put it
somewhere you will recognise:

```bash
uv run python -c 'import socket,json,time
s=socket.create_connection(("localhost",7451),timeout=10)
s.sendall(json.dumps({"type":"designate","kind":"dig",
  "rect":{"min":[50,58,9],"max":[57,69,9]}}).encode()+b"\n"); time.sleep(0.6); s.close()'
```

> `uv run` resolves to THIS repo (`pyproject.toml` + `.python-version`, both pinned 3.13). Nothing
> needs installing — stdlib `socket`/`json` only. `scripts/gate.sh` deliberately stays on bare
> `python3` because it runs from the pre-commit hook; the split is gate-path → `python3`,
> interactive → `uv run python`.

**Decay, which changes what you will still see.** Measured 2026-08-22 and unchanged: **digs
plateau** — an 8x12 rect gives 79 marks falling to a stable floor of ~50 from t+120, because the
remainder becomes unreachable. **Channels go to ZERO** — a channel only ever targets standable
ground, so every one is reachable; an 8x8 channel rect measured 39 marks, 14 by +52 ticks, **0 by
+114**. Consequence for this story: a channel you drag by hand is fine to look at immediately, and
a scripted `--drag channel` capture must fire at a **low** `--at-tick`, not a high one.

---

## 3. The four hand drags — AC19

Launch interactively. `--distance` and `--cursor` both **bail** without `--capture`, so there is no
zoom flag here; zoom with the keys.

```bash
gui.exe 7451                                # surface pass
```

**Controls.** `1` dig, `2` channel, `3` stockpile, `4` clear. Left button press-drag-release
commits. Right button **or** `Esc` during a drag abandons it; `Esc` with no drag leaves the mode.
`W`/`A`/`S`/`D` orbit, **`Q` zooms IN, `E` zooms OUT** (`distance` clamps to `[4.0, 500.0]`,
boot is 90.0), `,`/`.` step the slice, `F3` toggles the fps overlay.

**The hint bar is an AC on its own (AC9).** It must be visible in *every* frame and must name the
active mode. With no mode it reads `1 dig  2 channel  3 stockpile  4 clear`; in dig it reads
`dig: drag to designate  Esc leave` and, mid-drag, `dig: release to designate  Esc abort`. It is
ASCII-only on purpose — the shipped font draws a replacement box for anything else, which is what
the em-dash did in every capture from 7.1 onward. **A box glyph in that bar is a defect.**

Do these four, on the surface:

1. **`1`, drag a dig.** Blue slabs `(56,132,250)` appear on release.
2. **`2`, drag a channel.** Violet slabs `(150,96,230)`, and note they sit at a *different height*
   than dig — channel `-0.46`, dig `+0.54`. Watch the **preview** during the drag: as of the review
   patch it now takes the committing mode's own offset AND material, so what you drag is what you
   get. Before the patch every mode previewed in cyan at the dig height. **If the preview still
   reads cyan for a channel, the patch is not in the binary you are running — go back to §1.**
3. **`3`, drag a stockpile — on FLAT STANDABLE GROUND.** Slate-teal `(40,120,150)`.
   `PlaceStockpile` keeps only `is_standable` positions and **drops the rest without a word**, so a
   stockpile dragged across a cliff face or a slope legitimately keeps nothing and looks identical
   to a broken path. This is the one drag where the site choice is load-bearing.
4. **`4`, drag a clear over the §2 dig rect** (world tiles 50–57 x 58–69 at z 9). Clear sends
   **both** `CancelDesignation` and `RemoveStockpile`, in TUI order. The marks must disappear.

Then the same four **on a sliced underground level** — AC11's rendered half. Either step down with
`,` until the readout reads `Slice: z N/31 - underground`, or relaunch pinned:

```bash
gui.exe 7451 --z 10
```

**The thing to check underground is that the rect lands on the tiles you pointed at, not on the
world top.** That is the whole of AC11.

**Two aborts, while you are here** (AC7/AC8, and `Esc` was dead to every test before the review
patch): start a drag and press **`Esc`** — preview gone, nothing sent. Start another and press the
**right button** — same. Then press `Esc` with no drag running: the mode leaves and the hint bar
returns to the four-key line.

**One more, because the review patched it:** start a dig drag, and *while holding*, press `2`. The
drag must still commit as a **dig** — the mode is locked at anchor time. If it commits as a
channel, that patch is not in this binary.

---

## 4. The hover highlight on a vertical face — AC13, and the riskiest item here

This is 8.1's deferred HIGH, and the reason 8.2 exists to fix it: the hover slab used to sit at an
unconditional top-face offset, so on any tile with a drawn tile above it the highlight was sealed
inside the cube. Colour is teal `(80,220,210)`.

Point at, in this order:

1. **a cliff face** — the vertical side of a raised block,
2. **a corridor wall** — underground, on a sliced level,
3. **a shaft side** — a vertical drop.

**In each case the highlight must be drawn ON the face you are pointing at, standing up flat
against it — not lying flat on the tile above, and not edge-on.**

- Lying flat on top when you are pointing at a *side* → the entry-face computation is picking Top.
- A thin bright line instead of a face → the `.with_rotation(...)` onto the face normal is missing.
- The highlight appearing on the *neighbouring* cube → the X/Y face assignment is inverted.

All three of those were live possibilities in the code the review read, and none of them is
visible to any test that existed before 2026-08-26.

**One deliberate edge case.** `entry_face` returns `Top` unconditionally when the camera sits
*inside* solid geometry — there is no terrain collision and minimum zoom is `4.0`, so `Q` held down
against a rock face puts you there. The review confirmed this with an executed reproducer and it is
carried with a `// NOTE:`, as the task specified. Zoom in until you are inside rock, point at a
wall, and **record what it does** — this is a known limitation being observed, not a bug hunt.

---

## 5. The fps readings — AC19 / NFR6

`F3` toggles the overlay. **It cannot be read from a capture**: capture mode forces the overlay
off (`force_capture_overlay_off`), which is deliberate — the overlay would be in the PNG. So this
reading is interactive-only, with the input path live.

| Where | Floor | Read |
|---|---|---|
| Working zoom (`Q` in to roughly the 8.1 working framing) | **60 fps sustained** | |
| Full vista (`E` out to the boot framing, distance 90) | **≥30 fps sustained** | |

*Sustained*, not a peak — let it settle, and drag while you watch it, because the point is NFR6
holding **with the input path live**. 8.1 read >140 at both against these same floors, so a
collapse here would be new and would be the finding.

**A failed reading is the result and gets reported, not worked around.**

---

## 6. The instruments end to end — AC15 and AC16

These are the two ACs whose *headless* half is done and whose live half has never run.

**AC15 — a scripted drag through the real press/drag/release path.** Coordinates are **viewport
pixels** in the default 1280x720 window. One tile step spans **48.8 px at `--distance 30`**
(measured at the 2026-08-21 review), so the rect below is roughly 3x2 tiles.

```bash
gui.exe 7451 --capture 8-2-drag-dig.png --at-tick 3 --z 10 --distance 30 \
  --drag dig,600,340,745,437
```

Then a channel and a stockpile, changing only `--drag`:

```bash
gui.exe 7451 --capture 8-2-drag-channel.png   --at-tick 3 --z 10 --distance 30 \
  --drag channel,600,340,745,437
gui.exe 7451 --capture 8-2-drag-stockpile.png --at-tick 3 --z 10 --distance 30 \
  --drag stockpile,600,340,745,437
```

**Keep `--at-tick` low.** Channels decay to zero by ~114 ticks (§2); `--at-tick 3` is three ticks
after start and cannot be caught by that.

**Flag rules the review put in, worth knowing before you fight the parser:**

- `--cursor` and `--drag` are **mutually exclusive** and now `bail!` — previously `--cursor` was
  silently ignored while the capture still asserted the pick against it, a guaranteed spurious
  failure with a misleading message.
- `--capture` requires `--frames N` **or** `--at-tick N`. With `--at-tick` and no `--frames`, the
  frame budget defaults to **1500**.
- `--expect-work`, `--distance`, `--cursor`, `--at-tick` and `--drag` all require `--capture`.
- **`--drag stockpile` needs standable ground under those pixels**, for the same silent-drop reason
  as §3 step 3.
- **`--drag clear` asserts nothing about work** — a correct clear legitimately ends with nothing to
  count, so `assert_drag_produced_work` has an empty arm for it. A clear capture therefore proves
  strictly less than the other three. Take one if you like, but do not read a pass as evidence.

**What the output must say.** Match by **prefix**, never whole line — 7.1 changed the draw-set
oracle's shape and older recipes quoting the full line stopped matching.

| Line | Requirement |
|---|---|
| `slice: z N projected T terrain cubes (C of E cut-face tiles at z N)` | `C == E`, `T > 0` |
| `marks: z N designations=D of X zones=Z of Y` | **`D == X` and `Z == Y`** (projected == mirror), and for a dig/channel drag **`X > 0`**; for a stockpile drag **`Y > 0`** |
| `motion: ticks observed=.. dwarf position changes=P mid-blend frames=M ..` | `P > 0`, `M > 0` — the review restored these on the `--at-tick` path, where they had been silently dropped |
| `capture range check: warm-lit pixels=W ground-median-luminance=G` | `W ≥ 3000`, `G` in `[70, 180]` |

A `capture range check: the cut at z N is below the world top ...` line means the band assertions
**skipped**. That is expected at `--z 10` and is not a pass.

**Exit 0 is not a result. Read the numbers.** Three separate defects on 7.2 produced exit 0 while
showing nothing they claimed to show, and 8.2's own review found two more of the same shape.

**AC16 — the exhaustion half, which must be demonstrated, not assumed.** Until the review patch
`app.run()`'s exit status was discarded and an exhausted budget exited **0**. Force it:

```bash
gui.exe 7451 --capture 8-2-should-not-exist.png --at-tick 100000 --frames 30 --z 10
echo "exit=$?"
```

**Expect `exit=1`**, a stderr line reading `capture --at-tick 100000 did not reach tick 100000
within 30 frames; reached tick N`, **and no PNG written**. `exit=0` here is the AC16 defect
reopening, and it is invisible to every other step in this file.

---

## 7. The independent confirmation — AC18

AD-17 rung 1: a **different client** on the **same daemon** confirms the sim actually received what
the mouse designated. Not a second reading of the same mirror — a second process.

Right after the §3 dig drag (before it decays far), from WSL:

```bash
./target/debug/tui 7451 --frame --z 10 | tee 8-2-tui-crosscheck.txt
```

Count its own mark glyphs and **range-check the count** — assumed is not confirmed:

```bash
python3 - <<'PY'
t = open("8-2-tui-crosscheck.txt", encoding="utf-8").read()
counts = {"dig ×": t.count("×"), "channel ▼": t.count("▼"), "zone ≡": t.count("≡")}
print(counts)
PY
```

The glyphs are `×` dig, `▼` channel, `≡` zone (`tui/src/palette.rs:102-120`). **State the expected
count before you read it** — a dig rect of `w x h` tiles is `w*h` designations minus whatever the
dwarves have already consumed — and record both numbers. A count of zero for a kind you just
dragged means the command never reached the sim, whatever the Bevy client is drawing.

---

## 8. What only your eye can settle

In this order; the first is the one most likely to be wrong.

1. **The hover slab on a vertical face** — §4. On the face, standing up, right cube. This is the
   whole reason AC13 exists.
2. **Preview vs commit.** Drag a channel and watch the slab: does the preview sit and read as what
   it becomes, or does it jump height/colour on release? The patch says it should not.
3. **Four things or three?** Dig `(56,132,250)`, channel `(150,96,230)`, zone `(40,120,150)`, hover
   `(80,220,210)`. The hover teal and the zone slate-teal are the closest pair on the board.
4. **The hint bar** — legible, ASCII, no replacement boxes, present in every frame.
5. **The gutter between neighbouring slabs** is ~2 px at `--distance 30` and ~0.65 px at the boot
   vista. Separate tiles, or anti-aliasing noise? Carried open since 2026-08-21.
6. **Does dragging feel like giving an order?** Press, drag, release, and the fortress obeys. That
   is the beat this story exists for, and no instrument can read it.

---

## 9. Recording it

- Paste the **build commit and build wall-clock** from §1 into the Dev Agent Record. That is M2-7's
  fifth occurrence being worked around by hand; say so.
- Paste the `slice:`, `marks:`, `motion:` and `capture range check:` lines from each §6 capture.
- Paste the `exit=` from the AC16 exhaustion run.
- Paste the §7 glyph counts **and the expected count you stated first**.
- Record the two fps readings from §5, or record that a reading failed.
- Answer §8 in words — especially 1, 2 and 3.
- Tick Task 7's boxes **only for what was actually observed.**

Then AC13, AC15, AC16, AC18 and AC19 close, 8.2 moves to `done`, and the PR is Wolf's call.

---

## What NOT to do

- **Do not re-tune any look constant to make something read better.** One look change is in scope
  for this whole story — the hover highlight moving to the hit face — and it is already made. The
  art rule stands: the bar for a look change is a concrete defect. If a colour reads wrong, record
  it open; 6.2's campfire was carried open for exactly this reason and it was the right call.
- **Do not trust a green exit.** See §6.
- **Do not fix a stockpile that keeps nothing by moving the rect until it works** without first
  checking whether the ground under it was standable. Silent-drop and broken-path look identical.
- **Do not read fps from a capture** — the overlay is forced off there.
- **Do not push.** The branch is pushed to date; the PR waits on Wolf's explicit yes.

---

## Still absent after this session, whatever it finds

`codex review --base main` **never ran for this story** — killed twice by the harness,
quota-blocked once (8.2 took the weekly Codex quota from 0% to 100%). That is an **absence, not a
clean pass**, and this session does not close it.
