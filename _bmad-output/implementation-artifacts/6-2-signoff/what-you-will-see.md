# Story 6.2 sign-off artifact — "here is what you will see" (UX-DR22 opening half)

**Status: WRITTEN HALVES DRAFTED 2026-08-18, AWAITING WOLF.** Part (a), the before capture, is
**owed** — it needs one vehicle session and no code (the shipped 6.1 binary takes it). Until Wolf
approves this file as a whole, AC1 is unmet, no implementation commit may land and no Codex handoff
may be issued.

## (a) The before capture — OWED, one command on the vehicle

```cmd
gui.exe 7451 --capture 6-2-before.png --frames 1500
```

**1500 frames, not 600.** `simd` ticks at 10 Hz and gingerspice runs >143 fps, so 600 frames is
~4.4 s ≈ 44 ticks against the instrument's ≥100-tick floor: it panics *before* writing a PNG. That
cost a command on 6.1's first vehicle run and is not being paid twice.

What it shows: the camp exactly as it is today — campfire and four torches lit and breathing, five
dwarves wandering, and **no light of their own**. The dwarves are dark shapes moving through other
people's light. That is the "before" this story changes.

## (b) What this story adds

**One thing: each dwarf carries a lantern, and the light goes with them.**

A warm pool travels with each of the five dwarves, lighting the terrain they walk across — snow
brightening ahead of them and falling dark behind. Where two dwarves pass near each other their
pools overlap and brighten. Because 6.1 made the blend the sole writer of a projected entity's
position, the pool **slides** with the dwarf between ticks rather than jumping tile to tile.

The aim: the dwarves stop being dark shapes in a lit camp and become **the warm thing moving through
the cold**. It is also the lighting system's hardest case — every light before this one has been
nailed to a fixed tile.

## (c) What you will NOT see

Each line needs your ruling.

1. **No fuel, no pickup, no drop, no lantern economy.** Every dwarf has one, permanently, and it
   cannot be lost or lit or extinguished. There is nothing to manage.
2. **The dwarves themselves do not glow.** The lantern is a light source in the world; the dwarf cube
   keeps its own flat material. You will see what the lantern *lights*, not a glowing dwarf.
3. **No lantern glyph in the TUI.** Every dwarf carries one uniformly, so a glyph would distinguish
   nothing. The field still reaches both clients through the shared mirror.
4. **No visible lamp object.** There is no lantern model, no carried prop — dwarves remain scaled
   cubes. The light appears to come from the dwarf itself.
5. **No z-slicing (7.1) and no commands from `gui` (8.x).**
6. **RAISE THIS ONE EXPLICITLY — the camp may read brighter than the frame you approved at 5.4.**
   This adds five lights of 11,000,000 lumens each, moving around inside a camp that already has a
   campfire and four torches. 5.4's approved frame measured a ground-median luminance of **123**
   against a hard ceiling of **180**, and that band exists because night snow must stay midtone
   while only emissive sources approach white. If the lanterns push the camp past that ceiling, the
   fix is the **lantern intensity**, and the band is never widened to make the capture pass. Your
   call at the live viewing is whether the camp still reads as a cold night with warm fires in it.
7. **The lantern flicker is deliberately gentle.** Torch and campfire were raised to ±30% and ±40%
   at 6.1's viewing because they read as static. The lantern row was **left at ±5%** — a carried
   lamp is steadier than an open fire. If it reads as dead rather than steady, that is one number.

## Why this story is smaller than the epic says

The epic claims 6.2 is where `LightKind` gains its `Lantern` variant, "the last piece of M2's
sanctioned wire diff". **That is stale.** `protocol::LightKind::Lantern`, `sim_core::LightKind::
Lantern` and `Entity.light` all already exist and the bridge already translates between them — the
wire diff was spent at 5.1. This story changes two hardcoded `light: None` values and makes the
world say every dwarf carries one. `gui` needs approximately no change at all.
