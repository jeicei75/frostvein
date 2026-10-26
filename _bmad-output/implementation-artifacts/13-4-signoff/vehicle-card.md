# 13.4 seat card: two dwarves trade tiles in the tunnel (Q4 a)

Push first (`scripts/push.sh --fast`) so the seat builds this branch's HEAD. One short sitting, about 3 minutes.
The squeeze is judged by scenario tests and `simd --bench` (M2-26). This card answers only what a test cannot: does
a pass read acceptably with no swap animation? (13.2: no animation, out of scope. If it reads badly, that becomes an
issue, not 13.4's work.)

The run is Wolf's 12.12 seat save (#206's freeze). On `main` it freezes with 3 marks left. On this branch the
dwarves trade tiles and every mark is done about 1,000 ticks in. It replays
`13-4-signoff/stuck-tunnel-seat.json`, a copy of `scenarios/stuck-tunnel.json` at **Normal** speed (10 ticks a
second). The committed scenario runs at fast4x, 200 ticks a second, and there its first swap comes 0.3 s after
connect. The copy is the same save with the same (no) commands; its `--bench` state hash equals the scenario's
(`a5d257cac0182a1f`).

## Run it

```bash
# WSL, from the repo, on this branch's HEAD. A FRESH release daemon: it holds tick 18,082 until the gui connects.
cargo build --release -p simd
target/release/simd 7451 --scenario _bmad-output/implementation-artifacts/13-4-signoff/stuck-tunnel-seat.json
```

```powershell
# PowerShell, from the Windows checkout
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--z','10','--select','4')
```

`--select 4` follows dwarf 4. He is in 5 of the 7 swaps, including the first two in the tunnel (x 77, y 68-71, z 10).
`--z 10` slices at the tunnel's level. Both were checked headless on the devpod (lavapipe, a 160-frame capture,
`13-4-signoff/seat-framing-select4-z10.png`): he is lit in the tunnel at the frame centre. Zoom with `E`. `space` pauses, and you can pause any time: the speed does not change the outcome.

When to look, counted from connect (10 ticks a second at Normal):

| after connect | swap | where |
|---|---|---|
| ~6 s (t18,143) | dwarves 4 and 2, head-on job-holders | tunnel, (77,70,10) to (77,71,10) |
| ~20 s (t18,281) | 2 and 4 | tunnel, (77,68,10) to (77,69,10) |
| ~30 s, ~42 s | 1 and 4, then 4 and 1 | (73,63,10) to (74,63,10) |
| ~46 s | 3 and 1 | (70,63,10) to (71,63,10) |
| ~64 s (t18,717) | 4 and an IDLE dwarf 2 (his home moves) | (79,71,10) to (80,71,10) |
| ~112 s | 3 and 1 | (68,60,10) to (68,59,10) |

Every mark is done by about 100 s (1,000 ticks). The WSL terminal prints
`scenario stuck-tunnel-seat: done at tick 28082 state=…` only after the full 10,000 ticks (about 17 minutes);
you do not need to wait for it. If the gui exits early, **restart both**: Ctrl-C simd and start it fresh, then
relaunch the gui. The hold happens only for the first client.

## Questions

1. Did the dwarves get past each other in the tunnel, and were the marks finished (no dwarf left swinging at a
   mark, nobody stuck)?
2. Does a pass read acceptably with no animation: two dwarves simply trade places in one step? If it reads badly,
   say how, and it becomes an issue.

## Sitting

(not yet sat)
