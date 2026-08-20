# Task 6 — the live vehicle session (story 6.2, lanterns in the dark)

Everything in story 6.2 except this session is done, reviewed, patched and green — 12 review patches
applied, sabotage table extended to cover them.

It closes **ACs 9, 13, 14 and 15**. AC17 is your separate sign-off (Task 9) and no command here can
close it.

**Build and daemon:** see `../vehicle-session-runbook.md`. Run this after 6.1, before 7.1 — same
camp, same daemon, nothing new to set up.

---

## 1. The lantern capture

```cmd
gui.exe 7451 --capture 6-2-lantern.png --frames 3000
```

**`--frames 3000`, not fewer.** The instrument needs the lit region to have *moved*, which needs the
dwarves to have walked; 3000 frames is the figure this story's measurements were taken at.

📋 **Paste back all four printed lines**, verbatim — the `slice:` line, the `lantern:` line, the
`motion:` line and the range-check line. Expected shape:

```
lantern: dwarf positions observed=[..]  lit terrain tiles at dwarf positions=>0  moved=true
motion:  ticks observed=>=100  dwarf position changes=>0  mid-blend frames=>0
```

**Exit 0 is not a result.** Every number prints before any assertion, so paste them on a failure too
— that is exactly the run whose numbers are needed.

**One change from the code review of 2026-08-19 you should know about.** The lantern assertions used
to be skipped whenever no lantern was observed, which could not tell "the slice hides the dwarves"
from "lantern projection is broken entirely". They now ask the world whether a dwarf actually sits at
or below the cut. At full depth this changes nothing — but it means a silent lantern regression can
no longer pass, which is the point.

## 2. AC13 — the NFR6 reading

Press **F3**. Read sustained fps **with all five lanterns moving**, at:

- **working zoom** — bar is **60 fps**
- **full vista** — bar is **≥30 fps**

📋 **Paste back both, labelled `gingerspice / native Windows / NVIDIA`.** 6.1 measured >143 fps at
both, and five moving point lights is the only addition, so a failure here would be a genuine
finding rather than an expected cost.

## 3. AC9 + AC14 — confirm by eye, in words

📋 **State plainly:**

- a **warm pool travels with each dwarf** and lights the terrain it passes over — it is not a glow
  stuck to the dwarf cube;
- the pools are **distinguishable from the static torches and campfire** rather than merging into
  one wash;
- **the camp does not read blown out** against the 5.4 frame you approved.

The last one is the real risk. Five moving lights were added to a scene that already had five static
emitters, and the range check only guards the ground median inside `[70,180]` — it cannot tell you
whether the camp *looks* over-lit. That judgement is yours and there is no instrument for it.

---

## What comes back to me

The four instrument lines, the two fps figures, and your three sentences from §3. I check them
against ACs 9, 13, 14 and 15, tick what they close, and put the story in front of you for Task 9 —
**the sign-off only you can give.**
