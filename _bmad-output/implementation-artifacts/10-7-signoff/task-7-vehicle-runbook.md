# Story 10.7 — Task 7 vehicle session card

**What this session is for:** UX-DR22's **closing half**. Wolf approved a bench artifact before the
client change was written (`candidate-plus17.66.png`, the opening half, ruled 2026-09-03). This
session is the other half: **the built client, on the vehicle, compared against that artifact.**

**What changed in the client:** one number. The sun's elevation went from `-6.4181°` (below the
horizon, travelling upward, lighting nothing) to the approved `+17.66°`. Nothing else in the look
moved — not the ambient balance, not `directional_illuminance`, not the tree colours, not the
near-white ceiling.

## Expect these — they are not faults

- **`--capture` exits 101** with `near-white area is 2.25%, above the 1.5630% ceiling`. **The PNG is
  still written** (`capture.rs` saves before it validates). This breach **predates this story** —
  the shipped build breaches too, at 1.87 % — and it is filed as its own defect in
  `deferred-work.md` § "the near-white ceiling's calibration frame is gone". **Do not raise the
  constant to clear it.**
- **`touch crates/gui/build.rs` before every build.** Without it the stamp can lag a commit.

## 1. Build

```bash
export PATH="$HOME/.cargo/bin:$PATH"
git checkout 10-7-the-sun-lights-the-valley && git pull
touch crates/gui/build.rs
cargo build -p simd -p tui
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
git rev-parse --short HEAD
./target/debug/simd 7471
```

`gui build`: ____________  `HEAD`: ____________  — must match, must not say `-dirty`

## 2. Your eye — the closing half

```
gui.exe 7471
```

Open `_bmad-output/implementation-artifacts/10-7-signoff/candidate-plus17.66.png` beside the window.
**That frame is a Cycles render pinned to the client's constants, not something the client drew** —
so expect the materials and antialiasing to differ. What must agree is the **lighting**:

| what to check | approved artifact shows | vehicle | agrees? |
| --- | --- | --- | --- |
| trees cast shadows onto the snow | yes, angled down-left | | |
| the dig terraces read as depth, not as flat rings | yes | | |
| the campfire still owns its own pool of warm light | yes, not swamped by the ground | | |
| the valley reads as lit, not as ambient fill | yes | | |

**If the campfire has stopped reading as the valley's own light source, say so** — that was the
explicit trade-off in choosing `+17.66°` over `+25.87°`, and the vehicle is the venue that settles
it. A disagreement here is a real finding, not a tolerance.

## 3. One fps reading

Shadows now actually land on geometry. Before this story the cascades rendered against a sun below
the horizon, so this is **the first build where directional shadow work is real**. NFR6 was met at
10.4 (>100 fps typical, brief ~60, against 60/30 floors) and this story does not own NFR6 — but the
reading is worth having, because the cost changed.

working zoom fps: ________   full vista fps: ________   (floors: 60 / 30)

## 4. A windowed capture, for the record

```
gui.exe 7471 --capture 10-7-vista.png --frames 20000
```

Exits 101, writes the PNG. Paste the `capture range check:` line — the near-white figure on the
vehicle is the one number `deferred-work.md` is still missing for the calibration question.

`capture range check:` ______________________________________________

## 5. Paste back

- the two build stamps from §1
- the four agree/disagree rows from §2, and anything your eye caught that the table did not ask about
- the two fps figures from §3
- the `capture range check:` line from §4

Headless figures this session is checked against, both from `--subdiv 1 --frames 160` on the branch:
mean luminance **87.87 → 101.19** (change 13.31 against a worst same-build noise floor of 0.101,
**131.8x**), near-white **1.82 % → 2.25 %**.
