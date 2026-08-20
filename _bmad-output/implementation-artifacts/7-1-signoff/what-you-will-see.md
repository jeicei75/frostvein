# Story 7.1 sign-off artifact — "here is what you will see" (UX-DR22 opening half)

**Status: WRITTEN HALVES DRAFTED 2026-08-18, AWAITING WOLF.** Part (a), the before capture, is
**owed** — one vehicle session, no code (the shipped 6.1 binary takes it). Until Wolf approves this
file as a whole, AC1 is unmet and no implementation may start.

**This file also carries a decision that is better ruled now than at the viewing: which control
drives the slice.** See the end.

## (a) The before capture — OWED, one command on the vehicle

```cmd
gui.exe 7451 --capture 7-1-before.png --frames 1500
```

What it shows: the valley from outside, as a solid surface. Everything below the skin of the world
is invisible and unreachable — including the dig, whose excavation is one voxel deep and, as your
6.1 viewing found, barely reads at all.

## (b) What this story adds

**You can cut down into the mountain a level at a time, and you always know which level you are on.**

Choose a level and everything above it stops being drawn. The mountain opens up: where you were
looking at a snow surface, you are now looking down into the rock, and the **floor of the cut** —
tiles that were buried solid a moment ago — becomes visible as a flat face. Slice down to where the
dwarves have dug and you see the excavation from above and inside rather than as a scratch on the
surface. The current level is readable on screen at all times, without turning on the diagnostic
overlay.

The aim: the underground stops being a thing you infer from dwarf behaviour and becomes a place you
can look at.

## (c) What you will NOT see

Each line needs your ruling.

1. **No cutaway shading, no cross-section hatching, no special cut-face material.** The floor of the
   cut is drawn in the same terrain material as everything else. A distinct "you are looking at a
   cross-section" treatment is a story of its own.
2. **No designation or zone rendering (7.2).** Slicing down to a dig shows you the *tiles* the
   dwarves have removed, not the marks you made to order it.
3. **No commands or picking from `gui` (8.x).** You still designate from a TUI client.
4. **No deeper digging.** Slicing changes what you *see*, not what has been dug. The 6.1 excavation
   is one voxel deep because a designation covers one z-level, and that does not change here — you
   will be looking into a shallow cut, more legible than before but still shallow.
5. **The boot frame is unchanged EXCEPT for one new line of text.** The client still starts at the
   top level, so the 3D composition you approved at 5.4 is exactly as it was — but AC9 requires the
   level always be readable, so a permanent readout (`Slice: z 31/31 — surface`) now sits at the
   top-left in pale blue, below the F3 overlay's corner. **It is not suppressed in capture mode**,
   so it will appear in `7-1-slice.png` and in every future 5.4/6.1/6.2 capture. *(Corrected at
   code review 2026-08-19: this line previously claimed the boot frame was wholly unchanged, which
   contradicted the code you are being asked to sign off. Verified harmless to the inherited 5.4
   range checks — the readout is blue-dominant, so it cannot register as a warm lantern pixel, and
   it sits outside the ground-luminance sample region.)*
6. **Dwarves remain scaled cubes.**
7. **Entities above the cut are hidden** along with the terrain, so nobody floats over an opened
   mountain. *(If you would rather see surface dwarves while looking underground, say so — it is a
   ruling either way, and the story tests whichever you pick.)*

## The keys, so you are not told them out of band

`,` steps the cut **down** one level, `.` steps it **up**. These are the physical comma and period
keys, which is what makes `<` / `>` work — but the unshifted keys step too, so the binding is a
superset of the recorded `<` / `>` ruling. Nothing on screen names them; this list is where you
find them until the ruling is confirmed.

The world is 32 levels deep and the client boots at z 31. The 6.1 dig site is at **z 9**, which is
22 taps away, so at the viewing use `gui.exe <port> --z 9` to boot straight to it — `--z` no longer
requires `--capture` (changed at code review 2026-08-19; it previously refused to run without one,
which would have made this recipe error out).

## The control — rule this now, not at the viewing

The epic says this story resolves an "open control collision" because "the mousewheel is already
claimed by the zoom continuum". **Checked against the code: it is not.** `gui` binds **no mouse
input at all** today — no wheel, no buttons, no cursor — and camera zoom is on the **`Q`/`E` keys**.

So the collision is *planned*, not *implemented*, and the choice is:

- **`<` / `>` keys** — matches `tui --z N`, which you already use to designate; costs nothing now
  and nothing later. **Recommended.**
- **Modifier + wheel** — available today because the wheel is free, but it has to move when UX-DR2's
  zoom continuum takes the wheel, and then you learn it twice.
- **Slice follows selection** — no key at all, but `gui` has no picking until 8.x, so there is
  nothing to select with.

The story requires the ruling and the reasoning to be recorded, not just a working control.
