# Story 9.1 — Task 6 vehicle session card

**Vehicle:** gingerspice only (native Windows / NVIDIA Vulkan). This is a pre-session recipe, not
a record of a run. Fill every blank during Epic 9's shared vehicle sitting; do not infer values.

## 1. Build identity before any observation

```bash
# WSL
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p simd -p tui
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
git rev-parse --short HEAD
./target/debug/simd 7451
```

Copy `target/x86_64-pc-windows-gnu/release/gui.exe` to Windows. Start it there with `gui.exe
7451`; its first line must be `gui build <sha>`, matching the WSL `git rev-parse --short HEAD`
exactly and without `-dirty`. Stop if it does not.

## 2. Controlled shadows-off / shadows-on pair

Keep the same daemon, world, boot-vista framing, and `--at-tick 20` for both captures. Do not use
`--z`: a cut below the top skips the calibrated checks. First rebuild/run a deliberately
shadows-disabled binary, then rebuild/run the branch binary with campfire shadows enabled. Preserve
both PNGs and the complete `capture range check:` lines before changing framing, zoom, or world.

```text
shadows off — capture range check: _______________________________
  blown-pool: ______ %    p99: ______
shadows on  — capture range check: _______________________________
  blown-pool: ______ %    p99: ______
```

For each run, `gui build <sha>`: __________. The on result is judged against the 0.6651% ceiling;
record a failed ceiling as the result, not a reason to tune any withheld lever.

## 3. Performance, then Wolf's eye

With shadows on, use F3 and allow the overlay to settle. Read sustained fps at both locations;
write values before evaluating the visual judgement.

| view | NFR6 floor | sustained fps |
| --- | ---: | ---: |
| working zoom | 60 | |
| full boot vista | 30 | |

Only after all readings and PNG evidence are preserved, have Wolf compare the live camp against
`5-4-signoff/candidate-artifact-2026-08-15.png`: does it read as light on snow rather than glare,
and are dwarves, marks, and the hover slab discernible? Record Wolf's exact judgement:

`____________________________________________________________________________`

Then re-check the earlier hover-slab observation independently and record whether the campfire was
its cause (the rendered fix remains Story 9.2):

`____________________________________________________________________________`
