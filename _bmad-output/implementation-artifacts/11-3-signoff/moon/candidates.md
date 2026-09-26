# 11.3 Task 9d — moon candidates

Rendered 2026-09-25 in an ISOLATED `git worktree` of `71711bb` (the shared tree was never edited).
The probe reads two env overrides on `night_lighting()`: `directional_illuminance` (the moon)
and `ambient_brightness`. **Control:** the probe with no override set is `cmp`-identical to the
unpatched build at `--clock 22`. Boot framing, all effects on, one fresh `simd --pause-at 120`
per capture:

```
gui <port> --headless --static-world --lights-steady --subdiv 4 --frames 160 --clock <h> --capture <png>
```

| candidate | moon lux | night ambient | 22:00 ground median | 19:30 ground median | capture exit |
| --- | ---: | ---: | ---: | ---: | --- |
| m7000 (today) | 7,000 | 1,500 | 69 | 62 | 0 / 0 |
| m3000 | 3,000 | 1,500 | 62 | 57 | 0 / 0 |
| m1500 | 1,500 | 1,500 | 58 | 54 | 0 / **101** |
| m750 | 750 | 1,500 | 54 | 52 | **101** / **101** |
| m750a1000 | 750 | 1,000 | 48 | 44 | **101** / **101** |

A **101** is the capture band's ground-median floor (55), which was calibrated on the 7,000-lux
night. That floor must not choose the look. After the pick, it re-baselines with AC2's control and
the boot-calibrated rendered guards. Every PNG was still written.

Side by side: `moon-contact-sheet-71711bb.png`. Each row is a candidate; the left column is 22:00
and the right is 19:30.

For reference, the noon sun is 12,000 lux. Today's moon is 58% of that; m3000 is 25%, m1500 13%
and m750 6%.

## Picked: m750 (Wolf, 2026-09-25, *"1 750"*)

The approved night was `approved-night-750-8d16616.png` (now `superseded-night-750-no-haze-8d16616.png`, see below): boot, the same as `--clock 22`, two boots
`cmp`-identical, ground median 54.

**The haze is now nearly inert at night.** `haze-on-off-750-8d16616.png` (top: haze on; bottom:
haze off). The haze's lift came from scattering the key light, and the key is now 9× dimmer. The
11.2 guard reads the far-ridge median 46 → 43 with haze on (it *darkens*; the bar is a rise of ≥10)
and a contrast fall of 7.9% (the bar is ≥15%). Ground median: 54 with haze on, 59 with it off.

## Night haze gain: picked 14 (Wolf, 2026-09-26, *"1 14"*)

Wolf's intent: *"whole scene could be darker like in 750 but ofc Moon is in real life bright and
have visible lightbeam ... haze needs to still be visible during nights"*. So the key stays at 750,
and the haze alone scatters it harder: `FogVolume::light_intensity` follows the hourly table, 14 at
night and 1.0 by day (11.2's haze). Density, ramp and volume are untouched.

`haze-night-gain-sheet-9e9f6d8-plus-gain.png` (haze off / 5 / 9.33 / 14). Night haze guard, far
ridge median and contrast fall: gain 5 46 → 53, 19.8% (fails the +10 bar); 9.33 46 → 61, 26.5%
(the 11.2 strength exactly, 7,000/750); **14: 46 → 69, 31.7%**. Sky and stars are unchanged at every gain.

**The new approved night is `approved-night-750-haze14-b4a4b9d.png`:** boot, `cmp`-identical to
`--clock 22`. **Ground median 64** (54 without the gain, 69 on the old 7,000-lux night): the veil
lifts the median most of the way back, while the lit surfaces stay at 750.

**Moon disc (Wolf: *"2 yes"*).** An unlit sphere about 2° across, 640 m back along the installed
moonlight. It is hidden by day and fades with the sky like the stars. At boot it sits above the
frame. `moon-disc-and-shafts-2000-b4a4b9d.png` is 20:00 from 200 m out (top: haze on, showing the
shafts under the ridge; bottom: haze off).

**Box outline lines (#120, commented).** From outside the volume, the box's edges draw bright
lines, and the gain makes them much stronger. A side-wall density fade did not move them (135/140
luma with or without it, 13/14 with haze off), so it was reverted; see
`box-edge-side-fade-no-effect.png` (top: before, bottom: with the fade).
