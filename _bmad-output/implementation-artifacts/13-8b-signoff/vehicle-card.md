# 13.8b seat card: after the big dig (AC 11)

One run, one question. It is run 3 of 13.8 again (the same scenario), on the branch
`story-13-8b-after-the-big-dig`.

## What changed

Nothing a dwarf does. Three pieces of repeated work in the sim are now done once: a claim search that
already failed is not repeated while the ground is unchanged, the map of loose stones is built once a tick
instead of once per working dwarf, and the pile's free cells are found once per claim pass.

Headless, on seat-like (two idle runs each, same state hash `5511268fb5faf97f`):

| | `main` | this branch |
|---|---|---|
| mean tick, whole run | 20.2 / 20.3 ms | 11.8 / 11.9 ms |
| p99 | 131 / 132 ms | 105 / 107 ms |
| max | 403 / 397 ms | 404 / 406 ms |
| ticks over 10 ms | 11,456 / 11,375 | 5,970 / 6,018 |
| mean tick, t22,000 to t30,000 | 45 to 57 ms | 20 to 30 ms |

**What is not fixed, by your ruling (Task 0 Q1):** dwarves blocking each other (4 to 30 ms a tick, the
largest part right after the dig) and the burst of about 130 re-plans after each dig. The single worst
tick is unchanged (about 400 ms). Expect it to stop and start less, not to be smooth.

## The run

```bash
# WSL, from the repo, at the branch HEAD
cargo build --release -p simd
target/release/simd 7451 --scenario _bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-8b-seat-like.csv')
```

Run at Normal past tick 17,000 (the big dig: 1,500 digs, crew 200), and on to about tick 24,000 if you
can. Keep the zoom at one framing if you can, so the perf log compares with 13.8's run 3
(`13-8-signoff/13-8-seat-like-run3.csv`).

## Question

**After the big dig, does it still stop and start?** In 13.8's run 3 you said "fews secs stop few secs
stops", everyone at once. Is that gone, less, or the same?

Your answer in your own words, please. It goes into the story as you say it, and what remains goes to
the follow-up issue.

## Sitting

**Run 1, 2026-11-13, build `6b6a041f` (Wolf).** His words: "we still have issue"; asked gone, less or
the same as 13.8's run 3: "less but still visible". On closing: "I don't thik we are done with the story".

Perf log: `13-8b-seat-like-first-hour.csv` (the first 3,600 s of a 13,311 s log; the digs end at
3,299 s; the full 27 MB log stays untracked in `13-8-signoff/13-8b-seat-like.csv`, its only mark is
at 13,307 s). Against 13.8's run 3 (`13-8-signoff/13-8-seat-like-run3.csv`), up to 2,689 s where run 3 ends:

| | run 3 (13.8) | this run |
|---|---|---|
| arrivals | every 100.1 to 100.5 s, crew 200 at 1,701 s | every 100.1 to 101.5 s, crew 200 at 1,704 s |
| frames over 250 ms after startup | 0 | 0 (worst 172 ms) |
| tiles dug after the big dig | 894 | 912 |
| pauses over 3 s / over 5 s between digs landing | 56 / 8 | 53 / 7 |
| longest such pause | 8.0 s | 8.0 s |

So the gui does not freeze, and the log cannot show the improvement Wolf saw: it has no tick number,
and "pauses between digs landing" is only a stand-in for the world stopping.

**Devpod wire tap, 2026-11-14, same build** (`scripts/bench/wire_tap.py`, one client, Normal, idle
machine; `wire-tap-devpod-6b6a041f.csv`). The daemon does NOT stop and start here: every tick 0 to
24,500 arrives in order, and the longest silence between two ticks is 0.38 s.

| ticks | median gap | p99 | longest | over 0.5 s |
|---|---|---|---|---|
| 0 to 17,000 | 100 ms | 107 to 117 ms | 175 ms | 0 |
| 17,000 to 18,000 | 100 ms | 138 ms | 164 ms | 0 |
| 18,000 to 20,000 | 101 ms | 197 ms | 382 ms | 0 |
| 20,000 to 22,000 | 100 ms | 217 ms | 377 ms | 0 |
| 22,000 to 24,500 | 101 ms | 217 ms | 376 ms | 0 |

The daemon falls 10 s behind real time over the 7,500 ticks after the dig, in steps of at most 0.4 s.
The message per tick grows from about 81 KB to 133 to 144 KB at tick 17,000 and stays there (not yet
looked into). Not known: whether the seat's daemon (WSL, the gui beside it) is as even. Run 2 asks that.

## Run 2: where do the seconds go

Same run, with the tap as a second client on the Windows side. Start it AFTER the gui is up (the
scenario's clock starts at the first client).

```bash
# WSL, from the repo, at the branch HEAD
target/release/simd 7451 --scenario _bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-8b-seat-like-run2.csv')
# a second PowerShell window, once the gui shows the world
python scripts\bench\wire_tap.py D:\Workspace\frostvein\.bin\13-8b-wire-tap-seat.csv
```

Run at Normal to about tick 20,000 (about 35 minutes), or until you have seen it stop and start a few
times after the big dig. When you see a stop, press the mark key so the perf log has the moment. Ctrl+C
the tap at the end. Both files, please.

**Reading it:** ticks arriving evenly while the world visibly stops means the cause is in the gui;
ticks arriving in clumps of seconds means the connection or the seat's sim.

### Sitting, Run 2

**2026-11-15, build `19bab45a` (Wolf).** His words: "it's better but after 21 elapsed dwarwes started
to stop briefly (not all at once).. so I just wonder is it actually perf issue or more like decision
what next takes longer when more dwarves around.. it does not look smooth enough still". Asked whether
to close the story on its perf result or widen it: "I think we will continue until we are happy".

Files: `13-8b-seat-like-run2.csv` (gui, 3,115 s, one mark at 3,111 s) and `13-8b-wire-tap-seat.csv`
(the tap, tick 227 to 30,000).

**The gui does not freeze.** Arrivals land every 100.1 to 100.5 s up to crew 200 at 1,700 s. Minutes
15 to 27 have no frame over 100 ms (median 21 ms). After the big dig the median frame is 27 to 38 ms,
25 frames over 100 ms in 24 minutes, the worst 124 ms.

**The seat's daemon, by the tap** (gap between two ticks, 3-minute windows):

| minutes | ticks | median | p99 | longest | over 0.25 s | over 0.5 s | ticks/s | KB/tick |
|---|---|---|---|---|---|---|---|---|
| 0 to 27 | 227 to 16,400 | 100 ms | 103 to 168 ms | 308 ms | 3 | 0 | 9.97 to 9.99 | 46 to 79 |
| 27 to 30 | to 18,197 | 100 ms | 154 ms | 228 ms | 0 | 0 | 9.98 | 116 |
| 30 to 33 | to 19,954 | 100 ms | 222 ms | 433 ms | 9 | 0 | 9.76 | 137 |
| 33 to 36 | to 21,625 | 100 ms | 335 ms | 774 ms | 44 | 1 | 9.29 | 139 |
| 36 to 39 | to 23,119 | 101 ms | 476 ms | 967 ms | 99 | 12 | 8.30 | 141 |
| 39 to 45 | to 26,609 | 100 ms | 229 to 253 ms | 530 ms | 33 | 2 | 9.61 to 9.78 | 143 to 145 |
| 45 to 51 | to 29,764 | 101 ms | 320 to 348 ms | 774 ms | 139 | 4 | 8.62 to 8.91 | 147 to 149 |

So at minute 21, where Wolf saw the brief stops begin, the daemon is even (no gap over 227 ms) and the
gui is at its best: those stops are not a performance problem. After the big dig there IS a smaller
performance part: single ticks of up to 0.97 s (the devpod's worst was 0.38 s), with the world running
at 83 to 98% of real time. No multi-second silence anywhere.

**What the stops are: dwarves standing still inside the sim** (headless, `prototypes/stall-probe.patch`
on `19bab45a`, output `prototypes/stall-probe-30k.txt`). A stall is a dwarf that holds a job, is in
`Walk`, and stays on one tile for more than 12 ticks (a step is 11). Counted in ticks, so the same on
any machine. Per 1,000 ticks (100 s):

| ticks | crew | stalls | 13 to 23 ticks | 24 to 60 | over 60 | longest | dwarves stalled |
|---|---|---|---|---|---|---|---|
| 3,000 to 4,000 | 40 to 53 | 21 | 18 | 3 | 0 | 3.2 s | 17 |
| 7,000 to 8,000 | 96 to 112 | 105 | 104 | 1 | 0 | 3.1 s | 28 |
| 11,000 to 12,000 | 140 to 148 | 266 | 265 | 1 | 0 | 3.2 s | 48 |
| 12,000 to 13,000 | 148 to 164 | 267 | 256 | 9 | 2 | 15.2 s | 53 |
| 16,000 to 17,000 | 200 | 21 | 19 | 2 | 0 | 4.3 s | 4 |
| 18,000 to 19,000 | 200 | 1,456 | 1,149 | 229 | 78 | 23 s | 146 |
| 20,000 to 21,000 | 200 | 1,529 | 1,187 | 211 | 131 | 138 s | 151 |
| 29,000 to 30,000 | 200 | 1,184 | 964 | 170 | 50 | 41.5 s | 121 |

After the dig a walking dwarf stands for 10 to 25% of his walking time. Most stalls are as long as one
`SQUEEZE_WAIT_TICKS` wait (13.4: a blocked dwarf looks again one step later, then swaps). NOT yet
shown: which rule each stall comes from, and whether the blocker is idle (#227's camp crowd) or
working. That attribution is the next probe.

## Run 3: after the head-on swap and the ring step (AC 17)

Build `a28a5523` or later (pull the branch on both sides first). What changed since Run 2:

- Two working dwarves who meet head-on trade tiles at once; they no longer both stand for a second first.
- A ring of working dwarves each waiting for the next one's tile all step at once.
- A group of failed route searches in one tick floods the ground once, not once per dwarf.

Headless on the devpod (`stall-probe-a28a5523.txt`): stops per 100 s at crew 150 fell from 266 to 26, after
the big dig from 1,100-1,600 to 100-560; the worst tick from 369-446 ms to 188 ms.

```bash
# WSL, from the repo, at the branch HEAD
target/release/simd 7451 --scenario _bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-8b-seat-like-run3.csv')
# a second PowerShell window, once the gui shows the world
python scripts\bench\wire_tap.py D:\Workspace\frostvein\.bin\13-8b-wire-tap-seat-run3.csv
```

Run at Normal past tick 20,000 (about 35 minutes). Press the mark key when you see a stop. Ctrl+C the tap
at the end. Both files, please.

**Questions**

1. Around minute 21 (crew about 150, before the big dig): do dwarves still stop briefly?
2. After the big dig: does it still stop and start? Gone, less, or the same as Run 2?
3. Dwarves now pass through each other more often (#219). Does that bother you?

### Sitting, Run 3 (2026-11-15)

Wolf: "still the same issue after the big designation target is assigned.."

- Question 2: the same after the big dig. Questions 1 and 3: not answered.
- Later the same day, Wolf: "between 16-17 it started to struggle but then it recovered later on in the end when I came back to watch.. csv is in signoff folder". He did not watch the whole run.
- File: `13-8b-wire-tap-seat-run3.csv` (the tap; 3,158 s to tick 30,000). No gui log arrived.

Tap, seconds of wall clock per 1,000 ticks (100 s is real time) and ticks arriving more than 150 ms after the one before:

| ticks | Run 2 wall | Run 2 late ticks | Run 3 wall | Run 3 late ticks |
|---|---|---|---|---|
| to 17,000 | 100.1-100.5 | 0-43 | 100.1-101.1 | 1-31 |
| 18,000 | 100.1 | 7 | 100.9 | 71 |
| 19,000 | 103.7 | 99 | 114.7 | 294 |
| 20,000-21,000 | 100.8-102.2 | 80-95 | 101.6-103.9 | 97-139 |
| 22,000-23,000 | 119.8-120.8 | 247-268 | 121.2-125.4 | 285-327 |
| 24,000-27,000 | 101.9-105.4 | 86-150 | 117.3-128.1 | 250-301 |
| 28,000-29,000 | 113.6-113.7 | 246-247 | 106.7-108.0 | 152-153 |
| 30,000 | 117.3 | 265 | 100.8 | 63 |

Longest single gap: Run 2 967 ms, Run 3 581 ms. So at the seat the world still falls behind real time after the big dig
(up to 28% slow in Run 3), and recovers by tick 30,000, which matches what Wolf saw. The devpod bench does not show
this: there the tick averages 20-30 ms in the same windows.

## Run 4: a cheaper tick after the big dig (AC 21)

Build `b351ece5` or later (pull the branch on both sides first). What changed since Run 3:

- The daemon writes one line to its own error output every 1,000 ticks: how long the ticks took in the sim
  (`step_ms_mean`, `step_ms_max`) and in sending (`send_ms_mean`).
  **Reading the line (from build `3c727628` on):** a pause, a Load or a speed change ends the window early. That
  part is written as a shorter line ending in `cut=pause`, `cut=load` or `cut=speed`; the next window starts
  afresh. The first line of a seat run is usually `cut=speed`: the scenario starts at Normal and the speed is
  raised after. One `over` with small maxes in the window where the tap joined is the tap's connect snapshot,
  not the machine.
- The sim no longer searches the whole map for a dig that is shut in a small pocket nobody can reach. That
  search was most of the tick after the big dig.
- The pile's free cells are found once per change instead of once per hauler.

Nothing the dwarves do is different: the same seed gives the same world at tick 30,000. Only the time changes.
On the devpod the tick after the big dig is now 25-42% of what it was (mean 4.5-11.2 ms, worst 83 ms, was
11-30 ms and 196 ms).

```bash
# WSL, from the repo, at the branch HEAD. 2> keeps the timer's lines.
target/release/simd 7451 --scenario _bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json 2> /mnt/d/Workspace/frostvein/.bin/13-8b-simd-stderr-seat-run4.txt
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-8b-seat-like-run4.csv')
# a second PowerShell window, once the gui shows the world
python scripts\bench\wire_tap.py D:\Workspace\frostvein\.bin\13-8b-wire-tap-seat-run4.csv
```

Run at Normal to tick 30,000 (about 50 minutes). Press the mark key when you see a stop. Ctrl+C the tap and the
daemon at the end. All three files, please.

**If you have time, a second run with no gui:** the same daemon line (stderr to `...-seat-run4-nogui.txt`) and
the tap (to `...-wire-tap-seat-run4-nogui.csv`), no launcher. Comparing the two tells how much of the seat's
slowness is the gui sharing the machine.

**Questions**

1. After the big dig: does it still stop and start? Gone, less, or the same as Run 3?
2. Around minute 16-21: does it still struggle?

### Sitting, Run 4 (2026-11-17)

Wolf: "files are in folder but was busy with other things so forgot to watch". So questions 1 and 2 are not
answered by eye. No second run without the gui.

- Files: `13-8b-simd-stderr-seat-run4.txt` (the daemon's timer), `13-8b-seat-like-run4.csv` (the gui log),
  `13-8b-wire-tap-seat-run4.csv` (the tap; tick 30,000 at 2,989 s, Run 3 took 3,158 s).
- Build: the daemon log has no build stamp; its timer lines prove it is `cde6e93f` or later. The gui log names
  `fa2259b8`; no client or protocol code changed between that and `e6eb06d5`, so it is the same gui.

Tap, the same measure as Run 3:

| ticks | Run 3 wall | Run 3 late ticks | Run 4 wall | Run 4 late ticks |
|---|---|---|---|---|
| to 17,000 | 100.1-101.1 | 1-31 | 100.1-100.3 | 0-24 |
| 18,000 | 100.9 | 71 | 100.1 | 0 |
| 19,000 | 114.7 | 294 | 100.1 | 0 |
| 20,000-21,000 | 101.6-103.9 | 97-139 | 100.1 | 0 |
| 22,000-23,000 | 121.2-125.4 | 285-327 | 100.1-100.2 | 1-23 |
| 24,000-27,000 | 117.3-128.1 | 250-301 | 100.1-100.2 | 2-32 |
| 28,000-29,000 | 106.7-108.0 | 152-153 | 100.2 | 14-27 |
| 30,000 | 100.8 | 63 | 100.1 | 13 |

Longest single gap: Run 3 581 ms, Run 4 292 ms (at tick 7,000, before the dig; 226 ms after it).

The daemon's own line, after the big dig (t17,000-30,000): step mean 3.2-16.6 ms a tick, worst 31-144 ms,
sending 0.4-1.3 ms; 0-4 ticks per 1,000 ran over the 100 ms tick. Before the dig: mean 0.3-3.8 ms, worst
41-193 ms, 0-3 over.

The gui log (frame times) matches Run 2's: steady p50 22.2 ms (Run 2 23.3), p99 69.0 ms (65.2), worst 369 ms
(335). This story did not touch the gui. One mark, at the end of the log, after tick 30,000.

So at the seat the daemon now keeps real time through the whole run, after the big dig too: Run 3 fell up to
28% behind there. Whether the dwarves still look as if they stop and start is not answered: the files measure
the clock, not the blocking the stall probe counts, and Task 17 changed no behaviour.

### Sitting, Run 5 (2026-11-17): Run 4's build, watched

Wolf re-ran the Run 4 commands and watched from minute 25, just before the big dig, to the end: "1 gone I would
say .. i did not see hickups this time and watched it from 25 till the stop". No files were saved from this
sitting (corrected 2026-11-21: this card said "files named `run5`", which was an assumption). The sitting with
files is below.

- Question 1 (after the big dig): **gone**.
- Question 2 (minutes 16-21): not watched this time.

### Sitting, Run 5 with files (2026-11-21): the review patches' build

Wolf: "no hickups anymore (well fps drops between 20-30 some times .. probably stable 30 is ok for this game".

- Question 1 (after the big dig): **gone**, as on 11-17, now on the build with the review patches.
- The fps drops are the gui drawing, not the sim stopping: see the gui log below.
- Files: `13-8b-seat-like-run5.csv` (the gui log, build `1a50406b`), `13-8b-wire-tap-seat-run5.csv` (the tap;
  tick 30,000 at 3,009 s), `13-8b-simd-stderr-seat-run5.txt` (the daemon's timer). The daemon log has no build
  stamp and no `cut=` line (the run stayed at Normal), so its build is not proven by the log itself.

Tap, the same measure as Runs 3 and 4 (late = more than 150 ms after the tick before):

| ticks | Run 3 wall | Run 3 late ticks | Run 4 wall | Run 4 late ticks | Run 5 wall | Run 5 late ticks |
|---|---|---|---|---|---|---|
| to 17,000 | 100.1-101.1 | 1-31 | 100.1-100.3 | 0-24 | 100.1-100.3 | 1-4 |
| 18,000 | 100.9 | 71 | 100.1 | 0 | 100.1 | 0 |
| 19,000 | 114.7 | 294 | 100.1 | 0 | 100.2 | 47 |
| 20,000-21,000 | 101.6-103.9 | 97-139 | 100.1 | 0 | 100.1 | 4-10 |
| 22,000-23,000 | 121.2-125.4 | 285-327 | 100.1-100.2 | 1-23 | 100.3 | 69-71 |
| 24,000-27,000 | 117.3-128.1 | 250-301 | 100.1-100.2 | 2-32 | 100.6-103.7 | 49-113 |
| 28,000-29,000 | 106.7-108.0 | 152-153 | 100.2 | 14-27 | 100.8-102.0 | 45-64 |
| 30,000 | 100.8 | 63 | 100.1 | 13 | 100.5 | 16 |

Longest single gap: Run 5 299 ms (tick 26,477), Run 4 292 ms, Run 3 581 ms.

After the big dig, Run 5 sits between Runs 3 and 4: at most 3.7% behind real time (Run 3 28%, Run 4 0.2%) and
45-113 late ticks per 1,000 (Run 3 250-327, Run 4 0-32). No gap reached 300 ms, and Wolf saw no stops.

The daemon's own line says where the late ticks come from: the step. After the big dig (t17,000-30,000) the
step mean is 6.0-32.5 ms a tick (Run 4 3.2-16.6), worst 64-264 ms (31-144), 0-105 ticks per 1,000 over the
100 ms tick (0-4), worst at t26,000-28,000. Sending 0.6-2.2 ms (0.4-1.3). Before the dig the two runs match:
mean 0.5-2.3 ms (Run 4 0.3-3.8), 0-3 over.

So the same work took about twice as long after the dig. The sim did not change: `sim-core` is identical
between the two builds (the review patches touched only `simd`'s timer, not what it times), the seed is the
same, and before the dig the step is the same. That points at the seat machine having less to give the
daemon during this run's last 15 minutes. What else was running is not known. It did not show as stops.

The gui log, the same build as the tap: steady p50 19.8 ms (Run 4 22.2), p99 45.4 ms (69.0), worst 307 ms
(369); frames of 50 ms or more 598 (6,311). By 3-minute bucket the p50 is 47-59 fps except two stretches:
minutes 12-15 (27 fps, crew 120, half the frames under 30 fps) and minutes 30-36 (35-39 fps, a third of the
frames under 30 fps, the re-meshing after the big dig). Those are the drops Wolf saw. They are the gui's
crowd and re-mesh cost (#224), which this story does not touch. One mark, at the end of the log.
