# 13.8 seat card: they keep coming (AC 13, FR57)

Headless, the sim is already proven. `scenarios/they-keep-coming.json` grows a fresh 256² valley from 5 dwarves to
200 and ends at `crew=200 arriving=0`; the scenario tests pin the cadence, the cap, the walk-in and the unique
names. This sitting is for your eye. Do groups visibly come in from the valley edge? Does an arrival read by its
full name? At ~200, does the crew need a list? No measurement is taken.

Pull `story-13-8-they-keep-coming` and rebuild `gui.exe` at its HEAD before you start. The launcher refuses a
stale one.

## The run: a fresh game

```bash
# WSL, from the repo. Release: a debug daemon misses the tick period at 200 dwarves.
cargo build --release -p simd
target/release/simd 7451
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
```

This is a fresh 256² game with crew 5, and arrivals are on. A group of 5–20 appears at a random edge cell
every 1,000 ticks (100 s at Normal), the first at tick 1,000, and walks to the camp. The crew stops at 200,
near tick 17,000. Use `+` to speed up. At Fast4x a period is about 25 s and the whole run is about 7 min.
An arriving dwarf can take a job of his trade on the way in. With no job to take, he walks to the camp. A
walk that waits 100 ticks behind a dwarf who won't move ends where it stands (your ruling, 2026-11-03).

To watch a group come in, zoom out (`Q`, or wheel with `shift`) until the valley edges are on screen just
before an elapsed time that is a multiple of 1:40 (the readout's `elapsed` clock: 1,000 ticks = 1:40 of sim time).

## Questions

1. **Do groups appear at the edge and walk in?** Is the cadence visible: about every 100 s at Normal?
2. **Does selecting an arrival name him in full?** Click (LMB) a dwarf who came in after the start. The slot
   under the clock should read `Name Epithet` (e.g. `Thrain Copperbeard`), then his trade. With nothing
   selected, the roster lists everyone by full name.
3. **At around 200, do you need a crew list (FR57)?** The gui roster is one line per dwarf under the clock.
   The TUI roster is one row cut at the terminal width, and at 100 columns it already shows only 3 of the
   first 5 full names (Debug Log, AC 6). A "yes" becomes an issue or a story, not part of 13.8.
4. **Do the cadence and group size feel right?** They are `ARRIVAL_PERIOD = 1_000` ticks and 5–20 per group,
   with `CREW_CAP = 200`. They are tuning constants, so name new numbers if they feel off.

Answers in your own words, please. They go into the story as you say them.
