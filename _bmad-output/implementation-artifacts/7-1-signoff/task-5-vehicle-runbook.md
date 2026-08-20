# Task 5 — the live vehicle session (story 7.1, slice into the mountain)

Everything in story 7.1 except this session is done, reviewed, patched and green: 90 tests, 13 of 13
mutations killed, `GATE GREEN`.

It closes **ACs 8, 9, 10, 12 and 14**. AC1 and AC15 are your sign-off and no command here can close
them.

**Build and daemon:** see `../vehicle-session-runbook.md`. Do 6.1 first — this story needs that dig
to exist before there is anything worth slicing down to.

**What has never been observed:** all of it. The slice was measured against a live daemon — the cut
face is genuinely filled, 15,316 of 16,071 floor tiles come from the cut-face rule alone — but no
human has seen a single pixel of it. That is what this session is for.

---

## 1. Boot straight to the dig

```cmd
gui.exe 7451 --z 9
```

`--z` no longer requires `--capture` (changed at code review 2026-08-19; it previously refused, which
would have made this line error out). Without it you would boot at z 31 and need 22 keypresses.

**The keys are `,` and `.`** — comma steps the cut **down**, period steps it **up**. These are the
physical keys behind `<` / `>`, so shifted works too. Nothing on screen names them.

📋 **AC10 — the one ruling I need from you.** The readout says `surface` or `underground`. It now
decides that by asking whether any rock sits above the cut, not by where the cut is. **Known
residue:** at z 30 the world's 17-cube peak still counts as "above", so it reads `underground` while
the picture is indistinguishable from the surface. Tell me whether that reads right or wants a
threshold — it is one line either way.

## 2. AC8 — look into the dig

Slice down to **z 9** and look at the site at `[55,62,9]`–`[56,65,9]`.

📋 **State plainly:** the dug tiles read as an **excavation seen from inside the mountain** — a floor
you are looking down onto — rather than a hole punched in a surface. This is the whole point of the
story: 6.1's dig is one voxel deep and barely read; slicing is what is meant to make it legible.

## 3. AC9 — the readout

📋 **State plainly:** the level is readable at the boot framing, and it is **not** hidden behind the
F3 overlay. The review found it drawn underneath the overlay and moved it below the corner; this
session is where that fix is confirmed by eye.

## 4. AC12 — the capture

```cmd
gui.exe 7451 --capture 7-1-slice.png --frames 1500 --z 9
```

📋 **Paste back** the `slice:` line, the `lantern:` line, the `motion:` line and the range-check
line, verbatim. Expected shape:

```
slice: z 9 projected 36788 terrain cubes (16071 of 16071 cut-face tiles at z 9)
```

**Both cut-face numbers must match.** They are counted independently — one from what was drawn, one
from what the world says is there. If they differ, the cut has been drawn hollow, and that is a real
defect regardless of exit code. **Exit 0 is not a result.**

Every number prints before any assertion, so paste them even on a failure — especially then.

## 5. AC11 — two clients, two levels, one daemon

With `gui.exe` still at z 9, in WSL:

```bash
./target/debug/tui 7451 --z 4 --frames 30
```

📋 **Confirm in words:** the TUI shows level 4 while the Bevy client sits at level 9, from the same
running daemon, and neither disturbs the other. The slice is client-local view state and never
touches the wire.

## 6. AC14 — the NFR6 reading

Press **F3**. Read sustained fps **at a slice level, not at full depth**, at:

- **working zoom** — bar is **60 fps**
- **full vista** — bar is **≥30 fps**

📋 **Paste back both, labelled `gingerspice / native Windows / NVIDIA`.**

Slicing only ever *reduces* the draw set — measured across all 32 levels, never above the full-depth
53,365 — and 6.1 measured >143 fps, so there is large headroom. If a reading fails anyway, that
measurement is the finding and gets reported, not worked around.

---

## The control ruling is still yours to make

`,` / `.` is marked **PROVISIONAL** in the story. You have never chosen it. The mousewheel is
unclaimed in code today, so the wheel is available — it just costs a migration when UX-DR2 brings
wheel zoom. Rule it at the viewing, when you can feel both.

## What comes back to me

The `slice:` line and its three companions, the two fps figures, your AC10 ruling, the control
ruling, and your sentences from §2, §3 and §5.
