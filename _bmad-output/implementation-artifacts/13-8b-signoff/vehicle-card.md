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

(not yet run)
