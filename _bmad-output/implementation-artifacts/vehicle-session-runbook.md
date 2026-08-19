# The vehicle session — three stories, one sitting

**Status 2026-08-19.** Three stories are `in-progress` and every one of them is blocked on the same
thing: a window. Nothing else is outstanding in any of them — all three are reviewed, patched and
green.

**They share a binary.** Story 7.1's branch is stacked on 6.2, which is stacked on 6.1, so one
`gui.exe` built from `7-1-slice-into-the-mountain` contains all three features. One build, one
daemon, one sitting.

| Story | What is owed | Runbook |
| --- | --- | --- |
| 6.1 the world moves | Task 6 (AC 7, 12, 13, 15, 16) + your Task 9 sign-off | `6-1-signoff/task-6-vehicle-runbook.md` |
| 6.2 lanterns in the dark | Task 6 (AC 9, 13, 14, 15) + your Task 9 sign-off | `6-2-signoff/task-6-vehicle-runbook.md` |
| 7.1 slice into the mountain | Task 5 (AC 8, 9, 10, 12, 14) + your Task 8 sign-off | `7-1-signoff/task-5-vehicle-runbook.md` |

**No devpod can open a window** — measured at 5.3, both fallbacks walked to the end. The vehicle is
**gingerspice: cross-compiled `gui.exe` on native Windows, `simd` in WSL, localhost, native NVIDIA
Vulkan.**

**Never fake any of it.** If a step cannot run, report the step and stop. A missing measurement is a
finding; an invented one is a lie the whole gate rests on.

---

## Build once (WSL)

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p simd -p tui
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build -p gui --release --target x86_64-pc-windows-gnu
```

`gui.exe` lands at `target/x86_64-pc-windows-gnu/release/gui.exe`. Copy it to the Windows side.

## Start the daemon once (WSL, leave it running all session)

```bash
./target/debug/simd 7451
```

Port is positional. The seed is fixed in the binary, so this is the same world every measurement in
all three stories was taken against.

## Suggested order

1. **6.1 first** — it designates the dig site, and both later stories want that dig to exist.
2. **6.2 next** — same camp, lanterns moving, no new world state needed.
3. **7.1 last** — it needs the dig from step 1 to have something to slice down into.

One `simd` and one `gui.exe` serve all three. The only reason to restart `gui.exe` is to change its
launch flags.

---

## Two things that changed on 2026-08-19 and affect all three

**1. There is a new line of text on screen.** Story 7.1 adds a permanent level readout at the
top-left — `Slice: z 31/31 — surface`. It is **not** suppressed in capture mode, so it appears in
every PNG this session produces, including 6.1's and 6.2's. This is expected, not a regression. It
was verified not to disturb the inherited 5.4 range checks: it is blue-dominant so it cannot count
as a warm lantern pixel, and it sits outside the ground-luminance sample region.

**2. The startup draw-set line now names its level.** It reads

```
projected 53365 terrain cubes at z 31
```

The `53365` figure is unchanged at full depth — the oracle survives slicing intact — but the line is
longer than the one older recipes quote. If you are matching it by eye, match the prefix.

---

## What comes back to me

The pasted instrument lines, the fps figures, and your sentences. I check them against the ACs they
close, tick those boxes, and put each story in front of you for the sign-off only you can give.
