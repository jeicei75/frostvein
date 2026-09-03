# Story 10.4 — Task 6 vehicle session card

**This sitting owes AC12's closing half and nothing else: your eye on the pines, and two frame
rates.** Everything else is closed and measured headlessly.

**Vehicle:** gingerspice only. Branch `10-4-the-trees-look-right-the-pilot`, tip `12da79d`, PR #63.
Fill every blank; infer nothing. Why any figure below is what it is: `10-4-signoff/README.md`.

## Expect these — they are not faults

- `exit=101` on `near-white-area` above `1.5630%`. Pre-existing; the PNG is written first.
- **`touch crates/gui/build.rs` before every build.** Without it the stamp can lag a commit.
- **Never `--at-tick`.** Use `--frames`. **2000 is plenty** — measured on the vehicle: the run
  is tick-driven and takes ~14 s, ending on the capture health floor, not on the cap. (An
  earlier version of this card said "set it absurdly high"; on a vsync'd window that is just
  a long wait for nothing.)

## 1. Build

```bash
export PATH="$HOME/.cargo/bin:$PATH"
git checkout 10-4-the-trees-look-right-the-pilot && git pull
touch crates/gui/build.rs
cargo build -p simd -p tui
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
git rev-parse --short HEAD
./target/debug/simd 7451
```

Copy `gui.exe` alone into an **empty directory** — no `assets/`. That is the delivery.

`gui build`: ____________  `HEAD`: ____________  — must match, must not say `-dirty`

## 2. Startup lines

```
gui.exe 7451
```

| line | must read | got |
| --- | --- | --- |
| `gui tree assets:` | `4 of 4 embedded in this binary, 1277340 bytes` | |
| `projected N terrain cubes at z 31` | `39936` | |
| `gui trees:` | `meshes=265 scenes_loaded=true source=embedded` | |

**`meshes=0` or `scenes_loaded=false` → STOP.** Nothing below means anything.

CONFIRMED ON THE VEHICLE 2026-09-03, build `3b0c43f`: all four read as above, from a lone
`gui.exe` in `$env:TEMP` with no `assets/` beside it. The embedding delivery is proven on hardware.

## 3. Windowed capture

```
gui.exe 7451 --capture 10-4-vista.png --frames 2000
echo "exit=%ERRORLEVEL%"
```

No `--headless` — deliberate. **A panic on `0 == 265` or `capture drew a hollow cut` is a
regression and is the finding of the sitting.** Otherwise expect `exit=101`.

Two lines print here and ONLY here — the capture report writes them, so a plain
`gui.exe 7451` does not and never did:

| line | must read | got |
| --- | --- | --- |
| `slice: z 31 ...` | `(265 of 265 cut-face tiles at z 31)` | |
| `trees:` | `meshes=265 of 265 scenes_loaded=true source=embedded` | |

`capture range check:` ______________________________________  exit: ______

CONFIRMED ON THE VEHICLE 2026-09-03, build `3b0c43f`: both lines printed from a WINDOWED run and
the PNG was written (`10-4-vista.png`, 1280x720). **The regression this step exists to catch is
dead on hardware.** `exit=101` as predicted — but see the near-white note in `README.md`: the GPU
reads WORSE than the devpod, not better, which is the opposite of what this card predicted.

## 4. Your eye — AC12's closing half

Open `10-4-signoff/candidate-D-authored-pines-blender-5.2.1.png` beside the live client.
**Judge silhouette, proportion, density, variety — not colour or light** (Cycles vs Bevy PBR).

| # | question | reading |
| --- | --- | --- |
| a | Trees you chose? Compare `client-baseline-2ef194d-subdiv2.png` for the cube trees. | |
| b | Does the per-tree yaw read as less repetitive, or is four meshes still obvious? | |
| c | Root / terrain junction — **confirm, overturn or sharpen only.** Already filed both halves, and the foliage-ring candidate with 9.4's three reasons, in `deferred-work.md`. Do not re-file. | |
| d | Anything new. | |

## 5. Two fps readings

`F3` overlay. `Q` zoom in, `E` zoom out. The pines are **67 % of the triangles** at the default
subdivision (~1.19 M of ~1.76 M) and no frame rate has ever been taken against them.

| Where | Floor (NFR6) | Reading |
| --- | --- | --- |
| Working zoom | 60 fps sustained | |
| Full vista (boot framing) | ≥30 fps sustained | |

Let it settle and pan while you watch. **A failed reading is the result, reported with its number.**

## 6. Paste back

Build stamp · the four startup figures · the capture line and exit · the four eye readings · the
two fps numbers.
