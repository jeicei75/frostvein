# Task 6 — the live vehicle session (story 6.1, wow beat 2)

Everything in story 6.1 except this session is done, reviewed, patched and green. This is the
half no devpod can run: **no devpod has graphics userspace** (measured at 5.3, both fallbacks
walked to the end), so the window opens only on **gingerspice — native Windows client,
cross-compiled `gui.exe`, `simd` in WSL, localhost, native NVIDIA Vulkan**.

It closes **ACs 7, 12, 13, 15 and 16**. AC17 is Wolf's separate sign-off (Task 9) and no command
here can close it.

**Never fake any of this.** If a step cannot run, report the step and stop — a missing measurement
is a finding, an invented one is a lie the whole gate rests on.

---

## 0. Build (in the devpod / WSL)

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# the daemon and the terminal client stay on the Linux side
cargo build -p simd -p tui

# the Bevy client cross-compiles to Windows
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
```

`gui.exe` lands at `target/x86_64-pc-windows-gnu/release/gui.exe`. Copy it to the Windows side.

## 1. Start the daemon (WSL, leave it running)

```bash
./target/debug/simd 7451
```

Port is positional. The seed is fixed in the binary, so the world is the one every measurement in
this story was taken against.

## 2. The "before" capture (Windows side)

```cmd
gui.exe 7451 --capture 6-1-motion-before.png --frames 1500
```

**`--frames 1500`, not 600.** `simd` ticks at 10 Hz and this machine runs >135 fps, so 600 frames
is ~4.4 s ~= 44 delivered ticks — below the instrument's ≥100-tick floor, which would panic
*before* writing any PNG. 1500 frames is ~11 s ~= 110 ticks.

**Do not use the filenames `6-1-before.png` / `6-1-after.png`** — those are the pair Wolf approved
and are the baseline this run is compared against.

📋 **Paste back:** the `motion:` line and the range-check line, verbatim.

## 3. Designate the dig site (WSL, while the client runs)

```bash
./target/debug/tui 7451 --z 9 --frames 3 \
  --key d,h,h,h,h,h,h,h,h,h,k,k,enter,l,j,j,j,enter
```

`d` enters dig mode and resets the cursor to (64,64); 9×`h` + 2×`k` reach `[55,62]`; `enter`
anchors; 1×`l` + 3×`j` reach `[56,65]`; `enter` commits the rect `[55,62,9]`–`[56,65,9]` = 8
mineral tiles, **all eight of them diggable**. Measured live on 2026-08-18 against this exact sequence: designation reaches the wire ~2 ticks later, the first
dwarf enters `Work` ~24 ticks after that, **all 8 tiles are dug within 52 ticks (~5 s)**, and 8
stone items then stand at the site permanently.

## 4. The "after" capture, across the dig (Windows side)

Start this **before or as** you issue the designation, so the run spans the actual digging.

```cmd
gui.exe 7451 --capture 6-1-motion-after.png --frames 2000 --expect-work
```

📋 **Paste back:** the `motion:` line and the range-check line.

**Exit 0 is not a result.** The motion line now prints *before* the assertions, so even a failing
run gives you all five numbers — paste them either way. Expected shape:

```
motion: ticks observed=>=100  dwarf position changes=>0  mid-blend frames=>0
        max working dwarves=>=1  item count=>=1
```

Also expected at startup: `projected 53365 terrain cubes`.

## 5. AC16 — the TUI cross-check (rung 1)

Keep a `tui` client open on the same daemon **beside** the Bevy window for the whole session:

```bash
./target/debug/tui 7451 --z 9
```

📋 **Confirm in words:** the dwarves the Bevy client shows moving are in the same places the TUI
shows them, and the dug tiles/rubble agree. This is what separates real sim state from client
invention.

## 6. AC15 — the capture self-test on the vehicle

```bash
cargo test -p gui --test capture --no-run --target x86_64-pc-windows-gnu
```

Copy the built test exe to the Windows side, then:

```cmd
set FROSTVEIN_CAPTURE_FIRST=6-1-motion-before.png
set FROSTVEIN_CAPTURE_SECOND=6-1-motion-after.png
<test-exe> --ignored
```

It compares the **projected dig-site window**, not whole-file bytes (snowfall alone makes any two
frames differ), and now requires **≥200 changed pixels** inside that window. Reference from the
approved pair: 1,651 changed inside, ~5 expected from atmosphere alone.

📋 **Paste back:** pass/fail and the pixel count in the message.

## 7. AC13 — the NFR6 reading

Press **F3** for the frame-time overlay. Read sustained fps **with the dig in progress and all
lights flickering**, at:

- **working zoom** — bar is **60 fps**
- **full vista** — bar is **≥30 fps**

📋 **Paste back both, labelled `gingerspice / native Windows / NVIDIA`.** 5.4 measured >135 fps at
every zoom, so there is ~4.5x headroom; if a reading *fails*, that measurement is the story's
finding and gets reported, not worked around. First suspect is 5.4's cap-slab count
(`deferred-work.md:631-634`), **not** the blend.

## 8. AC7 + AC12 — confirm by eye, in words

📋 **State plainly:**

- dwarves **slide** between tiles rather than snapping;
- torch and campfire pools **breathe**, each on its own rhythm, not in unison;
- chips and stone rubble **sit at the dug tiles** and stay there;
- nothing else about the beat-1 frame changed.

**Two things already known, so they are not surprises:** the dig face is **0.30% of the boot
frame** and reads at *working zoom*, not at the boot vista (artifact line 8) — so judge it zoomed
in. And the flicker is deliberately subtle: measured torch ±7.0%, campfire ±11.0% (14% / 22%
peak-to-peak) at 1.7 Hz / 0.9 Hz. If it reads too faint to you, that is a **taste call, not a
bug** — the amplitude column at `crates/gui/src/appearance.rs:61` and `:68` is the single-number
knob, and widening it will not endanger the capture range checks.

**A dwarf never carries a stone here, by design.** 6.1 places no stockpile, so no haul job is ever
created and no carrying occurs — UX-DR14's carried-stone clause is formally not delivered in M2
(your ruling, 2026-08-16). Do not read its absence as a defect.

---

## What comes back to me

The two `motion:` lines, the two range-check lines, the self-test result, the two fps figures, and
your sentences from §5 and §8. I check them against ACs 7, 12, 13, 15 and 16, tick what they close,
and put the story in front of you for Task 9 — **the sign-off only you can give.**
