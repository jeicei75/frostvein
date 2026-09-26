# Story 8.3 vehicle card — Master of Time, and the Skeleton Walks in 3D

Run `simd` and `tui` in WSL. Run the client from PowerShell in `D:\Workspace\frostvein`; the
launcher checks that its built client matches HEAD. **The daemon writes `frostvein.save` in the
directory you start `simd` from.**

## 1. Two clients, one daemon

WSL, terminal 1 (a fresh daemon at tick 0), and terminal 2:

```text
simd 7451
tui 7451
```

PowerShell:

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

**Look first:** the hint at the bottom-left of the gui now reads
`1 dig  2 channel  3 stockpile  4 clear   Space pause  +/- speed  Ctrl+S save  Ctrl+L load`.
Is all of it inside the window? (Captures hide the UI, so only the seat can check this.)

## 2. The walking skeleton, end to end (AC6)

In the **gui**:
1. Press `1` and drag a **small** dig of 2–3 tiles into a hillside near the camp.
2. Press `3` and drag a **small** stockpile of 2–3 tiles on flat ground beside it. Press `Esc` to leave the mode.
3. Press `+` to go to Fast.

**Keep both small.** Hauls queue behind digs: at creation, 174 dig marks held back the first
delivery for thousands of ticks. With a small dig the stone should land within a minute at Fast.

Watch a dwarf dig, pick up the stone, and carry it onto the pile. **In the tui**, the same stone
glyph should appear on the same stockpile. (The tui follows the same world; its z-level may
need `<`/`>` to reach the pile.)

## 3. Save, change, load (AC1, AC6)

1. `Ctrl+S`. The **simd** terminal prints `saved tick N to frostvein.save`. The camera must not move.
2. Designate another dig and let it get dug.
3. `Ctrl+L`. Both clients **snap** back to the saved world at once, with no animated rewind:
   the new dig is undug and the dwarves are where they were at the save.

## 4. The sign-off (AC7)

Restart for a clean boot: stop the gui, restart `simd 7451`, relaunch the gui.
1. **The boot frame, on looks alone:** is it a wow?
2. **The alive moment, about 30 s later,** with a dig and a stockpile running: is it a wow?
3. Is any of these true of this client: **ugly, flat, cluttered, confusing, lifeless, camera unusable?**

## Questions for Wolf
1. Does the hint fit, and does it tell you the keys you need?
2. Did the stone reach the stockpile, in both clients?
3. Did `Ctrl+S` save (simd line) without moving the camera, and did `Ctrl+L` snap both clients back?
4. Both wow beats: yes or no, each.
5. Any of the six words true?
