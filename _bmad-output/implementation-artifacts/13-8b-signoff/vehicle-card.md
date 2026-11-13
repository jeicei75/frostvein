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

(not yet run)
