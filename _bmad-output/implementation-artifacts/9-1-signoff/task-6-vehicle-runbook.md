# Story 9.1 — Task 6 vehicle session card

**Vehicle:** gingerspice only (native Windows / NVIDIA Vulkan). This is a pre-session recipe, not
a record of a run. Fill every blank during Epic 9's shared vehicle sitting; do not infer values.

**Corrected at code review 2026-08-28.** The first draft could not be executed: its build-identity
stop-rule halted the shadows-off run, it never said how to disable shadows, and it carried no
`--capture` command or exit codes. All three are fixed below. Read §0 before anything else.

## 0. Two things that will otherwise look like failures

- **The shadows-OFF binary WILL report `-dirty`, and that is correct.** Disabling shadows needs an
  uncommitted source edit (§2), so its build line reads `gui build <sha>-dirty`. The `-dirty` stop
  rule applies to the shadows-ON run only.
- **The shadows-OFF capture is EXPECTED to exit 101.** Today's frame measures 0.9883 % against the
  0.6651 % ceiling, so the new assertion fires by design. **That is the measurement, not a broken
  build.** The PNG is written before validation, so the evidence always survives the abort. Record
  the number and the exit code and carry on.

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
exactly. **For the shadows-ON run it must carry no `-dirty` suffix — stop if it does.** For the
shadows-OFF run of §2, `-dirty` is expected (see §0).

## 2. Controlled shadows-off / shadows-on pair

Keep the same daemon, world, boot-vista framing, and `--at-tick 20` for both captures. Do not use
`--z`: a cut below the top skips the calibrated checks, and a run that skips them has judged
nothing. Take the **shadows-off** capture first, preserve its PNG and line, then revert and rebuild
for the **shadows-on** capture — the revert is what makes the second run the branch binary.

**Disabling shadows — the exact edit.** In `crates/gui/src/project.rs:422`, change

```rust
        shadow_maps_enabled: matches!(kind, protocol::LightKind::Campfire),
```

to `shadow_maps_enabled: false,`. Rebuild and re-copy. **Revert with
`git checkout -- crates/gui/src/project.rs` and rebuild again before the shadows-on run**, and
confirm the `-dirty` suffix is gone. There is no CLI flag or env var for this; the value is
hardcoded by design (YAGNI), so a rebuild is the only lever.

```bash
# Windows, once per half of the pair. Rename the PNG between halves so nothing is overwritten.
gui.exe 7451 --capture 9-1-vista-shadows-off.png --at-tick 20
echo "exit=$?"     # EXPECTED 101 — the ceiling fires; the PNG is still written
gui.exe 7451 --capture 9-1-vista-shadows-on.png --at-tick 20
echo "exit=$?"     # 0 if shadows brought the pool under 0.6651 %, 101 if they did not
```

Match the `capture range check:` output by **prefix**, never by whole line — 7.1 changed an
oracle's line shape and older recipes quoting whole lines stopped matching.

```text
shadows off — capture range check: _______________________________
  blown-pool: ______ %    p99: ______    exit: ______
shadows on  — capture range check: _______________________________
  blown-pool: ______ %    p99: ______    exit: ______
```

For each run, `gui build <sha>`: __________. The on result is judged against the 0.6651 % ceiling;
**record a failed ceiling as the result, not a reason to tune any withheld lever.** The ceiling has
about one ulp of headroom over boot7's own measurement, so a one-pixel overshoot is the intended
bar, not noise.

**The counter-test, which proves the other exit channel still works:**

```bash
gui.exe 7451 --capture 9-1-should-not-exist.png --at-tick 100000 --frames 30
echo "exit=$?"     # must be 1, with the exhaustion line on stderr and no PNG written
```

Recording `exit=101` above and `exit=1` here is the **only live evidence AC8 gets** that a failed
ceiling changes the process exit rather than merely computing a number. No headless test spawns the
binary, so if these two codes are not written down, AC8 stays partially met.

## 3. Performance, then Wolf's eye

With shadows on, use F3 and allow the overlay to settle. Read sustained fps at both locations;
write values before evaluating the visual judgement. **Do not resize or maximise the window before
any capture** — the ceiling is a whole-frame fraction calibrated at 1280×720, and a different
aspect ratio changes what is in frame and makes the number incomparable.

| view | NFR6 floor | sustained fps |
| --- | ---: | ---: |
| working zoom | 60 | |
| full boot vista | 30 | |

Only after all readings and PNG evidence are preserved, have Wolf compare the live camp against
`5-4-signoff/candidate-artifact-2026-08-15.png`:

1. Does it read as light on snow rather than glare, and are dwarves, marks, and the hover slab
   discernible? Record Wolf's exact judgement:

   `____________________________________________________________________________`

2. **Do the shadows themselves look right?** Point-light shadows are new to this build. Does the
   snow around the fire show stripes, banding or acne that were not there before, or a hard shadow
   edge that reads as wrong? The campfire light sits about half a world unit above the snow with
   Bevy's default depth and normal biases, which is the classic grazing-angle acne case — and
   AC14's other questions could both answer "yes" while this one makes the camp look worse.

   `____________________________________________________________________________`

Then re-check the earlier hover-slab observation independently and record whether the campfire was
its cause (the rendered fix remains Story 9.2):

`____________________________________________________________________________`
