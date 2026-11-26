# 13.10 seat card: hungry and miserable (AC 12)

Two short runs on the branch `story-13-10-hungry-and-miserable`: a fresh game, then a world with no food.

## What changed

Dwarves get hungry. In the fresh game hunger is on, and it rises one point a tick from a staggered start.

- **Hungry** (from 12,000: 20 min at Normal, 60 s at fast4x). A dwarf with empty hands leaves his work to eat.
  He eats a stored berries item on the pile first. With none stored, he eats at the nearest ripe bush that
  nobody has marked. He stands at it for 20 ticks and then goes back to work. A bush he ate from regrows,
  like a foraged one. His left job stays queued, and someone does it later. A dwarf carrying something
  finishes the carry first.
- **Miserable** (from 18,000: 10 min later at Normal, 30 s later at fast4x). Only when no food can be reached.
  A miserable dwarf drops his trade work and takes no new work until he has eaten.
- **One berry is one meal.** A berries load of 5 feeds five dwarves, one at a time, and is gone at 0.

Every committed scenario ends as it does on `main`. Hunger is off there, and it is on only in the fresh game and
the 256² bench.

The look, plain (your Q5 and Q6 rulings):

- **gui:** a miserable dwarf is drawn leaning forward (20°) and a little sunk (0.05 cells), over any animation. It
  pops on and off with no ease. Click him and the panel reads `miner miserable` (or `miner hungry`). The gui's
  terminal prints `gui dwarf N miserable on` / `off` at each change.
- **tui:** a miserable dwarf's `☻` turns cold grey-blue instead of his tunic colour, and a hungry one keeps his
  tunic. The roster adds `hungry` or `miserable` after the trade. While anyone is hungry, the status line adds
  `hungry H  miserable M`.

## Run 1: the fresh game (hungry dwarves eat)

```bash
# WSL, from the repo, at the branch HEAD
cargo build --release -p simd
target/release/simd 7451
```

```powershell
# PowerShell, from D:\Workspace\frostvein
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-10-seat-fresh.csv')
```

1. Make a pile near the camp (key 3). Mark nothing else yet.
2. Speed up to fast4x. Within about a minute every dwarf turns hungry one by one. Each walks to a ripe
   bush, stands there a moment, and goes back. That bush loses its berries and regrows about 15 s later.
3. Press 6 and drag over a few ripe bushes near the pile. Haulers carry the berries to the pile. When the next
   dwarves get hungry, they eat at the pile instead, and the load shrinks and then goes.

## Run 2: no food anywhere (miserable dwarves)

```bash
# WSL: Ctrl+C run 1's daemon first
target/release/simd 7451 --scenario _bmad-output/implementation-artifacts/13-10-signoff/no-food.json
```

```powershell
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-10-seat-nofood.csv')
```

The world is your 12.12 seat-slow save with no bushes and no berries. Its five dwarves are a few hundred ticks short
of miserable. It runs at fast4x once the gui connects. Within about 3 to 11 s each dwarf turns miserable, one at a
time: he leans forward, stops his work, and stays that way. There is nothing to eat.

The perf logs sit beside 13.9's seat reading for NFR13.

## Questions

**Q1. Does the slump read: can you tell a miserable dwarf from a working one at your usual zoom?**

**Q2. The tui (optional, `cargo run --release -p tui` in a second WSL window during run 2): does the cold `☻` and the status count
read?**

Your answers in your own words, please. They go into the story as you say them. A concrete look defect named
here is fixed in this story; anything else goes to an issue.

## Sitting

(none yet)
