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

## Run 3: seat-like again, after the claim_jobs skip (AC 14)

This is the same scenario as run 2, which is your seat run in miniature: a few trees and a little digging at t6,
then 1,500 digs at t17,000 (crew 200). Headless, the fix changes nothing the sim does (the same state hash),
and before the big dig the tick falls from about 10 ms to about 2.7 ms.

```bash
# WSL, from the repo, at the branch HEAD
cargo build --release -p simd
target/release/simd 7451 --scenario _bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-8-seat-like-run3.csv')
```

**What to expect.** Up to the big dig (tick 17,000), the stutter that grew with the haul backlog in run 2 should
be gone. **After the big dig, it stays slow:** headless, the tick is still about 50 ms from about tick 20,000
(failing claim searches and `execute_jobs`; your ruling, 2026-11-05: issue #231, not 13.8). The gui's
crowd cost at 200 (#224) is unchanged too. Keep the zoom at one framing if you can, so the perf log's buckets
compare with run 2's.

**R3 question:** before the big dig, does it run smoothly now, where run 2 stuttered?

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

## Sitting

2026-11-04, Wolf, fresh game (`simd 7451`, release), `71a7390c`.

- **Q1, Wolf: "yes".** Groups appear at the edge and walk in.
- **Q2, Wolf: "yes".** Selecting an arrival names him in full.
- **Q3, Wolf: "yes the current list is too big to fit ofc so we need to have story for hud and ui".**
  At around 200 the roster under the clock does not fit. Not part of 13.8 (AC 13).
- **Q4, Wolf: "I think it's ok now.. ofc new groups are coming too fast atm but it helps testing...
  and damn it looks cool when new dwarves arrive especially during night..."** The constants stay:
  `ARRIVAL_PERIOD = 1_000`, 5–20 per group, `CREW_CAP = 200`.
- **Unprompted, Wolf: "still performance issue .. all was ok as long as I cut down a few trees and
  digged a little bit ..when in the end I designated bigger dig are again stuttering started".** No
  measurement was taken at the sitting.
- **Then, Wolf: "it's not just digging issue but eventually after a while it starts to slow down and
  stutter and slow down (also fps) and it happens even without designations".** So the slowdown grows
  with run time, and the gui frame rate falls too, with or without a dig. In this run the crew also
  grows from 5 to 200 over the same span, so time and crew size are confounded. Candidates are #225
  (tick overruns its period at 200) and #224 (gui ~40 fps at 200), but a fall in fps over time was
  flat at 13.6's sitting (5-minute buckets p50 23.8–29.7 ms). The 13.8 bench shows no tick regression
  on two-hundred-on-256 (mean -0.2%, Debug Log). Unmeasured; the cause is not established.
- **Then, Wolf: "zooming to 200 dwarf nest will take fps to knees 6pfs or something".** Close framing on
  the camp crowd at 200, unmeasured (no perf log). #224 measured working zoom at ~40 fps on
  two-hundred-on-256, where the crowd is spread; this reading is the packed camp. Wolf: "performance issue
  we need to solve". Q3's crew list is #230.
