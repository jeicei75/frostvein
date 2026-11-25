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
  mark is used up when its bush is picked. A bush regrows after 3,000 ticks (5 minutes at Normal, 15 s at
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
5. Switch to fast4x and wait about 15 s: a picked bush regrows its berries but is not marked. Mark it again
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

**Run 3, 2026-11-22, build `bb687047` (Wolf).** His words: "well it still refuses to mark bushes in other than initial
surface but this time it marks them as red". The preview lit the bushes on every level, but the one forage rect spanning
several levels broke the wire's single-level rule: simd drops such a rect with only a log line ("invalid client rect"),
so nothing reached the sim. My run-2 fix was tested at the command builder only, never against the daemon. Fixed: one
single-level command per level holding a ripe bush. Live-probed on a real daemon (`seat3-forage-levels-probe.txt`): the
old command is dropped, and the new commands mark both bushes.

**Run 4, 2026-11-22, build `9d252bee` (Wolf).** His words: "ok works now .. performance issue is back when a lot of
dwarves and digging happening .. but let's continue tomorrow". Forage marking across levels works. The performance
regression under many dwarves and digging is open for the next session: measure first (13.8b's seat-like scenario and
tap, `main` vs this branch), and suspect 13.9's per-tick work first (the forage claim turn, `next_trip_job` searches,
`tend_plants`) before the known debts (#224 gui fps, #235 haul).

**Load measurement, 2026-11-23 (Opus, headless, release).** The sim is not the regression. `RUNS=3
scripts/tick-bench.sh 3c6cf3b7 4251c099` on seat-like: mean 3.328 ms on `main`, 3.328 on this branch, -0.0%
(control -1.4%, floors 1.7% / 2.3%); `state` differs because the branch's world has bushes
(`tick-bench-seatlike-main-vs-4251c099.txt`). For forage under load, seat-like gains forage marks on every ripe bush in
(60..200)², z 5..27 (about 1,100 marks; `prototypes/seat-like-forage-*.json`), one run each on this branch
(`seat-like-forage-bench-4251c099.txt`):

| seat-like variant | mean ms | p99 | max | over10 | foraged |
|---|---|---|---|---|---|
| no forage | 3.32 | 35.2 | 84 | 2,209 | 0 |
| marks at t7 | 3.49 | 38.3 | 106 | 2,225 | 567 |
| marks at the big dig (t17000) | 2.91 | 31.3 | 76 | 1,948 | 547 |

Still open: the gui's frame cost with about 1,900 bush entities (one per plant, reconciled with the designations each
pass), the wire, and whatever run 4's world was. That run saved no perf log or save that this devpod can read.

**Run 5 setup (2026-11-23): the measured run.** Wolf on run 4: "not sure .. well fps dropped that's for sure .. but
not even haze off helped". The sim measures the same as `main`, so this run looks at the gui. It repeats 13.8b's
run 5 with files (`13-8b-signoff/13-8b-seat-like-run5.csv`, build `1a50406b`, same scenario, same seat), so the two
gui logs compare bucket by bucket at the same crew size. The only world difference is this branch's ~1,900 bushes.

```bash
# WSL, from the repo, at the branch HEAD. 2> keeps the timer's lines.
cargo build --release -p simd
target/release/simd 7451 --scenario _bmad-output/implementation-artifacts/13-8-signoff/prototypes/seat-like.json 2> /mnt/d/Workspace/frostvein/.bin/13-9-simd-stderr-seat-run5.txt
```

```powershell
# PowerShell, from the Windows checkout, at the branch HEAD
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-9-seat-like-run5.csv')
# a second PowerShell window, once the gui shows the world
python scripts\bench\wire_tap.py D:\Workspace\frostvein\.bin\13-9-wire-tap-seat-run5.csv
```

Run at Normal to tick 30,000 (about 50 minutes), with the camera framed the way 13.8b's run 5 was. No forage
needed. Press the mark key when the fps drops. Ctrl+C the tap and the daemon at the end. All three files, please.

After tick 30,000 (so the perf log above stays comparable), press 6 and drag over a few ripe bushes. A marked bush
glows amber in the gui and shows as an amber `&` in the tui. No approved draft covered this look; the implementer
chose it (review round 1), so it is yours to judge here.

**Question 1:** compared with 13.8b's run 5 ("fps drops between 20-30 some times"): the same, or worse?

**Question 2:** does a marked bush read as marked, the amber glow in the gui and the amber `&` in the tui?


**Run 5b setup (2026-11-25): the short dig run.** Wolf, on a run 5 attempt: he came back about 18 minutes in, with
the crew arrived and idle, and the hiccups started when he designated digging. The sim is not the cause: on
seat-like, this branch and `main` do the same work (1,486 dug, 538 stored) with the same slow ticks (p99 34.6 vs
34.2 ms, max 83 vs 80 ms). At Normal a tick has 100 ms, so none of them overrun. That leaves the gui: the terrain
rebuild for each dug tile at `--subdiv 4` (on `main` too) or this branch's ~1,900 bushes.

`seat-dig-short.json` (in this folder) is seat-like in 10 minutes: seat-like's seed and dims, but all 200 dwarves at
the start, seat-like's stockpile at t6, idle until t1800 (3 minutes, the baseline), then all of seat-like's cut and
dig marks at once (30 cut, 1,600 dig). It ends at t6000. On the devpod it digs 571 tiles by then, with a slowest tick
of 60 ms.

```bash
# WSL, from the repo, at the branch HEAD. Copy the file first so the main run below reads the same one.
cp _bmad-output/implementation-artifacts/13-9-signoff/seat-dig-short.json /mnt/d/Workspace/frostvein/.bin/
cargo build --release -p simd
target/release/simd 7451 --scenario /mnt/d/Workspace/frostvein/.bin/seat-dig-short.json 2> /mnt/d/Workspace/frostvein/.bin/13-9-simd-stderr-short.txt
```

```powershell
# PowerShell, from the Windows checkout, at the branch HEAD
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--perf-log','D:\Workspace\frostvein\.bin\13-9-short.csv')
```

Leave the camera on the camp, at Normal, until t6000. Press the mark key at the first hiccup. No forage needed.

If it hiccups, run the same thing once more with both checkouts on `main`, the same file, and the names
`13-9-simd-stderr-short-main.txt` and `13-9-short-main.csv`. Only the bushes differ between the two runs.

**Question 3:** after t1800, does this branch hiccup, and does `main` hiccup the same?

**Run 5b result (2026-11-25): Question 1 answered, the same.** Three seat runs of `seat-dig-short.json`, each 551
edit frames with the first dig at about 192 s. The idle phase is before the first dig; the dig phase is the 290 s after it.

| log | build (gui and simd) | idle p50 | dig p50 / p95 / p99 | edit frame p50 | dig frames >= 50 ms |
|---|---|---:|---:|---:|---:|
| `13-9-short.csv` | `acce6385` | 19.8 ms | 28.0 / 37.6 / 50.0 ms | 43.0 ms | 100 |
| `13-9-short-repeat.csv` | `acce6385` | 22.0 ms | 28.9 / 39.5 / 52.6 ms | 44.6 ms | 143 |
| `13-9-short-main.csv` | `3c6cf3b7` (`main`) | 19.6 ms | 27.5 / 36.8 / 48.2 ms | 43.1 ms | 85 |

- The hiccups after a dig order are `main`'s cost at `--subdiv 4`: a 20 -> 28 ms median and ~43 ms terrain-rebuild
  frames, with or without the bushes.
- `13-9-short.csv` from 480 s on is not a measurement: Wolf switched the monitor to another input (vsync on).
- The two same-build runs differ by up to 2.2 ms at p50 (idle), so one run per build cannot resolve the 2-3 ms the
  repeat reads above `main` late in the dig (480-600 s).

The dig cost is filed as #245 (on `main`, not 13.9's).

**Question 2 answered (Wolf, 2026-11-25): yes.** A marked bush reads as marked, in both the amber gui glow and the
amber `&` in the tui. AC 15's seat questions are all answered.
