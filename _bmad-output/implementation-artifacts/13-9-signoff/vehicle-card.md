# 13.9 seat card: forage (AC 15)

One run, one question. A fresh game, on the branch `story-13-9-forage`.

## What changed

The world now grows berry bushes on snow (about 1 in 16 snow columns; a fresh 256² game has about 1,900).
Woodcutters forage them inside an area you mark, the berries are hauled to a pile like any item, and a
foraged bush regrows after 3,000 ticks (5 minutes at Normal, 75 s at fast4x). Nothing else a dwarf does
changes: every committed scenario ends as it does on `main`.

The look is plain, as drafted at Task 0 Q5:

- **Ripe bush:** a low dark-green block with red berry dots.
- **Growing bush:** the green block only, smaller.
- **Berries:** a small red cube beside the bush they fell from.
- **Forage area:** a berry-coloured slab on its cells (a pile under it still reads as a pile).

Key **6** is the forage mode. Drag over the bushes and release to mark. Esc leaves or aborts. Clear (key 4)
removes forage cells too. An area with no bush in it is refused: "forage area refused: no plant in it".

## The run

```bash
# WSL, from the repo, at the branch HEAD
cargo build --release -p simd
target/release/simd 7451
```

```powershell
# PowerShell, from D:\Workspace\frostvein
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-9-seat.csv')
```

1. Make a pile near the camp (key 3).
2. Press 6, then drag over a few bushes near the camp and release.
3. Watch a woodcutter forage a bush: the berry dots go and a red cube appears beside the bush.
4. Watch a hauler take the berries to the pile.
5. Switch to fast4x and wait about 75 s for the bush to regrow its berries and be foraged again.
6. Also try a drag over bare ground with no bush. It should be refused.

The perf log is compared with 13.8b seat run 5 (`13-8b-signoff/13-8b-seat-like-run5.csv`: p50 47-59 fps by
3-minute bucket, at 256²) for NFR13.

## Question

**Does foraging read: can you see a bush being picked, the berries carried off, and the bush coming back?**

Your answer in your own words, please. It goes into the story as you say it. A look defect named here is
fixed in this story if it is concrete; anything else goes to an issue.

## Sitting

**Run 1, 2026-11-22, build `a192ab88` (Wolf).** His words: "hmmm... maybe few things to fix.. 1) haulers could
probably do foraging also. No need to forage and haul separately but collect some amount and carry then to pile
2) forage are stays after selection fully even rejected area .. should work like cutting 3 there might be some
issues were woodcutter did not forage the nearest bush and actually even though there were forageable bushes just
continued idling .."
