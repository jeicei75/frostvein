# 13.9 seat card: forage (AC 15)

One run, one question. A fresh game, on the branch `story-13-9-forage`.

## What changed

The world grows berry bushes on snow (about 1 in 16 snow columns; a fresh 256² game has about 1,900).

**Since run 1 (Amendment 1, your three points):**

- **Haulers forage, on trips.** A free hauler goes to the nearest marked bush first. After each bush it goes
  on to the nearest other marked ripe bush within 10 cells, up to 5 bushes. Then it carries ONE berries load to
  the pile (the load's count is the number of bushes; it takes one pile cell). Woodcutters no longer forage.
- **Marks work like cut.** Key 6 marks only the RIPE bushes in the drag, nothing on bare ground or on growing
  bushes. A drag with no ripe bush is refused: "forage refused: no ripe bush, or the mark limit is reached". A
  mark is used up when its bush is picked. A bush regrows after 3,000 ticks (5 minutes at Normal, 75 s at
  fast4x) unmarked; mark it again to forage it again. Clear (key 4) removes marks.
- **Point 3 (idling):** I could not reproduce it in a probe (the woodcutter never idled more than 1 tick between
  bushes). "Not the nearest" did reproduce, and nearest-first answers it. If you see a hauler idle beside marked
  ripe bushes, that is the case to describe.

Nothing else a dwarf does changes: every committed scenario ends as it does on `main`.

The look, plain:

- **Ripe bush:** a low dark-green block with red berry dots. **Growing bush:** the green block only, smaller.
- **Marked bush:** the same bush glowing amber (the cut mark's idea; in the tui an amber `&`).
- **Berries:** a small red cube, carried by the hauler and then lying on the pile.

## The run

```bash
# WSL, from the repo, at the branch HEAD
cargo build --release -p simd
target/release/simd 7451
```

```powershell
# PowerShell, from D:\Workspace\frostvein
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-9-seat-run2.csv')
```

1. Make a pile near the camp (key 3).
2. Press 6, drag over a patch with several bushes near the camp, and release. Only the ripe bushes glow.
3. Watch a hauler go to the nearest marked bush, then on to the next ones, and carry one load to the pile.
4. Try a drag over bare ground with no bush. It should be refused.
5. Switch to fast4x and wait about 75 s: a picked bush regrows its berries but is not marked. Mark it again
   and a hauler comes back.

The perf log is compared with 13.8b seat run 5 (`13-8b-signoff/13-8b-seat-like-run5.csv`: p50 47-59 fps by
3-minute bucket, at 256²) for NFR13.

## Question

**Does foraging read now: can you see the haulers sweep the marked bushes nearest-first and carry the berries
to the pile?**

Your answer in your own words, please. It goes into the story as you say it. A look defect named here is
fixed in this story if it is concrete; anything else goes to an issue.

## Sitting

**Run 1, 2026-11-22, build `a192ab88` (Wolf).** His words: "hmmm... maybe few things to fix.. 1) haulers could
probably do foraging also. No need to forage and haul separately but collect some amount and carry then to pile
2) forage are stays after selection fully even rejected area .. should work like cutting 3 there might be some
issues were woodcutter did not forage the nearest bush and actually even though there were forageable bushes just
continued idling .."

**Run 2, 2026-11-22, build `48aba96a` (Wolf).** His words: "well dragging works only in the surface level it was
initiated". Cause: R3 sent forage like cut, one `Designate` on the cut-target level, and a bush is one cell tall, so a
drag missed every bush up or down a slope. Fixed: still ONE command (one refusal at most), now spanning every level
the followed ground touches; the preview follows the ground too.
